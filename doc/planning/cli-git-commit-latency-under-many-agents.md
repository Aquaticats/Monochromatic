# Commit latency with sixteen agents on one machine

What this answers:
which of the wrapper's own steps carry the slow tail of a commit
when sixteen agents share one node,
measured from the records the wrapper already writes,
and what was changed because of it.

The bar is the owner's:
"commits vs native git should not take 5s longer."
An agent ran `mise run //package/git-policy/cli:native:perf` three times with sixteen agents working:
two runs put commits a maximum of 5437 ms and 10 801 ms over native Git,
the first put the maximum at 1220 ms,
an unchanged tree and the branch under test both broke the bar,
and two runs of one binary differed by 2.6 times,
so those runs could attribute nothing.
This record reads the per-commit records instead,
which cost no load,
and then measures the one wait it explains.

## Where the numbers come from

Two records the wrapper writes anyway:

- `.git/monochromatic-git-policy-cli-commit-timing.jsonl`
  in the common Git directory of this repository,
  with its one generation `.1`,
  holds one line per wrapped commit:
  each phase,
  every Git step the client started with its offset and duration,
  and the quiet ledger.
  The fields are described in `package/git-policy/cli/SPEC.md`
  ("The longest quiet gap of every Git step and tool a command ran").
- The commit service's journal,
  lines starting `answered`,
  one per landing the service took:
  the queue,
  the wait for the index lock,
  how long the lock was held,
  the objects,
  the journal,
  the branch move,
  the index update,
  the landing and the answer,
  and the Git steps of that landing
  (`package/git-policy/cli/src/native/service/service_timing.rs`).

The window is the day ending 2026-11-09T03:33:34Z,
which holds 866 commits,
and the same day holds 891 landings in the journal.
Every figure in this record is that snapshot;
the log file was copied aside before reading,
because the machine keeps committing while this is read.

