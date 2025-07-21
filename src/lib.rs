use nih_plug::prelude::*;

use nih_plug_egui::{
    create_egui_editor, egui, resizable_window::ResizableWindow, widgets, EguiState,
};
use std::sync::{Arc, Mutex};

// Make sure our modules are declared
mod dsp;
mod sofa;

use crate::dsp::convolution::ConvolutionEngine;
use crate::dsp::parametric_eq::{BandConfig, FilterType, StereoParametricEQ};
use crate::sofa::loader::MySofa;
use std::path::PathBuf;

const NUM_EQ_BANDS: usize = 10;

pub enum Task {
    LoadSofa(PathBuf),
}

use parking_lot::RwLock;

#[derive(Params)]
struct OpenHeadstageParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[persist = "sofa-path"]
    pub sofa_file_path: Arc<RwLock<String>>,

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

    #[id = "eq_enable"]
    pub eq_enable: BoolParam,

    // EQ Bands (example for one band, repeat for NUM_EQ_BANDS)
    #[id = "eq_b1_en"]
    pub eq_band1_enable: BoolParam,
    #[id = "eq_b1_fc"]
    pub eq_band1_fc: FloatParam,
    #[id = "eq_b1_q"]
    pub eq_band1_q: FloatParam,
    #[id = "eq_b1_gain"]
    pub eq_band1_gain: FloatParam,
}

