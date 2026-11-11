# The statx a worktree add spends filling the new tree

Status:
implemented on branch `perf/git-policy-cli-worktree-add-tree-fill-cost`;
`package/git-policy/cli` says what the code does.
The owner's words this answers:
"Priorize on unblocking jams.",
"By "prioritize on unblocking jams" I also meant fix the wrapper.",
and,
for worktrees,
"60s is fine for adding worktrees".

The coordinator's reading of a `strace -f` of a real add on `0.6.1-alpha.8`
left one large cost unexplained:
about 750,000 `statx` calls while the add filled the new tree,
on a btrfs volume on a SATA SSD that measured 99 percent busy.
Two other costs of those adds were already gone on `main`
(the stand-in removals of `807f3ea26d`,
the Cargo build-state clone and clean of `ea00ad0f10`).
This page is the measurement of what remains,
and of what the fix removed.

## The fixture

`${HOME}/temp/agent/wt-add-statx/fixture.ts`,
run at
2026-11-10T17:35:12-05:00 over 545 s here:
one disposable btrfs subvolume holding a `git clone --shared` of this repository
(23,082 tracked files,
2,407 of their directories),
the ignored state a working checkout of it carries
(4,800 files in a root `node_modules`,
3,000 in twenty per-package stores,
400 build-state files and 600 incremental-cache files under `target/debug`,
50 run records,
200 files in `.cache`),
three nested worktrees that are btrfs subvolumes of their own,
as every worktree of this repository is,
and one nested worktree that is an ordinary directory holding a `.git` marker,
as a worktree stays until it is converted.
The fixture's own commit drops the LFS attribute lines:
`git-lfs` is not on the fixture's search path,
and the snapshot rung proves a file's bytes in both worktrees,
so an LFS attribute would refuse the rung for a reason unrelated to this page.

The wrapper under test answers as `git` through a link in the fixture,
and every add runs with a home and an `XDG_RUNTIME_DIR` of its own under the run folder,
with `GIT_CONFIG_NOSYSTEM` set and a global configuration that does not exist.
The two binaries are `${HOME}/temp/agent/wt-add-statx/before/git`
(this branch at `aa7f601c6d`,
built at 2026-11-10T16:51:07-05:00)
and `${HOME}/temp/agent/wt-add-statx/after/git`
(the same branch with the fix,
built at 2026-11-10T17:38:41-05:00).

## How the system calls were attributed

`add.ts` runs one add,
optionally under
`strace --follow-forks --summary-only`
for the classes above one percent of the calls,
and under
`strace --follow-forks --decode-fds=path --trace %stat,%file`
for the per-program and per-tree counts.
The wrapper reaches a file through a held directory handle by naming
`/proc/self/fd/<n>/<name>`,
so `breakdown.ts` follows the descriptors each pid opened and closed
and turns every name into the path it names.
`attribute.ts` ties each pid to the program it executed,
which tells the wrapper's own work from the Git child it starts.
`stacks-stream.py` reads a `strace -k` run of the debug build
and counts,
for every `statx`,
the first frame of the wrapper that is not the
standard library's,
which names the function that asked for it.

## What one add spends, before the fix

One add of the fixture,
traced at 2026-11-10T17:08:40-05:00
(`before-count-before-counts.txt` and `before-trace-before-trace.txt`):

- 183,094 calls in all,
  of which `statx` 85,224,
  `openat2` 24,529,
  `close` 34,855,
  `getdents64` 9,849,
  `openat` 6,436,
  `pread64` 7,441 and `fsync` 9.
- The classes above one percent of the trace's time are
  `futex` 62.14 percent and `epoll_wait` 21.13 percent,
  which is the multi-threaded runtime idle under the trace and not the add's work;
  then `statx` 8.62 percent (0.930 s),
  the snapshot `ioctl` 1.81 percent (one call,
  0.161 s),
  `openat2` 1.48 percent,
  `wait4` 1.34 percent and `close` 1.24 percent.
- By the program that issued them:
  the wrapper's own process 47,263 `statx`,
  the process the wrapper forks for its blocking proof 36,855,
  and real Git 576.
  The add's `statx` are therefore 99.3 percent the wrapper's own work,
  and real Git's checkout is not what fills the tree with them.
- By the tree each call named:
  23,042 on the tracked files of the snapshot stage,
  which the index proof walks;
  22,129 on the tracked files of the source,
  which the cleanliness proof walks;
  14,964 on directories of the stage and 14,640 on directories of the source,
  in both cases from re-opening the path to each directory;
  6,926 on the ignored state the leave-out sweeps;
  2,024 on the directories the proof opens;
  and the rest on `/proc`,
  `.git` and the records.
- By the function that issued them,
  from a `strace -k` run of the debug build
  (`git_policy_cli` frames below the standard library only):
  `subvolume::fs_beneath::entry_metadata_at` 52,955,
  `subvolume::fs_beneath::device_of` 28,919,
  `subvolume::fs_kind::is_stand_in` 2,525,
  `process::process_tree::descendants_of_read` 2,053,
  and 180 from `repo_read::repo_read_tree::walk`.

