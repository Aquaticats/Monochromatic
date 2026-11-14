# Agents at once on this machine

Every session on this machine reads this before it starts a Pi run or a subagent.
It holds the number that is allowed now,
how to count,
and the search for the number the drives carry.

## The cap now

At most **24** agents run on this machine at once since 2026-11-14T08:54:33-05:00,
by the owner's answer of that morning
("Resume at 24":
halve the gap between 16, which held, and 32 again,
with the probe ending the level on two slow readings in a row as before;
`doc/handover/cli-git-rust-implementation.md`,
"Answers on the owner's return").
Should twenty-four fail,
the rule gives twenty.
The filling unit is `fill-free-places-and-put-ended-reviews-back-at-level-24-20261114T085433`
(eight hours,
no successor)
and the probe a unit named `disk-ceiling-probe-level-24-` with the second it was started in.
What stood before,
and why:
at most 16 agents ran at once,
counted across every session
(raised from 4 to 8 on 2026-11-13 at 11:57,
from 8 to 16 at 12:58:50
and from 16 to 32 at 17:12,
each time when the level before had held its hour;
lowered from 32 to 24 on 2026-11-14 at 01:16:46,
from 24 to 20 at 01:21:35,
from 20 to 18 at 01:42:18,
from 18 to 17 at 02:03:15
and from 17 to 16 at 02:48:16,
each time when the level before failed its criterion,
by the rule under "What counts as the ceiling":
halve the gap between the last level that held, sixteen, and the one that failed).
Twenty-four stopped no start:
the provider pools take about twenty-five at most,
and twenty to twenty-two agents were going at both failures.
The levels from twenty down do:
nothing new starts until fewer are going,
and no run that is going is stopped.
**The search has ended at sixteen**,
the last level that held,
and goes no lower without the owner.
Level seventeen failed at 02:47:00 on 2026-11-14;
the probe runs on for readings only and ends on none of them.
**The search stopped measuring the count before it ended.**
The failing pairs at eighteen and seventeen came with sixteen and seventeen agents going,
at or one above the count of the level that held,
and after the daily update had ended.
Whether sixteen is right,
or the number goes back up once the stalls are explained,
is a question kept for the owner in `doc/handover/cli-git-rust-implementation.md`.
The readings and what is not established about their cause are under
"Levels 32 and 24 fail on the home drive, 2026-11-14T00:59:04-05:00 to 01:18:45-05:00",
"Level 20 fails on the home drive, 2026-11-14T01:37:37-05:00 to 01:41:00-05:00",
"Level 18 fails after the daily update ended; a snapshot cleanup has run since midnight, 2026-11-14T01:42:20-05:00 to 01:54:55-05:00" and
"Level 17 fails and the search ends at sixteen, 2026-11-14T02:03:18-05:00 to 02:47:00-05:00".
The branch `feat/pi-shared-subagent-driver-start-gate-holds-a-machine-wide-cap-on-agents-and-a-floor-for-the-runs-folder-drive`,
not in `main`,
carries the number in a file of its own that still reads sixteen;
the number is brought to this one when that branch is merged.

The owner,
2026-11-13,
after a full home drive ended twenty runs on 2026-11-12 and the machine stopped answering:
"you set your max concurrent pi+sonnet subagents starting from 4,
then bisecting up until we hit this disk's ceiling.
We'll re-find the ceiling once the new disk arrives."
Asked whether that counts one session's agents or all of them:
"Every agent on the machine".

The number is the level the search stands at,
a starting point that is raised,
and never a reason to stop work that is going.
When the session doing the search stopped a run of its own because another session's start had taken the machine to five,
the owner wrote:
"\"Forced you stop yours\"? Didn't I say bisect up until there's a problem?"
So:

- no run that is going is stopped to get back to the number;
- a count above the number with no symptom is a reading at that higher count,
  and is written down as one;
- the number only says when not to start one more yet.

## How to count before a start

1.  Count the Pi runs going,
    whoever started them:

    ```sh
    # prints one line for each agent unit that is going
    systemctl --user list-units --no-legend --plain --state=active 'pi-*.service'
    ```

    The pattern ends in `.service` because a run's own build scope may carry a name that starts with `pi-`
    (`pi-review-build-cli.scope` was counted as an agent on 2026-11-13 until 13:12).

2.  Add the subagents your own session has going that are no Pi run.
3.  Start one only when the sum is under the cap.

A batch list handed to `start-pi-runs-from-a-batch-list` starts every entry the provider pools have room for,
and the pools together take far more than the cap,
so hand it no more entries than there are free places.
The change that makes the launcher's start gate hold the cap itself is in work
(branch `feat/pi-shared-subagent-driver-start-gate-holds-a-machine-wide-cap-on-agents-and-a-floor-for-the-runs-folder-drive`);
when it is on `main`,
step 1 and step 3 are the gate's and this section says so.

## What counts as the ceiling

The owner's answer of 2026-11-13:
"Measured symptoms, but \"a commit through the wrapper takes longer than your rule\" is obviously a bad \"symptom\"
since that might just say the wrapper is bad."

A level fails on the first of:

- a Pi run gets no answer to its first prompt in 120 s,
  or ends on a disk error;
- a 4 KiB write-and-sync on either drive takes over 5 s twice in a row
  (the 5 s was offered in the question and not objected to).

A level is held at least an hour and until its runs have each built once.
The kernel's stall figures and the Git wrapper's commit time are written beside each reading and fail no level.
Only real queued work runs;
no load is made for the search.
Levels go four,
then double until one fails,
then halve the gap between the last that held and the one that failed.
The search is done again after the new drive is in
(expected 2026-11-17;
the owner said on 2026-11-14 at about 14:39 that the arrival slipped from 2026-11-15).

