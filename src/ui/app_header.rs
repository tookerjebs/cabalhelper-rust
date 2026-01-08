use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::core::window::find_game_window;

pub enum HeaderAction {
    Connect(HWND),
    Disconnect,
    Save,
    ToggleOverlay,
    None
}


/// Render the unified app header (Connection Status + Utility Buttons)
pub fn render_header(
    ui: &mut egui::Ui,
    game_hwnd: &mut Option<HWND>,
    game_title: &mut String,
) -> HeaderAction {
    let mut action = HeaderAction::None;
    
    ui.horizontal(|ui| {
        // --- Left Side: Game Connection Status Text only ---
        if let Some(hwnd) = game_hwnd {
            ui.label(egui::RichText::new(game_title.as_str()).color(egui::Color32::GREEN).strong());
            
            // Show minimal window info: just resolution
            if let Some((_, _, w, h)) = crate::core::window::get_client_rect_in_screen_coords(*hwnd) {
                ui.label(egui::RichText::new("•").color(egui::Color32::DARK_GRAY));
                ui.label(egui::RichText::new(format!("{}x{}", w, h)).color(egui::Color32::LIGHT_GRAY).small());
            }
        } else {
            ui.label(egui::RichText::new("Not Connected").color(egui::Color32::GRAY));
        }

        // --- Right Side: All Buttons ---
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
             // Save Settings
            if ui.button("Save Settings").clicked() {
                action = HeaderAction::Save;
            }
            
            // Overlay Toggle (No Icon)
            if ui.button("Overlay").clicked() {
                action = HeaderAction::ToggleOverlay;
            }

            // Connect/Disconnect Button
            if game_hwnd.is_none() {
                if ui.button("Connect").clicked() {
                    if let Some((hwnd, title)) = find_game_window() {
                        *game_hwnd = Some(hwnd);
                        *game_title = title;
                        action = HeaderAction::Connect(hwnd);
                    } else {
                        *game_title = "No D3D Window found".to_string();
                    }
                }
            } else {
                if ui.button("Disconnect").clicked() {
                    *game_hwnd = None;
                    *game_title = "Disconnected".to_string();
                    action = HeaderAction::Disconnect;
                }
            }
        });
    });
    
    action
}
