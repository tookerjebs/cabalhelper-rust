# Rust Migration Plan - Stellar Automation Tool

## Project Overview
Migrating from Python to Rust to create a fast, lightweight automation tool for game automation. Starting minimal and building incrementally.

**Long-term Goals:**
- Fast, performant automation tool
- Overlay mode with icon-based UI (floating on top of game window)
- Multiple QOL tabs/tools (Heils Clicker, Collection Filler, and more)
- Single lightweight executable (~10-15MB)

**Development Philosophy:**
- **INCREMENTAL**: Add code in small, testable chunks (50-200 lines per step)
- **TEST EARLY**: Verify each feature works before moving to next
- **LEARN GRADUALLY**: Understand Rust concepts as we build
- **NO BIG BANG**: Never add 1000+ lines in one go

**Reference Files:**
- Python implementation files are in `REFERENCE/` folder
- **AI should reference these** when implementing Rust equivalents to understand:
  - Exact behavior and edge cases
  - Algorithm flow and logic
  - Window API usage patterns
  - Settings structure
- See `REFERENCE/README.md` for details

---

## Phase 0: Project Setup (START HERE)

### Step 0.1: Initialize Rust Project
**Goal:** Get basic Rust project structure working
**Estimated Lines:** ~20 lines
**What to create:**
- `Cargo.toml` with basic dependencies
- `src/main.rs` with "Hello World"
- Verify `cargo run` works

**Dependencies to add:**
```toml
[dependencies]
# Windows API bindings
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Graphics_Gdi",
] }
```

**Success Criteria:**
- ✅ `cargo run` compiles and prints "Hello World"
- ✅ No errors in Cursor IDE

---

## Phase 1: Basic Clicking (MINIMAL START)

### Step 1.1: Test Window Finding
**Goal:** Find game window by title
**Estimated Lines:** ~50-80 lines
**What to add:**
- Function to enumerate windows
- Function to find window by title (e.g., "Cabal")
- Print window handle and position

**Success Criteria:**
- ✅ Can find game window when it's open
- ✅ Prints window info to console

**Testing:**
- Run program with game window open
- Verify it finds the window

---

### Step 1.2: Basic Click at Coordinates
**Goal:** Click at a hardcoded position in game window
**Estimated Lines:** ~80-120 lines
**What to add:**
- Function to click at window-relative coordinates
- Use `SendMessage` or `PostMessage` for clicking
- Hardcode coordinates (e.g., x=100, y=200)
- Add small delay between clicks

**Success Criteria:**
- ✅ Clicks at hardcoded position
- ✅ Works even when game window is minimized
- ✅ Can click repeatedly with delay

**Testing:**
- Set game window to specific position
- Run program, verify clicks happen
- Test with window minimized

---

### Step 1.3: Click Loop with Configurable Delay
**Goal:** Continuous clicking with adjustable delay
**Estimated Lines:** ~100-150 lines
**What to add:**
- Thread for click loop
- Configurable delay (milliseconds)
- Start/stop mechanism
- Print status messages

**Success Criteria:**
- ✅ Can start/stop clicking loop
- ✅ Delay is configurable
- ✅ Can click continuously until stopped

**Testing:**
- Start clicker, verify continuous clicks
- Change delay, verify timing changes
- Stop clicker, verify it stops

---

## Phase 2: Simple UI (Command Line First)

### Step 2.1: Command Line Interface
**Goal:** Control clicker via command line
**Estimated Lines:** ~80-120 lines
**What to add:**
- CLI argument parsing (use `clap` crate)
- Commands: `start`, `stop`, `set-delay <ms>`, `set-position <x> <y>`
- Interactive mode or one-shot commands

**Dependencies:**
```toml
clap = { version = "4.4", features = ["derive"] }
```

**Success Criteria:**
- ✅ Can start clicker from command line
- ✅ Can set position and delay via CLI
- ✅ Can stop clicker via CLI

**Testing:**
- Test all CLI commands
- Verify clicker responds to commands

---

### Step 2.2: Settings File (JSON)
**Goal:** Save/load click position and delay
**Estimated Lines:** ~100-150 lines
**What to add:**
- Read/write `settings.json`
- Store: click_position (x, y), delay_ms
- Load settings on startup
- Save settings when changed

