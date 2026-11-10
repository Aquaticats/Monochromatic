# One reclaimer per physical index

Written 2026-11-04 on `fix/git-policy-cli-one-reclaimer-per-physical-index`,
from `main` at `05c16d73d2`.
A proposal for the owner's veto;
the pick and what was built are in "Pick" and "What was built".

## The race

A reclaimer of an index lock reads the marker,
finds the holder gone,
and then takes the lock aside by one rename
(`reclaim_if` in `package/git-policy/cli/src/native/landing/service_index_reclaim.rs`).
Two steps cannot be one:
when a second reclaimer removed the stale lock and placed its own live lock between the first
reclaimer's look and its rename,
the first reclaimer takes the live lock aside,
finds it live,
and puts it back by a hard link.
While the live lock is off its name,
a third program (Git) can create `index.lock`;
the hard link then fails,
and the live
holder's lock is stranded at its `.aside-` name.

The control `a_live_lock_taken_aside_by_a_late_reclaimer_is_never_open_to_a_third_program` in
`package/git-policy/cli/src/native/landing/service_index_reclaim_tests.rs` is red on `main` plus the `reclaim-aside` fault point:
the third program places its lock while the live lock is off its name,
and the put-back reports `File exists`.
It is marked `#[ignore]` so the branch stays green;
run it with `--run-ignored all`.

What the stranded holder does at its release (read from `service_index_hold.rs` and `service_index_ownership.rs`,
not run):
the ownership check answers `Replaced`,
so it installs nothing,
logs that another program replaced the lock,
and leaves the batch
open;
a restart then finds the live index is not the file the landing saw and replays nothing.
The commit stays landed and no staging is overwritten;
the commit's paths stay un-updated in the index.
A holder that passed its ownership check just before the rename aside installs its index while a third program holds the lock,
and one of the two index writes is then lost.

## Who reclaims

Three call sites reclaim,
not one:

- `remove_stale` (`service_land_finish.rs`,
  restart of a dropped or finishing batch);
