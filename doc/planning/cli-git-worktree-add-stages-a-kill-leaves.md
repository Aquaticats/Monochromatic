# The stage an interrupted `git worktree add` leaves, and what an add pays for besides its repair

Status:
the reaping fix is implemented on this branch and its control is in
`package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_claim_tests.rs`;
the cost inventory below is measured but not yet answered, and it says which step is next.
The owner's words this answers:
"60s is fine for adding worktrees",
"nothing should ever wait forever or indefinitely",
and the cleanup rule that nothing the wrapper made is left for a person to remove.
The repository's rules this answers are `SLW` and `NQH`:
measure, under the pressure there is, what the wrapper adds over native `git worktree add`,
and remove it, make it cheaper, or move it off the waiting path.

## The live case

On 2026-11-11 two adds were killed from outside while they worked.
At 22:54 a command in another worktree printed, among other lines:

```text
monochromatic-git-policy-cli: the stage "<repo>/.claude/worktrees/.cli-git-worktree-copy-9f56192e-…" was left by an interrupted `git worktree add` and was not changed: nothing shows what .claude/worktrees/add-explains-slow-route is (creation time is not available for the filesystem), so nothing below it was examined and none of it was removed; remove it yourself once nothing needs it (`btrfs subvolume delete` for a snapshot, `rm -r` otherwise).
```

Nobody would ever run a command inside the worktree whose add died, so the other line of that
notice, "the first command run in that worktree makes it", never came true either.
The stage stood until a person deleted it at 23:56.

The audit of release 0.6.1-alpha.12 in
`${HOME}/temp/agent/sonnet-audits/alpha-12-worktree-add-remove-and-landed-sweep`
(finding F2, with `reproductions/s1_stage_with_stub.ts` and `s6_add_killed_at_times.ts`) found the
two shapes of one cause. A snapshot of a source that holds a nested worktree subvolume carries an
empty stand-in (inode 2) wherever that subvolume was, and the sweep that takes the stand-ins off
the waiting path replaces the folder around them with a fresh empty one. Both are younger than the
record of the stage's object, and both were judged "not the wrapper's own":

- a stand-in has no birth time at all, so `metadata.created()` fails and the entry is reported as
  unreadable;
- the fresh folder is born a few milliseconds after the record, so it is reported as an entry the
  record does not account for.

Every later add then repeated the notice and left the stage, and with no nested subvolume in the
source the same kills were reaped by the second add. Four kills left four stages in the audit's run.

## The change

A folder that holds no entry at all is accounted for whatever its birth time says
(`worktree_copy_stage_entries_the_record_accounts_for.rs`, `emptiness`).
An empty folder holds no byte of anybody's, so removing it discards nothing, and everything that
does hold a byte is judged exactly as before: an entry born after the record is still unaccounted
for, and a folder that holds one keeps the whole stage and is named by its path.

The control is `a_stage_holding_only_the_placeholders_a_snapshot_leaves_is_reaped`:

```text
thread '…::a_stage_holding_only_the_placeholders_a_snapshot_leaves_is_reaped' panicked at
src/native/worktree_copying/worktree_copy_stage_claim_tests.rs:2208:5:
a stage that holds nothing but the add's own empty folders goes: ["monochromatic-git-policy-cli: the stage
"…/.cli-git-worktree-copy-90e267c4-…" was left by an interrupted `git worktree add` and was not changed: it holds
.claude/elsewhere, which the record of the object it made does not account for; remove it yourself once nothing needs
it (`btrfs subvolume delete` for a snapshot, `rm -r` otherwise).", …]
```

and the protection kept is `a_folder_younger_than_the_record_that_holds_a_persons_file_keeps_the_stage`,
which passes before and after the change: the same kill with one file written where the add leaves
an empty folder keeps the stage and the line names the path.
`native:test:host:btrfs` is where a real stand-in is exercised, because a stand-in exists only on
btrfs.

## What an add pays for, besides the package repair

