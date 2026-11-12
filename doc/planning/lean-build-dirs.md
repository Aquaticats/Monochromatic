# Keeping every build directory lean

Proposal and measurements,
2026-11-04T17:45:00-05:00 (measurement round,
read-only).
Nothing was deleted and no build configuration was changed in that round.

Status,
2026-11-07T23:40:00-05:00:
the owner decided in the question tool ("A to F,
B allowed (Recommended)")
and the six mechanisms are built in `package/cargo-shadow/cli`.
Section "Owner decision and what was built" says what exists,
section "Measured picks" records the numbers with the owner's veto open,
and each item of "Open questions for the owner" now names its answer.

Scope:
every build directory of the monorepo at `/var/home/user/Monochromatic`:
Rust `target/` directories,
the Cargo build directory the host configuration keys by workspace path,
fuzz and verification output,
Gradle and Android output,
`dist/`,
`node_modules` and `.cache`.

## Owner requirements that bound this

- 2026-11-04,
  answering how the Git wrapper should handle copying build output into new worktrees:
  "No lefting out build outputs.
  The real fix is to make sure every build dir is always lean.
  That's not the wrapper's responsibility,
  but I'm assigning this to you to."
  So the fix is not in the Git wrapper (`package/git-policy/cli`).
- 2026-11-04,
  on guidance in `AGENTS.md`:
  "No exposing underlying machinary to always-loaded context."
  So no `AGENTS.md` rule is proposed.
- 2026-11-04:
  "(debuginfo level,
  incremental off for dependencies - no messing with these."
- 2026-11-04:
  "other profile settings (split-debuginfo,
  strip,
  codegen-units) should be off-limits too".
  So no build profile setting is changed or proposed here,
  and only mechanisms that keep a directory lean without
  changing what a build produces are ranked.
- Standing owner decisions on the copy itself,
  from `doc/planning/cli-git-main-worktree-ignored-copy.md` lines 15 to 20:
  "Builds must be copied.
  Other ignored files must be copied."
  (2026-11-02);
  "Skip nested worktrees and caches.
  The point is to make new worktrees immediately functional."
  (2026-11-02);
  "Skip run records (Recommended)."
  (2026-11-02).
  Leanness therefore has to come from the directories being small,
  not from the copy leaving them out.

## Summary

- A lean build directory here means two things hold at every quiescent moment (no build holds its lock):
  nothing in it can no longer be used by the compiler,
  workspace or run that a current build would use
  (after a grace period the owner picks),
  and its size is within a bound derived from what one current build needs
  (section "What always lean can mean").
  The mechanisms in this document reach that state eventually after a trigger;
  they cannot hold it during a running
  build.
- The main checkout holds 67.5 GiB in build directories by `du` (1 KiB blocks,
  2026-11-04),
  of which 62.5 GiB are
  Rust `target/` directories.
  Two container-built packages are 52.2 GiB of that:
  `package/ide/app/target` (38.7 GiB) and
  `package/nested-wayland-session/cli/target` (13.5 GiB).
