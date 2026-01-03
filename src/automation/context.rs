use rustautogui::{RustAutoGui, MatchMode};
use windows::Win32::Foundation::HWND;
use crate::core::window::get_client_rect_in_screen_coords;


/// Automation context that encapsulates common automation setup
pub struct AutomationContext {
    pub gui: RustAutoGui,
    pub game_hwnd: HWND,
    pub window_rect: (i32, i32, i32, i32),
}

impl AutomationContext {
    /// Create a new automation context
    pub fn new(game_hwnd: HWND) -> Result<Self, String> {
        let gui = RustAutoGui::new(false)
            .map_err(|e| format!("Failed to initialize RustAutoGui: {}", e))?;
        
        // Use Client Rect (inner content) to be robust against window border differences
        let window_rect = get_client_rect_in_screen_coords(game_hwnd)
            .ok_or_else(|| "Failed to get window client area".to_string())?;

        
        Ok(Self {
            gui,
            game_hwnd,
            window_rect,
        })
    }
    
    /// Convert window-relative area to screen region
    pub fn to_screen_region(&self, area: (i32, i32, i32, i32)) -> (u32, u32, u32, u32) {
        (
            (self.window_rect.0 + area.0) as u32,
            (self.window_rect.1 + area.1) as u32,
            area.2 as u32,
            area.3 as u32
        )
    }
    
    /// Store a template with a window-relative region
    pub fn store_template(
        &mut self,
        path: &str,
        window_relative_region: Option<(i32, i32, i32, i32)>,
        alias: &str
    ) -> Result<(), String> {
        let screen_region = window_relative_region.map(|r| self.to_screen_region(r));
        
        self.gui.store_template_from_file(path, screen_region, MatchMode::Segmented, alias)
            .map_err(|e| format!("Failed to load template '{}': {}", alias, e))
    }
}
