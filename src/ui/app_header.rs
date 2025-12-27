use eframe::egui;
use windows::Win32::Foundation::HWND;
use crate::core::window::find_game_window;

pub enum HeaderAction {
    Connect(HWND),
    Disconnect,
    Save,
    None
}

/// Render the app header with connection controls
pub fn render_header(
    ui: &mut egui::Ui,
    game_hwnd: &mut Option<HWND>,
    game_title: &mut String,
) -> HeaderAction {
    let mut action = HeaderAction::None;
    
    ui.horizontal(|ui| {
        ui.heading("Cabal Helper");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Save button always visible
            if ui.button("💾 Save Settings").clicked() {
                action = HeaderAction::Save;
            }
            
            ui.separator();

            if game_hwnd.is_none() {
                if ui.button("🔌 Connect to Game").clicked() {
                    if let Some(hwnd) = find_game_window() {
                        *game_hwnd = Some(hwnd);
                        // We set title here for immediate UI feedback, 
                         // but caller should also handle connection logic
                        *game_title = "Connected: PlayCabal EP36".to_string();
                        action = HeaderAction::Connect(hwnd);
                    } else {
                        *game_title = "Game not found".to_string();
                    }
                }
            } else {
                ui.label(egui::RichText::new(game_title.as_str()).color(egui::Color32::GREEN));
                if ui.button("❌ Disconnect").clicked() {
                    *game_hwnd = None;
                    *game_title = "Disconnected".to_string();
                    action = HeaderAction::Disconnect;
                }
            }
        });
    });
    
    action
}

/// Render tab navigation
pub fn render_tabs<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    selected_tab: &mut T,
    tabs: &[(T, &str)],
) {
    ui.horizontal(|ui| {
        for (tab_value, tab_label) in tabs {
            ui.selectable_value(selected_tab, *tab_value, *tab_label);
        }
    });
}
