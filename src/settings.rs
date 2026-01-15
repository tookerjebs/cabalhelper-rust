use serde::{Deserialize, Serialize};
use std::fs;

pub type NormPoint = (f32, f32);
pub type NormRect = (f32, f32, f32, f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub collection_filler: CollectionFillerSettings,

    #[serde(default)]
    pub accept_item: AcceptItemSettings,

    #[serde(default)]
    pub custom_macros: Vec<NamedMacro>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            collection_filler: CollectionFillerSettings::default(),
            accept_item: AcceptItemSettings::default(),
            custom_macros: vec![NamedMacro::default()],
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
    #[serde(default = "default_red_dot_tolerance")]
    pub red_dot_tolerance: f32,

    // Color filtering settings (to distinguish red dots from grey dots)
    #[serde(default = "default_min_red")]
    pub min_red: u8,
    #[serde(default = "default_red_dominance")]
    pub red_dominance: u8,

    // Red dot image path
    #[serde(default = "default_red_dot_path")]
    pub red_dot_path: String,

    #[serde(default = "default_true")]
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

fn default_red_dot_tolerance() -> f32 {
    0.85
}

fn default_min_red() -> u8 {
    150
}

fn default_red_dominance() -> u8 {
    30
}

fn default_red_dot_path() -> String {
    "red-dot.png".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptItemSettings {
    pub image_path: String,
    pub interval_ms: u64,
    pub tolerance: f32, // Treated as Minimum Confidence (0.0-1.0), default 0.85
    pub search_region: Option<NormRect>,
    #[serde(default = "default_true")]
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

fn default_scale_factor() -> u32 {
    2
}
fn default_true() -> bool {
    true
}
fn default_beam_width() -> u32 {
    10
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedMacro {
    pub name: String,
    #[serde(default = "default_true")]
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
        #[serde(default = "default_scale_factor")]
        scale_factor: u32,
        #[serde(default)]
        invert_colors: bool,
        #[serde(default = "default_true")]
        grayscale: bool,
        #[serde(default)]
        decode_mode: OcrDecodeMode,
        #[serde(default = "default_beam_width")]
        beam_width: u32,
        #[serde(default)]
        target_stat: String,
        #[serde(default)]
        target_value: i32,
        #[serde(default)]
        alt_target_enabled: bool,
        #[serde(default)]
        alt_target_stat: String,
        #[serde(default)]
        alt_target_value: i32,
        #[serde(default)]
        comparison: ComparisonMode,
        #[serde(default)]
        name_match_mode: OcrNameMatchMode,
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
    #[serde(default)]
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
            Ok(contents) => {
                match serde_json::from_str::<AppSettings>(&contents) {
                    Ok(mut settings) => {
                        // Ensure we have at least one macro
                        if settings.custom_macros.is_empty() {
                            settings.custom_macros.push(NamedMacro::default());
                        }
                        settings
                    }
                    Err(_) => Self::default(),
                }
            }
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
