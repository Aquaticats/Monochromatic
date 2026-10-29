# The desktop music player runs on Alpine Linux as a dynamically linked musl build

## Status

Accepted 2026-10-14.

## Context

The requirement from the user:
"Our app must run on Alpine."
The app is the desktop music player,
`package/music-player/desktop-app` (Rust and Slint).
Before this decision its Linux build targeted glibc only.
Alpine differs from the Fedora target in four ways that matter here:

- It uses musl libc,
  not glibc,
  and ships its own Rust toolchain whose host triple is `x86_64-alpine-linux-musl`
  (`aarch64-alpine-linux-musl` on arm64).
- Packages come from apk,
  and a plain install has no systemd or logind,
  so nothing creates `XDG_RUNTIME_DIR`.
- No sound server runs by default:
  no PipeWire and no PulseAudio.
- No desktop portal and no notification daemon are installed by default.

Android is out of scope.
Everything below was run,
not recalled:
Alpine 3.24.2 (`docker.io/library/alpine`,
index digest
`sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6`,
the stable release on 2026-10-14),
rustc 1.96.1 from Alpine's own package,
podman 5.8.7 on an x86_64 host,
and the architecture-native GitHub runners
for x86_64 and aarch64.

## Decision

The Linux musl target is a dynamically linked build made with Alpine's own Rust toolchain,
with the `alsa-fallback` cargo feature on,
and it is guarded in CI on x86_64 and aarch64 by the `test-alpine` job of `.github/workflows/music-player-release.yml`.

### Static versus dynamic

A fully static musl binary is rejected.
Two measurements decide it.

The static link fails.
`RUSTFLAGS="-C target-feature=+aes,+crt-static" cargo build --target x86_64-alpine-linux-musl` compiles every crate
and then fails at the final link:

```text
// log excerpt: /home/user/temp/agent/alp/static-1.log, 2026-10-14
ld: cannot find -lfontconfig: No such file or directory
ld: cannot find -lfreetype: No such file or directory
ld: cannot find -lpipewire-0.3: No such file or directory
```

Alpine ships no static `libfontconfig.a`,
`libfreetype.a` or `libpipewire-0.3.a`,
and the `pipewire` crate links the
PipeWire client library,
which itself loads its SPA plugins at run time.

A static musl program cannot open a shared library even if the link had worked.
A four-line C program that calls `dlopen("libwayland-client.so.0")` printed:

- statically linked (`gcc -static`):
  `Dynamic loading not supported`
- dynamically linked,
  same source,
  same container:
  `loaded`

winit and Slint open `libwayland-client`,
`libxkbcommon`,
the X11 libraries and EGL with `dlopen`,
so a static binary could never create a window.
Alpine's `rustc` already defaults to a dynamic musl link,
so no flag is needed:
`-C target-feature=-crt-static` is the default there,
and the dynamic build is the one every measurement below uses.
The consequence is that the binary runs only on musl systems with the runtime packages listed below,
which is the requirement.

### Rust toolchain

Alpine's own `rust` and `cargo` (1.96.1) are used,
not a musl-hosted rustup toolchain.
The repository pins no toolchain file for this package,
Slint 1.18 needs a recent stable,
and the Alpine package builds natively on both architectures with no download from outside the Alpine mirrors.
Alpine's `rustc` has the host triple `x86_64-alpine-linux-musl`,
a different name from rustup's `x86_64-unknown-linux-musl`.

### Code changes the build and run revealed

Each was found by running the Alpine build or the app,
not by reading.

1. `gxhash` refused to compile without the `aes` target feature,
   so the first Alpine build stopped with `Gxhash requires aes and sse2 intrinsics`.
   A `[target.'cfg(all(target_os = "linux", target_env = "musl"))']` table with `-C target-feature=+aes` fixed it,
   and the same 11 inputs gave the same 11 hashes on x86_64 and aarch64 (under qemu).
   Main then replaced `gxhash` with XXH3-64 from `twox-hash` (`doc/decision/music-player-fingerprint-hash.md`),
   which picks its SIMD kernel at run time,
   so the merge dropped the musl table and the Alpine build needs no target feature and no special CPU.
