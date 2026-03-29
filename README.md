<p align="center">
  <img src="assets/logo_dark.png" width="120" alt="Murmur logo">
</p>

<h1 align="center">Murmur</h1>

<p align="center">
  <strong>Local speech-to-text from your menubar.</strong><br>
  Hold a hotkey to record, release to transcribe. Fully offline.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS_(Apple_Silicon)-blue" alt="macOS">
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

## Install

### Homebrew (recommended)

```sh
brew tap anubhavitis/murmur
brew install --cask murmur
```

### Script

```sh
curl -sSL https://raw.githubusercontent.com/anubhavitis/murmur/main/install.sh | sh
```

Murmur auto-starts on login and restarts on crash. On first launch, it downloads the `tiny.en` model (~74 MB).

### Permissions

After install, grant these in **System Settings > Privacy & Security**:

| Permission | Why |
|---|---|
| **Input Monitoring** | Hotkey detection (Right Option / Caps Lock) |
| **Microphone** | Audio capture for transcription |
| **Accessibility** | Paste-at-cursor (simulates Cmd+V) |

Look for `murmur` in each permission list and toggle it on.

### Uninstall

```sh
curl -sSL https://raw.githubusercontent.com/anubhavitis/murmur/main/uninstall.sh | sh
```

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
| **Language preferences** | Pick preferred languages, auto-detect among them |
| **Audio feedback** | Beep on record start/stop |
| **Animated icon** | Tray icon spins while recording and transcribing |
| **Auto-start** | Launches on login via Launch Agent |

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

## Configuration

Config lives at `~/.murmur/config.json`:

```json
{
  "selected_model": "tiny.en",
  "output_mode": "paste_at_cursor",
  "hotkey": "right_alt",
  "languages": ["en"]
}
```

| Setting | Options |
|---|---|
| `output_mode` | `clipboard`, `paste_at_cursor` |
| `hotkey` | `right_alt`, `caps_lock` |
| `selected_model` | Any model from the table above |
| `languages` | Array of [whisper language codes](https://github.com/openai/whisper/blob/main/whisper/tokenizer.py) (e.g. `["en", "hi"]`) |

---

## Build from source

<details>
<summary>For developers</summary>

### Requirements

| Requirement | macOS |
|---|---|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| cmake | `brew install cmake` |
| C compiler | Included with Xcode CLT (`xcode-select --install`) |

> whisper.cpp is compiled from source automatically during `cargo build`.

### Build & run

```sh
git clone https://github.com/anubhavitis/murmur.git
cd murmur
cargo build --release
cargo run --release
```

### Release

```sh
./release.sh
```

Builds the binary, creates a tar.gz, generates the Homebrew cask formula, and prints upload instructions.

</details>

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
  languages.rs    Whisper language registry (100 languages)
  platform/       OS-specific code (paste key, sounds)
```

---

## Roadmap

- [ ] Live transcription (streaming partial results while recording)
- [ ] Windows support (code is cross-platform ready, untested)
- [ ] CoreML backend for Apple Silicon

---

## License

MIT
