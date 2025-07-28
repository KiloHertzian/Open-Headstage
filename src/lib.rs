// Copyright 2025 SignalVerse
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crossbeam_channel::{Receiver, Sender};
use nih_plug::prelude::*;
use nih_plug_egui::{EguiState, create_egui_editor, egui, widgets};
use parking_lot::{Mutex, RwLock};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use strum::IntoEnumIterator;

// Make sure our modules are declared
mod autoeq_parser;
mod dsp;
mod sofa;
mod ui;

use crate::autoeq_parser::BandSetting;
use crate::dsp::convolution::ConvolutionEngine;
use crate::dsp::parametric_eq::{BandConfig, FilterType, StereoParametricEQ};
use crate::sofa::loader::MySofa;
use crate::ui::speaker_visualizer::SpeakerVisualizer;
use egui_file_dialog::FileDialog;

const NUM_EQ_BANDS: usize = 10;

pub enum Task {
    LoadSofa(PathBuf),
    LoadAutoEq(PathBuf, Arc<Mutex<Option<Vec<BandSetting>>>>),
    RequestEqResponse(Sender<Vec<f32>>),
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
            .with_unit(" Hz")
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            q: FloatParam::new(
                "Q",
                0.7,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),
            gain: FloatParam::new(
                "Gain",
                0.0,
                FloatRange::Linear {
                    min: -16.0,
                    max: 16.0,
                },
            )
            .with_unit(" dB")
            .with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}

#[derive(Params)]
struct OpenHeadstageParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[persist = "sofa-path"]
    pub sofa_file_path: Arc<RwLock<String>>,

    #[id = "bypass"]
    pub master_bypass: BoolParam,

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
            editor_state: EguiState::from_size(1380, 805),
            sofa_file_path: Arc::new(RwLock::new(String::new())),
            master_bypass: BoolParam::new("Bypass", false),
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

#[derive(PartialEq, Eq, Clone, Copy)]
enum FileDialogRequest {
    Sofa,
    AutoEq,
}

struct EditorState {
    file_dialog: FileDialog,
    file_dialog_request: Option<FileDialogRequest>,
    auto_eq_result: Arc<Mutex<Option<Vec<BandSetting>>>>,
    loaded_eq_settings: Option<Vec<BandSetting>>,
    gui_task_sender: Sender<Task>,
    show_eq_editor: bool,
    eq_editor_bands: Vec<BandSetting>,
    #[allow(dead_code)]
    eq_response: Arc<Mutex<Option<Vec<f32>>>>,
}

impl EditorState {
    fn new(
        gui_task_sender: Sender<Task>,
        auto_eq_result: Arc<Mutex<Option<Vec<BandSetting>>>>,
        initial_eq_params: &[EqBandParams],
    ) -> Self {
        let eq_editor_bands = initial_eq_params
            .iter()
            .map(|p| BandSetting {
                enabled: p.enabled.value(),
                filter_type: p.filter_type.value(),
                frequency: p.frequency.value(),
                q: p.q.value(),
                gain: p.gain.value(),
            })
            .collect();

        Self {
            file_dialog: FileDialog::new(),
            file_dialog_request: None,
            auto_eq_result,
            loaded_eq_settings: None,
            gui_task_sender,
            show_eq_editor: false,
            eq_editor_bands,
            eq_response: Arc::new(Mutex::new(None)),
        }
    }
}

pub struct OpenHeadstagePlugin {
    params: Arc<OpenHeadstageParams>,
    convolution_engine: ConvolutionEngine,
    sofa_loader: Arc<parking_lot::Mutex<Option<MySofa>>>,
    parametric_eq: StereoParametricEQ,
    current_sample_rate: f32,
    has_logged_processing_start: AtomicBool,
    gui_task_sender: Sender<Task>,
    gui_task_receiver: Receiver<Task>,
    auto_eq_result: Arc<Mutex<Option<Vec<BandSetting>>>>,
}