Every number here is the wrapper's own step record
(`<common>/monochromatic-git-policy-cli-worktree-add-timing.jsonl`), read from three real adds of
this repository on 2026-11-12 between 03:54 and 04:01 with release 0.6.1-alpha.12, on a host whose
load average was far above its 32 CPUs and whose native `git worktree add` took 8.5 s to 45 s in
the same hours. One of those adds took 324,835 ms in all, 107,125 ms of it the package repair
(`repairCargo`), which is another branch's step; the add's own path was therefore about 218 s
against a bound of 60 s.

- `prune`, 145,317 ms (and 154,149 ms and 60,253 ms in the other two):
  the leave-out's walk. A snapshot of the main checkout carries everything, so the walk visits every
  directory to find the caches, the run records, the other checkouts and another command's stage
  containers, and for each directory it sets aside whole it walks that whole subtree looking for a
  stage container. This repository's `.claude/worktrees` holds 507,992 directories at depth six and
  `node_modules` 21,594 more, and a warm walk of `node_modules` alone took 646 ms to 1,259 ms here,
  so the price is the number of directories the walk visits, cold. Native Git does none of it.
  Its reason is the leave-out (`doc/planning/cli-git-worktree-add-cost-of-its-trash.md`,
  `doc/planning/cli-git-main-worktree-ignored-copy.md`) and the credentials a stage container holds.
- `installIndexFile`, 35,679 ms (2,289 ms and 1,640 ms in the other two):
  the new worktree's index written and flushed. The writer is
  `write_all_then_sync` (`subvolume_file_write_steps.rs`), so this is one `fsync`, which on this
  volume waits for the filesystem's own commit and for every other writer. Native Git fsyncs
  nothing for the index of a worktree it adds. This is the same class as the records the earlier
  branch stopped flushing (`doc/planning/cli-git-worktree-add-records-flushed-by-nobody.md`) and is
  not yet changed.
- `restoreDirty` (13,376 ms and 44,244 ms) with `git clean` (5,407 ms and 9,826 ms) inside it:
  only for an add whose source has uncommitted changes; it restores the copy to the new `HEAD`.
- `copyIgnored`, 8,095 ms (12,337 ms and 24,324 ms): the lifecycle's own copy of the ignored state,
  one `git ls-files` listing and the copies after it.
- `snapshotIoctl`, 2,951 ms (5,157 ms and 7,597 ms): the snapshot itself, which forces one btrfs
  transaction commit on the calling thread; the owner decided every add takes a fresh snapshot.
- `recover`, 1,388 ms (3,608 ms and 1,324 ms): the reaping and the settling of earlier adds'
  leftovers, on every command's path. This repository's journal root held 494 entries, most of them
  `settlement.lock.*.pending` directories from October, so the cost grows with what earlier commands
  left rather than with this command's work.
- `recordDestination` 1,835 ms, `reserveDestination` 579 ms, `git ls-files` 1,035 ms in five starts,
  `git check-ignore` 97 ms, `git status` 144 ms, `markCompleteFile` 4,477 ms in one add,
  `handOverTrash` 724 ms, and `otherMs` 107,083 ms, which is the repair and the record's own gaps.

The wall clock of one add cannot judge any of these on this host: two copies of one binary differed
by more than any difference between arms in the earlier branch's two runs. What does judge them is
the same instrument on the same add, the step record and the syscalls a `strace --follow-forks`
counts.

## What is next, largest first

1. `prune`'s walk: 145 s of a 218 s add. Two costs sit in it, and they need separate answers. The
   walk over every directory cannot be skipped without asking Git for the ignored tree instead of
   walking it (`git ls-files --others --ignored --exclude-standard -z`, one start, the same listing
   the copy already takes). The walk over each directory that is set aside whole, looking for a
   stage container, is proportional to a subtree that is then renamed into the trash in a tenth of a
   millisecond; the containers it looks for are all named by claims in the journal root, so the
   proof could read them instead of walking, with a control that a container no claim names is still
   found.
2. `installIndexFile`'s flush: one `fsync` of the new worktree's index, 35.7 s. Native Git does not
   sync it, the file is published by a link under an exclusive lock, and the module's own doc
   already says which writes may stay in the page cache.
3. `recover`: make the journal root's cost follow this command's work, not October's leftovers.
4. The `add_standing_notice` steps the audit's finding F5 lists: seven steps an add enters have no
   sentence, so a person watching a slow add reads a generic line while `prune` holds it.
