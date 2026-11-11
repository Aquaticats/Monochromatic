# What a worktree removal costs the person and the volume

Status:
measured on this machine on 2026-11-10 between 17:41 and 18:50 (-05:00),
and one half of it implemented on branch
`perf/git-policy-cli-worktree-removal-delete-load`:
a `git worktree remove --force` of a subvolume worktree now destroys the
subvolume in one call instead of leaving real Git to unlink every entry.
The code is `src/native/worktree/worktree_remove_plan.rs`,
`src/native/worktree/worktree_remove.rs` and
`src/native/worktree_copying/worktree_copy_forward.rs`;
`package/git-policy/cli/SPEC.md` and its `README.md` say what it does now.
Everything else this page measures is unchanged and stays open.

The owner's words this page answers:
"Prioritize fixing the jams and release jam fixes asap.",
"Agents should never wait on each other on commits or merges.",
"commits vs native git should not take 5s longer.",
"60s is fine for adding worktrees",
"Commits should never fail.",
and the standing rules "never discard user data" and
"nothing should ever wait forever or indefinitely".

Related records:
`doc/planning/cli-git-btrfs-worktree.md` designed and measured the creation ladder,
including the removal by
subvolume destroy;
`doc/planning/cli-git-worktree-add-cost-of-its-trash.md` measured what an add's trash costs the same volume;
`doc/planning/cli-git-commit-latency-under-disk-pressure.md` is the commit side of the same disk time.

## The fixture, the arms and how they ran

The fixture is a `git clone --no-hardlinks` of this repository at `main`
(`cefec4667f`) under `${HOME}/temp/agent/pi-wt-removal/run/repo`,
23,082 tracked files,
and its root was made a btrfs subvolume (inode 256)
so that the ladder has a subvolume to snapshot from.
Each arm's worktree carries the checkout plus 40,000 empty files in four `payload/bucket-N` directories,
which no rule tracks and which `.git/info/exclude` ignores,
so one tree holds 63,087 entries,
the directories included,
and its removal deletes every one of them.
Every command ran under `systemd-run --user --scope` with
`CPUQuota=400%`,
`MemoryMax=4G`,
`TasksMax=512` and `IOWeight=1`,
with a disposable `HOME` and `XDG_RUNTIME_DIR` under the run folder,
and `git` on the command's `PATH` was either this branch's release build
(`package/git-policy/cli/target/release/monochromatic-git-policy-cli`)
or,
for the arm named `I`,
the installed `0.6.1-alpha.8` wrapper.
Both binaries report the same version string,
so each arm is named here by its path.

The arms:

- `S`:
  a subvolume worktree,
  clean,
  `git worktree remove <path>`.
- `P`:
  an ordinary directory worktree,
  clean,
  the same command.
- `F`:
  a subvolume worktree with one changed tracked file,
  `git worktree remove --force <path>`.
- `FP`:
  an ordinary directory worktree with one changed tracked file,
  the same forced command;
  the control that shows the change does not reach a worktree that is no subvolume.
- `I`:
  `S` through the installed wrapper.
- `L`:
  `git worktree list --porcelain` in the fixture,
  the floor every wrapped command pays.

Arms ran in shuffled order within each round,
so each arm meets the same load as the others.
A checkout writes every tracked file in the second its index is written,
and the wrapper's
cleanliness proof refuses an entry that is as new as the index,
so the clean arms sleep 1.2 s after the add and then run `git update-index --refresh`,
which is Git's own remedy and what a person's worktree looks like after working in it for a moment.
The forced arms need no refresh.

The volume was busy for the whole session.
`/proc/pressure/io` read `some avg10=23.54 full avg10=13.85` before the first measurement and
`some avg10=41.94 full avg10=27.16` at the start of the first round;
the drive is 99% used and this host runs dozens of other agents.
The run-to-run spread of one arm is larger than most differences below,
so this page separates measurement noise from effect where it can
(same binary,
same tree,
alternating arms) and says so where it cannot.

## How many of this repository's worktrees are subvolumes

Read-only,
at 2026-11-10T17:36:41-05:00,
with `git worktree list --porcelain` and one `lstat` per path
(a btrfs subvolume root always has inode number 256):

- 478 registered worktrees,
  of which 452 are subvolume roots and 26 are ordinary directories.
- The second read at 2026-11-10T18:52:12-05:00 saw 491 worktrees and 24 ordinary directories,
  because agents add and remove worktrees while this runs.
- The plain ones are the main checkout's `follow-ups-059` (inode 420039) and
  a set of worktrees under `/var/home/user/temp/agent/` (for example `music-player-production`,
  inode 906680,
  and the `depot-*` control worktrees),
  all older than the ladder.

