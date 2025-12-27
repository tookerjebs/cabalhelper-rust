use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::fs;
use eframe::egui;
use serde::{Serialize, Deserialize};
use windows::Win32::Foundation::HWND;
use rustautogui::{RustAutoGui, MatchMode};
use crate::core::input::click_at_position;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSettings {
    // Detection Areas (stored as (left, top, width, height) relative to game window)
    pub collection_tabs_area: Option<(i32, i32, i32, i32)>,
    pub dungeon_list_area: Option<(i32, i32, i32, i32)>,
    pub collection_items_area: Option<(i32, i32, i32, i32)>,
    
    // Button Coordinates (x, y relative to game window)
    pub auto_refill_pos: Option<(i32, i32)>,
    pub register_pos: Option<(i32, i32)>,
    pub yes_pos: Option<(i32, i32)>,
    pub page_2_pos: Option<(i32, i32)>,
    pub page_3_pos: Option<(i32, i32)>,
    pub page_4_pos: Option<(i32, i32)>,
    pub arrow_right_pos: Option<(i32, i32)>,
    
    // Speed and matching settings
    pub delay_ms: u64,
    #[serde(default = "default_red_dot_tolerance")]
    pub red_dot_tolerance: f32,
}

fn default_red_dot_tolerance() -> f32 {
    0.85
}

impl Default for CollectionSettings {
    fn default() -> Self {
        Self {
            collection_tabs_area: None,
            dungeon_list_area: None,
            collection_items_area: None,
            auto_refill_pos: None,
            register_pos: None,
            yes_pos: None,
            page_2_pos: None,
            page_3_pos: None,
            page_4_pos: None,
            arrow_right_pos: None,
            delay_ms: 31,
            red_dot_tolerance: 0.85,
        }
    }
}

impl CollectionSettings {
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse JSON: {}", e))
    }
    
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        
        fs::write(path, json)
            .map_err(|e| format!("Failed to write file: {}", e))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CalibrationType {
    // Areas
    CollectionTabsArea,
    DungeonListArea,
    CollectionItemsArea,
    // Buttons
    AutoRefillButton,
    RegisterButton,
    YesButton,
    Page2Button,
    Page3Button,
    Page4Button,
    ArrowRightButton,
}

pub struct CollectionFillerTool {
    // Settings (calibrated by user)
    settings: CollectionSettings,
    
    // Runtime state
    running: Arc<Mutex<bool>>,
    status: Arc<Mutex<String>>,
    game_hwnd: Option<HWND>,
    
    // Calibration state
    calibrating: Option<CalibrationType>,
    area_selection_start: Option<(i32, i32)>,
    last_mouse_state: bool,
    
    // Red dot template path
    red_dot_path: String,
    
    // UI tolerance for red dot matching (synced to settings)
    tolerance_ui: f32,
}

impl Default for CollectionFillerTool {
    fn default() -> Self {
        Self {
            settings: CollectionSettings::default(),
            running: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Ready - Calibrate all items before starting".to_string())),
            game_hwnd: None,
            calibrating: None,
            area_selection_start: None,
            last_mouse_state: false,
            red_dot_path: "red-dot.png".to_string(),
            tolerance_ui: 0.85,
        }
    }
}

