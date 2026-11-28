//! What:
//! The crate root:
//! it lists the submodules of the `cargo-shadow` library.
//! The library holds all the logic;
//! `main.rs`
//! only decides which mode to start.
//! Why:
//! Keeping the logic in a library lets the unit tests call it without starting a process.
//! A module that a file split makes is `pub(crate)`,
//! and the items it holds are re-exported (`pub use`) by the module they lived in before,
//! so the file layout stays out of the library's visible surface,
//! which then holds the paths a later split does not widen.
//! A unit test's sidecar is declared by the module it tests
//! (`#[cfg(test)] #[path = "<name>_tests.rs"] mod <name>_tests;`) rather than here,
//! so this list holds subjects and not controls:
//! a round that adds a module with its tests adds one line to it.
//! Three kinds of test module still stand below:
//! a round's own suite,
//! the fixtures several test files share,
//! and the identity's tests,
//! whose subject is one of the four byte-identical copies
//! that the other shadow crates carry and `tests/shadow_identity_copies.rs` compares.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // index.ts
//! export * from './cli.ts';
//! // ...one line per module...
//! ```

// Linux only, by the owner's word ("Ignore Win and mac until I say so."): the journal decides whether a killed run's owner is
// alive by reading `/proc/<pid>`, which other Unix systems lack, so an active run there would look dead and be recovered
// from under its own feet.
#[cfg(not(target_os = "linux"))]
compile_error!(
    "cargo-shadow supports Linux only (its journal reads /proc); other systems are a later change"
);

/// The command-line grammar of the binary when it is started under its own name.
pub mod cli;

/// The fixed values (shadowed name,
/// file names,
/// exit codes).
pub mod constants;

/// Release-age and trust checks over newly locked versions.
pub mod checks;

/// Small named conversions for error mappers and time arithmetic.
pub mod adapt;

/// Warns about dependencies that do not use the workspace catalog.
pub mod catalog_warning;

/// A streaming reader for the dump's CSV tables.
pub mod csv;

/// Runs the real cargo for the install and update steps.
pub mod cargo_run;

/// Finds the crate roots that carry a lockfile,
/// and snapshots and restores them.
pub mod crates;

/// The one HTTP GET the registry lookups need.
pub mod http;

/// Runs the repository-wide catalog commands for the CLI.
pub mod commands;

/// What a catalog command prints:
/// its findings and its install and update reports (split out of `commands.rs`).
pub(crate) mod command_report_printing;

/// The restore of lockfiles after a run and the recording side of the journal that recovers a killed run.
pub mod journal;

/// The preservation protocol that replaces one file (an exchange;
/// every inode that was live keeps a name;
/// not an atomic
/// conditional replace).
pub mod journal_replace;

/// The directories a creation left unsynchronized (split out of the journal replacement module).
pub(crate) mod journal_replace_unsynced;

/// Keeping the displaced file under a surviving name (split out of the journal replacement module).
pub mod journal_replace_preserve;

/// The recovery of a run that was killed,
/// split out of `journal.rs`.
pub mod journal_recover;

/// The recovery of one journal:
/// its roots and the temporaries a killed run left (split out of `journal_recover.rs`).
pub(crate) mod journal_recover_one;

/// Reads of a journal directory:
/// index,
/// ready record and listing (split out of the journal recovery module).
pub mod journal_directory_readers;

/// The manifest side of a recovery,
/// split out of `journal_recover.rs`.
pub mod journal_manifests;

/// The transaction handle:
/// a journal held by a lock for the life of a run.
pub mod journal_transaction;

/// The workspace lock,
/// the journal's unique name and its ready record (split out of the journal transaction module).
pub(crate) mod journal_transaction_creation;

/// The completion marker of a journal (split out of the journal transaction module).
pub mod journal_done_marker;

/// The notices about files a journal keeps beside the lockfiles (split out of the journal transaction module).
pub mod journal_kept_notices;

/// The error of finishing a transaction,
/// told by whether the outputs were already on disk.
pub mod journal_finish;

/// The recovery of journals that never became ready,
/// and the survivor check.
pub mod journal_unready;

/// The verdict on a journal that has no ready record (split out of the journal unready module).
pub mod journal_unready_verdict;

/// The assessment of one journal's containment records (split out of the journal unready module).
pub mod journal_unready_assessment;

/// The record of one journal and the context it was made in (split out of the journal unready assessment module).
pub(crate) mod journal_unready_record_context;

/// Runs a child process within a time bound.
pub mod bounded_run;

/// The trace of a run and the hooks a test drives inside one (split out of the bounded run module).
pub(crate) mod bounded_run_trace_and_hooks;

/// The cache of the control group route (split out of the bounded run module).
pub mod bounded_run_route_cache;

/// The write of one containment record (split out of the bounded run module).
pub mod bounded_run_record_writes;

