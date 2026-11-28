# GitHub's hosted runners for the package checks, and a guest trial

Status:
a proposal for the repository owner,
unapproved and non-final.
Nothing here is adopted:
no workflow file is added,
no required check changes,
and no review from a vendor other than its writers' has read it.

Gate reads of this proposal:
three,
all same-vendor second passes by the vendor of its writers,
none an independent review.
The first,
on 2026-11-28,
returned five High,
nine Medium and seven Low findings;
the second,
later the same day,
four High,
ten Medium and nine Low;
the third,
also on 2026-11-28,
two High,
ten Medium and ten Low,
and found the file not ready to put to the owner.
Those counts are the reads' own.
This revision answers the third read finding by finding;
the answers are in the report of the run that made this revision,
with the command or the line each one rests on,
because all three reads are kept outside the repository and expire a day after their last change.

Written by Pi runs on 2026-11-28.
This revision was made in the worktree `.claude/worktrees/pi-github-hosted-runners-proposal-third-revision-the-guest-arms-moved-and-what-every-run-returns`,
on the branch `docs/planning-github-hosted-runners-proposal-third-revision-the-guest-arms-moved-and-what-every-run-returns`,
from `main` at `b6d8e0020c`.

Every figure states how it was measured and when,
or which record it is read from.
A claim that rests on reading code,
a record or a vendor page says so in the sentence that makes it;
one that is an inference says whose it is.
The arithmetic behind the admission figures is committed beside this file,
with the arrival times it reads,
so that every figure it prints can be rerun after the round's own scratch expires:
see `github-hosted-runners-for-package-checks-and-a-guest-evidence/` at the end of this file.

## What the owner is asked

