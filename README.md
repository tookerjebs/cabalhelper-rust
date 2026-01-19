# Cabal Helper - Rust Edition

Automation tools for Cabal Online, rewritten in Rust for performance and stability.

## Features

- **Collection Filler**: Automates collection completion via red-dot detection.
- **Image Clicker**: Finds an image on screen and clicks it on an interval.
- **Custom Macros**: Build action sequences (click/type/delay/OCR search).
- **Overlay Bar**: Always-on-top quick controls while the game is running.

## Requirements

- Windows 10/11

## Usage

1. Download the latest release from https://github.com/tookerjebs/cabalhelper-rust/releases
2. Put `image.png` and `red-dot.png` next to the executable (or set custom paths in the UI).
3. Run `cabalhelper-rust.exe` and connect to the game window.

## Notes

- Settings are saved locally in `cabalhelper_settings.json`.
- OCR actions require a visible game window for capture.

## Build

```bash
cargo build --release
```

## License

[MIT](LICENSE)
