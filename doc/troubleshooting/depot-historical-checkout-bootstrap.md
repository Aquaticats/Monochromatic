# Historical checkout and host preparation fail before Depot checks

## Scope

These incidents affect the isolated affected-CI harness,
not a proposed production migration.
The TOML source is `525d2e09f8168a3bdd7e513e26d4c80b363123e5`.
The native source is `cb95cf6d785c3073c192b95f0632cdae7b910894`.
Both use mise action version `2026.10.9` and separate harness/source checkouts.

## Missing historical JavaScript entry

### Symptom and source

Run `zn4q03mh59` failed before selected checks with Node's `MODULE_NOT_FOUND`
for `source/node_modules/@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs`.

At the TOML revision,
`package/git-policy/cli/package.json:17` declares:

```json
// package/git-policy/cli/package.json at 525d2e09f8
"bin": {
  "git": "dist/final/node/index.mjs"
}
```

Frozen dependency installation creates the workspace executable link;
it does not build the linked package's missing output.
The executor's initial Git identity read therefore starts a missing program.

### Verified correction

The harness runs the existing `//package/git-policy/cli:build` task before Git commands.
It then uses the historical `git cli-git trust --yes` command for the reviewed TypeScript policy.
Run `hgqj2nstwq` reached all selected core checks after this preparation.
The package build and policy trust are setup costs,
not TOML test coverage.

## Native revision needs its published policy tool

### Symptom and source

Run `5tbb7qftkw` copied the TOML bootstrap into the newer native case.
Both core and Android jobs stopped at:

```text
error: no matching package named `ignore` found
location searched: crates.io index
required by package `monochromatic-git-policy-cli v0.1.2`
```

At the native revision,
`package/git-policy/cli/mise.toml` defines its build as:

```toml
# package/git-policy/cli/mise.toml at cb95cf6d78
[tasks.build]
run = "cargo build --release --locked --offline --bin cli-git-native"
```

The native package's `package.json` has no executable mapping.
The source `mise.no-env.toml:129` instead installs `cargo:monochromatic-git-policy-cli`
and links its `cli-git-native` binary as `.cache/cli-git-bin/git`.
The source `package/git-policy/cli/README.md:135` says the trust commands are retired:

```text
The former `trust`,
 `untrust` and `status` commands are retired.
Configuration is data,
 so nothing needs approval.
```

The old assumption that every historical checkout needed the JavaScript wrapper build was wrong.
This checkout requires its locked published native executable.
An unrelated offline build adds a Cargo-cache requirement absent from the intended native case.

### Correction under verification

Harness commit `a24cfd43a7` installs the source lock's published policy version,
creates the authored per-checkout executable link,
and runs real Git version/help commands before selected checks.
It omits the unrelated wrapper source build and retired trust command.
Runs `j80v9dqfb4` and `xg3znh2j0p` exercise this correction.
The desktop attempt passed this setup and reached its separate Clang host probe.

Installing Rust and `cargo-binstall` precedes dependent Cargo tools;
see [Cargo backend prerequisites](mise-cargo-backend-needs-rust-installed.md).

## Immutable lockfile and policy-fixture boundaries

Earlier attempts `zgsc58pmfc` and `zxdc83cpbp` failed the executor's
`mise.lock changed during setup` assertion or the historical policy's `config-untrusted` guard.
The exact source input comparison prevented those attempts being counted as valid check runs.

Source commands now use `MISE_LOCKED=1`.
Tool installation occurs outside source using explicit versions read from its lockfile.
The action's implicit environment activation is disabled using `env: false`
and `export_path: false`;
each task activates its own configured environment.
The complete lockfile rewrite has not yet been retained as a diff,
so its individual metadata changes remain unclassified.

The generated-file fixture is outside the original trusted checkout's directory.
It therefore requires its own explicit trust operation after building the historical wrapper.
Trusting only the source root does not establish trust for a sibling fixture.
See [historical trust snapshots](cli-git-trusted-snapshot-stale-policy-code.md).

