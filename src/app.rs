use crate::core::window::is_window_valid;
use crate::settings::{AppSettings, NamedMacro, MAX_CUSTOM_MACROS};
use crate::tools::collection_filler::CollectionFillerTool;
use crate::tools::custom_macro::CustomMacroTool;
use crate::tools::image_clicker::ImageClickerTool;
use crate::tools::r#trait::Tool;
use eframe::egui;
use windows::Win32::Foundation::HWND;

// Macro to toggle a tool with mutual exclusion

pub struct CabalHelperApp {
    // Centralized settings
    settings: AppSettings,

    // Tools collection (hardcoded tools + dynamic macro tools)
    tools: Vec<Box<dyn Tool>>,

    // Mapping of tool indices to their names (for dynamic macro naming)
    tool_names: Vec<String>,

    // UI State
    selected_tab: String,

    // Game context
    game_hwnd: Option<HWND>,
    status_message: String,

    // Overlay state
    is_overlay_mode: bool,
    show_log_panel: bool,
    show_help_window: bool,

    // Optimization state
    last_window_check: std::time::Instant,
    last_esc_check: std::time::Instant,
}

impl Default for CabalHelperApp {
    fn default() -> Self {
        // Load settings
        let settings = AppSettings::load();

        // Build tools dynamically
        let (tools, tool_names) = Self::build_tools(&settings);

        // Set initial tab to first tool
        let selected_tab = tool_names
            .get(0)
            .cloned()
            .unwrap_or_else(|| "Image Clicker".to_string());

        Self {
            settings,
            tools,
            tool_names,
            selected_tab,
            game_hwnd: None,
            status_message: "Ready".to_string(),
            is_overlay_mode: false,
            show_log_panel: false,
            show_help_window: false,
            last_window_check: std::time::Instant::now(),
            last_esc_check: std::time::Instant::now(),
        }
    }
}

impl CabalHelperApp {
    /// Build tools dynamically: hardcoded tools + one tool per custom macro
    fn build_tools(settings: &AppSettings) -> (Vec<Box<dyn Tool>>, Vec<String>) {
        let mut tools: Vec<Box<dyn Tool>> = Vec::new();
        let mut names: Vec<String> = Vec::new();

        // Hardcoded tools
        tools.push(Box::new(ImageClickerTool::default()));
        names.push("Image Clicker".to_string());

        tools.push(Box::new(CollectionFillerTool::default()));
        names.push("Collection Filler".to_string());

        // Dynamic custom macro tools (single universal macro type)
        for (idx, named_macro) in settings.custom_macros.iter().enumerate() {
            tools.push(Box::new(CustomMacroTool::new(idx)));
            names.push(named_macro.name.clone());
        }

        (tools, names)
    }

    /// Rebuild tools after settings change (e.g., adding/deleting a macro)
    fn rebuild_tools(&mut self) {
        let (tools, names) = Self::build_tools(&self.settings);
        self.tools = tools;
        self.tool_names = names;

        // Ensure selected tab still exists
        if !self.tool_names.contains(&self.selected_tab) {
            self.selected_tab = self
                .tool_names
                .get(0)
                .cloned()
                .unwrap_or_else(|| "Image Clicker".to_string());
        }
    }

    fn tool_visible_in_overlay(&self, idx: usize) -> bool {
        match idx {
            0 => self.settings.accept_item.show_in_overlay,
            1 => self.settings.collection_filler.show_in_overlay,
            _ => self
                .settings
                .custom_macros
                .get(idx - 2)
                .map(|macro_settings| macro_settings.show_in_overlay)
                .unwrap_or(true),
        }
    }

    fn overlay_tool_indices(&self) -> Vec<usize> {
        (0..self.tools.len())
            .filter(|idx| self.tool_visible_in_overlay(*idx))
            .collect()
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
        }

