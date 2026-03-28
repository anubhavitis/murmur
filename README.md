<p align="center">
  <img src="assets/logo_dark.png" width="120" alt="Murmur logo">
</p>

<h1 align="center">Murmur</h1>

<p align="center">
  <strong>Local speech-to-text from your menubar.</strong><br>
  Hold a hotkey to record, release to transcribe. Fully offline.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-blue" alt="macOS">
  <img src="https://img.shields.io/badge/language-Rust-orange" alt="Rust">
  <img src="https://img.shields.io/badge/powered_by-whisper.cpp-green" alt="whisper.cpp">
  <img src="https://img.shields.io/badge/license-MIT-lightgrey" alt="MIT License">
</p>

---

## How it works

1. **Hold** Right Option key
2. **Speak**
3. **Release** — text appears in your clipboard (or pastes at cursor)

That's it. No cloud. No API keys. Everything runs locally on your machine via [whisper.cpp](https://github.com/ggerganov/whisper.cpp).

---

## Features

| Feature | Description |
|---|---|
| **Menubar app** | Lives in your system tray, zero windows |
| **Hold-to-record** | Right Option or Caps Lock as trigger |
| **Fully offline** | whisper.cpp runs locally, no data leaves your machine |
| **Auto-download** | Grabs the `tiny.en` model on first launch |
| **Clipboard or paste** | Copy to clipboard, or paste directly at cursor |
| **Model selection** | English and multilingual variants: tiny through large |
| **Live progress** | Download progress updates in-place in the menu |
| **Animated icon** | Tray icon spins while recording |

---

## Models

| Model | English | Multilingual | Size |
|---|---|---|---|
| tiny | tiny.en | tiny | 74 MB |
| base | base.en | base | 141 MB |
| small | small.en | small | 465 MB |
| medium | medium.en | medium | 1.4 GB |
| large | — | large-v3 | 2.9 GB |

Models are downloaded from [HuggingFace](https://huggingface.co/ggerganov/whisper.cpp) and stored at `~/.murmur/models/`.

---

## Getting started

### Requirements

- Rust (edition 2024)
- cmake — `brew install cmake`
- macOS permissions: **Input Monitoring** + **Microphone**

### Build & run

```sh
git clone https://github.com/anubhavitis/murmur.git
cd murmur
cargo build --release
cargo run --release
```

On first launch, Murmur downloads the `tiny.en` model (~74 MB) automatically.

---

## Configuration

Config lives at `~/.murmur/config.json`:

```json
{
  "selected_model": "tiny.en",
  "output_mode": "paste_at_cursor",
  "hotkey": "right_alt"
}
```

| Setting | Options |
|---|---|
| `output_mode` | `clipboard`, `paste_at_cursor` |
| `hotkey` | `right_alt`, `caps_lock` |
| `selected_model` | Any model from the table above |

---

## Architecture

```
src/
  main.rs         Event loop, wires everything together
  app.rs          State, events, menu commands
  config.rs       Config persistence (~/.murmur/config.json)
  tray.rs         Menubar tray icon, menu, animation
  hotkey.rs       Global hotkey listener (rdev)
  audio.rs        Audio capture + 16kHz resampling (cpal)
  transcriber.rs  Whisper inference on background thread
  downloader.rs   Model download from HuggingFace
  platform/       OS-specific code (paste modifier key)
```

**Design principles:** single binary, no async runtime, sync threads + `EventLoopProxy` for all communication, platform-specific code isolated in `src/platform/`.

---

## Roadmap

- [ ] Live transcription (streaming partial results while recording)
- [ ] Windows support (code is cross-platform ready, untested)
- [ ] Distributable `.app` bundle with signing
- [ ] Language selection
- [ ] CoreML backend for Apple Silicon

---

## License

MIT
