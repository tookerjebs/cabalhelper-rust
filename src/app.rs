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

    // Overlay state
    is_overlay_mode: bool,
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
            is_overlay_mode: false,
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

        let mut panel = egui::CentralPanel::default();
        if self.is_overlay_mode {
            panel = panel.frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT));
        }

        panel.show(ctx, |ui| {
            if self.is_overlay_mode {
                // Overlay View
                // Drag the window by its background
                let response = ui.allocate_response(ui.available_size(), egui::Sense::drag());
                if response.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                
                // Draw content on top
                let painter = ui.painter();
                // Draw a nice rounded rect background
                painter.rect_filled(
                    response.rect,
                    egui::Rounding::same(8.0),
                    egui::Color32::from_black_alpha(200)
                );
                
                // Use a horizontal layout for the dock
                ui.allocate_ui_at_rect(response.rect, |ui| {
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing = egui::vec2(2.0, 0.0);
                        
                        // Helper to create tool buttons
                        let tool_btn = |ui: &mut egui::Ui, text: &str, is_running: bool| -> bool {
                            let btn = egui::Button::new(
                                egui::RichText::new(text)
                                    .size(20.0)
                                    .strong()
                                    .color(if is_running { egui::Color32::GREEN } else { egui::Color32::WHITE })
                            ).min_size(egui::vec2(48.0, 48.0)); // Slightly less than 52 to fit padding
                            
                            ui.add(btn).clicked()
                        };

                        // 1. Heil Clicker
                        if tool_btn(ui, "1", self.heil_clicker.is_running()) {
                           if self.heil_clicker.is_running() {
                               self.heil_clicker.stop();
                           } else {
                               // Stop others first (Mutual Exclusion)
                               self.collection_filler.stop();
                               self.image_clicker.stop();
                               self.heil_clicker.start(&self.settings.heil_clicker); 
                           }
                        }

                        // 2. Collection Filler
                        if tool_btn(ui, "2", self.collection_filler.is_running()) {
                           if self.collection_filler.is_running() {
                               self.collection_filler.stop();
                           } else {
                               self.heil_clicker.stop();
                               self.image_clicker.stop();
                               self.collection_filler.start(&self.settings.collection_filler);
                           }
                        }

                        // 3. Accept Item
                        if tool_btn(ui, "3", self.image_clicker.is_running()) {
                           if self.image_clicker.is_running() {
                               self.image_clicker.stop();
                           } else {
                               self.heil_clicker.stop();
                               self.collection_filler.stop();
                               self.image_clicker.start(&self.settings.accept_item);
                           }
                        }

                        ui.separator();

                        // Back Button
                         let btn = egui::Button::new(
                                egui::RichText::new("🔙").size(16.0)
                            ).min_size(egui::vec2(32.0, 48.0));
                            
                        if ui.add(btn).clicked() {
                            // Stop everything when closing overlay? Or keep running?
                            // For now, let's keep running but switch view
                            self.is_overlay_mode = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([600.0, 450.0].into()));
                        }
                    });
                });
            } else {
                // Normal View
                
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
                    crate::ui::app_header::HeaderAction::ToggleOverlay => {
                        self.is_overlay_mode = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                        // 4 buttons (3 tools + 1 back) * 52px width approx
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([220.0, 52.0].into()));
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
            }
        });
    }
}

