# A step whose watch cannot start

Status:
a proposal put to the owner by the round for issue 6590,
unapproved and non-final.
Nothing here changes release behaviour,
and no decision record follows from it until the owner answers.
Written 2026-11-27 on the worktree
`fix/git-policy-cli-issue-6590-child-watch-step-inside-its-ceiling-control-holds-when-a-thread-is-refused`.

## What release code does when a watch cannot start

A watched step is a child the wrapper starts and reads,
and a watcher task announces the wait and ends the step when it shows no sign of work for its quiet window
(`package/git-policy/cli/src/native/push/child_watch.rs`).

`ChildWatch::start_under_stop` asks `crate::lazy_runtime::tasks_can_run()`.
Where that answers no,
no task is started,
the watch is `unwatched`,
and the line `no_watcher_text` words is said once,
with its lines broken at their clauses here where the issue quotes it whole:

```text
a step with a holder on its standard error has no watcher: the system refused a thread for it, so nothing announces
this wait, nothing ends the step for being quiet, and a step with a ceiling is ended at that ceiling by the wait
itself. Hint: check the process limit (`ulimit -u`) and the control group's `pids.max`, free some, and run the
command again.
```

The wait for the step's output then carries the step's ceiling itself
(`carried_for`,
`src/native/process/step_output_ceiling_carried_by_the_wait.rs`):
it ends the step and the processes the step started at that ceiling,
and words the ending with the same reason.

That is what issue 6476 settled for a refusal:
the commit at its process limit failed saying nothing,
so the processes that could not start a thread say which limit refused it
and write their lines on the thread that says them
(`src/native/foundation/lines_written_by_a_thread.rs`,
`src/native/foundation/diagnostic_lines_off_the_calling_thread.rs`).
A watch that cannot start is the same answer:
one line naming the limits to check,
never a wait without an end.

## What is wrong with it

### A build that is late is not a refused thread

`tasks_can_run()` answers no in four states,
and only two of them are a refusal:

- `BuiltFlavor::CurrentThreadAfterFailure`:
  the multi-thread build failed,
  or its helper thread could not start.
  A real refusal.
- `BuiltFlavor::CurrentThread`:
  the plan asked for no worker,
  because this process's thread and address-space limits
  left no room for one.
  A limit,
  in the way the sentence means it.
- `BuiltFlavor::CurrentThreadAfterDeadline`:
  the build outlasted `BUILD_DEADLINE`,
  and the runtime it was building is still on its way and is kept for the work it was built for
  (`the_build_of_the_process_runtime.rs`,
  `LATE_RUNTIME`).
  Nothing was refused,
  and a worker arrives.
  `the_worker_answers` knows that state:
  it reports `a_build_is_coming` beside `can_run`.
- A control's stand-in,
  `a_control_refuses_every_task`.

The watch asks only `tasks_can_run()`.
So a person's push on a machine slow to give a process its threads loses its watch on the first steps,
is told that a thread was refused,
and has those steps ended at their ceiling instead of being announced and watched,
while the runtime that would have watched them arrives a moment later.
`spawn_on_runtime_within` exists for exactly that state,
and the service's landing already uses it.

What this rests on:
the source of the four states and of `the_worker_answers`
(`src/native/foundation/lazy_runtime.rs`,
`src/native/foundation/the_build_of_the_process_runtime.rs`),
and the watch's single question (`src/native/push/child_watch.rs`).
The state itself was not produced on this host during this round:
it needs a build whose helper thread is slow,
which `CLI_GIT_TEST_ONLY_RUNTIME_BUILD_MILLIS` can make late,
and the round did not run a control of the watch in that state.

### A sentence a reader cannot tell from a control's own

The same sentence is said by a control that plants the refusal,
so a reader of a gate log cannot tell a real refusal from a planted one.
The round for issue 6562 read it as the cause of a failure that was a control's own window
(issue 6590,
and the counter control
`the_thread_a_watch_is_refused_is_no_limit_this_process_reached` in
`src/native/push/child_watch_tests.rs` measures that no limit is near when it is said).
That is a cost of the stand-in's words,
not of a release state,
and the control beside the sentence now says so in its own doc.

### The answer waits for a scan that names a leftover program

