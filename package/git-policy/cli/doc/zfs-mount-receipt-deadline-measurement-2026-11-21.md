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
A real `zfs-zed` ran with one test zedlet,
`/etc/zfs/zed.d/history_event-step7_probe`,
which appends its own arrival time and the event's `ZEVENT_CLASS`,
`ZEVENT_HISTORY_DSNAME` and `ZEVENT_POOL`.
Only the clone's own events were counted:
the arm matches the zedlet line's dataset field against the clone it changed.

## The rounds

No round was taken on 2026-11-21.
The one thrown-away guest on this machine was `fence-leftover-guest`,
made and left running by another session of this effort,
and this run's rules are one guest at a time and never to use,
stop or remove a guest it did not make.
Every host-side control passed,
and the guest arms are written and reachable through
`mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- zfs-protocol-pool`
and `-- zfs-protocol-arm` with a prepared guest;
this section is filled by the run that executes them.
No number here sizes the deadline,
and the provisional value stands on the plan's assignment of it to step 22.

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
