use nih_plug::prelude::*;
// use nih_plug_egui::create_egui_editor; // Commented out: UI disabled for now
use std::sync::Arc;

// Make sure our modules are declared (minimal for now)
mod dsp;
mod sofa;
// mod autoeq_parser;

use crate::dsp::convolution::{ConvolutionEngine, ConvolutionPath}; // Commented out
use crate::dsp::parametric_eq::{FilterType, StereoParametricEQ}; // StereoParametricEQ also commented
use crate::sofa::loader::{MySofa, SofaError}; // SofaError also commented
// use crate::autoeq_parser::{parse_autoeq_file_content, AutoEqProfile};

const DEFAULT_SPEAKER_RADIUS: f32 = 1.0; // Commented out
const NUM_EQ_BANDS: usize = 10; // Commented out
const DEFAULT_IR_LEN: usize = 512; // Default impulse response length

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

    #[id = "sofa_path"] #[persist = "sofa_path"]
    pub sofa_file_path: StringParam,

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
            sofa_file_path: StringParam::new("SOFA File", String::new()), // Commented out
            eq_enable: BoolParam::new("Enable EQ", false),
            eq_band1_enable: BoolParam::new("EQ B1 Enable", false),
            // eq_band1_type: EnumParam::new("EQ B1 Type", FilterTypeParamEnum::Peak), // Commented out
            eq_band1_fc: FloatParam::new("EQ B1 Fc", 1000.0, FloatRange::Skewed { min: 20.0, max: 20000.0, factor: FloatRange::skew_factor(-2.0) }).with_unit(" Hz"),
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
    convolution_engine: ConvolutionEngine, // Commented out
    sofa_loader: Option<MySofa>, // Commented out
    parametric_eq: StereoParametricEQ, // Commented out
    current_sample_rate: f32, // Commented out
}