- `try_acquire_on` (`service_index_lock.rs`,
  reached from the restart's `settle` and from `index_reset_support.rs`);
- `wait_and_place` (`service_index_attempts.rs`,
  the live landing).

One service serves one worktree's own index,
so an index inside the instance's Git directory has one reclaimer.
Two instances reclaim one physical index only when it is an alternate index (`GIT_INDEX_FILE`) shared by their worktrees,
or one instance's alternate index is another worktree's own index.

## Options

### A: refuse to finish an index outside the instance's own Git directory (the first pick, replaced; see "Pick")

Pros:
- no new lock,
  no new file;
- recovery no longer reaches a shared alternate index at all.

Cons:
- it covers `remove_stale` and `try_acquire` from `settle`,
  not `wait_and_place`,
  which a live landing on an alternate index
  runs;
  two live landings from two worktrees on one alternate index still race;
- extended to the live landing,
  a dead service's marked lock on an alternate index is never reclaimed,
  and the commit that meets
  it waits,
  then is withdrawn with the index in use (read,
  not run:
  `look_at_holder` and `classify_index_lock` end the wait
  at once for a dead pid file and at `LOCK_CEILING` for anything else);
  that is a failed commit on a path that succeeds today,
  against "Commits should never fail.";
- a restart no longer finishes an alternate index,
  which `SPEC.md` states it does today.

Visible result:
after a kill during a landing on an alternate index,
the commit's paths stay un-updated in that index.

### B: serialize the reclaim under a non-blocking exclusive `flock`

Every reclaimer takes `LOCK_EX | LOCK_NB` before it removes a stale lock;
the kernel drops the `flock` on death,
so no leftover
can block a reclaim,
and a reclaimer that does not get it backs off.
Precedent in the tree:
the sweep of a dead service's private copy and the conversion lock hold an exclusive `flock`.
Two variants,
B1 and B2,
differ in what the `flock` is taken on.

Pros of B (both variants):
- every reclaimer of one physical index serializes,
  whatever the number of worktrees and instances;
- the commit path still reclaims,
  so no commit fails for a stale lock;
- the hold lasts microseconds and leaves nothing behind.

Cons of B (both variants):
- a lock,
  against "Use as few locks as possible.";
  its reason is written beside it in the code and the lock inventory
  (`service_lock_inventory_tests.rs`) lists the file;
- `flock` is a hold of this kernel:
  reclaimers on different hosts over a shared filesystem are not serialized,
  and on a
  filesystem that gives no `flock` the stale lock is not removed at all (see "No `flock`");
- Git does not take it,
  but Git never reclaims,
  so it is not a party.

#### B1: the `flock` is on the stale lock file's own inode

The reclaimer opens the lock it will judge,
reads the marker from that descriptor,
takes the `flock` on that descriptor,
checks
that the name still refers to that inode,
and unlinks the name.
A second reclaimer that opened the same inode fails the `flock` and backs off.
The rename aside and the put-back by hard link are not needed any more,
because no other reclaimer can remove or replace the
file while the `flock` is held,
and a live landing cannot place a lock while the name is occupied.

Moving parts:
one descriptor,
one `flock`,
one inode comparison.
No sidecar name,
no directory hold,
no leftover.
The unit is the one file being reclaimed,
the smallest there is.
The physical index is identified by the inode,
so two spellings of one path (symlinks,
bind mounts) need nothing more.

Interleavings:
- two reclaimers:
  closed.
  The one that holds the `flock` removes;
  the other backs off,
  or opened the name after the removal and
  judges whatever is there (a live lock is never removed),
  or got the `flock` after the first finished and finds the name no
  longer refers to its inode.
- a reclaimer against a live landing:
  closed.
  A landing places a lock only where the name is absent,
  and the name stays occupied
  from the reclaimer's inode check to its unlink.
  A live lock found at the name is judged by its marker and left.
- a person's `rm index.lock` in the window:
  left,
  narrowed.
  Between the inode check and the unlink a person can remove the
  stale lock and Git can place its own,
  and the unlink then removes Git's lock.
  No system call removes a name only if it still
  names a given inode,
  so no variant closes it;
  the window is two system calls.

#### B2: the `flock` is on a sidecar file or on the lock's directory

The reclaimer takes the `flock` on `index.lock.reclaim` (or on the Git directory),
then runs today's sequence (rename aside,
look again,
remove or put back).

Moving parts:
the sidecar name or the directory descriptor,
the `flock`,
the rename aside,
the look again,
and the put-back by
hard link,
which still has its empty window against Git and its failure mode of stranding a lock.
The sidecar form leaves a file name behind ("Allow the rare leftover") and two spellings of one path make two sidecars;
the
directory form serializes the reclaim of every index in that Git directory.

Interleavings:
- two reclaimers:
  closed,
  when both take the same sidecar or directory.
- a reclaimer against a live landing:
  left.
  A live landing is not a party to the `flock`;
  the sequence that takes a lock aside
  and puts it back runs on,
  and a live lock taken aside in it is off its name for as long as the window lasts.
- a person's `rm index.lock` in the window:
  left,
  as in B1,
  and wider,
  since the window is the whole sequence.

### C: document the gap (the round 5 option B)

Pros:
no code.
Cons:
leaves the race open for two worktrees on one alternate index,
and a stranded lock that needs a person.

### Ranking

B1 > B2 > A > C.
B1 over B2 because it has fewer parts,
leaves no name,
identifies the physical index by its inode,
and closes the live-landing
interleaving that B2 leaves.
B2 over A because B2 serializes every reclaimer at every call site and no commit fails for a stale lock,
while A closes one
call site or fails commits.
A over C because A closes the restart route at the cost of the restart finishing an alternate index,
and C closes nothing.

### Pick

The coordinator picked B under the owner's delegation ("Pick options as if I had picked them and invite veto"),
replacing A,
and
then B1 for having fewer moving parts;
the owner's veto is open.
The reason for replacing A:
it covers only the restart,
and extended to the live landing it makes a stale lock on an alternate
index fail the commit at `LOCK_CEILING`,
against "Commits should never fail."

Note 2026-11-08T19:19:04-05:00:
the wait the live landing takes is `LANDING_LOCK_CEILING`,
one minute (`package/git-policy/cli/src/native/service/wait_ceilings.rs`),
so a commit that meets such a lock is withdrawn after one minute and not after ten,
and `LOCK_CEILING` keeps its ten minutes without any lock wait taking it.

