use crate::settings::{
    ComparisonMode, MacroAction, MouseButton, NamedMacro, OcrAltTarget, OcrDecodeMode,
    OcrNameMatchMode,
};
use eframe::egui;

#[derive(Debug)]
pub enum CustomMacroUiAction {
    StartCalibration(usize), // Click action index
    CancelCalibration,
    StartOcrRegionCalibration(usize), // OCR action index
    CancelOcrRegionCalibration,
    StartMacro,
    StopMacro,
    DeleteMacro,
    None,
}

#[derive(Clone, Copy, PartialEq)]
enum OcrPreprocessPreset {
    Default,
    HighContrast,
    Invert,
    Grayscale,
    Custom,
}

#[derive(Clone, Copy, PartialEq)]
enum OcrAccuracyMode {
    Fast,
    HighAccuracy,
}

fn infer_ocr_preprocess_preset(
    scale_factor: u32,
    invert_colors: bool,
    grayscale: bool,
) -> OcrPreprocessPreset {
    if invert_colors && grayscale && scale_factor >= 3 {
        OcrPreprocessPreset::HighContrast
    } else if invert_colors && !grayscale {
        OcrPreprocessPreset::Invert
    } else if !invert_colors && grayscale && scale_factor == 2 {
        OcrPreprocessPreset::Grayscale
    } else if !invert_colors && !grayscale && scale_factor == 2 {
        OcrPreprocessPreset::Default
    } else {
        OcrPreprocessPreset::Custom
    }
}

fn apply_ocr_preprocess_preset(
    preset: OcrPreprocessPreset,
    scale_factor: &mut u32,
    invert_colors: &mut bool,
    grayscale: &mut bool,
) {
    match preset {
        OcrPreprocessPreset::Default => {
            *scale_factor = 2;
            *invert_colors = false;
            *grayscale = false;
        }
        OcrPreprocessPreset::HighContrast => {
            *scale_factor = 3;
            *invert_colors = true;
            *grayscale = true;
        }
        OcrPreprocessPreset::Invert => {
            *scale_factor = 2;
            *invert_colors = true;
            *grayscale = false;
        }
        OcrPreprocessPreset::Grayscale => {
            *scale_factor = 2;
            *invert_colors = false;
            *grayscale = true;
        }
        OcrPreprocessPreset::Custom => {}
    }
}

