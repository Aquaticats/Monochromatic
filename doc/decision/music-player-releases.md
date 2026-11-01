# Release the music player from CI as GitHub pre-releases

## Status

Accepted 2026-10-14.
Owner-authorized: the owner asked for the release automation and answered the design questions below
in the session that built it.
The first real release is a separate act that needs the owner's go-ahead (see the consequences).

## Context

The owner asked for GitHub CI to release the music player automatically and put `.exe`, `.dmg`,
the Linux files and an `.apk` in GitHub releases.
Before this, the product had no release path:
`package/music-player/android-app` signed its release APK with the debug key,
`doc/decision/desktop-app-code-signing.md` described a manual, off-CI signing chain whose credentials do not exist yet
and deferred `.dmg` installers,
and the package README said "build from source".

The owner's answers to the design questions:

- Trigger and release shape follow the existing crate publication in `.github/workflows/cargo-publish.yml`:
  a version change merged to `main` triggers a run, a manual dispatch has a dry-run input that defaults to true,
  every file is attested, and one release job creates the release.
- Android: the owner generates a release keystore and stores it as repository secrets.
  Windows and macOS ship unsigned.
- Every release is a pre-release, because the README says the project is under active development.

Facts that constrain the design:

- Releases are immutable in this repository.
  A published release can never receive a replacement or an extra file,
  so a bad release is fixed only by a higher version (`doc/decision/forbidden-strings-windows-arm64-release-target.md`
  records the same lesson).
- The repository is public, so GitHub-hosted runners, including `windows-11-arm`, are free.
- No Windows, macOS or real-device Android build has ever been run by the project.
  Only the Linux x86_64 desktop build has been run, and the Android app has run on emulators.
- The package is planned to move to its own repository (`doc/planning/music-player-repository-extraction.md`),
  but has not moved.
  The workflow lives in this monorepo for now, and the README says so.
- The repository pins one Rust toolchain for every package in `mise.lock`:
  a dated nightly, chosen for the fuzzing crates, not because the music player needs nightly.

## Decision

### Trigger and versions

`.github/workflows/music-player-release.yml` runs on a push to `main` that changes
`package/music-player/desktop-app/Cargo.toml` or `package/music-player/android-app/rust/Cargo.toml`,
and on `workflow_dispatch` with a `dry-run` boolean that defaults to true.
Its detect job compares the version before and after the push,
as `cargo-publish.yml` does.

The product has one version.
Its single source is the `[package]` `version` of `package/music-player/desktop-app/Cargo.toml`.
The Android Gradle build reads that file:
`versionName` is the version and `versionCode` is `major * 10000 + minor * 100 + patch`,
so the two Android values cannot be edited to disagree, and the code never decreases while the version never decreases.
The detect job fails loudly when:

- the version is not plain `MAJOR.MINOR.PATCH` with minor and patch at most 99
  (a suffix would give two versions one code);
- the Android native crate's manifest carries a different version
  (the one place the Android side still states a version, and the one a person forgets to bump);
- a push lowers the version.

After the build, the Android job checks with `aapt2 dump badging` that the APK carries the computed
package name, `versionName` and `versionCode`.

The tag is `music-player-v<version>`.
If it already exists, the real release is skipped with a notice and the run succeeds:
a tag is never overwritten, and the release job re-checks the tag right before creating it.

### Safety against a bad first release

A run is real only when it is on `main` and is a push, or a dispatch with `dry-run` false.
The detect job computes `dry_run` once from the event and the ref,
refuses a real dispatch from any other ref,
and every later job reads that single output.
The workflow never relies on an empty `inputs.dry-run` to mean "real", unlike `cargo-publish.yml`:
on a push event that input is empty,
so `!inputs.dry-run` would be true on any branch that gained a push trigger.

A dry run builds, packages, verifies and signs everything,
uploads workflow artifacts that expire after one day,
and runs the release job's shell with `gh release create` replaced by a printed command,
so the command, tag, asset list and notes are visible without releasing.
It skips attestation and creates no tag and no release.

