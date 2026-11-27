//! What:
//! The install step,
//! the counterpart of `pnpm install`:
//! makes each listed crate root's `Cargo.lock` satisfy its manifests,
//! moving as little as the manifests force.
//! Why:
//! `mise run sync:files` renders catalog floors and path-crate versions into the manifests and never touches a lockfile,
//! so a
//! crate whose lockfile held an older release is left with a lockfile its own manifest refuses;
//! any plain cargo run then
//! rewrites it through cargo's own resolver,
//! past the trust policy.
//! pnpm repairs the same situation with `pnpm install`,
//! and follows these lockfile rules (<https://pnpm.io/settings/build>,
//! <https://pnpm.io/settings/store>,
//! read 2026-11-03):
//!
//! - Outside CI a stale lockfile is updated (`--no-frozen-lockfile`).
//! - In CI,
//!   or with `--frozen-lockfile`,
//!   a stale lockfile is a failure that names the roots and writes nothing.
//! - `--dry-run` reports what an install would update and writes nothing.
//! - A root that still satisfies its manifests keeps its lockfile byte for byte;
//!   in a stale root the dependencies whose
//!   requirements still hold keep their locked versions.
//!
//! Per crate root,
//! offline:
//! `cargo update --workspace --locked --offline` asks whether the lockfile already satisfies the
//! manifests (reads the index only,
//! writes nothing);
//! a refusal means stale;
//! `cargo update --workspace --offline` re-locks the
//! workspace members and what their changed requirements force;
//! `cargo metadata --locked --offline` confirms the result.
//! A
//! non-frozen install is one transaction:
//! every lockfile is snapshotted first,
//! and any failure or refusal restores all of
//! them.
//! A registry version that no lockfile held before goes through the release-age and trust checks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install.ts
//! export async function installRustLockfiles({ root, frozenLockfile, dryRun, now, runCargo, registry }): Promise<InstallReport>
//! ```