## What stays open under every option

A lock removed by hand (a person running `rm`),
and a Git on another host sharing the filesystem,
are not parties to any
serialization this service can make.

## What was built

B1,
at all three reclaim sites,
in `package/git-policy/cli/src/native/landing/service_index_reclaim.rs`:
`reclaim_if` opens the lock with `O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC`,
reads the marker from that descriptor,
takes
`LOCK_EX | LOCK_NB` on it,
checks device and inode against the name,
and unlinks the name through `Platform::unlink`.
It answers `Removed`,
`Changed`,
`Busy`,
`Kept` or `Stuck`;
`remove_stale` ignores the answer,
`try_acquire_on` places again after
`Removed` and `Changed` and otherwise leaves the batch open,
and `wait_and_place` continues after `Removed` and `Changed`,
fails with `Unavailable` after `Stuck`,
and waits as for any held lock after `Busy` and `Kept`.
The lock is on the inventory in `source_guard/service_lock_inventory_tests.rs` with its reason above the call.

Controls in `service_index_reclaim_tests.rs`:
- red first (`dd37f0c462`),
  green now:
  `a_live_lock_found_by_a_late_reclaimer_is_never_open_to_a_third_program`;
- refused:
  `a_second_reclaimer_of_one_stale_lock_backs_off_while_the_first_holds_it`,
  `a_reclaimer_whose_judged_file_was_replaced_removes_nothing`,
  `a_lock_placed_after_the_stale_one_was_found_is_not_removed`,
  `a_foreign_lock_and_a_running_holders_lock_stay`,
  `a_landing_waits_for_the_reclaimer_that_holds_a_stale_lock`;
- allowed:
  `a_stale_lock_is_removed_and_leaves_no_file`,
  `a_reclaimer_that_backed_off_can_reclaim_once_the_hold_is_dropped`.

Each guard was shown to matter:
with the `flock` result ignored,
the controls for the backed-off reclaimer and the landing's
wait fail;
with the inode comparison always true,
the controls for a replaced file fail.

Added after the cross-vendor review of `9db8794047` (`/var/home/user/temp/agent/reviews/one-reclaimer-sol-1.txt`,
verdict
"correct after named fixes"):
the three fixes in the next sections.
The sweep of aside files that was added with them is removed
again (see the last section);
a second review's two other findings are fixed:
the notice for a stale lock that is not removed is
written from the marker of the file that was judged,
so a live replacement is never called gone,
and the controls for a busy
stale lock hand off between threads so that they always meet the busy case.

## No `flock`

The first build removed the stale lock anyway when the `flock` failed for a reason other than another reclaimer holding it.
The
review showed that this route deletes a live lock:
two reclaimers without the hold both pass the inode check,
one removes the
stale lock and a landing places its own,
and the other then unlinks the name,
which is the live lock;
Git can then create its own
lock beside a landing that has passed its ownership check.
Every error but `WouldBlock` took that route,
not only a file system
without `flock`;
an NFS client that emulates `flock` with byte-range locks can refuse a lock on a read-only descriptor.

The second build (the coordinator's pick under the owner's delegation,
veto open) never removed the lock when the `flock` could
not be taken (`Reclaimed::Unguarded`):
the landing waited to `LOCK_CEILING`,
withdrew,
and told the person to run `rm`.
The owner rejected that on 2026-11-04:
"Must work perfectly even when flock isn't available.
Do better."
and then
"I'm not banning flock.
I'm just saying our wrapper must work perfectly even when flock isn't there."
So the `flock` on the stale lock's own inode stays wherever it works,
and the case where it is not there gets a path that
needs no `flock` at all.

### Facts the design rests on

