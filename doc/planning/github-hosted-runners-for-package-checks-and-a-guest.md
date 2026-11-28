# GitHub's hosted runners for the package checks, and a guest trial

Status:
a proposal for the repository owner,
unapproved and non-final.
Nothing here is adopted,
no workflow file is added,
no required check changes,
and no review from another vendor than the writer's has read it.

Written by a Pi run on 2026-11-28 in the worktree
`.claude/worktrees/pi-hosted-runners-proposal-revised-against-its-gate-read`,
on the branch `docs/planning-hosted-runners-proposal-revised-against-its-gate-read`,
from `main` at `5a84e361c0`,
with `origin/main` re-read at `9afd7194ab` before this revision ended.

Gate reads of this proposal:
one,
a same-vendor second pass by the vendor of the draft's writer and not an independent review.
It was made on 2026-11-28 and returned five High,
nine Medium and seven Low findings,
and it is kept outside the repository,
where it expires a day after its last change,
so its findings are answered here rather than linked,
under "What the gate read found,
and what changed".
A finding this round found wrong is named there with the command or the line that shows it.

What is measured says so and names the run,
the command or the file it came from.
What rests on another record says which,
and a record that is withdrawn is named as withdrawn where it is used.

## Why the owner is being asked

The owner was offered a paid provider's trial,
wrote "Don't we already have gh?",
and the record of that answer says what follows from it:
no Hetzner trial is authorised,
and "GitHub's hosted runners,
which the repository already has,
come first"
([the owner's answers](decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md),
"Whether the machine and the repository are set up right").
In the same turn the owner offered resources:
"If you think me giving more resources to you,
hardware,
or through a provider,
would make you work better,
tell me."

The record says which provider comes first.
It does not say what the hosted runners are for.
The aim this proposal is written against is this round's reading of two other records,
and is stated as an aim and not as a decision:
the machine ran out of memory on the night of 2026-11-27 with nineteen rounds and ten guests alive
([that answer](decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md),
"The rounds that were running,
and what bounds a later batch";
[the troubleshooting record](troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md)),
and the owner bounded a later batch by the guests alive at once.
So work that competes for this machine's memory is the work this proposal would move,
and every option says what it would take off this machine,
so the owner can reject that aim by choosing the option that moves least.

## What runs today, measured

### The advisory workflow, read whole

`.github/workflows/advisory-core.yml` has two jobs and two concurrency groups.

- The `gate` job runs on `ubuntu-latest`,
  has `timeout-minutes: 5`,
  and its group is `advisory-core-gate` with `cancel-in-progress: false`
  (lines 18 to 21 of that file).
  The group names no ref,
  so gate jobs of every branch and every push wait behind one another.
- The `core` job has `needs: gate` and runs only when `needs.gate.outputs.allowed == 'true'`
  (lines 73 and 74).
- The workflow's own group is `advisory-core-${{ github.ref }}` with `cancel-in-progress: false`
  (lines 11 to 13).
- The gate reads `gh run list --workflow advisory-core.yml --limit 100 --json databaseId,createdAt`
  (line 46),
  and counts every one of those runs except the current one
  (line 49:
  `[ .[] | select(.databaseId != $current) | age ] as $ages`).
  It asks nothing about whether a run's `core` job ran.
- The three windows are printed by the same step,
  `- budgets: 2 per minute, 4 per hour, 8 per day` (line 61),
  and the comparison that refuses a push is
  `if [ "$minute" -ge 2 ] || [ "$hour" -ge 4 ] || [ "$day" -ge 8 ]` (line 63).

The `core` job is more than the fast subset.
Read whole,
its steps are:

- five drift gates,
  each `continue-on-error: true` (lines 99 to 189):
  committed file drift (`file-enforcer check --outputs committed`),
  toolchain lock drift (`mpm-shadow install --lockfile-only --frozen-lockfile`),
  local file drift (`file-enforcer check --outputs local` and `file-enforcer check`),
  pnpr config drift (`//package/config/pnpr:check:config`),
  and the Rust lockfile frozen install (`cargo-shadow install --frozen-lockfile`,
  `timeout-minutes: 25`);
- `Static changed paths` (line 190),
  which runs
  `task-run lint:monochromatic-lint -- <changed .md, .mdx and .rs files>`;
- `Build fast subset` (line 215),
  five tasks:
  `//package/logger/core:build`,
  `//package/upstream-fork/p-map:build`,
  `//package/toml-edit/core:build`,
  `//package/forbidden-strings/cli:build`,
  `//package/linter/monochromatic-lint:build`;
- `Unit fast subset` (line 224),
  six tasks:
  those five packages' test tasks plus `//package/container-limit/core:test:unit`;
- `Summary`,
  which names all six packages,
  and `Enforce advisory result` (line 247),
  which turns any failed step's `outcome` into a failed run.

So a repaired advisory profile is worth more than one build and one test per push:
it is the only thing on this repository that checks file drift,
toolchain lock drift,
pnpr drift and the Rust lockfile together,
and no other workflow does.

### What its concurrency group does, from GitHub's documentation

GitHub's own page says,
of one concurrency group:

> there can be at most one running job or workflow in a concurrency group at any time.
> When a
> concurrent job or workflow is queued,
> if another job or workflow using the same concurrency group
> in the repository is in progress,
> the queued job or workflow will be
> pending.
> By default,
> any existing pending job or workflow in the same concurrency group will be
> canceled and the new queued job or workflow will take its place.

and,
for the `queue` property:

> `single` (default):
> At most one job or workflow run can be pending in the concurrency group.
> When a
> new job or workflow run is queued,
> any existing pending job or workflow run in the same group is
> canceled and replaced.

([Control the concurrency of workflows and jobs](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency),
read on 2026-11-28.)

With `cancel-in-progress: false` and no `queue`,
a push to `main` therefore starts at most one `advisory-core` run beside the one already running,
and the next push replaces the waiting one instead of queueing behind it.
That behaviour is the one the gate's own commit intended:
"Keep cancel-in-progress false so the rate gate,
not concurrency,
decides admission;
pending runs still coalesce"
(`e4a40ea3c1`,
commit message read whole).

`advisory-core.yml` is the only workflow in this repository that runs on every push and keeps
`cancel-in-progress: false`.
Read from all 22 workflow files by one script that parses each file's `on`,
`concurrency` and job
blocks,
the per-push check workflows
(`forbidden-strings.yml`,
`git-policy-cli-fuzz.yml`,
`logger-fuzz.yml`,
`toml-edit-fuzz.yml`,
`catalog-specifiers.yml`,
`fs-id.yml`,
`readonly-semantic-bridge.yml`,
`env-local-json.yml`,
`codeql.yml`)
each group on `${{ github.workflow }}-${{ github.ref }}` with `cancel-in-progress: true`,
which is what the CI provider record asks for:
"Put `concurrency:` groups with `cancel-in-progress: true` on per-push workflows so a burst of
auto-pushes cancels stale runs instead of queueing them"
([the CI provider decision](decision/ci-provider.md),
"Scaling plan for fuzz and mutation workflows").
The workflows that must not race a release
(`cargo-publish.yml`,
`crate-release.yml`,
`ide-release.yml`,
`music-player-release.yml`,
`npm-release.yml`,
`kotlin-linter-publish.yml`,
`pnpr-publish.yml`,
`playwright-image.yml`,
`cache-prune.yml`,
`npm-publish.yml`) keep `cancel-in-progress: false` on fixed groups,
as publishing needs.

### The closed door, measured

`gh run list --workflow advisory-core.yml --limit 100` read at 2026-11-28T06:43:06-05:00
answered 100 runs,
every one a `push` event and every one a `success`,
spanning `2026-11-28T01:02:09Z` to `2026-11-28T11:35:42Z`,
which is 10 hours 34 minutes,
or about 226 runs a day.
A run that admitted the `core` job cannot conclude in that time:
the one executed `core` job this round measured lasted
from `2026-11-25T02:11:46Z` to `02:19:42Z`,
7 minutes 56 seconds
(`gh run view 45531170934 --json jobs`).
Every run in the sample therefore ran the gate and skipped the checks,
and at the newest of them the trailing day held far more than eight runs,
so the day window refused that push and every push after it.

The same reading appears in the linter's own record:
"Of the 400 most recent runs of the workflow (2026-11-04 to 2026-11-07),
394 skipped the `advisory core` job"
([keeping the repository linter at zero](planning/repository-linter-stays-at-zero.md),
"The CI step cannot run").
At about 226 push-driven runs a day the day window can never empty,
and the profile is not a slow lane but a closed door.

### The window counts events, and the model it was copied from counts executions

The withdrawn Depot adoption decision gave the advisory run its starting point:

> The default-branch advisory run uses the sliding-window rate limits currently used by the
> Scorecard workflow as the starting point,
> and adjusts from measured capacity.
> It runs a fast static,
> build and unit subset;
> the complete profile runs in the weekly matrix.

([the Depot CI adoption decision](decision/depot-ci-adoption.md),
"Execution routing";
the record opens with "withdrawn on 2026-11-01" and "This decision is no longer in force",
and is named withdrawn here wherever it is used.)

That paragraph describes the numbers and says nothing about a queue.
The reason the draft gave for the window,
that a burst of auto-pushed commits should not queue twenty-minute runs,
is not in that record,
not in the Scorecard commit,
and not anywhere in `doc/decision`,
`doc/planning` or `doc/handover`
under a search for a twenty-minute queue rationale:
the only hits for the twenty-minute wording are the Depot grilling's own job cap
and unrelated handovers.
It is this round's inference,
and it is the ground the draft's first question rested on,
so the first question now asks the owner what the window is for instead of asserting it.

Scorecard's gate counts something else,
and its source says how:

```javascript
// .github/workflows/scorecard.yml
const executed = checks.nodes.some(({ conclusion }) => conclusion !== 'SKIPPED' && conclusion !== 'CANCELLED');
```

It asks through GraphQL for 100 runs a page with each run's checks,
follows a cursor,
and refuses to admit when the history passes 32 pages
(`scorecard.yml`,
the `rate-limit` job,
lines 57 to 150).
The copy in `advisory-core.yml` reads `--json databaseId,createdAt` only,
so it counts a skipped run against the windows that skipped it,
and a repaired counter has to read job or check conclusions the way Scorecard's does.

### What pushes actually arrive

`git log origin/main --first-parent` read 250 commits over
`2026-11-27T12:48:41-05:00` to `2026-11-28T11:41:54+00:00`,
0.75 of a day,
which is about 335 commits a day
(the script and its output are this round's,
kept as a measurement,
not a record):

- the median gap between two commits is 139 seconds,
  the mean 259 seconds,
  and the longest is 8,296 seconds,
  2 hours 18 minutes;
- 246 of the 249 gaps fall under 21 minutes 28 seconds,
  98.8 percent;
- 42 of the newest 200 commits,
  21.0 percent,
  change something under `package/git-policy/cli`
  when each is diffed against the commit main pointed at before it,
  so a workflow started by a wrapper-touching push fires about 70 times a day.

## What a hosted runner is, from GitHub's documentation read today

### A public repository's standard runner, at no charge

- "Use of the standard GitHub-hosted runners is free and unlimited on public repositories."
  ([GitHub-hosted runners reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners),
  "Standard GitHub-hosted runners for public repositories")
- The same table gives `ubuntu-latest`,
  `ubuntu-24.04`,
  `ubuntu-22.04` and `ubuntu-26.04`
  as Linux,
  x64,
  "4" processors,
  "16 GB" memory and "14 GB" SSD.
- The repository is public and has 6 forks
  (`gh repo view --json visibility,forkCount`,
  read 2026-11-28T06:43:06-05:00).
- `ubuntu-slim` is 1 processor,
  5 GB and 14 GB,
  its job timeout is 15 minutes,
  and its container "runs in unprivileged mode",
  so it cannot mount a file system.

### Concurrency, length and drop

- "Standard GitHub-hosted runner | Free | 20 | 5 | Not applicable"
  ([Actions limits](https://docs.github.com/en/actions/reference/limits),
  "Job concurrency limits for GitHub-hosted runners").
- "Each job in a workflow can run for up to 6 hours of execution time."
- "Workflow run time | 35 days / workflow run".
- The queue caps are "1500 events / 10 seconds / repository" and "500 workflow runs / 10 seconds",
  and the group cap is "100 workflow runs / concurrency group",
  reached only with `queue: max`.
- The 45-minute sentence sits under "Workflow continuity",
  whose first sentence is about GitHub Actions services being unavailable:
  "In addition,
  if the workflow run has been successfully queued,
  but has not been processed by a
  GitHub-hosted runner within 45 minutes,
  then the queued workflow run is discarded."
  ([GitHub-hosted runners](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners),
  "Workflow continuity").
  Whether it also drops a run that is waiting for a free slot under load is not stated on that page,
  and nothing here rests on it.

### Which hardware a job lands on

Standard 2-core and 4-core Linux jobs run on "a multi-cloud compute pool",
on Amazon Web Services "or Microsoft Azure",
and "Jobs are automatically scheduled across available cloud providers"
(the same concepts page,
"Cloud hosts used by GitHub-hosted runners").
So a runner's region,
processor vendor and clock differ from job to job,
and the trial landed on three regions and two vendors.

### Nested guests, and what GitHub promises about them

- "GitHub-hosted Linux runners support hardware acceleration for Android SDK tools,
  which makes running Android tests much faster and consumes fewer minutes"
  (the runners reference,
  "Supported runners and hardware resources").
- "While nested virtualization is technically possible while using runners,
  it is not officially supported.
  Any use of nested VMs is experimental and done at your own risk,
  we offer no guarantees regarding stability,
  performance,
  or compatibility."
  (the concepts page,
  "Preinstalled software for GitHub-owned images").

The draft said the documentation states no promise of the device.
That is wrong as written:
the reference page names Android SDK acceleration as a supported use.
What no page promises is `/dev/kvm`'s permission model,
the ACL grant,
or a general-purpose nested VM.

### Larger runners, and the repository's own settings that bound any new workflow

- "Larger runners are available for organizations and enterprises on GitHub Team and GitHub
  Enterprise Cloud plans"
  ([GitHub-hosted runners reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners),
  "Larger runners").
  This repository's owner is on neither plan for this repository's benefit,
  so a larger runner is not a free path here,
  and nothing in this proposal needs one.
- The repository's Actions settings,
  read with `gh api` at 2026-11-28T06:43:06-05:00:
  `allowed_actions: selected` with `sha_pinning_required: true` and the patterns
  `docker/*`,
  `jdx/*`,
  `actions/*`,
  `changesets/*` plus verified creators;
  default workflow token permission `read`;
  `can_approve_pull_request_reviews: false`;
  11 repository secret names,
  no values read;
  and one active ruleset,
  `Copilot review`,
  on all branches.
- A workflow triggered from a fork gets no repository secrets:
  "With the exception of GITHUB_TOKEN,
  secrets are not passed to the runner when a workflow is
  triggered from a forked repository.
  The GITHUB_TOKEN has read-only permissions in pull requests
  from forked repositories."
  ([Events that trigger workflows](https://docs.github.com/en/actions/reference/events-that-trigger-workflows),
  "Workflows in forked repositories").
- Docker Hub's rate limit does not apply to a hosted runner pulling a public image
  ("GitHub-hosted runners pulling public images:
  Docker Hub's rate limit is not applied",
  the limits page,
  "Docker Hub's rate limit for GitHub Actions"),
  so the gate's base-image pull is a network cost and not a quota risk.

Any new workflow here therefore pins its actions by full SHA from that allow-list,
takes the read-only token unless a job asks for more,
and runs pull-request code from a fork with no secret in reach.

## What the checks need, measured

### Three package unit tasks, run today

Each in its own `systemd-run --user --scope` on `agent-work.slice`
(`MemoryMax=6G`,
`CPUQuota=400%`,
`TasksMax=512`,
`IOWeight=1`,
`RuntimeMaxSec=300`),
one run each,
2026-11-28T06:45-05:00,
on a clean worktree at `origin/main`:

- `mise run //package/logger/core:test:unit` exited 1,
  five test files failed:
  `record-buffer.unit.test.ts`,
  `local-storage.unit.test.ts`,
  `opfs.unit.test.ts`,
  `file-batch.unit.test.ts`,
  `create-logger.unit.test.ts`.
- `mise run //package/toml-edit/core:test:unit` exited 1,
  two files failed:
  `value-encoders.unit.test.ts` and `wrapped-value-validation.unit.test.ts`,
  on `k = 0.0` where `k = -0.0` was expected.
- `mise run //package/container-limit/core:test:unit` exited 1,
  one file failed:
  `container-start-sites.unit.test.ts`,
  whose ledger lists every file that names `podman`
  and has not been taught `package/mvm/cli/src/backend/ovh/images.ts`.

Each finished in seconds and each is red on `main` today.
All three were run again after `origin/main` was merged at `471d462ccf`,
and all three are red there too,
failing the same files.
The third is the same shape as issue 6588,
which was closed on 2026-11-27 for one missing entry
([issue 6588](https://github.com/Aquaticats/Monochromatic/issues/6588)):
nothing on this repository's loop runs these tests after a merge,
so the ledger fell behind again within a day.
This round changes nothing that ships,
so the red is recorded here and not fixed.

### The wrapper's three TypeScript checks

Measured in the placement record for issue 6584,
in the same scope shape,
at `1c8acd5bfb`,
on a ten-CPU machine with 16 GiB available and a load average of 17 to 27
([where the TypeScript checks run](planning/git-policy-cli-where-the-typescript-unit-tests-and-lint-run.md),
"What was measured"):

- `lint:types` exited 0 in 2,810 ms and 2,639 ms,
  peaks of 452.9 MB and 447.1 MB;
- `lint:oxlint` exited 1 in 12,791 ms,
  a peak of 987.1 MB,
  printing `Found 1856 warnings and 850 errors.` and
  `Finished in 10.0s on 242 files with 491 rules using 4 threads.`;
- `test:unit` exited 1 in 22,166 ms,
  a peak of 4,541.9 MB,
  with `e2e/service-mode-words.unit.test.ts` and `bin/native-planted-controls.unit.test.ts` failing.

Those are the record's figures and its date,
not this round's.
What they show for a hosted step is the memory a step needs:
4.5 GB at that commit,
and 1.8 GB after issue 6580's fix
(the same record,
"What this does not settle",
question 5).

### The container gate, read from its own source

The gate is `mise run //package/git-policy/cli:native:test:container`
(`package/git-policy/cli/mise.toml`),
whose first step is `cargo-shadow lean records target/verification` and whose second is
`node bin/test-native-container.ts`.

- The image build is awaited,
  and it runs with `--network=none --http-proxy=false --pull=never`
  (`test-native-container.ts`,
  the `podman build` call),
  so the base image must already be pulled;
  `native-test-base.ts` pulls `docker.io/cachyos/cachyos-v4:latest` on every run.
- The suite and Clippy then start together and the gate waits for both
  (the same file,
  the comment on `runGateChecks`).
- The containers run as `user: '1000:1000'` (the same file),
  against a runner user of 1001.
- The suite's memory is read from the container's own cgroup under `/sys/fs/cgroup`,
  the `libpod-*.scope` podman makes (`native-gate-container-cgroup-memory.ts`).
- The containers get the host's `/usr/share/containers/seccomp.json` with `listmount` and
  `statmount` added to the rule that names `mount`
  (`native-gate-syscall-profile.unit.test.ts` and the gate log quoted in
  "The container gate,
  read from its own source").
- The bounds,
  with the measurements that size them,
  are in `native-gate-container-limits.ts`:
  `gateContainerCpus = 4` (which is also the test-thread count and the compile job count),
  `gateBuildMemory = '8g'` against a measured build compile peak of 5,363.4 MiB,
  `gateClippyMemory = '8g'` against Clippy peaks up to 5,278.7 MiB,
  and `gateSuiteMemory = '4g'` against a suite peak of 1,486.0 MiB,
  with 256 processes for each test thread,
  so 1024 at four threads.
- The gate runs as its own capped systemd user unit (`native:gate`),
  and
  `native:gate:main-tip` detaches a worktree at `main`'s tip and gates there.
- On this host the gate's own unit holds 10 CPUs and 20 GiB,
  and this host has 6 CPUs (`nproc`).

The newest gate this machine ran,
at `2026-11-28T05:39:53-05:00`,
labelled `gpc-gate-wrapper-next-alpha-on-d4faaf0807-20261128T053953-0500`,
printed

```text
Summary [1104.852s] 12517 tests run: 12517 passed (6 slow), 51 skipped
Clippy passed beside the test run.
gate-exit-status=0
```

so the suite's own wall time is 1,104.9 seconds,
18.4 minutes,
at four test threads,
and the gate around it takes about 21 minutes,
which is the figure the records already carry.
That log expires a day after its last write;
a reader can reproduce it with `mise run //package/git-policy/cli:native:gate:main-tip`.
The draft printed `Summary [1053.402s] 9173 tests run: 8988 passed (8 slow), 185 failed, 28 skipped`,
which is a run from 2026-11-19 before the gate's fixes;
the 185 failures were the host's ZFS mount behaviours,
and the gate now makes a tmpfs fixture root on every host,
so neither figure describes the gate today.
The draft's "What the run measured" heading does not exist in the gate record,
which is why this section cites the source that decides each number instead.

### The gate's memory fits a runner, its time on a runner is unmeasured

The three phases do not overlap:
the build is awaited,
then the suite and Clippy run together.
The peaks the gate's own bounds are sized from therefore give two figures:

- build alone,
  5,363.4 MiB,
  about 5.2 GiB;
- Clippy and suite together,
  5,278.7 + 1,486.0 = 6,764.7 MiB,
  about 6.6 GiB.

The trial measured 15,482 MiB available at the start of a runner's job,
so 6.6 GiB fits with about 8 GiB to spare.
The draft's "4 GB plus 8 GB plus 8 GB does not fit" added caps rather than peaks,
and its "8 GB compile that issue 6455 measured pressing its cap" was wrong:
issue 6455 measured the old 2 GiB cap,
and 8g is the fix.
What is not measured is time:
this host has 6 CPUs and the gate's containers take 4,
so the 21 minutes that section measures is a four-CPU figure,
but the runner's four vCPUs also carry the image build,
podman and every shell step of the gate,
and no gate has run on a runner.
A 21-to-35-minute gate on a runner is this round's estimate,
not a measurement.

The disk is the other unmeasured fit.
The gate's own image is 4.33 GB and its base 2.23 GB
(`podman images --format '{{.Repository}}:{{.Tag}} {{.Size}}'`),
and the documentation says a standard runner has a 14 GB SSD,
while the trial measured `145G` with `87G` available.
A fresh runner builds the image from scratch and has no such image cached,
so the image plus the crate's target directory share 14 GB unless the free-disk step
`ide-release.yml` already uses is copied.
No record shows the gate's full disk footprint.

### What a hosted runner refuses: the disk weight

The gate's containers are started with a block-layer weight:
`diskWeightArguments()` returns `['--blkio-weight=10', '--cgroup-conf=io.weight=10']`,
and returns nothing when `CONTAINER_LIMIT_DISK_WEIGHT=off`
(`package/container-limit/core/src/index.ts`,
`DISK_WEIGHT_SWITCH` and `diskWeightArguments`).
The gate reaches that function through `package/git-policy/cli/bin/container-limits.ts`,
which re-exports `package/mutation-test/cli-cargo-mutants/src/container-limits.ts`.

A hosted runner does not delegate the cgroup io controller.
The repository records the refusal and its cure:

```yaml
# .github/workflows/ide-release.yml, lines 215 to 220
# There is no disk weight in this job: hosted runners do not delegate
# the cgroup io controller, and run 42447009237 of music-player-release.yml (2026-11-08), on the same runner images, failed with
# `crun: opening file 'io.weight' for writing: No such file or directory`. CONTAINER_LIMIT_DISK_WEIGHT=off tells
# package/container-limit/core (read by package/ide/app/bin/container-bounds.ts and the task bodies) to leave the weight out.
CONTAINER_EXTRA_OPTIONS: '--pids-limit=1024'
CONTAINER_LIMIT_DISK_WEIGHT: 'off'
```

GitHub's own documentation does not mention this:
a search of `docs.github.com` for `io.weight` on 2026-11-28 returned "0 Search results".
The evidence is therefore the repository's own failed run,
and the writer of the failure is crun,
whose manual maps OCI `weight` onto `io.weight`
and which fails when the file is absent.

This round measured what the switch does to the gate's own arguments,
by importing the gate's modules and printing them
(2026-11-28T06:48:27-05:00,
one run each):

- with `CONTAINER_LIMIT_DISK_WEIGHT=off`:
  `["--memory=4g","--cpus=4","--pids-limit=1024"]`;
- with the variable absent:
  `["--memory=4g","--cpus=4","--pids-limit=1024","--blkio-weight=10","--cgroup-conf=io.weight=10"]`.

So a hosted gate needs `CONTAINER_LIMIT_DISK_WEIGHT=off` in the job's environment,
and the gate honours it without a code change.
The same workflow records what else is unproven on a runner image:
`--pids-limit` and `--ulimit` "were not proven" (line 299 of that file),
and it names `ubuntu-24.04` rather than `ubuntu-latest` because
"GitHub moves that label to Ubuntu 26.04 (glibc 2.43) in November 2026".
`ide-release.yml` is the repository's own precedent for a bounded podman container on a hosted runner:
`IDE_CONTAINER_CPUS: '4'`,
`IDE_CONTAINER_MEMORY: '12g'`,
a free-disk step,
and a first dry run of
"15 min 37 s" to its renderer gate.

### What a gate on a runner would still have to answer

No record shows any of the following working on a fresh runner,
and this round measured none of them because none of them can be measured without a hosted run:

- the `docker.io` pull of the CachyOS base image,
  then a `pacman` transaction with network access;
- `cargo-shadow lean records target/verification` inside the worktree;
- the cgroup read at `/sys/fs/cgroup/.../libpod-<id>.scope` on a runner whose user manager differs;
- container user `1000:1000` against the runner's user 1001 and its `/home/runner` workspace;
- the host seccomp profile edit,
  and the runner's kernel version,
  which the trial never printed
  (this host runs `7.3.5-2-cachyos`,
  and its podman is 6.2.1 against the runner's 5.7.0);
- the gate's disk footprint.

### The guest arms

Each ZFS arm runs only inside a throwaway guest,
and the guest's bounds are recorded:

- the guest has 8 GiB and adds a 6 GiB swap file,
  "so one `rustc` of this crate's tests at 6.7 to 6.9 GiB does not exhaust the guest's 8 GiB"
  ([`package/git-policy/cli/bin/mvm-guest-bounds.ts`](../../package/git-policy/cli/bin/mvm-guest-bounds.ts),
  `SWAP_GIBIBYTES`);
- one guest-recipe step may take `RECIPE_STEP_TIMEOUT_MS`,
  which is 120 seconds times 60,
  two hours,
  because the recipe builds the guest's packages,
  OpenZFS and the Rust toolchain
  (the same file).

So a guest arm asks a runner for about 14 GiB of memory and two hours of one job,
and the trial's runner had 15,482 MiB available.
One arm per job fits and nothing else does.
The draft's "at least a few minutes each" was wrong by an order of magnitude.

The October record on `ubuntu-24.04` says what the device does:

- "the named `user:1001:rw-` entry disappears after successful QEMU execution",
  and "The bounded raw workaround is a fresh named ACL grant before each QEMU invocation";
- "Its tradeoff is that this does not protect a libvirt operation that opens the device repeatedly";
- and,
  under "Corrected actual GitHub guest lifecycle",
  "Standard GitHub,
  Depot's GitHub runner and native Depot CI now each have a passing actual guest
  lifecycle in this cohort",
  with the child holding UID/GID 1001 and device group 993.

([the platform comparison](troubleshooting/depot-ci-versus-github-runner-platforms.md),
"Standard GitHub KVM access disappears after successful QEMU controls" and the section after it.)
The run that section names,
`39125868595`,
answers 404 today
(`gh run view 39125868595`,
read 2026-11-28),
so the record's text is all that is left of it.

The draft said one `setfacl` per boot and that `mvm`'s libvirt backend cannot be used as written.
Both are wrong as stated:
the grant is needed before each QEMU invocation,
and one October run passed the frozen `mvm` owner's own guest lifecycle with an added device group.
What remains open is an operation that opens the device repeatedly,
which is exactly what a libvirt backend does,
and no record shows it.

### What the gate is not

The gate is not a merge condition.
The owner's rule of 2026-11-23 says so in one line,
and the project then chose one gate per batch:

> "Merging to main shouldn't need a gate or a pass."

> The project chose "A:
> one gate on main per batch (Recommended)":
> "After a batch of merges,
> one whole gate on main's tip;
> a failure outside main's known baseline starts a fix round.
> Pros:
> one gate covers several branches;
> a breakage shows within the hour.
> Cons:
> a failure has to be traced to one branch of the batch."
> Not chosen:
> "B:
> gate only at a release",
> and "C:
> no gates".
> A round runs no whole gate of its own;
> it runs the host checks of the modules it touched.

([merges and publishes do not wait](decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md),
"What the owner said" and "What runs after a merge,
answered 2026-11-23".)

The gate is not what a release waits for either.
A release is a `workflow_dispatch` of `crate-release.yml`
([releases run in CI](decision/releases-run-in-ci.md),
"Decision"),
and the step its registry makes it run first is
`"dispatchGate": ["node", "bin/native-release-host-controls.ts"]`
(`package/crate-release/cli/releasable-crates.json`),
the host controls,
not the container gate.
The container gate is the coordinating session's practice before a dispatch,
and the record calls that "the coordinating session's practice and not the owner's answer,
with the owner's veto open".

A required check cannot by itself make a release wait,
and it can stop a release instead.
`main` today has `required_signatures: true`,
`enforce_admins: false`,
an empty required-check list and `strict: true`
(`gh api repos/Aquaticats/Monochromatic/branches/main/protection`).
The release pushes its bump to `main` as a signed API commit by `github-actions[bot]`
([releases run in CI](decision/releases-run-in-ci.md),
"Why the push is an API commit"),
and "the push of the bump starts no workflow"
(the same record,
"Consequences").
So a required status check on `main` would hold the bot's bump until a check ran on that commit,
and the commit starts no run to produce one.
That is this round's reading of two records and is not exercised;
it is the reason the question about required checks says so plainly.

## The trial that was made, corrected

The draft's trial was a workflow file on the trial branch,
`.github/workflows/hosted-runner-specimen.yml`,
triggered by a push to that branch alone
(read at `ca40ba0ca8`),
installing nothing that needs a secret,
using no larger runner and registering no self-hosted runner.
It ran three times and each run taught the next what to ask.
Each job concluded `success`,
which proves less than it looks:
the boot step carries `continue-on-error: true`,
so a run can end green with a failed guest,
and run 2 did exactly that.

- Run `46123229560`,
  `2026-11-28T07:53:04Z`,
  job 4 seconds:
  `Region: eastus2`,
  `Image: ubuntu-26.04`,
  `Model name: AMD EPYC 9V74 80-Core Processor`,
  `Virtualization: AMD-V`,
  `Mem: 15983` with 14682 free,
  `MemTotal: 16366636 kB`,
  `MemAvailable: 15523076 kB`,
  `/dev/root ext4 145G 58G 87G /`,
  `Docker version 29.4.2`,
  `podman version 5.7.0`,
  `crw-rw---- 1 root kvm 10, 232 Nov 28 07:52 /dev/kvm`,
  and after the grant `opened after the grant`.
- Run `46123307937`,
  `2026-11-28T07:54:09Z`,
  job 17 seconds:
  `Region: westus3`,
  the same AMD model and `AMD-V`,
  `MemTotal: 16366632 kB`,
  `MemAvailable: 15533176 kB`,
  `/dev/kvm` at `07:51`,
  `after the grant: opened, KVM_GET_API_VERSION answered 12`,
  then QEMU ended at once with `qemu-system-x86_64: --memory: invalid option`,
  and the step's own line read `qemu pid 2626, answered=false, seconds to login prompt 1`.
- Run `46123397702`,
  `2026-11-28T07:55:20Z`,
  job 59 seconds:
  `Region: centralus`,
  `Model name: INTEL(R) XEON(R) PLATINUM 8573C`,
  `Virtualization: VT-x`,
  `MemTotal: 16365620 kB`,
  `MemAvailable: 15467100 kB`,
  `/dev/kvm` at `07:55`,
  `KVM_GET_API_VERSION answered 12`,
  and `qemu pid 2600, answered=true, seconds to login prompt 44`,
  with the guest's own line `=== cirros: current=0.6.2 uptime=42.98 ===`.

(each read with `gh run view <id> --log` on 2026-11-28.)

So the draft's "What it printed",
presented as the last run's,
mixed run 1 and run 3:
the region,
vendor,
the two `MemTotal` and `MemAvailable` figures and the `07:52` device line
are run 1's,
and only the 44-second boot is run 3's.
The three runs landed in three regions on two vendors,
which is what multi-cloud scheduling predicts,
and the 44 seconds is one sample on Intel.

What the trial establishes:

- a standard public runner gives a working `/dev/kvm` after a named ACL grant,
  and a 512 MiB CirrOS image answers in under a minute,
  on two vendors;
- the runner image carries podman 5.7.0 and Docker 29.4.2,
  and 87 GB of free disk;
- the device and the grant are documented as unsupported for general use,
  so a check built on them is one GitHub may change without notice.

The trial's workflow file was removed from the branch after the runs.
The logs live on the run pages named in this section,
and nothing durable in this repository has held them until this section.

## What can run at once today, and under each decision

Today,
measured from the workflow files:

- `advisory-core` runs at most one run at a time on `main` plus one waiting,
  and its gate jobs serialize across every branch through `advisory-core-gate`;
- the per-push check workflows cancel their own predecessor on the same ref,
  so each runs one job at a time per ref and replaces the waiting one;
- the release and publish workflows serialize on fixed groups;
- a `push` to `main` starts `advisory-core` plus whichever path-filtered workflows match.
  At the read rate that is about 226 `advisory-core` runs a day and about 70 wrapper-touching pushes.

Under each of the six decisions:

- counting executions and keeping the numbers leaves the group as it is:
  at most one admitted `core` job runs,
  one waits,
  and the rest are refused before they queue;
- removing the window keeps the group,
  so a run is admitted per push at most once
  and the running one is never cancelled;
- `cancel-in-progress: true` replaces the running run at every push,
  and at a median gap of 139 seconds against a 21-minute gate almost every run would end cancelled;
- a hosted gate started by every wrapper-touching push would fire about 70 times a day
  at 21 to 35 minutes each,
  which is 25 to 40 job-hours a day against 20 slots;
  started per batch,
  as the merge bar asks,
  it fires as often as a batch ends;
- one workflow per changed package would add one job per package a push touches,
  which the path filters bound,
  and 20 slots bound the rest.

## Decisions for the owner

Six decisions,
each answerable alone,
and the combinations that cannot be built are named after them.
Each option says what it runs,
what it takes off this machine,
what it cannot do,
what it costs,
and what a yes still leaves undecided.

### 1. What should the advisory window count, and what is it for?

Terms:
the "window" is the 2 per minute,
4 per hour and 8 per day budget in `advisory-core.yml`'s gate;
"counting executions" means counting only runs whose `core` job actually ran,
the way Scorecard's gate does;
the "group" is the workflow's existing `advisory-core-${{ github.ref }}` concurrency group,
which already lets at most one run execute and one wait per ref.

Why it is asked:
the window counts events,
and at about 226 pushes a day it admitted nothing in the newest 100 runs,
which span 10 hours 34 minutes.
What it was for is not in any record,
so its purpose is the owner's to state,
and the answer decides whether the numbers matter at all.

#### Option 1: count executions, and leave the numbers and the group as they are

Mechanism:
the counting step reads each recent run's `core` job conclusion and skips runs whose job was
`skipped` or `cancelled`,
as `.github/workflows/scorecard.yml` does with one GraphQL query per 100 runs and a 32-page bound;
2 per minute,
4 per hour and 8 per day stay.
The group stays,
so one admitted job runs and one waits.

What it takes off this machine:
nothing directly;
it lets the existing advisory profile run at all,
which is the work the withdrawn Depot record named:
the static subset,
five builds and six unit tasks.
The 2026-11-25 dispatch of that job took 7 minutes 56 seconds of a hosted runner.

What it cannot do:
it does not make a red visible to anyone,
because nothing reads a red (decision 6);
it does not absorb a burst,
because the day window still closes after eight executions;
and in practice eight executions land in one or two bursts a day,
because the window opens faster than a day's pushes arrive,
so coverage is minutes,
not a day.
The static step still cannot start either,
because the job installs no `monochromatic-lint`:
its log says `task-run: line 1: monochromatic-lint: command not found`
([keeping the repository linter at zero](planning/repository-linter-stays-at-zero.md),
"The CI step cannot run"),
so this option needs that install as well.

Cost:
8 runs of about 8 minutes is roughly 64 hosted minutes a day,
free;
the change is in the counting step,
but it is not "one filter":
reading job conclusions needs a paginated query,
a cursor,
and a bound on pages,
as Scorecard's own step shows.
Upkeep is that query.

Example:
a push at `09:00Z` reads the newest 100 runs,
sees two `core` jobs executed in the last hour,
and refuses;
the next admitted push at `10:34Z` runs the six unit tasks and reports the three red
package tests measured in "Three package unit tasks,
run today".

What a yes leaves undecided:
what the numbers should be,
what an admitted run is for,
and whether a red ever reaches a person.

#### Option 2: count executions and drop the day window

Mechanism:
as option 1 with 8 per hour and no day cap,
group unchanged.

What it takes off this machine:
as option 1,
more often:
up to 64 hosted minutes an hour.

What it cannot do:
it cannot keep the newest commit checked either,
because the group keeps one pending run
and replaces it at the next push,
so the admitted run is whatever push arrived when the slot freed.

Cost:
as option 1,
with the counting query reading more runs;
the concurrency group,
not the numbers,
decides what executes.

Example:
a morning of 90 pushes admits eight runs an hour,
each on whichever commit was waiting;
the commits pushed between two slots are replaced and never checked.

What a yes leaves undecided:
whether any push beyond the admitted ones needs an answer at all,
and what happens to the commits the group replaced.

#### Option 3: remove the window and keep the group the workflow already has

Mechanism:
delete the counting step and the `needs.gate.outputs.allowed` condition,
leaving `advisory-core-${{ github.ref }}` with `cancel-in-progress: false`.
Every push is admitted,
one run executes and one waits per ref,
and the queue is the group.

What it takes off this machine:
one advisory run per 21-minute slot at most,
about 68 runs a day and 24 hosted hours a day,
against 20 concurrent slots;
the fast subset runs again on the newest commit that was waiting when a slot freed.

What it cannot do:
it checks the commit that was waiting when the previous run ended,
not the newest one,
and it cannot check every push:
at a median gap of 139 seconds the group replaces
about 98.8 percent of them.

Cost:
24 hosted job-hours a day,
free,
and one 21-minute job slot nearly always in use;
the upkeep is none,
and the gate job's five-minute timeout stays.

Example:
push A at `09:00:00Z` starts;
pushes B to J arrive over the next 20 minutes and replace each other
as the pending run;
at `09:21` the run for J starts.

What a yes leaves undecided:
whether an advisory run on `main` is worth a job slot at all times,
and whether the drift gates should report to a person.

#### Option 4: remove the window and let `cancel-in-progress: true` replace the running run

Mechanism:
as option 3 with the group's `cancel-in-progress` set to true.

What it takes off this machine:
at most one running advisory run,
cancelled by the next push.

What it cannot do:
with a 21-minute job and a 139-second median gap,
nearly every run is cancelled before it finishes,
so almost nothing is checked and the hosted minutes are spent cancelling.

Cost:
a cancelled run's minutes,
free,
and a check that reports nothing.

Example:
push A starts at `09:00`;
push B at `09:02` cancels it;
push C at `09:04` cancels B;
the day's runs all end `cancelled`.

What a yes leaves undecided:
nothing this option delivers,
which is why it is ranked last.

Ranking:
1 > 3 > 2 > 4.

- 1 over 3:
  option 1 keeps a bound on hosted minutes and keeps the profile's original shape,
  while delivering the same "it runs again at all" that option 3 does,
  and option 3 spends a job slot all day for a check nothing reads.
- 3 over 2:
  option 3 has no counting code to maintain and cannot be defeated by a miscount,
  while option 2 keeps a query whose numbers the group already overrides.
- 2 over 4:
  option 2 at least admits runs that finish,
  while option 4 cancels nearly all of them.

Answer for the purpose:
if the window's purpose was to keep advisory runs from crowding the 20 slots,
the measurements say it is not needed for that:
the group already holds a ref to one run and one wait,
and the CI provider record measured about 3 of 20 slots in use at its peak
([the CI provider decision](decision/ci-provider.md),
"Limits that actually apply").
If its purpose was something else,
the owner's answer decides,
and option 1 keeps every number.

What all four leave undecided:
whether `advisory-core` should exist at all.
The linter's own record ranks repairing this workflow last of its options
([keeping the repository linter at zero](planning/repository-linter-stays-at-zero.md),
"Ranking"),
which is worth reading beside this question.

### 2. If a hosted gate exists, does the local per-batch gate stop?

Terms:
the "local gate" is `mise run //package/git-policy/cli:native:gate`
(or `native:gate:main-tip`),
the 21-minute container gate on this machine;
the "batch gate" is the project's chosen rule,
one gate on `main` per batch of merges
([merges and publishes do not wait](decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md),
"What runs after a merge,
answered 2026-11-23").

Why it is asked:
a hosted gate takes load off this machine only if the local one stops,
and the owner's rule already says which gate the project runs.
The earlier draft never asked,
and its highest-ranked option rested on the load leaving.

#### Option 1: the batch gate runs on a hosted runner and the local one does not run

Mechanism:
the coordinating session dispatches one hosted gate after each batch instead of running
`native:gate:main-tip` on this machine;
the workflow's steps are the gate's own,
with `CONTAINER_LIMIT_DISK_WEIGHT=off`,
`--cpus 4` and a memory bound the runner can hold.

What it takes off this machine:
the 21 minutes,
the 10-CPU unit quota,
the 20 GiB unit bound,
the 6.6 GiB peak across two containers,
and up to 1,024 processes in the gate's containers.

What it cannot do:
it needs the one trial run first (decision 3);
it cannot run the guest arms,
the btrfs host controls or the service-manager arms;
and it makes a batch's verdict arrive later than a local gate's start,
by the runner's queue plus setup.

Cost:
21 to 35 hosted minutes per batch,
free;
a workflow file to keep in step with the gate's own tasks,
and one measured trial run before it can be trusted.

Example:
a batch of six merges ends at `11:20`;
the session dispatches the hosted gate at `11:21`;
the run builds the image,
runs the suite at four threads and Clippy beside it,
and prints the pass or the failure on the commit it ran on.

What a yes leaves undecided:
whether the gate's own unit-level replay and evidence manifest stay on this machine,
and what happens on a day when the runner image changes.

#### Option 2: both run, the hosted one first and the local one on failure

Mechanism:
the hosted gate runs per batch,
and a red hosted gate is re-run locally before a fix round starts,
because a red on a fresh runner may be the runner's own fault.

What it takes off this machine:
the same,
minus the days the hosted gate is red or suspect.

What it cannot do:
it doubles the gate's minutes on red days and keeps the local gate's disk and image,
so the machine still carries the gate's 4.33 GB image and its build cache.

Cost:
the hosted minutes of option 1,
plus a local gate on every red;
the upkeep of two paths that must agree.

Example:
the hosted gate is red on a merge touching the removal code;
the session runs the local gate;
the failure reproduces,
so the fix round starts from that log.

What a yes leaves undecided:
who decides a hosted red is suspect,
and how long the local copy stays.

#### Option 3: the local batch gate stays, and no hosted gate is added

Mechanism:
nothing changes;
the batch gate keeps running on this machine.

What it takes off this machine:
nothing.

What it cannot do:
it keeps the largest single block of this machine's CPU and memory for 21 minutes per batch,
which is the block the earlier draft ranked first.

Cost:
as today.

Example:
today,
unchanged.

What a yes leaves undecided:
nothing,
and it is the state the owner's rule already describes.

Ranking:
2 > 1 > 3.

- 2 over 1:
  option 2 answers the one thing a fresh runner can get wrong,
  its own image and
  kernel,
  without trusting a red it cannot explain,
  while option 1 acts on a red it has never
  seen on a runner.
- 1 over 3:
  option 1 is the only one that takes the 21 minutes off this machine,
  which is the aim stated at the top,
  and option 3 leaves the largest block where it is.

Both 1 and 2 rest on the trial run in decision 3.
Option 3 is the only one that is measured today.

### 3. Should the wrapper's container gate get a hosted workflow, and is one trial run made?

Terms:
the gate is `native:test:container`,
the image build,
the compile,
the whole suite and Clippy in
containers;
a "trial run" is one dispatch of that gate on a standard hosted runner,
made once,
to measure it.

Why it is asked:
this is the option the gate read called unmeasured,
and every claim that the gate fits a runner rests on records from this host.
The trial run is the only measurement that can settle it,
and it is the owner's to authorise
because it adds a workflow file and spends hosted minutes.

#### Option 1: make one trial run of the gate on a runner, then decide

Mechanism:
one workflow file on a branch,
`workflow_dispatch` only,
with `CONTAINER_LIMIT_DISK_WEIGHT=off`,
the gate's own task,
a free-disk step copied from `ide-release.yml`,
and steps that print `uname --kernel-release`,
`podman info`,
`df --human-readable /` before and after,
and the gate's peak memory readings.
It adds no required check,
no schedule and no push trigger.

What it takes off this machine:
nothing;
it measures.

What it cannot do:
it cannot prove the gate green on a runner means the same as green here,
because the guest arms and the btrfs host controls do not run in it;
one run is one sample of one runner image and one kernel.

What it shows:
whether the base image pulls,
whether the cgroup read works on a runner,
whether `1000:1000` against `/home/runner` breaks anything,
the gate's wall time and disk footprint on four vCPUs,
and whether its peak memory stays inside 16 GB.

Cost:
one workflow file that must not land on `main`,
21 to 35 hosted minutes,
free,
and the session's time to read one log.

Example:
dispatch the trial on a branch tip;
the log prints the kernel,
the podman version,
`Summary [<n>s] <n> tests run`,
the two memory peaks and the exit status in one place.

What a yes leaves undecided:
whether the workflow becomes a batch gate,
and what a red on it means.

#### Option 2: add the hosted gate as a batch gate without a trial run

Mechanism:
the workflow of option 1,
dispatched by the session after each batch.

What it takes off this machine:
the 21 minutes and the memory,
once it works.

What it cannot do:
it is unmeasured on a runner,
so the first batches are the trial,
and a red on the first batch is ambiguous between the gate and the runner.

Cost:
as option 1 plus the upkeep of a workflow whose first runs are its measurement.

Example:
the first batch after it lands is gated on a runner,
and its log is the first evidence.

What a yes leaves undecided:
the same,
plus what to do when the first run is red.

#### Option 3: no hosted gate

Mechanism:
nothing is added.

What it takes off this machine:
nothing.

What it cannot do:
it answers the gate read's central finding with "no".

Cost:
none.

Ranking:
1 > 2 > 3.

- 1 over 2:
  option 1 buys the measurement the ranking needs for 21 to 35 free minutes,
  while option 2 spends a batch's verdict on an unmeasured path.
- 2 over 3:
  option 2 at least leaves a workflow that can take the block off this machine,
  while option 3 leaves the block where it is.

What a yes to option 1 leaves undecided:
whether the local batch gate stops (decision 2),
and the memory and CPU bounds the hosted gate gets.

### 4. Should the packages' checks get a hosted workflow, and does it answer issue 6584?

Terms:
issue 6584 asks for a proposal on where the wrapper's TypeScript unit tests,
`lint:types` and
`lint:oxlint` run,
"so that a red one is seen within a batch of merges"
([issue 6584](https://github.com/Aquaticats/Monochromatic/issues/6584)).
The proposal exists,
and it ranks seven placements:
A inside the release gate,
C at the coordinator's batch tip check,
B in a unit beside the gate,
E in a CI workflow of its own,
D in the advisory fast subset,
F in every round,
G what comparable
repositories do;
"A > C > B > E > D > F > G"
([where the TypeScript checks run](planning/git-policy-cli-where-the-typescript-unit-tests-and-lint-run.md),
"Ranking").
That record also says the thing this decision must weigh:
"Nothing on this project's loop reads a push's CI result,
which the three red
`forbidden-strings` runs on `main` show"
(the same record,
option E's cons).

Why it is asked:
a hosted workflow is that proposal's E,
ranked fourth,
and choosing it here would answer issue 6584 in a different place from where that proposal puts it.
The owner would otherwise be answering one question twice.

#### Option 1: no new workflow; answer issue 6584 in its own proposal

Mechanism:
the placement question stays where it is asked,
and this proposal's decision 6 decides whether any answer becomes a required check.

What it takes off this machine:
nothing now;
placement A or C would move nothing off this machine,
while E or D would.

What it cannot do:
it leaves the three red package tests measured in "Three package unit tasks,
run today" unseen
until that proposal is answered.

Cost:
none here.

Example:
the coordinator's batch check runs the three checks,
as issue 6584's last sentence already
provides until an answer lands.

What a yes leaves undecided:
issue 6584's own question,
which the owner can answer there.

#### Option 2: one path-filtered workflow for the wrapper package only

Mechanism:
a workflow filtered on `package/git-policy/cli/**` running `lint:types`,
`lint:oxlint` and
`test:unit`,
with a memory bound of 4.5 GB (the record's peak) and no required check.
This is the placement record's E for one package.

What it takes off this machine:
three checks of 2.8 s,
12.8 s and 22.2 s on this machine,
and their 0.45,
0.99 and 4.5 GB peaks.

What it cannot do:
it does not make anyone read the result;
it prints 850 errors today,
so it would be red from its first run until issue 6581 is fixed;
and it adds a second place that claims to say whether the package is well.

Cost:
the fixed cost the record measured on a runner,
"about 15 s for that install beside 26 s of
checkout" (run `45754658990`),
plus the checks;
one path filter and a task list to keep.

Example:
a push touching `package/git-policy/cli/bin/native-gate-verdict.ts` starts the job;
its log ends `Found 1856 warnings and 850 errors.`
For a real Rust package the example is different:
a push touching
`package/forbidden-strings/cli/src/frx_scan.rs` would start a job running
`//package/forbidden-strings/cli:build` and its tests,
not a TypeScript check.

What a yes leaves undecided:
who reads it,
and whether it becomes required.

#### Option 3: one reusable, path-filtered workflow for every package with a unit task

Mechanism:
one reusable workflow,
called by per-package path filters,
running each package's `build`,
`test:unit` and the checks its `mise.toml` names.

What it takes off this machine:
every package's unit tasks,
which today run only when a person or a round starts one.

What it cannot do:
it cannot run the whole-tree lint or the Rust closure check;
it would be red on at least three packages today;
and it needs a list of packages kept current.

Cost:
about three hosted minutes per touched package plus the runner's 41-second fixed cost,
free,
and one entry per package.

Example:
a push touching `package/logger/core/src/sink/` runs that package's unit task,
which fails five files today and names them.

Ranking:
1 > 2 > 3.

- 1 over 2:
  option 1 keeps one question in one place,
  and the placement record's own ranking puts
  E below three placements that report where a person already looks,
  while option 2 answers the same question in a worse place without saying so.
- 2 over 3:
  option 2 covers the package whose reds are recorded and dated,
  and option 3 multiplies a red nobody reads across the 276 packages this round counted,
  which are the `package/<category>/<name>` folders holding a `mise.toml`.

### 5. Should the wrapper's ZFS guest arms move to a hosted runner?

Terms:
an "arm" is one `e2e:` task of the wrapper that boots a guest and drives a ZFS pool inside it;
`mvm` is this machine's guest tool,
with `libvirt`,
`hetzner` and,
since 2026-11-28,
an owner-chosen `ovhcloud` backend
([that decision](decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md),
"Decision").

Why it is asked:
the guests are the resource the owner bounded at six alive at once,
so they are the largest thing this proposal could move,
and the earlier draft priced an arm at minutes and said the backend could not be used,
both of which this round found wrong.

#### Option 1: keep the arms on this machine, and on the OVHcloud backend when it exists

Mechanism:
nothing changes here;
the arms keep booting `mvm` guests locally,
and the OVHcloud backend is built by its own round.

What it takes off this machine:
nothing until that backend lands.

What it cannot do:
it does not answer whether a runner can run an arm at all.

Cost:
this machine's memory for each guest,
8 GiB plus a 6 GiB swap file,
and the OVHcloud work belongs to another round.

Example:
today,
unchanged.

What a yes leaves undecided:
nothing here.

#### Option 2: prove one arm in a nested guest on a runner, then decide

Mechanism:
a trial workflow that boots one guest under QEMU with `--accel kvm`,
grants the device with `setfacl` before the invocation,
runs the OpenZFS recipe inside the guest and the arm's own steps,
and fails loudly when `/dev/kvm` cannot be opened.

What it takes off this machine:
nothing;
it measures whether one arm can leave.

What it cannot do:
it cannot promise that `mvm`'s `libvirt` backend works as it stands,
because the ACL entry disappears after a QEMU run and the record says that grant
"does not protect a libvirt operation that opens the device repeatedly";
it fits one arm per job and nothing beside it;
and it rests on a device GitHub documents as unsupported for general use.

Cost:
two hours is the recipe's own step bound,
and the guest wants 14 GiB of a runner's 15.5 GiB;
hosted minutes are free;
the upkeep is a second way of booting the guest.

What it shows:
whether OpenZFS builds and loads in a nested guest on a runner,
how long the recipe takes there,
and whether the 8 GiB guest survives on a 16 GB runner.

Example:
a dispatch runs one arm:
the log prints the KVM open,
the recipe's steps,
the pool's creation,
and the arm's own arbitration lines.

What a yes leaves undecided:
whether the other arms follow,
and whether `mvm` gains a backend for a runner at all.

#### Option 3: move the arms to a hosted runner now

Mechanism:
one job per arm,
as option 2 but for every arm.

What it takes off this machine:
whole guests.

What it cannot do:
as option 2,
and it multiplies an unmeasured path by the number of arms.

Cost:
two hours and 14 GiB per arm,
free at the point of use,
with a workflow to keep in step.

Ranking:
1 > 2 > 3.

- 1 over 2:
  option 1 keeps the arms where the pool and the rules already work,
  and option 2 spends a measurement on a path whose documented device support is experimental,
  so the answer may be "no" whatever the run says.
- 2 over 3:
  option 2 measures before it commits,
  and option 3 commits every arm to a path
  no record shows working.

What a yes to option 1 leaves undecided:
whether the OVHcloud backend will be cheaper than a hosted runner,
which is another round's question.

### 6. Who reads a hosted red, and does any check become required?

Terms:
a "red" is a failed hosted run;
"required" means a required status check on `main` in branch protection,
which only the owner can set.

Why it is asked:
today no check is required,
and nothing on this project's loop reads a push's CI result,
so a hosted red would arrive where nobody looks.
The repository's own numbers show what that costs:
of the newest 300 runs read on 2026-11-28T06:43-05:00,
`forbidden-strings.yml` is 110 failure,
85 cancelled and 105 success,
`git-policy-cli-fuzz.yml` is 128 failure,
134 cancelled and 38 success,
`toml-edit-fuzz.yml` is 102 failure,
86 cancelled and 35 success,
and `logger-fuzz.yml` is 52 failure,
3 cancelled and 11 success.
Those workflows are exactly the precedent for a hosted per-package check,
and their reds are not being read.

#### Option 1: nothing becomes required, and a red reaches a person by a named routine

Mechanism:
the coordinating session reads the hosted runs of the checks it dispatches,
at the batch check it already runs,
and names a red in its report to the owner;
no branch protection changes.

What it takes off this machine:
nothing;
it puts a reading duty on the session.

What it cannot do:
it depends on a routine that can leave a check out,
which is what issue 6584 records of the batch check's own commands:
and it does nothing for a red raised by a push nobody's batch covers.

Cost:
one `gh run list` per batch,
and one scheduled reading a day if the owner wants one.

Example:
at the batch check the session runs `gh run list --limit 20 --json conclusion,headSha,workflowName`,
sees a failed `git-policy-cli-fuzz` run on a commit in the batch,
and names it in the report.

What a yes leaves undecided:
who reads a red raised between batches.

#### Option 2: a required check on `main`, once a check is green on `main`

Mechanism:
the owner sets one required status check in branch protection after the chosen check has run green
for a while,
and the check's own workflow runs on `push` to `main`.

What it takes off this machine:
nothing.

What it cannot do:
it holds the release bot's bump commit,
which starts no workflow,
so the release would wait for a
check that cannot run until the record in
"What the gate is not" is changed;
and it holds every auto-push,
which is how this repository lands work.
It is also unusable while the chosen check is red:
the wrapper's lint is red with 850 errors,
and the three package unit tasks measured in "Three package unit tasks,
run today" are red.

Cost:
the owner's setting,
a change to how releases are dispatched,
and a fix round first.

Example:
after the lint is fixed,
the owner requires the whole-tree lint check;
a push to `main` that leaves it red is refused until the fix lands.

What a yes leaves undecided:
how a release dispatches its bump without a check that never runs.

#### Option 3: a required check on pull requests only

Mechanism:
the owner requires a check in the ruleset or protection for `pull_request` runs.

What it takes off this machine:
nothing.

What it cannot do:
it gates no merge here,
because merges are local:
the newest merged pull request is #1776 of 2026-10-19
(`gh pr list --state merged --limit 3`),
and every pull request since is an outside account's closed one (#6113,
#3472,
#3471).

Cost:
the owner's setting and a check that runs almost never.

Example:
an outside contributor's pull request waits for a check that no merge here was ever going to wait for.

Ranking:
1 > 2 > 3.

- 1 over 2:
  option 1 is the only one that can be built today with no change to how releases work,
  and option 2 needs the release path changed before it is safe.
- 2 over 3:
  option 2 at least gates the pushes that land work,
  while option 3 gates a path this repository does not merge through.

What a yes to any option leaves undecided:
whether the check's red is acted on by a person or by a round,
and how long a red may stand.

## Which combinations cannot be built

- A required check on `main` with the release's current push,
  at the same time:
  the bump commit starts no workflow,
  so the check cannot run and the release waits.
  Decision 6's option 2 and today's release path are exclusive until one changes.
- A guest arm inside a 16 GB runner beside anything else:
  the guest asks for 8 GiB plus a 6 GiB swap file,
  which is 14 GiB of 15.5 GiB available.
- A hosted gate started by every wrapper-touching push,
  with the owner's one-gate-per-batch rule:
  about 70 runs a day at 21 to 35 minutes each is 25 to 40 job-hours a day,
  and the rule asks for one gate per batch.
- A required check while the checks are red:
  the wrapper's lint has 850 errors and three package unit tasks fail today,
  so a required check would refuse every merge until they are fixed.
- `cancel-in-progress: true` on `advisory-core` with a finishing check:
  at a 139-second median push gap and a 21-minute job,
  almost every run ends cancelled.

## What this round did not measure

- Anything on a hosted runner.
  This round started no workflow run and pushed no workflow file.
- The gate's wall time,
  disk footprint,
  kernel version and podman behaviour on a runner.
- Whether OpenZFS builds and loads in a nested guest on a runner,
  and how long the recipe takes.
- Whether a required status check blocked the release bot's bump,
  which no write was made to test.
- How many of the day's pushes are one session's auto-pushes,
  which the sample does not separate.
- The 20-slot arithmetic across every workflow at the sampled rate,
  which needs every workflow's
  job list and not one workflow's.

## What the gate read found, and what changed

The read's counts are five High,
nine Medium and seven Low.
Each finding's answer states what this round checked or ran,
and whether the proposal changed or the finding was wrong.
`docs.github.com` searches and page reads were made on 2026-11-28,
and a measurement this round made is named with the command that made it,
so a reader can run it again.

### High

H1,
that the budget's account rests on withdrawn records and on an inferred purpose:
right,
and changed.
`doc/decision/depot-ci-adoption.md` line 4 reads "withdrawn on 2026-11-01" and line 8
"This decision is no longer in force",
and `depot-rolling-budget-evaluation.md`,
`depot-ci-measurement-targets.md`,
`depot-expanded-ci-baseline-register.md` and `depot-expanded-ci-rollout.md` each open
"This plan is withdrawn and none of it is in force" (all five read today).
The commit `e4a40ea3c1` reads in full
"Keep cancel-in-progress false so the rate gate,
not concurrency,
decides admission;
pending runs still coalesce",
and the dropped clause is the one that says a queue was never the risk.
No record gives the window a queue rationale:
an uncapped search of `doc/decision`,
`doc/planning`
and `doc/handover` for a twenty-minute queue finds only the Depot grilling's own job cap.
The revision names each withdrawn record where it is used,
marks the queue purpose as this round's
inference,
and asks the owner what the window is for.

H2,
that the first question contradicts the workflow's own concurrency group and omits the cheapest
option:
right,
and changed.
`advisory-core.yml:11-13` sets `group: advisory-core-${{ github.ref }}` with
`cancel-in-progress: false`,
and GitHub's page read today says at most one run executes and one waits per group under
`queue: single`,
with each new pending run replacing the last.
Of 250 commits on `origin/main` this round read,
the median gap is 139 seconds and 98.8 percent of
gaps are under 21 minutes 28 seconds,
so a per-push gate would mostly be replaced.
The first question now names the group,
prices each option against it,
and carries the option the
read said was missing:
remove the window and keep the group.

H3,
that the option and the release question do not deliver the stated purpose and run against a
record in force:
right,
and changed.
The 2026-11-23 rule reads "Merging to main shouldn't need a gate or a pass.",
and the project chose "one gate on main per batch",
with "A round runs no whole gate of its own"
(that record read whole).
The registry's gate for the wrapper is `"dispatchGate": ["node", "bin/native-release-host-controls.ts"]`,
not the container gate.
Decision 2 now asks whether the local batch gate stops,
and the question about required checks says what a required check would do to the release's bump:
the record says "the push of the bump starts no workflow",
so a required check could not be satisfied.

H4,
that the top-ranked option rests on stale or wrong numbers and is unmeasured:
right on every figure,
and changed.
The peaks in the gate's own limits file are 5,363.4 MiB for the build and
5,278.7 + 1,486.0 MiB for Clippy beside the suite,
so memory fits 15,482 MiB available;
the phases are sequential and only Clippy and the suite overlap,
which the source shows.
The 9,173-test example and the 185 failures are gone,
replaced by the newest gate's own summary
(12,517 tests,
1,104.9 seconds) and by the reason the failures went away.
The disk-weight refusal is now stated with the repository's own failed run and the switch that
cures it,
and this round measured the arguments the gate produces with and without the switch.
GitHub's documentation does not carry the `io.weight` refusal at all (0 search results today),
which is said plainly,
so the evidence is the recorded run rather than a vendor page.
The ranking of the container gate is now conditional on the one trial run,
and the trial run is its
own question.

H5,
that a hosted red would not be seen,
and that the third question duplicates an open proposal:
right,
and changed.
The placement record says "Nothing on this project's loop reads a push's CI result,
which the three
red `forbidden-strings` runs on `main` show",
and this round's own counts of the newest 300 runs show `git-policy-cli-fuzz.yml` at 128 failure and
134 cancelled of 300.
The per-package question is now tied to issue 6584 and to that proposal's ranking,
with the option that the placement question stays where it was asked ranked first,
and decision 6 asks who reads a red and what becomes required.

### Medium

M1,
that the trial's "What it printed" mixes three runs:
right,
and changed.
Read from the three runs' logs,
run `46123229560` printed `eastus2` and the AMD model with
`MemTotal: 16366636 kB`,
run `46123307937` printed `westus3` with `MemTotal: 16366632 kB`,
and run `46123397702` printed `centralus`,
the Intel model,
`VT-x` and `MemTotal: 16365620 kB`.
Each run's own line is now in its own place,
and the three regions and two vendors are stated.

M2,
that the KVM documentation is described as silent and the October record is cited for less than
it holds:
right,
and changed.
The runners reference says "GitHub-hosted Linux runners support hardware acceleration for Android SDK
tools",
so the documentation is not silent,
and the nested-virtualization warning stays.
The record's own words are now used:
the ACL entry "disappears after successful QEMU execution",
a fresh grant is needed "before each QEMU invocation",
and that "does not protect a libvirt operation
that opens the device repeatedly";
and its corrected lifecycle passage is quoted beside the statement that run `39125868595` answers
404 today.
The guest's 14 GiB and its two-hour recipe bound are now in option 5,
replacing "at least a few
minutes each".

M3,
that the first option's content and cost come from an October sample:
right,
and changed.
Read whole,
`advisory-core.yml`'s `core` job starts with five drift gates,
then the changed-path
static step,
then five builds and six unit tasks;
the unit subset is six packages,
not five.
The 21-minute October sample is replaced by the 2026-11-25 dispatch this round read
(7 minutes 56 seconds for the `core` job).
The red unit tasks are now this round's own measurement:
three of the three this round ran are red
today,
with the files named,
and the 2026-11-25 runner's state is not claimed for today.

M4,
that "one filter" is not one filter and that the numbers give bursts:
right,
and changed.
The run-level conclusion cannot distinguish an executed job from a skipped one,
and Scorecard's own step shows what counting executions takes:
a GraphQL query per 100 runs,
a cursor,
a 32-page bound.
The gate lists 100 runs,
and this round's read of the newest 100 shows they span 10 hours 34 minutes,
so a day window cannot be read from that list.
The change is now described as a paginated query,
not a filter,
and the burst shape is stated as a
consequence of a day window that opens faster than pushes arrive.

M5,
that nothing is said about what a public repository exposes:
right,
and changed.
A new section gives the settings this round read:
public with 6 forks;
`allowed_actions: selected` with SHA pinning and four patterns plus verified creators;
the default token at `read`;
11 secret names,
no values;
the `Copilot review` ruleset;
and GitHub's own sentence that a fork-triggered workflow gets no repository secrets and a read-only
token.
It also says what a new workflow must therefore do.

M6,
that the release question does not state the protection and that pull requests are not how this
repository merges:
right,
and changed.
`main` has `required_signatures: true`,
`enforce_admins: false`,
an empty check list and
`strict: true`,
read today;
the newest merged pull request is #1776 of 2026-10-19 and every later one was closed unmerged.
The question now says a pull-request-only check gates no merge here,
and what a required check would
do to the bump push.

M7,
that the options are complements ranked as alternatives and that two questions bundle choices:
right,
and changed.
The five options became six decisions,
each with its own ranking,
and the bundled options were split:
counting executions is separate from the numbers,
and the per-package workflow is separate from the placement question.
A section states which combinations cannot be built.

M8,
that the adjacent-pair reasons rest on host records and not on a measurement of the option:
right,
and changed.
Every ranking's criterion is now named beside it,
and each adjacent pair gives its reason.
The container gate's ranking no longer decides the top place,
because the decision that would put it
there is a separate question whose first option is the trial run,
and no ranking now leans on what exists or on effort to build.

M9,
that some questions are not the owner's yet and some need the rest of the proposal:
right,
and changed.
The question that a measurement settles (the trial) is now its own decision with "make one run,
then
decide" first;
the local-gate question the draft left unasked is decision 2;
and every decision defines its terms in its first paragraph,
so it reads alone.

### Low

L1,
the "thirteen days" figure:
right,
and changed.
The placement record gives the lint "at least 13 days 8 hours 58 minutes",
one unit test "1 day 2 hours 41 minutes" and the other "2 days 0 hours 8 minutes"
(`git-policy-cli-where-the-typescript-unit-tests-and-lint-run.md`,
"When each red check went red").
The draft's phrase is gone.

L2,
the second option's example:
right,
and changed.
`package/forbidden-strings/cli/src/` holds Rust files,
and
`Found 1856 warnings and 850 errors` is `//package/git-policy/cli:lint:oxlint`'s line
(the placement record,
"What was measured").
The example now uses a real Rust package for the Rust case and the wrapper for the TypeScript case.

L3,
that the 45-minute sentence sits in a different context:
right,
and changed.
The sentence is quoted with its section and with the page's own first sentence about services being
unavailable,
and the question whether it drops a run waiting for a free slot is named as not stated
and not rested on.

L4,
the citation slips in the container gate section:
right,
and changed.
The heading the draft cited does not exist;
the 9,173-test run and the 1,123.8 MiB suite peak are gone;
and the section now cites the gate's own source for each bound and its own logged summary for the
test count.
The `e2e:` descriptions the draft attributed to the slot-store design are cited to
`package/git-policy/cli/mise.toml` instead.

L5,
that the runner's image,
podman and kernel are unpinned:
right,
and unchanged in fact,
now stated.
The trial ran `ubuntu-26.04` with podman 5.7.0 and never printed its kernel;
this host runs podman 6.2.1 and kernel `7.3.5-2-cachyos`;
and the trial run question lists `uname --kernel-release` and `podman info` among the steps that
would print them.

L6,
that no action installs the Rust toolchain:
right,
and changed.
`advisory-core.yml` installs it with `rustup toolchain install` in a run step,
and `allowed_actions` is `selected` with SHA pinning,
so no action is needed.
The clause is gone.

L7,
that a green trial run is not evidence by itself:
right,
and changed.
The trial workflow at `ca40ba0ca8` carries `continue-on-error: true` on its boot step,
and run 2 ended `success` with `qemu-system-x86_64: --memory: invalid option`.
The section says so beside the run list.
