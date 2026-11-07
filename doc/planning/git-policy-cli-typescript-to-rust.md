# Porting the TypeScript of the Git wrapper package to Rust, proposal for the owner

Status:
proposal only.
No port code is written,
no TypeScript file is deleted,
no mise task is changed.
Written 2026-11-07T17:40:00-05:00 on a worktree of `6e8521c8fa` (the merge that put the wrapper on tokio).
Every count and run time in this proposal was measured on that tree on 2026-11-07,
and its source is named beside it;
a figure with no named source is not claimed.

## What to look at and how to answer

- Purpose:
  decide whether this plan (sidecar crates, an order of work that deletes TypeScript entry by entry,
  and the proof named for each step) is what the owner wants before any port code is written.
- Inspect first:
  the section "Choices for the owner",
  because the mutation steps cannot start until the first choice (the shared mutation engine) is answered.
  Then the section "Order of work",
  which says what each step deletes and what proves the Rust equal.
- Respond by:
  one line per choice in "Choices for the owner" (for example "mutation engine: Rust crate, sidecar shape: per kind"),
  or "all recommended" for the first option of each ranking.
  Two things are not choices and are answered separately:
  whether the pause on the port is lifted,
  and the name and place of any crate the plan adds.
  A veto on a name (a sidecar, a folder) is a one-line answer;
  the names here are working names that the owner's rules MKY and VGW still have to pass.
- Where the work stands:
  the owner paused "the TypeScript port" on 2026-11-04 in favour of layout and lint fixes
  (`doc/handover/cli-git-rust-implementation.md`, line 105).
  This proposal does not lift that pause;
  it only prepares the work.

## The task and what was measured

The task as the owner recorded it:
"Port the wrapper package's 142 TypeScript files (34,270 lines) to Rust; delete each TS part once ported".
The repository records the count and the intent in `doc/handover/cli-git-rust-implementation.md`, lines 495 to 499
("all to be ported to Rust"),
and the pause at line 105.
The quoted sentence itself is the task text given for this proposal;
no file of the repository holds it word for word.

The count has grown since.
Measured on 2026-11-07 with `git ls-files` and `wc --lines` over `package/git-policy/cli`:

- 145 TypeScript files,
  34,741 lines:
  `bin/` holds 65 files and 16,043 lines,
  `e2e/` holds 80 files and 18,698 lines.
- 31 of the files are tests named `*.unit.test.ts` (17 in `bin/`, 14 in `e2e/`),
  7,737 lines,
  with 228 `it(` calls.
- 42 files are longer than 300 lines (18,472 lines in all).
  The repository linter's `rust/max-lines` limit is 300 for Rust
  (`monochromatic-lint.config.jsonc`, the `rust` block),
  so a Rust file of equal size has to be split.
  The size of the Rust that results is not forecast.
- No third-party npm package is imported.
  `package.json` lists only workspace packages and `@types/node`.
- Data that the TypeScript reads or writes and that is not TypeScript:
  `e2e/commit-shape-trace.json`,
  eighteen `perf/*.json` records,
  `e2e/concurrent-commits.Containerfile` and `native-test-base.Containerfile`.

Run times of the current scripts,
taken where one run is cheap and recorded where it is not:

- `mise run //package/git-policy/cli:test:unit` ran once on 2026-11-07,
  in a `systemd-run --user --scope` with `CPUQuota=400%`, `MemoryMax=6G`, `TasksMax=1024` and `IOWeight=10`.
  It exited 0 and printed 70 suite-level `[PASS]` lines and no `[FAIL]` line
  (log `${HOME}/temp/agent/ts-to-rust/test-unit.log`).
  The wall time is about 14 s,
  read as the log's creation time to its last write
  (the `time` wrapper wrote nothing, so this figure is derived from file times and includes `mise` start-up).
- The container gate: about half an hour for a cold build at 2 CPUs,
  about ten minutes of tests and about four minutes of Clippy
  (`doc/handover/git-policy-cli-on-tokio.md`, section "How to run the gates").
  Not run for this proposal.
- One run of the planted controls: about 24 minutes per campaign
  (the top doc of `bin/native-planted-controls-proof.ts`).
  Not run for this proposal.
- The concurrent-commit suite: 36 scenarios passed and 2 were skipped with seed 1 on 2026-10-14
  (`e2e/README.md`, section "How to run").
  Not run for this proposal.
- Not measured and so not forecast:
  how long the port takes,
  how many Rust lines it yields,
  how long the new sidecars take to compile.

## The owner's words and the rules that bind this

The owner's words,
each with the file and lines where the repository records them:

- Names and visible dependencies,
  2026-11-04,
  `doc/planning/cli-git-source-layout-target-tree.md`,
  lines 69 to 76:
  "Principle 2:
  every name of a dirent must be understandable to a monkey.
  If jargon is unavoidable,
  the file's top doc
  must explain everything,
  in a way that would make any monkey understand."
  and "Principle 3:
  Who depends on who must be immediately visible from looking at the file structure."
  The rules `MKY`, `VGW`, `LYD`, `LYL`, `LYC` and `LYF` in `CLAUDE.md` carry them.
- No self-invocation,
  `doc/handover/cli-git-rust-implementation.md`,
  lines 378 to 382:
  "We should never invoke ourselves recursively.
  That's bad design.
  Audit,
  remove,
  and ban that design aggressively."
- No unbounded waits,
  `package/git-policy/cli/SPEC.md`,
  lines 2274 to 2275:
  "nothing should ever wait forever or
  indefinitely".
- Runtime and crates,
  `doc/planning/cli-git-source-layout-target-tree.md`,
  lines 196 to 233:
  "Time stamps:
  jiff,
  Digests:
  sha1 + sha2,
  Linux calls:
  rustix,
  Memory headroom:
  sysinfo";
  "Adopt tokio,
  do not adopt ureq,
  use the `hyper` set of https libs.";
  "Full on tokio.
  Multi-thread."
- Libraries, group folders and log lines,
  same file,
  lines 145 to 169:
  "Shelf + summaries (Recommended)",
  "Four libraries (Recommended)" and
  "tracing (Recommended)"
  (libraries emit through the `tracing` crate;
  the program installs the output).
- JavaScript identity is not a requirement,
  same file,
  lines 190 to 194:
  "\" text identical to JavaScript output\" - that's not needed."
  followed by the recorded consequence:
  "a reader of a format that is still in the repository is listed before that format changes."
  The sections "Inventory" and "Order of work" list those readers.
- Other platforms,
  `doc/handover/cli-git-logging.md`,
  line 29:
  "Ignore Win and mac until I say so."
- Container tests and mutation testing are required,
  `doc/decision/cli-git-rust-rewrite.md`,
  lines 24 to 26:
  "Container tests,
  mutation testing,
  and fuzzing are completion requirements."
  So no step may delete a TypeScript runner before its Rust replacement has been shown to fail when a guard is removed.
- Every Rust crate is its own workspace root,
  `doc/decision/rust-dependency-catalog.md`,
  line 15:
  "Every Rust crate is its own Cargo workspace root".
- One shared home for the product-neutral mutation tooling,
  `doc/planning/shared-cargo-mutants-package.md`,
  lines 28 to 31:
  "The user's direction is that all product-neutral cargo-mutants tooling lives in one workspace package,
  and that each product keeps only a thin adapter that supplies its names,
  scopes,
  and defaults."
  The owner's answers of 2026-10-29 on that page (lines 415 to 425) keep the package name and its location.
- The tooling is ours to fix,
  `package/git-policy/cli/DESIGN.md`,
  lines 18 to 24:
  "Everything in this package is ours:
  the wrapper,
  its tests,
  the gate image build,
  the launchers,
  the mutation and fuzz tooling,
  and these docs."
- What the end-to-end suite must keep proving,
  `doc/decision/cli-git-concurrent-commits.md`,
  lines 217 to 270:
  container end-to-end tests "with freshly created dummy repositories replaying realistic workloads"
  are "an inherent part of this design",
  "Runs are seeded;
  a failing seed replays deterministically.",
  and every run checks the listed invariants.

The rules of `CLAUDE.md` that shape the Rust in this proposal:

- `RNQ`:
  Rust packages use Rust tests through nextest.
- `SBS`:
  sidecars sit beside their subject as `<pkg>.<kind>`.
- `LYD`, `LYL`, `LYC`, `LYF`:
  a `uses/` folder in every source folder,
  crates list crates in `Cargo.toml`,
  files in one folder never loop.
- `MXL`:
  Rust files stay under the repository limit by splitting into sibling files.
- `GFP`:
  a guard test proves nothing until it is shown to fail.
- `TRV`:
  a green run counts only when its output holds the expected suite and test names.
- `SCR` and `NMJ`:
  scripts are TypeScript today.
  The owner's port decision is an exception for this package (see the risk "A rule says scripts are TypeScript").

## What the TypeScript does and how its parts depend on each other

The package has no product code in TypeScript.
Everything in `bin/` and `e2e/` verifies the Rust crate `monochromatic-git-policy-cli` (`src/native/`):

- It builds the gate image and runs the Rust tests and Clippy in a bounded container
  (`bin/test-native-container.ts` and its helpers).
- It layers cargo-mutants over that image and proves the harness can see a missing guard
  (the mutation and planted-controls files).
- It measures the latency the wrapper adds to Git (the `native-perf` files).
- It replays seeded concurrent commit workloads through the wrapper in a network-isolated container
  and checks invariants
  (the `e2e/` files and `bin/e2e-native-container.ts`).

The import graph was extracted from every `import` and `import(` specifier of the 145 files
(`${HOME}/temp/agent/ts-to-rust/gen-inventory.ts` and `simulate-deletion.ts`).
These facts in it decide the order of work:

- These files are entry points,
  that is, no other non-test file imports them:
  `bin/build-git-test-image.ts`,
  `bin/e2e-native-container.ts`,
  `bin/mutate-native-container.ts`,
  `bin/native-mutation-parallel.ts`,
  `bin/native-perf.ts`,
  `bin/native-planted-controls-only.ts`,
  `bin/native-planted-controls-reconcile.ts`,
  `bin/test-native-container.ts`,
  `e2e/commit-shape-trace-generator.ts`
  and `e2e/concurrent-e2e-consumer.ts`.
  Each is started by one mise task or by the end-to-end runner.
- The shared modules are held alive by several entries,
  so a module can be deleted only after the last entry that reaches it is gone.
  `bin/native-verification-process.ts` is imported by 15 non-test files and one test.
- `bin/native-perf-service.ts` imports `e2e/service-launch-fixture.ts`,
  so the benchmark and the end-to-end suite share the code that starts the commit service.
- Some modules are also imported by TypeScript in another package.
  `package/git-policy/cli.fuzz/bin` (10 files, 2,058 lines) imports
  `bin/native-verification-process.ts`,
  `bin/native-wrapper-snapshot.ts`,
  `bin/container-limits.ts`
  and `bin/native-nextest-snapshot.ts`.
  Inside this package nobody imports `bin/native-nextest-snapshot.ts` at all:
  only the fuzz sidecar's `fuzz-build.ts` does.

Further facts matter for the port itself:

- The planted-control tables (`bin/native-planted-controls-commit.ts`, `-entry.ts` and `-landing.ts`)
  name 48 guard removals
  by Rust source path and exact text.
  The wrapper's layout move (issue 6261) renames those paths,
  and `doc/planning/cli-git-source-layout-target-tree.md` says the tooling is updated in the same commit as each move
  (lines 4540 to 4560).
- The end-to-end hooks and editors are Node programs written to disk as text by `e2e/hook-program-fixture.ts`
  (`HOOK_PROGRAM`, `HOLDING_EDITOR_PROGRAM`, `SCRIPTED_EDITOR_PROGRAM`),
  which is why the end-to-end image installs Node.

## Inventory

Every one of the 145 files is listed once,
in the group where it is ported.
Each entry gives its line count,
what it does,
who runs it,
what it depends on,
and which package files import it.
Line counts are `wc --lines`.
"Run by" is computed from `package/git-policy/cli/mise.toml` and the import graph:
a module is run by every task whose entry reaches it.
"Depends on" lists workspace packages,
Node built-ins,
the outside things the file's text names
(Podman, systemd, real Git, paths of the Rust sources, the wrapper executable by name),
and its package files.
There is no third-party npm dependency.

The group totals,
which add up to 145 files and 34,741 lines:

- Release guard:
  1 file,
  241 lines.
- Plumbing shared by the container runners:
  2 files,
  281 lines.
- Image and gate runners:
  16 files,
  3,913 lines.
- Mutation campaign layer:
  7 files,
  2,123 lines.
- Planted controls:
  23 files,
  5,205 lines.
- Latency benchmark:
  15 files,
  3,989 lines.
- End-to-end host side:
  2 files,
  604 lines.
- End-to-end pure models and checkers:
  26 files,
  5,607 lines.
- End-to-end processes, repositories and services:
  29 files,
  6,431 lines.
- End-to-end scenario model and catalog:
  24 files,
  6,347 lines.

### Release guard

One guard test. The release itself is not in this package's TypeScript (see "Outside the 145 files").
This group holds 1 file, 1 of them a test, 241 lines.

- `bin/native-release-bins.unit.test.ts` (241 lines)
  - What: guard that `Cargo.toml`, the release job,
    the release check and the toolchain lock list the same executables.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs`, `path`;
    touches paths of the wrapper's Rust sources, the wrapper executable by name.

### Plumbing shared by the container runners

Both files are also imported by TypeScript of the sibling package `package/git-policy/cli.fuzz/bin`.
This group holds 2 files, 0 of them tests, 281 lines.

- `bin/native-verification-process.ts` (273 lines)
  - What: shell-free command execution with a 64 MiB capture limit, container creation and cleanup,
    shared by every runner.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`, `native:perf`, `native:test:container`,
    `test:e2e:concurrent:image`, `test:e2e:concurrent:native`, `test:e2e:concurrent:native:image`,
    `test:git-image`; also imported by the TypeScript of `package/git-policy/cli.fuzz/bin`.
  - Depends on: Node built-ins `child_process`, `events`, `stream`; touches Podman.
  - Imported by: 15 non-test files of the package.
  - Imported by the tests: `native-layered-build.unit.test.ts`.
- `bin/container-limits.ts` (8 lines)
  - What: one-line re-export of the shared package's memory, CPU,
    process and disk-weight limits and their Podman arguments.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:scoped`, `native:test:container`, `test:e2e:concurrent:image`, `test:e2e:concurrent:native`,
    `test:e2e:concurrent:native:image`, `test:git-image`;
    also imported by the TypeScript of `package/git-policy/cli.fuzz/bin`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; touches `cargo-mutants`.
  - Imported by: 7 non-test files of the package.
  - Imported by the tests: `native-gate-proof.unit.test.ts`, `native-planted-parallel.unit.test.ts`,
    `native-planted-controls-proof.unit.test.ts`, `native-planted-controls-ensure.unit.test.ts`.

### Image and gate runners

Everything that builds the gate image and runs the Rust tests and Clippy in a bounded container.
This group holds 16 files, 4 of them tests, 3913 lines.

- `bin/build-git-test-image.ts` (151 lines)
  - What: builds the gate base image (newest CachyOS image plus Rust, Git, Node, pnpm, strace,
    cargo-nextest) and prints the release of each tool.
  - Run by: mise task `test:git-image`.
  - Depends on: Node built-ins `fs/promises`, `path`; touches Podman, `strace`;
    package files `container-limits.ts`, `native-test-base.ts`, `native-verification-process.ts`.
- `bin/native-test-base.ts` (495 lines)
  - What: resolves and builds the base images (newest CachyOS image the host CPU runs) and what each runner needs
    from the network.
  - Run by: imported on the way to `native:mutation`, `native:test:container`, `test:e2e:concurrent:image`,
    `test:e2e:concurrent:native`, `test:e2e:concurrent:native:image`, `test:git-image`.
  - Depends on: Node built-ins `crypto`, `fs/promises`, `path`; touches Podman, the wrapper executable by name,
    `strace`; package files `container-limits.ts`, `native-verification-process.ts`.
  - Imported by: `build-git-test-image.ts`, `test-native-container.ts`, `e2e-native-container.ts`.
  - Imported by the tests: `native-test-base.unit.test.ts`.
- `bin/native-test-base.unit.test.ts` (121 lines)
  - What: tests that the image tag follows the pulled image and the Containerfile text.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`, `path`; touches Podman;
    package files `native-test-base.ts`.
- `bin/native-nextest-snapshot.ts` (169 lines)
  - What: downloads the pinned static musl `cargo-nextest` once,
    checks its sha256 and keeps it for the offline image.
  - Run by: imported by other files only; also imported by the TypeScript of `package/git-policy/cli.fuzz/bin`.
  - Depends on: Node built-ins `crypto`, `fs/promises`, `path`; package files `native-verification-process.ts`.
- `bin/native-scanner-snapshot.ts` (109 lines)
  - What: vendors the locked registry dependencies on the host for the offline image.
  - Run by: imported on the way to `native:mutation`, `native:test:container`, `test:e2e:concurrent:image`,
    `test:e2e:concurrent:native`, `test:e2e:concurrent:native:image`.
  - Depends on: Node built-ins `fs/promises`, `path`; package files `native-verification-process.ts`.
  - Imported by: `test-native-container.ts`, `e2e-native-container.ts`.
- `bin/native-wrapper-snapshot.ts` (225 lines)
  - What: the one list of crate sources every container snapshot copies: the wrapper and each path dependency,
    transitively.
  - Run by: imported on the way to `native:mutation`, `native:test:container`, `test:e2e:concurrent:image`,
    `test:e2e:concurrent:native`, `test:e2e:concurrent:native:image`;
    also imported by the TypeScript of `package/git-policy/cli.fuzz/bin`.
  - Depends on: Node built-ins `fs/promises`, `path`; touches paths of the wrapper's Rust sources;
    package files `native-verification-process.ts`.
  - Imported by: `native-test-snapshot.ts`, `native-e2e-snapshot.ts`, `native-test-containerfile.ts`,
    `test-native-container.ts`.
  - Imported by the tests: `native-wrapper-snapshot.unit.test.ts`, `native-layered-build.unit.test.ts`.
