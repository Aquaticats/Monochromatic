# Ordering the worktree adds of starts that come from several lists

## Status

Proposal,
written 2026-11-27T04:52:10-05:00 by the coordinating Claude Code session,
for [issue 6563](https://github.com/Aquaticats/Monochromatic/issues/6563).
Unapproved and non-final:
nothing here is built,
no code and no test changed,
and the owner decides what happens about starts.
This round measured and proposed only.

Two reviews of an earlier version are folded in.
The independent one,
from another vendor (`neuralwatt/nw-large`,
through the repository's advisor),
found the measured section overreaching,
and most of what changed since is its doing.
The other is a Claude Sonnet gate read,
a same-vendor second pass rather than an independent one:
it found the stage pairing wrong,
a claim about the sixteen restarted runs false,
and two of the issue's four owed items missing from the options.
Both sets of findings are in the document,
and where a finding changed a claim rather than a wording,
the text says so.

What is not known,
said again under "What the figures support,
and what they do not":

- the number of adds in flight at which an add starts to miss a bound of fifteen minutes,
  because the count of adds is one cause among others and the same shape of crowd spans
  22 s to 684 s of copy across one day on this machine;
- which part of the ignored-state copy holds the time,
  the walk of the source's tree,
  the staging of its entries,
  or settling their modes after they arrive;
- how far a killed add had got,
  because a kill writes no record of its steps;
- whether a cap of any figure would have held,
  because nothing in this round started or capped an add;
- the host's I/O pressure per add,
  which nothing sampled at the time.

## What the record already says about this

Three of the owner's own positions bear on the incident,
and none of them is an agent's reading:

- The hold of 2026-11-18,
  still unanswered in its record:
  "We shouldn't use worktrees heavily before we implement the zfs parts for the git wrapper."
  ([the hold](../decision/heavy-worktree-use-waits-for-the-zfs-route.md)).
  Asked later why worktrees were still made the old way,
  the owner answered "B:
  switch now",
  and the record of that hold says the wait is what the add pays for the checkout's ignored files,
  "which step 10's clone replaces".
- Up to 64 rounds at once,
  and,
  in the same record,
  "The owner set no start rule beside the 64."
  ([the decision](../decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md),
  "What it means").
- On waiting and locks,
  standing:
  "Use as few locks as possible.",
  "You should (almost) never be waiting.",
  "Avoid serializing embarrassingly parallel work."
  (quoted in [the ignored-copy record](cli-git-main-worktree-ignored-copy.md),
  "2026-11-03T02:42:59-05:00 The settlement lock is gone from worktree add").
  `AGENTS.md` rule `NLK` says the same of a lock:
  "an unavoidable lock covers the smallest unit,
  never a whole repository,
  with its reason written beside it."

So the incident is the shape the 2026-11-18 hold was about:
thirty rounds,
27 adds at once,
on an add whose wait is the per-entry copy of the checkout's ignored files.
The owner's recorded answer to that wait is the copy the ZFS clone replaces,
decided in outline on 2026-11-18
("A:
Own dataset (Recommended)",
[the ZFS route record](cli-git-zfs-worktree.md),
"Owner's answers of 2026-11-18")
and made the agent's act on 2026-11-26
([the host-setup decision](../decision/cli-git-zfs-host-setup-measurement-and-image-build-are-the-agents-act.md)).
What is left to decide is not whether that route is wanted but when the migration runs and what
holds until it is in place.

## What happens today

### A start waits on four things, and the add is only one

A start is refused or made to wait by four steps,
in this order:

1. the pool gate,
   which counts runs and refuses when every pool is full;
2. the question whether the `git` the session would run is this repository's wrapper,
   which is `monochromatic-git-policy-cli --help` bounded at ten seconds;
3. the worktree add,
   bounded at fifteen minutes and tried twice;
4. the run's unit and Pi's own start.

The second step fails starts of its own on a loaded host.
The issue's comment of 2026-11-27T08:53:41Z records two starts refused at 03:48 that day with
"`monochromatic-git-policy-cli --help` did not answer within 10000 milliseconds",
and three timings of that same call a minute later of 8.8 s,
9.36 s and 3.98 s,
against a 25 ms measurement on an idle host,
while 46 rounds ran and the load average read about 110 to 125.
Those forty-six rounds are the load the adds of this document met,
and that comment asks this proposal to count the wrapper's own start cost as part of what a start waits on.

### A list starts twenty seconds apart, and each list starts its own adds

`start-pi-runs-from-a-batch-list` pushes every entry's start and then waits,
so entries do not wait for each other:

```ts
// package/pi-shared/subagent-driver/src/batch-list.ts:20-23
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

The comment says why the stagger exists.
`README.md`,
"`start-pi-runs-from-a-batch-list`",
says only that entries start twenty seconds apart,
and nothing in the batch command knows another batch command exists.
A start can also spend time in the pool gate before it makes anything,
so twenty seconds apart is the starts' spacing and not the adds'.
That morning the adds began 0 s to 7 s after their start's own schedule,
and every one of the thirty was in flight within three and a half minutes of the first.

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
with the same fifteen-minute bound:

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
  (`worktree-of-a-run.ts:319` returns the last attempt);
- the branch the start made.
  The driver's `README.md`,
  "`start-pi-run-in-a-worktree`",
  says a branch the first attempt made "is removed too" when both attempts fail,
  and that is what the launcher tries.
  That morning it could not:
  every one of the sixteen failure lines ended
  "git refused to delete it (git branch --delete answered 1:
  error:
  cannot delete branch ... used by worktree at ...)",
  because the folder of the second attempt still had it checked out.
  So the record's expectation and the morning's outcome differ,
  and the folder is why;
- the stage folder `.claude/worktrees/.cli-git-worktree-copy-<id>`,
  which the wrapper's `README.md`,
  "State on disk",
  says the next command removes when it can prove it,
  and otherwise names and leaves whole.
  That morning's lines were the second kind:
  "was left by an interrupted `git worktree add` and was not changed:
  no record of the object its creation made stands beside its claim;
  remove it yourself once nothing needs it";
- the worktree's ignored state as a debt the wrapper records and later commands resume.

### Which step of an add waits

Issue 6563 quotes two wrapper lines that name steps,
one naming `git check-ignore` and one naming a bulk `git ls-files --others`,
and concludes the wait is after the copy the stage folder shows.
The wrapper's own record says otherwise about those two steps:

- `git ls-files` inside an add took 0.1 s to 159 s on this day,
  and for the adds of this incident 0.5 s to 3.1 s;
- `git check-ignore` took 0.1 s to 55 s that day,
  and for this incident's adds 0.1 s to 2.7 s;
- Git's own checkout of the 26,836 tracked files (the count in the `.out` files' progress lines)
  took 6.5 s to 35.8 s for this incident's adds;
- the ignored-state copy took 89% to 99% of each add that returned,
  and 40% to 96% of the adds of the whole day.

The wrapper's own `README.md`,
"What it does",
says what that copy is:
"the wrapper copies the ignored state of the source worktree into it without any lock",
and "State on disk" says it "clones files with a copy-on-write request,
in parallel on a large tree".
The checkout's ignored state was 39,692 entries that morning (issue 6563),
so thirty adds ask the same source for thirty copies of 39,692 entries,
and 27 at once ask for 1,071,684 clone and record steps against one tree.

The project measured what one copy of that tree costs at light load,
on 2026-11-23:
"the copy of 38,700 ignored entries 57.6 s (walk 3.1 s,
copy by eight workers 18.4 s,
settling and modes 33.2 s;
32,404 entries cloned)"
([the hold](../decision/heavy-worktree-use-waits-for-the-zfs-route.md),
"Measured on one real add after the switch").
So even alone an add on this checkout pays most of a minute,
and the settling of modes after the entries arrive is more than half of it.

## What was measured

### Where the figures come from

Everything here is read off what the machine left on 2026-11-27.
No add was started for this measurement,
and no cap,
delay or order was tried,
because an experiment of overlapping adds is synthetic load and this round's rules forbid it.

- `journalctl --user` for the three batch units of 02:37 and the restart unit,
  bounded by `--since`,
  `--until` and one unit at a time;
- the three batch lists' `.out` files under `~/temp/agent/pi-runs`;
- the run folders under `~/temp/agent/pi-runs`:
  a start makes its run folder 8 ms to 97 ms after its add returns,
  so a run folder's birth is when its add had ended;
- the birth times of the worktree folders `.claude/worktrees/pi-*`,
  which is when Git created the destination,
  0.3 s to 8 s after the add began;
- the birth times of the stage folders `.claude/worktrees/.cli-git-worktree-copy-*`;
- the wrapper's copy records in the checkout's Git directory,
  `monochromatic-git-policy-cli-worktree-copy/v1`,
  whose `.add` and `.unfilled` files name the worktree each copy was for and why one did not finish;
- the wrapper's per-add timing record,
  `<the checkout>/.git/monochromatic-git-policy-cli-worktree-add-timing.jsonl`,
  which holds one line per add that exited with the milliseconds of each step.
  It rotates by size,
  so a reader who comes later may find these lines gone;
  what this record uses from it is quoted here.

The limits of those sources:

- an add killed at its bound writes no timing line:
  the record holds a line for each of the fourteen adds of this round that returned and one for none of the forty killed.
  So nothing here knows how far a killed add had got;
- the birth time of a worktree folder is exact,
  and a first attempt's folder is deleted before its second attempt's is made,
  so for a worktree made twice the standing folder is the second attempt's;
- for the twenty-four first attempts that were killed,
  the folder is gone and the add's start is the batch's schedule,
  to within a few seconds;
  for the twenty-four second attempts the folder's birth less 0.3 s places the add.
  Every in-flight count beside a killed add carries that;
- the wrapper's copy records name worktrees and reasons exactly,
  and say nothing about when a copy started or how far it got before a kill.

### What one add cost that day, and what it met

The wrapper's record for 2026-11-27 holds seventy lines.
Every one of them is an add that returned,
and beside each the copy's share of it.
Read in the order the adds began:

- 01:26 one add,
  36 s;
- 01:32 one add,
  110 s;
- 01:45 one add,
  67 s;
- 01:50 three adds that began together,
  28 s each,
  the copy 22 s of it;
- 01:51 two adds,
  83 s;
- 01:53 three adds that began together,
  61 s each;
- 02:05 and 02:06 one add each,
  28 s and 27 s;
- 02:11 six adds that began together,
  98 s to 104 s each,
  the copy 94 s to 99 s;
- 02:21 six adds that began together,
  145 s to 152 s each,
  the copy 137 s to 140 s;
- 02:37 to 02:56 the incident's own fourteen,
  154 s to 897 s;
- 03:11,
  03:18 and 03:19 five adds in the 360 s and 716 s to 746 s band,
  with Git's own checkout 200 s to 259 s of the four at 03:18,
  and the copy 62% to 70% of them;
- 03:30 and 03:31 five adds that began together,
  659 s to 795 s each,
  the copy 536 s to 684 s of it,
  with no more than those five of the record's own adds overlapping,
  and forty-six rounds of other work running (the issue's comment);
- 04:10 one add,
  515 s,
  the copy 448 s of it.

Two things follow from that ordering,
and the second is the one this proposal turns on:

- the copy is the add at nearly every hour of the day,
  from 40% of an add at 02:27 to 99% of one at 02:56;
- the count of adds is one cause of its cost and not the dominant one.
  Three adds that began together at 01:50 cost 22 s of copy;
  three that began together at 02:37 cost 143 s to 145 s;
  five that began together at 03:30 cost 536 s to 684 s,
  with the machine carrying forty-six rounds.
  The same shape of crowd spans a factor of thirty across the day,
  and what moved with it is the work the rest of the machine was doing.

### The thirty starts of 02:37

- Three lists of 11,
  8 and 11 entries,
  units started at 02:37:23.115,
  02:37:24.174 and 02:37:25.317.
- Their first adds began at 02:37:29.45,
  three of them within 42 ms of each other,
  and the thirtieth began at about 02:40:45 (its list's schedule,
  to within a few seconds).
- The most adds in flight at one moment:
  27,
  from about 02:40:45 to 02:43:24.
  Then 26 until 02:52:14.
- First attempts:
  6 returned,
  taking 153.6 s,
  154.0 s,
  154.7 s,
  339.5 s,
  865.9 s and 869.5 s;
  24 were killed at 900 s.
- Those 24 first attempts began between 02:38:03 and 02:40:45,
  with 7 to 27 adds in flight;
  none of them returned.
- Second attempts:
  24 began between 02:54:52 and 02:56:27,
  25 s to 124 s after their own first attempt was killed,
  a gap holding the removal of what that attempt left and the start of the next;
  8 returned,
  taking 776.7 s to 896.8 s,
  and 16 were killed at 900 s.
- The sixteen that failed both attempts were started again at 03:12:25 from one list,
  which needed no add at all,
  because each worktree was already there,
  and their runs began between 03:13:10 and 03:17:29.
- At 04:2x,
  13 of the round's 30 worktrees still lacked the checkout's ignored state:
  eleven of the sixteen restarted ones and two that had started on their second attempt.
  Their records give two causes:
  "the ignored-state copy did not finish within the 360-second ceiling one command holds for a copy it did not start",
  and "ignored-state copy would overwrite differing destination entry" naming
  `.cache/cargo-shadow/stamps/package__git-policy__cli.json`,
  `.cache/cargo-shadow/lock` and `.cache/monochromatic-git-policy-cli-bin/git`.
  So the issue's "the wrapper makes the owed copy at the first command in each" is true of the attempt and
  not of the outcome:
  the attempt was made and did not finish.

### Each add, and what was in flight during it

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
not a measurement of how long the work needed.
The times of an add that returned are the wrapper's own record of it,
exact;
the times of an add killed at its bound are the batch's schedule and the bound,
to within a few seconds:

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

The eight second attempts that returned all ended between 03:09:20 and 03:10:36,
five of them 6 s to 8 s after the sixteenth killed add hit its bound,
and they had been running for thirteen to fifteen minutes when they ended.

### The stage folders, and what they say about the copy

The wrapper's copy records name the worktree each stage was filling,
so the pairing is read rather than guessed.
That corrects an earlier reading of this document:

- the four stages born at 02:39:39 were filling 6541,
  6540,
  6534 and 6536,
  which were not the three adds that began at 02:37:29 but four whose first attempts had begun
  56 s to 76 s earlier;
- the five born at 02:56:15 were filling 6548,
  6537,
  6542,
  6546 and 6536,
  second attempts that had begun 55 s to 73 s earlier;
- for the adds that began at 03:30:37,
  the stage folder appeared 2 s after their copies began,
  and their copies ran for 536 s to 684 s.

So a stage folder appears somewhere between 2 s and 70 s into a copy,
and most of a copy's time passes after its stage exists.
The wrapper's own split of a copy at light load agrees:
of 57.6 s,
the walk was 3.1 s,
the copy by eight workers 18.4 s,
and settling and modes 33.2 s.
Which of those the crowd stretches is not known,
and the record does not separate staging from settling.

At 03:46:25,
19 stage folders stood in `.claude/worktrees`,
nine of them born inside the incident's window:
four at 02:39:39,
and five of the six born at 02:56:15.
By 04:2x none stood:
each was removed when its copy finished or was settled,
and the worktrees whose copies failed carry an `.unfilled` record instead.

### What the figures support, and what they do not

Supported:

- An add's own Git checkout is a small part of it,
  and the copy is nearly all of it,
  in every add that returned.
- The copy's cost rose through that day by far more than the count of adds rose:
  22 s for three adds that began together at 01:50,
  94 s to 99 s for six at 02:11,
  137 s to 140 s for six at 02:21,
  143 s to 145 s for three at 02:37,
  and 536 s to 684 s for five adds that began together at 03:30,
  while the machine carried forty-six rounds of other work.
- Every first attempt that began with seven or more adds in flight that morning was killed at its bound;
  the six that returned had all begun with one to six.
  That is one morning's line,
  not a threshold.
- The second attempt rescued eight starts and lost sixteen;
  the eight it rescued took 776.7 s to 896.8 s,
  meeting the second wave's crowd of up to 24 adds rather than the first wave's 27.
- An add that hits its bound leaves its folder,
  its branch and its stage behind,
  writes no timing record,
  and the sixteen failures cost the two waves plus the restart list,
  whose sixteen runs began within five minutes of its start.
- The copy's cost is not a per-add constant:
  three adds that began within four seconds of each other took 302.8 s,
  833.3 s and 833.5 s.

Not supported:

- A figure at which an add starts to miss its bound.
  The counts rose while every add ran,
  the same group spread over a factor of 2.75,
  and the same shape of crowd spans 22 s to 684 s across the day.
  A bound taken from these figures would be a guess,
  which is the thing the owner's words of 2026-11-26 refuse for a rule about starts.
- That the count of adds is what sets an add's cost.
  Five adds at 03:30 cost 536 s to 684 s of copy with forty-six rounds of other work running,
  and three adds at 01:50 cost 22 s.
  A cap on adds bounds one of the causes the figures show and not the one that moved most.
- Any claim that a cap of N would have held.
  No capped run was measured,
  and a cap moves the queue from the disk to the launcher without making the copy cheaper:
  thirty adds at four at a time still move 1,190,760 entries,
  and the last of them starts after the first has finished rather than beside it.
- Which part of the copy dominates,
  and what a killed add had done when the bound ended it.
  A kill writes nothing,
  so the issue's reading of the stage folders and this document's reading of the timing record cannot be reconciled.
- The in-flight figures of a killed add to better than a few seconds.
  An add that returned is placed by the wrapper's own record to the millisecond;
  a killed add's first attempt is placed by its list's schedule.
- What the host's I/O pressure was per add.
  The only readings are the issue's own (`some avg10=75.07` at 02:44),
  and nothing sampled per add.

### What this document corrects in the issue

- Twenty-four first attempts were killed,
  not thirty:
  six returned,
  three of them in 153.6 s to 154.7 s and three in 339.5 s to 869.5 s.
- The adds in flight read 26 at 02:44 and 24 at 03:03 by this reconstruction,
  against the issue's own `ps` readings of 27 and 26.
  The difference is what those readings counted and this one cannot:
  a `git worktree remove --force` of a killed attempt's leftovers copies and deletes as much as an add,
  and this reconstruction counts adds only.
- "The wrapper makes the owed copy at the first command in each" is right about the attempt and
  wrong about the outcome:
  13 of the round's 30 worktrees still lacked the ignored state at 04:2x,
  with the 360 s ceiling and "would overwrite differing destination entry" in their records.

### What this round's evidence outlives

The journal read,
the stage and copy records,
the birth times and the analysis scripts are in this round's scratch folder,
which expires on 2026-11-28.
What must outlast them is in this document,
and the wrapper's timing record is the one durable source,
rotating by size.

## What the issue owes, and what each thing needs

The issue lists four things as owed.
Only the first is about ordering,
and the other three are smaller and separate:

- shared order for adds,
  bounded by a measured figure.
  The options below are about this,
  and the figures above are what can be measured from what is left.
- an add ended at its bound says how many adds were in flight and what the host's I/O pressure was.
  A killed add writes no record,
  so the sentence has to come from the driver,
  which sees the add return or be killed and can count the adds in flight and read `/proc/pressure/io` at that moment.
  Nothing in the package does this today,
  and it needs no figure decided.
- a start that fails after its attempts leaves no stage behind,
  or says which stages it left.
  The wrapper already names a stage it cannot prove and leaves it;
  today's lines said no creation record stood beside the claim,
  so the naming worked and the leaving was what the wrapper chose.
  The smaller change is for the launcher to remove what its own last attempt left,
  which its `git worktree remove --force` already does between attempts and could do after the last.
  The wider one is for the wrapper to keep a record that proves the stage,
  which the line itself names as the missing piece.
- a control with a stand-in `git` that takes longer when more are in flight.
  No such control exists,
  and it would prove a bound only once a bound exists.
  A control could equally show the opposite,
  which is what this round's figures do:
  an add lengthens with what the machine is doing,
  and the count of adds is one part of it.

## Options

Each option is designed until its problems show,
with its pros,
its cons,
and what a person starting thirty rounds would see.
Effort to build is a cost stated,
and not what ranks it.

### Option A: an add cheap enough to need no order

Make the add a clone of the checkout instead of a checkout plus a copy of the ignored files.
The wrapper already has that route and takes it before any other when the checkout is the mount of the ZFS dataset named `monochromatic-git-checkout`:

```text
package/git-policy/cli/README.md, "State on disk"
On a checkout that is the mount of the ZFS dataset named `monochromatic-git-checkout`,
an add at `.claude/worktrees/<name>` takes a different route before any rung:
the slot is registered in Git first and locked with one sentence computed from the add's random nonce,
the checkout is snapshotted and cloned with `mountpoint=legacy`,
the clone is mounted at the slot, ...
```

The clone is a copy of the whole checkout,
so the ignored files are at the slot from the start and nothing walks them one entry at a time.
This option does not redesign that route.
It names it and asks two things:
when the migration that makes `~/Monochromatic` its own dataset runs,
and what is relied on until then.

Where its problems show:

- The migration stops every session in the checkout,
  so it runs at a moment the owner picks,
  and the decisions of 2026-11-18 and 2026-11-26 already make it the agent's act.
  Until it runs,
  this option delivers nothing.
- The route's remaining pieces are named,
  not built:
  `SPEC.md` says step 19's files are installed by no release and that unmounting at shutdown is unbuilt,
  and the wrapper's `README.md`,
  "The clone route's repair and its watches,
  built and reached by no command",
  says "the skip of the separate ignored-file copy after a successful clone fill is not built yet,
  so every rung still copies the ignored files".
  So the copy this document measured would still run after the route serves the add,
  until that skip lands.
- The 0.036 s snapshot and 0.015 s clone in [the ZFS route record](cli-git-zfs-worktree.md)
  were measured on a throwaway dataset holding the tracked files only,
  and not on this checkout with its 39,692 ignored entries;
  the plan's step 22 still owes the real-pool figure.
  What is measured on this checkout's copy is the 57.6 s at light load above,
  and what the clone would make of it is a design claim until step 22 runs.

What a person starting thirty rounds would see:
thirty adds of seconds each,
in any order,
with no wave to wait out and nothing to queue.

Pros:
it removes the cost instead of ordering it,
which is the only option here that makes thirty starts faster;
it is the owner's own answer to the wait,
recorded twice;
it removes the stage folders and the owed copies a kill leaves;
and it needs no figure for a start rule,
so the question the owner refused on 2026-11-26 does not arise.

Cons:
it waits on a host migration and on two pieces not yet built;
its speed on this checkout is unmeasured until step 22 runs;
and it does nothing for a checkout that is not that dataset's mount.

### Option B: the copy off the add's waiting path

Leave the add its checkout and let the copy be somebody else's work.
The add returns once Git's own checkout is done,
records what it owes,
and the copy runs from a later command in that worktree,
which the wrapper already does for a copy a kill interrupted,
or from the commit service,
which already runs the pnpm relink from a record after the add returns.
The project has walked this way twice before,
in [the package-repair record](cli-git-worktree-add-package-repair-off-the-waiting-path.md)
and [the stand-ins record](cli-git-worktree-add-stand-ins-off-the-waiting-path.md).

Where its problems show:

- The run starts without its dependencies.
  That is not hypothetical:
  at 04:2x,
  13 of this round's 30 worktrees lacked the ignored state,
  and their runs were working in them.
  A run whose first build needs `node_modules` pays the copy itself,
  under its own wall clock,
  which is the failure the package's own `README.md` names as open
  ("a run whose first commands need `node_modules` may read it while it is being rewritten").
- The copy then competes with the builds it was meant to unblock,
  so the machine's I/O does not fall,
  it moves.
- Something has to decide who runs the copy and when it is abandoned;
  the wrapper's 360-second ceiling for a copy one command holds is what failed those 13,
  so the ceiling has to be part of the design.

What a person starting thirty rounds would see:
every start returns in the time of Git's own checkout,
6.5 s to 259 s that day,
and each run's first minutes carry a copy it may not need yet.

Pros:
it needs no host change and no figure;
it is the smallest step toward option A's shape,
since it moves the same work off the add;
and it removes the fifteen-minute bound as the thing that decides a start's fate.

Cons:
the wait moves into the runs,
where it is bounded by a run's wall clock instead of the add's;
it makes a run's own report carry a failure it did not cause;
and the 13 unfilled worktrees show what an unfinished copy costs a run today.

### Option C: accounting at the bound, and a bound that grows with the crowd

Keep the add's shape and change what its end says and how long it may take.
An add that ends at its bound writes how many adds were in flight and what `/proc/pressure/io` read,
and the bound itself becomes a function of that crowd rather than a constant fifteen minutes.

Where its problems show:

- The figures cannot size the function:
  the same group of three spread over 302.8 s to 833.5 s,
  and five adds at 03:30 took 536 s to 684 s of copy.
  A bound that grows with the count grows by a number nobody can state.
- A bound that grows with the load says the machine may be as slow as it likes,
  which is the shape of rule the owner refused.
- It reduces no work and stops no wave;
  it turns a killed add into an accounted one.

What a person starting thirty rounds would see:
about as long a wait as that morning,
with fewer kills and a line on each failed start naming the crowd and the pressure.

Pros:
it is the smallest change here,
it needs no figure decided if only the reporting is taken,
it keeps every promise about not waiting,
and it is the one option the figures directly support,
because the reporting needs no sizing at all.

Cons:
nothing a person feels gets faster;
the growing bound is a guess dressed as a function;
and a killed add still leaves its folder,
its branch and its stage.

### Option D: a claim per add that every launcher takes

Every add takes one of a fixed number of numbered slots before it starts,
and the number of slots is the bound.
The claim is a file made with `O_EXCL` or an atomic rename in a folder beside the worktrees,
named for its slot rather than for its worktree,
because a name per worktree never collides and bounds nothing;
an add that finds every slot taken recounts and waits.
The count is readable without a lock,
which is the shape the package already uses for pools:
`goingRunsNow` (`package/pi-shared/subagent-driver/src/going-runs.ts:342`) builds its list of holders
from `systemctl --user list-units --state=active,activating` and the launcher processes in `/proc`,
and reads each run's pool from the run's own command line.
A launcher could equally count live `git worktree add` processes,
which needs no claim file and no stale-claim proof.

Where its problems show:

- It must be taken by every launcher,
  including a person's own `git worktree add`,
  or it bounds only the launchers that know about it.
  Taken in the wrapper,
  the wait lands inside the driver's fifteen-minute bound,
  so a queued add can be killed for waiting;
  taken in the driver,
  it covers the driver's launchers only,
  and a launcher waiting for a slot still holds its pool slot.
- A stale claim bounds somebody else's work forever unless its owner can be proved gone,
  so it needs the proof the wrapper's stage claims use,
  which is a real design and not a file and a counter.
- It bounds a factor the figures show is not the dominant one:
  five adds took 536 s of copy while forty-six rounds ran.
- The figure is a rule about starts,
  which is the owner's to give,
  and the owner's standing words are "You should (almost) never be waiting" and
  "Avoid serializing embarrassingly parallel work".

What a person starting thirty rounds would see:
the batch command printing how many slots stand and which worktree holds each,
entries starting in list order four at a time,
and thirty rounds beginning over eight rounds of whatever the copy costs at four.

Pros:
it bounds adds whoever takes it,
lock-free if the slots are numbered,
with a counting shape the package already uses;
each add's wait is visible where it happens;
and the figure sits in one file beside the measured record of why it is that figure.

Cons:
every launcher must take it,
so the wrapper or a shared package has to own it;
stale claims need a proof of death;
it adds a wait to a program whose documented promise and whose owner's words are that adds do not wait on each other;
and it bounds one cause of the cost among several.

### Option E: one queue that one command owns

One command owns a queue,
and every start asks it for a turn.
The queue is one folder,
each waiting entry a file named for its order,
and the command moves one entry per free slot from waiting to running,
the way a commit service works through the records left for it.

Where its problems show:

- It bounds only what asks it,
  so a start from another launcher,
  or a person's own `git worktree add`,
  is outside it;
- the queue is a lock over a whole repository's adds,
  and `AGENTS.md` rule `NLK` allows one only over the smallest unit,
  "never a whole repository",
  so this option is forbidden as the rules stand unless the owner changes that rule;
- a stuck entry stops the queue rather than one batch.

What a person starting thirty rounds would see:
one list,
one queue,
a printed position for each entry,
and a number it was told to keep.

Pros:
the ordering is visible in one place,
the figure is one setting,
and the printed queue has the shape of the `WAITING:` line the retrying start already prints.

Cons:
it is a repository-wide lock;
it bounds only the launchers that use it;
and it duplicates the counting the pool gate already does.

## Ranking

Ranking:
A > B > C > D > E.

A above B:
A removes the copy and B only moves it.
The copy is 89% to 99% of an add that returned and 40% to 96% of every add the day recorded,
so an add that keeps it with the run pays the same entry-by-entry cost,
just under a run's wall clock instead of the add's bound,
and 13 worktrees of this round show what an unfinished copy leaves a run.
A also removes the stage folders and the owed copies that a kill leaves behind,
which B multiplies.

B above C:
B takes the fifteen-minute bound out of the start's fate,
which is the failure the issue records,
where C leaves the add as long as the disk makes it and only stops the killing.
B needs no figure,
and the figures show that no figure can be named for C's growing bound:
five adds took 536 s to 684 s of copy with forty-six rounds running.

C above D:
C changes no one's waiting and needs no figure if only the reporting is taken,
and its reporting is what the other options would need to prove themselves.
D makes every add wait on a count,
which contradicts the owner's standing words about waiting and serialising parallel work,
and it bounds a cause the figures show is not the dominant one:
three adds at 01:50 cost 22 s of copy and five at 03:30 cost 536 s,
so a bound of four adds would not have made that morning's copies cheap.

D above E:
D bounds adds lock-free over a small unit,
while E is a repository-wide lock that `AGENTS.md` rule `NLK` forbids as the rules stand,
and both cover only the launchers that know about them.

The order is by what each delivers for someone starting many rounds on a loaded host.
A delivers the most and waits on a host migration;
B delivers most of A's effect today and moves the cost into the runs;
C delivers accounting and no speed;
D and E deliver a queue and a wait.

## Questions only the owner can answer

1. Is any rule about starts wanted,
   or is the answer the hold of 2026-11-18 read again:
   no heavy worktree use until the ZFS route serves the add?
   What each costs:
   no rule keeps the morning's shape,
   including sixteen failures and 13 worktrees without their ignored state;
   a rule needs a figure nobody can measure,
   as "What the figures support" sets out,
   and a measurement before the rule would mean starting overlapping adds on purpose,
   which is synthetic load and needs the owner's word to run.
2. When does the checkout's migration to its own dataset run,
   and what holds until then?
   The decision of 2026-11-18 is the owner's and the act is the agent's;
   the migration stops every session in the checkout,
   so the moment is the owner's.
   What it costs:
   a stop of every session in the checkout,
   a move of about 5.5 GiB without the worktrees and 32.6 GiB with them,
   and the copy skip built after it.
   What it buys:
   option A,
   and the end of the wait this document measured.
3. Is the second attempt worth keeping?
   It rescued eight of the sixteen that would otherwise have failed,
   and it cost the other sixteen the same wait twice plus 25 s to 124 s of removal between the attempts.
   What each costs:
   keeping it fails 16 of 30 that morning instead of 24;
   dropping it fails 24 and saves the removal and the second wait.
4. Should the copy move off the add before the migration,
   as option B,
   and how long may a copy one command did not start hold on,
   given that the 360-second ceiling is what failed 13 of these worktrees?
   What it costs:
   a design for who runs the copy and when it is abandoned,
   and runs that start without their dependencies but carry the copy in their own minutes.
5. Should an add that is killed at its bound say how many adds were in flight and what the I/O pressure was,
   whether or not any rule follows?
   The driver is where that line can be written,
   because a killed add writes nothing of its own.
   What it costs:
   one counting mechanism in the driver and one field in the line it already prints,
   and it needs no figure decided.

## What this proposal does not do

- It builds nothing and changes no code and no test.
- It changes no bound,
  no stagger and no attempt count.
- It starts no add,
  and runs no experiment of overlapping adds,
  because that experiment is synthetic load.
- It does not decide any figure in the questions above.
- It leaves issue 6563 open,
  so that the owner's choice is the thing that closes it.
