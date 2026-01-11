#![windows_subsystem = "windows"]
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};

mod app;
mod automation;
mod calibration;
mod core;
mod settings;
mod tools;
mod ui;

use app::CabalHelperApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // Enable High DPI Awareness
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 520.0]) // Increased size for better tab view
            .with_title("Cabal Helper - Rust Edition")
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "Cabal Helper",
        options,
        Box::new(|_cc| Box::new(CabalHelperApp::default())),
    )
}
