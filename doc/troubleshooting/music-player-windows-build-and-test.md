# music-player on Windows: first test runs failed on libc, path separators, case folding, notify events and CI plumbing

Date:
 2026-10-15.
Scope:
 `package/music-player/desktop-app` built for `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` and
 `x86_64-pc-windows-gnu`, run on GitHub-hosted runners (`windows-2025`, `windows-11-arm`) and under Wine.
Decision record:
 [music-player-windows-linkers.md](../decision/music-player-windows-linkers.md).

The Windows test suite had never run before 2026-10-14.
Each section below is one cause found by the first runs, with the evidence and the fix that landed.
Most causes are in this repository's own code or CI, not in an external tool.
The external-tool findings are the `notify` Windows backend, GitHub's allowed-actions policy, GNU tar on Windows,
 `turso_sdk_kit`'s build script, and Wine.

## The lib-test build failed everywhere: `libc` is not a Windows dependency

### Symptom

Run 38668416002 (all four jobs) stopped before any test ran,
 because `src/refresh_cost_tests.rs` called into `libc`
 and the package depends on `libc` only for Linux and macOS targets.

### Root cause

`refresh_cost_tests.rs` measured the CPU time of the process with `libc::getrusage`.
The `libc` dependency sits under
 `[target.'cfg(any(target_os = "linux", target_os = "macos"))'.dependencies]` in `Cargo.toml`,
 so the symbol did not resolve on Windows.

### Fix

A `#[cfg(windows)] cpu_seconds` that reads `GetProcessTimes` through the `windows` crate (already a Windows dependency),
 and `#[cfg(not(windows))]` keeps the `libc` version.

## Eleven tests assumed Unix paths, symlinks and a case-sensitive disk

### Symptom

Run 38669400881 built on all four jobs and ran 1381 tests; the same eleven failed on every job, 1370 passed.

### Root cause and fix, by group

- A symlink assertion in `src/scan_stream_tests.rs` ran on Windows, where an unprivileged process cannot create the
  symlink: it is now `if cfg!(unix)`.
- `src/controller_refresh_flow_tests.rs` and `src/controller_flow_tests.rs` compared or built paths with a literal `/`:
  they now compare `/`-joined components and build paths with `.join("A").join("2.flac")`.
- `src/change_plan_tests.rs` and `src/walk_order_tests.rs` generated random file names that differ only by case.
  On NTFS (case-insensitive) two such names are the same file.
  `src/refresh_test_support.rs` probes once whether the temporary disk folds case (`folder_names_ignore_case`,
  a `OnceLock`) and `pick_stem` avoids a colliding stem. The Linux random sequences are unchanged.

## `Modify(Any)` from `notify` on Windows made every refresh request another refresh

### Symptom

Run 38670823208 failed five watch tests and run 38671794537 failed three, after the first fix.
The failing assertions expected only plain writes to the named files, and the events that arrived included
 `Modify(Any)` about the watched parent folder.

### Root cause

The `notify` 8.2.0 Windows backend maps every `FILE_ACTION_MODIFIED` to one unspecific kind:

```rust
// notify-8.2.0/src/windows.rs:448-449
FILE_ACTION_MODIFIED => {
    let kind = EventKind::Modify(ModifyKind::Any);
```

Linux and macOS report writes as `Modify(Data)` or `Modify(Metadata)`.
`Change::of_event` in `src/change_set.rs` treated every other modify kind as structural, so each Windows write made the
 refresh read the whole folder.

A second event came from a probe test added for one CI run on a GitHub Windows runner: `read_dir` of a watched folder,
 with no write at all, produced `Modify(Any)` for that folder and nothing else.
The watch filter in `windows.rs:284-290` lists file name, directory name, attributes, size, last write, creation and
 security, and does not list last access, so the mechanism that makes a plain read report a modification is not proven;
 only the measured event is.
The refresh itself reads folders, so with this event left in, each refresh would request the next one
 (inferred from the probe, not observed as a loop).

### Fix

