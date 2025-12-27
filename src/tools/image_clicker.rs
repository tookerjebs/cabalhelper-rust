use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use eframe::egui;
use rustautogui::{RustAutoGui, MatchMode};
use windows::Win32::Foundation::HWND;

pub struct ImageClickerTool {
    // UI Settings
    interval_ms: String,
    image_path: String,
    tolerance: f32, // UI displays tolerance (error), we convert to precision (match)
    
    // Region for searching (left, top, width, height) - window-relative
    search_region: Option<(i32, i32, i32, i32)>,
    
    // Status
    status: String,
    
    // Runtime control
    running: Arc<Mutex<bool>>,
    
    // Game window
    game_hwnd: Option<HWND>,
    
    // Calibration state
    calibrating: bool,
    area_selection_start: Option<(i32, i32)>,
    last_mouse_state: bool,
}

impl Default for ImageClickerTool {
    fn default() -> Self {
        Self {
            interval_ms: "1000".to_string(),
            image_path: "image.png".to_string(),
            tolerance: 0.15, // 15% tolerance = 0.85 precision
            search_region: None,
            status: "Ready".to_string(),
            running: Arc::new(Mutex::new(false)),
            game_hwnd: None,
            calibrating: false,
            area_selection_start: None,
            last_mouse_state: false,
        }
    }
}

