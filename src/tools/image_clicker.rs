use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::settings::AcceptItemSettings;
use crate::tools::r#trait::Tool;
use crate::calibration::{CalibrationManager, CalibrationResult};
use crate::automation::context::AutomationContext;
use crate::automation::detection::find_stored_template;
use crate::automation::interaction::delay_ms;
use crate::ui::image_clicker::{ImageUiAction, render_ui};
use crate::core::worker::Worker;
use std::sync::{Arc, Mutex};

pub struct ImageClickerTool {
    // UI state
    interval_ms_str: String,
    settings_synced: bool,

    // Runtime state (Worker)
    worker: Worker,

    // Calibration
    calibration: CalibrationManager,
}

impl Default for ImageClickerTool {
    fn default() -> Self {
        Self {
            interval_ms_str: "1000".to_string(),
            settings_synced: false,
            worker: Worker::new(),
            calibration: CalibrationManager::new(),
        }
    }
}

impl Tool for ImageClickerTool {
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

    fn start(&mut self, app_settings: &crate::settings::AppSettings, game_hwnd: Option<HWND>) {
         let settings = &app_settings.accept_item;

         if let Some(hwnd) = game_hwnd {
             self.start_automation(settings.clone(), hwnd);
         } else {
             self.worker.set_status("Connect to game first");
         }
    }

    fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, settings: &mut crate::settings::AppSettings, game_hwnd: Option<HWND>) {
        let settings = &mut settings.accept_item;

        // Sync UI with Settings on first load
        if !self.settings_synced {
            self.interval_ms_str = settings.interval_ms.to_string();
            self.settings_synced = true;
        }

        // Handle calibration interaction
        if let Some(hwnd) = game_hwnd {
            if let Some(result) = self.calibration.update(hwnd) {
                if let CalibrationResult::Area(l, t, w, h) = result {
                    settings.search_region = Some((l, t, w, h));
                    self.worker.set_status("Region calibrated");
                }
            }
        } else {
             // Disconnected logic
             if self.worker.is_running() {
                 self.worker.stop();
                 self.worker.set_status("Disconnected");
             }
        }

        // Repaint if calibrating to capture clicks immediately
        if self.calibration.is_active() {
             ctx.request_repaint();
        }

        let is_running = self.worker.is_running();
        let status = self.worker.get_status();
        let is_calibrating = self.calibration.is_active();
        let is_dragging = self.calibration.is_dragging();

        let action = render_ui(
            ui,
            &mut settings.image_path, // Bind directly to settings string
            &mut self.interval_ms_str,
            &mut settings.tolerance,
            settings.search_region,
            is_calibrating,
            is_dragging,
            is_running,
            &status,
            game_hwnd.is_some(),
        );

        // Update settings from string buffer immediately
        if let Ok(val) = self.interval_ms_str.parse::<u64>() {
            settings.interval_ms = val;
        }

        match action {
            ImageUiAction::StartRegionCalibration => {
                self.calibration.start_area();
                self.worker.set_status("Click and drag to select the search region");
            },
            ImageUiAction::CancelCalibration => {
                self.calibration.cancel();
                self.worker.set_status("Calibration cancelled");
            },
            ImageUiAction::ClearRegion => {
                settings.search_region = None;
            },
            ImageUiAction::Start => {
                if game_hwnd.is_none() {
                    self.worker.set_status("Connect to game first");
                } else {
                    self.start_automation(settings.clone(), game_hwnd.unwrap());
                }
            },
            ImageUiAction::Stop => {
                self.stop();
            },
            ImageUiAction::None => {}
        }
    }
}

impl ImageClickerTool {
    // start_automation kept as private helper
    fn start_automation(&mut self, settings: AcceptItemSettings, game_hwnd: HWND) {
        self.worker.set_status("Starting...");

        let image_path = settings.image_path.clone(); // Clone for thread

        self.worker.start(move |running: Arc<Mutex<bool>>, status: Arc<Mutex<String>>| {
            let mut ctx = match AutomationContext::new(game_hwnd) {
                Ok(c) => c,
                Err(e) => {
                    *status.lock().unwrap() = format!("Error: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };

            if let Err(e) = ctx.store_template(&image_path, settings.search_region, "target_image") {
                 *status.lock().unwrap() = format!("Image Error: {}", e);
                 *running.lock().unwrap() = false;
                 return;
            }

            *status.lock().unwrap() = "Searching...".to_string();

            while *running.lock().unwrap() {
                // Using settings.tolerance which is now treated as Minimum Confidence
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

                        // Hardcoded safety delay after click to prevent double-clicking
                        delay_ms(500);
                    },
                    _ => {
                        *status.lock().unwrap() = "Searching...".to_string();
                    }
                }

                // User-configured polling interval (how often to check screen)
                delay_ms(settings.interval_ms);
            }

            *status.lock().unwrap() = "Stopped".to_string();
        });
    }
}
