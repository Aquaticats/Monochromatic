//! What:
//! The two checks a frozen install makes of the registry versions in the lockfiles:
//! every entry (`trustLockfile = false`,
//! pnpm's
//! default,
//! below) or only the versions new since a base revision (`trustLockfile = true`,
//! `verify_added_since_base`).
//! Why:
//! pnpm 12.16.1 re-validates every entry of the lockfile on install,
//! frozen or not,
//! against the policies its resolvers were built
//! with,
//! unless `trustLockfile` is true (`pnpm11/installing/deps-installer/src/install/mutateModules.ts:314-343`,
//! `startLockfileVerification`:
//! "Re-validate every entry in the lockfile against the policies the resolver chain was built
//! with";
//! skipped when `opts.trustLockfile`).
//! A passing result is cached by a hash of the lockfile
//! (`pnpm11/installing/deps-installer/src/install/verifyLockfileResolutionsCache.ts`).
//! This is that check for `Cargo.lock`:
//! the
//! default,
//! no separate command.
//! The release-age rule needs the sparse index's publish time per crate;
//! the trust rule needs the
//! crate's history,
//! which the database dump fills in bulk.
//! A passing lockfile is remembered under
//! `.cache/cargo-shadow/verified/`,
//! keyed by a SHA-256 of the lockfile text and the policy settings,
//! so an unchanged lockfile costs
//! nothing the next time;
//! a failing one is never remembered.
//! Which of the two checks runs is the catalog setting `trustLockfile` (default false,
//! as in pnpm;
//! the repository's `rust-catalog.toml` sets it true by the owner's decision of 2026-10-29),
//! read by the caller (`install.rs`),
//! not here.
//! A remembered pass is keyed by a scope string as well
//! (the install mode,
//! and for `trustLockfile = true` the resolved base identity the caller supplies:
//! a commit,
//! or `unresolved:<spelling>`),
//! so a pass under one mode,
//! policy or base is never taken for a pass under another.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // verify.ts
//! export async function verifyLockedEntries({ root, snapshots, settings, registry, now }): Promise<string[]>
//! ```

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `verify_locked_entries`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
/// What:
/// `use std::path::{Path, PathBuf};` imports `Path`,
/// `PathBuf` from the standard library so this file can name them unqualified.
/// Why:
/// `Path` is used by `cache_file`,
/// `verify_locked_entries`;
/// `PathBuf` is used by `cache_file`,
/// `verify_locked_entries`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{Path, PathBuf};

/// What:
/// `use monochromatic_dependency_catalog::Settings;` imports `Settings` from the external crate `monochromatic_dependency_catalog` so this file can name it unqualified.
/// Why:
/// `Settings` is used by `cache_key`,
/// `cache_file`,
/// `verify_locked_entries`,
/// `verify_added_since_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::Settings;
/// What:
/// `use sha2::{Digest, Sha256};` imports `Digest`,
/// `Sha256` from the external crate `sha2` so this file can name them unqualified.
/// Why:
/// `Digest` must be in scope for its methods:
/// `update` and `finalize` on the hasher;
/// `Sha256` is used by `cache_key`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Digest, Sha256 } from 'sha2';
/// ```
use sha2::{Digest, Sha256};

/// What:
/// `use crate::checks::{CheckOutcome, check_policy};` imports `CheckOutcome`,
/// `check_policy` from another module of this crate so this file can name them unqualified.
/// Why:
/// `check_policy` runs the release-age and trust checks together and keeps a refusal when a later lookup fails;
/// `CheckOutcome` carries its findings and warnings out of the two verify functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CheckOutcome, checkPolicy } from 'crate/checks';
/// ```
use crate::checks::{CheckOutcome, check_policy};
/// What:
/// `use crate::crates::LockSnapshot;` imports `LockSnapshot` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LockSnapshot` is used by `cache_key`,
/// `cache_file`,
/// `verify_locked_entries`,
/// `verify_added_since_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot } from 'crate/crates';
/// ```
use crate::crates::LockSnapshot;
/// What:
/// `use crate::lockfile::{LockedVersion, added_versions, registry_versions};` imports `LockedVersion`,
/// `added_versions`,
/// `registry_versions` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LockedVersion` is used by `verify_locked_entries`,
/// `verify_added_since_base`;
/// `added_versions` is used by `verify_added_since_base`;
/// `registry_versions` is used by `verify_locked_entries`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion, added_versions, registry_versions } from 'crate/lockfile';
/// ```
use crate::lockfile::{LockedVersion, added_versions, registry_versions};
/// What:
/// `use crate::registry::Registry;` imports `Registry` from another module of this crate so this file can name it unqualified.
/// Why:
/// `Registry` is used by `verify_locked_entries`,
/// `verify_added_since_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry } from 'crate/registry';
/// ```
use crate::registry::Registry;
/// What:
/// `use crate::adapt::message_of;` imports `message_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `message_of` is used by `verify_locked_entries`,
/// `verify_added_since_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { message_of } from 'crate/adapt';
/// ```
use crate::adapt::message_of;

