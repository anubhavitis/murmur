# Murmur

Local speech-to-text from your menubar. Hold a hotkey to record, release to transcribe. Fully offline — powered by [whisper.cpp](https://github.com/ggerganov/whisper.cpp).

## Features

- **Menubar app** — lives in your system tray, no windows
- **Hold-to-record** — hold Right Option (or Caps Lock) to record, release to transcribe
- **Offline** — runs whisper.cpp locally, no cloud services
- **Auto-download** — downloads the `tiny` model on first launch
- **Clipboard or paste** — copies transcription to clipboard, or pastes directly at cursor
- **Model selection** — switch between tiny/base/small/medium/large from the menu
- **Cross-platform** — macOS today, Windows support planned

## Requirements

- Rust (edition 2024)
- cmake (`brew install cmake` on macOS)
- macOS: Input Monitoring + Microphone permissions

## Build & Run

```sh
cargo build --release
cargo run
```

## Configuration

Config stored at `~/.murmur/config.json`:

```json
{
  "selected_model": "tiny",
  "output_mode": "paste_at_cursor",
  "hotkey": "right_alt"
}
```

**Output modes:** `clipboard`, `paste_at_cursor`
**Hotkeys:** `right_alt`, `caps_lock`
**Models:** `tiny`, `base`, `small`, `medium`, `large`

Models are downloaded from HuggingFace and stored at `~/.murmur/models/`.

## Architecture

```
src/
  main.rs         — event loop, wires everything together
  app.rs          — state, events, menu commands
  config.rs       — config persistence (~/.murmur/config.json)
  tray.rs         — menubar tray icon and menu
  hotkey.rs       — global hotkey listener (rdev)
  audio.rs        — audio capture + 16kHz resampling (cpal)
  transcriber.rs  — whisper inference on background thread
  downloader.rs   — model download from HuggingFace
  platform/       — OS-specific code (paste modifier key)
```

## License

MIT
