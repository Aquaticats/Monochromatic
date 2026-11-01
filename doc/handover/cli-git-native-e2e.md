# Native end-to-end lane: the incumbent's e2e suite against the native executable

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/caught-value` is now `package/caught-value/core`.

Note,
 2026-10-14:
 the retirement of the TypeScript wrapper deleted the tasks `test:e2e:concurrent:native:surface`,
 `test:e2e:concurrent` and `pack:npm` that this record names,
 and rewrote `test:e2e:concurrent:image` to build the base image alone.
The record is `doc/handover/cli-git-native-retire.md`;
the incumbent that this record compares against is the commit `f4fc1fa213ece65672e4314983a75266837892f9`.

Branch `feat/cli-git-native-e2e`, based on the transactions head `13f60ef1b1`, merged into the integration branch.
This file holds the current state only; when the state changes, replace the text instead of appending.

## State in one paragraph

The incumbent's end-to-end suite (`package/git-policy/cli/e2e`) runs unchanged against the native executable
with `E2E_WRAPPER=native`.
On the integration branch, with the commit path enabled, all 38 catalog scenarios were run on Git 2.55.0 with seed 1:
36 pass and 2 skip by design (the `replay-unavailable-*` scenarios, which exist only for Git below 2.40.0).
The non-commit commands that the scenarios send through the wrapper behave identically in the incumbent and the
native executable (25 of 25 commands agree).
The integration document `doc/handover/cli-git-native-integration.md` holds the run and its rerun commands.

## What was built

Commits on this branch, in order (all touch only `package/git-policy/cli/e2e`, `bin/e2e-native-container.mjs` and
`mise.toml`; nothing under `src/native/`):

- `ef646ed87e`: `e2e/wrapper-selection-fixture.ts` (the selector), `e2e/repository-fixture.ts` reads it,
  `e2e/concurrent-e2e-consumer.ts` prints and verifies the selection,
  `bin/e2e-native-container.mjs`, and the mise tasks.
- `51a72ff4a0`: `e2e/README.md` documents the native tasks.
- `7957ab7129` and `d20e70c57e`: the surface probe (`e2e/wrapper-surface-probe.ts`,
  `e2e/wrapper-surface-steps-fixture.ts`) and the `surface` subcommand.

### The selector

- `E2E_WRAPPER` is `incumbent` (default, also when unset or empty) or `native`; any other value throws
  `WrapperSelectionError`.
  `WRAPPER_BIN` in `e2e/repository-fixture.ts` is the only place the wrapper directory enters a scenario: it is the
  first `PATH` entry of every wrapper run.
  Scenarios, the invariant checker and the report are untouched.
- Before the first scenario the consumer opens the selected `git` and checks that it is an ELF executable (native) or a
  `#!` script (incumbent), then prints `wrapper=<kind> <path> (<kind of program>)`.
  A run labelled native can therefore never drive the TypeScript wrapper.
- Unit test `e2e/wrapper-selection.unit.test.ts` (6 checks across 2 `describe` blocks) passes under `node`; the
  rejecting checks were proven to fail when an expectation is flipped.
  `mise run //package/git-policy/cli:lint:oxlint:paths` reports 0 warnings and 0 errors for every added or changed
  file, and `lint:types` reports no error under `e2e/` or `bin/` (the package's other `src/*.unit.test.ts` errors come
  from a missing `dist/`, unrelated).

### The native image

- `mise run //package/git-policy/cli:test:e2e:concurrent:native:image` runs `bin/e2e-native-container.mjs image`.
  Stage one compiles the release `cli-git-native` from a mount-free snapshot of this checkout in
  `docker.io/library/rust:1.97-bookworm` (offline, vendored, `--locked`, 2 GiB).
  Stage two starts from `localhost/cli-git-concurrent-e2e:latest` and links the executable as `/opt/native/bin/git`.
- Why bookworm: the end-to-end image is `node:24.11.0-slim` on Debian 12,
  so the executable must not need a newer glibc; the native gate image is Debian 13.
  The image has no `/usr/bin/git`, so the native wrapper finds the scenario's Git (`/opt/git/<version>/bin`) only
  through `PATH` order, exactly as the incumbent does.