/// The containment of a run:
/// its choice,
/// start and stop (split out of the bounded run module).
pub mod bounded_run_containment;

/// The start of a child inside the containment (split out of the containment module).
pub(crate) mod bounded_run_start;

/// The end of a bounded run,
/// and the stop of its scope unit after it (split out of the containment module).
pub(crate) mod bounded_run_end;

/// Error text that is built only when a failure happens,
/// passed as a `format_args!` value instead of a closure.
pub mod lazy_error_text;

/// SHA-1,
/// only to check that a stored Git object has the name it is stored under.
pub mod git_sha1;

/// Containment of everything a bounded run starts,
/// by a delegated control group (cgroup v2).
pub mod control_group;

/// The records of a scope and the membership verdict (split out of the control group module).
pub mod control_group_records;

/// One ask of the user manager where a slice's control group is,
/// and the rule that judges the bound it was given
/// (split out of the route discovery module).
pub(crate) mod control_group_route_one_ask;

/// Discovery of the route to a delegated control group (split out of the control group module).
pub mod control_group_route_discovery;

/// The mount view a control group route records (split out of the route discovery module).
pub(crate) mod control_group_route_view;

/// The short programs the route discovery runs under one deadline (split out of the route discovery module).
pub(crate) mod control_group_route_child_runs;

/// The read that tells a host where no user manager can be reached from a host whose manager is there and did not answer for a
/// moment (split out of the route discovery module).
pub(crate) mod control_group_route_manager_reach;

/// Process identity reads of `/proc` for the containment of a run (split out of the control group module).
pub mod control_group_process_identity;

/// Opening and reading the mounted procfs (split out of the control group process identity module).
pub(crate) mod control_group_procfs_reading;

/// The boot and machine identity that a journal record of a mount view is checked against (split out of the control group module).
pub mod control_group_machine_identity;

/// The stop of a scope,
/// and the reads of `cgroup.events` (split out of the control group module).
pub(crate) mod control_group_stop;

/// The resource limits of the caller's control group,
/// read so the scope of a run keeps them.
pub mod inherited_limits;

/// The command that acknowledges a journal of a run without a delegated control group.
pub mod acknowledge;

/// Pipe reading and writing that a caller can cancel.
pub mod bounded_io;

/// The transaction around the manifests that cargo's `update --breaking` rewrites.
pub mod manifest_edit;

/// The ownership evidence for a file a child process may have written (a window of time and the file's change time).
pub mod file_owner;

/// Fills the history cache from the crates.io database dump.
pub mod dump;

/// Reads the lockfiles of a base revision with git.
pub mod git_base;

/// How the trust base is chosen,
/// by the repository's files and then by git (split out of the git base module).
pub(crate) mod git_base_resolution;

/// Which system configuration file the selected Git reads (split out of the git base module).
pub mod git_system_config_lookup;

/// The reader of the few Git configuration keys the trust base needs,
/// without Git.
pub mod git_config;

/// The plain-file reader of `HEAD` and of where a repository's files are that chooses the trust base without Git.
pub mod git_refs;

/// What a reference names and the reference a branch's upstream names (split out of `git_refs.rs`).
pub(crate) mod git_reference_lookup;

/// The environment and the Git program a native read runs with.
/// (split out of the git refs module).
pub mod git_refs_environment;

/// The check,
/// from the object files,
/// that an object name in a reference names a commit.
pub mod git_objects;

/// The reading of the Git configuration files in Git's order,
/// split out of `git_refs.rs`.
pub mod git_settings;

/// The fetch refspecs of a remote and the mapping of a branch's merge reference through them.
pub mod git_refspec;

/// The choice of the trust base from the repository's files,
/// split out of `git_refs.rs`.
pub mod git_native;

/// The loop guard that keeps a launcher or link leading back to the shadow from being run as the real `cargo`.
pub mod guard;

/// The message of a cargo-shadow that was started by cargo-shadow.
pub mod arrival;

/// Registry versions in a lockfile and the difference between lockfile sets.
pub mod lockfile;

/// The release-age and trust rules.
pub mod policy;

/// The install step,
/// the counterpart of `pnpm install`.
pub mod install;

/// The stale crate roots of an install and the update that repairs them (split out of the install module).
pub(crate) mod install_stale_roots;

/// The helpers of the install that decide which policy checks a lockfile still needs,
/// split out of `install.rs`.
pub mod install_verify;

/// The error of an install and the classification of failures into it.
pub mod install_error;

/// Replaces refused new versions with the newest older release that passes.
pub mod set_aside;

/// The shadow's install step:
/// fingerprint fast path,
/// then the install over one workspace.
pub mod install_step;

/// Reads the few cargo options the shadow cares about.
pub mod options;

/// The marker file that tells an outer shadow instance that an inner one already printed the notice.
pub mod notice_marker;

