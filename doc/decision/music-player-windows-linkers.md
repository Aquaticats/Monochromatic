# music-player: Windows builds link with MSVC link.exe, plus a GNU build, and Linux and macOS keep their linkers

Status:
 Accepted.
Decided:
 2026-10-14 by the maintainer.
Verified on GitHub's Windows runners:
 2026-10-15.
Applies to `package/music-player/desktop-app`.

## Context

The maintainer reported that `lld-link.exe` "bugs out on Win" and asked for `mold` to be tried, with the condition
 "If mold works everywhere we'll switch to it everywhere."
`.cargo/config.toml` had pinned `linker = "lld-link.exe"` for `x86_64-pc-windows-msvc`
 since commit 9e14dc3383 (2026-06-08, "build on Windows via cpal/WASAPI"), which records no reason.
The release workflow branch recorded a concrete defect of that pin in commit 24b1c54f03:
 an x86_64 exe built with `+crt-static` and linked by `lld-link.exe` still imports the dynamic C runtime
 (`VCRUNTIME140.dll`, `api-ms-win-crt-*`), while Visual Studio's `link.exe` links it statically.
 The release workflow first worked around that with `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=link.exe`
 (removed on 2026-10-15,
 see "Instructions for the release workflow").

## The mold question

`mold` does not produce Windows executables, so the condition "works everywhere" fails on Windows and `mold` is not
 adopted.

- The upstream README (`rui314/mold`, default branch at commit c522789231c9) line 17:
  "mold supports x86-64, i386, ARM 32/64, RISC-V 32/64, PowerPC 32/64, s390x, LoongArch 32/64, SPARC64, m68k, and SH-4".
  Those are CPU architectures of ELF targets; the prebuilt binaries are "for Linux".
- The latest release, v3.0.0 (published 2026-10-05, `gh release view --repo rui314/mold`), is ELF only:
  its tag tree has no `macho/` directory.
- A correction to the premise that "mold links ELF only":
  unreleased `main` (1775 commits after v3.0.0) now has a Mach-O linker (`macho/`, `arch/macho-arm64`,
  `arch/macho-x86_64`, `tests/macho`; commit b4258ab43 "Move the Mach-O linker to macho/", 2026-10-08; the Mach-O work
  began in 2021, commit cab0ccf0b "[Mach-O] Add a stub").
  That could matter for macOS later. Neither tree has any PE or COFF output directory, so Windows is not served.
- The repository's own vet report, `doc/audit/tech-mold-linker-vet-2026-10-05.md` (hard constraint HC5 and the section
  "Non-Linux link surfaces that mold cannot serve"), reaches the same result for macOS, Windows PE and Android,
  and notes that `mold-3.0.0-x86_64-windows.zip` is a Windows-hosted cross-linker for ELF output only.

## Decision

- Windows MSVC (x86_64 and aarch64): no `linker` key. rustc runs Visual Studio's `link.exe`, which it finds through
  vswhere. No LLVM install and no PATH edit are needed.
  The arm64 target had no `[target]` table before; both MSVC targets carry `+crt-static`.
- A second Windows build for `x86_64-pc-windows-gnu`:
  the GNU build is for people who do not want Microsoft's toolchain.
  It is linked by mingw-w64 GNU ld through the `gcc` driver (`-C link-arg=-fuse-ld=bfd`), never by lld.
  On Windows it is built with MSYS2 (UCRT64 environment) and a rustup GNU host toolchain.
  On Linux it can be cross-built and tested under Wine in a container (`image:windows-gnu`, `build:windows-gnu`,
  `test:windows-gnu` in `package/music-player/desktop-app/mise.toml`).
- Both Windows linkers build a GUI-subsystem program (`#![windows_subsystem = "windows"]` in `src/main.rs`):
  rustc passes `/SUBSYSTEM:windows` to `link.exe` and `-Wl,--subsystem,windows` to GNU ld through gcc, so the GNU target
  needs no `-mwindows` flag and no change in `.cargo/config.toml`.
  Sources, the console attach and the CI proof: `doc/decision/music-player-windows-gui-subsystem.md`.
- Linux is unchanged and keeps `rust-lld`: the maintainer's switch to `mold` was conditional on it working
  everywhere, and it does not.
- macOS is unchanged and keeps its linker.

## Evidence

All of this ran on GitHub-hosted runners with `.github/workflows/music-player-windows.yml`
 (jobs `msvc-x64` on `windows-2025`, `msvc-arm64` on `windows-11-arm`, `gnu-x64-ucrt64` on `windows-2025`).
That workflow moved into the `test-windows` job of `.github/workflows/music-player-release.yml` on 2026-10-22.

- The linker that ran, printed with `cargo rustc ... --print link-args` (run 38670823208):
  MSVC x64 and arm64 used `...\VC\Tools\MSVC\14.51.36231\bin\HostX64\x64\link.exe` and `HostARM64\arm64\link.exe`
  (PE linker version 14.51, so not lld); the GNU job used `x86_64-w64-mingw32-gcc` with `-fuse-ld=bfd`
  (PE linker version 2.46, GNU ld). The workflow fails if the line names an lld program.
