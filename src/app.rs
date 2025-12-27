use eframe::egui;
use crate::tools::heil_clicker::HeilClickerTool;
use crate::tools::image_clicker::ImageClickerTool;
use crate::tools::collection_filler::CollectionFillerTool;
use crate::tools::r#trait::Tool;
use crate::core::window::is_window_valid;
use crate::settings::AppSettings;
use windows::Win32::Foundation::HWND;

pub struct CabalHelperApp {
    // Current valid tools
    heil_clicker: HeilClickerTool,
    image_clicker: ImageClickerTool,
    collection_filler: CollectionFillerTool,
    
    // Centralized settings
    settings: AppSettings,
    
    // Global Game State
    game_hwnd: Option<HWND>,
    game_title: String,
    
    // Tab state
    selected_tab: Tab,
}

impl Default for CabalHelperApp {
    fn default() -> Self {
        // Load settings on startup
        let settings = AppSettings::load();
        
        Self {
            heil_clicker: HeilClickerTool::default(),
            image_clicker: ImageClickerTool::default(),
            collection_filler: CollectionFillerTool::default(),
            settings,
            game_hwnd: None,
            game_title: "Not Connected".to_string(),
            selected_tab: Tab::default(),
        }
    }
}

#[derive(PartialEq, Eq, Default, Clone, Copy)]
enum Tab {
    #[default]
    HeilClicker,
    CollectionFiller,
    AcceptItem,
}

impl eframe::App for CabalHelperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Emergency stop on ESC key - using Windows API so it works even when game has focus
        use crate::core::input::is_escape_key_down;
        if is_escape_key_down() {
            self.heil_clicker.stop();
            self.collection_filler.stop();
            self.image_clicker.stop();
        }
        
        // Periodic check if window is still valid
        if let Some(hwnd) = self.game_hwnd {
            if !is_window_valid(hwnd) {
                self.game_hwnd = None;
                self.game_title = "Connection Lost".to_string();
                self.heil_clicker.set_game_hwnd(None);
                self.collection_filler.set_game_hwnd(None);
                self.image_clicker.set_game_hwnd(None);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Global Header
            // Global Header
            let header_action = crate::ui::app_header::render_header(
                ui,
                &mut self.game_hwnd,
                &mut self.game_title
            );
            
            match header_action {
                crate::ui::app_header::HeaderAction::Connect(hwnd) => {
                    self.heil_clicker.set_game_hwnd(Some(hwnd));
                    self.collection_filler.set_game_hwnd(Some(hwnd));
                    self.image_clicker.set_game_hwnd(Some(hwnd));
                },
                crate::ui::app_header::HeaderAction::Disconnect => {
                    self.heil_clicker.set_game_hwnd(None);
                    self.collection_filler.set_game_hwnd(None);
                    self.image_clicker.set_game_hwnd(None);
                },
                crate::ui::app_header::HeaderAction::Save => {
                    self.settings.auto_save();
                },
                crate::ui::app_header::HeaderAction::None => {}
            }
            
            ui.separator();
        
            // Tab navigation bar
            let tabs = [
                (Tab::HeilClicker, "Heil Clicker"),
                (Tab::CollectionFiller, "Collection Filler"),
                (Tab::AcceptItem, "Accept Item"),
            ];
            crate::ui::app_header::render_tabs(ui, &mut self.selected_tab, &tabs);
            ui.separator();

            // Content area
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.selected_tab {
                    Tab::HeilClicker => {
                        self.heil_clicker.update(ui, &mut self.settings.heil_clicker);
                    }
                    Tab::CollectionFiller => {
                        self.collection_filler.update(ctx, ui, &mut self.settings.collection_filler);
                    }
                    Tab::AcceptItem => {
                        self.image_clicker.update(ctx, ui, &mut self.settings.accept_item);
                    }
                }
            });
        });
    }
}