impl ImageClickerTool {
    pub fn set_game_hwnd(&mut self, hwnd: Option<HWND>) {
        self.game_hwnd = hwnd;
        if hwnd.is_none() {
            *self.running.lock().unwrap() = false;
            self.calibrating = false;
        }
    }
    
    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
        self.status = "Stopped (ESC pressed)".to_string();
    }

    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("Accept Item");
        ui.label("Automatically finds and clicks an image (e.g., accept button).");
        ui.separator();
        
        // Check if connected
        if self.game_hwnd.is_none() {
            ui.colored_label(egui::Color32::RED, "Please connect to game first (top right)");
            return;
        }
        
        // Handle calibration clicks
        self.handle_calibration_clicks();
        
        if self.calibrating {
            ctx.request_repaint();
        }

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
        
        // Region calibration
        ui.add_space(10.0);
        ui.label("Search Region (optional - improves performance):");
        ui.horizontal(|ui| {
            let icon = if self.search_region.is_some() { "✓" } else { " " };
            ui.label(format!("[{}] Region", icon));
            
            if self.calibrating {
                if ui.button("Cancel").clicked() {
                    self.calibrating = false;
                    self.area_selection_start = None;
                    self.status = "Calibration cancelled".to_string();
                }
            } else {
                if ui.button("Set Region").clicked() {
                    self.calibrating = true;
                    self.area_selection_start = None;
                    self.last_mouse_state = false;
                    self.status = "Click TOP-LEFT corner of search region".to_string();
                }
                if self.search_region.is_some() && ui.button("Clear").clicked() {
                    self.search_region = None;
                    self.status = "Region cleared - searching full screen".to_string();
                }
            }
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

    
    fn handle_calibration_clicks(&mut self) {
        use crate::core::input::is_left_mouse_down;
        use crate::core::window::{get_window_under_cursor, is_game_window_or_child, get_cursor_pos, screen_to_window_coords};

        if !self.calibrating || self.game_hwnd.is_none() {
            return;
        }

        let mouse_down = is_left_mouse_down();
        let just_pressed = mouse_down && !self.last_mouse_state;
        self.last_mouse_state = mouse_down;

        if !just_pressed {
            return;
        }

        // Check if click is on game window
        if let Some(cursor_hwnd) = get_window_under_cursor() {
            if let Some(game_hwnd) = self.game_hwnd {
                if is_game_window_or_child(cursor_hwnd, game_hwnd) {
                    if let Some((screen_x, screen_y)) = get_cursor_pos() {
                        if let Some((client_x, client_y)) = screen_to_window_coords(game_hwnd, screen_x, screen_y) {
                            self.process_calibration_click(client_x, client_y);
                        }
                    }
                }
            }
        }
    }
    
    fn process_calibration_click(&mut self, x: i32, y: i32) {
        if self.area_selection_start.is_none() {
            // First click - store start
            self.area_selection_start = Some((x, y));
            self.status = "Now click BOTTOM-RIGHT corner".to_string();
        } else {
            // Second click - calculate area
            let (x1, y1) = self.area_selection_start.unwrap();
            let left = x1.min(x);
            let top = y1.min(y);
            let width = (x1.max(x) - left).abs();
            let height = (y1.max(y) - top).abs();
            
            self.search_region = Some((left, top, width, height));
            self.calibrating = false;
            self.area_selection_start = None;
            self.status = format!("Region set: ({}, {}, {}, {})", left, top, width, height);
        }
    }

    fn start_clicker_thread(&mut self) {
        let delay = self.interval_ms.parse::<u64>().unwrap_or(1000);
        let path = self.image_path.clone();
        let precision = (1.0 - self.tolerance).clamp(0.01, 1.0) as f32;
        let search_region = self.search_region;
        let game_hwnd = self.game_hwnd;
        
        // Start thread
        let running = Arc::clone(&self.running);
        *running.lock().unwrap() = true;
        self.status = "Starting...".to_string();

        thread::spawn(move || {
            let mut gui = match RustAutoGui::new(false) {
                Ok(g) => g,
                Err(e) => {
                    println!("Failed to initialize RustAutoGui: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };
            
            // Convert region to screen coordinates if set
            let screen_region = if let (Some(region), Some(hwnd)) = (search_region, game_hwnd) {
                use crate::core::window::get_window_rect;
                if let Some((win_x, win_y, _, _)) = get_window_rect(hwnd) {
                    let (left, top, width, height) = region;
                    Some((
                        (win_x + left) as u32,
                        (win_y + top) as u32,
                        width as u32,
                        height as u32
                    ))
                } else {
                    None
                }
            } else {
                None
            };
            
            // Load template with region
            match gui.prepare_template_from_file(
                &path, 
                screen_region,
                MatchMode::Segmented
            ) {
                Ok(_) => {
                    println!("Template loaded: {}", path);
                    if let Some(r) = screen_region {
                        println!("Search region: {:?}", r);
                    }
                },
                Err(e) => {
                    println!("Failed to load template: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            }

            while *running.lock().unwrap() {
                match gui.find_image_on_screen(precision) {
                    Ok(Some(matches)) => {
                        // Check if we have a high-confidence match
                        if let Some((x, y, confidence)) = matches.first() {
                            // CRITICAL: Only click if confidence is high enough (prevents false positives)
                            // Default min_confidence is 0.90 (90%)
                            let min_confidence = 0.90_f32;
                            
                            if *confidence >= min_confidence {
                                // Only click if we have a game window
                                if let Some(hwnd) = game_hwnd {
                                    unsafe {
                                        use crate::core::window::screen_to_window_coords;
                                        
                                        let center_x = *x as i32;
                                        let center_y = *y as i32;
                                        
                                        // Convert screen coordinates to game window coordinates
                                        if let Some((client_x, client_y)) = screen_to_window_coords(hwnd, center_x, center_y) {
                                            // Only click if coordinates are within game window bounds
                                            if client_x >= 0 && client_y >= 0 {
                                                use crate::core::input::click_at_position;
                                                click_at_position(hwnd, client_x, client_y);
                                                println!("✓ Clicked at ({}, {}) with {:.1}% confidence", client_x, client_y, confidence * 100.0);
                                            } else {
                                                println!("Match outside window bounds, ignoring");
                                            }
                                        }
                                    }
                                }
                            } else {
                                println!("Low confidence match ({:.1}%), ignoring (need {:.1}%+)", 
                                    confidence * 100.0, min_confidence * 100.0);
                            }
                        }
                    },
                    Ok(None) => {},
                    Err(e) => {
                         println!("Search error: {}", e);
                    }
                }

                thread::sleep(Duration::from_millis(delay));
            }
        });
    }
}
