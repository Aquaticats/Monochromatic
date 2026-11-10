# Source layout of the Git wrapper crate (issue 6261), second design

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Status:
analysis in progress,
nothing moved;
a group order with no upward reference is found (section "Group order found,
from the bottom").
Started 2026-11-04 by the coordinator session after the owner stopped the child session
("I told the layout subagent to stop because it couldn't ever produce something good enough.
Lets do this ourselves.").
The child session's third proposal (branch `refactor/git-policy-cli-source-layout`,
`671eb1c6d1`) is not used;
only its measured edge data was looked at.

## What the owner asked for

Owner,
2026-11-04,
verbatim:

- "Principle 1:
  Lint issues need to be taken care of asap."
- "Principle 2:
  every name of a dirent must be understandable to a monkey.
  If jargon is unavoidable,
  the file's top doc must explain everything,
  in a way that would make any monkey understand."
- "Principle 3:
  Who depends on who must be immediately visible from looking at the file structure."
- "Consider monkey someone who only knows what git commit and git push are."
- "Don't discard work,
  but aside from layout + linting fixes,
  pause everything else - but let work already in flight run to their natural conclusions.
  Layout + linting fixes are the biggest priority."
- "Add in AGENTS.md to not use vague words like `native` when naming dirents."
  (AGENTS.md VGW;
  the draft names in section "Proposed top-level folder names" follow it,
  and `src/native/` goes away.)
  Then:
  "helper_<specific> is fine.
  The rule language needs to be better."
  So a vague word is allowed beside the specific part and never alone;
  VGW was reworded to say so.

Question tool answers,
2026-11-04:

- Structure,
  first answer:
  "B:
  numbered layers (Recommended)",
  with the note "Don't be afraid of using extremely long names.
  B,
  but make sure a monkey even just looking at one thing know what it means."
  Shown a sketch with two-digit layer numbers (`01_tools_for_files_programs_and_time/` to
  `11_deciding_what_to_do_with_one_git_command/`),
  the owner asked:
  "What if I still think this is pretty un-understandable?
  What is 01 to 11 supposed to mean?"
  The numbers meant "a folder uses only folders with a smaller number",
  which no name says on its own.
  Second answer,
  replacing the numbers:
  "A:
  a uses/ list in each folder (Recommended)".
  Every source folder holds a `uses/` folder whose entries are links to the folders its code uses;
  no layer numbers;
  two folders never use each other;
  the package's lint fails when the code and the links disagree.
  The sketch's names were rewritten from the view of someone who knows only `git commit` and `git push`
  (`small_helpers_for_files_programs_and_clocks`,
  `reading_what_git_saved_on_disk`,
  `running_git_for_you`,
  `your_settings`,
  `checking_your_changes_before_they_are_committed`,
  `what_happens_when_you_type_git_commit`);
  the owner named none of them as wrong.
  AGENTS.md LYD and LYF now say this.
- Granularity:
  "Files too,
  checked (Recommended)":
  files inside one folder never use each other in a loop,
  the folder's top doc lists them in dependency order,
  and the package's lint fails any file or folder that breaks its layer.
- Scope:
  "Repo-wide rules (Recommended)":
  AGENTS.md MKY,
  LYD and LYF (`b1955e0b49`).
- TypeScript in the crate:
  "A:
  port all to Rust (Recommended)" (separate work,
  paused).
- Question tool answers,
  2026-11-04 (after the group order and the draft names were shown):
  - Top-level folder names:
    "A:
    plain sentences (Recommended)",
    the list in section "Proposed top-level folder names",
    with `src/native/` gone and the folders directly in `src/`.
  - The two in-process recursions (`stacked_merge_push` planning a new invocation,
    `merge_second_pass_finish` running a nested command):
    "A:
    hand back 'run this next' (Recommended)":
    the part returns the next command and the top level runs it in its own loop.
  - The four smaller call changes (current-step record,
    the watcher's after-stop action from its caller,
    the system settings path passed to the reader,
    "finished" results for commands that only finish):
    "Make all four (Recommended)".

## Splitting into crates (owner, 2026-11-04)

