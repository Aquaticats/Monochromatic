# GitHub's hosted runners for the package checks, and a guest trial

Status:
a proposal for the repository owner,
unapproved and non-final.
Nothing here is adopted:
no workflow file is added,
no required check changes,
and no review from a vendor other than its writers' has read it.

Gate reads of this proposal:
two,
both same-vendor second passes by the vendor of its writers,
neither an independent review.
The first,
on 2026-11-28,
returned five High,
nine Medium and seven Low findings;
the second,
later the same day,
four High,
ten Medium and nine Low.
Those counts are the reads' own.
The second found the file not ready to put to the owner,
and this revision answers it finding by finding;
the answers are in the report of the run that made this revision,
with the command or the line each one rests on,
because both reads are kept outside the repository and expire a day after their last change.

Written by Pi runs on 2026-11-28.
This revision was made in the worktree `.claude/worktrees/pi-github-hosted-runners-proposal-revised-against-its-second-gate-read-with-a-trial-that-can-start`,
on the branch `docs/planning-github-hosted-runners-proposal-revised-against-its-second-gate-read-with-a-trial-that-can-start`,
from `main` at `0d03853329`.

Every figure states how it was measured and when,
or which record it is read from.
A claim that rests on reading code,
a record or a vendor page says so in the sentence that makes it;
one that is an inference says whose it is.

## What the owner is asked

The aim this proposal is written against is **inferred**:
this round's reading of two records,
not a decision of the owner's.
It is that the work worth moving off this machine is work that competes for its memory.
The machine ran out of memory on the night of 2026-11-27,
with nineteen rounds and ten guests alive ([the owner's answers of 2026-11-28][owner answers],
"The rounds that were running,
and what bounds a later batch";
[the troubleshooting record][oomd record]),
and the owner then bounded a later batch by the guests alive at once.
The way to reject that aim is to choose,
in every decision,
the option that moves least:
decision 1 option 1,
decision 2 option 1,
decision 3 option 3,
decision 4 option 1,
decision 5 option 1,
decision 6 option 1.

Six questions,
each answerable alone.
The recommended option is named with what its recommendation rests on.

1.  **What should the advisory window count?**
    What the window is for is the second half of the same question.
    Recommended:
    remove the window and keep the workflow's existing concurrency group (option 4 of 4).
    **Measured**:
    at the pushes' present rate that ends 152 runs a day,
    where the window as it stands ends none (this round's arithmetic on 400 pushes,
    "What a push does today").
2.  **Should a newer push cancel a running advisory run?**
    Recommended:
    no,
    keep `cancel-in-progress: false` (option 1 of 2).
    **Measured**:
    keeping it ends a check on about 152 pushes a day;
    cancelling ends a check on about 46,
    and cancels 88 percent of the runs it starts.
3.  **Where should the wrapper's container gate run?**
    Recommended:
    on a hosted runner per batch,
    with a red re-run locally (option 2 of 3),
    after one trial run of the gate on a runner.
    **Inferred**:
    no gate has run on a runner,
    so a green batch's verdict would rest on an unmeasured image and kernel for the first time;
    the local re-run is what keeps a red explainable.
4.  **Where should the wrapper's TypeScript unit tests and lint run (issue 6584)?**
    Recommended:
    no new workflow;
    keep answering that question in its own proposal (option 1 of 3).
    **Read**:
    the placement record puts a hosted workflow fourth of its seven placements,
    and records that nothing on this project's loop reads a push's CI result.
5.  **Should the wrapper's ZFS guest arms move to a hosted runner?**
    Recommended:
    keep them on this machine,
    and on the OVHcloud backend the owner already chose (option 1 of 3).
    **Read**:
    the owner chose "Guests on OVHcloud" alone,
    so a guest uses none of this machine's memory;
    and GitHub documents nested virtualization as unsupported.
6.  **Should any check become required?**
    Who reads a hosted red is asked with it.
    Recommended:
    nothing becomes required,
    and a named routine plus the owner's own notification setting carry a red to a person (option 1 of 3).
    **Read**:
    `main` protects nothing required today,
    and the owner's own pushes bypass branch protection by GitHub's default,
    so a required check would not hold them.

Which of these wait for a trial:

- decision 3 waits:
  both hosted options follow one trial run of the gate on a runner,
  and the trial is the session's own step (see "The trial of the container gate").
- all the others can be answered now.

Three questions a record or a setting half-answers,
shown beside the question where they belong:

- what the advisory window was for:
  the owner's own words in the Depot-era interview (decision 1);
- where a guest's memory comes from:
  the owner's choice of OVHcloud,
  made the same night (decision 5);
- whether a session may run a trial workflow itself:
  the project's rule that a session acts unasked on current-repository GitHub mutations (decisions 3 and 5).

What a yes to each recommendation leaves undecided,
in one clause each:

1.  whether the profile should exist at all,
    and whether its drift gates should report to a person;
2.  what an advisory verdict is worth when no check is required (decision 6);
3.  the memory and CPU bounds the hosted gate gets,
    and what a red on it means;
4.  issue 6584's own placement question;
5.  whether the OVHcloud sizes design passes its own gate read and is built;
6.  who acts on a red between batches.

Four combinations that cannot be built,
each with its reason,
are listed under "Combinations that cannot be built".

The sections between here and there carry the evidence:
what a push does today (measured),
what the advisory profile needs before it can be green (measured),
what a hosted runner is and is not (read from GitHub's pages and this repository's settings),
what the checks need (measured),
what the container gate needs (read from its source,
measured on this host),
what a guest arm asks for (read from its bounds),
and the unknowns a hosted gate would carry.

## What a push does today, measured

Read with `gh run list --workflow advisory-core.yml --limit 400` at 2026-11-28T07:51-05:00,
with the same workflow's runs read through the REST API for their actors at 2026-11-28T08:04-05:00,
and with one job-list read per run of the newest hundred:

- 400 runs,
  every one a `push` to `main`,
  every one concluding `success`,
  every one triggered by the account `Aquaticat`,
  spanning `2026-11-27T11:30:18Z` to `2026-11-28T12:50:27Z`:
  25.34 hours,
  which is 379 pushes a day.
- The gaps between them:
  median 122 seconds,
  mean 229 seconds,
  ninetieth percentile 508 seconds,
  longest 9,038 seconds (2 hours 31 minutes).
  351 of the 399 gaps are shorter than the newest executed job's 476 seconds (88 percent),
  372 shorter than 557 seconds (93 percent),
  and 394 shorter than 21 minutes (99 percent).
- The rate is uneven:
  the hourly counts of the same 400 runs run from 0 to 37,
  with 15 the middle hour.
- 432 first-parent commits landed on `main` in the same window (median gap 109 seconds,
  mean 211 seconds),
  so a push carries about one commit on average and sometimes a merge's whole branch.
- 111 of the 399 push ranges (27.8 percent) change something under `package/git-policy/cli`,
  which is about 105 pushes a day that would start a path-filtered wrapper workflow.

Every one of the newest hundred runs' job lists reads `advisory window=success;advisory core=skipped`:
the gate job ran,
the checks did not.
That is the closed door,
and it is measured rather than argued.

The reason is in the workflow,
read whole.
The gate job counts the newest 100 runs of the workflow and refuses when any window is full
(`.github/workflows/advisory-core.yml`,
the `Count recent admitted runs` step):

```text
# .github/workflows/advisory-core.yml, lines 46 to 63
runs=$(gh run list --repo "$GITHUB_REPOSITORY" --workflow advisory-core.yml --limit 100 --json databaseId,createdAt --jq '.' 2>/dev/null || echo '[]')
...
- budgets: 2 per minute, 4 per hour, 8 per day
if [ "$minute" -ge 2 ] || [ "$hour" -ge 4 ] || [ "$day" -ge 8 ]; then
```

It reads `databaseId` and `createdAt` only,
so a run whose `core` job was skipped counts against the windows that skipped it.
At 379 pushes a day the day window of 8 is full within the first hour of every day and never empties.
Arithmetic on the same 400 arrival times,
with today's rule,
admits 3 runs in the 25.34 hours,
all of them before the day window filled;
the other 397 pushes are refused by the count.

Two event paths skip the count altogether,
and the same step says so:
a `workflow_dispatch` is admitted with `allowed=true`,
and a `pull_request` is admitted with "push budget not consumed".
So the door is closed on pushes only,
and every open pull request runs the checks regardless of the window.
The first gate read counted five outside pull requests in 47 seconds on 2026-11-01;
that count was not re-measured here.

The concurrency group is the other half.
The workflow's own group is `advisory-core-${{ github.ref }}` with `cancel-in-progress: false` (lines 12 and 13),
and GitHub's page on concurrency says that "there can be at most one running job or workflow in a concurrency group at any time",
that a newly queued one is `pending`,
and that under the default `queue: single` "any existing pending job or workflow in the same concurrency group will be canceled and the new queued job or workflow will take its place" ([the concurrency page][concurrency page],
read 2026-11-28).
The commit that wrote the gate says what it meant:
"Keep cancel-in-progress false so the rate gate,
not concurrency,
decides admission;
pending runs still coalesce" (`e4a40ea3c1`,
message read whole).

