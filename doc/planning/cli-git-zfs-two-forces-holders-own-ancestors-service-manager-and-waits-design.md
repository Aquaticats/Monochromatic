# Two forces at a ZFS removal: the act that passes a holder, the command that ends its own ancestors, the service manager, and the waits' values

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
The third is implied by the answer and is the one place where "two force does everything" meets a refusal whose
reason is not a refusal at all but an inability to tell the requester apart.
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

## What this design read, and what it ran

Read whole:

- `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
  the accepted forced design,
  with its ways out of a shared namespace,
  its preflight,
  its signal stage,
  its act's liveness and its constants with no value.
- `doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
  the channel,
  the reach,
  the never-end set,
  the signal and the authority's duration.
- `doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`,
  the accepted design,
  its never-end set as built,
  its two kept refusals and its refusal under the service manager.
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
- The code under `src/native/worktree_clone_forced_removal/`,
  every release file of it,
  including `holder_endings.rs`,
  `holder_preflight.rs`,
  `the_structural_stage.rs`,
  `the_unmount_stage.rs`,
  `blocker_end.rs`,
  `started_by_the_service_manager.rs`,
  `caller_chain_walk.rs` and `forced_arm_numbers.rs`.
- `package/git-policy/cli/e2e/README.md`,
  "The forced act's guest arms" and "Guest work that runs longer than the agent's safe length",
  and the forced act's driver `e2e/forced-act-guest.ts`.

Run:
one guest of this round's own,
named below,
through `mvm --backend libvirt`,
with the probe program of this round;
the program and every arm's log are named in "The guest, its kernel and the fixture" and in "Revisions".
Nothing was run on this machine:
no mount,
no `setns`,
no `umount2`,
no signal to another process,
no `systemctl`,
no ZFS command.
No service manager was started on this machine,
and no command of this round opened `/dev/zfs` here.

## The guest, its kernel and the fixture

[TO FILL: the guest's name, its kernel, OpenZFS version, the fixture, the program's hash, the invocation, the bound.]

## Question 1: by what act two forces get past a process that holds the clone

### The state each candidate starts from

The removal's act reaches a clone the way the accepted design's states it:
the clone is a ZFS dataset mounted at its slot,
and another process holds it in one of two ways,
which the design calls a mount holder and a path holder.

A mount holder has a mount of the clone's dataset in its own mount namespace,
made either by the kernel copying the mount into a namespace created after the clone was mounted,
or by the holder mounting it itself.
A path holder has no mount of its own and holds the clone through a reference to the mounted one:
its working directory,
an open file,
its running executable,
a mapped file,
its root,
a thread's working directory,
or a unix socket bound to a path inside it.
Every one of those references is what makes a plain unmount refuse `EBUSY`
(`fs/namespace.c` at `v7.3`,
F17 of the forced design),
so a path holder is a blocker even where no copy of the mount is its own.

OpenZFS adds the second half of every arm:
the clone's head destroy refuses while the dataset still has a reference from a mount
("dataset is busy"),
so an act that does not release the last copy cannot be followed by the destroy,
however cleanly it ends processes.

### Candidate 1: end the holder

[TO FILL: mechanism, the calls, the four shapes, what each answered, what is left, what a person loses.]

### Candidate 2: unmount inside the holder's namespace without ending it

[TO FILL]

### Candidate 3: a forced unmount

[TO FILL]

### Candidate 4: a lazy unmount

[TO FILL]

### Candidate 5: destroy the clone while a copy stands

[TO FILL]

### What each leaves, and what a person loses

[TO FILL]

### Ranking

[TO FILL]

### The act two forces use

[TO FILL]

## Question 2: how a command that ends its own session or an ancestor still finishes

[TO FILL]

## Question 3: under the service manager

[TO FILL]

## Question 4: the values the waits ship with none of

[TO FILL]

## What could not be measured, and why

[TO FILL]

## The questions for the owner

[TO FILL]

## Revisions

[TO FILL]
