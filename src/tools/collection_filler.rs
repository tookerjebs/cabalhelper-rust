use std::sync::{Arc, Mutex};
use std::thread;
use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::settings::CollectionFillerSettings;
use crate::tools::r#trait::Tool;
use crate::calibration::CalibrationManager;
use crate::automation::context::AutomationContext;
use crate::automation::detection::find_stored_template;
use crate::automation::interaction::{click_at_screen, delay_ms, scroll_in_area, click_at_window_pos};
use crate::ui::collection_filler::{CalibrationItem, UiAction, apply_calibration_result, clear_calibration};

pub struct CollectionFillerTool {
    // Runtime state
    running: Arc<Mutex<bool>>,
    status: Arc<Mutex<String>>,
    game_hwnd: Option<HWND>,
    
    // Calibration
    calibration: CalibrationManager,
    calibrating_item: Option<CalibrationItem>,
    
    // UI State
    red_dot_path: String,
}

impl Default for CollectionFillerTool {
    fn default() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Ready - Calibrate all items before starting".to_string())),
            game_hwnd: None,
            calibration: CalibrationManager::new(),
            calibrating_item: None,
            red_dot_path: "red-dot.png".to_string(),
        }
    }
}

impl Tool for CollectionFillerTool {
    fn set_game_hwnd(&mut self, hwnd: Option<HWND>) {
        self.game_hwnd = hwnd;
        if hwnd.is_none() {
            *self.running.lock().unwrap() = false;
            self.calibration.cancel();
            self.calibrating_item = None;
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

impl CollectionFillerTool {
    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, settings: &mut CollectionFillerSettings) {
        // Handle calibration interaction
        if let Some(hwnd) = self.game_hwnd {
            if let Some(result) = self.calibration.handle_clicks(hwnd) {
                if let Some(item) = self.calibrating_item.take() {
                    apply_calibration_result(result, item, settings);
                    *self.status.lock().unwrap() = "Calibration recorded".to_string();
                }
            }
        }

        let is_running = *self.running.lock().unwrap();
        let status = self.status.lock().unwrap().clone();
        
        // Render UI and get action
        let action = crate::ui::collection_filler::render_ui(
            ui,
            ctx,
            settings,
            &self.calibration,
            &self.calibrating_item,
            is_running,
            &status,
            self.game_hwnd.is_some(),
        );

        // Handle action
        match action {
            UiAction::StartCalibration(item, is_area) => {
                self.calibrating_item = Some(item.clone());
                if is_area {
                    self.calibration.start_area();
                    *self.status.lock().unwrap() = "Click TOP-LEFT corner".to_string();
                } else {
                    self.calibration.start_point();
                    *self.status.lock().unwrap() = "Click the button".to_string();
                }
            },
            UiAction::CancelCalibration => {
                self.calibration.cancel();
                self.calibrating_item = None;
                *self.status.lock().unwrap() = "Calibration cancelled".to_string();
            },
            UiAction::ClearCalibration(item) => {
                 clear_calibration(item, settings);
            },
            UiAction::StartAutomation => {
                if self.is_fully_calibrated(settings) {
                    self.start_automation(settings.clone());
                } else {
                    *self.status.lock().unwrap() = "Please calibrate all required items first".to_string();
                }
            },
            UiAction::StopAutomation => {
                self.stop();
            },
            UiAction::None => {}
        }
    }

    fn is_fully_calibrated(&self, settings: &CollectionFillerSettings) -> bool {
        settings.collection_tabs_area.is_some() &&
        settings.dungeon_list_area.is_some() &&
        settings.collection_items_area.is_some() &&
        settings.auto_refill_pos.is_some() &&
        settings.register_pos.is_some() &&
        settings.yes_pos.is_some()
    }

    pub fn start(&mut self, settings: &CollectionFillerSettings) {
        if self.is_fully_calibrated(settings) {
            self.start_automation(settings.clone());
        } else {
            *self.status.lock().unwrap() = "Please calibrate all required items first".to_string();
        }
    }

    fn start_automation(&mut self, settings: CollectionFillerSettings) {
        let running = Arc::clone(&self.running);
        let status = Arc::clone(&self.status);
        let game_hwnd = self.game_hwnd.unwrap();
        let red_dot_path = self.red_dot_path.clone();

        *running.lock().unwrap() = true;
        *status.lock().unwrap() = "Starting automation...".to_string();

        thread::spawn(move || {
            let mut ctx = match AutomationContext::new(game_hwnd) {
                Ok(c) => c,
                Err(e) => {
                    *status.lock().unwrap() = format!("Error: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };

            // Load templates
            let res = (|| -> Result<(), String> {
                ctx.store_template(&red_dot_path, settings.collection_tabs_area, "tabs_dots")?;
                ctx.store_template(&red_dot_path, settings.dungeon_list_area, "dungeon_dots")?;
                ctx.store_template(&red_dot_path, settings.collection_items_area, "items_dots")?;
                Ok(())
            })();

            if let Err(e) = res {
                *status.lock().unwrap() = format!("Template Error: {}", e);
                *running.lock().unwrap() = false;
                return;
            }

            *status.lock().unwrap() = "Scanning tabs...".to_string();

            run_automation_loop(&mut ctx, settings, &running, &status);

            *running.lock().unwrap() = false;
            *status.lock().unwrap() = "Finished".to_string();
        });
    }
}

// Automation logic (non-UI)
fn run_automation_loop(
    ctx: &mut AutomationContext,
    settings: CollectionFillerSettings,
    running: &Arc<Mutex<bool>>,
    status: &Arc<Mutex<String>>
) {
     while *running.lock().unwrap() {
        match find_stored_template(&mut ctx.gui, "tabs_dots", settings.red_dot_tolerance) {
            Some(dots) if !dots.is_empty() => {
                let tab_pos = dots[0];
                *status.lock().unwrap() = "Found tab, clicking...".to_string();
                click_at_screen(&mut ctx.gui, tab_pos.0, tab_pos.1);
                delay_ms(settings.delay_ms);

                 process_dungeon_list(ctx, &settings, running, status, tab_pos);
            },
            _ => {
                *status.lock().unwrap() = "All collections complete!".to_string();
                break;
            }
        }
        delay_ms(settings.delay_ms);
     }
}

fn process_dungeon_list(
    ctx: &mut AutomationContext,
    settings: &CollectionFillerSettings,
    running: &Arc<Mutex<bool>>,
    status: &Arc<Mutex<String>>,
    original_tab_pos: (u32, u32)
) {
    let mut current_page = 1;
    let mut pages_checked_this_cycle = 0;
    
    let tab_check = |gui: &mut rustautogui::RustAutoGui| -> bool {
         find_stored_template(gui, "tabs_dots", settings.red_dot_tolerance)
            .map(|dots| dots.iter().any(|d| {
                 ((d.0 as f32 - original_tab_pos.0 as f32).powi(2) + (d.1 as f32 - original_tab_pos.1 as f32).powi(2)).sqrt() < 20.0
            })).unwrap_or(false)
    };

    while *running.lock().unwrap() && tab_check(&mut ctx.gui) {
        *status.lock().unwrap() = format!("Processing page {}", current_page);
        
        let found_work = process_page_dungeons(ctx, settings, running, status);
        
        if found_work {
            current_page = 1;
            pages_checked_this_cycle = 0;
        } else {
             pages_checked_this_cycle += 1;
             
             if current_page < 4 {
                 current_page += 1;
                 let btn = match current_page {
                     2 => settings.page_2_pos,
                     3 => settings.page_3_pos,
                     4 => settings.page_4_pos,
                     _ => None
                 };
                 if let Some((x, y)) = btn {
                     click_at_window_pos(&mut ctx.gui, ctx.game_hwnd, x, y);
                     delay_ms(settings.delay_ms);
                 }
             } else {
                 if pages_checked_this_cycle >= 4 {
                      if let Some((x, y)) = settings.arrow_right_pos {
                          click_at_window_pos(&mut ctx.gui, ctx.game_hwnd, x, y);
                          delay_ms(settings.delay_ms);
                          current_page = 1; 
                      } else {
                          break;
                      }
                 }
             }
             
             if pages_checked_this_cycle > 8 { 
                 break;
             }
        }
    }
}

fn process_page_dungeons(
    ctx: &mut AutomationContext,
    settings: &CollectionFillerSettings,
    running: &Arc<Mutex<bool>>,
    status: &Arc<Mutex<String>>
) -> bool {
    let mut work_done = false;
    
    match find_stored_template(&mut ctx.gui, "dungeon_dots", settings.red_dot_tolerance) {
        Some(dots) if !dots.is_empty() => {
             let d_pos = dots[0];
             click_at_screen(&mut ctx.gui, d_pos.0, d_pos.1);
             delay_ms(settings.delay_ms);
             
             if let Some(items_area) = settings.collection_items_area {
                 scroll_in_area(&mut ctx.gui, ctx.game_hwnd, items_area, -20);
             }
             delay_ms(settings.delay_ms);
             
             for _ in 0..50 {
                 if !*running.lock().unwrap() { break; }
                 
                 let _processed = process_visible_items(ctx, settings, running, status);
                 
                 if let Some(items_area) = settings.collection_items_area {
                     scroll_in_area(&mut ctx.gui, ctx.game_hwnd, items_area, 5);
                 }
                 delay_ms(settings.delay_ms);
             }
             
             work_done = true;
        },
        _ => {}
    }
    work_done
}

fn process_visible_items(
    ctx: &mut AutomationContext,
    settings: &CollectionFillerSettings,
    running: &Arc<Mutex<bool>>,
    status: &Arc<Mutex<String>>
) -> bool {
    let mut processed = false;
    let mut last_pos: Option<(u32, u32)> = None;
    
    while *running.lock().unwrap() {
        match find_stored_template(&mut ctx.gui, "items_dots", settings.red_dot_tolerance) {
            Some(dots) if !dots.is_empty() => {
                let pos = dots[0];
                
                if let Some(last) = last_pos {
                     let dist = ((pos.0 as f32 - last.0 as f32).powi(2) + (pos.1 as f32 - last.1 as f32).powi(2)).sqrt();
                     if dist < 5.0 {
                         *status.lock().unwrap() = "Stuck on item, skipping".to_string();
                         break;
                     }
                }
                last_pos = Some(pos);
                
                click_at_screen(&mut ctx.gui, pos.0, pos.1);
                delay_ms(settings.delay_ms);
                
                let btns = [settings.auto_refill_pos, settings.register_pos, settings.yes_pos];
                for btn in btns {
                    if let Some((x, y)) = btn {
                        click_at_window_pos(&mut ctx.gui, ctx.game_hwnd, x, y);
                         delay_ms(settings.delay_ms);
                    }
                }
                
                processed = true;
                delay_ms(settings.delay_ms * 2);
            },
            _ => break
        }
    }
    processed
}
