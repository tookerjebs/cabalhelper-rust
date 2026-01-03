# Performance & Dock Analysis

## 1. The "Idle" CPU Mystery (7-8% Usage)
You noticed:
- **Normal Mode**: ~0% CPU.
- **Overlay (Disconnected)**: ~0% CPU.
- **Overlay (Connected)**: ~7-8% CPU.

**The Cause**:
In `app.rs`, when `is_overlay_mode` and `game_hwnd` are active, we run this logic **10 times per second** (every 100ms):
1.  Ask Windows for the Game Window Position (`get_client_rect`).
2.  Ask Windows to Move the Overlay Window (`send_viewport_cmd`).
3.  **Crucially**: `egui` re-renders the entire UI frame every time we call `request_repaint`.

Even if the game window hasn't moved, we are forcing a layout calculation and GPU draw call 10 times a second.

## 2. Dynamic Dock: Performance Impact
**Would a Dock improve or worsen performance?**

*   **collapsed State (Idle)**: IMPROVES performance.
    *   Since it's just a tiny "handle" or "pill", drawing it is cheaper.
    *   We can reduce the poll rate to 1-2 FPS because a small handle doesn't need to snap as perfectly as a large UI.
*   **Expanding State (Hover)**: TEMPORARILY worsens.
    *   Animations require 60 FPS for ~0.2 seconds to look smooth.
*   **Overall Verdict**: **Neutral to Better**. It won't magically fix the 8% usage unless we change *how* we track the window.

## 3. Top Optimization Options (High Impact)

### Option A: "Lazy Snapping" (Recommended)
Instead of snapping every 100ms, strictly follow this rule:
- **Poll Game Position**: Every **1.0 second** (instead of 0.1s).
- **Trigger Snap**: ONLY if the game window moved > 5 pixels.
- **Result**: CPU usage should drop to **< 1%** instantly. The only downside is a 1-second delay for the overlay to "catch up" if you drag the game window.

### Option B: "Hover-Only" Snapping
Only update the overlay position when the mouse is **hovering** over the overlay.
- Be logic: "If I'm not using it, I don't care if it's perfectly aligned."
- **Result**: 0% idle CPU.

### Option C: Native Windows "Child Window" (Advanced)
Instead of a separate Overlay Window that we manually move, we can tell Windows API to make our Overlay a **Child** of the Game Window.
- **Pros**: Windows OS handles the movement for free (0% CPU). Perfect sync.
- **Cons**: High complexity. Might trigger anti-cheat protections (injecting windows into game process space). **Not recommended for this project.**

## 4. Dock Implementation Recommendation
If we go with the Dock, here is the "Slick + Low CPU" blueprint:

1.  **State**: 3 states - `Hidden` (Game not focused), `Collapsed` (Small pill), `Expanded` (Full tools).
2.  **Logic**:
    - If `Game Active`: Show `Collapsed`. Update position every **500ms**.
    - If `Mouse Hover`: Switch to `Expanded`. Update position every **16ms** (smooth).
    - If `Mouse Leave`: Return to `Collapsed`.

## Summary of Top Suggestions
1.  **Optimize the Snap Loop**: Change the `100ms` repaint to `500ms` or `1000ms` immediately. This is a 1-line code change that fixes the 8% CPU usage.
2.  **Implement the Dock**: It feels modern and keeps the screen clean.
3.  **Use "Smart" Repaint**: Only call `request_repaint` if data actually changed, rather than blindly every frame.