The owner:
"I just realized something.
We should not put everything in one crate."
Asked how fine the split should be (A:
one crate per top-level folder;
B:
a few layer crates;
C:
only the reusable parts),
the owner picked C with the note:
"What do you mean?
Crates obviously have to be reusable,
otherwise there's no point.
Also I refuse to believe C means splitting into fewer crates."
So every crate is a reusable unit (something another program could depend on for one job),
and the number of crates follows from how many such units the code holds,
not from a target count;
option C's sketch of four crates beside one large `cli` was wrong to suggest few.
Where they live:
"A,
and did you miss how we can't really use cargo workspace?":
each crate is its own package `package/git-policy/<name>` (crate `monochromatic-git-policy-<name>`),
its own Cargo workspace root (`doc/decision/rust-dependency-catalog.md`:
"Every Rust crate is its own Cargo workspace
root";
`.out-of-scope/cargo-workspace.md`:
profiles belong to each package),
depending on the others by path with an exact version pin,
as `package/file-enforcer/cli` depends on
`../core-rust` with `version = "=0.1.1"`.
The `uses/` links:
"A:
Cargo.toml between crates,
uses/ inside (Recommended)":
between crates a crate's `Cargo.toml` dependencies are its list (cargo refuses loops between crates);
folders inside a crate keep `uses/` links and the check.
Every crate the installed program depends on is published to crates.io,
because the program installs through
cargo-binstall from crates.io (`toolchain.lock.json`:
`cargo:monochromatic-git-policy-cli@latest`).
Superseded 2026-11-05 (`doc/planning/crates-io-footprint.md`,
section "Research results (2026-11-05)"):
the tools,
this wrapper among them,
move to the `archive` route (install from the GitHub release)
and stop publishing to crates.io after their one release each,
so the split crates need no crates.io names and are `publish = false`;
the one release of the wrapper must happen before it depends on an unpublished crate.
Where libraries go is the owner's open question in `doc/handover/questions-for-the-owner.md`.

## What the code looks like now

Measured on main at `2d82a3beb4`
(tools and outputs in `~/temp/agent/issue-6261/`:
`scc.ts`,
`fas.ts`,
`modules.ts`,
`groups.ts`,
`suggest.ts`,
and `scc/`).
A reference is a `crate::<module>` or `super::<module>` path in code,
with comments and string literals blanked;
test files (`*_tests.rs`) are left out because nothing in production uses them.

- 554 production modules,
  2,298 references between them.
- 266 modules sit in one dependency loop (plus loops of 4 and 2).
  A greedy order of the loop leaves 82 references pointing backwards;
  that is an upper bound on how many references must change,
  not a plan.
- Every folder is re-exported flat at the crate root (`pub use <folder>::*;` in `src/native/lib.rs`),
  so every file names every other as `crate::<module>`,
  and the folders say nothing about who uses whom.
  These re-exports go:
  after the move every `use` names the folder path.

## Method

1.  Put each module in a target group by what it does.
2.  Give the groups an intended order from the bottom (basic tools first,
    the command-line entry last).
3.  For every reference from a lower group to a higher one,
    decide:
    MOVE when the used module depends only on things below its users (it is simply in the wrong folder),
    KNOT when it also depends on something at or above its users (code must change:
    move a type or constant down,
    pass a value in,
    or split a module).
4.  Repeat until no reference points up and no two groups loop;
    then the same inside each group,
    file by file.
5.  Name every folder and file for someone who knows only `git commit` and `git push`,
    write each folder's top doc,
    and list in each folder's `uses/` the folders its code uses.
6.  Show the owner the whole tree (old name,
    new name,
    one-line explanation,
    what it uses) before any file moves.

## Group order found, from the bottom (working names, not the final names)

Measured on main at `6f731f15ca` (2026-11-04T18:12:05-05:00,
before the branch-write merge `de36a92311`),
with the item-level scanner `~/temp/agent/issue-6261/refs.ts`,
the crossing report `crossings.ts` (with a search for the group order that leaves the fewest upward references)
and the in-group loop report `inside.ts`.
The group map is committed beside this file as
[`cli-git-source-layout-groups.json`](cli-git-source-layout-groups.json):
`folders` and `modules` place each file in a group,
`items` moves single items (a type,
a constant,
a function) to another file,
and `(removed)` marks a reference that a named code change takes away.

