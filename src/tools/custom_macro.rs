use std::sync::{Arc, Mutex};
use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::settings::CustomMacroSettings;
use crate::tools::r#trait::Tool;
use crate::calibration::{CalibrationManager, CalibrationResult};
use crate::automation::interaction::delay_ms;
use crate::ui::custom_macro::{CustomMacroUiAction, render_ui};
use crate::core::worker::Worker;

pub struct CustomMacroTool {
    // Which macro profile this tool is managing
    macro_index: usize,
    
    // Runtime state (Generic Worker)
    worker: Worker,
    
    // Calibration
    calibration: CalibrationManager,
    calibrating_action_index: Option<usize>,
}

impl CustomMacroTool {
    pub fn new(macro_index: usize) -> Self {
        Self {
            macro_index,
            worker: Worker::new(),
            calibration: CalibrationManager::new(),
            calibrating_action_index: None,
        }
    }
}

impl Tool for CustomMacroTool {
    fn stop(&mut self) {
        self.worker.stop();
        if self.worker.get_status().contains("Stopped") {
             // Already stopped
        } else {
             self.worker.set_status("Stopped (ESC pressed)");
        }
    }

    fn is_running(&self) -> bool {
        self.worker.is_running()
    }

    fn get_name(&self) -> &str {
        "Custom Macro" // Name will be overridden dynamically in app.rs
    }

    fn start(&mut self, app_settings: &crate::settings::AppSettings, game_hwnd: Option<HWND>) {
        if self.macro_index >= app_settings.custom_macros.len() {
            self.worker.set_status("Macro profile not found");
            return;
        }
        
        let settings = &app_settings.custom_macros[self.macro_index].settings;
        
        if let Some(hwnd) = game_hwnd {
            if !settings.actions.is_empty() {
                self.start_macro(settings.clone(), hwnd);
            } else {
                self.worker.set_status("No actions configured");
            }
        } else {
             self.worker.set_status("Connect to game first");
        }
    }

    fn update(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, settings: &mut crate::settings::AppSettings, game_hwnd: Option<HWND>) {
        if self.macro_index >= settings.custom_macros.len() {
            ui.colored_label(egui::Color32::RED, "Error: Macro profile not found");
            return;
        }
        
        // Can delete this macro if there's more than 1 total
        // Calculate this BEFORE taking mutable borrow
        let can_delete = settings.custom_macros.len() > 1;
        
        let macro_settings = &mut settings.custom_macros[self.macro_index];
        
        // Handle calibration interaction
        if let Some(hwnd) = game_hwnd {
            if let Some(result) = self.calibration.update(hwnd) {
                if let CalibrationResult::Point(x, y) = result {
                    if let Some(idx) = self.calibrating_action_index.take() {
                        if let Some(action) = macro_settings.settings.actions.get_mut(idx) {
                            if let crate::settings::MacroAction::Click { coordinate, .. } = action {
                                *coordinate = Some((x, y));
                                self.worker.set_status(&format!("Click position set: ({}, {})", x, y));
                            }
                        }
                    }
                }
            }
        } else {
             // If disconnected, ensure we aren't running
             if self.worker.is_running() {
                 self.worker.stop();
                 self.worker.set_status("Disconnected");
             }
        }

        let is_running = self.worker.is_running();
        let status = self.worker.get_status();
        let is_calibrating = self.calibration.is_active();

        let action = render_ui(
            ui, 
            macro_settings,
            is_calibrating,
            self.calibrating_action_index,
            is_running, 
            &status, 
            game_hwnd.is_some(),
            can_delete
        );

        match action {
            CustomMacroUiAction::StartCalibration(action_index) => {
                self.calibrating_action_index = Some(action_index);
                self.calibration.start_point();
                self.worker.set_status("Click on the game window to set coordinates");
            },
            CustomMacroUiAction::CancelCalibration => {
                self.calibration.cancel();
                self.calibrating_action_index = None;
                self.worker.set_status("Cancelled");
            },
            CustomMacroUiAction::StartMacro => {
                if game_hwnd.is_none() {
                    self.worker.set_status("Connect to game first");
                } else if macro_settings.settings.actions.is_empty() {
                    self.worker.set_status("No actions configured");
                } else {
                    self.start_macro(macro_settings.settings.clone(), game_hwnd.unwrap());
                }
            },
            CustomMacroUiAction::StopMacro => {
                self.stop();
            },
            CustomMacroUiAction::DeleteMacro => {
                // Delete this macro from settings
                if settings.custom_macros.len() > 1 && self.macro_index < settings.custom_macros.len() {
                    settings.custom_macros.remove(self.macro_index);
                    settings.auto_save();
                    // Note: app.rs needs to rebuild tools after this frame
                }
            },
            CustomMacroUiAction::None => {}
        }
    }
}

impl CustomMacroTool {
    fn start_macro(&mut self, settings: CustomMacroSettings, game_hwnd: HWND) {
        self.worker.set_status("Running macro...");
        
        // Use generic worker
        self.worker.start(move |running: Arc<Mutex<bool>>, status: Arc<Mutex<String>>| {
            use crate::core::input::click_at_position;
            use crate::automation::context::AutomationContext;
            
            let mut ctx = match AutomationContext::new(game_hwnd) {
                Ok(c) => c,
                Err(e) => {
                    *status.lock().unwrap() = format!("Error: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };

            let loop_count = if settings.loop_enabled { settings.loop_count } else { 1 };

            for iteration in 0..loop_count {
                if !*running.lock().unwrap() {
                    break;
                }

                if settings.loop_enabled {
                    *status.lock().unwrap() = format!("Loop {}/{}", iteration + 1, loop_count);
                }

                for (idx, action) in settings.actions.iter().enumerate() {
                    if !*running.lock().unwrap() {
                        break;
                    }

                    match action {
                        crate::settings::MacroAction::Click { coordinate, button: _, use_mouse_movement } => {
                            if let Some((x, y)) = coordinate {
                                *status.lock().unwrap() = format!("Clicking at ({}, {})", x, y);
                                
                                if *use_mouse_movement {
                                    // Use screen coordinates with mouse movement
                                    use crate::automation::interaction::click_at_screen;
                                    click_at_screen(&mut ctx.gui, *x as u32, *y as u32);
                                } else {
                                    // Direct click without mouse movement
                                    click_at_position(game_hwnd, *x, *y);
                                }
                            } else {
                                *status.lock().unwrap() = format!("Action {}: Click position not set", idx + 1);
                            }
                        },
                        crate::settings::MacroAction::TypeText { text } => {
                            *status.lock().unwrap() = format!("Typing: {}", text);
                            if let Err(e) = ctx.gui.keyboard_input(text) {
                                *status.lock().unwrap() = format!("Keyboard error: {:?}", e);
                            }
                        },
                        crate::settings::MacroAction::Delay { milliseconds } => {
                            *status.lock().unwrap() = format!("Waiting {}ms", milliseconds);
                            delay_ms(*milliseconds);
                        },
                    }
                }
            }
            
            if *running.lock().unwrap() {
                *status.lock().unwrap() = "Macro completed!".to_string();
            } else {
                *status.lock().unwrap() = "Stopped by user".to_string();
            }
            *running.lock().unwrap() = false;
        });
    }
}
