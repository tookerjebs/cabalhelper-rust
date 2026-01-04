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
    
    ui.heading("Collection Filler");
    ui.separator();

    if !game_connected {
        ui.colored_label(egui::Color32::RED, "Please connect to game first (top right)");
        return UiAction::None;
    }

    // Repaint if calibrating
    if calibration.is_active() {
        ctx.request_repaint();
    }

    // Calibration Section
    ui.heading("⚙️ Calibration");
    
    ui.label("Detection Areas:");
    if let Some(act) = render_area_calibration(ui, "Tabs Area", CalibrationItem::CollectionTabsArea, 
        settings.collection_tabs_area, calibrating_item, calibration) { action = act; }
    if let Some(act) = render_area_calibration(ui, "Dungeon List", CalibrationItem::DungeonListArea, 
        settings.dungeon_list_area, calibrating_item, calibration) { action = act; }
    if let Some(act) = render_area_calibration(ui, "Items Area", CalibrationItem::CollectionItemsArea, 
        settings.collection_items_area, calibrating_item, calibration) { action = act; }
    
    ui.add_space(10.0);
    ui.label("Action Buttons:");
    if let Some(act) = render_button_calibration(ui, "Auto Refill", CalibrationItem::AutoRefillButton, 
        settings.auto_refill_pos, calibrating_item, calibration) { action = act; }
    if let Some(act) = render_button_calibration(ui, "Register", CalibrationItem::RegisterButton, 
        settings.register_pos, calibrating_item, calibration) { action = act; }
    if let Some(act) = render_button_calibration(ui, "Yes", CalibrationItem::YesButton, 
        settings.yes_pos, calibrating_item, calibration) { action = act; }
    if let Some(act) = render_button_calibration(ui, "Page 2", CalibrationItem::Page2Button, 
        settings.page_2_pos, calibrating_item, calibration) { action = act; }
    if let Some(act) = render_button_calibration(ui, "Page 3", CalibrationItem::Page3Button, 
        settings.page_3_pos, calibrating_item, calibration) { action = act; }
    if let Some(act) = render_button_calibration(ui, "Page 4", CalibrationItem::Page4Button, 
        settings.page_4_pos, calibrating_item, calibration) { action = act; }
    if let Some(act) = render_button_calibration(ui, "Arrow Right", CalibrationItem::ArrowRightButton, 
        settings.arrow_right_pos, calibrating_item, calibration) { action = act; }

    ui.add_space(10.0);
    
    // Settings
    ui.heading("⚙️ Settings");
    
    ui.horizontal(|ui| {
        ui.label("Red Dot Image:");
        ui.text_edit_singleline(&mut settings.red_dot_path);
        if ui.button("📁 Browse...").clicked() {
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
    
    ui.horizontal(|ui| {
        ui.label("Delay (ms):");
        let mut delay = settings.delay_ms.to_string();
        if ui.text_edit_singleline(&mut delay).changed() {
            if let Ok(v) = delay.parse() { settings.delay_ms = v; }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Red Dot Tolerance:");
        ui.add(egui::Slider::new(&mut settings.red_dot_tolerance, 0.01..=0.99));
    });

    ui.separator();

    // Control
    if !is_running {
        if ui.button("▶️ Start").clicked() {
            action = UiAction::StartAutomation;
        }
    } else {
        if ui.button("⏹️ Stop").clicked() {
            action = UiAction::StopAutomation;
        }
    }

    ui.separator();
    ui.heading("📊 Status");
    ui.label(status);
    
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
        let icon = if current.is_some() { "✓" } else { " " };
        ui.label(format!("[{}] {}", icon, label));

        let is_this_calibrating = calibrating_item.as_ref() == Some(&item);
        
        if is_this_calibrating {
            if ui.button("Cancel").clicked() {
                action = Some(UiAction::CancelCalibration);
            }
            if calibration.is_waiting_for_second_click() {
                ui.label("Click BOTTOM-RIGHT");
            } else {
                ui.label("Click TOP-LEFT");
            }
        } else {
            if ui.button("Set").clicked() {
                action = Some(UiAction::StartCalibration(item.clone(), true));
            }
            if current.is_some() && ui.button("Clear").clicked() {
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
        let icon = if current.is_some() { "✓" } else { " " };
        ui.label(format!("[{}] {}", icon, label));

        let is_this_calibrating = calibrating_item.as_ref() == Some(&item);

        if is_this_calibrating {
            if ui.button("Cancel").clicked() {
                action = Some(UiAction::CancelCalibration);
            }
            ui.label("Click Button");
        } else {
            if ui.button("Set").clicked() {
                action = Some(UiAction::StartCalibration(item.clone(), false));
            }
            if current.is_some() && ui.button("Clear").clicked() {
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
