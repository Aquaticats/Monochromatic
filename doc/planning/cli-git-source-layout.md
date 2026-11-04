# Source layout of the Git wrapper crate (issue 6261), second design

Status: analysis in progress, nothing moved;
a group order with no upward reference is found (section "Group order found, from the bottom").
Started 2026-11-04 by the coordinator session after the owner stopped the child session
("I told the layout subagent to stop because it couldn't ever produce something good enough. Lets do this ourselves.").
The child session's third proposal (branch `refactor/git-policy-cli-source-layout`, `671eb1c6d1`) is not used;
only its measured edge data was looked at.

## What the owner asked for

Owner, 2026-11-04, verbatim:

- "Principle 1: Lint issues need to be taken care of asap."
- "Principle 2: every name of a dirent must be understandable to a monkey. If jargon is unavoidable, the file's top doc must explain everything, in a way that would make any monkey understand."
- "Principle 3: Who depends on who must be immediately visible from looking at the file structure."
- "Consider monkey someone who only knows what git commit and git push are."
- "Don't discard work, but aside from layout + linting fixes, pause everything else - but let work already in flight run to their natural conclusions. Layout + linting fixes are the biggest priority."

Question tool answers, 2026-11-04:

- Structure, first answer: "B: numbered layers (Recommended)",
  with the note "Don't be afraid of using extremely long names. B, but make sure a monkey even just looking at one thing know what it means."
  Shown a sketch with two-digit layer numbers (`01_tools_for_files_programs_and_time/` to
  `11_deciding_what_to_do_with_one_git_command/`),
  the owner asked:
  "What if I still think this is pretty un-understandable? What is 01 to 11 supposed to mean?"
  The numbers meant "a folder uses only folders with a smaller number",
  which no name says on its own.
  Second answer, replacing the numbers: "A: a uses/ list in each folder (Recommended)".
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
- Granularity: "Files too, checked (Recommended)":
  files inside one folder never use each other in a loop,
  the folder's top doc lists them in dependency order,
  and the package's lint fails any file or folder that breaks its layer.
- Scope: "Repo-wide rules (Recommended)": AGENTS.md MKY, LYD and LYF (`b1955e0b49`).
- TypeScript in the crate: "A: port all to Rust (Recommended)" (separate work, paused).

## What the code looks like now

Measured on main at `2d82a3beb4`
(tools and outputs in `~/temp/agent/issue-6261/`: `scc.ts`, `fas.ts`, `modules.ts`, `groups.ts`, `suggest.ts`, and `scc/`).
A reference is a `crate::<module>` or `super::<module>` path in code, with comments and string literals blanked;
test files (`*_tests.rs`) are left out because nothing in production uses them.

- 554 production modules, 2,298 references between them.
- 266 modules sit in one dependency loop (plus loops of 4 and 2).
  A greedy order of the loop leaves 82 references pointing backwards;
  that is an upper bound on how many references must change, not a plan.
- Every folder is re-exported flat at the crate root (`pub use <folder>::*;` in `src/native/lib.rs`),
  so every file names every other as `crate::<module>`,
  and the folders say nothing about who uses whom.
  These re-exports go:
  after the move every `use` names the folder path.

## Method

1. Put each module in a target group by what it does.
2. Give the groups an intended order from the bottom (basic tools first, the command-line entry last).
3. For every reference from a lower group to a higher one, decide:
   MOVE when the used module depends only on things below its users (it is simply in the wrong folder),
   KNOT when it also depends on something at or above its users (code must change:
   move a type or constant down, pass a value in, or split a module).
4. Repeat until no reference points up and no two groups loop;
   then the same inside each group, file by file.
5. Name every folder and file for someone who knows only `git commit` and `git push`,
   write each folder's top doc,
   and list in each folder's `uses/` the folders its code uses.
6. Show the owner the whole tree (old name, new name, one-line explanation, what it uses) before any file moves.

## Group order found, from the bottom (working names, not the final names)

Measured on main at `6f731f15ca` (2026-11-04T18:12:05-05:00, before the branch-write merge `de36a92311`),
with the item-level scanner `~/temp/agent/issue-6261/refs.ts`,
the crossing report `crossings.ts` (with a search for the group order that leaves the fewest upward references)
and the in-group loop report `inside.ts`.
The group map is committed beside this file as
[`cli-git-source-layout-groups.json`](cli-git-source-layout-groups.json):
`folders` and `modules` place each file in a group,
`items` moves single items (a type, a constant, a function) to another file,
and `(removed)` marks a reference that a named code change takes away.

The scanner now also blanks character literals (`'"'` hid code up to the next quote, missing five references)
and leaves out test-only modules:
`*_tests.rs` files and every module whose `mod` line carries a `cfg` attribute naming `test`,
including ones declared inside a production file through `#[path]` (`index_reset_support`).
Test helpers live with the tests they serve and are not part of the shipped order.

Under this map the groups use each other in one direction only: no reference points from a lower group to a higher one.
From the bottom:

1. `basic_tools`: files, processes, time, logging, parallel work, watching a child for progress
   (today `foundation/`, `process/`, plus `child_watch`, `wait_progress`, `push_progress`, `add_timing`,
   `subvolume_sha`, `lfs_output_mask`, `shell_quote`, `client_journal`, `service_fault`, `service_durable`).
2. `git_files`: reading Git's own files without starting Git.
3. `typed_command`: reading the command the person typed, including the list of policy names (`policy_registry`).
4. `start_git`: finding and starting the real Git, the readers that may start it, `forwarded_command`,
   `worktree_identity`, `landing_location`.
