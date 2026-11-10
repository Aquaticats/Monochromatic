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

### Loops between files, as they stand

Measured on `main` at `f44d921e24` with the target groups (the first number of each edge is the line of the first use):

- loop of 3:
  `child_watch`,
  `child_watch_clock_marks_and_stopping`,
  `push_progress`
  - `push/child_watch.rs:63` uses `child_watch_clock_marks_and_stopping`:
    Mark,
    Marks,
    SHORTEST_SLEEP,
    WatchFlags,
    WatchTerms,
    grace_for,
    stop_process
  - `push/child_watch.rs:45` uses `push_progress`:
    Clock,
    SAMPLE_INTERVAL
  - `push/child_watch_clock_marks_and_stopping.rs:?` (through a planned item move) uses `child_watch`:
    Patience
  - `push/child_watch_clock_marks_and_stopping.rs:17` uses `push_progress`:
    Clock
  - `push/push_progress.rs:?` (through a planned item move) uses `child_watch`:
    StopCause
- loop of 4:
  `command_options`,
  `command_options_long`,
  `command_options_short`,
  `command_options_value`
  - `command/command_options.rs:29` uses `command_options_long`:
    scan_long_option
  - `command/command_options.rs:30` uses `command_options_short`:
    scan_short_cluster
  - `command/command_options_long.rs:26` uses `command_options`:
    Occurrence,
    OptionError,
    OptionErrorKind,
    OptionSpec,
    OptionValue,
    ParseMode,
    ParsedOptions
  - `command/command_options_long.rs:29` uses `command_options_value`:
    Spelling,
    take_value
  - `command/command_options_short.rs:25` uses `command_options`:
    Occurrence,
    OptionError,
    OptionErrorKind,
    OptionSpec,
    OptionValue,
    ParseMode,
    ParsedOptions
  - `command/command_options_short.rs:28` uses `command_options_value`:
    Spelling,
    take_value
  - `command/command_options_value.rs:25` uses `command_options`:
    Arity,
    OptionError,
    OptionErrorKind,
    OptionValue
- loop of 2:
  `git_metadata`,
  `repo_read_hook`
  - `repo_read/git_metadata.rs:16` uses `repo_read_hook`:
    try_metadata
  - `repo_read/repo_read_hook.rs:14` uses `git_metadata`:
    MetadataOutput
- loop of 2:
  `real_git`,
  `real_git_resolution`
  - `invocation/real_git.rs:27` uses `real_git_resolution`:
    RecordedGit,
    ResolvedGit,
    resolve_real_git
  - `invocation/real_git_resolution.rs:36` uses `real_git`:
    RealGitNotFound,
    ResolutionInputs,
    SearchedCandidate,
    search_path_for_real_git
- loop of 2:
  `landing_location`,
  `repo_read_adapter`
  - `landing/landing_location.rs:19` uses `repo_read_adapter`:
    landing_of
  - `repo_read/repo_read_adapter.rs:13` uses `landing_location`:
    LandingLocation
- loop of 2:
  `index_lock_holder_parsers`,
  `index_lock_holders`
  - `lock/index_lock_holder_parsers.rs:19` uses `index_lock_holders`:
    Holder
  - `lock/index_lock_holders.rs:444` uses `index_lock_holder_parsers`:
    LsofProcess,
    matching_holders,
    parse_lsof_fields,
    parse_lsof_number,
    parse_restart_manager_output
- loop of 2:
  `service_index_identity`,
  `service_index_marker`
  - `landing/service_index_identity.rs:26` uses `service_index_marker`:
    marker_of
  - `landing/service_index_marker.rs:106` uses `service_index_identity`:
    BOOT_SEPARATOR,
    HolderState,
    ReaderView,
    check_procfs_view,
    current_pid_scope,
    holder_is_elsewhere,
    holder_is_gone,
    holder_is_running,
    holder_state,
    holder_state_in,
    identity_in_view,
    identity_in_view_on,
    mark_names_a_gone_holder,
    own_identity,
    pid_of_self_stat
