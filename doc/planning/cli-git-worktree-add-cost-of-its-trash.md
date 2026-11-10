# What a worktree add pays for the directories its snapshot leaves out

Status:
the coordinator picked the route below under the owner's "Please make any non-destructive options
(when they are the ones I would likely have picked) yourself from now on for this session.",
and the owner may veto any part of it.
Implemented on branch `fix/git-policy-cli-worktree-add-does-not-wait-for-its-trash`;
`package/git-policy/cli/SPEC.md`,
"Leaving out of a snapshot",
and its README say what the code does.

The owner's words this route answers:
"never discard user data",
"nothing should ever wait forever or indefinitely",
"Use as few locks as possible.",
"Agents should never wait on each other on commits or merges.",
and, for worktrees,
"60s is fine for adding worktrees".

## What was measured on 2026-11-10

Ten `git worktree add` runs took more than ten minutes each on this machine on 2026-11-10.
The samples are `${HOME}/temp/agent/pi-runs/why-worktree-adds-are-slow-sample-1.txt` and
`why-worktree-adds-are-slow-sample-2.txt`,
made by `sample-why-worktree-adds-are-slow.ts` beside them.

Across every thread of those adds over 92 s,
the second sample counts 54 samples on a CPU and 2,796 blocked,
of which 2,182 are inside `unlink` or `rmdir`:

- `btrfs_unlink` under `filename_unlinkat`, waiting for a transaction: 981 samples,
- the same `unlink` waiting to reserve metadata: 555,
- `btrfs_rmdir` under `filename_rmdir`: 160,
- the same `rmdir` waiting to reserve metadata: 143.

The folders the processes held open were each add's own `.claude/cli-git-trash/<id>`,
the drive was 99% busy,
and the first sample counts 12,638 B-tree operations from `unlink` and `rmdir` over its window.
Nine adds ran at once in that window and each of them wrote about 270 MB.

Why the trash exists at all:
a snapshot of the main checkout carries everything,
so the leave-out sets the caches,
the other checkouts and the run records aside into that trash by rename
(`src/native/worktree/worktree_trash.rs`),
which costs a tenth of a millisecond per directory.
Emptying it is what costs minutes,
and every add emptied its own trash before it returned.

## Measured on 2026-11-11: the change, on this volume

The owner stopped this measurement to land the fix, so the numbers below are what was collected before
that, and nothing is estimated.

The fixture:
one disposable repository whose source is a btrfs subvolume,
holding 15,000 ignored entries the leave-out sets aside
(half in `.cache/alpha`, half in `target/debug/incremental`)
and 2,500 it keeps,
made by `${HOME}/temp/agent/wt-add-measure/measure3.ts`.
Each arm ran three adds at the same moment and the arms alternated,
so the same-binary spread is visible:
the arm order was before, after, before, after.

- before arm, three adds at once: 144.48, 133.37 and 133.98 s,
  144.48 s until the last returned.
- after arm, three adds at once: 244.33, 244.35 and 268.78 s,
  268.80 s until the last returned.
- before arm again: 273.91, 272.74 and 272.73 s, 273.94 s.
- after arm again: 100.28, 104.37 and 103.12 s, 104.64 s.

What the adds' own timing records say they spent on the trash
(`.git/monochromatic-git-policy-cli-worktree-add-timing.jsonl`):

- The before arm's emptying (`disposeTrash`) took 156, 175 and 181 ms in the first arm and
  185, 203 and 212 ms in the third.
- The after arm's hand-over (`handOverTrash`) took 12,075 and 12,158 ms in the two adds that
  found the repository's service not yet running, and 1 ms in the add that found it answering.
  In the fourth arm it took 1, 4,187 and 1 ms.

What this says, and what it does not:

- The same-binary spread is 1.9 times within the before arm (144 s against 274 s) and 2.6 times
  within the after arm (105 s against 269 s),
  so the host's load moved these numbers far more than the change did,
  and this fixture cannot separate the two.
- With 15,000 entries the emptying was 156 to 212 ms of a 133 to 274 s add on this drive today,
  so this volume under today's load does not reproduce the 2026-11-10 condition,
  where 2,182 of 2,796 blocked samples were inside `unlink` or `rmdir` and btrfs commits took
  9.2 s each.