- Which file systems give no `flock`:
  the kernel calls `f_op->flock` when a file system defines one and otherwise takes a local
  lock itself (`flock` system call in `fs/locks.c`:
  `if (f_op->flock) ... else locks_lock_file_wait`),
  so a local file system
  always gives `flock`.
  Only file systems that define the operation can refuse:
  NFS (`nfs_flock` in `fs/nfs/file.c`,
  a byte-range
  emulation that needs a lock manager),
  9p (`v9fs_file_flock_dotl` in `fs/9p/vfs_file.c`,
  returns `ENOLCK` unless converted),
  CIFS (`cifs_flock` in `fs/smb/client/file.c`,
  `ENOLCK` or `EOPNOTSUPP`),
  and FUSE (`fuse_file_flock` in `fs/fuse/file.c`,
  which
  falls back to a local lock only when the server has no lock support,
  so a daemon that answers the lock request can refuse it;
  Ceph and others define one too).
  Fetched from the kernel tree on 2026-11-04.
  A fixture with a real file system that refuses `flock` needs a mount of one of these,
  which a rootless container cannot make,
  so the controls stand in the refusal through `Platform::flock`.
- Transient errors:
  the local `flock` can fail with `ENOMEM` when the lock record cannot be allocated (`fs/locks.c`);
  that says
  nothing about the file system,
  so it is treated as a hold by someone else (try again),
  never as "no `flock` here".
- The coordinator's reading that only reclaimers on this machine can race is wrong.
  `holder_state_in`
  (`service_index_identity.rs`) answers `Gone` for a marker whose boot ID differs from the reader's,
  so a reader on another
  host judges a lock of a holder on a third host stale,
  and two such readers over one shared file system race for the same file
  (a live holder on yet another host reads as gone too;
  that is the existing cross-host limit,
  not new).
  A lock in
  `/run/lock`,
  `/dev/shm` or an abstract socket is local to one kernel (and one mount or network namespace),
  so it would not
  keep those apart;
  a claim file on the repository's own file system does.

### The claim chain

Used only when the `flock` on the stale file answers that the file system gives none (`ENOLCK`,
`EOPNOTSUPP`,
`ENOSYS`,
`EINVAL`,
`EBADF`).
`EWOULDBLOCK`,
`EINTR` and `ENOMEM` are back-offs,
and any other error is `Unguarded` (see `The cross-vendor review of the claim chain`).
All claims are files beside the lock,
named `<lock>.cli-reclaim-<key>-<slot>`,
with a key made from the stale lock's marker (batch,
process,
birth identity with the bytes a file name cannot hold escaped,
which keeps the name within the limit of a file name beside a long lock name) and a slot number counted from zero.
The first design keyed on the device and inode of the
stale file;
the review found that two mounts of one remote tree can give one file different device numbers,
so two reclaimers would
place at different names.
The marker is the file's own content,
the process it names is gone and places no further lock with that
marker,
and so no later lock carries the key.
A claim holds the service marker of its claimer (batch zero,
process,
birth identity,
boot,
PID namespace),
so
`holder_state_in` judges a claimer exactly as it judges a lock holder.
It is made like the lock:
written to a staging file,
then placed by a rename that does not replace,
else a hard link (`place_marked`),
so a claim is never visible half written.

The reclaimer that judged the lock stale takes a claim like this:

1.  Try to place its claim at slot zero.
    If the name is free,
    the claim is placed.
    If it exists,
    read it:
    a claimer that is
    gone (or a file that holds no whole marker,
    which only a power loss or a stranger can leave,
    since placed claims are whole)
    is passed over and the next slot is tried;
    a claimer that runs or cannot be judged makes this reclaimer back off (`Busy`).
2.  After placing at slot n,
    read every slot below n again.
    All must exist and name a gone claimer;
    otherwise remove the own
    claim and back off.
    This is the step that makes a reclaimer's earlier reading (which may be arbitrarily old) harmless.
3.  Holding the claim,
    run the ordinary steps:
    check that the name still refers to the stale file (device and inode),
    unlink.
4.  When the name no longer refers to the stale file (removed by this reclaimer,
    or by someone else),
    remove every claim from
    zero to the own slot,
    and nothing else.
    When the unlink failed,
    remove only the own claim.

