use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::settings::{OcrMacroSettings, MacroAction, OcrDecodeMode, OcrNameMatchMode, ComparisonMode};
use crate::tools::r#trait::Tool;
use crate::calibration::{CalibrationManager, CalibrationResult};
use crate::core::worker::Worker;
use crate::core::screen_capture::capture_region;
use crate::core::ocr_parser::{parse_ocr_result, matches_target};
use crate::ui::ocr_macro::{OcrMacroUiAction, render_ui};
use std::sync::{Arc, Mutex};
use crate::automation::interaction::delay_ms;
use ocrs::DecodeMethod;

// Embed the OCR models directly into the binary
const DETECTION_MODEL_BYTES: &[u8] = include_bytes!("../models/text-detection.rten");
const RECOGNITION_MODEL_BYTES: &[u8] = include_bytes!("../models/text-recognition.rten");

pub struct OcrMacroTool {
    macro_index: usize,
    
    // Runtime state
    worker: Worker,
    
    // Calibration managers
    ocr_region_calibration: CalibrationManager,
    calibration: CalibrationManager,
    calibrating_action_index: Option<usize>,
    
    // OCR result (shared with background thread)
    last_ocr_result: Arc<Mutex<String>>,
    match_found: Arc<Mutex<bool>>,
}

impl OcrMacroTool {
    pub fn new(macro_index: usize) -> Self {
        Self {
            macro_index,
            worker: Worker::new(),
            ocr_region_calibration: CalibrationManager::new(),
            calibration: CalibrationManager::new(),
            calibrating_action_index: None,
            last_ocr_result: Arc::new(Mutex::new(String::new())),
            match_found: Arc::new(Mutex::new(false)),
        }
    }
}

impl Tool for OcrMacroTool {
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
        if self.macro_index >= app_settings.ocr_macros.len() {
             self.worker.set_status("Macro profile not found");
             return;
        }

        let settings = &app_settings.ocr_macros[self.macro_index].settings;
        
        if let Some(hwnd) = game_hwnd {
            if settings.ocr_region.is_some() {
                // Validate target configuration
                if settings.target_stat.trim().is_empty() {
                    self.worker.set_status("Please set a target stat");
                    return;
                }
                
                // Validate reroll actions
                if settings.reroll_actions.is_empty() {
                     self.worker.set_status("Please add reroll actions");
                     return;
                }
                
                self.start_ocr_macro(settings.clone(), hwnd);
            } else {
                self.worker.set_status("Please set OCR region first");
            }
        } else {
            self.worker.set_status("Connect to game first");
        }
    }

    fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, settings: &mut crate::settings::AppSettings, game_hwnd: Option<HWND>) {
        if self.macro_index >= settings.ocr_macros.len() {
             ui.label("Error: Macro not found");
             return;
        }

        let mut_settings = &mut settings.ocr_macros[self.macro_index].settings;

        // Handle OCR region calibration
        if let Some(hwnd) = game_hwnd {
            if let Some(result) = self.ocr_region_calibration.update(hwnd) {
                if let CalibrationResult::Area(l, t, w, h) = result {
                    mut_settings.ocr_region = Some((l, t, w, h));
                    self.worker.set_status("OCR region calibrated");
                }
            }
            
            // Handle action point calibration
            if let Some(result) = self.calibration.update(hwnd) {
                if let CalibrationResult::Point(x, y) = result {
                    if let Some(idx) = self.calibrating_action_index.take() {
                         if let Some(action) = mut_settings.reroll_actions.get_mut(idx) {
                              if let MacroAction::Click { coordinate, .. } = action {
                                   *coordinate = Some((x, y));
                                   self.worker.set_status(&format!("Click position set: ({}, {})", x, y));
                              }
                         }
                    }
                }
            }
        } else {
            // Disconnected logic
            if self.worker.is_running() {
                self.worker.stop();
                self.worker.set_status("Disconnected");
            }
        }
        
        // Repaint if calibrating
        if self.ocr_region_calibration.is_active() || self.calibration.is_active() {
            ctx.request_repaint();
        }

        let is_running = self.worker.is_running();
        let status = self.worker.get_status();
        let is_ocr_calibrating = self.ocr_region_calibration.is_active();
        let is_ocr_waiting = self.ocr_region_calibration.is_waiting_for_second_click();
        
        // Get the latest OCR result and match status
        let ocr_result = self.last_ocr_result.lock().unwrap().clone();
        let match_found = *self.match_found.lock().unwrap();

        let action = render_ui(
            ui,
            mut_settings,
            is_ocr_calibrating,
            is_ocr_waiting,
            self.calibrating_action_index,
            is_running,
            &status,
            &ocr_result,
            match_found,
            game_hwnd.is_some(),
        );

        match action {
            OcrMacroUiAction::StartOcrRegionCalibration => {
                self.ocr_region_calibration.start_area();
                self.worker.set_status("Click TOP-LEFT corner of OCR region");
            },
            OcrMacroUiAction::CancelOcrRegionCalibration => {
                self.ocr_region_calibration.cancel();
                self.worker.set_status("OCR region calibration cancelled");
            },
            OcrMacroUiAction::ClearOcrRegion => {
                mut_settings.ocr_region = None;
                *self.last_ocr_result.lock().unwrap() = String::new();
            },
            OcrMacroUiAction::StartActionCalibration(idx) => {
                self.calibrating_action_index = Some(idx);
                self.calibration.start_point();
                self.worker.set_status("Click the action target position");
            },
            OcrMacroUiAction::CancelCalibration => {
                self.calibration.cancel();
                self.calibrating_action_index = None;
                self.worker.set_status("Calibration cancelled");
            },
            OcrMacroUiAction::Start => {
                if game_hwnd.is_none() {
                    self.worker.set_status("Connect to game first");
                } else {
                    // Reset match found flag
                    *self.match_found.lock().unwrap() = false;
                    self.start(settings, game_hwnd);
                }
            },
            OcrMacroUiAction::Stop => {
                self.stop();
            },
            OcrMacroUiAction::None => {}
        }
    }
}