impl Default for OpenHeadstagePlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(OpenHeadstageParams::default()),
            // editor_state: EguiState::from_size(400, 300), // Commented out
            convolution_engine: ConvolutionEngine::new(), // Commented out
            sofa_loader: None, // Commented out
            parametric_eq: StereoParametricEQ::new(NUM_EQ_BANDS, 44100.0), // Commented out
            current_sample_rate: 44100.0, // Default, will be updated in initialize
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

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    // const MIDI_OUTPUT: MidiConfig = MidiConfig::None; // Already MidiConfig::None by default
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    // Temporarily remove editor to focus on Params derive
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // Your editor implementation
        None // placeholder
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig, // Keep buffer_config to avoid unused var warning if current_sample_rate is re-added
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.current_sample_rate = buffer_config.sample_rate; // Commented out
        nih_log!("Plugin initialized. Sample rate: {}", self.current_sample_rate);

        let sofa_path_str = self.params.sofa_file_path.value();
        if !sofa_path_str.is_empty() {
            nih_log!("Attempting to load SOFA file from: {}", sofa_path_str);
            match MySofa::open(&sofa_path_str, self.current_sample_rate) {
                Ok(sofa_loader) => {
                    // Assuming MySofa has a method like `filter_length()`
                    // let filter_len = sofa_loader.filter_length();
                    // self.convolution_engine.set_config(self.current_sample_rate, filter_len);
                    // For now, let's assume MySofa stores HRIRs that ConvolutionEngine can access,
                    // or MySofa itself handles convolution. The task is structural integration.
                    // We might need to pass HRIR data to convolution_engine here.
                    // e.g., self.convolution_engine.load_hrirs(sofa_loader.get_hrirs());
                    nih_log!("Successfully loaded SOFA file: {}", sofa_path_str);
                    self.sofa_loader = Some(sofa_loader);
                }
                Err(e) => {
                    nih_log!("Failed to load SOFA file '{}': {:?}", sofa_path_str, e);
                    self.sofa_loader = None; // Ensure it's None if loading failed
                    // Potentially set convolution_engine to a default/passthrough state
                    // self.convolution_engine.set_config(self.current_sample_rate, DEFAULT_IR_LEN);
                }
            }
        } else {
            nih_log!("No SOFA file path configured. Skipping SOFA loading.");
            self.sofa_loader = None;
            // self.convolution_engine.set_config(self.current_sample_rate, DEFAULT_IR_LEN);
        }

        // Configure convolution engine based on SOFA loading outcome
        // This is a conceptual call. The actual method and parameters might differ.
        // It's also possible ConvolutionEngine is more tightly coupled with MySofa,
        // or MySofa itself provides the process() method.
        // For this subtask, we ensure it's part of initialization.
        let (actual_ir_l, actual_ir_r) = if let Some(loader) = &self.sofa_loader {
            // Placeholder: Assuming MySofa has methods to get HRIRs and their length
            // This part is highly dependent on MySofa and ConvolutionEngine's exact API
            // For example:
            // (loader.get_left_hrir(), loader.get_right_hrir(), loader.filter_length())
            // For now, we'll just log and use a default length for demonstration
            nih_log!("SOFA loader available, configuring convolution engine (conceptually).");
            // Let's assume MySofa has a method `get_filters()` returning (Vec<f32>, Vec<f32>)
            // and `filter_length()` returning usize. This is speculative.
            // let (hrir_l, hrir_r) = loader.get_filters();
            // let filter_len = loader.filter_length();
            // self.convolution_engine.update_ir(hrir_l, hrir_r, filter_len);
            (None, None) // Replace with actual HRIR data if available
        } else {
            nih_log!("No SOFA loader, using default/passthrough for convolution engine (conceptually).");
            (None, None) // No specific IRs
        };
        // Example of reconfiguring convolution engine:
        // self.convolution_engine.set_config(self.current_sample_rate, filter_length_from_sofa_or_default);
        // self.convolution_engine.load_impulse_responses(actual_ir_l, actual_ir_r);
        // Since ConvolutionEngine's API for this is not defined in the task,
        // we'll assume it's either configured internally or via methods not yet detailed.
        // The important part is that it's considered during initialization.
        // For now, we'll re-initialize it with the current sample rate,
        // assuming it might need it, and it can handle being re-initialized.
        self.convolution_engine = ConvolutionEngine::new(); // Or new(self.current_sample_rate) if API supports
        nih_log!("Convolution engine (re)initialized.");

        // Initialize Parametric EQ
        self.parametric_eq = StereoParametricEQ::new(NUM_EQ_BANDS, self.current_sample_rate);
        nih_log!("Parametric EQ initialized with {} bands at {} Hz.", NUM_EQ_BANDS, self.current_sample_rate);


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
        // Read speaker angle parameters
        let az_l = self.params.speaker_azimuth_left.smoothed.next();
        let el_l = self.params.speaker_elevation_left.smoothed.next();
        let az_r = self.params.speaker_azimuth_right.smoothed.next();
        let el_r = self.params.speaker_elevation_right.smoothed.next();

        // --- Conceptual HRIR Fetching & Configuration ---
        // This section outlines where HRIRs would be fetched based on angles
        // and then passed to the convolution engine.
        // The actual methods on MySofa and ConvolutionEngine might differ.

        /*
        // 1. Fetch HRIRs based on angles using the SOFA loader
        // These are conceptual types. Actual HRIR data might be Vec<f32> or similar.
        type HrirPair = (Option<ConvolutionPath>, Option<ConvolutionPath>); // Assuming ConvolutionPath holds HRIR data

        let (left_speaker_hrir_l, left_speaker_hrir_r): HrirPair =
            if let Some(loader) = &self.sofa_loader {
                // Conceptual: loader.get_stereo_hrir_for_angle(az_l, el_l, self.current_sample_rate)
                // This method would find the nearest HRIR in the SOFA file for the given angle
                // and potentially resample it if needed (MySofa might do this internally).
                // Returning a pair for stereo source (e.g. left ear, right ear impulse responses)
                // For now, we'll use placeholders.
                nih_log_once!("Fetching HRIR for L: az={}, el={}", az_l, el_l); // Log once to avoid spam
                // (Some(loader.get_hrir_l(az_l, el_l)), Some(loader.get_hrir_r(az_l, el_l)))
                (None, None) // Placeholder
            } else {
                // Fallback if no SOFA file is loaded. ConvolutionEngine might have its own defaults.
                // (self.convolution_engine.default_hrir_l(), self.convolution_engine.default_hrir_r())
                (None, None) // Placeholder
            };

        let (right_speaker_hrir_l, right_speaker_hrir_r): HrirPair =
            if let Some(loader) = &self.sofa_loader {
                nih_log_once!("Fetching HRIR for R: az={}, el={}", az_r, el_r); // Log once
                // (Some(loader.get_hrir_l(az_r, el_r)), Some(loader.get_hrir_r(az_r, el_r)))
                (None, None) // Placeholder
            } else {
                // (self.convolution_engine.default_hrir_l(), self.convolution_engine.default_hrir_r())
                (None, None) // Placeholder
            };

        // 2. Pass HRIRs to the Convolution Engine
        // This is also conceptual. The method might take paths, direct data, or be part of process().
        // self.convolution_engine.set_hrirs(
        //     left_speaker_hrir_l, left_speaker_hrir_r,
        //     right_speaker_hrir_l, right_speaker_hrir_r
        // );
        // Or, if ConvolutionEngine takes HRIRs directly in its process method:
        // self.convolution_engine.process(
        //     buffer,
        //     left_speaker_hrir_l, left_speaker_hrir_r,
        //     right_speaker_hrir_l, right_speaker_hrir_r
        // );
        */

        // --- Actual Processing ---
        // For now, only the convolution engine's process method is called if available.
        // If it's not doing anything internally (e.g. no HRIRs loaded), it might be a passthrough.
        // For this subtask, we assume HRIRs are managed internally by ConvolutionEngine
        // based on prior configuration (e.g. in initialize() or via other dedicated methods
        // that might be called when parameters like sofa_file_path or angles change).

        // The main processing loop
        let eq_enabled = self.params.eq_enable.value(); // Get non-smoothed for enable/disable check

        if eq_enabled {
            // Update EQ band coefficients - example for band 1
            // Assuming StereoParametricEQ uses 0-indexed bands
            // Also assuming a default FilterType::Peak if type enum param is not used.
            // The actual FilterType might need to come from a parameter if `eq_b1_type` was active.
            self.parametric_eq.update_band_coeffs(
                0, // Band index
                self.params.eq_band1_fc.smoothed.next(),
                self.params.eq_band1_q.smoothed.next(),
                self.params.eq_band1_gain.smoothed.next(),
                FilterType::Peak, // Assuming Peak as default, or this should be from a param
                self.params.eq_band1_enable.value(), // Use non-smoothed for enable
                self.current_sample_rate
            );
            // Conceptual: Loop for NUM_EQ_BANDS to update all bands
            // for band_idx in 0..NUM_EQ_BANDS {
            //    self.parametric_eq.update_band_coeffs(band_idx, ...);
            // }
        }

        for frame in buffer.iter_samples() {
            let mut input_l = *frame.get(0).unwrap_or(&0.0);
            let mut input_r = *frame.get(1).unwrap_or(&0.0);

            // 1. Process through Parametric EQ if enabled
            if eq_enabled {
                (input_l, input_r) = self.parametric_eq.process_stereo_sample(input_l, input_r);
            }

            // 2. Process through Convolution Engine
            let (output_l, output_r) = self.convolution_engine.process_stereo_sample(input_l, input_r);

            if let Some(sample) = frame.get_mut(0) {
                *sample = output_l;
            }
            if let Some(sample) = frame.get_mut(1) {
                *sample = output_r;
            }
        }

        // Apply master output gain after all processing
        let master_gain = self.params.output_gain.smoothed.next();
        for channel_samples in buffer.iter_samples() {
            for sample in channel_samples {
                *sample *= master_gain;
            }
        }
        ProcessStatus::Normal
    }
}

// Implement ClapPlugin trait
impl ClapPlugin for OpenHeadstagePlugin {
    const CLAP_ID: &'static str = "com.opensource.open-headstage";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Binaural speaker simulation plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some("http://example.com/manual");
    const CLAP_SUPPORT_URL: Option<&'static str> = Some("http://example.com/support");
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        // Remove this line: ClapFeature::Spatial,  // This variant doesn't exist
        // You can use other valid features like:
        // ClapFeature::Surround,
        // ClapFeature::Ambisonic,
    ];
}

// Implement Vst3Plugin trait
impl Vst3Plugin for OpenHeadstagePlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"OpenHeadstageXXX";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Spatial,
    ];
}

nih_export_clap!(OpenHeadstagePlugin);
nih_export_vst3!(OpenHeadstagePlugin);