use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use eframe::egui;
use rustautogui::{RustAutoGui, MatchMode};

pub struct ImageClickerTool {
    // UI Settings
    interval_ms: String,
    image_path: String,
    tolerance: f32, // UI displays tolerance (error), we convert to precision (match)
    
    // Status
    status: String,
    
    // Runtime control
    running: Arc<Mutex<bool>>,
}

impl Default for ImageClickerTool {
    fn default() -> Self {
        Self {
            interval_ms: "1000".to_string(),
            image_path: "image.png".to_string(),
            tolerance: 0.1, // 10% tolerance = 0.9 precision
            status: "Ready".to_string(),
            running: Arc::new(Mutex::new(false)),
        }
    }
}

impl ImageClickerTool {
    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
        self.status = "Stopped (ESC pressed)".to_string();
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        ui.heading("Image Clicker (RustAutoGui)");
        ui.label("Automatically finds an image on screen and clicks it.");
        ui.separator();

        // Settings
        ui.horizontal(|ui| {
            ui.label("Image Path:");
            ui.text_edit_singleline(&mut self.image_path);
        });
        
        ui.horizontal(|ui| {
            ui.label("Interval (ms):");
            ui.text_edit_singleline(&mut self.interval_ms);
        });

        ui.horizontal(|ui| {
            ui.label("Tolerance (0.0 - 1.0):");
            ui.add(egui::Slider::new(&mut self.tolerance, 0.01..=0.99));
        });

        ui.separator();

        // Controls
        let is_running = *self.running.lock().unwrap();
        
        if is_running {
            ui.colored_label(egui::Color32::GREEN, "RUNNING");
            if ui.button("Stop").clicked() {
                *self.running.lock().unwrap() = false;
                self.status = "Stopped by user".to_string();
            }
        } else {
            if ui.button("Start").clicked() {
                self.start_clicker_thread();
            }
        }

        ui.separator();
        
        // Status
        ui.label(format!("Status: {}", self.status));
    }

    fn start_clicker_thread(&mut self) {
        let delay = self.interval_ms.parse::<u64>().unwrap_or(1000);
        let path = self.image_path.clone();
        // Convert tolerance (error rate) to precision (match rate)
        // e.g. 0.1 tolerance -> 0.9 precision
        let precision = (1.0 - self.tolerance).clamp(0.01, 1.0) as f64;
        
        // Start thread
        let running = Arc::clone(&self.running);
        *running.lock().unwrap() = true;
        self.status = "Starting...".to_string();

        thread::spawn(move || {
            // Initialize RustAutoGui
            // Note: RustAutoGui::new(debug_mode) returns Result
            let mut gui = match RustAutoGui::new(false) {
                Ok(g) => g,
                Err(e) => {
                    println!("Failed to initialize RustAutoGui: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };
            
            // Try to load template
            match gui.prepare_template_from_file(
                &path, 
                None, // region: None = full screen
                MatchMode::Segmented
            ) {
                Ok(_) => {
                    println!("Template loaded successfully: {}", path);
                },
                Err(e) => {
                    println!("Failed to load template: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            }

            while *running.lock().unwrap() {
                // Find image only (no mouse movement)
                match gui.find_image_on_screen(precision as f32) {
                    Ok(Some(matches)) => {
                        if let Some((x, y, _)) = matches.first() {
                             // Use our low-level clicker which uses SendMessage
                             // Use unsafe block for WinAPI calls
                             unsafe {
                                use windows::Win32::UI::WindowsAndMessaging::WindowFromPoint;
                                use windows::Win32::Foundation::POINT;
                                use crate::core::input::click_at_position;
                                use crate::core::window::screen_to_window_coords;
                                
                                // Convert u32 match to i32 for Windows API
                                let center_x = *x as i32;
                                let center_y = *y as i32;
                                
                                // Find window at that position
                                let hwnd = WindowFromPoint(POINT { x: center_x, y: center_y });
                                if hwnd.0 != 0 {
                                    // Convert screen coordinates to client coordinates
                                    if let Some((client_x, client_y)) = screen_to_window_coords(hwnd, center_x, center_y) {
                                        // Send direct click message (PostMessage)
                                        // This does NOT move the cursor
                                        click_at_position(hwnd, client_x, client_y);
                                    }
                                }
                             }
                        }
                    },
                    Ok(None) => {
                        // Not found
                    },
                    Err(e) => {
                         println!("Search error: {}", e);
                    }
                }

                thread::sleep(Duration::from_millis(delay));
            }
        });
    }
}