The aim this proposal is written against is **inferred**:
this round's reading of three records,
not a decision of the owner's.
It is that the work worth moving off this machine is work that competes for its memory.
The machine ran out of memory on the night of 2026-11-27,
with nineteen rounds and ten guests alive ([the owner's answers of 2026-11-28][owner answers],
"The rounds that were running,
and what bounds a later batch";
[the troubleshooting record][oomd record]),
and the owner then bounded a later batch by the guests alive at once.

What that memory is spent by has changed since the night.
The owner's rule of 2026-11-28 puts every guest on OVHcloud from 2026-11-28T08:00-05:00,
so no guest of a round holds this machine's memory any more:
"from 2026-11-28T08:00-05:00 no guest work runs on this machine:
no round makes a local guest or starts a command in one,
and no brief names one" ([the OVHcloud record][ovh record],
"All guest work runs on OVHcloud,
2026-11-28").
So the largest block this proposal can still move is the container gate's:
about 6.6 GiB of container memory and four CPUs per batch,
inside a unit capped at ten CPUs and 20 GiB.
Decision 2 carries that aim alone,
and no option here moves a guest back onto this machine.

The way to reject that aim is to choose,
in every question,
the option that moves least:
decision 1 option 1,
decision 2 option 3,
decision 3 option 1,
decision 4 option 1,
decision 5 option 1.

Five questions,
each answerable alone.
The recommended option is named with what its recommendation rests on.

1.  **What should the advisory window count,
    and what is it for?**
    What the window is for is the second half of the same question.
    Recommended:
    remove the window and keep the workflow's existing concurrency group (option 4 of 4).
    **Arithmetic on measured arrival times**:
    at the pushes' present rate that starts runs on 152 pushes a day,
    where the window as it stands starts none after its first two hours (the arithmetic and its input are in
    `github-hosted-runners-for-package-checks-and-a-guest-evidence/`).
    Every one of those runs ends red until `cargo nextest` and `monochromatic-lint` are installed in the job
    (see "What the advisory profile needs before it can be green,
    measured").
    That is the profile's state and not the window's,
    so it is true under every option of this question:
    a yes here buys 152 red runs a day,
    and 239 of the sample's 400 pushes show `cancelled`,
    because a newer push replaced the run that was waiting.
    Whether a red then notifies anyone is not this repository's setting:
    GitHub notifies the account that triggered a run when it completes,
    "if you enable email or web notifications for GitHub Actions",
    and that account may "choose to receive a notification only when a workflow run has failed"
    ([the notifications page][notifications page]).
    Every push here is the owner's account's,
    and that account's setting is not readable from this repository.
    The recommendation moved between revisions:
    an earlier revision of this file ranked keeping the window first,
    by how often the profile ran and how much of a day that covered against the hosted minutes spent;
    ranking by finished checks alone,
    because the minutes are free,
    puts the window last.
2.  **Where should the wrapper's container gate run?**
    Recommended:
    on a hosted runner per batch,
    with a red re-run locally (option 2 of 3),
    after one trial run of the gate on a runner.
    **Inferred**:
    no gate has run on a runner,
    so a green batch's verdict would rest on an unmeasured image and kernel for the first time;
    the local re-run is what keeps a red explainable.
    A yes puts a workflow file on `main`,
    dispatched by the session once per batch;
    it does not change what a release waits for,
    which decision 2 states.
3.  **Should the wrapper's ZFS guest arms gain a hosted second path?**
    Recommended:
    no;
    the arms run on OVHcloud,
    where the owner's rule of 2026-11-28 put every guest (option 1 of 2).
    **Read and measured**:
    one arm of one driver has run there,
    on a four-core 16 GiB machine,
    and GitHub documents nested virtualization on a runner as unsupported;
    no arm's recipe has run on a runner.
    The session runs a trial of one arm on a runner either way,
    as its own step and not as part of this question,
    and records what it prints under "The guest arms".
4.  **Should any check become required?**
    Recommended:
    nothing becomes required (option 1 of 3).
    **Read**:
    `main` protects nothing required today,
    and the owner's own pushes bypass branch protection by GitHub's default,
    so a required check would not hold them;
    it would refuse the release's bump push,
    whose commit starts no run that could satisfy it.
    Who reads a red is a separate question,
    question 5,
    answerable either way beside this one.
5.  **Who reads a hosted red?**
    Recommended:
    the session names each red at the batch check it already runs (option 2 of 3).
    **Read**:
    nothing on this project's loop reads a push's CI result today,
    and the account that pushes is the owner's,
    whose own notification setting may or may not be on.
    This question and question 4 can both be answered;
    neither depends on the other,
    and the recommended option of each can be taken with or without the other.

Which of these wait for a trial:

- decision 2 waits:
  both hosted options follow one trial run of the gate on a runner,
  and the trial is the session's own step (see "The trial of the container gate").
  It is a workflow file on a branch of this repository,
  started by a push to that branch:
  it spends free hosted minutes,
  adds no required check and no trigger on `main`,
  and the session reads its log and records its figures here before any option of decision 2 is chosen.
- all the others can be answered now.

Two things this file settles itself,
and does not ask:

- whether a newer push cancels a running advisory run:
  it keeps `cancel-in-progress: false`,
  which the arithmetic settles (see "Two settings this file adopts unasked");
- where the wrapper's TypeScript unit tests and lint run:
  that is issue 6584's question,
  answered there (the same section).

Three questions a record or a setting half-answers,
shown beside the question where they belong:

- what the advisory window was for:
  an agent's note of a Depot-era interview,
  in the third person,
  whose Depot adoption was withdrawn on 2026-11-01 (decision 1);
- where a guest's memory comes from:
  the owner's rule of 2026-11-28,
  which puts every guest on OVHcloud (decision 3);
- whether a session may run a trial workflow itself:
  the project's rule that a session acts unasked on current-repository GitHub mutations (decisions 2 and 3).

What a yes to each recommendation leaves undecided,
in one clause each:

1.  whether the profile should exist at all,
    and whether its drift gates should report to a person;
2.  the memory and CPU bounds the hosted gate gets,
    what a red on it means,
    and whether the gate's evidence manifest stays on this machine;
3.  whether one arm's recipe completes in a nested guest on a runner,
    and what to do when GitHub changes the device again;
4.  how a release's bump push is dispatched without a check that never runs on it;
5.  who acts on a red raised between batches.

Seven combinations,
each labelled cannot be built,
can be built or an unmeasured tight fit,
are listed under "Combinations that cannot be built,
and ones that can".

Words this file uses,
each defined once here rather than at each use:

- the **advisory window**:
  the 2-per-minute,
  4-per-hour and 8-per-day budget in `advisory-core.yml`'s gate job,
  which decides whether a push's advisory run starts at all;
- the **profile**:
  what `advisory-core.yml`'s `core` job runs,
  the five drift gates,
  a static check of the changed files,
  five builds and six unit tasks;
- a **drift gate**:
  a check that a generated file matches what generates it;
- an **advisory verdict**:
  the result of an advisory run,
  which nothing requires and no merge waits for;
- the **batch gate**:
  one run of the wrapper's container gate on `main` per batch of merges;
- a **red**:
  a failed run.

The sections between here and there carry the evidence:
what a push does today (measured),
what the advisory profile needs before it can be green (measured),
what a hosted runner is and is not (read from GitHub's pages and this repository's settings),
what the checks need (measured),
what the container gate needs (read from its source,
measured on this host),
what a guest arm asks for (read from its bounds,
and one arm's run on OVHcloud),
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
# .github/workflows/advisory-core.yml, line 46, as written
          runs=$(gh run list --repo "$GITHUB_REPOSITORY" --workflow advisory-core.yml --limit 100 --json databaseId,createdAt --jq '.' 2>/dev/null || echo '[]')

# the same file, lines 61 and 63, as written
            echo "- budgets: 2 per minute, 4 per hour, 8 per day"
          if [ "$minute" -ge 2 ] || [ "$hour" -ge 4 ] || [ "$day" -ge 8 ]; then
```

It reads `databaseId` and `createdAt` only,
so a run whose `core` job was skipped counts against the windows that skipped it.
At 379 pushes a day the day window of 8 is full within the first two hours of every day and never empties.
Arithmetic on the same 400 arrival times,
with today's rule,
admits 4 runs in the 25.34 hours,
all of them before the day window filled;
the other 396 pushes are refused by the count.
The step drops the current run by `databaseId` before counting,
which is why the rule admits 4;
the committed script counts the arriving push as well and admits 3,
and both numbers are in its header.
Neither is what the record shows:
the `core` job is `skipped` on every one of the newest hundred runs,
so in the measured stretch the count admitted nothing,
and the arithmetic starts from the empty history only the first hours of a day have.

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
which is the fifth of the six tasks that step runs;
the sixth,
`//package/linter/monochromatic-lint:test`,
never starts,
and neither unit task can start on a runner as things stand:
the two crates' builds took 121 and 230 seconds in that log,
and the tests behind them ran nowhere.
So a green run is longer than 476 seconds by an unmeasured amount,
and every "finished checks a day" figure in this file is an upper bound.

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
  which is why the trial runs landed in four regions on three processor models.
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
- 98 of the newest 100 commits on `main` were committed by `Aquaticat`
  and two by `web-flow`,
  GitHub's own web-interface committer
  (`gh api repos/.../commits?sha=main&per_page=100`,
  read at 2026-11-28T08:2x-05:00;
  a read of the same window hours before had 99 and 1,
  which is the window moving);
  the two are the release bumps of 2026-11-28T12:15:25Z and 13:15:10Z,
  both `github-actions[bot]` commits that landed on protected `main`,
  so today's protection does not stop a bump from landing;
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
at `1c8acd5bfb` on 2026-11-27,
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
4.5 GB at `1c8acd5bfb`,
and 1.8 GB once issue 6580's fix is in the tree,
which the same record gives at `64e3eb1a55` ("What this does not settle",
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
The four trial runs measured `MemAvailable` of 15,159,
15,170,
15,104 and 14,922 MiB at job start,
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
- the twelve gate logs of 2026-11-27 and 2026-11-28 under `~/temp/agent/gate/` span,
  from file creation to last write,
  20.4,
  21.8,
  22.1,
  22.1,
  23.0,
  25.4,
  26.1,
  30.9,
  31.0,
  32.3,
  38.6 and 41.8 minutes (median 26.1),
  which is an upper bound on each unit's length because a log carries no timestamps;
  the method matches the two recorded start-to-stop figures within 12 s and 4 s,
  on the two gates it can be compared with,
  and the count was eleven when this file first counted it,
  the twelfth being a gate whose log name begins `20261128T080143`;
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
  and its anonymous pull is counted per address,
  whose limit no record here reads,
  then a `pacman` transaction with network access;
- `cargo-shadow lean records target/verification` inside the worktree;
- the cgroup read at `/sys/fs/cgroup/.../libpod-<id>.scope` on a runner whose user manager differs;
- container user `1000:1000` against the runner's user 1001 and its `/home/runner` workspace;
- the host seccomp profile edit,
  and the runner's kernel version,
  which the trial never printed (this host runs `7.3.5-2-cachyos`,
  and its podman is 6.2.1 against the runner's 5.7.0);
- which CPU level the base image resolves to there,
  because `-v4` needs AVX-512 and `-v3` AVX2,
  and the trial's three processor models were not read for either;
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
its four runs are `push` events on the trial branch.

A hosted gate dispatched afterwards needs its own file on `main`,
because a dispatch of a file that exists only on a branch cannot start
(the same page;
releases run in CI,
"Limits").
The trial's own push also starts every other workflow of this repository whose filters match that branch,
which the trial's record did not check.

The trial the decision-2 options rest on:

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
  and recorded before any option of decision 2 is chosen.

## The guest arms

Guests left this machine by the owner's rule:
"from 2026-11-28T08:00-05:00 no guest work runs on this machine" ([the OVHcloud record][ovh record],
"All guest work runs on OVHcloud,
2026-11-28"),
and a driver takes its backend from the environment variable `MVM_BACKEND`,
so an arm's guest is an OVHcloud machine unless a caller says otherwise (merge `832b3e25c2`,
the same record).

One arm of one driver has run there,
recorded at 2026-11-28T08:16:54-05:00 from the report of the round that moved the guest drivers
(merge `034df18341`,
the same record):
the driver `e2e:doctor-zfs-route` ran on one OVHcloud machine of four cores and 16 GiB,
through the shared guest steps and with no arm script changed,
and its steps took create 75 s,
prepare 7 min 5 s,
the crate's build in the guest 9 min 40 s,
the arm 2 min 3 s,
and destroy 35 s.
Two of the driver's three guest tests passed;
the third differs from what the arm expects,
which the record words as the arm expecting the cancel command's check to answer `unknown`
while two steps' facts are unread,
where the product answers `pass`,
and no local control was run for it.

What does not run there:
the recipe's steps whose subject is a reboot or a power cut,
because `mvm` never stops or resumes an OVHcloud machine,
and the step 22 measurement driver,
whose wait reads the host's own side of the folder a local guest shares.
Under the owner's rule both of those run nowhere,
and a nested guest on a runner is the only path left that could give them a home.

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
of which 7.71 GiB was resident.
That figure is from `doc/planning/mvm-guest-memory-held-for-a-rounds-life.md` ("What that costs per guest"),
which the owner shelved on 2026-11-26 until the crate split,
so it measures a guest building today's single crate and not what an arm would take on a runner.
Against the trial's `MemAvailable`,
15,104.6 MiB (14.75 GiB) at its third run and 14,921.7 MiB (14.57 GiB) at its fourth,
7.98 GiB leaves about 6.8 and 6.6 GiB:
one arm alone fits.
An arm beside the gate's containers does not:
the two come to 14.59 GiB,
which leaves 0.16 GiB at the third run's figure and nothing at the fourth's,
for QEMU's overhead,
the shell steps and the page cache.

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
It ran four times,
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
- Run `46124148349`,
  a `push` to the same branch at 2026-11-28T08:05:10Z,
  concluded `success`:
  `Region: us-west-2`,
  `Image: ubuntu-26.04`,
  `Model name: Intel(R) Xeon(R) 6975P-C`,
  `Virtualization: VT-x`,
  `MemTotal: 16077512 kB`,
  `MemAvailable: 15279824 kB`,
  `after the grant: opened, KVM_GET_API_VERSION answered 12`,
  `qemu pid 2741, answered=true, seconds to login prompt 45`,
  and `=== cirros: current=0.6.2 uptime=42.86 ===`.

So two vendors opened the device,
and two Intel runs booted a 512 MiB CirrOS image,
in 44 and 45 seconds,
while the AMD runs skipped the boot step once and failed on the QEMU option once:
no AMD run has booted a guest.
The fourth run printed the lowest `MemAvailable` of the four and a third processor model,
so the runner's CPU is not limited to the two models the first runs saw.
The trial's workflow file was removed from the branch after the runs,
and until this section nothing durable in this repository held their logs.

What the trial establishes:
a standard public runner gives a working `/dev/kvm` after a named ACL grant;
the runner image carries podman 5.7.0,
Docker 29.4.2 and 87 GB of free disk;
and the device and the grant are documented as unsupported for general use,
so anything built on them is what GitHub may change without notice.

## The session's trial of one guest arm

This is the session's own step and not a question,
the same act as the gate's trial:
a workflow file on a branch of this repository,
started by a push to that branch,
spending free hosted minutes and adding no trigger or required check on `main`.
It runs whatever the owner answers to decision 3,
because the answer applies to what its log shows,
and its figures are recorded here before that answer is read as settled.

- one job,
  with `setfacl --modify "u:$(id --user):rw" /dev/kvm` before the QEMU invocation,
  a guest of 8,192 MiB under `qemu-system-x86_64 --accel kvm`,
  the OpenZFS recipe inside the guest,
  and then the arm's own steps;
- steps printing `uname --kernel-release`,
  `free --mebi`,
  `df --human-readable /` before and after,
  the QEMU process's resident size while the guest runs,
  and the device's ACL before and after;
- read once,
  against the figures in this file,
  and recorded before any option of decision 3 is chosen.

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
  `ide-release.yml` names `ubuntu-24.04` because "GitHub moves that label to Ubuntu 26.04 (glibc 2.43) in November 2026".
  What the label resolved to is in this repository's own records:
  the dispatch log of 2026-11-25 prints `Image: ubuntu-26.04` for both of its jobs (run `45531170934`,
  `Set up job`),
  and all four trial runs printed the same.
  What a pin cannot say is which image a later run resolves to,
  because the images are updated weekly.
- **the pull-request path.**
  `advisory-core.yml` admits every `pull_request` and every `workflow_dispatch` without counting,
  so the window removal of decision 1 changes pushes only,
  and an open pull request has always run the checks.
- **memory beside the guests.**
  The gate's 6.6 GiB of container peak,
  plus one measured arm's 7.98 GiB of guest charge,
  is 14.59 GiB against the 14.75 GiB of `MemAvailable` the trial's third run printed
  and the 14.57 GiB of its fourth,
  which leaves 0.16 GiB at best and nothing at the lowest figure,
  for QEMU's overhead,
  the page cache and the shell steps.
  Two arms in one job do not fit;
  one arm beside the gate's containers fits at no figure the trial printed.
- **the CPU level the gate's image needs.**
  The gate's base image is `docker.io/cachyos/cachyos-v4:latest` on a CPU with AVX-512
  and `cachyos-v3` otherwise ([the base distro record][base distro record]),
  and the resolver picks on the host that builds and runs it.
  The four trial runs saw three processor models and no AVX-512 reading,
  so which level a runner's CPU can run,
  and whether a `-v4` image starts there at all,
  is what the trial's `--help` reading of `/lib64/ld-linux-x86-64.so.2` would print.
- **the anonymous pull of the base image.**
  `native-test-base.ts` pulls `docker.io/cachyos/cachyos-v4:latest` on every run,
  and Docker Hub counts anonymous pulls per address,
  so a runner starting many gate jobs at once may meet a limit this file did not read.
  A trial that prints the pull's answer would settle it.
- **the gate's share of the night's memory pressure.**
  This proposal moves,
  at most,
  one block of about 6.6 GiB and four CPUs per batch.
  How much of the 2026-11-27 out-of-memory came from gates rather than guests is not measured here,
  and since 2026-11-28T08:00-05:00 no guest of a round holds this machine's memory,
  so the gate's block is what is left to move.

## Decisions for the owner

Five questions,
each answerable alone.
Each option says what it runs,
what it takes off this machine,
what it cannot do,
what it costs,
an example with its visible result,
and what a yes still leaves undecided.
Two further settings this file adopts itself,
with the owner's veto open,
are stated after them,
with what settles each.

Every figure about the advisory profile's four admission rules,
and about its group,
comes from one arithmetic run on the 400 push arrival times read from GitHub,
at the 476-second length of the newest executed job.
It is committed beside this file,
as `github-hosted-runners-for-package-checks-and-a-guest-evidence/admission-arithmetic-over-those-arrivals.ts`
with the arrival times it reads and the output it prints,
so a reader can rerun every figure here after this round's scratch expires:

```sh
# in this folder, with node on the path
node admission-arithmetic-over-those-arrivals.ts advisory-core-newest-400-push-arrivals-2026-11-28.json
```

Its model is stated in the script's own header:
one run executing and one waiting per ref,
a cap checked against the runs that started executing in its window,
and no attempt to say how GitHub schedules.
The same arithmetic at the other two measured lengths,
401 s and 557 s,
is in its output.
At 401 s the counts move further apart (option 3's 162 against option 4's 187);
at 557 s options 3 and 4 both start and finish 142 runs,
so that pair is a tie and only their costs separate them.
An independent simulation of the same arrivals also ties them at 557 s and at every longer length,
which is where the hour cap stops binding:
it can only refuse while eight back-to-back starts fit inside one rolling hour,
that is while a job is shorter than about 514 seconds (3,600 divided by 7),
and a green run is longer than the red run that stopped at its fifth of six tasks.
Another script of the earlier round ordered the same group differently and gave 120 starts at 476 s;
its branch for a run that finished while a push was waiting started the waiting push at the wrong time
and dropped the arriving one,
so the figure that holds is this script's 160,
and the other went with that round's scratch.

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
and at 379 pushes a day it admits nothing after its first two hours.
What it was for is not in any record.
The nearest record is an agent's note of a Depot-era interview,
written in the third person,
and the Depot adoption the interview served was withdrawn on 2026-11-01:
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
in arithmetic on the measured arrivals,
at the present 379 pushes a day:

- the window as it stands,
  which option 1 keeps:
  4 runs in 25.34 hours by the workflow's own rule (the committed script counts the arriving push and gives 3),
  none after the day budget filled,
  and none at all in the measured stretch,
  where the `core` job is `skipped` on 100 of the newest 100 runs;
- option 2:
  8 runs a day at most by its own numbers,
  and 13 in the sample's 25.34 hours,
  because its first 8 start before any day's history exists and five more start as those age out;
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
it is the state that admits 4 runs in a day by its own rule,
and admitted none in the measured stretch.

What it cannot do:
it cannot admit a run after the day budget has filled,
which takes two hours at this rate,
so it examines 8 of 379 pushes at best,
in the first hours of a day;
and it counts the runs it refused as reasons to refuse the next.

Cost:
at most 8 runs of 6 minutes 41 seconds to 9 minutes 17 seconds,
about 0.9 to 1.2 hosted job-hours a day,
free,
and no code to change.

Example:
the first push after midnight starts a run,
and by the end of the second hour eight have started,
four in the first hour and four in the second;
every push after that,
371 of the day's 379,
reads `- skipped: newest tip <sha> marked pending` in the gate job's summary,
and its `advisory core` job reads `skipped`.

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
it lets the profile run at all.
Its sample count is 13 in 25.34 hours,
which is the sample's own edge:
the first 8 start before any day's history exists,
and 5 more start as those age out of the day window in its last 1.3 hours.
In steady state the rule admits 8 a day,
which is 1.06 job-hours at 476 seconds;
the sample's 13 runs are 1.6.

What it cannot do:
it cannot admit more than 8 executions a day,
and the counting query can refuse runs the group would have coalesced.

Cost:
about 1.6 hosted job-hours in the sample and 1.06 in steady state,
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
  option 4 ends 152 checks a day where option 3 ends 148 at the 476-second length,
  and at 557 s the two tie at 142 each;
  what separates them in delivered terms is that option 3's hourly counter refuses 63 of the sample's 400 pushes
  while the cap binds,
  which it does only at job lengths under about 514 seconds.
- 3 over 2:
  option 3 ends 148 checks a day where option 2 ends 8,
  and examines 39 percent of pushes against 2 percent.
- 2 over 1:
  option 2 ends 8 checks a day where option 1 ends 4 by its own rule and,
  after its first hours,
  none;
  the two differ only in whether the refused runs count as reasons to refuse.

Costs,
stated and not ranked:
option 3 and option 4 spend about 19.5 and 20 hosted job-hours a day,
option 2 about 1.6 in the sample and 1.06 in steady state,
and option 1 about 0.9 to 1.2.
The minutes are free;
what they hold is,
on average,
about 0.8 concurrent jobs of the twenty allowed for options 3 and 4,
and less than 0.1 for options 1 and 2.
Option 4's mechanism is the deletion of a step where option 3 keeps a paginated query,
a cursor and a page bound,
and that is a cost beside the coverage,
not a reason to rank it above option 3.
If the owner wants a ceiling on hosted job-hours rather than coverage,
only options 1 and 2 stay under two job-hours a day,
and between them option 2 ends 8 checks a day,
against option 1's 4 in the whole sample and none after the day budget fills.
This file ranks coverage because the minutes are free,
and an earlier revision of this file ranked keeping the window first instead,
by how often the profile ran and how much of a day that covered against the minutes spent.

What every option leaves undecided:
whether `advisory-core` should exist at all.
The linter's own record ranks repairing this workflow last of its options,
which is worth reading beside this question ([keeping the repository linter at zero][linter record],
"Ranking").

### 2. Where should the wrapper's container gate run?

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
Both put a workflow file on `main`,
which the session adds and dispatches once per batch:
its actions are pinned by full SHA from this repository's allow-list,
its token is the default read-only one,
and its steps have to be kept in step with the gate's own tasks.
Neither changes what a release waits for.
What a real release of the wrapper waits for is `native:release:host-controls` on this host
(`package/crate-release/cli/releasable-crates.json`,
the `dispatchGate` entry,
and that package's README,
"The dispatch gate of a crate"),
which a runner cannot run;
and the gate's container is what the release path sizes,
in its own source's words:
"A release waits for the gate" (`package/git-policy/cli/bin/native-gate-container-limits.ts`,
its header).
So a hosted gate moves the batch gate and leaves the release path where it is.

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
the log's last two lines have to be this host's own,
`Summary [1104.852s] 12517 tests run: 12517 passed (6 slow), 51 skipped` and `gate-exit-status=0`.
No run has printed them on a runner;
that is what the trial asks for,
and a runner's suite may take longer than these 1,104.9 seconds.

What a yes leaves undecided:
whether the gate's evidence manifest (`native-gate-manifest.ts`,
written into `target/verification/<run-id>/`)
and its verdict reader `native:gate:verdict` stay on this machine,
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
whether the machine's out-of-memory night comes again with the gate as the largest block it still holds.

Ranking,
by how much of a batch's gate work leaves this machine,
counting a red only where its verdict is one this repository has measured:

2 > 1 > 3.

- 2 over 1:
  both move a green batch's gate off the machine;
  on a red,
  option 2 delivers a verdict from the path that has measured this gate (1,104.9 s of suite,
  12,517 tests),
  where option 1 delivers one from an image and kernel nobody here has run.
- 1 over 3:
  option 1 moves the whole gate off the machine on every green batch,
  which option 3 never does,
  and option 3 keeps the 6.6 GiB and four CPUs here on every batch.

### 3. Should the wrapper's ZFS guest arms gain a hosted second path?

Terms:
an "arm" is one `e2e:` task of the wrapper that boots a guest and drives a ZFS pool inside it;
`mvm` is this machine's guest tool,
with `libvirt`,
`hetzner` and,
since 2026-11-28,
an owner-chosen `ovhcloud` backend ([that decision][ovh record],
"Decision").

Why it is asked:
the guests were the resource the owner bounded at once,
and the owner's rule of 2026-11-28 put every one of them on OVHcloud,
so no answer here can move a guest back onto this machine.
That rule is decided:
"from 2026-11-28T08:00-05:00 no guest work runs on this machine" ([the OVHcloud record][ovh record],
"All guest work runs on OVHcloud,
2026-11-28"),
and it ended the option of keeping the arms local.
What a hosted runner could add is a free second path that needs no billed machine,
and the question is whether to keep it after the session's trial has shown what it prints.
Two facts bound that:
GitHub documents nested virtualization on a runner as unsupported,
and no arm's recipe has run on a runner;
while one arm of one driver has run on OVHcloud,
with its times under "The guest arms".

#### Option 1: no hosted arm path; the arms run on OVHcloud

Mechanism:
nothing is added.
A round starts an arm through `mvm`'s `ovhcloud` backend,
which creates one billed machine of the size asked,
runs the recipe and the arm inside it,
and deletes the machine when the round ends.

What it takes off this machine:
whole guests,
on every arm,
which is where they already are since 2026-11-28T08:00-05:00.

What it cannot do:
it cannot run the recipe's reboot and power-cut steps,
because `mvm` never stops or resumes an OVHcloud machine,
nor the step 22 measurement driver,
whose wait reads the host's own side of the folder a local guest shares;
both run nowhere under this option.
It cannot give an arm a machine for nothing:
every arm-run is billed.

Cost:
the machine is billed by the second,
and the one measured arm-run took about twenty minutes of a four-core 16 GiB machine
(create 75 s,
prepare 7 min 5 s,
the crate's build in the guest 9 min 40 s,
the arm 2 min 3 s,
destroy 35 s);
it depends on the owner's OVHcloud project and its keys.

Example:
with this option,
the driver `e2e:doctor-zfs-route` runs on an OVHcloud machine and reports its three guest tests:
two pass,
and the third reports the product answering `pass` where the arm expected `unknown`,
which is the record's own account of that run.

What a yes leaves undecided:
whether the two recipe steps and the step 22 driver are worth a path of their own,
and what an arm-run's price is at the sizes a larger recipe asks for.

#### Option 2: a hosted arm job per arm, after the session's trial

Mechanism:
after the trial under "The session's guest trial" prints its figures,
the session adds a workflow file on `main` with one job per arm:
a fresh `setfacl` grant on `/dev/kvm`,
then `qemu-system-x86_64 --accel kvm` with the guest's 8 GiB,
the OpenZFS recipe inside the guest,
then the arm's own steps,
and a loud failure when the device cannot be opened.
The OVHcloud path stays,
because the owner's rule put the guests there and this question does not undo that.

What it takes off this machine:
whole guests,
on every arm,
without a billed machine.

What it cannot do:
it cannot promise the device:
GitHub documents nested virtualization as unsupported,
and the ACL grant "disappears after successful QEMU execution",
so each job must grant it again before its QEMU invocation.
It fits one arm per job and nothing beside it,
because one arm's charge and the gate's containers come to 14.59 GiB against the trial's 14.57 to 14.80 GiB of `MemAvailable`.
It is unmeasured for an arm:
two Intel runs booted a CirrOS guest and no AMD run has,
and no arm's recipe has run there.

Cost:
one job of up to two hours and 8 GiB of guest RAM per arm,
free at the point of use,
a workflow file to keep in step,
and the guest's disk on the runner's image.

Example:
the trial's fourth run is what a first job prints:
`MemAvailable: 15279824 kB`,
`after the grant: opened, KVM_GET_API_VERSION answered 12`,
`qemu pid 2741, answered=true, seconds to login prompt 45`,
and `=== cirros: current=0.6.2 uptime=42.86 ===`;
what it has not printed is any arm's own lines.

What a yes leaves undecided:
whether the arms' own recipes complete there,
and what to do when GitHub changes the device again.

Ranking,
by what each delivers toward arms that no longer use this machine's memory on a path this repository can keep:

1 > 2.

- 1 over 2:
  option 1 delivers arms on a path with a measured end-to-end run,
  its times and its one known failing test,
  on a machine the owner already pays for and whose lifecycle `mvm` owns;
  option 2 delivers nothing measured yet,
  and the evidence it would rest on is four trial runs of a CirrOS boot,
  two of them on Intel,
  none on AMD,
  with GitHub documenting the device as unsupported.

### 4. Should any check become required?

Terms:
"required" means a required status check on `main` in branch protection,
which only the owner can set;
and a "red" is a failed run.

Why it is asked:
today nothing is required on `main`,
so nothing holds a push for a check.
The repository's own numbers show what a push's CI result is worth now:
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

#### Option 1: nothing becomes required

Mechanism:
nothing changes.
`main`'s protection keeps its empty required-status list
(`required_status_checks.contexts: []` and `checks: []`,
read this round with `gh api`),
and every push lands as it does today.

What it takes off this machine:
nothing.

What it cannot do:
it cannot hold any push for a check,
so a red never stops a landing;
that is why a red needs a reader from question 5.

Cost:
none.

Example:
today's own record:
the release's bump of 2026-11-28T13:15:10Z landed `main` with no check required of it,
and nothing was refused.

What a yes leaves undecided:
who reads a red,
which is question 5.

#### Option 2: a required check on `main`, once a check is green on `main`

Mechanism:
the owner sets one required status check in branch protection after the chosen check has run green for a while.
The check required is one a workflow here already produces on every `push` to `main` without path filtering:
`catalog-specifiers`'s `check` job is the nearest,
run by `.github/workflows/catalog-specifiers.yml` on `pull_request`,
`merge_group` and `push` to `main`,
green on 16 of the newest 20 pushes and `cancelled` on the other four
(`gh run list --workflow catalog-specifiers.yml --limit 20`,
read this round).

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
A check cancelled by its own `cancel-in-progress` satisfies nothing either,
which the four cancelled runs above show the chosen check doing.
And it is unusable while the chosen check is red:
the wrapper's lint is red with 850 errors,
and the three package unit tasks measured on 2026-11-28 are red.

Cost:
the owner's setting,
a change to how the release dispatches its bump,
and a fix round first.

Example:
the owner requires `catalog-specifiers`'s `check` after its job has run green for a week;
a non-admin actor's push whose check is missing,
cancelled or failed is then refused with a protected-branch error rather than made to wait,
and the owner's own push to `main` still lands.

What a yes leaves undecided:
how a release dispatches its bump without a check that never runs,
whether a cancelled run should count as a failure,
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
The page states that of pull requests;
that a push is blocked by the same mechanism is inferred here,
not exercised.

Cost:
the owner's setting and a check that runs almost never.

Example:
an outside contributor's pull request waits for a check that no merge here was ever going to wait for,
and its check reads `Pending` until someone bypasses it.

What a yes leaves undecided:
what the setting was for.

Options 2 and 3 can both be chosen,
in either order:
the same check can be required on `main` and for pull requests,
and neither setting cancels the other.

Ranking,
by whether a red on a required check reaches a person who lands work here,
counting a refusal of a push that cannot carry the check as worse than no hold at all:

1 > 2 > 3.

- 1 over 2:
  option 2's hold lands on the release's bump push,
  whose commit starts no run that could satisfy the check,
  and it does not hold the owner's own pushes,
  which are the pushes that land work;
  option 1 holds nothing and refuses nothing.
- 2 over 3:
  option 2 covers pushes,
  which is how work lands here;
  option 3 covers pull requests,
  and the newest merged one is #1776 of 2026-10-19,
  so it would hold almost nothing.

### 5. Who reads a hosted red?

Terms:
a "red" is a failed run;
the "Actions tab" is GitHub's own run list for this repository.

Why it is asked:
nothing on this project's loop reads a push's CI result today,
and the shipped workflows are mostly red or cancelled:
of the newest 300 runs of each per-push workflow,
`forbidden-strings.yml` is 92 failures and 85 cancelled of 300,
and `git-policy-cli-fuzz.yml` 127 and 135 of 300 (the counts above).
So a hosted red arrives where nobody looks,
unless something reads it.

One setting half-answers it and this proposal cannot read it:
GitHub notifies the account that triggered a run when that run completes,
"if you enable email or web notifications for GitHub Actions",
and "You can also choose to receive a notification only when a workflow run has failed" ([the notifications page][notifications page]).
Every push here is the owner's account's,
so that account's own setting may already send a failed run's red to the owner.

#### Option 1: nothing new reads it; the Actions tab is the record

Mechanism:
nothing changes.
A red stands in the Actions tab until someone opens it,
and the owner's notification setting stays as it is.

What it takes off this machine:
nothing.

What it cannot do:
it cannot put a red in front of anyone;
that is the state this round measured as 92 failures and 85 cancelled runs in one workflow's newest 300.

Cost:
none.

Example:
a `git-policy-cli-fuzz` run fails on a commit in a batch,
and the next batch check starts and ends without naming it.

What a yes leaves undecided:
who ever reads it.

#### Option 2: the session names each red at its batch check

Mechanism:
the coordinating session reads the hosted runs of the checks it dispatches at the batch check it already runs,
and names a red in its report to the owner,
with the commit and the failing step;
no branch protection changes and no account setting changes.

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

#### Option 3: the owner's own Actions notifications carry a red

Mechanism:
the owner turns on Actions notifications for the account in GitHub's notification settings,
choosing "only when a workflow run has failed",
so GitHub emails or notifies the account that triggered each failed run.
No workflow and no branch protection changes.

What it takes off this machine:
nothing.

What it cannot do:
it cannot make a red stand out once there are many:
under decision 1's option 4 every advisory run is red until two tools are installed,
which is 152 notifications a day;
it depends on an account setting this repository cannot see or set;
and it tells only the account that triggered the run,
which is always the same one here.

Cost:
the owner's account setting,
and the notifications themselves.

Example:
the owner's push at 13:15:10Z starts no run;
the next push starts an advisory run that fails at `//package/forbidden-strings/cli:test`,
and the account receives one failed-run notification naming that run.

What a yes leaves undecided:
whether 152 reds a day stay readable,
and what happens to a red on a run the owner did not trigger.

Options 2 and 3 can both be chosen:
the routine does not depend on the notification setting,
and the setting does not depend on the routine.

Ranking,
by how surely a red reaches a person who acts on it:

2 > 3 > 1.

- 2 over 3:
  option 2 names each red once,
  with the commit and the failing step,
  at a moment someone reads reports;
  option 3 sends one notification per failed run,
  which under decision 1 option 4 is 152 a day until the two installs exist,
  so a red stops standing out.
- 3 over 1:
  option 3 reaches the owner for a red raised by a push no batch covers,
  which option 1 leaves in the Actions tab.

What a yes to any option leaves undecided:
how long a red may stand,
and whether a round or a person acts on it.

## Two settings this file adopts unasked

Neither of these is a question for the owner:
a measurement or a record settles each,
so this file takes the answer and records it here with the owner's veto open.

### The advisory group keeps `cancel-in-progress: false`

The arithmetic settles it.
With the window gone,
keeping the setting starts 160 runs in 25.34 hours and finishes 160,
at about 20 job-hours a day;
cancelling starts 400,
finishes 49 and spends about 50 job-hours for 46 finished checks a day,
so it spends two and a half times the minutes for less than a third of the checks.
Decision 1's option 4 therefore goes with `cancel-in-progress: false`,
which is what the workflow carries today.

One visible result of keeping it:
a run that starts at 09:00:00Z ends at 09:07:56,
and the push that starts then is the newest one waiting,
say the push of 09:06;
the push of 09:05 was replaced while it waited,
so its commit is examined only by a later run whose range happens to include it.

GitHub documents one more setting this file examined and did not adopt:
`queue: max`,
where "Up to 100 jobs or workflow runs can be pending in the concurrency group",
and which cannot be combined with `cancel-in-progress: true` ([the concurrency page][concurrency page]).
It examines pushes in order rather than replacing them,
at the price of staleness:
379 pushes a day against the 160 runs in 25.34 hours the group can finish leaves about 229 pushes a day queueing,
so a queue of 100 would be full within hours and the run that starts would examine a commit that waited the whole queue
(this last step is inferred from the page's bound and this round's counts,
and nothing here models it).

What stays open,
for the owner's word or veto:
how stale an examined commit may be.
The push rate decides that,
and the setting itself is unchanged by any answer to decision 1.

### The wrapper's TypeScript checks stay where issue 6584 asks

Issue 6584 is where the owner answers where `//package/git-policy/cli`'s `test:unit`,
`lint:types` and `lint:oxlint` run.
The proposal it asks for exists and ranks seven placements,
A > C > B > E > D > F > G ([where the TypeScript checks run][placement record],
"Ranking"),
in which a hosted workflow of its own is E,
fourth.
What the issue's own last sentence settles until then is who runs them:
"Until then the coordinating session runs `test:unit`,
`lint:types` and `lint:oxlint` on `main` with each batch check and names what is red" (issue 6584).
So this file does not ask that question here,
and a yes to decision 4 does not answer it.

What the checks cost where they run now,
for whoever answers there:
37.8 seconds of wall time and peaks of 0.45,
0.99 and 4.5 GB at `1c8acd5bfb`,
the last of which the record puts at 1.8 GB at `64e3eb1a55`.
A per-package workflow would need one entry per manifest:
276 `mise.toml` files under `package/<category>/<name>/`,
counted this round with
`git ls-tree --name-only -r HEAD package/ | rg -c '^package/[^/]+/[^/]+/mise\.toml$'`,
of which 206 name a `test:unit` task (`git grep -l 'test:unit' -- 'package/*/*/mise.toml' | wc --lines`).
276 is a count of manifests,
not of packages whose unit task is red;
the three red package unit tasks this file measured are three packages.

## Combinations that cannot be built, and ones that can

Seven combinations,
each labelled for what it is:
cannot be built,
can be built,
or an unmeasured tight fit.
Three cannot be built,
three can be built,
and one is a tight fit no measurement here settles.

Cannot be built:

- **A required check on `main` satisfied by the release's current bump push.**
  The bump commit starts no workflow (read:
  the release record's "Consequences",
  and GitHub's sentence about `GITHUB_TOKEN`),
  so the check can never pass,
  and the push is refused rather than delayed (inferred,
  not exercised).
  It becomes buildable only by changing how the release dispatches its publish,
  or by adding the bot to a bypass list,
  which is not checked.
- **A required check produced by a path-filtered workflow.**
  A skipped workflow leaves its check `Pending` (read:
  the triggering page),
  and a required check that never resolves blocks the pull request forever.
  This binds question 4 option 2 or 3 paired with decision 1's option 2 or 3.
- **A required check produced by an advisory run the group replaced.**
  With decision 1's option 2,
  3 or 4,
  a push whose waiting run is replaced starts no run,
  so no check is produced for that commit;
  at 476 s that is 180 pushes of 400 under option 3 and 239 under option 4,
  and the required context would stay unsatisfied for each of them
  (inferred from the concurrency page and this round's arithmetic;
  not exercised).

Can be built:

- **A required check that a skipped `advisory core` job satisfies.**
  A required check passes on "a successful,
  skipped,
  or neutral status" (read:
  the protected-branches page),
  `advisory core` is `skipped` on 100 of the newest 100 pushes,
  so requiring it would pass every push without examining anything.
  It can be built,
  and it delivers nothing.
- **A hosted gate for every wrapper-touching push under the one-gate rule.**
  About 105 wrapper-touching pushes a day (measured:
  111 of 399 push ranges) at 20 to 42 minutes each is 35 to 73 job-hours a day,
  one to four of the twenty slots.
  It can be built,
  and the owner's rule does not want it:
  the rule asks for one gate per batch and says a merge needs no gate,
  and a gate on a push holds no merge.
- **A finished check on most pushes with `cancel-in-progress: true`.**
  The setting is one line and can be built;
  at 476 seconds of job and the sample's gaps it starts 400 runs,
  cancels 351 and finishes 49 (this round's arithmetic),
  so it delivers a verdict on 49 pushes of 400.

An unmeasured tight fit:

- **A guest arm beside the gate's containers in one 16 GB job.**
  One measured arm's guest charge,
  7.98 GiB,
  plus 6.6 GiB of container peak is 14.59 GiB against the trial's 14.57 to 14.80 GiB of `MemAvailable`,
  which leaves nothing at the fourth run's figure and 0.16 GiB at the third's,
  for QEMU's overhead,
  the shell steps and the page cache (measured:
  the trial's `MemAvailable`,
  the gate's peaks and the guest's charge;
  inferred:
  the sum is too close).
  No measurement here settles it either way.

Combinations that look sealed and are not:

- decision 1's option 4 with `cancel-in-progress: true`:
  reachable,
  and its arithmetic is 400 starts,
  49 finishes and 50 job-hours,
  which is why the setting is not adopted (see "Two settings this file adopts unasked").
- decision 1's option 1 with `cancel-in-progress: true`:
  also reachable,
  and there the cancel almost never fires,
  because 4 runs a day rarely overlap.
- question 4's option 2 with a hosted gate:
  a required check and a hosted batch gate are separate settings on separate files,
  and both can exist.

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

Three,
all same-vendor second passes,
as the status says.
Their findings and this revision's answer to each are in the report of the run that made this revision,
finding by finding,
with the command or the line each answer rests on.
What this revision changed is in that report,
which names the file's lines and the commands;
an earlier revision's changelog does not belong in a proposal the owner reads to decide.

The arithmetic behind every admission figure is in this folder,
and reruns with `node` and no other input:

- `github-hosted-runners-for-package-checks-and-a-guest-evidence/advisory-core-newest-400-push-arrivals-2026-11-28.json`,
  the 400 runs read from GitHub at 2026-11-28T07:51-05:00;
- `github-hosted-runners-for-package-checks-and-a-guest-evidence/admission-arithmetic-over-those-arrivals.ts`,
  the rules and the group,
  with its model in its header;
- `github-hosted-runners-for-package-checks-and-a-guest-evidence/admission-arithmetic-output-at-401-476-and-557-seconds.json`,
  what it printed,
  which is what this file quotes.

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
[base distro record]: ../decision/container-base-distro.md