impl CollectionFillerTool {
    pub fn set_game_hwnd(&mut self, hwnd: Option<HWND>) {
        self.game_hwnd = hwnd;
        if hwnd.is_none() {
            *self.running.lock().unwrap() = false;
            self.calibrating = None;
            *self.status.lock().unwrap() = "Disconnected".to_string();
        }
    }

    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
        *self.status.lock().unwrap() = "Stopped (ESC pressed)".to_string();
    }

    pub fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("Collection Filler");
        ui.separator();

        // Check if connected
        if self.game_hwnd.is_none() {
            ui.colored_label(egui::Color32::RED, "Please connect to game first (top right)");
            return;
        }

        // Handle calibration clicks
        self.handle_calibration_clicks();
        
        // Ensure continuous updates during calibration to catch mouse polling
        if self.calibrating.is_some() {
            ctx.request_repaint();
        }

        // Calibration Section
        ui.heading("⚙️ Calibration");
        
        // Areas
        ui.label("Detection Areas:");
        self.render_area_calibration(ui, "Tabs Area", CalibrationType::CollectionTabsArea, self.settings.collection_tabs_area);
        self.render_area_calibration(ui, "Dungeon List", CalibrationType::DungeonListArea, self.settings.dungeon_list_area);
        self.render_area_calibration(ui, "Items Area", CalibrationType::CollectionItemsArea, self.settings.collection_items_area);
        
        ui.add_space(10.0);
        
        // Buttons
        ui.label("Action Buttons:");
        self.render_button_calibration(ui, "Auto Refill", CalibrationType::AutoRefillButton, self.settings.auto_refill_pos);
        self.render_button_calibration(ui, "Register", CalibrationType::RegisterButton, self.settings.register_pos);
        self.render_button_calibration(ui, "Yes", CalibrationType::YesButton, self.settings.yes_pos);
        self.render_button_calibration(ui, "Page 2", CalibrationType::Page2Button, self.settings.page_2_pos);
        self.render_button_calibration(ui, "Page 3", CalibrationType::Page3Button, self.settings.page_3_pos);
        self.render_button_calibration(ui, "Page 4", CalibrationType::Page4Button, self.settings.page_4_pos);
        self.render_button_calibration(ui, "Arrow Right", CalibrationType::ArrowRightButton, self.settings.arrow_right_pos);

        ui.add_space(10.0);

        // Delay setting
        ui.horizontal(|ui| {
            ui.label("Delay (ms):");
            let mut delay_str = self.settings.delay_ms.to_string();
            if ui.text_edit_singleline(&mut delay_str).changed() {
                if let Ok(val) = delay_str.parse::<u64>() {
                    self.settings.delay_ms = val;
                }
            }
        });

        ui.add_space(10.0);

        // Red dot tolerance slider
        ui.horizontal(|ui| {
            ui.label("Red Dot Tolerance:");
            if ui.add(egui::Slider::new(&mut self.tolerance_ui, 0.01..=0.99)).changed() {
                self.settings.red_dot_tolerance = self.tolerance_ui;
            }
            ui.label(format!("{:.2}", self.tolerance_ui));
        });

        ui.add_space(10.0);

        // Save/Load buttons
        ui.horizontal(|ui| {
            if ui.button("💾 Save Settings").clicked() {
                match self.settings.save_to_file("collection_settings.json") {
                    Ok(_) => *self.status.lock().unwrap() = "Settings saved successfully".to_string(),
                    Err(e) => *self.status.lock().unwrap() = format!("Save failed: {}", e),
                }
            }
            if ui.button("📂 Load Settings").clicked() {
                match CollectionSettings::load_from_file("collection_settings.json") {
                    Ok(settings) => {
                        self.tolerance_ui = settings.red_dot_tolerance;
                        self.settings = settings;
                        *self.status.lock().unwrap() = "Settings loaded successfully".to_string();
                    },
                    Err(e) => *self.status.lock().unwrap() = format!("Load failed: {}", e),
                }
            }
        });

        ui.separator();

        // Control Section
        ui.heading("🎮 Control");
        
        let is_running = *self.running.lock().unwrap();
        
        if !is_running {
            if ui.button("▶️ Start").clicked() {
                if self.is_fully_calibrated() {
                    self.start_automation();
                } else {
                    *self.status.lock().unwrap() = "Please calibrate all items first".to_string();
                }
            }
        } else {
            if ui.button("⏹️ Stop").clicked() {
                *self.running.lock().unwrap() = false;
                *self.status.lock().unwrap() = "Stopping...".to_string();
            }
        }

        ui.separator();

        // Status Section
        ui.heading("📊 Status");
        ui.label(self.status.lock().unwrap().as_str());
    }

    fn render_area_calibration(&mut self, ui: &mut egui::Ui, label: &str, cal_type: CalibrationType, current: Option<(i32, i32, i32, i32)>) {
        ui.horizontal(|ui| {
            let icon = if current.is_some() { "✓" } else { " " };
            ui.label(format!("[{}] {}", icon, label));
            
            let is_calibrating = self.calibrating.as_ref() == Some(&cal_type);
            
            if is_calibrating {
                if ui.button("Cancel").clicked() {
                    self.calibrating = None;
                    self.area_selection_start = None;
                    *self.status.lock().unwrap() = "Calibration cancelled".to_string();
                }
            } else {
                if ui.button("Set").clicked() {
                    self.calibrating = Some(cal_type.clone());
                    self.area_selection_start = None;
                    self.last_mouse_state = false;
                    *self.status.lock().unwrap() = format!("Click TOP-LEFT corner of {}", label);
                }
                if current.is_some() && ui.button("Clear").clicked() {
                    match cal_type {
                        CalibrationType::CollectionTabsArea => self.settings.collection_tabs_area = None,
                        CalibrationType::DungeonListArea => self.settings.dungeon_list_area = None,
                        CalibrationType::CollectionItemsArea => self.settings.collection_items_area = None,
                        _ => {}
                    }
                }
            }
        });
    }

    fn render_button_calibration(&mut self, ui: &mut egui::Ui, label: &str, cal_type: CalibrationType, current: Option<(i32, i32)>) {
        ui.horizontal(|ui| {
            let icon = if current.is_some() { "✓" } else { " " };
            ui.label(format!("[{}] {}", icon, label));
            
            let is_calibrating = self.calibrating.as_ref() == Some(&cal_type);
            
            if is_calibrating {
                if ui.button("Cancel").clicked() {
                    self.calibrating = None;
                    *self.status.lock().unwrap() = "Calibration cancelled".to_string();
                }
            } else {
                if ui.button("Set").clicked() {
                    self.calibrating = Some(cal_type.clone());
                    self.last_mouse_state = false;
                    *self.status.lock().unwrap() = format!("Click the {} button in game", label);
                }
                if current.is_some() && ui.button("Clear").clicked() {
                    match cal_type {
                        CalibrationType::AutoRefillButton => self.settings.auto_refill_pos = None,
                        CalibrationType::RegisterButton => self.settings.register_pos = None,
                        CalibrationType::YesButton => self.settings.yes_pos = None,
                        CalibrationType::Page2Button => self.settings.page_2_pos = None,
                        CalibrationType::Page3Button => self.settings.page_3_pos = None,
                        CalibrationType::Page4Button => self.settings.page_4_pos = None,
                        CalibrationType::ArrowRightButton => self.settings.arrow_right_pos = None,
                        _ => {}
                    }
                }
            }
        });
    }

    fn handle_calibration_clicks(&mut self) {
        use crate::core::input::is_left_mouse_down;
        use crate::core::window::{get_window_under_cursor, is_game_window_or_child, get_cursor_pos, screen_to_window_coords};

        if self.calibrating.is_none() || self.game_hwnd.is_none() {
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
        let cal_type = match &self.calibrating {
            Some(t) => t.clone(),
            None => return,
        };

        match cal_type {
            // Area calibration (2 clicks)
            CalibrationType::CollectionTabsArea | 
            CalibrationType::DungeonListArea | 
            CalibrationType::CollectionItemsArea => {
                if self.area_selection_start.is_none() {
                    // First click - store start
                    self.area_selection_start = Some((x, y));
                    *self.status.lock().unwrap() = format!("Now click BOTTOM-RIGHT corner");
                } else {
                    // Second click - calculate area
                    let (x1, y1) = self.area_selection_start.unwrap();
                    let left = x1.min(x);
                    let top = y1.min(y);
                    let width = (x1.max(x) - left).abs();
                    let height = (y1.max(y) - top).abs();
                    
                    let area = (left, top, width, height);
                    
                    match cal_type {
                        CalibrationType::CollectionTabsArea => self.settings.collection_tabs_area = Some(area),
                        CalibrationType::DungeonListArea => self.settings.dungeon_list_area = Some(area),
                        CalibrationType::CollectionItemsArea => self.settings.collection_items_area = Some(area),
                        _ => {}
                    }
                    
                    self.calibrating = None;
                    self.area_selection_start = None;
                    *self.status.lock().unwrap() = format!("Area calibrated: ({}, {}, {}, {})", left, top, width, height);
                }
            },
            // Button calibration (1 click)
            _ => {
                let pos = (x, y);
                
                match cal_type {
                    CalibrationType::AutoRefillButton => self.settings.auto_refill_pos = Some(pos),
                    CalibrationType::RegisterButton => self.settings.register_pos = Some(pos),
                    CalibrationType::YesButton => self.settings.yes_pos = Some(pos),
                    CalibrationType::Page2Button => self.settings.page_2_pos = Some(pos),
                    CalibrationType::Page3Button => self.settings.page_3_pos = Some(pos),
                    CalibrationType::Page4Button => self.settings.page_4_pos = Some(pos),
                    CalibrationType::ArrowRightButton => self.settings.arrow_right_pos = Some(pos),
                    _ => {}
                }
                
                self.calibrating = None;
                *self.status.lock().unwrap() = format!("Button calibrated: ({}, {})", x, y);
            }
        }
    }

    fn is_fully_calibrated(&self) -> bool {
        self.settings.collection_tabs_area.is_some() &&
        self.settings.dungeon_list_area.is_some() &&
        self.settings.collection_items_area.is_some() &&
        self.settings.auto_refill_pos.is_some() &&
        self.settings.register_pos.is_some() &&
        self.settings.yes_pos.is_some()
        // Page buttons are optional
    }

    fn start_automation(&mut self) {
        let running = Arc::clone(&self.running);
        let status = Arc::clone(&self.status);
        *running.lock().unwrap() = true;
        *status.lock().unwrap() = "Starting automation...".to_string();

        // Clone settings for thread
        let settings = self.settings.clone();
        let red_dot_path = self.red_dot_path.clone();
        let game_hwnd = self.game_hwnd.unwrap();

        thread::spawn(move || {
            // Initialize RustAutoGui
            let mut gui = match RustAutoGui::new(false) {
                Ok(g) => g,
                Err(e) => {
                    *status.lock().unwrap() = format!("❌ Failed to initialize RustAutoGui: {}", e);
                    *running.lock().unwrap() = false;
                    return;
                }
            };
            
            // Load red dot template
            if let Err(e) = gui.prepare_template_from_file(
                &red_dot_path,
                None,
                MatchMode::Segmented
            ) {
                *status.lock().unwrap() = format!("❌ Failed to load red-dot template: {}", e);
                *running.lock().unwrap() = false;
                return;
            }

            *status.lock().unwrap() = "🔍 Scanning collection tabs...".to_string();

            // Main automation loop - scan for tab red dots
            while *running.lock().unwrap() {
                let tabs_area = settings.collection_tabs_area.unwrap();
                
                match find_red_dots_in_area(&mut gui, game_hwnd, tabs_area, settings.red_dot_tolerance) {
                    Some(dots) if !dots.is_empty() => {
                        let tab_pos = dots[0];
                        *status.lock().unwrap() = format!("Found tab red dot at ({}, {}), clicking...", tab_pos.0, tab_pos.1);
                        
                        // Click the tab
                        click_at_screen(&mut gui, game_hwnd, tab_pos.0, tab_pos.1);
                        delay_ms(settings.delay_ms);
                        
                        // Process dungeons in this tab
                        process_dungeon_list(&mut gui, &settings, game_hwnd, &running, &status, tab_pos);
                    },
                    _ => {
                        *status.lock().unwrap() = "✓ All collections complete!".to_string();
                        break;
                    }
                }
                
                delay_ms(settings.delay_ms);
            }

            *running.lock().unwrap() = false;
            *status.lock().unwrap() = "Collection automation stopped".to_string();
        });
    }
}

