use std::sync::{Arc, Mutex};
use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::settings::{CustomMacroSettings, MacroAction, OcrDecodeMode, OcrNameMatchMode, ComparisonMode};
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
    ocr_region_calibration: CalibrationManager,
    ocr_calibrating_action_index: Option<usize>,
}

impl CustomMacroTool {
    pub fn new(macro_index: usize) -> Self {
        Self {
            macro_index,
            worker: Worker::new(),
            calibration: CalibrationManager::new(),
            calibrating_action_index: None,
            ocr_region_calibration: CalibrationManager::new(),
            ocr_calibrating_action_index: None,
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
                            if let MacroAction::Click { coordinate, .. } = action {
                                *coordinate = Some((x, y));
                                self.worker.set_status(&format!("Click position set: ({}, {})", x, y));
                            }
                        }
                    }
                }
            }

            if let Some(result) = self.ocr_region_calibration.update(hwnd) {
                if let CalibrationResult::Area(l, t, w, h) = result {
                    if let Some(idx) = self.ocr_calibrating_action_index.take() {
                        if let Some(action) = macro_settings.settings.actions.get_mut(idx) {
                            if let MacroAction::OcrSearch { ocr_region, .. } = action {
                                *ocr_region = Some((l, t, w, h));
                                self.worker.set_status("OCR region calibrated");
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
        let click_calibrating_index = self.calibrating_action_index;
        let ocr_calibrating_index = self.ocr_calibrating_action_index;

        let action = render_ui(
            ui,
            macro_settings,
            click_calibrating_index,
            ocr_calibrating_index,
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
            CustomMacroUiAction::StartOcrRegionCalibration(action_index) => {
                self.ocr_calibrating_action_index = Some(action_index);
                self.ocr_region_calibration.start_area();
                self.worker.set_status("Click TOP-LEFT corner of OCR region");
            },
            CustomMacroUiAction::CancelOcrRegionCalibration => {
                self.ocr_region_calibration.cancel();
                self.ocr_calibrating_action_index = None;
                self.worker.set_status("OCR region calibration cancelled");
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
            use crate::core::screen_capture::capture_region;
            use crate::core::ocr_parser::{parse_ocr_result, matches_target};
            use ocrs::{OcrEngine, OcrEngineParams, ImageSource, DecodeMethod};

            let mut ctx = match AutomationContext::new(game_hwnd) {
                Ok(c) => c,
                Err(e) => {
                    *status.lock().unwrap() = format!("Error: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };

            // Initialize OCR engine only if needed
            let has_ocr_actions = settings.actions.iter().any(|a| matches!(a, MacroAction::OcrSearch { .. }));
            let mut ocr_engine: Option<OcrEngine> = None;

            if has_ocr_actions {
                *status.lock().unwrap() = "Loading OCR models...".to_string();

                // Determine decode configuration from first OCR action
                let mut decode_mode_cfg = OcrDecodeMode::Greedy;
                let mut beam_width_cfg: u32 = 10;
                for a in &settings.actions {
                    if let MacroAction::OcrSearch { decode_mode, beam_width, .. } = a {
                        decode_mode_cfg = *decode_mode;
                        beam_width_cfg = *beam_width;
                        break;
                    }
                }

                // Embed the OCR models directly into the binary (same as OCR macro)
                const DETECTION_MODEL_BYTES: &[u8] = include_bytes!("../models/text-detection.rten");
                const RECOGNITION_MODEL_BYTES: &[u8] = include_bytes!("../models/text-recognition.rten");

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

                let dm = match decode_mode_cfg {
                    OcrDecodeMode::Greedy => DecodeMethod::Greedy,
                    OcrDecodeMode::BeamSearch => {
                        let width = beam_width_cfg.max(2);
                        DecodeMethod::BeamSearch { width }
                    }
                };

                let engine = match OcrEngine::new(OcrEngineParams {
                    detection_model: Some(detection_model),
                    recognition_model: Some(recognition_model),
                    decode_method: dm,
                    ..Default::default()
                }) {
                    Ok(engine) => engine,
                    Err(e) => {
                        *status.lock().unwrap() = format!("OCR Engine error: {:?}", e);
                        *running.lock().unwrap() = false;
                        return;
                    }
                };

                ocr_engine = Some(engine);
            }

            let mut iteration: u32 = 0;

            loop {
                if !*running.lock().unwrap() {
                    break;
                }

                // Determine if we should exit based on loop settings
                if settings.loop_enabled {
                    if !settings.infinite_loop && iteration >= settings.loop_count {
                        break;
                    }
                    if settings.infinite_loop {
                         *status.lock().unwrap() = format!("Loop {} (Infinite)", iteration + 1);
                    } else {
                         *status.lock().unwrap() = format!("Loop {}/{}", iteration + 1, settings.loop_count);
                    }
                } else {
                    if iteration >= 1 {
                        break;
                    }
                }

                for (idx, action) in settings.actions.iter().enumerate() {
                    if !*running.lock().unwrap() {
                        break;
                    }

                    match action {
                        MacroAction::Click { coordinate, button: _, click_method, use_mouse_movement: _ } => {
                            if let Some((x, y)) = coordinate {
                                *status.lock().unwrap() = format!("Clicking at ({}, {})", x, y);

                                match click_method {
                                    crate::settings::ClickMethod::SendMessage => {
                                        // Direct click without mouse movement (default)
                                        click_at_position(game_hwnd, *x, *y);
                                    },
                                    crate::settings::ClickMethod::PostMessage => {
                                        // Async click without mouse movement
                                        use crate::core::input::click_at_position_post;
                                        click_at_position_post(game_hwnd, *x, *y);
                                    },
                                    crate::settings::ClickMethod::MouseMovement => {
                                        // Use screen coordinates with mouse movement
                                        use crate::automation::interaction::click_at_screen;
                                        click_at_screen(&mut ctx.gui, *x as u32, *y as u32);
                                    },
                                }
                            } else {
                                *status.lock().unwrap() = format!("Action {}: Click position not set", idx + 1);
                            }
                        },
                        MacroAction::TypeText { text } => {
                            *status.lock().unwrap() = format!("Typing: {}", text);
                            if let Err(e) = ctx.gui.keyboard_input(text) {
                                *status.lock().unwrap() = format!("Keyboard error: {:?}", e);
                            }
                        },
                        MacroAction::Delay { milliseconds } => {
                            *status.lock().unwrap() = format!("Waiting {}ms", milliseconds);
                            delay_ms(*milliseconds);
                        },
                        MacroAction::OcrSearch {
                            ocr_region,
                            scale_factor,
                            invert_colors,
                            grayscale,
                            target_stat,
                            target_value,
                            comparison,
                            name_match_mode,
                            ..
                        } => {
                            if ocr_engine.is_none() {
                                *status.lock().unwrap() = "OCR engine not initialized".to_string();
                                *running.lock().unwrap() = false;
                                break;
                            }

                            let region = if let Some(region) = ocr_region {
                                *region
                            } else {
                                *status.lock().unwrap() = format!("Action {}: OCR region not set", idx + 1);
                                *running.lock().unwrap() = false;
                                break;
                            };

                            let engine = ocr_engine.as_ref().unwrap();

                            match capture_region(game_hwnd, region) {
                                Ok(img) => {
                                    let mut processed_img = image::DynamicImage::ImageRgb8(img);

                                    if *invert_colors {
                                        processed_img.invert();
                                    }

                                    if *grayscale {
                                        processed_img = image::DynamicImage::ImageLuma8(processed_img.to_luma8());
                                    }

                                    if *scale_factor > 1 {
                                        let (w, h) = (processed_img.width(), processed_img.height());
                                        processed_img = processed_img.resize(
                                            w * *scale_factor,
                                            h * *scale_factor,
                                            image::imageops::FilterType::Lanczos3,
                                        );
                                    }

                                    let rgb_img = processed_img.into_rgb8();
                                    let (width, height) = rgb_img.dimensions();

                                    let img_source = match ImageSource::from_bytes(rgb_img.as_raw(), (width, height)) {
                                        Ok(src) => src,
                                        Err(e) => {
                                            *status.lock().unwrap() = format!("Image Error: {:?}", e);
                                            continue;
                                        }
                                    };

                                    let ocr_input = match engine.prepare_input(img_source) {
                                        Ok(input) => input,
                                        Err(e) => {
                                            *status.lock().unwrap() = format!("Prep Error: {:?}", e);
                                            continue;
                                        }
                                    };

                                    match engine.get_text(&ocr_input) {
                                        Ok(text) => {
                                            *status.lock().unwrap() = format!("OCR: {}", text);

                                            if let Some((detected_stat, detected_value)) = parse_ocr_result(&text) {
                                                let matched = match name_match_mode {
                                                    OcrNameMatchMode::Exact => {
                                                        matches_target(
                                                            &detected_stat,
                                                            detected_value,
                                                            target_stat,
                                                            *target_value,
                                                            *comparison,
                                                        )
                                                    }
                                                    OcrNameMatchMode::Contains => {
                                                        let detected = detected_stat.to_lowercase();
                                                        let target = target_stat.to_lowercase().trim().to_string();
                                                        if target.is_empty() {
                                                            false
                                                        } else if !detected.contains(&target) {
                                                            false
                                                        } else {
                                                            match comparison {
                                                                ComparisonMode::Equals => detected_value == *target_value,
                                                                ComparisonMode::GreaterThanOrEqual => detected_value >= *target_value,
                                                                ComparisonMode::LessThanOrEqual => detected_value <= *target_value,
                                                            }
                                                        }
                                                    }
                                                };

                                                if matched {
                                                    *status.lock().unwrap() =
                                                        format!("MATCH FOUND! {} {}", detected_stat, detected_value);
                                                    *running.lock().unwrap() = false;
                                                    break;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            *status.lock().unwrap() = format!("OCR Error: {:?}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    *status.lock().unwrap() = format!("Capture Error: {}", e);
                                }
                            }
                        },
                    }
                }

                iteration += 1;
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
