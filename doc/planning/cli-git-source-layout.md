# Source layout of the Git wrapper crate (issue 6261), second design

Status: analysis in progress, nothing moved.
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
   number the siblings,
   and write each folder's top doc.
6. Show the owner the whole tree (old name, new name, one-line explanation, layer) before any file moves.

## Draft groups, from the bottom (working names, not the final names)

1. `basic_tools`: files, processes, time, logging, parallel work (today `foundation/`, `process/`, plus `child_watch`,
   `child_class`, `start_retry`, `wait_ceilings`, `tcp_sockets`, `journald_native`, `fs_mount_options`,
   `child_environment`).
2. `git_files`: reading Git's own files without starting Git (index format, refs, objects, packs, trees, configuration
   text; `worktree_identity`, `transaction_journal`).
3. `typed_command`: reading the command the person typed (`global_arguments`, `wrapper_controls`, `escape_hatch`,
   `forwarded_command`, `wrapper_invocation`, `git_builtins`, `announced_query`, `effective_target`, today's `command/`).
4. `start_git`: finding and starting the real Git (`real_git*`, `started_as_git`, `hooks_off`, `forwarding`,
   `git_version*`, `transaction_git`, `git_metadata`, `repository_location`, `repo_read_hook`, `repo_read_query*`).
5. `locks`, 6. `settings`, 7. `checks` (policies and the files they look at),
8. the commands (`shadow`, `subvolume`, `worktree_copying`, `worktree`, `lfs`, `push`, `conflict`, `merge`, `commit`,
   `repair`, `health`), each with its own number among siblings,
9. `service`, 10. `decide` (the top-level decision for one invocation), then test helpers and the crate root.

## Findings so far

The suggestion pass on that draft reports 12 plain moves and 56 knots.
Most knots come from two shapes:

- One folder holds a client and a server.
  `service` (today `service/` and `landing/`) holds both what a command needs to talk to the background service
  (protocol, messages, instance, review key, timing, journal) and the service itself (landing commits, after-commit
  work), which uses the commands.
  Split it into a client group below the commands and a server group above them.
- Shared parts sit in the folder of one user.
  - `bounded_push`,
    `push_progress` and `wait_progress` are a general "stop a program that makes no progress" tool kept under `push/`,
    and basic code uses them;
    the general part moves down and the push-specific lock clearing stays with push.
  - The `service_index_*` modules are the handling of Git's `index.lock`,
    which locks and merges use;
    they move to the locks group.
  - `transaction_git` (the one runner of real Git) reaches up into `add_timing` (worktree) and `service_busy` (service)
    for two small types;
    those types move down,
    or the runner takes a value from its caller.
  - `child_watch` (a basic process watcher) calls the stale reference lock clearing (`stopped_git_locks`);
    the clearing moves to the caller.
  - `repo_read_layout` (a reader of Git's files) calls `repo_read_hook` (the runner's in-process shortcut);
    one asker that tries the readers first and falls back to running Git replaces the two-way use,
    so readers never start Git and the runner never calls a reader.

## Next

Apply the client and server split,
move the shared parts down,
rerun the suggestion pass,
and turn every remaining knot into a named code change with the files it touches.
