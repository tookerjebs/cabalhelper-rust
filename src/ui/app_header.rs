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
    always_on_top: &mut bool,
) -> HeaderAction {
    let mut action = HeaderAction::None;

    // Use a Frame to give the header a distinct look
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(32, 33, 36)) // Darker background for header
        .inner_margin(egui::Margin::symmetric(16.0, 12.0))
        .rounding(egui::Rounding {
            sw: 8.0,
            se: 8.0,
            ..Default::default()
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // --- Left Side: Connection status ---
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // Status Dot
                    let (dot_color, status_text) = if game_hwnd.is_some() {
                        (egui::Color32::from_rgb(76, 175, 80), "Connected") // Green
                    } else {
                        (egui::Color32::from_rgb(244, 67, 54), "Disconnected") // Red
                    };

                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(10.0, 10.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().circle_filled(rect.center(), 4.0, dot_color);

                    // Status Text Stack
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(status_text)
                                .color(egui::Color32::from_rgb(220, 220, 220))
                                .strong()
                                .size(14.0),
                        );

                        if let Some(hwnd) = game_hwnd {
                            if let Some((_, _, w, h)) =
                                crate::core::window::get_client_rect_in_screen_coords(*hwnd)
                            {
                                ui.label(
                                    egui::RichText::new(format!("{} ({}x{})", game_title, w, h))
                                        .color(egui::Color32::from_rgb(150, 150, 150))
                                        .size(11.0),
                                );
                            }
                        } else {
                            ui.label(
                                egui::RichText::new("Waiting for game window...")
                                    .color(egui::Color32::from_rgb(100, 100, 100))
                                    .size(11.0),
                            );
                        }
                    });
                });

                // --- Right Side: Actions ---
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);

                    // Styled Button Helper
                    let styled_button = |ui: &mut egui::Ui, text: &str, fill: Option<egui::Color32>| {
                        let btn = egui::Button::new(
                            egui::RichText::new(text)
                                .size(13.0)
                                .color(egui::Color32::WHITE),
                        )
                        .min_size(egui::vec2(0.0, 28.0))
                        .rounding(6.0);
                        
                        // Use fill if provided (e.g., for Connect), otherwise transparent/default
                        if let Some(c) = fill {
                            ui.add(btn.fill(c))
                        } else {
                            ui.add(btn.fill(egui::Color32::from_white_alpha(10))) // Subtle frosted look
                        }
                    };

                    // Help
                    if ui.add(
                        egui::Button::new("?")
                            .rounding(100.0) // Circle
                            .min_size(egui::vec2(28.0, 28.0))
                            .fill(egui::Color32::from_white_alpha(10))
                    ).clicked() {
                        action = HeaderAction::Help;
                    }

                    // Log
                    if styled_button(ui, "Log", None).clicked() {
                        action = HeaderAction::ToggleLog;
                    }

                    // Overlay
                    if styled_button(ui, "Overlay", None).clicked() {
                        action = HeaderAction::ToggleOverlay;
                    }

                    ui.checkbox(always_on_top, "Always on top");

                    ui.add_space(8.0); // Separator between utilities and main action

                    // Connect/Disconnect
                    if game_hwnd.is_none() {
                        if styled_button(
                            ui, 
                            "Connect", 
                            Some(egui::Color32::from_rgb(50, 100, 200)) // Nice Blue
                        ).clicked() {
                            if let Some((hwnd, title)) = find_game_window() {
                                *game_hwnd = Some(hwnd);
                                *game_title = title;
                                action = HeaderAction::Connect(hwnd);
                            } else {
                                *game_title = "No D3D Window found".to_string();
                            }
                        }
                    } else {
                         if styled_button(
                            ui, 
                            "Disconnect", 
                            Some(egui::Color32::from_rgb(200, 60, 60)) // Red
                        ).clicked() {
                            *game_hwnd = None;
                            *game_title = "Disconnected".to_string();
                            action = HeaderAction::Disconnect;
                        }
                    }
                });
            });
        });

    action
}
