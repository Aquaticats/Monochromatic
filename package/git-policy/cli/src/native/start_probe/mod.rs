//! What:
//!  The root start probe the kernel-release answer asks for:
//!  its nine arms,
//!  the private namespace and the scratch tmpfs they run in,
//!  the holder in a second namespace,
//!  the move and the unmount from a second process,
//!  the record the mode writes under `/run`,
//!  and the one deadline the mode enforces.
//! Why:
//!  A functional probe at start is what the project chose in place of an
//!  allowlist of approved kernel releases,
//!  and the accepted design places it in a new mode of the root handler
//!  (`doc/decision/cli-git-zfs-start-probe.md`;
//!  `doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The proposed probe").
//!  The module gate stays beside it and replaces nothing:
//!  the gate proves the loaded module's declaration and its request contract,
//!  the probe exercises the attach,
//!  the move,
//!  the unmount and the namespace scan on the running build,
//!  and an arm that could not run keeps the managed route blocked
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  Q4).
//!  The probe never opens `/dev/zfs`,
//!  never runs in the host mount namespace and never writes outside the
//!  record's directory and the tmpfs inside it.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `probe_arms.rs`:
//!   the nine arms and every step,
//!   in the worker's order.
//! - `probe_status_prefix.rs`:
//!   the word every status line begins with.
//! - `probe_errno_names.rs`:
//!   the closed errno and reason vocabularies.
//! - `probe_line_refusal.rs`:
//!   why a status line could not be read.
//! - `probe_status_line.rs`:
//!   one step line,
//!   its parse and the feed that keeps whole lines.
//! - `probe_call_error.rs`:
//!   one failed call and its errno.
//! - `probe_open_beneath.rs`:
//!   the control's own `openat2` and the handle a mount lands on.
//! - `probe_processes.rs`:
//!   pipes,
//!   the fork,
//!   the process group,
//!   the pid descriptor and the waits.
//! - `probe_child_prelude.rs`:
//!   what a child does to itself before its steps.
//! - `probe_pipe_lines.rs`:
//!   whole lines on a pipe and the three answers a read gives.
//! - `probe_mount_calls.rs`:
//!   the namespace,
//!   propagation,
//!   tmpfs,
//!   bind,
//!   unmount and table calls.
//! - `probe_descriptor_calls.rs`:
//!   the file-system context,
//!   the create and the two moves.
//! - `probe_fingerprint.rs`:
//!   what the record pins about the machine and the build.
//! - `probe_source_name.rs`:
//!   the dataset-name rule and the fixed snapshot.
//! - `probe_configuration_refusal.rs`:
//!   why the source or the checkout could not be told,
//!   and the closed word the record carries for each cause.
//! - `probe_configuration.rs`:
//!   the two names the configuration supplies,
//!   and the design's own names the mount table supplies when it does not.
//! - `probe_mount_verify.rs`:
//!   what the read-back must find.
//! - `probe_namespace_scan.rs`:
//!   the scan's grouping,
//!   its source match and its two comparisons.
//! - `probe_attach_verify.rs`:
//!   the read-back after the move.
//! - `probe_attach_arm.rs`:
//!   the exclusive attach itself.
//! - `probe_walk_arm.rs`:
//!   the walk,
//!   its refusal controls and the checkout open.
//! - `probe_report.rs`:
//!   the lines folded into arm outcomes and one verdict.
//! - `probe_record.rs`:
//!   the record's fixed keys and its started form.
//! - `probe_record_files.rs`:
//!   the record's directory,
//!   its mode and its atomic write.
//! - `first_attach_record.rs`:
//!   this boot's first real attach,
//!   claimed once and published by one rename.
//! - `probe_deadline.rs`:
//!   the bound the mode enforces,
//!   cited beside its measurement.
//! - `probe_unmount_child.rs`:
//!   the child that moves and unmounts.
//! - `probe_holder.rs`:
//!   the child that holds the source in a second namespace.
//! - `probe_holder_control.rs`:
//!   the mode's side of that holder.
//! - `probe_status_drain.rs`:
//!   one status pipe drained into the report.
//! - `probe_worker_step_lines.rs`:
//!   the lines the worker says on its status pipe.
//! - `probe_worker_holder_scans.rs`:
//!   the two holder requests and the scans' comparisons.
//! - `probe_worker_unmount_child.rs`:
//!   arm 8's forked child and the lines it relays.
//! - `probe_worker.rs`:
//!   what the fork hands the worker,
//!   and arms 1 to 5,
//!   the scans,
//!   arm 8's fork and the teardown.
//! - `probe_test_switch.rs`:
//!   the one test-only switch:
//!   disabled,
//!   or held after a step.
//! - `probe_mode_words.rs`:
//!   the verdict line,
//!   the source readings and the skipped record.
//! - `probe_mode.rs`:
//!   the mode itself and its one deadline.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const status = gitWorktreeCloneMountHandler.run(['start-probe']); // the mode writes /run/git-worktree-clone-mount-probe/result
//! ```