- loop of 6:
  `service_index_attempts`,
  `service_index_claim`,
  `service_index_lock`,
  `service_index_placement`,
  `service_index_prestage`,
  `service_index_reclaim`
  - `landing/service_index_attempts.rs:27` uses `service_index_lock`:
    Announce,
    LockFailure,
    lock_path_of,
    notice_every
  - `landing/service_index_attempts.rs:29` uses `service_index_placement`:
    Platform,
    Reclaimed,
    log_placement_failure,
    place_marked,
    reclaim
  - `landing/service_index_attempts.rs:30` uses `service_index_reclaim`:
    foreign_words,
    stale_lock_words,
    unjudged_words
  - `landing/service_index_claim.rs:23` uses `service_index_placement`:
    Platform,
    place_marked
  - `landing/service_index_lock.rs:29` uses `service_index_attempts`:
    Placing,
    wait_and_place
  - `landing/service_index_lock.rs:42` uses `service_index_placement`:
    Platform,
    SYSTEM,
    log_placement_failure,
    place_marked,
    placement_failure_text,
    reclaim
  - `landing/service_index_lock.rs:35` uses `service_index_prestage`:
    Prestaged,
    take
  - `landing/service_index_lock.rs:46` uses `service_index_reclaim`:
    Reclaimed,
    foreign_words,
    remove_stale,
    stale_lock_words,
    unjudged_words
  - `landing/service_index_placement.rs:272` uses `service_index_reclaim`:
    Reclaimed,
    reclaim_if
  - `landing/service_index_prestage.rs:28` uses `service_index_lock`:
    lock_path_of
  - `landing/service_index_prestage.rs:30` uses `service_index_placement`:
    sync_file
  - `landing/service_index_reclaim.rs:23` uses `service_index_claim`:
    Claim,
    ClaimHold,
    take_claim
  - `landing/service_index_reclaim.rs:22` uses `service_index_lock`:
    lock_path_of
  - `landing/service_index_reclaim.rs:25` uses `service_index_placement`:
    Platform,
    SYSTEM
- loop of 6:
  `stale_ref_lock_found`,
  `stale_ref_lock_judge`,
  `stale_ref_lock_retire`,
  `stale_ref_lock_walk`,
  `stale_ref_locks`,
  `stopped_git_locks`
  - `lock/stale_ref_lock_found.rs:12` uses `stale_ref_lock_judge`:
    Verdict
  - `lock/stale_ref_lock_judge.rs:18` uses `stale_ref_lock_found`:
    PidFile
  - `lock/stale_ref_lock_judge.rs:21` uses `stale_ref_lock_walk`:
    pid_file_beside
  - `lock/stale_ref_lock_judge.rs:20` uses `stopped_git_locks`:
    pid_named_by
  - `lock/stale_ref_lock_retire.rs:18` uses `stale_ref_lock_found`:
    Found,
    Outcome
  - `lock/stale_ref_lock_retire.rs:19` uses `stale_ref_lock_judge`:
    Examined,
    Identity,
    Judge,
    Verdict,
    identity_of
  - `lock/stale_ref_lock_retire.rs:21` uses `stale_ref_lock_walk`:
    Candidate,
    companions_of
  - `lock/stale_ref_lock_walk.rs:13` uses `stopped_git_locks`:
    LOCK_SUFFIX,
    PID_LOCK_SUFFIX
  - `lock/stale_ref_locks.rs:25` uses `stale_ref_lock_found`:
    Found,
    Outcome,
    PidFile
  - `lock/stale_ref_locks.rs:23` uses `stale_ref_lock_judge`:
    Judge,
    Proof,
    Verdict
  - `lock/stale_ref_locks.rs:29` uses `stale_ref_lock_retire`:
    judge_and_clear
  - `lock/stale_ref_locks.rs:27` uses `stale_ref_lock_walk`:
    ref_lock_candidates
  - `lock/stopped_git_locks.rs:37` uses `stale_ref_lock_judge`:
    Judge,
    OwnEnded
  - `lock/stopped_git_locks.rs:41` uses `stale_ref_locks`:
    clear_stale_ref_locks
- loop of 4:
  `auto_push_single_flight`,
  `bounded_push`,
  `bounded_query`,
  `push_lock_clearing`
  - `push/auto_push_single_flight.rs:15` uses `bounded_query`:
    QueryOutput,
    query_window,
    run_bounded_query
  - `push/bounded_push.rs:20` uses `auto_push_single_flight`:
    PushAttempt,
    SingleFlightError
  - `push/bounded_push.rs:441` uses `push_lock_clearing`:
    run_clearing_locks
  - `push/bounded_query.rs:18` uses `auto_push_single_flight`:
    SingleFlightError
  - `push/bounded_query.rs:20` uses `bounded_push`:
    MAX_PUSH_OUTPUT,
    stop_push,
    wait_until_done
  - `lock/push_lock_clearing.rs:17` uses `auto_push_single_flight`:
    PushAttempt,
    SingleFlightError
  - `lock/push_lock_clearing.rs:18` uses `bounded_push`:
    run_bounded_push_once
