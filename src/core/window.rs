use windows::{
    Win32::Foundation::{HWND, POINT},
    Win32::UI::WindowsAndMessaging::{
        FindWindowA, GetWindowRect, IsWindow, WindowFromPoint, GetCursorPos, GetAncestor, GA_PARENT
    },
    Win32::Graphics::Gdi::{ScreenToClient, GetDC, GetPixel, ReleaseDC},
};

/// Find game window
/// Priority 1: Window with title "PlayCabal EP36"
/// Priority 2: Window with class name "D3D Window" (legacy/fallback)
pub fn find_game_window() -> Option<HWND> {
    unsafe {
        // 1. Try to find by specific Window Title
        let hwnd_by_title = FindWindowA(
            windows::core::PCSTR::null(),
            windows::core::PCSTR("PlayCabal EP36\0".as_ptr()),
        );

        if hwnd_by_title.0 != 0 && IsWindow(hwnd_by_title).as_bool() {
            return Some(hwnd_by_title);
        }

        // 2. Fallback: Try to find by class name "D3D Window"
        let hwnd_by_class = FindWindowA(
            windows::core::PCSTR("D3D Window\0".as_ptr()),
            windows::core::PCSTR::null(),
        );

        if hwnd_by_class.0 != 0 && IsWindow(hwnd_by_class).as_bool() {
            Some(hwnd_by_class)
        } else {
            None
        }
    }
}

/// Check if window handle is valid
pub fn is_window_valid(hwnd: HWND) -> bool {
    unsafe { IsWindow(hwnd).as_bool() }
}

/// Get window rectangle
pub fn get_window_rect(hwnd: HWND) -> Option<(i32, i32, i32, i32)> {
    unsafe {
        let mut rect = windows::Win32::Foundation::RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            Some((rect.left, rect.top, width, height))
        } else {
            None
        }
    }
}

/// Convert screen coordinates to window-relative coordinates
pub fn screen_to_window_coords(hwnd: HWND, screen_x: i32, screen_y: i32) -> Option<(i32, i32)> {
    unsafe {
        let mut point = POINT { x: screen_x, y: screen_y };
        if ScreenToClient(hwnd, &mut point).as_bool() {
            Some((point.x, point.y))
        } else {
            None
        }
    }
}

/// Get current cursor position in screen coordinates
pub fn get_cursor_pos() -> Option<(i32, i32)> {
    unsafe {
        let mut point = POINT::default();
        if GetCursorPos(&mut point).is_ok() {
            Some((point.x, point.y))
        } else {
            None
        }
    }
}

/// Get window under cursor
pub fn get_window_under_cursor() -> Option<HWND> {
    unsafe {
        if let Some((x, y)) = get_cursor_pos() {
            let point = POINT { x, y };
            let hwnd = WindowFromPoint(point);
            if hwnd.0 != 0 {
                Some(hwnd)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Check if a window is the game window or a child of it
pub fn is_game_window_or_child(check_hwnd: HWND, game_hwnd: HWND) -> bool {
    let mut current_hwnd = check_hwnd;
    for _ in 0..10 {
        if current_hwnd.0 == game_hwnd.0 {
            return true;
        }
        unsafe {
            current_hwnd = GetAncestor(current_hwnd, GA_PARENT);
        }
        if current_hwnd.0 == 0 {
            break;
        }
    }
    false
}

/// Get the RGB color of a pixel at screen coordinates
/// Returns (R, G, B) as u8 values
pub fn get_pixel_color(screen_x: i32, screen_y: i32) -> Option<(u8, u8, u8)> {
    unsafe {
        // Get device context for the entire screen
        let hdc = GetDC(HWND(0));
        if hdc.is_invalid() {
            return None;
        }
        
        // Get pixel color as COLORREF (0x00BBGGRR format)
        let color = GetPixel(hdc, screen_x, screen_y);
        
        // Release the device context
        let _ = ReleaseDC(HWND(0), hdc);
        
        // Convert COLORREF to u32 for bitwise operations
        // COLORREF format: 0x00BBGGRR
        let color_val = color.0 as u32;
        let r = (color_val & 0xFF) as u8;
        let g = ((color_val >> 8) & 0xFF) as u8;
        let b = ((color_val >> 16) & 0xFF) as u8;
        
        Some((r, g, b))
    }
}


