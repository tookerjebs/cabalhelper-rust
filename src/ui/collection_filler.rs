use eframe::egui;
use crate::settings::CollectionFillerSettings;
use crate::calibration::{CalibrationManager, CalibrationResult};

#[derive(Debug, Clone, PartialEq)]
pub enum CalibrationItem {
    // Areas
    CollectionTabsArea,
    DungeonListArea,
    CollectionItemsArea,
    // Buttons
    AutoRefillButton,
    RegisterButton,
    YesButton,
    Page2Button,
    Page3Button,
    Page4Button,
    ArrowRightButton,
}

#[derive(Debug)]
pub enum UiAction {
    StartCalibration(CalibrationItem, bool), // item, is_area
    CancelCalibration,
    ClearCalibration(CalibrationItem),
    StartAutomation,
    StopAutomation,
    None,
}

/// Render the Collection Filler UI
pub fn render_ui(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    settings: &mut CollectionFillerSettings,
    calibration: &CalibrationManager,
    calibrating_item: &Option<CalibrationItem>,
    is_running: bool,
    status: &str,
    game_connected: bool,
) -> UiAction {
    let mut action = UiAction::None;

    if !game_connected {
        ui.colored_label(egui::Color32::RED, "Please connect to game first (top right)");
        return UiAction::None;
    }

    // Repaint if calibrating
    if calibration.is_active() {
        ctx.request_repaint();
    }

    ui.add_space(8.0);

    // 1. Settings Group
    ui.group(|ui| {
        ui.heading(egui::RichText::new("Configuration").size(14.0).strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Red Dot Image:").strong());
            ui.text_edit_singleline(&mut settings.red_dot_path);
            if ui.button("Browse...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image Files", &["png", "jpg", "jpeg", "bmp"])
                    .set_title("Select Red Dot Image")
                    .set_directory(std::env::current_dir().unwrap_or_default())
                    .pick_file()
                {
                    settings.red_dot_path = path.display().to_string();
                }
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Delay (ms):").strong());
            let mut delay = settings.delay_ms.to_string();
            if ui.text_edit_singleline(&mut delay).changed() {
                if let Ok(v) = delay.parse() { settings.delay_ms = v; }
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Red Dot Tolerance:").strong());
            ui.add(egui::Slider::new(&mut settings.red_dot_tolerance, 0.01..=0.99));
        });
    });

    ui.add_space(12.0);

    // 2. Calibration Section
    ui.group(|ui| {
        ui.heading(egui::RichText::new("Calibration").size(14.0).strong());
        ui.add_space(4.0);

        ui.label(egui::RichText::new("Detection Areas:").strong().underline());
        ui.add_space(4.0);

        if let Some(act) = render_area_calibration(ui, "Tabs Area", CalibrationItem::CollectionTabsArea,
            settings.collection_tabs_area, calibrating_item, calibration) { action = act; }
        if let Some(act) = render_area_calibration(ui, "Dungeon List", CalibrationItem::DungeonListArea,
            settings.dungeon_list_area, calibrating_item, calibration) { action = act; }
        if let Some(act) = render_area_calibration(ui, "Items Area", CalibrationItem::CollectionItemsArea,
            settings.collection_items_area, calibrating_item, calibration) { action = act; }

        ui.add_space(8.0);
        ui.label(egui::RichText::new("Action Buttons:").strong().underline());
        ui.add_space(4.0);

        if let Some(act) = render_button_calibration(ui, "Auto Refill", CalibrationItem::AutoRefillButton,
            settings.auto_refill_pos, calibrating_item, calibration) { action = act; }
        if let Some(act) = render_button_calibration(ui, "Register", CalibrationItem::RegisterButton,
            settings.register_pos, calibrating_item, calibration) { action = act; }
        if let Some(act) = render_button_calibration(ui, "Yes", CalibrationItem::YesButton,
            settings.yes_pos, calibrating_item, calibration) { action = act; }
        ui.separator();
        if let Some(act) = render_button_calibration(ui, "Page 2", CalibrationItem::Page2Button,
            settings.page_2_pos, calibrating_item, calibration) { action = act; }
        if let Some(act) = render_button_calibration(ui, "Page 3", CalibrationItem::Page3Button,
            settings.page_3_pos, calibrating_item, calibration) { action = act; }
        if let Some(act) = render_button_calibration(ui, "Page 4", CalibrationItem::Page4Button,
            settings.page_4_pos, calibrating_item, calibration) { action = act; }
        if let Some(act) = render_button_calibration(ui, "Arrow Right", CalibrationItem::ArrowRightButton,
            settings.arrow_right_pos, calibrating_item, calibration) { action = act; }
    });

    ui.add_space(12.0);

    // 3. Control
    ui.vertical_centered(|ui| {
        let (btn_text, btn_color) = if is_running {
            ("Stop Filler", egui::Color32::from_rgb(255, 100, 100))
        } else {
            ("Start Filler", egui::Color32::from_rgb(100, 255, 100))
        };

        let button = egui::Button::new(egui::RichText::new(btn_text).size(16.0).color(btn_color))
            .min_size(egui::vec2(200.0, 35.0));

        if ui.add(button).clicked() {
            action = if is_running {
                UiAction::StopAutomation
            } else {
                UiAction::StartAutomation
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

fn render_area_calibration(
    ui: &mut egui::Ui,
    label: &str,
    item: CalibrationItem,
    current: Option<(i32, i32, i32, i32)>,
    calibrating_item: &Option<CalibrationItem>,
    calibration: &CalibrationManager,
) -> Option<UiAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));

        if let Some((left, top, width, height)) = current {
            ui.label(egui::RichText::new(format!("({}, {}, {}x{})", left, top, width, height))
                .monospace()
                .strong());
        } else {
            ui.label(egui::RichText::new("Not set")
                .color(egui::Color32::from_rgb(150, 150, 150))
                .italics());
        }

        let is_this_calibrating = calibrating_item.as_ref() == Some(&item);

        if is_this_calibrating {
            if ui.button(egui::RichText::new("Stop").color(egui::Color32::from_rgb(255, 100, 100))).clicked() {
                action = Some(UiAction::CancelCalibration);
            }
            let label = if calibration.is_waiting_for_second_click() {
                "Click bottom-right"
            } else {
                "Click top-left"
            };
            ui.label(egui::RichText::new(label).color(egui::Color32::YELLOW));
        } else {
            if ui.button("Set").clicked() {
                action = Some(UiAction::StartCalibration(item.clone(), true));
            }
            if current.is_some() && ui.button("Clear").on_hover_text("Clear").clicked() {
                action = Some(UiAction::ClearCalibration(item));
            }
        }
    });
    action
}

