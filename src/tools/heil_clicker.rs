use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::settings::HeilClickerSettings;
use crate::tools::r#trait::Tool;
use crate::calibration::{CalibrationManager, CalibrationResult};
use crate::automation::interaction::delay_ms;
use crate::ui::heil_clicker::{HeilUiAction, render_ui};
use crate::core::worker::Worker;
use std::sync::{Arc, Mutex};

pub struct HeilClickerTool {
    // UI state
    delay_ms_str: String,
    settings_synced: bool,
    
    // Runtime state (Generic Worker)
    worker: Worker,
    
    // Calibration
    calibration: CalibrationManager,
}

impl Default for HeilClickerTool {
    fn default() -> Self {
        Self {
            delay_ms_str: "200".to_string(),
            settings_synced: false,
            worker: Worker::new(),
            calibration: CalibrationManager::new(),
        }
    }
}

impl Tool for HeilClickerTool {
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

    fn get_status(&self) -> String {
        self.worker.get_status()
    }
}

impl HeilClickerTool {
    pub fn update(&mut self, ui: &mut egui::Ui, settings: &mut HeilClickerSettings, game_hwnd: Option<HWND>) {
        // Sync setting string if needed (on first load)
        if !self.settings_synced {
            self.delay_ms_str = settings.interval_ms.to_string();
            self.settings_synced = true;
        }

        // Handle calibration interaction
        if let Some(hwnd) = game_hwnd {
            if let Some(result) = self.calibration.handle_clicks(hwnd) {
                if let CalibrationResult::Point(x, y) = result {
                    settings.click_position = Some((x, y));
                    self.worker.set_status(&format!("Calibrated: ({}, {})", x, y));
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
            &mut self.delay_ms_str, 
            settings.click_position, 
            is_calibrating, 
            is_running, 
            &status, 
            game_hwnd.is_some()
        );

        // Update settings from string buffer immediately
        if let Ok(val) = self.delay_ms_str.parse::<u64>() {
            settings.interval_ms = val;
        }

        match action {
            HeilUiAction::StartCalibration => {
                self.calibration.start_point();
                self.worker.set_status("Calibrating... Click on the game window");
            },
            HeilUiAction::CancelCalibration => {
                self.calibration.cancel();
                self.worker.set_status("Calibration cancelled");
            },
            HeilUiAction::StartClicking => {
                let delay = self.delay_ms_str.parse::<u64>().unwrap_or(200);
                settings.interval_ms = delay;
                
                if game_hwnd.is_none() {
                    self.worker.set_status("Connect to game first");
                } else if settings.click_position.is_none() {
                    self.worker.set_status("Calibrate position first");
                } else {
                    self.start_clicking(settings.click_position.unwrap(), delay, game_hwnd.unwrap());
                }
            },
            HeilUiAction::StopClicking => {
                self.stop();
            },
            HeilUiAction::None => {}
        }
    }

    pub fn start(&mut self, settings: &HeilClickerSettings, game_hwnd: Option<HWND>) {
        let delay = self.delay_ms_str.parse::<u64>().unwrap_or(200);
        
        if game_hwnd.is_none() {
            self.worker.set_status("Connect to game first");
        } else if settings.click_position.is_none() {
            self.worker.set_status("Calibrate position first");
        } else {
            self.start_clicking(settings.click_position.unwrap(), delay, game_hwnd.unwrap());
        }
    }

    fn start_clicking(&mut self, pos: (i32, i32), delay: u64, game_hwnd: HWND) {
        self.worker.set_status(&format!("Clicking started at ({}, {})", pos.0, pos.1));
        
        // Use generic worker
        self.worker.start(move |running: Arc<Mutex<bool>>, status: Arc<Mutex<String>>| {
            use crate::core::input::click_at_position;

            while *running.lock().unwrap() {
                click_at_position(game_hwnd, pos.0, pos.1);
                delay_ms(delay);
            }
            *status.lock().unwrap() = "Clicking stopped".to_string();
        });
    }
}
