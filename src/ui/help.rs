use crate::core::hotkey::hotkey_label;
use crate::settings::AppSettings;
use eframe::egui;

pub fn render_help(ui: &mut egui::Ui, settings: &AppSettings) {
    ui.heading("Quick start");
    ui.label("- Use the header Connect button to hunt for the Cabal D3D window; the green dot confirms a match.");
    ui.label("- Pick a tool tab, fill the highlighted fields, then press Start (button turns Stop while running).");
    ui.label("- Use the Log button to follow progress and the emergency hotkey (header) to halt a running tool.");

    ui.add_space(6.0);
    ui.heading("Header controls");
    ui.label("- Connect / Disconnect: finds or drops the game window and shows its current size.");
    ui.label("- Overlay: switches to a compact toolbar; tools marked \"Show in overlay\" appear there.");
    ui.label("- Log: opens the right-hand log panel that shows the latest lines while running and the complete trace after stop.");
    ui.label("- ?: reopens this help panel when you need a refresher.");
    ui.label("- Always on top: keeps the main window above other apps.");
    ui.label(format!(
        "- Emergency stop: click to set the hotkey ({}) or press the hotkey/Stop to immediately cancel automation.",
        hotkey_label(&settings.emergency_stop_hotkey)
    ));

    ui.add_space(6.0);
    ui.heading("Image Clicker (Accept Item)");
    ui.label("- Image Path: the PNG/JPG the tool will scan for every cycle.");
    ui.label("- Interval (ms): time between scans; lower values repeat faster.");
    ui.label("- Confidence: how close the screenshot must match before clicking.");
    ui.label("- Detection Area: optionally limit the search rectangle for better speed.");
    ui.label("- Show in overlay: keeps this tool accessible from the overlay toolbar.");

    ui.add_space(6.0);
    ui.heading("Collection Filler");
    ui.label("- Red Dot Image + Tolerance: defines what to look for when scanning tabs.");
    ui.label("- Delay (ms): pause between automated clicks; keep it above 200 if the game feels unstable.");
    ui.label("- Calibrate the Tabs, Dungeon List, and Items Areas before running.");
    ui.label("- Calibrate the Auto Refill, Register, Yes, Page 2‑4, and Arrow Right buttons so clicks land correctly.");
    ui.label("- Show in overlay: include this automation in the compact toolbar.");

    ui.add_space(6.0);
    ui.heading("Custom Macros");
    ui.label("- Macro Name controls the tab text; \"Show in overlay\" makes it a quick toggle.");
    ui.label("- Actions execute sequentially: Click (position/button/method), Type Text, Delay, and OCR Search.");
    ui.label("- OCR Search: set a region by clicking top-left then bottom-right, enter the stat text, and the numeric value to compare.");
    ui.label("- Alt target: optional backup stat/value pair that respects the same comparison mode.");
    ui.label("- Comparison selects equals/≥/≤, and Name Match picks exact or contains.");
    ui.label("- Advanced OCR tweaks (scale, grayscale, invert, beam search) improve accuracy for different fonts.");

    ui.add_space(6.0);
    ui.heading("Notes");
    ui.label("- Recalibrate if the game window size or position changes.");
    ui.label("- Settings auto-save whenever you make a change.");
    ui.label("- If a tool shows an error, check the log (right panel) and stop before adjusting.");
}