/// This boot's first real attach:
/// its claim,
/// its started record and its outcome.
pub mod first_attach_record;
/// The nine arms and every step.
pub mod probe_arms;
/// The exclusive attach itself.
pub mod probe_attach_arm;
/// The read-back after the move.
pub mod probe_attach_verify;
/// One failed call and its errno.
pub mod probe_call_error;
/// What a child does to itself before its steps.
pub mod probe_child_prelude;
/// The two names the configuration supplies,
/// and the mount table's own.
pub mod probe_configuration;
/// Why the source or the checkout could not be told,
/// and the record's words.
pub mod probe_configuration_refusal;
/// The bound the mode enforces.
pub mod probe_deadline;
/// The file-system context,
/// the create and the two moves.
pub mod probe_descriptor_calls;
/// The closed errno and reason vocabularies.
pub mod probe_errno_names;
/// What the record pins about the machine and the build.
pub mod probe_fingerprint;
/// The child that holds the source in a second namespace.
pub mod probe_holder;
/// The mode's side of that holder.
pub mod probe_holder_control;
/// Why a status line could not be read.
pub mod probe_line_refusal;
/// The mode itself.
pub mod probe_mode;
/// The mode's words and readings.
pub mod probe_mode_words;
/// The namespace,
/// propagation,
/// tmpfs,
/// bind,
/// unmount and table calls.
pub mod probe_mount_calls;
/// What the read-back must find.
pub mod probe_mount_verify;
/// The scan's grouping and its comparisons.
pub mod probe_namespace_scan;
/// The control's own `openat2` and the handle a mount lands on.
pub mod probe_open_beneath;
/// Whole lines on a pipe.
pub mod probe_pipe_lines;
/// Pipes,
/// the fork,
/// the group,
/// the pid descriptor and the waits.
pub mod probe_processes;
/// The record's keys and its started form.
pub mod probe_record;
/// The record's directory,
/// its mode and its atomic write.
pub mod probe_record_files;
/// The lines folded into arms and a verdict.
pub mod probe_report;
/// The dataset-name rule and the fixed snapshot.
pub mod probe_source_name;
/// One status pipe drained into the report.
pub mod probe_status_drain;
/// One step's line,
/// its parse and the feed.
pub mod probe_status_line;
/// The word every status line begins with.
pub mod probe_status_prefix;
/// The test-only switch the timing and deadline arms set.
pub mod probe_test_switch;
/// The child that moves and unmounts.
pub mod probe_unmount_child;
/// The walk,
/// its refusal controls and the checkout open.
pub mod probe_walk_arm;
/// The worker's arms.
pub mod probe_worker;
/// The two holder requests and the scans' comparisons.
pub mod probe_worker_holder_scans;
/// The lines the worker says on its status pipe.
pub mod probe_worker_step_lines;
/// Arm 8's forked child and the lines it relays.
pub mod probe_worker_unmount_child;

/// The bound the mode enforces and its margin.
pub use probe_deadline::{
    PROBE_DEADLINE,
    PROBE_DEADLINE_MARGIN,
};
/// What the record pins about the machine and the build.
pub use probe_fingerprint::FingerprintPaths;
/// The mode the handler runs.
pub use probe_mode::ProbeMode;
/// The record a refused gate writes.
pub use probe_mode_words::skip_after_gate_refusal;
/// The record's text.
pub use probe_record::{
    ProbeRecord,
    started_text,
};
/// The record's place and its two mount points.
pub use probe_record_files::{
    HOLDER_NAME,
    RECORD_DIRECTORY,
    RESULT_NAME,
    RecordPlace,
    SCRATCH_NAME,
};
/// The test-only switch the timing and deadline arms set.
pub use probe_test_switch::{
    TEST_SWITCH_VARIABLE,
    TestSwitch,
    test_switch_from_environment,
};