- `bin/native-wrapper-snapshot.unit.test.ts` (208 lines)
  - What: guard that a path dependency the manifests link is also copied by the snapshot.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs`, `fs/promises`, `path`;
    touches paths of the wrapper's Rust sources; package files `native-e2e-snapshot.ts`, `native-test-snapshot.ts`,
    `native-wrapper-snapshot.ts`.
- `bin/native-test-snapshot.ts` (53 lines)
  - What: the source snapshot the gate image is built from.
  - Run by: imported on the way to `native:mutation`, `native:test:container`.
  - Depends on: Node built-ins `path`; touches paths of the wrapper's Rust sources;
    package files `native-test-settings-file.ts`, `native-wrapper-snapshot.ts`.
  - Imported by: `test-native-container.ts`.
  - Imported by the tests: `native-wrapper-snapshot.unit.test.ts`.
- `bin/native-e2e-snapshot.ts` (21 lines)
  - What: the source snapshot the end-to-end image compiles from, a small wrapper over the shared snapshot list.
  - Run by: imported on the way to `test:e2e:concurrent:image`, `test:e2e:concurrent:native`,
    `test:e2e:concurrent:native:image`.
  - Depends on: package files `native-wrapper-snapshot.ts`.
  - Imported by: `e2e-native-container.ts`.
  - Imported by the tests: `native-wrapper-snapshot.unit.test.ts`.
- `bin/native-test-settings-file.ts` (65 lines)
  - What: the name of the repository settings file as the gate image receives it.
  - Run by: imported on the way to `native:mutation`, `native:test:container`.
  - Depends on: Node built-ins `fs/promises`, `path`; touches the wrapper executable by name.
  - Imported by: `native-test-snapshot.ts`, `test-native-container.ts`.
- `bin/native-test-containerfile.ts` (138 lines)
  - What: text of the gate's test Containerfile with its layers ordered from least to most changing input.
  - Run by: imported on the way to `native:mutation`, `native:test:container`.
  - Depends on: touches `cargo-mutants`; package files `native-layered-build.ts`, `native-wrapper-snapshot.ts`.
  - Imported by: `test-native-container.ts`.
  - Imported by the tests: `native-layered-build.unit.test.ts`.
- `bin/native-layered-build.ts` (411 lines)
  - What: Containerfile steps that split the compile into a dependency layer and a source layer so a source edit
    rebuilds one crate.
  - Run by: imported on the way to `native:mutation`, `native:test:container`.
  - Depends on: Node built-ins `fs/promises`, `path`; package files `native-verification-process.ts`.
  - Imported by: `native-test-containerfile.ts`, `test-native-container.ts`.
  - Imported by the tests: `native-layered-build.unit.test.ts`.
- `bin/native-layered-build.unit.test.ts` (486 lines)
  - What: tests that read the generated Containerfile lines (layer order, stubs, no network).
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`, `path`;
    package files `native-layered-build.ts`, `native-test-containerfile.ts`, `native-verification-process.ts`,
    `native-wrapper-snapshot.ts`.
- `bin/test-native-container.ts` (256 lines)
  - What: the gate: builds the gate image from a snapshot and runs the tests and Clippy in a bounded container,
    then writes the proof.
  - Run by: mise tasks `native:test:container`, `native:mutation`.
  - Depends on: Node built-ins `fs/promises`, `path`; touches Podman, the wrapper executable by name, `strace`;
    package files `container-limits.ts`, `native-gate-proof.ts`, `native-layered-build.ts`, `native-test-base.ts`,
    `native-scanner-snapshot.ts`, `native-test-containerfile.ts`, `native-test-settings-file.ts`,
    `native-test-snapshot.ts`, `native-verification-process.ts`, `native-wrapper-snapshot.ts`.
- `bin/native-gate-proof.ts` (382 lines)
  - What: writes and reads the `manifest.json` and `passed.json` records that prove a gate passed on an image ID
    under given limits.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`, `native:test:container`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; Node built-ins `fs/promises`, `path`;
    touches `cargo-mutants`.
  - Imported by: 9 non-test files of the package.
  - Imported by the tests: `native-gate-proof.unit.test.ts`.
- `bin/native-gate-proof.unit.test.ts` (623 lines)
  - What: tests of the gate proof reader against disposable evidence directories, including torn,
    foreign and lower-limit records.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`, `os`, `path`;
    package files `container-limits.ts`, `native-gate-proof.ts`.

### Mutation campaign layer

The wrapper's thin layer over the shared TypeScript engine `@monochromatic-dev/mutation-test-cli-cargo-mutants`.
This group holds 7 files, 2 of them tests, 2123 lines.

- `bin/mutate-native-container.ts` (304 lines)
  - What: runs a scoped cargo-mutants campaign over the last gate image in bounded containers,
    after the planted controls.
  - Run by: mise tasks `native:mutation`, `native:mutation:scoped`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; Node built-ins `path`;
    touches the wrapper executable by name, `cargo-mutants`; package files `container-limits.ts`,
    `native-gate-proof.ts`, `native-pass-one.ts`, `native-planted-controls.ts`,
    `native-planted-controls-ensure.ts`, `native-planted-controls-key-parts.ts`,
    `native-planted-controls-harness.ts`, `native-planted-controls-obligation.ts`,
    `native-planted-controls-manifest.ts`, `native-planted-controls-record.ts`, `native-planted-controls-units.ts`,
    `native-mutation-options.ts`.
- `bin/native-mutation-options.ts` (111 lines)
  - What: the wrapper's own defaults (source folder, timeout bounds,
    test-selection variable) over the shared option parser.
  - Run by: imported on the way to `native:mutation`, `native:mutation:scoped`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; touches paths of the wrapper's Rust sources,
    `cargo-mutants`.
  - Imported by: `mutate-native-container.ts`.
- `bin/native-pass-one.ts` (105 lines)
  - What: pass one of a two-pass campaign:
    selects the tests that can reach the mutated files from the crate's own sources.
  - Run by: imported on the way to `native:mutation`, `native:mutation:scoped`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; Node built-ins `path`;
    touches paths of the wrapper's Rust sources, `cargo-mutants`.
  - Imported by: `mutate-native-container.ts`.
  - Imported by the tests: `native-pass-one.unit.test.ts`.
- `bin/native-pass-one.unit.test.ts` (171 lines)
  - What: tests of the pass-one test selection against this crate.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`, `mutation-test-cli-cargo-mutants`; Node built-ins `path`;
    touches paths of the wrapper's Rust sources, `cargo-mutants`; package files `native-pass-one.ts`.
- `bin/native-mutation-parallel.ts` (130 lines)
  - What: starts one capped systemd user unit per source file plus the one shared planted-controls unit.
  - Run by: mise task `native:mutation:parallel`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; Node built-ins `fs/promises`, `os`, `path`;
    touches Podman, `cargo-mutants`; package files `native-planted-controls-unit.ts`, `native-planted-parallel.ts`.
- `bin/native-planted-parallel.ts` (537 lines)
  - What: the controls of a parallel launch run beside the campaign units: unit names, systemd arguments,
    wait hints.
  - Run by: imported on the way to `native:mutation:parallel`, `native:mutation:planted`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; Node built-ins `crypto`, `path`;
    touches systemd, `cargo-mutants`; package files `container-limits.ts`, `native-gate-proof.ts`,
    `native-planted-controls-proof.ts`, `native-planted-controls-unit.ts`, `native-planted-controls-units.ts`.
  - Imported by: `native-mutation-parallel.ts`, `native-planted-controls-only.ts`.
  - Imported by the tests: `native-planted-parallel.unit.test.ts`.
- `bin/native-planted-parallel.unit.test.ts` (765 lines)
  - What: tests of the launch plan with a simulated launch token.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`, `test-harness-core`;
    Node built-ins `fs/promises`, `os`, `path`; touches systemd, `cargo-mutants`;
    package files `container-limits.ts`, `native-planted-parallel.ts`, `native-planted-controls-ensure.ts`,
    `native-planted-controls-record.ts`, `native-planted-controls-units.ts`, `native-planted-controls-harness.ts`,
    `native-planted-controls-unit.ts`, `native-planted-controls.ts`.

### Planted controls

Phase one of every campaign: plant a known guard removal, require the named test to fail, record the proof.
This group holds 23 files, 7 of them tests, 5205 lines.

- `bin/native-planted-controls.ts` (303 lines)
  - What: phase one of a campaign: plant each guard removal in its own container and require the named test to fail.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`.
  - Depends on: Node built-ins `fs/promises`, `path`; touches Podman, `cargo-mutants`;
    package files `native-planted-controls-commit.ts`, `native-planted-controls-entry.ts`,
    `native-planted-controls-landing.ts`, `native-planted-controls-outcome.ts`, `native-verification-process.ts`.
  - Imported by: 8 non-test files of the package.
  - Imported by the tests: `native-planted-parallel.unit.test.ts`,
    `native-planted-controls-key-parts.unit.test.ts`, `native-planted-controls-obligation.unit.test.ts`,
    `native-planted-controls-proof.unit.test.ts`, `native-planted-controls-ensure.unit.test.ts`.
- `bin/native-planted-controls.unit.test.ts` (67 lines)
  - What: tests that every control replaces text that occurs exactly once and names a test that exists.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`, `path`;
    package files `native-planted-controls-commit.ts`, `native-planted-controls-entry.ts`,
    `native-planted-controls-landing.ts`.
- `bin/native-planted-controls-commit.ts` (154 lines)
  - What: table of guard removals in replay, capture, commit preparation, shadow repository, signals,
    maintenance and pushes.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`.
  - Depends on: touches paths of the wrapper's Rust sources; package files `native-planted-controls.ts`.
  - Imported by: `native-planted-controls.ts`.
  - Imported by the tests: `native-planted-controls.unit.test.ts`.
- `bin/native-planted-controls-entry.ts` (107 lines)
  - What: table of guard removals in the entry point, configuration, index locks, hooks-off starts and hook reports.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`.
  - Depends on: touches paths of the wrapper's Rust sources; package files `native-planted-controls.ts`.
  - Imported by: `native-planted-controls.ts`.
  - Imported by the tests: `native-planted-controls.unit.test.ts`.
- `bin/native-planted-controls-landing.ts` (119 lines)
  - What: table of guard removals in worktree copy and the post-commit gate.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`.
  - Depends on: touches paths of the wrapper's Rust sources; package files `native-planted-controls.ts`.
  - Imported by: `native-planted-controls.ts`.
  - Imported by the tests: `native-planted-controls.unit.test.ts`.
- `bin/native-planted-controls-outcome.ts` (136 lines)
  - What: the verdict of one planted removal,
    telling a planted source that did not compile apart from an unnoticed guard.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`.
  - Depends on: nothing outside itself.
  - Imported by: `native-planted-controls.ts`.
  - Imported by the tests: `native-planted-controls-outcome.unit.test.ts`.
- `bin/native-planted-controls-outcome.unit.test.ts` (145 lines)
  - What: tests of the verdicts over nextest output text.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; touches paths of the wrapper's Rust sources,
    the wrapper executable by name; package files `native-planted-controls-outcome.ts`.
- `bin/native-planted-controls-harness.ts` (76 lines)
  - What: digest of the host-side code that runs and judges the controls, so a changed judge invalidates old proof.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`.
  - Depends on: Node built-ins `crypto`, `fs/promises`, `path`.
  - Imported by: `mutate-native-container.ts`, `native-planted-controls-proof.ts`,
    `native-planted-controls-ensure.ts`.
  - Imported by the tests: `native-planted-parallel.unit.test.ts`, `native-planted-controls-harness.unit.test.ts`.
- `bin/native-planted-controls-harness.unit.test.ts` (110 lines)
  - What: tests that the digest follows the sources and the file list.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`, `os`, `path`;
    package files `native-planted-controls-harness.ts`.
- `bin/native-planted-controls-key-parts.ts` (45 lines)
  - What: what a campaign's kept outcomes depend on, for the key of its evidence directory.
  - Run by: imported on the way to `native:mutation`, `native:mutation:scoped`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; touches `cargo-mutants`;
    package files `native-gate-proof.ts`, `native-planted-controls-record.ts`.
  - Imported by: `mutate-native-container.ts`.
  - Imported by the tests: `native-planted-controls-key-parts.unit.test.ts`.
- `bin/native-planted-controls-key-parts.unit.test.ts` (61 lines)
  - What: tests that each part changes the key.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; package files `native-planted-controls-key-parts.ts`,
    `native-planted-controls-record.ts`, `native-planted-controls.ts`.
- `bin/native-planted-controls-record.ts` (229 lines)
  - What: shape, table digest and atomic write of the controls record.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`.
  - Depends on: Node built-ins `crypto`, `fs/promises`, `path`; package files `native-planted-controls.ts`.
  - Imported by: `mutate-native-container.ts`, `native-planted-controls-key-parts.ts`,
    `native-planted-controls-manifest.ts`, `native-planted-controls-proof.ts`, `native-planted-controls-ensure.ts`.
  - Imported by the tests: `native-planted-parallel.unit.test.ts`,
    `native-planted-controls-key-parts.unit.test.ts`, `native-planted-controls-obligation.unit.test.ts`,
    `native-planted-controls-proof.unit.test.ts`, `native-planted-controls-ensure.unit.test.ts`.
- `bin/native-planted-controls-manifest.ts` (503 lines)
  - What: the obligations a campaign manifest holds and how to read them; a malformed one is an error,
    never repaired.
  - Run by: imported on the way to `native:mutation`, `native:mutation:planted`, `native:mutation:reconcile`,
    `native:mutation:scoped`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; Node built-ins `fs/promises`, `path`;
    touches `cargo-mutants`; package files `native-gate-proof.ts`, `native-planted-controls-record.ts`.
  - Imported by: `mutate-native-container.ts`, `native-planted-controls-obligation.ts`,
    `native-planted-controls-ensure.ts`.
  - Imported by the tests: `native-planted-controls-obligation.unit.test.ts`.
- `bin/native-planted-controls-obligation.ts` (398 lines)
  - What: what a campaign that deferred its controls still owes and whether a record meets it.
  - Run by: imported on the way to `native:mutation`, `native:mutation:reconcile`, `native:mutation:scoped`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; Node built-ins `path`; touches systemd,
    `cargo-mutants`; package files `native-planted-controls-manifest.ts`, `native-planted-controls-proof.ts`,
    `native-planted-controls-unit.ts`, `native-verification-process.ts`.
  - Imported by: `mutate-native-container.ts`, `native-planted-controls-reconcile.ts`.
  - Imported by the tests: `native-planted-controls-obligation.unit.test.ts`.
- `bin/native-planted-controls-obligation.unit.test.ts` (685 lines)
  - What: tests of the obligation check over many manifest and record shapes.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`, `test-harness-core`;
    Node built-ins `fs/promises`, `os`, `path`; touches `cargo-mutants`;
    package files `native-planted-controls-manifest.ts`, `native-planted-controls-obligation.ts`,
    `native-planted-controls-record.ts`, `native-planted-controls-unit.ts`, `native-planted-controls.ts`.
- `bin/native-planted-controls-proof.ts` (277 lines)
  - What: writes and reads the record a passing controls run leaves, as proof for a later campaign.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; Node built-ins `fs/promises`, `path`;
    touches `cargo-mutants`; package files `native-gate-proof.ts`, `native-planted-controls-harness.ts`,
    `native-planted-controls-record.ts`, `native-planted-controls.ts`.
  - Imported by: `native-planted-parallel.ts`, `native-planted-controls-obligation.ts`,
    `native-planted-controls-ensure.ts`.
  - Imported by the tests: `native-planted-controls-proof.unit.test.ts`.
- `bin/native-planted-controls-proof.unit.test.ts` (810 lines)
  - What: tests of the proof reader (image, limits, control table and judge digests, torn files).
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`, `os`, `path`,
    `timers/promises`; touches paths of the wrapper's Rust sources; package files `container-limits.ts`,
    `native-planted-controls-ensure.ts`, `native-planted-controls-proof.ts`, `native-planted-controls-record.ts`,
    `native-planted-controls.ts`.
- `bin/native-planted-controls-ensure.ts` (216 lines)
  - What: the decision to skip, defer to a unit, or run and record the planted controls.
  - Run by: imported on the way to `native:mutation`, `native:mutation:planted`, `native:mutation:scoped`.
  - Depends on: package files `native-gate-proof.ts`, `native-verification-process.ts`,
    `native-planted-controls-harness.ts`, `native-planted-controls-record.ts`, `native-planted-controls-proof.ts`,
    `native-planted-controls-manifest.ts`, `native-planted-controls-unit.ts`, `native-planted-controls-units.ts`,
    `native-planted-controls.ts`.
  - Imported by: `mutate-native-container.ts`, `native-planted-controls-only.ts`.
  - Imported by the tests: `native-planted-parallel.unit.test.ts`, `native-planted-controls-proof.unit.test.ts`,
    `native-planted-controls-ensure.unit.test.ts`.
- `bin/native-planted-controls-ensure.unit.test.ts` (331 lines)
  - What: tests of that decision against disposable evidence.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`, `os`, `path`;
    touches paths of the wrapper's Rust sources; package files `container-limits.ts`,
    `native-planted-controls-ensure.ts`, `native-planted-controls-record.ts`, `native-planted-controls-units.ts`,
    `native-planted-controls-unit.ts`, `native-planted-controls.ts`.
- `bin/native-planted-controls-only.ts` (139 lines)
  - What: runs only the planted controls over the tested image and records their pass (the extra unit of the
    parallel launch).
  - Run by: mise task `native:mutation:planted`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; Node built-ins `fs/promises`, `path`;
    touches Podman, the wrapper executable by name, `cargo-mutants`; package files `container-limits.ts`,
    `native-gate-proof.ts`, `native-planted-controls-ensure.ts`, `native-planted-controls.ts`,
    `native-planted-parallel.ts`, `native-verification-process.ts`.
