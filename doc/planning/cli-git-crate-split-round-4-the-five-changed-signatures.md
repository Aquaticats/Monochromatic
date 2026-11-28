# The crate split's fourth round: the five changed signatures, proposed for the owner

Status:
unapproved and non-final.
Prepared by a Pi run on 2026-11-28,
which read `main` at `4cf7cc74b8` (`git log --format=%cI`:
2026-11-28T02:37:45-05:00)
and gathered its measurements from 2026-11-28T03:04:30-05:00.
No code changed on this branch.
The gate reader has not read it,
and the owner has not answered any question it asks.

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
For each of the five call changes it states where the call is on `main` today,
what the plan says the call becomes,
at least two signatures that would do it,
what each delivers for the split and costs,
whether it is feasible and what was run to find out,
and a ranking.
It then states the order the five can land in,
which of them touch one file,
and the questions only the owner can answer.

## Which record governs where they disagree

The question this proposal answers is
"Open:
does the crate split's step 4 run now?"
(`doc/handover/questions-for-the-owner.md`),
which names the five as finished results,
system settings path,
after-stop action,
current step record and run this next.

The plan is `doc/planning/cli-git-source-layout.md`
(whose section "Splitting into crates (owner,
2026-11-04)" is the owner's decision the split serves)
and `doc/planning/cli-git-source-layout-target-tree.md`,
whose section "The call changes,
as signatures" is the draft nobody but the plan has stated.

Where they disagree this record follows the decision record first,
the plan second and the handover entry last,
as this branch's brief requires.
The disagreements,
each stated where it bites:

- The handover makes run this next a single call change in `stacked_merge_push`.
  The plan's run this next replaces two in-process recursions,
  `entry::plan_invocation` called from `stacked_merge_push`
  and `entry::run_process` called from `merge_second_pass_finish`
  (`cli-git-source-layout-target-tree.md` lines 6385-6405).
  The plan governs:
  this proposal describes the whole call change,
  with the merge half marked as the wider risk.
- The plan's prose for commands that only finish asks for
  "a small 'finished with this code and output' value from a low file"
  (`cli-git-source-layout.md`,
  "Changes to how code is called"),
  while its own signatures put the new file `what_a_command_hands_back`
  in the program folder `starting_git_and_describing_a_run`
  (`cli-git-source-layout-target-tree.md` line 6387),
  which is not a low file.
  The two placements deliver different things and both are options below:
  only the low one removes a line of the folder rule's allow list today,
  and only the plan's one needs no second move.
- The plan says `ScopeEnvironment` "gains the field `system_config`"
  (line 6430),
  but the struct already has that fact as `system_default`
  (`package/git-policy/cli/src/native/repo_read/repo_read_config_scan.rs:214`),
  filled at line 296.
  The change is a parameter,
  not a field,
  and the field's meaning today (unknown,
  which makes the scope unresolvable) is the one a parameter must keep.
- The plan says the line that calls `LockLedger::before_start(command)` "in `child_watch`" moves out
  (line 6421).
  On `main` there are three such lines,
  in `child_watch_one_shot_run.rs:238` and `child_watch_output_watched_on_the_runtime.rs:243`
  beside the one `child_watch` itself would have had,
  because round 2 of the split cut them out of `child_watch.rs` after the plan was written.
  It also does not name the sweep inside `child_watch_clock_marks_and_stopping::stop_process`,
  which is the half the change has to move.
- The plan names `transaction_git` alone for the current step record (line 6407),
  but `on_landing_thread` is also called by two watched-run files that the target tree sends to a library
  (`package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs:230`,
  `child_watch_output_watched_on_the_runtime.rs:235`),
  and the library's own `uses/` line names the folder those files would name
  (`doc/planning/cli-git-source-layout-target-tree-files.md:322`).
  A change that stops at `transaction_git` delivers nothing to that line.

## How the call sites were found, and what moved since 2026-11-27

Every call site below was found with `git grep` on this worktree's copy of `main`,
whose files are the commit `4cf7cc74b8` names.
The handover entry's list is from 2026-11-27T06:40:00-05:00,
and `main` took merges since;
what moved is stated beside each call change.

Two measurements cover all five call changes:

