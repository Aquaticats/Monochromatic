# Hall Monitor

Local,
 privacy-first desktop productivity monitor for Linux.
Every 5 minutes it captures the screen and webcam,
feeds both images to a local vision LLM ([LFM2.5-VL-1.6B][] via llama.
cpp),
and sends a desktop notification after 5 consecutive unproductive cycles.

All computation runs locally;
 no data leaves the machine.

## How it works

1. **Capture**:
    takes a screenshot (Spectacle) and a webcam frame (ffmpeg/v4l2),
    downscales both via ffmpeg
2. **Buffer**:
    stores captures in a 10-minute rolling in-memory buffer so the LLM can compare across time
3. **Analyze**:
    starts a local `llama-server`,
    sends the buffered captures to LFM2.5-VL-1.6B,
    receives a PRODUCTIVE / UNPRODUCTIVE verdict
4. **Notify**:
    tracks the last 5 verdicts in a sliding window;
    when all 5 are UNPRODUCTIVE,
    fires a `notify-send` critical notification with the LLM's summary,
    then resets the window
5. **Repeat**:
    the llama-server is stopped between cycles to free VRAM,
    then the loop sleeps for 5 minutes

## Requirements

- [Bun][]:
   runtime and bundler
- **ffmpeg**:
   image capture and downscaling
- **Spectacle** (KDE):
   screenshot capture (`spectacle -f -b -n`)
- **v4l2** webcam at `/dev/video0`:
   webcam capture
- [llama.cpp](https://github.com/ggerganov/llama.cpp):
   local LLM inference (`llama-server`)
- [LFM2.5-VL-1.6B GGUF][LFM2.5-VL-1.6B] + mmproj:
   vision model files
- [distrobox](https://distrobox.it):
   container to run llama-server (used here for AMD GPU overrides)
- **notify-send**:
   desktop notifications (libnotify)
- **logger**:
   syslog logging (systemd)

## Setup

```bash
bun install
```

Edit the paths at the top of `src/analyze/llama.ts` to match your system:

```typescript
// src/analyze/llama.ts
const MODEL = '/path/to/LFM2.5-VL-1.6B-Q4_0.gguf';
const MMPROJ = '/path/to/mmproj-LFM2.5-VL-1.6b-Q8_0.gguf';
const LLAMA_SERVER = '/path/to/llama-server';
```

Download model files from Hugging Face:

```bash
pip install huggingface-hub
hf download LiquidAI/LFM2.5-VL-1.6B-GGUF --include "*Q4_0*" "*mmproj*Q8_0*" --local-dir /path/to/models
```

If you don't use distrobox or don't need `HSA_OVERRIDE_GFX_VERSION`,
adjust the `Bun.spawn` call in the same file to invoke `llama-server` directly.

## Usage

```bash
# Run directly
mise run run

# Run with file-watch (auto-restart on source changes)
mise run run:dev

# Compile to a single binary
mise run build
./hall-monitor
```

### Flags

- `--kill-existing`:
   send SIGTERM (then SIGKILL) to any running instance and take over the lock

### Autostart

Copy the included desktop entry to your autostart directory:

```bash
cp hall-monitor.desktop ~/.config/autostart/
```

Edit the `Exec` path inside the file to point to your built binary.

## Configuration

All configuration lives as constants in source files.
The main knobs:

- `INTERVAL_MS` in `src/index.ts`:
   time between capture cycles (default:
   5 min)
- `RETENTION_MS` in `src/analyze/memory.ts`:
   how long captures stay in the buffer (default:
   10 min)
- `SCREENSHOT_LONG_EDGE` in `src/infra/capture.ts`:
   max long-edge resolution for screenshots (default:
   1440 px)
- `WEBCAM_LONG_EDGE` in `src/infra/capture.ts`:
   max long-edge resolution for webcam (default:
   720 px)
- `PORT` in `src/analyze/llama.ts`:
   port for the local llama-server (default:
   8787)
- `temperature` / `top_p` / `top_k` in `src/analyze.ts`:
   LLM sampling parameters (default:
   0.7 / 0.8 / 20)

## Logging

All output goes to syslog via `logger -t hall-monitor`.
 View logs with:

```bash
journalctl -t hall-monitor -f
```

## Architecture

```text
src/
  index.ts            Entry point, argument parsing, main loop, signal handling
  cycle.ts            Orchestrates one capture-analyze-notify cycle
  analyze.ts          LLM prompt construction, API call, verdict parsing
  analyze/
    llama.ts          llama-server lifecycle (start/stop/health check)
    memory.ts         Rolling in-memory capture buffer
  infra/
    capture.ts        Screenshot (Spectacle) and webcam (ffmpeg) capture
    lock.ts           Single-instance enforcement via abstract Unix socket
    notification.ts   Desktop notifications (notify-send)
    syslog.ts         Thin syslog wrapper (logger)
    screenlock.ts     D-Bus session lock check
    blackdetect.ts    Webcam privacy cover detection
```

[Bun]: https://bun.sh
[LFM2.5-VL-1.6B]: https://huggingface.co/LiquidAI/LFM2.5-VL-1.6B-GGUF
