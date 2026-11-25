# The ZFS route's delegation check forbids `allow` as well as the design's eight

Status: decided by the owner on 2026-11-25T18:29-05:00, answering "Should the ZFS route's `doctor` keep treating `allow` as a
forbidden permission?" with "Keep it forbidden (Recommended)".

## What was decided

The account the ZFS worktree route runs as must not hold `allow` on the checkout, the container, or their descendants,
beside the eight the accepted design already names (`mountpoint`, `canmount`, `setuid`, `devices`, `rename`, `promote`,
`receive` and `receive:append`; `doc/planning/cli-git-zfs-worktree-design-proposal.md`, the paragraph that begins "The
`mount` grant is an ioctl prerequisite").
`doctor` and the add's preflight, which run the same check
(`package/git-policy/cli/src/native/health/doctor_zfs_route/delegation_checks.rs`, `FORBIDDEN_PERMISSIONS`), refuse it, and
the refusal names the `zfs unallow` command that clears it.

## Why

OpenZFS 2.4.4's manual page for `zfs allow` says of the `allow` permission: "Must also have the permission that is being
allowed" (read on this host's `zfs-2.4.4-1`). So `allow` does not let the account grant itself a permission it lacks, and the
reason the build had written beside it ("it would let the account grant itself any of the others") was wrong.
What `allow` does give is the power to hand the account's own grants (`snapshot`, `clone`, `mount`, `destroy`, `create`,
`userprop`) to another account or to everyone. The removal's threat boundary rests on which actors can change the clones'
lineage; with `allow`, an actor other than the route's account and root can be given that power. Forbidding it keeps that set
to the route's account and root.

## Options that were not chosen

- Report it, don't fail: `doctor` names the grant and the route still runs. Not chosen: the person is told, but a second
  account can still be given destroy or clone on the route's datasets.
- Drop it, match the eight: the code matches the design's list exactly. Not chosen: nothing would then report who else can be
  given the power to change the clones.

## What follows

- The reason beside `FORBIDDEN_PERMISSIONS` is rewritten to the true one, and the refusal for `allow` names `zfs unallow`.
- The design's list of forbidden grants reads as nine, with this record as the amendment; issue 6472's "nine" is right.
