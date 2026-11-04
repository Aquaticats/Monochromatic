//! What:
//! The pieces of the install that decide which checks a lockfile still needs:
//! the name of the check a remembered pass belongs to,
//! whether every lockfile already passed it,
//! the maker of the base-revision reader,
//! and the pass that sets refused releases aside.
//! Why:
//! `install.rs` holds the transaction;
//! these helpers were split out to keep each file under the line limit,
//! and `install.rs` re-exports them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-verify.ts
//! export function verificationScope(settings: Settings, frozen: boolean, trustBase: string): string
//! export function verificationCached(args): boolean
//! ```

/// What:
/// `use std::cell::RefCell;` imports `RefCell`,
/// a value that can be changed through a shared reference.
/// Why:
/// The record of what this run's cargo calls wrote is lent to the set-aside pass,
/// which adds to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::path::Path;` imports `Path`,
/// the borrowed view of a file path (sibling:
/// `PathBuf`,
/// the owned one).
/// Why:
/// The repository root is lent to `verification_cached`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::Path;

/// What:
/// `use monochromatic_dependency_catalog::Settings;` imports the catalog settings type.
/// Why:
/// The scope of a remembered pass depends on `trustLockfile`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings } from 'monochromatic-dependency-catalog';
/// ```
use monochromatic_dependency_catalog::Settings;

/// What:
/// `use crate::checks::check_policy;` imports the two policy checks together over a list of locked versions.
/// Why:
/// `policy_findings` judges the versions a killed run's lockfile added.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { checkReleaseAge, checkTrust } from 'crate/checks';
/// ```
use crate::checks::check_policy;
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports the lockfile snapshot type and the reader of the lockfiles.
/// Why:
/// `verification_cached` reads the lockfiles to look up their remembered passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, readLocks } from 'crate/crates';
/// ```
use crate::crates::{LockSnapshot, read_locks};
/// What:
/// `use crate::install::{InstallError, InstallRequest, io_message, sources_of};` imports the install request and error types and the helpers that wrap a message as an error and list the lockfile texts.
/// Why:
/// The helpers here take the request and report the install's own errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { InstallError, InstallRequest, ioMessage, sourcesOf } from 'crate/install';
/// ```
use crate::install::{InstallError, InstallRequest, sources_of};
/// Imports the helper that wraps a message text as an install error.
use crate::install_error::io_message;
/// What:
/// `use crate::lockfile::{LockedVersion, added_versions};` imports the version type and the difference between two lockfile sets.
/// Why:
/// `policy_findings` checks only the registry versions the live lockfile holds and the snapshot did not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion, addedVersions } from 'crate/lockfile';
/// ```
use crate::lockfile::{LockedVersion, added_versions};
/// What:
/// `use crate::registry::Registry;` imports the trait the policy checks ask crates.io through.
/// Why:
/// `policy_findings` takes the registry as a parameter,
/// so tests can give it a fake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry } from 'crate/registry';
/// ```
use crate::registry::Registry;
/// What:
/// `use crate::set_aside::{SetAsideRequest, set_aside_until_clean};` imports the set-aside round and its request.
/// Why:
/// A refused release is moved to the newest older one that passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SetAsideRequest, setAsideUntilClean } from 'crate/set_aside';
/// ```
use crate::set_aside::{SetAsideRequest, set_aside_until_clean};
/// What:
/// `use crate::verify::{ALL_ENTRIES_SCOPE, is_verified};` imports the scope name and the lookup of a remembered pass.
/// Why:
/// `verification_scope` names the check and `verification_cached` asks whether it passed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ALL_ENTRIES_SCOPE, isVerified } from 'crate/verify';
/// ```
use crate::verify::{ALL_ENTRIES_SCOPE, is_verified};

/// A reader of one crate root's lockfile text at the base revision.
pub type LockBaseReader = Box<dyn Fn(&str) -> String>;

/// Makes a `LockBaseReader`,
/// or says why the base cannot be read.
pub type LockBaseFactory<'a> = &'a dyn Fn() -> Result<LockBaseReader, String>;

/// What:
/// `pub fn verification_scope(settings, frozen, trust_base) -> String` names the check a remembered pass belongs to.
/// Why:
/// A pass of the frozen check says nothing about the local check (which also sets refused releases aside),
/// and a pass
/// against one base revision says nothing about another,
/// so the remembered passes are keyed by this text as well as by the
/// lockfile and the settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const verificationScope = (settings: Settings, frozen: boolean, trustBase: string): string
/// ```
pub fn verification_scope(settings: &Settings, frozen: bool, trust_base: &str) -> String {
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    // Typed binding: `mode: &str`.
    let mode: &str = if frozen { "frozen" } else { "local" };
    if settings.trust_lockfile {
        // Macro `format!`: builds or prints text (or a list) from a template.
        return format!("{mode}-since-base:{trust_base}");
    }
    if frozen {
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        return ALL_ENTRIES_SCOPE.to_string();
    }
    // Macro `format!`: builds or prints text (or a list) from a template.
    return format!("{mode}-all-entries");
}