The two walks that own them are
`worktree::worktree_clean::check_tracked_by_content`
through `worktree::worktree_clean_tree_walk::tree_refusal_beneath`,
one `statx` per tracked file of the source,
and
`worktree::worktree_snapshot_index_step::install_index`
through `worktree_clean_tree_walk::tree_fields_by_content`,
one `statx` per tracked file of the stage.
Neither of those two is removable:
each is the proof the rung is built on.
What was removable sat in three places beside them:

1. `subvolume::fs_beneath::open_directory_beneath` read the root's device and then
   every directory on the way with one `statx` each,
   and both walks called it again for every directory change,
   so the whole chain above a directory was opened and examined again
   whenever the next entry was in a sibling directory:
   28,919 `statx`
   and the `openat2` calls under them.
2. `worktree::worktree_trash_set_aside_entries::remove_containers_beneath`
   asked the kernel whether each entry of a directory it sets aside is a directory,
   one `statx` per entry,
   although the listing it had just read says so:
   6,926 `statx`.
3. `fs_kind::is_stand_in`,
   which the leave-out calls for every directory of the snapshot,
   is a lookup that the listing cannot answer
   (see "What a listing cannot answer" below):
   2,525 `statx`,
   kept.

## What the fix changed

`subvolume/fs_beneath_directory_chain.rs`,
`DirectoryChain`,
holds the root and the chain of directories a walk has opened,
with the root's device read once,
and opens only the components a path does not share with the one before.
It refuses a link,
a mount and a directory on another device exactly as
`open_directory_beneath` does,
because it makes the same calls for the components it does open.
`worktree_clean_tree_walk.rs` walks through it in three places:
`tree_refusal_beneath`,
`tree_fields_by_content` and `StatCursor`.

`worktree_trash_set_aside_entries.rs`,
`remove_containers_beneath`,
takes the entry's type from the `read_dir` listing it already read,
through `listed_directory_of`,
and falls back to `is_directory_entry_at` only for a listing that does not say what an entry is.
The set of entries it walks and removes is unchanged.

## What one add spends, after the fix

The same fixture,
the same fixture state one add apart,
traced at 2026-11-10T18:19:12-05:00
(`after-count-after-counts.txt` and `after-trace-after-trace.txt`):

- 101,901 calls in all,
  of which `statx` 53,606,
  `openat2` 4,480,
  `close` 10,018,
  `getdents64` 9,852,
  `openat` 6,445,
  `pread64` 7,441 and `fsync` 9.
- `statx` 85,224 to 53,606,
  which is 31,618 fewer and 37 percent fewer.
- `openat2` 24,529 to 4,480,
  which is 20,049 fewer and 82 percent fewer.
- `close` 34,855 to 10,018 and the calls in all 183,094 to 101,901.
- `getdents64` is unchanged (9,849 to 9,852):
  every directory is still listed once,
  and no directory the leave-out or a proof used to read is now unread.

By tree,
before and after:

- the tracked files of the stage,
  the index proof:
  23,042 to 23,082;
- the tracked files of the source,
  the cleanliness proof:
  22,129 to 23,084;
- directories reopened on the way,
  stage:
  14,964 to 2,501;
- directories reopened on the way,
  source:
  14,640 to 2,255;
- the ignored state the leave-out sweeps:
  6,926 to 151;
- the directories the proof opens:
  2,024 to 2,024.

What is left is the two proofs' one read per tracked file,
one device check per directory each walk opens,
and one read per stand-in candidate.

## The wall time the change does not show

The adds were timed by alternating the two binaries in the same fixture,
one pair after another,
between 2026-11-10T18:05:33-05:00 and
2026-11-10T18:11:11-05:00,
each run adding a worktree of its own name.
The wall time is what the add returned in,
and the second number is the
`totalMs` the add's own record in
`.git/monochromatic-git-policy-cli-worktree-add-timing.jsonl` states:

- before,
  53,588 ms (record 53,412 ms);
- after,
  46,431 ms (record 44,368 ms);
- before,
  24,262 ms (24,135 ms);
- after,
  11,134 ms (11,105 ms);
- before,
  36,316 ms (36,291 ms);
- after,
  65,511 ms (65,455 ms);
- before,
  88,273 ms (88,236 ms);
- after,
  57,967 ms (56,625 ms);
- before,
  6,281 ms (6,216 ms);
- after,
  1,599 ms (1,548 ms).

The spread of the same binary is 6.3 s to 88.3 s before and 1.6 s to 65.5 s after,
which is 14 times and 41 times,
so this host under its load cannot separate these two binaries by wall time:
the before arm is faster in three of the five pairs and slower in the other two,
and the processor time the add and its children used
(851 to 2,418 ms before,
850 to 2,236 ms after)
moves less than the syscall counts do and still overlaps.
The step record moves the same way:
`checkSource` is 188 to 328 ms before and 158 to 395 ms after,
and `walkSnapshot` 337 to 5,431 ms before and 293 to 352 ms after,
with the largest values in runs whose other steps were also slow.

