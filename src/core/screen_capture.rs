use crate::core::window::{get_client_rect_in_screen_coords, get_window_rect_in_screen_coords};
use image::{ImageBuffer, Rgba};
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows_capture::capture::{Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};
use windows_capture::window::Window;

struct CapturedFrame {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct CaptureDebugInfo {
    pub requested_region_client: (i32, i32, i32, i32),
    pub client_rect_screen: (i32, i32, i32, i32),
    pub window_rect_screen: (i32, i32, i32, i32),
    pub frame_size: (u32, u32),
    pub scale: (f32, f32),
    pub crop_rect_frame: (i32, i32, i32, i32),
    pub window_dpi: u32,
}

struct CaptureFlags {
    region: (i32, i32, i32, i32),
    client_offset: (i32, i32),
    window_size: (i32, i32),
    client_rect_screen: (i32, i32, i32, i32),
    window_rect_screen: (i32, i32, i32, i32),
    window_dpi: u32,
    output: Arc<Mutex<Option<CapturedFrame>>>,
    debug: Arc<Mutex<Option<CaptureDebugInfo>>>,
}

struct OneShotCapture {
    flags: CaptureFlags,
    scratch: Vec<u8>,
}

impl GraphicsCaptureApiHandler for OneShotCapture {
    type Flags = CaptureFlags;
    type Error = String;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self {
            flags: ctx.flags,
            scratch: Vec::new(),
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let (region_x, region_y, region_w, region_h) = self.flags.region;
        if region_w <= 0 || region_h <= 0 {
            capture_control.stop();
            return Err("Invalid OCR region size".to_string());
        }

        let frame_w = frame.width();
        let frame_h = frame.height();
        if frame_w == 0 || frame_h == 0 {
            capture_control.stop();
            return Err("Invalid capture frame size".to_string());
        }

        let (offset_x, offset_y) = self.flags.client_offset;
        let (window_w, window_h) = self.flags.window_size;
        let scale_x = if window_w > 0 {
            frame_w as f32 / window_w as f32
        } else {
            1.0
        };
        let scale_y = if window_h > 0 {
            frame_h as f32 / window_h as f32
        } else {
            1.0
        };

        let start_x = ((offset_x + region_x) as f32 * scale_x).round() as i32;
        let start_y = ((offset_y + region_y) as f32 * scale_y).round() as i32;
        let end_x = (start_x as f32 + (region_w as f32 * scale_x)).round() as i32;
        let end_y = (start_y as f32 + (region_h as f32 * scale_y)).round() as i32;

        let sx = start_x.max(0).min(frame_w as i32);
        let sy = start_y.max(0).min(frame_h as i32);
        let ex = end_x.max(sx + 1).min(frame_w as i32);
        let ey = end_y.max(sy + 1).min(frame_h as i32);

        let buffer = frame
            .buffer_crop(sx as u32, sy as u32, ex as u32, ey as u32)
            .map_err(|e| format!("Capture buffer error: {}", e))?;

        let bytes = buffer.as_nopadding_buffer(&mut self.scratch).to_vec();
        let captured = CapturedFrame {
            width: buffer.width(),
            height: buffer.height(),
            rgba: bytes,
        };

        *self.flags.output.lock().unwrap() = Some(captured);
        *self.flags.debug.lock().unwrap() = Some(CaptureDebugInfo {
            requested_region_client: self.flags.region,
            client_rect_screen: self.flags.client_rect_screen,
            window_rect_screen: self.flags.window_rect_screen,
            frame_size: (frame_w, frame_h),
            scale: (scale_x, scale_y),
            crop_rect_frame: (sx, sy, ex - sx, ey - sy),
            window_dpi: self.flags.window_dpi,
        });
        capture_control.stop();
        Ok(())
    }
}

/// Capture a window region using Windows Graphics Capture.
pub fn capture_window_region_with_debug(
    hwnd: HWND,
    region: (i32, i32, i32, i32),
) -> Result<(ImageBuffer<Rgba<u8>, Vec<u8>>, CaptureDebugInfo), String> {
    let client_rect = get_client_rect_in_screen_coords(hwnd)
        .ok_or_else(|| "Failed to get client rect".to_string())?;
    let window_rect = get_window_rect_in_screen_coords(hwnd)
        .ok_or_else(|| "Failed to get window rect".to_string())?;
    let window_dpi = unsafe { GetDpiForWindow(hwnd) };

    let client_offset = (client_rect.0 - window_rect.0, client_rect.1 - window_rect.1);
    let window_size = (window_rect.2, window_rect.3);

    let output = Arc::new(Mutex::new(None));
    let debug = Arc::new(Mutex::new(None));
    let flags = CaptureFlags {
        region,
        client_offset,
        window_size,
        client_rect_screen: client_rect,
        window_rect_screen: window_rect,
        window_dpi,
        output: output.clone(),
        debug: debug.clone(),
    };

    let window = Window::from_raw_hwnd(hwnd.0 as *mut std::ffi::c_void);
    let settings = Settings::new(
        window,
        CursorCaptureSettings::Default,
        DrawBorderSettings::WithoutBorder,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Rgba8,
        flags,
    );

    let control = OneShotCapture::start_free_threaded(settings)
        .map_err(|e| format!("Capture start failed: {}", e))?;
    control
        .wait()
        .map_err(|e| format!("Capture wait failed: {}", e))?;

    let captured = output
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| "No capture frame received".to_string())?;
    let debug_info = debug
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| "No capture debug info".to_string())?;

    let image = ImageBuffer::from_raw(captured.width, captured.height, captured.rgba)
        .ok_or_else(|| "Failed to build capture image".to_string())?;

    Ok((image, debug_info))
}

pub fn window_capture_environment_diagnostics(hwnd: HWND) -> String {
    let client_rect = get_client_rect_in_screen_coords(hwnd);
    let window_rect = get_window_rect_in_screen_coords(hwnd);
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    let scale = dpi as f32 / 96.0;

    match (client_rect, window_rect) {
        (Some(c), Some(w)) => format!(
            "OCR ENV: dpi={} ({:.2}x), client_screen=({}, {}) {}x{}, window_screen=({}, {}) {}x{}",
            dpi, scale, c.0, c.1, c.2, c.3, w.0, w.1, w.2, w.3
        ),
        _ => format!(
            "OCR ENV: dpi={} ({:.2}x), failed to read client/window rects",
            dpi, scale
        ),
    }
}
