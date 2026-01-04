use eframe::egui;
use crate::tools::heil_clicker::HeilClickerTool;
use crate::tools::image_clicker::ImageClickerTool;
use crate::tools::collection_filler::CollectionFillerTool;
use crate::tools::email_clicker::EmailClickerTool;
use crate::tools::r#trait::Tool;
use crate::core::window::is_window_valid;
use crate::settings::AppSettings;
use windows::Win32::Foundation::HWND;

// Macro to toggle a tool with mutual exclusion


pub struct CabalHelperApp {

    
    // Centralized settings
    settings: AppSettings,
    
    // Tools collection
    tools: Vec<Box<dyn Tool>>,
    
    // UI State
    selected_tab: String,
    
    // Game context
    game_hwnd: Option<HWND>,
    status_message: String,
    
    // Overlay state
    is_overlay_mode: bool,
    
    // Optimization state
    last_window_check: std::time::Instant,
    last_esc_check: std::time::Instant,
    
    // Cached game window rect for smart overlay snapping
    cached_game_rect: Option<(i32, i32, i32, i32)>,
}

impl Default for CabalHelperApp {
    fn default() -> Self {
        // Load settings
        let settings = AppSettings::load();
        
        // Initialize independent tools
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(HeilClickerTool::default()),
            Box::new(ImageClickerTool::default()),
            Box::new(CollectionFillerTool::default()),
            Box::new(EmailClickerTool::default())
        ];
        
        // Set initial tab to first tool
        let selected_tab = tools[0].get_name().to_string();

        Self {
            settings,
            tools,
            selected_tab,
            game_hwnd: None,
            status_message: "Ready".to_string(),
            is_overlay_mode: false,
            last_window_check: std::time::Instant::now(),
            last_esc_check: std::time::Instant::now(),
            cached_game_rect: None,
        }
    }
}

impl eframe::App for CabalHelperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Adaptive repaint rate based on mode
        let repaint_interval = if self.is_overlay_mode {
            std::time::Duration::from_millis(100) // 10 FPS for overlay
        } else {
            std::time::Duration::from_millis(500) // 2 FPS for normal mode
        };
        ctx.request_repaint_after(repaint_interval);
        
        // Emergency stop on ESC key
        use crate::core::input::is_escape_key_down;
        if self.last_esc_check.elapsed() > std::time::Duration::from_millis(100) {
            if is_escape_key_down() {
                for tool in &mut self.tools {
                    tool.stop();
                }
            }
            self.last_esc_check = std::time::Instant::now();
        }
        
        // Periodic check if window is still valid
        if self.last_window_check.elapsed() > std::time::Duration::from_secs(2) {
            if let Some(hwnd) = self.game_hwnd {
                if !is_window_valid(hwnd) {
                    self.game_hwnd = None;
                    self.status_message = "Connection Lost".to_string();
                }
            }
            self.last_window_check = std::time::Instant::now();
        }

        let mut panel = egui::CentralPanel::default();
        if self.is_overlay_mode {
            panel = panel.frame(egui::Frame::none());
            
            // Smart Auto-Snap Logic: only move when game window changes
            if let Some(game_hwnd) = self.game_hwnd {
                if let Some((x, y, w, h)) = crate::core::window::get_client_rect_in_screen_coords(game_hwnd) {
                    let current_rect = (x, y, w, h);
                    
                    // Only update position if game window rect changed
                    let rect_changed = self.cached_game_rect != Some(current_rect);
                    
                    if rect_changed {
                        let overlay_w = 168; // Horizontal: 4×36px + 1×24px buttons
                        let target_x = x + (w / 2) - (overlay_w / 2);
                        let target_y = y as f32;
                        
                        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition([target_x as f32, target_y].into()));
                        self.cached_game_rect = Some(current_rect);
                    }
                }
            }
        }

        panel.show(ctx, |ui| {
            if self.is_overlay_mode {
                // Overlay View
                let response = ui.allocate_response(ui.available_size(), egui::Sense::drag());
                if response.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                
                ui.allocate_ui_at_rect(response.rect, |ui| {
                    // Collect button states and actions first
                    let mut tool_to_toggle: Option<usize> = None;
                    
                    // Horizontal layout - tight fit with borders
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
                        
                        // Tool buttons with borders
                        for (idx, tool) in self.tools.iter().enumerate() {
                           let is_running = tool.is_running();
                           let btn_text = format!("{}", idx + 1);
                           let btn = egui::Button::new(
                                egui::RichText::new(btn_text)
                                    .size(16.0) 
                                    .strong()
                                    .color(if is_running { egui::Color32::GREEN } else { egui::Color32::WHITE })
                            )
                            .min_size(egui::vec2(36.0, 36.0))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)));
                            
                            if ui.add(btn).clicked() {
                                tool_to_toggle = Some(idx);
                            }
                        }

                         // Settings button with border
                         let btn = egui::Button::new(
                                egui::RichText::new("⚙")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(150, 150, 150))
                            )
                            .min_size(egui::vec2(24.0, 36.0))
                            .fill(egui::Color32::from_rgba_premultiplied(40, 40, 40, 180))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)));
                            
                        if ui.add(btn).clicked() {
                            self.is_overlay_mode = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([600.0, 450.0].into()));
                        }
                    });
                    
                    // Apply the toggle action after UI rendering
                    if let Some(idx) = tool_to_toggle {
                        let is_running = self.tools[idx].is_running();
                        if is_running {
                            self.tools[idx].stop();
                        } else {
                            // Stop all tools first
                            for tool in &mut self.tools {
                                tool.stop();
                            }
                            // Start the requested tool
                            self.tools[idx].start(&self.settings, self.game_hwnd);
                            
                            // Switch to this tool's tab so user can configure and start it from main UI
                            self.selected_tab = self.tools[idx].get_name().to_string();
                        }
                        ctx.request_repaint();
                    }
                });
            } else {
                // Normal View
                let header_action = crate::ui::app_header::render_header(ui);
                
                ui.separator();
                
                let connection_action = crate::ui::app_header::render_connection_panel(
                    ui,
                    &mut self.game_hwnd,
                    &mut self.status_message // Changed to use status_message instead of game_title
                );
                
                let action = if matches!(connection_action, crate::ui::app_header::HeaderAction::None) {
                    header_action
                } else {
                    connection_action
                };
                
                match action {
                    crate::ui::app_header::HeaderAction::Connect(hwnd) => {
                        self.game_hwnd = Some(hwnd);
                    },
                    crate::ui::app_header::HeaderAction::Disconnect => {
                        self.game_hwnd = None;
                    },
                    crate::ui::app_header::HeaderAction::Save => {
                        self.settings.auto_save();
                    },
                    crate::ui::app_header::HeaderAction::ToggleOverlay => {
                        self.is_overlay_mode = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([168.0, 36.0].into())); // Horizontal: 4×36px + 1×24px buttons
                    },
                    crate::ui::app_header::HeaderAction::None => {}
                }
                
                ui.separator();
            
                // Dynamic Tab Rendering
                ui.horizontal(|ui| {
                    for tool in &self.tools {
                        let name = tool.get_name();
                        if ui.selectable_label(self.selected_tab == name, name).clicked() {
                            self.selected_tab = name.to_string();
                        }
                    }
                });
                
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Find the selected tool and update it
                    if let Some(tool) = self.tools.iter_mut().find(|t| t.get_name() == self.selected_tab) {
                         tool.update(ctx, ui, &mut self.settings, self.game_hwnd);
                    }
                });
            }
        });
    }
}