// Helper functions for automation
fn delay_ms(ms: u64) {
    if ms > 0 {
        thread::sleep(Duration::from_millis(ms));
    }
}

fn find_red_dots_in_area(
    gui: &mut RustAutoGui,
    game_hwnd: HWND,
    area: (i32, i32, i32, i32),
    precision: f32
) -> Option<Vec<(u32, u32)>> {
    use crate::core::window::get_window_rect;
    
    // Get window position
    let window_rect = get_window_rect(game_hwnd)?;
    
    // Convert window-relative area to screen coordinates
    let (left, top, width, height) = area;
    let screen_left = window_rect.0 + left;
    let screen_top = window_rect.1 + top;
    let screen_right = screen_left + width;
    let screen_bottom = screen_top + height;
    

    
    // Store template with region
    match gui.find_image_on_screen(precision) {
        Ok(Some(matches)) => {
            // Filter matches to be within our area
            let filtered: Vec<(u32, u32)> = matches.iter()
                .filter_map(|(x, y, _score)| {
                    if *x >= screen_left as u32 && *x <= screen_right as u32 &&
                       *y >= screen_top as u32 && *y <= screen_bottom as u32 {
                        Some((*x, *y))
                    } else {
                        None
                    }
                })
                .collect();
            
            if filtered.is_empty() {
                None
            } else {
                Some(filtered)
            }
        },
        Ok(None) => None,
        Err(e) => {
            println!("Error finding red dots: {}", e);
            None
        }
    }
}

