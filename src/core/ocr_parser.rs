use regex::Regex;
use crate::settings::ComparisonMode;

/// Parse OCR result into (stat_name, value)
/// Example: "Defense +20" -> ("defense", 20)
pub fn parse_ocr_result(text: &str) -> Option<(String, i32)> {
    let lower = text.to_lowercase();
    let number_re = Regex::new(r"[+-]?\d+").ok()?;
    let number_match = number_re.find(&lower)?;
    let value: i32 = number_match.as_str().parse().ok()?;

    let (left, right_with_number) = lower.split_at(number_match.start());
    let right = &right_with_number[number_match.as_str().len()..];

    let stat_left = extract_stat_words(left);
    let stat_right = extract_stat_words(right);
    let stat = if !stat_left.is_empty() {
        stat_left
    } else if !stat_right.is_empty() {
        stat_right
    } else {
        extract_stat_words(&lower)
    };

    if stat.is_empty() {
        None
    } else {
        Some((stat, value))
    }
}

fn extract_stat_words(text: &str) -> String {
    let word_re = Regex::new(r"[a-z]+").ok();
    let Some(re) = word_re else { return String::new(); };
    re.find_iter(text)
        .map(|m| m.as_str())
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
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
            Some(("crit dmg".to_string(), 15))
        );
    }

    #[test]
    fn test_parse_number_first() {
        assert_eq!(
            parse_ocr_result("+20 Defense"),
            Some(("defense".to_string(), 20))
        );
    }

    #[test]
    fn test_parse_number_above() {
        assert_eq!(
            parse_ocr_result("20\nDefense"),
            Some(("defense".to_string(), 20))
        );
    }

    #[test]
    fn test_parse_with_extra_text() {
        assert_eq!(
            parse_ocr_result("Defense +20% Bonus"),
            Some(("defense".to_string(), 20))
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
