use windows::{
    Win32::Foundation::{HWND, POINT},
    Win32::UI::WindowsAndMessaging::{
        FindWindowA, GetWindowRect, IsWindow, WindowFromPoint, GetCursorPos, GetAncestor, GA_PARENT
    },
    Win32::Graphics::Gdi::ScreenToClient,
};

/// Find game window by class name "D3D Window"
pub fn find_game_window() -> Option<HWND> {
    unsafe {
        // Try to find window by class name "D3D Window"
        let hwnd = FindWindowA(
            windows::core::PCSTR("D3D Window\0".as_ptr()),
            windows::core::PCSTR::null(),
        );

        if hwnd.0 != 0 && IsWindow(hwnd).as_bool() {
            Some(hwnd)
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
            Some((rect.left, rect.top, rect.right, rect.bottom))
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
