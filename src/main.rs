mod window;
mod clicker;

use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use window::{
    find_game_window, is_window_valid, screen_to_window_coords, get_window_under_cursor,
    get_cursor_pos,
};
use clicker::click_at_position;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

struct HeilsClickerApp {
    // UI state
    delay_ms: String,
    status: String,
    
    // Calibrated coordinates
    calibrated_x: Option<i32>,
    calibrated_y: Option<i32>,
    
    // Calibration state
    calibrating: Arc<Mutex<bool>>,
    last_mouse_state: bool,
    
    // Clicker state
    running: Arc<Mutex<bool>>,
    game_hwnd: Option<windows::Win32::Foundation::HWND>,
}

impl Default for HeilsClickerApp {
    fn default() -> Self {
        Self {
            delay_ms: "200".to_string(),
            status: "Ready - Click 'Calibrate' to set click position".to_string(),
            calibrated_x: None,
            calibrated_y: None,
            calibrating: Arc::new(Mutex::new(false)),
            running: Arc::new(Mutex::new(false)),
            game_hwnd: None,
            last_mouse_state: false,
        }
    }
}

impl eframe::App for HeilsClickerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for calibration click
        let is_calibrating = *self.calibrating.lock().unwrap();
        if is_calibrating && self.game_hwnd.is_some() {
            unsafe {
                // Check if left mouse button was just pressed (transition from up to down)
                let key_state = GetAsyncKeyState(0x01); // VK_LBUTTON
                let mouse_down = (key_state as u16) & 0x8000 != 0;
                let just_pressed = mouse_down && !self.last_mouse_state;
                self.last_mouse_state = mouse_down;
                
                if just_pressed {
                    // Mouse button is down, check if it's over the game window
                    if let Some(cursor_hwnd) = get_window_under_cursor() {
                        if let Some(game_hwnd) = self.game_hwnd {
                            // Check if the window under cursor is the game window or a child
                            let mut check_hwnd = cursor_hwnd;
                            let mut found = false;
                            
                            // Walk up the window hierarchy to find the game window
                            for _ in 0..10 {
                                if check_hwnd.0 == game_hwnd.0 {
                                    found = true;
                                    break;
                                }
                                check_hwnd = windows::Win32::UI::WindowsAndMessaging::GetAncestor(
                                    check_hwnd,
                                    windows::Win32::UI::WindowsAndMessaging::GA_PARENT,
                                );
                                if check_hwnd.0 == 0 {
                                    break;
                                }
                            }
                            
                            if found {
                                // Get cursor position and convert to window coordinates
                                if let Some((screen_x, screen_y)) = get_cursor_pos() {
                                    if let Some((x, y)) = screen_to_window_coords(game_hwnd, screen_x, screen_y) {
                                        self.calibrated_x = Some(x);
                                        self.calibrated_y = Some(y);
                                        *self.calibrating.lock().unwrap() = false;
                                        self.status = format!("Calibrated: ({}, {})", x, y);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Heils Clicker");
            ui.separator();

            // Delay input
            ui.horizontal(|ui| {
                ui.label("Delay (ms):");
                ui.text_edit_singleline(&mut self.delay_ms);
            });

            ui.separator();

            // Connect/Disconnect button
            if self.game_hwnd.is_none() {
                if ui.button("Connect to Game").clicked() {
                    if let Some(hwnd) = find_game_window() {
                        self.game_hwnd = Some(hwnd);
                        self.status = "Connected to game - Click 'Calibrate' to set position".to_string();
                    } else {
                        self.status = "Game window not found".to_string();
                    }
                }
            } else {
                if ui.button("Disconnect").clicked() {
                    self.game_hwnd = None;
                    *self.calibrating.lock().unwrap() = false;
                    self.status = "Disconnected".to_string();
                }
            }

            ui.separator();

            // Calibrate button
            if self.game_hwnd.is_some() {
                let calibrating = *self.calibrating.lock().unwrap();
                if !calibrating {
                    if ui.button("Calibrate Position").clicked() {
                        *self.calibrating.lock().unwrap() = true;
                        self.last_mouse_state = false; // Reset to avoid false trigger
                        self.status = "Calibrating... Click on the game window to set position".to_string();
                    }
                } else {
                    ui.label("🔴 Calibrating - Click on the game window now!");
                    if ui.button("Cancel Calibration").clicked() {
                        *self.calibrating.lock().unwrap() = false;
                        self.status = "Calibration cancelled".to_string();
                    }
                }
            }

            ui.separator();

            // Show calibrated coordinates
            if let (Some(x), Some(y)) = (self.calibrated_x, self.calibrated_y) {
                ui.label(format!("Calibrated Position: X={}, Y={}", x, y));
            } else {
                ui.label("Position: Not calibrated");
            }

            ui.separator();

            // Start/Stop button
            let is_running = *self.running.lock().unwrap();
            
            if !is_running {
                if ui.button("Start Clicking").clicked() {
                    if self.game_hwnd.is_none() {
                        self.status = "Connect to game first".to_string();
                    } else if self.calibrated_x.is_none() || self.calibrated_y.is_none() {
                        self.status = "Calibrate position first".to_string();
                    } else {
                        let x = self.calibrated_x.unwrap();
                        let y = self.calibrated_y.unwrap();
                        let delay = self.delay_ms.parse::<u64>().unwrap_or(200);

                        // Start clicking thread
                        let hwnd = self.game_hwnd.unwrap();
                        let running = Arc::clone(&self.running);
                        
                        *running.lock().unwrap() = true;
                        self.status = format!("Clicking started at ({}, {})", x, y);

                        thread::spawn(move || {
                            while *running.lock().unwrap() {
                                if is_window_valid(hwnd) {
                                    click_at_position(hwnd, x, y);
                                } else {
                                    *running.lock().unwrap() = false;
                                    break;
                                }
                                thread::sleep(Duration::from_millis(delay));
                            }
                        });
                    }
                }
            } else {
                if ui.button("Stop Clicking").clicked() {
                    *self.running.lock().unwrap() = false;
                    self.status = "Clicking stopped".to_string();
                }
            }

            ui.separator();
            
            // Update status based on running state
            let is_running_check = *self.running.lock().unwrap();
            if is_running_check && self.game_hwnd.is_some() {
                ui.label(format!("Status: Clicking... ({})", self.status));
            } else {
                ui.label(format!("Status: {}", self.status));
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_title("Heils Clicker"),
        ..Default::default()
    };

    eframe::run_native(
        "Heils Clicker",
        options,
        Box::new(|_cc| Box::new(HeilsClickerApp::default())),
    )
}
