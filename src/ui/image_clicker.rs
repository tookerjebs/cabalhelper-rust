use eframe::egui;

#[derive(Debug)]
pub enum ImageUiAction {
    StartRegionCalibration,
    CancelCalibration,
    ClearRegion,
    Start,
    Stop,
    None,
}

/// Render Image Clicker (Accept Item) UI
pub fn render_ui(
    ui: &mut egui::Ui,
    image_path: &mut String,
    interval_ms: &mut String,
    tolerance: &mut f32,
    search_region: Option<(i32, i32, i32, i32)>,
    is_calibrating: bool,
    is_waiting_for_second_click: bool,
    is_running: bool,
    status: &str,
    game_connected: bool,
) -> ImageUiAction {
    let mut action = ImageUiAction::None;

    ui.heading("Accept Item");
    ui.label("Automatically finds and clicks an image (e.g., accept button).");
    ui.separator();
    
    if !game_connected {
        ui.colored_label(egui::Color32::RED, "Please connect to game first (top right)");
        return ImageUiAction::None;
    }

    // Settings
    ui.horizontal(|ui| {
        ui.label("Image Path:");
        ui.text_edit_singleline(image_path);
    });
    
    ui.horizontal(|ui| {
        ui.label("Interval (ms):");
        ui.text_edit_singleline(interval_ms);
    });

    ui.horizontal(|ui| {
        ui.label("Min Confidence (0.0 - 1.0):");
        ui.add(egui::Slider::new(tolerance, 0.01..=0.99));
    });
    
    // Region calibration
    ui.add_space(10.0);
    ui.label("Search Region (optional - improves performance):");
    ui.horizontal(|ui| {
        let icon = if search_region.is_some() { "✓" } else { " " };
        ui.label(format!("[{}] Region", icon));
        
        if is_calibrating {
            if ui.button("Cancel").clicked() {
                action = ImageUiAction::CancelCalibration;
            }
            if is_waiting_for_second_click {
                ui.label("Click BOTTOM-RIGHT");
            } else {
                ui.label("Click TOP-LEFT");
            }
        } else {
            if ui.button("Set Region").clicked() {
                action = ImageUiAction::StartRegionCalibration;
            }
            if search_region.is_some() && ui.button("Clear").clicked() {
                action = ImageUiAction::ClearRegion;
            }
        }
    });

    ui.separator();

    // Controls
    if is_running {
        ui.colored_label(egui::Color32::GREEN, "RUNNING");
        if ui.button("Stop").clicked() {
            action = ImageUiAction::Stop;
        }
    } else {
        if ui.button("Start").clicked() {
            action = ImageUiAction::Start;
        }
    }

    ui.separator();
    
    // Status
    ui.label(format!("Status: {}", status));
    
    action
}
