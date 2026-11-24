# cli-git implementation handover

This page holds only the open work of the Git wrapper effort (`package/git-policy/cli`) and its current state.
Nothing on it is approved or final:
it is an agent's account,
and only the owner declares work complete.
It was rewritten on 2026-11-23T04:41:15-05:00,
when the owner asked for the handover to be cut down to what is relevant.

## Where the rest of this page went

Until commit `c84e2f1dbf` (2026-11-23T04:39:44-05:00) this path held 49,707 lines:
run logs from 2026-11-09 on,
the owner's decisions,
open findings and working rules.
That text is moved,
unchanged,
to `doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md`.
A document that points at a heading of this page that is no longer here means that file.

The old file is being sorted,
range by range,
into the places such facts belong,
and it is deleted when its last range is sorted:

- the owner's decisions go to records under `doc/decision/`;
- root causes and workarounds go to `doc/troubleshooting/`;
- unfinished work that nobody is working on goes to GitHub issues;
- an account of a run that carries no lasting fact is deleted.

The old file is not edited meanwhile,
so that a line number in it means the same thing to every sorter.
"The handover holds only open work" under "Initiatives" says how far the sorting is.

## How the work is laid out

The levels are Linear's,
by the owner's word of 2026-11-23.

- An **initiative** is a broad effort with a reason behind it.
  It lives on this page,
  one heading each under "Initiatives".
- A **sub-initiative** is one outcome inside an initiative that takes several runs to reach,
  such as making the ZFS sweep and start probe work with no configuration file.
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
- Heavy use of worktrees waits for it.
  The owner,
  2026-11-18:
  "We shouldn't use worktrees heavily before we implement the zfs parts for the git wrapper."
  About thirty wrapper and driver branches that wait for a review are held under this
  ("Held work").
