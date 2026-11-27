//! What:
//! The catalog update,
//! the counterpart of `pnpm update`:
//! moves lockfiles to newer releases under the release-age and trust
//! policy in `rust-catalog.toml`,
//! as one transaction.
//! Why:
//! pnpm's `update` is its own policy-aware command (<https://pnpm.io/cli/update>,
//! read 2026-11-03):
//! it moves dependencies
//! within their ranges and applies `minimumReleaseAge` and `trustPolicy` while it chooses.
//! Cargo's resolver applies neither,
//! so the shadow answers `cargo update` with this procedure instead of passing it to cargo's resolver.
//!
//! Two kinds.
//! The full kind re-resolves each crate root with
//! `cargo -Zunstable-options generate-lockfile --publish-time <now minus minimumReleaseAge>`,
//! which picks the newest releases
//! published before the cutoff (only `generate-lockfile` accepts `--publish-time`,
//! re-probed 2026-10-29),
//! then moves crates
//! the catalog exempts from the release age to their newest release.
//! The targeted kind runs the user's `cargo update` command
//! (named packages,
//! `--precise`,
//! `--breaking`) as given.
//! Both then collect the registry versions no lockfile held before,
//! set aside what the policy refuses,
//! check what is left,
//! and keep the new lockfiles only when nothing failed and it is not a
//! dry run;
//! otherwise every lockfile is restored.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update.ts
//! export async function updateRustLockfiles({ root, dryRun, now, runCargo, registry }): Promise<UpdateReport>
//! ```

/// What:
/// `use std::cell::RefCell;` imports `RefCell`,
/// a box whose contents can be changed through a shared reference;
/// the check that nothing else holds it happens at run time instead of at compile time.
/// Why:
/// The manifests a `--breaking` update may rewrite are collected into one before the run and read again when the run is not kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are shared mutable references by default; a RefCell is what Rust needs to get that.
/// ```
use std::cell::RefCell;

/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports the snapshot of one lockfile and the call that reads one per crate root.
/// Why:
/// The entry point snapshots every lockfile before the run and hands those snapshots to the restore when nothing is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, read_locks } from 'crate/crates';
/// ```
use crate::crates::{
    LockSnapshot,
    read_locks,
};
/// What:
/// `use crate::install::PolicyJudge;` imports the release-age and trust judge of an install from another module of this crate so this file can name it unqualified.
/// Why:
/// Finishing a journal that an earlier run left asks the same policy questions an install does,
/// so the recovery is given one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { PolicyJudge } from 'crate/install';
/// ```
use crate::install::PolicyJudge;
/// The refused run's own words,
/// so the update and the install refuse one way.
/// Why:
/// both keep a journal and say the same thing about it,
/// and all that differs is the nouns of what the run would have written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { refusalOfWriters } from 'crate/install';
/// ```
use crate::install::refusal_of_writers;
/// What:
/// `use crate::journal::{KeepAs, QUARANTINE_DIR};` imports the keep rule of a displaced manifest and the folder it is kept in.
/// Why:
/// A manifest the restore puts back displaces whatever is there now,
/// and that copy is kept beside the journal rather than dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { KeepAs, QUARANTINE_DIR } from 'crate/journal';
/// ```
use crate::journal::{
    KeepAs,
    QUARANTINE_DIR,
};
/// What:
/// `use crate::journal::{begin, recover_report, restore_if_unchanged};` imports the call that starts a transaction over the snapshots,
/// the recovery of an earlier run and the restore that skips a lockfile another writer changed.
/// Why:
/// These three are the transaction around the update:
/// finish what a killed run left,
/// journal this one,
/// and put the lockfiles back when it is not kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { begin, recover_report, restore_if_unchanged } from 'crate/journal';
/// ```
use crate::journal::{
    begin,
    recover_report,
    restore_if_unchanged,
};
/// What:
/// `use crate::journal_finish::{FinishError, settle_finish};` imports the failure of finishing a journal and the call that joins it with the update's own result.
/// Why:
/// A finish error never replaces the update's answer;
/// it is reported beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FinishError, settle_finish } from 'crate/journal_finish';
/// ```
use crate::journal_finish::{
    FinishError,
    settle_finish,
};
/// What:
/// `use crate::manifest_edit::restore_manifests;` imports the restore of a manifest a `--breaking` run may have rewritten.
/// Why:
/// The lockfiles and the manifests go back together,
/// and a manifest another writer changed meanwhile is left alone with a warning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { restore_manifests } from 'crate/manifest_edit';
/// ```
use crate::manifest_edit::{
    ManifestEdit,
    restore_manifests,
};
/// What:
/// `use crate::update_argument_and_warning_text::is_breaking_option;` imports the test that a `cargo update` command line carries `--breaking`.
/// Why:
/// Only a breaking update rewrites manifests,
/// so only that one journals them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { is_breaking_option } from 'crate/update_argument_and_warning_text';
/// ```
use crate::update_argument_and_warning_text::is_breaking_option;
/// What:
/// `use crate::update_run::run_update;` imports the transaction body of an update.
/// Why:
/// The entry point journals,
/// restores and settles around it,
/// and it holds no part of that work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { run_update } from 'crate/update_run';
/// ```
use crate::update_run::run_update;

