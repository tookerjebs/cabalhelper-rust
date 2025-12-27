use eframe::egui;
use crate::tools::heil_clicker::HeilClickerTool;
use crate::tools::image_clicker::ImageClickerTool;

pub struct CabalHelperApp {
    // Current valid tools
    heil_clicker: HeilClickerTool,
    image_clicker: ImageClickerTool,
    
    // Tab state
    selected_tab: Tab,
}

impl Default for CabalHelperApp {
    fn default() -> Self {
        Self {
            heil_clicker: HeilClickerTool::default(),
            image_clicker: ImageClickerTool::default(),
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
        egui::CentralPanel::default().show(ctx, |ui| {
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
