# The crate split's fourth round: the five changed signatures, proposed for the owner

Status:
unapproved and non-final.
Prepared by a Pi run on 2026-11-28,
which read this worktree's `main` at `5a84e361c0`
(`git log --format=%cI`:
2026-11-28T06:16:02-05:00)
and revised in place at 2026-11-28T06:43:30-05:00
after the gate read described below.
No code changed on this branch.
The owner has not answered any question it asks,
and no review from another vendor than the authors' has read it.

Gate reads of this proposal:
one,
a same-vendor second pass (Claude Sonnet,
the vendor of part of the work),
made on 2026-11-28 between about 03:20 and 04:05
against the draft at `d8fafa9ea6`.
It found five High,
eleven Medium and five Low findings,
and its verdict was "not ready to put to the owner as written".
This revision answers every one of them:
where a finding was right the proposal changed,
and where it was wrong the finding is named here with the line that settles it.
It is not an independent review (`AGENTS.md` rule `IRW`),
and another read from another vendor has not been made.

## What this record is for

The owner's answer of 2026-11-28 on the crate split's step 4 was
"Read the signatures first (Recommended)"
(`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
"The crate split's fourth round"),
and what follows it is that
"the five signatures of round 4 are prepared as a proposal,
read by a gate reader,
and then put to the owner"
and that
"Round 4 does not run before that".

This is that proposal.
For each of the five call changes it states where the call is today,
what the plan says the call becomes,
two or three signatures that would do it,
what each delivers for the split and what each costs,
what was run to find out whether it works,
and a ranking with a stated measure.
It then states the order the five can land in,
which of them touch one file,
and the questions only the owner can answer.

### The words these pages use

Read this once;
each question later repeats what it needs.

- The crate split is the owner's decision of 2026-11-04
  to break the wrapper into reusable crates
  (`doc/planning/cli-git-source-layout.md`,
  "Splitting into crates (owner,
  2026-11-04)").
  Its **step 7** is the move of the files into the target folders,
  after the call changes.
- The target tree is
  `doc/planning/cli-git-source-layout-target-tree.md`,
  which names every target folder and file,
  and `cli-git-source-layout-target-tree-files.md`,
  which lists them file by file with each folder's `uses/` line.
- A folder's **`uses/` list** is the list of folders whose code it uses,
  written as links in a `uses/` folder inside it (`AGENTS.md` LYD).
  Two folders that use each other are a **loop**,
  and no loop is allowed (`AGENTS.md` LYL).
- The allow list is `package/git-policy/cli/layout-allow-list/folder-rule.txt`,
  whose lines tolerate the loops `main` still has.
  It may only shrink
  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`),
  and a line whose use is gone is reported by name as dead.
- `Action` is the value the top level runs
  (`package/git-policy/cli/src/native/invocation/action.rs`):
  a command's exit status and its two streams,
  or a forwarded or planned run.
- `GitRequest` is what one step of a transaction asks for
  (`transaction/transaction_git_request_and_reply_types.rs`):
  its arguments,
  its directory,
  its class and its terms.
- `ScopeEnvironment` is the configuration scopes a reader may read,
  settled from a list of environment variables
  (`repo_read/repo_read_config_scan.rs`).
- `foundation` is today's `src/native/foundation/`,
  the folder of small helpers at the bottom,
  which step 7 splits among three libraries and six program folders
  (target tree,
  lines 819-824).
- The landing is the one operation that writes a commit in the service,
  the only place a step is never ended
  (the transaction's `step_bound`).

## Which record governs where they disagree

The question this proposal answers is
"Open:
does the crate split's step 4 run now?"
(`doc/handover/questions-for-the-owner.md`),
which names the five as finished results,
system settings path,
after-stop action,
current step record and run this next.

The owner's decision is
`doc/planning/cli-git-source-layout.md`
(section "Splitting into crates (owner,
2026-11-04)",
and the answers of 2026-11-04 in "What the owner asked for").
The plan is that file's section "Changes to how code is called"
and the target tree's section "The call changes,
as signatures",
which disagree with each other in the places below.

Where records disagree this proposal follows the owner's recorded answers first,
the plan's prose second,
the target tree's signatures third
and the handover entry last.
Each disagreement is stated where it bites:

1.  The handover makes run this next a single call change in `stacked_merge_push`.
   The plan's run this next replaces two in-process recursions,
   `entry::plan_invocation` called from `stacked_merge_push`
   and `entry::run_process` called from `merge_second_pass_finish`
   (target tree,
    lines 6434-6454).
   The plan governs:
   this proposal describes the whole call change.
2.  Commands that only finish:
   the plan's prose asks for
   "a small \"finished with this code and output\" value from a low file"
   (line 470),
   while the target tree's signature puts the new file `what_a_command_hands_back`
   in the program folder `starting_git_and_describing_a_run`
   (line 6436),
   which is not a low file.
   The two placements deliver different things,
   so the departure is the owner's to make:
   question 1.
3.  After-stop action:
   the plan's prose says
   "its caller hands it what to do after a stop"
   (line 458),
   which is signature B of section 3,
   while the target tree's signature says
   "The watcher returns the `StopReport` it already has;
   the caller in `bounded_push` and `transaction_git` builds the
   `LockLedger` before the start and sweeps after the stop"
   (lines 6472-6476),
   which is signature A.
   `StopReport` is not a type the watcher has:
   it is the push's progress clock
   (`push/stop_cause_and_words.rs:61`,
   used as `Stalled(StopReport)` at line 104),
   so the tree's sentence also names a type that would not travel.
   Question 2 puts the departure to the owner.
4.  Current step record:
   the plan's prose says
   "a small \"current step\" record in the basic tools that the runner writes and the service reads"
   (line 455),
   the target tree puts it in the program file `current_step_record`
   in `how_long_the_program_waits_for_git_steps`
   (lines 6457-6459),
   and the code carries it in a `tokio::task_local!`,
   not in a thread-local as the tree's sentence says
   (`landing/landing_state_carried_by_its_task.rs:44-48`,
   and the owner's choice 4 in
   `doc/planning/git-policy-cli-full-tokio.md`,
   lines 1655-1657 and 3370-3376).
   Question 3 puts the departure to the owner.
5.  System settings path:
   the tree says `ScopeEnvironment`
   "gains the field `system_config: Option<PathBuf>`"
   (line 6479),
   but the struct already holds that fact as `system_default`
   (`repo_read/repo_read_config_scan.rs:214`),
   filled at line 297,
   and its meaning today (unknown,
    which makes the scope unresolvable)
   is the one a parameter must keep.
   The change is a parameter,
    not a field,
   and that is the owner's own wording:
   "the system settings path passed to the reader"
   (line 118) and "the caller passes the path in" (line 462).
6.  The tree says the line that calls `LockLedger::before_start(command)` "in `child_watch`" moves out
   (line 6470).
   On `main` there are two such lines,
    in two sibling files,
   `push/child_watch_one_shot_run.rs:238` and
   `push/child_watch_output_watched_on_the_runtime.rs:243`,
   because round 2 of the split cut them out of `child_watch.rs` after the plan was written.
   The sentence also does not name the sweep inside
   `child_watch_clock_marks_and_stopping::stop_process` (lines 372-383),
   which is the half the change has to move.
7.  The tree names `transaction_git` alone for the current step record (line 6456),
   but `on_landing_thread` is also called by the two watched-run files above
   (`child_watch_one_shot_run.rs:230`,
   `child_watch_output_watched_on_the_runtime.rs:235`),
   and the watcher library's `uses/` line names the folder those files would name
   (target-tree-files,
    line 322).
   A change that stops at `transaction_git` delivers nothing to that line.
8.  Run this next:
   the tree says
   "A depth guard replaces the counter"
   (line 6454),
   while the code has a budget read from the todo list
   (`merge/merge_second_pass_finish_nesting.rs`,
   `FINISH_BUDGET` and `finish_budget`).
   A guard of eight is not a replacement of one kind for another;
   section 5 keeps the budget.
9.  The plan's order has the system settings change second,
    not third (target tree,
    lines 8185-8191 and 6956-6963).
   The earlier draft of this proposal said third;
   that was wrong.

## How the call sites were found, and what moved since 2026-11-27

Every call site below was read in this worktree at `5a84e361c0`,
with `rg` and `sed`,
and the line numbers are of that commit.
The handover entry's list is from 2026-11-27T06:40:00-05:00,
and `main` took merges since;
what moved is stated beside each call change.

Two measurements cover all five call changes,
both run on 2026-11-28 by this revision:

- The allow list's `uses` lines for the folder pairs each change adds or removes,
  printed by
  `rg -n "^uses src/native/<folder> -> src/native/<folder>" layout-allow-list/folder-rule.txt`.
  Where a pair has no line today,
  a change that adds the use adds a finding no line tolerates.
- The counts quoted below,
  each with the command that printed it,
  in the sentence that uses it.

The dead lines each change delivers were not measured here:
the earlier draft measured them on a throwaway branch,
and this revision did not rebuild that branch.
Section 1 therefore states the delivery as the earlier draft's measurement,
marked as such,
and the round that lands it measures it again.

## 1. Commands that only finish

### Where the call is today

- `package/git-policy/cli/src/native/subvolume/subvolume_command.rs:353`:
  `pub fn migrate_action(arguments: &MigrateArguments, cwd: &Path) -> Action`,
  which returns `Action::Exit` with `stdout: String::from(MIGRATE_HELP)`
  for `--help` (line 358),
  `Action::Exit` with `stdout: String::new()` and the refusal in `stderr`
  (lines 366-376),
  and `Action::Exit { code, stdout: String::new(), stderr: String::new() }`
  for the run (lines 383-387).
- `package/git-policy/cli/src/native/health/doctor.rs:607`:
  `pub async fn doctor_action_with_section(...) -> Action`,
  which returns `Action::Exit` with `stdout: String::new()` and the explanation in
  `stderr` (lines 662-666) and `Action::ExitBytes` (lines 676 and 698);
  `doctor_failure(message: &str) -> Action` at line 718,
  whose `Action::Exit` puts its words on `stderr` and nothing on `stdout` (lines 719-722).
  Its synchronous face `doctor_action` (line 441) has no release caller:
  the only caller is `invocation/management.rs:527`,
  which calls the async one.
  It is a published name
  (`source_guard/items_nameable_from_outside_release.txt:2451`),
  so rewriting its return type changes the library's surface,
  and deleting it instead is a choice this proposal does not take.
- `package/git-policy/cli/src/native/worktree/worktree_convert_command.rs:147`:
  `pub fn convert_action(...) -> Action`,
  returning `Action::Exit` for `--help` (line 154) and for the run (line 171).
- `package/git-policy/cli/src/native/push/stacked_merge_push.rs:226`:
  `let action: Action = plan_invocation(arguments.as_slice(), environment.as_slice(), &inputs);`
  then `return Ok(run_planned(action));` (line 227),
  and `fn run_planned(action: Action) -> i32` at line 119 matches the planned `Action`'s variants.
- The consumers that build the `Action` instead:
  `invocation/management.rs:479` (`return migrate_action(arguments, cwd);`),
  `:385` (`return crate::worktree_convert_command::convert_action(...)`),
  and `:527` (`let action: Action = match run_on_runtime(doctor_action_with_section(`).
- The exit code the move drags with it:
  `ENGINE_FAILURE_EXIT_CODE` is named on 52 release lines of 18 files in 9 folders
  (`rg -n "ENGINE_FAILURE_EXIT_CODE" -g '*.rs' -g '!*_tests.rs'`,
  counted by `wc -l` and by `xargs -n1 dirname | sort -u`),
  not on two.
  It is defined at `invocation/action.rs:45`,
  used at `subvolume_command.rs:240` and `doctor.rs:720`,
  and its published line is
  `invocation::action const ENGINE_FAILURE_EXIT_CODE`
  (`items_nameable_from_outside_release.txt:3142`).
  Moving it down is a change to eighteen files,
  so the earlier draft's "the two uses" understated it.

What moved since 2026-11-27:
nothing in the call.
The plan's fourth name,
`stacked_merge_push`,
has nothing to do here today:
its only `Action` is the planner's,
so the change that removes it is run this next,
not this one.

### What the plan says it becomes

The target tree's signature:
"Commands that only finish:
`subvolume_command`,
`doctor`,
`worktree_convert_command` and `stacked_merge_push` return
`Finished` and only the program crate's `entry` and `management` files build `Action`"
(lines 6482-6486).

