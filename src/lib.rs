use nih_plug::prelude::*;
use nih_plug_egui::create_egui_editor; // For UI
use std::sync::Arc;

// Make sure our modules are declared
mod dsp;
mod sofa;
mod autoeq_parser; // Assuming this exists from previous step

use crate::dsp::convolution::{ConvolutionEngine, ConvolutionPath};
use crate::dsp::parametric_eq::{FilterType, StereoParametricEQ}; // Assuming StereoParametricEQ
use crate::sofa::loader::{MySofa, SofaError};
// use crate::autoeq_parser::{parse_autoeq_file_content, AutoEqProfile}; // Will be used when params work

const DEFAULT_SPEAKER_RADIUS: f32 = 1.0; // Default radius in meters
const NUM_EQ_BANDS: usize = 10;

// --- Intended Parameters Structure (for UI design, known to have compile issues) ---
#[derive(Params)]
struct OpenHeadstageParams {
    #[persist = "output_gain_db"]
    #[param(name = "Output Gain", unit = "dB", range = (-30.0..=0.0), format = "{:.2} dB")]
    pub output_gain: FloatParam,

    #[persist = "speaker_az_left"]
    #[param(name = "L Azimuth", unit = "°", range = (-90.0..=90.0), format = "{:.1}°")]
    pub speaker_azimuth_left: FloatParam,
    #[persist = "speaker_el_left"]
    #[param(name = "L Elevation", unit = "°", range = (-45.0..=45.0), format = "{:.1}°")]
    pub speaker_elevation_left: FloatParam,

    #[persist = "speaker_az_right"]
    #[param(name = "R Azimuth", unit = "°", range = (-90.0..=90.0), format = "{:.1}°")]
    pub speaker_azimuth_right: FloatParam,
    #[persist = "speaker_el_right"]
    #[param(name = "R Elevation", unit = "°", range = (-45.0..=45.0), format = "{:.1}°")]
    pub speaker_elevation_right: FloatParam,

    #[persist = "sofa_path"]
    #[param(name = "SOFA File")]
    pub sofa_file_path: StringParam,

    #[persist = "eq_enable"]
    #[param(name = "Enable EQ")]
    pub eq_enable: BoolParam,

    // EQ Bands (example for one band, repeat for NUM_EQ_BANDS)
    // For a real implementation, a macro or helper might generate these
    #[persist = "eq_band1_enable"] #[param(name = "EQ B1 Enable")] pub eq_band1_enable: BoolParam,
    #[persist = "eq_band1_type"] #[param(name = "EQ B1 Type", enums(FilterTypeParamEnum))] pub eq_band1_type: EnumParam<FilterTypeParamEnum>,
    #[persist = "eq_band1_fc"] #[param(name = "EQ B1 Fc", unit = "Hz", range = (20.0..=20000.0), format = "{:.0} Hz")] pub eq_band1_fc: FloatParam,
    #[persist = "eq_band1_q"] #[param(name = "EQ B1 Q", range = (0.1..=10.0), format = "{:.2}")] pub eq_band1_q: FloatParam,
    #[persist = "eq_band1_gain"] #[param(name = "EQ B1 Gain", unit = "dB", range = (-24.0..=24.0), format = "{:.1} dB")] pub eq_band1_gain: FloatParam,
    // ... (conceptually, params for bands 2-10 would follow)
}

// Enum for FilterType parameter in UI
#[derive(Debug, PartialEq, Eq, Clone, Copy, Enum)]
enum FilterTypeParamEnum { Peak, LowShelf, HighShelf }

impl From<FilterTypeParamEnum> for FilterType {
    fn from(item: FilterTypeParamEnum) -> Self {
        match item {
            FilterTypeParamEnum::Peak => FilterType::Peak,
            FilterTypeParamEnum::LowShelf => FilterType::LowShelf,
            FilterTypeParamEnum::HighShelf => FilterType::HighShelf,
        }
    }
}


