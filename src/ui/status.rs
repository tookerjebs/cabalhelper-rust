use eframe::egui;

pub fn render_status(ui: &mut egui::Ui, status: &str, hotkey_error: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Status:").strong());

        let status_color = if status.contains("Running") || status.contains("Active") {
            egui::Color32::from_rgb(100, 255, 100)
        } else if status.contains("Error") || status.contains("Failed") {
            egui::Color32::from_rgb(255, 100, 100)
        } else {
            egui::Color32::GRAY
        };

        ui.label(egui::RichText::new(status).color(status_color));
    });

    if let Some(err) = hotkey_error {
        let full = format!("Hotkey error: {}", err);
        let shortened = if full.len() > 80 {
            format!("{}...", &full[..77])
        } else {
            full.clone()
        };
        let label = egui::RichText::new(shortened).color(egui::Color32::from_rgb(200, 120, 120));
        let response = ui.label(label);
        if full.len() > 80 {
            response.on_hover_text(full);
        }
    }
}
