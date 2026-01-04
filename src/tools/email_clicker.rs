use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::tools::r#trait::Tool;
use crate::calibration::{CalibrationManager, CalibrationResult};
use crate::automation::interaction::delay_ms;
use crate::ui::email_clicker::{EmailUiAction, render_ui};
use crate::core::worker::Worker;
use std::sync::{Arc, Mutex};

pub struct EmailClickerTool {
    // UI state
    cycles_str: String,
    delay_ms_str: String,
    settings_synced: bool,
    
    // Runtime state (Generic Worker)
    worker: Worker,
    
    // Calibration
    calibration: CalibrationManager,
    calibrating_button: Option<String>, // "receive" or "next"
}

impl Default for EmailClickerTool {
    fn default() -> Self {
        Self {
            cycles_str: "10".to_string(),
            delay_ms_str: "200".to_string(),
            settings_synced: false,
            worker: Worker::new(),
            calibration: CalibrationManager::new(),
            calibrating_button: None,
        }
    }
}

impl Tool for EmailClickerTool {
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
        "E-mail Clicker"
    }

    fn start(&mut self, app_settings: &crate::settings::AppSettings, game_hwnd: Option<HWND>) {
        let settings = &app_settings.email_clicker;
        
        if let Some(hwnd) = game_hwnd {
            if settings.receive_position.is_some() && settings.next_position.is_some() {
                self.start_clicking(
                    settings.receive_position.unwrap(),
                    settings.next_position.unwrap(),
                    settings.cycles,
                    settings.interval_ms,
                    hwnd
                );
            } else {
                self.worker.set_status("Set both button coordinates first");
            }
        } else {
             self.worker.set_status("Connect to game first");
        }
    }

    fn update(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, settings: &mut crate::settings::AppSettings, game_hwnd: Option<HWND>) {
        let settings = &mut settings.email_clicker;
        
        // Sync setting strings if needed (on first load)
        if !self.settings_synced {
            self.cycles_str = settings.cycles.to_string();
            self.delay_ms_str = settings.interval_ms.to_string();
            self.settings_synced = true;
        }

        // Handle calibration interaction
        if let Some(hwnd) = game_hwnd {
            if let Some(result) = self.calibration.update(hwnd) {
                if let CalibrationResult::Point(x, y) = result {
                    if let Some(button_name) = self.calibrating_button.take() {
                        if button_name == "receive" {
                            settings.receive_position = Some((x, y));
                            self.worker.set_status(&format!("Receive button set: ({}, {})", x, y));
                        } else if button_name == "next" {
                            settings.next_position = Some((x, y));
                            self.worker.set_status(&format!("Next button set: ({}, {})", x, y));
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
            &mut self.cycles_str,
            &mut self.delay_ms_str,
            settings.receive_position,
            settings.next_position,
            is_calibrating, 
            is_running, 
            &status, 
            game_hwnd.is_some()
        );

        // Update settings from string buffers immediately
        if let Ok(val) = self.cycles_str.parse::<u32>() {
            settings.cycles = val;
        }
        if let Ok(val) = self.delay_ms_str.parse::<u64>() {
            settings.interval_ms = val;
        }

        match action {
            EmailUiAction::StartReceiveCalibration => {
                self.calibrating_button = Some("receive".to_string());
                self.calibration.start_point();
                self.worker.set_status("Setting Receive button... Click on the game window");
            },
            EmailUiAction::StartNextCalibration => {
                self.calibrating_button = Some("next".to_string());
                self.calibration.start_point();
                self.worker.set_status("Setting Next button... Click on the game window");
            },
            EmailUiAction::CancelCalibration => {
                self.calibration.cancel();
                self.calibrating_button = None;
                self.worker.set_status("Cancelled");
            },
            EmailUiAction::StartClicking => {
                let cycles = self.cycles_str.parse::<u32>().unwrap_or(10);
                let delay = self.delay_ms_str.parse::<u64>().unwrap_or(200);
                settings.cycles = cycles;
                settings.interval_ms = delay;
                
                if game_hwnd.is_none() {
                    self.worker.set_status("Connect to game first");
                } else if settings.receive_position.is_none() || settings.next_position.is_none() {
                    self.worker.set_status("Set both button coordinates first");
                } else {
                    self.start_clicking(
                        settings.receive_position.unwrap(),
                        settings.next_position.unwrap(),
                        cycles,
                        delay,
                        game_hwnd.unwrap()
                    );
                }
            },
            EmailUiAction::StopClicking => {
                self.stop();
            },
            EmailUiAction::None => {}
        }
    }
}

impl EmailClickerTool {
    // Background clicking using SendMessage (user keeps mouse control)
    fn start_clicking(&mut self, receive_pos: (i32, i32), next_pos: (i32, i32), cycles: u32, delay: u64, game_hwnd: HWND) {
        self.worker.set_status(&format!("Collecting {} emails...", cycles));
        
        // Use generic worker
        self.worker.start(move |running: Arc<Mutex<bool>>, status: Arc<Mutex<String>>| {
            use crate::core::input::click_at_position;

            for i in 0..cycles {
                if !*running.lock().unwrap() {
                    break;
                }
                
                *status.lock().unwrap() = format!("Email {}/{}: clicking Receive...", i + 1, cycles);
                
                // Click Receive button
                click_at_position(game_hwnd, receive_pos.0, receive_pos.1);
                delay_ms(delay);
                
                // Click Next button
                *status.lock().unwrap() = format!("Email {}/{}: clicking Next...", i + 1, cycles);
                click_at_position(game_hwnd, next_pos.0, next_pos.1);
                delay_ms(delay);
            }
            
            if *running.lock().unwrap() {
                *status.lock().unwrap() = format!("Completed {} emails!", cycles);
            } else {
                *status.lock().unwrap() = "Stopped by user".to_string();
            }
            *running.lock().unwrap() = false;
        });
    }
}