## Generated-file fixture tries an obsolete LFS endpoint

Run `d1jgjp62lg`,
attempt `3xh4qrphll`,
failed during `git worktree add`:

```text
Error downloading object: package-deprecated/audit/oph-common-look-and-feel/screenshot/contact-page.png
lookup monochromatic-lfs.an1298.workers.dev on 8.8.8.8:53: no such host
fatal: ... smudge filter lfs failed
```

No generator consistency result was produced.
The initial action checkout retains LFS pointers,
but the later worktree checkout invoked the globally configured smudge filter.
The fixture must retain the same pointer representation as its source checkout.
Commit `43c0fb4e8f` sets `GIT_LFS_SKIP_SMUDGE=1` for that fixture operation.
`git lfs help smudge` documents that variable as disabling smudging.
This preserves pointer files and does not validate the archived image payloads.
The generator task does not select those archived screenshots as its input surface.
The obsolete endpoint itself is a separate unresolved configuration problem.

## Local patch application fails before source setup

Feature run `mr9cw18gds` checked out harness base `0886b2669d`,
then Depot's injected patch step failed:

```text
error: patch failed: package/monorepo-task/core-exec/mise.toml:140
error: package/monorepo-task/core-exec/mise.toml: patch does not apply
```

The launch used local head `2288fcec22` and uploaded a 12579-byte patch
because the remote branch still pointed at the older base.
The log identifies the checkout and patch failure;
it does not establish why the hunk failed.
No historical source check executed.
The next dispatch explicitly waits for `git push` to finish and verifies a zero-patch launch.
This avoids this patch transport boundary without changing source inputs.
The first replay remains under verification.

TOML full attempt `gfklnntq0x` reproduced the patch boundary on 2026-10-16:
all jobs checked out `3b3be538e7`,
then failed on a missing `.depot/workflows/depot-complete-logger-remainder.yml`
and an unapplied `package/logger/core.fuzz/mise.toml` hunk.
The provider logs contain the patch diagnostic and no source-check artifact.
The October 17 retry uses independently pushed branch `trial/depot-final-recovery-20261017`
at `409e6ec106`,
with clean local status and a matching remote SHA measured before dispatch.

## Linux completion replay omits desktop tool selection

Run `pbg4h8ww6k` reached host preparation,
then Node reported `spawnSync slint-viewer ENOENT` during `--version`.
No nextest cases ran and no check artifact was produced.
The workflow had `DEPOT_CI_CASE=desktop` but no `DEPOT_CI_WAVE=native-desktop`.
The harness selects the Slint tool only for that wave.
This differs from an installed Slint executable missing `libinput.so.10`.

Trial `409e6ec106` adds the desktop wave before exact source tool installation.
The retry must verify installation,
the executable's actual version,
and the complete test identities before the coverage gap can close.

## Slint executable lacks a runtime library

Native desktop run `w6ln4j09fz` passed the product build,
Clippy,
and 1406 nextest cases.
Its Slint check failed at executable loading:

```text
slint-viewer: error while loading shared libraries: libinput.so.10: cannot open shared object file: No such file or directory
```

The prepared Ubuntu image lacked that runtime dependency.
The next host preparation installs Ubuntu's `libinput10` package and invokes
the source-selected `slint-viewer --version` before package checks.
The dependency correction is under live verification.

## Desktop control requests lose standard input

The same run started its private systemd user manager and PipeWire server.
Its first control-socket attempts failed before the socket existed;
later attempts exited zero with empty output.
The harness finally emitted `Parent compositor must publish a real Wayland display before E2E starts`.
No authored desktop runtime session executed.
Container removal and overlay isolation checks passed.

