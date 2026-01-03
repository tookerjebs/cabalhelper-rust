// Calibration module - shared calibration logic for all tools
use windows::Win32::Foundation::HWND;
use crate::core::input::is_left_mouse_down;
use crate::core::window::{get_window_under_cursor, is_game_window_or_child, get_cursor_pos, screen_to_window_coords};

/// Result of a calibration operation
#[derive(Debug, Clone)]
pub enum CalibrationResult {
    Point(i32, i32),
    Area(i32, i32, i32, i32), // left, top, width, height
}

/// Manages calibration state and logic
pub struct CalibrationManager {
    active: bool,
    is_area: bool, // true for area calibration, false for point
    area_start: Option<(i32, i32)>,
    last_mouse_state: bool,
}

impl Default for CalibrationManager {
    fn default() -> Self {
        Self {
            active: false,
            is_area: false,
            area_start: None,
            last_mouse_state: false,
        }
    }
}

impl CalibrationManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Start calibrating a point (single click)
    pub fn start_point(&mut self) {
        self.active = true;
        self.is_area = false;
        self.area_start = None;
        self.last_mouse_state = false;
    }
    
    /// Start calibrating an area (two clicks: top-left, bottom-right)
    pub fn start_area(&mut self) {
        self.active = true;
        self.is_area = true;
        self.area_start = None;
        self.last_mouse_state = false;
    }
    
    /// Cancel current calibration
    pub fn cancel(&mut self) {
        self.active = false;
        self.area_start = None;
    }
    
    /// Check if calibration is active
    pub fn is_active(&self) -> bool {
        self.active
    }
    
    /// Check if waiting for second click (area calibration only)
    pub fn is_waiting_for_second_click(&self) -> bool {
        self.is_area && self.area_start.is_some()
    }
    
    /// Main update loop for calibration
    /// Handles mouse clicks and returns result if calibration finished this frame
    pub fn update(&mut self, game_hwnd: HWND) -> Option<CalibrationResult> {
         self.handle_clicks(game_hwnd)
    }

    /// Handle mouse clicks and return calibration result if complete
    /// Returns Some(result) when calibration is complete, None otherwise
    fn handle_clicks(&mut self, game_hwnd: HWND) -> Option<CalibrationResult> {
        if !self.active {
            return None;
        }
        
        // Detect mouse click
        let mouse_down = is_left_mouse_down();
        let just_pressed = mouse_down && !self.last_mouse_state;
        self.last_mouse_state = mouse_down;
        
        if !just_pressed {
            return None;
        }
        
        // Check if click is on game window
        if let Some(cursor_hwnd) = get_window_under_cursor() {
            if is_game_window_or_child(cursor_hwnd, game_hwnd) {
                if let Some((screen_x, screen_y)) = get_cursor_pos() {
                    if let Some((x, y)) = screen_to_window_coords(game_hwnd, screen_x, screen_y) {
                        return self.process_click(x, y);
                    }
                }
            }
        }
        
        None
    }
    
    fn process_click(&mut self, x: i32, y: i32) -> Option<CalibrationResult> {
        if self.is_area {
            // Area calibration (2 clicks)
            if self.area_start.is_none() {
                // First click - store start
                self.area_start = Some((x, y));
                None
            } else {
                // Second click - calculate area
                let (x1, y1) = self.area_start.unwrap();
                let left = x1.min(x);
                let top = y1.min(y);
                let width = (x1.max(x) - left).abs();
                let height = (y1.max(y) - top).abs();
                
                self.active = false;
                self.area_start = None;
                Some(CalibrationResult::Area(left, top, width, height))
            }
        } else {
            // Point calibration (1 click)
            self.active = false;
            Some(CalibrationResult::Point(x, y))
        }
    }
}