/// Render the Custom Macro Builder UI
pub fn render_ui(
    ui: &mut egui::Ui,
    named_macro: &mut NamedMacro,
    click_calibrating_action_index: Option<usize>,
    ocr_calibrating_action_index: Option<usize>,
    is_running: bool,
    status: &str,
    game_connected: bool,
    can_delete: bool, // Can this macro be deleted?
) -> CustomMacroUiAction {
    let mut action = CustomMacroUiAction::None;

    if !game_connected {
        ui.colored_label(
            egui::Color32::RED,
            "Please connect to game first (top left)",
        );
        return CustomMacroUiAction::None;
    }

    // 1. Header Section (Clean)
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Macro Name:").strong());
        ui.text_edit_singleline(&mut named_macro.name);

        // Spacer to push delete button to the right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if can_delete {
                if ui
                    .button(
                        egui::RichText::new("Delete").color(egui::Color32::from_rgb(255, 100, 100)),
                    )
                    .clicked()
                {
                    action = CustomMacroUiAction::DeleteMacro;
                }
            }
            ui.checkbox(&mut named_macro.show_in_overlay, "Show in Overlay");
        });
    });

    ui.add_space(8.0);

    // Toolbar for Adding Actions
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(40, 42, 45))
        .rounding(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Add Action:")
                        .strong()
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add_space(8.0);

                let toolbar_button = |ui: &mut egui::Ui, text: &str, color: egui::Color32| {
                    ui.add(
                        egui::Button::new(egui::RichText::new(text).color(color).strong())
                            .rounding(4.0),
                    )
                };

                let toolbar_color = egui::Color32::WHITE;

                if toolbar_button(ui, "+ Click", toolbar_color).clicked() {
                    named_macro.settings.actions.push(MacroAction::Click {
                        coordinate: None,
                        button: MouseButton::Left,
                        click_method: crate::settings::ClickMethod::SendMessage,
                        use_mouse_movement: false,
                    });
                }
                if toolbar_button(ui, "+ Type", toolbar_color).clicked() {
                    named_macro.settings.actions.push(MacroAction::TypeText {
                        text: String::new(),
                    });
                }
                if toolbar_button(ui, "+ Delay", toolbar_color).clicked() {
                    named_macro
                        .settings
                        .actions
                        .push(MacroAction::Delay { milliseconds: 100 });
                }
                if toolbar_button(ui, "+ OCR", toolbar_color).clicked() {
                    named_macro.settings.actions.push(MacroAction::OcrSearch {
                        ocr_region: None,
                        scale_factor: 2,
                        invert_colors: false,
                        grayscale: true,
                        decode_mode: OcrDecodeMode::Greedy,
                        beam_width: 10,
                        target_stat: String::new(),
                        target_value: 0,
                        comparison: ComparisonMode::GreaterThanOrEqual,
                        name_match_mode: OcrNameMatchMode::Contains,
                        alt_targets: Vec::new(),
                    });
                }
            });
        });

    ui.add_space(12.0);

    // 2. Actions List Section
    ui.heading(egui::RichText::new("Actions").size(16.0).strong());
    ui.add_space(4.0);

    if named_macro.settings.actions.is_empty() {
        ui.label(
            egui::RichText::new("No actions yet. Add some using the buttons above!").italics(),
        );
    } else {
        let mut to_remove: Option<usize> = None;
        let mut to_move_up: Option<usize> = None;
        let mut to_move_down: Option<usize> = None;
        let actions_len = named_macro.settings.actions.len();

        for (idx, macro_action) in named_macro.settings.actions.iter_mut().enumerate() {
            // Card Style Frame
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(32, 33, 36)) // Slightly lighter than background
                .rounding(6.0)
                .inner_margin(8.0)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50)))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    ui.horizontal(|ui| {
                        // Reorder buttons (Compact Vertical)
                        ui.vertical(|ui| {
                            let btn_size = egui::vec2(18.0, 18.0);
                            let arrow_btn = |ui: &mut egui::Ui, text: &str| {
                                ui.add_sized(btn_size, egui::Button::new(text).frame(false))
                            };

                            if idx > 0 {
                                if arrow_btn(ui, "⬆").on_hover_text("Move Up").clicked() {
                                    to_move_up = Some(idx);
                                }
                            } else {
                                ui.allocate_space(btn_size); // Placeholder
                            }

                            if idx < actions_len - 1 {
                                if arrow_btn(ui, "⬇").on_hover_text("Move Down").clicked() {
                                    to_move_down = Some(idx);
                                }
                            }
                        });

                        // Dark separator
                        ui.add_space(4.0);
                        let sep_rect = ui.allocate_space(egui::vec2(1.0, ui.available_height())).1;
                        ui.painter().line_segment(
                            [sep_rect.center_top(), sep_rect.center_bottom()],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60))
                        );
                        ui.add_space(4.0);


                        // Action Content
                        ui.vertical(|ui| {
                            // Header Row: Type | Index | Delete
                            ui.horizontal(|ui| {
                                let (title, color) = match macro_action {
                                    MacroAction::Click { .. } => ("CLICK", egui::Color32::from_rgb(100, 149, 237)),
                                    MacroAction::TypeText { .. } => ("TYPE", egui::Color32::from_rgb(200, 200, 200)),
                                    MacroAction::Delay { .. } => ("DELAY", egui::Color32::from_rgb(255, 215, 0)),
                                    MacroAction::OcrSearch { .. } => ("OCR", egui::Color32::from_rgb(218, 112, 214)),
                                };

                                // Removed colored indicator bar as requested

                                ui.label(
                                    egui::RichText::new(title)
                                        .strong()
                                        .color(color)
                                        .size(13.0),
                                );

                                // Push Delete to right
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        if ui
                                            .add(egui::Button::new(
                                                egui::RichText::new("✖") // Cross mark
                                                    .color(egui::Color32::from_rgb(150, 60, 60)),
                                            ).frame(false))
                                            .on_hover_text("Remove Action")
                                            .clicked()
                                        {
                                            to_remove = Some(idx);
                                        }
                                    },
                                );
                            });

                            ui.add_space(4.0);

                            // Config Fields (Indented)
                            ui.horizontal(|ui| {
                                ui.add_space(12.0); // Indent
                                ui.vertical(|ui| {
                                    match macro_action {
                                        MacroAction::Click {
                                            coordinate,
                                            button,
                                            click_method,
                                            use_mouse_movement: _,
                                        } => {
                                            ui.horizontal(|ui| {
                                                if let Some((x, y)) = coordinate {
                                                     ui.label(egui::RichText::new(format!("at ({:.0}, {:.0})", x, y)).monospace());
                                                } else {
                                                     ui.label(egui::RichText::new("Position not set").color(egui::Color32::RED));
                                                }

                                                // Inline calibration button
                                                let is_this_calibrating =
                                                    click_calibrating_action_index == Some(idx);

                                                if is_this_calibrating {
                                                    if ui.button(egui::RichText::new("CANCEL").size(10.0).color(egui::Color32::WHITE).strong()).clicked() {
                                                        action = CustomMacroUiAction::CancelCalibration;
                                                    }
                                                    ui.spinner();
                                                } else {
                                                    let btn_text = if coordinate.is_none() { "SET POS" } else { "SET" };
                                                    if ui.button(egui::RichText::new(btn_text).size(10.0)).clicked() {
                                                         action = CustomMacroUiAction::StartCalibration(idx);
                                                    }
                                                }

                                                ui.separator();

                                                ui.selectable_value(button, MouseButton::Left, "Left");
                                                ui.selectable_value(button, MouseButton::Right, "Right");
                                                ui.selectable_value(button, MouseButton::Middle, "Middle");

                                                ui.separator();

                                                egui::ComboBox::from_id_source(format!("method_{}", idx))
                                                    .selected_text(match click_method {
                                                        crate::settings::ClickMethod::SendMessage => "Direct",
                                                        crate::settings::ClickMethod::MouseMovement => "Mouse",
                                                    })
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(click_method, crate::settings::ClickMethod::SendMessage, "Direct (Backgr.)");
                                                        ui.selectable_value(click_method, crate::settings::ClickMethod::MouseMovement, "Physical Mouse");
                                                    });
                                            });
                                        }
                                        MacroAction::TypeText { text } => {
                                            ui.horizontal(|ui| {
                                                ui.label("Text:");
                                                ui.add(egui::TextEdit::singleline(text).hint_text("Enter text to type..."));
                                            });
                                        }
                                        MacroAction::Delay { milliseconds } => {
                                            ui.horizontal(|ui| {
                                                ui.label("Wait");
                                                ui.add(egui::DragValue::new(milliseconds).suffix(" ms").speed(10));
                                            });
                                        }
                                        MacroAction::OcrSearch {
                                            ocr_region,
                                            scale_factor,
                                            invert_colors,
                                            grayscale,
                                            decode_mode,
                                            beam_width,
                                            target_stat,
                                            target_value,
                                            comparison,
                                            name_match_mode,
                                            alt_targets,
                                        } => {
                                            // Compact OCR UI
                                            ui.horizontal(|ui| {
                                                if let Some((l, t, w, h)) = ocr_region {
                                                    ui.label(egui::RichText::new(format!("Region: {:.0},{:.0} {:.0}x{:.0}", l, t, w, h)).monospace().size(11.0));
                                                } else {
                                                    ui.label(egui::RichText::new("Region: Not Set").color(egui::Color32::RED).size(11.0));
                                                }

                                                let is_this_calibrating = ocr_calibrating_action_index == Some(idx);
                                                if is_this_calibrating {
                                                    if ui.button(egui::RichText::new("CANCEL").size(10.0)).clicked() {
                                                        action = CustomMacroUiAction::CancelOcrRegionCalibration;
                                                    }
                                                    ui.spinner();
                                                } else {
                                                     if ui.button(egui::RichText::new("SET AREA").size(10.0)).clicked() {
                                                         action = CustomMacroUiAction::StartOcrRegionCalibration(idx);
                                                     }
                                                }
                                            });

                                            ui.horizontal(|ui| {
                                                ui.add(egui::TextEdit::singleline(target_stat).desired_width(100.0).hint_text("Stat Name"));

                                                egui::ComboBox::from_id_source(format!("cmp_{}", idx))
                                                    .selected_text(match comparison {
                                                        ComparisonMode::Equals => "=",
                                                        ComparisonMode::GreaterThanOrEqual => "≥",
                                                        ComparisonMode::LessThanOrEqual => "≤",
                                                    })
                                                    .width(40.0)
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(comparison, ComparisonMode::Equals, "=");
                                                        ui.selectable_value(comparison, ComparisonMode::GreaterThanOrEqual, "≥");
                                                        ui.selectable_value(comparison, ComparisonMode::LessThanOrEqual, "≤");
                                                    });

                                                ui.add(egui::DragValue::new(target_value).speed(1));

                                                let match_label = match name_match_mode {
                                                    OcrNameMatchMode::Exact => "Match: Exact",
                                                    OcrNameMatchMode::Contains => "Match: Contains",
                                                };
                                                let match_combo = egui::ComboBox::from_id_source(
                                                    format!("match_inline_{}", idx),
                                                )
                                                .selected_text(match_label)
                                                .width(120.0);
                                                let match_response =
                                                    match_combo.show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            name_match_mode,
                                                            OcrNameMatchMode::Exact,
                                                            "Match: Exact",
                                                        );
                                                        ui.selectable_value(
                                                            name_match_mode,
                                                            OcrNameMatchMode::Contains,
                                                            "Match: Contains",
                                                        );
                                                    });
                                                match_response.response.on_hover_text(
                                                    "Exact: name must match fully. Contains: partial match.",
                                                );
                                            });

                                            if ui.link("Add alternate target").clicked() {
                                                alt_targets.push(OcrAltTarget {
                                                    target_stat: String::new(),
                                                    target_value: 0,
                                                    comparison: *comparison,
                                                    name_match_mode: *name_match_mode,
                                                    delay_ms: 100,
                                                });
                                            }

                                            let mut remove_alt: Option<usize> = None;
                                            for (alt_idx, alt) in alt_targets.iter_mut().enumerate()
                                            {
                                                ui.horizontal(|ui| {
                                                    ui.label(format!("Alt {}:", alt_idx + 1));
                                                    ui.add(
                                                        egui::TextEdit::singleline(
                                                            &mut alt.target_stat,
                                                        )
                                                        .desired_width(100.0)
                                                        .hint_text("Stat Name"),
                                                    );

                                                    egui::ComboBox::from_id_source(format!(
                                                        "alt_cmp_{}_{}",
                                                        idx, alt_idx
                                                    ))
                                                    .selected_text(match alt.comparison {
                                                        ComparisonMode::Equals => "=",
                                                        ComparisonMode::GreaterThanOrEqual => ">=",
                                                        ComparisonMode::LessThanOrEqual => "<=",
                                                    })
                                                    .width(40.0)
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut alt.comparison,
                                                            ComparisonMode::Equals,
                                                            "=",
                                                        );
                                                        ui.selectable_value(
                                                            &mut alt.comparison,
                                                            ComparisonMode::GreaterThanOrEqual,
                                                            ">=",
                                                        );
                                                        ui.selectable_value(
                                                            &mut alt.comparison,
                                                            ComparisonMode::LessThanOrEqual,
                                                            "<=",
                                                        );
                                                    });

                                                    ui.add(
                                                        egui::DragValue::new(&mut alt.target_value)
                                                            .speed(1),
                                                    );

                                                    let alt_match_label = match alt.name_match_mode {
                                                        OcrNameMatchMode::Exact => "Match: Exact",
                                                        OcrNameMatchMode::Contains => {
                                                            "Match: Contains"
                                                        }
                                                    };
                                                    egui::ComboBox::from_id_source(format!(
                                                        "alt_match_{}_{}",
                                                        idx, alt_idx
                                                    ))
                                                    .selected_text(alt_match_label)
                                                    .width(120.0)
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut alt.name_match_mode,
                                                            OcrNameMatchMode::Exact,
                                                            "Match: Exact",
                                                        );
                                                        ui.selectable_value(
                                                            &mut alt.name_match_mode,
                                                            OcrNameMatchMode::Contains,
                                                            "Match: Contains",
                                                        );
                                                    });

                                                    ui.label("Delay");
                                                    ui.add(
                                                        egui::DragValue::new(&mut alt.delay_ms)
                                                            .suffix(" ms")
                                                            .speed(10),
                                                    );

                                                    if ui.link("Remove").clicked() {
                                                        remove_alt = Some(alt_idx);
                                                    }
                                                });
                                            }
                                            if let Some(alt_idx) = remove_alt {
                                                alt_targets.remove(alt_idx);
                                            }

                                            egui::CollapsingHeader::new("Advanced")
                                                .id_source(format!("ocr_more_{}", idx))
                                                .default_open(false)
                                                .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label("Image preprocessing:");
                                                    let mut preset = infer_ocr_preprocess_preset(
                                                        *scale_factor,
                                                        *invert_colors,
                                                        *grayscale,
                                                    );
                                                    let previous_preset = preset;
                                                    let preset_label = match preset {
                                                        OcrPreprocessPreset::Default => "Default",
                                                        OcrPreprocessPreset::HighContrast => "High-contrast",
                                                        OcrPreprocessPreset::Invert => "Invert",
                                                        OcrPreprocessPreset::Grayscale => "Grayscale",
                                                        OcrPreprocessPreset::Custom => "Custom",
                                                    };
                                                    let preset_combo =
                                                        egui::ComboBox::from_id_source(format!(
                                                            "ocr_preset_{}",
                                                            idx
                                                        ))
                                                        .selected_text(preset_label);
                                                    preset_combo.show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut preset,
                                                            OcrPreprocessPreset::Default,
                                                            "Default",
                                                        );
                                                        ui.selectable_value(
                                                            &mut preset,
                                                            OcrPreprocessPreset::HighContrast,
                                                            "High-contrast",
                                                        );
                                                        ui.selectable_value(
                                                            &mut preset,
                                                            OcrPreprocessPreset::Invert,
                                                            "Invert",
                                                        );
                                                        ui.selectable_value(
                                                            &mut preset,
                                                            OcrPreprocessPreset::Grayscale,
                                                            "Grayscale",
                                                        );
                                                        if preset == OcrPreprocessPreset::Custom {
                                                            ui.selectable_value(
                                                                &mut preset,
                                                                OcrPreprocessPreset::Custom,
                                                                "Custom",
                                                            );
                                                        }
                                                    });
                                                    if preset != previous_preset {
                                                        apply_ocr_preprocess_preset(
                                                            preset,
                                                            scale_factor,
                                                            invert_colors,
                                                            grayscale,
                                                        );
                                                    }
                                                });

                                                ui.horizontal(|ui| {
                                                    ui.label("Accuracy vs speed:");
                                                    let mut accuracy = if matches!(
                                                        decode_mode,
                                                        OcrDecodeMode::BeamSearch
                                                    ) {
                                                        OcrAccuracyMode::HighAccuracy
                                                    } else {
                                                        OcrAccuracyMode::Fast
                                                    };
                                                    let previous_accuracy = accuracy;
                                                    egui::ComboBox::from_id_source(format!(
                                                        "ocr_accuracy_{}",
                                                        idx
                                                    ))
                                                    .selected_text(match accuracy {
                                                        OcrAccuracyMode::Fast => "Fast",
                                                        OcrAccuracyMode::HighAccuracy => "High accuracy",
                                                    })
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut accuracy,
                                                            OcrAccuracyMode::Fast,
                                                            "Fast",
                                                        );
                                                        ui.selectable_value(
                                                            &mut accuracy,
                                                            OcrAccuracyMode::HighAccuracy,
                                                            "High accuracy",
                                                        );
                                                    });
                                                    if accuracy != previous_accuracy {
                                                        *decode_mode = match accuracy {
                                                            OcrAccuracyMode::Fast => {
                                                                OcrDecodeMode::Greedy
                                                            }
                                                            OcrAccuracyMode::HighAccuracy => {
                                                                OcrDecodeMode::BeamSearch
                                                            }
                                                        };
                                                    }

                                                    if matches!(accuracy, OcrAccuracyMode::HighAccuracy) {
                                                        ui.label("Beam width:");
                                                        ui.add(egui::DragValue::new(beam_width).clamp_range(2..=20));
                                                    }
                                                });
                                            });
                                        }
                                    }
                                });
                            });
                        });
                    });
                });

            ui.add_space(4.0); // Spacing between cards
        }

        if let Some(idx) = to_remove {
            named_macro.settings.actions.remove(idx);
        }
        if let Some(idx) = to_move_up {
            named_macro.settings.actions.swap(idx, idx - 1);
        }
        if let Some(idx) = to_move_down {
            named_macro.settings.actions.swap(idx, idx + 1);
        }
    }

    ui.add_space(12.0);

    // 3. Loop Settings Section
    ui.group(|ui| {
        ui.heading(egui::RichText::new("Loop Settings").size(14.0).strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Don't forget to add delays between actions!")
                    .color(egui::Color32::from_rgb(255, 200, 100))
                    .size(12.0),
            );
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.checkbox(&mut named_macro.settings.loop_enabled, "Enable Loop");

            if named_macro.settings.loop_enabled {
                ui.separator();
                ui.checkbox(&mut named_macro.settings.infinite_loop, "Infinite");

                if !named_macro.settings.infinite_loop {
                    ui.label("Repeat:");
                    let mut count_str = named_macro.settings.loop_count.to_string();
                    if ui
                        .add(egui::TextEdit::singleline(&mut count_str).desired_width(80.0))
                        .changed()
                    {
                        if let Ok(val) = count_str.parse::<u32>() {
                            named_macro.settings.loop_count = val.max(1);
                        }
                    }
                    ui.label("times");
                }
            }
        });
    });

    ui.add_space(12.0);

    // 4. Control Buttons
    ui.vertical_centered(|ui| {
        let (btn_text, btn_color) = if is_running {
            ("Stop", egui::Color32::from_rgb(255, 100, 100))
        } else {
            ("Start", egui::Color32::from_rgb(100, 255, 100))
        };

        let button = egui::Button::new(egui::RichText::new(btn_text).size(16.0).color(btn_color))
            .min_size(egui::vec2(200.0, 35.0));

        if ui.add(button).clicked() {
            action = if is_running {
                CustomMacroUiAction::StopMacro
            } else {
                CustomMacroUiAction::StartMacro
            };
        }
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(6.0);

    // 5. Status Section
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Status:").strong());

        let status_color = if status.contains("Running") || status.contains("Active") {
            egui::Color32::from_rgb(100, 255, 100)
        } else if status.contains("Error") || status.contains("Failed") {
            egui::Color32::from_rgb(255, 100, 100)
        } else {
            egui::Color32::GRAY
        };

        ui.label(egui::RichText::new(status).color(status_color));
    });

    action
}
