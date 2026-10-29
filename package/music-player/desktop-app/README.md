# music-player

A minimal native music player built with the Slint GUI toolkit.
 On Linux it pairs a winit window (Wayland, or X11 when no Wayland display is set) with a native
PipeWire client;
 on macOS (Apple Silicon) and Windows it pairs a winit window with the OS audio engine through
cpal (CoreAudio on macOS,
 WASAPI on Windows).
 The window layer is winit on every platform,
 so only the audio
backend differs by target.
 Scope is deliberately small:
 a single watched Source Root and broad codec coverage.

## Scope

- Output:
   the platform's native stack.
   Linux uses a winit window and PipeWire audio (no
  PulseAudio backend).
   Builds made with the `alsa-fallback` cargo feature
  (the Alpine build,
   see [Alpine Linux](#alpine-linux))
  also open the default ALSA device when no PipeWire daemon answers.
   The window runs on Wayland when `WAYLAND_DISPLAY` (or `WAYLAND_SOCKET`) is set and otherwise on X11 when `DISPLAY` is
  set, because the Slint `backend-winit` feature turns on both of winit's Linux window systems;
   with neither variable set, startup fails with winit's error.
   The X11 libraries (`libX11`, `libxcb`, `libxkbcommon-x11`) are loaded at run time, not linked.
   macOS and Windows use a winit window and the OS audio engine via cpal (CoreAudio on
  macOS,
   WASAPI on Windows).
   The window is winit on every platform,
   so only the audio backend is platform-specific.
- Source:
   a single Source Root directory.
   The queue is whatever audio files exist under that root (subfolders scanned recursively),
   re-derived from disk rather than built up by hand;
   opening a folder makes it the new Source Root.
   The root is watched while the app runs,
   so files added,
   removed,
   or renamed on disk are reflected live by a debounced refresh of the paths that changed,
   and a full rescan on launch (after the watch is registered) self-corrects the restored queue.
- Transport:
   play/pause,
   seek,
   next/prev,
   and one playback mode of four:
   Repeat,
   In order,
   Shuffle (the folder on screen),
   and Shuffle all.
   There is no volume control in the app (D43): the operating system's mixer sets the level.
- Playback scope:
   the mode also chooses what playback walks.
   Repeat replays the current track when it ends.
   In order and Shuffle the folder confine playback to
  the current page (the track's top-level folder under the opened folder,
   or the virtual folder named by the first letter, A to Z or `#`, for a track directly in the opened folder);
   In order plays the page in name order,
   Shuffle the folder shuffles it.
   Shuffle all spans the whole queue,
   shuffled.
   In order stops at the end of the folder (D2: "In order = play to the end of the folder and stop"):
   the player pauses with the first track of the folder loaded,
   and Play starts the folder again.
   No later decision supersedes D2 or D6.
   A manual Next at the last track still wraps to the first.
   Shuffle the folder and Shuffle all start a new cycle when every track has played.
   Play with nothing selected starts the first track of the folder on screen.
   This deliberately cannot
  express non-shuffle with whole-queue repeat:
   when playing in order,
   playback stays inside the current folder
  rather than jumping to another artist once the folder ends.
- Metadata:
   filesystem path only,
   with no tag parsing and no album art.
   A track row shows a title cleaned from the file name (D86) and a supporting line from the track rows template
  (the length and the true peak by default, `$tf(mi(len), m:ss)$ $mi(peak)$`, editable in Settings).
   The seek bar's position and duration come from the
  decoder (frame count over sample rate),
   not from tags.
- Sample rate:
   each track is opened at its own native rate,
   and the OS audio server resamples to the device
  (PipeWire on Linux,
   CoreAudio on macOS,
   WASAPI on Windows),
   so the player itself never resamples.
   Playback is gapless between tracks of the same sample rate and channel count:
  the next track is opened a moment before the current one ends and follows it in the same stream.
- Output safety:
   per-track true-peak normalization,
   always on.
   Each track's true (inter-sample) peak is
  measured by oversampling,
   and the track plays at a single constant gain that brings that peak down to a
  -1 dBTP ceiling (the EBU R128 / ATSC A/85 true-peak ceiling).
   Normalization is attenuate-only:
   tracks already
  below the ceiling are left untouched,
   and a quiet track is never boosted (which would risk a sudden loud
  level).
   A hard clamp to the valid range backstops measurement error and any residual overshoot.
- Peak cache:
   an exact true-peak measurement means decoding the whole file,
   so each result is memoized on disk in a Turso database,
   `decisions.db` under the config directory (`src/peakcache_service.rs`),
   keyed by an opaque fingerprint (a hash of path,
   size,
   and modified-time).
   The database stores
  only `fingerprint -> peak` rows;
   no filename,
   path,
   or tag is ever written,
   so it reveals nothing about the
  library.
   On every queue load (an Open or the launch-time auto-load),
   a background sweep reads each track's cached result and header length at once,
   then measures the queue's non-current tracks into the cache with one worker per logical core at idle scheduling
  priority,
   skipping any already cached,
   so re-opening a known folder does little work.
  The current track never makes playback wait.
   On a cache miss it starts playing at once with the safe -1 dBTP ceiling gain while a dedicated worker measures it
  (`src/peak_swap.rs`).
  When the measured gain lands,
   future decoded samples swap to it;
   already-buffered fallback-gain samples are not flushed.

## Alpine Linux

The player builds and runs on Alpine Linux (musl libc),
on x86_64 and aarch64.
The decision and every measurement behind it are
 in [`doc/decision/music-player-alpine.md`](../../../doc/decision/music-player-alpine.md).
The binary is dynamically linked against Alpine's musl and runs only on musl systems.
A fully static binary is not possible:
the PipeWire,
fontconfig and freetype static libraries do not exist on Alpine,
and a static musl program cannot `dlopen` the Wayland,
xkbcommon and EGL libraries a window needs.

### Install the runtime packages

```sh
apk add fontconfig font-dejavu pipewire-libs libgcc wayland-libs-client wayland-libs-egl libxkbcommon \
  mesa-egl mesa-gles mesa-dri-gallium alsa-lib
```

- `fontconfig` and a font:
  with no font installed the player panics at startup,
  so install one (`font-dejavu`
  is the small one;
  any font package works).
- `pipewire-libs`,
  `libgcc`,
  `wayland-libs-client`,
  `wayland-libs-egl`,
  `libxkbcommon`:
  libraries the binary links or opens at run time.
- `mesa-egl`,
  `mesa-gles`,
  `mesa-dri-gallium`:
  the OpenGL ES stack the window draws with
  (the software rasterizer on a machine with no GPU).
- `alsa-lib`:
  the ALSA fallback when no PipeWire daemon runs.

You also need a Wayland compositor,
and for sound either PipeWire with a session manager or an ALSA device:

```sh
apk add pipewire wireplumber
```

Optional:
`dbus` (a session bus,
for notifications and the appearance setting),
`zenity` (the folder chooser when
no desktop portal runs),
and `xdg-desktop-portal`.

### Run

Alpine does not create `XDG_RUNTIME_DIR` unless `elogind` runs.
Wayland and PipeWire need it,
so create it by hand when it is unset:

```sh
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/runtime-$(id -u)}"
mkdir -p "$XDG_RUNTIME_DIR" && chmod 700 "$XDG_RUNTIME_DIR"
music-player path/to/music-folder
```

### What to expect

- Sound:
  the player uses PipeWire when a daemon answers,
  otherwise the default ALSA device (set it in
  `/etc/asound.conf` or `~/.asoundrc`).
  With neither,
  the player starts,
  shows "No sound output is available,
  so nothing will be heard.
  Start a sound server
  such as PipeWire,
  or set up an ALSA sound card,
  then restart the player.",
  and stays silent.
- Folder chooser:
  "Open folder" needs a desktop portal or `zenity`.
  Without either,
  the window says "This computer has no folder chooser for the player to open.
  Install zenity,
  or start
  the player with a folder:
  music-player /path/to/music".
- Notifications and the appearance setting need a D-Bus session bus.
  Without one the player logs a warning and runs.
- Settings,
  the session and the true-peak cache are written under `$XDG_CONFIG_HOME/musicplayer`
  (`~/.config/musicplayer` by default).

### Build from source on Alpine

```sh
apk add rust cargo build-base cmake samurai pkgconf linux-headers clang21-libclang \
  pipewire-dev alsa-lib-dev fontconfig-dev freetype-dev
cd package/music-player/desktop-app
SLINT_ENABLE_EXPERIMENTAL_FEATURES=1 cargo build --release --locked --features alsa-fallback
```

Unit tests also need `cargo-nextest dbus font-dejavu`:

```sh
SLINT_EMIT_DEBUG_INFO=1 SLINT_ENABLE_EXPERIMENTAL_FEATURES=1 cargo nextest run --locked --features alsa-fallback
```

### Checking the Alpine build

`mise run //package/music-player/desktop-app:test:alpine` builds `Containerfile.alpine` with podman,
runs the unit
tests,
builds the app and the repository's nested compositor,
and runs the app inside that compositor in three
scenarios (no sound,
ALSA,
PipeWire),
with explicit memory and CPU bounds.
The `test-alpine` job of `.github/workflows/music-player-release.yml` runs the same commands on x86_64 and aarch64,
on a release or a manual run of that workflow.

## Codecs

One demux path (symphonia) feeds two decode paths:

- symphonia decoders:
   FLAC,
   WAV/PCM,
   MP3,
   Vorbis (Ogg),
   AAC-LC and ALAC (MP4),
   ADPCM,
   AIFF.
- libopus (the `opus` crate) for Opus,
   fed raw packets from symphonia's Ogg demuxer.
   The symphonia 0.6 meta-crate exposes no Opus decoder
  (`symphonia-codec-opus` exists but is not in its `all` feature set),
   so the dedicated library handles it.

Per-codec decode is covered by one test each over the committed `fixture/` tones.

## Layout

The crate is a library plus a thin binary so the pure logic is unit-testable without audio or a GUI.

- `src/lib.rs`:
   module root.
- `src/command.rs`:
   the UI-to-engine `Command` and engine-to-UI `Update` message enums,
   plus `PlaybackMode` (Repeat,
   In order,
   Shuffle the folder,
   Shuffle all).
- `src/queue.rs`:
   the play-queue model (load order,
   the current scope's playback order,
   cursor,
   and the one playback mode),
   with a seedable PRNG for deterministic shuffle tests.
   The playback scope is the current
  track's page (In order and Shuffle the folder,
   computed via `pagination`) or the whole queue (Shuffle all).
   In order stops at the end of its page;
  a manual Next wraps.
   `display_paths` builds the UI list as each track's path relative to the opened folder (via `relpath`),
  so a library with one top-level folder keeps that folder's name.
- `src/session.rs`:
   save and restore of the last session under the platform config directory
  (the Source Root,
   the selected track path,
   the position,
   the playback mode, the two template texts, the Auto-play on select flag, `auto_play_on_select`, which is on
   when the field is missing (D133), and the four text sizes, `text_size_pane_title_px`, `text_size_group_heading_px`,
   `text_size_folder_name_px` and `text_size_track_title_px`, each at its default when its field is missing or wrong;
   it keeps no queue,
   so nothing is pruned or remapped: the launch rescan rebuilds the queue).
- `src/error.rs`:
   `PlayerError`,
   the single error type all fallible functions return.
- `src/decode.rs`:
   probing and decoding to interleaved `f32` PCM behind a `Source` trait (`AudioSpec`,
   `open`).
- `src/opus.rs`:
   the libopus `Source` for Opus.
- `src/output_pipewire.rs` and `src/output_cpal.rs`:
   the audio-output boundary,
   selected by target in
  `lib.rs` (PipeWire on Linux;
   cpal on every non-Linux target,
   driving CoreAudio on macOS and WASAPI on
  Windows).
   Both expose the identical `Output` surface (`new`,
   `set_playing`,
   `reconfigure`);
   `reconfigure`
  builds an output stream at a track's native format and returns the producer half of a lock-free ring buffer.
  Everything outside this boundary is platform-agnostic and only ever touches that producer,
   so adding the cpal
  backend changed no engine,
   controller,
   or decode code.
- `src/playback.rs`:
   device-free playback helpers,
   kept apart so they are unit-testable:
   the per-sample
  gain-and-clamp stage,
   frame-to-seconds conversion,
   recursive folder expansion,
   and the audio-file test.
  Folder scans enqueue only files whose extension is in the audio allowlist (flac,
   wav/wave,
   mp3,
   ogg/oga,
  opus,
   m4a/m4b/mp4,
   aac,
   aiff/aif/aifc),
   so cover art,
   playlists,
   and system files (`.DS_Store`,
   `.nomedia`,
  `.database_uuid`,
   and other dotfiles) never enter the queue.
- `src/truepeak.rs`:
   the desktop adapter over the shared `truepeak-core` crate,
   which owns the Catmull-Rom oversampling meter,
   the measurement policy,
   the attenuate-only normalization gain and the decision cache.
   This file hands the desktop decoder to its two resolvers:
   `resolve_current` for the track about to play and `resolve_full` for the background sweep's exact full scan.
- `src/peakcache.rs`,
   `src/peakcache_service.rs` and `src/peakcache_handle.rs`:
   the persistent peak cache.
   `peakcache.rs` computes the opaque fingerprint (through the shared XXH3-64 `track_fingerprint` of `truepeak-core`)
    and decides where `decisions.db` lives;
   the Turso-backed actor in `peakcache_service.rs` owns the database,
   and the synchronous `CacheHandle` in `peakcache_handle.rs` is what the controller and the sweep call.
- `src/measure.rs`:
   background measurement orchestration.
   `spawn_queue_measurement` runs the detached background sweep over a queue's non-current tracks:
   it first reports each track's cached decision and header length,
   then fans out one worker per logical core,
   and it is never cancelled (a newer queue load makes the controller drop the older sweep's reports).
   Every sweep thread first drops itself to the platform's
  lowest scheduling tier so its CPU-bound decoding never competes with the audio thread or the UI:
   Linux
  `SCHED_IDLE`,
   macOS background QoS (`pthread_set_qos_class_self_np`),
   and Windows `THREAD_PRIORITY_IDLE`,
   with a
  no-op on any other target.
- `src/sweep_gate.rs`:
   the pause gate of the background sweep (D107).
   Every sweep worker asks the gate for admission before it claims the next track,
   so Pause stops new claims, a track already being decoded finishes, Resume wakes the parked workers,
   and closing the gate at shutdown releases them.
- `src/analysis_notification_logic.rs`, `src/analysis_notification_wire.rs`, `src/analysis_notification.rs`:
   the true-peak analysis indicator (D107), which replaced the in-app banner.
   On KDE it is a Plasma job (`org.kde.kuiserver`) with a native Pause button;
   elsewhere it is one freedesktop notification replaced in place; with neither, analysis runs without an indicator.
   The logic file holds the pure rules (rate limit of two updates a second, skipping repeats, pause and dismissal),
   the wire file holds the D-Bus messages (Linux only),
   and the handle runs them on its own thread so the window and the sweep workers never wait on the bus.
   Protocol facts and pitfalls: `doc/troubleshooting/plasma-job-view-progress-protocol.md`.
- `src/unplayable_notification_logic.rs`, `src/unplayable_notification_wire.rs`, `src/unplayable_notification.rs`:
   the "cannot be played" report as one freedesktop notification (D122), which replaced the in-window message.
   The logic file holds the wording and a ledger that tells each file once per run,
   the wire file holds the one `Notify` call (normal urgency, the server's default expiry, no action buttons,
   a `desktop-entry` hint), and the handle runs it on its own thread and connection, separate from the analysis
   indicator's, so a failure of one never disables the other.
   Its tests run against a private `dbus-daemon` (`src/unplayable_notification_bus_support.rs`), never the session bus.
- `src/peak_swap.rs`:
   current-track peak swap orchestration.
   Cache hits return the measured gain immediately;
   cache misses set the temporary ceiling gain,
   spawn a current-track worker,
   and expose a pending result that the controller polls before decoding.
- `src/controller.rs`:
   the playback state machine (state struct,
   command handling,
   background-measurement
  kickoff).
   It owns the queue,
   the active decoder,
   the output,
   and the shared peak cache.
- `src/controller_audio.rs`:
   the second `impl Controller` block (loading,
   gain resolution,
   audio pumping,
  position reporting),
   split out to keep each file within the line budget.
- `src/engine.rs`:
   the worker-thread front door.
   `Engine::spawn` starts the background thread;
   `run` builds a
  `Controller` and drives it from the command channel.
- `src/pagination.rs`:
   pure queue pagination.
   `paginate` groups each track's relative display path:
  a track in a subfolder gets a page per top-level folder under the opened folder (one level only;
   the label is
  that folder,
   with deeper nesting shown in the row path),
   and a track directly in the opened folder sits in a virtual folder named by its first letter
  (the 26 English letters A-Z,
   case-insensitive,
   plus a `#` catch-all for digits,
   symbols,
   CJK,
   and
  non-English letters).
   Search names the folder of such a track the same way (`root_level_folder_label`).
   Pages sort folder-pages-first (case-insensitively by label,
   so lowercase-led folder
  names like `daniwellP` and `r-906` interleave with the capitalized names instead of trailing after `Zedd`),
  then A-Z,
   then `#`.
   `page_of_index` finds the page
  holding a given track.
   No GUI or audio,
   so it is unit-tested directly.
- `src/relpath.rs`:
   pure relative-path display.
   `relative_display_paths` strips the longest common directory
  prefix shared by all queued tracks (always leaving at least the filename),
   so the UI shows `Artist/Album/01.flac`
  rather than the full absolute path,
   or just `01.flac` when the whole queue is one folder.
   No I/O,
   so it is
  unit-tested directly.
- `src/identity.rs`:
   the single source of truth for the platform identity strings (the Wayland app id,
   the macOS
  `CFBundleIdentifier`,
   and the config-dir reverse-DNS triple),
   so the three platforms' names cannot drift.
   The
  three reverse-DNS roots differ on purpose;
   a unit test asserts the macOS `Info.plist` and the Linux `.desktop`
  file still carry the centralized values.
- `src/launcher.rs`:
   Linux desktop-shell integration.
   `set_window_app_id` is a winit window-attributes hook that
  stamps the Wayland app id (`monochromatic.music-player`, which winit also uses as the X11 `WM_CLASS`);
   `Launcher` emits the
  `com.canonical.Unity.LauncherEntry` `Update` signal on the session bus to drive the taskbar progress;
   what the bar shows is decided by the state machine in `src/launcher/taskbar_state.rs`,
   described in "Taskbar progress".
   Both are best-effort:
   no session bus or an unsupporting
  shell silently disables progress.
   KDE Plasma renders it natively;
   GNOME needs Dash-to-Dock.
   Both are Linux-only
  and compile to no-ops on macOS and Windows,
   where the Wayland app id and the D-Bus signal have no analogue.
  Windows still gets a taskbar progress bar,
   driven natively through the `ITaskbarList3` COM interface in
  `src/ui_progress.rs` (the Windows counterpart to the Linux LauncherEntry signal);
   macOS has no dock progress
  API,
   so there the taskbar progress is genuinely a no-op.
- `src/main.rs`:
   builds the Slint window,
   spawns the engine,
   and wires callbacks to commands and updates to
  properties.
   It installs the winit backend with the app-id hook before creating the window,
   creates the
  `Launcher`,
   and hands every engine message to `src/ui_progress.rs`,
   which feeds the taskbar state machine and the seek bar.
   The callbacks are wired in `src/ui_wire.rs` and the other `src/ui_*.rs` files.
   The pages (the folders of the picker and the rows of the selected one) come from a page index
  (`src/page_index.rs`) that `ScreenState` builds once per queue;
   the queue builds the same index to scope In order and Shuffle the folder to a page.
- `ui/app.slint` and `ui/app_window.slint`:
   `app.slint` is the entry point that `build.rs` compiles;
   the window markup is in `app_window.slint` (see the `ui/*.slint` files it imports).
   The folder picker lists the folders and the virtual letter folders of loose files,
   with a rail that jumps by letter or writing system (D28).
   Wide windows show the picker and the transport deck in the left pane and the selected folder's
  rows in the right pane,
   each its own scroll container;
   narrow windows show one pane at a time.
   The switch is an
  explicit width breakpoint,
   because the
  breakpointless wrapping `FlexboxLayout` root created a Slint `Flickable` layout-info loop on Slint 1.17;
   see `../../../doc/troubleshooting/slint-flickable-flexbox-layout-loop.md`.
   Each scroll region except the folder picker is driven by a prominent custom
  scrollbar in a right-hand gutter,
   since the default
  std-widgets scrollbar is a near-invisible hairline.
   The folder picker has no scrollbar on either input (D125):
   the selected folder, the highlighted letter of the rail and the order of the list already say where the user is,
   and the wheel, a touch drag and the keys still scroll it.
   Scrolling is animated by the
  Flickable itself:
   touchpad and touchscreen gestures fling with momentum,
   and mouse-wheel notches ease in over a
  short duration rather than snapping.
   The wheel animation is why the app depends on Slint 1.17 or newer (it is absent from
  the 1.16 releases);
   see `../../../doc/troubleshooting/slint-flickable-smooth-scroll.md`.
   The track list shows one folder's rows:
   a track row has a cleaned title and a supporting line,
   and a subfolder header labels the tracks under it (D86),
   so deeper nesting stays visible.
   The highlighted row is the currently selected track, playing or paused,
   and the transport deck names it (folder and title).
   A single click on a row other than the highlighted one selects it and loads it paused (D124),
   stopping whatever was playing, or also starts it when the Auto-play on select setting is on (D129);
   a click on the already-highlighted row toggles play and pause in both modes.
   A Search result for a track follows the same rule (D127): a result that is not the current track selects it (and
   starts it with the setting on), and the result of the current track toggles.
   The Play item of the track menu starts a row at once with the setting on or off.
   `src/row_activation.rs` holds the one pure rule, `row_activation_for`, used by `ui_wire::play_row` (click, Enter and
   Search result) and by `play_row_now` (the menu Play, which always plays).
   A row whose file cannot be opened stays selected and paused, is reported once, and the next click on it tries to
   load it again (D128); Next, Previous and the end of a track still walk past unreadable files.
   No double-click handling exists or is needed.
   The window title shows the playing track's title (the one the deck shows, without folder, extension or the prefix
  its folder's files share, D86) while audio plays,
   and reverts to "Music Player" when paused.
  The custom controls take their colours from the `Theme` roles in `ui/theme.slint`
   rather than hardcoded values,
   so they follow the OS accent colour
  and the light/dark theme (see "Theme colours").
  Seek-bar movement is debounced through the Rust helper in `src/progress.rs` (a tick at most every 250 ms,
   while a seek,
   a new track and play or pause state still update immediately),
   which keeps sub-second or zero-duration tracks from flickering the on-screen progress bar.
   The taskbar progress does not use that helper;
   see "Taskbar progress".

## Windows

Windows has two builds of the same program, selected by the Rust target.
Both use WASAPI through cpal, the same winit window, and libopus 1.6.1 built from source with CMake.
The linker is never lld: `.cargo/config.toml` sets no linker for the MSVC targets, and pins GNU ld for the GNU target.
Why, and what was measured: `doc/decision/music-player-windows-linkers.md`.

### MSVC build (`x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc`)

This is the default build.
Visual Studio's own `link.exe` links it; rustc finds the Visual Studio installation through vswhere.

- Install Visual Studio 2022 or newer, or the Build Tools, with the "Desktop development with C++" workload
  (the MSVC compiler, `link.exe` and a Windows SDK).
  On Windows on Arm also install "MSVC ARM64 build tools" and build natively on that machine.
- Install `rustup` with the MSVC host, the nightly pinned in the repository `mise.lock`, and `cmake` (4.x is fine).
- No LLVM and no PATH change are needed;
  earlier versions of this package required LLVM's `bin` on PATH for `lld-link.exe`.
- Run `mise run //package/music-player/desktop-app:build` (release) or `:test` (needs `cargo-nextest`).
  The output is `target\release\music-player.exe`.

### GNU build (`x86_64-pc-windows-gnu`)

This build needs no Microsoft toolchain.
The `gcc` driver runs GNU ld from MinGW-w64,
and the program imports only system DLLs; it needs no MinGW runtime DLL next to it.

- Install MSYS2, open its UCRT64 shell, and run `pacman -S mingw-w64-ucrt-x86_64-gcc`.
  That package brings GNU ld, `windres` and `dlltool`.
  Put `C:\msys64\ucrt64\bin` first on the Windows PATH,
  before Git for Windows and Strawberry Perl, which also ship a `gcc`.
- Install `rustup` with the GNU host, so that build scripts are not linked by Visual Studio either:
  `rustup toolchain install nightly-YYYY-MM-DD-x86_64-pc-windows-gnu` with the date from `mise.lock`, and make it the
  default or set `RUSTUP_TOOLCHAIN` to it.
- Install `cmake` and `ninja` (the Windows builds of both work with the MSYS2 gcc);
  `opusic-sys` uses the Ninja generator whenever `ninja` runs.
- Run `mise run //package/music-player/desktop-app:build:windows-gnu` or `:test:windows-gnu`.
  The output is `target\x86_64-pc-windows-gnu\release\music-player.exe`.

UCRT64 is the MSYS2 environment its documentation recommends,
and the build links the Universal C Runtime that Windows 10 and later include.
The MINGW64 environment (the msvcrt runtime that the Rust platform-support page names as the target default)
also builds and passes the tests, but MSYS2 deprecated it on 2026-03-15.
Both were run on GitHub Windows runners; see the decision record.

### Building the GNU target on Linux

`mise run //package/music-player/desktop-app:image:windows-gnu` builds a CachyOS image with mingw-w64 and Wine
(`Containerfile.windows-gnu`, unpinned like every container base image of the repository).
`:build:windows-gnu` then cross-compiles the program,
and `:test:windows-gnu` cross-compiles the tests and runs them under Wine.
Both run in a container capped at 6 GiB of memory and 4 CPUs.
This proves the GNU toolchain and most of the code; it is not a Windows run.
The cross build has no icon (the icon is embedded by a build script that runs only on a Windows host),
and six tests fail or hang under Wine that pass on the Windows runners;
`doc/troubleshooting/music-player-windows-build-and-test.md` lists them with what was and was not found.
`nextest.windows-gnu.toml` ends a hung test after five minutes.
To start the program under Wine, use a virtual desktop (`wine explorer /desktop=mp,1280x800 music-player.exe`):
on a bare Xvfb screen Slint's frame throttle divides by a refresh rate of 0 and the program exits.

### Continuous integration

The `test-windows` job of `.github/workflows/music-player-release.yml` builds, tests and starts the program
on `windows-2025` (x64) and `windows-11-arm` for the MSVC targets and on `windows-2025` for the GNU target.
It runs on a release or a manual run of that workflow, and a failure blocks the release.
It prints the linker rustc ran, so a `lld` link would fail the job, and the PE import table of the program.
It reads the console screen buffer and log files back to check that no ANSI escape sequence lands as text,
and reads the startup failure message box back through Win32 calls
(`doc/decision/music-player-windows-gui-subsystem.md`).
Each job caches the cargo registry, the git database and the target directory.

## Taskbar progress

The taskbar bar (KDE `LauncherEntry` on Linux,
 `ITaskbarList3` on Windows,
 nothing on macOS) means one thing: where the current track is,
 as a whole percent.
`src/launcher/taskbar_state.rs` (`TaskbarProgress`) is a state machine that `src/ui_progress.rs` feeds with every engine
message,
 never with the window's own position and duration,
 so a new track resets the position and sets its length in one step.
It answers with the next value to send,
 or with nothing when the bar already shows the right thing.

- The bar is shown from the moment a track's bar has been used (playback started or the position moved),
  and it stays shown while paused,
  so pausing, or choosing another row, does not hide and show it.
- The bar is hidden when no track is loaded,
  when the track has no usable length (a zero or missing duration),
  when a track is loaded but has not been played or moved since the app started,
  and when playback has stopped within half a second of the end of the track.
- The value is a whole percent sent as exactly `percent / 100`,
  because Plasma shows `qRound(progress * 100)`.
- While playing,
  a change of the percent is sent at most once every 250 ms,
  so a short track or a dragged seek bar cannot flood the bus,
  and a held change goes out with the next playback tick.
  Pausing, resuming, the first seek after ordinary playback, a new track, going idle and every change while paused
  are sent at once.
- Within a track the percent never moves backward except through a seek.

## Isolated visual verification

Never change the host desktop environment's global theme to test this app.
Run theme-dependent and other isolated GUI checks inside the repo-owned compositor at
`package/nested-wayland-session/cli`.
If that compositor cannot provide a required test feature,
add the feature to `nested-wayland-session` rather than mutating host-wide desktop settings.
Restore any app-local fixture state after capture.

## Theme colours

The operating system's accent colour is the theme root (A3, D108), and it is the only thing taken from the platform.
Slint reports it through `SlintInternal.accent-color`: the XDG portal `org.freedesktop.appearance` `accent-color` on
Linux (live, through the portal's `SettingChanged` signal), DWM colorization on Windows and `NSColor` accent on macOS.
A transparent accent (alpha 0) means none is available, and the root is then the Fold capture baseline `#4E5E8B`.
`SlintInternal` needs Slint's experimental-features switch (`SLINT_ENABLE_EXPERIMENTAL_FEATURES`), which the package
`mise.toml` and the repo `.cargo/config.toml` already set.

Every other colour is derived from the root in OKLCH by `src/theme_roles.rs` (math in `src/oklch.rs`):
each role has a lightness target, a chroma share and a hue offset, converted to sRGB with chroma reduction at constant
lightness and hue (a binary search, never per-channel clipping, which would shift the hue).
Text roles are then moved in lightness until WCAG AA holds on every surface they are drawn on and under the hover layer,
so contrast holds for any accent.
The canvas stays pure black in dark and pure white in light (D41, D34), and the error roles are constants that follow no
accent. Every gray (deck, rows, surfaces, outlines, text variants) carries the root hue at its own tint
 unless the root is
completely neutral, meaning its three 8-bit channels are equal (D118); an accent role is never less chromatic than the
strongest gray of its scheme, except under a completely neutral root, where every accent is gray.
Hover and pressed fills of the selected segment and the search match fill (D59) are OKLCH roles too;
state layers over a surface stay alpha-only.
The Rust side (`src/ui_theme.rs`) writes both schemes into `Theme.dark-roles` and `Theme.light-roles` at startup and
again whenever the accent changes; the light or dark choice stays inside Slint.
The literals in `ui/theme.slint` are the baseline roles, so a Slint viewer without Rust still renders the baseline;
a test fails when they differ from the derivation and says what to paste.
No `.slint` file may call `mix`, `brighter`, `darker` or `transparentize` (a test scans for them).

## Settings page

Settings (`ui/settings.slint`, D87, D99, D109, D110, D129, D132) lists the two templates, Track rows and Playing track,
under a Templates heading, the Auto-play on select switch row under a Playback heading,
and the Licenses row under an About heading (D132; the row opens the Licenses page of `ui/licenses.slint`).
The whole page is drawn on the one canvas colour (pure white in light, pure black in dark): no page tint, card fill,
header band, border or divider line, so the header, the heading and the rows are separated by spacing alone.
Only state carries a fill: the hover and pressed layers, the 40 px icon circle (the secondary container role) and the
selected row's pill.
`src/settings_layout.rs` holds the layout numbers as pure functions,
and the window tests compare the real layout with them.

The window decides the form with the same 600 px switch as the player.
Below 600 px Settings is one full-width page with no maximum column width and no centering:
a 16 px side margin, rows 72 px tall with a 40 px circular icon at x = 32 px from the window edge and the title at
x = 84 px, 2 px between rows and 16 px above the group heading, and the header with Back.
A tap on a row opens that template's editor page, and Back on the editor page returns to the list.
From 600 px Settings is two panes.
The list pane takes the default split ratio 0.3636 of the window width, rounded to a pixel, and the editor pane takes
the rest, with only the ordinary 16 px insets (a desktop has no crease).
The left pane is the list above, and the right pane shows the selected row's editor in place, so selecting never
navigates and nothing moves: the selected row is a filled pill in the secondary container role with a bold title (two
channels), the title's box is sized by a hidden bold copy, and the new content fades in over 150 ms.
Slint 1.18 exposes no reduced-motion setting (searched in its 1.18.1 crates), so the fade is always that short.
While Settings is open the player layouts are hidden, so nothing of the player shows through the fade.
The first row is selected when Settings opens.
Up, Down, Home and End move through the rows (selection follows in the wide form, and the single form shows the focused
row with the focus ring), and Enter or Space activates the focused row.
Back (the header arrow) and Escape close Settings in both forms.
Escape closes Settings only while Settings is the visible page and no template line is being edited.
Closing Settings, or selecting the other row, applies and saves the valid drafts.
Widening the window on an editor page continues that editor in the right pane.

The Playback group is one switch row in the same 72 px form: the `autoplay` icon circle, the title Auto-play on select,
one supporting line that states what selecting does, and a Material 3 switch (52 by 32 px, a 24 px thumb).
Click, Space and Enter toggle it in place and save `auto_play_on_select` at once; it opens no editor, is never a
selected row, and in the wide form the detail pane keeps the selected template's editor.
The state is shown by the thumb position, the track fill (the accent when on, the canvas with an outline border when
off) and the check or cross mark in the thumb.
The keyboard position reaches the row as the last stop of the list (Down and End), with the focus ring of the group
corners.
Under 300 px of row width, which is the list pane of a window between 600 and about 910 px, the switch sits under the
supporting line and the text wraps; `AutoPlaySwitch` in `ui/settings.slint` draws the switch for both places.

The row icons are Material Symbols Outlined (weight 400, grade 0, 24 px, unfilled) `queue_music` and `play_circle`
from [google/material-design-icons][material-icons], commit `49d4db35df873165d6bd6ba09b063c7dafbac2f4`
(`symbols/web/queue_music/materialsymbolsoutlined/queue_music_24px.svg` and
`symbols/web/play_circle/materialsymbolsoutlined/play_circle_24px.svg`).
They are embedded as Slint `Path` commands in `ui/settings.slint`, with no icon font and no network access at run time.
The SVG files use a `viewBox` of `0 -960 960 960`; the embedded data is the same outline moved onto a 24 by 24 grid
(x / 40 and (y + 960) / 40) in explicit absolute `M`, `L`, `Q` and `Z` commands, with a matching `viewbox` of 24.
The files close each contour with a line back to its start before `Z`, which Slint's femtovg renderer draws as a wrong
shape (the three bars of `queue_music` became one blob), so that redundant line is dropped; a test pins the form of the
data.
The icons are licensed under the Apache License 2.0.
The text is `LICENSES/Apache-2.0.txt` in this package, next to the font license, and the same file sits at the
repository root.

[material-icons]: https://github.com/google/material-design-icons

## Text size

Settings has a Text size group, between Playback and About, that sets the pixel size of four texts of the desktop
window, one size for each:

- the pane titles, the title of the left pane and of the right pane (default 20 px);
- the group heading letter, which heads a group of folder names in the picker (default 20 px);
- the folder names, in the picker (default 14 px);
- the track titles, in the track list (default 14 px).

The defaults are the sizes the player drew before the setting existed, so nothing changes until a stepper is pressed.
Each text has one row with a minus button, the size with its unit, and a plus button.
A press changes the size by 1 px, never below 10 px or above 32 px, shows at once in the live window, and saves the
size to the session file.
At an end of the range the button that would go past it is outlined instead of filled, draws its glyph 1 px thick
instead of 2 px, and does nothing, so the end shows by fill, weight and color together.
Each button is a 48 by 48 px target.
In the keyboard model every stepper row is a stop of the Settings list: Left and Right on the row under the ring step
its size, and Enter and Space on it do nothing.
The Reset text sizes row puts all four sizes back to their defaults with one save; at the defaults it is faded, says so
in its line, and does nothing.
In a row narrower than 372 px, which is a window under about 404 px or the list pane of a window between 600 and
about 1110 px, the stepper sits under the name.
The list scrolls the row the keyboard reaches into view, with 8 px of margin, because the group makes the list longer
than a 600 px window.

The four sizes are four flat keys of the session file (`text_size_pane_title_px`, `text_size_group_heading_px`,
`text_size_folder_name_px` and `text_size_track_title_px`).
A missing key, which is what a session file written before the setting has, is the default of that text and logs
nothing.
A key that is not a whole number from 10 to 32 falls back to the default of that text alone, with a warning that names
the key and the value, and never costs the other three sizes or the rest of the file.
Saving a size reads the file, changes the four sizes and writes it whole, so the playback fields, the templates and the
Auto-play on select flag stay as they were, and the shutdown save keeps the sizes.
`src/text_size.rs` holds the limits, the defaults and the step rule.
`ui/text_size.slint` holds the `TextSize` global that the five text sites read: `ui/bars.slint` for the two pane titles,
`ui/folders.slint` for the heading letter, the folder names and their hidden width probe, and `ui/tracks.slint` for the
track titles.
`src/ui_text_size.rs` writes the global from the session and handles the presses.
The folder name and its width probe read the same size, so selecting a name, which draws it heavier, never re-wraps the
names.

A larger size makes room instead of overlapping: a track title line grows from 24 px with the size (1.3 times the size
from 19 px up), the row keeps its 72 px pitch with a second line and its 48 px pitch without one, and the group heading
row grows from 38 px with the letter.
A title wider than the pane still ends in an ellipsis.
Android is not part of this setting: it follows the system font size.
`ui_text_size_window_tests.rs` checks the steppers, the limits, the saves and the layout at 10 and at 32 px in both
forms, and the `text-size` session of the `desktop-e2e` package presses the steppers with real keys across a restart.

## Search and scale

A library of 1,000 artists and 1,000,000 tracks is normal operation, so Search, the scan and the window projection are
built to cost the size of what is shown or typed, not the size of the library.
This section describes the pieces; the decisions behind them are D47 to D74 (Search behavior), D103, D111 (Search while
the scan runs) and D112 (the in-memory index).

### The Search page

`ui/search.slint` draws the page (D126 restyled it to match the Android Search page; the specification is in
`design/decisions.md`).
The header is the canvas color, 72 px high, with a 1 px `outline-variant` divider under it.
A result row is flat on the canvas and 72 px high.
Its rounded shape (radius 21.3 px) starts 8 px before the content and ends 8 px after the end inset, and bounds the
hover and pressed layer, the keyboard ring pair and the pointer target.
The glyph sits in a 40 px `secondary-container` circle in the 48 px slot, so the circle, the Back arrow and the title
keep their D58 columns.
The match highlight keeps square corners (D59).
The five empty and no-result states (empty query, no match, still scanning, unavailable, empty inventory) share one
block that starts 72 px below the divider: a 72 px `secondary-container` circle with a 32 px magnifier, the title in
`title-medium` at weight 500, and the support line.
The cover centers the block and the inner panel aligns it to the content start; a window shorter than the block scrolls
it with the wheel.
`src/ui_search_restyle_tests.rs` pins these numbers on the real window in both schemes.
The result list's scrollbar track starts where the rounded shape ends.

### The Search index

`search_index.rs` holds one entry per track: the display path and the normalized title, each in one shared text buffer
with end offsets (`search_index_text.rs`), and the title's length in UTF-16 units.
Two buckets, by first character and by first two characters of every word, list the tracks a term can match.
A query takes the smallest bucket among its terms, tiers each member with an allocation-free byte search at word
starts, keeps the best five hundred in the order the replaced matcher used (tier, length, collation key, path, folder
before track, position), and only then builds rows with highlights and parent context.
`search_index_rank.rs` computes collation keys lazily, only for the entries tied at the cut.
A bucket of 32,768 members or more is tiered by four threads and joined in order, which returns the same list.
The top five hundred of every first character is cached per character, and the cache is merged with the tracks
appended since, so it is exact at every moment (`search_index_top.rs`); an answer is never stale.
Normalization is unchanged: NFKD, combining marks removed, lowercase, trimmed by the Kotlin whitespace rule.

`search_service.rs` owns the index behind a read-write lock and three threads.
The query thread answers the newest query and drops the ones typed over, and never runs on the window thread.
The stream thread appends scan batches, and the warmer thread refreshes the one-character caches after a quiet moment.
Batches are normalized and bucketed outside the lock, and only the push runs under the write lock, which waits for
readers in short polls instead of blocking a keystroke.
Every batch signals a refresh, throttled to one answer every 120 ms, so an open Search page updates in place.

The old matcher is kept as `search_results_oracle.rs` (test builds only) and is the oracle for the index.
Tests compare tier, kind, name, library index, count, truncated flag and highlight ranges on hostile queries, on
seeded random libraries, on libraries that grow between batches, and on generated libraries.

### Searching while the scan runs

`scan_stream.rs` walks the root like `expand_paths` and reports every 5,000 tracks (the first after 150 ms) as
`Update::ScanBatch`, which `main.rs` hands to the stream thread.
Search therefore answers from the first batch, and results arrive in place while the scan continues.
`apply_answer` in `ui_search_apply.rs` keeps the query, the caret and the scroll position, edits one rows model in
place, and anchors the list (`ui_search_anchor.rs`) so the first visible row keeps its place when a better match is
inserted above it.
A tap resolves its row in the list last shown, not in a new search.
A query that matches nothing says the library is still being scanned until the scan completes (D69).
Browse rows still wait for the whole walk, because pages need every name.

The walk uses the file type of each directory entry and calls `is_file` only for symlinks.
A `stat` per file made the walk of 1,000,000 files take 10.6 seconds instead of 1.4 on a warm cache, and 55 seconds on
the one cold run measured.
The file watcher registers its recursive watch on its own thread (28 to 45 seconds at 1,000,000 files, where it used to
run before the walk on the engine thread) and ignores open and close events, which the walk itself raises.
When a registration ends it asks for one full rescan, which catches a change made while it ran.
The queue itself takes its names and page index from one pass each: `Queue::set_tracks` went from 2.2 seconds to 0.5
at 1,000,000 tracks.

### The window projection

`ScreenState` keeps the names as one shared list and a page index (`page_index.rs`) built once per queue generation.
A projection reads only the selected page, looks up measures only for that page, and shares names instead of cloning
them.
The sweep reports measures in chunks that are written in place; a projection is requested only when a chunk touches the
selected page, and at most four times a second.

### Numbers

All numbers are release builds on a shared 16-core host whose load average stayed near 40, so they are upper bounds.
At 1,000,000 tracks the index builds in about 0.2 seconds from the names and holds 136 MiB, the slowest push under the
write lock is about 4 ms, and a query of one letter takes about 2 ms.
Queries of two to eight characters take 5 to 8 ms on the generated library, and 4 to 8 ms on 1,000,000 realistic names.
Typed into the running app at 1,000,000 tracks while the scan was still running, a keystroke took 4 to 9 ms from the
key to the applied rows (compute 4 to 7 ms, apply under 1 ms).
The window projection of a page of 970 tracks takes about 3 ms.
A page of 30,000 loose tracks takes about 80 ms, because a projection is linear in the selected page, at about 2.8
microseconds per row; the row template is parsed once per template and not once per row.

### Tests at scale

The scale tests run in the normal `mise run //package/music-player/desktop-app:test` suite, including 1,000,000 tracks.
`scale_library_tests.rs` generates the library; debug builds repeat a 100,000-track base so the suite stays short.
`search_scale_tests.rs` times the index and `scale_tests.rs` times the window stages, each at 1,000, 10,000, 100,000
and 1,000,000 tracks, and both print the median with the minimum and maximum of five runs in release builds.
Run them with `--release` and `--no-capture` to read the numbers.

## Keyboard

The keyboard map and the focus model are decision D106 (extending D102) in `package/music-player/design/decisions.md`.
`src/keymap_bindings.rs` holds the whole table (`BINDINGS`) and the pure classifier in `src/keymap.rs` reads it, so the
key hint badges, the tooltips and the menu rows draw their texts from the same rows.
Control is Ctrl on Windows and Linux; on macOS the primary modifier is Command (`held_modifiers` in `src/ui_keys.rs`
swaps the two because Slint reports them swapped there).

### Bindings

Playback, anywhere on the player page (Space also works with the track list, the folder list or a button focused;
on a focused button Space and Enter press that button instead):

- Space toggles play and pause.
- Ctrl+Left and Ctrl+Right go to the previous and the next track (macOS: Command+Shift+[ and Command+Shift+]).
- Alt+Left and Alt+Right seek 10 s back and forward; with Shift, 60 s. A held key repeats.
- Ctrl+M cycles the playback mode; Ctrl+Shift+1 to 4 pick Repeat, In order, Shuffle folder and Shuffle all.
- Ctrl+Home restarts the track.

Browsing:

- Ctrl+F opens Search; double Shift opens it too (two Shift taps, the second released within 400 ms of the first
  press, on the player page only).
- Ctrl+Alt+S opens Settings (macOS: Command+,).
- Ctrl+O opens the folder picker (on the folded layout the picker overlay; on the two-pane layout it focuses the
  folder list); Ctrl+Shift+O asks for the system folder dialog.
- Alt+1 and Alt+2 focus the folder list and the track list.
- Alt+Up and Alt+Down show the previous and the next folder; Ctrl+G reveals the playing track (macOS: Command+L) and
  Ctrl+Shift+G shows the playing track's folder.
- Ctrl+Q quits. There is no key for refreshing the library or for measuring a track again: both happen by themselves
  (D113).

In a focused list:

- Up, Down, Home, End, PageUp and PageDown move the ring; Enter selects the row, and on the selected row it plays or
  pauses it (D124), or shows the folder.
- In the folder list a letter jumps to the first folder starting with it, and Left and Right move like Up and Down.
- On a track row: Menu or Alt+Enter opens the row menu (Shift+F10 is not used, it is Run in JetBrains), Delete moves the
  file to the trash, Ctrl+Shift+C copies the file name, Ctrl+Shift+E shows the file in the file manager. File details
  has no
  key of its own; it is a menu row.
  While the menu is open Up, Down and Enter choose a row, Escape closes it, and the access letter of a row runs it and
  closes
  the menu (D114). Every row draws its letter underlined: P Play, S Start shuffle from here, D File details, F Show in
  file
  manager, C Copy filename, M Move to trash. The letters are one table in `src/track_menu.rs`.

Pages:

- Escape closes the innermost thing: the track menu, the folder picker overlay, then the page (Search, Settings).
- In Settings the template line's first Escape returns to its list row and the second leaves Settings; on the
  single-form editor page Escape returns to the Settings list. Ctrl+Enter applies the template and Ctrl+Shift+Backspace
  resets it. Ctrl+F and double Shift do nothing in Settings.
- In the Search field bare keys are text; Up and Down still move through the results and Enter opens one.

### Focus model

Tab and Shift+Tab walk the folder picker (the Open pill; on the folded layout the folder trigger first), the folder and
track lists, the deck controls (seek bar, Previous, Play, Next, mode group), then Search and Settings, and wrap; no
element traps the keyboard. Every focusable element draws the focus ring and nothing else (D117), only while the
keyboard is in use: a key press turns the ring on and a pointer press turns it off. The template text field shows it
 for any focus, as a browser does for a text input; the Search query field has no ring of its own,
 because its caret already shows where the typing goes.

The ring is the pair Firefox paints for `:focus-visible`, hugging the element instead of its bounding box:
 a 2 px ring in
the `primary` role flush against the element's edge and a 1 px ring in the `on-primary` role around it, so the ring is
readable on any surface. Both follow the element's own shape: each corner's radius grows by the widths inside the ring,
so a pill stays a pill, a circle a circle, and a rounded rectangle keeps its curve; a square corner becomes a 2 px one.
One component draws it (`FocusRing` in `ui/focus.slint`; its widths, gap, colors and square-corner radius are named
parameters in `FocusRingStyle`), placed where the element is and given its radius. It reaches 3 px outside the element,
never moves content, and sits above any sibling that could paint over it. Focus adds no fill, tint, state layer, bold or
size change: hover and pressed keep their own state layers (`ui/state_layer.slint` has no focus state).

One exception: a row as wide as its pane (a track row, a template field row) has no room outside it,
because its pane ends at the window's edge, so its two rings are drawn on the row's own edge, inside it (`inside` on
the component). A Search result's ring is the same pair drawn inside the row's rounded shape, following its 21.3 px
corners (D126). The track menu's rows are as wide as the card, so their ring reaches 3 px past the card's edge; the card
is not clipped. A card pulled back against the window's edge has its ring cut by the window itself there.

The track menu card is the canvas color itself, pure white in the light scheme and pure black in the dark scheme with no
tint (D119), and is told apart from the page by a 1 px `outline-variant` outline.

### Key hint badges

Holding modifiers for 300 ms shows a small badge (the key letter) on each control that the held set triggers;
holding more modifiers narrows the badges to the chords of exactly that set; releasing hides them. The badges take no
pointer input and move no content; they are drawn from `Theme` roles.

### Known limits

- The end-to-end run presses the Menu key through the nested compositor (0.4.0 or newer); the window tests press it too.
- Ctrl+Shift+3 and 4 may be taken by the screenshot shortcuts of macOS, and the digit chords assume a layout where
  Shift+digit gives the usual symbol; neither is verified here, and the macOS and Windows key mapping is untested.

### End-to-end run

`mise run //package/music-player/desktop-app:test:e2e` builds the release binary, then hosts it in the nested
compositor that mise has installed (0.4.0 or newer; an older one is refused before the run starts) with scratch
XDG directories and a generated library (the 1k scale library plus tone tracks), and
presses every shortcut as a real key event. It plays into its own PipeWire null sink (checked by node id against
every output stream of the app, and stops at once if one links elsewhere), never touches another player process, and
prints one pass, fail or not-injectable line per shortcut. State is read from the app's `ui_state` log lines, from
screenshots and from files; `E2E_ONLY` (a check id or group prefix), `E2E_OUT` (default `target/e2e`, with
screenshots and `e2e-summary.json`), `E2E_TMP` and `E2E_LIBRARY_GENERATOR` tune it. Run it alone.
The runner itself is the `package/music-player/desktop-e2e` package (see its `README.md`);
the task only builds the release binary and calls that package's command from this directory.

## Environment overrides

`MUSIC_PLAYER_COLOR_SCHEME` forces the window's color scheme.
Set it to `light` or `dark`.
Leave it unset, or empty, to follow the operating system's scheme.
Any other value is logged and ignored, so the window follows the system.
Verification runs use it to capture both schemes without changing the host's theme.

`MUSIC_PLAYER_ACCENT` replaces the system accent as the theme root.
Set it to six hex digits such as `6060C0` or `#6060C0`.
Leave it unset, or empty, to follow the system accent.
Any other value is logged and ignored.
It sets `Theme.accent-override`, which previews and tests can also set (transparent follows the system).
Verification runs use it to capture several accents without changing the host's theme.

`MUSIC_PLAYER_SCAN_PAUSE_MS` makes the folder scan pause that many milliseconds after each reported batch.
Leave it unset for normal speed; any value that is not a whole number is ignored.
Verification runs use it to type into Search while the scan is still running.

`MUSIC_PLAYER_FORCE_STARTUP_FAILURE` makes a Windows debug build fail one startup step on purpose:
`window-system`, `window`, `event-loop` or `panic`.
Release builds and non-Windows builds do not contain the code that reads it, and any other value fails nothing.
The `test-windows` job of the release workflow uses it to drive the startup failure report (`src/startup_failure.rs`):
with no console and no stderr handle (a double-click start) the program shows a message box,
otherwise it prints the same text on stderr (the `panic` step prints the standard panic message).

## Typeface

The window draws its text in Inter Variable, bundled from `ui/fonts/InterVariable.ttf`
and registered by the `import` at the top of `ui/app_window.slint`,
so Latin text looks the same on every machine.
Inter has no CJK glyphs, so Japanese and other scripts still fall back to a system font.
Inter is licensed under the SIL Open Font License 1.1 (`LICENSES/OFL-1.1.txt`).
Text sizes are the logical pixel sizes in `ui/theme.slint`, and main text is 16px.
The template editor's code fields keep `Noto Sans Mono`.

## Folder navigation UX choices

Wide windows show the folder picker beside the selected folder's rows as two independent scroll containers,
so the picker and the track list scroll separately;
 narrow windows show one pane at a time with a folder button in the header.
 The picker lists every folder, so late-alphabet artists stay discoverable.

Rejected alternatives:

- Auto-collapse after each folder click:
   rejected because collapsing content above the track list changes the scroll geometry immediately after selection,
   which risks surprising jumps.
- Height-capped navigation plus `More folders`:
   rejected because hidden overflow makes late-alphabet artists such as Y and Z less discoverable.
- Text-link index:
   rejected because it weakens the button affordance.

## Library states and messages

Titles follow D86: a title is the file name without its extension and without the text that every file of its folder
starts with, cut back to a space or punctuation mark (`かめりあ(Camellia) - Aegis` shows as `Aegis`).
Nothing is removed for a single file, from `01 Song` and `02 Song` (they share only `0`), or when a title would be
left empty.
The deck heading and the window title use the same cleaned title, so the deck reads `folder / title` without the
folder twice.
Files and folders of a folder are listed in natural order (`Track 2` before `Track 10`, case ignored).
A library with one top-level folder shows that folder's own name; files directly in the opened folder sit in virtual
folders named by their first letter.

While a folder is being read the window shows "Reading your library" (not "No folder is open"), the deck is dimmed and
takes
no input, and the folder button opens nothing.
"No folder is open" appears only when neither a saved folder nor a music folder exists.
Play with nothing selected starts the first track of the folder on screen; with nothing to play it stays paused.

The library and the true peaks follow the disk by themselves (D113); no key, button or menu row refreshes or
re-analyzes.
The file watcher reports what changed under the opened folder after a short quiet time: the union of the paths its
events named, or "everything" when it lost events (an overflow of the kernel queue, an error), after each registration,
and when a folder is opened.
Reads of a watched folder are not changes.
A refresh costs what changed, not the size of the library: the named paths are looked at with one `lstat` each and
searched by binary search in the queue (`walk_order.rs`, `change_plan.rs`); a named folder has its direct entries
compared with the queue's, and a folder that is new or was recreated is read once.
Only an added or a removed file edits the list (in place, without a new load); a replaced file leaves the list alone and
is measured again when its size or modified time differs from the cache.
Only "everything" walks and refreshes the whole library, and so does a set of more than 100,000 distinct paths.
At most one refresh sweep runs; requests that arrive meanwhile merge into one follow-up that covers their union
(`refresh_coalescer.rs`).
The refresh sweep shows no progress and decodes no file twice, because the sweep of the load
 and the refresh sweeps share
the fingerprints they have taken.
A file that failed to decode has no stored result, so it is tried again at the next library change, whichever file
changed (every refresh sweep also takes the small set of files recorded as failed), and in every full refresh.
The engine reports each file once until a peak is measured for it (a folder opened again reports it again, which marks
its row again), and the row keeps its mark until a peak is measured for it.
The desktop notification is stricter: one per file for a run of the app (D122).

The selected track follows its file (`controller_reload.rs`): when the file that is playing is replaced, the new file is
opened first and swapped in at the same position (clamped to the new length), the player keeps playing, the peak and the
gain are worked out again, the seek bar takes the new length, and no message is shown; a replacement
 that does not open yet
changes nothing until the next event. A paused selected track keeps its position when the new file has it
 and starts again
from zero when it does not.

A file that cannot be decoded is skipped by Next, Previous and the end of a track, and its row shows "Can't be played".
A click, `Enter`, a Search result or the menu Play on such a row does not skip it: the row stays the selected row,
paused with nothing loaded, and the next click on it tries to load it again (D128);
if the file decodes by then it loads and plays, and the mark goes when a peak is measured for it.
The window draws no message for it.
On Linux the same report is one desktop notification titled "Cannot play a file", whose text is "`name` can't be
played." (several files collapse to a count that names the first) (D122).
It is posted once per file for a run of the app, even if the engine reports the file again, and it names no folder.
If no notification service answers, one warning goes to the log and nothing else is shown;
the row mark is the only sign.
macOS and Windows have no notification code yet (the analysis indicator has none either), so there the row mark is the
only sign of an unplayable file, and no message is shown.
When the selected file leaves the folder, the row disappears, playback stops, the elapsed time resets, a message names
the file,
and the view moves to the neighboring folder if the folder on screen is gone.
Undo of Move to trash finds the file even when the library was opened through a symbolic link
(the folder is resolved before the comparison), and says plainly when the file is no longer in the trash.
`Open folder` on Linux needs the desktop portal or `zenity`; with neither, a message says to install `zenity` or to
start the player
with a folder on the command line.

## Session

On exit the engine saves the Source Root path,
 the Selected Track path (if one was selected),
 the playback position,
 the playback mode,
 and the two saved template texts to a JSON file under the platform config directory (`$XDG_CONFIG_HOME/musicplayer`
 on Linux,
`~/Library/Application Support/dev.monochromatic.musicplayer` on macOS,
 the roaming AppData config directory on
Windows,
 all resolved by the `directories` crate).
The file also carries a `volume` field, always `1.0` as written, because no in-app control sets it (D43);
 a hand-edited value is still restored and multiplies the output gain.
It deliberately does not save the queue or a current index:
 the queue is re-derived from the Source Root,
 so only the root and the chosen track need persisting,
 and an older session file written by a previous version (carrying a queue and index) degrades gracefully,
 its unknown fields ignored and its missing ones defaulted.
The file also carries the four text sizes of the Text size setting (see that section).
Because `run` executes on the host,
 this is the real `~/.config/musicplayer`,
 persisting naturally across runs.
On launch,
 when no path
argument is given,
 the saved session is restored by re-scanning the saved Source Root to rebuild the queue from what is on disk now,
 reselecting the saved track by path when it still exists,
 and loading paused at the saved position.
 That launch rescan is the restore auto-correction:
 files added,
 removed,
 or renamed since the last run self-correct without a stale entry surviving.
 A command-line path takes precedence over a saved session.
 When the saved Source Root is missing or is no longer a directory,
 the user's music directory is restored instead (carrying the saved settings but no selection),
 so the queue is populated without playing.
 The directory is resolved from `XDG_MUSIC_DIR`,
 then the XDG
user-dirs file,
 then the `xdg-user-dir MUSIC` command,
 then the `Music` folder inside the home folder (D95: a library is always open),
 taking the first of these that is an existing folder;
 only when none exists does the window say that no folder is open;
 running on the host,
 these resolve directly,
 so no
bind-mount or `XDG_MUSIC_DIR` injection is needed.
 On macOS the `directories` crate resolves `~/Music`
directly,
 and the `xdg-user-dir` step simply falls through (that command does not exist there).
 On Windows the
`directories` crate resolves the Music known-folder (`%USERPROFILE%\Music`) directly,
 and the `xdg-user-dir`
fallback is compiled out entirely (it is gated `#[cfg(unix)]`).