`event_is_a_plain_write` (`src/change_set.rs`) treats `Modify(Any)` as a plain write on Windows only.
`folder_was_only_read_or_touched` drops an unspecific modified event about an existing folder on Windows only;
 real changes inside a folder arrive as their own events.
Both have unit tests in `src/change_set_tests.rs`; `src/watch_tests.rs` prints the event that arrived on failure and
 accepts the modified parent folder on Windows.

### Tradeoff

Dropping a folder's own `Modify(Any)` means a metadata-only change of a folder (for example an attribute change) does
 not request a refresh.
The app reads file names and file contents, not folder attributes, so nothing it displays depends
 on that event.

## A test was flaky: two sweeps of one load decoded a track twice

### Symptom

`measure::...two_sweeps_of_one_load_decode_a_track_only_once` failed once on the MSVC x64 job in run 38672700706
 ("2 vs 1") and once under Wine.

### Root cause

The non-quiet sweep peeks the cache when it starts and reports hits.
A thread that started late reported the track a second time, so the count depended on scheduling.

### Fix

The test waits for the first sweep's first progress report before it starts the quiet sweep (commit abd7619406).

## `startup_failure` from a workflow that uses `msys2/setup-msys2`

### Symptom

Run 38668138999 ended in `startup_failure` with no job started.

### Root cause

The repository's GitHub Actions policy is `selected` with `sha_pinning_required: true`.
`msys2/setup-msys2` is neither GitHub-owned, nor by a verified creator, nor in the pattern list:

```text
gh api repos/Aquaticat/Monochromatic/actions/permissions/selected-actions
patterns: pnpm/* cargo-bins/* google/* returntocorp/* ossf/* jdx/* changesets/*
```

### Fix

The workflow runs the MSYS2 that the `windows-2025` image already has at `C:\msys64` through
 `C:\msys64\usr\bin\bash.exe -lc 'pacman ...'`, and caches the pacman package downloads with `actions/cache`.
Check the allowed-actions list before adding a third-party action.

## GNU tar on Windows reads `C:` as a remote host

### Symptom

Run 38668190187 failed extracting the cargo-nextest archive with `tar (child): Cannot connect to C: resolve failed`.

### Root cause

`tar` from Git for Windows (GNU tar) treats `NAME:PATH` as `HOST:PATH`, so `--file C:\...` or `-C C:\...` names a host.

### Fix

The step changes into the target directory and passes relative paths to `tar`.

## `turso_sdk_kit` runs a bare `windres` for every non-MSVC Windows target

### Symptom

The cross build in the Fedora container stopped with "compile Windows version resource: No such file or directory".

### Root cause

The `turso_sdk_kit` 0.8.2 build script runs `windres` by that bare name for every Windows target that is not MSVC.
Fedora installs the cross tool only as `x86_64-w64-mingw32-windres`; MSYS2 on Windows has the bare name.

### Fix

`Containerfile.windows-gnu` links `/usr/local/bin/windres` to the prefixed tool.

## A fingerprint test failed on NTFS: file times have 100 ns resolution

### Symptom

After `main` added `peakcache::tests::fingerprint_feeds_path_size_and_mtime_to_the_shared_function`, run 38685877917
 failed that test on all three Windows jobs (`src\\peakcache_tests.rs:124`, `assertion \`left == right\` failed`).

### Root cause

The test set a modified time of 1,700,000,000 s plus 123,456,789 ns and expected the key for exactly that value.
NTFS stores file times in 100 ns ticks, so the file system returned 123,456,700 ns and the wrapper hashed that.
The file-system rounding is inferred from the passing result after the change, not read from a log of the left and
 right values.

### Fix

The test uses 123,456,700 ns (a multiple of 100) for both times, with a comment (commit 11adb30419).

## A workflow check compared the binary version with a fixed string

