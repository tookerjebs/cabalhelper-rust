// Calibration module - shared calibration logic for all tools
use windows::Win32::Foundation::HWND;
use crate::core::input::{is_left_mouse_down, was_left_mouse_pressed};
use crate::core::window::{get_window_under_cursor, is_game_window_or_child, get_cursor_pos, screen_to_window_coords, get_client_origin_in_screen_coords};
use crate::core::screen_draw::draw_focus_rect_screen;

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
    drag_start: Option<(i32, i32)>,
    last_pos: Option<(i32, i32)>,
    dragging: bool,
    last_drawn_rect: Option<(i32, i32, i32, i32)>,
}

impl Default for CalibrationManager {
    fn default() -> Self {
        Self {
            active: false,
            is_area: false,
            drag_start: None,
            last_pos: None,
            dragging: false,
            last_drawn_rect: None,
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
        self.drag_start = None;
        self.last_pos = None;
        self.dragging = false;
        self.clear_overlay();
    }

    /// Start calibrating an area (click and drag)
    pub fn start_area(&mut self) {
        self.active = true;
        self.is_area = true;
        self.drag_start = None;
        self.last_pos = None;
        self.dragging = false;
        self.clear_overlay();
    }

    /// Cancel current calibration
    pub fn cancel(&mut self) {
        self.active = false;
        self.drag_start = None;
        self.last_pos = None;
        self.dragging = false;
        self.clear_overlay();
    }

    /// Check if calibration is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Check if currently dragging an area
    pub fn is_dragging(&self) -> bool {
        self.is_area && self.dragging
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

        let mut cursor_in_game = || -> Option<(i32, i32)> {
            if let Some(cursor_hwnd) = get_window_under_cursor() {
                if is_game_window_or_child(cursor_hwnd, game_hwnd) {
                    if let Some((screen_x, screen_y)) = get_cursor_pos() {
                        return screen_to_window_coords(game_hwnd, screen_x, screen_y);
                    }
                }
            }
            None
        };

        if self.is_area {
            if !self.dragging {
                if !was_left_mouse_pressed() {
                    return None;
                }

                if let Some((x, y)) = cursor_in_game() {
                    self.drag_start = Some((x, y));
                    self.last_pos = Some((x, y));
                    self.dragging = true;
                }
                return None;
            }

            if let Some((x, y)) = cursor_in_game() {
                self.last_pos = Some((x, y));
            }

            if let (Some((x1, y1)), Some((x2, y2))) = (self.drag_start, self.last_pos) {
                self.update_overlay_rect(game_hwnd, x1, y1, x2, y2);
            }

            if !is_left_mouse_down() {
                if let (Some((x1, y1)), Some((x2, y2))) = (self.drag_start, self.last_pos) {
                    let left = x1.min(x2);
                    let top = y1.min(y2);
                    let width = (x1.max(x2) - left).abs();
                    let height = (y1.max(y2) - top).abs();

                    self.active = false;
                    self.drag_start = None;
                    self.last_pos = None;
                    self.dragging = false;
                    self.clear_overlay();
                    return Some(CalibrationResult::Area(left, top, width, height));
                }

                self.active = false;
                self.drag_start = None;
                self.last_pos = None;
                self.dragging = false;
                self.clear_overlay();
            }

            return None;
        }

        if !was_left_mouse_pressed() {
            return None;
        }

        if let Some((x, y)) = cursor_in_game() {
            self.active = false;
            return Some(CalibrationResult::Point(x, y));
        }

        None
    }

    fn update_overlay_rect(&mut self, game_hwnd: HWND, x1: i32, y1: i32, x2: i32, y2: i32) {
        let (left, top) = match get_client_origin_in_screen_coords(game_hwnd) {
            Some(origin) => origin,
            None => return,
        };

        let screen_left = left + x1.min(x2);
        let screen_top = top + y1.min(y2);
        let screen_right = left + x1.max(x2);
        let screen_bottom = top + y1.max(y2);

        let new_rect = (screen_left, screen_top, screen_right, screen_bottom);

        if let Some(prev) = self.last_drawn_rect {
            if prev == new_rect {
                return;
            }
            draw_focus_rect_screen(prev);
        }

        draw_focus_rect_screen(new_rect);
        self.last_drawn_rect = Some(new_rect);
    }

    fn clear_overlay(&mut self) {
        if let Some(prev) = self.last_drawn_rect.take() {
            draw_focus_rect_screen(prev);
        }
    }
}
