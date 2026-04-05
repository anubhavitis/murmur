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
  <img src="https://img.shields.io/badge/platform-Windows-blue" alt="Windows">
  <img src="https://img.shields.io/badge/language-Rust-orange" alt="Rust">
  <img src="https://img.shields.io/badge/powered_by-whisper.cpp_+_FluidAudio-green" alt="whisper.cpp + FluidAudio">
  <img src="https://img.shields.io/badge/license-MIT-lightgrey" alt="MIT License">
  <br>
  <img src="https://img.shields.io/badge/coming_soon-Live_Transcription-yellow" alt="Coming Soon: Live Transcription">
  <img src="https://img.shields.io/badge/coming_soon-Linux-yellow" alt="Coming Soon: Linux">
</p>

---

## How it works

1. **Hold** Right Option (macOS) or Right Alt (Windows)
2. **Speak**
3. **Release** — text appears in your clipboard (or pastes at cursor)

That's it. No cloud. No API keys. Everything runs locally on your machine via [whisper.cpp](https://github.com/ggerganov/whisper.cpp) or [FluidAudio](https://www.fluidaudio.net/) (Apple Neural Engine on Apple Silicon Macs).

---

## Install

### Windows

There is no Windows installer yet. Build from source on a Windows machine:

```powershell
git clone https://github.com/anubhavitis/murmur.git
cd murmur
cargo run --release
```

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

> **Note:** Murmur is not notarized yet (Apple charges $99/year and we're spending all our money on coffee instead). Because of this, macOS may reset permissions when you update. If the hotkey stops working after an upgrade, you'll need to re-grant permissions — see [Upgrading](#upgrading) below.

### Upgrading

```sh
brew update && brew upgrade --cask murmur
```

Since Murmur is not notarized, macOS treats each update as a new app and **resets all permissions**. After upgrading:

1. Open **System Settings > Privacy & Security**
2. Remove the old `murmur` entry from Input Monitoring, Microphone, and Accessibility
3. Re-add and enable the new one

We know this is painful. Apple charges $99/year to make this go away — contributions welcome so we can stop making you click through permission dialogs like it's 2005.

### Uninstall

```sh
curl -sSL https://raw.githubusercontent.com/anubhavitis/murmur/main/uninstall.sh | sh
```

---

## Features

| Feature | Description |
|---|---|
| **Menubar app** | Lives in your system tray, zero windows |
| **Hold-to-record** | Right Option on macOS, Right Alt on Windows, or Caps Lock |
| **Fully offline** | No data leaves your machine |
| **Tier system** | Fast, Standard, Accurate — pick your quality/speed tradeoff |
| **Dual backends** | whisper.cpp everywhere, FluidAudio on Apple Silicon Macs |
| **Zero-downtime upgrades** | Two-slot transcriber swaps backends without interruption |
| **Auto-download** | Bootstraps with `tiny.en` (~74 MB), upgrades to tier model in background |
| **Clipboard or paste** | Copy to clipboard, or paste directly at cursor using `Cmd+V` or `Ctrl+V` |
| **100+ languages** | Pick preferred languages, auto-detect among them |
| **Audio feedback** | Beep on record start/stop |
| **Animated icon** | Tray icon spins while recording and transcribing |
| **Auto-start** | macOS Launch Agent today; Windows packaging is zip-based for now |

---

## Tiers

| Tier | Best for |
| --- | --- |
| **Fast** | Low latency, real-time |
| **Standard** | Balanced accuracy and speed |
| **Accurate** | Highest accuracy, multilingual |

---

## Build from source

<details>
<summary>For developers</summary>

### Requirements

| Requirement | macOS | Windows |
|---|---|---|
| Rust 1.85+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | `winget install Rustlang.Rustup` |
| cmake | `brew install cmake` | `winget install Kitware.CMake` |
| C/C++ toolchain | Xcode CLT (`xcode-select --install`) | Visual Studio Build Tools with MSVC |

> whisper.cpp is compiled from source automatically during `cargo build`.

### Build & run

```sh
git clone https://github.com/anubhavitis/murmur.git
cd murmur
cargo build --release
cargo run --release
```

### Release

On macOS:

```sh
./release.sh
```

On Windows PowerShell:

```powershell
.\release-windows.ps1
```

Builds the release binary, packages `murmur.exe` into a zip, and prints upload instructions for the existing GitHub release tag.

</details>

---

## Roadmap

- [ ] Live transcription (streaming partial results while recording)
- [ ] Native Windows installer / auto-start setup
