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

### The conclusion entry's own flush, the last wait in `buildMs` (2026-11-11T08:15:00-05:00)

The change that removed the empty `store-held.json` record left one kind of flush in `buildMs`:
`package/git-policy/cli/src/native/commit/conclusion_copy.rs` publishes every conclusion entry it finds twice,
once into the shadow the scratch `git commit` reads and once into the transaction's comparison copy,
and both writes go through `write_private_file`,
which syncs the file.
A merge,
a cherry-pick or a revert leaves any of nine entries,
so one conclusion commit makes two flushes per entry.

Measured on this host on 2026-11-11,
three arms alternating over the four commits of
`service_landing_state::a_squash_merge_conclusion_lands_through_the_service_and_removes_its_squash_state`,
with each commit's own timing record read out of its served copy:

- Synced:
  `buildParts.shadow conclusion` 3,601,
  1,464,
  480 and 487 ms,
  whole commits 3,823,
  2,087,
  688 and 780 ms.
- The same bytes through `write_private_file_unsynced`:
  `buildParts.shadow conclusion` 0 ms in all four,
  whole commits 63,
  53,
  330 and 47 ms.
- Synced again:
  `buildParts.shadow conclusion` 2,892,
  4,614,
  1,313 and 105 ms,
  whole commits 3,415,
  4,997,
  1,908 and 179 ms.

The whole of that stretch is those syncs:
no Git step falls inside it,
and the scratch `git commit` of the same commits took 37 to 106 ms.
A conclusion commit through the wrapper therefore takes 0.2 to 4.9 s more than native Git's 25 to 463 ms for the same four commits,
and on a host whose disk stalls one flush costs what this record measured for it
("The change:
an empty conclusion record no longer flushes the disk";
24.9 s).