- loop of 2:
  `rule_linked_worktree`,
  `rule_linked_worktree_messages`
  - `command/rule_linked_worktree.rs:48` uses `rule_linked_worktree_messages`:
    dirty_worktree_message,
    main_worktree_message,
    outside_worktree_message
  - `command/rule_linked_worktree_messages.rs:15` uses `rule_linked_worktree`:
    GuardedCommand
- loop of 5:
  `auto_push`,
  `auto_push_reconcile`,
  `push_failure_notice`,
  `push_retry`,
  `push_retry_store`
  - `push/auto_push.rs:20` uses `auto_push_reconcile`:
    RECONCILE_FAILED_PREFIX,
    RECONCILE_LINE_PREFIX,
    reconcile_divergent_push
  - `push/auto_push_reconcile.rs:31` uses `push_retry`:
    CONFLICT_PHRASE
  - `push/push_failure_notice.rs:21` uses `auto_push`:
    PUSH_FAILED_NOTE
  - `push/push_retry.rs:21` uses `push_retry_store`:
    StoredPlan,
    load,
    remove,
    save
  - `push/push_retry_store.rs:17` uses `push_failure_notice`:
    fnv_hex
- loop of 2:
  `push_stacking`,
  `stacked_merge_push`
  - `push/push_stacking.rs:35` uses `stacked_merge_push`:
    gated_merge_push
  - `push/stacked_merge_push.rs:25` uses `push_stacking`:
    STACKED_PUSH_MARKER,
    run_stacked_push
- loop of 3:
  `worktree_snapshot_source`,
  `worktree_trash`,
  `worktree_trash_dispose`
  - `worktree/worktree_snapshot_source.rs:497` uses `worktree_trash`:
    trash_location_is_ignored
  - `worktree/worktree_trash.rs:27` uses `worktree_snapshot_source`:
    SnapshotSource
  - `worktree/worktree_trash.rs:28` uses `worktree_trash_dispose`:
    Reach,
    Router,
    TRASH_REFERENCE,
    delete_trash,
    dispose_pending_trash,
    dispose_pending_trash_with,
    empty_trash
  - `worktree/worktree_trash_dispose.rs:27` uses `worktree_trash`:
    TRASH_PARENT
- loop of 3:
  `worktree_copy`,
  `worktree_ladder`,
  `worktree_ladder_run`
  - `worktree_copying/worktree_copy.rs:35` uses `worktree_ladder`:
    LadderResult,
    begin_ladder
  - `worktree/worktree_ladder.rs:32` uses `worktree_copy`:
    GitRunner,
    GitStatus,
    GitStep
  - `worktree/worktree_ladder.rs:17` uses `worktree_ladder_run`:
    Ladder,
    LadderResult,
    begin_ladder
  - `worktree/worktree_ladder_run.rs:22` uses `worktree_copy`:
    GitStatus,
    GitStep
- loop of 2:
  `conflict_side_notice`,
  `conflict_sides`
  - `conflict/conflict_side_notice.rs:15` uses `conflict_sides`:
    SavedCopy
  - `conflict/conflict_sides.rs:42` uses `conflict_side_notice`:
    clean_paths_note,
    saved_notice
- loop of 3:
  `index_reset_file`,
  `service_land_copy`,
  `service_land_follow`
  - `index/index_reset_file.rs:31` uses `service_land_follow`:
    IndexPaths,
    PrivateCopy
  - `landing/service_land_copy.rs:21` uses `service_land_follow`:
    storage_failure
  - `landing/service_land_follow.rs:269` uses `index_reset_file`:
    reset_index_file
  - `landing/service_land_follow.rs:24` uses `service_land_copy`:
    PrivateCopy

### Breaking each loop

- Option reading (`command_options`,
  `_long`,
  `_short`,
  `_value`):
  new file `command_options_types` receives
  `Occurrence`,
  `OptionError`,
  `OptionErrorKind`,
  `OptionSpec`,
  `OptionValue`,
  `ParseMode`,
  `ParsedOptions`,
  `Arity`.
- Linked-worktree rule:
  `GuardedCommand` moves from `rule_linked_worktree` to `rule_linked_worktree_messages`.
- Conflict sides:
  `SavedCopy` moves from `conflict_sides` to `conflict_side_notice`.
- Landing copy:
  `IndexPaths` moves from `service_land_follow` to `index_reset_file`;
  `index_reset_file` imports `PrivateCopy` from `service_land_copy`,
  where it is defined.
