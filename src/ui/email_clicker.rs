use eframe::egui;

#[derive(Debug, Clone)]
pub enum EmailUiAction {
    StartReceiveCalibration,
    StartNextCalibration,
    CancelCalibration,
    StartClicking,
    StopClicking,
    None,
}

pub fn render_ui(
    ui: &mut egui::Ui,
    cycles_str: &mut String,
    delay_ms_str: &mut String,
    receive_position: Option<(i32, i32)>,
    next_position: Option<(i32, i32)>,
    is_calibrating: bool,
    is_running: bool,
    status: &str,
    game_connected: bool,
) -> EmailUiAction {
    let mut action = EmailUiAction::None;

    ui.heading("E-mail Clicker");
    ui.add_space(10.0);

    // Status display
    ui.horizontal(|ui| {
        ui.label("Status:");
        ui.colored_label(
            if is_running {
                egui::Color32::GREEN
            } else {
                egui::Color32::GRAY
            },
            status
        );
    });
    ui.add_space(10.0);

    // Coordinate setup section
    ui.group(|ui| {
        ui.label("📍 Coordinates");
        ui.add_space(5.0);

        // Receive button calibration
        ui.horizontal(|ui| {
            ui.label("Receive Button:");
            if let Some((x, y)) = receive_position {
                ui.label(format!("({}, {})", x, y));
            } else {
                ui.colored_label(egui::Color32::RED, "Not set");
            }
            
            if !is_calibrating && !is_running {
                if ui.button("Set").clicked() {
                    action = EmailUiAction::StartReceiveCalibration;
                }
            }
        });

        // Next button calibration
        ui.horizontal(|ui| {
            ui.label("Next Button:");
            if let Some((x, y)) = next_position {
                ui.label(format!("({}, {})", x, y));
            } else {
                ui.colored_label(egui::Color32::RED, "Not set");
            }
            
            if !is_calibrating && !is_running {
                if ui.button("Set").clicked() {
                    action = EmailUiAction::StartNextCalibration;
                }
            }
        });

        if is_calibrating {
            ui.colored_label(egui::Color32::YELLOW, "🖱️ Click on the game window to set coordinates");
            if ui.button("Cancel").clicked() {
                action = EmailUiAction::CancelCalibration;
            }
        }
    });
    
    ui.add_space(10.0);

    // Settings section
    ui.group(|ui| {
        ui.label("⚙ Settings");
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Number of Cycles:");
            ui.add(egui::TextEdit::singleline(cycles_str).desired_width(80.0));
            ui.label("emails");
        });

        ui.horizontal(|ui| {
            ui.label("Delay between clicks:");
            ui.add(egui::TextEdit::singleline(delay_ms_str).desired_width(80.0));
            ui.label("ms");
        });
    });

    ui.add_space(10.0);

    // Control buttons
    ui.horizontal(|ui| {
        if is_running {
            if ui.button("⏹ Stop").clicked() {
                action = EmailUiAction::StopClicking;
            }
        } else {
            let can_start = game_connected 
                && receive_position.is_some() 
                && next_position.is_some()
                && !is_calibrating;
            
            ui.add_enabled_ui(can_start, |ui| {
                if ui.button("▶ Start").clicked() {
                    action = EmailUiAction::StartClicking;
                }
            });

            if !game_connected {
                ui.colored_label(egui::Color32::RED, "Connect to game first");
            } else if receive_position.is_none() || next_position.is_none() {
                ui.colored_label(egui::Color32::RED, "Set both button coordinates first");
            }
        }
    });

    action
}
