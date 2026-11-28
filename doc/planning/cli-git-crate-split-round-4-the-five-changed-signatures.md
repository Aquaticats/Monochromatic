# The crate split's fourth round: the five changed signatures, proposed for the owner

Status:
unapproved and non-final.
Only the owner declares this record complete.
Prepared by a Pi run on 2026-11-28 and revised in place the same day,
against this worktree's `main` at `b6fc11da64`
(`git log --format=%cI -1`:
2026-11-28T08:18:51-05:00).
No code changed on this branch:
every drawing in it was put into a working tree,
compiled,
and taken out again,
and the proposal alone is committed here.

Gate reads of this proposal:
two,
both same-vendor second passes
(Claude Sonnet,
the vendor of part of the work),
neither of them an independent review (`AGENTS.md` rule `IRW`).

- The first read found five High,
  eleven Medium and five Low findings.
- The second read,
  of 2026-11-28,
  found four High,
  eleven Medium and eight Low findings,
  and its verdict was "not ready to put to the owner as written".
  Its four Highs were:
  both after-stop options add a folder use no allow-list line can record;
  the after-stop ranking rests on a sweep that happens in a narrower window than the file said,
  and both drawn call sites failed against the code;
  both step-record drawings dropped a guard the code has today;
  and the file named no combination of answers that cannot be built.
  Every one of those findings changed this record,
  and every finding of the read is answered in the round's report,
  with the line or the run that settles it.

No review from a vendor other than the authors' has read any revision of this file.

## The first screen

### What you are asked

Five questions,
each one decision,
each answerable alone.
The recommendation is first in each pair of options;
the reason given here is the delivered one,
and each question later states its mechanism,
its cost and what a yes leaves undecided.

1.  Where the value of a command that only finishes lives.
    Recommended:
    `Finished` in `foundation`,
    the plan's own words ("from a low file").
    Reason:
    it removes two allow-list lines in this round and three folder names at step 7,
    where the target tree's placement removes none now.
    Measured:
    the two allow-list lines do not die unless the exit code moves too (`ENGINE_FAILURE_EXIT_CODE`,
    52 release lines in 18 files).
2.  Who sweeps the Git locks a stopped step left.
    Recommended:
    the watcher calls back into the caller's ledger
    (`AfterTheStop`),
    which is the plan's prose ("its caller hands it what to do after a stop").
    Reason:
    the sweep stays where it runs today,
    inside the watcher's task,
    including when a watch is dropped while a stop is under way;
    the other option loses the sweep in exactly that window.
    Measured:
    both options add the same two folder uses
    (`transaction -> lock` and `repo_read -> lock`),
    which no allow-list line can record.
3.  How the current step record is carried.
    Recommended:
    on the run's `GitContext`,
    with no `Default`,
    so the compiler refuses a build site that does not answer.
    Reason:
    it is the only carrier that both the folder rule can see and no builder can forget;
    a field on `GitRequest` cannot be made non-defaulting while `GitRequest` derives `Default`.
    Measured:
    `GitContext` has 63 initializers (22 in release code,
    41 in controls),
    27 release functions return a `GitRequest`,
    and the compiler refuses both a missing context field and a non-defaulting request field.
4.  How far the system settings path is threaded.
    Recommended:
    to every caller that builds a scope,
    the three reader files included.
    Reason:
    it is the reach the plan's prose states,
    and it is the one that removes two `uses/` entries at step 7 instead of one.
    Read:
    of the five files of `reading_git_settings_from_files`,
    one names the asker today.
5.  Whether the attributes path changes in this round.
    Recommended:
    it does not.
    Reason:
    the plan records no signature for it,
    so changing it would make this file its only statement,
    and it delivers nothing the round's own record promises.
    Read:
    `system_attributes_for` is asked once,
    at `package/git-policy/cli/src/native/worktree/worktree_clean_content_attribute_sources.rs:194`.

### What binds one answer to another

- Question 1's stated delivery needs the exit code moved in the same branch.
  Anything short of that is buildable and delivers half of what its option says.
  Reason:
  read.
- Question 2,
  either option,
  lands two folder uses that no line of the allow list can record,
  so the round that answers it merges with two red findings or untangles them first.
  Reason:
  the linter,
  on a drawing of each option.
- Question 3,
  option A,
  cannot be done one build site at a time:
  the compiler refuses every `GitContext` initializer that does not answer.
  Reason:
  the compiler.
- Questions 2 and 3 cannot land in two branches,
  because both edit `transaction/transaction_git.rs`.
  Reason:
  read.

Inferred,
and said so where it is used:
how often a step is in the stop-under-way window,
the shape of run this next once it holds together,
and that the exit code can leave `action.rs` without a loop.

### What is told, not asked

- The system settings value becomes a parameter,
  not a field on `ScopeEnvironment`.
  The struct already holds that fact as `system_default`
  (`repo_read/repo_read_config_scan.rs:214`),
  and the owner's own words are "the system settings path passed to the reader".
  Question 4 asks how far it is threaded,
  not whether.
- The merge half of run this next is in this round.
  The owner settled that both in-process recursions are one change
  ("A:
  hand back 'run this next' (Recommended)",
  `doc/planning/cli-git-source-layout.md:111-115`).
- The branch shape is the plan's:
  one branch,
  one commit per call change
  (`doc/planning/cli-git-source-layout-target-tree.md:8185-8191`).
  It changes nothing that lands,
  only what the round costs,
  so it is recorded and not put to you.

### What no drawing has shown

Every drawing in this file was put into this worktree and compiled with
`cargo check --locked --offline --all-targets` in `package/git-policy/cli`
(none of them committed),
and one linter run read each after-stop drawing.
The compiler and the linter said:

- The after-stop option B's own machinery (the caller sweeps,
  the five one-shot functions answering
  `WatchedOutput`) compiles with no error:
  `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 27.89s`.
  It needed 18 lines changed in three control files and 3 release call sites.
- The after-stop option A's machinery (the watcher calls back,
  the trait in the library) compiles with no error:
  `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 33.89s`.
- The step-record drawing on `GitRequest`,
  with today's guard kept,
  compiles:
  `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 27.44s`,
  with two warnings (an import and a record field the drawing does not read).
- Both after-stop drawings add four `rust/folder-uses-links` findings,
  two for each new use,
  where the same run on an unedited tree printed none for those pairs:
  12 findings before,
  16 after.
- Three combinations are refused by the compiler,
  and the texts are quoted where they are named.

What no compile shows:
any behaviour.
No test ran in this round,
no nextest,
no container and no guest.
The landing's own steps,
the sweep's timing,
and the dropped-watch window are read from the code and named as read.

## What this record is for

