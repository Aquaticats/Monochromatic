# The ZFS route builds later pieces ahead, unwired, with up to 64 rounds at once

## Status

Decided by the owner on 2026-11-23,
through the question tool,
answering the session's question "Can we parallelize better?" put back to the owner as two choices.
Recorded by the session;
this record holds the owner's words and what follows from them.

## Building later pieces ahead of the plan's order

### The question

Step 10's second half (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 10") is the one round every later
step waits on,
directly or through another step's "Depends on".
Some later pieces need only work already on `main`:
step 15's deferred-origin check,
step 16's one-request snapshot delete,
step 12's refusal of `git worktree move` and its report on an unmounted clone,
step 13's distinction of the wrapper's own Git lock from one a person set,
and `doctor`'s checks for steps 2 to 9.
The question asked whether to build those now,
in parallel and unwired,
with step 11 kept after step 10 by the owner's earlier "Ship step 10 first (Recommended)".

### The answer

"Build them ahead, unwired (Recommended)"

### What it means

A piece built ahead merges as code no person's command reaches.
Its own step wires it in,
in the plan's order,
and that wiring is where the step's "Depends on" still holds.
Each piece gets its own round,
its own guest test,
and a control that is red without its guard.
A piece whose reading part depends on a record step 10's second half still changes redoes that part when step 10 lands;
that cost was stated in the question.
Step 11 stays after step 10.

## Rounds at once under the worktree hold

### The question

The owner's hold of 2026-11-18,
"We shouldn't use worktrees heavily before we implement the zfs parts for the git wrapper."
(`doc/decision/heavy-worktree-use-waits-for-the-zfs-route.md`),
left how many worktrees count as heavy unsaid.
The question offered three more rounds at once,
six more,
or none more,
beside the seven running then,
and gave the measurements behind each:
a pool with 2.4 TB free,
about 6 GiB per round's worktree,
26 GiB of memory available,
build-and-guest rounds peaking near 7 GiB and design rounds near 1 GiB.

### The answer

The owner wrote in their own option:
"Up to 64 rounds at once."

### What it means

Up to 64 Pi rounds may run on this machine at once,
counted across sessions,
each with its own worktree.
Each provider pool's own limit still bounds the rounds on it
(`package/pi-shared/subagent-driver/src/pool-limits.ts`),
so the pools open on 2026-11-23 hold fewer than 64 between them.
Memory is watched as rounds are added:
the freezes recorded in `doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md` came from memory,
not from the number of rounds,
so a rising memory stall pauses new starts,
and the readings go into that document.