- Cargo does not hash a nightly's date or commit into a unit's directory names ("whenever someone updates nightly,
  they won't have a bunch of stale artifacts",
  `compilation_files.rs` lines 939 to 941),
  but its fingerprint records the
  full compiler version (`fingerprint/mod.rs` line 1714).
  A nightly update therefore makes every unit dirty in place;
  the units that no later build rebuilt stay behind with an
  older compiler recorded and no reuse value under the newer one.
  The repository installed 16 dated nightlies from 2026-09-11 to 2026-11-01 and pins one per checkout.
  Grouping the units by that recorded compiler,
  34.0 GiB of the 51.7 GiB of units and incremental state in the two
  container-built directories were last built by a compiler older than the newest one in their profile directory.
  Cargo never removes them (upstream [rust-lang/cargo#5026][cargo-5026],
  open).
- The host Cargo build directory `/var/mnt/encrypted/cargo-build` holds 3545 sized trees (plus 101 created while
  measuring) that add up to 6166 GiB by `du`.
  Much of that is shared extents (clones of one tree by the wrapper's Cargo repair,
  and reflink copies inside trees),
  so
  `du` is not the disk it occupies.
  The whole-directory physical figure was not obtained:
  a single `btrfs filesystem du --summarize` over the base was
  started and stopped unfinished when the owner paused non-layout work.
  Per-tree samples (the first 770 trees,
  shards 00 up to about 34) total 1251 GiB,
  of which 247 GiB are exclusive to
  their tree and 441 GiB are extents shared (counted once per tree,
  so an upper bound on their physical size).
- Worktrees:
  335 registered;
  their build directories add up to 3218 GiB by `du`,
  mostly shared extents on the home
  filesystem,
  but a byte copy on `/var/mnt/encrypted`:
  the sampled copy of `package/ide/app/target` there is 21.47 GiB,
  all exclusive.
- Ranking (reasons in section "Ranking"):
  toolchain-keyed build directories,
  then an in-place sweep of units built by an
  older compiler,
  then verified-idle tree collection,
  then retention of run records at their producers,
  then size caps,
  then a scheduler for dormant directories,
  then a container volume split,
  then tracking upstream,
  and last
  a manual offline cleanup.

## Method and limits

- Everything ran read-only inside `systemd-run --user --scope` with `IOWeight=1`,
  `CPUQuota` of at most 100 percent,
  a memory cap and a task cap,
  under `ionice --class=3`.
  The scripts read file names,
  sizes and times.
  The only file contents read are the first 400 bytes of one fingerprint JSON per Cargo unit (to read its `rustc`
  field) and at most the first 900 bytes of up to a few hundred dep-info files per host tree (to find a source path).
- No build was run to measure build time.
- `du` counts every extent of every file in full,
  including extents shared by reflink or snapshot.
  `btrfs filesystem du` ([btrfs-filesystem(8)][btrfs-du]) reports `Exclusive` (extents referenced once) and `Set shared`
  per argument,
  "those files found by a recursive search of an argument",
  with overlapping shared extents counted once.
  `Exclusive 0.00B` means every extent has more than one reference;
  it does not say where the other reference is.
  Where `Total` equals `Set shared` (the main checkout's `package/ide/app/target`:
  38.49 and 38.39 GiB;
  `package/nested-wayland-session/cli/target`:
  13.45 and 13.45 GiB) no extent is referenced twice inside the directory,
  so the other references are outside it:
  consistent with snapshot or reflink worktrees (91 of the 139 worktrees nested in
  the main checkout were subvolumes on 2026-11-02,
  per `doc/planning/cli-git-main-worktree-ignored-copy.md`),
  but the sharers were not identified.
  Where `Total` exceeds `Set shared` (a host tree:
  15.76 against 6.60 GiB) extents are also referenced more than once
  inside the tree.
  `du` is the right measure for the cost of a copy across filesystems (a byte copy) and the wrong measure for disk
  pressure;
  no per-directory figure here proves how many bytes deleting that directory would free.
  All physical figures are per argument.
- The "older compiler" figures group Cargo units by the `rustc` field of their fingerprint,
  which is a hash of the full
  `rustc -vV` text (`fingerprint/mod.rs` line 1714),
  treat the group whose `invoked.timestamp` is newest in each profile
  directory as current,
  and sum the others.
  Bytes are counted once per inode (hardlinks),
  attributed to the first unit that reaches them.
  A profile directory not rebuilt since the last toolchain change counts as fully current,
  so a dormant directory shows
  no older-compiler bytes.
  Two toolchains alternating in one directory (a host build and a container build) make both groups live,
  so the
  figures are what a rule keeping the newest group would remove,
  not proof that nothing needs the rest.
- Dep-info files list source paths and are used here only to attribute host trees to a workspace.
  One file cannot prove ownership:
  a tree can name a dependency,
  a copied path or a path inside another tree.
  The classes are investigation leads,
  not authority to delete.
- The `find` on this machine resolves to a search shim inside `sh`;
  the inventory was repeated with `/usr/bin/find`
  and found the same 37 `target`,
  218 `dist`,
  5 `build`,
  3 `corpus`,
  3 `artifacts`,
  2 `.gradle` and 1 `out`
  directories in the main checkout.
- Not measured:
  `~/.cargo/registry` and `~/.cargo/git` (Cargo garbage-collects those),
  the pnpm content store outside the checkout,
  Android emulator images and the SDK.

## Measurements

### The main checkout, by kind

Sizes are `du --summarize --one-file-system` in 1 KiB blocks,
2026-11-04,
converted to GiB.

- `target/` directories:
  62.5 GiB in 38 directories (37 packages plus the repository root).
- `node_modules` at the root:
  3.1 GiB,
  of which `node_modules/.monochromatic` (module-logger files) is 1.2 GiB,
  `node_modules/.pnpm-store` 1.1 GiB and `node_modules/.cache` 0.8 GiB
  (`node_modules/.cache/prefer-readonly-parameter-type`).
- Gradle `build/`:
  0.9 GiB in 5 directories
  (`package/music-player/android-app/app/build` 0.54,
  `package/linter/kotlin/build` 0.38,
  `package/music-player/android-app/build`,
  `package/linter/linter/kotlin/build`,
  `package/ssg/aquati.cat/src/build`
  small);
  `.gradle`:
  5 MiB in 2 directories.
- `dist/`:
  0.6 GiB in 218 directories,
  of which `package/upstream-fork/deepmerge-ts.fuzz/dist` is 0.44 GiB
  and `package/ide/app/dist` 0.09 GiB.
- `package/music-player/truepeak-core.bench/out`:
  0.14 GiB.
- Fuzz `corpus`:
  0.08 GiB in 3 directories;
  fuzz `artifacts`:
  56 KiB in 3 directories.
- Root `.cache`:
  0.05 GiB;
  229 nested `.cache` directories under `package/`:
  each at most 2 MiB.
- `playwright-report`,
  `test-results`:
  negligible.
- Total:
  70,739,320 KiB,
  which is 67.5 GiB.

The brief's seven largest directories (65.6 GiB) matched.
Directories it did not list:
`package/terminal/app/target` (0.56 GiB,
mostly a Zig cache),
`package/music-player/android-app/app/build` (0.54 GiB),
`package/upstream-fork/deepmerge-ts.fuzz/dist` (0.44 GiB),
`package/linter/kotlin/build` (0.38 GiB),
`package/music-player/truepeak-core.bench/out` (0.14 GiB),
and `package/forbidden-strings/core.fuzz/corpus` (0.04 GiB).

### What is in the Rust `target/` directories

Two directories are container builds with the package directory bind-mounted at `/work` and no
`CARGO_BUILD_BUILD_DIR` (`package/ide/app/mise.toml` line 38,
`package/nested-wayland-session/cli/mise.toml` line 52),
so the host setting from `doc/decision/cargo-build-dir-host-cache.md` never reaches them;
that record predicts this
mixed state at line 340.
Every other package builds on the host,
where `target/` keeps only final artifacts and the intermediates live in the
host build directory.

Per directory,
with the old Cargo layout (`deps/`,
`.fingerprint/`) and the new one (`build/<package>/<hash>/`) both
present in the same directory (the Cargo Book says the layout changed in 1.100.0,
see [build cache][build-cache]):

- `package/ide/app/target`,
  38.7 GiB,
  104,897 files:
  `debug/incremental` 16.9,
  `debug/deps` 9.9,
  `release/deps` 3.6,
  `debug/build` 3.3,
  `dylint/debug/build` 2.4,
  `dylint/debug/incremental` 1.5,
  `runtime-config` 0.3,
  everything else under 0.3 each.
  `.rlib` 5.5,
  extensionless executables 3.9,
  `.rmeta` 3.4,
  `.so` 0.7.
  1209 incremental directories for 147 crates (63 for `ide_app` alone).
- `package/nested-wayland-session/cli/target`,
  13.5 GiB,
  18,945 files:
  `debug/build` 6.5,
  `debug/deps` 3.1,
  `release/deps` 1.5,
  `debug/incremental` 1.4,
  `release/build` 0.6.
  The largest single files are integration-test and fake-service executables of 200 to 250 MiB each.
- `target/verification`:
  6.2 GiB across `package/git-policy/cli.fuzz` (2.6),
  `package/linter/monochromatic-lint.fuzz`
  (2.0),
  `package/git-policy/cli` (1.1),
  `package/linter/monochromatic-lint` (0.5) and `package/forbidden-strings/cli`
  (0.04).
  These are retained run directories (`campaign-*`,
  `native-*`,
  `git-image-*`,
  `mutation-*`):
  27,
  20,
  247,
  117 and 58
  directories,
  the oldest from 2026-10-04.
  The fuzz README says "Results are retained under `target/verification/planted-*`" and that campaign logs are
  retained under `target/verification/campaign-*`.
- `package/git-policy/cli.fuzz/target/debug`:
  1.2 GiB in 14 executables of 83 MiB each,
  final artifacts.
- No two package targets share a `deps/` artifact name (6801 distinct names,
  none in two directories),
  because only
  the two container builds keep `deps/` in the tree.

### Which units and incremental state were built by an older compiler

Cargo hashes a nightly's channel name but not its date or commit into a unit's metadata,
so an update of the nightly
leaves the directory names unchanged and makes every unit dirty:
the fingerprint's `rustc` field is a hash of the full
`rustc -vV` text,
and cargo recompiles any unit whose recorded value differs,
in place.
The source says so:
"On "nightly"/"beta"/"dev"/etc,
keep each "channel" separate.
Don't hash the date/git information,
so that whenever someone updates "nightly",
they won't have a bunch of stale artifacts in the target directory"
([`compilation_files.rs` lines 939 to 941][cargo-hash-rustc]);
`-Zseparate-nightlies` would change that and is not set here (`cargo config get unstable` finds nothing;
no
`CARGO_UNSTABLE_*` variable).
A unit last built by an older compiler is therefore recompiled in place if a build needs it and dead weight if no build
does;
either way it has no reuse value under the newest compiler.
That is a narrower statement than "orphaned by the toolchain change",
and it is the one the measurements support.

Grouping by the compiler recorded in each unit's fingerprint
(`.fingerprint/<pkg>-<hash>/*.json` or `build/<pkg>/<hash>/fingerprint/*.json`),
with hardlinks counted once,
for the
two container-built directories:

- `package/ide/app/target`:
  units and incremental state 38.3 GiB.
  Units of the newest compiler 7.8 GiB (`debug` 2.8,
  `dylint/debug` 0.9,
  `dylint/release` 0.2,
  `release` 4.2);
  units last built by an older compiler 12.7 GiB (`debug` 11.8,
  `dylint/debug` 0.9);
  incremental state 17.8 GiB,
  of which 15.9 GiB is in directories last written before the newest compiler's first unit.
  The older group in `debug` was last built 2026-10-29,
  the newest 2026-11-01.
  Layout:
  3411 units in the old layout and 1371 in the new one in `debug`,
  1844 new-layout units in `dylint/debug`.
- `package/nested-wayland-session/cli/target`:
  13.4 GiB;
  newest compiler 7.1 GiB,
  older 4.8 GiB (newest of the older
  generations 2026-10-05 in `debug`),
  incremental 1.4 GiB of which 0.6 GiB predates the newest compiler.
- Together,
  34.0 GiB of 51.7 GiB (28.6 of 38.3 and 5.4 of 13.4).
Of that,
  17.5 GiB (12.7 and 4.8) are non-incremental units that sit in the copy set under the wrapper's rule (section
  on what the copy leaves out).
- Dormant directories are not caught by this grouping:
  `0b/ab24900891e355` (17.8 GiB,
  last written 2026-10-15) has one
  compiler group,
  so it shows nothing older although its single group is 20 days old.
  Idle time,
  not the compiler group,
  is the only signal for it.
- What remains is dominated by newest-compiler test and fixture executables of 200 to 250 MiB each in
  `package/nested-wayland-session/cli/target`;
  this document does not propose changing how they are built.
- By file time alone (no fingerprint read),
  46.6 of the 51.2 GiB under `deps/`,
  `build/` and `incremental/` in all
  main-checkout targets were last written before 2026-11-01,
  when the current pin `nightly-2026-11-01` was installed
  (`~/.rustup/toolchains` directory time 2026-10-31T22:57:24-04:00;
  `.cache/toolchain.env.json` pins it).
