use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Make sure our modules are declared
mod dsp;
mod sofa;
mod ui;

use crate::dsp::convolution::ConvolutionEngine;
use crate::dsp::parametric_eq::{BandConfig, FilterType, StereoParametricEQ};
use crate::sofa::loader::MySofa;
use crate::ui::speaker_visualizer::SpeakerVisualizer;
use egui_file_dialog::FileDialog;
use parking_lot::RwLock;

const NUM_EQ_BANDS: usize = 10;

pub enum Task {
    LoadSofa(PathBuf),
}

#[derive(Params)]
struct EqBandParams {
    #[id = "en"]
    pub enabled: BoolParam,
    #[id = "type"]
    pub filter_type: EnumParam<FilterType>,
    #[id = "fc"]
    pub frequency: FloatParam,
    #[id = "q"]
    pub q: FloatParam,
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for EqBandParams {
    fn default() -> Self {
        Self {
            enabled: BoolParam::new("Enabled", false),
            filter_type: EnumParam::new("Type", FilterType::Peak),
            frequency: FloatParam::new(
                "Frequency",
                1000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz"),
            q: FloatParam::new(
                "Q",
                1.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            ),
            gain: FloatParam::new(
                "Gain",
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

    #[nested(array, group = "EQ Bands")]
    pub eq_bands: Vec<EqBandParams>,
}

impl Default for OpenHeadstageParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(800, 700),
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
            eq_bands: (0..NUM_EQ_BANDS)
                .map(|_i| EqBandParams::default())
                .collect(),
        }
    }
}

struct EditorState {
    file_dialog: FileDialog,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            file_dialog: FileDialog::new(),
        }
    }
}

struct OpenHeadstagePlugin {
    params: Arc<OpenHeadstageParams>,
    convolution_engine: ConvolutionEngine,
    sofa_loader: Arc<parking_lot::Mutex<Option<MySofa>>>,
    parametric_eq: StereoParametricEQ,
    current_sample_rate: f32,
    has_logged_processing_start: AtomicBool,
}

