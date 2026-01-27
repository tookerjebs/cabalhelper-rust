use eframe::egui;

pub fn render_log_panel(ctx: &egui::Context, log_snapshot: &[String], is_running: bool) {
    const RUNNING_LOG_LINES: usize = 5;

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
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let display_count = if is_running {
                                log_snapshot.len().min(RUNNING_LOG_LINES)
                            } else {
                                log_snapshot.len()
                            };
                            let label = if is_running {
                                format!("{} lines (last {})", log_snapshot.len(), display_count)
                            } else {
                                format!("{} lines", log_snapshot.len())
                            };
                            ui.label(
                                egui::RichText::new(label)
                                    .small()
                                    .color(egui::Color32::DARK_GRAY),
                            );
                        });
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
                                let start_idx = if is_running {
                                    log_snapshot.len().saturating_sub(RUNNING_LOG_LINES)
                                } else {
                                    0
                                };
                                for line in &log_snapshot[start_idx..] {
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
