use rustautogui::RustAutoGui;
use windows::Win32::Foundation::HWND;
use std::thread;
use std::time::Duration;

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
        if let Err(e) = gui.move_mouse_to_pos(x, y, 0.0) {
            println!("Failed to move mouse (attempt {}): {}", attempt + 1, e);
            if attempt == 0 {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            return;
        }
        
        // Short sleep to stabilize cursor
        thread::sleep(Duration::from_millis(20));
        
        // Perform physical left click
        if let Err(e) = gui.left_click() {
            println!("Failed to click (attempt {}): {}", attempt + 1, e);
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

/// Click at window-relative coordinates (converts to screen coords first)
pub fn click_at_window_pos(gui: &mut RustAutoGui, game_hwnd: HWND, rel_x: i32, rel_y: i32) -> bool {
    use crate::core::window::get_window_rect;
    
    // Convert window-relative coordinates to screen coordinates
    if let Some((win_x, win_y, _, _)) = get_window_rect(game_hwnd) {
        let screen_x = win_x + rel_x;
        let screen_y = win_y + rel_y;
        
        click_at_screen(gui, screen_x as u32, screen_y as u32);
        true
    } else {
        false
    }
}

/// Scroll in a specific area (window-relative coordinates)
pub fn scroll_in_area(
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