- `bin/native-planted-controls-reconcile.ts` (55 lines)
  - What: prints, per deferred campaign, the record that validates it or "no record yet" with the unit and log;
    exits nonzero unless all are met.
  - Run by: mise task `native:mutation:reconcile`.
  - Depends on: Node built-ins `fs/promises`, `path`; package files `native-gate-proof.ts`,
    `native-planted-controls-obligation.ts`.
- `bin/native-planted-controls-unit.ts` (105 lines)
  - What: asks systemd whether the transient controls unit a campaign deferred to still runs.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:reconcile`, `native:mutation:scoped`.
  - Depends on: workspace packages `mutation-test-cli-cargo-mutants`; touches systemd, `cargo-mutants`.
  - Imported by: `native-mutation-parallel.ts`, `native-planted-parallel.ts`,
    `native-planted-controls-obligation.ts`, `native-planted-controls-ensure.ts`.
  - Imported by the tests: `native-planted-parallel.unit.test.ts`,
    `native-planted-controls-obligation.unit.test.ts`, `native-planted-controls-ensure.unit.test.ts`.
- `bin/native-planted-controls-units.ts` (134 lines)
  - What: tells a campaign unit that another unit runs the controls of its image.
  - Run by: imported on the way to `native:mutation`, `native:mutation:parallel`, `native:mutation:planted`,
    `native:mutation:scoped`.
  - Depends on: nothing outside itself.
  - Imported by: `mutate-native-container.ts`, `native-planted-parallel.ts`, `native-planted-controls-ensure.ts`.
  - Imported by the tests: `native-planted-parallel.unit.test.ts`, `native-planted-controls-ensure.unit.test.ts`.

### Latency benchmark

The host-side benchmark behind `native:perf`.
This group holds 15 files, 3 of them tests, 3989 lines.

- `bin/native-perf.ts` (468 lines)
  - What: the latency benchmark: paired direct-Git baseline for each scenario, wrapper-added delta,
    raw samples written as JSON.
  - Run by: mise task `native:perf`.
  - Depends on: Node built-ins `fs/promises`, `path`, `os`, `util`; touches the wrapper executable by name,
    `strace`; package files `native-perf-git.ts`, `native-perf-root.ts`, `native-perf-shape.ts`,
    `native-perf-service.ts`, `native-perf-judge.ts`, `native-perf-lines.ts`, `native-perf-evidence.ts`,
    `native-perf-scratch.ts`, `native-perf-scenarios.ts`, `native-perf-measure.ts`.
- `bin/native-perf-evidence.ts` (137 lines)
  - What: keeps the commit phases and the service's own account of each landing beside the timings.
  - Run by: imported on the way to `native:perf`.
  - Depends on: Node built-ins `fs/promises`, `path`; touches the wrapper executable by name.
  - Imported by: `native-perf.ts`, `native-perf-service.ts`.
- `bin/native-perf-git.ts` (402 lines)
  - What: runs the real Git at `/usr/bin/git` directly and through the wrapper in fixture repositories,
    with the scratch `bin` folder on PATH.
  - Run by: imported on the way to `native:perf`.
  - Depends on: Node built-ins `fs/promises`, `path`; touches real Git, `strace`;
    package files `native-verification-process.ts`.
  - Imported by: `native-perf.ts`, `native-perf-measure.ts`, `native-perf-service.ts`, `native-perf-shape.ts`.
- `bin/native-perf-judge.ts` (92 lines)
  - What: the two latency requirements and the verdict on a finished report.
  - Run by: imported on the way to `native:perf`.
  - Depends on: package files `native-perf-measure.ts`.
  - Imported by: `native-perf.ts`, `native-perf-lines.ts`.
  - Imported by the tests: `native-perf-judge.unit.test.ts`.
- `bin/native-perf-judge.unit.test.ts` (333 lines)
  - What: tests of the verdicts against the recorded baseline scenarios.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`;
    package files `native-perf-judge.ts`, `native-perf-lines.ts`, `native-perf-measure.ts`,
    `native-perf-statistics.ts`.
- `bin/native-perf-lines.ts` (75 lines)
  - What: the lines the benchmark prints per scenario and per broken requirement.
  - Run by: imported on the way to `native:perf`.
  - Depends on: package files `native-perf-judge.ts`, `native-perf-measure.ts`.
  - Imported by: `native-perf.ts`.
  - Imported by the tests: `native-perf-judge.unit.test.ts`.
- `bin/native-perf-measure.ts` (357 lines)
  - What: paired direct-versus-wrapped sampling for one scenario.
  - Run by: imported on the way to `native:perf`.
  - Depends on: Node built-ins `path`, `timers/promises`; touches `strace`; package files `native-perf-git.ts`,
    `native-perf-shape.ts`, `native-perf-statistics.ts`.
  - Imported by: `native-perf.ts`, `native-perf-judge.ts`, `native-perf-lines.ts`.
  - Imported by the tests: `native-perf-judge.unit.test.ts`.
- `bin/native-perf-root.ts` (99 lines)
  - What: the scratch root of a run: the copied wrapper, an empty home and the tools the hooks need.
  - Run by: imported on the way to `native:perf`.
  - Depends on: Node built-ins `fs/promises`, `path`; touches the wrapper executable by name.
  - Imported by: `native-perf.ts`.
- `bin/native-perf-scenarios.ts` (38 lines)
  - What: the table of forwarded read commands each held to the forwarded-command bound.
  - Run by: imported on the way to `native:perf`.
  - Depends on: touches real Git.
  - Imported by: `native-perf.ts`.
- `bin/native-perf-scratch.ts` (375 lines)
  - What: removes the scratch tree without racing detached Git background work.
  - Run by: imported on the way to `native:perf`.
  - Depends on: Node built-ins `fs/promises`, `path`, `timers/promises`.
  - Imported by: `native-perf.ts`.
  - Imported by the tests: `native-perf-scratch.unit.test.ts`.
- `bin/native-perf-scratch.unit.test.ts` (225 lines)
  - What: tests of that removal against a real detached writer.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `child_process`, `fs/promises`, `os`,
    `path`, `timers/promises`; package files `native-perf-scratch.ts`.
- `bin/native-perf-service.ts` (454 lines)
  - What: starts one commit service per wrapped repository and the signing key the service route needs.
  - Run by: imported on the way to `native:perf`.
  - Depends on: Node built-ins `child_process`, `fs/promises`, `events`, `net`, `path`, `timers/promises`;
    touches the wrapper executable by name, `strace`, `ssh-keygen`; package files `service-launch-fixture.ts`,
    `native-perf-git.ts`, `native-perf-evidence.ts`, `native-verification-process.ts`.
  - Imported by: `native-perf.ts`.
  - Imported by the tests: `native-perf-service.unit.test.ts`.
- `bin/native-perf-service.unit.test.ts` (410 lines)
  - What: tests of the service start and stop against real sockets.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`, `net`, `os`, `path`;
    touches the wrapper executable by name; package files `service-launch-fixture.ts`, `native-perf-service.ts`.
- `bin/native-perf-shape.ts` (274 lines)
  - What: reflink-copied fixture repositories with the shape of the person's main worktree,
    and the page cache of their files.
  - Run by: imported on the way to `native:perf`.
  - Depends on: Node built-ins `fs/promises`, `path`; touches real Git; package files `native-perf-git.ts`,
    `native-verification-process.ts`.
  - Imported by: `native-perf.ts`, `native-perf-measure.ts`.
- `bin/native-perf-statistics.ts` (250 lines)
  - What: median, p95 and median absolute deviation of the samples.
  - Run by: imported on the way to `native:perf`.
  - Depends on: nothing outside itself.
  - Imported by: `native-perf-measure.ts`.
  - Imported by the tests: `native-perf-judge.unit.test.ts`.

### End-to-end host side

What runs on the host: the runner that builds the images and starts the container, and the trace miner.
This group holds 2 files, 0 of them tests, 604 lines.

- `bin/e2e-native-container.ts` (291 lines)
  - What: host-side runner of the concurrent-commit suite with subcommands `base`,
    `image` and `run` that build the images and start the bounded, network-isolated container.
  - Run by: mise tasks `test:e2e:concurrent:image`, `test:e2e:concurrent:native:image`,
    `test:e2e:concurrent:native`.
  - Depends on: Node built-ins `fs/promises`, `path`; touches Podman, the wrapper executable by name, `ssh-keygen`;
    package files `container-limits.ts`, `native-e2e-snapshot.ts`, `native-scanner-snapshot.ts`,
    `native-test-base.ts`, `native-verification-process.ts`.
- `e2e/commit-shape-trace-generator.ts` (313 lines)
  - What: host-side miner that writes `e2e/commit-shape-trace.json` from real Git history, bypassing the wrapper.
  - Run by: mise task `e2e:concurrent:trace`.
  - Depends on: workspace packages `logger-core`; Node built-ins `child_process`, `events`, `fs/promises`, `path`,
    `util`, `stream/consumers`; touches real Git; package files `commit-shape-log-fixture.ts`,
    `commit-shape-trace-fixture.ts`.

### End-to-end pure models and checkers

Files that touch no process and no file system (the ledger only hashes),
so their behavior is a function of their input.
This group holds 26 files, 8 of them tests, 5607 lines.

- `e2e/commit-shape-log-fixture.ts` (368 lines)
  - What: parses `git log --raw --numstat -z` output into per-commit change records (pure text).
  - Run by: imported on the way to `e2e:concurrent:trace`; runs inside the end-to-end container.
  - Depends on: touches real Git.
  - Imported by: `commit-shape-trace-generator.ts`, `commit-shape-trace-fixture.ts`.
  - Imported by the tests: `commit-shape-trace.unit.test.ts`.
- `e2e/commit-shape-trace-fixture.ts` (625 lines)
  - What: the commit-shape trace model: anonymizes mined commits, serializes the trace and validates it on read.
  - Run by: imported on the way to `e2e:concurrent:trace`; runs inside the end-to-end container.
  - Depends on: package files `commit-shape-log-fixture.ts`.
  - Imported by: `commit-shape-trace-generator.ts`, `trace-replay-fixture.ts`, `concurrent-e2e-consumer.ts`,
    `scenario-model-fixture.ts`.
  - Imported by the tests: `commit-shape-trace.unit.test.ts`, `trace-replay.unit.test.ts`.
- `e2e/commit-shape-trace.unit.test.ts` (269 lines)
  - What: tests of the log parser and the trace model.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; package files `commit-shape-log-fixture.ts`,
    `commit-shape-trace-fixture.ts`.
- `e2e/content-fixture.ts` (299 lines)
  - What: deterministic synthetic file contents (text ends in one LF, binary starts with NUL).
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `seeded-random-fixture.ts`.
  - Imported by: 18 non-test files of the package.
  - Imported by the tests: `trace-replay.unit.test.ts`.
- `e2e/git-version-fixture.ts` (102 lines)
  - What: Git version comparison and replay-plumbing applicability.
  - Run by: runs inside the end-to-end container.
  - Depends on: nothing outside itself.
  - Imported by: `scenario-model-fixture.ts`.
- `e2e/invariant-fixture.ts` (421 lines)
  - What: the pure invariant checker for one scenario run.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `invariant-model-fixture.ts`.
  - Imported by: `observation-fixture.ts`, `observation-attempt-fixture.ts`, `scenario-model-fixture.ts`.
  - Imported by the tests: `invariant.unit.test.ts`.
- `e2e/invariant-model-fixture.ts` (277 lines)
  - What: the observation records the checker consumes (digests, not bytes).
  - Run by: runs inside the end-to-end container.
  - Depends on: nothing outside itself.
  - Imported by: 7 non-test files of the package.
  - Imported by the tests: `invariant.unit.test.ts`.
- `e2e/invariant.unit.test.ts` (230 lines)
  - What: tests of the invariants: clean run, exactly-once landing, killed attempts, wrong bytes,
    remote containment.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; package files `invariant-fixture.ts`,
    `invariant-model-fixture.ts`, `scenario-model-fixture.ts`.
- `e2e/jsonl-event-fixture.ts` (256 lines)
  - What: extracts JSONL policy events from wrapper stderr and checks them against the exit code.
  - Run by: runs inside the end-to-end container.
  - Depends on: workspace packages `caught-value-core`; touches real Git.
  - Imported by: `observation-fixture.ts`, `observation-attempt-fixture.ts`, `hooks-ignored-fixture.ts`,
    `scenario-foreign-unproven-fixture.ts`, `scenario-replay-degradation-fixture.ts`.
  - Imported by the tests: `jsonl-event.unit.test.ts`.
- `e2e/jsonl-event.unit.test.ts` (113 lines)
  - What: tests of event extraction, sequence gaps and exit consistency.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; touches real Git; package files `jsonl-event-fixture.ts`.
- `e2e/leftover-fixture.ts` (105 lines)
  - What: classifies Git administrative entries that must not survive a finished run.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches the wrapper executable by name.
  - Imported by: `observation-fixture.ts`.
  - Imported by the tests: `leftover.unit.test.ts`.
- `e2e/leftover.unit.test.ts` (105 lines)
  - What: tests of that classification.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; touches the wrapper executable by name;
    package files `leftover-fixture.ts`.
- `e2e/push-rejection-fixture.ts` (111 lines)
  - What: whether the refusal of an auto-push reached the person.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches paths of the wrapper's Rust sources; package files `ledger-fixture.ts`.
  - Imported by: `observation-attempt-fixture.ts`.
  - Imported by the tests: `push-rejection.unit.test.ts`.
- `e2e/push-rejection.unit.test.ts` (100 lines)
  - What: tests of that decision.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; package files `ledger-fixture.ts`,
    `push-rejection-fixture.ts`.
- `e2e/report-fixture.ts` (142 lines)
  - What: plain-text report of scenario results.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `scenario-model-fixture.ts`.
  - Imported by: `concurrent-e2e-consumer.ts`.
  - Imported by the tests: `trace-replay.unit.test.ts`.
- `e2e/repository-model-fixture.ts` (125 lines)
  - What: repository types shared by setup and workloads.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches real Git; package files `hook-program-fixture.ts`, `service-model-fixture.ts`.
  - Imported by: `real-git-fixture.ts`, `repository-fixture.ts`, `repository-setup-fixture.ts`,
    `scenario-service-kill-fixture.ts`.
- `e2e/seeded-random-fixture.ts` (350 lines)
  - What: seeded pseudo-random source with forks by stable label, so one printed seed replays a run.
  - Run by: runs inside the end-to-end container.
  - Depends on: nothing outside itself.
  - Imported by: 7 non-test files of the package.
  - Imported by the tests: `seeded-schedule.unit.test.ts`, `trace-replay.unit.test.ts`.
- `e2e/seeded-schedule-fixture.ts` (208 lines)
  - What: seeded start offsets, interleavings and fault timing for concurrent workloads.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `seeded-random-fixture.ts`.
  - Imported by: `scenario-helper-fixture.ts`, `scenario-index-writer-fixture.ts`,
    `scenario-phase-kill-fixture.ts`, `scenario-service-kill-fixture.ts`, `scenario-sigkill-fixture.ts`.
  - Imported by the tests: `seeded-schedule.unit.test.ts`.
- `e2e/seeded-schedule.unit.test.ts` (221 lines)
  - What: tests that schedules repeat for one seed.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; package files `seeded-random-fixture.ts`,
    `seeded-schedule-fixture.ts`.
- `e2e/service-model-fixture.ts` (63 lines)
  - What: what a scenario may do to its commit service.
  - Run by: runs inside the end-to-end container.
  - Depends on: nothing outside itself.
  - Imported by: `repository-model-fixture.ts`, `repository-fixture.ts`, `service-fixture.ts`.
- `e2e/trace-replay-fixture.ts` (266 lines)
  - What: replays a window of the committed trace as explicit-path commits with synthesized content.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `commit-shape-trace-fixture.ts`, `seeded-random-fixture.ts`.
  - Imported by: `trace-execution-fixture.ts`, `scenario-setup-fixture.ts`, `scenario-baseline-commit-fixture.ts`,
    `scenario-concurrent-paths-fixture.ts`.
  - Imported by the tests: `trace-replay.unit.test.ts`.
- `e2e/trace-replay.unit.test.ts` (225 lines)
  - What: tests of the replay plan.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; package files `commit-shape-trace-fixture.ts`,
    `content-fixture.ts`, `report-fixture.ts`, `scenario-model-fixture.ts`, `seeded-random-fixture.ts`,
    `trace-replay-fixture.ts`, `trace-execution-fixture.ts`.
- `e2e/ledger-fixture.ts` (288 lines)
  - What: per-scenario ledger of writes, staging, attempts with captured bytes and auxiliary commands,
    with content digests.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `crypto`; package files `invariant-model-fixture.ts`, `process-fixture.ts`.
  - Imported by: 20 non-test files of the package.
  - Imported by the tests: `push-rejection.unit.test.ts`, `ledger.unit.test.ts`.
- `e2e/ledger.unit.test.ts` (123 lines)
  - What: tests of the ledger.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; package files `ledger-fixture.ts`.
- `e2e/scenario-outcome-fixture.ts` (52 lines)
  - What: outcomes of a commit attempt the accepted design allows besides landing.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `ledger-fixture.ts`.
  - Imported by: `scenario-expectation-fixture.ts`, `scenario-lock-gc-fixture.ts`.
  - Imported by the tests: `scenario-model.unit.test.ts`.
- `e2e/scenario-expectation-fixture.ts` (163 lines)
  - What: expectations a scenario attaches to its run (every attempt exited 0 or ended as an accepted outcome).
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `invariant-model-fixture.ts`, `ledger-fixture.ts`, `scenario-outcome-fixture.ts`.
  - Imported by: `scenario-model-fixture.ts`.

### End-to-end processes, repositories and services

The container-side code that starts programs, builds repositories and watches the commit service.
This group holds 29 files, 4 of them tests, 6431 lines.

- `e2e/process-fixture.ts` (481 lines)
  - What: captured child runs, process-group handles for fault injection and file-marker waits.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `child_process`, `events`, `fs/promises`, `stream/consumers`, `timers/promises`;
    touches real Git.
  - Imported by: 13 non-test files of the package.
  - Imported by the tests: `process-fixture.unit.test.ts`.
- `e2e/process-fixture.unit.test.ts` (40 lines)
  - What: tests of a failed start.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; package files `process-fixture.ts`.
- `e2e/real-git-fixture.ts` (94 lines)
  - What: real Git by absolute path for setup and observation, never through the wrapper.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches real Git; package files `process-fixture.ts`, `repository-model-fixture.ts`.
  - Imported by: `repository-fixture.ts`, `repository-setup-fixture.ts`.
- `e2e/git-read-fixture.ts` (415 lines)
  - What: read-only real-Git queries the observation gatherer uses.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`; touches real Git; package files `ledger-fixture.ts`,
    `invariant-model-fixture.ts`, `process-fixture.ts`, `repository-fixture.ts`.
  - Imported by: `observation-fixture.ts`, `observation-attempt-fixture.ts`.
