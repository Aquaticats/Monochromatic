# The ZFS add's clone inherits `mountpoint=legacy` from the container, checked before and after the clone

Status: decided by the owner on 2026-11-25T19:01-05:00, answering two questions
(issue 6462; proposal `doc/planning/cli-git-zfs-clone-mountpoint-without-the-forbidden-grant.md`, third revision `32e2e5e63c`):

- "How should the ZFS add's clone get `mountpoint=legacy`?" with "C: inherit (Recommended)".
- "Only if you chose C: where should the add check the container?" with "Both (Recommended)".

## What was decided

The add, run as the account the ZFS route is delegated to, sends no `mountpoint` property in its clone request. The clone
inherits `mountpoint=legacy` from the container, as the accepted design says (clones inherit `legacy`; the account is never
granted `mountpoint`).

The add checks the container twice:

- In the preflight it shares with `doctor`, before it writes a claim, a slot, a Git registration, a snapshot or a clone: it
  refuses a container whose `mountpoint` is not `legacy`, or whose `canmount`, `setuid` or `devices` is on, naming the
  container, the property and the required value.
- After the clone request, it reads the clone's `mountpoint` value and its source. A clone that does not read `legacy`
  inherited from the container is reported at the add with its value and source and what the add left. This read only
  reports: the add destroys nothing, and a wrong clone stays until the removal's settle step (plan step 14).

## Why

- The `mountpoint` request is refused under the design's grants: measured in a throwaway VM with the same
  `zfs clone -o mountpoint=legacy` request as the delegated account, which left no clone. The add's own binary was not
  run as the account.
- C keeps the accepted design and its list of forbidden grants, and with the preflight it refuses a wrongly set container
  before any write.
- Both is above the preflight alone because it also reports a container changed between the preflight and the clone
  request.

## Limits, stated with the decision

- If root later changes the container's `mountpoint`, every existing clone follows at once, until `doctor` or the sweep
  (root's check when it attaches a clone) notices. Read from OpenZFS's property inheritance; no arm changed a container
  after a clone existed.
- The rejected B (root's handler sets `legacy` at attach) would have given each attached clone its own value, which
  survives such a change. It was not chosen; its costs were a property write the accepted design does not give the
  handler, and a repair that lands after the clone exists.

## Options that were not chosen

- B: send no property, and root's handler sets `legacy` on each clone when it attaches it.
- A: grant the account `mountpoint` and keep the request. It breaks the design's forbidden list, and with it the account
  can point a clone where root's boot-time `zfs mount -a` mounts it (measured).
- For the check: the preflight only; the read after creation only; no check in the add.

## What follows

- The add drops `mountpoint` from its clone request, gains the two checks, and its controls show each refusal and report.
- The three other places that say a clone inherits the origin's `mountpoint` are corrected (the proposal's correction lists
  name them: a `SPEC.md` passage, the binding record design's A9, and the implementation plan's line owned by issue 6462).
- The proposal's remaining wording findings from its third gate read
  (`~/temp/agent/gate-review-zfs-clone-mountpoint-third-revision-2026-11-25/keep/review.md`) are folded in when it is marked
  decided.
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
