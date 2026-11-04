# Keeping every build directory lean

Proposal and measurements, 2026-11-04T17:45:00-05:00 (measurement round, read-only).
Nothing was deleted and no build configuration was changed.
The owner picks an option; until then this document is the only change.

Scope: every build directory of the monorepo at `/var/home/user/Monochromatic`: Rust `target/` directories,
the Cargo build directory the host configuration keys by workspace path,
fuzz and verification output, Gradle and Android output, `dist/`, `node_modules` and `.cache`.

## Owner requirements that bound this

- 2026-11-04, answering how the Git wrapper should handle copying build output into new worktrees:
  "No lefting out build outputs. The real fix is to make sure every build dir is always lean.
  That's not the wrapper's responsibility, but I'm assigning this to you to."
  So the fix is not in the Git wrapper (`package/git-policy/cli`).
- 2026-11-04, on guidance in `AGENTS.md`: "No exposing underlying machinary to always-loaded context."
  So no `AGENTS.md` rule is proposed.
- 2026-11-04: "(debuginfo level, incremental off for dependencies - no messing with these."
- 2026-11-04: "other profile settings (split-debuginfo, strip, codegen-units) should be off-limits too".
  So no build profile setting is changed or proposed here, and only mechanisms that keep a directory lean without
  changing what a build produces are ranked.
- Standing owner decisions on the copy itself, from `doc/planning/cli-git-main-worktree-ignored-copy.md` lines 15 to 20:
  "Builds must be copied. Other ignored files must be copied." (2026-11-02);
  "Skip nested worktrees and caches. The point is to make new worktrees immediately functional." (2026-11-02);
  "Skip run records (Recommended)." (2026-11-02).
  Leanness therefore has to come from the directories being small, not from the copy leaving them out.

## Summary