/// The crates.io lookups.
pub mod registry;

/// Reading crates.io's answers and the history cache into records (split out of the registry module).
pub(crate) mod registry_record_reading;

/// Removes stale exclude selectors from the catalog.
pub mod prune;

/// The PATH lookup that finds the real `cargo` while skipping the shadow itself.
pub mod real_binary;

/// The selector syntax of the catalog's exclude lists.
pub mod selectors;

/// The line and the section the shadow adds after cargo's `--version` and `--help`,
/// the only places it names itself.
pub mod shadow_notice;

/// The steps the shadow runs around a cargo command.
pub mod shadow_step;

/// The shadow's answer to `cargo update` and `cargo generate-lockfile`.
pub mod shadow_update;

/// The rule that a shadow never starts a shadow:
/// the identity every shadow carries,
/// the check of a candidate,
/// and the arrival marker.
pub mod shadow_identity;

/// The shadow mode:
/// install step,
/// look up,
/// launch,
/// wait.
pub mod shim;

/// Timestamps from the registry.
pub mod time_text;

/// The rustup toolchain the repository pins.
pub mod toolchain;

/// Finds the catalog repository and the Cargo workspace of a directory.
pub mod workspace;

/// The fingerprint stamp that lets a cargo call skip the install check.
pub mod stamp;

/// Version ordering and caret compatibility.
pub mod version_order;

/// The catalog update,
/// the counterpart of `pnpm update`.
pub mod update;

/// The work values of the update (split out of the update module).
pub mod update_recorded_work;

/// The failure of an update,
/// the text it prints and the two conversions into it (split out of the update module).
pub(crate) mod update_error;

/// The locked crates a name-only exclude selector exempts from the release age (split out of the update module).
pub(crate) mod update_excluded_specs;

/// The manifests a `--breaking` update may rewrite (split out of the update module).
pub(crate) mod update_manifests;

/// The procedure an update runs,
/// what it needs and what it reports (split out of the update module).
pub(crate) mod update_request_and_report;

/// The transaction body of an update and the one cargo call it makes (split out of the update module).
pub(crate) mod update_run;

/// The option tests and the raised-floor warning wording that `update` asks about.
pub(crate) mod update_argument_and_warning_text;
/// The full update's resolution of each crate root,
/// split out of `update.rs`.
pub mod update_resolve;

/// The verification of the lockfiles an update leaves,
/// split out of `update.rs`.
pub mod update_verify;

/// The check of every locked entry against the policy,
/// the default of a frozen install.
pub mod verify;

/// Maps a `cargo update` or `cargo generate-lockfile` command line to the pnpm command it stands for.
pub mod update_plan;

/// The catalog half of `pnpm update --latest`:
/// raise catalog floors,
/// then sync.
pub mod tighten;

/// The command-word rules:
/// what the shadow does for a cargo command line.
pub mod verb;

/// The unit tests of the identity,
/// the launcher-record follow and the arrival marker.
/// These are declared here although one module owns them:
/// `src/shadow_identity.rs` is one of four byte-identical copies (`tests/shadow_identity_copies.rs`),
/// so a line declaring them there would differ between the four shadow crates.
#[cfg(test)]
mod shadow_identity_tests;

/// Scratch directories for the unit tests,
/// placed beside the build output.
#[cfg(test)]
mod test_scratch;

/// The differential tests of the plain-file base reader,
/// with Git as the oracle.
#[cfg(test)]
mod git_differential_tests;

/// The tests of the third review of the replace protocol and the journal.
#[cfg(test)]
mod journal_protocol_tests;

/// The round 6 tests of the command line named in errors.
#[cfg(test)]
mod rerun_tests;

