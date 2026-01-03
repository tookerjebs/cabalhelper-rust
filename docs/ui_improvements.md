# UI Polish & Overlay Improvements

## 1. Fix Overlay Position (Windowed Mode)
Currently, the overlay uses `get_window_rect` + padding, which puts it over the title bar in Windowed mode.

**The Solution (~5 LOC)**:
Use the new `get_client_rect_in_screen_coords` function.
```rust
// Replace existing positioning logic in app.rs
if let Some((x, y, w, _h)) = crate::core::window::get_client_rect_in_screen_coords(game_hwnd) {
    let overlay_w = 200.0;
    // Center logic remains the same
    let target_x = x as f32 + (w as f32 / 2.0) - (overlay_w / 2.0);
    // Anchor to TOP of CLIENT AREA (content), not window title
    let target_y = y as f32; 
    
    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition([target_x, target_y].into()));
}
```

## 2. UI Improvement Options

### Option A: Invisible Background (Transparent)
Make the overlay background fully transparent so buttons appear to float directly on the game.
- **Complexity**: Extremely Low (**2 LOC**)
- **Code**:
  ```rust
  // In app.rs, change the background fill:
  panel = panel.frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT)); // Fully invisible
  // OR strictly for the dock container:
  ui.painter().rect_filled(rect, rounding, Color32::from_black_alpha(0));
  ```

### Option B: The "Dynamic Dock"
A small "pill" that expands when you hover over it.
- **Behavior**: 
    - **Idle**: a small 30x10px semi-transparent handle at the top.
    - **Hover**: Expands to full tool list.
- **Complexity**: Medium (**~20-30 LOC**)
- **Implementation**:
  1. Add `is_dock_expanded: bool` to `CabalHelperApp`.
  2. In `update()`:
     ```rust
     let mouse_pos = ctx.input(|i| i.pointer.interact_pos());
     if let Some(pos) = mouse_pos {
         // Auto-expand if mouse is close
         self.is_dock_expanded = pos.distance(dock_rect.center()) < 100.0;
     } else {
        self.is_dock_expanded = false; 
     }
     ```
  3. Animate the size using `ctx.animate_bool_ease_out()`.

### Option C: Better Icons (Unicode vs Images)
Currently using text "1", "2", "3".
- **Sub-Option 1: Unicode (Simple)** (**~3 LOC**)
  - Replace text with: "⚔️" (Heil), "🎒" (Collection), "👁️" (Vision/Image).
  - *Pros*: Zero external dependencies.
- **Sub-Option 2: SVG Icons (Premium)** (**~50 LOC + Dependencies**)
  - Requires adding `egui_extras` and `image` crates.
  - Loading `.svg` or `.png` assets for each tool.

## Recommendation
1.  **Apply Fix #1** immediately (Overlay Position).
2.  ** Adopt Option A** (Transparent Background) for immediate "slickness".
3.  ** Adopt Option C1** (Unicode Icons) to replace numbers.
4.  ** Wait on Option B** (Dynamic Dock) until you are happy with the static look.