- A branch may not make anything `main` does well worse.
  The owner waived this once,
  for one case:
  `doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
  "The merge bar is waived for an add above the warning's count".
- A branch merges into `main` when its round has ended and its report is read:
  no gate,
  pass,
  review or mutation run holds a merge.
  Sonnet passes run only at a release,
  over everything merged since the last one;
  one whole gate runs on `main` after each batch of merges;
  a design branch merges when its round ends and stays a proposal until accepted
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

From lines 39306 to 39308 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Owner:
  "Do whatever you need to do,
  but don't abandon any work."

## The restart the owner announced on 2026-11-23

The owner,
on 2026-11-23,
shortly before this state was read at 2026-11-23T05:30:24-05:00:
"I need to restart Claude Code very soon.
Try to make sure everything can survive that."
And then:
"In fact,
I'd have to restart my whole system soon too.
Don't abandon anything,
tell me whenever it is a good time."

So no new round or agent is started until the machine is back.
That sets aside "its next step started in the same turn" under "Standing directions" for that time.
The machine is back:
the current boot began at 2026-11-23T15:12:47-05:00,
after several restarts from about 05:42,
and AppArmor enforces on it since (`doc/handover/full-system-policy-handover.md`).
The owner,
on the session's return:
"I'm back.
apparmor is now on.
Add rules to unblock whatever you need to do as you go this session."
Every run that the restarts ended was picked up at 16:31 as "Picking a run up" says,
and the two Pi rounds go on as
`pi-zfs-step-9-unmount-after-the-machine-restarted` and `pi-clone-rung-partial-clone-after-the-machine-restarted`,
on `opencode-go`.
Writer rounds had run on standard Neuralwatt DeepSeek,
a pool the owner closed on 2026-11-11,
because the skill `pi-as-subagent` listed it first and the round launchers named it;
the owner saw the calls on 2026-11-23:
"I'm seeing a lot of deepseek-v4.1-flash (standard) calls in neuralwatt which has drained a lot of my quota.
This shouldn't happen."
The skill is corrected in `e24cf7d46d`,
and the launchers now take their pool from `writer-pool-for-a-round.ts` in the launch folder,
which refuses that pool.
The driver and the worktree launcher now refuse a run on a closed pool,
and the qwen plan's DeepSeek pool is open for runs that name it:
merged as `65a1dc0e78`,
and its pass's findings merged as `cdc21b3661`:
a reopened pool now reaches the run's unit (shown on real units),
`lint` is green,
and a malformed `PI_RUNS_POOL_LIMITS` refuses only the pool it touches.
The owner answered the pass's three questions on 2026-11-23
(`doc/decision/pi-driver-closed-pools-and-the-worktree-launchers-default.md`):
the qwen plan's pool keeps 5,
a reopened pool is forwarded into the unit,
and a worktree launch naming no provider takes the first open pool instead of refusing.
Running:
`pi-driver-launcher-takes-the-first-open-pool`,
on that last answer,
on `fix/pi-shared-subagent-driver-launcher-takes-the-first-open-pool`.
A fresh worktree's `lint` and `lint:types` fail on a stale store link
(`ERR_PNPM_LOCKFILE_CONFIG_MISMATCH` when the wrapper relinks it),
seen by three passes today (issue 6339's family).
The repository's tool links folder `.cache/toolchain-bin` is missing,
so `file-enforcer` and `monochromatic-lint` are not on `PATH`;
only `toolchain:install` makes it,
and the owner said it waits for issue 6316:
"Because we need to make mpm-shadow prefer system package managers on non-atomic distros first."
A run that ends is still read and recorded here.
Under each sub-initiative,
"Running" and "Next" say what was going at that reading and what is started first afterwards.

### What a restart ends

- A restart of Claude Code ends the coordinating session's own agents,
  which are the Sonnet passes and the sorters,
  and its waiting shell commands.
  Units of the account's service manager go on:
  the Pi rounds,
  the gates and scopes the passes started,
  and a guest.
- A restart of the machine ends those too.
- Nothing under `~/temp/agent` is counted on to outlast a restart of the machine.
  "Only in scratch" lists what would be lost with it.

### A good time, by checks

Safe,
meaning a restart loses no work and costs only what was running:

- this page is on `origin`;
- `list-what-a-restart-would-lose.ts` in the launch folder prints only the main checkout,
  whose four changed files are not this session's and are left alone;
- `git log --branches --not --remotes --oneline` prints nothing;
- every run still going has a commit of its work on `origin` or a `state-so-far.md` in its `keep/`.

Clean,
meaning nothing is interrupted:
no `pi-*` unit,
gate unit or guest is running,
no gate container is left,
and every agent has returned.

### Picking a run up

Every running round and agent was told at 2026-11-23T05:25-05:00 to commit its work at each step,
a broken state too,
to keep each commit on `origin`,
and to keep `state-so-far.md` current in its `keep/` folder.

A Pi round that the restart ended goes on from its own session:
from the launch folder,
`node resume-a-build-round.ts <round folder> <worktree name> <branch> <continuation file> <a new unit name>`.
It starts Pi from the newest session of that worktree and makes the worktree again when it is gone.
The continuation file is the round's own,
in its round folder,
and the session already holds its turns.
A round that ended by itself has `summary.json` in its run folder
and its report under the round folder's `keep/`.

A Sonnet pass or sorter that a restart ended is resumed,
not started again:
its transcript outlives a restart of Claude Code and of the machine
(`~/.claude/projects/-home-user-Monochromatic/<session id>/subagents/agent-<id>.jsonl`,
with its description in the `.meta.json` beside it),
and SendMessage to its agent ID resumes it from there.
The six passes and the sorter that a usage-limit refusal stopped at about 05:30 were resumed that way at 2026-11-23T16:31-05:00,
each told what the restarts ended and to read its `state-so-far.md` first.
A fresh agent with its brief is the fallback when a transcript is gone.
The briefs are in the launch folder
(`brief-*.md`,
and `rules-for-an-integration-pass.md`,
which every pass reads)
and in the housekeeping folder (`brief-sorter-*.md`).
The second pass on the no-configuration branch and the fourth recheck of the object reader have no brief file:
each is briefed again from the rules file,
the earlier review in its `keep/` folder and its state file.

After a restart of Claude Code alone,
a gate unit whose pass is gone is left to end,
and its containers and image tag are then cleared as the gate rules say.
The pass units listed here before all read `inactive` at 2026-11-23T19:4x-05:00.

### Results that are not acted on yet

- Two run-log sorters' reports
  (`keep/report-as-returned-to-the-coordinator.md` in `sorter-lines-153-to-7110/` and `sorter-lines-21104-to-28113/`
  of the housekeeping folder)
  propose rule text and name contradictions;
  the ones acted on so far are the lost hall-monitor answer and the stale sentence this list held.
  Still open from them:
  issue 6295 is open while 6299,
  the same defect,
  is closed with its fix on `main` (`47a403a0e2`);
  `.agents/skills/pi-as-subagent/SKILL.md` says "The practical limit is 20",
  against `doc/handover/agents-at-once-on-this-machine.md`;
  and two records were made for the commit service needing no service manager,
  one from each sorter.

### Only in scratch

These exist only under `~/temp/agent`,
and a lost scratch folder would cost redoing them:

- the launch folder's scripts and briefs;
- each reviewer's files under `gate-review-*/keep/` and `integration-*/keep/`,
  which hold the findings,
  the veto lists in plain words and the wording of the questions for the owner;
- the two finished sorters' ledgers and issue drafts;
- the backlog's ledger with the links of what was filed and labelled,
  which GitHub itself also shows.

The owner is asked whether these go onto a branch on `origin` before the restart.

## Initiatives

### The ZFS worktree route

Why:
on this machine's ZFS pool `git worktree add` falls back to slower routes,
and the owner made a ZFS route the first priority on 2026-11-17.
The design is decided in `doc/decision/cli-git-zfs-worktree-design.md`
and built step by step from `doc/planning/cli-git-zfs-worktree-implementation-plan.md`.

State:
plan steps 1 to 8 are built and on `main`.
Step 9 is in two branches.
Ten open items of the plan needed a design before their steps can be built;
four are accepted and on `main`,
and six are in review.
Nothing of the route is installed on this machine,
and nothing is published.

The designs' rounds keep their files under `~/temp/agent/<design name>-2026-11-22/`
and their reviews under `~/temp/agent/gate-review-<name>-2026-11-22/keep/`.
Build rounds keep theirs under `~/temp/agent/<round name>-2026-11-22/`,
and their integration passes under `~/temp/agent/integration-*-2026-11-22/keep/`.
The scripts that start a round are in `~/temp/agent/zfs-open-item-design-rounds-launch-2026-11-22/`.

#### The sweep and the start probe need no configuration file

Outcome:
with no configuration file the root handler finds the checkout,
the clone container and the probe dataset by their names.

State:
branch `feat/git-policy-cli-zfs-sweep-and-start-probe-need-no-configuration-file` at `ae7e91e94b`,
after a fix round on its first integration pass.
The owner's answers are in `doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
"Four further answers on the look-up with no configuration file".
Its second Sonnet pass
(`review-second-pass.md` in
`~/temp/agent/integration-pass-zfs-sweep-and-start-probe-need-no-configuration-file-2026-11-22/keep/`)
found it fit after two fixes:
the documents say the route cannot read `snapdir`,
which it can (B1),
and the probe's refusal prints the sweep's remedy,
which does not settle the probe (B2).
Told what hiding snapshot directories comes to,
the owner chose `snapdir=disabled` on the checkout and the clone container,
set by step 19 and reported by step 18
(`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
"What hiding comes to,
answered 2026-11-23").
The round on B1,
B2,
B4,
the lows and that answer ended with every item done
(`keep/report-pi-zfs-no-configuration-second-pass-findings.md` in
`~/temp/agent/zfs-sweep-and-start-probe-need-no-configuration-file-2026-11-22/`).
Its merge into `main` stopped on two conflicts with step 9's unmount:
`package/git-policy/cli/src/native/lib.rs` and `package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`.
The SPEC now says the probe with no file costs about 25 ms once per boot (1.14 times),
spread over three look-ups,
with the cause not found.
Running:
`pi-zfs-no-configuration-merge-main-after-step-9-unmount`,
merging `main` and resolving both.

Next:
merge it;
then a fresh round that removes the probe's 25 ms,
or moves it off the path a person waits on.

#### Step 8's follow-up: the recheck's findings and the sweep's cost

Outcome:
the attaching sweep's cost per mounted child is flat,
and the recheck's findings on the merged step 8 are closed.

State:
branch `fix/git-policy-cli-zfs-step-8-recheck-findings-and-sweep-cost` at `c87a50a09b`.
Its second Sonnet pass
(`~/temp/agent/integration-second-pass-zfs-step-8-recheck-findings-and-sweep-cost-2026-11-22/keep/review.md`)
found the first pass's two cases closed and the steady sweep at 0.20 to 0.79 of `main`'s time,
and the branch not fit on P2-H1:
renaming a mounted `legacy` dataset fires no mount-table event,
so after a swap by name the sweep answers `attached` and files a false `done` receipt where `main` refuses.
The owner chose to read a standing mount's source with `statmount(2)` by its mount number,
falling back to the live table on older kernels,
and said the question was framed wrong in ranking the easier option first
(`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
"A standing mount's source is read by its mount number";
the ranking rule is now `AGENTS.md`'s `RBE`).
The round on that,
P2-M1 to P2-M3 and the lows ended with each done and a control red without its guard
(`keep/report-pi-zfs-step-8-sweep-second-pass-findings.md` in
`~/temp/agent/zfs-step-8-recheck-findings-and-sweep-cost-2026-11-22/`),
but its context ran out before the timing set,
so the cost of the added `statx` and `statmount` is unmeasured.
Its merge into `main` stopped on a conflict with step 9's unmount in `sweep_one_clone_tests.rs`.
Its corrective sentence for `f334d1db60`'s message is posted as a commit comment.
Running:
`pi-zfs-step-8-merge-main-and-the-sweep-timing`,
a fresh round on the same branch
(folder `~/temp/agent/zfs-step-8-merge-main-and-the-sweep-timing-2026-11-23/`):
merge `main`,
then the timing set interleaved with `main`.

Next:
merge when it ends.

#### Step 9: the holder scan across mount namespaces

Outcome:
a removal learns which processes in any mount namespace hold a clone.

State:
branch `feat/git-policy-cli-zfs-step-9-holder-scan-across-mount-namespaces` at `1b574285f3`.
Its Sonnet pass
(`~/temp/agent/integration-pass-zfs-step-9-holder-scan-across-mount-namespaces-2026-11-22/keep/review.md`)
found it not fit:
under intent `unmount` the scan sets aside every copy of the clone and answers clear while four namespaces hold it,
and four medium findings.
Nothing `main` does got worse.
The owner answered its three questions
(`doc/decision/cli-git-zfs-holder-scan-names-every-holder-it-can-read.md`):
no set-aside,
read every thread and root,
and the wrapper names the holders before each destroy.
Running:
`pi-zfs-step-9-holder-scan-integration-pass-findings`,
with the answers in its inbox.

Next:
a second Sonnet pass.
This branch lands before the unmount.

#### Step 9: the unmount, staging and reattach

Outcome:
the root handler unmounts a clone through a staging folder,
puts it back when the unmount is refused,
and finishes after a kill.

State:
merged as `931d6a2c9c`,
after its round on the integration pass
(`~/temp/agent/integration-pass-zfs-step-9-unmount-staging-and-reattach-2026-11-22/keep/review.md`)
fixed the handle reopen,
wired the recovery after a kill at four kill points,
and built the owner's `cancelled` answer
(`doc/decision/cli-git-zfs-staging-under-cache.md`,
"The answer a stopped unmount files,
answered 2026-11-23").
All of it sits behind the pin plan step 19 writes,
so none reaches a user.
Not yet shown:
the busy branch's check against the pass's adversarial arm,
the pass's M1 and M2 guest arms,
VM item 15 and M3 to M9,
a host control for each of ten guards,
and the reboot phase of the kill points.
Running:
`pi-zfs-step-9-unmount-what-the-last-round-left`,
on the same branch.

Next:
merge when it ends.

#### Step 10: the add route from registration to a verified fill

Outcome:
a `git worktree add` on ZFS claims its slot,
registers the worktree already locked,
snapshots and clones the main checkout,
and fills and checks the slot,
resuming after a kill.

State:
the owner accepted step 10's two designs on 2026-11-23,
kept all 21 of their choices,
reworded the plan's pointer sentence,
and put the add's read of the plan folder in step 10
(`doc/decision/cli-git-zfs-step-10-add-time-lock-and-binding-record.md`).
The designs are on `main` as proposals:
`doc/planning/cli-git-zfs-add-binding-record-design.md` and `doc/planning/cli-git-zfs-add-time-git-lock-design.md`.
The lock reviewer's eleventh read
(`review-eleventh-read-and-step-10-question.md` in
`~/temp/agent/gate-review-zfs-add-time-git-lock-and-binding-record-designs-2026-11-22/keep/`)
left two fixes to the binding,
neither step 10's:
a wrong sentence about a `snapshotted` resume,
and `doctor` naming two commands for one plan state.
The registration half merged as `cde8ad55a8`:
the add's record family,
the lock's words,
the finder,
the recipe,
the resume states and the plan-folder read,
in `package/git-policy/cli/src/native/worktree_clone_binding_record/`,
with 70 unit controls and nothing wired into the add yet.
Its round decided seven points the designs leave open
(`keep/state-so-far.md` in
`~/temp/agent/zfs-step-10-add-route-to-a-verified-fill-2026-11-23/`),
which the binding design's writer is to put into its text.
Running:
`pi-zfs-step-10-execution-half`,
on the same branch:
the route from the claimed slot to a verified fill,
its wiring,
kills and resumes,
and the guest tests.

Next:
merge it when it ends;
then a round putting both rounds' decisions into the binding design.

#### The removal-side records: the destruction plan and the origin cleanup record

Outcome:
the owner accepts the records a removal persists.

State:
the owner accepted the three records on 2026-11-23,
choosing one `key=value` format for all three and one plan line per attempt,
and asked for the 14 edits to two accepted designs and a clause for a fallback that made no clone
(`doc/decision/cli-git-zfs-removal-side-records-and-the-adds-record-accepted.md`).
The binding merged with K14 closed as `ed8ad9a749`.
Running:
the origin record's round on the format (`continuation-one-key-value-format.md`),
the plan record's round on the history (`continuation-one-line-per-attempt.md`),
and `pi-zfs-accepted-designs-true-to-the-records`,
which applies the 14 entries by script from the review's words and drafts the clause,
on `docs/planning-cli-git-zfs-accepted-designs-true-to-the-records`.

Next:
merge each when it ends;
the owner reads the clause in that diff.

#### Step 17's designs: forced removal, and a cancelled or interrupted removal

Outcome:
the owner accepts the two designs.

State:
both are on `main` as proposals
(`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`).
The owner confirmed the direction on 2026-11-23:
unmount by the marker inside a holder's namespace,
and go ahead and report for both sandbox cases
(`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
"The unmount inside a holder's namespace,
confirmed as a direction on 2026-11-23").
That rests on Tier 0 on kernel 6.8 and in a rootless container on 7.3 over ZFS,
and Tier 1 on a guest pool
(`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`;
read against the logs in `review-second-probe-round.md` in
`~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/`).
Tier 2 needs step 17's code.
Running:
`pi-zfs-forced-removal-probe-arms-second-read-findings`,
on the read's faults in the measurements section
(which program ran where,
the heading's time,
a user-namespace holder named as unshown,
one misleading T0-10 line).

Next:
merge it;
then the acceptance question for the two designs.

#### Accepted designs that wait for their build steps

- The deferred-destroy checks,
  for plan step 16:
  `doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`.
- Private propagation after a replacement mount,
  for step 19:
  `doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`.
- The dirty-main restore on rung 1,
  for step 11,
  with its frozen proof built in step 10:
  `doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`.
- The triggers a lost last event waits for,
  for steps 19 and 22:
  `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`.
  The wrapper left that design's worktree in place at the merge;
  look whether it is gone before removing it by hand.

Each one's acceptance is the section "The design is accepted" of its decision record.

### The clone rung serves every source no slower than `main`

Why:
the owner's bar for a worktree add,
restated on 2026-11-23:
a declined add costs what `main` paid for the same source.


### The wrapper's test gate tells the truth

Why:
a merge rests on the whole container gate,
and a control that fails once in a gate for no fault of the product costs a round each time.
The gate's known causes are lettered in
`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`.
Two tests fail in every gate on this host,
the pair that document names under cause H.


#### The tests and the gate under AppArmor

Outcome:
the wrapper's host tests and its container gate run under the AppArmor enforcement of 2026-11-23 with no denial.

State:
the host's `git` profile confined the container's own Git and refused the tests' fixtures,
so every gate and host test run after the restart failed for that reason.
Two batches of rules are in,
with their causes and tradeoffs in
`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
"The Git wrapper's own tests,
gate and helpers under enforcement".
The owner:
"No,
please do rules for everything instead of a dedicated profile that agents would have to remember to use."
A gate result after the restart and before the rules is void.
Five batches are in now.
The second whole gate on `main`
(`apparmor-second-gate-on-main-20261123`,
at `a467a1d22c`,
after the fourth batch)
gave `9914 tests run: 9904 passed (8 slow), 10 failed, 36 skipped`:
the cause-H pair,
the signed-commit control,
three `ssh_terminal` controls,
three `stale_ref_lock` controls,
and the size read's child control.
Its denials led to the fifth batch,
for `sshd` and `ssh-keygen` in the gate's fixtures,
the wrapper's thread reads,
`gh`'s cache and Buildah's build-context unmount.
No denial maps to the three `stale_ref_lock` controls
(their killed fetch's hook leaves its lock and the wrapper's fetch exits 0),
and no signal denial was logged.
The audit's lost count stayed at 69,301 through that gate,
so the raised backlog dropped nothing.
Gates were released at 2026-11-23T18:02:32-05:00,
with those ten as `main`'s baseline that counts against no branch
(`note-gates-may-run-against-mains-baseline.md` in the rounds' launch folder,
sent to every running round and pass).
A gate runs through `run-one-whole-gate-in-a-worktree.ts` in the AppArmor folder,
which writes the summary and failures to its `keep/`,
clears the run's containers by label and removes the tag.
The two branch gates after the fifth batch
(object reader at `cd6f2e1185`,
issue 6345 at `04b255e004`)
each failed eight,
all in that baseline:
the signed-commit control and the unknown-host-key `ssh_terminal` control pass now,
and the three `stale_ref_lock` controls still fail.
The sixth batch lets Git run hooks of fixtures under `~/temp/agent`
(72 denied executions of `reference-transaction` in one pass's host tests)
and lets `git//ssh` run `/usr/bin/false`,
the passphrase program of the two `ssh_terminal` controls still failing (6 denials).
The gate on `main` at `25d719bc6f`,
after the seventh batch and the second batch of merges,
failed seven,
all in that baseline:
the cause-H pair,
the two `ssh_terminal` controls,
and the three `stale_ref_lock` controls;
the size read's child control passes now.
The three `stale_ref_lock` controls were the test killing `timeout` and not Git,
fixed in the test and merged as `02de1145eb`
(the account's "The three stale lock controls killed the `timeout` and not the Git").
The two `ssh_terminal` controls were `sshd-session`,
a profile of its own,
refused the fixture's `authorized_keys`;
the eighth batch gives it `owner /work/** r`
and gives `git` the cgroup controller list.
Denials are read from `/var/log/audit/audit.log` with `list-denials-in-a-window.ts`,
since neither the journal nor `ausearch`'s window showed them
(the account's "Rules added").
The gate on `main` at `6f2b9ef708`,
after the eighth batch and the `stale_ref_lock` fix,
gave `10085 tests run: 10083 passed (7 slow), 2 failed, 37 skipped`:
the cause-H pair alone,
which is not AppArmor's
(`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`, "Cause H").
Its only denials were `umount`'s `dac_override` (no test needs it)
and the browser's.
So AppArmor enforcement costs `main` no test now.
The kernel's audit backlog dropped 69,301 records at its default of 64;
the owner chose 8192,
kept across reboots,
which is set for this boot and written to `/etc/audit/rules.d/10-backlog-limit.rules`.
`audit-rules.service`,
which loads that file at boot,
is enabled since 2026-11-23,
by the owner
("Things a new session would not find elsewhere" has the owner's words).

Next:
read the denials of each later batch gate with `list-denials-in-a-window.ts`
and add rules for what this work meets;
cause H is its own work
(the account's "Cause H").

#### Tests that failed once and are not diagnosed

Issues 6348 to 6352 on GitHub hold them,
each with the journal text that was kept.
Nobody is working on them.

### The desktop stays usable under agent runs

Why:
the desktop froze for minutes on 2026-11-22 under this session's runs.

State:
two kernel settings were changed at runtime,
their effect is unverified,
and no limit on runs was set.
The account is `doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
and what a session does is in `doc/handover/agents-at-once-on-this-machine.md`,
"The freeze of 2026-11-22 set no limit on runs".
Two later readings are in `~/temp/agent/desktop-froze-under-agent-runs-memory-stall-2026-11-22/keep/`,
both under lighter loads than the freeze's.
The machine's restarts on 2026-11-23 cleared both settings,
which are runtime values;
the session applied them again at 2026-11-23T19:26:27-05:00,
since the owner's answer keeps them until the comparison.
After any restart,
read both and apply them again
(the commands are in the freeze account).
Running:
a start read at 19:26:55 under four guests,
the batch gate and the Pi rounds' builds,
and a sampler of the batch gate's container memory every 30 seconds
(scope `sample-gate-container-memory-second-evening-batch-20261123-b`).

Next:
the end read when the batch gate ends,
and the rates between the two reads;
then the persistent form of the two settings,
for the owner to apply.
A gate's fixture file system is a `tmpfs` capped at 768 MiB,
so one gate's fixtures hold at most that
(the freeze account's question 4 has the one reading so far).

### The handover holds only open work

Why:
the owner asked on 2026-11-23 for this page and the task list to be cut down to what is relevant.

State:
this page is rewritten,
and the old text is frozen in the file named under "Where the rest of this page went".
The sorting's working files,
and the ledger of which range went where,
are in `~/temp/agent/handover-cull-and-task-list-rebuild-2026-11-23/keep/`.
The held backlog is on GitHub
("Held work").
Ranges applied so far:
lines 34,867 to 49,707 of the old file,
committed as `a2b4fca7c8`.
Five sorters gave each range a disposition,
and `apply-the-cut-moves-to-their-targets.ts` in that folder moved 187 entries verbatim into 73 targets
(41 existing files and 32 new records,
each new one saying in its status that it holds moved text only),
each after a line naming its lines in the old file;
one entry was already held and was skipped.
Before that,
every quotation of the owner's inside a `delete` item was made a `move`
(`quotations-the-ended-ledgers-would-delete/keep/` and `ledgers-keep-every-owner-quotation/keep/` in that folder),
because the first check of quotations read them by alternation and by their opening characters and missed some
(its controls and flaws:
`keep/quotation-check-controls-2026-11-23.md`).
Among the owner's words the rewrite had lost are
"Treat reviews as async."
and "You should (almost) never be waiting.
There is always more work.",
now in `doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md`.

That audit read only `delete` items,
so the owner's words inside `issue`,
`home` and `open` items were never checked;
"Move hall-monitor to package-paused altogether.",
at old-file line 45973 in an `issue` item,
is held by no tracked file but the frozen one.
The run logs,
lines 153 to 34,866,
were sorted by five Sonnet sorters
(`brief-sorter-run-logs.md`)
and their 98 moves applied into 42 targets as `98d8601c93`,
188 entries already held being skipped.
The first five ranges' 35 issue drafts are filed as issues 6401 to 6427,
with comments on 13 others
(`filing-of-the-sorters-issue-drafts/keep/ledger.jsonl`).
The harness refuses a subagent's write of a report file:
"Subagents should return findings as text,
not write report files."
So a sorter's report is its final message,
saved by the coordinator as `keep/report-as-returned-to-the-coordinator.md`.
Running:
two agents scanning every non-move item for the owner's words,
one for lines 34,867 to 49,707
(`brief-owner-quotations-in-every-non-move-item.md`)
and one for the run logs
(`brief-owner-quotations-in-the-run-log-ledgers-non-move-items.md`);
and an agent filing the run-log sorters' 31 drafts
(`brief-file-the-run-log-sorters-issue-drafts.md`).

Next:
apply the two scans' amended ledgers' new moves the same way;
then delete the old file.

## Held work

The wrapper and driver backlog waits under the owner's hold of 2026-11-18,
and other packages' work waits for the owner to name it.
It lives on GitHub.
The held,
paused and future entries the coordinating session's task list carried until 2026-11-23 were each checked against `main`
and GitHub that day:
those still to do are issues now,
those GitHub already tracked got a comment where the entry knew more,
and those that had landed or were no piece of work were dropped.
Which entry went where is `backlog-to-github-issues/keep/ledger.jsonl`,
with the links in `filed.jsonl` beside it,
in the housekeeping folder named under "The handover holds only open work".
An issue that waits on something carries a `blockedBy:<specific>` label,
by the owner's standing instruction of 2026-11-23
(`doc/agent/triage-labels.md`,
"Blocked work").
The new issues and the older ones the backlog pointed at carry theirs:
most wait for the ZFS route (`blockedBy:zfs-route`),
some for a decision of the owner's (`blockedBy:owner-decision`),
some for the owner to name work outside the wrapper (`blockedBy:wrapper-focus`),
and a few for another issue,
by its number.

The old file's "Open work" section,
from line 34,867,
is sorted and applied,
and the held branches it names are in the issues above.
The run logs name more unmerged branches:
the sorter of lines 14,100 to 21,103 found 44 named by no issue,
grouped into eleven drafts in its `keep/`,
which go to GitHub with the other drafts.

What that check could not put on a public tracker,
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
  The rule against acting on the system service manager in the skill `pi-as-subagent` binds the agents a session briefs.
  Applying anything ZFS-related to the host stays the owner's act.
- No round runs `mise run toolchain:install`,
  not even for one tool in its own worktree:
  it waits for issue 6316,
  in the owner's words "Because we need to make mpm-shadow prefer system package managers on non-atomic distros first."
  Issue 6354's round ran it for `cargo-nextest` (it installed nothing).
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
- Orphaned cargo build folders under `~/.cargo/build`,
  left by integration passes
  (`68/a5e436cd0c0598` and four from issue 6342's second pass),
  are not removed without the owner.
- Applying anything ZFS-related to this machine is the owner's act.
  Old drives stay read-only and unwiped until the ZFS route is released.
