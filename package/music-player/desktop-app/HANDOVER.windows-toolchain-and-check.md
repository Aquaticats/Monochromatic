# Handover: Windows (x86_64, MSVC) toolchain + perceptual check

The Windows port (WASAPI via cpal;
 opus via opusic-sys/libopus 1.6.1) is
implemented and verified up to the boundary an agent can reach over SSH.
 This
runbook is the last step:
 confirming on the Windows machine's own screen and
speakers that the window renders,
 audio is audible,
 and transport works.
 Run it
from a terminal in your logged-in Windows desktop session (not the SSH session).

## Update 2026-10-14: the linker changed

The `lld-link.exe` pin was removed on 2026-10-14
(the maintainer found lld unreliable on Windows,
and a release probe showed it leaves the dynamic C runtime
in an `+crt-static` x86_64 exe).
`x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc` now link
with Visual Studio's own `link.exe`, which rustc finds through vswhere,
so **LLVM's `bin` no longer has to be on PATH**
and the Visual Studio C++ build tools are the only linker requirement.
The build and check results below were recorded with `lld-link.exe`;
re-run `mise run //package/music-player/desktop-app:build`
on the Windows machine to confirm the `link.exe` build there
(the GitHub Windows runners already build and test it, in the `test-windows` job of
`.github/workflows/music-player-release.yml`).
A GNU-toolchain build (`x86_64-pc-windows-gnu`, MSYS2 MinGW-w64, GNU ld, no Visual Studio) is available through
`mise run //package/music-player/desktop-app:build:windows-gnu`; the perceptual check below applies to either binary
(`target\x86_64-pc-windows-gnu\release\music-player.exe` for the GNU one).
Decision record: `doc/decision/music-player-windows-linkers.md`.

## What this proves

That `src/output_cpal.rs` plays sound through WASAPI,
 the winit window renders on
Windows,
 and the new `ITaskbarList3` taskbar progress bar actually moves,
 end to
end as a user experiences it.
 The code-level layers (build,
 link,
 clippy,
 unit
tests) are already verified over SSH (see "Already verified");
 the audio and the
taskbar bar can only be confirmed on the real logged-in desktop.

## Already verified (no action needed)

- `cargo build` (debug and `--release`) on the Windows box (Windows 10 build 19044,
  `x86_64-pc-windows-msvc`,
   CMake 4.3.3,
   rustc nightly):
   `Finished` with zero
  errors.
   cpal 0.18.1 (WASAPI),
   Slint 1.17.0-dev,
   and `opus` (opus-rs HEAD ->
  opusic-sys 0.7.3,
   libopus 1.6.1) all compiled;
   the binary linked with LLVM
  `lld-link.exe` (the linker then pinned in `.cargo/config.toml`) and embedded the icon via `winresource`.
   No
  `CMAKE_POLICY_VERSION_MINIMUM` override was needed (libopus 1.6.1's
  `cmake_minimum_required` is 3.16).
   The build needs
  `SLINT_ENABLE_EXPERIMENTAL_FEATURES=1` (the page-tab bar uses the experimental
  `FlexboxLayout`);
   every mise task sets it,
   so use
  `mise run //package/music-player/desktop-app:build`,
   not a bare `cargo build`.
- The Windows-native additions compile and are clippy-clean:
   the background-sweep
  `THREAD_PRIORITY_IDLE` call in `src/measure.rs`,
   and the `ITaskbarList3` taskbar
  progress in `src/ui_progress.rs`.
   Both use the `windows` crate (pinned 0.62),
  unified with the single `windows` 0.62 that cpal 0.18 and the Slint stack pull.
- `cargo clippy -- -D warnings`:
   clean (no warnings).
- `cargo test`:
   56 lib tests pass,
   0 failed (the pure-logic suite plus the two
  identity drift-guard tests,
   identical to Linux/macOS).
- Linux regression:
   host build,
   clippy (`-D warnings`),
   and the 56-test suite stay
  green;
   libopus is built via cmake from the mise `aqua:Kitware/CMake` tool.

## Bridges tried before handing this off

