# The closure lint in the linter, for every Rust package

Status:
proposal of 2026-10-19.
The owner chose Mechanism A on the same day (see the section "Answers of 2026-10-19 to the proposal" of
[the decision][decision]),
and it is implemented on the branch `feat/linter-closure-everywhere`:
`package/linter/monochromatic-lint/README.md`,
section "Closure check",
describes what shipped.
The text below is the proposal as it was written,
with the measurements it rests on;
the figures were taken before the implementation existed.
It answers the owner requirement recorded in the section "Owner requirement of 2026-10-19" of
[the decision][decision]:
the Dylint lint `avoidable_closure` of `package/linter/monochromatic-lint-closure`
runs by default on every Rust package,
as part of `monochromatic-lint`,
with no `lint:closures` task of its own.

## Requirement

- Every Rust package gets the check by default.
  The syntax-only rule `rust/no-anonymous-functions` that did this was removed on 2026-10-18,
  and since then only `package/desktop-app/ide` has a closure check (its `lint:closures` task).
- The check is a part of `monochromatic-lint` (the root task `lint:monochromatic-lint`),
  so a developer or CI runs one linter and reads one result channel.
- There is no separate `lint:closures` task in any package.
- The lint needs rustc type information (captures and the callee's `'static` bound),
  so `monochromatic-lint` must either run the Dylint library itself or carry the criterion in its own type analysis.
  [The proposal of 2026-10-18][proposal] rejected the second option.

## How the figures were measured

All figures come from this session on one host and one toolchain,
in the worktree
`.claude/worktrees/closure-everywhere` (branch `docs/closure-lint-everywhere`,
off `origin/main`).

- Host:
  28 logical cores and 61 GiB of memory;
  the shared nightly `rustc 1.101.0-nightly (8006091eb 2026-10-17)`
  (toolchain `nightly-2026-10-18`,
  with `rustc-dev`),
  `cargo-dylint 6.2.0`,
  and `dylint-link 6.2.0`.
- Every build wrote to a scratch directory under `~/temp/agent/closure-everywhere`
  through `CARGO_TARGET_DIR` and `CARGO_BUILD_BUILD_DIR`,
  so no package `target` directory was modified.
  Long runs were systemd user units with `MemoryMax=40G`.
- Host feasibility:
  `cargo check --all-targets --locked --offline` in each package directory with
  `RUSTUP_TOOLCHAIN=nightly-2026-10-18`,
  three packages at a time.
- Lint run,
  the way the IDE task runs it:
  `cargo dylint --lib-path <staged library> --manifest-path <package>/Cargo.toml -- --all-targets --keep-going --message-format json`
  with `DYLINT_RUSTFLAGS='-W avoidable_closure'` and `CARGO_NET_OFFLINE=true`,
  one package at a time with
  `CARGO_BUILD_JOBS=8`.
  The staged library is the copy named `libmonochromatic_lint_closure@nightly-2026-10-18-x86_64-unknown-linux-gnu.so`
  that `dylint-link` writes when `cargo build --locked` runs in the lint package.
- Cold time:
  the first run with an empty scratch target directory,
  so it includes compiling every dependency.
  Warm time:
  the immediate second run,
  which replays cargo's cached diagnostics.
  Each time is a single run;
  run-to-run spread was not measured,
  so differences of a second or less are noise.
- Counts are unique reports,
  taken from the compiler-message records and deduplicated on file,
  line and column,
  because one source line is reported once per target (library,
  tests,
  binaries) that compiles it.
  A report is "in tests" when its file sits under `tests/`,
  `benches/`,
  `examples/` or `fuzz_targets/`,
  or is named `*_test.rs`,
  `*_tests.rs` or `tests.rs`.
- Inputs prepared outside the run:
  the 6 packages that failed offline were fetched with
  `cargo +stable fetch --locked` in their directories,
  which fills the cargo registry and git caches only;
  the IDE's `target/debug/runtime` and `target/crate-licenses.json` were copied from the main checkout
  (made by a previous run of the IDE `runtime` and `notices` tasks),
  because those tasks need containers and network.
  The copy lives in scratch and was linked into the build directory the way the IDE task links it.

## Measured feasibility per package

The Rust packages are the 24 tracked `package/**/Cargo.toml` files (`git ls-files 'package/**/Cargo.toml'`),
which include 5 fuzz crates and 2 bench crates.
They are separate crates with their own `[workspace]` and lockfile,
they are linted by `monochromatic-lint` today,
and they compile,
so all of them are listed.
Findings are "own" reports in the package's files;
reports inside generated Slint code are listed separately.

For each package:
the host build,
what it needs,
cold and warm time of the lint run,
and the reports (source,
tests;
capture nothing,
capture but the callee does not keep it).

- `package/forbidden-strings/cli`:
  builds on the host with nothing extra.
  Cold 29.8 s,
  warm 0.6 s.
  155 reports (123 source,
  32 tests;
  127 capture nothing,
  28 capture).
- `package/forbidden-strings/cli.fuzz`:
  builds with nothing extra.
  Cold 27.9 s,
  warm 0.5 s.
  15 reports (1 source,
  14 fuzz targets;
  14 capture nothing,
  1 captures).
- `package/cli/nested-wayland-session`:
  builds offline for `cargo check`,
  but `cargo metadata` inside Dylint needs the crates of other platforms,
  so it failed offline until `cargo +stable fetch --locked` ran.
  Cold 16.9 s,
  warm 0.7 s.
  221 reports (167 source,
  54 tests;
  132 capture nothing,
  89 capture).
- `package/wg/cli-quicker-exempt`:
  builds with nothing extra.
  Cold 1.5 s,
  warm 0.4 s.
  58 reports (38 source,
  20 tests;
  50 capture nothing,
  8 capture).
- `package/codex-plugin/bash-limit`:
  builds with nothing extra.
  Cold 1.9 s,
  warm 0.4 s.
  21 reports (9 source,
  12 tests;
  21 capture nothing).
- `package/desktop-app/file-manager`:
  needs system GTK 4 (4.22.5 here,
  through `pkg-config`) and the offline fetch.
  Cold 24.4 s,
  warm 0.6 s.
  57 reports (55 source,
  2 tests;
  35 capture nothing,
  22 capture).
- `package/desktop-app/file-manager-gtk-sticky`:
  needs system GTK 4 and the offline fetch;
  it depends on `file-manager` by path,
  whose files Dylint does not lint here.
  Cold 23.8 s,
  warm 0.6 s.
  27 reports (25 source,
  2 tests;
  18 capture nothing,
  9 capture).
- `package/desktop-app/file-manager-qt`:
  needs the Qt 6 development packages with `qmake` (Qt 6.11.2 here),
  a C++ toolchain,
  and the offline fetch.
  Cold 50.8 s,
  warm 0.8 s.
  1 report (source;
  captures nothing).
- `package/desktop-app/ide`:
  `cargo check` fails on the host without the prepared runtime:
  `build.rs` panics at line 207 because `<build dir>/debug/build/runtime/manifest.json` cannot be read.
  With the runtime and the licenses file linked into the build directory as the IDE task does,
  cold 37.2 s,
  warm 1.0 s.
  0 reports,
  which matches the task's current clean state (5 fulfilled `expect` attributes;
  no other diagnostics).
- `package/desktop-app/terminal`:
  needs `zig` 0.15.x (the root mise tool),
  the `pkg-config` font libraries of Slint,
  the zig cache directory that the root `.cargo/config.toml` sets (`ZIG_GLOBAL_CACHE_DIR`),
  and the offline fetch.
  A copy outside the worktree failed with `zig build failed` until that variable was set.
  Cold 36.4 s,
  warm 0.9 s.
  134 reports,
  of which 118 are inside the generated `app.rs` (see "Generated Slint code");
  the 16 own reports are 12 source,
  4 tests;
  9 capture nothing,
  7 capture.
- `package/git-policy/cli`:
  builds with nothing extra.
  Cold 33.9 s,
  warm 0.6 s.
  767 reports (524 source,
  243 tests;
  538 capture nothing,
  229 capture).
- `package/git-policy/cli.fuzz`:
  builds with nothing extra.
  Cold 30.2 s,
  warm 0.6 s.
  17 reports (11 source,
  6 fuzz targets;
  7 capture nothing,
  10 capture).
- `package/linter/monochromatic-lint`:
  builds with nothing extra.
  Cold 18.1 s,
  warm 0.7 s.
  67 reports (30 source,
  37 tests;
  47 capture nothing,
  20 capture).
- `package/linter/monochromatic-lint-closure`:
  builds because the shared nightly has `rustc-dev`;
  the lint library lints itself.
  Cold 9.2 s,
  warm 0.7 s.
  0 reports.
- `package/linter/monochromatic-lint.fuzz`:
  builds with nothing extra.
  Cold 17.6 s,
  warm 0.7 s.
  0 reports.
- `package/music-player/android-app/rust`:
  `cargo check` fails on the host target (`ndk-sys only supports compiling for Android`),
  then fails for the Android target because the `opusic-sys` build script cannot find the NDK
  (`Android: Neither the NDK or a standalone toolchain was found`).
  It builds with the installed NDK (29.0.13846066 under `ANDROID_HOME`) through `cargo ndk --target arm64-v8a check`.
  The Dylint run works with the variables from `cargo ndk-env --target arm64-v8a`
  and `-- --all-targets --target aarch64-linux-android`;
  the wrapper `cargo ndk ... dylint` fails because it appends
  `--target` before the subcommand's own arguments,
  which `cargo dylint` rejects.
  Cold 21.4 s (warm not run).
  25 reports (25 source;
  18 capture nothing,
  7 capture).
- `package/music-player/desktop-app`:
  builds with the Slint build script and the repository's `SLINT_ENABLE_EXPERIMENTAL_FEATURES=1`
  (set in the root `.cargo/config.toml`).
  Cold 50.2 s,
  warm 1.4 s,
  and 11.1 s after touching `src/main.rs` (single run).
  6571 reports,
  of which 5252 are inside the generated `app.rs`;
  the 1319 own reports are 670 source,
  649 tests;
  849 capture nothing,
  470 capture.
- `package/music-player/truepeak-core`:
  builds with nothing extra.
  Cold 3.2 s,
  warm 0.5 s.
  18 reports (13 source,
  5 tests;
  11 capture nothing,
  7 capture).
- `package/music-player/truepeak-core.bench`:
  builds with nothing extra.
  Cold 3.9 s,
  warm 0.5 s.
  70 reports (70 source in the bench crate;
  45 capture nothing,
  25 capture).
- `package/forbidden-strings/core`:
  builds with nothing extra.
  Cold 4.3 s,
  warm 0.5 s.
  124 reports (76 source,
  48 tests;
  59 capture nothing,
  65 capture).
- `package/forbidden-strings/core.bench`:
  needs the offline fetch.
  Cold 69.3 s,
  warm 0.5 s.
  88 reports (88 source;
  47 capture nothing,
  41 capture).
- `package/forbidden-strings/core.fuzz`:
  builds with nothing extra.
  Cold 4.7 s,
  warm 0.5 s.
  2 reports (2 source;
  1 captures nothing,
  1 captures).
- `package/rust-module/jsonc-edit`:
  builds with nothing extra.
  Cold 1.1 s,
  warm 0.4 s.
  62 reports (19 source,
  43 tests;
  37 capture nothing,
  25 capture).
- `package/rust-module/jsonc-edit.fuzz`:
  builds with nothing extra.
  Cold 4.1 s,
  warm 0.5 s.
  30 reports (8 source,
  22 fuzz targets;
  12 capture nothing,
  18 capture).

### Totals

- Own reports over the 24 packages:
  3160 (1967 source,
  1193 tests);
  2078 capture nothing,
  so a named function can take their place,
  and 1082 capture state that the callee does not keep,
  so a loop or a function with parameters can.
  105 are `let`-bound closures and 11 have a callee that is not a declared function.
  Fuzz crates hold 64 of them and bench crates 158.
- Reports inside generated Slint code:
  5370 (5252 in `package/music-player/desktop-app`,
  118 in `package/desktop-app/terminal`).
- Cold time summed over the 24 packages:
  522 s,
  run one after another at 8 jobs.
  Warm time summed over the 23 that were run twice:
  14.4 s.
  The whole-repository `monochromatic-lint` run in this worktree takes 1.9 s (`time monochromatic-lint`,
  single run).
  Nearly all of the cold time is the dependency compile that `cargo check` itself pays;
  the lint adds little:
  plain `cargo check --all-targets` took 40.7 s cold on `package/git-policy/cli` and 33.6 s on
  `package/forbidden-strings/cli` (three packages at a time),
  against 33.9 s and 29.8 s for the lint run (one at a time).
- Incremental runs after touching one file (single runs):
  1.2 s for `package/linter/monochromatic-lint`
  (`src/lib.rs`) and 11.1 s for `package/music-player/desktop-app` (`src/main.rs`).
  A touch of `package/git-policy/cli/src/lib.rs` rebuilt nothing in 0.6 s because its library root is `src/native/lib.rs`,
  so that run is not a measurement.

### Generated Slint code

The IDE reports nothing in generated code because it uses `slint::slint!`,
a procedural macro,
and the lint skips spans from external macros (`in_external_macro` in the lint's `src/lib.rs`).
`package/desktop-app/terminal` and `package/music-player/desktop-app` use `slint::include_modules!()`,
which pastes a file from the build directory,
so its spans look like ordinary source.
Without a change those 5370 reports cannot be rewritten by anyone,
because the generated file is overwritten on every build.
Two fixes exist.

- A lint change:
  skip a span whose file lies outside the package (under `OUT_DIR`,
  or any absolute path outside the manifest directory).
  Not tried,
  because the lint is outside this task.
- An attribute on the module boundary that both packages already have for the Clippy restriction lints
  (`mod slint_generated` in the terminal's `src/main.rs`,
  `src/slint_generated.rs` in the music player).
  Tried on a disposable copy of the terminal:
  adding
  `#[cfg_attr(dylint_lib = "monochromatic_lint_closure", allow(avoidable_closure))]` before `mod slint_generated`
  took the run from 134 reports to 16 (zero generated;
  positive control:
  the unmodified package reports 118 there).

## What the linter is today

All of this was read from `package/linter/monochromatic-lint` and the repository workflows in this session.

- Build and install:
  a standalone crate (edition 2024,
  its own `[workspace]`),
  library `monochromatic_lint`,
  binary
  `monochromatic-lint` whose `main` only returns `run_process()`.
  It is published to crates.io with prebuilt release binaries (`.github/workflows/cargo-publish.yml`),
  and the repository installs it through the root `mise.toml` entry `"cargo:monochromatic-lint" = "latest"`,
  which cargo-binstall resolves to the prebuilt archive (version 0.2.0 on this host).
  The published crate includes only `src/**/*.rs`,
  `fixtures`,
  and its manifest files.
  The Dylint library crate is `publish = false` at version 0.0.0.
- Invocation:
  the root task `lint:monochromatic-lint` runs `monochromatic-lint` with the paths after `--`
  (no path means the working directory).
  It is step two of the root `lint`,
  beside dprint and the skills check.
  `package/desktop-app/ide` and `package/wg/cli-quicker-exempt` call it from their own `lint` for their directory.
  CI:
  `.github/workflows/advisory-core.yml` passes the changed `.md`,
  `.mdx` and `.rs` files;
  `.depot/workflows/weekly-native.yml` runs it on the whole tree.
  `advisory-core.yml` installs only `pkg-config` besides the mise tools and runs `mise install` with
  `continue-on-error` on `ubuntu-latest` with a 45 minute timeout;
  the whole-tree step of `weekly-native.yml` has a 15 minute timeout.
- File discovery:
  `run_paths` in `src/run_command.rs` walks the given literal paths with the `ignore` crate
  (gitignore-aware,
  `.rs`,
  `.md`,
  `.mdx` only),
  plans each file against the flat JSONC configuration,
  then runs the per-file rules on scoped worker threads (`src/run_workers.rs`).
  Findings are JSON Lines with the rule in `code`;
  exit status is 0,
  1,
  or 2 (the run could not be trusted to be complete).
  `--stdin` lints one buffer and `--fix` rewrites files.
- Subprocesses:
  the per-file rules spawn none.
  The one semantic rule,
  `rust/require-explicit-types`,
  loads a Cargo workspace through rust-analyzer
  (`src/rust_workspace.rs`,
  `ra_ap_*` crates pinned to `=0.0.336`),
  after the workers,
  on one thread.
  That loader runs `rustc --print sysroot` (`src/rust_toolchain.rs`) and cargo through `ra_ap_project_model`,
  always `--offline --locked`,
  and its comments state that semantic checking "does not fetch packages" and that
  preparation is "fixed by the host workflow,
  never an arbitrary executable from JSONC".
  The repository configuration does not enable that rule.
- Tests:
  the package's tests run in a mount-free,
  network-disabled container with 2 GiB and 2 CPUs
  (`test:container`),
  so a test that needs the host nightly,
  `rustc-dev`,
  or `cargo-dylint` cannot run there.
- Where a Dylint step would live:
  new modules beside `src/rust_workspace.rs`,
  each under the 300 line limit of
  `rust/max-lines`,
  called from `run_paths` between `process_plans` and `finish`,
  with the plans telling it which Rust files have the rule enabled and at what severity.

## In-linter type-aware implementation: not more feasible

- Nothing changed since the spike:
  `git log --since=2026-10-17` over `src/rust_workspace.rs`,
  `src/rust_semantic_session.rs` and `src/run_finish.rs` lists no commit,
  `run_finish.rs` still requests only `WorkspacePreparation::SourceOnly`,
  the `ra_ap_*` pin is still `=0.0.336`,
  and `rust_semantic_session.rs` contains no closure code.
  The commits since touched directives and their tests.
- The spike results therefore stand ([the proposal][proposal],
  "Spike results" and "Round two results"):
  under `SourceOnly` the type criterion allows 25 of the IDE's 67 closures;
  62 of 67 needs the full dependency graph,
  `cfg(test)` plus `cfg(debug_assertions)`,
  `BuildGenerated`
  (which runs every dependency's build script and proc macros unsandboxed),
  and an edit to the IDE's `slint!` call;
  it costs 22 to 46 s on the IDE alone against 0.06 s for the syntax rule.
- The measured Dylint run on the same package:
  37.2 s cold including the whole dependency compile,
  1.0 s warm,
  with the 37 generated Slint setters resolved and no edit to the IDE.
  The type information comes from the compiler that builds the code,
  so there is no second dependency-graph loader.
- A port would also face the prerequisites of all 24 packages (see "Measured feasibility per package"),
  not only the IDE's,
  and the loader's recorded hazard that a failed workspace load is retried for every file.

## Mechanisms

### Mechanism A: the linter runs `cargo dylint` per Rust package

`monochromatic-lint` groups its Rust inputs by owning manifest (`discover_manifest` already does the ancestor search),
and for each manifest whose files select a new rule `rust/avoidable-closure` it runs
`cargo dylint --path <library> --manifest-path <manifest> -- --all-targets --keep-going --message-format json`.
It parses the `avoidable_closure` records,
maps each to a finding in the file,
applies the file's configured severity,
and turns the severity into `-W` or `-D` for the lint.
A per-package prerequisite manifest says what the build needs before it can compile (see "Moving the IDE special cases").
The library comes from a path named in the rule's options,
relative to the configuration file.

#### A: pros

- Meets the requirement literally:
  one linter,
  one JSONL channel,
  severities and exemptions from
  `monochromatic-lint.config.jsonc` (for example a block turning the rule off for tests,
  as `rust-exemptions` does).
- Every measured result in this document used this exact command shape,
  so no part of the method is speculative:
  23 of 24 packages ran on the host (6 after the offline fetch,
  the IDE with its prepared artifacts),
  and the 24th after the NDK variables.
- `cargo dylint --path ../../linter/monochromatic-lint-closure` built the library and linted
  `package/wg/cli-quicker-exempt` in 16.4 s under the repository's shared `build-dir` setting
  (the library landed under `CARGO_TARGET_DIR/dylint`),
  so the staged-copy dance of the IDE task may not be needed.
  Single run;
  the IDE task's comment says the staging was needed with its own `CARGO_BUILD_BUILD_DIR`.
- Reuses the linter's existing offline posture (`--offline --locked`,
  no fetch) and its exit status 2 for
  "the run could not be completed".
- The warm cost is small (0.4 to 1.4 s per package).

#### A: cons

- A published,
  prebuilt linter would learn about a repository-specific,
  unpublished library
  (provenance question:
  see "Open questions").
- The linter would need the host nightly with `rustc-dev`,
  `cargo-dylint`,
  and `dylint-link` for any lint run,
  and a floating-nightly bump can break the lint source for every package at once (the package README says so);
  today only the IDE gate is exposed to that.
- Tests of the new modules cannot run in the linter's own bounded container;
  they need a stub `cargo` for unit tests
  and one host-only integration task (a task that is not a `lint:closures` task,
  but is new surface).
- With changed-file inputs (CI passes them),
  one touched `.rs` file triggers a lint run of its whole crate:
  cold seconds,
  not milliseconds,
  and with the package's prerequisites.
- Cold cost:
  522 s summed over the packages at 8 jobs on this 28-core host,
  against 1.9 s for the linter today,
  and the Depot whole-tree step has a 15 minute timeout.

### Mechanism B: one workspace-level Dylint pass

One synthetic Cargo workspace lists every package,
and one `cargo dylint --workspace` run lints them all.

#### B: pros

- One cargo invocation shares dependency compilation between packages (the cold sum of 522 s would shrink).
- Prerequisites are set once.

#### B: cons

- It breaks a repository invariant:
  every package is a standalone workspace root with its own `Cargo.lock`,
  enforced by file-enforcer (`[workspace]` in each manifest).
  Resolution in a copy tree succeeded (`cargo +stable metadata` exit 0) only after deleting all 24 lockfiles,
  and it produced a single lock of 1134 packages with newer crate versions against the IDE's own 653:
  every package would be linted with different dependencies than it ships,
  and feature unification across members
  changes what compiles.
  Cargo also warned that the `[profile]` tables of non-root members are ignored.
- The compile check of that synthetic workspace (`cargo check --workspace --all-targets --exclude musicplayer-native
  --exclude ide`) failed in my copy,
  but for causes that belong to the copy (the repository root files and
  `.cargo/config.toml` environment were missing),
  so it neither supports nor refutes unification problems.
- It cannot express per-package needs:
  the Android crate needs `--target aarch64-linux-android` with NDK variables,
  the IDE needs its runtime linked into the build directory,
  and those conflict inside one invocation.
- One broken package (for example a missing system library) fails the whole pass.

### Mechanism C: an in-linter type-aware port

The rejected option of 2026-10-18:
`monochromatic-lint` implements captures and the `'static` bound with its
rust-analyzer session and drops the Dylint library.

#### C: pros

- No second tool,
  no `rustc-dev`,
  no dependence on rustc-internal API drift;
  the linter stays a prebuilt binary.
- The existing semantic engine already loads a Cargo workspace per manifest.

#### C: cons

- All the spike results stand (see "In-linter type-aware implementation:
  not more feasible"):
  25 of the IDE's 67 under the current load mode,
  and the full result needs build scripts and proc macros run unsandboxed by a linter,
  the dependency graph fetched (6 of the 23 host-buildable packages failed `cargo metadata` offline here),
  a changed cfg,
  and IDE source edits.
- 22 to 46 s on the IDE alone,
  against 1.0 s warm for Dylint.
- Wrappers such as `Arc::new(closure)` and fn-trait values are recognised by neither route;
  the rust-analyzer route adds
  its own gaps (macro token trees,
  closures whose captures it reports as zero without the full graph).
- The ra-ap crates are pinned to one exact release and the engine is unused by the repository configuration,
  so this would be its first production workload.

### Mechanism D: a mise task fan-out hidden behind the linter

The root `lint:monochromatic-lint` task gains a second step that runs a hidden task in every Rust package
(each package's `mise.toml` declares its prerequisites with `depends`,
as the IDE's does today),
and `monochromatic-lint` itself is unchanged.

#### D: pros

- Fastest to deliver and nothing in the published linter changes;
  per-package environment,
  tools and prerequisites stay in each package's `mise.toml`
  (for example `QMAKE` for the Qt package,
  which only a package's mise environment sets).
- Works today for the IDE.

#### D: cons

- It is a task per package under another name;
  whether that meets "no separate `lint:closures` task" is the owner's
  reading,
  and the requirement says the check is part of the linter,
  while this is part of the root task,
  not of the binary.
- Two result channels (cargo's output beside the linter's JSONL),
  no use of the configuration's severities or exemptions,
  and CI's changed-file mode (`lint:monochromatic-lint -- <files>`) cannot select packages by file without more glue.
- 24 package `mise.toml` edits.

## Ranking

Ranking:
A > D > C > B.

- A over D:
  A is the only option that makes the check a part of the binary,
  with one result channel and the
  existing severity and exemption configuration;
  D is cheaper to deliver and keeps repository specifics out of the
  published crate,
  but it leaves the check as a per-package task fan-out,
  which may or may not be what the owner ruled out.
  If the owner will not let the published linter carry a repository-specific library,
  D beats A.
- D over C:
  D rests on measured results for all 24 packages,
  while C has only the IDE measurement
  and that measurement needs the unsandboxed build-script load mode and IDE source edits.
- C over B:
  C keeps every package a standalone workspace with its own lockfile and adds no second toolchain
  requirement;
  B changes what is linted (one re-resolved lock of 1134 packages) and cannot express the Android or IDE needs.

## Rewrite cost

- Own reports:
  3160,
  of which 1967 are in source and 1193 in tests;
  generated-code reports (5370) are not rewritable
  and need one of the two fixes in "Generated Slint code".
- By package,
  the largest are `package/music-player/desktop-app` (1319 own),
  `package/git-policy/cli` (767),
  `package/cli/nested-wayland-session` (221),
  `package/forbidden-strings/cli` (155) and `package/forbidden-strings/core` (124).
  The IDE has none left.
- For scale:
  the IDE's conversion branch went from 1135 reports to 67 by rewriting closures into loops,
  `match`,
  and named types,
  and the 67 then became the Dylint gate's allowed set.
  The reports here are about three times that count,
  with no scheduling estimate,
  because no rate of rewriting
  has been measured on these packages.
- 2078 of the own reports capture nothing,
  so a named function replaces them;
  1082 capture state that is lent for the call,
  so a loop or a function with parameters replaces them;
  105 are `let`-bound closures that need restructuring or an enclosing-item suppression (rustc rejects an attribute on a
  closure in a `let` initializer,
  E0658).
- Severity decides when the cost falls due:
  warnings keep the gate green,
  as the removed syntax rule did
  ("warn everywhere until no finding is left,
  then error",
  the owner's schedule of 2026-10-05).
- 1193 of the reports are in tests;
  the TypeScript lint setup exempts tests from its arrow ban,
  and a Rust counterpart would clear more than a third of the count without a rewrite (an owner decision).

## Moving the IDE special cases into the linter

The IDE task does four things that no other package needs together.

- An OpenSnitch rule on this host rejects the network of the nightly cargo path
  (`doc/troubleshooting/cargo-nightly-1-101-cannot-reach-crates-io.md`),
  so the task fetches with `cargo +stable fetch --locked`
  and builds offline.
  Inside the linter this should not become a fetch:
  the linter already refuses to fetch,
  and the rule is a property of one host,
  not of the repository.
  What the linter needs is a precondition check with an actionable message (name the package and the exact command
  `cargo fetch --locked`),
  plus a place where the repository runs that command first:
  a fetch step in the root `lint` before the linter,
  or the prerequisite hook of Mechanism A.
  Measured effect:
  6 packages (not only the IDE) failed `cargo metadata` offline until fetched,
  because Dylint needs the crates of every platform in the lockfile.
- The runtime and licenses artifacts (`runtime` and `notices`,
  containers and network) feed `build.rs`,
  which panics without them.
  To live inside the linter,
  either the linter invokes a package-declared prerequisite (a mise task name from
  the per-package manifest,
  which keeps container logic out of the binary),
  or the IDE `build.rs` stops needing the artifacts for a lint build (an empty runtime under an environment variable).
  The second removes containers,
  network and the symlinks from the lint path entirely;
  it is an IDE source change and was not tried.
- The symlinks in the build directory exist because `build.rs` climbs three directories from `OUT_DIR`
  and the repository's shared `build-dir` setting moves intermediates away from `target/`.
  The same fix (reading the paths from an environment variable or from `CARGO_MANIFEST_DIR`) removes them.
- `DYLINT_RUSTFLAGS='-D avoidable_closure -D unfulfilled_lint_expectations'` becomes the severity mapping:
  the linter sets `-W` or `-D` from the rule's configured severity and reports unfulfilled expectations as findings.

Other packages bring their own needs into the same manifest:
the Android crate (NDK variables from `cargo ndk-env --target arm64-v8a`,
the target flag),
the terminal (`zig` and the zig cache variable),
the GTK pair (system GTK 4),
the Qt package (Qt 6 development packages,
`qmake`,
C++ compiler),
and the Slint pair (the experimental-features variable from `.cargo/config.toml`,
which cargo applies by itself when it runs inside the checkout).
Because the existing design says preparation is "never an arbitrary executable from JSONC",
the manifest should name prerequisites from a fixed list or name a mise task,
not hold a shell command.

## CI and developer machine requirements

- Toolchain:
  the shared floating nightly (root `mise.toml`) with the `rustc-dev` component
  (about 634 MB by the file sizes in its manifest on this host;
  the whole toolchain with the Android targets is 2.8 GB).
- Tools:
  `cargo-dylint` and `dylint-link`,
  both 6.2.0 here,
  installed by mise through cargo-binstall from
  signed prebuilt archives (the root `mise.toml` comment).
  `cargo dylint` builds a driver per toolchain on first use (42 MB under `~/.dylint_drivers/<toolchain>`;
  cached for three toolchains on this host,
  so the first-build cost and its network use were not measured).
- Library:
  built per toolchain with `dylint-link`;
  `cargo dylint` finds it only under the name
  `lib<name>@<toolchain>.so`.
- Dependencies:
  all-platform crates must be in the cargo cache before an offline run
  (a fetch step),
  and the lint package's own crates (`dylint_linting`) too.
- System requirements by package:
  GTK 4 development files,
  Qt 6 development files and `qmake`,
  `zig`,
  font `pkg-config`
  files,
  a C++ toolchain,
  and the Android NDK with `cargo-ndk`.
  The CI job that runs the linter today installs `pkg-config` only,
  so those packages would fail on a runner
  until the prerequisites are installed or the packages are skipped,
  and a skip policy is an owner decision.
- Cfg coverage:
  Dylint lints only the code that compiles for the host target and features.
  The Windows-only and macOS-only code in the desktop packages is not linted on Linux;
  the size of that code was not measured.

## When the step runs

- Directory walk and explicit paths:
  group the Rust inputs by owning manifest and lint each crate once.
- Changed-file runs in CI:
  each touched `.rs` file selects its whole crate.
- `--stdin` (the Git policy hook lints one Markdown buffer) and `--fix`:
  the step is skipped,
  because the lint has no
  machine fix and a buffer is not a crate.
- Virtual Rust inside Markdown fences and doc tests:
  not covered,
  because only compiled code is linted.

## Open questions for the owner

- May the published `monochromatic-lint` carry a repository-specific rule?
  The Dylint library is `publish = false`;
  a developer or CI machine needs its source (a path in the rule's options,
  relative to the configuration file) or a published library.
- Severity on enable:
  warnings at first (the schedule of 2026-10-05),
  or errors with the reports allowed by
  `expect` attributes from the start?
- Tests:
  apply the rule to tests (1193 own reports),
  or exempt them as the TypeScript setup does?
- Sidecar crates (fuzz and bench,
  222 own reports):
  in scope or exempt?
- Generated Slint code:
  fix the lint to skip spans outside the package,
  or add the attribute to each module boundary?
- Missing prerequisites (GTK,
  Qt,
  NDK,
  `zig`) on a machine or CI runner:
  fail the linter with status 2,
  or skip that package with a note?
- Floating nightly:
  accept that a nightly bump can break the lint for every package,
  or pin the toolchain
  (the decision record lists a pin as not approved)?
- Comment directives:
  should the Dylint findings also honour the linter's comment directives,
  or only rustc's `expect` attribute,
  as the decision record says?
- Is Mechanism D acceptable as a first step while A is built?

## What was not measured

- The IDE `runtime` and `notices` tasks (containers and network):
  the artifacts were copied from the main checkout.
- The first `cargo dylint` driver build for a toolchain,
  and any network use it needs.
- A clean CI-like machine without GTK,
  Qt,
  zig,
  the NDK,
  or `rustc-dev`.
- Run-to-run spread of every reported time (all are single runs).
- Code behind `cfg(windows)` and `cfg(target_os = "macos")`,
  and non-default feature sets.
- The compile of the synthetic workspace (it failed on causes in the copy).
- The incremental time of most packages;
  the cost of the linter orchestration itself,
  which does not exist yet.

[decision]: ../decision/rust-closure-lint-dylint.md
[proposal]: linter-selective-anonymous-functions.md
