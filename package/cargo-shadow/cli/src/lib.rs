//! What:
//! The crate root:
//! it lists the submodules of the `cargo-shadow` library.
//! The library holds all the logic;
//! `main.rs`
//! only decides which mode to start.
//! Why:
//! Keeping the logic in a library lets the unit tests call it without starting a process.
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
compile_error!("cargo-shadow supports Linux only (its journal reads /proc); other systems are a later change");

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

/// The restore of lockfiles after a run and the recording side of the journal that recovers a killed run.
pub mod journal;

/// The preservation protocol that replaces one file (an exchange;
/// every inode that was live keeps a name;
/// not an atomic
/// conditional replace).
pub mod journal_replace;

/// Keeping the displaced file under a surviving name (split out of the journal replacement module).
pub mod journal_replace_preserve;

/// The recovery of a run that was killed,
/// split out of `journal.rs`.
pub mod journal_recover;

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

/// Runs a child process within a time bound.
pub mod bounded_run;

/// The cache of the control group route (split out of the bounded run module).
pub mod bounded_run_route_cache;

/// The write of one containment record (split out of the bounded run module).
pub mod bounded_run_record_writes;

/// The containment of a run:
/// its choice,
/// start and stop (split out of the bounded run module).
pub mod bounded_run_containment;

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

/// Discovery of the route to a delegated control group (split out of the control group module).
pub mod control_group_route_discovery;

/// Process identity reads of `/proc` for the containment of a run (split out of the control group module).
pub mod control_group_process_identity;

/// The boot and machine identity that a journal record of a mount view is checked against (split out of the control group module).
pub mod control_group_machine_identity;

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

/// Which system configuration file the selected Git reads (split out of the git base module).
pub mod git_system_config_lookup;

/// The reader of the few Git configuration keys the trust base needs,
/// without Git.
pub mod git_config;

/// The plain-file reader of `HEAD`,
/// branch configuration and references that chooses the trust base without Git.
pub mod git_refs;

/// The environment and the Git program a native read runs with.
/// (split out of the git refs module).
pub mod git_refs_environment;

/// The check,
/// from the object files,
/// that an object name in a reference names a commit.
pub mod git_objects;

/// The check that a commit stored in a pack file is a commit (split out of the git objects module).
pub mod git_objects_packed_commit;

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

/// The full update's resolution of each crate root,
/// split out of `update.rs`.
pub mod update_resolve;
/// The option tests and the raised-floor warning wording that `update` asks about.
pub(crate) mod update_argument_and_warning_text;

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
#[cfg(test)]
mod shadow_identity_tests;

/// Scratch directories for the unit tests,
/// placed beside the build output.
#[cfg(test)]
mod test_scratch;

/// What:
/// `#[cfg(test)] mod catalog_warning_tests;` declares the tests in `src/catalog_warning_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the catalog warning:"
/// so the behavior of `catalog_warning` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // catalog_warning_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod catalog_warning_tests;

/// What:
/// `#[cfg(test)] mod checks_tests;` declares the tests in `src/checks_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the release-age and trust checks over newly locked versions."
/// so the behavior of `checks` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // checks_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod checks_tests;

/// What:
/// `#[cfg(test)] mod cli_tests;` declares the tests in `src/cli_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the command grammar:"
/// so the behavior of `cli` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // cli_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod cli_tests;

/// What:
/// `#[cfg(test)] mod dump_tests;` declares the tests in `src/dump_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the CSV reader and the dump fill,"
/// so the behavior of `dump` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // dump_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod dump_tests;

/// What:
/// `#[cfg(test)] mod git_base_tests;` declares the tests in `src/git_base_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the choice of the trust base,
/// against real throwaway git repositories."
/// so the behavior of `git_base` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // git_base_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod git_base_tests;

/// What:
/// `#[cfg(test)] mod git_config_tests;` declares the tests in `src/git_config_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the Git configuration reader."
/// so the behavior of `git_config` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // git_config_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod git_config_tests;

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

/// The tests of the SHA-1 used for object names.
#[cfg(test)]
mod git_sha1_tests;

