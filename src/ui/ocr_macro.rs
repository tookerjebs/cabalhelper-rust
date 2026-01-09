use eframe::egui;
use crate::settings::{OcrMacroSettings, ComparisonMode, MacroAction, MouseButton, OcrDecodeMode, OcrNameMatchMode};

#[derive(Debug)]
pub enum OcrMacroUiAction {
    StartOcrRegionCalibration,
    CancelOcrRegionCalibration,
    ClearOcrRegion,
    StartActionCalibration(usize),
    CancelCalibration,
    Start,
    Stop,
    None,
}

pub fn render_ui(
    ui: &mut egui::Ui,
    settings: &mut OcrMacroSettings,
    is_ocr_calibrating: bool,
    is_ocr_waiting: bool,
    calibrating_action_index: Option<usize>, // Changing is_click_calibrating to this
    is_running: bool,
    status: &str,
    ocr_result: &str,
    match_found: bool,
    game_connected: bool,
) -> OcrMacroUiAction {
    let mut action = OcrMacroUiAction::None;

    ui.add_space(5.0);
    
    if !game_connected {
        ui.colored_label(egui::Color32::RED, "Please connect to game first (top right)");
        return OcrMacroUiAction::None;
    }

    // 1. OCR Configuration
    ui.group(|ui| {
        ui.heading(egui::RichText::new("1. OCR Region & Settings").size(14.0).strong());
        ui.add_space(4.0);
        
        // Region Selection
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Region:").strong());
            if let Some((l, t, w, h)) = settings.ocr_region {
                ui.monospace(format!("({}, {}, {}x{})", l, t, w, h));
            } else {
                ui.label(egui::RichText::new("Not set").color(egui::Color32::YELLOW).italics());
            }

            if is_ocr_calibrating {
                if ui.button(egui::RichText::new("Stop").color(egui::Color32::RED)).clicked() {
                    action = OcrMacroUiAction::CancelOcrRegionCalibration;
                }
                if is_ocr_waiting {
                    ui.label(egui::RichText::new("Click BOTTOM-RIGHT...").color(egui::Color32::YELLOW));
                } else {
                    ui.label(egui::RichText::new("Click TOP-LEFT...").color(egui::Color32::YELLOW));
                }
            } else {
                if ui.button("Set Region").clicked() {
                    action = OcrMacroUiAction::StartOcrRegionCalibration;
                }
                if settings.ocr_region.is_some() && ui.button("Clear").clicked() {
                    action = OcrMacroUiAction::ClearOcrRegion;
                }
            }
        });
        
        ui.add_space(4.0);
        
        // Image Preprocessing Settings
        ui.collapsing("Advanced OCR Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Scale:");
                egui::ComboBox::from_id_source("scale")
                    .selected_text(format!("{}x", settings.scale_factor))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut settings.scale_factor, 1, "1x");
                        ui.selectable_value(&mut settings.scale_factor, 2, "2x");
                        ui.selectable_value(&mut settings.scale_factor, 3, "3x");
                        ui.selectable_value(&mut settings.scale_factor, 4, "4x");
                    });
                
                ui.checkbox(&mut settings.invert_colors, "Invert Colors");
                ui.checkbox(&mut settings.grayscale, "Grayscale");
            });

            ui.add_space(4.0);

            // Decode method settings
            ui.horizontal(|ui| {
                ui.label("Decode:");
                egui::ComboBox::from_id_source("decode_mode")
                    .selected_text(match settings.decode_mode {
                        OcrDecodeMode::Greedy => "Greedy (fast)",
                        OcrDecodeMode::BeamSearch => "Beam Search",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut settings.decode_mode, OcrDecodeMode::Greedy, "Greedy (fast)");
                        ui.selectable_value(&mut settings.decode_mode, OcrDecodeMode::BeamSearch, "Beam Search (more accurate)");
                    });

                if matches!(settings.decode_mode, OcrDecodeMode::BeamSearch) {
                    ui.label("Beam width:");
                    ui.add(
                        egui::DragValue::new(&mut settings.beam_width)
                            .clamp_range(2..=64)
                    );
                }
            });
        });
    });
    
    ui.add_space(8.0);
    
    // 2. Target Configuration
    ui.group(|ui| {
        ui.heading(egui::RichText::new("2. Target Criteria").size(14.0).strong());
        ui.add_space(4.0);
        
        ui.horizontal(|ui| {
            ui.label("Stop when:");
            ui.text_edit_singleline(&mut settings.target_stat)
                .on_hover_text("e.g. 'Defense', 'HP', 'Crit Dmg'");
            ui.label("is");
        });
        
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_source("comparison")
                .selected_text(match settings.comparison {
                    ComparisonMode::Equals => "Equal to (=)",
                    ComparisonMode::GreaterThanOrEqual => "Greater or Equal (>=)",
                    ComparisonMode::LessThanOrEqual => "Less or Equal (<=)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.comparison, ComparisonMode::Equals, "Equal to (=)");
                    ui.selectable_value(&mut settings.comparison, ComparisonMode::GreaterThanOrEqual, "Greater or Equal (>=)");
                    ui.selectable_value(&mut settings.comparison, ComparisonMode::LessThanOrEqual, "Less or Equal (<=)");
                });
            
            ui.add(egui::DragValue::new(&mut settings.target_value));
        });

        ui.horizontal(|ui| {
            ui.label("Name match:");
            egui::ComboBox::from_id_source("ocr_name_match_mode")
                .selected_text(match settings.name_match_mode {
                    OcrNameMatchMode::Exact => "Exact name",
                    OcrNameMatchMode::Contains => "Contains text",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.name_match_mode, OcrNameMatchMode::Exact, "Exact name");
                    ui.selectable_value(&mut settings.name_match_mode, OcrNameMatchMode::Contains, "Contains text");
                });
        });
    });

    ui.add_space(8.0);

    // 3. Reroll Action (Sequence)
    ui.group(|ui| {
        ui.heading(egui::RichText::new("3. Reroll Sequence").size(14.0).strong());
        ui.add_space(4.0);
        
        ui.horizontal(|ui| {
             ui.label("Add Action:");
             if ui.button("Click").clicked() {
                settings.reroll_actions.push(MacroAction::Click {
                    coordinate: None,
                    button: MouseButton::Left,
                    click_method: crate::settings::ClickMethod::SendMessage,
                    use_mouse_movement: false,
                });
            }
            if ui.button("Type").clicked() {
                settings.reroll_actions.push(MacroAction::TypeText {
                    text: String::new(),
                });
            }
            if ui.button("Delay").clicked() {
                settings.reroll_actions.push(MacroAction::Delay {
                    milliseconds: 100,
                });
            }
        });
        
        ui.add_space(8.0);
        
        if settings.reroll_actions.is_empty() {
             ui.label(egui::RichText::new("No actions. Add 'Click' or 'Type' to reroll.").italics().color(egui::Color32::YELLOW));
        } else {
             // Action List Rendering
             let mut to_remove: Option<usize> = None;
             let mut to_move_up: Option<usize> = None;
             let mut to_move_down: Option<usize> = None;
             let actions_len = settings.reroll_actions.len();
             
             egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                 for (idx, macro_action) in settings.reroll_actions.iter_mut().enumerate() {
                     ui.group(|ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                             // Reorder controls
                            ui.vertical(|ui| {
                                if idx > 0 && ui.button("⬆").clicked() { to_move_up = Some(idx); }
                                if idx < actions_len - 1 && ui.button("⬇").clicked() { to_move_down = Some(idx); }
                            });
                            
                            ui.add_space(5.0);
                            
                            // Action Details
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let title = match macro_action {
                                        MacroAction::Click { .. } => "Click",
                                        MacroAction::TypeText { .. } => "Type",
                                        MacroAction::Delay { .. } => "Delay",
                                        MacroAction::OcrSearch { .. } => "OCR Search (unused here)",
                                    };
                                    ui.label(egui::RichText::new(format!("{}. {}", idx + 1, title)).strong());
                                    
                                     ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button(egui::RichText::new("DEL").color(egui::Color32::from_rgb(255, 100, 100))).clicked() {
                                            to_remove = Some(idx);
                                        }
                                    });
                                });
                                
                                match macro_action {
                                    MacroAction::Click { coordinate, button, click_method, .. } => {
                                        ui.horizontal(|ui| {
                                            if let Some((x, y)) = coordinate {
                                                ui.monospace(format!("({}, {})", x, y));
                                            } else {
                                                ui.label(egui::RichText::new("Not set").color(egui::Color32::RED));
                                            }
                                            
                                            let is_this_calibrating = calibrating_action_index == Some(idx);
                                            if is_this_calibrating {
                                                 if ui.button(egui::RichText::new("Stop").color(egui::Color32::RED)).clicked() {
                                                     action = OcrMacroUiAction::CancelCalibration;
                                                 }
                                                 ui.label(egui::RichText::new("Click...").color(egui::Color32::YELLOW));
                                            } else {
                                                 if ui.button("Set").clicked() {
                                                     action = OcrMacroUiAction::StartActionCalibration(idx);
                                                 }
                                            }
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.radio_value(button, MouseButton::Left, "L");
                                            ui.radio_value(button, MouseButton::Right, "R");
                                            ui.radio_value(click_method, crate::settings::ClickMethod::SendMessage, "Dir").on_hover_text("Direct");
                                            ui.radio_value(click_method, crate::settings::ClickMethod::PostMessage, "Async").on_hover_text("Async PostMessage");
                                            ui.radio_value(click_method, crate::settings::ClickMethod::MouseMovement, "Move").on_hover_text("Mouse Movement");
                                        });
                                    },
                                    MacroAction::TypeText { text } => {
                                        ui.text_edit_singleline(text);
                                    },
                                    MacroAction::Delay { milliseconds } => {
                                        ui.horizontal(|ui| {
                                            ui.label("ms:");
                                            ui.add(egui::DragValue::new(milliseconds));
                                        });
                                    }
                                    MacroAction::OcrSearch { .. } => {
                                        ui.label(egui::RichText::new("OCR Search actions are not supported in this sequence").italics());
                                    }
                                }
                            });
                        });
                     });
                     ui.add_space(2.0);
                 }
             });
             
             if let Some(idx) = to_remove { settings.reroll_actions.remove(idx); }
             if let Some(idx) = to_move_up { settings.reroll_actions.swap(idx, idx - 1); }
             if let Some(idx) = to_move_down { settings.reroll_actions.swap(idx, idx + 1); }
        }
        
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Loop Interval (ms):");
            ui.add(egui::DragValue::new(&mut settings.interval_ms).clamp_range(100..=10000));
        });
    });

    ui.add_space(12.0);

    // 4. Controls & Status
    ui.vertical_centered(|ui| {
        if is_running {
            if ui.button(egui::RichText::new("STOP").size(18.0).color(egui::Color32::WHITE).background_color(egui::Color32::RED)).clicked() {
                action = OcrMacroUiAction::Stop;
            }
        } else {
             // Disable start if not configured
            let ready = settings.ocr_region.is_some() && 
                        !settings.target_stat.trim().is_empty() &&
                        !settings.reroll_actions.is_empty();
                         
            if ui.add_enabled(ready, egui::Button::new(egui::RichText::new("START MACRO").size(18.0).color(egui::Color32::WHITE).background_color(egui::Color32::from_rgb(0, 150, 0)))).clicked() {
                action = OcrMacroUiAction::Start;
            }
        }

        
        ui.add_space(8.0);
        
        // Status display
        let color = if match_found {
            egui::Color32::GREEN
        } else if status.contains("Error") {
            egui::Color32::RED
        } else {
            egui::Color32::LIGHT_BLUE
        };
        ui.label(egui::RichText::new(status).color(color).strong());
    });
    
    ui.add_space(8.0);
    
    // 5. Live Feed
    ui.group(|ui| {
        ui.heading("OCR Output:");
        ui.add(egui::TextEdit::multiline(&mut ocr_result.to_string())
            .font(egui::TextStyle::Monospace)
            .desired_rows(3)
            .desired_width(f32::INFINITY));
    });

    action
}
