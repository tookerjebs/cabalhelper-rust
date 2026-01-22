use crate::settings::{HotkeyConfig, HotkeyKey, HotkeyModifiers};
use eframe::egui;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};

pub fn hotkey_label(config: &HotkeyConfig) -> String {
    let Some(key) = config.key else {
        return "Disabled".to_string();
    };

    let mut parts: Vec<&'static str> = Vec::new();
    if config.modifiers.ctrl {
        parts.push("Ctrl");
    }
    if config.modifiers.alt {
        parts.push("Alt");
    }
    if config.modifiers.shift {
        parts.push("Shift");
    }
    if config.modifiers.meta {
        parts.push("Meta");
    }
    parts.push(hotkey_key_label(key));
    parts.join("+")
}

pub fn hotkey_from_config(config: &HotkeyConfig) -> Option<HotKey> {
    let key = config.key?;
    let code = hotkey_key_to_code(key);
    let modifiers = hotkey_modifiers_to_code(config.modifiers);
    if modifiers.is_empty() {
        Some(HotKey::new(None, code))
    } else {
        Some(HotKey::new(Some(modifiers), code))
    }
}

fn hotkey_key_label(key: HotkeyKey) -> &'static str {
    match key {
        HotkeyKey::A => "A",
        HotkeyKey::B => "B",
        HotkeyKey::C => "C",
        HotkeyKey::D => "D",
        HotkeyKey::E => "E",
        HotkeyKey::F => "F",
        HotkeyKey::G => "G",
        HotkeyKey::H => "H",
        HotkeyKey::I => "I",
        HotkeyKey::J => "J",
        HotkeyKey::K => "K",
        HotkeyKey::L => "L",
        HotkeyKey::M => "M",
        HotkeyKey::N => "N",
        HotkeyKey::O => "O",
        HotkeyKey::P => "P",
        HotkeyKey::Q => "Q",
        HotkeyKey::R => "R",
        HotkeyKey::S => "S",
        HotkeyKey::T => "T",
        HotkeyKey::U => "U",
        HotkeyKey::V => "V",
        HotkeyKey::W => "W",
        HotkeyKey::X => "X",
        HotkeyKey::Y => "Y",
        HotkeyKey::Z => "Z",
        HotkeyKey::Digit0 => "0",
        HotkeyKey::Digit1 => "1",
        HotkeyKey::Digit2 => "2",
        HotkeyKey::Digit3 => "3",
        HotkeyKey::Digit4 => "4",
        HotkeyKey::Digit5 => "5",
        HotkeyKey::Digit6 => "6",
        HotkeyKey::Digit7 => "7",
        HotkeyKey::Digit8 => "8",
        HotkeyKey::Digit9 => "9",
        HotkeyKey::F1 => "F1",
        HotkeyKey::F2 => "F2",
        HotkeyKey::F3 => "F3",
        HotkeyKey::F4 => "F4",
        HotkeyKey::F5 => "F5",
        HotkeyKey::F6 => "F6",
        HotkeyKey::F7 => "F7",
        HotkeyKey::F8 => "F8",
        HotkeyKey::F9 => "F9",
        HotkeyKey::F10 => "F10",
        HotkeyKey::F11 => "F11",
        HotkeyKey::F12 => "F12",
        HotkeyKey::Escape => "Esc",
        HotkeyKey::Space => "Space",
        HotkeyKey::Enter => "Enter",
        HotkeyKey::Tab => "Tab",
        HotkeyKey::Backspace => "Backspace",
        HotkeyKey::Insert => "Insert",
        HotkeyKey::Delete => "Delete",
        HotkeyKey::Home => "Home",
        HotkeyKey::End => "End",
        HotkeyKey::PageUp => "Page Up",
        HotkeyKey::PageDown => "Page Down",
        HotkeyKey::ArrowUp => "Up",
        HotkeyKey::ArrowDown => "Down",
        HotkeyKey::ArrowLeft => "Left",
        HotkeyKey::ArrowRight => "Right",
    }
}

