# The records a worktree add writes for itself are flushed by nobody

Status:
implemented on branch `perf/git-policy-cli-worktree-add-within-60-s-on-the-pass-12-tip`;
`package/git-policy/cli/SPEC.md`,
"The records an add writes for itself",
says what the code does.
The owner's words this answers:
"60s is fine for adding worktrees",
and "nothing should ever wait forever or indefinitely".
The repository's rules this answers are `SLW` and `NQH`:
measure under the pressure there is,
what we add over native `git worktree add`,
and remove it,
make it cheaper,
or move it off the waiting path.

The rule that decides it is already in `SPEC.md`,
"Local durability follows what the remote already holds":
the baseline is what native Git syncs for the same data,
and where the wrapper's own records are concerned the coordinator decides,
with a test that leaves the record absent,
torn or empty as a power cut can,
and shows the next command proceeds at once and loses nothing the rule protects.

## The fixture

`${HOME}/temp/agent/worktree-add-60s/measure.ts`,
run on 2026-11-11 on this repository's own volume
(btrfs on dm-crypt,
`compress=zstd:3`,
89 % full,
load average 90 to 100 while other sessions worked):

- one disposable btrfs subvolume holding a local clone of this repository
  (23,419 tracked files) and the empty ignored `.claude/worktrees` folder this repository adds into;
- one `btrfs subvolume snapshot` of that clone per measured add,
  so every add starts from the same bytes and the source is a subvolume root,
  which is what the ladder's snapshot rung needs;
- two arms: `git worktree add --quiet -b <name> .claude/worktrees/<name> HEAD` spelled identically,
  through the wrapper (its own `bin/git` link leading `PATH`) and natively (`PATH` without that link);
- each arm twice, from separate fixtures, so a difference between two copies of one binary bounds the spread;
- a home and a runtime directory of its own per arm,
  a `DBUS_SESSION_BUS_ADDRESS` naming a socket that does not exist
  (so neither arm can reach a commit service or this machine's session bus),
  `GIT_CONFIG_NOSYSTEM=1`,
  and `core.fsmonitor=false`,
  as the owner decided on 2026-11-03;
- each fresh fixture warmed with one untimed `git status --porcelain=v1 -z --untracked-files=normal --no-renames`,
  because an established checkout's stat data is in the page cache and a freshly snapshotted one's is not:
  the first `git status` of a new snapshot took 21.7 s here and the second 0.57 s,
  which is the fixture's cost and not the add's.

Every wrapped add writes its own split into
`<common-git-dir>/monochromatic-git-policy-cli-worktree-add-timing.jsonl`
(`src/native/worktree/add_timing.rs`),
so the steps below are the wrapper's own record of the same runs,
not a second instrument.

## What an add spends, and what native Git spends

A `strace -f` of one wrapped add and one native add of the same fixture,
same environment,
same tree
(`--trace=fsync,fdatasync,syncfs,sync_file_range,ioctl,renameat2,rename,unlinkat`):

- native `git worktree add`: no `fsync` at all, four `rename` calls, 8.5 s wall clock;
- the wrapped add: twelve `fsync` calls summing 43.4 s of 82.5 s wall clock.

The twelve were
the stage claim and its journal directory (1.98 s + 9.92 s),
the destination record written three times, planned, named, bound (4.62 s + 1.88 s and two more),
the registration's owner proof (3.35 s),
the copy's install log, its stage binding and its container directory (2.84 s, 2.54 s, 2.24 s, 1.87 s),
and the record of an unfilled worktree (0.93 s, 0.82 s).

One `fsync` of a small file on this volume,
measured on its own while other sessions wrote,
took 33 ms at one moment and 8.0 s at another,
because a flush waits for the filesystem's log commit,
which waits for every other writer's data.
The same wait is why `mark_complete_unsynced` already existed for an add whose destination a subvolume was swapped into.

## The change

The four records an add writes for itself are published by an atomic rename and flushed by nobody:

- the stage claim `<id>.stage`, written before the container exists;
- the destination record `<id>.add`, written planned before real Git runs and rewritten when the add names its Git
  and when the binding is recorded;
- the proof `monochromatic-git-policy-cli-copy-owner` in the new registration's administrative directory;
- the removals the recovery makes when it settles a dead add's claim and its journal root.

Nothing else changes:

- the completion marker keeps its flush, because recovery removes a registered worktree whose add has no marker,
  so a lost marker would remove a worktree a person was told about;
- the journal of a copy, its install log and the record of an unfilled worktree keep theirs;
- the trash and the set-aside entries keep theirs, which is the rule's own case of a person's uncommitted bytes
  being moved.

## The controls

`worktree_copy_stage_claim_tests.rs`,
`the_records_of_an_add_are_published_whole_and_flush_nothing`:

before the change it failed with

```
assertion `left == right` failed: an add's own records flush no file
  left: [".../ac7208a8-...stage....tmp", ".../ac7208a8-....add....tmp", "...add....tmp", "...add....tmp"]
 right: []
```

and it also holds the positive control that the recorders see a flush:
the same claim's completion marker is still flushed.

`worktree_add_binding_tests.rs`,
`the_proof_of_who_made_a_registration_flushes_nothing`:

before the change it failed with

```
assertion `left == right` failed: the proof of who made the registration flushes no file
  left: [".../repo/.git/worktrees/topic/monochromatic-git-policy-cli-copy-owner/monochromatic-git-policy-cli-copy-owner"]
 right: []
```

`worktree_copy_stage_claim_tests.rs`,
`an_absent_destination_record_costs_no_flush_and_loses_no_bytes`,
which is the test `SPEC.md` asks for when a sync is removed:
a destination record removed as a power cut can remove it,
then the next command:

before the change it failed with

```
assertion `left == right` failed: and none for a directory
  left: [".../worktree-copy/v1"]
 right: []
```

and after it the recovery asks for no filesystem commit,
the person's source keeps its bytes,
the worktree Git registered stands,
and what the cut costs is the notice that names the worktree,
never a byte.