**Dependencies:**
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Success Criteria:**
- ✅ Settings persist between runs
- ✅ Can modify settings file manually
- ✅ Program loads settings on startup

**Testing:**
- Set position, close program, reopen
- Verify settings are loaded

---

## Phase 3: GUI (Simple Window)

### Step 3.1: Basic Window with egui
**Goal:** Create simple window with egui
**Estimated Lines:** ~100-150 lines
**What to add:**
- Window using `eframe` (egui framework)
- Basic UI: title, status label
- Window that can be moved/resized

**Dependencies:**
```toml
eframe = "0.24"
egui = "0.24"
```

**Success Criteria:**
- ✅ Window opens and displays
- ✅ Can move and resize window
- ✅ No crashes

**Testing:**
- Run program, verify window appears
- Test window interactions

---

### Step 3.2: Clicker Controls in GUI
**Goal:** Add start/stop buttons and delay input
**Estimated Lines:** ~150-200 lines
**What to add:**
- Start/Stop button
- Delay input field (number)
- Position display (x, y)
- Status label (running/stopped)

**Success Criteria:**
- ✅ Can start/stop from GUI
- ✅ Can change delay from GUI
- ✅ Status updates correctly

**Testing:**
- Click start, verify clicker runs
- Change delay, verify it updates
- Click stop, verify it stops

---

### Step 3.3: Position Picker
**Goal:** Click to set click position
**Estimated Lines:** ~150-200 lines
**What to add:**
- "Set Position" button
- On click, wait for mouse click on game window
- Capture click coordinates
- Display coordinates in UI
- Save to settings

**Success Criteria:**
- ✅ Can click "Set Position"
- ✅ Can click in game window to set position
- ✅ Coordinates are saved and displayed

**Testing:**
- Set position, verify coordinates update
- Restart program, verify position is loaded

---

## Phase 4: Heils Clicker Complete

### Step 4.1: Polish and Error Handling
**Goal:** Add proper error handling and edge cases
**Estimated Lines:** ~100-150 lines
**What to add:**
- Error handling for window not found
- Error handling for click failures
- Validation (delay > 0, position set)
- User-friendly error messages

**Success Criteria:**
- ✅ Handles errors gracefully
- ✅ Shows clear error messages
- ✅ Doesn't crash on edge cases

**Testing:**
- Test with game window closed
- Test with invalid settings
- Test rapid start/stop

---

### Step 4.2: Build Release Executable
**Goal:** Create distributable exe
**Estimated Lines:** ~10 lines (just commands)
**What to do:**
- Run `cargo build --release`
- Test the exe
- Verify it's standalone (no DLLs needed)
- Check file size

**Success Criteria:**
- ✅ Exe runs without Rust installed
- ✅ Exe is ~5-10MB
- ✅ All functionality works

**Testing:**
- Copy exe to different folder
- Run on clean system (if possible)
- Verify all features work

---

## Phase 5: Collection Filler - Foundation

### Step 5.1: Screenshot Capture
**Goal:** Capture area of game window
**Estimated Lines:** ~150-200 lines
**What to add:**
- Function to capture window area using BitBlt
- Convert to image format (use `image` crate)
- Test with hardcoded area coordinates
- Save screenshot to file for verification

**Dependencies:**
```toml
image = "0.24"
```

**Success Criteria:**
- ✅ Can capture specific area of game window
- ✅ Screenshot is saved correctly
- ✅ Works with minimized window

**Testing:**
- Capture area, verify image file
- Test with different areas
- Test with window minimized

---

### Step 5.2: Load Template Image
**Goal:** Load red-dot.png template
**Estimated Lines:** ~50-80 lines
**What to add:**
- Load template image from file
- Handle file not found errors
- Cache template in memory
- Print template dimensions

**Success Criteria:**
- ✅ Loads template image
- ✅ Handles missing file gracefully
- ✅ Template is cached

**Testing:**
- Test with template present
- Test with template missing
- Verify template loads correctly

---

### Step 5.3: OpenCV Template Matching (Basic)
**Goal:** Find template in screenshot
**Estimated Lines:** ~150-200 lines
**What to add:**
- Add OpenCV dependency
- Basic template matching
- Find first match only
- Print match location and confidence
- Test with hardcoded screenshot area

**Dependencies:**
```toml
opencv = { version = "0.86", default-features = false, features = ["imgproc", "imgcodecs"] }
```

