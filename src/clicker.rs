use windows::{
    Win32::Foundation::{HWND, LPARAM, WPARAM},
    Win32::UI::WindowsAndMessaging::{
        SendMessageA, WM_LBUTTONDOWN, WM_LBUTTONUP,
    },
};

// MK_LBUTTON constant value
const MK_LBUTTON: u32 = 0x0001;

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