- The folder rule's own linter,
  run over `package/git-policy/cli` at `4cf7cc74b8`,
  reports today's findings:
  eleven `rust/folder-uses-links` findings (each a use that joined a loop the allow list already tolerates,
  the tangle the decision of 2026-11-28 "Merge red,
  untangle later" records),
  fourteen `rust/max-lines` findings and one skipped `rust/avoidable-closure`,
  which is twenty-six findings in all.
- A throwaway branch whose only change was emptying the `uses` lines of
  `package/git-policy/cli/layout-allow-list/folder-rule.txt`
  made the linter report the crate as one loop,
  with one proof per folder pair:
  five hundred and sixty-three `uses` lines.
  It was discarded.

A line of `folder-rule.txt` that a change makes dead is reported by name
("The allow-list entry `uses src/native/subvolume -> src/native/invocation` ... tolerates nothing any more"),
so the dead lines each change delivers were measured,
not reasoned about:
the branch of each change was run through the same linter,
and the run that answered this for the first call change is quoted where it lands.

Each Rust block below that rewrites one call site is an excerpt of a function body,
not a whole file:
it shows the lines the change lands on,
and a block that stands alone is labelled with the file it would become.
The complete list of what the linter checks is in
`package/linter/monochromatic-lint/src/layout_folder_links_tolerated_uses.rs`
and `layout_allow_list_dead_entry_findings.rs`;
a line stays only while its use is still made.

## 1. Commands that only finish

### Where the call is today

- `package/git-policy/cli/src/native/subvolume/subvolume_command.rs:353`:
  `pub fn migrate_action(arguments: &MigrateArguments, cwd: &Path) -> Action`,
  which returns `Action::Exit { code: 0, stdout: String::from(MIGRATE_HELP), stderr: String::new() }`
  for `--help` (line 358),
  `Action::Exit` with the refusal message (line 367),
  and `Action::Exit { code, stdout: String::new(), stderr: String::new() }` for the run (line 383).
- `package/git-policy/cli/src/native/health/doctor.rs:607`:
  `pub async fn doctor_action_with_section(context: &GitContext, cwd: &Path, home: Option<&Path>, section: &dyn Fn(...)) -> Action`,
  which returns `Action::Exit` (line 662) and `Action::ExitBytes` (lines 676 and 698);
  `doctor_failure(message: &str) -> Action` at line 718.
  Its synchronous face `doctor_action` (line 441) has no release caller:
  the only caller is `invocation/management.rs:527`,
  which calls the async one.
- `package/git-policy/cli/src/native/worktree/worktree_convert_command.rs:147`:
  `pub fn convert_action(arguments: &ConvertArguments, cwd: &Path, environment: &[(OsString, OsString)], real_git: Option<&Path>) -> Action`,
  returning `Action::Exit` for `--help` (line 154) and for the run (line 171).
- `package/git-policy/cli/src/native/push/stacked_merge_push.rs:226`:
  `let action: Action = plan_invocation(arguments.as_slice(), environment.as_slice(), &inputs);`
  then `return Ok(run_planned(action));` (line 227),
  and `fn run_planned(action: Action) -> i32` at line 119 matches the planned `Action`'s variants.
- The consumers that would build the `Action` instead:
  `invocation/management.rs:479` (`return migrate_action(arguments, cwd);`),
  `:385` (`return crate::worktree_convert_command::convert_action(arguments, cwd, environment, real_git);`),
  and `:527` (`let action: Action = match run_on_runtime(doctor_action_with_section(`).

What moved since 2026-11-27:
nothing in the call.
The plan's fourth name,
`stacked_merge_push`,
has nothing to do here today:
its only `Action` is the planner's,
so the change that removes it is run this next,
not this one.

### What the plan says it becomes

"Commands that only finish:
`subvolume_command`,
`doctor`,
`worktree_convert_command` and `stacked_merge_push` return
`Finished` and only the program crate's `entry` and `management` files build `Action`"
(`doc/planning/cli-git-source-layout-target-tree.md` lines 6433-6437).
The prose version adds "from a low file"
(`doc/planning/cli-git-source-layout.md`,
"Changes to how code is called").

Which folder stops depending on which,
in the target tree:
`converting_the_main_checkout` (`cli-git-source-layout-target-tree-files.md:853`),
`converting_a_linked_worktree` (`:828`) and `checking_that_your_setup_works` (`:731`)
each name `what_happens_when_you_type_a_command` in their `uses/` lists today,
and `action.rs` is the file that goes there
(`the_decision_one_command_ends_in.rs`,
`:3700`,
in the folder at `:3649`).
The change takes that name out of all three.

Which line of `layout-allow-list/folder-rule.txt` it removes today:
measured,
`uses src/native/subvolume -> src/native/invocation` (line 943)
and `names-group src/native/subvolume src/native/invocation` (line 538),
both of which the linter named as tolerating nothing once the `Action` uses and the import were gone.
`health` and `worktree` keep using `invocation` for other files,
so their lines stay:
`health/traditional_hook_warning.rs:32` names `crate::hooks_off::NO_HOOKS_DIRECTORY`,
and `worktree` names `crate::real_git`,
`crate::real_git_candidate`,
`crate::global_arguments`,
`crate::child_environment`,
`crate::forwarding` and `crate::escape_hatch`
across eight of its files.

### Signature A: one `Finished` in a low file, bytes for both streams

New file `package/git-policy/cli/src/native/foundation/a_finished_command.rs`:

```rust
//! package/git-policy/cli/src/native/foundation/a_finished_command.rs
/// What a command that only finishes hands back:
/// the exit status and the bytes of each stream.
pub struct Finished {
    /// The status the process ends with.
    pub code: i32,
    /// What the command answers,
    ///  as bytes because a report may name a path that is not text.
    pub standard_output: Vec<u8>,
    /// What it explains,
    ///  as bytes for the same reason.
    pub standard_error: Vec<u8>,
}

/// What a command's answer is built from.
impl Finished {
    /// A command that printed `text` and ended with `code`.
    pub fn saying(code: i32, text: &str) -> Finished {
        return Finished {
            code,
            standard_output: text.as_bytes().to_vec(),
            standard_error: Vec::new(),
        };
    }

    /// A command whose progress went to standard error as it ran and that ended with `code`.
    pub fn quietly(code: i32) -> Finished {
        return Finished {
            code,
            standard_output: Vec::new(),
            standard_error: Vec::new(),
        };
    }
}
```

`migrate_action`,
`convert_action` and `doctor_action_with_section` return it
(in `subvolume_command.rs`):

```rust
//! package/git-policy/cli/src/native/subvolume/subvolume_command.rs
/// The migration's answer.
pub fn migrate_action(
    arguments: &MigrateArguments,
    cwd: &Path,
) -> Finished {
    if arguments.help {
        return Finished::saying(0, MIGRATE_HELP);
    }
    let root: PathBuf = match main_checkout_root(cwd) {
        Ok(found) => found,
        Err(message) => {
            return Finished::saying(
                REFUSED_EXIT_CODE,
                format!("git monochromatic-git-policy-cli migrate-to-subvolume: refused: {message}\n").as_str(),
            );
        }
    };
    let code: i32 = execute(&LinuxPlatform::default(), arguments, root.as_path(), &Interrupt::never(), &mut say_on_standard_error);
    return Finished::quietly(code);
}
```

and one call site rewritten,
in `invocation/management.rs` where the `Action` is built now:

```rust
//! package/git-policy/cli/src/native/invocation/management.rs
let finished: Finished = migrate_action(arguments, cwd);
return Action::ExitBytes {
    code: finished.code,
    stdout: finished.standard_output,
    stderr: finished.standard_error,
};
```

What it delivers:
the two lines above,
`uses src/native/subvolume -> src/native/invocation` and
`names-group src/native/subvolume src/native/invocation`,
measured dead by the linter.
`subvolume` keeps its use of `foundation` (line 425 of the allow list already tolerates it).

What it costs a caller:
one struct to build from the value,
and the `Action::Exit` arm in `entry.rs` stays the single place that writes a report,
so the ordering the wrapper has today (plan,
then write) is unchanged.

Which tests change:
`subvolume_command_tests.rs:207,218,254`,
`worktree_convert_command_tests.rs:294`,
`doctor_tests.rs` (twenty-four calls through `exit_parts`),
`doctor_fsmonitor_tests.rs:54`,
and `management`'s own tests read `Finished` instead of `Action`.

What it leaves undone:
`stacked_merge_push`,
which run this next covers;
and the two uses of `ENGINE_FAILURE_EXIT_CODE`
(`subvolume_command.rs:237`,
`doctor.rs:717`),
which the same change must move down with the type,
as the plan's own item move already asks
(`cli-git-source-layout.md`,
"Items that move down":
"`ENGINE_FAILURE_EXIT_CODE`").
It also puts the type in `foundation`,
which the rescan splits among three libraries and six program folders
(`cli-git-source-layout-target-tree.md:819-824`),
so step 7 moves the file again.

### Signature B: the plan's own, beside `Action`

New file `package/git-policy/cli/src/native/invocation/what_a_command_hands_back.rs`:

```rust
//! package/git-policy/cli/src/native/invocation/what_a_command_hands_back.rs
/// What a command that only finishes hands back.
pub struct Finished {
    /// The status the process ends with.
    pub code: i32,
    /// What the command answers.
    pub stdout: String,
    /// What it explains.
    pub stderr: String,
}

/// The same,
///  for a command whose answer may name a path that is not text.
pub struct FinishedBytes {
    /// The exit status.
    pub code: i32,
    /// What the command answers.
    pub stdout: Vec<u8>,
    /// What it explains.
    pub stderr: Vec<u8>,
}
```

`migrate_action` returns `Finished`,
`convert_action` returns `Finished`,
`doctor_action_with_section` returns `FinishedBytes`,
and `management.rs` builds the `Action` as in signature A.

What it delivers:
nothing to `folder-rule.txt` today:
the type sits in the same folder as `Action`,
so `subvolume`,
`health` and `worktree` go on using `invocation`,
and the linter reports no dead line.
It delivers at step 7,
where the three folders' `uses/` lists lose `what_happens_when_you_type_a_command`
and gain nothing,
because each already names `starting_git_and_describing_a_run`.

What it costs a caller:
one struct to build from the value,
and two types for one idea,
or one bytes type that every text caller converts with `into_bytes`.

Which tests change:
the same tests as signature A.

What it leaves undone:
the same two `ENGINE_FAILURE_EXIT_CODE` uses,
and the whole delivery waits for step 7.

### Signature C: no value at all, the command writes and returns the status

```rust
//! package/git-policy/cli/src/native/subvolume/subvolume_command.rs
/// The migration's answer.
pub fn migrate_action(
    arguments: &MigrateArguments,
    cwd: &Path,
    say: &mut dyn std::io::Write,
) -> i32 {
    if arguments.help {
        let _written = say.write_all(MIGRATE_HELP.as_bytes());
        return 0;
    }
    // ... the refusal is written to `say` instead of being carried ...
    return execute(&LinuxPlatform::default(), arguments, root.as_path(), &Interrupt::never(), &mut say_on_standard_error);
}
```

and the call site:

```rust
//! invocation/management.rs
return Action::Exit {
    code: migrate_action(arguments, cwd, &mut std::io::stdout()),
    stdout: String::new(),
    stderr: String::new(),
};
```

What it delivers:
the same two dead lines as signature A,
measured by the same probe,
with no type to move later.

What it costs a caller:
the wrapper loses the one place that writes a command's answer after planning
(`entry.rs`'s `write_exit`),
so the answer is written from inside the command,
and `doctor`'s report leaves before the `Action` that carries it exists.
No recheck sits between the two today (`run_process` rechecks only in its `Forward` arm),
so the visible behaviour is the same,
but a later recheck that covers exits would not see these commands.
Every test that reads `Action::Exit { stdout }` must read a writer's bytes instead.

Which tests change:
as signature A,
and every test that captures the `Action` becomes a test that captures a `Vec<u8>`.

What it leaves undone:
`stacked_merge_push`,
as above,
and the single-write-point invariant.

### Whether it is feasible

Feasible on the reading,
not tried with a compiler:
the probe that measured the dead lines also replaced the four `Action` uses with a bare
`Finished` type and a local constant on a throwaway branch,
which is what the linter needed and all a parser sees.
The compiler question is a plain type move,
and the one part in doubt,
whether `ENGINE_FAILURE_EXIT_CODE` can leave `action.rs` without a cycle,
is settled by the plan's own item move to the basic tools,
which `foundation` is today.

### Ranking

A > B > C.

A over B because A removes two lines of the allow list now,
which is the round's work,
and the move it costs at step 7 is one file move inside a round that is already moving a hundred;
B removes nothing until step 7,
so a branch that lands B leaves the loop `subvolume` and `invocation` form standing.
B over C because B keeps the wrapper's one place that writes a report and the value a test can read,
where C trades the same two lines for losing both.

## 2. System settings path

### Where the call is today

- `package/git-policy/cli/src/native/repo_read/repo_read_config_scan.rs:296`:
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
  all eleven of them:
  `index/index_reset_file_surroundings.rs:114`,
  `repo_read/repo_read_query.rs:90`,
  `repo_read/repo_read_layout.rs:264,310,353,411`,
  `repo_read/repo_read_hook.rs:105`,
  `push/auto_push_trigger_ahead_of_upstream.rs:113`,
  `merge/merge_second_pass_tree_attributes.rs:84`,
  `worktree/worktree_clean_content_attribute_sources.rs:93`,
  and `subvolume/subvolume_platform.rs:133`.

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

"System settings path:
`ScopeEnvironment` in `repo_read_config_scan` gains the field `system_config: Option<PathBuf>`
and stops calling `git_system_config::system_config_for`;
the callers that build a `ScopeEnvironment` pass the result"
(`doc/planning/cli-git-source-layout-target-tree.md` lines 6429-6432).

Which folder stops depending on which:
none today.
The asker and the reader are two files of one folder,
and no line of `folder-rule.txt` names a pair between them.
In the target tree the asker is a file of its own,
`system_settings_file_of_git.rs` in `starting_git_and_describing_a_run`
(`cli-git-source-layout-target-tree-files.md:2602,2589`),
and the reader is `configuration_scopes_scan.rs` in `reading_git_settings_from_files`
(`:1943,1936`).
The target tree keeps the folder edge either way:
`answering_questions_without_starting_git` names `starting_git_and_describing_a_run`
in its `uses/` list
(`cli-git-source-layout-target-tree.md:3386-3396`).
So what this change delivers is not a folder line but the promise in that folder's name:
after it,
the files that answer without starting Git do not start Git,
and the ask sits with a file that may.
It delivers that only for the files that are handed a value.

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
            // ... every field as today, with `system_default` taken from the parameter ...
            system_default,
            ..ScopeEnvironment::default()
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

and one call site rewritten:

```rust
//! package/git-policy/cli/src/native/repo_read/repo_read_layout.rs
let scope: ScopeEnvironment = ScopeEnvironment::from_variables(
    variables.as_slice(),
    crate::git_system_config::system_config_for(variables.as_slice()),
);
```

What it delivers:
the reader's file stops starting Git,
and the ask moves to the eleven call sites.
No allow-list line changes.
At step 7 the reader's folder can stop naming the asker only for the callers that pass a value;
the three reader files that build their own scope (`repo_read_query.rs:90`,
`repo_read_hook.rs:105`,
`repo_read_layout.rs:264`) go on asking from inside that folder.

What it costs a caller:
one line each,
and a caller whose variables name no real Git passes `None`,
which is the answer it gets today from the same variables,
so no read changes its answer.

Which tests change:
`repo_read_config_scan_tests.rs` (six calls),
`git_system_config_tests.rs:273-294` (builds scopes and sets `system_default`).

What it leaves undone:
`system_attributes_for`,
the same shape for `$(prefix)/etc/gitattributes`,
imported at `worktree/worktree_clean_content_attribute_sources.rs:18` and asked at line 194,
and the reader files that build their own scope.

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

and one call site:

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

and inside `from_variables`,
the call at line 296 becomes
`scope.system_default = crate::git_system_config::the_system_config_remembered();`,
with `remember_the_system_config_for_this_git` called once where the invocation knows its real Git
(the `Ok(inputs)` arm of `run_process`,
beside the resolution,
and once at the service's start).

What it delivers:
no call site changes,
so no caller gains an upward use,
and the reader's folder names no asker at all,
which takes `starting_git_and_describing_a_run` out of
`answering_questions_without_starting_git`'s `uses/` list at step 7.

What it costs a caller:
nothing at the call site,
and everything at the edges:
a process-wide value is state the compiler cannot see,
and a Git replaced while a long-lived service runs is not noticed,
where today's `learned_path` re-reads the executable's identity on every ask.
The repository has one precedent,
`invocation/git_version_cache.rs`.

Which tests change:
none of the eleven call sites,
but every test that expects a scope to name a system file must learn one first,
and a test that does not now sees a scope that declines,
which is what the existing controls already expect of an unknown system file.

What it leaves undone:
`system_attributes_for`,
unless it is remembered the same way.

### Whether it is feasible

Feasible on the reading,
not tried with a compiler,
because the cheap compiler answer (every caller passes `None`) would compile
and would prove nothing about the question:
what each caller can pass.
That was answered by reading the callers:
every one of the eleven holds the variables the ask reads
(`with_real_git` documents that the forward-target variable is in every child's environment,
and `repo_read_query.rs:90` and `subvolume_platform.rs:133` build a scope from the process's variables,
which carry no forward-target variable,
so they get `None` today as well).

### Ranking

C > A > B.

C over A because C is the only one that takes the ask out of the reader's folder entirely,
which is the promise `answering_questions_without_starting_git` makes,
and it does it without adding a use to any caller;
A leaves three reader files asking from inside that folder.
A over B because A is one fact in one place and B's second type buys a guarantee
that no code today can break without the compiler saying so anyway.

The price of C is stated above and is the one thing the owner has to weigh:
a Git replaced mid-process is not noticed.

## 3. After-stop action

### Where the call is today

- `package/git-policy/cli/src/native/push/child_watch.rs:195`:
  `return Self::start_with_locks(waiting_for, pid, terms, LockLedger::none());`,
  the body of `start`;
  `start_with_locks(waiting_for, pid, terms, locks: LockLedger)` at line 206;
  `start_under_stop(waiting_for, pid, terms, locks: LockLedger, stop: Option<StopArm>)` at line 227;
  `start_when_a_task_may_run(..., locks: LockLedger, ...)` at line 260;
  and `watch_loop(incoming, flags, subject, pid, terms, locks, stop, root_pin)` at line 302.
- `package/git-policy/cli/src/native/push/child_watch_task_loop_and_notice.rs:76`:
  `locks: LockLedger,` as a parameter of `watch_loop`,
  passed to `stop_process(pid, &locks, &mut incoming, &root_pin)` at lines 162 and 180,
  and to `end_for_the_stop(..., locks: &LockLedger, ...)` at line 318.
- `package/git-policy/cli/src/native/push/child_watch_clock_marks_and_stopping.rs:308`:
  `pub(crate) async fn stop_process(pid: u32, locks: &LockLedger, incoming: &mut UnboundedReceiver<Mark>, root_pin: &RootPin)`,
  with the sweep at lines 372-383 on the blocking pool.
- `package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs:238`:
  `let locks: LockLedger = LockLedger::before_start(command);`,
  passed at line 285.
- `package/git-policy/cli/src/native/push/child_watch_output_watched_on_the_runtime.rs:243`:
  the same line,
  passed at line 259.
- The callers that already build a ledger:
  `push/bounded_push.rs:421`,
  `push/bounded_query_run_of_one_command.rs:244`,
  and `push/stop_of_a_push_and_sweep_of_its_locks.rs:162,172,381,456`.
- The callers that pass none:
  `lfs/lfs_upload_child_run.rs:369`,
  `process/child_awaited_until_it_exits_or_the_service_is_stopped.rs:46`,
  `process/child_awaited_until_it_exits_the_stop_or_the_ceiling.rs:328,373`.

What moved since 2026-11-27:
the handover says "`LockLedger` in six files of `push/`".
Today eight release files of `push/` name it,
because round 2's split cut `child_watch.rs` into five files
(`child_watch_one_shot_run.rs`,
`child_watch_output_watched_on_the_runtime.rs`,
`child_watch_task_loop_and_notice.rs`,
`child_watch_clock_marks_and_stopping.rs`,
`child_watch_running_task.rs`),
and `lfs/` and `process/` name it too,
which the handover did not say.

### What the plan says it becomes

"After-stop action:
`ChildWatch::start_with_locks(waiting_for, pid, terms, locks)` and `watch_loop(... locks)` lose the
`locks: LockLedger` parameter,
and the line in `child_watch` that calls `LockLedger::before_start(command)` moves out.
The watcher returns the `StopReport` it already has;
the caller in `bounded_push` and `transaction_git` builds the
`LockLedger` before the start and sweeps after the stop.
The lock folder is in the program,
above the watcher library,
so the watcher cannot name it"
(`doc/planning/cli-git-source-layout-target-tree.md` lines 6418-6427).

Which folder stops depending on which:
no line of `folder-rule.txt`,
measured by the survivors:
`push` keeps `lock` through `bounded_push.rs:421`,
`bounded_query_run_of_one_command.rs:244` and the sweep file.
What it delivers is at the library boundary.
The five `child_watch` files are the files the rescan sends to
`starting-and-watching-programs`,
and that library folder's `uses/` line names `clearing_leftover_reference_locks`
today
(`cli-git-source-layout-target-tree-files.md:322`,
the same name at `:279` in the sibling folder).
The change is what takes it out,
which the plan itself counts on:
the watcher's cleanliness "depends on one change of the draft"
(`cli-git-source-layout-target-tree.md`,
"Changes so that a library stands alone").

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
pub struct WatchedOutput {
    /// What the step wrote,
    ///  or why its output could not be read.
    pub output: std::io::Result<std::process::Output>,
    /// Whether the watch itself ended the child,
    ///  which is when its locks must be swept.
    pub ended_by_the_watch: bool,
}
```

and one call site rewritten,
in `transaction_git.rs` where the step is started:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git.rs
let locks: LockLedger = LockLedger::before_start(&command);
let watched: WatchedOutput = output_watched_within_for_a_stop(
    &mut command,
    subject.as_str(),
    ledger_step.as_str(),
    terms,
    stoppable,
    bound,
    a_stop_lets_it_finish,
    started_child,
)?;
if watched.ended_by_the_watch {
    locks.sweep();
}
let watched_output: std::process::Output = watched.output?;
```

What it delivers:
`clearing_leftover_reference_locks` out of the watcher library's `uses/` list,
which is what the library lift needs;
no allow-list line today.

What it costs a caller:
every caller of the four one-shot run functions reads a `WatchedOutput` instead of an
`std::io::Result<Output>`,
and the sweep moves to the caller,
which already has one (`bounded_push`'s `stop_push`) or gets one.
The watcher's own sweep happened up to a grace after the stop,
inside its task;
the caller's sweep happens when it reads the end,
so the sweep is as prompt as the caller is,
and `LockLedger::sweep` judges each lock before removing it,
so an early sweep removes nothing whose holder is not proven gone.

Which tests change:
`child_watch_tests.rs` (its `watch_loop` call),
`child_watch_drop_time_measurement_tests.rs:183`,
`child_watch_output_watched_on_the_runtime_tests.rs`,
`bounded_push_tests.rs:384,444`,
`bounded_push_awaiting_tests.rs:73`,
`stop_of_a_push_and_sweep_of_its_locks_tests.rs:53,95`,
and every test that calls the four one-shot functions.

What it leaves undone:
the swallow of a sweep when a caller drops the watch without reading its end
(nothing sweeps then,
where the watcher's task would have),
and `child_watch_running_task.rs`,
whose task holds the flag the caller reads.

### Signature B: the watcher is handed what to do after a stop

The library defines the behaviour,
the program implements it:

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

and the same call site:

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
    subject.as_str(),
    child.id(),
    terms,
    std::sync::Arc::new(SweepTheLocks {
        ledger: LockLedger::before_start(&command),
    }),
);
```

What it delivers:
the same library entry as A,
and the sweep keeps running inside the watcher's task,
so a caller that drops the watch still gets its locks swept.

What it costs a caller:
an allocation and a type per call site,
and a behaviour handed across a crate boundary,
which the plan's own note forbids for these changes
("none of the changes below hands a closure;
each hands a value or an enum",
`cli-git-source-layout-target-tree.md` lines 6378-6382):
a trait object is not a closure,
but it is the same shape and the same cost to reason about.
The library must define a trait whose only implementation is the program's.

Which tests change:
as A,
plus a stand-in `AfterTheStop` in every control that starts a watch.

What it leaves undone:
nothing the change names,
and the watcher's own task grows one call it cannot see into.

### Whether it is feasible

Tried,
with a compiler.
The throwaway branch `probe/after-stop-action-a` removed the `locks` parameter
from all four faces and from `stop_process`,
removed the sweep and the ledger builds from the five watcher files,
and dropped the argument from the two test call sites.
`cargo check --locked --offline --all-targets` in `package/git-policy/cli` was run on it
twice under a scope capped at 10G of memory and 4 CPUs.
The first run answered two errors,
both in `child_watch_drop_time_measurement_tests.rs:183`,
where a measurement builds a ledger of its own and had been getting the type through
`child_watch.rs`'s import;
the release code compiled in that run.
The second run,
after that test got its own `use crate::stopped_git_locks::LockLedger;`,
printed no error and ended `Finished `dev` profile [unoptimized + debuginfo] target(s) in 43.32s`.
The branch was discarded.

### Ranking

A > B.

A over B because A hands the caller a value and a flag,
which is the shape the plan asks every one of these changes to have,
and because it keeps the library free of a trait whose only job is to reach back into the program.
B's one advantage is stated above and is real:
a watch dropped without being read still sweeps.
That is a cost of A to weigh,
not a reason to rank it lower,
because the callers that drop a watch today (`repo_read/git_metadata.rs:443`) pass no ledger at all.

## 4. Current step record

### Where the call is today

- `package/git-policy/cli/src/native/transaction/transaction_git_step_about_to_start.rs:229`:
  `enter_step(format!("{} in {}", command_text(&request.arguments), request.cwd.display()))`,
  held in the returned `StepAboutToStart` as `record` and dropped with the step.
- `package/git-policy/cli/src/native/transaction/transaction_git_step_bounds.rs:113`:
  `if crate::service_busy_step_record_on_this_thread::on_landing_thread() {`,
  deciding that a step on the landing thread is unbounded.
- `package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs:230` and
  `child_watch_output_watched_on_the_runtime.rs:235`:
  `crate::service_busy_step_record_on_this_thread::on_landing_thread(),`
  as the first argument of `begin_unfinished_work`.
- The record itself:
  `package/git-policy/cli/src/native/service/service_busy_step_record_on_this_thread.rs`
  (`on_landing_thread` at line 109,
  `use_on_this_thread` at 114,
  `enter_step` at 176),
  which names `landing_state_carried_by_its_task` and `service_busy`'s
  `LandingBoard`,
  `STEP_FILE` and `now_millis`.
- The landing that registers its board:
  `package/git-policy/cli/src/native/service/service_server.rs:174`:
  `crate::service_busy_step_record_on_this_thread::use_on_this_thread(Arc::clone(&board));`,
  and the detail a landing sets:
  `landing/service_land_attempt_branch_move.rs:99`.

What moved since 2026-11-27:
nothing in the call,
and the handover's "in `transaction/`,
`service/` and `push/`" is still right.
What it does not say is that two of the three files outside `transaction/` are the watched-run files
the target tree sends to a library.

### What the plan says it becomes

"Current step record:
`transaction_git` stops calling `service_busy::enter_step` and `on_landing_thread`.
A thread-local record (step file path and detail) moves to the new program file `current_step_record`
in the folder `how_long_the_program_waits_for_git_steps`
(it names Git steps,
so it does not go to a library);
the runner writes it,
the landing code registers its file path there,
and `LandingBoard` stays in `service_busy`.
The two functions listed as removed in the group map are
`service_busy_step_record_on_this_thread::enter_step` and `::on_landing_thread`"
(`doc/planning/cli-git-source-layout-target-tree.md` lines 6406-6417).

Which folder stops depending on which:
no line of `folder-rule.txt` today,
measured by the survivor:
`transaction` keeps `service` through `wait_ceilings`
(`transaction/transaction_phase.rs:42`,
`transaction_git_step_bounds.rs:29,91,93`).
At step 7 `wait_ceilings` becomes `longest_waits.rs` in `how_long_the_program_waits_for_git_steps`
(`cli-git-source-layout-target-tree-files.md:1218`),
and then the transaction folder uses only that low folder.
The delivery that matters now is the library's:
`landing_commits` out of the watcher library's `uses/` line
(`cli-git-source-layout-target-tree-files.md:322`),
which needs the two push call sites in this change,
not only `transaction_git`.

### Signature A: the plan's thread-local record

```rust
//! package/git-policy/cli/src/native/service/current_step_record.rs,
// later how_long_the_program_waits_for_git_steps/current_step_record.rs
thread_local! {
    static THE_LANDING_THIS_THREAD_SERVES: std::cell::RefCell<Option<LandingWrites>> =
        const { std::cell::RefCell::new(None) };
}

/// Where one landing writes its step.
struct LandingWrites {
    /// The folder the step file lives in.
    directory: PathBuf,
    /// What the write that follows does.
    detail: Option<String>,
}

/// Say where the landing this thread serves writes the step it is inside.
pub fn a_landing_writes_its_step_to(directory: PathBuf);

/// Whether the calling thread serves a landing,
///  the one operation that waits for a step it cannot end.
pub fn on_a_landing_thread() -> bool;

/// Say what the write that follows does,
///  for as long as the guard lives.
pub fn detail(text: String) -> DetailGuard;

/// Record that this thread is inside a step that is never ended,
///  until the guard is dropped.
pub fn enter_step(command: &str) -> Option<StepGuard>;
```

and one call site rewritten:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git_step_about_to_start.rs
let record: Option<crate::current_step_record::StepGuard> = if matches!(bound, StepBound::Unbounded) {
    crate::current_step_record::enter_step(
        format!("{} in {}", command_text(&request.arguments), request.cwd.display()).as_str(),
    )
} else {
    None
};
```

and the two watched-run sites lose their ask by taking the answer in the terms:

```rust
//! package/git-policy/cli/src/native/push/child_watch_clock_marks_and_stopping.rs
/// The terms a watch runs under.
pub struct WatchTerms {
    // ... the quiet window, the notice grace and the ceiling ...
    /// Whether the caller serves a landing,
    ///  where a step is never ended.
    pub on_a_landing_thread: bool,
}

// package/git-policy/cli/src/native/push/child_watch_one_shot_run.rs
let mut unfinished: Unfinished = match begin_unfinished_work(terms.on_a_landing_thread, waiting_for) { /* ... */ };
```

What it delivers:
`landing_commits` out of the watcher library's `uses/` line,
and,
after step 7,
the transaction folder's use of `service` gone.
No allow-list line today.

What it costs a caller:
the landing registers a directory instead of a board
(`service_server.rs:174`),
the record grows its own clock and its own step file name,
replacing `now_millis` and `STEP_FILE` from `service_busy`,
and every constructor of `WatchTerms` sets the new field
(`child_class`'s builder and the tests).
The record is thread-local state,
which is what the folder rule cannot see.

Which tests change:
`service_busy_tests.rs` (eighteen calls to `enter_step`),
`transaction_git_tests.rs:1114,1707`,
`service_land_hung_write_tests.rs:737,744`,
`service_land_tests.rs:1564`,
`child_watch_tests.rs`.

What it leaves undone:
`LandingBoard`'s reader of the step file
(`service/service_busy_words_of_a_stuck_write.rs:33`) still reads the path the record writes,
so the file's name is known in two places;

### Signature B: the record is a value the runner holds

```rust
//! package/git-policy/cli/src/native/service/current_step_record.rs,
// later how_long_the_program_waits_for_git_steps/current_step_record.rs
/// Where one landing writes the step it is inside,
///  and what that step does.
pub struct StepRecord {
    /// The folder the step file lives in.
    directory: PathBuf,
    /// What the write that follows does.
    detail: Option<String>,
}

/// The record's own steps.
impl StepRecord {
    /// The record of the landing at `directory`.
    pub fn in_the_landing_at(directory: PathBuf) -> StepRecord;

    /// The same record,
    ///  saying what the next write does.
    pub fn doing(&self, text: String) -> StepRecord;

    /// Record that this thread is inside a step that is never ended,
    ///  until the guard is dropped.
    pub fn enter(&self, command: &str) -> Option<StepGuard>;
}
```

and the transaction's request carries it:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git_request_and_reply_types.rs
/// One step of a transaction as its runner receives it.
pub struct StepRequest<'a> {
    // ... the arguments, the directory, the class and the terms ...
    /// The landing's step record,
    ///  which the service hands to every step it runs.
    pub step_record: &'a StepRecord,
    /// Whether this step runs on the landing,
    ///  where it is never ended.
    pub on_the_landing: bool,
}
```

and the call site:

```rust
//! package/git-policy/cli/src/native/transaction/transaction_git_step_about_to_start.rs
let record: Option<StepGuard> = if matches!(bound, StepBound::Unbounded) {
    request.step_record.enter(
        format!("{} in {}", command_text(&request.arguments), request.cwd.display()).as_str(),
    )
} else {
    None
};
```

What it delivers:
the same as A,
and the value makes the use visible to the folder rule,
so a program folder can no longer reach the record without naming it,
and the watcher's `bool` arrives with the record instead of beside it.

What it costs a caller:
a parameter through every constructor of the transaction's request types,
which is the widest call-site cost of the five changes,
and the landing must build the record before it dispatches a step.

Which tests change:
as A,
plus every test that builds a transaction request.

What it leaves undone:
the same file-name duplication as A,
unless the board is handed the name from the record.

### Whether it is feasible

Feasible on the reading,
not tried with a compiler.
The doubt was whether the new low file can write the step file without naming `service_busy`,
and it can:
`enter_step` needs a directory,
a detail and a clock
(`service_busy_step_record_on_this_thread.rs:176-190`),
all three of which the record can hold,
and `STEP_FILE` is a constant whose value can sit beside the writer.
`now_millis` is `SystemTime` in two lines.

### Ranking

B > A.

B over A because both remove the same use,
and B removes it in a way the folder rule can see,
so the tangle cannot grow back through a name no line records;
B also gives the watcher its answer in the same value,
where A needs a second field beside the thread-local.
A's advantage is real and stated:
a parameter through the request types is more call sites than a thread-local.
Effort is a cost,
not a reason to rank a signature lower.

## 5. Run this next

### Where the call is today

The push half:

- `package/git-policy/cli/src/native/push/stacked_merge_push.rs:33`:
  `use crate::entry::plan_invocation;`,
  and line 40 `run_stacked_push,`;
  `pub fn gated_merge_push(git: &PushGit, global_prefix: &[OsString], tokens: &[OsString], destination: &[String]) -> Result<i32, SingleFlightError>`
  at line 209,
  whose last two lines are
  `let action: Action = plan_invocation(arguments.as_slice(), environment.as_slice(), &inputs);`
  and `return Ok(run_planned(action));` (lines 226-227).
- `fn run_planned(action: Action) -> i32` at line 119,
  whose `Action::ForwardPush` arm calls `run_stacked_push(&forward, &gated_merge_push)` at line 161.
- `package/git-policy/cli/src/native/push/push_stacking.rs:353`:
  `pub fn run_stacked_push(forward: &PushForward, merge_push: MergePush<'_>) -> i32`.
- The top level that would run the next command:
  `invocation/entry.rs:580` (`return run_stacked_push(&forward, &gated_merge_push);`)
  and `invocation/entry_run_forwarded_actions.rs`,
  which `entry.rs:545` reaches through `run_coordinated_action(&forward)`.

The merge half:

- `package/git-policy/cli/src/native/merge/merge_second_pass_finish.rs:63`:
  `use crate::entry::run_nested_process;`,
  `fn run_nested(arguments: &[OsString], environment: &[(OsString, OsString)]) -> i32` at line 308,
  counting `NESTED_FINISHES` around `run_nested_process(arguments, environment)` (lines 312-314),
  called at lines 340,
  392 and 418.
- `NESTED_FINISHES` itself,
  `merge/merge_second_pass_finish_nesting.rs:31`.
- `invocation/entry.rs:614`:
  `pub fn run_nested_process(arguments: &[OsString], environment: &[(OsString, OsString)]) -> i32`.
- The path the status travels up:
  `finish` (line 356) is called by `merge/merge_second_pass_worktree.rs:267`,
  whose `resolve_after(...) -> Option<i32>` (line 116) is called by
  `lock/index_writer_forward.rs:342` inside `run_coordinated_forward(...) -> CoordinatedRun` (line 202),
  which `entry.rs`'s `ForwardCoordinated` arm runs.

What moved since 2026-11-27:
nothing in the call,
and the handover's `plan_invocation` in `push/stacked_merge_push.rs` is still at line 226.
What the handover leaves out is the merge half,
which is where the nesting counter lives and where the status has four functions to travel through.

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
(`doc/planning/cli-git-source-layout-target-tree.md` lines 6385-6405).

Which folder stops depending on which:
no line of `folder-rule.txt` today,
measured by the survivors:
`push` keeps `invocation` through `push_stacking.rs:34` (`crate::action::PushForward`)
and `:60` (`crate::child_environment::environment_with_overlay`),
`bounded_push.rs:215` (`crate::forwarding::ChildOutcome`),
`manual_push_lifecycle.rs:408` (`crate::typed_network_command_relay`).
At step 7 `stacked_merge_push.rs` becomes
`push_of_a_merge_that_integrated_a_lost_race.rs` in `working_out_what_a_typed_push_would_send`
(`cli-git-source-layout-target-tree-files.md:3755,3776`),
whose `uses/` list names `starting_git_and_describing_a_run`
(`:3725`) and would lose it,
and the merge folder's entry loses it too,
provided `PushForward` moves into `push_stacking` first:
that item move is still owed,
and `push_stacking.rs:34` still imports it from `crate::action`.

### Signature A: the plan's step functions

```rust
//! package/git-policy/cli/src/native/invocation/what_a_command_hands_back.rs
/// One command the wrapper plans and runs as its own child.
pub struct NextCommand {
    /// The words after the program name.
    pub arguments: Vec<OsString>,
    /// The environment the command runs with.
    pub environment: Vec<(OsString, OsString)>,
}

/// How a stacking step ends.
pub enum StackingStep {
    /// The stacking is over.
    Done {
        /// The status the process ends with.
        code: i32,
    },
    /// Run this command,
    ///  then hand its status back with `resume`.
    RunThis {
        /// The command to run.
        command: NextCommand,
        /// The state to hand its status back with.
        resume: StackingState,
    },
}

/// How a nested finish ends.
pub enum FinishStep {
    /// The finish is over.
    Done {
        /// The status the process ends with.
        code: i32,
    },
    /// Run this command,
    ///  then hand its status back.
    RunThis {
        /// The command to run.
        command: NextCommand,
    },
}
```

```rust
//! package/git-policy/cli/src/native/push/stacked_merge_push.rs
/// The marked push the top level runs.
pub fn gated_merge_push(
    git: &PushGit,
    global_prefix: &[OsString],
    tokens: &[OsString],
    destination: &[String],
) -> Result<NextCommand, SingleFlightError> {
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.push(OsString::from("push"));
    arguments.extend(tokens.iter().cloned());
    arguments.extend(destination.iter().map(OsString::from));
    return Ok(NextCommand {
        arguments,
        environment: marked_environment(git.environment.as_slice()),
    });
}
```

and the top level:

```rust
//! package/git-policy/cli/src/native/invocation/entry.rs,
//!  in the arm that runs a coordinated forward
/// How deep the top level runs what a command handed back.
const NESTING_LIMIT: u32 = 8;
let mut nesting: u32 = 0;
let mut wanted: Option<NextCommand> = None;
loop {
    let action: Action = match wanted.take() {
        Some(command) => plan_invocation(command.arguments.as_slice(), command.environment.as_slice(), &inputs),
        None => Action::ForwardCoordinated {
            forward,
            stderr: String::new(),
        },
    };
    match run_and_carry(action) {
        Ran::Ended(code) => return code,
        Ran::Next(command) => {
            nesting += 1;
            if nesting > NESTING_LIMIT {
                return ENGINE_FAILURE_EXIT_CODE;
            }
            wanted = Some(command);
        }
    }
}
```

Where `run_and_carry` is today's `run_process` body with one added outcome,
which is the change the merge half also needs:
`CoordinatedRun` (`lock/index_writer_forward.rs:202`) gains
`Next(NextCommand)` beside its `Ran` and `Failed`,
so `finish` can hand the command up through `resolve_after` and `run_coordinated_forward`.

What it delivers:
at step 7,
`what_happens_when_you_type_a_command` out of
`working_out_what_a_typed_push_would_send`'s and the merge folder's `uses/` lists,
and the rule the plan wants restored:
the top level is called only from the top.
No allow-list line today.

What it costs a caller:
one `NextCommand` to run and a status to hand back,
two new enums,
one new variant in the coordinated run's outcome,
a loop and a depth guard at the top,
and the merge path's return type changed through four functions
(`finish`,
`resolve_after`,
`run_coordinated_forward`,
`run_coordinated_action`).

Which tests change:
`push_stacking_tests.rs:486`,
`entry`'s tests that read a planned `Action`,
`merge_second_pass_finish_tests.rs`,
`merge_second_pass_worktree`'s tests,
`index_writer_forward`'s tests,
and `auto_push_reconcile_tests.rs:3533`,
which describes this call in prose.

What it leaves undone:
the second recursion the plan names is the wider half and can land after the push half,
provided `CoordinatedRun` gains the variant in the same branch;
`PushForward`'s move down,
without which the folder keeps its use of `invocation`;
and the file `what_a_command_hands_back` is shared with this round's
commands-that-only-finish change,
so the two changes touch one file.

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

and one call site rewritten:

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
        let action: Action = plan_invocation(command.arguments.as_slice(), command.environment.as_slice(), self.inputs);
        return run_the_action(action);
    }
}

return run_stacked_push(&forward, &gated_merge_push, &mut TheTopLevel { inputs: &inputs });
```

What it delivers:
the same folder payoff as A at step 7,
with a far smaller diff,
because the stacking loop stays where it is.

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
the merge recursion and the rule the change serves.

### Whether it is feasible

Feasible on the reading,
not tried with a compiler.
The doubt was whether the status a nested finish needs can travel up through four functions,
and it can,
because every one of them already returns a value that carries the outcome
(`FinishStep` from `finish`,
`Option<i32>` from `resolve_after`,
`CoordinatedRun` from `run_coordinated_forward`,
`i32` from `run_coordinated_action`).
What no reading settles is how much of the merge path's own test suite assumes a status is returned
rather than a command,
which is the measurement the round itself would take.
A compiler check was not run for this signature
because the check would need the whole merge path converted,
which is the round,
not a probe.

### Ranking

A > B.

A over B because A is the only one of the two that restores the rule the change serves:
after B the top level is still called from inside the stacking loop,
so a program that runs a command from below stays possible and the folder tangles can regrow.
B's advantage is stated and real:
it is a fraction of A's diff,
and its push half could land before the merge half.

## Across the five: the order they can land in

Which files two of them both touch,
measured by the call sites above:

- Commands that only finish and run this next share two things:
the new file `what_a_command_hands_back`,
where the finished value and the next command sit together,
and `push/stacked_merge_push.rs`,
whose `Action` the first would remove and whose `plan_invocation` the second removes.
The second covers the first's part there,
so the first cannot land second without undoing it.
- After-stop action and current step record share
`push/child_watch_one_shot_run.rs` and `push/child_watch_output_watched_on_the_runtime.rs`,
where the after-stop change removes a parameter from the call
and the step-record change adds the landing answer to `WatchTerms`
and changes the first argument of `begin_unfinished_work` beside it.
They also meet in `transaction_git.rs` and `transaction_git_awaiting.rs`,
which the step-record change reaches through the request types.
- System settings path touches no file any other change touches.

So round 4 is not five rounds side by side.
It is at most three branches at once:
`{2}`,
`{3 then 4}` and `{1 then 5}`,
and the two pairs are ordered inside themselves.
The plan's own order
(`cli-git-source-layout-target-tree.md` lines 8135-8141,
"step 4's five call changes,
one commit each")
lists them finished results,
system settings path,
after-stop action,
current step record,
run this next,
which keeps the two pairs in their own order and puts the system settings change third.
Nothing in the plan requires that order;
keeping it costs nothing and keeps a round's commits reading like the plan.

The dependencies that are not about one file:

- Run this next needs the finished value whether or not the finished-results change runs,
because the merge half hands `NextCommand` and `Finished` through one file.
- Run this next's folder payoff needs `PushForward` to move into `push_stacking`,
which the plan's items list asks for and `main` still has not done
(`push_stacking.rs:34` imports it from `crate::action`).
Without that move the folder keeps its use of `invocation` and the change delivers nothing to the target tree.
- After-stop action and current step record both need the library's answer
(`cli-git-source-layout-target-tree-files.md:322`),
and neither delivers it alone:
the first removes `clearing_leftover_reference_locks` from those files,
the second removes `landing_commits`,
and a round that lands one without the other leaves the library line listing the other.
- The system settings change delivers nothing to any line until step 7,
whichever signature is chosen,
so it is the one that can wait without holding anything else.

## Questions for the owner

Each question is a choice between things the repository cannot measure,
and each is answerable alone.
The recommendation is first,
with what it costs.

### The first question: where the finished value lives

Recommendation:
signature A,
the value in a low file of `foundation`,
which removes two lines of the allow list in this round
(`uses src/native/subvolume -> src/native/invocation` and
`names-group src/native/subvolume src/native/invocation`)
and costs one more file move at step 7,
when `foundation` is split among three libraries and six program folders.

- The plan's own placement (signature B,
  `what_a_command_hands_back` in `starting_git_and_describing_a_run`)
  removes no line in this round and needs no second move;
  it delivers at step 7,
  when three command folders drop `what_happens_when_you_type_a_command` from their `uses/` lists.
  Its cost is that a branch landing it leaves the `subvolume` and `invocation` loop standing,
  which the owner's decision of 2026-11-28 wanted untangled over time rather than held.
- Neither value at all (signature C,
  each command writes its own output and returns the status)
  removes the same two lines as A and needs no move later.
  Its cost is the wrapper's one place that writes a report
  (`entry.rs`'s `write_exit`),
  and every test that reads an `Action` becomes a test that reads bytes.

A concrete difference,
in one sentence each:
with A,
`migrate_action` returns `Finished::quietly(0)` and `management` turns it into `Action::ExitBytes`;
with B,
the same value is built from `invocation` and `management` does the same;
with C,
`migrate_action` writes its own answer to a writer and returns `0`.

### The second question: which file asks real Git for the system settings path

Recommendation:
signature C,
one answer learned per process and read without asking,
because it is the only one that takes the ask out of the folder whose name promises not to start Git,
and it adds no use to any caller.
Its cost is stated in full:
a Git replaced while a long-lived service runs is not noticed,
where today's `learned_path` re-reads the executable's identity on every ask.

- The plan's parameter (signature A)
  changes nothing a read answers today:
every builder passes `system_config_for` of the same variables,
so no answer moves.
  Its cost is that the three reader files that build their own scope go on asking from inside the reader's folder,
  so the folder's promise holds only for the callers that hand a value over.
- Two types with one settled (signature B)
  gives the same as A plus a scope that cannot be read half-built.
  Its cost is a second public type and two calls at every site.

What each costs in behaviour,
concretely:
with A,
a `ScopeEnvironment` built from the process's own variables passes `None`,
which is what it gets today;
with C,
a test that never learns the path sees a scope that declines,
which is what the existing controls around an unknown system file already assert.

### The third question: whether the step record is a thread-local or a value

Recommendation:
signature B,
a `StepRecord` value the landing builds and the runner holds,
because it is the only one of the two the folder rule can see,
so a program folder cannot reach the record again without naming it,
and it gives the watched-run files their landing answer in the same value.
Its cost is the widest call-site change of the round:
the transaction's request types gain a field,
and every constructor of them changes.

- The plan's thread-local (signature A)
  costs fewer call sites:
  one registration,
  one new field in `WatchTerms`,
  and the same two dead uses at step 7.
  Its cost is that the record stays a name no line records,
  which is the shape that let the tangle grow.

A concrete difference:
with A,
`transaction_git_step_about_to_start.rs` calls `crate::current_step_record::enter_step(...)` and nothing else changes;
with B,
the same line calls `request.step_record.enter(...)`
and the service builds that record once per landing.

### The fourth question: one branch or two

Recommendation:
two branches side by side,
`{2, 3, 4}` and `{1, 5}`,
which is two of the three units the order above allows,
because no file is touched by both,
and the plan's reason for one branch (a branch that edits a file another is about to move fights it)
holds inside each pair and not across the pairs.
What it costs:
two merges,
two gate runs,
and the linter's red findings split across two branches,
so the handover's list of them is written once per branch.

- One branch with five commits,
  in the plan's order,
  costs one merge and one gate run,
  and keeps the commits readable as the plan's step.
  Its cost is the wall-clock of one round doing all five,
  which is the shape the handover's own entry already proposes
  ("one round per call change",
  with three of them waiting for work in flight).

Whether either is even startable now depends on work in flight,
which the handover entry names:
the push split cuts `child_watch_one_shot_run.rs`,
issue 6573 changes the control that reads `repo_read/git_system_config.rs`,
and the answer to issue 6529 edits `merge_second_pass_finish.rs`.
This proposal was written without waiting for them.

### The fifth question: whether the merge recursion is in this round

Recommendation:
in,
because the plan's run this next is one call change that names both recursions,
and the depth counter it removes (`NESTED_FINISHES`) is in the merge half;
leaving the merge half out would leave the counter and the recursion the plan names.
What it costs:
four return types and their tests,
and the widest single part of the round.

- The push half only,
  as the handover entry reads it,
  costs far less and still removes the push's in-process planning.
  Its cost is that a round named after the plan's call change would leave half of it,
  and the next round would revisit the same file (`what_a_command_hands_back`) to finish it.

## What was measured, and what was not

Measured,
and quoted above where each number is used:

- Today's lint findings for the crate at `4cf7cc74b8`:
  eleven `rust/folder-uses-links`,
  fourteen `rust/max-lines`,
  one skipped rule.
- The crate's folder uses with the allow list emptied,
  one proof per pair,
  five hundred and sixty-three `uses` lines.
- The dead lines commands that only finish deliver,
  named by the linter on a branch that held the type substitution.
- The compiler's answer for the after-stop change:
  `cargo check --locked --offline --all-targets` over the whole package,
  green in 43.32 s,
  with `child_watch.rs`,
  `child_watch_task_loop_and_notice.rs`,
  `child_watch_clock_marks_and_stopping.rs`,
  `child_watch_one_shot_run.rs` and
  `child_watch_output_watched_on_the_runtime.rs` no longer naming `LockLedger` at all.

The probes' own runs are kept with this run's report,
not cited here:
this record states what each answered.

Not run,
and why:

- No test suite,
  nextest run,
  gate,
  guest or release.
  This branch changes no code;
  the probes were compiler and linter answers only,
  and a green `cargo check` says nothing about behaviour.
- No compiler check for the other four call changes.
  The reason for each is stated where it is claimed,
  and where the cheap check would have answered nothing about the doubt.
- The round's own measurements of behaviour
  (what a nested command does to the process's streams,
  how a watch's sweep timing changes)
  are for the round that lands it.

## What this proposal does not settle

- The plan's two further call changes found by the moved-item check
  (`copy_file_exclusive`'s error type and the hint functions of `event_hints`)
  belong to step 5,
  not round 4,
  and are not described here.
- `system_attributes_for` is the same shape as the system settings path
  and the plan does not name it.
  It stays asking wherever the chosen signature leaves it.
- Which red `rust/folder-uses-links` findings `main` prints today
  (`doc/handover/cli-git-rust-implementation.md` keeps that list)
  is not read here;
  this proposal's line numbers are of `layout-allow-list/folder-rule.txt` at `4cf7cc74b8`,
  and a line added or deleted before the round starts moves them.
- The owner's other open points for the split
  (`cli-git-source-layout-target-tree.md`,
  "Open points for the owner")
  hold steps 7 and after,
  not this round,
  except that reading D or E of the library count decides whether
  `starting-and-watching-programs` is a library at all.
  Both readings keep it,
  so the after-stop and step-record changes are needed under either.
