# GitHub's hosted runners for the package checks, and a guest trial

Status:
a proposal for the repository owner.
Unapproved and non-final:
nothing here is adopted,
no required check changes,
and a reviewer from another vendor has not read it.

Written by a Pi run on 2026-11-28,
in the worktree
`.claude/worktrees/pi-github-hosted-runners-carry-package-checks-and-a-guest-trial-proposed`
on the branch
`docs/planning-github-hosted-runners-carry-package-checks-and-a-guest-trial-proposed`,
from `main` at `4cf7cc74b8`.
What was measured is marked as measured,
with the run or the record beside it;
what rests on another record says which;
what is not measured says so.

## What the owner asked for

The owner was offered a paid provider's trial and wrote
"Don't we already have gh?",
and the record answers it:
"GitHub's hosted runners,
which the repository already has,
come first"
([the owner's answers](decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md),
"Whether the machine and the repository are set up right").
The question this proposal answers is which of the repository's checks belong there,
so that work which today competes for one machine's memory has somewhere else to run.

The machine's pressure is recorded:
on the night of 2026-11-27 it ran out of memory with nineteen rounds and ten guests alive,
and the owner bounded a later batch by the guests alive at once
([the same record](decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md),
"The rounds that were running,
and what bounds a later batch";
[troubleshooting](troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md)).

## Why the budget exists, and whether its reason still holds

### The lines that set it

`.github/workflows/advisory-core.yml` holds a `gate` job whose step
`Count recent admitted runs` reads the workflow's own history and answers one question:
may this push run the `core` job.
Its summary line states the windows:

```text
echo "- budgets: 2 per minute, 4 per hour, 8 per day"
```

and its refusal is one comparison over three trailing windows:

```text
if [ "$minute" -ge 2 ] || [ "$hour" -ge 4 ] || [ "$day" -ge 8 ]; then
  echo "allowed=false" >> "$GITHUB_OUTPUT"
```

The `core` job runs only when `needs.gate.outputs.allowed == 'true'`,
so a refused push runs the gate and nothing else.

### The record that says what the budget was for

The advisory profile was designed inside the Depot evaluation,
whose record names the budget as a starting point rather than a measurement:

> The default-branch advisory run uses the sliding-window rate limits currently used by the
> Scorecard workflow as the starting point,
> and adjusts from measured capacity.
> It runs a fast static,
> build and unit subset;
> the complete profile runs in the weekly matrix.
>
> [the Depot adoption decision](decision/depot-ci-adoption.md),
> "Execution routing"

Its author's own commit says the same,
and adds the one thing the budget was there to ration:
"Keep `cancel-in-progress` false so the rate gate,
not concurrency,
decides admission" (`e4a40ea3c1`).
That was the recorded purpose:
on a pool of 20 concurrent jobs,
a burst of auto-pushed commits should not queue twenty-minute runs,
so an admission rule decides which pushes run.

Scorecard's own gate is the model,
and it counts something narrower than events:

> A rate-limit job counts earlier runs whose analysis job executed and skips the analysis job once
> a budget is spent.
>
> `cf3e5d40cb`,
> the commit that added Scorecard's budgets

> It counts an analysis when its conclusion is neither `skipped` nor `cancelled`.
>
> [the rolling-budget evaluation](planning/depot-rolling-budget-evaluation.md),
> "Existing implementation and measured overhead"

The copy in `advisory-core` lost that filter.
Its counting expression takes every run of the workflow except the current one and ages it:

```text
[ .[] | select(.databaseId != $current) | age ] as $ages |
```

Nothing there asks whether the `core` job ran,
so a skipped run counts against the windows that skipped it.

### Measured on 2026-11-28

`gh run list --workflow advisory-core.yml --limit 200 --json conclusion,createdAt,updatedAt,event`
answered with 200 runs between `2026-11-27T17:13:22Z` and `2026-11-28T07:46:57Z`
(read at 2026-11-28T02:50-05:00):

- every one of the 200 concluded `success`,
  and every one was a `push` event;
- each run lasted between 6 and 16 seconds,
  mean 8.7 seconds,
  1745 seconds in total:
  a run that admitted the `core` job cannot last that long,
  because the job alone took 21 minutes 28 seconds when it last ran
  ([the measurement record](planning/depot-ci-measurement-targets.md)).
  So all 200 ran the gate and skipped the checks;
- at the newest of the 200,
  the trailing day held 199 runs and the trailing hour held 8,
  so both the day window and the hour window refused that push.

The workflow's runs API for the newest 100 runs
(`created >= 2026-11-26T00:00:00Z`) shows 37 runs on 2026-11-27 and 63 on 2026-11-28,
with 0 admitted and a longest run of 13 seconds.
The older measurement stands beside it:
"Of the 400 most recent runs of the workflow (2026-11-04 to 2026-11-07),
394 skipped the `advisory core` job"
([keeping the repository linter at zero](planning/repository-linter-stays-at-zero.md),
"Why nothing caught it").

### Whether the reason still holds

The recorded reason has two parts,
and neither survives contact with the measurements.

The first part is the resource it rations.
GitHub's own billing page says standard hosted runners cost nothing here:
"The use of standard GitHub-hosted runners is free:
In public repositories"
([GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions)).
The only capacity left to ration is concurrency,
and the repository's own provider record measured about 3 of 20 slots in use at its peak
([the CI provider decision](decision/ci-provider.md),
"Limits that actually apply").
Nothing the budget protects is short.

The second part is the rule it implements.
Scorecard counts executions;
`advisory-core` counts events,
and at 199 push-driven runs a day the day window can never empty.
The gate is therefore not a slow lane:
it is a closed door,
and the profile admitted nothing in the newest 100 runs,
which span two days.
The two facts a reader can check in a minute:
0 admitted in the newest 100 runs,
and 0 of the 200 sampled runs lasting long enough to have run the checks.

What the budget was for still exists as a question,
and it is a question about concurrency,
not about the number:
a push storm should not queue dozens of twenty-minute runs behind each other.
A budget that counts executions answers that question.
This one does not.

## What a hosted runner is today

Every fact below is from GitHub's own documentation,
read on 2026-11-28;
none of it is recalled.

### The runners a public repository gets, at no charge

- "Use of the standard GitHub-hosted runners is free and unlimited on public repositories."
  ([GitHub-hosted runners reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners),
  "Standard GitHub-hosted runners for public repositories")
- "The use of standard GitHub-hosted runners is free:
  In public repositories"
  ([GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions))
- "For public repositories,
  jobs using the workflow labels shown in the table below will run with the associated
  specifications."
  ([the same reference page](https://docs.github.com/en/actions/reference/runners/github-hosted-runners))

The processors,
memory and disk the labels name,
for public repositories:

- `ubuntu-latest`,
  `ubuntu-24.04`,
  `ubuntu-22.04`,
  `ubuntu-26.04`:
  "Linux",
  4 processors,
  16 GB memory,
  14 GB SSD,
  x64 ([the same reference page](https://docs.github.com/en/actions/reference/runners/github-hosted-runners));
- `ubuntu-slim`:
  1 processor,
  5 GB,
  14 GB,
  x64,
  and "The job timeout for single-CPU runners is 15 minutes.
  If a job reaches this limit,
  the job is terminated and fails."
  ([the same page](https://docs.github.com/en/actions/reference/runners/github-hosted-runners),
  "Single-CPU runners");
  it "runs in unprivileged mode",
  so it cannot mount file systems.

The measured machine was larger than the table in one respect:
its root file system reported 145 GB with 87 GB free,
against the table's 14 GB SSD.
That is what the run printed;
whether it is the pool or the image is not something the run could separate.

### Limits on concurrency and on length

- Concurrency:
  "Standard GitHub-hosted runner | Free | 20 | 5 | Not applicable",
  which the page heads as total concurrent jobs and maximum concurrent macOS jobs,
  and adds "GitHub Support can increase job concurrency limits for GitHub Actions."
  ([Actions limits](https://docs.github.com/en/actions/reference/limits),
  "Job concurrency limits for GitHub-hosted runners")
- A job's length:
  "Each job in a workflow can run for up to 6 hours of execution time.
  If a job reaches this limit,
  the job is terminated and fails."
  ([the same page](https://docs.github.com/en/actions/reference/limits),
  "Existing system limits")
- A workflow run's length:
  "Workflow run time | 35 days / workflow run",
  and "If a workflow run reaches this limit,
  the workflow run is cancelled."
  ([the same page](https://docs.github.com/en/actions/reference/limits))
- The queueing caps:
  "1500 events / 10 seconds / repository" and "500 workflow runs / 10 seconds"
  ([the same page](https://docs.github.com/en/actions/reference/limits),
  "Workflows queuing")
- A job that waits too long is dropped:
  "if the workflow run has been successfully queued,
  but has not been processed by a GitHub-hosted runner within 45 minutes,
  then the queued workflow run is discarded."
  ([GitHub-hosted runners](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners),
  "Workflow continuity")

### Larger runners: cost, and who may use one

- "Larger runners are only available for organizations and enterprises using the GitHub Team or
  GitHub Enterprise Cloud plans."
  ([Larger runners](https://docs.github.com/en/actions/concepts/runners/larger-runners),
  "Who can use this feature?")
- "Larger runners are not free for public repositories."
  ([Actions runner pricing](https://docs.github.com/en/billing/reference/actions-minute-multipliers))
- The rate for the shape nearest a standard runner:
  "Linux 4-core | linux_4_core | $0.012"
  ([the same page](https://docs.github.com/en/billing/reference/actions-minute-multipliers),
  "x64-powered larger runners")
- The disk a 4-core larger runner adds:
  4 processors,
  16 GB memory,
  150 GB SSD for Ubuntu
  ([Larger runners reference](https://docs.github.com/en/actions/reference/runners/larger-runners),
  "Specifications for general larger runners")
- "You must have a valid credit card on file in order to use larger runners."
  ([the same page](https://docs.github.com/en/actions/reference/runners/larger-runners),
  "Troubleshooting larger runners")

So a larger runner buys disk and a static shape,
not more processors for less money,
and it needs a payment method and a plan this repository's owner would have to choose.

### Whether a hosted Linux runner offers `/dev/kvm`

GitHub's documentation states no promise of the device.
What it states is the boundary:

- "While nested virtualization is technically possible while using runners,
  it is not officially supported.
  Any use of nested VMs is experimental and done at your own risk,
  we offer no guarantees regarding stability,
  performance,
  or compatibility."
  ([GitHub-hosted runners](https://docs.github.com/en/actions/concepts/runners/github-hosted-runners),
  "Preinstalled software for GitHub-owned images")

The device is there anyway,
and the trial below measured it:
`/dev/kvm` exists as `crw-rw---- 1 root kvm 10, 232`,
the runner user cannot open it until a grant is written,
and after the grant a QEMU guest boots under KVM and answers in 44 seconds.
The earlier record on `ubuntu-24.04` found the same boundary
([the platform comparison](troubleshooting/depot-ci-versus-github-runner-platforms.md),
"Standard GitHub KVM access disappears after successful QEMU controls").

## What the repository's checks need

Everything here is from the repository's own records,
or measured by this run where the record says so.

### The fast subset the advisory profile runs

On GitHub-hosted Linux,
run `39202293518` took 21 minutes 28 seconds from the `core` job's start to its end:
7 minutes 10 seconds to install the repository tools,
7 minutes 22 seconds to build the five-owner subset,
6 minutes 15 seconds to test it
([the measurement record](planning/depot-ci-measurement-targets.md),
"Observed samples").
Five packages' build and test tasks therefore fit one standard runner as it is,
with room to spare,
and the tool installation is the part a cache or a prepared image would cut.

### One package's unit task, measured on this machine today

`mise run //package/logger/core:test:unit` in its own scope at 2026-11-28T03:00-05:00
exited 1 in 6.4 seconds with a scope peak of 1,643,687,936 bytes;
`mise run //package/toml-edit/core:test:unit` exited 1 in 7.0 seconds with a peak of
1,648,033,792 bytes.
Both exited 1 on the tree this proposal was written on,
in one run each,
and this run neither diagnosed nor repeated them:
`logger/core` failed five test files including `local-storage.unit.test.ts`,
and `toml-edit/core` failed `value-encoders.unit.test.ts` on the sign of a negative zero.
The peak is the harness's parallel Node processes,
not one process,
and the same shape of run recorded 4.5 GB before issue 6580's fix
([where the TypeScript checks run](planning/git-policy-cli-where-the-typescript-unit-tests-and-lint-run.md),
"What was measured").

The wrapper's three TypeScript checks were measured in the same shape:

- `lint:types` exits 0 in 2.0 to 2.8 seconds with a peak of 0.41 to 0.45 GB;
- `lint:oxlint` exits 1 in 8.4 to 12.8 seconds with a peak of 0.94 to 0.99 GB;
- `test:unit` exits 0 in 19.4 to 22.2 seconds with a peak of 1.8 GB,
  after the fix that took it down from 4.5 GB.

([the same record](planning/git-policy-cli-where-the-typescript-unit-tests-and-lint-run.md),
"What was measured",
and its "Where main moved while this was written")

### The wrapper's container gate

`mise run //package/git-policy/cli:native:test:container` is the check a release waits for.
Its substantive step is the whole suite and Clippy in containers built from one image.

- the suite printed `Summary [1053.402s] 9173 tests run: 8988 passed (8 slow), 185 failed, 28 skipped`
  on this host,
  where 185 of the failures are the host's ZFS mount behaviours rather than the product
  ([the container gate record](troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md),
  "Symptom");
- the suite container peaked at 1123.8 MiB of its 2048 MiB bound,
  with 994 MiB of process pages and 222 MiB of fixture bytes,
  and the suite's own comment records 401 processes at its peak
  ([the same record](troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md),
  "What the run measured");
- a gate takes 20 minutes 36 seconds to 21 minutes 43 seconds from unit start to stop,
  of which the suite is 889 to 1259 seconds
  ([the same record](troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md),
  as cited in the TypeScript placement record);
- the gate's own unit holds 10 CPUs and 20 GiB,
  and its containers get 4 CPUs,
  4 GB for the suite,
  8 GB for the image build's compile and 8 GB for Clippy's
  ([the TypeScript placement record](planning/git-policy-cli-where-the-typescript-unit-tests-and-lint-run.md),
  "A:
  inside the release gate");
- the image build compiles the tests under its own cap,
  and the cap was measured binding at 2047 MiB of 2048 MiB with the host well under pressure
  ([issue 6455](https://github.com/Aquaticats/Monochromatic/issues/6455),
  "Measured").

### The whole-tree static checks

- `monochromatic-lint` over the documentation tree took 3.84 seconds
  ([keeping the repository linter at zero](planning/repository-linter-stays-at-zero.md),
  "The full lint is a manual task");
- the Rust closure check inside the full lint runs Dylint once per Cargo package,
  and a run restricted to one Rust file took 89.65 seconds because it checked the file's package
  ([the same record](planning/repository-linter-stays-at-zero.md),
  "The full lint is a manual task");
- the advisory profile's `Static changed paths` step cannot start at all:
  `monochromatic-lint` is not installed by `repo-tools-setup`,
  and its log says `task-run: line 1: monochromatic-lint: command not found`
  ([the same record](planning/repository-linter-stays-at-zero.md),
  "The CI step cannot run").

### The complete build profile

`mise run build` needs system libraries and services a generic runner does not carry:
glib,
gdk-pixbuf,
cairo,
pango,
GTK,
Qt6,
Podman,
the Android build,
the desktop application and the Wayland session build,
and it failed with `SIGABRT` and thread-creation failures under fanout
([the baseline register](planning/depot-expanded-ci-baseline-register.md),
"Build profile" and "CI baseline runs").
The record's own disposition is that desktop,
Android,
Qt and Podman stay in the weekly prepared
environment
([the rollout plan](planning/depot-expanded-ci-rollout.md),
"Execution routing").

### The wrapper's guest arms

Each ZFS arm runs only inside a throwaway guest of `mvm`'s `libvirt` backend,
"Ubuntu 24.04,
OpenZFS 2.4.4",
against one file-backed pool,
and several arms need root inside it
([`package/git-policy/cli/mise.toml`](../../package/git-policy/cli/mise.toml),
the `e2e:` tasks;
[the slot store design](planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md),
"Guest arms,
and why each needs root or a real pool").

### Which of these a hosted runner carries

As it is,
with the 4 processors and 16 GB the table names:

- the five-owner fast subset (measured,
  21 minutes 28 seconds);
- a package's TypeScript checks (measured here,
  40 seconds and under 4.5 GB);
- the Markdown rules over the whole documentation tree (measured,
  3.84 seconds);
- the container gate,
  if its caps are given a runner's budget:
  its suite plus Clippy plus the image build fit 16 GB only when they are not all at their caps at
  once,
  and its wall time on a 4-CPU runner is the 20 minutes the gate already takes on 4 CPUs here;
- one QEMU guest,
  booted in 44 seconds (measured).

Only split up:

- the container gate's three phases,
  if each is given the cap it measured for itself,
  because 4 GB plus 8 GB plus 8 GB does not fit 16 GB beside the runner's own work;
- a package's build and its unit tests,
  where the build needs a toolchain the runner must install;
- the ZFS guest arms,
  one guest per job,
  since each needs its own kernel with OpenZFS and its own pool.

Not at all,
on the record:

- the complete build profile,
  which needs Qt6,
  GTK,
  Podman,
  the Android SDK and a Wayland session;
- the wrapper's btrfs host controls,
  which name this host's file systems by design;
- anything that opens `/dev/zfs` on a host,
  which is this machine's rule and not a runner capability;
- mutation campaigns,
  which are paused.

## The trial run, and what it printed

The trial was a workflow file on this branch,
triggered by a push to this branch alone,
installing nothing that needs a secret,
using no larger runner and registering no self-hosted runner.
It measured three times,
because each run taught the next what to ask:

1. run `46123229560` printed the machine and found `/dev/kvm` present but refused to the runner user;
2. run `46123307937` wrote the grant and printed
   `after the grant: opened, KVM_GET_API_VERSION answered 12`,
   then ended QEMU at once with `qemu-system-x86_64: --memory: invalid option`;
3. run `46123397702` corrected that and the image format,
   and booted the guest.

The last run is
[runs/46123397702](https://github.com/Aquaticats/Monochromatic/actions/runs/46123397702),
started `2026-11-28T07:55:20Z`,
job `specimen` from `2026-11-28T07:55:22Z` to `2026-11-28T07:56:21Z`,
59 seconds in all.

What it printed:

- the image:
  `Image: ubuntu-26.04`,
  version `20261122.9.1`,
  a `Hosted Compute Agent` on `Region: eastus2`,
  `Cloud: Azure`;
- processors:
  `nproc` answered `4`;
  `CPU(s): 4`,
  two cores with two threads each,
  `Model name: AMD EPYC 9V74 80-Core Processor`,
  `Hypervisor vendor: Microsoft`,
  `Virtualization: AMD-V`,
  and the `svm` flag present in `/proc/cpuinfo`;
  `systemd-detect-virt` answered `microsoft`;
- memory:
  `Mem: 15983` MiB total,
  14682 free at start,
  `MemTotal: 16366636 kB`,
  `MemAvailable: 15523076 kB`,
  and no pressure visible at the start;
- disk:
  `/dev/root ext4 145G 58G used 87G available /`,
  and `tmpfs 7.9G` on `/dev/shm`;
- container runtimes:
  `Docker version 29.4.2` and `podman version 5.7.0`,
  so the wrapper gate's own runtime is already installed;
- `/dev/kvm`:
  `crw-rw---- 1 root kvm 10, 232 Nov 28 07:52`,
  ACL `user::rw-`,
  `group::rw-`,
  `other::---`;
  the probe's control `opened read-write` `/dev/null`,
  while `/dev/kvm: open read-write failed: [Errno 13] Permission denied: '/dev/kvm'`;
- after `sudo setfacl --modify u:1001:rw /dev/kvm`:
  `after the grant: opened, KVM_GET_API_VERSION answered 12`;
- the guest:
  `QEMU emulator version 10.2.1`,
  the CirrOS 0.6.2 `qcow2` image,
  `--machine q35 --accel kvm --cpu host --smp 2 -m 512`,
  and the answer
  `qemu pid 2600, answered=true, seconds to login prompt 44`,
  with the guest's own line `=== cirros: current=0.6.2 uptime=42.98 ===`.

Two facts to carry forward from that:

- a standard public runner here does give working KVM,
  after one `setfacl` per boot,
  and a small cloud image answers in under a minute;
- the device and the grant are not documented as supported,
  so a check built on them is a check that GitHub may change without notice.

The trial's workflow file was removed after the runs,
so that no workflow that can never run on `main` lands there;
its steps are reproduced by the list above and by the logs,
which stay in the run pages.

## Options

Each option says what it would run,
what it takes off this machine,
what it cannot do,
and what it costs in minutes and upkeep.
Minutes are the hosted runner's and they cost nothing here;
the unit is stated anyway because it is the queue the budget was invented for.

### Option A: fix `advisory-core`, its window and its linter step, and nothing else

What it would run:
the `core` job as written,
which is the changed-path `monochromatic-lint` step,
the five-owner build subset and the five-owner unit subset,
on every push that the window admits,
plus `monochromatic-lint` and its Dylint driver installed the way `repo-tools-setup` installs the
other tools.

What it takes off this machine:
the five packages' build and test tasks,
7 minutes 22 seconds plus 6 minutes 15 seconds of this machine's own compute per admitted run,
and the changed-file lint,
which today runs nowhere.

What it cannot do:
it stays advisory and changed-path scoped,
so a file that becomes dirty without being touched is never seen;
it covers five of the 225 package directories the baseline register counts
  ([the baseline register](planning/depot-expanded-ci-baseline-register.md),
  "Pinned revision and owner map");
it says nothing about the wrapper's gate,
its guests,
or the desktop and Android builds.

Cost:
8 admitted runs a day is 172 minutes of hosted time,
which is free,
and about 330 gate-only runs a day is another 48 minutes;
the upkeep is the window's counting rule,
one install step,
and the Dylint driver that the closure check needs.
Two of the five packages' unit tasks exited 1 on the tree this was written from,
so a repaired workflow would report red until they are fixed.

Example:
a push touching `package/toml-edit/core/src/value-encoders.unit.test.ts` runs the gate,
is admitted if the trailing windows allow,
and its `core` job names `//package/toml-edit/core:test:unit` in the run's summary.

### Option B: one check workflow per package, started by the paths a push changes

What it would run:
one reusable workflow,
called by jobs filtered on the package's paths,
running that package's `build`,
`test:unit` and the checks its `mise.toml` names,
the way `.github/workflows/toml-edit-fuzz.yml` and `.github/workflows/logger-fuzz.yml` already
filter on paths.

What it takes off this machine:
every package's TypeScript and Rust unit tasks,
which today exist only as something a person or an agent starts by hand:
measured here at 6 to 22 seconds and 1.5 to 4.5 GB each.

What it cannot do:
it cannot replace the whole-tree lint or the closure check,
which need every package's dependencies and a Dylint driver per package;
it cannot run a check whose package needs Qt6,
Android or a Wayland session.

Cost:
about three minutes of hosted time per touched package,
with 20 concurrent jobs available;
the upkeep is a path filter and a task list per package,
or one reusable workflow and a table of packages,
and a discipline that new packages get an entry
(the repository's own scaling plan asks for path filters for exactly this reason).

Example:
a push touching `package/forbidden-strings/cli/src/index.ts` starts that package's job only,
which prints `Found 1856 warnings and 850 errors` and fails in three minutes.

### Option C: the wrapper's container gate on a hosted runner

What it would run:
`mise run //package/git-policy/cli:native:test:container` in a workflow on `ubuntu-latest`,
with `--cpus 4` and a `--memory` the runner can hold,
which is what the gate already asks of a smaller host.

What it takes off this machine:
the largest block any single command here holds:
20 minutes 36 seconds to 21 minutes 43 seconds per batch,
a unit quota of ten CPUs and a memory bound of 20 GiB on this host,
four CPUs and up to 8 GB inside its containers,
401 processes at the suite's peak,
and the 8 GB compile that issue 6455 measured pressing its cap.

What it cannot do:
the guest arms,
the btrfs host controls and the `test:service-manager` arms,
which need this host's file systems or a manager of their own;
and it replaces no release decision unless the owner makes its check required,
because a red hosted run arrives after the commit is public.

Cost:
about 25 to 35 minutes of hosted time per admitted gate,
free;
the upkeep is the image build's prerequisites on a fresh runner,
the action that installs the locked Rust toolchain,
and a decision about which events start it,
because at this repository's push rate a gate per push is 330 runs a day.

Example:
a batch tip pushes to a branch,
the workflow builds the gate image,
runs the suite and Clippy with `--cpus 4`,
and prints `Summary [1053.402s] 9173 tests run` from the runner instead of from this machine.

### Option D: the wrapper's guest arms on a hosted runner under KVM

What it would run:
one `e2e:` arm per job,
booting a QEMU guest with `--accel kvm` as the trial did,
whose recipe is the ZFS binding's own (Ubuntu 24.04,
OpenZFS 2.4.4),
then the arm's own `create`,
`prepare`,
`run` and `destroy` steps inside it.

What it takes off this machine:
whole guests,
which are the resource the owner bounded at nine alive at once,
and the memory a machine under the desktop's pressure cannot spare.

What it cannot do:
it cannot use `mvm`'s `libvirt` backend as written,
because the drivers name that backend and this machine's rules about a session bus were written for
this machine;
it cannot run an arm that names this host's own pool;
and it rests on a device GitHub documents as unsupported,
so the workflow needs a control that fails loudly when KVM is missing rather than silently running
a guest at a tenth of the speed.

Cost:
44 seconds for the boot measured here,
plus whatever the OpenZFS recipe costs inside the guest,
which is not measured;
nine arms is nine jobs of at least a few minutes each;
the upkeep is a second way of booting the guest,
kept in step with `mvm`'s own,
and a proof that OpenZFS 2.4.4 builds and loads in a nested guest.

Example:
a push touching the wrapper's removal code starts the removal arm,
which boots a guest in under a minute,
builds OpenZFS in it,
creates a file-backed pool,
and prints the same arbitration lines the arm prints on this machine.

### Option E: leave things as they are

What it would run:
the gate on every push and nothing else,
which is what runs today.

What it takes off this machine:
nothing.

What it cannot do:
anything at all on a hosted runner,
including the checks the advisory profile was built for.

Cost:
about 48 minutes of hosted time a day,
all of it gate,
free,
with no upkeep and no result.

Example:
the newest run in the sample,
created `2026-11-28T07:46:57Z`,
ended 8 seconds later,
skipped `advisory core`,
and reported `success`.

### Ranking

Ranking:
C > D > B > A > E,
because:

- C over D:
  C moves the one measured block that holds this machine's whole CPU budget and 20 GiB of its
  memory for twenty minutes at a time,
  and every piece it needs on a runner was measured present (podman 5.7.0,
  4 processors,
  16 GiB);
  D moves whole guests,
  which is valuable,
  but its load-bearing step (OpenZFS 2.4.4 inside a nested guest) is the one step no record shows
  working,
  while C's twenty minutes and 401 processes are recorded on this host already.
- D over B:
  D takes virtual machines off this machine,
  which is the resource the owner bounded and the one an out-of-memory night names;
  B takes 6 to 22 second checks whose peak is under 4.5 GB.
- B over A:
  B reports a red at the push for every package a push touches,
  and the record of two wrapper checks unseen for thirteen days is what it answers;
  A covers five of the 225 package directories and its static step cannot start.
- A over E:
  A repairs a workflow that exists and currently admits nothing,
  so the changed-file lint and the fast subset run again;
  E leaves both broken and spends the same hosted minutes on gates that decide nothing.

## Questions for the owner

Five decisions,
each one answerable without the others.

### 1. Should `advisory-core`'s push window count only the runs whose `advisory core` job executed?

Options,
recommended first:

1. Count executions,
   as Scorecard's gate does,
   and keep 2 per minute,
   4 per hour,
   8 per day.
   Costs:
   at most 172 minutes of hosted time a day for the admitted runs,
   plus about 48 minutes of gates,
   free;
   the change is one filter in the counting step,
   and the profile is advisory as before.
   What it gives:
   the profile runs again,
   on at most eight pushes a day.
   What it does not:
   no push storms are absorbed,
   because the day window still closes.
2. Count executions and widen the day window,
   for example 8 per hour and no day cap.
   Costs:
   up to 172 minutes of hosted time an hour if pushes keep coming,
   still bounded by the 20 concurrent jobs,
   and a queue that can hold other workflows behind it.
   What it gives:
   every push within an hour of a quiet period gets a result.
   What it does not:
   it reintroduces the queueing the budget was written to prevent whenever pushes
   arrive faster than runs finish.
3. Remove the window and let `cancel-in-progress: true` decide.
   Costs:
   a run per push,
   about 330 a day at 21 minutes each,
   which is about 118 hours of hosted time a day against 20 concurrent jobs,
   so most pushes wait in a queue behind their own successors.
   What it gives:
   the newest commit is always the one being checked.
   What it does not:
   it makes the queue,
   not the newest push,
   the thing a red competes with.

### 2. Should the wrapper's container gate get a hosted workflow?

Options,
recommended first:

1. Yes,
   advisory,
   started by a push that changes `package/git-policy/cli` and its dependencies.
   Costs:
   about 25 to 35 minutes of hosted time per run,
   free,
   with one action that installs the locked Rust toolchain and a job that passes `--cpus 4`;
   a red run arrives after the commit is public,
   as everything advisory does.
   What it gives:
   the twenty minutes and 401 processes leave this machine for every batch that pushes.
   What it does not:
   it does not run the guests,
   the service-manager arms or the btrfs controls.
2. Yes,
   but only on a label or a dispatch,
   as the gate's own release path does.
   Costs:
   the same minutes,
   only when asked,
   plus the discipline of asking.
   What it gives:
   a second opinion on a release without spending a run per push.
   What it does not:
   nothing catches a red between releases.
3. No,
   keep the gate on this machine.
   Costs:
   the twenty minutes of this machine's CPU and up to 20 GiB,
   per batch,
   as today.
   What it gives:
   the gate keeps the ZFS mount behaviours this host has,
   which is what its 185
   recorded failures are about.
   What it does not:
   it keeps the check that most competes for this machine's memory.

### 3. Should the packages' TypeScript checks get one reusable, path-filtered workflow?

Options,
recommended first:

1. Yes,
   for every package that has a `test:unit` or a `lint:types` task.
   Costs:
   about three minutes per touched package,
   free,
   and one list of packages to keep current,
   which the repository's own scaling plan already assumes.
   What it gives:
   a red arrives within minutes of the push,
   instead of being found by a batch or not at all.
   What it does not:
   it does not run the Rust closure rules or the whole-tree lint.
2. Only for the packages whose checks have gone unseen and red,
   such as `git-policy/cli`.
   Costs:
   one package's three minutes,
   and a list that grows as more are found.
   What it gives:
   the two failures of 2026-11-27 are covered first.
   What it does not:
   the next unseen red is in another package.
3. No,
   keep them local and inside the gate.
   Costs:
   the same seconds on this machine,
   and the same silence when nothing starts them.
   What it gives:
   nothing new.
   What it does not:
   it leaves the checks that ran red for days with no starter.

### 4. Should the wrapper's ZFS guest arms move to a hosted runner under KVM?

Options,
recommended first:

1. First prove the recipe in a nested guest on a runner,
   then decide.
   Costs:
   one trial workflow,
   free,
   and the time to build OpenZFS 2.4.4 in a guest;
   it answers a question no record answers today.
   What it gives:
   a measured answer instead of a guess,
   and a control that names the missing device when the pool cannot exist.
   What it does not:
   it moves no arm yet.
2. Move them now,
   one job per arm.
   Costs:
   at least a few minutes per arm,
   free,
   plus a second way of booting the guest beside `mvm`'s,
   kept in step with it,
   and a workflow that must fail loudly when KVM is absent.
   What it gives:
   the guests leave this machine,
   which is the resource the owner bounded.
   What it does not:
   it cannot use `mvm`'s `libvirt` backend as written,
   and it rests on an unsupported device.
3. Keep them here,
   and on the OVHcloud backend when it exists.
   Costs:
   this machine's memory for each guest,
   and the OVHcloud work is another round's
   ([that decision](decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md)).
   What it gives:
   the guests run where the pool and the rules already work.
   What it does not:
   it removes nothing from this machine until the backend lands.

### 5. Should any hosted check become required on a pull request?

Options,
recommended first:

1. No check becomes required.
   Costs:
   nothing;
   a red still arrives after a push,
   which is the recorded advisory model
   ([the Depot adoption decision](decision/depot-ci-adoption.md),
   "Advisory-only model").
   What it gives:
   no commit is blocked by a check whose baseline is not known-good.
   What it does not:
   it does not stop a red from landing.
2. The repository linter required on pull requests only.
   Costs:
   the owner sets the branch protection,
   which no file can do,
   and every pull request waits for a whole-tree lint the repository has never had green.
   What it gives:
   the one rule the owner called the biggest priority gets a gate.
   What it does not:
   it covers pull requests only,
   and auto-push to `main` bypasses it.
3. The container gate required before a release.
   Costs:
   the owner's branch-protection setting,
   and a release waits for a hosted run of 25 to 35 minutes.
   What it gives:
   the check the release rests on becomes a real gate.
   What it does not:
   it does not cover the guests or the host controls.

## What this proposal did not measure

- Whether the 8 admitted runs a day were ever spent.
  The newest 100 runs contain none,
  and the runs before them are outside the sample.
- What the container gate would take on a runner.
  Its caps and its wall time are measured on this host;
  the image build on a fresh runner,
  with the toolchain installed by an action,
  is unmeasured.
- Whether OpenZFS 2.4.4 builds and loads inside a KVM guest on a runner.
  The boot is measured;
  the recipe is not.
- What the desktop,
  Android,
  Qt,
  Podman and Wayland builds would need on a runner.
  The records say they need a prepared environment,
  and nothing here prepared one.
- The repository's plan.
  Larger runners need a payment method and a Team or Enterprise plan,
  and nothing in this proposal requires either.
- The push rate that closed the window.
  The sample shows 200 runs in 14 hours 33 minutes;
  how many of those were one session's auto-pushes is not separated.