Why no sync is dropped here:
the table of what a torn pre-landing record does
(`doc/planning/cli-git-spawn-and-sync-reduction.md`,
"J3 outcome:
what recovery does with emptied,
halved and dropped records")
covers `T/conclusion/store-held.json` and no other file of `T/conclusion`,
so the two copies of a conclusion entry are outside the evidence that rule asks for before a pre-landing record's file sync goes
(`doc/planning/cli-git-spawn-and-sync-reduction.md`,
"Journal syncs").
Extending that harness to `T/conclusion/<entry>` and to the shadow's copy is what the next reading needs;
whether the syncs then go is the owner's call,
because one direction that loses is a merge that lands as a plain commit.

What it costs a control:
`service_landing_state::a_conclusion_lands_through_the_service_and_removes_the_state_native_git_removes`
landed 28 conclusion commits in one test,
took 284.9 s alone on 2026-11-11 with other sessions loading the host,
15 s under the runner's five-minute stop,
and one run of it reached that stop and was ended naming no state.
Each conclusion state is its own control since `4bf304b393`.

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

## The otherMs share split, and the removal off the path (2026-11-09T06:41:47-05:00)

The record could not say how much of `otherMs` was the removal of the commit's scratch tree and how much was the wrapper
writing what the person reads,
whose write waits for its reader.
Both are now measured,
and the removal no longer happens on the person's path.

What was measured,
and how:

- Two fields state the split,
  taken from the ledgers the command already keeps,
  one clock read each:
  `scratchRemovalMs` with `removalsThatDidNotFinish`,
  and `stderrMs`
  (`package/git-policy/cli/SPEC.md`,
  "Timing logs";
  `package/git-policy/cli/README.md`,
  "Splitting a slow commit").
- Four fixtures shaped like this tree were made by copying a repository that holds this tree's 22,190 tracked files and
  its 2.9 MB index,
  with this repository's own policy configuration:
  two for the binary before the change and two for the binary after it,
  so every round compares arms that cannot have prepared each other's state.
  The four arms alternated inside each round.
  The host was at load average 47 to 139 on six CPUs,
  and `/proc/pressure/io` reported `full` at 29.6 to 37.3 percent over ten seconds.
  No load was generated for this measurement,
  and each probe ran in a `systemd-run --user --scope` scope.
- The change itself is `fix(git-policy/cli): a commit no longer removes its scratch tree on its own path`
  (`e6a69463ed`,
  with `103a0cff05` and `1a56f2b115` before it,
  on `fix/git-policy-cli-commit-scratch-removal-off-the-path`);
  the tree now lives in the repository scratch directory the sweep covers,
  the service removes it beside its landing loop and bounded,
  a command that no service took it from removes it as it ends,
  and a tree a kill left is removed by a later command's preparation once it is two hours old
  (that last rule changed on 2026-11-09,
  when a tree a kill left began going at the next command;
  see "The tree a kill leaves goes at the next command,
  and a live one never goes",
  below).
  `package/git-policy/cli/SPEC.md`,
  "The commit's scratch directory",
  holds the rules and every size.

The split,
over the eight commits of each binary,
in whole milliseconds
(the four arms were run at 06:41:47-05:00 and again at 07:14:17-05:00,
the second time on the final build;
the earlier run gave 17 ms of removal and 0 after it,
and the figures here are the last one):

- The scratch tree's removal:
  `scratchRemovalMs` was 3,
  1,
  28,
  0,
  1,
  0,
  0 and 1 before the change,
  34 ms in all and 28 ms at its worst,
  and 0 in eight of eight commits after it.
  A tree of one commit held 9,081,951 bytes in three files on this host,
  and `strace -f -c` of one commit through the earlier binary counted 34 `unlinkat` calls at 1.5 ms of system time.
- The writes to standard error:
  `stderrMs` was 0 in all sixteen records,
  because the wrapper's lines fit in the pipe buffer,
  so on an ordinary reader the write costs nothing at all.
- What no phase covered:
  `otherMs` was 4,
  2,
  29,
  1,
  1,
  1,
  1 and 1 before the change,
  median 1 ms and 40 ms in all,
  and 0 in eight of eight commits after it.
  The removal was therefore the whole of `otherMs` here (34 of its 40 ms),
  and the record's 58,733 ms of `otherMs` and its 2.5 s `unlinkat` are the device-stall tail,
  not the median path.
- Whole commit,
  wall clock:
  the before arms had medians of 7,946 ms and 5,852 ms,
  the after arms 7,953 ms and 5,483 ms,
  and one arm's own samples span 993 to 16,898 ms.
  The two binaries are not told apart by this,
  and the same-binary control arms differ by 1.36 times,
  so this record claims no whole-commit improvement:
  what the change takes away is the removal's own measured wait,
  0 to 28 ms here and 2.5 s for one `unlinkat` once,
  and it takes that wait off the person's path for good.

Proof by removal,
each control run against the branch's own change undone:

- `the_commit_hands_its_scratch_tree_on_and_never_removes_it` failed with
  `the command that made it must not remove it: .../monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-0a1b2c3d`
  once `Scratch`'s old `Drop` was put back.
- `the_scratch_tree_is_made_where_the_sweep_covers_it` and `a_scratch_tree_of_a_killed_command_is_swept` failed with the
  tree at `.../repo/.git/monochromatic-git-policy-cli-service-0a1b2c3d` where the test names
  `.../repo/.git/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-0a1b2c3d`,
  once the tree was made in the Git directory again.
- `a_scratch_announcement_names_only_a_scratch_directory_of_the_worktree` failed
  with `.../monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-0a1b-2c` refused,
  once the service accepted only the Git directory.
- `an_abandoned_scratch_tree_goes_before_the_sweeps_day` failed with `left: 0, right: 1`,
  once the age rule removed nothing.
  That test and the age rule it proves were replaced on 2026-11-09
  (see "The tree a kill leaves goes at the next command,
  and a live one never goes",
  below);
  the test that holds the same ground now is `an_unmarked_tree_goes_by_age_and_a_fresh_one_stays`.
- `a_removal_says_what_became_of_it` failed with `a removal that outlasts its ceiling says so: Finished`,
  once the removal was given an hour instead of `SCRATCH_REMOVAL_CEILING`.

What this leaves open:

- Whether the two counts above hold on a warm page cache,
  where the record's live commits run:
  the fixtures start cold,
  and one commit there pays a `git update-index --refresh` of 122 to 173 seconds.
- What bounds the trees in a repository nobody commits in again:
  eight per command,
  a day of age for one with no marker,
  stated in `SPEC.md`,
  but every one of those rules belongs to a command,
  so a repository left alone keeps the trees it had.

## The tree a kill leaves goes at the next command, and a live one never goes (2026-11-09T08:12:27-05:00)

The rule above decided a leftover tree by its age alone,
and age cannot tell a killed command's tree from a live one's:
the scratch `git commit` runs with the terminal attached and no watch,
so a person who leaves an editor open in the scratch repository holds the tree for as long as they like,
and a two-hour-old tree of a running commit is what the next command would then have removed:
the private indexes and the typed message of a commit still being made.
The tree is now owned,
and ownership decides:

- As a tree is made,
  the command writes `owner` into it,
  a marker in the index lock's own format
  (`cli-git-service v2 0 <pid> <birth>`,
  `service_index_marker.rs`),
  naming its process and that process's birth identity.
- A later command's preparation reads that marker and removes the tree at once when `/proc`
  proves the process is gone,
  at most `SCRATCH_TREES_REMOVED_PER_COMMAND` (eight) of them,
  and leaves a live command's tree alone however old it is.
- A tree with no marker,
  which a release before this one made,
  or one a command was killed in between making and marking,
  goes once it is `UNMARKED_SCRATCH_TREE_AGE` old (two hours,
  as every tree did before),
  and the sweep of the place still removes whatever is a day old whatever its number.
- `ScratchRenewal` sets the tree's modification time to now every `SCRATCH_REFRESH_EVERY`
  (an hour) while the command lives,
  so the age those age rules see is the command's liveness,
  and it stops when the command lets the tree go.
- The two rules sit beside the landing rather than in the foundation:
  `src/native/landing/service_scratch_abandonment.rs`,
  `src/native/landing/service_scratch_renewal.rs`,
  with `scratch_entry_removal.rs` keeping only the bounded removal of one entry.

What this adds to the preparation,
measured on the four fixtures of the round above,
every arm alternating inside each round,
six rounds and twenty-four commits,
all of them landed (exit 0):

- The whole stretch the work is in,
  `prepareParts.scratch` (making the tree,
  marking it,
  reading the place for trees to collect,
  starting the renewal):
  0 ms in twenty-three of the twenty-four records and 2 ms in one,
  and 0 in every record of the two arms without the work.
  The record's resolution is a whole millisecond,
  so what the change spends is under it.
- `prepareMs`,
  every sample:
  488 to 6,409 ms on the first arm with the work and 561 to 3,954 ms on the second,
  545 to 5,923 ms on the first arm without it and 3,090 to 16,469 ms on the second,
  with medians of 4,482,
  2,655,
  1,085 and 6,239 ms.
  The medians of the same binary differ by 1.7 times in the first pair and 5.8 times in the second,
  and whole-commit wall clock spans 1,446 to 22,335 ms,
  so these records cannot tell the two builds apart:
  the change buys a rule,
  not time.
- The host was at load average 124.81,
  105.99 and 89.60 (one,
  five and fifteen minutes)
  at 2026-11-09T07:41:33-05:00,
  before the round,
  and at 58.83,
  59.99 and 70.51 at 2026-11-09T08:12:52-05:00,
  when it ended,
  on six CPUs,
  with `/proc/pressure/io` reporting `full` at 46.40 percent over ten seconds at the end;
  no load was generated for this measurement,
  and every probe ran in a `systemd-run --user --scope` scope.

Proof by removal,
each control run against the branch's own change undone:

- `a_made_tree_names_the_command_that_made_it` failed with `the tree names the command that made it`
  once `mark_the_owner` was no longer called as the tree is made.
- `a_made_tree_is_renewed_while_its_command_lives` failed with
  `a made tree is renewed while the command lives: ["git_policy_cli-", "landing::servic", "unit-test-watch"]`
  once the renewal was no longer started.
- `a_cleanup_names_only_a_scratch_directory_of_the_worktree` failed with
  `.../monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-`
  accepted at the decoder,
  once the cleanup was checked with the earlier rule
  (`is_absolute` and `starts_with(git_dir)`) again.
  That rule was also the one that refused the place this release makes the tree in,
  and it accepted any directory at all inside the Git directory,
  so a request could ask the service to remove one the wrapper never made.

What this leaves open:

- A released scratch tree's removal starts where its job sits in the batch,
  before that batch's landings run,
  so a client killed while its request is still queued can have its tree removed before its own landing reads it.
  The fix is to collect the batch's releases and start them after its landings;
  no test drives a killed client's queued request today,
  so the change is left for a round that can prove it.

## What the preparation spends, named (2026-11-09T06:41:47-05:00)

The record ranked the preparation first at 21.7 percent of the uncovered time and could not say which step spends it.
The record now names every stretch of a preparation that holds no Git child,
as `prepareParts`
(`package/git-policy/cli/SPEC.md`,
"Timing logs"),
each figure the stretch's own time with the Git children that ran inside it taken out,
because those are recorded apart in `steps`.

The split,
over the sixty-eight commits the four fixtures recorded with this build,
in whole milliseconds:

- `index`,
  the build of the private commit index,
  which is where the question whether the real index's stat data still
  describes the worktree is asked:
  median 3,811,
  max 13,405,
  59 of the 68 commits over 1,000 ms,
  and 310,068 ms in all.
  In the four commits that also paid `git update-index --refresh` it was 460,
  1,040,
  144 and 215,
  because that Git child had answered the question already.
- `policies` (the policy pass over the staged content) median 68,
  max 6,389,
  and 5 commits over 1,000;
  `unmergedFiles` (the listing of unmerged entries and the staging of them) median 75,
  max 1,684;
  `changedPaths` (the candidates against the base) median 22,
  max 1,586;
  `changes` (the tree check and the change list the service is sent) median 1,
  max 5,271;
  `scratch` (making the tree,
  and removing a previous attempt's) median 0,
  max 1,103;
  `layout` max 36,
  `conclusionState` max 7,
  and the other six names,
  `unmergedBefore`,
  `capture`,
  `stagedDeletions`,
  `interactive`,
  `worktreeRecords` and `treeCheck`,
  0 or 1 in every commit.
- The preparation's Git children,
  taken out of those figures and listed in `steps`:
  `git update-index --refresh` 122,219,
  137,331,
  162,420 and 172,689 in the four commits that ran it,
  `git read-tree` 15,
  `git diff` 13,
  and `git config` and `git rev-parse` one to nine,
  in all sixty-eight commits.
  So a preparation is either one long Git child (the cold refresh) or the in-process `index` stretch,
  and nothing else is
  large enough to look at.

What `index` spends it on:
`initialize_commit_index` asks `refresh_stat_data_when_stale`,
which asks `index_stat_data_needs_refresh`
(`package/git-policy/cli/src/native/commit/commit_index_stat_staleness.rs`),
and that question reads the whole index and stats the file of every entry whose stat data it must check.
`strace -f -c` of one commit in the same fixture counted 25,460 `newfstatat` calls at 2.09 s and 22,915 `statx` calls at
0.52 s,
against 22,190 entries in the index,
so the walk costs one or two filesystem questions per tracked file on every commit,
and it answers "not needed" whenever the last commit refreshed the index,
which is the usual case in a loop of agents.
The Git child it saves is far dearer when the answer is "needed":
the same fixtures paid `git update-index --refresh` 122 to 173 seconds cold,
against 460 to 1,040 ms for the walk in the same commits,
so the walk earns its place and is not the thing to remove.

The next change,
which this record proposes and does not build:

- Name the walk apart from the index build.
  `index` holds both,
  and the two are different work;
  one more name in the same table tells the next reading which of them the tail is,
  and one boolean recording the answer the walk gave says how often the expensive Git child is even needed.
- Then take the walk off the commit's path rather than making it cheaper:
  the answer it computes is a fact about the index and the worktree,
  not about this commit,
  and it is asked again by the next commit with the same index.
  Starting it beside the shadow's creation,
  which waited 16.0 seconds in one of these fixtures with no Git child at all,
  or asking it in the service while the previous landing holds the index,
  would hide its 1.4 s median inside a wait that already happens.
- Not proposed here,
  and not to be touched from this branch:
  what the landing does with the index,
  which another branch is changing.
  The measurement under "What the preparation spends,
  named" is of the preparation only.

What this could not determine:

- Which of the two `stat` families per entry is the walk's own and which belongs to the index reader,
  because `strace -f -c` counts the commit's calls and not each caller's.
  The next reading wants the walk's own figure,
  which the first bullet above adds.
- Whether the live commits pay the same share:
  the installed release writes no `prepareParts`,
  so the 866 commits of this record cannot be split retroactively,
  and the next reading of the live log can.

## The shadow's creation, measured and made cheaper

The creation that the flush's change left unsized is the stretch between the end
of the preparation and the shadow's first reference step:
the skeleton's directories and `HEAD`,
its config and alternates,
the links to the real common directory's shared entries,
the conclusion copy,
and then the ref store.
The records name that stretch exactly,
because every phase and every Git step carries its offset from the command's start,
and it is where 93 to 99 percent of each measured hour's `buildMs` sits,
except the last hour of the window,
19:00,
where it is 56 percent and the
  machine stopped stalling,
so measuring `buildMs` and measuring the creation were the same measurement in
every hour that stalled.

### What it costs in the records

Over the day ending 2026-11-09T19:01Z,
733 records of the log and its one generation,
696 of them landed on their first attempt:

- the creation's median is 3742 ms,
  p90 20 047 ms,
  p99 71 966 ms,
  max 167 656 ms.
- the whole commit's median over the same records is 13 356 ms.
- it moved with the machine and not with the tree:
  by hour,
  medians of 5846 ms at 11:00,
  1970 ms at 12:00,
  5435 ms at 13:00,
  3925 ms at 14:00,
  3668 ms at 15:00,
  3814 ms at 16:00,
  6989 ms at 17:00,
  2669 ms at 18:00 and 183 ms at 19:00.
- the same records count the shadow ref store's Git starts:
  one `git for-each-ref`,
  two `git symbolic-ref` and one `git update-ref` per commit,
  with medians of 5 ms,
  3 ms and 4 ms and maxima of 250 ms,
  8815 ms and 1924 ms.

A read-only probe of the calls themselves says the Git directory was not charging
more per lookup than a fixture:
2000 `stat` calls on the repository's own `.git` (79 entries) took 5.8 us each,
2.7 us for a name that does not exist and 26.0 us for a directory listing,
against 3.5 to 4.5 us,
2.8 us and 22 to 24 us in the shaped fixtures.
What the creation waits for is the machine's device,
hour by hour;
what the wrapper controls is how many calls it stands in that queue.

### The calls it makes, and what two of them were for

One wrapped commit in a shaped fixture of this tree,
traced with `strace -f -T -e trace=%file,%desc`,
whose common directory carries the 39 private index copies the person's own holds,
made 392 filesystem calls in that stretch:
`symlink` 47,
`statx` 89,
`read` 96,
`openat` 45,
`close` 31,
`readlink` 24,
`getdents64` 14,
`pread64` 12,
`mkdir` 8,
and the rest under 10 each.

Two of those groups were the wrapper's own doing and nothing else's:

- Every entry of the real common directory that the transaction does not keep
  privately was linked into the shadow,
  and the wrapper's private copy of an index sat on neither list.
  The copy is made beside the real index once per command that predicts what
  `git add` would stage,
  and the person's common directory held 39 of them,
  left by commands that died before they removed their own.
  Every commit created two filesystem entries for each.
- Each link was preceded by a lookup of the name it would be made at,
  to remove a template entry that only the reftable path's `git init` creates.
  On the files path a fresh shadow holds nothing at those names,
  so the lookup was one call per entry per commit for an answer that is always
  "nothing there".

### The change, in four commits

- `is_private_shadow_entry` now names the private index copies by their prefix,
  read from `PRIVATE_INDEX_PREFIX` (`shadow_links.rs`),
  so the writer and the reader of the name are one constant.
- A link is made first and a refused one removes what sits at the name and links
  again,
  so the common case is one call per entry (`shadow_links.rs`).
  Windows,
  where a shared file is a copy and not a link,
  keeps the removal first.
- On the files backend the wrapper writes the shadow's private target ref,
  its symbolic refs and its `HEAD` itself (`shadow_refs.rs`),
  as it already wrote `packed-refs`:
  a loose ref is the object name and a line feed,
  a symbolic one is `ref: <target>` and a line feed.
  The reftable backend keeps Git,
  because its refs live in tables only Git writes.
- A branch that is itself a symbolic ref to another branch keeps its chain,
  and the private target is written where that chain ends,
  at most `SYMBOLIC_REF_CHAIN_LIMIT` steps,
  which is Git's own limit
  (`shadow_refs.rs`).
  `git update-ref`,
  which the third step replaces,
  follows such a chain;
  a direct write at the first name would turn the chain into a direct ref that a
  hook can tell apart.

Red controls first,
for the two steps that changed what the wrapper does:

- `shadow::shadow_links::tests::the_wrappers_private_index_copies_stay_out_of_the_shadow`
  failed with
  `assertion failed: std::fs::symlink_metadata(shadow.join(dead.file_name().expect("name"))).is_err()`.
- `shadow::shadow_refs::tests::a_files_snapshot_writes_every_ref_without_a_git_start`
  failed with the three starts that wrote a few bytes each
  (`symbolic-ref refs/heads/alias refs/heads/main`,
  `update-ref refs/heads/main 9ff701ae...`,
  `symbolic-ref HEAD refs/heads/main`).
- `shadow::shadow_refs::tests::a_files_detached_head_is_written_where_git_reads_it`
  failed the same way on `update-ref --no-deref HEAD`.
- `shadow::shadow_refs::tests::a_files_target_that_is_a_symbolic_ref_is_written_at_the_end_of_its_chain`
  failed with `left: "refs/heads/main", right: "refs/heads/alias"` on
  `git symbolic-ref HEAD` of a shadow whose `HEAD` names `refs/heads/alias`,
  and that was the cross-vendor review's finding against the third step.

Both ref controls then read the store back through real Git:
`git symbolic-ref HEAD`,
`git rev-parse` of the private target and of the snapshot's tag and second branch,
and `git symbolic-ref` of a symbolic ref.
The reftable and byte-named-ref controls are unchanged and pass.
The link step is a shape with no behaviour of its own;
the existing controls for a replaced template entry,
a linked worktree's private
`info` and the shared `rr-cache` cover it.

### After

The same method,
the same fixture shape,
the same 39 private index copies,
one
commit traced with that call filter (each trace is its own fixture,
so the
figures are per call and not per run of the same tree):

- 306 calls against 392;
  `symlink` 8 against 47,
  `statx` 42 against 89.
  The first commit of a fixture,
  which does more,
  fell from 1695 calls to 1612,
  with `symlink` 44 to 5 and `statx` 470 to 426.
- the commit started one Git for the shadow's refs where it started three:
  `git for-each-ref` stays,
  and the `git symbolic-ref` and `git update-ref` are gone from the trace
  entirely,
  and from the records:
  3 starts in each of 8 commits before,
  1 in each of 8
  after.
  The person's own records count four before the change,
  one `git for-each-ref`,
  one `git update-ref` and two `git symbolic-ref` per
  commit,
  and the shaped fixtures have one symbolic ref fewer than that repository.

Three arms of 8 commits each,
alternating which arm went first,
in fixtures of this tree's shape that carry the common directory's 39 private
index copies,
where the first two arms are the same binary on unchanged code:

- whole commit,
  by the record's own total:
  median 719 ms (241 ms to 30 370 ms) and,
  on the identical arm,
  842 ms (254 ms to 22 862 ms),
  against 285 ms (246 ms to 8070 ms).
  The identical arms differ by 1.17 times at the median,
  and each of them caught one stall of 23 to 30 s that the changed arm did not,
  so this record claims no whole-commit improvement from the medians alone.
- the creation's own stretch:
  median 5 ms and 4 ms before (3 ms to 38 ms and 3 ms to 50 ms),
  median 2 ms after (2 ms to 8 ms).
- the shadow's Git starts:
  3 in every before record,
  1 in every after record.

What stays,
and why:

- `git for-each-ref`,
  one start per commit:
  the snapshot must be the real repository's refs as they stood at invocation,
  and above `REF_LISTING_LIMIT` refs the wrapper's own reader declines,
  after reading `packed-refs` and checking each ref against the object database.
  That attempt is most of the creation's 96 `read` calls,
  55 of the 96 in the traced commit read the real `packed-refs`,
  and its 12 `pread64` calls are the object-database checks,
  so this is the largest group the creation still holds.
- the skeleton's 8 `mkdir` calls and its `HEAD`,
  the shadow's config and alternates,
  and the conclusion copy's directory:
  they are the repository Git is handed,
  and each is one call.
- the `readlink` calls in path resolution and the `getdents64` of the one
  directory listing,
  which the links need to know what to link.

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

- Taken:
  measured and made cheaper by the section
  "The shadow's creation,
  measured and made cheaper".
  It was 93 to 99 percent of each hour's `buildMs`,
  and its calls fell from 392 to 306 in a fixture of this tree's shape,
  with the shadow's Git starts down from three to one per commit.
- Still open,
  and now named:
  the wrapper's other scratch directories in a common directory
  (`cli-git-scratch`,
  `cli-git-captures`,
  `monochromatic-git-policy-cli-scratch`,
  `monochromatic-git-policy-cli-merges`,
  and its own `*.jsonl` logs) are still linked into every shadow.
  They are six or seven entries,
  not thirty-nine,
  and no measurement says a hook needs them,
  so this record does not take them;
  the private index copies were taken because
  a common directory had collected one per command that died.

## The preparation's split, measured on a shaped fixture

What this answers:
which step inside `prepareMs` spends the milliseconds,
weighed on real commits through a built wrapper in a disposable repository of this tree's shape,
and what the record states so that the next slow commit explains itself.

The record every commit writes now carries `prepareSpans`
(`package/git-policy/cli/SPEC.md`,
"The longest quiet gap of every Git step and tool a command ran"):
each step of the preparation by name,
with how often it ran,
the whole of its time and its longest run.
The names are the index's stat check,
the one read of the index,
the walk of those bytes for the seed decision,
the index copies,
the answer of whether the index already holds the base,
the writes of corrected entries,
the candidate set the content policies of a pass share,
and every policy with the lifecycle point whose pass ran it.
The owner's rule for an unexplained slow commit was to log more,
and the field is that logging:
it costs one clock reading at each seam and one `write` in the record the commit already writes.

How the split was measured:

- The fixture is a clone of this checkout's `HEAD` with its remote removed:
  22,457 tracked files and a 3,016,055-byte index,
  reflinked for each arm,
  its index refreshed and `core.untrackedcache` set,
  with this repository's `post-checkout`,
  `post-commit`,
  `post-merge` and `pre-push` hooks and `git-lfs` on the arm's `PATH`.
- The harness is `mise run //package/git-policy/cli:native:perf -- --only commit-service --samples 20 --shape <clone>`,
  whose two arms alternate every round:
  the same 20 rounds ran through the release wrapper and through `/usr/bin/git`,
  so the paired delta of a round holds what the wrapper added while both arms met the same host.
- The host was under the pressure other sessions keep up:
  load average 61 to 109 on eight CPUs,
  and no synthetic load was generated.
  Every probe ran in a `systemd-run --user --scope` with
  `CPUQuota=700%`,
  `MemoryMax=12G`,
  `TasksMax=900` and `IOWeight=10`.
- No run waited for a quiet host and none called this one quiet:
  the spread of one binary on unchanged input is reported beside each figure,
  and the two runs of the unchanged wrapper on the same fixture
  (paired delta median 6,629 ms and 7,685 ms,
  p95 19,725 ms and 25,659 ms,
  and 13 then 12 of the 20 rounds over the owner's five seconds)
  show how much of any single figure is the host.

The split over 20 wrapped commits:

- `prepareMs`:
  median 1,919 ms,
  p95 19,815 ms,
  max 50,596 ms.
- `submitMs`:
  median 783 ms,
  max 21,835 ms.
- `gateMs`:
  median 1,337 ms,
  p95 6,408 ms.
- `buildMs`:
  median 3 ms:
  every commit of this shape took the route where the service builds the commit from the request,
  so no scratch `git commit` ran.
- The named steps,
  each a median over the 20 commits:
  - `candidates prepared` 3,404 ms,
    p95 15,077 ms,
    max 17,491 ms.
    It sums the pre-forward pass and the post-commit gate of each commit,
    so it can exceed `prepareMs`.
  - `policy final-newline at pre-forward` 1,468 ms.
  - `policy final-newline at post-commit` 1,173 ms.
  - `policy security/forbidden-strings at pre-forward` 31 ms,
    with one run of 41,481 ms.
  - `index stat check` 22 ms (max 816 ms),
    `index read` 19 ms (max 490 ms),
    `index copy` 0 ms (max 940 ms),
    `index equals base` 2 ms.
  - Every other policy:
    0 to 2 ms.

The answer to the question this record could not settle is that no policy spends the preparation's time:
the candidate set does.
The first content policy that asks for it,
`final-newline`,
pays for the private index its listings read,
the listings themselves,
and the content behind each path,
and every later policy of the pass reads what it paid for.
`final-newline` was named as the cost in the first reading of the new field
(`prepareSpans` with the policy names only),
which is why the set has a name of its own now.

What this split still does not name:
the child Git processes the in-process readers fall back to.
Those start outside the step ledger,
so a commit whose preparation waits on one shows the wait in the phase and in no step.
That gap is a defect of the record,
not of the fixture,
and it is what the next change fixes.

The bar,
on the same 20 rounds:

- `/usr/bin/git` median 92.1 ms and p95 1,598 ms.
- The wrapper median 8,043 ms,
  p95 25,741 ms,
  largest 52,225 ms.
- The paired delta:
  median 7,685 ms,
  p95 25,659 ms,
  max 35,956 ms,
  so 12 of the 20 rounds were over the owner's five seconds and eight were not.

## What the split changed

Three commits came out of the split,
and the third was measured against the unchanged binary rather than against a
single run.

### The record lists the reads the in-process readers decline

`56a9f07fe5`.
A commit that waited on a Git child the in-process readers fall back to showed that wait in its phase and in no step:
one preparation of 50,596 ms explained 7,094 ms of itself.
`run_metadata_git_within` now notes the child it starts in the same ledger every other step uses,
so the record names it and how long it ran,
and the control closes that gap by switching the readers off and asking the ledger for the child.
This changes what the record says and no work a commit does;
it is what let the next measurement see the copies at all.

### A request that only lists the index reads it in place

`195f167602`.
The candidate set copied the index into a private directory for every request.
A commit makes that copy after it already holds its own private index,
so the copy is three megabytes read and written for nothing:
the harness's `strace` put that third copy at 2,288 ms and 4,813 ms in two commits,
against 0.130 ms and 0.125 ms for the two copies the commit itself makes.
Only a request that stages into the index can change it,
so `git add` prediction and the direct scope keep the copy,
and the requests that only list it read the index the caller named,
which that command has already isolated from the live one.
The control closes the Git directory for writing and asks a staged request for its candidates:
a preparation that copies fails there with `Permission denied`,
and this one answers with the same candidate,
leaving the index byte for byte as it was.

The same 20-round harness and the same shaped fixture,
the unchanged binary run with it:

- The unchanged binary,
  four runs:
  paired delta median 6,629 ms,
  7,685 ms,
  8,276 ms and 6,955 ms,
  13,
  12,
  12 and 10 of 20 rounds over the owner's five seconds,
  `prepareMs` median 4,301 ms,
  1,919 ms,
  3,925 ms and 2,795 ms,
  `candidates prepared` median 3,404 ms,
  3,458 ms and 2,899 ms in the three runs that could name it,
  and the direct arm's own median 92.6 ms,
  92.1 ms,
  82.3 ms and 105.6 ms.
- This change,
  one run:
  paired delta median 1,988 ms,
  8 of 20 rounds over five seconds,
  `prepareMs` median 214 ms,
  `candidates prepared` median 7 ms,
  and the direct arm's median 71.1 ms.
- The spread of the unchanged binary's four medians is 6,629 ms to 8,276 ms,
  a quarter of the lowest of them,
  while the direct arm's own medians moved 71.1 ms to 105.6 ms,
  half of the lowest,
  across the same runs.
  This change is 3.3 times below the lowest of the four,
  so the host alone does not explain it.

### The clone that the measurement rejected

`42b2c1d4c6`,
reverted by `a30eceb87f`.
Sharing the index's extents instead of copying its bytes was the first attempt at the same waste.
With the disk busy it is slower than the copy:
`index copy` median 6,120 ms across a commit's four copies against 0 ms for the two the older record could name,
`candidates prepared` median 6,172 ms against 3,458 ms,
and a paired delta median 16,458 ms against 8,276 ms,
17 of 20 rounds over five seconds against 12,
while the direct arm moved only from 82.3 ms to 85.0 ms.
A clone of a freshly written index waits for that index's own writeback inside the copy the commit is waiting on,
and a byte copy leaves that writeback to happen after the commit.
One run under a lighter disk put the same binary at 452 ms,
which is why every figure here is repeated beside the
unchanged binary and none is reported alone.

What the trace of a later commit shows the reverted copy was:
`copy_file_range`,
not a byte copy.
`std::io::copy` of two `File`s asks the kernel to copy the range,
and on btrfs that shares the source's extents,
so the copies of every traced commit were reflinks at 4 ms to 172 ms,
and the two copies this record blames were never byte copies after `a30eceb87f`.
That is why the stall stayed in the records measured after the revert.

### Both private copies are written from one read of the live index

`79050a88b2`.
The third entry of "What would take the tail under five seconds",
measured.

A preparation read an index's bytes four times over:
the stat check read the real one,
the captured copy was made from it,
the seed decision read that copy,
and the commit's own copy was a copy of it.
An index commit made three of those reads:
the stat check,
the captured copy,
and the commit's own copy read from the captured one.
`IndexBytes` (`commit_index_bytes.rs`) reads the file once and writes every copy from those bytes,
so both private copies hold one state by construction:
a staging that lands in the real index after the read reaches neither,
and none of another command's staged work can be recorded under a commit that did not capture it.
An index commit decides no seed,
so only an explicit-path preparation walks those bytes (`index seed`),
and both kinds state the one read as `index read`.

The measurement,
one session,
the same shaped clone and the same 20-round harness,
two runs of the unchanged binary and two of this change:

- The unchanged binary:
  paired delta median 7,483 ms and 3,075 ms,
  p95 15,666 ms and 17,169 ms,
  14 and 6 of the 20 rounds over the owner's five seconds,
  `index copy` a median 180 ms and 2 ms per round,
  its largest 3,381 ms and 5,716 ms,
  8 and 6 rounds with a copy over 900 ms,
  and 20,410 ms and 16,083 ms in all.
- This change:
  paired delta median 4,442 ms and 547 ms,
  p95 13,015 ms and 5,416 ms,
  10 and 2 of the 20 rounds over five seconds,
  `index copy` a median of 0 ms per round,
  its largest 328 ms and 1,216 ms,
  no round over 900 ms in one run and one round in the other,
  and 407 ms and 1,369 ms in all.
- The copies' own spread between the two runs of the unchanged binary is 20,410 ms against 16,083 ms,
  so their fall to 407 ms and 1,369 ms is twelve times below the lower of those.
- The one read of the index costs what a read of a warm file costs:
  `index read` and `index seed` are each under a millisecond in every round of both runs,
  against 19 ms and 47 ms for the seed's own read of a private copy before.
- What the copies are now,
  in the trace of six wrapped commits:
  no `copy_file_range` call at all,
  the index read in blocks of 0.39 ms to 0.53 ms,
  and each copy written by `write` in 0.03 ms to 1.42 ms,
  against two `copy_file_range` calls per commit before.
- The paired delta did not move outside the host's own spread:
  the unchanged binary's two runs differ by 2.4 times in their medians,
  and the direct arm's median halved between the two pairs (281 ms and 337 ms against 152 ms and 150 ms),
  so this change's runs met a lighter host,
  and what it removed was worth seconds in the rounds where a copy stalled,
  not seconds of every round's delta.
- What the tail is left to:
  `submitMs` holds the rounds over five seconds in both runs of this change
  (a median 181 ms and 1,398 ms,
  with single rounds at 12,967 ms),
  which is the landing's queue and lock,
  the second entry of "What remains".
- Both arms signed every commit:
  the shaped clone carries SSH commit signing in its own configuration.
  The unchanged binary's runs signed with a key another session's scratch root held,
  and this change's runs sign with one made for them;
  a signature is the same work for either arm of a round either way.

## What remains over the five seconds

With the copy gone,
eight of the 20 rounds still take more than five seconds beyond native Git,
and the phases of those rounds name three causes:

- The service's landing:
  `submitMs` holds 6,136 ms,
  7,518 ms,
  10,324 ms,
  19,271 ms and 25,114 ms in five of the eight.
  That is where a landing waits for the queue and the index lock,
  which this record's ranking already put second and which this task left alone.
- The scanner's first use of a rule cache:
  one round holds 24,669 ms in `policy security/forbidden-strings at pre-forward`,
  and the trace of a commit in a fresh `HOME` shows what it is:
  `fsync` calls of 6.7 s and 3.0 s and a `rename` of 1.5 s on
  `$HOME/.cache/forbidden-strings/<release>/<hash>/`,
  the compiled rules written and flushed inside the commit that first needs them.
  Every run of this fixture starts from an empty `HOME`,
  so its first commit pays that;
  a machine pays it once per cache invalidation,
  not per commit.
  The commit no longer waits on that write:
  "The scanner's compiled rules are published off the commit's path"
  holds the change and its measurement.
- What is left of the preparation:
  three rounds hold 5,938 ms,
  4,538 ms and 3,531 ms there,
  and the named steps say it is the commit's own copies of the live index:
  `index copy` takes 5,793 ms in one of them and 4,227 ms in another,
  once each,
  while `candidates prepared` is 6 to 8 ms and every policy is under 30 ms.
  A cold three-megabyte read of `.git/index` costs seconds on this disk,
  and the wrapper reads that index once more than native Git does.
  Measured again in one session on 2026-11-09:
  both copies are now written from one read of that index,
  `index copy` holds 407 ms and 1,369 ms in all over 20 rounds against 20,410 ms and 16,083 ms
  ("Both private copies are written from one read of the live index"),
  and the reading that places a read of `.git/index` in this list is corrected there.

## What would take the tail under five seconds

Not this change,
and not the ones this record could make inside the client alone:
the tail is the landing's queue and lock,
the scanner's first cache write,
and the host's own stalls inside reads
native Git also makes.
What would,
in the order the numbers rank it:

- Shorten what a landing holds and how long a commit queues for it,
  which is the second entry of "What remains" and the only one that also shortens other agents' commits.
- Move the scanner's rule-cache write off the commit's path,
  by building it in the service or before the commit starts,
  which is worth up to 24 seconds on the first commit of a cold cache.
  Done:
  "The scanner's compiled rules are published off the commit's path"
  takes the write and its device waits off the command that needs the rules,
  and leaves only the compile there.
- Read the live index once per commit where the wrapper reads it twice:
  the captured copy and the commit's own copy hold the same bytes at that moment,
  and `git add` reads the live index again in the same command,
  which is the 4,227 ms to 5,793 ms of `index copy` that the rounds over the bar hold.
  Done (`79050a88b2`,
  under "Both private copies are written from one read of the live index");
  what it removed was the copies' second and third pass over the bytes and not the reads,
  which the same records put at 4 ms to 182 ms.

## The scanner's compiled rules are published off the commit's path (2026-11-09T16:45:00-05:00)

What this answers:
the wait one commit of the window above held for 24 669 ms in
`policy security/forbidden-strings at pre-forward`,
whose trace was `fsync` calls of 6.7 s and 3.0 s and a `rename` of 1.5 s on
`$HOME/.cache/forbidden-strings/<release>/<hash>/`.

Two things put that wait on the commit:

- The command's own process wrote the compiled artifact,
  because the scanner publishes what it compiles on a miss.
  The warm this record already had made the miss rare and never took the write off the command that hit it:
  a repository's first landing,
  a rules text a checkout brought,
  and a release whose cache partition is new all leave the write where it was.
- The publisher synced the artifact and then its directory before it answered,
  so the writer waited for this host's device.

What changed,
in three commits on `perf/git-policy-cli-scanner-rules-cache-off-the-commit-path`:

- `perf(forbidden-strings-cli)` `444f215485`:
  the publisher fills a private temporary and renames it over the artifact,
  and syncs neither the file nor its directory.
  Every read validates the whole artifact (magic,
  schema,
  scanner version,
  platform,
  source digest,
  payload digest,
  engine digest and trailing bytes),
  so a copy a crash tore is rejected and recompiled,
  and the artifact is derived data the scanner's own README already said may be deleted.
- `feat(forbidden-strings-cli)` `1086dca458`:
  a load says whether a miss may write the cache (`CacheAccess::ReadOnly`),
  and `compiled_rules_artifact_exists` answers beforehand whether the cache holds a file for the rules text at all,
  with one `stat` rather than a second read of the artifact.
- The wrapper's change:
  a command whose cache holds no artifact for its rules text asks the service of that worktree to compile and publish
  (the one-way `warm` message a worktree-moving command already sends),
  and then loads the rules without writing,
  so the rules it compiled are only ever a scan's input.
  It publishes the artifact itself in the two cases where nothing else can:
  a worktree with no service running,
  and a rules list taken from `FORBIDDEN_STRINGS_RULES` in a repository whose configuration names no rules files,
  which the service's environment does not carry and would compile into another text's partition.
  A configuration that names rules files wins over the variable,
  so a repository with both leaves the list to the service,
  and a cache that already holds an artifact is read here and writes nothing either way.

Controls,
the red one run against the branch's change undone
(a read-only load made a plain write):

- `candidate::scanner_adapter::tests::a_load_writes_the_cache_only_when_it_may` failed with
  `a load that may not write wrote: [".../forbidden-strings/v0.6.2/linux-x86_64/<digest>/rules.bin"]`,
  and passes with the change.
- `policy::policy_forbidden_strings::tests::a_command_with_no_service_publishes_the_rules_itself`
  holds the other side:
  a worktree with no running service publishes,
  and the load after it reads that artifact.

Two controls hold what must never change,
and both pass with the change:

- `candidate::scanner_adapter::tests::a_partial_artifact_is_never_used`:
  an artifact torn in the middle is refused as `invalid` rather than read as `missing`,
  and the rules file speaks instead.
- `candidate::scanner_adapter::tests::a_cache_of_other_rules_is_never_used`:
  the cache holds the artifact of one rules text,
  the rules files change,
  the load reports a miss for the new text,
  the old text's token passes and the new text's token is reported.

The measurement:

- The harness is `mise run //package/git-policy/cli:native:perf` with
  `--only commit-service --samples 6 --change-rules`,
  `--binary` naming each arm's release binary,
  and a shaped fixture of this tree's shape:
  a clone of this checkout with 22 462 tracked files and a 3.0 MB index,
  this repository's configuration,
  and its `post-checkout`,
  `post-commit`,
  `post-merge` and `pre-push` hooks with `git-lfs` on the arm's `PATH`.
  `--change-rules` appends one comment line to the optional local appendix before every sample,
  so every sample is the first commit after a rules change and meets a cache with no compiled form of that text.
- What is read is the commit's own record:
  the `longestMs` of the `policy security/forbidden-strings at pre-forward` span
  (`prepareSpans` in `.git/monochromatic-git-policy-cli-commit-timing.jsonl`,
  copied aside by the harness into its report).
- The host was under the pressure other sessions keep up:
  load average 57 to 119 on 8 CPUs,
  `/proc/pressure/io` reporting `full` at 19 to 38 percent over ten seconds,
  while these runs went end to end from 15:48 to 16:37.
  No load was generated,
  and every probe ran in a `systemd-run --user --scope` with
  `CPUQuota=600%`,
  `MemoryMax=24G`,
  `TasksMax=900` and `IOWeight=10`.
- Each arm ran twice,
  so the spread of one binary on unchanged input stands beside each figure.

The policy's own step,
per sample,
in whole milliseconds:

- Before the change:
  1683,
  1712,
  2644,
  3558,
  8850 and 9130 (median 3558) in the first run,
  and 1807,
  1834,
  3996,
  4848,
  4855 and 6789 (median 4848) in the second.
- After it:
  1331,
  1350,
  1358,
  1389,
  1435 and 1460 (median 1389) in the first run,
  and 1475,
  1708,
  1720,
  1746,
  1783 and 1895 (median 1746) in the second.
- The two runs of the unchanged binary differ by 1.36 times at the median,
  and the two runs of the changed binary by 1.26 times.
  The change is 2.6 and 2.8 times below the unchanged binary's two medians,
  more than either spread,
  and the slowest sample after the change (1895 ms) is below the unchanged binary's medians.
  The unchanged binary's two fastest samples (1683 ms and 1712 ms) are the whole of its overlap with the changed one.
- What is left in the step is the compile the commit still does,
  measured at 1.2 s to 1.3 s on 2026-11-07 and 1.7 s in the handover's own sample.
  The write is no longer part of it.
- The whole commit is not told apart by this:
  the six commits of the first changed run took 8312,
  4482,
  7271,
  4648,
  4043 and 5489 ms,
  and one of them held 8312 ms while its policy step held 1331 ms.
  The other phases' spread on this host is larger than this change.

What the traces say,
with `--trace-directory` and `strace --trace=fsync,fdatasync,rename,renameat,renameat2,openat,unlink,unlinkat`,
five samples per arm (tracing inflates the spans,
so these figures size the calls and not the step):

- Before the change,
  the commit's own trace held,
  per sample,
  an `fsync` of the temporary artifact at 14.3,
  2164.4,
  17.7,
  2392.1 and 4072.9 ms,
  the `rename` that publishes it at 0.2,
  5.4,
  3.7,
  0.2 and 150.1 ms,
  and an `fsync` of the digest directory at 14.6,
  13.6,
  8.4,
  14.3 and 899.1 ms.
  That is 29 ms to 4972 ms of the same five commits' policy steps,
  which held 2422,
  4878,
  2845,
  5586 and 8207 ms under the trace.
  The rest of each step is the compile.
- After the change,
  the same traces hold no `fsync`,
  no `fdatasync`,
  and no `rename` on any path under `.cache/forbidden-strings`,
  and the artifact is opened read-only.
  The same run's trace of the service holds five renames of
  `.rules.bin.<pid>.<n>.tmp` over `rules.bin` (0.0 to 0.6 ms each) from the service's blocking-pool threads,
  and no `fsync` on any cache path:
  the publication happened,
  in the service,
  and nothing waited for the device.

Where the write went,
by the same harness with `--samples 5` and no `--change-rules`:

- The changed binary's five commits held 1405 ms in the policy step for the first,
  which met an empty cache,
  and 24,
  34,
  30 and 29 ms for the four after it,
  which read the compiled artifact.
  The artifact those four read was published by the service,
  because the four traces of that arm hold no cache write:
  the compile-and-publish ask fills the cache for the next command.
- The cold sample still pays the compile.
  Taking that off the commit's path needs the rules text in a cache before the command asks,
  which the warm does after a landing,
  a restart and a worktree-moving command,
  and not for a text nobody has committed yet.

What this leaves open:

- A machine whose service declines the warm (its address space leaves less than `WARM_ADDRESS_SPACE`)
  keeps no artifact from a commit's compile,
  so those commits compile once each:
  correct and slower.
  `package/git-policy/cli/SPEC.md` states it beside the warm.
- The service's publication is still a job of the blocking pool that nothing bounds,
  so a device that stalls it delays the warm and never the commit.
- Whether the same syncs cost what they cost here on the person's own disk:
  the 24,669 ms round this record began with held 6.7 s and 3.0 s of `fsync` where these five held 29 ms to 4972 ms,
  and both are this host at different moments.

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
  `reads_ms` stays the whole,
  and the named parts never add up to more than it.
- `landing.lockHoldMs` is how long the landing held the index lock,
  which is what the commits behind it queued on,
  and `landing.queuedBehind` is how many requests were still waiting for their answer when the landing finished.
- `landing.syncs` is every file or directory sync the landing asked for,
  in the order it asked,
  each with whether the landing waited for it and how long it took.
  A background batch of a landing's records says in the journal how many it made and what they cost.

The service measures all three,
so the answer to a commit carries them and the record states them;
a service of a release that sends none leaves the field `null`,
never zeros.
The `answered` line of the journal states the same figures,
so the two files agree.

### The measurement

One binary,
two runs of four rounds of eight overlapping commits in a disposable clone of this repository
(452 990 objects and 22 462 tracked files),
started together on one branch so they land as one batch:

- run A:
  32 commits,
  median 4 280 ms,
  p95 8 623 ms,
  14 of 32 over 5 000 ms.
- run B,
  the same binary:
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

The commit's phases,
as the record counts them:

- prepare:
  median 3 487 ms in run A and 5 442 ms in B (p95 5 276 and 19 282).
- submit:
  median 755 ms and 10 843 ms (p95 3 025 and 24 813).
- gate:
  median 151 ms and 1 519 ms (p95 2 701 and 10 569).
- build:
  median 3 ms in both;
  after-landing,
  handoff and uncovered time under 80 ms.

The landing,
as the service splits it:

- landing:
  median 382 ms in run A and 10 756 ms in B (p95 2 773 and 17 113).
- the wait for another program's hold on the index:
  median 10 539 ms in run B (p95 14 249),
  which is 98 percent of that run's median landing.
- everything the landing itself does:
  reads median 29 ms in A and 15 ms in B,
  objects 17 ms,
  the intent journal 0 ms,
  the branch move 14 ms,
  the index update 89 ms.
- the hold on the index:
  median 165 ms in A and 190 ms in B (p95 207 and 2 771).
- queued behind a landing:
  seven commits for the first request of each batch and none for the rest,
  because the eight commits of a round land as one batch.
- six syncs per landing,
  always handed to the background and never waited for:
  193 over 32 landings in each run,
  named `intent-file`,
  `intent-dir`,
  `index-dir`,
  `indexed-file`,
  `indexed-dir` and `journal-dir`,
  with `lock-marker` only for the first landing of a run (the marker is made ahead after that).

### The same clone, one commit at a time

A third run,
of six commits one after another in the same clone,
so nothing ever queued:

- the wait for another program's hold on the index:
  median 1 ms,
  largest 3 040 ms.
- the hold on the index:
  median 305 ms,
  largest 724 ms.
- the index update:
  median 192 ms.
- reads:
  median 4 ms.
- queued behind a landing:
  none,
  in every one of the six.

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
(`git update-index --refresh`,
which takes `index.lock`),
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
  p95 2 701 ms in run A and 4 380 ms in the run one commit at a time,
  which "Where the gate's seconds were,
  and what a gate spends now" settles as the gate's own copy
  of the real index,
  gone from this worktree since `195f167602`;
- the per-commit refresh of the real index by the preparation,
  which the landing of a batch waits for and which a batch needs once;
  closed by the section below,
  which also corrects what the 10,539 ms of run B was.

The first is the preparation's,
which another branch is changing,
the second is the gate's,
and the third was taken off the preparation's path on 2026-11-09.

## What a batch's own refresh cost, and what moving it into the landing bought (2026-11-09T16:52:30-05:00)

Branch `perf/git-policy-cli-commit-preparation-without-the-real-index-lock`,
by the owner's instruction for it,
which quotes the requirements this record is judged by:
"commits vs native git should not take 5s longer.",
"Commits should never fail.",
"Use as few locks as possible."
and "never discard user data".
`doc/decision/cli-git-concurrent-commits.md:29` already says "Each commit prepares without the real index lock."

### What the 10,539 ms of run B was

The preparations do take the real index lock.
Measured directly,
with `strace --follow-forks` over one batch of two commits in a disposable clone
(`${HOME}/temp/agent/pi-commit-preparation-before-after/cause-lock.strace`):
the missing child
(`execve("/usr/bin/git", [..., "update-index", "--refresh", ...])`,
pid 1576606)
opened `.git/index.lock` with `O_RDWR|O_CREAT|O_EXCL`,
and its sibling (pid 1576613) was refused that file with `EEXIST`
and refreshed its own `commit.index.lock` instead.
So the batch's members did contend on the one file the landing needs,
and the landing could only take it after them.

The figure above that piece of this record read as that contention,
10,539 ms at the median,
is not a wait for a holder.
The service's `answered` line splits the acquisition:
`lock_wait_ms` is the whole of it,
`lock_stage_ms` is the marked lock file's own creation and flush,
and `lock_place_ms` is the wait for a holder (`service_index_found.rs`,
`WaitNotes`).
Over the landing agent's own run B document,
`package/git-policy/cli/perf/commit-landing-2026-11-09-overlapping-b.json`,
the three are
10,539 ms,
5,818 ms
and 17 ms at the median,
with `lock_place_ms` at most 4,720 ms.
The run one commit at a time in the same document set has `lock_place_ms` 0 at the median and 3,040 ms `lock_wait_ms`.
So the batch did not make the landing wait ten seconds for a holder:
the marker's flush on this disk took most of that,
and the holder wait was tens of milliseconds,
seconds at its worst.

### What changed

- `commit/commit_index.rs` refreshes the private commit index,
  after the copy and the selection are in it,
  and never the real one (`initialize_commit_index`,
  `initialize_retry_index`).
  The in-process question about the copy comes first
  (`commit_index_stat_staleness.rs`),
  so a copy that describes the worktree costs no Git start and no lock.
- `landing/service_land_follow.rs` records the worktree's stat data in the copy the landing is about to install,
  through the same question and the same refresh,
  while it holds the real index lock anyway.
  It leaves out the entries the reset just wrote,
  so it never hashes the batch's own change set,
  and `index/index_reset_file.rs` now hands back the names it reset,
  which is what lets a list derived from the branch leave them out too.
- A refresh that fails,
  is killed,
  or is ended at its step ceiling costs time and never a result:
  the landing installs the copy it built either way,
  and it flushes that copy only when Git replaced it.
- A path whose bytes the commit does not hold keeps no stat data,
  so a person's unstaged edit is still a change after the landing (`a_landing_whose_refresh_meets_an_unstaged_change_keeps_it_visible`).

The sharing native Git gets from the one index it owns now happens once per batch,
in the step that already holds the lock,
instead of once per client that then blocks the landing.

### The measurement, same method as the runs above

One binary per arm,
built from the same merged base,
four rounds of eight overlapping commits in a disposable clone of this repository,
four runs alternating:
base,
this change,
this change,
base.
The harness is the one this section's runs used
(`native-commit-landing-perf.ts`,
`${HOME}/temp/agent/gitp-bin`),
and the four documents are
`perf/commit-landing-2026-11-09-base-a.json`,
`...-mine-a.json`,
`...-mine-b.json`
and `...-base-b.json`
beside this record.

The commit's own time,
median of 32 commits,
p95,
largest,
and commits over 5,000 ms,
in whole milliseconds:

- base run A:
  929,
  3,134,
  3,159,
  0 of 32.
- this change,
  run A:
  785,
  4,936,
  4,940,
  0 of 32.
- this change,
  run B:
  23,461,
  35,551,
  35,553,
  24 of 32.
- base run B:
  17,260,
  50,124,
  50,126,
  32 of 32.

Two runs of one binary differ by 18.6 times at the median for the base and 29.9 times for this change,
and 16.0 and 7.2 times at p95.
The arms overlap completely,
so the whole-commit times of this host cannot resolve the change,
exactly as the 5.1 times spread of the runs above warned.

What the runs do say,
because it does not depend on the host's spread:

- A preparation never held the real index any more.
  In the two probes
  (three agents,
  three rounds,
  the same disposable source),
  the preparation's `git update-index` step is there on both sides:
  83 to 86 ms in round 1 for the base,
  4,249 to 4,277 ms in round 1 for this change.
  Beside the base's steps the real `.git/index.lock` shows as an unmarked file,
  held for the six samples of 5 ms that fall inside them;
  beside this change's steps the real index file is never seen,
  and the only spans that name it are the service's own marked lock,
  in the landings.
  The same trace of one batch for this change
  (`${HOME}/temp/agent/pi-commit-preparation-before-after/mine-lock.strace`)
  has one `O_RDWR|O_CREAT|O_EXCL` open of `.git/index.lock`,
  the clone's own,
  and after it the file is only ever created by the service's marker rename and read by its holder checks;
  the ten private index locks the run took are the preparations' own.
- The landing now runs that refresh itself when it is needed:
  `git update-index` appears among the landing's steps in 3 of 32 landings for this change and none for the base.
- The landing's hold of the index grew by the walk it now makes:
  median 39 ms against 79 ms in the quiet pair of runs,
  and 217 against 358 ms in the slow pair.
  Its `lock_place_ms` (the wait for a holder) stayed at 0 and 19 ms at the median,
  2,414 and 3,489 ms at the worst.
  Those longest waits appear on both sides,
  so they are not the preparations'.

### What this leaves

- The landing's walk over the copy is the one cost this change adds:
  22,000 entries stat'ed per landing,
  inside the lock,
  measured as part of the 39 to 79 ms above.
  Consecutive landings of one batch ask it again over an index the previous landing had just refreshed,
  so a stamp of the real index and the answer it gave would take the walk off all but the first landing of a batch.
  Not built here.
- A whole index that no longer describes the worktree (a moved repository) still pays one full refresh per cold batch,
  in the landing,
  and each of that batch's preparations still pays its own full refresh of its private copy,
  as the fallback already did.
  That case was not measured here:
  the controls use repositories of twelve files behind a sleeping clean filter
  (`commit_index_stat_tests.rs`),
  never a whole checkout.

## Where the gate's seconds were, and what a gate spends now (2026-11-09T16:10:00-05:00)

The gate's p95 of 2 701 ms in run A and 4 380 ms in the run one commit at a time
was not a policy's work and not a Git child of the gate.
Every candidate request used to copy the real index into a private one,
and the gate prepares the landed commit's candidates after the branch moves,
so one of those copies was the gate's own:
three megabytes read and written inside the command's process,
which starts no Git child and therefore stood in no `steps` entry of the record.
The trace behind "A request that only lists the index reads it in place"
put the copy a candidate request makes at 2 288 ms and 4 813 ms in two commits
under the host's input and output pressure.
`195f167602` removed the copy from every request that only lists the index,
the landed one included,
so a gate in this release copies nothing.

### The measurement

The method is the one the figures above came from,
run again on 2026-11-09 between 15:12 and 16:20,
under the load the host had then:
other sessions were running their own builds and a performance run of their own,
and the load average read 61 and 74 at one minute in the two samples taken while the runs were going,
on eight CPUs.
Each run makes eight overlapping commits in each of three rounds,
where run A above made four,
started together on one branch of a disposable clone of this worktree
(458 808 objects in packs and 22 556 tracked files),
with the wrapper's own service,
and its own `HOME`,
journal and runtime directory.
The spread of one unchanged binary comes first,
because it is the floor any difference is read against:

- `77d79100fd`,
  the binary the figures above were measured with:
  gate median 3 571 ms and 2 047 ms,
  p95 6 136 ms and 5 818 ms.
- `b9000c1929`,
  this worktree's `HEAD`,
  which carries `195f167602`:
  gate median 72 ms and 115 ms,
  p95 96 ms and 248 ms.

Two runs of one binary differ by 1.7 times at the gate's median for `77d79100fd` and 1.6 times for `b9000c1929`,
and by 1.1 and 2.6 times at its p95.
The two binaries differ by 18 to 50 times at the median and 24 to 64 times at the p95,
in every pairing of their runs,
which is not the host's spread.
One more binary settles which change did it:
`77d79100fd` with `195f167602` applied to it and nothing else,
built the same day,
holds the gate at 45 ms at the median and 55 ms at its p95 in one run of the same method,
with every one of its 24 commits between 33 ms and 57 ms.
That change touches the candidates a gate prepares and nothing else about the gate;
it also removes the two copies the preparation itself made,
so the control's commits are shorter than the record's overall and only its gate is compared here.

The arms were not interleaved:
the two runs of `77d79100fd` ran first,
then the two of `b9000c1929`,
then the control.
The control therefore ran last and fastest,
so the effect is not a quieter host.

The documents the runs wrote are `perf/gate-account-2026-11-09-*.json`:
`baseline-a` and `baseline-b` are the runs of `77d79100fd`,
`before-a` and `before-b` the runs of `b9000c1929`,
and `control` the run of the one-change build,
and `probe` the one-round run cited below.
Each holds every phase of its commits,
the gate's spans,
the four slowest gates with their spans and Git children,
and the service's `answered` lines.

### What a gate spends now, by name

The record's spans name every stretch of a gate in this release:

- `policy final-newline at post-commit`,
  14 ms and 28 ms at the median in the two runs of `HEAD`.
  It is the gate's first content policy,
  so it pays for the landed commit's candidate set:
  the `git diff-tree` of the landed commit (11 ms and 18 ms at the median)
  and the reading of the bytes that listing names.
  The record's shared `candidates prepared` span covers the same preparation at both lifecycle points,
  so the gate's own share of that work is read from this span and from `git diff-tree` beside it.
- `policy security/forbidden-strings at post-commit`,
  52 ms and 84 ms at the median.
  It holds the loading of the rules,
  the listing of the landed commit's parents (3 ms and 9 ms in `git rev-list`),
  the listing of a parent's whole tree for the content a move already put in history
  (25 ms and 35 ms in `git ls-tree`),
  and the scan itself.
- `policy markdown/autofix at post-commit` and `policy mono/catalog-specifiers at post-commit`,
  0 ms each:
  a commit of one text file has no candidate of either kind.
- what those four spans leave of `gateMs`,
  which is the facts provider and the pass itself,
  3 ms at the median and 16 ms at the largest.

A gate's tail under pressure is not a policy's either.
A first run of the same method with one round,
written before the two above and never discarded,
put `gateMs` at 163 ms at the median and 6 342 ms at its p95:
the slowest of its eight commits spent 6 298 ms of a 6 342 ms gate in `policy final-newline at post-commit`,
where its `git diff-tree` of the landed commit ran 6 275 ms,
while the other seven commits of its batch compiled the scanner's rule cache
(16 034 ms in that record's `policy security/forbidden-strings at pre-forward`,
against 14 ms for the same gate's `git ls-tree` later in the same command).
That is this record's second-ranked entry at work on the gate:
the rule-cache compile saturates the host inside the preparation,
and the gate's own Git children queue behind it.
It costs nothing on a machine whose cache is warm,
which is why the two three-round runs of the same binary never crossed 251 ms at their p95.

For a commit of one text file,
no step of a gate is worth removing.
Two more candidates are named and left:

- the scanner's rules are loaded again at the gate,
  in a state of its own,
  which the scanner's own command does in under 10 ms from a warm cache on this host
  (32.04 s cold,
  then 0.00 s twice);
- `markdown/autofix` runs at the gate only to warn in this repository's configuration,
  and an `error` in another configuration can block there,
  so skipping it would change a verdict for the 0 ms it costs a commit of one text file.

The one duplication left is the parent tree's listing,
which the same command's preparation already made for the same commit:
the preparation lists the base's whole tree to answer the same question,
and the landed commit's parent is that base unless a replay rebuilt the commit.
That is 25 to 35 ms of a 72 to 115 ms gate,
about 0.2 percent of a commit,
and sharing it between the two checks would need process-wide state between two readers,
which `candidate_store_commit_listings.rs` keeps out of the store on purpose.

What these runs leave unmeasured is a commit that changes Markdown:
the embedded linter runs at both lifecycle points for such a commit,
and every commit in these runs is one text file.
A measurement of a Markdown commit's gate would need a run whose agents commit Markdown.

## What this could not determine

- Which in-process step inside a preparation spends the median 332 ms.
  The records of this window name Git steps and phases,
  not policy rules.
  "The preparation's split,
  measured on a shaped fixture" settles it for the wrapper as it stands:
  the candidate set the content policies share spends it,
  and the copy of the index that set made for itself was the larger part until `195f167602` removed it.
- Whether the 351 s and 493 s stalls of 2026-11-08T23:26 were the conclusion copy's flush.
  Both sit in the same uncovered `buildMs` stretch,
  The records name Git steps and phases,
  not policy rules,
  and this host has no profiler.
- How much of the 351 s and 493 s stalls of 2026-11-08T23:26 was the conclusion copy's flush.
  Both sit in the stretch the section "The shadow's creation,
  measured and made
  cheaper" now sizes,
  between the shadow's `git config` read and its first reference write,
  so they are the creation's stall and not a phase boundary one;
  the flush was one call inside it,
  and the same stretch reproduced at 1279 ms,
  2530 ms and 5569 ms
  with a 24.9 s flush measured inside it.
  No record separates that flush from the 95 other calls of the file work around it,
  so which call a given stall held cannot be told from the log.
- How much of the 7.1 percent of `otherMs` is the scratch removal
  and how much is the wrapper writing what the person reads on standard error,
  which blocks when the reader is slow.
- Whether native Git would have waited as long in a flush under the same device state.
  Native Git flushes the index only when `core.fsync` asks,
  which is unset here,
  so this record compares step by step and not device stall by device stall.
- Whether a torn ref file of a shadow changes any recovery outcome.
  The ref store is written unsynced and not through a lock file,
  on the argument
  that nothing reads a shadow's refs after a crash and recovery removes the tree
  whole.
  The `packed-refs` half of that argument already rested on it,
  and both comments that state it cite
  `binary_recovery_tearing_shadow_tests.rs`,
  which this branch does not hold:
  `git log --all` finds no such path and no `binary/` control mentions tearing.
  The argument stands on the code's own shape,
  and no control on this branch exercises it.

## Decisions this record takes, open to the owner's veto

- The ranking of the four entries above changes:
  the landing's own share is smaller than the records suggested
  (382 ms of a 4 280 ms median commit,
  with no named read over 51 ms),
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
- The gate's two figures are settled,
  and no step of a gate is worth removing for a commit of one text file.
  Its p95 was the gate's own copy of the real index,
  which `195f167602` removed before the gate was measured again:
  a build of the record's binary with that one change applied holds the gate at 45 ms at the median,
  against 2 047 to 3 571 ms without it,
  and this worktree's `HEAD` holds it at 72 to 115 ms at the median
  with the landed commit's candidate set and the forbidden-strings scan inside it.
  Three candidates are named and left:
  the parent tree's listing,
  which repeats what the same command's preparation listed for the same commit
  and is worth 25 to 35 ms of that gate,
  the scanner's second load of its rules,
  worth under 10 ms from a warm cache,
  and `markdown/autofix` at the gate,
  which can block under a configuration that sets it to `error` and costs nothing to a text-file commit.
  What a gate still waits on under pressure is the preparation's cold rule-cache compile,
  which is the second entry above,
  and the gate's own Git children queue behind it:
  one run of the same method put a `git diff-tree` of the landed commit at 6 275 ms inside a 6 342 ms gate.
  A commit that changes Markdown is unmeasured:
  the embedded linter runs at both lifecycle points for it.
- The four ranked entries of this record are proposals,
  not work in progress.

## What the build phase waits on, named (2026-11-09T21:13:51-05:00)

What this answers:
what `buildMs` and `otherMs` held that no step named,
what concurrent commits share,
and what the record states now so that a future slow build names its cause.

The owner's words for the slow commits nobody could explain:
"If we can't figure it out,
then we have a defect and should log more."
This is that logging.

### Where the numbers come from

Two files the wrapper writes for every wrapped commit,
copied aside before they were read because the machine keeps committing:
`.git/monochromatic-git-policy-cli-commit-timing.jsonl` and its one generation `.1`,
1 077 records of the release `0.6.1-alpha.5` between 2026-11-09T17:36Z and 2026-11-10T01:44Z,
about twenty agents committing.
A record states its phases,
`startMillis`,
every Git step with the offset it began at and how long it ran,
and then the figures of the landing.

1 009 of those records name a step inside the shadow repository,
which is the first Git child of the shadow's ref snapshot,
`git for-each-ref`.
The stretch between the end of the preparation and the first of those steps is what no step covers,
and the section "The shadow's creation,
measured and made cheaper" already sized it.
This section states what it is,
what a stopped commit does with it,
and what concurrent commits have in common.

### The share of the build it is

Over the 940 records that landed:

- the whole command:
  11 745 636 ms in all,
  median 6 491 ms.
- `buildMs`:
  5 936 684 ms,
  50.5 percent of the whole.
- the stretch between the preparation's end and the shadow's first Git child:
  5 633 600 ms,
  94.9 percent of `buildMs` and 48.0 percent of the whole command,
  median 1 208 ms,
  p90 16 386 ms,
  p95 24 412 ms,
  p99 61 980 ms,
  max 141 560 ms.
  491 of the 940 records hold more than one second of it,
  and in 585 of them it is all of `buildMs` bar a hundred milliseconds.
  Over all 1 009 records that name a creation the same percentiles are 1 416,
  18 525,
  28 478,
  71 614 and 141 560 ms,
  541 of them over one second.
- every Git child of those records,
  10 579 steps of every phase added up:
  911 262 ms,
  15.3 percent of their `buildMs` and 7.8 percent of the whole,
  a median of 248 ms of steps for one commit,
  which is the figure the section "The steps,
  by what they cost" of the first reading puts at 153 ms for its own window.

So the question "what is inside `buildMs` that no step names" has one answer in these records:
the shadow repository's creation,
which is in-process filesystem work,
so `steps` cannot name it and the phases cannot split it.

The four records this task began from,
quoted whole in the fields that matter:

- `2026-11-10T01:36:44.711Z` on `refs/heads/main`:
  `totalMs` 65 222,
  `prepareMs` 1 502,
  `buildMs` 63 444,
  `submitMs` 131,
  and its longest step `git commit` at 65 ms.
  Its shadow's steps begin at an offset of 64 835 ms,
  so 63 333 ms of the build held no child at all.
- `2026-11-10T01:28:10.156Z`,
  `2026-11-10T01:28:10.158Z` and `2026-11-10T01:28:13.036Z`,
  on three different branches:
  `buildMs` 30 298,
  30 265 and 30 347,
  and the stretch before their first shadow step was 29 648,
  29 647 and 29 647 ms.

### What a stopped commit did with it

69 of the records stopped.
67 of them state `buildMs` 0 while their shadow steps ran,
and their `otherMs` is 1 313 193 ms in all against 1 105 754 ms of that same unnamed stretch.

The shape is one line of code:
`service_commit` added the build's elapsed time on its way out of the phase,
so a stop inside the phase left through the error path and the figure was never taken,
and what no phase covered then held the whole phase.

One record shows it plainly,
`2026-11-10T00:43:22.851Z`:
`totalMs` 91 526,
`prepareMs` 17 475,
`buildMs` 0,
`otherMs` 74 051,
and its shadow steps begin at an offset of 87 909 ms.
Its build was 70 434 ms and the record gave it to no phase.

### What concurrent commits share

The three records of 01:28 are the evidence,
and it is stronger than the reading this task began from:

- their creations lasted 29 648,
  29 647 and 29 647 ms,
  and ended at 01:28:08.347Z,
  the same millisecond.
- their preparations ended at 01:27:38.700Z,
  also the same millisecond,
  although their commands had begun 5.4 s and 1.4 s apart.

Over all 1 009 records,
a creation ends within 10 ms of another creation 122 times,
while a preparation ends within 10 ms of another preparation 41 times.
Among the 362 creations that lasted over five seconds,
the same comparison is 70 times against 16.

So the creations of concurrent commits finish together far more often than they start together:
something they share is released at one instant,
and the journal says what:
at 17:36:06.238748 and 17:36:06.241497 local,
two different commit services answered two requests,
and those two commits' creations ended at 17:36:06.024 and 17:36:06.026.
The journal's own last line before that is at 17:36:03.432,
so no line it carries was written in those 2.8 s,
and everything it carries resumes at once.
The 01:28 cluster's end is the same event:
20:28:08 local is when the journal resumes there.
The two queries,
for a reader who wants to see the gaps:
`journalctl --user --since "2026-11-09 17:36:02" --until "2026-11-09 17:36:07"` and
`journalctl --user --since "2026-11-09 20:27:00" --until "2026-11-09 20:29:00"`.

What they share is the repository's volume,
`/dev/mapper/crypt_sda` on btrfs with `compress=zstd:3`,
and the calls every creation makes on it:
a stall that ends releases every process blocked in one at the same instant.
This record cannot say what makes the volume stall:
the kernel journal holds no blocked-task warning in either window,
and the load the other sessions keep up is normal here,
which makes it neither a cause of a stall nor a reason to stop measuring one.
What it can do is make a commit say which call it was waiting in,
which is the change below.

### What the creation is, counted

One wrapped commit of the release built from this branch,
in a disposable repository of twelve files whose common directory holds the 40 private index copies this one does,
its own service under a runtime directory of its own,
traced with
`strace -f -ttt -T -e trace=%file,%desc`:

- from the first call that names the shadow directory to the `execve` of the step that reads its refs:
  193 to 194 calls in 8.0 to 10.2 ms of wall time,
  and their own durations add up to 2.1 to 2.7 ms,
  so the rest is this host's scheduling.
- by call:
  `openat` 34,
  `read` 39,
  `readlink` 27,
  `statx` 24,
  `getdents64` 10,
  `mkdir` 8,
  `write` 7,
  `symlink` 6,
  `chmod` 4,
  `renameat2` 1,
  and five or fewer each of five more.
- six commits of the same fixture without tracing:
  `buildMs` 8 to 16 ms,
  and the creation's own share of it 2 to 6 ms.

Against that,
the 29 648 ms of the record at 01:28 is the same work of the same shape,
taken there where this fixture takes 8 to 10 ms of wall time.
The two are not the same call for call:
the wrapper those records were written by was installed at 07:23 local on 2026-11-09,
before `918f05385b` took the private index copies out of every shadow at 14:18 local,
and this repository's common directory holds 40 of them.
So the figures do not divide into a rate per call.
What they say is that this work,
and no step of it,
is what the host charges for the wait.

The calls are what Git and a hook in the shadow read:
`mkdir` makes the repository Git is handed,
the `openat` and `read` are that repository's config and the real repository's,
`getdents64` and `symlink` are the shared entries,
and `openat` and `statx` also ask the conclusion-state question.

### The change, in one commit

`feat(git-policy-cli): the timing record names every stretch of the build phase, and a stopped build states its time`,
made on `fix/git-policy-cli-commit-build-phase-unattributed-time`:

- `buildParts` states every stretch of the build that held no Git child,
  by name,
  in the same shape as `prepareParts` and through the same writer,
  with its own ledger and its own bound (`buildPartsDropped`),
  because a build's name must never be read as a preparation's.
  Its names are `branch`,
  `message`,
  `amend tree`,
  `fitting request`,
  `request`,
  `shadow`,
  `shadow skeleton`,
  `shadow config`,
  `shadow links`,
  `shadow conclusion`,
  `shadow refs`,
  `client build`,
  `commit read back` and `amend fields`,
  and the seams sit where the work already changed hands:
  the creation marks its stages in `shadow_create.rs`,
  and the clock is handed to it so no stretch can hold another's work.
- The build phase's clock is closed where the command stops and,
  if no seam closed it,
  where the record is written,
  so a stopped commit states `buildMs` and never leaves the phase to `otherMs`.
- `routeMs` states the one stretch before the first preparation that no phase covered:
  the route to a service,
  this commit's identity,
  the standard input,
  and the client's settings.
  So `otherMs` is what it says it is:
  the removal of a scratch tree and a write that waits for its reader,
  both of which the record already names apart.

Proof by removal,
each control run against this branch's own change undone:

- `commit_timing::the_record_names_every_stretch_of_the_build_phase` failed with
  `the record has no build parts: {...}` once the record no longer wrote the field,
  and passes with it.
- `commit_timing::a_build_stopped_in_its_scratch_commit_still_states_its_time` failed with
  `the stopped build states its time: {...,"buildMs":0,"buildParts":{...},"otherMs":2025,...}`
  while its `git commit` step ran 2 022 ms,
  once both closers were removed.
  That record is the defect of the live records reproduced on purpose:
  `buildMs` 0 and the whole phase in `otherMs`.
- `commit_build_part_durations::tests::*` hold the ledger itself:
  an open stretch is kept when the build leaves,
  a Git child is taken out of the stretch that held it,
  a name marked twice is added together with its `calls`,
  the list stops at its bound and counts the rest,
  and a hostile name stays one JSON member.

### After

The same fixture,
the release built from this branch,
six commits of one service:

- one commit's build of 1 438 ms:
  `shadow skeleton` 1 426 ms,
  every other name 0.
  The stall that the live records hide inside an unnamed stretch is now one name.
- twenty more commits:
  `buildMs` median 9 ms,
  max 16 ms,
  `otherMs` 0 in all twenty,
  and the creation named whenever it cost anything (`shadow skeleton` 7 ms and 5 ms).
- `routeMs` is 0 to 2 ms in those six records and 3 ms in the record of another run,
  which is the size of the stretch that used to be part of `otherMs`.
- After `main` was merged into the branch,
  twelve more commits of the same fixture:
  `buildMs` median 8 ms,
  max 10 ms,
  `otherMs` 0 in all twelve,
  `routeMs` 0 in eleven of them and 3 ms in one,
  and the creation named when it cost anything (`shadow skeleton` 2 ms once).

What this does not claim:
these commits met no stall,
so nothing here measures a faster commit.
What the records now measure is where a slow build waited.

### The choices this leaves to the owner

Nothing in the creation can be removed without changing what a commit is allowed to do:

- The links are one per shared entry of the common directory,
  enumerated rather than named,
  so a repository's creation cost grows with whatever that directory collects.
  Naming a fixed list instead would stop that,
  and would also stop a hook in the shadow from resolving an entry the list does not hold
  (`modules`,
  `lfs`,
  `info`,
  `rr-cache`,
  `branches`,
  a file an earlier release left).
  The private index copies were taken out of the enumeration already,
  on the argument that no hook resolves one.

  Taken on 2026-11-11,
  by the brief on the pooled shadow's refusals:
  the links are a named list now
  (`SHARED_ENTRY_NAMES` in `shadow_links.rs`)
  and a claim of the pool is judged on it,
  so an entry no commit reads costs neither a link nor a refused member.
  The list holds `modules`,
  `lfs`,
  `info`,
  `rr-cache`,
  `branches` and `remotes`,
  so what it stops is a file an earlier release left,
  which no Git reads,
  and a member of the pool stays usable across it
  (`shadow_pool_member.rs`).
- The creation has to exist before the scratch `git commit` runs,
  so it is on the commit's path by construction.
  It could be made beside the preparation,
  or in the service,
  which would overlap it with work the commit already does,
  at the cost of a tree whose owner,
  lifetime and removal all move.
  That is a decision about where a commit's state lives,
  not a performance fix this record can make.

What this does not determine:

- whether the same calls cost the same on the person's own disk:
  the fixture is on this volume,
  and the live records are this volume at other moments.
- what makes the volume stall.
  The record now names the call a commit was in when it did,
  which is what a later reading needs to correlate it with the kernel's own view.

## The creation's calls, what waits, and what each way off it costs (2026-11-09T22:51:40-05:00)

What this answers:
which of the shadow creation's system calls wait when the volume stalls,
how much slower a wrapped commit is than native Git on this volume,
which way of taking that wait off the commit path removes what,
and what each one would have to prove before it could be built.

### What waits, sampled from the host without adding load

A stall cannot be called on:
the wrapper's own rules forbid generating load on the host,
and the stall is the host's.
So the measurement watched the commits that ran and added nothing to them:
four times a second
`/proc/<pid>/syscall` named the call each wrapper process and each process it started was in,
`/proc/<pid>/wchan` named the kernel function it waited in,
and `/proc/<pid>/io` gave a count of finished system calls,
so a call held past 300 ms with an unchanged count is one call waiting,
not a loop of cheap ones.
Two windows were sampled:
2 174 samples and 958 waits from 2026-11-09T22:23:17-05:00 for 240 s,
and 2 705 samples and 2 060 waits from 2026-11-09T22:39:35-05:00 for 300 s,
over 128 and 149 processes at the most.
The sampler is one process held at half a processor:
`${HOME}/temp/agent/pi-shadow-stall-probe/monitor.ts`,
kept out of the repository.
Every wait is counted from the sample that first found the process in that call,
so a wait longer than the window is a lower bound,
and the same event is counted once for each process it held.

The waits of the second window,
by call and by the kernel function that held it,
as the longest recorded wait and how many were recorded:

- `linkat` in `handle_reserve_ticket`:
  112 686 ms,
  9 waits.
- `linkat` in `wait_current_trans`:
  112 268 ms, 7.
- `fsync` in `wait_for_commit`:
  12 680 ms, 68.
- `fsync` in `btrfs_btree_wait_writeback_range`:
  10 111 ms, 6.
- `openat` creating a file,
  in `handle_reserve_ticket`:
  4 922 ms, 11.
- `unlink` in `handle_reserve_ticket`:
  3 304 ms, 4.
- `fsync` in `wait_log_commit`:
  3 190 ms, 71.
- `mkdir` in `wait_current_trans`:
  2 411 ms, 4.
- `unlink` and `link` in `wait_current_trans`:
  2 207 ms,
  12 and 1.
- `openat` in `wait_current_trans`:
  2 204 ms, 19.
- `mkdir` in `btrfs_tree_lock_nested`:
  1 798 ms, 3.
- `mkdir` in `handle_reserve_ticket`:
  1 200 ms, 2.
- `rename` in `btrfs_tree_read_lock_nested`:
  1 180 ms, 2.
- `rename` in `wait_current_trans`:
  1 118 ms, 2.
- `close` in `wait_current_trans`:
  1 104 ms, 3.
- `symlink` in `handle_reserve_ticket`:
  626 ms,
  once.
- `chmod` in `wait_current_trans`:
  314 ms,
  once.

The first window reaches further:
`linkat` in `wait_current_trans` 226 622 ms,
`linkat` in `handle_reserve_ticket` 43 619 ms,
`fsync` in `wait_for_commit` 27 517 ms.

The calls that only read never appeared in any of those functions.
`statx`,
`read`,
`getdents64` and `readlink` are absent from the list above,
and the read waits that were recorded are ordinary block and btree work,
an order of magnitude below the entry waits:
`getdents64` in `read_extent_buffer_pages` at 2 056 ms,
`statx` in `d_alloc_parallel` at 1 955 ms,
`openat` in `btrfs_opendir` at 1 637 ms.

So the answer to the first question is:
the calls that wait are the ones that create or remove a directory entry,
write a file,
or flush one,
and they wait in the volume's own transaction,
reservation and log functions,
`wait_current_trans`,
`handle_reserve_ticket`,
`wait_for_commit`,
`wait_log_commit` and `btrfs_sync_log`.
The calls that read do not wait there.
`chmod` changes a mode without joining a transaction and waited once,
at 314 ms.

### The volume these waits belong to

The figures taken beside the sampling:
`/var/mnt/encrypted` is btrfs mounted with `noatime,compress=zstd:3,ssd`,
3.5 TiB used of 3.64 TiB (95 percent),
its metadata 137.21 GiB used of 150.00 GiB allocated in DUP (91 percent),
and `/proc/pressure/io` `full avg10` between 10.5 and 68.3 across the windows.
`handle_reserve_ticket` is a writer waiting for space the volume can reserve,
and `wait_current_trans` is a writer waiting for the volume to finish committing,
which is why the waits are seconds long here and why they end for every waiter at one instant.
That is also the shared thing the earlier section found in the records,
now named from the kernel's side.

### The arms, paired on this volume

`bin/native-perf.ts`,
`--only commit-signed-service`,
with the fixtures and their commits on the same volume as the repository
(`HOME` on `/var/mnt/encrypted`),
60 rounds,
the arm that goes first flipped every round,
in a shaped fixture of thirteen tracked files whose common directory carries forty of the wrapper's index
copies and a dozen shared entries,
so the creation links as many entries as this repository's fifteen,
the traces below counting 13.7 `symlink` calls per commit.
Every commit in the repository is signed (`commit.gpgsign=true`,
`gpg.format=ssh`),
so every wrapped commit runs native Git in a shadow and creates one.

- native `git commit`:
  median 15.1 ms,
  median absolute deviation 5.6,
  p90 388.2,
  p95 730.6,
  max 2 269.0.
- the wrapper:
  median 1 031.0 ms,
  deviation 963.9,
  p90 9 965.2,
  p95 11 082.6,
  max 16 499.2.
- the paired delta,
  the wrapped sample minus the direct sample of the same round:
  median 1 021.5 ms,
  deviation 969.1,
  p90 9 727.4,
  p95 10 852.9,
  max 16 110.9.
- 17 of the 60 wrapped samples took more than 5 s,
  and none of the 60 direct samples did.
  The benchmark's own verdict is
  `OVER BOUND: commit-signed-service paired commit delta maximum 16 110.9 ms >= 5000.0 ms`.

The same fixture in a window where nothing stalled,
16 rounds without a pause between them and 4 s from the first sample to the last:
native 9.5 ms at the median,
the wrapper 75.7 ms,
the paired delta 66.0 ms with a maximum of 888.3 ms.
So the 60-round spread is not the wrapper's cost,
which is tens of milliseconds,
but the waits the volume charged 17 of the 60 commits.

### Where in a wrapped commit the wait lands

The same 60 wrapped commits' records,
`buildParts` at the stages the build phase names.
Of 22 657 ms of `buildMs`:

- `shadow links` 11 383 ms,
  its longest 9 217 ms in one commit,
  the largest stage in 9 of the 60.
- `shadow skeleton` 1 350 ms,
  longest 362.
- `shadow refs` 458,
  `client build` 339 (longest 205),
  `shadow config` 330,
  `shadow conclusion` 55.

Two records hold their wait outside the creation:
their scratch `git commit` step took 1 000 and 4 545 ms,
and for the second that step is the whole of its build.

The repository's own records over the 48 minutes before these runs,
163 landed commits of release `0.6.1-alpha.5`,
which has no `buildParts` field:

- `buildMs` is 735 323 ms of 1 227 696 ms of the whole command (60 percent),
  median 82 ms,
  p90 12 665 ms,
  p95 29 624 ms,
  max 63 736 ms.
- `prepareMs` median 312 ms,
  p95 8 015 ms,
  max 20 404 ms.
- `submitMs`,
  the landing,
  max 33 748 ms,
  so a stall lands in the landing too.
- `gateMs` max 7 424 ms.

### The creation, counted from a trace of both arms

Six commits per arm traced with `strace --follow-forks --syscall-times --decode-fds=path`,
the calls that can join a transaction selected by name:
`mkdir`,
`symlink`,
`link`,
`unlink`,
`rename`,
a creating `openat`,
`write` and `fsync`.

One wrapped commit makes 196 such calls:
77.7 name the shadow tree,
58.3 name the repository's own Git directory (the private index copies,
the captured index,
the journal),
and 59.7 name the worktree and other paths.
Of those naming the shadow:
13.7 `symlink`,
10.0 `mkdir`,
16.0 `write`,
14.0 `unlink`,
4.0 `chmod`,
2.0 `rename`,
1.0 `renameat2`,
and 86.0 `openat` of which most are reads.

The creation's own share of them,
from `shadow_create.rs` and its steps:
two `mkdir` of the parent that answer `EEXIST`,
the seven `mkdir` of the skeleton and one `write` of its `HEAD`,
one `chmod` of the shadow,
`config` and `objects/info/alternates` written and each `chmod`ed,
the thirteen `symlink` of the shared entries,
and then the ref snapshot:
`git for-each-ref`,
and the `packed-refs`,
the private target ref,
two symbolic refs and `HEAD` written.
That is about 34 calls on the path every commit waits on,
13 of them the links step,
which the records above put at half of `buildMs`.

### The candidates, each with what it removes and what it must prove

#### A shadow made ahead, claimed with one rename

What it removes:
every call of the creation but the claim and the ref snapshot.
A resident service writes a complete shadow into a private staging directory
and publishes it with one `rename`,
which is the only way a tree becomes visible whole;
a commit claims one by renaming it to its own scratch directory,
which fails for the second claimant,
so the claim is a lock-free reservation of one item
(`AGENTS.md`,
`NLK`).
What the commit then makes is its ref snapshot and the `git commit` in the shadow,
about 20 of the 34.

What it must prove:
a claimed member is byte for byte what a fresh creation would have made,
for each entry and mode;
the ref snapshot is written after the claim and from this invocation's refs,
never from the member;
no conclusion state,
index,
object or ref of an earlier commit is in a member;
a member made by another release is refused rather than trusted;
the pool is bounded,
and a member nobody claimed is removed by age rather than kept for ever;
a claim that finds no member falls back to the creation as it is,
so a commit never fails over an empty pool.

What breaks it:
a worktree with no resident service (the fallback covers it,
at today's cost);
a member made for another repository or another worktree;
a release whose shadow differs from the one that filled the pool;
a kill between the publish and the claim,
which leaves a member the sweep removes;
two claims of one member,
which the rename makes impossible but a test must show.

#### One shadow per worktree, reset between commits

What it removes:
the same 34 calls from every commit after the first in a worktree,
leaving the ref snapshot and the shadow's own `git commit`.
A reset is cheaper than a creation only in what it does not repeat:
the directories and the links stay,
and the previous commit's objects and refs go.

What it must prove:
a shadow is used by one commit at a time,
and a second commit in the same worktree takes
a second one,
so agents still never wait on each other;
a shadow left by a process killed at any instruction is never trusted,
which needs a generation written before use and read back after the claim;
the object store cannot grow without a bound,
so what a commit wrote is removed or capped;
the link set is checked against the common directory on every use,
since an entry that appeared
after the shadow was made is one a hook would not resolve;
the conclusion state of the previous commit is gone before the next reads it.

What breaks it:
a half-finished reset,
which is the one failure that can put one commit's staged bytes into another,
and the owner's "never discard user data" rules that out without a proof stronger than the pool needs;
a common directory that gains an entry between the check and the use;
a worktree whose commits overlap.

#### Fewer metadata operations in the creation

What it removes:
nothing measurable.
The two `mkdir` calls of the parent that always answer `EEXIST` could be replaced by a read,
and on this volume a refused `mkdir` costs what the `statx` that would replace it costs,
10 us against 9 us under a trace,
so the change would buy nothing.
Of the mode calls,
`chmod` waited once in about 3 000 samples at 314 ms,
against 112 468 ms for `linkat`,
so there is nothing to win there either.
The only large group is the thirteen links,
and the same decision record requires them:
"every real Git-dir entry that is not transaction-private
(such as `info`,
`rr-cache`,
`lfs`,
and `modules`) is symlinked into the shadow by default".
So this candidate reaches no call worth removing on its own,
and the only way past it is a behaviour change the decision record has to make first.

#### The shadow on another volume

What it removes:
77.7 of the 196 blocking calls a wrapped commit makes,
being everything that names the shadow tree,
and with them the stage the records put at half of `buildMs`.
It leaves the 58.3 that name the repository's Git directory and the 59.7 elsewhere,
and the landing's writes in the service,
which are the ones that reach 33 748 ms in the records.

What it must prove:
a shadow on another filesystem is still a shadow Git will use with `objects/info/alternates`
pointing back at the real object store,
for every backend and for the hooks that resolve it;
the directory it moves to is private,
exists for a command whose environment has no `HOME`,
and is swept of what a killed process leaves;
a cross-device `rename` never happens on the commit's path;
the wrapper's own recorded reason for keeping scratch in the repository
(`doc/planning/cli-git-scratch-locations.md`:
"on the repository's own device,
needs no environment
variable (hooks and sandboxes may lack `HOME`),
and is collected with the repository")
is replaced by a new one,
which is the owner's decision.

What breaks it:
a commit in an environment without `XDG_STATE_HOME` or `HOME`;
a state directory on a filesystem where execution or mode bits differ;
a sweep that collects a live commit's tree because the age it uses no longer means what it meant
when the tree lived in the repository.

### The ranking

1.  **A shadow made ahead,
   claimed with one rename.**
   It takes the single largest stage off the path every commit waits on,
   it leaves a commit working exactly as it does when the pool is empty,
   and its new risk is a trust question with one answer that a test can hold:
   a member is either the tree a creation makes or it is refused.
2.  **The shadow on another volume.**
   It removes more than any other candidate,
   40 percent of the blocking calls,
   and it needs no new trust in anything the wrapper made earlier.
   It ranks below the pool because it changes where a commit's private state lives,
   which the record above argues against,
   and because it still leaves the landing's writes
   on the stalled volume.
3.  **One shadow per worktree,
   reset between commits.**
   It removes the same calls after the first commit,
   but its reset makes the same class of call it saves,
   and its one dangerous failure,
   a half-reset shadow,
   is the one the owner's data rule forbids
   without a proof the other two do not need.
4.  **Fewer metadata operations in the creation.**
   Measured at no call worth removing,
   and the 13 links that would matter are required in writing.
   It cannot carry the tail and should not be built on its own.

### What this record asks the owner

- Whether a commit may take its shadow from a pool the resident service filled ahead,
  which is a new directory inside the wrapper's own scratch tree,
  a renamed claim per commit,
  and a fallback that creates one inline when the pool is empty.
  The measurement behind it:
  `shadow links` holds 11 383 ms of 22 657 ms of `buildMs`,
  its longest wait 9 217 ms,
  and the creation makes about 34 of the 196 blocking calls a commit makes.
- Whether a commit's shadow may live outside the repository,
  on the per-user state directory the wrapper already keeps scratch in,
  which removes 40 percent of a commit's blocking calls and contradicts the recorded reason for
  repository-local scratch.
- Whether `chmod`'s 314 ms rarity is enough to leave the mode calls alone,
  which this record takes as
  yes:
  they are three calls,
  none of them waited in a way worth a change,
  and the parent `mkdir` calls that answer `EEXIST` cost what a read costs.

### A defect this measurement found beside the question

Every shadow links the repository scratch root into itself:
`monochromatic-git-policy-cli-scratch` is not in `PRIVATE_SHADOW_ENTRIES` and does not carry the
private index prefix,
so the creation makes a fourteenth link whose target is the directory the
shadow lives in.
No removal follows it (`remove_dir_all` removes the link,
never its target),
so nothing in the wrapper deletes through it today,
but a hook inside the shadow can reach every other commit's private index copy through it.
Nothing was changed for it here.

Note 2026-11-10T01:03:00-05:00:
this is fixed.
The shadow leaves both names of the record scratch root out now,
through `is_private_shadow_entry` in `package/git-policy/cli/src/native/shadow/shadow_links.rs`,
and `the_repository_scratch_root_never_reaches_the_shadow` beside it fails whenever the rule is removed.
Measured by counting the links one shadow makes for this repository's common directory:
27 before,
25 after,
because that directory held `monochromatic-git-policy-cli-scratch` and the earlier `cli-git-scratch` both.
Nothing reads a scratch root through a shadow:
the wrapper's own `shadow.join` calls name `config`,
`objects` and the entries a shadow writes for itself,
and no hook reads one either.

### What this record does not claim

- That any of these candidates takes a commit under 5 s while the volume stalls.
  The landing writes to the same volume and reached 33 748 ms in the records,
  so a commit can still wait there whatever the creation does.
- That the stalls are the repository's fault.
  The volume is 95 percent full with 91 percent of its metadata allocated,
  and `handle_reserve_ticket` is the volume asking for space it can reserve,
  which no wrapper change makes cheaper.
- That the numbers hold at another hour.
  The two sampled windows differ by a factor of six in I/O pressure,
  and the records differ by
  the same before them.

### Where the evidence of this reading sits

Nothing of the measurement is in the repository yet.
`${HOME}/temp/agent/pi-shadow-stall-probe` holds the sampler (`monitor.ts`),
its two windows (`monitor-20261109T222317.json`,
`monitor-2.json`),
the benchmark's two reports and its traces,
and the script that builds the shaped fixture and the one that counts a trace.
If the owner wants this reproducible from the repository,
the sampler belongs beside `bin/native-perf.ts`,
with a control of its own,
which is a change this record does not make.

## What each part of a commit's private repository is for, and how many commits need it (2026-11-10T05:42:20-05:00)

What this answers:
for each part the creation of a commit's shadow repository makes,
what reads it,
whether a given commit needs it,
and how many real commits needed it;
and,
because the owner chose the pool ahead of the alternatives
("Pool made ahead (Recommended),
We don't need to build a shadow for every commit.
Be smarter.",
`doc/handover/questions-for-the-owner.md`,
"May a commit take its private repository from a pool made ahead?"),
which route the numbers pick:
one that makes no shadow for a commit that needs none,
or one that reuses a finished one.

### Where the numbers come from

The two records the wrapper writes for every wrapped commit,
copied aside before they were read because the machine keeps committing:
`.git/monochromatic-git-policy-cli-commit-timing.jsonl` and its one generation `.1`
of the main checkout,
271 records of the releases `0.6.1-alpha.6` and `0.6.1-alpha.7`
between 2026-11-10T06:33:22Z and 2026-11-10T10:31:01Z,
about twenty agents committing.
The figures of this repository's own shape,
which are what a creation costs here rather than in a fixture:

- 1,968 refs (`git for-each-ref | wc -l`),
  of them 878 branches and one symbolic ref;
- 78 entries in the common Git directory,
  of which 25 are the entries a creation links
  (`is_private_shadow_entry` excludes the other 53);
- 296 worktrees (`git worktree list | wc -l`),
  each with its own commit service.

### Which commits need a shadow at all

A commit builds in a shadow only when Git must build it:
the preparation forces that route for
`config_native || unborn || detached || state_present || changes.is_none()`
(`service_prepare.rs`,
`ready.needs_native = forced`),
and `build_with_git` is the only caller of the creation on the commit path
(`service_native.rs`).
The other route,
a commit the service builds itself,
runs no `git commit` and makes no shadow (`service_commit_request_of.rs`).

Counted:

- 248 of the 271 records reached the build phase,
  and every one of those 248 made a shadow
  (`buildParts["shadow skeleton"].calls == 1`).
  None took the service-built route.
- 23 stopped or ended before the build (`buildMs` 0,
  no `buildParts`).
- The last 300 first-parent commits of `main` all carry a `gpgsig` header
  (`git log --first-parent --format='%H %p' -n 300 HEAD`,
  then `git cat-file --batch`),
  so this repository signs every commit,
  and a signing request is exactly what `config_needs_native` refuses to build without Git.
  Every commit this repository makes therefore needs a shadow:
  a route that makes no shadow for a commit that needs none removes nothing here.
- 242 of those 300 commits are merges,
  and the merge path builds its commit with `git commit-tree` and no shadow at all
  (`merge_isolation_apply.rs`,
  `commit_merge`).
  The "needs no shadow" population of this repository is its merges,
  and that route already exists.

So the route that reuses a finished shadow is the one that removes the most,
and the sections below size what a commit can stop doing:
the parts a commit cannot read at all go first,
the rest come from a pool.

### The parts, one by one

Every part below is made by `create_shadow_repository_carrying`
(`shadow_create.rs`),
for every commit that reaches the build.

The parent and skeleton directories,
the placeholder `HEAD` and the private mode:
ten `mkdir` calls,
one `write` and one `chmod`.
Every commit needs them:
they are the repository Git's `commit` is pointed at
(`native_commit_arguments`,
`--git-dir=<shadow>`).

`config`,
and `objects/info/alternates`:
two `write` calls and two `chmod` calls.
Every commit needs them:
the config carries the format facts,
the include of the real config,
the worktree root and every hook event switched off (`shadow_config.rs`),
and the alternates are how the shadow's Git reads real objects
while its own store receives the commit (`shadow_create.rs`).

The links,
25 directory-entry creations in this repository:
every non-private entry of the common Git directory,
one symbolic link each (`shadow_links.rs`,
`link_shadow_entries`).
`doc/decision/cli-git-concurrent-commits.md` requires them
("every real Git-dir entry that is not transaction-private
(such as `info`,
`rr-cache`,
`lfs`,
and `modules`) is symlinked into the shadow by default").
What a commit can actually read through them is narrower than the set:
every shadow command points `core.hooksPath` at an absent directory
(`shadow_git.rs`,
`shadow_request`;
`native_commit_arguments` for `git commit`),
so no hook resolves anything,
and the Git children a shadow runs
(`commit`,
`rev-parse`,
`cat-file`,
`mktree`,
`hash-object`,
`read-tree`,
`write-tree`,
`update-index`,
`update-ref`,
`symbolic-ref`)
read `info` for attributes and a sparse checkout and `lfs` for the filter's store.
They are kept,
because the decision record requires the set,
and a pool amortizes them over many commits.

The conclusion copies:
one `mkdir` of the transaction's `conclusion` directory,
one `stat` per name in `CONCLUSION_STATE_FILES`,
and a synced write per entry that exists (`conclusion_copy.rs`).
Only a commit that concludes a merge,
a cherry-pick or a revert has anything to copy:
copies exist so the landing removes exactly the entries native Git removed
(`conclusion_cleanup.rs`,
`reproduce_conclusion_cleanup`,
which answers "no conclusion state was copied" when the directory is absent).
The records name no conclusion state,
so how many of the 248 had any cannot be read from them;
the `mkdir` happens for all 248.

The ref snapshot:
one `git for-each-ref` over the real repository,
one `packed-refs` write of every real direct ref (1,968 of them here),
one loose file per real symbolic ref (one here),
the private target ref and `HEAD` (`shadow_refs.rs`,
`write_shadow_refs`).
Every commit makes it,
and `doc/decision/cli-git-concurrent-commits.md` requires the snapshot.
What a commit reads from it is `refs/stash`
(`autostash_carry.rs`,
`conflicted_autostash`) and the symbolic chain a target branch can be
(`end_of_symbolic_chain`).

Reading the commit back,
`git rev-parse HEAD^{commit}` and `git cat-file commit`
(`commit_preparation.rs`,
`read_prepared_commit`):
every commit needs both.

### Where the seconds were, in these 248 records

The whole command:
median 469 ms,
p95 13,611 ms,
the largest 65,948 ms,
and 35 of the 248 over 5,000 ms.

`buildMs`,
which holds the creation and Git's build:
236,572 ms in all,
median 57 ms,
p95 721 ms,
the largest 65,365 ms,
and only six of the 248 over 5,000 ms there.
So most of the commits over the bar wait somewhere else,
and this record speaks to the six.

By the name each stretch has in `buildParts`,
with the records that spent any time in it,
their total,
and the largest:

- `shadow conclusion`:
  44 records,
  196,670 ms,
  64,126 ms.
  This is 83.1 percent of every `buildMs` in the window,
  and five records spent more than 5,000 ms in this stretch alone,
  one of them 64,126 ms of its 65,365 ms build
  (2026-11-10T06:53:52.459Z on `refs/heads/main`,
  whose every other stretch was under 510 ms).
- `shadow skeleton`:
  22 records,
  14,721 ms,
  9,757 ms.
- `shadow links`:
  22 records,
  1,929 ms,
  1,401 ms.
- `shadow refs`:
  all 248,
  1,169 ms,
  427 ms (median 1 ms).
- `shadow config`:
  10 records,
  72 ms,
  30 ms.
- `client build` and `commit read back`:
  35 ms and 7 ms in all.

The only call inside the `shadow conclusion` stretch that creates anything
is `create_private_directory(<scratch>/conclusion)`,
the first call of `copy_conclusion_state`;
everything after it is a `stat` or a read of a file that a commit without conclusion state does not hold.
A `mkdir` is one of the calls this volume holds longest
(`doc/planning/cli-git-commit-latency-under-many-agents.md`,
"What waits,
sampled from the host without adding load":
`mkdir` in `wait_current_trans` at 2,411 ms and in `handle_reserve_ticket` at 1,200 ms),
and reads do not wait there.
So the largest single stretch of these builds is one `mkdir`
that a commit with nothing to conclude does not need.

### What a commit can stop doing

- The conclusion copies directory:
  nothing,
  when the worktree holds no `MERGE_HEAD`,
  `MERGE_AUTOSTASH`,
  `MERGE_MSG`,
  `MERGE_MODE`,
  `SQUASH_MSG`,
  `AUTO_MERGE`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`,
  `MERGE_RR` or `sequencer` state,
  which is what a commit that concludes nothing holds.
  The landing already treats an absent directory as "no state was copied".
  This removes the largest stretch above from those commits.
- Everything else the creation makes is needed by every commit that builds in a shadow,
  or is required by `doc/decision/cli-git-concurrent-commits.md`,
  and cannot be dropped per commit.

That is the reuse route this record's findings pick,
and the parts are built as:
a creation that makes only what the commit can read (the conclusion copies wait for state),
and a pool of finished shadows for the parts every commit needs,
whose member a commit claims with one `rename`
and refuses when it is not what a creation would have made.

## What the pool removes from a commit, counted in calls (2026-11-10T08:01:06-05:00)

What this answers:
what was built for the owner's answer,
what a commit stops doing,
and what the latency harness could and could not resolve about it on this host.

### What was built

- `take_or_create_shadow` (`shadow_create.rs`) takes a finished shadow from the pool when one is waiting,
  and makes the creation it made before when none is.
- `shadow_pool.rs` fills,
  claims and sweeps members,
  and `shadow_pool_member.rs` decides whether a claimed member is what a creation would have made.
- `service_shadow_pool_fill.rs` fills one member beside the landing loop,
  on a thread of its own,
  so no commit waits for it.
- `copy_conclusion_state` makes the copies directory only when the worktree holds conclusion state
  (the section above).
- `is_private_shadow_entry` no longer links the file a command stages a lock in,
  which the pool's own refusal found:
  every command of a repository stages one,
  so no member could be taken twice in the same repository (the commit that fixed it).

### What a commit stops doing, counted

Wall time on this volume cannot say it:
the calls the creation makes are the ones the volume stalls for seconds at a time,
so the same work costs 2 ms or 64 s depending on when it runs.
Their number does not change with the host's pressure,
so that is what was counted.

One disposable repository (twelve shared entries in its common Git directory,
fifteen links a creation makes),
one service the probe starts under its own runtime directory,
three wrapped commits with `commit.cleanup = strip` so every one is built by Git in a shadow,
each traced with
`strace --follow-forks --decode-fds=path --syscall-times -e trace=mkdir,mkdirat,symlink,symlinkat,link,linkat,rename,renameat,renameat2,chmod,fchmodat,write`,
counting the calls whose path is inside the commit's own shadow:

- a commit that made its creation:
  47 calls,
  of them 10 `mkdir`,
  16 `write`,
  3 `chmod`,
  15 `symlink`,
  1 `renameat2` and 2 `rename`.
- a commit that took a member:
  20 calls,
  of them 3 `mkdir`,
  13 `write`,
  1 `chmod`,
  1 `renameat2` and 2 `rename`.

The 20 are the work both routes do:
`git commit` inside the shadow writes the index,
the message file and its objects,
and the ref store is written after a claim as after a creation.
The 27 calls the claim removes are the creation's own:
7 `mkdir`,
3 `write`,
2 `chmod` and 15 `symlink`.
This repository's own common directory holds 25 shared entries rather than the fixture's 15,
so a commit here stops making about 37 calls
(the count of links in `.git/monochromatic-git-policy-cli-commit-timing.jsonl`'s window:
`is_private_shadow_entry` leaves 25 of its 78 entries,
plus the ten `mkdir`,
four `write` and two `chmod` of the skeleton and the config).

### What the harness's arms could and could not resolve

The method is the one the sections above used:
`mise run //package/git-policy/cli:native:perf` with
`--only commit-signed-service --samples 8`,
its fixtures copied from a prepared repository shaped like this repository's own
(thirteen tracked files,
a common Git directory holding the forty private index copies and a dozen shared entries),
one binary per arm built from the same base
(`b73a61f9f0` for the base,
this branch for the change),
and the arms alternated:
base,
this change,
base.
The documents are
`perf/commit-pool-2026-11-10-before-a.json`,
`...-after-a.json` and `...-before-b.json` beside this record.

The whole wrapped command,
the median of its samples,
its p95,
the paired deltas against native Git of the same round,
and the samples that failed,
in whole milliseconds:

- base,
  run A:
  median 62,709,
  p95 81,215,
  deltas 72,407,
  31,124,
  50,943,
  80,708,
  63,970,
  74,470,
  48,036,
  61,085,
  no failed sample.
- this change:
  median 93,536,
  p95 179,051,
  deltas 93,202,
  45,086,
  73,210,
  90,414,
  175,694,
  three failed samples.
- base,
  run B:
  median 47,431,
  p95 180,880,
  deltas 41,193,
  180,592,
  66,456,
  34,079,
  34,507,
  37,100,
  two failed samples.

Two runs of one binary differ by 1.3 times at the median and 2.2 times at p95,
and native Git's own commits reached a median of 7,187 ms in the change's run against 97 ms in the base's first run:
the host's commits were waiting tens of seconds whatever the wrapper did,
which is the volume stall this record names elsewhere,
not the creation.
So the arms overlap completely,
and the whole-command times of this host cannot resolve a change of 27 calls
under this load,
as the record's own 18.6-times spread warned.

What the runs do say,
because it does not depend on the host:

- Every commit in the change's run that reached the build named `shadow pool`,
  and the ones that took a member named no `shadow skeleton`,
  `shadow config` or `shadow links` stretch
  (`buildParts`,
  `README.md`,
  `SPEC.md` "A shadow taken from the pool").
  A reading of the next records of this repository says how many commits took the pool there;
  the records of the main checkout cannot yet,
  because the wrapper installed there is the release before this change.

### The probe's own records, and what they named

The three commits of the count above left records in their repository:

- the creation:
  `buildMs` 43,
  with every creation stretch named:
  `shadow skeleton`,
  `shadow config`,
  `shadow links`,
  `shadow conclusion` and `shadow refs`.
- the two claims:
  `buildMs` 91 and 33,
  with `shadow pool`,
  `shadow conclusion` and `shadow refs`,
  and none of the three creation stretches.

The first claim of a fresh repository is refused once,
which the probe shows as a warning naming the entry:
a member made before the commit's own timing record appeared in the common Git directory lacks that entry,
and a creation would have linked it.
The next fill holds it,
so the refusal costs one member and one warning,
never a commit,
and the commit makes its creation as it did before.

### What this leaves

- The fill reads a repository's facts a second way,
  from a landing request rather than from an invocation's capture;
  `the_facts_a_fill_reads_are_the_facts_a_capture_reads` holds the two equal field by field,
  and a fact it gets wrong can only cost a refused member,
  because the claim compares the member with what this invocation's creation would write.
- A common directory whose entry set changes
  (the first timing record,
  a rotation that creates `.1`,
  a tool's own directory)
  refuses the member that was made before the change,
  once per change.
  Measured:
  one refusal in the probe's first commit of a fresh repository.
- The pool is per worktree,
  and a reftable repository and a linked worktree that keeps its own `info` are never pooled.
- The fill costs one creation and two Git starts per landing the service takes,
  beside the landing and never inside it,
  which no commit waits for.

## Where the pool lives, and the leak check that found it (2026-11-10T15:57:02-05:00)

What this answers:
where the pool of finished shadows sits now,
why it moved out of the repository scratch directory,
and what the two controls that failed on `main` prove about it.

### The two controls that failed

The pool was built inside the repository scratch directory,
at `<common-dir>/monochromatic-git-policy-cli-scratch/shadow-pool/<worktree>`,
and two controls of the release gate on `main` at `483bbc6587` said what that costs.
Both are the check that catches leaked scratch:

- `markdown::a_commit_keeps_its_git_input_in_the_repository_scratch_directory`
  (`src/native/binary/binary_markdown_tests.rs`) failed with
  `nothing is left in .../repo/.git/monochromatic-git-policy-cli-scratch after 30s: ["shadow-pool"]`;
- `service_scratch::a_commit_leaves_no_scratch_directory_once_its_landing_is_over`
  (`src/native/binary/binary_service_scratch_tests.rs`) failed with
  `the repository scratch directory holds nothing`.

Neither control is wrong.
A member of the pool is standing state the owner asked for
("Pool made ahead (Recommended),
We don't need to build a shadow for every commit.
Be smarter.",
`doc/handover/questions-for-the-owner.md`,
"The private repository every commit builds"),
and the owner did not say where it lives.
A directory whose emptiness is the wrapper's own proof that nothing leaked
cannot also hold the one thing the wrapper keeps on purpose.

### What moved

The pool is now its own folder inside the Git common directory and beside the scratch directory:
`<common-dir>/monochromatic-git-policy-cli-shadow-pool/<worktree>`
(`SHADOW_POOL_FOLDER` in `wrapper_names_old_and_new.rs`,
`pool_root_for` in `shadow_pool.rs`).
The name is a fresh one this release is the first to write,
so it has no earlier spelling.

Every pass that removes scratch was read for what it would do with it,
and each one leaves a member alone by construction rather than by a list:

- the sweep a command that takes scratch runs
  (`scratch_location.rs`,
  `prepare_repository_scratch`)
  reaches entries of the scratch directory only,
  and of the earlier directory an earlier release made
  (`sweep_earlier_repository_scratch`);
- the release of a commit's scratch tree after its landing
  (`service_scratch_release_removal.rs`)
  refuses a name that is not under the scratch directory or the worktree's Git directory,
  and that does not carry the wrapper's commit-tree prefix
  (`is_scratch_path`);
- the age decision both of them share
  (`service_scratch_abandonment.rs`,
  `may_go`)
  is asked about an entry of a scratch place and about nothing else;
- `doctor` and the reclamation of a removed worktree's state read no scratch entry at all
  (searched:
  no read of the scratch directory or of the pool in `invocation/management*.rs`,
  `health/`,
  or `service_units_of_removed_worktrees.rs`).

One thing moving the pool out of scratch did need:
the folder is a private shadow entry now,
like the scratch root beside it
(`shadow_links.rs`,
`is_private_shadow_entry`).
Without that,
a creation would link the pool into every shadow,
so one shadow could reach every other member,
and whether the folder existed would decide what a claimed member has to hold:
the first fill of a repository would refuse the member it had just made.

An in-place note,
2026-11-11:
`is_private_shadow_entry` no longer exists.
A shadow links the named entries a commit reads and nothing else
(`SHARED_ENTRY_NAMES` in `shadow_links.rs`),
so the pool still never reaches a shadow,
and the occasion for it is the named list rather than a denylist.

### What says it works

- The two controls above pass,
  and `service_scratch::a_landed_commit_leaves_the_pool_of_finished_shadows_alone`
  is the same check from the other side:
  after a landed commit the scratch directory holds nothing,
  and a mark left in the pool's folder still holds its bytes,
  so the release and the sweep leave standing state alone.
- `shadow::shadow_pool::tests::the_pool_is_its_own_folder_beside_the_scratch_directory`
  pins the place,
  and `a_pooled_shadow_survives_a_sweep_of_the_scratch_directory`
  shows a stale tree of the scratch directory removed while a member older still stays.
- `shadow::shadow_links::tests::the_pool_of_finished_shadows_never_reaches_the_shadow`
  pins the link rule.

## What a commit waited for under fifty agents, and the two waits that were silence (2026-11-10T19:20:00Z)

What this answers:
where the time of a wrapped commit went on the installed `0.6.1-alpha.8` with about fifty agents working,
why the commit of 2026-11-10T17:50:19Z ended with exit 1 although the service landed it,
and what of that wait is left after the pool of private repositories made ahead (`373e8adfe2`).

### The records, split by the steps they name

The two commit timing records of the main checkout (`.git/monochromatic-git-policy-cli-commit-timing.jsonl`
and its `.1`,
read only),
272 records of the releases `0.6.1-alpha.7` and `0.6.1-alpha.8`,
2,436,718 ms in all,
in whole milliseconds:

- `build` 1,250,649,
  `prepare` 527,191,
  `submit` 502,195,
  `handoff` 42,377,
  `provision` 38,627,
  `gate` 33,452,
  `other` 41,105.
- within `submit`:
  425,180 waiting for the answer and 47,603 waiting for a service to start.
- of the 45 records over 10 s (1,795,649 ms in all),
  `build` 1,137,857 and `submit` 326,044.
- `buildParts` of those 45:
  `shadow conclusion` 959,581 (the `mkdir` of the conclusion copies directory,
  which the pool's release no longer makes when a commit concludes nothing),
  `shadow skeleton` 76,064,
  `shadow refs` 10,561,
  `shadow config` 3,022,
  `shadow links` 723,
  and everything else under 70.
No step name is to blame for the rest:
`git commit` 2,648 ms and `git update-index` 225 ms in the record of 17:50:19Z are the two largest steps of it.

### The commit that ended exit 1

Its record:
`sends=6`,
`connectTries=6`,
`answerWaitMs=31,084`,
`positions` empty,
`waitingNotices` 0,
`landing` null,
`outcome "stopped"`,
`exitCode 1`.
Six sends of about 5.18 s each is six expiries of `HEARTBEAT_SILENCE` (5 s) and of `RECONNECTS` (5 reconnects),
which ends `submit_loop` with `ClientFailure::ServiceSilent`.

The service's own journal for that request (`requestId c2b378d0-1772-44f9-ab44-ba570a27db09`,
instance `f30a114aaabc977cf77309db0f77f1ee`) says where the silence was:

- the batch answered `outcome=landed ... read_ms=32019 ... lock_hold_ms=3118 index_update_ms=3101 landing_ms=3194`.
  `read_ms` is the time from a connection being accepted to its request reaching the queue:
  thirty-two seconds in which the connection wrote nothing at all.
- the six sends were answered `outcome=already` with `queued_ms=3196`,
  so the commit was on the branch and every later send was told so,
  after the client had given up.

Between those two instants the connection task awaits `is_foreign`
(`service_server_connection_handling.rs`),
which resolves the Git directory's real path on the blocking pool,
and no beat is written until `queue_a_commit` starts after it.
So a live service that is waiting for storage looked exactly like a dead one,
which is the one thing the client's silence bound must not be.

### The two changes

- `da2ebbd589`:
  the beats begin with the connection,
  and the check's answer goes through the same single writer,
  so a request for another worktree still never lands here.
- `1b995c2aaa`:
  a request that names this worktree's own Git directory exactly is this instance's without the
  resolution at all,
  because the same text names the same directory;
  only a request spelling it another way waits for the filesystem.

Measured with `native:test:host`,
one control each:

- `a_commit_whose_directory_check_waits_is_beaten_while_it_waits`:
  before,
  FAILED with "the beat comes while the check waits:
  Elapsed(())" after six beat intervals;
  after,
  PASS in 1.02 s.
- `where_no_thread_can_start_a_commit_naming_this_worktree_still_reaches_the_landing`:
  before,
  FAILED with "the instance of a Git directory could not be looked at:
  the system refused a thread for a
  blocking step";
  after,
  PASS in 0.075 s.

### Many commits at once, against native Git

Method:
one disposable repository of 2,000 tracked files,
24 linked worktrees of it on 24 branches,
one service per worktree started at a socket of its own under a private runtime directory,
and 24 commits started in the same tick,
twice per arm,
native Git first and then the wrapper.
The host was not idle:
its load average was 162,
142 and 141 over the run,
with about fifty agents working.
The harness is scaffolding in `${HOME}/temp/agent/pi-commit-wait` and is not in the tree.

Wall time of one commit,
in whole milliseconds:

- native Git:
  48 commits,
  minimum 14,
  median 27,
  p95 235,
  largest 237.
- the wrapper:
  48 commits,
  minimum 16,
  median 1,197,
  p95 8,161,
  largest 8,243.
- native Git's own two rounds:
  median 24 and 24,
  p95 40 and 235,
  which is the spread of the unchanged control.
- the wrapper's two rounds:
  median 869 and 8,086,
  p95 909 and 8,167.

The phases of the 46 wrapped records of that run:

- the 23 commits of the first round:
  `build` 99 ms in all,
  `prepare` 1,130,
  `submit` 17,654 (answer wait 17,651).
- the 23 of the second round:
  `build` 115 ms in all,
  `prepare` 1,386,
  `submit` 184,092 (answer wait 184,089).

So the pool did what it was made for:
the creation of a private repository is gone from `build`,
which costs four to five milliseconds a commit here,
and every second over the bar is now the wait for the service's answer.

### What that answer wait is, traced

The landing's own figures of those slow records:
`lockHoldMs` 26 to 47,
`readsMs` 6 to 16,
`queuedBehind` 0,
so neither the landing nor a queue of other commits was the eight seconds.
The service's `answered` line for three commits in a row in one worktree names it:

- 3,066 ms in `lock_hold_ms`,
  `lock_stage_kind=cold`;
- `lock_wait_ms=9369 lock_stage_ms=9333 lock_stage_kind=pre`,
  and no `lock_looks` field,
  which means the index lock was free at the first placement;
- `lock_wait_ms=12867 lock_stage_ms=12825 lock_stage_kind=pre`,
  the same.

`lock_stage_ms` with `kind=pre` is the whole of `take` in `service_index_prestage.rs`,
which waits for the marker made ahead of the next landing to finish being flushed
("A marker still being flushed is waited for.").
That flush is a synced write on this volume,
and it is seconds long under this load:
the commit waits for the durability of the lock's own marker before it can even place the lock.

That wait is kept,
and its cost is now named:
removing it means placing a marker that a kill can lose,
which is the record a stale lock is judged by.
What can still be made cheaper is when it starts:
today it is started after a landing and waited for by the next commit,
never finished before it.

### What is still missing from the record

The stop reason.
A record says `outcome` and `exitCode`,
and nothing names which `ClientFailure` ended it nor its words,
so the six-send silence above had to be read from the counts and from the service's journal rather than
stated by the record.

## The marker made ahead: the flush stays and no landing waits for it (2026-11-10T21:45:00Z)

What this answers:
what a landing spent when it waited for the index lock's marker to finish its flush,
and how the flush is kept without that wait.

### The pick

The owner's rules are "never discard user data" and "commits vs native git should not take 5s longer",
and the measured wait was 9 to 13 seconds with `lock_stage_kind=pre`,
which is the whole of `take` in `service_index_prestage.rs`.
The coordinator's pick,
recorded in `doc/handover/questions-for-the-owner.md` with the owner's veto open,
is neither side of the trade:
the flush stays,
it starts as early as the code allows,
and a landing that finds it still running takes the lock the ordinary way and never waits for it.

### What changed

- The marker of the batch after this one is started as soon as the landing has its lock
  (`service_land_take_index_lock.rs`),
  so the flush spends the landing's own seconds instead of beginning after the answer.
- `take` and `start` never wait (`service_index_prestage.rs`):
  a marker still being flushed stays in the service's slot and the landing stages its own,
  and the marker that was flushing is removed once it is finished,
  by the batch that follows or by the service's end.
  `discard`,
  which runs between landings and at the service's end,
  still waits for it,
  so no file is left behind and one file at most is ever being made.

### Before and after, on one worktree and three commits in a row

The `answered` line of the service,
in whole milliseconds:

- before:
  `lock_wait_ms=4 lock_stage_ms=0 lock_stage_kind=cold`,
  then `lock_wait_ms=9369 lock_stage_ms=9333 lock_stage_kind=pre`,
  then `lock_wait_ms=12867 lock_stage_ms=12825 lock_stage_kind=pre`.
- after:
  `lock_wait_ms=0 lock_stage_ms=0 lock_stage_kind=cold`,
  three times,
  with `lock_hold_ms` 32,
  91 and 14 and `landing_ms` 44,
  101 and 27.

### Many commits at once, after

The method is the one the section above used:
2,000 tracked files,
24 linked worktrees,
one service per worktree,
24 commits started in the same tick,
two rounds per arm.
This run's host was quieter than that one's (load average 33,
31 and 39 against 162,
142 and 141),
so the two runs' medians are not comparable and only the lock figures are.

- native Git:
  48 commits,
  median 35,
  p95 55,
  largest 59.
- the wrapper:
  48 commits,
  median 372,
  p95 834,
  largest 842.
- every one of the 46 landings named `lock_stage_kind=cold`,
  with `lock_stage_ms` 0 for 42 of them,
  1 for three and 7 for one,
  and `lock_wait_ms` never above 13.
  The flush did not finish between one round and the next on this disk,
  so no landing took a marker made ahead and none waited for one:
  a flush here commits the whole filesystem's dirty data,
  which is the cost the design exists to keep off a commit's path.

### What this tree keeps of both

Each section here measured its own tree.
The section above stood the landing aside from a flush still running and started one marker,
and the section below made two markers and handed their flush to the background task.
The tree merged from the two keeps both:
two markers are started when a landing takes the lock,
`take` never waits for one that is still being made,
and the flush of a marker is the background task's,
so a commit either places a finished marker or stages its own.

## What the installed release spent over its Git steps, and the two lock markers (2026-11-10T18:45:00-05:00)

What this answers:
which commits of the installed `0.6.1-alpha.8` took more than five seconds beyond their own Git steps,
which recorded step or service figure took that time,
which causes `main` already removes and which it keeps,
and what the largest wait `main` keeps was worth before and after starting it earlier.

### The window, the bar, and what the record can say

The two commit timing records of the main checkout
(`.git/monochromatic-git-policy-cli-commit-timing.jsonl` and its `.1`,
read only)
hold 382 records of the release `0.6.1-alpha.8`,
2,350,865 ms in all,
from `2026-11-10T17:29:09.065Z` to `2026-11-10T22:29:17.766Z`.
Their Git steps took 463,635 ms of that.

The bar is the owner's:
"commits vs native git should not take 5s longer."
A record states one time per phase and one per Git child
(`SPEC.md`,
"A slow commit is split by phase from logs that exist without verbose mode"),
so a commit is over that bar in the one reading the record supports
when its `totalMs` exceeds the sum of its own `steps` by more than 5,000 ms.
That figure is the time the wrapper added around its Git children,
whether it waited or worked.
It is not a comparison with native Git in the same second,
which no record holds;
the arms further down are that comparison.

86 of the 382 are over it.
Their whole time is 1,556,885 ms and their Git steps 162,538 ms,
so 1,394,347 ms of it was the wrapper's own.

Where a record holds only a total for its answer wait,
the service's `answered` line for the same `requestId` names the part of it,
and that line is matched by identity rather than by the second:
both records carry the request's identity.
One of the 86 (`e86d4d58`,
outcome `stopped`,
no send) has no landing and no journal line,
so its own `answerWaitMs` stands for its answer wait.

### The commits over the bar, grouped by the one step that took the most of each

Each line names the commit by the time its record was written and its `requestId` (first eight digits),
the time beyond its own Git steps and its whole time,
the one recorded step or service figure that took the most of that time,
and the next largest when a second cause is close.
Each figure after a cause is that cause's milliseconds.
`preparation` names a `prepareParts` stretch,
`build` a `buildParts` stretch,
and the rest are the service's own figures from its `answered` journal line for the same request,
which answers at the same second the record was written.

#### `shadow conclusion`: 20 commits, 547542 ms in it

- 2026-11-10T19:25:21.335Z `383fcf50`:
  83507 ms over its Git steps of 86623 ms,
  the time of `shadow conclusion` 70564 ms,
  then the hand-off of the push.
- 2026-11-10T17:50:19.213Z `c2b378d0`:
  80246 ms over its Git steps of 83451 ms,
  the time of `shadow conclusion` 47953 ms,
  then the connection reading the request.
- 2026-11-10T18:11:31.508Z `812246c4`:
  66471 ms over its Git steps of 72463 ms,
  the time of `shadow conclusion` 17513 ms,
  then the hand-off of the push.
- 2026-11-10T18:09:03.861Z `ad60e7e7`:
  65981 ms over its Git steps of 69467 ms,
  the time of `shadow conclusion` 54411 ms,
  then `index` in the preparation.
- 2026-11-10T21:01:23.753Z `cdc53264`:
  54466 ms over its Git steps of 55489 ms,
  the time of `shadow conclusion` 50888 ms,
  then the connection reading the request.
- 2026-11-10T18:06:25.844Z `810de44d`:
  52556 ms over its Git steps of 53121 ms,
  the time of `shadow conclusion` 39360 ms,
  then the hand-off of the push.
- 2026-11-10T17:38:51.720Z `dda2498e`:
  46550 ms over its Git steps of 49714 ms,
  the time of `shadow conclusion` 37816 ms,
  then the queue.
- 2026-11-10T17:35:50.633Z `685515f4`:
  44305 ms over its Git steps of 49848 ms,
  the time of `shadow conclusion` 37883 ms,
  then the wait for what an earlier batch owes.
- 2026-11-10T18:02:25.717Z `e99a9c8a`:
  42867 ms over its Git steps of 43107 ms,
  the time of `shadow conclusion` 32988 ms,
  then the hand-off of the push.
- 2026-11-10T21:06:47.943Z `84bc92a6`:
  35290 ms over its Git steps of 36651 ms,
  the time of `shadow conclusion` 30012 ms,
  then `scratch` in the preparation.
- 2026-11-10T18:06:01.269Z `e4109b88`:
  30843 ms over its Git steps of 31293 ms,
  the time of `shadow conclusion` 15520 ms,
  then the hand-off of the push.
- 2026-11-10T19:17:10.311Z `3cfe9606`:
  29019 ms over its Git steps of 29222 ms,
  the time of `shadow conclusion` 25365 ms,
  then `shadow refs` in the build.
- 2026-11-10T18:32:02.802Z `a03e858d`:
  25760 ms over its Git steps of 29672 ms,
  the time of `shadow conclusion` 24707 ms,
  then `index` in the preparation.
- 2026-11-10T22:29:17.766Z `b11661bc`:
  16036 ms over its Git steps of 16638 ms,
  the time of `shadow conclusion` 12173 ms,
  then writing the answer.
- 2026-11-10T20:33:13.520Z `f8f1490e`:
  14909 ms over its Git steps of 17391 ms,
  the time of `shadow conclusion` 11929 ms,
  then the post-commit gate.
- 2026-11-10T21:22:21.263Z `e3d2690c`:
  14509 ms over its Git steps of 14672 ms,
  the time of `shadow conclusion` 13296 ms,
  then `policies` in the preparation.
- 2026-11-10T18:01:32.620Z `d935dac7`:
  13006 ms over its Git steps of 13190 ms,
  the time of `shadow conclusion` 11916 ms,
  then the index install under the index lock.
- 2026-11-10T21:45:41.258Z `2955ff64`:
  9830 ms over its Git steps of 10014 ms,
  the time of `shadow conclusion` 5799 ms,
  then the hand-off of the push.
- 2026-11-10T17:50:46.708Z `e86d4d58`:
  6033 ms over its Git steps of 7460 ms,
  the time of `shadow conclusion` 4635 ms,
  then `shadow refs` in the build.
- 2026-11-10T19:41:55.217Z `5794f1dd`:
  5358 ms over its Git steps of 8141 ms,
  the time of `shadow conclusion` 2814 ms,
  then time no phase covers.

#### `scratch` in the preparation: 10 commits, 52344 ms in it

- 2026-11-10T20:07:16.049Z `f707e4e4`:
  16368 ms over its Git steps of 17275 ms,
  the time of `scratch` in the preparation 10434 ms,
  then the index install under the index lock.
- 2026-11-10T19:43:30.468Z `4db4193d`:
  10771 ms over its Git steps of 12916 ms,
  the time of `scratch` in the preparation 6663 ms,
  then `index` in the preparation.
- 2026-11-10T20:16:37.996Z `504b2e2e`:
  10747 ms over its Git steps of 10972 ms,
  the time of `scratch` in the preparation 5096 ms,
  then the wait to place the lock.
- 2026-11-10T19:10:02.133Z `26d8f380`:
  9741 ms over its Git steps of 12531 ms,
  the time of `scratch` in the preparation 5090 ms,
  then `index` in the preparation.
- 2026-11-10T20:08:47.826Z `f8624b05`:
  7912 ms over its Git steps of 14311 ms,
  the time of `scratch` in the preparation 4599 ms,
  then `shadow skeleton` in the build.
- 2026-11-10T19:52:46.086Z `b9c8bddf`:
  6729 ms over its Git steps of 7047 ms,
  the time of `scratch` in the preparation 6326 ms,
  then the index install under the index lock.
- 2026-11-10T17:58:14.491Z `839bfe71`:
  5755 ms over its Git steps of 6034 ms,
  the time of `scratch` in the preparation 5346 ms,
  then the index install under the index lock.
- 2026-11-10T20:10:35.365Z `7205bcd4`:
  5721 ms over its Git steps of 6531 ms,
  the time of `scratch` in the preparation 2472 ms,
  then the wait for what an earlier batch owes.
- 2026-11-10T17:52:28.267Z `22ebb3f7`:
  5524 ms over its Git steps of 5741 ms,
  the time of `scratch` in the preparation 2547 ms,
  then time no phase covers.
- 2026-11-10T19:52:46.455Z `287c0dd8`:
  5008 ms over its Git steps of 5299 ms,
  the time of `scratch` in the preparation 3771 ms,
  then `index` in the preparation.

#### the wait for the lock marker to be flushed: 10 commits, 51188 ms in it

- 2026-11-10T20:55:08.392Z `06afcc81`:
  13552 ms over its Git steps of 13705 ms,
  the time of the wait for the lock marker to be flushed 12338 ms,
  then the wait to place the lock.
- 2026-11-10T17:58:59.534Z `681701c2`:
  11991 ms over its Git steps of 12151 ms,
  the time of the wait for the lock marker to be flushed 9246 ms,
  then the wait to place the lock.
- 2026-11-10T20:44:29.852Z `a040f016`:
  9071 ms over its Git steps of 10009 ms,
  the time of the wait for the lock marker to be flushed 3109 ms,
  then `shadow skeleton` in the build.
- 2026-11-10T18:26:39.698Z `4ed3ec00`:
  7717 ms over its Git steps of 8004 ms,
  the time of the wait for the lock marker to be flushed 4329 ms,
  then the wait to place the lock.
- 2026-11-10T18:58:55.424Z `5331dfaf`:
  7438 ms over its Git steps of 7678 ms,
  the time of the wait for the lock marker to be flushed 5600 ms,
  then `index` in the preparation.
- 2026-11-10T18:15:17.440Z `db37994f`:
  6930 ms over its Git steps of 7145 ms,
  the time of the wait for the lock marker to be flushed 3823 ms,
  then the branch move under the index lock.
- 2026-11-10T18:13:16.362Z `5c193238`:
  6495 ms over its Git steps of 6905 ms,
  the time of the wait for the lock marker to be flushed 4515 ms,
  then the wait to place the lock.
- 2026-11-10T18:06:08.705Z `d785da4e`:
  6270 ms over its Git steps of 6478 ms,
  the time of the wait for the lock marker to be flushed 3479 ms,
  then the wait to place the lock.
- 2026-11-10T18:22:35.399Z `a95e9799`:
  5588 ms over its Git steps of 5828 ms,
  the time of the wait for the lock marker to be flushed 2110 ms,
  then the wait to place the lock.
- 2026-11-10T17:52:20.878Z `eb9705ad`:
  5058 ms over its Git steps of 5306 ms,
  the time of the wait for the lock marker to be flushed 2639 ms,
  then the index install under the index lock.

#### the index install under the index lock: 7 commits, 50099 ms in it

- 2026-11-10T20:01:15.205Z `deea5210`:
  21936 ms over its Git steps of 23081 ms,
  the time of the index install under the index lock 11935 ms,
  then the landing's other reads.
- 2026-11-10T18:10:58.799Z `ce01f8ab`:
  16073 ms over its Git steps of 22595 ms,
  the time of the index install under the index lock 6207 ms,
  then the wait for what an earlier batch owes.
- 2026-11-10T18:31:25.302Z `ad71026a`:
  12424 ms over its Git steps of 12798 ms,
  the time of the index install under the index lock 4826 ms,
  then the branch move under the index lock.
- 2026-11-10T20:04:13.592Z `02fb7b5c`:
  11652 ms over its Git steps of 12409 ms,
  the time of the index install under the index lock 9275 ms,
  then `index` in the preparation.
- 2026-11-10T19:45:39.648Z `c87115d7`:
  9813 ms over its Git steps of 12699 ms,
  the time of the index install under the index lock 6676 ms,
  then `index` in the preparation.
- 2026-11-10T18:09:48.690Z `0473add4`:
  9543 ms over its Git steps of 15339 ms,
  the time of the index install under the index lock 5112 ms,
  then the branch move under the index lock.
- 2026-11-10T18:30:48.616Z `9ae253bd`:
  7922 ms over its Git steps of 11184 ms,
  the time of the index install under the index lock 6068 ms,
  then `changedPaths` in the preparation.

#### time no phase covers: 9 commits, 41362 ms in it

- 2026-11-10T19:44:46.421Z `dd4500fd`:
  12599 ms over its Git steps of 13213 ms,
  the time of time no phase covers 6135 ms,
  then `scratch` in the preparation.
- 2026-11-10T20:26:07.287Z `27e9b872`:
  12004 ms over its Git steps of 12415 ms,
  the time of time no phase covers 6578 ms,
  then the wait for the lock marker to be flushed.
- 2026-11-10T19:44:38.658Z `3cf34123`:
  7448 ms over its Git steps of 8408 ms,
  the time of time no phase covers 5821 ms,
  then `index` in the preparation.
- 2026-11-10T18:15:29.605Z `4f86553d`:
  7040 ms over its Git steps of 7221 ms,
  the time of time no phase covers 3399 ms,
  then the wait for the lock marker to be flushed.
- 2026-11-10T21:21:34.609Z `333d2ec1`:
  6846 ms over its Git steps of 7253 ms,
  the time of time no phase covers 6463 ms,
  then the index install under the index lock.
- 2026-11-10T20:21:05.489Z `1deab5d4`:
  6735 ms over its Git steps of 8851 ms,
  the time of time no phase covers 5670 ms,
  then the index install under the index lock.
- 2026-11-10T18:09:01.237Z `152219c9`:
  5764 ms over its Git steps of 6742 ms,
  the time of time no phase covers 2214 ms,
  then `scratch` in the preparation.
- 2026-11-10T18:09:22.313Z `b79a1142`:
  5344 ms over its Git steps of 10598 ms,
  the time of time no phase covers 2089 ms,
  then `scratch` in the preparation.
- 2026-11-10T18:09:09.120Z `a3cfb7ff`:
  5172 ms over its Git steps of 6186 ms,
  the time of time no phase covers 2993 ms,
  then `scratch` in the preparation.

#### the wait for what an earlier batch owes: 6 commits, 36815 ms in it

- 2026-11-10T20:00:50.351Z `ed491398`:
  15765 ms over its Git steps of 16400 ms,
  the time of the wait for what an earlier batch owes 13631 ms,
  then time no phase covers.
- 2026-11-10T18:48:15.208Z `5287d111`:
  12122 ms over its Git steps of 16365 ms,
  the time of the wait for what an earlier batch owes 6266 ms,
  then `index` in the preparation.
- 2026-11-10T18:25:29.861Z `b8d4b6ee`:
  10736 ms over its Git steps of 11131 ms,
  the time of the wait for what an earlier batch owes 4656 ms,
  then `shadow skeleton` in the build.
- 2026-11-10T18:32:24.318Z `950ff124`:
  7376 ms over its Git steps of 7860 ms,
  the time of the wait for what an earlier batch owes 4673 ms,
  then `scratch` in the preparation.
- 2026-11-10T18:12:03.752Z `223263ab`:
  5535 ms over its Git steps of 6952 ms,
  the time of the wait for what an earlier batch owes 3394 ms,
  then `scratch` in the preparation.
- 2026-11-10T20:28:53.795Z `c60f2672`:
  5277 ms over its Git steps of 6199 ms,
  the time of the wait for what an earlier batch owes 4195 ms,
  then `index` in the preparation.

#### `index` in the preparation: 5 commits, 26488 ms in it

- 2026-11-10T21:57:12.820Z `836fdd96`:
  8472 ms over its Git steps of 8887 ms,
  the time of `index` in the preparation 6913 ms,
  then `scratch` in the preparation.
- 2026-11-10T20:14:24.670Z `707b10e7`:
  7830 ms over its Git steps of 8079 ms,
  the time of `index` in the preparation 7279 ms,
  then `scratch` in the preparation.
- 2026-11-10T18:33:40.151Z `33169ce5`:
  6878 ms over its Git steps of 9538 ms,
  the time of `index` in the preparation 3282 ms,
  then `shadow links` in the build.
- 2026-11-10T19:00:46.327Z `dac4d29c`:
  6452 ms over its Git steps of 6711 ms,
  the time of `index` in the preparation 3726 ms,
  then `shadow refs` in the build.
- 2026-11-10T20:14:09.337Z `acd4259d`:
  6237 ms over its Git steps of 13688 ms,
  the time of `index` in the preparation 5288 ms,
  then `scratch` in the preparation.

#### writing the answer: 3 commits, 25348 ms in it

- 2026-11-10T19:40:58.215Z `1f58e671`:
  21685 ms over its Git steps of 23603 ms,
  the time of writing the answer 20487 ms,
  then the index install under the index lock.
- 2026-11-10T20:11:06.009Z `776171c5`:
  6289 ms over its Git steps of 9213 ms,
  the time of writing the answer 2792 ms,
  then the landing's other reads.
- 2026-11-10T17:49:14.152Z `14893143`:
  6252 ms over its Git steps of 8054 ms,
  the time of writing the answer 2069 ms,
  then `index` in the preparation.

#### the queue: 2 commits, 22133 ms in it

- 2026-11-10T21:31:27.806Z `879c1d02`:
  17459 ms over its Git steps of 18877 ms,
  the time of the queue 15782 ms,
  then `policies` in the preparation.
- 2026-11-10T18:15:07.224Z `4758bdd2`:
  13603 ms over its Git steps of 14002 ms,
  the time of the queue 6351 ms,
  then `scratch` in the preparation.

#### the connection reading the request: 1 commits, 21763 ms in it

- 2026-11-10T20:14:19.701Z `ea66d582`:
  38437 ms over its Git steps of 39283 ms,
  the time of the connection reading the request 21763 ms,
  then the hand-off of the push.

#### the landing's other reads: 2 commits, 20071 ms in it

- 2026-11-10T18:07:59.480Z `1903b2d2`:
  21365 ms over its Git steps of 24295 ms,
  the time of the landing's other reads 8411 ms,
  then `shadow skeleton` in the build.
- 2026-11-10T20:01:39.328Z `4f24fe1d`:
  14602 ms over its Git steps of 15997 ms,
  the time of the landing's other reads 11660 ms,
  then `index` in the preparation.

#### `shadow skeleton`: 3 commits, 15052 ms in it

- 2026-11-10T19:53:50.148Z `0682e732`:
  11572 ms over its Git steps of 20910 ms,
  the time of `shadow skeleton` 7164 ms,
  then the objects.
- 2026-11-10T18:18:01.255Z `d45bfa85`:
  11460 ms over its Git steps of 12081 ms,
  the time of `shadow skeleton` 5388 ms,
  then the hand-off of the push.
- 2026-11-10T20:01:42.002Z `8848ca3a`:
  5182 ms over its Git steps of 6015 ms,
  the time of `shadow skeleton` 2500 ms,
  then writing the answer.

#### the branch move under the index lock: 3 commits, 14193 ms in it

- 2026-11-10T20:04:59.133Z `b0014a55`:
  9910 ms over its Git steps of 10714 ms,
  the time of the branch move under the index lock 8747 ms,
  then the post-commit gate.
- 2026-11-10T18:09:55.157Z `641619da`:
  6002 ms over its Git steps of 13400 ms,
  the time of the branch move under the index lock 2932 ms,
  then `scratch` in the preparation.
- 2026-11-10T18:22:27.620Z `98a7a3b9`:
  5866 ms over its Git steps of 8978 ms,
  the time of the branch move under the index lock 2514 ms,
  then the wait for the lock marker to be flushed.

#### `shadow refs`: 2 commits, 9700 ms in it

- 2026-11-10T18:11:04.206Z `58cf9aa1`:
  13594 ms over its Git steps of 20676 ms,
  the time of `shadow refs` 5915 ms,
  then `scratch` in the preparation.
- 2026-11-10T18:42:27.093Z `644c37e0`:
  12459 ms over its Git steps of 15237 ms,
  the time of `shadow refs` 3785 ms,
  then writing the answer.

#### the wait to place the lock: 2 commits, 6085 ms in it

- 2026-11-10T20:11:59.323Z `94de50c5`:
  6815 ms over its Git steps of 7040 ms,
  the time of the wait to place the lock 3784 ms,
  then the wait for the lock marker to be flushed.
- 2026-11-10T20:09:08.776Z `4d82868d`:
  5812 ms over its Git steps of 9740 ms,
  the time of the wait to place the lock 2301 ms,
  then `scratch` in the preparation.

#### `stagedDeletions` in the preparation: 1 commits, 5921 ms in it

- 2026-11-10T18:38:12.392Z `028c40e8`:
  13060 ms over its Git steps of 16475 ms,
  the time of `stagedDeletions` in the preparation 5921 ms,
  then `index` in the preparation.

### The causes, ranked by the time they cost over those 86 commits

- `shadow conclusion` in the build:
  548909 ms.
- `scratch` in the preparation:
  99671 ms.
- the index install under the index lock:
  81394 ms.
- `index` in the preparation:
  68549 ms.
- the wait for the lock marker to be flushed:
  63413 ms.
- time no phase covers:
  56966 ms.
- the wait for what an earlier batch owes:
  53138 ms.
- the hand-off of the push:
  47329 ms.
- the connection reading the request:
  45455 ms.
- `shadow skeleton` in the build:
  39050 ms.
- writing the answer:
  35071 ms.
- the queue:
  33883 ms.
- the landing's other reads:
  31189 ms.
- the branch move under the index lock:
  28364 ms.
- the post-commit gate:
  22574 ms.
- the wait to place the lock:
  21043 ms.
- `shadow refs` in the build:
  16872 ms.
- the objects:
  15075 ms.
- `stagedDeletions` in the preparation:
  12839 ms.
- `policies` in the preparation:
  12231 ms.
- `shadow links` in the build:
  4388 ms.
- `changedPaths` in the preparation:
  3805 ms.
- waiting for a service to start:
  3199 ms.
- `shadow config` in the build:
  2608 ms.
- `layout` in the preparation:
  1039 ms.
- `unmergedFiles` in the preparation:
  687 ms.
- `capture` in the preparation:
  622 ms.
- `changes` in the preparation:
  158 ms.
- `client build` in the build:
  114 ms.
- `conclusionState` in the preparation:
  46 ms.
- removing scratch:
  25 ms.
- `commit read back` in the build:
  16 ms.
- `amend tree` in the build:
  11 ms.
- `amend fields` in the build:
  1 ms.

### What `main` keeps

The causes over 40,000 ms in that window,
and what each is:

- `scratch` in the preparation,
  99,671 ms:
  `prepare_commit_scratch` makes the commit's tree and,
  before it,
  sweeps the repository scratch directory
  (a listing,
  one `lstat` per entry,
  and a removal of what is older than a day) on the commit's path.
- the index install under the index lock,
  81,394 ms:
  the service builds the new index beside the real one and installs it while it holds the index lock,
  and every other commit of that worktree waits there.
- `index` in the preparation,
  68,549 ms:
  the private copies of the live index that the commit's policies and its shadow commit read.
- the wait for the lock marker to be flushed,
  63,413 ms:
  `lock_stage_ms` with `lock_stage_kind=pre`,
  which is the whole of `take` in `service_index_prestage.rs`.
- time no phase covers,
  56,966 ms:
  the record cannot say.
  `stderrMs` is 0 in all 86,
  `scratchRemovalMs` is 25 and `routeMs` is 160,
  so what is left lies outside every phase the record names.
- the wait for what an earlier batch owes,
  53,138 ms:
  `reads_owed_ms`,
  the index update and the held push a batch owes before its branch moves.
- the hand-off of the push,
  47,329 ms,
  and the connection reading the request,
  45,455 ms.

The three above the marker wait are work and writes a landing or a preparation must make,
and each needs a decision this record does not take:
the index install is the write that makes the landing true for the next command,
and it is flushed under the lock by a decision that a power cut never certifies an index whose bytes are not on
the disk;
the two lock-hold parts are what native Git writes under the same lock;
and the sweep is the guarantee that a tree a killed command left is collected.

The marker wait is different:
nothing is read,
written or decided while a commit waits,
and the file it waits for is the service's own.
Before this change the service made the next batch's marker only after a landing had been answered,
so its flush was the next commit's wait.

### The change: two markers, started when the landing takes its lock

`take_index_lock` now starts the markers of batches `n + 1` and `n + 2` as soon as a landing takes the lock for
batch `n`.
The first overlaps the branch move and the index build that follow;
the second is the spare,
so that one marker is finished while another is being made,
and the next landing takes a file that has had a whole landing of its own to reach the disk.
`service_index_prestage.rs` keeps two files at most:
`start` adds a marker for a batch it is not making,
drops a marker for another index,
and keeps at most `MARKERS_MADE_AHEAD` of them;
`take` keeps the spare and removes only a marker of a passed batch or another index.
The `start_prestage` after the landing stays,
and is a no-op when its marker is already being made.
The shutdown removal iterates both files.

What it costs:
one more flush per landing,
and two small files in the Git directory instead of one,
both of them off the commit's path.

The control failed first,
twice.
With only the next batch's marker started at the lock,
on `native:test:host`:

```text
FAIL [  72.910s] (1/1) landing::service_land_take_index_lock::tests::the_batch_after_this_one_begins_flushing_when_the_landing_takes_the_lock
  thread '...' panicked at src/native/landing/service_land_take_index_lock_tests.rs:73:9:
  no marker of the batch after this one was started while the landing held the lock
```

With the spare removed again,
after the control was widened to the two markers:

```text
FAIL [  63.745s] (1/1) landing::service_land_take_index_lock::tests::the_batch_after_this_one_begins_flushing_when_the_landing_takes_the_lock
  thread '...' panicked at src/native/landing/service_land_take_index_lock_tests.rs:74:9:
  the markers of the two batches after this one were not started while the landing held the lock:
  ["index.lock.cli-git-marker-2532773.4026531836-8-c9088cb9-3f32-4ffb-a219-5c09d78e9cd5"]
```

Both pass after the change:
`PASS [ 0.354s] ... the_batch_after_this_one_begins_flushing_when_the_landing_takes_the_lock`,
and the module's own 15 controls pass with it,
`Summary [ 4.397s] 16 tests run: 16 passed, 6274 skipped`.

### What the arms measured

Method:
one disposable copy of a repository with 300 tracked files,
4 linked worktrees on 4 branches,
one service per worktree at a socket of its own under a private runtime directory,
and every commit made through plain `git` with that arm's program first in its `PATH`.
The fixture sets `commit.cleanup = strip`,
so Git builds every commit and each one needs the private repository.
4 commits per worktree in a row,
the worktrees starting together,
and the arms alternating within one run.
The host was not idle:
load average 94 to 134 on 8 CPUs,
and `/proc/pressure/io` reporting `full` at 27 to 33 percent over ten seconds.
No synthetic load was generated,
and every probe ran in its own `systemd-run --user --scope`.

#### What `main` removes, counted in the records

16 commits through each program,
from the records the commits left in the fixture's common Git directory:

- the installed `0.6.1-alpha.8`:
  `shadow conclusion` in 6 records (20 ms in all),
  `shadow skeleton` in 16 (463 ms),
  `shadow links` in 13,
  `shadow config` in 13,
  `shadow refs` in 13,
  and `shadow pool` in none.
- `main` without this change:
  `shadow conclusion` in none,
  `shadow skeleton` in 9 (230 ms),
  `shadow links` in 7,
  `shadow config` in 5,
  `shadow refs` in 14,
  and `shadow pool` in 12 (1,626 ms).

So the two build causes `main` removes are gone as causes:
no commit of `main` makes a conclusion copies directory,
and 12 of 16 take a member of the pool instead of creating one,
while the 4 that still create one are the fixture's first fill and the members a first claim refuses
(the section "The probe's own records,
and what they named" has the same finding from its own probe).
`shadow refs` stays,
as its code says.
The seconds the conclusion copies cost in this window are the volume's stalls on one `mkdir`
(548,909 ms over 86 commits);
32 shadow commits through two programs did not hit one,
which is why this run states the cause by its presence in the records and the window states its cost.

`ef43a5353e` needed no arm:
none of the 382 records holds a `check-ignore` step,
so there was nothing for it to remove.

#### The marker started earlier

The same method with two wrappers that differ only in this change,
`main` without it and `main` with it,
16 commits each over 3 rounds:

The wait for the lock marker,
`lock_stage_ms` with `lock_stage_kind=pre`,
per landing of each arm,
in whole milliseconds:

- `main` without this change:
  36 pre landings,
  4 of them with no wait,
  270,513 ms in all,
  median 2,176,
  largest 24,178.
- `main` with the two markers:
  36 pre landings,
  22 of them with no wait,
  58,243 ms in all,
  median 0,
  largest 8,782.

The whole commit,
per round and arm,
median and largest:

- without the change:
  1,940 and 3,313,
  8,053 and 24,341,
  10,535 and 19,210.
- with it:
  2,257 and 9,455,
  364 and 9,887,
  361 and 5,117.

The first round of the two arms says nothing (the change is in none of its numbers),
and the two later rounds differ by 22 and 29 times at the median,
which is far outside the spread the unchanged arm showed here (1,940 to 10,535).
An earlier,
narrower change started only the next batch's marker at the lock:
its 36 pre landings held 125,729 ms against the 141,633 ms of the arm without it,
which this host's spread cannot resolve,
and that is why the spare was added.

The controls that hold the mechanism,
from `native:test:host`:

- the control `the_batch_after_this_one_begins_flushing_when_the_landing_takes_the_lock`
  (in `landing::service_land_take_index_lock::tests`):
  FAILED before the change after its 60 s bound with
  "no marker of the batch after this one was started while the landing held the lock",
  and passes after it.
- the prestage module's own controls (`landing::service_index_prestage::tests`)
  and `landing_state_carried_by_its_task`'s:
  `Summary [ 1.514s] 22 tests run: 22 passed, 6268 skipped`.

What these arms can and cannot say is the same limit the section
"What the pool removes from a commit,
counted in calls" recorded for its own arms:
two rounds of one program on this host differ by more than the change does,
so the wall times of the whole command do not resolve it.
What they do show is the wait's own figure,
which the service states per landing,
and that figure falls by 78 percent.

## The repository scratch sweep and a conclusion's copies, off the commit's path (2026-11-11T09:30:00-05:00)

What this answers:
which recorded step carried a wrapped commit over the owner's bound
("commits vs native git should not take 5s longer")
in the records of the installed `0.6.1-alpha.10` on this machine,
and the two changes that took the two largest of them off the commit's path,
each with its own figure before and after.

### Where the numbers come from

The five generations of the main checkout's
`.git/monochromatic-git-policy-cli-commit-timing.jsonl`
(`.4` to the current file),
read only,
as every section above reads them.

- 99 records of the release `0.6.1-alpha.10`
  between `2026-11-11T12:33:40.727Z` (the commit that installed it,
  `a10e75ceac`) and `14:14:28.027Z`.
  Their whole time is 413,882 ms.
- The distribution of `totalMs`:
  smallest 321,
  median 2,218,
  p90 9,580,
  p95 12,087,
  largest 29,039.
  30 of the 99 took more than 5,000 ms in all.
- By the reading the record supports
  ("the bar is the owner's",
  above),
  a commit is over the bound when its `totalMs` exceeds the sum of its own `steps` by more than 5,000 ms:
  21 of the 99.
  Their largest named cause is `shadow conclusion` in 5,
  `scratch` in 6,
  `index` in 2,
  `shadow pool` in 4,
  `provision` in 6,
  and the wait for the service's answer in the rest.
- Where the 99 spent their time:
  `prepare` 43.3 percent,
  `build` 25.6,
  `submit` 16.4 (`answerWaitMs` 61,004 ms of it),
  `provision` 7.2,
  `gate` 2.7.
- The named stretches over the 99:
  `scratch` 44,091 ms (the largest),
  `index` 29,408,
  `shadow pool` 18,076,
  `stagedDeletions` 17,237,
  `policies` 12,034.
- The same log over every generation,
  879 records of the releases `0.6.1-alpha.6` to `0.6.1-alpha.10`,
  holds 167 merge,
  cherry-pick and revert conclusion commits
  (a `buildParts."shadow conclusion"` figure above zero).
  67 of those took more than 5,000 ms in all,
  33 spent more than 5,000 ms in that one stretch,
  the stretch holds 750,263 ms of their 1,655,063 ms,
  and the largest single one is 85,722 ms.
  Two of them are in the window above:
  13:51:58 with 25,992 ms and 13:51:58 with 25,191 ms in that stretch.

### The two changes

1.  A commit makes its scratch tree and collects nothing;
    the resident service collects the repository scratch directory
    beside its landing loop
    (`package/git-policy/cli/src/native/service/service_repository_scratch_sweep.rs`,
    commit `7dd97d64de`).
    A commit never reads what the collection answered,
    and the collection lists the directory,
    asks the file system about every entry,
    and may remove up to `SCRATCH_TREES_REMOVED_PER_COMMAND` trees.
2.  A conclusion's copies are written without a flush of the disk
    (`package/git-policy/cli/src/native/commit/conclusion_copy.rs`,
    commit `f98b61d178`,
    from the cross-vendor audit of the commit and landing route that alpha.10 ships).
    The shadow's copy is read by this process's own `git commit` immediately after it is written,
    the comparison copy by the landing this same command asks for,
    and the service copies what the cleanup compares into the batch's journal before the branch moves
    (`landing/conclusion_state_copy.rs`).
    A copy a power cut never gave back keeps the worktree's own entry instead of removing it,
    so no content is lost.
    The store-held record keeps its flush only where a missing one is damage,
    which is the reftable backend.
    The owner delegated this choice:
    "I think you can decide yourself on which fsync to remove and not have to ask me."

Red controls first:

- `foundation::scratch_location::tests::a_commits_scratch_tree_collects_nothing_on_the_commits_own_path`
  FAILED on the code as it stood with
  "the commit's own path collects nothing,
  so the stale entry is still there for the service's collection".
- `commit::conclusion_copy::tests::a_merge_conclusion_copies_its_entries_without_flushing_the_disk`
  FAILED with
  "nothing of a merge conclusion's copy is flushed:
  [.../shadow/MERGE_HEAD,
  .../transaction/conclusion/MERGE_HEAD,
  .../shadow/MERGE_MSG,
  ...]"
  naming six files.

The controls that hold the other side are
`service::service_repository_scratch_sweep::tests::the_collection_takes_what_a_killed_command_left_beside_the_caller`,
`a_reftable_copy_keeps_the_record_and_its_flush`,
and the four `conclusion_cleanup` controls that remove an entry only while it holds the copied bytes.

### The scratch sweep, before and after

Method:
a disposable `git clone --no-hardlinks` of this repository on this volume,
copied per arm with `cp --archive --reflink=always`,
its own `HOME`,
`XDG_RUNTIME_DIR` and commit service,
SSH signing with a throwaway key as this repository signs every commit,
so every commit builds a shadow;
four linked worktrees of each copy commit in a loop meanwhile
(real commits,
no synthetic load;
the native arm made 195,
196,
194 and 193 of them in the before run and 289,
288,
278 and 275 in the after run,
the wrapped arm 45,
24,
16 and 15,
then 65 in the after run),
and one explicit-path signed commit per round,
alternating the arms.
Sixteen rounds each.
Host load average 47 and 66 during the runs.

- `prepareParts.scratch` over the sixteen commits of a run:
  before 5,738 ms,
  worst 5,306;
  after 938 ms,
  worst 719.
  So the stretch falls by 84 percent and its worst case by 86 percent.
- Whole commits:
  before 27,385 ms over sixteen,
  of which 5,463 ms in their own Git steps;
  after 39,808 ms,
  of which 4,717.
- Native arm:
  before median 475 ms (169 to 18,874,
  one sample lost 18.9 s to a stall of the disk itself);
  after 525 ms (170 to 2,827).
- Wrapped arm:
  before median 1,189 ms (516 to 6,033);
  after 2,096 ms (483 to 9,366).
- Over the bar by the record's own reading:
  1 of 16 in each run.
  The after run's worst commit spent 7,551 ms in `index`,
  a stretch neither change touches.

What that says and what it does not:
the stretch the change targets falls by five sixths,
and the whole-commit figures of one run on this host cannot resolve the change,
exactly as the sections above found for theirs:
the after run drew 48 percent more load commits and one of its commits lost 7.5 s in the private index build,
which the before run did not.
The service's collection is still real work on the same disk,
which is the trade this takes:
it is off the commit's own path,
and it removes nothing while the repository scratch directory holds no entry old enough.

### A conclusion's flushes, before and after

Method:
the script the audit of the commit and landing route used
(`fsync_in_conclusion_copy.sh`,
a 2,000-file fixture,
one `git merge --no-ff --no-commit` and `git commit` per round,
the wrapper under `strace -f -T -e trace=fsync,fdatasync`,
native Git in a copy of the same fixture before it),
six rounds with the binary of the branch point and six with the changed one.

- Before:
  8 client fsyncs in each of the six rounds,
  4.99 s inside them in all,
  0.08 to 1.20 s a round;
  the wrapper's commit 0.42 to 2.09 s (8.21 s in all),
  native Git 0.01 to 0.02 s.
- After:
  0 client fsyncs in each of the six rounds,
  0.00 s;
  the wrapper's commit 0.15 to 1.58 s (2.65 s in all),
  native Git 0.01 to 0.02 s.

The stalls the audit recorded on the same script
(2 to 53 s a commit,
five of ten rounds over native plus five seconds,
one round with single fsyncs of 13.8 s,
17.3 s and 8.1 s)
are what the count turns into:
no flush is left to wait for the disk.

### What this leaves

- `shadow pool`,
  18,076 ms over the 99 records and 6,708 ms in one:
  the claim is one `rename` plus about seventy questions to the file system,
  thirty of them `read_link` and
  `symlink_metadata` per shared entry of the member (`shadow_pool_member.rs`,
  `refusal`).
  Two ways off it,
  neither taken here because each weakens a recorded guarantee
  ("a member is either the tree a creation makes,
  or it is refused"):
  a manifest the fill writes and the claim reads (one file,
  one listing),
  or a member named for the template's digest.
  The first would no longer catch a member changed after it was published;
  the second would catch nothing a name cannot say.
  A cheaper subset is safe and unbuilt:
  `read_link` alone already refuses a missing entry and one that is not a link,
  so the thirty `symlink_metadata` calls are redundant.
- `index`,
  29,408 ms over the 99 and 7,551 ms in one commit of the after run:
  the private index copy and the stat-data walk,
  the "take the walk off the commit's path" item of the section above is still open.
- `stagedDeletions`,
  17,237 ms and 1,874 ms in one commit:
  the stretch holds `git diff-index --cached --diff-filter=D` and no other work,
  yet the record of that commit lists no such step,
  so either the child is unrecorded there or an in-process call stalls.
  Unsettled,
  and named here so the next reading starts from it.
- `provision`,
  29,931 ms over the 99:
  one provisioning per worktree on its first commit after the upgrade,
  1.3 to 4.0 s each,
  which is a manager reload and two starts against the owner's user manager.
  A one-time cost of an install,
  not of a commit.

## The landing's side reference, written without a Git start (2026-11-11T18:31:02-05:00)

What this answers:
what `objects_ms` in the service's `answered` line times,
whether the person's `git commit` waits for it and has to,
and the change that takes the one Git start out of that wait,
with the before and after of the stretch,
the spread of one binary,
and the calls a commit makes.

### What `objects_ms` times

`objects_ms` is one call of `objects_of_batch`
(`src/native/landing/service_land_objects_of_batch.rs`),
and what that call spends depends on which of two shapes the batch is:

- A commit Git built,
  which every signed commit and every merge conclusion is here:
  `read_commit` and `adds_only_present_objects`
  (`service_built_checks.rs`,
  both in process),
  the batch's marker file in the journal,
  and one `git update-ref --no-deref refs/worktree/cli-git-service/<batch> <commit>`,
  the reference that keeps the batch's commits reachable until the branch moves.
- A commit the wrapper builds from the request's changes:
  one `git fast-import`,
  which writes the objects and moves that same reference itself.

The objects of a commit Git built are not written by the service at all:
the client's shadow `git commit` wrote the commit,
its trees and its blobs into the repository's object store,
and the private index wrote the blobs under it (`commit/private_index_stage.rs`).
So for that shape `objects_ms` is a check,
a marker and one reference write,
and the reference is what the person waits on.

The wait is real and it is the answer's:
the service sends `Landed` only after `land_batch_timed` returns,
and the branch move is one transaction that updates the branch and deletes that reference,
so the reference has to exist before the move.
The guarantee it carries is recorded (`SPEC.md`,
"Maintenance that may run while a commit is made",
and the held path of "Merges and commits never fail"):
while the batch is in the landing,
a maintenance that drops objects no reference names cannot take its commits.
That guarantee is why the write stays before the branch move;
nothing in it says a Git process has to make the write.

### The evidence that the wait was a Git start

The two records the earlier section left as a disk stall,
read from the journal of the installed `0.6.1-alpha.10` on this machine
(`journalctl --user`,
2026-11-11T13:48:24-05:00 and 13:52:45-05:00):

- `objects_ms=5651` with steps `git config@168+23;git config@168+22;git rev-parse@195+7;git rev-parse@196+10;git update-ref@544+5314;git update-ref@5894+86;git update-index@6363+51`,
  so 5,314 of 5,651 ms are one `git update-ref`.
- `objects_ms=12695` with steps `git config@1229+4;git config@1229+3;git rev-parse@1236+144;git rev-parse@1237+153;git update-ref@1417+12659;git update-ref@14329+76;git update-index@15348+151`,
  so 12,659 of 12,695 ms are one `git update-ref`.

Both are the batch's own reference,
the first of the two `update-ref` steps (the second is the branch move).

What that child spends is not the write.
Traced on a disposable clone (`strace --follow-forks --syscall-times`),
the write is `mkdir` 83 us,
the exclusive lock file 68 us,
two writes of the object name and a rename of 364 us,
and no `fsync` or `fdatasync` at all,
because Git does not flush loose references by default.
The child that makes it ends 13 to 36 ms after its `execve` in the ordinary case,
and the 311 ms case in the same trace spends 188 ms of that in one gap between two of its own calls.
So the span the person waits on is a process start,
a start that the host's congestion stretches to seconds sometimes.

### The change

`src/native/landing/side_reference_written_here.rs` writes that reference itself,
and Git writes it wherever the landing cannot prove its write is Git's.
The name is the wrapper's own:
a number below a directory the wrapper makes,
never packed,
never symbolic,
and never given a reflog,
because the write this replaces passes `core.logAllRefUpdates=false`.
The file is written where a per-worktree reference lives,
which is the worktree's own Git directory
(measured on Git 2.56.0:
in a linked worktree `update-ref` writes
`.git/worktrees/<id>/refs/worktree/cli-git-service/<batch>`,
and `git rev-parse --git-path` names that same file).
A repository that sets `core.lockfilePid` gets the `~pid.lock` mark Git writes beside the lock,
with this process's number,
removed once the rename names the commit.

The write is left to Git for every shape whose write Git would make differently:
a reference store `core.fsync` flushes,
a `core.sharedRepository` repository,
whose reference files Git widens to the shared mode,
a `core.lockfilePid` value that is not a plain boolean,
a storage that is not plain files,
a configuration the landing read with the caller's own global options,
which proves nothing about the store,
a name that is not the wrapper's own,
an object name that is not whole,
a lock another writer already holds,
a symbolic reference at the name,
and a path this writer cannot make.
Every one of those answers `LeftToGit` and nothing else changes:
Git writes the reference and words its refusal as it always did.

Red controls first:

- `landing::service_land_objects_of_batch::tests::a_batch_side_reference_is_written_without_a_git_start`
  FAILED on the code as it stood with
  "the objects of a batch Git built are written without a Git start,
  and this landing answered Unavailable",
  because the only writer of that reference was a Git child.
- The controls that hold the other side are
  `landing::service_land_objects_of_batch::tests::a_landing_that_leaves_the_reference_to_git_lands_the_same_batch`,
  `landing::side_reference_written_here::tests::the_landing_writes_the_batchs_reference_where_git_writes_it`,
  and the eight controls of that module beside it:
  a standing lock,
  a symbolic reference,
  a name the wrapper does not own,
  an object name that is not whole,
  a path that cannot be made,
  the mark `core.lockfilePid` asks for,
  a second write of one batch,
  and the settings that leave the write to Git.
- `landing::service_land::side_reference_write_tests::a_held_commit_git_built_is_kept_when_its_side_ref_write_outlasts_the_wait_for_it`
  is the release gate's shape,
  a Git write that outlasts its wait,
  and it now sets `core.fsync=reference` in its fixture so that it still exercises Git's write.

### What the write costs made here, and what the child costs

A temporary probe in the wrapped binary (removed again before the commit that shipped it)
printed the span of `write_side_reference` itself:
twelve writes over two fixtures took 143,
177,
177,
184,
188,
192,
206,
206,
209,
214,
220 and 559 us,
so 0.14 to 0.56 ms and a median of 0.20 ms.
The same write by a Git child costs 1 to 6 ms in a commit's own step ledger (`git update-ref` at 1,
1,
2,
2,
3 and 6 ms
across these runs) and 13 to 36 ms traced from its `execve` to its exit,
before any stall:
the journal's 5,314 ms and 12,659 ms records are that same span,
stalled.

### Before and after, one binary on each arm

Method:
the paired harness of the sections above,
one reflink copy of a disposable clone of this repository per arm,
each arm with its own `HOME`,
`XDG_RUNTIME_DIR` and commit service,
one explicit-path signed commit per round,
the arms alternating and the order flipped each round,
the figures read from each commit's own record and the service's `answered` line,
and the runs repeated with the order flipped between them.

- The calls a commit makes,
  which is the currency that does not move with the load:
  two `git update-ref` steps per commit before,
  one after,
  in each of the 36 commits of the two runs,
  Seven Git steps per commit before,
  six after.
- The pair that shows the class the change removes:
  run 1,
  round 1,
  before 5,656 ms of which the reference's `git update-ref` is 5,374,
  after 9 ms.
  Same fixture shape,
  same host,
  same round,
  and the arm that changed is the only difference.
- `objects_ms` over run 1's eight commits:
  before 6,738 ms (median 49,
  largest 5,656),
  after 355 ms (median 15.5,
  largest 144).
- `objects_ms` over run 2's ten commits,
  the order flipped:
  before 52 ms (median 2,
  largest 16),
  after 1,765 ms (median 51,
  largest 1,257).
  The largest of them spent its 1,257 ms in process,
  with the branch move's `git update-ref` still to come at offset 1,259,
  so it is the object check and not this write.
- The same binary on both arms,
  which is the spread this stretch has on this host:
  one arm landing the write (`core.sharedrepository` unset) and the other leaving it to Git
  (`core.sharedrepository=false`),
  ten rounds each:
  `objects_ms` 0,
  26,
  9,
  43,
  539,
  33,
  227,
  18,
  10 and 25 against 16,
  40,
  43,
  152,
  2,005,
  596,
  24,
  19 and 17
  (median 25.5 against 40,
  and the run lost one sample of the Git arm),
  where an earlier run of the same pair read 3,
  1,
  405,
  14,
  35,
  79,
  91,
  23,
  2 and 0
  against 2,
  2,
  2,
  2,
  2,
  12,
  3,
  2 and 2.
  So `objects_ms` moves by more than the write costs either way,
  and no sum over a few commits resolves a change of a millisecond in it;
  the resolved figures are the calls a commit makes,
  the write's own 0.2 ms,
  and the child's 1 to 6 ms or the seconds it stalls for.
- Whole commits:
  run 1 before 25,281 ms over the eight (median 1,677),
  after 21,835 ms (median 2,130);
  run 2 before 6,038 ms over the ten (median 158),
  after 11,383 ms (median 271),
  with `index_update_ms` of 7,536 against 137 in the same rounds.
  The whole-command figures are their unchanged stretch's spread,
  as every section above found.
- The same binary on both arms before the change (`control1`):
  `objects_ms` 1,202 against 500 ms over eight commits (median 67.5 against 51,
  largest 599 against 116).
- Both arms' repositories end with an empty `refs/worktree/cli-git-service`:
  the branch transaction deletes the reference in either case,
  and neither arm leaves a `~pid.lock` behind.

### What this leaves

- The in-process part of `objects_ms` stays:
  `read_commit`,
  `adds_only_present_objects` and the batch's marker file,
  which took 0 to 236 ms across these runs and are the stretch's largest share when the page cache is cold.
  That check is what makes a damaged commit fail before the branch moves,
  and it is not this section's to remove.
- A commit the wrapper builds from its request's changes still spends one `git fast-import` in that stretch,
  which writes the reference itself;
  this repository signs every commit,
  so no commit here takes that shape.
- The branch move's own `git update-ref` stays,
  with the reference lock wait and its bounds,
  because it is Git's compare-and-swap on the branch and the owner's "Never cut a branch write".