What the counts are worth in time,
measured rather than estimated:
under the trace,
the `statx` these adds made took 0.930 s before and 0.753 s after,
the `openat2` 0.154 s and 0.036 s,
and the `close` 0.129 s and 0.038 s.
The trace inflates each call,
so this bounds the saving at about 0.2 to 0.4 s of an add that took
1.5 to 88 s here,
and it does not settle what it is on a host that is not loaded.

## What keeps its old shape, and why

- The one `statx` per tracked file in each of the two proofs:
  the cleanliness proof is what makes the source a snapshot's source,
  and the index proof is what makes the snapshot's files the index it installs.
  Neither is measured from the other,
  because they are two instants,
  and a source that changed between them must not become a worktree whose index lies.
- The device check on every directory a walk opens:
  a nested btrfs subvolume has a device of its own,
  and `openat2`'s `RESOLVE_NO_XDEV` does not refuse one,
  because a subvolume is not a mount.
  What the chain removed is reading the same device again for components it still holds.
- `fs_kind::is_stand_in`'s own lookup for every directory of the snapshot,
  and the same question in
  `worktree::worktree_snapshot_placeholders::is_stand_in`.
- The `fsync` of the add's record (`record_binding` in
  `worktree_copying/worktree_copy_stage_claim.rs`),
  which is what makes "a killed add leaves a record or nothing" true.
  It is also the step that moves most between runs:
  0.01 s to 1.9 s here,
  and one earlier broadcast run of this fixture saw 3.1 s in it.

## What a listing cannot answer

The first attempt at the stand-in question also took its inode number from the
`read_dir` listing,
with `DirEntry::ino()`.
`worktree::worktree_snapshot::tests::btrfs_a_snapshot_removes_the_stand_ins_of_stages_and_keeps_the_one_no_worktree_names`
refused it at 2026-11-10T17:27:41-05:00:
the folder it names was removed,
where the control says it must stay.
Btrfs reports a stand-in's dentry number in `getdents`,
not its inode number:
measured here at 2026-11-10T17:33:04-05:00,
`os.scandir` names `79139` for a stand-in whose `lstat` says `2`,
while a plain directory's two numbers agree (`257`).
So a listing can say whether an entry is a directory and cannot say whether it
is a stand-in,
and only the second question changed back.

## The controls

Added,
`subvolume/fs_beneath_directory_chain_tests.rs`,
four controls for the chain:
a deeper path after a sibling keeps the part it shares and answers the same directory,
a link is refused,
a directory on another filesystem is refused,
and a nested btrfs subvolume is refused while a plain directory beside it is not.
Run at 2026-11-10T17:19:02-05:00:
`Summary [0.044s] 4 tests run: 4 passed, 6289 skipped`.

The existing controls that hold the behaviour this page changes nothing about,
run at 2026-11-10T17:52:41-05:00:

- `Summary [106.688s] 231 tests run: 231 passed, 6062 skipped`,
  which is every test whose name holds
  `worktree_add`,
  `worktree_clean`,
  `worktree_trash`,
  `worktree_snapshot`,
  `worktree_ladder`,
  `fs_beneath`,
  `fs_kind` or `directory_chain`.
- The binary-level `fault_trace` and `worktree_copy` tests:
  `Summary [447.707s] 73 tests run: 73 passed (2 slow), 851 skipped`
  at 2026-11-10T18:32:04-05:00 with `--test-threads 2`.
  The same group run without that limit at 2026-11-10T18:11:31-05:00,
  beside another session's full suite,
  gave
  `Summary [947.011s] 75 tests run: 67 passed (27 slow), 7 failed, 1 timed out, 849 skipped`,
  and `fault_trace::a_worktree_copy_killed_at_each_unlink_leaves_a_recoverable_repository`
  passed alone in 63.8 s at 2026-11-10T18:05:12-05:00.
  Seven tests whose every case is a kill and a recovery do not become wrong
  because the host is busy,
  so the serial run is the verdict and the parallel one
  is the contention `AGENTS.md` rule `SUF` describes.

## What was not measured

- The wall-time saving on a host that is not loaded.
  This one was,
  and the same-binary spread is wider than the change.
- The same counts on the real repository,
  which no wrapper may run against.
  The real checkout has 427 registered worktrees where this fixture has four,
  all of them subvolumes,
  so the stand-in step and the leave-out see more directories there;
  the two proofs read one entry per tracked file and scale with the 23,082 a clone carries.
- The 750,000 `statx` the coordinator's trace counted on `0.6.1-alpha.8`.
  The counts here are 85,224 for one add on this branch,
  and the difference between the two releases is not attributed in this page.
- How much the `/proc` scan inside `descendants_of_read` costs on a host with
  more processes than this one:
  2,053 `statx` here,
  in one add's trace.
