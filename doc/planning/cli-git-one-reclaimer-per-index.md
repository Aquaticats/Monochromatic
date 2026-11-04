# One reclaimer per physical index

Written 2026-11-04 on `fix/git-policy-cli-one-reclaimer-per-physical-index`, from `main` at `05c16d73d2`.
A proposal for the owner's veto; nothing here is built except the red control named below.

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

### A: refuse to finish an index outside the instance's own Git directory (the coordinator's pick, as worded)

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

### B: serialize the reclaim sequence under a non-blocking exclusive `flock`

The reclaim (rename aside, look again, remove or put back) runs only while the reclaimer holds `LOCK_EX | LOCK_NB` on the
lock's directory, or on a sidecar file beside the lock.
The kernel drops the `flock` on death, so no leftover can block a reclaim; a reclaimer that does not get it skips the reclaim and
loops back to its placement attempt, which finds either the lock gone or the other reclaimer's work done.
Precedent in the tree: the sweep of a dead service's private copy holds an exclusive `flock` on its directory.

Pros:
- every reclaimer of one physical index serializes, whatever the number of worktrees and instances;
- the commit path still reclaims, so no commit fails for a stale lock;
- no leftover state, and it lasts microseconds.

Cons:
- a lock, against "Use as few locks as possible."; the reason is the one the rule asks for, and its unit is one reclaim;
- `flock` is a hold of this kernel, like the private-copy sweep: reclaimers on different hosts over a shared filesystem
  are not serialized;
- Git does not take it, but Git never reclaims, so it is not a party;
- the directory form serializes the reclaim of every index in that Git directory, the sidecar form leaves a file name behind
  ("Allow the rare leftover").

Visible result: none for a person; the control above goes green.

### C: document the gap (the round 5 option B)

Pros: no code. Cons: leaves the race open for two worktrees on one alternate index, and a stranded lock that needs a person.

### Ranking

B > A > C.
B over A because B closes all three call sites and no commit fails for a stale lock, while A closes one or fails commits.
A over C because A closes the restart route at the cost of the restart finishing an alternate index, and C closes nothing.

## What stays open under every option

A lock removed by hand (a person running `rm`), and a Git on another host sharing the filesystem, are not parties to any
serialization this service can make.
