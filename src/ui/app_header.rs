use crate::core::window::find_game_window;
use eframe::egui;
use windows::Win32::Foundation::HWND;

pub enum HeaderAction {
    Connect(HWND),
    Disconnect,
    ToggleLog,
    ToggleOverlay,
    Help,
    None,
}

/// Render the unified app header (Connection Status + Utility Buttons)
pub fn render_header(
    ui: &mut egui::Ui,
    game_hwnd: &mut Option<HWND>,
    game_title: &mut String,
) -> HeaderAction {
    let mut action = HeaderAction::None;

    ui.horizontal(|ui| {
        // --- Left Side: Connection status stack ---
        ui.vertical(|ui| {
            if let Some(hwnd) = game_hwnd {
                ui.label(
                    egui::RichText::new(format!("Connected to {}", game_title))
                        .color(egui::Color32::from_rgb(168, 226, 187))
                        .strong(),
                );

                if let Some((_, _, w, h)) =
                    crate::core::window::get_client_rect_in_screen_coords(*hwnd)
                {
                    ui.label(
                        egui::RichText::new(format!("{}x{}", w, h))
                            .color(egui::Color32::from_rgb(140, 140, 140))
                            .small(),
                    );
                }
            } else {
                ui.label(
                    egui::RichText::new(format!("Status: {}", game_title))
                        .color(egui::Color32::from_rgb(200, 200, 200))
                        .strong(),
                );
            }
        });

        // --- Right Side: All Buttons ---
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
            let button_size = egui::vec2(66.0, 26.0);
            let compact_size = egui::vec2(66.0, 26.0);
            let help_size = egui::vec2(26.0, 26.0);

            if ui
                .add_sized(
                    help_size,
                    egui::Button::new(egui::RichText::new("?").strong())
                        .rounding(egui::Rounding::same(13.0)),
                )
                .clicked()
            {
                action = HeaderAction::Help;
            }

            if ui
                .add_sized(compact_size, egui::Button::new("Log"))
                .clicked()
            {
                action = HeaderAction::ToggleLog;
            }

            // Overlay Toggle (No Icon)
            if ui
                .add_sized(compact_size, egui::Button::new("Overlay"))
                .clicked()
            {
                action = HeaderAction::ToggleOverlay;
            }

            // Connect/Disconnect Button
            if game_hwnd.is_none() {
                if ui
                    .add_sized(button_size, egui::Button::new("Connect"))
                    .clicked()
                {
                    if let Some((hwnd, title)) = find_game_window() {
                        *game_hwnd = Some(hwnd);
                        *game_title = title;
                        action = HeaderAction::Connect(hwnd);
                    } else {
                        *game_title = "No D3D Window found".to_string();
                    }
                }
            } else {
                if ui
                    .add_sized(button_size, egui::Button::new("Disconnect"))
                    .clicked()
                {
                    *game_hwnd = None;
                    *game_title = "Disconnected".to_string();
                    action = HeaderAction::Disconnect;
                }
            }
        });
    });

    action
}
