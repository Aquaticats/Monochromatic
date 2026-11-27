//! What:
//! The transaction body of a catalog update and the one cargo call it makes:
//! `run_or_fail` runs a cargo command in a crate root and turns a non-zero exit into an error naming the crate,
//! `sources_of` copies the lockfile texts out of snapshots,
//! and `run_update` resolves or runs the user's update,
//! sets refused versions aside,
//! checks what is left and raises catalog floors when the new lockfiles are kept.
//! Why:
//! These three are one unit:
//! the body decides,
//! the diff needs plain lockfile texts,
//! and every cargo step of an update fails the same way.
//! The entry point that snapshots,
//! recovers and restores around them is `update.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update_run.ts: runOrFail(args), sourcesOf(snapshots), runUpdate(request, kind, before, produced, edits, transaction)
//! ```

/// What:
/// `use monochromatic_dependency_catalog::Settings;` imports `Settings` from the external crate `monochromatic_dependency_catalog` so this file can name it unqualified.
/// Why:
/// `run_update` reads the release age out of the settings to decide whether a cutoff exists at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::Settings;
/// What:
/// `use std::cell::RefCell;` imports `RefCell`,
/// a box whose contents can be changed through a shared reference;
/// the check that nothing else holds it happens at run time instead of at compile time.
/// Why:
/// `run_update` hands `produced` (the lockfiles this run wrote) and `edits` (the manifests it may rewrite) to cargo-calling helpers that must record into them while they hold only a shared borrow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are shared mutable references by default; a RefCell is what Rust needs to get that.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::fs;` imports the standard library's file-system module so this file can name it unqualified.
/// Why:
/// `run_update` reads and writes the catalog file when it prunes stale excludes or raises a floor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync, writeFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// A recorded cargo call answers an `io::Result` when recording itself failed,
/// and `run_update` reports that failure as the catch-all update error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an I/O failure is an ordinary thrown Error in TS
/// ```
use std::io;

