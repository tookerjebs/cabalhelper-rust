use rustautogui::RustAutoGui;
use std::time::Instant;

/// Find red dots (or any stored template) on screen using a pre-stored template
/// Returns a list of (x, y) positions in screen coordinates
pub fn find_stored_template(
    gui: &mut RustAutoGui,
    alias: &str,
    precision: f32
) -> Option<Vec<(u32, u32)>> {
    let start_time = Instant::now();
    
    match gui.find_stored_image_on_screen(precision, alias) {
        Ok(Some(matches)) => {
            let filtered: Vec<(u32, u32)> = matches.iter()
                .map(|(x, y, _score)| (*x, *y))
                .collect();
            
            if filtered.is_empty() {
                None
            } else {
                Some(filtered)
            }
        },
        Ok(None) => {
            None
        },
        Err(_) => {
            None
        }
    }
}

/// Check if a position is near another position (within threshold pixels)
pub fn is_position_near(pos1: (u32, u32), pos2: (u32, u32), threshold: f32) -> bool {
    let dist = ((pos1.0 as f32 - pos2.0 as f32).powi(2) +
               (pos1.1 as f32 - pos2.1 as f32).powi(2)).sqrt();
    dist <= threshold
}
