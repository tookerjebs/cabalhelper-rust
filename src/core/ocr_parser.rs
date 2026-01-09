use regex::Regex;
use crate::settings::ComparisonMode;

/// Parse OCR result into (stat_name, value)
/// Example: "Defense +20" -> ("defense", 20)
pub fn parse_ocr_result(text: &str) -> Option<(String, i32)> {
    // 1. Normalize: lowercase + remove special chars
    let normalized = text
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>();
    
    // 2. Extract stat name and value
    // Pattern: "stat_name number" (e.g., "defense 20", "hp 500")
    let re = Regex::new(r"^([a-z\s]+)\s+(\d+)$").ok()?;
    let caps = re.captures(normalized.trim())?;
    
    let stat = caps.get(1)?.as_str().trim().to_string();
    let value: i32 = caps.get(2)?.as_str().parse().ok()?;
    
    Some((stat, value))
}

/// Check if detected stat/value matches target
pub fn matches_target(
    detected_stat: &str,
    detected_value: i32,
    target_stat: &str,
    target_value: i32,
    comparison: ComparisonMode,
) -> bool {
    // Normalize both for comparison
    let detected = detected_stat.to_lowercase().trim().to_string();
    let target = target_stat.to_lowercase().trim().to_string();
    
    // Stat name must match
    if detected != target {
        return false;
    }
    
    // Compare value based on mode
    match comparison {
        ComparisonMode::Equals => detected_value == target_value,
        ComparisonMode::GreaterThanOrEqual => detected_value >= target_value,
        ComparisonMode::LessThanOrEqual => detected_value <= target_value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_defense() {
        assert_eq!(
            parse_ocr_result("Defense +20"),
            Some(("defense".to_string(), 20))
        );
    }

    #[test]
    fn test_parse_hp() {
        assert_eq!(
            parse_ocr_result("HP +500"),
            Some(("hp".to_string(), 500))
        );
    }

    #[test]
    fn test_parse_with_dots() {
        assert_eq!(
            parse_ocr_result("Crit. Dmg +15"),
            Some(("critdmg".to_string(), 15))
        );
    }

    #[test]
    fn test_matches_equal() {
        assert!(matches_target("defense", 20, "defense", 20, ComparisonMode::Equals));
        assert!(!matches_target("defense", 19, "defense", 20, ComparisonMode::Equals));
    }

    #[test]
    fn test_matches_gte() {
        assert!(matches_target("hp", 500, "hp", 500, ComparisonMode::GreaterThanOrEqual));
        assert!(matches_target("hp", 501, "hp", 500, ComparisonMode::GreaterThanOrEqual));
        assert!(!matches_target("hp", 499, "hp", 500, ComparisonMode::GreaterThanOrEqual));
    }
}