`advisory-core.yml` is not the only workflow here that runs on a push to `main` with `cancel-in-progress: false`:
`scorecard.yml` runs on `push` to `main` as well,
keeps `cancel-in-progress: false` on the job-level group `scorecard-rate-limit`,
and counts analysis checks that actually ran rather than arrivals.
It is the live precedent for counting executions,
and the count it does is a GraphQL query per 100 runs,
with a cursor and a 32-page bound (`.github/workflows/scorecard.yml`,
the `rate-limit` job).

## What the advisory profile needs before it can be green, measured

Three `workflow_dispatch` runs of the workflow exist,
and all three are red.
Their `core` jobs are the only ones this proposal can price:

- run `45531170934`,
  2026-11-25T02:11:46Z to 02:19:42Z,
  476 seconds,
  `failure`;
- run `45529509832`,
  2026-11-25T01:57:50Z to 02:07:07Z,
  557 seconds,
  `failure`;
- run `40864046534`,
  2026-10-29T11:16:08Z to 11:22:49Z,
  401 seconds,
  `failure`.

So the job runs in 6 minutes 41 seconds to 9 minutes 17 seconds,
and those are red runs that stopped early,
not a green run's length.
The `Summary` and `Enforce advisory result` steps then fail the run for any failed step.

The steps of run `45531170934`,
read from its log:
`Build fast subset` 352 seconds and success,
`Unit fast subset` 19 seconds and failure.
The unit step stops at its first failing task,
and the task it stops at cannot start on a runner at all:

```text
# the log of run 45531170934, the `Unit fast subset` step
task-run [completed] "//package/container-limit/core:test:unit"
task-run [started] "//package/forbidden-strings/cli:test"
error: no such command: `nextest`
task-run [failed] "//package/forbidden-strings/cli:test"
```

Two installs are missing,
and no workflow here adds either:

- `cargo nextest`,
  which `//package/forbidden-strings/cli:test` runs as `cargo nextest run --release` (`package/forbidden-strings/cli/mise.toml`) and `//package/linter/monochromatic-lint:test` runs as `cargo nextest run --all-targets` (`package/linter/monochromatic-lint/mise.toml`).
  `.github/actions/repo-tools-setup/action.yml` can install `file-enforcer`,
  `mpm-shadow`,
  `cargo-shadow` and `git-policy-cli`,
  and `advisory-core.yml` asks it for the first three;
  the workflow mentions `nextest` zero times.
- `monochromatic-lint` on the path,
  which the `Static changed paths` step needs:
  it runs `task-run lint:monochromatic-lint`,
  and the root task's body is the bare program name.
  The record of the workflow's own runs has the failure in its own words:
  "the step log says `task-run: line 1: monochromatic-lint: command not found`" ([keeping the repository linter at zero][linter record],
  "The CI step cannot run").
  That step is `continue-on-error: true`,
  so its conclusion reads `success` and only its `outcome` is a failure,
  which the last step turns into a failed run.

The same dispatch was also red for three reasons that are the state of that commit rather than a missing tool:
the committed-file drift found four `Cargo.toml` files the catalog renders differently,
the pnpr config drift found five packages missing from `config.yaml`,
and the Rust lockfile gate found three stale `Cargo.lock` files.
The unit step's first four tasks passed before it reached the one that needs `nextest`.
The committed-file drift passes on `main` today:
`file-enforcer check --outputs committed` printed `{"code":"check-clean","message":"check: every committed generated file is up to date","drifted":0}` and exited 0 (read 2026-11-28T07:58-05:00,
in a capped scope).
Whether the pnpr config gate and the Rust lockfile gate pass today was not measured here.

So a yes to any option of decision 1 buys a red run until `nextest` and `monochromatic-lint` are installed,
whatever the window admits,
and no example in this file shows the six unit tasks passing on a runner,
because no run has shown it.

The three package unit tasks this proposal's earlier round measured red on this host (`logger/core`,
`toml-edit/core` and `container-limit/core`) are three of the four tasks the 2026-11-25 run completed,
with `p-map`,
in 19 seconds,
before it stopped.
That is not evidence they pass on a runner today:
they were red on this host,
and a runner's state was measured once.

## What a hosted runner is, and is not

Read from GitHub's pages on 2026-11-28,
and from this repository's own settings.

### The machine a job gets

- "Use of the standard GitHub-hosted runners is free and unlimited on public repositories",
  and the row for Linux,
  x64 gives 4 processors,
  16 GB of memory and 14 GB of SSD for `ubuntu-latest`,
  `ubuntu-24.04`,
  `ubuntu-22.04` and `ubuntu-26.04` ([the hosted runners reference][runners reference],
  "Standard GitHub-hosted runners for public repositories").
- "Standard GitHub-hosted runner | Free | 20 | 5 | Not applicable",
  in the row for job concurrency ([the limits page][limits page],
  "Job concurrency limits for GitHub-hosted runners"):
  twenty concurrent jobs for this repository's plan,
  which is the owner organisation's free plan,
  read with `gh api`.
- "Each job in a workflow can run for up to 6 hours of execution time",
  the same page.
- "The software tools included in our GitHub-owned images are updated weekly" ([the concepts page][concepts page],
  "Preinstalled software for GitHub-owned images"),
  and "The `-latest` runner images are the latest stable images that GitHub provides,
  and might not be the most recent version of the operating system available from the operating system vendor" (the runners reference).
- A Linux job may land on AWS or Azure:
  "These jobs can run on either Amazon Web Services (AWS) or Microsoft Azure" and "Jobs are automatically scheduled across available cloud providers" (the concepts page,
  "Cloud hosts used by GitHub-hosted runners"),
  which is why the three trial runs landed in three regions on two processor vendors.
- The 45-minute sentence is about GitHub Actions services being unavailable:
  "if the workflow run has been successfully queued,
  but has not been processed by a GitHub-hosted runner within 45 minutes,
  then the queued workflow run is discarded" (the concepts page,
  "Workflow continuity").
  Whether it also drops a run waiting for a free slot under load is not stated on that page,
  and nothing here rests on it.
- A larger runner is not free here:
  "Larger runners are available for organizations and enterprises on GitHub Team and GitHub Enterprise Cloud plans" (the runners reference),
  and this owner is on the free plan.

### Nested guests, in GitHub's own words

- "GitHub-hosted Linux runners support hardware acceleration for Android SDK tools" (the runners reference,
  "Supported runners and hardware resources").
- "While nested virtualization is technically possible while using runners,
  it is not officially supported.
  Any use of nested VMs is experimental and done at your own risk,
  we offer no guarantees regarding stability,
  performance,
  or compatibility" (the concepts page,
  "Preinstalled software for GitHub-owned images").

What no page promises is the permission on `/dev/kvm`,
the ACL grant this repository's October record needed,
or a general-purpose nested VM.

### This repository's settings that bound any new workflow

Read with `gh api` at 2026-11-28T07:51-05:00:

- the repository is public,
  belongs to an organisation,
  and has 6 forks;
- `allowed_actions: selected` with `sha_pinning_required: true` and the patterns `docker/*`,
  `jdx/*`,
  `actions/*`,
  `changesets/*` plus verified creators;
- the default workflow token permission is `read`,
  and `can_approve_pull_request_reviews` is false;
- 11 repository secret names,
  no value read;
- one active ruleset,
  `Copilot review`,
  on all branches,
  with an empty bypass list.

A workflow triggered from a fork gets no repository secret:
"With the exception of `GITHUB_TOKEN`,
secrets are not passed to the runner when a workflow is triggered from a forked repository.
The `GITHUB_TOKEN` has read-only permissions in pull requests from forked repositories" ([the events reference][events reference],
"Workflows in forked repositories").
So a new workflow here pins its actions by full SHA from that allow-list,
takes the read-only token unless a job asks for more,
and runs fork code with no secret in reach.

### What a required check on `main` would do

Read today,
this is the state of `main`'s protection (`gh api repos/Aquaticats/Monochromatic/branches/main/protection`):

```text
required_signatures.enabled: true
enforce_admins.enabled: false
required_status_checks.strict: true
required_status_checks.contexts: []
required_status_checks.checks: []
```

and of the pushes that land work:

