//! What:
//! Reading a scope's plan and lock together,
//! once,
//! so the revisions recorded for the later publish describe
//!           exactly the bytes that were parsed;
//! plus the stricter read `install` and `list` use,
//! which refuses when the
//!           plan and lock disagree.
//! Why:
//! Parsing a second read of a file would let a concurrent write slip between the revision and the contents.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // load.ts
//! export function load(scope: Scope): Loaded
//! export function loadAgreeing(scope: Scope): { plan: Plan; lock: Lock }  // throws when they disagree
//! ```

/// Imports what this module uses from `super::agree`.
use super::agree::{compare, refusal};
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::lock`.
use super::lock::{Lock, ParsedLock, parse_lock};
/// Imports what this module uses from `super::plan`.
use super::plan::{Plan, parse_plan};
/// Imports what this module uses from `super::publish`.
use super::publish::{Revisions, read_optional, revision_of};
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;

/// What:
/// The parsed plan,
/// the parsed lock when the file exists,
/// the lock's exact text,
/// and the revisions of both
///           files as read.
/// Why:
/// One struct carries everything a mutating command needs to later call `publish_lock`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Loaded = { plan: Plan; lock?: ParsedLock; lockText?: string; revisions: Revisions };
/// ```
#[derive(Clone, Debug)]
pub struct Loaded {
    /// The decoded plan.
    pub plan: Plan,
    /// The decoded lock,
    /// or `None` when the file does not exist.
    pub lock: Option<ParsedLock>,
    /// The lock file's text as read,
    /// for "already current" comparisons.
    pub lock_text: Option<String>,
    /// The revisions of the plan and lock bytes that were parsed.
    pub revisions: Revisions,
}

/// Reads and decodes the plan (which must exist) and the lock (which may not).
pub fn load(scope: &Scope) -> Result<Loaded, ToolchainError> {
    let plan_bytes: Option<Vec<u8>> = read_optional(&scope.plan_file)?;
    let Some(plan_content) = &plan_bytes else {
        return Err(ToolchainError::config(format!(
            "the toolchain plan {} does not exist",
            scope.plan_file.display()
        )));
    };
    let plan_text: String = match String::from_utf8(plan_content.clone()) {
        Ok(text) => text,
        Err(error) => {
            return Err(ToolchainError::config(format!("{} is not valid UTF-8: {error}", scope.plan_file.display())));
        }
    };
    let plan: Plan = parse_plan(&plan_text, &scope.plan_file.display().to_string())?;
    let lock_bytes: Option<Vec<u8>> = read_optional(&scope.lock_file)?;
    let mut lock: Option<ParsedLock> = None;
    let mut lock_text: Option<String> = None;
    if let Some(content) = &lock_bytes {
        let text: String = match String::from_utf8(content.clone()) {
            Ok(text) => text,
            Err(error) => {
                return Err(ToolchainError::config(format!("{} is not valid UTF-8: {error}", scope.lock_file.display())));
            }
        };
        lock = Some(parse_lock(&text, &scope.lock_file.display().to_string())?);
        lock_text = Some(text);
    }
    let revisions: Revisions = Revisions { plan: revision_of(&plan_bytes), lock: revision_of(&lock_bytes) };
    return Ok(Loaded { plan, lock, lock_text, revisions });
}

/// What:
/// The plan and the lock for a command that acts on installed tools (`install`,
/// `list`):
/// both must exist,
/// the
///           digest must match,
/// and they must agree on every field.
/// Why:
/// Installing from a lock that does not describe the plan would install the wrong tools,
/// so the command
///           refuses and names `mpm-shadow install --lockfile-only --offline` or `mpm-shadow update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function loadAgreeing(scope: Scope): { plan: Plan; lock: Lock }
/// ```
pub fn load_agreeing(scope: &Scope) -> Result<(Plan, Lock), ToolchainError> {
    let loaded: Loaded = load(scope)?;
    let Some(parsed) = loaded.lock else {
        return Err(ToolchainError::drift(format!(
            "{} does not exist; run `mpm-shadow update` to resolve the plan's tools and write it",
            scope.lock_file.display()
        )));
    };
    let verdict = compare(&loaded.plan, &parsed);
    if !verdict.is_empty() {
        return Err(refusal(&verdict));
    }
    return Ok((loaded.plan, parsed.lock));
}