fn click_at_screen(gui: &mut RustAutoGui, game_hwnd: HWND, x: u32, y: u32) {
    // Don't need gui - use direct click
    let _ = gui;
    
    // x, y are screen coordinates from red dot detection
    // Convert to window-relative coordinates
    use crate::core::window::get_window_rect;
    
    if let Some(window_rect) = get_window_rect(game_hwnd) {
        let window_x = x as i32 - window_rect.0;
        let window_y = y as i32 - window_rect.1;
        
        // Use direct click at window-relative position
        click_at_position(game_hwnd, window_x, window_y);
    }
}

fn click_button(gui: &mut RustAutoGui, game_hwnd: HWND, button_pos: Option<(i32, i32)>) -> bool {
    // Don't need gui for direct clicking
    let _ = gui;
    
    if let Some((x, y)) = button_pos {
        // Use direct click at position - much faster
        click_at_position(game_hwnd, x, y);
        return true;
    }
    false
}

fn process_dungeon_list(
    gui: &mut RustAutoGui,
    settings: &CollectionSettings,
    game_hwnd: HWND,
    running: &Arc<Mutex<bool>>,
    status: &Arc<Mutex<String>>,
    original_tab_pos: (u32, u32)
) {
    let mut current_page = 1;
    let mut pages_checked_this_cycle = 0;
    let mut cycles_without_hits = 0;
    let max_empty_cycles = 2;
    
    while *running.lock().unwrap() && tab_still_has_red_dot(gui, settings, game_hwnd, original_tab_pos) {
        *status.lock().unwrap() = format!("Processing dungeon list on page {}", current_page);
        
        let found_dungeons = process_dungeons_on_current_page(
            gui,
            settings,
            game_hwnd,
            running,
            status
        );
        
        if found_dungeons {
            // Reset pagination
            current_page = 1;
            pages_checked_this_cycle = 0;
            cycles_without_hits = 0;
        } else {
            // No work on this page, advance to next
            pages_checked_this_cycle += 1;
            
            if current_page < 4 {
                current_page += 1;
                *status.lock().unwrap() = format!("No work on page {}, advancing to page {}", current_page - 1, current_page);
                let button = match current_page {
                    2 => settings.page_2_pos,
                    3 => settings.page_3_pos,
                    4 => settings.page_4_pos,
                    _ => None,
                };
                click_button(gui, game_hwnd, button);
                delay_ms(settings.delay_ms);
            } else {
                // On page 4, try arrow right
                *status.lock().unwrap() = "Reaching end of page set, clicking arrow right...".to_string();
                if click_button(gui, game_hwnd, settings.arrow_right_pos) {
                    delay_ms(settings.delay_ms);
                    current_page = 1;
                } else {
                    break;
                }
            }
            
            // After a full cycle of 4 pages with no hits, check if we should give up
            if pages_checked_this_cycle >= 4 {
                cycles_without_hits += 1;
                pages_checked_this_cycle = 0;
                
                if cycles_without_hits >= max_empty_cycles || !tab_still_has_red_dot(gui, settings, game_hwnd, original_tab_pos) {
                    *status.lock().unwrap() = "Tab processing complete".to_string();
                    break;
                }
            }
        }
    }
}