/// What:
/// `pub fn verification_cached(root, crate_dirs, settings, frozen, trust_base) -> bool` says whether every lockfile already
/// passed the policy check of this mode,
/// under these settings and this base.
/// Why:
/// The freshness stamp answers "do the lockfiles satisfy the manifests";
/// this answers "were they checked against the policy".
/// A call may take the fast path only when both are true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function verificationCached({ root, crateDirs, settings, frozen, trustBase }): boolean
/// ```
pub fn verification_cached(root: &Path, crate_dirs: &[String], settings: &Settings, frozen: bool, trust_base: &str) -> bool {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `snapshots: Vec<LockSnapshot>`.
    let snapshots: Vec<LockSnapshot> = match read_locks(root, crate_dirs) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => return false,
    };
    // Typed binding: `scope: String`.
    let scope: String = verification_scope(settings, frozen, trust_base);
    // Borrow `&snapshots`: lends the value without copying or giving it away.
    for snapshot in &snapshots {
        // Borrow `&scope`: lends the value without copying or giving it away.
        if !is_verified(root, snapshot, settings, &scope) {
            return false;
        }
    }
    return true;
}

/// What:
/// `fn base_reader(request) -> Result<LockBaseReader, InstallError>` makes the base-revision lockfile reader.
/// Why:
/// An install under `trustLockfile = true` compares with the base,
/// and without a reader nothing can be trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function baseReader(request: InstallRequest): (crateDir: string) => string  // throws
/// ```
pub(crate) fn base_reader(request: &InstallRequest) -> Result<LockBaseReader, InstallError> {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `factory: LockBaseFactory`.
    let factory: LockBaseFactory = match request.lock_base {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        Some(found) => found,
        // `None`: the present or the absent value of an `Option`.
        None => {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(InstallError::Io {
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                message: "trustLockfile is true in rust-catalog.toml, but no base-revision lockfile reader was given, so nothing can be trusted".to_string(),
            });
        }
    };
    // `match`: picks the arm whose pattern fits the value.
    return match factory() {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(reader) => Ok(reader),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(message) => Err(io_message(message)),
    };
}

/// What:
/// `fn set_aside_pass(request, baseline, produced) -> Result<Vec<String>, InstallError>` sets every release the policy
/// refuses (among the versions `baseline` did not hold) aside for the newest older release that passes,
/// prints one line per move and returns the lines.
/// Why:
/// pnpm resolves to the newest release older than the cutoff instead of refusing,
/// and the owner decided (2026-11-03) that a
/// local install does the same for a committed lockfile:
/// the refused release never runs,
/// the older copy does.
/// The moves go through the transaction (`produced` records each lockfile this run wrote),
/// so another writer's lockfile is
/// never replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function setAsidePass(request, baseline, produced): Promise<string[]>
/// ```
pub(crate) fn set_aside_pass(request: &InstallRequest, baseline: &[String], produced: &RefCell<Vec<LockSnapshot>>) -> Result<Vec<String>, InstallError> {
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `forward: Vec<String>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let forward: Vec<String> = if request.offline { vec!["--offline".to_string()] } else { Vec::new() };
    // Borrow `&SetAsideRequest`: lends the value without copying or giving it away.
    // Typed binding: `lines: Vec<String>`.
    let lines: Vec<String> = set_aside_until_clean(&SetAsideRequest {
        root: request.root,
        crate_dirs: request.crate_dirs,
        before: baseline,
        registry: request.registry,
        settings: request.settings,
        now: request.now,
        runner: request.runner,
        forward: &forward,
        produced,
    })
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
    // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (error) { throw new MyError(String(error)); }
    // ```
    .map_err(io_message)?;
    // Borrow `&lines`: lends the value without copying or giving it away.
    for line in &lines {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: {line}");
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(lines);
}

/// The policy findings for the registry versions `after` holds that `before` did not;
/// used to judge a killed run's leftovers.
pub fn policy_findings(settings: &Settings, registry: &dyn Registry, now: i64, before: &[LockSnapshot], after: &[LockSnapshot]) -> Vec<String> {
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&sources_of`: lends the value without copying or giving it away.
    // Typed binding: `added: Vec<LockedVersion>`.
    let added: Vec<LockedVersion> = match added_versions(&sources_of(before), &sources_of(after)) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `vec!`: builds or prints text (or a list) from a template.
        Err(message) => return vec![format!("a Cargo.lock could not be read for its registry versions: {message}")],
    };
    if added.is_empty() {
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        return Vec::new();
    }
    // `let mut`: a binding that may change.
    // Typed binding: `findings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut findings: Vec<String> = Vec::new();
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&added`: lends the value without copying or giving it away.
    match check_policy(&added, settings, registry, now) {
        Ok(outcome) => findings.extend(outcome.problems),
        Err(error) => findings.push(error.message),
    }
    return findings;
}