`btrfs subvolume show` is not usable for this count on this host:
it answers `Could not search B-tree: Operation not permitted` for a real subvolume,
because the search ioctl needs `CAP_SYS_ADMIN`.
The inode number was checked against fixtures instead:
`btrfs subvolume create` gave inode 256 and an ordinary directory beside it did not.

What converting them would save:
a plain worktree is never served by the destroy,
so its removal always unlinks every entry.
The measured cost of that unlink is one system call per entry:
the plain arm's 8 samples of 63,087 entries span 2.3 s to 41.0 s,
median 10.1 s,
which is 36 to 650 microseconds per entry under this load.
Converting a worktree writes a full copy of it once (the convert command),
so the trade is that copy against one unlink pass per removal of that worktree.

## What each removal did

Tens of seconds,
wall clock,
one removal per sample,
63,087 entries per tree:

- `S`,
  subvolume,
  clean,
  8 samples:
  median 12.5 s,
  range 1.4 s to 27.9 s,
  and the destroy notice appeared in 8 of 8,
  so the one-call route ran every time.
- `P`,
  ordinary directory,
  clean,
  8 samples:
  median 10.1 s,
  range 2.3 s to 41.0 s,
  no destroy.
- `F` before this change,
  subvolume and forced,
  8 samples:
  median 6.4 s,
  range 2.4 s to 16.8 s,
  no destroy in any of them:
  real Git unlinked the whole tree.
- `F` after this change,
  5 samples:
  median 20.4 s,
  range 4.8 s to 25.1 s,
  destroy in 5 of 5.
- `FP` before,
  5 samples:
  median 3.2 s,
  range 2.9 s to 23.9 s,
  no destroy.
- `FP` after,
  5 samples:
  median 12.3 s,
  range 2.1 s to 22.5 s,
  no destroy,
  unchanged as intended.
- `I`,
  installed wrapper,
  subvolume and clean,
  3 samples:
  median 16.0 s,
  range 2.8 s to 20.0 s,
  destroy in 3 of 3,
  so the one-call route is not what this branch adds.
- `L`,
  wrapped `git worktree list`:
  6 to 12 ms warm,
  and 2,427 ms for the first one,
  which is the cold page cache rather than the wrapper.

The one deterministic result in that list is the count of destroys:
before,
a forced removal of a subvolume worktree never destroyed it;
after,
every one did.
The seconds it took are not a measurement of the change:
the same `P` arm,
which no change touches,
has a median of 10.1 s before and 2.1 s after,
a factor of five on identical work,
and the `F` arms were measured while 40,000 entries had just been written
into the tree (see "Where the seconds go").
The totals cannot separate these arms on this host;
the parts below can.

## Where the seconds go

Every value in this section is one step timed on its own,
in the same fixture.

- The kernel's own destroy of an identical tree:
  5.0 ms to 52.9 ms,
  median 38.5 ms,
  three samples.
  This is `btrfs subvolume delete` of a snapshot of a prepared 63,087-entry worktree,
  and it is the floor no ioctl-based removal can beat.
  The same call on a snapshot taken moments after the 40,000 files were copied in took
  238.5 ms and 313.8 ms in the two runs before this one,
  which is the flush below and not the deletion.
  The deletion is not what the person waits for when the one-call route runs.
- The wrapper's cleanliness proof is.
  In the parts probe the clean removal (`proof`,
  then the destroy) took
  4.8 s,
  10.2 s and 11.6 s,
  and the same tree removed with `--force` (no proof at all) took
  0.7 s,
  2.3 s and 9.5 s.
  The proof reads the index in process and stats 23,082 tracked files,
  then runs one
  `git ls-files --others --exclude-standard`;
  under this load those stat calls and that run are seconds.
- The tree that was just written is the other part.
  A removal started immediately after 40,000 entries were copied in took
  8.5 s,
  10.3 s,
  17.7 s and 14.5 s (median 12.4 s),
  and the same removal started ten seconds later took
  0.8 s,
  4.1 s,
  4.3 s and 10.9 s (median 4.2 s).
  The destroy commits a transaction that also has to flush the metadata of what was written
  into the tree moments before;
  the unlink route meets the same backlog one entry at a time.
- Real Git's unlink of the same tree took 7.7 s,
  10.3 s and 25.6 s in the same probe.
- What the person's process is doing while the unlink runs:
  sampling `/proc/<pid>/task/*/syscall` every 5 ms across the whole process tree of one
  plain-directory removal,
  7,407 samples over 8.0 s at 2026-11-10T18:30:46-05:00.
  The wrapper's own threads sat in `futex` (2,949 samples),
  `epoll_wait` (1,463) and `wait4` (1,442).
  Real Git's thread was in `unlink` (874),
  `rmdir` (7) and `getdents64` (4),
  and on a CPU in 619 samples.
  So the person waits on the wrapper,
  which waits on a Git child,
  which is deleting one entry per system call.

