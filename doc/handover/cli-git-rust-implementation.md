# cli-git implementation handover

This page holds only the open work of the Git wrapper effort (`package/git-policy/cli`) and its current
state.
Nothing on it is approved or final:
it is an agent's account,
and only the owner declares work complete.
It was rewritten on 2026-11-25T23:30-05:00 to the state the coordinating session read at
2026-11-25T23:25-05:00,
when the owner asked for the page to hold tonight's state and nothing that has landed.

What landed left this page for the record that owns it:
a root cause or a workaround went to `doc/troubleshooting/`,
a decision to `doc/decision/`,
a measured number to the page that measures it,
and unfinished work nobody is on to a GitHub issue.
`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` held the run logs from
2026-11-09 on until they were sorted the same way and the file was deleted (`f0f870e305`).

## How the work is laid out

The levels are Linear's,
by the owner's word of 2026-11-23.

- An **initiative** is a broad effort with a reason behind it.
  It lives on this page,
  one heading each under "Initiatives".
- A **sub-initiative** is one outcome inside an initiative that takes several runs to reach.
  It lives on this page under its initiative:
  the outcome,
  the state,
  what comes next,
  and where its records are.
- An **issue** is one run or one action with one end that can be checked.
  While it is being worked it is an entry in the coordinating session's task list,
  marked in progress only while a run on it is live.
  A defect,
  or work nobody is on,
  is a GitHub issue (`doc/agent/issue-tracker.md`).

Neither an initiative nor a sub-initiative is ever an entry in the task list.

## Standing directions

Each is the owner's,
and each stays in force until the owner changes it.

- The ZFS route comes first.
  Heavy use of worktrees waits for it,
  by the owner's word of 2026-11-18:
  "We shouldn't use worktrees heavily before we implement the zfs parts for the git wrapper."
  For the ZFS route's own work the width is the owner's of 2026-11-23:
  "Up to 64 rounds at once.",
  and later steps' pieces are built ahead of the plan's order,
  unwired:
  "Build them ahead,
  unwired (Recommended)"
  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
- A branch may not make anything `main` does well worse.
  The owner waived it once,
  for one case:
  `doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
  "The merge bar is waived for an add above the warning's count".
- The merge bar,
  restated on 2026-11-25 and unchanged:
  a branch merges into `main` when its round has ended and its report is read,
  and no gate,
  pass,
  review or mutation run holds a merge;
  one whole container gate runs on `main` per batch of merges;
  a design branch merges as a proposal and stays one until accepted;
  Sonnet passes run only at a release,
  over everything merged since the last one
  (`doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md`).
- An alpha release of any package may be bumped and dispatched unasked;
  any other release is asked each time
  (the same record,
  "Release dispatches,
  answered 2026-11-23").
- This session answers for the wrapper,
  the Pi driver,
  mpm and the way Cargo is run here.
  The owner,
  2026-11-13:
  "You're responsible for the driver,
  the wrapper,
  mpm,
  cargo.
  If you find a issue you or a subagent fix it."
- Every run is kept moving:
  when one ends,
  its result is read,
  recorded here,
  and its next step started in the same turn.
- No limit on runs is in force for memory
  (`doc/handover/agents-at-once-on-this-machine.md`,
  "The freeze of 2026-11-22 set no limit on runs").
- Who writes and who reviews:
  the skill `pi-as-subagent`,
  section "Who writes and who reviews".
- The owner,
  2026-11-23:
  "Do whatever you need to do,
  but don't abandon any work."

## The state at 2026-11-25T23:25-05:00

`main` was `7bc2c0d820`.

### Rounds open tonight

- Step 17's forced kernel calls,
  window `zfs-step-17-forced-arm-ends-holders-and-measures-its-wait`,
  branch `feat/git-policy-cli-zfs-step-17-forced-arm-act-proven-in-a-guest`,
  started at 23:11.
- Step 22's guest half,
  window `zfs-step-22-guest-half-waits-sweep-and-add-against-git`,
  branch `docs/git-policy-cli-zfs-step-22-guest-half-measurements`,
  started at 22:54.
- The linter's alpha.9 release-pass fixes,
  window `monochromatic-lint-alpha-9-release-pass-fixes`,
  branch `fix/monochromatic-lint-layout-allow-list-release-pass-findings`,
  started at 23:16.
- The layout rules' stale lists,
  window `layout-rules-on-with-a-shrinking-allow-list`,
  branch `chore/git-policy-cli-layout-rules-on-with-a-shrinking-allow-list`,
  built and waiting to merge inside the window "The crate split,
  and the layout rules that land before it"
  describes.

The first two are what that window waits for.

### Merged tonight

- 6462 as `d169557d37`:
  a clone inherits `mountpoint=legacy` from its container,
  so the add's clone needs no forbidden grant.
- The attach path's A5 as `d647717f85`:
  the attach step answers `done` for a clone already at its slot,
  and the owner kept the `first-attach` record naming the standing mount,
  in `doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md` A5.
- The host-key relay as `5a25462296`:
  option K of `doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`.
- The merge's file calls on the blocking pool as `10f4826fdc`,
  debt ledger item 18.
- The test-binary children through one helper and the fixture support both test targets share as
  `1911c927c0`,
  debt ledger items 10 and 11.
- 6478 as `3ce0c713e9`,
  debt ledger item 3b's descriptor room.
- 6473,
  6466,
  6468,
  6471 and 6480 to 6482 by their own rounds.

### The whole container gate

One gate runs on `main` per batch of merges,
and its tip is the batch's.
The last gate was at `64a71a4c9b`,
and it failed only the cause H pair,
which issues 6466 and 6468 fixed in the controls.
Every gate run,
with its counts and what it failed,
is in `doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
"The gates run on `main`,
in order,
with what each failed".
No gate has covered tonight's merges yet;
the next one is due on `main`'s tip and a failure outside the recorded baseline starts a fix round.

