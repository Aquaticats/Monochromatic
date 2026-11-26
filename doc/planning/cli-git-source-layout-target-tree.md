# Target source tree of the Git wrapper (issue 6261), proposal for the owner

Status:
proposal only,
nothing moved,
renamed or edited in the wrapper's source.
The owner rejected the draft crate split of 46 libraries on 2026-11-07T02:03:01-05:00
("Not good enough,
assume every crate must stand on its own
and no one else is going to use our crates to build a git wrapper."),
and then tightened the rule ("Assume no one else is going to use anything related to git from us.").
Section "The crate split" is redone against both:
eight libraries,
a ninth for tests,
and the program crate with everything Git-related inside it as folders.
No file moves until the owner confirms the redone tree.
Updated 2026-11-07T10:43:12-05:00 after the owner adopted published crates for several Git-free jobs
and answered "One level of groups":
the rescan of `main` (section "Rescan of main after the adopted crates"),
the check of each library against published crates (section "The eight libraries against published crates"),
the program's group folders and the options for the rule that relates a group to its children
(section "One level of groups in the program"),
the open points of the test helpers and of where the libraries live brought to today's facts,
and the third review.
Updated 2026-11-26T02:00:00-05:00 after the rustfmt reformat and the layout rules:
the rescan of `aba76005c3` (section "Rescan of main after the reformat (2026-11-26)"),
where each move-order step stands (section "Where each step stands today (2026-11-26)"),
the 67 file loops and one folder loop with a break for each,
and the round order that replaces "Next".
Nothing in the tree moved or was decided by it.
Drafted 2026-11-04 for the owner's review,
on top of `doc/planning/cli-git-source-layout.md`
(the owner's words and the group order live there).
The per-file tree is in [`cli-git-source-layout-target-tree-files.md`](cli-git-source-layout-target-tree-files.md);
the machine-readable map is [`cli-git-source-layout-groups.json`](cli-git-source-layout-groups.json) (key `target`).

## What to look at and how to answer

- Purpose:
  decide whether this tree,
  the eight libraries plus the test-folders library and the program,
  and this move order are what the owner wants before any file moves.
- What changed since the previous version of this file:
  the owner's two rulings of 2026-11-07 removed 38 of the 46 libraries;
  they fold into the program crate as folders with `uses/` links.
  Every library that stays had to give up its Git-bound files and some of its names,
  so seven named changes were added (section "Changes so that a library stands alone").
  The stand-alone test used to judge them is the coordinator's reading of the owner's words,
  not something the owner approved.
- Added 2026-11-07T10:43:12-05:00:
  the verdict of each library against the crates the owner adopted and other published crates,
  26 draft entries for the program's one level of groups,
  and three options for the rule the owner must word about a group folder and its children.
  Nothing is decided in them;
  the two open points the owner answered "Not yet."
  (test helpers,
  where libraries live) are only updated to today's facts.
- Inspect first:
  the stand-alone test and the verdicts (section "The crate split"),
  what each of the eight changes (same section),
  and the open points (section "Open points for the owner").
- Respond by:
  accepting or vetoing each open point;
  a veto on a crate or a name can be answered in one line,
  because the tree is generated from one table kept in `cli-git-source-layout-groups.json` under `target`.
  The first open point asks which of the eight libraries to keep,
  so a one-line answer ("keep the four with tier-one evidence") settles most of the file.

## The owner's words and the rules that bind this

The owner,
2026-11-04,
verbatim:

- "Principle 2:
  every name of a dirent must be understandable to a monkey.
  If jargon is unavoidable,
  the file's top doc
  must explain everything,
  in a way that would make any monkey understand."
- "Principle 3:
  Who depends on who must be immediately visible from looking at the file structure."
- "Consider monkey someone who only knows what git commit and git push are."
- "Layout + linting fixes are the biggest priority."
- On crates:
  "We should not put everything in one crate."
  and
  "Crates obviously have to be reusable,
  otherwise there's no point.
  Also I refuse to believe C means splitting into fewer crates."
- Where crates live:
  each is its own package `package/git-policy/<name>` (crate `monochromatic-git-policy-<name>`),
  its own Cargo workspace root,
  depending on the others by `path` with an exact version pin;
  (Open again on 2026-11-07:
  the libraries' path is open point "Where the libraries live and what they are named".)
  "A:
  Cargo.toml between crates,
  uses/ inside (Recommended)".
- 2026-11-07,
  on the drafted split of 46 libraries:
  "Not good enough,
  assume every crate must stand on its own
  and no one else is going to use our crates to build a git wrapper."
  So a crate whose only other user would be a Git wrapper,
  a front end for the commit service,
  a push service or a commit-landing service does not count as standing on its own.
- 2026-11-07T02:14:26-05:00 (the time this proposal recorded it,
  shortly after the owner said it),
  tightening the same rule,
  verbatim:
  "Assume no one else is going to use anything related to git from us."
  So a crate whose job is about Git has no outside user however general it is,
  and only jobs with no Git in them can stay separate crates.
- 2026-11-07,
  on one shared build folder (`CARGO_TARGET_DIR`) for the split packages:
  "One shared build folder,
  but first,
  I'd like you to confirm with me each pkg can stand on its own itself."
- 2026-11-07,
  where releases go:
  "deploy all releases on pnpr,
  but only publish rc and stable releases on crates.io"
  (versions ending `-alpha.N` or `-beta.N` go to pnpr only;
  `-rc.N` and plain numbers go to pnpr and crates.io;
  owner's pick "Label decides").
- 2026-11-07T02:44:49-05:00,
  on the open points of the redone split:
  - Which Git-free jobs become libraries:
    no option picked;
    the owner asked instead,
    verbatim,
    "Why are we not using already established crates for some functions?"
    So each Git-free job is first checked against published crates;
    a job a vetted crate already does needs no crate of ours.
  - The test helpers:
    "Not yet."
  - The program's folders:
    "One level of groups"
    (one level of group folders in the program,
    so a rule for how a group folder and its child folders relate is still needed from the owner).
  - Where the libraries live and what they are named:
    "Not yet."
  - Rescan before step 0:
    taken by the coordinator without asking (option A),
    because the measured zero loops describe the tree of `f44d921e24`,
    not today's `main`.
- 2026-11-07T11:05:21-05:00,
  after the rescan and library verdicts (section "The eight libraries against published crates"),
  in the question tool:
  - How a group folder relates to its children:
    "Shelf + summaries (Recommended)".
    A group folder holds only its child folders and a top doc;
    each child links exactly the folders it uses,
    across groups too;
    each group also gets a `uses/` folder that summarizes which other groups or one-folder entries its children use.
    The summaries need a change to `monochromatic-lint`,
    which today rejects a link no code in that folder uses.
  - How many Git-free libraries stay:
    "Four libraries (Recommended)":
    `starting-and-watching-programs`,
    `parallel-work-within-a-budget`,
    `fix-packages-after-copying-a-folder`,
    `dependent-version-bump-check`.
    The rest of `safe-file-saving`,
    `linux-file-and-disk-calls`,
    `text-time-and-log-line-formats` and
    `talking-to-the-systemd-user-manager` folds into the program.
  - How a library writes log lines:
    "tracing (Recommended)":
    libraries emit through the `tracing` crate;
    the program installs the output,
    keeps today's line format and the URL credential masking in that output layer.
  - Settled by the coordinator from those answers (not asked):
    `shell_quote` (kept as ours by the earlier owner pick) lives in the lowest crate that uses it,
    so the program uses it from there;
    the exclusive copy that `private_storage::copy_file_exclusive` gives a library is specified on the adopted calls
    (create-new open,
    copy,
    sync),
    never `std::fs::copy`,
    which replaces an existing file.
  - Settled by the coordinator when the group checks merged (`62874c227d`),
    from "One level of groups" and "Shelf + summaries (Recommended)" (not asked):
    the linter knows a group by its structure,
    any folder holding child source folders,
    and needs no marker line;
    under those two answers only a group holds folders,
    so the structure already says it,
    and a marker that someone forgets would switch the group checks off.
    The one-folder entries' own `uses/` folders link the exact folder inside a group,
    never the group,
    because a one-folder entry is a folder like a group's children;
    the entry lists in "One level of groups in the program" that name a group for a one-folder entry are read that way
    and are reworded when the move reaches them.
  - On text identical to JavaScript output (the job of `text-time-and-log-line-formats` as drafted),
    verbatim:
    "\" text identical to JavaScript output\" - that's not needed."
    So no job keeps byte-identity with JavaScript or the old TypeScript tools as a requirement;
    a reader of a format that is still in the repository is listed before that format changes.
  - On the published-crates study (`doc/research/wrapper-git-free-jobs-and-published-crates.md`),
    picks verbatim:
    replace with published crates "Time stamps:
    jiff,
    Digests:
    sha1 + sha2,
    Linux calls:
    rustix,
    Memory headroom:
    sysinfo";
    how many library crates of ours remain:
    "I'm not ready to answer this specific question yet.";
    D-Bus:
    "zbus 6,
    and I suppose the lack of a blocking api can be worked around."
    (to be proven inside the wrapper before it is built on);
    shell quoting:
    "Keep ours (readable)".
    The replacements are built after the release.
  - Then,
    verbatim:
    "Adopt tokio,
    do not adopt ureq,
    use the `hyper` set of https libs."
    So the wrapper gets tokio as its async runtime (zbus 6 runs on it,
    no blocking workaround needed),
    and the standards-review HTTPS call moves from `ureq` to `hyper`,
    `hyper-util`,
    `hyper-rustls` and `http-body-util`
    (all already in `rust-catalog.toml`).
    Which tokio runtime flavour is chosen by measurement inside the wrapper.
  - Then,
    on the runtime:
    picked "Multi-thread,
    sized to budget"
    (worker count from the wrapper's existing budget for parallel work),
    and verbatim:
    "Full on tokio.
    Multi-thread."
    So the wrapper moves to tokio as a whole,
    not only for D-Bus and HTTPS;
    the migration plan is `doc/planning/git-policy-cli-full-tokio.md`,
    and nothing is rewritten before the owner
    has seen it.
  - On the toolchain,
    verbatim:
    "Rust nightly should always be latest."
    Moving the lock and keeping it latest:
    `doc/planning/rust-nightly-always-latest.md`.

`AGENTS.md`,
verbatim:

- MKY:
  "Name every file and folder so someone who knows only `git commit` and `git push` understands it alone;
  long names are fine;
  jargon nothing can replace is explained in that file's top doc."
- VGW:
  "Each file and folder name says what its contents do;
  a word that fits any code (`native`,
  `core`,
  `helpers`)
  never stands alone:
  `helpers_for_git_push`,
  not `helpers`.
  Package folders are exempt."
- LYD:
  "Show who depends on whom in the tree:
  every source folder inside a crate holds a `uses/` folder of links to the
  folders its code uses."
- LYL:
  "Two source folders never use each other,
  and the package's lint fails when a folder's `uses/` folder of links and
  its code disagree."
- LYC:
  "A crate lists the crates it uses in its `Cargo.toml` dependencies,
  never in a `uses/` folder;
  cargo refuses loops
  between crates."
- LYF:
  "Files inside one folder never use each other in a loop;
  the folder's top doc lists them in dependency order,
  and
  the package's lint fails any loop."

Decision records that bear on it:

- `doc/decision/rust-dependency-catalog.md`,
  line 15:
  "Every Rust crate is its own Cargo workspace root".
  So no workspace spans the new packages,
  and no `[workspace.dependencies]` is shared;
  each new package carries its own `Cargo.toml`,
  `Cargo.lock` and `target` folder.
- `.out-of-scope/cargo-workspace.md`:
  profiles belong to each package,
  which is why a repository-wide Cargo workspace
  is out of scope.
- Since 2026-11-05 the split crates were to be `publish = false` (`doc/planning/cli-git-source-layout.md`,
  end of section
  "Splitting into crates",
  and `doc/planning/crates-io-footprint.md`).
  Superseded 2026-11-07:
  every release of every crate goes to pnpr,
  and only `-rc.N` and plain-number releases also go to crates.io.
  The wrapper's one crates.io release still happens before it depends on a crate crates.io does not hold.

## Rescan of main

Measured 2026-11-04T20:45:14-05:00 on `main` at `f44d921e24` (contains the lint merge `977e697ad5`),
with the planning doc's scanners copied unchanged into the scratch folder
(`~/temp/agent/wrapper-target-tree/scan/`:
`refs.ts`,
`crossings.ts`,
`inside.ts`,
`modules.ts`)
plus three new ones for this proposal (`engine.ts` for the crate,
folder and file graph,
`testrefs.ts` for test references,
`reexports.ts` for named re-exports).

- Production modules scanned:
  627 files;
  2607 module-to-module references;
  5456 item-level references.
  The test side:
  604 `*_tests.rs` sidecars and 29 test-only modules.
- Every file the lint split added was placed by the name of the file it was split from
  (25 files),
  except four placed by what they hold.
  Before that,
  35 references pointed upward between groups;
  after it,
  zero.
  Four decisions came out of it:
  - `push_deadline_and_patience_settings` is a basic tool (child watching needs it),
    not a push file;
  - `conflict_side_state` moves down to the readers of Git's files (the merge code needs `conflicted_paths`);
  - `editor_would_open` and `stdin_and_stdout_are_one_terminal` move from `merge_isolation_git_reads` to
    `merge_second_pass_git_runner`;
  - the group order is corrected to the planning doc's text (`push` before `worktree`);
    the old JSON had them swapped.
- The group map is updated in place (`cli-git-source-layout-groups.json`):
  157 module placements,
  the item moves of the planning doc re-keyed to the files that now define them,
  the new moves of this proposal,
  and the new key `target` that gives every production module its crate,
  folder and new file name.
- File loops between files of one group:
  17 loops touching 53 files (the planning doc said 27 and 79 before the
  lint split).
  Every one is listed with its edges in section "Loops between files,
  as they stand".

### Lint baseline

The repository's installed `monochromatic-lint` does not know the two layout rules yet,
so the linter was built from
`package/linter/monochromatic-lint` on `main` into the scratch folder
(`mise run //package/linter/monochromatic-lint:build:debug` with `CARGO_TARGET_DIR` set there),
and run on the wrapper with a throwaway configuration that switches both rules on
(`package/git-policy/cli/**/*.rs`,
severity error),
2026-11-04T20:47:48-05:00,
`main` at `f44d921e24`:

- `rust/folder-uses-links`:
  317 findings:
  288 folders use another folder without a link,
  28 flat `pub use folder::*;` glob re-exports,
  and 1 folder loop (the crate root and every folder).
- `rust/file-order-in-folder`:
  41 findings:
  27 folders without a file list in their top doc,
  and 14 file loops.
- Total 358.

A fact the owner may trip on:
the linter's README shows the file-list marker split over two lines,
but the rule reads
the exact single line `Files in this folder, each using only files listed before it:`
(`package/linter/monochromatic-lint/src/layout_top_doc.rs`,
`LIST_MARKER`).
Measured on a throwaway crate (scratch folder `rootcheck/`):
the two-line form reports "no file list",
the one-line form
passes.
The same experiment shows a crate root `lib.rs` that holds only `pub mod x;` lines needs no `uses/` folder,
and adding `pub use x::Thing;` to it makes the root use `x` and demands `src/uses/x`.
So every `lib.rs` in the target tree holds only `mod` lines and its top doc,
with no re-exports.

## Rescan of main after the adopted crates

Measured between 2026-11-07T10:26:36-05:00 and 2026-11-07T10:43:12-05:00 on the wrapper source of `e728c63911c046eed2c66f055aed26bf08d797ac`,
the base of the branch `docs/git-policy-cli-layout-after-adopted-crates`.
`main` was at `dcd52b9fb9` while this ran;
`git diff --stat e728c63911 main -- package/git-policy` prints nothing,
so the wrapper source of the two commits is the same.
This commit holds tokio steps T1 to T5 (`48611f7e09`) and the swaps to `tempfile`,
`jiff`,
`sha1`,
`sha2`,
`sysinfo` and `rustix`.
`hyper` and `zbus` 6 are adopted by the owner's pick and are not built into the wrapper yet
(`Cargo.toml` still lists `ureq` and the hand-written D-Bus client is still the one in `systemd_dbus.rs`).

### Method

- The scanners are the ones of the section "Rescan of main",
  copied unchanged from `~/temp/agent/wrapper-redo/scan/` into `~/temp/agent/wrapper-layout-after-crates/scan/`:
  `refs.ts`,
  `testrefs.ts`,
  `reexports.ts`,
  `modules.ts`,
  `unmapped.ts`,
  `inside.ts`,
  `engine5.ts`,
  `testcheck5.ts`
  and `moveduses.ts`.
  Two scanners hard-code the folder they read (`reexports.ts`,
  `moveduses.ts`),
  so one path constant in each copy was changed to the worktree of this branch;
  nothing else in a scanner changed.
- The group map is `groups8.json` of the redo plus the named changes of `spec/changes.txt`,
  as in the redo;
  `spec/spec6.txt` is the redo's placement table (`spec5.txt`) with the placements of this section added.
- The scanner treats a module as test code only when a `cfg(test)` line says so.
  The sixteen support files of the `native_binary` test target (`binary_*` without `_tests`),
  `fixture_executable`,
  `test_alive` and the two crate roots (`lib`,
  `main`) have no such line,
  so they were added by hand to the scratch list of test-only modules.
  They start the built program or are crate roots;
  none is a production module.
- The lint baseline ran with the installed `monochromatic-lint` 0.5.1,
  which lists both layout rules (`monochromatic-lint --rules`),
  with the throwaway configuration of the first baseline
  (`rust/folder-uses-links` and `rust/file-order-in-folder`,
  severity error,
  on `package/git-policy/cli`).
  The first baseline needed a linter built from source;
  this one did not.
- Every command ran in its own `systemd-run --user --scope` with `CPUQuota`,
  `MemoryMax`,
  `TasksMax` and `IOWeight=10`.
  No load was generated.

### What the scan counts now

- Production files scanned:
  653 (627 at `f44d921e24`);
  module references 2722 (2607);
  item references 5683 (5456);
  sidecar test files 632 (604).
- Loops between files of one group,
  as the files stand (`inside.ts`):
  17 loops touching 53 files,
  the same set as before.
- The lint baseline:
  362 findings (358 before).
  `rust/folder-uses-links` has 321:
  292 findings of a folder that uses another folder without a link,
  28 flat `pub use folder::*;` re-exports and 1 folder loop.
  `rust/file-order-in-folder` has 41:
  27 findings of a folder without a file list in its top doc and 14 file loops.
  One of the 14 file loops is one the scan cannot see:
  `discard_audit_generations` is declared inside `discard_audit` by `#[path]` as the child module `generations`,
  `discard_audit` names `generations::Retention`,
  and the child reads `super::AUDIT_FILE`.
  The scanner reads `crate::` paths and `super::<module>` paths,
  and `super::AUDIT_FILE` names an item of the parent,
  so the scan has no edge between the two files (`scan/item-edges.tsv` holds none).
  The linter sees it.
  This is a blind spot of the scan that the first review already listed ("`self::`,
  macros,
  trait impls"),
  and a seventh named change is needed:
  `AUDIT_FILE` moves into `discard_audit_generations`,
  or `retention_of` takes the file name as a value.

### Files added since the first scan, placed

Twenty-two production modules were not in the placement table
(one of them,
`runtime_probe`,
is a test-only entry that lives in production code).
The four the proposal expected to join libraries did:
`side_by_side_threads`,
`rename_that_never_replaces`,
`line_sink` and `call_catching_panics`.
The placements in this list are proposals,
and the folder names are working names that the owner's veto round settles;
several fail MKY alone today (names to revisit are listed in the section "One level of groups in the program").

- Join `parallel-work-within-a-budget`:
  `side_by_side_threads`,
  `call_catching_panics` and `line_sink`
  (folder `running_independent_jobs_side_by_side`);
  `runtime_limits`,
  `runtime_plan`,
  `lazy_runtime` and `blocking_job`
  (a second folder,
  `the_shared_runtime_that_runs_waiting_jobs`).
  The tokio plan says this library owns the budget and the runtime-building policy.
- Join `starting-and-watching-programs`
  (folder `starting_programs_catching_signals_and_keeping_their_output`):
  `captured_child_run_to_its_end` and `child_awaited_through_a_pidfd`.
- Join `linux-file-and-disk-calls` (folder `fast_copies_and_swaps_on_linux`):
  `rename_that_never_replaces`.
- Join `safe-file-saving` (folder `saving_files_so_a_crash_leaves_no_half_file`):
  `name_held_while_one_thread_publishes_it`.
- Join the program:
  `runtime_probe` (the test-only entry that builds the runtime;
  folder `what_happens_when_you_type_a_command`),
  `push_failure_notice_machine_output` (folder `pushing_what_the_commit_service_saved`),
  and the files of the rename of the program's own names,
  in four new program folders:
  - `names_this_release_and_the_one_before`:
    `wrapper_names_old_and_new`.
  - `telling_whether_an_earlier_commit_service_still_runs`:
    `earlier_unix_sockets_that_listen`,
    `earlier_service_process_look`.
  - `finishing_what_an_earlier_commit_service_left_open`:
    `earlier_service_journal_adopted`.
  - `moving_kept_files_to_the_new_names`:
    `earlier_folder_entries_moved_into_current_folder`,
    `earlier_log_lines_added_to_current_log`,
    `earlier_name_moved_to_current_in_place`,
    `earlier_names_folded_into_current`,
    `kept_names_in_the_git_directories`.
- Suggested plain names for the new files whose names fail MKY alone are in the group map under
  `after_adopted_crates.renames`
  (for example `lazy_runtime` to `the_one_runtime_built_when_first_needed`).

### What the scan found, and the named changes it needs

Placing the files and changing nothing else gave
one loop between crates,
four uses of a program file by a library file
(`blocking_job` and `lazy_runtime` use `panic_notice`,
`scratch_location` uses `wrapper_names_old_and_new`,
`diagnostic_log` uses `lfs_output_mask`),
one loop between folders
and one loop between files.
The check was therefore able to fail on this tree.
Each finding has one named change;
with the first six of the list that follows the engine's counts are zero.
The seventh change is for a loop the engine cannot count.

- `Announcer` joins `service_index_lock_names_and_failures`
  (the old file loop of `service_index_attempts` and `service_index_lock` came back through a new item).
- `while_panics_are_expected` and `panics_are_expected_here` with their per-thread counter move from `panic_notice`
  to a new library file `panics_that_the_caller_expects`;
  the program's `panic_notice` reads them from there.
  Without it the runtime files of `parallel-work-within-a-budget` use the program's panic hook file.
- `scratch_location` takes the repository scratch folder name as a value
  (an instance of "Names of the program,
  its variables and its Git commands passed in");
  the constant stays in `wrapper_names_old_and_new`.
- `lfs_output_mask` moves whole to the text library as a Git-free file
  (and is renamed,
  for example `hiding_the_password_in_web_addresses_in_text`,
  because the name says `lfs`)
  (it is a masker of the user and password part of a URL inside text;
  only its doc comments say `git-lfs`,
  which is reworded).
  `diagnostic_log` uses it,
  and six program files use it from there.
  This reverses the old plan,
  which sent the file to the program.
- `earlier_journals_in_state` and `EarlierJournals` move from `earlier_service_journal_adopted` to
  `wrapper_names_old_and_new`.
  Without it `earlier_names_folded_into_current` uses a folder that uses `landing_commits`,
  which closes a loop of five folders through `repository_facts`.
- `service_review_key` is placed in `messages_between_the_wrapper_and_the_commit_service`,
  as section "Breaking each loop" already says.
  The placement table of the redo had kept it in the review folder,
  which made the messages folder use the HTTPS review folder;
  the redo's zero loops held either way.
- The edge `private_storage` to `worktree_copy_error` that an earlier named change dropped no longer exists in the code,
  so that line of the edge list does nothing.

### Counts after the changes

Superseded 2026-11-26:
these are the counts of the 2026-11-07 table,
and today's tree has 62 source folders and 1622 production modules against the 653 the table was built from.
Section "Rescan of main after the reformat (2026-11-26)" measures the same counts on today's tree
and lists what stands between them and zero loops.
Kept as the record of what the 2026-11-07 placement achieved.

Printed by `engine5.ts` on `spec/spec6.txt`:

- crates:
  9 (8 libraries and the program);
  crate loops 0.
- folders:
  92 (78 in the program,
  14 in libraries);
  folder loops 0.
- files placed:
  617 (536 in the program,
  81 in libraries);
  loops between any files 0.
- uses of a program file by a library file:
  0.
- test references from a library's sidecar to a module outside its crate and outside the test helpers:
  2 (`scratch_location` names `child_class`,
  `package_repair_report` names `wait_ceilings`,
  as before);
  44 more name test helpers.
- Edges from the three binary support files (`binary_fault_support`,
  `binary_service_landing_support`,
  `binary_support_observation`) are not placed;
  they belong to the `native_binary` test target.

What this shows and what it does not:
it is a structural check of the production model,
with no use of a program file by a library file and no loop,
and independent builds and tests of the packages are not verified
(the owner's "I'd like you to confirm with me each pkg can stand on its own itself" stays pending;
the 46 test references named in the counts and the scanner's blind spots remain).
The 653 scanned files and the 617 placed files differ by index files (`mod.rs`),
the two crate roots,
the test-only files and the new files that the named changes create.

Facts the scan does not settle:

- Which of the tokio steps still to come (T6 to T13) add files or edges is unknown;
  they convert code in files this table places.
  The owner's answer 8 ("All tokio now") runs them on today's layout,
  so the table is rescanned again after them.
- `Cargo.toml` said 0.6.0 and the crates.io API listed `monochromatic-git-policy-cli` 0.6.0 updated 2026-11-07
  (fetched 2026-11-07 for this section).
  Whether that is the release step 0 asks for was not checked then.
  Checked 2026-11-10 against the crates.io versions list:
  0.6.0,
  published 2026-11-07T13:10:36Z,
  is the newest plain-number version,
  every version after it is an `-alpha.N` one,
  and `Cargo.toml` now says 0.6.1-alpha.8.
  So crates.io holds the last plain-number release of the unsplit crate,
  which is what step 0 asks for before the split depends on a crate crates.io does not hold.

## Rescan of main after the reformat (2026-11-26)

Measured on `aba76005c3`,
`main`'s tip when this round began,
with the crate already formatted by the root `rustfmt.toml`
(`57bf0918fc`,
merged `00b6a940b4`)
and the two layout rules already on
(`44e8ef805e`,
merged `a1b9b1e850`).
Since the last scan (`e728c63911`,
2026-11-07T10:43:12-05:00) 5269 commits changed the crate's `src`:
2046 files were added,
9 deleted,
and the tree grew from 653 production modules to 1622.
The move order below was written for the first of those numbers,
so this section is what every later step is read against.

### Method

The scanners of the earlier rescans lived in scratch folders,
which are gone,
so this round wrote them again in its own scratch folder
and this section names what each one reads:

- a file walk of `package/git-policy/cli/src` that takes every `.rs` file,
  calls a file production code unless its parent index file declares it under `cfg(test)`
  or its sidecar name ends in `_tests`,
  and adds by hand the modules of `native/binary`,
  `native/testing` and `native/source_guard`,
  which the crate compiles in every build and only tests use
  (the last scan added the same sixteen `binary_*` files by hand for the same reason);
- a reference reader that blanks line comments,
  block comments,
  strings,
  raw strings and character literals,
  keeps line numbers,
  and follows every `crate::` and `super::` path,
  including `crate::{a, b::{c}}` group imports and `#[path]` child-module declarations,
  counting one reference per item named;
- a loop reader that reads the two allow lists in `package/git-policy/cli/layout-allow-list/`
  and reports the loops they record as strongly connected sets;
- an item reader that,
  for every tolerated use,
  names the items the using file takes,
  which answers for 522 of the 560 uses;
  the 38 it cannot name are `#[path]` child-module declarations and uses written through a form it does not read;
- a placement finder and an engine that apply the table to today's modules
  and report loops between crates,
  between target folders and between the files of one target folder,
  plus every use a library file still makes of a program file.

The scan does not see `self::` paths,
macro-generated paths or trait implementations,
which is why two measurements of the same loops disagree by a few,
and it reads no sidecar.
Every command ran in its own `systemd-run --user --scope` with memory,
CPU,
task and disk-weight caps and a runtime limit.
Nothing was built and no test ran:
this round changes no source file.

### What the scan counts now

- 3219 `.rs` files:
  1622 modules the scan calls production (653),
  1478 sidecars (632),
  64 index files,
  and 119 modules declared `cfg(test)`.
  Of the 1622,
  41 are the three test-support folders,
  so 1581 production files are placed.
- 62 source folders (55).
  One `uses/` folder exists in 24 of them,
  and all 24 are folders the ZFS route opened after 2026-11-07;
  the other 38 folders still have none.
- 9659 module references between production modules (2722),
  in 7930 pairs,
  and no two production modules share a name.
- The two allow lists:
  560 file uses inside 67 file loops touching 281 files,
  564 folder uses inside one folder loop touching 57 of 58 folders,
  53 flat `pub use folder::*` re-exports,
  1318 files a folder's top doc does not list,
  46 folders with no file list at all,
  386 `uses/` links not yet written,
  17 group summaries missing and 38 group-folder names the code disagrees with,
  58 files sitting in a group folder,
  3 `link` lines and 3 `order` lines,
  15 index items,
  1 foreign name and 1 unused link.
  That is 3077 lines,
  against the 63 that tolerated nothing and the 190 findings no line recorded when the lists were first measured
  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`,
  "When the rules land").

Two measurements of the same tree do not agree exactly:
the allow lists record 67 file loops,
while the scanner,
which also counts the 1318 files a top doc omits and which the rule leaves out of its loop graph,
finds 61 loops touching 261 modules.
The lists miss at least one loop the scanner sees,
`invocation/real_git` and `invocation/real_git_resolution`,
because one of the two files is not listed in its folder's top doc.
So the lists remain the authority for which line a round may delete,
and a round that breaks loops measures again before it trusts either count.

### Files added since the last scan, placed

1052 production modules are not in the 2026-11-07 table.
41 of them are the test-support modules above,
and the other 1011 are placed by three rules,
in this order:

- a module the table already places keeps its crate,
  folder and file name;
  that is 570 of today's modules;
- a module added since then joins the target of the placed module in its own source folder whose name is its longest
  prefix on an underscore boundary,
  which is how a split file lands beside the file it was split from;
- everything else takes the target its source folder's line in the list below gives.

The source folders keep their files together,
in the order their own `mod.rs` top doc already lists them,
except where a line below sends part of a folder to a library.

- `native/account_companion`:
  `the_account_companion_of_the_zfs_add` (19).
- `native/candidate`:
  `reading_the_bytes_a_commit_would_save` (15),
  `scanning_those_bytes_for_forbidden_strings` (5).
- `native/checkout_boot_units`:
  `the_boot_units_that_reconcile_zfs_clones` (7).
- `native/checkout_underlay_entry`:
  `the_dataset_the_checkout_sits_on` (2).
- `native/clone_holder_scan`:
  `finding_who_holds_a_zfs_clone_folder` (8).
- `native/command`:
  `what_each_git_command_accepts` (22),
  `rules_about_where_a_command_may_run` (13),
  `reading_options_the_way_git_does` (6),
  `rules_that_add_flags_to_a_command` (4).
- `native/commit`:
  `the_commit_command` (45),
  `rules_about_what_a_commit_may_contain` (5),
  `the_hidden_second_git_folder` (3),
  `starting_git_and_describing_a_run` (2),
  `log_lines_of_the_commit_service` (1).
- `native/config`:
  `the_settings_file_cli_git_config_jsonc` (10),
  `git_settings_read_once` (2).
- `native/conflict`:
  `saving_copies_of_what_a_command_would_throw_away` (23),
  `the_settings_file_cli_git_config_jsonc` (1),
  `reading_branches_tags_and_markers` (1).
- `native/dependent_version`:
  library `dependent-version-bump-check` (14),
  `checking_version_bumps_in_a_commit` (4),
  `running_policies_and_settling_their_corrections` (1).
- `native/foundation`:
  library `parallel-work-within-a-budget` (25),
  library `safe-file-saving` (15),
  library `text-time-and-log-line-formats` (14),
  `moving_kept_files_to_the_new_names` (5),
  `names_this_release_and_the_one_before` (4),
  `the_settings_file_cli_git_config_jsonc` (3),
  `finding_the_git_folders` (2),
  `stopping_the_program_at_a_named_step_for_tests` (2),
  `the_words_meant_for_this_wrapper` (1).
- `native/health`:
  `checking_that_your_setup_works` (7),
  `hooks_a_repository_configures` (3).
- `native/health/doctor_zfs_route`:
  `the_doctor_report_of_the_zfs_route` (23).
- `native/health/sanoid_coverage`:
  `the_doctor_report_of_sanoid_coverage` (8).
- `native/index`:
  `reading_and_writing_the_index_file_of_git` (11),
  `landing_commits` (8),
  `settings_that_decide_how_the_index_is_read` (1).
- `native/invocation`:
  `what_happens_when_you_type_a_command` (28),
  `finding_the_real_git_program` (6),
  `the_words_meant_for_this_wrapper` (4),
  `starting_git_and_describing_a_run` (3),
  `what_each_git_command_accepts` (3),
  `running_git_for_a_transaction` (2),
  `environment_variables_for_git_children` (1),
  `spellings_that_switch_the_wrapper_off` (1).
- `native/invocation/typed_network_command_relay`:
  `relaying_what_git_writes_while_a_typed_command_runs` (7).
- `native/kept_references`:
  `the_references_kept_for_unlanded_commits` (10).
- `native/landing`:
  `landing_commits` (51),
  `the_commit_command` (30),
  `after_commit_work_of_the_service` (29),
  `holding_the_index_lock_for_the_commit_service` (18),
  `work_left_for_the_service_to_finish_later` (11),
  `messages_between_the_wrapper_and_the_commit_service` (9),
  `asking_the_commit_service_to_save_a_commit` (5),
  `pushing_what_the_commit_service_saved` (4),
  `finishing_what_an_earlier_commit_service_left_open` (3),
  `telling_whether_an_earlier_commit_service_still_runs` (2),
  `syncing_files_off_the_waiting_path` (2),
  `what_the_wrapper_asks_git_about_a_repository` (1).
- `native/lfs`:
  `uploading_large_files` (17),
  `lines_about_worktrees_and_large_files` (1).
- `native/lock`:
  `finding_who_holds_a_lock_file` (10),
  `clearing_leftover_reference_locks` (10),
  `waiting_for_the_index_lock_of_git` (2),
  library `starting-and-watching-programs` (2),
  `starting_git_and_describing_a_run` (1),
  `what_happens_when_you_type_a_command` (1),
  `running_git_push_until_it_ends_or_stalls` (1).
- `native/markdown`:
  `fixing_markdown_in_process` (6).
- `native/merge`:
  `merging_without_touching_your_files` (42),
  `recovering_a_merge_that_was_cut_short` (13),
  `rerunning_a_merge_word_by_word` (13),
  `merging_text_by_line_and_by_word` (2).
- `native/mount_namespace_scan`:
  `scanning_other_mount_namespaces` (3).
- `native/policy`:
  `the_built_in_policies` (15),
  `running_policies_and_settling_their_corrections` (10),
  `the_list_of_checks_every_commit_runs` (3),
  `putting_corrected_files_into_the_private_index` (1),
  library `safe-file-saving` (1),
  `the_words_meant_for_this_wrapper` (1).
- `native/process`:
  library `starting-and-watching-programs` (30),
  `who_owns_a_transaction_folder` (1).
- `native/push`:
  `pushing_what_the_commit_service_saved` (30),
  library `starting-and-watching-programs` (14),
  `running_git_push_until_it_ends_or_stalls` (10),
  `working_out_what_a_typed_push_would_send` (10),
  `how_long_the_program_waits_for_git_steps` (2),
  `rules_about_what_a_commit_may_contain` (1).
- `native/repair`:
  library `fix-packages-after-copying-a-folder` (19),
  `repairs_the_service_runs_for_a_new_worktree` (11).
- `native/repo_read`:
  `reading_saved_commits_and_files` (10),
  `running_policies_and_settling_their_corrections` (8),
  `reading_git_settings_from_files` (5),
  `answering_questions_without_starting_git` (5),
  `finding_the_git_folders` (4),
  `reading_branches_tags_and_markers` (3),
  `what_the_wrapper_asks_git_about_a_repository` (2),
  `running_git_for_a_transaction` (1),
  `starting_git_and_describing_a_run` (1).
- `native/repository_link_check`:
  `checking_the_repository_link` (8).
- `native/service`:
  `the_service_program` (31),
  `asking_the_commit_service_to_save_a_commit` (18),
  `log_lines_of_the_commit_service` (6),
  `messages_between_the_wrapper_and_the_commit_service` (6),
  `landing_commits` (5),
  `naming_the_service_of_one_worktree` (4),
  `sending_log_entries_to_the_system_journal` (3),
  library `talking-to-the-systemd-user-manager` (2),
  `stopping_the_program_at_a_named_step_for_tests` (2),
  `after_commit_work_of_the_service` (1),
  `reviewing_landed_commits_against_standards` (1),
  `how_long_the_program_waits_for_git_steps` (1).
- `native/shadow`:
  `the_hidden_second_git_folder` (15),
  `replaying_changes_onto_new_commits` (10).
- `native/standards_review`:
  `reviewing_landed_commits_against_standards` (9).
- `native/start_probe`:
  `the_root_start_probe` (34).
- `native/subvolume`:
  `converting_the_main_checkout` (28),
  library `linux-file-and-disk-calls` (15),
  `sha_1_and_sha_256_of_bytes` (1).
- `native/transaction`:
  `running_git_for_a_transaction` (7),
  library `safe-file-saving` (2),
  `starting_git_and_describing_a_run` (1),
  `state_shared_by_preparing_and_landing_a_commit` (1),
  `who_owns_a_transaction_folder` (1),
  `stopping_the_program_at_a_named_step_for_tests` (1).
- `native/worktree`:
  `making_a_worktree_quickly` (61),
  `removing_a_linked_worktree` (27),
  `converting_a_linked_worktree` (11),
  `is_the_worktree_clean` (8),
  `the_trash_a_worktree_removal_sets_aside` (8),
  `the_snapshots_a_removal_takes_before_it_goes` (6),
  `planning_a_worktree_add_and_what_git_made` (5),
  `recording_where_worktree_add_spent_its_time` (1),
  `log_lines_of_the_commit_service` (1),
  library `text-time-and-log-line-formats` (1),
  library `safe-file-saving` (1),
  library `linux-file-and-disk-calls` (1),
  `what_the_wrapper_asks_git_about_a_repository` (1),
  `lines_about_worktrees_and_large_files` (1).
- `native/worktree_clone_add_route`:
  `making_a_clone_of_a_worktree_with_zfs` (25).
- `native/worktree_clone_binding_record`:
  `the_record_and_the_lock_the_zfs_clone_route_keeps` (18).
- `native/worktree_clone_forced_removal`:
  `removing_a_clone_the_kernel_will_not_release` (20).
- `native/worktree_clone_link_repair`:
  `repairing_the_links_of_a_zfs_clone` (2).
- `native/worktree_clone_managed_lock`:
  `the_record_and_the_lock_the_zfs_clone_route_keeps` (15).
- `native/worktree_clone_mount`:
  `attaching_a_zfs_clone_folder` (17).
- `native/worktree_clone_mount_handler`:
  `the_root_modes_that_attach_and_unmount_zfs_clones` (36).
- `native/worktree_clone_mount_protocol`:
  `the_receipts_and_requests_of_a_clone_mount` (19).
- `native/worktree_clone_move_refusal`:
  `the_record_and_the_lock_the_zfs_clone_route_keeps` (1).
- `native/worktree_clone_origin_cleanup`:
  `cleaning_up_the_origin_of_a_zfs_clone` (10).
- `native/worktree_clone_prune_pointer`:
  `the_pointer_a_prune_keeps` (5).
- `native/worktree_clone_record_settling`:
  `settling_the_records_of_a_zfs_clone` (11).
- `native/worktree_clone_removal_cancel`:
  `cancelling_a_zfs_clone_removal` (12).
- `native/worktree_clone_removal_finish_pass`:
  `finishing_a_zfs_clone_removal` (10).
- `native/worktree_clone_removal_proof_and_plan`:
  `proving_a_zfs_clone_may_be_removed` (33).
- `native/worktree_clone_slot_state`:
  `the_state_of_one_zfs_clone_slot` (8).
- `native/worktree_clone_snapshot_batch`:
  `taking_snapshots_for_the_zfs_clone_route` (8).
- `native/worktree_clone_unmount`:
  `unmounting_a_zfs_clone_folder` (15).
- `native/worktree_copying`:
  `copying_ignored_files_into_a_stage` (28),
  `copying_ignored_files_around_one_git_command` (27),
  `what_happens_when_you_type_a_command` (1).
- `native/worktree_removal_deferred_origin_check`:
  `removing_a_linked_worktree` (9).
- `native/zfs_compatibility_gate`:
  `the_zfs_kernel_gate` (5).
- `native/zfs_mounted_dataset`:
  `reading_the_datasets_zfs_mounts` (3).

Thirty-two target folders are new,
all of them in the program crate,
because none of the jobs they hold existed on 2026-11-07:

- `the_account_companion_of_the_zfs_add`:
  the user-namespace check the ZFS add runs before its first write.
- `the_boot_units_that_reconcile_zfs_clones`:
  the boot and shutdown units,
  and the gate that reconciles clone mounts.
- `the_dataset_the_checkout_sits_on`:
  the immutable underlay entry,
  and the verdict a boot takes before attaching a clone.
- `finding_who_holds_a_zfs_clone_folder`:
  the all-namespace holder scan and its refusals.
- `the_references_kept_for_unlanded_commits`:
  the kept references and the pass that prunes them.
- `scanning_other_mount_namespaces`:
  live tasks by namespace and one mount-table read per namespace.
- `checking_the_repository_link`:
  the Rust copy of the repository link check,
  and the gate before a Git command.
- `the_root_start_probe`:
  the nine probe arms,
  their private namespace and the record under
  `/run`.
- `making_a_clone_of_a_worktree_with_zfs`:
  the execution half of the ZFS add route,
  from the claimed slot to a verified fill.
- `the_record_and_the_lock_the_zfs_clone_route_keeps`:
  the binding record,
  its inventories and the managed Git lock.
- `attaching_a_zfs_clone_folder`:
  the descriptor attach,
  the folder identities and the post-attach check.
- `the_root_modes_that_attach_and_unmount_zfs_clones`:
  the root modes of the one executable and the sweep of standing mounts.
- `the_receipts_and_requests_of_a_clone_mount`:
  the mount intent,
  the receipts and the readiness record.
- `cancelling_a_zfs_clone_removal`:
  the guarded way back from a removal that did not finish.
- `finishing_a_zfs_clone_removal`:
  the finish pass over a destruction plan whose head is gone.
- `proving_a_zfs_clone_may_be_removed`:
  the lineage proof,
  the inventory and the persisted destruction plan.
- `taking_snapshots_for_the_zfs_clone_route`:
  the verified snapshot set one removal destroys.
- `unmounting_a_zfs_clone_folder`:
  the staging folder,
  its record and the identity comparisons.
- `removing_a_clone_the_kernel_will_not_release`:
  the forced arm,
  its spared holders and its words.
- `cleaning_up_the_origin_of_a_zfs_clone`:
  the record that carries an origin snapshot's cleanup past the clone.
- `settling_the_records_of_a_zfs_clone`:
  the settling command a state names and this release does not have.
- `the_pointer_a_prune_keeps`:
  the underlay pointer a prune reads,
  and the pass it runs before going.
- `repairing_the_links_of_a_zfs_clone`:
  the report of a damaged link between a worktree and its registration.
- `the_state_of_one_zfs_clone_slot`:
  what one slot is,
  read without changing anything.
- `the_zfs_kernel_gate`:
  the start-time compatibility gate and the module identity read.
- `reading_the_datasets_zfs_mounts`:
  the datasets this process's mount table shows,
  and the design's names.
- `the_doctor_report_of_the_zfs_route`:
  the device,
  ABI,
  mount and snapshot-directory checks doctor gained.
- `the_doctor_report_of_sanoid_coverage`:
  the `sanoid.conf` sections and their exclusions,
  compared by doctor.
- `relaying_what_git_writes_while_a_typed_command_runs`:
  the pseudo-terminal relay of a typed network command's error stream.
- `repairs_the_service_runs_for_a_new_worktree`:
  the package repair the add hands to the commit service.
- `the_trash_a_worktree_removal_sets_aside`:
  the trash,
  its manifest,
  its ignore rule and the pass that empties it.
- `the_snapshots_a_removal_takes_before_it_goes`:
  the snapshots a removal takes before it goes.

Twenty rows of the 2026-11-07 table name a file that no longer exists,
because a later split took its contents:
`blob_batch`,
`bounded_push_waiting`,
`command_commit`,
`command_options_types`,
`conflict_side_decision`,
`conflict_side_notice`,
`current_step_record`,
`discard_audit_generations`,
`how_a_child_program_ended`,
`push_attempt_and_failure`,
`repo_read_layout_without`,
`service_index_lock_names_and_failures`,
`service_replay_path_and_line_text`,
`single_owner_claim`,
`stale_ref_lock_names`,
`stale_ref_lock_nested_symbols`,
`standards_review_append`,
`what_a_command_hands_back`,
`worktree_command_facts` and
`worktree_copy_git_steps`.
Five of them named a change the move order still asks for
(`command_options_types`,
`current_step_record`,
`push_attempt_and_failure`,
`what_a_command_hands_back`,
`worktree_command_facts`),
so the change is still to do under a name today's tree will choose.

Today's file names are kept,
and only the names the monkey test still fails wait for the owner's veto round,
as MKY allows:
`zfs`,
`sanoid`,
`lfs`,
`D-Bus` and `btrfs` are Linux's or Git's own words and are explained in the top doc of each folder that carries one,
`cli` and `core-rust` are the repository's package spellings,
and the ZFS route's 32 new folder names are this round's own wording,
offered here for the veto round and decided by no one.

### What today's tree changes about the plan

- The crate count does not move:
  nine crates,
  the four libraries reading D keeps,
  the test-folder library and the program.
  The libraries have not been lifted,
  and no `package/*/core-rust` folder exists for any of them.
- The folder count moves:
  110 program folders and 13 library folders,
  against the 78 and 14 of section "Inside each crate",
  so the folder list of that section is superseded by the list above and the companion file.
- The four largest program folders stay larger than the file lists they will carry:
  `the_commit_command` (76 files),
  `landing_commits` (64),
  `making_a_worktree_quickly` (61) and
  `merging_without_touching_your_files` (42).
  The plan already says a large folder gets folders inside it the same way,
  and these four are the first to reach for it.
- Measured under this placement,
  the tree is not yet acyclic:
  one crate loop holds 8 of the 9 crates,
  one folder loop holds 100 of the 123 folders,
  and 66 file loops stand in 45 folders.
  The crate loop,
  and the library folders inside the folder loop,
  come from the same 72 uses a library file still makes of a program file,
  in 36 files,
  so the stand-alone changes of "Changes so that a library stands alone" are what removes them.
  The program's own folder loop stands apart from that:
  51 of today's 58 source folders use each other both ways,
  and step 3's breaks and step 6's re-export removal are what work on it.
  The file loops are the loops below.
- Those 72 uses are concentrated:
  `starting-and-watching-programs` 40,
  `fix-packages-after-copying-a-folder` 20,
  `parallel-work-within-a-budget` 5,
  `safe-file-saving` 3,
  `text-time-and-log-line-formats` 3 and
  `linux-file-and-disk-calls` 1.
  The plan's seven stand-alone changes answer most of them;
  the ones the plan does not name are new,
  and they are the watcher's and the service's own records:
  the commit step's scratch records,
  the forwarding value,
  the transaction's request and reply types,
  the busy-step record and the push limit a round holds.
  A round that lifts the watcher must take each of them,
  either as a value the caller passes or as an item moved down.
- Step 8's plan is overtaken in one half and untouched in the other.
  Adding every `uses/` link in one commit and switching the rules on was done the other way round:
  the rules went on with two allow lists,
  and 24 folders now carry their own `uses/` folder,
  all of them the ZFS route's.
  What is left of step 8 is the 386 links the lists tolerate,
  the 53 flat re-exports,
  the 17 group summaries and the 1318 unlisted files,
  and each of them goes when the round that owns it lands.

## The crate split

Redone 2026-11-07T02:30:00-05:00 against the owner's rule that every crate must stand on its own
and that nobody else will use anything related to Git from us.
The draft of 46 libraries is withdrawn;
what it got right (the folders,
the file order inside each folder,
the named changes that break loops) is kept,
and most of its crates become folders of the program crate.

### Method

Each crate is a union of whole target folders;
the group order of the planning doc makes any union of consecutive groups acyclic,
and cutting inside a group was checked with a script,
not by eye:
`engine.ts` reads the table,
applies the named changes of section "Changes the tree needs before files can move",
and reports loops between crates,
between folders of one crate,
and between any files (global).
Named re-exports (`pub use crate::a::X;` in `b.rs`) are resolved so a use through `b` counts as a use of `a`
(284 re-export names found),
and every moved item's own uses are cut out of its source and charged to its new file.

The redo added one check to a copy of `engine.ts` (`scan/engine5.ts`).
The redo's scratch folder is `~/temp/agent/wrapper-redo/` (the first draft's is `~/temp/agent/wrapper-target-tree/`);
the chain that produces every number and table here is
`spec/survivors.txt` (the eight),
`spec/overrides.txt` (files moved between crates),
`spec/changes.txt` and `spec/edges.txt` (the named changes),
`mkspec5.py` (writes `spec/spec5.txt`),
`scan/engine5.ts` (loops and the new check),
`scan/testcheck5.ts` (test references),
`spec/renames-final.txt`,
`gen/meta5.py`,
`gen/genredo.py` and `gen/genmain.py` (the file tree and the lists),
`gen/groupjson5.py` (the group map),
and `doc/build.py` (assembles this file from the previous version).
The check:
for every file of a surviving library,
every use of a file that goes into the program is printed with the item names,
and the run reports how many there are.
A surviving library must show none.
The check was proven able to fail:
with the final placement but without this redo's named changes it prints 18 uses in 11 files;
with the first guess of 15 surviving libraries it printed 44 uses in 14 files.

Numbers printed by the final run,
2026-11-07T02:37:27-05:00,
on the scan of `main` at `f44d921e24`:

- crates:
  9 production crates (8 libraries and the program);
  the test-folders library has no production file in the table and is a dev-dependency of the others,
  so its own build is shown at its lift;
  crate loops 0.
- folders:
  87 (74 in the program);
  folder loops 0.
- files placed:
  590 (68 in libraries,
  522 in the program);
  loops between any files 0;
  loops between files of one folder 0.
- uses of a program file by a library file:
  0.
- test references from a library's sidecar to a module outside its own crate and outside the test helpers:
  2 (named in section "Tests that name a higher crate");
  42 more name the test helpers that move to the test-folders crate.

### The stand-alone test

The owner's words are in section "The owner's words and the rules that bind this".
This test is the coordinator's reading of them,
a suggestion the owner can revise:
the owner has not approved it.
A library crate stands on its own only if all of these hold:

- Its README's first paragraph states its job without mentioning a Git wrapper,
  the commit service,
  our policies or any other crate of this split.
- Its job,
  README,
  name and public API mention nothing related to Git
  (owner,
  2026-11-07:
  "Assume no one else is going to use anything related to git from us.").
- The program it names as a plausible other user is not a Git wrapper,
  a Git front end,
  a push service,
  a merge driver or a commit-landing service.
  A real sibling tool of this repository that is not a Git wrapper counts (the pnpm wrapper,
  for example).
- Its public API exposes no type from a crate that fails the test.
- It builds,
  lints and passes its tests from its own `Cargo.toml` alone
  (the test-folders crate counts only if it passes too).
- Its name passes the monkey test alone,
  or the jargon in it is explained in the top doc of its `lib.rs`
  (the owner's rule for names,
  MKY,
  allows jargon that nothing can replace).
- It may depend on lower libraries of this split that also pass
  (crates list their dependencies in `Cargo.toml`,
  never in `uses/`),
  never on anything that goes into the program.

To make the third point checkable the redo used a bar the owner has not seen:
a plausible user is named only by the path of a sibling that today carries its own copy of the job (evidence tier one),
or whose own job would call it (evidence tier two,
weaker).
"Any program that saves files" names nobody and does not pass.

### Result

Eight of the 46 libraries pass the test on paper,
against the scan;
the proof is the lift,
one library at a time ("How standing alone will be shown").
Four have tier-one evidence and four tier-two evidence.
Tier one means a sibling carries its own copy of an operation the library's first paragraph names;
tier two means no such copy,
but a sibling whose own job would call the library.
The two fixed uses of "operation":
`renameat2` exchange is named in the first paragraph of `linux-file-and-disk-calls`,
while `time_text.rs` reads RFC 3339 text from a registry and the first paragraph of
`text-time-and-log-line-formats` names JavaScript's exact output,
so that one is tier two.
The owner may regrade;
the counts follow.
The fixtures crate passes only in its Git-free half,
as a ninth library used only by tests.
The other 38 libraries fold into the program crate as folders with `uses/` links,
or give some of their files to a passing library.
The program crate holds 74 folders and 522 files.

How this squares with "I refuse to believe C means splitting into fewer crates":
the number of crates now follows from the owner's two rulings of 2026-11-07,
which narrow the earlier sentence,
and from the Git-free jobs the repository's other tools could use
(the coordinator's evidence bar,
not the owner's words),
and not from a target;
the program crate keeps principle 3 inside itself,
because each of its folders holds a `uses/` folder of links,
so fewer crates is not everything in one crate.
The tension with the owner's earlier sentence is not settled by this paragraph:
the owner decides it,
and open point "Which Git-free jobs become libraries" has the wider option.

### The 46 libraries and the fixtures crate, one by one

Passing,
each with the sibling that could use it:

- `safe-file-saving`
  (tier one):
  saves a file so a crash never leaves half of it,
  makes private scratch folders.
  `package/cargo-shadow/cli/src/journal_replace.rs` and `journal.rs` write,
  synchronize and replace files the same way;
  `package/file-enforcer/core-rust/src/publish.rs` synchronizes published files.
- `linux-file-and-disk-calls`
  (tier one;
  was `linux-filesystem-primitives`,
  "primitives" fails the monkey test):
  `package/cargo-shadow/cli/src/journal_replace.rs` swaps two paths with `renameat2`,
  and `package/wg/cli-quicker-exempt/src/pin.rs` calls `renameat2` itself.
  It takes three buried folder-removal files.
- `starting-and-watching-programs`
  (tier one):
  `package/cargo-shadow/cli/src/bounded_run.rs` runs a child under a bound and contains what it started.
  It loses seven Git-bound files and takes four.
- `parallel-work-within-a-budget`
  (tier one):
  `package/forbidden-strings/core/src/parallel.rs` is its own copy of the job,
  and `package/linter/monochromatic-lint/src/run_command.rs` asks for the available threads.
- `text-time-and-log-line-formats`
  (tier two):
  `package/cargo-shadow/cli/src/time_text.rs` reads and writes time text,
  and that tool's README says its catalog tools used to be a TypeScript package.
  It loses two Git-bound files.
- `talking-to-the-systemd-user-manager`
  (tier two;
  was `systemd-user-manager-client`):
  `package/nested-wayland-session/cli/src/systemd.rs` starts transient user scopes by running `systemd-run`.
- `fix-packages-after-copying-a-folder`
  (tier two):
  `package/pnpm-shadow/cli` and `package/cargo-shadow/cli` own the pnpm and Cargo state such a repair rewrites.
  Weakest of the eight:
  no sibling carries a copy or asks for it,
  and no concrete operation of a sibling that would call it is named yet.
  A candidate pending evidence;
  the first to fold if the owner finds none.
- `dependent-version-bump-check`
  (tier two):
  `package/pnpm-shadow/cli` reads the same workspace of `package.json` files.
  The coordinator named the pnpm wrapper as the sibling;
  no concrete call from it is named yet,
  so this one is also a candidate pending evidence.
  It loses three policy-bound files.

Failing (the other 38 libraries and the fixtures crate),
in the draft's order.
Three kinds of reason appear and each entry says which:
coupled to Git (fails the owner's ruling),
no demonstrated user (fails the third point of the test),
or duplicates a published crate the siblings already use (a rule the coordinator added for this redo,
separate from the test).
Each entry gives the reason and what happens to it
("folds" means the crate becomes folders of the program crate;
the folder names are in section "Inside each crate"):

- `git-environment-variables`:
  its job is the environment of a Git child.
  Folds.
- `git-index-file`:
  the Git index format.
  Folds.
- `read-git-files-without-git`:
  reads `.git`.
  Folds.
- `git-lock-files`:
  Git's lock files.
  Folds,
  except three Git-free files (which processes hold a file open,
  and process numbers inside nested containers)
  that go to `starting-and-watching-programs`.
- `typed-command-reader`:
  reads Git command lines.
  Folds.
- `start-the-real-git`:
  starts Git.
  Folds.
- `wrapper-settings`:
  reads the wrapper's settings file and Git's settings.
  Folds.
- `commit-candidate-content`:
  the bytes a commit would save.
  Folds.
- `commit-service-messages`:
  the commit service's messages.
  Folds.
- `system-journal-entries`:
  a Git-free job (a datagram to the journal socket) with only the text formats below it,
  but no sibling writes to the journal today
  (a search of the other packages' Rust sources for `journald` finds nothing).
  The difference from the systemd user manager client:
  `nested-wayland-session` logs with `tracing` to standard error,
  which its unit's journal already captures without a client,
  while a D-Bus client would replace a process it starts today.
  Folds;
  three files,
  lifts again the day a sibling needs it.
- `commit-service-log-lines`:
  the commit service's log lines.
  Folds.
- `crash-points-for-tests`:
  widening it leaves only the arm,
  kill and pause half of `transaction_phase` (a named step read from an environment variable),
  because `service_fault` holds the service's own switches and both hold this wrapper's phase names.
  No user is demonstrated:
  the sibling `package/cargo-shadow/cli/src/failpoints.rs` arms its points in process,
  once,
  on one thread,
  and whether it would take an environment-variable mechanism is untested.
  Folds.
- `durable-work-items`:
  a Git-free file queue,
  but widening it changes four files' API (the kind list,
  the Git object-name rule,
  the `cli-git/work` folder and the service log call),
  and no sibling keeps such a queue.
  Folds.
- `rules-about-git-command-lines`:
  Git command lines.
  Folds.
- `policy-engine`:
  the wrapper's own policies;
  `event_hints` alone names Git 35 times.
  Folds.
- `standards-review-log`:
  reviews landed commits.
  Folds.
- `commit-service-client`:
  talks to the commit service.
  Folds.
- `isolated-merge-recovery`:
  Git merges.
  Folds.
- `sha-1-and-sha-256-digests`:
  a Git-free job,
  but `package/mpm-shadow/cli`,
  `package/cargo-shadow/cli`,
  `package/browserslist/core-rust` and `package/file-enforcer/core-rust` already use the published `sha2` crate,
  so no sibling would pick ours.
  Folds;
  open point "SHA digests".
- `commit-service-index-lock`:
  Git's `index.lock` for the commit service.
  Folds.
- `three-way-text-merge`:
  a Git-free job (one file),
  but a search of the other packages finds no text merge.
  Folds;
  lifts again the day a sibling needs it.
- `merge-conflicts-word-by-word`:
  merge driver.
  Folds.
- `push-with-a-deadline`,
  `upload-large-files-before-a-push`,
  `background-push-after-commit`:
  pushes and LFS.
  Fold.
- `hidden-git-copy-for-preparing-commits`,
  `land-prepared-commits`,
  `keep-copies-of-discarded-work`:
  Git repositories and commits.
  Fold.
  `keep-copies-of-discarded-work` holds a Git-free store of saved copies with age and size limits,
  with no sibling that needs one.
- `markdown-autofix-check`:
  its policy file and its config types name the wrapper's policy and settings types
  (17 uses of program files by 5 of its 6 files in the engine's first run),
  and no sibling runs the Markdown linter in process with a guard.
  Folds.
- `convert-a-checkout-to-a-btrfs-subvolume`:
  five of its 23 files name Git heavily (the checks before,
  the index files,
  the arguments,
  the command),
  and no sibling converts folders to subvolumes
  (a search for `btrfs` and `subvolume` finds only tests,
  a container file and documents).
  Folds;
  its Git-free core (journal,
  copy of attributes,
  entry count) would lift on the day a sibling needs it.
- `new-worktrees-quickly`:
  Git worktrees.
  Folds.
- `shipped-policies`:
  our policies.
  Folds.
- `commit-command`,
  `service-after-commit-work`,
  `commit-service-program`,
  `doctor-report`,
  `merge-in-isolation`,
  `typed-push-planning`:
  orchestration with no outside user even in the draft.
  Fold.
- `disposable-test-fixtures`:
  half of it makes disposable folders,
  helper child processes and output sinks with no Git in them;
  the other half makes Git repositories,
  a recording Git and an HTTP server for Git.
  The Git-free half becomes the ninth library,
  `disposable-test-folders`;
  the rest stays with the program's tests.
  Its sibling users are the test suites of the eight libraries,
  and the test helpers of `package/cargo-shadow/cli` (`cgroup_test_support.rs`).

### What each passing library changes

For each:
the first paragraph of its README,
what leaves or joins,
what changes in its API,
and how it will be shown to stand alone.
The line numbers are of the scanned files in `package/git-policy/cli/src/native/` at `f44d921e24`.

#### safe-file-saving

README,
first paragraph:
"Saves a file so a crash never leaves half of it,
makes private scratch folders and random names,
and cleans up paths."

- Leaves:
  nothing.
  Joins:
  `environment_value` (look a variable up the way `getenv` does) from `child_environment`,
  because `scratch_location` uses it (lines 24,
  97,
  103).
- API:
  `scratch_location` names the program `cli-git` in seven places (lines 38,
  41,
  100,
  107,
  125,
  156,
  162).
  The program name,
  the scratch folder name and the state folder name become values the caller passes in;
  the constants move to the program.
- Tests:
  the sidecars use the folder helpers and `run_child_under_file_size_limit` (to the test-folders crate);
  one names `child_class` (a local stand-in).
- Depends on:
  `text-time-and-log-line-formats` (two uses of `diagnostic_log`).

#### linux-file-and-disk-calls

README,
first paragraph:
"Calls the Linux functions for files and disks:
which kind of disk and mount a folder is on,
btrfs folder copies,
cloning a file's data,
swapping two paths in one step,
and opening and removing files without following links."

- Joins three files that remove a folder tree below a held parent without following links or crossing mounts
  (`worktree_trash_remove`,
  `subvolume_remove`,
  `subvolume_remove_walk`),
  renamed `removing_a_folder_tree_below_a_held_parent_folder`,
  `removing_a_folder_tree_after_making_folders_writable` and `emptying_a_held_folder_step_by_step`.
  They use only `fs_beneath`.
  The engine shows no other use.
- API:
  comments say "the wrapper needs";
  reworded.
  Public names carry no Git words (checked by search).
- Depends on:
  `text-time-and-log-line-formats` (one use of `diagnostic_log`).

#### starting-and-watching-programs

README,
first paragraph:
"Starts another program with its output kept in files,
passes signals to it,
tells which programs are running and since when and which hold a file open,
and ends a program that shows no sign of work."

- Leaves for the program:
  `child_class` (the kinds of Git step),
  `push_deadline_and_patience_settings`,
  `wait_ceilings`,
  `add_timing` (worktree add),
  `transaction_owner` (the commit transaction's owner record),
  `pid_namespace` (reads `LOCK_CEILING` from `wait_ceilings`),
  and the new `current_step_record`.
- Joins:
  `index_lock_holders` and `index_lock_holder_parsers` (which processes hold a file open,
  renamed
  `programs_holding_a_file_open` and `reading_the_output_of_a_holder_scan`),
  `stale_ref_lock_nested_numbers` (process numbers inside nested containers),
  and the new `how_a_child_program_ended`,
  which receives `ChildOutcome` and `outcome_of` from `forwarding`.
- Items that move into it so it needs nothing above:
  `Patience` into the progress file,
  `positive_seconds` into the notice clock,
  `STOP_GRACE` into the watch clock file.
  The push-specific quiet window (`CONNECTED_PUSH_QUIET_WINDOW`) stays in the program and is passed in
  (it was the default of `Clock::start`,
  line 363 of `push_progress`;
  the watcher's terms carry it).
- API:
  the watcher takes plain terms (quiet window,
  notice grace,
  and the name of the variable that changes the window) instead of a Git step class
  (`child_watch_clock_marks_and_stopping` lines 14,
  54,
  67,
  81;
  `child_watch` lines 33,
  303,
  317,
  319);
  the program's `child_class` builds the terms.
  The default ceiling of `output_watched` (`OBJECT_STEP_CEILING`,
  line 35 and 361) and the subject text
  `git cat-file --batch` (line 303) are passed in.
  The environment variable names (`CLI_GIT_WAIT_NOTICE_SECONDS`,
  `CLI_GIT_CHILD_WAIT_SECONDS`) and the program name in
  thread names,
  the anonymous file name and notices become values the caller passes in.
  The after-stop lock sweep is already handed in by the draft's "After-stop action".
- Tests:
  three sidecar references need a Git repository (`REAL_GIT`,
  `repository`,
  `isolated_overlay`);
  those tests move to the program.
- Depends on:
  `text-time-and-log-line-formats` (nine uses).

#### parallel-work-within-a-budget

README,
first paragraph:
"Runs independent jobs side by side on as many threads as the computer can spare."

- Nothing leaves or joins,
  nothing changes in its API,
  no use of any other crate.
  Its sidecars use helper children from the test-folders crate.

#### text-time-and-log-line-formats

README,
first paragraph:
"Writes text,
UTC time stamps and compact JSON records byte for byte as JavaScript does,
quotes text for a shell command,
and prints tagged debug and warning lines."

- Leaves:
  `lfs_output_mask` and `worktree_path_failure_message`
  (to the program folder `lines_about_worktrees_and_large_files`).
- API:
  `diagnostic_log` prints `[cli-git]` (line 105);
  the tag becomes a value the caller passes in.
  Comments that say "both wrappers" and "the incumbent" are reworded to say what the format is.
- Depends on:
  nothing.

#### talking-to-the-systemd-user-manager

README,
first paragraph:
"Starts and looks up systemd user units by speaking D-Bus to the user manager directly.
systemd is the Linux program that starts and watches background services;
a user unit is one such service run for one person;
D-Bus is the channel the user manager listens on."

- Nothing leaves or joins,
  nothing in its API changes,
  no use of any other crate;
  one sidecar uses the folder helper from the test-folders crate.

#### fix-packages-after-copying-a-folder

README,
first paragraph:
"Repairs the installed packages of a copied folder
(linked `node_modules`,
Cargo build state,
toolchain launchers)
so they work at the new location."

- API:
  `package_repair_cargo_workspaces` runs `git ls-files -z -- *Cargo.toml` (lines 132 to 138),
  and `RepairRequest` carries a Git context (`package_repair_request` line 192).
  The request carries the list of manifest directories instead,
  and the program builds that list with Git before it calls.
  No closure is handed (the other branch removes closures).
- API:
  `package_repair` and `package_repair_cargo` call `add_timing::work` around steps.
  The repair reports its step durations in its report,
  and the program records them in `add_timing` (new use by `worktree_copy`).
- `REPAIR_TOOL_CEILING` moves into `package_repair_request` with its measurement.
  `repair_worktree` is renamed (`repair_copied_folder`),
  and the notices that say "cli-git" and "the worktree" take the program name and the word "folder".
- Tests:
  one sidecar builds a Git context (`git_context`) and moves to the program.
- Depends on:
  `parallel-work-within-a-budget`,
  `safe-file-saving`,
  `starting-and-watching-programs`,
  `text-time-and-log-line-formats`.

#### dependent-version-bump-check

README,
first paragraph:
"Plans a version bump for every package that depends on a package whose version was raised."

- Leaves:
  `dependent_version_lifecycle` (reads the commit candidates),
  `dependent_version_policy` (takes the policy `Trigger`),
  and `policy_dependent_version` (the policy adapter).
  They go to the program folder `checking_version_bumps_in_a_commit`.
- API:
  `PolicyIncomplete` of `dependent_version_content` is renamed.
- Depends on:
  nothing.

#### disposable-test-folders

The test-only ninth library.
README,
first paragraph:
"Makes disposable folders,
helper child processes and output sinks for tests,
and runs a child under a file-size limit."

- Takes the Git-free helpers of `test_support`
  (`fixture`,
  `fixture_with_prefix`,
  `fixture_root`,
  `settle_fixture`,
  `remove`,
  `age`,
  `spawn_owned`,
  `own_group`,
  `this_test_binary`,
  `SharedSink`,
  `ActionSink`,
  `arm_watchdog`,
  `forbid_core_dumps`,
  `run_child_under_file_size_limit`),
  `no_threads_child_test_support`,
  `parallel_test_support`,
  `umask_child_test_support`,
  `fixture_executable` and `test_alive`.
  Variable names such as `CLI_GIT_KEEP_FIXTURES` become values the caller passes in.
- Stays with the program's tests:
  `git`,
  `git_output`,
  `git_text`,
  `repository`,
  `isolated_overlay`,
  `git_context`,
  `REAL_GIT`,
  `dead_copy_owner`,
  `strip_hint_fields`,
  `recording_git` and `smart_http_support`.

#### How standing alone will be shown

Before the shared build folder is set (owner:
"One shared build folder,
but first,
I'd like you to confirm with me each pkg can stand on its own itself."),
each of the nine is lifted bottom first and shown,
one at a time:

- It has its own `Cargo.toml` that lists only the nine libraries of the
  What each passing library changes section as `path` dependencies with an exact `=` pin,
  and,
  for every package except `disposable-test-folders` itself,
  that library as a dev-dependency.
- Its README opens with the job line its subsection gives;
  a search of its README,
  public names and `Cargo.toml` for the Git words (`git`,
  `lfs`,
  `push`,
  `commit`,
  `worktree`,
  `cli-git`,
  `wrapper`,
  `policy`) finds none,
  and the public items listed from the build (`cargo doc --no-deps`) are read,
  not only searched.
  This holds for the names of option A in open point "Where the libraries live and what they are named";
  with the draft's `monochromatic-git-policy-<name>` names it fails by construction.
- Its own tests pass from its own folder with `CARGO_TARGET_DIR` unset,
  through its `mise.toml` tasks,
  in a throwaway copy that holds only that package and its listed dependencies.
  The test output is read for the expected suite and test names (`TRV`).
- `cargo tree --depth 1` lists exactly the dependencies of its `Cargo.toml`,
  and the full tree is read for anything else that comes in.
- The owner is told the result for each before the next one is lifted.

### Jobs found buried in the program

The owner asked to look for general jobs that sit inside the program.
Two groups joined passing libraries and are counted above:
three files that tell which processes hold a file open
(they used only `/proc`,
`lsof` and the process facts),
and three that remove a folder tree without following links.

Git-free clusters that stay in the program because no sibling needs them today.
Each lifts again the day one does:

- `three-way-text-merge`:
  `word_merge`,
  one file,
  no use of anything.
- `system-journal-entries`:
  three files,
  uses only the text formats.
- `sha-1-and-sha-256-digests`:
  one file;
  replace by the `sha2` crate for SHA-256 and a SHA-1 crate for SHA-1 instead
  (`package/forbidden-strings/cli` already takes both from `gix-hash`).
- The conversion core of a folder into a btrfs subvolume:
  about ten files of `converting_the_main_checkout`
  (journal,
  entry count,
  attribute copying,
  file write steps,
  error).
- The folder of saved copies with age and size limits,
  and the rotating audit log (`discard_audit_generations`).
- The unified-diff reader and reverse-apply check of `replaying_changes_onto_new_commits`
  (`replay_patch`,
  `replay_reverse_apply`).
- The durable work-item queue (four files).
- The inotify wait on a folder (`service_index_watch`).
- The test switches that stop or pause a program at a named step.

### Files of `main` added after the scan

Superseded 2026-11-07T10:43:12-05:00:
the files added after `f44d921e24` are placed in section "Rescan of main after the adopted crates".
The text this section held (a list of nine files as unplaced) described `HEAD` before that scan.

### The program crate

The program crate `package/git-policy/cli` keeps `monochromatic-git-policy-cli` as its name.
Note 2026-11-26:
today's placement puts its files in 110 folders,
as section "Rescan of main after the reformat (2026-11-26)" lists;
74 is the 2026-11-07 count this section keeps as its record.
Its `src/` holds 74 folders and `main.rs` and `lib.rs`;
each folder is a former crate or a folder of a former crate,
listed in section "Inside each crate" with its `uses/` links.
It depends on the eight libraries in its `Cargo.toml`.
Superseded 2026-11-07T10:43:12-05:00:
the folders no longer sit flat.
The owner answered "One level of groups",
so they sit in group folders,
and section "One level of groups in the program" drafts the groups and ranks the options for the rule that relates a group
to its children.
The flat list in this section and in "Inside each crate" is the list of child folders.

Former crates and the program folders they became,
in the order of the draft:

- `git-environment-variables`:
  `environment_variables_for_git_children`
- `system-journal-entries`:
  `sending_log_entries_to_the_system_journal`
- `sha-1-and-sha-256-digests`:
  `sha_1_and_sha_256_of_bytes`
- `git-lock-files`:
  `clearing_leftover_reference_locks`,
  `finding_who_holds_a_lock_file`
- `commit-service-index-lock`:
  `waiting_for_the_index_lock_of_git`,
  `holding_the_index_lock_for_the_commit_service`,
  `syncing_files_off_the_waiting_path`
- `push-with-a-deadline`:
  `running_git_push_until_it_ends_or_stalls`
- `wrapper-settings`:
  `the_settings_file_cli_git_config_jsonc`,
  `git_settings_read_once`,
  `hooks_a_repository_configures`
- `three-way-text-merge`:
  `merging_text_by_line_and_by_word`
- `isolated-merge-recovery`:
  `recovering_a_merge_that_was_cut_short`
- `commit-service-messages`:
  `messages_between_the_wrapper_and_the_commit_service`,
  `naming_the_service_of_one_worktree`
- `commit-service-log-lines`:
  `log_lines_of_the_commit_service`
- `durable-work-items`:
  `work_left_for_the_service_to_finish_later`
- `commit-service-client`:
  `asking_the_commit_service_to_save_a_commit`
- `hidden-git-copy-for-preparing-commits`:
  `the_hidden_second_git_folder`,
  `replaying_changes_onto_new_commits`
- `convert-a-checkout-to-a-btrfs-subvolume`:
  `converting_the_main_checkout`
- `upload-large-files-before-a-push`:
  `uploading_large_files`
- `background-push-after-commit`:
  `pushing_what_the_commit_service_saved`
- `merge-in-isolation`:
  `merging_without_touching_your_files`
- `doctor-report`:
  `checking_that_your_setup_works`
- `git-index-file`:
  `reading_and_writing_the_index_file_of_git`
- `read-git-files-without-git`:
  `settings_that_decide_how_the_index_is_read`,
  `reading_branches_tags_and_markers`,
  `reading_saved_commits_and_files`,
  `reading_git_settings_from_files`,
  `finding_the_git_folders`,
  `state_shared_by_preparing_and_landing_a_commit`
- `typed-command-reader`:
  `reading_options_the_way_git_does`,
  `spellings_that_switch_the_wrapper_off`,
  `what_each_git_command_accepts`,
  `the_words_meant_for_this_wrapper`
- `commit-candidate-content`:
  `reading_the_bytes_a_commit_would_save`,
  `scanning_those_bytes_for_forbidden_strings`,
  `putting_corrected_files_into_the_private_index`
- `rules-about-git-command-lines`:
  `rules_about_where_a_command_may_run`,
  `rules_that_add_flags_to_a_command`,
  `rules_about_what_a_commit_may_contain`
- `standards-review-log`:
  `reviewing_landed_commits_against_standards`
- `policy-engine`:
  `running_policies_and_settling_their_corrections`
- `shipped-policies`:
  `the_built_in_policies`,
  `the_list_of_checks_every_commit_runs`
- `markdown-autofix-check`:
  `fixing_markdown_in_process`
- `keep-copies-of-discarded-work`:
  `saving_copies_of_what_a_command_would_throw_away`
- `merge-conflicts-word-by-word`:
  `rerunning_a_merge_word_by_word`
- `land-prepared-commits`:
  `landing_commits`
- `service-after-commit-work`:
  `after_commit_work_of_the_service`
- `commit-service-program`:
  `the_service_program`
- `commit-command`:
  `the_commit_command`
- `typed-push-planning`:
  `working_out_what_a_typed_push_would_send`
- `start-the-real-git`:
  `finding_the_real_git_program`,
  `starting_git_and_describing_a_run`,
  `answering_questions_without_starting_git`,
  `running_git_for_a_transaction`,
  `what_the_wrapper_asks_git_about_a_repository`
- `new-worktrees-quickly`:
  `planning_a_worktree_add_and_what_git_made`,
  `is_the_worktree_clean`,
  `copying_ignored_files_into_a_stage`,
  `making_a_worktree_quickly`,
  `copying_ignored_files_around_one_git_command`,
  `converting_a_linked_worktree`,
  `removing_a_linked_worktree`
- `crash-points-for-tests`:
  `stopping_the_program_at_a_named_step_for_tests`

### Dependency graph

Listed from the bottom:
a package appears after every package it uses.
It is acyclic (checked),
and each package's `Cargo.toml` lists exactly these `path` dependencies with an exact `=` pin.

```text
dependent-version-bump-check
    uses crates: none
parallel-work-within-a-budget
    uses crates: none
text-time-and-log-line-formats
    uses crates: none
safe-file-saving
    uses crates: text-time-and-log-line-formats
starting-and-watching-programs
    uses crates: text-time-and-log-line-formats
fix-packages-after-copying-a-folder
    uses crates: parallel-work-within-a-budget, safe-file-saving, starting-and-watching-programs, text-time-and-log-line-formats
linux-file-and-disk-calls
    uses crates: text-time-and-log-line-formats
talking-to-the-systemd-user-manager
    uses crates: none
cli (the program, monochromatic-git-policy-cli)
    uses crates: dependent-version-bump-check, fix-packages-after-copying-a-folder, linux-file-and-disk-calls, parallel-work-within-a-budget, safe-file-saving, starting-and-watching-programs, talking-to-the-systemd-user-manager, text-time-and-log-line-formats
disposable-test-folders
    uses crates: none (every other package lists it under dev-dependencies)
```

### Crates considered and rejected

- The draft's 46 libraries:
  rejected by the owner on 2026-11-07;
  the verdicts are above.
- One crate per top-level folder of the planning doc (23 groups):
  rejected,
  because a group is a layer,
  not a job.
- A few layer crates (the owner's option B and C sketches):
  not chosen,
  because a layer is not a reusable unit ("Crates obviously have to be reusable").
- One crate:
  rejected by the owner ("We should not put everything in one crate.").
- `process-facts`,
  `how-long-to-wait` and `watched-child-programs`:
  merged into `starting-and-watching-programs`,
  as in the draft.
  `pid_namespace` reads `LOCK_CEILING` from the waits table and the watcher reads process facts,
  so any two of the three would have looped.
  The waits table and `pid_namespace` now leave for the program;
  the other process facts stay.
- Cutting the draft's `new-worktrees-quickly` into crates:
  moot,
  since it folds.
  The folder cut along `is_the_worktree_clean` stays possible inside the program.
- A Git-aware library crate for reading `.git`,
  the index and lock files:
  rejected by the owner on 2026-11-07,
  however general.

### What a crate costs

Measured on `package/catalog-specifier/core-rust`,
a small library of this repository:
besides `src/` it carries `Cargo.toml` (50 lines),
`Cargo.lock` (2755 bytes),
`mise.toml` (43 lines),
`README.md` (114 lines) and a `LICENSES/` folder with two licence texts (about 43 kB),
plus its own `target` folder.
Nine new libraries mean nine of each,
about 390 kB of licence text,
against 47 in the draft.
The wrapper's `target` is 1.4 GB today,
and each package is its own workspace,
so dependencies such as `serde_json` compile once per package unless the packages share one `CARGO_TARGET_DIR`
(open point "One shared build folder").
File-enforcer's manifests globs (`package/*/*/Cargo.toml`) already cover new packages
two folders deep,
so the lints and workspace blocks arrive without a new entry.
The exact-pin cascade (a bump at the bottom edits every dependent `Cargo.toml` and lockfile)
now reaches the program and a few libraries instead of dozens of manifests.

## The eight libraries against published crates

Written 2026-11-07T10:43:12-05:00.
The owner asked,
verbatim,
"Why are we not using already established crates for some functions?"
The study `doc/research/wrapper-git-free-jobs-and-published-crates.md` answered it job by job on 2026-11-07T03:30:00-05:00.
Since then the owner picked tokio (multi-thread) for processes,
waits and parallel work,
`tempfile` for safe file saving,
`rustix` for Linux calls,
`jiff` for time stamps,
`sha1` and `sha2` for digests,
`sysinfo` for memory headroom,
`hyper` for HTTPS and `zbus` 6 for D-Bus,
and said of file saving,
verbatim,
"Everything async,
and safe-file-saving shouldn't be something we own at all."
This section checks each library of the table against those picks and against other published crates,
with the code of `e728c63911` (section "Rescan of main after the adopted crates").

### Which gates ran

The skill `choosing-technology` applies.
Every candidate here is open-source local technology,
so the managed-service gates do not apply.
What ran:

- Category fit,
  licence,
  newest release and normal dependencies of each crate not yet adopted,
  from the crates.io API on 2026-11-07
  (`~/temp/agent/wrapper-layout-after-crates/crates/facts.out`,
  `deps.out`).
- A frozen list of ten registry queries,
  each page one,
  sorted by relevance
  (`search.out`):
  "redact url credentials in text" (779 results),
  "mask password in url" (335),
  "remove directory tree no cross mount openat" (6),
  "unlinkat recursive no follow symlinks" (3),
  "btrfs subvolume" (56),
  "mountinfo parser" (1),
  "ordered concurrent results stream" (2181),
  "run command inactivity timeout" (274),
  "pidfd async" (30),
  "systemd unit start dbus" (50).
  Page one only:
  this is not a saturated discovery.
- The measurements and source reads of the study and of `doc/audit/tech-safe-file-saving-published-crates-vet-2026-11-07.md`,
  which are cited where they decide a verdict.
- Sibling packages searched for users of each job
  (`rg` over `package/` outside `package/git-policy`).

What did not run:
no clone,
upstream test run,
maintenance sample,
execution of third-party code or scoring for any crate that is not already adopted.
So this section recommends no new crate.
A crate named as a candidate is a name for the owner's next step,
and the report that would let anyone adopt it does not exist.
Where a verdict says "nothing published fits",
it means nothing screened fits,
with the queries listed;
it is not a claim about crates this page did not show.

### Crates screened that are not adopted

- `btrfs-uapi` 0.13.0:
  MIT or Apache-2.0,
  updated 2026-05-14,
  9,792 downloads,
  dependencies `bindgen` (build),
  `bitflags`,
  `btrfs-disk`,
  `nix`,
  `uuid`.
  The build dependency needs `libclang`;
  the study built it in 21.1 s on this host and did not check the container images.
  Screening result:
  not removed,
  not validated.
- `procfs` and `procfs-core` 0.18.0:
  MIT or Apache-2.0,
  last release 2025-08-30,
  about 94 million and 78 million downloads;
  the study swept 745 processes and found `procfs` opens with `openat`,
  not `openat2`.
  Screening result:
  not removed,
  not validated against the wrapper's tests.
- `netlink-packet-sock-diag` 0.5.0:
  MIT,
  updated 2026-08-24;
  has both byte counters the watcher reads (study,
  `src/inet/nlas.rs` lines 599 and 600).
  Not removed,
  not run.
- `shell-quote` 0.8.0:
  Apache-2.0,
  updated 2026-08-07,
  optional `bstr` only.
  The owner picked "Keep ours (readable)",
  so it is not a candidate unless the owner changes that.
- `remove_dir_all` 1.0.0:
  MIT or Apache-2.0,
  last release 2024-11-22,
  depends on `fs_at`,
  `normpath`,
  `cvt`,
  `libc`,
  `cfg-if`,
  `log`.
  Named from general knowledge,
  not surfaced by this page's queries.
  Whether it refuses to cross a mount below a held parent was not read.
- `whichdisk` 0.7.0:
  MIT or Apache-2.0,
  updated 2026-10-01,
  about 20,000 downloads,
  depends on `rustix`,
  `libc`,
  `bytes`,
  `memchr`.
  Its description (disk and mount a path is on) overlaps `fs_kind` and the mount reader.
  Not read.
- `async-pidfd` 0.1.5 (MIT or Apache-2.0,
  optional `async-io`),
  `pidfd-util` 0.1.0 (MIT or Apache-2.0,
  `nix`,
  optional `async-io`),
  `waitpidx` 0.0.1 (25 downloads):
  pidfd wrappers.
  `async-io` is an optional feature of each,
  and whether they work with tokio's `AsyncFd` was not established;
  not read.
- `systemd-transient` 0.2.1 (Apache-2.0 or MIT,
  72 downloads,
  `zbus`,
  `tokio`):
  starts transient units by D-Bus.
  The wrapper starts an existing unit,
  so it is a neighbour,
  not a fit.
- `uuid` and `tracing`:
  `tracing` 0.1.44 is already in the lock file (study);
  `uuid` with only the `v4` feature adds one package (study).
  Neither is a direct dependency of the wrapper.

### Verdict for each library

Three outcomes are used:
a published crate now does the whole job so no crate of ours is needed,
a crate does part and a remainder is left (with where it belongs;
the line counts of each remainder are given because "thin" is not claimed for the large ones),
or nothing published fits so the library stays.

- `safe-file-saving`:
  a crate does part,
  and the remainder is not a library of ours:
  about forty lines of glue and fallback that the owner asked to keep in the program,
  plus program-specific rules.
  Evidence:
  `tempfile::Builder` with `persist_noclobber` takes the staging name and the no-replace publish,
  `rustix::fs::fsync` the directory sync,
  `tokio::fs` the create-new,
  the `O_NOFOLLOW` read and the private folder
  (vet report,
  "Behavior measured with the published calls alone",
  15 probe lines read PASS).
  Already in `main`:
  `foundation/private_storage.rs` publishes through `tempfile` (9 uses) after commit `2a503fc9e1`.
  What no crate gives (vet report,
  "What no crate gives"):
  the fallback to a checked rename on a disk with neither `renameat2` no-replace nor hard links
  (about ten lines and a fifteen-line test,
  which the owner kept:
  "Keep the fallback (Recommended)"),
  the composition of file sync,
  publish and directory sync (about fifteen lines),
  and removing a staging file a kill left.
  The other files of the table are rules of this program's own scratch locations
  (`scratch_location`,
  371 lines,
  the rule of `doc/planning/cli-git-scratch-locations.md`),
  path helpers (`lexical_path`,
  `path_bytes`,
  `worktree_add_plan_path_questions`),
  random names in a layout (`random_id`,
  131 lines;
  `uuid` with `v4` is the published candidate)
  and recovery file helpers.
  Where the remainder belongs:
  the program,
  in the folder that keeps the name `saving_files_so_a_crash_leaves_no_half_file`;
  it shrinks again when the 38 callers become asynchronous (tokio step T11).
  The tier-one evidence of the table (siblings that write,
  sync and replace files)
  no longer argues for a crate of ours,
  because the siblings can use the same published calls.
- `linux-file-and-disk-calls`:
  a crate does part (the syscall layer),
  and the rest has no validated crate,
  so it is a remainder of about 1,350 lines that folds on the stand-alone test,
  not on size.
  Evidence:
  `rustix` (adopted,
  already linked) did `openat2` with the four resolve flags,
  `renameat_with` exchange and no-replace,
  `statfs`,
  `ioctl_ficlone` and inotify on a btrfs probe (study,
  "Linux file and disk calls");
  at `e728c63911` none of these files calls `rustix` yet
  (113 `libc::` uses and 32 `unsafe` in the library's files),
  so the swap is still to do.
  `rename_that_never_replaces` (48 lines) is the one file that `rustix` replaces whole.
  What `rustix` lacks:
  the btrfs ioctls (`fs_btrfs_ioctl`,
  456 lines;
  a search of `rustix` 1.1.4 for `btrfs` found nothing),
  the mount-option reader (`fs_mount_options`,
  341),
  the error mapping (`fs_primitive_error`, 110)
  and the three files that remove a folder tree below a held parent without crossing a mount (437 together).
  Candidates for those pieces are in the list of crates screened here (`btrfs-uapi` needs `libclang`;
  `procfs-core` and `whichdisk` have a mount-table reader;
  `remove_dir_all` removes trees);
  none is validated.
  Where the remainder belongs:
  the program,
  because it has no outside user:
  no sibling package uses a btrfs ioctl,
  a mount-table reader or `openat2` outside `cargo-shadow`'s control-group file
  (`rg` over `package/` outside `package/git-policy`:
  btrfs appears only in tests and text,
  `mountinfo` only in `ide`'s audit tool).
  The two sibling copies the table named as evidence (`journal_replace.rs`,
  `pin.rs`) call `renameat2`,
  which `rustix` already does.
- `starting-and-watching-programs`:
  nothing published fits the core,
  so the library stays,
  smaller where `rustix` and tokio take over.
  Evidence:
  no inactivity-based watcher on the first page of 274 results of "run command inactivity timeout"
  (page one;
  `command_timeout` 0.1.3 is a tokio command timeout by elapsed time,
  which the owner ruled out as a reason to end a child),
  the study screened `process-wrap` and `wait-timeout` by description and read the source of `kill_tree`,
  and found none with start-time guarded signalling or a no-progress rule;
  `signal-hook-registry::unregister` leaves the signal ignored instead of restoring it
  (tokio plan,
  "The signal relay cannot be tokio's signal handling");
  tokio's own child start can report a failed start after the child runs
  (`tokio::process`,
  `src/process/unix/mod.rs`,
  read in the tokio plan),
  which is why the owner picked "std start + pidfd".
  Already in `main`:
  `child_awaited_through_a_pidfd` (166 lines) is `rustix` and `tokio::io::unix::AsyncFd`,
  `child_watch` is a task on the shared runtime.
  The pidfd crates in the list of crates screened here have not been shown to work with tokio,
  so none is shown to replace it.
  The sibling that carries its own copy of the operation:
  `package/cargo-shadow/cli/src/bounded_run.rs` and `control_group.rs` use `rustix` process and event calls and `pidfd`,
  and `package/wg/cli-quicker-exempt/src/watcher_sentinel.rs` uses `pidfd`.
  What may still go to published crates:
  the `/proc` parsers to `procfs` and the netlink read to `netlink-packet-sock-diag`
  (study:
  candidates,
  not validated).
  Where the remainder belongs:
  this library,
  with the new dependency on `parallel-work-within-a-budget` for the shared runtime
  (`captured_child_run_to_its_end` and `child_watch` use `lazy_runtime` and `blocking_job`).
- `parallel-work-within-a-budget`:
  a crate does part,
  and what is left is thinner each tokio step.
  Evidence:
  `sysinfo` (adopted) now reads the memory headroom (`parallel_budget` holds 3 uses),
  tokio's `JoinSet` and `Semaphore` are the planned form of the ordered runner (tokio plan,
  step T12),
  and `std::thread::available_parallelism` follows the cgroup quota.
  What nothing published gives:
  the budget formula with the owner's margins,
  and the policy for building the runtime
  (address space left under `RLIMIT_AS`,
  threads left under `RLIMIT_NPROC` and `pids.max`,
  the arena cap,
  the deadline-bounded build,
  the stand-in runtime;
  tokio plan,
  "A multi-thread runtime panics,
  then hangs,
  when the system refuses it threads");
  `rayon`'s global pool panics where our code runs the job in turn (study,
  `rayon-core-1.13.0/src/registry.rs` line 171).
  Today the library holds 10 files and 1,426 lines,
  of which the scoped-thread runner (`parallel_runner`,
  `scoped_start`,
  `side_by_side_threads`,
  about 620 lines) goes at step T12.
  Where the remainder belongs:
  the library while a sibling needs the runner,
  and a part of the program after step T12,
  unless a sibling that builds a tokio runtime names a call.
  A fold is not free:
  `starting-and-watching-programs` uses `lazy_runtime` and `blocking_job`,
  so folding the runtime into the program would make that library use program files again.
  Either the caller hands the library a tokio runtime handle and the runtime-building policy stays in the program,
  or the runtime component stays independent;
  the graph is rescanned before the fold is treated as viable.
  Today's siblings do not:
  `package/forbidden-strings/core/src/parallel.rs` builds rules on its own threads
  and `package/linter/monochromatic-lint/src/run_command.rs` only asks for the thread count.
  Whether the policy files alone justify a crate is for the owner,
  after step T12 shows how large they are.
- `text-time-and-log-line-formats`:
  a crate does part (time stamps and JSON quoting),
  and about 850 lines remain that have no validated crate and no outside user
  (`json_record` and `js_text`,
  468 lines together,
  wait for a comparison of their readers before anyone can say whether a standard form replaces them).
  Evidence:
  `jiff` (adopted) writes the time stamps,
  and `iso_time` is now 56 lines over it;
  `serde_json` does the JSON quoting;
  the owner dropped JavaScript identity ("\" text identical to JavaScript output\" - that's not needed.").
  What is left in the table:
  `diagnostic_log` (169 lines),
  `shell_quote` (104,
  kept by the owner's pick),
  the credential masker `lfs_output_mask` (106;
  the two registry queries about masking credentials in URLs returned applications and agents only,
  no library),
  `json_record` (347,
  its ordered object writer and the safe-integer rules,
  readers not compared)
  and `js_text` (121,
  its readers not compared).
  Where the remainder belongs:
  the program.
  One consequence is new:
  with this library gone,
  the four libraries that stay would still call program files for log lines,
  quoting and text trimming (section "What the verdicts do to the table"),
  so the owner must say how a library prints a log line.
- `talking-to-the-systemd-user-manager`:
  a published crate now does the whole job.
  Evidence:
  `zbus` 6.2.1 on tokio is the owner's pick,
  and two sibling packages already depend on it
  (`package/nested-wayland-session/cli/Cargo.toml` line 89 and `package/music-player/desktop-app/Cargo.toml` line 159,
  both with the catalog floor `>=6.2.1`).
  The hand-written client is still in `main` (`systemd_dbus.rs`,
  396 lines,
  and `systemd_dbus_wire_format.rs`,
  317);
  tokio step T8 replaces it.
  The one thing `zbus` lacks is one deadline over connect,
  authentication and the calls
  (its only timeout is `method_timeout`,
  study `src/connection/mod.rs` lines 269 and 270);
  a `tokio::time::timeout` around the call does that in the caller.
  So no crate of ours is needed,
  and the few lines of call and deadline belong in the program,
  in the folder that starts the commit service's unit.
  The tier-two evidence of the table (a sibling that starts a unit by `systemd-run`)
  is moot,
  because the sibling uses `zbus` itself.
- `fix-packages-after-copying-a-folder`:
  nothing published fits,
  so the library stays.
  Evidence:
  the study's queries ("package.json dependent version bump",
  "pnpm workspace",
  "node_modules relink",
  "repair node_modules",
  "cargo target directory relocate") found this repository's own packages,
  release tools and unrelated names;
  they were not rerun here.
  It is the weakest library on evidence (tier two,
  still no sibling that carries a copy or calls it),
  and the first to fold if the owner finds none.
  Its published parts are already crates (`serde_json`,
  `tempfile`,
  tokio);
  tokio steps T6 and T12 convert its `std::thread` uses (9 of them).
- `dependent-version-bump-check`:
  published coverage is undetermined,
  because two candidates are unread,
  so the library stays in the model provisionally and this is not an established case of the third outcome.
  Evidence:
  the study's four queries returned release tools (`changesette`,
  `changepacks`,
  `sampo`,
  `monochange`,
  `vup-rs`),
  and it read none as a library;
  `semver` is in the lock file.
  Reading `changesette` and `changepacks-core` against `dependent_version_plan_tests.rs` is the work that would turn this into
  "a crate does part" or "nothing fits".
  Its 11 files (2,070 lines) use no adopted crate and no program file.

### What the verdicts do to the table

The owner has not said how many libraries remain ("I'm not ready to answer this specific question yet.").
These are readings,
not a proposal to decide:

- If every verdict in the list of verdicts is taken,
  `safe-file-saving`,
  `linux-file-and-disk-calls`,
  `text-time-and-log-line-formats` and `talking-to-the-systemd-user-manager` leave the list of libraries,
  and four production libraries remain:
  `starting-and-watching-programs`,
  `parallel-work-within-a-budget`,
  `fix-packages-after-copying-a-folder` and `dependent-version-bump-check`.
  The study's reading (option a of its open choice 1) was six.
- The four that remain were checked by the engine with the four that leave placed in the program
  (`spec/spec7.txt`).
  The counts:
  5 crates,
  crate loops 1 (`fix-packages-after-copying-a-folder` and `starting-and-watching-programs` with the program),
  folder loops 0,
  file loops 0,
  and 19 uses of a program file by a library file in 15 files.
  The 19 uses are five small things:
  `diagnostic_log` (`debug` and `warn`,
  10 files),
  `js_text` (`trim_javascript`,
  `decimal_attempt`,
  4 files),
  `shell_quote` (2 files),
  `json_record` (`quote`,
  `MAX_SAFE_INTEGER`,
  2 files)
  and `private_storage::copy_file_exclusive` (1 file).
  Each has a published or standard-library form
  (`tracing` or the `log` facade for the lines,
  `str::trim` and `str::parse` once the readers are compared,
  `serde_json::to_string`,
  and for `private_storage::copy_file_exclusive` a create-new copy built on the adopted `tokio::fs`
  or `tempfile` calls;
  `std::fs::copy` is not that form,
  because it overwrites an existing destination,
  so the exclusive copy must be specified and tested with the owner-kept fallback before the edge counts as solved),
  except `shell_quote`,
  which the owner chose to keep.
  None of these replacements is approved by this section:
  naming `tracing` or the `log` facade says only that a published form exists.
- So taking the verdicts makes two owner decisions necessary:
  how a library prints a log line
  (a facade crate that the program installs a printer for,
  or a tiny crate of ours,
  which would keep one library that the verdicts would otherwise remove),
  and where `shell_quote` lives
  (two repair files quote shell words).
- The conditional target graph for reading D,
  which holds only after the replacements in the `What the verdicts do to the table` section
  and the two owner decisions
  (the engine measured one crate loop and 19 edges until then):
  `starting-and-watching-programs` depends on `parallel-work-within-a-budget` (the shared runtime),
  `fix-packages-after-copying-a-folder` on both,
  and no library depends on the text or file-saving libraries.

## One level of groups in the program

Written 2026-11-07T10:43:12-05:00.
The owner answered the question of flat folders or grouping with,
verbatim,
"One level of groups".
So the program's 78 folders sit in 16 group folders and 10 one-folder entries,
one level deep:
a group folder holds child folders,
and a child folder holds files and its own `uses/` folder.
The owner still has to word the rule for how a group folder and its children relate
under "Two source folders never use each other" (LYL);
the section "The rule the owner must word" gives the options.

### How the groups were drafted

- The starting point was the 23 groups of `cli-git-source-layout.md`,
  section "Proposed top-level folder names",
  which the owner named none of as wrong;
  the groups that went to libraries are gone and the groups for the new folders are added.
- The groups were then checked against the folder graph of the scan
  (`~/temp/agent/wrapper-layout-after-crates/groups-design.py` and `folder-edges.json`,
  492 uses of one program folder by another,
  both from the placement of the rescan).
  A group by theme alone was not enough:
  the first draft had two loops between groups
  (the real-Git group and the lock group used each other,
  and the extra working folders and the btrfs folders used each other).
  Six folders moved off their theme to break them:
  `clearing_leftover_reference_locks` to the real-Git group,
  `converting_a_linked_worktree` to the extra working folders,
  and the four folders for messages,
  log lines,
  saved notes and service naming to a group of their own that the lock group uses.
  The final draft has no loop between groups,
  checked by the script (strongly connected sets of two or more groups:
  none).
- The group map keeps the draft under `after_adopted_crates.program_groups`
  (entries,
  group links and the order from the bottom).
- The result is 26 entries directly in `src/`:
  16 group folders with two or more child folders,
  and 10 entries that hold one folder only.
  A one-folder entry needs no group folder,
  so the folder sits directly in `src/` under its own name,
  and other entries' `uses/` links name it by that name.
  The 78 folders are all placed (the script's `missing` and `extra` lists are empty).
- The entries.
  The group `uses/` links shown are the summaries of Option B in the section "The rule the owner must word",
  which the installed lint rejects today;
  an initial layout under Option A would omit them and keep the exact links of the child folders
  (listed in the paragraph that starts "The children's own `uses/` links").
  Listed in dependency order,
  with each entry using only entries already listed:
  - `understanding_the_git_command_you_typed/` (a group folder):
    Reads the words typed after `git` the way Git reads them,
    and the words meant only for this wrapper.
    Child folders:
    `reading_options_the_way_git_does`,
    `spellings_that_switch_the_wrapper_off`,
    `the_words_meant_for_this_wrapper`,
    `what_each_git_command_accepts`.
    `uses/` links of the group:
    none.
  - `waits_logs_names_and_checksums_used_everywhere/` (a group folder):
    How long the program waits for a Git step,
    the log lines and journal entries it writes,
    the names of
    this release,
    checksums,
    and the other small things every other entry may use.
    Child folders:
    `environment_variables_for_git_children`,
    `how_long_the_program_waits_for_git_steps`,
    `lines_about_worktrees_and_large_files`,
    `names_this_release_and_the_one_before`,
    `recording_where_worktree_add_spent_its_time`,
    `sending_log_entries_to_the_system_journal`,
    `sha_1_and_sha_256_of_bytes`,
    `stopping_the_program_at_a_named_step_for_tests`,
    `syncing_files_off_the_waiting_path`,
    `who_owns_a_transaction_folder`.
    `uses/` links of the group:
    none.
  - `reading_what_git_saved_on_disk/` (a group folder):
    Git keeps commits and settings as files in the hidden `.git` folder;
    this reads them directly,
    which is
    faster than starting Git.
    Child folders:
    `finding_the_git_folders`,
    `reading_and_writing_the_index_file_of_git`,
    `reading_branches_tags_and_markers`,
    `reading_git_settings_from_files`,
    `reading_saved_commits_and_files`,
    `settings_that_decide_how_the_index_is_read`,
    `state_shared_by_preparing_and_landing_a_commit`.
    `uses/` links of the group:
    `waits_logs_names_and_checksums_used_everywhere`.
  - `running_the_real_git_program/` (a group folder):
    This wrapper only pretends to be `git`;
    this finds the real one,
    starts it,
    and answers some questions
    without starting it.
    Child folders:
    `answering_questions_without_starting_git`,
    `clearing_leftover_reference_locks`,
    `finding_the_real_git_program`,
    `git_settings_read_once`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`,
    `what_the_wrapper_asks_git_about_a_repository`.
    `uses/` links of the group:
    `reading_what_git_saved_on_disk`,
    `understanding_the_git_command_you_typed`,
    `waits_logs_names_and_checksums_used_everywhere`.
  - `rules_about_what_a_command_may_do/` (a group folder):
    The rules that refuse a command,
    add words to it,
    or say where it may run.
    Child folders:
    `rules_about_what_a_commit_may_contain`,
    `rules_about_where_a_command_may_run`,
    `rules_that_add_flags_to_a_command`.
    `uses/` links of the group:
    `running_the_real_git_program`,
    `understanding_the_git_command_you_typed`.
  - `running_git_push_until_it_ends_or_stalls/` (one folder,
    no group folder):
    Runs `git push`,
    stops it when it shows no sign of work,
    and runs one push at a time.
    `uses/` links of the entry:
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`,
    `waits_logs_names_and_checksums_used_everywhere`.
  - `what_the_wrapper_and_the_commit_helper_say_to_each_other/` (a group folder):
    The messages,
    log lines and saved "still to do" notes that pass between a command and the background
    commit helper.
    Child folders:
    `log_lines_of_the_commit_service`,
    `messages_between_the_wrapper_and_the_commit_service`,
    `naming_the_service_of_one_worktree`,
    `work_left_for_the_service_to_finish_later`.
    `uses/` links of the group:
    `running_the_real_git_program`,
    `waits_logs_names_and_checksums_used_everywhere`.
  - `your_settings_for_this_wrapper/` (a group folder):
    Reads the settings file where checks are turned on and off,
    and the hooks a repository sets.
    Child folders:
    `hooks_a_repository_configures`,
    `the_settings_file_cli_git_config_jsonc`.
    `uses/` links of the group:
    `running_the_real_git_program`,
    `understanding_the_git_command_you_typed`,
    `waits_logs_names_and_checksums_used_everywhere`.
  - `names_kept_from_the_earlier_release/` (a group folder):
    Moves files kept under the old program name to the new name,
    and checks whether the old commit helper
    still runs.
    Child folders:
    `moving_kept_files_to_the_new_names`,
    `telling_whether_an_earlier_commit_service_still_runs`.
    `uses/` links of the group:
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`.
  - `reading_what_a_commit_would_save/` (a group folder):
    Loads each file a commit would save,
    scans it for forbidden strings,
    and writes corrected files into the
    private index.
    Child folders:
    `putting_corrected_files_into_the_private_index`,
    `reading_the_bytes_a_commit_would_save`,
    `scanning_those_bytes_for_forbidden_strings`.
    `uses/` links of the group:
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `your_settings_for_this_wrapper`.
  - `reviewing_landed_commits_against_standards/` (one folder,
    no group folder):
    Reviews landed commits against the written standards with one HTTPS call and keeps the log of the
    answers.
    `uses/` links of the entry:
    `running_the_real_git_program`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`.
  - `uploading_large_files/` (one folder,
    no group folder):
    Git LFS keeps big files on a separate server;
    this uploads them before a push.
    `uses/` links of the entry:
    `reading_what_git_saved_on_disk`,
    `running_git_push_until_it_ends_or_stalls`,
    `running_the_real_git_program`,
    `understanding_the_git_command_you_typed`,
    `waits_logs_names_and_checksums_used_everywhere`.
  - `waiting_your_turn_to_change_git_files/` (a group folder):
    Git puts a lock file beside a file it changes;
    this waits for those,
    holds the one the commit helper
    needs,
    and finds who holds one.
    Child folders:
    `finding_who_holds_a_lock_file`,
    `holding_the_index_lock_for_the_commit_service`,
    `waiting_for_the_index_lock_of_git`.
    `uses/` links of the group:
    `running_the_real_git_program`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`.
  - `combining_two_versions_of_the_same_file/` (a group folder):
    When two people changed the same file,
    this merges the text,
    tries again word by word,
    and finishes a
    merge a dead process left half done.
    Child folders:
    `merging_text_by_line_and_by_word`,
    `recovering_a_merge_that_was_cut_short`,
    `rerunning_a_merge_word_by_word`.
    `uses/` links of the group:
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`,
    `waiting_your_turn_to_change_git_files`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`.
  - `converting_the_main_checkout/` (one folder,
    no group folder):
    btrfs is a kind of disk that can copy a whole folder instantly;
    this converts the main project folder so
    it can be copied that way.
    `uses/` links of the entry:
    `names_kept_from_the_earlier_release`,
    `running_the_real_git_program`,
    `waits_logs_names_and_checksums_used_everywhere`.
  - `a_hidden_copy_of_git_data_for_preparing_commits/` (a group folder):
    Prepares a commit in a hidden second Git folder,
    and re-applies prepared changes onto commits that
    landed meanwhile.
    Child folders:
    `replaying_changes_onto_new_commits`,
    `the_hidden_second_git_folder`.
    `uses/` links of the group:
    `combining_two_versions_of_the_same_file`,
    `reading_what_a_commit_would_save`,
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`.
  - `saving_copies_of_what_a_command_would_throw_away/` (one folder,
    no group folder):
    Saves a copy of work a command would throw away,
    records what happened,
    and removes old copies.
    `uses/` links of the entry:
    `a_hidden_copy_of_git_data_for_preparing_commits`,
    `reading_what_git_saved_on_disk`,
    `rules_about_what_a_command_may_do`,
    `running_the_real_git_program`,
    `understanding_the_git_command_you_typed`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`,
    `your_settings_for_this_wrapper`.
  - `checking_that_your_setup_works/` (one folder,
    no group folder):
    Prints the doctor report on hooks,
    settings,
    the disk and file watching.
    `uses/` links of the entry:
    `names_kept_from_the_earlier_release`,
    `running_the_real_git_program`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `your_settings_for_this_wrapper`.
  - `running_every_check_in_order/` (a group folder):
    Runs every check one after another for a commit or a push,
    including the version bump check and the
    Markdown fixer.
    Child folders:
    `checking_version_bumps_in_a_commit`,
    `fixing_markdown_in_process`,
    `running_policies_and_settling_their_corrections`,
    `the_built_in_policies`,
    `the_list_of_checks_every_commit_runs`.
    `uses/` links of the group:
    `names_kept_from_the_earlier_release`,
    `reading_what_a_commit_would_save`,
    `reading_what_git_saved_on_disk`,
    `rules_about_what_a_command_may_do`,
    `running_the_real_git_program`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `understanding_the_git_command_you_typed`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `your_settings_for_this_wrapper`.
  - `asking_the_commit_service_to_save_a_commit/` (one folder,
    no group folder):
    Asks the background helper to save a commit,
    starting it first when nothing answers.
    `uses/` links of the entry:
    `names_kept_from_the_earlier_release`,
    `reading_what_a_commit_would_save`,
    `reading_what_git_saved_on_disk`,
    `reviewing_landed_commits_against_standards`,
    `running_every_check_in_order`,
    `running_the_real_git_program`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`,
    `your_settings_for_this_wrapper`.
  - `pushing_after_a_commit_and_pushing_by_hand/` (a group folder):
    The push the wrapper makes after each commit,
    the push you type,
    and what happens when someone else
    pushed first.
    Child folders:
    `pushing_what_the_commit_service_saved`,
    `working_out_what_a_typed_push_would_send`.
    `uses/` links of the group:
    `asking_the_commit_service_to_save_a_commit`,
    `combining_two_versions_of_the_same_file`,
    `names_kept_from_the_earlier_release`,
    `reading_what_a_commit_would_save`,
    `reading_what_git_saved_on_disk`,
    `rules_about_what_a_command_may_do`,
    `running_git_push_until_it_ends_or_stalls`,
    `running_the_real_git_program`,
    `understanding_the_git_command_you_typed`,
    `uploading_large_files`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`.
  - `extra_working_folders_for_one_project/` (a group folder):
    Makes,
    converts,
    cleans and removes the extra working folders ("worktrees") of one project,
    copying the
    files Git ignores.
    Child folders:
    `converting_a_linked_worktree`,
    `copying_ignored_files_around_one_git_command`,
    `copying_ignored_files_into_a_stage`,
    `is_the_worktree_clean`,
    `making_a_worktree_quickly`,
    `planning_a_worktree_add_and_what_git_made`,
    `removing_a_linked_worktree`.
    `uses/` links of the group:
    `asking_the_commit_service_to_save_a_commit`,
    `converting_the_main_checkout`,
    `names_kept_from_the_earlier_release`,
    `pushing_after_a_commit_and_pushing_by_hand`,
    `reading_what_a_commit_would_save`,
    `reading_what_git_saved_on_disk`,
    `running_the_real_git_program`,
    `understanding_the_git_command_you_typed`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`.
  - `merging_without_touching_your_files/` (one folder,
    no group folder):
    Completes a merge or pull without touching staged work.
    `uses/` links of the entry:
    `combining_two_versions_of_the_same_file`,
    `pushing_after_a_commit_and_pushing_by_hand`,
    `running_every_check_in_order`,
    `running_the_real_git_program`,
    `understanding_the_git_command_you_typed`,
    `waiting_your_turn_to_change_git_files`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`,
    `your_settings_for_this_wrapper`.
  - `the_background_commit_helper/` (a group folder):
    The background helper itself:
    lands commits in batches,
    pushes them and tidies up,
    and finishes what an
    earlier helper left open.
    Child folders:
    `after_commit_work_of_the_service`,
    `finishing_what_an_earlier_commit_service_left_open`,
    `landing_commits`,
    `the_service_program`.
    `uses/` links of the group:
    `a_hidden_copy_of_git_data_for_preparing_commits`,
    `asking_the_commit_service_to_save_a_commit`,
    `extra_working_folders_for_one_project`,
    `names_kept_from_the_earlier_release`,
    `pushing_after_a_commit_and_pushing_by_hand`,
    `reading_what_git_saved_on_disk`,
    `reviewing_landed_commits_against_standards`,
    `running_git_push_until_it_ends_or_stalls`,
    `running_the_real_git_program`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `uploading_large_files`,
    `waiting_your_turn_to_change_git_files`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`.
  - `the_commit_command/` (one folder,
    no group folder):
    The `git commit` command:
    prepares the commit aside,
    asks the helper to land it,
    and reports.
    `uses/` links of the entry:
    `a_hidden_copy_of_git_data_for_preparing_commits`,
    `asking_the_commit_service_to_save_a_commit`,
    `combining_two_versions_of_the_same_file`,
    `extra_working_folders_for_one_project`,
    `reading_what_a_commit_would_save`,
    `reading_what_git_saved_on_disk`,
    `rules_about_what_a_command_may_do`,
    `running_every_check_in_order`,
    `running_the_real_git_program`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `the_background_commit_helper`,
    `understanding_the_git_command_you_typed`,
    `uploading_large_files`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`,
    `your_settings_for_this_wrapper`.
  - `what_happens_when_you_type_a_command/` (one folder,
    no group folder):
    The first stop of every command:
    reads what was typed,
    then hands it to the real Git,
    runs one of the
    others,
    or refuses and says why.
    `uses/` links of the entry:
    `asking_the_commit_service_to_save_a_commit`,
    `checking_that_your_setup_works`,
    `combining_two_versions_of_the_same_file`,
    `converting_the_main_checkout`,
    `extra_working_folders_for_one_project`,
    `merging_without_touching_your_files`,
    `names_kept_from_the_earlier_release`,
    `pushing_after_a_commit_and_pushing_by_hand`,
    `reading_what_a_commit_would_save`,
    `reading_what_git_saved_on_disk`,
    `rules_about_what_a_command_may_do`,
    `running_every_check_in_order`,
    `running_git_push_until_it_ends_or_stalls`,
    `running_the_real_git_program`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `the_background_commit_helper`,
    `the_commit_command`,
    `understanding_the_git_command_you_typed`,
    `uploading_large_files`,
    `waiting_your_turn_to_change_git_files`,
    `waits_logs_names_and_checksums_used_everywhere`,
    `what_the_wrapper_and_the_commit_helper_say_to_each_other`,
    `your_settings_for_this_wrapper`.

- The children's own `uses/` links:
  the lists of section "Inside each crate" stand for the 74 older folders,
  with these differences:
  `messages_between_the_wrapper_and_the_commit_service` uses nothing of the review folder
  (`service_review_key` is now its own file),
  the new folders and their links are in the group map under `after_adopted_crates.program_folder_uses`,
  which holds all 78 folders' links as the scan computed them.
- Folder names that no longer say what they hold after the rescan:
  `lines_about_worktrees_and_large_files` (the credential masker went to the text library;
  only the worktree path failure message is left,
  so a name like `words_for_a_failed_step_on_a_worktree_path` fits),
  and `sha_1_and_sha_256_of_bytes` (two calls into `sha1` and `sha2` since commit `1aea0ae238`;
  the folder can dissolve into its users).
  Neither is renamed here.
- Names that fail MKY alone today and are for the veto round,
  not settled by it:
  `the_service_program` (which service,
  and what it does),
  `the_built_in_policies` (what the policies check),
  `after_commit_work_of_the_service` (which work),
  `syncing_files_off_the_waiting_path` (scheduling jargon),
  `scanning_those_bytes_for_forbidden_strings` ("those" needs its context),
  `who_owns_a_transaction_folder` (transaction is unexplained),
  and `clearing_leftover_reference_locks` (Git's reference lock needs its top doc).
  Each either gets a plainer name or a top doc that explains the term to someone who knows only `git commit` and `git push`.

### The rule the owner must word

Today's rule (`AGENTS.md`,
LYL,
verbatim):
"Two source folders never use each other,
and the package's lint fails when a folder's `uses/` folder of links and its code disagree."
A group folder is a second kind of folder.
The question is whether it is a source folder,
and what links it holds.
The options were tried on a throwaway crate with the installed `monochromatic-lint` 0.5.1
(`~/temp/agent/wrapper-layout-after-crates/groupfixture/`:
two groups `g` and `h`,
children `g/a`,
`g/b` and `h/c`,
`h/c` using `g/a`,
both rules on at severity error),
so what each lint would check is measured where the installed linter already decides it.

- Option A,
  the group is a shelf:
  a group folder holds only child folders and its own top doc and `mod` lines;
  it is not a source folder and has no `uses/`;
  every child's `uses/` names the exact child folders it uses,
  inside its group or in another group
  (`h/c/uses/a` is a link to `../../../g/a`).
  Draft wording:
  "A group folder holds only folders and its top doc.
  Two source folders never use each other,
  and the package's lint fails when a source folder's `uses/` links and its code disagree."
  - Pros:
    the installed lint already enforces its links and its folder-loop check and needs no change for them
    (measured:
    without links it printed one finding for `g/b` using `g/a` and one for `h/c` using `g/a`,
    with exact child links it printed none);
    the links keep the precision of today's rule,
    so a child cannot use a folder it did not declare;
    it can start with step 8 of the move order and loses nothing if a group level is added later;
    the folder graph is already loop free (78 folders,
    0 loops),
    so no regrouping is needed to pass.
  - Cons:
    the dependencies between groups are not visible from the group folders;
    to see which groups a group uses one opens every child's `uses/` and compares,
    which is less than principle 3 asks;
    432 of the 492 links cross groups,
    so most links climb out of a group.
  - Example from this tree:
    `rules_about_what_a_commit_may_contain` in the group `rules_about_what_a_command_may_do` links its sibling
    `rules_that_add_flags_to_a_command`
    and,
    in other groups,
    `reading_options_the_way_git_does`,
    `what_each_git_command_accepts` and `what_the_wrapper_asks_git_about_a_repository`.
  - What the lint checks:
    what it checks today,
    per child folder;
    a group folder that grows code of its own is then a source folder and needs links to its children
    (measured:
    a `pub use a::x::X;` in `g/mod.rs` printed "src/g uses the folder src/g/a ... but src/g/uses has no link to it").
    The installed lint does not enforce the shelf-only restriction:
    a properly linked re-export in a group folder would pass it.
    Enforcing it mechanically needs a new check that fails a group folder holding a file other than its top doc and `mod` lines;
    without that check the restriction is a convention.
- Option B,
  the shelf and a group-level summary:
  Option A plus,
  in each group folder,
  a `uses/` folder with one link per other entry that any of its children uses.
  Draft wording:
  "A group folder holds only folders,
  its top doc and a `uses/` folder with one link per other group or one-folder entry
  its children use.
  The lint fails when the group's links and its children's code disagree."
  - Pros:
    principle 3 is answered at two scales:
    the 26 entries are readable on their own (160 links among them),
    and each child still names the exact folders;
    the group links can be written by the same fix that writes today's links.
  - Cons:
    the installed lint rejects it today
    (measured:
    a link `h/uses/g` printed "links to the folder src/g,
    but no code in src/h uses it"),
    so it needs a change in `package/linter/monochromatic-lint`;
    the group links repeat what the children say,
    so every new cross-group use edits two `uses/` folders;
    the group graph must stay loop free,
    which the drafted groups are only after six folders moved;
    one-folder entries have no group level.
  - Example from this tree:
    `running_every_check_in_order` links `reading_what_a_commit_would_save`,
    `rules_about_what_a_command_may_do`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `your_settings_for_this_wrapper`
    and five more entries,
    and its child `the_list_of_checks_every_commit_runs` links exact folders such as `rules_about_where_a_command_may_run`
    and `saving_copies_of_what_a_command_would_throw_away`.
  - What the lint checks:
    Option A's checks,
    plus for every group:
    each group link must be used by some child,
    each child use of another entry must be linked at the group,
    and the groups must not form a loop.
- Option C,
  groups hold the cross-group links:
  a child's `uses/` links only siblings in its own group;
  its uses of other entries are declared once,
  by the group's `uses/`.
  A one-folder entry is a source folder with no group,
  so under B and C it keeps exact links to the entries it uses,
  as in A.
  Draft wording:
  "A folder's `uses/` links the folders of its group that it uses;
  a group's `uses/` links the groups its children use.
  The lint fails when a link and the code disagree."
  - Pros:
    the children's `uses/` folders get short (60 of the 492 links stay inside a group);
    the overview is complete at group level.
  - Cons:
    the installed lint rejects it today
    (measured:
    with `h/c/uses/a` removed and only `h/uses/g` kept,
    it printed the group link as unused and the child's use as unlinked);
    a child may use any folder of a linked group without saying so,
    which is weaker than today's rule and breaks "who depends on who" at the folder level
    (principle 3 asks for it at the file structure,
    and LN7 asks for approval before loosening a rule);
    a group of one sibling gets no links of its own.
  - Example from this tree:
    `making_a_worktree_quickly` would show only its siblings in `extra_working_folders_for_one_project`
    (`planning_a_worktree_add_and_what_git_made`,
    `copying_ignored_files_into_a_stage`,
    `is_the_worktree_clean`);
    its uses of the commit helper and the btrfs folder would be on the group.
  - What the lint checks:
    a new rule for sibling-only child links,
    and Option B's group checks.

Ranking:
B > A > C,
because:

- B over A:
  only B shows the dependencies between the 26 entries from the file structure,
  and it keeps A's exact child links,
  so it costs a lint change and a second set of links where A costs nothing;
  A is the first step of B,
  so "A now,
  B when the linter has it" is reachable and moves no folder.
- A over C:
  A keeps today's rule at today's precision and passes the installed lint,
  while C weakens the precision of the child links
  and needs a new rule to be checked at all.

Under A any theme can group folders,
because only child folders are linked;
under B and C the groups themselves must not use each other in a loop,
which is what moved six folders off their theme in the draft.

## Inside each crate

Note 2026-11-26:
the folder list below is the 2026-11-07 proposal,
74 program folders and 14 library folders,
and it is superseded by section "Rescan of main after the reformat (2026-11-26)",
which places today's files in 110 program folders and 13 library folders and names the 32 folders
today's tree adds.
The file-by-file list that follows the table is in the companion file,
regenerated from that placement.
Kept as the record of the proposal the owner has not answered yet.

- A crate with one folder keeps its files directly in `src/`,
  listed in dependency order in the top doc of `lib.rs`.
  Its folder name from the table is only a label and appears nowhere on disk.
- A crate with several folders holds one folder per label below,
  each with a `mod.rs` that lists its files in
  dependency order,
  and a `uses/` folder of links to the folders of the same crate it uses.
  A crate root `lib.rs` holds only `mod` lines.
- Superseded 2026-11-07T10:43:12-05:00:
  the owner answered "One level of groups",
  so a child folder of the program sits inside a group folder,
  one level deep and no more;
  section "One level of groups in the program" holds the groups and the options for how a group relates to its children.
  The libraries' folders stay as drafted,
  because the owner's words name the program.
- Folder names are plain sentences.
  The folders that exist on disk,
  with their `uses/` links.
  A program folder never links to a library;
  the program's `Cargo.toml` lists the libraries it uses.
  The single-folder libraries (`safe-file-saving`,
  `parallel-work-within-a-budget`,
  `talking-to-the-systemd-user-manager`,
  `fix-packages-after-copying-a-folder`,
  `dependent-version-bump-check`) keep their files directly in `src/`
  and have no folder list here:

- `text-time-and-log-line-formats` (2 folders,
  5 files)
  - `text_time_and_records_written_like_javascript/`:
    Formats that must equal JavaScript's output.
    `uses/` links:
    none.
  - `lines_printed_for_the_person/`:
    Debug and warning lines on standard error,
    and quoting text for a shell command.
    `uses/` links:
    `text_time_and_records_written_like_javascript`.
- `starting-and-watching-programs` (3 folders,
  19 files)
  - `starting_programs_catching_signals_and_keeping_their_output/`:
    Starts children,
    relays signals and keeps their output in files.
    `uses/` links:
    none.
  - `which_programs_are_running_and_since_when/`:
    Process facts:
    birth identity,
    start time,
    tree,
    users of a file,
    and network connections.
    `uses/` links:
    none.
  - `watching_a_started_program_until_it_stops_making_progress/`:
    Watches a child and ends it when it shows no sign of work.
    `uses/` links:
    `starting_programs_catching_signals_and_keeping_their_output`,
    `which_programs_are_running_and_since_when`.
- `linux-file-and-disk-calls` (3 folders,
  9 files)
  - `which_disk_a_folder_is_on/`:
    Mount tables,
    the kind of disk,
    and why a disk call failed.
    `uses/` links:
    none.
  - `fast_copies_and_swaps_on_linux/`:
    Cloning a file's data and exchanging two paths.
    `uses/` links:
    `which_disk_a_folder_is_on`.
  - `file_operations_that_never_follow_a_link/`:
    File operations relative to an open folder that never follow a link.
    `uses/` links:
    none.
- `cli` (74 folders,
  522 files)
  - `environment_variables_for_git_children/`:
    The environment variables added for every real Git the program starts.
    `uses/` links:
    none.
  - `reading_git_settings_from_files/`:
    Reads Git's settings files and proves what they say at startup.
    `uses/` links:
    `environment_variables_for_git_children`.
  - `finding_the_git_folders/`:
    Finds the Git folder and the shared Git folder of a command without starting Git.
    `uses/` links:
    `environment_variables_for_git_children`,
    `reading_git_settings_from_files`.
  - `reading_and_writing_the_index_file_of_git/`:
    Reads and writes the index file Git keeps in the hidden `.git` folder,
    with its extensions.
    `uses/` links:
    none.
  - `reading_saved_commits_and_files/`:
    Reads objects,
    packs,
    trees and index entries.
    `uses/` links:
    `finding_the_git_folders`,
    `reading_and_writing_the_index_file_of_git`.
  - `state_shared_by_preparing_and_landing_a_commit/`:
    The repository state that preparing and landing a commit both name.
    `uses/` links:
    none.
  - `reading_branches_tags_and_markers/`:
    Reads branches,
    tags and the markers of a merge or rebase in progress.
    `uses/` links:
    `reading_saved_commits_and_files`,
    `state_shared_by_preparing_and_landing_a_commit`.
  - `finding_the_real_git_program/`:
    Finds the real `git` on the computer and checks its version.
    `uses/` links:
    `environment_variables_for_git_children`.
  - `reading_options_the_way_git_does/`:
    Reads one command's options exactly as Git reads them.
    `uses/` links:
    none.
  - `spellings_that_switch_the_wrapper_off/`:
    The spellings of the escape hatches that several files share.
    `uses/` links:
    none.
  - `what_each_git_command_accepts/`:
    The option tables of Git commands,
    one per command,
    and the list of Git's own commands.
    `uses/` links:
    `reading_options_the_way_git_does`,
    `spellings_that_switch_the_wrapper_off`.
  - `the_words_meant_for_this_wrapper/`:
    The control words that only this wrapper understands,
    and which worktree a command targets.
    `uses/` links:
    `reading_options_the_way_git_does`,
    `spellings_that_switch_the_wrapper_off`,
    `what_each_git_command_accepts`.
  - `starting_git_and_describing_a_run/`:
    Starts real Git with hooks off or with the caller's arguments,
    and describes a run and its answer.
    `uses/` links:
    `environment_variables_for_git_children`,
    `finding_the_real_git_program`,
    `the_words_meant_for_this_wrapper`,
    `what_each_git_command_accepts`.
  - `answering_questions_without_starting_git/`:
    Answers the wrapper's own read-only Git queries from files.
    `uses/` links:
    `finding_the_git_folders`,
    `reading_and_writing_the_index_file_of_git`,
    `reading_branches_tags_and_markers`,
    `reading_git_settings_from_files`,
    `reading_saved_commits_and_files`,
    `starting_git_and_describing_a_run`.
  - `how_long_the_program_waits_for_git_steps/`:
    The kinds of Git step,
    the longest waits,
    and the push deadline setting.
    `uses/` links:
    none.
  - `who_owns_a_transaction_folder/`:
    The record of which process owns a commit-transaction folder,
    and the rule for owners in other containers.
    `uses/` links:
    `how_long_the_program_waits_for_git_steps`.
  - `clearing_leftover_reference_locks/`:
    Judges and removes the reference lock files a killed Git left behind.
    `uses/` links:
    `finding_the_git_folders`,
    `who_owns_a_transaction_folder`.
  - `recording_where_worktree_add_spent_its_time/`:
    The record of where one wrapped worktree creation spent its time.
    `uses/` links:
    none.
  - `running_git_for_a_transaction/`:
    Runs real Git for a commit,
    with its limits and the rules for running a step again.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `clearing_leftover_reference_locks`,
    `finding_the_real_git_program`,
    `how_long_the_program_waits_for_git_steps`,
    `recording_where_worktree_add_spent_its_time`,
    `starting_git_and_describing_a_run`,
    `what_each_git_command_accepts`.
  - `git_settings_read_once/`:
    One read of every Git setting,
    and which parts Git flushes to disk.
    `uses/` links:
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`.
  - `finding_who_holds_a_lock_file/`:
    Finds the programs that hold a lock file open and gathers the evidence.
    `uses/` links:
    none.
  - `lines_about_worktrees_and_large_files/`:
    Hides passwords in LFS URLs and words the message for a failed step on a worktree path.
    `uses/` links:
    none.
  - `naming_the_service_of_one_worktree/`:
    The name of the service instance that belongs to a worktree.
    `uses/` links:
    none.
  - `sending_log_entries_to_the_system_journal/`:
    Sends log entries to the systemd journal over its native socket.
    `uses/` links:
    none.
  - `log_lines_of_the_commit_service/`:
    The lines that say where time went in a commit and which Git runs the service started.
    `uses/` links:
    `lines_about_worktrees_and_large_files`,
    `naming_the_service_of_one_worktree`,
    `recording_where_worktree_add_spent_its_time`,
    `sending_log_entries_to_the_system_journal`,
    `starting_git_and_describing_a_run`.
  - `reviewing_landed_commits_against_standards/`:
    Reviews landed commits against the written standards with one HTTPS call and keeps the log of the answers.
    `uses/` links:
    `how_long_the_program_waits_for_git_steps`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`.
  - `messages_between_the_wrapper_and_the_commit_service/`:
    The typed messages,
    their wire form and the checks on names.
    `uses/` links:
    `reviewing_landed_commits_against_standards`.
  - `sha_1_and_sha_256_of_bytes/`:
    Computes SHA-1 and SHA-256 of bytes,
    written out in full.
    `uses/` links:
    none.
  - `stopping_the_program_at_a_named_step_for_tests/`:
    Lets a test stop or pause the program at a named step by setting an environment variable.
    `uses/` links:
    `how_long_the_program_waits_for_git_steps`.
  - `syncing_files_off_the_waiting_path/`:
    Decides where the service's file flushes wait.
    `uses/` links:
    `stopping_the_program_at_a_named_step_for_tests`.
  - `holding_the_index_lock_for_the_commit_service/`:
    Places,
    marks,
    holds and reclaims `index.lock` for the service.
    `uses/` links:
    `finding_who_holds_a_lock_file`,
    `how_long_the_program_waits_for_git_steps`,
    `log_lines_of_the_commit_service`,
    `messages_between_the_wrapper_and_the_commit_service`,
    `sha_1_and_sha_256_of_bytes`,
    `stopping_the_program_at_a_named_step_for_tests`,
    `syncing_files_off_the_waiting_path`.
  - `reading_the_bytes_a_commit_would_save/`:
    Loads each file a commit would save,
    from the index,
    the commit before it or a prepared copy.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `finding_the_git_folders`,
    `how_long_the_program_waits_for_git_steps`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`.
  - `putting_corrected_files_into_the_private_index/`:
    Writes a corrected file into the private index a commit is prepared in.
    `uses/` links:
    `reading_the_bytes_a_commit_would_save`,
    `running_git_for_a_transaction`.
  - `rules_that_add_flags_to_a_command/`:
    Rules that add words to a command line before it runs.
    `uses/` links:
    `reading_options_the_way_git_does`,
    `what_each_git_command_accepts`.
  - `what_the_wrapper_asks_git_about_a_repository/`:
    Asks Git where the repository is and which worktree a command targets.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `finding_the_git_folders`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`,
    `the_words_meant_for_this_wrapper`.
  - `rules_about_what_a_commit_may_contain/`:
    Rules about what a commit may contain and why a push scope is unknown.
    `uses/` links:
    `reading_options_the_way_git_does`,
    `rules_that_add_flags_to_a_command`,
    `what_each_git_command_accepts`,
    `what_the_wrapper_asks_git_about_a_repository`.
  - `rules_about_where_a_command_may_run/`:
    Rules that say a command may run only in a certain kind of worktree.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `reading_options_the_way_git_does`,
    `spellings_that_switch_the_wrapper_off`,
    `the_words_meant_for_this_wrapper`,
    `what_each_git_command_accepts`.
  - `the_settings_file_cli_git_config_jsonc/`:
    Finds,
    reads and checks `cli-git.config.jsonc`.
    `uses/` links:
    `the_words_meant_for_this_wrapper`,
    `what_each_git_command_accepts`,
    `what_the_wrapper_asks_git_about_a_repository`.
  - `scanning_those_bytes_for_forbidden_strings/`:
    Runs the repository scanner over those bytes and names its failures.
    `uses/` links:
    `putting_corrected_files_into_the_private_index`,
    `reading_the_bytes_a_commit_would_save`,
    `running_git_for_a_transaction`,
    `the_settings_file_cli_git_config_jsonc`.
  - `running_policies_and_settling_their_corrections/`:
    Runs the configured policies in order for one point in a commit or push,
    and settles their corrections.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `finding_the_git_folders`,
    `how_long_the_program_waits_for_git_steps`,
    `putting_corrected_files_into_the_private_index`,
    `reading_options_the_way_git_does`,
    `reading_the_bytes_a_commit_would_save`,
    `rules_about_what_a_commit_may_contain`,
    `rules_about_where_a_command_may_run`,
    `rules_that_add_flags_to_a_command`,
    `running_git_for_a_transaction`,
    `scanning_those_bytes_for_forbidden_strings`,
    `starting_git_and_describing_a_run`,
    `the_settings_file_cli_git_config_jsonc`,
    `the_words_meant_for_this_wrapper`,
    `what_the_wrapper_asks_git_about_a_repository`.
  - `work_left_for_the_service_to_finish_later/`:
    Keeps small files that say "this landed commit still needs a push" so a crash loses nothing.
    `uses/` links:
    `log_lines_of_the_commit_service`.
  - `asking_the_commit_service_to_save_a_commit/`:
    Asks the background commit service to save a commit,
    starting the service first when nothing answers.
    `uses/` links:
    `finding_the_git_folders`,
    `git_settings_read_once`,
    `log_lines_of_the_commit_service`,
    `messages_between_the_wrapper_and_the_commit_service`,
    `naming_the_service_of_one_worktree`,
    `reading_saved_commits_and_files`,
    `reading_the_bytes_a_commit_would_save`,
    `recording_where_worktree_add_spent_its_time`,
    `reviewing_landed_commits_against_standards`,
    `running_git_for_a_transaction`,
    `running_policies_and_settling_their_corrections`,
    `starting_git_and_describing_a_run`,
    `stopping_the_program_at_a_named_step_for_tests`,
    `the_settings_file_cli_git_config_jsonc`,
    `work_left_for_the_service_to_finish_later`.
  - `running_git_push_until_it_ends_or_stalls/`:
    Runs `git push` with its own deadline,
    stops it when it shows no sign of work,
    and runs one push at a time.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `clearing_leftover_reference_locks`,
    `finding_the_git_folders`,
    `finding_the_real_git_program`,
    `how_long_the_program_waits_for_git_steps`,
    `starting_git_and_describing_a_run`.
  - `the_hidden_second_git_folder/`:
    Creates the hidden second Git folder and copies merge state into it.
    `uses/` links:
    `finding_the_git_folders`,
    `git_settings_read_once`,
    `log_lines_of_the_commit_service`,
    `reading_branches_tags_and_markers`,
    `running_git_for_a_transaction`,
    `sending_log_entries_to_the_system_journal`,
    `starting_git_and_describing_a_run`,
    `state_shared_by_preparing_and_landing_a_commit`,
    `what_the_wrapper_asks_git_about_a_repository`.
  - `saving_copies_of_what_a_command_would_throw_away/`:
    Saves a copy of work that a command would throw away,
    records it,
    and removes old copies.
    `uses/` links:
    `finding_the_git_folders`,
    `messages_between_the_wrapper_and_the_commit_service`,
    `reading_branches_tags_and_markers`,
    `reading_options_the_way_git_does`,
    `reading_saved_commits_and_files`,
    `recording_where_worktree_add_spent_its_time`,
    `rules_about_where_a_command_may_run`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`,
    `the_hidden_second_git_folder`,
    `the_settings_file_cli_git_config_jsonc`,
    `the_words_meant_for_this_wrapper`,
    `what_each_git_command_accepts`.
  - `settings_that_decide_how_the_index_is_read/`:
    The settings that decide what Git does to index extensions.
    `uses/` links:
    `reading_git_settings_from_files`.
  - `waiting_for_the_index_lock_of_git/`:
    Waits for any program's hold on the lock before a command changes the checkout.
    `uses/` links:
    `finding_who_holds_a_lock_file`,
    `holding_the_index_lock_for_the_commit_service`,
    `how_long_the_program_waits_for_git_steps`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`,
    `the_words_meant_for_this_wrapper`.
  - `landing_commits/`:
    Lands a batch of prepared commits on a branch and updates the index without starting Git.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `asking_the_commit_service_to_save_a_commit`,
    `finding_the_git_folders`,
    `git_settings_read_once`,
    `holding_the_index_lock_for_the_commit_service`,
    `how_long_the_program_waits_for_git_steps`,
    `log_lines_of_the_commit_service`,
    `messages_between_the_wrapper_and_the_commit_service`,
    `naming_the_service_of_one_worktree`,
    `reading_and_writing_the_index_file_of_git`,
    `reading_git_settings_from_files`,
    `reading_saved_commits_and_files`,
    `running_git_for_a_transaction`,
    `running_git_push_until_it_ends_or_stalls`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `sending_log_entries_to_the_system_journal`,
    `settings_that_decide_how_the_index_is_read`,
    `sha_1_and_sha_256_of_bytes`,
    `starting_git_and_describing_a_run`,
    `state_shared_by_preparing_and_landing_a_commit`,
    `stopping_the_program_at_a_named_step_for_tests`,
    `syncing_files_off_the_waiting_path`,
    `the_hidden_second_git_folder`,
    `waiting_for_the_index_lock_of_git`.
  - `converting_the_main_checkout/`:
    Turns a project folder into a btrfs subvolume,
    which can be copied instantly,
    and can undo it.
    `uses/` links:
    `how_long_the_program_waits_for_git_steps`,
    `reading_git_settings_from_files`,
    `sha_1_and_sha_256_of_bytes`,
    `starting_git_and_describing_a_run`,
    `who_owns_a_transaction_folder`.
  - `planning_a_worktree_add_and_what_git_made/`:
    Plans a `git worktree add` and binds it to what real Git created.
    `uses/` links:
    `git_settings_read_once`,
    `how_long_the_program_waits_for_git_steps`,
    `lines_about_worktrees_and_large_files`,
    `starting_git_and_describing_a_run`,
    `what_each_git_command_accepts`.
  - `copying_ignored_files_into_a_stage/`:
    Copies the ignored state of a worktree into a private stage with a journal.
    `uses/` links:
    `planning_a_worktree_add_and_what_git_made`,
    `recording_where_worktree_add_spent_its_time`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`,
    `the_words_meant_for_this_wrapper`,
    `who_owns_a_transaction_folder`.
  - `is_the_worktree_clean/`:
    Decides without Git whether a worktree is clean enough to copy from or remove.
    `uses/` links:
    `converting_the_main_checkout`,
    `finding_the_git_folders`,
    `lines_about_worktrees_and_large_files`,
    `reading_branches_tags_and_markers`,
    `reading_git_settings_from_files`,
    `reading_saved_commits_and_files`,
    `recording_where_worktree_add_spent_its_time`,
    `running_git_for_a_transaction`,
    `sha_1_and_sha_256_of_bytes`,
    `starting_git_and_describing_a_run`.
  - `making_a_worktree_quickly/`:
    The rungs that make a new worktree:
    snapshot,
    clone,
    slow route,
    trash handling.
    `uses/` links:
    `asking_the_commit_service_to_save_a_commit`,
    `converting_the_main_checkout`,
    `copying_ignored_files_into_a_stage`,
    `finding_the_git_folders`,
    `git_settings_read_once`,
    `how_long_the_program_waits_for_git_steps`,
    `is_the_worktree_clean`,
    `lines_about_worktrees_and_large_files`,
    `log_lines_of_the_commit_service`,
    `messages_between_the_wrapper_and_the_commit_service`,
    `planning_a_worktree_add_and_what_git_made`,
    `reading_saved_commits_and_files`,
    `reading_the_bytes_a_commit_would_save`,
    `recording_where_worktree_add_spent_its_time`,
    `running_git_for_a_transaction`,
    `sending_log_entries_to_the_system_journal`,
    `starting_git_and_describing_a_run`,
    `state_shared_by_preparing_and_landing_a_commit`,
    `the_words_meant_for_this_wrapper`,
    `work_left_for_the_service_to_finish_later`.
  - `recovering_a_merge_that_was_cut_short/`:
    Records an isolated merge and finishes or discards it if the process died.
    `uses/` links:
    `log_lines_of_the_commit_service`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`,
    `who_owns_a_transaction_folder`.
  - `merging_text_by_line_and_by_word/`:
    Merges text three ways by line and,
    where both sides changed a line,
    word by word.
    `uses/` links:
    none.
  - `rerunning_a_merge_word_by_word/`:
    Tries again word by word on the files a Git merge left conflicted.
    `uses/` links:
    `environment_variables_for_git_children`,
    `finding_the_git_folders`,
    `merging_text_by_line_and_by_word`,
    `reading_branches_tags_and_markers`,
    `reading_git_settings_from_files`,
    `reading_saved_commits_and_files`,
    `running_git_for_a_transaction`,
    `sending_log_entries_to_the_system_journal`,
    `starting_git_and_describing_a_run`,
    `waiting_for_the_index_lock_of_git`.
  - `uploading_large_files/`:
    Uploads Git LFS files (large pictures and videos) before a push.
    `uses/` links:
    `finding_the_git_folders`,
    `git_settings_read_once`,
    `how_long_the_program_waits_for_git_steps`,
    `lines_about_worktrees_and_large_files`,
    `reading_branches_tags_and_markers`,
    `reading_saved_commits_and_files`,
    `running_git_push_until_it_ends_or_stalls`,
    `starting_git_and_describing_a_run`,
    `the_words_meant_for_this_wrapper`,
    `what_each_git_command_accepts`.
  - `pushing_what_the_commit_service_saved/`:
    Pushes a landed commit in the background,
    retries,
    and merges when someone else pushed first.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `asking_the_commit_service_to_save_a_commit`,
    `finding_the_git_folders`,
    `git_settings_read_once`,
    `how_long_the_program_waits_for_git_steps`,
    `lines_about_worktrees_and_large_files`,
    `log_lines_of_the_commit_service`,
    `messages_between_the_wrapper_and_the_commit_service`,
    `naming_the_service_of_one_worktree`,
    `reading_branches_tags_and_markers`,
    `reading_git_settings_from_files`,
    `reading_saved_commits_and_files`,
    `recording_where_worktree_add_spent_its_time`,
    `recovering_a_merge_that_was_cut_short`,
    `rerunning_a_merge_word_by_word`,
    `running_git_push_until_it_ends_or_stalls`,
    `sending_log_entries_to_the_system_journal`,
    `starting_git_and_describing_a_run`,
    `uploading_large_files`,
    `who_owns_a_transaction_folder`,
    `work_left_for_the_service_to_finish_later`.
  - `after_commit_work_of_the_service/`:
    The work the service does after a landing:
    push,
    tidy-up and review.
    `uses/` links:
    `environment_variables_for_git_children`,
    `git_settings_read_once`,
    `holding_the_index_lock_for_the_commit_service`,
    `how_long_the_program_waits_for_git_steps`,
    `landing_commits`,
    `log_lines_of_the_commit_service`,
    `making_a_worktree_quickly`,
    `naming_the_service_of_one_worktree`,
    `pushing_what_the_commit_service_saved`,
    `reading_git_settings_from_files`,
    `reviewing_landed_commits_against_standards`,
    `running_git_for_a_transaction`,
    `running_git_push_until_it_ends_or_stalls`,
    `sending_log_entries_to_the_system_journal`,
    `starting_git_and_describing_a_run`,
    `uploading_large_files`,
    `work_left_for_the_service_to_finish_later`.
  - `hooks_a_repository_configures/`:
    Reads the hooks a repository sets and warns about hooks in the hooks folder.
    `uses/` links:
    `git_settings_read_once`,
    `starting_git_and_describing_a_run`,
    `the_settings_file_cli_git_config_jsonc`.
  - `checking_that_your_setup_works/`:
    Prints the doctor report on hooks,
    settings,
    the disk and file watching.
    `uses/` links:
    `git_settings_read_once`,
    `hooks_a_repository_configures`,
    `how_long_the_program_waits_for_git_steps`,
    `running_git_for_a_transaction`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `starting_git_and_describing_a_run`,
    `the_settings_file_cli_git_config_jsonc`.
  - `checking_version_bumps_in_a_commit/`:
    Reads the commit candidates for the version bump check and turns its plan into policy findings.
    `uses/` links:
    `putting_corrected_files_into_the_private_index`,
    `reading_the_bytes_a_commit_would_save`,
    `running_git_for_a_transaction`,
    `running_policies_and_settling_their_corrections`,
    `the_settings_file_cli_git_config_jsonc`,
    `what_the_wrapper_asks_git_about_a_repository`.
  - `converting_a_linked_worktree/`:
    Converts a linked worktree into a btrfs subvolume with a journal.
    `uses/` links:
    `converting_the_main_checkout`,
    `is_the_worktree_clean`,
    `lines_about_worktrees_and_large_files`.
  - `copying_ignored_files_around_one_git_command/`:
    Runs the ignored-file copy around one forwarded command.
    `uses/` links:
    `copying_ignored_files_into_a_stage`,
    `making_a_worktree_quickly`,
    `recording_where_worktree_add_spent_its_time`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`,
    `the_words_meant_for_this_wrapper`.
  - `fixing_markdown_in_process/`:
    Fixes Markdown files in the same process,
    with a guard against a stuck linter.
    `uses/` links:
    `how_long_the_program_waits_for_git_steps`,
    `reading_the_bytes_a_commit_would_save`,
    `running_git_for_a_transaction`,
    `running_policies_and_settling_their_corrections`,
    `the_settings_file_cli_git_config_jsonc`,
    `what_the_wrapper_asks_git_about_a_repository`.
  - `merging_without_touching_your_files/`:
    Completes a merge or pull without touching staged work.
    `uses/` links:
    `environment_variables_for_git_children`,
    `holding_the_index_lock_for_the_commit_service`,
    `log_lines_of_the_commit_service`,
    `pushing_what_the_commit_service_saved`,
    `recovering_a_merge_that_was_cut_short`,
    `rerunning_a_merge_word_by_word`,
    `running_git_for_a_transaction`,
    `running_policies_and_settling_their_corrections`,
    `sending_log_entries_to_the_system_journal`,
    `starting_git_and_describing_a_run`,
    `stopping_the_program_at_a_named_step_for_tests`,
    `the_settings_file_cli_git_config_jsonc`,
    `the_words_meant_for_this_wrapper`,
    `waiting_for_the_index_lock_of_git`,
    `who_owns_a_transaction_folder`.
  - `removing_a_linked_worktree/`:
    Removes a linked worktree that is a subvolume and rescues its unshown notices.
    `uses/` links:
    `is_the_worktree_clean`,
    `lines_about_worktrees_and_large_files`,
    `naming_the_service_of_one_worktree`,
    `pushing_what_the_commit_service_saved`,
    `starting_git_and_describing_a_run`,
    `work_left_for_the_service_to_finish_later`.
  - `replaying_changes_onto_new_commits/`:
    Re-applies prepared changes onto commits that landed in the meantime.
    `uses/` links:
    `environment_variables_for_git_children`,
    `reading_the_bytes_a_commit_would_save`,
    `rerunning_a_merge_word_by_word`,
    `running_git_for_a_transaction`,
    `starting_git_and_describing_a_run`,
    `the_hidden_second_git_folder`.
  - `the_built_in_policies/`:
    The policies the program ships.
    `uses/` links:
    `reading_the_bytes_a_commit_would_save`,
    `running_policies_and_settling_their_corrections`,
    `scanning_those_bytes_for_forbidden_strings`,
    `the_settings_file_cli_git_config_jsonc`,
    `what_the_wrapper_asks_git_about_a_repository`.
  - `the_list_of_checks_every_commit_runs/`:
    The one pass that runs every shipped policy.
    `uses/` links:
    `checking_version_bumps_in_a_commit`,
    `fixing_markdown_in_process`,
    `putting_corrected_files_into_the_private_index`,
    `rules_about_where_a_command_may_run`,
    `rules_that_add_flags_to_a_command`,
    `running_policies_and_settling_their_corrections`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `the_built_in_policies`,
    `the_settings_file_cli_git_config_jsonc`,
    `the_words_meant_for_this_wrapper`,
    `what_the_wrapper_asks_git_about_a_repository`.
  - `the_commit_command/`:
    The `git commit` command:
    prepares the commit aside,
    asks the service to land it,
    and reports.
    `uses/` links:
    `answering_questions_without_starting_git`,
    `asking_the_commit_service_to_save_a_commit`,
    `environment_variables_for_git_children`,
    `fixing_markdown_in_process`,
    `git_settings_read_once`,
    `hooks_a_repository_configures`,
    `how_long_the_program_waits_for_git_steps`,
    `landing_commits`,
    `log_lines_of_the_commit_service`,
    `making_a_worktree_quickly`,
    `messages_between_the_wrapper_and_the_commit_service`,
    `putting_corrected_files_into_the_private_index`,
    `reading_and_writing_the_index_file_of_git`,
    `reading_options_the_way_git_does`,
    `reading_the_bytes_a_commit_would_save`,
    `replaying_changes_onto_new_commits`,
    `rerunning_a_merge_word_by_word`,
    `rules_about_what_a_commit_may_contain`,
    `rules_about_where_a_command_may_run`,
    `running_git_for_a_transaction`,
    `running_policies_and_settling_their_corrections`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `scanning_those_bytes_for_forbidden_strings`,
    `sending_log_entries_to_the_system_journal`,
    `starting_git_and_describing_a_run`,
    `state_shared_by_preparing_and_landing_a_commit`,
    `stopping_the_program_at_a_named_step_for_tests`,
    `the_built_in_policies`,
    `the_hidden_second_git_folder`,
    `the_list_of_checks_every_commit_runs`,
    `the_settings_file_cli_git_config_jsonc`,
    `the_words_meant_for_this_wrapper`,
    `uploading_large_files`,
    `what_each_git_command_accepts`,
    `what_the_wrapper_asks_git_about_a_repository`,
    `work_left_for_the_service_to_finish_later`.
  - `the_service_program/`:
    The background commit service itself:
    accepts requests,
    lands them in batches and answers.
    `uses/` links:
    `after_commit_work_of_the_service`,
    `environment_variables_for_git_children`,
    `finding_the_real_git_program`,
    `holding_the_index_lock_for_the_commit_service`,
    `how_long_the_program_waits_for_git_steps`,
    `landing_commits`,
    `log_lines_of_the_commit_service`,
    `messages_between_the_wrapper_and_the_commit_service`,
    `naming_the_service_of_one_worktree`,
    `pushing_what_the_commit_service_saved`,
    `recording_where_worktree_add_spent_its_time`,
    `running_git_push_until_it_ends_or_stalls`,
    `sending_log_entries_to_the_system_journal`,
    `starting_git_and_describing_a_run`,
    `stopping_the_program_at_a_named_step_for_tests`,
    `syncing_files_off_the_waiting_path`,
    `work_left_for_the_service_to_finish_later`.
  - `working_out_what_a_typed_push_would_send/`:
    Works out what a typed `git push` would send and merges in a race that was lost.
    `uses/` links:
    `environment_variables_for_git_children`,
    `finding_the_real_git_program`,
    `how_long_the_program_waits_for_git_steps`,
    `lines_about_worktrees_and_large_files`,
    `pushing_what_the_commit_service_saved`,
    `reading_options_the_way_git_does`,
    `reading_the_bytes_a_commit_would_save`,
    `rules_about_what_a_commit_may_contain`,
    `running_git_for_a_transaction`,
    `running_git_push_until_it_ends_or_stalls`,
    `starting_git_and_describing_a_run`,
    `uploading_large_files`,
    `what_each_git_command_accepts`.
  - `what_happens_when_you_type_a_command/`:
    The first stop of every command:
    reads what was typed,
    then hands it to the real Git,
    runs one of the other folders,
    or refuses and says why.
    `uses/` links:
    `after_commit_work_of_the_service`,
    `asking_the_commit_service_to_save_a_commit`,
    `checking_that_your_setup_works`,
    `clearing_leftover_reference_locks`,
    `converting_a_linked_worktree`,
    `converting_the_main_checkout`,
    `copying_ignored_files_around_one_git_command`,
    `copying_ignored_files_into_a_stage`,
    `environment_variables_for_git_children`,
    `finding_the_git_folders`,
    `finding_the_real_git_program`,
    `git_settings_read_once`,
    `holding_the_index_lock_for_the_commit_service`,
    `hooks_a_repository_configures`,
    `how_long_the_program_waits_for_git_steps`,
    `merging_without_touching_your_files`,
    `planning_a_worktree_add_and_what_git_made`,
    `pushing_what_the_commit_service_saved`,
    `reading_options_the_way_git_does`,
    `reading_the_bytes_a_commit_would_save`,
    `recovering_a_merge_that_was_cut_short`,
    `removing_a_linked_worktree`,
    `rerunning_a_merge_word_by_word`,
    `rules_about_what_a_commit_may_contain`,
    `rules_about_where_a_command_may_run`,
    `running_git_for_a_transaction`,
    `running_git_push_until_it_ends_or_stalls`,
    `running_policies_and_settling_their_corrections`,
    `saving_copies_of_what_a_command_would_throw_away`,
    `scanning_those_bytes_for_forbidden_strings`,
    `starting_git_and_describing_a_run`,
    `the_commit_command`,
    `the_list_of_checks_every_commit_runs`,
    `the_service_program`,
    `the_settings_file_cli_git_config_jsonc`,
    `the_words_meant_for_this_wrapper`,
    `uploading_large_files`,
    `waiting_for_the_index_lock_of_git`,
    `what_each_git_command_accepts`,
    `what_the_wrapper_asks_git_about_a_repository`,
    `working_out_what_a_typed_push_would_send`.

- Every file's old path,
  new path and one-line explanation are in the companion file
  [`cli-git-source-layout-target-tree-files.md`](cli-git-source-layout-target-tree-files.md),
  one fenced tree per crate,
  files in the dependency order each top doc must list.
  The explanation of a file in a library is still the first sentence of its old top doc,
  so it may say Git;
  the new top doc is written without it (section "The crate split").
- File names:
  279 of the 590 placed files get a new name (the `target` table of the group map),
  the rest keep theirs because
  the name already says what the file does.
  Ten files are new (section "Changes the tree needs before files can move").
  The old folder `mod.rs` files and the old `lib.rs` are replaced by the new index files.
- Sidecar tests follow their subject:
  `<subject>_tests.rs` becomes `<new subject name>_tests.rs`,
  and a sidecar with an extra word (`candidate_store_lifecycle_tests.rs`) keeps the word after the new subject name.
  The `#[path = "..."]` line in the subject is the only place that names it.
- Jargon that no plain name replaces (index extension,
  pack,
  reference lock,
  btrfs subvolume,
  LFS,
  D-Bus) stays in the
  name where it is Git's or Linux's own word and is explained in that file's top doc,
  as MKY requires.

## Changes the tree needs before files can move

Every change is a commit that keeps the single crate building and its tests passing (see "Move order").
The loop checks in the What the scan counts now section hold only with all of these applied.
Where this section says "crate" for one of the draft's 46 libraries that is gone,
read the program folders listed in section "The crate split" under "Former crates and the program folders they became".

### Loops between files, and how each is broken (2026-11-26)

Read from the two allow lists of this commit,
which record every finding of the two layout rules,
and from the scanner for the items each use names.
The two sections of 2026-11-07 that this one replaces,
"Loops between files,
as they stand" and "Breaking each loop",
described the tree of `f44d921e24`,
and no loop they named has closed.
Two of the moves they named did land on their own:
`Patience` now lives in `push/push_deadline_and_patience_settings.rs`,
and `service_review_key` left the review folder for `service/`.
Four other moves came in sideways
(`StopCause` into `push/stop_cause_and_words.rs`,
`trash_location_is_ignored` into `worktree/worktree_trash_ignore_rule.rs`,
`IndexPaths` into `landing/service_land_follow_index_reset.rs`
and `CONFLICT_PHRASE` into `push/push_retry_failure.rs`)
changed the loops' shapes without closing any.

The file loops stand at 67,
touching 281 files over 560 uses;
the scanner,
which also counts the files a top doc does not list,
finds 61 loops over 261 modules and names no loop the lists do not hold
except `invocation/real_git` and `invocation/real_git_resolution`.
Both numbers are of the tree before any move,
so a round breaks them folder by folder,
deletes the lines of the uses it takes away,
and measures again when a folder has none.
Each break below is one commit and takes the smallest move that closes its loop:
an item moves to the file that is its only other user,
or a new file at the bottom of the folder receives what both sides name.
Where the 2026-11-07 plan already named the same move,
that wording is kept.

- Loop of 2,
  `command`:
  `command/rule_linked_worktree.rs`,
  `command/rule_linked_worktree_messages.rs`.
  `command/rule_linked_worktree.rs` uses `command/rule_linked_worktree_messages.rs` (`dirty_worktree_message`,
  `exemption_notice`,
  `ignored_file_message`,
  `main_worktree_message`,
  `outside_worktree_message`,
  `repository_stash_message`).
  `command/rule_linked_worktree_messages.rs` uses `command/rule_linked_worktree.rs` (`GuardedCommand`).
  Break:
  `GuardedCommand` moves from `rule_linked_worktree` into `rule_linked_worktree_messages`.

- Loop of 2,
  `commit`:
  `commit/scratch_step_held_by_the_kernel.rs`,
  `commit/scratch_step_stall_decided_once.rs`.
  `commit/scratch_step_held_by_the_kernel.rs` uses `commit/scratch_step_stall_decided_once.rs`
  (`WhyTheStepWasWaitedFor`).
  `commit/scratch_step_stall_decided_once.rs` uses `commit/scratch_step_held_by_the_kernel.rs` (`HeldProgram`).
  Break:
  `HeldProgram` moves from `scratch_step_held_by_the_kernel` into `scratch_step_stall_decided_once`.

- Loop of 2,
  `conflict`:
  `conflict/discard_audit.rs`,
  `conflict/discard_audit_record.rs`.
  `conflict/discard_audit.rs` uses `conflict/discard_audit_record.rs` (`AuditRecord`,
  `DecidedForm`,
  `audit_line`).
  `conflict/discard_audit_record.rs` uses `conflict/discard_audit.rs` (`AuditOutcome`).
  Break:
  `AuditOutcome` moves from `discard_audit` into `discard_audit_record`.

- Loop of 2,
  `dependent_version`:
  `dependent_version/dependent_version_yaml_lines.rs`,
  `dependent_version/dependent_version_yaml_lines_scalar.rs`.
  `dependent_version/dependent_version_yaml_lines.rs` uses `dependent_version/dependent_version_yaml_lines_scalar.rs`
  (the child module).
  `dependent_version/dependent_version_yaml_lines_scalar.rs` uses `dependent_version/dependent_version_yaml_lines.rs`
  (the child module).
  Break:
  `dependent_version_yaml_lines_scalar` is declared in `mod.rs` instead of by `#[path]` under its parent,
  since it reads nothing from the parent today,
  and its own items stay where they are.

- Loop of 2,
  `foundation`:
  `foundation/private_storage.rs`,
  `foundation/private_storage_replacing_a_record.rs`.
  `foundation/private_storage.rs` uses `foundation/private_storage_replacing_a_record.rs` (the child module).
  `foundation/private_storage_replacing_a_record.rs` uses `foundation/private_storage.rs` (the child module).
  Break:
  `private_storage_replacing_a_record` is declared in `mod.rs` instead of by `#[path]`,
  and
  `AFTER_THE_STAGING_FILE_IS_WRITTEN` moves into it.

- Loop of 2,
  `invocation/typed_network_command_relay`:
  `invocation/typed_network_command_relay/git_error_stream_relayed_as_it_arrives.rs`,
  `invocation/typed_network_command_relay/the_relayed_bytes.rs`.
  `invocation/typed_network_command_relay/git_error_stream_relayed_as_it_arrives.rs` uses
  `invocation/typed_network_command_relay/the_relayed_bytes.rs` (`copy_what_is_readable`,
  `drain_what_is_left`,
  `make_non_blocking`).
  `invocation/typed_network_command_relay/the_relayed_bytes.rs` uses
  `invocation/typed_network_command_relay/git_error_stream_relayed_as_it_arrives.rs` (`KEPT_TAIL_BYTES`).
  Break:
  `KEPT_TAIL_BYTES` moves from `the_relayed_bytes` into `git_error_stream_relayed_as_it_arrives`.

- Loop of 2,
  `invocation/typed_network_command_relay`:
  `invocation/typed_network_command_relay/the_options_before_the_remote.rs`,
  `invocation/typed_network_command_relay/the_remote_a_typed_command_names.rs`.
  `invocation/typed_network_command_relay/the_options_before_the_remote.rs` uses
  `invocation/typed_network_command_relay/the_remote_a_typed_command_names.rs` (`TypedNetworkCommand`).
  `invocation/typed_network_command_relay/the_remote_a_typed_command_names.rs` uses
  `invocation/typed_network_command_relay/the_options_before_the_remote.rs` (`remote_operand`).
  Break:
  `remote_operand` moves from `the_options_before_the_remote` into `the_remote_a_typed_command_names`.

- Loop of 2,
  `kept_references`:
  `kept_references/kept_reference_log.rs`,
  `kept_references/kept_reference_pass.rs`.
  `kept_references/kept_reference_log.rs` uses `kept_references/kept_reference_pass.rs` (`Pruned`,
  `WhyPruned`).
  `kept_references/kept_reference_pass.rs` uses `kept_references/kept_reference_log.rs` (`log_one_pruning`).
  Break:
  `Pruned` and `WhyPruned` move from `kept_reference_pass` into `kept_reference_log`.

- Loop of 2,
  `landing`:
  `landing/owed_index_repair.rs`,
  `landing/owed_index_update.rs`.
  `landing/owed_index_repair.rs` uses `landing/owed_index_update.rs` (`Owed`,
  `records_and_unreadable`).
  `landing/owed_index_update.rs` uses `landing/owed_index_repair.rs` (`exclude_what_a_landed_commit_left`,
  `exclude_with_note`,
  `excluded_notice`,
  `unreadable_notice`).
  Break:
  `exclude_what_a_landed_commit_left`,
  `exclude_with_note`,
  `excluded_notice` and `unreadable_notice` move from `owed_index_repair` into `owed_index_update`.

- Loop of 2,
  `landing`:
  `landing/push_record_of_a_moved_branch.rs`,
  `landing/service_handoff.rs`.
  `landing/push_record_of_a_moved_branch.rs` uses `landing/service_handoff.rs` (`ServiceHandoff`,
  `unqueued_branch_note`).
  `landing/service_handoff.rs` uses `landing/push_record_of_a_moved_branch.rs` (`hand_over_push`,
  `write_push_record`).
  Break:
  `hand_over_push` and `write_push_record` move from `service_handoff` into `push_record_of_a_moved_branch`,
  which already holds the record they write.

- Loop of 2,
  `landing`:
  `landing/service_amend_replayed.rs`,
  `landing/service_commit_requests_and_landing_report.rs`.
  `landing/service_amend_replayed.rs` uses `landing/service_commit_requests_and_landing_report.rs`
  (`request_for_git_built`).
  `landing/service_commit_requests_and_landing_report.rs` uses `landing/service_amend_replayed.rs` (`AmendedCommit`).
  Break:
  `request_for_git_built` moves from `service_commit_requests_and_landing_report` into `service_amend_replayed`.

- Loop of 2,
  `landing`:
  `landing/service_index_identity.rs`,
  `landing/service_index_marker.rs`.
  `landing/service_index_identity.rs` uses `landing/service_index_marker.rs` (`marker_of`).
  `landing/service_index_marker.rs` uses `landing/service_index_identity.rs` (`BOOT_SEPARATOR`,
  `HolderState`,
  `ReaderView`,
  `check_procfs_view`,
  `current_pid_scope`,
  `holder_is_elsewhere`,
  `holder_is_gone`,
  `holder_is_running`,
  `holder_state`,
  `holder_state_in`,
  `identity_in_view`,
  `identity_in_view_on`,
  `mark_names_a_gone_holder`,
  `own_identity`,
  `pid_of_self_stat`).
  Break:
  `marker_of` moves from `service_index_marker` into `service_index_identity`.

- Loop of 2,
  `landing`:
  `landing/service_land_side_reference_write.rs`,
  `landing/service_land_without_its_scratch_reference.rs`.
  `landing/service_land_side_reference_write.rs` uses `landing/service_land_without_its_scratch_reference.rs`
  (`the_words_for_a_commit_no_reference_names`).
  `landing/service_land_without_its_scratch_reference.rs` uses `landing/service_land_side_reference_write.rs`
  (`keep_the_batches_commit`,
  `the_batch_reference_names`).
  Break:
  `the_words_for_a_commit_no_reference_names` moves from `service_land_without_its_scratch_reference` into
  `service_land_side_reference_write`.

- Loop of 2,
  `landing`:
  `landing/service_replay.rs`,
  `landing/service_replay_capture_order.rs`.
  `landing/service_replay.rs` uses `landing/service_replay_capture_order.rs` (`capture_order_of_each`).
  `landing/service_replay_capture_order.rs` uses `landing/service_replay.rs` (`Captured`,
  `MergeSite`).
  Break:
  `Captured` and `MergeSite` move from `service_replay` into `service_replay_capture_order`.

- Loop of 2,
  `lock`:
  `lock/index_lock_evidence.rs`,
  `lock/index_lock_evidence_words.rs`.
  `lock/index_lock_evidence.rs` uses `lock/index_lock_evidence_words.rs` (`DEAD_OWNER_LOCK_LEFT`,
  `THE_ENDING_SAYS_IT`,
  `WaitEnd`,
  `blocker_words`,
  `ceiling_line`,
  `dead_owner_diagnostic`,
  `proven_holder_line`,
  `proven_holder_progress_line`,
  `unproven_removal_advice`,
  `unproven_wait_line`).
  `lock/index_lock_evidence_words.rs` uses `lock/index_lock_evidence.rs` (`Evidence`,
  `PidFile`,
  `Verdict`,
  `classify_index_lock`).
  Break:
  `Evidence`,
  `PidFile`,
  `Verdict` and `classify_index_lock` move from `index_lock_evidence` into `index_lock_evidence_words`.

- Loop of 2,
  `lock`:
  `lock/index_lock_holder_parsers.rs`,
  `lock/index_lock_holders.rs`.
  `lock/index_lock_holder_parsers.rs` uses `lock/index_lock_holders.rs` (`Holder`).
  `lock/index_lock_holders.rs` uses `lock/index_lock_holder_parsers.rs` (`LsofProcess`,
  `matching_holders`,
  `parse_lsof_fields`,
  `parse_lsof_number`,
  `parse_restart_manager_output`).
  Break:
  `Holder` moves from `index_lock_holders` into `index_lock_holder_parsers`.

- Loop of 2,
  `merge`:
  `merge/merge_isolation_carry.rs`,
  `merge/merge_isolation_worktree_entry.rs`.
  `merge/merge_isolation_carry.rs` uses `merge/merge_isolation_worktree_entry.rs` (`WorktreeEntry`,
  `not_one_edit_a_merge_can_carry`,
  `worktree_entry_at`).
  `merge/merge_isolation_worktree_entry.rs` uses `merge/merge_isolation_carry.rs` (`could_not_be_read`,
  `mode_of`,
  `the_permissions`).
  Break:
  `could_not_be_read`,
  `mode_of` and `the_permissions` move from `merge_isolation_worktree_entry` into `merge_isolation_carry`.

- Loop of 2,
  `merge`:
  `merge/merge_second_pass_worktree.rs`,
  `merge/merge_second_pass_worktree_unchanged.rs`.
  `merge/merge_second_pass_worktree.rs` uses `merge/merge_second_pass_worktree_unchanged.rs` (`Before`,
  `Held`,
  `Kept`,
  `held_paths`,
  `still_the_same`).
  `merge/merge_second_pass_worktree_unchanged.rs` uses `merge/merge_second_pass_worktree.rs` (`conflicts_of_index`).
  Break:
  `conflicts_of_index` moves from `merge_second_pass_worktree` into `merge_second_pass_worktree_unchanged`.

- Loop of 2,
  `merge`:
  `merge/word_merge.rs`,
  `merge/word_merge_invented_text.rs`.
  `merge/word_merge.rs` uses `merge/word_merge_invented_text.rs` (`refuses_word_merge`).
  `merge/word_merge_invented_text.rs` uses `merge/word_merge.rs` (`LINE_END`,
  `Level`,
  `is_line_end`,
  `token_ranges`).
  Break:
  `LINE_END`,
  `Level`,
  `is_line_end` and `token_ranges` move from `word_merge` into `word_merge_invented_text`.

- Loop of 2,
  `policy`:
  `policy/direct_fix_install_attributes.rs`,
  `policy/direct_fix_install_attributes_reading.rs`.
  `policy/direct_fix_install_attributes.rs` uses `policy/direct_fix_install_attributes_reading.rs` (the child module).
  `policy/direct_fix_install_attributes_reading.rs` uses `policy/direct_fix_install_attributes.rs` (the child module).
  Break:
  `direct_fix_install_attributes_reading` is declared in `mod.rs` instead of by `#[path]`,
  and
  `ATTRIBUTE_ATTEMPTS` moves into it.

- Loop of 2,
  `process`:
  `process/open_file_of_a_descriptor.rs`,
  `process/open_file_of_a_descriptor_in_a_child.rs`.
  `process/open_file_of_a_descriptor.rs` uses `process/open_file_of_a_descriptor_in_a_child.rs`
  (`opened_file_of_in_a_child`).
  `process/open_file_of_a_descriptor_in_a_child.rs` uses `process/open_file_of_a_descriptor.rs` (`OpenedFile`).
  Break:
  `OpenedFile` moves from `open_file_of_a_descriptor` into `open_file_of_a_descriptor_in_a_child`.

- Loop of 2,
  `push`:
  `push/blocked_commit_lines.rs`,
  `push/push_held_back_by_a_blocked_commit.rs`.
  `push/blocked_commit_lines.rs` uses `push/push_held_back_by_a_blocked_commit.rs` (`Held`,
  `Stopped`).
  `push/push_held_back_by_a_blocked_commit.rs` uses `push/blocked_commit_lines.rs` (`UNDECIDED_REASON`,
  `named_reason`).
  Break:
  `UNDECIDED_REASON` and `named_reason` move from `blocked_commit_lines` into `push_held_back_by_a_blocked_commit`.

- Loop of 2,
  `push`:
  `push/push_stacking.rs`,
  `push/stacked_merge_push.rs`.
  `push/push_stacking.rs` uses `push/stacked_merge_push.rs` (`gated_merge_push`).
  `push/stacked_merge_push.rs` uses `push/push_stacking.rs` (`STACKED_PUSH_MARKER`,
  `run_stacked_push`).
  Break:
  the wiring that hands
  `gated_merge_push` to the stack moves from `push_stacking` to its caller,
  which imports both files,
  so
  `push_stacking` stops naming `stacked_merge_push`.

- Loop of 2,
  `service`:
  `service/service_files_of_a_stopped_instance.rs`,
  `service/service_instance.rs`.
  `service/service_files_of_a_stopped_instance.rs` uses `service/service_instance.rs` (`service_unit_instance`,
  `socket_path`).
  `service/service_instance.rs` uses `service/service_files_of_a_stopped_instance.rs` (`NOTES_FOLDER`).
  Break:
  `NOTES_FOLDER` moves from `service_files_of_a_stopped_instance` into `service_instance`.

- Loop of 2,
  `service`:
  `service/service_server.rs`,
  `service/service_server_own_worktree_question.rs`.
  `service/service_server.rs` uses `service/service_server_own_worktree_question.rs` (`own_worktree_gone_now`).
  `service/service_server_own_worktree_question.rs` uses `service/service_server.rs` (`ServeSettings`).
  Break:
  `ServeSettings` moves from `service_server` into `service_server_own_worktree_question`.

- Loop of 2,
  `start_probe`:
  `start_probe/probe_configuration.rs`,
  `start_probe/probe_configuration_refusal.rs`.
  `start_probe/probe_configuration.rs` uses `start_probe/probe_configuration_refusal.rs` (`CheckoutRefusal`,
  `SourceRefusal`).
  `start_probe/probe_configuration_refusal.rs` uses `start_probe/probe_configuration.rs` (`CHECKOUT_KEY`,
  `DATASET_KEY`,
  `START_PROBE_KEY`).
  Break:
  `CHECKOUT_KEY`,
  `DATASET_KEY` and `START_PROBE_KEY` move from `probe_configuration` into `probe_configuration_refusal`.

- Loop of 2,
  `transaction`:
  `transaction/transaction_git.rs`,
  `transaction/transaction_git_step_about_to_start.rs`.
  `transaction/transaction_git.rs` uses `transaction/transaction_git_step_about_to_start.rs` (`StepAboutToStart`,
  `answer_of_a_start`,
  `start_failure`,
  `step_about_to_start`,
  `step_finished`,
  `step_is_run_again`).
  `transaction/transaction_git_step_about_to_start.rs` uses `transaction/transaction_git.rs` (`GitContext`,
  `GitOutput`,
  `GitRequest`,
  `build_command`,
  `input_file`,
  `refused_by_the_system`,
  `step_subject`,
  `command_text`).
  Break:
  `GitContext`,
  `GitOutput`,
  `GitRequest`,
  `build_command`,
  `input_file`,
  `refused_by_the_system`,
  `step_subject` and `command_text` move from `transaction_git` into `transaction_git_step_about_to_start`.

- Loop of 2,
  `worktree`:
  `worktree/worktree_convert_phases.rs`,
  `worktree/worktree_convert_unrecorded_stage.rs`.
  `worktree/worktree_convert_phases.rs` uses `worktree/worktree_convert_unrecorded_stage.rs` (`owns_unrecorded_stage`).
  `worktree/worktree_convert_unrecorded_stage.rs` uses `worktree/worktree_convert_phases.rs` (`stage_identity_of`).
  Break:
  `stage_identity_of` moves from `worktree_convert_phases` into `worktree_convert_unrecorded_stage`.

- Loop of 2,
  `worktree`:
  `worktree/worktree_ladder_finish.rs`,
  `worktree/worktree_ladder_finish_clones.rs`.
  `worktree/worktree_ladder_finish.rs` uses `worktree/worktree_ladder_finish_clones.rs` (the child module).
  `worktree/worktree_ladder_finish_clones.rs` uses `worktree/worktree_ladder_finish.rs` (the child module).
  Break:
  `worktree_ladder_finish_clones` is declared in `mod.rs` instead of by `#[path]`,
  and
  `checkout_like_git_with` moves into it.

- Loop of 2,
  `worktree`:
  `worktree/worktree_removal_rescue.rs`,
  `worktree/worktree_removal_rescue_removal_state.rs`.
  `worktree/worktree_removal_rescue.rs` uses `worktree/worktree_removal_rescue_removal_state.rs` (the child module).
  `worktree/worktree_removal_rescue_removal_state.rs` uses `worktree/worktree_removal_rescue.rs` (the child module).
  Break:
  `worktree_removal_rescue_removal_state` is declared in `mod.rs` instead of by `#[path]`,
  and the items the two still share move into it.

- Loop of 2,
  `worktree_clone_binding_record`:
  `worktree_clone_binding_record/registration_recipe.rs`,
  `worktree_clone_binding_record/registration_removals.rs`.
  `worktree_clone_binding_record/registration_recipe.rs` uses `worktree_clone_binding_record/registration_removals.rs`
  (`done_if_missing`,
  `remove_the_other_entries`,
  `remove_the_ref_store`,
  `remove_the_slot_pointer`).
  `worktree_clone_binding_record/registration_removals.rs` uses `worktree_clone_binding_record/registration_recipe.rs`
  (`RecipeRefusal`,
  `ToClear`).
  Break:
  `RecipeRefusal` and `ToClear` move from `registration_recipe` into `registration_removals`.

- Loop of 2,
  `worktree_clone_mount_handler`:
  `worktree_clone_mount_handler/sweep_configuration.rs`,
  `worktree_clone_mount_handler/sweep_configuration_refusal.rs`.
  `worktree_clone_mount_handler/sweep_configuration.rs` uses
  `worktree_clone_mount_handler/sweep_configuration_refusal.rs` (`ConfigurationRefusal`).
  `worktree_clone_mount_handler/sweep_configuration_refusal.rs` uses
  `worktree_clone_mount_handler/sweep_configuration.rs` (`CHECKOUT_DATASET_KEY`,
  `CHECKOUT_KEY`,
  `CLONE_MOUNTS_KEY`,
  `PARENT_KEY`,
  `PROBE_OBJECT_KEY`).
  Break:
  `CHECKOUT_DATASET_KEY`,
  `CHECKOUT_KEY`,
  `CLONE_MOUNTS_KEY`,
  `PARENT_KEY` and `PROBE_OBJECT_KEY` move from `sweep_configuration` into `sweep_configuration_refusal`.

- Loop of 2,
  `worktree_clone_unmount`:
  `worktree_clone_unmount/unmount_record.rs`,
  `worktree_clone_unmount/unmount_turn.rs`.
  `worktree_clone_unmount/unmount_record.rs` uses `worktree_clone_unmount/unmount_turn.rs` (`UnmountOutcome`,
  `UnmountRequest`,
  `call_refused`,
  `refused`).
  `worktree_clone_unmount/unmount_turn.rs` uses `worktree_clone_unmount/unmount_record.rs` (`check_the_folders_shape`,
  `record_the_bind`).
  Break:
  `check_the_folders_shape` and `record_the_bind` move from `unmount_turn` into `unmount_record`.

- Loop of 2,
  `worktree_copying`:
  `worktree_copying/worktree_copy_forward.rs`,
  `worktree_copying/worktree_removal_of_a_bound_clone.rs`.
  `worktree_copying/worktree_copy_forward.rs` uses `worktree_copying/worktree_removal_of_a_bound_clone.rs`
  (`refuse_the_removal_of_a_bound_clone`).
  `worktree_copying/worktree_removal_of_a_bound_clone.rs` uses `worktree_copying/worktree_copy_forward.rs` (`CopyFacts`,
  `ForwardedRun`).
  Break:
  `CopyFacts` moves into the planned `worktree_command_facts` file and `ForwardedRun` into the file that describes a
  forwarded Git run,
  so the removal refuses through the facts file rather than through the forward.

- Loop of 2,
  `worktree_copying`:
  `worktree_copying/worktree_copy_prune.rs`,
  `worktree_copying/worktree_copy_selection.rs`.
  `worktree_copying/worktree_copy_prune.rs` uses `worktree_copying/worktree_copy_selection.rs` (`STAGE_PREFIX`,
  `is_separator_byte`).
  `worktree_copying/worktree_copy_selection.rs` uses `worktree_copying/worktree_copy_prune.rs` (`Prune`,
  `names_a_private_stage`).
  Break:
  `Prune` and `names_a_private_stage` move from `worktree_copy_selection` into `worktree_copy_prune`.

- Loop of 2,
  `worktree_removal_deferred_origin_check`:
  `worktree_removal_deferred_origin_check/origin_refusal.rs`,
  `worktree_removal_deferred_origin_check/planned_origin.rs`.
  `worktree_removal_deferred_origin_check/origin_refusal.rs` uses
  `worktree_removal_deferred_origin_check/planned_origin.rs` (`PlannedOrigin`).
  `worktree_removal_deferred_origin_check/planned_origin.rs` uses
  `worktree_removal_deferred_origin_check/origin_refusal.rs` (`OriginRefusal`).
  Break:
  `OriginRefusal` moves from `origin_refusal` into `planned_origin`.

- Loop of 2,
  `zfs_mounted_dataset`:
  `zfs_mounted_dataset/mounted_dataset_refusal.rs`,
  `zfs_mounted_dataset/the_designs_datasets_in_the_mount_table.rs`.
  `zfs_mounted_dataset/mounted_dataset_refusal.rs` uses `zfs_mounted_dataset/the_designs_datasets_in_the_mount_table.rs`
  (`DESIGN_CHECKOUT_DATASET_NAME`,
  `DESIGN_CONTAINER_DATASET_NAME`,
  `DESIGN_PROBE_DATASET_UNDER_THE_POOL`).
  `zfs_mounted_dataset/the_designs_datasets_in_the_mount_table.rs` uses `zfs_mounted_dataset/mounted_dataset_refusal.rs`
  (`MountedDatasetRefusal`).
  Break:
  `MountedDatasetRefusal` moves from `mounted_dataset_refusal` into `the_designs_datasets_in_the_mount_table`.

- Loop of 3,
  `conflict`:
  `conflict/conflict_sides.rs`,
  `conflict/discarded_prune.rs`,
  `conflict/discarded_report.rs`.
  `conflict/conflict_sides.rs` uses `conflict/discarded_prune.rs` (`Limits`,
  `Problem`,
  `directory_name_of`,
  `prune`).
  `conflict/discarded_prune.rs` uses `conflict/conflict_sides.rs` (`SAVED_DIRECTORY`);
  `conflict/discarded_report.rs` (`Report`,
  `doctor_section`,
  `report`).
  `conflict/discarded_report.rs` uses `conflict/conflict_sides.rs` (`SAVED_DIRECTORY`);
  `conflict/discarded_prune.rs` (`Limits`,
  `list_stamped_below`).
  Break:
  a new file
  `where_discarded_copies_live` receives `SAVED_DIRECTORY`,
  `Limits`,
  `Problem` and `list_stamped_below`,
  so the prune pass and the report use it instead of each other.

- Loop of 3,
  `foundation`:
  `foundation/diagnostic_lines_off_the_calling_thread.rs`,
  `foundation/diagnostic_log.rs`,
  `foundation/lines_written_by_a_thread.rs`.
  `foundation/diagnostic_lines_off_the_calling_thread.rs` uses `foundation/lines_written_by_a_thread.rs` (`QueuePolicy`,
  `SharedQueue`,
  `hand`,
  `start_the_queue`,
  `wait_until_written`,
  `write_what_waits`,
  `as`,
  `queue_write_what_waits`,
  `dropped_so_far`,
  `queue_dropped_so_far`,
  `give_the_thread`,
  `queue_give_the_thread`,
  `queued`,
  `queue_queued`,
  `the_reader_stopped`,
  `queue_the_reader_stopped`).
  `foundation/diagnostic_log.rs` uses `foundation/diagnostic_lines_off_the_calling_thread.rs` (`write`).
  `foundation/lines_written_by_a_thread.rs` uses `foundation/diagnostic_log.rs` (`debug`).
  Break:
  the one
  `debug` line `lines_written_by_a_thread` prints moves into `diagnostic_lines_off_the_calling_thread`,
  which already holds that file's other queue operations.

- Loop of 3,
  `landing`:
  `landing/held_commit_settle.rs`,
  `landing/held_commit_settle_move.rs`,
  `landing/service_land_finish.rs`.
  `landing/held_commit_settle.rs` uses `landing/held_commit_settle_move.rs` (`move_the_branch`);
  `landing/service_land_finish.rs` (`follow_up`,
  `settle`,
  `Settled`,
  `RecoveryReport`,
  `Handled`).
  `landing/held_commit_settle_move.rs` uses `landing/held_commit_settle.rs` (`kept_reference`).
  `landing/service_land_finish.rs` uses `landing/held_commit_settle.rs` (`record_of`,
  `settle_a_recorded_hold`,
  `say_settled_batch_record_cannot_be_read`).
  Break:
  `kept_reference` moves from `held_commit_settle` into `held_commit_settle_move`,
  and
  `record_of`,
  `settle_a_recorded_hold` and `say_settled_batch_record_cannot_be_read` move into `service_land_finish`.

- Loop of 3,
  `landing`:
  `landing/service_commit.rs`,
  `landing/service_commit_after_a_failure.rs`,
  `landing/service_landed_aftermath.rs`.
  `landing/service_commit.rs` uses `landing/service_commit_after_a_failure.rs` (`FailedAttempt`,
  `answer_after_a_failure`);
  `landing/service_landed_aftermath.rs` (`LandedAftermath`,
  `the_landed_commit`).
  `landing/service_commit_after_a_failure.rs` uses `landing/service_commit.rs` (`ServiceCommit`);
  `landing/service_landed_aftermath.rs` (`LandedAftermath`,
  `after_a_silent_or_failed_service`).
  `landing/service_landed_aftermath.rs` uses `landing/service_commit.rs` (`ServiceCommit`).
  Break:
  `ServiceCommit` moves from `service_commit` into `service_landed_aftermath`.

- Loop of 3,
  `lfs`:
  `lfs/lfs_upload_plan.rs`,
  `lfs/lfs_upload_plan_arguments.rs`,
  `lfs/lfs_upload_plan_awaiting.rs`.
  `lfs/lfs_upload_plan.rs` uses `lfs/lfs_upload_plan_arguments.rs` (`Destinations`,
  `a_bare_push`,
  `a_branch_is_needed`,
  `destinations_of`,
  `remote_name_of`,
  `same_place`,
  `specs_of`,
  `submodules_of`,
  `the_plan`,
  `unique_refs`,
  `uploads_nothing`);
  `lfs/lfs_upload_plan_awaiting.rs` (`plan_upload_awaiting`).
  `lfs/lfs_upload_plan_arguments.rs` uses `lfs/lfs_upload_plan.rs` (`UploadPlan`,
  `configured_remotes`,
  `has_push_rewrite`,
  `lfs_destination`).
  `lfs/lfs_upload_plan_awaiting.rs` uses `lfs/lfs_upload_plan.rs` (`Source`,
  `UploadPlan`,
  `directory_of`,
  `lfs_server_of`,
  `names_of`,
  `refspec_source`,
  `star_split`);
  `lfs/lfs_upload_plan_arguments.rs` (`Destinations`,
  `a_bare_push`,
  `a_branch_is_needed`,
  `destinations_of`,
  `remote_name_of`,
  `same_place`,
  `specs_of`,
  `submodules_of`,
  `the_plan`,
  `unique_refs`,
  `uploads_nothing`).
  Break:
  a new file
  `the_upload_plan_itself` receives `UploadPlan`,
  `Source`,
  `Destinations` and the pure plan readers,
  so the arguments,
  the awaiting form and the plan all use it.

- Loop of 3,
  `merge`:
  `merge/merge_second_pass.rs`,
  `merge/merge_second_pass_git_lines.rs`,
  `merge/merge_second_pass_report.rs`.
  `merge/merge_second_pass.rs` uses `merge/merge_second_pass_git_lines.rs` (`merge_with_git_lines`);
  `merge/merge_second_pass_report.rs` (`BLOBS_FIELD`,
  `EXCERPT_CHARACTERS`,
  `FILE_FIELD`,
  `INPUTS_HINT`,
  `OPERATION_FIELD`,
  `PARTS_FIELD`,
  `REPORT_MARK`,
  `journal_entry`,
  `journal_merged`,
  `part_text`,
  `report_lines`).
  `merge/merge_second_pass_git_lines.rs` uses `merge/merge_second_pass.rs` (`Conflict`,
  `GitReply`,
  `GitRunner`).
  `merge/merge_second_pass_report.rs` uses `merge/merge_second_pass.rs` (`Merged`).
  Break:
  `Conflict`,
  `GitReply` and `GitRunner` move into `merge_second_pass_git_lines`,
  and
  `Merged` moves into `merge_second_pass_report`.

- Loop of 3,
  `policy`:
  `policy/direct_fix_install.rs`,
  `policy/direct_fix_install_judging.rs`,
  `policy/direct_fix_install_messages.rs`.
  `policy/direct_fix_install.rs` uses `policy/direct_fix_install_judging.rs` (`SiteState`,
  `displaced_is_unchanged`,
  `holds_exactly`,
  `state_of`);
  `policy/direct_fix_install_messages.rs` (`could_not_remove`,
  `install_failure`,
  `journal_refusal`,
  `kept_bytes_notice`,
  `kept_notice`,
  `unchanged_failure`).
  `policy/direct_fix_install_judging.rs` uses `policy/direct_fix_install.rs` (`Prepared`).
  `policy/direct_fix_install_messages.rs` uses `policy/direct_fix_install.rs` (`Prepared`).
  Break:
  a new file
  `what_a_direct_fix_prepares` receives `Prepared`,
  which the install,
  the judging and the messages all name.

- Loop of 3,
  `process`:
  `process/signal_relay.rs`,
  `process/signal_relay_descriptor.rs`,
  `process/signal_report.rs`.
  `process/signal_relay.rs` uses `process/signal_relay_descriptor.rs` (`ensure_relay_descriptor`,
  `pidfd_of`,
  `point_relay_at`);
  `process/signal_report.rs` (`read_report`).
  `process/signal_relay_descriptor.rs` uses `process/signal_relay.rs` (`RELAY_FD`).
  `process/signal_report.rs` uses `process/signal_relay.rs` (`is_process_sender`).
  Break:
  `RELAY_FD` moves into `signal_relay_descriptor`,
  and
  `is_process_sender` moves into `signal_report`.

- Loop of 3,
  `push`:
  `push/manual_push_probe.rs`,
  `push/manual_push_range.rs`,
  `push/manual_push_range_published_tips.rs`.
  `push/manual_push_probe.rs` uses `push/manual_push_range_published_tips.rs` (`oids_by_ref`).
  `push/manual_push_range.rs` uses `push/manual_push_probe.rs` (`PushUpdate`);
  `push/manual_push_range_published_tips.rs` (`peel_commits`).
  `push/manual_push_range_published_tips.rs` uses `push/manual_push_probe.rs` (`PushUpdate`);
  `push/manual_push_range.rs` (`ManualPushRangeError`,
  `lines_of`,
  `range_error`,
  `run_git_bytes`).
  Break:
  a new file
  `what_a_push_would_update` receives `PushUpdate`,
  `ManualPushRangeError`,
  `range_error`,
  `lines_of` and `run_git_bytes`,
  so the three files use it instead of each other.

- Loop of 3,
  `service`:
  `service/service_protocol.rs`,
  `service/service_protocol_decode_lines_to_messages.rs`,
  `service/service_protocol_forgetting_programs.rs`.
  `service/service_protocol.rs` uses `service/service_protocol_decode_lines_to_messages.rs` (`decode_client`,
  `decode_service`).
  `service/service_protocol_decode_lines_to_messages.rs` uses `service/service_protocol_forgetting_programs.rs`
  (`forgetting_programs`).
  `service/service_protocol_forgetting_programs.rs` uses `service/service_protocol.rs` (`ClientMessage`).
  Break:
  `service_protocol_forgetting_programs` folds into `service_protocol_decode_lines_to_messages`.

- Loop of 3,
  `worktree_clone_mount_handler`:
  `worktree_clone_mount_handler/sweep_one_clone.rs`,
  `worktree_clone_mount_handler/sweep_one_clone_answers.rs`,
  `worktree_clone_mount_handler/sweep_unmount_answers.rs`.
  `worktree_clone_mount_handler/sweep_one_clone.rs` uses `worktree_clone_mount_handler/sweep_one_clone_answers.rs`
  (`answer_the_attach`,
  `answer_the_standing_mount`,
  `answer_unmount`,
  `record_the_first_attach`,
  `refused_by_the_checks`,
  `the_first_attach_outcome`,
  `the_first_attach_refusal`).
  `worktree_clone_mount_handler/sweep_one_clone_answers.rs` uses `worktree_clone_mount_handler/sweep_one_clone.rs`
  (`CloneOutcome`,
  `RECEIPT_STEP`,
  `SweepContext`);
  `worktree_clone_mount_handler/sweep_unmount_answers.rs` (`refuse_an_unmount`,
  `answer_the_unmount`,
  `HandlerDatasetGuids`).
  `worktree_clone_mount_handler/sweep_unmount_answers.rs` uses `worktree_clone_mount_handler/sweep_one_clone.rs`
  (`CloneOutcome`,
  `RECEIPT_STEP`,
  `SweepContext`);
  `worktree_clone_mount_handler/sweep_one_clone_answers.rs` (`receipt_of`).
  Break:
  a new file
  `the_sweep_context_and_what_it_outcomes` receives `CloneOutcome`,
  `RECEIPT_STEP` and `SweepContext`,
  which the sweep and both answer files name.

- Loop of 3,
  `worktree_clone_removal_proof_and_plan`:
  `worktree_clone_removal_proof_and_plan/removal_completion.rs`,
  `worktree_clone_removal_proof_and_plan/removal_origin_step.rs`,
  `worktree_clone_removal_proof_and_plan/removal_settle_step.rs`.
  `worktree_clone_removal_proof_and_plan/removal_completion.rs` uses
  `worktree_clone_removal_proof_and_plan/removal_origin_step.rs` (`attempt_the_origin_cleanup`,
  `publish_the_origin_record`);
  `worktree_clone_removal_proof_and_plan/removal_settle_step.rs` (`settle_the_registration`).
  `worktree_clone_removal_proof_and_plan/removal_origin_step.rs` uses
  `worktree_clone_removal_proof_and_plan/removal_completion.rs` (`RemovalCompletion`).
  `worktree_clone_removal_proof_and_plan/removal_settle_step.rs` uses
  `worktree_clone_removal_proof_and_plan/removal_completion.rs` (`RemovalCompletion`).
  Break:
  a new file
  `what_a_removal_completes` receives `RemovalCompletion`,
  which the completion,
  the origin step and the settle step all name.

- Loop of 4,
  `landing`:
  `landing/work_item.rs`,
  `landing/work_item_enqueue.rs`,
  `landing/work_item_holding.rs`,
  `landing/work_item_queued_ids.rs`.
  `landing/work_item.rs` uses `landing/work_item_enqueue.rs` (`Enqueued`,
  `enqueue`,
  `enqueue_all`);
  `landing/work_item_holding.rs` (`block`,
  `forget_held`,
  `hold`,
  `promote`,
  `record_pass`,
  `record_refusal`,
  `record_verdict`,
  `requeue_after_giving_up`);
  `landing/work_item_queued_ids.rs` (`queued_ids`).
  `landing/work_item_enqueue.rs` uses `landing/work_item.rs` (`CLAIMED`,
  `DONE`,
  `FAILED`,
  `PENDING`,
  `RECOVERING`,
  `WorkItem`,
  `entries_oldest_first`,
  `id_of_entry`,
  `state_directory`).
  `landing/work_item_holding.rs` uses `landing/work_item.rs` (`CLAIMED`,
  `DONE`,
  `FAILED`,
  `HELD`,
  `PENDING`,
  `RECOVERING`,
  `WorkItem`,
  `WorkKind`,
  `state_directory`,
  `find_claim`).
  `landing/work_item_queued_ids.rs` uses `landing/work_item.rs` (`CLAIMED`,
  `HELD`,
  `PENDING`,
  `RECOVERING`,
  `entries_oldest_first`,
  `id_of_entry`).
  Break:
  a new file
  `the_record_of_one_work_item` receives `WorkItem`,
  `WorkKind` and the six state constants,
  so the queue,
  the holding pass and the id listing use it.

- Loop of 4,
  `policy`:
  `policy/policy_branch_worktree.rs`,
  `policy/policy_checks.rs`,
  `policy/policy_linked_worktree.rs`,
  `policy/policy_linked_worktree_nested_repositories.rs`.
  `policy/policy_branch_worktree.rs` uses `policy/policy_checks.rs` (`clean`,
  `fact_unavailable`,
  `rejected`).
  `policy/policy_checks.rs` uses `policy/policy_branch_worktree.rs` (`check_branch_worktree`);
  `policy/policy_linked_worktree.rs` (`check_linked_worktree`).
  `policy/policy_linked_worktree.rs` uses `policy/policy_checks.rs` (`clean`,
  `fact_unavailable`,
  `rejected`);
  `policy/policy_linked_worktree_nested_repositories.rs` (`AllowedForm`,
  `allow_a_discarding_form`).
  `policy/policy_linked_worktree_nested_repositories.rs` uses `policy/policy_checks.rs` (`clean`,
  `rejected`).
  Break:
  a new file
  `the_answers_a_policy_gives` receives `clean`,
  `fact_unavailable` and `rejected`,
  which the three policies share.

- Loop of 4,
  `repair`:
  `repair/package_repair_cargo.rs`,
  `repair/package_repair_cargo_jobs.rs`,
  `repair/package_repair_prewarm_job.rs`,
  `repair/package_repair_prewarm_job_filling.rs`.
  `repair/package_repair_cargo.rs` uses `repair/package_repair_cargo_jobs.rs` (the child module);
  `repair/package_repair_prewarm_job.rs` (`stage_the_worktrees_own_state`,
  `Staged`).
  `repair/package_repair_cargo_jobs.rs` uses `repair/package_repair_cargo.rs` (the child module).
  `repair/package_repair_prewarm_job.rs` uses `repair/package_repair_prewarm_job_filling.rs` (the child module).
  `repair/package_repair_prewarm_job_filling.rs` uses `repair/package_repair_cargo.rs` (`locked_clean_arguments`,
  `manual_command`).
  Break:
  the three
  `#[path]` children are declared in `mod.rs`,
  and
  `locked_clean_arguments` and `manual_command` move into a new file `the_commands_a_repair_prints`.

- Loop of 4,
  `repair`:
  `repair/package_repair_relink_notice.rs`,
  `repair/package_repair_toolchain.rs`,
  `repair/package_repair_toolchain_copy_removal.rs`,
  `repair/package_repair_toolchain_install.rs`.
  `repair/package_repair_relink_notice.rs` uses `repair/package_repair_toolchain.rs` (`MANUAL_COMMAND`).
  `repair/package_repair_toolchain.rs` uses `repair/package_repair_toolchain_install.rs` (the child module).
  `repair/package_repair_toolchain_copy_removal.rs` uses `repair/package_repair_relink_notice.rs`
  (`git_dir_of_a_worktree`,
  `record_notice`).
  `repair/package_repair_toolchain_install.rs` uses `repair/package_repair_toolchain_copy_removal.rs` (the child
  module).
  Break:
  the
  `#[path]` children are declared in `mod.rs`,
  and
  `MANUAL_COMMAND` moves into a new file `the_commands_a_repair_prints`,
  which the notice and the toolchain repair both use.

- Loop of 4,
  `shadow`:
  `shadow/shadow_create.rs`,
  `shadow/shadow_drawn.rs`,
  `shadow/shadow_pool.rs`,
  `shadow/shadow_pool_member.rs`.
  `shadow/shadow_create.rs` uses `shadow/shadow_drawn.rs` (`HandedShadow`);
  `shadow/shadow_pool.rs` (`take`);
  `shadow/shadow_pool_member.rs` (`refusal`).
  `shadow/shadow_drawn.rs` uses `shadow/shadow_create.rs` (`ShadowTemplate`);
  `shadow/shadow_pool.rs` (`members`,
  `pool_key`,
  `pool_root`);
  `shadow/shadow_pool_member.rs` (`refusal`).
  `shadow/shadow_pool.rs` uses `shadow/shadow_create.rs` (`ShadowTemplate`,
  `create_shadow_template`);
  `shadow/shadow_pool_member.rs` (`refusal`).
  `shadow/shadow_pool_member.rs` uses `shadow/shadow_create.rs` (`ShadowTemplate`).
  Break:
  a new file
  `the_template_a_shadow_is_drawn_from` receives `ShadowTemplate` and `create_shadow_template`,
  so the pool,
  the drawn shadow and the pool member use it instead of the create file.

- Loop of 5,
  `command`:
  `command/command_options.rs`,
  `command/command_options_long.rs`,
  `command/command_options_long_resolution.rs`,
  `command/command_options_short.rs`,
  `command/command_options_value.rs`.
  `command/command_options.rs` uses `command/command_options_long.rs` (`scan_long_option`);
  `command/command_options_short.rs` (`scan_short_cluster`).
  `command/command_options_long.rs` uses `command/command_options.rs` (`Occurrence`,
  `OptionError`,
  `OptionSpec`,
  `OptionValue`,
  `ParseMode`,
  `ParsedOptions`);
  `command/command_options_long_resolution.rs` (`Candidate`,
  `Resolution`,
  `resolve_long_option`);
  `command/command_options_value.rs` (`Spelling`,
  `take_value`).
  `command/command_options_long_resolution.rs` uses `command/command_options.rs` (`Arity`,
  `OptionErrorKind`,
  `OptionSpec`).
  `command/command_options_short.rs` uses `command/command_options.rs` (`Occurrence`,
  `OptionError`,
  `OptionErrorKind`,
  `OptionSpec`,
  `OptionValue`,
  `ParseMode`,
  `ParsedOptions`);
  `command/command_options_value.rs` (`Spelling`,
  `take_value`).
  `command/command_options_value.rs` uses `command/command_options.rs` (`Arity`,
  `OptionError`,
  `OptionErrorKind`,
  `OptionValue`).
  Break:
  a new file
  `command_options_types` receives `Occurrence`,
  `OptionError`,
  `OptionErrorKind`,
  `OptionSpec`,
  `OptionValue`,
  `ParseMode`,
  `ParsedOptions` and `Arity`,
  as the 2026-11-07 plan already said.

- Loop of 6,
  `lock`:
  `lock/stale_ref_lock_found.rs`,
  `lock/stale_ref_lock_judge.rs`,
  `lock/stale_ref_lock_retire.rs`,
  `lock/stale_ref_lock_walk.rs`,
  `lock/stale_ref_locks.rs`,
  `lock/stopped_git_locks.rs`.
  `lock/stale_ref_lock_found.rs` uses `lock/stale_ref_lock_judge.rs` (`Verdict`).
  `lock/stale_ref_lock_judge.rs` uses `lock/stale_ref_lock_found.rs` (`PidFile`);
  `lock/stale_ref_lock_walk.rs` (`pid_file_beside`);
  `lock/stopped_git_locks.rs` (`pid_named_by`).
  `lock/stale_ref_lock_retire.rs` uses `lock/stale_ref_lock_found.rs` (`Found`,
  `Outcome`);
  `lock/stale_ref_lock_judge.rs` (`Examined`,
  `Identity`,
  `Judge`,
  `Verdict`,
  `identity_of`);
  `lock/stale_ref_lock_walk.rs` (`Candidate`,
  `companions_of`).
  `lock/stale_ref_lock_walk.rs` uses `lock/stopped_git_locks.rs` (`LOCK_SUFFIX`,
  `PID_LOCK_SUFFIX`).
  `lock/stale_ref_locks.rs` uses `lock/stale_ref_lock_found.rs` (`Found`,
  `Outcome`,
  `PidFile`);
  `lock/stale_ref_lock_judge.rs` (`Judge`,
  `Proof`,
  `Verdict`);
  `lock/stale_ref_lock_retire.rs` (`judge_and_clear`);
  `lock/stale_ref_lock_walk.rs` (`ref_lock_candidates`,
  `Candidate`).
  `lock/stopped_git_locks.rs` uses `lock/stale_ref_lock_judge.rs` (`Judge`);
  `lock/stale_ref_locks.rs` (`clear_stale_ref_locks`).
  Break:
  a new file
  `stale_ref_lock_names` receives `LOCK_SUFFIX`,
  `PID_LOCK_SUFFIX`,
  `pid_named_by` and `pid_file_beside`,
  and
  `Verdict` moves into `stale_ref_lock_found`,
  as the 2026-11-07 plan already said.

- Loop of 7,
  `landing`:
  `landing/service_round.rs`,
  `landing/service_work.rs`,
  `landing/service_work_drain_pending_claims.rs`,
  `landing/service_work_drain_pushes_of_each_branch.rs`,
  `landing/service_work_drains_a_snapshot_ahead.rs`,
  `landing/service_work_drains_awaiting.rs`,
  `landing/service_work_items_side_by_side.rs`.
  `landing/service_round.rs` uses `landing/service_work.rs` (`CLAIM_LEASE`,
  `Drained`,
  `PUSH_KINDS`,
  `WorkHandlers`,
  `common_directory_of`,
  `drain_common_records_planned_awaiting`,
  `drain_planned_awaiting`);
  `landing/service_work_drains_awaiting.rs` (the child module).
  `landing/service_work.rs` uses `landing/service_work_drains_awaiting.rs` (`drain_common_records_planned_awaiting`,
  `drain_planned_awaiting`,
  `drain_planned_with_awaiting`).
  `landing/service_work_drain_pending_claims.rs` uses `landing/service_work_drain_pushes_of_each_branch.rs`
  (`push_the_branches_awaiting`);
  `landing/service_work_drains_a_snapshot_ahead.rs` (`drain_a_snapshot_ahead`);
  `landing/service_work_items_side_by_side.rs` (`run_and_settle_side_by_side`,
  `run_and_settle_what_the_round_may_start`,
  `slow_items_side_by_side`).
  `landing/service_work_drain_pushes_of_each_branch.rs` uses `landing/service_work_items_side_by_side.rs`
  (`pushes_side_by_side`,
  `waves_of`).
  `landing/service_work_drains_a_snapshot_ahead.rs` uses `landing/service_work.rs` (`Verdict`).
  `landing/service_work_drains_awaiting.rs` uses `landing/service_work_drain_pending_claims.rs`
  (`drain_pending_awaiting`).
  `landing/service_work_items_side_by_side.rs` uses `landing/service_round.rs` (`REPOSITORY_MEMORY`).
  Break:
  `REPOSITORY_MEMORY` moves into a new file `the_memory_one_round_keeps`,
  and the
  `#[path]` child `service_work_drains_awaiting` is declared in `mod.rs`.

- Loop of 7,
  `repo_read`:
  `repo_read/repository_facts.rs`,
  `repo_read/repository_facts_abort_discard.rs`,
  `repo_read/repository_facts_abort_discard_worktree_entries.rs`,
  `repo_read/repository_facts_ignored_files.rs`,
  `repo_read/repository_facts_staged_entries.rs`,
  `repo_read/repository_facts_staged_selection.rs`,
  `repo_read/repository_facts_untracked_files.rs`.
  `repo_read/repository_facts.rs` uses `repo_read/repository_facts_abort_discard.rs` (`abort_discard`);
  `repo_read/repository_facts_ignored_files.rs` (`ignored_files_the_tree_tracks`);
  `repo_read/repository_facts_staged_selection.rs` (`staged_selection`);
  `repo_read/repository_facts_untracked_files.rs` (`untracked_files_the_tree_tracks`).
  `repo_read/repository_facts_abort_discard.rs` uses `repo_read/repository_facts.rs` (`GitFacts`);
  `repo_read/repository_facts_abort_discard_worktree_entries.rs` (`worktree_top`,
  `holds_an_entry`);
  `repo_read/repository_facts_staged_entries.rs` (`staged_entries`).
  `repo_read/repository_facts_abort_discard_worktree_entries.rs` uses `repo_read/repository_facts.rs` (`GitFacts`,
  `RepositoryFacts`).
  `repo_read/repository_facts_ignored_files.rs` uses `repo_read/repository_facts.rs` (`GitFacts`).
  `repo_read/repository_facts_staged_entries.rs` uses `repo_read/repository_facts.rs` (`GitFacts`).
  `repo_read/repository_facts_staged_selection.rs` uses `repo_read/repository_facts.rs` (`GitFacts`,
  `RepositoryFacts`).
  `repo_read/repository_facts_untracked_files.rs` uses `repo_read/repository_facts.rs` (`GitFacts`);
  `repo_read/repository_facts_ignored_files.rs` (`tree_paths`).
  Break:
  a new file
  `the_facts_a_repository_reads` receives `GitFacts` and `RepositoryFacts`,
  which the worktree entries,
  the staged entries and the selection passes all name.

- Loop of 8,
  `landing`:
  `landing/service_messages.rs`,
  `landing/service_messages_of_a_failure.rs`,
  `landing/service_messages_of_a_manager_that_did_not_answer.rs`,
  `landing/service_messages_of_a_platform.rs`,
  `landing/service_messages_of_a_service_that_stopped.rs`,
  `landing/service_messages_of_a_start_that_cannot_come.rs`,
  `landing/service_messages_of_a_start_that_was_asked_and_did_not_come.rs`,
  `landing/service_messages_of_an_interrupted_commit.rs`.
  `landing/service_messages.rs` uses `landing/service_messages_of_a_failure.rs` (`failure_code`);
  `landing/service_messages_of_a_manager_that_did_not_answer.rs` (`did_not_answer`);
  `landing/service_messages_of_a_platform.rs` (`platform_remedy`);
  `landing/service_messages_of_a_service_that_stopped.rs` (`nothing_answered`,
  `silent`,
  `not_reading`,
  `silent_after_preparing`);
  `landing/service_messages_of_a_start_that_cannot_come.rs` (`socket_path_too_long`,
  `service_elsewhere`);
  `landing/service_messages_of_a_start_that_was_asked_and_did_not_come.rs` (`ManagerLastWords`,
  `RequestedStart`,
  `did_not_start`);
  `landing/service_messages_of_an_interrupted_commit.rs` (`interrupted`,
  `not_known`).
  `landing/service_messages_of_a_failure.rs` uses `landing/service_messages.rs` (`ClientFailure`).
  `landing/service_messages_of_a_manager_that_did_not_answer.rs` uses `landing/service_messages.rs` (`NOTHING_CHANGED`,
  `STATUS_COMMAND`).
  `landing/service_messages_of_a_platform.rs` uses `landing/service_messages.rs` (`LINGER_COMMAND`,
  `Platform`).
  `landing/service_messages_of_a_service_that_stopped.rs` uses `landing/service_messages.rs` (`CHECK_COMMANDS`,
  `NOTHING_CHANGED`,
  `NOTHING_CHANGED_AND_THE_STAGED_WORK`,
  `RESTART_ONE_UNIT_COMMAND`,
  `STATUS_COMMAND`,
  `WhatTheBranchShows`).
  `landing/service_messages_of_a_start_that_cannot_come.rs` uses `landing/service_messages.rs` (`NOTHING_CHANGED`).
  `landing/service_messages_of_a_start_that_was_asked_and_did_not_come.rs` uses `landing/service_messages.rs`
  (`ClientFailure`).
  `landing/service_messages_of_an_interrupted_commit.rs` uses `landing/service_messages.rs` (`CHECK_COMMANDS`,
  `LOG_COMMAND`,
  `NOTHING_CHANGED`,
  `RESTART_COMMAND`,
  `it_had_said`,
  `waiting_for`).
  Break:
  a new file
  `the_words_a_service_message_repeats` receives `ClientFailure`,
  `Platform`,
  `WhatTheBranchShows`,
  `NOTHING_CHANGED`,
  `STATUS_COMMAND`,
  `CHECK_COMMANDS`,
  `LOG_COMMAND`,
  `RESTART_COMMAND`,
  `RESTART_ONE_UNIT_COMMAND`,
  `LINGER_COMMAND` and `NOTHING_CHANGED_AND_THE_STAGED_WORK`,
  so the seven message files stop naming the one they are split from.

- Loop of 10,
  `push`:
  `push/auto_push_single_flight.rs`,
  `push/bounded_push.rs`,
  `push/bounded_query.rs`,
  `push/child_watch.rs`,
  `push/child_watch_clock_marks_and_stopping.rs`,
  `push/child_watch_one_shot_run.rs`,
  `push/child_watch_task_loop_and_notice.rs`,
  `push/push_progress.rs`,
  `push/push_server_naming.rs`,
  `push/push_wait_until_done.rs`.
  `push/auto_push_single_flight.rs` uses `push/bounded_push.rs` (`STOP_GRACE`);
  `push/bounded_query.rs` (`QueryOutput`,
  `query_window`,
  `run_bounded_query`,
  `run_bounded_query_awaiting`).
  `push/bounded_push.rs` uses `push/auto_push_single_flight.rs` (`PushAttempt`,
  `SingleFlightError`);
  `push/push_progress.rs` (`WaitReport`);
  `push/push_server_naming.rs` (`ServerNaming`,
  `first_word`);
  `push/push_wait_until_done.rs` (`flooded`,
  `wait_line_words`,
  `MAX_PUSH_OUTPUT`,
  `WAIT_NOTE_PHRASE`,
  `wait_until_done`,
  `wait_until_done_noting`,
  `wait_until_done_up_to`).
  `push/bounded_query.rs` uses `push/auto_push_single_flight.rs` (`SingleFlightError`);
  `push/bounded_push.rs` (`MAX_PUSH_OUTPUT`,
  `StopCause`,
  `flooded`,
  `stop_push`,
  `wait_until_done_up_to`);
  `push/child_watch.rs` (`ended_by_the_stop`,
  `refused_while_stopping`);
  `push/push_progress.rs` (`Clock`);
  `push/push_wait_until_done.rs` (the child module).
  `push/child_watch.rs` uses `push/child_watch_clock_marks_and_stopping.rs` (`Mark`,
  `WatchFlags`,
  `WatchTerms`,
  `grace_for`,
  `RootPin`,
  `pin_at_start`,
  `the_root_is_proven`);
  `push/child_watch_one_shot_run.rs` (`output_watched`,
  `output_watched_stopping`,
  `output_watched_within`,
  `output_watched_within_for_a_stop`,
  `reader_subject`);
  `push/child_watch_task_loop_and_notice.rs` (`watch_loop`).
  `push/child_watch_clock_marks_and_stopping.rs` uses `push/bounded_push.rs` (`Patience`,
  `STOP_GRACE`);
  `push/push_progress.rs` (`Clock`).
  `push/child_watch_one_shot_run.rs` uses `push/child_watch.rs` (`ChildWatch`);
  `push/child_watch_clock_marks_and_stopping.rs` (`WatchTerms`).
  `push/child_watch_task_loop_and_notice.rs` uses `push/bounded_push.rs` (`StopCause`);
  `push/child_watch_clock_marks_and_stopping.rs` (`Mark`,
  `Marks`,
  `SHORTEST_SLEEP`,
  `WatchFlags`,
  `WatchTerms`,
  `stop_process`,
  `RootPin`,
  `the_root_is_still_there`);
  `push/push_progress.rs` (`Clock`,
  `SAMPLE_INTERVAL`).
  `push/push_progress.rs` uses `push/bounded_push.rs` (`StopCause`,
  `StopReport`,
  `StopWindow`).
  `push/push_server_naming.rs` uses `push/bounded_push.rs` (`STOP_GRACE`);
  `push/bounded_query.rs` (`QueryOutput`,
  `query_window`,
  `run_bounded_query_awaiting`).
  `push/push_wait_until_done.rs` uses `push/auto_push_single_flight.rs` (`SingleFlightError`);
  `push/push_progress.rs` (`Clock`,
  `WaitReport`).
  Break:
  `PushAttempt` and `SingleFlightError` move into a new file `one_push_attempt_and_its_failure`;
  `MAX_PUSH_OUTPUT`,
  `stop_push`,
  `wait_until_done`,
  `run_bounded_push_once`,
  `flooded`,
  `wait_line_words` and `WAIT_NOTE_PHRASE` move into `push_wait_until_done`;
  `StopCause` moves into `push_progress`;
  and
  `STOP_GRACE` and `Patience` are read from the files that define them.

- Loop of 10,
  `worktree`:
  `worktree/worktree_snapshot_source.rs`,
  `worktree/worktree_trash.rs`,
  `worktree/worktree_trash_dispose.rs`,
  `worktree/worktree_trash_dispose_awaiting.rs`,
  `worktree/worktree_trash_hand_over.rs`,
  `worktree/worktree_trash_ignore_rule.rs`,
  `worktree/worktree_trash_own_manifest.rs`,
  `worktree/worktree_trash_set_aside.rs`,
  `worktree/worktree_trash_set_aside_entries.rs`,
  `worktree/worktree_trash_unaccounted.rs`.
  `worktree/worktree_snapshot_source.rs` uses `worktree/worktree_trash_ignore_rule.rs` (`trash_location_is_ignored`,
  `trash_location_is_ignored_awaiting`).
  `worktree/worktree_trash.rs` uses `worktree/worktree_snapshot_source.rs` (`SnapshotSource`);
  `worktree/worktree_trash_dispose.rs` (`PendingTrash`,
  `TRASH_REFERENCE`,
  `TrashLeft`,
  `delete_trash`,
  `delete_trash_awaiting`,
  `empty_trash`);
  `worktree/worktree_trash_dispose_awaiting.rs` (`empty_trash_awaiting`);
  `worktree/worktree_trash_hand_over.rs` (`Reach`,
  `Recorded`,
  `hand_over_to_the_service`,
  `hand_over_with`,
  `record_the_trash`);
  `worktree/worktree_trash_set_aside.rs` (the child module);
  `worktree/worktree_trash_set_aside_entries.rs` (the child module);
  `worktree/worktree_trash_unaccounted.rs` (`name_unaccounted_trash`).
  `worktree/worktree_trash_dispose.rs` uses `worktree/worktree_trash.rs` (`TRASH_PARENT`);
  `worktree/worktree_trash_own_manifest.rs` (`MANIFEST_FILE_NAME`,
  `OwnManifest`,
  `its_own`).
  `worktree/worktree_trash_dispose_awaiting.rs` uses `worktree/worktree_trash_dispose.rs` (`delete_trash_awaiting`).
  `worktree/worktree_trash_hand_over.rs` uses `worktree/worktree_trash.rs` (`PendingTrash`,
  `TRASH_PARENT`,
  `TRASH_REFERENCE`);
  `worktree/worktree_trash_dispose.rs` (the child module).
  `worktree/worktree_trash_ignore_rule.rs` uses `worktree/worktree_trash.rs` (`TRASH_PARENT`).
  `worktree/worktree_trash_own_manifest.rs` uses `worktree/worktree_trash.rs` (`TRASH_PARENT`).
  `worktree/worktree_trash_set_aside.rs` uses `worktree/worktree_trash.rs` (the child module).
  `worktree/worktree_trash_set_aside_entries.rs` uses `worktree/worktree_trash.rs` (the child module);
  `worktree/worktree_trash_set_aside.rs` (the child module).
  `worktree/worktree_trash_unaccounted.rs` uses `worktree/worktree_trash.rs` (`TRASH_PARENT`);
  `worktree/worktree_trash_dispose.rs` (`PendingTrash`,
  `RECORD_LIMIT`);
  `worktree/worktree_trash_hand_over.rs` (`Recorded`,
  `record_the_trash`);
  `worktree/worktree_trash_own_manifest.rs` (`read`).
  Break:
  `trash_location_is_ignored` moves into `worktree_trash_ignore_rule`,
  and a new file
  `the_names_and_the_manifest_of_the_trash` receives `TRASH_PARENT`,
  `TRASH_REFERENCE`,
  `MANIFEST_FILE_NAME`,
  `OwnManifest` and `PendingTrash`,
  which the nine trash files all name.

- Loop of 11,
  `push`:
  `push/auto_push.rs`,
  `push/auto_push_awaiting.rs`,
  `push/auto_push_reconcile.rs`,
  `push/auto_push_reconcile_awaiting.rs`,
  `push/push_failure_notice.rs`,
  `push/push_failure_notice_reports.rs`,
  `push/push_failure_notice_words.rs`,
  `push/push_retry.rs`,
  `push/push_retry_failure.rs`,
  `push/push_retry_store.rs`,
  `push/push_tracking_ref_repair.rs`.
  `push/auto_push.rs` uses `push/auto_push_awaiting.rs` (`push_once_awaiting`);
  `push/auto_push_reconcile.rs` (`RECONCILE_FAILED_PREFIX`,
  `RECONCILE_LINE_PREFIX`);
  `push/push_retry_failure.rs` (`record_words`).
  `push/auto_push_awaiting.rs` uses `push/auto_push.rs` (`AutoPushOutcome`,
  `AutoPushResult`,
  `PUSH_LOCK_PAUSE`,
  `PUSH_LOCK_RETRIES`,
  `Planned`,
  `REFERENCE_LOCK_REFUSAL`,
  `Refspec`,
  `Sent`,
  `TAG`,
  `held_back_result`,
  `moved_past_the_pin`,
  `push_arguments`,
  `pushed_tracking_ref`,
  `refspec_of`,
  `reported`,
  `skipped_with`,
  `source_of`);
  `push/auto_push_reconcile_awaiting.rs` (`reconcile_divergent_push_awaiting`);
  `push/push_tracking_ref_repair.rs` (`resolve_awaiting`,
  `bring_up_to_date_awaiting`,
  `still_behind`).
  `push/auto_push_reconcile.rs` uses `push/auto_push_reconcile_awaiting.rs` (`MergePusherAwaiting`,
  `integrate_and_push_awaiting`).
  `push/auto_push_reconcile_awaiting.rs` uses `push/push_retry_failure.rs` (`CONFLICT_PHRASE`).
  `push/push_failure_notice.rs` uses `push/push_failure_notice_reports.rs` (`REPORT_MAGIC`,
  `record_push_report`);
  `push/push_failure_notice_words.rs` (`PushOutcome`,
  `push_failure_text`,
  `worded`,
  `worded_lost`,
  `worded_report`,
  `worded_skipped`).
  `push/push_failure_notice_reports.rs` uses `push/push_failure_notice.rs` (`BRANCH_FIELD`,
  `COMMIT_FIELD`,
  `LONGEST_KEY`,
  `NAME_SEPARATOR`,
  `clean_words`,
  `fnv_hex`,
  `notice_directory`).
  `push/push_failure_notice_words.rs` uses `push/auto_push.rs` (`PUSH_FAILED_NOTE`);
  `push/push_failure_notice.rs` (`NO_REASON`,
  `clean_reason`).
  `push/push_retry.rs` uses `push/push_retry_store.rs` (`LoadedPlans`,
  `load_reporting`).
  `push/push_retry_failure.rs` uses `push/push_retry.rs` (`pause_before`).
  `push/push_retry_store.rs` uses `push/push_failure_notice.rs` (`fnv_hex`).
  `push/push_tracking_ref_repair.rs` uses `push/auto_push.rs` (`PUSH_LOCK_PAUSE`,
  `PUSH_LOCK_RETRIES`).
  Break:
  `fnv_hex` moves into a new file `file_name_of_one_branch`;
  `PUSH_FAILED_NOTE` moves into `push_failure_notice`;
  `CONFLICT_PHRASE` moves into `auto_push_reconcile_awaiting`;
  and
  `PUSH_LOCK_PAUSE`,
  `PUSH_LOCK_RETRIES` and `TAG` move into a new file `the_words_a_push_repeats`.

- Loop of 12,
  `worktree`:
  `worktree/worktree_removal_clears_the_ahead_pool.rs`,
  `worktree/worktree_removal_finishes_an_interrupted_destroy.rs`,
  `worktree/worktree_removal_goes_around_a_foreign_volume.rs`,
  `worktree/worktree_removal_interrupted_destroy.rs`,
  `worktree/worktree_removal_takes_its_pending_deletions.rs`,
  `worktree/worktree_remove.rs`,
  `worktree/worktree_remove_around.rs`,
  `worktree/worktree_remove_hand_over.rs`,
  `worktree/worktree_remove_last_look.rs`,
  `worktree/worktree_remove_reports.rs`,
  `worktree/worktree_snapshot_ahead.rs`,
  `worktree/worktree_snapshot_ahead_claim.rs`.
  `worktree/worktree_removal_clears_the_ahead_pool.rs` uses `worktree/worktree_remove_hand_over.rs`
  (`leave_a_subvolume_for_the_service`);
  `worktree/worktree_snapshot_ahead.rs` (`POOL_FOLDER`).
  `worktree/worktree_removal_finishes_an_interrupted_destroy.rs` uses
  `worktree/worktree_removal_clears_the_ahead_pool.rs` (`clear_the_pools_ahead_of_a_worktree`);
  `worktree/worktree_removal_interrupted_destroy.rs` (`DestroyRecord`,
  `RECORD_FOLDER`,
  `RECORD_LIMIT`,
  `forget_destroy`,
  `read_record`).
  `worktree/worktree_removal_goes_around_a_foreign_volume.rs` uses
  `worktree/worktree_removal_takes_its_pending_deletions.rs` (`is_a_leaving_name`);
  `worktree/worktree_remove_reports.rs` (`prune_command`).
  `worktree/worktree_removal_interrupted_destroy.rs` uses `worktree/worktree_removal_finishes_an_interrupted_destroy.rs`
  (`finish_interrupted_destroys`).
  `worktree/worktree_removal_takes_its_pending_deletions.rs` uses `worktree/worktree_remove_hand_over.rs`
  (`DELETING_PREFIX`,
  `leave_a_subvolume_for_the_service`).
  `worktree/worktree_remove.rs` uses `worktree/worktree_removal_clears_the_ahead_pool.rs`
  (`clear_the_pools_ahead_of_a_worktree`,
  `data_folders`);
  `worktree/worktree_removal_goes_around_a_foreign_volume.rs` (`volumes_below`);
  `worktree/worktree_removal_interrupted_destroy.rs` (`forget_destroy`,
  `record_destroy`,
  `record_destroyed`);
  `worktree/worktree_removal_takes_its_pending_deletions.rs` (`VolumesBelow`,
  `take_the_pending_deletions_below`);
  `worktree/worktree_remove_around.rs` (`Continuation`,
  `continuation_of`,
  `refusal_beside_a_volume`,
  `go_around`);
  `worktree/worktree_remove_hand_over.rs` (`hand_over_removal`);
  `worktree/worktree_remove_last_look.rs` (the child module);
  `worktree/worktree_remove_reports.rs` (`incomplete_report`,
  `leftover_stage_report`,
  `pending_deletion_report`,
  `registration_volume_report`,
  `snapshots_ahead_report`,
  `stage_state`,
  `unwalkable_registration_report`).
  `worktree/worktree_remove_around.rs` uses `worktree/worktree_removal_goes_around_a_foreign_volume.rs`
  (`around_incomplete_report`,
  `stopped_around_report`,
  `volumes_below`,
  `refused_beside_a_volume_report`,
  `BesideAVolume`);
  `worktree/worktree_removal_interrupted_destroy.rs` (`forget_destroy`);
  `worktree/worktree_remove.rs` (`Held`,
  `RemoveOutcome`,
  `RemoveRequest`,
  `TAG`,
  `finish`).
  `worktree/worktree_remove_hand_over.rs` uses `worktree/worktree_removal_goes_around_a_foreign_volume.rs`
  (`volumes_below`,
  `emptied_around_report`,
  `record_left_in_place`,
  `around_removal_report`);
  `worktree/worktree_removal_interrupted_destroy.rs` (`record_leaving`);
  `worktree/worktree_remove.rs` (`RemoveOutcome`,
  `RemoveRequest`,
  `remove_with`,
  `LinuxOperations`).
  `worktree/worktree_remove_last_look.rs` uses `worktree/worktree_remove.rs` (the child module).
  `worktree/worktree_remove_reports.rs` uses `worktree/worktree_snapshot_ahead.rs` (`pool_folder`).
  `worktree/worktree_snapshot_ahead.rs` uses `worktree/worktree_remove_hand_over.rs` (`DELETE_REFERENCE`);
  `worktree/worktree_snapshot_ahead_claim.rs` (`look_for_a_snapshot_ahead`).
  `worktree/worktree_snapshot_ahead_claim.rs` uses `worktree/worktree_snapshot_ahead.rs` (`AHEAD_KEPT`,
  `AheadLook`,
  `AheadService`,
  `POLL`,
  `ahead_key`,
  `candidates`,
  `kept_in`,
  `made_at`,
  `over_kept`,
  `pool_folder`).
  Break:
  a new file
  `what_a_removal_is_asked_and_answers` receives `RemoveRequest`,
  `RemoveOutcome`,
  `TAG` and `finish`,
  which the around pass,
  the hand-over and the reports all name,
  and
  `POOL_FOLDER` and `DELETE_REFERENCE` move beside the snapshot-ahead files that read them.

- Loop of 14,
  `landing`:
  `landing/landing_state_carried_by_its_task.rs`,
  `landing/service_durable.rs`,
  `landing/service_index_attempts.rs`,
  `landing/service_index_claim.rs`,
  `landing/service_index_hold.rs`,
  `landing/service_index_lock.rs`,
  `landing/service_index_ownership.rs`,
  `landing/service_index_placement.rs`,
  `landing/service_index_prestage.rs`,
  `landing/service_index_reclaim.rs`,
  `landing/service_index_refused_lock.rs`,
  `landing/service_index_unheld_removal.rs`,
  `landing/service_land_copy.rs`,
  `landing/service_land_follow.rs`.
  `landing/landing_state_carried_by_its_task.rs` uses `landing/service_durable.rs` (`Pending`);
  `landing/service_index_prestage.rs` (`Slot`);
  `landing/service_index_refused_lock.rs` (`Refused`).
  `landing/service_durable.rs` uses `landing/landing_state_carried_by_its_task.rs` (`LandingState`,
  `on_the_landing_state`).
  `landing/service_index_attempts.rs` uses `landing/service_index_lock.rs` (`Announce`,
  `Announcer`,
  `LockFailure`,
  `notice_every`,
  `lock_path_of`);
  `landing/service_index_placement.rs` (`Placed`,
  `Platform`,
  `Reclaimed`,
  `log_placement_failure`,
  `place_lock`,
  `reclaim`);
  `landing/service_index_reclaim.rs` (`foreign_words`,
  `stale_lock_words`,
  `unjudged_words`);
  `landing/service_index_refused_lock.rs` (`forget`,
  `recall`,
  `remember`);
  `landing/service_index_unheld_removal.rs` (`Unheld`,
  `remove_if_unheld`,
  `told_words`);
  `landing/service_land_follow.rs` (`storage_failure`).
  `landing/service_index_claim.rs` uses `landing/landing_state_carried_by_its_task.rs` (`LandingState`,
  `on_the_landing_state`);
  `landing/service_index_placement.rs` (`Platform`,
  `place_marked`).
  `landing/service_index_hold.rs` uses `landing/service_durable.rs` (`flush_index`,
  `flush_directory`);
  `landing/service_index_ownership.rs` (`Ownership`,
  `note_unsettled`).
  `landing/service_index_lock.rs` uses `landing/service_index_attempts.rs` (`Placing`,
  `wait_and_place`);
  `landing/service_index_hold.rs` (`IndexLock`,
  `Record`);
  `landing/service_index_ownership.rs` (`Ownership`,
  `inspect_ownership`,
  `unsettled_locks`);
  `landing/service_index_placement.rs` (`Placed`,
  `Platform`,
  `SYSTEM`,
  `placement_failure_text`,
  `log_placement_failure`,
  `place_lock`,
  `reclaim`);
  `landing/service_index_prestage.rs` (`Prestaged`,
  `take`);
  `landing/service_index_reclaim.rs` (`Reclaimed`,
  `foreign_words`,
  `remove_stale`,
  `stale_lock_words`,
  `unjudged_words`);
  `landing/service_land_follow.rs` (`storage_failure`).
  `landing/service_index_ownership.rs` uses `landing/landing_state_carried_by_its_task.rs` (`LandingState`,
  `on_the_landing_state`).
  `landing/service_index_placement.rs` uses `landing/service_durable.rs` (`defer_file`);
  `landing/service_index_ownership.rs` (`Ownership`,
  `inspect_ownership`);
  `landing/service_index_reclaim.rs` (`Reclaimed`,
  `StaleMark`,
  `reclaim_if`).
  `landing/service_index_prestage.rs` uses `landing/landing_state_carried_by_its_task.rs` (`LandingState`,
  `on_the_landing_state`,
  `with_the_landing_state`);
  `landing/service_index_lock.rs` (`lock_path_of`);
  `landing/service_index_placement.rs` (`sync_file`).
  `landing/service_index_reclaim.rs` uses `landing/service_index_claim.rs` (`Claim`,
  `ClaimHold`,
  `take_claim`);
  `landing/service_index_lock.rs` (`lock_path_of`);
  `landing/service_index_placement.rs` (`Platform`,
  `SYSTEM`).
  `landing/service_index_refused_lock.rs` uses `landing/landing_state_carried_by_its_task.rs` (`LandingState`,
  `on_the_landing_state`).
  `landing/service_index_unheld_removal.rs` uses `landing/service_index_lock.rs` (`lock_path_of`);
  `landing/service_index_placement.rs` (`Platform`);
  `landing/service_index_reclaim.rs` (`open_to_judge`,
  `same_file`).
  `landing/service_land_copy.rs` uses `landing/service_index_hold.rs` (the child module);
  `landing/service_index_lock.rs` (`IndexLock`);
  `landing/service_land_follow.rs` (`storage_failure`).
  `landing/service_land_follow.rs` uses `landing/service_index_hold.rs` (the child module);
  `landing/service_index_lock.rs` (`IndexLock`);
  `landing/service_land_copy.rs` (`PrivateCopy`).
  Break:
  a new file
  `the_names_of_the_index_lock` receives `lock_path_of`,
  `Announce`,
  `Announcer`,
  `LockFailure`,
  `notice_every` and `storage_failure`;
  `reclaim` moves from `service_index_placement` into `service_index_reclaim`;
  and
  `LandingState` and `on_the_landing_state` move into a new file `the_landing_state_one_task_carries`.

- Loop of 14,
  `repo_read`:
  `repo_read/git_metadata.rs`,
  `repo_read/repo_read_diff.rs`,
  `repo_read/repo_read_hook.rs`,
  `repo_read/repo_read_index.rs`,
  `repo_read/repo_read_layout.rs`,
  `repo_read/repo_read_object.rs`,
  `repo_read/repo_read_odb.rs`,
  `repo_read/repo_read_pack.rs`,
  `repo_read/repo_read_query.rs`,
  `repo_read/repo_read_query_quiet.rs`,
  `repo_read/repo_read_query_refs.rs`,
  `repo_read/repo_read_query_request_answer_and_context.rs`,
  `repo_read/repo_read_tree.rs`,
  `repo_read/repo_read_tree_cached.rs`.
  `repo_read/git_metadata.rs` uses `repo_read/repo_read_hook.rs` (`try_metadata`).
  `repo_read/repo_read_diff.rs` uses `repo_read/repo_read_object.rs` (`to_hex`);
  `repo_read/repo_read_tree.rs` (`FlatEntry`,
  `MODE_GITLINK`).
  `repo_read/repo_read_hook.rs` uses `repo_read/git_metadata.rs` (`MetadataOutput`);
  `repo_read/repo_read_query.rs` (`Answer`,
  `Query`,
  `answer_with`);
  `repo_read/repo_read_query_request_answer_and_context.rs` (the child module).
  `repo_read/repo_read_index.rs` uses `repo_read/repo_read_layout.rs` (`ObjectFormat`);
  `repo_read/repo_read_object.rs` (`name_length`).
  `repo_read/repo_read_layout.rs` uses `repo_read/repo_read_hook.rs` (`enabled`).
  `repo_read/repo_read_object.rs` uses `repo_read/repo_read_layout.rs` (`ObjectFormat`).
  `repo_read/repo_read_odb.rs` uses `repo_read/repo_read_layout.rs` (`ObjectFormat`,
  `RepoLayout`);
  `repo_read/repo_read_object.rs` (`OBJECT_SIZE_LIMIT`,
  `RawObject`,
  `name_length`,
  `parse_loose`,
  `to_hex`);
  `repo_read/repo_read_pack.rs` (`BaseByName`,
  `Pack`).
  `repo_read/repo_read_pack.rs` uses `repo_read/repo_read_object.rs` (`OBJECT_SIZE_LIMIT`,
  `ObjectKind`,
  `RawObject`,
  `inflate_exact`).
  `repo_read/repo_read_query.rs` uses `repo_read/repo_read_diff.rs` (`PathFilter`,
  `diff_sorted`,
  `name_lines`,
  `parse_diff_filter`,
  `raw_lines`,
  `select`);
  `repo_read/repo_read_layout.rs` (`RepoLayout`,
  `env_config_avoids`,
  `read_layout_with`);
  `repo_read/repo_read_object.rs` (`ObjectKind`,
  `from_hex`,
  `name_length`,
  `to_hex`);
  `repo_read/repo_read_odb.rs` (`ObjectDatabase`);
  `repo_read/repo_read_query_quiet.rs` (`diff_index_quiet`);
  `repo_read/repo_read_query_refs.rs` (`config_list`,
  `for_each_ref`,
  `remote_url`,
  `rev_parse`,
  `symbolic_ref`);
  `repo_read/repo_read_query_request_answer_and_context.rs` (`Answer`,
  `Query`,
  `Context`,
  `Parsed`,
  `flags_are`,
  `parse`);
  `repo_read/repo_read_tree.rs` (`FlatEntry`,
  `flatten_pair`,
  `flatten_tree_shared`,
  `parse_commit`,
  `subject_of`);
  `repo_read/repo_read_tree_cached.rs` (`changes_with_cache`).
  `repo_read/repo_read_query_quiet.rs` uses `repo_read/repo_read_diff.rs` (`PathFilter`);
  `repo_read/repo_read_index.rs` (`INTENT_TO_ADD`,
  `IndexEntry`,
  `ParsedIndex`);
  `repo_read/repo_read_object.rs` (`ObjectKind`,
  `from_hex`);
  `repo_read/repo_read_odb.rs` (`ObjectDatabase`);
  `repo_read/repo_read_query_request_answer_and_context.rs` (`Answer`,
  `Context`,
  `Parsed`,
  `flags_are`);
  `repo_read/repo_read_tree.rs` (`FlatEntry`,
  `MODE_GITLINK`,
  `flatten_tree_shared`,
  `parse_commit`);
  `repo_read/repo_read_tree_cached.rs` (`UnequalParts`,
  `outside`,
  `unequal_parts`).
  `repo_read/repo_read_query_refs.rs` uses `repo_read/repo_read_object.rs` (`ObjectKind`,
  `from_hex`,
  `to_hex`);
  `repo_read/repo_read_odb.rs` (`ObjectDatabase`);
  `repo_read/repo_read_query_request_answer_and_context.rs` (`Answer`,
  `Context`,
  `Parsed`,
  `flags_are`).
  `repo_read/repo_read_query_request_answer_and_context.rs` uses `repo_read/repo_read_index.rs` (`INTENT_TO_ADD`,
  `IndexEntry`,
  `ParsedIndex`,
  `parse_index_with_tree`);
  `repo_read/repo_read_layout.rs` (`RepoLayout`);
  `repo_read/repo_read_object.rs` (`ObjectKind`,
  `from_hex`);
  `repo_read/repo_read_odb.rs` (`ObjectDatabase`);
  `repo_read/repo_read_tree.rs` (`FlatEntry`,
  `parse_commit`).
  `repo_read/repo_read_tree.rs` uses `repo_read/repo_read_layout.rs` (`ObjectFormat`);
  `repo_read/repo_read_object.rs` (`ObjectKind`,
  `name_length`,
  `from_hex`);
  `repo_read/repo_read_odb.rs` (`ObjectDatabase`).
  `repo_read/repo_read_tree_cached.rs` uses `repo_read/repo_read_diff.rs` (`Change`,
  `diff_sorted`);
  `repo_read/repo_read_layout.rs` (`ObjectFormat`);
  `repo_read/repo_read_object.rs` (`ObjectKind`,
  `name_length`);
  `repo_read/repo_read_odb.rs` (`ObjectDatabase`);
  `repo_read/repo_read_tree.rs` (`FlatEntry`,
  `is_tree_mode`,
  `parse_tree`).
  Break:
  `MetadataOutput` moves into `repo_read_query_request_answer_and_context`,
  and
  `enabled` moves into `repository_location_without_git`,
  both as the 2026-11-07 plan already said.

- Loop of 19,
  `merge`:
  `merge/merge_isolation_carry_apply.rs`,
  `merge/merge_isolation_carry_kept_path.rs`,
  `merge/merge_isolation_carry_notice.rs`,
  `merge/merge_isolation_carry_place.rs`,
  `merge/merge_isolation_carry_plans.rs`,
  `merge/merge_isolation_carry_writes.rs`,
  `merge/merge_isolation_index_reading.rs`,
  `merge/merge_isolation_kept_copies_decisions.rs`,
  `merge/merge_isolation_moved.rs`,
  `merge/merge_isolation_recovery.rs`,
  `merge/merge_isolation_recovery_carry.rs`,
  `merge/merge_isolation_recovery_cleanup.rs`,
  `merge/merge_isolation_recovery_complete.rs`,
  `merge/merge_isolation_recovery_install.rs`,
  `merge/merge_isolation_recovery_notice.rs`,
  `merge/merge_isolation_recovery_reads.rs`,
  `merge/merge_isolation_recovery_records.rs`,
  `merge/merge_isolation_recovery_steps.rs`,
  `merge/merge_isolation_take_asides.rs`.
  `merge/merge_isolation_carry_apply.rs` uses `merge/merge_isolation_carry_kept_path.rs` (`Staged`,
  `index_after`,
  `staged_of`);
  `merge/merge_isolation_carry_notice.rs` (`Report`,
  `add_to_the_report`,
  `note_left_alone`);
  `merge/merge_isolation_carry_place.rs` (`explained_entries`,
  `judged_of`);
  `merge/merge_isolation_carry_plans.rs` (`plan_index_awaiting`,
  `plan_worktree_awaiting`);
  `merge/merge_isolation_carry_writes.rs` (`Batch`,
  `FileWrite`,
  `IndexWrite`,
  `Step`,
  `StepResult`,
  `write_steps_awaiting`,
  `CarryFailure`,
  `LockWait`);
  `merge/merge_isolation_take_asides.rs` (`put_back_stranded_takes_awaiting`).
  `merge/merge_isolation_carry_kept_path.rs` uses `merge/merge_isolation_carry_writes.rs` (`IndexWrite`).
  `merge/merge_isolation_carry_notice.rs` uses `merge/merge_isolation_carry_kept_path.rs` (`Kept`,
  `Staged`,
  `kept_line`,
  `kept_path`);
  `merge/merge_isolation_carry_plans.rs` (`Note`).
  `merge/merge_isolation_carry_place.rs` uses `merge/merge_isolation_carry_writes.rs` (`CarryFailure`,
  `FileWrite`,
  `IndexWrite`);
  `merge/merge_isolation_take_asides.rs` (`are_the_file_form_of_awaiting`,
  `may_remove_the_aside_awaiting`).
  `merge/merge_isolation_carry_plans.rs` uses `merge/merge_isolation_carry_writes.rs` (`CarryFailure`,
  `FileWrite`,
  `IndexWrite`).
  `merge/merge_isolation_carry_writes.rs` uses `merge/merge_isolation_carry_place.rs` (`JudgedFile`,
  `Placed`,
  `SetAsides`,
  `is_still_the_judged_entry`,
  `place_file_awaiting`).
  `merge/merge_isolation_index_reading.rs` uses `merge/merge_isolation_recovery_steps.rs` (`RecoveryPlace`,
  `Standing`).
  `merge/merge_isolation_kept_copies_decisions.rs` uses `merge/merge_isolation_recovery_records.rs` (`owner_liveness`);
  `merge/merge_isolation_recovery_steps.rs` (`RecoveryPlace`);
  `merge/merge_isolation_take_asides.rs` (`CopyRemoval`,
  `ProofNothingCanWrite`,
  `may_remove_the_copy_awaiting`,
  `raw_bytes_of_awaiting`,
  `the_file_form_of_awaiting`).
  `merge/merge_isolation_moved.rs` uses `merge/merge_isolation_index_reading.rs` (`entries_by_path`,
  `index_entries_awaiting`,
  `standing_of_index`);
  `merge/merge_isolation_recovery_cleanup.rs` (`an_installed_merge_behind_a_held_index`);
  `merge/merge_isolation_recovery_install.rs` (the child module);
  `merge/merge_isolation_recovery_steps.rs` (`IndexHeldAt`,
  `Installed`,
  `Judged`,
  `MergeRecoveryAction`,
  `MergeRefusal`,
  `RecoveryPlace`,
  `Standing`,
  `index_holds_the_merge_awaiting`,
  `install_entries_awaiting`,
  `judge_changes_awaiting`,
  `merge_changes_awaiting`,
  `put_back_stranded_takes_awaiting`,
  `remove_record`);
  `merge/merge_isolation_take_asides.rs` (the child module).
  `merge/merge_isolation_recovery.rs` uses `merge/merge_isolation_index_reading.rs` (the child module);
  `merge/merge_isolation_moved.rs` (`settle_moved_branch_awaiting`);
  `merge/merge_isolation_recovery_carry.rs` (`SideRef`,
  `carry_onto_awaiting`,
  `carry_the_capture_awaiting`,
  `drop_the_records_refs_then_the_record_or_say`,
  `restore_the_capture_awaiting`);
  `merge/merge_isolation_recovery_cleanup.rs` (`settle_a_saved_merge_over_newer_edits`,
  `a_landed_merge_in_another_branchs_worktree`,
  `edits_on_the_merges_own_paths`,
  `no_commit_names_the_merges_parent`,
  `staged_work_on_the_merges_paths`,
  `drop_record_and_side_ref_awaiting`,
  `partly_installed`);
  `merge/merge_isolation_recovery_complete.rs` (`complete_awaiting`);
  `merge/merge_isolation_recovery_notice.rs` (`recovery_notice`,
  `saved_merge_waits_for_the_index_lock`);
  `merge/merge_isolation_recovery_reads.rs` (`is_ancestor_awaiting`,
  `holds_the_merge`,
  `holds_the_merges_entry`,
  `paths_the_branch_does_not_hold_awaiting`,
  `resolve_commit_awaiting`,
  `symbolic_ref_awaiting`,
  `the_capture_holds_every_file_in_the_way`,
  `the_paths_hold_only_the_persons_own_bytes_awaiting`);
  `merge/merge_isolation_recovery_records.rs` (`MergeRecoveryOutcome`,
  `merge_intents_to_recover`,
  `recover_interrupted_merges`,
  `recover_interrupted_merges_awaiting`);
  `merge/merge_isolation_recovery_steps.rs` (`IndexHeldAt`,
  `Installed`,
  `Judged`,
  `MergeRecoveryAction`,
  `MergeRefusal`,
  `RecoveryPlace`,
  `Standing`,
  `TakenOnDisk`,
  `finish_asides`,
  `merge_changes`,
  `index_standings_awaiting`,
  `judge_changes_awaiting`,
  `merge_changes_awaiting`,
  `put_back_stranded_takes_awaiting`);
  `merge/merge_isolation_take_asides.rs` (the child module).
  `merge/merge_isolation_recovery_carry.rs` uses `merge/merge_isolation_carry_apply.rs` (`CarryFailure`,
  `LandedFile`,
  `LockWait`,
  `carry_awaiting`,
  `changes_between_awaiting`,
  `restore_awaiting`);
  `merge/merge_isolation_carry_notice.rs` (`report_lines`);
  `merge/merge_isolation_carry_writes.rs` (the child module);
  `merge/merge_isolation_recovery_cleanup.rs` (`partly_installed`,
  `drop_side_ref_awaiting`);
  `merge/merge_isolation_recovery_steps.rs` (`MergeRefusal`,
  `RecoveryPlace`,
  `merge_changes_awaiting`,
  `remove_record`).
  `merge/merge_isolation_recovery_cleanup.rs` uses `merge/merge_isolation_recovery_carry.rs`
  (`drop_the_records_refs_then_the_record`,
  `SideRef`,
  `drop_capture_awaiting`);
  `merge/merge_isolation_recovery_reads.rs` (`resolve_commit_awaiting`);
  `merge/merge_isolation_recovery_steps.rs` (`MergeRefusal`,
  `RecoveryPlace`,
  `MergeRecoveryAction`).
  `merge/merge_isolation_recovery_complete.rs` uses `merge/merge_isolation_recovery_cleanup.rs`
  (`a_branch_that_could_not_be_put_back`,
  `a_landed_merge_in_another_branchs_worktree`,
  `a_worktree_that_switched`,
  `an_installed_merge_behind_a_held_index`,
  `partly_installed`);
  `merge/merge_isolation_recovery_install.rs` (the child module);
  `merge/merge_isolation_recovery_reads.rs` (`resolve_commit_awaiting`,
  `symbolic_ref_awaiting`);
  `merge/merge_isolation_recovery_steps.rs` (`Installed`,
  `Judged`,
  `MergeRefusal`,
  `RecoveryPlace`,
  `install_entries_awaiting`).
  `merge/merge_isolation_recovery_install.rs` uses `merge/merge_isolation_recovery_steps.rs` (`Installed`,
  `Judged`,
  `MergeRefusal`,
  `RecoveryPlace`,
  `Standing`,
  `TakenOnDisk`);
  `merge/merge_isolation_take_asides.rs` (`Take`,
  `finish_asides_awaiting`,
  `put_back_awaiting`,
  `restore_what_git_did_not_write_awaiting`,
  `take_aside_awaiting`).
  `merge/merge_isolation_recovery_notice.rs` uses `merge/merge_isolation_recovery.rs` (`MergeRecoveryOutcome`);
  `merge/merge_isolation_recovery_records.rs` (the child module);
  `merge/merge_isolation_recovery_steps.rs` (`MergeRecoveryAction`).
  `merge/merge_isolation_recovery_reads.rs` uses `merge/merge_isolation_recovery_steps.rs` (`Judged`,
  `MergeRefusal`,
  `RecoveryPlace`,
  `Standing`);
  `merge/merge_isolation_take_asides.rs` (`the_file_form_of_awaiting`).
  `merge/merge_isolation_recovery_records.rs` uses `merge/merge_isolation_kept_copies_decisions.rs`
  (`sweep_kept_copies_awaiting`);
  `merge/merge_isolation_recovery.rs` (`settle_failed_merge_awaiting`);
  `merge/merge_isolation_recovery_notice.rs` (`recovery_notice`);
  `merge/merge_isolation_recovery_steps.rs` (`MergeRecoveryAction`,
  `RecoveryPlace`).
  `merge/merge_isolation_recovery_steps.rs` uses `merge/merge_isolation_index_reading.rs` (`entries_by_path`,
  `index_entries`,
  `index_entries_awaiting`,
  `index_standings`,
  `index_standings_awaiting`,
  `standing_of_index`);
  `merge/merge_isolation_recovery_install.rs` (`install_entries`,
  `install_entries_awaiting`,
  `remove_empty_parents`);
  `merge/merge_isolation_take_asides.rs` (`Take`,
  `TakeFailure`,
  `TakenOnDisk`,
  `finish_asides`,
  `finish_asides_awaiting`,
  `put_back_awaiting`,
  `put_back_stranded_takes_awaiting`,
  `restore_what_git_did_not_write_awaiting`,
  `take_aside_awaiting`).
  `merge/merge_isolation_take_asides.rs` uses `merge/merge_isolation_kept_copies_decisions.rs`
  (`settle_the_asides_of_a_merge_awaiting`);
  `merge/merge_isolation_recovery_steps.rs` (`Judged`,
  `RecoveryPlace`).
  Break:
  a new file
  `a_merge_recovery_record` receives `MergeRefusal`,
  `RecoveryPlace`,
  `Installed`,
  `Judged`,
  `Standing`,
  `MergeRecoveryAction` and `TakenOnDisk`,
  and a new file
  `a_carry_write` receives `CarryFailure`,
  `FileWrite`,
  `IndexWrite`,
  `Batch`,
  `Step`,
  `StepResult` and `LockWait`.

- Loop of 28,
  `worktree_copying`:
  `worktree_copying/worktree_copy.rs`,
  `worktree_copying/worktree_copy_after_lock.rs`,
  `worktree_copying/worktree_copy_install_log.rs`,
  `worktree_copying/worktree_copy_interrupted_add_settlement.rs`,
  `worktree_copying/worktree_copy_interrupted_checkout.rs`,
  `worktree_copying/worktree_copy_journal.rs`,
  `worktree_copying/worktree_copy_journal_removal.rs`,
  `worktree_copying/worktree_copy_owed_copy.rs`,
  `worktree_copying/worktree_copy_owed_copy_proofs.rs`,
  `worktree_copying/worktree_copy_recovery.rs`,
  `worktree_copying/worktree_copy_registry.rs`,
  `worktree_copying/worktree_copy_source_still_being_filled.rs`,
  `worktree_copying/worktree_copy_stage.rs`,
  `worktree_copying/worktree_copy_stage_binding.rs`,
  `worktree_copying/worktree_copy_stage_claim.rs`,
  `worktree_copying/worktree_copy_stage_claim_owed_copy_elsewhere.rs`,
  `worktree_copying/worktree_copy_stage_claim_pointer_presence.rs`,
  `worktree_copying/worktree_copy_stage_claim_reaping.rs`,
  `worktree_copying/worktree_copy_stage_claim_swept_files.rs`,
  `worktree_copying/worktree_copy_stage_made_object.rs`,
  `worktree_copying/worktree_copy_stage_reaping_proof_and_words.rs`,
  `worktree_copying/worktree_copy_stage_snapshot_origin.rs`,
  `worktree_copying/worktree_copy_swapped_add_settlement.rs`,
  `worktree_copying/worktree_copy_synchronize.rs`,
  `worktree_copying/worktree_copy_transaction.rs`,
  `worktree_copying/worktree_copy_unfilled_record.rs`,
  `worktree_copying/worktree_copy_unreadable_record_aside.rs`,
  `worktree_copying/worktree_copy_unreadable_records.rs`.
  `worktree_copying/worktree_copy.rs` uses `worktree_copying/worktree_copy_after_lock.rs` (`AfterLock`,
  `finish_trash_and_repair`);
  `worktree_copying/worktree_copy_owed_copy.rs` (`owed_selection_of`);
  `worktree_copying/worktree_copy_recovery.rs` (`RecoveryDestinations`,
  `recover_pending_worktree_copies`);
  `worktree_copying/worktree_copy_registry.rs` (`WorktreeCopyObservation`,
  `destination_is_registered`,
  `observe_copy_source`);
  `worktree_copying/worktree_copy_stage_claim.rs` (`StageClaim`,
  `claim_stage_for_creation`);
  `worktree_copying/worktree_copy_synchronize.rs` (`synchronize_after`).
  `worktree_copying/worktree_copy_after_lock.rs` uses `worktree_copying/worktree_copy.rs` (`Invocation`).
  `worktree_copying/worktree_copy_install_log.rs` uses `worktree_copying/worktree_copy_journal.rs` (`Role`,
  `assert_private_path`).
  `worktree_copying/worktree_copy_interrupted_add_settlement.rs` uses `worktree_copying/worktree_copy_stage_claim.rs`
  (`COMPLETE_SUFFIX`);
  `worktree_copying/worktree_copy_stage_claim_pointer_presence.rs` (the child module);
  `worktree_copying/worktree_copy_stage_made_object.rs` (`the_made_object_stands_at`).
  `worktree_copying/worktree_copy_interrupted_checkout.rs` uses
  `worktree_copying/worktree_copy_interrupted_add_settlement.rs` (`INDEX`,
  `INDEX_LOCK`,
  `add_is_finished`).
  `worktree_copying/worktree_copy_journal.rs` uses `worktree_copying/worktree_copy_stage.rs` (`StagedSnapshot`);
  `worktree_copying/worktree_copy_unreadable_record_aside.rs` (the child module);
  `worktree_copying/worktree_copy_unreadable_records.rs` (`UNREADABLE_FOLDER`).
  `worktree_copying/worktree_copy_journal_removal.rs` uses `worktree_copying/worktree_copy_journal.rs`
  (`write_journal`).
  `worktree_copying/worktree_copy_owed_copy.rs` uses `worktree_copying/worktree_copy_interrupted_add_settlement.rs`
  (`registration_directory`,
  `the_add_finished`);
  `worktree_copying/worktree_copy_interrupted_checkout.rs` (`CheckoutFinish`,
  `finish_interrupted_checkout`);
  `worktree_copying/worktree_copy_journal.rs` (`journal_root`,
  `read_pending_journals`);
  `worktree_copying/worktree_copy_owed_copy_proofs.rs` (`clear_the_own_stage`,
  `source_is_registered`,
  `whole_registration`);
  `worktree_copying/worktree_copy_recovery.rs` (`RecoveryDestinations`,
  `finishes_destination`);
  `worktree_copying/worktree_copy_registry.rs` (`WorktreeCopyObservation`,
  `read_registered_roots`);
  `worktree_copying/worktree_copy_source_still_being_filled.rs` (`source_is_still_being_filled`);
  `worktree_copying/worktree_copy_stage_claim.rs` (`DESTINATION_SUFFIX`,
  `StageClaim`,
  `claim_names`);
  `worktree_copying/worktree_copy_stage_claim_reaping.rs` (`StageVerdicts`,
  `read_claim`);
  `worktree_copying/worktree_copy_swapped_add_settlement.rs` (`is_a_completed_exchange`);
  `worktree_copying/worktree_copy_synchronize.rs` (`synchronize_created_worktree_noting`);
  `worktree_copying/worktree_copy_unfilled_record.rs` (`remember_unfilled_with`).
  `worktree_copying/worktree_copy_owed_copy_proofs.rs` uses
  `worktree_copying/worktree_copy_interrupted_add_settlement.rs` (`add_is_finished`,
  `registration_directory`);
  `worktree_copying/worktree_copy_journal.rs` (`Role`,
  `assert_private_path`);
  `worktree_copying/worktree_copy_journal_removal.rs` (`remove_stage`);
  `worktree_copying/worktree_copy_stage_binding.rs` (`stage_binding`,
  `STAGE_BINDING_NAME`);
  `worktree_copying/worktree_copy_stage_claim_reaping.rs` (`SCRATCH_SUFFIX`).
  `worktree_copying/worktree_copy_recovery.rs` uses `worktree_copying/worktree_copy_journal.rs` (`Role`,
  `adopt_journal`,
  `assert_private_path`,
  `journal_owner_is_dead`,
  `read_pending_journals`,
  `journal_items_exist`);
  `worktree_copying/worktree_copy_journal_removal.rs` (`remove_pending_journal`);
  `worktree_copying/worktree_copy_owed_copy.rs` (`owed_in_another_worktree_notice`,
  `finish_owed_worktree_copies`);
  `worktree_copying/worktree_copy_stage_binding.rs` (`stage_binding`,
  `verify_stage_binding`);
  `worktree_copying/worktree_copy_stage_claim.rs` (`claim_naming_stage`);
  `worktree_copying/worktree_copy_stage_claim_reaping.rs` (`StageVerdicts`,
  `judge_stage_owners`,
  `reap_judged_stages`);
  `worktree_copying/worktree_copy_transaction.rs` (`Outcome`,
  `SnapshotSource`,
  `complete_journal`);
  `worktree_copying/worktree_copy_unfilled_record.rs` (`report_unfilled_records`);
  `worktree_copying/worktree_copy_unreadable_records.rs` (`settle_unreadable_records`).
  `worktree_copying/worktree_copy_registry.rs` uses `worktree_copying/worktree_copy_stage_claim.rs` (`StageClaim`).
  `worktree_copying/worktree_copy_source_still_being_filled.rs` uses `worktree_copying/worktree_copy_journal.rs`
  (`journal_owner_is_dead`,
  `journal_root`,
  `read_pending_journals`);
  `worktree_copying/worktree_copy_stage_claim.rs` (`DESTINATION_SUFFIX`,
  `claim_names`);
  `worktree_copying/worktree_copy_stage_claim_reaping.rs` (`read_claim`);
  `worktree_copying/worktree_copy_unfilled_record.rs` (`standing_record_of`).
  `worktree_copying/worktree_copy_stage.rs` uses `worktree_copying/worktree_copy_stage_binding.rs`
  (`record_stage_binding`).
  `worktree_copying/worktree_copy_stage_binding.rs` uses `worktree_copying/worktree_copy_journal.rs`
  (`write_atomic_unsynced`).
  `worktree_copying/worktree_copy_stage_claim.rs` uses `worktree_copying/worktree_copy_journal.rs`
  (`ensure_journal_root`,
  `journal_root`,
  `write_atomic`,
  `write_atomic_unsynced`);
  `worktree_copying/worktree_copy_journal_removal.rs` (`remove_file_if_present`);
  `worktree_copying/worktree_copy_stage.rs` (`new_stage_container`);
  `worktree_copying/worktree_copy_stage_made_object.rs` (`remove_made`).
  `worktree_copying/worktree_copy_stage_claim_owed_copy_elsewhere.rs` uses
  `worktree_copying/worktree_copy_interrupted_add_settlement.rs` (`add_is_finished`,
  `registration_directory`,
  `the_add_finished`);
  `worktree_copying/worktree_copy_recovery.rs` (`RecoveryDestinations`,
  `finishes_destination`,
  `destination_registration`,
  `Registration`);
  `worktree_copying/worktree_copy_stage_made_object.rs` (`the_made_object_stands_at`).
  `worktree_copying/worktree_copy_stage_claim_pointer_presence.rs` uses
  `worktree_copying/worktree_copy_interrupted_add_settlement.rs` (the child module).
  `worktree_copying/worktree_copy_stage_claim_reaping.rs` uses
  `worktree_copying/worktree_copy_interrupted_add_settlement.rs` (`*`);
  `worktree_copying/worktree_copy_journal.rs` (`Role`,
  `assert_private_path`,
  `journal_root`,
  `read_pending_journals`,
  `temporary_writer_token`);
  `worktree_copying/worktree_copy_journal_removal.rs` (`remove_file_if_present`,
  `remove_stage`);
  `worktree_copying/worktree_copy_recovery.rs` (`RecoveryDestinations`);
  `worktree_copying/worktree_copy_stage_claim.rs` (`COMPLETE_SUFFIX`,
  `CONTAINER_HEX_PREFIX`,
  `DESTINATION_SUFFIX`,
  `OWNER_LINE_PREFIX`,
  `claim_names`);
  `worktree_copying/worktree_copy_stage_claim_owed_copy_elsewhere.rs` (`copy_owed_to_another_worktree`);
  `worktree_copying/worktree_copy_stage_claim_swept_files.rs` (`remove_dead_temporaries`,
  `remove_records_nothing_claims`);
  `worktree_copying/worktree_copy_stage_made_object.rs` (`remove_made`);
  `worktree_copying/worktree_copy_stage_reaping_proof_and_words.rs` (`Provenance`,
  `kept_stage_words`,
  `provenance_of`,
  `removed_copy_stage_words`,
  `removed_stage_words`,
  `still_there`);
  `worktree_copying/worktree_copy_swapped_add_settlement.rs` (`settle_before_the_stage_proof`);
  `worktree_copying/worktree_copy_unreadable_records.rs` (`settle_records_without_a_claim`).
  `worktree_copying/worktree_copy_stage_claim_swept_files.rs` uses `worktree_copying/worktree_copy_journal_removal.rs`
  (`remove_file_and_say`);
  `worktree_copying/worktree_copy_stage_claim_reaping.rs` (`StageVerdicts`);
  `worktree_copying/worktree_copy_stage_made_object.rs` (`MADE_SUFFIX`).
  `worktree_copying/worktree_copy_stage_made_object.rs` uses `worktree_copying/worktree_copy_journal.rs`
  (`ensure_journal_root`,
  `journal_root`,
  `write_atomic_unsynced`);
  `worktree_copying/worktree_copy_journal_removal.rs` (`remove_file_if_present`);
  `worktree_copying/worktree_copy_stage_snapshot_origin.rs` (`SnapshotOrigin`).
  `worktree_copying/worktree_copy_stage_reaping_proof_and_words.rs` uses
  `worktree_copying/worktree_copy_stage_made_object.rs` (`is_the_recorded_object`,
  `made_record_beside_claim`,
  `read_made`);
  `worktree_copying/worktree_copy_stage_snapshot_origin.rs` (`proved_created_after_the_record`).
  `worktree_copying/worktree_copy_stage_snapshot_origin.rs` uses `worktree_copying/worktree_copy_journal.rs`
  (`journal_root`).
  `worktree_copying/worktree_copy_swapped_add_settlement.rs` uses `worktree_copying/worktree_copy_stage_claim.rs`
  (`DESTINATION_SUFFIX`,
  `StageClaim`);
  `worktree_copying/worktree_copy_stage_made_object.rs` (`MadeObject`,
  `read_made`);
  `worktree_copying/worktree_copy_stage_reaping_proof_and_words.rs` (`still_there`).
  `worktree_copying/worktree_copy_synchronize.rs` uses `worktree_copying/worktree_copy.rs` (`Invocation`,
  `summary_line`);
  `worktree_copying/worktree_copy_journal.rs` (`create_journal`);
  `worktree_copying/worktree_copy_registry.rs` (`CreatedWorktrees`,
  `WorktreeCopyObservation`,
  `command_token`,
  `find_created_worktree`,
  `registered_by_another`);
  `worktree_copying/worktree_copy_source_still_being_filled.rs` (`source_is_still_being_filled`,
  `source_still_being_filled_notice`);
  `worktree_copying/worktree_copy_stage.rs` (`StagedSnapshot`,
  `stage_ignored_snapshot_for`);
  `worktree_copying/worktree_copy_stage_binding.rs` (`verify_stage_binding`);
  `worktree_copying/worktree_copy_stage_claim.rs` (`StageClaim`,
  `claim_stage_for_add`);
  `worktree_copying/worktree_copy_transaction.rs` (`Outcome`,
  `SnapshotSource`,
  `complete_journal`).
  `worktree_copying/worktree_copy_transaction.rs` uses `worktree_copying/worktree_copy_install_log.rs` (`InstallLog`,
  `open_install_log`);
  `worktree_copying/worktree_copy_journal.rs` (`write_journal`);
  `worktree_copying/worktree_copy_journal_removal.rs` (`remove_pending_journal`);
  `worktree_copying/worktree_copy_stage.rs` (`StagedSnapshot`);
  `worktree_copying/worktree_copy_stage_binding.rs` (`stage_binding`);
  `worktree_copying/worktree_copy_unfilled_record.rs` (`remember_unfilled`,
  `forget_unfilled`,
  `remember_unfilled_with`).
  `worktree_copying/worktree_copy_unfilled_record.rs` uses `worktree_copying/worktree_copy_journal.rs` (`Role`,
  `assert_private_path`,
  `journal_root`,
  `write_atomic_unsynced`);
  `worktree_copying/worktree_copy_journal_removal.rs` (`remove_file_if_present`).
  `worktree_copying/worktree_copy_unreadable_record_aside.rs` uses `worktree_copying/worktree_copy_journal.rs`
  (`ensure_journal_root`,
  `ensure_private_directory`,
  `journal_root`).
  `worktree_copying/worktree_copy_unreadable_records.rs` uses
  `worktree_copying/worktree_copy_interrupted_add_settlement.rs` (`settle_interrupted_add`);
  `worktree_copying/worktree_copy_journal.rs` (`JournalFile`,
  `Role`,
  `assert_private_path`,
  `journal_owner_is_dead`,
  `journal_root`,
  `read_listed_journals`);
  `worktree_copying/worktree_copy_journal_removal.rs` (`remove_file_if_present`);
  `worktree_copying/worktree_copy_stage_binding.rs` (`BindingFile`,
  `STAGE_BINDING_NAME`,
  `stage_binding_reading`);
  `worktree_copying/worktree_copy_stage_claim.rs` (`DESTINATION_SUFFIX`,
  `claim_names`,
  `rewrite_claim`,
  `COMPLETE_SUFFIX`);
  `worktree_copying/worktree_copy_stage_claim_reaping.rs` (`ClaimFile`,
  `StageVerdicts`,
  `claim_file_reading`);
  `worktree_copying/worktree_copy_stage_made_object.rs` (`read_made`);
  `worktree_copying/worktree_copy_unfilled_record.rs` (`UnfilledFile`,
  `unfilled_file_reading`,
  `unfilled_names`);
  `worktree_copying/worktree_copy_unreadable_record_aside.rs` (`move_aside`,
  `UNREADABLE_FOLDER`,
  `moved_record_path`,
  `unreadable_record_names`,
  `unreadable_records_stand`).
  Break:
  split by role into
  `worktree_copy_journal_record` (the journal and its encoding),
  `the_stage_a_copy_fills` (the stage,
  its claim and its records),
  `worktree_copy_owed_work` (what a killed copy still owes) and `worktree_copy_git_steps`,
  which the removal plan already named.

The folder loop is one strong set of 57 folders over 564 uses,
and it is not only the crate root:
take the root out of the same edges and one loop of 51 folders over 510 uses is left,
so the folders genuinely use each other both ways today.
The root's 53 flat re-exports add the other six folders and 54 uses,
and step 6 takes those away with the re-exports.
What remains is the set the lists tolerate,
and a pair leaves it when the uses the loop breaks above take away are gone.

### The call changes, as signatures

The other agent on `refactor/git-policy-cli-fewer-closure-suppressions` is removing closure-taking functions.
So none of the changes below hands a closure;
each hands a value or an enum,
and a closure parameter that exists today
turns into an enum there too.
Drift in signatures from that branch is expected and harmless to the layout.

- Run this next (replaces the two in-process recursions `entry::plan_invocation` from `stacked_merge_push` and
  `entry::run_process` from `merge_second_pass_finish`).
  New file `what_a_command_hands_back` in the program folder `starting_git_and_describing_a_run`
  defines `NextCommand { arguments, environment }` and `Finished { code, stdout, stderr }`.
  - `gated_merge_push` returns `Result<NextCommand, SingleFlightError>` (the marked arguments and environment it builds
    today) and no longer calls `plan_invocation`,
    `run_planned`,
    `run_stacked_push` or `status_of_forwarded`.
  - The stacking loop in `push_stacking` becomes a step function:
    `StackingStep::{Done(i32), RunThis { command:
    NextCommand, resume: StackingState }}`,
    with the loop's state carried by value in `StackingState`.
  - `merge_second_pass_finish::run_nested` becomes `FinishStep::{Done(i32), RunThis(NextCommand)}`;
    the nesting counter
    `NESTED_FINISHES` moves to the loop.
  - Consumer:
    the top-level loop in the program crate (`entry_run_forwarded_actions`) runs the command with
    `entry::run_process`'s planning,
    feeds the status back to the step function,
    and repeats until `Done`.
    A depth guard replaces the counter.
- Current step record:
  `transaction_git` stops calling `service_busy::enter_step` and `on_landing_thread`.
  A thread-local record (step file path and detail) moves to the new program file `current_step_record`
  in the folder `how_long_the_program_waits_for_git_steps`
  (it names Git steps,
  so it does not go to a library);
  the runner writes it,
  the landing code
  registers its file path there,
  and `LandingBoard` stays in `service_busy`.
  The two functions listed as removed in the
  group map are `service_busy_step_record_on_this_thread::enter_step` and `::on_landing_thread`.
- After-stop action:
  `ChildWatch::start_with_locks(waiting_for, pid, terms, locks)` and `watch_loop(... locks)` lose the
  `locks: LockLedger` parameter,
  and the line in `child_watch` that calls `LockLedger::before_start(command)` moves out.
  The watcher returns the `StopReport` it already has;
  the caller in `bounded_push` and `transaction_git` builds the
  `LockLedger` before the start and sweeps after the stop.
  The lock folder is in the program,
  above the watcher library,
  so the watcher cannot
  name it.
- System settings path:
  `ScopeEnvironment` in `repo_read_config_scan` gains the field `system_config: Option<PathBuf>`
  and stops calling `git_system_config::system_config_for`;
  the callers that build a `ScopeEnvironment` pass the result.
- Commands that only finish:
  `subvolume_command`,
  `doctor`,
  `worktree_convert_command` and `stacked_merge_push` return
  `Finished` and only the program crate's `entry` and `management` files build `Action`.

Two more call changes were found by the moved-item check and are new:

- `copy_file_exclusive` (moving to `private_storage`) returns `std::io::Error`;
  the copy stage wraps it in
  `WorktreeCopyError`.
  Today it returns `WorktreeCopyError`,
  a type of the worktree crate.
- `render_configuration_warning` and `render_engine_failure` (moving to `invocation_config`) call two hint functions
  of `event_hints`;
  `engine_failure_hint` and `push_hint_field` move with them.
  Compile check expected.

### Changes so that a library stands alone

The loop checks of this section hold only with all of these applied too.
They exist because a surviving library used files that go into the program
(18 uses in 11 files before them,
0 after;
section "The crate split").
Each is a commit that keeps the single crate building and its tests passing (see "Move order").
None hands a closure.
The watcher's cleanliness also depends on one change of the draft:
the "After-stop action" below removes `LockLedger` from `child_watch` and its clock file
(the group map's `items` mark `stopped_git_locks::LockLedger` as `(removed)`),
which is why the engine counts no use of the lock file by the watcher.

- Plain terms for the watcher.
  `WatchTerms::for_class(ChildClass)` and `grace_for(class, ...)` leave `child_watch_clock_marks_and_stopping`;
  the watcher takes a quiet window,
  a notice grace,
  the quiet window of a connected transfer and the name of the variable that changes the window,
  and the program's `child_class` builds them from its step classes.
  `child_watch` stops naming `ChildClass`,
  `CHILD_WAIT_VARIABLE` and `OBJECT_STEP_CEILING`;
  its callers pass the ceiling and the subject text.
- Items that move down into the libraries:
  - `ChildOutcome` to `how_a_child_program_ended` in `starting-and-watching-programs` (from `starting_git_with_the_callers_arguments` in the program folder `starting_git_and_describing_a_run` (old file `forwarding`)).
  - `outcome_of` to `how_a_child_program_ended` in `starting-and-watching-programs` (from `starting_git_with_the_callers_arguments` in the program folder `starting_git_and_describing_a_run` (old file `forwarding`)).
  - `STOP_GRACE` to `watch_clock_marks_and_stopping` in `starting-and-watching-programs` (from `pushing_until_it_ends_or_stalls` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `bounded_push`) or `longest_waits` in the program folder `how_long_the_program_waits_for_git_steps` (old file `wait_ceilings`)).
  - `Patience` to `measuring_progress_of_a_program_and_what_it_started` in `starting-and-watching-programs` (from `pushing_until_it_ends_or_stalls` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `bounded_push`) or `how_long_a_push_may_show_nothing` in the program folder `how_long_the_program_waits_for_git_steps` (old file `push_deadline_and_patience_settings`)).
  - `positive_seconds` to `wait_notice_clock` in `starting-and-watching-programs` (from `pushing_until_it_ends_or_stalls` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `bounded_push`) or `how_long_a_push_may_show_nothing` in the program folder `how_long_the_program_waits_for_git_steps` (old file `push_deadline_and_patience_settings`)).
  - `REPAIR_TOOL_CEILING` to `package_repair_request` in `fix-packages-after-copying-a-folder` (from `longest_waits` in the program folder `how_long_the_program_waits_for_git_steps` (old file `wait_ceilings`)).
  - `environment_value` to `private_scratch_places` in `safe-file-saving` (from `environment_variables_for_git_children` in the program folder `environment_variables_for_git_children` (old file `child_environment`)).
- Names of the program,
  its variables and its Git commands passed in.
  Every literal in a library file that names the program (`cli-git`),
  a `CLI_GIT_` variable,
  Git,
  LFS or a worktree becomes a value the caller passes in,
  or is reworded so it names the job;
  the constants move to the program.
  The checklist is a search of the eight libraries' files at `f44d921e24` outside comments:
  it finds 45 lines in 18 files
  (`scratch_location`,
  `diagnostic_log`,
  `child_watch`,
  `child_watch_clock_marks_and_stopping`,
  `child_watch_step_bounds_and_errors`,
  `wait_progress`,
  `output_capture`,
  `thread_quiet`,
  `index_lock_holders`,
  `worktree_trash_remove`,
  and eight `package_repair` files),
  and the same search must find nothing after each lift.
  Three public names are renamed:
  `repair_worktree` (to `repair_copied_folder`),
  `PolicyIncomplete` of `dependent_version_content`
  and `SERVICE_SCAN_BUDGET` of `index_lock_holders`.
- The repair is given the manifest folders.
  `RepairRequest` loses its Git context;
  it carries the list of folders that hold a `Cargo.toml`,
  which the program builds by running `git ls-files` before it calls
  (edges dropped:
  `package_repair_cargo_workspaces` to `transaction_git` and its request types;
  `package_repair_request` to the same types).
- The repair reports its step times.
  `package_repair` and `package_repair_cargo` stop calling `add_timing::work`;
  the report carries the durations and the program records them
  (edge dropped:
  `package_repair` and `package_repair_cargo` to `add_timing`;
  edge added:
  `worktree_copy` to `add_timing` and to `transaction_git`).
- Policy-bound version files stay out.
  `dependent_version_lifecycle`,
  `dependent_version_policy` and `policy_dependent_version` are program files;
  the eleven others take no program type.
- The test helpers split.
  The Git-free half of `test_support` and its neighbours go to `disposable-test-folders`;
  the Git half stays with the program's tests.

### Plain data types that three folders share

`CopyFacts` (with `AddPlan` and `RemovePlan`,
which are plain structs) moves to the new file `worktree_command_facts`
in the program folder `starting_git_and_describing_a_run`,
because `PushForward` and `CoordinatedForward`,
which the planning doc already moves down,
hold it;
leaving it in `worktree_copy_forward` would make the lock and push folders use the worktree folder.
The planning functions stay where they are.

### Moves of single items

The planning doc's moves,
re-keyed to the files that define the items today,
and the moves above,
from the group map
(`items`).
Each line names the item,
where it is defined today,
and its new file:

- `ChildOutcome`:
  from `starting_git_with_the_callers_arguments` in the program folder `starting_git_and_describing_a_run` (old file `forwarding`) to `how_a_child_program_ended` in `starting-and-watching-programs`
- `outcome_of`:
  from `starting_git_with_the_callers_arguments` in the program folder `starting_git_and_describing_a_run` (old file `forwarding`) to `how_a_child_program_ended` in `starting-and-watching-programs`
- `subcommand_of`:
  from `which_git_steps_run_again_or_end_early` in the program folder `running_git_for_a_transaction` (old file `transaction_git_step_rerun_and_end_rules`) to `kinds_of_git_step` in the program folder `how_long_the_program_waits_for_git_steps`
- `shell_quote`:
  from `words_about_reference_locks` in the program folder `clearing_leftover_reference_locks` (old file `stale_ref_lock_message`) to `process_numbers_in_other_containers` in the program folder `who_owns_a_transaction_folder`
- `current_boot_id`:
  from `claim_time` in the program folder `work_left_for_the_service_to_finish_later` (old file `claim_time`) to `telling_one_process_from_a_reused_number` in `starting-and-watching-programs`
- `current_machine_id`:
  from `claim_time` in the program folder `work_left_for_the_service_to_finish_later` (old file `claim_time`) to `telling_one_process_from_a_reused_number` in `starting-and-watching-programs`
- `is_machine_id`:
  from `claim_time` in the program folder `work_left_for_the_service_to_finish_later` (old file `claim_time`) to `telling_one_process_from_a_reused_number` in `starting-and-watching-programs`
- `SERVICE_UNIT_NAME`:
  from `service_provision` in the program folder `asking_the_commit_service_to_save_a_commit` (old file `service_provision`) to `real_git_the_service_was_set_up_with` in the program folder `finding_the_real_git_program`
- `runtime_unit_dir`:
  from `service_provision` in the program folder `asking_the_commit_service_to_save_a_commit` (old file `service_provision`) to `real_git_the_service_was_set_up_with` in the program folder `finding_the_real_git_program`
- `remember_own_executable`:
  from `service_route` in the program folder `asking_the_commit_service_to_save_a_commit` (old file `service_route`) to `finding_the_real_git` in the program folder `finding_the_real_git_program`
- `Limits`:
  from `binary_supervisor` (old file `binary_supervisor`) to `discarded_copies_settings` in the program folder `the_settings_file_cli_git_config_jsonc`
- `append_line`:
  from `discard_audit` in the program folder `saving_copies_of_what_a_command_would_throw_away` (old file `discard_audit`) to `debug_and_warning_lines_on_standard_error` in `text-time-and-log-line-formats`
- `effective_directory`:
  from `commit_command` in the program folder `the_commit_command` (old file `commit_command`) to `which_worktree_a_command_targets` in the program folder `the_words_meant_for_this_wrapper`
- `SequencerFacts`:
  from `is_a_merge_or_rebase_in_progress` in the program folder `rules_about_what_a_commit_may_contain` (old file `rule_commit_sequencer`) to `answer_shapes_built_from_a_repository_layout` in the program folder `what_the_wrapper_asks_git_about_a_repository`
- `SequencerState`:
  from `is_a_merge_or_rebase_in_progress` in the program folder `rules_about_what_a_commit_may_contain` (old file `rule_commit_sequencer`) to `answer_shapes_built_from_a_repository_layout` in the program folder `what_the_wrapper_asks_git_about_a_repository`
- `sequencer_state`:
  from `is_a_merge_or_rebase_in_progress` in the program folder `rules_about_what_a_commit_may_contain` (old file `rule_commit_sequencer`) to `answer_shapes_built_from_a_repository_layout` in the program folder `what_the_wrapper_asks_git_about_a_repository`
- `capture_stamp`:
  from `service_replay` in the program folder `the_commit_command` (old file `service_replay`) to `claim_time` in the program folder `work_left_for_the_service_to_finish_later`
- `LANDING_ATTEMPTS`:
  from `service_land` in the program folder `landing_commits` (old file `service_land`) to `service_protocol` in the program folder `messages_between_the_wrapper_and_the_commit_service`
- `Bind`:
  from `service_server_socket_binding` in the program folder `the_service_program` (old file `service_server_socket_binding`) to `service_protocol` in the program folder `messages_between_the_wrapper_and_the_commit_service`
- `work_directory`:
  from `service_work_handlers_replanning_and_acceptance` in the program folder `after_commit_work_of_the_service` (old file `service_work_handlers_replanning_and_acceptance`) to `work_item` in the program folder `work_left_for_the_service_to_finish_later`
- `ConversionEnvironment`:
  from `worktree_clean_content_attribute_sources` in the program folder `is_the_worktree_clean` (old file `worktree_clean_content_attribute_sources`) to `subvolume_platform` in the program folder `converting_the_main_checkout`
- `SystemAttributes`:
  from `worktree_clean_content_attribute_sources` in the program folder `is_the_worktree_clean` (old file `worktree_clean_content_attribute_sources`) to `subvolume_platform` in the program folder `converting_the_main_checkout`
- `CLAIM_PREFIX`:
  from `worktree_convert_journal` in the program folder `converting_a_linked_worktree` (old file `worktree_convert_journal`) to `subvolume_preflight` in the program folder `converting_the_main_checkout`
- `enabled`:
  from `trying_in_process_answers_first` in the program folder `answering_questions_without_starting_git` (old file `repo_read_hook`) to `repository_location_without_git` in the program folder `finding_the_git_folders`
- `STOP_GRACE`:
  from `pushing_until_it_ends_or_stalls` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `bounded_push`) to `watch_clock_marks_and_stopping` in `starting-and-watching-programs`
- `positive_seconds`:
  from `how_long_a_push_may_show_nothing` in the program folder `how_long_the_program_waits_for_git_steps` (old file `push_deadline_and_patience_settings`) to `wait_notice_clock` in `starting-and-watching-programs`
- `Patience`:
  from `how_long_a_push_may_show_nothing` in the program folder `how_long_the_program_waits_for_git_steps` (old file `push_deadline_and_patience_settings`) to `measuring_progress_of_a_program_and_what_it_started` in `starting-and-watching-programs`
- `StopCause`:
  from `pushing_until_it_ends_or_stalls` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `bounded_push`) to `measuring_progress_of_a_program_and_what_it_started` in `starting-and-watching-programs`
- `RESTART_COMMAND`:
  from `service_messages` in the program folder `asking_the_commit_service_to_save_a_commit` (old file `service_messages`) to `how_to_change_the_push_deadline` in the program folder `running_git_push_until_it_ends_or_stalls`
- `RELEASE`:
  from `service_protocol_messages_and_name_checks` in the program folder `messages_between_the_wrapper_and_the_commit_service` (old file `service_protocol_messages_and_name_checks`) to `where_worktree_add_spent_its_time` in the program folder `recording_where_worktree_add_spent_its_time`
- `SHADOW_NO_HOOKS_NAME`:
  from `shadow_git` in the program folder `the_hidden_second_git_folder` (old file `shadow_git`) to `warning_about_hooks_in_the_hooks_folder` in the program folder `hooks_a_repository_configures`
- `SHADOW_ROOT_NAME`:
  from `shadow_git` in the program folder `the_hidden_second_git_folder` (old file `shadow_git`) to `warning_about_hooks_in_the_hooks_folder` in the program folder `hooks_a_repository_configures`
- `parse_size`:
  from `reading_a_cat_file_batch_reply` in the program folder `reading_the_bytes_a_commit_would_save` (old file `candidate_batch`) to `discarded_copies_settings` in the program folder `the_settings_file_cli_git_config_jsonc`
- `ENGINE_FAILURE_EXIT_CODE`:
  from `the_decision_one_command_ends_in` in the program folder `what_happens_when_you_type_a_command` (old file `action`) to `kinds_of_git_step` in the program folder `how_long_the_program_waits_for_git_steps`
- `WorktreeIdentity`:
  from `asking_git_which_worktree_this_is` in the program folder `what_the_wrapper_asks_git_about_a_repository` (old file `worktree_identity`) to `which_worktree_a_command_targets` in the program folder `the_words_meant_for_this_wrapper`
- `StopReport`:
  from `pushing_until_it_ends_or_stalls` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `bounded_push`) to `measuring_progress_of_a_program_and_what_it_started` in `starting-and-watching-programs`
- `EngineFailureCode`:
  from `messages_as_json_lines` in the program folder `running_policies_and_settling_their_corrections` (old file `diagnostics`) to `settings_of_the_worktree_a_command_runs_in` in the program folder `the_settings_file_cli_git_config_jsonc`
- `LEGACY_CONFIG_IGNORED_CODE`:
  from `messages_as_json_lines` in the program folder `running_policies_and_settling_their_corrections` (old file `diagnostics`) to `settings_of_the_worktree_a_command_runs_in` in the program folder `the_settings_file_cli_git_config_jsonc`
- `render_configuration_warning`:
  from `messages_as_json_lines` in the program folder `running_policies_and_settling_their_corrections` (old file `diagnostics`) to `settings_of_the_worktree_a_command_runs_in` in the program folder `the_settings_file_cli_git_config_jsonc`
- `render_engine_failure`:
  from `messages_as_json_lines` in the program folder `running_policies_and_settling_their_corrections` (old file `diagnostics`) to `settings_of_the_worktree_a_command_runs_in` in the program folder `the_settings_file_cli_git_config_jsonc`
- `start_directory`:
  from `worktree_add_plan` in the program folder `planning_a_worktree_add_and_what_git_made` (old file `worktree_add_plan`) to `clearing_locks_before_fetch_and_push` in the program folder `clearing_leftover_reference_locks`
- `read_plain_file`:
  from `worktree_convert_journal` in the program folder `converting_a_linked_worktree` (old file `worktree_convert_journal`) to `saving_private_files` in `safe-file-saving`
- `copy_file_exclusive`:
  from `worktree_copy_stage_leaf_copying` in the program folder `copying_ignored_files_into_a_stage` (old file `worktree_copy_stage_leaf_copying`) to `saving_private_files` in `safe-file-saving`
- `Verdict`:
  from `service_work_handlers_replanning_and_acceptance` in the program folder `after_commit_work_of_the_service` (old file `service_work_handlers_replanning_and_acceptance`) to `work_item` in the program folder `work_left_for_the_service_to_finish_later`
- `PushForward`:
  from `the_decision_one_command_ends_in` in the program folder `what_happens_when_you_type_a_command` (old file `action`) to `typed_push_that_lost_a_race` in the program folder `working_out_what_a_typed_push_would_send`
- `CoordinatedForward`:
  from `index_writer_forward` in the program folder `what_happens_when_you_type_a_command` (old file `index_writer_forward`) to `waiting_for_any_programs_hold_on_the_index_lock` in the program folder `waiting_for_the_index_lock_of_git`
- `CoordinatedRun`:
  from `index_writer_forward` in the program folder `what_happens_when_you_type_a_command` (old file `index_writer_forward`) to `waiting_for_any_programs_hold_on_the_index_lock` in the program folder `waiting_for_the_index_lock_of_git`
- `ForwardedRun`:
  from `worktree_copy_forward` in the program folder `what_happens_when_you_type_a_command` (old file `worktree_copy_forward`) to `the_git_command_a_forward_runs` in the program folder `running_git_for_a_transaction`
- `run_forwarded_git`:
  from `worktree_copy_forward` in the program folder `what_happens_when_you_type_a_command` (old file `worktree_copy_forward`) to `the_git_command_a_forward_runs` in the program folder `running_git_for_a_transaction`
- `ShippedChecks`:
  from `the_list_of_shipped_checks` in the program folder `the_list_of_checks_every_commit_runs` (old file `policy_checks`) to `one_complete_policy_pass` in the program folder `the_list_of_checks_every_commit_runs`
- `editor_would_open`:
  from `merge_isolation_git_reads` in the program folder `merging_without_touching_your_files` (old file `merge_isolation_git_reads`) to `merge_second_pass_git_runner` in the program folder `rerunning_a_merge_word_by_word`
- `stdin_and_stdout_are_one_terminal`:
  from `merge_isolation_git_reads` in the program folder `merging_without_touching_your_files` (old file `merge_isolation_git_reads`) to `merge_second_pass_git_runner` in the program folder `rerunning_a_merge_word_by_word`
- `Occurrence`:
  from `reading_a_command_line_option_by_option` in the program folder `reading_options_the_way_git_does` (old file `command_options`) to `types_of_option_reading` in the program folder `reading_options_the_way_git_does`
- `OptionError`:
  from `reading_a_command_line_option_by_option` in the program folder `reading_options_the_way_git_does` (old file `command_options`) to `types_of_option_reading` in the program folder `reading_options_the_way_git_does`
- `OptionErrorKind`:
  from `reading_a_command_line_option_by_option` in the program folder `reading_options_the_way_git_does` (old file `command_options`) to `types_of_option_reading` in the program folder `reading_options_the_way_git_does`
- `OptionSpec`:
  from `reading_a_command_line_option_by_option` in the program folder `reading_options_the_way_git_does` (old file `command_options`) to `types_of_option_reading` in the program folder `reading_options_the_way_git_does`
- `OptionValue`:
  from `reading_a_command_line_option_by_option` in the program folder `reading_options_the_way_git_does` (old file `command_options`) to `types_of_option_reading` in the program folder `reading_options_the_way_git_does`
- `ParseMode`:
  from `reading_a_command_line_option_by_option` in the program folder `reading_options_the_way_git_does` (old file `command_options`) to `types_of_option_reading` in the program folder `reading_options_the_way_git_does`
- `ParsedOptions`:
  from `reading_a_command_line_option_by_option` in the program folder `reading_options_the_way_git_does` (old file `command_options`) to `types_of_option_reading` in the program folder `reading_options_the_way_git_does`
- `Arity`:
  from `reading_a_command_line_option_by_option` in the program folder `reading_options_the_way_git_does` (old file `command_options`) to `types_of_option_reading` in the program folder `reading_options_the_way_git_does`
- `GuardedCommand`:
  from `rule_linked_worktree_only` in the program folder `rules_about_where_a_command_may_run` (old file `rule_linked_worktree`) to `rejection_texts_of_the_linked_worktree_rule` in the program folder `rules_about_where_a_command_may_run`
- `SavedCopy`:
  from `conflict_sides` in the program folder `saving_copies_of_what_a_command_would_throw_away` (old file `conflict_sides`) to `conflict_side_notice` in the program folder `saving_copies_of_what_a_command_would_throw_away`
- `IndexPaths`:
  from `service_land_follow` in the program folder `landing_commits` (old file `service_land_follow`) to `index_reset_file` in the program folder `landing_commits`
- `PrivateCopy`:
  from `service_land_copy` in the program folder `landing_commits` (old file `service_land_copy`) to `service_land_copy` in the program folder `landing_commits`
- `RealGitNotFound`:
  from `finding_the_real_git` in the program folder `finding_the_real_git_program` (old file `real_git`) to `deciding_if_a_path_is_real_git` in the program folder `finding_the_real_git_program`
- `ResolutionInputs`:
  from `finding_the_real_git` in the program folder `finding_the_real_git_program` (old file `real_git`) to `deciding_if_a_path_is_real_git` in the program folder `finding_the_real_git_program`
- `SearchedCandidate`:
  from `finding_the_real_git` in the program folder `finding_the_real_git_program` (old file `real_git`) to `deciding_if_a_path_is_real_git` in the program folder `finding_the_real_git_program`
- `search_path_for_real_git`:
  from `finding_the_real_git` in the program folder `finding_the_real_git_program` (old file `real_git`) to `deciding_if_a_path_is_real_git` in the program folder `finding_the_real_git_program`
- `MetadataOutput`:
  from `running_a_read_only_git_query` in the program folder `running_git_for_a_transaction` (old file `git_metadata`) to `query_request_answer_and_context` in the program folder `answering_questions_without_starting_git`
- `LandingLocation`:
  from `where_the_real_index_lives` in the program folder `what_the_wrapper_asks_git_about_a_repository` (old file `landing_location`) to `answer_shapes_built_from_a_repository_layout` in the program folder `what_the_wrapper_asks_git_about_a_repository`
- `marker_of`:
  from `marker_inside_the_held_lock` in the program folder `holding_the_index_lock_for_the_commit_service` (old file `service_index_marker`) to `who_a_lock_holder_is` in the program folder `holding_the_index_lock_for_the_commit_service`
- `Marked`:
  from `marker_inside_the_held_lock` in the program folder `holding_the_index_lock_for_the_commit_service` (old file `service_index_marker`) to `who_a_lock_holder_is` in the program folder `holding_the_index_lock_for_the_commit_service`
- `Announce`:
  from `the_hold_on_the_index_lock` in the program folder `holding_the_index_lock_for_the_commit_service` (old file `service_index_lock`) to `lock_names_and_failures` in the program folder `holding_the_index_lock_for_the_commit_service`
- `LockFailure`:
  from `the_hold_on_the_index_lock` in the program folder `holding_the_index_lock_for_the_commit_service` (old file `service_index_lock`) to `lock_names_and_failures` in the program folder `holding_the_index_lock_for_the_commit_service`
- `lock_path_of`:
  from `the_hold_on_the_index_lock` in the program folder `holding_the_index_lock_for_the_commit_service` (old file `service_index_lock`) to `lock_names_and_failures` in the program folder `holding_the_index_lock_for_the_commit_service`
- `notice_every`:
  from `the_hold_on_the_index_lock` in the program folder `holding_the_index_lock_for_the_commit_service` (old file `service_index_lock`) to `lock_names_and_failures` in the program folder `holding_the_index_lock_for_the_commit_service`
- `storage_failure`:
  from `service_land_follow` in the program folder `landing_commits` (old file `service_land_follow`) to `lock_names_and_failures` in the program folder `holding_the_index_lock_for_the_commit_service`
- `reclaim`:
  from `putting_the_marked_file_in_place_as_the_lock` in the program folder `holding_the_index_lock_for_the_commit_service` (old file `service_index_placement`) to `removing_a_stale_index_lock` in the program folder `holding_the_index_lock_for_the_commit_service`
- `LOCK_SUFFIX`:
  from `locks_a_stopped_git_may_leave` in the program folder `clearing_leftover_reference_locks` (old file `stopped_git_locks`) to `names_of_reference_lock_files` in the program folder `clearing_leftover_reference_locks`
- `PID_LOCK_SUFFIX`:
  from `locks_a_stopped_git_may_leave` in the program folder `clearing_leftover_reference_locks` (old file `stopped_git_locks`) to `names_of_reference_lock_files` in the program folder `clearing_leftover_reference_locks`
- `pid_named_by`:
  from `locks_a_stopped_git_may_leave` in the program folder `clearing_leftover_reference_locks` (old file `stopped_git_locks`) to `names_of_reference_lock_files` in the program folder `clearing_leftover_reference_locks`
- `pid_file_beside`:
  from `finding_reference_lock_files` in the program folder `clearing_leftover_reference_locks` (old file `stale_ref_lock_walk`) to `names_of_reference_lock_files` in the program folder `clearing_leftover_reference_locks`
- `Verdict`:
  from `judging_one_reference_lock` in the program folder `clearing_leftover_reference_locks` (old file `stale_ref_lock_judge`) to `record_of_a_judged_reference_lock` in the program folder `clearing_leftover_reference_locks`
- `Holder`:
  from `programs_holding_a_file_open` in `starting-and-watching-programs` (old file `index_lock_holders`) to `reading_the_output_of_a_holder_scan` in `starting-and-watching-programs`
- `PushAttempt`:
  from `running_one_push_at_a_time` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `auto_push_single_flight`) to `one_push_attempt_and_its_failure` in the program folder `running_git_push_until_it_ends_or_stalls`
- `SingleFlightError`:
  from `running_one_push_at_a_time` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `auto_push_single_flight`) to `one_push_attempt_and_its_failure` in the program folder `running_git_push_until_it_ends_or_stalls`
- `MAX_PUSH_OUTPUT`:
  from `pushing_until_it_ends_or_stalls` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `bounded_push`) to `waiting_for_a_push_to_end` in the program folder `running_git_push_until_it_ends_or_stalls`
- `stop_push`:
  from `pushing_until_it_ends_or_stalls` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `bounded_push`) to `waiting_for_a_push_to_end` in the program folder `running_git_push_until_it_ends_or_stalls`
- `wait_until_done`:
  from `pushing_until_it_ends_or_stalls` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `bounded_push`) to `waiting_for_a_push_to_end` in the program folder `running_git_push_until_it_ends_or_stalls`
- `run_bounded_push_once`:
  from `pushing_until_it_ends_or_stalls` in the program folder `running_git_push_until_it_ends_or_stalls` (old file `bounded_push`) to `waiting_for_a_push_to_end` in the program folder `running_git_push_until_it_ends_or_stalls`
- `fnv_hex`:
  from `record_of_a_failed_backup_push` in the program folder `pushing_what_the_commit_service_saved` (old file `push_failure_notice`) to `file_name_of_one_branch` in the program folder `pushing_what_the_commit_service_saved`
- `PUSH_FAILED_NOTE`:
  from `pushing_a_landed_commit` in the program folder `pushing_what_the_commit_service_saved` (old file `auto_push`) to `record_of_a_failed_backup_push` in the program folder `pushing_what_the_commit_service_saved`
- `CONFLICT_PHRASE`:
  from `retrying_a_stopped_push` in the program folder `pushing_what_the_commit_service_saved` (old file `push_retry`) to `words_and_limits_of_a_reconcile` in the program folder `pushing_what_the_commit_service_saved`
- `trash_location_is_ignored`:
  from `worktree_trash` in the program folder `making_a_worktree_quickly` (old file `worktree_trash`) to `worktree_snapshot_source` in the program folder `making_a_worktree_quickly`
- `TRASH_PARENT`:
  from `worktree_trash` in the program folder `making_a_worktree_quickly` (old file `worktree_trash`) to `worktree_trash_manifest` in the program folder `making_a_worktree_quickly`
- `GitRunner`:
  from `worktree_copy` in the program folder `copying_ignored_files_around_one_git_command` (old file `worktree_copy`) to `worktree_copy_git_steps` in the program folder `copying_ignored_files_into_a_stage`
- `GitStatus`:
  from `worktree_copy` in the program folder `copying_ignored_files_around_one_git_command` (old file `worktree_copy`) to `worktree_copy_git_steps` in the program folder `copying_ignored_files_into_a_stage`
- `GitStep`:
  from `worktree_copy` in the program folder `copying_ignored_files_around_one_git_command` (old file `worktree_copy`) to `worktree_copy_git_steps` in the program folder `copying_ignored_files_into_a_stage`
- `rename_noreplace`:
  from `putting_the_marked_file_in_place_as_the_lock` in the program folder `holding_the_index_lock_for_the_commit_service` (old file `service_index_placement`) to `saving_private_files` in `safe-file-saving`
- `CopyFacts`:
  from `worktree_copy_forward` in the program folder `what_happens_when_you_type_a_command` (old file `worktree_copy_forward`) to `worktree_command_facts` in the program folder `starting_git_and_describing_a_run`
- `AddPlan`:
  from `worktree_add_plan` in the program folder `planning_a_worktree_add_and_what_git_made` (old file `worktree_add_plan`) to `worktree_command_facts` in the program folder `starting_git_and_describing_a_run`
- `RemovePlan`:
  from `worktree_remove_plan` in the program folder `removing_a_linked_worktree` (old file `worktree_remove_plan`) to `worktree_command_facts` in the program folder `starting_git_and_describing_a_run`
- `engine_failure_hint`:
  from `hints_after_a_finding` in the program folder `running_policies_and_settling_their_corrections` (old file `event_hints`) to `settings_of_the_worktree_a_command_runs_in` in the program folder `the_settings_file_cli_git_config_jsonc`
- `push_hint_field`:
  from `hints_after_a_finding` in the program folder `running_policies_and_settling_their_corrections` (old file `event_hints`) to `settings_of_the_worktree_a_command_runs_in` in the program folder `the_settings_file_cli_git_config_jsonc`
- `WaitNotes`:
  from `what_waiting_for_the_lock_found` in the program folder `holding_the_index_lock_for_the_commit_service` (old file `service_index_found`) to `service_timing` in the program folder `log_lines_of_the_commit_service`

The moves that make a library stand alone are listed in "Changes so that a library stands alone"
and replace the destination of the same items here.

### Whole-file moves the Moves of single items section leaves out

- `worktree_add_plan_path_questions` (path and byte questions) moves to `safe-file-saving`:
  `start_directory`,
  which
  moves to the lock notice,
  uses it.
- `service_durable` and `service_fault` stay in the program,
  in the folders of the lock hold and of the test switches.
- `index_lock_holders`,
  `index_lock_holder_parsers` and `stale_ref_lock_nested_numbers` move to `starting-and-watching-programs`;
  `worktree_trash_remove`,
  `subvolume_remove` and `subvolume_remove_walk` move to `linux-file-and-disk-calls`.
- `conflict_side_state`,
  `editor_would_open` and `stdin_and_stdout_are_one_terminal` as in the rescan.
- `service_review_key` moves to the program folder `messages_between_the_wrapper_and_the_commit_service`.

### Removal of the flat re-exports

`src/native/lib.rs` re-exports 28 folders flat with `pub use folder::*;` and 2 test folders with `pub(crate) use`.
All go;
no library `lib.rs` re-exports anything (measured above:
a re-export in the root makes the root use the folder).
Every `use crate::<file>::Item;` names the folder and the file instead:
515 production files hold 3073 lines that name a sibling through `crate::` or `super::`,
and 513 test files hold
2418 more.
Across packages the path is `monochromatic_<job>::<folder>::<file>::Item` for the eight libraries;
inside the program it is `crate::<folder>::<file>::Item`.
The draft counted 1029 items one of its 47 crates used from another;
with 9 packages the count is smaller and is redone when each library is lifted.
In addition,
126 non-index `pub use` lines re-export a name by name;
each is
replaced by the consumers naming the defining file,
because each is a hidden use the tree cannot show.

### Tests that name a higher crate

`scan/testcheck5.ts` lists the test files of the eight libraries whose references reach a module outside their crate.
Forty-four references:
two name a program file,
and the other 42 name test helpers.

- `safe-file-saving` tests name `child_class`:
  `scratch_location`.
  The sidecar gets a local stand-in for the one constant it reads.
- `fix-packages-after-copying-a-folder` tests name `wait_ceilings`:
  `package_repair_report`.
  The constant it reads is now `REPAIR_TOOL_CEILING` in `package_repair_request`,
  so the sidecar names that.
- The 42 references to test helpers go to `disposable-test-folders`
  (the generic helpers of `test_support`,
  the child-process helpers),
  or to a helper file that moves with its subject
  (`dependent_version_test_support`,
  `fs_btrfs_test_support`).
- Three sidecar references of `starting-and-watching-programs` and one of `fix-packages-after-copying-a-folder`
  need a Git repository or Git context;
  those tests move to the program.

Inside the program crate a test may name any folder's files,
because they are one crate.
The draft's thirty crate pairs and 49 references no longer apply.
Nothing in production changes for them.

## Cargo packaging facts that bear on the tree

- The wrapper's `Cargo.toml` today sets `include` by hand (not through `file-enforcer.config.jsonc`,
  whose `cargo-base`
  entry only sets keys such as `edition`,
  `license`,
  `publish` and the lint blocks).
  Cargo ignores `exclude` when `include`
  is set (`doc/research/tests-inside-published-rust-crates.md`).
- Measured 2026-11-04 with cargo 1.101.0-nightly (31d3246e0 2026-10-27) on a scratch crate with three folders and four
  `uses/` links (`src/b/uses/a`,
  `src/c/uses/a`,
  `src/c/uses/b`),
  `cargo package --list --offline --allow-dirty`:
  - `include = ["src/**/*.rs", "/Cargo.toml", "README.md"]` lists the linked files through the links,
    including a nested copy `src/c/uses/b/uses/a/mod.rs`;
  - `include = ["src/**/*.rs", "!src/**/uses/**", "/Cargo.toml", "README.md"]` lists each file once;
  - `exclude = ["**/uses"]` without `include` lists each file once.
- Releases (owner,
  2026-11-07):
  every release of every package goes to pnpr,
  and only `-rc.N` and plain-number releases also go to crates.io.
  So `cargo package` matters for the program and the libraries,
  and the program's `uses/` links would be packed many times over;
  keep `exclude = ["**/uses"]` or the `!src/**/uses/**` pattern in `include`,
  and use `cargo package --list` as the check.
  A `toml-keys` entry in `file-enforcer.config.jsonc` can set it.
  The wrapper's one last release from the unsplit crate happens before the first `uses/` link exists
  (move order,
  step 0).
- Exact pins:
  `doc/decision/rust-dependency-catalog.md`,
  line 57,
  says the sync writes each path dependency's `version`
  as `=<target package.version>`;
  `package/file-enforcer/cli/Cargo.toml` shows the form
  (`monochromatic-file-enforcer = { path = "../core-rust", version = "=0.1.1" }`).
  A version bump at the bottom library therefore edits every dependent `Cargo.toml` and lockfile through that sync;
  the cascade is mechanical and now reaches the program and a few libraries.

## Move order

Every step is one commit or one branch,
builds,
and passes the crate's tests.

Where each step stands on 2026-11-26,
and what today's tree changes about it,
is at "Where each step stands today (2026-11-26)" below.

0.  Step 0:
    release the wrapper's last crates.io version from the unsplit crate (the planning doc requires this before it
    depends on an unpublished crate;
    the owner's release answers of 2026-11-07 are in the section "Cargo packaging facts that bear on the tree").
    No other step runs earlier.
1.  Rescan `main` (open point "Rescan before step 0") and place the files added since `f44d921e24`.
    Done once on 2026-11-07 (section "Rescan of main after the adopted crates");
    tokio steps T6 to T13 are still to land,
    so it runs again before step 7.
2.  Every change in the Changes the tree needs before files can move section that is not a loop break
    also gets its own commit before the first grouping that needs it:
    the `worktree_command_facts` move,
    the `copy_file_exclusive` error type,
    the hint-function moves,
    `rename_noreplace`,
    the removal of the named re-exports and of the test-folder root re-exports.
    Imports are rewritten wherever they
    appear (`use`,
    qualified expressions,
    type paths,
    `super::`),
    not only in `use crate::<file>` lines.
3.  Loop breaks inside the one crate:
    each bullet of "Breaking each loop" is a commit (a type moves,
    imports change).
4.  The call changes,
    one commit each,
    in this order:
    finished results,
    system settings path,
    after-stop action,
    current step record,
    run this next.
    Each keeps `mise run //package/git-policy/cli:test` green.
5.  The seven changes of "Changes so that a library stands alone",
    one commit each.
6.  Remove the flat re-exports:
    first rewrite every `use crate::<file>` to name the current folder
    (`crate::<old_folder>::<file>`),
    then delete the 28 `pub use` lines;
    the build proves the rewrite.
7.  Group the files into the target folders of the same package:
    `src/<folder>/<file>.rs` for the program,
    or `src/<group>/<folder>/<file>.rs` under the owner's one level of groups
    (a one-folder entry keeps `src/<folder>/<file>.rs`),
    and `src/<crate>/<folder>/<file>.rs` for the eight libraries while they still live inside it,
    one commit per former crate,
    bottom first;
    here also the renames,
    the new index files with their top docs,
    the sidecar renames,
    the `test_support` split and the planted-control path updates (the How the tests and scripts follow section).
8.  Add every `uses/` link in one commit,
    then switch the two layout lint rules on for the whole package
    (enabling them folder by folder would leave the rest failing).
    They then guard acyclicity and the links;
    a use of a higher folder is caught only when it closes a loop,
    so the order between the libraries and the program is kept by the per-package `Cargo.toml` after step 10.
    The linter's `LIST_MARKER` one-line form is used in every top doc.
9.  Extract `disposable-test-folders` (every lifted library uses it as a dev-dependency).
10. Lift the libraries,
    one per commit,
    in the order of the dependency graph:
    new `Cargo.toml` with `path` dependencies pinned `=<version>`,
    a README whose first paragraph is the job line,
    a `mise.toml` like a sibling,
    the sidecars with it,
    the `pub(crate)` items that another package names widened.
    After each lift,
    show that the library stands alone (section "The crate split",
    "How standing alone will be shown")
    and tell the owner before the next lift.
    The shared build folder is set only after all are shown.
    Every generated index file declares its folders and files `pub mod` where another crate names them.
11. The program crate is what is left;
    its `Cargo.toml` keeps `[[bin]]`,
    the one `[[test]]` target for the built program,
    the binstall metadata and the release profile,
    and loses the dependencies the libraries took.

Steps 7 and 8 can be done before the owner decides on the libraries,
because they only need the folder table;
the lift step is where the owner's decision bites,
and it can stop after any library.

### Where each step stands today (2026-11-26)

Measured on `aba76005c3` against the steps above,
one line per step and per item a step names.
Nothing in this list moves a file;
it says which branch a step still needs.

- Step 0,
  the last crates.io release from the unsplit crate:
  done.
  `Cargo.toml` says `0.6.1-alpha.46`,
  and crates.io's newest plain-number version for `monochromatic-git-policy-cli` is `0.6.0`,
  published 2026-11-07T13:10:36Z.
  Every version after it is an `-alpha.N` one.
- Step 1,
  the rescan:
  done twice,
  on 2026-11-07 and in the section "Rescan of main after the reformat (2026-11-26)".
  The condition it waited on is gone:
  the tokio steps are in the tree,
  with `tokio`,
  `zbus` 6 and the `hyper` set in `Cargo.toml`,
  so no later rescan waits for them.
- Step 2,
  the changes that are not loop breaks.
  `copy_file_exclusive`'s error type landed as `36affbb3ec`,
  after the red test `6f5a74db22`.
  Not landed:
  the `worktree_command_facts` move (no such file),
  the two hint functions (`policy/event_hints.rs` still defines `engine_failure_hint` and `push_hint_field`),
  `rename_noreplace`'s move to `private_storage`
  (the file is `foundation/rename_that_never_replaces.rs`,
  created by `a2e8d98f2d`,
  and the library it would move to does not exist),
  `copy_file_exclusive`'s own move (it is still in `worktree_copying/worktree_copy_stage_leaf_copying.rs`),
  and the removal of the named re-exports
  (51 `pub use folder::*;` lines in `src/native/lib.rs`,
  two of them `pub(crate)`,
  and 126 named `pub use` lines elsewhere).
- Step 3,
  the loop breaks:
  no loop the 2026-11-07 breaks named has closed,
  and the loops stand at 67 instead of 17.
  Two of the moves they named landed on their own:
  `Patience` in `push/push_deadline_and_patience_settings.rs`,
  and `service_review_key` out of the review folder into `service/`.
  (section "Loops between files,
  and how each is broken (2026-11-26)").
  Four moves came in sideways and changed four loops' shapes without closing any:
  `StopCause` into `push/stop_cause_and_words.rs` (`3f69c6f1cc`),
  `trash_location_is_ignored` into `worktree/worktree_trash_ignore_rule.rs` (`41dd04d1c5`),
  `IndexPaths` into `landing/service_land_follow_index_reset.rs` (`4a11a9c7f1`)
  and `CONFLICT_PHRASE` into `push/push_retry_failure.rs` (`3324130e10`).
- Step 4,
  the five call changes:
  none landed.
  `subvolume_command` and `doctor` still build `Action` themselves,
  `repo_read_config_scan` still calls `git_system_config::system_config_for`,
  the watcher's callers still pass a `LockLedger`,
  `transaction_git` still enters its step through `service_busy_step_record_on_this_thread`,
  and `stacked_merge_push` still plans a whole invocation in process
  (`entry::plan_invocation` at its line 226)
  while `merge_second_pass_finish` still counts nested finishes in a static.
- Step 5,
  the stand-alone changes:
  none landed.
  Every library file that used a program file on 2026-11-07 still does,
  and the rescan measures 72 such uses in 36 files.
- Step 6,
  the flat re-exports:
  still there,
  51 lines.
- Step 7,
  the grouping into target folders:
  not started.
  The crate still holds 62 source folders under `src/native/`,
  and the placement of "Rescan of main after the reformat (2026-11-26)" says where each of their files goes.
- Step 8,
  every `uses/` link in one commit and the rules switched on:
  overtaken,
  and this is what is left of it.
  The rules are on since `44e8ef805e`,
  with the two allow lists regenerated against `main` (`83f5dfd6c8`,
  `d1617807ca`),
  so no round can add a loop or a linkless use.
  The links are no longer one commit either:
  24 folders carry their own `uses/` folder today,
  all of them folders the ZFS route created,
  and the 386 links the lists tolerate are written folder by folder as each folder's loops go.
  Still to do:
  those 386 links,
  the 1318 files no top doc lists,
  the 46 folders with no file list,
  the 53 flat re-exports and the 17 group summaries.
- Step 9,
  `disposable-test-folders`:
  not started;
  no such package exists.
- Step 10,
  the lifts:
  not started.
  No `package/<job>/core-rust` folder exists for any of the six libraries,
  and the crate has not been split.
- Step 11,
  the program crate as what is left:
  nothing to do until step 10 lifts something.

### How the tests and scripts follow

- Sidecars follow their subject and their `#[path]` line;
  no sidecar changes except its file name and its imports.
- The planted-control scripts (`package/git-policy/cli/bin/native-planted-controls-*.ts`),
  the fuzz drivers
  (`package/git-policy/cli.fuzz/bin/planted-*.ts`),
  the mutation tool (`package/mutation-test/cli-cargo-mutants/src/`),
  `package/git-policy/cli/mise.toml` (`--scope-prefix src/native/`,
  and the file list in the mutation task description),
  `monochromatic-lint.config.jsonc` (the `package/git-policy/cli/src/native/**/*.rs` scope near line 131),
  the e2e fixtures
  and `native-test-base.Containerfile` name source paths.
  They are updated in the same commit as the move of each file.
  Production files they name exactly,
  with their new homes:

- `src/native/candidate/candidate_batch.rs` to `package/git-policy/cli/src/reading_the_bytes_a_commit_would_save/reading_a_cat_file_batch_reply.rs`
- `src/native/candidate/candidate_stage.rs` to `package/git-policy/cli/src/reading_the_bytes_a_commit_would_save/reading_stage_records.rs`
- `src/native/command/command_add.rs` to `package/git-policy/cli/src/what_each_git_command_accepts/command_add.rs`
- `src/native/commit/capture_invocation.rs` to `package/git-policy/cli/src/the_hidden_second_git_folder/capture_invocation.rs`
- `src/native/commit/commit_args.rs` to `package/git-policy/cli/src/the_commit_command/commit_args.rs`
- `src/native/commit/commit_index.rs` to `package/git-policy/cli/src/the_commit_command/commit_index.rs`
- `src/native/commit/commit_preparation.rs` to `package/git-policy/cli/src/the_commit_command/commit_preparation.rs`
- `src/native/commit/commit_transaction.rs` to `package/git-policy/cli/src/the_commit_command/commit_transaction.rs`
- `src/native/commit/conclusion_copy.rs` to `package/git-policy/cli/src/the_hidden_second_git_folder/conclusion_copy.rs`
- `src/native/commit/post_commit.rs` to `package/git-policy/cli/src/the_commit_command/post_commit.rs`
- `src/native/commit/private_index_stage.rs` to `package/git-policy/cli/src/the_commit_command/private_index_stage.rs`
- `src/native/config/config_file.rs` to `package/git-policy/cli/src/the_settings_file_cli_git_config_jsonc/finding_and_reading_the_settings_file.rs`
- `src/native/config/config_loading.rs` to `package/git-policy/cli/src/the_settings_file_cli_git_config_jsonc/deciding_how_to_load_settings.rs`
- `src/native/config/config_parse.rs` to `package/git-policy/cli/src/the_settings_file_cli_git_config_jsonc/turning_settings_text_into_typed_settings.rs`
- `src/native/config/config_rules_file.rs` to `package/git-policy/cli/src/the_settings_file_cli_git_config_jsonc/the_rules_file_option.rs`
- `src/native/dependent_version/dependent_version_imports.rs` to `package/dependent-version-bump-check/core-rust/src/dependent_version_imports.rs`
- `src/native/dependent_version/dependent_version_text.rs` to `package/dependent-version-bump-check/core-rust/src/dependent_version_text.rs`
- `src/native/health/external_hooks.rs` to `package/git-policy/cli/src/hooks_a_repository_configures/hooks_set_in_git_settings.rs`
- `src/native/invocation/entry.rs` to `package/git-policy/cli/src/what_happens_when_you_type_a_command/deciding_what_one_git_command_does.rs`
- `src/native/invocation/git_version.rs` to `package/git-policy/cli/src/running_git_for_a_transaction/checking_the_real_git_version.rs`
- `src/native/invocation/global_arguments.rs` to `package/git-policy/cli/src/what_each_git_command_accepts/global_arguments.rs`
- `src/native/invocation/hooks_off.rs` to `package/git-policy/cli/src/starting_git_and_describing_a_run/starting_git_with_hooks_off.rs`
- `src/native/invocation/real_git_candidate.rs` to `package/git-policy/cli/src/finding_the_real_git_program/deciding_if_a_path_is_real_git.rs`
- `src/native/invocation/startup_gate.rs` to `package/git-policy/cli/src/what_happens_when_you_type_a_command/first_step_of_a_command.rs`
- `src/native/invocation/wrapped_command.rs` to `package/git-policy/cli/src/what_happens_when_you_type_a_command/deciding_what_a_wrapped_command_does.rs`
- `src/native/invocation/wrapper_controls.rs` to `package/git-policy/cli/src/the_words_meant_for_this_wrapper/control_words_for_the_wrapper.rs`
- `src/native/invocation/wrapper_invocation.rs` to `package/git-policy/cli/src/the_words_meant_for_this_wrapper/removing_control_words_from_a_command.rs`
- `src/native/landing/maintenance_decision.rs` to `package/git-policy/cli/src/landing_commits/maintenance_decision.rs`
- `src/native/landing/maintenance_run.rs` to `package/git-policy/cli/src/landing_commits/maintenance_run.rs`
- `src/native/landing/service_claim_watch.rs` to `package/git-policy/cli/src/after_commit_work_of_the_service/service_claim_watch.rs`
- `src/native/landing/service_replay.rs` to `package/git-policy/cli/src/the_commit_command/service_replay.rs`
- `src/native/landing/service_work.rs` to `package/git-policy/cli/src/after_commit_work_of_the_service/service_work.rs`
- `src/native/lock/index_lock_evidence.rs` to `package/git-policy/cli/src/finding_who_holds_a_lock_file/evidence_about_a_foreign_index_lock.rs`
- `src/native/lock/single_owner_claim.rs` to `package/git-policy/cli/src/converting_the_main_checkout/single_owner_claim.rs`
- `src/native/markdown/markdown_linter_config.rs` to `package/git-policy/cli/src/fixing_markdown_in_process/markdown_linter_config.rs`
- `src/native/markdown/markdown_linter_output.rs` to `package/git-policy/cli/src/fixing_markdown_in_process/markdown_linter_output.rs`
- `src/native/outer/inner.rs` (not a production module of the scan:
  test support or a path in a test)
- `src/native/policy/event_path.rs` to `package/git-policy/cli/src/running_policies_and_settling_their_corrections/repository_path_in_an_event.rs`
- `src/native/policy/policy_final_newline.rs` to `package/git-policy/cli/src/the_built_in_policies/policy_final_newline.rs`
- `src/native/policy/policy_registry.rs` to `package/git-policy/cli/src/the_words_meant_for_this_wrapper/names_of_the_built_in_policies.rs`
- `src/native/process/process_identity.rs` to `package/starting-and-watching-programs/core-rust/src/which_programs_are_running_and_since_when/telling_one_process_from_a_reused_number.rs`
- `src/native/process/process_start.rs` to `package/starting-and-watching-programs/core-rust/src/which_programs_are_running_and_since_when/when_a_process_started.rs`
- `src/native/process/signal_relay.rs` to `package/starting-and-watching-programs/core-rust/src/starting_programs_catching_signals_and_keeping_their_output/passing_signals_to_the_child.rs`
- `src/native/push/auto_push_reconcile.rs` to `package/git-policy/cli/src/pushing_what_the_commit_service_saved/publishing_after_the_remote_moved.rs`
- `src/native/push/auto_push.rs` to `package/git-policy/cli/src/pushing_what_the_commit_service_saved/pushing_a_landed_commit.rs`
- `src/native/push/manual_push_probe.rs` to `package/git-policy/cli/src/working_out_what_a_typed_push_would_send/what_a_dry_run_push_would_send.rs`
- `src/native/push/manual_push_range.rs` to `package/git-policy/cli/src/working_out_what_a_typed_push_would_send/commits_a_push_newly_publishes.rs`
- `src/native/push/push_failure_notice.rs` to `package/git-policy/cli/src/pushing_what_the_commit_service_saved/record_of_a_failed_backup_push.rs`
- `src/native/push/wait_progress.rs` to `package/starting-and-watching-programs/core-rust/src/watching_a_started_program_until_it_stops_making_progress/wait_notice_clock.rs`
- `src/native/repo_read/object_abbreviation.rs` to `package/git-policy/cli/src/reading_saved_commits_and_files/short_object_names.rs`
- `src/native/repo_read/repo_read_layout.rs` to `package/git-policy/cli/src/finding_the_git_folders/repository_location_without_git.rs`
- `src/native/service/service_fault.rs` to `package/git-policy/cli/src/stopping_the_program_at_a_named_step_for_tests/fault_points_the_service_can_be_told_to_hit.rs`
- `src/native/service/service_instance.rs` to `package/git-policy/cli/src/naming_the_service_of_one_worktree/service_instance.rs`
- `src/native/service/service_main.rs` to `package/git-policy/cli/src/the_service_program/the_service_as_a_mode_of_the_program.rs`
- `src/native/service/service_server.rs` to `package/git-policy/cli/src/the_service_program/service_server.rs`
- `src/native/shadow/replay_patch.rs` to `package/git-policy/cli/src/replaying_changes_onto_new_commits/replay_patch.rs`
- `src/native/shadow/replay_reverse_apply.rs` to `package/git-policy/cli/src/replaying_changes_onto_new_commits/replay_reverse_apply.rs`
- `src/native/shadow/replay_subsumption.rs` to `package/git-policy/cli/src/replaying_changes_onto_new_commits/replay_subsumption.rs`
- `src/native/shadow/shadow_config.rs` to `package/git-policy/cli/src/the_hidden_second_git_folder/shadow_config.rs`
- `src/native/shadow/shadow_links.rs` to `package/git-policy/cli/src/the_hidden_second_git_folder/shadow_links.rs`
- `src/native/shadow/shadow_refs.rs` to `package/git-policy/cli/src/the_hidden_second_git_folder/shadow_refs.rs`
- `src/native/transaction/transaction_phase.rs` to `package/git-policy/cli/src/stopping_the_program_at_a_named_step_for_tests/phase_markers_for_crash_tests.rs`
- `src/native/worktree_copying/worktree_copy_install_log.rs` to `package/git-policy/cli/src/copying_ignored_files_into_a_stage/worktree_copy_install_log.rs`
- `src/native/worktree_copying/worktree_copy_install.rs` to `package/git-policy/cli/src/copying_ignored_files_into_a_stage/worktree_copy_install.rs`
- `src/native/worktree_copying/worktree_copy_journal.rs` to `package/git-policy/cli/src/copying_ignored_files_into_a_stage/worktree_copy_journal.rs`
- `src/native/worktree_copying/worktree_copy_recovery.rs` to `package/git-policy/cli/src/copying_ignored_files_into_a_stage/worktree_copy_recovery.rs`
- `src/native/worktree_copying/worktree_copy_rollback.rs` to `package/git-policy/cli/src/copying_ignored_files_into_a_stage/worktree_copy_rollback.rs`
- `src/native/worktree_copying/worktree_copy.rs` to `package/git-policy/cli/src/copying_ignored_files_around_one_git_command/worktree_copy.rs`
- `src/native/worktree/worktree_convert_journal.rs` to `package/git-policy/cli/src/converting_a_linked_worktree/worktree_convert_journal.rs`

- After each planted-control path update,
  `bin/native-planted-controls-proof.ts` is the check that the mutated line still
  exists;
  a path that no longer exists must fail there,
  not pass silently.
- Documents that name `src/native` paths (`README.md`,
  `SPEC.md`,
  `DESIGN.md`,
  `doc/` entries in the package) are updated
  once at the end of step 5;
  they are prose,
  not tooling.

### Folders added after the scan

The repository move to the 860 drive added code the scan did not see
(`doc/planning/repository-on-the-860-drive.md`,
item W2).
It moves with the layout like every other folder,
in the same commits as the files it sits beside.

- The folder `src/native/repository_link_check/` moves as one unit
  (the identity file reader,
  the answers and messages,
  the check with its deadline,
  the probe,
  the paths an invocation names,
  the gate,
  and the shared JSON table of fixture states).
  It has no `uses/` folder yet;
  the layout move gives it one with the others.
  Its group is the program crate's startup step beside `startup_gate`,
  which calls it.
- The shared table `repository_link_shared_table.json` is read by relative path from the TypeScript suite
  `package/repository-link/check/src/check-link.unit.test.ts`
  (and by `include_str!` from the Rust suite);
  the commit that moves the folder updates that path,
  and a test of that suite fails when the path stops resolving.
- `src/native/subvolume/fs_uuid_of_source.rs` (the filesystem UUID of a mount's source device)
  moves with `fs_mount_options.rs` to `linux-file-and-disk-calls`.

## Open points for the owner

Each has options with pros and cons and a ranking;
the recommendation is the first option.

- Which Git-free jobs become libraries.
  The eight pass the coordinator's stand-alone test (section "The crate split");
  four have a sibling that carries its own copy today (tier one),
  four only a sibling whose job would call them (tier two).
  Options:
  - A,
    the eight and the test-folders library.
    Pros:
    every Git-free job with a plausible sibling is a library,
    which is what "crates obviously have to be reusable" asks for.
    Cons:
    four of them have no sibling that asks;
    needs all seven named changes,
    which are code changes beyond the four the owner approved.
  - B,
    only the four with tier-one evidence
    (`safe-file-saving`,
    `linux-file-and-disk-calls`,
    `starting-and-watching-programs`,
    `parallel-work-within-a-budget`) and the test-folders library.
    Pros:
    each has a sibling that already does the job;
    fewer named changes
    (the watcher terms,
    the items moving down,
    the program name and the test split;
    not the two repair changes nor the version files).
    Cons:
    `text-time-and-log-line-formats` is needed by three of the four,
    so it would have to come along or its few files be copied,
    and the repair and version jobs go into the program although they are Git-free.
  - C,
    A plus the Git-free clusters with no user today
    (every bullet of the list in section "Jobs found buried in the program").
    Pros:
    the program shrinks;
    the owner's "do not put everything in one crate" is read widest.
    Cons:
    each extra library costs a manifest,
    a README,
    licence texts and a place in the pin cascade,
    and none has a sibling that asks.
  Ranking:
  A > B > C.
  A over B because the four tier-two jobs are as Git-free as the others and a sibling plausibly calls each,
  while B's cut line (a sibling already carries a copy) is stricter than anything the owner said;
  B over C because C adds libraries no sibling asks for,
  and any of them lifts later for the price of moving one folder.
  Update 2026-11-07T10:43:12-05:00.
  The owner answered this point with a question,
  and section "The eight libraries against published crates" answers it job by job.
  Options A to C count libraries by the coordinator's tier test,
  which the verdicts change:
  for `safe-file-saving` the tier-one evidence (siblings that write,
    sync and replace files) no longer argues for a crate of ours,
  because the siblings can use the same published calls,
  and `talking-to-the-systemd-user-manager` is a job `zbus` 6 does whole,
  which two siblings already depend on.
  Two readings replace them for the owner to choose between
  (the first three are kept as written for the record):
  - D,
    the libraries that remain after every verdict:
    `starting-and-watching-programs`,
    `parallel-work-within-a-budget`,
    `fix-packages-after-copying-a-folder` and `dependent-version-bump-check`.
    Pros:
    no library does what a published crate does,
    and every Git-free job the owner's tightened rule allows keeps its library.
    Cons:
    19 uses of program files by these libraries need five small changes and two owner decisions
    (how a library prints a log line,
    and where `shell_quote` lives);
    two of the four (`fix-packages-after-copying-a-folder`,
    `dependent-version-bump-check`) still have no sibling that carries a
    copy or asks;
    and the reason for `parallel-work-within-a-budget` shrinks when tokio step T12 removes the thread runner.
  - E,
    only `starting-and-watching-programs` and `parallel-work-within-a-budget`,
    the two with a sibling that carries its own copy of an operation
    (`cargo-shadow`'s `bounded_run.rs` and `control_group.rs` with `pidfd`,
    `forbidden-strings`' `parallel.rs`);
    the repair and version jobs fold into the program.
    Pros:
    every remaining library has a sibling that duplicates a related operation (not a consumer of the library's API),
    and fewer manifests,
    READMEs and licence texts.
    Cons:
    two Git-free jobs of about 2,300 and 2,100 lines go into the program,
    which is a narrow reading of "We should not put everything in one crate.";
    and `parallel-work-within-a-budget` may follow them after step T12.
  Ranking:
  D > E,
  because D keeps both Git-free jobs that the owner's tightened rule lets stand alone,
  while E's cut (a sibling already carries a copy) is stricter than anything the owner said,
  and a fold from D to E later costs one folder move per job.
  The owner has not answered the count ("I'm not ready to answer this specific question yet."),
  so neither reading is taken.
- The test helpers.
  Options:
  - A,
    `disposable-test-folders` as a ninth library,
    the Git-free half of the test helpers.
    Pros:
    one copy,
    and the eight libraries' tests run from their own folders.
    Cons:
    one more package with its own README and licences.
  - B,
    every library keeps its own small helpers.
    Pros:
    no ninth package.
    Cons:
    the same dozen helpers copied eight times that drift apart.
  - C,
    the helpers stay in the program and the libraries' tests use only `std`.
    Pros:
    nothing to split.
    Cons:
    the libraries' tests could not run alone,
    which fails the stand-alone test.
  Ranking:
  A > B > C.
  A over B because a copy per library drifts;
  B over C because C fails the test the owner asked for.
  Facts after the adopted crates,
  2026-11-07T10:43:12-05:00
  (the owner's answer was "Not yet."
    and this decides nothing):
  - `tempfile` is adopted (`Builder::tempdir_in`,
    `TempDir`,
    and `keep` for the keep-fixtures switch),
    but `test_support` (509 lines,
    21 public functions) still makes its folders by hand and uses `tempfile` nowhere,
    so option A's library would hold the process helpers
    (the watchdog,
    the owned process group,
    the core-dump ban,
    the file-size-limit child,
    for which the study found no crate;
    study,
    "Disposable test folders") more than folders.
  - The `testing/` folder holds 1,228 lines in nine non-sidecar files,
    of which the Git half (`recording_git`,
    `smart_http_support`,
    part of `test_support`) stays with the program's tests.
  - Sixteen support files of the `native_binary` test target (about 4,700 lines,
    `binary_*` without `_tests`)
    are new since the first scan.
    They start the built program,
    so they stay with the program's tests under every option.
  - Tokio changed the test side too:
    `no_threads_child_test_support` and the `strace` counter `binary_budget_trace` are new kinds of helper.
  - The ninth library has no production file in the table;
    the scan counted 44 test references from the libraries' sidecars to these helpers.
- One level of groups in the program.
  Answered by the owner on 2026-11-07T02:44:49-05:00,
  verbatim,
  "One level of groups".
  The earlier options were flat folders (ranked first) and one level of groups (the owner's pick).
  What is open is the rule that relates a group folder to its child folders
  and the final names of the groups:
  section "One level of groups in the program" drafts 26 entries and ranks three options for the rule
  (B > A > C).
  The ranking is by value and the recommendation is by order:
  both are reachable,
  in the order A then B,
  because B is A plus a group-level summary.
  Recommendation:
  A now,
  because the installed lint already enforces it and loses nothing if B is added later,
  and B when the owner words it and the linter gains the group check.

- Where the libraries live and what they are named.
  Options:
  - A,
    `package/<job>/core-rust`,
    crate `monochromatic-<job>`,
    as `package/catalog-specifier/core-rust` does.
    Pros:
    nothing Git-related in the path or the name;
    matches the sibling Rust libraries.
    Cons:
    the rule that directory segments are singular (`doc/planning/singular-dir-name-invariant.md`) would call
    `starting-and-watching-programs`,
    `parallel-work-within-a-budget`,
    `fix-packages-after-copying-a-folder` and others plural,
    unless the owner exempts job names.
  - B,
    `package/git-policy/<job>`,
    as the draft.
    Pros:
    the family stays together.
    Cons:
    puts "git-policy" in the path of a library whose job and name must mention nothing related to Git.
  Ranking:
  A > B.
  A because the new rule is about the name and the path is part of it.
  Facts after the adopted crates,
  2026-11-07T10:43:12-05:00
  (the owner's answer was "Not yet."
    and this decides nothing):
  - The number of libraries depends on the open point "Which Git-free jobs become libraries":
    eight as drafted,
    four if the verdicts are taken (reading D),
    two in reading E.
    A library that is dissolved needs no path and no name.
  - The wrapper is already published as `monochromatic-git-policy-cli` 0.6.0 on crates.io
    (API fetched 2026-11-07);
    the path `package/git-policy/cli` and the name did not change.
    Libraries under option A would be `monochromatic-<job>` under `package/<job>/core-rust`,
    as `package/catalog-specifier/core-rust` is,
    and under the owner's rule every release goes to pnpr,
    with `-rc.N` and plain numbers also on crates.io.
  - The directory spellings `core-rust` and `cli` are the repository's precedent,
    not names that pass MKY alone;
    that is for the owner's veto round and is not decided here.
  - Three of the four remaining names are ones the open point "Singular directory names" calls plural
    (`starting-and-watching-programs`,
    `parallel-work-within-a-budget` and `fix-packages-after-copying-a-folder`);
    `dependent-version-bump-check` is not.
  - The dependency graph of the four is:
    `parallel-work-within-a-budget` and `dependent-version-bump-check` use no crate of ours,
    `starting-and-watching-programs` uses `parallel-work-within-a-budget`,
    and `fix-packages-after-copying-a-folder` uses both of the other two.
    The program lists all four.
- Singular directory names.
  The rule that repo-chosen directory segments are singular nouns (`doc/planning/singular-dir-name-invariant.md`)
  would call the library names `starting-and-watching-programs`,
  `parallel-work-within-a-budget`,
  `fix-packages-after-copying-a-folder` and others plural.
  Options:
  - A,
    keep the job names and exempt them.
    Pros:
    the names say what the job does,
    which the monkey test asks for.
    Cons:
    an exemption the owner must write down.
  - B,
    singularize them
    (`starting-and-watching-program`).
    Pros:
    no exemption.
    Cons:
    names that read oddly to someone who knows only `git commit` and `git push`.
  Ranking:
  A > B.
  A because principle 2 is the owner's own and names are read more than rules are;
  B only if the owner will not exempt.
- SHA digests.
  Answered and done:
  the owner picked "Digests:
  sha1 + sha2" (2026-11-07),
  and `subvolume/subvolume_sha.rs` is two calls into `sha1` 0.11.0 and `sha2` 0.11.1 since commit `1aea0ae238`.
  The program folder `sha_1_and_sha_256_of_bytes` is therefore one thin file and can dissolve into its users.

- Rescan before step 0.
  `main` has moved since `f44d921e24` (section "The crate split",
  "Files of `main` added after the scan").
  Options:
  - A,
    rerun the scanners and place the new files first.
    Pros:
    the loop counts and the table describe the code that will move.
    Cons:
    the table must be extended by hand for about eight new files.
  - B,
    move on the old scan and place the new files at lift time.
    Pros:
    nothing to do now.
    Cons:
    loops that the new files add are found late.
  Ranking:
  A > B.
  A because the measured zero loops are of an old tree.
  Done 2026-11-07T10:43:12-05:00:
  section "Rescan of main after the adopted crates" places the files added since `f44d921e24`
  and lists the named changes the rescan found.

Carried over from the previous version,
with the recommendation unchanged unless noted:

- Source-text controls (`source_guard` and its tests) stay in the program crate and read the sibling packages' `src/`
  folders.
  Accept,
  since they police the whole family and the program is the only package that sees all of it.
- `cli` as a package folder name does not pass MKY alone (VGW exempts package folders,
  MKY does not).
  Keep `cli`,
  because the lock row `cargo:monochromatic-git-policy-cli` and the release archive name it,
  and explain it in the README.
- `PKG` says a package is complete only with tests covering every exported code path;
  the sidecars move with their subjects and keep covering them;
  no new tests are written for the move.
- The linter README example of the file list is split over two lines and fails the rule it documents.
  Fix the README to one line;
  it is outside this proposal.
- Two call changes (copy error type,
  hint functions) were found by the moved-item check.
  Accept both.
- The `include` list:
  nothing is published afterwards from the unsplit crate's `uses/` links,
  so drop it at the split.
- The exact-pin cascade now touches the program and a few libraries on a bump;
  rely on the catalog sync and say in the sync's doc that the family needs it.
- Module visibility,
  the scanner's `cfg` scope and surfaces it may not see (`self::`,
  macros,
  trait impls),
  and the manifest migration inventory stay open from the first review.

Withdrawn with the 46-library split:
the places of `FailureCode`,
`service_fault` and `service_durable`,
the crate count,
and the folding of `commit-command`,
`doctor-report`,
`merge-in-isolation`,
`typed-push-planning` and `service-after-commit-work`
(they fold now by the owner's rule).

From lines 45994 to 45997 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The four libraries' shared test helpers:
  **"A separate library (Recommended)"** (`disposable-test-folders`).
- Where the libraries live:
  **"package/<job>/core-rust (Recommended)"**,
  crate `monochromatic-<job>`.
- Their plural names:
  **"Keep the names,
  exempt them (Recommended)"**;
  `doc/planning/singular-dir-name-invariant.md` gets the entries.
  With these three the plan's open points for the owner are closed (`doc/planning/cli-git-source-layout-target-tree.md`,
  "Open points for the owner").

## First review, of the 46-library draft (2026-11-04)

The reviewer was `openai/gpt-6.1-sol` (thinking xhigh),
given the proposal,
the companion and the planning doc,
with no tools.
It read documents only and did not rerun the scanners.
Its findings,
and what changed:

- Missing crate edges (`start-the-real-git` to `git-lock-files` for the lock sweep;
  `cli` to `commit-service-program`):
  correct.
  Both edges were added to the checked graph and stay acyclic.
- The move order left changes unscheduled,
  and extraction order omitted the fixtures crate:
  correct.
  The move order now schedules each non-loop change as its own commit,
  rewrites every path form,
  extracts
  `disposable-test-fixtures` first and says that index files declare `pub mod` where another crate names a module.
- Lint rollout one folder at a time cannot be clean,
  and the layer guarantee was too strong:
  correct.
  The rollout is
  now one commit for all links,
  and the text says the rules catch loops,
  not every layer breach.
- Package names and many retained file names fail MKY alone (`cli`,
  `policy-engine`,
  `typed-command-reader`,
  `inner_git_words`,
  `service_busy`,
  `service_route`,
  `worktree_ladder` and others):
  correct and not yet changed.
  The names are one table (`target` in the group map) and the owner's veto round is the place to settle them;
  the reviewer's replacements are kept in the Open points added by that review list.
- `new-worktrees-quickly` rejection unsupported by the folder graph:
  partly.
  The early three-crate cut that looped
  was the copy,
  ladder and clean-check cut,
  before `worktree_path_failure_message` moved down;
  with that file moved the
  clean-check folder is a leaf.
  The text keeps the crate whole but names it as the first candidate to cut.
- Crates with no outside consumer (`commit-command`,
  `doctor-report`,
  `merge-in-isolation`,
  `service-after-commit-work`,
  `typed-push-planning`,
  `commit-service-program`):
  accepted as a finding.
  They are orchestration;
  folding the first four
  and the push planner into `cli` is cycle-free and is recorded as an open point.
- Other findings kept as open points:
  module visibility,
  the scanner's `cfg` scope and other surfaces it may not see
  (`self::`,
  macros,
  trait impls),
  upward test cases needing concrete plans,
  `durable-work-items` and
  `starting-and-watching-programs` claiming more independence than they have,
  misleading homes for `shell_quote`,
  `append_line`,
  `RELEASE` and `ENGINE_FAILURE_EXIT_CODE`,
  the manifest migration inventory,
  tooling updated twice
  (intermediate and final paths),
  the missing fixtures tree and `main.rs` in the companion.
- Claims withdrawn:
  that single-file crates cannot merge without a cycle (`three-way-text-merge` into its only
  consumer would be cycle-free;
  it stays for reuse),
  and "zero loops" without its scope (the scanner reads `crate::`,
  `super::`,
  grouped imports and named re-exports;
  other surfaces are unchecked).

Open points added by that review,
with its folding advice overtaken by the owner's rule of 2026-11-07 (those crates now fold):

- Fold `commit-command`,
  `doctor-report`,
  `merge-in-isolation` and `typed-push-planning` into `cli`,
  and
  `service-after-commit-work` into `commit-service-program`.
  Recommendation:
  fold them unless the owner names an outside user.
- Rename packages and files that fail MKY alone,
  using the reviewer's examples as a start.
  Recommendation:
  rename in
  step 5,
  where files move anyway.
- Split `new-worktrees-quickly` along `is_the_worktree_clean`.
  Recommendation:
  yes,
  as the first lift after the lower half.

## Second review, of the redone split (2026-11-07)

The reviewer was `openai/gpt-6.1-sol` (thinking xhigh),
given only the owner's words and the section "The crate split" with its stand-alone test,
with no tools.
It started at 2026-11-07T02:30:42-05:00 and answered at 2026-11-07T02:36:50-05:00,
well inside the hour.
It did not rerun the scanners or read the source.
Its findings,
and what was done:

- Two passes (`fix-packages-after-copying-a-folder`,
  `dependent-version-bump-check`) lack evidence of a concrete call by the named sibling:
  taken in part.
  Both are marked candidates pending evidence and are the first to fold;
  they were not folded because the coordinator named the pnpm wrapper.
- The digest library was rejected by a criterion the test does not hold:
  taken.
  The rule "duplicates a published crate the siblings already use" is now named as the coordinator's addition,
  separate from the test.
- The tiers are graded unevenly:
  taken.
  The unit is now stated (an operation the library's first paragraph names),
  and the one case that depends on it (`text-time-and-log-line-formats`) is explained;
  the owner may regrade.
- Folded entries mix "no demonstrated user" with "different design":
  taken.
  Each failing entry says which of three kinds of reason applies,
  and the crash-points entry no longer calls the sibling's design a reason.
- The sentence "as the owner asked" credits the owner with the evidence bar:
  taken.
  It now says the bar is the coordinator's.
- "Fewer crates is not everything in one crate" does not answer "I refuse to believe C means splitting into fewer crates",
  and "rejected by the owner's own words" overstated the earlier sentences:
  taken.
  The tension is stated as the owner's to decide,
  and the layer-crates entry no longer claims an owner rejection.
- Crate names of the draft (`monochromatic-git-policy-<name>`) contradict the Git-free check:
  taken.
  The check is stated against the names of option A and fails by construction for option B.
- `CONNECTED_PUSH_QUIET_WINDOW` carries push tuning into a library,
  and the moved files have no neutral names:
  taken.
  The constant stays in the program and is passed in;
  the engine was rerun (still no use of a program file by a library file,
  still no loops);
  the three moved files' new names are given;
  the public items are listed from the build and read at each lift.
- The name `talking-to-the-systemd-user-manager` fails the "name alone" gate with unexplained jargon:
  taken.
  The test now keeps the owner's exception for jargon explained in the top doc,
  and the README paragraph explains systemd,
  user unit and D-Bus.
- The crate total leaves out the ninth library:
  taken.
  The figure is labelled production crates,
  and the test-folders library's build is shown at its lift.
- "Pass" overstates what a plan on an old scan shows:
  taken.
  The section says "pass the test on paper,
  against the scan".
- `sha2` does not replace SHA-1:
  taken and corrected;
  a SHA-1 crate is named too.
- The fixture library would list itself as a dev-dependency,
  and `cargo tree` shows transitive dependencies:
  taken,
  both reworded.
- Nothing found in the nine README opening paragraphs that mentions Git or the wrapper
  (the reviewer's only finding of that kind was the naming conflict above).
  Not acted on:
  its remark that no folded crate is shown to pass every condition,
  since none is claimed to.

## Third review, of the rescan, the library verdicts and the groups (2026-11-07)

The reviewer was `openai/gpt-6.1-sol` (thinking xhigh),
given the owner's words,
the raw command output lines the sections cite,
and the changed sections whole (the rescan,
the library verdicts,
the groups and the open points),
with no tools.
It started at 2026-11-07T10:44:48-05:00 and answered at 2026-11-07T10:58:57-05:00,
well inside the hour.
It could not read the source or the scanners.
It reviewed the version of 2026-11-07T10:43:12-05:00
(the first commit of the branch plus the fixes of the second);
its input is `~/temp/agent/wrapper-layout-after-crates/review/input.md`.
Its twenty-one findings,
and what was done:

- The count of new modules does not match the list (medium):
  taken.
  The sections said twenty;
  the unmatched list minus the support and root files is twenty-two,
  and that was already corrected before the answer.
  The text now says twenty-two and notes that `runtime_probe` is a test-only entry.
- The 292 lint findings are not 292 folders (low):
  taken.
  The wording says findings.
- The four-library graph is stated as settled although the measured model has a loop and 19 edges (high):
  taken.
  The graph is labelled the conditional target graph of reading D,
  with the measured loop and the 19 edges attached,
  and the text says that naming `tracing` or `log` approves nothing.
- Folding the runtime after step T12 would make the process library use program files (high):
  taken.
  The text now says the caller must hand a runtime handle or the runtime component must stay independent,
  and that the graph is rescanned before the fold is viable.
- `std::fs::copy` does not copy exclusively (high):
  taken.
  The text names a create-new copy on `tokio::fs` or `tempfile` with the owner-kept fallback,
  says `std::fs::copy` overwrites,
  and does not count the edge as solved.
- "Thin remainder" is not established for two libraries (medium):
  taken.
  The outcome wording no longer claims thinness for the large remainders,
  the line counts are given,
  and the 468 lines of `json_record` and `js_text` are named as waiting for a comparison of their readers.
- The version-check verdict assigns an outcome the text says is unknown (medium):
  taken.
  Its coverage is undetermined and the library stays provisionally.
- Optional `async-io` does not show the pidfd candidates cannot work with tokio (medium):
  taken.
  The text says their tokio suitability was not established.
- Search coverage is overstated (medium):
  taken.
  The watcher claim says "first page",
  and the test-helper facts say the study found no crate instead of saying no crate gives the helpers.
- The introduction and options do not handle the ten one-folder entries (medium):
  taken.
  The introduction says 16 group folders and 10 one-folder entries,
  and options B and C say a one-folder entry keeps exact links as in A.
- The group link lists assume Option B (medium):
  taken.
  They are labelled the summaries of Option B,
  and the text says an initial layout under Option A omits them.
- Option A overstates what the installed lint enforces (medium):
  taken.
  The text separates the links and the folder-loop check (enforced) from the shelf-only restriction
  (not enforced;
  a convention until a new check exists).
- The Option B example names an entry that does not exist (medium):
  taken.
  It names `saving_copies_of_what_a_command_would_throw_away`.
- The list is in dependency order,
  not by count (low):
  taken.
- The stand-alone claim exceeds the evidence (high):
  taken in part.
  The rescan now says what it shows (a structural check of the production model with no library use of a program file and no loop)
  and what it does not (independent builds and tests are unverified and the owner's confirmation is pending).
  The sentence "The eight pass the coordinator's stand-alone test" is older text of the second review's version
  and was left unedited;
  the new sections carry the qualification.
- Test-helper Option C contradicts itself (medium):
  not acted on.
  The option and its ranking are the text the owner saw and answered "Not yet."
  to;
  the section only adds facts.
  The reviewer's point (tests that use only `std` can run alone,
  at a cost of rewrites and duplication) is recorded here for the owner's reading.
- "Demonstrated outside use" is really duplication of a related operation (medium):
  taken.
  Option E now says duplicates a related operation,
  not a consumer of the library's API.
- `cli` and `core-rust` directory names fail the naming rule (medium):
  taken in part.
  The facts of the location point say these spellings are precedent and not names that pass MKY alone.
  They are not changed because the owner answered "Not yet."
  on where libraries live.
- The masker keeps an `lfs` name (medium):
  taken.
  A rename is proposed in the text and in the group map (`hiding_the_password_in_web_addresses_in_text`).
- The veto round is not a naming-compliance test,
  and several folder names are opaque (medium):
  taken in part.
  The text says the veto round settles names that fail today,
  and lists the reviewer's examples as names for a plainer name or a top-doc explanation.
  They are not renamed here (proposal only).
- Relative pointers (low):
  taken for the new sections;
  older text of the file that says above or below is left.

## Next

Read against today's tree on 2026-11-26 (section "Rescan of main after the reformat (2026-11-26)").
Each item below is one branch,
in the order it must run,
because a branch that edits a file another branch is about to move or rewrite fights it.
The order keeps four rules:

- a change that splits a file or creates one goes before the move that places files,
  since the placement table carries whole files;
- a change that only rewrites a file's body goes after the move of its folder,
  so it is written once,
  on the file's final path;
- nothing that writes a folder's file list or its `uses/` links runs before the content changes of that folder,
  or the lists go stale;
- a test-side change runs after the move,
  because every sidecar is renamed with its subject and its `#[path]` line rewritten.

The rounds,
in order:

1.  `refactor/git-policy-cli-split-single-item-moves`:
    the rest of step 2.
    Touches `worktree_copying/`,
    `policy/`,
    `invocation/`,
    `landing/` and `transaction/`,
    and five of the changes it names have to be re-pointed at today's files,
    because the table rows that named them are gone
    (`worktree_command_facts`,
    `current_step_record`,
    `what_a_command_hands_back`,
    `push_attempt_and_failure`,
    `command_options_types`).
2.  `refactor/git-policy-cli-split-oversized-files`:
    debt ledger item 15,
    one branch per file,
    before the loop breaks because a split changes a loop's members,
    and before the move because a split changes which files exist.
    Touches the 24 files over 300 non-comment lines,
    of which the largest are `worktree/worktree_remove_hand_over.rs` (569),
    `worktree_clone_mount_handler/record_checkout_mode.rs` (437),
    `worktree_clone_mount_handler/mount_path_descendant_mode.rs` (436),
    `repair/package_repair_toolchain.rs` (434),
    `binary/binary_support_fixture_service.rs` (432),
    `worktree_copying/worktree_copy_owed_copy.rs` (407),
    `landing/service_claim_watch.rs` (404),
    `repair/package_repair_prewarm_job.rs` (386),
    `landing/service_work_drain_pending_claims.rs` (368),
    `worktree_clone_add_route/add_route_steps.rs` (358)
    and `worktree_clone_mount_handler/handler_mode_arguments.rs` (356).
    This is the planning doc's "Oversized files" section,
    which already says the files are split now and the layout pass moves the pieces.
3.  `refactor/git-policy-cli-split-loop-breaks`:
    step 3,
    one commit per loop of the list above,
    folder by folder from the bottom,
    with the allow-list lines deleted as each use goes.
4.  `refactor/git-policy-cli-split-call-changes`:
    step 4's five call changes,
    one commit each:
    finished results,
    system settings path,
    after-stop action,
    current step record,
    run this next.
5.  `refactor/git-policy-cli-split-libraries-stand-alone`:
    step 5's seven changes and the new ones the rescan found,
    one commit each,
    which is what removes the 72 uses in 36 files and with them the crate and folder loops.
6.  `refactor/git-policy-cli-split-flat-re-exports`:
    step 6,
    which rewrites every `use crate::<file>` to name the folder and then deletes the 51 flat re-exports
    and the 126 named ones.
7.  `refactor/git-policy-cli-split-group-into-folders`:
    step 7,
    one commit per source folder of the rescan's list,
    bottom first,
    which is the move the placement table describes.
8.  `refactor/git-policy-cli-split-byte-tests`:
    debt ledger item 12,
    after the move and after item 15,
    because it rewrites bodies in about 90 files and creates one helper,
    so it must not edit a file another round is about to split,
    and its new helper is placed in the folder it belongs to before any file list is written.
9.  `refactor/git-policy-cli-split-folder-lists-and-links`:
    the rest of step 8:
    each folder's file list and its `uses/` links written together,
    the 386 tolerated links and the 1318 tolerated entries deleted as they go,
    the 17 group summaries written once the owner words the group rule.
10. `refactor/git-policy-cli-split-test-folders-library`:
    step 9,
    the Git-free half of
    `testing/` lifted as `disposable-test-folders`.
11. `refactor/git-policy-cli-split-lift-<library>`:
    step 10,
    one branch per library the owner keeps,
    each shown to stand alone before the next is lifted.
    The order is a dependency order whichever reading the owner picks:
    `text-time-and-log-line-formats`,
    `dependent-version-bump-check` and
    `talking-to-the-systemd-user-manager`,
    then `parallel-work-within-a-budget`,
    then `safe-file-saving`,
    then `starting-and-watching-programs`,
    then `linux-file-and-disk-calls`,
    then `fix-packages-after-copying-a-folder`.
    Reading D lifts four of them and reading A eight,
    and the test-folders library of step 9 comes first in either.
12. `refactor/git-policy-cli-split-test-waits-<folder>`:
    debt ledger item 7,
    one branch per target folder,
    after step 9 so the helpers are final and after the harness rounds in flight,
    because every sidecar it edits is renamed by step 7 and every wait helper it uses moves in step 9.
13. `refactor/git-policy-cli-split-program-leftovers`:
    step 11,
    the program crate as what is left,
    run after the last lift:
    no step of it can start before a library is lifted,
    its `Cargo.toml` keeps the binary,
    the one test target and the release profile and loses the libraries' dependencies.

What the order keeps apart:

- Debt ledger item 15 (step 2 here) and the loop breaks (step 3) touch the same folders,
  so item 15 runs first and each of its files is split before the loop it sits in is broken,
  since a split changes that loop's members.
- Debt ledger item 12 (step 8) touches release files across every folder,
  so it runs after the move and before the lists are written,
  or the new helper file would be missing from 110 file lists.
- Debt ledger item 7 (step 12) touches sidecars in every folder,
  so it runs after the move,
  after the test-folders library and after the harness rounds already in flight,
  and it runs folder by folder so two of its branches never edit one helper.
- The rounds already in flight and what they share with the move:
  issues 6483 to 6487,
  6489,
  6493,
  6494 and 6495 edit sidecars and
  `testing/test_support.rs`,
  which step 7 renames and step 9 splits;
  ledger item 19 (fixed-length lists become slices) edits release files whose imports steps 6 and 7 rewrite;
  the Ubuntu `setgroups` skip touches `account_companion/` and `start_probe/`,
  which move whole in step 7 and appear nowhere else in this order.
  None of them is scheduled here.

What still needs the owner,
and is not decided in this section:

- The test helpers,
  which the owner answered "Not yet."
  on 2026-11-07 (open point "The test helpers").
  Today's `testing/` folder is what the answer will split.
- Where the libraries live and what they are named,
  which the owner answered "Not yet."
  on 2026-11-07 (open point "Where the libraries live and what they are named").
- The 32 new folder names of the ZFS route,
  which are this round's wording and need the veto round MKY asks for.
- Whether `repairs_the_service_runs_for_a_new_worktree` stays a program folder:
  its 11 modules run inside the commit service and belong to the repair library's subject,
  and the owner's reading D or E decides which crate owns them.
- Whether the ZFS route's folders become a group of their own under "One level of groups",
  or stand as one-folder entries beside the existing ones.
- The wording of the group rule LYL now checks,
  since the linter reports 17 missing group summaries and 38 group names the code disagrees with:
  the owner must word it or approve the checker's reading.