/// The error,
/// the request and the report of the update (split out of this module).
pub use crate::update_error::UpdateError;
/// The two conversions every update step maps its own error through (split out of this module).
pub(crate) use crate::update_error::{
    other,
    other_text,
};
/// The locked crates a name-only exclude selector exempts (split out of this module).
pub use crate::update_excluded_specs::excluded_update_specs;
/// The procedure an update runs,
/// what it needs and what it reports (split out of this module).
pub use crate::update_request_and_report::{
    UpdateKind,
    UpdateReport,
    UpdateRequest,
};
/// The one cargo call the update's steps share (split out of this module).
pub(crate) use crate::update_run::run_or_fail;

/// The work value of one recorded cargo call (split out of the update module).
pub(crate) use crate::update_recorded_work::CargoRun;
/// What:
/// `pub fn update_rust_lockfiles(request: &UpdateRequest, kind: &UpdateKind) -> Result<UpdateReport, UpdateError>` runs the
/// update over the listed crate roots.
/// Why:
/// It first finishes any run that was killed (`journal.rs`),
/// snapshots every lockfile,
/// and puts the snapshots back on a failure,
/// a refusal or a dry run,
/// but only into lockfiles that still hold what this run produced,
/// so a refused version never stays in
/// some crates and not others and a change written by another process is never discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function updateRustLockfiles(args): Promise<UpdateReport>
/// ```
pub fn update_rust_lockfiles(
    request: &UpdateRequest,
    kind: &UpdateKind,
) -> Result<UpdateReport, UpdateError> {
    // Typed binding: `recovered: Vec<String>`.
    let recovery = recover_report(
        request.root,
        &PolicyJudge {
            settings: request.settings,
            registry: request.registry,
            now: request.now,
        },
    );
    let mandatory_recovery_failure: bool = recovery.mandatory_failure;
    let recovered: Vec<String> = recovery.messages;
    // Borrow `&recovered`: lends the value without copying or giving it away.
    for message in &recovered {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: warning: {message}");
    }
    // A mandatory phase of finishing an earlier run's journal failed: that journal is not finished, so this command does not go on.
    // A failed removal of a journal already marked done only warns.
    if mandatory_recovery_failure {
        return Err(other("cargo-shadow: the recovery of an earlier run could not finish its journal (see the warnings above), so nothing was changed by this command and it was not run; fix the cause and run the command again".to_string()));
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `before: Vec<LockSnapshot>`.
    let before: Vec<LockSnapshot> =
        read_locks(request.root, request.crate_dirs).map_err(other_text)?;
    // Macro `eprintln!`: builds or prints text (or a list) from a template.
    eprintln!("cargo-shadow: snapshotted {} lockfiles", before.len());
    // Borrow `&before`: lends the value without copying or giving it away.
    // Review finding (failed begin): without a complete recovery record nothing may be mutated, so a journal that cannot be
    // started stops the update here, before the first cargo call.
    let transaction = begin(request.root, &before, matches!(kind, UpdateKind::Targeted { cargo_args } if cargo_args.iter().map(String::as_str).any(is_breaking_option))).map_err(other_text)?;
    if let Some(notice) = transaction.kept_notice() {
        crate::journal_transaction::announce_kept_notice(notice);
    }
    // What:     `RefCell<Vec<LockSnapshot>>` is a list that can be added to through a shared reference (sibling: `&mut Vec`, which needs exclusive access); `RefCell::new(Vec::new())` makes an empty one.
    // Why:      `produced` starts empty because cargo has not written yet; each cargo write is recorded the moment it succeeded.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const produced: LockSnapshot[] = [];
    // ```
    let produced: RefCell<Vec<LockSnapshot>> = RefCell::new(Vec::new());
    // The manifests a `--breaking` run may rewrite: empty for every other update.
    let edits: RefCell<Vec<ManifestEdit>> = RefCell::new(Vec::new());
    // Borrow `&before`: lends the value without copying or giving it away.
    // Typed binding: `outcome: Result<UpdateReport, UpdateError>`.
    let outcome: Result<UpdateReport, UpdateError> =
        run_update(request, kind, &before, &produced, &edits, &transaction);
    // See `install_rust_lockfiles`: writers that may remain forbid both the rollback and the acceptance (round 6, 5c N2).
    if transaction.writers_may_remain() {
        // What:     `transaction.writers_may_remain_reason()` is `Some(text)` when the stop recorded why it could not show every
        //           writer gone, and `None` for a run that was marked without one (a test's stand-in).
        // Why:      The refusal says how long the stop waited and what it was waiting for, so a member that has exited is told
        //           from one that is alive and a loaded host from a member that cannot be killed (issue 6528).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const shortfall = transaction.writersMayRemainReason() ?? '';
        // throw refusalOfWriters('lockfiles and manifests', shortfall);
        // ```
        let shortfall: String = transaction.writers_may_remain_reason().unwrap_or_default();
        return Err(other_text(refusal_of_writers(
            "lockfiles and manifests",
            &shortfall,
        )));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&outcome`: lends the value without copying or giving it away.
    // Typed binding: `keep: bool`.
    // Macro `matches!`: builds or prints text (or a list) from a template.
    let keep: bool = matches!(&outcome, Ok(report) if report.written);
    // `let mut`: a binding that may change.
    // Typed binding: `restore_warnings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut restore_warnings: Vec<String> = Vec::new();
    if !keep {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: restoring every lockfile that this run changed");
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&before`: lends the value without copying or giving it away.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        let (lock_warnings, lock_failure) =
            restore_if_unchanged(request.root, &before, &produced.borrow());
        restore_warnings = lock_warnings;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&edits`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        let (manifest_warnings, manifest_failure) = restore_manifests(
            &edits,
            &KeepAs::in_directory(&request.root.join(QUARANTINE_DIR), ".manifest.displaced"),
        );
        restore_warnings.extend(manifest_warnings);
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some(error) = lock_failure.or(manifest_failure) {
            for warning in &restore_warnings {
                eprintln!("cargo-shadow: warning: {warning}");
            }
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(other_text(error));
        }
        // Borrow `&restore_warnings`: lends the value without copying or giving it away.
        for warning in &restore_warnings {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: warning: {warning}");
        }
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Review finding (4c F1): a finish error never replaces the update's result; it is reported beside it.
    let finished: Result<(), FinishError> = transaction.finish();
    if let Err(error) = &finished {
        eprintln!("cargo-shadow: warning: {error}");
    }
    return settle_finish(outcome, finished, recovered, restore_warnings);
}