### Threaded-only test failures

A control that passes in the gate and fails in a threaded `cargo test` is that control's defect
(`AGENTS.md`'s SUF,
`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
"Tests hold as threads of one process").
Five are filed and open tonight:
6483 (the content-safety control misses a same-tick write),
6484 (two heartbeat controls share one manager switch),
6485 (two supervisor controls plant kills in one process-wide registry),
6486 (four `run_progress` controls read the shared test process) and
6487 (the process-wide blocking room in `foundation`).
They are debt ledger item 3's,
and they wait for the crate split's window.

A `panicked at` line from a thread named `git-policy-worker` in a combined run is not a failure:
it is the wrapper or the service started as a child by the fault-limit controls,
by the same guide's section.

### Owner questions open tonight

`doc/handover/questions-for-the-owner.md` holds:
the account companion on Ubuntu 24.04,
where AppArmor refuses `setgroups` (recorded 2026-11-25T23:30-05:00);
the forced arm's two refusals,
a locked holder and a taskless namespace,
which wait for step 17's guest result (the same time);
and `doctor`'s two managed-lock wordings,
which wait for step 18 to be wired (2026-11-25T20:35-05:00).

The next step a typed network command names at a new host key is also undecided:
the owner chose neither K2 nor C,
C2 or D,
and K alone is what runs today
(`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`).

### Memory, and how many rounds build at once

Memory is judged by the stall and by `MemAvailable` plus the ZFS ARC above `c_min`,
because `MemAvailable` understates the free memory the ARC can give back.
The settings in force and the measurement that ended the swapping are in
`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`.
One scope's build peaked at 7,080 MiB on 2026-11-25,
which sizes how many rounds compile at once,
and the guest memory a round holds is what
`doc/planning/mvm-guest-memory-held-for-a-rounds-life.md` measures.
A start that waits on memory names its test:
stall not rising (`avg10` at or under `avg60`) and `MemFree` plus the ARC above `c_min` at least one
build's measured peak.
No synthetic load is made to force a wait
(`AGENTS.md`'s NLG).

## Initiatives

### The ZFS worktree route

Why:
on this machine's ZFS pool `git worktree add` falls back to slower routes,
and the owner made a ZFS route the first priority on 2026-11-17.
The design is decided in `doc/decision/cli-git-zfs-worktree-design.md`
and built step by step from `doc/planning/cli-git-zfs-worktree-implementation-plan.md`.

State,
at 2026-11-25T23:25-05:00,
by the plan's 23 steps.
Steps 1 to 16 are on `main`,
with 12 to 16 wired;
and the removal command path is still cut:
steps 15 and 16 name commands and `doctor` lines that steps 17 and 18 now have,
and the switch still needs step 22's request budget,
so nothing calls `enter_the_removal` yet and `source_guard`'s
`removal_command_path_writes_nothing_tests.rs` holds the cut.

- Step 9's unmount,
  staging and reattach is done (`5d93d9cfd7`),
  behind the pin plan step 19 writes,
  and nothing calls it yet.
- Step 10's add route serves an add whose source is the main checkout on a checkout dataset only step 19
  makes (`SPEC.md`,
  "The add that registers a ZFS clone before it fills the slot").
  Its build matches its accepted designs and its guest arms pass.
  The three points that kept a person's add off it are closed:
  6462 as `d169557d37`,
  2C as `98cc2c9420`,
  and the attach path's A5 as `d647717f85`.
- Steps 11 to 16 are each merged with their guest arms,
  and their limits are `SPEC.md`'s "What this release does not yet do".
- Step 17's cancel command merged as `df15f27a7a`,
  and the forced arm's decision half as `64a71a4c9b`;
  its kernel calls are tonight's round.
  The forced removal is measured for the holder kinds the guest showed and refuses the rest
  (`doc/decision/cli-git-zfs-worktree-design.md`,
  "What still gates shipping").
- Step 18's `doctor` checks merged unwired (`838b7f5e71` and `d463965dcd`),
  as did the store and `snapdir` delegation check (`2bf72fb230`).
- Step 19's migration entry and units are built ahead (`72a0be07ef`),
  and the migration has not run on this host.
  The owner answered every choice
  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`),
  and the chosen design merged as `1ef4afefa8`.