fn process_dungeons_on_current_page(
    gui: &mut RustAutoGui,
    settings: &CollectionSettings,
    game_hwnd: HWND,
    running: &Arc<Mutex<bool>>,
    status: &Arc<Mutex<String>>
) -> bool {
    let mut items_processed = false;
    let dungeon_area = settings.dungeon_list_area.unwrap();
    
    while *running.lock().unwrap() {
        // Find next dungeon with red dot
        match find_red_dots_in_area(gui, game_hwnd, dungeon_area, settings.red_dot_tolerance) {
            Some(dots) if !dots.is_empty() => {
                let dungeon_pos = dots[0];
                *status.lock().unwrap() = format!("Found dungeon at ({}, {}), clicking...", dungeon_pos.0, dungeon_pos.1);
                
                // Click the dungeon
                click_at_screen(gui, game_hwnd, dungeon_pos.0, dungeon_pos.1);
                delay_ms(settings.delay_ms);
                
                // Scroll to top of items area
                scroll_in_area(gui, game_hwnd, settings.collection_items_area.unwrap(), -20);
                delay_ms(settings.delay_ms);
                
                // Process items with double-check pattern like Python version
                let max_scroll_passes = 50;
                for scroll_pass in 0..max_scroll_passes {
                    if !*running.lock().unwrap() {
                        break;
                    }
                    
                    // Process all items at current scroll position
                    if process_all_items_at_position(gui, settings, game_hwnd, running, status) {
                        items_processed = true;
                    }
                    
                    // Critical: Double-check that no red dots remain
                    let items_area = settings.collection_items_area.unwrap();
                    match find_red_dots_in_area(gui, game_hwnd, items_area, settings.red_dot_tolerance) {
                        Some(remaining) if !remaining.is_empty() => {
                            // Still have dots, process again (belt and suspenders)
                            process_all_items_at_position(gui, settings, game_hwnd, running, status);
                            delay_ms(settings.delay_ms);
                        },
                        _ => {}
                    }
                    
                    // Check if this dungeon still has a red dot nearby
                    match find_red_dots_in_area(gui, game_hwnd, dungeon_area, settings.red_dot_tolerance) {
                        Some(remaining_dungeons) => {
                            let dungeon_still_red = remaining_dungeons.iter().any(|(x, y)| {
                                let dist = ((*x as f32 - dungeon_pos.0 as f32).powi(2) +
                                          (*y as f32 - dungeon_pos.1 as f32).powi(2)).sqrt();
                                dist <= 20.0
                            });
                            
                            if !dungeon_still_red {
                                *status.lock().unwrap() = "Dungeon complete, moving to next...".to_string();
                                break; // Move to next dungeon
                            }
                        },
                        None => break,
                    }
                    
                    // Scroll down for next batch of items
                    scroll_in_area(gui, game_hwnd, items_area, 5);
                    delay_ms(settings.delay_ms);
                }
            },
            _ => {
                // No dungeon with red dot found
                break;
            }
        }
    }
    
    items_processed
}