- C runtime imports of the debug exe, read by the workflow's PE inspection step. Before `+crt-static` reached this
  branch (run 38670823208) the MSVC debug exe imported `vcruntime140.dll` and `api-ms-win-crt-*`. After the merge of
  `main`, which carries `+crt-static` for both MSVC targets, both MSVC jobs printed "C runtime imports: none"
  (run 38688163368), so `link.exe` with `+crt-static` leaves no C runtime DLL import in the debug exe.
  The release exe was not inspected on this branch; the release workflow's own check covers it.
  The UCRT64 build imports only `api-ms-win-crt-*` DLLs and Windows system DLLs, and none of `libgcc_s`,
  `libwinpthread` or `libstdc++`; the workflow fails otherwise.
- A release-profile GNU build with thin LTO built and ran (run 38676895175, about 15 minutes).
- Tests: at commit 11adb30419 (which includes the merge of `main` that carried release 0.1.1) the suite ran 1422 tests
  on every Windows job with 1422 passed and 1 skipped (run 38688163368).
  On the Linux host, 1450 passed and 1 skipped; the difference is Linux-only tests.
  The Windows suite had never run before 2026-10-14; the fixes it needed are in
  `doc/troubleshooting/music-player-windows-build-and-test.md`.
- Under Wine (cross-build in the Fedora 44 container, before the final merge of `main`): 1383 tests run,
  1377 passed, 4 failed, 2 killed after hanging. The same tests pass on the real runners, so these are recorded as
  Wine differences, not defects of the build (details in the troubleshooting document).
  The cross-built release exe imports only system DLLs and `msvcrt.dll` (GNU ld 2.46), prints its version,
  and stays up in a Wine virtual desktop with a visible window.

### UCRT64 chosen over MINGW64

Both MSYS2 environments passed in the four green runs in which both jobs existed. The workflow keeps UCRT64 only.
The measured C runtime imports of the two were: MINGW64 imported only `msvcrt.dll` (GNU ld 2.47) and UCRT64 imported
only `api-ms-win-crt-*` DLLs (GNU ld 2.46), both read from the PE inspection step of run 38670823208.
Rust documents `msvcrt` as the default C library of `x86_64-pc-windows-gnu`
 (`src/doc/rustc/src/platform-support/windows-gnu.md`), so UCRT64 is outside the documented default,
 but it passed every test and imports only `api-ms-win-crt-*` DLLs and Windows system DLLs.
MSYS2 deprecated its MINGW64 environment on 2026-03-15 (no new packages; UCRT64 or CLANG64 recommended).
The Linux cross build uses Fedora's mingw64 packages, and its exe imports `msvcrt.dll`.
(Since 2026-10-29 the cross image is CachyOS, whose mingw-w64 packages default to UCRT: the cross-built test executable
 imports `api-ms-win-crt-*` DLLs, as the release asset does, read with `x86_64-w64-mingw32-objdump -p` in the image.
 The sentence below describes the Fedora image.)
So the Wine proof and the CI build use different C runtimes for the same target; the release asset below uses UCRT64.

## Consequences

- Link time of `link.exe` against `lld-link.exe` was not measured, so no speed difference is claimed.
- A new workflow, `.github/workflows/music-player-windows.yml`, builds and tests all three Windows targets on
  pull requests and pushes that touch the desktop app or `truepeak-core`
  (moved into the `test-windows` job of `.github/workflows/music-player-release.yml` on 2026-10-22,
  where it runs on a release or a manual run and no longer on pull requests).
- `msys2/setup-msys2` could not be used: the repository allows only GitHub-owned actions, verified creators and the
  patterns `pnpm/* cargo-bins/* google/* returntocorp/* ossf/* jdx/* changesets/*`, with `sha_pinning_required: true`
  (the first push ended in `startup_failure`). The workflow runs the MSYS2 that the `windows-2025` image already has at
  `C:\msys64`.
- The windows-gnu icon: `build.rs` uses `#[cfg(windows)]`, which tests the host, so a Linux cross build has no icon.

### Cache design

Each job caches the cargo registry index and downloaded crates, the git database and the package `target` directory,
 with `actions/cache` pinned to the SHA the repository already uses (`27d5ce7f107fe9357f9df03efb73ab90386fccae`,
 restore and save halves separately).

- Key: `mp-win-v1-<runner OS>-<arch>-<job>-<target>-<toolchain>-<hash of both Cargo.lock files>`.
- Restore-keys: the same string without the lockfile hash, so a lockfile change rebuilds only the changed crates.
  The repository's `cargo-publish.yml` uses an exact-key-only cache, which rebuilds everything on any change.
- The GNU job also caches the MSYS2 package downloads (`pacman` cache, cleaned with `pacman --sync --clean` before the
  save): key `msys2-pkg-v1-<OS>-<arch>-<MSYSTEM>-<YYYY-MM>`, rotating monthly so the cache cannot grow without bound.
- The entries saved during the branch proof are scoped to the branch (main and pull requests to main cannot restore
  them) and were deleted afterwards. The first run on `main` after the merge is cold by design.
