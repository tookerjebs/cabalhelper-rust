mod core;
mod tools;
mod app;

use eframe::egui;
use app::CabalHelperApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 450.0]) // Increased size for better tab view
            .with_title("Cabal Helper - Rust Edition"),
        ..Default::default()
    };

    eframe::run_native(
        "Cabal Helper",
        options,
        Box::new(|_cc| Box::new(CabalHelperApp::default())),
    )
}