fn process_all_items_at_position(
    gui: &mut RustAutoGui,
    settings: &CollectionSettings,
    game_hwnd: HWND,
    running: &Arc<Mutex<bool>>,
    status: &Arc<Mutex<String>>
) -> bool {
    let mut items_processed = false;
    let items_area = settings.collection_items_area.unwrap();
    let mut last_pos: Option<(u32, u32)> = None;
    let mut stuck_count = 0;
    
    while *running.lock().unwrap() {
        match find_red_dots_in_area(gui, game_hwnd, items_area, settings.red_dot_tolerance) {
            Some(dots) if !dots.is_empty() => {
                let item_pos = dots[0];
                
                // Check if stuck on same item
                if let Some(last) = last_pos {
                    let dist = ((item_pos.0 as f32 - last.0 as f32).powi(2) +
                               (item_pos.1 as f32 - last.1 as f32).powi(2)).sqrt();
                    if dist <= 5.0 {
                        stuck_count += 1;
                        if stuck_count >= 3 {
                            *status.lock().unwrap() = "Stuck on item, skipping...".to_string();
                            break;
                        }
                    } else {
                        stuck_count = 0;
                    }
                }
                
                last_pos = Some(item_pos);
                
                // Click the item
                click_at_screen(gui, game_hwnd, item_pos.0, item_pos.1);
                delay_ms(settings.delay_ms);
                
                // Execute button sequence
                execute_button_sequence(gui, settings, game_hwnd, status);
                items_processed = true;
                delay_ms(settings.delay_ms);
            },
            _ => break,
        }
    }
    
    items_processed
}