/// Round 8 tests of acceptance rows that no other test file of the round holds.
#[cfg(test)]
mod acceptance_rows_tests;
/// The round 6 tests of what a bounded run contains.
#[cfg(test)]
mod bounded_contain_tests;
/// The probes of rounds 7 and 8 as tests:
/// what a delegated scope does on this host.
#[cfg(test)]
mod cgroup_probe_tests;
/// The support the control-group tests share:
/// finding the host's route and starting a launcher in a scope.
#[cfg(test)]
mod cgroup_test_support;
/// Round 7 tests built from the sixth review's counterexamples to the containment of a bounded run.
#[cfg(test)]
mod contain_counterexample_tests;
/// Rounds 7 and 8 tests of what a journal records about containment and what a recovery and an admission do with it.
#[cfg(test)]
mod containment_record_tests;
/// Round 8 tests of the failures on the containment path after a program started,
/// and of the bounds on the waits.
#[cfg(test)]
mod quiescence_failure_tests;
/// The journals a refusal keeps,
/// and what each kind of next run of the same workspace does with one.
#[cfg(test)]
mod refusal_journal_next_runs_tests;
/// Round 11 tests:
/// the launcher record that lets a killed wrapper be recognized.
#[cfg(test)]
mod round_eleven_tests;
/// Round 14 and 15 tests:
/// the start time of a launcher read from another time namespace.
#[cfg(test)]
mod round_fourteen_tests;
/// Round 9 tests:
/// the pending launcher,
/// the bounds on discovery and the internal locks.
#[cfg(test)]
mod round_nine_tests;
/// Round 17 tests:
/// the fixes asked for by the second certification pass.
#[cfg(test)]
mod round_seventeen_tests;
/// Round 16 tests:
/// the fixes asked for by the first certification pass.
#[cfg(test)]
mod round_sixteen_tests;
/// Round 10 tests:
/// the unsettled scope,
/// the strict boot identity and the discovery's shared deadline.
#[cfg(test)]
mod round_ten_tests;
/// Round 13 tests:
/// what a changed process number in procfs does and does not show.
#[cfg(test)]
mod round_thirteen_tests;
/// Round 12 tests:
/// the launcher check against the procfs of another process id namespace,
/// and the boot id read from the checked object.
#[cfg(test)]
mod round_twelve_tests;
/// Round 8 tests of acceptance rows no other test file of the round holds.
#[cfg(test)]
mod round_twentyone_tests;
/// Round 7 tests of how a workspace finds the journals whose runs may have left a writer.
#[cfg(test)]
mod survivor_discovery_tests;

/// Helpers the journal tests share.
#[cfg(test)]
mod journal_support;

/// The tests of the third review of the Git reader.
#[cfg(test)]
mod git_fallback_tests;

/// The tests of the bounds on cargo calls and registry pauses.
#[cfg(test)]
mod wait_bound_tests;

/// Test-only switches that make one named step fail once.
#[cfg(test)]
pub mod failpoints;

/// The device and inode of an entry,
/// the clock and the deadline of the lean-folder tools.
pub mod lean_identity;

/// The non-blocking take of a Cargo profile directory's build locks.
pub mod lean_lock_probe;

/// The scan of `/proc` for processes using a path.
pub mod lean_proc_scan;

/// The one way the lean-folder tools remove an entry.
pub mod lean_removal;

/// Disk sizes of trees.
pub mod lean_sizes;

/// The proof that a profile directory is not in use.
pub mod lean_guard;

/// The reader of a profile directory's units.
pub mod lean_unit_reader;

/// The removal of units under the shared protocol.
pub mod lean_unit_removal;

/// The decision of the sweep of older compilers.
pub mod lean_sweep_plan;

/// Fixtures of the lean-folder tests.
#[cfg(test)]
mod lean_test_support;

/// The tests of the sweep and the protocol under it.
#[cfg(test)]
mod lean_sweep_tests;

/// Mechanism A:
/// the toolchain-keyed build folder of a host cargo run.
pub mod lean_keyed_folder;

/// The trees of a host build base.
pub mod lean_build_trees;

/// Mechanism C:
/// the collection of verified-idle trees.
pub mod lean_idle_trees;

/// Mechanism E:
/// the size cap of a profile directory.
pub mod lean_size_caps;

/// Mechanism D:
/// the retention of verification run records.
pub mod lean_run_records;

/// The tests of the wiring of the run-record retention to the tasks that write records.
#[cfg(test)]
mod lean_records_wiring_tests;

/// The checkouts of a repository,
/// found from the Git directory's files.
pub mod lean_checkouts;

/// The removal of dated nightly toolchains no checkout pins.
pub mod lean_rustup_toolchains;

/// The words of the repository's sources that can name an image or a volume.
pub mod lean_container_references;

/// The removal of unused container images and volumes.
pub mod lean_podman_cleanup;

/// Real Podman behind the cleanup's interface.
pub mod lean_podman_real;

/// The numbers of the lean-folder tools and what each rests on.
pub mod lean_policy;

/// Where each long pass stopped,
/// so the next run continues there.
pub mod lean_progress;

/// Where the lean-folder run looks.
pub mod lean_run_discovery;

/// The free-space figures of a lean-folder run.
pub mod lean_run_report;

/// The run-record retention with the real clock.
pub mod lean_run_records_glue;

/// The steps of a lean-folder run.
pub mod lean_run_steps;

/// The lean-folder run.
pub mod lean_run;

/// The idle-trees mechanism of a run and what its pass reports.
pub mod lean_run_trees;

/// Mechanism F:
/// the continuous systemd user service and its installer.
pub mod lean_service_units;

/// The command-line glue of `cargo-shadow lean`.
pub mod lean_command;

/// Named reads of paths and text shared by the lean-folder tools.
pub mod lean_text;

/// The tests of the ban on the lean-folder tools starting cargo or themselves.
#[cfg(test)]
mod lean_no_recursion_tests;
