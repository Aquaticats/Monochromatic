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
`drive-pi-subagent.ts` itself still starts a run on a closed pool that a caller names;
the round `pi-driver-refuses-a-run-on-a-closed-pool`
made it and the worktree launcher refuse one,
and reopened the qwen plan's DeepSeek pool for runs that name it,
on `fix/pi-shared-subagent-driver-refuses-a-run-on-a-closed-pool` at `c1c1c1a1f8`
(report:
`~/temp/agent/pi-driver-refuses-a-run-on-a-closed-pool-2026-11-23/keep/report-pi-driver-refuses-a-run-on-a-closed-pool.md`).
Its whole unit suite failed a different real-process test on each of three runs,
each passing alone,
and its `lint` task cannot run in a fresh worktree (issue 6339).
Running:
its Sonnet integration pass,
to `~/temp/agent/integration-pass-pi-driver-refuses-a-closed-pool-2026-11-23/keep/review.md`.
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

Units a pass started,
seen running at that reading and its own to stop:
`ipass-holder-scan-plants-4`,
`declined-step-pass-loadlog`,
`recheck-f4-gate-2-20261123`,
`ssc2-6344-followups-gate-20261123`,
and the scope `ipnc2-drivers`.
After a restart of Claude Code alone,
a gate unit whose pass is gone is left to end,
and its containers and image tag are then cleared as the gate rules say.

### Results that are not acted on yet

- Issue 6342's follow-up round ended by itself after 109 turns.
  Its report,
  `keep/report-pi-lock-control-second-pass-lows.md` under
  `~/temp/agent/lock-control-freed-name-pid-file-failed-once-in-a-gate-2026-11-22/`,
  and its diff are unread,
  and its Sonnet pass is not started.
- The binding record's round on both seventh reads ended by itself after 62 turns,
  at `86ce40be3c`.
  Its report,
  `keep/report-seventh-reads.md` in its round folder,
  and its diff are unread.
- The guest round on the forced removal's first probe arms ended.
  Its report and its results are unread.
- The origin cleanup record and the plan record have lows from the record reviewer's seventh read,
  and no round on them is started.
- Two sorters' ledgers are complete and neither is applied
  ("The handover holds only open work").

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
Running:
`pi-zfs-no-configuration-second-pass-findings`,
on B1,
B2,
B4 and the lows,
with that answer in its inbox.

Next:
read its diff;
then merge.

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
Running:
`pi-zfs-step-8-sweep-second-pass-findings`,
on that and P2-M1 to P2-M3 and the lows.

