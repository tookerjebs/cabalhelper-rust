use crate::core::coords::{denormalize_point, denormalize_rect};
use crate::core::window::client_to_screen_coords;
use crate::settings::{NormPoint, NormRect};
use rustautogui::RustAutoGui;
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::HWND;

/// Delay for a specified number of milliseconds
pub fn delay_ms(ms: u64) {
    if ms > 0 {
        thread::sleep(Duration::from_millis(ms));
    }
}

/// Click at screen coordinates (with retry logic from Python version)
pub fn click_at_screen(gui: &mut RustAutoGui, x: u32, y: u32) {
    // Python does 2 click attempts with 50ms delay
    for attempt in 0..2 {
        // Move mouse to position (screen coordinates)
        if let Err(_) = gui.move_mouse_to_pos(x, y, 0.0) {
            if attempt == 0 {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            return;
        }

        // Short sleep to stabilize cursor
        thread::sleep(Duration::from_millis(20));

        // Perform physical left click
        if let Err(_) = gui.left_click() {
            if attempt == 0 {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
        } else {
            // Success on first or second attempt
            return;
        }
    }
}

/// Right click at screen coordinates (with retry logic from Python version)
pub fn right_click_at_screen(gui: &mut RustAutoGui, x: u32, y: u32) {
    // Python does 2 click attempts with 50ms delay
    for attempt in 0..2 {
        // Move mouse to position (screen coordinates)
        if let Err(_) = gui.move_mouse_to_pos(x, y, 0.0) {
            if attempt == 0 {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            return;
        }

        // Short sleep to stabilize cursor
        thread::sleep(Duration::from_millis(20));

        // Perform physical right click
        if let Err(_) = gui.right_click() {
            if attempt == 0 {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
        } else {
            // Success on first or second attempt
            return;
        }
    }
}

/// Middle click at screen coordinates (with retry logic from Python version)
pub fn middle_click_at_screen(gui: &mut RustAutoGui, x: u32, y: u32) {
    // Python does 2 click attempts with 50ms delay
    for attempt in 0..2 {
        // Move mouse to position (screen coordinates)
        if let Err(_) = gui.move_mouse_to_pos(x, y, 0.0) {
            if attempt == 0 {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            return;
        }

        // Short sleep to stabilize cursor
        thread::sleep(Duration::from_millis(20));

        // Perform physical middle click
        if let Err(_) = gui.middle_click() {
            if attempt == 0 {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
        } else {
            // Success on first or second attempt
            return;
        }
    }
}

/// Click at normalized window-relative coordinates (converts to screen coords first)
pub fn click_at_window_pos(gui: &mut RustAutoGui, game_hwnd: HWND, pos: NormPoint) -> bool {
    let (rel_x, rel_y) = match denormalize_point(game_hwnd, pos.0, pos.1) {
        Some(coords) => coords,
        None => return false,
    };
    let (screen_x, screen_y) = match client_to_screen_coords(game_hwnd, rel_x, rel_y) {
        Some(coords) => coords,
        None => return false,
    };
    click_at_screen(gui, screen_x as u32, screen_y as u32);
    true
}

/// Scroll in a specific area (normalized window-relative coordinates)
pub fn scroll_in_area(gui: &mut RustAutoGui, game_hwnd: HWND, area: NormRect, amount: i32) {
    let (left, top, width, height) =
        match denormalize_rect(game_hwnd, area.0, area.1, area.2, area.3) {
            Some(rect) => rect,
            None => return,
        };
    let center_x = left + width / 2;
    let center_y = top + height / 2;
    let (screen_x, screen_y) = match client_to_screen_coords(game_hwnd, center_x, center_y) {
        Some(coords) => coords,
        None => return,
    };

    // Move mouse to center of area (instant, no animation)
    if let Err(_) = gui.move_mouse_to_pos(screen_x as u32, screen_y as u32, 0.0) {
        return;
    }
    delay_ms(20);

    // Scroll (reduced from 20 to 5 ticks since game only processes ~1 unit anyway)
    let scroll_ticks = if amount.abs() > 5 { 5 } else { amount.abs() };
    if amount < 0 {
        for _ in 0..scroll_ticks {
            let _ = gui.scroll_up(120);
        }
    } else {
        for _ in 0..scroll_ticks {
            let _ = gui.scroll_down(120);
        }
    }
}
