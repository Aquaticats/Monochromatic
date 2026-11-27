# Step 10's reach, plan text and grant check, and when the checkout migrates

Status: accepted by the owner on 2026-11-24T14:10:00-05:00, through the question tool.
The proposal these answers settle is `doc/planning/cli-git-zfs-step-10-open-choices.md`,
revised and merged as `dc0635971e`.
The questions were built from a same-vendor read of that revision
(`~/temp/agent/gate-review-zfs-step-10-open-choices-revision-2026-11-24/keep/read-of-the-step-10-open-choices-revision.md`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here)),
whose corrections they carry.
That read's owner blocks put each choice in plain words,
with its options best first,
and carried the two facts every reach option meets,
stated under the heading above.

## What the owner was told first

- On this machine the checkout is a subtree of the shared home dataset,
  so no reach option serves it until the checkout becomes its own dataset.
- The route has no removal yet for the worktrees it makes (plan steps 14 to 16),
  so any wider reach is worth turning on only once removal exists.

## The answers

### Choice 1, which checkouts the route serves

Answer:
"E: a root command records it (Recommended)".
A root command records the checkout's path,
dataset and GUID,
container and pool,
and the add serves the checkout that record names,
refusing a dataset swapped in under the same name.

### Choice 2, what the add prints beside a standing removal plan

Answer:
"2C: read the plan and the pool (Recommended)".
Once removal plans exist,
the add reads the plan's head word and whether a child of the container still carries the clone,
and prints the one command that ends the plan,
the finish-pass wait line where the pool cannot prove the clone gone,
and the cancel command when nothing has started.

### Choice 3, which grants the add checks before it claims

Answer:
"3C: refuse on six, warn on three (Recommended)".
The add refuses when one of the six permissions it uses is missing,
and warns,
naming the `zfs allow` command,
when one of the other three is missing.
The split moves at plan step 11,
when the add's own failure path starts using both `destroy` grants.

### Who runs the checkout migration

The owner was told that the agent has passwordless sudo and can run every command,
that the accepted records make applying ZFS changes to this machine the owner's act,
and that every session works inside the checkout the migration moves.
The owner's first reply was a question:
"Need my hands? But you have passwordless sudo."
Asked again,
the owner answered:
"You run it whenever you feel convenient."

So the agent runs the migration of plan steps 12,
18 and 19,
and step 19's `canmount=noauto` change on `zpcachyos/nosnap/vm-builder-output`,
itself with sudo,
at a time it chooses.
It chooses only a time when no process has the checkout open,
because every Claude Code and Pi session works inside it.

### When the migration happens

Answer:
"After removal exists (Recommended)".
Plan steps 14 to 16 land before the migration runs,
so every worktree the route makes on this machine can be removed with the wrapper from the first day.

### What the owner already decided, adopted without a new question

A dirty main checkout on the route takes rung 3 until step 11 lands,
as the owner decided on 2026-11-19
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, lines 1858 to 1862);
the code refused it instead,
which is issue 6452.

## State

The proposal is rewritten to these answers next,
and the plan's order gains step 19 after steps 14 to 16.