- A lean build directory here means: nothing in it was produced by a compiler generation, workspace or run that nothing
  uses any more, and its size stays within a bound derived from what one current build needs (section "What always lean
  can mean").
- The main checkout holds 67.5 GiB in build directories by `du` (1 KiB blocks, 2026-11-04), of which 62.5 GiB are
  Rust `target/` directories.
  Two container-built packages are 52.2 GiB of that: `package/ide/app/target` (38.7 GiB) and
  `package/nested-wayland-session/cli/target` (13.5 GiB).
- Cargo never removes an intermediate artifact once a newer compiler has superseded it.
  Cargo's metadata hash includes the compiler version, so each toolchain change orphans every intermediate built
  before it (`src/compiler/build_runner/compilation_files.rs` lines 923 to 932).
  The repository installed a new nightly 18 times since 2026-09-11 and pins one per checkout.
  Grouping the units of those two directories by the `rustc` field of their fingerprint files, 36.5 GiB of the 54.5 GiB
  of units and incremental state belong to a compiler generation older than the newest one in their profile directory.
- The host Cargo build directory `/var/mnt/encrypted/cargo-build` holds 3545 sized trees (plus 101 created while
  measuring) that add up to 6166 GiB by `du`, much of it shared by reflink; PEND-PHYSICAL.
- Ranking (reasons in section "Ranking"):
  the compiler-generation sweep first, orphan-tree collection second, per-directory size caps third,
  generation-keyed build directories fourth, a container build-directory split fifth,
  a timer-driven runner sixth, tracking upstream garbage collection seventh, manual `cargo clean` last.

## Method and limits

- Everything ran read-only inside `systemd-run --user --scope` with `IOWeight=1`, `CPUQuota` of at most 100 percent,
  a memory cap and a task cap, under `ionice --class=3`.
  The scripts read file names, sizes and times.
  The only file contents read are the first 400 bytes of one fingerprint JSON per Cargo unit (to read its `rustc`
  field) and the first 900 bytes of one dep-info file per host tree (to find its workspace).
- No build was run to measure build time.
- `du` counts every extent of every file in full, including extents shared by reflink or snapshot.
  That matters here in both directions.
  `btrfs filesystem du` (documented at [btrfs-filesystem(8)][btrfs-du]) reported `Exclusive 0.00B` for the main
  checkout's `package/ide/app/target`, `package/nested-wayland-session/cli/target` and
  `package/git-policy/cli/target`: every extent is also referenced elsewhere (snapshot worktrees; 91 of the 139
  worktrees nested in the main checkout were subvolumes on 2026-11-02, per
  `doc/planning/cli-git-main-worktree-ignored-copy.md`).
  Deleting such a directory in the main checkout alone frees nothing physical while a sharer exists.
  Across filesystems the same files are a byte copy, so `du` is the right measure for the cost of a copy
  into `~/temp/agent` (`/var/mnt/encrypted`, another filesystem), and the wrong measure for disk pressure.
  The `btrfs filesystem du` set definition is per argument ("We define each set as those files found by a recursive
  search of an argument"), so physical figures below are per tree.
- The "stale" figures group Cargo units by the `rustc` field of their fingerprint, treat the group whose
  `invoked.timestamp` is newest in each profile directory as current, and sum the others.
  Several compiler generations can be live at once in one directory when two toolchains alternate (a host build and a
  container build of the same directory), so the figures are what a rule keeping one generation would remove, not a
  guarantee that nothing needs them.
  A profile directory not rebuilt since the last toolchain change counts as fully current.
- The `find` on this machine resolves to a search shim inside `sh`; the inventory was repeated with `/usr/bin/find`
  and found the same 37 `target`, 218 `dist`, 5 `build`, 3 `corpus`, 3 `artifacts`, 2 `.gradle` and 1 `out`
  directories in the main checkout.
- Not measured: `~/.cargo/registry` and `~/.cargo/git` (Cargo garbage-collects those),
  the pnpm content store outside the checkout, Android emulator images and the SDK.

## Measurements

### The main checkout, by kind

Sizes are `du --summarize --one-file-system` in 1 KiB blocks, 2026-11-04, converted to GiB.

- `target/` directories: 62.5 GiB in 38 directories (37 packages plus the repository root).
- `node_modules` at the root: 3.1 GiB, of which `node_modules/.monochromatic` (module-logger files) is 1.2 GiB,
  `node_modules/.pnpm-store` 1.1 GiB and `node_modules/.cache` 0.8 GiB
  (`node_modules/.cache/prefer-readonly-parameter-type`).
- Gradle `build/`: 0.9 GiB in 5 directories
  (`package/music-player/android-app/app/build` 0.54, `package/linter/kotlin/build` 0.38,
  `package/music-player/android-app/build`, `package/linter/linter/kotlin/build`, `package/ssg/aquati.cat/src/build`
  small); `.gradle`: 5 MiB in 2 directories.
- `dist/`: 0.6 GiB in 218 directories, of which `package/upstream-fork/deepmerge-ts.fuzz/dist` is 0.44 GiB
  and `package/ide/app/dist` 0.09 GiB.
- `package/music-player/truepeak-core.bench/out`: 0.14 GiB.
- Fuzz `corpus`: 0.08 GiB in 3 directories; fuzz `artifacts`: 56 KiB in 3 directories.
- Root `.cache`: 0.05 GiB; 229 nested `.cache` directories under `package/`: each at most 2 MiB.
- `playwright-report`, `test-results`: negligible.
- Total: 70,739,320 KiB, which is 67.5 GiB.

The brief's seven largest directories (65.6 GiB) matched.
Directories it did not list: `package/terminal/app/target` (0.56 GiB, mostly a Zig cache),
`package/music-player/android-app/app/build` (0.54 GiB), `package/upstream-fork/deepmerge-ts.fuzz/dist` (0.44 GiB),
`package/linter/kotlin/build` (0.38 GiB), `package/music-player/truepeak-core.bench/out` (0.14 GiB),
and `package/forbidden-strings/core.fuzz/corpus` (0.04 GiB).

### What is in the Rust `target/` directories

Two directories are container builds with the package directory bind-mounted at `/work` and no
`CARGO_BUILD_BUILD_DIR` (`package/ide/app/mise.toml` line 38, `package/nested-wayland-session/cli/mise.toml` line 52),
so the host setting from `doc/decision/cargo-build-dir-host-cache.md` never reaches them; that record predicts this
mixed state at line 340.
Every other package builds on the host, where `target/` keeps only final artifacts and the intermediates live in the
host build directory.

Per directory, with the old Cargo layout (`deps/`, `.fingerprint/`) and the new one (`build/<package>/<hash>/`) both
present in the same directory (the Cargo Book says the layout changed in 1.100.0, see [build cache][build-cache]):

- `package/ide/app/target`, 38.7 GiB, 104,897 files:
  `debug/incremental` 16.9, `debug/deps` 9.9, `release/deps` 3.6, `debug/build` 3.3, `dylint/debug/build` 2.4,
  `dylint/debug/incremental` 1.5, `runtime-config` 0.3, everything else under 0.3 each.
  `.rlib` 5.5, extensionless executables 3.9, `.rmeta` 3.4, `.so` 0.7.
  1209 incremental directories for 147 crates (63 for `ide_app` alone).
- `package/nested-wayland-session/cli/target`, 13.5 GiB, 18,945 files:
  `debug/build` 6.5, `debug/deps` 3.1, `release/deps` 1.5, `debug/incremental` 1.4, `release/build` 0.6.
  The largest single files are integration-test and fake-service executables of 200 to 250 MiB each.
- `target/verification`: 6.2 GiB across `package/git-policy/cli.fuzz` (2.6), `package/linter/monochromatic-lint.fuzz`
  (2.0), `package/git-policy/cli` (1.1), `package/linter/monochromatic-lint` (0.5) and `package/forbidden-strings/cli`
  (0.04).
  These are retained run directories (`campaign-*`, `native-*`, `git-image-*`, `mutation-*`): 27, 247, 20, 117 and 58
  directories, the oldest from 2026-10-04.
  The fuzz README says "Results are retained under `target/verification/planted-*`" and that campaign logs are
  retained under `target/verification/campaign-*`.
- `package/git-policy/cli.fuzz/target/debug`: 1.2 GiB in 14 executables of 83 MiB each, final artifacts.
- No two package targets share a `deps/` artifact name (6801 distinct names, none in two directories), because only
  the two container builds keep `deps/` in the tree.

### Which units and incremental state are stale

For the two container-built directories, grouping by the compiler recorded in each unit's fingerprint
(`.fingerprint/<pkg>-<hash>/*.json` or `build/<pkg>/<hash>/fingerprint/*.json`):

- `package/ide/app/target/debug`: current generation (newest `invoked.timestamp` 2026-11-01) 2.8 GiB;
  one older generation (newest 2026-10-29) 11.8 GiB of units; incremental state 18.0 GiB of which 16.5 GiB is in
  directories last written before the current generation began.
- `package/ide/app/target/dylint/debug`: current 0.9, older 1.6; incremental 1.5 of which 1.0 predates the current one.
- `package/ide/app/target/release`: one generation, 4.2 GiB, not rebuilt since 2026-10-29.
- `package/nested-wayland-session/cli/target/debug`: current 6.6, older 3.4 (newest 2026-10-05);
  `release`: current 0.6, two older generations 1.6.
- Removing every non-newest generation and the incremental directories older than it frees about 30.8 GiB of the
  40.9 GiB of units and incremental state in `package/ide/app/target`, and 5.6 GiB of 13.6 in
  `package/nested-wayland-session/cli/target`.
  Together 36.5 GiB of 54.5 GiB.
- What remains is dominated by current-generation test and fixture executables; this document does not propose
  changing how they are built.
- By file time alone (no fingerprint read), 46.6 of the 51.2 GiB under `deps/`, `build/` and `incremental/` in all
  main-checkout targets were last written before 2026-11-01, when the current pin `nightly-2026-11-01` was installed
  (`~/.rustup/toolchains` directory time 2026-10-31T22:57:24-04:00; `.cache/toolchain.env.json` pins it).
- Installed toolchains: 20 directories under `~/.rustup/toolchains`, about 2.7 GiB each, including 17 nightlies from
  2026-09-11 to 2026-11-01.
  Each retired one is a generation whose intermediates nothing will reuse.
- Why it happens: Cargo hashes the compiler version into each unit's metadata (`hash_rustc_version`,
  [`compilation_files.rs` lines 923 to 932][cargo-hash-rustc]), and its own notes say "Normally cargo updates ship with
  rustc updates which will cause a new hash due to the rustc version changing" (line 27).
  Nothing removes the old hash: upstream tracks it as [rust-lang/cargo#5026][cargo-5026]
  ("cargo ./target fills with outdated artifacts as toolchains are updated/changed", open).

### The host Cargo build directory

`~/.cargo/config.toml` (comment dated 2026-10-14) sets
`build-dir = "/var/mnt/encrypted/cargo-build/{workspace-path-hash}"`, relocated from `~/.cargo/build` after that
directory reached 438 GiB (`doc/troubleshooting/cargo-build-dir-workspace-path-hash-unbounded.md`).
The mount has 730 GiB free of 3.7 TiB.

- 3646 trees at measurement end (256 shard directories); 3545 existed when sizing started.
  Sizes by `du`: 6166 GiB in total; 1539 trees over 1 GiB, 1155 between 0.1 and 1 GiB, 851 under 0.1 GiB;
  the largest are 41.8, 38.7, 24.4, 23.8, 19.1 and 17.8 GiB.
- Trees by last write: 2026-10-14 to 2026-10-31 hold 697 GiB;
  2026-11-01 and 2026-11-02 hold 362 GiB; 2026-11-03 holds 3517 GiB; 2026-11-04 holds 1771 GiB.
  PEND-CLONE-EXPLANATION
- Mapping each tree to the workspace that owns it by the dep-info file next to its final artifact:
  PEND-CLASSES
- Layout inside a tree: the biggest trees contain both layouts and, in the 8 largest, 3 to 7 compiler generations per
  profile (`74/8a90f0615defdd` has 7 generations in `debug`: current 2.1 GiB against 14.0 GiB of units from older
  ones and 20.9 of 23.7 GiB of incremental state older than the current generation).
  For the 8 largest trees, a rule keeping the newest generation frees 83.7 of 181.4 GiB of units and incremental state.
- Unit-level duplication across trees: PEND-UNITS

### Worktrees

PEND-WORKTREES

### Gradle, Android, `dist/`, fuzz and `node_modules`

- Gradle cleans its own project cache and user-home caches: project `.gradle/<version>` directories are deleted after
  7 days unused, user-home caches after 7 or 30 days ([Gradle directory layout][gradle-cleanup]).
  `~/.gradle` is 3.2 GiB.
  `build/` has no automatic cleanup (Gradle's own text: "the latter are transient files Gradle uses to support features
  like incremental builds"); `package/music-player/android-app/mise.toml` already has a `clean` task
  (`./gradlew clean`).
  All Gradle `build/` directories together are 0.9 GiB, so no option below targets them.
- `dist/` outputs total 0.6 GiB; 218 directories, one at 0.44 GiB.
  The `session-start-housekeeping` plugin removes only stray Git metadata inside `dist/final` (its README).
- Fuzz `corpus` is input data (0.08 GiB), `artifacts` is 56 KiB.
- Root `node_modules`: `.monochromatic` already has retention inside its producer
  (`doc/decision/module-logger-log-accumulation.md` lines 5 to 12: "retention window 14 days and 512 MiB and 50,000 files"),
  and the pnpm store and `prefer-readonly-parameter-type` cache are rebuilt by their tools.

### Outside the checkout

`podman system df` reports 3462 images of 391.6 GB (389 GB reclaimable) and 33 volumes of 87.6 GB (86.6 GB
reclaimable); 198 images are `localhost/git-policy-native-test` and 38 `localhost/git-policy-native-mutation`.
Storage root `~/.local/share/containers/storage` (overlay).
These are build output of the container test tasks but not a directory of the monorepo; whether they are in scope is
an open question.

### What the Git wrapper's copy already leaves out

`package/git-policy/cli/src/native/worktree_copying/worktree_copy_prune.rs` skips `target/**/incremental`,
`target/**/zig-global-cache`, `target/verification`, `.cache`, `.pnpm-store`, `.gradle`, `.kotlin`, `.zig-cache`,
`node_modules/.monochromatic`, `mutants.out`, `playwright-report` and `test-results` for a main-checkout source.
Under that rule the target directories that remain to be copied, by apparent size, are 21.8 GiB for
`package/ide/app/target`, 12.1 GiB for `package/nested-wayland-session/cli/target`, 1.2 GiB for
`package/git-policy/cli.fuzz/target`, 0.5 GiB for `package/linter/monochromatic-lint/target`,
0.3 GiB for `package/git-policy/cli/target` and 0.1 GiB for `package/terminal/app/target`: 36.1 GiB.
That does not reconcile with the 49.5 GB per add in the brief; the difference is not explained by anything measured
here (an installed wrapper older than this rule, or ignored state outside `target/`, are the untested candidates).
Of the 36.1 GiB, the stale non-incremental units above are 13.3 GiB of `package/ide/app/target` and 5.0 GiB of
`package/nested-wayland-session/cli/target`.
If the wrapper binary in use predates the rule, the incremental state (19.3 GiB) is copied as well.

## What always lean can mean

A directory is lean when both hold:

- Currency: it contains no unit or incremental state that belongs to a compiler generation older than the newest one
  built in its profile directory (after a grace period the owner picks), no tree whose workspace no longer exists,
  and no run record older than the owner's retention.
- Size: it is no larger than a bound taken from one current build of its own configuration.

Proposed bounds, each backed by a measurement above and each needing the owner's decision before it is shipped
(a shipped limit has to cite a measurement or an owner decision beside the value):

- A package `target/` directory that keeps intermediates in the tree (container builds):
  at most one and a half times the bytes of its current generation, measured now at 2.8 + 0.9 + 4.2 GiB of units and
  about 1.5 GiB of incremental state for `package/ide/app/target` (about 9.4 GiB, bound about 14 GiB) and
  6.6 + 0.6 GiB plus 0.8 GiB of incremental state for `package/nested-wayland-session/cli/target` (about 8 GiB, bound
  about 12 GiB), against 38.7 and 13.5 GiB now.
- A package `target/` directory that holds final artifacts only (host builds): at most 2 GiB; every one except the
  verification run records measures under 1.2 GiB now.
- `target/verification`: bounded by retention the owner picks; today 6.2 GiB, oldest 2026-10-04.
- Host Cargo build trees: no tree whose workspace path is gone; each tree at most one and a half times its current
  generation (the largest current generation measured is 13.2 GiB for `0b/ab24900891e355` and 8.4 GiB for the clones
  `75/`, `76/`, `77/`), and the base directory at most the number of live workspaces times that bound.
- Gradle `build/`, `dist/`, fuzz `corpus` and `artifacts`, root `node_modules` caches: no new bound; each is under 1.2
  GiB and owned by a tool that already retains or rebuilds it.

A check that stays true: a read-only walk reports every profile directory with more than one compiler generation,
every tree whose workspace is gone, and every directory over its bound.
That check is the acceptance test for each option.

## Mechanisms researched

Cargo (version in use: 1.101.0-nightly, 2026-10-27; there are no `rust-toolchain` files in the repository, the pin is
`RUSTUP_TOOLCHAIN` from `.cache/toolchain.env.json`):

- `build.build-dir` and `{workspace-path-hash}`: "The directory where intermediate build artifacts will be stored"
  ([config reference][cargo-config-build-dir]); the layout inside is "internal to Cargo, and is subject to change"
  ([build cache][build-cache]).
- `cache.auto-clean-frequency` covers only the global cache in the Cargo home; the reference says support for tracking
  build artifacts "is not yet implemented" ([config reference][cargo-config-cache]).
- `cargo clean`: with no options it deletes the entire target directory; `--dry-run` is a read-only preview;
  `-p <package>` removes every hash of that package (current and stale alike: `src/ops/cargo_clean.rs` matches
  `starts_with("<crate>-")` in `incremental` and by package name in fingerprints and `build`), so it is "rebuild this
  package" and not "drop only stale" ([cargo-clean(1)][cargo-clean]).
  `cargo clean -p` takes the exclusive build-directory lock (`Layout::new(..., true, true)`, `cargo_clean.rs`
  line 201), so it waits for a running build.
  A full `cargo clean` does not: "we don't bother grabbing a lock here" (`cargo_clean.rs` lines 129 to 131), so it can
  delete under a running build.
- `-Zgc` and `cargo clean gc` exist on this cargo (`cargo clean gc --help`) but govern only the global cache
  (source, crate, index and git caches; [unstable reference][cargo-gc]).
- Locks: a build holds `<build-dir>/<profile>/.cargo-build-lock` exclusively unless `-Zfine-grain-locking`
  ([`layout.rs` lines 264 to 285][cargo-layout]), implemented with `flock`
  ([`flock.rs` lines 161 to 174][cargo-flock]).
  A sweeper that takes the same lock waits for or excludes cargo builds in that profile directory, not in others.
- Upstream direction: [rust-lang/cargo#13136][cargo-13136] (garbage collect whole `target/`, open) and
  [rust-lang/cargo#5026][cargo-5026]; a maintainer's plan is to hash the Cargo version into `{workspace-path-hash}` so
  each version gets its own build directory that can be tracked and cleaned
  ([comment of 2025-04-14][cargo-5026-plan]).
  [rust-lang/cargo#6229][cargo-6229] (clean up within `target`) was closed as a duplicate of #5026 on 2026-08-13.
  No released Cargo feature removes stale intermediates.
- `-Zbuild-dir-new-layout` and the 1.100.0 layout change move toward one directory per unit, which makes
  per-unit removal simpler later; today both layouts are in the same directories.

Third-party:

- [cargo-sweep][cargo-sweep] (age, installed toolchains, size, stamp-and-sweep): its README opens with "currently
  unmaintained", and an open issue asks for `build-dir` support ([holmgr/cargo-sweep#140][sweep-140]); it reads
  `target/`, which holds nothing but final artifacts for host builds.
- [cargo-clean-all][cargo-clean-all]: finds and removes whole `target` directories by age; one open issue says
  `CARGO_TARGET_DIR` is not considered.
- [sccache][sccache-rust]: requires incremental compilation off ("`rustc`'s incremental compilation needs to be
  disabled"), and `bin`, `cdylib` and `proc-macro` crates cannot be cached; it would also not shrink on-disk output.

Profile settings (`debug`, `split-debuginfo`, `strip`, `codegen-units`, `incremental`): excluded by the owner on
2026-11-04; not researched or proposed.

Non-cargo build systems found: Gradle (retains its own caches; project `build/` is removed by `gradlew clean`),
tsdown `dist/` (rebuilt per task; `package/ssg/aquati.cat` and the Electron packages already have `build:clean`),
Zig (`zig-global-cache`, skipped by the wrapper copy).

## What the repository already owns

Records that bear on this, quoted:

- `doc/decision/cargo-build-dir-host-cache.md` line 17: "Accepted 2026-09-19. The user set it directly in
  `~/.cargo/config.toml` on this machine and asked for it to be recorded here."
  Lines 292 to 297: worktree builds "outlive their worktree"; "Worktrees created and discarded by cli-git accumulate
  this way."
  Lines 299 to 315: "Nothing prunes `~/.cargo/build/` automatically"; reclaim by `cargo clean` or by deleting
  the hash directories of workspace paths no longer built.
  Lines 317 to 342: container builds "keep writing full intermediates into the repository tree. Expect a mixed state".
  Its section on the layout says "Do not write repository tooling that parses these paths."
  The sweep below has to read fingerprint files, which conflicts with that line; the owner decides whether the
  conflict is accepted (a reader that does nothing when it cannot parse is the mitigation proposed).
- `doc/troubleshooting/cargo-build-dir-workspace-path-hash-unbounded.md`: 438 GiB from 284 trees in three weeks;
  lists "Periodic removal of trees not modified within N days" with the tradeoff that "mtime reflects the last build,
  so a workspace that is used but rarely built loses its cache".
- `package/git-policy/cli/src/native/repair/package_repair_cargo.rs` line 8: the owner chose on 2026-11-03 "Reuse deps,
  rebuild own", done with "Run cargo clean -p"; the wrapper clones the source workspace's build directory for each
  new worktree, which is why trees repeat (three trees of 15.9 GiB have identical contents).
- `doc/decision/module-logger-log-accumulation.md` lines 5 to 12, accepted by the owner on 2026-10-22: retention
  inside the producer, swept at open, with an age, size and file-count cap.
  This is the precedent for a component owning the retention of what it writes.
- `doc/decision/ci-caching.md` lines 185 to 187: "Cargo never prunes artifacts of an older compiler, so restoring a
  stale `target/` only enlarges the next save."
  `doc/decision/music-player-windows-linkers.md` lines 135 to 147: the Windows jobs prune the package's own artifacts
  before saving.
- `doc/decision/dist-in-git.md` line 163: `session-start-housekeeping` "already cleans stale dist artifacts on session
  start"; per its README it removes only stray Git metadata and fires only for Claude Code sessions.
- `package/cargo-shadow/cli/README.md`: every repository cargo run is the shadow
  (".cache/cargo-shadow-bin/cargo"); it already runs a step before cargo and has subcommands (`install`, `update`).
  It does not cover cargo inside containers.
- `toolchain:update` (`mise.toml` line 535) is where a new compiler arrives.
  No existing task or tool cleans Cargo output; the only `clean` tasks are Gradle, Electron `build:clean` and the
  site `rm -rf dist`.

## Options

Each option says what it frees here, what it costs, how it stays automatic, how it behaves while another session
builds in the same directory, and which component owns it.
The sweep options remove whole units, never part of a unit, and move a unit into a trash directory by an atomic
rename before deleting it, so a reader sees either the unit or nothing.

### A: sweep older compiler generations out of every Cargo output directory

- What: a repository tool reads each profile directory's fingerprints, keeps the newest compiler generation (plus any
  generation built within a grace period the owner picks), and removes the other generations' units and the
  incremental directories older than the kept one.
- Frees here: 36.5 GiB of the 54.5 GiB of units and incremental state in the two container-built targets, which is
  all of the stale non-incremental part of the 36.1 GiB copy set (18.3 GiB) and, if the wrapper in use copies
  incremental state, 18.2 GiB more; 83.7 GiB of the 181.4 GiB in the 8 largest host trees.
- Costs: it parses Cargo's internal fingerprint files, which two layouts and a pending layout change make a standing
  maintenance cost, against the line in `doc/decision/cargo-build-dir-host-cache.md` that says not to.
  Fail-safe: a tree it cannot parse is left alone and logged.
  A generation removed while another toolchain is still in use (a host build and a container build alternating in
  one directory) costs a rebuild, never wrong output.
- Automatic: run by the owner component (a) at the first Cargo run after the compiler's identity changes for a
  workspace, (b) after `toolchain:update`, and (c) after a container task finishes, on the bind-mounted `target/`.
- Concurrency: per profile directory it takes the exclusive `.cargo-build-lock`, the lock `cargo clean -p` takes.
  It uses a non-blocking attempt and skips a busy directory until the next trigger, so it never delays a build and
  never deletes under one.
  Held only for the renames; deletion happens after the lock is released.
  A cargo build in another worktree uses another tree and is untouched.
- Owner (proposed): `package/cargo-shadow/cli`, as a new subcommand beside `install` and `update`, because it is
  already on the path of every host cargo run and ships the same release cycle as the repository's Cargo policy.
  Alternative owners: a new package (more surface), `toolchain:update` alone (misses containers and workspaces
  built on a compiler that was never "updated" in this checkout).

### B: collect build trees whose workspace no longer exists, and idle ones

- What: the owner component records, for every cargo run, the workspace root and the build directory Cargo reports
  (`cargo metadata` names `build_directory`, which `package_repair_cargo.rs` already reads), at most once a day removes
  trees whose workspace root is gone, and removes trees idle longer than a period the owner picks.
  A one-time backfill maps the existing trees by the dep-info file next to each tree's final artifact.
- Frees here: PEND-B
- Costs: a ledger to maintain; idle removal costs a rebuild for a workspace that is used but rarely built (the cost
  the troubleshooting record names).
- Automatic: runs from the same trigger as A, rate-limited like Cargo's own "1 day" auto-clean frequency.
- Concurrency: a tree is removed only when its workspace root is gone (nothing can be building in it) or when the
  non-blocking exclusive lock on every profile directory succeeds; otherwise it is skipped.
- Owner (proposed): the same component as A.
  A different owner is possible for the worktree case: the component that creates the clone trees
  (`package_repair_cargo.rs`) knows the exact path it created and could record it for removal when the worktree is
  removed, but the owner kept leanness out of the wrapper, so this needs an explicit yes.

### C: per-directory size caps with oldest-first eviction

- What: after A and B have run, a cap per directory (section "What always lean can mean") evicts the units with the
  oldest `invoked.timestamp` until the directory is under it, the same shape as the logger's 14 days and 512 MiB and
  50,000 files.
- Frees here: nothing that A and B leave if a cap is set above the current generation; its job is to make "always"
  true when one generation alone outgrows its bound.
- Costs: a cap set too low evicts units that the next build recompiles; it needs a number from the owner.
- Automatic and concurrent: as A.
- Owner: the same component.

### D: generation-keyed build directories (upstream's own plan, applied locally)

- What: the shadow sets `CARGO_BUILD_BUILD_DIR` to `<base>/<toolchain>/<workspace key>` for host runs, so retiring a
  toolchain is removing one directory, with no fingerprint parsing; the maintainer plan in
  [cargo#5026][cargo-5026-plan] is the same shape.
- Frees here: the stale generations of the host trees on first retirement; not the container-built targets and not
  runs that do not go through the shadow (an IDE launching Cargo directly keeps using `~/.cargo/config.toml`).
- Costs: one cold rebuild of every host workspace when the location changes; the environment variable is used
  verbatim ("No template expansion is applied to a literal path given this way", `cargo-build-dir-host-cache.md`), so
  the shadow must compute its own workspace key, which also gives it the workspace-to-tree map B needs.
- Concurrency: a retired directory is removed only when no process holds its locks.
- Owner: the shadow.

### E: split container builds' intermediates away from `target/`

- What: the container tasks (`package/ide/app`, `package/nested-wayland-session/cli`, `package/terminal/app`,
  `package/music-player/desktop-app`) pass `CARGO_BUILD_BUILD_DIR` pointing into a named volume, as they already keep
  the registry in a named volume (`ide-cargo`, `nested-wayland-session-cargo`).
  Then the in-tree `target/` holds final artifacts only, like host builds.
- Frees here: the in-tree copy set shrinks from 36.1 GiB to final artifacts (a few GiB); physical bytes move into the
  volumes (87.6 GB of volumes exist already) and still need A to B to stay lean.
- Costs: one cold container build per package; a volume per package is shared by every worktree of the package, and
  inside the container every worktree is `/work`, so the workspace-path hash is the same: worktrees would thrash one
  tree, and a volume per worktree would need its own lifecycle.
- Owner: each container package's `mise.toml` runner, so many files.

### F: a timer-driven runner

- What: a `systemd --user` timer runs the tool of A to C daily.
- Frees: what A to C free, without waiting for a trigger.
- Costs: a new unit to install, which the repository's toolchain install does not do today; runs when nobody builds.
- Concurrency: as A.
- Owner: a new component; only a delivery vehicle, so it ranks as an addition and never alone.

### G: track upstream garbage collection and adopt it when it ships

- What: do nothing now; adopt the Cargo feature when [#13136][cargo-13136] and [#5026][cargo-5026] land.
- Frees: nothing now.
- Costs: the directories keep growing meanwhile (about 20 GiB per day was measured before the relocation).
- Owner: Cargo; the repository keeps a pointer in `doc/troubleshooting`.

### H: status quo, manual `cargo clean`

- What: as `doc/decision/cargo-build-dir-host-cache.md` lines 309 to 315 say.
- Frees: whatever someone remembers; a full `cargo clean` takes no lock (see Mechanisms), so it can break a running
  build.
- Owner: nobody.

Rejected without ranking:
- A shared `build.build-dir` without the workspace hash: removes the growth but makes every session contend for one
  lock and lets worktrees of one package overwrite each other's units
  (`doc/troubleshooting/cargo-build-dir-workspace-path-hash-unbounded.md` lists it as bounded, not as safe).
- sccache: see Mechanisms.
- Btrfs deduplication: it lowers physical use but not the bytes a cross-filesystem copy reads, which is the motivating
  cost.
- Any profile setting: excluded by the owner.

## Ranking

PEND-RANKING

## Open questions for the owner

PEND-QUESTIONS

## Review

PEND-REVIEW

[btrfs-du]: https://btrfs.readthedocs.io/en/latest/btrfs-filesystem.html#:~:text=Each%20argument%20to%20btrfs%20filesystem%20du%20will%20have%20a%20set%20shared%20value%20calculated%20for%20it
[build-cache]: https://doc.rust-lang.org/cargo/reference/build-cache.html#:~:text=The%20build-dir%20layout%20is%20considered%20internal%20to%20Cargo
[cargo-hash-rustc]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/compiler/build_runner/compilation_files.rs#L923-L932
[cargo-5026]: https://github.com/rust-lang/cargo/issues/5026
[cargo-5026-plan]: https://github.com/rust-lang/cargo/issues/5026#issuecomment-2803122104
[cargo-13136]: https://github.com/rust-lang/cargo/issues/13136
[cargo-6229]: https://github.com/rust-lang/cargo/issues/6229
[cargo-config-build-dir]: https://doc.rust-lang.org/cargo/reference/config.html#:~:text=The%20directory%20where%20intermediate%20build%20artifacts%20will%20be%20stored
[cargo-config-cache]: https://doc.rust-lang.org/cargo/reference/config.html#:~:text=Support%20for%20tracking%20build%20artifacts%20is%20not%20yet%20implemented
[cargo-clean]: https://doc.rust-lang.org/cargo/commands/cargo-clean.html#:~:text=With%20no%20options%2C%20cargo%20clean%20will%20delete%20the%20entire%20target%20directory
[cargo-gc]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#:~:text=The-Zgc%20flag%20is%20used%20to%20enable%20certain%20features%20related%20to%20garbage-collection
[cargo-layout]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/compiler/layout.rs#L264-L285
[cargo-flock]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/util/flock.rs#L161-L174
[cargo-sweep]: https://github.com/holmgr/cargo-sweep#:~:text=currently%20unmaintained
[sweep-140]: https://github.com/holmgr/cargo-sweep/issues/140
[cargo-clean-all]: https://github.com/dnlmlr/cargo-clean-all
[sccache-rust]: https://github.com/mozilla/sccache/blob/main/docs/Rust.md#:~:text=incremental%20compilation%20needs%20to%20be%20disabled
[gradle-cleanup]: https://docs.gradle.org/current/userguide/directory_layout.html#:~:text=They%20are%20deleted%20if%20they%20haven%27t%20been%20used%20for%207%20days
