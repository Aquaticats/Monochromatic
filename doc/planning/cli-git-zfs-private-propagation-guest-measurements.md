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

The arm's own lines, from the run that passed (`test result: ok. 1 passed`, 46.92 s):

```text
PROPAGATION fixture guest=zfs-r6-propagation pool=propagation checkout=/root/propagation-checkout
PROPAGATION state=shared before=shared outcome=applied after=private instance=4294971982
PROPAGATION state=private before=private outcome=already-private after=private instance=4294972009
PROPAGATION state=unbindable before=unbindable outcome=left after=unbindable instance=4294972036
PROPAGATION state=shared+slave before=shared+slave outcome=applied after=private master=302
PROPAGATION refused=source receipts=0 outcome=refused
PROPAGATION trigger arming=[replacement=arming seen=true starts=1 reloads=1 propagation=shared waited=106]   seen=10/10 quiet=[starts=0 reloads=0]   burst=[replacements=8 starts=3 reloads=1 limits=upholds-16-per-10s,service-5-per-10s,table-5-per-1s]
PROPAGATION done
```

What each line holds:

- A shared checkout is set private through the descriptor the step verified,
  and the instance's unique number is the same before and after,
  so the set changed that mount and no other.
- A private checkout is reported `already-private`,
  and the step's acts stop before the set: the setter is never called.
- An unbindable checkout is reported `left`,
  and its set is the one it had.
- A shared and slave checkout is set private,
  and the master the set detached is in the record,
  which is the cost the project stated with its answer.
  The set is read on the copy the kernel delivers into a namespace whose root
  is a slave: this kernel delivers that copy as a `slave`,
  and the arm makes it shared,
  which keeps the master and gives the set the design measured as
  `shared,slave`.
  A mount made under a slave parent is `private` in this kernel, which is why
  the two namespaces make one phase.
- A mount of another dataset at the checkout path is refused at the source
  check,
  the mount's set is what it was,
  and the receipts folder holds what it held.
- The trigger's readings are the two rules' material, below.

## The two rules applied

The design's "seen" rule
("one start of the propagation unit and one completed reload of the boot unit,
with the checkout private when that finishes, inside the reading window")
holds for every replacement:

- Ten replacements at a person's pace were each seen once:
  one start of `checkout-propagation.service` and one reload of the gate,
  inside the 10 s window,
  the first within 110 ms of the mount.
- The step was run on the instance each replacement left,
  and the checkout read `private` afterwards every time,
  so the half of the rule that speaks of the checkout's state is read on the
  same instance.
  The reconcile itself did not run the step: this release's gate does not call
  it, and the report says so.

The design's "loops" rule holds:

- No replacement produced more than one start.
- Over the 20 s quiet interval after the last replacement,
  no start and no reload arrived.
- No start happened that no mount-table change preceded: the counts rise only
  at a replacement.

The limits, which the design says describe the cost and are not a pass or a
fail, sat here:

- Eight replacements inside one burst produced three starts and one reload:
  the upheld queue's sixteen starts in ten seconds and the service's own five
  in ten seconds held the rest back,
  and systemd's mount-table limit (five events in one second) is the third
  bound those replacements meet.

The gate's own log carries thirty `reload` lines for the run:
ten paced replacements, the arming replacement,
and the burst's own share.

So the reading is: **the provisional systemd trigger stays.**
That is the reading and not the decision:
the trigger is dropped or kept by the project,
and this record states what the run found.
