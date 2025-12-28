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
    
    // Optimization state
    last_window_check: std::time::Instant,
    last_esc_check: std::time::Instant,
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
            last_window_check: std::time::Instant::now(),
            last_esc_check: std::time::Instant::now(),
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
        // Adaptive repaint rate based on mode
        // Overlay: 10 FPS for smooth auto-snap, Normal: 2 FPS for low CPU
        let repaint_interval = if self.is_overlay_mode {
            std::time::Duration::from_millis(100) // 10 FPS for overlay
        } else {
            std::time::Duration::from_millis(500) // 2 FPS for normal mode
        };
        ctx.request_repaint_after(repaint_interval);
        
        // Emergency stop on ESC key - throttled to every 100ms
        use crate::core::input::is_escape_key_down;
        if self.last_esc_check.elapsed() > std::time::Duration::from_millis(100) {
            if is_escape_key_down() {
                self.heil_clicker.stop();
                self.collection_filler.stop();
                self.image_clicker.stop();
            }
            self.last_esc_check = std::time::Instant::now();
        }
        
        // Periodic check if window is still valid - throttled to every 2 seconds
        if self.last_window_check.elapsed() > std::time::Duration::from_secs(2) {
            if let Some(hwnd) = self.game_hwnd {
                if !is_window_valid(hwnd) {
                    self.game_hwnd = None;
                    self.game_title = "Connection Lost".to_string();
                    self.heil_clicker.set_game_hwnd(None);
                    self.collection_filler.set_game_hwnd(None);
                    self.image_clicker.set_game_hwnd(None);
                }
            }
            self.last_window_check = std::time::Instant::now();
        }

        let mut panel = egui::CentralPanel::default();
        if self.is_overlay_mode {
            panel = panel.frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT));
            
            // Auto-Snap Logic: Track Game Window
            if let Some(game_hwnd) = self.game_hwnd {
                if let Some((x, y, w, _h)) = crate::core::window::get_window_rect(game_hwnd) {
                     // Overlay Size is ~200x47 (10% smaller than 220x52)
                     // Target: Center-Top of Game Window
                     // Center X = x + (w / 2) - (overlay_width / 2)
                     let overlay_w = 200;
                     let target_x = x + (w / 2) - (overlay_w / 2);
                     let target_y = y + 8; // +8 for title bar padding
                     
                     // Send command to move window
                     // Note: To avoid jitter, we might want to check current pos first, 
                     // but egui doesn't give us window pos easily in update().
                     // We just send the command. Windows OS handles it efficiently if it's the same.
                     ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition([target_x as f32, target_y as f32].into()));
                }
            }
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
                                    .size(18.0) // Slightly smaller font
                                    .strong()
                                    .color(if is_running { egui::Color32::GREEN } else { egui::Color32::WHITE })
                            ).min_size(egui::vec2(43.0, 43.0)); // 10% smaller buttons
                            
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
                           ctx.request_repaint(); // Immediate visual update
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
                           ctx.request_repaint(); // Immediate visual update
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
                           ctx.request_repaint(); // Immediate visual update
                        }

                        ui.separator();

                        // Back Button
                         let btn = egui::Button::new(
                                egui::RichText::new("🔙").size(14.0)
                            ).min_size(egui::vec2(28.0, 43.0));
                            
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
                        // Scaled down size: 200x47
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([200.0, 47.0].into()));
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

