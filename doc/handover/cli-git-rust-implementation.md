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
That text was moved,
unchanged,
to `doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md`.
A document that points at a heading of this page that is no longer here means that file,
which `git show c84e2f1dbf:doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` still prints.

The old file was sorted,
range by range,
into the places such facts belong,
and deleted as `f0f870e305` (2026-11-23T20:56:00-05:00) when its last range was sorted.
Every moved entry names its lines and the commit at which the file held them;
the sorting's ledgers and scripts are in `~/temp/agent/handover-cull-and-task-list-rebuild-2026-11-23/`.
The sorting put:

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
  For the ZFS route's own work the owner set the width on 2026-11-23:
  "Up to 64 rounds at once.",
  and later steps' pieces are built ahead of the plan's order,
  unwired:
  "Build them ahead,
  unwired (Recommended)"
  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
  A rising memory stall pauses new starts.
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
plan steps 1 to 10 are built and on `main`;
step 11 is being built (`pi-zfs-step-11-dirty-main-restore`),
with issue 6452 (a dirty checkout takes rung 3 until then) fixed as `c493a97158`;
`main` fails `source_guard's the_release_code_keeps_blocking_work_off_the_runtime` since the account companion merged (`8ce60ff4f5`, its reaper thread at `companion_process_calls.rs:224`),
which `pi-fix-companion-reaper-thread-source-guard` fixes,
and every writer round now runs the `source_guard` module before it reports;
the gate on `main` at `adcb708071` (run `native-mEmcZt`) gave `10855 tests run: 10843 passed (6 slow), 12 failed, 42 skipped`:
the cause-H pair, that guard,
and nine companion tests that read the host's `XDG_RUNTIME_DIR`,
which the gate's container does not set (`pi-fix-companion-tests-inject-their-runtime-dir` fixes them);
step 10's add route gated on a checkout dataset that only step 19 makes,
so no person's add takes it yet.
Pieces of steps 12,
13,
15,
16 and 18 are built ahead and on `main`,
unwired.
Nothing of the route is installed on this machine,
and the release after alpha.45 is the first to carry steps 9 and 10.

The designs' rounds keep their files under `~/temp/agent/<design name>-2026-11-22/`
and their reviews under `~/temp/agent/gate-review-<name>-2026-11-22/keep/`.
Build rounds keep theirs under `~/temp/agent/<round name>-2026-11-22/`,
and their integration passes under `~/temp/agent/integration-*-2026-11-22/keep/`.
The scripts that start a round are in `~/temp/agent/zfs-open-item-design-rounds-launch-2026-11-22/`.

#### Step 9: the unmount, staging and reattach

Outcome:
the root handler unmounts a clone through a staging folder,
puts it back when the unmount is refused,
and finishes after a kill.

