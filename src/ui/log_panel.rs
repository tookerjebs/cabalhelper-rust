use eframe::egui;

pub fn render_log_panel(ctx: &egui::Context, log_snapshot: &[String]) {
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
                            ui.label(
                                egui::RichText::new(format!("{} lines", log_snapshot.len()))
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
