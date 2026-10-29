# Audit of /tmp uses (issue 1810)

Issue: [#1810](https://github.com/Aquaticats/Monochromatic/issues/1810).
Rule: `XTD` in `AGENTS.md` (never use `/tmp`; the device behind it is unknowable).
The user chose one pass (not one layer per run) on 2026-10-29.

## Ownership

- Session `monochromatic-ff` owns every Rust use in `package/git-policy/cli`: the production trio
  (`markdown_linter.rs`, `transaction_git.rs`, `manual_push_probe.rs`) and the 12 test files that still mention
  `temp_dir()` or `/tmp`.
  It messages `monochromatic-80` when `mise run //package/git-policy/cli:native:clippy` is clean under the new ban.
- The root `clippy.toml` ban (below) is uncommitted until that message arrives, because every crate sets
  `disallowed_methods = "deny"` and the ban would fail that crate's clippy.
- The TypeScript container gate runner in `package/git-policy/cli/bin/test-native-container.mjs` and
  `bin/native-verification-process.mjs` copies the source snapshot into `/tmp/cli-git-native-test-*`.
  It belongs to the non-Rust part of this issue.

## Done

- `package/search-shadow/cli`: unit tests use `src/test_scratch.rs` (rooted beside the test executable, in the cargo
  build directory), the shim integration test uses `CARGO_TARGET_TMPDIR`.
  Clippy clean under the ban, 49 tests pass, guard test
  `test_scratch::the_scratch_directory_is_not_under_the_system_temporary_directory`.
  Commit `eacb935ed4`.
  The `/tmp` strings in its `mise.toml` run inside a container image and stay.

## Uncommitted

- Root `clippy.toml`: `disallowed-methods` also bans `std::env::temp_dir`, `tempfile::tempdir`,
  `tempfile::tempfile`, `TempDir::new` and `NamedTempFile::new`.
  Verified able to fail: clippy over `search-shadow/cli` reported 66 matches before the fix.

## Measured, not yet acted on

- Rust crates besides `git-policy/cli` and `search-shadow/cli` with `tempfile` or `temp_dir()`:
  `ide/app` (64 files), `music-player/desktop-app` (23), `forbidden-strings/cli` (12), `linter/monochromatic-lint` (6),
  `nested-wayland-session/cli` (3), `wg/cli-quicker-exempt` (2), and one file each in
  `music-player/truepeak-core`, `linter/monochromatic-lint-closure`, `file-manager/variant-gtk-sticky`,
  `file-manager/app`.
  A clippy sweep over them (unit `issue-1810-sweep`, script `~/temp/agent/clippy-sweep.ts`) writes
  `~/temp/agent/sweep-summary.json` and one log per crate.
- TypeScript: about 210 files import `tmpdir` from `node:os` (323 calls, 5 `os.tmpdir()`), mostly
  `mkdtemp(join(tmpdir(), 'prefix-'))`.
  Largest packages: `file-enforcer/core` (46 files), `wg/cli-quicker` (17), `mvm/cli` (16),
  `oxlint-plugin/prefer-readonly-parameter-type` (15), `claude-code-plugin/source` (14).
  No shared scratch helper exists yet; `fs-path/core` has 24 consumers.
- Config with `/tmp`: `search-shadow/cli`, `music-player/desktop-app`, `linter/monochromatic-lint`,
  `test-fixture/file-enforcer-perf`, `translation-repair/coverage-census` and `upstream-fork/conf` `mise.toml` files,
  `upstream-fork/deepmerge-ts.fuzz/container/stryker.config.json`, and `doc/audit/*.evidence.json` plus
  `doc/planning/depot-*.json` (historical records; keep their text).

## Findings

- Cargo's current layout puts test executables under `<cargo-build-dir>/<profile>/build/<package>/<hash>/out/`, so
  "three levels above the executable" is the package build directory, not the target directory.
  Scratch still lands on the disk-backed cargo build directory.
- `file-enforcer/core/src/toolchain/*` and `.projenrc.ts` are another session's uncommitted work; never stage them.

## Next

1. Read `~/temp/agent/sweep-summary.json` and fix each crate's violations (build-directory scratch helper per crate).
2. Decide the TypeScript helper (location, base directory, dependency fan-out) and the oxlint restriction on `tmpdir`.
3. Move the container gate runner scratch in `git-policy/cli/bin`.
4. Land the `clippy.toml` ban after `monochromatic-ff` reports clean.
5. Commit with `Refs #1810`, and `Closes #1810` on the last commit.