impl Default for OpenHeadstageParams {
    fn default() -> Self {
        Self {
            output_gain: FloatParam::new("Output Gain", util::db_to_gain(0.0), FloatRange::Linear { min: util::db_to_gain(-30.0), max: util::db_to_gain(0.0) })
                .with_smoother(SmoothingStyle::Logarithmic(50.0)).with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2)).with_string_to_value(formatters::s2v_f32_gain_to_db()),
            speaker_azimuth_left: FloatParam::new("L Azimuth", -30.0, FloatRange::Linear { min: -90.0, max: 90.0 }).with_smoother(SmoothingStyle::Linear(50.0)).with_unit("°"),
            speaker_elevation_left: FloatParam::new("L Elevation", 0.0, FloatRange::Linear { min: -45.0, max: 45.0 }).with_smoother(SmoothingStyle::Linear(50.0)).with_unit("°"),
            speaker_azimuth_right: FloatParam::new("R Azimuth", 30.0, FloatRange::Linear { min: -90.0, max: 90.0 }).with_smoother(SmoothingStyle::Linear(50.0)).with_unit("°"),
            speaker_elevation_right: FloatParam::new("R Elevation", 0.0, FloatRange::Linear { min: -45.0, max: 45.0 }).with_smoother(SmoothingStyle::Linear(50.0)).with_unit("°"),
            sofa_file_path: StringParam::new("SOFA File", String::new()),
            eq_enable: BoolParam::new("Enable EQ", false),
            eq_band1_enable: BoolParam::new("EQ B1 Enable", false),
            eq_band1_type: EnumParam::new("EQ B1 Type", FilterTypeParamEnum::Peak),
            eq_band1_fc: FloatParam::new("EQ B1 Fc", 1000.0, FloatRange::Skewed { min: 20.0, max: 20000.0, factor: FloatRange::skew_factor_logc(1000.0) }).with_unit(" Hz"),
            eq_band1_q: FloatParam::new("EQ B1 Q", 1.0, FloatRange::Linear { min: 0.1, max: 10.0 }),
            eq_band1_gain: FloatParam::new("EQ B1 Gain", 0.0, FloatRange::Linear { min: -24.0, max: 24.0 }).with_unit(" dB"),
            // ... (default initializers for other bands)
        }
    }
}
// --- End of Intended Parameters Structure ---


// Main plugin struct
struct OpenHeadstagePlugin {
    // Using Arc<()> for params as a workaround for derive(Params) issues.
    params: Arc<()>, // This will be Arc<OpenHeadstageParams> if derive macro worked
    editor_state: Arc<EguiState>, // For nih-plug-egui

    convolution_engine: ConvolutionEngine,
    sofa_loader: Option<MySofa>,
    // parametric_eq: StereoParametricEQ, // Would be initialized in default()
    current_sample_rate: f32,
    // Fields for tracking last parameter values to trigger updates would be here
}

impl Default for OpenHeadstagePlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(()), // Arc::new(OpenHeadstageParams::default()) if derive worked
            editor_state: EguiState::from_size(400, 300), // Default UI size
            convolution_engine: ConvolutionEngine::new(),
            sofa_loader: None,
            // parametric_eq: StereoParametricEQ::new(NUM_EQ_BANDS, 44100.0),
            current_sample_rate: 44100.0,
        }
    }
}

// Placeholder for methods that would interact with full params
impl OpenHeadstagePlugin {
    fn _load_sofa_file(&mut self, _path_str: &str) { /* ... */ }
    fn _update_hrirs(&mut self) { /* ... */ }
    // fn _update_eq_bands(&mut self) { /* ... */ }
}


impl Plugin for OpenHeadstagePlugin {
    const NAME: &'static str = "Open Headstage";
    const VENDOR: &'static str = "Open Source Community";
    const URL: &'static str = "https://example.com";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    type Params = (); // Using () due to derive(Params) issues on OpenHeadstageParams

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&self, async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // Accessing the *intended* params struct conceptually for UI design.
        // This part will not compile if OpenHeadstageParams itself causes issues.
        // To make it "compile" (ignoring derive errors), one might pass Arc::new(())
        // or a dummy Arc<ActualParams::default()> to create_egui_editor.
        // For this subtask, we write the UI logic as if params were working.

        // let params_for_ui = self.params.clone(); // This would be Arc<OpenHeadstageParams>
                                                 // Since self.params is Arc<()>, this won't work directly.
                                                 // We'll use a placeholder for UI design purposes.
        let placeholder_params = Arc::new(OpenHeadstageParams::default());


