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
730 records of the log and its one generation,
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

## What this could not determine

- Which in-process step inside a preparation spends the median 332 ms.
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
