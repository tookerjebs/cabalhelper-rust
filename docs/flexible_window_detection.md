# Flexible Game Window Detection

## Current Problem
The current `find_game_window()` function is too strict:
- Hardcoded to "PlayCabal EP36" title
- Fails for other regions (e.g., "PlayCabal EP25", "PlayCabal EU", etc.)
- Doesn't show the actual window title to the user

## Proposed Solution: Smart Detection

### Strategy
1. **Primary**: Search by class name `"D3D Window"` (common across all Cabal versions)
2. **Secondary**: Partial title matching for any window containing "cabal" (case-insensitive)
3. **Display**: Show the actual window title in the UI

### Implementation (~30 lines of code)

```rust
/// Find any Cabal game window using flexible matching
pub fn find_game_window() -> Option<(HWND, String)> {
    unsafe {
        // 1. Try class name first (most reliable)
        let hwnd_by_class = FindWindowA(
            windows::core::PCSTR("D3D Window\0".as_ptr()),
            windows::core::PCSTR::null(),
        );
        
        if hwnd_by_class.0 != 0 && IsWindow(hwnd_by_class).as_bool() {
            let title = get_window_title(hwnd_by_class);
            return Some((hwnd_by_class, title));
        }
        
        // 2. Fallback: Enumerate all windows and find by partial title
        find_window_by_partial_title("cabal")
    }
}

/// Get window title (helper function)
fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let mut buffer = vec![0u8; 256];
        let len = GetWindowTextA(hwnd, &mut buffer);
        
        if len > 0 {
            String::from_utf8_lossy(&buffer[..len as usize]).to_string()
        } else {
            "Unknown Window".to_string()
        }
    }
}

/// Find window by partial title match (case-insensitive)
fn find_window_by_partial_title(partial: &str) -> Option<(HWND, String)> {
    unsafe {
        let mut result: Option<(HWND, String)> = None;
        let partial_lower = partial.to_lowercase();
        
        // Callback for EnumWindows
        extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            unsafe {
                let result_ptr = lparam.0 as *mut Option<(HWND, String, String)>;
                let (_, partial_lower, _) = &*result_ptr;
                
                let title = get_window_title(hwnd);
                if title.to_lowercase().contains(partial_lower) {
                    *result_ptr = Some((hwnd, title, partial_lower.clone()));
                    return BOOL(0); // Stop enumeration
                }
                BOOL(1) // Continue
            }
        }
        
        let mut context = (None, partial_lower, String::new());
        EnumWindows(Some(enum_callback), LPARAM(&mut context as *mut _ as isize));
        
        context.0.map(|(hwnd, title, _)| (hwnd, title))
    }
}
```

### Required Changes

**1. Update `src/core/window.rs`:**
- Change `find_game_window()` signature to return `Option<(HWND, String)>`
- Add `GetWindowTextA` and `EnumWindows` imports
- Implement the flexible search logic

**2. Update `src/ui/app_header.rs`:**
```rust
// Before:
if let Some(hwnd) = find_game_window() {
    *game_hwnd = Some(hwnd);
    *game_title = "PlayCabal EP36".to_string();
}

// After:
if let Some((hwnd, title)) = find_game_window() {
    *game_hwnd = Some(hwnd);
    *game_title = title; // Display actual window title
}
```

**3. Update `Cargo.toml` (if needed):**
Add `EnumWindows` to Windows features (already included in `Win32_UI_WindowsAndMessaging`)

## Benefits
✅ Works with any Cabal version (EP25, EP36, EU, etc.)
✅ Works with different server names
✅ Shows actual window title to user
✅ Still prioritizes "D3D Window" class for reliability
✅ Fallback to partial title search if class fails

## Complexity
- **Lines of Code**: ~30-40 lines
- **Risk**: Low (backwards compatible, just more flexible)
- **Testing**: Easy to verify with different game versions
