use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{DrawFocusRect, GetDC, ReleaseDC};

/// Draw (or erase) a focus rectangle using XOR on the desktop.
/// Calling this twice with the same rect erases it.
pub fn draw_focus_rect_screen(rect: (i32, i32, i32, i32)) {
    unsafe {
        let hdc = GetDC(HWND(0));
        if hdc.is_invalid() {
            return;
        }

        let (left, top, right, bottom) = rect;
        let rect = RECT {
            left,
            top,
            right,
            bottom,
        };

        let _ = DrawFocusRect(hdc, &rect);
        let _ = ReleaseDC(HWND(0), hdc);
    }
}