The wrapper supplied Node's subprocess `input`,
but invoked `podman exec` without its interactive standard-input flag.
The reviewed wrapper now uses `podman exec --interactive` whenever input is supplied.
A `cat` positive control must echo the exact payload before compositor queries run.
The complete desktop replay is under verification;
the empty output is not evidence that the compositor cannot publish a display.

## Desktop application cannot create a thread

Runtime-log recovery `km2xkxwpjs` at desktop source `3434eaa455` and `63r0kzrsmn` at native source `cb95cf6d78`
both retain the first wide session's `wide.log`.
The application panics in Rust's `std::thread::spawn` with:

```text
failed to spawn thread: Os { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }
```

The log then reports hosted-client exit code 101.
Preceding EGL diagnostics did not stop startup:
the application initialized its analysis,
watch,
and measurement paths before the panic.
Those EGL messages do not establish the panic's cause.

The trial's private systemd container declares two CPUs,
2 GiB memory,
and a 512-task PID limit.
The initial failing attempts did not retain live cgroup counters or process limits.
Subsequent runtime-only diagnostics `fs5t4ns6j6` and `tlc73pbmkk`
retain `pids.current`,
`pids.max`,
`pids.events`,
`memory.events`,
and `/proc/self/limits` before and after the actual consumer task.
The probe runs inside the disposable container and traverses bounded cgroup scope.
Both consumers increased PID-controller rejection counters from zero to four inside the private user hierarchy.
Their memory counters recorded no OOM event.
The measured user slice limit was 168 tasks,
and the measured default service limit was 76 tasks,
inside the declared 512-task container.
The hierarchy therefore has tighter limits than the outer container budget.
Because `pids.events` aggregates descendant events,
these samples do not establish which individual child limit rejected the thread.
The earlier hypothesis that the outer 512-task limit itself was exhausted remains unproven.

The trial helper at harness `4c4063e78bc9aaed8e293c1b478db089d6eef0ce`
writes overrides only inside the disposable rootfs overlay,
before starting the private user manager:

```toml
# Private /etc/systemd/user.conf.d/50-depot-trial.conf
[Manager]
DefaultTasksMax=256
```

```toml
# Private /etc/systemd/system/user-<source-uid>.slice.d/50-depot-trial.conf
[Slice]
TasksMax=448
```

