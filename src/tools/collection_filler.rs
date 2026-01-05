use std::sync::{Arc, Mutex};
use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::settings::CollectionFillerSettings;
use crate::tools::r#trait::Tool;
use crate::calibration::CalibrationManager;
use crate::automation::context::AutomationContext;
use crate::automation::detection::{find_stored_template, is_position_near};
use crate::automation::interaction::{click_at_screen, delay_ms, scroll_in_area, click_at_window_pos};
use crate::ui::collection_filler::{CalibrationItem, UiAction, apply_calibration_result, clear_calibration};
use crate::core::worker::Worker;

pub struct CollectionFillerTool {
    // Runtime state (Worker)
    worker: Worker,
    
    // Calibration
    calibration: CalibrationManager,
    calibrating_item: Option<CalibrationItem>,
}

impl Default for CollectionFillerTool {
    fn default() -> Self {
        Self {
            worker: Worker::new(),
            calibration: CalibrationManager::new(),
            calibrating_item: None,
        }
    }
}

impl Tool for CollectionFillerTool {
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
         let settings = &app_settings.collection_filler;
         
         if self.is_fully_calibrated(settings) {
             if let Some(hwnd) = game_hwnd {
                 self.start_automation(settings.clone(), hwnd);
             } else {
                  self.worker.set_status("Connect to game first");
             }
         } else {
             self.worker.set_status("Please calibrate all required items first");
         }
    }

    fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, settings: &mut crate::settings::AppSettings, game_hwnd: Option<HWND>) {
        let settings = &mut settings.collection_filler;
        
        // Handle calibration interaction
        if let Some(hwnd) = game_hwnd {
            if let Some(result) = self.calibration.update(hwnd) {
                if let Some(item) = self.calibrating_item.take() {
                    apply_calibration_result(result, item, settings);
                    self.worker.set_status("Calibration recorded");
                }
            }
        } else {
             // Disconnected logic
             if self.worker.is_running() {
                 self.worker.stop();
                 self.worker.set_status("Disconnected");
             }
             self.calibration.cancel();
             self.calibrating_item = None;
        }

        let is_running = self.worker.is_running();
        let status = self.worker.get_status();
        
        // Render UI and get action
        let action = crate::ui::collection_filler::render_ui(
            ui,
            ctx,
            settings,
            &self.calibration,
            &self.calibrating_item,
            is_running,
            &status,
            game_hwnd.is_some(),
        );

        // Handle action
        match action {
            UiAction::StartCalibration(item, is_area) => {
                self.calibrating_item = Some(item.clone());
                if is_area {
                    self.calibration.start_area();
                    self.worker.set_status("Click TOP-LEFT corner");
                } else {
                    self.calibration.start_point();
                    self.worker.set_status("Click the button");
                }
            },
            UiAction::CancelCalibration => {
                self.calibration.cancel();
                self.calibrating_item = None;
                self.worker.set_status("Calibration cancelled");
            },
            UiAction::ClearCalibration(item) => {
                 clear_calibration(item, settings);
            },
            UiAction::StartAutomation => {
                if self.is_fully_calibrated(settings) {
                    // Need game_hwnd here
                    if let Some(hwnd) = game_hwnd {
                        self.start_automation(settings.clone(), hwnd);
                    } else {
                         self.worker.set_status("Connect to game first");
                    }
                } else {
                    self.worker.set_status("Please calibrate all required items first");
                }
            },
            UiAction::StopAutomation => {
                self.stop();
            },
            UiAction::None => {}
        }
    }
}

impl CollectionFillerTool {
    fn is_fully_calibrated(&self, settings: &CollectionFillerSettings) -> bool {
        settings.collection_tabs_area.is_some() &&
        settings.dungeon_list_area.is_some() &&
        settings.collection_items_area.is_some() &&
        settings.auto_refill_pos.is_some() &&
        settings.register_pos.is_some() &&
        settings.yes_pos.is_some()
    }

    // start method removed as it's now internal to UiAction handling

