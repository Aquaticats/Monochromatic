# cli-git ZFS worktree design

## Status

Accepted 2026-11-19 in the question tool, under "Answered in the question tool, recorded 2026-11-19T16:22:16-05:00",
with its open items still open.
The evidence,
the probes,
and the rejected options live in
[`doc/planning/cli-git-zfs-worktree-design-proposal.md`](../planning/cli-git-zfs-worktree-design-proposal.md).
The execution record is
[`doc/handover/cli-git-rust-implementation.md`](../handover/cli-git-rust-implementation.md).
The place the implementation interface will live is `package/git-policy/cli/SPEC.md`.

## Decision

The project accepted the ZFS worktree route for the Git wrapper, answering "Accept now (Recommended)":
the design is accepted with its open items still open.
The design keeps the checkout at its existing path and uses account-owned slots over their Git registrations.
The project chose "ZED, Clones, reflink. There is a reason we have a doctor subcommand."
The Git wrapper owns Git validation and registration,
unprivileged snapshot and clone creation,
the removal workflow and the same-dataset reflink fallback.
The root-installed ZED handler and its lifecycle unit own clone mount and unmount,
not Git or automatic deletion.
The account executes managed clone destruction under delegated `destroy`,
answering "Delegated destroy (Recommended)".
A normal authorized removal destroys the clone and its own snapshots in that invocation,
answering "B: Destroy at once".

Three further answers of 2026-11-19 settle what the design left open:

- The explicit cancel command, "Keep the command (Recommended)".
  A guarded `cancel-worktree-removal <slot>` stays as the manual way back,
  including for a removal whose wrapper was killed after it recorded the unmount or that ran past its deadline.
  It reports any snapshots already destroyed and cannot restore them.
- The deferred-destroy check, "Also check first (Recommended)".
  The check that the origin is not marked for deferred destroy also runs before the first snapshot delete,
  so such a removal refuses before this run deletes anything.
  The check before the head destroy stays, and a mark set between the two remains the accepted residual.
- The never-end set, "Spare the caller (Recommended)".
  The removal never ends the caller and the processes of its session,
  the shell that ran the command and that shell's other jobs.
  When the caller's session holds the clone, the removal refuses and says to leave the folder and run the command
  again.
  Other holders are still ended, as "Kill or otherwise remove or destroy the holders whenever --force is specified"
  says.

## What still gates shipping

The two owed VM measurements still gate shipping,
and they decide only the sentences already conditional on them:
forcing the removal of a clone a holder keeps,
and the refusal of a removal whose origin is marked for deferred destroy.
Both are owed under
[Forced removal and deferred origins][owed]
in the proposal.
The proposal states each part the decisions leave without a design,
and most of those parts as open items; this record does not copy them.

[owed]: ../planning/cli-git-zfs-worktree-design-proposal.md#forced-removal-and-deferred-origins
