# Agents at once on this machine

Every session on this machine reads this before it starts a Pi run or a subagent.
It holds the number that is allowed now,
how to count,
and the search for the number the drives carry.

## The cap now

At most **4** agents run on this machine at once,
counted across every session.

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
- The probe was started again at 10:57 with its home file under `~/.cache`,
  which is on `/var/home`.
  The hour of the level counts from 10:57.
