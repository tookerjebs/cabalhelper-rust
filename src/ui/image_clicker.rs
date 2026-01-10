use eframe::egui;

#[derive(Debug)]
pub enum ImageUiAction {
    StartRegionCalibration,
    CancelCalibration,
    ClearRegion,
    Start,
    Stop,
    None,
}

/// Render Image Clicker (Accept Item) UI
pub fn render_ui(
    ui: &mut egui::Ui,
    image_path: &mut String,
    interval_ms: &mut String,
    tolerance: &mut f32,
    search_region: Option<(i32, i32, i32, i32)>,
    is_calibrating: bool,
    is_dragging: bool,
    is_running: bool,
    status: &str,
    game_connected: bool,
) -> ImageUiAction {
    let mut action = ImageUiAction::None;

    // Introduction
    ui.label(egui::RichText::new("Automatically finds and clicks an image (e.g., accept button).").italics());

    if !game_connected {
        ui.colored_label(egui::Color32::RED, "Please connect to game first (top right)");
        return ImageUiAction::None;
    }

    ui.add_space(8.0);

    // 1. Settings Group
    ui.group(|ui| {
        ui.heading(egui::RichText::new("Configuration").size(14.0).strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Image Path:").strong());
            ui.text_edit_singleline(image_path);
            if ui.button("Browse...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image Files", &["png", "jpg", "jpeg", "bmp"])
                    .set_title("Select Target Image")
                    .set_directory(std::env::current_dir().unwrap_or_default())
                    .pick_file()
                {
                    *image_path = path.display().to_string();
                }
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Interval (ms):").strong());
            ui.text_edit_singleline(interval_ms);
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Confidence:").strong());
            ui.add(egui::Slider::new(tolerance, 0.01..=0.99));
        });
    });

    ui.add_space(12.0);

    // 2. Region Group
    ui.group(|ui| {
        ui.heading(egui::RichText::new("Detection Area").size(14.0).strong());
        ui.add_space(4.0);

        ui.label(egui::RichText::new("Optional: Improve performance by limiting search area.").small().color(egui::Color32::GRAY));
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Region:").strong());

            if let Some((left, top, width, height)) = search_region {
                ui.label(egui::RichText::new(format!("({}, {}, {}x{})", left, top, width, height))
                    .monospace()
                    .strong());
            } else {
                ui.label(egui::RichText::new("Not set (Full Screen)")
                    .color(egui::Color32::YELLOW)
                    .italics());
            }

            ui.separator();

            if is_calibrating {
                if ui.button(egui::RichText::new("Stop").color(egui::Color32::from_rgb(255, 100, 100))).clicked() {
                    action = ImageUiAction::CancelCalibration;
                }
                if is_dragging {
                    ui.label(egui::RichText::new("Release to finish...").color(egui::Color32::YELLOW));
                } else {
                    ui.label(egui::RichText::new("Click and drag...").color(egui::Color32::YELLOW));
                }
            } else {
                if ui.button("Set Region").clicked() {
                    action = ImageUiAction::StartRegionCalibration;
                }
                if search_region.is_some() && ui.button("Clear").on_hover_text("Clear Region").clicked() {
                    action = ImageUiAction::ClearRegion;
                }
            }
        });
    });

    ui.add_space(12.0);

    // 3. Controls
    ui.vertical_centered(|ui| {
        let (btn_text, btn_color) = if is_running {
            ("Stop Image Clicker", egui::Color32::from_rgb(255, 100, 100))
        } else {
            ("Start Image Clicker", egui::Color32::from_rgb(100, 255, 100))
        };

        let button = egui::Button::new(egui::RichText::new(btn_text).size(16.0).color(btn_color))
            .min_size(egui::vec2(200.0, 35.0));

        if ui.add(button).clicked() {
            action = if is_running {
                ImageUiAction::Stop
            } else {
                ImageUiAction::Start
            };
        }
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(6.0);

    // 4. Status
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

    action
}
