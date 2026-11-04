# One reclaimer per physical index

Written 2026-11-04 on `fix/git-policy-cli-one-reclaimer-per-physical-index`, from `main` at `05c16d73d2`.
A proposal for the owner's veto; the pick and what was built are in "Pick" and "What was built".

## The race

A reclaimer of an index lock reads the marker, finds the holder gone, and then takes the lock aside by one rename
(`reclaim_if` in `package/git-policy/cli/src/native/landing/service_index_reclaim.rs`).
Two steps cannot be one: when a second reclaimer removed the stale lock and placed its own live lock between the first
reclaimer's look and its rename, the first reclaimer takes the live lock aside, finds it live, and puts it back by a hard link.
While the live lock is off its name, a third program (Git) can create `index.lock`; the hard link then fails, and the live
holder's lock is stranded at its `.aside-` name.

The control `a_live_lock_taken_aside_by_a_late_reclaimer_is_never_open_to_a_third_program` in
`package/git-policy/cli/src/native/landing/service_index_reclaim_tests.rs` is red on `main` plus the `reclaim-aside` fault point:
the third program places its lock while the live lock is off its name, and the put-back reports `File exists`.
It is marked `#[ignore]` so the branch stays green; run it with `--run-ignored all`.

What the stranded holder does at its release (read from `service_index_hold.rs` and `service_index_ownership.rs`, not run):
the ownership check answers `Replaced`, so it installs nothing, logs that another program replaced the lock, and leaves the batch
open; a restart then finds the live index is not the file the landing saw and replays nothing.
The commit stays landed and no staging is overwritten; the commit's paths stay un-updated in the index.
A holder that passed its ownership check just before the rename aside installs its index while a third program holds the lock,
and one of the two index writes is then lost.

## Who reclaims

Three call sites reclaim, not one:

- `remove_stale` (`service_land_finish.rs`, restart of a dropped or finishing batch);
- `try_acquire_on` (`service_index_lock.rs`, reached from the restart's `settle` and from `index_reset_support.rs`);
- `wait_and_place` (`service_index_attempts.rs`, the live landing).

One service serves one worktree's own index, so an index inside the instance's Git directory has one reclaimer.
Two instances reclaim one physical index only when it is an alternate index (`GIT_INDEX_FILE`) shared by their worktrees,
or one instance's alternate index is another worktree's own index.

## Options

### A: refuse to finish an index outside the instance's own Git directory (the first pick, replaced; see "Pick")

Pros:
- no new lock, no new file;
- recovery no longer reaches a shared alternate index at all.

Cons:
- it covers `remove_stale` and `try_acquire` from `settle`, not `wait_and_place`, which a live landing on an alternate index
  runs; two live landings from two worktrees on one alternate index still race;
- extended to the live landing, a dead service's marked lock on an alternate index is never reclaimed, and the commit that meets
  it waits, then is withdrawn with the index in use (read, not run: `look_at_holder` and `classify_index_lock` end the wait
  at once for a dead pid file and at `LOCK_CEILING` for anything else); that is a failed commit on a path that succeeds today,
  against "Commits should never fail.";
- a restart no longer finishes an alternate index, which `SPEC.md` states it does today.

Visible result: after a kill during a landing on an alternate index, the commit's paths stay un-updated in that index.

### B: serialize the reclaim under a non-blocking exclusive `flock`

Every reclaimer takes `LOCK_EX | LOCK_NB` before it removes a stale lock; the kernel drops the `flock` on death, so no leftover
can block a reclaim, and a reclaimer that does not get it backs off.
Precedent in the tree: the sweep of a dead service's private copy and the conversion lock hold an exclusive `flock`.
Two variants, B1 and B2, differ in what the `flock` is taken on.

Pros of B (both variants):
- every reclaimer of one physical index serializes, whatever the number of worktrees and instances;
- the commit path still reclaims, so no commit fails for a stale lock;
- the hold lasts microseconds and leaves nothing behind.

Cons of B (both variants):
- a lock, against "Use as few locks as possible."; its reason is written beside it in the code and the lock inventory
  (`service_lock_inventory_tests.rs`) lists the file;
- `flock` is a hold of this kernel: reclaimers on different hosts over a shared filesystem are not serialized, and on a
  filesystem that gives no `flock` the stale lock is not removed at all (see "No `flock`" below);
- Git does not take it, but Git never reclaims, so it is not a party.

#### B1: the `flock` is on the stale lock file's own inode

The reclaimer opens the lock it will judge, reads the marker from that descriptor, takes the `flock` on that descriptor, checks
that the name still refers to that inode, and unlinks the name.
A second reclaimer that opened the same inode fails the `flock` and backs off.
The rename aside and the put-back by hard link are not needed any more, because no other reclaimer can remove or replace the
file while the `flock` is held, and a live landing cannot place a lock while the name is occupied.

Moving parts: one descriptor, one `flock`, one inode comparison. No sidecar name, no directory hold, no leftover.
The unit is the one file being reclaimed, the smallest there is.
The physical index is identified by the inode, so two spellings of one path (symlinks, bind mounts) need nothing more.

Interleavings:
- two reclaimers: closed. The one that holds the `flock` removes; the other backs off, or opened the name after the removal and
  judges whatever is there (a live lock is never removed), or got the `flock` after the first finished and finds the name no
  longer refers to its inode.
- a reclaimer against a live landing: closed. A landing places a lock only where the name is absent, and the name stays occupied
  from the reclaimer's inode check to its unlink. A live lock found at the name is judged by its marker and left.
- a person's `rm index.lock` in the window: left, narrowed. Between the inode check and the unlink a person can remove the
  stale lock and Git can place its own, and the unlink then removes Git's lock. No system call removes a name only if it still
  names a given inode, so no variant closes it; the window is two system calls.

#### B2: the `flock` is on a sidecar file or on the lock's directory

The reclaimer takes the `flock` on `index.lock.reclaim` (or on the Git directory), then runs today's sequence (rename aside,
look again, remove or put back).

