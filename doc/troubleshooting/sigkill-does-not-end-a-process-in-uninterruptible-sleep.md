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
AMENDED 2026-11-12 on
`fix/git-policy-cli-commit-does-not-fail-when-its-scratch-commit-stalls`:
the ceiling now stands aside for a held step instead of ending it
(workaround 1 below),
so on 2026-11-11 an ordinary commit no longer fails in a disk wait,
and the line about a stop the kernel has not released is said once.
The other answers below stay unbuilt,
as the record of what he chose over.
AMENDED 2026-11-12 again,
after the review of `0076cb74ed`
(`/var/home/user/temp/agent/reviews/commit-does-not-fail-when-its-scratch-commit-stalls/review.md`):
the stand-aside is one decision the two enforcers of the ceiling share
(`commit/scratch_step_stall_decided_once.rs`),
a look that could not be made decides nothing and is made again
instead of being read as "not held",
the words of a stop are taken whole from the enforcer that ended the step
(`commit/scratch_step_stop_record.rs`),
and only the wait that owns the child prints the line about the wait,
so it is said once.
Its three findings were reproduced in the crate at `0076cb74ed`,
and the controls that hold them are named under workaround 1.

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
the wait that keeps waiting says it once,
and the command still returns only when the kernel lets the step go
(the owner's choice under "Verified workarounds").
Since 2026-11-12 the step is not stopped at all while the kernel holds it:
the ceiling stands aside for it,
one line says so and names the bound it now runs under,
and only a step the kernel still holds past that bound is signalled,
so the state this doc describes is one a commit reaches only after it has been
waited for far longer than any disk on this host took.
The silence above is what a person saw before either change.

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

One arrangement **was** measured on 2026-11-12,
because it needs no privilege and no mount:
the cgroup v2 `io` controller on this host is delegated to the user manager,
and a scope that carries
`IOWriteBandwidthMax=/dev/mapper/crypt_sda 102400`
(a property `systemd-run --user --scope` accepts)
puts every write of that scope to the encrypted volume at 100 KB/s.
`dd if=/dev/zero of=<file> bs=4096 count=2048 oflag=direct` in such a scope
was in state `D` in `/proc/<pid>/stat` for the whole 11.4 seconds it was sampled,
so a true disk wait is reachable for a bounded time,
only the scope's own I/O is throttled,
and no other command on the host slows down.
That arrangement does not reach the scratch `git commit` of a real commit:
the step runs inside the commit *service* process
(`landing/service_native.rs` calls `run_native_commit`),
not in the command that asks for the commit.
The command's own timing record says so
(`scratchSteps` is empty for a commit the service landed),
and the service is a unit of its own,
so throttling the invoking command's scope throttles none of the step's writes.
Throttling the service unit's cgroup would reach it,
and that was not done:
on this host every repository's service is a unit of the same manager,
the probe's unit could not be told apart from another repository's by anything in
its unit text,
and a limit written in the wrong one slows another session's commit.
The gate container could make the same arrangement reach the step,
by running the wrapper and its service in one throttled cgroup,
and it has not been tried there.

## Verified workarounds

Each is a change to the wrapper,
not to the kernel.
The first two are what the wrapper does:
the second since 2026-11-10,
the owner's answer
("Keep waiting and say so"),
and the first since 2026-11-11,
because a step the kernel holds is slow and not stuck.
The third was the owner's answer to the wait he then had,
and the fourth was rejected.

1.  **Stand aside at the ceiling for a step the kernel holds,
    and wait for it to a measured bound.**
    BUILT 2026-11-11 on
    `fix/git-policy-cli-commit-does-not-fail-when-its-scratch-commit-stalls`.
    A step in an uninterruptible call is waiting for the disk and not stuck in its own work,
    so the ceiling it reached is not what should end it:
    on 2026-11-11 two ordinary commits failed that way in this repository,
    their steps held for 37,153 and 47,978 ms under a 16,332 ms ceiling,
    and the second one landed when the same command was run again.
    Both enforcers of the ceiling read one decision about the step
    (`commit/scratch_step_stall_decided_once.rs`),
    so neither can end it at the usual ceiling after the other has stood aside,
    and the first stand-aside keeps its bound;
    a look that found the kernel holding a process of the step makes that decision
    (`commit/scratch_step_look_below.rs`,
    `what_the_look_says`),
    one that found none has the usual ceiling end the step,
    and one that could not be made decides nothing:
    the ceiling stays armed, the look is made again,
    and a step nothing can be learned about is ended at the bound it is waited for
    instead of at the usual ceiling;
    the ceiling stands aside once for a held step,
    and the step is waited for to `SCRATCH_COMMIT_STEP_STALL_CEILING`
    (`service/wait_ceilings.rs`,
    300,000 ms,
    five times the ceiling and 6.2 times the slower step the kernel ever held here),
    which an ordinary slow disk never reaches.
    One line,
    said by the wait that owns the step and by no other enforcer,
    says the ceiling stood aside,
    how long the step has lasted,
    the process the kernel is holding and the bound the step now runs under;
    the timing record keeps the moment (`heldAfterCeilingMs`) and the bound
    (`stallCeilingMs`),
    so a slow step that landed is told from one that was stopped;
    a step the kernel holds past that bound is ended as any step at its ceiling is,
    and the stop's words say it was already waited for;
    the words are taken whole from the enforcer that ended the step
    (`commit/scratch_step_stop_record.rs`),
    and name the bound that really ended it, never one a ceiling stood aside for.
    Another bound is never needed:
    no commit is built twice,
    no scratch state of a stopped attempt is left behind,
    and a step that cannot be in a disk wait (an editor waiting for input)
    is ended at the ceiling exactly as before.
    - Tradeoff:
      a step nothing will release holds the commit for up to five minutes
      instead of sixteen seconds.
      The person sees the elapsed time in every notice line,
      so the wait is never silent,
      and the alternative was a failed commit.
    - Proven by controls that state the kernel's answer about a real signer and a real step
      (`commit/commit_preparation_tests.rs`:
      `a_scratch_commit_the_kernel_holds_past_its_ceiling_lands_exactly_one_commit`,
      `a_scratch_commit_the_kernel_never_releases_is_ended_with_a_message`;
      `process/child_awaited_until_it_exits_the_stop_or_the_ceiling_tests.rs`:
      `a_step_the_kernel_holds_past_its_ceiling_lands_when_it_ends`,
      `a_step_the_kernel_never_releases_is_ended_at_the_bound_it_is_waited_for`,
      `a_step_the_kernel_holds_that_ends_after_the_bound_is_reported_by_its_own_status`;
      `commit/native_commit_waiting_notice_tests.rs`:
      `the_ceiling_stands_aside_for_a_step_the_kernel_holds`,
      `the_ceiling_that_stood_aside_prints_no_line_of_its_own`;
      `commit/scratch_step_stall_decided_once_tests.rs`:
      every control of the one decision;
      `commit/scratch_step_stop_record_tests.rs`:
      `a_stop_the_wait_ended_at_the_usual_ceiling_names_the_usual_bound`;
      `commit/commit_preparation_tests.rs`:
      `a_scratch_commit_whose_ceiling_look_failed_lands_when_the_step_ends`,
      `a_ceiling_that_stood_aside_is_not_undone_by_the_other_enforcer`),
      each of which fails with the stand-aside taken out,
      and the last three of which failed at `0076cb74ed` for the three findings of the
      review of that commit.
2.  **Say what is happening,
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
    it says the line once,
    because a person who met the same sentence three times in one failed commit read nothing
    new the second time;
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
      `the_line_is_said_once_however_long_the_wait_lasts`,
      `a_look_that_could_not_be_made_is_made_again`,
      `a_line_the_writer_could_not_take_is_said_again_when_it_can`,
      `a_helper_held_below_the_step_is_the_process_the_line_names`).
3.  **Leave after a bound,
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
      He chose the second over it (2026-11-10).
4.  **No ceiling at all for this step.**
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