fn execute_button_sequence(
    gui: &mut RustAutoGui,
    settings: &CollectionSettings,
    game_hwnd: HWND,
    _status: &Arc<Mutex<String>>
) {
    // Auto Refill
    click_button(gui, game_hwnd, settings.auto_refill_pos);
    delay_ms(settings.delay_ms);
    
    // Register
    click_button(gui, game_hwnd, settings.register_pos);
    delay_ms(settings.delay_ms);
    
    // Yes
    click_button(gui, game_hwnd, settings.yes_pos);
    delay_ms(settings.delay_ms);
}

fn tab_still_has_red_dot(
    gui: &mut RustAutoGui,
    settings: &CollectionSettings,
    game_hwnd: HWND,
    original_pos: (u32, u32)
) -> bool {
    let tabs_area = settings.collection_tabs_area.unwrap();
    
    match find_red_dots_in_area(gui, game_hwnd, tabs_area, 0.85) {
        Some(dots) => {
            dots.iter().any(|(x, y)| {
                let dist = ((*x as f32 - original_pos.0 as f32).powi(2) + 
                           (*y as f32 - original_pos.1 as f32).powi(2)).sqrt();
                dist <= 20.0
            })
        },
        None => false,
    }
}

fn scroll_in_area(
    gui: &mut RustAutoGui,
    game_hwnd: HWND,
    area: (i32, i32, i32, i32),
    amount: i32
) {
    use crate::core::window::get_window_rect;
    
    if let Some(window_rect) = get_window_rect(game_hwnd) {
        let (left, top, width, height) = area;
        let center_x = window_rect.0 + left + width / 2;
        let center_y = window_rect.1 + top + height / 2;
        
        // Move mouse to center of area
        if let Err(e) = gui.move_mouse_to_pos(center_x as u32, center_y as u32, 0.1) {
            println!("Failed to move for scroll: {}", e);
            return;
        }
        delay_ms(50);
        
        // Scroll
        if amount < 0 {
            for _ in 0..amount.abs() {
                let _ = gui.scroll_up(120);
            }
        } else {
            for _ in 0..amount {
                let _ = gui.scroll_down(120);
            }
        }
    }
}