- Saves happen only on `main` and on manual dispatch, never on pull requests, and only when the build succeeded and the
  exact key missed. This pull-request rule is written in the workflow and passes actionlint, but has not been exercised
  by a pull request run.
- Entries are lean: `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, no incremental directories, and this
  package's own artifacts are pruned before the save.

Measured on the branch (the temporary branch trigger is removed):

- Cold, every cache a miss (run 38686990916): total 636 s on `msvc-x64`, 610 s on `msvc-arm64`, 731 s on
  `gnu-x64-ucrt64`; "Build the tests" took 334 s, 296 s and 384 s. All three saved their entries.
- Warm, every cargo cache an exact hit and the MSYS2 cache an exact hit (run 38688163368, a manual dispatch):
  total 272 s, 409 s and 415 s; "Build the tests" took 77 s, 82 s and 81 s; the restore took 17 s, 50 s and 12 s;
  no save ran.
  The remaining 77 to 82 s is the rebuild of the package's own crate, whose artifacts are pruned before the save by
  design (that time was not profiled further).
- Restore-key fallback (run 38675620801, with the key suffix `-probe` added for the test and reverted): every job logged
  "restored from restore-key", rebuilt in 70 to 106 s, then saved a new entry.
- A last manual dispatch on the final head (commit 8974cf000e, run 38690107583) was a miss on every cache, not a
  fallback: the entries saved by the earlier runs on the branch were no longer listed 30 minutes later, while the
  repository was over the cap. It passed all 1422 tests on every Windows job, with totals of 644 s (`msvc-x64`),
  591 s (`msvc-arm64`) and 783 s (`gnu-x64-ucrt64`), and saved new entries of about the same sizes.
- Entry sizes after the runs: 653,906,882 bytes (`msvc-x64`), 634,836,019 bytes (`msvc-arm64`), 652,299,238 bytes
  (`gnu-x64-ucrt64`) and 72,795,969 bytes (MSYS2 package cache), about 2.0 GB together.
  The uncompressed `target` directory is 2.3 to 2.5 GB.
- Repository cache usage right after the warm run was 10,251,013,083 bytes (`gh cache list`), at the 10 GB cap
  (5.5 GB were reported as used earlier the same day); about 8.2 GB belonged to other workflows.
  After the last run and the deletion of every entry of this workflow it was 11,070,701,437 bytes, all from other
  workflows, so the repository is over the cap without these entries.
  GitHub evicts the least recently used entries when the cap is reached, so these entries can be evicted by saves
  from other branches. The first run after an eviction is a cold run; it is not a failure.

## Instructions for the release workflow

`.github/workflows/music-player-release.yml` is owned by the release work and was not edited here.
Done on 2026-10-15 on branch `feat/music-player-release-gnu` (the export is removed,
 and the GNU asset
 `music-player-<version>-windows-x64-gnu.exe` is built);
 the list below is the original instruction,
 and
 `doc/decision/music-player-releases.md` records what was built.

- Remove the `CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=link.exe` export and its comment about `.cargo/config.toml`
  pinning `lld-link.exe`: the config no longer sets a linker. Keep the `+crt-static` and the import check.
- Keep the MSVC asset names `music-player-<version>-windows-x64.exe` (`windows-2025`) and
  `music-player-<version>-windows-arm64.exe` (`windows-11-arm`).
- To publish the GNU build, add the matrix entry
  `{ os: windows, arch: x64-gnu, target: x86_64-pc-windows-gnu, runner: windows-2025 }`
  and the asset `music-player-<version>-windows-x64-gnu.exe`.
  Its toolchain step installs `<mise.lock version>-x86_64-pc-windows-gnu` (a GNU host toolchain, so that host equals
  target); its MSYS2 steps are copied from `.github/workflows/music-player-windows.yml`
  (moved into the `test-windows` job of `.github/workflows/music-player-release.yml` on 2026-10-22;
  the month step, the package cache, `pacman --sync --refresh --noconfirm --needed "${MINGW_PACKAGE_PREFIX}-gcc"` with
  `MSYSTEM=UCRT64`, prepend `C:\msys64\ucrt64\bin` to the path, and verify `gcc -dumpmachine`);
  then `cargo build --release --locked --target x86_64-pc-windows-gnu`.
- The existing import check names `vcruntime` and `msvcp` and does not cover GNU. Add a check that rejects imports of
  `libgcc_s*`, `libwinpthread*` and `libstdc++*`. `+crt-static` is not needed for the GNU target.
- The names above differ from the task's example names (`...-windows-x86_64-msvc.exe`); the existing release names are
  kept, with `-gnu` appended for the new asset.

## Not verified

- No real Windows desktop, real speakers or real arm64 hardware were used; the runners have no audio device
  (the app logs "audio init failed; degrading to silent mode" and stays up).
- Audio under Wine; the content a Wine window draws (no screenshot was taken).
- The Wine suite after the final merge of `main`; the real runners ran it.
- The pull-request no-save rule of the cache policy, as stated above.
- The GNU build on a developer's own Windows machine with a different MSYS2 setup.
