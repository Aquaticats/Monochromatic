# Process and child-lifetime route findings on `main`

Status:
audit,
2026-11-09T01:25:08-05:00.
Audited revision:
`main` at `379abd6e37`.
The reading was made through the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-process-and-child-lifetime-route-findings-on-main`
(head `1470ccd014`).
Every source file this page cites is identical on `main` and on that head:
`git diff --stat main HEAD -- package/git-policy/cli/src/native` reports only
`commit/commit_index_stat_staleness.rs` and its test sidecar,
which this route does not touch.
Linux only;
macOS and Windows branches are named where they exist and are not audited,
which the owner's "Ignore Win and mac until I say so."
covers.
No production code was changed and no test was committed.
The four controls are appended to the crate's own test sidecars,
were run through `mise run //package/git-policy/cli:native:test:host`,
and were reverted after their output was taken.

Scope:
`package/git-policy/cli/src/native/process/` (31 files,
7,088 lines,
test sidecars included),
and every place outside it that starts a program:
`invocation/`,
`push/`,
`commit/`,
`lfs/`,
`repair/`,
`repo_read/`,
`candidate/`,
`lock/`,
`landing/`,
`service/`,
`standards_review/`.

Purpose:
the owner's standing rules that this route carries are
"nothing should ever wait forever or indefinitely",
"Commits should never fail.",
and
"We should never invoke ourselves recursively...".
A reviewer of one branch found that signalling a process by its id after checking it
is racy,
and that a thread blocked writing a notice can keep a command from ever returning.
Both are shared shapes,
so every instance is looked for here.
This page answers,
for every child the wrapper starts,
what ends the wait,
what bounds it,
how a signal finds the right process,
and what a failed start,
a failed thread,
a failed descriptor or a failed look at `/proc` is reported as.
Four findings are reproduced with a seam and a bounded wait;
five are read from the code only,
and each of those says so.

## The records this route sits under

`doc/handover/cli-git-rust-implementation.md`,
section "Open review findings",
records nine open findings and eight fixes on the unmerged copy branch,
and none of them is about this route;
the four Critical findings of the commits-never-fail review are open by construction.
So no recorded finding is repeated here.

Owner decisions that these findings touch,
quoted verbatim as the handover and `SPEC.md` carry them:

