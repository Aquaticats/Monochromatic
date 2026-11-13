# Agents at once on this machine

Every session on this machine reads this before it starts a Pi run or a subagent.
It holds the number that is allowed now,
how to count,
and the search for the number the drives carry.

## The cap now

At most **16** agents run on this machine at once,
counted across every session
(raised from 4 to 8 on 2026-11-13 at 11:57 and from 8 to 16 at 12:59,
each time when the level before had held its hour).

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
    systemctl --user list-units --no-legend --plain --state=active 'pi-*'
    ```

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
  The bound is 5,000 ms twice in a row,
  and the probe's hour ended on that reading,
  so whether a second would have followed is not known.
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

### Level 16, started 2026-11-13T12:59:30-05:00

- Going when it started: eight runs of the session doing the search
  (resumes of wrapper work,
  two launcher authors answering reviews,
  reviews).
- Added: queue entries twenty seconds apart until sixteen are going,
  reviews and resumes of wrapper work that each build Rust.
- Probe: unit `disk-ceiling-probe-level-16-20261113`,
  the same file,
  one hour.