/// What:
/// `use crate::cargo_run::CargoRunner;` imports the trait that runs the real cargo for the install step.
/// Why:
/// `InstallRequest` holds one,
/// so a test passes a fake cargo in place of the real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoRunner } from 'crate/cargo_run';
/// ```
use crate::cargo_run::CargoRunner;
/// What:
/// `use crate::checks::CheckOutcome;` imports the findings of the policy checks.
/// Why:
/// The frozen path answers with one,
/// empty when every locked version passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CheckOutcome } from 'crate/checks';
/// ```
use crate::checks::CheckOutcome;
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports the snapshot of one lockfile and the call that reads one per crate root.
/// Why:
/// The entry point snapshots every lockfile before the transaction and hands those snapshots to the restore.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, read_locks } from 'crate/crates';
/// ```
use crate::crates::{
    LockSnapshot,
    read_locks,
};
/// The failure of an install (it lives in `install_error.rs`).
pub use crate::install_error::InstallError;
/// What:
/// `use crate::install_error::{InstallError, io_error, registry_error, unapproved_roots};` imports the failure of an install and the three
/// classifications this file reports.
/// Why:
/// A journal that cannot start or a restore that fails is an I/O failure over a file this run wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { InstallError, io_error, registry_error, unapproved_roots } from 'crate/install_error';
/// ```
use crate::install_error::{
    io_error,
    registry_error,
    unapproved_roots,
};
/// The lockfile texts of snapshots (split out of this module).
pub(crate) use crate::install_stale_roots::sources_of;
/// What:
/// `use crate::install_stale_roots::{sources_of, stale_roots, update_stale_roots};` imports the read-only staleness check,
/// the transaction body and the lockfile texts.
/// Why:
/// A frozen install asks only whether any lockfile is stale,
/// and a non-frozen one hands the snapshots to the body.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { sources_of, stale_roots, update_stale_roots } from 'crate/install_stale_roots';
/// ```
use crate::install_stale_roots::update_stale_roots;
/// The cargo command lines the install step runs and the update of a stale root (split out of this module).
pub use crate::install_stale_roots::{
    CHECK_ARGS,
    UPDATE_ARGS,
    VERIFY_ARGS,
    arguments_for,
    is_continuous_integration,
    is_stale,
    stale_roots,
};
/// What:
/// `use crate::install_verify::{LockBaseReader, PolicyJudge, base_reader, verification_scope};` and the four verification calls
/// (`is_verified`,
/// `mark_verified`,
/// `verify_added_since_base_outcome`,
/// `verify_locked_entries_outcome`).
/// Why:
/// The frozen path verifies the lockfiles it finds,
/// and the same remembered passes tell it what to skip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockBaseReader, PolicyJudge, base_reader, verification_scope, is_verified, mark_verified, verify_added_since_base_outcome, verify_locked_entries_outcome } from 'crate/install_verify';
/// ```
use crate::install_verify::base_reader;
/// The helpers split out of this file before it,
/// so callers keep their paths.
pub use crate::install_verify::{
    LockBaseFactory,
    LockBaseReader,
    PolicyJudge,
    policy_findings,
    verification_cached,
    verification_scope,
};
/// What:
/// `use crate::journal::{begin, recover_report, restore_if_unchanged};` imports the start of a transaction over the snapshots,
/// the recovery of an earlier run and the restore that skips a lockfile another writer changed.
/// Why:
/// These three are the transaction around an install:
/// finish what a killed run left,
/// journal this one,
/// and put the lockfiles back when nothing is kept.
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
/// `use crate::journal_finish::{FinishError, settle_finish};` imports the failure of finishing a journal and the call that joins it with the install's own result.
/// Why:
/// The entry point finishes the transaction and reports a finish error beside the install's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { FinishError, settle_finish } from 'crate/journal_finish';
/// ```
use crate::journal_finish::FinishError;
/// What:
/// `use crate::lockfile::LockedVersion;` imports the record of one locked registry crate.
/// Why:
/// `InstallReport` lists the versions the install newly locked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion } from 'crate/lockfile';
/// ```
use crate::lockfile::LockedVersion;
/// What:
/// `use crate::registry::Registry;` imports the trait the policy checks ask crates.io through.
/// Why:
/// `InstallRequest` holds one,
/// so a test answers the checks from memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry } from 'crate/registry';
/// ```
use crate::registry::Registry;
/// What:
/// `use crate::verify::{is_verified, mark_verified, verify_added_since_base_outcome, verify_locked_entries_outcome};` imports the two
/// verifications a frozen install runs and the remembered pass they read and write.
/// Why:
/// Under `trustLockfile` only the versions new since the base are checked,
/// and otherwise every locked entry is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { is_verified, mark_verified, verify_added_since_base_outcome, verify_locked_entries_outcome } from 'crate/verify';
/// ```
use crate::verify::{
    is_verified,
    mark_verified,
    verify_added_since_base_outcome,
    verify_locked_entries_outcome,
};
/// What:
/// `use monochromatic_dependency_catalog::Settings;` imports the release-age and trust settings from the external catalog crate.
/// Why:
/// `InstallRequest` holds them,
/// and they decide which lockfile entries the frozen path checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::Settings;
/// What:
/// `use std::cell::RefCell;` imports a box whose contents can be changed through a shared reference (sibling:
/// `&mut`,
/// which needs exclusive access).
/// Why:
/// The record of what this run's cargo calls wrote grows while the transaction body holds a shared reference to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::path::Path;` imports the borrowed path type from the standard library so this file can name it unqualified.
/// Why:
/// `InstallRequest` borrows the repository root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::Path;
/// The outcome of an install run.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstallReport {
    /// Every crate root examined.
    pub crate_dirs: Vec<String>,
    /// Roots whose lockfile did not satisfy its manifests (updated,
    /// or only reported under `--dry-run`).
    pub stale: Vec<String>,
    /// Registry versions the install newly locked (kept or restored).
    pub added: Vec<LockedVersion>,
    /// Findings that refused the install.
    pub problems: Vec<String>,
    /// Findings reported without refusing.
    pub warnings: Vec<String>,
    /// Whether updated lockfiles were kept.
    pub written: bool,
}