The version `0.1.0` was already in `main` before the workflow existed,
so no push will release it.
Its first release is a real dispatch from `main` after a green dry run on `main`,
and the owner decides when.

### Files

One release holds, with `<version>` filled in:

- Windows: `music-player-<version>-windows-x64.exe` and `music-player-<version>-windows-arm64.exe`,
  native builds on `windows-2025` and `windows-11-arm`.
  The `.exe` is the whole program: Slint renders without extra files, and the C runtime is linked in statically
  (`+crt-static` in `package/music-player/desktop-app/.cargo/config.toml`),
  so a clean machine needs no Visual C++ Redistributable.
  The job reads the executable's import tables with `pefile` and fails if a Visual C++ runtime DLL is among them.
  That check found that `+crt-static` was not enough on x86_64:
  the repository's config then pinned `lld-link.exe` for that target,
  and the executable linked by it still imported `VCRUNTIME140.dll` and the `api-ms-win-crt` DLLs,
  while the same code linked by the MSVC `link.exe` imported only system DLLs (the arm64 job, and an x64 probe run).
  The probe changed only the linker, so this shows a correlation with the linker, not a root cause;
  `doc/troubleshooting/windows-x64-lld-link-crt-static-dynamic-imports.md` records what is and is not established.
  The release build first worked around that with `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=link.exe`.
  Superseded on 2026-10-15:
   `desktop-app/.cargo/config.toml` no longer sets a `linker` key for either MSVC target
  (`doc/decision/music-player-windows-linkers.md`),
   so rustc runs `link.exe` by default,
   the export is removed,
  and the build step now prints the linker command and fails unless the program is `link.exe`.
  The executable was built as a console program through release 0.1.3 (no `windows_subsystem` attribute existed in the
  source), so Windows opened a console window next to the app.
  Since the change recorded in `doc/decision/music-player-windows-gui-subsystem.md` the executable is a GUI-subsystem
  program that attaches to the console of its parent process, and the release workflow fails when the Subsystem field
  of a Windows executable is not 2.
  The icon is embedded by the existing `build.rs`.
- Windows GNU:
   `music-player-<version>-windows-x64-gnu.exe`,
   a native build on `windows-2025` added on 2026-10-15
  at the owner's request for a Windows build without Microsoft's toolchain
  (job `build-desktop`,
   matrix entry `x64-gnu`,
   target `x86_64-pc-windows-gnu`).
  It uses the GNU host toolchain `<mise.lock version>-x86_64-pc-windows-gnu` and MinGW-w64 gcc from the MSYS2 UCRT64
  environment that the runner image carries in `C:\msys64`
  (`msys2/setup-msys2` is not an allowed action in this repository),
   so GNU ld links the program.
  The steps follow the `gnu-x64-ucrt64` entry of the `test-windows` job of `.github/workflows/music-player-release.yml`.
  The job reads the import tables and fails if the executable imports `libgcc_s*`,
   `libwinpthread*` or `libstdc++*`,
  because the program would then need those DLLs next to it;
   the C runtime is the UCRT
  (`api-ms-win-crt-*`,
   a Windows system component),
   not a statically linked one,
   and `+crt-static` is not used.
- macOS:
   `music-player-<version>-macos-arm64.dmg`,
   a native build on `macos-15`.
  macOS x64 is not built:
   the owner decided on 2026-10-14 that the release ships Apple silicon only.
  Intel Mac users build from source (`mise run //package/music-player/desktop-app:build` on the Mac),
  and the README says so.
  Each holds `Music Player.app`, assembled by the repository's own `bundle:macos` task from `macos/Info.plist`
  (bundle id `dev.monochromatic.musicplayer`, version, icon, high-resolution flag) and `asset/music-player.icns`,
  plus an `Applications` link.
  The bundle gets an ad-hoc signature (`codesign --sign -`), which is no developer identity:
  it is not notarized, so Gatekeeper asks for confirmation on the first launch.
  This extends `desktop-app-code-signing.md`, which deferred `.dmg` installers until the signing chain was proven;
  the owner chose unsigned installers now.
  The ad-hoc signature is an addition of this change that the owner may veto.
