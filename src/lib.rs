use nih_plug::prelude::*;
// use nih_plug_egui::create_egui_editor; // Commented out: UI disabled for now
use std::sync::Arc;

// Make sure our modules are declared (minimal for now)
// mod dsp;
// mod sofa;
// mod autoeq_parser;

// use crate::dsp::convolution::{ConvolutionEngine, ConvolutionPath}; // Commented out
// use crate::dsp::parametric_eq::{FilterType}; // StereoParametricEQ also commented
// use crate::sofa::loader::{MySofa}; // SofaError also commented
// use crate::autoeq_parser::{parse_autoeq_file_content, AutoEqProfile};

// const DEFAULT_SPEAKER_RADIUS: f32 = 1.0; // Commented out
// const NUM_EQ_BANDS: usize = 10; // Commented out

// --- Parameters Structure aligned with nih-plug rev a33569b... ---
#[derive(Params)]
struct OpenHeadstageParams {
    #[id = "out_gain"]
    pub output_gain: FloatParam,

    #[id = "az_l"]
    pub speaker_azimuth_left: FloatParam,
    #[id = "el_l"]
    pub speaker_elevation_left: FloatParam,

    #[id = "az_r"]
    pub speaker_azimuth_right: FloatParam,
    #[id = "el_r"]
    pub speaker_elevation_right: FloatParam,

    // #[id = "sofa_path"]
    // pub sofa_file_path: StringParam, // Commented out

    #[id = "eq_enable"]
    pub eq_enable: BoolParam,

    // EQ Bands (example for one band, repeat for NUM_EQ_BANDS)
    #[id = "eq_b1_en"] pub eq_band1_enable: BoolParam,
    // #[id = "eq_b1_type"] pub eq_band1_type: EnumParam<FilterTypeParamEnum>, // Commented out
    #[id = "eq_b1_fc"] pub eq_band1_fc: FloatParam,
    #[id = "eq_b1_q"] pub eq_band1_q: FloatParam,
    #[id = "eq_b1_gain"] pub eq_band1_gain: FloatParam,
    // ... (conceptually, params for bands 2-10 would follow with unique IDs)
}

/* // Enum and From impl commented out as eq_band1_type is commented
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
*/

impl Default for OpenHeadstageParams {
    fn default() -> Self {
        Self {
            output_gain: FloatParam::new(
                "Output Gain",
                util::db_to_gain(0.0),
                FloatRange::Linear { min: util::db_to_gain(-30.0), max: util::db_to_gain(0.0) },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
                .with_value_to_string(formatters::v2s_f32_gain_to_db(2)).with_string_to_value(formatters::s2v_f32_gain_to_db()),
            speaker_azimuth_left: FloatParam::new("L Azimuth", -30.0, FloatRange::Linear { min: -90.0, max: 90.0 }).with_smoother(SmoothingStyle::Linear(50.0)).with_unit("°"),
            speaker_elevation_left: FloatParam::new("L Elevation", 0.0, FloatRange::Linear { min: -45.0, max: 45.0 }).with_smoother(SmoothingStyle::Linear(50.0)).with_unit("°"),
            speaker_azimuth_right: FloatParam::new("R Azimuth", 30.0, FloatRange::Linear { min: -90.0, max: 90.0 }).with_smoother(SmoothingStyle::Linear(50.0)).with_unit("°"),
            speaker_elevation_right: FloatParam::new("R Elevation", 0.0, FloatRange::Linear { min: -45.0, max: 45.0 }).with_smoother(SmoothingStyle::Linear(50.0)).with_unit("°"),
            // sofa_file_path: StringParam::new("SOFA File", String::new()), // Commented out
            eq_enable: BoolParam::new("Enable EQ", false),
            eq_band1_enable: BoolParam::new("EQ B1 Enable", false),
            // eq_band1_type: EnumParam::new("EQ B1 Type", FilterTypeParamEnum::Peak), // Commented out
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
    params: Arc<OpenHeadstageParams>,
    // editor_state: Arc<EguiState>, // Commented out
    // convolution_engine: ConvolutionEngine, // Commented out
    // sofa_loader: Option<MySofa>, // Commented out
    // parametric_eq: StereoParametricEQ, // Commented out
    // current_sample_rate: f32, // Commented out
}

impl Default for OpenHeadstagePlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(OpenHeadstageParams::default()),
            // editor_state: EguiState::from_size(400, 300), // Commented out
            // convolution_engine: ConvolutionEngine::new(), // Commented out
            // sofa_loader: None, // Commented out
            // parametric_eq: StereoParametricEQ::new(NUM_EQ_BANDS, 44100.0), // Commented out
            // current_sample_rate: 44100.0, // Commented out
        }
    }
}

/* // Commented out helper methods block
impl OpenHeadstagePlugin {
    fn _load_sofa_file(&mut self, _path_str: &str) { /* ... */ }
    fn _update_hrirs(&mut self) { /* ... */ }
    // fn _update_eq_bands(&mut self) { /* ... */ }
}
*/