Moving parts: the sidecar name or the directory descriptor, the `flock`, the rename aside, the look again, and the put-back by
hard link, which still has its empty window against Git and its failure mode of stranding a lock.
The sidecar form leaves a file name behind ("Allow the rare leftover") and two spellings of one path make two sidecars; the
directory form serializes the reclaim of every index in that Git directory.

Interleavings:
- two reclaimers: closed, when both take the same sidecar or directory.
- a reclaimer against a live landing: left. A live landing is not a party to the `flock`; the sequence that takes a lock aside
  and puts it back runs on, and a live lock taken aside in it is off its name for as long as the window lasts.
- a person's `rm index.lock` in the window: left, as in B1, and wider, since the window is the whole sequence.

### C: document the gap (the round 5 option B)

Pros: no code. Cons: leaves the race open for two worktrees on one alternate index, and a stranded lock that needs a person.

### Ranking

B1 > B2 > A > C.
B1 over B2 because it has fewer parts, leaves no name, identifies the physical index by its inode, and closes the live-landing
interleaving that B2 leaves.
B2 over A because B2 serializes every reclaimer at every call site and no commit fails for a stale lock, while A closes one
call site or fails commits.
A over C because A closes the restart route at the cost of the restart finishing an alternate index, and C closes nothing.

### Pick

The coordinator picked B under the owner's delegation ("Pick options as if I had picked them and invite veto"), replacing A, and
then B1 for having fewer moving parts; the owner's veto is open.
The reason for replacing A: it covers only the restart, and extended to the live landing it makes a stale lock on an alternate
index fail the commit at `LOCK_CEILING`, against "Commits should never fail."

## What stays open under every option

A lock removed by hand (a person running `rm`), and a Git on another host sharing the filesystem, are not parties to any
serialization this service can make.

## What was built

B1, at all three reclaim sites, in `package/git-policy/cli/src/native/landing/service_index_reclaim.rs`:
`reclaim_if` opens the lock with `O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC`, reads the marker from that descriptor, takes
`LOCK_EX | LOCK_NB` on it, checks device and inode against the name, and unlinks the name through `Platform::unlink`.
It answers `Removed`, `Changed`, `Busy`, `Kept` or `Stuck`; `remove_stale` ignores the answer, `try_acquire_on` places again after
`Removed` and `Changed` and otherwise leaves the batch open, and `wait_and_place` continues after `Removed` and `Changed`,
fails with `Unavailable` after `Stuck`, and waits as for any held lock after `Busy` and `Kept`.
The lock is on the inventory in `source_guard/service_lock_inventory_tests.rs` with its reason above the call.