- Linux: `music-player-<version>-linux-x64.tar.gz` and `music-player-<version>-linux-arm64.tar.gz`,
  native builds on `ubuntu-24.04` and `ubuntu-24.04-arm`.
  Each holds `bin/music-player`, the launcher file, `share/icons/hicolor/scalable/apps/music-player.svg`,
  an `INSTALL.txt` and the license, in a reproducible tar stream.
  The icon is the repository's `asset/icon.svg`, which calls itself a placeholder;
  no designed icon exists, and none was invented.
  The human ruled on 2026-10-15 that a placeholder icon is fine.
  No AppImage is shipped, and the human ruled on 2026-10-15 that none is needed while the program ships as a
  single executable application (one `bin/music-player` file):
  the program links `libpipewire` and `libfontconfig` from the host,
  and bundling PipeWire's client library and its plugins is the part that cannot be validated across distributions
  from this project's single Linux machine,
  so an AppImage without them is the archive with a different wrapper and one with them is untested.
  The question can be reopened with a distribution test matrix.
- Alpine Linux: `music-player-<version>-linux-musl-x64.tar.gz` and `music-player-<version>-linux-musl-arm64.tar.gz`,
  native builds on `ubuntu-24.04` and `ubuntu-24.04-arm` (job `build-desktop-musl`),
  added on 2026-10-15 as `doc/decision/music-player-alpine.md` specifies.
  The layout is the glibc archive's:
  `bin/music-player`, the launcher file, the icon, `LICENSE` and an `INSTALL.txt` that names the Alpine packages
  (the `apk add` list of the "Runtime packages" section of the Alpine decision, which the release notes repeat)
  and the requirement of Alpine 3.24 or newer.
  The glibc archives do not run on Alpine and these do not run on glibc systems, so the two are separate files.
  The binary is dynamically linked against musl (a static musl binary cannot open a window or link PipeWire, see the
  Alpine decision) and is built with the `alsa-fallback` feature, so it plays through the default ALSA device when no
  PipeWire daemon runs.
- Android: `music-player-<version>-android-arm64-v8a.apk` and `music-player-<version>-android-x86_64.apk`, both signed
  with the release key, each holding only its own ABI's native library (decided 2026-10-22; releases up to 0.1.7
  shipped one `music-player-<version>-android.apk` with both libraries, 160,027,082 bytes in 0.1.7).
- `SHA256SUMS`, produced by a job that runs only after every build succeeded.
- Since 2026-10-15 the release also ships `music-player-<version>-desktop-third-party-licenses.txt` and
  `music-player-<version>-android-third-party-licenses.txt`,
  every Linux archive,
  the `.dmg` and the APK carry the matching file,
  and every desktop executable prints its notices with `--licenses`;
  `doc/decision/music-player-third-party-licenses.md` records how they are written and checked.

### Build mechanics

- The Rust toolchain is read from `mise.lock` in each job except the Alpine one and installed with `rustup`,
  so the release never drifts from what the repository builds with,
  and one mechanism works on every desktop platform that is not Alpine.
  The Android job installs Rust, JDK 21, the Android command-line tools and `cargo-ndk` with mise,
  installs the SDK components with the repository's `prepare:android` task, and builds with the existing
  `build:release` task.
  Its Rust is the `mise.lock` toolchain: the log of the dry run 38669432843 prints
  `rustc 1.101.0-nightly (e9bbadf68 2026-10-12)`, which is `nightly-2026-10-13`, the version in the native cache key.
  mise 2026.10 refused to install `cargo-ndk` before Rust was installed through mise
  (`doc/troubleshooting/mise-cargo-backend-needs-rust-installed.md`).
- The glibc Linux jobs install a package list that builds on the runner images
  (the Alpine job gets its libraries from the packages of `Containerfile.alpine`):
  `pkg-config clang libclang-dev cmake libpipewire-0.3-dev libspa-0.2-dev libfontconfig1-dev libfreetype-dev`.
  It was found by adding packages until the build passed.
  No package was removed to test whether it is needed, so the list may hold more than the minimum.
  Windowing and GL libraries are loaded at run time and need no headers.
