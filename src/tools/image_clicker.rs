use std::sync::{Arc, Mutex};
use std::thread;
use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::settings::AcceptItemSettings;
use crate::tools::r#trait::Tool;
use crate::calibration::{CalibrationManager, CalibrationResult};
use crate::automation::context::AutomationContext;
use crate::automation::detection::find_stored_template;
use crate::automation::interaction::delay_ms;
use crate::ui::image_clicker::{ImageUiAction, render_ui};

pub struct ImageClickerTool {
    // UI state
    interval_ms_str: String,
    
    // Runtime state
    running: Arc<Mutex<bool>>,
    status: Arc<Mutex<String>>,
    game_hwnd: Option<HWND>,
    
    // Calibration
    calibration: CalibrationManager,
}

impl Default for ImageClickerTool {
    fn default() -> Self {
        Self {
            interval_ms_str: "1000".to_string(),
            running: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Ready".to_string())),
            game_hwnd: None,
            calibration: CalibrationManager::new(),
        }
    }
}

impl Tool for ImageClickerTool {
    fn set_game_hwnd(&mut self, hwnd: Option<HWND>) {
        self.game_hwnd = hwnd;
        if hwnd.is_none() {
            *self.running.lock().unwrap() = false;
            self.calibration.cancel();
            *self.status.lock().unwrap() = "Disconnected".to_string();
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

impl ImageClickerTool {
    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, settings: &mut AcceptItemSettings) {
        // Handle calibration interaction
        if let Some(hwnd) = self.game_hwnd {
            if let Some(result) = self.calibration.handle_clicks(hwnd) {
                if let CalibrationResult::Area(l, t, w, h) = result {
                    settings.search_region = Some((l, t, w, h));
                    *self.status.lock().unwrap() = "Region calibrated".to_string();
                }
            }
        }
        
        // Repaint if calibrating to capture clicks immediately
        if self.calibration.is_active() {
             ctx.request_repaint();
        }

        let is_running = *self.running.lock().unwrap();
        let status = self.status.lock().unwrap().clone();
        let is_calibrating = self.calibration.is_active();
        let is_waiting = self.calibration.is_waiting_for_second_click();

        let action = render_ui(
            ui,
            &mut settings.image_path, // Bind directly to settings string
            &mut self.interval_ms_str,
            &mut settings.tolerance,
            settings.search_region,
            is_calibrating,
            is_waiting,
            is_running,
            &status,
            self.game_hwnd.is_some(),
        );

        match action {
            ImageUiAction::StartRegionCalibration => {
                self.calibration.start_area();
                *self.status.lock().unwrap() = "Click TOP-LEFT corner of search region".to_string();
            },
            ImageUiAction::CancelCalibration => {
                self.calibration.cancel();
                *self.status.lock().unwrap() = "Calibration cancelled".to_string();
            },
            ImageUiAction::ClearRegion => {
                settings.search_region = None;
            },
            ImageUiAction::Start => {
                if self.game_hwnd.is_none() {
                    *self.status.lock().unwrap() = "Connect to game first".to_string();
                } else {
                    let interval = self.interval_ms_str.parse::<u64>().unwrap_or(1000);
                    settings.interval_ms = interval;
                    self.start_automation(settings.clone());
                }
            },
            ImageUiAction::Stop => {
                self.stop();
            },
            ImageUiAction::None => {}
        }
    }

    fn start_automation(&mut self, settings: AcceptItemSettings) {
        let running = Arc::clone(&self.running);
        let status = Arc::clone(&self.status);
        let game_hwnd = self.game_hwnd.unwrap();

        *running.lock().unwrap() = true;
        *status.lock().unwrap() = "Starting...".to_string();

        thread::spawn(move || {
            let mut ctx = match AutomationContext::new(game_hwnd) {
                Ok(c) => c,
                Err(e) => {
                    *status.lock().unwrap() = format!("Error: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };

            // Store template
            // Note: AutomationContext uses memory cache, but standard find_template 
            // loads from disk every time. For optimization let's assume we want to cache it.
            // But detection module's `find_stored_template` expects a key.
            // Let's us use it directly.
            
            if let Err(e) = ctx.store_template(&settings.image_path, settings.search_region, "target_image") {
                 *status.lock().unwrap() = format!("Image Error: {}", e);
                 *running.lock().unwrap() = false;
                 return;
            }

            *status.lock().unwrap() = "Searching...".to_string();

            while *running.lock().unwrap() {
                // Using new settings.tolerance which is now treated as Minimum Confidence
                match find_stored_template(&mut ctx.gui, "target_image", settings.tolerance) {
                    Some(matches) if !matches.is_empty() => {
                        let (screen_x, screen_y) = matches[0];
                        
                        *status.lock().unwrap() = format!("Found at ({}, {}), clicking...", screen_x, screen_y);
                        
                        // Convert screen coords to window coords for Direct Click
                        use crate::core::window::screen_to_window_coords;
                        use crate::core::input::click_at_position;
                        
                        if let Some((client_x, client_y)) = screen_to_window_coords(game_hwnd, screen_x as i32, screen_y as i32) {
                             click_at_position(game_hwnd, client_x, client_y);
                        } else {
                             *status.lock().unwrap() = "Error converting coordinates".to_string();
                        }
                        
                        // Wait a bit extra after click
                        delay_ms(500);
                    },
                    _ => {
                        *status.lock().unwrap() = "Searching...".to_string();
                    }
                }
                
                delay_ms(settings.interval_ms);
            }
            
            *status.lock().unwrap() = "Stopped".to_string();
        });
    }
}
