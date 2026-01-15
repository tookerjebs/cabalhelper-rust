use crate::settings::{
    ComparisonMode, MacroAction, MouseButton, NamedMacro, OcrDecodeMode, OcrNameMatchMode,
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
            "Please connect to game first (top right)",
        );
        return CustomMacroUiAction::None;
    }

    // 1. Header Section (Grouped)
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Macro Name:").strong());
            ui.text_edit_singleline(&mut named_macro.name);
            ui.checkbox(&mut named_macro.show_in_overlay, "Show in overlay");

            if can_delete {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            egui::RichText::new("Delete")
                                .color(egui::Color32::from_rgb(255, 100, 100)),
                        )
                        .clicked()
                    {
                        action = CustomMacroUiAction::DeleteMacro;
                    }
                });
            }
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Add Action:").strong());
            if ui.button("Click").clicked() {
                named_macro.settings.actions.push(MacroAction::Click {
                    coordinate: None,
                    button: MouseButton::Left,
                    click_method: crate::settings::ClickMethod::SendMessage,
                    use_mouse_movement: false,
                });
            }
            if ui.button("Type Text").clicked() {
                named_macro.settings.actions.push(MacroAction::TypeText {
                    text: String::new(),
                });
            }
            if ui.button("Delay").clicked() {
                named_macro
                    .settings
                    .actions
                    .push(MacroAction::Delay { milliseconds: 100 });
            }
            if ui.button("OCR Search").clicked() {
                named_macro.settings.actions.push(MacroAction::OcrSearch {
                    ocr_region: None,
                    scale_factor: 2,
                    invert_colors: false,
                    grayscale: true,
                    decode_mode: OcrDecodeMode::Greedy,
                    beam_width: 10,
                    target_stat: String::new(),
                    target_value: 0,
                    alt_target_enabled: false,
                    alt_target_stat: String::new(),
                    alt_target_value: 0,
                    comparison: ComparisonMode::GreaterThanOrEqual,
                    name_match_mode: OcrNameMatchMode::Contains,
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
            ui.group(|ui| {
                ui.set_min_width(ui.available_width());

                ui.horizontal(|ui| {
                    // Reorder buttons (Vertical, Left) - Removed separator
                    ui.vertical(|ui| {
                        ui.add_space(2.0); // slight top offset
                        if idx > 0 {
                            if ui.button("⬆").on_hover_text("Move Up").clicked() {
                                to_move_up = Some(idx);
                            }
                        } else {
                            ui.add_space(20.0); // approximate button height placeholder
                        }

                        if idx < actions_len - 1 {
                            if ui.button("⬇").on_hover_text("Move Down").clicked() {
                                to_move_down = Some(idx);
                            }
                        }
                    });

                    ui.add_space(8.0);

                    // Main Content
                    ui.vertical(|ui| {
                        // Title Row
                        ui.horizontal(|ui| {
                            let title = match macro_action {
                                MacroAction::Click { .. } => "Click",
                                MacroAction::TypeText { .. } => "Type Text",
                                MacroAction::Delay { .. } => "Delay",
                                MacroAction::OcrSearch { .. } => "OCR Search",
                            };

                            ui.label(
                                egui::RichText::new(format!("{}. {}", idx + 1, title))
                                    .strong()
                                    .size(14.0),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button(
                                            egui::RichText::new("DEL")
                                                .color(egui::Color32::from_rgb(255, 100, 100)),
                                        )
                                        .clicked()
                                    {
                                        to_remove = Some(idx);
                                    }
                                },
                            );
                        });

                        ui.add_space(6.0);

                        // Config Fields
                        match macro_action {
                            MacroAction::Click {
                                coordinate,
                                button,
                                click_method,
                                use_mouse_movement: _,
                            } => {
                                ui.horizontal(|ui| {
                                    ui.label("Position:");
                                    if let Some((x, y)) = coordinate {
                                        ui.label(
                                            egui::RichText::new(format!("({:.3}, {:.3})", x, y))
                                                .monospace()
                                                .strong(),
                                        );
                                    } else {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(255, 100, 100),
                                            "Not set",
                                        );
                                    }

                                    let is_this_calibrating =
                                        click_calibrating_action_index == Some(idx);
                                    if is_this_calibrating {
                                        if ui
                                            .button(
                                                egui::RichText::new("Stop")
                                                    .color(egui::Color32::from_rgb(255, 100, 100)),
                                            )
                                            .clicked()
                                        {
                                            action = CustomMacroUiAction::CancelCalibration;
                                        }
                                        ui.label(
                                            egui::RichText::new("Click on game...")
                                                .color(egui::Color32::YELLOW),
                                        );
                                    } else {
                                        if ui.button("Set").clicked() {
                                            action = CustomMacroUiAction::StartCalibration(idx);
                                        }
                                        if coordinate.is_some()
                                            && ui
                                                .button("Clear")
                                                .on_hover_text("Clear Position")
                                                .clicked()
                                        {
                                            *coordinate = None;
                                        }
                                    }
                                });
                                ui.add_space(2.0);

                                ui.horizontal(|ui| {
                                    ui.label("Button:");
                                    ui.radio_value(button, MouseButton::Left, "Left");
                                    ui.radio_value(button, MouseButton::Right, "Right");
                                });
                                ui.add_space(2.0);

                                ui.horizontal(|ui| {
                                    ui.label("Method:");
                                    ui.radio_value(
                                        click_method,
                                        crate::settings::ClickMethod::SendMessage,
                                        "Direct",
                                    )
                                    .on_hover_text("SendMessage - Default, reliable for most apps");
                                    ui.radio_value(
                                        click_method,
                                        crate::settings::ClickMethod::MouseMovement,
                                        "Move",
                                    )
                                    .on_hover_text("Mouse Movement - Physically moves cursor");
                                });
                            }
                            MacroAction::TypeText { text } => {
                                ui.horizontal(|ui| {
                                    ui.label("Text:");
                                    ui.text_edit_singleline(text);
                                });
                            }
                            MacroAction::Delay { milliseconds } => {
                                ui.horizontal(|ui| {
                                    ui.label("Duration (ms):");
                                    let mut ms_str = milliseconds.to_string();
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut ms_str)
                                                .desired_width(80.0),
                                        )
                                        .changed()
                                    {
                                        if let Ok(val) = ms_str.parse() {
                                            *milliseconds = val;
                                        }
                                    }
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
                                alt_target_enabled,
                                alt_target_stat,
                                alt_target_value,
                                comparison,
                                name_match_mode,
                            } => {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Region:");
                                        if let Some((l, t, w, h)) = ocr_region {
                                            ui.monospace(format!(
                                                "({:.3}, {:.3}, {:.3}x{:.3})",
                                                l, t, w, h
                                            ));
                                        } else {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(255, 200, 100),
                                                "Not set",
                                            );
                                        }

                                        let is_this_ocr_calibrating =
                                            ocr_calibrating_action_index == Some(idx);
                                        if is_this_ocr_calibrating {
                                            if ui
                                                .button(
                                                    egui::RichText::new("Stop")
                                                        .color(egui::Color32::RED),
                                                )
                                                .clicked()
                                            {
                                                action =
                                                    CustomMacroUiAction::CancelOcrRegionCalibration;
                                            }
                                            ui.label(
                                                egui::RichText::new(
                                                    "Click top-left, then bottom-right",
                                                )
                                                .color(egui::Color32::YELLOW),
                                            );
                                        } else {
                                            if ui.button("Set Region").clicked() {
                                                action =
                                                    CustomMacroUiAction::StartOcrRegionCalibration(
                                                        idx,
                                                    );
                                            }
                                            if ocr_region.is_some() && ui.button("Clear").clicked()
                                            {
                                                *ocr_region = None;
                                            }
                                        }
                                    });

                                    ui.add_space(4.0);
                                    egui::CollapsingHeader::new("Advanced settings")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label("Scale:");
                                                egui::ComboBox::from_id_source(format!(
                                                    "ocr_scale_{}",
                                                    idx
                                                ))
                                                .selected_text(format!("{}x", *scale_factor))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(scale_factor, 1, "1x");
                                                    ui.selectable_value(scale_factor, 2, "2x");
                                                    ui.selectable_value(scale_factor, 3, "3x");
                                                    ui.selectable_value(scale_factor, 4, "4x");
                                                });

                                                ui.checkbox(invert_colors, "Invert");
                                                ui.checkbox(grayscale, "Grayscale");
                                            });

                                            ui.add_space(4.0);

                                            ui.horizontal(|ui| {
                                                ui.label("Decode:");
                                                egui::ComboBox::from_id_source(format!(
                                                    "ocr_decode_mode_{}",
                                                    idx
                                                ))
                                                .selected_text(match decode_mode {
                                                    OcrDecodeMode::Greedy => "Greedy (fast)",
                                                    OcrDecodeMode::BeamSearch => "Beam Search",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        decode_mode,
                                                        OcrDecodeMode::Greedy,
                                                        "Greedy (fast)",
                                                    );
                                                    ui.selectable_value(
                                                        decode_mode,
                                                        OcrDecodeMode::BeamSearch,
                                                        "Beam Search",
                                                    );
                                                });

                                                if matches!(decode_mode, OcrDecodeMode::BeamSearch)
                                                {
                                                    ui.label("Beam width:");
                                                    ui.add_sized(
                                                        egui::vec2(80.0, 0.0),
                                                        egui::DragValue::new(beam_width)
                                                            .clamp_range(2..=64),
                                                    );
                                                }
                                            });
                                        });

                                    ui.add_space(4.0);

                                    ui.horizontal(|ui| {
                                        ui.label("Stat:");
                                        ui.text_edit_singleline(target_stat);
                                        ui.label("Value:");
                                        let mut val_str = target_value.to_string();
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut val_str)
                                                    .desired_width(80.0),
                                            )
                                            .changed()
                                        {
                                            if let Ok(v) = val_str.parse() {
                                                *target_value = v;
                                            }
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        ui.checkbox(alt_target_enabled, "Alt target (OR)");
                                        if *alt_target_enabled {
                                            ui.label("Stat:");
                                            ui.text_edit_singleline(alt_target_stat);
                                            ui.label("Value:");
                                            let mut val_str = alt_target_value.to_string();
                                            if ui
                                                .add(
                                                    egui::TextEdit::singleline(&mut val_str)
                                                        .desired_width(80.0),
                                                )
                                                .changed()
                                            {
                                                if let Ok(v) = val_str.parse() {
                                                    *alt_target_value = v;
                                                }
                                            }
                                        }
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Stat Value:");
                                        egui::ComboBox::from_id_source(format!("ocr_cmp_{}", idx))
                                            .selected_text(match comparison {
                                                ComparisonMode::Equals => "Equal (==)",
                                                ComparisonMode::GreaterThanOrEqual => "≥",
                                                ComparisonMode::LessThanOrEqual => "≤",
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    comparison,
                                                    ComparisonMode::Equals,
                                                    "Equal (==)",
                                                );
                                                ui.selectable_value(
                                                    comparison,
                                                    ComparisonMode::GreaterThanOrEqual,
                                                    "≥",
                                                );
                                                ui.selectable_value(
                                                    comparison,
                                                    ComparisonMode::LessThanOrEqual,
                                                    "≤",
                                                );
                                            });
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Name match:");
                                        egui::ComboBox::from_id_source(format!(
                                            "ocr_name_match_{}",
                                            idx
                                        ))
                                        .selected_text(match name_match_mode {
                                            OcrNameMatchMode::Exact => "Exact name",
                                            OcrNameMatchMode::Contains => "Contains text",
                                        })
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                ui.selectable_value(
                                                    name_match_mode,
                                                    OcrNameMatchMode::Exact,
                                                    "Exact name",
                                                );
                                                ui.selectable_value(
                                                    name_match_mode,
                                                    OcrNameMatchMode::Contains,
                                                    "Contains text",
                                                );
                                            },
                                        );
                                    });

                                    ui.label(
                                        egui::RichText::new("If match is found, macro stops.")
                                            .size(11.0)
                                            .italics(),
                                    );
                                });
                            }
                        }
                    });
                });
            });
            ui.add_space(8.0);
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
            ("Stop Macro", egui::Color32::from_rgb(255, 100, 100))
        } else {
            ("Start Macro", egui::Color32::from_rgb(100, 255, 100))
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