- all 400 of the newest `advisory-core` runs,
  which are pushes to `main`,
  were triggered by `Aquaticat`,
  whose permission on the repository is `admin` (`gh api .../actions/workflows/advisory-core.yml/runs`,
  read 2026-11-28T08:04-05:00,
  and `gh api .../collaborators/Aquaticat/permission`);
- 99 of the newest 100 commits on `main` were committed by `Aquaticat` and one by `web-flow`,
  GitHub's own web-interface committer (`gh api repos/.../commits?sha=main&per_page=100`);
- the repository has no deploy keys,
  so the workflow run's actor is the pushing account (the events reference:
  a deploy-key push would show the repository admin who verified the key).

GitHub's own rule for what that means:
"By default,
the restrictions of a branch protection rule don't apply to people with admin permissions to the repository or custom roles with the 'bypass branch protections' permission" ([protected branches][protected branches],
"Do not allow bypassing the above settings").
So a required status check on `main` would not hold the owner's own pushes,
unless the owner also turns on the setting that page names,
"Do not allow bypassing the above settings",
which this repository does not have on.

What it would stop is the release's bump,
and by GitHub's own pages a stopped push is refused,
not made to wait:

- "Required status checks must have a successful,
  skipped,
  or neutral status before collaborators can make changes to a protected branch",
  and "After all required status checks pass,
  any commits must either be pushed to another branch and then merged or pushed directly to the protected branch" (protected branches,
  "Require status checks before merging");
- the page's own example of a refused protected-branch update,
  for a review requirement,
  is an error:
  `remote: error: GH006: Protected branch update failed for refs/heads/main.`
- the bump is an API commit by `github-actions[bot]` under `GITHUB_TOKEN` ([releases run in CI][releases record],
  "Why the push is an API commit"),
  and "if a workflow run pushes code using the repository's `GITHUB_TOKEN`,
  a new workflow will not run even when the repository contains a workflow configured to run when push events occur" ([triggering a workflow][triggering page],
  "Triggering a workflow from a workflow");
- the release record says the same of its own consequences:
  "The push of the bump starts no workflow,
  so the checks that run on `push` to `main` ...
  skip that commit" (releases run in CI,
  "Consequences").

So:
a required check would arrive at the bump commit with a check that no run can produce,
and the push would be refused.
That last step is **inferred** from those three pages and two records;
no push was made to test it,
and the release record itself says of its own reading,
"Rulesets and branch protection were read,
not exercised".
Whether the `github-actions` app could be added to a bypass list is not checked;
the protected-branches page says only that bypass lists need an organisation-owned repository,
which this is.

## What the checks need, measured

### The wrapper's three TypeScript checks

Measured in the placement record for issue 6584,
at `1c8acd5bfb`,
each in its own capped scope on this host ([where the TypeScript checks run][placement record],
"What was measured"):

- `lint:types` exited 0 in 2,810 ms and 2,639 ms,
  peaks of 452.9 MB and 447.1 MB;
- `lint:oxlint` exited 1 in 12,791 ms,
  a peak of 987.1 MB,
  printing `Found 1856 warnings and 850 errors.` and `Finished in 10.0s on 242 files with 491 rules using 4 threads.`;
- `test:unit` exited 1 in 22,166 ms,
  a peak of 4,541.9 MB,
  with two test files failing.

Those are that record's figures and its date,
not this round's.
For a hosted step they bound the memory such a step needs:
4.5 GB at that commit,
and 1.8 GB after issue 6580's fix (the same record,
"What this does not settle",
question 5).

### The three package unit tasks, and the wrapper's placement record

This proposal's earlier round ran three package unit tasks in capped scopes on a clean worktree at
`origin/main` on 2026-11-28,
one run each:
`//package/logger/core:test:unit`,
`//package/toml-edit/core:test:unit` and `//package/container-limit/core:test:unit`.
and all three exited 1,
naming the failing files.
The third is the shape of issue 6588,
closed on 2026-11-27 for one missing ledger entry:
nothing on this project's loop runs these tests after a merge,
so the ledger fell behind again.
The second read reproduced these three runs' results.
This round changes nothing that ships,
so the red is recorded and not fixed.

## The container gate: what it runs, what it needs, what a runner must still answer

### What it runs, read from its source

The gate is `mise run //package/git-policy/cli:native:test:container` (`package/git-policy/cli/mise.toml`),
whose first step is `cargo-shadow lean records target/verification`,
and whose second is `node bin/test-native-container.ts`.

- The image build is awaited and runs with `--network=none --http-proxy=false --pull=never`,
  so the base image must already be pulled;
  `native-test-base.ts` pulls `docker.io/cachyos/cachyos-v4:latest` on every run and,
  when that pull returns an image it does not already hold,
  "builds a new derived base with network" (`package/git-policy/cli/bin/native-test-base.ts`,
  its header).
  A fresh runner is that case on its first run,
  and every layer is cold.
- The suite and Clippy then start together and the gate waits for both.
- Containers run as `user: '1000:1000'`,
  against the runner user of 1001.
- The bounds are in `native-gate-container-limits.ts`:
  `gateContainerCpus = 4`,
  which is also the test-thread and compile-job count,
  `gateBuildMemory = '8g'` against a measured compile peak of 5,363.4 MiB,
  `gateClippyMemory = '8g'` against Clippy peaks up to 5,278.7 MiB,
  `gateSuiteMemory = '4g'` against a suite peak of 1,486.0 MiB,
  and 256 processes per test thread,
  so 1,024 at four.
- The containers get the host's `/usr/share/containers/seccomp.json`,
  with `listmount` and `statmount` added to the rule that names `mount`.
- The gate runs as its own capped systemd user unit (`native:gate`),
  and `native:gate:main-tip` detaches a worktree at `main`'s tip and gates there.

The peaks are measured on this host,
on this kernel.
They bound the two phases,
not the gate's total:
build alone is about 5.2 GiB,
and Clippy beside the suite is 5,278.7 + 1,486.0 = 6,764.7 MiB,
about 6.6 GiB.
The three trial runs measured `MemAvailable` of 15,159,
15,170 and 15,104 MiB at job start,
so 6.6 GiB of container peak fits a 16 GB runner with about 8 GiB to spare,
provided nothing else in the job holds that much,
which is not measured.

### How long it takes on this host, and why that is a spread

The newest gate on this machine printed,
in its own log:

```text
Summary [1104.852s] 12517 tests run: 12517 passed (6 slow), 51 skipped
Clippy passed beside the test run.
gate-exit-status=0
```

so the suite's own wall time is 1,104.9 seconds,
18.4 minutes,
at four test threads,
and the gate around it is longer.

The gate's whole length is not one number,
and this file no longer pretends it is:

- two gates of 2026-11-27 took 20 minutes 36 seconds and 21 minutes 43 seconds from unit start to stop,
  of which the suite was 889 s and 1,259 s (the placement record,
  "For scale",
  reading `doc/planning/git-policy-cli-cachyos-test-containers.md`);
- the eleven gate logs of 2026-11-27 and 2026-11-28 under `~/temp/agent/gate/` span,
  from file creation to last write,
  20.4,
  21.8,
  22.1,
  22.1,
  25.4,
  26.1,
  30.9,
  31.0,
  32.3,
  38.6 and 41.8 minutes (median 26.1),
  which is an upper bound on each unit's length because a log carries no timestamps;
  the method matches the two recorded start-to-stop figures within 12 s and 4 s,
  on the two gates it can be compared with;
- the same placement record says a gate costs "20 minutes to 50 minutes of wall time plus its queue".

That spread is this host's,
under its own load:
gates of one binary on different commits differ by twenty minutes.
A runner's length is unmeasured,
and one run would be one sample of it.

Two things on a runner are outside that spread and unmeasured:
the cold derived-base build with network on the first run,
and the runner's four vCPUs carrying the image build,
podman and every shell step where this host's containers get four CPUs of their own.

### The gate's disk

Measured today with `podman images --format '{{.Repository}}:{{.Tag}} {{.Size}}'`:
the eleven gate images are 4.32 to 4.36 GB each,
the derived bases 2.22 to 2.27 GB,
and the pulled CachyOS base 1.24 GB.
A standard runner's SSD is 14 GB by the reference table,
and the trial measured `145G` with `87G` available,
so the numbers in the table and in the log are not the same figure,
and no record shows the gate's full disk footprint on a runner.
`ide-release.yml` already copies a free-disk step for its own containers,
and a hosted gate would copy it.

### The one refusal the repository has already recorded

