# Agents at once on this machine

Every session on this machine reads this before it starts a Pi run or a subagent.
It holds the number that is allowed now,
how to count,
and the search for the number the drives carry.

## The cap now

At most **32** agents run on this machine at once,
counted across every session
(raised from 4 to 8 on 2026-11-13 at 11:57,
from 8 to 16 at 12:58:50
and from 16 to 32 at 17:12,
each time when the level before had held its hour).
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
The search is done again after the new drive is in (expected 2026-11-15).

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
- The hour counts from the first reading with the count above sixteen for good,
  and is read by the counts it holds:
  a count that stays near twenty is a reading at twenty,
  not thirty-two held.
