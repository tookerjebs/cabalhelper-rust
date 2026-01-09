use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    GetWindowDC, ReleaseDC, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject,
    DeleteDC, DeleteObject, GetDIBits, BitBlt, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};
use image::{ImageBuffer, Rgb};
use crate::core::window::get_client_rect_in_screen_coords;

/// Capture a region of a window using BitBlt
/// Note: This captures visible pixels, so the window should be visible
pub fn capture_region(
    hwnd: HWND,
    region: (i32, i32, i32, i32),
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, String> {
    unsafe {
        // Get window dimensions
        let window_rect = get_client_rect_in_screen_coords(hwnd)
            .ok_or_else(|| "Failed to get window client area".to_string())?;
        
        let window_width = window_rect.2;
        let window_height = window_rect.3;
        
        // Get window device context
        let hdc = GetWindowDC(hwnd);
        if hdc.is_invalid() {
            return Err("Failed to get window device context".to_string());
        }
        
        // Create compatible DC and bitmap for the entire window
        let mem_dc = CreateCompatibleDC(hdc);
        if mem_dc.is_invalid() {
            let _ = ReleaseDC(hwnd, hdc);
            return Err("Failed to create compatible DC".to_string());
        }
        
        let bitmap = CreateCompatibleBitmap(hdc, window_width, window_height);
        if bitmap.is_invalid() {
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(hwnd, hdc);
            return Err("Failed to create compatible bitmap".to_string());
        }
        
        let old_bitmap = SelectObject(mem_dc, bitmap);
        
        // Use BitBlt to capture the window content
        let result = BitBlt(
            mem_dc,
            0,
            0,
            window_width,
            window_height,
            hdc,
            0,
            0,
            SRCCOPY,
        );
        
        // BitBlt returns Result<(), windows::core::Error> in windows 0.52
        if result.is_err() {
            let _ = SelectObject(mem_dc, old_bitmap);
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(hwnd, hdc);
            return Err("BitBlt failed - could not capture window".to_string());
        }
        
        // Prepare bitmap info for GetDIBits
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: window_width,
                biHeight: -window_height, // Negative for top-down bitmap
                biPlanes: 1,
                biBitCount: 24, // RGB (3 bytes per pixel)
                biCompression: BI_RGB.0 as u32,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Default::default(); 1],
        };
        
        // Allocate buffer for pixel data (BGR format from Windows)
        let buffer_size = (window_width * window_height * 3) as usize;
        let mut buffer: Vec<u8> = vec![0; buffer_size];
        
        // Get bitmap bits
        let scan_lines = GetDIBits(
            mem_dc,
            bitmap,
            0,
            window_height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );
        
        // Cleanup GDI objects
        let _ = SelectObject(mem_dc, old_bitmap);
        let _ = DeleteObject(bitmap);
        let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(hwnd, hdc);
        
        if scan_lines == 0 {
            return Err("Failed to get bitmap bits".to_string());
        }
        
        // Extract the requested region from the full window capture
        let (region_x, region_y, region_width, region_height) = region;
        
        // Validate region bounds
        if region_x < 0 || region_y < 0 
            || region_x + region_width > window_width 
            || region_y + region_height > window_height 
        {
            return Err(format!(
                "Region ({}, {}, {}x{}) is out of window bounds ({}x{})",
                region_x, region_y, region_width, region_height, window_width, window_height
            ));
        }
        
        // Create output image buffer (RGB format)
        let mut img_buffer = ImageBuffer::new(region_width as u32, region_height as u32);
        
        // Copy pixels from captured buffer to image buffer
        // Windows uses BGR format, we need RGB
        for y in 0..region_height {
            for x in 0..region_width {
                let src_x = region_x + x;
                let src_y = region_y + y;
                let src_idx = ((src_y * window_width + src_x) * 3) as usize;
                
                // Convert BGR to RGB
                let b = buffer[src_idx];
                let g = buffer[src_idx + 1];
                let r = buffer[src_idx + 2];
                
                img_buffer.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
            }
        }
        
        Ok(img_buffer)
    }
}