The gate's containers are started with a block-layer weight:
`diskWeightArguments()` returns `['--blkio-weight=10', '--cgroup-conf=io.weight=10']`,
and returns nothing when `CONTAINER_LIMIT_DISK_WEIGHT` is `off` (`package/container-limit/core/src/index.ts`).
Measured today,
by importing the gate's own module (`package/git-policy/cli/bin/container-limits.ts`) and printing its answer:

```text
CONTAINER_LIMIT_DISK_WEIGHT unset: ["--memory=4g","--cpus=4","--pids-limit=1024","--blkio-weight=10","--cgroup-conf=io.weight=10"]
CONTAINER_LIMIT_DISK_WEIGHT=off:  ["--memory=4g","--cpus=4","--pids-limit=1024"]
```

The refusal itself is the repository's own,
recorded in `ide-release.yml`:

```yaml
# .github/workflows/ide-release.yml, lines 215 to 220
# There is no disk weight in this job: hosted runners do not delegate
# the cgroup io controller, and run 42447009237 of music-player-release.yml (2026-11-08), on the same runner images, failed with
# `crun: opening file 'io.weight' for writing: No such file or directory`. CONTAINER_LIMIT_DISK_WEIGHT=off tells
# package/container-limit/core (read by package/ide/app/bin/container-bounds.ts and the task bodies) to leave the weight out.
CONTAINER_EXTRA_OPTIONS: '--pids-limit=1024'
CONTAINER_LIMIT_DISK_WEIGHT: 'off'
```

GitHub's documentation does not mention `io.weight` at all:
a search of `docs.github.com` on 2026-11-28 returned no result,
so the evidence is that one failed run,
on the images of 2026-11-08,
not on the image a gate would use today.
The switch makes the gate work either way;
what is unproven is the weight itself,
not the gate.

### What a gate on a runner must still answer

No record shows any of the following working on a fresh runner,
and this round measured none of them because none can be measured without a hosted run:

- the `docker.io` pull of the CachyOS base image,
  then a `pacman` transaction with network access;
- `cargo-shadow lean records target/verification` inside the worktree;
- the cgroup read at `/sys/fs/cgroup/.../libpod-<id>.scope` on a runner whose user manager differs;
- container user `1000:1000` against the runner's user 1001 and its `/home/runner` workspace;
- the host seccomp profile edit,
  and the runner's kernel version,
  which the trial never printed (this host runs `7.3.5-2-cachyos`,
  and its podman is 6.2.1 against the runner's 5.7.0);
- the gate's disk footprint.

## The trial of the container gate

This is the session's own step,
not a question for the owner.
The project's rule is that a session acts and reports on local work and on current-repository GitHub mutations,
and a workflow file on a branch of this repository is one of those;
the first trial was made that way.
What the owner is asked is what to do with the result.

The mechanism has one constraint that the previous revision got wrong:
a `workflow_dispatch`-only file on a trial branch cannot start.
"This event will only trigger a workflow run if the workflow file exists on the default branch",
and "Once a workflow has run at least once,
you can dispatch it against any branch or tag via the GitHub API or GitHub CLI" ([the events reference][events reference],
`workflow_dispatch`).
This repository's own release record says the same of its own workflow:
"It cannot be dispatched before it is on the default branch" (releases run in CI,
"Limits").
So a trial that must not land on `main` is started by a push to its own branch,
which is what the first trial did:
its three runs are `push` events on the trial branch.

The trial the decision-3 options rest on:

- one workflow file on a branch,
  triggered by a push to that branch alone,
  adding no required check,
  no schedule and no push trigger for `main`;
- the gate's own task,
  with `CONTAINER_LIMIT_DISK_WEIGHT=off`,
  the free-disk step copied from `ide-release.yml`,
  and steps printing `uname --kernel-release`,
  `podman info`,
  `df --human-readable /` before and after,
  and both containers' `memory.peak`;
- read once,
  against the figures in this file,
  and recorded before any option of decision 3 is chosen.

## The guest arms

Each ZFS arm runs only inside a throwaway guest,
and the guest's bounds are recorded
([`package/git-policy/cli/bin/mvm-guest-bounds.ts`](../../package/git-policy/cli/bin/mvm-guest-bounds.ts)):

- the guest has 8 GiB of RAM:
  a guest is created asking for 8,192 MiB (`doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`,
  "What that costs per guest");
- the guest makes a 6 GiB swap file for itself,
  inside the guest's own disk (`fallocate --length ${SWAP_GIBIBYTES} /swapfile`,
  in the arm scripts),
  "so one `rustc` of this crate's tests at 6.7 to 6.9 GiB does not exhaust the guest's 8 GiB".
  That 6 GiB is guest disk,
  not runner memory,
  and the two do not add;
- one guest-recipe step may take `RECIPE_STEP_TIMEOUT_MS`,
  which is 120 seconds times 60,
  that is two hours.
  It is a bound,
  not a length,
  and no record gives the recipe's wall time on a runner.

So a guest arm asks a runner for 8,192 MiB of guest RAM plus QEMU's own overhead,
and for the guest's disk on top of the image.
One measured arm's charge on this host peaked at 7.98 GiB,
of which 7.71 GiB was resident (the same record,
"What that costs per guest").
Against the trial's 15,104 MiB of `MemAvailable`,
7.98 GiB leaves about 6.8 GiB,
which is just over the gate's 6.6 GiB peak of container memory:
one arm alone fits,
and an arm beside the gate's containers leaves a few hundred MiB for the shell steps,
the page cache and QEMU's overhead,
which is measured nowhere.

The October record on `ubuntu-24.04` says what the device does ([the platform comparison][platform comparison],
"Standard GitHub KVM access disappears after successful QEMU controls" and the section after it):

- "the named `user:1001:rw-` entry disappears after successful QEMU execution",
  and "The bounded raw workaround is a fresh named ACL grant before each QEMU invocation";
- "Its tradeoff is that this does not protect a libvirt operation that opens the device repeatedly";
- under "Corrected actual GitHub guest lifecycle",
  "Standard GitHub,
  Depot's GitHub runner and native Depot CI now each have a passing actual guest lifecycle in this cohort",
  with the child holding UID/GID 1001 and device group 993;
- the run that section names,
  `39125868595`,
  answers 404 today (`gh run view`,
  read 2026-11-28),
  so the record's text is all that is left of it.

## The trial that was made

The first trial was a workflow file on a trial branch,
`.github/workflows/hosted-runner-specimen.yml`,
triggered by a push to that branch alone,
installing nothing that needs a secret,
using no larger runner and registering no self-hosted runner.
It ran three times,
each run teaching the next what to ask.
Every job concluded `success`,
which proves less than it looks:
the boot step carries `continue-on-error: true`,
so a run can end green with a failed guest,
and the second run did exactly that.

- Run `46123229560`,
  2026-11-28T07:53:04Z,
  job 4 seconds:
  `Region: eastus2`,
  `Image: ubuntu-26.04`,
  `Model name: AMD EPYC 9V74 80-Core Processor`,
  `Virtualization: AMD-V`,
  `MemTotal: 16366636 kB`,
  `MemAvailable: 15523076 kB`,
  `/dev/root ext4 145G 58G 87G /`,
  `Docker version 29.4.2`,
  `podman version 5.7.0`,
  `crw-rw---- 1 root kvm 10, 232 Nov 28 07:52 /dev/kvm`,
  and after the grant `opened after the grant`.
  The boot step was skipped.
- Run `46123307937`,
  2026-11-28T07:54:09Z,
  job 17 seconds:
  `Region: westus3`,
  the same AMD model and `AMD-V`,
  `MemAvailable: 15533176 kB`,
  `after the grant: opened, KVM_GET_API_VERSION answered 12`,
  then QEMU ended at once with `qemu-system-x86_64: --memory: invalid option`,
  and the step's own line read `qemu pid 2626, answered=false, seconds to login prompt 1`.
- Run `46123397702`,
  2026-11-28T07:55:20Z,
  job 59 seconds:
  `Region: centralus`,
  `Model name: INTEL(R) XEON(R) PLATINUM 8573C`,
  `Virtualization: VT-x`,
  `MemAvailable: 15467100 kB`,
  `KVM_GET_API_VERSION answered 12`,
  `qemu pid 2600, answered=true, seconds to login prompt 44`,
  and the guest's own line `=== cirros: current=0.6.2 uptime=42.98 ===`.

So two vendors opened the device,
and one of them booted a 512 MiB CirrOS image in 44 seconds;
the AMD runs skipped the boot step once and failed on the QEMU option once.
The trial's workflow file was removed from the branch after the runs,
and until this section nothing durable in this repository held their logs.

What the trial establishes:
a standard public runner gives a working `/dev/kvm` after a named ACL grant;
the runner image carries podman 5.7.0,
Docker 29.4.2 and 87 GB of free disk;
and the device and the grant are documented as unsupported for general use,
so a check built on them is one GitHub may change without notice.