impl Plugin for OpenHeadstagePlugin {
    const NAME: &'static str = "Open Headstage";
    const VENDOR: &'static str = "Open Source Community"; // Changed to generic
    const URL: &'static str = "http://example.com"; // Changed to generic
    const EMAIL: &'static str = "info@example.com"; // Changed to generic
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    // If derive(Params) works, this associated type is automatically OpenHeadstageParams

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    // const MIDI_OUTPUT: MidiConfig = MidiConfig::None; // Already MidiConfig::None by default
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    /*
    fn editor(&self, async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // The `#[param]` attributes are gone, so direct UI generation from them
        // might not work in the same way as with newer nih-plug versions.
        // The `setter` object passed to the draw closure is the primary way to interact
        // with parameters in older nih-plug versions for egui.
        // We will use `self.params.clone()` which should now be Arc<OpenHeadstageParams>.

        let params_arc = self.params.clone();
        // let editor_state_arc = self.editor_state.clone(); // Commented out

        create_egui_editor(
            // editor_state_arc, // Commented out
            self.editor_state.clone(), // This will cause error if editor_state is removed from struct
            (), // Userstate for the editor (can be simple struct)
            |_, _| {}, // build closure (not used here)
            move |ui, setter, _uistate| { // draw closure
                ui.heading("Open Headstage Controls");
                ui.separator();

                // Accessing parameters via the `setter` or the cloned `params_arc`.
                // For displaying current value, `params_arc.sofa_file_path.value()` is fine.
                // For modifying, `setter` must be used.
                // The UI widgets ParamSlider, ParamCheckbox might need to be adapted if the
                // `for_param` constructor isn't available or works differently.
                // Older nih_plug_egui often used `ui.param_slider(setter, &params_arc.my_param, ...)`

                // ui.label(format!("SOFA File: {}", params_arc.sofa_file_path.value())); // Commented out
                if ui.button("Load SOFA File").clicked() {
                    // File dialog logic (conceptual, needs async handling)
                    // This would typically be handled via tasks or commands sent to plugin
                    // For now, just log or show a placeholder.
                    // let task = rfd::FileDialog::new().pick_file();
                    // if let Some(path) = task {
                    //     setter.begin_set_parameter(&params.sofa_file_path);
                    //     setter.set_parameter(&params_arc.sofa_file_path, path.to_string_lossy().into_owned());
                    // }
                    nih_log!("SOFA File load button clicked (dialog not implemented in this step).");
                }
                ui.separator();

                // Output Gain - Assuming ParamSlider::for_param still exists or similar API
                // If not, this will need to be e.g. setter.param_slider(&params_arc.output_gain, "Output Gain")
                // For now, keeping existing widget calls and will adapt if they fail.
                // ui.add(widgets::ParamSlider::for_param(&params_arc.output_gain, setter).with_label("Output Gain")); // Commented out
                ui.separator();

                // Speaker Angles
                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.label("Left Speaker");
                        // ui.add(widgets::ParamSlider::for_param(&params_arc.speaker_azimuth_left, setter).with_label("Azimuth (L)")); // Commented out
                        // ui.add(widgets::ParamSlider::for_param(&params_arc.speaker_elevation_left, setter).with_label("Elevation (L)")); // Commented out
                    });
                    ui.group(|ui| {
                        ui.label("Right Speaker");
                        // ui.add(widgets::ParamSlider::for_param(&params_arc.speaker_azimuth_right, setter).with_label("Azimuth (R)")); // Commented out
                        // ui.add(widgets::ParamSlider::for_param(&params_arc.speaker_elevation_right, setter).with_label("Elevation (R)")); // Commented out
                    });
                });
                ui.separator();

                // Parametric EQ Section
                ui.horizontal(|ui| {
                    ui.label("Headphone EQ");
                    // ui.add(widgets::ParamCheckbox::for_param(&params_arc.eq_enable, setter)); // Commented out
                });

                // Example for one band (conceptual)
                ui.collapsing("EQ Band 1", |ui| {
                    // ui.add(widgets::ParamCheckbox::for_param(&params_arc.eq_band1_enable, setter).with_label("Enable")); // Commented out
                    // Type selection: ui.enum_param_buttons(setter, &params_arc.eq_band1_type, "Type") or similar
                    // ui.add(widgets::ParamSlider::for_param(&params_arc.eq_band1_fc, setter).with_label("Fc")); // Commented out
                    // ui.add(widgets::ParamSlider::for_param(&params_arc.eq_band1_q, setter).with_label("Q")); // Commented out
                    // ui.add(widgets::ParamSlider::for_param(&params_arc.eq_band1_gain, setter).with_label("Gain")); // Commented out
                });
            },
        )
    }
    */
    // Temporarily remove editor to focus on Params derive
    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        None
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig, // Keep buffer_config to avoid unused var warning if current_sample_rate is re-added
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // self.current_sample_rate = buffer_config.sample_rate; // Commented out
        nih_log!("Plugin initialized.");
        // No complex initialization needed for minimal example
        true
    }

    fn reset(&mut self) {
        nih_log!("Plugin reset.");
        // No state to reset in minimal example
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Simplified gain processing like in nih-plug Gain example
        for channel_samples in buffer.iter_samples() {
            // .next() is not needed for older nih_plug param smoothing; direct value access.
            // let gain = self.params.output_gain.smoothed.next();
            let gain = self.params.output_gain.value; // Direct value access for older nih-plug
            for sample in channel_samples {
                *sample *= gain;
            }
        }
        ProcessStatus::Normal
    }
}

nih_export_clap!(OpenHeadstagePlugin);
nih_export_vst3!(OpenHeadstagePlugin);