The `--version` step of `.github/workflows/music-player-windows.yml`
(moved into the `test-windows` job of `.github/workflows/music-player-release.yml` on 2026-10-22)
compared the output with `music-player 0.1.0`.
The merge of `main` that carried release 0.1.1 failed that step on every job (run 38684829641, after the cache
 saves), and release 0.1.2 would have again. The step now reads the version from `Cargo.toml`.

## The default Fedora mirror was about 50 KiB/s

The image build downloads hundreds of megabytes of packages (gcc, mingw, Wine).
`podman build --build-arg FEDORA_MIRROR=https://mirrors.kernel.org/fedora` replaces the metalink with that mirror;
 package signatures are still checked.
(Since 2026-10-29 `Containerfile.windows-gnu` is CachyOS and has no `FEDORA_MIRROR` argument; `pacman` takes its mirrors
 from the image's mirror list. The Test runner there is `wine`, because the Arch `wine` package installs no `wine64`.)

## Wine differs from Windows in ways the real runners do not show

These were seen in `mise run //package/music-player/desktop-app:test:windows-gnu`
 (cross-built GNU binaries run by `wine64` in `localhost/monochromatic/music-player-windows-gnu`, Fedora 44, Wine 11.0).
The same tests pass on the real Windows runners (1383 tests run, 1382 passed, 1 skipped, on all jobs in the green runs
 of that time; after later changes from `main` the Windows jobs run 1422 tests, all passed, 1 skipped).

The last full Wine run (fonts installed) ended with 1383 tests run: 1377 passed (2 flagged leaky), 4 failed, 2 killed,
 1 skipped, in 1855 s.

- Without system fonts in the Wine prefix, three `ui_focus_ring_pages_tests` panicked inside Slint
  (`systemfonts.rs:71`) in an earlier run. The image now installs the `wine-*-winefonts` packages and they pass.
- `controller_flow::tests::removing_the_selected_file_reports_it_and_stops_playback` fails at
  `src/controller_flow_tests.rs:288` (`assert!(!controller.playing)`), and
  `controller_measures::refresh_tests::flow::the_selected_track_follows_its_path_...` fails at
  `src/controller_refresh_flow_tests.rs:104` (`current_path().is_none()`).
  Both remove the file that the decoder holds open and expect the next rescan not to find it.
  Under Wine the rescan still finds it (consistent with a delete-pending file that stays listed while open;
   the listing behavior was not probed).
- `watch::tests::a_file_written_in_place_is_reported_by_its_own_path` fails at `src/watch_tests.rs:222` with
  "a change is reported: Timeout". Wine's `ReadDirectoryChangesW` is the suspect; not proven.
- `refresh_scale_tests::one_replaced_file_in_a_million_tracks_costs_milliseconds_not_a_walk` fails at
  `src/refresh_scale_tests.rs:95` with `Os { code: 2, kind: NotFound, message: "File not found." }` from
  `create_dir_all`. The cause was not probed.
- `ui_search_ingest_tests::an_answer_to_an_older_query_is_dropped` and
  `ui_search_ingest_tests::no_match_is_not_claimed_before_the_scan_is_complete` hang until killed (1668 s, SIGKILL).
  The sibling tests in the same file pass in under a second.
  At about 1650 s the test process had every thread asleep and its CPU time not growing
  (read from `/proc/<pid>/task/*/stat`), so it is blocked, not spinning.
  The 20 s `PATIENCE` deadline of `Scan::pump` did not fire, so the block is outside that loop.
  Both tests apply a hand-built `SearchState::NoMatch` answer. The cause was not found.
  Run alone, the first test hangs again at once: with a 5 s period and `terminate-after = 2` nextest reported
  `TIMEOUT [10.253s]`, so the hang is deterministic under Wine and not a contention effect of the full run.
- The first Wine runs passed `--config profile.default.slow-timeout...` to `cargo nextest run`, and the hung tests were
  never terminated (slow-test lines up to 1620 s, killed by hand). `cargo nextest run --help` documents
  `--config <KEY=VALUE>` as "Override a Cargo configuration value": it is Cargo's option, passed to Cargo, so nextest
  never saw it. The timeout now comes from `nextest.windows-gnu.toml` through nextest's `--config-file`, and the
  `TIMEOUT` result above shows that route ends a hung test.

### Launching the release exe under Wine

The cross-built `music-player.exe` (release, 43,460,608 bytes) imports only system DLLs and `msvcrt.dll` (no
 `libgcc_s`, `libwinpthread` or `libstdc++`), and its PE linker version is 2.46 (GNU ld), read with
 `x86_64-w64-mingw32-objdump -p`.
`wine64 music-player.exe --version` prints the version from `Cargo.toml`.

- Started directly on a bare Xvfb screen, the app scanned the 8-track fixture, created its window and then panicked
  with exit code 101: see the section "Slint divides by a refresh rate of 0 under Wine on Xvfb".
- Started inside a Wine virtual desktop (`wine64 explorer /desktop=mp,1280x800 music-player.exe /work/fixture`) the
  app stayed up for the 25 s observed, and `xdotool` listed a visible `Music Player` window of 488x634.
  No screenshot was taken, so what the window drew is not verified.
- The first start of a fresh Wine prefix took exactly 5 minutes (04:56:25 to 05:01:25) with the default Mono and HTML
  engine handling, which showed a `Wine Mono Installer` window, and 7 seconds with
  `WINEDLLOVERRIDES=mscoree,mshtml=d`. A round 5 minutes looks like a prompt timeout, not work;
  not investigated further.
- The app logged `audio init failed; degrading to silent mode` (`no default output device`) because the container has
  no sound device; audio under Wine is not verified.

A Wine failure that the real runners do not reproduce is not a defect of the Windows build.
Treat the real runners as the authority, and Wine as a cross-compile proof that the binary builds, starts, and passes
 the large majority of the suite.

## Slint divides by a refresh rate of 0 under Wine on Xvfb

### Symptom

The cross-built app, started on a bare Xvfb screen, scanned the fixture, created its window, and then ended with
 exit code 101:

```text
thread 'main' (344) panicked at .../i-slint-backend-winit-1.18.1/frame_throttle.rs:58:46:
attempt to divide by zero
```

### Root cause

`i-slint-backend-winit` 1.18.1 (`frame_throttle.rs:51-58`; the same lines are in `slint-ui/slint` master at commit
 60e27ac6dae1, read 2026-10-15) treats only a missing refresh rate as unknown:

```rust
let refresh_interval_millihertz = winit_window
    .current_monitor()
    .and_then(|monitor| monitor.refresh_rate_millihertz())
    .unwrap_or(60000) as u64;
...
std::time::Duration::from_millis((1000 * 1000) / refresh_interval_millihertz);
```

`winit` 0.30.13 on Windows returns `Some(mode.dmDisplayFrequency * 1000)`
 (`src/platform_impl/windows/monitor.rs:190`), so a display that reports a frequency of 0 gives `Some(0)`,
 `unwrap_or` does not replace it, and the division panics.
Wine's virtual monitor on Xvfb is such a display; real Windows monitors report a rate.
That Wine reports 0 is inferred from the panic, not read from Wine's source or a log of the value.
The panic was not seen on the real runners.

### Verification

- Harness: `wine-smoke`-style run in the image (Xvfb, `XDG_RUNTIME_DIR` set, `WINEDLLOVERRIDES=mscoree,mshtml=d`,
  `wine64 music-player.exe /work/fixture` for 25 s, debug build).
- Unpatched: the app exited early with code 101 and the panic above.
- With the one-line patch to a copy of the 1.18.1 registry source, applied as a Cargo `patch.crates-io` path override
  (`--config 'patch.crates-io.i-slint-backend-winit.path="/proto/i-slint-backend-winit"'`): the app was still running
  after 25 s and `xdotool` listed a visible `Music Player` window of 480x600.
- Both runs used the same fixture, image and script; only the patch differed.

### Verified workarounds

- Start the app in a Wine virtual desktop (`wine64 explorer /desktop=mp,1280x800 music-player.exe`), where the monitor
  reports a rate. Tradeoff: it is a Wine-only launch form and says nothing about the app on Windows.
- The patch, applied to the dependency (not to this repository):

```diff
--- a/frame_throttle.rs
+++ b/frame_throttle.rs
@@ -51,6 +51,8 @@
         let refresh_interval_millihertz = winit_window
             .current_monitor()
             .and_then(|monitor| monitor.refresh_rate_millihertz())
+            // A driver can report 0 for "the hardware default" (dmDisplayFrequency 0 or 1 on Windows).
+            .filter(|millihertz| *millihertz > 0)
             .unwrap_or(60000) as u64;
         let window_adapter = self.window_adapter.clone();
         let timer = Rc::downgrade(&self.timer);
```

  Tradeoff: a monitor that truly refreshes below 1 Hz cannot exist, so no real rate is changed; 60 Hz is only the
  fallback for an unknown rate, as it already is for a missing one.

### What does not work

- `SLINT_BACKEND=winit-software` does not avoid it: the throttle runs for every winit window.
- Patching Slint inside this repository was not attempted; the patch above stays a dependency-side proof.

## Upstream filing decision

### Slint

- `.out-of-scope/` has no entry for Slint.
- Is it upstream's fault: the unguarded division is upstream behavior; the trigger is a display that reports 0.
- Can upstream fix it: yes, with the one-line change above.
- Do they support the use case: `winit` and Windows are supported; Wine is not stated either way.
- Would the repo welcome a contribution: `CONTRIBUTING.md` on master asks for self-contained changes and says AI coding
  assistants read `AGENTS.md`; no ban on AI-assisted filings was found in `CONTRIBUTING.md`.
- Will they likely fix it: no signal against; the file's last commit is 2026-05-29.
- Prototype: done above, with before and after.
- Duplicate search on `slint-ui/slint` (issues and pull requests, open and closed) for `refresh_rate_millihertz`
  and `frame_throttle divide by zero refresh rate` returned no result on 2026-10-15.
- Result: all six constraints hold, so the draft below is fileable once a human has re-run the reproduction and
  corrected its disclosure line. It is not filed: filing is outside the scope of this task and the default policy is
  not to file, so the maintainer decides.

~~~md
Title: winit backend: frame throttle divides by zero when the monitor reports a 0 Hz refresh rate

Labels: bug, winit

`internal/backends/winit/frame_throttle.rs` `request_throttled_redraw` uses
`monitor.refresh_rate_millihertz().unwrap_or(60000)`, which only replaces a missing rate. winit's Windows backend
returns `Some(dmDisplayFrequency * 1000)`, so a display that reports a frequency of 0 (allowed by the Windows API for
"hardware default") gives `Some(0)`, and `(1000 * 1000) / refresh_interval_millihertz` panics.

Reproduction: a Windows build of a Slint app run under Wine on an Xvfb screen; the window is created, then
`thread 'main' panicked at .../i-slint-backend-winit-1.18.1/frame_throttle.rs:58:46: attempt to divide by zero`.

Suggested fix: `.filter(|millihertz| *millihertz > 0)` before `.unwrap_or(60000)`. With that change the same app stayed
up for the 25 s observed and showed its window.

Disclosure: the investigation and this text were produced with an AI assistant. The reproduction, the source trace and
the before and after runs were run by that assistant; no human has re-run them yet.
~~~

### notify

- `.out-of-scope/` has no entry for `notify` or GitHub Actions policy.
- The read-triggers-modified event and the `nextest --config` finding are not upstream defects: the second is
  documented behavior of nextest.
- Duplicate search: `gh search issues --repo notify-rs/notify "FILE_ACTION_MODIFIED ModifyKind::Any windows"` returned
  no result on 2026-10-15.
- The `Modify(Any)` mapping is the backend's coarse design, not a defect with a clear fix; no minimal fix was
  prototyped, and the read-triggers-modified event lacks a proven mechanism.
  The default policy is not to file, so nothing is filed.
