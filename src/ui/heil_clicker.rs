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

    // Set coordinates button and position display in one row
    if game_connected {
        ui.horizontal(|ui| {
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
            
            // Show position inline
            if let Some((x, y)) = calibrated_pos {
                ui.label(egui::RichText::new(format!("Position: X={}, Y={}", x, y)).color(egui::Color32::LIGHT_GRAY));
            } else if !is_calibrating {
                ui.label(egui::RichText::new("Position: Not set").color(egui::Color32::GRAY));
            }
        });
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