## Unknowns a hosted gate carries

Named here so the trial's prints can answer them,
and so nothing in the decisions rests on them:

- **the cache cap.**
  GitHub's cache is 10 GB per repository on this plan,
  and the repository's own cache policy record has it over that cap:
  "11.3 GB of 10 GB at the first reading,
  7.5 GB after eviction" ([the cache record][ci caching record]).
  Measured today the repository holds 0.99 GB in 10 entries,
  most of it `codeql-trap` entries of 132 MB each.
  A gate that cached its 4.33 GB image plus a cargo cache would press that cap,
  and the owner called paid expansion "prohibitively expensive" (the Depot-era interview).
- **the runner label.**
  `advisory-core.yml` names `ubuntu-latest`;
  `ide-release.yml` names `ubuntu-24.04` because "GitHub moves that label to Ubuntu 26.04 (glibc 2.43) in November 2026",
  and the trial's log printed `Image: ubuntu-26.04`.
  Which image `ubuntu-latest` resolves to on the day of a run is not knowable from a pin,
  and the images are updated weekly.
- **the pull-request path.**
  `advisory-core.yml` admits every `pull_request` and every `workflow_dispatch` without counting,
  so the window removal of decision 1 changes pushes only,
  and an open pull request has always run the checks.
- **memory beside the guests.**
  The gate's 6.6 GiB of container peak,
  plus one measured arm's 7.98 GiB of guest charge,
  is 14.6 GiB against the 15.1 GiB of `MemAvailable` the trial printed,
  which leaves about 0.5 GiB for QEMU's overhead,
  the page cache and the shell steps.
  Two arms in one job do not fit;
  one arm beside the gate's containers is unmeasured.
- **the gate's share of the night's memory pressure.**
  This proposal moves,
  at most,
  one block of about 6.6 GiB and four CPUs per batch.
  How much of the 2026-11-27 out-of-memory came from gates rather than guests is not measured here,
  and the guests are the larger part.

## Decisions for the owner

Six decisions,
each answerable alone.
Each option says what it runs,
what it takes off this machine,
what it cannot do,
what it costs,
an example with its visible result,
and what a yes still leaves undecided.

Every figure about the advisory profile's four admission rules,
and about its group,
comes from one arithmetic this round ran on the 400 push arrival times read from GitHub,
at the 476-second length of the newest executed job;
the scripts and their outputs are in the report of the round,
and the model is stated there:
one run executing and one waiting per ref,
a cap checked against the runs that started executing in its window,
and no attempt to say how GitHub schedules.
The same arithmetic at the other two measured lengths,
401 s and 557 s,
is in the report;
at 557 s the counts differ by about a tenth and no ranking changes.

### 1. What should the advisory window count, and what is it for?

Terms:
the "window" is the 2 per minute,
4 per hour and 8 per day budget in `advisory-core.yml`'s gate;
"counting executions" means counting only runs whose `core` job actually ran,
the way `scorecard.yml`'s gate does;
the "group" is the workflow's own `advisory-core-${{ github.ref }}` concurrency group,
which lets at most one run execute and one wait per ref.

Why it is asked:
the window counts arrivals,
and at 379 pushes a day it admits nothing after its first hour.
What it was for is not in any record.
The nearest record is the owner's own words in the Depot-era interview:
"The user wants sliding-window rate limits,
or an inexpensive way to express them without GitHub's overhead.
They rejected hypothetical burst/debounce questions and requested inspection of actual GitHub CI history",
and "The user highlighted GitHub's 10 GB cache space as another concern" (`doc/handover/depot-trial-grilling.md`).
So the record names a wish for a bound and a rejection of burst questions,
and says nothing about a twenty-minute queue.
Options 3 and 4 remove the window;
if the owner's wish was for a bound at all,
option 3 or 4 answers it with the group instead of a count.

The window came from the withdrawn Depot adoption record,
which named the Scorecard workflow's sliding windows as the starting point and "adjusts from measured capacity" ([the Depot CI adoption record][depot adoption record],
"Execution routing";
the record opens with "withdrawn on 2026-11-01" and "This decision is no longer in force").
That paragraph describes the numbers and says nothing about a queue,
so where the window came from is read,
and what it was for is still the owner's to say.

What each option delivers,
measured,
at the present 379 pushes a day:

- the window as it stands,
  which option 1 keeps:
  3 runs in 25.34 hours,
  none after the day budget filled;
- option 2:
  8 runs a day at most by its own numbers,
  and 13 in the sample's 25.34 hours,
  because its first 8 start before any day's history exists;
- option 3:
  156 runs in 25.34 hours,
  148 a day;
- option 4:
  160 runs in 25.34 hours,
  152 a day.

#### Option 1: keep counting arrivals and keep the numbers as they are

Mechanism:
nothing changes.
The step reads `databaseId` and `createdAt` for the newest 100 runs,
counts every run except the current one,
and refuses when any window is full.
The group stays as it is.

What it takes off this machine:
nothing;
it is the state that admits 3 runs in a day.

What it cannot do:
it cannot admit a run after the day budget has filled,
which takes under an hour at this rate,
so it examines 8 of 379 pushes at best,
in the first hours of a day;
and it counts the runs it refused as reasons to refuse the next.

Cost:
at most 8 runs of 6 minutes 41 seconds to 9 minutes 17 seconds,
about 1.1 to 1.3 hosted job-hours a day,
free,
and no code to change.

Example:
the first push after midnight starts a run;
the eighth fills the day budget;
the next 371 pushes of that day read `- skipped: newest tip <sha> marked pending` in the gate job's summary,
and their `advisory core` job reads `skipped`.

What a yes leaves undecided:
what an admitted run is for,
and what the three red package unit tasks would tell anyone.

#### Option 2: count executions, with the same numbers

Mechanism:
the counting step asks each recent run's `core` job conclusion and skips runs whose job was `skipped` or `cancelled`,
as `scorecard.yml` does with one GraphQL query per 100 runs,
a cursor and a 32-page bound.
The numbers and the group stay.

What it runs:
the profile the withdrawn Depot record named:
the five drift gates,
the static step,
five builds and six unit tasks,
still red for the two missing installs until they are added.

What it takes off this machine:
nothing directly;
it lets the profile run at all,
13 times in 25.34 hours in this round's arithmetic (12 a day),
because the 2-per-minute and 4-per-hour windows bind before the day's 8.

What it cannot do:
it cannot admit more than 8 executions a day,
and the counting query can refuse runs the group would have coalesced.

Cost:
about 1.6 hosted job-hours a day,
free;
a paginated query,
a cursor and a page bound to keep,
where the step now reads one REST page.

Example:
a push at 09:00Z reads the last day's executions,
sees 8 already,
and refuses;
at 10:34Z the day window has not moved,
so it refuses again,
and 371 of the day's pushes read `- skipped` in the gate job's summary.

What a yes leaves undecided:
whether 8 a day is the number the owner wants,
and what happens to the 371 pushes it refuses.

#### Option 3: count executions, 8 an hour and no day cap

Mechanism:
as option 2,
with the day window removed and the hour window at 8;
the group decides what actually executes.

What it runs:
the same profile,
156 times in 25.34 hours in this round's arithmetic (148 a day).

What it takes off this machine:
nothing directly;
it examines 39 percent of the day's pushes.

