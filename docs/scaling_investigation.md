# Investigation: DPI Scaling & Robust Coordinates

## 1. The Problem: DPI Scaling ("Zoom")
When a user sets Windows to 125% or 150% scaling:
- **Default Behavior**: Windows "virtualizes" the screen for your app. If the screen is 1920 pixels wide, your app is told it is only 1280 pixels wide (at 150%).
- **Result**: When you call `GetWindowRect`, you get "logical" (fake) coordinates. When you take a screenshot, you might get a blurry, scaled version.
- **Fix**: You must tell Windows "I am DPI Aware" so it gives you the raw, physical pixel coordinates.

## 2. The Problem: Window Borders (Windowed vs Windowless)
Currently, `src/core/window.rs` uses `GetWindowRect`. 
- `GetWindowRect` returns the position of the **outer edge** of the window (including title bar and borders).
- If a user plays in Windowed mode, the "Game Content" starts roughly `8px` to the right and `30px` down from the `WindowRect`.
- If they switch to Borderless/Fullscreen, the "Game Content" starts exactly at `WindowRect`.
- **Consequence**: If you calibrate in Windowed mode and run in Borderless (or vice versa), all your clicks will be shifted by the size of the borders (~30 pixels).

## 3. Recommended Solutions

### A. Force DPI Awareness
We need to explicitly request `PerMonitorV2` awareness at the very start of your application. This ensures `1 pixel` in your code equals `1 pixel` on the monitor, regardless of scaling settings.

**Changes required in `src/main.rs`**:
```rust
// Add this import
use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};

fn main() -> Result<(), eframe::Error> {
    // 1. Enable High DPI Awareness immediately
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    
    // ... rest of your code
}
```
*Note: This requires adding the `Win32_UI_HiDpi` feature to `Cargo.toml`.*

### B. Switch to "Client Area" Coordinates
Instead of anchoring clicks to the top-left of the *Window* (which moves depending on how thick the borders are), you should anchor them to the top-left of the *Client Area* (the actual black box where the game renders).

**Concept**:
- **Current**: `Click_X = Window_Left + relative_x`
- **Better**: `Click_X = Client_Top_Left_X + relative_x`

**How to implement in `src/core/window.rs`**:
You need a function that finds the screen position of the inner content area (Client Area).

```rust
pub fn get_client_rect_in_screen_coords(hwnd: HWND) -> Option<(i32, i32, i32, i32)> {
    unsafe {
        // 1. Get the size of the inner content area
        let mut client_rect = windows::Win32::Foundation::RECT::default();
        if !GetClientRect(hwnd, &mut client_rect).as_bool() {
            return None;
        }

        // 2. Convert (0,0) of client area to Screen Coordinates
        let mut top_left = POINT { x: 0, y: 0 };
        if !ClientToScreen(hwnd, &mut top_left).as_bool() {
            return None;
        }

        // 3. Convert bottom-right
        let mut bottom_right = POINT { x: client_rect.right, y: client_rect.bottom };
        ClientToScreen(hwnd, &mut bottom_right);

        Some((
            top_left.x,
            top_left.y,
            bottom_right.x - top_left.x, // Width
            bottom_right.y - top_left.y  // Height
        ))
    }
}
```

### C. Update Coordinate Handling
When you **Save** a calibration point:
1. Get the specific clicked screen coordinate (e.g., `100, 200`).
2. Get the `Client Area` top-left screen coordinate (e.g., `50, 50`).
3. Save `relative_x = 100 - 50 = 50`.

When you **Load/Run** automation:
1. Get `Client Area` top-left (e.g., still `50, 50`).
2. Target `x = 50 + 50 = 100`.

*Even if the user switches from Windowed to Borderless, the `Client Area` top-left might move on screen, but the relative distance from the inner content edge to the button remains identical.*

## 4. Implementation Plan

1.  **Update `Cargo.toml`**: Add `Win32_UI_HiDpi` feature to the `windows` crate.
2.  **Update `main.rs`**: Set DPI awareness context at startup.
3.  **Update `window.rs`**: Add `get_client_rect_in_screen_coords`.
4.  **Refactor `AutomationContext`**: Use the new client rect function instead of `get_window_rect` for anchoring.

This approach resolves both the scaling issue (by reading true pixels) and the window mode issue (by ignoring the variable-size borders).
