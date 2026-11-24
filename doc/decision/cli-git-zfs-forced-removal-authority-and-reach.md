# cli-git ZFS forced removal: the authority, a seen holder, the reach, and who is spared

## Status

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T19:00:56-05:00.
It answers four questions that the gate review of the two step 17 designs put to the project
(`~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review-second-pass.md`,
section "The single set of questions for the owner",
questions S1,
S2,
S3 and S5),
about the open items "Forced removal:
holders ended,
caller's session spared" and "Cancelled or interrupted removals and the cancel command"
of `doc/planning/cli-git-zfs-worktree-design-proposal.md`.
The designs it binds are still unapproved:
`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`
and `doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
each on its own branch.

## What was decided

### By what channel a holder of another account is ended

Asked:
when `git worktree remove --force` has to end a process of another account or a unit's task,
which privileged way may the wrapper use.
The project chose "B:
root handler (Recommended)":
the wrapper writes a request that the installed root handler reads,
and root ends the holders,
through the property and receipt protocol the route already installs.
No second privileged channel is added.

The cost the question stated with this option:
a property the account can write makes root end processes,
so the handler's own checks of who asked,
which holders,
and who is spared carry the weight.

### What a removal without `--force` does when its scan sees a holder

Asked:
the project's earlier answer says "the scan that refuses the holders it sees stays",
and two designs read that sentence in opposite ways.
The project chose "A:
cancel,
delete nothing (Recommended)":
the removal stops before any snapshot is deleted,
attaches the clone at its slot again,
and reports the holder.
A removal that would have gone through had the holder left a moment later is cancelled and is run again by the person.

This fixes the reading of the earlier sentence:
a holder the scan sees stops the destruction.

### How much a forced removal ends in another mount namespace

Asked as a direction and not as a final acceptance,
because the narrower options need root to enter another mount namespace and unmount there,
which the accepted design does not authorise
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
line 732)
and which no guest has measured.
The project chose "B:
unmount,
end blockers only":
the handler unmounts the clone inside the holder's namespace and ends only the tasks that block that unmount,
and when the namespace cannot be entered or the unmount is refused,
the forced removal refuses and names the holder.
It never falls back to ending every task of the namespace.

The recommended option was the same unmount with that fallback;
the project did not take the fallback.
A forced removal can therefore still refuse.

Still owed before this is final:
the design states how the mount to unmount is named inside the holder's namespace,
what is checked before and after the call,
what happens to a peer group and to a child mount,
and a guest measures it;
then the project confirms.

### Which processes a forced removal never ends

Asked:
the project's decision spares "the caller and the processes of its session",
and the review measured on this machine that a command run by a harness's shell tool has its shell in one session
and the harness in another,
and that a command started as a `systemd-run` unit has the service manager as its parent.
The project chose "D:
ancestors,
refuse as unit (Recommended)":
a forced removal never ends the caller's session or any ancestor of the command up to the first process,
and refuses,
naming it,
when one of them holds the clone;
and it refuses `--force` outright when the command's parent is the service manager,
telling the person to run the removal as a child of the process that wants it.

The cost the question stated with this option:
a forced removal launched as a service unit is refused.

## What was not chosen

- A root helper started through `sudo -n`,
  and the account acting alone,
  for the channel.
- Refusing and keeping the removal pending,
  and going on to delete the clone's snapshots,
  for a holder a scan sees without `--force`.
- Ending every task of each holder namespace,
  alone or as a fallback,
  for the reach.
- The caller's session alone,
  and the session with its ancestors but no refusal for a service unit,
  for who is spared.

## Four further answers

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T19:52:06-05:00,
after both designs were rewritten to the first four answers and reviewed again
(`~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review-after-the-owners-answers.md`,
section "The questions as I would put them",
questions S12,
S4,
S6 and S7).

### A holder in a namespace whose mounts are shared

Asked:
the review measured that a systemd service with mount sandboxing,
started while the worktree existed,
holds its copy of the clone in a namespace where every mount is shared;
the kernel does not move a mount out of a shared parent,
and the design changed no other unit's mount setup,
so under the third answer of this record a forced removal refused for the commonest namespace holder.
The project chose "B:
design an unmount there":
the design finds a way to unmount the copy inside such a namespace,
for example by making its parent mount private first,
so that the service keeps running and the removal finishes.
The recommended option was to refuse and name the unit;
the project did not take it.

The cost the question stated with this option,
in its words:
"it changes what a running service's namespace receives from the host,
which the accepted design does not authorise today;
one more design round and a guest measurement before you can confirm."
So it is a direction like the third answer:
a design round,
then a guest measurement,
then the project confirms.

Correction of 2026-11-22T20:34:39-05:00:
this section first added that the answer "is that authorisation,
for a forced removal's unmount of the clone's own copy and nothing wider",
and cited a line of the proposal for it.
The option said neither,
its own example changes a mount other than the clone's copy,
and the cited line is about a path unmount.
How far the change may reach is for the design round to propose
and for the project to confirm after the measurement.
The forced-arm design's round of that evening and its review were briefed with the withdrawn sentence.

### The signal

Asked:
may a forced ending first give the process a chance to save and exit.
The project chose "A:
ask,
wait,
then kill (Recommended)":
`SIGTERM`,
a wait for the process to exit,
then `SIGKILL` if it has not.
The length of the wait ships as no number until the project decides one or a measurement sizes it.

### How long a forced request's authority lasts

The project chose "A:
only while the command lives (Recommended)":
the handler ends processes only while the very command that asked is still running,
checked by its process identity before every signal and every unmount.
Afterwards the standing request may still cause mount work that ends nothing.

### A cancellation after part of the history is gone

Asked:
a holder no scan saw can surface at the head destroy,
after some of the clone's own snapshots were destroyed,
and the removal then cancels itself.
The project chose "A:
mount again and report (Recommended)":
the wrapper attaches the clone at its slot again,
with nobody present,
and reports each snapshot that was destroyed.

### Not chosen among these four

- Refusing and naming the unit,
  withdrawing the unmount inside other namespaces,
  and ending every task of such a namespace.
- `SIGKILL` alone,
  and `SIGTERM` alone.
- Authority while the nonce matches and the session exists,
  and authority for a fixed time.
- Leaving the clone unmounted after a loss.

## One more answer: a unit whose main process is a script

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T20:21:19-05:00
(the same review,
question S13).

Asked:
by the fourth answer of this record a forced removal spares the processes that led to the command
and refuses when the command was started directly by the service manager.
A unit whose main process is a script that then runs the removal is missed:
the command's parent is the script,
so the removal goes ahead,
and whoever started the unit is not on the spared chain.

The project chose "B:
refuse that too (Recommended)":
a forced removal is refused also when the command's session was started directly by the service manager,
not only when the command itself was.
Commands that agents run stay allowed,
since their sessions are started by the agent.

Not chosen:
the first wording alone,
which refuses only when the command's own parent is the service manager.

## The unmount inside a holder's namespace, confirmed as a direction on 2026-11-23

Asked in the question tool on 2026-11-23,
after the forced removal design's Tier 0 arms ran on kernel 6.8 in a throwaway guest and on this machine's kernel 7.3 in a
rootless container over ZFS,
and its Tier 1 arms on a file-backed pool,
all with hand-made mounts
(`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`;
the reviewer's read against the logs:
`~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review-second-probe-round.md`).
It confirms a direction,
not a build:
the design's third group of measurements needs step 17's code.

How the copy is taken out of a holder's view:
the project chose "A:
unmount by the marker (Recommended)":
"Where the marker exists,
unmount the copy where it stands and change no sharing;
otherwise,
or where the service shares its mount with the host both ways,
stop that one mount sharing,
move the copy onto a short-lived 0700 folder in the checkout,
unmount it there,
remove the folder."
Not chosen:
"B:
always make private,
move,
unmount",
and "C:
refuse and name the service".

A sandbox that holds no copy but receives from the checkout's mount,
when the second route stops that mount sharing:
the project chose "Go ahead and report it (Recommended)":
"The removal finishes;
the report names the sandbox and the later mounts it will no longer receive."
Not chosen:
"Refuse and name the sandbox".

A sandbox whose own mount sits on the very folder where its copy of the clone is mounted:
the project chose "Go ahead and report it (Recommended)":
"The kernel removes the hidden copy;
the sandbox's `findmnt` at the folder still shows its own mount;
the report names the sandbox and its mount."
Not chosen:
"Take the second route for it".
A mount lower down,
such as on `one/target`,
is a case the design refuses and no test ran.

## What this record does not decide

How long the record of a cancelled removal's loss lives,
and whether a forced ending leaves a durable record beyond the journal,
are the reviews' questions S8 and S11 and are still open.
That the cancel command may reverse a forced removal,
and that a cancellation's loss is no standing notice,
are the designs' own choices inside the accepted answers,
listed for the project's veto.