The scanner now also blanks character literals (`'"'` hid code up to the next quote,
missing five references)
and leaves out test-only modules:
`*_tests.rs` files and every module whose `mod` line carries a `cfg` attribute naming `test`,
including ones declared inside a production file through `#[path]` (`index_reset_support`).
Test helpers live with the tests they serve and are not part of the shipped order.

Under this map the groups use each other in one direction only:
no reference points from a lower group to a higher one.
From the bottom:

1.  `basic_tools`:
    files,
    processes,
    time,
    logging,
    parallel work,
    watching a child for progress
    (today `foundation/`,
    `process/`,
    plus `child_watch`,
    `wait_progress`,
    `push_progress`,
    `add_timing`,
    `subvolume_sha`,
    `lfs_output_mask`,
    `shell_quote`,
    `client_journal`,
    `service_fault`,
    `service_durable`).
2.  `git_files`:
    reading Git's own files without starting Git.
3.  `typed_command`:
    reading the command the person typed,
    including the list of policy names (`policy_registry`).
4.  `start_git`:
    finding and starting the real Git,
    the readers that may start it,
    `forwarded_command`,
    `worktree_identity`,
    `landing_location`.
5.  `locks`:
    Git's lock files,
    including the service's hold on `index.lock` (`service_index_*`).
6.  `push_runner`:
    running `git push` with a deadline and one push at a time
    (`bounded_push`,
    `bounded_query`,
    `auto_push_single_flight`,
    `push_lock_clearing`,
    `push_deadline_advice`).
7.  `settings`:
    the wrapper's own configuration,
    including the discarded-copies section (`config_discarded`).
8.  `checks`:
    the policies and the files they read,
    the rule modules from `command/` and `commit/`,
    `diagnostics`,
    the hook warnings,
    the standards review key.
9.  `service_talk`:
    what a command needs to talk to the background commit service
    (protocol,
    instance,
    messages,
    provisioning,
    client,
    route,
    work records,
    claim time,
    timing,
    handoff,
    landed summary,
    commit timing,
    pruned content,
    `inner_git_words`).
10. `merge_engine`:
    the merge machinery that push and shadow reuse (second pass,
    isolation journal and recovery).
