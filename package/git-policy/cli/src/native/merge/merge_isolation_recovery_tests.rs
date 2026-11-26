//! What:
//!  The controls of the recovery of an interrupted merge,
//!  each on a repository left in one of the states a killed merge leaves,
//!  in the sibling files this one declares.
//! Why:
//!  One file holding every state a killed merge leaves outgrew what a reader can hold
//!  (`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`,
//!  item 15),
//!  so the controls are split by the behaviour each group covers,
//!  and this file is the one place that names them.
//!  Its files,
//!  each using only files listed before it:
//!
//! - `merge_isolation_recovery_test_fixtures_tests.rs`:
//!   the repositories the controls build and the intent records they leave in one.
//! - `merge_isolation_recovery_test_readings_tests.rs`:
//!   what a control reads back:
//!   the staged entries,
//!   the action per record,
//!   the judgement of every changed path,
//!   a checked-out mode and a commit's bytes.
//! - `merge_isolation_recovery_record_decisions_tests.rs`:
//!   the decision recovery makes for each state of one record,
//!   and the starts
//!   a completion makes.
//! - `merge_isolation_recovery_refusals_and_notices_tests.rs`:
//!   the refusal that names another namespace's record,
//!   and the line that reports
//!   a merge it finished.
//! - `merge_isolation_recovery_lock_waiting_tests.rs`:
//!   what a recovery does about `index.lock`.
//! - `merge_isolation_recovery_late_changes_tests.rs`:
//!   a path that changed after the merge judged it.
//! - `merge_isolation_recovery_saved_merge_finish_tests.rs`:
//!   the finish of a merge a command saved on its side ref,
//!   and what each refusal keeps.
//! - `merge_isolation_recovery_installed_merge_tests.rs`:
//!   a merge whose index an earlier attempt wrote,
//!   and every way the next command
//!   stops on one.
//! - `merge_isolation_recovery_begun_merge_tests.rs`:
//!   whether a saved merge counts as begun,
//!   without reading the whole worktree.
//! - `merge_isolation_recovery_written_path_shapes_tests.rs`:
//!   the shapes a merge's write meets:
//!   a link,
//!   a nested directory,
//!   an emptied directory and a branch moved under a written checkout.
//! - `merge_isolation_recovery_settle_and_other_commands_tests.rs`:
//!   the settle as the awaited route and as another command meet it.
//! - `merge_isolation_recovery_kept_copy_decisions_tests.rs`:
//!   the one decision every command makes about a copy a merge kept.
//! - `merge_isolation_recovery_saves_and_holders_tests.rs`:
//!   whether a save beside a path is the merge's own bytes,
//!   and what a program holding a copy open means for it.
//! - `merge_isolation_recovery_carried_edits_tests.rs`:
//!   finishing a saved merge over the person's own edits.
//! - `merge_isolation_recovery_capture_refs_tests.rs`:
//!   the ref that keeps a capture alive while a saved merge is finished.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // merge/merge_isolation_recovery_tests/mod.ts: one `export * from './file'` per file below.
//! ```

/// The repositories the controls build and the intent records they leave in one.
#[path = "merge_isolation_recovery_test_fixtures_tests.rs"]
mod fixtures;

/// What a control reads back from a repository it built.
#[path = "merge_isolation_recovery_test_readings_tests.rs"]
mod readings;

/// The decision recovery makes for each state of one record.
#[path = "merge_isolation_recovery_record_decisions_tests.rs"]
mod record_decisions;

///  The refusal that names another namespace's record,
///  and the line that reports a finished merge.
#[path = "merge_isolation_recovery_refusals_and_notices_tests.rs"]
mod refusals_and_notices;

/// What a recovery does about `index.lock`.
#[path = "merge_isolation_recovery_lock_waiting_tests.rs"]
mod lock_waiting;

/// A path that changed after the merge judged it.
#[path = "merge_isolation_recovery_late_changes_tests.rs"]
mod late_changes;

/// The finish of a merge a command saved on its side ref.
#[path = "merge_isolation_recovery_saved_merge_finish_tests.rs"]
mod saved_merge_finish;

/// A merge whose index an earlier attempt wrote.
#[path = "merge_isolation_recovery_installed_merge_tests.rs"]
mod installed_merge;

/// Whether a saved merge counts as begun.
#[path = "merge_isolation_recovery_begun_merge_tests.rs"]
mod begun_merge;

/// The shapes a merge's write meets at one path.
#[path = "merge_isolation_recovery_written_path_shapes_tests.rs"]
mod written_path_shapes;

/// The settle as the awaited route and as another command meet it.
#[path = "merge_isolation_recovery_settle_and_other_commands_tests.rs"]
mod settle_and_other_commands;

/// The one decision every command makes about a copy a merge kept.
#[path = "merge_isolation_recovery_kept_copy_decisions_tests.rs"]
mod kept_copy_decisions;

/// Whether a save beside a path is the merge's own bytes.
#[path = "merge_isolation_recovery_saves_and_holders_tests.rs"]
mod saves_and_holders;

/// Finishing a saved merge over the person's own edits.
#[path = "merge_isolation_recovery_carried_edits_tests.rs"]
mod carried_edits;

/// The ref that keeps a capture alive while a saved merge is finished.
#[path = "merge_isolation_recovery_capture_refs_tests.rs"]
mod capture_refs;