    fn start_automation(&mut self, settings: CollectionFillerSettings, game_hwnd: HWND) {
        self.worker.set_status("Starting automation...");
        let red_dot_path = settings.red_dot_path.clone();

        self.worker.start(move |running: Arc<Mutex<bool>>, status: Arc<Mutex<String>>| {
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
        // Find potential tab dots (using lower tolerance to catch all candidates)
        let potential_dots = match find_stored_template(&mut ctx.gui, "tabs_dots", settings.red_dot_tolerance) {
            Some(dots) if !dots.is_empty() => dots,
            _ => {
                *status.lock().unwrap() = "All collections complete!".to_string();
                break;
            }
        };
        
        // Filter by color to keep only RED dots (not grey dots)
        let red_dots = crate::automation::detection::filter_red_dots(
            potential_dots,
            settings.min_red,
            settings.red_dominance
        );
        
        if red_dots.is_empty() {
            *status.lock().unwrap() = "All collections complete!".to_string();
            break;
        }
        
        let tab_pos = red_dots[0];
        *status.lock().unwrap() = "Found tab, clicking...".to_string();
        click_at_screen(&mut ctx.gui, tab_pos.0, tab_pos.1);
        delay_ms(settings.delay_ms);

         process_dungeon_list(ctx, &settings, running, status, tab_pos);
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
    let mut any_work_done = false;

    // Loop until no more red dots found in dungeon list on this page
    while *running.lock().unwrap() {
        // Find potential dungeon dots and filter by color
        let potential_dots = match find_stored_template(&mut ctx.gui, "dungeon_dots", settings.red_dot_tolerance) {
            Some(dots) if !dots.is_empty() => dots,
            _ => break // No more dungeons on this page
        };
        
        let red_dots = crate::automation::detection::filter_red_dots(
            potential_dots,
            settings.min_red,
            settings.red_dominance
        );
        
        if red_dots.is_empty() {
            break; // No red dungeons on this page
        }
        
        let dungeon_dot = red_dots[0];

        // Found a dungeon with a red dot
        *status.lock().unwrap() = "Processing dungeon...".to_string();
        click_at_screen(&mut ctx.gui, dungeon_dot.0, dungeon_dot.1);
        delay_ms(settings.delay_ms);
        // Note: No scroll-up needed - game UI always starts at top when entering dungeon

        let max_scroll_passes = 50;
        let mut dungeon_finished = false;

        for _ in 0..max_scroll_passes {
            if !*running.lock().unwrap() { break; }

            // 1. Process all visible items at current scroll
            let _ = process_visible_items(ctx, settings, running, status);
            any_work_done = true;

            // 2. Double check item area for stragglers (Python logic compliance)
            let _ = process_visible_items(ctx, settings, running, status);

            // 3. Check if THIS dungeon is complete
            // We scan the dungeon list again to see if our dungeon_dot is still red
            let still_active = match find_stored_template(&mut ctx.gui, "dungeon_dots", settings.red_dot_tolerance) {
                Some(dots) => dots.iter().any(|d| is_position_near(*d, dungeon_dot, 20.0)),
                None => false
            };

            if !still_active {
                dungeon_finished = true;
                break; // Dungeon done!
            }

            // 4. Scroll down to find more items (1 tick = 1 row in game)
            if let Some(items_area) = settings.collection_items_area {
                scroll_in_area(&mut ctx.gui, ctx.game_hwnd, items_area, 1);
            }
            delay_ms(settings.delay_ms);
        }

        if !dungeon_finished {
            // Safe guard: if we scrolled 50 times and it's still red, maybe we're stuck.
            // But we break the inner loop to move to next dungeon check (or see it again)
             *status.lock().unwrap() = "Dungeon timeout/stuck, scanning list again...".to_string();
        }
    }
    
    any_work_done
}

fn process_visible_items(
    ctx: &mut AutomationContext,
    settings: &CollectionFillerSettings,
    running: &Arc<Mutex<bool>>,
    status: &Arc<Mutex<String>>
) -> bool {
    let mut processed = false;
    let mut last_pos: Option<(u32, u32)> = None;
    let mut stuck_hits = 0;
    
    while *running.lock().unwrap() {
        // Find potential item dots and filter by color
        let potential_dots = match find_stored_template(&mut ctx.gui, "items_dots", settings.red_dot_tolerance) {
            Some(dots) if !dots.is_empty() => dots,
            _ => break
        };
        
        let red_dots = crate::automation::detection::filter_red_dots(
            potential_dots,
            settings.min_red,
            settings.red_dominance
        );
        
        match red_dots.first() {
            Some(&pos) => {
                
                // Stuck check
                if let Some(last) = last_pos {
                     if is_position_near(pos, last, 5.0) {
                         stuck_hits += 1;
                         if stuck_hits >= 3 {
                             *status.lock().unwrap() = "Stuck on item, skipping".to_string();
                             break;
                         }
                     } else {
                         stuck_hits = 0;
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
                delay_ms(settings.delay_ms); 
            },
            None => break
        }
    }
    processed
}