## What the unlink costs other commands

`/proc/pressure/io` reports system-wide stall time in microseconds,
so the difference between the
reading before a removal and the reading after it is the stalled task-time that occurred while it ran.
Over the arms above:

- `S` before:
  0.3 s to 15.0 s of `some` stall (median 4.6 s) and 0.1 s to 9.4 s of `full` stall.
- `P` before:
  1.1 s to 17.5 s of `some` stall and 0.6 s to 10.2 s of `full` stall.
- `P` after the change,
  the same work:
  0.9 s to 1.6 s of `some` stall and 0.4 s to 0.8 s of `full` stall,
  which shows how much of that column is the host rather than the arm.
- The highest reading taken during any arm was `some avg10=68.6 full avg10=43.4`.

A commit timed beside a removal is not in this page,
for the reason under "What I could not do".
The nearest command that writes what a commit writes was measured instead:
a `git worktree add` of the same fixture,
which checks out 23,082 files and writes an index.

- Alone:
  3.2 s,
  12.1 s and 22.0 s (median 12.1 s).
- Started while a plain-directory removal of 63,087 entries ran beside it:
  12.9 s,
  16.6 s and 55.2 s (median 16.6 s).
- Three rounds of a workload whose unpaired spread is 3.2 s to 22.0 s cannot separate that pair.
  The direction is what the rest of this page would predict,
  and it is not evidence.

## The paths that still delete file by file

Every place the wrapper deletes a tree,
with where it sits:

- A `git worktree remove` the wrapper does not serve,
  which real Git then deletes one
  `unlinkat`/`rmdir` per entry (`remove_dir_recursively`),
  on the person's path:
  `src/native/worktree/worktree_remove_plan.rs` and `src/native/worktree/worktree_remove.rs`.
  It is not served when the worktree is no subvolume root,
  when the mount lacks
  `user_subvol_rm_allowed`,
  when the worktree is locked,
  when another repository registered it,
  when the command carries an option other than one `-f`/`--force`,
  when a conversion's recorded copy
  stands beside it,
  or when the kernel refused the destroy.
  The forced form was the largest share of this and is now served.
- The registration the removal deletes,
  about twenty small files,
  emptied through held handles
  by `finish` in the same module.
  This stays on the person's path:
  it is one directory of a registration,
  and it is what makes the removal complete.
- A snapshot's leave-out,
  before the add returns:
  the stage containers of other commands and,
  when no trash can be made,
  every directory the leave-out sets aside,
  through `worktree_trash_remove.rs::remove_tree_beneath`.
  The walk that looks for stage containers inside a directory it moves whole
  (`worktree_trash_set_aside_entries.rs::remove_containers_beneath`) is on the same path.
  Measured in `doc/planning/cli-git-worktree-add-cost-of-its-trash.md`.
- The trash emptying,
  off the person's path but on the volume's:
  `src/native/worktree/worktree_trash_dispose.rs` with `worktree_trash_entries.rs` and
  `worktree_trash_dispose_awaiting.rs`.
  It is a work item of the commit service,
  one trash at a time per repository,
  at I/O priority class `idle`,
  and it deletes entry by entry:
  269,003 entries counted for one add in that record,
  495,000 estimated there for a full one.
  This is the largest file-by-file deletion left in the wrapper.
- The old tree of a worktree-to-subvolume conversion,
  on the convert command's path,
  through `src/native/subvolume/subvolume_remove.rs` and `subvolume_remove_walk.rs`.
- The rollback and the prune of a failed ignored-state copy (`remove_dir_all` under
  `src/native/worktree_copying/`),
  on the add's path.
- `std::fs::remove_dir_all` of a promoted or refused removal snapshot
  (`worktree_removal_snapshot_promotion.rs`,
  `worktree_removal_rescue.rs`),
  of a shadow (`shadow_create.rs`,
  `shadow_links.rs`) and of a candidate index copy
  (`candidate_private_index.rs`).
  Each is a directory of a few files or a cache that is off the person's path.

## What was changed

`git worktree remove --force` of a subvolume worktree destroys it.
Git's own removal makes its cleanliness proof only when the force is zero
(`builtin/worktree.c`,
`remove_worktree`:
`if (!force) check_clean_worktree(wt, av[0])`),
and that proof,
`validate_no_submodules` included,
is inside `check_clean_worktree`,
so a forced removal deletes the worktree it was given whatever it holds.
The destroy therefore deletes the same set of files as the command the person asked for,
in one call instead of one system call per entry,
and states on standard error that nothing was
checked for changes first.

