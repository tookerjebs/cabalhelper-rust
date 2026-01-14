use crate::core::coords::denormalize_rect;
use crate::core::window::get_client_rect_in_screen_coords;
use crate::settings::NormRect;
use rustautogui::{MatchMode, RustAutoGui};
use windows::Win32::Foundation::HWND;

/// Automation context that encapsulates common automation setup
pub struct AutomationContext {
    pub gui: RustAutoGui,
    pub game_hwnd: HWND,
}

impl AutomationContext {
    /// Create a new automation context
    pub fn new(game_hwnd: HWND) -> Result<Self, String> {
        let gui = RustAutoGui::new(false)
            .map_err(|e| format!("Failed to initialize RustAutoGui: {}", e))?;

        Ok(Self { gui, game_hwnd })
    }

    /// Convert normalized window-relative area to screen region
    pub fn to_screen_region(&self, area: NormRect) -> Option<(u32, u32, u32, u32)> {
        let (client_left, client_top, _, _) = get_client_rect_in_screen_coords(self.game_hwnd)?;
        let (rel_x, rel_y, width, height) =
            denormalize_rect(self.game_hwnd, area.0, area.1, area.2, area.3)?;
        Some((
            (client_left + rel_x) as u32,
            (client_top + rel_y) as u32,
            width as u32,
            height as u32,
        ))
    }

    /// Store a template with a window-relative region
    pub fn store_template(
        &mut self,
        path: &str,
        window_relative_region: Option<NormRect>,
        alias: &str,
    ) -> Result<(), String> {
        let screen_region = match window_relative_region {
            Some(region) => Some(
                self.to_screen_region(region)
                    .ok_or_else(|| "Failed to convert region".to_string())?,
            ),
            None => None,
        };

        self.gui
            .store_template_from_file(path, screen_region, MatchMode::Segmented, alias)
            .map_err(|e| format!("Failed to load template '{}': {}", alias, e))
    }
}
