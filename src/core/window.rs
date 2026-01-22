use windows::{
    Win32::Foundation::{HWND, POINT},
    Win32::Graphics::Gdi::{ClientToScreen, GetDC, GetPixel, ReleaseDC, ScreenToClient},
    Win32::UI::WindowsAndMessaging::{
        FindWindowA, GetAncestor, GetClientRect, GetCursorPos, GetWindowRect, GetWindowTextA,
        IsWindow, WindowFromPoint, GA_PARENT,
    },
};

/// Find game window
/// Searches for "D3D Window" class (universal for all Cabal versions)
pub fn find_game_window() -> Option<(HWND, String)> {
    unsafe {
        let hwnd = FindWindowA(
            windows::core::PCSTR("D3D Window\0".as_ptr()),
            windows::core::PCSTR::null(),
        );

        if hwnd.0 != 0 && IsWindow(hwnd).as_bool() {
            // Get actual window title
            let mut buffer = [0u8; 256];
            let len = GetWindowTextA(hwnd, &mut buffer);
            let title = if len > 0 {
                String::from_utf8_lossy(&buffer[..len as usize]).to_string()
            } else {
                "D3D Window".to_string()
            };
            Some((hwnd, title))
        } else {
            None
        }
    }
}

/// Check if window handle is valid
pub fn is_window_valid(hwnd: HWND) -> bool {
    unsafe { IsWindow(hwnd).as_bool() }
}

/// Get client area rectangle in screen coordinates (excludes borders/title bar)
pub fn get_client_rect_in_screen_coords(hwnd: HWND) -> Option<(i32, i32, i32, i32)> {
    unsafe {
        // 1. Get the size of the inner content area
        let mut client_rect = windows::Win32::Foundation::RECT::default();
        if GetClientRect(hwnd, &mut client_rect).is_err() {
            return None;
        }

        // 2. Convert (0,0) of client area to Screen Coordinates
        let mut top_left = POINT { x: 0, y: 0 };
        if !ClientToScreen(hwnd, &mut top_left).as_bool() {
            return None;
        }

        // 3. Convert bottom-right
        let mut bottom_right = POINT {
            x: client_rect.right,
            y: client_rect.bottom,
        };
        ClientToScreen(hwnd, &mut bottom_right);

        Some((
            top_left.x,
            top_left.y,
            bottom_right.x - top_left.x, // Width
            bottom_right.y - top_left.y, // Height
        ))
    }
}

/// Get window rectangle in screen coordinates (includes borders/title bar).
pub fn get_window_rect_in_screen_coords(hwnd: HWND) -> Option<(i32, i32, i32, i32)> {
    unsafe {
        let mut rect = windows::Win32::Foundation::RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return None;
        }
        Some((
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
        ))
    }
}

/// Get client area size (width, height)
pub fn get_client_size(hwnd: HWND) -> Option<(i32, i32)> {
    unsafe {
        let mut client_rect = windows::Win32::Foundation::RECT::default();
        if GetClientRect(hwnd, &mut client_rect).is_err() {
            return None;
        }
        Some((client_rect.right, client_rect.bottom))
    }
}

/// Convert screen coordinates to window-relative coordinates
pub fn screen_to_window_coords(hwnd: HWND, screen_x: i32, screen_y: i32) -> Option<(i32, i32)> {
    unsafe {
        let mut point = POINT {
            x: screen_x,
            y: screen_y,
        };
        if ScreenToClient(hwnd, &mut point).as_bool() {
            Some((point.x, point.y))
        } else {
            None
        }
    }
}

/// Convert window-relative coordinates to screen coordinates
pub fn client_to_screen_coords(hwnd: HWND, x: i32, y: i32) -> Option<(i32, i32)> {
    unsafe {
        let mut point = POINT { x, y };
        if ClientToScreen(hwnd, &mut point).as_bool() {
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
