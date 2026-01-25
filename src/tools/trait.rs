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

    /// Start the tool with the given settings
    fn start(&mut self, settings: &AppSettings, game_hwnd: Option<HWND>);

    /// Update loop for UI and logic
    fn update(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        settings: &mut AppSettings,
        game_hwnd: Option<HWND>,
        hotkey_error: Option<&str>,
    );

    /// Read current status log (for UI display)
    fn get_log(&self) -> Vec<String>;
}