impl OcrMacroTool {
    fn start_ocr_macro(&mut self, settings: OcrMacroSettings, game_hwnd: HWND) {
        self.worker.set_status("Loading OCR models...");
        
        // Clear previous results
        *self.last_ocr_result.lock().unwrap() = String::new();
        *self.match_found.lock().unwrap() = false;
        
        let ocr_result = Arc::clone(&self.last_ocr_result);
        let match_found = Arc::clone(&self.match_found);
        
        self.worker.start(move |running: Arc<Mutex<bool>>, status: Arc<Mutex<String>>| {
             // 0. Initialize Context (for keyboard/mouse move)
            use crate::automation::context::AutomationContext;
            let mut ctx = match AutomationContext::new(game_hwnd) {
                Ok(c) => c,
                Err(e) => {
                    *status.lock().unwrap() = format!("Error: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };

            // Load models from embedded bytes
            let detection_model = match rten::Model::load(DETECTION_MODEL_BYTES.to_vec()) {
                Ok(m) => m,
                Err(e) => {
                    *status.lock().unwrap() = format!("Detection model error: {:?}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };
            
            let recognition_model = match rten::Model::load(RECOGNITION_MODEL_BYTES.to_vec()) {
                Ok(m) => m,
                Err(e) => {
                    *status.lock().unwrap() = format!("Recognition model error: {:?}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };
            
            // Select decode method (greedy vs beam search)
            let decode_method = match settings.decode_mode {
                OcrDecodeMode::Greedy => DecodeMethod::Greedy,
                OcrDecodeMode::BeamSearch => {
                    let width = settings.beam_width.max(2);
                    DecodeMethod::BeamSearch { width }
                }
            };

            // Initialize OCR engine
            let ocr_engine = match ocrs::OcrEngine::new(ocrs::OcrEngineParams {
                detection_model: Some(detection_model),
                recognition_model: Some(recognition_model),
                decode_method,
                ..Default::default()
            }) {
                Ok(engine) => engine,
                Err(e) => {
                    *status.lock().unwrap() = format!("OCR Engine error: {:?}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };
            
            *status.lock().unwrap() = "OCR Macro running...".to_string();
            
            while *running.lock().unwrap() {
                // 1. Execute Reroll Actions Sequence
                for action in &settings.reroll_actions {
                     if !*running.lock().unwrap() { break; }
                     
                     match action {
                        MacroAction::Click { coordinate, button: _, click_method, use_mouse_movement: _ } => {
                            if let Some((x, y)) = coordinate {
                                match click_method {
                                    crate::settings::ClickMethod::SendMessage => {
                                        crate::core::input::click_at_position(game_hwnd, *x, *y);
                                    },
                                    crate::settings::ClickMethod::PostMessage => {
                                        crate::core::input::click_at_position_post(game_hwnd, *x, *y);
                                    },
                                    crate::settings::ClickMethod::MouseMovement => {
                                        // Use AutomationContext
                                         use crate::automation::interaction::click_at_screen;
                                         click_at_screen(&mut ctx.gui, *x as u32, *y as u32);
                                    },
                                }
                            }
                        },
                        MacroAction::TypeText { text } => {
                            if let Err(e) = ctx.gui.keyboard_input(text) {
                                *status.lock().unwrap() = format!("Keyboard error: {:?}", e);
                            }
                        },
                        MacroAction::Delay { milliseconds } => {
                            delay_ms(*milliseconds);
                        },
                        MacroAction::OcrSearch { .. } => {
                            // Not used in OCR macro reroll actions
                        },
                    }
                }
                
                // 2. Main Interval
                std::thread::sleep(std::time::Duration::from_millis(settings.interval_ms));
                
                // 3. Capture OCR region & Process
                if let Some(region) = settings.ocr_region {
                    match capture_region(game_hwnd, region) {
                        Ok(img) => {
                            let mut processed_img = image::DynamicImage::ImageRgb8(img);
                            
                            // Apply image preprocessing
                            if settings.invert_colors {
                                processed_img.invert();
                            }
                            
                            if settings.grayscale {
                                processed_img = image::DynamicImage::ImageLuma8(processed_img.to_luma8());
                            }
                            
                            if settings.scale_factor > 1 {
                                let (w, h) = (processed_img.width(), processed_img.height());
                                processed_img = processed_img.resize(
                                    w * settings.scale_factor,
                                    h * settings.scale_factor,
                                    image::imageops::FilterType::Lanczos3
                                );
                            }

                            let rgb_img = processed_img.into_rgb8();
                            let (width, height) = rgb_img.dimensions();
                            
                            let img_source = match ocrs::ImageSource::from_bytes(rgb_img.as_raw(), (width, height)) {
                                Ok(src) => src,
                                Err(e) => {
                                    *status.lock().unwrap() = format!("Image Error: {:?}", e);
                                    continue;
                                }
                            };
                            
                            let ocr_input = match ocr_engine.prepare_input(img_source) {
                                Ok(input) => input,
                                Err(e) => {
                                    *status.lock().unwrap() = format!("Prep Error: {:?}", e);
                                    continue;
                                }
                            };
                            
                            match ocr_engine.get_text(&ocr_input) {
                                Ok(text) => {
                                    *ocr_result.lock().unwrap() = text.clone();
                                    
                                    if let Some((detected_stat, detected_value)) = parse_ocr_result(&text) {
                                        let matched = match settings.name_match_mode {
                                            OcrNameMatchMode::Exact => {
                                                matches_target(
                                                    &detected_stat,
                                                    detected_value,
                                                    &settings.target_stat,
                                                    settings.target_value,
                                                    settings.comparison,
                                                )
                                            }
                                            OcrNameMatchMode::Contains => {
                                                let detected = detected_stat.to_lowercase();
                                                let target = settings.target_stat.to_lowercase().trim().to_string();
                                                if target.is_empty() {
                                                    false
                                                } else if !detected.contains(&target) {
                                                    false
                                                } else {
                                                    match settings.comparison {
                                                        ComparisonMode::Equals => detected_value == settings.target_value,
                                                        ComparisonMode::GreaterThanOrEqual => detected_value >= settings.target_value,
                                                        ComparisonMode::LessThanOrEqual => detected_value <= settings.target_value,
                                                    }
                                                }
                                            }
                                        };

                                        if matched {
                                            *match_found.lock().unwrap() = true;
                                            *status.lock().unwrap() = format!("MATCH FOUND! {} {}", detected_stat, detected_value);
                                            *running.lock().unwrap() = false;
                                            break; // Stop!
                                        } else {
                                            *status.lock().unwrap() = format!("Searching... ({} {})", detected_stat, detected_value);
                                        }
                                    } else {
                                        *status.lock().unwrap() = "Searching... (no parse)".to_string();
                                    }
                                },
                                Err(e) => {
                                    *status.lock().unwrap() = format!("OCR Error: {:?}", e);
                                }
                            }
                        },
                        Err(e) => {
                            *status.lock().unwrap() = format!("Capture Error: {}", e);
                        }
                    }
                }
            }
            
            if !*match_found.lock().unwrap() && !*running.lock().unwrap() && !status.lock().unwrap().contains("MATCH FOUND") {
                *status.lock().unwrap() = "Stopped".to_string();
            }
        });
    }
}