fn render_button_calibration(
    ui: &mut egui::Ui,
    label: &str,
    item: CalibrationItem,
    current: Option<(i32, i32)>,
    calibrating_item: &Option<CalibrationItem>,
    _calibration: &CalibrationManager,
) -> Option<UiAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));

        if let Some((x, y)) = current {
            ui.label(egui::RichText::new(format!("({}, {})", x, y))
                .monospace()
                .strong());
        } else {
            ui.label(egui::RichText::new("Not set")
                .color(egui::Color32::from_rgb(150, 150, 150))
                .italics());
        }

        let is_this_calibrating = calibrating_item.as_ref() == Some(&item);

        if is_this_calibrating {
            if ui.button(egui::RichText::new("Stop").color(egui::Color32::from_rgb(255, 100, 100))).clicked() {
                action = Some(UiAction::CancelCalibration);
            }
            ui.label(egui::RichText::new("Click Button...").color(egui::Color32::YELLOW));
        } else {
            if ui.button("Set").clicked() {
                action = Some(UiAction::StartCalibration(item.clone(), false));
            }
            if current.is_some() && ui.button("Clear").on_hover_text("Clear").clicked() {
                action = Some(UiAction::ClearCalibration(item));
            }
        }
    });
    action
}

/// Apply calibration result to settings
pub fn apply_calibration_result(
    result: CalibrationResult,
    item: CalibrationItem,
    settings: &mut CollectionFillerSettings,
) {
    match (item, result) {
        (CalibrationItem::CollectionTabsArea, CalibrationResult::Area(l, t, w, h)) =>
            settings.collection_tabs_area = Some((l, t, w, h)),
        (CalibrationItem::DungeonListArea, CalibrationResult::Area(l, t, w, h)) =>
            settings.dungeon_list_area = Some((l, t, w, h)),
        (CalibrationItem::CollectionItemsArea, CalibrationResult::Area(l, t, w, h)) =>
            settings.collection_items_area = Some((l, t, w, h)),

        (CalibrationItem::AutoRefillButton, CalibrationResult::Point(x, y)) =>
            settings.auto_refill_pos = Some((x, y)),
        (CalibrationItem::RegisterButton, CalibrationResult::Point(x, y)) =>
            settings.register_pos = Some((x, y)),
        (CalibrationItem::YesButton, CalibrationResult::Point(x, y)) =>
            settings.yes_pos = Some((x, y)),
        (CalibrationItem::Page2Button, CalibrationResult::Point(x, y)) =>
            settings.page_2_pos = Some((x, y)),
        (CalibrationItem::Page3Button, CalibrationResult::Point(x, y)) =>
            settings.page_3_pos = Some((x, y)),
        (CalibrationItem::Page4Button, CalibrationResult::Point(x, y)) =>
            settings.page_4_pos = Some((x, y)),
        (CalibrationItem::ArrowRightButton, CalibrationResult::Point(x, y)) =>
            settings.arrow_right_pos = Some((x, y)),
        _ => {}
    }
}

/// Clear calibration value from settings
pub fn clear_calibration(item: CalibrationItem, settings: &mut CollectionFillerSettings) {
    match item {
        CalibrationItem::CollectionTabsArea => settings.collection_tabs_area = None,
        CalibrationItem::DungeonListArea => settings.dungeon_list_area = None,
        CalibrationItem::CollectionItemsArea => settings.collection_items_area = None,
        CalibrationItem::AutoRefillButton => settings.auto_refill_pos = None,
        CalibrationItem::RegisterButton => settings.register_pos = None,
        CalibrationItem::YesButton => settings.yes_pos = None,
        CalibrationItem::Page2Button => settings.page_2_pos = None,
        CalibrationItem::Page3Button => settings.page_3_pos = None,
        CalibrationItem::Page4Button => settings.page_4_pos = None,
        CalibrationItem::ArrowRightButton => settings.arrow_right_pos = None,
    }
}
