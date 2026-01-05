use eframe::egui;
use crate::settings::{MacroAction, MouseButton, NamedMacro};

#[derive(Debug)]
pub enum CustomMacroUiAction {
    StartCalibration(usize), // action index
    CancelCalibration,
    StartMacro,
    StopMacro,
    DeleteMacro,
    None,
}

/// Render the Custom Macro Builder UI
pub fn render_ui(
    ui: &mut egui::Ui,
    named_macro: &mut NamedMacro,
    _is_calibrating: bool,
    calibrating_action_index: Option<usize>,
    is_running: bool,
    status: &str,
    game_connected: bool,
    can_delete: bool, // Can this macro be deleted?
) -> CustomMacroUiAction {
    let mut action = CustomMacroUiAction::None;
    
    if !game_connected {
        ui.colored_label(egui::Color32::RED, "Please connect to game first (top right)");
        return CustomMacroUiAction::None;
    }

    // Name editing field, delete button, and add action buttons in one section
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut named_macro.name);
        
        // Delete button (only if allowed)
        if can_delete {
            if ui.button("🗑 Delete").clicked() {
                action = CustomMacroUiAction::DeleteMacro;
            }
        }
        
        ui.separator();
        
        // Add action buttons
        if ui.button("+ Click").clicked() {
            named_macro.settings.actions.push(MacroAction::Click {
                coordinate: None,
                button: MouseButton::Left,
                click_method: crate::settings::ClickMethod::SendMessage,
                use_mouse_movement: false,
            });
        }
        if ui.button("+ Type Text").clicked() {
            named_macro.settings.actions.push(MacroAction::TypeText {
                text: String::new(),
            });
        }
        if ui.button("+ Delay").clicked() {
            named_macro.settings.actions.push(MacroAction::Delay {
                milliseconds: 100,
            });
        }
    });

    ui.add_space(10.0);

    // Actions List
    ui.heading("Actions");
    if named_macro.settings.actions.is_empty() {
        ui.label("No actions yet. Add some using the buttons above!");
    } else {
        let mut to_remove: Option<usize> = None;
        let mut to_move_up: Option<usize> = None;
        let mut to_move_down: Option<usize> = None;
        let actions_len = named_macro.settings.actions.len(); // Capture length before iterating

        for (idx, macro_action) in named_macro.settings.actions.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Order controls
                    ui.vertical(|ui| {
                        if idx > 0 {
                            if ui.small_button("^").clicked() {
                                to_move_up = Some(idx);
                            }
                        }
                        if idx < actions_len - 1 {
                            if ui.small_button("v").clicked() {
                                to_move_down = Some(idx);
                            }
                        }
                    });

                    ui.vertical(|ui| {
                        match macro_action {
                            MacroAction::Click { coordinate, button, click_method, use_mouse_movement: _ } => {
                                ui.label(format!("{}. Click", idx + 1));
                                
                                ui.horizontal(|ui| {
                                    ui.label("Position:");
                                    if let Some((x, y)) = coordinate {
                                        ui.label(format!("({}, {})", x, y));
                                    } else {
                                        ui.colored_label(egui::Color32::RED, "Not set");
                                    }
                                    
                                    let is_this_calibrating = calibrating_action_index == Some(idx);
                                    if is_this_calibrating {
                                        if ui.button("Cancel").clicked() {
                                            action = CustomMacroUiAction::CancelCalibration;
                                        }
                                        ui.label("Click on game...");
                                    } else {
                                        if ui.button("Set Position").clicked() {
                                            action = CustomMacroUiAction::StartCalibration(idx);
                                        }
                                        if coordinate.is_some() && ui.button("Clear").clicked() {
                                            *coordinate = None;
                                        }
                                    }
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Mouse Button:");
                                    ui.radio_value(button, MouseButton::Left, "Left");
                                    ui.radio_value(button, MouseButton::Right, "Right");
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Click Method:");
                                    ui.radio_value(click_method, crate::settings::ClickMethod::SendMessage, "Direct")
                                        .on_hover_text("SendMessage - Default, reliable for most apps");
                                    ui.radio_value(click_method, crate::settings::ClickMethod::PostMessage, "Async")
                                        .on_hover_text("PostMessage - Asynchronous, may work where Direct fails");
                                    ui.radio_value(click_method, crate::settings::ClickMethod::MouseMovement, "Movement")
                                        .on_hover_text("Mouse Movement - Physically moves cursor (slower, visible)");
                                });
                            },
                            MacroAction::TypeText { text } => {
                                ui.label(format!("{}. Type Text", idx + 1));
                                ui.horizontal(|ui| {
                                    ui.label("Text:");
                                    ui.text_edit_singleline(text);
                                });
                            },
                            MacroAction::Delay { milliseconds } => {
                                ui.label(format!("{}. Delay", idx + 1));
                                ui.horizontal(|ui| {
                                    ui.label("Duration (ms):");
                                    let mut ms_str = milliseconds.to_string();
                                    if ui.text_edit_singleline(&mut ms_str).changed() {
                                        if let Ok(val) = ms_str.parse() {
                                            *milliseconds = val;
                                        }
                                    }
                                });
                            },
                        }
                    });

                    // Delete button
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑").clicked() {
                            to_remove = Some(idx);
                        }
                    });
                });
            });
            ui.add_space(5.0);
        }

        // Handle reordering
        if let Some(idx) = to_move_up {
            named_macro.settings.actions.swap(idx, idx - 1);
        }
        if let Some(idx) = to_move_down {
            named_macro.settings.actions.swap(idx, idx + 1);
        }
        if let Some(idx) = to_remove {
            named_macro.settings.actions.remove(idx);
        }
    }

    ui.add_space(10.0);

    // Loop Settings
    ui.heading("Loop Settings");
    
    // Hint
    ui.label(egui::RichText::new("Tip: Always add a Delay action to loop infinitely safely!").color(egui::Color32::YELLOW).small());

    ui.horizontal(|ui| {
        ui.checkbox(&mut named_macro.settings.loop_enabled, "Enable Loop");
        
        if named_macro.settings.loop_enabled {
            ui.checkbox(&mut named_macro.settings.infinite_loop, "Infinite");
            
            if !named_macro.settings.infinite_loop {
                ui.label("Repeat:");
                let mut count_str = named_macro.settings.loop_count.to_string();
                if ui.text_edit_singleline(&mut count_str).changed() {
                    if let Ok(val) = count_str.parse::<u32>() {
                        named_macro.settings.loop_count = val.max(1);
                    }
                }
                ui.label("times");
            }
        }
    });

    ui.add_space(10.0);

    // Control buttons
    if !is_running {
        if ui.button("Start Macro").clicked() {
            action = CustomMacroUiAction::StartMacro;
        }
    } else {
        if ui.button("Stop").clicked() {
            action = CustomMacroUiAction::StopMacro;
        }
    }

    ui.separator();
    ui.heading("Status");
    ui.label(status);
    
    action
}