- The first hand-over in a repository whose service is not running waits for the service's greeting,
  bounded by the same start wait a commit uses; 12.1 s was measured under this load.
  Every later hand-over in that repository answered in 1 ms.
- What the change does remove is in the records themselves:
  the before arm has `disposeTrash` work and a `disposeTrashWait` step,
  the after arm has neither,
  and the deletions it used to run now happen in the service after the add returns,
  at I/O priority class `idle`, one trash at a time for the repository.

Not measured, and stated as missing rather than estimated:

- How long the service takes to empty one of these trashes at the idle class,
  and how much the idle class helps on this drive.
- An add whose trash is as large as the real repository's,
  which the leave-out counting section below bounds but does not settle.

## The four steps the owner set

The route is four steps,
and each one is a fact about where the deletions are:

1. The add returns once the worktree is in place,
   without waiting for the trash to be emptied.
2. A standing trash cannot be staged whatever the target commit ignores.
3. The emptying runs off every command's waiting path,
   one trash at a time across the repository,
   at the lowest I/O priority Linux offers.
4. A trash left by a killed add or a stopped service is still emptied later,
   is never lost track of,
   and is still reported when it cannot be emptied,
   and a later add never carries a trash into its own copy or snapshot.

## A standing trash cannot be staged

The repository's own `info/exclude` holds one line,
`/.claude/cli-git-trash/`,
written before the leave-out asks Git whether that place is ignored
(`src/native/worktree/worktree_trash_exclude_line.rs`).
No commit writes that file,
so no target commit can decide whether another command's state is stageable.

Measured with `/usr/bin/git` 2.56.0 on 2026-11-11,
in a throwaway repository with a linked worktree
(`${HOME}/temp/agent/trash-exclude-probe/probe.ts` and `probe2.ts`):

- Before any line,
  `git check-ignore --no-index` in the linked worktree names nothing,
  and `git status --porcelain` says `?? .claude/`.
- With the line in the **common** `info/exclude`,
  `check-ignore` names that same file and the paths below it,
  `git status --porcelain` says nothing,
  and `git add --all` stages nothing of a trash standing there.
- With the line only in `$GIT_DIR/worktrees/<name>/info/exclude`,
  nothing is ignored and the trash is untracked:
  a linked worktree reads the common directory's file and never its own.
- A target commit whose `.gitignore` ignores nothing under `.claude/`,
  and one that writes `.claude/` then `!.claude/cli-git-trash/`,
  both leave the trash out of `git add --all`.
  Only the `.gitignore` itself is staged.

The controls that hold this are
`real_git_keeps_a_standing_trash_out_of_the_index_of_a_linked_worktree` and
`a_target_commit_that_stops_ignoring_the_trash_place_cannot_stage_it`.

## One trash at a time, and no new lock

The record a trash is handed over with is written into the repository's **common** work directory,
`<common>/monochromatic-git-policy-cli/work/pending/trash-<name>`,
and the service is woken for that common directory.
A service round drains one lane per repository
(`src/native/landing/service_round.rs`),
and the drain of one work directory empties its trashes one after another
(`src/native/landing/service_work_drain_pending_claims.rs`).
So concurrent adds enqueue beside each other and never delete beside each other,
whichever worktree holds each trash.

No lock is added for this:
the claim the drain takes is the lease every other kind of work already uses,
and a record whose holder died is taken back when its lease runs out.
The emptying itself is the face that already existed for a work item of an earlier release,
`empty_trash` (`src/native/worktree/worktree_trash_dispose.rs`),
with the same identity checks:
the worktree's `.git` must point into the record's Git directory or one of its `worktrees` folders,
the trash must be the object the leave-out made,
and each entry removed must be the object moved there.

## The lowest I/O priority Linux offers

Each removal of a trash entry,
and the removal of the trash directory itself,
runs inside I/O priority class `idle`
(`src/native/worktree/idle_io_priority.rs`,
one `ioprio_set` before the work and one after it to put the thread back).
The class is set on the thread that unlinks,
because the service pushes a branch and reads the repository on its other threads while a trash is emptied.
A thread whose class cannot be read or set is left as it is,
and the emptying is correct either way.

