// Calibration module - shared calibration logic for all tools
use crate::core::coords::{normalize_point, normalize_rect};
use crate::core::input::is_left_mouse_down;
use crate::core::window::{
    get_client_rect_in_screen_coords, get_cursor_pos, get_window_under_cursor,
    is_game_window_or_child, screen_to_window_coords,
};
use windows::Win32::Foundation::HWND;

/// Result of a calibration operation
#[derive(Debug, Clone)]
pub enum CalibrationResult {
    Point(f32, f32),
    Area(f32, f32, f32, f32), // left, top, width, height (normalized)
}

/// Manages calibration state and logic
pub struct CalibrationManager {
    active: bool,
    is_area: bool, // true for area calibration, false for point
    area_start: Option<(i32, i32)>,
    last_left_down: bool,
}

impl Default for CalibrationManager {
    fn default() -> Self {
        Self {
            active: false,
            is_area: false,
            area_start: None,
            last_left_down: false,
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
        self.last_left_down = false;
    }

    /// Start calibrating an area (click top-left, then bottom-right)
    pub fn start_area(&mut self) {
        self.active = true;
        self.is_area = true;
        self.area_start = None;
        self.last_left_down = false;
    }

    /// Cancel current calibration
    pub fn cancel(&mut self) {
        self.active = false;
        self.area_start = None;
        self.last_left_down = false;
    }

    /// Check if calibration is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Check if an area calibration is waiting for the second click
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

        let cursor_in_game = || -> Option<(i32, i32)> {
            let (screen_x, screen_y) = get_cursor_pos()?;

            if let Some((left, top, width, height)) = get_client_rect_in_screen_coords(game_hwnd) {
                let right = left + width;
                let bottom = top + height;
                if screen_x >= left && screen_x < right && screen_y >= top && screen_y < bottom {
                    return screen_to_window_coords(game_hwnd, screen_x, screen_y);
                }
            }

            if let Some(cursor_hwnd) = get_window_under_cursor() {
                if is_game_window_or_child(cursor_hwnd, game_hwnd) {
                    return screen_to_window_coords(game_hwnd, screen_x, screen_y);
                }
            }

            None
        };

        let is_down = is_left_mouse_down();
        if !is_down {
            self.last_left_down = false;
            return None;
        }
        if self.last_left_down {
            return None;
        }
        self.last_left_down = true;

        if self.is_area {
            if let Some((x, y)) = cursor_in_game() {
                if let Some((x1, y1)) = self.area_start {
                    let left = x1.min(x);
                    let top = y1.min(y);
                    let width = (x1.max(x) - left).abs();
                    let height = (y1.max(y) - top).abs();

                    self.active = false;
                    self.area_start = None;
                    if let Some((nl, nt, nw, nh)) =
                        normalize_rect(game_hwnd, left, top, width, height)
                    {
                        return Some(CalibrationResult::Area(nl, nt, nw, nh));
                    }
                    return None;
                }

                self.area_start = Some((x, y));
            }

            return None;
        }

        if let Some((x, y)) = cursor_in_game() {
            self.active = false;
            if let Some((nx, ny)) = normalize_point(game_hwnd, x, y) {
                return Some(CalibrationResult::Point(nx, ny));
            }
        }

        None
    }
}