Why one claimer at a time holds while the stale file is at its name:
a dead claim is never removed by anyone but a claimer that has seen the stale file leave its name;
while the file is there,
claims only get added,
except that a claimer removes its own live claim when it backs off.
So a slot that one claimer read as
gone at some moment still exists,
unchanged,
for as long as the file is at its name,
and no claimer can be placed at a lower
slot behind it.
Two holders would need slots a and b with a below b;
b's step 2 reads slot a,
which is either not there
(then b backs off) or a's live claim (b backs off).
Two reclaimers placing at one slot are kept apart by the exclusive
placement.
A dead claimer's claim is neither judged nor removed by a race,
because nobody removes it before the stale file is
gone,
and after that nothing destructive is left to guard:
every later holder fails the check of the name.
That is also why the rule needs no birth time of the file:
an orphan claim left under the same key is a gone claim at a low slot and
is passed over.

Leftovers:
a claimer killed between the unlink and the removal of claims leaves `<lock>.cli-reclaim-*` files.
They are inert,
passed over by every later reclaim of a file with that key,
and removed by the first reclaim that finishes one.
To remove them by
hand,
once no git command and no commit of this service is running in the repository:

```sh
# claims are named <lock>.cli-reclaim-<device>-<inode>-<slot>; each holds one line starting "cli-git-service "
ls -l -- "$(git rev-parse --absolute-git-dir)"/index.lock.cli-reclaim-*
rm -- "$(git rev-parse --absolute-git-dir)"/index.lock.cli-reclaim-*
```

What this does not close,
with the schedule:
- The mixed case:
  reclaimer one's `flock` works and holds,
  reclaimer two's `flock` on the same file answers "not there".
  One holds the
  `flock` and no claim,
  the other a claim and no `flock`,
  and they do not see each other.
  It takes two mounts of one file system that
  differ in lock support,
  or a lock manager that answers `ENOLCK` to one call and grants the next.
  No call closes it without making
  every reclaimer take the claim chain (the reviewer's fix:
  drop the `flock` fast path,
  which the owner said to keep:
  "I'm not banning
  flock.");
  the errors are mapped so that a back-off errno (`EWOULDBLOCK`,
  `EINTR`,
  `ENOMEM`) never takes the claim path and an error
  that does not say "no `flock` here" (`EIO`,
  a timeout) leaves the lock and says so.
- A claimer that is alive on another host reads as gone (boot ID differs),
  as a lock holder does:
  the existing cross-host limit.
  The
  same reading makes a reclaimer on another host judge a live claim gone,
  and then two reclaimers can both pass the name check.
  The
  alternative,
  reading an incomparable identity as unknown,
  would stop recovery after a reboot of a shared file system,
  which
  `holder_state_in` is written to allow;
  not changed here,
  and listed as open.
- A claimer that is alive but cannot be judged (another PID namespace,
  an unreadable `/proc`) makes reclaimers back off to the
  ceiling of the wait,
  as for any holder that cannot be judged;
  no commit waits without bound.
- A person's `rm index.lock` between the name check and the unlink,
  as in every variant.
- No space for the claim file (the same disk that would refuse the landing's own lock):
  the stale lock stays and the notice of
  `Unguarded` is written,
  now only for that case and for a refusal of the placement itself.

Alternatives considered:
- Remove anyway,
  as the first build did.
  Rejected:
  the interleaving above deletes a live lock.
- Never remove,
  wait for a person (the second build).
  Rejected by the owner.
- Fall back to the old rename aside and put-back.
  Rejected:
  it brings back the empty-name window in which Git takes the name and
  the live lock is stranded.
- A `flock` on a file in `/run/lock` or `/dev/shm`.
  Rejected:
  it is still `flock`,
  it is local to one kernel and one mount
  namespace while stale locks can be judged from other hosts,
  permissions and the sticky bit decide which accounts can open it,
  and it leaves files that nothing owns.
- An abstract Unix socket bound under a name made from the stale file's identity (no file,
  no `flock`,
  released on death).
  Rejected for the same scope reason:
  one network namespace of one kernel.
- One claim file per stale file,
  with a dead claimer's claim removed by name.
  Rejected:
  two readers of the dead claim,
  one
  removes it and places a live one,
  the other removes that;
  the race moves one level down and no Linux call removes a name
  only if it still names a given file.
- The same race broken by recursing the reclaim onto the dead claim (a claim for the claim).
  Not taken:
  the chain needs no
  recursion and no depth bound.
- Exchange the names with `RENAME_EXCHANGE` to take over.
  Rejected:
  while a live lock is exchanged out its holder's install
  renames whatever stands at the lock name.
- A claim file kept after the fact (never removed,
  one per reclaim).
  Rejected:
  unbounded leftovers.

Ranking:
the claim chain after the `flock` > never removing (the second build) > the fall back to a rename aside > removing anyway.

### What was built for the claim chain

`package/git-policy/cli/src/native/landing/service_index_claim.rs` (`take_claim`,
`ClaimHold`),
called from `reclaim_if` in
`service_index_reclaim.rs` when `hold` answers `Unavailable`.
`hold` answers `Busy` for `EWOULDBLOCK`,
`EINTR` and `ENOMEM`,
and
`Unavailable` for every other error.
The controls are in `service_index_claim_tests.rs`;
the controls that encoded the second
build's behaviour (`a_failed_flock_never_has_the_stale_lock_removed`,
and the landing that waited to its ceiling) are inverted
there (`a_stale_lock_is_removed_where_flock_is_not_available`,
`a_landing_takes_a_stale_lock_where_flock_is_not_available_without_waiting`),
and the ceiling and notice controls now refuse the placement of claims as well,
since `Unguarded` is what remains when neither
the `flock` nor a claim is possible.