        create_egui_editor(
            self.editor_state.clone(),
            (), // User state for the editor (can be simple struct)
            |_, _| {}, // build closure (used for egui_glow context, not needed for basic egui)
            move |ui, setter, _uistate| { // draw closure
                ui.heading("Open Headstage Controls");
                ui.separator();

                // Conceptual UI for SOFA File Loading
                // In a real scenario, `setter.params()` would provide `Arc<OpenHeadstageParams>`
                // For now, using `placeholder_params` to allow UI code to be written.
                let params = &placeholder_params; // setter.params();

                ui.label(format!("SOFA File: {}", params.sofa_file_path.value()));
                if ui.button("Load SOFA File").clicked() {
                    // File dialog logic (conceptual, needs async handling)
                    // This would typically be handled via tasks or commands sent to plugin
                    // For now, just log or show a placeholder.
                    // let task = rfd::FileDialog::new().pick_file();
                    // if let Some(path) = task {
                    //     setter.begin_set_parameter(&params.sofa_file_path);
                    //     setter.set_parameter_normalized_str(&params.sofa_file_path, path.to_string_lossy().as_ref());
                    //     setter.end_set_parameter(&params.sofa_file_path);
                    // }
                    nih_log!("SOFA File load button clicked (dialog not implemented in this step).");
                }
                ui.separator();

                // Output Gain
                ui.add(widgets::ParamSlider::for_param(&params.output_gain, setter).with_label("Output Gain"));
                ui.separator();

                // Speaker Angles
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.label("Left Speaker");
                        ui.add(widgets::ParamSlider::for_param(&params.speaker_azimuth_left, setter).with_label("Azimuth (L)"));
                        ui.add(widgets::ParamSlider::for_param(&params.speaker_elevation_left, setter).with_label("Elevation (L)"));
                    });
                    ui.group(|ui| {
                        ui.label("Right Speaker");
                        ui.add(widgets::ParamSlider::for_param(&params.speaker_azimuth_right, setter).with_label("Azimuth (R)"));
                        ui.add(widgets::ParamSlider::for_param(&params.speaker_elevation_right, setter).with_label("Elevation (R)"));
                    });
                });
                ui.separator();

                // Parametric EQ Section
                ui.horizontal(|ui| {
                    ui.label("Headphone EQ");
                    ui.add(widgets::ParamCheckbox::for_param(&params.eq_enable, setter));
                });

                // Example for one band (conceptual)
                // In a real UI, this would loop NUM_EQ_BANDS or use a helper.
                ui.collapsing("EQ Band 1", |ui| {
                    ui.add(widgets::ParamCheckbox::for_param(&params.eq_band1_enable, setter).with_label("Enable"));
                    // Type selection would need a ComboBox or Radio Buttons for EnumParam
                    // ui.add(ParamEnumComboBox::for_param(&params.eq_band1_type, setter));
                    ui.add(widgets::ParamSlider::for_param(&params.eq_band1_fc, setter).with_label("Fc"));
                    ui.add(widgets::ParamSlider::for_param(&params.eq_band1_q, setter).with_label("Q"));
                    ui.add(widgets::ParamSlider::for_param(&params.eq_band1_gain, setter).with_label("Gain"));
                });
                // ... (imagine more bands or a scroll area)
            },
        )
    }


    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.current_sample_rate = buffer_config.sample_rate;
        nih_log!("Plugin initialized. Sample rate: {} Hz", self.current_sample_rate);
        // self.parametric_eq = StereoParametricEQ::new(NUM_EQ_BANDS, self.current_sample_rate);

        // Placeholder: Load default IRs as parameter system is stubbed
        self.convolution_engine.set_ir(ConvolutionPath::LSL, vec![1.0]);
        self.convolution_engine.set_ir(ConvolutionPath::LSR, vec![0.0]);
        self.convolution_engine.set_ir(ConvolutionPath::RSL, vec![0.0]);
        self.convolution_engine.set_ir(ConvolutionPath::RSR, vec![1.0]);
        true
    }

    fn reset(&mut self) {
        nih_log!("Plugin reset.");
        // self.parametric_eq.reset_all_bands_state();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let block_size = buffer.samples();
        let mut left_input_temp = vec![0.0f32; block_size];
        let mut right_input_temp = vec![0.0f32; block_size];
        let mut left_output_temp = vec![0.0f32; block_size];
        let mut right_output_temp = vec![0.0f32; block_size];

        for (i, mut frame) in buffer.iter_samples().enumerate() {
            left_input_temp[i] = *frame.get_mut(0).unwrap_or(&mut 0.0);
            if frame.len() > 1 {
                right_input_temp[i] = *frame.get_mut(1).unwrap_or(&mut 0.0);
            }
        }

        self.convolution_engine.process_block(
            &left_input_temp,
            &right_input_temp,
            &mut left_output_temp,
            &mut right_output_temp,
        );

        // TODO: Apply EQ (self.parametric_eq.process_block) here if enabled
        // TODO: Apply output gain from params here if params were working

        for (i, mut frame) in buffer.iter_samples().enumerate() {
            if let Some(s) = frame.get_mut(0) { *s = left_output_temp[i]; }
            if frame.len() > 1 {
                if let Some(s) = frame.get_mut(1) { *s = right_output_temp[i]; }
            }
        }

        ProcessStatus::Normal
    }
}

nih_export_clap!(OpenHeadstagePlugin);