- Real Git search:
  `RealGitNotFound`,
  `ResolutionInputs`,
  `SearchedCandidate` and `search_path_for_real_git` move from
  `real_git` to `real_git_candidate`.
- Read-only Git queries:
  `MetadataOutput` moves from `git_metadata` to `repo_read_query_request_answer_and_context`.
- Index location:
  `LandingLocation` moves from `landing_location` to `repo_read_adapter`.
- Index-lock marker and identity:
  `marker_of` and `Marked` move from `service_index_marker` to `service_index_identity`.
- The six-file index-lock acquisition loop:
  new file `service_index_lock_names_and_failures` receives `Announce`,
  `LockFailure`,
  `lock_path_of`,
  `notice_every` (from `service_index_lock`) and `storage_failure`
  (from `service_land_follow`);
  `reclaim` moves from `service_index_placement` to `service_index_reclaim`,
  which already
  holds `reclaim_if`.
  Then placement uses nothing of reclaim,
  and the order
  names,
  placement,
  claim,
  reclaim,
  prestage,
  attempts,
  lock has no back edge.
- Reference locks (six files):
  new file `stale_ref_lock_names` receives `LOCK_SUFFIX`,
  `PID_LOCK_SUFFIX`,
  `pid_named_by` (from `stopped_git_locks`) and `pid_file_beside` (from `stale_ref_lock_walk`);
  `Verdict` moves from `stale_ref_lock_judge` to `stale_ref_lock_found`.
- Holder scan:
  `Holder` moves from `index_lock_holders` to `index_lock_holder_parsers`.
- Log lines and the index-lock hold (the two were draft crates;
  now two program folders):
  `WaitNotes` (plain data) moves from `service_index_found` to `service_timing`,
  so the log-lines folder does not use the index-lock folder.
  Kept:
  it also keeps the two liftable.
- Review key:
  `service_review_key` (75 lines,
  uses nothing) moves from the review folder to
  the messages folder,
  so the messages folder uses no other folder and the index-lock folder does not pull the
  HTTPS client and the real-Git folder in through the protocol.
  Kept for the same reason.
- Push runner (four files):
  new file `push_attempt_and_failure` receives `PushAttempt` and `SingleFlightError`;
  new file `bounded_push_waiting` receives `MAX_PUSH_OUTPUT`,
  `stop_push`,
  `wait_until_done` and `run_bounded_push_once`
  (all from `bounded_push`),
  which breaks the cycle between `bounded_push` and `push_lock_clearing`.
- Child watch and push progress:
  `StopCause` moves from `bounded_push` to `push_progress`;
  `Patience` is read from `push_deadline_and_patience_settings`,
  where it is defined.
- Background push (five files):
  `fnv_hex` moves from `push_failure_notice` to `auto_push_branch_key`;
  `PUSH_FAILED_NOTE` from `auto_push` to `push_failure_notice`;
  `CONFLICT_PHRASE` from `push_retry` to
  `auto_push_reconcile_basics_and_limits`.
- Push stacking and stacked merge push:
  `push_stacking` stops importing `gated_merge_push`;
  the wiring that passes it
  (today at the end of `push_stacking`) moves to its caller `entry_push_stacking_and_lfs_upload`,
  which already imports
  both.
  This goes with the first call change below.
- Snapshot trash:
  `trash_location_is_ignored` moves from `worktree_trash` to `worktree_snapshot_source`;
  `TRASH_PARENT` moves to `worktree_trash_manifest`.
- Copy and ladder:
  new file `worktree_copy_git_steps` receives `GitRunner`,
  `GitStatus` and `GitStep` from `worktree_copy`.

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

The owner answered the group rule,
the count of libraries and how a library prints a log line on 2026-11-07T11:05:21-05:00
(section "The owner's words and the rules that bind this").
Still open from the owner:
the test helpers and where the libraries live ("Not yet."
to both).
Next:
the `monochromatic-lint` change for group summaries starts now (it touches no wrapper file);
the tokio steps still to come land,
the rescan runs again,
and steps 2 to 6 of the move order start,
each on its own branch.

Progress,
2026-11-10T19:05:00Z:
step 0's release is in place (the crates.io check in section "Cargo packaging facts that bear on the tree");
step 2's `copy_file_exclusive` item landed,
red commit `6f5a74db22` and fix commit `36affbb3ec`,
and the copy stage words the failure exactly as before.
Not started:
the other four items of step 2,
the loop breaks of step 3,
the call changes of step 4,
and the rescan of step 1,
which waits for the tokio steps T6 to T13 to land.