Controls in `service_index_reclaim_tests.rs`:
- red first (`dd37f0c462`), green now: `a_live_lock_found_by_a_late_reclaimer_is_never_open_to_a_third_program`;
- refused: `a_second_reclaimer_of_one_stale_lock_backs_off_while_the_first_holds_it`,
  `a_reclaimer_whose_judged_file_was_replaced_removes_nothing`, `a_lock_placed_after_the_stale_one_was_found_is_not_removed`,
  `a_foreign_lock_and_a_running_holders_lock_stay`, `a_landing_waits_for_the_reclaimer_that_holds_a_stale_lock`;
- allowed: `a_stale_lock_is_removed_and_leaves_no_file`, `a_reclaimer_that_backed_off_can_reclaim_once_the_hold_is_dropped`.

Each guard was shown to matter: with the `flock` result ignored, the controls for the backed-off reclaimer and the landing's
wait fail; with the inode comparison always true, the controls for a replaced file fail.

Added after the cross-vendor review of `9db8794047` (`/var/home/user/temp/agent/reviews/one-reclaimer-sol-1.txt`, verdict
"correct after named fixes"): the three fixes in the next sections, and the sweep of aside files.

## No `flock`

The first build removed the stale lock anyway when the `flock` failed for a reason other than another reclaimer holding it. The
review showed that this route deletes a live lock: two reclaimers without the hold both pass the inode check, one removes the
stale lock and a landing places its own, and the other then unlinks the name, which is the live lock; Git can then create its own
lock beside a landing that has passed its ownership check. Every error but `WouldBlock` took that route, not only a file system
without `flock`; an NFS client that emulates `flock` with byte-range locks can refuse a lock on a read-only descriptor.

Pick (the coordinator's, under the owner's delegation "Pick options as if I had picked them and invite veto"; veto open): a
reclaimer that cannot take the hold never removes the lock (`Reclaimed::Unguarded`). A landing treats the lock as a held lock:
the ordinary wait to `LOCK_CEILING`, then the withdrawal that keeps the commit on its side ref. Its notice names the stale
lock's path, the batch and the dead process from its marker, and says to remove it by hand with `rm` once no git command runs
there, as native Git leaves a stale `index.lock` to the person. A restart leaves the batch open.
An interrupted call is not a refusal: it is treated as a hold by another reclaimer and asked again by the wait.

Alternatives considered:
- Remove anyway, as the first build did. Rejected: the interleaving above deletes a live lock, and it needs no manual `rm`.
- Fall back to the old rename aside and put-back where there is no `flock`. Rejected: it brings back the empty-name window in
  which Git takes the name and the live lock is stranded, for exactly the file systems that cannot be tested here.
- Fail the landing at once. Rejected: "Commits should never fail."; the bounded wait lets the person remove the lock and the
  commit land, and the withdrawal keeps the commit on its side ref.
- Land the commit and leave the index update for a later recovery when the lock cannot be had. Not taken: the update needs the
  lock to be safe, and the replay gate would have to accept a batch with no lock ever held; a larger design change.
- Retry the `flock` on a descriptor opened for writing, for NFS emulation. Not built: opening another account's lock file for
  writing may fail, and the case is unexercised here; it would narrow the set of file systems that wait for a person.

Ranking: not removing > failing at once > falling back to the aside window > removing anyway.

## A busy stale lock

A `Busy` answer leaves no event in the directory when the other reclaimer lets go without unlinking (it died, or its file was
already replaced), so the wake of the wait on the directory would come only with the notice timer. The wait now looks again after
`POLL_PAUSE` while the last answer was `Busy`, inside the same acquisition and under the same ceiling.
The review also asked whether the holder classifier ends the wait at once for a stale marker: it does not for a service marker,
which has no pid file, and the control `a_landing_takes_the_stale_lock_when_the_other_reclaimers_hold_is_dropped` shows the
acquisition succeeding with no dead-holder announcement.

## The control for the old aside window

`a_reclaimer_that_finds_a_live_lock_leaves_no_event_on_the_lock_name` watches the directory with inotify while a reclaimer works
on a live lock, and fails on any rename, removal or creation of the lock's name, so it needs no hook at a point inside the
reclaimer. Run against the code of `dd37f0c462` (a throwaway worktree, since removed) it fails with the rename of `index.lock`
to its `.aside-` name in the events; against the current code it passes.

## Aside files an earlier release left

`remove_stranded_asides` runs after each placement, beside the sweep of dead staging files. It removes
`<lock>.aside-<process>-<time>` files that have no other name and that no process has open (a live holder keeps its file open),
and writes what each held into the log line. They are lock files, not user data. A scan that stopped at its time bound leaves them.
The descriptors of other accounts' processes cannot be listed; a holder of these files is a service of this account.