- The base image `localhost/cli-git-concurrent-e2e:latest` is built by the existing
  `test:e2e:concurrent:image` task (needs `pack:npm`); the native image task does not build it and fails with that
  instruction when it is missing.
- Image built for the runs below: `localhost/cli-git-concurrent-e2e-native:e2e`, id `1d843b0f655c`,
  compiled from the `src/native` of `13f60ef1b1`.
  Repeating the image task with unchanged sources hits the layer cache; the run task depends on it.

### The surface probe

`e2e/wrapper-surface-probe.ts` runs 25 non-commit commands that the scenarios send through the wrapper (and a few that
policies may refuse) in a disposable linked-worktree repository, once per wrapper in the same image, and the driver
compares exit code, arguments and output with object IDs masked.
The two runs are sequential because both relabel the `e2e/` mount privately (`:Z`); concurrent containers contest it.

## Rerun commands

All from the repository root, through the host slot limiter (four slots shared with the other lanes; a mutation
campaign can hold every slot for hours, so a heavy command may wait).
`GIT_POLICY_NATIVE_IMAGE_TAG` names the image tag; use one value per worktree.

```sh
# repository root: build the native end-to-end image (cached when src/native is unchanged)
GIT_POLICY_NATIVE_IMAGE_TAG=e2e node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:test:e2e:concurrent:native:image

# repository root: the whole catalog on Git 2.55.0, the result in this document
GIT_POLICY_NATIVE_IMAGE_TAG=e2e node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:test:e2e:concurrent:native --seed 1 --git 2.55.0

# repository root: the non-commit commands, incumbent against native
GIT_POLICY_NATIVE_IMAGE_TAG=e2e node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:test:e2e:concurrent:native:surface
```

The incumbent control (default selector, packed incumbent from the existing image, same bounds as the task):

```sh
# package/git-policy/cli
node "${HOME}/temp/agent/gate-slot.ts" podman run --memory=2g --cpus=2 --rm --network=none \
  --env E2E_SEED=1 --env E2E_GIT_VERSIONS=2.55.0 \
  --env E2E_SCENARIOS=baseline-explicit-commit,baseline-checker-positive-control,baseline-sigkill-post-commit \
  --volume "$PWD/e2e:/fixture/e2e:ro,Z" --volume "$PWD/../../module/caught-value:/fixture/caught-value:ro,Z" \
  localhost/cli-git-concurrent-e2e:latest node /fixture/e2e/concurrent-e2e-consumer.ts
```

## Results with the commit path enabled

Seed 1, Git 2.55.0, 38 scenarios (the catalog has 38; with three Git versions it would be 114 runs, but the native
floor is Git 2.55.0, so the native run uses `--git 2.55.0` only).
36 scenarios pass, including the five `sigkill-phase-*` scenarios, the reservation scenario and every replay
scenario; 2 skip: `replay-unavailable-fails-fast` and `replay-unavailable-disjoint-burst` run only on Git 2.39.5
(`replayPlumbing: 'absent'`), which the native executable refuses.
Native workload digests equal the incumbent's for the three scenarios both ran (`f5c1d30bb33a` for
`baseline-explicit-commit`, `520136487c60` for `baseline-sigkill-post-commit`, `52025dfc1bc6` for
`baseline-checker-positive-control`), so the seeded workload does not depend on the wrapper.
`branch-switch-during-commit` is a real pass now: its held commit exits 1 with the `branch-switched` finding.

## Non-commit surface: no native bug found

The incumbent in this comparison and in the control run is the build baked into
`localhost/cli-git-concurrent-e2e:latest` (image id `1d29d429cddf`, created 2026-09-26), whose
`dist/final/node/index.mjs` has sha256 `8103d0848e168b52654e32a80a940de21535972033499f2e124eb451b09a944a`;
it is not rebuilt from `13f60ef1b1`, so a difference introduced into the incumbent after that date is not covered.

