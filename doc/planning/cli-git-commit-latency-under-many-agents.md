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
except the last hour of the window, 19:00, where it is 56 percent and the
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
  by hour, medians of 5846 ms at 11:00,
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
  with medians of 5 ms, 3 ms and 4 ms and maxima of 250 ms, 8815 ms and 1924 ms.

A read-only probe of the calls themselves says the Git directory was not charging
more per lookup than a fixture:
2000 `stat` calls on the repository's own `.git` (79 entries) took 5.8 us each,
2.7 us for a name that does not exist and 26.0 us for a directory listing,
against 3.5 to 4.5 us, 2.8 us and 22 to 24 us in the shaped fixtures.
What the creation waits for is the machine's device, hour by hour;
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
  Windows, where a shared file is a copy and not a link, keeps the removal first.
- On the files backend the wrapper writes the shadow's private target ref,
  its symbolic refs and its `HEAD` itself (`shadow_refs.rs`),
  as it already wrote `packed-refs`:
  a loose ref is the object name and a line feed,
  a symbolic one is `ref: <target>` and a line feed.
  The reftable backend keeps Git,
  because its refs live in tables only Git writes.
- A branch that is itself a symbolic ref to another branch keeps its chain,
  and the private target is written where that chain ends,
  at most `SYMBOLIC_REF_CHAIN_LIMIT` steps, which is Git's own limit
  (`shadow_refs.rs`).
  `git update-ref`, which the third step replaces, follows such a chain;
  a direct write at the first name would turn the chain into a direct ref that a
  hook can tell apart.

Red controls first, for the two steps that changed what the wrapper does:

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
the existing controls for a replaced template entry, a linked worktree's private
`info` and the shared `rr-cache` cover it.

### After

The same method, the same fixture shape, the same 39 private index copies, one
commit traced with that call filter (each trace is its own fixture, so the
figures are per call and not per run of the same tree):

- 306 calls against 392;
  `symlink` 8 against 47,
  `statx` 42 against 89.
  The first commit of a fixture, which does more,
  fell from 1695 calls to 1612,
  with `symlink` 44 to 5 and `statx` 470 to 426.
- the commit started one Git for the shadow's refs where it started three:
  `git for-each-ref` stays,
  and the `git symbolic-ref` and `git update-ref` are gone from the trace
  entirely,
  and from the records: 3 starts in each of 8 commits before, 1 in each of 8
  after.
  The person's own records count four before the change,
  one `git for-each-ref`, one `git update-ref` and two `git symbolic-ref` per
  commit,
  and the shaped fixtures have one symbolic ref fewer than that repository.

Three arms of 8 commits each, alternating which arm went first,
in fixtures of this tree's shape that carry the common directory's 39 private
index copies,
where the first two arms are the same binary on unchanged code:

- whole commit, by the record's own total:
  median 719 ms (241 ms to 30 370 ms) and, on the identical arm,
  842 ms (254 ms to 22 862 ms), against 285 ms (246 ms to 8070 ms).
  The identical arms differ by 1.17 times at the median,
  and each of them caught one stall of 23 to 30 s that the changed arm did not,
  so this record claims no whole-commit improvement from the medians alone.
- the creation's own stretch:
  median 5 ms and 4 ms before (3 ms to 38 ms and 3 ms to 50 ms),
  median 2 ms after (2 ms to 8 ms).
- the shadow's Git starts: 3 in every before record, 1 in every after record.

What stays, and why:

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
  they are the repository Git is handed, and each is one call.
- the `readlink` calls in path resolution and the `getdents64` of the one
  directory listing, which the links need to know what to link.

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
  "The shadow's creation, measured and made cheaper".
  It was 93 to 99 percent of each hour's `buildMs`,
  and its calls fell from 392 to 306 in a fixture of this tree's shape,
  with the shadow's Git starts down from three to one per commit.
- Still open, and now named:
  the wrapper's other scratch directories in a common directory
  (`cli-git-scratch`,
  `cli-git-captures`,
  `monochromatic-git-policy-cli-scratch`,
  `monochromatic-git-policy-cli-merges`,
  and its own `*.jsonl` logs) are still linked into every shadow.
  They are six or seven entries, not thirty-nine,
  and no measurement says a hook needs them,
  so this record does not take them; the private index copies were taken because
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
the index read,
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
- Each arm ran twice, so the spread of one binary on unchanged input stands beside each figure.

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
five samples per arm (tracing inflates the spans, so these figures size the calls and not the step):

- Before the change,
  the commit's own trace held, per sample,
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
  the publication happened, in the service, and nothing waited for the device.

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
  which the landing of a batch waits for and which a batch needs once.

The first and the third are the preparation's,
which another branch is changing,
and the second is the gate's.

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
  Both sit in the stretch the section "The shadow's creation, measured and made
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
  The ref store is written unsynced and not through a lock file, on the argument
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
