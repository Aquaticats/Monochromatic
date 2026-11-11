# A `SIGKILL`ed process in uninterruptible sleep on Linux 6.x does not exit, and its parent waits for it

FOUND 2026-11-09 while reviewing the scratch `git commit` ceiling of
`package/git-policy/cli` (`commit/native_commit_waiting_notice.rs`).
BUILT 2026-11-10:
the owner chose to keep waiting and say so
("Keep waiting and say so"),
so the command now says,
once and again for as long as the wait lasts,
that the stop was sent
and the kernel has not released the process it is holding,
and keeps waiting.
AMENDED 2026-11-10 after the review of `62994776ec`
(`/var/home/user/temp/agent/kimi-reviews/stuck-step-kernel-notice-1/review.md`):
the wait that lasts says the line and not the notice's thread,
which a process at its thread limit never starts,
a look that could not be made is made again,
a line the writer could not take is said again,
and the line names the process the kernel is actually holding.
The other answers below stay unbuilt,
as the record of what he chose over.

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

Since 2026-11-10 the notice's lines stop with a line that names the commit and the
process the kernel is holding,
says the stop was sent and the kernel has not released it,
and says nothing of the person's is at risk;
the wait that keeps waiting says it again at every notice interval,
and the command still returns only when the kernel lets the step go
(the owner's choice under "Verified workarounds").
The silence above is what a person saw before that change.

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
    and the commit's thread then waits for the child on its pidfd
    (`process/child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`),
    reached from
    `commit/commit_preparation.rs` (`run_guarded_child`,
    through
    `SignalGuard::run_child_telling_its_number`).
    A reap returns when the child terminates,
    so the commit's thread waits
    for the kernel operation even though the step was signalled long before.
    Until 2026-11-10 that thread was inside
    `SignalGuard::run_child`
    (`process/signal_relay.rs`,
    which ends in `child.wait()`);
    the pidfd wait replaced it,
    and the wait is now the owner's choice rather than an oversight.

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
The first is what the wrapper does since 2026-11-10:
the owner chose it
("Keep waiting and say so")
over the second,
and the third was rejected.

1.  **Say what is happening,
    keep waiting.**
    BUILT 2026-11-10,
    and amended after the review of `62994776ec`.
    After the ceiling's signal,
    the wait that owns the step says a line that names the commit it stopped and the
    process the kernel is holding,
    says the stop was sent and the kernel has not released it,
    names the state the kernel reports (`D`,
    which the notice already
    counts as `blockedLooks`),
    says nothing was committed and that the person's index and files are as they were,
    and says the terminal stays until the kernel lets go
    (`commit/scratch_step_held_by_the_kernel.rs`).
    The wait says it and not the notice's thread
    (`process/wait_after_the_stop_saying_the_kernel_holds_it.rs`),
    because a process at its thread limit starts no notice thread and such a commit
    then waited in silence;
    it says the line again at every notice interval until the step is reaped,
    because a person who reads it once and then sees nothing reads a hang;
    a look that `/proc` refused is made again and never read as the kernel having
    let go;
    a line the writer could not take is handed over again as soon as it can take one;
    and the process the line names is the one the look found in uninterruptible sleep,
    which is a program the step started when that program is the one held.
    The line is said only of a process that a second look,
    one look interval after the signal,
    still reports in uninterruptible sleep,
    so one that acts on its stop in that interval is never said to be held.
    - Tradeoff:
      the person's wait is unchanged;
      only the words change.
      The record gains nothing it does not already count.
    - This is the smallest honest change and breaks nothing,
      and it does not satisfy "nothing should ever wait forever";
      the owner made it an exception to that rule.
    - No control on this host can produce a process in uninterruptible sleep
      (it takes a stalled disk,
      a dead network file system or a stopped file-system server,
      and the gate's runs mount nothing),
      so the line is proven with a process a control states the kernel holds,
      and a step a control states has not been reaped
      (`process/wait_after_the_stop_saying_the_kernel_holds_it_tests.rs`:
      `a_wait_whose_notice_thread_was_refused_still_says_the_kernel_holds_the_step`,
      `the_line_is_said_again_for_as_long_as_the_wait_lasts`,
      `a_look_that_could_not_be_made_is_made_again`,
      `a_line_the_writer_could_not_take_is_said_again_when_it_can`,
      `a_helper_held_below_the_step_is_the_process_the_line_names`).
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
      He chose the first over it (2026-11-10).
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