impl Default for OpenHeadstageParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(800, 600),

            sofa_file_path: Arc::new(RwLock::new(String::new())),

            output_gain: FloatParam::new(
                "Output Gain",
                util::db_to_gain(0.0),
                FloatRange::Linear {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(0.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            speaker_azimuth_left: FloatParam::new(
                "L Azimuth",
                -30.0,
                FloatRange::Linear {
                    min: -90.0,
                    max: 90.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit("°"),
            speaker_elevation_left: FloatParam::new(
                "L Elevation",
                0.0,
                FloatRange::Linear {
                    min: -45.0,
                    max: 45.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit("°"),
            speaker_azimuth_right: FloatParam::new(
                "R Azimuth",
                30.0,
                FloatRange::Linear {
                    min: -90.0,
                    max: 90.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit("°"),
            speaker_elevation_right: FloatParam::new(
                "R Elevation",
                0.0,
                FloatRange::Linear {
                    min: -45.0,
                    max: 45.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit("°"),
            eq_enable: BoolParam::new("Enable EQ", false),
            eq_band1_enable: BoolParam::new("EQ B1 Enable", false),
            eq_band1_fc: FloatParam::new(
                "EQ B1 Fc",
                1000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz"),
            eq_band1_q: FloatParam::new(
                "EQ B1 Q",
                1.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            ),
            eq_band1_gain: FloatParam::new(
                "EQ B1 Gain",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB"),
        }
    }
}

struct OpenHeadstagePlugin {
    params: Arc<OpenHeadstageParams>,
    convolution_engine: ConvolutionEngine,
    sofa_loader: Arc<Mutex<Option<MySofa>>>,
    parametric_eq: StereoParametricEQ,
    current_sample_rate: f32,
}

impl Default for OpenHeadstagePlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(OpenHeadstageParams::default()),
            convolution_engine: ConvolutionEngine::new(),
            sofa_loader: Arc::new(Mutex::new(None)),
            parametric_eq: StereoParametricEQ::new(NUM_EQ_BANDS, 44100.0),
            current_sample_rate: 44100.0,
        }
    }
}

impl Plugin for OpenHeadstagePlugin {
    const NAME: &'static str = "Open Headstage";
    const VENDOR: &'static str = "Open Source Community";
    const URL: &'static str = "http://example.com";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = Task;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let egui_state = self.params.editor_state.clone();
        create_egui_editor(
            egui_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                ResizableWindow::new(Self::NAME).show(egui_ctx, &egui_state, |ui| {
                    // Use a grid for layout
                    egui::Grid::new("params_grid")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            // --- Output Gain ---
                            ui.label("Output Gain");
                            ui.add(widgets::ParamSlider::for_param(&params.output_gain, setter));
                            ui.end_row();

                            // --- Speaker Angles ---
                            ui.strong("Speaker Configuration");
                            ui.end_row();

                            // Left Speaker
                            ui.label("Left Azimuth");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.speaker_azimuth_left,
                                setter,
                            ));
                            ui.end_row();

                            ui.label("Left Elevation");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.speaker_elevation_left,
                                setter,
                            ));
                            ui.end_row();

                            // Right Speaker
                            ui.label("Right Azimuth");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.speaker_azimuth_right,
                                setter,
                            ));
                            ui.end_row();

                            ui.label("Right Elevation");
                            ui.add(widgets::ParamSlider::for_param(
                                &params.speaker_elevation_right,
                                setter,
                            ));
                            ui.end_row();

                            // --- SOFA File ---
                            ui.strong("SOFA HRTF File");
                            ui.end_row();

                            if ui.button("Select SOFA File").clicked() {
                                if let Some(file) = rfd::FileDialog::new()
                                    .add_filter("SOFA Files", &["sofa"])
                                    .pick_file()
                                {
                                    let path_str = file.to_string_lossy().to_string();
                                    *params.sofa_file_path.write() = path_str;
                                    async_executor.execute_gui(Task::LoadSofa(file));
                                }
                            }
                            ui.label(params.sofa_file_path.read().as_str());
                            ui.end_row();

                            // --- Ear Shape Placeholder ---
                            ui.strong("Ear Shape");
                            ui.end_row();
                            ui.label("(Placeholder for ear shape image)");
                            ui.end_row();
                        });
                });
            },
        )
    }

    fn task_executor(&mut self) -> Box<dyn Fn(Self::BackgroundTask) + Send> {
        let sample_rate = self.current_sample_rate;
        let sofa_loader = self.sofa_loader.clone();

        Box::new(move |task| {
            match task {
                Task::LoadSofa(path) => {
                    nih_log!("Loading SOFA file from: {:?}", path);
                    match MySofa::open(path.to_string_lossy().as_ref(), sample_rate) {
                        Ok(loader) => {
                            nih_log!("Successfully loaded SOFA file: {:?}", path);
                            *sofa_loader.lock().unwrap() = Some(loader);
                            // TODO: Re-initialize the convolution engine with the new HRIRs
                        }
                        Err(e) => {
                            nih_log!("Failed to load SOFA file '{:?}': {:?}", path, e);
                            *sofa_loader.lock().unwrap() = None;
                        }
                    }
                }
            }
        })
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.current_sample_rate = buffer_config.sample_rate;
        nih_log!(
            "Plugin initialized. Sample rate: {}",
            self.current_sample_rate
        );
        nih_log!(
            "SOFA file path on initialization: '{}'",
            *self.params.sofa_file_path.read()
        );

        let sofa_path_str = self.params.sofa_file_path.read();
        if !sofa_path_str.is_empty() {
            nih_log!("Attempting to load SOFA file from: {}", *sofa_path_str);
            match MySofa::open(&sofa_path_str, self.current_sample_rate) {
                Ok(sofa_loader) => {
                    nih_log!("Successfully loaded SOFA file: {}", sofa_path_str);
                    *self.sofa_loader.lock().unwrap() = Some(sofa_loader);
                }
                Err(e) => {
                    nih_log!("Failed to load SOFA file '{}': {:?}", sofa_path_str, e);
                    *self.sofa_loader.lock().unwrap() = None;
                }
            }
        } else {
            nih_log!("No SOFA file path configured. Skipping SOFA loading.");
            *self.sofa_loader.lock().unwrap() = None;
        }

        self.convolution_engine = ConvolutionEngine::new();
        nih_log!("Convolution engine (re)initialized.");

        self.parametric_eq = StereoParametricEQ::new(NUM_EQ_BANDS, self.current_sample_rate);
        nih_log!(
            "Parametric EQ initialized with {} bands at {} Hz.",
            NUM_EQ_BANDS,
            self.current_sample_rate
        );

        true
    }

    fn reset(&mut self) {
        nih_log!("Plugin reset.");
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let _az_l = self.params.speaker_azimuth_left.smoothed.next();
        let _el_l = self.params.speaker_elevation_left.smoothed.next();
        let _az_r = self.params.speaker_azimuth_right.smoothed.next();
        let _el_r = self.params.speaker_elevation_right.smoothed.next();

        let [left_slice, right_slice] = buffer.as_slice() else {
            panic!("Expected exactly two audio channels, but got a different number.");
        };

        let eq_enabled = self.params.eq_enable.value();
        if eq_enabled {
            let band_config = BandConfig {
                filter_type: FilterType::Peak,
                center_freq: self.params.eq_band1_fc.smoothed.next(),
                q: self.params.eq_band1_q.smoothed.next(),
                gain_db: self.params.eq_band1_gain.smoothed.next(),
                enabled: self.params.eq_band1_enable.value(),
            };
            self.parametric_eq
                .update_band_coeffs(0, self.current_sample_rate, &band_config);

            for i in 0..left_slice.len() {
                (left_slice[i], right_slice[i]) = self
                    .parametric_eq
                    .process_stereo_sample(left_slice[i], right_slice[i]);
            }
        }

        let input_l = left_slice.to_vec();
        let input_r = right_slice.to_vec();

        self.convolution_engine
            .process_block(&input_l, &input_r, left_slice, right_slice);

        let master_gain = self.params.output_gain.smoothed.next();
        for sample in left_slice.iter_mut() {
            *sample *= master_gain;
        }
        for sample in right_slice.iter_mut() {
            *sample *= master_gain;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for OpenHeadstagePlugin {
    const CLAP_ID: &'static str = "com.opensource.open-headstage";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Binaural speaker simulation plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some("http://example.com/manual");
    const CLAP_SUPPORT_URL: Option<&'static str> = Some("http://example.com/support");
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for OpenHeadstagePlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"OpenHeadstageXXX";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Spatial];
}

nih_export_clap!(OpenHeadstagePlugin);
nih_export_vst3!(OpenHeadstagePlugin);