- `e2e/hook-program-fixture.ts` (305 lines)
  - What: the text of the Node programs registered as hooks and installed as editors (they hold and release on
    marker files).
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`.
  - Imported by: `repository-model-fixture.ts`, `repository-fixture.ts`, `repository-setup-fixture.ts`,
    `scenario-setup-fixture.ts`.
- `e2e/editor-attempt-fixture.ts` (44 lines)
  - What: options that make a commit open the suite's scripted editor.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches real Git; package files `repository-fixture.ts`.
  - Imported by: `scenario-amend-switch-fixture.ts`, `scenario-baseline-editor-fixture.ts`,
    `scenario-baseline-hook-fixture.ts`, `scenario-sigkill-fixture.ts`.
- `e2e/repository-fixture.ts` (256 lines)
  - What: fresh dummy repositories for one scenario: bare remote, upstream, linked worktree, hooks, SSH signing.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`; touches real Git; package files `hook-program-fixture.ts`,
    `real-git-fixture.ts`, `repository-model-fixture.ts`, `repository-setup-fixture.ts`,
    `service-model-fixture.ts`, `wrapper-selection-fixture.ts`.
  - Imported by: 19 non-test files of the package.
- `e2e/repository-setup-fixture.ts` (285 lines)
  - What: setup steps of a scenario repository.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`; touches real Git, `ssh-keygen`;
    package files `hook-program-fixture.ts`, `process-fixture.ts`, `real-git-fixture.ts`,
    `repository-model-fixture.ts`.
  - Imported by: `repository-fixture.ts`.
- `e2e/wrapper-selection-fixture.ts` (196 lines)
  - What: names the wrapper put first on PATH and checks it is the native executable.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`; touches the wrapper executable by name.
  - Imported by: `repository-fixture.ts`, `concurrent-e2e-consumer.ts`.
  - Imported by the tests: `wrapper-selection.unit.test.ts`.
- `e2e/wrapper-selection.unit.test.ts` (86 lines)
  - What: tests with a fake ELF and a fake script `git`.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`, `path`;
    package files `wrapper-selection-fixture.ts`.
- `e2e/service-connection-fixture.ts` (78 lines)
  - What: connection probes of the commit service socket and the files it leaves.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `events`, `net`; touches the wrapper executable by name.
  - Imported by: `service-fixture.ts`, `service-settle-fixture.ts`, `push-notice-fixture.ts`.
- `e2e/service-launch-fixture.ts` (192 lines)
  - What: launching the commit service program of the image with the service words.
  - Run by: imported on the way to `native:perf`; runs inside the end-to-end container.
  - Depends on: Node built-ins `child_process`, `fs/promises`; touches paths of the wrapper's Rust sources,
    the wrapper executable by name, real Git.
  - Imported by: `native-perf-service.ts`, `service-fixture.ts`.
  - Imported by the tests: `native-perf-service.unit.test.ts`, `service-launch.unit.test.ts`,
    `service-mode-words.unit.test.ts`.
- `e2e/service-launch.unit.test.ts` (55 lines)
  - What: tests that the harness names a worktree's service as the wrapper does (same vector as a Rust test).
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; touches paths of the wrapper's Rust sources,
    the wrapper executable by name; package files `scenario-catalog-fixture.ts`, `service-fixture.ts`,
    `service-launch-fixture.ts`.
- `e2e/service-mode-words.unit.test.ts` (113 lines)
  - What: guard that the words the harness starts the service with equal the words in `service_main.rs`.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs`, `path`;
    touches paths of the wrapper's Rust sources; package files `service-launch-fixture.ts`.
- `e2e/service-fixture.ts` (341 lines)
  - What: the commit service of one scenario, started as socket activation would start it.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `child_process`, `events`, `fs/promises`, `crypto`, `path`, `timers/promises`;
    touches the wrapper executable by name; package files `service-connection-fixture.ts`,
    `service-launch-fixture.ts`, `service-model-fixture.ts`.
  - Imported by: `scenario-model-fixture.ts`.
  - Imported by the tests: `service-launch.unit.test.ts`.
- `e2e/service-settle-fixture.ts` (161 lines)
  - What: waits for the service to finish the after-commit work of a repository.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`, `timers/promises`; touches the wrapper executable by name;
    package files `service-connection-fixture.ts`.
  - Imported by: `observation-fixture.ts`, `scenario-baseline-editor-fixture.ts`,
    `scenario-baseline-hook-fixture.ts`, `scenario-service-kill-fixture.ts`.
- `e2e/client-connection-fixture.ts` (138 lines)
  - What: observes a wrapper's open socket to the commit service, which marks the end of its preparation.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`, `timers/promises`; package files `process-fixture.ts`.
  - Imported by: `trace-execution-fixture.ts`, `scenario-queue-fixture.ts`.
- `e2e/barrier-fixture.ts` (163 lines)
  - What: hook barriers and settle windows that pin scenario interleavings.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`; package files `process-fixture.ts`, `repository-fixture.ts`.
  - Imported by: `scenario-amend-switch-fixture.ts`, `scenario-foreign-unproven-fixture.ts`,
    `scenario-lock-gc-fixture.ts`, `scenario-replay-degradation-fixture.ts`, `scenario-sigkill-fixture.ts`.
- `e2e/foreign-commit-fixture.ts` (110 lines)
  - What: a real-Git commit by absolute path whose editor holds `index.lock`,
    standing for activity that bypasses the wrapper.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches real Git; package files `process-fixture.ts`, `repository-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-foreign-unproven-fixture.ts`, `scenario-lock-gc-fixture.ts`.
- `e2e/worker-fixture.ts` (476 lines)
  - What: scenario actors: worktree writes, commit attempts through the wrapper, auxiliary wrapper commands.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`; touches real Git; package files `ledger-fixture.ts`,
    `process-fixture.ts`, `repository-fixture.ts`.
  - Imported by: 24 non-test files of the package.
- `e2e/work-record-fixture.ts` (328 lines)
  - What: observes the after-commit work records a landed commit leaves for the service.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`, `timers/promises`; touches the wrapper executable by name;
    package files `process-fixture.ts`, `repository-fixture.ts`.
  - Imported by: 7 non-test files of the package.
- `e2e/landed-commit-fixture.ts` (42 lines)
  - What: reads the commit an attempt landed, found by the token its message carries.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches real Git; package files `ledger-fixture.ts`, `repository-fixture.ts`,
    `scenario-model-fixture.ts`.
  - Imported by: `scenario-baseline-editor-fixture.ts`, `scenario-baseline-hook-fixture.ts`,
    `scenario-sigkill-fixture.ts`.
- `e2e/landing-kill-fixture.ts` (57 lines)
  - What: kills a committing agent at the instant its commit landed.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `scenario-model-fixture.ts`, `worker-fixture.ts`, `work-record-fixture.ts`.
  - Imported by: `scenario-sigkill-fixture.ts`.
- `e2e/push-notice-fixture.ts` (74 lines)
  - What: shows the service's push failure notices to the harness's own wrapped command.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`; touches paths of the wrapper's Rust sources,
    the wrapper executable by name; package files `service-connection-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `observation-fixture.ts`.
- `e2e/observation-fixture.ts` (464 lines)
  - What: reduces a finished scenario's repositories and ledger to the observation, with real Git by absolute path.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`; touches real Git; package files `git-read-fixture.ts`,
    `invariant-model-fixture.ts`, `invariant-fixture.ts`, `leftover-fixture.ts`, `jsonl-event-fixture.ts`,
    `ledger-fixture.ts`, `observation-attempt-fixture.ts`, `process-fixture.ts`, `push-notice-fixture.ts`,
    `repository-fixture.ts`, `service-settle-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-model-fixture.ts`.
- `e2e/observation-attempt-fixture.ts` (514 lines)
  - What: builds per-attempt observations: which history commits carry the token and what they landed.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`; touches the wrapper executable by name, real Git;
    package files `git-read-fixture.ts`, `invariant-model-fixture.ts`, `invariant-fixture.ts`,
    `jsonl-event-fixture.ts`, `ledger-fixture.ts`, `push-rejection-fixture.ts`, `process-fixture.ts`,
    `repository-fixture.ts`.
  - Imported by: `observation-fixture.ts`.
- `e2e/trace-execution-fixture.ts` (482 lines)
  - What: runs planned trace operations through the wrapper with bounded concurrency.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `content-fixture.ts`, `ledger-fixture.ts`, `scenario-model-fixture.ts`,
    `trace-replay-fixture.ts`, `client-connection-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-baseline-commit-fixture.ts`, `scenario-concurrent-paths-fixture.ts`.
  - Imported by the tests: `trace-replay.unit.test.ts`.
- `e2e/hooks-ignored-fixture.ts` (129 lines)
  - What: expectations for repositories that configure hooks the wrapper no longer runs.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `path`; package files `jsonl-event-fixture.ts`, `ledger-fixture.ts`,
    `process-fixture.ts`, `scenario-model-fixture.ts`.
  - Imported by: `scenario-baseline-hook-fixture.ts`, `scenario-hooked-fixture.ts`.
- `e2e/concurrent-e2e-consumer.ts` (312 lines)
  - What: container entry point: parses options, runs the scenario catalog,
    prints the report and the replay command.
  - Run by: started inside the end-to-end container by `e2e-native-container.ts run`.
  - Depends on: Node built-ins `child_process`, `crypto`, `fs/promises`, `path`, `util`; touches real Git;
    package files `commit-shape-trace-fixture.ts`, `repository-fixture.ts`, `report-fixture.ts`,
    `scenario-catalog-fixture.ts`, `scenario-model-fixture.ts`, `seeded-random-fixture.ts`,
    `wrapper-selection-fixture.ts`.

### End-to-end scenario model and catalog

The scenario contract, the helpers every scenario shares and the scenario definitions themselves.
This group holds 24 files, 2 of them tests, 6347 lines.

- `e2e/scenario-model-fixture.ts` (535 lines)
  - What: the scenario contract and runner: fresh repositories, workload, observation, invariant check.
  - Run by: runs inside the end-to-end container.
  - Depends on: workspace packages `caught-value-core`; Node built-ins `fs/promises`, `path`, `timers/promises`;
    package files `commit-shape-trace-fixture.ts`, `invariant-model-fixture.ts`, `invariant-fixture.ts`,
    `ledger-fixture.ts`, `observation-fixture.ts`, `service-fixture.ts`, `process-fixture.ts`,
    `repository-fixture.ts`, `seeded-random-fixture.ts`, `worker-fixture.ts`, `git-version-fixture.ts`,
    `scenario-expectation-fixture.ts`.
  - Imported by: 27 non-test files of the package.
  - Imported by the tests: `invariant.unit.test.ts`, `trace-replay.unit.test.ts`, `scenario-model.unit.test.ts`,
    `service-kill-lease.unit.test.ts`.
- `e2e/scenario-model.unit.test.ts` (155 lines)
  - What: tests of the runner and the conflict check.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; package files `scenario-model-fixture.ts`,
    `scenario-outcome-fixture.ts`.
- `e2e/scenario-helper-fixture.ts` (390 lines)
  - What: building blocks shared by scenario definitions.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `timers/promises`; package files `content-fixture.ts`, `ledger-fixture.ts`,
    `repository-fixture.ts`, `scenario-model-fixture.ts`, `seeded-schedule-fixture.ts`, `seeded-random-fixture.ts`,
    `worker-fixture.ts`.
  - Imported by: 16 non-test files of the package.
- `e2e/scenario-setup-fixture.ts` (300 lines)
  - What: setup steps shared by several scenarios: trace seeding, token-free amend bases, partial staging.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches real Git; package files `content-fixture.ts`, `hook-program-fixture.ts`,
    `ledger-fixture.ts`, `repository-fixture.ts`, `scenario-model-fixture.ts`, `trace-replay-fixture.ts`,
    `worker-fixture.ts`.
  - Imported by: `scenario-amend-switch-fixture.ts`, `scenario-baseline-amend-fixture.ts`,
    `scenario-baseline-commit-fixture.ts`, `scenario-baseline-hook-fixture.ts`,
    `scenario-concurrent-paths-fixture.ts`, `scenario-hooked-fixture.ts`.
- `e2e/scenario-queue-fixture.ts` (123 lines)
  - What: pins an interleaving by freezing the scenario's commit service.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `client-connection-fixture.ts`, `ledger-fixture.ts`, `scenario-model-fixture.ts`,
    `worker-fixture.ts`.
  - Imported by: `scenario-lock-gc-fixture.ts`, `scenario-phase-kill-fixture.ts`,
    `scenario-service-kill-fixture.ts`, `scenario-shared-file-fixture.ts`, `scenario-sigkill-fixture.ts`,
    `scenario-subsumption-fixture.ts`.
- `e2e/scenario-catalog-fixture.ts` (50 lines)
  - What: the complete scenario catalog in report order.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `scenario-amend-switch-fixture.ts`, `scenario-baseline-amend-fixture.ts`,
    `scenario-baseline-commit-fixture.ts`, `scenario-baseline-editor-fixture.ts`,
    `scenario-baseline-hook-fixture.ts`, `scenario-concurrent-paths-fixture.ts`,
    `scenario-foreign-unproven-fixture.ts`, `scenario-hooked-fixture.ts`, `scenario-index-writer-fixture.ts`,
    `scenario-lock-gc-fixture.ts`, `scenario-model-fixture.ts`, `scenario-phase-kill-fixture.ts`,
    `scenario-replay-degradation-fixture.ts`, `scenario-service-kill-fixture.ts`,
    `scenario-shared-file-fixture.ts`, `scenario-sigkill-fixture.ts`, `scenario-subsumption-fixture.ts`.
  - Imported by: `concurrent-e2e-consumer.ts`.
  - Imported by the tests: `service-launch.unit.test.ts`.
- `e2e/scenario-claim-recovery-fixture.ts` (216 lines)
  - What: what a killed service's claims must come to once a service runs again.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `scenario-model-fixture.ts`, `repository-fixture.ts`, `work-record-fixture.ts`.
  - Imported by: `scenario-service-kill-fixture.ts`.
- `e2e/service-kill-lease.unit.test.ts` (110 lines)
  - What: tests that the claim-recovery wait equals the lease the service gives a claim and fits the scenario
    timeout.
  - Run by: mise task `test:unit`.
  - Depends on: workspace packages `test-harness-core`; Node built-ins `fs/promises`, `path`;
    touches paths of the wrapper's Rust sources; package files `scenario-service-kill-fixture.ts`,
    `scenario-model-fixture.ts`.
- `e2e/scenario-amend-switch-fixture.ts` (291 lines)
  - What: scenarios: an amend racing concurrent commits, and a branch switch during preparation.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches real Git; package files `barrier-fixture.ts`, `content-fixture.ts`,
    `editor-attempt-fixture.ts`, `repository-fixture.ts`, `scenario-concurrent-paths-fixture.ts`,
    `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`, `scenario-setup-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-baseline-amend-fixture.ts` (147 lines)
  - What: scenarios: single-commit baselines for amend and a lint-staged style partial staging.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `content-fixture.ts`, `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`,
    `scenario-setup-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-baseline-commit-fixture.ts` (291 lines)
  - What: scenarios: single-commit baselines for explicit paths, index commits and sequential trace replay.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `content-fixture.ts`, `scenario-model-fixture.ts`, `scenario-helper-fixture.ts`,
    `scenario-setup-fixture.ts`, `trace-execution-fixture.ts`, `trace-replay-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-baseline-editor-fixture.ts` (244 lines)
  - What: scenarios: baselines for the message editor and a kill after the landing.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `editor-attempt-fixture.ts`, `content-fixture.ts`, `ledger-fixture.ts`,
    `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`, `landed-commit-fixture.ts`,
    `service-settle-fixture.ts`, `work-record-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-baseline-hook-fixture.ts` (285 lines)
  - What: scenarios: baselines for configured hooks, signing and the checker's positive control.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `fs/promises`, `path`; touches real Git; package files `content-fixture.ts`,
    `editor-attempt-fixture.ts`, `hooks-ignored-fixture.ts`, `ledger-fixture.ts`, `repository-fixture.ts`,
    `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`, `scenario-setup-fixture.ts`,
    `landed-commit-fixture.ts`, `service-settle-fixture.ts`, `work-record-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-concurrent-paths-fixture.ts` (145 lines)
  - What: scenarios: concurrent agents over disjoint paths and over replayed trace shapes.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`,
    `scenario-setup-fixture.ts`, `trace-execution-fixture.ts`, `trace-replay-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`, `scenario-amend-switch-fixture.ts`,
    `scenario-foreign-unproven-fixture.ts`, `scenario-hooked-fixture.ts`, `scenario-lock-gc-fixture.ts`.