- "nothing should ever wait forever or indefinitely" (2026-11-03),
  the ceiling of a step
  (`package/git-policy/cli/SPEC.md`,
  "Git steps,
  tools and checks end only on no progress").
- "End Git,
  hold hooks (Recommended)" and "Stop only on no progress (Recommended)" (2026-11-03),
  picks 4 and 9 of `doc/planning/cli-git-shipped-limits-audit.md`,
  open to the owner's veto
  (`SPEC.md:2340-2365`).
- "Never cut a branch write (Recommended)" (2026-11-04),
  which `SPEC.md:2401-2412` turns into the rule that a step keeps a ceiling exactly when it can run twice.
- "3x worst case" (2026-11-07),
  the stop ceiling of
  `process/stop_waits_while_a_member_cannot_be_interrupted.rs:41-46`.
- "C:
  shutdown stops its Gits (Recommended)" (2026-11-07T22:19:43-05:00),
  the service's stop
  (`process/stop_of_the_service_noticed_by_its_children.rs:1-20`).
- "Use as few locks as possible.",
  which is why there is no table of children:
  each wait owns its child (`process/stop_of_the_service_noticed_by_its_children.rs:10-14`).

One precedent outside this route bears on the first finding.
A review of cargo-shadow returned,
as a P1,
"the process-tree ownership heuristic can kill unrelated work",
and cargo-shadow replaced the heuristic with a delegated control group
(`doc/handover/cli-git-rust-implementation.md`,
"cargo-shadow",
round 6 and 7).

## Every child start, what ends the wait and what bounds it

### Steps under a child watch

Every Git step of a commit and every Git step of a landing goes through one runner:
`transaction/transaction_git.rs:412` from a synchronous caller,
`transaction/transaction_git_awaiting.rs:245` from a task.
`transaction/transaction_git_step_bounds.rs:55-103` picks one of three bounds,
and the bound decides everything about the wait.

- `StepBound::EndAt(class ceiling)`
  (`transaction_git_step_bounds.rs:62-64`,
  for a step `may_be_ended` decides,
  that is,
  one that can run twice):
  the child's output goes to two pipes made before the start
  (`process/step_output_read_whole_through_pipes.rs:64-90`),
  and `push/child_watch.rs:587` reads them to their end on the runtime while the child's
  exit is awaited through its pidfd (`step_output_read_whole_through_pipes.rs:193-201`).
  What ends the wait:
  the child's exit **and** both pipes reaching end of file,
  or the watch's stop.
  What bounds it:
  the watch's quiet window,
  then `class ceiling` as a hard end
  (`push/child_watch_task_loop_and_notice.rs:83-96`),
  plus the stop's grace and `STOP_CEILING`
  (`push/child_watch_clock_marks_and_stopping.rs:246-262`).
- `StepBound::LeaveRunning { after, extra }`
  (`transaction_git_step_bounds.rs:66-72`,
  for a side-ref write and for a step that is not ended
  off the landing thread):
  the output goes to two anonymous files (`process/output_capture.rs:68-75`),
  the child is handed to a waiter thread,
  and the caller waits
  `after + extra` on a channel (`push/child_watch.rs:558-580`).
  What bounds it:
  that channel wait.
  A child that outlives it is left running,
  with no reader.
- `StepBound::Unbounded`
  (`transaction_git_step_bounds.rs:95-101`,
  on the landing thread only,
  for a write that
  must not be cut):
  the caller blocks in `Child::wait()` on its own thread (`push/child_watch.rs:594-600`),
  the watch announces and never ends the child.
  What ends the wait:
  the child,
  the service's stop,
  or the person.
  There is no ceiling,
  which the owner decided and `SPEC.md:2495-2508` records.

### Children under a meter

- A push,
  a fetch,
  and a bounded LFS upload (`push/bounded_push.rs:433`):
  output into one anonymous file,
  the exit awaited through a pidfd,
  and
  `push/bounded_push.rs:245-280` ends it on a stall,
  on a flood of output,
  or on the service's stop,
  then `stop_of_a_push_and_sweep_of_its_locks.rs:168-215` stops it.
- The read-only queries of the auto-push (`push/bounded_query.rs:179`):
  the same meter.
- The repair tool (`repair/package_repair_command.rs:353`):
  its own process group,
  output into files,
  a meter and `REPAIR_TOOL_CEILING`
  (`package_repair_command.rs:355-390`),
  then a bounded reap with a late reaper handed to a task
  (`package_repair_command.rs:434-465`).
- The long-lived object reader (`candidate/candidate_reader.rs:223`):
  `cat-file --batch` over two pipes,
  under a watch with `OBJECT_STEP_CEILING`
  (`candidate_reader.rs:252`).

### Children with no watch and no ceiling

These waits end only when the child ends,
and nothing in the wrapper ends it:

- `push/auto_push_reconcile_merge_commit.rs:45-71`:
  `spawn()`,
  then `wait_with_output()` on the calling thread,
  then `writer.join()`.
  Every command of the reconcile's word-level second pass runs here
  (`auto_push_reconcile_merge_commit.rs:40-43`,
  `:145-160`).
- `push/manual_push_range.rs:100-112`:
  `spawn_retrying()`,
  the whole input written inline with `write_all`,
  then
  `wait_with_output()`.
- `lfs/lfs_interactive.rs:93-97`:
  `.output()` for each of the upload plan's Git questions.
- `lfs/lfs_upload.rs:171-178`:
  the interactive `git-lfs push` under `SignalGuard`,
  whose wait is
  `process/child_awaited_until_it_exits_or_the_service_is_stopped.rs:36-41`,
  which has no watch,
  no ceiling and no notice.
- `repo_read/git_metadata.rs:94-96`:
  `.output_retrying()`,
  which is `Command::output`.
- `repo_read/git_system_config.rs:85` and
  `invocation/management_service_repository_selection.rs:123`:
  `captured_child_run_to_its_end::output_of_blocking`,
  a pidfd wait with its output in files,
  but no meter,
  no ceiling and no notice.
- `invocation/forwarding.rs:201`:
  `status_retrying`,
  which is `Command::status`.
- `lock/index_lock_holders.rs:461` and `:508`:
  `lsof` or `powershell.exe`,
  `.output()`
  (macOS and Windows only).
- `process/process_identity.rs:287` and `process/process_start.rs:321` and `:368`:
  `ps` or `powershell.exe`,
  `.output()` (macOS and Windows only).

### How a refused start is reported

`push/start_retry.rs:229-300` retries a start whose error kind is `WouldBlock`
(that is `EAGAIN`) with a 5 second budget of pauses,
warns on each refusal,
and then fails with
the refusal's own kind plus how often it was refused.
A refusal that is not `WouldBlock` (a descriptor limit,
a missing program) is returned at once
with its own kind.
That is reported as what it is.

## Every signal sent and how the target's identity is held

- `process_tree::signal_members` (`process/process_tree.rs:239-249`) sends `SIGTERM` or `SIGKILL`
  to each member whose `/proc/<pid>/stat` start time still matches,
  read immediately before
  the `kill`.
  The identity is a start-time string,
  not a descriptor.
- `push/child_watch_clock_marks_and_stopping.rs:191-198` `signal_root` does the same for the
  step's own process,
  and sends nothing at all when no member was found.
- `push/stop_of_a_push_and_sweep_of_its_locks.rs:59-70` `ask_to_stop` sends `SIGTERM` to the
  child's number with no check at all,
  which is safe while the child is unreaped.
- `commit/native_commit_waiting_notice.rs:247-252` sends `SIGKILL` to every member of
  `descendants_of(std::process::id())`,
  each checked only by the start time it was found with,
  and the list is every child of the wrapper and not the step's.
- `repair/package_repair_command.rs:501-506` sends `SIGKILL` to `-child.id()`,
  a process group,
  with no check.
- `process/signal_relay.rs:628-648` publishes a child by opening a pidfd for it and pointing one
  never-closed descriptor number at it with `dup3`,
  so the relay's own signals cannot reach a
  reused number (`process/signal_relay.rs:120-160`).
  Off Linux the relay falls back to `CHILD_PID`,
  whose reuse window the module names
  (`process/signal_relay.rs:26-32`).
- `process/process_identity.rs:443-465` `signal_probe` only asks whether a number names a process;
  each caller compares identities itself.
- `binary/binary_supervisor_pin.rs:1-30` and `:44-160` hold a process by pidfd and re-read the
  start time around the descriptor's creation.
  That is the binary tests' guard,
  not production code.

## Children that outlive the wrapper

- A step left running (`StepBound::LeaveRunning`) is on purpose:
  it writes into files,
  not into
  pipes this process holds (`process/output_capture.rs:1-20`),
  and the system's init or subreaper
  reaps it.
  `SPEC.md:2525-2545` records this.
- A step with no ceiling on the landing thread outlives a service that is killed rather than
  stopped,
  and its side ref is what recovery reads.
- No production child dies with the wrapper:
  `rg 'PR_SET_PDEATHSIG'` over `src/native` finds it only in
  `binary/binary_support_fixture_service.rs:375`,
  a test fixture.
  A `SIGKILL` of the wrapper therefore orphans every Git it waits for,
  which is the case `process/signal_relay.rs` exists to make rare.

## Findings ranked by what they risk

### 1. A step's ceiling ends every child of the process, not the step's children

Where:
`commit/native_commit_waiting_notice.rs:243-253`,
with `programs_below` at `:165-177`.
Reproduced,
in "The scratch commit's ceiling ends a process beside the step".

The sequence:

1.  A landing of one repository runs the scratch `git commit` through
    `commit/commit_preparation.rs:208-221`,
    which starts the notice with
    `SCRATCH_COMMIT_STEP_CEILING`.
2.  The service runs landings of different repositories on worker threads of one process
    (`landing/service_native.rs:226` runs the scratch commit;
    `landing/service_round.rs:247-249` and `foundation/parallel_runner.rs:350-357` run the lanes at once),
    so a second landing's Git step is a child of the same process at the same moment.
3.  The first landing's step reaches its ceiling.
    The notice thread reads `descendants_of(std::process::id())`,
    the whole process's children,
    and sends `SIGKILL` to every one of them.

The wrong result:
the second landing's step is cut in the middle of its write,
with no ceiling of its own,
which is exactly what "Never cut a branch write (Recommended)" and
`SPEC.md:2401-2412` ("a step keeps its ceiling exactly when it can run twice") rule out.
A cut `update-ref`,
`merge`,
or side-ref write leaves the lock files and the half state the same
owner decision exists to prevent,
and the second landing reports a failure it cannot explain.
The comment that defends the code,
"Everything below this process is the step and its signer;
nothing of the person's runs there",
holds for a single interactive commit and not for the service.

Suggested fix (a suggestion,
not a decision):
hold the step's own pid,
or a pidfd,
and end only that process and the descendants read below it,
as `stop_of_a_push_and_sweep_of_its_locks.rs:178-215` already does for a push;
the panic-free form of the same is `binary/binary_supervisor_pin.rs:170-200`.

### 2. A pipe a descendant holds keeps a step's read from ending, past its ceiling

Where:
`process/step_output_read_whole_through_pipes.rs:193-201`,
reached from `push/child_watch.rs:585-589`.
Reproduced,
in "A read that a descendant holds does not end".

The sequence:

1.  A step that ends at its ceiling starts with its output on two pipes
    (`push/child_watch.rs:520-524`).
2.  The step's Git starts a program that inherits those streams and outlives it.
    Hooks are off for every Git the wrapper starts,
    so the programs left are the ones Git runs from
    configuration:
    a smudge or clean filter,
    a merge driver,
    or a signing program.
3.  The Git process ends while the program lives on,
    so the kernel reparents the program away from it,
    and the stop's own read of the tree
    (`push/child_watch_clock_marks_and_stopping.rs:229-238`)
    finds only the process that has ended
    (a second window is a program the step starts after that read).
4.  `tokio::join!` of the two reads and the child's wait
    (`step_output_read_whole_through_pipes.rs:199`) waits for both reads.
    A read ends at end of file,
    which needs every write end closed.

The wrong result:
the commit or the landing never returns.
The child has ended and the watcher task has finished,
so nothing is left that could end
the read:
`expired`,
`ceiling_hit` and the sentence the person should read
(`push/child_watch_step_bounds_and_errors.rs`) are never reached,
because they are read after the
join.
The command holds its locks and its transaction state for as long as the program lives,
which is the case "nothing should ever wait forever or indefinitely" is about,
and a second command then waits on those locks,
which is what `doc/planning/cli-git-no-self-invocation.md`'s siblings and
`doc/planning/cli-git-no-unbounded-waits.md` keep out of the design.

Suggested fix (a suggestion):
put the reads on a deadline of their own,
from the same ceiling the child has,
and report "the output of the step is still held open by a program it started"
with the program's number from `descendants_of` taken before the child ended;
or read the output into anonymous files as `StepBound::LeaveRunning` already does,
which is the reason `SPEC.md:2530-2545` gives for files in the repair tool.

### 3. A notice thread that cannot write keeps the commit from returning

Where:
`commit/native_commit_waiting_notice.rs:278-285` (`finish` joins with no bound),
called at `commit/commit_preparation.rs:221`.
Reproduced,
in "A notice whose sink blocks does not return".

The sequence:

1.  A commit whose message came with the command starts the notice with
    `sink: Box::new(std::io::stderr())` (`commit/commit_preparation.rs:210-219`).
2.  The wrapper's standard error is a pipe whose reader stops reading:
    a pager a person stopped,
    an editor or an agent that stopped draining it,
    or a terminal stopped with the flow control key.
    That state is one `NHP` names ("a closed terminal or broken pipe").
3.  The pipe fills,
    and the notice thread blocks inside `sink.write_all`
    (`native_commit_waiting_notice.rs:255-258`).
4.  The scratch `git commit` ends,
    and `finish()` sends its stop byte and calls
    `thread.join()`.

The wrong result:
the commit command never returns,
although the commit itself ended and the transaction's next
step would have run.
`join` has no bound,
the blocked write is not interruptible by the stop byte,
and the process keeps the index lock and the transaction while it waits.
The same shape,
a wait inside the command that no ceiling wraps,
is recorded against the landing's in-process content read
(`doc/handover/cli-git-rust-implementation.md`,
"the content judge",
finding 4).

Suggested fix (a suggestion):
write the line the way the watch does,
on the blocking pool and only while the stop is not told
(`push/child_watch_task_loop_and_notice.rs:150-180`),
and never join the notice thread without a bound:
a bound of the notice's own cadence is enough,
since the line is advisory.

### 4. A `/proc` look that fails is read as "the process is gone", so the step's root is never signalled

Where:
`process/process_tree.rs:140-150` (`member_of` returns nothing for any read failure),
`:162-172` (`still_running` returns false for any read failure),
and `push/child_watch_clock_marks_and_stopping.rs:191-198` (`signal_root` sends nothing when the
root was not read).
Reproduced,
in "A root whose record could not be read is never signalled".

The sequence:

1.  A watched step's quiet window passes,
    or its ceiling comes.
    `stop_process` reads `member_of(raw)` and `descendants_of(raw)`.
2.  Both reads open `/proc/<pid>/stat`,
    and both fail:
    at a descriptor limit (`NHR`'s "process and descriptor limits near exhaustion") with `EMFILE`,
    or with `EACCES`,
    `EIO`,
    or `ENOMEM`.
    `member_of` answers `None` and `descendants_of` answers an empty list,
    because each answers `continue` or `None` on any error.
3.  `signal_root` then has `root: None` and `identified: true` on Linux,
    so `root.as_ref().map_or(!identified, still_running)` is `false`,
    and no `SIGTERM` and no
    `SIGKILL` is ever sent to the step.
4.  `terminate_members` and `kill_members` have an empty list to walk.

The wrong result:
the child keeps running,
the wait keeps blocking,
and the ceiling that should end the step ends nothing,
so the command waits for as long as the
child does.
The final loop at `child_watch_clock_marks_and_stopping.rs:264-271` also exits at
once,
because `still_running` answers false for every member,
so the lock sweep runs against a live tree and finds nothing.
Every other reader of the process table in this crate separates a refusal from an exit:
`process/process_identity.rs:308-322` (`record_is_gone`),
`process/process_use.rs:90-99` (`exited`),
and the `unavailable` and `unreadable` fields of
`UseScan`,
which exist for exactly this reason.
The two functions the stop uses do not.

Suggested fix (a suggestion):
give `member_of` and `still_running` the three-valued answer `process_use` already uses
(alive,
gone,
unreadable),
and make `signal_root` treat "unreadable" as "send it anyway",
which is what the non-Linux branch already does with `!identified`.

### 5. Waits on a child that nothing can end

Where:
the list in "Children with no watch and no ceiling".
Read from the code only.

The sequence,
per instance:

1.  `push/auto_push_reconcile_merge_commit.rs:45-71` runs `git merge-tree`,
    the word-level second
    pass,
    and `git commit-tree -S` with `spawn()` and `wait_with_output()`.
    A signing program that waits for a smartcard touch,
    or a second-pass command that never
    answers,
    blocks the reconcile with nothing printed,
    no notice,
    no meter and no ceiling.
    The auto-push of that branch then waits for as long as the child does,
    which is the owner's "nothing should ever wait forever or indefinitely" broken without a record,
    on a path whose sibling (`push/bounded_push.rs`) has all four.
2.  `lfs/lfs_upload.rs:171-178` waits for `git-lfs push` through `SignalGuard` alone,
    so a server that accepts the connection and never answers holds an interactive push forever.
    Pick 9 ("Deadlines of tools that are not Git:
    end only on no progress") is the decision this
    instance does not carry out;
    the bounded twin for the same program exists at `lfs/lfs_upload.rs:327`.
3.  `lfs/lfs_interactive.rs:95` and `repo_read/git_metadata.rs:96` block in `Command::output()`.
4.  `invocation/forwarding.rs:201` blocks in `Command::status()` for the forwarded command,
    which is by design (the person's Git owns the terminal) and is named here for completeness.

The wrong result:
a command or the service's reconcile that waits forever,
with nothing on standard error to say
which program it waits for,
and with no way to learn that from the wrapper.
The `quiet_ledger` records nothing for these,
because no watch exists to record it.

Suggested fix (a suggestion):
route 1 and 2 through the two existing forms rather than a new one:
`push/bounded_query.rs` for a query (meter,
no wall clock) and `push/bounded_push.rs` for the
upload (meter and stop),
which is what `lfs/lfs_upload.rs:327` already calls.

### 6. A signal is swallowed when the child cannot be published to the relay

Where:
`process/signal_relay.rs:602-613` (`publish` fails,
`relaying` stays true),
`:628-648` (`publish` returns false at a refused `pidfd_open`),
and `:180-220` (`relay_handler` drops a pending signal while nothing is published).
Read from the code only.

The sequence:

1.  A commit's Git runs under `SignalGuard::install()`,
    which installs the four handlers
    (`commit/commit_preparation.rs:241-250`,
    `lfs/lfs_upload.rs:171`).
2.  `publish` fails:
    `pidfd_open` is refused at a descriptor limit,
    `ensure_relay_descriptor` cannot open `/dev/null`,
    or `dup3` fails.
    The failure is logged and returns `false`.
3.  The person sends `SIGINT`,
    or the service sends `SIGTERM`.
    `relay_handler` stores it in `PENDING` and calls `relay_pending`,
    which returns at its first
    line because `CHILD_PUBLISHED` is false.

The wrong result:
the signal reaches neither the child nor the wrapper.
The wrapper does not die,
because its disposition is the handler,
and Git does not end,
because nothing was relayed.
A second signal is dropped in the same way,
so the only way out is `SIGKILL`.
`process/signal_relay.rs:634` states the intent as
"The commit must go on:
without the descriptor the child is not relayed to,
and the log says so",
and the price of that choice is that the command can no longer be interrupted.

Suggested fix (a suggestion):
when `publish` fails,
put the saved dispositions back for the life of that guard
(`restore` already does it on a failed install),
so the wrapper ends by the signal as it would without a guard,
or relay by number as the non-Linux branch does,
and say in the warning that a reused number could
be reached.

### 7. Signalling by number after checking the number is racy

Where:
`process/process_tree.rs:239-249` (check,
then `kill`),
`push/child_watch_clock_marks_and_stopping.rs:191-198`,
and `repair/package_repair_command.rs:501-506` (a process group,
no check).
Read from the code only.

The sequence:

1.  `signal_members` reads `/proc/<pid>/stat` and matches the recorded start time.
2.  The process exits and the kernel reuses its number for a process of the same account,
    which is what `/proc/sys/kernel/pid_max` allows once enough processes have been forked.
3.  `kill(pid, SIGKILL)` reaches the new process.

The wrong result:
a process beside the wrapper,
of the same account,
is ended,
which is the same wrong result the cargo-shadow review called a P1
("the process-tree ownership heuristic can kill unrelated work").
The window is small and the repo already holds the answer:
`binary/binary_supervisor_pin.rs:44-160` opens a pidfd,
re-reads the start time around it,
and
sends through the descriptor,
and `process/signal_relay.rs:628-648` does the same for one child.

Suggested fix (a suggestion):
hold a pidfd for each member found and signal through it,
falling back to the number only where `pidfd_open` is refused,
and then saying so.

### 8. A refused thread silently removes the only thing that would end a step

Where:
`push/child_watch.rs:205-215` (`spawn_on_runtime` returns `None`),
`push/child_watch.rs:98-101` (`running_watcher_of` maps it to no watcher),
and `foundation/lazy_runtime.rs:275-284`.
Read from the code only.

The sequence:

1.  The OS refuses a thread,
    or the process's runtime cannot be built
    (`NHR`'s "process and descriptor limits near exhaustion").
2.  `ChildWatch::start_under_stop` gets `None` from `spawn_on_runtime`.
    The watch has no task,
    so it announces nothing,
    ends nothing,
    and keeps no quiet figure.
3.  The caller waits as `push/child_watch.rs:585-600` says:
    a pipe read on the runtime,
    or a
    blocking `Child::wait()`.

The wrong result:
a step with no ceiling (`StepBound::Unbounded`) and one that keeps quiet are both waited for
forever,
with no notice,
and nothing says why.
`foundation/blocking_job.rs` was written for this same refusal and reports it
("the system refused a thread for a blocking step");
the watch's path reports nothing,
and `ChildWatch::drop` and `close_awaiting` treat "no task" as "nothing to wait for".

Suggested fix (a suggestion):
record the refusal in the watch,
print it once through the same notice the watch would print,
and give the wait the ceiling the watch would have given it,
or fail the step with the sentence `blocking_job` uses.

### 9. A full request pipe deadlocks one Git helper

Where:
`push/manual_push_range.rs:100-112`.
Read from the code only,
and its reach is narrow.

The sequence:

1.  `run_git_bytes` starts a Git that reads requests from standard input and answers on standard
    output,
    with both as pipes (`manual_push_range.rs:91-99`).
    The only caller that passes input is `peel_commits`
    (`manual_push_range.rs:157-173`,
    `cat-file --batch-check`).
2.  The wrapper writes the whole request list with `write_all` **before** it reads anything.
3.  The reply pipe fills,
    so Git blocks writing,
    so it stops reading requests,
    so the wrapper blocks
    writing them.

The wrong result:
a deadlock,
with no bound and no notice.
Its reach is narrow because the input is one line per destination named by the update,
and a push
with thousands of destinations is needed to reach a pipe's capacity.
The other two callers pass no input (`:225`,
`:335`),
so they cannot deadlock this way,
but they share the unbounded `wait_with_output`.

Suggested fix (a suggestion):
hand the input to a thread,
as `auto_push_reconcile_merge_commit.rs:52-71` does,
and judge the command with the query meter.

## Reproductions

Four controls were appended to the crate's test sidecars,
run with
`mise run //package/git-policy/cli:native:test:host -- <filter>`,
and then removed.
Each proves the wrong result rather than asserting the right one,
and each contains its own positive control or its own bound.

This page itself was formatted with
`mise run //:format:monochromatic-lint -- <this page>`.
`mise run //:lint:monochromatic-lint` was started and waited 598.98 s inside its
`fetch:rust-dependencies` dependency,
behind another session's Cargo package-cache lock,
until the caller's bound killed it,
so the task's own command line was run instead:
`mise exec -- monochromatic-lint <this page>`,
which exits 0.
The probe was checked against a copy of this page with one pipe table appended,
which the same command reported
(`markdown/no-pipe-tables`,
exit 1).

### A read that a descendant holds does not end

`push/child_watch_tests.rs`,
control `a_pipe_that_a_descendant_holds_keeps_the_read_from_ending`:
a shell starts a sleeper that inherits the step's two pipes,
writes its number,
and exits;
the control waits,
bounded,
for the shell to be gone,
then reads the pipes exactly as `push/child_watch.rs:587` does,
and asserts that the read has not returned after 3 seconds,
then kills the sleeper and asserts that the read returns only then.

Output:

```text
PASS [   3.032s] (3/3) monochromatic-git-policy-cli push::child_watch::tests::a_pipe_that_a_descendant_holds_keeps_the_read_from_ending
```

### A root whose record could not be read is never signalled

`push/child_watch_tests.rs`,
control `a_root_whose_record_could_not_be_read_is_never_signalled`:
a sleeper is started,
`member_of` reads its record,
and `signal_root(&None, true, pid, SIGKILL)`,
the state a failed `/proc` read leaves,
is called;
the sleeper is shown alive after 3 seconds.
The positive control is the same call with the member found,
which ends it within the bound.

Output:

```text
PASS [   3.026s] (2/3) monochromatic-git-policy-cli push::child_watch::tests::a_root_whose_record_could_not_be_read_is_never_signalled
```

### A notice whose sink blocks does not return

`commit/native_commit_waiting_notice_tests.rs`,
control
`a_notice_whose_sink_blocks_keeps_the_commit_from_returning`:
the notice is started with the same `start_within` the commit path uses and a sink that says when
its `write` was entered and then waits to be let go,
the state of a standard error whose reader
stopped reading;
the control asserts that `finish()` has not returned after 2 seconds,
lets the sink write,
and asserts that `finish()` returns only then.

Output:

```text
PASS [   2.035s] (1/3) monochromatic-git-policy-cli commit::native_commit_waiting_notice::tests::a_notice_whose_sink_blocks_keeps_the_commit_from_returning
```

### The scratch commit's ceiling ends a process beside the step

`commit/native_commit_waiting_notice_tests.rs`,
control
`the_ceiling_of_the_scratch_commit_ends_every_child_of_the_process`:
a `sleep` is started that is nobody's step but is a child of the test process,
the notice is started with a 100 ms ceiling,
and `finish()` returning a duration is the proof that
the ceiling kill ran;
the control then asserts that the `sleep` was ended.
Nothing in the code under test connects the two:
the kill walks
`descendants_of(std::process::id())`.

Output:

```text
PASS [   0.152s] (1/1) monochromatic-git-policy-cli commit::native_commit_waiting_notice::tests::the_ceiling_of_the_scratch_commit_ends_every_child_of_the_process
```

## What I could not determine

- Whether a Git step the wrapper runs can leave a program holding its pipes in practice.
  Hooks are off (`src/native/invocation/hooks_off.rs`),
  so the candidates are a filter,
  a merge
  driver,
  a signing program and a transport helper.
  The first finding's reproduction shows the read cannot end while such a program lives;
  whether one exists in this repository's own configuration needs a run of the built wrapper on a
  repository that has one,
  which is outside an audit.
- How often the descriptor limit that finding 4 and finding 6 need occurs here.
  The rule `NHR` says it is normal;
  no log records `EMFILE` counts,
  and a probe would have to
  exhaust the limits of the agent's own session.
- Whether a reused process number has ever been signalled.
  The window is microseconds;
  nothing in the wrapper records the start time it checked beside the
  number it used,
  so no log can answer it.
- The number of commits in the service that run the scratch commit at the same moment as another
  landing's step.
  `foundation/parallel_runner.rs:350-357` hands lanes to workers concurrently,
  and `landing/service_round.rs:247-249` sizes the workers from the host;
  I did not run the service to count the overlap.
- The behaviour on macOS and Windows,
  which the owner's pause covers.
  Every non-Linux branch this page names is marked untested in the code.

## What I checked and found sound

- A refused start is retried only for `EAGAIN` and only within a stated budget,
  and the final
  error keeps its kind and adds how often it was refused (`push/start_retry.rs:140-190`).
- A failed `pidfd_open` or a failed registration changes only how a child's exit is awaited,
  from a descriptor to a 10 ms timer,
  and never reports a failed start
  (`process/child_awaited_through_a_pidfd.rs:104-135`,
  `:180-200`).
- A child is owned from the moment it starts:
  `OwnedChild::drop` kills and reaps it,
  and the two hand-over calls are the only ways a child leaves that ownership
  (`process/child_awaited_through_a_pidfd.rs:200-260`).
- The relay's descriptor number is never closed and is repointed with `dup3`,
  so a signal cannot reach a reused number on Linux (`process/signal_relay.rs:120-160`).
- A notice that a watch prints is written on the blocking pool,
  so a terminal that does not take
  the line holds a pool thread and not a runtime worker
  (`push/child_watch_task_loop_and_notice.rs:150-180`).
- A service's stop reaches every waiting child,
  and the last resort is an `alarm` in the handler,
  so a runtime whose workers are all blocked still ends the process
  (`process/stop_of_the_service_noticed_by_its_children.rs:128-143`,
  `:213-220`).
- The stop past its grace waits only while a member is in uninterruptible sleep,
  and never past
  `STOP_CEILING` (the owner's "3x worst case"),
  `process/stop_waits_while_a_member_cannot_be_interrupted.rs:120-130`.
- A push's waits end on a stall,
  on a flood of output,
  on the service's stop,
  and never on a wall
  clock while a remote connection is open (`push/bounded_push.rs:245-280`).
- The repair tool's output goes to files,
  so a helper that keeps an inherited stream open holds
  nothing up,
  and its reap is bounded,
  with a late reaper handed to a task
  (`repair/package_repair_command.rs:355-390`,
  `:451-470`).
- A wrapper that arrives as Git stops and explains itself,
  with the starter held by its number and
  its birth identity,
  and the check refuses to stop a process whose number was reused
  (`invocation/started_as_git.rs:275-305`,
  `:391-400`).
  Every real-Git start marks itself (`repo_read/git_system_config.rs:84`,
  `invocation/management_service_repository_selection.rs:122`,
  `invocation/forwarding.rs:100`,
  `:239`,
  `push/auto_push_single_flight.rs:158`),
  so the recursion the owner ruled out cannot
  repeat:
  a grandchild has another process in between and does not match the marker.
  The one program started without a marker is `git-lfs` (`lfs/lfs_upload.rs:242`,
  `:327`),
  whose own `git` runs with the real Git's directory first on `PATH`;
  a `git` reached from there starts a fresh wrapper,
  which stops at its own first real-Git start,
  so the recursion is bounded at one extra wrapper.
- `process/process_use.rs` and `process/process_identity.rs` separate a refusal from an exit,
  which
  is the shape findings 4 and 7 ask for elsewhere.

## What to inspect and how to respond

Read in this order:

1.  Finding 1,
    and answer whether a step's ceiling may end a process it did not start.
    The reproduction starts one `sleep` beside the notice and needs no load.
2.  Findings 2,
    3 and 4 together,
    because each one is a wait that nothing can end,
    and each has a different fix that already exists elsewhere in the crate.
3.  Finding 5,
    and say whether the reconcile's second pass and the interactive LFS upload are in
    pick 9's scope.
4.  Findings 6,
    7,
    8 and 9 are the smaller ones;
    each is a suggestion,
    and each says which existing function to reuse.

Nothing in this page is a decision.
No production code was changed.