/// What:
/// `use crate::cargo_run::CargoOutcome;` imports `CargoOutcome` from another module of this crate so this file can name it unqualified.
/// Why:
/// `run_or_fail` and `run_update` pass the exit status and output of one real cargo call around as this value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoOutcome } from 'crate/cargo_run';
/// ```
use crate::cargo_run::CargoOutcome;
/// What:
/// `use crate::checks::CheckOutcome;` imports `CheckOutcome` from another module of this crate so this file can name it unqualified.
/// Why:
/// The verification of the lockfiles an update leaves answers one of these,
/// and `run_update` splits it into refusals and warnings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CheckOutcome } from 'crate/checks';
/// ```
use crate::checks::CheckOutcome;
/// What:
/// `use crate::crates::LockSnapshot;` imports `LockSnapshot` from another module of this crate so this file can name it unqualified.
/// Why:
/// An update is measured by comparing the lockfile texts before it with the ones it leaves,
/// and a snapshot holds one root's text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot } from 'crate/crates';
/// ```
use crate::crates::LockSnapshot;
/// What:
/// `use crate::crates::read_locks;` imports the call that reads one snapshot per crate root.
/// Why:
/// The body reads the lockfiles the update leaves,
/// to compare them with the ones taken before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { read_locks } from 'crate/crates';
/// ```
use crate::crates::read_locks;
/// What:
/// `use crate::file_owner::Claim;` imports `Claim` from another module of this crate so this file can name it unqualified.
/// Why:
/// A recorded cargo call answers what the ownership evidence says about the files the child may have written,
/// and `run_update` ignores it here because the record itself is the point.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Claim } from 'crate/file_owner';
/// ```
use crate::file_owner::Claim;
/// What:
/// `use crate::journal::{KeepAs, QUARANTINE_DIR, ReplaceOutcome, replace_keeping};` imports the keep rule of a displaced file,
/// the folder it is kept in,
/// the outcome of a preserving replace and that replace itself.
/// Why:
/// A catalog raise writes the file only while it still holds the text that was read,
/// and a displaced copy is kept where a person can find it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { KeepAs, QUARANTINE_DIR, ReplaceOutcome, replaceKeeping } from 'crate/journal';
/// ```
use crate::journal::{
    KeepAs,
    QUARANTINE_DIR,
    ReplaceOutcome,
    replace_keeping,
};
/// What:
/// `use crate::journal::{Transaction, run_recording};` imports the transaction handle and the runner that records a cargo call.
/// Why:
/// `run_update` records every cargo call of a `--breaking` update against the transaction,
/// so a killed run is recovered and a restore can tell a concurrent writer from this run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Transaction, runRecording } from 'crate/journal';
/// ```
use crate::journal::{
    Transaction,
    run_recording,
};
/// What:
/// `use crate::lockfile::{LockedVersion, added_versions, registry_versions};` imports the record of one locked registry crate,
/// the difference between two sets of lockfile texts and the registry versions of a set of texts.
/// Why:
/// The report lists what the update newly locked,
/// the checks judge what it leaves,
/// and a floor raise compares the versions before with the versions after.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion, added_versions, registry_versions } from 'crate/lockfile';
/// ```
use crate::lockfile::{
    LockedVersion,
    added_versions,
    registry_versions,
};
/// What:
/// `use crate::manifest_edit::ManifestEdit;` imports `ManifestEdit` from another module of this crate so this file can name it unqualified.
/// Why:
/// A `--breaking` update hands the manifests it snapshotted to the transaction as these edits,
/// and the body holds the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ManifestEdit } from 'crate/manifest_edit';
/// ```
use crate::manifest_edit::ManifestEdit;
/// What:
/// `use crate::manifest_edit::snapshot_manifests;` imports the call that records a crate root's manifest texts.
/// Why:
/// A `--breaking` update takes the manifests of every crate root before cargo runs,
/// so a partial rewrite goes back with the transaction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { snapshot_manifests } from 'crate/manifest_edit';
/// ```
use crate::manifest_edit::snapshot_manifests;
/// What:
/// `use crate::prune::prune_excludes;` imports `prune_excludes` from another module of this crate so this file can name it unqualified.
/// Why:
/// A kept full update drops the exclude selectors that no locked version needs any more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { prune_excludes } from 'crate/prune';
/// ```
use crate::prune::prune_excludes;
/// What:
/// `use crate::set_aside::{SetAsideRequest, set_aside_until_clean};` imports the request of a set-aside pass and the pass itself.
/// Why:
/// A version the policy refuses is replaced by the newest older release that passes,
/// and the pass runs over every refused crate until none is left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SetAsideRequest, set_aside_until_clean } from 'crate/set_aside';
/// ```
use crate::set_aside::{
    SetAsideRequest,
    set_aside_until_clean,
};
/// What:
/// `use crate::tighten::{raise_floors, raised_names};` imports the catalog floor raise and the crate names it applies to.
/// Why:
/// `cargo update --breaking` is `pnpm update --latest`:
/// every floor whose crate moved up is raised to the version now locked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { raise_floors, raised_names } from 'crate/tighten';
/// ```
use crate::tighten::{
    raise_floors,
    raised_names,
};
/// What:
/// `use crate::time_text::publish_time_cutoff;` imports `publish_time_cutoff` from another module of this crate so this file can name it unqualified.
/// Why:
/// A full update resolves against the release-age cutoff,
/// which is the current time less the age in minutes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { publish_time_cutoff } from 'crate/time_text';
/// ```
use crate::time_text::publish_time_cutoff;
/// What:
/// `use crate::update_argument_and_warning_text::{is_breaking_option, is_precise_option, raised_floor_warning};` imports the two option tests of a `cargo update` command line and the warning line of a raised floor.
/// Why:
/// `--precise` forbids a set-aside,
/// `--breaking` makes the manifests part of the transaction,
/// and each raised floor is reported as one warning line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { is_breaking_option, is_precise_option, raised_floor_warning } from 'crate/update_argument_and_warning_text';
/// ```
use crate::update_argument_and_warning_text::{
    is_breaking_option,
    is_precise_option,
    raised_floor_warning,
};
/// What:
/// `use crate::update_error::{UpdateError, other, other_text};` imports the update's error type and the two conversions into it.
/// Why:
/// Every failure inside the transaction is reported as an `UpdateError`,
/// from a failed cargo call to an unreadable catalog.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdateError, other, otherText } from 'crate/update_error';
/// ```
use crate::update_error::{
    UpdateError,
    other,
    other_text,
};
/// What:
/// `use crate::update_manifests::{sync_recording, workspace_manifests};` imports the two manifest helpers of a `--breaking` update.
/// Why:
/// The body lists every manifest cargo may rewrite before cargo runs,
/// and renders the raised floors through the sync afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { sync_recording, workspace_manifests } from 'crate/update_manifests';
/// ```
use crate::update_manifests::{
    sync_recording,
    workspace_manifests,
};
/// What:
/// `use crate::update_recorded_work::{CargoRun, ManifestsThenCargo};` imports the two work values that record what a cargo call wrote.
/// Why:
/// A targeted update records each crate root's manifests before cargo runs and the lockfile after it,
/// so both files go back with the transaction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoRun, ManifestsThenCargo } from 'crate/update_recorded_work';
/// ```
use crate::update_recorded_work::{
    CargoRun,
    ManifestsThenCargo,
};
/// What:
/// `use crate::update_request_and_report::{UpdateKind, UpdateRequest, UpdateReport};` imports what an update is asked and what it answers.
/// Why:
/// The body takes the request and the kind,
/// and builds the report the entry point returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdateKind, UpdateRequest, UpdateReport } from 'crate/update_request_and_report';
/// ```
use crate::update_request_and_report::{
    UpdateKind,
    UpdateReport,
    UpdateRequest,
};
/// What:
/// `use crate::update_resolve::resolve_all;` imports the resolution of every crate root of a full update.
/// Why:
/// `run_update` calls it for `UpdateKind::Full`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { resolveAll } from 'crate/update_resolve';
/// ```
use crate::update_resolve::resolve_all;
/// What:
/// `use crate::update_verify::verify_update;` imports the verification an install runs on the lockfiles it leaves.
/// Why:
/// The update is judged by the lockfiles it leaves,
/// the way an install judges them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { verify_update } from 'crate/update_verify';
/// ```
use crate::update_verify::verify_update;
/// What:
/// `use std::path::PathBuf;` imports `PathBuf` from the standard library so this file can name it unqualified.
/// Why:
/// `workspace_manifests` answers with owned manifest paths,
/// and they are joined to the repository root by this file's caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for a path: string
/// ```
use std::path::PathBuf;
/// What:
/// `fn run_or_fail(request, crate_dir, args) -> Result<CargoOutcome, UpdateError>` runs one cargo command in a crate root and turns a non-zero exit into an error naming the crate.
/// `Result<T, E>` is Rust's success-or-failure value (sibling:
/// `Option<T>`,
/// a value or nothing);
/// `?` on one returns early with the failure.
/// Why:
/// Every cargo step of an update fails the same way,
/// so the message is in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runOrFail(args): Promise<CargoOutcome>  // throws
/// ```
pub(crate) fn run_or_fail(
    request: &UpdateRequest,
    crate_dir: &str,
    args: &[String],
) -> Result<CargoOutcome, UpdateError> {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      `outcome` is declared as `CargoOutcome`; only the lines after it turn a non-zero exit into an `UpdateError::Cargo`, so the type states what the runner returned.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let outcome: CargoOutcome = request
        .runner
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      The runner needs the crate root path and the arguments for the call only, so both are lent.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        .run(&request.root.join(crate_dir), args)
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      `other` is a named function, not a closure, so the runner's `String` error becomes the catch-all `UpdateError` in one `map_err` call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
        // Why:      `run_or_fail` hands the runner's `String` error to `other`, which makes it the catch-all `UpdateError`, so `?` can return it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { ... } catch (error) { throw new MyError(String(error)); }
        // ```
        .map_err(other)?;
    if !outcome.ok {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      `run_or_fail` returns a cargo failure as an `UpdateError::Cargo` value naming the crate and the command, so the caller restores and reports.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      The error stores the crate directory and the command as owned text because it outlives this call and is printed by the caller.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(UpdateError::Cargo {
            crate_dir: crate_dir.to_string(),
            command: format!("cargo {}", args.join(" ")),
            output: outcome.output,
        });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(outcome);
}

/// What:
/// `fn sources_of(snapshots) -> Vec<String>` copies the lockfile texts out of snapshots;
/// `.clone()` makes an independent copy of owned text.
/// Why:
/// The lockfile diff takes plain texts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sourcesOf = (snapshots) => snapshots.map((s) => s.source)
/// ```
fn sources_of(snapshots: &[LockSnapshot]) -> Vec<String> {
    // Closure `|snapshot|`: an anonymous function, like a TS arrow function.
    // `.clone()`: an independent owned copy.
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      `sources_of` returns plain lockfile texts to the diff, so the lazy copies must run here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    return snapshots.iter().map(LockSnapshot::source_text).collect();
}

/// What:
/// `fn run_update(request, kind, before, produced) -> Result<UpdateReport, UpdateError>` is the transaction body:
/// resolve or run the user's update,
/// set refused versions aside,
/// check what is left,
/// prune and raise catalog entries when kept.
/// `&RefCell<Vec<...>>` is a borrowed list the function may add to.
/// Why:
/// The caller restores on an error or a refusal;
/// `produced` records each lockfile the moment this run's cargo wrote it,
/// so the restore can tell a concurrent writer from this run,
/// and a root this run never wrote is never restored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runUpdate(args): Promise<UpdateReport>
/// ```
pub(crate) fn run_update(
    request: &UpdateRequest,
    kind: &UpdateKind,
    before: &[LockSnapshot],
    produced: &RefCell<Vec<LockSnapshot>>,
    edits: &RefCell<Vec<ManifestEdit>>,
    transaction: &Transaction,
) -> Result<UpdateReport, UpdateError> {
    // Borrow `&Settings`: lends the value without copying or giving it away.
    // Typed binding: `settings: &Settings`.
    let settings: &Settings = request.settings;
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      With a release age of zero there is no cutoff, and the empty text makes `resolve_crate` run a plain `generate-lockfile`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    // Typed binding: `cutoff: String`.
    // Constructor `String::new()`: an empty owned collection or a boxed value.
    let cutoff: String = if settings.minimum_release_age == 0 {
        String::new()
    } else {
        publish_time_cutoff(request.now, settings.minimum_release_age)
    };
    // `let mut`: a binding that may change.
    // Typed binding: `fell_back: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut fell_back: Vec<String> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `set_aside_allowed: bool`.
    let mut set_aside_allowed: bool = true;
    // `match`: picks the arm whose pattern fits the value.
    match kind {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&cutoff`: lends the value without copying or giving it away.
        UpdateKind::Full => fell_back = resolve_all(request, &cutoff, produced)?,
        UpdateKind::Targeted {
            cargo_args,
        } => {
            // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
            // Why:      `args` starts as a copy of the user's cargo arguments so forwarded options can be appended without changing the caller's slice.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // [...slice]
            // ```
            // `let mut`: a binding that may change.
            // Typed binding: `args: Vec<String>`.
            let mut args: Vec<String> = cargo_args.to_vec();
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            args.extend(request.forward.iter().cloned());
            // An explicit `--precise` names the version the user wants: nothing is set aside in its place, and the policy judges
            // that version after the run whether or not the command changed the lockfile (`precise_target.rs`).
            set_aside_allowed = !cargo_args.iter().map(String::as_str).any(is_precise_option);
            // Review finding (`--breaking` edits outside the transaction): cargo's breaking update rewrites the requirements in the
            // manifests, so their text is taken before the run and restored with the lockfiles when the update is not kept.
            let breaking: bool = cargo_args
                .iter()
                .map(String::as_str)
                .any(is_breaking_option);
            if breaking {
                for crate_dir in request.crate_dirs {
                    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                    // Typed binding: `paths: Vec<PathBuf>`.
                    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
                    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // // TS has one type for this: string
                    // ```
                    let paths: Vec<PathBuf> = workspace_manifests(request, crate_dir)?;
                    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                    // Borrow `&paths`: lends the value without copying or giving it away.
                    // `.map_err(...)`: converts the callee's error into this function's error type.
                    snapshot_manifests(crate_dir, &paths, edits).map_err(other_text)?;
                }
                // Delegated pick (2026-11-03): the manifests are journaled, so a killed run is recovered for them too.
                transaction
                    .add_manifests(&edits.borrow())
                    .map_err(other_text)?;
            }
            for crate_dir in request.crate_dirs {
                // Both the lockfile and the manifests of this root are attributed by the evidence in `file_owner.rs`, whether the
                // call succeeds or fails; the manifests of other roots are never claimed by this call.
                let (ran, _claim): (io::Result<Result<CargoOutcome, UpdateError>>, Claim) =
                    run_recording(
                        request.root,
                        crate_dir,
                        produced,
                        ManifestsThenCargo {
                            cargo: CargoRun {
                                request,
                                crate_dir,
                                args: &args,
                            },
                            edits,
                        },
                    )
                    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                    // `.map_err(...)`: converts the callee's error into this function's error type.
                    .map_err(other_text)?;
                // `.map_err(...)`: converts the callee's error into this function's error type.
                ran.map_err(other_text)??;
            }
        }
    }
    // Typed binding: `before_sources: Vec<String>`.
    let before_sources: Vec<String> = sources_of(before);
    // `let mut`: a binding that may change.
    // Typed binding: `warnings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut warnings: Vec<String> = Vec::new();
    if set_aside_allowed {
        // Borrow `&SetAsideRequest`: lends the value without copying or giving it away.
        warnings = set_aside_until_clean(&SetAsideRequest {
            root: request.root,
            crate_dirs: request.crate_dirs,
            // Borrow `&before_sources`: lends the value without copying or giving it away.
            before: &before_sources,
            registry: request.registry,
            settings,
            now: request.now,
            runner: request.runner,
            forward: request.forward,
            produced,
        })
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        .map_err(other)?;
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `after_snapshots: Vec<LockSnapshot>`.
    let after_snapshots: Vec<LockSnapshot> =
        read_locks(request.root, request.crate_dirs).map_err(other_text)?;
    // Borrow `&after_snapshots`: lends the value without copying or giving it away.
    // Typed binding: `after: Vec<String>`.
    let after: Vec<String> = sources_of(&after_snapshots);
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&before_sources`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `added: Vec<LockedVersion>`.
    let added: Vec<LockedVersion> = added_versions(&before_sources, &after).map_err(other)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // Borrow `&added`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `age: CheckOutcome`.
    // Review finding (3c P1-2): the update is judged by the lockfiles it leaves, the way an install judges them, not by the
    // version pairs it added: a refused version that was already locked, or an edge moved to one, is not new as a pair.
    let outcome: CheckOutcome = verify_update(request, &after_snapshots)?;
    // `let mut`: a binding that may change.
    // Typed binding: `problems: Vec<String>`.
    let mut problems: Vec<String> = outcome.problems;
    warnings.extend(outcome.warnings);
    // Typed binding: `keep: bool`.
    let mut keep: bool = problems.is_empty() && !request.dry_run;
    // `let mut`: a binding that may change.
    // Typed binding: `pruned: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut pruned: Vec<String> = Vec::new();
    // Macro `matches!`: builds or prints text (or a list) from a template.
    if keep && request.prune_catalog && matches!(kind, UpdateKind::Full) {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `source: String`.
        let source: String = fs::read_to_string(request.catalog_path).map_err(other_text)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&after`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `locked: Vec<LockedVersion>`.
        let locked: Vec<LockedVersion> = registry_versions(&after).map_err(other)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&source`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        let result = prune_excludes(&source, settings, &locked).map_err(other)?;
        if !result.removed.is_empty() {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&result.source`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            fs::write(request.catalog_path, &result.source).map_err(other_text)?;
        }
        pruned = result.removed;
    }
    // `--breaking` is `pnpm update --latest`: the new range goes into the catalog entry and the sync renders it (tighten.rs).
    if keep
        && let UpdateKind::Targeted {
            cargo_args,
        } = kind
        && cargo_args
            .iter()
            .map(String::as_str)
            .any(is_breaking_option)
    {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&before_sources`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `before_versions: Vec<LockedVersion>`.
        let before_versions: Vec<LockedVersion> =
            registry_versions(&before_sources).map_err(other)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&after`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `after_versions: Vec<LockedVersion>`.
        let after_versions: Vec<LockedVersion> = registry_versions(&after).map_err(other)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `source: String`.
        let source: String = fs::read_to_string(request.catalog_path).map_err(other_text)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&source`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        let (raised_source, changes) = raise_floors(
            &source,
            &after_versions,
            &raised_names(&before_versions, &after_versions),
        )
        .map_err(other)?;
        if !changes.is_empty() {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&raised_source`: lends the value without copying or giving it away.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            let catalog_keep: KeepAs =
                KeepAs::in_directory(&request.root.join(QUARANTINE_DIR), ".catalog.displaced");
            // Review finding (2c 6): the catalog is replaced only while it still holds the text that was read, never over another
            // writer's edit.
            let raised: ReplaceOutcome =
                replace_keeping(request.catalog_path, &source, &raised_source, &catalog_keep)
                    .map_err(other_text)?;
            // Macro `matches!`: builds or prints text (or a list) from a template.
            if !matches!(raised, ReplaceOutcome::Replaced(_)) {
                // Macro `format!`: builds or prints text (or a list) from a template.
                problems.push(format!("the catalog changed while the update ran, so its floors were not raised and the update was not kept ({raised:?})"));
                keep = false;
            }
            // Closure `|change|`: an anonymous function, like a TS arrow function.
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            // Adapter `.map(`: transforms or selects items with a closure.
            // Macro `format!`: builds or prints text (or a list) from a template.
            warnings.extend(changes.iter().map(String::as_str).map(raised_floor_warning));
            // `match`: picks the arm whose pattern fits the value.
            // Review finding (3c P2-3): an error from recording the manifests is a failed sync like any other, so the catalog goes back.
            let synced: Result<String, String> = if keep {
                // `match`: picks the arm whose pattern fits the value.
                match sync_recording(request, edits) {
                    // `Ok(...)`: builds the success or the failure variant of a `Result`.
                    Ok(answer) => answer,
                    // `Err(...)`: builds the success or the failure variant of a `Result`.
                    // Macro `format!`: builds or prints text (or a list) from a template.
                    Err(error) => Err(format!(
                        "the manifests could not be recorded around the sync ({error})"
                    )),
                }
            } else {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                Err("the catalog was not raised".to_string())
            };
            // `match`: picks the arm whose pattern fits the value.
            match synced {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(done) => warnings.push(done),
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // Delegated pick (2026-11-03): the sync belongs to the transaction. A catalog raised but not rendered would leave
                // the floors and the manifests disagreeing, so the catalog goes back and the caller restores the manifests and
                // the lockfiles; the finding names the failure.
                Err(message) => {
                    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                    // Borrow `&source`: lends the value without copying or giving it away.
                    // `.map_err(...)`: converts the callee's error into this function's error type.
                    // The catalog goes back only while it still holds the raised text this run wrote; an edit made meanwhile is kept.
                    if keep {
                        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                        // Borrow `&raised_source`: lends the value without copying or giving it away.
                        // `.map_err(...)`: converts the callee's error into this function's error type.
                        // Typed binding: `back: ReplaceOutcome`.
                        let back: ReplaceOutcome = replace_keeping(
                            request.catalog_path,
                            &raised_source,
                            &source,
                            &catalog_keep,
                        )
                        .map_err(other_text)?;
                        // Macro `matches!`: builds or prints text (or a list) from a template.
                        if !matches!(back, ReplaceOutcome::Replaced(_)) {
                            // Macro `format!`: builds or prints text (or a list) from a template.
                            warnings.push(format!("the catalog was edited while the sync ran and keeps that edit ({back:?})"));
                        }
                        // Macro `format!`: builds or prints text (or a list) from a template.
                        problems.push(format!("the catalog floors were not rendered into the manifests, so the update was not kept: {message}"));
                    }
                    keep = false;
                }
            }
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.to_vec()`: copies the slice into a new owned `Vec`.
    return Ok(UpdateReport {
        crate_dirs: request.crate_dirs.to_vec(),
        added,
        problems,
        warnings,
        fell_back,
        written: keep,
        pruned,
    });
}
