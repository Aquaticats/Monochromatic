# The ZFS removal's snapshot batch takes a request budget of 65,536 bytes

## Status

Decided by the owner on 2026-11-27,
between 21:50:32 and 21:53:29 -05:00,
through the question tool.
Recorded by the coordinating session;
this record holds the question as it was asked,
the owner's answer,
and what follows from it.

## The question

`git worktree remove` of a slot that a ZFS clone is bound to destroys the clone's own snapshots in batched requests,
and one request is bounded by a budget of packed input bytes,
`RequestBudget::of_input_bytes`
(`package/git-policy/cli/src/native/worktree_clone_snapshot_batch/request_budget.rs`).
Release code held no value for it.
That file says:

```text
the value is measured rather than picked: the plan's request budget is step 22's measurement
```

The measurement is the guest half of plan step 22,
`doc/planning/cli-git-zfs-step-22-guest-half-measurements.md`,
an agent's draft.
It says:

```text
The proposal is `RequestBudget::of_input_bytes(65_536)`, that is 64 KiB. The largest request the VM accepted was 114,756 bytes, and the 64 KiB budget is 2.3 times the 512-name request and under the largest accepted one.
```

and of all its proposed values:

```text
Each proposed value carries its citation. None is set, and the constant in code stays as it is until the owner accepts one.
```

Its figures are a guest's,
on a file-backed pool.
A removal's request is small beside the proposal
("64 names were 3,652 bytes"),
and the largest request that guest accepted was 114,756 bytes.

The question put three values to the owner,
ranked in this order.
The round that switches the removal's command path on was already running with the first in its brief,
labelled as the session's pick.

- 65,536 bytes,
  as the record proposes.
  It is under the largest request the guest accepted and far over what a removal on this machine asks;
  it comes from a guest's file-backed pool and not from this machine's.
- A smaller value,
  the size of the 512-name request the record measured.
  It lies further inside what was measured;
  a clone with many snapshots then takes more requests,
  each one more place a kill can land.
- No value until step 22's storage half measures this machine's pool.
  The bound would cite the workload it limits;
  but the storage half runs after the checkout migration,
  and the migration waits for removal to exist
  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
  "After removal exists (Recommended)"),
  so that option waited on itself.

## The answer

"65,536 bytes (Recommended)"

## What follows

- The constant is 65,536 bytes of packed request input,
  and the citation beside it names this record and the step 22 measurement.
- The branch of the round that switches the removal's command path on merges with that value.
- The other values the step 22 record proposes,
  the mount and unmount deadlines and the forced removal's waits,
  are not accepted by this answer,
  and none of them is set.
- A note of the session's own,
  not the owner's:
  the value rests on a guest's file-backed pool,
  and step 22's storage half,
  run on this machine's pool after the checkout migration,
  is where a request of that size is measured again.
