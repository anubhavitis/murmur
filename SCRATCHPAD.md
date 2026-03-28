# Scratchpad - Decision Log

## Completed Phases

### Phase 1 — Menubar tray app + config (`b07f8eb`)

- tao event loop + tray-icon for menubar
- Config at `~/.murmur/config.json`, models at `~/.murmur/models/`
- No async runtime — sync threads + EventLoopProxy

### Phase 2 — Global hotkey + audio capture (`97843dc`)

- rdev for global hotkey (Right Option default)
- cpal for audio capture, mono downmix, 16kHz resampling
- Switched to fufesou/rdev fork to fix macOS `TSMGetInputSourceProperty` crash on background thread

### Phase 3 — Whisper transcription + clipboard (`5122d21`)

- whisper-rs with long-lived background thread (model loaded once, samples via channel)
- Auto-download tiny model from HuggingFace on first run
- arboard for clipboard copy
- Default model changed from "small" to "tiny.en"
- Suppressed all whisper-rs internal C-level logs via `whisper_log_set(None)`

### Phase 4 — Paste at cursor (`3af9024`)

- enigo for key simulation (Cmd+V on macOS, Ctrl+V on Windows)
- Platform abstraction: `platform::paste_modifier()` returns correct key per OS

### Phase 5 — Model download via menu (`b992334`)

- Background thread download with streaming progress
- Progress shown in tray menu via in-place `MenuItem::set_text()` — menu stays open during download
- First progress event triggers one `rebuild()` to add progress item, all subsequent updates use `set_text()`
- Auto-selects model after download completes

### Post-phase polish

- Custom logo: mic that looks like a smiling mouth
- Tray icon rotates during recording (36 pre-computed frames, bilinear rotation from large PNG, Lanczos3 downscale to 44x44)
- WaitUntil timer at 33ms (~30 FPS) during Recording state
- Cross-platform paste modifier (platform/macos.rs, platform/windows.rs)
- Cleaned up all info logs — only errors + 3 status lines (recording, transcribing, result)
- Model registry: both english (.en) and multilingual variants for tiny/base/small/medium
- Large model uses `large-v3` filename (ggml-large.bin doesn't exist on HuggingFace)
- Download icon (⬇) for unavailable models + model size shown in menu
- Model sizes verified from HuggingFace Content-Length headers

## Key Decisions

- Default hotkey: Right Option/Alt
- Default model: `tiny.en` (English-only, fastest)
- HuggingFace URL: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{file_name}.bin`
- Config: `~/.murmur/config.json`, models: `~/.murmur/models/`
- No async runtime — sync threads + EventLoopProxy
- tao event loop on main thread, everything else on background threads
- English-only transcription (configurable later)
- 4 whisper threads
- fufesou/rdev fork (fixes macOS dispatch_assert_queue crash)
- Rotate full logo for animation (not just arc) — swap image later if needed
- Menu stays open during download — use `MenuItem::set_text()` for in-place updates

## Future / Deferred

- **Live transcribing** — streaming partial results while recording
- **Windows testing** — code is cross-platform ready but untested
- **Distribution** — .app bundle, signing, notarization, DMG/Homebrew
- **Language selection** — currently hardcoded to English
- **CoreML backend** — whisper-rs with CoreML for better Apple Silicon perf