- `e2e/scenario-foreign-unproven-fixture.ts` (210 lines)
  - What: scenarios: a foreign `index.lock` holder with no owner evidence.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `timers/promises`; touches real Git; package files `barrier-fixture.ts`,
    `content-fixture.ts`, `foreign-commit-fixture.ts`, `jsonl-event-fixture.ts`, `ledger-fixture.ts`,
    `scenario-concurrent-paths-fixture.ts`, `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`,
    `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-hooked-fixture.ts` (248 lines)
  - What: scenarios: concurrent agents in hooked, signed and lint-staged repositories.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `content-fixture.ts`, `hooks-ignored-fixture.ts`,
    `scenario-concurrent-paths-fixture.ts`, `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`,
    `scenario-setup-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-index-writer-fixture.ts` (327 lines)
  - What: scenarios: commits interleaved with `git add` writers in a seeded order.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches real Git; package files `content-fixture.ts`, `repository-fixture.ts`,
    `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`, `seeded-schedule-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-lock-gc-fixture.ts` (267 lines)
  - What: scenarios: a foreign `index.lock` holder, and `gc --prune=now` while commits wait.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `timers/promises`; touches real Git; package files `barrier-fixture.ts`,
    `content-fixture.ts`, `foreign-commit-fixture.ts`, `scenario-concurrent-paths-fixture.ts`,
    `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`, `scenario-outcome-fixture.ts`,
    `scenario-queue-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-phase-kill-fixture.ts` (274 lines)
  - What: scenarios: `SIGKILL` to the service at each instant of a landing batch, then restart,
    retries and a follow-up commit.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `timers/promises`; touches paths of the wrapper's Rust sources;
    package files `content-fixture.ts`, `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`,
    `scenario-queue-fixture.ts`, `seeded-schedule-fixture.ts`, `work-record-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`, `scenario-service-kill-fixture.ts`.
- `e2e/scenario-replay-degradation-fixture.ts` (295 lines)
  - What: scenarios: replay on a Git without replay plumbing fails fast with the documented code.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches real Git; package files `content-fixture.ts`, `barrier-fixture.ts`,
    `jsonl-event-fixture.ts`, `ledger-fixture.ts`, `repository-fixture.ts`, `scenario-helper-fixture.ts`,
    `scenario-model-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-service-kill-fixture.ts` (405 lines)
  - What: scenarios: `SIGKILL` to the service from outside at observable or seeded instants.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `timers/promises`; touches paths of the wrapper's Rust sources;
    package files `content-fixture.ts`, `seeded-random-fixture.ts`, `repository-model-fixture.ts`,
    `scenario-claim-recovery-fixture.ts`, `scenario-phase-kill-fixture.ts`, `scenario-helper-fixture.ts`,
    `scenario-model-fixture.ts`, `scenario-queue-fixture.ts`, `seeded-schedule-fixture.ts`,
    `service-settle-fixture.ts`, `work-record-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
  - Imported by the tests: `service-kill-lease.unit.test.ts`.
- `e2e/scenario-shared-file-fixture.ts` (320 lines)
  - What: scenarios: two agents editing one file with far-apart and overlapping hunks.
  - Run by: runs inside the end-to-end container.
  - Depends on: package files `content-fixture.ts`, `ledger-fixture.ts`, `scenario-helper-fixture.ts`,
    `scenario-queue-fixture.ts`, `scenario-model-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-sigkill-fixture.ts` (468 lines)
  - What: scenarios: `SIGKILL` to one committing agent at each observable point, then recovery.
  - Run by: runs inside the end-to-end container.
  - Depends on: Node built-ins `timers/promises`; package files `barrier-fixture.ts`, `content-fixture.ts`,
    `editor-attempt-fixture.ts`, `landing-kill-fixture.ts`, `ledger-fixture.ts`, `landed-commit-fixture.ts`,
    `scenario-helper-fixture.ts`, `scenario-model-fixture.ts`, `scenario-queue-fixture.ts`,
    `seeded-schedule-fixture.ts`, `work-record-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.
- `e2e/scenario-subsumption-fixture.ts` (251 lines)
  - What: scenarios: a second agent edits next to a first agent's in-flight edit and replays with subsumption.
  - Run by: runs inside the end-to-end container.
  - Depends on: touches real Git; package files `content-fixture.ts`, `repository-fixture.ts`,
    `scenario-helper-fixture.ts`, `scenario-queue-fixture.ts`, `scenario-model-fixture.ts`, `worker-fixture.ts`.
  - Imported by: `scenario-catalog-fixture.ts`.

## Outside the 145 files

### What names the TypeScript and has to change with it

These are not among the 145 files,
but each one names, runs or imports them,
so each changes in the step that deletes its target.

- `package/git-policy/cli/mise.toml`:
  every task that runs `node bin/...ts`,
  the inline TypeScript of `e2e:concurrent:trace`,
  `lint:oxlint:paths` and `format:oxlint:paths`,
  and the tasks that extend the root templates `lint`, `lint:types`, `lint:oxlint`, `format:oxlint` and `test:unit`.
- `package/git-policy/cli/package.json` (four `bin` names:
  `git-policy-native-test`,
  `git-policy-native-mutation`,
  `git-policy-native-e2e`,
  `git-policy-test-git-image`;
  an `rg` over `.github`, the root `mise.toml` and every package's `mise.toml` and `package.json`
  finds no user of them),
  `tsconfig.json`,
  and the importer entry `package/git-policy/cli:` in `pnpm-lock.yaml` (line 2402).
- Documents that name the files:
  `README.md`,
  `SPEC.md`,
  `e2e/README.md`,
  `doc/planted-control-and-durable-survivors-resume.md`,
  `native-test-base.Containerfile`,
  `toolchain.plan.jsonc` (a comment naming `bin/native-scanner-snapshot.ts`),
  `.github/workflows/cargo-publish.yml` (a comment naming `bin/native-release-bins.unit.test.ts`)
  and `package/mutation-test/cli-cargo-mutants/README.md`.
- `package/git-policy/cli/Cargo.toml` has a stale comment:
  it says the release guard is `native_release_bins_tests` in `src/native/binary/binary_release_bins_tests.rs`,
  but that file does not exist.
  The Rust half of the guard is `src/native/source_guard/release_bins_tests.rs`,
  and the half that reads files outside the crate is the TypeScript test.
  The release-guard step fixes the comment.
- The shared engine `package/mutation-test/cli-cargo-mutants`:
  111 TypeScript files,
  25,875 lines,
  and 43 files `src/golden/git-policy-*.json` that pin the arguments this package's scripts pass.
  Its `package.json` consumers are this package and five others:
  `forbidden-strings/cli`,
  `forbidden-strings/cli.fuzz`,
  `forbidden-strings/core`,
  `linter/monochromatic-lint`
  and `linter/monochromatic-lint.fuzz`.
  This package imports 34 distinct symbols from six of its modules in 15 files (tests included).
- `package/git-policy/cli.fuzz/bin/*.ts` (10 files, 2,058 lines) imports four modules of this package's `bin/`
  (listed in the section "What the TypeScript does and how its parts depend on each other").
- `package/crate-release/cli/bin/dispatch-release.ts` and `await-release.ts`,
  run by `native:release` and `native:release:wait`:
  another package's TypeScript.
- `package/crate-release/cli/releasable-crates.json`:
  the release bumps exact pins only in the files this list names for the wrapper
  (`pinFiles` and `lockDirs` name the fuzz sidecar today),
  so every sidecar that pins the wrapper has to be added to it.

Formats that stay in the repository after the port,
and so each have a reader that must keep working while both implementations exist
(the owner's rule recorded at `doc/planning/cli-git-source-layout-target-tree.md`, lines 190 to 194):

- `e2e/commit-shape-trace.json`:
  written by `e2e/commit-shape-trace-generator.ts`,
  read by `e2e/concurrent-e2e-consumer.ts` through `e2e/commit-shape-trace-fixture.ts`.
- The gate records `manifest.json` and `passed.json`:
  written by `bin/test-native-container.ts`,
  read by `bin/native-gate-proof.ts`
  (and, through the shared engine, by the mutation campaigns).
- The planted-controls record `planted-controls-passed.json`
  and the campaign records `manifest.json`, `exit.json`, `container-started.json` and `attempt-protocol.json`:
  written by this package and the shared engine,
  read by `bin/native-planted-controls-manifest.ts`, `-obligation.ts` and `-proof.ts`.
- `package/crate-release/cli/releasable-crates.json` and `toolchain.lock.json`:
  read by `bin/native-release-bins.unit.test.ts`.
- `perf/*.json`:
  written by `bin/native-perf.ts`.
  No TypeScript file reads them,
  so there is no reader to keep working in both directions;
  the documents cite them,
  so they stay as the historical record,
  and the new program writes new files that `jq` can read (rule `WR6` covers any moved identifier).

### What cannot be Rust, and what only looked as if it could not

Nothing in the 145 files has to stay JavaScript.

- No GitHub Actions step runs any of them.
  `rg --hidden --fixed-strings` over `.github` for the file name of each of the 145 files,
  for `git-policy/cli:<task>` of each non-lint task of the package and for the four `bin` names of `package.json`
  (178 patterns) found one hit,
  a comment (`.github/workflows/cargo-publish.yml`, line 2414, names the release guard).
  The workflows build and publish with `cargo`,
  and run `task-run //package/<name>:test:unit` for the packages they name
  (`toml-edit-fuzz.yml`, `env-local-json.yml`, `advisory-core.yml`, `logger-fuzz.yml`)
  or for a published package (`npm-publish.yml`, whose package is an input;
  this package is `"private": true` and is never published to npm).
  So the case "a step the Actions runner must run as JavaScript" does not occur here.
- Node and pnpm stay in the gate image after the port.
  The wrapper's own Rust code starts a real `pnpm` (`src/native/repair/package_repair_pnpm.rs`, line 28:
  `const PROGRAM: &str = "pnpm";`),
  and `native-test-base.Containerfile` says "`nodejs` and `pnpm` serve the controls that run a real pnpm".
  Only `e2e/concurrent-commits.Containerfile` loses `nodejs`,
  and only when the hook and editor programs are Rust (the choice "Hook and editor programs").
- The shared engine and the sibling fuzz sidecar's TypeScript are outside the 145 files.
  They stay TypeScript until the owner answers the choices "The shared mutation engine"
  and "The fuzz sidecar's TypeScript".
  Until then the mutation step and the deletion of four shared modules are blocked,
  and the proposal says which files wait.
- The release dispatch and wait scripts in `package/crate-release/cli/bin` are another package's work.
  The tasks `native:release` and `native:release:wait` keep running them.

## Rust shape

### Design adopted for every group

These are adopted unless the owner vetoes them.
Each rests on a quoted owner decision or on a repository precedent named beside it.

- Where the Rust lives:
  sidecar crates beside the subject,
  `package/git-policy/cli.<kind>`.
  The published crate cannot take another executable:
  `Cargo.toml` says "The release archive holds exactly this one file,
  and cargo-binstall refuses an archive that lacks any `[[bin]]` of the crate",
  and the guard that is being ported exists to keep it so.
  The precedent is `package/git-policy/cli.fuzz`:
  `publish = false`,
  several `[[bin]]`,
  the subject as a path dependency with an exact version pin,
  its own workspace root and lock file seeded by the task `lock:subject`,
  and a `mise.toml` with `test`, `lint` and `lint:types` tasks.
  Rule `SBS` places such a package as `<pkg>.<kind>`.
  The sidecars' exact pins are written by the catalog sync
  (`doc/decision/rust-dependency-catalog.md`, lines 56 to 58),
  and bumped at release only where `releasable-crates.json` lists them.
- Runtime:
  tokio with the multi-thread runtime,
  from the owner's "Full on tokio.
  Multi-thread."
  Time stamps with `jiff`,
  digests with `sha1` and `sha2`,
  Linux calls with `rustix`,
  memory headroom with `sysinfo`,
  JSON with `serde_json`:
  all owner picks or already direct dependencies of the wrapper.
- Log lines through `tracing` (the owner's pick for libraries),
  with each binary installing the output.
- Flags:
  the choice "Flag parsing".
- Errors:
  enums implementing `std::error::Error` with `source()`,
  which replaces `@monochromatic-dev/caught-value-core`.
  No crate is added for this.
- Waits and cleanup:
  every wait on a child,
  a socket,
  a file marker or a container has a deadline in the call
  and a named constant that cites its measurement or the owner's decision (rule `LQM`).
  A deadline only ends the wait,
  so the runner also owns what it started.
  On a deadline,
  an output overflow,
  a cancellation or a failed start it sends `SIGTERM` to the child's process group,
  waits a named grace,
  sends `SIGKILL`,
  reaps the child,
  closes and drains the output pipes within a named bound even when a descendant still holds them,
  and for a container runs a bounded `podman rm --force`.
  The cleanup itself has a deadline and reports what it could not clean.
  Each sidecar copies the wrapper's `.config/nextest.toml` (`slow-timeout` of 60 s,
  `terminate-after` 5),
  so a hung test ends with its name.
- No self-invocation:
  no sidecar program starts itself,
  and no runner is registered as a program Git or a unit will start.
  A program another program has to run (a hook, an editor, a supervised unit) is a separate executable.
- Linux only:
  nothing carries a `#[cfg(not(unix))]` branch.
- Exit codes and streams:
  `doc/decision/cli-output-and-exit-codes.md` binds the programs
  (0 for success and explicit `--help`,
  1 when the operation failed,
  2 for a usage or configuration error;
  result text only on stdout,
  diagnostics on stderr).
  The decision on argument parsing (`doc/decision/cli-arg-validation.md`) prescribes `node:util` `parseArgs`,
  so it does not speak for Rust.
- Layout from the first file:
  the rules `LYD`, `LYL`, `LYC` and `LYF` apply as written,
  and each sidecar's lint configuration selects the checks that enforce them
  (the linter has them off unless a configuration block selects them,
  `doc/handover/cli-git-rust-implementation.md`, the entry for 2026-11-05).
  Folder and file names are plain sentences under `MKY` and `VGW`,
  each folder's top doc lists its files in dependency order,
  each source folder holds a `uses/` folder,
  Rust files stay under the repository's 300 line limit,
  every item has rustdoc,
  and unit tests sit in `_tests.rs` sidecars.
  The owner's "Shelf + summaries (Recommended)" answer concerns the wrapper's group folders;
  a sidecar has no group folders until it needs them.
- Tests:
  nextest through each sidecar's `test` task (rule `RNQ`).
  Scratch lives under `${HOME}/temp/agent` or the build directory,
  never `/tmp` (rules `XTD`, `HOM`).

What replaces each npm dependency:

- `@monochromatic-dev/test-harness-core`
  (31 test files):
  `#[test]` functions in `_tests.rs` sidecars,
  run by nextest.
  The `describe` and `it` names become module and test names.
- `@monochromatic-dev/logger-core`
  (only `e2e/commit-shape-trace-generator.ts`):
  `tracing`.
- `@monochromatic-dev/caught-value-core`
  (`e2e/jsonl-event-fixture.ts`, `e2e/scenario-model-fixture.ts`):
  error enums with `source()`.
- `@monochromatic-dev/mutation-test-cli-cargo-mutants`
  (15 files in `bin/`):
  the choice "The shared mutation engine".
- `@monochromatic-dev/config-typescript` and `@types/node`:
  dropped with `tsconfig.json`.

What replaces each Node built-in the files import:

- `node:child_process` and `node:events`:
  `tokio::process` under the cleanup contract of the design;
  process groups for fault injection through `rustix`.
- `node:fs` and `node:fs/promises`:
  `std::fs` where a step is short and `tokio::fs` where it sits among other waits.
- `node:crypto` (`createHash`):
  `sha2` and `sha1`.
  `randomUUID` in the TypeScript becomes `getrandom`,
  already a dependency of the wrapper.
- `node:net`:
  `tokio::net::UnixStream` for the commit service socket.
- `node:timers/promises`:
  `tokio::time` with an explicit timeout.
- `node:os`, `node:path`, `node:util`, `node:stream`:
  `std::env`, `std::path`, the flag parser of the choice "Flag parsing",
  and `tokio` streams.
- `performance.now`:
  `std::time::Instant`.

### Sidecars and how they depend on each other

The working names in this section are proposals that rules `MKY` and `VGW` still have to accept;
a package folder is exempt from `VGW`,
and `MKY` still asks that a name say what its contents do.

- `package/git-policy/cli.container-tests`:
  the image and gate runners,
  the gate proof,
  the host-side release guard,
  and, until the choice "Where the shared command runner lives" moves it,
  the command runner and container limits the other sidecars use.
- `package/git-policy/cli.mutation`:
  the mutation campaign layer and the planted controls.
  Its `Cargo.toml` lists `cli.container-tests` (campaigns layer over the gate image and read its proof),
  and the engine crate the owner picks.
- `package/git-policy/cli.e2e`:
  the concurrent-commit suite,
  its host runner,
  the trace miner and the hook and editor programs.
  Its `Cargo.toml` lists `cli.container-tests` (images, snapshots and the command runner)
  and the wrapper library for the names it shares (command word, service words, service socket naming, claim lease).
- `package/git-policy/cli.bench`:
  the latency benchmark.
  `bench` is one of the kinds `SBS` names.
  Its `Cargo.toml` lists the wrapper library for the same shared names,
  and the command runner from wherever the choice "Where the shared command runner lives" puts it.

Cargo refuses loops between crates,
so this order is also the build order:
the wrapper library,
then `cli.container-tests`,
then `cli.bench`, `cli.e2e` and `cli.mutation`.

### Release guard

- Crate and place:
  a `[[test]]` target `tests/the_release_lists_name_the_same_executables.rs` in `cli.container-tests`,
  run on the host,
  because it reads files outside the crate and the gate container holds only crate sources.
  The Rust half that reads `Cargo.toml` already exists in the wrapper
  (`src/native/source_guard/release_bins_tests.rs`, two tests);
  the port adds the three lists it lacks:
  the release job's `--bin` and `bin=` lines in `cargo-publish.yml`,
  `releasable-crates.json` `checkBins`,
  and `toolchain.lock.json` `bins`.
- Shape:
  the comparison is a function that takes the four texts and returns one line per disagreement,
  as `disagreements` does in the TypeScript.
  The test of the real files and the tests of planted texts call that function,
  so removing the function's enforcement makes the planted-text tests fail.
- Test tool:
  nextest.
- npm replacement:
  `serde_json` for the two JSON files;
  the workflow is read as text between the job markers,
  as the TypeScript does,
  so no YAML crate is added.
- Behavior is kept:
  a release job that is not found is an error and not an empty list,
  and a manifest with no `[[bin]]` fails the test,
  as in the TypeScript tests.

### Plumbing shared by the container runners

- Crate and place:
  `cli.container-tests`,
  a folder `running_programs_without_a_shell/` (working name),
  until the choice "Where the shared command runner lives" moves the command runner.