State:
done on `main` as `5d93d9cfd7`,
behind the pin plan step 19 writes,
so none reaches a user:
the handle reopen,
the recovery after a kill at four kill points and over a reboot,
the owner's `cancelled` answer
(`doc/decision/cli-git-zfs-staging-under-cache.md`,
"The answer a stopped unmount files,
answered 2026-11-23"),
the busy branch's check against the integration pass's A12 arm,
the A10 arm,
ten host controls each red without its guard,
and the pass's M1 to M9 as guest arms,
none of which found a code fault,
with M3's positive reader control showing the bind's slave copy and a leak caught.
Rounds and logs:
`~/temp/agent/zfs-step-9-unmount-staging-and-reattach-2026-11-22/keep/`.

Next:
nothing until step 17 or step 19 calls it;
its Sonnet pass is the next release's.

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
The registration half merged as `cde8ad55a8`
(`package/git-policy/cli/src/native/worktree_clone_binding_record/`),
and the execution half as `bee6531780`
(`package/git-policy/cli/src/native/worktree_clone_add_route/`):
the route in the binding design's write order,
asked by `run_served` before the ladder,
and served only for an add whose source is the main checkout on a ZFS dataset whose leaf is `monochromatic-git-checkout`,
which only step 19's migration makes;
every other add answers `None` and keeps its route
(`SPEC.md`,
"The add that registers a ZFS clone before it fills the slot").
Its guest run of 2026-11-24T00:12-05:00 passed three tests:
the whole add with a second add refused,
a kill at each of nine phases with its resume,
T22,
T25 and T26;
three guards were shown red,
not every one.
T27 (a record across a reboot) and T28 (a flushed write after a power loss) did not run.
Both rounds decided points the designs leave open
(`keep/state-so-far.md` in
`~/temp/agent/zfs-step-10-add-route-to-a-verified-fill-2026-11-23/`).

The build decisions are in both designs (`3e651a9678`);
the eleventh read's two fixes were already in.
That round found the build short of its accepted designs in four places
(its report in `~/temp/agent/zfs-step-10-binding-design-takes-the-build-decisions-2026-11-24/keep/`):
the preflight reads no grants,
the refusal beside a standing plan names the removal even where the plan design says it refuses,
the account companion the start-probe record put in step 10 is unbuilt,
and the gate serves only the step 19 dataset;
those are merged as `2d9fa02ea5` but for the gate,
the add's printed sentence and the grant set,
proposed as `doc/planning/cli-git-zfs-step-10-open-choices.md` (`dce1e893a3`);
its Sonnet read (`read-of-the-step-10-open-choices.md` in `~/temp/agent/gate-review-zfs-step-10-open-choices-2026-11-24/keep/`)
found a dirty checkout refused on every widened reach until step 11,
the add printing a Rust `Debug` dump of the plan stop today,
and 3B's example false;
the owner answered E, 2C and 3C and the migration's runner and timing
(`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`),
and `pi-zfs-step-10-answers-into-proposal-and-plan` writes them in;
the revision merged as `dc0635971e` (choice 1 `E > C > G > A` with H asked on its own and W an add-on; choice 2 `2C > 2F > 2E > 2A`; choice 3 `3C > 3B > 3A > 3D`);
a Sonnet read of it writes the owner's questions (`~/temp/agent/gate-review-zfs-step-10-open-choices-revision-2026-11-24/`);
its two defects on `main` are issues 6450 (the add printed a `Debug` dump; fixed as `ca615ddbe1`, controls red then green)
and 6451 (the scope finding printed no command; fixed, each wide grant now prints the narrowing `zfs allow` before the `zfs unallow`);
and `SPEC.md` names `canmount=off` and a fall to rung 3 the code does not do.
Waiting for the memory stall to ease
(`full` avg60 1.02 and rising at 2026-11-24T01:46-05:00):
`pi-zfs-step-10-build-matches-its-accepted-designs` and `pi-zfs-step-10-account-companion-probe`
(the companion merged as `8ce60ff4f5`:
it runs after the route gate and before the claim with a 100 ms deadline,
about 0.12 to 0.14 of an add's time,
with its guest arm,
the whole add after a skip and `doctor`'s print still to come once a guest reaches the route),
whose prompts are ready in their folders under `~/temp/agent/` (names ending `-2026-11-24`);
the first also writes up what widening the gate would need, for the owner.
T27 and T28 merged as `9c80f28d21`:
a reboot with an export and import, and a sysrq power cut, each read back green in a guest and red with its guard removed.

Next:
merge both when they end.

#### The removal-side records: the destruction plan and the origin cleanup record

Outcome:
the owner accepts the records a removal persists.

State:
the owner accepted the three records on 2026-11-23,
choosing one `key=value` format for all three and one plan line per attempt,
and asked for the 14 edits to two accepted designs and a clause for a fallback that made no clone
(`doc/decision/cli-git-zfs-removal-side-records-and-the-adds-record-accepted.md`).
The binding merged with K14 closed as `ed8ad9a749`.
The 14 edits and the no-clone clause merged as `ab140367b3`,
the origin record in its one `key=value` format,
and the plan record with one line per attempt as `8457f7fcab`
(`tuple=<kind> <attempt> <outcome> <guid> <createtxg> <errno> name=<name>`,
the attempt a per-snapshot number from `0`).

Next:
the records are built in steps 14 and 16;
the owner may read the no-clone clause,
quoted to the owner on 2026-11-23.

#### One program only: the root modes join the wrapper

Outcome:
the release holds one executable,
and the ZFS route's root modes are modes of `monochromatic-git-policy-cli`.

State:
the owner answered "One program only,
ever" on 2026-11-23
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`);
alpha.45 shipped the second program by the owner's "Let it finish (Recommended)".
The design merged as a proposal,
`eee444cc0b`
(`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`),
with four choices each ranked A over B:
the selector (the installed name plus a word),
what the names point at (links to a root-owned copy under `/usr/libexec/monochromatic-git-policy-cli/`),
where the dispatch sits (the first statement of `main`,
before the panic hook and the environment),
and whether the account's `PATH` keeps a short name for the modes (no,
with `doctor` printing the whole command).
Its Sonnet gate review
(`review-of-the-one-program-design.md` in
`~/temp/agent/gate-review-zfs-root-modes-in-the-one-wrapper-executable-2026-11-24/keep/`)
found that the name selector reverses `SPEC.md`'s "the executable does not read the name it was started under"
and two source guards,
that a root-owned file under the wrapper's own name runs Git as root for a mistyped mode,
and an incomplete change list.
Asked first,
the owner answered "Lift it for the ZED hook only (Recommended)"
(`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`):
only the zedlet's file name selects a root mode,
and units,
adoption and a person use words.
The revision to that answer and the review merged as a proposal,
`1e5f417430`:
the zedlet's file name or the words `monochromatic-git-policy-cli zfs-root` select a root mode,
probed over nineteen fixture starts with the guard predicates replayed,
and six choices left open (the group word,
what root's install holds,
where the dispatch sits,
what `--version` says,
the account's short name,
and whether release keeps the modes' test-only variables),
each ranked.
Its cold first-start read ran on the host with each file's pages evicted
(the wrapper 6.04 ms slower than the handler at the median),
because mvm's Fedora guest refused every file route
(`keep/state-so-far.md` in `~/temp/agent/zfs-root-modes-after-the-start-name-answer-2026-11-24/`).
Its Sonnet re-read
(`re-read-of-the-revised-one-program-design.md` in `~/temp/agent/gate-review-zfs-root-modes-revision-2026-11-24/keep/`)
found it not ready:
under the recommended install a unit's words are selected by the zedlet's name,
which is not the answer's letter,
and the proposed guard admits a call site,
not one name;
it drafted a rule that keeps both the answer and a fail-closed install.
The second revision merged as a proposal,
`37143a1daf`:
words select every mode on any file,
the zedlet's name selects only ZED's no-argument start,
nothing at the root path reaches Git (probed on 28 starts),
the guards hold every variant the re-read planted,
and a disk-cold start in a guest costs the wrapper 1.87 times the handler (+6.06 ms).
Running:
a Sonnet read of that revision's diff
(`~/temp/agent/gate-review-zfs-root-modes-second-revision-2026-11-24/`).
mvm's Fedora template now starts the guest agent unconfined with SELinux still enforcing
(`8ca430dd2f`;
`doc/troubleshooting/mvm-fedora-guest-confined-by-selinux.md`).
mvm was rebuilt from `main` at 2026-11-24T02:19:46-05:00 and the cached `template-fedora.qcow2` deleted,
so the next Fedora guest bakes the fixed template.
Open:
a template cache that never reuses a path,
so a re-bake cannot corrupt a running guest's backing file.

Next:
start the second revision when the stall eases,
then put its choices to the owner,
then a build round removes the second `[[bin]]` and wires the modes.

#### Accepted designs that wait for their build steps

- The forced removal and the cancelled or interrupted removal,
  for plan step 17,
  accepted on 2026-11-23 with V10 answered "Stop that unit (Recommended)"
  (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`;
  the forced design carries the answer since `3464f1b4ee`,
  and its edits of that round are read by no reviewer yet).
  Tier 2 and the wait's length need step 17's code.
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

#### Later steps' pieces built ahead, unwired

Outcome:
pieces of later steps that need only work on `main` are merged as code no person's command reaches,
so their steps wait only for their wiring.

State:
built one round each,
under `rules-for-a-build-ahead-round-from-2026-11-23.md` in
`~/temp/agent/zfs-open-item-design-rounds-launch-2026-11-22/`,
and merged:
step 15's deferred-origin check (`54c10a0435`),
step 16's one-request snapshot delete (`ca9d947b71`),
step 13's managed lock distinction (`915aa6638b`)
and step 18's `doctor` checks for steps 2 to 9 (`fcfda032e3`).
Each round's report under its folder's `keep/` says what the wiring step should know.
Step 12's move refusal and unmounted report merged too (`3ea4fa16c3`).
Step 19's two open design parts merged as a proposal,
`9ab8cd0055`
(`doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`:
`zfs-mount.service` mounts the checkout on this host,
with the ordering and its failure control shown in a guest over unattended reboots).
Its Sonnet gate review
(`review-of-the-step-19-design.md` in
`~/temp/agent/gate-review-zfs-step-19-checkout-boot-mount-provider-2026-11-24/keep/`)
found it not ready:
the ZED hook is a `.sh` script,
the option the owner declined;
the failure control ran without the configuration it recommends,
and the provider's failing status came from the nested dataset;
three boots' logs were overwritten,
so their quoted lines rest on the journal or on nothing;
and `After=zfs-mount.service` carries a provider restart to the gate,
which two accepted designs rely on.
Its revision merged as a proposal,
`eeb76c96bf`:
the hook is the root-owned wrapper file under a ZED-legal name with no argument,
the failure arm ran with `canmount=noauto`,
every boot kept its own log,
`RequiresMountsFor=` is the first V3 option,
and a start bound (V8) and a unit per nested dataset (V9) are new;
its arms ran on systemd 258 in a Fedora guest set permissive,
so no result is about an enforcing boot.
Its Sonnet re-read
(`re-read-of-the-revised-step-19-design.md` in `~/temp/agent/gate-review-zfs-step-19-revision-2026-11-24/keep/`)
found V9's unit running a mode no grammar holds,
V8's timeout evidence cited to logs that do not hold it,
the proposed unit never booted as written,
and the ZED hook not allowed by this host's `zed` AppArmor profile.
The third revision merged as `ae0a9ea7ee`;
its Sonnet read (`read-of-the-third-step-19-revision.md` in `~/temp/agent/gate-review-zfs-step-19-third-revision-2026-11-24/keep/`)
found no high, and nine mediums, among them V9-B not existing on `main` either and the AppArmor section's need misstated.
The fourth revision merged as `ba7ade182a`,
with the unit booted as written, a flag-on hazard arm that fails closed, and this host's unit without key lines;
it ranks a dedicated `px` profile for the ZED hook above rules in `local/zed`,
which must be put to the owner beside the owner's earlier answer for rules over a dedicated profile.
The owner answered every choice:
`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`.
The chosen design merged as `1ef4afefa8`,
every unit text booted in a guest:
under `Wants=` a provider restart leaves the nested unit's invocation unchanged,
and the nested unit skips itself when its dataset is already mounted (`ConditionPathIsMountPoint=!`).
Step 19 waits on steps 12, 14 to 16 and 18 and on the root modes;
the agent runs its migration with sudo when no process has the checkout open
(`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`).

Next:
the re-read,
then put the design to the owner.

### The clone rung serves every source no slower than `main`

Why:
the owner's bar for a worktree add,
restated on 2026-11-23:
a declined add costs what `main` paid for the same source.

State:
the alpha.45 release pass's clone rung findings and the owner's four answers
(`doc/decision/cli-git-clone-rung-four-answers-after-the-alpha-45-release-pass.md`)
merged as `5c84ab3368`:
the one-turn arm clones the entries it was handed (H2),
an untracked-only source takes the clean rule (D1),
the whole-tree limit sums only what the clone copies (D3),
and the equal-length earlier conversion is documented as a limit (D4).
The Btrfs round D2 asked for merged as `edfd13bd90`
(`doc/planning/cli-git-clone-rung-dirty-shapes-on-btrfs-measured.md`):
the clone was no slower than Git's checkout on every Btrfs shape,
so the record's condition to drop the dirty-tree limit there holds.
The Btrfs dirty limit is dropped as `ce3f2c8518`,
shown in a Btrfs guest red with the old choice and green with the new.

Next:
nothing until the next release's pass.


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
(`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
"Cause H").
Its only denials were `umount`'s `dac_override` (no test needs it)
and the browser's.
So AppArmor enforcement costs `main` no test now.
The gate on `main` at `0576c39994`
(run `native-TtGef4`,
2026-11-24T01:57:53Z to 02:51:53Z,
beside six guests and a dozen rounds)
gave `10219 tests run: 10215 passed (10 slow), 4 failed, 37 skipped`:
the cause-H pair,
the size read's child control
(the clone rung pass's L1,
fixed after that tip in `5c84ab3368`:
the kernel clears the parent-death signal on an AppArmor profile change at exec),
and issue 6399's repair control,
whose round is running.
Its denials showed `systemd-oomd` refused its cgroup reads,
fixed by the owner's "Add the read rule now (Recommended)"
(the account's "systemd-oomd reading the cgroup tree").
The gate on `main` at `8519c599d0`
(run `native-bNr3O6`, 2026-11-24T06:18:44Z to 06:46:59Z)
gave `10745 tests run: 10741 passed (7 slow), 4 failed, 41 skipped`:
the cause-H pair;
the source guard `the_release_code_keeps_blocking_work_off_the_runtime`,
on two `std::thread::sleep` calls step 10 brought (`add_mount.rs:70`, `add_route_plan.rs:265`);
and the `statmount` control,
whose call answers `ENOSYS` inside the rootless container though the host kernel has it:
podman's default seccomp profile lists neither `statmount` nor `listmount`
(`doc/troubleshooting/podman-default-seccomp-blocks-statmount-in-a-container.md`);
fixed as `ada7a937b0`,
the gate deriving its profile from the host's with those two calls added
and the controls proving the fallback under a real seccomp refusal.
The end-to-end push program controls passed in about 5 ms each,
so they declined there too:
the gate image now holds a real `git-lfs`.
Fixed as `09f58f1470`:
every list the controls hand a push names the fixture's own folders,
so a real `git-lfs` in `/usr/bin` no longer turns the red control's refusal into a push that succeeds,
the decline is gone,
and the restart control acts on the service the commands reach.
Its new denials (`git` asking `dac_override`,
`git//ssh` running `/work/stubs/askpass`,
`ssh-agent` asking `setgid` and making sockets,
`ssh` running `true`, `false`, `cat` and `echo`)
map to no failure.
Their trace (the account's "Ninth batch: the gate at 8519c599d0", merged as `68c8dcaa21`)
put most on the L4 design round's own probe containers,
left the 28 capability records unattributed with no grant,
and proposed one rule for `systemd-analyze`'s working folder;
applied with the two read rules a run showed it needs,
the daily-update timer's `systemd-analyze --user verify` now exits 0.
It also named a blind spot,
the ssh-terminal and askpass controls passing when their askpass program is refused,
fixed as `2cd655453e`.
The gate on `main` at `f3b710cbc9`
(run `native-MgA4LN`, 2026-11-24T07:45:49Z to 08:09:44Z)
gave `10809 tests run: 10807 passed (6 slow), 2 failed, 41 skipped`:
the cause-H pair alone,
so the blocking-sleep guard,
the `statmount` control and Clippy all hold on `main`.
Its window's denials were `git` asking `dac_override` (88) and `umount` the same (3),
with no test failing;
the ninth batch declines that grant,
and a run of one named control with the audit log read beside it is what would attribute them.
Waiting for the memory stall to ease,
with prompts ready in their folders under `~/temp/agent/`:
all three merged since
(`a80675a4af`, `ada7a937b0` and `09f58f1470`).
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

#### The wrapper's `0.6.1-alpha.45` and what its release passes found

Outcome:
the release passes' findings are fixed for the next alpha.

State:
`monochromatic-git-policy-cli-v0.6.1-alpha.45` is published
(release run 45308756772,
publish run 45308886552,
archives for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`;
an alpha is not published to crates.io).
It ships the second program `mount-git-worktree-clone-datasets`,
by the owner's "Let it finish (Recommended)";
the owner's "One program only,
ever" removes it from later releases
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`),
and `pi-zfs-root-modes-in-the-one-wrapper-executable-design` designs how the root modes join the wrapper.
The first dry run failed at `pnpm install --frozen-lockfile`
(`ERR_PNPM_LOCKFILE_CONFIG_MISMATCH`);
the fix merged as `7703a307ea`:
`toolchain:update:daily` now ends with that frozen install,
and the lock was regenerated with no version moved.
The commit that left the pair out of step,
`aead90ed51`,
came from a run that did not finish,
not from a misordered task.
A release note holds at most 200 characters,
and the dispatch refuses while `origin/main` lags the local `main`.
The second dry run passed (run 45308464804),
and the release was dispatched as run 45308756772 at `0576c39994`.
The ZFS route's release pass found it fit to ship,
with two medium findings on the handler's words and its `start-probe`'s missing root check
(`zfs-route-reach/keep/review.md` in
`~/temp/agent/release-pass-wrapper-alpha-45-2026-11-23/`).
For its `start-probe` the owner chose "Refuse non-root,
exit 2 (Recommended)";
its `--help` paths are left to the fold.
The commit,
push,
LFS and lock pass (`commit-push-lfs-lock/keep/review.md` beside the ZFS one)
found nothing that loses work;
its medium finding is that the 22 controls using the real `git-lfs` skip in the gate,
whose image has none,
and its Decision 1,
`git-lfs` in the image with a skip made a failure there,
is adopted with the owner's veto open.
The fixes merged:
the ZFS pass's handler words and checks as `08b87aa69b`,
the commit pass's M1,
L5,
L1 and L3 as `075241a8b2`,
and the clone rung pass's as `5c84ab3368`.
The commit pass's L2 is answered,
"Show any segment under 20 chars (Recommended)"
(`doc/decision/memory-settings-persist-and-lfs-url-mask-shows-short-segments.md`),
and merged as `51e90be69e`,
with the build's narrower reading (a short segment with `%` or other punctuation still hides) recorded there,
veto open.
Its L4 waits in `doc/handover/questions-for-the-owner.md`,
and its options merged as a proposal,
`41bdf36a2a`
(`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`,
ranked B > D > C > E > A, B being the wrapper as the askpass program).
Its gate review
(`review-of-the-host-key-next-step-design.md` in `~/temp/agent/gate-review-typed-network-command-next-step-2026-11-24/keep/`)
found B's transcripts composed, never run,
its passphrase sentence wrong when ssh goes on to a key that works,
and B needing AppArmor rules in two stacked profiles per build,
and added an option J:
the sentence itself as `SSH_ASKPASS`'s value,
which OpenSSH prints in its exec failure line.
Its revision merged as a proposal,
`082c3ff8eb`,
ranked B > C > J > D > C2 > E > A with a run transcript for every option,
B run live through a stand-in of its contract in an unconfined container,
and J shown silent under `-q`.
Its Sonnet read (`read-of-the-revised-host-key-design.md` in `~/temp/agent/gate-review-typed-network-command-revision-2026-11-24/keep/`)
found B's printed transcript not the run's,
and B and E meeting the owner's self-start rule in `doc/planning/cli-git-no-self-invocation.md`,
which is the owner's call.
The third revision merged as `077c006094`,
ranked both ways: rule read strictly `J > C > C2 > D > A`, exception granted `B > J > C > E > C2 > D > A`.
A Sonnet read of it writes the owner's question (`~/temp/agent/gate-review-typed-network-command-third-revision-2026-11-24/`).
The LFS round found `push_programs_held_in_memory::a_push_a_restarted_service_has_to_try_again_stops_and_names_the_command`
failing 4 of 8 on the host,
not its regression:
issue 6447,
fixed as `0127911061` and closed:
the control stopped a second fixture service no command reached.
The same fixture pattern in `binary_push_programs_end_to_end_tests.rs` is fixed as `09f58f1470`.

The next alpha waits for the one-program build round:
the record says "alpha.45 shipped with the second program,"
and "the change starts with the release after it"
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`),
so a release cut before the second `[[bin]]` is gone would ship it again.

Next:
merge each round when it ends;
put L4's options to the owner;
release the next alpha once the one-program build has merged.

#### Markdown line breaks in the files changed on 2026-11-23

Outcome:
every Markdown file changed that day passes `monochromatic-lint`.

State:
`markdown/semantic-line-breaks` reported findings in 117 of the 169 Markdown files changed on 2026-11-23,
mostly where the old handover's entries were moved verbatim with that file's wrapping
(the cull's apply script ran no linter).
The linter's whitespace-only `--fix` is committed for 108 files,
each equal to its committed version with whitespace collapsed,
and nine records' moved headings were raised to one level below their section;
both scripts are in `~/temp/agent/lint-line-breaks-of-the-2026-11-23-markdown/`.
Left out:
the files that unmerged branches change
(among them `doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md` with 580 findings and
`package/pi-shared/subagent-driver/README.md`),
everything under `package/git-policy/cli/`,
and step 19's proposal,
whose fix changed more than whitespace
(kept as `two-files-left-out.patch` there)
and which its gate review is reading by line number.
The other such file,
`doc/troubleshooting/cachyos-apparmor-missing-dbus-profile-boot-failure.md`,
is fixed:
its one change beyond whitespace was a blockquote marker on a split quotation,
and `pandoc` renders it to the same HTML.
`monochromatic-lint` is not on the main checkout's `PATH`,
so `mise run //:lint:monochromatic-lint` fails with "command not found";
the binary is under `~/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-lint/`.

Next:
when those branches have merged,
run `fix-whitespace-only-line-breaks.ts` again for the rest,
and fix step 19's proposal once its review has ended.

#### Tests that failed once and are not diagnosed

Issues 6348 to 6352 on GitHub hold them,
each with the journal text that was kept.
Nobody is working on them.
Issue 6399's repair control is fixed as `d1af169091` and the issue closed:
a test-only seam waits for the fixture's first write,
so the scene is forced rather than raced.
Its round found issue 6448:
a control that makes a non-UTF-8 folder name fails on this host,
whose home dataset is `utf8only`;
fixed as `06d3e542b8`:
the 88 such controls run in a user and mount namespace with a tmpfs root when their own root refuses the name.

### The desktop stays usable under agent runs

Why:
the desktop froze for minutes on 2026-11-22 under this session's runs.

State:
two kernel settings,
`vm.watermark_scale_factor` 125 and `lru_gen` `min_ttl_ms` 1000,
are in force and,
by the owner's "Install both files (Recommended)" of 2026-11-24,
kept across reboots by `/etc/sysctl.d/90-vm-watermark-scale-factor.conf` and `/etc/tmpfiles.d/lru-gen-min-ttl.conf`
(`doc/decision/memory-settings-persist-and-lfs-url-mask-shows-short-segments.md`).
No limit on runs was set
(`doc/handover/agents-at-once-on-this-machine.md`,
"The freeze of 2026-11-22 set no limit on runs").
The first read under a load of the freeze's shape,
six guests,
a whole gate and Rust builds,
held with memory `full` at 5.99% and no blocked task,
while direct reclaim still did 82% of the scanning
(`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
"The comparison that would verify the change").
No arm without the settings has run,
so their effect is still not shown.
`systemd-oomd` was blind under AppArmor until 2026-11-24
and reads the cgroups again.

Next:
nothing is owed.
After the next reboot,
read both values once to confirm the files applied them.

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