The plan's prose:
"they return a small \"finished with this code and output\" value from a low file;
only the top level builds `Action`" (lines 470-471).

Which folder stops depending on which,
in the target tree:
`converting_the_main_checkout`
(target-tree-files,
line 853,
its `uses/` at 855),
`converting_a_linked_worktree` (line 828,
`uses/` at 830) and
`checking_that_your_setup_works` (line 731,
`uses/` at 733)
name `what_happens_when_you_type_a_command` in their `uses/` lists today,
where `action.rs` goes
(`the_decision_one_command_ends_in.rs`,
line 3700,
in the folder at 3649).
The change takes that name out of all three.

Which line of the allow list it removes today:
measured by the earlier draft on a branch that held the substitution,
`uses src/native/subvolume -> src/native/invocation`
(`layout-allow-list/folder-rule.txt:943`)
and `names-group src/native/subvolume src/native/invocation` (line 538),
both of which the linter then named as tolerating nothing.
`health` and `worktree` keep using `invocation` for other files,
so their lines stay.

### Signature A: one `Finished` in a low file, one constructor per stream

New file `package/git-policy/cli/src/native/foundation/a_finished_command.rs`:

```rust
//! package/git-policy/cli/src/native/foundation/a_finished_command.rs
/// What a command that only finishes hands back:
/// the exit status and the bytes of each stream.
pub(crate) struct Finished {
    /// The status the process ends with.
    pub(crate) code: i32,
    /// What the command answers,
    ///  as bytes because a report may name a path that is not text.
    pub(crate) standard_output: Vec<u8>,
    /// What it explains,
    ///  as bytes for the same reason.
    pub(crate) standard_error: Vec<u8>,
}

/// The three shapes a caller's answer takes.
impl Finished {
    /// A command that printed `text` on standard output and ended with `code`.
    pub(crate) fn saying(code: i32, text: &str) -> Finished {
        return Finished {
            code,
            standard_output: text.as_bytes().to_vec(),
            standard_error: Vec::new(),
        };
    }

    /// A command that explained itself on standard error and ended with `code`.
    /// The refusal of `migrate-to-subvolume` and the doctor's no-hooks report are these.
    pub(crate) fn explaining(code: i32, text: &str) -> Finished {
        return Finished {
            code,
            standard_output: Vec::new(),
            standard_error: text.as_bytes().to_vec(),
        };
    }

    /// A command whose progress went to the terminal as it ran and ended with `code`.
    pub(crate) fn quietly(code: i32) -> Finished {
        return Finished {
            code,
            standard_output: Vec::new(),
            standard_error: Vec::new(),
        };
    }
}
```

The three constructors are the correction to the earlier draft,
which had only `saying` and `quietly`
and so could not build either of the two refusals the code writes on standard error.

`migrate_action`,
`convert_action` and `doctor_action_with_section` return it,
and the refusal keeps its stream
(in `subvolume_command.rs`,
today's lines 353-388):

```rust
//! package/git-policy/cli/src/native/subvolume/subvolume_command.rs
/// The migration's answer.
pub(crate) fn migrate_action(
    arguments: &MigrateArguments,
    cwd: &Path,
) -> Finished {
    if arguments.help {
        return Finished::saying(0, MIGRATE_HELP);
    }
    let root: PathBuf = match main_checkout_root(cwd) {
        Ok(found) => found,
        // Today this branch is `Action::Exit { code: REFUSED_EXIT_CODE, stdout: String::new(),
        // stderr: format!(...) }` at lines 366-376, so the words stay on standard error.
        Err(message) => {
            return Finished::explaining(
                REFUSED_EXIT_CODE,
                format!("git monochromatic-git-policy-cli migrate-to-subvolume: refused: {message}\n  What to do: run the command inside the repository whose main checkout you want to convert.\nNothing was changed.\n").as_str(),
            );
        }
    };
    let code: i32 = execute(
        &LinuxPlatform::default(),
        arguments,
        root.as_path(),
        &Interrupt::never(),
        &mut say_on_standard_error,
    );
    return Finished::quietly(code);
}
```

and one real call site rewritten,
in `invocation/management.rs` where `run_migrate` builds the `Action` today (lines 476-481):

```rust
//! package/git-policy/cli/src/native/invocation/management.rs
/// Run the migration command on the platform that has btrfs subvolumes.
fn run_migrate(
    arguments: &MigrateArguments,
    cwd: &Path,
) -> Action {
    let finished: Finished = migrate_action(arguments, cwd);
    return Action::ExitBytes {
        code: finished.code,
        stdout: finished.standard_output,
        stderr: finished.standard_error,
    };
}
```

What it delivers:
the two allow-list lines named above,
measured by the earlier draft's branch and not re-measured here,
with no new use:
`subvolume` already uses `foundation`
(`uses src/native/subvolume -> src/native/foundation`,
line 942),
so the move adds no finding.

What it costs a caller:
one value to build from the answer,
and `entry.rs`'s `write_exit` stays the one place that writes a report,
so the order the wrapper has today (plan,
then write) is unchanged.
The `ENGINE_FAILURE_EXIT_CODE` move adds eighteen files to the same branch,
or a second commit in it.

Which tests change:
`subvolume_command_tests.rs:207,218,254`,
`worktree_convert_command_tests.rs:294`,
`doctor_tests.rs` (23 calls through `exit_parts` and 4 through `exit_bytes_parts`,
`rg -c "exit_parts\(" health/doctor_tests.rs` answering 24 and 5 with the two definitions counted),
`doctor_fsmonitor_tests.rs:54`,
and `management`'s own tests read `Finished` instead of `Action`.

What it leaves undone:
`stacked_merge_push`,
which run this next covers,
and the type's second move at step 7,
when `foundation` is split.
A yes to this placement does not decide whether `ENGINE_FAILURE_EXIT_CODE`
moves in the same commit or its own,
nor whether `REFUSED_EXIT_CODE` moves with it:
the latter is defined in `subvolume_command.rs` itself
and is imported by `worktree_copying/worktree_removal_of_a_bound_clone.rs:70`,
which is a use of `subvolume` from `worktree_copying` that this change does not touch.

### Signature B: the plan's own placement, beside `Action`

New file `package/git-policy/cli/src/native/invocation/what_a_command_hands_back.rs`,
next to `action.rs`:

```rust
//! package/git-policy/cli/src/native/invocation/what_a_command_hands_back.rs
/// What a command that only finishes hands back.
pub(crate) struct Finished {
    /// The status the process ends with.
    pub(crate) code: i32,
    /// What the command answers.
    pub(crate) stdout: String,
    /// What it explains.
    pub(crate) stderr: String,
}