- What it holds:
  the command runner with the 64 MiB capture limit
  (a command that cannot start,
  overflows or exits nonzero fails naming the command),
  the cleanup contract of the design,
  container creation and cleanup,
  and the container limits (`--memory`, `--cpus`, process count, disk weight) with their Podman arguments.
- Dependency on the engine:
  `bin/container-limits.ts` only re-exports the engine's module,
  so the Rust limits are either that engine's module ported
  (choice "The shared mutation engine", option A)
  or a Rust copy in `cli.container-tests` that shares test vectors with `container-limits.unit.test.ts` of the engine.
- Tool:
  `tokio::process`,
  or the wrapper library's captured-run helper if its public items suffice
  (`pub mod process` is declared in `src/native/lib.rs`, line 267, and `captured_child_run_to_its_end` is a `pub mod`
  there;
  whether the helper meets the cleanup contract was not checked for this proposal).

### Image and gate runners

- Crate and place:
  `cli.container-tests`,
  two programs (working names):
  `build-the-test-base-image` (task `test:git-image`)
  and `run-the-tests-in-a-container` (tasks `native:test:container` and the first half of `native:mutation`).
- Folders (working names):
  `choosing_and_building_base_images/`,
  `copying_the_wrapper_sources_into_an_image/`
  (wrapper, test, end-to-end and scanner snapshots,
  the settings-file name,
  the pinned `cargo-nextest` download that only the fuzz sidecar uses today),
  `writing_the_test_containerfile/`
  (the Containerfile text and its layered build)
  and `recording_that_a_gate_passed/`.
- Tests:
  the four test files become `_tests.rs`.
  The Containerfile text is pure,
  so a test compares generated lines.
- npm replacement:
  `sha2` for the digests,
  `tokio::process` for `podman`,
  `serde_json` for the records.
- Records:
  `manifest.json` and `passed.json` keep their fields while both gates exist,
  because the TypeScript reader and the shared engine read them.

### Mutation campaign layer and planted controls

- Crate and place:
  `cli.mutation`.
  Programs (working names):
  `mutate-the-wrapper` (tasks `native:mutation` second half and `native:mutation:scoped`),
  `mutate-the-wrapper-in-parallel`,
  `run-only-the-planted-controls`
  and `check-the-planted-controls-were-met` (task `native:mutation:reconcile`).
- Folders (working names):
  `choosing_the_tests_that_can_reach_a_mutated_file/`,
  `launching_campaigns_in_parallel/`,
  `planting_guard_removals_and_judging_them/`
  (the planted-control tables as Rust constants split by area,
  the runner and the verdict),
  `recording_what_the_planted_controls_proved/`
  (record,
  harness digest,
  key parts,
  manifest,
  obligation,
  proof,
  ensure,
  unit and units).
- Tests:
  the nine test files become `_tests.rs`.
  One of them asserts that every control replaces text that occurs exactly once and names a test that exists;
  it keeps that job and reads the wrapper's sources by path.
- Rust shape differs from the TypeScript in one place:
  the engine builds the unit's command as shell text (`buildShellBody`, a retry loop run by `systemd-run`).
  Rule `SCR` forbids new shell scripts,
  so the Rust port supervises the retry in a small program of its own instead of generating shell.
  That changes the unit's command line,
  not what a campaign records.
- Evidence a Rust judge cannot inherit:
  a record proves the harness that wrote it
  (`bin/native-planted-controls-harness.ts` makes a changed judge invalidate old proof).
  So every record written before the port is intentionally invalid as proof for the Rust judge,
  and the first campaign after the port reruns the controls (about 24 minutes).
  The Rust reader still parses the old records and says why it rejects them.
- Dependency:
  everything here waits for the owner's answer to "The shared mutation engine"
  and for the wrapper's layout move,
  because the tables name wrapper source paths.

### Latency benchmark

- Crate and place:
  `cli.bench`,
  one program `measure-the-latency-the-wrapper-adds` (working name) for the task `native:perf`.
- Folders (working names):
  `sampling_git_directly_and_through_the_wrapper/`,
  `statistics_of_the_samples/`,
  `judging_the_two_latency_requirements/`,
  `fixture_repositories_the_size_people_commit_at/`
  (the reflink copy and the page cache of the files),
  `removing_scratch_after_detached_git_work/`
  and `starting_the_commit_service_for_the_service_route/`.
- Tests:
  the three test files become `_tests.rs`;
  the statistics and the verdicts are pure.
- npm replacement:
  `std::time::Instant`,
  `rustix` for the reflink and the cache calls,
  `tokio::process` and `tokio::net` for the service,
  `serde_json` for the raw samples.
- The benchmark starts the commit service the way the end-to-end suite does.
  The port starts it through the wrapper library's names
  (`SERVICE_WORDS` in `src/native/service/service_main.rs` and `socket_for` in `src/native/service/service_instance.rs`
  are declared `pub`, checked 2026-11-07),
  so the benchmark does not depend on `cli.e2e`.
  Its command runner comes from the choice "Where the shared command runner lives".

### End-to-end host side and the trace miner

- Crate and place:
  `cli.e2e`,
  programs (working names):
  `run-the-concurrent-commit-suite` with the subcommands `base`, `image` and `run`
  (tasks `test:e2e:concurrent:image`, `test:e2e:concurrent:native:image`, `test:e2e:concurrent:native`)
  and `mine-commit-shapes` (task `e2e:concurrent:trace`,
  whose inline TypeScript in `mise.toml` becomes a plain `cargo run`).
- Image change:
  the image stage that compiles the release wrapper in the gate's base image also compiles the suite's programs,
  and the end-to-end base image stops installing `nodejs`
  and stops linking `caught-value-core` under `/node_modules`
  (two lines of `e2e/concurrent-commits.Containerfile`),
  once the hooks and editors are Rust.
- Tests:
  the trace miner is checked against real Git on a disposable clone,
  not in the gate.

### End-to-end pure models and checkers

- Crate and place:
  `cli.e2e`,
  folders (working names):
  `deciding_whether_a_run_kept_its_promises/`
  (invariants, observation model, expectations, outcomes, leftovers, push refusals),
  `what_the_harness_did_to_a_repository/` (ledger, content, repository and service models),
  `random_choices_that_replay_from_one_seed/`,
  `commit_shapes_mined_from_history/`,
  `reading_policy_events_from_stderr/`,
  `git_versions_and_replay_plumbing/`
  and `plain_text_report/`.
- Tests:
  the eight test files become `_tests.rs`.
  These files touch no process,
  so their Rust is plain functions.
- Seeded generator:
  Mulberry32 with an FNV-1a seed derivation over the text `<seed>:<label>`
  (`e2e/seeded-random-fixture.ts`).
  Both use only 32-bit wrapping arithmetic,
  so Rust reproduces them exactly (choice "The seeded generator").
  The seed fixes the workload's random decisions;
  it does not fix operating-system scheduling.
- The trace reader and writer keep `e2e/commit-shape-trace.json` readable,
  as the section "Outside the 145 files" lists.

### End-to-end processes, repositories and services

- Crate and place:
  `cli.e2e`,
  folders (working names):
  `starting_and_watching_programs_in_the_container/`
  (captured runs,
  process group handles for fault injection,
  file marker waits),
  `building_dummy_repositories/`,
  `running_the_commit_service/`,
  `scenario_actors/`
  and `what_a_finished_run_looks_like/`,
  plus the container entry program `run-the-scenarios-in-the-container`.
- The hook and editor programs:
  separate executables of `cli.e2e`,
  working names `hook-that-records-and-can-hold`,
  `editor-that-holds-until-released`
  and `editor-that-marks-two-points` (choice "Hook and editor programs").
  The lint-staged emulation inside the hook keeps starting real `git`
  and every wait keeps its deadline (120 s for a hook barrier in the TypeScript,
  with no measurement recorded beside it,
  so rule `LQM` asks for one when the constant is ported).
- Tests:
  the four test files become `_tests.rs`.
  Three guards of the TypeScript read or re-derive what the wrapper already defines:
  `service-mode-words.unit.test.ts` parses `service_main.rs` and `wrapper_names_old_and_new.rs` for the service words,
  `service-kill-lease.unit.test.ts` parses `CLAIM_LEASE` out of the service source,
  and `service-launch.unit.test.ts` re-derives the service instance name against a vector the wrapper also pins.
  The harness links the wrapper library and uses its constants and functions
  (choice "How the harness learns the wrapper's names"),
  so the first and the third stop being parsers.
  The lease guard keeps its job as a Rust test:
  the harness's claim-recovery budget must exceed the library's `CLAIM_LEASE` and fit the scenario timeout,
  and lowering the budget to less than the lease in a disposable copy must fail it.

### End-to-end scenario model and catalog

- Crate and place:
  `cli.e2e`,
  folders (working names):
  `what_a_scenario_is_and_how_it_runs/`
  and `scenarios_by_what_they_stress/` with one file per definition group (the 16 definition files of the TypeScript,
  4,468 lines,
  each split under the 300 line limit if its Rust is over it).
- Catalog:
  an ordered constant,
  as `e2e/scenario-catalog-fixture.ts` is today,
  plus a committed list of scenario names that neither implementation generates
  (the independent expected catalog the proof compares against).
- Tests:
  the two test files become `_tests.rs`.

## Order of work

The order follows the import graph,
not the groups:
a TypeScript file can go only when nothing that remains imports it,
tests included.
`${HOME}/temp/agent/ts-to-rust/simulate-v2.ts` (kept in scratch, not committed) simulates the deletions step by step
over the 145 files.
It treats a remaining entry or a remaining test as a root,
deletes a test at the earliest step at which one of the files it imports is deleted
(so its Rust twin must exist by that step),
and checks after every step that no remaining file imports a deleted file.
It reported that check as validated.
The section "Deletion manifest by step" lists every file at its step.
The figures in each step are that simulation's,
with the four modules the fuzz sidecar imports kept until the last step.

The steps,
smallest and most independent first:

1.  The choices and the skeletons.
2.  The release guard.
3.  The base image build and the plumbing it needs.
4.  The commit-shape trace miner.
5.  The latency benchmark.
6.  The gate and its proof.
7.  The concurrent-commit suite, scenario by scenario.
8.  The mutation campaign layer and the planted controls.
9.  The fuzz sidecar's own TypeScript (only if the owner picks to port it).
10. The four modules the fuzz sidecar imports.
11. The package's JavaScript scaffolding.

### Proof protocol for every step

- Capture first,
  and keep the reference:
  run the TypeScript on fixed inputs and commit the outputs as fixtures of the sidecar before porting.
  Inject the clocks,
  random sources and paths that make an output vary;
  where a field still varies,
  normalize only that documented field.
  A fixture that stays unstable is a blocker of its step and is named as one,
  never dropped.
  The commit that holds the last TypeScript state is recorded in `doc/planning/`,
  and the captured reports and fixtures are the durable oracle,
  because a TypeScript tree checked out later needs its workspace packages installed to run.
- Equality is semantic,
  because the owner dropped JavaScript identity.
  A correctness difference is always a finding:
  a failed or missing invariant,
  a missing or extra event,
  an unexpected exit,
  an absent scenario,
  a changed pass or fail verdict.
  Statistical tolerance applies only to named timing measurements,
  and each step that has one names which.
- Formats that stay in the repository:
  each is read by both implementations in both directions while both exist,
  and the readers are tested on rejected,
  deferred,
  torn,
  foreign and changed-digest inputs as well as on good ones.
  Parsing another implementation's record is not accepting it as evidence:
  the compatibility of each record kind is stated as parsed,
  accepted as proof,
  or intentionally rejected.
- Same inputs to both:
  where a step has a runner,
  both run on the same snapshot,
  the same seed or the same release binary,
  and their verdicts are compared.
- Positive control (rules `GFP`, `QPC`):
  every ported guard or checker is shown to fail on a planted defect,
  and the TypeScript is shown to fail on the same defect,
  so a Rust test that cannot fail never replaces a TypeScript test that can.
  A guard is separated from its test:
  the guard is a function,
  the test calls it with allowed and rejected fixtures,
  and the proof removes the guard's enforcement and shows the rejected fixtures stop failing the guard
  (a test whose assertion is removed is vacuous, not failing).
  Each step lists the negative controls it needs,
  one per failure branch of what it ports.
