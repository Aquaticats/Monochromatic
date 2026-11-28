# Two forces at a ZFS removal: the act past a holder, the command that ends its own ancestor, and the waits

Status:
a design written by an agent,
**unapproved and non-final**,
with measurements from a guest of its own.
It designs the two things the owner's answer of 2026-11-28 named and did not decide,
and it answers the two questions those things rest on.
The owner declares it complete,
and every question it ends with is the owner's to answer.
No release code and no test of the release was changed by the round that wrote it.

## What the owner's answer left to be designed

On 2026-11-28 the owner decided,
in these words,
"--force overrides anything and delete and destroy no matter what.",
and,
asked how far that reaches,
"Two force does everything.
One force - you pick."
The record's own reading is that `git worktree remove --force --force <slot>` passes every refusal the removal has
(`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
"What force does on a ZFS removal, two refusals without it, and the foundation pick").

That record then names what the answer did not decide:

> Two things the texts named and did not say how they work,
> so they are not yet decided and come back to the owner as a design:
> how a command that ends its own ancestors still finishes its removal,
> and by what act two forces get past other processes that hold the clone
> while the forced arm's waits have no measured value.

This document is that design.
It answers four questions,
in this order:

1. By what act two forces get past another process that holds the clone.
2. How a command that must end a process of its own session,
   or one of its own ancestors,
   still finishes its removal.
3. Under the service manager,
   where one force is refused today,
   what two forces do,
   and what the reason for that refusal still protects.
4. The values the forced arm's waits ship with none of.

The first two are the record's own words.
The third follows from "two force does everything",
because the service-manager refusal is the one refusal whose reason is not a state of the clone but an inability to tell
the requester apart.
The fourth is the condition the accepted forced design set for switching the arm on
(`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`,
"The forced removal design").

Where the code stands,
in `package/git-policy/cli/SPEC.md`,
"Removal of a ZFS worktree clone":
"`--force` is refused in every spelling".
The forced arm itself is built and reached by no command
(`package/git-policy/cli/SPEC.md`,
"The forced arm, built and reachable from no command";
`src/native/worktree_clone_forced_removal/`).

What the answer amends,
in the record's own list,
is three accepted records for two forces:
the lock design's sentence that a removal must not release a lock it cannot attribute to this add,
the answer of 2026-11-26 that named three refusals "and no more",
and the forced design's set of processes a forced removal never ends.
This document designs only what that amendment needs.
It changes no record and proposes no decision:
the questions at its end are the choices,
and each one says what a yes costs.

## What this design read, and what it ran

Read whole:

- `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
  the accepted forced design:
  its ways out of a namespace whose mounts are shared,
  its preflight,
  its signal stage,
  its act's liveness,
  its constants with no value,
  and its measurements owed.
- `doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
  the channel,
  the reach,
  the never-end set,
  the signal,
  the authority's duration and the script unit.
- `doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`,
  the accepted design,
  the two refusals the built arm keeps,
  and the refusal under the service manager.
- `package/git-policy/cli/SPEC.md`,
  "Removal of a ZFS worktree clone",
  "The cancel command" and "The forced arm, built and reachable from no command".
- `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
  the holder arms already run:
  item 4's busy holder,
  items 3 and 15's readers,
  the Tier 0 and Tier 1 rounds of 2026-11-23,
  the fourth round's six locked-holder shapes of 2026-11-24,
  and the stop of a locked unit's own user unit of 2026-11-27.
- `doc/planning/cli-git-zfs-step-22-guest-half-measurements.md`,
  "The forced arm's three waits" and "The values this measurement would support".
- The release code under `src/native/worktree_clone_forced_removal/`,
  every file,
  and of `src/native/process/` the files the arm calls:
  `member_signalled_through_its_pidfd.rs`,
  `process_use.rs`,
  `process_tree.rs`,
  `a_locked_holders_own_unit.rs` and
  `a_service_unit_stopped_through_the_accounts_manager.rs`.
- `package/git-policy/cli/e2e/README.md`,
  "The forced act's guest arms" and "Guest work that runs longer than the agent's safe length",
  and the forced act's driver `e2e/forced-act-guest.ts`,
  whose guest steps this round borrowed.

Run:
one guest of this round's own,
named below,
through `mvm --backend libvirt`,
with the probe program of this round;
its arms and their logs are named in "The guest, its kernel and the fixture" and in "Revisions".
Nothing was run on this machine:
no mount,
no `setns`,
no `umount2`,
no signal to a process this round did not start,
no `systemctl`,
no ZFS command,
and no command opened `/dev/zfs` here.
No service manager was started on this machine.

## The guest, its kernel and the fixture

One guest,
`rm-g-holders`,
made through the worktree's own `mvm`
(`mise run //package/mvm/cli:build`,
then `mvm --backend libvirt create`),
from the cached Ubuntu image `template-ubuntu.qcow2`,
nothing downloaded to make it,
and destroyed through `mvm` when the round's arms ended:
`mvm --backend libvirt list` no longer names it.
Its domain file carried `freePageReporting='on'` when it was created
(issue 6655).

What it booted and ran:

```text
# guest-prepare-install.log
prepared uname=6.8.0-147-generic zfs=2.2.2-0ubuntu9.5 python=Python 3.12.3
```

So the guest is Ubuntu 24.04.5 on kernel `6.8.0-147-generic`,
with OpenZFS 2.2.2 from the distribution's own packages,
module and userspace the same,
Python 3.12.3,
and systemd 255.
This machine's kernel is 7.3 and this machine's systemd is 262,
which is what the limits below name.

The fixture is one file-backed pool of 2 GiB and four datasets made by the program:

- `rmg/work`, mounted at `/root/rmg-fixture/work`, which stands for the checkout,
- `rmg/work/slot`, mounted inside it, which stands for the clone at a slot,
- `rmg/work/control` and later `rmg/work/fresh`,
  datasets nothing holds, which the arms destroy beside the clone,
- and a file-backed pool image at `/root/rmg-fixture/pool.img`.

The program is `probe-holders-and-stops.py`,
run as root in the guest,
sha256 `f986e99d8636ad115b01b4f41d54f807bd63213ee3db6112ec32c77845eaf31f`,
one arm per invocation,
each arm printing one line per fact to a log the driver pulled back.
The driver is `run-guest-arms.ts`,
which names the package's own guest steps
(`bin/mvm-guest-host-command.ts`,
`bin/mvm-guest-script.ts`,
`bin/mvm-guest-detached-script.ts`)
and runs every arm as a detached script with a bound of 900 s,
because a guest command bounded above five minutes runs detached
(`package/git-policy/cli/e2e/README.md`,
"Guest work that runs longer than the agent's safe length").
No arm reached its bound;
the longest was about 20 s.

The calls every arm makes are these,
and each arm's own block below quotes the ones it adds:

```python
# the probe's own calls, quoted from probe-holders-and-stops.py
LIB_C = ctypes.CDLL("libc.so.6", use_errno=True)
CLONE_NEWNS = 0x00020000; MNT_FORCE = 1; MNT_DETACH = 2
# a holder: a namespace of its own, a user namespace of its own, a working directory, or an open file
subprocess.Popen(["unshare", "--mount", "--propagation", "private", "--", "sleep", "600"])
subprocess.Popen(["unshare", "--user", "--map-root-user", "--mount", "--", "sleep", "600"])
subprocess.Popen(["sh", "-c", f"cd {SLOT} && exec sleep 600"])
# a single-threaded child enters one holder's namespace and runs one call there
descriptor = os.open(f"/proc/{pid}/ns/mnt", os.O_RDONLY); LIB_C.setns(descriptor, CLONE_NEWNS)
LIB_C.umount2(SLOT.encode(), 0)          # or MNT_FORCE, or MNT_DETACH
# the ending the forced arm is built to make, through a pin
pidfd = os.pidfd_open(pid); signal.pidfd_send_signal(pidfd, signal.SIGTERM)
wait_exit(pidfd, bound)                  # poll(POLLIN) on the pidfd
signal.pidfd_send_signal(pidfd, signal.SIGKILL)
# the destroy, and the same destroy of a dataset nothing holds beside it
subprocess.run(["zfs", "destroy", "rmg/work/slot"])
```

One reading needs its own sentence,
because it changes what every other reading means:
`zfs destroy` of a mounted dataset first tries to unmount it in the caller's namespace.
A refusal that says `dataset is busy` is therefore the dataset's own answer that a mount still holds it,
and a `gone` is the CLI's own unmount of whatever stood in its namespace plus the kernel's destroy.
Where a copy stands in another namespace,
the CLI cannot unmount it and prints
`cannot unmount '...': pool or dataset is busy`,
which is the reading the arms below quote.

## Question 1: by what act two forces get past a process that holds the clone

### The state each candidate starts from

The clone is a ZFS dataset mounted at its slot.
Another process holds it in one of two ways:

- a mount holder has a mount of the dataset in a mount namespace of its own,
  made by the kernel when that namespace was copied from one where the clone was already mounted,
  or by the holder mounting it itself;
- a path holder holds no mount of its own and references the mounted one:
  its working directory,
  an open file,
  its running executable,
  a mapped file,
  its root,
  a thread's working directory,
  or a unix socket bound inside the clone.

The second kind is what makes a plain unmount refuse `EBUSY`
(`fs/namespace.c` at `v7.3`,
F17 of the forced design),
and OpenZFS adds the other half:
the head destroy refuses while the dataset still has a reference from a mount,
whoever's namespace that mount is in
(F20 to F22).

### Candidate 1: end the holder

How it works.
The act pins the task it means to end and signals through the pin, never through the number:
`pidfd_open`,
the start time read before and after the open,
then `pidfd_send_signal`,
which is what `blocker_end.rs` builds
(`src/native/worktree_clone_forced_removal/blocker_end.rs`).
It asks the holder out with `SIGTERM`,
waits,
and sends `SIGKILL` to a survivor.
Where the holder is a mount namespace whose only task is the holder's,
ending that task frees the namespace,
and freeing a mount namespace detaches its whole mount tree
(F19),
so the copy goes with it and the dataset's reference goes too.
Where the holder is a path holder,
the end is the only thing that can release a working directory, an open file, a mapped file or a bound socket.

What it changes for good:
the holder's work is gone,
and with it whatever it had not saved.
No mount of the act's own changes,
no sharing changes,
and no file is written.

What the guest answered,
in `arm-end-holder.log`,
for the four shapes:

```text
# arm-end-holder.log
A1 kind=namespace-private holder=pid=2349 ppid=2262 session=2259 starttime=52497 ns=mnt:[4026532355] rows=1
A1 kind=namespace-private end term_ms=0.274 left_after_term=True kill_ms=not-sent
A1 kind=namespace-private holder-alive-after=False
A1 kind=namespace-private our-unmount accepted
STATE A1-namespace-private clone-destroy gone rc=0 said=
A1 kind=namespace-userns holder=pid=2459 ppid=2262 session=2259 starttime=52566 ns=mnt:[4026532356] rows=1
A1 kind=namespace-userns end term_ms=0.206 left_after_term=True kill_ms=not-sent
A1 kind=namespace-userns our-unmount accepted
STATE A1-namespace-userns clone-destroy gone rc=0 said=
A1 kind=cwd holder=pid=2484 ppid=2262 session=2259 starttime=52636 ns=mnt:[4026531841] rows=1
A1 kind=cwd end term_ms=0.221 left_after_term=True kill_ms=not-sent
A1 kind=cwd our-unmount accepted
STATE A1-cwd clone-destroy gone rc=0 said=
A1 kind=open-file holder=pid=2510 ppid=2262 session=2259 starttime=52635 rows=1
A1 kind=open-file end term_ms=0.564 left_after_term=True kill_ms=not-sent
A1 kind=open-file our-unmount accepted
STATE A1-open-file clone-destroy gone rc=0 said=
```

The label `left_after_term=True` is the wait's answer that the holder left before the bound,
and `kill_ms=not-sent` is the stage not sending `SIGKILL` at all.
What each line proves:

- a holder with a copy in a namespace of its own (mount id `mnt:[4026532355]`, ours `mnt:[4026531841]`)
  left in 0.274 ms and its copy went with its namespace,
  so the act's own unmount then answered `accepted` and the destroy `gone`;
- a holder whose mount namespace is owned by its own user namespace
  (`mnt:[4026532356]`, the shape bubblewrap and rootless containers make)
  left in 0.206 ms,
  and its locked copy went with its namespace:
  this is the measurement that matters for two forces,
  because candidate 2 cannot unmount that copy at all (its own arm below answers `EINVAL`);
- a path holder with its working directory inside the clone left in 0.221 ms,
  and one with an open file there in 0.564 ms,
  and each was the only thing that made a plain unmount refuse.

What a person loses:
the ended task's unsaved work,
and nothing else.
A job that flushes and exits loses nothing;
a job that ignores the ask loses whatever it held at the `SIGKILL`.

### Candidate 2: unmount inside the holder's namespace without ending it

How it works.
The act forks one single-threaded child per holder namespace,
enters it with `setns(CLONE_NEWNS)`,
and unmounts the copy there
(`src/native/worktree_clone_forced_removal/mount_namespace_child.rs`,
`the_unmount_stage.rs`).
`setns` refuses a multithreaded caller with `EINVAL`,
which is why the child is single-threaded
(F15).
The unmount names the copy by a descriptor,
never by a path resolved after the proof:
way 8 opens the add's recorded handle on the checkout's mount and unmounts the topmost mount at that dentry,
and way 7 makes the parent stop propagating,
moves the copy onto a short-lived `0700` directory in the checkout,
and unmounts it there.
What this candidate adds over candidate 1 is that the holder keeps running:
its namespace loses the copy and nothing else,
which is the promise the accepted design makes to a running service.

What it changes for good:
the holder's copy is gone,
and in way 7 the checkout's mount in that namespace stops propagating,
which no later share restores exactly (L15 of the forced design);
one directory appears in the checkout and is removed after the unmount.

What the guest answered,
in `arm-unmount-in-namespace.log`:

```text
# arm-unmount-in-namespace.log
A2 kind=namespace-private holder=... ns=mnt:[4026532355]
   rows=['387 385 0:48 / /root/rmg-fixture/work/slot ... rmg/work/slot ...']
A2 kind=namespace-private inside-the-holders-namespace umount2={'umount2': 'accepted'}
A2 kind=namespace-private after rows-in-that-namespace={'rows': 0}
A2 kind=namespace-private holder-still-alive=True
A2 kind=namespace-private our-unmount accepted
STATE A2-namespace-private clone-destroy gone rc=0 said=
A2 kind=namespace-userns holder=... ns=mnt:[4026532355] rows=['378 377 0:47 / /root/rmg-fixture/work/slot ...']
A2 kind=namespace-userns inside-the-holders-namespace umount2={'umount2': 'EINVAL'}
A2 kind=namespace-userns after rows-in-that-namespace={'rows': 1}
A2 kind=namespace-userns holder-still-alive=True
A2 kind=namespace-userns our-unmount accepted
STATE A2-namespace-userns clone-destroy refused rc=1 said=cannot destroy 'rmg/work/slot': dataset is busy
A2 kind=cwd holder=... ns=mnt:[4026531841] rows=['323 88 0:47 / /root/rmg-fixture/work/slot ... shared:308 ...']
A2 kind=cwd inside-the-holders-namespace umount2={'umount2': 'EBUSY'}
A2 kind=cwd after rows-in-that-namespace={'rows': 1}
A2 kind=cwd holder-still-alive=True
A2 kind=cwd our-unmount refused errno=EBUSY
STATE A2-cwd clone-destroy refused rc=1 said=cannot unmount '/root/rmg-fixture/work/slot': pool or dataset is busy
```

What each line proves:

- a namespace of the holder's own with nothing else in it loses the copy and keeps running:
  the unmount is `accepted`, the namespace's table holds no row for the clone, the holder is alive,
  the act's own unmount then succeeds, and the destroy answers `gone`.
  This is the accepted design's promise, measured again on this round's guest;
- a locked copy refuses the unmount with `EINVAL`,
  and the clone stays busy with the holder alive and not a single task of it ended:
  no unmount, forced or lazy, reaches a locked copy (its own arm below answers `EINVAL` for both flags);
- a path holder's reference refuses the unmount with `EBUSY`,
  and there the act's own unmount is refused too, because the holder is in the act's own namespace.

What a person loses:
nothing of the holder's own state,
except what way 7 changes about propagation,
and the copy the holder was using:
a job whose working directory was inside it keeps a stale view,
and its reads and writes after the unmount answer `ENOENT` or land in the underlay.

### Candidate 3: a forced unmount

How it works.
`umount2(path, MNT_FORCE)` asks the file system to give the mount up even while other references stand.
It changes no sharing and writes no directory,
so it would be the smallest act of all if the kernel and OpenZFS honoured it.

What the guest answered,
in `arm-forced-with-a-reference.log`,
where a task's own working directory is the reference,
which is the only state in which the flag can differ from a plain unmount:

```text
# arm-forced-with-a-reference.log
A5 flag=MNT_FORCE holder=... rows=['220 88 0:47 / /root/rmg-fixture/work/slot ... shared:299 ...'] same-ns=True
A5 flag=MNT_FORCE answered={'umount2': 'EBUSY'}
A5 flag=MNT_FORCE holder-alive=True rows-after=['220 88 0:47 / /root/rmg-fixture/work/slot ...']
A5 flag=MNT_FORCE control-destroy-made-after-the-copy rc=0 said=
A5 flag=MNT_FORCE clone-destroy refused rc=1 said=cannot unmount '/root/rmg-fixture/work/slot': pool or dataset is busy
A5 flag=MNT_FORCE after-the-holder-left clone-destroy gone rc=0 said=
A5 locked flag=MNT_FORCE answered={'umount2': 'EINVAL'}
A5 locked flag=MNT_FORCE holder-alive=True
```

The control line is a dataset made after the copy stood,
so no copy of it exists anywhere:
it is destroyed in the same moment,
which shows the destroy path works while the clone's own refusal is the clone's.

What it proves:
`MNT_FORCE` does not pass a live reference.
The kernel's `EBUSY` is the same one a plain unmount gives,
the mount stays in the table,
the holder stays alive,
and the clone stays busy.
On a locked copy the flag answers `EINVAL`, like every other unmount there.
So this candidate is not an act for either of the two states that refuse a plain unmount.

What it changes for good:
nothing, when it refuses.
Where a file system does honour the flag,
it drops the mount's state without asking the jobs that used it,
which for a clone would be the same loss as a lazy detach with none of the release.

### Candidate 4: a lazy unmount

How it works.
`umount2(path, MNT_DETACH)` skips the `EBUSY` check and detaches the mount from its namespace at once.
The mount object lives on while any reference remains
(F18),
and OpenZFS releases the dataset's long hold only when the superblock is destroyed,
which is when the last reference goes
(F20 to F22).

What the guest answered,
in the same log:

```text
# arm-forced-with-a-reference.log
A5 flag=MNT_DETACH answered={'umount2': 'accepted'}
A5 flag=MNT_DETACH holder-alive=True rows-after=[]
A5 flag=MNT_DETACH control-destroy-made-after-the-copy rc=0 said=
A5 flag=MNT_DETACH clone-destroy refused rc=1 said=cannot destroy 'rmg/work/slot': dataset is busy
A5 flag=MNT_DETACH after-the-holder-left clone-destroy gone rc=0 said=
A5 locked flag=MNT_DETACH answered={'umount2': 'EINVAL'}
```

and,
where the namespace holds nothing else and the act's own mount was already taken out,
in `arm-forced-and-lazy.log`:

```text
# arm-forced-and-lazy.log
A3 kind=namespace-private flag=MNT_DETACH our-unmount-first accepted
A3 kind=namespace-private flag=MNT_DETACH answered={'umount2': 'accepted'}
A3 kind=namespace-private flag=MNT_DETACH after-in-that-namespace={'rows': 0}
A3 kind=namespace-private flag=MNT_DETACH holder-alive=True
A3 kind=namespace-private flag=MNT_DETACH clone-destroy gone rc=0 said=
```

What each line proves:

- the call answers `accepted`, always, whatever stands against it.
  That answer says nothing at all about the dataset;
- with a live reference inside,
  the mount leaves the table,
  the job keeps working through the reference,
  and the clone still refuses the destroy, until the job is gone;
- where nothing referenced it, the detach did release the dataset.
  That case is the one a plain unmount would also have taken,
  which is why the first reading is the one that decides this candidate.

What it changes for good:
the mount is detached, so nothing new can reach it by path,
and the jobs that held it are stranded on a mount no table shows.
A person looking at their own `findmnt` sees the folder empty while the job still writes into the clone.

### Candidate 5: destroy the clone while a copy stands

How it works.
The head destroy sends the destroy request and the kernel answers whether it may.
OpenZFS refuses `EBUSY` unless the dataset's long holds equal the count it expects
(F21),
so one standing copy anywhere is enough.
This candidate is not an act of its own:
it is what all the others are for,
and what it measures is the floor they must reach.

What the guest answered,
in `arm-destroy-while-held.log`:

```text
# arm-destroy-while-held.log
A4 copy-stands holder=... ns=mnt:[4026532355] rows=1
A4 our-unmount accepted ours-rows=[]
A4 control-destroy refused rc=1 said=cannot destroy 'rmg/work/control': dataset is busy
A4 with-the-holders-copy-standing clone-destroy refused rc=1 said=cannot destroy 'rmg/work/slot': dataset is busy
A4 inside-the-holders-namespace answered={'umount2': 'accepted'}
A4 holder-alive=True
A4 after-the-copy-went clone-destroy gone rc=0 said=
A4 second-half our-unmount accepted
A4 second-half before-the-end clone-destroy refused rc=1 said=cannot destroy 'rmg/work/slot': dataset is busy
A4 second-half after-the-end clone-destroy gone rc=0 said=
```

The control dataset was itself copied into the holder's namespace,
because the namespace was made while its mount stood,
so its refusal is that same propagation rule and not a second fact:
the control this arm can trust is `arm-forced-with-a-reference.log`'s dataset made after the copy stood.

What each line proves:

- with the act's own mount already taken out and the only copy in the holder's namespace,
  the destroy refuses `dataset is busy`;
- after the copy is unmounted inside that namespace, with the holder still alive,
  the destroy answers `gone`;
- and where no unmount is made, ending the holder alone frees the dataset just as well,
  because its namespace went with it.

So the destroy cannot pass anything by itself.
Two forces therefore need an act that either ends the holder or takes its copy out,
and they need it before the destroy, not instead of it.

What a person loses:
whatever was inside the clone at the moment the kernel destroyed it,
which is why every arm above takes the copy out first.

### What each leaves, and what a person loses

The reading, one act at a time:

- ending the holder:
  the holder is gone,
  its namespace and its copy go with it,
  the dataset is free,
  and the person loses that job's unsaved work.
  A holder in a locked namespace is freed this way and in no other.
- unmounting inside the holder's namespace:
  the holder is alive,
  its copy is gone,
  the dataset is free when nothing else references it,
  and the person loses the view of the clone the job was using,
  plus, in way 7, whatever the mount under it stops receiving.
- an unmount that ends the blockers first:
  what the built act does,
  which is the two rows above in one order,
  and the person loses the blockers' unsaved work and the copy.
- a forced unmount:
  nothing,
  it is refused wherever it would matter.
- a lazy unmount:
  the holder is alive and stranded,
  the mount is off the table,
  the dataset is still busy,
  and the person sees a folder that looks empty while the job still writes.
- destroying while a copy stands:
  the destroy is refused and nothing changes.
- a refusal:
  everything stands, and the person has to leave the folder or stop the job.

### Ranking

The ranking is
*end the blockers, then unmount, then destroy* >
*unmount only* >
*end only* >
*a lazy unmount* >
*a forced unmount* >
*destroy alone*.

The reasons, in adjacent pairs:

- end and unmount over unmount only:
  an unmount alone cannot pass a path holder (`EBUSY`, `arm-unmount-in-namespace.log`)
  and cannot pass a locked copy (`EINVAL`, the same log),
  so it leaves two of the four shapes the removal meets;
  and the two together leave none of the four shapes measured here.
- unmount only over end only:
  ending alone frees the dataset in every shape measured,
  but it ends a service's job where the service could have kept running,
  which is the one promise the accepted design makes to a namespace holder.
- end only over a lazy unmount:
  the lazy unmount does not free the dataset while a reference stands,
  so it cannot be followed by the destroy that two forces promised.
- a lazy unmount over a forced unmount:
  the lazy unmount at least answers `accepted` and does release the dataset where nothing else references it,
  and the forced unmount answered `EBUSY` in the one state that mattered.
- a forced unmount over destroy alone:
  both free nothing,
  but the forced unmount leaves everything as it was,
  while a destroy that could half-run is the thing the removal's order exists to prevent.
- destroy alone last:
  it is refused in every state the others are needed for,
  and it is the step every other act is for.

Effort to build is not what ranks these:
the built arm already makes the two acts the ranking puts first,
and what it lacks is not code but three rows of its kind table,
named below.

### The act two forces use

Two forces use exactly the act the built arm already is,
in the order the design states,
with three additions the owner's answer authorises and the kind table currently refuses:

1. The preflight, unchanged.
   A blind spot, an unenterable namespace, a parent that is not the checkout's and a mount under the copy still refuse:
   two forces pass a *holder*,
   and none of those is one.
2. The structural stage, unchanged:
   way 8's open and check,
   or way 7's slave change and directory.
3. The signal stage, unchanged in its calls and now applied to every kind:
   `SIGTERM` through the pin,
   a wait of `T_wait`,
   `SIGKILL` through the same pin while the caller still lives.
   The kinds the table refuses today for want of an arm,
   another account's task and a unit's task,
   are ended by this same call:
   the pin and the signal are one mechanism,
   and what two forces add is the authority to use them,
   not a second one.
4. A locked holder that is not a service unit of the account's own user manager:
   with two forces its tasks are ended,
   which frees the namespace and the locked copy with it
   (`arm-end-holder.log`'s `namespace-userns` shape: 0.206 ms, the act's unmount then accepted, the destroy `gone`).
   One force keeps `holder-locked` here,
   because nothing measured an unmount of a locked copy and nothing can.
5. A locked holder that *is* such a unit keeps the manager's stop
   (`a_service_unit_stopped_through_the_accounts_manager.rs`),
   which is gentler than ending its tasks,
   and two forces add only the fallback:
   where the manager refuses the stop,
   or the unit comes back,
   the tasks are ended as in the row above.
6. The unmount stage, unchanged,
   and the destroy after it.

What it never does, under either force count,
because none of these is a holder:
it never destroys while a copy stands,
never detaches a mount it cannot unmount,
never unmounts with `MNT_FORCE`,
and never ends a task that holds nothing.

## Question 2: how a command that ends its own session or an ancestor still finishes

The refusal this lifts is the never-end set's:
a holder in the caller's own session,
or among the command's ancestors,
refuses the removal in both arms today
(`package/git-policy/cli/SPEC.md`,
"The forced arm, built and reachable from no command";
`src/native/worktree_clone_forced_removal/caller_chain_walk.rs`).
With two forces that refusal lifts,
and the command must then do what no single command can do by itself:
end the session it runs in,
or a process above it,
and still reach its own last step.
The four candidates below are the ones this round could see.

### Candidate 1: the removal moves itself into a unit of its own through the account's manager

How it works.
Before it ends anything,
the removal re-executes its own act as a transient service unit of the account's own user manager
(`systemd-run --user --unit=<name> --collect`),
hands that unit the request it already wrote,
and waits for it.
The unit's tasks are the manager's children,
they have their own session and their own control group,
and none of them is the command's parent,
so the act that ends the caller's session leader,
or an ancestor,
reaches nothing that is doing the removal.
The unit writes the removal's plan,
destroys,
and files its receipt;
the wrapper's own process has nothing left to do but read the receipt,
and it may die with its session.

What the guest answered,
in `arm-unit-of-its-own.log`,
where the work is a script of the guest's account started as a transient user unit,
and the session leader it ends is the leader of the session the "person's command" runs in:

```text
# arm-unit-of-its-own.log
B1 user-manager=running
B1 session-leader=pid=3229 ppid=3195 session=3229 starttime=75670 ns=mnt:[4026531841]
B1 unit-start rc=0 said=
B1 our-unmount-after-the-unit-started refused errno=EBUSY
B1 unit-finished=True session-leader-alive=False
B1 unit-state said=inactive
B1 journal B1 work unit-pid=3232 ppid=3115 sid=3232
   cgroup=0::/user.slice/user-0.slice/user@0.service/app.slice/rmg-b1.service
B1 journal B1 work ending the session leader 3229
B1 journal B1 work after-the-end session-leader-state=Z
B1 journal B1 work clone=gone
B1 journal B1 work finished pid=3232
B1 dataset-now rc=1 said=cannot open 'rmg/work/slot': dataset does not exist
B1 slot-folder=
```

What each line proves:

- the unit's task has the manager (pid 3115) as its parent,
  its own session (`sid=3232`) and its own control group;
- it ended the session leader,
  whose state then read `Z`,
  and went on:
  it wrote into the clone, unmounted its own copy, destroyed the dataset (`clone=gone`, and the dataset does not exist
  afterwards) and finished;
- the caller's own unmount was refused `EBUSY` while the unit held its copy,
  which is the act's own order stated from the other side:
  the holder's copy has to go before the act's namespace can unmount.

What the person's terminal shows:
the shell that ran the command is gone, with every job it had,
and the removal's lines are in the manager's journal for that unit and in the receipt.
Nothing of the removal prints to the dead terminal.

What stands if the machine stops half way:
the plan the removal wrote and the registration it has not removed,
which is the state the cancel command and the resume paths already take
(`package/git-policy/cli/SPEC.md`,
"The cancel command").
The unit is transient and gone;
a later command finds the plan and finishes.

What it gives up:
the account must have a running user manager and lingering,
which a server account, a container and a `systemd-run --scope` without a session do not have;
the removal's own output moves off the terminal it was started from;
and a unit a person did not start now holds the act's authority,
which is a new kind of holder of it.

### Candidate 2: the removal hands the work to the root mode, which outlives the caller

How it works.
The wrapper writes the request it already writes,
and the installed root handler does the act,
as the forced design's channel already says it does
(`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
"By what channel a holder of another account is ended").
The handler is a system unit's task,
so no process of the caller's session is among its ancestors and it survives the caller's death.
That is the attraction and also the problem:
the authority's duration was decided as "only while the command lives",
checked by the wrapper's process identity before every signal and every unmount.
For this candidate to work,
a two-force request would have to say "finish this even though I am gone",
which is a new kind of request and a new risk:
a request whose writer died keeps destroying.

What the guest answered,
in `arm-root-mode-outlives.log`,
where the handler is a system unit of the guest that waits for a request file,
then ends the caller's session leader and finishes the removal:

```text
# arm-root-mode-outlives.log
B2 handler-start rc=0 said=
B2 handler-ns=lrwxrwxrwx 1 root root 0 ... /proc/3382/ns/mnt -> mnt:[4026531841] our-ns=mnt:[4026531841]
B2 caller=pid=3401 ppid=3350 session=3401 starttime=82971 ns=unreadable:ENOENT
B2 handler-finished=True caller-alive=False
B2 journal B2 handler pid=3382 ppid=1 sid=3382 cgroup=0::/system.slice/rmg-b2-handler.service
B2 journal B2 handler saw-the-request caller=3401
B2 journal B2 handler caller-state=Z
B2 journal B2 handler clone-destroy accepted
B2 journal B2 handler clone=gone
B2 journal B2 handler finished
B2 journal rmg-b2-handler.service: Deactivated successfully.
```

What each line proves:

- the handler's parent is pid 1 and its session is its own,
  so the caller's death reaches nothing of it;
- it shares this namespace and not a copy:
  a system unit with no `PrivateMounts=` has no mount namespace of its own,
  so it works on the act's own mount directly,
  which is one mount that has to be unmounted rather than a copy;
- it ended the caller (state `Z`), then wrote into the clone, unmounted it, destroyed the dataset and finished.

What the person's terminal shows:
nothing:
the shell is gone and the handler's lines are in the root handler's journal,
and the wrapper's own report never reaches that terminal.

What stands if the machine stops half way:
whatever the handler had destroyed,
with the plan as the only record of it,
and a standing request that could act again after the machine comes back.

What it gives up:
the check that a request outlives nothing,
which is what makes a stale request harmless today.

### Candidate 3: the removal writes its plan, ends what it must, and leaves the rest to the next run

How it works.
The removal writes its destruction plan first,
as the accepted order already does before its first snapshot request,
and then ends the holders,
the caller's session leader and its own ancestors among them.
Its own process dies with them,
because its output and its place in the tree are the terminal's,
so it never reaches the destroy.
The next run reads the plan,
adopts it under a fresh nonce,
and finishes the order.
This is the option's own text:
"The command can then end the shell it runs in before it has finished, leaving a destruction plan for the next run".

What the guest answered,
in `arm-plan-then-next-run.log`,
where run one writes its plan and then reports while the reader of its standard output is gone:

```text
# arm-plan-then-next-run.log
B3 run-one=pid=3079 ppid=3054 session=3079 starttime=66514 plan-written=True
B3 run-one last-line-refused errno=EPIPE
B3 run-one-exit status=0 by-signal=False signal=none plan-still-here=True
B3 clone-after-run-one rc=0 said=rmg/work/slot
B3 run-two reads-plan=head=present target=/root/rmg-fixture/work/slot
B3 run-two our-unmount accepted
B3 run-two clone-destroy gone rc=0 said=
B3 run-two plan-removed=True
```

What each line proves:

- run one wrote its plan and then could not deliver its last line at all:
  the write answered `EPIPE`,
  because the terminal it would have printed to was gone;
- it exited without the destroy,
  and the clone was still there for the next run;
- run two read the plan, unmounted, destroyed, and removed the plan,
  so the second command finished what the first could not.

A wrapper written in Rust meets the same wall and one more:
its standard library sets `SIGPIPE` to ignore,
so the write answers `EPIPE` rather than ending the process,
which is why the last line is lost rather than the command killed.

What the person's terminal shows:
the shell is gone, and so is every line the removal would have printed.
The place to look is the plan and the next run's report.

What stands if the machine stops half way:
the plan, with every name it settled, and no other record of what the first run ended.

What it gives up:
one command,
which is what a person asked for when they typed one,
and the first run's own account of what it destroyed.

### Candidate 4: the removal never ends them and unmounts under them

How it works.
Question 1's candidate 2 is applied to the caller's own namespace:
the removal does not end the caller's shell,
and takes the copy out of that shell's namespace instead.
That is already what the removal does today where the caller's reference is a namespace copy,
since a copy that receives from the checkout is not the caller's own working directory.

What the guest answered,
in `arm-unmount-under-the-caller.log`:

```text
# arm-unmount-under-the-caller.log
B4 caller-in-the-clone=... ns=mnt:[4026531841] same-ns=True
B4 our-unmount-while-it-stands refused errno=EBUSY
B4 control-destroy gone rc=0 said=
B4 clone-destroy-while-it-stands refused rc=1 said=cannot unmount '/root/rmg-fixture/work/slot': pool or dataset is busy
B4 caller-cwd-after-leaving=/
B4 our-unmount-after-it-left accepted
B4 clone-destroy-after-it-left gone rc=0 said=
```

What each line proves:
a caller standing in the clone refuses the unmount with `EBUSY`,
and the dataset stays busy with it;
once the caller's working directory is somewhere else,
the same unmount is accepted and the destroy answers `gone`.
So this candidate passes the refusal only by asking the person to do what the refusal asked them to do.

What the person's terminal shows:
the removal's ordinary report, because the terminal is still there.

What stands if the machine stops half way:
nothing new, because this candidate changes no process's life.

What it gives up:
the case the refusal was about,
which is the caller standing inside the clone.

### Ranking

The ranking is
*the unit of its own* >
*the plan and the next run* >
*the unmount under them* >
*the root mode outliving the caller*.

The reasons, in adjacent pairs:

- the unit over the plan:
  both pass the never-end set,
  and only the unit finishes in one command,
  which is what the person asked for;
  the plan is the fallback where the manager is not there.
- the plan over the unmount:
  the unmount does not pass the caller standing in the clone at all
  (`EBUSY`, `arm-unmount-under-the-caller.log`),
  where the plan passes every case by ending them and paying a second command.
- the unmount over the root mode:
  the unmount ends nothing and needs no new authority,
  where the root mode needs a request that outlives its writer.
  The unmount's limit is a state a person can leave in one `cd`;
  the root mode's cost is the authority rule the project already decided.
- the root mode last:
  it finishes in one command and survives the caller,
  and it is the only candidate that makes a destructive request act after the requester is gone,
  which is the one thing the accepted authority rule exists to prevent.

### The rule, and when it falls back

Two forces make the candidates a rule rather than a choice:

1. Before the first ending,
   the removal asks whether any holder is the caller itself,
   a task of its session,
   or one of its ancestors.
   That is the preflight's existing read;
   with two forces its answer is no longer a refusal but a decision.
2. When the answer is yes and the account has a running user manager,
   the wrapper re-executes the act as a unit of that manager under the request's nonce,
   and waits for the receipt.
   The unit's act is the same act:
   its own session is not the caller's,
   so no ending it makes can reach it.
3. When the answer is yes and there is no manager,
   or the command is already a unit of one,
   the removal writes the plan,
   ends what it must,
   prints one line saying that the removal will be finished by the next command,
   and leaves the rest to `finish_the_removal_plans` on the next command or to the cancel command.
4. Where the answer is no,
   nothing changes:
   the act runs in the command's own process as it does today.

What this rule does not do:
it never hands a destructive request to a process that outlives its writer
(candidate 2),
and it never ends the caller without either a unit to finish in or a plan on disk.

## Question 3: under the service manager

### What the refusal is today, and what it protects

The forced arm refuses `--force` outright when the command's parent is the service manager,
or when the command's session was started by it,
and tells the person to run the removal as a child of the process that wants it
(`package/git-policy/cli/SPEC.md`,
"The forced arm, built and reachable from no command";
`src/native/worktree_clone_forced_removal/started_by_the_service_manager.rs`).
The reason is the design's own:
"the manager
... cannot be told apart from the process that asked for it"
(`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
"A command whose parent is the service manager,
or whose session was started by it").
Two reads implement it:
the wrapper's parent's `/proc/<pid>/cgroup` and `comm`,
and the session leader's parent's,
with an unreadable leader refusing as an orphan.

What that refusal protects is the never-end set and nothing else.
The set is "the caller and every process of its session, and every ancestor to the first process".
A command whose parent is the manager has the manager among its ancestors,
so the set's upper half is the manager itself and pid 1,
and its lower half is the command alone:
every other unit of the account stands between them,
neither spared nor the caller.
Ending "the holders" from such a command therefore means ending other people's units,
and the one identity the arm could check before each signal would be its own.

### What two forces do there

What the guest answered,
in `arm-service-manager.log` and `arm-scopes.log`:

```text
# arm-service-manager.log
C ourselves=pid=3504 ppid=3503 session=3501 starttime=88503 ns=mnt:[4026531841]
C guest-pid1=pid=1 ppid=0 session=1 starttime=22
C user-manager-pid=3116 pid=3116 ppid=3115 session=3115 starttime=70244
C pipe-main-pid=3519 pid=3519 ppid=3115 session=3519 starttime=88512
C pipe-main-cgroup=0::/user.slice/user-0.slice/user@0.service/app.slice/rmg-c-pipe.service ns=mnt:[4026531841]
C pipe-unit-attributed-by=rmg-c-pipe.service
C our-session=3501
C script-main-unit=3541 pid=3541 ppid=3115 session=3541 starttime=88814
C script-main-cgroup=0::/user.slice/user-0.slice/user@0.service/app.slice/rmg-c-script.service
C2 scope-name=run-r2f6982eac6724d90af11e5b3db39f24b.scope shell=pid=3593 ppid=3573 session=3570 starttime=91657
C2 scope-child=pid=3594 ppid=3593 session=3570 starttime=91659
C2 scope-jobs-before=2
C2 scope-stop rc=0 said=
C2 scope-jobs-after=1 shell-alive=False child-alive=False
C2 pipe-main-after=True
```

What each line proves:

- a unit's task has the manager as its parent (pid 3115),
  its own session,
  and its own control group,
  and the manager names the unit its request came from (`Id` = `rmg-c-pipe.service`);
- a unit whose main process is a script that then runs the removal is the same shape:
  the script's parent is the manager and its session is its own,
  so the design's two reads cannot tell that unit's script from the person;
- a scope's jobs keep the caller's session (`session=3570`, which is the probe's own),
  so a scope is the person's own session in the manager's own words;
- stopping the scope ended both of its jobs,
  and left the unit's job alive:
  a stop is a scope-wide ending,
  which is exactly what "end the caller's session" would mean if a removal took a scope for its caller.

The rule this measurement points at,
for two forces:
the requester is named by its own unit,
not by the parent chain.
The command reads its own unified control-group line,
takes the last component,
and requires it to end `.service` or `.scope`;
that name is its identity,
and the never-end set becomes the tasks in that unit's control group.
The manager is never ended,
and no other unit is ever ended,
because neither is in the set.

What the person's terminal shows:
the removal's ordinary report,
since a unit's own output is in the journal and the scope's is on the terminal it was started from.

What stands if the machine stops half way:
the plan, as in question 2.

### What the refusal's reason still protects after the change

Three things, and each is a sentence the design must keep:

- the removal never ends the manager,
  nor pid 1,
  nor anything above the unit it was started in.
  With the identity read from the unit's own control group,
  the manager is not in the set at all;
- the removal never ends another unit,
  which the old never-end set could not promise once its upper half became the manager:
  with the manager as an ancestor,
  every other unit was neither spared nor the caller;
- a removal started in the person's own shell through `systemd-run --scope`
  spares that shell and its jobs,
  because the scope is the session and its jobs are in the set.
  The measurement above is why this is a rule and not a preference:
  stopping the scope ends both of its jobs.

What it gives up:
the refusal that told a person to start the removal as a child of the process that wants it.
A request from a unit can now act,
and its only witness is the manager's journal and the receipt.

## Question 4: the values the waits ship with none of

### What each constant governs

The forced arm ships four numbers as `Option`s that answer nothing,
and one function refuses while any of them is unset
(`src/native/worktree_clone_forced_removal/forced_arm_numbers.rs`):

- `TERMINATION_WAIT`, the design's `T_wait`:
  how long a blocker has to leave after `SIGTERM` before it is sent `SIGKILL`.
- `HEARTBEAT_PERIOD`, the design's `T_h`:
  how often the handler advances its progress file's counter.
- `GIVE_UP_HEARTBEATS`, which is a relation and not a period:
  four whole periods, after which a counter that has not moved reads as a dead handler.
- `STOP_WAIT`, the design's `T_stop`:
  how long a locked unit is given to leave its control group after its manager was asked to stop it.

The design states what each must exceed and refuses to state a number
(`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
"The signal",
"The act's liveness",
L9 and L21).
What it does not say,
and what this round had to decide before it could measure anything,
is what each number is a bound on:

- `T_wait` bounds how long a polite exit is waited for,
  so it must exceed the slowest exit a cooperating holder was measured to make,
  and it is a ceiling rather than a target:
  a holder that never leaves costs the whole wait and is killed at its end.
- `T_h` bounds how long the wrapper may read a counter that has not moved and still call the handler alive.
  It is sized from the longest stretch in which the handler advances nothing but its own timer:
  one stage of the act, and one publish of the progress file.
  Too small a period makes a slow stage read as a dead handler;
  too large a one makes a dead handler cost the person a long wait before the wrapper says so.
- `T_stop` bounds a manager's answer,
  which is a program's and not the act's own call,
  so it must exceed the slowest stop measured and not the act's stages.

### How each was measured

Every number below was taken in the guest named above,
in rounds where the arm being measured and a control arm ran interleaved,
so that a loaded host moves both and the answer is a ratio as well as a time
(rule `TVC`).
Each round's own times are printed in the order they were taken,
and the range is the lowest and the highest of the rounds,
never one round's figure.
The control arm is the measurement's subject with the smallest work that still shows the number:
a bare `sleep` for the termination wait,
a unit that holds nothing for the stop's wait,
the same write without its syncs for the progress file,
and the same unmount at home for the act's stage.

The numbers are the guest's.
This machine's kernel is 7.3 and the guest's is 6.8;
this machine's systemd is 262 and the guest's is 255;
and the guest's pool is a file with the distribution's OpenZFS 2.2.2,
not this machine's ZFS 2.4.4 over a real disk.
Where any of those could change an answer it does so by a constant,
not by an order of magnitude,
which is why the values below are stated as multiples of measured figures and not as the figures alone.

### T_wait

```text
# arm-waits.log
W t-wait arm=control-bare-sleep ms=[0.453, 0.346, 0.448, 1.044, 0.349]
W t-wait arm=holder-that-syncs-then-leaves ms=[43.528, 24.401, 4.24, 22.775, 23.744]
W t-wait arm=holder-that-writes-16m-then-leaves ms=[87.081, 34.594, 36.239, 66.916, 34.288]
W t-wait-ignorer arm=holder-that-ignores-the-ask ms=[1000.614, 1000.633]
   said={"term_ms": null, "left_after_term": false, "kill_ms": 0.129, "left_after_kill": true}
```

Read as ratios to the control taken in the same rounds:

- a holder that runs `sync` and leaves took a median of 23.744 ms against the control's 0.448 ms,
  53 times the control, over a range of 4.24 to 43.528 ms;
- a holder that writes 16 MiB and syncs it before leaving took a median of 36.239 ms,
  81 times the control, over a range of 34.288 to 87.081 ms;
- a holder that ignores the ask did not leave within the 1000 ms the arm gave it,
  and the `SIGKILL` through the same pin was observed 0.129 and 0.135 ms later.

So the slowest cooperating exit measured is 87.081 ms,
and the wait must exceed that by enough that a host under load does not turn a polite exit into a kill.
Two thousand milliseconds is 23 times that figure;
one thousand is 11.5.

### T_h and its four-period window

The handler's own longest stretch without advancing anything but its timer is one stage of the act.
The stage measured here is the act's cheapest and its commonest:
enter one holder's namespace and unmount the copy.

```text
# arm-waits.log
W act-stage arm=enter-a-holders-namespace-and-unmount ms=[13.217, 9.952, 15.324]
W act-stage arm=control-unmount-in-our-own-namespace ms=[3.454, 4.534, 6.928]
W progress-file arm=publish-with-both-syncs ms=[0.049, 0.025, 0.023, 0.022, 0.022]
W progress-file arm=control-publish-without-syncs ms=[0.025, 0.019, 0.018, 0.018, 0.018]
```

Read as ratios:

- the stage in a holder's namespace has a median of 13.217 ms against 4.534 ms for the same unmount at home,
  2.9 times the control, over a range of 9.952 to 15.324 ms;
- the progress file's publish, with both syncs, has a median of 0.023 ms against the control's 0.018 ms,
  1.3 times the control, over a range of 0.022 to 0.049 ms.

A period of 1000 ms is 65 times the slowest stage measured here,
and the four-period window of 4000 ms is 260 times it.
The publish is three orders of magnitude under the period,
so the file's own write never decides the period,
and what the period must cover is a stage,
which is a mount call inside another namespace and the reads around it.

### T_stop

```text
# arm-waits.log
W t-stop arm=locked-unit-stop ms=[5.661, 5.838, 6.867]
W t-stop arm=control-unit-stop ms=[5.03, 4.95, 4.515]
```

Read as ratios:
a locked unit's stop, from the manager's answer to the unit's main task being gone,
has a median of 5.838 ms against 4.95 ms for a unit that holds nothing,
1.2 times the control, over a range of 5.661 to 6.867 ms.
Both are the manager's own work:
a stop's overhead dominates, and the lock costs about 0.9 ms of it.

### The values this design proposes

Each proposed value is a multiple of the measured figure it bounds,
and the multiple is stated so a later measurement can move it:

- `TERMINATION_WAIT` = 2000 ms,
  23 times the slowest cooperating exit measured (87.081 ms),
  and the whole of what a holder that ignores the ask costs before the `SIGKILL`
  that is observed 0.13 ms later.
- `HEARTBEAT_PERIOD` = 1000 ms,
  65 times the slowest act stage measured (15.324 ms).
- `GIVE_UP_HEARTBEATS` = 4, unchanged, so the window is 4000 ms,
  which is 260 times the slowest stage.
- `STOP_WAIT` = 2000 ms,
  291 times the slowest stop measured (6.867 ms).

These are the same figures step 22's measurement proposed
(`doc/planning/cli-git-zfs-step-22-guest-half-measurements.md`,
"The values this measurement would support"),
and this round's readings do not move them.
What this round adds is the reason each is safe:
the largest figure each bounds was measured on this round's guest,
interleaved with a control,
and the margin is stated as a ratio.

## What could not be measured, and why

- A copy that receives from a shared parent, and the peer and slave shapes of the receiver closure.
  They are the 2026-11-23 rounds' measurements and were not repeated here.
- A taskless namespace.
  Its route needs `listns`, which the guest's 6.8 kernel answers with `ENOSYS`,
  and the earlier fourth round ran it on a 7.0 kernel.
  So what two forces do about a namespace no task holds is unmeasured in this round,
  and the design leaves its refusal where the built arm has it.
- A holder of another account, and a task of a unit the caller cannot signal.
  The guest has one account, so no arm sent a signal to another account's task.
  The pin and the signal are the same mechanism for every kind,
  and what is unmeasured is the kernel's answer `EPERM` to a same-namespace signal from an unprivileged caller,
  which the handler never makes because it runs as root.
- A unit of `v262`.
  The guest's systemd is 255,
  which decides whether a user manager's unit gets a user namespace of its own
  (F60 and F62 of the forced design).
  So the locked-holder route is measured only in the shape 255 gives,
  where a unit's mount namespace is owned by the initial user namespace,
  and this machine's own reading of 262 is the decision record's.
- Way 7 end to end inside the act.
  This round measured its parts (the unmount in a namespace, the destroy after it)
  and not the move onto the directory inside a holder's namespace,
  which the forced act's own guest arm covers
  (`e2e/forced-act-guest.ts`,
  phases `namespace-way7-repaired` and `namespace-way7-shared`).
- What the person's terminal does when its session dies.
  A guest has no terminal,
  so the readings are the session leader's state and the failing write (`EPIPE`),
  not a screen.
- A host under this machine's own load.
  The guest ran on a file-backed pool while other rounds' guests ran on this host,
  so the ranges are a contended machine's and not a quiet one's.

## The questions for the owner

Each question stands alone.
Each option says how it works,
what the measurement behind it was,
and what a yes leaves undecided.

### Question 1: which act two forces make on a holder that is not the caller's own

The ranking is B, then A, then C.

Option B, and the one this design recommends:
end the blockers,
unmount the holder's copy where it can be unmounted,
and destroy after both.
How it works:
the built arm's order, unchanged,
with its signal stage applied to every kind of holder,
including another account's task, a unit's task,
and the tasks of a locked namespace that is not the account's own unit;
where the copy is a namespace's only reference, the holder keeps running.
Measured:
a namespace holder keeps running while its copy goes and the destroy answers `gone`
(`arm-unmount-in-namespace.log`);
a task's working directory or open file refuses the unmount with `EBUSY` and frees it only when ended
(the same log, and `arm-end-holder.log`);
a locked copy answers `EINVAL` to the unmount and is freed by ending its namespace's tasks
(`arm-end-holder.log`'s `namespace-userns`, 0.206 ms, destroy `gone`).
What a yes leaves undecided:
whether the two kinds with no arm of their own may be ended on the strength of the mechanism alone,
which the owner's "two force does everything" answers but a guest of another account has not shown;
and what happens to a taskless namespace, which stays refused.

Option A:
keep the refusal for a locked namespace that is not the account's own unit,
and for another account's task and a unit's task,
so two forces pass only the kinds the built table has a measured arm for.
How it works:
the table's rows are unchanged and the retort stays `holder-locked` or the kind's own word.
Measured:
the same as B;
nothing in the guest shows a case where A finishes and B cannot.
What a yes leaves undecided:
which of the two answers "two force does everything" means for the very refusals the answer named.

Option C:
end every task of every holder namespace first,
then unmount and destroy.
How it works:
one pass of `SIGKILL` over every namespace the scan found,
with the unmount kept only as a cleanup.
Measured:
ending a namespace's last task frees its copy and the dataset in every shape measured here,
so C would finish.
What a yes leaves undecided:
it ends a running service's jobs where the same removal could have kept the service up,
which is the promise the accepted design makes to a namespace holder,
and it makes the `SIGTERM` wait pointless for every holder.

### Question 2: for a locked holder that is a service unit of the person's own user manager

The ranking is A, then B, then C.

Option A, recommended:
keep the manager's stop as the route,
and add the tasks' end as its fallback where the stop fails or the unit comes back.
How it works:
four reads tell the unit apart (the namespace's owner, the unit's name in the control-group line,
the manager's answer for that name, the protected set),
then `StopUnit` and the wait for the control group and the namespace to go;
a stop the manager refused, a cgroup still there at `T_stop`,
or a unit active again ends the unit's tasks instead and says so.
Measured:
a locked unit's stop ended the unit, its control group and its namespace,
and the clone's own unmount and destroy followed
(`arm-end-holder.log`'s `namespace-userns` is the shape,
and the 2026-11-27 arm's `locked-holder-stopped-through-its-unit` is the route's own code on it).
What a yes leaves undecided:
the fallback's order against the manager's own `Restart=` attempts,
which no arm has watched.

Option B:
end the locked unit's tasks directly, and never ask the manager.
How it works:
the signal stage over the unit's control group,
with no manager call at all.
Measured:
the same arms;
nothing measured a unit whose `Restart=` would restart it before the next scan.
What a yes leaves undecided:
it ends a service the person was running where a stop would have ended it the way its manager does,
and it removes the one route where the manager's own accounting says what happened.

Option C:
keep today's refusal.
How it works:
`holder-locked`, before any signal, naming the unit and the stop command.
Measured:
the same.
What a yes leaves undecided:
whether two forces may leave a refusal in place at all.

### Question 3: how a command that must end its own session or an ancestor finishes

The ranking is A, then B, then C, then D.

Option A, recommended:
the unit of its own where a user manager is there, and the plan where it is not.
How it works:
the preflight's own read says whether the protected set holds;
a yes re-executes the act as a transient user unit under the request's nonce and waits for the receipt,
and where no manager answers, the plan is written, the endings are made, and one line says the next command
finishes it.
Measured:
a unit of the account's manager ended its own session leader (state `Z`),
then wrote into the clone, unmounted, destroyed and finished
(`arm-unit-of-its-own.log`);
and run one wrote its plan, could not deliver its last line (`EPIPE`),
and run two finished with one destroy (`arm-plan-then-next-run.log`).
What a yes leaves undecided:
the report a person sees when the manager finds no room for the unit,
and whether the fallback should be a refusal for two forces rather than a delayed finish.

Option B:
always the plan and the next run.
How it works:
write the plan, end what must be ended, and stop.
Measured:
`arm-plan-then-next-run.log`'s two runs.
What a yes leaves undecided:
it makes every removal that ends its own ancestor a two-command removal,
which is not what a person typing one command asked for.

Option C:
the unmount under them, never ending them.
How it works:
question 1's second candidate applied to the caller's namespace.
Measured:
`EBUSY` while the caller stands in the clone (`arm-unmount-under-the-caller.log`),
and accepted once its working directory is elsewhere.
What a yes leaves undecided:
it passes only the caller whose reference is not a path into the clone,
so the refusal it was meant to lift stays for the commonest case.

Option D:
hand the act to the root mode and let the request outlive the caller.
How it works:
the installed handler does the act after the requester is gone,
which needs a request whose authority outlasts its writer.
Measured:
a system unit survived the caller's death and finished
(`arm-root-mode-outlives.log`).
What a yes leaves undecided:
the rule "the handler ends processes only while the very command that asked is still running",
which this option reverses, and the standing request a crash would leave.

### Question 4: under the service manager

The ranking is A, then B.

Option A, recommended:
lift the refusal and name the requester by its own unit.
How it works:
the command reads its unified control-group line,
requires a last component ending `.service` or `.scope`,
and the never-end set becomes the tasks in that unit's control group;
the manager and every other unit are never ended;
a request whose unit cannot be read still refuses.
Measured:
a unit's task has the manager as its parent and its own session
(`arm-service-manager.log`),
the manager names the unit (`Id` = `rmg-c-pipe.service`),
a scope's jobs keep the caller's session and a stop of that scope ends them
(`arm-scopes.log`),
and a unit's job survives that stop.
What a yes leaves undecided:
what a request from a `.scope` that is the person's own shell session may end
(the scope's own jobs are its set, and ending another scope's jobs is still refused),
and whether a unit name that no longer exists should refuse or fall back.

Option B:
keep today's refusal.
How it works:
exit 2 with the sentence that names the manager and the remedy.
Measured:
the same.
What a yes leaves undecided:
whether two forces may leave a refusal in place whose reason is about telling the requester apart
and not about the clone.

### Question 5: the values the waits ship with

The ranking is A, then B, then C.

Option A, recommended:
the step 22 proposals, now with a measured basis.
How it works:
`T_wait` 2000 ms, `T_h` 1000 ms, its four-period window 4000 ms, `T_stop` 2000 ms,
set in `forced_arm_numbers.rs` and nothing else changed.
Measured:
23 times the slowest cooperating exit (87.081 ms),
65 times the slowest act stage (15.324 ms),
260 times that stage for the window,
291 times the slowest stop (6.867 ms).
What a yes leaves undecided:
a parent with more clones than the sixty-four step 22 measured,
whose sweep could exceed the period,
and this machine's own kernel and manager, which this round did not run.

Option B:
tighter, at half.
How it works:
`T_wait` 1000 ms, `T_h` 500 ms, window 2000 ms, `T_stop` 1000 ms.
Measured:
11.5 times the slowest cooperating exit, 33 times the slowest stage,
and 146 times the slowest stop.
What a yes leaves undecided:
how much of the margin a holder that flushes a large file on this machine's real disk would spend,
which no arm measured here.

Option C:
looser, at two and a half times A.
How it works:
`T_wait` 5000 ms, `T_h` 2000 ms, window 8000 ms, `T_stop` 5000 ms.
Measured:
57 times the slowest cooperating exit and 130 times the slowest stage.
What a yes leaves undecided:
the wait a person pays when a holder ignores the ask,
which is the whole of `T_wait` and is bounded by nothing else.

## Revisions

- The round that wrote this document:
  the first text,
  its structure and its questions;
  then the measurements,
  the guest and the program above,
  and every reading quoted with the log that answered it.
  No commit of this document changes release code or a test of the release.

What is not here, and why:
a decision.
Every section above states what a thing would do and what it measured;
the choices are the five questions,
and the owner answers them.