- Step 20 merged unwired (`a7502138a4`);
  writing `sanoid.conf` is the owner's act.
  Step 21 merged.
- Step 22's guest half is a round tonight,
  and its storage half is the owner's act.
- Step 23 is not started.

Before a person's add takes the route:
wiring steps 17 and 18 with step 22's request budget and lifting the removal path's cut;
step 19's migration on this host;
steps 19,
22 and 23;
and a release carrying them,
installed.
Nothing of the route is installed on this machine,
and applying anything ZFS-related to this machine is the owner's act.

Next:
merge step 17's round and step 22's when they end,
read each report,
and start what its "Next" names in the same turn.

Rounds keep their files under `~/temp/agent/<round name>-<date>/`,
with the report in its `keep/`.
The scripts that start a round,
the briefs and the note every guest start reads are in
`~/temp/agent/zfs-open-item-design-rounds-launch-2026-11-22/`.

#### Accepted designs that wait for their build steps

- Private propagation after a replacement mount,
  for step 19:
  `doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`.
- The triggers a lost last event waits for,
  for steps 19 and 22:
  `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`.
  The wrapper left that design's worktree in place at the merge;
  look whether it is gone before removing it by hand.
- The forced removal's V10 answer for a locked holder,
  and its taskless arm,
  both waiting for step 17's guest result and both put to the owner:
  `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`.

Each one's acceptance is the section "The design is accepted" of its decision record.

#### Later steps' pieces built ahead, unwired

Pieces that need only work on `main` are merged as code no person's command reaches,
so their steps wait only for their wiring.
What is still built ahead:
step 18's `doctor` lines (`838b7f5e71`,
`d463965dcd`,
`2bf72fb230`),
step 19's migration entry and units (`72a0be07ef`),
and step 20's `sanoid.conf` writing (`a7502138a4`).
Step 16's deferred-destroy checks,
step 15's deferred-origin check and step 13's managed lock are wired,
though no command reaches them while the cut stands.
Step 12's move refusal and unmounted report are wired and in use (`3ea4fa16c3`).

