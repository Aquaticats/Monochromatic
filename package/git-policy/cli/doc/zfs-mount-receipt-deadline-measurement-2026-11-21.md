# The clone-mount receipt path, measured 2026-11-21

This measurement is unapproved and non-final.
It was taken by an agent on 2026-11-21 in one disposable `mvm` libvirt guest.
Only the owner declares a value settled.

## What was measured

The plan's step 7 measurement
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 7:
The intent property,
the receipts and the readiness record",
"VM measurements") is item 5's property event to a test zedlet,
extended to the product handler and the product receipt.
This document records the extension it could take and the one it could not.

One property change on a clone is a ZFS history event.
The rounds time three points after the `zfs set` returns:

- the test zedlet's own arrival,
  which ZED delivers,
  read from the line the zedlet appends;
- the product receipt,
  written by `ReceiptsFolder::publish` after the arm read the clone's intent
  through the binding's `objset_stats` and the product `read_intent`;
- the product wait's answer,
  `wait_for_answer`,
  which reads that receipt.

The times are `Instant::elapsed` readings around the calls,
in microseconds,
interleaved in one test with the arms above it.

## What the deadline is and is not sized from

`PROVISIONAL_MOUNT_REQUEST_DEADLINE` in
`package/git-policy/cli/src/native/worktree_clone_mount_protocol/mount_request_wait.rs`
stays 30,000 milliseconds.
The plan leaves this value to step 22,
so it is provisional and the code says so.
The rounds below are context for it and not its derivation:
no round in this document exercised a product handler that attaches a mount,
because the attaching sweep is step 8's,
and the receipt here is written by the product's receipt code from the test
rather than by a handler mode.
The provisional value is therefore not read as a measured latency finding.

## Where

The throwaway `mvm` libvirt guest of the ZFS binding's recipe,
Ubuntu 24.04,
kernel `6.8.0-147-generic`,
module and userspace OpenZFS `2.4.4-1`,
pool `zfsstep7` on a 1 GiB encrypted file-backed image,
clone parent `zfsstep7/clones` with `mountpoint=legacy` and `canmount=off`,
and clone `zfsstep7/clones/one`.
A real `zfs-zed` ran with one test zedlet in the prefix's
`etc/zfs/zed.d` (`/opt/openzfs-2.4.4/etc/zfs/zed.d/history_event-step7_probe`,
because this guest builds OpenZFS with `--prefix`).
It appends its own arrival time and the event's `ZEVENT_CLASS`,
`ZEVENT_HISTORY_DSNAME`,
`ZEVENT_POOL` and `ZEVENT_HISTORY_INTERNAL_STR`.
A `zfs set` is an internal history record,
so the property and its value are in
`history_internal_str` and `ZEVENT_HISTORY_CMD` is empty;
the arm matches the nonce inside that string,
so only the clone's own property change starts a round.

## The rounds

Four runs of
`worktree_clone_mount_protocol::mount_protocol_arm_tests::mount_protocol_arm_intent_receipt_and_nudge`
on the guest above passed on 2026-11-21,
each printing one round;
the kept logs are `keep/guest/step7-suite.log` and
`keep/rounds/round-2.log`,
`round-3.log` and `round-4.log`.

- Round 2:
  event to the test zedlet 10,581 microseconds,
  event to the product receipt 11,482,
  event to the product wait's answer 11,523.
- Round 3:
  zedlet 10,529,
  receipt 12,113,
  answer 12,149.
- Round 4:
  zedlet 10,218,
  receipt 11,171,
  answer 11,207.

The first run of the arm took no usable round:
it printed `event-to-zedlet-us=24` because the helper measured from its own
first poll rather than from the property change.
That reading was wrong,
the helper now takes the round's start from its caller,
and the run was discarded rather than recorded.

The three kept rounds put the test zedlet's arrival 10,218 to 10,581
microseconds after the property change was issued,
the product receipt 11,171 to 12,113 microseconds after it,
and the product wait's answer up to 36 to 64 microseconds after the receipt,
which is one poll at most.
No round sizes the deadline,
and the provisional value stands on the plan's assignment of it to step 22.

## The acceptance arms

The same test covers every step 7 arm in one run:
a malformed local intent,
an oversized one,
an inherited one,
each followed by checks that the clone is not mounted and the slot folder is
empty;
a stale receipt that the product wait refuses;
the nudge delivered to the delegated account with only
`mount,userprop` on the parent's descendants and refused on the parent itself;
and the event-to-receipt round above.
It passed in the four runs,
and one earlier run failed only at the event step
because the zedlet logged `ZEVENT_HISTORY_CMD`,
which the kernel leaves empty for a `zfs set`;
its suite log is `keep/guest/step7-suite.log` from before the correction and the
failure is recorded here.

## Limits

- The guest's pool is a file-backed image,
  so these numbers are the guest's semantics and not this host's storage.
- The handler-side work in a round is the product's property read,
  intent read and receipt write called from the test.
  It is not the sweep that step 8 adds,
  so the round does not measure an attach or the handler's own process start.
- ZED's delivery is one observed line,
  and the round does not measure a dropped event or a replayed queue.
- The wrappers' wait polled at 50 milliseconds,
  so the answer time includes up to one poll interval.
- Per-round host load was not measured;
  the rounds ran in one guest with no other arm in flight,
  because the recipe runs the suite one test thread at a time.