**Success Criteria:**
- ✅ Finds template in screenshot
- ✅ Returns match coordinates
- ✅ Confidence threshold works

**Testing:**
- Capture area with red dot
- Run matching, verify it finds dot
- Test with area without red dot

---

### Step 5.4: Find All Matches
**Goal:** Find all red dots in area
**Estimated Lines:** ~100-150 lines
**What to add:**
- Find all matches above confidence threshold
- Filter duplicate matches (close together)
- Return list of coordinates
- Test with multiple red dots

**Success Criteria:**
- ✅ Finds all red dots
- ✅ Filters duplicates correctly
- ✅ Returns correct coordinates

**Testing:**
- Test with 1 red dot
- Test with multiple red dots
- Test with overlapping matches

---

## Phase 6: Collection Filler - Logic

### Step 6.1: Collection Tab Detection
**Goal:** Find red dots in collection tabs area
**Estimated Lines:** ~100-150 lines
**What to add:**
- Define collection tabs area (hardcoded for now)
- Scan for red dots in that area
- Click first red dot found
- Print which tab was clicked

**Success Criteria:**
- ✅ Finds red dots in tabs area
- ✅ Clicks first red dot
- ✅ Works correctly

**Testing:**
- Test with red dot in tabs
- Test with no red dots
- Verify click happens

---

### Step 6.2: Dungeon List Processing
**Goal:** Process dungeons with red dots
**Estimated Lines:** ~150-200 lines
**What to add:**
- Define dungeon list area
- Find red dots in dungeon list
- Click dungeon with red dot
- Process items for that dungeon
- Navigate to next dungeon

**Success Criteria:**
- ✅ Finds red dots in dungeon list
- ✅ Clicks dungeons correctly
- ✅ Processes multiple dungeons

**Testing:**
- Test with multiple dungeons
- Verify clicking order
- Test with no red dots

---

### Step 6.3: Item Processing
**Goal:** Process collection items
**Estimated Lines:** ~200-250 lines
**What to add:**
- Define collection items area
- Find red dots in items area
- Click item with red dot
- Execute button sequence (Auto Refill -> Register -> Yes)
- Scroll and continue processing

**Success Criteria:**
- ✅ Finds items with red dots
- ✅ Executes button sequence
- ✅ Scrolls and continues

**Testing:**
- Test with items to process
- Verify button sequence
- Test scrolling

---

### Step 6.4: Full Collection Filler Loop
**Goal:** Complete automation loop
**Estimated Lines:** ~200-300 lines
**What to add:**
- Main loop: tabs -> dungeons -> items
- Page navigation (page 2, 3, 4, arrow)
- Scroll handling
- Stop conditions
- Error handling

**Success Criteria:**
- ✅ Runs full automation cycle
- ✅ Handles all pages
- ✅ Stops when complete
- ✅ Handles errors

**Testing:**
- Run full automation
- Test with various scenarios
- Verify it completes correctly

---

## Phase 7: Collection Filler - UI

### Step 7.1: Collection Filler Tab in GUI
**Goal:** Add UI for collection filler
**Estimated Lines:** ~200-250 lines
**What to add:**
- New tab in egui for Collection Filler
- Area selection buttons (tabs, dungeon list, items)
- Button coordinate setup
- Start/Stop button
- Status display

**Success Criteria:**
- ✅ New tab appears
- ✅ Can set all areas
- ✅ Can start/stop automation

**Testing:**
- Test all UI elements
- Verify settings save
- Test automation start/stop

---

### Step 7.2: Area Selector
**Goal:** Visual area selection
**Estimated Lines:** ~200-250 lines
**What to add:**
- Click to select area
- Draw rectangle overlay
- Save area coordinates
- Preview selected areas

**Success Criteria:**
- ✅ Can select areas visually
- ✅ Areas are saved
- ✅ Preview works

**Testing:**
- Select all required areas
- Verify coordinates saved
- Test area preview

---

## Phase 8: Polish & Optimization

### Step 8.1: Settings Management
**Goal:** Unified settings system
**Estimated Lines:** ~150-200 lines
**What to add:**
- Settings struct for all tools
- Save/load all settings
- Settings validation
- Default values

**Success Criteria:**
- ✅ All settings persist
- ✅ Settings are validated
- ✅ Defaults work