- Installed toolchains:
  20 directories under `~/.rustup/toolchains`,
  about 2.7 GiB each,
  including 16 dated nightlies from 2026-09-11 to 2026-11-01.

### The host Cargo build directory

`~/.cargo/config.toml` (comment dated 2026-10-14) sets
`build-dir = "/var/mnt/encrypted/cargo-build/{workspace-path-hash}"`,
relocated from `~/.cargo/build` after that
directory reached 438 GiB (`doc/troubleshooting/cargo-build-dir-workspace-path-hash-unbounded.md`).
The mount has 730 GiB free of 3.7 TiB.

- 3646 trees at measurement end (256 shard directories);
  3545 existed when sizing started.
  Sizes by `du`:
  6166 GiB in total;
  1539 trees over 1 GiB,
  1155 between 0.1 and 1 GiB,
  851 under 0.1 GiB;
  the largest are 41.8,
  38.7,
  24.4,
  23.8,
  19.1 and 17.8 GiB.
- Trees by last write:
  2026-10-14 to 2026-10-31 hold 517 GiB;
  2026-11-01 and 2026-11-02 hold 362 GiB;
  2026-11-03 holds 3517 GiB;
  2026-11-04 holds 1771 GiB.
  Most of these are not build days.
  The wrapper's Cargo repair clones a source workspace's build tree for each new worktree and keeps file times
  (`package/git-policy/cli/src/native/repair/package_repair_tree_clone.rs`),
  and three of the largest trees
  (`75/3315c269a08379`,
  `76/f291bafb8314c3`,
  `77/6e49526b9630eb`) have identical file counts,
  byte totals and per-day
  write profiles (file contents were not compared).
  Each reports `Exclusive 0.00B`,
  so every extent has another reference;
  that is consistent with clones,
  but the other
  references were not identified,
  and `du` counts the same extents once per tree.
- Attributing each tree to a workspace by the first dep-info file that names a source outside the registry,
  git and
  toolchain directories (the Cargo Book documents dep-info files as Makefile-style lists with absolute paths),
  for the
  3545 sized trees;
  these are leads,
  not ownership records:
  - 1419 trees (1555 GiB) whose dep-info names paths inside another tree:
    consistent with the Cargo repair's clones.
  - 887 trees (1420 GiB) whose sources are an unpacked crate under another tree's `package/<crate>-<version>`
    directory:
    consistent with `cargo package` and `cargo publish --dry-run` verification builds,
    which unpack the crate
    under the build directory and build it at that new path (and so under a new workspace hash).
    17 crates,
    up to 90 trees for one crate,
    almost all last written on 2026-11-03 and 2026-11-04.
    The number of runs and their cause were not established here;
    `package/*/mise.toml` have `package` and
    `publish:dry-run` tasks.
    A tree built from sources inside another tree can be building while that other tree looks idle.
  - 436 trees (743 GiB) whose sources are in the main checkout:
    262 (438 GiB) in a package directory that exists,
    174 (305 GiB) in one that is gone.
  - 60 trees (96 GiB) whose sources are in a `.claude/worktrees` checkout:
    26 (48 GiB) existing,
    34 (48 GiB) gone.
  - 21 trees (15 GiB) whose sources are in `~/temp/agent` or `/tmp`:
    18 gone.
  - 715 trees (2334 GiB) whose dep-info names only toolchain or dependency sources that no longer exist,
    so no
    workspace is identifiable;
    7 trees (4 GiB) have no dep-info.
- Layout inside a tree:
  the biggest trees contain both layouts and,
  in the 8 largest,
  up to 7 compiler groups per
  profile (`74/8a90f0615defdd` has 7 groups in `debug`:
  newest 2.5 GiB of units against 14.9 GiB of units from older
  ones and 20.9 of 23.6 GiB of incremental state older than the newest group).
  For the 8 largest trees,
  a rule keeping the newest group removes 98.9 of 179.1 GiB of units and incremental state
  (three of the eight are the identical-looking trees `75/`,
  `76/` and `77/`,
  each contributing 5.1 GiB).
- Per-class sample of the first 770 trees sized one by one (GiB total,
  exclusive,
  set shared):
  clone-shape 349 trees
  348,
  101,
  114;
  package-shape 185 trees 244,
  45,
  81;
  main-checkout sources 105 trees 181,
  18,
  81;
  unidentified 162
  trees 511,
  94,
  175;
  scratch 5 trees 2,
  2, 0.
- Unit-level duplication across trees,
  by directory name only (profile,
  package,
  hash) over the 3516 trees that have
  per-unit directories:
  1,741,401 unit directories but 34,957 distinct ones;
  19,282 distinct units occur in more than
  one tree and 4528 occur in 100 or more.
  Most of that is the clone-shape trees plus registry crates compiled with identical settings in several workspaces.
  Sharing them is not possible with one directory per workspace path,
  and the shared-directory alternative is rejected
  in the rejected list at the end of the options.

### Worktrees

`git worktree list` registers 335 worktrees besides the main checkout:
241 under `.claude/worktrees`,
77 under
`~/temp/agent` (a symlink into `/var/mnt/encrypted`),
16 under `~/worktrees`,
and one elsewhere.
Searching each (depth 5,
`/usr/bin/find`) found 23,914 build directories:
`target` 3065 GiB,
`build` 80 GiB,
`dist` 55
GiB,
`out` 12 GiB and `corpus` 6.5 GiB,
3218 GiB by `du` in all,
in 325 worktrees.
96 worktrees hold more than 1 GiB of build directories and 87 more than 5 GiB;
every `package/ide/app/target` copy is
21.6 to 21.8 GiB (the part of `target` that the copy rule keeps).

- On the home filesystem (the 241 under `.claude/worktrees` and the 16 under `~/worktrees`) the copy is a snapshot or a
  reflink.
  The sampled `.claude/worktrees/fast-snapshot-add/package/ide/app/target` is 21.60 GiB total,
  0.00 B exclusive and
  20.43 GiB set-shared,
  so it costs almost no physical bytes while the main checkout's copy exists.
- On the other filesystem (the 67 `~/temp/agent` worktrees that hold build directories hold 664 of them,
  105 GiB by
  `du`) the copy is a byte copy.
  The sampled `/var/mnt/encrypted/temp/agent/branch-write-round5/package/ide/app/target` is 21.47 GiB total and
  21.47 GiB exclusive,
  0 shared.
  This is the cost in the brief,
  and it scales with the size of the main checkout's `target/` directories at the moment
  of the add.

### Gradle, Android, `dist/`, fuzz and `node_modules`

