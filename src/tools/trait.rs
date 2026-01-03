// Shared trait for all automation tools
use windows::Win32::Foundation::HWND;
use eframe::egui;
use crate::settings::AppSettings;

/// Common interface that all tools must implement
pub trait Tool {

    
    /// Stop the tool (emergency stop)
    fn stop(&mut self);
    
    /// Check if the tool is currently running
    fn is_running(&self) -> bool;
    
    /// Get current status message
    #[allow(dead_code)] // Used by implementations, not called directly on trait
    fn get_status(&self) -> String;

    /// Start the tool with the given settings
    fn start(&mut self, settings: &AppSettings, game_hwnd: Option<HWND>);

    /// Update loop for UI and logic
    fn update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, settings: &mut AppSettings, game_hwnd: Option<HWND>);

    /// Get tool name for tab identification
    fn get_name(&self) -> &str;
}
