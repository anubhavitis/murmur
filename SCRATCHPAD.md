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
- Tray icon rotates during recording AND transcribing (36 pre-computed frames, bilinear rotation, Lanczos3 downscale to 44x44)
- WaitUntil timer at 33ms (~30 FPS) during non-Idle states
- Cross-platform paste modifier (platform/macos.rs, platform/windows.rs)
- Cleaned up all info logs — only errors + 3 status lines (recording, transcribing, result)
- Model registry: both english (.en) and multilingual variants for tiny/base/small/medium
- Large model uses `large-v3` filename (ggml-large.bin doesn't exist on HuggingFace)
- Download icon (⬇) for unavailable models + model size shown in menu
- Model sizes verified from HuggingFace Content-Length headers

### Language preferences

- Config field `languages: ["en"]` — English always default, can't be removed
- Languages submenu: selected at top, "Add language..." for the rest (100 languages)
- Transcriber uses `lang_detect()` filtered to preferred languages only
- Single language → set directly, no auto-detect
- Multilingual models use `set_language(Some("auto"))` — NOT `detect_language(true)` (that returns 0 segments)
- English-only `.en` models ignore language preferences

### Audio & UX feedback

- Audio beep on record start (Tink.aiff) and stop (Pop.aiff) via `afplay`
- Windows: PowerShell `SystemSounds`
- Self-healing hotkey: AtomicU64 timestamp replaces AtomicBool, stale press (>500ms) treated as fresh
- Diagnostic logging when hotkey press rejected due to busy state
- Error sound on audio start failure

### Permission flow

- Non-blocking: tray icon appears immediately regardless of permission state
- Microphone: triggered via cpal stream creation on startup (system dialog)
- Accessibility: `AXIsProcessTrusted()` checked via FFI, opens Settings pane if missing
- Input Monitoring: handled by rdev::listen failure → opens Settings → self-restart via exec()
- Retry limit: 3 attempts for hotkey listener, then gives up
- Tray menu shows `⚠ Microphone: grant permission` / `⚠ Accessibility: grant permission` when missing
- Permission timer polls every 2s, stops when all granted

### Resilience fixes (v0.2.1)

- Transcriber thread: no more `.expect()` panics — errors sent via AppEvent
- Transcriber sends `TranscriberReady` event when model loaded — tray shows "Loading model..." until then
- Model download moved to transcriber background thread (non-blocking startup)
- Whisper state created once, reused across transcriptions
- `sender.send()` checked — sends TranscriptionError if channel dead
- Download race prevented — `download_progress` set immediately before spawning thread
- Hotkey change triggers self-restart to apply immediately
- Empty languages enforced to `["en"]` in config load and toggle
- `fs::remove_file` before `fs::rename` for Windows compatibility

### Distribution (v0.2.2 → v0.2.3)

- **.app bundle**: `Murmur.app` at `/Applications/` with Info.plist
  - CFBundleIdentifier: `com.murmur.app` — permissions tied to bundle ID, persist across upgrades
  - LSUIElement: true — no Dock icon
  - CFBundleIconFile: AppIcon.icns — logo shows in Finder, permissions dialogs
  - NSMicrophoneUsageDescription for mic prompt
- **Install script** (`install.sh`): downloads zip, extracts Murmur.app to /Applications, creates Launch Agent, migrates old bare-binary install
- **Uninstall script** (`uninstall.sh`): removes .app, plist, optionally config+models
- **Homebrew cask**: `app "Murmur.app"` stanza, preflight strips quarantine + unloads old agent, postflight creates plist + starts agent
- **Release script** (`release.sh`): builds binary, creates .app structure with icon, zips, computes sha256, generates cask formula, creates git tag
- **Archive format**: .zip (not .tar.gz) for Homebrew cask compatibility
- **Gatekeeper**: `xattr -cr` in cask preflight (before move to /Applications) and in install.sh
- **Launch Agent**: `com.murmur.app`, RunAtLoad, KeepAlive on crash only, ThrottleInterval 30s
- **Log rotation**: rename to murmur.log.1, create fresh murmur.log on startup
- **Landing page**: `docs/index.html` via GitHub Pages, Tailwind CDN, dark theme

### About section

- Version from `env!("CARGO_PKG_VERSION")` shown in tray menu
- Website link opens landing page in browser

## Key Decisions

- Default hotkey: Right Option/Alt
- Default model: `tiny.en` (English-only, fastest)
- HuggingFace URL: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{file_name}.bin`
- Config: `~/.murmur/config.json`, models: `~/.murmur/models/`
- App bundle: `/Applications/Murmur.app` (permissions persist via bundle ID)
- No async runtime — sync threads + EventLoopProxy
- tao event loop on main thread, everything else on background threads
- 4 whisper threads
- fufesou/rdev fork (fixes macOS dispatch_assert_queue crash)
- Rotate full logo for animation — swap image later if needed
- Menu stays open during download — use `MenuItem::set_text()` for in-place updates
- No Apple Developer account / notarization — open source, quarantine stripped via xattr
- No CI/CD — manual releases via `./release.sh`

## Releases

- **v0.1.0** — initial release, bare binary
- **v0.2.0** — language preferences, audio feedback, permission flow, bug fixes
- **v0.2.1** — critical fixes (non-blocking startup, transcriber resilience)
- **v0.2.2** — .app bundle (permissions persist across upgrades)
- **v0.2.3** — Gatekeeper fix, app icon, clean upgrade handoff

### App icon fix (opaque background)

- Previous `AppIcon.icns` had transparent background — invisible on macOS glass/light/tinted menubar themes
- Replaced with white logo on solid dark (#1E1E1E) opaque background
- Generated all required sizes (16x16 through 512x512@2x) via Swift + `iconutil`
- macOS applies squircle mask automatically — works on all themes

## Future / Deferred

- **Live transcribing** — streaming partial results while recording
- **Windows testing** — code is cross-platform ready but untested
- **CoreML backend** — whisper-rs with CoreML for better Apple Silicon perf
- **Apple Developer signing** — free signing for open source, would eliminate Gatekeeper issues entirely