---

### Step 8.2: Performance Optimization
**Goal:** Optimize for speed
**Estimated Lines:** ~100-150 lines
**What to add:**
- Profile code
- Optimize hot paths
- Reduce allocations
- Cache templates/images

**Success Criteria:**
- ✅ Faster execution
- ✅ Lower memory usage
- ✅ Smooth UI

---

### Step 8.3: Error Handling & Logging
**Goal:** Robust error handling
**Estimated Lines:** ~150-200 lines
**What to add:**
- Comprehensive error types
- Logging system
- Error recovery
- User-friendly messages

**Dependencies:**
```toml
log = "0.4"
env_logger = "0.10"
anyhow = "1.0"
thiserror = "1.0"
```

**Success Criteria:**
- ✅ All errors handled
- ✅ Logs are useful
- ✅ User sees clear messages

---

## Phase 9: Future Features (Long-term)

### Overlay Mode
- Always-on-top window
- Icon-based UI
- Click icon to start tool
- Minimal footprint

### Additional Tools
- Stellar System automation
- Arrival Skill automation
- Other QOL tools

### Advanced Features
- Hotkeys
- Profiles
- Statistics
- Auto-update

---

## Development Guidelines for AI

### Code Addition Rules:
1. **MAX 200 lines per step** - Break into smaller steps if needed
2. **Test after each step** - Verify functionality before moving on
3. **Explain changes** - Comment what each addition does
4. **Incremental builds** - Each step should compile and run
5. **No skipping** - Don't jump ahead, follow the plan
6. **Reference Python code** - Check `REFERENCE/` folder for implementation details when needed
   - Window finding: See `game_connector.py`
   - Clicking: See `clicking/sendmessage_clicker.py`
   - Heils clicker: See `heils_clicker.py`
   - Collection filler: See `collection_automation.py`
   - Settings: See `settings.json`

### When Adding Code:
- Start with smallest working version
- Add one feature at a time
- Test immediately after adding
- Fix errors before continuing
- Document what was added

### Error Handling:
- Handle errors gracefully
- Show user-friendly messages
- Don't crash on edge cases
- Log errors for debugging

### Testing Checklist (After Each Step):
- [ ] Code compiles without errors
- [ ] Program runs without crashing
- [ ] New feature works as expected
- [ ] No regressions in existing features
- [ ] Error cases handled

---

## Project Structure (Will Build Over Time)

```
stellar-automation-rust/
├── Cargo.toml                 # Dependencies and project config
├── Cargo.lock                 # Locked dependency versions (auto-generated)
├── settings.json              # User settings (created at runtime)
├── red-dot.png                # Template image for collection filler
├── src/
│   ├── main.rs                # Entry point
│   ├── window.rs              # Window finding/handling
│   ├── clicking.rs            # Click functionality
│   ├── screenshot.rs          # Screenshot capture
│   ├── template_matching.rs  # OpenCV template matching
│   ├── collection.rs          # Collection filler logic
│   ├── ui.rs                  # GUI code
│   └── settings.rs            # Settings management
└── target/                    # Build output (auto-generated)
    └── release/
        └── stellar-automation.exe
```

**Note:** Files will be created incrementally as we progress through phases.

---

## Getting Started

1. **Install Rust:**
   ```powershell
   # Download and run rustup-init.exe from https://rustup.rs/
   # Or use: winget install Rustlang.Rustup
   ```

2. **Verify Installation:**
   ```powershell
   rustc --version
   cargo --version
   ```

3. **Create New Project:**
   ```powershell
   cargo new stellar-automation-rust
   cd stellar-automation-rust
   ```

4. **Start with Phase 0, Step 0.1**
   - Create basic `Cargo.toml`
   - Create minimal `src/main.rs`
   - Run `cargo run` to verify

5. **Follow Plan Incrementally**
   - Complete one step at a time
   - Test after each step
   - Don't skip ahead

---

## Notes

- **This plan is a living document** - Can be updated as we learn
- **Start minimal** - Phase 0 and 1 are the absolute minimum
- **Build gradually** - Each phase builds on previous
- **Test frequently** - Verify each step works
- **Ask questions** - If something is unclear, ask before coding

**Remember:** The goal is to learn Rust and build a working tool, not to rush to completion. Take time to understand each step.