## What the trash divides up into

The owner stopped this walk to land the fix, so it did not finish:
`${HOME}/temp/agent/trash-leave-out-probe/probe2.ts` had walked 28,000 of the main checkout's
directories and had not reached the largest trees when it was stopped.
No count below is an estimate: each is what the walk had counted at that point,
written as it went, at about 40% of the walk.

- the run record `.monochromatic` below a `node_modules`: 202 directories, 165,161 entries
- `.pnpm-store`: 1 directory, 37,501 entries
- `zig-global-cache` below a `target`: 1 directory, 24,392 entries
- `incremental` below a `target`: 7 directories, 17,992 entries
- another checkout, a directory holding `.git`: 28 directories, 16,537 entries
- the run record `verification` in a `target`: 7 directories, 2,661 entries
- the run record `mutants.out` and `mutants.out.old`: 1 each, 1,740 entries each
- a `.cache` directory below the root: 221 directories, 1,106 entries
- `.gradle`: 4 directories, 135 entries
- `.kotlin`: 2 directories, 18 entries
- the run records `test-results`: 3 directories, 18 entries together
- the run record `playwright-report`: 1 directory, 2 entries

At that point the walk had counted 269,003 entries moved into a trash by one snapshot,
and 111,052 entries a new worktree keeps.
The largest single directories it had found were
`package/git-policy/cli/node_modules/.monochromatic` with 160,034 entries and
`.pnpm-store` with 37,501.
The main checkout has 345 registered worktrees,
which this count does not include:
most are subvolumes, which a snapshot does not carry at all.

## What would disappear if the largest folder were a subvolume

This is the owner's choice; nothing of it was built.

Measured on this volume by `${HOME}/temp/agent/pi-runs/probe-btrfs-subvolume-delete-and-rename.ts`:
a snapshot does not carry a nested subvolume,
so if a folder the leave-out sets aside were a subvolume of the main checkout,
the snapshot would show an empty stand-in where it stands,
and the add would neither delete it nor move it.

From the counts above, the largest folder is `node_modules/.monochromatic`,
the run records of this repository's own tooling:
were it a subvolume in the main checkout,
roughly 165,000 of the 269,000 deletions that one snapshot's leave-out causes would not happen,
which is the largest single share of them.
The next two, `.pnpm-store` and `target/*/zig-global-cache`, would take another 62,000 with them,
so those three kinds alone are most of what an add used to delete.

Nothing measured says what makes a folder a subvolume again:
`pnpm` and `zig` recreate their cache folders as plain directories,
and this repository's own tooling recreates `.monochromatic` as one,
so a subvolume made once would be replaced by an ordinary folder the next time the tool runs.
That is why this is a question for the owner and not a change:
making it work means either teaching each tool to leave a subvolume in place,
or having the wrapper make the folder a subvolume again when it finds it plain,
and both are design changes of their own.

## What the add now takes

- One record and one wake, instead of emptying the trash inside the command.
  In the fixture above that was 1 ms once the repository's service was answering,
  and 12.1 s the first time in a repository whose service was not running,
  which is the same bounded start wait a commit uses.
- Adding `/.claude/cli-git-trash/` to the repository's `info/exclude` once,
  inside the step the add already ran to ask Git whether that place is ignored:
  1,057 ms of work in the fixture's first add under this load, once per repository.
- No `disposeTrash` work and no `disposeTrashWait` step in the add at all.
- The deletions moved off the add's path are the manifest's entries:
  7,500 in the fixture (one left-out directory of 7,500 names and one of 7,500),
  and in the real main checkout at least 269,003 by the unfinished count above.

## What is left for the owner

1. The subvolume question above, which is a design change of its own.
2. The first hand-over's wait for a service that is not running (12.1 s measured).
   The alternatives are a wake that writes its line and returns without the service's greeting,
   which loses the confirmation that the service took the message,
   and a wake that only connects to a socket that already answers,
   which leaves the record for the next command that reaches the service.
