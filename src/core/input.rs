use windows::{
    Win32::Foundation::{HWND, LPARAM, WPARAM},
    Win32::UI::WindowsAndMessaging::{
        SendMessageA, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP,
    },
    Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState,
};

// MK_LBUTTON constant value
const MK_LBUTTON: u32 = 0x0001;
const MK_RBUTTON: u32 = 0x0002;

/// Click at coordinates using SendMessage (direct click, frees up mouse)
pub fn click_at_position(hwnd: HWND, x: i32, y: i32) -> bool {
    unsafe {
        // Create lParam: low word = x, high word = y
        let lparam_value = ((y as u32) << 16) | (x as u32 & 0xFFFF);
        let lparam = LPARAM(lparam_value as isize);

        // Send mouse down and up messages
        SendMessageA(hwnd, WM_LBUTTONDOWN, WPARAM(MK_LBUTTON as usize), lparam);
        SendMessageA(hwnd, WM_LBUTTONUP, WPARAM(0), lparam);

        true
    }
}

/// Right click at coordinates using SendMessage (direct click, frees up mouse)
pub fn right_click_at_position(hwnd: HWND, x: i32, y: i32) -> bool {
    unsafe {
        // Create lParam: low word = x, high word = y
        let lparam_value = ((y as u32) << 16) | (x as u32 & 0xFFFF);
        let lparam = LPARAM(lparam_value as isize);

        // Send mouse down and up messages
        SendMessageA(hwnd, WM_RBUTTONDOWN, WPARAM(MK_RBUTTON as usize), lparam);
        SendMessageA(hwnd, WM_RBUTTONUP, WPARAM(0), lparam);

        true
    }
}

/// Click at coordinates using PostMessage (async click, frees up mouse)
pub fn click_at_position_post(hwnd: HWND, x: i32, y: i32) -> bool {
    unsafe {
        // Create lParam: low word = x, high word = y
        let lparam_value = ((y as u32) << 16) | (x as u32 & 0xFFFF);
        let lparam = LPARAM(lparam_value as isize);

        // Post mouse down and up messages asynchronously
        use windows::Win32::UI::WindowsAndMessaging::PostMessageA;
        PostMessageA(hwnd, WM_LBUTTONDOWN, WPARAM(MK_LBUTTON as usize), lparam).ok();
        PostMessageA(hwnd, WM_LBUTTONUP, WPARAM(0), lparam).ok();

        true
    }
}

/// Right click at coordinates using PostMessage (async click, frees up mouse)
pub fn right_click_at_position_post(hwnd: HWND, x: i32, y: i32) -> bool {
    unsafe {
        // Create lParam: low word = x, high word = y
        let lparam_value = ((y as u32) << 16) | (x as u32 & 0xFFFF);
        let lparam = LPARAM(lparam_value as isize);

        // Post mouse down and up messages asynchronously
        use windows::Win32::UI::WindowsAndMessaging::PostMessageA;
        PostMessageA(hwnd, WM_RBUTTONDOWN, WPARAM(MK_RBUTTON as usize), lparam).ok();
        PostMessageA(hwnd, WM_RBUTTONUP, WPARAM(0), lparam).ok();

        true
    }
}

/// Check if left mouse button was pressed since last call
pub fn was_left_mouse_pressed() -> bool {
    unsafe {
        let key_state = GetAsyncKeyState(0x01); // VK_LBUTTON
        (key_state as u16) & 0x0001 != 0
    }
}

/// Check if ESC key is currently down (works even when app doesn't have focus)
pub fn is_escape_key_down() -> bool {
    unsafe {
        let key_state = GetAsyncKeyState(0x1B); // VK_ESCAPE
        (key_state as u16) & 0x8000 != 0
    }
}
