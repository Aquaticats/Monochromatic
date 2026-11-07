# Target tree of the Git wrapper, file by file

Companion to [`cli-git-source-layout-target-tree.md`](cli-git-source-layout-target-tree.md);
read that file first.
Redone 2026-11-07 for the eight libraries,
the test-folders library and the program crate.
Generated from the table in [`cli-git-source-layout-groups.json`](cli-git-source-layout-groups.json)
(key `target`),
the file-to-file references of `main` at `f44d921e24` (2026-11-04T20:45:14-05:00) with the named changes
applied,
and the one line each file's own top doc gives.

How to read one entry:
the new file name,
then `<-` and the old path under `package/git-policy/cli/`,
then the explanation
(the first sentence of the old top doc,
cut at about a hundred characters;
the new top doc will say more,
and for a file in a library it says nothing about Git,
so an explanation here that names Git is reworded when the file moves).
Within a folder (or within `src/` for a crate with one folder) files are in the dependency order the top doc must list:
every file uses only files above it.
Files marked `NEW` are created by the changes of the main document.
A `uses/` line lists the folders of the same crate that the folder uses.
The program crate (`cli`) holds 74 folders;
the libraries sit at `package/<name>/core-rust` (open point "Where the libraries live and what they are named").

This file does not yet list the tree of `disposable-test-folders` nor `main.rs` of the program crate;
both follow from the main document
(the test-folders library holds the Git-free half of `test_support` and its neighbours).
Files added to `main` after the scan are not listed (section "The crate split",
"Files of `main` added after the scan").

## The crates, bottom first

### dependent-version-bump-check

Package `package/dependent-version-bump-check/core-rust`,
crate `monochromatic-dependent-version-bump-check`.
Plans a version bump for every package that depends on a package whose version was raised.

Depends on crates:
none.

```text
package/dependent-version-bump-check/core-rust/src/
  lib.rs  (index: top doc lists the files below in this order, then one `mod` line each)
  dependent_version_content.rs  <- src/native/dependent_version/dependent_version_content.rs
    The repository content the dependent-version planner reads, behind one small interface, and the two ways planning can ...
  dependent_version_release.rs  <- src/native/dependent_version/dependent_version_release.rs
    Patch-bump a plain `major.minor.patch` release version, and quote text the way `JSON.stringify` does.
  dependent_version_graph.rs  <- src/native/dependent_version/dependent_version_graph.rs
    Walk the workspace dependency graph from the packages whose version was raised and plan a patch bump for every ...
  dependent_version_imports.rs  <- src/native/dependent_version/dependent_version_imports.rs
    Decide whether source text imports a package or one of its subpaths. Why: A development dependency is bundled, and so ...
  dependent_version_paths.rs  <- src/native/dependent_version/dependent_version_paths.rs
    Which repository paths the dependent-version planner reads: workspace manifests, the registry configuration, and a ...
  dependent_version_manifest.rs  <- src/native/dependent_version/dependent_version_manifest.rs
    Read the facts a dependent bump needs from one `package.json`: its name, its version, and the workspace names it ...
  dependent_version_states.rs  <- src/native/dependent_version/dependent_version_states.rs
    Read every workspace manifest at the candidate state and at the base, and decide which packages' versions were raised.
  dependent_version_bundled.rs  <- src/native/dependent_version/dependent_version_bundled.rs
    Decide which development dependencies each dependent bundles, reading source only where an edge could carry a raised ...
  dependent_version_publishable.rs  <- src/native/dependent_version/dependent_version_publishable.rs
    Read the publishable package names from the generated registry configuration (`package/config/pnpr/config.yaml`).
  dependent_version_text.rs  <- src/native/dependent_version/dependent_version_text.rs
    Rewrite the top-level `version` value in a manifest's text, leaving every other byte unchanged.
  dependent_version_plan.rs  <- src/native/dependent_version/dependent_version_plan.rs
    Plan patch bumps for publishable dependents of packages whose version differs from the base, with the exact manifest ...
```

### parallel-work-within-a-budget

Package `package/parallel-work-within-a-budget/core-rust`,
crate `monochromatic-parallel-work-within-a-budget`.
Runs independent jobs side by side on as many threads as the computer can spare.

Depends on crates:
none.

```text
package/parallel-work-within-a-budget/core-rust/src/
  lib.rs  (index: top doc lists the files below in this order, then one `mod` line each)
  how_many_jobs_may_run_together.rs  <- src/native/foundation/parallel_budget.rs
    How many workers a set of independent items may use together: the CPUs the process is allowed minus a margin, the ...
  starting_a_job_on_a_helper_thread.rs  <- src/native/foundation/scoped_start.rs
    Start a closure on a scoped thread, and run it on the calling thread when the system cannot start one.
  running_jobs_side_by_side.rs  <- src/native/foundation/parallel_runner.rs
    Runs independent items on a bounded number of threads and reports what they say and what they return in input order
```

### text-time-and-log-line-formats

Package `package/text-time-and-log-line-formats/core-rust`,
crate `monochromatic-text-time-and-log-line-formats`.
Writes text,
UTC time stamps and compact JSON records byte for byte as JavaScript does,
quotes text for a shell command,
and prints tagged debug and warning lines.

Depends on crates:
none.

```text
package/text-time-and-log-line-formats/core-rust/src/
  lib.rs  (index: one `mod` line per folder, no code, no re-exports)
  text_time_and_records_written_like_javascript/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    utc_time_stamps_like_javascript.rs  <- src/native/foundation/iso_time.rs
      UTC timestamps in the exact form of JavaScript's `Date.prototype.toISOString`
    compact_json_records.rs  <- src/native/foundation/json_record.rs
      Reading and writing the compact JSON records both wrappers share
    text_operations_like_javascript.rs  <- src/native/foundation/js_text.rs
      Text operations with the exact semantics of the JavaScript built-ins the incumbent
  lines_printed_for_the_person/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> text_time_and_records_written_like_javascript
    debug_and_warning_lines_on_standard_error.rs  <- src/native/foundation/diagnostic_log.rs
      Line-oriented debug and warning diagnostics on standard error
    quoting_bytes_for_a_shell_command.rs  <- src/native/worktree/shell_quote.rs
      Quoting of bytes that are printed inside a shell command for a person to copy and run.
```

### safe-file-saving

Package `package/safe-file-saving/core-rust`,
crate `monochromatic-safe-file-saving`.
Saves a file so a crash never leaves half of it,
makes private scratch folders and random names,
and cleans up paths.

Depends on crates:
`text-time-and-log-line-formats`.

```text
package/safe-file-saving/core-rust/src/
  lib.rs  (index: top doc lists the files below in this order, then one `mod` line each)
  cleaning_up_a_path_by_its_text.rs  <- src/native/foundation/lexical_path.rs
    Lexical path normalization, the way Node's `path.resolve` does it.
  paths_as_exact_bytes.rs  <- src/native/policy/path_bytes.rs
    Exact repository path bytes as the operating system takes them
  random_names.rs  <- src/native/foundation/random_id.rs
    Random identifiers in the layout of the incumbent's `randomUUID`
  saving_private_files.rs  <- src/native/foundation/private_storage.rs
    Private
  why_a_file_step_failed.rs  <- src/native/transaction/recovery_error.rs
    The fail-closed failure of reading or recovering durable transaction state.
  file_steps_that_report_why_they_failed.rs  <- src/native/transaction/recovery_files.rs
    The small file operations the shadow, the pack keep files, the conclusion copies and the worktree installs share
  private_scratch_places.rs  <- src/native/foundation/scratch_location.rs
    Where the wrapper keeps short-lived private files
  questions_about_a_path_or_a_byte.rs  <- src/native/worktree/worktree_add_plan_path_questions.rs
    Small questions about a path or a byte that the `git worktree add` plans ask: whether it is a slash, a link, a `..`
```

### starting-and-watching-programs

Package `package/starting-and-watching-programs/core-rust`,
crate `monochromatic-starting-and-watching-programs`.
Starts another program with its output kept in files,
passes signals to it,
tells which programs are running and since when and which hold a file open,
and ends a program that shows no sign of work.

Depends on crates:
`text-time-and-log-line-formats`.

```text
package/starting-and-watching-programs/core-rust/src/
  lib.rs  (index: one `mod` line per folder, no code, no re-exports)
  starting_programs_catching_signals_and_keeping_their_output/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    how_a_child_program_ended.rs  <- NEW
      How a child program ended (exit code or signal) and the plain value that says so, which the signal relay and the ...
    child_output_kept_in_files.rs  <- src/native/process/output_capture.rs
      Output of a child process captured into two anonymous files instead of two pipes, and read back from the start ...
    starting_a_program_again_when_processes_run_out.rs  <- src/native/push/start_retry.rs
      Starting a child process again when the operating system refuses it for lack of processes.
    passing_signals_to_the_child.rs  <- src/native/process/signal_relay.rs
      Catching `SIGINT`, `SIGTERM`, `SIGHUP` and `SIGQUIT` while the wrapper waits for a Git child
  which_programs_are_running_and_since_when/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    reading_the_output_of_a_holder_scan.rs  <- src/native/lock/index_lock_holder_parsers.rs
      Parsers of the holder-scan output that is not `/proc`
    programs_holding_a_file_open.rs  <- src/native/lock/index_lock_holders.rs
      Processes holding a lock file open
    telling_one_process_from_a_reused_number.rs  <- src/native/process/process_identity.rs
      Process-birth identities
    when_a_process_started.rs  <- src/native/process/process_start.rs
      The wall-clock start time of a process
    a_process_and_the_programs_below_it.rs  <- src/native/process/process_tree.rs
      The processes below one process: found before it is stopped, and signalled afterwards, each only while it is still
    which_programs_use_a_file_or_folder.rs  <- src/native/process/process_use.rs
      The processes of this host that use a directory or a file
    process_numbers_inside_nested_containers.rs  <- src/native/lock/stale_ref_lock_nested_numbers.rs
      Which processes of the table carry a given process number inside a PID namespace nested in the reader's.
    open_network_connections_and_bytes_sent.rs  <- src/native/push/tcp_sockets.rs
      The TCP sockets of this machine that are connected, with the bytes each has sent and had acknowledged and the bytes ...
  watching_a_started_program_until_it_stops_making_progress/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> starting_programs_catching_signals_and_keeping_their_output, which_programs_are_running_and_since_when
    measuring_progress_of_a_program_and_what_it_started.rs  <- src/native/push/push_progress.rs
      The progress of a push: whether the push and everything it started are at work, sampled at a small interval, and the
    wait_notice_clock.rs  <- src/native/push/wait_progress.rs
      The shared clock of every wait notice
    watch_clock_marks_and_stopping.rs  <- src/native/push/child_watch_clock_marks_and_stopping.rs
      What a child watch is made of below its public face: the terms a watch runs under, the marks the waiting thread sends
    watch_step_limits_and_errors.rs  <- src/native/push/child_watch_step_bounds_and_errors.rs
      How a watched step may be ended and the errors that say it was: the bound a step runs under, the error of a step ...
    longest_quiet_gaps.rs  <- src/native/process/quiet_ledger.rs
      The longest quiet gap of every Git step and every tool this process ran, kept until a timing record takes it.
    watching_a_child_program.rs  <- src/native/push/child_watch.rs
      A watcher for a wait that blocks inside a read or a wait on a child process
    is_a_thread_at_work.rs  <- src/native/process/thread_quiet.rs
      Whether one thread of this process is at work, judged with the signs the push meter reads for a child process, and the
```

### fix-packages-after-copying-a-folder

Package `package/fix-packages-after-copying-a-folder/core-rust`,
crate `monochromatic-fix-packages-after-copying-a-folder`.
Repairs the installed packages of a copied folder (linked `node_modules`,
Cargo build state,
toolchain launchers) so they work at the new location.

Depends on crates:
`parallel-work-within-a-budget`,
`safe-file-saving`,
`starting-and-watching-programs`,
`text-time-and-log-line-formats`.

```text
package/fix-packages-after-copying-a-folder/core-rust/src/
  lib.rs  (index: top doc lists the files below in this order, then one `mod` line each)
  package_repair_cargo_concurrent.rs  <- src/native/repair/package_repair_cargo_concurrent.rs
    Run one job for every item of a list on several threads, each item once, and give one result per item in item order.
  package_repair_cargo_lock.rs  <- src/native/repair/package_repair_cargo_lock.rs
    Reading `Cargo.lock` before a `cargo clean` and putting it back as it was afterwards.
  package_repair_command.rs  <- src/native/repair/package_repair_command.rs
    Starting one package-manager program for a worktree repair: finding it on a search path, running it in a directory ...
  package_repair_request.rs  <- src/native/repair/package_repair_request.rs
    What a repair of a new worktree reads and is asked: the person's switches, what the process offers, and the request
  package_repair_cargo_workspaces.rs  <- src/native/repair/package_repair_cargo_workspaces.rs
    The Cargo workspaces of a new worktree: what `cargo metadata --no-deps` says about one, the tracked manifests that
  package_repair_report.rs  <- src/native/repair/package_repair_report.rs
    What a worktree repair tells the person: one line when the repairs worked, and one plain line per repair that could ...
  package_repair_tree_clone.rs  <- src/native/repair/package_repair_tree_clone.rs
    Cloning one directory tree for a worktree repair: every file cloned (or copied when the filesystem cannot clone), ...
  package_repair_cargo.rs  <- src/native/repair/package_repair_cargo.rs
    The Cargo repair: for each Cargo workspace of a new worktree, clone the source workspace's build state when Cargo ...
  package_repair_pnpm.rs  <- src/native/repair/package_repair_pnpm.rs
    The pnpm repair: `pnpm install --offline --frozen-lockfile` in a new worktree whose `node_modules` was copied.
  package_repair_toolchain.rs  <- src/native/repair/package_repair_toolchain.rs
    The toolchain repair: when the launchers copied into a new worktree's `.cache/toolchain-launchers` still name another
  package_repair.rs  <- src/native/repair/package_repair.rs
    The repair of a new worktree's package-manager state: after a rung filled the worktree and the ignored files were
```

### linux-file-and-disk-calls

Package `package/linux-file-and-disk-calls/core-rust`,
crate `monochromatic-linux-file-and-disk-calls`.
Calls the Linux functions for files and disks:
which kind of disk and mount a folder is on,
btrfs folder copies,
cloning a file's data,
swapping two paths in one step,
and opening and removing files without following links.

Depends on crates:
`text-time-and-log-line-formats`.