- Every build uses `cargo build --release --locked`.
- Caches, decided by the owner on 2026-10-15 over this document's first choice of cold builds.
  Each desktop job restores `~/.cargo/registry/index`, `~/.cargo/registry/cache`, `~/.cargo/git/db`
  and the package `target` directory under the key
  `music-player-desktop-v1-<OS>-<arch>-<target triple>-<Rust version from mise.lock>-<hash of Cargo.lock>`,
  with a restore key that drops the lock file hash.
  So a changed `Cargo.lock` starts from the previous dependencies and cargo rebuilds only the crates whose version
  changed, where `cargo-publish.yml`'s exact-key cache would rebuild everything.
  The Android job does the same for `android-app/rust/target`
  (key: runner, Rust version, hash of the SDK component lines of `prepare:android`, hash of its `Cargo.lock`)
  and restores `~/.gradle/caches` (dependencies, transforms and the Gradle build cache)
  and `~/.gradle/wrapper/dists` under a key from the Gradle build files, with the same kind of restore key.
  Before saving, a step deletes this package's own artifacts, its path dependency's, the final binary and the
  incremental directories, because they are rebuilt on every run anyway.
  Only runs on `main` and manual dispatches save caches (the detect job's `save_cache` output),
  and the workflow has no pull request trigger.
  The restore and save steps are `actions/cache/restore` and `actions/cache/save` at the commit `cargo-publish.yml`
  uses, so a run saves only on an exact-key miss and after its build succeeded.
  mise's own tool cache stays off, and the Android SDK is not cached (see the measurements below).
  No pnpm store is cached because the release builds no JavaScript package.
  Provenance is unchanged: the attested bytes are the bytes the run built and uploaded,
  and cargo and Gradle rebuild whatever their own fingerprints say changed.
  A stale or poisoned cache entry is the new risk: the key names the toolchain and the lock file,
  and a bad entry is cured by changing the `v1` in the keys.
- The Alpine job (`build-desktop-musl`) builds `Containerfile.alpine` (targets `build` and `runtime`),
  checks that the toolchain's host triple is `x86_64-alpine-linux-musl` or `aarch64-alpine-linux-musl`
  (there is no `--target`),
  runs `cargo build --release --locked --features alsa-fallback` in the build image with the source mounted read-only,
  and then starts the binary with `--version` in the `runtime` image, which holds only the packages that `INSTALL.txt`
  lists, so a short list fails the job.
  Its Rust is Alpine's own `rustc`, not the `mise.lock` nightly, as the Alpine decision explains.
  The archive is packaged like the glibc one and uploaded as `asset-linux-musl-<arch>`.
  The dry-run and real-run gates, the attestation and the retention days are the glibc jobs' exactly.
- The Alpine job caches nothing, decided on 2026-10-15 under `doc/decision/ci-caching.md`
  (the owner asked for caching in general, so this is a deviation the owner may veto; the repository was at its cap).
  The release build left 684 MB of `CARGO_HOME` and 2.3 GB of target directory on both architectures before any trim
  (dry run 38684256021, `du`).
  The compressed size of a trimmed entry was not measured; the Alpine guard's entries, which hold debug builds, are
  949 MB and 928 MB, above the 650 MB guide of `doc/decision/ci-caching.md`.
  The usage of the repository was 9.98 GB of 10 GB, in 12 entries, when the run finished.
  A release run happens once per version bump, and an entry unread for seven days is evicted,
  so a cache would often be gone when the next release starts.
  The most a cache could save is the dependency phase, measured from the log as the time before cargo printed
  `Compiling music-player`: 9 min 07 s on x86_64 and 5 min 16 s on aarch64.
  The final crate, 9 min 55 s and 7 min 57 s, is rebuilt in every run, cache or not.
  A warm run was not measured, because no entry exists to restore.
  If a later decision adds one, the key must start with `music-player-alpine-release-` and
  `.github/workflows/cache-prune.yml` needs a matching `FAMILIES` pattern, because the Alpine guard's pattern
  (`music-player-alpine-<arch>-rustc_`) does not match that prefix.
- The Windows GNU job caches no cargo registry or target directory,
   decided on 2026-10-15 under
  `doc/decision/ci-caching.md` (the owner asked for caching in general,
   so this is a deviation the owner may veto).
  The repository's cache usage was 11.4 GB against the 10 GB cap when the dry run finished,
   and the rules there
  allow no new large family without an edit to that document.
  The cold job took 16 min 07 s in dry run 38692770715:
   the build step 14 min 37 s,
   of which the dependencies took
  7 min 14 s (from the start of the step to `Compiling music-player`) and the final crate and its path dependency
  7 min 23 s,
   which a cache does not remove.
  So a cache could save at most about 7 minutes,
   and only if its entry is warm and not evicted (an entry unread for
  seven days is evicted,
   and releases are rarer than that).
  In that run the job finished 1 min 04 s before the slowest job (Alpine x86_64,
   17 min 11 s),
   so it did not set the
  length of the run;
   the Alpine jobs take 14 to 20 minutes (see the measurements below),
   which is the same range.
  The compressed size of a GNU release entry was not measured;
   the debug entry of the same target in
  the `test-windows` job (a standalone workflow when this was measured) is about 652 MB,
   which is at the 650 MB guide.
  If a later decision adds a cache,
   its key can keep the shape of the other desktop jobs
  (`music-player-desktop-v1-Windows-X64-x86_64-pc-windows-gnu-<toolchain>-<lock hash>`),
   which the existing
  `music-player-desktop-v1-.+?-nightly-` pattern of `cache-prune.yml` already matches as its own family
  (checked by reading the pattern:
   it captures everything before `-nightly-`,
   so the target is part of the family).
  The job does cache the MSYS2 package downloads (70 MB;
   an exact hit in the dry run) under the key
  the `test-windows` job of the same workflow uses,
   so the two jobs share one entry per month
  (the release workflow alone saves it since 2026-10-22; two workflows shared it before).
- Both `jdx/mise-action` steps (macOS node,
   Android) pin `version: 2026.10.9` and do not set `minimum_release_age`,
  decided on 2026-10-15.
  Without a version,
   the release-age setting made every such job list mise releases through the GitHub API with
  the workflow token;
   run 38688512043 failed attempt 1 in the Android and the macOS jobs with
  `GitHub releases API returned 403 Forbidden` and passed on rerun.
  The action treats any 4xx except 429 as final,
   and ignores `minimum_release_age` when `version` is set
  (source lines in `doc/troubleshooting/mise-action-version-ahead-of-release.md`).
  The cost is a pin that a person must update;
   the owner and the steps are in the runbook section
  "The mise version pin".
  Dry run 38692770715:
   both steps logged `Download mise@2026.10.9` and
  `Verified mise-v2026.10.9-...tar.zst against signed checksums`,
   no line `Selected mise`,
   and `mise --version`
  printed `2026.10.9` on macOS arm64 and Linux x64.
- Each desktop binary is started with `--version` on its own runner, and the output must be the product version.
  This is the only run any non-Linux binary gets.
- `.cargo/config.toml` gained `+aes` entries for `aarch64-unknown-linux-gnu` and `aarch64-pc-windows-msvc`,
  which `gxhash` requires on every target,
  and `+crt-static` for both Windows targets.
  A CPU without AES instructions fails with an illegal instruction, as the existing comments there record.
  Superseded on 2026-10-15: the fingerprint hash became XXH3-64 from `twox-hash` (issue 545,
  `doc/decision/music-player-fingerprint-hash.md`), every `+aes` entry was removed, and no CPU feature is required.
  The release notes of `music-player-v0.1.0` and `music-player-v0.1.1` still carry the old AES sentence and cannot be
  edited, because releases are immutable.

### Attestation and checksums

Every uploaded file, including `SHA256SUMS`, is attested with `actions/attest-build-provenance`
on the exact bytes uploaded, in the job that built it (or computed the checksums).
Only those jobs hold `id-token: write` and `attestations: write`.
The release job holds `contents: write`, and nothing else does.
The release notes say how to run `gh attestation verify`.

### Android signing

The release key is a PKCS12 keystore with an RSA 4096 key,
alias `music-player-release`,
valid 36500 days,
with a neutral subject (`CN=Music Player release, O=Monochromatic`) that names no person.
It was generated on 2026-10-14 with `keytool` and a random password from the operating system,
and stored as four repository secrets:
`MUSIC_PLAYER_KEYSTORE_BASE64`,
`MUSIC_PLAYER_KEYSTORE_PASSWORD`,
`MUSIC_PLAYER_KEY_ALIAS`,
and `MUSIC_PLAYER_KEY_PASSWORD`.
The backup is two sops files encrypted to the maintainer's age identity,
`music-player-release.p12.sops.json` and `credentials.txt.sops.json`,
in `~/.local/share/music-player-release-keys` on the maintainer's machine (the owner chose sops over a password
manager on 2026-10-15; the plaintext copies were deleted after a decrypt-and-compare of both files).

