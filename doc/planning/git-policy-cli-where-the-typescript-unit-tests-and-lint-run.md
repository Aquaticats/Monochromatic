# Where the wrapper package's TypeScript unit tests and lint run

## Status

A proposal,
unapproved and non-final,
written by a Pi round for issue 6584 on 2026-11-27T08:11:12-05:00.
It changes no code,
no test,
no task and no workflow,
and the release gate's own steps stay the owner's until the owner chooses
(`gh issue view 6584`:
"No change to the gate itself without the owner's choice:
what the gate runs is recorded as the owner's").
The owner declares completion.

The round ran the three checks on this host,
once at the start and once after merging `main`'s later tip,
read what runs today out of the tasks,
the gate's scripts and the workflows,
and fixed the first red commit of each check.
`main` moved to `64e3eb1a55` while the proposal was written,
with the fixes for issues 6579 and 6580,
so the unit tests pass at the tree this branch ends on and the lint is still red;
both sets of figures are below,
and each is dated to the tree it was taken on.
What is not yet known:

- what the three checks cost on a CI runner,
  whose four virtual cores and cold caches are not this host,
- whether the owner wants a red check to hold a merge,
  which decides whether a CI placement is a signal or a gate,
- the exact commit at which oxlint went red between 2026-11-04 and 2026-11-13,
  because pinning it needs an oxlint run at each commit,
  and this round ran it at one,
- the run-to-run spread of `test:unit`'s peak memory,
  which is the largest figure any placement has to size a bound for.

## The three checks, and which were red

On `main` at `1c8acd5bfb`,
in this round's worktree,
`mise run //package/git-policy/cli:lint:types` exits 0,
`mise run //package/git-policy/cli:lint:oxlint` exits 1 with 850 errors and 1,856 warnings,
and `mise run //package/git-policy/cli:test:unit` exits 1 with two failing files.
The three failures have their own issues:
the service-words guard is issue 6579,
the planted-controls walk is issue 6580,
and the lint is issue 6581.
The first two were fixed on `main` while this proposal was written,
so at `64e3eb1a55` the unit tests pass and the lint is still red.
This document is about where the checks run,
not about the three failures.

The package holds 63 `*.unit.test.ts` files,
49 under `bin/` and 14 under `e2e/`,
and 242 TypeScript files across `bin/`,
`e2e/` and `perf/`:

```sh
rg --files -g '*.unit.test.ts' package/git-policy/cli | wc -l
rg --files -g '*.ts' package/git-policy/cli/bin package/git-policy/cli/e2e package/git-policy/cli/perf | wc -l
```

Those files sit beside the gate's own harness:
`bin/` holds the gate's launcher,
its limits,
its manifest,
its verdict reader,
and the planted-controls and mutation scripts,
which is why the unit tests of that harness are the check that would have shown the two red files.

## What the gate is for

The project's record says what the batch rests on.
`doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md:63` records the owner's choice,
named "A:
one gate on main per batch" (`:64`),
and its words are (`:65`):

```text
After a batch of merges,
one whole gate on main's tip;
a failure outside main's known baseline starts a fix round.
```

The handover states what that gate is for
(`doc/handover/cli-git-rust-implementation.md:761`):

```text
one gate on `main` per batch of merges is the check the batch rest on afterwards,
and a control that fails once in a gate for no fault of the product costs a round each time.
```

`doc/planning/git-policy-cli-cachyos-test-containers.md:400` says what a release waits for:

```text
A wrapper release waits for two things:
the container gate,
which `launch-gate.ts` starts as one systemd user unit named `gpc-gate-<what it covers>-<timestamp>`
and which runs `native:test:container`,
and the btrfs host controls,
which `native:release` runs through the crate's `dispatchGate` registry entry
(`native:test:host:btrfs`).
```

So the gate is the batch's own check,
the record of what a batch's code does,
and the thing a release waits for.
Nothing in those records says which checks the gate runs beyond the Rust suite and Clippy,
and the issue makes that set the owner's.

## What runs today

### The release gate runs the Rust suite and Clippy

`package/git-policy/cli/mise.toml:5` starts it,
after sweeping the run records:

```sh
run = ["cargo-shadow lean records target/verification", "node bin/test-native-container.ts"]
```

Its description says what it is
(`package/git-policy/cli/mise.toml:2`):
"Run native wrapper tests and Clippy together in bounded containers with pacman's current Git".

The suite is nextest over every target
(`package/git-policy/cli/bin/native-test-containerfile.ts:36`):

```text
export const runArguments = [
  'cargo',
  'nextest',
  'run',
  '--offline',
  '--locked',
  '--features',
  'test-only-switches',
  '--all-targets',
  '--no-fail-fast',
  '--success-output=final',
] as const;
```

and that array is the image's default command
(`package/git-policy/cli/bin/native-test-containerfile.ts:147`,
`CMD ${JSON.stringify([...runArguments])}`).
The gate starts that and Clippy as a pair
(`package/git-policy/cli/bin/native-gate-checks.ts:2`):

```text
The release gate's two checks, Clippy and the whole test suite, started together and judged together.
```

`package/git-policy/cli/bin/test-native-container.ts:343` says why:

```text
// The suite and Clippy start together and the gate waits for both: a clean Clippy is silent, and a Clippy compile
// behind the suite cost a fifth of the gate's wall time while both checks had nothing to say to each other.
```

Two facts follow.
The gate names no TypeScript task at all:
its two members are `testRun` and `clippyRun`,
and neither is a `mise run` of this package.
And its containers cannot run one as the image stands:
`package/git-policy/cli/bin/native-test-containerfile.ts:142` copies the vendored crate root
(`COPY --chown=1000:1000 ${imageRootDirectory} /work`),
and the file copies each `package/<root>` of the crate's path dependencies
(`:157`),
but nothing in either file copies the repository's `package.json`,
`pnpm-workspace.yaml`,
`tsconfig.json` or `node_modules`,
so a unit test that imports `@monochromatic-dev/test-harness-core/ts` resolves nothing there.

### The package's own tasks, and the templates behind them

`package/git-policy/cli/mise.toml:180` makes the package lint the shared template,
and the package's own TypeScript lint and unit tasks extend their templates too
(`:197`,
`:200` and `:234`):

```text
[tasks.lint]
extends = "lint"
```

```text
[tasks."lint:types"]
extends = "lint:types"

[tasks."lint:oxlint"]
extends = "lint:oxlint"
```

```text
[tasks."test:unit"]
extends = "test:unit"
```

The templates are in the root `mise.toml`.
`lint` runs every direct child named `lint:*`,
one task level deep,
all in parallel
(`mise.toml:74`):

```text
# node -e fanout script: discovers direct child tasks (one `:` level deeper) and
# runs each as a plain `mise run <child>` subprocess, all in parallel, then fails
# if any child failed.
```

`lint:types` is a TypeScript build
(`mise.toml:578`):

```text
[task_templates."lint:types"]
description = "TypeScript"
shell = "node --input-type=module-typescript -e"
run = """
{{vars.dispatch_workspace_node}}
runWorkspaceNode('package/monorepo-task/core-exec', 'tsc-filter', ['--build'])
"""
```

`lint:oxlint` is one oxlint-wrapper call with no paths,
so it lints the package by its own configuration
(`mise.toml:562`):

```text
[task_templates."lint:oxlint"]
description = "Lint with Oxlint"
shell = "node --input-type=module-typescript -e"
run = """
{{vars.dispatch_workspace_node}}
ensureOxlintConfig()
runWorkspaceNode('package/monorepo-task/core-exec', 'oxlint-wrapper', [])
"""
```

`test:unit` is the unit-test runner,
whose description says the shape
(`mise.toml:499`):
"Run active-tree *.unit.test.ts in parallel via node (excludes *.expensive.*
unless --all);
pass file paths to filter",
and whose body spawns one `node` process per file,
bounded by `availableParallelism()`
(`mise.toml:346`).

