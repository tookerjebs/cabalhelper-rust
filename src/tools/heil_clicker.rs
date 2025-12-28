use std::sync::{Arc, Mutex};
use std::thread;
use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::settings::HeilClickerSettings;
use crate::tools::r#trait::Tool;
use crate::calibration::{CalibrationManager, CalibrationResult};
use crate::automation::interaction::delay_ms;
use crate::ui::heil_clicker::{HeilUiAction, render_ui};

pub struct HeilClickerTool {
    // UI state
    delay_ms_str: String,
    settings_synced: bool,
    
    // Runtime state
    running: Arc<Mutex<bool>>,
    status: Arc<Mutex<String>>,
    game_hwnd: Option<HWND>,
    
    // Calibration
    calibration: CalibrationManager,
}

impl Default for HeilClickerTool {
    fn default() -> Self {
        Self {
            delay_ms_str: "200".to_string(),
            settings_synced: false,
            running: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Ready - Click 'Calibrate' to set click position".to_string())),
            game_hwnd: None,
            calibration: CalibrationManager::new(),
        }
    }
}

impl Tool for HeilClickerTool {
    fn set_game_hwnd(&mut self, hwnd: Option<HWND>) {
        self.game_hwnd = hwnd;
        if hwnd.is_none() {
            *self.running.lock().unwrap() = false;
            self.calibration.cancel();
            *self.status.lock().unwrap() = "Disconnected".to_string();
        } else {
             *self.status.lock().unwrap() = "Connected - Ready to calibrate".to_string();
        }
    }

    fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
        *self.status.lock().unwrap() = "Stopped (ESC pressed)".to_string();
    }

    fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    fn get_status(&self) -> String {
        self.status.lock().unwrap().clone()
    }
}

impl HeilClickerTool {
    pub fn update(&mut self, ui: &mut egui::Ui, settings: &mut HeilClickerSettings) {
        // Sync setting string if needed (on first load)
        if !self.settings_synced {
            self.delay_ms_str = settings.interval_ms.to_string();
            self.settings_synced = true;
        }

        // Handle calibration interaction
        if let Some(hwnd) = self.game_hwnd {
            if let Some(result) = self.calibration.handle_clicks(hwnd) {
                if let CalibrationResult::Point(x, y) = result {
                    settings.click_position = Some((x, y));
                    *self.status.lock().unwrap() = format!("Calibrated: ({}, {})", x, y);
                }
            }
        }

        let is_running = *self.running.lock().unwrap();
        let status = self.status.lock().unwrap().clone();
        let is_calibrating = self.calibration.is_active();

        let action = render_ui(
            ui, 
            &mut self.delay_ms_str, 
            settings.click_position, 
            is_calibrating, 
            is_running, 
            &status, 
            self.game_hwnd.is_some()
        );

        // Update settings from string buffer immediately
        if let Ok(val) = self.delay_ms_str.parse::<u64>() {
            settings.interval_ms = val;
        }

        match action {
            HeilUiAction::StartCalibration => {
                self.calibration.start_point();
                *self.status.lock().unwrap() = "Calibrating... Click on the game window".to_string();
            },
            HeilUiAction::CancelCalibration => {
                self.calibration.cancel();
                *self.status.lock().unwrap() = "Calibration cancelled".to_string();
            },
            HeilUiAction::StartClicking => {
                let delay = self.delay_ms_str.parse::<u64>().unwrap_or(200);
                settings.interval_ms = delay;
                
                if self.game_hwnd.is_none() {
                    *self.status.lock().unwrap() = "Connect to game first".to_string();
                } else if settings.click_position.is_none() {
                    *self.status.lock().unwrap() = "Calibrate position first".to_string();
                } else {
                    self.start_clicking(settings.click_position.unwrap(), delay);
                }
            },
            HeilUiAction::StopClicking => {
                self.stop();
            },
            HeilUiAction::None => {}
        }
    }

    pub fn start(&mut self, settings: &HeilClickerSettings) {
        let delay = self.delay_ms_str.parse::<u64>().unwrap_or(200);
        
        if self.game_hwnd.is_none() {
            *self.status.lock().unwrap() = "Connect to game first".to_string();
        } else if settings.click_position.is_none() {
            *self.status.lock().unwrap() = "Calibrate position first".to_string();
        } else {
            self.start_clicking(settings.click_position.unwrap(), delay);
        }
    }

    fn start_clicking(&mut self, pos: (i32, i32), delay: u64) {
        let running = Arc::clone(&self.running);
        let status = Arc::clone(&self.status);
        let game_hwnd = self.game_hwnd.unwrap();
        
        *running.lock().unwrap() = true;
        *status.lock().unwrap() = format!("Clicking started at ({}, {})", pos.0, pos.1);

        thread::spawn(move || {
            // Using direct SendMessage click (background click)
            // This does NOT move the mouse cursor
            use crate::core::input::click_at_position;

            while *running.lock().unwrap() {
                click_at_position(game_hwnd, pos.0, pos.1);
                
                delay_ms(delay);
            }
            *status.lock().unwrap() = "Clicking stopped".to_string();
        });
    }
}
