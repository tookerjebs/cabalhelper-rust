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

    last_window_always_on_top: bool,
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
            last_window_always_on_top: false,
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

    fn sync_tool_names_from_settings(&mut self) {
        let mut names: Vec<String> = Vec::with_capacity(2 + self.settings.custom_macros.len());
        names.push("Image Clicker".to_string());
        names.push("Collection Filler".to_string());
        for named_macro in &self.settings.custom_macros {
            names.push(named_macro.name.clone());
        }

        if names == self.tool_names {
            return;
        }

        let selected_index = self
            .tool_names
            .iter()
            .position(|name| name == &self.selected_tab);
        self.tool_names = names;
        if let Some(idx) = selected_index {
            if let Some(new_name) = self.tool_names.get(idx) {
                self.selected_tab = new_name.clone();
            }
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
        const LOG_PANEL_WIDTH: f32 = 280.0;
        const MIN_WINDOW_WIDTH: f32 = 400.0;

        // Adaptive repaint rate based on mode
        let repaint_interval = if self.is_overlay_mode {
            std::time::Duration::from_millis(100) // 10 FPS for overlay
        } else {
            std::time::Duration::from_millis(500) // 2 FPS for normal mode
        };
        ctx.request_repaint_after(repaint_interval);

        if !self.is_overlay_mode && self.last_window_always_on_top != self.settings.always_on_top {
            let level = if self.settings.always_on_top {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
            self.last_window_always_on_top = self.settings.always_on_top;
        }

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
                            let level = if self.settings.always_on_top {
                                egui::WindowLevel::AlwaysOnTop
                            } else {
                                egui::WindowLevel::Normal
                            };
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                            self.last_window_always_on_top = self.settings.always_on_top;
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                                [720.0, 620.0].into(),
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
                    &mut self.settings.always_on_top,
                );

                match action {
                    crate::ui::app_header::HeaderAction::Connect(hwnd) => {
                        self.game_hwnd = Some(hwnd);
                    }
                    crate::ui::app_header::HeaderAction::Disconnect => {
                        self.game_hwnd = None;
                    }
                    crate::ui::app_header::HeaderAction::ToggleLog => {
                        let inner_rect = ctx.input(|i| i.viewport().inner_rect);
                        let monitor_size = ctx.input(|i| i.viewport().monitor_size);
                        let current_size = inner_rect
                            .map(|rect| rect.size())
                            .unwrap_or(egui::vec2(720.0, 620.0));

                        self.show_log_panel = !self.show_log_panel;

                        let delta = if self.show_log_panel {
                            LOG_PANEL_WIDTH
                        } else {
                            -LOG_PANEL_WIDTH
                        };
                        let mut new_width = (current_size.x + delta).max(MIN_WINDOW_WIDTH);
                        if let Some(monitor) = monitor_size {
                            new_width = new_width.min(monitor.x);
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                            [new_width, current_size.y].into(),
                        ));
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

                if self.show_help_window {
                    let help_viewport_id = egui::ViewportId::from_hash_of("help_window");
                    let help_builder = egui::ViewportBuilder::default()
                        .with_title("Help")
                        .with_inner_size([640.0, 620.0])
                        .with_min_inner_size([520.0, 420.0])
                        .with_resizable(true);
                    let should_close = ctx.show_viewport_immediate(
                        help_viewport_id,
                        help_builder,
                        |ctx, _class| {
                            let mut close_requested = false;
                            if ctx.input(|i| i.viewport().close_requested()) {
                                close_requested = true;
                            }

                            egui::CentralPanel::default().show(ctx, |ui| {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.heading("Quick start");
                                        ui.label("- Connect to the game window.");
                                        ui.label("- Pick a tool tab, configure it, then press Start.");
                                        ui.label("- Press ESC any time to stop running tools.");

                                        ui.add_space(10.0);
                                        ui.heading("Image Clicker settings");
                                        ui.label("- Image Path: file to find on screen; use a clean PNG/JPG.");
                                        ui.label("- Interval (ms): time between searches; lower is faster.");
                                        ui.label("- Confidence: match threshold; higher is stricter.");
                                        ui.label("- Detection Area: optional; set region to speed up search.");
                                        ui.label("- Show in overlay: show this tool in the overlay bar.");

                                        ui.add_space(10.0);
                                        ui.heading("Collection Filler settings");
                                        ui.label("- Red Dot Image: the red dot screenshot used for detection.");
                                        ui.label("- Delay (ms): wait between clicks; too low can misclick.");
                                        ui.label("- Red Dot Tolerance: match threshold for the red dot.");
                                        ui.label("- Tabs Area: box that contains the collection tabs.");
                                        ui.label("- Dungeon List: box that contains the dungeon list.");
                                        ui.label("- Items Area: box that contains the collection items grid.");
                                        ui.label(
                                            "- Auto Refill/Register/Yes/Page 2-4/Arrow Right: click points.",
                                        );
                                        ui.label("- Show in overlay: show this tool in the overlay bar.");

                                        ui.add_space(10.0);
                                        ui.heading("Custom Macro settings");
                                        ui.label("- Macro Name: name shown in the tab and overlay.");
                                        ui.label("- Show in overlay: show this macro in the overlay bar.");
                                        ui.label("- Action: Click uses Position, Button, and Method.");
                                        ui.label("- Action: Type Text sends the exact text you enter.");
                                        ui.label("- Action: Delay waits the given milliseconds.");
                                        ui.label("- Action: OCR Search checks text/value in a region.");
                                        ui.label("  - Region: click top-left then bottom-right.");
                                        ui.label("  - Stat: text to look for (example: \"Defense\").");
                                        ui.label("  - Value: number to compare against OCR result.");
                                        ui.label("  - Alt target: optional second stat/value (OR).");
                                        ui.label("  - Value check: how the number is compared.");
                                        ui.label("  - Name match: exact or contains text match.");
                                        ui.label("  - Advanced settings: scale, grayscale, invert, decode.");

                                        ui.add_space(10.0);
                                        ui.heading("Notes");
                                        ui.label("- Recalibrate regions if the game window size changes.");
                                        ui.label("- Overlay mode shows a small always-on-top toolbar.");
                                        ui.label("- Log panel shows what the active tool is doing.");
                                        ui.label("- Settings save automatically.");
                                    });
                            });

                            close_requested
                        },
                    );

                    if should_close {
                        self.show_help_window = false;
                    }
                }

                ui.add_space(8.0); // Spacing after header

                // --- Browser-Style Tabs ---
                egui::Frame::none()
                    .fill(egui::Color32::TRANSPARENT)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
                            let tab_rounding = egui::Rounding {
                                nw: 6.0,
                                ne: 6.0,
                                sw: 0.0,
                                se: 0.0,
                            };

                            for (_idx, name) in self.tool_names.iter().enumerate() {
                                let is_selected = self.selected_tab == *name;
                                let (text_color, bg, stroke) = if is_selected {
                                    (
                                        egui::Color32::WHITE,
                                        egui::Color32::from_rgb(35, 35, 38),
                                        egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)),
                                    )
                                } else {
                                    (
                                        egui::Color32::from_rgb(170, 170, 170),
                                        egui::Color32::from_rgb(22, 22, 24),
                                        egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40)),
                                    )
                                };

                                let btn = egui::Button::new(
                                    egui::RichText::new(name)
                                        .size(13.0)
                                        .color(text_color)
                                        .strong(),
                                )
                                .frame(true)
                                .fill(bg)
                                .stroke(stroke)
                                .rounding(tab_rounding)
                                .min_size(egui::vec2(0.0, 30.0));

                                if ui.add(btn).clicked() {
                                    self.selected_tab = name.clone();
                                }
                            }

                            if self.settings.custom_macros.len() < MAX_CUSTOM_MACROS {
                                let btn = egui::Button::new(
                                    egui::RichText::new("+")
                                        .size(16.0)
                                        .color(egui::Color32::from_rgb(140, 140, 140)),
                                )
                                .frame(true)
                                .fill(egui::Color32::from_rgb(22, 22, 24))
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 40)))
                                .rounding(tab_rounding)
                                .min_size(egui::vec2(30.0, 30.0));

                                if ui.add(btn).clicked() {
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
                    });

                ui.add_space(4.0);

                // --- Main Content Area ---
                // Framed area for the tool content to give it depth
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(25, 25, 25)) // Slightly lighter than background
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 45, 45)))
                    .show(ui, |ui| {
                         egui::ScrollArea::vertical()
                            .auto_shrink([false, false]) // Expand to fill
                            .show(ui, |ui| {
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
                    });

                self.sync_tool_names_from_settings();

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
