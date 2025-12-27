use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub collection_filler: CollectionFillerSettings,
    
    #[serde(default)]
    pub heil_clicker: HeilClickerSettings,
    
    #[serde(default)]
    pub accept_item: AcceptItemSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionFillerSettings {
    // Detection Areas (stored as (left, top, width, height) relative to game window)
    pub collection_tabs_area: Option<(i32, i32, i32, i32)>,
    pub dungeon_list_area: Option<(i32, i32, i32, i32)>,
    pub collection_items_area: Option<(i32, i32, i32, i32)>,
    
    // Button Coordinates (x, y relative to game window)
    pub auto_refill_pos: Option<(i32, i32)>,
    pub register_pos: Option<(i32, i32)>,
    pub yes_pos: Option<(i32, i32)>,
    pub page_2_pos: Option<(i32, i32)>,
    pub page_3_pos: Option<(i32, i32)>,
    pub page_4_pos: Option<(i32, i32)>,
    pub arrow_right_pos: Option<(i32, i32)>,
    
    // Speed and matching settings
    pub delay_ms: u64,
    #[serde(default = "default_red_dot_tolerance")]
    pub red_dot_tolerance: f32,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeilClickerSettings {
    pub click_position: Option<(i32, i32)>,
    pub interval_ms: u64,
}

impl Default for HeilClickerSettings {
    fn default() -> Self {
        Self {
            click_position: None,
            interval_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptItemSettings {
    pub image_path: String,
    pub interval_ms: u64,
    pub tolerance: f32,
    pub search_region: Option<(i32, i32, i32, i32)>,
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,
}

impl Default for AcceptItemSettings {
    fn default() -> Self {
        Self {
            image_path: "image.png".to_string(),
            interval_ms: 1000,
            tolerance: 0.15, // 15% tolerance = 0.85 precision
            search_region: None,
            min_confidence: 0.90, // Only click if 90%+ confident
        }
    }
}

fn default_red_dot_tolerance() -> f32 {
    0.85
}

fn default_min_confidence() -> f32 {
    0.90
}

impl AppSettings {
    const SETTINGS_FILE: &'static str = "cabalhelper_settings.json";
    
    /// Load settings from file, or create default if doesn't exist
    pub fn load() -> Self {
        match fs::read_to_string(Self::SETTINGS_FILE) {
            Ok(contents) => {
                match serde_json::from_str(&contents) {
                    Ok(settings) => {
                        println!("✓ Loaded settings from {}", Self::SETTINGS_FILE);
                        settings
                    },
                    Err(e) => {
                        println!("⚠️ Failed to parse settings: {}, using defaults", e);
                        Self::default()
                    }
                }
            },
            Err(_) => {
                println!("No settings file found, using defaults");
                Self::default()
            }
        }
    }
    
    /// Save settings to file (auto-save)
    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        
        fs::write(Self::SETTINGS_FILE, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        Ok(())
    }
    
    /// Auto-save (ignores errors, just logs)
    pub fn auto_save(&self) {
        if let Err(e) = self.save() {
            println!("⚠️ Auto-save failed: {}", e);
        }
    }
}