11. `shadow`,
    then `subvolume`,
    `repair`,
    `lfs`,
    `push`,
    `worktree` (with today's `worktree_copying/`),
    `conflict`.
12. `checks_runner`:
    `policy_checks`,
    which runs every check and calls the conflict judges.
13. `merge`,
    `health`,
    then `service_server` (the service itself:
    landing,
    after-commit work,
    index reset),
    `commit` (with the commit-through-the-service files),
    and `decide` (the top-level decision,
    `index_writer_forward`,
    `worktree_copy_forward`).

The order puts `commit` above `service_server`:
the commit command reuses landing pieces (`update_index`,
`IndexPaths`) in its own process,
and nothing in the service uses the commit command.

## Named code changes the order needs

Moving whole files to the group that matches what they do accounts for most of the change.
The rest are items that move to a lower file,
plus five changes to how code is called.

### Items that move down (from the group map's `items`)

- Process outcome types and the subcommand reader (`ChildOutcome`,
  `outcome_of`,
  `subcommand_of`) to `child_class`.
- Machine and boot identity (`current_boot_id`,
  `current_machine_id`,
  `is_machine_id`) to `process_identity`.
- The service unit name and its directory to `real_git_record`;
  `remember_own_executable` to `real_git`.
- `LANDING_ATTEMPTS` and `Bind` to `service_protocol`;
  `work_directory` and `Verdict` to `work_item`.
- `capture_stamp` to `claim_time`;
  `effective_directory` to `effective_target`;
  `WorktreeIdentity` to `effective_target`.
- The sequencer facts (`SequencerFacts`,
  `SequencerState`,
  `sequencer_state`) to `repo_read_adapter`.
- The in-process reading switch (`repo_read_hook::enabled`) to the readers.
- Conversion facts (`ConversionEnvironment`,
  `SystemAttributes`,
  `CLAIM_PREFIX`) to the subvolume files.
- `ForwardedRun` and `run_forwarded_git` to `forwarded_command`;
  `CoordinatedForward` and `CoordinatedRun` to the index-lock wait;
  `PushForward` to `push_stacking`.
- Small helpers to the basic tools:
  `append_line`,
  `read_plain_file`,
  `copy_file_exclusive`,
  `STOP_GRACE`,
  `ENGINE_FAILURE_EXIT_CODE`.
- Configuration warning rendering (`render_configuration_warning`,
  `render_engine_failure`,
  `EngineFailureCode`)
  to the settings group;
  `Limits` and `parse_size` to `config_discarded`.
- `FailureCode` and `storage_failure` to the index-lock files;
  `ShippedChecks` to `policy_pass`.

The simulation does not follow what a moved item itself uses;
the compiler checks that when the item really moves.

### Changes to how code is called

- The push of an integrated merge (`stacked_merge_push`) plans a whole new invocation through `entry::plan_invocation`,
  and the clean finish of a merge (`merge_second_pass_finish`) runs a nested command through `entry::run_process`.
  Both are the top level called from below,
  and both are recursion inside one process.
  Proposed:
  they return "run this command next" to the top level,
  which runs it in its loop.
  This follows the owner's ban on the wrapper invoking itself;
  open to veto.
- The one runner of real Git (`transaction_git`) tells the service which step it is in
  (`service_busy::enter_step`,
  `on_landing_thread`).
  Proposed:
  a small "current step" record in the basic tools that the runner writes and the service reads.
- The process watcher (`child_watch`) sweeps stale Git locks itself (`stopped_git_locks::LockLedger`).
  Proposed:
  its caller hands it what to do after a stop.
- The configuration reader (`repo_read_config_scan`) asks for the system configuration path,
  which can start real Git (`git_system_config::system_config_for`).
  Proposed:
  the caller passes the path in,
  so a reader never starts Git.
- Commands that only finish (`subvolume_command`,
  `doctor`,
  `worktree_convert_command`,
  `stacked_merge_push`)
  build the top-level `Action`.
  Proposed:
  they return a small "finished with this code and output" value from a low file;
  only the top level builds `Action`.

## Loops between files inside one group

With the map applied,
the groups have no loop between them,
but 27 loops between files remain inside groups,
touching 79 files (`~/temp/agent/issue-6261/scc2/inside.txt`).
Most share one shape:
a file and a helper split off from it each use the other's types.
The usual fix is a third file at the bottom of the folder that holds the shared types,
used by both.
Some need a closer look at the functions,
for example the placement,
reclaim and claim files of the `index.lock` hold,
which may call each other in a cycle.

## Proposed top-level folder names (draft for the owner, nothing renamed)

Each name is meant to be understood by someone who knows only `git commit` and `git push`.
Each folder's top doc explains its words further,
and its `uses/` folder holds one link per folder its code uses.
Listed from the folder that uses nothing to the folder that uses everything.

1.  `saving_files_starting_programs_and_keeping_time`:
    saving files safely,
    starting other programs and watching that they still make progress,
    clocks,
    log lines.
    Uses nothing.
2.  `reading_what_git_saved_on_disk`:
    Git keeps your commits and settings as files in the hidden `.git` folder;
    this reads them directly,
    which is faster than starting Git.
    Uses:
    `saving_files_starting_programs_and_keeping_time`.
3.  `understanding_the_git_command_you_typed`:
    reads the words after `git` the way Git does,
    and the words meant for this wrapper.
    Uses:
    `saving_files_starting_programs_and_keeping_time`.
4.  `running_the_real_git_program`:
    this wrapper only pretends to be `git`;
    this finds the real `git` on the computer,
    starts it,
    and answers some of its questions without starting it.
    Uses:
    1 to 3 by name:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `understanding_the_git_command_you_typed`.
5.  `waiting_your_turn_to_change_git_files`:
    Git puts a "lock" file beside a file it is changing so that nothing else changes it at the same time;
    this waits for those,
    takes them,
    and removes the ones a crashed program left behind.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`.
6.  `running_git_push_with_a_time_limit`:
    runs `git push`,
    stops it when it stops making progress,
    and runs one push at a time.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`,
    `waiting_your_turn_to_change_git_files`.
7.  `your_settings_for_this_wrapper`:
    reads `cli-git.config.jsonc`,
    where checks are turned on and off,
    and Git's own settings.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `understanding_the_git_command_you_typed`,
    `running_the_real_git_program`.
8.  `checking_your_changes_before_they_are_saved`:
    the checks a commit or a push must pass (no secrets in files,
    every file ends with a newline,
    package versions raised,
    and so on),
    and the files each check reads.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `understanding_the_git_command_you_typed`,
    `running_the_real_git_program`,
    `your_settings_for_this_wrapper`.
9.  `talking_to_the_background_commit_helper`:
    commits are saved by a helper program that keeps running in the background,
    so that many commits go through quickly one after another;
    this is how a command asks it to save a commit and hears back.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`,
    `waiting_your_turn_to_change_git_files`,
    `your_settings_for_this_wrapper`,
    `checking_your_changes_before_they_are_saved`.
10. `combining_two_versions_of_the_same_file`:
    when two people changed the same file,
    Git combines the two;
    where Git gives up on a line,
    this tries again word by word.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`,
    `checking_your_changes_before_they_are_saved`,
    `talking_to_the_background_commit_helper`.
11. `a_hidden_copy_of_git_data_for_preparing_commits`:
    to prepare a commit without touching your work,
    the wrapper makes a hidden second Git folder
    that shares your saved commits.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`,
    `your_settings_for_this_wrapper`,
    `checking_your_changes_before_they_are_saved`,
    `talking_to_the_background_commit_helper`,
    `combining_two_versions_of_the_same_file`.
12. `instant_folder_copies_on_btrfs_disks`:
    btrfs is a kind of disk that can copy a whole folder instantly;
    this converts a project folder so it can be copied that way.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `running_the_real_git_program`.
13. `fixing_installed_tools_in_a_copied_project_folder`:
    after a project folder is copied,
    its installed packages still point at the old folder;
    this repairs them.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `running_the_real_git_program`.
14. `uploading_big_files_kept_outside_git`:
    Git LFS keeps big files (pictures,
    videos) on a separate server;
    this uploads them before a push.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `understanding_the_git_command_you_typed`,
    `running_the_real_git_program`,
    `running_git_push_with_a_time_limit`,
    `your_settings_for_this_wrapper`.
15. `what_happens_when_you_type_git_push`:
    the push you type,
    the push the wrapper makes for you after each commit,
    and what happens when someone else pushed first.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `understanding_the_git_command_you_typed`,
    `running_the_real_git_program`,
    `running_git_push_with_a_time_limit`,
    `your_settings_for_this_wrapper`,
    `checking_your_changes_before_they_are_saved`,
    `talking_to_the_background_commit_helper`,
    `combining_two_versions_of_the_same_file`,
    `uploading_big_files_kept_outside_git`.
16. `extra_working_folders_for_one_project`:
    Git can give one project several working folders ("worktrees");
    this makes new ones quickly and copies over the files Git ignores.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `understanding_the_git_command_you_typed`,
    `running_the_real_git_program`,
    `your_settings_for_this_wrapper`,
    `checking_your_changes_before_they_are_saved`,
    `talking_to_the_background_commit_helper`,
    `instant_folder_copies_on_btrfs_disks`,
    `fixing_installed_tools_in_a_copied_project_folder`,
    `what_happens_when_you_type_git_push`.
17. `keeping_what_a_command_would_throw_away`:
    some commands throw away work that is not committed yet,
    or keep one side of a file two changes disagree on;
    this saves a copy first and records what happened.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `understanding_the_git_command_you_typed`,
    `running_the_real_git_program`,
    `your_settings_for_this_wrapper`,
    `checking_your_changes_before_they_are_saved`,
    `talking_to_the_background_commit_helper`,
    `a_hidden_copy_of_git_data_for_preparing_commits`,
    `instant_folder_copies_on_btrfs_disks`.
18. `running_every_check_in_order`:
    runs all the checks one after another,
    including the ones that look at the saved copies.
    Uses:
    `understanding_the_git_command_you_typed`,
    `running_the_real_git_program`,
    `your_settings_for_this_wrapper`,
    `checking_your_changes_before_they_are_saved`,
    `keeping_what_a_command_would_throw_away`.
19. `what_happens_when_you_type_git_merge_or_git_pull`.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `understanding_the_git_command_you_typed`,
    `running_the_real_git_program`,
    `waiting_your_turn_to_change_git_files`,
    `your_settings_for_this_wrapper`,
    `checking_your_changes_before_they_are_saved`,
    `talking_to_the_background_commit_helper`,
    `combining_two_versions_of_the_same_file`,
    `what_happens_when_you_type_git_push`,
    `keeping_what_a_command_would_throw_away`.
20. `checking_that_your_setup_works`:
    the `git cli-git doctor` report.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `running_the_real_git_program`,
    `your_settings_for_this_wrapper`,
    `checking_your_changes_before_they_are_saved`,
    `instant_folder_copies_on_btrfs_disks`,
    `keeping_what_a_command_would_throw_away`.
21. `the_background_commit_helper`:
    the background program itself:
    it saves commits one batch at a time,
    then pushes them and tidies up.
    Uses:
    `saving_files_starting_programs_and_keeping_time`,
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`,
    `waiting_your_turn_to_change_git_files`,
    `running_git_push_with_a_time_limit`,
    `your_settings_for_this_wrapper`,
    `checking_your_changes_before_they_are_saved`,
    `talking_to_the_background_commit_helper`,
    `a_hidden_copy_of_git_data_for_preparing_commits`,
    `uploading_big_files_kept_outside_git`,
    `what_happens_when_you_type_git_push`,
    `extra_working_folders_for_one_project`,
    `keeping_what_a_command_would_throw_away`.
22. `what_happens_when_you_type_git_commit`.
    Uses:
    every folder from 1 to 21 by name except `waiting_your_turn_to_change_git_files`,
    `running_git_push_with_a_time_limit`,
    `instant_folder_copies_on_btrfs_disks`,
    `fixing_installed_tools_in_a_copied_project_folder`,
    `what_happens_when_you_type_git_push`,
    `what_happens_when_you_type_git_merge_or_git_pull` and `checking_that_your_setup_works`.
23. `deciding_what_to_do_with_each_command_you_type`:
    the first stop of every command:
    reads what was typed,
    then hands it to the real Git,
    runs one of the other folders,
    or refuses and says why.
    Uses every other folder.

The list numbers here only order this draft;
they appear in no folder name.
Code that only tests use goes in `setup_code_for_tests`;
`lib.rs` and `main.rs` stay at the crate root.
The folders sit directly in `src/`:
today's `src/native/` is named by a word that fits any code (`AGENTS.md` VGW),
so it goes away.
The large folders (`checking_your_changes_before_they_are_saved`,
`extra_working_folders_for_one_project`,
`the_background_commit_helper`,
`what_happens_when_you_type_git_commit`) get folders inside them the same way,
each with its own `uses/`.

### The `uses/` links and cargo

Measured on 2026-11-04 with cargo 1.101.0-nightly (31d3246e0 2026-10-27) on a throwaway crate
(`src/top/uses/middle -> ../../middle`,
`src/middle/uses/bottom -> ../../bottom`):
`cargo package --list` follows the links and packs nested copies
(`src/top/uses/middle/uses/bottom/mod.rs`),
so across the wrapper's many folders the published crate would hold the same files many times over.
With `exclude = ["**/uses"]` in `Cargo.toml` the list holds each file once,
and the crate builds.
The wrapper's `Cargo.toml` is managed by file-enforcer (`CLAUDE.md` WC2),
so the line goes into its source.
Other tools that follow links (the repository linter's walker,
editors' indexers) are checked when the links are built.

## Oversized files

The 37 files over the `rust/max-lines` limit overlap with the file loops
(`transaction_git`,
`service_land`,
`child_watch`,
the `index.lock` files,
`bounded_push`,
`service_work`).
They are split now by the lint agents (branches `refactor/git-policy-cli-lint-*`,
from `7663284896`),
each given its folders' loop list so a split moves shared types into a file of their own and breaks a loop
rather than adding one.
The layout pass then renames and moves the resulting files;
the scan reruns on the merged result before the owner sees the tree.

## Next

The target tree,
crate split,
named changes,
move order and open points are drafted for the owner in
[`cli-git-source-layout-target-tree.md`](cli-git-source-layout-target-tree.md)
(file by file in [`cli-git-source-layout-target-tree-files.md`](cli-git-source-layout-target-tree-files.md)).
Nothing moves until the owner answers its open points.
Then the file loops are broken with the named changes,
the large groups are split into folders,
and every folder and file is named for someone who knows only `git commit` and `git push`.
