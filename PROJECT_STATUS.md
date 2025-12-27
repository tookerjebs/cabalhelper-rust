# Cabal Helper - Rust Edition

## Project Status Update

### ✅ Completed
1. **Restructured the project** for scalability and maintainability
2. **Implemented tab-based UI** using egui with 3 tabs:
   - Heil Clicker (fully functional)
   - Collection Filler (placeholder)
   - Image Clicker (fully functional with RustAutoGui)

3. **Integrated RustAutoGui** for computer vision and image detection
   - Replaced custom CV implementation with battle-tested library
   - Uses fast Segmented template matching algorithm
   - Cross-platform support (Windows/Linux/macOS)

### 📊 File Size Comparison
- **Before (Heil Clicker only)**: ~4.5 MB
- **After (with RustAutoGui CV)**: ~11 MB
- **Python Version**: ~120 MB

**Result: Still ~11x smaller than Python! 🚀**

### 🏗️ Project Structure
```
src/
├── main.rs              // Entry point
├── app.rs               // Main App with Tab navigation
├── core/                // Shared low-level utilities
│   ├── mod.rs
│   ├── window.rs        // Window detection & coordinate conversion
│   └── input.rs         // Mouse/keyboard input simulation
└── tools/               // Individual tool features
    ├── mod.rs
    ├── heil_clicker.rs  // Automated clicking tool
    └── image_clicker.rs // CV-based image detection & clicking
```

### 🎯 Image Clicker Features
- Load any template image (PNG, JPEG, etc.)
- Configurable search interval
- Adjustable tolerance (precision)
- Automatic click on image detection
- Uses RustAutoGui's Segmented matching (fast!)

### 🔧 Usage
1. Place a template image in the project root as `image.png`
2. Run the application
3. Navigate to "Image Clicker" tab
4. Adjust interval and tolerance as needed
5. Click "Start"

The tool will continuously:
- Search for the template image on screen
- Move the mouse to the center of the match
- Click the left mouse button
- Wait for the specified interval
- Repeat

### 📝 Next Steps
Ready to port the **Collection Filler** - the most complex tool from the Python version!

### 💡 Notes
- RustAutoGui uses FFT and Segmented template matching
- Much faster than Python's PyAutoGUI + OpenCV
- No complex OpenCV dependencies needed
- Works out of the box on Windows