2. The background sweep silently stayed at normal priority.
   musl's `sched_setscheduler` always returns -1 with errno 38 (ENOSYS),
   measured with a C program;
   the raw system call returns 0 in the same container.
   `src/measure.rs` now calls the system call directly,
   which has the same per-thread meaning on glibc and musl,
   zeroes `sched_param` (the struct literal does not compile on musl,
   which has extra `sched_ss_*` fields),
   and logs the OS error if the call fails.
   A test reads the scheduling policy back from the kernel.
3. Linux audio was PipeWire only.
   With no PipeWire daemon the engine logged `audio init failed; degrading to silent mode` and played nothing,
   and on Alpine no daemon runs by default.
   A new cargo feature `alsa-fallback` (off by default,
   so every other Linux build is unchanged) makes
   `output::Output` try PipeWire first and the default ALSA device second,
   through cpal and the existing
   `output_cpal.rs` (`src/output_linux.rs`).
   The Alpine build turns it on.
4. The window said nothing when no audio output could be opened.
   The engine now sends `Update::AudioUnavailable` and the window shows a message that names PipeWire and ALSA
   and says to restart the player.
5. `output_cpal.rs` failed the repository's `implicit_return` clippy rule once it compiled on Linux,
   so its five implicit returns are explicit.

One finding needs a prerequisite,
not a code change.
With fontconfig present but no font installed,
`fontique` panics (`NoMatch`) the first time text is laid out.
It failed 227 of the UI tests and the app at startup,
and it is not Alpine specific,
but a minimal Alpine has no font.
Installing any font fixes it (`font-dejavu` is the small one),
so the runtime list below includes one.

### Runtime packages

The list a person installs to run the app on a Wayland desktop:

```sh
apk add fontconfig font-dejavu pipewire-libs libgcc wayland-libs-client wayland-libs-egl libxkbcommon \
  mesa-egl mesa-gles mesa-dri-gallium alsa-lib
```

Why each is there is written beside the `runtime` stage of `package/music-player/desktop-app/Containerfile.alpine`,
and the `smoke` stage builds on exactly this set.
The release binary's direct dependencies (`ldd`) are `libfontconfig`,
`libasound`,
`libpipewire-0.3`,
`libgcc_s` and musl;
the rest are opened at run time.
Beyond this list a person needs a compositor,
and for sound either PipeWire with a session manager
(`apk add pipewire wireplumber`) or an ALSA device.
Optional:
`dbus` for a session bus (notifications,
appearance),
and `zenity` for the folder chooser.

### What was run in Alpine, with what result