Next:
a Sonnet pass on its diff;
then merge.

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
branch `feat/git-policy-cli-zfs-step-9-unmount-staging-and-reattach` at `59f2bd1876`.
Its integration pass said not fit
(`~/temp/agent/integration-pass-zfs-step-9-unmount-staging-and-reattach-2026-11-22/keep/review.md`):
the reopen of the staged folder by its handle fails on a real kernel,
no recovery after a kill is wired,
and two mount calls act on a mount the handler did not prove.
All of it sits behind the pin plan step 19 writes,
so none reaches a user.
The owner chose that a stopped unmount files an answer in an extended format
(`doc/decision/cli-git-zfs-staging-under-cache.md`,
"The answer a stopped unmount files,
answered 2026-11-23").
Running:
`pi-zfs-step-9-unmount-after-the-machine-restarted`,
the same round resumed from its session after the restarts ended it at 05:42.

Next:
a second Sonnet pass.

#### Step 10's designs: the add-time lock and the binding record

Outcome:
the owner accepts the two designs step 10 is built from.

State:
the lock design is on `docs/planning-cli-git-zfs-add-time-git-lock-design` at `4625537ae5`,
the binding record on `docs/planning-cli-git-zfs-add-binding-record-design` at `c077bdb1e1`,
each after a round on the reviewer's sixth read.
The reviewer's sixth read
(`review-sixth-read-before-acceptance.md` in
`~/temp/agent/gate-review-zfs-add-time-git-lock-and-binding-record-designs-2026-11-22/keep/`)
found the lock design fit with three lows,
and the binding fit after fixes its round has since made:
a stale test paragraph,
and a hazard in the settling command,
which on a `settling` binding could remove the registration of a worktree Git had already checked out.
That file holds both veto lists in plain words and the two plan sentences as they would read after the change.
The same reviewer's seventh read,
`review-seventh-read-before-acceptance.md` beside it,
found the lock design fit to put to the owner as it stands,
with two lows that can ride with its next commit,
and plan step 10 buildable from the two documents and the accepted records.
It found the binding fit after three mediums,
all in the settling command and none of them step 10's:
its two modes overlap on a half-made checkout;
a settling command that was killed and is run again once the pool answers has no command the text shows to work;
and the third option of the veto entry V23 sends a person in a circle.
The record reviewer's seventh read found the same overlap from its side,
and one more state with no command:
a clone destroyed by hand before any removal ran.
The binding's round on both reads ended at `86ce40be3c`
(`continuation-seventh-reads.md` in its round folder is what it was asked);
its report,
`keep/report-seventh-reads.md`,
says all eight items are done,
H1 and H2 among them.
Both reviewers' eighth reads
(`review-eighth-read-before-acceptance.md` beside each seventh read;
the lock reviewer is agent `a162ea896d0a950e2`,
the record reviewer `a6207db50e3431e04`)
found the lock design fit to put to the owner as it stands,
and the binding fit after one narrow round:
a `settling` binding with no plan is told two different commands
(the lock reviewer's U26,
the record reviewer's J1),
with lows and the unrecorded clone at `snapshotted` (U27).
The round on both eighth reads ended at `abeb058f63`,
its report saying every item done,
with one section,
"Which command is true for a state",
that the other rules cite.
Both ninth reads
(`review-ninth-read-before-acceptance.md` beside each eighth read)
found the lock design fit as it stands,
and the binding not yet:
its new section said a `settling` binding never has a plan,
which is false,
so a plan a killed removal left could end with no clone,
no binding and no command
(the record reviewer's K5,
blocking),
and the removal's rule had lost its scope
(U34 and K2).
The round on both ended at `64b0ce5a36`,
every item done by its report (`keep/report-ninth-reads.md`).
It adds one rule:
the add refuses while a plan stands.
Running:
each reviewer's tenth read of that diff
(`review-tenth-read-before-acceptance.md` beside each ninth read),
with the acceptance material updated to it.

Next:
the acceptance questions,
step 10's two designs and the three records.
It names what the owner accepts for step 10,
and puts one sentence of the plan to the owner:
step 10's "discarded copied Git administration",
which the accepted dirty-main design keeps and the binding calls wrong.

#### The removal-side records: the destruction plan and the origin cleanup record

Outcome:
the owner accepts the records a removal persists,
together with the binding record they share states with.

State:
the plan record is on `docs/planning-cli-git-zfs-removal-destruction-plan-record-design` at `28c590b4c4`,
and the origin cleanup record on `docs/planning-cli-git-zfs-origin-cleanup-record-design` at `77410b209a`,
each after a round on the reviewer's sixth read.
The reviewer's sixth read
(`review-sixth-read-before-acceptance.md` in
`~/temp/agent/gate-review-zfs-removal-side-record-designs-2026-11-22/keep/`)
found each fit after listed fixes and the set not yet ready,
for two things in the binding its round has since been given.
It holds the three veto lists in plain words.
The same reviewer's seventh read,
`review-seventh-read-before-acceptance.md` beside it,
found the two records fit after lows,
and the set not ready for two things in the binding
("Step 10's designs").
Its check 6 lists fourteen sentences of accepted designs that the three documents leave untrue,
each with the words that should replace it.
The rounds on H9 and H10 ended with every low done:
the origin cleanup record at `ea271e9e1d`,
the plan record at `db69df972a`.
The record reviewer's eighth read found the three fit with lows and the set ready to go to the owner with its list of accepted-design sentences
(now its check 4,
with entries 2,
3,
5,
12 and 13 changed),
once the binding's J1 is mended.
Its ninth read retracted that set verdict on K5
("Step 10's designs"),
and found the two records fit as they stand once the binding takes K5's fix.

Next:
its tenth read
("Step 10's designs");
then the acceptance question for the three,
with that list.

The plan record's one rule for a kept plan with an `unknown` outcome leaves sentences of two accepted designs untrue:
four in `doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
and the sentences on A27 and on what a removal names in
`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`.
The review gives the replacement text for each.
An edit to an accepted design is the owner's to allow,
so these go to the owner with the acceptance question.

#### Step 17's designs: forced removal, and a cancelled or interrupted removal

Outcome:
the owner accepts the two designs,
and confirms or changes the direction to unmount inside a holder's mount namespace.

State:
the forced arm is on `docs/planning-cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design` at `b62601213f`,
and the cancelled removal on
`docs/planning-cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design` at `c8cd85501c`,
each after a round on the reviewer's seventh read.
The reviewer's seventh read
(`review-seventh-read.md` in
`~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/`)
found each fit after listed fixes and nothing high.
Of the forced arm's first group of probe arms it found two ready to be scripted as written:
no arm starts in a fresh private mount namespace,
and none builds the peer group its rows assume.
It holds both veto lists and the confirmation question's wording.
The forced arm's round gave every arm a common start,
the mode of `unshare` it uses and a positive control;
the coordinator read that section,
and no review follows it,
because the guest round is its test.
The cancelled removal's round closed its one medium
(it no longer refuses a kept receiver the forced arm does not refuse);
the coordinator read its diff.
The guest round `pi-zfs-forced-removal-first-probe-arms-in-a-guest` scripted and ran Tier 0 from the design at `b62601213f`
on kernel 6.8 with hand-made mounts,
and wrote the results as a section of
`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`
on `docs/planning-cli-git-zfs-forced-removal-first-probe-arms-measured`
(folder `~/temp/agent/zfs-forced-removal-first-probe-arms-in-a-guest-2026-11-23/`).
The design's reviewer read it against the logs
(`review-guest-probe-arms.md` in that review's `keep/`):
every result follows from its log,
every arm started fresh,
and the direction holds there,
with five mediums:
way 3's unmount half shows nothing,
one variant of T0-3 shows nothing of its own,
no arm starts from the production shape where the host mounts the clone,
T0-5c gives two answers to whether way 8 is taken,
and way 7 end to end never ran.
Nothing is shown on kernel 7.3 or on ZFS.
The design places the confirmation after Tier 0,
Tier 1 (ZFS, no route) and Tier 2 (step 17's code);
the reviewer drafted the question with every slot the guest could not fill left empty.
Running:
`pi-zfs-forced-removal-probe-arms-gaps-kernel-7-and-tier-1`,
on the mediums' arms,
Tier 0 on kernel 7.3 (a guest of 6.19 or newer, or a rootless container on this machine's kernel),
and Tier 1;
and the design writer's round on the text edits the read asks for,
G4's policy among them
(`continuation-guest-read-edits.md` in `~/temp/agent/zfs-forced-removal-holders-ended-and-caller-spared-design-2026-11-22/`).

Next:
a read of both against the logs;
then the confirmation question for the direction,
since Tier 2 needs step 17's code;
then the acceptance question.

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

#### The partial clone of a dirty source

Outcome:
a source with uncommitted changes is cloned,
and Git writes only the changed files.

State:
branch `feat/git-policy-cli-clone-rung-partial-clone-of-a-dirty-source` at `45499c3154`.
Its recheck said not fit for one finding
(`~/temp/agent/integration-recheck-clone-rung-partial-clone-2026-11-22/keep/review.md`).
The owner's four answers are in `doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`,
"Four further answers after the recheck,
answered 2026-11-23".
Running:
`pi-clone-rung-partial-clone-after-the-machine-restarted`,
the same round resumed from its session after the restarts ended it at 05:42.

Next:
a third Sonnet pass.

#### The size read beside the source check

Outcome:
rung 1 reads sizes without making a declined add slower.

State:
merged as `e37566df9c`.
Its follow-up is on `feat/git-policy-cli-fence-size-read-beside-source-check` at `9329642a95`.
Its Sonnet pass
(`~/temp/agent/integration-pass-fence-size-read-declined-add-step-cost-2026-11-22/keep/review.md`)
found it not fit:
the declined add's step cost is gone,
but the parse moved onto the adding thread,
so the `conversionFence` step of every served and dirty add is 7.8 to 13.4 ms longer than `main`'s.
Running:
`pi-fence-size-read-integration-pass-findings`,
which puts the parse back off the adding thread.

Next:
a second Sonnet pass.

#### The object reader's declined branches

Outcome:
each branch on which the merged object reader declines is held by a control.

State:
branch `fix/git-policy-cli-object-reader-declined-branches-controls-and-lows` at `daa0836b75`.
Its fourth Sonnet recheck
(`review-fourth-recheck.md` in
`~/temp/agent/integration-recheck-object-reader-the-commit-refusal-is-out-2026-11-22/keep/`)
found it fit after one fix:
the note's reason for a replacement base the command names is false.
The owner chose to record that case as a limit
(`doc/decision/cli-git-object-reader-named-replacement-base-is-a-recorded-limit.md`),
filed as issue 6396.
The round on that and the recheck's two lows ended at `cd6f2e1185`
(`~/temp/agent/object-reader-commit-memory-is-per-landing-2026-11-22/keep/report-pi-object-reader-fourth-recheck-findings.md`):
the false sentence corrected,
the limit recorded with issue 6396 in the note and the README,
the ladder target's open held by a control shown red and green,
and the words made to read alone.
Its host tests fail the same nine that `main` fails on the same filter.
Running:
one whole gate on its tip,
`gate-object-reader-declined-branches-20261123`,
its summary written to `~/temp/agent/apparmor-rules-for-the-wrapper-work-2026-11-23/keep/`.

Next:
read the gate against `main`'s baseline
("The tests and the gate under AppArmor");
then merge.
Issue 6347,
the one case the reader's refusal was right about,
is held work.

### The wrapper's test gate tells the truth

Why:
a merge rests on the whole container gate,
and a control that fails once in a gate for no fault of the product costs a round each time.
The gate's known causes are lettered in
`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`.
Two tests fail in every gate on this host,
the pair that document names under cause H.

#### Issue 6342: the lock control and the PID file

State:
merged as `1ba2fe767d`.
The merged code compares the number the PID file names,
and the owner chose that its contents be compared
(`doc/decision/cli-git-lock-cleanup-pid-file-contents-compared-before-unlink.md`).
The round that builds the byte comparison and folds in the second pass's lows has ended,
on `fix/git-policy-cli-lock-control-freed-name-pid-file-failed-once-in-a-gate` at `3ad5c8752a`.
Running:
its third Sonnet pass,
the second-pass reviewer resumed as agent `a53733389dde02725`,
to `review-third-pass.md` in its earlier pass's `keep/`.

Next:
merge,
or a fix round.

#### Issue 6344: the sibling shadow control

State:
merged as `0b7a48def4`.
The follow-up round on what its pass left ended on
`fix/git-policy-cli-shadow-sibling-control-two-timing-windows` at `48346ef6c2`,
all seven items done,
its gate (before enforcement) failing only the cause-H pair.
The guard costs about one second per gate,
and the owner kept it
(`doc/decision/cli-git-shadow-sibling-control-guard-costs-a-second-per-gate.md`).
Running:
its Sonnet pass on the follow-ups,
the integration pass's reviewer resumed as agent `a8d0bc7335c71001c`,
to `review-follow-ups.md` in that pass's `keep/`.

Next:
merge,
or a fix round.

#### Issue 6345: the worktree-removal control

State:
branch `fix/git-policy-cli-worktree-removal-held-commit-control-failed-in-two-gates` at `1b68b52819`.
Its second pass found the code sound and the wording wrong,
and one case the test wait misses
(`~/temp/agent/integration-second-pass-worktree-removal-held-commit-control-2026-11-22/keep/review.md`).
The owner chose a third look for the wait
(`doc/decision/cli-git-test-support-wait-for-an-idle-commit-service-takes-a-third-look.md`).
The round on both has ended:
the wait asks for three empty samples in the order a record moves,
with a control for each,
and causes N and G are rewritten.
Its gate had one failure that is on no list,
`fault_trace::a_merge_killed_at_each_pwrite64_leaves_a_recoverable_checkout`.

Its third Sonnet pass
(`~/temp/agent/integration-third-pass-worktree-removal-held-commit-control-2026-11-23/keep/review.md`)
found it not fit as it stands:
the fourth control passes on `main`'s wait too,
so it guards nothing,
and the troubleshooting text says things the runs contradict.
The wait and the fixture's drop are sound,
and its gate was void under the missing AppArmor rules.
That failure is filed as issue 6397,
and the pass drafted the comment for issue 6345 at the merge (its report,
`keep/report-from-the-agents-last-message.md` beside the review).
The round on the third pass's findings ended at `04b255e004`.
Running:
one whole gate on that tip,
`gate-worktree-removal-control-6345-20261123`,
its summary written to `~/temp/agent/apparmor-rules-for-the-wrapper-work-2026-11-23/keep/`.

Next:
read the gate against `main`'s baseline
("The tests and the gate under AppArmor");
then merge and post the comment.
The three product defects the pass confirmed are issues 6353,
6354 and 6355.
Issue 6354 is decided
(`doc/decision/cli-git-worktree-removal-not-woken-warning-names-the-way-out.md`)
and edits the same documents,
so it is built on a branch made from this one's tip,
`fix/git-policy-cli-worktree-removal-not-woken-warning-names-the-way-out`,
by the round `pi-worktree-removal-not-woken-warning-names-the-way-out`,
and merged after it.

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
Gates were released at 2026-11-23T18:20-05:00,
with those ten as `main`'s baseline that counts against no branch
(`note-gates-may-run-against-mains-baseline.md` in the rounds' launch folder,
sent to every running round and pass).
A gate runs on a branch through `run-one-whole-gate-in-a-worktree.ts` in the AppArmor folder,
which writes the summary and failures to its `keep/`,
clears the run's containers by label and removes the tag.
The kernel's audit backlog dropped 69,301 records at its default of 64;
the owner chose 8192,
kept across reboots,
which is set for this boot and written to `/etc/audit/rules.d/10-backlog-limit.rules`.
That file is loaded at boot only once the owner runs `sudo systemctl enable audit-rules.service`.

Next:
read each branch gate's denials as it ends,
and add rules for what it still meets;
find why the three `stale_ref_lock` controls fail under enforcement,
by one of them run alone in the gate's image;
then a whole gate on `main` again to shrink the baseline.

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

Next:
the same counters under a load like the freeze's,
which cannot be made on purpose and has to be caught;
then the persistent form of the two settings,
for the owner to apply;
and one gate's fixture file system memory,
measured.

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
none.

Two sorters have ended,
each with a ledger that covers its range exactly once and with issue drafts,
under `sorter-lines-<first>-to-<last>/keep/` in that folder:

- lines 44,768 to 46,713,
  in 307 items.
  The coordinator's check script passed its coverage,
  and one stretch read by hand agreed with it.
  Its `delete` items are not applied until each quotation inside them that stands nowhere else is moved or shown to be held.
- lines 46,714 to 49,499,
  in 151 items,
  after the same lesson was sent to it:
  it moved the owner's sentences and answer labels out of `delete`.
  Four of its items are private and get no target.
  Not yet checked by the coordinator.

Each sorter's report came as its last message,
with contradictions between the old text and current documents and proposals for rule text.
The second sorter's is also in its `keep/state-so-far.md`;
the first sorter's is only in the session's record until it is copied into its `keep/`.

The sorters of lines 41,470 to 44,767
and of the open review findings,
the picks and the working rules
(40,844 to 41,469 and 49,500 to 49,707)
have ended too,
the second with 78 items covering its 834 lines exactly.

The check of quotations inside `delete` items first matched a passage's first forty characters;
a control set showed it wrong both ways,
the unsafe way being a sentence taken as held because another opens alike
(`keep/quotation-check-controls-2026-11-23.md`).
Its replacement,
`list-whole-quotations-a-ledger-would-delete.ts`,
matches whole passages with whitespace read as one space,
and got all three control items right.
Over the four ended ledgers it finds 208 quoted passages in `delete` items,
159 of them held nowhere else:
76 in lines 41,470 to 44,767,
65 in 44,768 to 46,713,
17 in 46,714 to 49,499,
and one in the findings,
picks and working rules.

Running:
the sorter of the open work (34,867 to 40,843),
briefed by `brief-sorter-open-work.md` in that folder;
and a Sonnet audit that tells each of the 159 as the owner's words with a target record,
held in another form,
or not the owner's
(`brief-quotations-the-ended-ledgers-would-delete.md`).

Next:
apply that audit's `owner` moves to their records,
then the ledgers by script;
the ranges in this order:
the owner's decisions (lines 41,470 to 49,499 of the old file),
the open review findings and the picks made under delegation (40,844 to 41,469),
the open work (34,867 to 40,843),
the working rules (49,500 to 49,707),
and the run logs last (153 to 34,866).
A sorter gives each range a disposition with its evidence and moves nothing;
a script then moves text verbatim,
so that the owner's words pass through no model.

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

About thirty wrapper and driver branches that wait for a review are listed in the old file under "Open work";
that section is not sorted yet.

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