/// What an install run needs.
pub struct InstallRequest<'a> {
    /// The repository root.
    pub root: &'a Path,
    /// The crate roots to examine,
    /// repo-relative.
    pub crate_dirs: &'a [String],
    /// Fail on a stale lockfile instead of updating it (pnpm's `--frozen-lockfile`,
    /// the CI default).
    pub frozen: bool,
    /// Keep every cargo call offline;
    /// false in CI,
    /// where no registry index is cached yet.
    pub offline: bool,
    /// Report what would be updated,
    /// then restore every lockfile.
    pub dry_run: bool,
    /// The current time in milliseconds.
    pub now: i64,
    /// The catalog settings.
    pub settings: &'a Settings,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// The registry for the policy checks.
    pub registry: &'a dyn Registry,
    /// Makes the reader of one crate's lockfile text at the base revision (empty when absent there);
    /// an install under `trustLockfile` checks only the versions this reader's lockfiles did not hold.
    /// A factory,
    /// not a reader,
    /// because resolving the base starts git,
    /// which a call whose checks are all remembered must not pay for.
    pub lock_base: Option<LockBaseFactory<'a>>,
    /// The key of the base revision:
    /// the commit it resolved to
    /// (so a base that moves is a different base),
    /// or `unresolved:<spelling>` when it names no commit
    /// (`shadow_step::trust_base_setup` supplies it);
    /// part of the key of a remembered pass under `trustLockfile`,
    /// so a pass against one base is never taken for another.
    pub trust_base: &'a str,
    /// The freshness stamp already showed that every lockfile satisfies its manifests,
    /// so the staleness check (a cargo call) is skipped;
    /// the policy checks are not.
    pub known_consistent: bool,
}

/// What:
/// `pub(crate) fn refusal_of_writers(nouns: &str, shortfall: &str) -> String` is why a run's stop refused it,
/// in the words a person reads.
/// Why:
/// The install and the update refuse for the same reason and differ only in what the run would have written,
/// so the words are built once here and both interpolate the one sentence about the journal
/// (`journal_recover::KEPT_JOURNAL`),
/// which names no scope because a stop of a bare process group keeps the same journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function refusalOfWriters(nouns: string, shortfall: string): string
/// ```
pub(crate) fn refusal_of_writers(
    nouns: &str,
    shortfall: &str,
) -> String {
    // What:     `if shortfall.is_empty() { String::new() } else { format!(": {shortfall}") }` keeps the colon inside the reason.
    // Why:      A stop that recorded no reason (a test's stand-in) must not print a colon with nothing after it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const reason = shortfall === '' ? '' : `: ${shortfall}`;
    // ```
    let reason: String = if shortfall.is_empty() {
        String::new()
    } else {
        format!(": {shortfall}")
    };
    return format!(
        "a process started by cargo could not be stopped, so it may still write the {nouns}{reason}; they were not rolled back or accepted; {}",
        crate::journal_recover::KEPT_JOURNAL
    );
}

