//! What:
//! Replaces a newly locked version that the release-age or trust policy refuses with the newest older release in the same
//! caret class that passes,
//! the way pnpm's picker skips a version the policy refuses and takes the next best.
//! Why:
//! `cargo update` has no release-age cutoff and no trust policy,
//! so cargo picks the newest release;
//! this step moves the
//! refused crates down with `cargo update --package <name>@<version> --precise <candidate>`,
//! in rounds,
//! because a move can
//! lock new transitive versions that need checking in turn.
//! A refusal with no workable candidate stays a finding.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // set-aside.ts
//! export async function setAsideUntilClean(args): Promise<string[]>  // warnings
//! ```

/// Imports `RefCell`,
/// a list or value that can be changed through a shared reference;
/// it holds the record of the lockfiles this run wrote.
use std::cell::RefCell;
/// What:
/// `use std::collections::BTreeSet;` imports `BTreeSet` from the standard library so this file can name it unqualified.
/// Why:
/// `BTreeSet` is used by `set_aside_until_clean`
/// to hold the versions this run already refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BTreeSet } from 'std/collections';
/// ```
use std::collections::BTreeSet;
/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `SetAsideRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;
/// The owned path type of the directory a recorded cargo command runs in.
use std::path::PathBuf;

/// What:
/// `use monochromatic_dependency_catalog::Settings;` imports `Settings` from the external crate `monochromatic_dependency_catalog` so this file can name it unqualified.
/// Why:
/// `Settings` is used by `SetAsideRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::Settings;

/// What:
/// `use crate::adapt::message_of;` imports `message_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `message_of` is used by `set_aside_until_clean`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { message_of } from 'crate/adapt';
/// ```
use crate::adapt::message_of;
/// What:
/// `use crate::adapt::text_of;` imports `text_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `text_of` is used by `set_aside_until_clean`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { text_of } from 'crate/adapt';
/// ```
use crate::adapt::text_of;
/// What:
/// `use crate::cargo_run::CargoRunner;` imports `CargoRunner` from another module of this crate so this file can name it unqualified.
/// Why:
/// `CargoRunner` is used by `SetAsideRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoRunner } from 'crate/cargo_run';
/// ```
use crate::cargo_run::{
    CargoOutcome,
    CargoRunner,
};
/// What:
/// `use crate::checks::{TrustRefusal, trust_refusals};` imports `TrustRefusal`,
/// `trust_refusals` from another module of this crate so this file can name them unqualified.
/// Why:
/// `TrustRefusal` is used by `move_crate`
/// and others;
/// `trust_refusals` is used by `set_aside_until_clean`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TrustRefusal, trust_refusals } from 'crate/checks';
/// ```
use crate::checks::{
    TrustRefusal,
    trust_refusals,
};
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports `LockSnapshot`,
/// `read_locks` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LockSnapshot` is used by `crates_locking`,
/// `set_aside_until_clean`;
/// `read_locks` is used by `set_aside_until_clean`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, read_locks } from 'crate/crates';
/// ```
use crate::crates::{
    LockSnapshot,
    read_locks,
};
/// Imports `Claim`,
/// what the ownership evidence says about a file a child wrote.
use crate::file_owner::Claim;
/// Imports `run_recording`,
/// which runs a child and records the lockfile this run's cargo wrote.
use crate::journal::run_recording;
/// What:
/// `use crate::lockfile::{LockedVersion, added_versions, registry_versions};` imports `LockedVersion`,
/// `added_versions`,
/// `registry_versions` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LockedVersion` is used by `set_aside_until_clean`;
/// `added_versions` is used by `set_aside_until_clean`;
/// `registry_versions` is used by `crates_locking`.
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
/// `use crate::registry::Registry;` imports `Registry` from another module of this crate so this file can name it unqualified.
/// Why:
/// `Registry` is used by `SetAsideRequest`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry } from 'crate/registry';
/// ```
use crate::registry::Registry;
/// Imports the release-age refusals of one round (split out of this module).
use crate::set_aside_older_release_choice::age_refusals;
/// The candidate list for one refused version,
/// re-exported so its path here is unchanged (split out of this module).
pub use crate::set_aside_older_release_choice::set_aside_candidates;

/// How many rounds of moving refused crates are tried.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
pub const SET_ASIDE_ROUNDS: usize = 5;

/// How many candidates are tried per refused version and crate.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
pub const PRECISE_ATTEMPTS: usize = 10;

/// What a set-aside round needs.
pub struct SetAsideRequest<'a> {
    /// The repository root.
    pub root: &'a Path,
    /// The crate roots in play.
    pub crate_dirs: &'a [String],
    /// Lockfile texts before the whole run.
    pub before: &'a [String],
    /// The registry.
    pub registry: &'a dyn Registry,
    /// The catalog settings.
    pub settings: &'a Settings,
    /// The current time in milliseconds.
    pub now: i64,
    /// The newest allowed publish time in milliseconds;
    /// zero for no cutoff.
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// Options forwarded to every cargo call (`--offline`,
    /// `--config`,
    /// ...).
    pub forward: &'a [String],
    /// The record of the lockfiles this run wrote;
    /// every successful `--precise` move is added,
    /// so a rollback knows the root is this run's own.
    pub produced: &'a RefCell<Vec<LockSnapshot>>,
}