What it cannot do:
it cannot examine the pushes the group replaces while a run executes (180 of the 400 in this round's arithmetic),
and its counter can still refuse a run.

Cost:
about 19.5 hosted job-hours a day,
free,
which is about 0.8 concurrent jobs of the twenty allowed;
the same query as option 2.

Example:
an hour of 37 pushes admits 8 runs;
the group executes one at a time and replaces the waiting one,
so the seventh run of that hour starts on whichever push was waiting when the sixth ended,
and 63 of the sample's 400 pushes are refused by the hourly count.

What a yes leaves undecided:
whether a counter is worth keeping when the group is what bounds execution.

#### Option 4: remove the window and keep the group

Mechanism:
delete the counting step and the `needs.gate.outputs.allowed` condition,
leaving `advisory-core-${{ github.ref }}` with `cancel-in-progress: false`.
Every push is admitted;
one run executes and one waits per ref.

What it runs:
the same profile,
160 times in 25.34 hours in this round's arithmetic (152 a day),
which is 40 percent of the pushes.

What it takes off this machine:
nothing directly;
it examines the most pushes of the four options.

What it cannot do:
it cannot examine a push made while a run is executing:
that push replaces the waiting one,
so 239 of the 400 pushes in this round's arithmetic never start a run,
and the run that does start examines the commit that was waiting when the slot freed.

Cost:
about 20 hosted job-hours a day,
free,
about 0.8 concurrent jobs of the twenty allowed at all times;
the upkeep is the deletion itself.
The gate job holds no step but the counting one,
so removing the step lets the job and its five-minute timeout go too,
and the `core` job's `needs.gate.outputs.allowed` condition with them.

Example:
push A at 09:00:00Z starts a run;
pushes at 09:02 and 09:04 replace each other as the waiting run;
at 09:07:56 the run for the 09:04 push starts,
and its log ends `task-run [failed] "//package/forbidden-strings/cli:test"` after `error: no such command: nextest` until that install is added.

What a yes leaves undecided:
whether an advisory run on `main` is worth about 0.8 concurrent jobs of the twenty allowed at all times,
and whether the drift gates should report to a person.

Ranking,
by how many of the day's pushes end with a finished check:

4 > 3 > 2 > 1.

- 4 over 3:
  option 4 ends 152 checks a day where option 3 ends 148,
  and its mechanism is the deletion of a step,
  rather than a paginated query that refuses 63 of the sample's 400 pushes and delivers four fewer checks.
- 3 over 2:
  option 3 ends 148 checks a day where option 2 ends 8,
  and examines 39 percent of pushes against 2 percent.
- 2 over 1:
  option 2 ends 8 checks a day where option 1 ends 3 and,
  after its first hour,
  none;
  the two differ only in whether the refused runs count as reasons to refuse.

Costs,
stated and not ranked:
option 3 and option 4 spend about 19.5 and 20 hosted job-hours a day,
option 2 about 1.6,
and option 1 about 1.1 to 1.3.
The minutes are free;
what they hold is,
on average,
about 0.8 concurrent jobs of the twenty allowed for options 3 and 4,
and less than 0.1 for options 1 and 2.
If the owner wants a ceiling on hosted job-hours rather than coverage,
only options 1 and 2 stay under two job-hours a day,
and between them option 2 ends 8 checks a day,
against option 1's 3 in the whole sample and none after the day budget fills.
This file ranks coverage because the minutes are free.

What every option leaves undecided:
whether `advisory-core` should exist at all.
The linter's own record ranks repairing this workflow last of its options,
which is worth reading beside this question ([keeping the repository linter at zero][linter record],
"Ranking").

### 2. When a push arrives while an advisory run is executing, should it be cancelled?

Terms:
`cancel-in-progress` is the workflow group's setting;
`true` means a new push cancels the executing run and takes its place,
`false` means the executing run finishes and the new push waits.

Why it is asked:
decision 1 does not decide this,
and the two settings differ by a factor of three in finished checks.
The revision before this one folded `cancel-in-progress: true` into one of decision 1's own options and asked nothing.

#### Option 1: keep `cancel-in-progress: false`

Mechanism:
the executing run finishes;
the newest waiting push takes the slot when it frees.
This is today's setting.

What it runs:
whatever decision 1 admits,
each run to completion.

What it takes off this machine:
nothing.

What it cannot do:
it cannot examine pushes that arrive while a run executes;
they are replaced by later ones.

Cost:
the job-hours decision 1 names;
nothing extra.

Example:
at 09:00:00Z a run starts;
at 09:07:56 it finishes and the push that arrived at 09:04 starts;
the pushes of 09:05 and 09:06 replaced each other while it waited.

What a yes leaves undecided:
how stale an examined commit may be,
which the push rate decides.

#### Option 2: `cancel-in-progress: true`

Mechanism:
each admitted push cancels the executing run and starts its own.

What it runs:
one run per admitted push,
most of which are cancelled before they finish.

What it takes off this machine:
nothing.

What it cannot do:
it cannot finish a run whose successor arrives within the job's length,
which is 88 percent of the sample's gaps at 476 seconds:
of 400 pushes it starts 400 runs,
finishes 49 and cancels 351.

Cost:
about 50 hosted job-hours a day,
free,
about 2 concurrent jobs of the twenty allowed,
for 46 finished checks a day against option 1's 152.

Example:
a push at 09:00 starts a run,
one at 09:02 cancels it and starts its own,
one at 09:04 cancels that,
so the first run's page reads `cancelled` and its log stops after whichever step was running,
with no `gate-exit-status` line.

What a yes leaves undecided:
whether a cancelled run's partial steps are worth their minutes.

Ranking,
by how many of the day's pushes end with a finished check:
1 > 2.

- 1 over 2:
  option 1 ends 152 checks a day at about 20 job-hours;
  option 2 ends 46 at about 50,
  so it spends two and a half times the minutes for less than a third of the checks.

### 3. Where should the wrapper's container gate run?

Terms:
the gate is `native:test:container`,
the image build,
the compile,
the whole suite and Clippy in containers,
and on this host it is `native:gate` or `native:gate:main-tip`;
the "batch gate" is the project's rule,
one gate on `main` per batch of merges ([merges and publishes do not wait][merge bar record],
"What runs after a merge,
answered 2026-11-23");
a "red" is a failed gate.

Why it is asked:
the gate is the largest single block this machine gives one task:
21 to 42 minutes inside a unit capped at ten CPUs and 20 GiB (`native-gate-caps.ts`),
and the owner's aim is memory.
The owner's rule also says what the gate is not:
"Merging to main shouldn't need a gate or a pass",
and the project chose "A:
one gate on main per batch" over "gate only at a release" and "no gates",
with "A round runs no whole gate of its own" (the same record).

The two hosted options follow the trial in "The trial of the container gate",
and neither is available until it has run.

#### Option 1: the batch gate runs on a hosted runner and the local one does not run

Mechanism:
after each batch the session dispatches one hosted run of the gate instead of starting `native:gate:main-tip`;
the workflow's steps are the gate's own,
with `CONTAINER_LIMIT_DISK_WEIGHT=off`,
four CPUs and a memory bound the runner can hold.

What it takes off this machine:
the whole gate,
per batch:
21 to 42 minutes of it,
the ten-CPU quota and the 20 GiB cap its own unit carries (`native-gate-caps.ts`,
`unitCpuQuota: '1000%'` and `unitMemoryMax: '20G'`),
the 6.6 GiB peak of container memory,
and up to 1,024 processes.

What it cannot do:
it cannot run the guest arms,
the btrfs host controls,
or any task that needs this machine's service manager;
the btrfs controls are a release step (`native:release:host-controls`),
so they are not lost from the gate because they were never in it.
It cannot tell a runner's fault from a product fault in a red,
and a fresh runner's first run builds the derived base with network and cold layers.

Cost:
20 to 42 hosted minutes per batch as this host's spread stands,
unmeasured on a runner,
free;
one workflow file to keep in step with the gate's own tasks,
and the trial run before it can be trusted.

Example:
a batch of six merges ends at 11:20;
the session dispatches the gate at 11:21;
the log's last lines are `Summary [<n>s] <n> tests run: <n> passed` and `gate-exit-status=0`.
No run has printed those lines on a runner;
that is what the trial asks for.

What a yes leaves undecided:
whether the gate's unit-level replay and evidence manifest stay on this machine,
and what a red on an image nobody has run means.

#### Option 2: both run, the hosted one first and the local one on a red

Mechanism:
the hosted gate runs per batch,
and a red hosted gate is re-run locally before a fix round starts,
because a red on a fresh runner may be the runner's own.

What it takes off this machine:
the same,
on every green batch;
on red days the local gate runs as well.

What it cannot do:
it cannot take the gate off the machine on a red day,
and it keeps the local gate's 4.33 GB image and its build cache on this machine.

Cost:
the hosted minutes of option 1 plus a local gate on every red,
free;
two paths that must agree,
and one rule for which of them decides.

Example:
the hosted gate is red on a merge touching the removal code;
the session starts `native:gate:main-tip` on that tip;
its log ends `gate-exit-status=2` with the same failing test,
so the fix round starts from the local log.

What a yes leaves undecided:
who decides a hosted red is suspect,
and how long the local copy stays.

#### Option 3: the local batch gate stays, and no hosted gate is added

Mechanism:
nothing changes;
the batch gate keeps running on this machine as its own capped unit.

What it takes off this machine:
nothing.

What it cannot do:
it cannot keep the largest single block of this machine's memory and CPU off it.

Cost:
as today:
20 to 42 minutes and a 20 GiB bound per batch.

Example:
today,
unchanged:
a batch ends,
the session starts `native:gate:main-tip`,
and its log ends `gate-exit-status=0` after about 21 to 42 minutes.

What a yes leaves undecided:
whether the machine's out-of-memory night comes again with the guests it already bounds.

Ranking,
by how much of a batch's gate work leaves this machine while a red still gets a verdict this repository has measured:

2 > 1 > 3.

- 2 over 1:
  both move a green batch's gate off the machine;
  option 2 also reproduces a red on the path that has measured the gate (1,104.9 s of suite,
  12,517 tests) instead of acting on a red from an image and kernel nobody here has run.
- 1 over 3:
  option 1 moves the whole gate off the machine on every batch,
  which option 3 never does,
  and option 3 keeps the 6.6 GiB and four CPUs where the night's memory ran out.

### 4. Where should the wrapper's TypeScript unit tests and lint run?

Terms:
issue 6584 asks for a proposal on where `//package/git-policy/cli`'s `test:unit`,
`lint:types` and `lint:oxlint` run,
"so that a red one is seen within a batch of merges".
The proposal exists and ranks seven placements:
A inside the release gate,
C at the coordinator's batch tip check,
B in a unit beside the gate,
E in a CI workflow of its own,
D in the advisory fast subset,
F in every round,
G what comparable repositories do,
"A > C > B > E > D > F > G" ([where the TypeScript checks run][placement record],
"Ranking").

Why it is asked:
a hosted workflow is that proposal's E,
ranked fourth,
and choosing it here would answer issue 6584 somewhere other than where it is asked.
The placement record also says what this decision must weigh:
"Nothing on this project's loop reads a push's CI result,
which the three red `forbidden-strings` runs on `main` show" (the same record,
option E's cons),
and this round's own counts agree:
the newest 300 runs of `forbidden-strings.yml` are 92 failures,
85 cancelled and 123 successes,
read at 2026-11-28T08:00-05:00.

#### Option 1: no new workflow; the question stays in its own proposal

Mechanism:
the placement question stays where it is asked,
and decision 6 decides whether any answer becomes a required check.
Until then the coordinator runs the three checks with each batch,
which is what issue 6584's last sentence already does.

What it takes off this machine:
nothing;
the checks are 37.8 seconds of wall time and a 4.5 GB peak in the wrapper's own scopes today.

What it cannot do:
it cannot make the three reds visible to anyone who is not at the batch check.

Cost:
none here.

Example:
the batch check runs `mise run //package/git-policy/cli:test:unit` at `main`'s tip and prints `the gate's checks failed: the TypeScript checks printed Found 1856 warnings and 850 errors.`,
which is the placement record's own form of the refusal.

What a yes leaves undecided:
issue 6584's own question,
which the owner answers there.

#### Option 2: one path-filtered workflow for the wrapper package

Mechanism:
a workflow filtered on `package/git-policy/cli/**` running `lint:types`,
`lint:oxlint` and `test:unit`,
with a memory bound of 4.5 GB and no required check.
This is placement E.

What it takes off this machine:
2.8 s,
12.8 s and 22.2 s of the coordinator's batch check,
and their 0.45,
0.99 and 4.5 GB peaks.

What it cannot do:
it cannot make anyone read the result;
it prints 850 errors today,
so it is red from its first run until issue 6581 is fixed;
and it adds a second place that claims to say whether the package is well.

Cost:
a runner's own setup,
measured on another workflow at about 15 s of install beside 26 s of checkout (run `45754658990`),
plus the checks;
one path filter and a task list to keep.
The minutes are free.

Example:
a push touching `package/git-policy/cli/bin/native-gate-verdict.ts` starts the job,
and its log ends with the line this host's own run printed,
`Found 1856 warnings and 850 errors.`,
so the run is red.

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
one entry per package,
which is 276 `mise.toml` files under `package/<category>/<name>/` counted this round;
the minutes are free,
and the runner's fixed cost is paid per job.

Example:
a push touching `package/logger/core/src/sink/` runs that package's unit task,
which fails five test files today and names them in the log.

What a yes leaves undecided:
whether 276 unread reds are better than one.

Ranking,
by how surely a red on these checks reaches someone who acts on it:

1 > 2 > 3.

- 1 over 2:
  option 1's reds reach the session at the batch check,
  where a person acts;
  option 2's reds reach the Actions tab,
  where this repository's own record says "nothing on this project's loop" looks,
  and its first run is red for 850 errors nobody reads.
- 2 over 3:
  option 2 covers the package whose reds are recorded and dated,
  and option 3 multiplies that unread red across 276 packages with one entry each to keep.

### 5. Should the wrapper's ZFS guest arms move to a hosted runner?

Terms:
an "arm" is one `e2e:` task of the wrapper that boots a guest and drives a ZFS pool inside it;
`mvm` is this machine's guest tool,
with `libvirt`,
`hetzner` and,
since 2026-11-28,
an owner-chosen `ovhcloud` backend ([that decision][ovh record],
"Decision").

Why it is asked:
the guests are the resource the owner bounded at once,
and they are the largest thing this proposal could move.
The record already says where a guest's memory comes from,
and the owner chose it:
"Guests on OVHcloud" alone,
"so a guest uses none of this machine's memory" ([the owner's answers of 2026-11-28][owner answers],
"Where more memory comes from").
That path is decided and not working yet:
"neither billed run produced a machine that answered,
the sizes design came back from its gate read not ready (five High points),
and a third run needs your yes" (`doc/planning/mvm-create-takes-vcpus-and-memory-and-picks-a-size.md`).
What a hosted runner would add is a second way to move a guest,
needing no billed machine.

#### Option 1: keep the arms on this machine, and on the OVHcloud backend when it works

Mechanism:
nothing changes here;
the arms keep booting `mvm` guests locally,
and the OVHcloud backend is finished by its own round and its own questions.

What it takes off this machine:
nothing until that backend lands;
afterwards,
whole guests.

What it cannot do:
it cannot answer whether a runner can run an arm at all.

Cost:
this machine's memory for each guest while the backend is unbuilt,
8 GiB of RAM plus 6 GiB of the guest's own disk for its swap file;
the OVHcloud work is another round's.

Example:
today,
unchanged:
an arm boots a guest,
runs its recipe and prints the arm's own arbitration lines,
and the machine holds 8 GiB for it.

What a yes leaves undecided:
whether the OVHcloud sizes design passes and is built,
and what a runner would have cost instead.

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
because the ACL entry disappears after a QEMU run,
and the record says that grant "does not protect a libvirt operation that opens the device repeatedly";
it rests on a device GitHub documents as unsupported;
and it fits one arm per job and nothing beside it.

Cost:
the recipe's own step bound is two hours,
which bounds the job and is not its length;
the guest wants 8 GiB of a runner's 15.1 GiB available,
and the job's disk must hold the guest's image and its 6 GiB swap file.
The minutes are free;
the upkeep is a second way of booting.

Example:
a dispatch runs one arm and its log prints the KVM open,
the recipe's steps,
the pool's creation and the arm's arbitration lines.
No run has printed them;
this is what the trial would ask for.

What a yes leaves undecided:
whether the other arms follow,
and whether `mvm` gains a runner backend at all.

#### Option 3: move the arms to a hosted runner now

Mechanism:
one job per arm,
as option 2 but for every arm.

What it takes off this machine:
whole guests,
on every arm.

What it cannot do:
the same as option 2,
and it multiplies an unmeasured path by the number of arms.

Cost:
one job of up to two hours and 8 GiB per arm,
free at the point of use,
with a workflow to keep in step.

Example:
the first arm round after it lands boots its guest on a runner,
and its log is the first evidence;
if the device's grant has gone,
the job fails at the QEMU open and the arm's red says nothing about the product.

What a yes leaves undecided:
what to do with the arms when GitHub changes the device again.

Ranking,
by what each delivers toward arms that no longer use this machine's memory,
on a path this repository can keep:

1 > 2 > 3.

- 1 over 2:
  option 1's delivery is a decided backend that rents a real machine by the second;
  option 2 moves no arm off this machine,
  and its answer may be no whatever it measures,
  because GitHub guarantees nothing about the device.
- 2 over 3:
  option 3 commits every arm to a path whose grant disappears after each QEMU run,
  so its arms fail unpredictably where option 2 would have measured that first.

The memory aim is served by the OVHcloud record either way,
which is why the arms rank below the gate:
a gate on this machine holds about 6.6 GiB,
and one measured guest's charge was 7.98 GiB.

### 6. Who reads a hosted red, and should any check become required?

Terms:
a "red" is a failed hosted run;
"required" means a required status check on `main` in branch protection,
which only the owner can set.

Why it is asked:
today nothing is required on `main`,
and nothing on this project's loop reads a push's CI result,
so a hosted red arrives where nobody looks.
The repository's own numbers show what that costs:
of the newest 300 runs of each per-push workflow,
read at 2026-11-28T08:00-05:00,

- `forbidden-strings.yml`:
  92 failures,
  85 cancelled,
  123 successes;
- `git-policy-cli-fuzz.yml`:
  127 failures,
  135 cancelled,
  38 successes;
- `toml-edit-fuzz.yml`:
  102 failures,
  86 cancelled,
  35 successes,
  over 223 runs,
  which is that workflow's whole history;
- `logger-fuzz.yml`:
  52 failures,
  3 cancelled,
  11 successes,
  over the 66 runs of its history.

These counts move between reads:
the second gate read had 100,
83 and 117 for the first workflow and 125,
136 and 38 for the second,
so read them as a shape and not a total.

There is one setting this question half-answers and this proposal never looked at:
GitHub notifies the account that triggered a run when it completes,
if that account has Actions notifications on,
and "You can also choose to receive a notification only when a workflow run has failed" ([the notifications page][notifications page]).
Every push is `Aquaticat`'s,
so that account's own setting may already send a failed run's red to the owner.
Whether it is on is the owner's setting,
not readable here.

#### Option 1: nothing becomes required, and a red reaches a person by a named routine

Mechanism:
the coordinating session reads the hosted runs of the checks it dispatches at the batch check it already runs,
and names a red in its report to the owner;
no branch protection changes.
The owner's own notification setting stays as it is.

What it takes off this machine:
nothing;
it puts a reading duty on the session.

What it cannot do:
it depends on a routine that can leave a check out,
which is what issue 6584 records of the batch check's own commands,
and it does nothing for a red raised by a push no batch covers.

Cost:
one `gh run list` per batch,
and one scheduled reading a day if the owner wants one.

Example:
at the batch check the session runs `gh run list --limit 20 --json conclusion,headSha,workflowName`,
sees a failed `git-policy-cli-fuzz` run on a commit in the batch,
and names it in the report with the commit and the failing step.

What a yes leaves undecided:
who reads a red raised between batches.

#### Option 2: a required check on `main`, once a check is green on `main`

Mechanism:
the owner sets one required status check in branch protection after the chosen check has run green for a while,
and that check's workflow runs on `push` to `main`.

What it takes off this machine:
nothing.

What it cannot do:
it cannot hold the owner's own pushes while `enforce_admins` is off,
which is today's state:
GitHub's rule exempts admins by default,
and the account that made all 400 of the newest pushes is one.
It would refuse the release bot's bump,
whose commit starts no run to satisfy the check (inferred from the release record and two GitHub pages;
not exercised).
It is also unusable while the chosen check is red:
the wrapper's lint is red with 850 errors,
and the three package unit tasks measured on 2026-11-28 are red.

Cost:
the owner's setting,
a change to how the release dispatches its bump,
and a fix round first.

Example:
after the lint is fixed the owner requires the whole-tree lint check;
any non-admin actor's push that lacks the check is then refused with a protected-branch error rather than made to wait,
and the owner's own push to `main` still lands.

What a yes leaves undecided:
how a release dispatches its bump without a check that never runs,
and whether the owner also turns on "Do not allow bypassing the above settings".

#### Option 3: a required check on pull requests only

Mechanism:
the owner requires a check in the ruleset or protection for `pull_request` runs.

What it takes off this machine:
nothing.

What it cannot do:
it gates no merge here,
because merges are local:
the newest merged pull request is #1776 of 2026-10-19 (`gh pr list --state merged --limit 3`),
and every newer one is an outside account's closed one (#6113,
#3472,
#3471).
It also cannot be built from a path-filtered workflow:
a workflow skipped by path filtering leaves its check "in a 'Pending' state",
as GitHub's page puts it,
and a pull request that requires it is blocked from merging.

Cost:
the owner's setting and a check that runs almost never.

Example:
an outside contributor's pull request waits for a check that no merge here was ever going to wait for,
and its check reads `Pending` until someone bypasses it.

What a yes leaves undecided:
what the setting was for.

Ranking,
by whether a red reaches someone without breaking how work lands here:

1 > 2 > 3.

- 1 over 2:
  option 2 refuses pushes for a check that cannot run on the bump,
  and it does not hold the pushes that actually land work;
  option 1 reaches a person through a routine that exists.
- 2 over 3:
  option 2 at least refuses the bot's bump and a fork's merge,
  both of which are non-admin actors,
  where option 3 refuses a path this repository does not merge through.

What a yes to any option leaves undecided:
how long a red may stand,
and whether a round or a person acts on it.

## Combinations that cannot be built

Each with the reason that holds,
and the label of what the reason rests on.

- **A required check on `main` satisfied by the release's current bump push.**
  The bump commit starts no workflow (read:
  the release record's "Consequences",
  and GitHub's sentence about `GITHUB_TOKEN`),
  so the check can never pass,
  and the push is refused rather than delayed.
  Buildable only by changing how the release dispatches its publish,
  or by adding the bot to a bypass list,
  which is not checked.
- **A required check produced by a path-filtered workflow.**
  A skipped workflow leaves its check `Pending` (read:
  the triggering page),
  and a required check that never resolves blocks the push or the pull request forever.
  This binds decision 6 option 2 or 3 paired with decision 4 option 2 or 3.
- **A guest arm beside the gate's containers in one 16 GB job.**
  One measured arm's guest charge,
  7.98 GiB,
  plus 6.6 GiB of container peak is 14.6 GiB against the trial's 15.1 GiB of `MemAvailable`,
  which leaves about 0.5 GiB for QEMU's overhead,
  the shell steps and the page cache (measured:
  the trial's `MemAvailable`,
  the gate's peaks and the guest's charge;
  inferred:
  the sum is too close).
- **A hosted gate for every wrapper-touching push under the one-gate rule.**
  Not impossible:
  about 105 wrapper-touching pushes a day (measured:
  111 of 399 push ranges) at 20 to 42 minutes each is 35 to 73 job-hours a day,
  which is one to four of the twenty slots.
  It is unwanted by the owner's rule,
  which asks for one gate per batch and says a merge needs no gate,
  and a gate on a push holds no merge.
- **A finished check on most pushes with `cancel-in-progress: true`.**
  At 476 seconds of job and the sample's gaps,
  351 of 400 started runs are cancelled and 49 finish (this round's arithmetic),
  so the setting cannot deliver a verdict on most pushes.

Combinations that look sealed and are not:

- decision 1 option 4 with decision 2 option 2:
  cancelling is reachable with the window gone,
  and the arithmetic for that pair is in the report (400 starts,
  49 finishes,
  50 job-hours).
- decision 1 option 1 with decision 2 option 2:
  also reachable,
  and there the cancel almost never fires,
  because 8 runs a day rarely overlap.
- decision 3 option 2 with decision 4 option 2:
  a hosted batch gate and a hosted per-package workflow are separate files and can both exist.

## What this round did not measure

- Anything on a hosted runner:
  no workflow was dispatched,
  no file under `.github/` was edited,
  and no workflow file was added.
- The gate's wall time,
  disk footprint,
  kernel and podman behaviour on a runner.
- Whether OpenZFS builds and loads in a nested guest on a runner,
  and how long its recipe takes.
- Whether a required status check refused the release bot's bump,
  which needs a write.
- How much of the 2026-11-27 out-of-memory came from gates rather than guests.
- Whether the pnpr config gate and the Rust lockfile gate pass on `main` today.
- The owner's Actions notification setting,
  which is an account setting,
  not a repository one.

## Gate reads of this proposal

Two,
both same-vendor second passes,
as the status says.
Their findings and this revision's answer to each are in the report of the run that made this revision,
finding by finding,
with the command or the line each answer rests on.
What this revision changed is in that report,
which names the file's lines and the commands;
an earlier revision's changelog does not belong in a proposal the owner reads to decide.

[owner answers]: ../decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md
[oomd record]: ../troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md
[linter record]: ../planning/repository-linter-stays-at-zero.md
[placement record]: ../planning/git-policy-cli-where-the-typescript-unit-tests-and-lint-run.md
[releases record]: ../decision/releases-run-in-ci.md
[merge bar record]: ../decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md
[ovh record]: ../decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md
[platform comparison]: ../troubleshooting/depot-ci-versus-github-runner-platforms.md
[runners reference]: https://docs.github.com/en/actions/reference/runners/github-hosted-runners
[concepts page]: https://docs.github.com/en/actions/concepts/runners/github-hosted-runners
[events reference]: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows
[triggering page]: https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow
[protected branches]: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches
[notifications page]: https://docs.github.com/en/actions/concepts/workflows-and-actions/notifications-for-workflow-runs
[concurrency page]: https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency
[limits page]: https://docs.github.com/en/actions/reference/limits
[ci caching record]: ../decision/ci-caching.md
[depot adoption record]: ../decision/depot-ci-adoption.md
