use crate::core::hotkey::hotkey_label;
use crate::settings::AppSettings;
use eframe::egui;

pub fn render_help(ui: &mut egui::Ui, settings: &AppSettings) {
    ui.heading("Quick start");
    ui.label("- Connect to the game window.");
    ui.label("- Pick a tool tab, configure it, then press Start.");
    ui.label("- Optional: set an emergency stop hotkey in the header.");
    ui.label(format!(
        "- When set, press {} to stop running tools.",
        hotkey_label(&settings.emergency_stop_hotkey)
    ));

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
    ui.label("- Auto Refill/Register/Yes/Page 2-4/Arrow Right: click points.");
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
}
