# Cabal Helper - Rust Edition

An Application offering quality of life macros for Cabal Online. Offers 2 hard coded macros (Collection filler and image clicker) and the ability to create your own macros including OCR search capabilities - meaning you can use it to automate the Myth Level, Stellar Link, Arrival Skill rolling. Or just simply use it for trivial, repetitive movements (e.g. collecting your e-mails, buying multiple positions from agent shop) Use it in ways that are compliant with the terms of service of the server you are playing on.

## Features

- **Collection Filler**: Automates collection completion via red-dot detection.
- **Image Clicker**: Finds an image on screen and clicks it on an interval.
- **Custom Macros**: Build action sequences (click/type/delay/OCR search).
- **Overlay Bar**: Always-on-top quick controls while the game is running.

## Requirements

- Windows 10/11

## Usage

1. Download the latest release from https://github.com/tookerjebs/cabalhelper-rust/releases (coming soon)
2. Put `image.png` and `red-dot.png` next to the executable (or screenshot those yourself and set custom paths in the UI).
3. Run `cabalhelper-rust.exe` and connect to the game window.

## Notes

- Settings are saved automatically and locally in `cabalhelper_settings.json`.
- OCR actions require a visible game window for capture.

## Build

```bash
cargo build --release
```

## License

[MIT](LICENSE)
