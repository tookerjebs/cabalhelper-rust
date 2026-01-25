use crate::core::hotkey::{hotkey_label, try_capture_hotkey};
use crate::core::window::find_game_window;
use crate::settings::{HotkeyConfig, HotkeyModifiers};
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
    emergency_stop_hotkey: &mut HotkeyConfig,
    capturing_emergency_hotkey: &mut bool,
    hotkey_error: Option<&str>,
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
            let show_connection_detail = ui.available_width() >= 520.0;

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
                    ui.add(btn.fill(egui::Color32::from_white_alpha(10)))
                    // Subtle frosted look
                }
            };

            ui.horizontal(|ui| {
                // --- Left Side: Connection status ---
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    // Status Dot
                    let dot_color = if game_hwnd.is_some() {
                        egui::Color32::from_rgb(76, 175, 80) // Green
                    } else {
                        egui::Color32::from_rgb(244, 67, 54) // Red
                    };

                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 4.0, dot_color);

                    // Status Text Stack
                    ui.vertical(|ui| {
                        if game_hwnd.is_none() {
                            if styled_button(
                                ui,
                                "Connect",
                                Some(egui::Color32::from_rgb(50, 100, 200)), // Nice Blue
                            )
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
                        } else if styled_button(
                            ui,
                            "Disconnect",
                            Some(egui::Color32::from_rgb(200, 60, 60)), // Red
                        )
                        .clicked()
                        {
                            *game_hwnd = None;
                            *game_title = "Disconnected".to_string();
                            action = HeaderAction::Disconnect;
                        }

                        if show_connection_detail {
                            if let Some(hwnd) = game_hwnd {
                                if let Some((_, _, w, h)) =
                                    crate::core::window::get_client_rect_in_screen_coords(*hwnd)
                                {
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(format!(
                                                "{} ({}x{})",
                                                game_title, w, h
                                            ))
                                            .color(egui::Color32::from_rgb(150, 150, 150))
                                            .size(11.0),
                                        )
                                        .wrap(false),
                                    );
                                }
                            } else {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new("Waiting for game window...")
                                            .color(egui::Color32::from_rgb(100, 100, 100))
                                            .size(11.0),
                                    )
                                    .wrap(false),
                                );
                            }
                        }
                    });
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                // --- Middle: Quick actions ---
                if styled_button(ui, "Overlay", None).clicked() {
                    action = HeaderAction::ToggleOverlay;
                }
                if styled_button(ui, "Log", None).clicked() {
                    action = HeaderAction::ToggleLog;
                }
                if ui
                    .add(
                        egui::Button::new("?")
                            .rounding(100.0) // Circle
                            .min_size(egui::vec2(28.0, 28.0))
                            .fill(egui::Color32::from_white_alpha(10)),
                    )
                    .clicked()
                {
                    action = HeaderAction::Help;
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                ui.checkbox(always_on_top, "Always on top");

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                // --- Emergency hotkey ---
                ui.label(
                    egui::RichText::new("Emergency stop:")
                        .color(egui::Color32::from_rgb(180, 180, 180)),
                );

                let label = if *capturing_emergency_hotkey {
                    "Press a key...".to_string()
                } else {
                    hotkey_label(emergency_stop_hotkey)
                };

                let button =
                    egui::Button::new(egui::RichText::new(label).color(egui::Color32::WHITE))
                        .min_size(egui::vec2(0.0, 24.0))
                        .fill(if *capturing_emergency_hotkey {
                            egui::Color32::from_rgb(90, 90, 120)
                        } else {
                            egui::Color32::from_white_alpha(10)
                        });

                if ui.add(button).clicked() {
                    *capturing_emergency_hotkey = true;
                }

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Clear")
                                .color(egui::Color32::from_rgb(200, 160, 160)),
                        )
                        .fill(egui::Color32::from_white_alpha(10))
                        .min_size(egui::vec2(0.0, 22.0)),
                    )
                    .clicked()
                {
                    emergency_stop_hotkey.key = None;
                    emergency_stop_hotkey.modifiers = HotkeyModifiers::default();
                }

                let _ = hotkey_error;
            });

            if *capturing_emergency_hotkey {
                if let Some(new_hotkey) = try_capture_hotkey(ui.ctx()) {
                    *emergency_stop_hotkey = new_hotkey;
                    *capturing_emergency_hotkey = false;
                }
            }
        });

    action
}