/// Where passing digests are remembered,
/// relative to the repository root.
pub const VERIFIED_DIR: &str = ".cache/cargo-shadow/verified";

/// The lowercase hex of a SHA-256 digest,
/// written per byte (the digest type has no `LowerHex` in sha2 0.11).
fn hex(digest: &[u8]) -> String {
    let mut text: String = String::new();
    for byte in digest {
        text.push_str(&format!("{byte:02x}"));
    }
    return text;
}

/// The scope of the remembered passes of the frozen check of every locked entry.
pub const ALL_ENTRIES_SCOPE: &str = "frozen-all-entries";

/// What:
/// `pub fn cache_key(snapshot, settings, scope) -> String` is the cache key of one lockfile under one policy and one check:
/// a digest of the text,
/// of every setting and of the scope.
/// Why:
/// A lockfile that passed one check (for example every entry,
/// or only what is new since one base) says nothing about another
/// check,
/// another base or another policy,
/// so each of those gets its own key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const cacheKey = (snapshot: LockSnapshot, settings: Settings, scope: string): string => sha256(snapshot.source + JSON.stringify(settings) + scope);
/// ```
pub fn cache_key(snapshot: &LockSnapshot, settings: &Settings, scope: &str) -> String {
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(snapshot.source.as_bytes());
    hasher.update(format!("{settings:?}").as_bytes());
    // A separator keeps the lockfile text and the scope from running into each other.
    hasher.update([0u8]);
    hasher.update(scope.as_bytes());
    return hex(hasher.finalize().as_slice());
}

/// What:
/// `fn cache_file(root, snapshot, settings, scope) -> PathBuf` is the marker file for one lockfile under one policy and check.
/// Why:
/// The lookup before a check and the write after a pass must agree on the name,
/// which holds the crate directory and the digest of the lockfile text,
/// settings and scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cacheFile(root: string, snapshot: LockSnapshot, settings: Settings, scope: string): string
/// ```
fn cache_file(root: &Path, snapshot: &LockSnapshot, settings: &Settings, scope: &str) -> PathBuf {
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    return root.join(VERIFIED_DIR).join(format!("{}.{}.ok", snapshot.crate_dir.replace('/', "__"), cache_key(snapshot, settings, scope)));
}

/// What:
/// `pub fn is_verified(root, snapshot, settings, scope) -> bool` says whether this lockfile text passed this check under this
/// policy before.
/// Why:
/// An unchanged,
/// already verified lockfile costs one file lookup instead of registry requests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const isVerified = (root, snapshot, settings, scope): boolean => existsSync(cacheFile(root, snapshot, settings, scope));
/// ```
pub fn is_verified(root: &Path, snapshot: &LockSnapshot, settings: &Settings, scope: &str) -> bool {
    return cache_file(root, snapshot, settings, scope).exists();
}

/// What:
/// `pub fn mark_verified(root, snapshot, settings, scope)` remembers that this lockfile text passed this check.
/// Why:
/// Only a pass is ever written;
/// a failing lockfile is checked again each time.
/// A write that fails only costs the next check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function markVerified(root, snapshot, settings, scope): void  // best effort
/// ```
pub fn mark_verified(root: &Path, snapshot: &LockSnapshot, settings: &Settings, scope: &str) {
    // Typed binding: `path: PathBuf`.
    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let path: PathBuf = cache_file(root, snapshot, settings, scope);
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    if let Some(parent) = path.parent() {
        // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
        // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // void expr;   // or try { expr } catch {}
        // ```
        let _ = fs::create_dir_all(parent);
    }
    // `let _ =`: discards the result on purpose.
    let _ = fs::write(path, "");
}

/// What:
/// `pub fn verify_locked_entries(...) -> Result<Vec<String>, String>` returns the findings against every registry version the
/// given lockfiles hold,
/// skipping a lockfile whose digest passed before.
/// Why:
/// The findings are the release-age problems (strict) and the trust problems;
/// the lockfiles that pass are remembered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function verifyLockedEntries(args): Promise<string[]>
/// ```
pub fn verify_locked_entries(root: &Path, snapshots: &[LockSnapshot], settings: &Settings, registry: &dyn Registry, now: i64) -> Result<Vec<String>, String> {
    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    return Ok(verify_locked_entries_outcome(root, snapshots, settings, registry, now)?.problems);
}