`app/build.gradle.kts` has a `release` signing config that reads the four values from the environment or Gradle
properties.
Without them the release APK is still debug-signed, exactly as before.
With only some of them, or with `MUSIC_PLAYER_REQUIRE_RELEASE_SIGNING=true` and none, the build fails.

The workflow reads the secrets in one step of the Android job,
and only for runs on `main`.
It writes the keystore to `RUNNER_TEMP` and deletes it in a step that always runs.
Every other ref signs with a throwaway key generated for that run, with a masked random password.
On a real release the job also fails unless the APK's certificate SHA-256 equals the fingerprint written in the
workflow, so a throwaway-signed or debug-signed APK can never be published.
Dry runs on `main` sign with the real key, so the signing path is proven before a real release;
the file is a workflow artifact that expires after one day.

## Measurements

Dry runs on GitHub, 2026-10-15, branch `feat/music-player-release-workflow`, wall time per job:

- Cold run 38667490084 (every cache missed, then saved):
  Android 11m23s, Windows arm64 11m01s, macOS arm64 11m18s, Windows x64 10m13s, Linux x64 9m08s,
  Linux arm64 7m07s.
- Warm run 38668625715 (every cache hit by exact key):
  Android 1m52s, Windows arm64 5m59s, macOS arm64 4m33s, Windows x64 7m01s, Linux x64 3m02s,
  Linux arm64 3m16s.
