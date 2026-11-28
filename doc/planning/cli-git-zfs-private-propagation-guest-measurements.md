# The private-propagation step's guest half and the provisional trigger's measurement

Status: unapproved and non-final;
written by the round that built the step
(`package/git-policy/cli/src/native/checkout_propagation/`),
which the owner has not read and the project has not accepted.
Nothing here is a decision:
choice V8 of
`doc/decision/cli-git-zfs-checkout-propagation-repair.md` (`:163` to `:167`)
leaves the two rules to a run and the trigger's fate to the reading,
and the reading is reported to the coordinating session,
which is where the trigger stays or is dropped.

## What was measured, and how

The arm is
`package/git-policy/cli/src/native/checkout_propagation/propagation_guest_arms_tests.rs`,
driven by `package/git-policy/cli/e2e/propagation-step-guest.ts`,
in one throwaway `mvm` guest of the ZFS binding's recipe
(Ubuntu 24.04, OpenZFS 2.4.4, kernel 6.8).

Two halves, on one guest and one checkout dataset:

- **The step's kernel half.**
  A file-backed pool,
  a checkout dataset mounted at `/root/propagation-checkout`,
  and one phase per state the kernel can hand the step:
  `shared`,
  `shared+slave`,
  `private`,
  `slave`,
  `unbindable`,
  and a mount of another dataset at the checkout path.
  Each state phase runs in a mount namespace of its own,
  drives `run_the_step` as root,
  and compares the record's `before` and `after` with `findmnt`'s own answer and
  the mount's unique number before and after the run.
  No root mode calls the step in this release,
  so the arm calls it itself.
- **The trigger's measurement.**
  The gate and the propagation pair are the texts the release code builds
  (`checkout_boot_units::propagation_service`,
  `propagation_mount_drop_in`),
  with one stand-in:
  the gate's `ExecStart` does nothing and its `ExecReload` appends one line to
  `/root/propagation-gate.log`,
  because this release's gate runs `boot-reconcile`,
  which does not call the step yet.
  The arm starts the gate,
  then replaces the checkout's mount at a person's pace
  (`zfs unmount`, a pause of the length a person types in, `zfs mount`),
  reads whether the mount unit's activation started the propagation service
  once and reloaded the gate once inside the reading window,
  waits a quiet interval for a loop,
  and then bursts replacements to read where the three limits sit.
  After each replacement it runs the step on the instance the replacement
  left,
  so the half the design's "seen" rule states second (the checkout reads
  private when the reconcile finishes) is read on the same instance,
  and the report never claims the reconcile itself made it private.

## The counts, stated before the run

Stated 2026-11-28T00:13:23-05:00,
before any part of the arm had run:

- **Ten replacements at a person's pace** for the "seen" rule,
  which is the count the design's own example states
  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
  choice V8's example).
- **A reading window of 10 s** after each replacement,
  inside which the one start and the one reload must appear.
- **A pause of 2 s** between the unmount and the mount of one paced
  replacement,
  which is the length a person typing the next command takes.
- **A quiet interval of 20 s** after the last paced replacement,
  over which a loop would show itself.
- **Eight replacements inside one burst** for the limits reading,
  which is a flap the three limits are read under.

## The readings

(To be filled from the run's `propagation-arm.log`.)

## The two rules applied

(To be filled: whether every replacement was seen, whether anything looped, and
where the three limits sat.)