```text
package/linux-file-and-disk-calls/core-rust/src/
  lib.rs  (index: one `mod` line per folder, no code, no re-exports)
  which_disk_a_folder_is_on/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    why_a_disk_call_failed.rs  <- src/native/subvolume/fs_primitive_error.rs
      The typed failure of one Linux filesystem primitive (`statfs`, a btrfs ioctl, `FICLONE`, `renameat2`)
    which_kind_of_disk_a_folder_is_on.rs  <- src/native/subvolume/fs_kind.rs
      Which kind of filesystem holds a path (btrfs, overlayfs, other), whether a directory is the root of a btrfs
    mounted_disks_and_their_options.rs  <- src/native/subvolume/fs_mount_options.rs
      The mount of a path read from `/proc/self/mountinfo`: its mount point, filesystem type, source and both option
  fast_copies_and_swaps_on_linux/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> which_disk_a_folder_is_on
    btrfs_folder_copy_calls.rs  <- src/native/subvolume/fs_btrfs_ioctl.rs
      The three btrfs ioctls the wrapper needs, called in process: create an empty subvolume, snapshot a subvolume
    cloning_file_data_and_swapping_paths.rs  <- src/native/subvolume/fs_clone_exchange.rs
      Two more Linux primitives: clone a whole file's data blocks into another file (`FICLONE`), and swap two paths
  file_operations_that_never_follow_a_link/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    file_operations_that_never_follow_a_link.rs  <- src/native/subvolume/fs_beneath.rs
      Directory-relative file operations that never follow a symbolic link or cross a mount: open a directory, a file or a
    emptying_a_held_folder_step_by_step.rs  <- src/native/subvolume/subvolume_remove_walk.rs
      Empty a directory that is held open, one descriptor-relative step at a time, refusing every nested subvolume and mount
    removing_a_folder_tree_after_making_folders_writable.rs  <- src/native/subvolume/subvolume_remove.rs
      Remove a directory tree, making each directory the user owns writable on the way down, and never going into a nested
    removing_a_folder_tree_below_a_held_parent_folder.rs  <- src/native/worktree/worktree_trash_remove.rs
      Remove a directory tree beneath a held parent directory: every directory is opened by one plain name below the handle ...
```

### talking-to-the-systemd-user-manager

Package `package/talking-to-the-systemd-user-manager/core-rust`,
crate `monochromatic-talking-to-the-systemd-user-manager`.
Starts and looks up systemd user units by speaking D-Bus to the user manager directly.

Depends on crates:
none.

```text
package/talking-to-the-systemd-user-manager/core-rust/src/
  lib.rs  (index: top doc lists the files below in this order, then one `mod` line each)
  systemd_dbus_wire_format.rs  <- src/native/service/systemd_dbus_wire_format.rs
    The D-Bus wire format the client speaks to the user manager: how a method call is written (header fields, strings
  systemd_dbus.rs  <- src/native/service/systemd_dbus.rs
    A minimal D-Bus client for the user session bus: authenticate as the current user, say `Hello`, and call methods of
```

### cli (the program)

Package `package/git-policy/cli`,
crate `monochromatic-git-policy-cli`.
The command-line program:
decides what each command does and runs it,
with everything Git-related inside it as folders.

Depends on crates:
`dependent-version-bump-check`,
`fix-packages-after-copying-a-folder`,
`linux-file-and-disk-calls`,
`parallel-work-within-a-budget`,
`safe-file-saving`,
`starting-and-watching-programs`,
`talking-to-the-systemd-user-manager`,
`text-time-and-log-line-formats`.