        if !self.is_overlay_mode && self.show_log_panel {
            let log_snapshot = self
                .tool_names
                .iter()
                .position(|name| name == &self.selected_tab)
                .and_then(|idx| self.tools.get(idx))
                .map(|tool| tool.get_log())
                .unwrap_or_default();

            egui::SidePanel::right("log_panel")
                .resizable(true)
                .default_width(280.0)
                .min_width(200.0)
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(12, 12, 12))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Log")
                                        .strong()
                                        .color(egui::Color32::LIGHT_GRAY),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} lines",
                                                log_snapshot.len()
                                            ))
                                            .small()
                                            .color(egui::Color32::DARK_GRAY),
                                        );
                                    },
                                );
                            });

                            ui.add_space(6.0);
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    if log_snapshot.is_empty() {
                                        ui.label(
                                            egui::RichText::new("No log entries yet.")
                                                .italics()
                                                .color(egui::Color32::DARK_GRAY),
                                        );
                                    } else {
                                        for line in log_snapshot {
                                            ui.label(
                                                egui::RichText::new(line)
                                                    .monospace()
                                                    .color(egui::Color32::from_rgb(200, 200, 200)),
                                            );
                                        }
                                    }
                                });
                        });
                });
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
                    let overlay_indices = self.overlay_tool_indices();

                    // Horizontal layout - tight fit with borders
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);

                        // Tool buttons with borders
                        for idx in overlay_indices {
                            let tool = &self.tools[idx];
                            let is_running = tool.is_running();
                            let name = self.tool_names.get(idx).map(|n| n.as_str()).unwrap_or("");
                            let btn_text: String = name.chars().take(2).collect();
                            let btn = egui::Button::new(
                                egui::RichText::new(btn_text).size(16.0).strong().color(
                                    if is_running {
                                        egui::Color32::GREEN
                                    } else {
                                        egui::Color32::WHITE
                                    },
                                ),
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
                                .color(egui::Color32::from_rgb(150, 150, 150)),
                        )
                        .min_size(egui::vec2(24.0, 36.0))
                        .fill(egui::Color32::from_rgba_premultiplied(40, 40, 40, 180))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)));

                        if ui.add(btn).clicked() {
                            self.is_overlay_mode = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                egui::WindowLevel::Normal,
                            ));
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                                [600.0, 450.0].into(),
                            ));
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

                            // Switch to this tool's tab
                            self.selected_tab = self.tool_names[idx].clone();
                        }
                        ctx.request_repaint();
                    }
                });
            } else {
                // Normal View
                let action = crate::ui::app_header::render_header(
                    ui,
                    &mut self.game_hwnd,
                    &mut self.status_message,
                );

                match action {
                    crate::ui::app_header::HeaderAction::Connect(hwnd) => {
                        self.game_hwnd = Some(hwnd);
                    }
                    crate::ui::app_header::HeaderAction::Disconnect => {
                        self.game_hwnd = None;
                    }
                    crate::ui::app_header::HeaderAction::ToggleLog => {
                        self.show_log_panel = !self.show_log_panel;
                    }
                    crate::ui::app_header::HeaderAction::ToggleOverlay => {
                        self.is_overlay_mode = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                            egui::WindowLevel::AlwaysOnTop,
                        ));

                        // Dynamic overlay sizing
                        let num_tools = self.overlay_tool_indices().len();
                        let overlay_width = (num_tools as f32 * 36.0) + 24.0; // 36px per tool + 24px settings button
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                            [overlay_width, 36.0].into(),
                        ));

                        // Initial positioning: top-center of game window (one-time only)
                        if let Some(game_hwnd) = self.game_hwnd {
                            if let Some((x, y, w, _h)) =
                                crate::core::window::get_client_rect_in_screen_coords(game_hwnd)
                            {
                                let target_x = x + (w / 2) - (overlay_width as i32 / 2);
                                let target_y = y as f32;
                                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                                    [target_x as f32, target_y].into(),
                                ));
                            }
                        }
                    }
                    crate::ui::app_header::HeaderAction::Help => {
                        self.show_help_window = true;
                    }
                    crate::ui::app_header::HeaderAction::None => {}
                }

                egui::Window::new("Help")
                    .open(&mut self.show_help_window)
                    .resizable(true)
                    .default_width(420.0)
                    .show(ctx, |ui| {
                        ui.heading("Cabal Helper");
                        ui.label("Automation tools for Cabal Online with overlay support.");
                        ui.add_space(6.0);

                        ui.collapsing("Overview", |ui| {
                            ui.label("- Connect to the game window to enable tools.");
                            ui.label("- Pick a tool tab, configure it, then Start.");
                            ui.label("- ESC stops any running tool immediately.");
                        });

                        ui.collapsing("Click Methods (Custom Macro -> Click)", |ui| {
                            ui.label("- Direct: SendMessage, default and reliable.");
                            ui.label("- Move: moves the cursor and clicks physically.");
                        });

                        ui.collapsing("Settings and Options", |ui| {
                            ui.label("- Image Clicker: image, interval, confidence, region.");
                            ui.label("- Collection Filler: red dot, delay, tolerance.");
                            ui.label("- Calibration sets areas and button positions.");
                            ui.label("- OCR Search: region, scale, invert, grayscale, decode.");
                        });

                        ui.collapsing("Overlay and Log", |ui| {
                            ui.label("- Overlay is an always-on-top tool bar.");
                            ui.label("- Drag the overlay bar to reposition it.");
                            ui.label("- Log panel shows output for the active tab.");
                        });

                        ui.collapsing("Autosave", |ui| {
                            ui.label("- Settings auto-save after changes.");
                            ui.label("- No manual save button is required.");
                        });
                    });

                ui.separator();

                // Dynamic Tab Rendering
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

                    for (_idx, name) in self.tool_names.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_tab == *name, name)
                            .clicked()
                        {
                            self.selected_tab = name.clone();
                        }
                    }

                    if self.settings.custom_macros.len() < MAX_CUSTOM_MACROS {
                        if ui.button("New Macro").clicked() {
                            let new_macro_name =
                                format!("Macro {}", self.settings.custom_macros.len() + 1);
                            self.settings
                                .custom_macros
                                .push(NamedMacro::new(new_macro_name.clone()));
                            self.rebuild_tools();
                            self.selected_tab = new_macro_name;
                            self.settings.auto_save();
                        }
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Find the selected tool by name and update it
                    if let Some(idx) = self
                        .tool_names
                        .iter()
                        .position(|name| name == &self.selected_tab)
                    {
                        if let Some(tool) = self.tools.get_mut(idx) {
                            tool.update(ctx, ui, &mut self.settings, self.game_hwnd);
                        }
                    }
                });

                // Check if macro count changed (e.g., macro was deleted)
                // We need to rebuild tools to stay in sync
                // 2 hardcoded (Image Clicker, Collection Filler) + N Custom macros
                let expected_tool_count = 2 + self.settings.custom_macros.len();
                if self.tools.len() != expected_tool_count {
                    self.rebuild_tools();
                }

                // Auto-save settings after tool updates
                self.settings.auto_save();
            }
        });
    }
}