/// What:
/// `struct RunnerWork<'a> { runner, directory, args }` is the work of running one cargo command through a runner in a crate root.
/// Why:
/// `run_recording` takes the work as a value,
/// and this holds what the closure that stood here captured.
struct RunnerWork<'a> {
    /// The runner that starts cargo.
    runner: &'a dyn CargoRunner,
    /// The directory cargo runs in.
    directory: PathBuf,
    /// The arguments of the cargo command.
    args: &'a [String],
}

/// What:
/// The implementation of `RecordedWork` for `RunnerWork`.
/// Why:
/// `run_recording` runs it between its before and after observations.
impl crate::file_owner::RecordedWork<Result<CargoOutcome, String>> for RunnerWork<'_> {
    /// What:
    /// Runs the command through the runner.
    /// Why:
    /// The result is the one the closure returned.
    fn run(self) -> Result<CargoOutcome, String> {
        return self.runner.run(&self.directory, self.args);
    }
}

/// What:
/// `fn locks_key(versions: &[LockedVersion], key: &str) -> bool` says whether any locked version has the key.
/// Why:
/// A loop replaces the `any` closure that captured the key.
fn locks_key(
    versions: &[LockedVersion],
    key: &str,
) -> bool {
    for locked in versions {
        if locked.key() == key {
            return true;
        }
    }
    return false;
}

/// What:
/// `fn is_pending(pending: &[TrustRefusal], refusal: &TrustRefusal) -> bool` says whether a refusal of the same name and version is already pending.
/// Why:
/// A loop replaces the `any` closure that captured the refusal.
fn is_pending(
    pending: &[TrustRefusal],
    refusal: &TrustRefusal,
) -> bool {
    for known in pending {
        if known.name == refusal.name && known.version == refusal.version {
            return true;
        }
    }
    return false;
}

/// What:
/// `fn crates_locking(snapshots, key) -> Vec<String>` lists the crate roots whose lockfile holds `name@version`.
/// `Vec<String>` is a growable list of owned text (siblings:
/// `&[String]`,
/// a borrowed view;
/// `[String; N]`,
/// fixed size).
/// Why:
/// Only those roots need the refused version moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cratesLocking(snapshots: LockSnapshot[], key: string): string[]
/// ```
fn crates_locking(
    snapshots: &[LockSnapshot],
    key: &str,
) -> Vec<String> {
    // `let mut`: a binding that may change.
    // Typed binding: `dirs: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut dirs: Vec<String> = Vec::new();
    for snapshot in snapshots {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        // Borrow `&snapshot.source`: lends the value without copying or giving it away.
        if let Ok(versions) = registry_versions(std::slice::from_ref(&snapshot.source))
            // Closure `|locked|`: an anonymous function, like a TS arrow function.
            // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
            // Why:      The collection stays usable after the walk and no item is copied until it must be.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // for (const item of list) { ... }  // or list.map(...)
            // ```
            && locks_key(&versions, key)
        {
            // `.clone()`: an independent owned copy.
            dirs.push(snapshot.crate_dir.clone());
        }
    }
    return dirs;
}

/// What:
/// `fn move_crate(request, crate_dir, refused, candidates) -> Result<String, String>` tries the candidates in order in one crate root with `cargo update --package name@version --precise candidate` and returns the first that cargo accepts,
/// or the empty text.
/// Why:
/// Cargo refuses a candidate the dependency graph cannot hold;
/// the next one is tried,
/// at most ten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function moveCrate(args): Promise<string>
/// ```
fn move_crate(
    request: &SetAsideRequest,
    crate_dir: &str,
    refused: &TrustRefusal,
    candidates: &[String],
) -> Result<String, String> {
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    for candidate in candidates.iter().take(PRECISE_ATTEMPTS) {
        // `let mut`: a binding that may change.
        // Typed binding: `args: Vec<String>`.
        // Macro `vec!`: builds or prints text (or a list) from a template.
        let mut args: Vec<String> = vec![
            // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
            // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // String(value)  // JS strings are always owned values
            // ```
            "update".to_string(),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            "--package".to_string(),
            // Macro `format!`: builds or prints text (or a list) from a template.
            format!("{}@{}", refused.name, refused.version),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            "--precise".to_string(),
            // `.clone()`: an independent owned copy.
            candidate.clone(),
        ];
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        args.extend(request.forward.iter().cloned());
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&request.root.join`: lends the value without copying or giving it away.
        let (ran, _claim): (Result<crate::cargo_run::CargoOutcome, String>, Claim) =
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Borrow `&request.root.join`: lends the value without copying or giving it away.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            run_recording(request.root, crate_dir, request.produced, RunnerWork { runner: request.runner, directory: request.root.join(crate_dir), args: &args }).map_err(text_of)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        let outcome = ran?;
        if outcome.ok {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.clone()`: an independent owned copy.
            return Ok(candidate.clone());
        }
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!(
            "cargo-shadow: {crate_dir}: cargo refused {} {candidate} ({})",
            refused.name,
            outcome.output.trim()
        );
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Constructor `String::new()`: an empty owned collection or a boxed value.
    return Ok(String::new());
}

