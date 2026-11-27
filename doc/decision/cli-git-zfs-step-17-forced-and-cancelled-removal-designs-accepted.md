# Step 17's two removal designs accepted, and a locked holder's unit stopped through its own manager

## Status

Decided by the owner on 2026-11-23,
through the question tool,
after the designs' reviewer's third read
(`~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review-third-read-after-the-probe-rounds.md`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here)),
the designs' rounds on it (`64b52da212`,
`63488826bc`)
and the probe round on the arms that need no step 17 code (`2149a02e14`).
Recorded by the session.
That read's high finding was that the design's own example,
a `watcher.service` with `PrivateMounts=yes` in the person's own user manager,
falls in the class the design refuses,
and that the lock it refused on cannot be read at all:
`/proc/<pid>/mountinfo` prints no lock bit and `statmount` carries none.
It proposed reading each holder's mount namespace owner with `NS_GET_USERNS` instead,
which is the read recorded under "What the question rested on".

## The forced removal design

`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`.

### The design is accepted

The question described what a forced removal does:
it stops the jobs that hold the clone and never touches the caller's own command or session;
takes the clone out of a running service's view while the service keeps running,
in place by the marker where the marker exists and the service's mount shares with a group of its own,
and otherwise through a short-lived folder at the top of the checkout;
refuses before ending anything when it cannot enter or read a holder,
a mount sits under the copy,
or the holder's copy is locked by a user namespace of its own;
and reports every sandbox that stops receiving the service's later mounts.
It said the whole removal against a real service,
a removal killed midway,
and the timings that size the wait need step 17's code.

The answer:
"Accept it (Recommended)"

The design's veto-open defaults ride along with it,
as the question named them,
and the wait's length stays unset until it is measured,
so the forced arm does not ship before that.

### A holder whose copy is locked (V10)

The question:
under systemd 262,
this machine's,
a unit of the person's own user manager with `PrivateMounts=yes`
(and,
by the source,
`PrivateTmp=`,
`ProtectHome=` and the bind options)
has its mount namespace owned by a user namespace of its own,
so the kernel refuses both the unmount and the move of the clone's copy,
and nothing but ending that namespace frees it.
Its example was `watcher.service` holding `.claude/worktrees/one` when the person runs `git worktree remove --force one`.
The options were to stop that unit through the person's own user manager and finish the removal in the same run,
to refuse and print the exact stop command,
or to refuse and name the holder (the design's answer until then).

The answer:
"Stop that unit (Recommended)"

So for a locked holder that is a unit of the account's own user manager,
the forced removal stops the unit through that manager,
whose stop ends the unit's jobs in the manager's own way,
and the removal then proceeds;
the report names the unit it stopped so the person can start it again.
A locked holder that is not such a unit
(a bubblewrap sandbox or a rootless container started by hand)
has no manager to stop it through,
so for it the design's refusal stands;
that reading of the answer's scope is the session's,
from the option's words,
open to the owner's veto.

### What the question rested on

- A probe in a guest with systemd 255 and kernel `7.0.0-43`
  (`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
  "The three arms that need no step 17 code,
  2026-11-23T21:16:46-05:00"):
  a holder made by `unshare --user --map-root-user --mount` reads a mount namespace owned by `user:[4026532392]`,
  and the move and the unmount of its copy both answer `EINVAL`;
  that guest's user units with `PrivateMounts=yes` read a mount namespace owned by the initial user namespace,
  and both actions are accepted.
- A read on this machine at about 2026-11-24T02:24Z,
  by the session:
  a transient user unit started with `systemd-run --user --property=PrivateMounts=yes sleep 90`
  had its mount namespace `mnt:[4026533806]` owned,
  by `NS_GET_USERNS`,
  by `user:[4026533557]`,
  while the session's shell's mount namespace is owned by the initial `user:[4026531837]`;
  the unit was stopped afterwards.
  No unmount was tried on this machine;
  the lock is read from the owner the guest showed decides it.

## The cancelled or interrupted removal design

`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`.

### The design is accepted

The question described it:
a removal without `--force` that meets a holder stops,
puts the clone back at its slot and exits with status 1 and a report;
a holder no scan could see,
found after some snapshots were destroyed,
leaves the clone attached again without them,
each named,
with the record of the loss kept until that clone's next removal;
a removal that times out or is killed is neither cancelled nor finished,
and its standing request may keep moving the clone's mount out and back at later triggers until the job leaves,
the removal runs again,
or `cancel-worktree-removal <slot>` ends it;
and that command checks that the slot,
the clone and the records agree before it moves anything,
and cannot restore a destroyed snapshot.

The answer:
"Accept it (Recommended)"

Its veto-open defaults ride along with it,
as the question named them,
and the receipt's new outcome word is step 9's to propose.

## The two refusals the built arm keeps (owner, 2026-11-26T00:05-05:00)

The forced removal's act is built and its guest arm passed with all seven phases on the final code
(branch `feat/git-policy-cli-zfs-step-17-forced-arm-act-proven-in-a-guest`,
its round's `keep/results-forced-act.md`).
It refuses two kinds of holder that the design answered differently.

A locked holder,
a copy inside a mount namespace owned by a user namespace of its own,
answered `EINVAL` to the unmount and the move in the guest,
and the design's V10 answer was "Stop that unit (Recommended)".
No arm has measured that stop,
because the guests have no user manager of the account's own,
so the arm refuses with `holder-locked` and names the `systemctl --user stop` that ends it.
Asked whether to keep that refusal or build the stop,
the owner answered "Keep the refusal (Recommended)":
the refusal is what the arm does until an arm watches the stop,
and the design records it so.

A taskless namespace,
a mount namespace whose tasks are gone and which still holds the copy,
can be found only through `listns`,
which every guest the arms use answers with `ENOSYS`;
the arm refuses it with `holder-unendable`.
Asked whether to keep that refusal until a guest of 6.19 or newer measures the walk,
the owner answered "Keep the refusal (Recommended)".