5. `locks`: Git's lock files, including the service's hold on `index.lock` (`service_index_*`).
6. `push_runner`: running `git push` with a deadline and one push at a time
   (`bounded_push`, `bounded_query`, `auto_push_single_flight`, `push_lock_clearing`, `push_deadline_advice`).
7. `settings`: the wrapper's own configuration, including the discarded-copies section (`config_discarded`).
8. `checks`: the policies and the files they read, the rule modules from `command/` and `commit/`, `diagnostics`,
   the hook warnings, the standards review key.
9. `service_talk`: what a command needs to talk to the background commit service
   (protocol, instance, messages, provisioning, client, route, work records, claim time, timing, handoff,
   landed summary, commit timing, pruned content, `inner_git_words`).
10. `merge_engine`: the merge machinery that push and shadow reuse (second pass, isolation journal and recovery).
11. `shadow`, then `subvolume`, `repair`, `lfs`, `push`, `worktree` (with today's `worktree_copying/`), `conflict`.
12. `checks_runner`: `policy_checks`, which runs every check and calls the conflict judges.
13. `merge`, `health`, then `service_server` (the service itself: landing, after-commit work, index reset),
    `commit` (with the commit-through-the-service files), and `decide` (the top-level decision,
    `index_writer_forward`, `worktree_copy_forward`).

The order puts `commit` above `service_server`:
the commit command reuses landing pieces (`update_index`, `IndexPaths`) in its own process,
and nothing in the service uses the commit command.

## Named code changes the order needs

Moving whole files to the group that matches what they do accounts for most of the change.
The rest are items that move to a lower file, plus five changes to how code is called.

### Items that move down (from the group map's `items`)

- Process outcome types and the subcommand reader (`ChildOutcome`, `outcome_of`, `subcommand_of`) to `child_class`.
- Machine and boot identity (`current_boot_id`, `current_machine_id`, `is_machine_id`) to `process_identity`.
- The service unit name and its directory to `real_git_record`; `remember_own_executable` to `real_git`.
- `LANDING_ATTEMPTS` and `Bind` to `service_protocol`; `work_directory` and `Verdict` to `work_item`.
- `capture_stamp` to `claim_time`; `effective_directory` to `effective_target`; `WorktreeIdentity` to `effective_target`.
- The sequencer facts (`SequencerFacts`, `SequencerState`, `sequencer_state`) to `repo_read_adapter`.
- The in-process reading switch (`repo_read_hook::enabled`) to the readers.
- Conversion facts (`ConversionEnvironment`, `SystemAttributes`, `CLAIM_PREFIX`) to the subvolume files.
- `ForwardedRun` and `run_forwarded_git` to `forwarded_command`;
  `CoordinatedForward` and `CoordinatedRun` to the index-lock wait;
  `PushForward` to `push_stacking`.
- Small helpers to the basic tools: `append_line`, `read_plain_file`, `copy_file_exclusive`, `STOP_GRACE`,
  `ENGINE_FAILURE_EXIT_CODE`.
- Configuration warning rendering (`render_configuration_warning`, `render_engine_failure`, `EngineFailureCode`)
  to the settings group; `Limits` and `parse_size` to `config_discarded`.
- `FailureCode` and `storage_failure` to the index-lock files; `ShippedChecks` to `policy_pass`.

The simulation does not follow what a moved item itself uses;
the compiler checks that when the item really moves.

### Changes to how code is called

- The push of an integrated merge (`stacked_merge_push`) plans a whole new invocation through `entry::plan_invocation`,
  and the clean finish of a merge (`merge_second_pass_finish`) runs a nested command through `entry::run_process`.
  Both are the top level called from below, and both are recursion inside one process.
  Proposed: they return "run this command next" to the top level, which runs it in its loop.
  This follows the owner's ban on the wrapper invoking itself; open to veto.
- The one runner of real Git (`transaction_git`) tells the service which step it is in
  (`service_busy::enter_step`, `on_landing_thread`).
  Proposed: a small "current step" record in the basic tools that the runner writes and the service reads.
- The process watcher (`child_watch`) sweeps stale Git locks itself (`stopped_git_locks::LockLedger`).
  Proposed: its caller hands it what to do after a stop.
- The configuration reader (`repo_read_config_scan`) asks for the system configuration path,
  which can start real Git (`git_system_config::system_config_for`).
  Proposed: the caller passes the path in, so a reader never starts Git.
- Commands that only finish (`subvolume_command`, `doctor`, `worktree_convert_command`, `stacked_merge_push`)
  build the top-level `Action`.
  Proposed: they return a small "finished with this code and output" value from a low file;
  only the top level builds `Action`.

## Loops between files inside one group

With the map applied, the groups have no loop between them,
but 27 loops between files remain inside groups, touching 79 files (`~/temp/agent/issue-6261/scc2/inside.txt`).
Most share one shape: a file and a helper split off from it each use the other's types.
The usual fix is a third file at the bottom of the folder that holds the shared types,
used by both.
Some need a closer look at the functions,
for example the placement, reclaim and claim files of the `index.lock` hold, which may call each other in a cycle.

## Next

Break the file loops with named changes,
then split the large groups (`checks` holds 66 files, `worktree` 41) into folders,
name every folder and file for someone who knows only `git commit` and `git push`,
and show the owner the whole tree before any file moves.
