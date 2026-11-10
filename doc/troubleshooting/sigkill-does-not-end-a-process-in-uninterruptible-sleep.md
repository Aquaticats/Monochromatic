# A `SIGKILL`ed process in uninterruptible sleep on Linux 6.x does not exit, and its parent waits for it

FOUND 2026-11-09 while reviewing the scratch `git commit` ceiling of
`package/git-policy/cli` (`commit/native_commit_waiting_notice.rs`).
NOT FIXED:
the honest answers need an owner decision,
and each is listed
under "Verified workarounds" with what it does to a person's commit.

## Symptom

A `git commit` the wrapper runs in its scratch repository reaches its ceiling.
The notice sends `SIGKILL` to the step,
records that it stopped it,
and ends.
The command still does not return.
Its notice lines stop,
so the person sees the last line saying the step
"is stopped if it lasts N seconds" and then nothing,
for as long as the kernel operation the step is inside takes.

The person-visible shape is the one `AGENTS.md` names in `NHP`
("a process killed at any instruction") followed by silence,
and it is the one state in which this wrapper's own ceiling cannot bound its
own wait.

## Root cause

Three facts,
in order,
and the call chain that joins them.

1.  A signal is only acted on when a process returns from kernel mode.
    `signal(7)`,
    section "Execution of signal handlers":

    ```text
    Whenever there is a transition from kernel-mode to user-mode
    execution (e.g., on return from a system call or scheduling of a
    thread onto the CPU), the kernel checks whether there is a pending
    unblocked signal for which the process has established a signal
    handler.
    ```

    A process in uninterruptible sleep (`D`) is inside a kernel path that has
    not returned,
    so it cannot make that transition,
    and `SIGKILL` stays
    pending.
    `signal(7)` also states the part that does not help here:
    "The signals SIGKILL and SIGSTOP cannot be caught,
    blocked,
    or ignored."
    Not being blockable is not the same as being acted on immediately.

2.  The state letter the wrapper already reads means exactly that.
    `package/git-policy/cli/src/native/process/process_tree.rs`
    (`in_uninterruptible_sleep`) documents it as the state "held by a stalled
    disk call or a processor it is not given",
    and the module that acts on it
    names the consequence:
    `package/git-policy/cli/src/native/process/stop_waits_while_a_member_cannot_be_interrupted.rs`
    says a member in that state "cannot act on a signal until the call
    returns".

3.  The wrapper waits for the reaped child,
    not for the signal.
    `commit/native_commit_waiting_notice.rs` sends the `SIGKILL` from its own
    thread and returns its facts,
    and the commit's thread is inside
    `SignalGuard::run_child`
    (`process/signal_relay.rs`,
    which ends in `child.wait()`),
    reached from
    `commit/commit_preparation.rs` (`run_guarded_child`).
    `wait(2)` returns when the child terminates,
    so the commit's thread waits
    for the kernel operation even though the step was signalled long before.

The wrong reading this replaces,
from the review that found it:
"the ceiling bounds the wait,
because it sends `SIGKILL`".
It bounds the wait for a step that can act on a signal.
Under this host's normal load (`NHR`:
a stalled disk,
memory pressure)
a Git in `D` cannot.

## Verification

Version:
Linux 6.17 on this host,
`signal(7)` from man-pages 6.19
(2026-02-08),
and the crate at
`package/git-policy/cli` on branch
`fix/git-policy-cli-scratch-commit-ceiling-and-three-slow-commits`.

What was run:

- The crate's own controls for the notice,
  including the one that shows a
  step being signalled and reaped
  (`mise run //package/git-policy/cli:native:test:host -- native_commit_waiting_notice`).
  They prove the signal is sent and the child ends when it can.
- Reading the three code paths above.

What was **not** run,
and what it would need:
a harness that puts a process into `D` on demand.
That needs a filesystem or network operation that never returns
(an `NFS` mount with a dead server,
a `FUSE` mount whose daemon is stopped,
or a block device with an error timeout that outlasts the test),
and none of those can be made on this host or in the gate container,
whose filesystems are local and whose runs are mount-free (`BOX`).
So the claim rests on the documented kernel behaviour and on the crate's own
code,
not on a reproduction here.

## Verified workarounds

Each is a change to the wrapper,
not to the kernel.
None is built:
the first two change what a person's commit does,
so they are
the owner's call.

1.  **Say what is happening,
    keep waiting.**
    After the ceiling's signal,
    the notice's line states that the step was
    stopped and that the kernel has not released it yet,
    naming the process and its state letter (`D`,
    which the notice already
    reads as `blockedLooks`).
    - Tradeoff:
      the person's wait is unchanged;
      only the words change.
      The record gains nothing it does not already count.
    - This is the smallest honest change and breaks nothing,
      and it does not satisfy "nothing should ever wait forever".
2.  **Leave after a bound,
    with the child unreaped.**
    The commit's thread stops waiting at the ceiling plus a grace,
    prints the stop's words and exits nonzero,
    leaving a process that is already `SIGKILL`ed to be reparented and reaped
    by init when the kernel releases it.
    - Tradeoff:
      the wrapper's exit no longer waits for what it killed,
      so the
      scratch directory can be removed while a killed process still has it
      open (its files are unlinked;
      the process dies on return).
      Nothing of the person's is in the scratch step,
      so nothing of theirs is
      at risk,
      and the next build writes the objects again.
    - This is the only option that bounds the wait,
      and it changes the exit
      path of a failed commit:
      it needs the owner's decision.
3.  **No ceiling at all for this step.**
    - Tradeoff:
      rejected.
      It is the state that let a 36,651 ms step land under
      a 3,006 ms ceiling (2026-11-08),
      which is what this ceiling exists to
      stop,
      and `AGENTS.md`'s rule is that nothing waits for ever.

## What does not work

- Sending `SIGKILL` earlier or twice:
  neither changes when the kernel can
  act on it.
- `SIGTERM` before `SIGKILL` for this step:
  the step is known to have
  nothing of the person's in it,
  and a handler it installs cannot run while
  it is in `D` either.
- Killing the process group:
  the process is still in `D`,
  so the same wait
  applies to every member.
- A pidfd and `poll` on it:
  the descriptor is readable when the process
  exits,
  which is the event that has not happened.
  (The pidfd is still the
  right tool for **which** process is signalled,
  which is a different
  problem:
  `process/process_tree.rs`,
  `PinnedMember`.)

## Upstream filing decision

Nothing to file.
The behaviour is the kernel's documented design (`signal(7)`,
`wait(2)`),
not a defect,
and no upstream project can bound a wait on an operation the
kernel has not finished.
This doc is the durable record for this repository;
the decision the wrapper needs is internal and is listed under
"Verified workarounds".
