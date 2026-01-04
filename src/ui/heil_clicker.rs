use eframe::egui;

#[derive(Debug)]
pub enum HeilUiAction {
    StartCalibration,
    CancelCalibration,
    StartClicking,
    StopClicking,
    None,
}

/// Render Heil Clicker UI
pub fn render_ui(
    ui: &mut egui::Ui,
    delay_ms: &mut String,
    calibrated_pos: Option<(i32, i32)>,
    is_calibrating: bool,
    is_running: bool,
    status: &str,
    game_connected: bool,
) -> HeilUiAction {
    let mut action = HeilUiAction::None;

    ui.heading("Heils Clicker");
    ui.separator();

    // Delay input
    ui.horizontal(|ui| {
        ui.label("Delay (ms):");
        ui.text_edit_singleline(delay_ms);
    });

    ui.separator();

    // Connection status
    if !game_connected {
        ui.colored_label(egui::Color32::RED, "Please connect to game first (top right)");
    } else {
        ui.colored_label(egui::Color32::GREEN, "Game Connected");
    }

    ui.separator();

    // Set coordinates button
    if game_connected {
        if !is_calibrating {
            if ui.button("Set Coordinates").clicked() {
                action = HeilUiAction::StartCalibration;
            }
        } else {
            ui.label("🔴 Setting coordinates - Click on the game window now!");
            if ui.button("Cancel").clicked() {
                action = HeilUiAction::CancelCalibration;
            }
        }
    }

    ui.separator();

    // Show set coordinates
    if let Some((x, y)) = calibrated_pos {
        ui.label(format!("Position: X={}, Y={}", x, y));
    } else {
        ui.label("Position: Not set");
    }

    ui.separator();

    // Start/Stop button
    if !is_running {
        if ui.button("Start Clicking").clicked() {
            action = HeilUiAction::StartClicking;
        }
    } else {
        if ui.button("Stop Clicking").clicked() {
            action = HeilUiAction::StopClicking;
        }
    }

    ui.separator();
    
    // Status
    if is_running && game_connected {
        ui.label(format!("Status: Clicking... ({})", status));
    } else {
        ui.label(format!("Status: {}", status));
    }
    
    action
}
