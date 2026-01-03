#![windows_subsystem = "windows"]
use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};

mod core;
mod tools;
mod app;
mod settings;
mod automation;
mod calibration;
mod ui;

use eframe::egui;
use app::CabalHelperApp;

fn main() -> Result<(), eframe::Error> {
    // Enable High DPI Awareness
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 450.0]) // Increased size for better tab view
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