`test:e2e:concurrent:native:surface`: 25 of 25 commands agree between that incumbent and the native executable on
exit code, arguments and output (Git 2.55.0, linked worktree).
The commands: `status --short` (clean, modified, after checkout, after apply), `add -- <path>`,
`diff --staged --name-only -z --diff-filter=ACMR`, `diff --binary --unified=0 ... --staged`, `stash create`,
`stash store --quiet --message`, `stash list --format`, `stash drop --quiet`, `checkout --force -- <path>`,
`apply -v --whitespace=nowarn --recount --unidiff-zero`, `switch side`, `switch <branch>`, `gc --quiet --prune=now`,
`push`, `rev-parse --abbrev-ref HEAD`, `log --oneline --max-count=1`, `branch --list`, `config --get`,
`commit --dry-run --short`, and `add --all` (exit 1 in both: the bulk-add policy), `reset --hard`, `clean --force`.
The comparator can detect a difference: with object IDs unmasked it reported `stash store`, `log` and `reset --hard`
as different (the output carries IDs that differ between two containers), so the probe masks seven and forty digit
hexadecimal tokens that hold a decimal digit.
Unexercised: hook-nested wrapper calls (they need a commit), Git older than 2.55.0, Windows.

## Incumbent control

The default selector still drives the incumbent: `baseline-explicit-commit`, `baseline-sigkill-post-commit` and
`baseline-checker-positive-control` pass on Git 2.55.0 with seed 1
(`wrapper=incumbent /opt/cli-git/node_modules/.bin/git (Node script)`), the last one detecting its five planted
violations.
The packed incumbent in that run is the one baked into `localhost/cli-git-concurrent-e2e:latest`; the `e2e/` code
under test is the mounted working copy.

## Facts for the integrator

### Git floor: the native suite runs on Git 2.55.0 only

The native minimum is Git 2.55.0 (user decision of 2026-10-10 in `~/temp/agent/lane-brief.md`;
`MINIMUM_GIT_VERSION` in `src/native/git_version.rs`).
The executable therefore refuses Git 2.39.5 and 2.40.0, observed on a real invocation in the native image
(`git add -- added.txt` in a repository made by `/opt/git/2.39.5/bin/git`): exit 2 with the engine failure
`cli-git: real Git at /opt/git/2.39.5/bin/git reports version 2.39.5; commits, hooks and the locks that coordinate
with them need Git 2.55.0 or newer. Nothing was run. Upgrade Git, then run the command again.`
So on those two versions the harness's own first wrapper command fails and the scenarios fail in setup
(`baseline-explicit-commit` and `baseline-index-commit` observed, seed 1).
Consequences: runs on 2.39.5 and 2.40.0 and both `replay-unavailable-*` scenarios (they prove degradation on 2.39.5)
are out of scope for the native executable.
`doc/handover/cli-git-native-transactions.md` states the floor and no longer asks for degradation on Git 2.39.5.

### Phase markers

`src/native/transaction_phase.rs` implements the incumbent's protocol: `<phase>:kill`, `<phase>:kill:<directory>` and
`<phase>:pause:<directory>`, with `<directory>/<phase>.reached` written at the phase, `<directory>/<phase>.release`
waited for under `pause`, and `race-lost-<n>` files for the occurrence.
The five `sigkill-phase-*` scenarios and the `race-lost` pause in `reservation-after-lost-races` pass with it.

### Not yet done

The concurrency runs with both wrappers mixed (a trailing gate).
The JSONL comparison against the incumbent is `mise run //package/git-policy/cli:native:differential:commit`.

## Scenarios and rerun

The scenarios, grouped by fixture file under `package/git-policy/cli/e2e` (all 36 non-skipped ones on 2.55.0 need the
commit path):

- `scenario-baseline-commit-fixture.ts`: `baseline-explicit-commit`, `baseline-index-commit`,
  `baseline-trace-sequential`.
- `scenario-baseline-amend-fixture.ts`: `baseline-amend`, `baseline-lint-staged`.
- `scenario-baseline-hook-fixture.ts`: `baseline-hooks-hookdir`, `baseline-hooks-config`, `baseline-ssh-signing`,
  `baseline-sigkill-post-commit`, `baseline-checker-positive-control`.