- Cleanup (the design's contract):
  each runner step has negative tests for a child that ignores `SIGTERM`,
  a descendant that holds an output pipe,
  output past the capture limit,
  an interrupted container run and a failed start.
- Names (rule `TRV`):
  the commit that deletes a TypeScript test file lists in its body each `it(` name
  and the Rust test that replaces it,
  and the nextest output of the proof run shows those names.
- Contention:
  nothing is repeated to generate load;
  a heavy proof run is one real job in its own `systemd-run --user --scope` with CPU,
  memory,
  process-count and disk-weight bounds sized to it.
- Commits:
  the proof and the deletion are separate commits with explicit pathspecs,
  each committed before work on another step starts.

### The choices and the skeletons

- Creates:
  the answers of "Choices for the owner" recorded in `doc/planning/`,
  the sidecar crates as empty compilable packages with their `mise.toml`, `README.md`, nextest bound and lock files
  (each added to `releasable-crates.json` `pinFiles` and `lockDirs` if it pins the wrapper),
  the first `cargo check` timings of the sidecars that link the wrapper library
  (the cost the section "Risks" says is unmeasured),
  the capture fixtures of every pure group
  (generator vectors,
  invariant verdicts,
  parser outputs,
  Containerfile text,
  record shapes,
  limit parsing),
  and for the end-to-end suite:
  the seed corpus,
  the TypeScript reports for the whole corpus,
  and the independent list of scenario names.
- Deletes:
  nothing.
- Proof:
  the skeletons pass their own `lint` and `test` tasks,
  and every fixture is regenerated from the TypeScript twice to show it is stable.

### The release guard

- Creates:
  the comparison function and its tests in `cli.container-tests`,
  the missing lists,
  and the corrected comment in `Cargo.toml`.
- Deletes:
  `bin/native-release-bins.unit.test.ts`
  (1 file,
  241 lines;
  144 files and 34,500 lines remain).
- Negative controls:
  a second `[[bin]]`,
  a lock or a release check that still lists the executable before the rename,
  a missing release job,
  and an empty release job.
- Proof:
  both guards pass on the real files;
  both fail on a disposable copy of each file with each planted defect,
  naming the same disagreements;
  and with the comparison function made to return nothing,
  the rejected-fixture tests fail (rule `GFP`).

### The base image build and the plumbing it needs

- Creates:
  the command runner with its cleanup contract,
  the container limits,
  the base image logic and the program `build-the-test-base-image`.
- Deletes:
  `bin/build-git-test-image.ts`
  (1 file,
  151 lines;
  143 files and 34,349 lines remain).
  The shared modules stay because other entries still reach them.
- Negative controls:
  a pull or build that fails,
  a limit text that is not a size,
  and the cleanup tests of the protocol.
- Proof:
  both programs name the same image tag for the same pulled image and Containerfile text
  (the image tag is a function of those inputs,
  so equality is checked on recorded vectors and on one live pull),
  both print the same release of each tool,
  and the limit parser agrees with the engine's test vectors.

### The commit-shape trace miner

- Creates:
  the log parser,
  the trace model and the program `mine-commit-shapes`.
- Deletes:
  `e2e/commit-shape-trace-generator.ts`
  (1 file,
  313 lines;
  142 files and 34,036 lines remain).
  The parser and model stay until the suite is ported,
  because the TypeScript consumer reads the trace through them.
- Negative controls:
  a log with a malformed record,
  an unknown change kind,
  and a repository with no commits.
- Proof:
  both miners run on a disposable clone at the same revision with the same `--limit`,
  and the two outputs are compared byte for byte.
  The format stays in the repository,
  so a byte difference is a finding that the owner decides
  (regenerate `e2e/commit-shape-trace.json`,
  or keep the old writer's layout),
  not noise.
  The committed trace must also parse and re-serialize unchanged in Rust.

### The latency benchmark

- Creates:
  `cli.bench` and its program.
- Deletes:
  all of the group:
  15 files,
  3,989 lines
  (127 files and 30,047 lines remain).
- Negative controls:
  a planted defect in the pairing of direct and wrapped samples,
  a planted slow tail that moves the 95th percentile and the largest paired commit delta
  while leaving the median nearly unchanged,
  a scenario dropped from the selection,
  and a wrong unit.
- Proof:
  the statistics and the verdicts reproduce the recorded vectors and the recorded baseline scenarios of
  `bin/native-perf-judge.unit.test.ts`.
  The two programs then run on the same release binary and the same prepared fixture,
  alternating arms,
  and each program's run-to-run spread on unchanged input is measured first
  (rule `QNB`).
  The comparison checks the sample counts,
  the pairing,
  the scenario selection,
  the units and signs,
  the 95th percentile (the task's "added p95 under 2,000 ms"),
  and the largest paired commit delta (its "every paired delta under 5,000 ms"),
  not the median alone;
  a timing difference inside the measured spread counts as equal,
  and a count,
  pairing or verdict difference never does.
  The count of paired samples is taken from the TypeScript task (`--samples 30`),
  not chosen again.
  The host's load is whatever it is (rule `NQH`);
  the arms alternate so the load falls on both.

### The gate and its proof

- Creates:
  the image and gate runners,
  `run-the-tests-in-a-container`,
  and the gate proof writer and reader.
- Deletes:
  `bin/test-native-container.ts`,
  `bin/native-test-containerfile.ts`,
  `bin/native-test-snapshot.ts`,
  `bin/native-layered-build.ts`,
  `bin/native-test-settings-file.ts`
  and the tests of the layered build and of the wrapper snapshot,
  the second of which imports the snapshot modules this step and the next two steps delete
  (7 files,
  1,617 lines;
  120 files and 28,430 lines remain).
  The gate proof stays in TypeScript until the mutation step,
  because the TypeScript mutation entries read it.
- Negative controls:
  a failing test,
  a Clippy failure,
  a run in which an expected suite or test name is missing,
  a timeout,
  output past the capture limit,
  a command that fails to start,
  an interrupted run that must leave no container,
  and a path dependency the snapshot does not copy.
- Proof:
  the Containerfile text is generated by both from the same snapshot and compared line by line;
  both gates run on the same checkout (two real runs of about half an hour cold each,
  at the gate's 2 CPUs and 2 GiB, in their own units)
  and list the same tests and the same Clippy result
  (the nextest test names are the oracle,
  and the run counts only when the expected names appear,
  rule `TRV`);
  the records each writes are read by the other's reader,
  including a record for another image,
  a lower limit,
  a torn file and a changed digest;
  and each negative control fails both gates with a named reason.

### The concurrent-commit suite, scenario by scenario

- Creates,
  in this order inside `cli.e2e`:
  the pure models and checkers,
  the process,
  repository and service code,
  the hook and editor programs,
  the scenario model,
  the minimal Rust consumer and host runner
  and a task that runs the scenarios still in TypeScript through the TypeScript consumer
  and the others through the Rust consumer,
  failing when the union of names differs from the independent expected catalog or repeats a name.
  The consumer and the union task exist before the first scenario is deleted,
  so no deletion can make the suite greener by removing a scenario.
- Deletes,
  in this order:
  the 16 scenario definition files (4,468 lines in all),
  each when no remaining file imports it
  (removing its line from the TypeScript catalog is not enough:
  `amend-switch`, `foreign-unproven`, `hooked` and `lock-gc` import `concurrent-paths`,
  and `service-kill` imports `phase-kill`).
  The order the simulation computed,
  importers first:
  `amend-switch`,
  `baseline-amend`,
  `baseline-commit`,
  `baseline-editor`,
  `baseline-hook`,
  `foreign-unproven`,
  `hooked`,
  `index-writer`,
  `lock-gc`,
  `replay-degradation`,
  `service-kill`,
  `shared-file`,
  `sigkill`,
  `subsumption`,
  `concurrent-paths`,
  `phase-kill`.
  Then the rest of the suite with the entries:
  `bin/e2e-native-container.ts`,
  `e2e/concurrent-e2e-consumer.ts`,
  the pure and process code,
  the tests that go with them,
  `bin/native-test-base.ts` and its test,
  `bin/native-scanner-snapshot.ts` and `bin/native-e2e-snapshot.ts`.
  In all the step deletes 84 files and 19,422 lines
  (36 files and 9,008 lines remain);
  it is more than half of the TypeScript,
  which is why it is split by scenario.
- Negative controls:
  the suite's own positive-control scenario (`scenario-baseline-hook-fixture.ts`),
  a wrapper build with a guard removed (the planted controls' guard removals are the source of such builds),
  a catalog with a missing name,
  a catalog with a repeated name,
  a killed service that is never restarted,
  and a hook barrier that is never released.
- Proof,
  per scenario:
  the TypeScript consumer and the Rust consumer run the scenario with the same seed
  against the same image and the same Git,
  and the invariant verdict,
  the observation fields the invariants read and the scenario's outcome are compared;
  the TypeScript side comes from the reports captured for the whole corpus before any deletion.
  The seed makes the same random decisions in both (the same generator),
  and does not make the operating system schedule them the same way,
  so a scenario that depends on an interleaving is run in each runner more than once.
  A verdict,
  invariant,
  event or exit difference is a finding in either runner whatever the spread;
  only the duration the report prints is compared against the runners' measured spread.
  Last,
  the whole catalog runs through the Rust consumer over the seed corpus,
  and the report must show the same scenario names and verdicts as the captured TypeScript reports
  (36 scenarios passed and 2 skipped with seed 1 on 2026-10-14 is the only recorded run;
  the corpus is the choice "The seed corpus").

### The mutation campaign layer and the planted controls

- Waits for:
  the answer to "The shared mutation engine",
  and the wrapper's layout move.
- Creates:
  `cli.mutation` with its programs and the engine crate (or seam) the owner picked.
- Deletes:
  the mutation layer,
  the planted controls and the gate proof:
  32 files,
  8,333 lines
  (the layer 2,123 and the controls 5,205 of the groups,
  and the gate proof with its test,
  382 and 623,
  which the Rust gate has written and read since the gate step;
  4 files and 675 lines remain:
  the four modules the fuzz sidecar imports).
- Negative controls:
  a control whose guard is removed but whose test still passes (the guard removal must be reported unnoticed),
  a control whose named test is missing (the harness must fail and never count it caught),
  a planted source that does not compile,
  a record for another image,
  a record under lower limits,
  a torn record,
  and a changed control table or judge digest.
- Proof:
  the pure readers and writers reproduce the recorded shape fixtures,
  each controls table has the same entries and the same `from` text as the TypeScript table,
  and the test that every control's `from` text occurs exactly once and its test exists
  passes against the wrapper's sources.
  The readers are tested in both directions on every record kind with the compatibility stated in the design.
  Then one planted-controls run through each program on the same gate image
  (about 24 minutes each)
  must report the same noticed controls,
  and a blinded control (its test kept but its assertion disabled in a disposable copy, then its guard removed)
  must be reported unnoticed by both.
  Last,
  one scoped mutation campaign over one small source file through each program
  (`native:mutation:scoped -- --file <glob>`)
  must list the same mutants and give the same caught and missed counts,
  or the difference is recorded and explained.

### The fuzz sidecar's own TypeScript

- Applies only if the owner picks option A of "The fuzz sidecar's TypeScript".
- Creates:
  Rust programs in `package/git-policy/cli.fuzz` for its 10 TypeScript files (2,058 lines),
  using the command runner and snapshot modules from `cli.container-tests`,
  with `cli.container-tests` added to its `Cargo.toml`.
- Deletes:
  `package/git-policy/cli.fuzz/bin/*.ts`
  (outside the 145 files).
- Proof:
  the sidecar's `build`, `smoke` and `test:planted` tasks give the same results through both implementations
  on the same image,
  and its planted controls fail on their planted defects through both.

### The four modules the fuzz sidecar imports

- Waits for:
  the answer to "The fuzz sidecar's TypeScript"
  (option B copies the modules into the sidecar first;
  option C leaves this step undone).
- Deletes:
  `bin/native-verification-process.ts`,
  `bin/native-wrapper-snapshot.ts`,
  `bin/container-limits.ts`
  and `bin/native-nextest-snapshot.ts`
  (4 files,
  675 lines;
  nothing remains).
- Proof:
  `package/git-policy/cli.fuzz/bin` no longer names any file of `package/git-policy/cli/bin`,
  and the fuzz sidecar's tasks pass.

### The package's JavaScript scaffolding

- Deletes:
  `package/git-policy/cli/package.json`,
  `tsconfig.json`,
  the inline TypeScript tasks,
  the oxlint tasks (`lint:oxlint`, `lint:oxlint:paths`, `format:oxlint`, `format:oxlint:paths`),
  and the package's importer entry in `pnpm-lock.yaml`,
  regenerated by pnpm (rule `LFW`).
  `lint:types` and `test:unit` are retired or retargeted by the choice "Task names that go away".
  Under option C of "The fuzz sidecar's TypeScript",
  the four retained modules keep the package configuration they need.
- Fixes:
  the documents that name the files (the list in "What names the TypeScript and has to change with it").
- Proof:
  `rg` for every deleted path finds no remaining reference outside historical records,
  the package `lint` and each sidecar's `test` and `lint` tasks pass,
  and a before and after list of the package's mise task names accounts for every name:
  kept,
  retargeted or intentionally retired.

## Choices for the owner

Each choice is a decision only the owner can make;
the ranking is mine.
Two things are not choices and are answered apart:
whether the pause on the port is lifted
(accepting this plan does not lift it),
and the name and place of any crate the plan adds.

### The shared mutation engine

The mutation and planted-controls files import 34 symbols of
`@monochromatic-dev/mutation-test-cli-cargo-mutants`,
a TypeScript package of 111 files and 25,875 lines with five consumers besides this one.
The 34 fall into groups:
limit parsing (`diskWeightArguments`, `memoryBytes`, `runLimitArguments`),
constants (`attemptProtocolFile`, `containerStartedFile`, `startedMarkerProtocolVersion`, `unitIoWeight`),
the campaign flow (`runCampaign`, `campaignCommand`),
option parsing (`baselineSkipArguments`, `parseFileScopes`, `parseTimeoutAndExamine`, `parseLauncherOptions`),
test selection (`testScope`, `scopeFilterset`, `selectTestsArguments`),
parallel launching (`planUnits`, `startUnit`, `buildLogBase`, `buildShellBody`, `buildSystemdRunArguments`,
`buildUnitEnvironment`, `buildUnitName`, `formatWaitHint`),
and helpers (`isContentAddressedId`, `isMissing`, `isRecord`, `runCaptured`, `passOneDirectoryName`,
`CargoMutantsToolingError`),
plus types.
Whatever the owner picks decides whether the mutation step can be Rust at all.

- Option A, a Rust crate in the existing shared package folder:
  `package/mutation-test/cli-cargo-mutants` gains a `Cargo.toml` and a Rust crate beside its TypeScript,
  as `package/forbidden-strings/cli` already holds a `Cargo.toml`, a `package.json`
  and TypeScript `bin/` files together.
  Start with the 34 symbols this package uses,
  and let each other consumer move over when it ports.
  - Pros:
    keeps the owner's direction that product-neutral tooling lives in one package at its approved location
    (`doc/planning/shared-cargo-mutants-package.md`, lines 28 to 31, and the decisions of 2026-10-29);
    nothing is copied;
    ends Node in this package's mutation path.
  - Cons:
    one package holds two languages until the other consumers port;
    the crate needs a name and its interface must later serve the options the linter's campaigns use
    (`--jobserver-tasks`, `--caught`, noted on that page under Risks);
    the TypeScript engine and the Rust crate must agree on the record formats meanwhile.
  - Endpoint:
    this package has no TypeScript in the mutation path;
    the engine package keeps TypeScript for the other consumers.
    Blocks:
    the mutation step until the crate exists.
- Option D, the engine pieces inside `cli.mutation`:
  port the same 34 symbols as private modules of this package's sidecar.
  - Pros:
    nothing outside this package changes.
  - Cons:
    breaks the one-package direction,
    so the second product to port extracts them again;
    two engines must agree on the record formats while the others stay in TypeScript.
  - Endpoint:
    the same as option A for this package;
    the extraction is later work.
    Blocks:
    nothing outside this package.
- Option B, keep the TypeScript engine and call it from Rust:
  add command-line entries to the engine and have the Rust sidecar start them.
  The campaign flow and the parallel launch can be commands;
  limit parsing,
  constants,
  option parsing,
  test selection and the helpers are pure functions and constants that a Rust caller cannot import,
  so each is either copied into Rust or served by another command.
  - Pros:
    one engine for the flows,
    so the 43 golden files and the five other consumers stay valid.
  - Cons:
    adds new TypeScript and keeps Node in the path of the proof that gates every release;
    most of the 34 symbols still have to be copied or wrapped,
    so the seam is not the smallest port;
    the launcher's retry loop stays generated shell.
  - Endpoint:
    this package keeps Node and a TypeScript engine dependency for the mutation path.
    Blocks:
    the scaffolding step's removal of every `package.json` dependency on the engine.
- Option C, leave the layer and the controls in TypeScript until the engine's language is decided:
  - Pros:
    no risk to the campaigns now;
    the layout move can finish first.
  - Cons:
    30 files and 7,328 lines stay,
    plus the gate proof and its test,
    so the owner's "delete each TS part once ported" is not met for them.
  - Endpoint:
    this package keeps 32 files and 8,333 lines,
    with `package.json`, `tsconfig.json` and the template tasks they need.
    Blocks:
    the mutation step,
    the fuzz modules step and the scaffolding step.

Ranking: A > D > B > C,
because A puts the engine where the owner said product-neutral tooling lives
while D would leave a copy to extract later;
D > B because D removes TypeScript and Node from this package's mutation path now,
while B adds new TypeScript entries and still copies most of the symbols;
and B > C because B still ports and deletes this package's own 30 files,
while C ports nothing of them.

### Sidecar shape

- Option A, one sidecar per kind (`cli.container-tests`, `cli.mutation`, `cli.e2e`, `cli.bench`):
  - Pros:
    the tree and each `Cargo.toml` show who depends on whom;
    `cli.fuzz` is the precedent;
    each builds,
    lints and tests alone,
    so an end-to-end compile error never blocks the gate runner.
  - Cons:
    one lock file,
    one `mise.toml` and one `target` folder per sidecar,
    and the wrapper library is compiled by each sidecar that links it (the compile time was not measured);
    the shared runner code lives in the lowest sidecar until it moves.
- Option B, one `cli.verification` sidecar with every program:
  - Pros:
    one lock file,
    one build,
    folders and their `uses/` links show the dependencies inside one crate.
  - Cons:
    the name says little about what the contents do (rule `MKY`);
    whether a change to one program rebuilds the others depends on how the crate is split,
    and was not measured.
- Option C, targets inside the published crate:
  - Pros:
    no new package.
  - Cons:
    the crate cannot take a second `[[bin]]` (cited in "Design adopted for every group"),
    and a test target that starts containers would run inside the container gate that it starts.

Ranking: A > B > C,
because A keeps each concern in its own package with its own lint and test tasks and follows the `cli.fuzz` precedent,
while B trades that for one lock file;
and B > C because C is blocked by the binstall rule for programs and by recursion for container-starting tests.

### Where the shared command runner lives

The gate,
the mutation layer,
the suite,
the benchmark and the fuzz sidecar all start programs with a capture limit and the cleanup contract.

- Option A, the owner's library `starting-and-watching-programs`
  (one of the four libraries of the answer "Four libraries (Recommended)"):
  - Pros:
    the owner already chose this job as a library;
    every sidecar and the wrapper share one runner.
  - Cons:
    it does not exist until the layout move lifts it;
    until then another home is needed.
- Option B, the lowest sidecar's library (`cli.container-tests`), with the others depending on it:
  - Pros:
    available immediately;
    one runner for the sidecars.
  - Cons:
    `cli.bench` would depend on a container-focused sidecar for a command runner;
    the runner moves again when option A becomes possible.
- Option C, a small runner in each sidecar:
  - Pros:
    no coupling between sidecars.
  - Cons:
    the cleanup contract is copied into each,
    and each copy has to get it right.

Ranking: A > B > C,
because A is the owner's own home for this job while B only serves until A exists;
and B > C because C copies the cleanup contract that every copy must get right.
Until A exists,
B is used.

### Hook and editor programs

The end-to-end suite registers programs that Git runs as hooks and editors.

- Option A, separate helper executables in `cli.e2e`:
  - Pros:
    no Node in the end-to-end image;
    each executable has one job and a plain name.
  - Cons:
    the image build compiles more programs.
- Option B, one standalone helper executable with a mode word for each role:
  - Pros:
    one binary for all three roles;
    still a different executable from the runner,
    so Git starting it is not self-invocation.
  - Cons:
    a binary that does three jobs needs mode dispatch,
    and its name cannot say what it does.
- Option C, keep the Node scripts the harness writes to disk:
  - Pros:
    nothing to rewrite;
    Node stays in the image.
  - Cons:
    TypeScript-era text embedded in Rust as strings,
    and the image keeps `nodejs` only for them.
- Excluded:
  registering the scenario runner itself as the hook.
  The runner would start Git and Git would start the runner,
  the self-invocation the owner banned.

Ranking: A > B > C,
because A gives each role a plain name and one job
while B needs a mode word and a name that fits three jobs;
and B > C because B ends Node in the end-to-end image and leaves no embedded scripts.

### How the harness learns the wrapper's names

The suite and the benchmark need the command word,
the service words,
the service socket naming and the claim lease.

- Option A, link the wrapper library:
  `COMMAND_WORD`, `SERVICE_WORDS`, `socket_for` and `CLAIM_LEASE` are declared `pub` today.
  - Pros:
    the names are true by construction,
    and the guard tests that parse Rust source text go away.
  - Cons:
    each sidecar that links it compiles it (not measured),
    and each wrapper version bump touches the exact pin
    (the catalog sync writes it, and the release bumps it where `releasable-crates.json` lists it).
- Option C, copy the constants and test that they equal the library's:
  - Pros:
    no production dependency on the wrapper library.
  - Cons:
    the test still links the library,
    and the copy can drift between a bump and its test.
- Option B, read the Rust source text as the TypeScript does:
  - Pros:
    no build coupling.
  - Cons:
    a text parser of Rust declarations in a test,
    the most fragile of the three.

Ranking: A > C > B,
because A makes disagreement impossible while C only detects it;
and C > B because C compares values while B parses declarations.

### The seeded generator

- Option A, keep Mulberry32 with FNV-1a seed derivation exactly:
  - Pros:
    seeds printed by past runs replay,
    and the TypeScript and Rust runners make the same random decisions from one seed.
  - Cons:
    keeps a small hand-written generator.
- Option B, a published generator:
  - Pros:
    no hand-written generator.
  - Cons:
    unless it yields exactly this sequence (whether any published crate does was not checked),
    every old seed changes meaning and the two runners no longer share decisions.

Ranking: A > B,
because A keeps the replay and the common decisions while B gives them up for a dependency.
After the TypeScript is deleted,
B stays open as a later change.

### The fuzz sidecar's TypeScript

Four modules of this package are imported by `package/git-policy/cli.fuzz/bin`.

- Option A, port the fuzz sidecar's 10 TypeScript files in the same program:
  - Pros:
    the four modules can go,
    and the Rust modules have a second user from the start.
  - Cons:
    it is another package's work,
    2,058 lines more.
  - Endpoint:
    both packages have no TypeScript in their verification tooling.
    Blocks:
    nothing.
- Option C, leave the four modules (675 lines) in `cli/bin` until the fuzz sidecar ports:
  - Pros:
    no copy,
    and no scope growth.
  - Cons:
    this package keeps 4 files and 675 lines of TypeScript for as long as the fuzz sidecar is TypeScript.
  - Endpoint:
    this package keeps those files and the package configuration they need.
    Blocks:
    the fuzz modules step and the scaffolding step.
- Option B, give the fuzz sidecar its own copies of the four modules:
  - Pros:
    this package finishes at zero TypeScript.
  - Cons:
    duplicates 675 lines that must then be ported twice.
  - Endpoint:
    this package has none;
    the fuzz sidecar holds copies.
    Blocks:
    nothing in this package.

Ranking: A > C > B,
because A finishes both packages while C leaves a bounded residue;
and C > B because C keeps one copy while B creates two to port.

### Fuzzing the ported readers

The quoted completion requirement (`doc/decision/cli-git-rust-rewrite.md`, lines 24 to 26)
was written for the wrapper's rewrite.
Whether it reaches verification tooling is not recorded.
The port creates readers of persistent formats (gate and controls records, the commit-shape trace,
the JSONL policy events, the Git log parser).

- Option A, fuzz targets for the readers in the existing `package/git-policy/cli.fuzz`:
  - Pros:
    follows the repository's practice for parsers and the quoted requirement;
    coverage-guided.
  - Cons:
    `cli.fuzz` would depend on the sidecars' libraries,
    and its container smoke run grows.
- Option B, property tests with generated malformed inputs under nextest in each sidecar:
  - Pros:
    runs in every test run;
    bounded and cheap.
  - Cons:
    not coverage-guided.
- Option C, none,
  on the reading that the requirement covers the wrapper only:
  - Pros:
    no extra work.
  - Cons:
    no evidence that a malformed record never panics or hangs a runner.

Ranking: A > B > C,
because A buys coverage-guided evidence on the same terms the repository holds its other parsers to
while B buys only generated cases;
and B > C because B at least shows malformed input is rejected without a panic or a hang.

### The seed corpus

- Option A, a committed corpus of seeds chosen before the first scenario is deleted,
  with the TypeScript reports for the whole corpus captured then:
  - Pros:
    the comparison has a fixed oracle that outlives the TypeScript.
  - Cons:
    the owner must set the number of seeds,
    and each seed is a real run of the suite.
- Option B, seed 1 only (the one recorded run):
  - Pros:
    smallest cost.
  - Cons:
    one seed exercises one set of random decisions.
- Option C, a fresh random seed for each comparison:
  - Pros:
    widest coverage over time.
  - Cons:
    no stable oracle,
    so a difference cannot be told from a new seed.

Ranking: A > B > C,
because A fixes the oracle and widens coverage beyond one seed;
and B > C because B at least has a recorded result to compare against.

### Flag parsing

- Option A, `clap`:
  - Pros:
    already a catalog entry;
    generated usage text and exit code 2 on a usage error match the exit-code decision.
  - Cons:
    a dependency in each sidecar.
- Option B, the wrapper's hand-written parser:
  - Pros:
    no dependency.
  - Cons:
    repeats the parsing and usage text in each program.

Ranking: A > B,
because A gives the decided exit code and usage text for free
while B repeats that work in every program.

### Task names that go away

The package's `lint:types`,
`test:unit`,
`lint:oxlint`,
`lint:oxlint:paths`,
`format:oxlint` and `format:oxlint:paths` tasks run TypeScript tools.
The four oxlint tasks have nothing left to lint in either option.

- Option A, retarget `lint:types` and `test:unit`:
  `lint:types` runs `cargo check` and Clippy of the sidecars (the name `cli.fuzz` already gives its Cargo check),
  and `test:unit` runs their nextest tests.
  - Pros:
    every caller keeps working.
  - Cons:
    a name that once meant TypeScript now means Rust.
- Option B, retire all six names:
  - Pros:
    every name says what it runs.
  - Cons:
    breaks callers of the two names for no new capability.

Ranking: A > B,
because A keeps every caller working under the precedent `cli.fuzz` set
while B breaks callers for a cleaner name.

## Risks

### The wrapper's layout move renames what the planted controls name

The tables name wrapper source paths and test names that issue 6261 moves,
and the layout plan says the tooling is updated in the same commit as each move.
The mutation step comes late for this reason;
porting it before the move would write each table twice.
The check that a moved line still exists (`bin/native-planted-controls-proof.ts`, today)
must be kept and must fail on a path that no longer exists.

### A rule says scripts are TypeScript

Rules `SCR` and `NMJ` say scripts are `.ts` files.
The owner's port is a decision to the contrary for this package,
so after it the package's tooling is Rust programs run by `mise run` tasks,
and `CLAUDE.md` still tells agents to write scripts in TypeScript.
Whether to word an exception is the owner's;
`AGENTS.md` is not edited here.

### Heavy proof runs on a loaded host

The gate,
the planted controls and the end-to-end suite are real jobs of tens of minutes.
Each runs once per implementation in its own unit with bounds,
never repeated to make load,
and the spread is measured on unchanged input before any timing comparison (rules `NQH`, `QNB`).

### A passing Rust test that cannot fail

A ported test passes against code that is simply missing the check.
Rules `GFP`,
`QPC` and `TRV` are the answer:
each ported guard is shown to fail on a planted defect before its TypeScript counterpart is deleted,
the guard is separated from its test so removing its enforcement is what the proof does,
and the nextest output must hold the expected names.

### A runner has timing the event loop hid

The scenarios pin interleavings with barriers and markers,
and a multi-thread runtime schedules differently from Node's one thread.
A scenario that passes in one runner and fails in the other is a finding about a race,
whatever the timing spread,
and only printed durations are compared against a measured spread.

### Other agents edit the TypeScript while it is ported

The files in `bin/` and `e2e/` have been edited by other work
(for example the program rename touched two TypeScript tests).
The port freezes a file's behavior at its capture fixtures;
a TypeScript edit after capture must be mirrored in the Rust before that file is deleted,
and the coordinator should route TypeScript edits to the porting branch.

### The fuzz sidecar and the shared engine still import what is ported

The four modules and the engine's golden files tie the deletion order to other packages
(see the choices).
Deleting a file early breaks `package/git-policy/cli.fuzz/bin`.

### The release bumps exact pins only where it is told

`native:release` bumps the crate and the fuzz crate's exact pin
(the task's description in `package/git-policy/cli/mise.toml`),
and `releasable-crates.json` lists the pin files and lock folders.
A sidecar that pins the wrapper and is not listed there breaks the next release.
Adding each sidecar to that list is part of its skeleton.

### The images change

The end-to-end image stops installing `nodejs` and compiles more programs;
the gate image keeps `nodejs` and `pnpm`.
Both image tags are functions of their Containerfile text,
so every change is a new tag;
how much of a rebuild the layered build reuses was not measured.
The base stays the newest CachyOS `latest` image the host CPU runs (rule `BXD`).

### Removing `package.json` changes the pnpm lock

The importer entry of `package/git-policy/cli` in `pnpm-lock.yaml` goes with it,
and the lock is regenerated by pnpm,
never edited by hand (rule `LFW`).
This is the last step so no other step needs the package to be a workspace member.

### Compile cost and target folders are not measured

Each sidecar has its own `target` folder,
and the ones that link the wrapper library compile it.
The owner wants "One shared build folder" only after confirming each package stands on its own
(`doc/planning/cli-git-source-layout-target-tree.md`, lines 110 to 114),
so the cost is a measurement the first skeleton step must take with `cargo check` before the later steps are scheduled.

### Stale documents name the TypeScript

The list in "What names the TypeScript and has to change with it" is the work of the last step,
and each deletion commit updates the documents it breaks.

## Deletion manifest by step

The manifest is generated by `${HOME}/temp/agent/ts-to-rust/simulate-v2.ts`
and lists every file at the step that deletes it,
with the four modules the fuzz sidecar imports kept until their own step.
A test appears at the earliest step that deletes a file it imports,
and that step is the latest by which its Rust replacement must exist.

### Deleted by: The release guard

This step deletes 1 file, 241 lines;
144 files remain, 34,500 lines.

- `bin/native-release-bins.unit.test.ts` (241 lines, a test)

### Deleted by: The base image build and the plumbing it needs

This step deletes 1 file, 151 lines;
143 files remain, 34,349 lines.

- `bin/build-git-test-image.ts` (151 lines)

### Deleted by: The commit-shape trace miner

This step deletes 1 file, 313 lines;
142 files remain, 34,036 lines.

- `e2e/commit-shape-trace-generator.ts` (313 lines)

### Deleted by: The latency benchmark

This step deletes 15 files, 3,989 lines;
127 files remain, 30,047 lines.

- `bin/native-perf-evidence.ts` (137 lines)
- `bin/native-perf-git.ts` (402 lines)
- `bin/native-perf-judge.ts` (92 lines)
- `bin/native-perf-judge.unit.test.ts` (333 lines, a test)
- `bin/native-perf-lines.ts` (75 lines)
- `bin/native-perf-measure.ts` (357 lines)
- `bin/native-perf-root.ts` (99 lines)
- `bin/native-perf-scenarios.ts` (38 lines)
- `bin/native-perf-scratch.ts` (375 lines)
- `bin/native-perf-scratch.unit.test.ts` (225 lines, a test)
- `bin/native-perf-service.ts` (454 lines)
- `bin/native-perf-service.unit.test.ts` (410 lines, a test)
- `bin/native-perf-shape.ts` (274 lines)
- `bin/native-perf-statistics.ts` (250 lines)
- `bin/native-perf.ts` (468 lines)

### Deleted by: The gate and its proof

This step deletes 7 files, 1,617 lines;
120 files remain, 28,430 lines.

- `bin/native-layered-build.ts` (411 lines)
- `bin/native-layered-build.unit.test.ts` (486 lines, a test)
- `bin/native-test-containerfile.ts` (138 lines)
- `bin/native-test-settings-file.ts` (65 lines)
- `bin/native-test-snapshot.ts` (53 lines)
- `bin/native-wrapper-snapshot.unit.test.ts` (208 lines, a test)
- `bin/test-native-container.ts` (256 lines)

### Deleted by: The concurrent-commit suite, scenario by scenario

This step deletes 84 files, 19,422 lines;
36 files remain, 9,008 lines.

- `bin/e2e-native-container.ts` (291 lines)
- `bin/native-e2e-snapshot.ts` (21 lines)
- `bin/native-scanner-snapshot.ts` (109 lines)
- `bin/native-test-base.ts` (495 lines)
- `bin/native-test-base.unit.test.ts` (121 lines, a test)
- `e2e/barrier-fixture.ts` (163 lines)
- `e2e/client-connection-fixture.ts` (138 lines)
- `e2e/commit-shape-log-fixture.ts` (368 lines)
- `e2e/commit-shape-trace-fixture.ts` (625 lines)
- `e2e/commit-shape-trace.unit.test.ts` (269 lines, a test)
- `e2e/concurrent-e2e-consumer.ts` (312 lines)
- `e2e/content-fixture.ts` (299 lines)
- `e2e/editor-attempt-fixture.ts` (44 lines)
- `e2e/foreign-commit-fixture.ts` (110 lines)
- `e2e/git-read-fixture.ts` (415 lines)
- `e2e/git-version-fixture.ts` (102 lines)
- `e2e/hook-program-fixture.ts` (305 lines)
- `e2e/hooks-ignored-fixture.ts` (129 lines)
- `e2e/invariant-fixture.ts` (421 lines)
- `e2e/invariant-model-fixture.ts` (277 lines)
- `e2e/invariant.unit.test.ts` (230 lines, a test)
- `e2e/jsonl-event-fixture.ts` (256 lines)
- `e2e/jsonl-event.unit.test.ts` (113 lines, a test)
- `e2e/landed-commit-fixture.ts` (42 lines)
- `e2e/landing-kill-fixture.ts` (57 lines)
- `e2e/ledger-fixture.ts` (288 lines)
- `e2e/ledger.unit.test.ts` (123 lines, a test)
- `e2e/leftover-fixture.ts` (105 lines)
- `e2e/leftover.unit.test.ts` (105 lines, a test)
- `e2e/observation-attempt-fixture.ts` (514 lines)
- `e2e/observation-fixture.ts` (464 lines)
- `e2e/process-fixture.ts` (481 lines)
- `e2e/process-fixture.unit.test.ts` (40 lines, a test)
- `e2e/push-notice-fixture.ts` (74 lines)
- `e2e/push-rejection-fixture.ts` (111 lines)
- `e2e/push-rejection.unit.test.ts` (100 lines, a test)
- `e2e/real-git-fixture.ts` (94 lines)
- `e2e/report-fixture.ts` (142 lines)
- `e2e/repository-fixture.ts` (256 lines)
- `e2e/repository-model-fixture.ts` (125 lines)
- `e2e/repository-setup-fixture.ts` (285 lines)
- `e2e/scenario-amend-switch-fixture.ts` (291 lines)
- `e2e/scenario-baseline-amend-fixture.ts` (147 lines)
- `e2e/scenario-baseline-commit-fixture.ts` (291 lines)
- `e2e/scenario-baseline-editor-fixture.ts` (244 lines)
- `e2e/scenario-baseline-hook-fixture.ts` (285 lines)
- `e2e/scenario-catalog-fixture.ts` (50 lines)
- `e2e/scenario-claim-recovery-fixture.ts` (216 lines)
- `e2e/scenario-concurrent-paths-fixture.ts` (145 lines)
- `e2e/scenario-expectation-fixture.ts` (163 lines)
- `e2e/scenario-foreign-unproven-fixture.ts` (210 lines)
- `e2e/scenario-helper-fixture.ts` (390 lines)
- `e2e/scenario-hooked-fixture.ts` (248 lines)
- `e2e/scenario-index-writer-fixture.ts` (327 lines)
- `e2e/scenario-lock-gc-fixture.ts` (267 lines)
- `e2e/scenario-model-fixture.ts` (535 lines)
- `e2e/scenario-model.unit.test.ts` (155 lines, a test)
- `e2e/scenario-outcome-fixture.ts` (52 lines)
- `e2e/scenario-phase-kill-fixture.ts` (274 lines)
- `e2e/scenario-queue-fixture.ts` (123 lines)
- `e2e/scenario-replay-degradation-fixture.ts` (295 lines)
- `e2e/scenario-service-kill-fixture.ts` (405 lines)
- `e2e/scenario-setup-fixture.ts` (300 lines)
- `e2e/scenario-shared-file-fixture.ts` (320 lines)
- `e2e/scenario-sigkill-fixture.ts` (468 lines)
- `e2e/scenario-subsumption-fixture.ts` (251 lines)
- `e2e/seeded-random-fixture.ts` (350 lines)
- `e2e/seeded-schedule-fixture.ts` (208 lines)
- `e2e/seeded-schedule.unit.test.ts` (221 lines, a test)
- `e2e/service-connection-fixture.ts` (78 lines)
- `e2e/service-fixture.ts` (341 lines)
- `e2e/service-kill-lease.unit.test.ts` (110 lines, a test)
- `e2e/service-launch-fixture.ts` (192 lines)
- `e2e/service-launch.unit.test.ts` (55 lines, a test)
- `e2e/service-mode-words.unit.test.ts` (113 lines, a test)
- `e2e/service-model-fixture.ts` (63 lines)
- `e2e/service-settle-fixture.ts` (161 lines)
- `e2e/trace-execution-fixture.ts` (482 lines)
- `e2e/trace-replay-fixture.ts` (266 lines)
- `e2e/trace-replay.unit.test.ts` (225 lines, a test)
- `e2e/work-record-fixture.ts` (328 lines)
- `e2e/worker-fixture.ts` (476 lines)
- `e2e/wrapper-selection-fixture.ts` (196 lines)
- `e2e/wrapper-selection.unit.test.ts` (86 lines, a test)

### Deleted by: The mutation campaign layer and the planted controls

This step deletes 32 files, 8,333 lines;
4 files remain, 675 lines.

- `bin/mutate-native-container.ts` (304 lines)
- `bin/native-gate-proof.ts` (382 lines)
- `bin/native-gate-proof.unit.test.ts` (623 lines, a test)
- `bin/native-mutation-options.ts` (111 lines)
- `bin/native-mutation-parallel.ts` (130 lines)
- `bin/native-pass-one.ts` (105 lines)
- `bin/native-pass-one.unit.test.ts` (171 lines, a test)
- `bin/native-planted-controls-commit.ts` (154 lines)
- `bin/native-planted-controls-ensure.ts` (216 lines)
- `bin/native-planted-controls-ensure.unit.test.ts` (331 lines, a test)
- `bin/native-planted-controls-entry.ts` (107 lines)
- `bin/native-planted-controls-harness.ts` (76 lines)
- `bin/native-planted-controls-harness.unit.test.ts` (110 lines, a test)
- `bin/native-planted-controls-key-parts.ts` (45 lines)
- `bin/native-planted-controls-key-parts.unit.test.ts` (61 lines, a test)
- `bin/native-planted-controls-landing.ts` (119 lines)
- `bin/native-planted-controls-manifest.ts` (503 lines)
- `bin/native-planted-controls-obligation.ts` (398 lines)
- `bin/native-planted-controls-obligation.unit.test.ts` (685 lines, a test)
- `bin/native-planted-controls-only.ts` (139 lines)
- `bin/native-planted-controls-outcome.ts` (136 lines)
- `bin/native-planted-controls-outcome.unit.test.ts` (145 lines, a test)
- `bin/native-planted-controls-proof.ts` (277 lines)
- `bin/native-planted-controls-proof.unit.test.ts` (810 lines, a test)
- `bin/native-planted-controls-reconcile.ts` (55 lines)
- `bin/native-planted-controls-record.ts` (229 lines)
- `bin/native-planted-controls-unit.ts` (105 lines)
- `bin/native-planted-controls-units.ts` (134 lines)
- `bin/native-planted-controls.ts` (303 lines)
- `bin/native-planted-controls.unit.test.ts` (67 lines, a test)
- `bin/native-planted-parallel.ts` (537 lines)
- `bin/native-planted-parallel.unit.test.ts` (765 lines, a test)

### Deleted by: The four modules the fuzz sidecar imports

This step deletes 4 files, 675 lines;
0 files remain, 0 lines.

- `bin/container-limits.ts` (8 lines)
- `bin/native-nextest-snapshot.ts` (169 lines)
- `bin/native-verification-process.ts` (273 lines)
- `bin/native-wrapper-snapshot.ts` (225 lines)