The owner's answer of 2026-11-28 on the crate split's step 4 was
"Read the signatures first (Recommended)"
(`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
"The crate split's fourth round"),
and what follows it is that
"the five signatures of round 4 are prepared as a proposal,
read by a gate reader,
and then put to the owner",
and that
"Round 4 does not run before that".

This is that proposal.
The question it answers is
"Open:
does the crate split's step 4 run now?"
(`doc/handover/questions-for-the-owner.md`),
which names the five as finished results,
system settings path,
after-stop action,
current step record and run this next.

For each of the five call changes it states where the call is today,
what the plan says it becomes,
two or three shapes that would do it,
what each delivers for the split and what each costs,
what was run to find out whether it works,
and a ranking with a stated measure.
It also states the combinations of answers that cannot be built,
and the order the five can land in.

## The words these pages use

Read this once;
each question later repeats the words it needs.

- The crate split is the owner's decision of 2026-11-04
  to break the wrapper into reusable crates
  (`doc/planning/cli-git-source-layout.md`,
  "Splitting into crates (owner,
  2026-11-04)").
  Its **step 7** is the move of the files into the target folders,
  after the call changes.
- The target tree is `doc/planning/cli-git-source-layout-target-tree.md` and
  `doc/planning/cli-git-source-layout-target-tree-files.md`,
  which lists the target folders file by file with each folder's `uses/` line.
- A folder's **`uses/` list** is the list of folders whose code it uses,
  written as links in a `uses/` folder inside it (`AGENTS.md` rule `LYD`).
  Two folders that use each other are a **loop**,
  and no loop is allowed (`AGENTS.md` rule `LYL`).
- The **allow list** is `package/git-policy/cli/layout-allow-list/folder-rule.txt`,
  whose lines tolerate the loops and uses `main` still has.
  It may only shrink
  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`),
  and a linter finding for a use that joins a loop the list already tolerates has no key,
  so no line of the list can record it
  (that decision's lines 115 to 119).
- `Action` is the value the top level runs
  (`package/git-policy/cli/src/native/invocation/action.rs`):
  a command's exit status and its two streams,
  or a forwarded or planned run.
- `Finished` is the value this proposal would put in a command that only finishes:
  an exit status and each stream's bytes.
- `GitRequest` is what one step of a transaction asks for
  (`package/git-policy/cli/src/native/transaction/transaction_git_request_and_reply_types.rs:91`):
  its arguments,
  its directory,
  its environment adjustments,
  its input and its child cell.
  It is not a class and not a set of watch terms;
  those come from the arguments and from `WatchTerms`.
- `GitContext` is the run a request belongs to
  (`package/git-policy/cli/src/native/transaction/transaction_git_request_and_reply_types.rs:57`):
  the real Git,
  the overlay,
  the global prefix and the common Git directory.
  It is built once per operation and passed down;
  it has no `Default`.
- `LockLedger` is the record of where a command's Git child can leave reference locks,
  taken before the child starts (`package/git-policy/cli/src/native/lock/stopped_git_locks.rs:96`).
  Its `sweep` judges each lock and removes only one it can prove stale.
- `ScopeEnvironment` is the configuration scopes a reader may read,
  settled from a list of environment variables
  (`package/git-policy/cli/src/native/repo_read/repo_read_config_scan.rs:274`).
- `WatchTerms` is what a watch runs under
  (`package/git-policy/cli/src/native/push/child_watch_clock_marks_and_stopping.rs:80`):
  the quiet window,
  the notice cadence and where notices go.
- `AfterTheStop` is the behaviour this proposal would hand the watcher:
  what to do once the watch has ended the child and the processes below it.
- the **landing** is the one operation that writes a commit in the service,
  the only place a step is never ended
  (`package/git-policy/cli/src/native/landing/landing_state_carried_by_its_task.rs`).
- `foundation` is today's `src/native/foundation/`,
  the folder of small helpers at the bottom,
  which step 7 splits among three libraries and six program folders.

## How every claim here was checked

Every line citation in this file was checked by a script against the commit this file names,
and every quotation was compared with its file by the same script (its method and its output
are in the round's report).
The compiler runs and the linter runs are named where their output is used.

Run on 2026-11-28 in this worktree,
each in its own
`systemd-run --user --scope --slice=agent-work.slice` with `MemoryMax`,
`CPUQuota`,
`TasksMax`,
`IOWeight` and `RuntimeMaxSec`:

- `cargo check --locked --offline --all-targets` in `package/git-policy/cli`,
  four times:
  once on the unedited tree (a baseline,
  `Finished \`dev\` profile ... in 1m 05s`),
  once with each after-stop drawing in place,
  and once with the step-record drawing in place.
  Three further runs were made only to read a refusal,
  and their errors are quoted below.
- `/home/user/Monochromatic/.cache/toolchain-bin/monochromatic-lint`,
  three times:
  on the unedited tree,
  and once with each after-stop drawing in place.
  The linter reported one rule it could not run
  (`rust/avoidable-closure`:
  the Dylint driver for this toolchain is absent and would have to be downloaded),
  so this round says nothing about closures.

Not run:
no test suite,
no nextest run,
no container,
no guest,
no `mise` task,
no editor,
nothing on ZFS,
no network call,
no release,
no workflow.

## Which record governs where they disagree

Where records disagree this proposal follows the owner's recorded answers first,
the plan's prose second,
the target tree's signatures third
and the handover entry last.
Each disagreement is stated where it bites.

1.  The handover makes run this next a single call change in `stacked_merge_push`.
    The plan's run this next replaces two in-process recursions,
    `entry::plan_invocation` called from `stacked_merge_push`
    and `entry::run_process` called from `merge_second_pass_finish`
    (`doc/planning/cli-git-source-layout-target-tree.md:6434-6454`).
    The plan governs:
    this proposal describes the whole call change.
2.  Commands that only finish:
    the plan's prose asks for
    "a small \"finished with this code and output\" value from a low file"
    (`doc/planning/cli-git-source-layout.md:470`),
    while the target tree's signature puts the new file `what_a_command_hands_back`
    in the program folder `starting_git_and_describing_a_run`
    (`doc/planning/cli-git-source-layout-target-tree.md:6436`),
    which is not a low file.
    The two placements deliver different things,
    so the departure is the owner's to make:
    question 1.
3.  After-stop action:
    the plan's prose says "its caller hands it what to do after a stop"
    (`doc/planning/cli-git-source-layout.md:458`),
    which is signature A of section 3,
    while the target tree's signature says
    "The watcher returns the `StopReport` it already has;
    the caller in `bounded_push` and `transaction_git` builds the
    `LockLedger` before the start and sweeps after the stop"
    (`doc/planning/cli-git-source-layout-target-tree.md:6473-6477`),
    which is signature B.
    `StopReport` is not a type the watcher has:
    it is the push's progress clock
    (`package/git-policy/cli/src/native/push/stop_cause_and_words.rs:61`),
    so the tree's sentence also names a type that would not travel.
    Question 2 puts the departure to the owner.
4.  Current step record:
    the plan's prose says
    "a small \"current step\" record in the basic tools that the runner writes and the service reads"
    (`doc/planning/cli-git-source-layout.md:455`),
    the target tree puts it in the program file `current_step_record`
    in `how_long_the_program_waits_for_git_steps`
    (`doc/planning/cli-git-source-layout-target-tree.md:6457-6459`),
    and the code carries it in a `tokio::task_local!`,
    not in a thread-local as the tree's sentence says
    (`package/git-policy/cli/src/native/landing/landing_state_carried_by_its_task.rs:44-48`,
    and the owner's choice 4 in
    `doc/planning/git-policy-cli-full-tokio.md:1655-1657`).
    Question 3 puts the departure to the owner.
5.  System settings path:
    the tree says `ScopeEnvironment` "gains the field `system_config: Option<PathBuf>`"
    (`doc/planning/cli-git-source-layout-target-tree.md:6479`),
    but the struct already holds that fact as `system_default`
    (`package/git-policy/cli/src/native/repo_read/repo_read_config_scan.rs:214`),
    filled at line 296,
    and its meaning today (unknown,
    which makes the scope unresolvable)
    is the one a parameter must keep.
    The change is a parameter,
    not a field,
    and that is the owner's own wording:
    "the system settings path passed to the reader"
    (`doc/planning/cli-git-source-layout.md:118`).
6.  The tree says the line that calls `LockLedger::before_start(command)` "in `child_watch`" moves out
    (`doc/planning/cli-git-source-layout-target-tree.md:6470`).
    On `main` there are two such lines,
    in two sibling files,
    `package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs:238` and
    `package/git-policy/cli/src/native/push/child_watch_output_watched_on_the_runtime.rs:243`,
    because round 2 of the split cut them out of `child_watch.rs` after the plan was written.
    The sentence also does not name the sweep inside
    `child_watch_clock_marks_and_stopping::stop_process`
    (`package/git-policy/cli/src/native/push/child_watch_clock_marks_and_stopping.rs:372-383`),
    which is the half the change has to move.
7.  The tree names `transaction_git` alone for the current step record
    (`doc/planning/cli-git-source-layout-target-tree.md:6456`),
    but `on_landing_thread` is also called by the two watched-run files named in item 6
    (lines 231 and 236),
    and the watcher library's `uses/` line names the folder those files would name
    (`doc/planning/cli-git-source-layout-target-tree-files.md:322`).
    A change that stops at `transaction_git` delivers nothing to that line.
8.  Run this next:
    the tree says "A depth guard replaces the counter"
    (`doc/planning/cli-git-source-layout-target-tree.md:6454`),
    while the code has a depth read from the todo list
    (`package/git-policy/cli/src/native/merge/merge_second_pass_finish_nesting.rs`,
    `FINISH_BUDGET` and `finish_budget`).
    A guard of eight is not a replacement of one kind for another;
    section 5 keeps the budget.
9.  The plan's order has the system settings change second,
    not third (`doc/planning/cli-git-source-layout-target-tree.md:8185-8191`).

## 1. Commands that only finish

### Where the call is today

- `package/git-policy/cli/src/native/subvolume/subvolume_command.rs:353`:
  `pub fn migrate_action(arguments: &MigrateArguments, cwd: &Path) -> Action`,
  which returns `Action::Exit` with `stdout: String::from(MIGRATE_HELP)` for `--help` (line 358),
  `Action::Exit` with the refusal in `stderr` (lines 366 to 376),
  and `Action::Exit { code, stdout: String::new(), stderr: String::new() }` for the run (line 383).
- `package/git-policy/cli/src/native/health/doctor.rs:607`:
  `pub async fn doctor_action_with_section(...) -> Action`,
  which returns `Action::Exit` with the explanation in `stderr` (lines 662 to 666)
  and `Action::ExitBytes` (lines 676 and 698);
  `doctor_failure(message: &str) -> Action` at line 718.
  Its synchronous face `doctor_action` (line 441) has no release caller:
  the only caller is `package/git-policy/cli/src/native/invocation/management.rs:527`,
  which calls the async one.
- `package/git-policy/cli/src/native/worktree/worktree_convert_command.rs:147`:
  `pub fn convert_action(...) -> Action`,
  returning `Action::Exit` for `--help` (line 154) and for the run (line 171).
- The consumers that build the `Action` instead:
  `package/git-policy/cli/src/native/invocation/management.rs:479`,
  `:385` and `:527`.
- The exit code the move drags with it:
  `ENGINE_FAILURE_EXIT_CODE` is named on 52 release lines of 18 files in 9 folders
  (`rg -n "ENGINE_FAILURE_EXIT_CODE" -g '*.rs' -g '!*_tests.rs'`,
  counted by `wc -l` and by `xargs -n1 dirname | sort -u`).
  It is defined at `package/git-policy/cli/src/native/invocation/action.rs:45`,
  used at `package/git-policy/cli/src/native/subvolume/subvolume_command.rs:240`
  and `package/git-policy/cli/src/native/health/doctor.rs:720`,
  and its published line is `invocation::action const ENGINE_FAILURE_EXIT_CODE`
  (`package/git-policy/cli/src/native/source_guard/items_nameable_from_outside_release.txt:3278`).
- The four names that are published and change shape:
  `subvolume::subvolume_command fn migrate_action` (`items_nameable_from_outside_release.txt:8376`),
  `worktree::worktree_convert_command fn convert_action` (line 9075),
  `health::doctor fn doctor_action_with_section` (line 2588)
  and `health::doctor fn doctor_action` (line 2587).
  Each is `pub fn` today,
  and each would become `pub(crate)`
  under the guide's reach rule (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md:833`)
  unless the round keeps a `pub` face for the library,
  which means editing that list either way.

### What the plan says it becomes

The target tree's signature:
"Commands that only finish:
`subvolume_command`,
`doctor`,
`worktree_convert_command` and `stacked_merge_push` return
`Finished` and only the program crate's `entry` and `management` files build `Action`"
(`doc/planning/cli-git-source-layout-target-tree.md:6482-6486`).

The plan's prose:
"they return a small \"finished with this code and output\" value from a low file;
only the top level builds `Action`"
(`doc/planning/cli-git-source-layout.md:470-471`).

Which folder stops depending on which,
in the target tree:
`converting_the_main_checkout` (target-tree-files:853,
its `uses/` at 855),
`converting_a_linked_worktree` (line 828,
`uses/` at 830) and
`checking_that_your_setup_works` (line 731,
`uses/` at 733)
name `what_happens_when_you_type_a_command` in their `uses/` lists today,
where `action.rs` goes (`the_decision_one_command_ends_in.rs`,
line 3700,
in the folder at 3649).
The change takes that name out of all three.

Which line of the allow list it removes,
and when:
`uses src/native/subvolume -> src/native/invocation`
(`package/git-policy/cli/layout-allow-list/folder-rule.txt:943`)
and `names-group src/native/subvolume src/native/invocation` (line 538).
Neither dies on the type change alone.
`subvolume_command.rs` names two things from `invocation`:
`Action` (line 31) and
`ENGINE_FAILURE_EXIT_CODE` (lines 30 to 33,
used at 240).
Taking `Action` out of the file leaves the exit code,
and the linter keeps a `uses` line alive for as long as any name of that folder is used
(measured in this round:
the linter's finding for a use that a line records is absent exactly when nothing names the folder,
which is what the two after-stop drawings showed from the other side).
So option A's two lines die only if `ENGINE_FAILURE_EXIT_CODE` leaves `action.rs` in the same branch.
`health` and `worktree` keep using `invocation` for other files,
so their lines stay.

### Signature A: `Finished` in `foundation`, one constructor per stream

New file `package/git-policy/cli/src/native/foundation/a_finished_command.rs`:

```rust
//! package/git-policy/cli/src/native/foundation/a_finished_command.rs
/// What a command that only finishes hands back:
/// the exit status and the bytes of each stream.
pub(crate) struct Finished {
    /// The status the process ends with.
    pub(crate) code: i32,
    /// What the command answers,
    /// as bytes because a report may name a path that is not text.
    pub(crate) standard_output: Vec<u8>,
    /// What it explains,
    /// as bytes for the same reason.
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

The three constructors are the correction to an earlier draft that had two,
and so could not build either of the two refusals the code writes on standard error.

`migrate_action`,
`convert_action` and `doctor_action_with_section` return it,
and the refusal keeps its stream:

```rust
//! package/git-policy/cli/src/native/subvolume/subvolume_command.rs
/// The migration's answer.
pub(crate) fn migrate_action(arguments: &MigrateArguments, cwd: &Path) -> Finished {
    if arguments.help {
        return Finished::saying(0, MIGRATE_HELP);
    }
    let root: PathBuf = match main_checkout_root(cwd) {
        Ok(found) => found,
        // Today this branch is `Action::Exit { code: REFUSED_EXIT_CODE, stdout: String::new(),
        // stderr: format!(...) }` at lines 366 to 376, so the words stay on standard error.
        Err(message) => {
            return Finished::explaining(
                REFUSED_EXIT_CODE,
                format!(
                    "git monochromatic-git-policy-cli migrate-to-subvolume: refused: {message}\n  What to do: run the command inside the repository whose main checkout you want to convert.\nNothing was changed.\n"
                )
                .as_str(),
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

and one real call site,
where `run_migrate` builds the `Action` today (lines 476 to 481):

```rust
//! package/git-policy/cli/src/native/invocation/management.rs
/// Run the migration command on the platform that has btrfs subvolumes.
fn run_migrate(arguments: &MigrateArguments, cwd: &Path) -> Action {
    let finished: Finished = migrate_action(arguments, cwd);
    return Action::ExitBytes {
        code: finished.code,
        stdout: finished.standard_output,
        stderr: finished.standard_error,
    };
}
```

What it delivers:
the two allow-list lines,
**if** the exit code moves with it (see "Which line of the allow list it removes"),
and no new folder use:
`subvolume` already uses `foundation`
(`uses src/native/subvolume -> src/native/foundation`,
`folder-rule.txt:942`).
At step 7 it takes `what_happens_when_you_type_a_command` out of those three folders' `uses/` lists.

What it costs:
one value to build from the answer;
`entry.rs`'s `write_exit` stays the one place that writes a report,
so the order the wrapper has today (plan,
then write) is unchanged;
the type moves again at step 7 into whichever library `foundation` becomes;
and the exit code's move adds 18 files to the same branch,
or a second commit in it.

Worked case:
`git monochromatic-git-policy-cli migrate-to-subvolume --help`
builds `Finished::saying(0, MIGRATE_HELP)`,
`run_migrate` turns it into `Action::ExitBytes { code: 0, stdout: <the help text as bytes>, stderr: [] }`,
and the person sees the same help on standard output and exit 0 as today.
A refusal outside a repository builds `Finished::explaining(REFUSED_EXIT_CODE, <the sentence>)`,
and the sentence stays on standard error.

A yes leaves undecided:
whether `ENGINE_FAILURE_EXIT_CODE` and `REFUSED_EXIT_CODE` move in the same commit,
its own,
or not at all in this round
(they are three outcomes with different deliveries,
not one),
and whether the four published names narrow to `pub(crate)` or keep a `pub` face for the library.

### Signature B: the target tree's placement, beside `Action`

New file `package/git-policy/cli/src/native/invocation/what_a_command_hands_back.rs`,
next to `action.rs`,
with `Finished` (text streams) and `FinishedBytes` (byte streams)
as the target tree draws them.

What it delivers:
nothing to the allow list in this round.
The type sits in the folder `subvolume` already names for `Action`,
so `subvolume`,
`health` and `worktree` go on using `invocation`
and the linter reports no dead line.
At step 7 it delivers less than the plan's prose:
of the three folders,
`converting_the_main_checkout` names `starting_git_and_describing_a_run` today
(`doc/planning/cli-git-source-layout-target-tree-files.md:855`),
but `checking_that_your_setup_works` (line 733) and `converting_a_linked_worktree` (line 830) do not,
so under this placement those two gain a use of the program folder when they stop naming the decision folder.

What it costs:
two types for one idea,
or one bytes type every text caller converts with `into_bytes`;
and the whole delivery waits for step 7.

Worked case:
the same command,
the same help text,
with the value built as `invocation::Finished { code: 0, stdout: MIGRATE_HELP.to_string(), stderr: String::new() }`;
the allow list is byte-for-byte what it was before the round.

A yes leaves undecided:
the same two items as option A.

### Signature C, not offered: no value, the command writes

Each command writes its own answer to two writers and returns a status.
It removes the same lines as A,
but it loses the one place that writes a report
(`entry.rs`'s `write_exit`),
so a later recheck that covers exits would not see these commands.
It is named here because it is the shape a reader reaches for first.

### Ranking

A > B.

The measure:
what each delivers to the split,
which is the allow list's shrinking lines in this round
and the target tree's `uses/` lists at step 7,
against what it changes for a person running the command.

- A over B:
  with the exit code moved,
  A removes two lines now and the same three names at step 7,
  and B removes nothing now and leaves two folders gaining a use at step 7.
  A's extra cost is a second move of the type at step 7,
  which is effort.

## 2. System settings path

### Where the call is today

- `package/git-policy/cli/src/native/repo_read/repo_read_config_scan.rs:296`:
  `scope.system_default = crate::git_system_config::system_config_for(variables);`,
  inside `ScopeEnvironment::from_variables` (line 274),
  and the value decides `scope.unresolvable` on the next line.
  The ask is gated:
  it happens only inside
  `if boolean_ok && !scope.no_system && scope.system.is_none()` (lines 295 to 298),
  and a path that is not absolute makes the scope unresolvable (lines 299 to 307).
- The definition it calls:
  `package/git-policy/cli/src/native/repo_read/git_system_config.rs:215`,
  `pub fn system_config_for(variables: &[(OsString, OsString)]) -> Option<PathBuf>`,
  which reads the forward-target variable and asks that Git.
- The builders of a `ScopeEnvironment`,
  all ten (`rg -n "ScopeEnvironment::from_variables" -g '*.rs' -g '!*_tests.rs'`):
  `index/index_reset_file_surroundings.rs:114`,
  `repo_read/repo_read_query.rs:90`,
  `repo_read/repo_read_layout.rs:264`,
  `:310`,
  `:353`,
  `:411`,
  `repo_read/repo_read_hook.rs:105`,
  `push/auto_push_trigger_ahead_of_upstream.rs:113`,
  `merge/merge_second_pass_tree_attributes.rs:84`,
  and `worktree/worktree_clean_content_attribute_sources.rs:93`.
  An eleventh the earlier draft counted,
  `subvolume/subvolume_platform.rs:133`,
  is a `ConversionEnvironment::from_variables`,
  a different type.

### What the plan says it becomes

The target tree's signature:
"`ScopeEnvironment` ... stops calling `git_system_config::system_config_for`;
the callers that build a `ScopeEnvironment` pass the result"
(`doc/planning/cli-git-source-layout-target-tree.md:6478-6481`).
The plan's prose:
"the caller passes the path in,
so a reader never starts Git"
(`doc/planning/cli-git-source-layout.md:462-463`),
and the owner's own words:
"the system settings path passed to the reader" (line 118).

Which folder stops depending on which:
none today,
because the asker and the reader are two files of one folder.
At step 7 the reader is `configuration_scopes_scan.rs` in `reading_git_settings_from_files`
(target-tree-files:1943),
and the asker is `system_settings_file_of_git.rs` in `starting_git_and_describing_a_run`
(target-tree-files:2602).
Seven release files name `git_system_config` today
(`rg -ln "git_system_config" src/native/repo_read -g '!*_tests.rs'`),
and of the five files of the target reader folder
only `repo_read_config_scan.rs` names it,
so the reader's own `uses/` entry for the asker (`target-tree-files:1938`) dies when that file stops calling it.
The three reader builders (`repo_read_query.rs`,
`repo_read_hook.rs`,
`repo_read_layout.rs`) do not name the asker today,
only the reader;
their folders are `answering_questions_without_starting_git` (its `uses/` at 605)
and `finding_the_git_folders` (its `uses/` at 1038).
Under option A those three would call the asker themselves,
which adds `starting_git_and_describing_a_run` to `finding_the_git_folders`' list (1038 does not name it today)
and keeps it in `answering_questions_without_starting_git`'s (605 does).

### Signature A: the parameter, at every caller that builds a scope

```rust
//! package/git-policy/cli/src/native/repo_read/repo_read_config_scan.rs
/// The facts of a scope,
    /// settled from variables and one path.
impl ScopeEnvironment {
    /// The facts of an environment as a child would see it,
    /// with the system configuration file's path,
    /// which the caller learned from that Git.
    pub(crate) fn from_variables(
        variables: &[(OsString, OsString)],
        system_default: Option<PathBuf>,
    ) -> ScopeEnvironment {
        // The three gates and the relative-path check are today's, in today's order:
        // the ask moves out, the decision about what an unknown path means does not.
        let mut scope: ScopeEnvironment = ScopeEnvironment {
            home: nonempty_variable(variables, "HOME"),
            xdg: nonempty_variable(variables, "XDG_CONFIG_HOME"),
            no_system,
            system: path_variable(variables, "GIT_CONFIG_SYSTEM"),
            system_default,
            global: path_variable(variables, "GIT_CONFIG_GLOBAL"),
            unresolvable: !boolean_ok,
        };
        if boolean_ok && !scope.no_system && scope.system.is_none() && scope.system_default.is_none() {
            scope.unresolvable = true;
        }
        let mut relative: bool = false;
        for path in [&scope.home, &scope.xdg, &scope.system, &scope.global] {
            if let Some(found) = path.as_ref()
                && !found.is_absolute()
            {
                relative = true;
            }
        }
        scope.unresolvable = scope.unresolvable || relative;
        return scope;
    }
}
```

and the same builder rewritten,
`repo_read/repo_read_layout.rs:264`:

```rust
//! package/git-policy/cli/src/native/repo_read/repo_read_layout.rs
let scope: ScopeEnvironment = ScopeEnvironment::from_variables(
    variables.as_slice(),
    crate::git_system_config::system_config_for(variables.as_slice()),
);
```

The gate stays where it is,
in the caller's read of the variables:
a builder that knows `GIT_CONFIG_NOSYSTEM` is set passes `None` without asking,
and a builder that does not passes the ask's answer.
That is a correction to an earlier draft,
which put the ask outside the gate and would have started Git (or read its cache) for every builder,
even with `GIT_CONFIG_NOSYSTEM=1` or `GIT_CONFIG_SYSTEM` set.

What it delivers:
the reader's file stops starting Git,
so at step 7 the reader's folder loses its use of the asker;
the ask moves to every caller that builds a scope,
including the three reader builders,
which adds the asker to `finding_the_git_folders`' list and keeps it in `answering_questions_without_starting_git`'s.
No allow-list line changes in this round.

What it costs:
one line at each of the ten builders;
and the three reader builders now start Git where their folders are named for not doing so,
which is the cost that buys the reader's line.

Worked case:
`git config --get core.hooksPath` runs on a path that builds a scope in
`repo_read_layout.rs`;
the builder passes `system_config_for(variables)`,
which answers `Some("/etc/gitconfig")` from the forward-target Git,
and `configuration_scopes_scan.rs` resolves the scopes without starting a Git of its own.
The person sees the same value as today.

A yes leaves undecided:
whether a builder that cannot learn a path passes `None` (making the scope unresolvable,
as today) or whether the round gives such a scope a sentence of its own.

### Signature B: the parameter, at the ten builders only

The same parameter,
and the three reader files go on calling `system_config_for` themselves.

What it delivers:
the reader's folder line dies at step 7,
and nothing else:
`answering_questions_without_starting_git` keeps the asker in its list,
and `finding_the_git_folders` never gains it.

What it costs:
the reader's file still starts Git for the three reader builders,
so the promise in `answering_questions_without_starting_git`'s name holds only where a value was handed over;
and the round costs one line at ten builders instead of thirteen.

Worked case:
the same command,
the same value;
the difference is which file runs the ask,
and it is visible in the folder lists at step 7,
not to the person.

A yes leaves undecided:
the same item as A.

### Signature C, not offered: one answer per process, learned once

A `OnceLock` in `git_system_config.rs`,
learned once at the service's start and read by every reader.
It delivers nothing to the folder lists (the reader still names the file that remembers),
it fails the guide's rule for anything a test reads
(`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
section "Nothing a test changes may be process-wide",
which names the current directory,
the environment,
the umask,
a signal's disposition and a resource limit,
and which a `OnceLock` set by a test would join in spirit though it names no static),
it remembers a failed ask as `None` for good,
and the ask blocks its caller,
so inside a `OnceLock` closure it would block every other reader.
It is not a pass of the owner's words either:
a process-wide value is not something a caller passes.

### Ranking

A > B.

The measure:
what each delivers to the split,
which here is how many target folders stop naming the asker at step 7,
against what each changes for the files whose names promise no Git start.

- A over B:
  A removes two `uses/` entries (the reader's,
  and `answering_questions_without_starting_git`'s) and adds one to `finding_the_git_folders`;
  B removes one and adds none,
  and leaves three reader files starting Git in folders named for not doing so.
  B's advantage is a diff of ten lines instead of thirteen,
  which is effort.

## 3. After-stop action

### Where the call is today

- `package/git-policy/cli/src/native/push/child_watch.rs:195`:
  `start` hands `LockLedger::none()` to `start_with_locks` (line 206),
  which hands it to `start_under_stop` (line 227),
  which hands it to `start_when_a_task_may_run` (line 260),
  which hands it to `watch_loop` (line 302).
  `impl Drop for ChildWatch` is at line 483.
- `package/git-policy/cli/src/native/push/child_watch_task_loop_and_notice.rs:70`:
  `watch_loop`,
  whose `locks: LockLedger` (line 77) goes to `stop_process` (lines 162,
  180 and 324)
  and to `end_for_the_stop` (lines 123 and 197).
- `package/git-policy/cli/src/native/push/child_watch_clock_marks_and_stopping.rs:308`:
  `stop_process(pid, locks, incoming, root_pin)`,
  whose sweep is at lines 372 to 383,
  inside `crate::blocking_job::run_blocking_job`.
  Its own comment says the locks are "swept then,
  or when the watch is dropped,
  whichever comes first".
- The two ledger builds inside the watcher:
  `package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs:238` and
  `package/git-policy/cli/src/native/push/child_watch_output_watched_on_the_runtime.rs:243`.
- The five one-shot run functions:
  `output_watched` (line 99),
  `output_watched_stopping` (121),
  `output_watched_within` (160),
  `output_watched_within_for_a_stop` (196),
  and the awaited `output_watched_within_for_a_stop_awaiting`
  (`push/child_watch_output_watched_on_the_runtime.rs:202`).
  All five return `std::io::Result<Output>` today.
- The three release call sites of those five:
  `package/git-policy/cli/src/native/transaction/transaction_git.rs:386` (the synchronous face),
  `transaction/transaction_git_awaiting.rs:291` (the awaited face),
  and `package/git-policy/cli/src/native/repo_read/git_metadata.rs:443`,
  which calls `output_watched_within`
  and so reaches `LockLedger::before_start` today.
  An earlier draft said that read passed no ledger;
  that was wrong.
- `child_watch::start_with_locks` is a published name
  (`package/git-policy/cli/src/native/source_guard/items_nameable_from_outside_release.txt:6113`),
  and both options remove it,
  so both edit that list.

### What the sweep is, and the window in which a dropped watch sweeps today

The sweep reads and removes reference lock files a stopped step left.
It runs in one place today:
the end of `stop_process`,
on the blocking pool.
`stop_process` runs from exactly three places in the watcher's loop,
when the ceiling passed (line 157),
when the quiet window passed (line 179),
and when the stop arm fired (`end_for_the_stop`,
line 324).
A watch dropped while none of those is under way sweeps nothing:
`watch_loop` returns on a closed channel at line 98 and at line 193 without a sweep,
and `Drop for ChildWatch` waits for the task and keeps the longest gap.
A watch dropped **while a stop is under way** does sweep today:
the wait after the kill reads the closed channel and breaks out to the sweep
(`package/git-policy/cli/src/native/push/child_watch_clock_marks_and_stopping.rs:362-371`).
So the window is narrow,
and this file names it instead of calling it a general property:
the loss an option can cause is a sweep lost when a stop is under way and the caller stops reading,
and what that costs is that the lock waits for the next command's stale-lock judgement
(`push/stop_of_a_push_and_sweep_of_its_locks.rs:432-440`).

Nothing here measures how often a landing's or a reader's step is in that window.
The window is read from those three callers of `stop_process` and from the two returns of `watch_loop`.

### What the plan says it becomes

The target tree's signature:
"`start_with_locks(...)` and `watch_loop(...)` lose the `locks: LockLedger` parameter,
and the line in `child_watch` that calls `LockLedger::before_start(command)` moves out.
The watcher returns the `StopReport` it already has;
the caller in `bounded_push` and `transaction_git` builds the `LockLedger` before the start and sweeps after the stop.
The lock folder is in the program,
above the watcher library,
so the watcher cannot name it"
(`doc/planning/cli-git-source-layout-target-tree.md:6467-6477`).

The plan's prose:
"its caller hands it what to do after a stop"
(`doc/planning/cli-git-source-layout.md:458`),
from the owner's answer
"Make all four (Recommended)" (lines 118 to 120).

Which folder stops depending on which:
no allow-list line,
and both options add two uses that no line can record.
Measured with one linter run over the package with each drawing applied,
and one on the unedited tree as a control:

- unedited:
  12 `rust/folder-uses-links` findings,
  none of them about `lock`;
- either drawing:
  16,
  the four new ones being
  `src/native/transaction uses the folder src/native/lock` and
  `src/native/repo_read uses the folder src/native/lock`,
  each reported twice:
  once for a missing `uses/` link,
  and once as
  "both sit inside one loop the allow list tolerates today ... no line of the list records it,
  so a tolerated loop may only lose uses".
  The linter's help line names the link the round would create:
  `mkdir --parents src/native/transaction/uses && ln --symbolic ../../lock src/native/transaction/uses/lock`.

What survives either option,
and is not part of it:
both one-shot files call `crate::lock_holder_records::note_child_started(child.id())`
(`package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs:249` and
`package/git-policy/cli/src/native/push/child_watch_output_watched_on_the_runtime.rs:251`),
and the target tree has no row for `lock_holder_records.rs`,
so the watcher library goes on naming the lock area after either option.
Taking that use out is a second change,
which neither option makes,
and the library lift needs it as much as it needs this one.
The decision record says such a finding has an empty key,
"so no line of the list can record it"
(`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md:115-119`),
and the owner's "Merge red,
untangle later" covers the wiring of a built ZFS piece only;
"any other new use inside a tolerated loop is still a finding to remove before its branch merges"
is the coordinating session's reading,
with the veto open (lines 162 to 165).
So whichever way this question is answered,
the after-stop commit lands two uses that must be untangled before it can be lint-clean,
and a third placement exists that adds neither:
the ledger built and swept by a `push/` function the caller calls,
which adds no use because `transaction` and `repo_read` already name `push`.
That placement is not drawn here,
its shape (an opaque sweeper type in `push`,
so the caller never names `lock`) is not measured,
and it needs its own linter run.

### Signature A: the watcher calls back

```rust
//! package/git-policy/cli/src/native/push/child_watch.rs
/// What a caller wants done once the watch has ended its child.
pub trait AfterTheStop: Send + Sync {
    /// Run after the child and the processes below it are gone.
    fn after_the_stop(&self);
}

/// The faces of the watcher.
impl ChildWatch {
    /// Start watching `pid`,
    ///  calling `after` once the watch has ended the child and its processes below it.
    pub fn start_after_the_stop(
        waiting_for: &str,
        pid: u32,
        terms: WatchTerms,
        after: Option<std::sync::Arc<dyn AfterTheStop>>,
    ) -> Self;
}
```

and the caller's behaviour,
at `transaction/transaction_git.rs`:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git.rs
/// The locks a step of this transaction can leave.
struct SweepTheLocks {
    /// The ledger of the command whose step is watched.
    ledger: crate::stopped_git_locks::LockLedger,
}

/// What the program wants done once its child is gone.
impl crate::child_watch::AfterTheStop for SweepTheLocks {
    /// Sweep the lock files the stopped step left.
    fn after_the_stop(&self) {
        let _swept: Vec<std::path::PathBuf> = self.ledger.sweep();
    }
}

let after: std::sync::Arc<dyn crate::child_watch::AfterTheStop> = std::sync::Arc::new(SweepTheLocks {
    ledger: crate::stopped_git_locks::LockLedger::before_start(&step.command),
});
let watched: std::io::Result<std::process::Output> = output_watched_within_for_a_stop(
    &mut step.command,
    step.subject.as_str(),
    step.ledger_step.as_str(),
    step.terms,
    step.stoppable,
    step.bound,
    step.a_stop_lets_it_finish,
    request.started_child.as_ref(),
    Some(after),
);
```

The callback travels the same path the ledger travels today:
`watch_loop` and `stop_process` take it in place of `locks`,
and the double call runs inside the same `run_blocking_job` where the sweep runs today.
The awaited face and `git_metadata.rs` are the same drawing with the same value,
and `git_metadata.rs` moves from `output_watched_within` to
`output_watched_within_for_a_stop` so it can hand one over.

What it delivers:
`clearing_leftover_reference_locks` out of the watcher library's folder at step 7
(`doc/planning/cli-git-source-layout-target-tree-files.md:322`),
which is what the library lift needs;
the sweep's moment unchanged,
including the stop-under-way window;
and no new folder use beyond the two above.

What it costs:
one trait in the library whose only implementations are the program's,
which the plan's note disfavours for these changes ("none of the changes below hands a closure;
each hands a value or an enum",
`doc/planning/cli-git-source-layout-target-tree.md:6427-6429`);
a value per call site (an `Arc` and a small struct);
and the published `start_with_locks` line in the source guard's list changes.

Whether `LockLedger` can be the state of a `Send + Sync` implementation is settled by reading,
not by a compile:
it holds a `Vec<PathBuf>` and an optional `Judge`,
whose fields are a `PathBuf`,
a `Duration`,
an `Option<SystemTime>`,
a `Vec<String>` and an `Option<Vec<i64>>`
(`package/git-policy/cli/src/native/lock/stopped_git_locks.rs:96-103`,
`package/git-policy/cli/src/native/lock/stale_ref_lock_judge.rs:94-117`),
so it is both.

Worked case:
a landing's `git update-ref` step is stopped by the service.
The watch ends the child,
the wait after the kill passes,
and the watcher's task calls
`SweepTheLocks::after_the_stop` on the blocking pool,
which removes the two `refs/heads/*.lock` files it can prove stale and leaves one it cannot.
The landing's next step finds no lock it must judge.
If the caller dropped its watch 40 seconds into the same stop,
the sweep still happens,
because the callback lives in the watch's task,
not in the caller's reading.

Compiled:
this shape was put into this worktree and
`cargo check --locked --offline --all-targets` printed
`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 33.89s`
with no error and no warning.
Changes:
11 files,
106 insertions and 38 deletions,
including 8 control call lines in two control files and one control's ledger path.

A yes leaves undecided:
whether the same callback carries anything else later,
which the trait's shape allows and nothing asks for now.

### Signature B: the caller sweeps

The five one-shot functions answer `WatchedOutput`:

```rust
//! package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs
/// The answer of a watched step and how the watch ended it.
#[derive(Debug)]
pub struct WatchedOutput {
    /// What the step wrote,
    ///  or why the watch or the start ended it.
    pub output: std::io::Result<Output>,
    /// Whether the watch ended the child,
    ///  which is when its locks must be swept.
    pub ended_by_the_watch: bool,
}
```

where `ended_by_the_watch` is the three endings that ran the stop
(`ended_by_the_stop`,
or `ceiling_hit`/`expired` when the watch could end the step),
and each early return that ends a watch takes `watch.ended_by_the_stop()` with it,
which is exactly the stop-under-way window.
`output` holds the watch-ended error,
so a caller can sweep before it decides to return;
the outer `Result` is the start failure alone,
where no child exists and nothing is swept today.

and the three call sites,
each in a place that can do what it must:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git.rs:
//!  the synchronous face
let locks: crate::stopped_git_locks::LockLedger =
    crate::stopped_git_locks::LockLedger::before_start(&step.command);
let watched: crate::child_watch_one_shot_run::WatchedOutput = output_watched_within_for_a_stop(
    &mut step.command,
    step.subject.as_str(),
    step.ledger_step.as_str(),
    step.terms,
    step.stoppable,
    step.bound,
    step.a_stop_lets_it_finish,
    request.started_child.as_ref(),
)?;
// This face is on no runtime worker, so the sweep runs here.
if watched.ended_by_the_watch {
    let _swept: Vec<std::path::PathBuf> = locks.sweep();
}
return Ok(step_finished(step.ledger_step.as_str(), request, step.began, watched.output?));

// package/git-policy/cli/src/native/transaction/transaction_git_awaiting.rs, the awaited face
let watched: crate::child_watch_one_shot_run::WatchedOutput = watched?;
if watched.ended_by_the_watch
    && let Err(error) = crate::blocking_job::run_blocking_job(move || {
        locks.sweep();
    })
    .await
{
    crate::diagnostic_log::debug("transactionGit", format!("the lock files of the stopped step were not swept: {error}").as_str());
}

// package/git-policy/cli/src/native/repo_read/git_metadata.rs
let locks: crate::stopped_git_locks::LockLedger =
    crate::stopped_git_locks::LockLedger::before_start(&command);
let watched: crate::child_watch_one_shot_run::WatchedOutput = output_watched_within(
    &mut command, subject.as_str(), subject.as_str(), terms, true, StepBound::EndAt(ceiling),
)?;
if watched.ended_by_the_watch {
    let _swept: Vec<std::path::PathBuf> = locks.sweep();
}
watched.output
```

What it delivers:
the same library entry as A,
and no new folder use beyond the two above.

What it costs:
18 control lines in three files read `WatchedOutput` instead of `Output` (compiler-counted:
the first check of this drawing printed 16 `E0308` and 2 `E0609` in three control files),
so the round edits controls the earlier draft did not list;
`git_metadata.rs` moves to the innermost function,
which the earlier draft did not draw;
and the sweep's moment moves from the watcher's ending to the caller's reading.

Worked case:
the same landing step and the same stop.
The synchronous face sweeps inline after the watch answers;
the awaited face sweeps on the blocking pool;
`git_metadata.rs` sweeps after its watched read.
A caller that reads the answer late but does read it sweeps;
a caller that drops the watch and returns during the stop sweeps nothing,
and the lock waits for the next command's stale-lock judgement.

Compiled:
this shape was put into this worktree and
`cargo check --locked --offline --all-targets` printed
`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 27.89s`,
with no error and no warning. Changes: 8 files, 150 insertions and 45 deletions.
With the awaited face left as it was, the same check prints
`src/native/transaction/transaction_git_awaiting.rs:291:58:
error[E0308]:
mismatched types:
expected \`Result<Output,
Error>\`,
found \`Result<WatchedOutput,
Error>\``,
so that combination cannot be built.

A yes leaves undecided:
whether the five convenience functions keep answering `WatchedOutput` or map it away for their callers,
which is a shape the round chooses when it edits the 18 control lines.

### Ranking

A > B.

The measure:
what each delivers to the split,
which here is the same library folder for both,
against what each changes for a stopped step whose locks are swept.

- A over B:
  both remove the same use,
  and A keeps the sweep where it runs today,
  inside the watcher's task on the blocking pool,
  so the stop-under-way window keeps its sweep;
  B moves the sweep to every caller and loses it in that window,
  and it edits 18 control lines against A's 8.
  B's advantage is a plain answer value instead of a trait object,
  which is the shape the target tree asks for,
  and that is a cost of A's shape,
  not a reason to rank A lower.

## 4. Current step record

### Where the call is today

- `package/git-policy/cli/src/native/transaction/transaction_git_step_about_to_start.rs:228`:
  `let record: Option<StepGuard> = if matches!(bound, StepBound::Unbounded) { enter_step(...) } else { None };`,
  whose comment says "only such a step is announced" (lines 214 to 222).
  The guard is held in the value the function answers,
  so the record lives as long as the step does.
- `package/git-policy/cli/src/native/transaction/transaction_git_step_bounds.rs:112`:
  `not_ended_bound(class_ceiling)`,
  which asks `crate::service_busy_step_record_on_this_thread::on_landing_thread()` (line 113)
  and answers `StepBound::Unbounded` on a landing,
  else `leave_running_bound(class_ceiling)`.
  `step_bound(request: &GitRequest)` reaches it at line 80.
- `package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs:231` and
  `push/child_watch_output_watched_on_the_runtime.rs:236`:
  `crate::service_busy_step_record_on_this_thread::on_landing_thread(),`
  as the first argument of `begin_unfinished_work`.
- The record itself:
  `package/git-policy/cli/src/native/service/service_busy_step_record_on_this_thread.rs`
  (`on_landing_thread` at line 109,
  `use_on_this_thread` at 114,
  `enter_step` at 176),
  which reads and writes through `task_local! { static LANDING: LandingState; }`
  (`package/git-policy/cli/src/native/landing/landing_state_carried_by_its_task.rs:47`)
  and writes through `service_busy`'s `LandingBoard`,
  `STEP_FILE` and `now_millis`.
- The landing that registers its board:
  `package/git-policy/cli/src/native/service/service_server.rs:174`,
  and the detail a landing sets:
  `package/git-policy/cli/src/native/landing/service_land_attempt_branch_move.rs:99`.

The owner's choice 4 was "Rewrite per-thread state",
and the plan that records it says each `thread_local!` becomes a task-local or a context value
(`doc/planning/git-policy-cli-full-tokio.md:1655-1657`).
The target tree's word "thread-local" predates that.

### What the plan says it becomes

The target tree's signature:
"`transaction_git` stops calling `service_busy::enter_step` and `on_landing_thread`.
A thread-local record (step file path and detail) moves to the new program file `current_step_record`
in the folder `how_long_the_program_waits_for_git_steps` ...
the runner writes it,
the landing code registers its file path there,
and `LandingBoard` stays in `service_busy`"
(`doc/planning/cli-git-source-layout-target-tree.md:6455-6466`).

The plan's prose:
"a small \"current step\" record in the basic tools that the runner writes and the service reads"
(`doc/planning/cli-git-source-layout.md:455`),
which is a third placement again.

Which folder stops depending on which:
no allow-list line today,
measured by the survivor:
`transaction` keeps `service` through `wait_ceilings`
(`package/git-policy/cli/src/native/transaction/transaction_phase.rs:42`).
The delivery that matters is the watcher library's:
`landing_commits` out of the library's `uses/` line
(`doc/planning/cli-git-source-layout-target-tree-files.md:322`),
which needs the two watched-run sites in this change,
not only `transaction_git`.

The overlapped change:
the target tree's "Plain terms for the watcher"
(`doc/planning/cli-git-source-layout-target-tree.md:6514-6522`) takes
`WatchTerms::for_class` out of `child_watch_clock_marks_and_stopping` at step 5,
and this round adds a field to `WatchTerms` and a parameter to `for_class`.
Both edits land on one function,
in two steps,
and the second is small.

### How many places build the two kinds of value

Measured in this round:

- 27 release functions return a `GitRequest`
  (`rg -n -- "-> GitRequest" src -g '*.rs' -g '!*_tests.rs'`):
  18 in `landing/`,
  4 in `commit/`,
  3 in `worktree/`,
  1 in `config/` and 1 in `shadow/`.
  `GitRequest` derives `Default`
  (`package/git-policy/cli/src/native/transaction/transaction_git_request_and_reply_types.rs:88`),
  so a new field of a defaultable type compiles at all 27 without a word.
  Which of the 27 run on a landing is not enumerated by any read of this round:
  the answer today is ambient,
  so any step started while the landing runs carries it.
- 63 initializers of `GitContext` in the crate,
  22 in release code and 41 in controls,
  counted by the compiler:
  with a field added,
  `cargo check` printed one `E0063` for each
  (41 in release files counting the two builds,
  22 distinct).
  `GitContext` has no `Default`,
  and it derives `Eq` and `PartialEq`,
  so a field on it must itself be `Eq` and `PartialEq`:
  the same run printed
  `error[E0277]: the trait bound \`CurrentStepRecord:
  std::cmp::Eq\` is not satisfied`
  and `error[E0369]:
  binary operation \`==\` cannot be applied to type \`CurrentStepRecord\``.

### Signature A: the record rides the run's `GitContext`

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git_request_and_reply_types.rs
/// Where the landing a run serves writes the step it is inside,
///  and what that step does.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CurrentStepRecord {
    /// The folder the step file lives in;
    ///  none when this run is not a landing.
    directory: Option<PathBuf>,
    /// What the write that follows does.
    detail: Option<String>,
}

/// The record's own steps.
impl CurrentStepRecord {
    /// The record of a run that is not a landing.
    pub(crate) fn nothing() -> CurrentStepRecord;
    /// The record of the landing that writes into `directory`.
    pub(crate) fn in_the_landing_at(directory: PathBuf) -> CurrentStepRecord;
    /// Whether this run is a landing,
    ///  where a step is never ended.
    pub(crate) fn on_a_landing(&self) -> bool;
    /// Record that this step is inside a step that is never ended,
    ///  until the guard is dropped.
    pub(crate) fn enter(&self, command: &str) -> Option<StepGuard>;
}

/// What:
///  The run a request belongs to.
/// Why:
///   It is built once per operation and passed down,
/// so a fact of the run belongs here and not on a request.
pub struct GitContext {
    // ... the real Git, the overlay, the global prefix and the common Git directory, as today ...
    /// Where the landing this run serves writes the step it is inside.
    pub step_record: CurrentStepRecord,
}
```

and the bound asks the run,
while the guard stays where it is:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git_step_bounds.rs
/// What:
///  What ends a step:
/// a ceiling,
///  a stop,
///  or nothing.
/// Why:
///   Both faces pass the same bound,
/// so a step cannot be waited for longer on one of them.
pub(crate) fn step_bound(context: &GitContext, request: &GitRequest) -> StepBound {
    // ... the class ceiling, as today ...
    if may_be_ended(request) {
        return StepBound::EndAt(class_ceiling);
    }
    if is_side_ref_write(request) {
        return leave_running_bound(class_ceiling);
    }
    return not_ended_bound(class_ceiling, context.step_record.on_a_landing());
}

/// The bound of a step that is never ended:
///  unbounded on a landing,
///  else left running.
fn not_ended_bound(class_ceiling: Duration, on_a_landing: bool) -> StepBound {
    if on_a_landing {
        return StepBound::Unbounded;
    }
    return leave_running_bound(class_ceiling);
}

//! package/git-policy/cli/src/native/transaction/transaction_git_step_about_to_start.rs
// Only a step that is never ended is announced, as today.
let record: Option<StepGuard> = if matches!(bound, StepBound::Unbounded) {
    context.step_record.enter(
        format!("{} in {}", command_text(&request.arguments), request.cwd.display()).as_str(),
    )
} else {
    None
};
```

What it delivers:
the same library line as the other shapes,
with a use the folder rule can see;
a record every build site must answer,
because `GitContext` has no `Default`;
and no need to know which of the 27 request builders is reached from a landing,
which is the fact nothing in this round could count.

What it costs:
a field at all 63 `GitContext` initializers,
each refused by the compiler until it answers
(22 release,
41 control);
`Eq` and `PartialEq` on the record;
and a `step_bound` that takes the context,
which is two call sites in the two faces.

Worked case:
a landing runs `git update-ref`;
the context the landing built carries `CurrentStepRecord::in_the_landing_at(<the board's folder>)`,
`step_bound` answers `StepBound::Unbounded`,
`step_about_to_start` enters the step because the bound is unbounded,
and the client that reads the step file (`service/service_busy_words_of_a_stuck_write.rs:33`)
sees the step while it runs.
A read of one object started beside it,
with the same context,
takes the same answer,
so its watch never ends it.
A command outside the service builds its context with `CurrentStepRecord::nothing()`,
gets `leave_running_bound` for a not-rerunable step,
and no file is written.

A yes leaves undecided:
where the record's file writer lives
(the drawing keeps `service_busy`'s writer and moves only the carrier and the guard),
and how the watcher's `begin_unfinished_work` answer reaches the two watched-run sites,
which is the same `WatchTerms` field under either carrier.

### Signature B: the record stays task-carried, and moves to a program file

The record is today's `task_local!`,
moved out of `service_busy` into
`how_long_the_program_waits_for_git_steps/current_step_record.rs`,
with `enter_step`,
`on_landing_thread`,
`use_on_this_thread` and the file's writer beside it,
and `LandingBoard` left in `service_busy`.
The watcher's answer rides a field on `WatchTerms` (`on_a_landing: bool`),
set by whoever builds the terms.

What it delivers:
the same library line,
and no build site to answer,
because the answer comes from the task.
What it costs:
the record stays state no line records,
which is what the folder rule cannot see;
20 `WatchTerms {` literals in seven control files must set the new field
(`rg -c "WatchTerms \{" -g '*_tests.rs'`),
and six call sites of `for_class` pass the answer.

Worked case:
the same landing step;
the answer comes from the task the landing runs in,
and the step file is written exactly as today.

A yes leaves undecided:
the same two items as A,
and whether the state becomes a value later.

### Signature C, not offered: a field on `GitRequest`

`GitRequest` is the value the runner already has,
and a record of a folder and an optional detail would be a natural field.
It is not offered because its default cannot be made safe:
`GitRequest` derives `Default`,
so a field of a defaultable type compiles at all 27 builders without a word,
and a step built without the field would answer "not a landing",
which is `leave_running_bound`,
which ends a landing's write mid-flight.
Making the record non-defaultable is refused by the compiler while `GitRequest` derives `Default`:
with the field added and the derive removed from the record,
`cargo check` printed
`src/native/transaction/transaction_git_request_and_reply_types.rs:170:5: error[E0277]:
the trait bound \`CurrentStepRecord:
std::default::Default\` is not satisfied`.
With the derive kept, the same tree compiles green
(`Finished \`dev\` profile ... in 27.44s`,
3 files,
57 insertions) and that silence is exactly the defect.

### Ranking

A > B > C.

The measure:
what each makes visible to the folder rule,
against what each can silently get wrong.

- A over B:
  both deliver the library line,
  and A puts the use where a line can record it,
  so a program folder cannot reach the record without naming it and the tangle cannot regrow through a name;
  B leaves the use invisible and keeps the tangle's cause.
  A's cost is 63 sites against B's none,
  which is effort.
- B over C:
  B cannot be forgotten,
  because nothing has to be remembered;
  C compiles at all 27 builders with the wrong answer for a landing step,
  and cannot be made to refuse that by the compiler while `GitRequest` derives `Default`.

## 5. Run this next

Told here and not asked:
both recursions are this change
(`doc/planning/cli-git-source-layout.md:111-115`).
What follows is what the round would write and what this round found out about it.

### Where the call is today

The push half:

- `package/git-policy/cli/src/native/push/stacked_merge_push.rs:36`:
  `use crate::entry::plan_invocation;`,
  and line 40 `run_stacked_push,`;
  `pub fn gated_merge_push(...) -> Result<i32, SingleFlightError>` at line 209,
  whose last two lines are
  `let action: Action = plan_invocation(arguments.as_slice(), environment.as_slice(), &inputs);`
  and `return Ok(run_planned(action));` (lines 226 to 227).
- `fn run_planned(action: Action) -> i32` at line 119,
  whose `Action::ForwardPush` arm calls `run_stacked_push(&forward, &gated_merge_push)` at line 161.
- `package/git-policy/cli/src/native/push/push_stacking.rs:353`:
  `pub fn run_stacked_push(forward: &PushForward, merge_push: MergePush<'_>) -> i32`,
  whose loop state is `StackRequest<'a>` (line 110)
  and whose step is `stack_failed_push(...) -> Option<i32>` (line 195).
- The top level that would run the next command:
  `package/git-policy/cli/src/native/invocation/entry.rs:580`
  and `invocation/entry_run_forwarded_actions.rs`,
  which `package/git-policy/cli/src/native/invocation/entry.rs:543` reaches.

The merge half:

- `package/git-policy/cli/src/native/merge/merge_second_pass_finish.rs:63`:
  `use crate::entry::run_nested_process;`,
  `fn run_nested(...) -> i32` at line 308,
  counting `NESTED_FINISHES` around `run_nested_process` (lines 312 to 314),
  called at lines 340,
  392 and 418.
- `NESTED_FINISHES` and `FINISH_BUDGET`
  (`merge/merge_second_pass_finish_nesting.rs:31` and `:35`,
  with `finish_budget` at 76 and `nesting_allows` at 91),
  and `plan_finish` refusing at `package/git-policy/cli/src/native/merge/merge_second_pass_finish.rs:219`
  when the budget is spent.
  It is a **depth**:
  `NESTED_FINISHES` is added before and subtracted after each nested process,
  and `nesting_allows(running, budget)` compares that depth with the budget.
- `invocation/entry.rs:614`:
  `pub fn run_nested_process(...) -> i32`.
- What runs on the status afterwards:
  inside `finish`,
  the commit step's status decides whether `cherry-pick --continue` runs (line 418);
  after `finish` answers,
  `merge_second_pass_worktree::resolve_after` reads the index when the status is not zero
  (`package/git-policy/cli/src/native/merge/merge_second_pass_worktree.rs:116`);
  and after that,
  `lock/index_writer_forward.rs:342` sweeps the worktrees a merge landed.
- The path the status travels up:
  `finish` (line 356) is called by `package/git-policy/cli/src/native/merge/merge_second_pass_worktree.rs:267`,
  whose `resolve_after` is called by `lock/index_writer_forward.rs:342`
  inside `run_coordinated_forward(...) -> CoordinatedRun` (line 202),
  which `entry.rs`'s `ForwardCoordinated` arm runs.
  `second_pass_watch` is a local of that function (line 308),
  not of `entry.rs`.

### What the plan says it becomes

"New file `what_a_command_hands_back` in the program folder `starting_git_and_describing_a_run`
defines `NextCommand { arguments, environment }` and `Finished { code, stdout, stderr }`" ...
"The stacking loop in `push_stacking` becomes a step function:
`StackingStep::{Done(i32), RunThis { command: NextCommand, resume: StackingState }}`" ...
"`merge_second_pass_finish::run_nested` becomes `FinishStep::{Done(i32), RunThis(NextCommand)}`;
the nesting counter `NESTED_FINISHES` moves to the loop" ...
"Consumer:
the top-level loop in the program crate (`entry_run_forwarded_actions`) runs the command with
`entry::run_process`'s planning,
feeds the status back to the step function,
and repeats until `Done`.
A depth guard replaces the counter"
(`doc/planning/cli-git-source-layout-target-tree.md:6434-6454`).

### Which folder stops depending on which

No allow-list line today for the push half.
For the merge half,
`uses src/native/merge -> src/native/invocation`
(`package/git-policy/cli/layout-allow-list/folder-rule.txt:820`) stays alive:
ten release files of `merge/` name `invocation` modules
(`rg -ln "crate::(entry|plan_invocation|run_nested_process|action::|child_environment|forwarded_command|escape_hatch|repository_selection)" src/native/merge -g '*.rs' -g '!*_tests.rs'`),
and only one of them names `run_nested_process`.
So the delivery of this change is the rule the plan wants restored,
"the top level is called only from the top",
not a line.
An earlier draft claimed the line rested on one file;
that was wrong.

At step 7,
the push half takes `what_happens_when_you_type_a_command` out of
`working_out_what_a_typed_push_would_send`'s `uses/` line (target-tree-files:3757)
and out of `rerunning_a_merge_word_by_word`'s (2263).

### What the round would write, and what this round could not settle

The signatures the round would compile first,
each named as new where it is new:

- `NextCommand { arguments, environment }` and `Finished { code, stdout, stderr }` in the new file;
- `StackingState<'a>` (today's `StackRequest` plus the status its decision read) and
  `StackingStep<'a>`;
- `FinishStep` and a resume value per nested finish
  (`EndingTheRebase`,
  `Committing`,
  `ContinuingTheSequence`),
  because a finish hands up two commands
  (the commit,
  then `--continue`) and each needs its own continuation;
- a `Pending` stack in `entry.rs`,
  because a finish's second command can itself stop and hand one up;
- the budget as a depth:
  pushed when a finish is entered,
  popped when it answers,
  with `nesting_allows(running, finish_budget(...))` deciding,
  as `plan_finish` does today.

An earlier draft's drawing of this loop did not hold:
its `Pending::Stacking` held a `StackingState<'static>` that cannot carry the borrows `StackingState<'a>` carries,
`resume` was drawn with two arguments and matched against a type it does not return,
its helper names existed nowhere,
and `Finish` and `Watch` were called private when both are `pub struct`
(`merge/merge_second_pass_finish.rs:151`,
`merge/merge_second_pass_worktree_watch.rs:48`).
Those are corrected here as shapes,
not as compiled code:
this round compiled nothing for run this next,
because the merge half is the round itself.

What no reading settles:
how much of the merge path's own controls assume a status returned rather than a command handed up,
and where `run_coordinated_forward` is cut so the second pass's watch travels with the stack.

## Across the five: the order they can land in

Which files two of them both touch,
for the recommended options (1A,
2A,
3A,
4A,
5A):

- Commands that only finish and run this next share `push/stacked_merge_push.rs`,
  and three more files (`invocation/entry.rs`,
  `invocation/entry_run_forwarded_actions.rs`
  and `lock/index_writer_forward.rs`) only if the exit code moves in the same commit.
  Without that move,
  this change touches none of those three.
- After-stop and current step record share four files:
  `push/child_watch_one_shot_run.rs`,
  `push/child_watch_output_watched_on_the_runtime.rs`
  (the after-stop change removes the ledger,
  the step-record change hands the landing answer to `WatchTerms`),
  `push/child_watch_clock_marks_and_stopping.rs`,
  and `transaction/transaction_git.rs`
  (the after-stop change builds the ledger or the callback there,
  and the step-record change reads the context there).
- The system settings path touches no file another change touches:
  its ten builders are in `index/`,
  `repo_read/` (four files),
  `push/`,
  `merge/` and `worktree/`.
- The attributes path changes nothing under option 5A.

So round 4 is at most three units at once:
`{4}`,
`{3 then 2}` and `{1 then 5}`,
and the two pairs are ordered inside themselves.
The plan's order
(`doc/planning/cli-git-source-layout-target-tree.md:8185-8191`) lists them
finished results,
system settings path,
after-stop action,
current step record,
run this next,
which is compatible with all three units and with the pairs' order changed.
Following it makes a round's commits read like the plan.

## The combinations, and which cannot be built

Which answer of one question fixes another,
and which combinations cannot be built.
Every reason below is a run this round made,
or a line it read,
and each says which.

1.  Question 1,
    option A,
    without the exit code's move:
    buildable,
    and its stated delivery does not happen.
    `subvolume_command.rs` names `invocation` twice,
    for `Action` (line 31) and for
    `ENGINE_FAILURE_EXIT_CODE` (lines 30 to 33,
    used at 240),
    and the linter keeps a `uses` line for as long as any name of the folder is used,
    which the two after-stop linter runs showed from the other side.
    So the two allow-list lines die only with that move,
    which option A's "leaves undecided" names.
    Reason:
    read,
    with the linter's own behaviour behind it.
2.  Question 2,
    either option,
    with a lint-clean round:
    cannot be built.
    Both add `transaction -> lock` and `repo_read -> lock`,
    and the linter gives each of those two a finding twice,
    one of them with no key,
    so no line of the allow list can record them (measured:
    12 findings before,
    16 after,
    the same with either drawing).
    Reason:
    the linter.
3.  Question 2,
    option B,
    with the awaited face unchanged:
    cannot be built.
    `package/git-policy/cli/src/native/transaction/transaction_git_awaiting.rs:291` prints
    `error[E0308]: mismatched types: expected \`Result<Output,
    Error>\`,
    found \`Result<WatchedOutput,
    Error>\``.
    Reason:
    the compiler.
4.  Question 3,
    option C,
    with a record that cannot be defaulted:
    cannot be built.
    `error[E0277]: the trait bound \`CurrentStepRecord:
    std::default::Default\` is not satisfied`
    at `package/git-policy/cli/src/native/transaction/transaction_git_request_and_reply_types.rs:170`.
    Reason:
    the compiler.
5.  Question 3,
    option A:
    buildable,
    and every `GitContext` initializer must answer.
    Adding the field printed `E0063` for 63 initializers (22 release,
    41 control),
    and the record must also be `Eq` and `PartialEq`
    (`error[E0277] ... \`CurrentStepRecord:
    std::cmp::Eq\` is not satisfied`,
    `error[E0369]`).
    Reason:
    the compiler.
6.  Question 4,
    option A,
    with the attributes path unchanged (question 5,
    option A):
    buildable,
    and "the reader never starts Git" is not delivered for conversions,
    because the conversion builder asks `system_attributes_for` at
    `package/git-policy/cli/src/native/worktree/worktree_clean_content_attribute_sources.rs:194`.
    Reason:
    read.
7.  Question 5,
    option B,
    in the same file as question 4's change:
    buildable,
    and the two are different types,
    not one parameter twice:
    ten `ScopeEnvironment` builders against one `ConversionEnvironment` builder
    (`subvolume/subvolume_platform.rs:133`).
    Reason:
    measured by `rg`,
    quoted in section 2.
8.  Questions 1 and 2,
    landed apart:
    reachable,
    and they share no file.
    `subvolume_command.rs` and the three call sites of the five one-shot functions are disjoint.
    Reason:
    read.
9.  Questions 2 and 3,
    landed apart:
    not reachable.
    Both edit `transaction/transaction_git.rs`,
    and both edit the two watched-run files.
    Reason:
    read.

## Questions for the owner

Each question is one decision and each is answerable alone.
The recommendation is first,
with its mechanism and its cost,
and each option says what a yes would leave undecided.
The letters are the same letters the sections use.

### Question 1: where the finished value lives

The owner's decision of 2026-11-04 was
"they return a small \"finished with this code and output\" value from a low file;
only the top level builds `Action`" (`doc/planning/cli-git-source-layout.md:470-471`),
and the target tree puts the file in the program folder `starting_git_and_describing_a_run`
(`doc/planning/cli-git-source-layout-target-tree.md:6436`).
Both placements are the project's own words;
they disagree about what the round delivers.

Option A (Recommended):
`Finished` in `foundation`,
the plan's prose.
How it works:
one struct of an exit status and two byte streams,
built by `saying`,
`explaining` and `quietly`;
`migrate_action`,
`convert_action` and `doctor_action_with_section` return it;
`management` turns it into `Action::ExitBytes`,
which is the one place a report is written.
What it changes for good:
with `ENGINE_FAILURE_EXIT_CODE` moved in the same branch,
`uses src/native/subvolume -> src/native/invocation` (`folder-rule.txt:943`)
and `names-group src/native/subvolume src/native/invocation` (line 538) die;
at step 7 `what_happens_when_you_type_a_command` leaves three folders' `uses/` lists.
Example:
`git monochromatic-git-policy-cli migrate-to-subvolume --help`
builds `Finished::saying(0, MIGRATE_HELP)`,
prints the same help,
and exits 0.
What it gives up:
the type moves again at step 7,
and the exit code's move puts 18 files on the same branch or in a second commit.
A yes leaves undecided:
whether the exit code moves (in this round,
in its own commit,
or not),
and whether the four published names narrow to `pub(crate)` or keep a `pub` face.

Option B:
`Finished` and `FinishedBytes` in `invocation`,
beside `action.rs`,
the target tree's signature.
How it works:
two structs,
text streams for the two text commands and bytes for the doctor;
nothing in the allow list changes.
What it changes for good:
nothing this round;
at step 7 the decision folder leaves `converting_the_main_checkout`,
and `checking_that_your_setup_works` and `converting_a_linked_worktree` gain a use of the program folder.
Example:
the same command,
the same help,
with the value built as `invocation::Finished`.
What it gives up:
this round delivers nothing to the allow list,
and two folder uses are created at step 7 where A creates none.
A yes leaves undecided:
the same two items as A.

Ranking:
A > B,
because A removes two lines now and three names at step 7
where B removes none now and leaves two folders gaining a use;
B's advantage is that the value sits beside `Action`,
and its diff is smaller,
which is effort.

### Question 2: who sweeps the locks a stopped step left

Today the watcher's own task builds the ledger and sweeps on the blocking pool,
and the sweep also happens when a watch is dropped while a stop is under way.
The plan's prose says "its caller hands it what to do after a stop"
(`doc/planning/cli-git-source-layout.md:458`);
the target tree's signature says the watcher loses the ledger and the caller sweeps.

Option A (Recommended):
the watcher calls back.
How it works:
the library defines `trait AfterTheStop: Send + Sync`;
the caller builds its ledger before the start and hands an `Arc` of a small struct implementing it;
the watcher runs it inside its task,
on the blocking pool,
where it sweeps today.
What it changes for good:
`clearing_leftover_reference_locks` leaves the watcher library's `uses/` entry at step 7;
the library exports one trait whose only implementations are the program's;
`start_with_locks` leaves the source guard's published list (`items_nameable_from_outside_release.txt:6113`).
Example:
a landing's `git update-ref` is stopped;
the watch ends the child,
the wait after the kill passes,
and the watcher's task sweeps the two stale `refs/heads/*.lock` files it can prove stale,
leaving one it cannot.
A watch dropped 40 seconds into the same stop still sweeps.
What it gives up:
a behaviour crosses the crate boundary,
which the plan's note disfavours for these changes
(`doc/planning/cli-git-source-layout-target-tree.md:6427-6429`),
and each call site builds one small value.
A yes leaves undecided:
whether the same callback carries anything else later.
Compiled:
yes,
`Finished \`dev\` profile ... in 33.89s`.

Option B:
the caller sweeps.
How it works:
the five one-shot functions answer `WatchedOutput { output, ended_by_the_watch }`;
the caller builds the ledger before the start,
reads the flag,
and sweeps,
on the blocking pool where it is a task.
What it changes for good:
the same library entry,
and the sweep's moment moves from the watcher's ending to the caller's reading.
Example:
the same stopped step;
the synchronous face sweeps inline,
the awaited face on the blocking pool,
and a caller that drops its watch during the stop and returns sweeps nothing,
so the lock waits for the next command's stale-lock judgement.
What it gives up:
the stop-under-way window's sweep,
and 18 control lines in three files read the new type.
A yes leaves undecided:
whether the five convenience functions keep answering `WatchedOutput` or map it away.
Compiled:
yes,
`Finished \`dev\` profile ... in 27.89s`; with the awaited face unchanged it prints `E0308`.

Both options add `transaction -> lock` and `repo_read -> lock`,
which no allow-list line can record
(measured:
12 findings before,
16 after).
A third placement,
the ledger built and swept by a `push/` function,
adds neither and is not drawn here;
it is named in section 3 and needs its own linter run.

Ranking:
A > B,
because both deliver the same library entry,
and A keeps the sweep in the window where it happens today while B loses it there;
B's advantage is a plain value instead of a trait object,
and its shape is the one the target tree asks for,
which is a cost of A's shape and not a reason to rank A lower.

### Question 3: how the current step record is carried

Today it is carried by the task,
in a `tokio::task_local!`
(`landing/landing_state_carried_by_its_task.rs:47`),
which is what the owner's choice 4 asked for.
The target tree says a thread-local record in a program file,
and the plan's prose says the basic tools.

Option A (Recommended):
on the run's `GitContext`.
How it works:
`GitContext` gains a `CurrentStepRecord` (a folder it writes into and the detail of the next write);
the landing builds its context with `in_the_landing_at(folder)` and every other run with `nothing()`;
`step_bound` asks the context,
and `step_about_to_start` keeps today's guard,
entering the step only when the bound is `Unbounded`.
What it changes for good:
the record becomes a name the folder rule can see,
and no build site can be forgotten,
because `GitContext` has no `Default`.
Example:
the landing's `git update-ref` gets `StepBound::Unbounded` from the landing's own context,
the step file is written,
and the client that reads it sees the step while it runs;
a command outside the service builds `nothing()`,
gets `leave_running_bound`,
and writes no file.
What it gives up:
a field at 63 `GitContext` initializers (22 release,
41 control),
each refused by the compiler until it answers,
and `Eq`/`PartialEq` on the record.
A yes leaves undecided:
where the record's file writer lives,
and how the watcher's answer reaches its two call sites.

Option B:
task-carried,
moved to the program file the target tree names.
How it works:
today's `task_local!` and its writer move to `current_step_record` in
`how_long_the_program_waits_for_git_steps`;
`LandingBoard` stays in `service_busy`;
the watcher's answer rides a new `WatchTerms` field.
What it changes for good:
the same library line,
and nothing to answer at any build site.
Example:
the same landing step,
answered from the task,
with the step file written exactly as today.
What it gives up:
the use stays invisible to the folder rule,
so the tangle can regrow through a name;
20 `WatchTerms {` literals in seven control files and six `for_class` call sites take the new answer.
A yes leaves undecided:
the same items as A,
and whether the state becomes a value later.

Option C,
not offered:
a field on `GitRequest`.
It compiles at all 27 request builders with the unsafe answer,
and cannot be made to refuse it
(`E0277` at `package/git-policy/cli/src/native/transaction/transaction_git_request_and_reply_types.rs:170`).

Ranking:
A > B > C,
because A makes the use visible and un-forgettable,
B makes it un-forgettable but invisible,
and C makes it visible but silently wrong for a landing step.

### Question 4: how far the system settings path is threaded

The plan's prose says "the caller passes the path in,
so a reader never starts Git"
(`doc/planning/cli-git-source-layout.md:462-463`).
Today the reader asks,
inside three gates,
and the ask can start real Git.

Option A (Recommended):
every caller that builds a scope passes the path,
the three reader builders included.
How it works:
`ScopeEnvironment::from_variables(variables, system_default)`;
the ten builders pass `system_config_for(variables)` (or `None` where a gate says the file is not read);
the reader stops calling the asker.
What it changes for good:
at step 7 `reading_git_settings_from_files` drops the asker from its `uses/` line
(target-tree-files:1938) and `answering_questions_without_starting_git` drops it from 605;
`finding_the_git_folders` gains it at 1038,
which does not name it today.
Example:
`git config --get core.hooksPath` builds a scope in `repo_read_layout.rs`;
the builder passes `Some("/etc/gitconfig")` learned from the forward-target Git,
and the reader resolves without starting one.
What it gives up:
the three reader builders now start Git in folders named for not doing so,
which is the cost of the reader's line.
A yes leaves undecided:
what a builder passes when it cannot learn a path.

Option B:
the ten builders pass the path and the three reader files go on asking.
How it works:
the same parameter;
`repo_read_query.rs`,
`repo_read_hook.rs` and `repo_read_layout.rs` call the asker themselves.
What it changes for good:
the reader's line dies at step 7,
and nothing else changes in the folder lists.
Example:
the same command,
the same value,
asked in a different file.
What it gives up:
two of the three delivered `uses/` entries,
and the promise in `answering_questions_without_starting_git`'s name.
A yes leaves undecided:
the same item as A.

Ranking:
A > B,
because A removes two entries and adds one where B removes one and adds none;
B's advantage is three fewer lines in this round,
which is effort.

### Question 5: whether the attributes path changes in this round

`system_attributes_for` is the same shape as the settings path and the plan does not name it
(`repo_read/git_system_config.rs:236`,
asked at `package/git-policy/cli/src/native/worktree/worktree_clean_content_attribute_sources.rs:194`).

Option A (Recommended):
leave it asking in this round.
How it works:
nothing changes here.
What it changes for good:
nothing.
Example:
`git monochromatic-git-policy-cli` converting a clone still asks the attributes file,
and the person sees no difference.
What it gives up:
"the reader never starts Git" is not delivered for conversions.
A yes leaves undecided:
which step takes it.

Option B:
change it in the same round.
How it works:
the same parameter on `ConversionEnvironment::from_variables`,
with the ask moved to its caller (`subvolume/subvolume_platform.rs:133`).
What it changes for good:
the conversion builder stops starting Git for both of its questions,
which completes this change's promise where the plan's items reach.
Example:
the same conversion,
with the attributes path learned before the builder runs.
What it gives up:
a second parameter and a wider diff,
and the plan has no signature for it,
so this file would be its only statement.
A yes leaves undecided:
whether the parameter is one attributes value or an optional path and a kind.

Ranking:
A > B,
because A keeps the round to the five changes the plan records,
and B changes a call the plan does not name with no record behind it beyond this file;
B's advantage is the completed promise,
which A leaves to a later round.

## What was measured, what was read, and what is inferred

Measured in this worktree at `b6fc11da64`,
each with the command that printed it:

- `ENGINE_FAILURE_EXIT_CODE`:
  52 release lines in 18 files in 9 folders
  (`rg -n ... | wc -l`,
  `rg -l ... | wc -l`,
  `rg -l ... | xargs -n1 dirname | sort -u | wc -l`).
- The ten `ScopeEnvironment` builders
  (`rg -n "ScopeEnvironment::from_variables" -g '*.rs' -g '!*_tests.rs'`),
  and seven release files of `repo_read/` naming `git_system_config`.
- The 27 release functions that return a `GitRequest`,
  and their five folders
  (`rg -n -- "-> GitRequest" ...`).
- The 63 `GitContext` initializers (22 release,
  41 control),
  by the compiler's `E0063` count.
- The five one-shot functions' call sites:
  3 release,
  and 18 control lines the compiler refused
  with the answer type changed.
- `WatchTerms {` literals in controls:
  20;
  `for_class` call lines:
  8,
  of which one is the definition
  and one the internal `Self::for_class`.
- `GitRequest`:
  503 lines of 143 files,
  122 of them release files.
- The allow-list pairs:
  `subvolume -> invocation` at 943 and its `names-group` at 538;
  `subvolume -> foundation` at 942;
  `merge -> invocation` at 820;
  `lock -> transaction` at 805;
  and no `transaction -> lock` line at all.
- The `rust/folder-uses-links` findings:
  12 on the unedited tree,
  16 with either after-stop drawing.
- Four `cargo check --locked --offline --all-targets` runs and three linter runs,
  named in "How every claim here was checked".

Read,
and not measured:

- The window in which a dropped watch sweeps today (section 3).
- The `merge -> invocation` line's survival (section 5).
- Which target folder each source file lands in,
  from the target-tree rows.
- That `Action` is the only other name `subvolume_command.rs` takes from `invocation` besides the exit code.

Inferred,
and said so where it is used:

- How often a landing's or a reader's step is in the stop-under-way window.
  Nothing measured it.
- What the five run-this-next signatures will look like when they hold together;
  nothing compiled them.
- That the exit code can leave `action.rs` without a loop.
  The plan's own item list says it moves down;
  this round did not build it.

## What this proposal does not settle

- `doctor_action`'s future:
  it has no release caller and is published
  (`items_nameable_from_outside_release.txt:2587`),
  so the round either rewrites it with the others or deletes it and its line.
- Which red `rust/folder-uses-links` findings `main` prints today
  (`doc/handover/cli-git-rust-implementation.md` keeps that list);
  this file's line numbers are of `b6fc11da64`,
  and a line added or deleted before the round starts moves them.
- The plan's two further call changes found by the moved-item check
  (`copy_file_exclusive`'s error type and the hint functions of `event_hints`) belong to step 5,
  not this round.
- The owner's other open points for the split (target tree,
  "Open points for the owner") hold steps 7 and after,
  except that reading D or E of the library count decides whether `starting-and-watching-programs` is a library at all.
  Both readings keep it,
  so the after-stop and step-record changes are needed under either.