/// What:
/// `pub fn install_rust_lockfiles(request: &InstallRequest) -> Result<InstallReport, InstallError>` runs the install over the
/// listed crate roots.
/// Why:
/// A frozen install only checks and fails naming the stale roots.
/// A non-frozen install first finishes any run that was killed
/// (`journal.rs`),
/// then snapshots every lockfile,
/// updates the stale ones,
/// checks the newly locked versions,
/// and puts the
/// snapshots back on a failure,
/// a refusal or a dry run,
/// but only into lockfiles that still hold what this run produced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function installRustLockfiles(args): Promise<InstallReport>  // throws StaleLockfilesError, CargoInstallError
/// ```
pub fn install_rust_lockfiles(request: &InstallRequest) -> Result<InstallReport, InstallError> {
    if request.frozen {
        // A frozen run starts no transaction, so the kept-files warning is announced here (it is promised on every run past the count).
        crate::journal_transaction::announce_kept_files_for(request.root, request.crate_dirs);
        // The stamp showed the lockfiles consistent, so no cargo call asks again; the policy check below still runs.
        if !request.known_consistent {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Typed binding: `stale: Vec<String>`.
            let stale: Vec<String> = stale_roots(request)?;
            if !stale.is_empty() {
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!(
                    "cargo-shadow: frozen install: {} stale lockfiles, nothing written",
                    stale.len()
                );
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                return Err(InstallError::Stale {
                    stale_dirs: stale,
                });
            }
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `locks: Vec<LockSnapshot>`.
        let locks: Vec<LockSnapshot> =
            read_locks(request.root, request.crate_dirs).map_err(io_error)?;
        // Typed binding: `outcome: CheckOutcome`.
        let outcome: CheckOutcome = if request.settings.trust_lockfile {
            // Typed binding: `scope: String`.
            let scope: String = verification_scope(request.settings, true, request.trust_base);
            // `let mut`: a binding that may change.
            // Typed binding: `pending: Vec<LockSnapshot>`.
            // Constructor `Vec::new()`: an empty owned collection or a boxed value.
            let mut pending: Vec<LockSnapshot> = Vec::new();
            // Borrow `&locks`: lends the value without copying or giving it away.
            for snapshot in &locks {
                // Borrow `&scope`: lends the value without copying or giving it away.
                if !is_verified(request.root, snapshot, request.settings, &scope) {
                    // `.clone()`: an independent owned copy.
                    pending.push(snapshot.clone());
                }
            }
            if pending.is_empty() {
                // Constructor `Vec::new()`: an empty owned collection or a boxed value.
                CheckOutcome::default()
            } else {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Typed binding: `read_base: LockBaseReader`.
                let read_base: LockBaseReader = base_reader(request)?;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Borrow `&pending`: lends the value without copying or giving it away.
                // `.map_err(...)`: converts the callee's error into this function's error type.
                // Typed binding: `found: Vec<String>`.
                let found: CheckOutcome = verify_added_since_base_outcome(
                    &pending,
                    &*read_base,
                    request.settings,
                    request.registry,
                    request.now,
                )
                .map_err(registry_error)?;
                // A pass with a warning is not remembered, so the warning is shown again.
                if found.problems.is_empty() && found.warnings.is_empty() {
                    // Borrow `&pending`: lends the value without copying or giving it away.
                    for snapshot in &pending {
                        // Borrow `&scope`: lends the value without copying or giving it away.
                        mark_verified(request.root, snapshot, request.settings, &scope);
                    }
                }
                found
            }
        } else {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Borrow `&locks`: lends the value without copying or giving it away.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            verify_locked_entries_outcome(
                request.root,
                &locks,
                request.settings,
                request.registry,
                request.now,
            )
            .map_err(registry_error)?
        };
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
        // Why:      The result can be stored or modified independently of the slice it came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // [...slice]
        // ```
        return Ok(InstallReport {
            crate_dirs: request.crate_dirs.to_vec(),
            problems: outcome.problems,
            warnings: outcome.warnings,
            ..InstallReport::default()
        });
    }
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
    // A mandatory phase of finishing an earlier run's journal failed (its outputs are not durable, or its completion marker is not):
    // that journal is not finished, so this command does not go on. A failed removal of a journal already marked done only warns.
    if mandatory_recovery_failure {
        return Err(InstallError::Unsettled {
            message: "cargo-shadow: the recovery of an earlier run could not finish its journal (see the warnings above), so nothing was changed by this command and it was not run; fix the cause and run the command again".to_string(),
        });
    }
    // Typed binding: `before: Vec<LockSnapshot>`.
    let before: Vec<LockSnapshot> =
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        read_locks(request.root, request.crate_dirs).map_err(io_error)?;
    // Macro `eprintln!`: builds or prints text (or a list) from a template.
    eprintln!("cargo-shadow: snapshotted {} lockfiles", before.len());
    // Borrow `&before`: lends the value without copying or giving it away.
    // Review finding (failed begin): without a complete recovery record nothing may be mutated, so a journal that cannot be
    // started stops the install here, before the first cargo call.
    let transaction = begin(request.root, &before, false).map_err(io_error)?;
    if let Some(notice) = transaction.kept_notice() {
        crate::journal_transaction::announce_kept_notice(notice);
    }
    // What:     `RefCell<Vec<LockSnapshot>>` is a list that can be added to through a shared reference (sibling: `&mut Vec`, which needs exclusive access); `RefCell::new(Vec::new())` makes an empty one.
    // Why:      `produced` starts empty because cargo has not written yet; each cargo write is recorded the moment it succeeded, so a failure never has to guess what the run wrote.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const produced: LockSnapshot[] = [];
    // ```
    let produced: RefCell<Vec<LockSnapshot>> = RefCell::new(Vec::new());
    // Borrow `&before`: lends the value without copying or giving it away.
    // Typed binding: `outcome: Result<InstallReport, InstallError>`.
    let outcome: Result<InstallReport, InstallError> =
        update_stale_roots(request, &before, &produced);
    // A process that cargo started could not be stopped: it may still write the lockfiles, so they are neither rolled back nor
    // accepted. The transaction is dropped without `finish`, the journal stays, and the command does not run (round 6, 5c N2).
    if transaction.writers_may_remain() {
        // What:     `transaction.writers_may_remain_reason()` is `Some(text)` when the stop recorded why it could not show every
        //           writer gone, and `None` for a run that was marked without one (a test's stand-in).
        // Why:      The refusal says how long the stop waited and what it was waiting for, so a member that has exited is told
        //           from one that is alive and a loaded host from a member that cannot be killed (issue 6528).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const shortfall = transaction.writersMayRemainReason() ?? '';
        // return refused(refusalOfWriters('lockfiles', shortfall));
        // ```
        let shortfall: String = transaction.writers_may_remain_reason().unwrap_or_default();
        return Err(InstallError::Unsettled {
            message: format!(
                "cargo-shadow: {}",
                refusal_of_writers("lockfiles", &shortfall)
            ),
        });
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
        let (restore_found, restore_failure) =
            restore_if_unchanged(request.root, &before, &produced.borrow());
        // A failed root must not hide what the other roots reported: the warnings travel in the failure's text.
        let restored: std::io::Result<Vec<String>> = match restore_failure {
            None => Ok(restore_found),
            Some(error) if restore_found.is_empty() => Err(error),
            Some(error) => Err(std::io::Error::other(format!(
                "{error}; the other lockfiles reported: {}",
                restore_found.join("; ")
            ))),
        };
        // Review finding (2c 1): output of this run's cargo that is still live after the rollback is unapproved, and the shim's
        // `--locked` fallback would build it, so the step stops instead of reporting an operational failure.
        let still_live: Vec<String> = unapproved_roots(request.root, &produced.borrow());
        // Review finding (3c P1-1): a policy refusal is a successful report with problems, not an error; a rollback that cannot
        // complete stops the command whether cargo failed or its output was refused, and the refusal travels in the message.
        if !still_live.is_empty() || restored.is_err() {
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&outcome`: lends the value without copying or giving it away.
            // Typed binding: `refusal: String`.
            let refusal: String = match &outcome {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                Ok(report) if !report.problems.is_empty() => {
                    format!("\nThe policy refused it: {}", report.problems.join("; "))
                }
                // The command's own failure came first: the rollback failure is added to it, never put in its place.
                Err(original) => format!("\nThe run had already failed: {original}"),
                // Constructor `String::new()`: an empty owned collection or a boxed value.
                _ => String::new(),
            };
            return Err(InstallError::Unsettled {
                message: format!(
                    "cargo-shadow: this run's cargo changed {} and the change could not be put back ({}), so the command was not run{refusal}",
                    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                    if still_live.is_empty() {
                        "a lockfile".to_string()
                    } else {
                        still_live.join(", ")
                    },
                    match &restored {
                        Ok(warnings) => warnings.join("; "),
                        Err(error) => error.to_string(),
                    }
                ),
            });
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        restore_warnings = restored.map_err(io_error)?;
        // Borrow `&restore_warnings`: lends the value without copying or giving it away.
        for warning in &restore_warnings {
            // Macro `eprintln!`: builds or prints text (or a list) from a template.
            eprintln!("cargo-shadow: warning: {warning}");
        }
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Review finding (4c F1): a finish error never replaces an established refusal (or any other result): it is reported beside it,
    // and the journal stays for the next recovery to settle.
    let finished: Result<(), FinishError> = transaction.finish();
    if let Err(error) = &finished {
        eprintln!("cargo-shadow: warning: {error}");
    }
    return match (outcome, finished) {
        (Ok(mut report), Ok(())) => {
            report.warnings.extend(recovered);
            report.warnings.extend(restore_warnings);
            Ok(report)
        }
        // The policy refused the run: the refusal stands, and the failed barrier is one more thing the person must know.
        (Ok(mut report), Err(error)) if !report.problems.is_empty() => {
            report.warnings.extend(recovered);
            report.warnings.extend(restore_warnings);
            if error.cleanup_only() {
                report.warnings.push(error.to_string());
            } else {
                report.problems.push(error.to_string());
            }
            Ok(report)
        }
        // A failed barrier is a stopped run, never a clean pass: the command must not run on a lockfile that may not survive.
        (Ok(_), Err(error)) if !error.cleanup_only() => Err(InstallError::Unsettled {
            message: format!("cargo-shadow: {error}; the command was not run, so run it again"),
        }),
        (Ok(mut report), Err(error)) => {
            report.warnings.extend(recovered);
            report.warnings.extend(restore_warnings);
            report.warnings.push(error.to_string());
            Ok(report)
        }
        // An operational error alone lets the shim run the command again with `--locked`; a failed mandatory phase of finishing
        // forbids that, so both diagnostics travel in the stopping error (round 7, 6c R1).
        (Err(original), Err(error)) if !error.cleanup_only() => Err(InstallError::Unsettled {
            message: format!(
                "cargo-shadow: {error}; the command was not run, so run it again\nThe install step had also failed: {original}"
            ),
        }),
        (Err(error), _) => Err(error),
    };
}

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
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "install_policy_tests.rs"]
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
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "install_tests.rs"]
mod install_tests;