Not about the watch,
but on the same path and found in the same round.
A step whose output a program it started still holds open is answered after a drain of `OUTPUT_DRAIN_AFTER_EXIT`,
and then the process table is scanned to name that program,
up to `PROCESS_SCAN_CEILING`,
with one descriptor read
that cannot be interrupted and so can overrun the bound.
The scan delays the answer only to fill in the sentence naming the holder.

## Options

Each option says what a person would notice.

### A1. The watch waits, bounded, for a build that is coming

`ChildWatch` asks for its task through `spawn_on_runtime_within` with the watch's own bound
(one `BUILD_DEADLINE` more,
or the step's remaining quiet window,
whichever is shorter),
and the sentence is worded from the flavor (`flavor_words`) rather than assuming a refusal.

- Pro:
  a step on a machine slow to give threads is still announced and watched,
  which is the promise the watch makes.
- Pro:
  the sentence tells the person what happened,
  so the hint names the limit only where a limit refused.
- Con:
  a push waits up to one more second before a step whose watch cannot start is treated as unwatched.
- Con:
  the state needs a control before it can be trusted,
  and producing a late build in a control needs the
  build-pause switch,
  which is a stand-in for the slow machine rather than the machine.

### A2. Keep giving up at once, and word the state from the flavor

The watch keeps asking `tasks_can_run()` once,
and `no_watcher_text` takes the flavor and the two answers about a worker,
so the line says a refusal where one happened and a late build where one is coming.

- Pro:
  no wait is added to a push,
  and the change is words plus one argument.
- Pro:
  the line names what happened,
  which is the rule issue 6476 set for every refusal.
- Con:
  the first steps of a push on a slow machine still run unannounced and are ended at their ceilings,
  where the wrapper's own rule is that a step that stalls must say so.

### A3. Leave the watch as it is, and record the state

A note in `child_watch.rs` and in `SPEC.md` says that a build that outlasts its deadline costs the first steps of a
push their watch.

- Pro:
  no behaviour changes.
- Con:
  the wrapper tells a person that a thread was refused when none was,
  and the push pays a ceiling ending for a late build.

### B1. The scan that names a holder moves off the answer's path

The answer returns at the drain's end,
and the sentence naming the program holding the stream is said by the diagnostic queue once the scan answers.

- Pro:
  a person's push stops waiting up to two seconds,
  and longer on a loaded host,
  for a sentence about a leftover
  program.
- Pro:
  the answer's bytes stop depending on the process table.
- Con:
  the sentence can arrive after the answer,
  or not at all where the process ends first,
  which is what the queue already promises for its lines.
- Con:
  the count of what the answer holds becomes weaker evidence for a control that the read was cut,
  and the control would rest on the drain's bound instead.

### B2. Keep the scan where it is, and bound it tighter

`PROCESS_SCAN_CEILING` is measured at five times the longest walk of 2026-11-07;
a second scan bound for this one caller would be smaller.

- Pro:
  no structural change.
- Con:
  a tighter bound names fewer holders on a loaded host,
  which is where the sentence is most useful.

### B3. Keep the answer as it is

- Pro:
  no behaviour changes,
  and the sentence stays in the answer.
- Con:
  the answer keeps waiting on the process table for up to two seconds and one uninterruptible read,
  which is what made a control's own three-second window fail under load.

## Ranking

A1 > A2 > A3 for the watch:
A1 keeps the promise the watch makes and tells the person the truth,
A2 tells the truth at no cost but keeps the unwatched step,
A3 keeps a sentence that names a refusal that did not happen.
A1's cost is one more bound of waiting and a stand-in in a control,
which the round after this one can pay.

B1 > B3 > B2 for the scan:
B1 takes work nothing bounds off the answer's path,
B3 is honest but leaves the cost,
and B2 narrows the sentence exactly where it is worth most.

The two questions are separable.
The watch's answer changes what a person sees at the start of a push;
the scan's changes what a person waits for at its end.

## What this round did instead

It fixed the control whose window this issue was opened for,
and added the control that measures where the refusal stands,
changing no release behaviour
(the commits of
`fix/git-policy-cli-issue-6590-child-watch-step-inside-its-ceiling-control-holds-when-a-thread-is-refused`).

## Open questions for the owner

1. Does a push's first step deserve the extra second A1 costs,
   or is A2's truth in the words enough?
2. May the sentence naming a program that holds a stream move off the answer's path (B1),
   given that it can then arrive after the answer or not at all?