/// What:
/// `pub fn set_aside_until_clean(request: &SetAsideRequest) -> Result<Vec<String>, String>` repeats rounds of finding refused
/// new versions and moving them down,
/// and returns one warning per move.
/// Why:
/// The rounds stop when nothing is refused,
/// when nothing could be moved,
/// or after the round limit.
/// What stays refused is
/// reported by the caller's final check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function setAsideUntilClean(args): Promise<string[]>
/// ```
pub fn set_aside_until_clean(request: &SetAsideRequest) -> Result<Vec<String>, String> {
    // `let mut`: a binding that may change.
    // Typed binding: `rejected: BTreeSet<String>`.
    // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
    let mut rejected: BTreeSet<String> = BTreeSet::new();
    // `let mut`: a binding that may change.
    // Typed binding: `warnings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut warnings: Vec<String> = Vec::new();
    for _round in 0..SET_ASIDE_ROUNDS {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
        // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { ... } catch (error) { throw new MyError(String(error)); }
        // ```
        // Typed binding: `current: Vec<LockSnapshot>`.
        let current: Vec<LockSnapshot> =
            read_locks(request.root, request.crate_dirs).map_err(text_of)?;
        // Closure `|snapshot|`: an anonymous function, like a TS arrow function.
        // `.clone()`: an independent owned copy.
        // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
        // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Array.from(iterable)  // or [...iterable]
        // ```
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // Adapter `.map(`: transforms or selects items with a closure.
        // Typed binding: `sources: Vec<String>`.
        let sources: Vec<String> = current.iter().map(LockSnapshot::source_text).collect();
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&sources`: lends the value without copying or giving it away.
        // Typed binding: `added: Vec<LockedVersion>`.
        let added: Vec<LockedVersion> = added_versions(request.before, &sources)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&added`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // `let mut`: a binding that may change.
        // Typed binding: `pending: Vec<TrustRefusal>`.
        let mut pending: Vec<TrustRefusal> = age_refusals(&added, request).map_err(message_of)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&added`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        for refusal in trust_refusals(&added, request.settings, request.registry, request.now)
            .map_err(message_of)?
        {
            // Closure `|known|`: an anonymous function, like a TS arrow function.
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            if !is_pending(&pending, &refusal) {
                pending.push(refusal);
            }
        }
        // Closure `|refusal|`: an anonymous function, like a TS arrow function.
        // Borrow `&format`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let held: Vec<TrustRefusal> = std::mem::take(&mut pending);
        for refusal in held {
            if !rejected.contains(&format!("{}@{}", refusal.name, refusal.version)) {
                pending.push(refusal);
            }
        }
        if pending.is_empty() {
            break;
        }
        // Borrow `&pending`: lends the value without copying or giving it away.
        for refusal in &pending {
            // Macro `format!`: builds or prints text (or a list) from a template.
            rejected.insert(format!("{}@{}", refusal.name, refusal.version));
        }
        // `let mut`: a binding that may change.
        // Typed binding: `moved_any: bool`.
        let mut moved_any: bool = false;
        // Borrow `&pending`: lends the value without copying or giving it away.
        for refusal in &pending {
            // Typed binding: `key: String`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            let key: String = format!("{}@{}", refusal.name, refusal.version);
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&rejected`: lends the value without copying or giving it away.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            // Typed binding: `candidates: Vec<String>`.
            let candidates: Vec<String> =
                set_aside_candidates(refusal, request, &rejected).map_err(message_of)?;
            if candidates.is_empty() {
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!("cargo-shadow: no older release can replace {key}; its finding stands");
                continue;
            }
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            // Typed binding: `snapshots: Vec<LockSnapshot>`.
            let snapshots: Vec<LockSnapshot> =
                read_locks(request.root, request.crate_dirs).map_err(text_of)?;
            // Borrow `&snapshots`: lends the value without copying or giving it away.
            for crate_dir in crates_locking(&snapshots, &key) {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Borrow `&crate_dir`: lends the value without copying or giving it away.
                // Typed binding: `instead: String`.
                let instead: String = move_crate(request, &crate_dir, refusal, &candidates)?;
                if instead.is_empty() {
                    // Macro `eprintln!`: builds or prints text (or a list) from a template.
                    eprintln!(
                        "cargo-shadow: {crate_dir}: no candidate for {key} resolved; its finding stands"
                    );
                    continue;
                }
                moved_any = true;
                // Macro `format!`: builds or prints text (or a list) from a template.
                warnings.push(format!("Set aside {key} (release-age or trust policy) and locked {instead} in {crate_dir}."));
            }
        }
        if !moved_any {
            break;
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(warnings);
}