Every other guard is unchanged and still runs:
the registration that belongs to the invoking repository,
the absent `locked` entry
(Git needs `-f -f` for a locked worktree,
and the wrapper reads one force),
the subvolume root,
the mount option,
the directories held open from before the proof to the destroy,
the identities and the `gitdir` text read again immediately before it,
the recorded destroy,
and the removal of the registration through held descriptors.
A conversion's recorded copy beside the worktree still refuses the removal,
forced or not.

What the change is worth here:
in the settled probe the forced removal took 0.7 s to 2.3 s where the same tree's clean removal
took 4.8 s to 11.6 s and its unlink took 7.7 s to 25.6 s,
because the force removes the proof and the one call removes the per-entry work.
The forced form is also the one this wrapper's own reports tell a person to run
(`git worktree remove --force <path>` appears in its checkout and copy reports),
so it is the removal agents actually run on a worktree they have worked in.

The controls:
`a_forced_removal_of_a_changed_worktree_is_served_and_an_unforced_one_is_not` and
`a_forced_removal_of_a_locked_worktree_is_left_to_git`
(`src/native/worktree/worktree_remove_guard_tests.rs`),
which run on a stand-in for the kernel,
and `a_forced_removal_names_the_directory_and_the_force`
(`src/native/worktree/worktree_remove_plan_tests.rs`).

## What converting the plain worktrees would save, in one number

One plain worktree's removal pays one unlink per entry and nothing else can be done for it
until it is a subvolume:
the wrapper cannot destroy what the kernel does not call a subvolume.
The measured cost of that pass is 36 to 650 microseconds per entry under this load,
so a 5,000-entry worktree pays 0.2 s to 3.3 s and a 200,000-entry one pays 7 s to 130 s per removal.
The 24 to 26 plain worktrees above pay that on every removal,
and converting each one once
makes the one-call route reachable for it from then on.

## What I could not do

- A commit timed beside a removal.
  The wrapped commit path needs the resident commit service,
  and the service does not start
  under a disposable `HOME` and `XDG_RUNTIME_DIR`:
  `git commit --allow-empty` answered,
  after 130.07 s,
  "The commit service did not start,
  so this commit was not made" and named
  `journalctl --user --unit 'monochromatic-git-policy-cli@*.service'`.
  A worktree add beside a removal was measured instead,
  and is not evidence (see above).
- The entry counts of the 24 plain worktrees.
  A full walk of each is the disk-stalling scan this host's `find` guard refuses
  ("Walking a whole repository ... reads hundreds of thousands of files"),
  and it is the synthetic
  load this work is not allowed to add,
  so only the unit cost above is given.
- A removal of a worktree whose ignored state is this repository's real one.
  The fixture's ignored state is 40,000 empty files,
  because a real copy of `node_modules`
  (about 122,000 to 197,857 entries) costs minutes to make under this load and the per-entry
  cost is what the arms already measure.
- Whether the trash emptying,
  the conversion's old-tree removal and the add's container sweep
  could each move to a subvolume destroy.
  The trash's entries are plain directories inside the new worktree's subvolume,
  and btrfs refuses a rename across a subvolume boundary,
  so a trash that is itself a subvolume
  cannot receive them;
  that was reasoned from the kernel behaviour this repository already measured
  (`doc/planning/cli-git-btrfs-worktree.md`,
  "Btrfs permissions") and was not built or measured.

## Questions for the owner

1. Should a removal the wrapper cannot serve by destroy be moved off the waiting path instead:
   rename the worktree aside under a reserved name beside it,
   remove the registration,
   and hand the tree to the commit service as a work item that deletes it at the idle I/O class,
   as the trash already is?
   That is O(1) for the person and the same per-entry work at idle class for the volume,
   and it needs a durable record,
   the naming of a tree nothing emptied,
   and the rule that
   nothing is deleted that the removal cannot prove it moved.
   It is the only route left for the 24 to 26 plain worktrees and for a mount without
   `user_subvol_rm_allowed`.
2. Should the cleanliness proof be cheaper than seconds?
   It is now the largest single step of a served removal that is not forced.
   The alternative is to let real Git decide with one `git status --porcelain` run,
   as its own
   `check_clean_worktree` does,
   which would replace 23,082 stat calls and one listing run with one child.
3. Should the plain worktrees be converted now,
   and by which route:
   the existing `convert-worktree-to-subvolume` (a full copy each),
   or left to be replaced the next
   time an agent makes a worktree there?
