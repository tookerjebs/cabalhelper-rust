use crate::core::window::get_client_size;
use windows::Win32::Foundation::HWND;

fn clamp01(value: f32) -> f32 {
    if value < 0.0 {
        0.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}

pub fn normalize_point(hwnd: HWND, x: i32, y: i32) -> Option<(f32, f32)> {
    let (width, height) = get_client_size(hwnd)?;
    if width <= 0 || height <= 0 {
        return None;
    }
    let nx = clamp01(x as f32 / width as f32);
    let ny = clamp01(y as f32 / height as f32);
    Some((nx, ny))
}

pub fn normalize_rect(
    hwnd: HWND,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> Option<(f32, f32, f32, f32)> {
    let (client_w, client_h) = get_client_size(hwnd)?;
    if client_w <= 0 || client_h <= 0 {
        return None;
    }
    let nx = clamp01(left as f32 / client_w as f32);
    let ny = clamp01(top as f32 / client_h as f32);
    let nw = clamp01(width as f32 / client_w as f32);
    let nh = clamp01(height as f32 / client_h as f32);
    Some((nx, ny, nw, nh))
}

pub fn denormalize_point(hwnd: HWND, x: f32, y: f32) -> Option<(i32, i32)> {
    let (width, height) = get_client_size(hwnd)?;
    if width <= 0 || height <= 0 {
        return None;
    }
    let max_x = (width - 1).max(0) as f32;
    let max_y = (height - 1).max(0) as f32;
    let px = (clamp01(x) * max_x).round() as i32;
    let py = (clamp01(y) * max_y).round() as i32;
    Some((px, py))
}

pub fn denormalize_rect(
    hwnd: HWND,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Option<(i32, i32, i32, i32)> {
    let (client_w, client_h) = get_client_size(hwnd)?;
    if client_w <= 0 || client_h <= 0 {
        return None;
    }
    let left = (clamp01(x) * client_w as f32).round() as i32;
    let top = (clamp01(y) * client_h as f32).round() as i32;
    let mut w = (clamp01(width) * client_w as f32).round() as i32;
    let mut h = (clamp01(height) * client_h as f32).round() as i32;
    if left + w > client_w {
        w = (client_w - left).max(0);
    }
    if top + h > client_h {
        h = (client_h - top).max(0);
    }
    Some((left, top, w, h))
}