/// The round 6 tests of the recovery of unready journals and of the ready record.
#[cfg(test)]
mod journal_recover_tests;

/// The round 6 tests of what a bounded run contains.
#[cfg(test)]
mod bounded_contain_tests;
/// Round 7 tests built from the sixth review's counterexamples to the containment of a bounded run.
#[cfg(test)]
mod contain_counterexample_tests;
/// The probes of rounds 7 and 8 as tests:
/// what a delegated scope does on this host.
#[cfg(test)]
mod cgroup_probe_tests;
/// The support the control-group tests share:
/// finding the host's route and starting a launcher in a scope.
#[cfg(test)]
mod cgroup_test_support;
/// Round 7 tests of how a workspace finds the journals whose runs may have left a writer.
#[cfg(test)]
mod survivor_discovery_tests;
/// Round 9 tests:
/// the pending launcher,
/// the bounds on discovery and the internal locks.
#[cfg(test)]
mod round_nine_tests;
/// Round 10 tests:
/// the unsettled scope,
/// the strict boot identity and the discovery's shared deadline.
#[cfg(test)]
mod round_ten_tests;
/// Round 11 tests:
/// the launcher record that lets a killed wrapper be recognized.
#[cfg(test)]
mod round_eleven_tests;
/// Round 12 tests:
/// the launcher check against the procfs of another process id namespace,
/// and the boot id read from the checked object.
#[cfg(test)]
mod round_twelve_tests;
/// Round 13 tests:
/// what a changed process number in procfs does and does not show.
#[cfg(test)]
mod round_thirteen_tests;
/// Round 14 and 15 tests:
/// the start time of a launcher read from another time namespace.
#[cfg(test)]
mod round_fourteen_tests;
/// Round 16 tests:
/// the fixes asked for by the first certification pass.
#[cfg(test)]
mod round_sixteen_tests;
/// Round 17 tests:
/// the fixes asked for by the second certification pass.
#[cfg(test)]
mod round_seventeen_tests;
/// Round 8 tests of acceptance rows no other test file of the round holds.
#[cfg(test)]
mod round_twentyone_tests;
/// Round 8 tests of acceptance rows that no other test file of the round holds.
#[cfg(test)]
mod acceptance_rows_tests;
/// Round 8 tests of the failures on the containment path after a program started,
/// and of the bounds on the waits.
#[cfg(test)]
mod quiescence_failure_tests;
/// Round 7 tests of the three phases of finishing a transaction and how an update joins them with its own outcome.
#[cfg(test)]
mod finish_phase_tests;
/// Rounds 7 and 8 tests of what a journal records about containment and what a recovery and an admission do with it.
#[cfg(test)]
mod containment_record_tests;

/// Helpers the journal tests share.
#[cfg(test)]
mod journal_support;

/// The tests of the third review of the Git reader.
#[cfg(test)]
mod git_fallback_tests;

/// The tests of the bounds on cargo calls and registry pauses.
#[cfg(test)]
mod wait_bound_tests;

/// The tests of the runner that starts the real cargo for the install and update steps.
#[cfg(test)]
mod cargo_run_tests;

/// The tests of the bounded child runs.
#[cfg(test)]
mod bounded_run_tests;

/// The guard that a production build never computes a trace event.
#[cfg(test)]
mod bounded_run_trace_guard_tests;

/// Test-only switches that make one named step fail once.
#[cfg(test)]
pub mod failpoints;

/// What:
/// `#[cfg(test)] mod git_refs_tests;` declares the tests in `src/git_refs_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the plain-file reader of references and branch configuration,
/// over hand-made repository directories."
/// so the behavior of `git_refs` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // git_refs_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod git_refs_tests;

/// What:
/// `#[cfg(test)] mod file_owner_tests;` declares the tests in `src/file_owner_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the ownership evidence for a file a child may have written."
/// so the behavior of `file_owner` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // file_owner_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod file_owner_tests;

/// What:
/// `#[cfg(test)] mod guard_tests;` declares the tests in `src/guard_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the loop guard:"
/// so the behavior of `guard` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // guard_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod guard_tests;