The machine at the time was under pressure of the kind other sessions keep up
(the owner's rules hold that this is normal and never an excuse):
load average 27 to 62 on 6 CPUs,
and `/proc/pressure/io` reporting `full` at 36 percent over ten seconds
and 23 percent over five minutes,
while a `repository-move-cutover` unit copied a disk.
Every measurement this record reports alternates its arms and reports its counts,
no synthetic load was generated,
and the probes ran in `systemd-run --user --scope` scopes.

## The steps, by what they cost

Client steps,
one entry per Git step of the commit record,
with its occurrences,
median,
the slowest 1 in 100,
the maximum,
and its total over the window:

- `git commit`:
  837 occurrences over 820 commits,
  median 42 ms,
  p99 4169 ms,
  max 171 383 ms,
  400 437 ms in all.
  This is Git building the commit in the scratch repository,
  signing included,
  and the only step that a run of retries repeats.
- `git add`:
  848 over 848,
  median 37 ms,
  p99 1783 ms,
  max 10 567 ms,
  116 527 ms.
- `git read-tree`:
  267 over 261,
  median 52 ms,
  p99 3272 ms,
  max 5909 ms,
  51 340 ms.
- `git update-index`:
  391 over 353,
  median 14 ms,
  p99 660 ms,
  max 3202 ms,
  18 932 ms.
- `git rev-parse`:
  2300 over 866,
  median 2 ms,
  p99 10 ms,
  max 1607 ms,
  9917 ms.
- `git symbolic-ref`:
  1674 over 820,
  median 2 ms,
  p99 14 ms,
  max 1795 ms,
  6939 ms.
- `git update-ref`:
  837 over 820,
  median 3 ms,
  p99 15 ms,
  max 2350 ms,
  5530 ms.
- `git for-each-ref`:
  837 over 820,
  median 4 ms,
  p99 55 ms,
  max 142 ms,
  4444 ms.
- `git cat-file`:
  826 over 809,
  median 2 ms,
  p99 5 ms,
  max 7 ms,
  1905 ms.
- `git write-tree`:
  22 over 16,
  median 9 ms,
  max 667 ms,
  1427 ms.
- `git diff-index` 188 ms,
  `git merge-tree` 149 ms,
  `git diff` 58 ms,
  `git hash-object` 51 ms,
  `git config` 31 ms,
  and three steps under 15 ms each,
  in all.

Every client step is on the commit's path:
the command waits for each one before it answers.

The service starts its own Git steps for a landing,
and they are small:

- `git maintenance`:
  713,
  median 4 ms,
  p99 245 ms,
  max 1797 ms.
- `git update-ref`:
  875,
  median 3 ms,
  p99 116 ms,
  max 1837 ms.
- `git show`:
  1046,
  median 4 ms,
  max 75 ms.
- `git config`:
  713,
  median 3 ms,
  max 185 ms.
- `git rev-parse`:
  674,
  median 3 ms,
  max 11 ms.
- `git rev-list`:
  38,
  median 4 ms,
  max 647 ms.
- `git cat-file`:
  349,
  median 3 ms,
  max 16 ms,
  and the rest under 60 ms each.

So the Git children are not the tail:
one commit's steps add up to a median of 153 ms,
while its whole record has a median of 1172 ms.

## The commit, phase by phase

The phases are disjoint stretches in this order and sum to `totalMs`:

- `prepareMs`:
  median 450 ms,
  p99 13 576 ms,
  max 25 626 ms,
  total 1 375 619 ms,
  26.1 percent of all time.
- `buildMs`:
  median 109 ms,
  p99 26 947 ms,
  max 493 699 ms,
  total 2 724 762 ms,
  51.7 percent.
- `submitMs`:
  median 87 ms,
  p99 13 659 ms,
  max 37 239 ms,
  total 625 227 ms,
  11.9 percent.
- `gateMs`:
  median 30 ms,
  p99 3733 ms,
  max 8687 ms,
  total 176 889 ms,
  3.4 percent.
- `otherMs`,
  what no phase covers:
  median 3 ms,
  p99 7702 ms,
  max 58 733 ms,
  total 358 549 ms,
  6.8 percent.
- `provisionMs` 6637 ms,
  `afterLandingMs` 1435 ms,
  `handoffMs` 3310 ms in all.

The whole commit:
median 1172 ms,
p99 49 766 ms,
max 494 667 ms.

## What the steps wait on

A phase is not a wait and a wait is not a phase,
so the question is what each phase holds that no Git step covers.
The phases are laid out in the record's own order,
each gap between two steps is split across the phases it overlaps,
and what is left in a phase is the time the command spent there without a Git child:

- `buildMs`:
  median 24 ms,
  p90 6231 ms,
  p99 23 386 ms,
  max 493 591 ms,
  total 2 503 307 ms,
  53.7 percent of all uncovered time.
  186 commits hold more than 1000 ms of it and 103 more than 5000 ms.
  Two commits hold 845 s of the total between them
  (2026-11-08T23:26:54.998Z at 493 591 ms and
  2026-11-08T23:26:55.001Z at 351 540 ms);
  without them the total is 1 658 176 ms.
- `prepareMs`:
  median 332 ms,
  p90 3873 ms,
  p99 11 884 ms,
  max 25 460 ms,
  total 1 009 772 ms,
  21.7 percent.
  256 commits hold more than 1000 ms of it and 50 more than 5000 ms.
- `submitMs`:
  median 87 ms,
  p90 973 ms,
  p99 13 531 ms,
  max 37 239 ms,
  total 623 728 ms,
  13.4 percent.
  86 commits hold more than 1000 ms of it and 36 more than 5000 ms.
- `otherMs`:
  median 3 ms,
  p99 7393 ms,
  max 58 733 ms,
  total 329 506 ms,
  7.1 percent.
  52 commits hold more than 1000 ms of it and 14 more than 5000 ms.
- `gateMs`:
  median 30 ms,
  p99 3733 ms,
  max 8687 ms,
  total 176 838 ms,
  3.8 percent,
  39 commits over 1000 ms.
- Everything else:
  11 372 ms.

The commit's whole uncovered time:
median 957 ms,
p90 13 340 ms,
p99 43 174 ms,
max 494 448 ms,
4 656 792 ms in all,
3 809 938 ms without those two commits.

`buildMs` holds three kinds of work and only one of them starts a child Git:
the shadow repository is created and its entries linked,
the conclusion state is copied into it and into the transaction,
and then Git builds the commit there (`git commit`,
recorded).
The copy is where a durable write happens:
the preparation published `conclusion/store-held.json` with a file sync,
one flush per commit.
Measured in a throwaway repository of this tree's shape,
`strace -f -T` put one such `fsync` at
24.9 s,
2.5 s,
1279 ms,
386 ms,
179 ms and 8 ms across commits,
and it sat in the same uncovered stretch of `buildMs`
that the live records show at 1.3 s,
2.5 s,
5.6 s and,
twice,
at 351 s and 493 s.

The preparation's uncovered time is the policy engine's own work:
the same shape took 3.7 s of wall time with 0.076 s of user
and 0.242 s of system time,
so it is waiting on this host rather than computing,
and the Git children of the preparation are recorded separately
(`git add`,
`git read-tree`,
`git update-index`).
It is not the markdown linter:
commits whose linter ran have a median prepare gap of 330 ms and those without 309 ms.

The service's share is what one landing does that no step covers,
plus what the commit waits for in the queue:

- residual `reads_ms` (the branch reads and the comparisons,
  `LandTiming::reads`):
  median 6 ms,
  p99 9047 ms,
  max 37 068 ms,
  293 578 ms in all.
- `queued_ms`:
  p99 2170 ms,
  max 24 726 ms,
  85 627 ms in all.
- the wait for a foreign holder of the index lock:
  p99 342 ms,
  max 11 490 ms,
  in 136 of the 891 landings.
- the lock's hold while the branch moves and the index is built:
  median 51 ms,
  p99 3554 ms,
  max 8101 ms.
- `index_update_ms`:
  median 44 ms,
  p99 3247 ms,
  max 7768 ms,
  127 151 ms in all.

The client never waited on that index lock in this window:
`lockHolderKind` is null in all 866 records.

What the `otherMs` share holds is what a command does outside those phases:
the removal of the scratch tree when the command ends
(one `unlinkat` of 2.5 s in a shaped fixture),
the wrapper writing what the person reads on standard error,
whose write waits for its reader,
and the start of the command before its first phase.

Which side each comes from:

- From this wrapper:
  the conclusion copy's flush,
  the copy and read of the whole index into a private policy index,
  the removal of the scratch tree on the way out,
  the landing's index update and its hold,
  and the in-process policy work of the preparation.
- From the host:
  how long any of those take under the disk and CPU pressure other sessions keep up,
  and how long the signing program and the programs holding the index lock take.
- Baseline,
  because native Git does it too:
  one index write,
  the commit object and its signature,
  the reference update,
  and the reads it makes of the branch.

## The share over the bar, and how native was bounded

Native Git was bounded two ways.

The first bound is the wrapper's own Git children:
every step the record names is work native Git also does for the same commit,
so a commit's whole time minus the sum of its step durations
is time native Git plausibly does not spend.
Taking all of them,
retries included,
is generous to native,
so the share it gives is a lower bound.

- 217 of 866 commits (25.1 percent)
  took more than 5000 ms beyond their own Git steps.
- Counting instead the excess over a whole-commit native estimate of 200,
  500 or 2000 ms
  moves that count by little:
  24.6,
  23.7 and 20.7 percent.
  A native commit does not plausibly take 2000 ms here,
  so the share sits between 20.7 and 25.1 percent.

The second bound is measured:
in a throwaway repository of this tree's shape,
six paired native `git commit` runs took a median of 168 ms (142 to 781 ms),
against 1359 ms for the wrapper's median in the same rounds.
Both arms alternate,
so neither owns a position.

## The change: an empty conclusion record no longer flushes the disk

`store-held.json` names the conclusion values a backend keeps in its ref store.
A files-backend repository keeps none,
so the record said `{}` on every commit and was still published with a file sync
(`package/git-policy/cli/src/native/commit/conclusion_copy.rs`).
The flush sat in the `buildMs` uncovered time that holds
53.7 percent of the record's waiting,
and it waited for the disk 24.9 s in one measured commit.

The record is now written only where it can name something:
a backend whose ref store can hold a conclusion value keeps the file and its flush,
and a files-backend copy writes none,
because absence and `{}` answer the same question there.
Both sides read one rule,
`store_names_conclusions`
(`package/git-policy/cli/src/native/commit/conclusion_cleanup.rs`),
so a name added to the table cannot leave the writer and the reader apart.
`ObjectWriter::is_empty` is the one helper the writer needed.

Red controls first,
both failing on the code as it stood:

- `commit::conclusion_copy::tests::a_files_copy_names_nothing_in_the_store_so_it_writes_no_record_and_syncs_no_file`
  failed with
  `nothing in the copy is flushed: [".../transaction/conclusion/store-held.json"]`.
- `commit::conclusion_cleanup::tests::store_records_must_be_json`
  failed with
  `reading .../transaction/conclusion/store-held.json failed: No such file or directory (os error 2)`
  for a files-backend transaction.

Two controls hold the other side:
`a_reftable_copy_keeps_the_record_and_its_flush`
and `reftable_pseudorefs_are_deleted_through_git`.

Three tests outside those modules had built "a cleanup that cannot be done"
out of a copies directory with no record in it,
which now names nothing:
`a_merge_in_progress_is_copied_to_the_shadow` in `shadow_create_tests.rs`,
`a_late_switch_keeps_the_batch_open_while_its_cleanup_is_pending` in `service_land_head_tests.rs`,
and the two `service_land_tests.rs` scenes
`a_cleanup_that_fails_does_not_make_the_index_update_replayable`
and `a_restart_after_a_switch_keeps_the_batch_open_while_its_cleanup_is_pending`.
Each now writes a record that is not JSON,
which is damage in every backend and keeps the subject it was written for.

Measured in a throwaway repository of this tree's shape,
alternating arms,
three commits per arm through `strace -f -T -e trace=%file,%desc,fsync`:

- Before:
  the record was flushed in 3 of 3 commits,
  taking 0.386 s,
  0.053 s and 0.179 s.
- After:
  0 of 3 commits flushed it.
- The uncovered stretch of `buildMs` the flush sits in,
  before:
  median 29 ms,
  max 128 ms over 6 commits.
  After:
  median 4 ms,
  max 6 ms over 6 commits.
  Earlier runs of the same fixture before the change showed that stretch at
  1279 ms,
  2530 ms and 5569 ms.
- Whole-commit wall time:
  before median 1359 ms (355 to 3136 ms),
  after median 2381 ms (489 to 5451 ms),
  native median 168 ms (142 to 781 ms),
  six samples per arm.
  The spread of one arm on unchanged input is wider than the change,
  so this record claims no whole-commit improvement from it:
  what it removes is one flush whose own cost was measured directly.

## The scratch commit's ceiling, which turns a wait into a refusal

`package/git-policy/cli/src/native/commit/commit_preparation.rs` runs the scratch `git commit`
under a notice whose ceiling is `SCRATCH_COMMIT_STEP_CEILING`,
3006 ms (`wait_ceilings.rs`:
"3 x 1,002 ms",
the owner's answer of 2026-11-08,
"About 3 s now"),
and ends the step and its descendants at that ceiling
when the message came with the command.
That is a wait limit on the commit's own path,
and it can turn a slow wait into a refusal,
which the owner's "Commits should never fail."
rules against;
the ceiling was chosen with that price stated.

In this window the refusal appears 8 times:
8 commits of the 866 (0.9 percent) carry `outcome=stopped`,
exit 1,
and a scratch `git commit` step of 3024,
3027,
3029,
3050,
3074,
4169,
4259 and 4359 ms
(the ceiling plus the time to end the process tree),
all of them on release 0.6.1-alpha.4.
One more commit stopped on an older release after a 171 383 ms step,
before the 3006 ms ceiling existed.
The 287 records of the installed release 0.6.1-alpha.5 hold none,
and their slowest scratch commit is 1748 ms.

What that step was doing is build and signature in the scratch repository,
in one process with no sub-step figure:
`commit.gpgsign` is true here with `gpg.format=ssh`,
so `git commit` starts the SSH signing program,
and the stop's own words name it
("The commit signs the commit because `commit.gpgsign` or `-S` asks for it,
so the signing program is the first thing to check").
The step is cheap when nothing competes:
median 42 ms over the window,
39 ms and a p90 of 151 ms on the installed release,
and my own `strace` run of a shaped commit stopped the same step at the same ceiling
while the signing program ran.
The same window also holds a commit that landed after a 36 651 ms scratch commit,
so the ceiling does not end every slow one:
the notice answers "the step ended on its own" when its look for descendants finds none,
and that commit's 36.6 s says the look can find none while the step is still running.

Two things follow for the owner:
a ceiling of 3006 ms on a step whose tail is contention on this host refuses commits
that would have landed,
and
the check that arms it is not the same for every commit,
which is why one instance of that tail refused a commit and another did not.

## What remains

Each entry is a step this wrapper adds,
with the measurement that sizes it,
and its share of the record's uncovered time.
The ranking weighs how much of the measured tail each can take
against how much of the shared,
owner-held machinery it would move.

### The preparation's private policy index and its in-process work

- What:
  the preparation reads the whole index into memory,
  copies it into a private policy index,
  writes it again with `git update-index`,
  and runs the policies over the staged content.
- Cost:
  21.7 percent of uncovered time,
  median 332 ms,
  p99 11 884 ms,
  256 commits over 1000 ms and 50 over 5000 ms;
  3.5 s,
  1.2 s and 0.35 s of `git update-index` in three shaped commits,
  1.5 s to read the 2.9 MB index and 1.05 s to copy it under `strace`,
  and over the day `git read-tree` p99 3272 ms and `git add` p99 1783 ms.
- Pros:
  it is the largest bucket this wrapper owns outright,
  it is the largest write the wrapper adds per commit,
  and nothing about it crosses into another process.
- Cons:
  `core.fsmonitor`,
  the cached stat data and the landing's comparison all depend on
  that copy,
  another branch is changing index handling at landing just now,
  and the records cannot say which policy spends the milliseconds without a profiler.
- Ranking:
  1 of 4,
  for the largest own share at a blast radius of one process.

### The service's landing

- What:
  the residual reads of a landing,
  its index update under the lock,
  and the queue behind an earlier batch holding that lock.
- Cost:
  13.4 percent of this commit's uncovered time,
  median 87 ms,
  p99 13 531 ms,
  86 commits over 1000 ms and 36 over 5000 ms,
  and the largest cross-commit wait in the records:
  `queued_ms` reaching 24 726 ms,
  `reads_ms` 37 068 ms,
  `index_update_ms` 7768 ms,
  and 28 landings over 500 ms of index update.
- Pros:
  it is the only entry that shortens other agents' commits as well,
  because a landing holds the lock others queue on.
- Cons:
  it changes where a landing waits and what it holds,
  the widest blast radius of the four,
  and its own share is smaller than the preparation's.
- Ranking:
  2 of 4,
  for the second largest own share and the only cross-commit one.

### Removing the scratch tree after the commit landed

- What:
  `Scratch::drop` removes the transaction directory recursively when the command ends,
  which is after the commit landed.
- Cost:
  one `unlinkat` of 2.5 s in a shaped fixture;
  7.1 percent of uncovered time,
  52 commits over 1000 ms and 14 over 5000 ms,
  with `otherMs` reaching 58 733 ms.
- Pros:
  nothing after the removal needs the bytes,
  a sweep already removes stale scratch trees,
  and the change is small and local.
- Cons:
  a kill or a crash leaves a tree for the sweep instead of removing it at once,
  and the test fixtures need the synchronous path kept for them.
- Ranking:
  3 of 4,
  for a smaller share than the two before it but the least risk of all.

### The rest of the shadow's creation

- What:
  the metadata of `create_shadow_repository_carrying`,
  which created directories and symlinks and wrote the shadow's config and alternates
  without syncing any of them.
- Cost:
  unknown on its own.
  It shared the measured uncovered `buildMs` stretch with the flush,
  and the fixture runs that named the flush could not separate them,
  so this entry is the part of 53.7 percent that the change did not take.
- Pros:
  it is the largest bucket still open.
- Cons:
  nothing measured sizes it,
  so any change there would be a guess about a bucket that is small when the device is not stalling.
- Ranking:
  4 of 4,
  because it cannot be sized until it is measured.

## What the record now says about a landing, and what eight overlapping commits measured

### The record

Three fields were added to the commit timing record because a slow landing could not explain itself:

- `landing.reads` names each of the residual reads of a landing one by one:
  the configuration snapshot,
  the branch and checked-out reads,
  the parents of an amend,
  the comparisons with the branch,
  the content check,
  the index witness,
  the conclusion copy,
  the records a batch owes,
  and the head question under the lock.
  `reads_ms` stays the whole, and the named parts never add up to more than it.
- `landing.lockHoldMs` is how long the landing held the index lock, which is what the commits behind it queued on,
  and `landing.queuedBehind` is how many requests were still waiting for their answer when the landing finished.
- `landing.syncs` is every file or directory sync the landing asked for,
  in the order it asked,
  each with whether the landing waited for it and how long it took.
  A background batch of a landing's records says in the journal how many it made and what they cost.

The service measures all three,
so the answer to a commit carries them and the record states them;
a service of a release that sends none leaves the field `null`,
never zeros.
The `answered` line of the journal states the same figures, so the two files agree.

### The measurement

One binary, two runs of four rounds of eight overlapping commits in a disposable clone of this repository
(452 990 objects and 22 462 tracked files),
started together on one branch so they land as one batch:

- run A: 32 commits,
  median 4 280 ms,
  p95 8 623 ms,
  14 of 32 over 5 000 ms.
- run B, the same binary:
  32 commits,
  median 22 002 ms,
  p95 40 868 ms,
  32 of 32 over 5 000 ms.

Two runs of one binary differ by 5.1 times at the median,
which is the host's own spread and not a change:
every figure below is stated with both runs beside it.

The three documents the runs wrote are `perf/commit-landing-2026-11-09-overlapping-a.json`,
`perf/commit-landing-2026-11-09-overlapping-b.json` and
`perf/commit-landing-2026-11-09-solo.json`;
each holds the phases of every commit,
every landing line as the service wrote it,
and the landing figures the records carry.
The harness that wrote them was scaffolding in `${HOME}/temp/agent` and is not in the tree:
`mise run //package/git-policy/cli:native:perf` measures one commit at a time,
and measuring overlapping commits again means the same three steps by hand:
clone the repository somewhere disposable,
point its `origin` at a bare repository of that scratch tree so no push reaches the checkout it came from,
and start the wrapper's service for the clone with `bench`-style arguments under an `XDG_RUNTIME_DIR` of its own.

The commit's phases, as the record counts them:

- prepare: median 3 487 ms in run A and 5 442 ms in B (p95 5 276 and 19 282).
- submit: median 755 ms and 10 843 ms (p95 3 025 and 24 813).
- gate: median 151 ms and 1 519 ms (p95 2 701 and 10 569).
- build: median 3 ms in both; after-landing, handoff and uncovered time under 80 ms.

The landing, as the service splits it:

- landing: median 382 ms in run A and 10 756 ms in B (p95 2 773 and 17 113).
- the wait for another program's hold on the index: median 10 539 ms in run B (p95 14 249),
  which is 98 percent of that run's median landing.
- everything the landing itself does:
  reads median 29 ms in A and 15 ms in B,
  objects 17 ms,
  the intent journal 0 ms,
  the branch move 14 ms,
  the index update 89 ms.
- the hold on the index: median 165 ms in A and 190 ms in B (p95 207 and 2 771).
- queued behind a landing: seven commits for the first request of each batch and none for the rest,
  because the eight commits of a round land as one batch.
- six syncs per landing,
  always handed to the background and never waited for:
  193 over 32 landings in each run,
  named `intent-file`, `intent-dir`, `index-dir`, `indexed-file`, `indexed-dir` and `journal-dir`,
  with `lock-marker` only for the first landing of a run (the marker is made ahead after that).

### The same clone, one commit at a time

A third run, of six commits one after another in the same clone,
so nothing ever queued:

- the wait for another program's hold on the index: median 1 ms, largest 3 040 ms.
- the hold on the index: median 305 ms, largest 724 ms.
- the index update: median 192 ms.
- reads: median 4 ms.
- queued behind a landing: none, in every one of the six.

Seven siblings in one batch turn that 1 ms into 10 539 ms,
and no sibling at all leaves it at 1 ms.
The wait is therefore the batch's own members holding the index,
not the landing.

### What it means

The landing's own work is small and no named read is waste:
every suggestion of reading once per batch,
locking only around the write,
one sync per batch and one index update per batch is already how the landing works.
The landing's one large cost is a wait it does not cause:
each commit of a batch refreshes the real index in its own preparation
(`git update-index --refresh`, which takes `index.lock`),
so the batch's landing waits for its own members to stop refreshing,
and those members wait for the landing in turn.

So no change to the landing would take the tail under 5 s:
the landing is 382 ms of a 4 280 ms median commit in the quieter run,
and its largest piece is a wait on another step.
The tail is elsewhere:

- the preparation itself,
  median 3 487 ms in run A,
  6 675 ms in the run one commit at a time,
  p95 34 511 ms,
  and the slowest commit of that run spent 34 511 ms of its 37 227 ms there;
- the post-commit gate,
  p95 2 701 ms in run A and 4 380 ms in the run one commit at a time;
- the per-commit refresh of the real index by the preparation,
  which the landing of a batch waits for and which a batch needs once.

The first and the third are the preparation's,
which another branch is changing,
and the second is the gate's.

## What this could not determine

- Which in-process step inside a preparation spends the median 332 ms.
  The records name Git steps and phases,
  not policy rules,
  and this host has no profiler.
- Whether the 351 s and 493 s stalls of 2026-11-08T23:26 were the conclusion copy's flush.
  Both sit in the same uncovered `buildMs` stretch,
  between the shadow's `git config` read and its first reference write,
  which is where that flush is,
  and the same stretch reproduced at 1279 ms,
  2530 ms and 5569 ms
  with a 24.9 s flush measured inside it.
  The record holds no figure for that stretch,
  so the live commits cannot be proven to be it.
- How much of the 7.1 percent of `otherMs` is the scratch removal
  and how much is the wrapper writing what the person reads on standard error,
  which blocks when the reader is slow.
- Whether native Git would have waited as long in a flush under the same device state.
  Native Git flushes the index only when `core.fsync` asks,
  which is unset here,
  so this record compares step by step and not device stall by device stall.

## Decisions this record takes, open to the owner's veto

- The ranking of the four entries above changes:
  the landing's own share is smaller than the records suggested
  (382 ms of a 4 280 ms median commit, with no named read over 51 ms),
  and what its number was made of is a wait on the preparation of the batch's own members.
  Removing waste inside the landing would therefore not move the tail,
  and the entry that would is the preparation's per-commit refresh of the real index,
  which is not one of the four.

- The conclusion record's fail-closed row narrows to the backend that carries values.
  `doc/planning/cli-git-spawn-and-sync-reduction.md` keeps `store-held.json` in the
  "must stay synced" list because a torn one stuck every command after the reference moved.
  That reason cannot apply where no conclusion name can be in the store at all,
  and a record that is never written cannot tear,
  so the files-backend case now answers `{}` where it used to fail closed.
  The other reading is to keep writing and flushing the record everywhere,
  which costs every commit a flush of the disk,
  and to take the wait off the
  command's path another way.
- The four ranked entries of this record are proposals,
  not work in progress.
