# The ZFS route's host setup, real-pool measurement and image build are the agent's act

Recorded 2026-11-26T01:20-05:00.
The owner decided this on 2026-11-26,
answering three questions put through the question tool.

## Why it was asked

The checkout migration of plan steps 12,
18 and 19 was already the agent's act by the owner's answer of 2026-11-24
(`cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
"Who runs the checkout migration").
Every other change the route makes to this machine was kept for the owner
by the designs' own wording ("the owner's act on the owner's authorisation, never a step's"),
and no recorded failure stood behind that wording.
The owner asked why those steps were theirs when the agent could do them.

## The host setup

The question named the `zfs allow` grants for the account,
the boot and shutdown units,
the ZED hook in `/etc/zfs/zed.d`,
the immutable flag on the root-mode entry,
`sanoid.conf`,
and the snapshot properties on other datasets.
The owner answered "You apply, logged (Recommended)".

So the agent runs each with sudo once its step is wired and proven in a guest,
and records the exact command,
its output and how to undo it in that step's record.
`doctor` still prints each command,
so a person on another machine can run it themselves.

## The real-pool measurement and the image build

The question named step 22's storage half
(clone adds and removals measured on the real pool after the migration,
read-only measurements first)
and step 23's custom CachyOS guest image,
whose build may need loop devices or `pacstrap` on the host.
The owner answered "You run both (Recommended)".

So the agent runs both,
logging what each touched;
the measurement creates and destroys only worktree datasets the route itself made.

## The moments everything stops

The migration runs while no process has the checkout open,
which means every Claude Code and Pi session is stopped,
and the boot units' last control needs this machine rebooted.
The owner answered "You pick, I prepare (Recommended)".

So before each,
the agent commits and pushes everything,
checks that no other session is mid-work,
and tells the owner the moment is ready;
the owner chooses it,
and the agent runs the migration on the owner's word.
A reboot stays the owner's to choose,
because it ends every session and the desktop,
and because a reboot on this machine's SSD can make uncommitted data unreadable.

## What this changes

The plan's steps 19,
20,
22 and 23 said these were the owner's act;
each now cites this record.