/// What:
/// `#[cfg(test)] mod manifest_edit_tests;` declares the tests in `src/manifest_edit_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the manifest transaction around cargo's breaking update."
/// so the behavior of `manifest_edit` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // manifest_edit_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod manifest_edit_tests;

/// What:
/// `#[cfg(test)] mod notice_marker_tests;` declares the tests in `src/notice_marker_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the notice marker:"
/// so the behavior of `notice_marker` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // notice_marker_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod notice_marker_tests;

/// What:
/// `#[cfg(test)] mod real_binary_tests;` declares the tests in `src/real_binary_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the PATH lookup:"
/// so the behavior of `real_binary` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // real_binary_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod real_binary_tests;

/// What:
/// `#[cfg(test)] mod shadow_notice_tests;` declares the tests in `src/shadow_notice_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the notice:"
/// so the behavior of `shadow_notice` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // shadow_notice_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod shadow_notice_tests;

/// What:
/// `#[cfg(test)] mod shadow_update_tests;` declares the tests in `src/shadow_update_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the stand-in lockfile that a full update of a root without a lockfile uses."
/// so the behavior of `shadow_update` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // shadow_update_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod shadow_update_tests;

/// What:
/// `#[cfg(test)] mod shadow_step_tests;` declares the tests in `src/shadow_step_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the fallback after a failed install step:"
/// so the behavior of `shadow_step` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // shadow_step_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod shadow_step_tests;

/// What:
/// `#[cfg(test)] mod install_step_tests;` declares the tests in `src/install_step_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the shadow's install step:"
/// so the behavior of `install_step` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // install_step_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod install_step_tests;

/// What:
/// `#[cfg(test)] mod install_policy_tests;` declares the tests in `src/install_policy_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the policy checks of a local (not frozen) install:"
/// so the behavior of `install` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // install_policy_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod install_policy_tests;

/// What:
/// `#[cfg(test)] mod install_tests;` declares the tests in `src/install_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the install step,"
/// so the behavior of `install` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // install_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod install_tests;

/// What:
/// `#[cfg(test)] mod journal_tests;` declares the tests in `src/journal_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of compare-before-replace and of recovery from a killed run."
/// so the behavior of `journal` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // journal_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod journal_tests;

/// What:
/// `#[cfg(test)] mod lockfile_tests;` declares the tests in `src/lockfile_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the lockfile reading,"
/// so the behavior of `lockfile` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // lockfile_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod lockfile_tests;

/// What:
/// `#[cfg(test)] mod options_tests;` declares the tests in `src/options_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the option reading:"
/// so the behavior of `options` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // options_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod options_tests;

/// What:
/// `#[cfg(test)] mod policy_tests;` declares the tests in `src/policy_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the release-age and trust decisions,"
/// so the behavior of `policy` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // policy_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod policy_tests;

/// What:
/// `#[cfg(test)] mod selectors_tests;` declares the tests in `src/selectors_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the selector syntax,"
/// so the behavior of `selectors` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // selectors_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod selectors_tests;

/// What:
/// `#[cfg(test)] mod tighten_tests;` declares the tests in `src/tighten_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the catalog floor raise that `cargo update --breaking` performs,"
/// so the behavior of `tighten` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // tighten_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod tighten_tests;

/// What:
/// `#[cfg(test)] mod time_text_tests;` declares the tests in `src/time_text_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the timestamp reading and writing."
/// so the behavior of `time_text` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // time_text_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod time_text_tests;

/// What:
/// `#[cfg(test)] mod toolchain_tests;` declares the tests in `src/toolchain_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the toolchain choice,"
/// so the behavior of `toolchain` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // toolchain_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod toolchain_tests;

/// What:
/// `#[cfg(test)] mod update_tests;` declares the tests in `src/update_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the catalog update,"
/// so the behavior of `update` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // update_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod update_tests;

/// What:
/// `#[cfg(test)] mod update_plan_tests;` declares the tests in `src/update_plan_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the `cargo update` to `pnpm update` mapping."
/// so the behavior of `update_plan` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // update_plan_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod update_plan_tests;