impl Default for OpenHeadstagePlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(OpenHeadstageParams::default()),
            convolution_engine: ConvolutionEngine::new(),
            sofa_loader: Arc::new(parking_lot::Mutex::new(None)),
            parametric_eq: StereoParametricEQ::new(NUM_EQ_BANDS, 44100.0),
            current_sample_rate: 44100.0,
            has_logged_processing_start: AtomicBool::new(false),
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

        create_egui_editor(
            self.params.editor_state.clone(),
            EditorState::default(),
            |_, _| {},
            move |egui_ctx, setter, state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(Self::NAME);
                    });

                    egui::Grid::new("params_grid")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Output Gain");
                            ui.add(widgets::ParamSlider::for_param(&params.output_gain, setter));
                            ui.end_row();

                            ui.strong("Speaker Configuration");
                            ui.end_row();
                            ui.label("Left Azimuth");
                            ui.add(widgets::ParamSlider::for_param(&params.speaker_azimuth_left, setter));
                            ui.end_row();
                            ui.label("Left Elevation");
                            ui.add(widgets::ParamSlider::for_param(&params.speaker_elevation_left, setter));
                            ui.end_row();
                            ui.label("Right Azimuth");
                            ui.add(widgets::ParamSlider::for_param(&params.speaker_azimuth_right, setter));
                            ui.end_row();
                            ui.label("Right Elevation");
                            ui.add(widgets::ParamSlider::for_param(&params.speaker_elevation_right, setter));
                            ui.end_row();

                            let visualizer = SpeakerVisualizer {
                                left_azimuth: params.speaker_azimuth_left.value(),
                                left_elevation: params.speaker_elevation_left.value(),
                                right_azimuth: params.speaker_azimuth_right.value(),
                                right_elevation: params.speaker_elevation_right.value(),
                            };
                            ui.add(visualizer);
                            ui.end_row();

                            ui.strong("SOFA HRTF File");
                            ui.end_row();

                            if ui.button("Select SOFA File").clicked() {
                                state.file_dialog.pick_file();
                            }
                            ui.label(params.sofa_file_path.read().as_str());
                            ui.end_row();
                        });

                    ui.separator();

                    ui.strong("Parametric Equalizer");
                    ui.add(widgets::ParamSlider::for_param(&params.eq_enable, setter));

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("eq_grid")
                            .num_columns(6)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("On");
                                ui.label("Type");
                                ui.label("Freq");
                                ui.label("Q");
                                ui.label("Gain");
                                ui.end_row();

                                for (i, band) in params.eq_bands.iter().enumerate() {
                                    ui.label(format!("{}", i + 1));
                                    ui.add(widgets::ParamSlider::for_param(&band.enabled, setter));
                                    ui.add(widgets::ParamSlider::for_param(&band.filter_type, setter));
                                    ui.add(widgets::ParamSlider::for_param(&band.frequency, setter));
                                    ui.add(widgets::ParamSlider::for_param(&band.q, setter));
                                    ui.add(widgets::ParamSlider::for_param(&band.gain, setter));
                                    ui.end_row();
                                }
                            });
                    });
                });

                state.file_dialog.update(egui_ctx);

                if let Some(path) = state.file_dialog.take_picked() {
                    let path_str = path.to_string_lossy().to_string();
                    *params.sofa_file_path.write() = path_str;
                    async_executor.execute_gui(Task::LoadSofa(path));
                }
            },
        )
    }

    fn task_executor(&mut self) -> Box<dyn Fn(Self::BackgroundTask) + Send> {
        let sample_rate = self.current_sample_rate;
        let sofa_loader = self.sofa_loader.clone();

        Box::new(move |task| match task {
            Task::LoadSofa(path) => {
                nih_log!("BACKGROUND: Loading SOFA file from: {:?}", path);
                match MySofa::open(path.to_string_lossy().as_ref(), sample_rate) {
                    Ok(loader) => {
                        nih_log!("BACKGROUND: Successfully loaded SOFA file: {:?}", path);
                        *sofa_loader.lock() = Some(loader);
                    }
                    Err(e) => {
                        nih_log!("BACKGROUND: Failed to load SOFA file '{:?}': {:?}", path, e);
                        *sofa_loader.lock() = None;
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
        nih_log!("Initializing Open Headstage v{}", Self::VERSION);

        self.current_sample_rate = buffer_config.sample_rate;
        self.parametric_eq = StereoParametricEQ::new(NUM_EQ_BANDS, self.current_sample_rate);
        self.convolution_engine = ConvolutionEngine::new();

        let sofa_path_str = self.params.sofa_file_path.read();
        if !sofa_path_str.is_empty() {
            nih_log!("Attempting to load initial SOFA file: {}", sofa_path_str);
            match MySofa::open(&sofa_path_str, self.current_sample_rate) {
                Ok(sofa_loader) => {
                    nih_log!("Successfully loaded SOFA file.");
                    *self.sofa_loader.lock() = Some(sofa_loader)
                },
                Err(e) => nih_log!("Failed to load SOFA file '{}': {:?}", sofa_path_str, e),
            }
        }

        nih_log!("Initialization complete.");
        true
    }

    fn reset(&mut self) {
        self.parametric_eq.reset_all_bands_state();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if !self.has_logged_processing_start.swap(true, Ordering::Relaxed) {
            nih_log!("Audio processing started.");
        }

        let _az_l = self.params.speaker_azimuth_left.smoothed.next();
        let _el_l = self.params.speaker_elevation_left.smoothed.next();
        let _az_r = self.params.speaker_azimuth_right.smoothed.next();
        let _el_r = self.params.speaker_elevation_right.smoothed.next();

        let [left, right] = buffer.as_slice() else {
            return ProcessStatus::Error("Mismatched channel count");
        };

        if self.params.eq_enable.value() {
            for (i, band_params) in self.params.eq_bands.iter().enumerate() {
                let band_config = BandConfig {
                    filter_type: band_params.filter_type.value(),
                    center_freq: band_params.frequency.smoothed.next(),
                    q: band_params.q.smoothed.next(),
                    gain_db: band_params.gain.smoothed.next(),
                    enabled: band_params.enabled.value(),
                };
                self.parametric_eq.update_band_coeffs(i, self.current_sample_rate, &band_config);
            }
            self.parametric_eq.process_block(left, right);
        }

        let input_l = left.to_vec();
        let input_r = right.to_vec();
        self.convolution_engine.process_block(&input_l, &input_r, left, right);

        let master_gain = self.params.output_gain.smoothed.next();
        for mut channel_samples in buffer.iter_samples() {
            for sample in channel_samples.iter_mut() {
                *sample *= master_gain;
            }
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