impl Default for OpenHeadstagePlugin {
    fn default() -> Self {
        let (gui_task_sender, gui_task_receiver) = crossbeam_channel::unbounded();
        Self {
            params: Arc::new(OpenHeadstageParams::default()),
            convolution_engine: ConvolutionEngine::new(),
            sofa_loader: Arc::new(parking_lot::Mutex::new(None)),
            parametric_eq: StereoParametricEQ::new(NUM_EQ_BANDS, 44100.0),
            current_sample_rate: 44100.0,
            has_logged_processing_start: AtomicBool::new(false),
            gui_task_sender,
            gui_task_receiver,
            auto_eq_result: Arc::new(Mutex::new(None)),
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let editor_state = EditorState::new(
            self.gui_task_sender.clone(),
            self.auto_eq_result.clone(),
            &self.params.eq_bands,
        );

        create_egui_editor(
            self.params.editor_state.clone(),
            editor_state,
            |egui_ctx, _| {
                let mut style = (*egui_ctx.style()).clone();
                style
                    .text_styles
                    .get_mut(&egui::TextStyle::Body)
                    .unwrap()
                    .size = 14.0;
                style
                    .text_styles
                    .get_mut(&egui::TextStyle::Button)
                    .unwrap()
                    .size = 16.0;
                style
                    .text_styles
                    .get_mut(&egui::TextStyle::Heading)
                    .unwrap()
                    .size = 20.0;
                style.spacing.item_spacing = egui::vec2(8.0, 8.0);
                egui_ctx.set_style(style);
            },
            move |egui_ctx, setter, state| {
                // EQ Editor Panel (conditionally shown)
                if state.show_eq_editor {
                    egui::SidePanel::right("eq_editor_panel")
                        .resizable(false)
                        .default_width(800.0)
                        .show(egui_ctx, |ui| {
                            ui.heading(egui::RichText::new("Parametric Equalizer").size(22.0));

                            // Placeholder for the EQ curve visualization
                            ui.add_space(10.0);
                            ui.group(|ui| {
                                ui.set_min_size(egui::vec2(ui.available_width(), 294.0));
                                ui.vertical_centered(|ui| {
                                    ui.label("EQ Curve Visualization (Future)");
                                });
                            });
                            ui.add_space(10.0);

                            // Scrollable area for the EQ bands
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (i, band_setting) in
                                    state.eq_editor_bands.iter_mut().enumerate()
                                {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("{:>2}", i + 1));
                                            let enabled_text = if band_setting.enabled {
                                                "Enabled"
                                            } else {
                                                "Disabled"
                                            };
                                            ui.toggle_value(
                                                &mut band_setting.enabled,
                                                enabled_text,
                                            );

                                            ui.add_space(10.0);

                                            egui::ComboBox::new(
                                                format!("filter_type_{}", i),
                                                "Type",
                                            )
                                            .selected_text(format!(
                                                "{:?}",
                                                band_setting.filter_type
                                            ))
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    for filter_type in FilterType::iter() {
                                                        ui.selectable_value(
                                                            &mut band_setting.filter_type,
                                                            filter_type,
                                                            format!("{:?}", filter_type),
                                                        );
                                                    }
                                                },
                                            );

                                            ui.add_space(20.0);

                                            let _freq_label = ui.label("Freq");
                                            let freq_drag = ui.add(
                                                egui::DragValue::new(&mut band_setting.frequency)
                                                    .speed(1.0)
                                                    .suffix(" Hz")
                                                    .range(20.0..=20000.0),
                                            );
                                            if freq_drag.double_clicked() {
                                                band_setting.frequency = 1000.0;
                                            }

                                            ui.add_space(20.0);

                                            let q_label = ui.label("Q");
                                            let q_drag = ui.add(
                                                egui::DragValue::new(&mut band_setting.q)
                                                    .speed(0.01)
                                                    .range(0.1..=10.0)
                                                    .fixed_decimals(2),
                                            );
                                            if q_drag.double_clicked() || q_label.double_clicked() {
                                                band_setting.q = 0.7;
                                            }

                                            ui.add_space(20.0);

                                            let gain_label = ui.label("Gain");
                                            let gain_drag = ui.add(
                                                egui::DragValue::new(&mut band_setting.gain)
                                                    .speed(0.1)
                                                    .suffix(" dB")
                                                    .range(-24.0..=24.0),
                                            );
                                            if gain_drag.double_clicked()
                                                || gain_label.double_clicked()
                                            {
                                                band_setting.gain = 0.0;
                                            }
                                        });
                                    });
                                }
                            });

                            ui.add_space(10.0);
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui
                                    .add(egui::Button::new("Apply").min_size(egui::vec2(0.0, 45.0)))
                                    .clicked()
                                {
                                    // Apply the temporary settings to the actual params
                                    for (i, band_setting) in
                                        state.eq_editor_bands.iter().enumerate()
                                    {
                                        if let Some(band_param) = params.eq_bands.get(i) {
                                            setter.begin_set_parameter(&band_param.enabled);
                                            setter.set_parameter(
                                                &band_param.enabled,
                                                band_setting.enabled,
                                            );
                                            setter.end_set_parameter(&band_param.enabled);

                                            setter.begin_set_parameter(&band_param.filter_type);
                                            setter.set_parameter(
                                                &band_param.filter_type,
                                                band_setting.filter_type,
                                            );
                                            setter.end_set_parameter(&band_param.filter_type);

                                            setter.begin_set_parameter(&band_param.frequency);
                                            setter.set_parameter(
                                                &band_param.frequency,
                                                band_setting.frequency,
                                            );
                                            setter.end_set_parameter(&band_param.frequency);

                                            setter.begin_set_parameter(&band_param.q);
                                            setter.set_parameter(&band_param.q, band_setting.q);
                                            setter.end_set_parameter(&band_param.q);

                                            setter.begin_set_parameter(&band_param.gain);
                                            setter
                                                .set_parameter(&band_param.gain, band_setting.gain);
                                            setter.end_set_parameter(&band_param.gain);
                                        }
                                    }
                                    state.show_eq_editor = false;
                                }
                                if ui
                                    .add(
                                        egui::Button::new("Cancel").min_size(egui::vec2(0.0, 45.0)),
                                    )
                                    .clicked()
                                {
                                    state.show_eq_editor = false;
                                }
                            });
                        });
                }

                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    // Main controls panel
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::RichText::new(Self::NAME).size(22.0));
                    });

                    ui.add_space(10.0);

                    egui::collapsing_header::CollapsingHeader::new(
                        egui::RichText::new("Master Output").size(22.0),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut bypass_state = params.master_bypass.value();
                        if ui.toggle_value(&mut bypass_state, "Bypass").changed() {
                            setter.begin_set_parameter(&params.master_bypass);
                            setter.set_parameter(&params.master_bypass, bypass_state);
                            setter.end_set_parameter(&params.master_bypass);
                        }
                        if ui.button("Reset to Default").clicked() {
                            let default_params = OpenHeadstageParams::default();

                            setter.begin_set_parameter(&params.master_bypass);
                            setter.set_parameter(&params.master_bypass, default_params.master_bypass.default_plain_value());
                            setter.end_set_parameter(&params.master_bypass);

                            setter.begin_set_parameter(&params.output_gain);
                            setter.set_parameter(&params.output_gain, default_params.output_gain.default_plain_value());
                            setter.end_set_parameter(&params.output_gain);

                            setter.begin_set_parameter(&params.speaker_azimuth_left);
                            setter.set_parameter(&params.speaker_azimuth_left, default_params.speaker_azimuth_left.default_plain_value());
                            setter.end_set_parameter(&params.speaker_azimuth_left);

                            setter.begin_set_parameter(&params.speaker_elevation_left);
                            setter.set_parameter(&params.speaker_elevation_left, default_params.speaker_elevation_left.default_plain_value());
                            setter.end_set_parameter(&params.speaker_elevation_left);

                            setter.begin_set_parameter(&params.speaker_azimuth_right);
                            setter.set_parameter(&params.speaker_azimuth_right, default_params.speaker_azimuth_right.default_plain_value());
                            setter.end_set_parameter(&params.speaker_azimuth_right);

                            setter.begin_set_parameter(&params.speaker_elevation_right);
                            setter.set_parameter(&params.speaker_elevation_right, default_params.speaker_elevation_right.default_plain_value());
                            setter.end_set_parameter(&params.speaker_elevation_right);

                            setter.begin_set_parameter(&params.eq_enable);
                            setter.set_parameter(&params.eq_enable, default_params.eq_enable.default_plain_value());
                            setter.end_set_parameter(&params.eq_enable);

                            for (i, band) in params.eq_bands.iter().enumerate() {
                                setter.begin_set_parameter(&band.enabled);
                                setter.set_parameter(&band.enabled, default_params.eq_bands[i].enabled.default_plain_value());
                                setter.end_set_parameter(&band.enabled);

                                setter.begin_set_parameter(&band.filter_type);
                                setter.set_parameter(&band.filter_type, default_params.eq_bands[i].filter_type.default_plain_value());
                                setter.end_set_parameter(&band.filter_type);

                                setter.begin_set_parameter(&band.frequency);
                                setter.set_parameter(&band.frequency, default_params.eq_bands[i].frequency.default_plain_value());
                                setter.end_set_parameter(&band.frequency);

                                setter.begin_set_parameter(&band.q);
                                setter.set_parameter(&band.q, default_params.eq_bands[i].q.default_plain_value());
                                setter.end_set_parameter(&band.q);

                                setter.begin_set_parameter(&band.gain);
                                setter.set_parameter(&band.gain, default_params.eq_bands[i].gain.default_plain_value());
                                setter.end_set_parameter(&band.gain);
                            }
                        }
                        ui.label("Output Gain");
                        ui.add(widgets::ParamSlider::for_param(&params.output_gain, setter));
                    });

                    egui::collapsing_header::CollapsingHeader::new(
                        egui::RichText::new("Speaker Configuration").size(22.0),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add(SpeakerVisualizer {
                            left_azimuth: params.speaker_azimuth_left.value(),
                            left_elevation: params.speaker_elevation_left.value(),
                            right_azimuth: params.speaker_azimuth_right.value(),
                            right_elevation: params.speaker_elevation_right.value(),
                        });
                        egui::Grid::new("speaker_grid")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Left");
                                ui.label("Right");
                                ui.end_row();

                                ui.label("Azimuth");
                                ui.add(widgets::ParamSlider::for_param(
                                    &params.speaker_azimuth_left,
                                    setter,
                                ));
                                ui.add(widgets::ParamSlider::for_param(
                                    &params.speaker_azimuth_right,
                                    setter,
                                ));
                                ui.end_row();

                                ui.label("Elevation");
                                ui.add(widgets::ParamSlider::for_param(
                                    &params.speaker_elevation_left,
                                    setter,
                                ));
                                ui.add(widgets::ParamSlider::for_param(
                                    &params.speaker_elevation_right,
                                    setter,
                                ));
                                ui.end_row();
                            });
                    });

                    egui::collapsing_header::CollapsingHeader::new(
                        egui::RichText::new("Headphone Equalization").size(22.0),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        if ui
                            .add(
                                egui::Button::new("Select SOFA File")
                                    .min_size(egui::vec2(0.0, 20.0)),
                            )
                            .clicked()
                        {
                            state.file_dialog.pick_file();
                            state.file_dialog_request = Some(FileDialogRequest::Sofa);
                        }

                        let mut eq_enabled = params.eq_enable.value();
                        if ui.toggle_value(&mut eq_enabled, "Enable EQ").changed() {
                            setter.begin_set_parameter(&params.eq_enable);
                            setter.set_parameter(&params.eq_enable, eq_enabled);
                            setter.end_set_parameter(&params.eq_enable);
                        }

                        if ui
                            .add(
                                egui::Button::new("Edit Parametric EQ")
                                    .min_size(egui::vec2(0.0, 20.0)),
                            )
                            .clicked()
                        {
                            state.show_eq_editor = !state.show_eq_editor;
                            if state.show_eq_editor {
                                // When opening the editor, copy current params to our temporary state
                                state.eq_editor_bands = params
                                    .eq_bands
                                    .iter()
                                    .map(|p| BandSetting {
                                        enabled: p.enabled.value(),
                                        filter_type: p.filter_type.value(),
                                        frequency: p.frequency.value(),
                                        q: p.q.value(),
                                        gain: p.gain.value(),
                                    })
                                    .collect();
                            }
                        }

                        if ui
                            .add(
                                egui::Button::new("Load AutoEQ Profile")
                                    .min_size(egui::vec2(0.0, 20.0)),
                            )
                            .clicked()
                        {
                            state.file_dialog.pick_file();
                            state.file_dialog_request = Some(FileDialogRequest::AutoEq);
                        }

                        if let Some(bands) = &state.loaded_eq_settings {
                            if ui
                                .add(
                                    egui::Button::new("Apply Loaded EQ")
                                        .min_size(egui::vec2(0.0, 20.0)),
                                )
                                .clicked()
                            {
                                setter.set_parameter(&params.eq_enable, true);
                                for (i, band_param) in params.eq_bands.iter().enumerate() {
                                    if let Some(band_setting) = bands.get(i) {
                                        setter.set_parameter(
                                            &band_param.enabled,
                                            band_setting.enabled,
                                        );
                                        setter.set_parameter(
                                            &band_param.filter_type,
                                            band_setting.filter_type,
                                        );
                                        setter.set_parameter(
                                            &band_param.frequency,
                                            band_setting.frequency,
                                        );
                                        setter.set_parameter(&band_param.q, band_setting.q);
                                        setter.set_parameter(&band_param.gain, band_setting.gain);
                                    }
                                }
                            }
                        }
                    });
                });

                if let Some(path) = state.file_dialog.update(egui_ctx).picked() {
                    match state.file_dialog_request {
                        Some(FileDialogRequest::Sofa) => {
                            let path_str = path.to_path_buf().to_string_lossy().to_string();
                            *params.sofa_file_path.write() = path_str;
                            state
                                .gui_task_sender
                                .send(Task::LoadSofa(path.to_path_buf()))
                                .unwrap();
                        }
                        Some(FileDialogRequest::AutoEq) => {
                            let result_mutex = state.auto_eq_result.clone();
                            state
                                .gui_task_sender
                                .send(Task::LoadAutoEq(path.to_path_buf(), result_mutex))
                                .unwrap();
                        }
                        None => nih_log!("File dialog picked but no request was made."),
                    }
                    state.file_dialog_request = None;
                }

                if let Some(bands) = state.auto_eq_result.lock().take() {
                    state.loaded_eq_settings = Some(bands);
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
            Task::LoadAutoEq(path, result_mutex) => {
                nih_log!("BACKGROUND: Loading AutoEQ profile from: {:?}", path);
                match autoeq_parser::parse_autoeq_csv(&path) {
                    Ok(parsed_bands) => {
                        nih_log!(
                            "BACKGROUND: Successfully parsed {} EQ bands from {:?}.",
                            parsed_bands.len(),
                            path
                        );
                        *result_mutex.lock() = Some(parsed_bands);
                    }
                    Err(e) => {
                        nih_log!(
                            "BACKGROUND: Failed to parse AutoEQ file '{:?}': {:?}",
                            path,
                            e
                        );
                    }
                }
            }
            Task::RequestEqResponse(_) => {
                nih_log!("BACKGROUND: RequestEqResponse task not implemented yet.");
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
                }
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
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if let Ok(task) = self.gui_task_receiver.try_recv() {
            context.execute_background(task);
        }

        if !self
            .has_logged_processing_start
            .swap(true, Ordering::Relaxed)
        {
            nih_log!("Audio processing started.");
        }

        if !self.params.master_bypass.value() {
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
                    self.parametric_eq
                        .update_band_coeffs(i, self.current_sample_rate, &band_config);
                }
                self.parametric_eq.process_block(left, right);
            }

            let input_l = left.to_vec();
            let input_r = right.to_vec();
            self.convolution_engine
                .process_block(&input_l, &input_r, left, right);

            let master_gain = self.params.output_gain.smoothed.next();
            for mut channel_samples in buffer.iter_samples() {
                for sample in channel_samples.iter_mut() {
                    *sample *= master_gain;
                }
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

// VST3 support is disabled by default to avoid the GPLv3 license.
// To re-enable, add the `vst3` feature to the `nih_plug` dependency in `Cargo.toml`
// and uncomment the code below.
//
// impl Vst3Plugin for OpenHeadstagePlugin {
//     const VST3_CLASS_ID: [u8; 16] = *b"OpenHeadstageXXX";
//     const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
//         &[Vst3SubCategory::Fx, Vst3SubCategory::Spatial];
// }

nih_export_clap!(OpenHeadstagePlugin);
// nih_export_vst3!(OpenHeadstagePlugin);
