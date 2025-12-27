use eframe::egui;
use crate::tools::heil_clicker::HeilClickerTool;
use crate::tools::image_clicker::ImageClickerTool;
use crate::core::window::{find_game_window, is_window_valid};
use windows::Win32::Foundation::HWND;

pub struct CabalHelperApp {
    // Current valid tools
    heil_clicker: HeilClickerTool,
    image_clicker: ImageClickerTool,
    
    // Global Game State
    game_hwnd: Option<HWND>,
    game_title: String,
    
    // Tab state
    selected_tab: Tab,
}

impl Default for CabalHelperApp {
    fn default() -> Self {
        Self {
            heil_clicker: HeilClickerTool::default(),
            image_clicker: ImageClickerTool::default(),
            game_hwnd: None,
            game_title: "Not Connected".to_string(),
            selected_tab: Tab::default(),
        }
    }
}

#[derive(PartialEq, Eq, Default)]
enum Tab {
    #[default]
    HeilClicker,
    CollectionFiller,
    ImageClicker,
}

impl eframe::App for CabalHelperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // Periodic check if window is still valid
        if let Some(hwnd) = self.game_hwnd {
            if !is_window_valid(hwnd) {
                self.game_hwnd = None;
                self.game_title = "Connection Lost".to_string();
                self.heil_clicker.set_game_hwnd(None);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Global Header
            ui.horizontal(|ui| {
                ui.heading("Cabal Helper");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.game_hwnd.is_none() {
                        if ui.button("🔌 Connect to Game").clicked() {
                            if let Some(hwnd) = find_game_window() {
                                self.game_hwnd = Some(hwnd);
                                // For now we assume typical title if found, or generic. 
                                // Ideally we get window title from HWND but for now simple static text:
                                self.game_title = "Connected: PlayCabal EP36".to_string(); 
                                self.heil_clicker.set_game_hwnd(Some(hwnd));
                            } else {
                                self.game_title = "Game not found".to_string();
                            }
                        }
                    } else {
                        ui.label(egui::RichText::new(&self.game_title).color(egui::Color32::GREEN));
                        if ui.button("❌ Disconnect").clicked() {
                            self.game_hwnd = None;
                            self.game_title = "Disconnected".to_string();
                            self.heil_clicker.set_game_hwnd(None);
                        }
                    }
                });
            });
            
            ui.separator();
        
            // Tab navigation bar
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::HeilClicker, "Heil Clicker");
                ui.selectable_value(&mut self.selected_tab, Tab::CollectionFiller, "Collection Filler");
                ui.selectable_value(&mut self.selected_tab, Tab::ImageClicker, "Image Clicker");
            });
            ui.separator();

            // Content area
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.selected_tab {
                    Tab::HeilClicker => {
                        self.heil_clicker.update(ui);
                    }
                    Tab::CollectionFiller => {
                        ui.heading("Collection Filler");
                        ui.label("This tool is coming soon!");
                        ui.add_space(10.0);
                        ui.colored_label(egui::Color32::from_rgb(255, 128, 0), "Placeholder: Migration pending.");
                    }
                    Tab::ImageClicker => {
                        self.image_clicker.update(ui);
                    }
                }
            });
        });
    }
}
