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
  ("the longest quiet gap of every Git step").
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

The window is the day ending 2026-11-09T03:26:44Z,
which holds 854 commits and 891 landings.
Every figure in this record is that window.

The machine at the time was under pressure of the kind other sessions keep up
(the owner's rules hold that this is normal and never an excuse):
load average 27 to 62 on 6 CPUs,
and `/proc/pressure/io` reporting `full` at 36 percent over ten seconds,
23 percent over five minutes,
while a `repository-move-cutover` unit copied a disk.
Every measurement this record reports alternates its arms and reports its counts,
no synthetic load was generated,
and the probes ran in `systemd-run --user --scope` scopes.

## The steps, by what they cost

Client steps,
one entry per line of the commit record that names a Git step,
with occurrences,
median,
the slowest 1 in 100,
the maximum,
and the total over the window:

- `git commit`:
  826 occurrences over 809 commits,
  median 42 ms,
  p99 4169 ms,
  max 171 383 ms,
  399 856 ms in all.
  This is Git building the commit in the scratch repository,
  signing included,
  and the only step a run of retries repeats.
- `git add`:
  836 over 836,
  median 37 ms,
  p99 1783 ms,
  max 10 567 ms,
  114 248 ms.
- `git read-tree`:
  260 over 254,
  median 52 ms,
  p99 3272 ms,
  max 5909 ms,
  49 753 ms.
- `git update-index`:
  376 over 341,
  median 14 ms,
  p99 660 ms,
  max 3202 ms,
  18 528 ms.
- `git rev-parse`:
  2261 over 854,
  median 2 ms,
  p99 12 ms,
  max 1607 ms,
  9794 ms.
- `git symbolic-ref`:
  1652 over 809,
  median 2 ms,
  p99 14 ms,
  max 1795 ms,
  6841 ms.
- `git update-ref`:
  826 over 809,
  median 3 ms,
  p99 15 ms,
  max 2350 ms,
  5485 ms.
- `git for-each-ref`:
  826 over 809,
  median 4 ms,
  p99 46 ms,
  max 142 ms,
  4334 ms.
- `git cat-file`:
  815 over 798,
  median 2 ms,
  p99 4 ms,
  max 7 ms,
  1873 ms.
- `git write-tree`:
  22 over 16,
  median 9 ms,
  max 667 ms,
  1427 ms.
- The rest together (`git diff-index`,
  `git merge-tree`,
  `git diff`,
  `git hash-object`,
  `git config`,
  `git checkout-index`,
  `git check-attr`,
  `git merge-base`):
  under 400 ms in all.

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
  max 16 ms.

So the Git children are not the tail:
one commit's steps add up to a median of 153 ms,
while its whole record has a median of 1153 ms.

## The commit, phase by phase

The phases are disjoint and sum to `totalMs`:

- `prepareMs`:
  median 445 ms,
  p99 13 576 ms,
  max 25 626 ms,
  total 1 343 967 ms,
  25.8 percent of all time.
- `buildMs`:
  median 108 ms,
  p99 26 947 ms,
  max 493 699 ms,
  total 2 703 169 ms,
  51.8 percent.
- `submitMs`:
  median 87 ms,
  p99 13 659 ms,
  max 37 239 ms,
  total 621 585 ms,
  11.9 percent.
- `gateMs`:
  median 30 ms,
  p99 3733 ms,
  max 8687 ms,
  total 175 973 ms,
  3.4 percent.
- `otherMs`,
  what no phase covers:
  median 3 ms,
  p99 7702 ms,
  max 58 733 ms,
  total 355 993 ms,
  6.8 percent.
- `provisionMs` 6639 ms,
  `afterLandingMs` 1435 ms,
  `handoffMs` 3298 ms in all.

The whole commit:
median 1153 ms,
p99 49 766 ms,
max 494 667 ms.

## What the steps wait on

The phases overlap the Git steps,
so the honest question is where the time sits that no step covers.
Walking each record's steps in order gives every gap a phase:

- gaps in `prepareMs`:
  3 466 179 ms,
  74.6 percent of all uncovered time.
- gaps in `submitMs`:
  900 143 ms,
  19.4 percent.
- gaps in `buildMs`:
  150 884 ms,
  3.2 percent.
- gaps in `provisionMs`:
  125 061 ms,
  2.7 percent.
- gaps in `gateMs`:
  657 ms.

For the commits over the bar the same split holds (76.3,
17.4,
3.7,
2.5 percent),
and the largest single gap of an over-bar commit is a prepare gap in 150 of 197,
a submit gap in 39,
a provision gap in 5,
and a build gap in 3.

The prepare gap is the wrapper's own work and the waiting around it:
median 323 ms,
p90 3838 ms,
p99 11 884 ms,
max 25 460 ms,
over 1000 ms in 249 commits and over 5000 ms in 49.
It is not the markdown linter:
commits whose linter ran have a median prepare gap of 330 ms and those without 309 ms.
It is not CPU either.
One commit in a throwaway repository of this tree's shape took 3.7 s of wall time
with 0.076 s of user and 0.242 s of system time,
so the process was waiting,
not computing.
`strace -T` on the same shape names what it waited on.

What each wait is:

- Waiting on a child Git:
  every Git step this record lists,
  all recorded.
  Signing lives in the scratch `git commit` (`git commit` max 171 s),
  and native Git pays it too (`commit.gpgsign` is true here).
- Waiting on a lock:
  the service waits for a foreign holder of the index lock
  (`lock_wait_ms`,
  p99 342 ms,
  max 11 490 ms,
  in 136 of 891 landings),
  holds it while it moves the branch and builds the index
  (`lock_hold_ms`,
  median 51 ms,
  p99 3554 ms,
  max 8101 ms),
  and landings queue behind each other
  (`queued_ms`,
  p99 2170 ms,
  max 24 726 ms).
  The client never waited on that lock in this window:
  `lockHolderKind` is null in all 854 records.
- Waiting on a flush:
  the preparation-time conclusion copy published `conclusion/store-held.json`
  with a file sync on every commit.
  One such `fsync` blocked for 24.9 s,
  another for 2.5 s,
  1.3 s,
  0.386 s,
  0.179 s and 8 ms across samples.
  Native Git syncs no such record for the same commit.
- Waiting on the device in other ways:
  the wrapper reads the whole 2.9 MB index and copies it into the policy index
  (1.5 s and 1.05 s in one traced commit),
  and the scratch tree is removed by a recursive delete when the command ends
  (one `unlinkat` of 2.5 s;
  `otherMs` reaches 58 733 ms).
- Waiting on the service:
  the landing's residual `reads_ms`
  (the branch reads and the comparisons,
  `LandTiming::reads`),
  p99 9047 ms,
  max 37 068 ms,
  293 578 ms in all,
  and the index update `index_update_ms`,
  p99 3247 ms,
  max 7768 ms,
  127 151 ms in all.

Which side each comes from:

- From this wrapper:
  the conclusion copy's flush,
  the copy and read of the whole index into a private policy index,
  the removal of the scratch tree on the way out,
  the landing's index update and its hold,
  and the in-process policy work of the preparation.
- From the host:
  how long any of those take under the disk and CPU pressure other sessions keep up,
  and how long the signing program and the other programs holding the index lock take.
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

- 197 of 854 commits (23.1 percent)
  took more than 5000 ms beyond their own Git steps.
- Counting only the excess over a whole-commit native estimate of 100,
  200,
  500 or 2000 ms
  changes the count by little:
  24.6,
  24.2,
  23.4 and 20.6 percent.
  A native commit does not plausibly take 2000 ms here,
  so the share sits between 20.6 and 24.6 percent.

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
The flush is on the commit's path,
inside `buildMs`,
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

Measured in a throwaway repository of this tree's shape,
alternating arms,
three commits per arm through `strace -f -T -e trace=%file,%desc,fsync`:

- Before:
  the record was flushed in 3 of 3 commits,
  taking 0.386 s,
  0.053 s and 0.179 s.
- After:
  0 of 3 commits flushed it.
- The wait inside `buildMs` that no Git step covers,
  before:
  median 29 ms,
  max 128 ms over 6 commits.
  After:
  median 4 ms,
  max 6 ms over 6 commits.
  Earlier runs of the same fixture before the change showed that wait at
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

## What remains

Each entry is a step this wrapper adds,
with the measurement that sizes it.

### The private policy index

- What:
  the preparation reads the whole index into memory and copies it,
  then writes it again with `git update-index`,
  in the private policy index.
- Cost:
  3.5 s,
  1.2 s and 0.35 s of `git update-index` in three shaped commits;
  1.5 s to read the 2.9 MB index and 1.05 s to copy it under `strace`;
  over the day,
  `git update-index` p99 660 ms and `git read-tree` p99 3272 ms.
- Pros of shrinking it:
  it is the largest measured write this wrapper adds per commit,
  and it sits in the bucket that holds 74.6 percent of uncovered time.
- Cons:
  `core.fsmonitor`,
  stat data,
  and the landing's comparison all depend on that copy;
  another branch is changing index handling at landing just now,
  and one index defect is worth more than 3 s of latency.
- Ranking:
  1 of 4.

### Removing the scratch tree after the commit landed

- What:
  `Scratch::drop` removes the transaction directory recursively when the command ends,
  which is after the commit landed.
- Cost:
  one `unlinkat` of 2.5 s in a shaped fixture;
  `otherMs`,
  which holds it,
  reaches 58 733 ms and 6.8 percent of all time.
- Pros:
  nothing after the removal needs the bytes,
  a sweep already removes stale scratch trees,
  and the change is small and local.
- Cons:
  a kill or a crash leaves a tree for the sweep instead of removing it at once,
  and test fixtures need the synchronous path kept for them.
- Ranking:
  2 of 4.

### The service's landing

- What:
  the residual `reads_ms` of a landing,
  its index update,
  and the queue behind an earlier batch holding the index lock.
- Cost:
  `reads_ms` p99 9047 ms and max 37 068 ms,
  `index_update_ms` p99 3247 ms,
  `queued_ms` max 24 726 ms,
  and 39 of 197 over-bar commits had their largest gap there.
- Pros:
  it is the largest wait one commit imposes on the others,
  so it is the only entry that shrinks other agents' commits too.
- Cons:
  it touches the shared index lock and the landing order,
  the widest blast radius of the four,
  and 19.4 percent of uncovered time is a smaller share than the preparation's.
- Ranking:
  3 of 4.

### The preparation's in-process work

- What:
  the policy engine's own work between the Git steps of a preparation.
- Cost:
  median 323 ms per commit,
  p99 11 884 ms,
  249 commits over 1000 ms and 49 over 5000 ms,
  which makes it the typical cost and the tail at once.
- Pros:
  it is the whole of the largest bucket,
  and it is pure waiting on this host rather than on the disk.
- Cons:
  no profiler is installed here (`perf` is absent),
  so the split between the policy scan,
  the rules load and the capture is unknown,
  and guessing a rule to cut risks the policies the wrapper exists for.
- Ranking:
  4 of 4.

The ranking orders them by how much of the measured tail each can take
against how much of the shared,
owner-held machinery it would move.
The first two are technical and local;
the third and fourth change where a landing or a policy waits.

## What this could not determine

- Which in-process step inside a preparation spends the median 323 ms.
  The records name Git steps and phases,
  not policy rules,
  and this host has no profiler.
- Whether the 493 s and 351 s stalls of 2026-11-08T23:26 were the conclusion copy's flush.
  Both sat in the same `buildMs` gap,
  between the shadow's `git config` read
  and its first reference write,
  which is where that flush is,
  and the same gap reproduced at 1.3 s,
  2.5 s and 5.6 s with a 24.9 s flush measured inside it.
  The record holds no sub-step figure,
  so the live commit cannot be proven to be that flush.
- How much of `otherMs` is the scratch removal and how much is the wrapper writing
  what the person reads on standard error,
  which blocks when the reader is slow.
- Whether native Git would have waited as long in a flush under the same device state.
  Native Git writes the index and flushes it only when `core.fsync` asks,
  which is unset here,
  so the comparison in this record is per step and not per device stall.

## Decisions this record takes, open to the owner's veto

- The conclusion record's fail-closed row narrows to the backend that carries values.
  `doc/planning/cli-git-spawn-and-sync-reduction.md` keeps `store-held.json` in the
  "must stay synced" list because a torn one stuck every command after the reference moved.
  That reason cannot apply where no conclusion name can be in the store at all,
  and a record that is never written cannot tear,
  so the files-backend case now answers `{}` where it used to fail closed.
  The other reading is to keep writing and flushing the record everywhere and to solve
  this wait some other way,
  which costs every commit a flush of the disk.
- The four ranked entries of this record are proposals,
  not work in progress.