/// The same,
///  for a command whose answer may name a path that is not text.
pub(crate) struct FinishedBytes {
    /// The exit status.
    pub(crate) code: i32,
    /// What the command answers.
    pub(crate) stdout: Vec<u8>,
    /// What it explains.
    pub(crate) stderr: Vec<u8>,
}
```

`migrate_action` returns `Finished` (its refusal in `stderr`),
`convert_action` returns `Finished`,
`doctor_action_with_section` returns `FinishedBytes`,
and one real call site rewritten,
the same one:

```rust
//! package/git-policy/cli/src/native/invocation/management.rs
/// Run the migration command on the platform that has btrfs subvolumes.
fn run_migrate(
    arguments: &MigrateArguments,
    cwd: &Path,
) -> Action {
    let finished: Finished = migrate_action(arguments, cwd);
    return Action::Exit {
        code: finished.code,
        stdout: finished.stdout,
        stderr: finished.stderr,
    };
}
```

What it delivers:
nothing to the allow list today:
the type sits in the folder `subvolume` already names for `Action`,
so `subvolume`,
`health` and `worktree` go on using `invocation`
and the linter reports no dead line.
At step 7 it delivers less than the earlier draft claimed:
of the three folders,
`converting_the_main_checkout` names `starting_git_and_describing_a_run` today
(its `uses/` line,
target-tree-files:855) and so gains nothing,
but `checking_that_your_setup_works` (line 733) and
`converting_a_linked_worktree` (line 830) do not name it,
so under this placement those two gain a use of the program folder
when they stop naming the decision folder.
The earlier draft said each "already names" it;
two of the three do not.

What it costs a caller:
one value to build from the answer,
and two types for one idea,
or one bytes type that every text caller converts with `into_bytes`.

Which tests change:
the same tests as signature A.

What it leaves undone:
the whole delivery waits for step 7,
and the two new uses above are a cost of waiting there rather than here.

### Signature C: no value at all, the command writes and returns the status

```rust
//! package/git-policy/cli/src/native/subvolume/subvolume_command.rs
/// The migration's answer,
///  written by the command itself.
pub(crate) fn migrate_action(
    arguments: &MigrateArguments,
    cwd: &Path,
    say: &mut dyn std::io::Write,
    explain: &mut dyn std::io::Write,
) -> i32 {
    if arguments.help {
        let _written = say.write_all(MIGRATE_HELP.as_bytes());
        return 0;
    }
    // Today's refusal is `Action::Exit { stdout: String::new(), stderr: format!(...) }`;
    // with two writers it stays on the second one.
    let _written = explain.write_all(refusal_text.as_bytes());
    return REFUSED_EXIT_CODE;
}
```

and one real call site,
the same `run_migrate`:

```rust
//! package/git-policy/cli/src/native/invocation/management.rs
/// Run the migration command on the platform that has btrfs subvolumes.
fn run_migrate(
    arguments: &MigrateArguments,
    cwd: &Path,
) -> Action {
    let code: i32 = migrate_action(
        arguments,
        cwd,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    return Action::Exit {
        code,
        stdout: String::new(),
        stderr: String::new(),
    };
}
```

Two writers are drawn because one is not enough:
the earlier draft's single `say` writer would have moved
`migrate-to-subvolume`'s refusal from standard error to standard output,
which a person scripting the command would see.
With two writers the command has the value's shape without the value:
the caller still hands two streams down and reads nothing back.

What it delivers:
the same two dead lines as signature A,
by the same measurement.

What it costs a caller:
the wrapper loses the one place that writes a command's answer after planning
(`entry.rs`'s `write_exit`),
so the answer is written from inside the command,
and the doctor's report leaves before the `Action` that carries it exists.
No recheck sits between the two today
(`run_process` rechecks only in its `Forward` arm),
so the visible behaviour is the same,
but a later recheck that covers exits would not see these commands.

Which tests change:
as signature A,
and every test that captures the `Action` becomes a test that captures two `Vec<u8>`.

What it leaves undone:
`stacked_merge_push`,
and the single-write-point invariant.

### Whether it is feasible

Not tried with a compiler for any of the three.
The earlier draft's check
(`cargo check --locked --offline --all-targets` on `probe/finished-results`)
replaced `Action` with an enum that had `Action`'s own variants
(`Finished::Exit`,
`Finished::ExitBytes`,
`Finished::Forward`,
`Finished::ForwardSettled`,
`Finished::ForwardPush`),
which is not the struct any signature above draws,
and it was read by the linter only.
So what is known is that the four files compile when the type name changes
and nothing else does;
nothing is known about the three signatures' own shapes,
and a green check of one of them is a round's first commit.

The one part in doubt,
whether `ENGINE_FAILURE_EXIT_CODE` can leave `action.rs` without a loop,
is settled by the plan's own item move to the basic tools,
which `foundation` is today
(`cli-git-source-layout.md`,
"Items that move down":
"`ENGINE_FAILURE_EXIT_CODE`"),
and not measured.

### Ranking

A > C > B.

The measure:
what each delivers to the split,
which is the allow list's shrinking lines in this round
and the target tree's `uses/` lists at step 7,
against what it changes for a person running the command.

- A over C:
  both remove the same two lines now and the same three names at step 7,
  and A keeps the one place that writes a report and each command's own stream,
  where C gives both up.
  A's extra cost is the type's second move at step 7,
  which is effort,
  and effort is a cost,
  not a reason to rank a signature lower.
- C over B:
  C removes two lines in this round and B removes none,
  and at step 7 C takes the name out of all three folders
  where B leaves two of them gaining a use.
  B's advantage is that two streams stay two fields,
  which C must rebuild with two writers and still cannot read back.

## 2. System settings path

### Where the call is today

- `package/git-policy/cli/src/native/repo_read/repo_read_config_scan.rs:297`:
  `scope.system_default = crate::git_system_config::system_config_for(variables);`,
  inside `ScopeEnvironment::from_variables` (line 274),
  and the value decides `scope.unresolvable` on the next line.
- The definition it calls:
  `package/git-policy/cli/src/native/repo_read/git_system_config.rs:215`,
  `pub fn system_config_for(variables: &[(OsString, OsString)]) -> Option<PathBuf>`,
  which reads the forward-target variable and asks that Git
  (`system_config_of`,
  line 114,
  and `learned_path`,
  line 145).
- The builders of a `ScopeEnvironment`,
  all ten of them
  (`rg -n "ScopeEnvironment::from_variables" -g '*.rs' -g '!*_tests.rs'`):
  `index/index_reset_file_surroundings.rs:114`,
  `repo_read/repo_read_query.rs:90`,
  `repo_read/repo_read_layout.rs:264,310,353,411`,
  `repo_read/repo_read_hook.rs:105`,
  `push/auto_push_trigger_ahead_of_upstream.rs:113`,
  `merge/merge_second_pass_tree_attributes.rs:84`,
  and `worktree/worktree_clean_content_attribute_sources.rs:93`.
  The earlier draft's eleventh,
  `subvolume/subvolume_platform.rs:133`,
  is `ConversionEnvironment::from_variables(&[])`,
  a different type that asks its own two questions.
  That matters for what this change leaves undone,
  below.

What moved since 2026-11-27:
the handover entry says
"`system_config_for` in two files of `repo_read/`".
Today there is one call site.
The round for issue 6540 moved `output_of_within_bound` out of `git_system_config.rs`
into `git_system_config_asking.rs` (`e58254244d`,
2026-11-27),
and the second file it names is that one,
which asks Git but does not call `system_config_for`.

### What the plan says it becomes

The target tree's signature:
"`ScopeEnvironment` in `repo_read_config_scan` gains the field `system_config: Option<PathBuf>`
and stops calling `git_system_config::system_config_for`;
the callers that build a `ScopeEnvironment` pass the result"
(lines 6478-6481).
The plan's prose:
"the caller passes the path in,
so a reader never starts Git"
(lines 462-463),
and the owner's own words for the same change:
"the system settings path passed to the reader"
(line 118).

Which folder stops depending on which:
none today.
The asker and the reader are two files of one folder,
and no line of the allow list names a pair between them.
In the target tree the asker is a file of its own,
`system_settings_file_of_git.rs` in `starting_git_and_describing_a_run`
(target-tree-files,
line 2602),
and the reader is `configuration_scopes_scan.rs` in `reading_git_settings_from_files`
(line 1940).
The reader's folder names the asker's folder today
(`reading_git_settings_from_files/uses/ -> ... starting_git_and_describing_a_run ...`,
line 1936),
and that line stays unless every reader file is handed the value or stops asking.
What this change delivers now is therefore not a folder line
but the promise in one folder's name:
the files that answer without starting Git stop starting it,
for the files that are handed a value.

### Signature A: the plan's parameter

```rust
//! package/git-policy/cli/src/native/repo_read/repo_read_config_scan.rs
/// The scope a reader resolves its configuration files from.
impl ScopeEnvironment {
    /// The facts of an environment as a child would see it,
    /// with the system configuration file's path,
    ///  which the caller learned from that Git.
    pub fn from_variables(
        variables: &[(OsString, OsString)],
        system_default: Option<PathBuf>,
    ) -> ScopeEnvironment {
        let mut scope: ScopeEnvironment = ScopeEnvironment {
            home: nonempty_variable(variables, "HOME"),
            xdg: nonempty_variable(variables, "XDG_CONFIG_HOME"),
            no_system,
            system: path_variable(variables, "GIT_CONFIG_SYSTEM"),
            system_default,
            global: path_variable(variables, "GIT_CONFIG_GLOBAL"),
            unresolvable: !boolean_ok,
        };
        // Git reads its compiled-in system file unless the environment removes or replaces it,
        // and a path nobody could learn leaves the scope unresolvable, as it does today.
        if boolean_ok && !scope.no_system && scope.system.is_none() && scope.system_default.is_none() {
            scope.unresolvable = true;
        }
        return scope;
    }
}
```

and one real call site rewritten,
`repo_read/repo_read_layout.rs:264`:

```rust
//! package/git-policy/cli/src/native/repo_read/repo_read_layout.rs
let scope: ScopeEnvironment = ScopeEnvironment::from_variables(
    variables.as_slice(),
    crate::git_system_config::system_config_for(variables.as_slice()),
);
```

What it delivers:
the reader's file stops starting Git,
and the ask moves to the ten call sites.
No allow-list line changes now.
At step 7 the reader's folder can stop naming the asker only if the three reader files
that build their own scope
(`repo_read_query.rs:90`,
`repo_read_hook.rs:105`,
`repo_read_layout.rs:264`)
are handed a value too,
which is question 4.

What it costs a caller:
one line each,
and a caller whose variables name no real Git passes `None`,
which is the answer it gets today from the same variables,
so no read changes its answer.
A failed ask is retried on the next call,
as today.

Which tests change:
`repo_read_config_scan_tests.rs` (7 calls,
`rg -n "from_variables" repo_read/repo_read_config_scan_tests.rs`),
`git_system_config_tests.rs:256-301`
(the control that builds scopes from a fixture Git,
named `the_scope_reads_the_file_the_git_names`,
which asserts a learned path and an unknown one in the same body).

What it leaves undone:
`system_attributes_for`,
the same shape for `$(prefix)/etc/gitattributes`,
imported at `worktree/worktree_clean_content_attribute_sources.rs:18` and asked at line 194
inside `ConversionEnvironment::from_variables`,
whose three test calls are `worktree_clean_content_safety_tests.rs:714,719,724`.
So "the reader never starts Git" is not delivered for conversions by this change alone,
whoever's signature is chosen.

### Signature B: two types, one settled

```rust
//! package/git-policy/cli/src/native/repo_read/repo_read_config_scan.rs
/// The configuration scopes an environment settles without asking Git.
pub struct ScopesAwaitingTheSystemFile {
    /// The home folder.
    home: Option<PathBuf>,
    /// The XDG configuration folder.
    xdg: Option<PathBuf>,
    /// Whether the system file is switched off.
    no_system: bool,
    /// `GIT_CONFIG_SYSTEM`.
    system: Option<PathBuf>,
    /// `GIT_CONFIG_GLOBAL`.
    global: Option<PathBuf>,
    /// Whether a fact could not be settled.
    unresolvable: bool,
}

/// The two steps that settle a scope.
impl ScopesAwaitingTheSystemFile {
    /// Every fact the variables alone settle.
    pub fn from_variables(variables: &[(OsString, OsString)]) -> ScopesAwaitingTheSystemFile;
    /// The scopes,
    ///  with the system file's path learned by a file that may start Git.
    pub fn with_the_system_file(self, system_default: Option<PathBuf>) -> ScopeEnvironment;
}
```

and one real call site,
the same one:

```rust
//! package/git-policy/cli/src/native/repo_read/repo_read_layout.rs
let scope: ScopeEnvironment = ScopesAwaitingTheSystemFile::from_variables(variables.as_slice())
    .with_the_system_file(crate::git_system_config::system_config_for(variables.as_slice()));
```

What it delivers:
the same as A,
and a type that cannot be read before the system file is settled,
so no later code can call the reader and get a scope that silently declines.

What it costs a caller:
two calls instead of one,
and one more public type in the folder that has to move at step 7.

Which tests change:
as A,
plus every test that names `ScopeEnvironment::from_variables` directly.

What it leaves undone:
`system_attributes_for`,
as in A.

### Signature C: one answer per process, learned once

```rust
//! package/git-policy/cli/src/native/repo_read/git_system_config.rs
/// The path learned once for this process.
static THE_SYSTEM_CONFIG_OF_THIS_GIT: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();

/// Learn the system configuration file's path once,
///  from the Git this process runs.
pub fn remember_the_system_config_for_this_git(variables: &[(OsString, OsString)]) {
    let _learned: &Option<PathBuf> = THE_SYSTEM_CONFIG_OF_THIS_GIT
        .get_or_init(|| system_config_for(variables));
}

/// The path learned earlier,
///  or nothing where nobody learned one.
pub fn the_system_config_remembered() -> Option<PathBuf> {
    return THE_SYSTEM_CONFIG_OF_THIS_GIT.get().cloned().flatten();
}
```

with `remember_the_system_config_for_this_git` called once at the service's start,
and one real call site,
the same one:

```rust
//! package/git-policy/cli/src/native/repo_read/repo_read_config_scan.rs
// Today line 297. Nothing at this site changes.
scope.system_default = crate::git_system_config::the_system_config_remembered();
```

What it delivers:
no call site changes,
so no caller gains an upward use,
and nothing to the allow list or to the target tree's `uses/` lists:
the remembered value lives in `git_system_config.rs`,
which becomes `system_settings_file_of_git.rs`
in `starting_git_and_describing_a_run` (target-tree-files,
line 2602),
and the reader calls it,
so `reading_git_settings_from_files` goes on naming
`starting_git_and_describing_a_run` (its `uses/` line,
1936).
The earlier draft's claim that C
"takes the ask out of the reader's folder entirely"
holds only if the remembered value lives in a file below the reader,
which is not drawn.
C is therefore not a pass of the owner's words either:
those say "the caller passes the path in" (line 462),
and a process-wide value is not something a caller passes.

What it costs,
and why this revision ranks it last:

1.  The precedent is not one.
   The earlier draft cited `invocation/git_version_cache.rs` as the repository's one process-wide cache.
   It holds no `static`,
    `OnceLock`,
    `Mutex` or `thread_local`
   (`rg -n "static|OnceLock|Mutex|thread_local" invocation/git_version_cache.rs` prints nothing).
   It is an on-disk cache keyed by the executable's identity,
   and its own doc says the opposite property from C's:
   "an upgrade replaces the file (new inode) or rewrites it (new size,
    times),
   so a replaced Git is noticed on the next start and a stale entry is never trusted"
   (lines 10-14).
   `system_config_for` already works that way,
   through `learned_path` (line 145).
2.  The guide forbids the shape for anything a test reads.
   Nothing a test changes may be process-wide
   (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
   section "Nothing a test changes may be process-wide"),
   and a `OnceLock` cannot be reset,
   so in a threaded run the first test to learn a path decides for every later one.
   "Learn one first" is not a thing a test can do:
   the existing control `the_scope_reads_the_file_the_git_names`
   runs its body in a child of its own and asserts a learned path and an unknown scope
   one after the other in the same body
   (`git_system_config_tests.rs:256-301`),
   which a value once set can never do again.
   Every other test of a system file would have to join it in a child process.
3.  A failed ask would be remembered.
   Today each call retries;
    `get_or_init` keeps the first answer,
    `None` included.
4.  The first caller's variables would decide for every later caller,
   and the service runs more than one Git.
5.  The ask blocks its caller (`output_of_within_bound` starts a child),
   so inside a `OnceLock` closure it would block every other reader for up to the wait's ceiling,
   possibly on a runtime worker.
6.  A step that never resolves leaves it `None` for good:
   at `run_process`'s `Ok(inputs)` arm (`invocation/entry.rs:452-460`)
   the wrapper has resolved nothing about the real Git,
   `ResolutionInputs` carries no Git path
   (`invocation/real_git.rs:105-126`),
   and the process's own environment carries no forward-target variable
   (`repo_read_config_scan.rs:270-272` says the wrapper's own environment does not carry it),
   so a call placed there would store `None`
   and `get_or_init` would keep it.

Which tests change:
every test that expects a scope to name a system file would need a reset seam and a child process,
and the guide's rule says the seam itself must refuse outside the child.

What it leaves undone:
`system_attributes_for`,
unless it is remembered the same way,
and with it a second process-wide value.

### Whether it is feasible

A and B are feasible on the reading and not tried with a compiler.
C is feasible to compile and not to test under the guide's rule.
The cheap compiler answer (every caller passes `None`) would compile
and prove nothing about the question,
which is what each caller can pass.
That was answered by reading the callers:
every one of the ten holds the variables the ask reads
(`with_real_git` documents that the forward-target variable is in every child's environment,
and `repo_read_query.rs:90` builds a scope from the base variables its caller holds),
and `subvolume_platform.rs:212,225` builds its `ConversionEnvironment`
from the same kind of list,
so the two conversions' answers are unchanged by A.

### Ranking

A > B > C.

The measure:
what each delivers to the split,
which here is whether the reader's folder can stop naming the asker's at step 7,
against what it costs every test of a system file.

- A over B:
  both deliver the same and leave the same open,
  and A is one fact in one place,
  where B adds a public type whose guarantee no caller needs
  and makes every builder write two calls.
- B over C:
  neither delivers the folder line as drawn,
  and B at least keeps each caller's own answer,
  its own retry,
  and the guide's rule about process-wide facts,
  where C loses all three and keeps the folder edge anyway.

No question asks which signature here:
A is the plan's own and the owner's recorded words,
so this record takes it and does not put it.
Questions 4 and 5 ask what A leaves open.

## 3. After-stop action

### Where the call is today

- `package/git-policy/cli/src/native/push/child_watch.rs:195`:
  `return Self::start_with_locks(waiting_for, pid, terms, LockLedger::none());`,
  the body of `start`;
  `start_with_locks(waiting_for, pid, terms, locks: LockLedger)` at line 206;
  `start_under_stop(waiting_for, pid, terms, locks: LockLedger, stop: Option<StopArm>)` at line 227;
  `start_when_a_task_may_run(..., locks: LockLedger, ...)` at line 260;
  and `watch_loop(incoming, flags, subject, pid, terms, locks, stop, root_pin)` at line 302.
  `impl Drop for ChildWatch` is at line 474.
- `package/git-policy/cli/src/native/push/child_watch_task_loop_and_notice.rs:76`:
  `locks: LockLedger,` as a parameter of `watch_loop`,
  passed to `stop_process` at lines 162 and 180,
  and to `end_for_the_stop(..., locks: &LockLedger, ...)` at lines 123 and 197,
  whose body starts at line 314 and calls `stop_process` again at line 324.
  A watch that is dropped closes its channel,
  and that path sweeps:
  the comment inside `stop_process` says the locks are
  "swept then,
  or when the watch is dropped,
  whichever comes first".
- `package/git-policy/cli/src/native/push/child_watch_clock_marks_and_stopping.rs:308`:
  `pub(crate) async fn stop_process(pid: u32, locks: &LockLedger, incoming: &mut UnboundedReceiver<Mark>, root_pin: &RootPin)`,
  with the sweep at lines 372-383,
  which runs through `crate::blocking_job::run_blocking_job(move || { sweeping.sweep(); })`
  because,
  as the comment above it says,
  `the owner, 2026-11-07, "Fail plainly": no inline run of a call that can wait on storage`.
- The two ledger builds inside the watcher:
  `push/child_watch_one_shot_run.rs:238` and
  `push/child_watch_output_watched_on_the_runtime.rs:243`,
  both `let locks: LockLedger = LockLedger::before_start(command);`.
- The five one-shot run functions,
  not four (`rg -n "pub fn output_watched|pub async fn output_watched_within_for_a_stop_awaiting" push/`):
  `output_watched` (line 99),
  `output_watched_stopping` (121),
  `output_watched_within` (160),
  `output_watched_within_for_a_stop` (196),
  and the awaited `output_watched_within_for_a_stop_awaiting` (runtime file,
  line 202).
  All five return `std::io::Result<Output>` today.
- The callers that already build a ledger:
  `push/bounded_push.rs:421`,
  `push/bounded_query_run_of_one_command.rs:244`,
  and `push/stop_of_a_push_and_sweep_of_its_locks.rs:162,172,381,456`.
- The callers that hand the watcher no ledger of their own:
  `lfs/lfs_upload_child_run.rs:369`,
  `process/child_awaited_until_it_exits_or_the_service_is_stopped.rs:46`,
  and `process/child_awaited_until_it_exits_the_stop_or_the_ceiling.rs:328,373`.
- `repo_read/git_metadata.rs:443` is not one of those.
  It calls `output_watched_within`,
  which reaches `LockLedger::before_start` at `child_watch_one_shot_run.rs:238`,
  so that read has a ledger today and the watcher sweeps its ended reads.
  The earlier draft said it "pass[es] no ledger at all";
  that was wrong.
  The one caller of `ChildWatch::start` that drops a watch with no ledger is
  `candidate/candidate_reader.rs:315`
  (`rg -n "ChildWatch::start\(" -g '*.rs' -g '!*_tests.rs'` answers that one line),
  which the earlier draft did not cite.

What moved since 2026-11-27:
the handover says "`LockLedger` in six files of `push/`".
Today eight release files of `push/` name it,
because round 2's split cut `child_watch.rs` into five files,
and `lfs/` and `process/` name it too.

### What the plan says it becomes

The target tree's signature:
"`ChildWatch::start_with_locks(waiting_for, pid, terms, locks)` and `watch_loop(... locks)` lose the
`locks: LockLedger` parameter,
and the line in `child_watch` that calls `LockLedger::before_start(command)` moves out.
The watcher returns the `StopReport` it already has;
the caller in `bounded_push` and `transaction_git` builds the
`LockLedger` before the start and sweeps after the stop.
The lock folder is in the program,
above the watcher library,
so the watcher cannot name it"
(lines 6467-6477).

The plan's prose and the owner's answer:
"its caller hands it what to do after a stop" (line 458),
from the answer "Make all four (Recommended)" (lines 116-120).

Which folder stops depending on which:
no line of the allow list,
measured by the survivors:
`push` keeps `lock` through `bounded_push.rs:421`,
`bounded_query_run_of_one_command.rs:244` and the sweep file.
What it delivers is at the library boundary.
The five `child_watch` files are the files the rescan sends to
`starting-and-watching-programs`,
and that library folder's `uses/` line names `clearing_leftover_reference_locks`
(target-tree-files,
line 322,
the same name at 279 in the sibling folder).
The change is what takes it out of the folder at 322,
which the plan itself counts on:
the watcher's cleanliness "depends on one change of the draft"
(target tree,
"Changes so that a library stands alone").
It does not take the name out of the folder at 279:
`process/child_awaited_until_it_exits_or_the_service_is_stopped.rs:46` and
`process/child_awaited_until_it_exits_the_stop_or_the_ceiling.rs:328,373` keep
`stop_push(owned, STOP_GRACE, LockLedger::none())`,
which neither signature changes,
so `starting_programs_catching_signals_and_keeping_their_output`
keeps using `clearing_leftover_reference_locks`.
The earlier draft claimed the change takes the name out of the library;
that holds for one of the library's two folders.

### Signature A: the caller sweeps, the watch says how it ended

```rust
//! package/git-policy/cli/src/native/push/child_watch.rs
/// The faces of the watcher.
impl ChildWatch {
    /// Start watching `pid`,
    ///  a child that holds no Git lock.
    pub fn start(waiting_for: &str, pid: u32, terms: WatchTerms) -> Self;

    /// Start watching `pid`,
    ///  ended when the service's stop is told through `stop`.
    pub fn start_under_stop(
        waiting_for: &str,
        pid: u32,
        terms: WatchTerms,
        stop: Option<crate::stop_of_the_service_noticed_by_its_children::StopArm>,
    ) -> Self;
}

// package/git-policy/cli/src/native/push/child_watch_task_loop_and_notice.rs
/// The watcher's loop.
pub(crate) async fn watch_loop(
    incoming: UnboundedReceiver<Mark>,
    flags: WatchFlags,
    waiting_for: String,
    pid: u32,
    terms: WatchTerms,
    stop: Option<StopArm>,
    root_pin: crate::child_watch_clock_marks_and_stopping::RootPin,
);

// package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs
/// The output of a watched step,
///  and how the watch ended it.
pub(crate) struct WatchedOutput {
    /// What the step wrote,
    ///  or why its output could not be read.
    pub output: std::io::Result<std::process::Output>,
    /// Whether the watch itself ended the child,
    ///  which is when its locks must be swept.
    pub ended_by_the_watch: bool,
}
```

All five one-shot functions change their answer to
`std::io::Result<WatchedOutput>`,
which the earlier draft never wrote;
without it the call site below does not type,
because `Output` has no `.output` field.

and one real call site rewritten,
`transaction/transaction_git.rs:386`,
which today reads
`let watched: std::io::Result<std::process::Output> = output_watched_within_for_a_stop(`:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git.rs
let locks: LockLedger = LockLedger::before_start(&step.command);
let watched: WatchedOutput = output_watched_within_for_a_stop(
    &mut step.command,
    step.subject.as_str(),
    step.ledger_step.as_str(),
    step.terms,
    step.stoppable,
    step.bound,
    step.a_stop_lets_it_finish,
    request.started_child.as_ref(),
)?;
// The caller is on the runtime here, and `sweep` reads and removes files,
// so it goes to the blocking pool the watcher used for the same work.
if watched.ended_by_the_watch {
    crate::blocking_job::run_blocking_job(move || { locks.sweep(); }).await?;
}
return Ok(step_finished(
    step.ledger_step.as_str(),
    request,
    step.began,
    Ok(watched.output?),
));
```

What it delivers:
`clearing_leftover_reference_locks` out of the watcher library folder at 322,
which is what the library lift needs;
no allow-list line today.

What it costs a caller:
every caller of the five one-shot functions reads a `WatchedOutput`,
and every caller that may sweep must run the sweep on the blocking pool itself,
because `LockLedger::sweep` reads and removes files
and the owner's rule of 2026-11-07 quoted above forbids the inline call.
A caller that drops the watch without reading its end sweeps nothing,
where today the watcher's task sweeps when the channel closes:
that is the early return in `child_watch_one_shot_run.rs`
for a step left running (`StepBound::LeaveRunning`,
lines 341-360),
which ends the watch,
forgets the captured output and answers
`Err(std::io::ErrorKind::TimedOut)`,
and every other path that ends a watch without reading it.
A sweep lost that way is not retried:
`settle_locks`'s own doc says
"a sweep before that finds the tree running and removes nothing,
and nothing would try again"
(`push/stop_of_a_push_and_sweep_of_its_locks.rs:432-440`),
so the lock waits for the next command's stale-lock judgement.

Which tests change:
`child_watch_tests.rs` (its `watch_loop` call and its five `output_watched` calls),
`child_watch_drop_time_measurement_tests.rs:183`,
`child_watch_output_watched_on_the_runtime_tests.rs`,
`child_watch_one_shot_through_pipes_tests.rs`,
`bounded_push_tests.rs:384,444`,
`bounded_push_awaiting_tests.rs:73`,
`stop_of_a_push_and_sweep_of_its_locks_tests.rs:53,95`,
and `lock/git_stop_lock_tests.rs`,
whose `watched_step` (line 917) calls `output_watched` (line 935)
and asserts the lock state after a stop.
The earlier draft listed the `stop_push` controls,
which A does not change,
and left this one out.

What it leaves undone:
the dropped-watch sweep,
and `child_watch_running_task.rs`,
whose task holds the flag the caller reads.

### Signature B: the watcher is handed what to do after a stop

The library defines the behaviour,
the program implements it,
which is the plan's prose and the owner's answer of 2026-11-04:

```rust
//! in the watcher library,
//!  later package/starting-and-watching-programs/core-rust
/// What a caller wants done once the watch has ended its child.
pub trait AfterTheStop: Send + Sync {
    /// Run after the child and the processes below it are gone.
    fn after_the_stop(&self);
}

/// The signatures this record proposes.
impl ChildWatch {
    /// A watch that calls the caller once it has ended its child.
    pub fn start_after_the_stop(
        waiting_for: &str,
        pid: u32,
        terms: WatchTerms,
        after: std::sync::Arc<dyn AfterTheStop>,
    ) -> Self;
}
```

and the same real call site,
`transaction/transaction_git.rs:386`:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git.rs
/// The ledger the program hands the watcher.
struct SweepTheLocks {
    /// The ledger.
    ledger: LockLedger,
}

/// What the program wants done once its child is gone.
impl crate::child_watch::AfterTheStop for SweepTheLocks {
    /// Sweep the locks of the stopped step.
    fn after_the_stop(&self) {
        self.ledger.sweep();
    }
}

let watch: ChildWatch = ChildWatch::start_after_the_stop(
    step.subject.as_str(),
    child.id(),
    step.terms,
    std::sync::Arc::new(SweepTheLocks {
        ledger: LockLedger::before_start(&step.command),
    }),
);
```

What it delivers:
the same library entry as A,
and the sweep stays inside the watcher's task,
where it runs through `run_blocking_job` as it does today,
and where a dropped watch still sweeps.
A caller reads no flag,
so no caller can lose the sweep by returning early.

What it costs a caller:
an allocation and a type per call site,
and a behaviour handed across a crate boundary,
which the plan's own note disfavours for these changes:
"none of the changes below hands a closure;
each hands a value or an enum"
(target tree,
lines 6378-6382).
A trait object is not a closure,
but it is the same shape and the same cost to reason about,
and the library would define a trait whose only implementation is the program's.
Whether `LockLedger` can be the state of a `Send + Sync` implementation is untried:
it is `Clone, Debug` and holds `Vec<PathBuf>` and an optional judge
(`lock/stopped_git_locks.rs:95-103`),
and it is already `Send` because `stop_process` moves a clone into a blocking job;
nothing shows it is `Sync`,
and a compile is what says.

Which tests change:
as A,
plus a stand-in `AfterTheStop` in every control that starts a watch.

What it leaves undone:
nothing the change names,
and the watcher's own task grows one call it cannot see into.

### Whether it is feasible

The only compiler check is of A's removal,
not of A:
the throwaway branch `probe/after-stop-action-a` removed the `locks` parameter
from the faces and from `stop_process`,
removed the sweep and the ledger builds,
and dropped the argument from the test call sites.
`cargo check --locked --offline --all-targets` in `package/git-policy/cli` was run on it twice,
under a scope capped at 10G of memory and 4 CPUs;
the first run answered two errors in
`child_watch_drop_time_measurement_tests.rs:183`,
where a measurement builds a ledger and had been getting the type through
`child_watch.rs`'s import,
and the second run printed no error and ended with the compiler's own line,
whose backticks are the compiler's:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 43.32s
```

That shows the watcher files compile without a ledger.
It does not show that the callers can get a flag,
that the awaited face can sweep,
or that a control of the watcher's sweep still means anything;
none of those was compiled.
Neither signature's own shape has been through a compiler.

### Ranking

B > A.

The measure:
what each delivers to the split,
which here is the library folder's `uses/` line,
against what it changes for a stopped step whose locks are swept.

- B over A:
  both deliver the same library entry,
  and B keeps the sweep where it runs today,
  inside the watcher's task on the blocking pool,
  and keeps a dropped watch swept;
  A moves the sweep to every caller,
  which must each reach the blocking pool themselves,
  and loses the sweep on the paths that end a watch without reading it.
  A's advantage is a plain value and a flag instead of a trait object,
  which is the shape the target tree asks for,
  and B's cost is the trait whose only implementation is the program's.

This is the earlier draft's ranking reversed.
It rested on `repo_read/git_metadata.rs:443` passing no ledger,
which is false:
that read reaches `LockLedger::before_start`
through `output_watched_within` and its ended reads are swept today.
The one path that drops a watch with no ledger is `candidate_reader.rs:315`,
where nothing is swept today and nothing would be swept under either signature.

## 4. Current step record

### Where the call is today

- `package/git-policy/cli/src/native/transaction/transaction_git_step_about_to_start.rs:229`:
  `enter_step(format!("{} in {}", command_text(&request.arguments), request.cwd.display()))`,
  held in the returned `StepAboutToStart` as `record` and dropped with the step.
- `package/git-policy/cli/src/native/transaction/transaction_git_step_bounds.rs:113`:
  `if crate::service_busy_step_record_on_this_thread::on_landing_thread() {`,
  inside `not_ended_bound`,
  which `step_bound(request: &GitRequest)` reaches at line 80.
  The request is already the argument of `step_bound`,
  so a fact the bound needs
  is a fact the request can carry.
- `package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs:230` and
  `child_watch_output_watched_on_the_runtime.rs:235`:
  `crate::service_busy_step_record_on_this_thread::on_landing_thread(),`
  as the first argument of `begin_unfinished_work`.
- The record itself:
  `package/git-policy/cli/src/native/service/service_busy_step_record_on_this_thread.rs`
  (`on_landing_thread` at line 109,
  `use_on_this_thread` at 114,
  `enter_step` at 176),
  which reads and writes through
  `task_local! { static LANDING: LandingState; }`
  (`landing/landing_state_carried_by_its_task.rs:44-48`)
  and names `service_busy`'s
  `LandingBoard`,
  `STEP_FILE` and `now_millis`.
- The landing that registers its board:
  `package/git-policy/cli/src/native/service/service_server.rs:174`:
  `crate::service_busy_step_record_on_this_thread::use_on_this_thread(Arc::clone(&board));`,
  and the detail a landing sets:
  `landing/service_land_attempt_branch_move.rs:99`.

What moved since 2026-11-27:
nothing in the call.
What the state is has moved since the plan was written:
it is task-carried,
not thread-local.
The owner's choice 4 was "Rewrite per-thread state",
and the plan that records it says each `thread_local!` becomes a task-local or a context value
(`doc/planning/git-policy-cli-full-tokio.md`,
lines 1655-1657),
and lists "the board and the detail of a never-ended write"
among the ten that went into the task-local (lines 3370-3376).
The target tree's word "thread-local" predates that.
A thread-local would not misread today,
because the landing is one synchronous loop,
but it would the day the landing runs as a task,
which `in_a_landing_task` exists for.

### What the plan says it becomes

The target tree's signature:
"`transaction_git` stops calling `service_busy::enter_step` and `on_landing_thread`.
A thread-local record (step file path and detail) moves to the new program file `current_step_record`
in the folder `how_long_the_program_waits_for_git_steps`
(it names Git steps,
so it does not go to a library);
the runner writes it,
the landing code
registers its file path there,
and `LandingBoard` stays in `service_busy`.
The two functions listed as removed in the
group map are `service_busy_step_record_on_this_thread::enter_step` and `::on_landing_thread`"
(lines 6455-6466).

The plan's prose:
"a small \"current step\" record in the basic tools that the runner writes and the service reads"
(line 455),
which is a third placement again.

Which folder stops depending on which:
no line of the allow list today,
measured by the survivor:
`transaction` keeps `service` through `wait_ceilings`
(`transaction/transaction_phase.rs:42`,
`transaction_git_step_bounds.rs:29,91,93`).
At step 7 `wait_ceilings` becomes `longest_waits.rs` in `how_long_the_program_waits_for_git_steps`
(target-tree-files,
line 1218),
and then the transaction folder uses only that low folder.
The delivery that matters now is the library's:
`landing_commits` out of the watcher library's `uses/` line (line 322),
which needs the two push call sites in this change,
not only `transaction_git`.

### Signature A: the record as it is, moved, still task-carried

```rust
//! package/git-policy/cli/src/native/service/current_step_record.rs,
//! later how_long_the_program_waits_for_git_steps/current_step_record.rs
/// The step the landing this task serves is inside,
///  which is the one operation that waits for a step it cannot end.
///  It is carried by the task,
///  not by the thread,
///  because a task moves between them.
tokio::task_local! {
    static THE_STEP_OF_THIS_LANDING: std::cell::RefCell<Option<LandingWrites>>;
}

/// Where one landing writes its step.
struct LandingWrites {
    /// The folder the step file lives in.
    directory: PathBuf,
    /// What the write that follows does.
    detail: Option<String>,
}

/// Say where the landing this task serves writes the step it is inside.
pub(crate) fn a_landing_writes_its_step_to(directory: PathBuf);

/// Whether the calling task serves a landing.
pub(crate) fn on_a_landing() -> bool;

/// Say what the write that follows does,
///  for as long as the guard lives.
pub(crate) fn detail(text: String) -> DetailGuard;

/// Record that this task is inside a step that is never ended,
///  until the guard is dropped.
pub(crate) fn enter_step(command: &str) -> Option<StepGuard>;
```

and one real call site rewritten,
`transaction/transaction_git_step_about_to_start.rs:229`:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git_step_about_to_start.rs
// Today's line calls `enter_step` unconditionally and the record itself decides
// whether this task is a landing; this keeps that and only moves where the record lives.
let record: Option<crate::current_step_record::StepGuard> = crate::current_step_record::enter_step(
    format!("{} in {}", command_text(&request.arguments), request.cwd.display()).as_str(),
);
```

and the two watched-run sites take the answer in the terms,
because the watcher may not name the program's record:

```rust
//! package/git-policy/cli/src/native/push/child_watch_clock_marks_and_stopping.rs
/// The terms a watch runs under.
pub struct WatchTerms {
    // ... the quiet window, the notice grace and where notices go ...
    /// Whether the caller serves a landing,
    ///  where a step is never ended.
    pub on_a_landing: bool,
}

/// The terms a step of `class` runs under,
///  with the landing answer the caller's request holds.
pub fn for_class(class: ChildClass, on_a_landing: bool) -> Self;

// package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs
let mut unfinished: Unfinished = match begin_unfinished_work(terms.on_a_landing, waiting_for) { /* ... */ };
```

What it delivers:
`landing_commits` out of the watcher library's `uses/` line (322),
and,
after step 7,
the transaction folder's use of `service` gone.
No allow-list line today.

What it costs a caller:
the landing registers a directory instead of a board (`service_server.rs:174`),
the record grows its own clock and its own step file name,
replacing `now_millis` and `STEP_FILE` from `service_busy`,
and every maker of `WatchTerms` sets the new field:
`for_class` has 6 callers and `WatchTerms {` is written 24 times in 7 test files
(`rg -c "WatchTerms \{" -g '*_tests.rs'`).
The record stays state no line records,
which is what the folder rule cannot see and what let the tangle grow.

Which tests change:
`service_busy_tests.rs`,
`transaction_git_tests.rs`,
`service_land_hung_write_tests.rs`,
`service_land_tests.rs`,
and `child_watch_tests.rs`,
beside the seven files that write a `WatchTerms` literal.

What it leaves undone:
`LandingBoard`'s reader of the step file
(`service/service_busy_words_of_a_stuck_write.rs:33`) reads the path the record writes,
so the file's name is known in two places,
and `git_metadata.rs:443`'s read,
which is built with no request in scope
(the function takes arguments,
an overlay,
a ceiling and an ending),
would pass `false` where today a read on the landing answers `true`.
That is a real loss of a safety default,
and it is A's too:
the watched read has no request to ask,
so its answer has to be handed to it
by whoever built its command.

### Signature B: the record is a value the request holds

The carrier is `GitRequest`,
the type the runner already has,
not a new request type:
`GitRequest` is `#[derive(Clone, Debug, Default)]`
(`transaction/transaction_git_request_and_reply_types.rs:89`)
and a record of a folder and an optional detail is `Default` too,
so no lifetime and no constructor changes.

```rust
//! package/git-policy/cli/src/native/service/current_step_record.rs,
//! later how_long_the_program_waits_for_git_steps/current_step_record.rs
/// Where one landing writes the step it is inside,
///  and what that step does.
#[derive(Clone, Debug, Default)]
pub struct CurrentStepRecord {
    /// The folder the step file lives in,
    ///  which is empty when this step runs anywhere but a landing.
    directory: Option<PathBuf>,
    /// What the write that follows does.
    detail: Option<String>,
}

/// The record's own steps.
impl CurrentStepRecord {
    /// The record of the landing that writes into `directory`.
    pub fn in_the_landing_at(directory: PathBuf) -> CurrentStepRecord;
    /// The same record,
    ///  saying what the next write does.
    pub fn doing(&self, text: String) -> CurrentStepRecord;
    /// Record that this task is inside a step that is never ended,
    ///  until the guard is dropped.
    pub fn enter(&self, command: &str) -> Option<StepGuard>;
    /// Whether this step runs on a landing,
    ///  where it is never ended.
    pub fn on_a_landing(&self) -> bool;
}
```

and the request carries it:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git_request_and_reply_types.rs
/// One step of a transaction as its runner receives it.
#[derive(Clone, Debug, Default)]
pub struct GitRequest {
    // ... the arguments, the directory, the class and the terms, as today ...
    /// The landing's step record,
    ///  which the service hands to every step it runs;
    ///  the default is a step outside a landing.
    pub step_record: CurrentStepRecord,
}
```

and one real call site rewritten,
the same `step_about_to_start`,
together with the bound the request now decides:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git_step_about_to_start.rs
let record: Option<crate::current_step_record::StepGuard> = request.step_record.enter(
    format!("{} in {}", command_text(&request.arguments), request.cwd.display()).as_str(),
);

// package/git-policy/cli/src/native/transaction/transaction_git_step_bounds.rs
/// The bound of a step that is never ended:
///  unbounded on a landing,
///  else left running after the class ceiling plus the short extra wait.
fn not_ended_bound(class_ceiling: Duration, on_a_landing: bool) -> StepBound {
    if on_a_landing {
        return StepBound::Unbounded;
    }
    return leave_running_bound(class_ceiling);
}
```

What it delivers:
the same as A,
and the value makes the use visible to the folder rule,
so a program folder cannot reach the record without naming it,
and the watcher's answer arrives with the record instead of beside it.

What it costs a caller:
a field on the type that 503 lines of 143 files name
(`rg -n "GitRequest" -g '*.rs' | wc -l` and `rg -l ... | wc -l`,
122 of the files release ones),
which is a cost only where a request is built for a landing:
the `Default` keeps every other construction as it is.

The name `StepRecord` is taken:
`account_companion/companion_records.rs:45` already declares `pub struct StepRecord`,
and the crate re-exports flat today,
so a second type of that name would invite an ambiguous path at step 6.
`CurrentStepRecord` is free.

Which tests change:
as A,
plus every test that builds a request for a landing,
and the 24 `WatchTerms {` literals and 6 `for_class` callers either way.

What it leaves undone:
the same file-name duplication as A,
unless the board is handed the name from the record,
and `git_metadata.rs:443`'s read,
which still needs its answer handed to it:
that read takes no request,
so its callers must pass the flag down,
and that is a parameter through `git_metadata`'s own callers.

### Whether it is feasible

A is feasible on the reading and not tried with a compiler.
B is feasible on the reading and not tried.
The doubt was whether the new low file can write the step file without naming `service_busy`,
and it can:
`enter_step` needs a directory,
a detail and a clock
(`service_busy_step_record_on_this_thread.rs:176-190`),
all three of which the record can hold,
and `STEP_FILE` is a constant whose value can sit beside the writer.
The other doubt,
whether the bound can read the request,
is settled by `step_bound(request: &GitRequest)` already taking it.

### Ranking

B > A.

The measure:
what each delivers to the split,
which is the same library line for both,
against what it changes for a step that may start after a stop.

- B over A:
  both remove the same use,
  and B removes it in a way the folder rule can see,
  so the tangle cannot regrow through a name no line records;
  B also gives the watcher its answer in the same value,
  where A needs a second field beside the task-local state.
  A's advantage is real:
  a field on the request is more sites than a task-local,
  and it is effort,
  which is a cost and not a reason to rank lower.

## 5. Run this next

### Where the call is today

The push half:

- `package/git-policy/cli/src/native/push/stacked_merge_push.rs:36`:
  `use crate::entry::plan_invocation;`,
  and line 40 `run_stacked_push,`;
  `pub fn gated_merge_push(...) -> Result<i32, SingleFlightError>` at line 209,
  whose last two lines are
  `let action: Action = plan_invocation(arguments.as_slice(), environment.as_slice(), &inputs);`
  and `return Ok(run_planned(action));` (lines 226-227).
- `fn run_planned(action: Action) -> i32` at line 119,
  whose `Action::ForwardPush` arm calls `run_stacked_push(&forward, &gated_merge_push)` at line 161.
- `package/git-policy/cli/src/native/push/push_stacking.rs:353`:
  `pub fn run_stacked_push(forward: &PushForward, merge_push: MergePush<'_>) -> i32`,
  whose loop state is `StackRequest<'a>` (line 110),
  whose step is `stack_failed_push(request: &StackRequest<'_>, failed_code: i64, err: &mut dyn Write) -> Option<i32>` (line 195),
  and whose decision is taken by its answer.
- The top level that would run the next command:
  `invocation/entry.rs:580` (`return run_stacked_push(&forward, &gated_merge_push);`)
  and `invocation/entry_run_forwarded_actions.rs`,
  which `entry.rs:543` reaches through `run_coordinated_action(&forward)`.

The merge half:

- `package/git-policy/cli/src/native/merge/merge_second_pass_finish.rs:63`:
  `use crate::entry::run_nested_process;`,
  `fn run_nested(arguments: &[OsString], environment: &[(OsString, OsString)]) -> i32` at line 308,
  counting `NESTED_FINISHES` around `run_nested_process(arguments, environment)` (lines 312-314),
  called at lines 340,
  392 and 418.
- `NESTED_FINISHES` and `FINISH_BUDGET`
  (`merge/merge_second_pass_finish_nesting.rs`,
  with `finish_budget` and `nesting_allows`),
  and `plan_finish` refusing at `merge_second_pass_finish.rs:215-221`
  when the budget is spent,
  which leaves the operation stopped and Git's own status standing.
- `invocation/entry.rs:614`:
  `pub fn run_nested_process(arguments: &[OsString], environment: &[(OsString, OsString)]) -> i32`.
- What runs on the status afterwards,
  which the earlier draft's loop could not do:
  inside `finish`,
  the commit step's status decides whether
  `cherry-pick --continue` runs at line 418
  (a clean pick whose `sequencer/todo` still has lines gets a second command);
  after `finish` answers,
  `merge_second_pass_worktree::resolve_after`
  reads the index again when the status is not zero and prints what it holds
  (`merge_second_pass_worktree.rs:116` and 267-285);
  and after that,
  `lock/index_writer_forward.rs:342-358` sweeps the worktrees a merge landed,
  with the final status.
- The path the status travels up:
  `finish` (line 356) is called by `merge_second_pass_worktree.rs:267`,
  whose `resolve_after(...) -> Option<i32>` (line 116) is called by
  `lock/index_writer_forward.rs:342` inside `run_coordinated_forward(...) -> CoordinatedRun` (line 202),
  which `entry.rs`'s `ForwardCoordinated` arm runs.

What moved since 2026-11-27:
nothing in the call.
The handover leaves out the merge half,
which is where the budget lives and where the status has four functions to travel through.

### What the plan says it becomes

"Run this next (replaces the two in-process recursions ...).
New file `what_a_command_hands_back` in the program folder `starting_git_and_describing_a_run`
defines `NextCommand { arguments, environment }` and `Finished { code, stdout, stderr }`.
`gated_merge_push` returns `Result<NextCommand, SingleFlightError>` ...
The stacking loop in `push_stacking` becomes a step function:
`StackingStep::{Done(i32), RunThis { command: NextCommand, resume: StackingState }}` ...
`merge_second_pass_finish::run_nested` becomes `FinishStep::{Done(i32), RunThis(NextCommand)}`;
the nesting counter `NESTED_FINISHES` moves to the loop.
Consumer:
the top-level loop in the program crate (`entry_run_forwarded_actions`) runs the command with
`entry::run_process`'s planning,
feeds the status back to the step function,
and repeats until `Done`.
A depth guard replaces the counter"
(lines 6434-6454).

The owner settled that both recursions are this change:
"The two in-process recursions (`stacked_merge_push` planning a new invocation,
`merge_second_pass_finish` running a nested command):
"A:
hand back 'run this next' (Recommended)":
the part returns the next command and the top level runs it in its own loop"
(`cli-git-source-layout.md`,
lines 111-115).
So the earlier draft's fifth question,
whether the merge half is in this round,
is answered by the record and is told here,
not asked.

Which folder stops depending on which:
no line of the allow list today,
measured by the survivors:
`push` keeps `invocation` through `push_stacking.rs:34` (`crate::action::PushForward`)
and `:60` (`crate::child_environment::environment_with_overlay`),
`bounded_push.rs:215` (`crate::forwarding::ChildOutcome`),
`manual_push_lifecycle.rs:408` (`crate::typed_network_command_relay`).
The merge half is the one that kills a line today:
`uses src/native/merge -> src/native/invocation` (`folder-rule.txt:820`)
rests on one file,
`merge_second_pass_finish.rs`'s `use crate::entry::run_nested_process`
(`rg -ln "crate::entry|crate::plan_invocation|crate::run_nested_process|crate::action::" src/native/merge/*.rs` answers that file alone).
The push half kills none by itself:
`stacked_merge_push.rs` becomes
`push_of_a_merge_that_integrated_a_lost_race.rs` in `working_out_what_a_typed_push_would_send`
(target-tree-files,
line 3755),
whose `uses/` line (3757) names both
`starting_git_and_describing_a_run` and `what_happens_when_you_type_a_command`,
and would lose the second,
not the first;
and the folder keeps the pair until `PushForward` moves into `push_stacking`,
which the plan's items ask for and `main` has not done (`push_stacking.rs:34` still imports it from `crate::action`).

The earlier draft said the push folder's line at 3725 names `starting_git_and_describing_a_run` and would lose it.
3725 is the `uses/` line of `who_owns_a_transaction_folder`;
the push folder's is 3757.
The merge folder `rerunning_a_merge_word_by_word` does name `what_happens_when_you_type_a_command`
(its `uses/` line,
2263),
so its gain is what the earlier draft said,
for the folder it said it about.

### Signature A: the plan's step functions, with the state they need

The earlier draft drew four names that exist nowhere
(`StackingState`,
`Ran`,
`run_and_carry`,
`NESTING_LIMIT`) and a step that could not run what happens after a nested command.
Both are corrected here.

```rust
//! package/git-policy/cli/src/native/invocation/what_a_command_hands_back.rs
/// One command the wrapper plans and runs as its own child.
pub(crate) struct NextCommand {
    /// The words after the program name.
    pub arguments: Vec<OsString>,
    /// The environment the command runs with.
    pub environment: Vec<(OsString, OsString)>,
}

/// What the stacking loop carries between the commands it hands up.
///  It is today's `StackRequest` plus the status its decision read.
pub(crate) struct StackingState<'a> {
    /// The push's facts.
    pub git: &'a PushGit,
    /// The global prefix of the arguments.
    pub global_prefix: &'a [OsString],
    /// The words after the subcommand.
    pub region: &'a [OsString],
    /// The way the wrapper integrates a merge.
    pub merge_push: MergePush<'a>,
}

/// How a stacking step ends.
pub(crate) enum StackingStep<'a> {
    /// The stacking is over.
    Done {
        /// The status the process ends with.
        code: i32,
    },
    /// Run this command,
    ///  then hand its status back to the step function with `resume`.
    RunThis {
        /// The command to run.
        command: NextCommand,
        /// The state the status is handed back with.
        resume: StackingState<'a>,
    },
}

/// What a finished step does with the status of the command it handed up.
enum FinishResume {
    /// The rebase's `--continue`,
    ///  whose status is the finish's own.
    EndingTheRebase {
        /// The plan the second pass made.
        plan: Finish,
    },
    /// The commit that ends a stopped merge or pick.
    Committing {
        /// The plan the second pass made.
        plan: Finish,
        /// The environment the nested command runs in.
        environment: Vec<(OsString, OsString)>,
    },
    /// The rest of a pick or revert sequence,
    ///  when the commit landed and the todo list has lines.
    ContinuingTheSequence {
        /// The plan the second pass made.
        plan: Finish,
        /// The environment the nested command runs in.
        environment: Vec<(OsString, OsString)>,
    },
}

/// How a nested finish ends.
pub(crate) enum FinishStep {
    /// The finish is over.
    Done {
        /// The status the process ends with.
        code: i32,
    },
    /// Run this command,
    ///  then hand its status back to `resume`.
    RunThis {
        /// The command to run.
        command: NextCommand,
        /// What the finish does with the status.
        resume: FinishResume,
    },
}

/// What the second pass does with the status of the command it forwarded.
pub(crate) struct AfterTheCommand {
    /// The watch that may have made the second pass.
    pub watched: Option<Watch>,
    /// The forwarded command it is about.
    pub forward: CoordinatedForward,
}
```

and the top level holds a stack of them,
because a finish's second command can itself stop and hand one up:

```rust
//! package/git-policy/cli/src/native/invocation/entry.rs,
//!  in the arm that runs a coordinated forward
/// One step waiting for the status of a command it handed up.
enum Pending {
    /// The stacking loop's step function.
    Stacking {
        /// The state the stacking loop carries between commands.
        state: StackingState<'static>,
    },
    /// A nested finish's step function.
    Finish {
        /// What the finish does with the status.
        resume: FinishResume,
    },
    /// The second pass's work after the command it forwarded.
    SecondPass {
        /// What the second pass needs to go on.
        after: AfterTheCommand,
    },
}

let mut pending: Vec<Pending> = vec![Pending::SecondPass {
    after: AfterTheCommand {
        watched: second_pass_watch.take(),
        forward,
    },
}];
let mut wanted: Option<NextCommand> = None;
// The budget the outermost finish set from its input, kept as today.
let mut spent: usize = 0;
loop {
    let status: i32 = match wanted.take() {
        Some(command) => run_one_planned(&command, &inputs),
        None => run_the_first_step(&mut pending, &mut wanted),
    };
    // Hand the status to the innermost step that waits for one.
    let Some(step) = pending.pop() else {
        return status;
    };
    match resume(step, status, &mut wanted, &mut spent) {
        Step::Done(code) => return code,
        Step::Waiting => {}
    }
}
```

where `resume` is the step function and keeps the budget:

```rust
//! package/git-policy/cli/src/native/merge/merge_second_pass_finish.rs
/// What a finish does with the status of a command it handed up,
///  under the budget the outermost finish set from its input.
pub(crate) fn resume(
    resume: FinishResume,
    code: i32,
) -> FinishStep {
    match resume {
        FinishResume::EndingTheRebase { .. } => FinishStep::Done { code },
        FinishResume::Committing { plan, environment } => {
            if code != 0 || plan.operation != Operation::Pick || !plan.git_dir.join("sequencer/todo").exists() {
                return FinishStep::Done { code };
            }
            // Today's line 418: the rest of the sequence ends it.
            return FinishStep::RunThis {
                command: invocation_command(&plan, "cherry-pick", "--continue", environment.as_slice()),
                resume: FinishResume::ContinuingTheSequence { plan, environment },
            };
        }
        FinishResume::ContinuingTheSequence { .. } => FinishStep::Done { code },
    }
}
```

and the budget is not a constant:
the loop counts the commands one operation handed up,
and when `finish_budget(operation, git_dir)` is spent,
the finish answers `Done` with the status it has,
which is what `plan_finish` does today when `nesting_allows` refuses
(`merge_second_pass_finish.rs:215-221`).
A rebase that stops at its ninth conflict still ends with its own status,
not with `ENGINE_FAILURE_EXIT_CODE`,
which is what the earlier draft's `NESTING_LIMIT: u32 = 8` would have changed.

One real call site rewritten,
the tail of `finish` (today lines 412-422):

```rust
//! package/git-policy/cli/src/native/merge/merge_second_pass_finish.rs
// Today: `let committed: i32 = commit_step(plan, command, environment);` then the
// `sequencer/todo` test and `run_nested(invocation(command, &[word, "--continue"]), environment)`.
return FinishStep::RunThis {
    command: NextCommand {
        arguments: commit_arguments(plan, command),
        environment: environment.to_vec(),
    },
    resume: FinishResume::Committing {
        plan: plan.clone(),
        environment: environment.to_vec(),
    },
};
```

What a nested command still gets of `run_process`'s pre-steps,
which the earlier draft left open
(`invocation/entry.rs:437-465`):

- `hooks_off::activate()` once,
  before the loop,
  because it is one fact of the process.
- `abandoned_locks::meet_the_locks_of_the_working_directory(environment, start)` once per nested command,
  with that command's own environment and directory,
  because a nested command can wait on a lock.
- `runtime_probe::is_requested` once at the top,
  since it ends the process.
- `process_resolution_inputs` once,
  and `refusal_when_started_as_git` once,
  because both are about this executable,
  not the command it runs.
- `recheck_before_the_command` once per forwarded command,
  which is where it runs today.

What it delivers:
at step 7,
`what_happens_when_you_type_a_command` out of
`working_out_what_a_typed_push_would_send`'s (3757) and the merge folder's (2263) `uses/` lists,
and the rule the plan wants restored:
the top level is called only from the top.
No allow-list line today for the push half;
one,
`uses src/native/merge -> src/native/invocation` (820),
for the merge half alone.

What it costs a caller:
one `NextCommand` per nested command,
two new enums and three resume variants,
a new variant on the coordinated run's outcome,
a stack and a budget at the top,
and the merge path's return type changed through four functions
(`finish`,
`resolve_after`,
`run_coordinated_forward`,
`run_coordinated_action`).
`Finish` and `Watch` must be nameable by the step functions,
which today are private to their files.

Which tests change:
`push_stacking_tests.rs:486`,
`entry`'s tests that read a planned `Action`,
`merge_second_pass_finish_tests.rs`,
`merge_second_pass_worktree`'s tests,
`index_writer_forward`'s tests,
and `auto_push_reconcile_tests.rs:3533`,
which describes this call in prose.

What it leaves undone:
`PushForward`'s move down,
without which the push folder keeps its use of `invocation`,
and the await of a nested command's status inside `resolve_after`,
which today is a synchronous call into a function that returns an `i32`.

### Signature B: the loop keeps its place and calls a runner

```rust
//! package/git-policy/cli/src/native/push/push_stacking.rs
/// Runs one command the wrapper planned,
///  and answers its status.
pub trait RunsOneCommand {
    /// Run the command and answer its status.
    fn run(&mut self, command: NextCommand) -> i32;
}

/// The stacking loop,
///  with the way to run a command handed in.
pub fn run_stacked_push(
    forward: &PushForward,
    merge_push: MergePush<'_>,
    runs: &mut dyn RunsOneCommand,
) -> i32;
```

and one real call site rewritten,
`invocation/entry.rs:580`:

```rust
//! package/git-policy/cli/src/native/invocation/entry.rs
/// The top level's way to run one command it planned.
struct TheTopLevel<'a> {
    /// The facts the planner needs about this process.
    inputs: &'a ResolutionInputs,
}

/// How the top level runs a command.
impl RunsOneCommand for TheTopLevel<'_> {
    /// Plan the command and run it.
    fn run(&mut self, command: NextCommand) -> i32 {
        let action: Action = plan_invocation(
            command.arguments.as_slice(),
            command.environment.as_slice(),
            self.inputs,
        );
        return run_the_action(action);
    }
}

return run_stacked_push(&forward, &gated_merge_push, &mut TheTopLevel { inputs: &inputs });
```

What it delivers:
the push folder's name at step 7,
with a far smaller diff,
because the stacking loop stays where it is.
It delivers nothing today.

What it costs a caller:
a trait object and behaviour handed across the crate boundary,
which the plan's note resists;
and the run still happens inside the stacking call,
so the top level is entered from below exactly as it is today,
only one frame further out.
The merge half gets no shape from this at all:
`finish` needs an `i32` synchronously,
and only a re-entered loop can give it one,
which is the recursion the change exists to remove.

Which tests change:
as A,
minus the merge path.

What it leaves undone:
the merge recursion,
the merge folder's line (820),
and the rule the change serves.

### Whether it is feasible

Neither signature was compiled.
A compiler check of the push half alone would compile
and would prove nothing about the merge half,
which is the wide part;
a check of the merge half is the round itself,
and the resume values above are the design the round would compile first.
The one thing a reading settles is that the status has somewhere to go now:
every function on the path already returns a value that carries an outcome,
and the two places that run code on a status after a nested command
(`finish`'s `sequencer/todo` test and `run_coordinated_forward`'s sweep)
are the two the resume values above are shaped for.
What no reading settles is how much of the merge path's own test suite
assumes a status is returned rather than a command.

### Ranking

A > B.

The measure:
what each delivers to the split,
which is whether the top level is called only from the top,
against what it costs in the merge path.

- A over B:
  A is the only one of the two that removes both recursions and so delivers the merge folder's line today;
  B leaves the merge recursion,
  the merge folder's use of `invocation`,
  and the top level still called from inside the stacking loop,
  so a program that runs a command from below stays possible and the folder tangles can regrow.
  B's advantage is real:
  a fraction of A's diff,
  and its push half could land first.

## Across the five: the order they can land in

Which files two of them both touch,
from the call sites above and the first choices
A (finished),
A (settings),
B (after-stop),
B (step record) and A (run this next):

- Commands that only finish and run this next share three files,
or four if the exit code moves in the same commit:
`push/stacked_merge_push.rs` either way,
and `invocation/entry.rs`,
`invocation/entry_run_forwarded_actions.rs` and
`lock/index_writer_forward.rs` through the `ENGINE_FAILURE_EXIT_CODE` move,
whose 18 files include all three.
They share no new file:
`what_a_command_hands_back` is shared only under the plan's placement of the finished value,
and run this next's drawn code does not use `Finished`.
The earlier draft's claim that run this next "needs the finished value whether or not the finished-results change runs"
is wrong under either placement of section 1.
- After-stop action and current step record share four files:
`push/child_watch_one_shot_run.rs`,
`push/child_watch_output_watched_on_the_runtime.rs`,
`push/child_watch_clock_marks_and_stopping.rs`,
where the after-stop change removes the ledger and the step-record change adds the landing answer to `WatchTerms`,
and `transaction/transaction_git.rs`.
- System settings path touches no file any other change touches.
Its ten call sites are in `index/`,
  `repo_read/` (four files),
  `push/`,
  `merge/` and `worktree/`,
and none of those files is on another change's list.
The earlier draft's claim that the settings change touches no file any other change touches
held for its recommended signature C,
  which edited `entry.rs`;
under A it holds,
  and the reason is the one stated here.

So round 4 is at most three units at once:
`{2}`,
`{3 then 4}` and
`{1 then 5}`,
and the two pairs are ordered inside themselves.
The plan's own order
(target tree,
lines 8185-8191 and 6956-6963)
lists them finished results,
system settings path,
after-stop action,
current step record,
run this next,
which puts the system settings change second and is compatible with all three units.
Nothing in the plan requires that order;
keeping it keeps a round's commits reading like the plan.

Two facts of the pair `{1, 5}` matter to the owner's decision:

- The exit code's move is what puts three of run this next's files on the finished change's list.
If it moves in its own commit,
  first,
  the two changes share `stacked_merge_push.rs` alone.
- Inside `{1, 5}` the order does not matter to the compiler:
1 removes `plan_invocation` from `stacked_merge_push.rs` and 5 removes the `Action` that file builds,
and either can land first without undoing the other,
as long as the same branch lands both before a gate.

## Questions for the owner

Each question is one decision,
and each is answerable alone.
The recommendation is first,
with its mechanism and its cost,
and each option says what a yes would still leave undecided.

### Question 1: where the finished value lives

Today the three commands build the top-level `Action` themselves.
The owner's decision of 2026-11-04 was to change that:
"they return a small \"finished with this code and output\" value from a low file;
only the top level builds `Action`"
(`cli-git-source-layout.md`,
lines 470-471).
The target tree's signature puts the file elsewhere:
`what_a_command_hands_back` in the program folder `starting_git_and_describing_a_run`
(line 6436).
That is the departure this question puts to you.

Option A,
recommended:
the value is `Finished` in `foundation`,
built by `saying`,
`explaining` and `quietly`,
and `management` turns it into `Action::ExitBytes`.
Mechanism:
one struct of three fields,
bytes for both streams;
the command builds no `Action`;
`subvolume` names `foundation` instead of `invocation`.
What it changes for good:
two allow-list lines die in this round
(`uses src/native/subvolume -> src/native/invocation`,
line 943,
and `names-group src/native/subvolume src/native/invocation`,
line 538),
and step 7 moves the file again into one of the three libraries the rescan makes of `foundation`.
What it gives up:
the value sits below the command folders,
so `stacked_merge_push` cannot return it without naming `foundation` too,
and the type is not beside `Action`,
which is the folder a reader of `Action` looks in.
A yes leaves undecided:
whether `ENGINE_FAILURE_EXIT_CODE` and `REFUSED_EXIT_CODE` move in the same commit,
which is one commit against eighteen files or two commits;
and whether `doctor_action`,
which has no release caller and is a published name,
is rewritten or deleted.

Option B:
the plan's placement:
`Finished` and `FinishedBytes` in `invocation`,
beside `action.rs`.
Mechanism:
two types,
or one bytes type every text caller converts;
the command builds no `Action`;
nothing in the allow list changes now.
What it changes for good:
step 7 takes `what_happens_when_you_type_a_command` out of `converting_the_main_checkout`,
and out of `checking_that_your_setup_works` and `converting_a_linked_worktree` too,
which then gain a use of `starting_git_and_describing_a_run`
(lines 733 and 830 do not name it today).
What it gives up:
the loop `subvolume` and `invocation` form stays in this round and in every branch that lands this alone.
A yes leaves undecided:
two types or one bytes type;
and the same two questions about the exit codes and `doctor_action`.

A third shape was considered and is not offered:
no value at all,
each command writing its own answer to two writers and returning a status
(section 1,
signature C).
It removes the same two lines as A but loses the one place that writes a report,
and it cannot be built without handing every command two writers,
at which point it has the value's shape and none of its uses.

### Question 2: who sweeps the locks a stopped step left

Today the watcher's own task sweeps them on the blocking pool,
and also when a watch is dropped without being read.
The plan's prose and your answer of 2026-11-04 say
"its caller hands it what to do after a stop" (line 458);
the target tree's signature says the watcher loses the ledger and the caller sweeps (lines 6467-6477).
That is the departure this question puts to you.

Option A,
recommended:
the watcher calls back.
The library defines `trait AfterTheStop`,
the program implements it once per call site with its `LockLedger`,
and the watcher runs it after the child and the processes below it are gone,
inside its task,
on the blocking pool,
and when the watch is dropped.
What it changes for good:
the watcher library folder stops naming `clearing_leftover_reference_locks` (target-tree-files,
line 322);
the library exports one trait whose only implementation is the program's.
What it gives up:
nothing a person sees,
and a behaviour crosses the crate boundary,
which the plan's note disfavours ("each hands a value or an enum",
lines 6378-6382).
A yes leaves undecided:
whether `LockLedger` is `Sync` as the trait needs,
which one compile settles;
and whether the same callback carries anything else later,
which the trait's shape allows and nothing asks for.

Option B:
the caller sweeps.
Each of the five one-shot functions answers `WatchedOutput { output, ended_by_the_watch }`,
the caller builds the ledger before the start and sweeps after the end,
and every caller that may sweep reaches the blocking pool itself.
What it changes for good:
the same library folder line dies;
the sweep's moment moves from the watcher's ending to the caller's reading of it.
What it gives up:
a watch that is dropped without being read sweeps nothing,
where today it sweeps,
and a lost sweep is not retried
(`settle_locks`:
"a sweep before that finds the tree running and removes nothing,
and nothing would try again"),
so the lock waits for the next command's stale-lock judgement;
and nine call sites gain a sweep.
A yes leaves undecided:
nothing about the trait,
which would not exist;
and whether the dropped-watch loss is acceptable,
which is the price named above.

### Question 3: how the current step record is carried

Today it is carried by the task
(`tokio::task_local!`,
`landing/landing_state_carried_by_its_task.rs:44-48`),
which is what your choice 4 asked for ("Rewrite per-thread state").
The target tree says a thread-local record in a program file (lines 6455-6466),
and the plan's prose says the basic tools (line 455).
The first choice below departs from the target tree's word;
that is the departure this question puts to you.

Option A,
recommended:
the record is a value on `GitRequest`,
named `CurrentStepRecord`:
a folder it writes into and the detail of the next write,
defaulting to "no landing".
The bound reads `request.step_record.on_a_landing()`,
and `WatchTerms::for_class` takes that answer at every call site.
What it changes for good:
the record becomes a name the folder rule can see,
so a program folder cannot reach it without naming it;
the watcher's answer arrives with the record.
What it gives up:
a field on the type 503 lines of 143 files name,
set only where a landing builds a request;
and the witnessed reads built without a request must be handed the answer,
`repo_read/git_metadata.rs:443` among them.
A yes leaves undecided:
whether `WatchTerms::for_class`'s new argument is a `bool` or the record itself;
and where `git_metadata`'s callers get the answer from.

Option B:
the record stays task-carried,
moved to a program file as the target tree says,
with the watcher's answer as a second field on `WatchTerms`.
Mechanism:
a `tokio::task_local!` holding the folder and the detail,
written by the runner,
read by the watcher through the flag it was handed.
What it changes for good:
the same library line dies;
the record remains state no line records.
What it gives up:
the folder rule cannot see the use,
so the tangle can regrow through a name;
and the same `WatchTerms` change and the same `git_metadata` question as A.
A yes leaves undecided:
whether the state stays a `task_local!` or becomes a value later;
and the same `git_metadata` question.

Option C,
the target tree's word read literally,
a new `thread_local!`,
is not offered:
it would put back a per-thread fact your choice 4 moved into the task,
and it would misread the day the landing runs as a task,
which `in_a_landing_task` exists for.

### Question 4: how far the system settings path is threaded

Option A,
recommended:
the ten builder call sites pass the path,
and the three reader files that build their own scope go on asking.
Mechanism:
one extra parameter on `ScopeEnvironment::from_variables`
and one line at each builder:
`crate::git_system_config::system_config_for(variables.as_slice())`.
What it changes for good:
the reader's file stops starting Git for the callers that hand a value over;
the reader's folder goes on naming the asker's folder at step 7.
What it gives up:
the folder line (target-tree-files, 1936) stays,
so the promise in `answering_questions_without_starting_git`'s name
holds only where a value was handed over.
A yes leaves undecided:
nothing about the signature;
what the three reader files do is the next question.

Option B:
also thread the value into `repo_read_query.rs:90`,
`repo_read_hook.rs:105` and `repo_read_layout.rs:264`,
so their callers learn the path and the reader's folder can stop naming the asker at step 7.
Mechanism:
the same parameter,
plus a parameter through those three files' callers;
`repo_read_query` builds its scope from base variables,
so its caller must supply the path.
What it changes for good:
the reader's folder can lose the asker at step 7,
which is the only delivery this change has.
What it gives up:
a parameter through the readers' callers,
some of which hand a path they must learn earlier,
and a larger diff in this round.
A yes leaves undecided:
whether any of those callers still has a real Git to ask when it builds the scope.

### Question 5: whether the attributes path changes in this round

`system_attributes_for` is the same shape as the settings path
and the plan does not name it
(`worktree/worktree_clean_content_attribute_sources.rs:18,194`,
inside `ConversionEnvironment::from_variables`,
whose three test calls are `worktree_clean_content_safety_tests.rs:714,719,724`).

Option A,
recommended:
leave it asking in this round.
What it changes:
nothing.
What it gives up:
"the reader never starts Git" is not delivered for conversions,
because that builder still asks for the attributes file.
A yes leaves undecided:
which step takes it.

Option B:
change it in the same round,
with the same parameter.
What it changes:
the conversion builder stops starting Git for both of its questions.
What it gives up:
a second parameter and a wider diff in the same round,
and the plan has no signature for it,
so this proposal would be its only statement.
A yes leaves undecided:
whether the parameter is one `SystemAttributes` value or an `Option<PathBuf>` and a kind.

### Question 6: one branch or two

The three units above are `{2}`,
`{3 then 4}` and `{1 then 5}`.

Option A,
recommended:
two branches side by side,
`{2, 3, 4}` and `{1, 5}`.
Mechanism:
no file is touched by both branches;
the pair inside each branch is ordered.
What it changes for good:
the round's red lint findings split across two branches,
and the handover's list of them is written once per branch.
What it gives up:
two merges and two gate runs.
A yes leaves undecided:
which branch merges first,
which nothing in the records decides.

Option B:
one branch with five commits in the plan's order.
Mechanism:
one merge,
one gate run,
commits in the order the plan lists.
What it changes for good:
nothing,
and the round reads like the plan's step.
What it gives up:
the wall-clock of one round doing all five,
and a single red gate listing every finding at once.
A yes leaves undecided:
the same question of which lands first,
and nothing else.

Whether either is startable now depends on work in flight,
which the handover entry names:
the push split cuts `child_watch_one_shot_run.rs`,
issue 6573 changes the control that reads `repo_read/git_system_config.rs`,
and the answer to issue 6529 edits `merge_second_pass_finish.rs`.
This proposal was written without waiting for them.

## What was measured, and what was not

Measured on 2026-11-28 in this worktree at `5a84e361c0`,
with the command named:

- `ENGINE_FAILURE_EXIT_CODE`:
  52 release lines in 18 files in 9 folders
  (`rg -n ... | wc -l`,
  `rg -l ... | wc -l`,
  `rg -l ... | xargs -n1 dirname | sort -u | wc -l`),
  and its published line at `items_nameable_from_outside_release.txt:3142`.
- The builders of a `ScopeEnvironment`:
  10 release call sites
  (`rg -n "ScopeEnvironment::from_variables" -g '*.rs' -g '!*_tests.rs'`),
  and the eleventh the earlier draft counted is a `ConversionEnvironment`
  (`rg -n "ConversionEnvironment::from_variables"`).
- The one-shot run functions:
  5,
  and the one caller of `ChildWatch::start` that passes no ledger:
  1 (`rg -n "ChildWatch::start\(" -g '*.rs' -g '!*_tests.rs'`).
- `WatchTerms {` literals:
  24 in 7 test files
  (`rg -c "WatchTerms \{" -g '*_tests.rs'`),
  and `WatchTerms::for_class` callers: 6.
- `GitRequest`:
  503 lines of 143 files,
  122 of them release files.
- The allow-list pairs each change adds or removes,
  by `rg -n "^uses src/native/<a> -> src/native/<b>" layout-allow-list/folder-rule.txt`:
  `subvolume -> invocation` at 943 and its `names-group` at 538;
  `subvolume -> foundation` at 942;
  `merge -> invocation` at 820;
  `lock -> transaction` at 805,
  with no `transaction -> lock` line at all;
  and the pairs `push -> service`,
  `transaction -> service`,
  `merge -> invocation`,
  `push -> invocation`,
  `transaction -> repo_read`,
  `service -> foundation`,
  `push -> foundation`
  each present once or twice.
- The target-tree rows quoted:
  279,
  322,
  731-733,
  828-830,
  853-855,
  1936,
  2263,
  2602,
  3757,
  and the plan's order at 8185-8191.
- The two source lines and the control that the gate read named:
  `migrate_action`'s refusal on standard error (`subvolume_command.rs:366-376`),
  the doctor's standard-error-only exit (`doctor.rs:662-666`),
  and `the_scope_reads_the_file_the_git_names` in a child of its own
  (`git_system_config_tests.rs:256-301`).

The dead lines section 1 states as delivered
are the earlier draft's measurement on a branch this revision did not rebuild,
and each is marked so where it is used.

Not run,
and why:

- No build,
  no compiler check,
  no test suite,
  no nextest run,
  no container and no guest.
  This branch changes no code;
  the two compiler answers this proposal quotes are the earlier draft's,
  and one of them is of a removal,
  not of a signature.
- No linter on a trial branch,
  so the dead lines and the added `transaction -> lock` use
  are stated from the source and the earlier draft,
  not re-measured.
  The linter's answer for the added use is the round's first measurement.
- No trial branch for the file sets of the order section.
  They are read from the call sites above,
  and a diff of a trial branch is what would confirm them.

## What this proposal does not settle

- The plan's two further call changes found by the moved-item check
  (`copy_file_exclusive`'s error type and the hint functions of `event_hints`)
  belong to step 5,
  not round 4,
  and are not described here.
- `doctor_action`'s future:
it has no release caller and is published
  (`items_nameable_from_outside_release.txt:2451`),
  so the round either rewrites it with the others or deletes it and its line,
  and deleting a published name is a change to the library's surface
  that needs the owner's word when the next release is cut.
- Which red `rust/folder-uses-links` findings `main` prints today
  (`doc/handover/cli-git-rust-implementation.md` keeps that list) is not read here;
  this proposal's line numbers are of the allow list at `5a84e361c0`,
  and a line added or deleted before the round starts moves them.
- The owner's other open points for the split
  (target tree,
  "Open points for the owner")
  hold steps 7 and after,
  not this round,
  except that reading D or E of the library count decides whether
  `starting-and-watching-programs` is a library at all.
  Both readings keep it,
  so the after-stop and step-record changes are needed under either.

## What the gate read changed

Five High,
eleven Medium and five Low findings;
each is answered here or in the round's report.
The changes that matter to a reader of the earlier draft:

- The run-this-next signature now carries a resume value per step function
  and a stack at the top (H1),
  and keeps the todo list's budget instead of a constant of eight (H2).
- The system settings ranking is reversed:
  the plan's parameter is first and the process-wide value is last,
  with its evidence stated (H3).
- The after-stop ranking is reversed:
  the claim that `git_metadata.rs:443` passes no ledger was wrong,
  and the watcher's callback keeps the blocking pool and the dropped-watch sweep (H4).
- The step record is described as it is on `main`,
  task-carried,
  the carrier is `GitRequest` rather than a type that does not exist,
  and the name is `CurrentStepRecord` because `StepRecord` is taken (H5,
  L3).
- Every count and line number the earlier draft stated is re-measured at `5a84e361c0` (L1,
  L2,
  M2),
  the step-7 delivery claims are corrected (M11),
  the order's file sets are recomputed (M3),
  the published surface of every drawn item is `pub(crate)` (M7),
  and the questions are six,
  one decision each (M4).