/// What:
/// `#[cfg(test)] mod verb_tests;` declares the tests in `src/verb_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the command-word rules:"
/// so the behavior of `verb` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // verb_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod verb_tests;

/// What:
/// `#[cfg(test)] mod verify_tests;` declares the tests in `src/verify_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the check of every locked entry:"
/// so the behavior of `verify` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // verify_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod verify_tests;

/// What:
/// `#[cfg(test)] mod version_order_tests;` declares the tests in `src/version_order_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the version ordering,"
/// so the behavior of `version_order` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // version_order_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod version_order_tests;

/// What:
/// `#[cfg(test)] mod workspace_tests;` declares the tests in `src/workspace_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the repository and workspace lookup."
/// so the behavior of `workspace` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // workspace_tests.test.ts, run by the test runner only
/// ```
#[cfg(test)]
mod workspace_tests;

/// The tests of the limits a run's scope keeps from its caller.
#[cfg(test)]
mod inherited_limits_tests;

/// The device and inode of an entry,
/// the clock and the deadline of the lean-folder tools.
pub mod lean_identity;

/// The non-blocking take of a Cargo profile directory's build locks.
pub mod lean_lock_probe;

/// The scan of `/proc` for processes using a path.
pub mod lean_proc_scan;

/// The tests of the process scan and of what it cannot read.
#[cfg(test)]
mod lean_proc_scan_tests;

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

/// The tests of the removal primitive.
#[cfg(test)]
mod lean_removal_tests;

/// Mechanism A:
/// the toolchain-keyed build folder of a host cargo run.
pub mod lean_keyed_folder;

/// The tests of the toolchain-keyed build folder.
#[cfg(test)]
mod lean_keyed_folder_tests;

/// The trees of a host build base.
pub mod lean_build_trees;

/// Mechanism C:
/// the collection of verified-idle trees.
pub mod lean_idle_trees;
/// Mechanism C's removal of one tree,
/// under the proof that nothing uses it.
pub mod lean_tree_removal;
/// Mechanism C's two reads before it judges:
/// every tree's ledger,
/// and the age that orders them.
pub mod lean_tree_survey;

/// The tests of the idle-tree collection.
#[cfg(test)]
mod lean_idle_trees_tests;

/// Mechanism E:
/// the size cap of a profile directory.
pub mod lean_size_caps;

/// The tests of the size cap.
#[cfg(test)]
mod lean_size_caps_tests;

/// Mechanism D:
/// the retention of verification run records.
pub mod lean_run_records;

/// The tests of the run-record retention.
#[cfg(test)]
mod lean_run_records_tests;

/// The tests of the wiring of the run-record retention to the tasks that write records.
#[cfg(test)]
mod lean_records_wiring_tests;

/// The checkouts of a repository,
/// found from the Git directory's files.
pub mod lean_checkouts;

/// The tests of the checkout listing.
#[cfg(test)]
mod lean_checkouts_tests;

/// The removal of dated nightly toolchains no checkout pins.
pub mod lean_rustup_toolchains;

/// The tests of the toolchain retirement.
#[cfg(test)]
mod lean_rustup_toolchains_tests;

/// The words of the repository's sources that can name an image or a volume.
pub mod lean_container_references;

/// The removal of unused container images and volumes.
pub mod lean_podman_cleanup;

/// Real Podman behind the cleanup's interface.
pub mod lean_podman_real;

/// The tests of the Podman cleanup.
#[cfg(test)]
mod lean_podman_cleanup_tests;

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

/// The tests of the record of where a pass stopped.
#[cfg(test)]
mod lean_progress_tests;

/// The tests of the order a run takes its mechanisms in.
#[cfg(test)]
mod lean_run_tests;

/// Mechanism F:
/// the continuous systemd user service and its installer.
pub mod lean_service_units;

/// The tests of the service units.
#[cfg(test)]
mod lean_service_units_tests;

/// The command-line glue of `cargo-shadow lean`.
pub mod lean_command;

/// Named reads of paths and text shared by the lean-folder tools.
pub mod lean_text;

/// The tests of the ban on the lean-folder tools starting cargo or themselves.
#[cfg(test)]
mod lean_no_recursion_tests;