fn hotkey_key_to_code(key: HotkeyKey) -> Code {
    match key {
        HotkeyKey::A => Code::KeyA,
        HotkeyKey::B => Code::KeyB,
        HotkeyKey::C => Code::KeyC,
        HotkeyKey::D => Code::KeyD,
        HotkeyKey::E => Code::KeyE,
        HotkeyKey::F => Code::KeyF,
        HotkeyKey::G => Code::KeyG,
        HotkeyKey::H => Code::KeyH,
        HotkeyKey::I => Code::KeyI,
        HotkeyKey::J => Code::KeyJ,
        HotkeyKey::K => Code::KeyK,
        HotkeyKey::L => Code::KeyL,
        HotkeyKey::M => Code::KeyM,
        HotkeyKey::N => Code::KeyN,
        HotkeyKey::O => Code::KeyO,
        HotkeyKey::P => Code::KeyP,
        HotkeyKey::Q => Code::KeyQ,
        HotkeyKey::R => Code::KeyR,
        HotkeyKey::S => Code::KeyS,
        HotkeyKey::T => Code::KeyT,
        HotkeyKey::U => Code::KeyU,
        HotkeyKey::V => Code::KeyV,
        HotkeyKey::W => Code::KeyW,
        HotkeyKey::X => Code::KeyX,
        HotkeyKey::Y => Code::KeyY,
        HotkeyKey::Z => Code::KeyZ,
        HotkeyKey::Digit0 => Code::Digit0,
        HotkeyKey::Digit1 => Code::Digit1,
        HotkeyKey::Digit2 => Code::Digit2,
        HotkeyKey::Digit3 => Code::Digit3,
        HotkeyKey::Digit4 => Code::Digit4,
        HotkeyKey::Digit5 => Code::Digit5,
        HotkeyKey::Digit6 => Code::Digit6,
        HotkeyKey::Digit7 => Code::Digit7,
        HotkeyKey::Digit8 => Code::Digit8,
        HotkeyKey::Digit9 => Code::Digit9,
        HotkeyKey::F1 => Code::F1,
        HotkeyKey::F2 => Code::F2,
        HotkeyKey::F3 => Code::F3,
        HotkeyKey::F4 => Code::F4,
        HotkeyKey::F5 => Code::F5,
        HotkeyKey::F6 => Code::F6,
        HotkeyKey::F7 => Code::F7,
        HotkeyKey::F8 => Code::F8,
        HotkeyKey::F9 => Code::F9,
        HotkeyKey::F10 => Code::F10,
        HotkeyKey::F11 => Code::F11,
        HotkeyKey::F12 => Code::F12,
        HotkeyKey::Escape => Code::Escape,
        HotkeyKey::Space => Code::Space,
        HotkeyKey::Enter => Code::Enter,
        HotkeyKey::Tab => Code::Tab,
        HotkeyKey::Backspace => Code::Backspace,
        HotkeyKey::Insert => Code::Insert,
        HotkeyKey::Delete => Code::Delete,
        HotkeyKey::Home => Code::Home,
        HotkeyKey::End => Code::End,
        HotkeyKey::PageUp => Code::PageUp,
        HotkeyKey::PageDown => Code::PageDown,
        HotkeyKey::ArrowUp => Code::ArrowUp,
        HotkeyKey::ArrowDown => Code::ArrowDown,
        HotkeyKey::ArrowLeft => Code::ArrowLeft,
        HotkeyKey::ArrowRight => Code::ArrowRight,
    }
}

fn hotkey_modifiers_to_code(modifiers: HotkeyModifiers) -> Modifiers {
    let mut mods = Modifiers::empty();
    if modifiers.ctrl {
        mods |= Modifiers::CONTROL;
    }
    if modifiers.alt {
        mods |= Modifiers::ALT;
    }
    if modifiers.shift {
        mods |= Modifiers::SHIFT;
    }
    if modifiers.meta {
        mods |= Modifiers::META;
    }
    mods
}

pub fn try_capture_hotkey(ctx: &egui::Context) -> Option<HotkeyConfig> {
    let modifiers = ctx.input(|i| i.modifiers);
    let events = ctx.input(|i| i.events.clone());
    for event in events {
        if let egui::Event::Key {
            key,
            pressed: true,
            ..
        } = event
        {
            if let Some(hotkey_key) = egui_key_to_hotkey_key(key) {
                return Some(HotkeyConfig {
                    key: Some(hotkey_key),
                    modifiers: HotkeyModifiers {
                        ctrl: modifiers.ctrl,
                        alt: modifiers.alt,
                        shift: modifiers.shift,
                        meta: modifiers.command,
                    },
                });
            }
        }
    }
    None
}