```text
package/git-policy/cli/src/
  lib.rs  (index: one `mod` line per folder, no code, no re-exports)
  environment_variables_for_git_children/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    environment_variables_for_git_children.rs  <- src/native/invocation/child_environment.rs
      The environment variables cli-git adds for every real Git it starts.
  reading_git_settings_from_files/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> environment_variables_for_git_children
    configuration_file_text.rs  <- src/native/repo_read/repo_read_config_text.rs
      A reader of Git configuration file text that gives the entries in file order, and gives up on any spelling it
    configuration_scopes_scan.rs  <- src/native/repo_read/repo_read_config_scan.rs
      The proof, from files alone, that no configuration scope Git reads for repository discovery sets `core.bare` or
    merged_configuration_listing.rs  <- src/native/repo_read/repo_read_config_list.rs
      The merged configuration listing of `git config --list -z --show-scope`, built in process from the system, global
    kinds_of_configuration_values.rs  <- src/native/repo_read/repo_read_config_value_kinds.rs
      The table of which configuration keys Git reads at startup and what kind of value each takes: the word lists, the key
    typed_configuration_values.rs  <- src/native/repo_read/repo_read_config_typed.rs
      The proof that every configuration value Git types at startup, in every scope the readers look at, is a value
  finding_the_git_folders/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> environment_variables_for_git_children, reading_git_settings_from_files
    git_folder_roots.rs  <- src/native/repo_read/git_directory_roots.rs
      The Git directories a command works in, found without starting Git: the Git directory and the common Git directory.
    disk_and_directory_facts.rs  <- src/native/repo_read/repo_read_layout_disk_and_directory_facts.rs
      Small facts about the disk and the directories a layout read needs: a small file's text, whether a directory holds the
    environment_rules_of_repository_discovery.rs  <- src/native/repo_read/repo_read_layout_environment_rules.rs
      The rules about the environment of a Git command: which `GIT_` variables can move repository discovery, which
    repository_location_without_git.rs  <- src/native/repo_read/repo_read_layout.rs
      The repository location that `git rev-parse --path-format=absolute --is-bare-repository --git-dir --git-common-dir
    location_with_options_left_out.rs  <- src/native/repo_read/repo_read_layout_without.rs
      The layout reader's answer for an invocation with some global options and variables left out.
  reading_and_writing_the_index_file_of_git/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    finding_bytes_in_a_slice.rs  <- src/native/index/index_byte_search.rs
      Byte searches over a slice that the index readers share: where a byte first appears, how long a run of one byte is
    cache_tree_extension.rs  <- src/native/index/index_cache_tree.rs
      The `TREE` (cache-tree) index extension: read, written back byte for byte, and invalidated along a path
    compressed_bitmaps.rs  <- src/native/index/index_ewah.rs
      Git's EWAH compressed bitmap (`ewah/ewah_bitmap.c`, `ewah/ewah_io.c`): read from bytes, listed as bit positions
    per_entry_flags.rs  <- src/native/index/index_flags.rs
      A read of the per-entry flags of an index file that decide whether the index may seed an explicit-path commit
    stat_fields_of_an_entry.rs  <- src/native/index/index_stat.rs
      The nine 32-bit stat fields an index entry and the untracked-cache extension store, and the Git varint.
    file_system_monitor_extension.rs  <- src/native/index/index_fsmonitor.rs
      The `FSMN` (fsmonitor) index extension: the last-update token and the bitmap of entries not known to be unchanged
    resolve_undo_extension.rs  <- src/native/index/index_resolve_undo.rs
      The `REUC` (resolve-undo) index extension: read, recorded into when an unmerged entry leaves the index, written back
    untracked_cache_extension.rs  <- src/native/index/index_untracked.rs
      The `UNTR` (untracked cache) index extension as a tree of directories, and its invalidation along a path
    reading_and_writing_the_untracked_cache.rs  <- src/native/index/index_untracked_io.rs
      Reading and writing the `UNTR` index extension (`dir.c`: `read_untracked_extension`, `write_untracked_extension`).
    the_whole_index_in_memory.rs  <- src/native/index/index_full.rs
      The whole content of an index file the in-process index update changes: every entry with its stat data and flags, and ...
    writing_the_whole_index.rs  <- src/native/index/index_full_write.rs
      Writes a `FullIndex` as Git's `do_write_index` does for the plain case: the version Git picks, the entries, the kept
  reading_saved_commits_and_files/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> finding_the_git_folders, reading_and_writing_the_index_file_of_git
    byte_and_prefix_checks_for_short_names.rs  <- src/native/repo_read/object_abbreviation_byte_and_prefix_checks.rs
      Three small checks on bytes and names that the object-name abbreviation uses: a NUL separator, a hexadecimal digit, ...
    object_shapes_and_zlib.rs  <- src/native/repo_read/repo_read_object.rs
      The shapes of a Git object read in process, object names as bytes and hexadecimal, and the zlib step.
    reading_a_pack_file.rs  <- src/native/repo_read/repo_read_pack.rs
      A reader of one Git pack: the version 2 index lookup by object name and the entry read, with `OFS_DELTA` and
    the_object_database.rs  <- src/native/repo_read/repo_read_odb.rs
      The object database of one repository read in process: loose objects, packs and alternates.
    short_object_names.rs  <- src/native/repo_read/object_abbreviation.rs
      The abbreviated name Git prints for an object (`git commit`'s `[main 30b2211]`, `git log --oneline`), computed in
    trees_and_commits.rs  <- src/native/repo_read/repo_read_tree.rs
      Trees and commits read in process: the entries of a tree, the flat list of a whole tree, and the header fields
    difference_of_two_trees.rs  <- src/native/repo_read/repo_read_diff.rs
      The difference between two sorted lists of tree-like entries (a flattened tree, an index), in Git's raw and
    index_entries.rs  <- src/native/repo_read/repo_read_index.rs
      The entries of an index file read in process, for versions 2, 3 and 4.
    tree_against_index_difference.rs  <- src/native/repo_read/repo_read_tree_cached.rs
      The part of a tree that differs from an index, found by walking the tree beside the index's cache-tree: a directory ...
  state_shared_by_preparing_and_landing_a_commit/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    repository_state_of_a_transaction.rs  <- src/native/transaction/transaction_journal.rs
      The values a commit's preparation and the service's landing share about the repository state they work on: the base
  reading_branches_tags_and_markers/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> reading_saved_commits_and_files, state_shared_by_preparing_and_landing_a_commit
    operation_in_progress.rs  <- src/native/conflict/conflict_side_state.rs
      Which operation is in progress in a worktree's Git directory
    ref_file_answers.rs  <- src/native/repo_read/ref_files.rs
      Reads of the files ref storage that answer a question without a Git process
    refs_in_files.rs  <- src/native/repo_read/repo_read_refs.rs
      The files ref storage read in process: one ref by exact name, the symbolic chain behind it, and the list of every ref.
    proof_that_a_marker_names_nothing.rs  <- src/native/repo_read/sequencer_marker.rs
      Proof, without a Git process, that a sequencer marker such as `MERGE_HEAD` names nothing.
  finding_the_real_git_program/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> environment_variables_for_git_children
    remembered_git_version.rs  <- src/native/invocation/git_version_cache.rs
      A private, per-user cache of the text `git version` prints
    deciding_if_a_path_is_real_git.rs  <- src/native/invocation/real_git_candidate.rs
      Decide whether one PATH candidate is real Git
    real_git_the_service_was_set_up_with.rs  <- src/native/invocation/real_git_record.rs
      Read the real Git that the wrapper's commit service was set up with
    the_one_resolution_of_real_git.rs  <- src/native/invocation/real_git_resolution.rs
      The one resolution of the real Git that every start uses
    finding_the_real_git.rs  <- src/native/invocation/real_git.rs
      Find the real Git executable on PATH without ever selecting this wrapper.
    marker_that_git_was_started_by_the_wrapper.rs  <- src/native/invocation/started_as_git.rs
      The marker that every start of the real Git carries
  reading_options_the_way_git_does/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    types_of_option_reading.rs  <- NEW
      The plain types that the four option-reading files share (occurrence, error, spec, value, mode, arity).
    option_values.rs  <- src/native/command/command_options_value.rs
      Decide which bytes
    long_options.rs  <- src/native/command/command_options_long.rs
      Match one `--name[=value]` token against a Git option table as Git 2.56.0 does.
    clusters_of_short_options.rs  <- src/native/command/command_options_short.rs
      Read one `-abc` token as Git 2.56.0 reads a cluster of short options.
    reading_a_command_line_option_by_option.rs  <- src/native/command/command_options.rs
      Read one Git command's arguments the way Git 2.56.0 `parse-options.c` reads them.
    questions_about_typed_options.rs  <- src/native/command/command_options_query.rs
      Small read-only questions over a tokenized Git argument region.
  spellings_that_switch_the_wrapper_off/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    escape_hatch_spellings.rs  <- src/native/invocation/escape_hatch.rs
      The spellings of the wrapper-only escape hatches several modules share.
  what_each_git_command_accepts/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> reading_options_the_way_git_does, spellings_that_switch_the_wrapper_off
    command_add.rs  <- src/native/command/command_add.rs
      The complete `git add` option table of Git 2.56.0 and the bulk-staging facts read
    command_branch_table.rs  <- src/native/command/command_branch_table.rs
      The complete option table of `git branch` in Git 2.56.0.
    command_branch_mode.rs  <- src/native/command/command_branch_mode.rs
      Whether a tokenized `git branch` region creates a branch in Git 2.56.0.
    command_checkout_table.rs  <- src/native/command/command_checkout_table.rs
      The complete option tables of `git checkout` and `git switch` in Git 2.56.0.
    command_branch_target.rs  <- src/native/command/command_branch_target.rs
      Explicit creation and the remote-guess candidate of tokenized `git checkout` and
    command_branch_create.rs  <- src/native/command/command_branch_create.rs
      Branch-creation facts of `git branch`
    command_clean.rs  <- src/native/command/command_clean.rs
      The complete `git clean` option table of Git 2.56.0 and the deletion facts read
    command_commit_table.rs  <- src/native/command/command_commit_table.rs
      The complete option table of `git commit` in Git 2.56.0.
    command_commit.rs  <- src/native/command/command_commit.rs
      Facts about the arguments after `git commit`
    command_config_table.rs  <- src/native/command/command_config_table.rs
      The option groups of `git config` in Git 2.56.0
    command_config.rs  <- src/native/command/command_config.rs
      The form and file scope of a `git config` region
    command_discard.rs  <- src/native/command/command_discard.rs
      Which regions of `git checkout`
    command_push.rs  <- src/native/command/command_push.rs
      The complete `git push` option table of Git 2.56.0 and the facts read from it.
    command_reset.rs  <- src/native/command/command_reset.rs
      The complete `git reset` option table of Git 2.56.0 and the mode read from it.
    command_stash_table.rs  <- src/native/command/command_stash_table.rs
      The option tables of every `git stash` subcommand in Git 2.56.0.
    command_stash.rs  <- src/native/command/command_stash.rs
      Which `git stash` subcommand a region runs in Git 2.56.0
    global_arguments.rs  <- src/native/invocation/global_arguments.rs
      Locate Git 2.56.0 command boundaries without reconstructing native arguments.
    command_status.rs  <- src/native/command/command_status.rs
      The complete `git status` option table of Git 2.56.0
    command_worktree.rs  <- src/native/command/command_worktree.rs
      The one fact the wrapper needs about `git worktree`
    git_builtins.rs  <- src/native/invocation/git_builtins.rs
      The names of the commands built into Git 2.56.0
  the_words_meant_for_this_wrapper/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> reading_options_the_way_git_does, spellings_that_switch_the_wrapper_off, what_each_git_command_accepts
    commands_that_ask_what_this_is.rs  <- src/native/invocation/announced_query.rs
      Recognize the four top-level spellings after which the wrapper says it is not plain Git
    which_worktree_a_command_targets.rs  <- src/native/invocation/effective_target.rs
      Classify the worktree an invocation targets for worktree-enforcing policies.
    names_of_the_built_in_policies.rs  <- src/native/policy/policy_registry.rs
      The fixed registry of policies compiled into the native wrapper.
    control_words_for_the_wrapper.rs  <- src/native/invocation/wrapper_controls.rs
      The wrapper-only control tokens
    removing_control_words_from_a_command.rs  <- src/native/invocation/wrapper_invocation.rs
      Remove every wrapper control from one invocation
  starting_git_and_describing_a_run/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> environment_variables_for_git_children, finding_the_real_git_program, the_words_meant_for_this_wrapper, what_each_git_command_accepts
    starting_git_with_hooks_off.rs  <- src/native/invocation/hooks_off.rs
      Start the real Git with every hook switched off.
    starting_git_with_the_callers_arguments.rs  <- src/native/invocation/forwarding.rs
      Start real Git with the caller's exact arguments
    system_settings_file_of_git.rs  <- src/native/repo_read/git_system_config.rs
      The system configuration file that one real Git executable reads when `GIT_CONFIG_SYSTEM` is not set
    what_a_git_run_asks_and_answers.rs  <- src/native/transaction/transaction_git_request_and_reply_types.rs
      The request that starts one real Git, the reply it gets back, and the failure that names the command
    what_a_command_hands_back.rs  <- NEW
      The two plain values a part hands to the top level instead of running or building them itself: a command to run next, ...
    worktree_command_facts.rs  <- NEW
      The plain facts about a worktree command that forwarding keeps: which worktree, whether it creates one, the add plan ...
  answering_questions_without_starting_git/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> finding_the_git_folders, reading_and_writing_the_index_file_of_git, reading_branches_tags_and_markers, reading_git_settings_from_files, reading_saved_commits_and_files, starting_git_and_describing_a_run
    query_request_answer_and_context.rs  <- src/native/repo_read/repo_read_query_request_answer_and_context.rs
      What the in-process Git queries share: the request (`Query`), the answer (`Answer`), the parsed command line ...
    quiet_diff_index_answer.rs  <- src/native/repo_read/repo_read_query_quiet.rs
      The exit status of `git diff-index --cached --quiet <commit-or-HEAD> [-- <paths>]` without starting Git: 0 when the ...
    reference_and_configuration_answers.rs  <- src/native/repo_read/repo_read_query_refs.rs
      In-process answers for the reference and configuration command lines the wrapper issues
    answers_to_the_wrappers_own_git_queries.rs  <- src/native/repo_read/repo_read_query.rs
      Answers, without a Git process, to the exact read-only Git command lines the wrapper itself issues
    trying_in_process_answers_first.rs  <- src/native/repo_read/repo_read_hook.rs
      The two places where the wrapper's Git runners try the in-process answers first.
  how_long_the_program_waits_for_git_steps/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    kinds_of_git_step.rs  <- src/native/push/child_class.rs
      The two kinds of Git step a commit starts for itself, and the quiet window and the notice grace of each.
    current_step_record.rs  <- NEW
      The step a Git run is in, written by the runner for the thread that runs it and read by the service to tell waiting ...
    how_long_a_push_may_show_nothing.rs  <- src/native/push/push_deadline_and_patience_settings.rs
      How long a push may show nothing before it is stopped: the environment variable that sets the deadline, its default
    longest_waits.rs  <- src/native/service/wait_ceilings.rs
      The longest the wrapper waits for each thing it waits for without a person at the keyboard, with the measurement or ...
  who_owns_a_transaction_folder/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> how_long_the_program_waits_for_git_steps
    process_numbers_in_other_containers.rs  <- src/native/process/pid_namespace.rs
      The PID namespace an owner record names, and the rule for judging an owner that lives in another one.
    owner_record_of_a_folder.rs  <- src/native/transaction/transaction_owner.rs
      The owner record of one commit-transaction directory (`owner.json`
  clearing_leftover_reference_locks/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> finding_the_git_folders, who_owns_a_transaction_folder
    record_of_a_judged_reference_lock.rs  <- src/native/lock/stale_ref_lock_found.rs
      The record of one judged reference lock, and the outcome of one clearing pass.
    identity_of_a_lock_file_when_judged.rs  <- src/native/lock/stale_ref_lock_identity.rs
      A file's identity at the moment it was judged: the removal checks that it is still that file.
    names_of_reference_lock_files.rs  <- NEW
      The suffixes and name rules of reference lock files and their pid files.
    judging_one_reference_lock.rs  <- src/native/lock/stale_ref_lock_judge.rs
      How one reference lock is judged: too new, held, stale on a proof, or unproven.
    finding_reference_lock_files.rs  <- src/native/lock/stale_ref_lock_walk.rs
      Which files below a Git directory are reference locks, and which files go with one.
    setting_aside_and_removing_a_stale_lock.rs  <- src/native/lock/stale_ref_lock_retire.rs
      Judge one candidate and, when it is proven stale, set it aside, check that it is the file that was judged, and remove ...
    clearing_all_stale_reference_locks.rs  <- src/native/lock/stale_ref_locks.rs
      The reference locks a killed Git left behind: found, judged one by one, removed when their holder is proven gone, and
    clearing_locks_before_fetch_and_push.rs  <- src/native/lock/forwarded_lock_notice.rs
      The clearing of stale reference locks for the person's own `git fetch` and `git push`, and the notice that goes before
    words_about_reference_locks.rs  <- src/native/lock/stale_ref_lock_message.rs
      The words the wrapper prints about reference locks: what it removed, what it left and why, and the exact command that
    locks_a_stopped_git_may_leave.rs  <- src/native/lock/stopped_git_locks.rs
      The reference locks a stopped Git child may have left behind, judged and reported once the child and what it started
  recording_where_worktree_add_spent_its_time/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    where_worktree_add_spent_its_time.rs  <- src/native/worktree/add_timing.rs
      The record of where one wrapped `git worktree add` spent its time, step by step, appended to a size-capped file in the
  running_git_for_a_transaction/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> answering_questions_without_starting_git, clearing_leftover_reference_locks, finding_the_real_git_program, how_long_the_program_waits_for_git_steps, recording_where_worktree_add_spent_its_time, starting_git_and_describing_a_run, what_each_git_command_accepts
    which_git_steps_run_again_or_end_early.rs  <- src/native/transaction/transaction_git_step_rerun_and_end_rules.rs
      Which Git steps may be run again after the system refused a process, and which may be ended at a ceiling
    how_long_a_git_step_is_waited_for.rs  <- src/native/transaction/transaction_git_step_bounds.rs
      How long a Git step is waited for and what happens at the bound: ended at the class ceiling, never ended but left
    running_git_for_a_transaction.rs  <- src/native/transaction/transaction_git.rs
      Running real Git for transaction
    the_git_command_a_forward_runs.rs  <- src/native/invocation/forwarded_command.rs
      The Git command a forwarded invocation actually runs
    running_a_read_only_git_query.rs  <- src/native/repo_read/git_metadata.rs
      Run a read-only real-Git query and capture its output as raw bytes.
    checking_the_real_git_version.rs  <- src/native/invocation/git_version.rs
      The real Git version check
  git_settings_read_once/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> running_git_for_a_transaction, starting_git_and_describing_a_run
    which_parts_git_flushes_to_disk.rs  <- src/native/config/config_fsync.rs
      Git's `core.fsync` value read into the set of components it flushes, and the value that keeps every component of a set
    one_read_of_all_git_settings.rs  <- src/native/config/config_snapshot.rs
      One read of the whole Git configuration (`git config --list -z --show-scope`), parsed here
  finding_who_holds_a_lock_file/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    evidence_about_a_foreign_index_lock.rs  <- src/native/lock/index_lock_evidence.rs
      Evidence about a foreign `index.lock` and the verdict it supports
    git_retry_schedule_for_lock_files.rs  <- src/native/lock/index_lock_wait.rs
      Git's own lock-retry schedule: the backoff position, its next step, and the jittered wait it prescribes.
  lines_about_worktrees_and_large_files/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    hiding_passwords_in_urls.rs  <- src/native/lfs/lfs_output_mask.rs
      Hide the credentials inside URLs in the text the wrapper relays from `git-lfs`.
    message_when_a_step_on_a_path_fails.rs  <- src/native/worktree/worktree_path_failure_message.rs
      The one message shape the worktree commands give when a step on a path fails: `<path> <what> (<error>)`.
  naming_the_service_of_one_worktree/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    service_instance.rs  <- src/native/service/service_instance.rs
      Which commit service instance belongs to a worktree: a name made from the worktree's own Git directory, and the
  sending_log_entries_to_the_system_journal/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    where_journal_entries_are_sent.rs  <- src/native/service/journald_native_socket_target.rs
      The address the native journal entries are sent to: the journal's own socket, or a path or abstract name set by a ...
    writing_to_the_system_journal.rs  <- src/native/service/journald_native.rs
      The service's way into the journal: the native journal protocol over a datagram socket when the service's standard
    journal_entry_from_the_wrapper.rs  <- src/native/service/client_journal.rs
      One entry from the wrapper a person ran to the journal, as a datagram on the native socket, sent without waiting and
  log_lines_of_the_commit_service/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> lines_about_worktrees_and_large_files, naming_the_service_of_one_worktree, recording_where_worktree_add_spent_its_time, sending_log_entries_to_the_system_journal, starting_git_and_describing_a_run
    commit_timing.rs  <- src/native/commit/commit_timing.rs
      The record of where one wrapped `git commit` spent its time
    service_timing.rs  <- src/native/service/service_timing.rs
      The lines the resident commit service writes to its log (standard error
    inner_git_words.rs  <- src/native/worktree/inner_git_words.rs
      Where the words of a Git run the wrapper or the service started on its own go, now that none of those runs carries
  reviewing_landed_commits_against_standards/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> how_long_the_program_waits_for_git_steps, running_git_for_a_transaction, starting_git_and_describing_a_run
    review_gateway_credential.rs  <- src/native/service/service_review_key.rs
      The gateway credential of the standards review as it travels from the wrapper to the service: a value that prints as
    changed_line_spans_of_a_file.rs  <- src/native/standards_review/standards_review_patch.rs
      Reading one file's changed new-side line spans out of a `git show --patch` text, so the review needs no
    text_rules_of_the_review_log.rs  <- src/native/standards_review/standards_review_text.rs
      The few text behaviors of the standards-review log that JavaScript defines and Rust does not share
    which_commits_get_a_review.rs  <- src/native/standards_review/standards_review_input.rs
      The pure part of the standards-review shadow run: which landed commits get an entry, and the capped model input
    request_body_of_a_review.rs  <- src/native/standards_review/standards_review_request.rs
      The Decisions API request body the standards-review shadow run sends, and the reading of the gateway's answer into
    review_log_entries.rs  <- src/native/standards_review/standards_review_entry.rs
      The log entries of the standards-review shadow run, rendered as the root `mise.toml` task `shadow-review` writes
    adding_a_review_entry_to_the_log.rs  <- src/native/standards_review/standards_review_append.rs
      How the service adds a review entry to the repository's live shadow log, which every worktree's service instance
    the_one_https_call_of_a_review.rs  <- src/native/standards_review/standards_review_client.rs
      The one HTTPS call of the standards-review shadow run, made in-process and ended only when it shows no sign of work.
    running_a_review_end_to_end.rs  <- src/native/standards_review/standards_review_run.rs
      The standards-review shadow run, end to end and in-process: classify the landed commit's names, read the commit
    git_reads_of_a_review.rs  <- src/native/standards_review/standards_review_source.rs
      The Git reads of the standards-review shadow run, in as few real-Git starts as the data allows: one `git show`
  messages_between_the_wrapper_and_the_commit_service/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> reviewing_landed_commits_against_standards
    service_protocol_messages_and_name_checks.rs  <- src/native/service/service_protocol_messages_and_name_checks.rs
      The messages between the wrapper and the resident commit service as typed values, and the checks on the names they ...
    service_protocol_decode_lines_to_messages.rs  <- src/native/service/service_protocol_decode_lines_to_messages.rs
      Reads one line of the protocol as a client message or a service message, checking every field.
    service_protocol_encode_messages_to_lines.rs  <- src/native/service/service_protocol_encode_messages_to_lines.rs
      Writes a client message or a service message as one JSON object on one line.
    service_protocol.rs  <- src/native/service/service_protocol.rs
      The messages between the wrapper and the resident commit service, one JSON object per line.
  sha_1_and_sha_256_of_bytes/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    sha_1_and_sha_256.rs  <- src/native/subvolume/subvolume_sha.rs
      SHA-1 and SHA-256 of a byte string, written out in full.
  stopping_the_program_at_a_named_step_for_tests/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> how_long_the_program_waits_for_git_steps
    fault_points_the_service_can_be_told_to_hit.rs  <- src/native/service/service_fault.rs
      Test-only fault points of the resident service: `CLI_GIT_TEST_ONLY_SERVICE_FAULT=<point>` ends the service with
    phase_markers_for_crash_tests.rs  <- src/native/transaction/transaction_phase.rs
      Test-only phase markers of the isolated merge and of a commit headed for the service, for crash and interleaving ...
  syncing_files_off_the_waiting_path/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> stopping_the_program_at_a_named_step_for_tests
    syncs_off_the_waiting_path.rs  <- src/native/landing/service_durable.rs
      Where the landing's own syncs wait: not on the path the person waits on, unless the landing publishes nothing.
  holding_the_index_lock_for_the_commit_service/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> finding_who_holds_a_lock_file, how_long_the_program_waits_for_git_steps, log_lines_of_the_commit_service, messages_between_the_wrapper_and_the_commit_service, sha_1_and_sha_256_of_bytes, stopping_the_program_at_a_named_step_for_tests, syncing_files_off_the_waiting_path
    who_a_lock_holder_is.rs  <- src/native/landing/service_index_identity.rs
      Who a marker's holder is
    what_waiting_for_the_lock_found.rs  <- src/native/landing/service_index_found.rs
      What an acquisition of the index lock found when it had to wait
    lock_names_and_failures.rs  <- NEW
      The names and failures the index-lock files share: lock path, announcements, failure of a hold, and the failure code ...
    is_the_lock_still_ours.rs  <- src/native/landing/service_index_ownership.rs
      Whether the file at the path of an index lock is still the file this holder placed there
    watching_the_lock_folder.rs  <- src/native/landing/service_index_watch.rs
      The inotify watch on the directory of an index lock
    putting_the_marked_file_in_place_as_the_lock.rs  <- src/native/landing/service_index_placement.rs
      How the service puts its marked file in place as `index.lock`
    marker_inside_the_held_lock.rs  <- src/native/landing/service_index_marker.rs
      The marker the service writes into the `index.lock` it holds
    staging_files_of_the_lock.rs  <- src/native/landing/service_index_staging.rs
      The staging files of an index lock
    choosing_one_reclaimer_of_a_stale_lock.rs  <- src/native/landing/service_index_claim.rs
      The claim chain: how one reclaimer at a time is chosen to remove a stale index lock, using only files placed whole ...
    removing_a_stale_index_lock.rs  <- src/native/landing/service_index_reclaim.rs
      The removal of a stale index lock by one reclaimer at a time: the reclaimer opens the lock it will judge, reads the ...
    attempts_and_waiting_for_the_index_lock.rs  <- src/native/landing/service_index_attempts.rs
      The attempts and the wait of an acquisition of `index.lock`
    is_the_new_index_the_live_one.rs  <- src/native/landing/service_index_same.rs
      Whether a new index file holds the bytes of the live one.
    holding_the_lock_after_it_is_placed.rs  <- src/native/landing/service_index_hold.rs
      The service's hold on one index lock after it is placed
    marker_of_the_next_batch.rs  <- src/native/landing/service_index_prestage.rs
      The marker of the next batch's index lock
    the_hold_on_the_index_lock.rs  <- src/native/landing/service_index_lock.rs
      The service's hold on Git's own `index.lock` across a landing: taken first, by exclusive creation, waited for without
    waiting_for_the_services_hold.rs  <- src/native/landing/service_index_wait.rs
      The wait of a wrapped command that writes the index for the commit service's own hold on the index: event-driven, ...
  reading_the_bytes_a_commit_would_save/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> answering_questions_without_starting_git, finding_the_git_folders, how_long_the_program_waits_for_git_steps, running_git_for_a_transaction, starting_git_and_describing_a_run
    loading_many_files_through_one_git_process.rs  <- src/native/candidate/blob_batch.rs
      Loading many blobs through one `git cat-file --batch`
    failures_of_reading_candidates.rs  <- src/native/candidate/candidate_error.rs
      The one failure type of the candidate layer
    object_names_and_modes.rs  <- src/native/candidate/candidate_object.rs
      The two Git vocabulary types every candidate carries
    paths_per_git_command.rs  <- src/native/candidate/candidate_pathspec_chunk_size.rs
      The number of paths one Git command of the candidate layer names at a time.
    files_marked_not_text.rs  <- src/native/candidate/candidate_attribute.rs
      Which candidate paths have their `text` Git attribute unset (`-text`, which the `binary` macro includes)
    reading_a_cat_file_batch_reply.rs  <- src/native/candidate/candidate_batch.rs
      Read one `git cat-file --batch` reply from a byte stream
    git_runs_of_a_prediction.rs  <- src/native/candidate/candidate_prediction_git_runs.rs
      The Git runs a candidate prediction makes: one captured command with its failure messages, and the path of the
    private_copy_of_the_index.rs  <- src/native/candidate/candidate_private_index.rs
      A private copy of the real index
    reading_diff_records.rs  <- src/native/candidate/candidate_record.rs
      Parse the NUL-delimited raw records of `git diff-index` and `git diff-tree`.
    reading_stage_records.rs  <- src/native/candidate/candidate_stage.rs
      Parse the NUL-delimited stage records of `git ls-files --stage -z`
    one_long_lived_cat_file_process.rs  <- src/native/candidate/candidate_reader.rs
      One long-lived `git cat-file --batch` process that answers every object read of an invocation.
    one_candidate_version.rs  <- src/native/candidate/candidate_version.rs
      An immutable candidate version
    owner_of_candidate_versions.rs  <- src/native/candidate/candidate_store.rs
      The per-invocation owner of candidate versions and candidate bytes.
    candidates_that_need_a_private_index.rs  <- src/native/candidate/candidate_prediction.rs
      The two candidate sources that need a private index
    reading_tree_listings.rs  <- src/native/candidate/candidate_tree.rs
      Parse the NUL-delimited records of `git ls-tree -r -z --full-tree <commit>`: every file of one commit with its mode ...
    listings_of_one_commits_tree.rs  <- src/native/candidate/candidate_store_commit_listings.rs
      The listings of one commit's tree, of the parents of a set of commits, and of a tree's blobs, as a second block
  putting_corrected_files_into_the_private_index/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> reading_the_bytes_a_commit_would_save, running_git_for_a_transaction
    putting_corrections_into_the_private_index.rs  <- src/native/policy/direct_fix_install.rs
      Install the corrections a converged `git cli-git fix` settled on into the
  rules_that_add_flags_to_a_command/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> reading_options_the_way_git_does, what_each_git_command_accepts
    rule_add_names_explicitly.rs  <- src/native/command/rule_add_explicit.rs
      Pure decision of the add-explicit policy
    adding_words_to_a_command_line.rs  <- src/native/command/rule_argument_rewrite.rs
      The shared result and builder of fixed transforms that add tokens to a command line.
    rule_push_all_or_nothing.rs  <- src/native/command/rule_atomic_push.rs
      The pure decision of the atomic-push transform
    rule_status_hints.rs  <- src/native/command/rule_status_hints.rs
      The pure decision of the status-hints transform
  what_the_wrapper_asks_git_about_a_repository/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> answering_questions_without_starting_git, finding_the_git_folders, running_git_for_a_transaction, starting_git_and_describing_a_run, the_words_meant_for_this_wrapper
    asking_git_which_worktree_this_is.rs  <- src/native/worktree/worktree_identity.rs
      Ask real Git which repository and worktree an invocation selects.
    asking_git_where_the_repository_is.rs  <- src/native/repo_read/repository_location.rs
      Ask real Git
    answer_shapes_built_from_a_repository_layout.rs  <- src/native/repo_read/repo_read_adapter.rs
      The shapes the wrapper's existing queries answer with, built from an in-process `RepoLayout`.
    where_the_real_index_lives.rs  <- src/native/landing/landing_location.rs
      Where a forwarded command's real index lives
  rules_about_what_a_commit_may_contain/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> reading_options_the_way_git_does, rules_that_add_flags_to_a_command, what_each_git_command_accepts, what_the_wrapper_asks_git_about_a_repository
    why_a_push_scope_is_unknown.rs  <- src/native/push/manual_push_cause.rs
      Why a manual push's scope could not be established, as a cause the hint is chosen by.
    is_the_index_the_same_as_head.rs  <- src/native/commit/rule_commit_index.rs
      The pure core of the index-against-`HEAD` check used by the commit-only transform.
    messages_of_the_commit_only_rule.rs  <- src/native/commit/rule_commit_only_message.rs
      The user-facing diagnostics of the commit-only transform.
    rule_commit_only_what_was_named.rs  <- src/native/commit/rule_commit_only.rs
      The pure decision of the commit-only transform
    is_a_merge_or_rebase_in_progress.rs  <- src/native/commit/rule_commit_sequencer.rs
      The pure core of the "is a merge
  rules_about_where_a_command_may_run/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> answering_questions_without_starting_git, reading_options_the_way_git_does, spellings_that_switch_the_wrapper_off, the_words_meant_for_this_wrapper, what_each_git_command_accepts
    rule_branch_worktree_only.rs  <- src/native/command/rule_branch_worktree.rs
      Pure decision of the branch-worktree-only policy
    rejection_texts_of_the_linked_worktree_rule.rs  <- src/native/command/rule_linked_worktree_messages.rs
      The rejection texts of the linked-worktree policy: outside any worktree, in the main worktree, and in a
    does_this_worktree_hold_uncommitted_state.rs  <- src/native/command/rule_worktree_dirty.rs
      The question "does this worktree hold uncommitted state?" as one Git query and its reading.
    rule_linked_worktree_only.rs  <- src/native/command/rule_linked_worktree.rs
      Pure decision of the linked-worktree-only policy
    rule_require_root.rs  <- src/native/command/rule_require_root.rs
      Pure decision of the require-root policy
  the_settings_file_cli_git_config_jsonc/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> the_words_meant_for_this_wrapper, what_each_git_command_accepts, what_the_wrapper_asks_git_about_a_repository
    rejected_settings.rs  <- src/native/config/config_error.rs
      The one failure type for rejected `cli-git.config.jsonc` content or files.
    typed_readers_over_json_values.rs  <- src/native/config/config_values.rs
      Typed readers over parsed JSONC values
    discarded_copies_settings.rs  <- src/native/conflict/config_discarded.rs
      The `discardedCopies` section of `cli-git.config.jsonc`: how long and how much the folder of saved copies of ...
    typed_settings.rs  <- src/native/config/config_schema.rs
      The typed settings a validated `cli-git.config.jsonc` produces
    checking_the_markdown_policy_options.rs  <- src/native/config/config_policies_markdown.rs
      Validation of the options object of the `markdown/autofix` policy and of its `rules` list.
    the_rules_file_option.rs  <- src/native/config/config_rules_file.rs
      The `rulesFile` option of `security/forbidden-strings`
    checking_the_policies_section.rs  <- src/native/config/config_policies.rs
      Validation of the `policies` section and of each policy's options.
    turning_settings_text_into_typed_settings.rs  <- src/native/config/config_parse.rs
      Turn `cli-git.config.jsonc` text into validated typed settings.
    finding_and_reading_the_settings_file.rs  <- src/native/config/config_file.rs
      Find and read `cli-git.config.jsonc` at a repository root.
    deciding_how_to_load_settings.rs  <- src/native/config/config_loading.rs
      Policy-configuration loading decisions before expensive policy initialization.
    settings_of_the_worktree_a_command_runs_in.rs  <- src/native/config/invocation_config.rs
      Load the policy configuration of the worktree one invocation runs in
  scanning_those_bytes_for_forbidden_strings/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> putting_corrected_files_into_the_private_index, reading_the_bytes_a_commit_would_save, running_git_for_a_transaction, the_settings_file_cli_git_config_jsonc
    the_forbidden_strings_scanner.rs  <- src/native/candidate/scanner_adapter.rs
      The repository's forbidden-strings scanner
    which_rules_files_the_scanner_loads.rs  <- src/native/candidate/scanner_selection.rs
      Which rules files the scanner loads
    content_the_parent_commit_already_has.rs  <- src/native/candidate/candidate_known_content.rs
      Which candidates hold content that a parent commit's tree already holds
    scanning_one_version.rs  <- src/native/candidate/scanner_run.rs
      Scan every eligible candidate of one version through the loaded scanner.
    failure_codes_of_scanning.rs  <- src/native/candidate/scanner_failure_code.rs
      Which engine failure code each failure of the candidate and scanner layers carries.
  running_policies_and_settling_their_corrections/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> answering_questions_without_starting_git, finding_the_git_folders, how_long_the_program_waits_for_git_steps, putting_corrected_files_into_the_private_index, reading_options_the_way_git_does, reading_the_bytes_a_commit_would_save, rules_about_what_a_commit_may_contain, rules_about_where_a_command_may_run, rules_that_add_flags_to_a_command, running_git_for_a_transaction, scanning_those_bytes_for_forbidden_strings, starting_git_and_describing_a_run, the_settings_file_cli_git_config_jsonc, the_words_meant_for_this_wrapper, what_the_wrapper_asks_git_about_a_repository
    messages_as_json_lines.rs  <- src/native/policy/diagnostics.rs
      Render wrapper diagnostics as the JSONL events callers already parse.
    hints_after_a_finding.rs  <- src/native/policy/event_hints.rs
      The hint that follows a finding, an engine failure or a warn-unsafe notice: a guess at what the person was
    repository_path_in_an_event.rs  <- src/native/policy/event_path.rs
      A repository path as an event reports it
    may_a_fix_change_an_unselected_file.rs  <- src/native/dependent_version/policy_added_path.rs
      Whether a direct fix may change a tracked file it did not select, and the `patch-conflict` message when it may not.
    points_at_which_policies_run.rs  <- src/native/policy/policy_trigger.rs
      The lifecycle points at which policies run
    events_of_a_policy_pass.rs  <- src/native/policy/policy_events.rs
      The events one policy pass produces
    running_policies_in_order.rs  <- src/native/policy/policy_engine.rs
      Run shipped policies in registry order for one lifecycle point and settle the
    repository_facts_policies_need.rs  <- src/native/repo_read/repository_facts.rs
      The repository facts policies and fixed transforms need
    content_offered_to_policies.rs  <- src/native/policy/policy_content.rs
      What one lifecycle offers the content policies
    repeating_a_pass_until_it_settles.rs  <- src/native/policy/policy_convergence.rs
      The bounded loop that repeats a whole policy pass after corrections changed the
    fixed_changes_to_a_command.rs  <- src/native/policy/policy_transforms.rs
      The fixed argument transforms of a forwarded command
    work_not_ported_yet.rs  <- src/native/policy/unported.rs
      The work of the installed cli-git that this executable does not do yet
  work_left_for_the_service_to_finish_later/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> log_lines_of_the_commit_service
    claim_time.rs  <- src/native/landing/claim_time.rs
      A reading of the clocks a claim's lease is judged by: the machine and the boot the host is in, the host's monotonic ...
    work_item_state_directories_and_claim_names.rs  <- src/native/landing/work_item_state_directories_and_claim_names.rs
      The folders an item moves through (`pending`, `claimed`, `recovering`, `done`, `failed`), the names of the claim files
    work_item.rs  <- src/native/landing/work_item.rs
      The durable work-item record: a small file in the Git directory that says "this landed object needs this
    work_item_recovery.rs  <- src/native/landing/work_item_recovery.rs
      The recovery half of the work-item record
  asking_the_commit_service_to_save_a_commit/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> finding_the_git_folders, git_settings_read_once, log_lines_of_the_commit_service, messages_between_the_wrapper_and_the_commit_service, naming_the_service_of_one_worktree, reading_saved_commits_and_files, reading_the_bytes_a_commit_would_save, recording_where_worktree_add_spent_its_time, reviewing_landed_commits_against_standards, running_git_for_a_transaction, running_policies_and_settling_their_corrections, starting_git_and_describing_a_run, stopping_the_program_at_a_named_step_for_tests, the_settings_file_cli_git_config_jsonc, work_left_for_the_service_to_finish_later
    landed_summary.rs  <- src/native/landing/landed_summary.rs
      The one summary line a landed commit prints, the way `git commit` prints it: `[<branch> <name>] <subject>`, with
    service_client_listen_and_ask_to_retire.rs  <- src/native/service/service_client_listen_and_ask_to_retire.rs
      The lowest steps of the client of the commit service: connecting to the service's address, reading one line as a ...
    service_messages.rs  <- src/native/landing/service_messages.rs
      Everything the wrapper tells the user when it cannot get a commit landed by the resident service, and the notices
    service_provision_unit_file_ownership_and_writes.rs  <- src/native/service/service_provision_unit_file_ownership_and_writes.rs
      The checks and writes on the unit files of the commit service: which files carry the wrapper's marker, whether any
    service_provision.rs  <- src/native/service/service_provision.rs
      The wrapper makes socket activation for a worktree's commit service exist the first time a commit needs it: it writes
    service_client_settings_and_watches.rs  <- src/native/service/service_client_settings_and_watches.rs
      The settings of one client run (how it reaches the service, how long it waits, what it keeps fresh) and the watch it ...
    service_client_reach_service.rs  <- src/native/service/service_client_reach_service.rs
      Reaches the commit service: connects to its address, starts it through its unit when nothing answers, and waits for a
    service_client.rs  <- src/native/service/service_client.rs
      The wrapper's side of a commit through the resident service: connect, wait honestly while the operating system
    service_handoff.rs  <- src/native/landing/service_handoff.rs
      The wrapper's side of after-commit work: leave the push and upkeep records for a commit the service landed and wake
    service_pruned.rs  <- src/native/landing/service_pruned.rs
      How a commit headed for the service notices that the content it prepared was removed from the repository before it
    service_route.rs  <- src/native/landing/service_route.rs
      How a commit reaches the resident service: already listening, or set up first by the wrapper.
  running_git_push_until_it_ends_or_stalls/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> answering_questions_without_starting_git, clearing_leftover_reference_locks, finding_the_git_folders, finding_the_real_git_program, how_long_the_program_waits_for_git_steps, starting_git_and_describing_a_run
    one_push_attempt_and_its_failure.rs  <- NEW
      One push attempt and the failure of a push that ran alone.
    how_to_change_the_push_deadline.rs  <- src/native/push/push_deadline_advice.rs
      The words that tell a person how to change the push deadline, for the program that is running the push.
    waiting_for_a_push_to_end.rs  <- NEW
      Waiting for a push child to end, ending it, and the one bounded push run.
    one_git_query_that_ends_when_idle.rs  <- src/native/push/bounded_query.rs
      Run one read-only Git query that is ended only when it shows no sign of work, with separate output streams.
    running_one_push_at_a_time.rs  <- src/native/push/auto_push_single_flight.rs
      The service's Git for a push: the real Git, its working directory and environment, the command builder, the
    clearing_locks_around_a_push.rs  <- src/native/lock/push_lock_clearing.rs
      The wrapper's own fetch and push, run after the reference locks of a killed Git in their repository have been judged
    pushing_until_it_ends_or_stalls.rs  <- src/native/push/bounded_push.rs
      Run the auto-push's real `git push` (and its fetch and Git LFS upload) until it ends, or stops showing any sign of ...
  the_hidden_second_git_folder/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> finding_the_git_folders, git_settings_read_once, log_lines_of_the_commit_service, reading_branches_tags_and_markers, running_git_for_a_transaction, sending_log_entries_to_the_system_journal, starting_git_and_describing_a_run, state_shared_by_preparing_and_landing_a_commit, what_the_wrapper_asks_git_about_a_repository
    empty_tree.rs  <- src/native/shadow/empty_tree.rs
      The object name of the empty tree under each object format Git supports
    capture_invocation.rs  <- src/native/commit/capture_invocation.rs
      The transaction facts recorded before any Git mutation
    shadow_git.rs  <- src/native/shadow/shadow_git.rs
      Git commands run against a transaction's shadow repository
    conclusion_cleanup.rs  <- src/native/commit/conclusion_cleanup.rs
      Native conclusion cleanup reproduced in the owning worktree's Git directory after a
    conclusion_copy.rs  <- src/native/commit/conclusion_copy.rs
      The merge, cherry-pick and revert conclusion state copied into the shadow at preparation
    shadow_config.rs  <- src/native/shadow/shadow_config.rs
      The shadow repository's `config` file, written by cli-git itself
    shadow_links.rs  <- src/native/shadow/shadow_links.rs
      Links every non-private entry of the real common Git directory into a shadow repository
    shadow_refs.rs  <- src/native/shadow/shadow_refs.rs
      The shadow ref store: a snapshot of every real ref
    shadow_skeleton.rs  <- src/native/shadow/shadow_skeleton.rs
      The files and directories of an empty bare repository on the files ref backend, created in-process.
    shadow_create.rs  <- src/native/shadow/shadow_create.rs
      Creates the shadow repository of one published transaction
  saving_copies_of_what_a_command_would_throw_away/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> finding_the_git_folders, messages_between_the_wrapper_and_the_commit_service, reading_branches_tags_and_markers, reading_options_the_way_git_does, reading_saved_commits_and_files, recording_where_worktree_add_spent_its_time, rules_about_where_a_command_may_run, running_git_for_a_transaction, starting_git_and_describing_a_run, the_hidden_second_git_folder, the_settings_file_cli_git_config_jsonc, the_words_meant_for_this_wrapper, what_each_git_command_accepts
    autostash_carry.rs  <- src/native/conflict/autostash_carry.rs
      Carrying back what Git's apply of a merge's autostash left in the commit's scratch repository when the apply ...
    command_conflict_side.rs  <- src/native/conflict/command_conflict_side.rs
      Which `git checkout` and `git restore` command lines take one side of a conflicted file
    conflict_markers.rs  <- src/native/conflict/conflict_markers.rs
      Which committed files still hold conflict marker lines, read from the committed content, and the one line that tells a
    conflict_side_decision.rs  <- src/native/conflict/conflict_side_decision.rs
      Whether every token of a command line names a conflicted file
    conflict_side_modes.rs  <- src/native/conflict/conflict_side_modes.rs
      The pathspec modes and the pathspec magic that make a conflict-side line unreadable to the guard
    conflict_side_notice.rs  <- src/native/conflict/conflict_side_notice.rs
      The plain-language notices of a conflict-side command
    conflict_side_source.rs  <- src/native/conflict/conflict_side_source.rs
      Finding the worktree file a conflict-side command is about to replace
    conflict_sides.rs  <- src/native/conflict/conflict_sides.rs
      The saved copy of each file a conflict-side command replaces
    discard_audit.rs  <- src/native/conflict/discard_audit.rs
      The append-only record of every command form that can discard uncommitted state: rejected, allowed because the
    discarded_prune_limits.rs  <- src/native/conflict/discarded_prune_limits.rs
      The two limits of the folder of saved copies, their defaults, and the reading of a size such as `256m`.
    discarded_store_records.rs  <- src/native/conflict/discarded_store_records.rs
      The plain values the descriptor-based reader of the folder of saved copies hands around: which directory was seen
    discarded_prune_stamps.rs  <- src/native/conflict/discarded_prune_stamps.rs
      The time stamps in the names of saved-copy directories, and the entry that carries one.
    discarded_prune_plan.rs  <- src/native/conflict/discarded_prune_plan.rs
      Which saved-copy directories a pass removes, and why: the ones past the age limit, then the oldest until the folder ...
    discarded_prune_audit_line.rs  <- src/native/conflict/discarded_prune_audit_line.rs
      The audit line one removed directory of saved copies leaves: the outcome, the reason, the limit, the sizes and the ...
    discarded_prune_problems.rs  <- src/native/conflict/discarded_prune_problems.rs
      What can go wrong in a pass over the folder of saved copies, the words that say it, and the listing that carries both.
    discarded_store.rs  <- src/native/conflict/discarded_store.rs
      The folder of saved copies (`<common git dir>/cli-git/discarded`) opened once without following a link, and every
    discarded_prune.rs  <- src/native/conflict/discarded_prune.rs
      The bound on `<common git dir>/cli-git/discarded/`: copies older than a number of days are removed, and the folder is
    conflict_side_guard.rs  <- src/native/conflict/conflict_side_guard.rs
      The linked-worktree policy's carve-out for conflicted files
    discard_audit_generations.rs  <- src/native/conflict/discard_audit_generations.rs
      The history of `cli-git-audit.jsonl`: where a rotated file goes, which generations may be removed, and the pass that
  settings_that_decide_how_the_index_is_read/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> reading_git_settings_from_files
    index_extension_settings.rs  <- src/native/index/index_settings.rs
      The configuration that decides what Git does to the extensions of an index it reads and writes, and whether the
  waiting_for_the_index_lock_of_git/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> finding_who_holds_a_lock_file, holding_the_index_lock_for_the_commit_service, how_long_the_program_waits_for_git_steps, running_git_for_a_transaction, starting_git_and_describing_a_run, the_words_meant_for_this_wrapper
    waiting_for_any_programs_hold_on_the_index_lock.rs  <- src/native/lock/index_holder_wait.rs
      The wait of a command that is about to change the checkout for any program's hold on Git's `index.lock`: the commit
    does_a_command_write_the_index.rs  <- src/native/lock/index_writer_commands.rs
      Whether a resolved forwarded command writes the real index.
  landing_commits/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> answering_questions_without_starting_git, asking_the_commit_service_to_save_a_commit, finding_the_git_folders, git_settings_read_once, holding_the_index_lock_for_the_commit_service, how_long_the_program_waits_for_git_steps, log_lines_of_the_commit_service, messages_between_the_wrapper_and_the_commit_service, naming_the_service_of_one_worktree, reading_and_writing_the_index_file_of_git, reading_git_settings_from_files, reading_saved_commits_and_files, running_git_for_a_transaction, running_git_push_until_it_ends_or_stalls, saving_copies_of_what_a_command_would_throw_away, sending_log_entries_to_the_system_journal, settings_that_decide_how_the_index_is_read, sha_1_and_sha_256_of_bytes, starting_git_and_describing_a_run, state_shared_by_preparing_and_landing_a_commit, stopping_the_program_at_a_named_step_for_tests, syncing_files_off_the_waiting_path, the_hidden_second_git_folder, waiting_for_the_index_lock_of_git
    index_reset.rs  <- src/native/index/index_reset.rs
      `git reset <tree> -- <paths>` on an index held in memory: for every path the literal pathspecs name, the entries of ...
    index_reset_publish.rs  <- src/native/index/index_reset_publish.rs
      How the in-process index update gets its result onto the landing's private index: the name of that index, a temporary ...
    index_reset_tree.rs  <- src/native/index/index_reset_tree.rs
      The tree questions an in-process path reset asks: a commit's root tree, the entries of that tree at or below given
    service_land_copy.rs  <- src/native/landing/service_land_copy.rs
      The landing's private copy of the live index: a directory this landing made for itself beside the live index, holding ...
    index_reset_file.rs  <- src/native/index/index_reset_file.rs
      The landing's index update without a Git process: read the private index file, reset the commit's paths to the commit
    maintenance_decision.rs  <- src/native/landing/maintenance_decision.rs
      The decision behind automatic maintenance after a landing
    maintenance_run.rs  <- src/native/landing/maintenance_run.rs
      Running automatic maintenance after a landing
    service_built_checks.rs  <- src/native/landing/service_built_checks.rs
      The two questions the landing asks about a commit that Git built (a signed commit), answered from the object store ...
    service_busy_answers_and_name_checks.rs  <- src/native/service/service_busy_answers_and_name_checks.rs
      The shapes the landing board answers with (an early answer, a group admission, a naming, a decision) and the checks ...
    service_journal_intent_text_and_parse.rs  <- src/native/service/service_journal_intent_text_and_parse.rs
      The intent of a batch: the record of what a landing is about to do, written as `key=value` lines and read back, with ...
    service_land_start.rs  <- src/native/landing/service_land_start.rs
      The witness of the live index file as the landing found it under the index lock, recorded in the journal, and the ...
    service_journal.rs  <- src/native/service/service_journal.rs
      The resident service's journal: one file per batch of commits, with the one durable write the batch needs, and the
    service_busy.rs  <- src/native/service/service_busy.rs
      What the commit service's one landing thread is doing, shared with the threads that talk to waiting clients: the ...
    service_busy_step_record_on_this_thread.rs  <- src/native/service/service_busy_step_record_on_this_thread.rs
      The record a thread keeps of the write it is inside when that write is never ended: which landing board the thread
    service_captures.rs  <- src/native/landing/service_captures.rs
      The service's memory of when the bytes of the commits it landed were captured, kept per worktree.
    service_head.rs  <- src/native/landing/service_head.rs
      Whether a worktree's head still names the branch a commit was prepared on.
    service_land_context.rs  <- src/native/landing/service_land_context.rs
      What the commit service needs to land a batch: how to run Git, where the journal lives, a test seam and the lock ...
    service_land_follow.rs  <- src/native/landing/service_land_follow.rs
      What the service's landing does once the branch has moved: the worktree's index entries of the commit's paths and the
    service_land_finish.rs  <- src/native/landing/service_land_finish.rs
      What follows the branch move of a landing and of a restart: the conclusion state and the index update, and the ...
    service_land_git_steps_and_import_stream.rs  <- src/native/landing/service_land_git_steps_and_import_stream.rs
      The Git steps of a landing and the text it feeds Git: running one step and telling a step ended at its ceiling from ...
    service_land_objects_of_batch.rs  <- src/native/landing/service_land_objects_of_batch.rs
      The objects a batch lands: the commit Git built, checked for its parents and its objects, or the commits ...
    service_land_take_index_lock.rs  <- src/native/landing/service_land_take_index_lock.rs
      Takes the index lock for a batch that is ready to land, notes the wait in the timing and the log, and cleans up when ...
    service_land.rs  <- src/native/landing/service_land.rs
      The service's serialized step: land a batch of prepared commits on a branch, and finish the batches a killed
  converting_the_main_checkout/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> how_long_the_program_waits_for_git_steps, reading_git_settings_from_files, sha_1_and_sha_256_of_bytes, starting_git_and_describing_a_run, who_owns_a_transaction_folder
    subvolume_error.rs  <- src/native/subvolume/subvolume_error.rs
      The failure type of the subvolume migration and the test hook that simulates a kill.
    single_owner_claim.rs  <- src/native/lock/single_owner_claim.rs
      The right of one running command to work on one item (a worktree being converted, a checkout being migrated), held
    subvolume_anchor.rs  <- src/native/subvolume/subvolume_anchor.rs
      An open directory used as the fixed point of every operation inside it: the directory is opened one component at
    subvolume_arguments.rs  <- src/native/subvolume/subvolume_arguments.rs
      The grammar of `git cli-git migrate-to-subvolume`, the wrapper's command that converts the main checkout into a
    subvolume_index.rs  <- src/native/subvolume/subvolume_index.rs
      The stat rewrite of a Git index: the bytes of an index file in, the bytes of the same index out, with the stat
    subvolume_file_write_steps.rs  <- src/native/subvolume/subvolume_file_write_steps.rs
      Two file steps that the migration's writers share: write bytes and flush them to disk, and flush a directory.
    subvolume_index_install_into_new_worktree.rs  <- src/native/subvolume/subvolume_index_install_into_new_worktree.rs
      The install of a first index into a new worktree: written into `index.lock`, linked under the name `index` without
    subvolume_journal.rs  <- src/native/subvolume/subvolume_journal.rs
      The journal of a subvolume migration: a small JSON file beside the checkout that records which steps finished
    subvolume_lock.rs  <- src/native/subvolume/subvolume_lock.rs
      The index locks the migration holds: one `index.lock` file for the main index and one for the index of each linked
    subvolume_index_files.rs  <- src/native/subvolume/subvolume_index_files.rs
      The linked worktrees of a main checkout, found from `.git/worktrees/*/gitdir`, and the stat rewrite applied to the
    subvolume_platform.rs  <- src/native/subvolume/subvolume_platform.rs
      The operating-system operations the subvolume migration needs, behind one trait, and the Linux implementation
    subvolume_entry_count.rs  <- src/native/subvolume/subvolume_entry_count.rs
      The count of the entries under a checkout: every file, directory and link below its root, counted once each.
    subvolume_preflight.rs  <- src/native/subvolume/subvolume_preflight.rs
      The paths of one migration and the refusals that stop it before anything is created: every refusal names its cause
    subvolume_sync_attrs.rs  <- src/native/subvolume/subvolume_sync_attrs.rs
      Copying the attributes of one directory entry onto its copy: owner, permission bits, access and modification
    subvolume_sync_pass_state_and_checks.rs  <- src/native/subvolume/subvolume_sync_pass_state_and_checks.rs
      The pieces every step of a tree sync shares: the options and the statistics, the work stack's entries, the open ...
    subvolume_sync.rs  <- src/native/subvolume/subvolume_sync.rs
      Make one directory tree match another, in process: copy what is missing or changed (cloning file data where the
    subvolume_run.rs  <- src/native/subvolume/subvolume_run.rs
      The state one migration command carries between its steps: the platform, the paths, the journal, the progress
    subvolume_migrate.rs  <- src/native/subvolume/subvolume_migrate.rs
      Convert a main checkout into a btrfs subvolume at the same path, in process, resumably.
    subvolume_rollback.rs  <- src/native/subvolume/subvolume_rollback.rs
      Undo a migration (`--rollback`) and delete the old tree after a finished one (`--remove-old`), plus the removal of
    subvolume_command.rs  <- src/native/subvolume/subvolume_command.rs
      `git cli-git migrate-to-subvolume`: find the repository's main checkout from the working directory, run the
    subvolume_sync_copy_one_entry.rs  <- src/native/subvolume/subvolume_sync_copy_one_entry.rs
      The copying of one source entry (a file, a symbolic link or a named pipe) into the destination, as methods of `Pass`.
  planning_a_worktree_add_and_what_git_made/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> git_settings_read_once, how_long_the_program_waits_for_git_steps, lines_about_worktrees_and_large_files, starting_git_and_describing_a_run, what_each_git_command_accepts
    checkout_workers.rs  <- src/native/worktree/checkout_workers.rs
      Whether the checkout of a new worktree may be given Git's parallel checkout, and the option that does it.
    worktree_add_binding.rs  <- src/native/worktree/worktree_add_binding.rs
      The binding of one `git worktree add` to the objects real Git made for it (the destination directory and the
    worktree_add_plan.rs  <- src/native/worktree/worktree_add_plan.rs
      The plan for one `git worktree add` the cheap-worktree ladder may serve: where the new worktree goes and where in
  copying_ignored_files_into_a_stage/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> planning_a_worktree_add_and_what_git_made, recording_where_worktree_add_spent_its_time, running_git_for_a_transaction, starting_git_and_describing_a_run, the_words_meant_for_this_wrapper, who_owns_a_transaction_folder
    worktree_copy_error.rs  <- src/native/worktree_copying/worktree_copy_error.rs
      The one failure type of the linked-worktree ignored-state copy
    worktree_copy_prune.rs  <- src/native/worktree_copying/worktree_copy_prune.rs
      What a copy from a main checkout leaves out of the ignored state it hands over
    worktree_copy_selection.rs  <- src/native/worktree_copying/worktree_copy_selection.rs
      Which source paths the linked-worktree copy selects
    worktree_copy_manifest.rs  <- src/native/worktree_copying/worktree_copy_manifest.rs
      The deterministic, parent-first list of entries a copy transfers
    worktree_copy_compare.rs  <- src/native/worktree_copying/worktree_copy_compare.rs
      Exact comparison of one manifest entry against the filesystem
    worktree_copy_git_steps.rs  <- NEW
      The plain description of a Git step, its status and its runner that the copy and the ladder both use.
    worktree_copy_stage_leaf_copying.rs  <- src/native/worktree_copying/worktree_copy_stage_leaf_copying.rs
      The copying of a staged snapshot's files and links from the source into the stage, one worker per slice of the ...
    worktree_copy_install_entry.rs  <- src/native/worktree_copying/worktree_copy_install_entry.rs
      The exclusive creation of one destination entry from the private stage
    worktree_copy_rollback.rs  <- src/native/worktree_copying/worktree_copy_rollback.rs
      Removal of the paths one failed installation created
    worktree_copy_install.rs  <- src/native/worktree_copying/worktree_copy_install.rs
      Installation of one validated stage into a newly registered worktree
    worktree_copy_journal_encoding_and_validation.rs  <- src/native/worktree_copying/worktree_copy_journal_encoding_and_validation.rs
      The exact JSON encoding of one destination's linked-worktree copy journal, and the strict reading and validation of ...
    worktree_copy_owner.rs  <- src/native/worktree_copying/worktree_copy_owner.rs
      The process that owns one item of a copy transaction (a journal, a stage claim, a temporary file), written into the
    worktree_copy_stage.rs  <- src/native/worktree_copying/worktree_copy_stage.rs
      The private, validated copy of one source's ignored state
    worktree_copy_journal.rs  <- src/native/worktree_copying/worktree_copy_journal.rs
      The durable journal of one destination's linked-worktree copy
    worktree_copy_install_log.rs  <- src/native/worktree_copying/worktree_copy_install_log.rs
      The append-only install log inside one transaction's private stage
    worktree_copy_journal_removal.rs  <- src/native/worktree_copying/worktree_copy_journal_removal.rs
      The removal of a finished or abandoned copy transaction's stage and journal.
    worktree_copy_stage_claim.rs  <- src/native/worktree_copying/worktree_copy_stage_claim.rs
      The durable claim on a stage container's name, written before the container exists
    worktree_copy_stage_claim_reaping.rs  <- src/native/worktree_copying/worktree_copy_stage_claim_reaping.rs
      The judging of stage claims and their temporaries, and the removal of the stages that interrupted copies and ...
    worktree_copy_transaction.rs  <- src/native/worktree_copying/worktree_copy_transaction.rs
      One destination's journaled installation to an end, and the synchronization of every created worktree.
    worktree_copy_recovery.rs  <- src/native/worktree_copying/worktree_copy_recovery.rs
      Recovery of interrupted linked-worktree copies: every pending journal is resumed, completed, or discarded
    worktree_copy_registry.rs  <- src/native/worktree_copying/worktree_copy_registry.rs
      The linked worktree a `git worktree add` registered at the destination its command line names
  is_the_worktree_clean/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> converting_the_main_checkout, finding_the_git_folders, lines_about_worktrees_and_large_files, reading_branches_tags_and_markers, reading_git_settings_from_files, reading_saved_commits_and_files, recording_where_worktree_add_spent_its_time, running_git_for_a_transaction, sha_1_and_sha_256_of_bytes, starting_git_and_describing_a_run
    worktree_attributes.rs  <- src/native/worktree/worktree_attributes.rs
      Git's attribute-file lexer and pattern matcher over raw bytes, as far as a refusal needs them: which lines of an
    worktree_clean_stat_verdicts.rs  <- src/native/worktree/worktree_clean_stat_verdicts.rs
      What a clean check says about one tracked file: the refusal types, one index entry's recorded stat data, and the ...
    worktree_clean_content_attribute_sources.rs  <- src/native/worktree/worktree_clean_content_attribute_sources.rs
      The environment and the attribute files a content check reads: where the system and global attributes files are, what
    worktree_clean_content.rs  <- src/native/worktree/worktree_clean_content.rs
      The decision, by content and without a Git process, whether a tracked file that its index entry's stat data cannot
    worktree_clean_tree_walk.rs  <- src/native/worktree/worktree_clean_tree_walk.rs
      The walks of a worktree's tracked files against the index's recorded stat data, by metadata alone or by content for ...
    worktree_clean.rs  <- src/native/worktree/worktree_clean.rs
      Whether a worktree is clean enough to be copied from, or removed without asking Git: no operation in progress, an
  making_a_worktree_quickly/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> asking_the_commit_service_to_save_a_commit, converting_the_main_checkout, copying_ignored_files_into_a_stage, finding_the_git_folders, git_settings_read_once, how_long_the_program_waits_for_git_steps, is_the_worktree_clean, lines_about_worktrees_and_large_files, log_lines_of_the_commit_service, messages_between_the_wrapper_and_the_commit_service, planning_a_worktree_add_and_what_git_made, reading_saved_commits_and_files, reading_the_bytes_a_commit_would_save, recording_where_worktree_add_spent_its_time, running_git_for_a_transaction, sending_log_entries_to_the_system_journal, starting_git_and_describing_a_run, state_shared_by_preparing_and_landing_a_commit, the_words_meant_for_this_wrapper, work_left_for_the_service_to_finish_later
    worktree_snapshot_source.rs  <- src/native/worktree/worktree_snapshot_source.rs
      Whether the creating worktree can be the source of a snapshot for a new worktree, decided before real Git runs, and
    worktree_clone.rs  <- src/native/worktree/worktree_clone.rs
      Rung 2 of the cheap-worktree ladder: each tracked file of the clean source is cloned (`FICLONE`) into the new worktree
    worktree_completion_file_inspection.rs  <- src/native/worktree/worktree_completion_file_inspection.rs
      The descriptor-bound look at one added path's worktree copy: its identity, whether it still holds the original bytes, ...
    worktree_completion.rs  <- src/native/worktree/worktree_completion.rs
      Bringing each added path's worktree copy to the landed bytes after a commit landed
    worktree_ladder_target.rs  <- src/native/worktree/worktree_ladder_target.rs
      What both rungs that fill a worktree themselves need to know about the target of a `git worktree add` once real Git
    worktree_dirty_restore.rs  <- src/native/worktree/worktree_dirty_restore.rs
      The restore of a snapshot of a main checkout that had uncommitted changes: make the copy's index and files equal the
    worktree_slow_route.rs  <- src/native/worktree/worktree_slow_route.rs
      The plain sentence that tells a person why a `git worktree add` was not made as a snapshot, and the places it goes
    worktree_subvolume.rs  <- src/native/worktree/worktree_subvolume.rs
      The empty btrfs subvolume the wrapper makes at the destination of a `git worktree add`, before real Git runs, and
    worktree_ladder_run.rs  <- src/native/worktree/worktree_ladder_run.rs
      The ladder around one `git worktree add` on Linux: the choice of rung before real Git runs, and, after it
    worktree_ladder.rs  <- src/native/worktree/worktree_ladder.rs
      The cheap-worktree ladder around one `git worktree add`: what runs before real Git and after it, on Linux; on other
    worktree_trash_manifest.rs  <- src/native/worktree/worktree_trash_manifest.rs
      What a snapshot's leave-out made and moved, with identities: the trash directory it created and each directory it
    worktree_trash_entries.rs  <- src/native/worktree/worktree_trash_entries.rs
      The removal of the entries a snapshot's leave-out moved into its trash, side by side.
    worktree_trash_dispose.rs  <- src/native/worktree/worktree_trash_dispose.rs
      The disposal of the trash a snapshot's leave-out made: hand it to the commit service as a work item, or empty it here ...
    worktree_trash.rs  <- src/native/worktree/worktree_trash.rs
      The leave-out step of a snapshot of the main checkout: set aside the nested worktrees that are ordinary directories
    worktree_snapshot.rs  <- src/native/worktree/worktree_snapshot.rs
      The mechanics of rung 1 of the cheap-worktree ladder: a snapshot of the source subvolume taken at a stage name beside
    worktree_ladder_finish.rs  <- src/native/worktree/worktree_ladder_finish.rs
      What the ladder does after real Git returned: bind the add to the directories Git made, the snapshot rung's steps ...
  recovering_a_merge_that_was_cut_short/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> log_lines_of_the_commit_service, running_git_for_a_transaction, starting_git_and_describing_a_run, who_owns_a_transaction_folder
    merge_isolation_changes.rs  <- src/native/merge/merge_isolation_changes.rs
      The raw tree differences between the two commits of an isolated merge, and the state of a worktree path
    merge_isolation_journal.rs  <- src/native/merge/merge_isolation_journal.rs
      The durable intent record of an isolated merge: the commit it created, the branch value it expects, and the
    merge_isolation_recovery_steps.rs  <- src/native/merge/merge_isolation_recovery_steps.rs
      The steps both the recovery of an interrupted merge and the settling of a merge whose branch moved take: where
    merge_isolation_moved.rs  <- src/native/merge/merge_isolation_moved.rs
      The settling of an isolated merge whose branch moved while the merge was prepared or applied: what the merge had begun
    merge_isolation_recovery.rs  <- src/native/merge/merge_isolation_recovery.rs
      Recovery of an isolated merge whose process died: discard it when it never touched the worktree or the index
  merging_text_by_line_and_by_word/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    word_merge.rs  <- src/native/merge/word_merge.rs
      A three-way merge of text by line, and by word for the line regions that both sides changed, in process.
  rerunning_a_merge_word_by_word/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> environment_variables_for_git_children, finding_the_git_folders, merging_text_by_line_and_by_word, reading_branches_tags_and_markers, reading_git_settings_from_files, reading_saved_commits_and_files, running_git_for_a_transaction, sending_log_entries_to_the_system_journal, starting_git_and_describing_a_run, waiting_for_the_index_lock_of_git
    merge_second_pass_conflicts.rs  <- src/native/merge/merge_second_pass_conflicts.rs
      The conflicted files of a merge tree: the three stages of a path, the conflict they make, and the reading of `git ...
    merge_second_pass_git_runner.rs  <- src/native/merge/merge_second_pass_git_runner.rs
      How the word-level second pass starts Git: what a start gave back, the thing that runs a start, and the one that runs ...
    word_level_second_pass.rs  <- src/native/merge/merge_second_pass.rs
      The word-level second pass over the files a Git merge left conflicted: which conflicted files may be re-merged, their
    merge_second_pass_finish_stops.rs  <- src/native/merge/merge_second_pass_finish_stops.rs
      Which options of a merge, a pull, a cherry-pick or a revert make the finish stop where Git would stop, and the ...
    merge_second_pass_finish.rs  <- src/native/merge/merge_second_pass_finish.rs
      The finish after the word-level second pass resolved every conflicted file of a merge, rebase, cherry-pick, revert or ...
    merge_second_pass_tree.rs  <- src/native/merge/merge_second_pass_tree.rs
      The word-level second pass for a merge computed from objects: given the tree `git merge-tree` wrote and the stages of
    merge_second_pass_worktree.rs  <- src/native/merge/merge_second_pass_worktree.rs
      The word-level second pass after a forwarded merge, rebase, cherry-pick or revert that Git stopped with a conflict
  uploading_large_files/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> finding_the_git_folders, git_settings_read_once, how_long_the_program_waits_for_git_steps, lines_about_worktrees_and_large_files, reading_branches_tags_and_markers, reading_saved_commits_and_files, running_git_push_until_it_ends_or_stalls, starting_git_and_describing_a_run, the_words_meant_for_this_wrapper, what_each_git_command_accepts
    is_a_blob_a_large_file_pointer.rs  <- src/native/lfs/lfs_pointer.rs
      Whether a blob is a Git LFS pointer, decided the way `git-lfs` 3.8.1 decides it when it scans a push.
    large_files_found_in_the_commits_to_push.rs  <- src/native/lfs/lfs_pointer_scan.rs
      The LFS objects a push introduces, found in process: the pointer blobs that the commits to be pushed add.
    when_a_push_needs_the_large_file_upload.rs  <- src/native/lfs/lfs_pre_push.rs
      When a push needs the Git LFS upload, and the texts the parent prints around it.
    settings_the_upload_plan_reads.rs  <- src/native/lfs/lfs_upload_settings.rs
      The configuration the Git LFS upload plan reads, and the answers derived from it: the default remote, the branch
    what_the_large_file_upload_sends.rs  <- src/native/lfs/lfs_upload_plan.rs
      What the Git LFS upload before a push sends: the destinations `git lfs push` takes and the refs whose objects it
    exact_large_files_a_push_introduces.rs  <- src/native/lfs/lfs_object_plan.rs
      Narrow an upload plan from "scan these refs with `git-lfs`" to the exact objects the push introduces, decided in
    can_the_large_file_upload_start.rs  <- src/native/lfs/lfs_push_precheck.rs
      The check, before anything starts, that the Git LFS upload a push is about to run can start.
    running_the_large_file_upload.rs  <- src/native/lfs/lfs_upload.rs
      The parent's run of the Git LFS upload before a push: `git-lfs push <remote> <refs>` as one child it owns.
    push_preceded_by_large_file_upload.rs  <- src/native/lfs/lfs_bounded_push.rs
      Run one bounded `git push` of the auto-push, preceded by the Git LFS upload when the repository has one.
    large_file_step_before_a_typed_push.rs  <- src/native/lfs/lfs_interactive.rs
      The Git LFS step before a push the caller typed: the pre-check, the plan and the upload, with the caller's streams.
  pushing_what_the_commit_service_saved/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> answering_questions_without_starting_git, asking_the_commit_service_to_save_a_commit, finding_the_git_folders, git_settings_read_once, how_long_the_program_waits_for_git_steps, lines_about_worktrees_and_large_files, log_lines_of_the_commit_service, messages_between_the_wrapper_and_the_commit_service, naming_the_service_of_one_worktree, reading_branches_tags_and_markers, reading_git_settings_from_files, reading_saved_commits_and_files, recording_where_worktree_add_spent_its_time, recovering_a_merge_that_was_cut_short, rerunning_a_merge_word_by_word, running_git_push_until_it_ends_or_stalls, sending_log_entries_to_the_system_journal, starting_git_and_describing_a_run, uploading_large_files, who_owns_a_transaction_folder, work_left_for_the_service_to_finish_later
    what_configured_refspecs_make_of_a_branch.rs  <- src/native/push/auto_push_refspec_judging.rs
      The reading of a configured push refspec and the judgement of what the refspecs make of one branch: which remote names
    where_a_plain_push_would_send_a_branch.rs  <- src/native/push/auto_push_destination.rs
      Where a plain `git push` of one named branch would send it, decided from the configuration and the local refs
    words_and_limits_of_a_reconcile.rs  <- src/native/push/auto_push_reconcile_basics_and_limits.rs
      The words, limits and small Git steps that the reconcile of a rejected push shares: the line prefixes, the attempt
    paths_a_dirty_worktree_holds.rs  <- src/native/push/auto_push_dirty_paths.rs
      The paths a dirty worktree has uncommitted work in, the paths a fast-forward would change, and the overlap
    moving_the_local_branch_after_a_reconcile.rs  <- src/native/push/auto_push_reconcile_branch_advance.rs
      The move of the local branch after the merge was pushed: the fast-forward, the record that guards it, the check that
    the_merge_commit_of_a_reconcile.rs  <- src/native/push/auto_push_reconcile_merge_commit.rs
      The merge commit of a reconcile: the merge of the remote tip and the local tip computed in the object store (with the
    publishing_after_the_remote_moved.rs  <- src/native/push/auto_push_reconcile.rs
      Publishing a landed commit when the remote branch advanced after the commit landed
    pushing_a_landed_commit.rs  <- src/native/push/auto_push.rs
      The commit service's push of a landed commit's branch: which push runs, what its output shows
    file_name_of_one_branch.rs  <- src/native/push/auto_push_branch_key.rs
      The reversible, flat, case-insensitive-safe file name of one branch's push files.
    log_of_push_trigger_decisions.rs  <- src/native/push/push_trigger_log.rs
      The record of every decision of the push trigger after a command that adds commits
    handing_a_push_to_the_commit_service.rs  <- src/native/push/auto_push_trigger.rs
      Hands a push to the commit service after a command other than `git commit` added commits to the branch: a merge, a
    record_of_a_failed_backup_push.rs  <- src/native/push/push_failure_notice.rs
      The record of a backup push that failed after its commit was saved, and the rule for when the wrapper shows it
    retry_plans_on_disk.rs  <- src/native/push/push_retry_store.rs
      The retry plan on disk: one small file per branch whose push waits for a new try, in a directory of the service ...
    retrying_a_stopped_push.rs  <- src/native/push/push_retry.rs
      The service's own retry of a push it stopped or that failed: the pause before each new try, the list of branches ...
  after_commit_work_of_the_service/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> environment_variables_for_git_children, git_settings_read_once, holding_the_index_lock_for_the_commit_service, how_long_the_program_waits_for_git_steps, landing_commits, log_lines_of_the_commit_service, making_a_worktree_quickly, naming_the_service_of_one_worktree, pushing_what_the_commit_service_saved, reading_git_settings_from_files, reviewing_landed_commits_against_standards, running_git_for_a_transaction, running_git_push_until_it_ends_or_stalls, sending_log_entries_to_the_system_journal, starting_git_and_describing_a_run, uploading_large_files, work_left_for_the_service_to_finish_later
    service_claim_renewal.rs  <- src/native/landing/service_claim_renewal.rs
      Keeps the lease of the claims one service instance holds fresh while it works, so a claim is taken over only when its ...
    service_claim_watch.rs  <- src/native/landing/service_claim_watch.rs
      The Git directories whose after-commit work this service instance has been woken for, kept on disk in the instance's ...
    service_hold_retry.rs  <- src/native/landing/service_hold_retry.rs
      Runs a forwarded index writer again when real Git lost the race for the index to the commit service, instead of ...
    service_retry.rs  <- src/native/landing/service_retry.rs
      The repositories the service must look at again, and when: those with a claim that another instance holds and that ...
    service_review.rs  <- src/native/service/service_review.rs
      The service's handler for a `review` work item: read the landed commit, ask the decision gateway, and append the
    service_work_handlers_replanning_and_acceptance.rs  <- src/native/landing/service_work_handlers_replanning_and_acceptance.rs
      What the work of the commit service is made of: the kinds it runs, the handlers that run each kind, which records are
    service_work_drain_pending_claims.rs  <- src/native/landing/service_work_drain_pending_claims.rs
      Drains the pending items of one work directory: claims them under a lease, settles the unacceptable ones, groups the ...
    service_work.rs  <- src/native/landing/service_work.rs
      The service's consumer of after-commit work: pending `push` and `maintenance` work items of one repository are
    service_round.rs  <- src/native/landing/service_round.rs
      The drain of several repositories in one round of the service's after-commit work, side by side.
    service_work_run_round.rs  <- src/native/landing/service_work_run_round.rs
      Runs one round of the service's work over several directories: hands each directory its due retries, drains them side ...
  hooks_a_repository_configures/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> git_settings_read_once, starting_git_and_describing_a_run, the_settings_file_cli_git_config_jsonc
    hooks_set_in_git_settings.rs  <- src/native/health/external_hooks.rs
      The external hooks a repository configures (`hook.<name>.event` with `hook.<name>.command`), read from the
    warning_about_hooks_in_the_hooks_folder.rs  <- src/native/health/traditional_hook_warning.rs
      The non-blocking warning a wrapped commit prints when the configuration sets `core.hooksPath`.
  checking_that_your_setup_works/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> git_settings_read_once, hooks_a_repository_configures, how_long_the_program_waits_for_git_steps, running_git_for_a_transaction, saving_copies_of_what_a_command_would_throw_away, starting_git_and_describing_a_run, the_settings_file_cli_git_config_jsonc
    doctor_config.rs  <- src/native/health/doctor_config.rs
      What `git cli-git doctor` reports from the configuration: the external hooks Git configuration defines, and the
    doctor_fsmonitor.rs  <- src/native/health/doctor_fsmonitor.rs
      The fsmonitor section of `git cli-git doctor`: whether Git's own file-system monitor (`core.fsmonitor`) is off, and
    doctor.rs  <- src/native/health/doctor.rs
      `git cli-git doctor`: list the traditional hooks a repository holds and the external hooks its Git
    doctor_filesystem.rs  <- src/native/health/doctor_filesystem.rs
      The filesystem section of `git cli-git doctor`: what the repository's main checkout sits on (btrfs subvolume
  checking_version_bumps_in_a_commit/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> putting_corrected_files_into_the_private_index, reading_the_bytes_a_commit_would_save, running_git_for_a_transaction, running_policies_and_settling_their_corrections, the_settings_file_cli_git_config_jsonc, what_the_wrapper_asks_git_about_a_repository
    dependent_version_lifecycle.rs  <- src/native/dependent_version/dependent_version_lifecycle.rs
      The dependent-version planner's view of a prepared candidate state: every tracked file of the private index, its ...
    dependent_version_policy.rs  <- src/native/dependent_version/dependent_version_policy.rs
      The decision of `mono/dependent-version-bump`: when to plan, and the findings a plan becomes.
    policy_dependent_version.rs  <- src/native/dependent_version/policy_dependent_version.rs
      The `mono/dependent-version-bump` check
  converting_a_linked_worktree/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> converting_the_main_checkout, is_the_worktree_clean, lines_about_worktrees_and_large_files
    worktree_convert_journal.rs  <- src/native/worktree/worktree_convert_journal.rs
      The journal of one conversion of a worktree into a subvolume.
    worktree_convert_inspection.rs  <- src/native/worktree/worktree_convert_inspection.rs
      The first look at a worktree a conversion might change: what its `.git` file names, and every reason the conversion ...
    worktree_convert_compare.rs  <- src/native/worktree/worktree_convert_compare.rs
      A read-only comparison of the old tree of a conversion with the installed one: the first entry of the old tree that ...
    worktree_convert_remove.rs  <- src/native/worktree/worktree_convert_remove.rs
      Remove the old tree of a conversion, but only the directory the journal recorded, pinned by a descriptor from the ...
    worktree_convert_phases.rs  <- src/native/worktree/worktree_convert_phases.rs
      The journaled phases of one worktree conversion: the claim on the stage, the copy and its proof, the swap, the index
    worktree_convert.rs  <- src/native/worktree/worktree_convert.rs
      Convert one registered linked worktree that is an ordinary directory into a btrfs subvolume at the same path
    worktree_convert_arguments.rs  <- src/native/worktree/worktree_convert_arguments.rs
      The grammar of `git cli-git convert-worktree-to-subvolume`.
    worktree_convert_command.rs  <- src/native/worktree/worktree_convert_command.rs
      `git cli-git convert-worktree-to-subvolume`: parse the options, find the worktrees to convert, run the conversion on ...
  copying_ignored_files_around_one_git_command/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> copying_ignored_files_into_a_stage, making_a_worktree_quickly, recording_where_worktree_add_spent_its_time, running_git_for_a_transaction, starting_git_and_describing_a_run, the_words_meant_for_this_wrapper
    worktree_copy.rs  <- src/native/worktree_copying/worktree_copy.rs
      The linked-worktree ignored-state copy around one forwarded Git invocation: the public entry point the
  fixing_markdown_in_process/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> how_long_the_program_waits_for_git_steps, reading_the_bytes_a_commit_would_save, running_git_for_a_transaction, running_policies_and_settling_their_corrections, the_settings_file_cli_git_config_jsonc, what_the_wrapper_asks_git_about_a_repository
    markdown_exclude.rs  <- src/native/markdown/markdown_exclude.rs
      The `exclude` option of `markdown/autofix`
    markdown_linter_config.rs  <- src/native/markdown/markdown_linter_config.rs
      The linter configuration the `markdown/autofix` policy hands `monochromatic-lint`
    markdown_linter_guard.rs  <- src/native/markdown/markdown_linter_guard.rs
      Run one job on a thread of its own and wait for it until it ends, or until the thread has shown no sign of work for a
    markdown_linter_output.rs  <- src/native/markdown/markdown_linter_output.rs
      Read what one in-process `monochromatic-lint` fix run produced
    policy_markdown.rs  <- src/native/markdown/policy_markdown.rs
      The optional `markdown/autofix` policy
    markdown_linter.rs  <- src/native/markdown/markdown_linter.rs
      The linter the executable gives `markdown/autofix`
  merging_without_touching_your_files/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> environment_variables_for_git_children, holding_the_index_lock_for_the_commit_service, log_lines_of_the_commit_service, pushing_what_the_commit_service_saved, recovering_a_merge_that_was_cut_short, rerunning_a_merge_word_by_word, running_git_for_a_transaction, running_policies_and_settling_their_corrections, sending_log_entries_to_the_system_journal, starting_git_and_describing_a_run, stopping_the_program_at_a_named_step_for_tests, the_settings_file_cli_git_config_jsonc, the_words_meant_for_this_wrapper, waiting_for_the_index_lock_of_git, who_owns_a_transaction_folder
    merge_isolation_message.rs  <- src/native/merge/merge_isolation_message.rs
      The text of an isolated merge's commit message and the name Git gives the merged commit in it.
    merge_options.rs  <- src/native/merge/merge_options.rs
      The shape of a `git merge` command line that the wrapper can complete in isolation, or the reason it cannot.
    merge_isolation_run_types.rs  <- src/native/merge/merge_isolation_run_types.rs
      The values an isolated merge passes between its steps: the input, the outcome, the way a step gives up, the settings ...
    merge_isolation_git_reads.rs  <- src/native/merge/merge_isolation_git_reads.rs
      The questions an isolated merge asks Git and the terminal: the merge settings, the unsaved paths, the fetch head, and ...
    merge_isolation_report.rs  <- src/native/merge/merge_isolation_report.rs
      The refusals of an isolated merge, as the finding events a caller reads, each naming the paths involved and
    merge_isolation_apply.rs  <- src/native/merge/merge_isolation_apply.rs
      The writing half of an isolated merge: the merge tree, the merge commit, and the two-way update that applies it to ...
    merge_isolation.rs  <- src/native/merge/merge_isolation.rs
      A true `git merge <branch>` completed in isolation when the index holds staged changes that make native Git
    pull_isolation.rs  <- src/native/merge/pull_isolation.rs
      A `git pull` that merges, turned into the fetch it starts with and the `git merge FETCH_HEAD` it ends with, so the ...
  removing_a_linked_worktree/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> is_the_worktree_clean, lines_about_worktrees_and_large_files, naming_the_service_of_one_worktree, pushing_what_the_commit_service_saved, starting_git_and_describing_a_run, work_left_for_the_service_to_finish_later
    worktree_removal_rescue.rs  <- src/native/worktree/worktree_removal_rescue.rs
      What happens to the unshown push-failure notices and the unfinished backup pushes of a linked worktree when Git ...
    worktree_remove_registry.rs  <- src/native/worktree/worktree_remove_registry.rs
      Which registration of the invoking repository a worktree directory belongs to, and whether Git would pick that same
    worktree_remove.rs  <- src/native/worktree/worktree_remove.rs
      The removal of a clean linked worktree that is a btrfs subvolume, by one subvolume destroy call and the
    worktree_remove_plan.rs  <- src/native/worktree/worktree_remove_plan.rs
      The plan for one `git worktree remove` that the wrapper may serve with a subvolume destroy: the one directory it ...
  replaying_changes_onto_new_commits/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> environment_variables_for_git_children, reading_the_bytes_a_commit_would_save, rerunning_a_merge_word_by_word, running_git_for_a_transaction, starting_git_and_describing_a_run, the_hidden_second_git_folder
    replay_capture_order.rs  <- src/native/shadow/replay_capture_order.rs
      The capture-order decision for one shared path of a replay.
    replay_patch.rs  <- src/native/shadow/replay_patch.rs
      The unified-diff parser that replay's subsumption reads
    replay_containment.rs  <- src/native/shadow/replay_containment.rs
      The context-tolerant half of replay's subsumption
    replay_merge.rs  <- src/native/shadow/replay_merge.rs
      The merge step of one replay: the tree from `git merge-tree --write-tree` of the prepared change onto the landed tip.
    replay_reverse_apply.rs  <- src/native/shadow/replay_reverse_apply.rs
      The exact emulation of `git apply --reverse --check` for one text file
    replay_shadow.rs  <- src/native/shadow/replay_shadow.rs
      Running real Git against the shadow repository for replay
    replay_shared_paths.rs  <- src/native/shadow/replay_shared_paths.rs
      The paths both the prepared commit and the landed history changed since the preparation base
    replay_subsumption_text.rs  <- src/native/shadow/replay_subsumption_text.rs
      The text half of replay's subsumption
    replay_subsumption.rs  <- src/native/shadow/replay_subsumption.rs
      Subsumption before replay's three-way merge
  the_built_in_policies/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> reading_the_bytes_a_commit_would_save, running_policies_and_settling_their_corrections, scanning_those_bytes_for_forbidden_strings, the_settings_file_cli_git_config_jsonc, what_the_wrapper_asks_git_about_a_repository
    policy_catalog_specifiers.rs  <- src/native/policy/policy_catalog_specifiers.rs
      The optional `mono/catalog-specifiers` policy
    policy_final_newline.rs  <- src/native/policy/policy_final_newline.rs
      The built-in `final-newline` policy
    policy_forbidden_strings.rs  <- src/native/policy/policy_forbidden_strings.rs
      The optional `security/forbidden-strings` policy
    policy_root_context.rs  <- src/native/policy/policy_root_context.rs
      The optional `mono/forbidden-root-context` policy
  the_list_of_checks_every_commit_runs/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> checking_version_bumps_in_a_commit, fixing_markdown_in_process, putting_corrected_files_into_the_private_index, rules_about_where_a_command_may_run, rules_that_add_flags_to_a_command, running_policies_and_settling_their_corrections, saving_copies_of_what_a_command_would_throw_away, the_built_in_policies, the_settings_file_cli_git_config_jsonc, the_words_meant_for_this_wrapper, what_the_wrapper_asks_git_about_a_repository
    one_complete_policy_pass.rs  <- src/native/policy/policy_pass.rs
      One complete policy pass over the shipped policies
    fixing_files_in_place_command.rs  <- src/native/policy/direct_fix.rs
      `git cli-git fix`
    the_list_of_shipped_checks.rs  <- src/native/policy/policy_checks.rs
      The shipped policies as the engine sees them
  the_commit_command/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> answering_questions_without_starting_git, asking_the_commit_service_to_save_a_commit, environment_variables_for_git_children, fixing_markdown_in_process, git_settings_read_once, hooks_a_repository_configures, how_long_the_program_waits_for_git_steps, landing_commits, log_lines_of_the_commit_service, making_a_worktree_quickly, messages_between_the_wrapper_and_the_commit_service, putting_corrected_files_into_the_private_index, reading_and_writing_the_index_file_of_git, reading_options_the_way_git_does, reading_the_bytes_a_commit_would_save, replaying_changes_onto_new_commits, rerunning_a_merge_word_by_word, rules_about_what_a_commit_may_contain, rules_about_where_a_command_may_run, running_git_for_a_transaction, running_policies_and_settling_their_corrections, saving_copies_of_what_a_command_would_throw_away, scanning_those_bytes_for_forbidden_strings, sending_log_entries_to_the_system_journal, starting_git_and_describing_a_run, state_shared_by_preparing_and_landing_a_commit, stopping_the_program_at_a_named_step_for_tests, the_built_in_policies, the_hidden_second_git_folder, the_list_of_checks_every_commit_runs, the_settings_file_cli_git_config_jsonc, the_words_meant_for_this_wrapper, uploading_large_files, what_each_git_command_accepts, what_the_wrapper_asks_git_about_a_repository, work_left_for_the_service_to_finish_later
    commit_args.rs  <- src/native/commit/commit_args.rs
      Argument transforms of native preparation
    commit_facts.rs  <- src/native/commit/commit_facts.rs
      The repository facts of a commit transaction
    commit_transaction.rs  <- src/native/commit/commit_transaction.rs
      The values every commit of the service route shares: the invocation, how it can stop, what it came to, the selection
    post_commit.rs  <- src/native/commit/post_commit.rs
      The post-commit policy gate over a landed commit's own changes.
    private_index_request.rs  <- src/native/commit/private_index_request.rs
      The private index and object store a commit's preparation reads and writes, and the Git request that selects them.
    service_commit_args_editor_opening.rs  <- src/native/landing/service_commit_args_editor_opening.rs
      Whether a `commit` command line opens an editor: the long options that never decide it, the options that take a ...
    service_commit_args.rs  <- src/native/landing/service_commit_args.rs
      Which `git commit` command lines the resident service lands, and the message such a line stands for.
    service_message.rs  <- src/native/landing/service_message.rs
      The message and the identities of a commit the service lands, made from the command line the way native
    facts_of_the_commit_an_amend_replaces.rs  <- src/native/landing/service_amend.rs
      The facts of the commit an amend replaces, read once from the repository: its parents, its first parent's tree
    service_commit_input_notes_and_embedded_commit.rs  <- src/native/landing/service_commit_input_notes_and_embedded_commit.rs
      The small steps of a service commit: the arguments after the subcommand, the name of a switched branch, the standard ...
    service_prepare_ready_shapes.rs  <- src/native/landing/service_prepare_ready_shapes.rs
      The shapes a service preparation hands back: the scratch directory that removes itself, what a preparation came to
    service_commit_requests_and_landing_report.rs  <- src/native/landing/service_commit_requests_and_landing_report.rs
      The pieces of a service commit that need no loop: the request for a commit Git built, the request for a commit the ...
    commit_preparation.rs  <- src/native/commit/commit_preparation.rs
      Native `git commit` in the shadow repository against the private commit index
    service_native.rs  <- src/native/landing/service_native.rs
      The commits real Git builds for the commit service: an editor, a signature, and every command line the service does
    commit_conclusion.rs  <- src/native/commit/commit_conclusion.rs
      The events of a settled commit with the fix summary the policies' corrections earned.
    private_index_stage.rs  <- src/native/commit/private_index_stage.rs
      Write one corrected file into the private commit index: its bytes as a blob
    commit_convergence.rs  <- src/native/commit/commit_convergence.rs
      The commit path's bounded policy convergence over the private commit index
    commit_index_copying.rs  <- src/native/commit/commit_index_copying.rs
      The copying of the real index into a private index: a copy that keeps the source's timestamps, the check of
    commit_staged_additions.rs  <- src/native/commit/commit_staged_additions.rs
      The staged additions an explicit-path commit must carry into its private index, and the pathspec-file reader
    commit_index.rs  <- src/native/commit/commit_index.rs
      The private commit index of one transaction
    commit_staged_deletions.rs  <- src/native/commit/commit_staged_deletions.rs
      The commit-only finding for a staged deletion that an explicit-path commit cannot record because the path is
    commit_worktree_records.rs  <- src/native/commit/commit_worktree_records.rs
      The worktree completion records a landed commit owes the worktree: selected final-newline corrections.
    commit_no_change.rs  <- src/native/commit/commit_no_change.rs
      The normalization-only commit: a correction whose complete intended tree equals the recorded base ends without an ...
    service_prepare_changes_config_and_pathspec_steps.rs  <- src/native/landing/service_prepare_changes_config_and_pathspec_steps.rs
      The smaller steps of a service preparation: the changes a commit makes, whether the configuration needs native Git, ...
    service_replay_path_and_line_text.rs  <- src/native/landing/service_replay_path_and_line_text.rs
      Small text steps of the replay of captured changes: a path as Latin-1 text and back, and whether a line of Git's ...
    service_replay.rs  <- src/native/landing/service_replay.rs
      The retry of a commit the service sent back because a path it names changed on the branch since its base: the change
    service_prepare.rs  <- src/native/landing/service_prepare.rs
      The preparation of a commit that goes through the resident service: the private index, the selection UI, the policies
    service_commit.rs  <- src/native/landing/service_commit.rs
      The commit command's path through the resident service: prepare the commit in this process (private index
    commit_command.rs  <- src/native/commit/commit_command.rs
      The `git commit` command of the native wrapper: the preparation and landing through the resident service, the settled
  the_service_program/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> after_commit_work_of_the_service, environment_variables_for_git_children, finding_the_real_git_program, holding_the_index_lock_for_the_commit_service, how_long_the_program_waits_for_git_steps, landing_commits, log_lines_of_the_commit_service, messages_between_the_wrapper_and_the_commit_service, naming_the_service_of_one_worktree, pushing_what_the_commit_service_saved, recording_where_worktree_add_spent_its_time, running_git_push_until_it_ends_or_stalls, sending_log_entries_to_the_system_journal, starting_git_and_describing_a_run, stopping_the_program_at_a_named_step_for_tests, syncing_files_off_the_waiting_path, work_left_for_the_service_to_finish_later
    service_reply.rs  <- src/native/service/service_reply.rs
      The service's side of one waiting client: the queue of messages the landing thread answers with, and the loop that ...
    service_server_queue_types_and_send.rs  <- src/native/service/service_server_queue_types_and_send.rs
      The shapes the commit service's connection threads and landing thread pass between them (a queued request, the key ...
    service_server_connection_handling.rs  <- src/native/service/service_server_connection_handling.rs
      Handles one client connection of the commit service: reads the request line within the read timeout, checks who
    service_server_land_group.rs  <- src/native/service/service_server_land_group.rs
      Lands one group of queued requests as a batch and answers each client: settles open batches first, answers repeats and
    service_server_socket_binding.rs  <- src/native/service/service_server_socket_binding.rs
      Gets the listening socket of the commit service: the one the service manager passed in, or a socket bound at a path.
    service_server_work_loop.rs  <- src/native/service/service_server_work_loop.rs
      The loop of the commit service's work thread: it waits for wakes and due retries, runs a round of repository work for ...
    service_server.rs  <- src/native/service/service_server.rs
      The resident service's loop: accept connections, queue requests in arrival order, land what is queued as a batch
    last_words_when_a_signal_ends_the_service.rs  <- src/native/service/service_signal.rs
      The service's last line when a signal ends it
    the_service_as_a_mode_of_the_program.rs  <- src/native/service/service_main.rs
      The resident commit service as a mode of the one executable: `cli-git-native cli-git service run <options>`.
  working_out_what_a_typed_push_would_send/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> environment_variables_for_git_children, finding_the_real_git_program, how_long_the_program_waits_for_git_steps, lines_about_worktrees_and_large_files, pushing_what_the_commit_service_saved, reading_options_the_way_git_does, reading_the_bytes_a_commit_would_save, rules_about_what_a_commit_may_contain, running_git_for_a_transaction, running_git_push_until_it_ends_or_stalls, starting_git_and_describing_a_run, uploading_large_files, what_each_git_command_accepts
    reading_dry_run_push_output.rs  <- src/native/push/manual_push_porcelain.rs
      The parser of `git push --dry-run --porcelain` output, which tells the manual-push probe what a push would send.
    one_pushed_ref_and_query_limits.rs  <- src/native/push/manual_push_update_and_query_limits.rs
      The public update of one pushed reference and the limits on how many queries of a push probe run side by side.
    what_a_dry_run_push_would_send.rs  <- src/native/push/manual_push_probe.rs
      The authoritative ref updates a `git push` would send, discovered by a dry-run push whose report Git writes itself.
    commits_a_push_newly_publishes.rs  <- src/native/push/manual_push_range.rs
      The commits a push newly publishes, relative to what each destination remote already has.
    scope_of_one_typed_push.rs  <- src/native/push/manual_push_lifecycle.rs
      The manual-push candidate scope of one `git push`: the commits and trees the destination lacks.
    typed_push_that_lost_a_race.rs  <- src/native/push/push_stacking.rs
      An explicit `git push` that lost a race: the commits that landed on the remote since the local base are merged in
    push_of_a_merge_that_integrated_a_lost_race.rs  <- src/native/push/stacked_merge_push.rs
      The push of a merge that integrated a lost race, planned and run inside the running wrapper.
  what_happens_when_you_type_a_command/
    mod.rs  (index: top doc lists the files below in this order, then one `mod` line each)
    uses/ -> after_commit_work_of_the_service, asking_the_commit_service_to_save_a_commit, checking_that_your_setup_works, clearing_leftover_reference_locks, converting_a_linked_worktree, converting_the_main_checkout, copying_ignored_files_around_one_git_command, copying_ignored_files_into_a_stage, environment_variables_for_git_children, finding_the_git_folders, finding_the_real_git_program, git_settings_read_once, holding_the_index_lock_for_the_commit_service, hooks_a_repository_configures, how_long_the_program_waits_for_git_steps, merging_without_touching_your_files, planning_a_worktree_add_and_what_git_made, pushing_what_the_commit_service_saved, reading_options_the_way_git_does, reading_the_bytes_a_commit_would_save, recovering_a_merge_that_was_cut_short, removing_a_linked_worktree, rerunning_a_merge_word_by_word, rules_about_what_a_commit_may_contain, rules_about_where_a_command_may_run, running_git_for_a_transaction, running_git_push_until_it_ends_or_stalls, running_policies_and_settling_their_corrections, saving_copies_of_what_a_command_would_throw_away, scanning_those_bytes_for_forbidden_strings, starting_git_and_describing_a_run, the_commit_command, the_list_of_checks_every_commit_runs, the_service_program, the_settings_file_cli_git_config_jsonc, the_words_meant_for_this_wrapper, uploading_large_files, waiting_for_the_index_lock_of_git, what_each_git_command_accepts, what_the_wrapper_asks_git_about_a_repository, working_out_what_a_typed_push_would_send
    the_decision_one_command_ends_in.rs  <- src/native/invocation/action.rs
      The decision one wrapper invocation ends in.
    worktree_copy_forward.rs  <- src/native/worktree_copying/worktree_copy_forward.rs
      Running a forwarded Git command through the linked-worktree ignored-state copy
    entry_forward_action_planning.rs  <- src/native/invocation/entry_forward_action_planning.rs
      Plans what a forwarded Git command does once the wrapper has stripped its own controls
    entry_push_stacking_and_lfs_upload.rs  <- src/native/invocation/entry_push_stacking_and_lfs_upload.rs
      Turns a plain forward of a push into a stacked push or a push that uploads Git LFS objects first, and adds the
    index_writer_forward.rs  <- src/native/lock/index_writer_forward.rs
      Running a forwarded index writer as a child of the wrapper
    entry_run_forwarded_actions.rs  <- src/native/invocation/entry_run_forwarded_actions.rs
      Runs a forwarded Git command under the index-writer coordination, finishes a settled forward, and writes to the
    management_arguments.rs  <- src/native/invocation/management_arguments.rs
      The grammar of `git cli-git <command>`
    management_selection.rs  <- src/native/invocation/management_selection.rs
      The `--policy` selection of a direct `git cli-git check` or `fix`
    first_step_of_a_command.rs  <- src/native/invocation/startup_gate.rs
      The startup step of a wrapped, read-only or direct command: settle an isolated merge whose process died, stop on a
    management_direct_commands.rs  <- src/native/invocation/management_direct_commands.rs
      The direct commands of the `cli-git` namespace that run in this process (`git cli-git <command> ...`)
    management_service_repository_selection.rs  <- src/native/invocation/management_service_repository_selection.rs
      Which repository the service commands (`cli-git service ...`) mean: the global options name it, or Git is asked for
    deciding_what_a_cli_git_command_does.rs  <- src/native/invocation/management.rs
      Decide what one `git cli-git ...` invocation does.
    wrapped_command_notices_snapshots_and_manual_push.rs  <- src/native/invocation/wrapped_command_notices_snapshots_and_manual_push.rs
      Steps of a wrapped command that do not decide its ending: the escape-hatch use record, the push failure notices
    deciding_what_a_wrapped_command_does.rs  <- src/native/invocation/wrapped_command.rs
      Decide what one wrapped Git command does once real Git is known
    deciding_what_one_git_command_does.rs  <- src/native/invocation/entry.rs
      Decide what one `git` invocation of the native wrapper does
    message_when_the_program_panics.rs  <- src/native/invocation/panic_notice.rs
      The panic hook of the executable
```