- `scenario-concurrent-paths-fixture.ts`: `concurrent-disjoint-paths`, `concurrent-trace-replay`.
- `scenario-shared-file-fixture.ts`: `shared-file-non-overlapping-hunks`, `shared-file-overlapping-hunks`.
- `scenario-subsumption-fixture.ts`: `shared-file-adjacent-edits`.
- `scenario-index-writer-fixture.ts`: `interleaved-index-writers`.
- `scenario-hooked-fixture.ts`: `hooks-hookdir-concurrent`, `hooks-config-concurrent`, `ssh-signing-concurrent`,
  `lint-staged-stash-concurrent`.
- `scenario-amend-switch-fixture.ts`: `amend-during-commits`, `branch-switch-during-commit`.
- `scenario-reservation-fixture.ts`: `reservation-after-lost-races` (uses the `race-lost` markers).
- `scenario-lock-gc-fixture.ts`: `foreign-index-lock-holder`, `gc-prune-during-commits`.
- `scenario-foreign-unproven-fixture.ts`: `foreign-index-lock-unproven`.
- `scenario-sigkill-fixture.ts`: `sigkill-pre-commit`, `sigkill-prepare-commit-msg`, `sigkill-commit-msg`,
  `sigkill-post-commit`, `sigkill-offset`.
- `scenario-phase-kill-fixture.ts`: `sigkill-phase-preparation-done`, `sigkill-phase-landing-locked`,
  `sigkill-phase-objects-migrated`, `sigkill-phase-ref-updated`, `sigkill-phase-index-installed`
  (use the phase markers).
- `scenario-replay-degradation-fixture.ts`: `replay-unavailable-fails-fast`, `replay-unavailable-disjoint-burst`
  (Git 2.39.5 only; out of scope for the native executable, see "Git floor").

The command that runs all of them (a full run takes about 33 minutes under host load, so start it as its own unit with a
memory cap and wait for its log):

```sh
# repository root
systemd-run --user --unit=e2e-native-full --collect --property=MemoryMax=3G \
  --setenv=PATH="$PATH" --setenv=HOME="$HOME" --setenv=GIT_POLICY_NATIVE_IMAGE_TAG=e2e \
  --working-directory="$PWD" --property=StandardOutput=file:"$HOME/temp/agent/e2e-native-full.log" \
  --property=StandardError=file:"$HOME/temp/agent/e2e-native-full.log" \
  node "$HOME/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:test:e2e:concurrent:native --seed 1 --git 2.55.0
```

Only the sigkill-phase scenarios:

```sh
# repository root
GIT_POLICY_NATIVE_IMAGE_TAG=e2e node "${HOME}/temp/agent/gate-slot.ts" \
  mise run //package/git-policy/cli:test:e2e:concurrent:native --seed 1 --git 2.55.0 \
  --scenario sigkill-phase-preparation-done,sigkill-phase-landing-locked,sigkill-phase-objects-migrated,sigkill-phase-ref-updated,sigkill-phase-index-installed
```

Git 2.39.5 and 2.40.0 stay out of the native runs (see "Git floor").
The task exits non-zero when any scenario fails or errors; every run prints its seed first, so a failure replays with
the same `--seed`.

## Choices open to veto

- The native image is built on top of `localhost/cli-git-concurrent-e2e:latest` instead of rebuilding Git from source,
  so Node and the three Git builds are exactly the incumbent run's.
  The cost is that the base image must exist and holds an incumbent that the native runs never put on `PATH`.
- The executable is compiled in `rust:1.97-bookworm`, not in the native gate image (Debian 13), to match the
  runtime glibc.
  The gate stays the authority for Clippy and unit tests.
- `bin/e2e-native-container.mjs` copies the snapshot file list of `bin/test-native-container.mjs` and
  `bin/interop-native-container.mjs` (a third copy); extracting one shared function would touch scripts other lanes
  edit, so it is left for the integrator.
- The seed 1 run covers one interleaving of the free-running scenarios;
  the integration document records the further seeds.

## Trailing gates

No gate was skipped by this lane.
`native:test:container` and `native:clippy` were not run because nothing under `src/native/` changed; this lane's
container evidence is the native end-to-end image `1d843b0f655c` (base `1d29d429cddf`).
Mutation testing, planted controls and fuzz targets do not apply: the lane adds test harness code and no
`src/native` code.