The window,
input,
audio and notification checks ran inside Alpine containers with explicit podman bounds
(`--memory=3g --cpus=4`),
with the app hosted in this repository's nested compositor
(`package/nested-wayland-session/cli`,
built from source on Alpine,
headless,
`--gpu software`,
so Mesa's llvmpipe).
The library was invented tones,
never a person's own files.

- Start:
  the binary prints its normal startup logs.
  Without a display it fails with winit's message (`neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set`).
- Window:
  the compositor's screenshot shows the full UI at 1280x720:
  folder rail,
  track list,
  transport.
  A click sent through the compositor's control socket on "Open folder" showed the chooser message below.
- PipeWire:
  with `pipewire` and `wireplumber` started in the container and a null sink,
  the app's stream `music-player` (`Stream/Output/Audio`) was linked to the sink and running,
  and a recording of the sink's monitor held a 440.0 Hz tone for 8.4 seconds.
- ALSA fallback:
  with `PIPEWIRE_CORE` moved so the app's PipeWire connection fails,
  the log said
  `PipeWire is unavailable; trying the default ALSA device` then `audio output: the default ALSA device`,
  and the stream (`alsa_playback.music-player`) reached the null sink through `pipewire-alsa`,
  recorded as 439.9 Hz.
  A free-running `null` ALSA PCM (no clock) drains the ring buffer far faster than real time and the capture it
  produced held only zeros;
  the paced device above is the evidence for the ALSA path.
- No audio device at all:
  the app starts,
  logs one warning,
  and the window shows
  "No sound output is available,
  so nothing will be heard.
  Start a sound server such as PipeWire,
  or set up an ALSA
  sound card,
  then restart the player."
- Notifications:
  with no notification service the app logs one warning (`analysis progress indicator unavailable`)
  and runs.
  With the compositor's recording service (`--notifications freedesktop`),
  the Notify calls were captured:
  "Cannot play a file" for a corrupt file,
  and the "Analyzing true peak" progress from 0 of 3 to 3 of 3.
- Folder chooser:
  with no desktop portal and no `zenity`,
  "Open folder" shows
  "This computer has no folder chooser for the player to open.
  Install zenity,
  or start the player with a folder:
  music-player /path/to/music".
  After `apk add zenity` the click opened a real `zenity` folder chooser as a second window of the compositor.
- State on disk:
  the session (`session.json`) and the peak cache (`decisions.db`) are written under
  `$XDG_CONFIG_HOME/musicplayer`,
  with `XDG_CACHE_HOME` and `XDG_DATA_HOME` pointed at throwaway directories.
- The X11 path was not tested:
  the repository's compositor hosts Wayland clients only.

Unit tests inside Alpine:
`cargo nextest run --features alsa-fallback`,
1449 run,
1449 passed,
1 skipped,
on x86_64 and on aarch64 (native runner) at the final tree (run 38681970548).
The host run on glibc (`mise run //package/music-player/desktop-app:test`) passes 1448 with the same 1 skipped;
the one extra test on Alpine is the one in `src/output_linux_tests.rs`,
which exists only with `alsa-fallback`.
The notification tests need `dbus-daemon` (apk `dbus`) and the UI tests need a font.

### The compositor needed no change

`monochromatic-nested-wayland-session` 0.6.0 built unmodified on Alpine (musl,
both architectures in CI),
started headless without a GPU or a seat,
and offered every control the checks used:
`screenshot`,
`click`,
`windows`,
`notification-log`,
`ping`,
`quit`.
Its socket lives under `XDG_RUNTIME_DIR` like any other.
It needs `mesa-egl mesa-gles mesa-dri-gallium wayland-libs-server libdrm libxkbcommon` at run time.
No version bump is needed.

### Continuous integration

The `test-alpine` job of `.github/workflows/music-player-release.yml` runs as part of the release workflow,
on a push to main that bumps the version and on manual dispatch;
the release waits on it through the `checksums` job, so a failing test blocks the release.
It no longer runs on pull requests or on every push that touches the player
(until 2026-10-22 it was the standalone workflow `.github/workflows/music-player-alpine.yml`, which did).
Each architecture runs natively (`ubuntu-latest` and `ubuntu-24.04-arm`) and:

1. builds `Containerfile.alpine` (targets `build` and `smoke`),
2. runs the nextest suite with `--features alsa-fallback`,
3. builds the app and builds the compositor from source in the container,
4. runs `music-player-smoke` in the `smoke` image:
   three scenarios (no audio,
   ALSA,
   PipeWire),
   each asserting the
   first screen was drawn,
   no panic,
   a non-blank screenshot,
   and the audio line for that scenario.

### Caching

The guard caches,
and the cache was proven on GitHub,
not assumed.
`CARGO_HOME` (registry and git),
the app target directory and the compositor target directory are bind-mounted from `.ci-cache`,
restored with `actions/cache/restore` and saved with `actions/cache/save`
(both pinned to the repository's SHA `27d5ce7f107fe9357f9df03efb73ab90386fccae`).

- The key is `music-player-alpine-<runner arch>-<rustc version and a hash of the -dev package versions>-<hash of the
  two Cargo.lock files>`.
  The restore key is the same without the lockfile hash,
  so a changed lockfile restores the newest entry of the same architecture and toolchain and rebuilds only the
  crates that changed.
- Only pushes to main and manual dispatches save;
  a pull request restores and never saves.
  A run that restored the exact key saves nothing.
- Entries stay lean:
  `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=0` leave no incremental directories and no debug info,
  and a trim step deletes what the next run rebuilds anyway
  (the unpacked registry sources,
  and the outputs of the player,
  `truepeak-core` and the compositor).
  The trim took the x86_64 entry from 1,134,955,562 to 948,893,157 bytes.
- The family is listed in `.github/workflows/cache-prune.yml` (`music-player-alpine-<arch>`),
  so only the newest entry per architecture and ref stays.
  The largest part is the debug build of the `turso` crates (about 400 MB of static and shared libraries),
  so the entry stays above the 650 MB guide in `doc/decision/ci-caching.md`;
  the pair costs about 1.9 GB of the repository cap,
  which that document's budget now lists.
- The local equivalent is `mise run //package/music-player/desktop-app:test:alpine`.

Measured on branch pushes of 2026-10-15 (the repository cap is 10 GB):

- Cold,
  key missed on both architectures (run 38678771471,
  attempt 2):
  x86_64 16 min 50 s (nextest 9 min 25 s,
  compositor 5 min 19 s),
  aarch64 11 min 9 s (nextest 5 min 56 s,
  compositor 3 min 11 s).
  Entries saved:
  X64 948,893,157 bytes,
  ARM64 928,035,774 bytes.
- Warm,
  same lockfiles (the same run,
  attempt 3):
  both jobs logged `Cache hit for:` (exact key) and saved nothing.
  Restore took 18 s (x86_64) and 13 s (aarch64).
  x86_64 5 min 35 s (nextest 2 min 38 s,
  app 37 s,
  compositor 39 s),
  aarch64 5 min 11 s (nextest 2 min 35 s,
  app 37 s,
  compositor 34 s).
  Only the three path crates recompiled.
- Lockfile change (run 38677516601,
  before the trim step existed):
  one transitive dependency (`crossbeam-utils`) was bumped in the compositor's `Cargo.lock`.
  The exact key missed,
  the log read `Cache hit for restore-key:`,
  and cargo rebuilt only that crate,
  `crossbeam-channel` and the path crates.
  The bump was reverted afterwards.
- Merge of main with an unchanged lockfile (run 38678226939) was an exact hit on both architectures,
  x86_64 5 min 54 s and aarch64 5 min 12 s.
- The first dry run failed (run 38675162941) because the Ubuntu 24.04 runners ship podman 4.9.3,
  whose builder reads each line of a Containerfile heredoc as an instruction (`Unknown instruction`).
  The smoke script in `Containerfile.alpine` is therefore written with `printf`,
  one argument per line.

## Release artifact the release workflow should add

Implemented on 2026-10-15 as the job `build-desktop-musl` of `.github/workflows/music-player-release.yml`
(`doc/decision/music-player-releases.md` records the result and the measurements);
this section is the specification it was built from, kept as written.
Its job `build-desktop` builds glibc tarballs `music-player-<version>-linux-x64.tar.gz` and
`music-player-<version>-linux-arm64.tar.gz`
(`doc/decision/music-player-releases.md`).
Alpine needs two more assets from a job that runs once per architecture,
named to match that scheme:

- `music-player-<version>-linux-musl-x64.tar.gz` on `ubuntu-24.04`,
- `music-player-<version>-linux-musl-arm64.tar.gz` on `ubuntu-24.04-arm`.

Each archive has the layout of the glibc one
(`bin/music-player`,
the launcher file,
the icon,
`LICENSE`,
`INSTALL.txt`),
with an `INSTALL.txt` that names the Alpine packages and the musl requirement.
The workflow artifact is `asset-linux-musl-<arch>`,
so the existing `asset-*` download picks it up,
and the expected list in the `SHA256SUMS` job gains the two names.

Job steps,
with `permissions: contents: read, id-token: write, attestations: write` and the `VERSION` of the `detect` job:

1. Checkout with `actions/checkout`,
   `persist-credentials: false`.
2. Build the images from the repository's `Containerfile.alpine`:

   ```sh
   # the release job, per architecture
   context="$(mktemp -d)"
   podman build --target build --tag music-player-alpine-build \
     --file package/music-player/desktop-app/Containerfile.alpine "$context"
   podman build --target runtime --tag music-player-alpine-runtime \
     --file package/music-player/desktop-app/Containerfile.alpine "$context"
   ```

3. Build with the shipping profile and the feature.
   There is no `--target` option because Alpine's host triple is already the musl one:

   ```sh
   # the release job, per architecture
   mkdir --parents "$RUNNER_TEMP/cargo" "$RUNNER_TEMP/target"
   podman run --rm --memory=12g --cpus=4 --security-opt label=disable \
     --volume "$PWD/package/music-player:/src:ro" \
     --volume "$RUNNER_TEMP/cargo:/cargo" --volume "$RUNNER_TEMP/target:/target" \
     --env CARGO_HOME=/cargo --env CARGO_TARGET_DIR=/target \
     --env SLINT_ENABLE_EXPERIMENTAL_FEATURES=1 --workdir /src/desktop-app \
     music-player-alpine-build \
     cargo build --release --locked --features alsa-fallback
   ```

4. Check the libraries,
   not only the link.
   The `runtime` image holds exactly the packages listed above,
   so this fails if that list is short:

   ```sh
   # the release job, per architecture
   podman run --rm --security-opt label=disable \
     --volume "$RUNNER_TEMP/target/release/music-player:/usr/local/bin/music-player:ro" \
     music-player-alpine-runtime music-player --version
   ```

   The output must equal `music-player ${VERSION}`,
   as the glibc job checks.
   The release binary also passed `--version` and all three smoke scenarios in a local run;
   the job may add the scenarios by running `music-player-smoke` in the `smoke` image.
5. Package like the glibc job (fixed order,
   owner and timestamps),
   with the stage name `music-player-${VERSION}-linux-musl-${ARCH}`,
   and `ARCH` equal to `x64` or `arm64`.
6. Attest the tarball with `actions/attest-build-provenance`,
   then upload it with `actions/upload-artifact` under the name `asset-linux-musl-${ARCH}`.

The release binary measured 54 MB.
On a local run with 8 cpus the final crate compile (thin LTO) took 5 min 28 s after the dependencies were built.
A full cold release build on a 4 vCPU runner was not timed.
If the job caches `$RUNNER_TEMP/target` and `$RUNNER_TEMP/cargo`,
its key should start with `music-player-alpine-release-` and `.github/workflows/cache-prune.yml` then needs the
matching `FAMILIES` pattern,
because the existing Alpine pattern does not match that prefix.

The release notes should say:
needs Alpine 3.24 or newer on musl,
x86_64 or aarch64,
and one line to install:
`apk add fontconfig font-dejavu pipewire-libs libgcc wayland-libs-client wayland-libs-egl libxkbcommon mesa-egl mesa-gles mesa-dri-gallium alsa-lib`.
They should also say that without PipeWire the player uses the default ALSA device.

## Consequences

- The musl binary runs only on musl systems with the listed packages;
  a glibc binary does not run on Alpine and the
  reverse.
  Each architecture needs its own build,
  which the release job above does natively.
- Without a portal or `zenity` there is no native folder chooser.
  The app says so in the window and the folder can be passed on the command line.
- The ALSA fallback is behind a cargo feature,
  so the glibc Linux build is unchanged and still PipeWire only.
  Enabling the feature for glibc Linux is a separate decision (it adds `libasound` as a link-time dependency).
- Notifications and appearance need a D-Bus session bus,
  which minimal Alpine lacks;
  without one the app logs a warning and runs.
- `XDG_RUNTIME_DIR` must exist for Wayland and PipeWire.
  Alpine creates it with `elogind` or by hand:
  `mkdir -p /tmp/xdg-runtime && chmod 700 /tmp/xdg-runtime` and export it.
- `cargo clippy` on Alpine's stable 1.96.1 reports `manual_range_contains` in `src/oklch.rs`,
  which the repository's nightly host clippy (`mise run //package/music-player/desktop-app:lint:clippy`) does not
  flag:
  that task finished with no finding on 2026-10-15.
  The two compilers disagree and the behavior of the flagged code differs for NaN,
  so the Alpine guard runs tests,
  builds and the smoke test,
  not clippy,
  and the lint was not loosened.
- Not verified:
  real Alpine hardware,
  other desktops and compositors (only this repository's compositor was used),
  Alpine edge,
  postmarketOS,
  the X11 path,
  and Raspberry Pi class devices.