### The cross-vendor review of the claim chain

`/var/home/user/temp/agent/reviews/claim-chain-sol-1.txt` (2026-11-04):
no break of the core interleavings;
a backed-off claim,
a slot read as gone long before,
and the cleanup against a reused inode were each attacked and held.
Findings folded in:
the key
from the device and inode (above);
an unreadable claim must not read as gone (it never did:
only a successful read of text that is
no marker does;
a claim is now mode 0644 so another account can read it);
an ambiguous placement answer (the claim carries a random
token,
and a file at the slot with this attempt's text is its own);
the mixed case and the remote live claimer (above,
kept open).

## Claims on every file system, and the machine ID (2026-11-04)

Two owner-side decisions followed the first claim chain.

### Claim files always

The owner picked "Claim files always (Recommended)" on 2026-11-04 through the question tool:
"Every wrapper uses the claim files,
on every
file system;
the `flock` path for this is retired.
Pro:
one mechanism,
race closed."
So `reclaim_if` always takes its place in the claim
chain and never takes the `flock` on the stale lock's inode;
`Platform::flock`,
`flock_exclusive` and the routing by `flock` error are
removed,
and `service_index_reclaim.rs` leaves the lock inventory because it holds no lock primitive (the inventory test would report
a stale entry).
This closes the mixed case that the first chain left open (one reclaimer on the `flock`,
another on a claim):
there is
one path.
The sections that describe the `flock` path (B1,
"What was built",
the routing by errno) are history.

What the routing keeps:
an interrupted or out-of-memory placement of a claim (`EINTR`,
`ENOMEM`,
`EWOULDBLOCK`) is a back-off and not a
refusal;
a refusal of the placement is `Unguarded`,
as before;
a claim whose maker is on another machine is `Foreign`.

### The machine ID in the marker

The review of the first chain,
and the coordinator's follow-up,
found that `holder_state_in` answered `Gone` for any marker whose boot ID
differed from the reader's.
Over a repository on a shared file system that makes a live service on another machine look gone,
and its lock
(or claim) is removed.
The rule now is the one the owner kept for the claims of work (`claim_time.rs`;
"same machine id plus a different
boot id clears liveness only"):

- a marker's identity ends `#<boot>#<namespace>#<machine>` when the host has a valid machine ID (32 lowercase hexadecimal digits,
  not all
  zero;
  `/etc/machine-id`,
  else `/var/lib/dbus/machine-id`);
  a writer without a valid one writes none and is read as before;
- same machine ID and a different boot ID:
  gone (a reboot ends every process);
- a machine ID that is not the reader's,
  is not valid,
  or a reader with none:
  unknown,
  never gone;
  the lock or claim stays,
  and the
  landing's notice (`foreign_words`) names the file,
  the batch and the process and gives the `rm`;
- a marker with no machine ID (an earlier release's,
  or a writer without one):
  today's reading.

An earlier release reading a new marker sees a namespace field that carries `#<machine>` and answers unknown when the boot is the same,
which
keeps the lock;
it compares the boot first,
so on another boot (another machine over a shared file system) it still answers gone,
as it
always did.
Every caller of the boot comparison was searched for:
`holder_state_in`
(`service_index_identity.rs`) is the only one for markers and their claims;
the claims of work (`claim_time.rs`,
`work_item_recovery.rs`) already compare the machine ID;
the reference-lock judge reads the boot time by the wall clock only.
Machines cloned with one machine ID cannot be told apart and are judged as one machine.

Release overlap:
closed by the marker format,
see "The third cross-vendor review" (an earlier release reads no marker in a lock of
this release,
and this release leaves every lock in the earlier format).


Strictness of the machine ID:
`is_machine_id` also governs the claims of work (`claim_time.rs`),
so an uppercase or all-zero machine ID there
now reads as none.
That is safe:
`machine-id(5)` specifies 32 lowercase hexadecimal digits and says an all-zero value is not valid,
so no
real host is affected,
and a reading with no machine ID falls back to the wall clock,
as before for hosts without one.

Where this still does not hold:
a service on another machine whose host has no valid machine ID writes a marker with none,
and a reader
that also has none keeps the earlier reading,
so between two such hosts over a shared file system a live holder can still read as gone.
No Linux call can tell a process of another machine from a gone one by the marker alone;
the machine ID is the only evidence.

## The third cross-vendor review (2026-11-04)

`/var/home/user/temp/agent/reviews/claims-always-sol-3.txt`,
verdict "Merge after named fixes".
What was done for each finding.

### A: markers without a machine ID

The review's schedule:
a live service on machine A writes a marker with no machine ID (an earlier release,
or a host with no valid one);
machine B reads another boot as gone and removes the live lock.
Now:
a marker that names no process is never gone;
a marker that names no
machine ID and another boot is unknown to a reader that has a machine ID (it cannot be told from another machine),
and keeps the earlier
reading only for a reader that has none either (nothing better exists there).
The cost:
after the upgrade,
a stale lock that an older
release's killed service left (and which names another boot) stays,
with a notice and the `rm`,
instead of being removed.

### B: earlier releases that reclaim under a `flock`

The review's schedule:
an earlier release takes the `flock` and this release a claim,
both pass the name check,
one unlinks,
a landing
places its lock,
the other unlinks it.
No call lets a claim reclaimer and a `flock` reclaimer exclude each other without this release
taking the `flock` too (a second lock,
and the owner retired it) or waiting without bound for an earlier release's reclaimer it cannot see.
The transition rule that needs neither:
each release reclaims only the locks of its own format,
and the formats are told apart by the
marker itself.
A marker of this release reads `cli-git-service v2 <batch> <process> <identity>`;
an earlier release's parser takes `v2`
for a batch number,
fails,
and reads no marker,
so it leaves the lock (`Kept`) and its landing waits as for a foreign lock.
This release
in turn leaves every stale lock in the earlier format (answer Foreign,
with words that say why).
So no lock can be reclaimed by both
mechanisms,
on any worktree or alias of one physical index,
with no prerequisite about which services run.
The claim names also changed
(a digest in place of the escaped identity);
an earlier release's claim files are never read by this one and the reverse,
which is now
harmless for the same reason.
Cost:
a stale lock in the earlier format is removed by hand once,
and after a downgrade the locks of
this release are the person's to remove.
Owner veto is open on this trade (a stale lock from an older release that needs one `rm`
against a possible second unlink of a live lock).

### C: claims that cannot be judged

A claim whose maker is in another PID namespace (or that cannot be read) used to read as a running claimer and so as passing contention.
Now it is `Slot::Unjudged`,
the answer `Reclaimed::Unjudged { file, marked }`,
and `unjudged_words` names the claim,
says it keeps the stale
lock from being reclaimed,
and gives the quoted `rm`.
It is never treated as gone.
The landing still waits only to its ceiling.

### D: an abandoned ambiguous claim

A placement that took effect but answered with an error,
followed by a failed read of the claim,
could leave a live-looking claim of the
service's own process that no later attempt recognised (each attempt writes a new token).
The attempt now records the path and text per
thread,
and the next attempt of that thread removes the claim if it still holds exactly that text.
Per thread,
because the same process
number serves concurrent attempts of other threads (the service reclaims on one thread;
a second thread's attempt is never taken for an
abandoned one).
A claim whose recorder never returns stays until the process ends,
which then reads as gone.

### The limits assessment

- Cloned machine IDs:
  unchanged;
  deployment must regenerate them.
- An older reader deleting a new marker on another boot:
  closed by the format (it reads no marker).
- Holderless keys:
  a marker that names no process is never judged stale now,
  so no claim is ever made for one;
  the key function keeps a
  defensive branch for it.
- Long lock names:
  the claim name is a digest of fixed length (the first sixteen bytes of SHA-256),
  so a claim is placed beside any lock
  whose own name and staging name fit a file name;
  the control places a claim beside a lock with a 150-byte name.
- Work-claim stamps:
  the first fix made `decode` return nothing for a whole stamp whose machine ID was uppercase or all zero,
  which
  moves such a claim to wall-clock expiry or drops it;
  that was wrong.
  `decode` again accepts what an earlier release wrote,
  and only the
  host's own reading is strict (an uppercase `/etc/machine-id` reads as none,
  so the wall clock judges).
  The control goes through the
  claim-name parser the recovery uses,
  and was shown red against the strict decode.