/// What:
/// `pub fn verify_locked_entries_outcome(...) -> Result<CheckOutcome, String>` is `verify_locked_entries` with the release-age warnings.
/// Why:
/// Review finding (2c 8):
/// a young release that the non-strict release age allows must still be reported on the frozen path,
/// and a pass that carries a warning is not remembered,
/// so the warning is shown again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function verifyLockedEntriesOutcome(args): Promise<CheckOutcome>
/// ```
pub fn verify_locked_entries_outcome(
    root: &Path,
    snapshots: &[LockSnapshot],
    settings: &Settings,
    registry: &dyn Registry,
    now: i64,
// What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
// Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
) -> Result<CheckOutcome, String> {
    let mut pending: Vec<&LockSnapshot> = Vec::new();
    for snapshot in snapshots {
        if !is_verified(root, snapshot, settings, ALL_ENTRIES_SCOPE) {
            pending.push(snapshot);
        }
    }
    if pending.is_empty() {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(CheckOutcome::default());
    }
    let sources: Vec<String> = pending.iter().copied().map(LockSnapshot::source_text).collect();
    let versions: Vec<LockedVersion> = registry_versions(&sources)?;
    eprintln!("cargo-shadow: verifying {} locked versions from {} lockfiles", versions.len(), pending.len());
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
    // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (error) { throw new MyError(String(error)); }
    // ```
    // Typed binding: `outcome: CheckOutcome`.
    let outcome: CheckOutcome = check_policy(&versions, settings, registry, now).map_err(message_of)?;
    if outcome.problems.is_empty() && outcome.warnings.is_empty() {
        for snapshot in pending {
            mark_verified(root, snapshot, settings, ALL_ENTRIES_SCOPE);
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(outcome);
}

/// What:
/// `pub fn verify_added_since_base(...) -> Result<Vec<String>, String>` returns the trust findings for the registry versions
/// that the given lockfiles hold and that the lockfiles at the base revision did not.
/// Why:
/// This is the frozen install under `trustLockfile = true`,
/// the owner's decision of 2026-10-29
/// (`doc/decision/rust-dependency-catalog.md`):
/// committed lockfiles are part of the trusted base because the owner never
/// accepts pull requests that change a lockfile,
/// so only what changed since the base is checked,
/// and only for trust;
/// release
/// age is a resolution-time rule,
/// so a version that was locked has already passed it.
/// A lockfile that did not exist at the base
/// reads as the empty text,
/// so every version in it counts as new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function verifyRustLockfiles({ root, now, registry, readBaseLock }): Promise<string[]>
/// ```
pub fn verify_added_since_base(
    snapshots: &[LockSnapshot],
    read_base_lock: &dyn Fn(&str) -> String,
    settings: &Settings,
    registry: &dyn Registry,
    now: i64,
) -> Result<Vec<String>, String> {
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    return Ok(verify_added_since_base_outcome(snapshots, read_base_lock, settings, registry, now)?.problems);
}

/// What:
/// `pub fn verify_added_since_base_outcome(...) -> Result<CheckOutcome, String>` is `verify_added_since_base` with the release-age warnings.
/// Why:
/// The same as `verify_locked_entries_outcome`,
/// for the versions new since the base.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function verifyAddedSinceBaseOutcome(args): Promise<CheckOutcome>
/// ```
pub fn verify_added_since_base_outcome(
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    snapshots: &[LockSnapshot],
    // Borrow `&str`: lends the value without copying or giving it away.
    // What:     `&dyn Trait` is a borrowed value of any type that implements the trait (a TS interface type); the call goes through a lookup table at run time.
    // Why:      The caller may pass the real implementation or a test fake.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function f(registry: Registry) { ... }  // an interface type
    // ```
    read_base_lock: &dyn Fn(&str) -> String,
    settings: &Settings,
    // `dyn` trait object `dyn Registry`: any type that implements the trait.
    registry: &dyn Registry,
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    now: i64,
) -> Result<CheckOutcome, String> {
    let mut base_sources: Vec<String> = Vec::new();
    for snapshot in snapshots {
        base_sources.push(read_base_lock(&snapshot.crate_dir));
    }
    let current: Vec<String> = snapshots.iter().map(LockSnapshot::source_text).collect();
    let versions: Vec<LockedVersion> = added_versions(&base_sources, &current)?;
    eprintln!("cargo-shadow: verifying {} versions newly locked since the base from {} lockfiles", versions.len(), snapshots.len());
    if versions.is_empty() {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(CheckOutcome::default());
    }
    // Borrow `&versions`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    return check_policy(&versions, settings, registry, now).map_err(message_of);
}
