use serde::{Deserialize, Serialize};
use std::fs;

pub type NormPoint = (f32, f32);
pub type NormRect = (f32, f32, f32, f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub collection_filler: CollectionFillerSettings,

    pub accept_item: AcceptItemSettings,

    pub custom_macros: Vec<NamedMacro>,

    #[serde(default = "default_emergency_stop_hotkey")]
    pub emergency_stop_hotkey: HotkeyConfig,

    pub always_on_top: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            collection_filler: CollectionFillerSettings::default(),
            accept_item: AcceptItemSettings::default(),
            custom_macros: vec![NamedMacro::default()],
            emergency_stop_hotkey: default_emergency_stop_hotkey(),
            always_on_top: false,
        }
    }
}

fn default_emergency_stop_hotkey() -> HotkeyConfig {
    HotkeyConfig {
        key: None,
        modifiers: HotkeyModifiers::default(),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HotkeyKey {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Escape,
    Space,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HotkeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub key: Option<HotkeyKey>,
    pub modifiers: HotkeyModifiers,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            key: Some(HotkeyKey::E),
            modifiers: HotkeyModifiers {
                ctrl: true,
                shift: true,
                alt: false,
                meta: false,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionFillerSettings {
    // Detection Areas (stored as normalized (x, y, w, h) relative to client size)
    pub collection_tabs_area: Option<NormRect>,
    pub dungeon_list_area: Option<NormRect>,
    pub collection_items_area: Option<NormRect>,

    // Button Coordinates (normalized x, y relative to client size)
    pub auto_refill_pos: Option<NormPoint>,
    pub register_pos: Option<NormPoint>,
    pub yes_pos: Option<NormPoint>,
    pub page_2_pos: Option<NormPoint>,
    pub page_3_pos: Option<NormPoint>,
    pub page_4_pos: Option<NormPoint>,
    pub arrow_right_pos: Option<NormPoint>,

    // Speed and matching settings
    pub delay_ms: u64,
    pub red_dot_tolerance: f32,

    // Color filtering settings (to distinguish red dots from grey dots)
    pub min_red: u8,
    pub red_dominance: u8,

    // Red dot image path
    pub red_dot_path: String,

    pub show_in_overlay: bool,
}

impl Default for CollectionFillerSettings {
    fn default() -> Self {
        Self {
            collection_tabs_area: None,
            dungeon_list_area: None,
            collection_items_area: None,
            auto_refill_pos: None,
            register_pos: None,
            yes_pos: None,
            page_2_pos: None,
            page_3_pos: None,
            page_4_pos: None,
            arrow_right_pos: None,
            delay_ms: 31,
            red_dot_tolerance: 0.85,
            min_red: 150,
            red_dominance: 30,
            red_dot_path: "red-dot.png".to_string(),
            show_in_overlay: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptItemSettings {
    pub image_path: String,
    pub interval_ms: u64,
    pub tolerance: f32, // Treated as Minimum Confidence (0.0-1.0), default 0.85
    pub search_region: Option<NormRect>,
    pub show_in_overlay: bool,
}

impl Default for AcceptItemSettings {
    fn default() -> Self {
        Self {
            image_path: "image.png".to_string(),
            interval_ms: 100, // Reduced from 1000ms for faster detection
            tolerance: 0.85,
            search_region: None,
            show_in_overlay: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum ComparisonMode {
    Equals,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl Default for ComparisonMode {
    fn default() -> Self {
        ComparisonMode::GreaterThanOrEqual
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum OcrDecodeMode {
    Greedy,
    BeamSearch,
}

impl Default for OcrDecodeMode {
    fn default() -> Self {
        OcrDecodeMode::Greedy
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum OcrNameMatchMode {
    Exact,
    Contains,
}

impl Default for OcrNameMatchMode {
    fn default() -> Self {
        OcrNameMatchMode::Contains
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OcrAltTarget {
    pub target_stat: String,
    pub target_value: i32,
    pub comparison: ComparisonMode,
    pub name_match_mode: OcrNameMatchMode,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedMacro {
    pub name: String,
    pub show_in_overlay: bool,
    pub settings: CustomMacroSettings,
}

impl NamedMacro {
    pub fn new(name: String) -> Self {
        Self {
            name,
            show_in_overlay: true,
            settings: CustomMacroSettings::default(),
        }
    }
}

impl Default for NamedMacro {
    fn default() -> Self {
        Self::new("My Macro".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MacroAction {
    Click {
        coordinate: Option<NormPoint>,
        button: MouseButton,
        #[serde(default)]
        click_method: ClickMethod,
        use_mouse_movement: bool,
    },
    TypeText {
        text: String,
    },
    Delay {
        milliseconds: u64,
    },
    OcrSearch {
        ocr_region: Option<NormRect>,
        scale_factor: u32,
        invert_colors: bool,
        grayscale: bool,
        decode_mode: OcrDecodeMode,
        beam_width: u32,
        target_stat: String,
        target_value: i32,
        comparison: ComparisonMode,
        name_match_mode: OcrNameMatchMode,
        alt_targets: Vec<OcrAltTarget>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum ClickMethod {
    SendMessage,   // Direct click (current default)
    MouseMovement, // Physical mouse movement
}

impl Default for ClickMethod {
    fn default() -> Self {
        ClickMethod::SendMessage
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl Default for MouseButton {
    fn default() -> Self {
        MouseButton::Left
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMacroSettings {
    pub actions: Vec<MacroAction>,
    pub loop_enabled: bool,
    pub infinite_loop: bool,
    pub loop_count: u32,
}

impl Default for CustomMacroSettings {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            loop_enabled: false,
            infinite_loop: false,
            loop_count: 1,
        }
    }
}

pub const MAX_CUSTOM_MACROS: usize = 10;

impl AppSettings {
    const SETTINGS_FILE: &'static str = "cabalhelper_settings.json";

    /// Load settings from file, or create default if doesn't exist
    pub fn load() -> Self {
        match fs::read_to_string(Self::SETTINGS_FILE) {
            Ok(contents) => match serde_json::from_str::<AppSettings>(&contents) {
                Ok(settings) => settings,
                Err(_) => Self::default(),
            },
            Err(_) => Self::default(),
        }
    }

    /// Save settings to file (auto-save)
    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        fs::write(Self::SETTINGS_FILE, json).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    /// Auto-save (ignores errors)
    pub fn auto_save(&self) {
        let _ = self.save();
    }
}