- Ran `cargo build`/`clippy`/`test` over `ssh x13-win` (all pass).
- A Windows OpenSSH session runs in a non-interactive session (session 0),
   not your
  logged-in desktop,
   so launching the GUI from SSH would not appear on your screen
  and has no audio device routing.
   The window-and-speaker check therefore has to run
  from a terminal in your logged-in Windows session;
   no agent bridge reaches it
  (`agent-browser` is web-only,
   and there is no `xdotool`/`wtype` equivalent that
  reaches into another Windows session over SSH).

## Setup

Status:
 TODO | DONE

1. In your logged-in Windows session,
    open **Windows Terminal** (or **PowerShell**
   or **Command Prompt**).
    Expected:
    a shell prompt on your desktop.
2. Run `cd %USERPROFILE%\mp-verify` (Command Prompt) or `cd $env:USERPROFILE\mp-verify`
   (PowerShell).
    Expected:
    no error.
    This is the synced tree the agent built;
    the
   debug binary is at `target\debug\music-player.exe`.

## Steps

Status:
 TODO | DONE

1. Run `target\debug\music-player.exe fixtures\tone.opus`.
    Expected:
    a music-player
   **window** opens on screen,
    and a steady 440 Hz tone plays from the speakers (the
   0.3s fixture loops,
    so it repeats).
    This fixture specifically exercises the new
   **opusic-sys / libopus 1.6.1** decode path.
    The window **title bar** shows the
   track path while playing.
2. For a longer,
    clearer listen,
    instead run `target\debug\music-player.exe %USERPROFILE%\Music`
   (or any folder of audio files).
    Expected:
    the **queue list** fills with the
   folder's tracks and the first one plays.
3. Click the highlighted **playing row** once (or the **play/pause** button).
   Expected:
    audio **stops instantly** (no ~1 second of buffered sound keeps
   playing),
    and the window title reverts to **Music Player**.
4. Click the same row again (or **play/pause**).
    Expected:
    audio resumes from where
   it paused.
5. Drag the **volume** slider down and up.
    Expected:
    loudness follows immediately.
6. Click **next** then **prev** (the transport buttons).
    Expected:
    playback moves to
   the adjacent track and the highlighted row follows.
7. Drag the **seek bar** to a new position.
    Expected:
    playback jumps to that point.
8. With a track still playing (the longer folder from step 2 gives time to watch),
   look at the **music-player icon on the Windows taskbar**.
    Expected:
    a green
   **progress bar** fills across the icon as playback advances.
    Then pause (click the
   playing row or **play/pause**):
    the bar **clears**;
    resume:
    it **returns**.
    This
   is the new `ITaskbarList3` taskbar progress,
    the Windows counterpart to the Linux
   KDE LauncherEntry bar.
9. Close the **window**.
    Expected:
    the process exits and the prompt returns.

## What to check

Status:
 TODO | DONE

- A window actually appears and is interactive (not just a process that runs).
- Sound is audible in step 1 (the Opus fixture) or step 2.
- In step 3,
   pausing cuts the sound immediately.
   This is the specific behavior the
  cpal callback's "don't drain the ring buffer while paused" logic guarantees;
   a
  delayed stop would mean that guard is wrong.
- No line containing `music-player: audio init failed` or
  `music-player: cpal stream error` is printed in the terminal at any point.
- In step 8,
   the taskbar icon shows a green progress bar that tracks playback and
  clears on pause.
   A bar that never appears means the window `HWND` was not resolved
  or COM init failed (the code fails silently and playback is unaffected,
   so this is
  the only place that path is observable).

## Restore

Status:
 TODO | DONE

1. If the app is still running,
    close its window or press **Ctrl+C** in the
   terminal.
    Expected:
    the prompt returns.
2. Optional:
    remove the throwaway build tree with `rmdir /s /q %USERPROFILE%\mp-verify`
   (Command Prompt) or `Remove-Item -Recurse -Force $env:USERPROFILE\mp-verify`
   (PowerShell).
    This is a synced copy the agent created,
    not a repo checkout,
    so
   deleting it affects nothing else.