- `rm` in the words:
  the path is single-quoted for a shell and `rm --` is used.

## A busy stale lock

A `Busy` answer leaves no event in the directory when the other reclaimer lets go without unlinking (it died,
or its file was
already replaced),
so the wake of the wait on the directory would come only with the notice timer.
The wait now looks again after
`POLL_PAUSE` while the last answer was `Busy`,
inside the same acquisition and under the same ceiling.
The review also asked whether the holder classifier ends the wait at once for a stale marker:
it does not for a service marker,
which has no pid file,
and the control `a_landing_takes_the_stale_lock_when_the_other_reclaimers_hold_is_dropped` shows the
acquisition succeeding with no dead-holder announcement.

## The control for the old aside window

`a_reclaimer_that_finds_a_live_lock_leaves_no_event_on_the_lock_name` watches the directory with inotify while a reclaimer works
on a live lock,
and fails on any rename,
removal or creation of the lock's name,
so it needs no hook at a point inside the
reclaimer.
Run against the code of `dd37f0c462` (a throwaway worktree,
since removed) it fails with the rename of `index.lock`
to its `.aside-` name in the events;
against the current code it passes.

## Aside files an earlier release left

Release 0.5.28 took a live lock aside by a rename and put it back by a hard link;
when a third program took the name in between,
the file stayed at `<lock>.aside-<process>-<time>` beside the lock (for example `index.lock.aside-4242-18e4cb0dfc4e8cad` in the
Git directory,
or beside an alternate index).
This release makes none,
and **nothing removes the ones already there**.

An automatic sweep was built and removed again (the coordinator's pick under the owner's delegation,
veto open;
the second
cross-vendor review,
`/var/home/user/temp/agent/reviews/one-reclaimer-sol-2.txt`):
a scan of the processes this process can see
does not prove that no process holds the file (another account,
another PID namespace),
and deleting the only name of a live
holder's lock loses it;
the name prefix does not prove the content is a service marker,
since the old reclaim could take a native
Git lock aside,
whose bytes are a pending index;
and the sweep ran while the landing held the lock.
The owner's "Allow the rare
leftover" covers the files,
and the risk of deleting a live lock outweighs the tidiness.

To remove them,
once no git command and no commit of this service is running in this repository,
look at them first and then
remove exactly what is listed:

```sh
# list, then read the first line of each: a service lock starts with "cli-git-service "
ls -l -- "$(git rev-parse --absolute-git-dir)"/index.lock.aside-*
rm -- "$(git rev-parse --absolute-git-dir)"/index.lock.aside-*
```

A file that does not start with `cli-git-service ` may be a native Git lock holding an index its Git never installed;
keep it
until you know.
For an alternate index (`GIT_INDEX_FILE`),
use the directory of that index instead of the Git directory.