### The crate split, and the layout rules that land before it

Why:
the ZFS route's rounds build this crate alone,
and one build of it peaked around 6.6 GiB,
of which half its test code left out measured 4.7 GiB
(`package/git-policy/cli/doc/development-profile-measurements.md`,
"Memory per build,
measured 2026-11-25").
The owner chose how to cut it on 2026-11-25 at about 22:05:
"Window after live rounds (Recommended)".

State:
no new round starts in `package/git-policy/cli` until the two rounds open tonight,
step 17's forced kernel calls and step 22's guest half,
have merged.
Then the layout rules' two lists are regenerated against `main` with their round's generator and merged,
the crate is formatted by the root `rustfmt.toml` in one commit of its own,
and the split's rounds run on that tree
(`doc/planning/cli-git-source-layout.md`,
"The window,
as the owner set it on 2026-11-25").

Waiting for that window:
debt ledger items 3 (the directory-chain descriptor tests),
7,
12,
15 and 19
(`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`),
and issues 6483 to 6487,
the threaded-only failures.

The layout rules `AGENTS.md` sets for every crate are switched on for this one,
with every finding of the two rules on a list that only shrinks,
by the owner's answer "Everything now,
untangle over time."
(`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`,
"What the list holds").
`monochromatic-lint` `0.5.2-alpha.9` carries the `allow` option the list needs and is locked
(`ef648a619e`).
Against tonight's `main` the branch's lists are stale,
and the defects the alpha.9 release pass found are fixed on `main`,
released as `alpha.10` (`6cfbcbb5f4`);
`toolchain.lock.json` still locks `alpha.9`,
so the lock's bump is a step of its own
(that record's "When the rules land").

Every crate outside this one is formatted by the root `rustfmt.toml`,
which the owner set on 2026-11-25.

### The wrapper's test gate tells the truth

Why:
one gate on `main` per batch of merges is the check the batch rest on afterwards,
and a control that fails once in a gate for no fault of the product costs a round each time.

State:
the gate's known causes are lettered in
`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
with what each fixes and the runs each appeared in.
Cause H,
the two standing failures,
is diagnosed and fixed in its controls
(issues 6466 and 6468).
AppArmor enforcement costs `main` no test now,
and every rule the work needed is in
`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
with the owner's "Always widen apparmor."
A denial the work meets is answered by a rule in the profile's `local/` file,
never by moving the work aside and never by naming the profile.

Next:
run the next whole gate on `main`'s tip,
read its window's denials with `list-denials-in-a-window.ts`,
and start a fix round for any failure outside the recorded baseline.

#### The Markdown the line-break sweep left out

`monochromatic-lint`'s `markdown/semantic-line-breaks` still reports in the files the whitespace-only
sweep of 2026-11-23 left out,
because the branches that were changing them could not take the diff:
`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md` alone reports 683 findings on
`main`,
and everything under `package/git-policy/cli` and `package/pi-shared/subagent-driver/README.md` is the
same.
`fix-whitespace-only-line-breaks.ts` in `~/temp/agent/lint-line-breaks-of-the-2026-11-23-markdown/`
applies that fix,
and each file it rewrites is equal to its old text with whitespace collapsed,
which is how the sweep's 108 files were committed.
Run it once the rounds that edit those files have merged,
one file's commit each.

#### The release passes

`0.6.1-alpha.46` is released,
the first with one program:
`monochromatic-git-policy-cli-v0.6.1-alpha.46` published
(publish run 45608993443,
archives for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`,
and an alpha is not published to crates.io).
The three Sonnet passes over `alpha.45..955051b29f` are read,
and the owner answered their four questions
(`doc/decision/cli-git-alpha-46-release-pass-four-answers.md`):
"Put `false` back (Recommended)",
"A:
full spelling,
not yet (Recommended)",
"B:
refuse only the slot (Recommended)" and
"B:
skip the check below alpha.46 (Recommended)".
Their fixes are merged:
the ZFS route's as `7ed5ec0b34`,
the one-program pass's as `88f5fff28c` and `ff474f29ee`,
and the commit,
push and LFS pass's as `90848c599c`.
The one defect the passes left open is the next step a typed network command names at a new host key,
which is the owner's question "Owner questions open tonight" names.

### The desktop stays usable under agent runs

Why:
the desktop froze for minutes on 2026-11-22 under this session's runs.

State:
two kernel settings,
`vm.watermark_scale_factor` 125 and `lru_gen` `min_ttl_ms` 1000,
are in force and kept across reboots by `/etc/sysctl.d/90-vm-watermark-scale-factor.conf` and
`/etc/tmpfiles.d/lru-gen-min-ttl.conf`,
by the owner's "Install both files (Recommended)" of 2026-11-24
(`doc/decision/memory-settings-persist-and-lfs-url-mask-shows-short-segments.md`).
The swapping that followed was `kswapd` and not the ARC,
and the fix that held was Madvise defrag with `compaction_proactiveness` 20;
the measurement,
its limit,
and the zram writeback to the old NVMe are in
`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`.
`systemd-oomd` reads the cgroups again since AppArmor's rules allow it.

Next:
after the next reboot,
read both values once to confirm the files applied them,
and read `/proc/pressure/memory` once before starting rounds.

## Held work

The wrapper and driver backlog waits under the owner's hold of 2026-11-18,
and other packages' work waits for the owner to name it.
It lives on GitHub,
by the owner's standing instruction of 2026-11-23
(`doc/agent/triage-labels.md`,
"Blocked work"):
an issue that waits on something carries a `blockedBy:<specific>` label,
most of them `blockedBy:zfs-route`,
some `blockedBy:owner-decision`,
some `blockedBy:wrapper-focus`,
and a few another issue's number.

What no public tracker holds,
or that is no issue:

- When the held queue is drained,
  the coordinating session re-reviews the whole wrapper effort once.
- Nothing of the WireGuard tools (`package/wg`) is deployed.
  Deploying is the owner's act,
  it takes a live connection down and up,
  and its steps were written for the owner on 2026-11-11 in a scratch digest,
  not in the repository.
- A scan on 2026-11-14 found secret values in the saved sessions of six Pi runs.
  Those sessions are never resumed or opened,
  and rotating the values is the owner's.

## Things a new session would not find elsewhere

- Do not run `pnpm install` in a new worktree (issue 6339).
- The coordinating session makes a host change the owner has authorised itself,
  with the passwordless `sudo`,
  and does not hand it back.
  The owner,
  2026-11-23,
  after enabling `audit-rules.service` when the session had left that to the owner:
  "I didn't need to run it myself but I just ran it as a courtesy."
  The rule against acting on the system service manager in the skill `pi-as-subagent` binds the agents a
  session briefs.
  Applying anything ZFS-related to the host stays the owner's act.
- No round runs `mise run toolchain:install`,
  not even for one tool in its own worktree:
  it waits for issue 6316,
  in the owner's words
  "Because we need to make mpm-shadow prefer system package managers on non-atomic distros first."
- An answer from the owner changes the bar for reviewers that are already running.
  Send it to each of them in the turn it is recorded.
- A question to the owner blocks the coordinator until it is answered.
  Start every round that can run before asking one.
- A round's report is not evidence of its edits.
  Read the diff before a design goes to the owner or a branch is merged.
- A Pi round can hang in one command of its own without the driver noticing.
  The unit `watch-pi-rounds-for-a-command-that-never-returns`,
  started from the script of that name in the launch folder,
  ends and prints the process when a round's tool call is older than 100 minutes.
  It runs for three hours at a time;
  start it again while rounds are going.
- The repository's tool links live in the main checkout's `.cache/toolchain-bin`,
  so `monochromatic-lint` and `file-enforcer` are there and not in a fresh worktree.
- Orphaned cargo build folders under `~/.cargo/build`,
  left by integration passes,
  are not removed without the owner.
- Applying anything ZFS-related to this machine is the owner's act.
  Old drives stay read-only and unwiped until the ZFS route is released.