## Readings

Probe readings,
one line a minute:
`~/temp/agent/disk-ceiling-search-by-agent-count-2026-11-13/keep/probe-readings.jsonl`.

### Level 4, started 2026-11-13T10:43:42-05:00

- Before any run of the search: `/proc/pressure/io` `full avg60=12.45`,
  with one sizing walk on the home drive and one run of another session going.
- Started with one run of another session and three of the session doing the search.
  At 10:45:25 the probe counted five units:
  the other session had started a second run,
  which nothing had told it not to.
  The session doing the search stopped its newest run at 10:49:59 to bring the machine back to four,
  which was wrong (the owner's words under "The cap now"):
  five with no symptom was a reading to keep, not a breach.
  The stopped run had worked 5 minutes 25 seconds and was started again at 10:50:28,
  when a run of the other session had ended.
- Of the runs going,
  only `integration-pass-18-for-alpha-16` compiles Rust;
  the level is not judged before the places hold runs that each built.
- First readings (10:45:25 and 10:46:25, five units): write-and-sync 2 to 3 ms,
  `full avg60` 15.24 and 10.43.
  Both probe files of that first probe were on the build volume
  (`~/temp` is a link to `/var/mnt/encrypted/temp`),
  so the readings in the log from 10:44 to 10:56 labelled `homeMilliseconds` are build-volume readings.
- The probe was started again with its home file under `~/.cache`,
  which is on `/var/home`;
  its first reading is of 10:56:44 (four units,
  write-and-sync 1 ms on the home drive and 2 ms on the build volume,
  `full avg60` 4.42).
  The hour of the level counts from there.
- **Held.** Sixty readings from 10:56:44 to 11:55:48,
  fifty-seven of them with four units going and three with three.
  Write-and-sync on the home drive: median 1 ms,
  slowest 32 ms.
  On the build volume: median 2 ms,
  slowest 2,157 ms (11:44:46) and 850 ms (11:10:44),
  every other reading under 50 ms;
  the bound is 5,000 ms twice in a row.
  `full avg60`: median 4.2,
  highest 15.6.
- No run got no answer to its first prompt and none ended on a disk error:
  three runs ended `idle` with their reports in that hour
  (the third run of integration pass 18,
  which built and tested Rust,
  and the two launcher changes),
  and the two resumes and the review that took their places made turns.
- Both slow readings on the build volume fall within forty seconds after a run ended
  (11:10:20 and about 11:44:07).
  Whether a run's end (its worktree or its unit going) is what stalls the volume for a second or two is not established;
  two cases are no measurement.

### Level 8, started 2026-11-13T11:57:30-05:00

- Going when it started: two resumes of wrapper work,
  the review of the start gate's change,
  and one run of another session.
- Added: three more resumes of wrapper work and the review of the launcher's scratch-folder change.
- Probe: unit `disk-ceiling-probe-level-8-20261113`,
  the same file,
  one hour.
- Seen at this level and no symptom by the criterion:
  one run's lint waited 529 seconds on the shared cargo cache lock and never started
  (`fetch:rust-dependencies`,
  reported at 12:16).
  A lock held by another agent's build is a wait on a neighbour,
  not on a drive;
  it is written here because it grows with the count.
- **Held.** The reading after the 5,901 ms one,
  at 12:58:07 with eight units going,
  took 1 ms on the home drive:
  one reading over the bound and not two in a row,
  which is what the criterion asks for.
  The session doing the search had first written "held for a second hour";
  that extra hour was a precaution of its own with no failure behind it and was dropped after one reading
  (rule `HLD` in `AGENTS.md`).
  The 5.9 s stands recorded as the worst the home drive showed at eight.
- Sixty readings from 11:57:29 to 12:56:31,
  forty-six with eight units going,
  ten with seven,
  three with six and one with four.
  Build volume: median 7 ms,
  slowest 358 ms.
  Home drive: median 1 ms,
  and every reading under 100 ms but the last,
  **5,901 ms at 12:56:31** with seven units going.
  The bound is 5,000 ms twice in a row;
  the probe's hour ended on that reading,
  and the reading that followed it is in the bullet marked "Held".
  What was writing to the home drive in that minute is not established;
  two starts fell in the forty seconds before it
  (a resume at 12:55:20 that started nothing and a review at 12:56:03).
  `full avg60`: median 4.13,
  highest 9.1.
  One-minute load: median 11.2,
  highest 38.1.
- No run got no answer to its first prompt and none ended on a disk error in that hour.
  Nine runs ended with their reports
  (two of them marked `driver-failed` by issue 6295 after their agents had finished).
- One run ended at this level with `endReason: driver-failed` (12:29:40,
  a run of the other session).
  It is no disk symptom:
  its agent had given its report and settled,
  and the driver then failed to parse a result the feed had cut short
  (issue 6295).

### Level 16, started 2026-11-13T12:58:50-05:00

- Going when it started: eight runs of the session doing the search
  (resumes of wrapper work,
  two launcher authors answering reviews,
  reviews).
- Added between 12:58:51 and 13:01:21,
  twenty seconds apart:
  the reviews of the index-lock fix and of the commit that does not list every reference,
  and resumes of the runs
  `abort-and-skip-are-refused-with-an-opt-out-where-they-would-discard-work`,
  `discarding-commands-are-refused-with-an-opt-out-where-they-would-destroy-a-file`,
  `guards-refuse-with-an-opt-out-where-the-wrapper-saved-and-went-on`,
  `removal-goes-around-a-subvolume-or-mount-it-did-not-make`,
  `removal-refuses-a-worktree-a-process-works-in-and-logs-each-ignored-file-it-deletes`
  and `snapshot-of-a-person-is-never-proved-to-be-the-wrappers-own`.
  Sixteen units were going at 13:01:43.
- Probe: unit `disk-ceiling-probe-level-16-20261113`,
  the same file,
  one hour from its first reading at 12:58:48,
  which was taken with eight units going.
  Its readings at 12:59:48 and 13:00:53 were taken with eleven and fourteen units,
  so the hour holds fewer readings at sixteen than the hours of the levels before held at theirs.
- A sizing walk of the scratch folders on the build volume
  (`btrfs filesystem du`,
  disk weight 1)
  was started between 13:02 and 13:05 by the session doing the search and ended at 13:15:18;
  a slow build-volume reading in that time is read against it.
  It read 62.7 GiB as the scratch folders' own data
  (largest: `kimi-trials` 15.3 GiB,
  `pkgrepair-repair-off-the-path` 13.1 GiB,
  `module-test-logs` 9.8 GiB,
  `cli-git-btrfs-fixtures` 5.6 GiB),
  with some fixture files it could not read left out.
- A reading above the level: seventeen agents at 13:19:20.
  The starter of the session doing the search read fifteen three times in a row,
  because a run just started has no unit for some seconds,
  and started three reviews into one free place (13:17:49 to 13:18:35).
  Nothing was stopped;
  the starter now counts the free places once before its first start.
- A second sizing walk, of every worktree, was started at 13:16:06 at disk weight 1,
  and one of the shared cargo build folder shortly after.
- Readings to 13:33:18,
  thirty-five at this level:
  home drive slowest 144 ms;
  build volume over one second eight times
  (4,436 ms at 13:06:54,
  1,613 at 13:12:59,
  **5,676 at 13:15:01**,
  1,894 at 13:18:09,
  2,828 at 13:20:11,
  1,824 at 13:21:14,
  1,121 at 13:22:16,
  3,059 at 13:33:18),
  where the hour at eight had 358 ms as its slowest.
  One reading is over the bound and the one after it is not.
- **These readings cannot be put down to the agents alone**:
  every one of them falls while a sizing walk of the session doing the search ran on the build volume
  (the scratch walk to 13:15:18,
  the worktree walk from 13:16:06,
  and a `du` of the shared cargo build folder).
  Both walks still going were stopped at 13:33:48,
  the worktree walk having read 57 worktrees and the other nothing.
  The level is judged on readings taken after that,
  for an hour from there,
  and no sizing walk is run during a level's hour again.
- Readings after the walks stopped,
  13:34:21 to 13:58:48,
  twenty-five of them with ten to sixteen units going:
  home drive median 2 ms and slowest 22 ms;
  build volume median 10 ms,
  and over one second four times:
  1,261 ms at 13:36:21,
  2,597 at 13:38:22,
  **10,419 at 13:40:25**
  and **7,130 at 13:51:37**.
  Two readings over the bound,
  never two in a row,
  so the criterion is not met;
  the build volume stalls for seconds at this level all the same,
  which it did not at eight.
- The probe ended at 13:58:48 and none ran until 14:37,
  while the session doing the search waited on a question;
  one more was started at 14:37 for thirty-six minutes
  (unit `disk-ceiling-probe-level-16-after-the-walks-20261113`).
  From 14:12:41 the queue was empty and the count fell to nine by 14:38:24,
  so its first readings are below the level.
- Seen at this level and no symptom by the criterion:
  the user service manager (PID 5289) wrote no log line from about 14:48:10 to 14:49:00,
  where it wrote two to three every ten seconds before and eighteen in the ten seconds after;
  in that time two callers got no listing from it within twenty seconds
  (the probe's own count at 14:48:24, logged as -1,
  and a watcher of the session doing the search, which ended at 14:49:07).
  At 14:49:30 the same listing took 8 ms.
  **What it waited on is not established, and nothing measured points at a drive**:
  - the probe's two writes at 14:48:24, the start of that time, took 65 ms on the home drive and 9 ms on the build volume;
  - the one-minute load went from 13.0 to 60.7 in that minute on 28 processors,
    and the kernel's I/O figure `full avg10` read 23.75;
  - memory at 14:49:30: 968 MiB free,
    11.3 GiB in compressed swap,
    and the memory figure `full avg10` at 2.00;
  - the manager's whole wait for a processor since it started at 08:57:19 is 45.5 s
    (`/proc/5289/schedstat`),
    less than that one silence,
    so the silence was not spent waiting for a processor;
  - it has had 244 major page faults in that time and holds 3,108 kB in swap;
  - the kernel's accounting of block-I/O delay per process is off (`kernel.task_delayacct` is 0),
    and the manager's current system call cannot be read by this account,
    so what it slept in cannot be told afterwards.
  Three reviews ended at 14:49:16 to 14:49:18,
  each on the provider's answer "429 … Allocated quota exceeded",
  in the seconds after the silence ended,
  while seven other reviews on the same provider went on.
  A recorder now samples the manager while an answer is slow:
  its state, its time on and waiting for a processor, its page faults, the journal's state and the kernel's pressure figures
  (unit `record-what-the-user-service-manager-waits-on-20261113`,
  log `~/temp/agent/disk-ceiling-search-by-agent-count-2026-11-13/keep/slow-answers-of-the-user-service-manager.jsonl`).
  A manager that stops answering for a time is what `doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md` is about.
- Readings of the probe started at 14:37,
  14:37:21 to 15:13:14,
  thirty-six of them with eight to sixteen units going
  (sixteen once, at 14:57:08, and twelve to fourteen for most of it):
  home drive over the bound once,
  **5,756 ms at 15:00:08**,
  with 65 ms the next slowest;
  build volume over one second twice
  (1,240 ms at 14:42:22 and 1,321 at 14:44:23)
  and under 500 ms otherwise.
  One reading over the bound and the one after it 2 ms,
  so the criterion is not met.
  This is no symptom at the counts there were,
  and it is not sixteen held for an hour:
  the count stood at sixteen for one reading.
- From 15:18:02 the count is bounded by something other than the drives.
  Eighteen reviews ended that afternoon on the reviewer's provider answering "429 … Allocated quota exceeded",
  each with six to nine runs on the reviewer's model going,
  and the script that fills free places now starts no run on that model while five are going
  (`doc/handover/cli-git-rust-implementation.md`,
  the section dated 2026-11-13T15:24:13-05:00).
  So how many agents go now follows how many author runs the queue holds,
  and a reading at sixteen or above needs that many runs that are not reviews.
- Free room at 15:24:13: `/var/home` 513 GiB,
  the build volume 1,529 GiB:
  26 GiB less than at 14:37:39,
  in forty-seven minutes.
- Free room at 14:37:39: `/var/home` 514 GiB,
  the build volume 1,555 GiB:
  61 GiB less than at 13:15:18,
  in eighty-two minutes.
  What takes it is not measured;
  `/var/mnt/encrypted/cargo-build` holds 259 entries (14:38:24),
  and each review and each resumed run builds the wrapper in a tree of its own.
- Free room at 13:15:18: `/var/home` 517 GiB,
  the build volume 1,616 GiB
  (519 GiB and 1,640 GiB at 10:27:50,
  so the build volume took 24 GiB in two hours and forty-seven minutes;
  what took it is not measured,
  and it is not the scratch folders by this reading,
  whose own data read 63.5 GiB earlier in the day).
- Ten entries are left in the resume queue,
  one of them waiting for the branch it builds on.
  Whether thirty-two can be reached with real work is not settled by that count:
  reviews of the branches that finish add entries.
- **The hour that judges this level**:
  probe readings from 16:11:15 to 17:10:33,
  sixty of them,
  one a minute,
  with the queue holding enough author runs to keep the places filled.
  Agents going at each reading:
  thirteen once,
  fourteen at five readings,
  fifteen at eleven,
  sixteen at twenty-four
  and seventeen at nineteen.
  - Home drive: median 1 ms;
    one reading over one second and over the bound,
    **7,819 ms at 16:57:24**,
    the reading after it under the bound.
  - Build volume: median 7 ms;
    over one second twice
    (4,464 ms at 16:13:15 and 2,020 ms at 16:38:21),
    none over the bound.
  - Never two readings in a row over the bound on either drive.
  - Thirteen Pi runs were started in the hour
    (by the creation time of each run's feed),
    and each gave its first answer within the bound of 120 s:
    slowest 76,930 ms,
    four of them over 60 s
    (`~/temp/agent/disk-ceiling-search-by-agent-count-2026-11-13/keep/time-to-first-answer-of-each-pi-run-started-in-the-hour-of-level-16-2026-11-13T1611.txt`).
  - Thirteen runs ended in the hour
    (`list-runs-that-ended-since`),
    none on a disk error:
    eight settled,
    four ended `driver-failed` with Pi's exit code zero on the parse error of issue 6299,
    and one review ended on the provider's rate limit.
  - So the criterion is not met:
    sixteen held its hour.
  - No sizing walk ran during the hour.
  - Free room at 17:11:42: `/var/home` 507 GiB,
    the build volume 1,488 GiB
    (41 GiB less than at 15:24:13,
    in one hour and forty-seven minutes).

### Level 32, started 2026-11-13T17:12:00-05:00

- The unit that fills free places was stopped between two of its rounds and started again at the new level
  (`fill-free-places-and-put-ended-reviews-back-at-level-32-20261113T1712`):
  at most two starts every 150 s,
  and no more than five runs on the reviewer's model at once,
  which bounds how fast the count climbs and how far it gets with a queue that is mostly reviews.
- Going at 17:14:28: seventeen Pi runs,
  one of them another session's.
- A sizing walk of the shared cargo build folder
  (`btrfs filesystem du --summarize`,
  disk weight 1,
  unit `blocks-each-build-folder-holds-alone-20261113T1713`)
  was started at 17:13,
  after the hour of level sixteen and before this level's probe;
  the probe of this level
  (unit `disk-ceiling-probe-level-32-20261113`,
  the same readings file)
  starts when the walk ends,
  so no reading of this level's hour falls in the walk.
- The walk was still going at 17:21:02 with nothing printed,
  and no probe had read since 17:10:33 while the count rose.
  So a second probe reads while the walk runs,
  from 17:22,
  fifteen minutes at a time,
  renewed while the walk goes
  (unit `disk-ceiling-probe-level-32-while-the-build-folder-walk-runs-20261113T1722`,
  the same readings file,
  level field 32).
  Its readings are there to catch a symptom,
  and are read against the walk:
  they do not count toward the hour that judges the level.
- The hour counts from the first reading with the count above sixteen for good,
  and is read by the counts it holds:
  a count that stays near twenty is a reading at twenty,
  not thirty-two held.
- **The pools bound the count at about twenty-five, and the owner keeps it so.**
  A run that names no model goes to one open pool,
  `neuralwatt-flex`,
  which takes twenty,
  and reviews are held to five.
  Asked at about 17:56 through the question tool whether to open more room for author runs,
  the owner answered:
  "Keep it.
  I suspect any more would blow up the disk anyway."
  So thirty-two stays the number that says when not to start one more,
  the count does not pass about twenty-five,
  and the search goes no higher until a pool changes or the new drive is in.
- Readings from 17:21:32 to 18:06:06,
  forty-four of them,
  **every one taken while the sizing walk ran**:
  agents going twenty at two readings,
  twenty-one at one,
  twenty-two at four,
  twenty-three at four,
  twenty-four at nineteen
  and twenty-five at fourteen.
  - Home drive: slowest 42 ms.
  - Build volume: over one second eight times
    (1,503 ms at 17:26:32,
    2,179 at 17:32:34,
    **5,764 at 17:37:13**,
    3,326 at 17:50:19,
    1,366 at 17:52:45,
    4,264 at 17:53:47,
    **9,820 at 17:57:53**,
    2,860 at 18:03:03);
    two over the bound,
    never two in a row.
  - By the criterion no symptom;
    by the rule this file set on 2026-11-13 at 13:33,
    readings taken during a sizing walk cannot be put down to the agents alone,
    so these do not judge the level.
  - The walk had run fifty-three minutes at 18:06 with nothing printed.
- Free room at 18:06:54: `/var/home` 506 GiB,
  the build volume 1.5 TiB by `df`'s rounding
  (1,467 GiB by the launcher's count at 17:51).
- The walk was stopped by hand at 18:24:20,
  once it had printed what each toolchain's build folder holds.
  The probe of this level started by itself at that moment and was stopped at 18:30,
  because the owner had the build folders of two old toolchains deleted from 18:30:32
  (`doc/handover/cli-git-rust-implementation.md`,
  the section dated 2026-11-13T18:33:32-05:00),
  and readings taken during that deletion cannot be put down to the agents alone.
  The deletion takes its own write-and-sync reading of both drives before each shard,
  recorded in the same readings file with a `during` field and no agent count.
  **The hour that judges this level starts when the deletion has ended.**

#### The hour that judges, 2026-11-13T18:51:10-05:00 to 19:49:23-05:00

Not final and not approved:
read by the coordinating session from the probe's own file
(`~/temp/agent/disk-ceiling-search-by-agent-count-2026-11-13/keep/probe-readings.jsonl`,
summarised by `summarise-the-probe-readings-between-two-times.ts` beside it).
Neither the sizing walk nor the deletion ran in this hour.

- **What was going was twenty to twenty-two agent units,
  not thirty-two.**
  59 readings,
  one a minute;
  the count of `pi-*.service` units at each:
  least 20,
  median 21,
  most 22.
  That count holds every session's Pi runs on this machine and no agent of another kind.
  The cap of thirty-two was never reached,
  because one author pool of twenty and the reviews held to five bound it,
  which the owner chose to keep at 17:56.
- Home drive write-and-sync:
  median 1 ms,
  slowest 91 ms.
- Build volume write-and-sync:
  median 7 ms,
  slowest 3,361 ms;
  the five slowest 1,288,
  1,461,
  2,395,
  2,572 and 3,361 ms.
  No reading over five seconds,
  so never two in a row.
- Load average over one minute:
  least 5.27,
  median 11.11,
  most 20.27.
- Runs started in the hour:
  twelve,
  each with a first answer;
  the slowest first answer 75,032 ms,
  none over 120 s
  (`time-to-first-answer-of-each-pi-run-started-since.ts`).
- Runs that ended in the hour:
  ten;
  none on a disk error.
  Two ended `driver-failed` by the defect of issue 6299,
  each with its work done,
  before its fix reached `main` at 19:36:16.
- **By the criterion,
  twenty to twenty-two agents held for an hour on this disk.**
  That is the highest count this file holds an undisturbed hour for.
  The ceiling is not found:
  no count above twenty-five can be reached while the pools stay as they are,
  so the search rests here until the owner changes a pool or the new disk arrives,
  when it is found again from four.
- The cap in "The cap now" stays thirty-two:
  it is a bound never to pass,
  and nothing measured says it is too high or too low.

#### A later hour at the same count, 2026-11-14T00:15:00-05:00 to 00:58:17-05:00

Not final and not approved;
measured by the coordinating session.
No probe of the drives ran in this hour:
the last reading before it is of 2026-11-13T19:49:23-05:00.

- **Agents**: 22 agent units going at 00:59,
  the count of the judged hour.
- **First answers**
  (`time-to-first-answer-of-each-pi-run-started-since.ts`,
  runs started since 23:15):
  seventeen runs,
  all answered,
  the slowest in 98,843 ms,
  six of them in over a minute,
  none over the 120 s that fails a level.
  In the judged hour the slowest was under 75 s.
- **The user service manager**
  (`~/temp/agent/disk-ceiling-search-by-agent-count-2026-11-13/keep/slow-answers-of-the-user-service-manager.jsonl`):
  29 slow answers asked between 00:16 and 00:56,
  from 7,600 ms to 73,292 ms;
  the nine hours and twenty minutes before hold 41.
  In its samples the load average runs from 35 to 96 on eight threads
  (13.8 at the judged hour's last reading),
  the kernel's full I/O stall over ten seconds from 3 to 40 percent,
  and the journal daemon is in uninterruptible sleep in most.
  These figures fail no level.
- **What it did to work**:
  five of 49 commits through the wrapper took over five seconds
  (`doc/handover/cli-git-rust-implementation.md`,
  the section of 2026-11-14T00:58:17-05:00),
  one of them 19.8 s between the manager logging a unit started and the program's first line.
  By the owner's answer under "What counts as the ceiling" a slow commit is no symptom.
- **By the criterion the level still holds**
  (no longer so:
  the probe started at 01:00 failed the level at 01:10:30,
  see "Levels 32 and 24 fail on the home drive, 2026-11-14T00:59:04-05:00 to 01:18:45-05:00"):
  no first answer over 120 s,
  no run ended on a disk error.
  The second half of the criterion had no reading,
  so the probe runs again since 01:00
  (unit `disk-ceiling-probe-level-32-while-reviews-and-four-passes-build-20261114T0100`,
  eight hours;
  it ends with status 3 when a drive's write-and-sync takes over five seconds twice in a row).
- What differs from the judged hour at the same count is not measured.
  Not ruled out:
  reviews that build the wrapper's crate cold,
  four integration passes building at once,
  a `cargo-shadow-lean.service` that ran 56 minutes until 00:31,
  and `toolchain-daily-update.service`,
  whose timer fired at 00:16:25,
  two seconds before the first slow answer of the service manager in this hour was asked
  (found at 01:18, after this section was written).

#### Levels 32 and 24 fail on the home drive, 2026-11-14T00:59:04-05:00 to 01:18:45-05:00

Not final and not approved;
measured by the coordinating session.
The cause is not established and none is claimed.

- **The readings**
  (`keep/probe-readings.jsonl`;
  home drive write-and-sync in ms,
  agents going beside each):
  00:59:04 111,136 (22);
  01:01:55 1 (20);
  01:02:55 128 (20);
  01:04:17 1 (20);
  01:05:17 1 (21);
  01:06:17 3 (21);
  01:07:17 132,701 (21);
  01:10:30 10,339 (20);
  01:12:25 29,020 (21);
  01:13:54 7 (21);
  01:14:54 102,109 (21);
  01:17:36 37,279 (21);
  01:18:45 40,119 (22);
  01:20:25 52 (21).
  The build volume in the same readings:
  2 to 178 ms,
  once 2,553 ms.
- **Level thirty-two fails at 01:10:30**:
  two readings in a row over five seconds
  (132,701 ms and 10,339 ms)
  with twenty and twenty-one agents going.
  The rule gives twenty-four.
  The filling unit that had run since 17:12 ended at 01:12:16;
  one at level twenty-four was started at 01:16:46.
- **Level twenty-four fails at 01:18:45**:
  37,279 ms at 01:17:36 and 40,119 ms at 01:18:45,
  two readings in a row,
  both after the filling unit was at twenty-four,
  with twenty-one and twenty-two agents going.
  They are the last reading of one probe process and the first of the next
  (the first had ended with status 3 on 102,109 ms and 37,279 ms,
  of which only the second falls after 01:16:46),
  so no single process printed this failure;
  the criterion asks for two readings in a row and these are.
  The rule gives twenty.
  The filling unit at twenty-four was stopped between two rounds
  and one at twenty started at 01:21:35
  (`fill-free-places-and-put-ended-reviews-back-at-level-20-20261114T0123`,
  eight hours,
  no successor;
  the stamp in its name is two minutes late).
  The probe runs again since 01:21:42
  (`disk-ceiling-probe-level-20-since-the-filler-is-at-20-20261114T0122`,
  eight hours).
- **The count did not change between the level that held and the two that failed**:
  the judged hour of level thirty-two
  (2026-11-13 18:51 to 19:49)
  had twenty to twenty-two agents and no slow reading twice in a row;
  both failures have twenty to twenty-two.
  So the label on the level discriminates nothing here,
  and what does is not known.
- **Running in the failing hour and not in the judged one**:
  `toolchain-daily-update.service`
  (its timer fired 00:16:25,
  the service started 00:16:48 and was still in its start step at 01:20:49;
  `mise run toolchain:update:daily`,
  which checks the packages before it updates;
  `CPUQuota` 200%,
  `MemoryMax` 8G,
  `IOWeight` 1,
  `Nice` 10).
  By 01:18 its cgroup had read 7.83 GB and written 5.24 GB on the home device.
  Other timers of the hour:
  `snapper-cleanup` 00:00:07,
  `logrotate` 00:22:41,
  `btrfs-qgroup-rescan-home` 00:37:12
  (answered "already consistent, nothing to do";
  the kernel logged a quota scan of the home filesystem completed at 00:14:49 and at 00:53:37),
  `kopia-snapshot-all` 00:40:36
  (in `auto-restart` at 01:19),
  `rclone-bisync-garage` 01:00:36.
  None of these is shown to be the cause.
- **Who used the home device in one minute of it**
  (01:17:21 to 01:18:21,
  from each unit's `io.stat` under the user manager's `app.slice`;
  `keep/home-drive-bytes-by-unit-over-one-minute-2026-11-14T0617.txt`;
  system units are not in the sample):
  all 99 units and scopes together wrote about 2 MiB and read about 1.4 GiB.
  The largest readers:
  the JetBrains toolbox 490 MiB,
  the daily update 233 MiB,
  sixteen command scopes 298 MiB together,
  twenty-one Pi units 173 MiB together.
  A stall of 37 s falls in that minute,
  so the writes of the user's units do not explain it;
  reads,
  system units and the filesystem's own work are not told apart by this sample.
- **Against the other half of the criterion**:
  no Pi run went without a first answer for 120 s,
  and none ended on a disk error.
- **Not done yet**:
  the control reading,
  the same count with the daily update ended,
  to see whether the stalls go with it.
  The question whether an hour that holds the daily update may judge a level is kept for the owner
  (`doc/handover/cli-git-rust-implementation.md`,
  "Questions kept for the owner's return").

#### Level 20 fails on the home drive, 2026-11-14T01:37:37-05:00 to 01:41:00-05:00

Not final and not approved;
measured by the coordinating session.
The cause is not established and none is claimed.

- **The readings at level twenty**
  (home drive write-and-sync in ms,
  agents going beside each;
  the filling unit was at twenty from 01:21:35):
  01:25:57 2 (21);
  01:26:57 540 (21);
  01:28:06 1 (19);
  01:29:06 70,462 (18);
  01:31:16 233 (19);
  01:32:18 33 (20);
  01:33:18 87 (19);
  01:34:19 1 (20);
  01:35:19 11 (20);
  01:36:19 187 (20);
  01:37:37 143,475 (20);
  01:41:00 57,262 (18).
  The build volume in the same readings:
  6 to 47 ms.
- **Level twenty fails at 01:41:00**:
  143,475 ms and 57,262 ms,
  two readings in a row,
  with twenty and eighteen agents going.
  The probe ended with status 3.
  The rule gives eighteen.
  The filling unit at twenty was stopped and one at eighteen started at 01:42:18
  (`fill-free-places-and-put-ended-reviews-back-at-level-18-20261114T0142`,
  eight hours,
  no successor);
  the probe runs again since then
  (`disk-ceiling-probe-level-18-since-the-filler-is-at-18-20261114T0142`,
  eight hours).
- **What else ran in those minutes**:
  `toolchain-daily-update.service` until 01:40:01,
  so the first of the two readings falls wholly inside it
  and the second began about three seconds after it ended;
  and this session's own work in the main checkout:
  one run of `package/mpm-shadow/cli`'s Clippy task from 01:36
  (in a scope with `IOWeight=1`,
  its build folder on the build volume),
  and the merge of integration pass 28 at 01:38:12 with its push
  (26 files,
  4,112 lines).
  None of these is shown to be the cause.
- **The control hour starts here**:
  the daily update has ended,
  so readings from 01:42 on are the same machine without it.
  If the stalls stop while eighteen to twenty agents go,
  that is evidence about the daily update and none about the count,
  and the level is still the one the criterion gave;
  raising it again is the owner's question already kept.
  (They did not stop:
  the section that follows has the readings.)

#### Level 18 fails after the daily update ended; a snapshot cleanup has run since midnight, 2026-11-14T01:42:20-05:00 to 01:54:55-05:00

Not final and not approved;
measured by the coordinating session.
No cause is claimed.

- **The readings at level eighteen**
  (home drive write-and-sync in ms,
  agents going beside each;
  the daily update ended at 01:40:01):
  01:42:20 45,431 (18);
  01:44:06 4 (18);
  01:45:06 59,278 (18);
  01:47:05 27 (17);
  01:48:09 47,148 (17);
  01:49:56 63 (count not read);
  01:51:33 141,638 (17);
  01:54:55 24,317 (16).
  The build volume in the same readings:
  6 to 62 ms.
- **Level eighteen fails at 01:54:55**:
  141,638 ms and 24,317 ms,
  two readings in a row,
  with seventeen and sixteen agents going.
  The rule gives seventeen.
  A filling unit at seventeen runs since 02:03:15
  (`fill-free-places-and-put-ended-reviews-back-at-level-17-20261114T0204`,
  eight hours,
  no successor),
  and the probe since 02:04
  (`disk-ceiling-probe-level-17-readings-20261114T0204`).
  Between 01:54:52 and 02:03:15 no filling unit ran,
  for a reason of its own
  (`doc/handover/cli-git-rust-implementation.md`,
  the section of 2026-11-14T02:05:16-05:00:
  a lock left in Pi's folder stopped every new start).
- **What the control showed**:
  five stalls of 24 to 142 s came after the daily update had ended.
  So the stalls do not go with the daily update,
  and the question kept for the owner about hours that hold it is withdrawn.
- **The count**:
  every other reading at this level is a stall,
  with sixteen to eighteen agents going.
  The level that held its hour on 2026-11-13 from 18:51 to 19:49 had twenty to twenty-two.
  Fewer agents and more stalls:
  the criterion keeps failing and the count does not explain it.
- **A lead, read from the system's journal and not tested**:
  `snapper-cleanup.service` started at 00:00:07 and was still running at 01:55:53.
  At 00:14:49 it logged the deletion of a batch of snapshots of the root configuration in one second
  (numbers 11 to 22 among them;
  the whole count was not read),
  and the kernel logged in the same second a quota scan of the home filesystem completed
  (`BTRFS info (device dm-1): qgroup scan completed`),
  after `qgroup marked inconsistent` at 00:00:20.
  The first slow answer of the user service manager that night was asked at 00:16:27,
  98 s after those deletions.
  `snapper-timeline.service` started at 01:00:55 and had not ended at 01:55:53.
  `btrfs-zstd-policy.service` showed 977 restarts.
  The cleanup did not run in the hour that held:
  its only start since noon of 2026-11-13 is the one at 00:00:07.
  The home folder is the `home` subvolume of a btrfs filesystem on `dm-1`,
  mounted with `discard=async`;
  the kernel's qgroup lines for `dm-1` show quota groups in use there.
  These are system units:
  this session stops none of them and cannot read their I/O
  (the per-unit sample of 01:17 covers the user's units only).
- **What would test the lead**:
  readings after the cleanup has ended;
  and whether a night without it stalls.
  Both are readings the probe gives if it is left running.

#### Level 17 fails and the search ends at sixteen, 2026-11-14T02:03:18-05:00 to 02:47:00-05:00

Not final and not approved;
measured by the coordinating session.
No cause is claimed.

- **The readings at level seventeen**:
  forty,
  one a minute where the minute before did not stall,
  from 02:03:18 to 02:47:00,
  with fifteen to seventeen agents going
  (four readings could not count them:
  `systemctl` gave no answer in twenty seconds).
  The home drive's write-and-sync took a median of 43 ms,
  and over the bound of 5,000 ms seven times
  (ms, agents going beside each):
  02:03:18 33,165 (16);
  02:07:01 13,765 (17);
  02:10:21 20,942 (17);
  02:21:27 14,272 (16);
  02:28:50 11,826 (17);
  02:45:53 6,462 (17);
  02:47:00 7,204 (17).
  The build volume in the same readings:
  a median of 10 ms,
  and over one second three times
  (02:11:42 1,408; 02:12:43 3,172; 02:24:41 2,271).
- **Level seventeen fails at 02:47:00**:
  6,462 ms and 7,204 ms,
  two readings in a row.
  It had stood for forty-four minutes,
  short of its hour.
  The stalls are shorter and further apart than at eighteen
  (seven in forty readings against four in eight,
  the longest 33 s against 142 s);
  one level's readings do not say whether that is the count or the night moving on.
- **What was done**:
  the filling unit at seventeen was stopped at 02:48:16 and one at sixteen started in the same second
  (`fill-free-places-and-put-ended-reviews-back-at-level-16-20261114T024816`,
  eight hours,
  no successor);
  no run that was going was stopped.
  The probe runs on at sixteen for readings only
  (`disk-probe-for-readings-only-at-level-16-20261114T024816`,
  eight hours;
  the probe script takes `readings-only` as a third argument and then never ends on slow readings),
  into the same log,
  `keep/probe-readings.jsonl` of the search's scratch folder.
- **The lead still stands untested**:
  at 02:48:37 `snapper-cleanup.service` was still running
  (since 00:00:07),
  and `snapper-timeline.service` too
  (since 01:00:55).
  So no reading yet comes from after the cleanup's end.

#### The stalls end within seven minutes of the snapshot cleanup's end, 2026-11-14T02:48:16-05:00 to 08:38:18-05:00

Not final and not approved;
measured by the coordinating session.
No cause is claimed:
two things changed in the same hours.

- **The readings at level sixteen**,
  one a minute,
  340 of them from 02:48:16 to 08:38:18.
  Slow ones
  (home drive write-and-sync over 5,000 ms)
  by hour:
  02:48 to 03:00 one of twelve (13,843 ms);
  03:00 to 04:00 eight of 54 (the longest 49,518 ms);
  04:00 to 05:00 ten of 56 (33,815 ms);
  05:00 to 06:00 one of 59,
  the last of all at 05:08:45 (10,608 ms);
  06:00 to 08:38 none of 159,
  the longest 106 ms.
- **`snapper-cleanup.service` ended at 05:15:35**,
  five hours and fifteen minutes after its start at 00:00:07
  (`systemctl show`, `InactiveEnterTimestamp`).
  The last slow reading is six minutes and fifty seconds before that.
- **What also changed**:
  no Pi run was started from 04:00:14 on
  (a queue entry that failed every start;
  `doc/handover/cli-git-rust-implementation.md`,
  the section of 2026-11-14T08:50:39-05:00),
  so the count of agents fell through those hours and was nothing from 05:59:02.
  The hour from 04:00 to 05:00 had the most slow readings of the night's last three with the count already falling,
  which speaks for the cleanup and against the count;
  it does not separate them.
- **What would separate them**:
  sixteen agents going with no cleanup running,
  which the day gives once the places are full again;
  and the next cleanup,
  which the system starts by its timer.
  The probe records until 10:48 and is started again then.
  (It was stopped at 08:54:33 instead,
  when the level went to twenty-four and a probe that ends on slow readings took its place.)

#### Level 24, started 2026-11-14T08:54:35-05:00

Not final and not approved;
measured by the coordinating session.

- **To 09:22:48**:
  twenty-nine readings,
  the agents going rising from five to seventeen as the places filled.
  One slow reading of the home drive:
  7,754 ms at 08:55:36,
  with eight agents going,
  the kernel's `some` I/O pressure at 22 percent over ten seconds,
  and neither `snapper-cleanup.service` nor `snapper-timeline.service` running.
  The build volume took 2,958 ms once,
  at 09:03:44.
  Never two slow readings in a row,
  so the level stands.
- **A reading above the level: twenty-eight agents from 09:43 to 09:45**
  (the probe's last four readings to 09:45:58,
  and `systemctl` at 09:44:26 and 09:45:50),
  four over twenty-four,
  because the loop that fills the places does not count a review it has asked for until its launcher starts it.
  From 09:23:48 to 09:45:58:
  twenty-three readings,
  none slow,
  the home drive 204 ms at most.
  Nothing was stopped.
- **What that reading says about the lead**:
  a stall of the same kind,
  shorter,
  with no snapshot cleanup running and fewer agents than any level that failed.
  So the cleanup is not the only thing that stalls this drive.
  In the minutes before it this session installed a wrapper release and made several commits in the main checkout;
  nothing was measured that ties the reading to either.
