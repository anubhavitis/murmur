# Scratchpad - Decision Log

## Phase 1 Decisions
- Default hotkey: Right Option/Alt (will make configurable later)
- HuggingFace URL pattern: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{modelSize}.bin`
- Config location: `~/.murmur/config.json`
- Models location: `~/.murmur/models/`
- No async runtime — sync threads + EventLoopProxy
- tao event loop on main thread, everything else on background threads
