# Handover: macOS (Apple Silicon) perceptual check

The macOS port (CoreAudio via cpal) is implemented and verified up to the boundary
an agent can reach over SSH.
 This runbook is the last step:
 confirming on the Mac's
own screen and speakers that the window renders,
 audio is audible,
 and transport
works.
 Run it in a Terminal on the Mac itself (a logged-in GUI session).

## What this proves

That the new `src/output_cpal.rs` backend plays sound through CoreAudio and
that the winit window renders on macOS,
 end to end as a user experiences it.
 The
code-level and launch-level layers are already verified (see "Already verified").

## Already verified (no action needed)

- Build and clippy on the M1 (arm64,
   macOS 26.5.1) against the CURRENT dependency
  set:
   `cargo build` and `cargo clippy -- -D warnings` both finish clean,
   zero
  warnings.
   cpal 0.18.1 + coreaudio-rs compile;
   `opus` builds libopus 1.6.1 from
  source via CMake (the opusic-sys backend;
   there is no Homebrew/pkg-config system
  libopus in the link);
   and the macOS background-QoS path in `src/measure.rs`
  (`pthread_set_qos_class_self_np` / `QOS_CLASS_BACKGROUND`) compiles.
   Note the
  build needs `SLINT_ENABLE_EXPERIMENTAL_FEATURES=internal-types` because `ui/theme.slint`
  reads `SlintInternal.accent-color`;
   every mise task sets it,
   so
  build with `mise run //package/music-player/desktop-app:build`,
   not a bare
  `cargo build` (which fails with `Cannot access id 'SlintInternal'`).
- An earlier launch over SSH ran the event loop for ~8s with no panic and no
  stderr,
   so `Output::new` opened the default output device (the engine prints
  `music-player: audio init failed: ...` from `engine.rs`:
  300 on failure,
   and stderr
  was empty).
   The device-init path is unchanged by the cpal 0.15 -> 0.18 bump.
- Linux regression:
   build,
   clippy (`-D warnings`),
   and the 56-test suite stay
  green;
   the PipeWire path is unchanged.

## Bridges tried before handing this off

- Ran the built binary over `ssh m1` (it stayed alive,
   no errors).
- `screencapture -x` on the Mac over SSH to grab the window:
   failed with
  `could not create image from display`.
   An SSH session is not in the Mac's Aqua
  (GUI) session,
   so it can reach neither the WindowServer (to capture the window)
  nor,
   reliably,
   the audio output.
   The screen-and-speaker check therefore has to
  run from a Terminal on the Mac itself;
   no agent bridge reaches it.

## Setup

Status:
 TODO

1. On the Mac,
    open **Terminal** (the real macOS Terminal app,
    not an SSH
   session).
    Expected:
    a shell prompt in your logged-in desktop session.
2. Run `cd ~/music-player-verify`.
    Expected:
    no error (this is the synced tree
   the agent built;
    the debug binary is at `target/debug/music-player`).

## Steps

Status:
 TODO

1. Run `./target/debug/music-player fixture/tone.flac`.
    Expected:
    a music-player
   **window** opens on screen,
    and a steady 440 Hz tone plays from the speakers
   (the 0.3s fixture loops,
    so it repeats).
    The window **title bar** shows the
   track path while playing.
2. For a longer,
    clearer listen,
    instead run
   `./target/debug/music-player ~/Music` (or any folder of audio files).
    Expected:
   the **queue list** fills with the folder's tracks and the first one plays.
3. Click the highlighted **playing row** once (or the **play/pause** button).
   Expected:
    audio **stops instantly** (no ~1 second of buffered sound keeps
   playing),
    and the window title reverts to **Music Player**.
4. Click the same row again (or **play/pause**).
    Expected:
    audio resumes from
   where it paused.
5. Drag the **volume** slider down and up.
    Expected:
    loudness follows immediately.
6. Click **next** then **prev** (the transport buttons).
    Expected:
    playback moves
   to the adjacent track in the list and the highlighted row follows.
7. Drag the **seek bar** to a new position.
    Expected:
    playback jumps to that point.
8. Close the **window**.
    Expected:
    the process exits and the prompt returns.

## What to check

Status:
 TODO

- A window actually appears and is interactive (not just a process that runs).
- Sound is audible in step 1 or 2.
- In step 3,
   pausing cuts the sound immediately.
   This is the specific behavior the
  cpal callback's "don't drain the ring buffer while paused" logic guarantees;
   a
  delayed stop would mean that guard is wrong.
- No line containing `music-player: audio init failed` or
  `music-player: cpal stream error` is printed to the Terminal at any point.

## Restore

Status:
 TODO

1. If the app is still running,
    close its window or press **Ctrl+C** in the
   Terminal.
    Expected:
    the prompt returns.
2. Optional:
    remove the throwaway build tree with `rm -rf ~/music-player-verify`.
   This is a synced copy the agent created,
    not your repo checkout,
    so deleting it
   affects nothing else.