fn egui_key_to_hotkey_key(key: egui::Key) -> Option<HotkeyKey> {
    match key {
        egui::Key::Escape => Some(HotkeyKey::Escape),
        egui::Key::Enter => Some(HotkeyKey::Enter),
        egui::Key::Tab => Some(HotkeyKey::Tab),
        egui::Key::Backspace => Some(HotkeyKey::Backspace),
        egui::Key::Space => Some(HotkeyKey::Space),
        egui::Key::ArrowLeft => Some(HotkeyKey::ArrowLeft),
        egui::Key::ArrowUp => Some(HotkeyKey::ArrowUp),
        egui::Key::ArrowRight => Some(HotkeyKey::ArrowRight),
        egui::Key::ArrowDown => Some(HotkeyKey::ArrowDown),
        egui::Key::PageUp => Some(HotkeyKey::PageUp),
        egui::Key::PageDown => Some(HotkeyKey::PageDown),
        egui::Key::Home => Some(HotkeyKey::Home),
        egui::Key::End => Some(HotkeyKey::End),
        egui::Key::Insert => Some(HotkeyKey::Insert),
        egui::Key::Delete => Some(HotkeyKey::Delete),
        egui::Key::F1 => Some(HotkeyKey::F1),
        egui::Key::F2 => Some(HotkeyKey::F2),
        egui::Key::F3 => Some(HotkeyKey::F3),
        egui::Key::F4 => Some(HotkeyKey::F4),
        egui::Key::F5 => Some(HotkeyKey::F5),
        egui::Key::F6 => Some(HotkeyKey::F6),
        egui::Key::F7 => Some(HotkeyKey::F7),
        egui::Key::F8 => Some(HotkeyKey::F8),
        egui::Key::F9 => Some(HotkeyKey::F9),
        egui::Key::F10 => Some(HotkeyKey::F10),
        egui::Key::F11 => Some(HotkeyKey::F11),
        egui::Key::F12 => Some(HotkeyKey::F12),
        egui::Key::Num0 => Some(HotkeyKey::Digit0),
        egui::Key::Num1 => Some(HotkeyKey::Digit1),
        egui::Key::Num2 => Some(HotkeyKey::Digit2),
        egui::Key::Num3 => Some(HotkeyKey::Digit3),
        egui::Key::Num4 => Some(HotkeyKey::Digit4),
        egui::Key::Num5 => Some(HotkeyKey::Digit5),
        egui::Key::Num6 => Some(HotkeyKey::Digit6),
        egui::Key::Num7 => Some(HotkeyKey::Digit7),
        egui::Key::Num8 => Some(HotkeyKey::Digit8),
        egui::Key::Num9 => Some(HotkeyKey::Digit9),
        egui::Key::A => Some(HotkeyKey::A),
        egui::Key::B => Some(HotkeyKey::B),
        egui::Key::C => Some(HotkeyKey::C),
        egui::Key::D => Some(HotkeyKey::D),
        egui::Key::E => Some(HotkeyKey::E),
        egui::Key::F => Some(HotkeyKey::F),
        egui::Key::G => Some(HotkeyKey::G),
        egui::Key::H => Some(HotkeyKey::H),
        egui::Key::I => Some(HotkeyKey::I),
        egui::Key::J => Some(HotkeyKey::J),
        egui::Key::K => Some(HotkeyKey::K),
        egui::Key::L => Some(HotkeyKey::L),
        egui::Key::M => Some(HotkeyKey::M),
        egui::Key::N => Some(HotkeyKey::N),
        egui::Key::O => Some(HotkeyKey::O),
        egui::Key::P => Some(HotkeyKey::P),
        egui::Key::Q => Some(HotkeyKey::Q),
        egui::Key::R => Some(HotkeyKey::R),
        egui::Key::S => Some(HotkeyKey::S),
        egui::Key::T => Some(HotkeyKey::T),
        egui::Key::U => Some(HotkeyKey::U),
        egui::Key::V => Some(HotkeyKey::V),
        egui::Key::W => Some(HotkeyKey::W),
        egui::Key::X => Some(HotkeyKey::X),
        egui::Key::Y => Some(HotkeyKey::Y),
        egui::Key::Z => Some(HotkeyKey::Z),
        _ => None,
    }
}