- Gradle cleans its own project cache and user-home caches:
  project `.gradle/<version>` directories are deleted after
  7 days unused,
  user-home caches after 7 or 30 days ([Gradle directory layout][gradle-cleanup]).
  `~/.gradle` is 3.2 GiB.
  `build/` has no automatic cleanup (Gradle's own text:
  "the latter are transient files Gradle uses to support features
  like incremental builds");
  `package/music-player/android-app/mise.toml` already has a `clean` task
  (`./gradlew clean`).
  All Gradle `build/` directories together are 0.9 GiB,
  so no option targets them.
- `dist/` outputs total 0.6 GiB;
  218 directories,
  one at 0.44 GiB.
  The `session-start-housekeeping` plugin removes only stray Git metadata inside `dist/final` (its README).
- Fuzz `corpus` is input data (0.08 GiB),
  `artifacts` is 56 KiB.
- Root `node_modules`:
  `.monochromatic` already has retention inside its producer
  (`doc/decision/module-logger-log-accumulation.md` lines 5 to 12:
  "retention window 14 days and 512 MiB and 50,000 files"),
  and the pnpm store and `prefer-readonly-parameter-type` cache are rebuilt by their tools.

### Outside the checkout

`podman system df` reports 3462 images of 391.6 GB (389 GB reclaimable) and 33 volumes of 87.6 GB (86.6 GB
reclaimable);
198 images are `localhost/git-policy-native-test` and 38 `localhost/git-policy-native-mutation`.
Storage root `~/.local/share/containers/storage` (overlay).
These are build output of the container test tasks but not a directory of the monorepo;
whether they are in scope is
an open question.

### What the Git wrapper's copy already leaves out

`package/git-policy/cli/src/native/worktree_copying/worktree_copy_prune.rs` skips `target/**/incremental`,
`target/**/zig-global-cache`,
`target/verification`,
`.cache`,
`.pnpm-store`,
`.gradle`,
`.kotlin`,
`.zig-cache`,
`node_modules/.monochromatic`,
`mutants.out`,
`playwright-report` and `test-results` for a main-checkout source.
Under that rule the target directories that remain to be copied,
by apparent size,
are 20.8 GiB for
`package/ide/app/target`,
12.1 GiB for `package/nested-wayland-session/cli/target`,
1.2 GiB for
`package/git-policy/cli.fuzz/target`,
0.5 GiB for `package/linter/monochromatic-lint/target`,
0.3 GiB for `package/git-policy/cli/target` and 0.1 GiB for `package/terminal/app/target`:
35.0 GiB.
That does not reconcile with the 49.5 GB per add in the brief,
which is within a percent of the brief's listed total
minus incremental state (70.5 GB less 21.3 GB is 49.2 GB),
as if a build that skips incremental state but not
`target/verification` or the `node_modules` caches had run.
Which wrapper build ran was not verified.
Of the 35.0 GiB,
the non-incremental units last built by an older compiler are 12.7 GiB of
`package/ide/app/target` and 4.8 GiB of `package/nested-wayland-session/cli/target`.
If the wrapper binary in use predates the rule,
the incremental state (19.8 GiB in these two directories) is copied as well.

## What always lean can mean

Contract:
at any quiescent moment (no cargo holds the directory's build lock),
after the next trigger has run,
a
directory is lean when all hold:

- Currency:
  it contains no unit or incremental state last built by a compiler older than the newest one built in its
  profile directory,
  once a grace period the owner picks has passed;
  no tree whose workspace is gone and which has been
  idle longer than the grace period;
  and no run record older than the owner's retention.
- Size:
  it is no larger than a bound taken from one current build of its own configuration.
- Busy,
  unclassifiable (unknown layout) and within-grace directories are reported as such,
  never counted as lean or as
  violations.
  During a running build a directory may exceed its bound;
  the contract is eventual,
  not an invariant.
  A dormant directory (no build since the last compiler change) has a single compiler group,
  so only the idle rule and the
  size bound reach it.

Proposed bounds,
each backed by a measurement in the section Measurements and each needing the owner's decision before
it is shipped (a shipped limit has to cite a measurement or an owner decision beside the value):

- A package `target/` directory that keeps intermediates in the tree (container builds):
  at most one and a half times
  the bytes of its newest-compiler units and incremental state,
  which measure 7.8 GiB of units and 1.9 GiB of
  incremental state for `package/ide/app/target` (about 10 GiB,
  bound about 15 GiB) and 7.1 GiB and 0.8 GiB for
  `package/nested-wayland-session/cli/target` (about 8 GiB,
  bound about 12 GiB),
  against 38.3 and 13.4 GiB now.
- A package `target/` directory that holds final artifacts only (host builds):
  at most 2 GiB;
  every one except the
  verification run records measures under 1.2 GiB now.
- `target/verification`:
  bounded by a retention the owner picks;
  today 6.2 GiB,
  oldest 2026-10-04.
- Host Cargo build trees:
  none idle past the grace period whose workspace is gone;
  each at most one and a half times its
  newest-compiler bytes (a whole tree,
  units and incremental state of both profiles:
  17.8 GiB for `0b/ab24900891e355`,
  about 13 GiB for each of the clones `75/`,
  `76/`,
  `77/`);
  the base directory at most the live workspaces times that.
- Gradle `build/`,
  `dist/`,
  root `node_modules` caches:
  no new bound;
  each is under 1.2 GiB and owned by a tool that
  retains,
  rebuilds or cleans it.
- Fuzz `corpus` is input data and `artifacts` are findings;
  neither is ever an eviction target.

Acceptance check for each option:
a read-only walk that classifies every profile directory as lean,
within grace,
violating,
busy or unknown layout.

## Owner decision and what was built

The owner's answers of 2026-11-07 (question tool,
recorded at 2026-11-07T22:19:43-05:00 in
`doc/handover/questions-for-the-owner.md`):

- "A to F,
  B allowed (Recommended)":
  all six mechanisms in order,
  in cargo-shadow (`package/cargo-shadow/cli`),
  with `doc/decision/cargo-build-dir-host-cache.md` amended so the sweep (B) may read Cargo's fingerprint files,
  for that sweep only;
  the tools' first run,
  under their own locks,
  cleans what exists today;
  grace periods and size caps are measured and recorded as coordinator picks with the owner's veto open.
- "Rust toolchains (Recommended)" and "Podman images,
  volumes (Recommended)":
  remove nightlies that no checkout's lock pins,
  and images and volumes no container uses and no current image tag of this repository names,
  each after the grace period.

Where each mechanism lives (all in `package/cargo-shadow/cli/src`,
described in its README,
section "Keeping the build folders lean"):

- A,
  toolchain-keyed build folders:
  `lean_keyed_folder.rs`,
  called from `shim.rs` for the child cargo only.
  Done for host runs.
  Not done for container runs:
  `package/ide/app/mise.toml` and `package/nested-wayland-session/cli/mise.toml` are outside cargo-shadow and the owner's
  answer did not name them,
  so those two in-tree builds are kept lean by B and E.
- B,
  the older-compiler sweep:
  `lean_unit_reader.rs`,
  `lean_sweep_plan.rs`,
  `lean_unit_removal.rs`.
- C,
  verified-idle trees:
  `lean_build_trees.rs`,
  `lean_idle_trees.rs`.
- D,
  run-record retention:
  `lean_run_records.rs`.
  The plan assigned D to the producing packages;
  the owner's answer put all six in cargo-shadow,
  so the retention runs from `cargo-shadow lean run`.
  The producing tasks (five packages with TypeScript runners) do not call it themselves.
  Wiring each producer to call it first is a separate change and is open.
- E,
  size caps:
  `lean_size_caps.rs`.
  The caps are absolute per profile directory (see the `Measured picks, 2026-11-07` section),
  because a cap relative to a directory's own newest-compiler bytes can never bind.
- F,
  the service that keeps the cleanups running:
  `lean_service_units.rs`,
  installed by `cargo-shadow lean service-install`
  (the name `timer-install` still works and says what it does now).
  It was built as a daily timer on 2026-11-07 and became a continuous service on 2026-11-11.
- Also in the same run:
  `lean_rustup_toolchains.rs` (dated nightlies no checkout pins),
  `lean_podman_cleanup.rs`,
  `lean_podman_real.rs` and `lean_container_references.rs` (images and volumes no container uses and no current source names).

The proof of "not in use" is the same for all of them
(`lean_lock_probe.rs`,
`lean_proc_scan.rs`,
`lean_guard.rs`,
`lean_removal.rs`):
a profile directory's Cargo locks taken exclusively without waiting,
no waiter queued in `/proc/locks`,
a scan of `/proc` finding no use under the entry,
and the entry's device and inode checked when judged,
after the claiming rename and just before the unlink.
Each removal's log line carries the proof.
The claim is the rename:
`cargo-shadow lean` takes no lock of its own,
and it never starts `cargo` or itself (`lean_no_recursion_tests.rs` fails if a lean source does).
The lock held through a container's bind mount was checked again on 2026-11-07:
a `flock` taken inside a Fedora container on a file of a bind-mounted directory made the run skip the profile directory
("busy"),
and the host's `flock --nonblock` failed while the container held it.

How the service runs (replaced the daily timer on 2026-11-11):
each pass of `lean run` ends on its own deadline,
the unit kills a pass that outlasts `RuntimeMaxSec`,
and the next pass starts 60 s after the last one ended,
for as long as the user's manager runs.
The owner,
2026-11-11:
"Make the cleanups a systemd service that runs continuously."
The daily timer it replaced fired once a day at 04:30 and never made up a missed run
(`Persistent=false`,
no `[Install]` section on the service,
the timer's `WantedBy=timers.target` only making the timer itself available when the user's systemd starts).
That schedule lost a day on 2026-11-11:
the build volume held 6.5 MB free at 13:13,
hours before the daily pass,
and a manual pass hit its own deadline before it reached every profile directory.
So `cargo-shadow lean service-install` now writes one unit,
stops and disables `cargo-shadow-lean.timer`,
removes its file and writes no timer again.
The service has `Restart=always`,
`RestartSec=60s` and `StartLimitIntervalSec=0`,
so a pass that finds nothing never stops the loop,
and it is installed for `default.target`,
so the user's manager starts it at login.
Nothing in cargo-shadow starts itself:
the unit runs only after a person installed and enabled it.
The service runs `<main checkout>/.cache/toolchain-bin/cargo-shadow lean run` with disk weight 5 (never above 10),
two threads of CPU,
4 GiB of memory,
512 tasks,
and shows nothing on the desktop.
The pause between passes is sized from a measurement of a pass that finds nothing
(`PAUSE_BETWEEN_PASSES_SECONDS` in `src/lean_policy.rs`):
measured 2026-11-11 with the release binary,
a pass over a world with nothing at all to clean cost 0.052 s at best and 0.054 s at the median of five runs,
and the same pass over 500 in-tree profile directories it kept cost 0.177 s,
so a minute keeps the loop at work under a thousandth of the time there,
where systemd's default 100 ms restart delay would put it at work about a third of the time.
On this machine an idle pass costs at least 27 minutes
(398 s for the run records on 2026-11-08,
334 s for Podman and 880 s to list 68,755 profile directories on 2026-11-10),
so the pause is a twenty-seventh of the shortest one.

## Measured picks, 2026-11-07

Coordinator picks,
each beside its measurement,
with the owner's veto open
(`package/cargo-shadow/cli/src/lean_policy.rs` holds the same numbers and the same reasoning):

- Grace period (units,
  toolchains,
  images,
  volumes) and the idle period of a tree whose workspace is gone or whose toolchain is retired:
  seven days.
  Measured:
  342 live worktrees were idle (newest write among their top-level entries) a median of 5.4 days,
  6.9 days at the 75th percentile and 20.3 days at the 90th;
  dated nightlies were installed at most seven days apart
  (2026-10-21 to 2026-10-28),
  and the newest one is under one day old.
  A worktree or second toolchain left alone for a week is still kept.
- Idle period of a tree nothing identifies:
  twenty-one days (the 90th percentile of worktree idleness).
- Run-record retention:
  fourteen days and 2 GiB per verification directory,
  with a one-day protection window.
  Measured:
  372 records in six packages,
  the newest 6.9 days old and the oldest 34.5 days;
  the fuzz records average about 95 MiB.
  Fourteen days is the age of the accepted logger decision.
- Size cap of one in-tree profile directory:
  12 GiB.
  Measured newest-compiler groups (units of the newest compiler plus the incremental state written since it first ran):
  `package/nested-wayland-session/cli/target/debug` 7.35 GiB,
  `package/ide/app/target/debug` 4.23 GiB,
  `package/ide/app/target/release` 3.98 GiB,
  `package/ide/app/target/dylint/debug` 2.09 GiB.
  One and a half times the largest is 11.0 GiB.
- Size cap of one host-tree profile directory:
  24 GiB.
  Measured:
  `0b/ab24900891e355/debug` 15.83 GiB,
  `75/3315c269a08379/debug` 8.44 GiB,
  `74/8a90f0615defdd/debug` 4.82 GiB.
  One and a half times the largest is 23.7 GiB.
- These are measurements of the current builds in place (no new build was run:
  a build of the two container packages is hours of load,
  and the owner's rule bans synthetic load).
  The 2026-11-04 round measured the same configurations at 7.8 and 1.9 GiB for `package/ide/app` and 7.1 and 0.8 GiB for
  `package/nested-wayland-session/cli`.

## Mechanisms researched

Cargo (version in use:
1.101.0-nightly,
2026-10-27;
there are no `rust-toolchain` files in the repository,
the pin is
`RUSTUP_TOOLCHAIN` from `.cache/toolchain.env.json`):

- `build.build-dir` and `{workspace-path-hash}`:
  "The directory where intermediate build artifacts will be stored"
  ([config reference][cargo-config-build-dir]);
  the layout inside is "internal to Cargo,
  and is subject to change"
  ([build cache][build-cache]).
- `cache.auto-clean-frequency` covers only the global cache in the Cargo home;
  the reference says support for tracking
  build artifacts "is not yet implemented" ([config reference][cargo-config-cache]).
- `cargo clean`:
  with no options it deletes the entire target directory;
  `--dry-run` is a read-only preview;
  `-p <package>` removes every hash of that package (current and stale alike:
  `src/ops/cargo_clean.rs` matches
  `starts_with("<crate>-")` in `incremental` and by package name in fingerprints and `build`),
  so it is "rebuild this
  package" and not "drop only stale" ([cargo-clean(1)][cargo-clean]).
  `cargo clean -p` takes the exclusive build-directory lock (`Layout::new(..., true, true)`,
  `cargo_clean.rs`
  line 201),
  so it waits for a running build.
  A full `cargo clean` does not:
  "we don't bother grabbing a lock here" (`cargo_clean.rs` lines 129 to 131),
  so it can
  delete under a running build.
- `-Zgc` and `cargo clean gc` exist on this cargo (`cargo clean gc --help`) but govern only the global cache
  (source,
  crate,
  index and git caches;
  [unstable reference][cargo-gc]).
- Locks:
  a build holds `<build-dir>/<profile>/.cargo-build-lock` exclusively unless `-Zfine-grain-locking`
  ([`layout.rs` lines 264 to 285][cargo-layout]),
  implemented with `flock`
  ([`flock.rs` lines 161 to 174][cargo-flock]).
  A sweeper that takes the same lock waits for or excludes cargo builds in that profile directory,
  not in others.
- Upstream direction:
  [rust-lang/cargo#13136][cargo-13136] (garbage collect whole `target/`,
  open) and
  [rust-lang/cargo#5026][cargo-5026];
  a maintainer's plan is to hash the Cargo version into `{workspace-path-hash}` so
  each version gets its own build directory that can be tracked and cleaned
  ([comment of 2025-04-14][cargo-5026-plan]).
  [rust-lang/cargo#6229][cargo-6229] (clean up within `target`) was closed as a duplicate of #5026 on 2026-08-13.
  No released Cargo feature removes stale intermediates.
- Nightly versus stable:
  the metadata hash keeps the channel name only for a nightly,
  so a nightly update changes no
  directory name and dirties units in place (see the section on older-compiler bytes);
  a stable update does change names,
  and stable toolchains here (1.98.1,
  1.99.0) would leave whole generations.
- `-Zbuild-dir-new-layout` and the 1.100.0 layout change move toward one directory per unit,
  which makes
  per-unit removal simpler later;
  today both layouts are in the same directories.

Third-party:

- [cargo-sweep][cargo-sweep] (age,
  installed toolchains,
  size,
  stamp-and-sweep):
  its README opens with "currently
  unmaintained",
  and an open issue asks for `build-dir` support ([holmgr/cargo-sweep#140][sweep-140]);
  it reads
  `target/`,
  which holds nothing but final artifacts for host builds.
- [cargo-clean-all][cargo-clean-all]:
  finds and removes whole `target` directories by age;
  one open issue says
  `CARGO_TARGET_DIR` is not considered.
- [sccache][sccache-rust]:
  requires incremental compilation off ("`rustc`'s incremental compilation needs to be
  disabled"),
  and `bin`,
  `cdylib` and `proc-macro` crates cannot be cached;
  it would also not shrink on-disk output.

Profile settings (`debug`,
`split-debuginfo`,
`strip`,
`codegen-units`,
`incremental`):
excluded by the owner on
2026-11-04;
not researched or proposed.

Non-cargo build systems found:
Gradle (retains its own caches;
project `build/` is removed by `gradlew clean`),
tsdown `dist/` (rebuilt per task;
`package/ssg/aquati.cat` and the Electron packages already have `build:clean`),
Zig (`zig-global-cache`,
skipped by the wrapper copy).

## What the repository already owns

Records that bear on this,
quoted:

- `doc/decision/cargo-build-dir-host-cache.md` line 17:
  "Accepted 2026-09-19.
  The user set it directly in
  `~/.cargo/config.toml` on this machine and asked for it to be recorded here."
  Lines 292 to 297:
  worktree builds "outlive their worktree";
  "Worktrees created and discarded by cli-git accumulate
  this way."
  Lines 299 to 315:
  "Nothing prunes `~/.cargo/build/` automatically";
  reclaim by `cargo clean` or by deleting
  the hash directories of workspace paths no longer built.
  Lines 317 to 342:
  container builds "keep writing full intermediates into the repository tree.
  Expect a mixed state".
  Its section on the layout says "Do not write repository tooling that parses these paths."
  Option B has to read fingerprint files,
  which conflicts with that line;
  the owner decides whether the
  conflict is accepted (a reader that does nothing when it cannot parse is the mitigation proposed).
- `doc/troubleshooting/cargo-build-dir-workspace-path-hash-unbounded.md`:
  438 GiB from 284 trees in three weeks;
  lists "Periodic removal of trees not modified within N days" with the tradeoff that "mtime reflects the last build,
  so a workspace that is used but rarely built loses its cache".
- `package/git-policy/cli/src/native/repair/package_repair_cargo.rs` line 8:
  the owner chose on 2026-11-03 "Reuse deps,
  rebuild own",
  done with "Run cargo clean -p";
  the wrapper clones the source workspace's build directory for each
  new worktree,
  which is why trees repeat (three trees of 15.9 GiB have identical contents).
- `doc/decision/module-logger-log-accumulation.md` lines 5 to 12,
  accepted by the owner on 2026-10-22:
  retention
  inside the producer,
  swept at open,
  with an age,
  size and file-count cap.
  This is the precedent for a component owning the retention of what it writes.
- `doc/decision/ci-caching.md` lines 185 to 187:
  "Cargo never prunes artifacts of an older compiler,
  so restoring a
  stale `target/` only enlarges the next save."
  `doc/decision/music-player-windows-linkers.md` lines 135 to 147:
  the Windows jobs prune the package's own artifacts
  before saving.
- `doc/decision/dist-in-git.md` line 163:
  `session-start-housekeeping` "already cleans stale dist artifacts on session
  start";
  per its README it removes only stray Git metadata and fires only for Claude Code sessions.
- `package/cargo-shadow/cli/README.md`:
  every repository cargo run is the shadow
  (".cache/cargo-shadow-bin/cargo");
  it already runs a step before cargo and has subcommands (`install`,
  `update`).
  It does not cover cargo inside containers.
- `toolchain:update` (`mise.toml` line 535) is where a new compiler arrives.
  No existing task or tool cleans Cargo output;
  the only `clean` tasks are Gradle,
  Electron `build:clean` and the
  site `rm -rf dist`.

## Options

Each option says what it frees here,
what it costs,
how it stays automatic,
how it behaves while another session builds
in the same directory,
and which component owns it.
Cargo's own locking is the only protocol that cooperating builds honor.
Any option that deletes inside a Cargo output directory follows the same shared protocol:

- Take the exclusive `.cargo-build-lock` of a profile directory with a non-blocking attempt before classifying,
  classify
  and revalidate under the lock,
  skip and report the directory if the attempt fails or the filesystem gives no lock
  (Cargo itself skips locking on NFS),
  never rename or unlink a lock file,
  and release the lock before deleting.
- Move what is to go into a trash directory on the same filesystem by atomic rename while holding the lock,
  then delete
  the trash.
  In the old layout one unit is several paths (`deps/`,
  `.fingerprint/`,
  `build/`,
  `incremental/`);
  all are renamed
  under
  the one lock,
  so no cooperating cargo sees a half unit,
  but a non-cargo reader of those paths can.
  Final artifacts in `target/<profile>/` are not touched.
- Readers that do not take the lock (a test runner executing a binary from `deps/`,
  nextest) are protected only by the
  grace period and by sweeping older-compiler units,
  which a run under the newest compiler does not use.
- For container builds the lock is the same file through the bind mount:
  a probe on 2026-11-04 held `flock` on a file in
  a bind-mounted throwaway directory inside a podman container,
  and a host `flock --nonblock` on the same path reported
  busy (exit 1) while the container held it and succeeded (exit 0) after it ended.
  The container's Cargo version and lock protocol were not checked.
- A full `cargo clean` takes no lock and is not used by any option.

### A: toolchain-keyed build directories

- What:
  the owner component sets `CARGO_BUILD_BUILD_DIR` to `<base>/<toolchain>/<workspace key>` for Cargo runs it
controls (host runs through the shadow;
  container runs through each package's runner,
  as
  `<package>/target/build/<toolchain>`
  inside the tree),
  so retiring a toolchain is removing a directory.
  This is the shape a Cargo maintainer describes for upstream ([cargo#5026 comment][cargo-5026-plan]).
- Frees here:
  because a nightly update dirties every unit anyway,
  the first build under a new toolchain loses nothing by
  starting in a fresh directory;
  the retired directory goes whole.
  It would have removed the 34.0 GiB of older-compiler bytes in the two container-built directories as toolchains
  retired,
  and the older groups of the host trees,
  without reading any Cargo file.
  It does not remove superseded units inside the newest compiler's directory (a dependency or feature change).
- Costs:
  one cold rebuild per workspace when the location changes,
  which each nightly update already causes;
  runs that bypass the shadow (an IDE launching Cargo directly) keep using `~/.cargo/config.toml` and are not covered;
  the environment variable is used verbatim ("No template expansion is applied to a literal path given this way",
  `doc/decision/cargo-build-dir-host-cache.md`),
  so the owner component computes its own workspace key,
  which also
  gives it the workspace-to-tree map the other options need.
  In-tree container directories stay in the copied tree,
  so the copy still carries the newest toolchain's bytes
  (about 10 and 8 GiB).
- Automatic:
  on the first run after the toolchain changes,
  and after `toolchain:update`.
- Concurrency:
  a retired directory is removed only after the grace period,
  with the shared protocol of this section
  applied to its
  profile directories;
  a build that started under that toolchain meanwhile fails loudly on its renamed directory,
  never
  corrupts another toolchain's.
- Owner (proposed):
  `package/cargo-shadow/cli` for host runs;
  each container runner (`package/ide/app`,
  `package/nested-wayland-session/cli`) for container runs.
  Setting an environment variable for the child cargo is new behavior for the shadow,
  whose README says it edits no
  argument of its own accord;
  the owner decides.

### B: sweep units built by an older compiler out of existing directories

- What:
  a repository tool reads each profile directory's fingerprints,
  keeps the newest compiler group (plus any group
  built within the grace period),
  and removes the other groups' units and the incremental directories older than the
  kept group.
- Frees here:
  34.0 GiB of the 51.7 GiB in the two container-built targets (17.5 GiB of it in the 35.0 GiB copy set,
  and
  all of the incremental part if the wrapper in use copies incremental state);
  98.9 GiB of 179.1 GiB in the 8 largest
  host trees.
  Works on directories A does not key (existing ones,
  runs outside the shadow).
- Costs:
  it parses Cargo's internal fingerprint files,
  which two layouts and a layout change in progress make a standing maintenance cost,
  against the line in `doc/decision/cargo-build-dir-host-cache.md` that says not to;
  a tree it
  cannot parse is left alone and reported.
  Removal is a cache eviction:
  an alternating second toolchain loses its units and rebuilds them.
- Automatic:
  after `toolchain:update`,
  at the first cargo run after a compiler change,
  and after a container task ends.
- Concurrency:
  the shared protocol.
- Owner (proposed):
  `package/cargo-shadow/cli` as a subcommand beside `install` and `update`.

### C: collect verified-idle trees in the host build directory

- What:
  the owner component keeps a ledger of (workspace root,
  build directory) for every run it controls and,
  at most
once a day,
  removes a tree only when all hold:
  its workspace root is gone,
  the tree has been idle longer than the
  grace
  period (no write in any profile directory),
  every profile directory yields its lock,
  and no other tree names
  sources inside it (remove the building tree first,
  then the one holding sources).
  A workspace root being gone alone is not enough:
  a process can outlive its directory.
  Removal renames the tree root into trash first.
  For existing trees the ledger is backfilled from the first dep-info source path,
  which is a lead and not ownership,
  so backfilled trees go only under the idle rule and the owner's explicit go.
- Frees here:
  trees attributed to a gone workspace:
  226 (367 GiB by `du`:
  174 in the main checkout,
  34 in
  `.claude/worktrees`,
  18 in scratch),
  plus package-shape and clone-shape trees once nothing names them;
  the 715 trees
  with no identifiable workspace (2334 GiB by `du`) fall only under the idle rule.
  Deleting a tree frees only extents nothing else references:
  in the first 770 trees sized one by one,
  exclusive bytes
  were 247 of 1251 GiB (19 to 29 percent by class;
  raw outputs are kept in
  `~/temp/agent/lean-build-review/measurements.md`,
  not committed).
- Costs:
  a ledger;
  idle removal costs a rebuild for a workspace used but rarely built (the cost
  `doc/troubleshooting/cargo-build-dir-workspace-path-hash-unbounded.md` names).
- Automatic:
  daily rate limit,
  from the same trigger as A.
- Concurrency:
  the shared protocol plus the idle test and the removal order.
- Owner (proposed):
  the same component.
  The component that creates the clone trees (the Git wrapper) is not proposed as owner;
  the owner excluded it.

### D: retention of run records at their producers

- What:
  `target/verification` run records get an age and size retention swept by the task that writes them,
  following
  the accepted logger decision (`doc/decision/module-logger-log-accumulation.md`:
  "retention inside the logger",
  14 days,
  512 MiB,
  50,000 files),
  with the owner's numbers.
- Frees here:
  up to 6.2 GiB now (747 retained run directories in five packages;
  the oldest from 2026-10-04);
  these are
  already left out of the worktree copy.
- Costs:
  records are evidence for fuzz and mutation runs;
  the owner decides how long they must live.
- Automatic:
  at the start of each producing task.
- Concurrency:
  each run writes its own uniquely named directory;
  the sweep removes only directories older than the
  retention and not the one in use.
- Owner:
  the packages that write them (`package/git-policy/cli`,
  `package/git-policy/cli.fuzz`,
  `package/linter/monochromatic-lint`,
  `package/linter/monochromatic-lint.fuzz`,
  `package/forbidden-strings/cli`).

### E: per-directory size caps with oldest-first eviction

- What:
  a cap per directory (section "What always lean can mean") evicts the oldest units by `invoked.timestamp` until
  the directory is under it,
  the same shape as the logger's age and size caps.
- Frees here:
  nothing that A to C leave if the cap is not under the newest compiler's bytes;
  its job is to make the
  bound
  true when one compiler's units alone outgrow it.
- Costs:
  a cap set too low evicts live units that the next build recompiles;
  it needs a number from the owner.
- Automatic and concurrent:
  the shared protocol;
  checked after ordinary cargo runs,
  so it covers repeated builds with one
  compiler,
  which the toolchain triggers do not.
- Owner:
  the same component as B.

### F: a timer for dormant directories

- What:
  a `systemd --user` timer runs the tool of B to E daily,
  so grace expiry,
  dormant worktrees and busy-skipped
  directories are retried without a cargo run.
- Frees:
  what B to E free;
  closes the "eventually" gap of the contract.
- Costs:
  a new unit to install,
  which the repository's toolchain install does not do today.
- Concurrency:
  the shared protocol,
  as B.
- Owner:
  a new component that only schedules;
  never alone.

Replaced 2026-11-11.
The owner:
"Make the cleanups a systemd service that runs continuously."
The scheduler stays a systemd unit in cargo-shadow,
but it no longer waits for a time of day:
each pass ends on its own deadline and the next starts a minute after it,
so a volume that fills within hours does not wait for 04:30.
What was built is in the section "Owner decision and what was built";
the pause between passes and its measurement are in `PAUSE_BETWEEN_PASSES_SECONDS`
(`package/cargo-shadow/cli/src/lean_policy.rs`).

### G: split container builds' intermediates away from the tree

- What:
  the container runners pass `CARGO_BUILD_BUILD_DIR` into a named volume,
  as they already keep the registry in
  one (`ide-cargo`,
  `nested-wayland-session-cargo`).
- Frees here:
  the copy set shrinks from 35.0 GiB to the final artifacts.
- Costs:
  it reverses the 2026-11-02 decision "Builds must be copied" for these packages (a new worktree's first
  container
  build starts cold),
  so it needs the owner to revisit it;
  inside the container every worktree is `/work`,
  so one volume
  is shared by all worktrees unless the key includes a host-derived workspace key,
  which then needs a volume lifecycle.
- Owner:
  each container runner;
  many files.

### H: track upstream garbage collection

- What:
  adopt the Cargo feature when [#13136][cargo-13136] and [#5026][cargo-5026] ship;
  keep the pointer in
  `doc/troubleshooting`.
- Frees:
  nothing now;
  about 20 GiB per day was measured before the relocation.
- Owner:
  Cargo.

### I: manual offline cleanup (status quo)

- What:
  `cargo clean` per package by hand,
  as `doc/decision/cargo-build-dir-host-cache.md` lines 309 to 315 say.
  A full `cargo clean` takes no lock and also removes the external host build directory of that workspace,
  so it can
  delete under a running build.
- Not to be confused with a one-time coordinated cleanup of what exists now,
  which is a rollout step for A to C,
  run
  by the tool under its locks after the owner's explicit go.

Rejected without ranking:
- A shared `build.build-dir` without the workspace hash:
  removes the growth but makes every session contend for one lock
  and lets worktrees of one package overwrite each other's units
  (`doc/troubleshooting/cargo-build-dir-workspace-path-hash-unbounded.md` lists it as bounded,
  not as safe).
- sccache:
  see Mechanisms.
- Btrfs deduplication:
  it lowers physical use but not the bytes a cross-filesystem copy reads,
  the motivating cost.
- Any profile setting:
  excluded by the owner.

## Ranking

Ranking:
A > B > C > D > E > F > G > H > I.
A to E complement each other and can be adopted in that order;
F schedules them;
G is optional and conflicts with a
standing owner decision.

- A over B:
  A needs no read of Cargo's internal files and retires whole directories,
  costs nothing extra at a toolchain
  change (every unit is dirty then anyway),
  and matches the shape a Cargo maintainer describes;
  B parses a layout the
  decision record says not to parse and which is in transition,
  but covers directories A does not key.
- B over C:
  B reaches the in-tree directories the motivating copy reads and decides from the compiler recorded in each
  unit;
  C removes whole trees on ownership inferred from one file and so needs the strictest protocol.
- C over D:
  C addresses the largest tree population (the host base,
  6166 GiB by `du`),
  where D addresses 6.2 GiB.
- D over E:
  D frees bytes that exist now at low risk,
  following the accepted logger decision;
  E frees nothing that A to
  C leave and needs a number the owner has not chosen.
- E over F:
  E enforces the bound during repeated builds with one compiler;
  F only schedules and changes nothing alone.
- F over G:
  F changes no owner decision;
  G reverses "Builds must be copied" and adds a volume lifecycle.
- G over H:
  G frees copy-set bytes now at a price the owner may accept;
  H frees nothing until Cargo ships a collector.
- H over I:
  H adopts a maintained fix without repository work;
  I relies on memory and a full `cargo clean` can delete
  under a running build.

## Open questions for the owner

Answered 2026-11-07 (question tool);
each item names its answer after the question,
and the section "Measured picks" holds the numbers.

- Owner of the sweep and of toolchain-keyed directories:
  `package/cargo-shadow/cli`;
  setting `CARGO_BUILD_BUILD_DIR` for the child cargo is accepted;
  reading Cargo's fingerprint files is allowed for the sweep only
  (amendment of `doc/decision/cargo-build-dir-host-cache.md`).
- Grace and idle periods:
  seven days,
  and twenty-one days for a tree nothing identifies (measured).
- Bounds and the run-record retention:
  per-profile caps and fourteen days with 2 GiB (measured).
- One-time cleanup:
  the tools' first run,
  under their own locks.
- Scope:
  dated nightlies no checkout's lock pins,
  and images and volumes no container uses and no current source names,
  each after the grace period.
- Container builds:
  keep intermediates in the tree;
  A is not applied to them,
  B and E keep them lean;
  G is not built.
- The two open items among the original questions were answered on 2026-11-08;
  their answers sit beside the questions,
  and section "Open" of `doc/handover/cargo-shadow-lean-build-folders.md` holds the measurements.

The original questions follow.


- Owner of the sweep and of toolchain-keyed directories:
  is `package/cargo-shadow/cli` the component?
  Is setting `CARGO_BUILD_BUILD_DIR` for the child cargo acceptable for the shadow?
  Is reading Cargo's fingerprint files (option B) acceptable against the line in
  `doc/decision/cargo-build-dir-host-cache.md`?
- Grace period (zero keeps one compiler group;
  a few days keeps the previous one while a second toolchain may still be
  in use),
  and the idle period for trees.
- The bounds in section "What always lean can mean",
  and a retention for `target/verification` run records.
- One-time cleanup of what exists now:
  it needs the owner's explicit go because other sessions build in these
  directories;
  the safe form is the tool's own run under its locks.
- Scope:
  whether `~/.rustup/toolchains` (20 directories,
  about 2.7 GiB each) and the podman images and volumes (391.6 GB
  and 87.6 GB as reported) count as build directories.
- Container builds:
  keep intermediates in the tree (A and B keep them lean) or split them away (G) against the
  2026-11-02 decision that builds are copied.
- What runs `cargo package` or `cargo publish --dry-run` about 90 times per crate,
  and whether it should stop at the
  source.
  Answered 2026-11-08:
  `cargo package` verifies by unpacking the crate into the workspace's build tree and compiling it
  (measured:
  `<base>/75/4a9477468c27be/package/tiny-probe-0.1.0` with `CARGO_BUILD_BUILD_DIR=<base>/{workspace-path-hash}`),
  and the invokers are 16 `run: cargo package` and 16 `run: cargo publish --dry-run` steps of
  `.github/workflows/cargo-publish.yml` (one of each per crate job),
  the release preflight of `package/crate-release/cli` (a `cargo publish --dry-run` that does verify),
  and each crate's manual `package`/`publish:dry-run` task.
  The "about 90 times per crate" figure is not reproducible as a run count:
  the hash-shaped host build trees hold 6,763 unpacked crate directories over 45 distinct versions,
  25 of them the wrapper's,
  and the copies multiply because each worktree's build tree is cloned.
  Whether to stop at the source is the owner's call;
  `--no-verify` on the two verifying steps would remove the only compile of the packaged crate.
- Which wrapper build is installed:
  the source skips `target/**/incremental`,
  `target/verification` and the caches,
  and
  the brief's 49.5 GB is within a percent of its listed total minus incremental state (49.2 GB).
  Answered 2026-11-08:
  the installed and locked build is `monochromatic-git-policy-cli` 0.6.1-alpha.4
  (`toolchain.lock.json`,
  route `archive`;
  the launcher record `.cache/toolchain-launchers/monochromatic-git-policy-cli.launch` names that payload),
  and it carries the whole rule,
  which entered at version 0.5.14
  (`5e773a62f4`,
  2026-11-02)
  and first shipped in `monochromatic-git-policy-cli-v0.5.18`.
  Which build ran for the brief's own measurement is not established;
  its 49.5 GB matches a build that left out incremental state only.

## Review

Cross-vendor review by `openai/gpt-6.1-sol` through `pi` on 2026-11-04 (inputs:
the first full draft of this document,
a measurement appendix and verbatim Cargo and repository source excerpts;
the raw review is kept in
`~/temp/agent/lean-build-review/review.out`).
Verdict on that draft:
needs changes.
No second review ran,
because the owner paused non-layout work.

Findings and what changed:

- Copy-set arithmetic (36.1 GiB and 21.8 GiB were wrong;
  35.0 and 20.8 are right) and the unit totals that exceeded
  their
  directory:
  fixed;
  units are now counted once per inode and the copy-set derivation is stated.
- Negative layout counts:
  an instrumentation bug,
  fixed and re-measured.
- The central causal claim was wrong for a nightly:
  Cargo does not hash the date or commit,
  so an update dirties units in
  place instead of creating new directory names.
  Rewritten around "built by an older compiler" (no reuse value under the newer one),
  with the source lines cited,
  and
  toolchain-keyed directories promoted to the first option because that matches what the source says.
- "Newest group is the only live one" and "orphaned":
  relabelled as eviction candidates,
  with a grace period and a
  dormant-directory caveat.
- `Exclusive 0` and clone claims:
  reinterpreted (another reference exists,
  location unknown unless Total equals Set
  shared);
  "identical contents" dropped to what was measured.
- Dep-info attribution:
  stated as leads,
  not ownership;
  no deletion on it alone.
- Concurrency:
  the single "workspace root is gone" shortcut was removed;
  a full protocol is stated (lock before
  classifying,
  revalidate,
  no lock-file renames,
  trash by rename,
  legacy multi-path removal,
  cross-tree sources,
  lock
  failure means skip,
  readers that take no lock).
  The lock through a container bind mount was probed.
- Coverage and timing:
  the contract is now eventual,
  with busy and unknown states and a timer option.
- Ranking:
  reworked after those corrections;
  producer retention for run records and a size-cap option were separated.
- The wrapper-side alternative was removed.
- Container split now says it reverses the 2026-11-02 decision that builds are copied.
- Not done:
  the container Cargo version and lock protocol were not checked;
  the whole-directory physical figure and a
  second review were not produced.

[btrfs-du]: https://btrfs.readthedocs.io/en/latest/btrfs-filesystem.html#:~:text=Each%20argument%20to%20btrfs%20filesystem%20du%20will%20have%20a%20set%20shared%20value%20calculated%20for%20it
[build-cache]: https://doc.rust-lang.org/cargo/reference/build-cache.html#:~:text=The%20build%2Ddir%20layout%20is%20considered%20internal%20to%20Cargo
[cargo-hash-rustc]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/compiler/build_runner/compilation_files.rs#L924-L945
[cargo-5026]: https://github.com/rust-lang/cargo/issues/5026
[cargo-5026-plan]: https://github.com/rust-lang/cargo/issues/5026#issuecomment-2803122104
[cargo-13136]: https://github.com/rust-lang/cargo/issues/13136
[cargo-6229]: https://github.com/rust-lang/cargo/issues/6229
[cargo-config-build-dir]: https://doc.rust-lang.org/cargo/reference/config.html#:~:text=The%20directory%20where%20intermediate%20build%20artifacts%20will%20be%20stored
[cargo-config-cache]: https://doc.rust-lang.org/cargo/reference/config.html#:~:text=Support%20for%20tracking%20build%20artifacts%20is%20not%20yet%20implemented
[cargo-clean]: https://doc.rust-lang.org/cargo/commands/cargo-clean.html#:~:text=With%20no%20options%2C%20cargo%20clean%20will%20delete%20the%20entire%20target%20directory
[cargo-gc]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#:~:text=The%20%2DZgc%20flag%20is%20used%20to%20enable%20certain%20features%20related%20to%20garbage%2Dcollection
[cargo-layout]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/compiler/layout.rs#L264-L285
[cargo-flock]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/util/flock.rs#L161-L174
[cargo-sweep]: https://github.com/holmgr/cargo-sweep#:~:text=currently%20unmaintained
[sweep-140]: https://github.com/holmgr/cargo-sweep/issues/140
[cargo-clean-all]: https://github.com/dnlmlr/cargo-clean-all
[sccache-rust]: https://github.com/mozilla/sccache/blob/main/docs/Rust.md#:~:text=incremental%20compilation%20needs%20to%20be%20disabled
[gradle-cleanup]: https://docs.gradle.org/current/userguide/directory_layout.html#:~:text=They%20are%20deleted%20if%20they%20haven%E2%80%99t%20been%20used%20for%207%20days
