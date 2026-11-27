# Ordering the worktree adds of starts that come from several lists

## Status

Proposal,
written 2026-11-27T04:14:00-05:00 by the coordinating Claude Code session,
for [issue 6563](https://github.com/Aquaticats/Monochromatic/issues/6563).
Unapproved and non-final:
nothing here is built,
no code and no test changed,
and the owner decides whether a rule about starts exists at all.
This round measured and proposed only.

Two reviews were taken of an earlier version.
The independent one,
from another vendor (`neuralwatt/nw-large`,
through the repository's advisor),
is folded in:
its findings corrected claims of overreach in the measured section,
that no attempt returns beside twenty-four adds in flight,
that the count of adds alone sets an add's cost,
that a group of adds lived at a steady level,
and that a killed add's wait is known to be in one half of the copy.
The other is a Claude Sonnet gate read,
a same-vendor second pass rather than an independent one,
which was still running when this version was written;
what it returns is folded in before the owner is asked,
or named as unreturned if it does not.

Whether a new rule about starts is the owner's to give,
and the project holds both halves of it:

- The owner,
  2026-11-26:
  "I didn't tell you to apply the 3 minutes apart and memory test rules."
  (quoted in [the planning record](pi-round-start-failures-and-what-would-stop-them.md),
  "Limits an agent made up,
  later read as the owner's").
  A bound that an agent measured and then applied as policy is exactly what that sentence refuses.
- How many rounds run at once is already recorded as
  "Up to 64 rounds at once."
  ([the decision record](../decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md),
  "What it means").

So the figures below are given as a measurement with its limits,
never as a limit to adopt,
and the questions at the end are the owner's.

What is not known,
and said again under "What the figures support,
and what they do not":

- the number of adds in flight at which an add starts to miss a bound of fifteen minutes,
  because the adds measured here lived through a rising crowd rather than a steady level,
  and the spread inside one group of adds that began together is larger than the difference between the groups;
- whether the count of adds is even the thing that sets an add's cost,
  because seven adds started together at 02:12 that morning each had a run within about eighty seconds (issue 6563),
  while the three adds that began together at 02:37 took 143 s to 145 s of copy alone;
- which half of the ignored-state copy holds the time,
  the walk of the source's ignored tree,
  the staging of its entries,
  or installing them into the worktree;
- how far a killed add had got,
  because a killed add writes no record of its steps;
- whether a cap of any figure would have held that morning,
  because nothing in this round started or capped an add;
- the host's I/O pressure per add,
  which nothing sampled at the time.

## What happens today

### A list starts twenty seconds apart, and each list starts its own adds

`start-pi-runs-from-a-batch-list` pushes every entry's start and then waits,
so entries do not wait for each other,
and one list's adds are twenty seconds apart from first to last:

```ts
// package/pi-shared/subagent-driver/src/batch-list.ts:22
/**
 Milliseconds between starting one entry and the next, which is what keeps a batch's starts from all asking for a slot at once.
 */
export const BATCH_STAGGER_MILLISECONDS = 20_000;
```

```ts
// package/pi-shared/subagent-driver/src/batch-list.ts:500,508
    started.push(start({
      ...
    await wait(BATCH_STAGGER_MILLISECONDS,);
```

Nothing counts adds,
and nothing in the batch command knows another batch command exists.
`README.md`,
"`start-pi-runs-from-a-batch-list`",
says the stagger is what keeps one batch's starts from asking at once,
which is true of one batch and of no pair of batches.

### The add, its bound, and its second attempt

The add is `git worktree add` through the Git policy wrapper,
bounded at fifteen minutes and tried twice:

```ts
// package/pi-shared/subagent-driver/src/worktree-of-a-run.ts:42-48
/**
 How long `git worktree add` may take, in minutes: making a worktree on this machine copies the checkout's ignored state, and the
 wrapper says so while it copies. Fifteen minutes is a bound rather than a measurement, sized for a loaded machine: on
 2026-11-11 several adds ran at once, the wrapper reported one still copying after 110 seconds, and the two-minute bound every
 other git question gets killed it, which lost two reviews.
 */
const WORKTREE_ADD_MINUTES = 15;
```

```ts
// package/pi-shared/subagent-driver/src/worktree-of-a-run.ts:60-64
/**
 How many times one start tries to make its worktree. A first attempt killed by its own bound leaves a partly made worktree and
 a registered path behind, so the next attempt starts by clearing what this start made, and one kill does not lose the run.
 */
const WORKTREE_ADD_ATTEMPTS = 2;
```

Between the attempts the launcher clears what the killed one left,
with the same fifteen-minute bound,
because that removal copies and deletes as much as the add did:

```ts
// package/pi-shared/subagent-driver/src/worktree-of-a-run.ts:322-334
      // A failed add may have made the folder and registered the path before its bound killed it. Both are this start's own,
      // because the folder held no worktree when this began, so they are cleared and the attempt is made again.
      await git({
        folder: checkoutFolder,
        words: [
          'worktree',
          'remove',
          '--force',
          worktree,
        ],
        // Removing what a killed add left is the same copy the add was making, so it gets the same bound.
        timeoutMilliseconds: WORKTREE_ADD_TIMEOUT_MILLISECONDS,
      },);
```

### What a killed attempt leaves

The driver kills the wrapper at the bound,
so the wrapper never reaches its own cleanup,
and the machine keeps:

- the worktree folder and its registration in Git,
  because the second attempt's `git worktree remove --force` is what would clear them,
  and the last attempt is never followed by one
  (`worktree-of-a-run.ts:319`:
  "`if ((attempted.status === 0) || (attempt === WORKTREE_ADD_ATTEMPTS))`" returns the attempt);
- the branch the start made,
  because the branch delete after a failed add is refused while a worktree has it checked out,
  and the launcher's line says so;
- the stage folder `.claude/worktrees/.cli-git-worktree-copy-<id>` with whatever the copy had staged,
  which the wrapper's own `README.md` describes in "State on disk":
  "A stage a killed `git worktree add` left beside its destination is removed by the next command that runs the recovery,
  with one line saying what went,
  when the record written beside the stage's claim names the object still standing at that name and the folder holds nothing that record cannot account for;
  a stage nothing proves is named,
  left whole,
  and keeps its claim";
- the worktree's ignored state as a debt:
  "A worktree whose ignored-state copy ended before it finished is filled by the next command run in that worktree"
  (`package/git-policy/cli/README.md`,
  "State on disk").

The wrapper's `README.md` also says the add itself never waits on another add:
"Concurrent adds in one repository never wait for one another,
and each worktree receives exactly its own copy".
That sentence is the wrapper's decision,
and it is what the third option below would have to change or leave alone.

### The two attempts do not run together, so the wave repeats

A killed first attempt is followed by its own removal and its own second attempt,
so the second wave starts one wave later,
with whatever is left still in flight.
That is how sixteen starts spent thirty minutes on two failed attempts each,
which the measured figures below show.

### Which step of an add waits

The wrapper names the steps it is in,
and issue 6563 quotes two of those lines,
one naming `git check-ignore` and one naming a bulk `git ls-files --others`.
Measured,
those are not where the time went:

- `git ls-files` inside an add took 0.5 s to 3.1 s,
  and `git check-ignore` 0.1 s to 2.7 s,
  in every add that returned;
- Git's own checkout of the 26,836 tracked files took 6.5 s to 35.8 s;
- the ignored-state copy took 143.4 s to 859.1 s,
  which is 89% to 99% of each add.

The wrapper's own `README.md`,
"Repairing package-manager state in a new worktree",
says what that copy is:
"the wrapper copies the ignored state of the source worktree into it without any lock",
and "the copy clones files with a copy-on-write request,
in parallel on a large tree".
The checkout's ignored state was 39,692 entries that morning (issue 6563),
so thirty adds ask the same source for 30 copies of 39,692 entries,
and 27 at once ask for 1,071,684 clone and record steps against one tree.

A killed add is the part of this the figures cannot speak for,
and the issue reads it differently from this document.
The issue says the stage folder held every entry of the ignored-state copy in each add that was killed,
"so it is after the copy".
What this round can add to that:

- for the adds that returned,
  89% to 99% of the add is inside the copy,
  and the steps after it (the records,
  the package-state repair,
  the relink hand-over) are 1 s to 2.5 s together;
- the wrapper's own last line before a killed add's bound names the ignored-file copy as what it was still doing,
  in the `.out` files of the three lists:
  "`git worktree add` is still running after 891 seconds;
  it is copying the ignored files into the new worktree";
- so both readings put the time inside the copy,
  and they differ on which half of it:
  staging the entries into the stage folder,
  or installing them into the worktree afterwards.
  A killed add writes nothing that would settle it.

## What was measured

### Where the figures come from

Everything here is read off what the morning of 2026-11-27 left.
No add was started for this measurement,
and no cap,
delay or order was tried,
because an experiment of overlapping adds is synthetic load and the round's rules forbid it.

- `journalctl --user --unit 'launch-*' --since '2026-11-27 02:00'`,
  which holds the batch units' own starts,
  the restart list's start,
  and one warning entry per add the wrapper told about its route.
- The batch lists' `.out` files beside them under `~/temp/agent/pi-runs`,
  which are the units' whole output.
- The run folders under `~/temp/agent/pi-runs`:
  a start makes its run folder about 30 ms to 100 ms after its add returns,
  which this round measured on the six adds that returned in the first wave,
  so a run folder's birth time is when its add had ended.
- The birth times of the worktree folders `.claude/worktrees/pi-*`,
  which is when Git created the destination,
  within seconds of the add's start (the limits below say how near).
- The birth and change times of the stage folders `.claude/worktrees/.cli-git-worktree-copy-*`,
  which is when the copy's staging began and last wrote.
- The wrapper's own per-add timing record,
  `<the checkout>/.git/monochromatic-git-policy-cli-worktree-add-timing.jsonl`,
  which holds one line per add that exited,
  with the milliseconds of each step:
  `gitAdd`,
  `copyIgnored`,
  `git ls-files`,
  `git check-ignore`,
  the route it took and the reason it took the slow one.
  It rotates by size,
  so a reader who comes later may find the incident's lines gone;
  what this record uses from it is quoted below.

The limits of those sources,
stated once here and relied on below:

- an add killed at its bound writes no timing line:
  the record holds a line for each of the fourteen adds that returned and one for none of the forty killed,
  because a kill gives the wrapper no way to write one.
  So nothing here knows how far a killed add had got,
  only that it had not finished;
- the journal's warning line marks where the ladder decided the route,
  which is after Git's own checkout and before the copy,
  so it is not the add's start nor its end;
- the birth time of a worktree folder is exact,
  and it is when Git created the destination,
  which the wrapper's own record puts 0.3 s to 8 s after the add began;
  a first attempt's folder is deleted before its second attempt's is made,
  so for a worktree made twice the standing folder is the second attempt's;
- for the twenty-four first attempts that were killed,
  the folder is gone and the add's start is the batch's schedule to within a few seconds.

### The thirty starts

- Three lists of 11,
  8 and 11 entries,
  started at 02:37:23.115,
  02:37:24.174 and 02:37:25.317.
- Their first adds began at 02:37:29.45,
  three of them within 42 ms of each other,
  and the thirtieth began at about 02:40:45 (its list's schedule,
  to within a few seconds):
  every add of the round was in flight within three and a half minutes of the first.
- The most adds in flight at one moment:
  27,
  from about 02:40:45 to 02:43:24.
  Then 26 until 02:52:14,
  and 24 until the last first attempt was killed.
- First attempts:
  6 returned,
  taking 153.6 s,
  154.0 s,
  154.7 s,
  339.5 s,
  865.9 s and 869.5 s;
  24 were killed at 900 s.
- Second attempts:
  24 began between 02:54:52 and 02:56:27,
  which is 30 s to 117 s after their own first attempt was killed,
  a gap holding the removal of what that attempt left and the start of the next;
  8 returned,
  taking 776.7 s,
  799.6 s,
  810.3 s,
  865.4 s,
  870.5 s,
  888.5 s,
  894.8 s and 896.8 s,
  and 16 were killed at 900 s.
- The sixteen that failed both attempts were started again at 03:12:16 from one list,
  which needed no add at all,
  because each worktree was already there,
  and their runs began between 03:13:10 and 03:17:29.
  Their ignored state was still owed,
  so each run's first command made the copy the killed add never finished.
- The eight second-attempt adds that returned all ended between 03:09:20 and 03:10:36,
  five of them 6 s to 8 s after the sixteenth killed add hit its bound.
  They had been running for thirteen to fifteen minutes when they ended.

### Each add, and what was in flight during it

The same figures one add at a time.
`A`,
`B` and `C` are the three lists,
the digit is the entry's place in its list,
and `add` says which attempt it was.
`began` and `ended` are local times on 2026-11-27,
`seconds` is the add's own length,
`at start` is how many adds were in flight at the moment it began,
`peak` the most that were in flight while it ran,
and `mean` the average over its own life.
A `seconds` of 900.0 is the bound,
not a measurement of how long the work needed:

```text
start  add  began     ended     seconds  at start  peak  mean
C1   1 02:37:29 02:40:04   154.7        1    24  13.8
B1   1 02:37:29 02:40:03   154.0        2    24  13.8
A1   1 02:37:29 02:40:03   153.6        3    24  13.7
A2   1 02:37:45 02:52:14   869.5        5    27  24.3
C2   1 02:37:45 02:43:24   339.5        5    27  21.7
B2   1 02:37:48 02:52:14   865.9        6    27  24.4
C3   1 02:38:03 02:53:03   900.0        7    27  24.7
B3   1 02:38:04 02:53:04   900.0        8    27  24.7
A3   1 02:38:05 02:53:05   900.0        9    27  24.7
C4   1 02:38:23 02:53:23   900.0       10    27  25.0
B4   1 02:38:24 02:53:24   900.0       11    27  25.0
A4   1 02:38:25 02:53:25   900.0       12    27  25.0
C5   1 02:38:43 02:53:43   900.0       13    27  25.1
B5   1 02:38:44 02:53:44   900.0       14    27  25.1
A5   1 02:38:45 02:53:45   900.0       15    27  25.1
C6   1 02:39:03 02:54:03   900.0       16    27  25.1
B6   1 02:39:04 02:54:04   900.0       17    27  25.1
A6   1 02:39:05 02:54:05   900.0       18    27  25.1
C7   1 02:39:23 02:54:23   900.0       19    27  25.0
B7   1 02:39:24 02:54:24   900.0       20    27  25.0
A7   1 02:39:25 02:54:25   900.0       21    27  25.0
C8   1 02:39:43 02:54:43   900.0       22    27  24.7
B8   1 02:39:44 02:54:44   900.0       23    27  24.7
A8   1 02:39:45 02:54:45   900.0       24    27  24.7
C9   1 02:40:03 02:55:03   900.0       24    27  24.4
A9   1 02:40:05 02:55:05   900.0       23    27  24.4
C10  1 02:40:23 02:55:23   900.0       24    27  24.3
A10  1 02:40:25 02:55:25   900.0       25    27  24.2
C11  1 02:40:43 02:55:43   900.0       26    27  24.1
A11  1 02:40:45 02:55:45   900.0       27    27  24.1
A4   2 02:54:52 03:09:52   900.0        9    24  23.3
B3   2 02:54:52 03:09:52   900.0        9    24  23.3
C3   2 02:54:52 03:09:52   900.0        9    24  23.3
B4   2 02:54:52 03:09:52   900.0       10    24  23.3
C4   2 02:54:52 03:09:52   900.0       11    24  23.3
A3   2 02:54:52 03:09:52   900.0       12    24  23.3
B6   2 02:55:02 03:10:02   900.0       13    24  23.4
C5   2 02:55:02 03:10:02   900.0       14    24  23.4
A5   2 02:55:02 03:10:02   900.0       15    24  23.4
B5   2 02:55:02 03:10:02   900.0       16    24  23.4
C6   2 02:55:02 03:10:02   900.0       17    24  23.4
A6   2 02:55:06 03:10:06   900.0       16    24  23.3
B8   2 02:55:20 03:10:20   900.0       17    24  23.2
A8   2 02:55:20 03:10:20   900.0       18    24  23.2
C8   2 02:55:21 03:10:21   900.0       19    24  23.2
C9   2 02:55:28 03:10:28   900.0       18    24  23.1
A7   2 02:55:38 03:10:35   896.8       19    24  23.0
A9   2 02:55:39 03:10:34   894.8       20    24  23.1
B7   2 02:55:45 03:10:34   888.5       19    24  23.1
C10  2 02:56:05 03:10:35   870.5       20    24  23.1
A10  2 02:56:10 03:10:36   865.4       21    24  23.2
C11  2 02:56:23 03:09:20   776.7       22    24  24.0
A11  2 02:56:26 03:09:46   799.6       23    24  24.0
C7   2 02:56:27 03:09:57   810.3       24    24  23.9
```

The times of an add that returned are the wrapper's own record of it,
exact.
The times of an add killed at its bound are the batch's schedule and the bound,
to within a few seconds for a first attempt and to within a second for a second attempt,
whose folder was there to read.

### The phases inside an add

For the fourteen adds that returned,
counted from the wrapper's timing record:

- `gitAdd`,
  the destination's registration and Git's own checkout of 26,836 files:
  6.5 s beside three adds to 35.8 s in the crowd,
  so the checkout rose too,
  though by far less than the copy;
- `copyIgnored`,
  the ignored-state copy:
  143.4 s to 859.1 s,
  which is 89% to 99% of each add;
- `git ls-files`:
  0.5 s to 3.1 s;
- `git check-ignore`:
  0.1 s to 2.7 s;
- everything else the add did,
  the recovery,
  the configuration,
  the records:
  0.9 s to 2.4 s.

The copy's own rise,
as each group of adds lived it.
No group lived a steady level:
each began with a few adds in flight and met the rest of the wave while it ran,
and the peak during every one of them was 24 to 27.
That is why these are groups and not a level each:

- the three that began together at 02:37:29.45,
  each with one to three adds in flight at its start,
  copies of 143.4 s,
  144.3 s and 145.3 s;
- the three that began between 02:37:45.15 and 02:37:48.81,
  each with five or six adds in flight at its start:
  copies of 302.8 s,
  833.3 s and 833.5 s,
  so three adds that began within four seconds of each other,
  on the same source,
  spread over a factor of 2.75;
- the eight that returned on their second attempt,
  each with nineteen to twenty-four adds in flight at its start:
  copies of 750.8 s to 859.1 s.

What the groups do not show is a level to level comparison:
the counts rose while each add ran,
and the spreadsheet of one add cannot be separated from the crowd it met.

Where the stage folder appears against the copy tells the same story from another side:
for the three adds that began together,
the copy began at 02:37:35.9 and its stage folder was born at 02:39:39.06,
123 s later,
while the copy lasted 143 s in all.
For a set of adds starting at 03:30:37 the stage folder appeared 2 s after the copy began.
So most of those 143 s were spent before the copy had a stage folder,
and the timing record does not say what it was doing then.
The wrapper's own measurement of a single add at light load splits the copy into selecting the roots,
staging,
installing and removing the stage,
and only the last three make a stage folder
([the ignored-copy record](cli-git-main-worktree-ignored-copy.md),
"The add itself").

### What a killed attempt left, counted

At 03:46:25 on 2026-11-27,
19 stage folders stood in `.claude/worktrees`,
nine of them born inside the incident's window:
four at 02:39:39 and five at 02:56:15.
The issue's own count at 03:11 was eighteen.
The number moves because a copy that finishes removes its stage,
because a run's first command resumes a copy that was owed,
and because later adds leave new ones.
The stage folder measured at 03:46 held 4,917 entries of the 39,692 that morning's count named,
which says what that folder held then,
after the run in that worktree had resumed the copy,
and not what the killed add had left in it.

The session's wrapper prints two lines for each of them on every command in the repository,
which is what the coordinator saw on every command for the rest of the night.

### What the figures support, and what they do not

Supported:

- An add's own Git checkout is a small part of it,
  and the ignored-state copy is nearly all of it,
  in every add that returned.
- The time those adds took rose through the morning:
  143 s to 145 s of copy for the three that began with one to three adds in flight,
  302.8 s to 833.5 s for the three that began with five or six,
  and 750.8 s to 859.1 s for the eight that began with nineteen to twenty-four.
- Every first attempt that began with seven or more adds in flight was killed at its bound:
  the first add of that shape began at 02:38:03,
  and no first attempt after it returned.
  The six first attempts that returned had all begun before it,
  with one to six adds in flight,
  and the three that began with five or six returned after 339.5 s,
  865.9 s and 869.5 s.
- The second wave rescued eight starts and lost sixteen,
  and the eight it rescued took 776.7 s to 896.8 s,
  because they met the first wave's crowd.
- An add that hits its bound leaves its folder,
  its branch and its stage behind,
  writes no timing record,
  and makes the sixteen failures cost a second list and another half hour.
- The copy's cost is not a per-add constant:
  three adds that began within four seconds of each other,
  on the same source,
  spread over 302.8 s to 833.5 s.

Not supported:

- A figure at which an add starts to miss its bound.
  The same group of three adds spread over a factor of 2.75,
  and the groups were measured while the crowd was still growing,
  so no count to time relation can be read off them.
  A bound taken from these figures would be a guess,
  which is the thing the owner refused on 2026-11-26.
- That the count of adds is what sets an add's cost.
  Seven adds started together at 02:12 that morning each had a run within about eighty seconds (issue 6563),
  which is faster than the three adds of 02:37:29 managed,
  so the rest of the machine's load moves this figure too,
  and a cap on adds bounds one of its causes.
- Any claim that a cap of N would have held.
  No capped run was measured,
  and a cap moves the queue from the disk to the launcher without making the copy cheaper:
  thirty adds at four at a time still move 1,190,760 entries,
  and the last of them starts after the first has finished rather than beside it.
- Which half of the copy dominates.
  The stage's appearance says the first 0.4 s to 123 s of a copy is before staging,
  and nothing here says what that time is spent on.
  The adds that were killed are silent about it,
  because a kill leaves no record of their steps.
- What a killed add had done when its bound ended it.
  Nothing here knows how far into its copy a killed add was,
  which is why the issue's reading of the stage folders and this document's reading of the timing record cannot be reconciled.
- The in-flight figures of a killed add to better than a few seconds.
  An add that returned is placed by the wrapper's own record to the millisecond;
  a killed add's first attempt is placed by its list's schedule,
  and its second attempt by its folder's birth less 0.3 s,
  so its own `at start` and `peak` figures carry that uncertainty.
- What the host's I/O pressure was per add.
  The only readings are the issue's own (`some avg10=75.07` at 02:44),
  and nothing sampled per add.
- Whether the machine's own load or the adds caused the copy's rise.
  Other rounds and their builds were running that night,
  sixty-two of them ended between 21:00 and 01:19 (the planning record,
  "Runs waiting on things that are not limits"),
  and this round measured no arm without them.

### The one thing this document corrects in the issue

The issue says every first attempt ran into the fifteen-minute bound and was killed.
Six did not:
the three adds that began at 02:37:29.45 returned after 153.6 s,
154.0 s and 154.7 s,
and the three that began at 02:37:45-48 returned after 339.5 s,
865.9 s and 869.5 s,
each with a line of its own in the wrapper's timing record.
So 24 first attempts were killed,
not 30,
and the fourteen runs that had started by 03:11 are those six and the eight second attempts that returned.
Nothing else in the issue's account changes,
and that count is what the sixteen failures and the list started again at 03:12:16 stand on.

### What this round's evidence outlives

The journal read,
the extracted timing lines,
the birth times and the analysis scripts are in this round's scratch folder,
which expires on 2026-11-28.
What must outlast them is in this document:
the counts,
the ranges,
the phase split and the limits above.
The wrapper's timing record is the one durable source,
and it rotates by size,
so the incident's own lines will be gone from it before long.

## Options

Each option is designed until its problems show.
The effort to build each is stated as a cost and is not what ranks it.

### A claim per add that every launcher takes

Each add takes a claim before it starts,
and the number of claims standing is the number of adds allowed.
The claim is one file made with `O_EXCL` or one atomic rename in a claims folder beside the worktrees,
named for the worktree it is for,
and removed by the add that made it or by the first command that finds its owner gone.
The figure comes from the claims standing,
not from a list of holders,
which is exactly how the pool gate counts slots:
`goingRunsNow` (`package/pi-shared/subagent-driver/src/going-runs.ts:342`) reads
`systemctl --user list-units --state=active,activating` and the launcher processes in `/proc`,
and takes each run's pool from the run's own command line.
An add with no claim free waits,
says how many claims stand and who holds them,
and starts when one is given up.

Where its problems show:

- The claim must be taken by every launcher,
  including a person's own `git worktree add`,
  or it bounds only the launchers that know about it.
  A claim taken inside the wrapper covers every caller and puts the wait in a program that today promises never to wait
  (`package/git-policy/cli/README.md`,
  "State on disk").
  A claim taken in the driver covers the driver's launchers and leaves the wrapper's promise alone.
- A stale claim bounds somebody else's work forever unless its owner can be proved gone,
  so it needs the same proof the wrapper's stage claims use,
  which is a real design,
  not a file and a counter.
- The figure is a rule about starts,
  which is the owner's to give.

What a person starting thirty rounds sees:
the batch command prints,
for each entry,
how many adds stand and which worktree holds each claim,
and entries start in list order at four at a time.
Thirty rounds then begin over about half an hour rather than in three minutes,
and no add is killed.

Pros:
it bounds adds whoever starts them,
lock-free,
with the counting rule the package already uses;
each add's wait is visible where it happens;
the figure sits in one file beside the measured record of why it is that figure.

Cons:
every launcher must take it,
so the wrapper or a shared package has to own it;
stale claims need a proof of death;
it adds a wait to a program whose documented promise is that adds never wait on each other;
and it queues work that the disk could otherwise overlap,
which a bound of four would make the common case.

### One queue that one command owns

`start-pi-runs-from-a-batch-list` (or a new command beside it) owns a queue,
and every start asks it for a turn.
The queue is one folder,
each waiting entry a file named for its order,
and the command moves one entry per free slot from waiting to running.
The precedent is the wrapper's own relink record,
which one commit service works through in order
(`package/git-policy/cli/README.md`,
"What the hand-over costs you").

Where its problems show:

- It bounds only what asks it.
  A start from another launcher,
  and a person's own `git worktree add`,
  is not in the queue,
  so the number in flight is bounded only if every launcher is made to ask,
  which makes it the same work as the claim with a different mechanism.
- The queue is a lock by another name,
  and `AGENTS.md` rule `NLK` allows one only over the smallest unit with its reason beside it,
  so the reason has to be a measured one.
- It moves the wait into a command a person runs,
  so a stuck entry stops the queue rather than one batch.

What a person starting thirty rounds sees:
one command,
one list,
the queue printing which entry runs and which waits,
and a number it was told to keep.

Pros:
the ordering is visible in one place,
the figure is one setting,
and the printed queue is the same shape as the `WAITING:` line the retrying start already prints.

Cons:
it is a lock over a whole repository's worktree adds,
which is the largest unit any rule in this repository allows one over;
it bounds only the launchers that use it;
and it duplicates the counting the pool gate already does.

### A bound on the add that grows with the adds in flight

The add's bound stops being a constant.
An add that begins beside N other adds gets a bound sized for N,
from the measured record,
and says at its end how many adds were in flight and what the host's I/O pressure was.
An add that would otherwise be killed is instead allowed to finish,
or killed with a line that names the N it saw.

Where its problems show:

- The figures above cannot size it:
  six adds in flight produced 153 s and 869 s adds,
  so a bound of f(N) would need a spread wide enough to be no bound.
- A bound that grows with N lets the worst case grow with N,
  which is precisely the case the issue records:
  27 adds would get a bound in the tens of minutes each,
  and thirty starts would still take that long.
- It does not reduce the work,
  and it does not stop the wave.

What a person starting thirty rounds sees:
nothing changes about how long the thirty take,
except that fewer are killed and more are named with the crowd they were in.

Pros:
it is the smallest change,
it keeps the add's promise of never waiting,
it makes an add's own line say what it was up against,
and it turns a killed add into an accounted one.

Cons:
it does not bound anything that a person can feel;
the figures cannot size it honestly today;
and a bound that grows with load is a rule that says the machine may be as slow as it likes.

### An add cheap enough to need no order

The copy is what costs,
so make the copy cheap,
and the question of an order stops mattering.
The wrapper already has such a route and takes it before any other when the checkout is the mount of the ZFS dataset named `monochromatic-git-checkout`:

```text
package/git-policy/cli/README.md, "State on disk"
On a checkout that is the mount of the ZFS dataset named `monochromatic-git-checkout`,
an add at `.claude/worktrees/<name>` takes a different route before any rung:
the slot is registered in Git first and locked with one sentence computed from the add's random nonce,
the checkout is snapshotted and cloned with `mountpoint=legacy`,
the clone is mounted at the slot, ...
```

That route does not copy the ignored state at all:
the clone is a snapshot of the whole checkout,
so the files the copy would move are already at the slot,
and the wrapper's own README says so of the snapshot rung beside it
(`package/git-policy/cli/README.md`,
rung 1 of the ladder:
"The snapshot shares every data block with its source and carries its source's whole ignored state (`node_modules` included),
so the ignored-state copy is skipped for it").
For the clone route,
the same README says the skip is not built yet
(`package/git-policy/cli/README.md`,
"The clone route's repair and its watches,
built and reached by no command":
"the skip of the separate ignored-file copy after a successful clone fill is not built yet,
so every rung still copies the ignored files"),
so that skip is part of what this option would have to have built.
What it costs is measured on this machine in
[the ZFS route record](cli-git-zfs-worktree.md),
"What was measured on this machine":
a snapshot of the checkout 0.036 s,
a clone 0.015 s to 0.017 s,
a mount 0.026 s to 0.029 s,
a plain `git checkout` of the 24,129 tracked files 2.44 s,
and a reflink copy of a 56,478-file worktree 5.63 s,
against 48.84 s for the add that does it the slow way.
The btrfs snapshot rung beside it measures 2 s to 3 s for the same command
([the ignored-copy record](cli-git-main-worktree-ignored-copy.md),
"The add itself").

This option does not redesign that route.
It names it and asks one thing:
whether this checkout should be that dataset's mount,
which is a change to the host and not a run's,
and what the route leaves open where the copy is skipped
(`package/git-policy/cli/README.md`,
"The clone route's repair and its watches,
built and reached by no command").

Where its problems show:

- It needs the host changed:
  the checkout must be the mount of that dataset,
  which is the coordinating session's decision and the owner's,
  not a run's.
- The wrapper's own route already declines on this checkout for two causes the journal recorded that morning,
  `unsupported: /home/user/Monochromatic is not a btrfs subvolume`
  and `git-running: index.lock exists in /home/user/Monochromatic/.git`,
  so a second cause has to be gone as well as the mount.
- The skip of the copy after a clone fill is not built,
  so today the route carries the ignored state twice over and its own copy still runs.
- It changes the repository's storage layout,
  which is the kind of change that cannot be made by one run.

What a person starting thirty rounds sees:
thirty adds of a few seconds each,
no wave,
no killed add,
and nothing to order.

Pros:
it removes the cost rather than queueing it,
which is the only option here that makes thirty starts take less time rather than more;
it removes the 39,692-entry walk and the stage folders that a kill leaves;
it is the wrapper's own route,
already designed,
with its measurements and its open pieces written down.

Cons:
it is a host change and a storage decision;
it needs the copy skip built;
and it leaves the general question open for a checkout that is not that dataset's mount,
including every host where the route is not available.

## Ranking

Ranking:
D > A > C > B.

D above A:
D takes the cost away,
so thirty starts take seconds each and no order is needed,
while A keeps the cost and makes thirty starts take half an hour.
D is the only option whose figure would not have to be argued:
it needs no cap,
so the question the owner refused on 2026-11-26 does not arise,
and every add's own record would show a few seconds.
A is second because it is the only option that bounds adds whoever starts them without a lock,
and because its figure can be argued from the record rather than made up.

A above C:
A bounds the number of adds,
which is the cause of the cost the figures show rising and the one quantity a caller can count;
C bounds the time each add may take,
which these figures cannot size,
since three adds that began within four seconds of each other took 302.8 s,
833.3 s and 833.5 s of copy,
so C would need a bound that is a function of a count it cannot weigh.
C also leaves thirty starts taking as long as the disk makes them take,
with more of them named rather than killed,
so it improves the report and not the wait.

C above B:
C changes one command's own bound,
keeps every launcher's promise that adds never wait on each other,
and leaves the choice of a figure visible in one place,
while B is a lock over the whole repository's adds,
which `AGENTS.md` rule `NLK` allows only over the smallest unit with its reason written beside it.
B bounds only the launchers that ask it,
so it does not bound adds "whoever starts them",
which is what the issue asks for,
and it is the same work as A with a heavier mechanism.

## Questions only the owner can answer

1. Is a rule about starts wanted at all,
   and if so at what figure:
   no cap,
   a cap of four,
   a cap of eight,
   or a cap the owner names?
   What each costs:
   no cap keeps the morning's shape and needs nothing built;
   a cap of four makes thirty starts take about seven waves of the measured copy time and needs option A built with stale-claim proof;
   the same holds for any other figure,
   and this round cannot name a defensible one.
2. Where does the order live,
   in the driver's launchers or in the Git wrapper?
   The driver's launchers cover every run this project starts and leave the wrapper's promise alone;
   the wrapper covers every caller,
   including a person's own `git worktree add`,
   and makes adds wait on each other,
   which its `README.md` says they never do.
   The first is a smaller change and the second is the only one that bounds "whoever starts them".
3. Is the second attempt worth keeping?
   Both attempts were killed in all sixteen failures,
   and a first attempt that has spent fifteen minutes beside twenty-four others has not been shown to do better on a second.
   A single attempt with a measured bound,
   or no bound at all with a line at the end,
   would have cost those sixteen starts one wave instead of two.
   What each costs:
   one attempt means a start that hits trouble fails sooner and is started again by hand or by list,
   which is what happened anyway;
   the two-attempt rule spends the same wait twice,
   and 30 s to 117 s between the attempts on removing what the first one left.
4. Does the owner want this checkout on the ZFS dataset mount the wrapper's fast route needs,
   with the copy skip built first?
   What it costs:
   a host change by the coordinating session,
   the copy skip built and measured,
   and a run's worth of design already written in `doc/planning/cli-git-zfs-worktree.md`
   and `doc/planning/cli-git-main-worktree-ignored-copy.md`.
   What it buys:
   the add costs seconds,
   so the number of adds in flight stops mattering.
5. Should an add that is killed at its bound say how many adds were in flight and what the I/O pressure was,
   whether or not any cap is adopted?
   What it costs:
   the wrapper or the launcher writes one more field into the record it already writes,
   and the driver's line names it.
   This is the smallest item here and the only one that needs no decision about a number.

## What this proposal does not do

- It builds nothing and changes no code and no test.
- It changes no bound,
  no stagger and no attempt count.
- It starts no add,
  and runs no experiment of overlapping adds,
  because that experiment is synthetic load.
- It does not decide the figures in question 1,
  which this round cannot measure from what is left.
- It leaves issue 6563 open,
  so that the owner's choice is the thing that closes it.