- Fallback run 38669453427, whose exact keys were made to miss while saving was off, so only the restore keys could
  match: all seven restore steps reported a miss on the exact key and restored the entry by prefix,
  and the jobs took Android 2m19s, Windows arm64 5m50s, macOS arm64 3m23s, Windows x64 6m24s, Linux x64 4m03s,
  Linux arm64 3m25s.
  So a changed lock file costs about as little as a warm run, not a cold one.

Compressed size of each saved entry (what counts against the repository's 10 GB cap):
macOS arm64 517 MB, Windows arm64 601 MB, Windows x64 609 MB, Linux x64 617 MB, Linux arm64 606 MB,
Android cargo and native target 387 MB, Gradle 651 MB.
That is about 4.0 GB for one generation of the keys, and up to twice that while a lock file change leaves the
previous entries beside the new ones until GitHub evicts the least recently used.
The repository already held about 5.5 GB before this workflow, so the budget is tight:
evictions of other workflows' caches are possible, and the sizes should be rechecked after the first runs on `main`.

Dry run 38684256021 on branch `feat/music-player-release-musl`, 2026-10-15, wall time per job.
The Alpine jobs ran cold, as they do every time;
the other desktop jobs and the Android job restored their caches by exact key:

- Alpine x86_64 (`ubuntu-24.04`) 20m00s: the image builds 27s, the release build 19m02s
  (dependencies 9m07s, then the crate 9m55s), and packaging 7s.
- Alpine aarch64 (`ubuntu-24.04-arm`) 14m07s: the image builds 30s, the release build 13m13s
  (dependencies 5m16s, then the crate 7m57s), and packaging 4s.
- Run 38686797998, the same workflow dispatched again 28 minutes later (the Alpine jobs have no cache, so it is a
  repeat of the cold build, not a warm one): Alpine x86_64 15m18s (release build 14m00s) and aarch64 14m48s,
  so the repeat differed by 4m42s on x86_64 and 41s on aarch64 with nothing changed.
  Treat the Alpine job time as 14 to 20 minutes; the spread comes from the runners, not from a cache.
  The whole second run took 16m09s.
- Linux x64 3m57s, Linux arm64 3m17s, Android 3m48s, Windows arm64 5m39s, Windows x64 6m05s, macOS arm64 4m28s.
- The whole run took 20m45s from its creation to the end of the release job.
  Its longest build had been 7m01s in the warm run above, and the checksum job waits for every build,
  so the Alpine x86_64 build now sets the length of a release run.
- The release binaries are 54,189,848 bytes (x86_64) and the tarballs 22,211,539 and 20,855,451 bytes.

Dry run 38692770715 on branch `feat/music-player-release-gnu`,
 2026-10-15,
 wall time per job
(version 0.1.2;
 the cargo caches of the other desktop jobs and the Android job were exact hits from `main`,
 so
nothing was saved;
 the Alpine jobs and the Windows GNU job ran cold):

- Windows GNU x64 16m07s (build step 14m37s,
   MinGW-w64 install 35s,
   MSYS2 package cache an exact hit).
  The linker it printed:
   `x86_64-w64-mingw32-gcc`;
   the PE header linker version is 2.46 (GNU ld).
  Its imports are `api-ms-win-crt-*` (the Universal C Runtime of Windows) and system DLLs,
   none of `libgcc_s`,
  `libwinpthread`,
   `libstdc++`,
   `vcruntime`,
   `msvcp`;
   `music-player.exe --version` printed `music-player 0.1.2`.
  The file is 44,017,152 bytes.
- Windows x64 (MSVC) 12m13s,
   Windows arm64 5m46s.
  Both printed a `link.exe` linker (Visual Studio 18,
   MSVC 14.51.36231,
   `HostX64\x64\link.exe` and
  `HostARM64\arm64\link.exe`),
   PE header linker version 14.51,
   and no C runtime DLL among the imports.
  The x64 job rebuilt 449 crates although its cache key was an exact hit,
   because the cached entry was built with
  `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=link.exe` in the environment and the build no longer passes it:
  a changed linker setting makes cargo rebuild everything (a local control with a path dependency and a build script
  did the same,
   and a second build with unchanged settings rebuilt nothing).
  Because the key matched exactly,
   the job saves no new entry,
   so a dispatch for the same version stays slow until
  `Cargo.lock` changes;
   a version bump changes it,
   so the next release restores the entry by prefix,
  rebuilds once and saves a fresh one.
- Alpine x86_64 17m11s,
   Linux x64 3m08s,
   Linux arm64 3m58s,
   macOS arm64 6m20s,
   Android 2m18s.
- Dry run 38695662196 on the same branch at the head that changed only the notes wording
 (35c1617a48),
 about 30 minutes later:
 Windows GNU 16m27s (20 s from the first run,
 so the cold GNU time is 16 to 16.5 minutes),
 Windows x64 (MSVC) 15m13s with 449 crates compiled again and no cache save,
 which confirms that a dispatch of the same version stays slow until the lock file changes.
 Alpine x86_64 took 20m41s there.
 The GNU executables of the two runs differ in their SHA-256 (the build is not reproducible byte for byte),
 and both passed the import check.
- All nine files passed `sha256sum --check` against `SHA256SUMS` after download,
   and the `release` job's printed
  `gh release create` command listed all nine files and `SHA256SUMS`.

## Consequences

- The first real release of a version is permanent.
  A broken release can be fixed only by a higher version, and its tag stays reserved.
  The dry run on `main` and a read of its log are the guard;
  the runbook `doc/runbook/release-the-music-player.md` says how.
- Windows and macOS builds are unsigned.
  Windows SmartScreen and macOS Gatekeeper warn on the first launch,
  and the notes and the README tell users the steps.
  Signing both is possible later with the chain in `desktop-app-code-signing.md`.
- Only the Linux x86_64 build has been run by the project.
  Releases and the README say "built by CI, not yet tested on this platform" for the rest,
  and CI starts each desktop binary with `--version` and nothing more.
  The Alpine builds have more evidence, though not from hardware:
  the `test-alpine` job of `.github/workflows/music-player-release.yml` runs the unit tests and a headless window and
  sound check in Alpine containers on both architectures, on debug builds and not on the released files.
  The dry run 38684256021 started the x86_64 release binary in an `alpine:3.24` container after `apk add` of the
  listed packages and printed `music-player 0.1.1`; the same binary in a bare `alpine:3.24` failed with
  `Error loading shared library libfontconfig.so.1`, so the check can fail.
  The aarch64 binary was started with `--version` only by the job, on a native arm64 runner.
- Losing the Android key means installed copies can no longer be updated.
  The key lives in the repository secrets, which cannot be read back, and in the backup.
  The backup must be copied into the password manager; rotation and loss are covered in the runbook.
- The repository-level secrets are visible to any workflow with write access to the repository.
  Restricting them to a protected environment limited to `main` would close that,
  and was not requested.
- The Android APK is about 160 MB, because it carries two ABIs of the native engine and the two Turso libraries.
  Release assets allow far more.
- The macOS arm64 job once failed with `hdiutil: create failed - Resource busy`,
  which did not recur in the six later runs that built the image;
  the job now retries `hdiutil create` and `detach` up to five times
  (`doc/troubleshooting/macos-hdiutil-create-resource-busy.md`; the retry has not yet fired in CI, and the cause is
  unknown).
- Two version-bearing manifests must be bumped together,
  and their lock files refreshed with `cargo update --workspace`,
  because the release builds with `--locked`.
  `file-enforcer` does not own the `version` field: `buildCargoManifestPlan` in `file-enforcer.config.ts` enforces
  edition, license, publish, lints, shared dependencies, repository, readme, homepage and profiles, and a throwaway
  copy with both versions bumped came out of `node file-enforcer.config.ts` with the bumps intact.
- The repository's pinned Rust is a floating nightly recorded as a date in `mise.lock`.
  A date that Rust no longer serves would fail the build until `mise.lock` moves on.
- Releases are built from the workflow's commit, which is in the monorepo today.
  After the repository extraction the workflow, the secrets and the tag prefix must move with the package.
- Bundled Rust crates carry their own licenses, and no third-party notice file is generated yet.
  The archives ship the project license only.
  Superseded on 2026-10-15:
  the release workflow writes the third-party notices for every asset
  (`doc/decision/music-player-third-party-licenses.md`).

## Alternatives rejected

- Release-please or another tool that opens version pull requests:
  the existing crate publication shows the repository's convention is a hand-edited version,
  and the owner chose to follow it.
- Cross-compiling the macOS and Windows binaries from Linux:
  fragile SDK linking (`desktop-app-code-signing.md`); native runners exist for every target and are free here.
- An environment-scoped secret store:
  the owner asked for repository secrets; see the consequences.
- Cold release builds with no cache: this document's first choice, overruled by the owner on 2026-10-15
  for the glibc, Windows, macOS and Android jobs.
  The Alpine job is cold on purpose (see the build mechanics), and the owner may overrule that too.
- A static musl binary:
  rejected by the Alpine decision (a static musl program cannot `dlopen` the windowing libraries, and Alpine ships no
  static PipeWire, fontconfig or freetype libraries),
  so the Alpine files are dynamically linked and built with Alpine's own Rust inside the Alpine image.
- Caching the Android SDK and NDK:
  the owner asked for caching in general, and this entry was part of that.
  It was built and measured, then left out as a deviation the owner may veto:
  restoring the 1.0 GB entry took 17 seconds against 25 seconds to install the components,
  so it saved 8 seconds for a tenth of the repository's 10 GB cache budget.
  Restoring it needs one more `actions/cache` pair in the Android job (path `$ANDROID_HOME`, key from the
  component lines).