So the package's own `lint` runs both TypeScript checks,
and `test:unit` runs the unit files.
Nothing else in the package calls them,
and the two do not call each other.

### The repository-wide tasks that would reach them

Three root tasks reach this package's files without anyone naming it.
`//:lint` fans out to every package's own `lint`
(`mise.toml:1000`):

```text
run = [
  "{{vars.fanout_packages}}",
  { tasks = ["lint:dprint", "lint:monochromatic-lint", "lint:skills"] },
]
```

`//:test` runs the root unit suite,
the browser suite,
the e2e suite and then every package's own `test` task
(`mise.toml:802`,
whose description is
"Run all tests (root unit,
browser,
e2e and every package's own test task)").
`//:test:unit` discovers `**/*.unit.test.ts` and leaves a file alone only when its package declares its own `test` task
(`mise.toml:781`):

```text
# Ownership policy, stated once and implemented in two places: a package that
# declares `[tasks.test]` owns its own suites, including its `*.unit.test.ts`
# files, and the root unit glob leaves those files to that package task (see
# [task_templates."test:unit"]). Every other package is reached by the root glob.
```

This package declares `test:unit`,
`test:service-manager` and three `test:e2e:*` tasks,
and no `test`,
so `//:test:unit` does discover its 63 files,
while the package fanout `//:test:packages` passes it by.
That is a reading of the task text above rather than a run of the root suite,
which this round did not start.

### What a round runs before it ends

The package guide settles that
(`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md:675`):

```text
A branch merges into `main` when its round has ended and its report is read:
no whole container gate and no review is a condition of the merge
(`doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md`),
and one whole gate runs on `main` for each batch of merges.
So a round runs the modules it touched,
its guest arms where its brief asks for them,
and the controls its brief asks for,
red and green,
and says in its report which checks ran and which did not.
```

The modules are Rust modules,
and `source_guard` is a Rust test module,
so a round that changes `bin/` runs no TypeScript check at all.
The guide's other rules name Rust commands only,
apart from the linter the repository runs over Rust,
Markdown and MDX.

### What the coordinator's batch check runs

The handover recorded the tip check the coordinator ran after a batch
(`doc/handover/cli-git-rust-implementation.md`
[section removed on 2026-11-27 under rule H0P; the quotation below is its record]):

```text
`main`'s tip is checked after a batch of merges and not by each merge:
the merge routine runs `git merge-tree` and compiles nothing.
At `7a830014b3` (2026-11-27T07:34:30-05:00),
with the tip read before and after each run and found unchanged,
`native:check` exited 0,
the four Clippy tasks exited 0
(`native:clippy`,
`native:clippy:test-only-switches`,
`native:clippy:measurement`
and `native:clippy:start-probe-raised-deadline`),
and the `source_guard` module passed 179 of 179.
```

`native:check` is `cargo check --locked --offline --all-targets`
(`package/git-policy/cli/mise.toml:105`).
So the batch check compiles the crate,
lints it four ways and runs one Rust test module,
and names no TypeScript task either.

### What CI runs, on which events

Merges on this project are made locally and pushed:
the newest merged pull request is `#1776` of 2026-10-19
(`gh pr list --state merged --limit 5`),
and since then the merges are local `git merge` commits,
5,392 of them since that date and 423 since 2026-11-27T00:00
(`git rev-list --merges --count --since=<date> HEAD`).
The workflows that fire on a push to `main` are therefore the ones that can see this package.

`advisory-core.yml` runs on `push` to `main`,
on `pull_request` to `main` and on `workflow_dispatch`
(`.github/workflows/advisory-core.yml:2`).
Its push runs are budgeted
(`:61`):

```text
- budgets: 2 per minute, 4 per hour, 8 per day
```

and a run past the budget skips its work.
Measured on the newest push run at the time of writing
(`gh run view 46004244481`),
the job named `advisory core` is `skipped`,
so the budget is being spent:
this repository pushes far more often than eight times a day.
The work it does run is a fast subset of six packages
(`.github/workflows/advisory-core.yml:227`),
none of them this one,
and its own summary says the results are advisory,
with "no required checks and no branch-protection change"
(`:241`).

`git-policy-cli-fuzz.yml` is the one workflow that filters on this package.
It runs on `push` to `main`,
on `pull_request` and in the merge queue,
filtered to `package/git-policy/cli/**`,
its sidecar and the crates the sidecar links
(`.github/workflows/git-policy-cli-fuzz.yml:23`),
and it runs one task
(`:130`):

```sh
task-run //package/git-policy/cli.fuzz:lint:types
```

That is the sidecar's type check,
not this package's.
Its own header says what it is for
(`:9`):
"The check is the sidecar's own `lint:types` task through `task-run`,
the task CLI every workflow uses,
because mise is never invoked in CI".

The remaining workflows that name the package are release paths:
`cargo-publish.yml` builds and publishes the crate,
`crate-release.yml` bumps it,
and `npm-release.yml` builds the binary for a release.
None of them runs `test:unit`,
`lint:types` or `lint:oxlint`.

### What runs nowhere

Nothing scheduled runs `mise run //package/git-policy/cli:test:unit`,
`:lint:types` or `:lint:oxlint`.
Not the release gate,
which is the check the batch rests on.
Not a round before it merges.
Not the coordinator's batch tip check.
Not a workflow on any event.
The only ways they run today are a person typing the task,
`mise run //package/git-policy/cli:lint` (which reaches the two lint tasks),
and the repository-wide `mise run //:lint` or `mise run //:test`,
neither of which anything in this effort runs,
because they carry every other package,
the browser suite and the e2e suite.

## What was measured

Each check ran in this worktree on this host,
in its own `systemd-run --user --scope` on `agent-work.slice`,
with `MemoryMax=8G`,
`CPUQuota=400%`,
`TasksMax=1024`,
`IOWeight=1` and `RuntimeMaxSec=1800`.
Wall time is taken around the `mise run` call inside the scope,
and peak memory is the scope's own `memory.peak`,
read from inside the scope before it ended.
The host is a ten-CPU machine with 61 GiB of memory,
16 GiB of it available,
and its load average was 17 to 27 during these runs.
The figures here are the runs at `1c8acd5bfb`:
`lint:types` ran twice,
once with no TypeScript build record and once with one,
and the other two ran once each.
The runs after merging `main`'s later tip are in "Where main moved while this was written".
`RuntimeMaxSec` was passed as the brief asks,
and `systemctl --user show <scope> -p RuntimeMaxUSec` reported `infinity` for a scope unit,
so each run was bounded by the round's own command bound instead.

What each check cost:

- `mise run //package/git-policy/cli:lint:types`
  exited 0 in 2,810 ms on its first run,
  with a peak of 452.9 MB;
  the second run exited 0 in 2,639 ms with a peak of 447.1 MB.
  The two agree within 6 per cent,
  so the build is not much cheaper when its record already exists,
  and the figure to plan with is 2.8 s and 0.45 GB.
- `mise run //package/git-policy/cli:lint:oxlint`
  exited 1 in 12,791 ms with a peak of 987.1 MB,
  and printed
  `Found 1856 warnings and 850 errors.`
  `Finished in 10.0s on 242 files with 491 rules using 4 threads.`
  The four threads are what oxlint read from the scope's 400 per cent CPU quota.
- `mise run //package/git-policy/cli:test:unit`
  exited 1 in 22,166 ms with a peak of 4,541.9 MB,
  and printed
  `Error: test files failed: .../package/git-policy/cli/e2e/service-mode-words.unit.test.ts, .../package/git-policy/cli/bin/native-planted-controls.unit.test.ts`.
  The 4.5 GB is the runner's parallel `node` processes,
  not one process,
  and it is the figure a memory bound for the step has to hold.

So the three checks cost 37.8 s of wall time run one after another,
and their slowest member is 22.2 s,
which is what a step that ran all three beside each other would cost in wall time,
with the same 4.5 GB peak.

For scale,
the gate's own recorded phases are much larger.
`doc/planning/git-policy-cli-cachyos-test-containers.md:498` records the suite at 883 to 1,104 s and Clippy at 195 to 261 s
for the gates of 2026-11-10,
and the gates of 2026-11-27 took 20 minutes 36 seconds and 21 minutes 43 seconds
from unit start to stop,
of which the suite was 889 s and 1,259 s
(`:1009`).
Against those figures the three checks are about 3 per cent of a gate's wall time.

A CI job's fixed cost was read from a runner's own log rather than run here.
`env-local-json`'s `Set up node, pnpm and the task CLI` step runs `pnpm install --frozen-lockfile`,
and on 2026-11-26 that step took 15 s while its store was cold
(`reused 0, downloaded 613` in the log of run `45754658990`),
and that job's checkout took 26 s.
So a runner pays about 40 s before a check starts,
and what the three checks then cost there is the figure this round could not take.

What could not be measured:

- what the three checks cost on a CI runner,
  whose CPUs are four virtual cores and whose caches are cold,
  against this host's four-CPU quota and warm install,
- the spread of `test:unit`'s peak memory or wall time across runs,
  because the brief asks for one run each and a second full unit run is a second full suite,
- the marginal cost of the checks inside the gate,
  because that needs the gate changed and a gate run,
  which is the owner's decision this document asks for,
- the oxlint count at the commit it went red,
  because that needs an install and a lint run at that commit.

### Where main moved while this was written

`main` reached `64e3eb1a55` during the round,
with the fixes for issues 6579 and 6580,
and the round merged it and re-ran the three checks once each in the same scopes.
That tree is the one this branch ends on,
and its figures are what the checks cost now:

- `test:unit` exits 0 in 19,365 ms with a peak of 1,819.0 MB.
  The memory drop from 4,541.9 MB is the fix's own effect:
  the planted-controls walk read 160,599 source paths and joined them,
  and it now reads each file once.
  The wall time also fell,
  from 22,166 ms,
  and the host's load average had fallen from 17 to 27 at the first runs to about 10.6 at these,
  so the time difference is not attributed to the fix.
- `lint:oxlint` still exits 1 with the same 850 errors and 1,856 warnings,
  in 8,378 ms with a peak of 939.8 MB.
- `lint:types` exits 0 in 1,996 ms with a peak of 413.1 MB,
  on a tree that gained `bin/source-tree-files.ts` and its unit test.

The three cost 29.7 s run one after another at `64e3eb1a55`,
and the memory a step needs there is 1.9 GB rather than 4.5 GB.

## When each red check went red, and how long it stayed unseen

Each red is dated to the first commit on `main`'s first-parent chain whose content fails the check,
and the time unseen runs from that commit to the issue's creation
(2026-11-27T12:52:10Z,
which is 07:52:10-05:00).

### The service-words unit test, issue 6579

The test read the line `pub const SERVICE_WORDS: [&str; 3] = [`,
which is line 77 of `package/git-policy/cli/e2e/service-mode-words.unit.test.ts` at `1c8acd5bfb`,
and `14053c4fd5` (2026-11-26T05:10:52-05:00,
"refactor(git-policy-cli):
hold every list of names,
sites and words as a slice")
changed the declaration to a slice
(`git log -S'pub const SERVICE_WORDS: &[&str]'`):

```text
-pub const SERVICE_WORDS: [&str; 3] = [COMMAND_WORD.current, "service", "run"];
+pub const SERVICE_WORDS: &[&str] = &[COMMAND_WORD.current, "service", "run"];
```

The test has compared nothing since,
and it went red at that commit.
The fix that landed with `afa9691f87`,
merged in `9f7d1a04e3`,
reads the declaration as the tree writes it now
(`SERVICE_WORDS_DECLARATION` at `package/git-policy/cli/e2e/service-mode-words.unit.test.ts:42`),
which is why the account above is written of the tree that was red.
It stayed unseen for 1 day 2 hours 41 minutes,
across 251 merges
(`git rev-list --first-parent --merges 14053c4fd5..1c8acd5bfb --count`).

### The planted-controls unit test, issue 6580

The walk is older than the failure.
`dc020315a8` (2026-11-03) wrote the recursive read
(`readdir(join(packageDirectory, 'src', 'native'), { recursive: true })`,
line 33 of `package/git-policy/cli/bin/native-planted-controls.unit.test.ts` at `1c8acd5bfb`),
which follows the symlinked `uses/` folders the layout rules put in the source tree.
`583f1af407` (2026-11-19T21:35:02-05:00) added the first `uses/` link under `src/native`,
and the walk then read the same file once per path that reaches it.
The merged fix replaced that call with one walk of `bin/source-tree-files.ts`,
imported at line 17 of the same test file at `64e3eb1a55`.
The test went red at the first commit whose reachable text passed the runtime's longest string,
which a probe of the same call fixed at `727721cc5a`
(2026-11-25T07:43:15-05:00,
"Merge branch 'feat/git-policy-cli-zfs-step-16-destruction-and-settle-behind-the-cut'").
The probe is the test's own three calls over one root,
run over the package's `src/native` as `git archive` wrote it at each commit:

```ts
// The walk bin/native-planted-controls.unit.test.ts makes, as a probe of one source root.
const names = (await readdir(root, { recursive: true })).filter((name) => name.endsWith('.rs'));
const text = (await Promise.all(names.map(async (name) => await readFile(join(root, name), 'utf8')))).join('\n');
```

```text
PROBE .../6b525b4290.../src/native rs_names=48500 joined_chars=531194252 result=joined
PROBE .../727721cc5a.../src/native result=RangeError: Invalid string length
PROBE .../pi-issue-6584-proposal-where-the-typescript-checks-run/package/git-policy/cli/src/native result=RangeError: Invalid string length
```

The first line is the merge's parent,
the second is the merge,
and the third is the worktree as it stands,
which is the positive control that the probe can show the failure.
It stayed unseen for 2 days 0 hours 8 minutes,
across 350 merges.

### The package lint, issue 6581

The first file that carries oxlint errors today arrived unchanged on 2026-11-13.
`751e2b0775` (2026-11-13T22:53:16-05:00) added `bin/service-manager-install-proof.ts`,
which has no later commit,
and that file alone reports 56 warnings and 41 errors
(`mise run //package/git-policy/cli:lint:oxlint:paths -- bin/service-manager-install-proof.ts`,
`Found 56 warnings and 41 errors.`).
The settings it breaks have been errors since before that date:
`git log -1 -S"'no-restricted-syntax/no-sync': 'error'" -- package/config/oxlint` answers with the package rename `ece5b7553f` of 2026-07-15,
which is the newest commit that changed the count of any of those six declarations,
so none of them changed after the file arrived.
The bulk came later,
with the eleven ZFS guest scripts from 2026-11-20 and the two `perf/` scripts of 2026-11-21
(`git log --diff-filter=A --reverse -- 'package/git-policy/cli/e2e/*guest*.ts'`).
The issue's own record names the last commit that says oxlint is clean as `cbb5b463e8` of 2026-11-04.
So the lint has been red for at least 13 days 8 hours 58 minutes,
across 605 merges,
and possibly from 2026-11-04,
which would make it 23 days.
Pinning the exact commit needs an oxlint run at each of those commits,
which this round did not do.

## The options

Seven placements.
The issue names four of them:
inside the gate is A,
a step of its own beside it is B,
the package's `lint` and `test` tasks run on `main` per batch is C,
and CI is D and E,
whose difference is the budget in front of D's job and the workflow of E's own.
F is a placement the issue does not name,
the round that changes the package before its branch merges,
and G is what comparable repositories do.
Each is designed until its problems show,
and each has its cost stated,
because effort is a cost and not a reason to rank a placement lower.

### A: inside the release gate, as a third check beside the suite and Clippy

`bin/test-native-container.ts` would gain a third member beside `testRun` and `clippyRun`,
running the three `mise` tasks on the host inside the gate's unit,
which already carries its own caps and its own log,
because the gate's image has no workspace install to run them in.
`bin/native-gate-checks.ts` would judge three checks and name the failing one,
`bin/native-ts-checks.ts` would be the natural runner for the three tasks,
and the gate's refusal would quote what the check printed.

Pros:

- The gate is the check the batch rests on,
  and its verdict is what a release waits for,
  so nobody has to remember anything:
  the step runs whenever a batch's gate runs.
- A refusal already names the check that failed,
  so a red arrives with its own evidence in the gate log and in `native:gate:verdict`,
  beside the suite's summary the same person is reading.
- The gate's unit already holds ten CPUs and 20 GiB
  (`defaultGateCaps` in `bin/native-gate-caps.ts:54`,
  whose `unitCpuQuota` is `1000%` and whose `unitMemoryMax` is `20G`),
  which is more room than `test:unit`'s peak needs,
  4.5 GB at `1c8acd5bfb` and 1.8 GB at `64e3eb1a55`,
  and that unit's own peak was 1,017 MiB of it
  (`doc/planning/git-policy-cli-cachyos-test-containers.md:1093`).
- The whole placement is one package's own code and one task,
  and the gate is already the place a person looks after a batch.

Cons:

- It changes what the gate runs,
  which the issue makes the owner's decision.
- A red TypeScript check turns the batch's gate red,
  and a red gate costs a whole gate:
  20 minutes to 50 minutes of wall time plus its queue,
  for a failure that the batch could have been told about before the gate started.
- The gate worktree has to hold a workspace install.
  `native:gate:main-tip` only detaches an existing gate worktree at `main`'s tip
  (`git checkout --detach` in `bin/native-gate-main-tip.ts:184`),
  so the install it carries is the one its own `git worktree add` copied,
  and a merge that moves `pnpm-lock.yaml` leaves it behind until someone updates that worktree.
  A gate whose install is stale or missing would then refuse for an infrastructure reason that reads like a product failure.
- The TypeScript failure and the Rust failure now share one verdict,
  so a release that is waiting reads a red gate without knowing which kind of work to start.

What a person sees when it goes red:
the gate's log ends `gate-exit-status=2` with a refusal that quotes the check,
such as `the gate's checks failed: the TypeScript checks printed Found 1856 warnings and 850 errors.`,
and `mise run //package/git-policy/cli:native:gate:verdict -- <log> <deadline>` prints the same refusal.

### B: beside the gate, in its own unit started with it

A task starts `native:ts-checks` in a capped user unit at the same moment the gate's unit starts,
and a verdict reader reads both logs' answers.
No file of the gate changes.

Pros:

- The gate's contents stay exactly as the owner recorded them,
  so nothing here needs the owner's decision on what the gate runs.
- The two run beside each other,
  so the 37.8 s costs no gate wall time at all,
  and the gate's own 4 CPU containers are untouched by it.
- It reuses `native:ts-checks`,
  which already runs the unit tests and the type check in capped scopes of its own,
  prints one line per check and exits 2 on any failure.

Cons:

- Two units and two verdicts,
  and nothing ties them together.
  The batch is declared green by a person reading the gate's answer,
  and the second unit's answer is read only if that person also asks for it,
  which is the shape of the failure this issue records.
- Two things start from one action,
  so a launcher that starts only the gate (as `native:gate:main-tip` does today) leaves the second unstarted,
  and nothing says so.- It puts a second 4.5 GB peak on the host while the gate compiles,
  which on this host is the kind of pressure the gate's own controls are sensitive to,
  and 1.8 GB once issue 6580's fix is in the tree.

What a person sees when it goes red:
two verdict lines in one note,
one `gate-exit-status=0` and one `checks failed: //package/git-policy/cli:lint:oxlint`,
with the second one easy to miss if only the first is read.

### C: at the coordinator's batch tip check, through `native:ts-checks`

`native:ts-checks` already exists
(`package/git-policy/cli/mise.toml:81`),
and its default task list is two of the three checks
(`bin/native-ts-checks.ts:55`):

```text
export const recordedCheckTasks: readonly string[] = [
  '//package/git-policy/cli:test:unit',
  '//package/git-policy/cli:lint:types',
];
```

The placement adds `//package/git-policy/cli:lint:oxlint` to that list,
and the handover's batch tip check gains one command:
`mise run //package/git-policy/cli:native:ts-checks`.

Pros:

- It is the smallest change of the seven:
  one line in a task list and one line in the batch routine.
- It runs sooner than the gate does,
  because the tip check runs as soon as the merges are made,
  while the gate takes 20 minutes to 50 minutes and is often queued.
- It catches everything a batch landed,
  including a break that came from merging two branches that were each fine,
  which a round's own check cannot.
- `native:ts-checks` already prints one line per check,
  names a failing check's lines,
  bounds each check with its own deadline,
  and exits 2 on any failure,
  so a batch's note carries the answer without a person reading a log.
- Its memory cost lands on the coordinator's own moment,
  not on the gate's containers.

Cons:

- It is a routine rather than a gate:
  nothing fails if the batch's check is skipped,
  and the routine's own record is a handover paragraph,
  which is exactly the kind of record that went stale here.
- It runs after the merge,
  so the red is on `main` before anyone knows.
- It uses the host's own toolchain and install,
  so a red can come from a broken local install rather than from the code,
  which the gate's image rules out.

What a person sees when it goes red:
`native:ts-checks` prints
`failed //package/git-policy/cli:lint:oxlint: Found 1856 warnings and 850 errors.` beside its twelve failure lines,
exits 2,
and the batch's note has that line in it.

### D: in the existing advisory CI workflow's fast subset

`.github/workflows/advisory-core.yml` gains three `task-run` lines
beside the six packages it already runs
(`:224`),
and its summary line names the package.

Pros:

- Every push to `main` and every pull request is offered the run,
  so the red lands on the commit itself,
  with no one deciding to run anything.
- The job already installs the workspace,
  sets up the pinned Node and pnpm and uses `task-run`,
  so the three checks are three lines in a step that exists.
- The runner has no other agent's work on it,
  so a red there is the code's and not the host's.

Cons:

- The workflow's own window budgets its push runs at 2 per minute,
  4 per hour and 8 per day,
  and a run past the budget skips its work.
  Measured at the time of writing,
  the newest push run skipped the job,
  so at this merge rate most pushes would never run the checks.
- Its summary says the results are advisory with no required checks,
  and this project merges locally and by hand,
  so a red CI run on `main` is evidence that nothing in the loop reads CI:
  `forbidden-strings` failed on `main` on the last three pushes
  (`gh run list --limit 20`).
- The step reports the failures of six packages and the repository's own tools in one job,
  so a red is one item in a 45 minute budget's report,
  not a named answer about this package.

What a person sees when it goes red:
a red `advisory-core` run whose `Unit fast subset` step ends with
`//package/git-policy/cli:lint:oxlint` failing,
on the eight or fewer pushes a day that the window admitted.

### E: in a CI workflow of its own, path-filtered like the fuzz workflow

A new workflow,
modelled on `.github/workflows/git-policy-cli-fuzz.yml`,
filtered to `package/git-policy/cli/**`,
`pnpm-lock.yaml`,
`pnpm-workspace.yaml`,
the root `tsconfig.json`,
`package/config/oxlint/**` and its own file,
running on `push` to `main` and in the merge queue,
with `install-dependencies: true` in `task-cli-setup`,
and three steps:
`task-run //package/git-policy/cli:lint:types`,
`task-run //package/git-policy/cli:lint:oxlint`,
`task-run //package/git-policy/cli:test:unit`.

Pros:

- Every push to `main` runs it,
  path-filtered,
  with no budget:
  the fuzz workflow in that shape ran on every push this round saw,
  and its recent runs took 3.3 to 4.8 minutes.
- Its own workflow means its own check mark,
  so a red is one named job rather than an item in someone else's summary,
  and it can be made a required check later without touching anything else.
- It reports before a release and beside the merge,
  and the job's log keeps the whole failure text.

Cons:

- It needs a workspace install the fuzz workflow avoids
  (`install-dependencies: false` there),
  and a runner pays about 15 s for that install beside 26 s of checkout,
  measured above from run `45754658990`,
  so the job's fixed cost is about 40 s before the three checks start.
  What those three cost there,
  on four virtual cores and cold caches,
  is not this host's figure.
- Nothing on this project's loop reads a push's CI result,
  which the three red `forbidden-strings` runs on `main` show.
- CI's answer is advisory unless the owner sets a required check,
  and a required check would hold the merge the project's own record says nothing holds.
- It duplicates the authority of the gate:
  two places now claim to say whether the package is well.

What a person sees when it goes red:
a red `git-policy-cli-typescript` job on the commit that was just pushed,
whose log ends `Found 1856 warnings and 850 errors.`

### F: in every round that changes this package, before its branch merges

The package guide's "A round runs no whole container gate" section gains one more check:
a round that touches `bin/`,
`e2e/` or `perf/` runs
`mise run //package/git-policy/cli:native:ts-checks -- //package/git-policy/cli:lint:oxlint`
and names the result in its report,
beside the modules it ran and `source_guard`.

Pros:

- It is the earliest possible answer:
  the branch that breaks a check learns it before it merges,
  from the agent that changed the code.
- It costs about 38 s inside a round that runs for hours,
  and it needs no gate,
  no workflow and no coordinator.
- It puts the check in the same contract that already makes a round run its Rust modules,
  so a round that skips it has skipped a stated rule,
  not an unwritten one.

Cons:

- It depends on every round's brief and guide being read and obeyed,
  and a round is the actor that is watching its own work,
  which is the weakest form of "seen by someone who was not watching".
- It cannot see a break that two branches make together,
  and the two red checks here were both made by one branch each,
  so it would have caught both.
- It leaves `main` unguarded for anything merged without a round,
  such as a toolchain bump or a hand edit.

What a person sees when it goes red:
the round's own report has a line naming `native:ts-checks` and the failing check,
written before the branch merges,
and a fix in the same branch.

### G: what comparable repositories do, two path-filtered CI workflows with required checks

Repositories that keep a Rust crate beside its TypeScript tooling in one repository run both toolchains from CI on every pull request,
in one workflow per toolchain,
with the JavaScript side path-filtered and its job required.
Biome is the closest shape:
`pull_request.yml` runs `cargo lint`,
`cargo check` and `cargo nextest run --workspace --no-fail-fast`
(`.github/workflows/pull_request.yml` in `biomejs/biome`),
and `pull_request_js.yml` runs the JavaScript side,
filtered to
`packages/@biomejs/**`,
`package.json` and `pnpm-lock.yaml`,
on `pull_request` and in the merge queue
(`.github/workflows/pull_request_js.yml` in `biomejs/biome`).
This repository has the same shape available:
`git-policy-cli-fuzz.yml` is exactly one path-filtered workflow per subject.

Pros:

- It is the shape that exists in the field for this combination,
  and the checks become a condition of the merge rather than a note after it.
- Each toolchain gets its own job,
  its own red and its own required check,
  so a red says which kind of work broke.

Cons:

- It needs pull requests to be the way branches merge.
  This project merges locally and pushes,
  so a `pull_request` job has not run since 2026-10-19,
  and adopting the shape means changing the merge bar the owner recorded
  (`doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md`).
- It needs required checks and branch protection,
  which that same record says the project does not have.
- It inherits both of E's costs:
  the workspace install and a second place that claims authority over the package's health.

What a person sees when it goes red:
the pull request shows the `pull_request_js` job red,
a required check blocks the merge button,
and the author's next push to the branch has to fix it.

## Ranking

The order is by how soon a red is seen and how surely it is seen by someone who was not watching.
A:
inside the gate.
C:
at the batch tip check.
B:
a unit beside the gate.
E:
its own CI workflow.
D:
the advisory CI fast subset.
F:
in the round.
G:
the comparable repositories' shape.
The ranking is A > C > B > E > D > F > G.

- A over C:
  both answer on `main` after the merges,
  and C answers sooner and for 38 s rather than for a gate's refusal.
  A ranks higher because it is the answer the batch's own record makes binding:
  the gate is what a release waits for and what the batch rests on,
  so its step cannot lapse without a person noticing a missing gate,
  while C is a routine whose omission is silent,
  which is the failure this issue records.
- C over B:
  C's three checks run in the batch tip check the coordinator already runs,
  so it adds one command to a step that happens anyway and its answer arrives before the gate starts.
  B runs at the same moment as the gate and needs a second verdict read,
  with nothing tying the two together,
  so it is later than C and less sure than A.
- B over E:
  B's answer is read where the batch's green is declared,
  while E's answer lands on a surface this project demonstrably does not read:
  `forbidden-strings` failed on `main` on the last three pushes,
  and the advisory workflow's own job was skipped on the newest push it was offered.
  E is not later in principle,
  but on this repository it is less surely seen.
- E over D:
  both are CI on every push,
  and E runs every time its path filter matches while D runs only when the advisory window admits the push,
  which was 8 runs a day against a merge rate of hundreds,
  so D would miss most merges entirely.
- D over F:
  D reports without anyone deciding to run it,
  while F needs each round's brief and guide to be obeyed,
  and F cannot see a break that two branches make together.
- F over G:
  F runs today,
  on every branch a round makes,
  and needs no merge-process change,
  while G cannot run at all until branches merge through pull requests with required checks,
  which the owner's recorded merge bar rules out.

Effort,
stated as a cost and not as a ranking reason:

- A is a third member in the gate's check pair,
  a refusal line,
  and a decision by the owner about what the gate runs.
- C is one line in `recordedCheckTasks` and one line in the batch routine's record.
- B is a launcher that starts two units and a reader that prints two verdicts.
- E is a workflow file of about sixty lines,
  modelled on the fuzz workflow,
  plus the install it does not currently need.
- D is three `task-run` lines and one summary line.
- F is a paragraph in the package guide and the line in the briefs that quote it.
- G is E plus a change to how branches merge.

## What this does not settle

The proposal is not the fix.
At `1c8acd5bfb` two of the three checks were red.
At the tree this branch ends on,
issue 6581 alone keeps the lint red with 850 errors while the unit tests pass,
and every placement above makes that redness visible,
which is the point,
but a gate or a required check that lands while the lint is red refuses every batch and every merge until that issue is fixed.
That ordering is one of the questions below.

## Questions only the owner can answer

1. Which placement,
   A,
   C,
   B,
   E,
   D,
   F or G?
   This round ranks A > C > B > E > D > F > G.
2. If A,
   what does the gate run?
   The issue records the gate's steps as the owner's,
   so a third check beside the suite and Clippy is the owner's decision,
   including whether a TypeScript failure may turn a batch's gate red.
3. Should a red check hold a merge?
   That decides whether CI is a signal or a gate,
   and it needs a required check and branch protection,
   which the recorded merge bar says the project does not have.
4. Does the placement land before or after issue 6581 is fixed,
   and before or after the removals issue 6581 asks to weigh?
   Landing it first makes every batch's gate red for a reason unrelated to the batch's Rust work,
   and landing it after leaves the lint unrun for however long that takes.
5. What memory and CPU bound does the step get wherever it runs?
   `test:unit` peaked at 4.5 GB at `1c8acd5bfb` and 1.8 GB at `64e3eb1a55`,
   so the bound has to hold the first figure,
   which is twice the suite container's 2 GiB and inside the gate unit's 20 GiB.
6. Should `native:ts-checks` grow `lint:oxlint` in its default list?
   C and F both rest on that,
   and it is one line.
7. Is the eleven ZFS guest scripts' and two `perf/` scripts' oxlint debt to be fixed in place,
   or is some of that code done,
   so that issue 6581's removal question changes how much the lint has to catch?