The helper reloads the private system manager and asserts both effective limits before running E2E.
It retains the outer two-CPU,
2 GiB,
512-task boundary and adds local PID-event and peak counters to subsequent samples.
The actual unchanged consumer reruns are `v0xj2z7q66` for native source `cb95cf6d78`
and `4xgjgr8gbr` for desktop source `3434eaa455`.
Both runs crossed the original thread-creation boundary and verified cleanup.
All retained samples,
including the outer cgroup before and after the consumer,
show zero PID rejections and zero OOM events.
This verifies the private-limit repair at the original failing boundary.
Each consumer emitted 60 passing,
nine failing,
and one skipped wide-session assertions.
An uncaught `CheckFailureError` while waiting for a folder containing 30 tracks then ended the task.
The remaining configured sessions did not execute and the E2E summary file was not produced.
The evidence record retains every emitted assertion identity,
counter sample,
artifact digest,
and provider allocation interval.
At this checkpoint,
issue [#1110](https://github.com/Aquaticats/Monochromatic/issues/1110) remained open for full independent coverage.
The controls in [complete independent-session verification](#complete-independent-session-verification)
now satisfy that acceptance boundary.

These interaction failures remain a separate boundary from the repaired PID limit.
The harness had overridden `E2E_COMPOSITOR` with a binary built from the historical source checkout.
The authored task normally selects the published compositor declared in that source's `mise.lock`.
Both declare version `0.6.0`,
which does not establish that their executable bytes or implementation match.
Harness `9f4a01f699f09abb25f10258455ac0faed0ecae8`
installs the locked published tool,
retains its ELF SHA-256 and actual version output,
and clears the inherited override before running the unchanged authored task.
The source-built software parent and resource bounds remain the same.
Controls `xdlnfpwhzz` and `7x5lk9dx9q` completed for the native and desktop historical sources respectively.
Their retained results and subsequent headless controls are recorded in
[the display geometry diagnosis](depot-desktop-e2e-display-geometry.md).
Executable selection alone does not establish the cause of an individual interaction failure.

Without trial continuation,
configured sessions after the first failing session remain unexecuted.
The `runtimeCompleted` field means the invocation and cleanup returned,
not that every configured session passed or executed.

### Complete independent-session verification

Development controls `w21t2hzv67` and `d83q9lrgfx` at harness
`2294b7f1171c2cd88d0ca50dd148404a3a0db4b6` are terminal and fully collected.
The trial's explicitly recorded in-memory orchestration hook preserves ordinary session failures
and continues independent sessions.
Safety,
unknown,
and cleanup errors remain fatal.
Historical source and assertion files on disk remain unchanged.

Actual committed selectors yield six native and seven desktop sessions,
including desktop `auto-play`.
Every configured session has retained raw logs and actual assertion outcomes.
Effective private limits are verified before invocation;
all retained before/after PID-rejection and OOM counters remain zero.
The helper verifies restored private audio sinks,
no active test units,
no application process,
and removal of the private container and overlay writes.
Issue `#1110` is closed on this complete independent-session evidence.

Native reports 106 passes,
one original chooser failure,
one authored skip,
and a separate synthetic failure retaining the wide-session folder setup error.
Desktop reports 110 passes with the same failure and skip classifications.
The source inventory still has six original wide assertions unexecuted per case.
Source issues `#1004` and `#1005` remain open,
as does production orchestration issue `#1006`.
These development controls do not establish a passing source gate or final coherent replay.

### Completed session logs disappear before collection

The original `package/music-player/desktop-e2e/src/run.ts` writes the summary,
then removes its scratch base.
`package/music-player/desktop-e2e/src/session.ts`
writes each session's stdout/stderr as a log inside that base.
The trial's post-command collector therefore copies the directory after those completed-run logs have disappeared.
Historical continuation controls `s3cmrhf2kp` and `wxq11xm2hd` retain summaries and screenshots,
but have no raw session logs.
This retention boundary is separate from aborted-session coverage and the TOML active-evidence loss.

Trial implementation `b6b44d679853c29e39a3c801b64ca66a32d30a97`
extends the existing source-hash-checked loader hook.
It copies only regular top-level `*.log` files to a private archive outside the source scratch base,
checks copied byte counts and SHA256,
and writes an archive manifest.
The original source removal stays in `finally`,
including archive-copy failure paths.
A post-run probe rejects missing or corrupt archives and verifies that the source scratch base is absent.

The actual generated hook passed 16 archive controls across the source cases,
including quoted/newline paths,
an existing destination,
symlinks,
an archive nested inside the source base,
and missing or duplicate source anchors.
Removing the original cleanup lets the module return successfully,
but the actual post-run probe fails.
The updated hook also passed 17 continuation/safety controls per source case.

The actual provider controls retain 16 native logs totaling 2,022,505 bytes
and 17 desktop logs totaling 2,128,752 bytes.
Every downloaded file matches its manifest's byte count and SHA256.
The provider archive event matches that manifest,
and the post-run probe confirms original scratch removal.
Every raw log records the actual headless software-compositor invocation.
The desktop archive includes `auto-play.log` and its four original assertion outcomes.
Issues `#1133` for retention and `#1120` for source-selected inventory are closed.
The `desktopRuntimeSessionLogArchiveControl` field in the final recovery evidence retains the complete proof.

## Source identity input coverage

The helper-specific incident is tracked in
[#1114](https://github.com/Aquaticats/Monochromatic/issues/1114),
closed after the metadata/path and independent index-invariant controls met its acceptance criteria.
The issue keeps the original gap,
both repair stages,
the indexed-exclusion tradeoff,
and unchanged historical-source bytes distinct.
Its body and `bug` label were fresh-read before closing;
the final read verified its body,
[closure comment](https://github.com/Aquaticats/Monochromatic/issues/1114#issuecomment-6230481503),
label,
and closed state at `2026-10-17T06:04:17Z`.
The closure applies to the isolated trial helper,
without claiming production adoption or a provider replay.

The actual `trial:depot:ci:source-identity` task at harness `4c4063e78bc9aaed8e293c1b478db089d6eef0ce`
passed before/after controls against clean historical sources `cb95cf6d78` and `3434eaa455`.
The controls used private workspace directories with source/harness symlinks,
`GIT_OPTIONAL_LOCKS=0`,
and a private identity output directory.
The shared source worktrees retained their original heads and clean tracked status.
The shared harness task file stayed byte-identical throughout verification.
No cloud run was dispatched.

The task asserts source HEAD against the selected manifest,
rejects dirty tracked status,
compares configured input bytes to the committed Git blobs,
and compares the final source/tree/input/manifest identity to the saved baseline.
The native control retained 16 input hashes;
the desktop control retained nine.
Actual negative controls rejected wrong source,
ordinary tracked-source edits,
a changed hashed input hidden by `assume-unchanged`,
a changed manifest,
an invalid phase,
and an absent before record.
The disposable fixture was restored and its final after control passed.

The explicit hash enumeration has a narrower boundary than all configured native inputs.
`package/monorepo-task/core-exec/mise.toml:816` starts with root JavaScript/mise metadata,
then line 818 adds exact metadata filenames immediately inside each selected package:

```javascript
// package/monorepo-task/core-exec/mise.toml at harness 4c4063e78b
const paths=new Set(['mise.lock','mise.toml','pnpm-lock.yaml','pnpm-workspace.yaml','package.json']);
for(const pkg of packages)for(const name of ['mise.toml','package.json','Cargo.toml','Cargo.lock','tsconfig.json','.cargo/config.toml'])if(trackedFiles.has(`${pkg}/${name}`))paths.add(`${pkg}/${name}`);
```

The native baseline omitted these tracked configured inputs:

- `.cargo/config.toml`.
- `package/music-player/android-app/rust/Cargo.toml`.
- `package/music-player/android-app/rust/Cargo.lock`.

Each omission was tested independently in a private clone.
The control marked that file `assume-unchanged`,
changed its bytes,
confirmed empty tracked status,
and ran the actual after task.
Every changed omitted input was accepted with exit zero and the original saved identity.
In contrast,
the same index-hidden change to the hashed `mise.lock` was rejected with
`Configured source input differs: mise.lock`.
The byte-comparison positive control therefore detected a hidden change when its path was enumerated.

This demonstrates a hash coverage gap under index-hidden edits;
it does not establish that a real provider run changed one of these files or set an index flag.
Ordinary tracked edits were rejected.
Untracked files and ignored build outputs remain outside the helper contract.
The harness owner implemented the repairs documented in Verified repair and Index exclusions are rejected.
The verification agent changed only disposable fixtures and documentation.
The final workflow must retain the helper's actual input set and avoid claiming omitted config hashes.

The [fixture evidence](../audit/depot-source-identity-controls-2026-10-17.evidence.json)
retains the actual task identity,
both source/tree/manifest identities,
input hashes,
terminal control outcomes,
and changed/original hashes for each omitted input.
The fixture driver and complete command output remain in
`~/temp/agent/depot-source-identity-controls-20261017`.

### Verified repair

Harness commit `c0baaeb1d6` expands the input enumeration to root configuration,
root `.cargo/` files,
and nested metadata inside selected or artifact-prerequisite packages.
Commit `faf388e838737e6b08b9a8075cdcbf4b34f02b2b` changes tracked path enumeration to NUL boundaries:

```javascript
// package/monorepo-task/core-exec/mise.toml:817 at harness faf388e838
const trackedFiles=new Set(git(['ls-files','-z']).split('\\0').filter(Boolean));
```

A fresh fixture run through the actual task verified the repaired helper with 19 terminal controls.
Both historical before/after pairs passed with unchanged source heads and clean tracked status.
The native identity now retained 36 input hashes;
the desktop identity retained 26.
Every formerly omitted input named in Source identity input coverage was hashed,
and each index-hidden byte change was rejected with `Configured source input differs`.
The original wrong-source,
dirty-source,
hashed-input,
manifest-change,
invalid-phase,
and missing-baseline controls also retained their expected rejection outcomes.

An additional disposable source commit added nested `Cargo.toml` metadata whose directory name contains
spaces,
single and double quotes,
and a newline.
The actual before/after task included its exact path and correct byte hash.
An index-hidden edit to that file was rejected;
restoring it produced a passing after check and clean tracked status.
This exercises the NUL path boundary through Git enumeration,
filesystem reads,
Git blob reads,
JSON persistence,
and final identity comparison.

The [repaired fixture evidence](../audit/depot-source-identity-repaired-controls-2026-10-17.evidence.json)
is separate from the unchanged original 15-control record.
It retains harness revision `faf388e838737e6b08b9a8075cdcbf4b34f02b2b`
and task-file SHA-256 `d769e393662c59fef7c00216fe302912f46610520c9ca1de709c1da836f8da8e`.
The shared harness task file remained byte-identical throughout the repaired verification.
Only disposable fixture state and documentation were changed;
no cloud run or production test suite was dispatched.
These controls verify the named input and path boundaries,
not every possible repository path.
The repaired fixture driver and command logs remain in
`~/temp/agent/depot-source-identity-repaired-controls-20261017`.

### Index exclusions are rejected

Harness `a07d84a129f0dd16861551adfb76b1a3bbc4083d` adds an independent index invariant before tracked status.
`package/monorepo-task/core-exec/mise.toml:818` enumerates NUL-delimited status tags:

```javascript
// package/monorepo-task/core-exec/mise.toml:818 at harness a07d84a129
const indexEntries=git(['ls-files','-v','-z']).split('\\0').filter(Boolean);
```

The assertion on line 819 rejects uppercase `S` or lowercase status tags.
The installed Git `2.56.0` help identifies `-v` as using lowercase tags for `assume-unchanged` files.
Disposable controls observed `h` after `--assume-unchanged`
and `S` after `--skip-worktree`.
The task emits:

```text
Immutable source index must not hide tracked inputs with assume-unchanged or skip-worktree flags
```

A fresh focused fixture run completed 12 controls.
Both historical source before/after pairs passed.
The ordinary tracked `package/music-player/truepeak-core/src/lib.rs` is outside the selected metadata hash set.
For each index flag,
the unchanged ordinary file was rejected,
then an edited version with empty tracked status was also rejected.
Restoring the bytes and clearing the fixture-local flags restored a passing after check.
The unflagged metadata path containing spaces,
quotes,
and a newline still passed before/after checks with its exact hash.

The metadata hash list remains a selected evidence subset,
not an exhaustive byte inventory of every source file.
The independent flag assertion prevents Git index exclusions from hiding paths outside that subset.
The tradeoff is deliberate:
indexed exclusions are incompatible with the fully checked immutable-source policy,
even when the excluded file currently matches its committed bytes.
A checkout using `assume-unchanged` or `skip-worktree` cannot pass this gate.
The controls changed flags only in a disposable clone;
shared historical source worktrees and the shared harness remained unchanged.

The [index-invariant evidence](../audit/depot-source-identity-index-controls-2026-10-17.evidence.json)
retains the additional controls separately from the unchanged 15-control and 19-control records.
The tested task-file SHA-256 is `afb1293dedc5dd4fb398a0684661fc9a56d92cdccc20db8ab164880b8311db4b`.
The driver and logs remain in `~/temp/agent/depot-source-identity-index-controls-20261017`.
This verification dispatched no provider run and repeated no broad test suite.

## Logger cohort preflight accepts a provisional case

The actual logger cohort guard at trial harness `673ab92e49885dd61fcf08e168ed9c59bf29c6be`
accepts its committed case manifest with `completeGraph: false`.
The source,
harness,
route,
and outer common-manifest digest can all match while the static case registration remains provisional.
This is separate from the byte-preserving transport boundary in `#1119`.

Root trial harness `ae3aad924c7b74c83d27ad0f286832008195eebf` adds the missing assertion to
`package/logger/core.fuzz/mise.toml`,
task `trial:depot:cohort-identity`:

```text
Final route requires a frozen complete case manifest
```

The assertion reads the actual committed local case bytes before accepting the shared cohort.
The integration retains the existing exact-byte,
Git-blob,
workflow,
clean-harness,
and harness-index checks.
It also rejects conflicting selected cases and provider revisions,
requires the provider revision in CI,
validates canonical dispatch inputs,
and sets a 30-second timeout on the Contents API subprocess.

Disposable clones execute the real task with a fixture supplying only the absent canonical Contents API response.
The unmodified guard accepts the actual provisional manifest.
The repaired guard rejects that manifest,
accepts valid before/after complete-case fixtures,
and rejects 21 additional malformed binding/input/harness controls.
Removing only the new complete-case predicate restores invalid acceptance.
The source and all harness fixtures finish with clean tracked state.
At root integration `ea0306a772`,
the task body equals the controlled task body,
with SHA256 `ce2425d4b0d3e84f3f4370976dc10f9a61f7b4c0ce66513c8c40ef52f4edf9ca`.

Issue `#1141` is filed and closed with fresh body/state verification.
The `rootLoggerIntegrationPreparation` field in
`doc/audit/depot-final-recovery-2026-10-17.evidence.json` retains the controls and issue receipt.
These fixtures do not publish a fake common manifest or establish actual provider acceptance.
Actual case manifests remain provisional until the complete static registration is frozen.

## Generated logger host task directories freeze a local path

Root preparer `7d487813bfe559b0fa5299e36aba010d71439b34` embedded a literal Mise directory template
inside its own inline task body.
Mise expanded that template before the Node generator wrote the new helper definitions.
The source identity,
prerequisite,
and result helpers consequently contained the generator's local absolute directory,
followed by `/../../..`.
The generated file establishes this cause;
the native owner's original portable template is unaffected.

The first actual source preflight reported Node `ENOENT` reading `.depot/manifests/logger.json`
outside the harness checkout.
Root `bcc683bd85` addressed that symptom by binding case reads and harness Git calls to
the explicit `GITHUB_WORKSPACE/harness` checkout.
That repair alone did not make the generated task directories portable.

Root `d05781e816` preserves the directory template in all three definitions.
The preparation task constructs the template delimiters at Node runtime,
after Mise has rendered the generator task.
Actual Mise directory probes reach the disposable harness for each corrected helper;
the same probes reach the wrong directory with each committed legacy definition.
Fresh source/cohort controls accept four valid boundaries and reject nine altered inputs.
The existing result gate accepts a complete receipt and rejects seven failure,
skip,
and runtime variants;
removing the result gate admits a failed source command.

These are Linux harness controls.
The source and lock prerequisite prefix passes,
but this proof neither installs dependencies nor executes the original source build.
Native platform prerequisite verification remains a separate development workflow.
`rootLoggerHostPreparation` in `doc/audit/depot-final-recovery-2026-10-17.evidence.json`
retains the initial failure and corrected controls separately from actual provider acceptance.

## Named formatter install refreshes an unrelated lock entry

Local Mise `2026.10.0 linux-x64 (2026-10-02)` ran `mise install dprint@0.61.1`
inside the prepared root trial harness with `MISE_LOCKED=0`.
The named installation changed `mise.lock`'s recorded Rust version from `nightly-2026-10-13`
to `nightly-2026-10-17`.
The tracked diff established the unrelated lock change;
it does not establish that an unrelated Rust compiler was installed.
Only this exact owned diff was restored,
and the worktree then verified clean.

The real invocation `mise --no-config install dprint@0.61.1` returned zero in a disposable fixture
and preserved its lock bytes.
The before/after lock SHA256 is `95704fad9023cea1b8b530d5de696cfbd839f912aba7c0dc2c1c46519ebef733`.
The changed-Cargo formatter uses this isolated installation and
`mise --no-config exec dprint@0.61.1 -- dprint check --list-different <selected-file>`.
Its actual Mise fixture controls execute and verify each selected Cargo file,
leave the source lockfile unchanged,
and prove that malformed unselected Cargo input remains detectable by an unscoped dprint invocation.
The adapter still uses the source checkout's dprint configuration.

The [formatter controls](../audit/depot-changed-cargo-format-controls-2026-10-17.evidence.json)
retain the measured local Mise version,
source-locked formatter version,
isolation receipt,
and exact helper/control definitions.
The provider bootstrap uses Mise `2026.10.9`;
its final execution remains a separate acceptance boundary.
No third-party defect is inferred from this local side effect.

## Prepared Android launch retained a legacy unit

The initial local Android staging adapter updated standalone `depot-trial-fold` literals,
but missed the launch argument `--unit=depot-trial-fold`.
Actual `mise tasks info launch --json` therefore exposed a launch/boot unit mismatch,
despite valid syntax and a preparation record claiming a unique unit.
The initial claim was retracted before any emulator launch or device operation.
This was an adapter defect,
not a systemd or Android Emulator failure.

The repair replaces the complete launch argument,
rejects any residual legacy-unit reference,
and checks every generated task through Mise's actual parser against its syntax-checked body hash.
A retained pre-fix fixture reproduces the parsed mismatch;
the corrected launch and boot both name `depot-coherent-android-a038cc3c`.
The [preparation evidence](../audit/depot-coherent-android-consumer-2026-10-17.evidence.json)
retains original/repaired hashes,
the parsed-command control,
and repeated offline producer-input checks.
The [preparation contract](../audit/depot-coherent-android-consumer-2026-10-17.md)
keeps final-producer and live-device dependencies explicit.
Issue [#1115](https://github.com/Aquaticats/Monochromatic/issues/1115) is closed for this verified local correction.
No source bytes,
owned AVD,
or shared historical consumer were changed.

## Verification catalog

- Passed:
  historical JavaScript wrapper build and policy trust before TOML core execution.
- Passed:
  published native wrapper installation and source tool gate before the desktop host probe.
- Failed:
  Git reads immediately after dependency linking at the older JavaScript-wrapper revision.
- Failed:
  inherited offline native-wrapper source build without its registry cache.
- Failed:
  generated worktree checkout with default LFS smudging against the obsolete endpoint.
- Pending:
  generated consistency after pointer-preserving checkout and fixture-local policy trust.

Do not disable policy guards,
change historical source inputs,
or hand-edit the lockfile to make these setup assertions pass.

## Upstream filing decision

These are repository harness and historical configuration boundaries.
No third-party defect has been established.
The upstream-fault criterion therefore fails;
fixability and supported-use-case criteria do not establish responsibility.
External contribution-policy and comparable-issue reviews are not grounds to file without a defect.
No external filing or prototype is proposed.
Owned issue tracking retains the separate incidents and provider costs.

## Related evidence

- [Executor queue and run ledger](../handover/depot-affected-ci-executor.md).
- [Shallow checkout ancestry failure](cargo-publish-shallow-before-sha.md).
- [Expanded coverage issue](https://github.com/Aquaticats/Monochromatic/issues/1030).
