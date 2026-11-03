//! What:
//! The full update's resolution of each crate root:
//! `cargo generate-lockfile` under the release-age cutoff,
//! the fall back to resolving without it,
//! and the move of the crates exempt from the release age to their newest release.
//! Why:
//! `update.rs` holds the transaction;
//! these two functions were split out to keep each file under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-resolve.ts
//! export async function resolveAll(request: UpdateRequest, cutoff: string, produced: LockSnapshot[]): Promise<string[]>
//! ```

/// What:
/// `use std::cell::RefCell;` imports `RefCell`,
/// a value that can be changed through a shared reference.
/// Why:
/// The record of the lockfiles this run wrote is lent to `resolve_all`,
/// which adds to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::RefCell;

/// What:
/// `use crate::cargo_run::CargoOutcome;` imports the result of one cargo call.
/// Why:
/// `resolve_crate` looks at whether the cutoff-constrained call succeeded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CargoOutcome } from 'crate/cargo_run';
/// ```
use crate::cargo_run::CargoOutcome;
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports the lockfile snapshot type and the reader of the lockfiles.
/// Why:
/// After a crate root is resolved its lockfile is read to find the crates that are exempt from the release age.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, readLocks } from 'crate/crates';
/// ```
use crate::crates::{LockSnapshot, read_locks};
/// What:
/// `use crate::journal::run_recording;` imports the function that runs a child and records the lockfile it wrote.
/// Why:
/// Each root is attributed with the ownership evidence around its cargo call,
/// whether the call succeeds or fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runRecording } from 'crate/journal';
/// ```
use crate::journal::run_recording;
/// Imports `Claim`,
/// what the ownership evidence says about a file a child wrote.
use crate::file_owner::Claim;
/// What:
/// `use crate::update::{UpdateError, UpdateRequest, excluded_update_specs, other, other_text, run_or_fail};` imports the update's request and error types and the helpers that run cargo and wrap errors.
/// Why:
/// The resolution is part of the update and reports its errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { UpdateError, UpdateRequest, excludedUpdateSpecs, other, otherText, runOrFail } from 'crate/update';
/// ```
use crate::update::{UpdateError, UpdateRequest, excluded_update_specs, other, other_text, run_or_fail};

/// What:
/// `fn resolve_crate(request, crate_dir, cutoff) -> Result<bool, UpdateError>` re-resolves one crate root under the cutoff and returns whether it had to fall back to ignoring the cutoff.
/// Why:
/// With `minimumReleaseAgeStrict` false a crate that cannot resolve under the cutoff resolves without it and is reported;
/// strict fails instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveCrate(args): Promise<boolean>  // throws
/// ```
fn resolve_crate(request: &UpdateRequest, crate_dir: &str, cutoff: &str) -> Result<bool, UpdateError> {
    // `let mut`: a binding that may change.
    // Typed binding: `constrained: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut constrained: Vec<String> = Vec::new();
    if cutoff.is_empty() {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        constrained.push("generate-lockfile".to_string());
    } else {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        constrained.extend(["-Zunstable-options".to_string(), "generate-lockfile".to_string(), "--publish-time".to_string(), cutoff.to_string()]);
    }
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    constrained.extend(request.forward.iter().cloned());
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|message|`: an anonymous function, like a TS arrow function.
    // Borrow `&request.root.join`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `first: CargoOutcome`.
    let first: CargoOutcome = request.runner.run(&request.root.join(crate_dir), &constrained).map_err(other)?;
    if first.ok {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(false);
    }
    // Closure `||`: an anonymous function, like a TS arrow function.
    // Review finding (the global cutoff defeats exemptions): cargo's `--publish-time` has no per-crate form, so a crate the
    // manifests need in a young release, and that `minimumReleaseAgeExclude` exempts, can only resolve without the cutoff.
    // The resolution then continues without it, and the set-aside and the final check (`check_release_age`) keep the cutoff
    // for every crate that is not exempt, so the policy is applied after the selection instead of before it.
    let has_exemptions: bool = !request.settings.minimum_release_age_exclude.is_empty();
    if (request.settings.minimum_release_age_strict && !has_exemptions) || cutoff.is_empty() {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(UpdateError::Cargo { crate_dir: crate_dir.to_string(), command: format!("cargo {}", constrained.join(" ")), output: first.output });
    }
    // Macro `eprintln!`: builds or prints text (or a list) from a template.
    eprintln!("cargo-shadow: {crate_dir}: no resolution satisfies the release-age cutoff {cutoff}; resolving without it (minimumReleaseAgeStrict is false, or minimumReleaseAgeExclude lists crates that may be young), then holding every crate that is not exempt to the cutoff");
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `let mut`: a binding that may change.
    // Typed binding: `fallback: Vec<String>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let mut fallback: Vec<String> = vec!["generate-lockfile".to_string()];
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    fallback.extend(request.forward.iter().cloned());
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&fallback`: lends the value without copying or giving it away.
    run_or_fail(request, crate_dir, &fallback)?;
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(true);
}

/// What:
/// `fn resolve_all(request, cutoff) -> Result<Vec<String>, UpdateError>` resolves every listed crate root and then moves the exempt crates to their newest release.
/// Why:
/// The exempt update reads the lockfile that the same root's resolution just wrote,
/// so the two steps stay together per root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveAll(args): Promise<string[]>
/// ```
pub(crate) fn resolve_all(request: &UpdateRequest, cutoff: &str, produced: &RefCell<Vec<LockSnapshot>>) -> Result<Vec<String>, UpdateError> {
    // `let mut`: a binding that may change.
    // Typed binding: `fell_back: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut fell_back: Vec<String> = Vec::new();
    for crate_dir in request.crate_dirs {
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        eprintln!("cargo-shadow: resolving {crate_dir}{}", if cutoff.is_empty() { String::new() } else { format!(" with releases published before {cutoff}") });
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Recorded whether the resolution succeeded or failed (see `run_recording`).
        let (resolved, _claim): (Result<bool, UpdateError>, Claim) = run_recording(request.root, crate_dir, produced, || { return resolve_crate(request, crate_dir, cutoff); }).map_err(other_text)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Typed binding: `moved_to_fallback: bool`.
        let moved_to_fallback: bool = resolved?;
        if moved_to_fallback {
            // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
            // Why:      `fell_back` lists the crate roots that could not resolve under the cutoff and resolved without it, so the report can name them; the root name is copied because the loop still uses it.
            // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // structuredClone(value)  // or [...list] / {...object}
            // ```
            fell_back.push(crate_dir.clone());
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `snapshots: Vec<LockSnapshot>`.
        let snapshots: Vec<LockSnapshot> = read_locks(request.root, std::slice::from_ref(crate_dir)).map_err(other_text)?;
        // Closure `|snapshot|`: an anonymous function, like a TS arrow function.
        // Borrow `&str`: lends the value without copying or giving it away.
        // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
        // Why:      If no snapshot comes back, the text is empty and no crate gets an exclusion update.
        // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // value ?? fallback
        // ```
        // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
        // Why:      `first()` then `map` takes the text of the one snapshot, if there is one, without copying it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // list.map(f).filter(g)
        // ```
        // Typed binding: `source: &str`.
        let source: &str = snapshots.first().map(LockSnapshot::source_str).unwrap_or("");
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|message|`: an anonymous function, like a TS arrow function.
        // Borrow `&request.settings.minimum_release_age_exclu...`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `specs: Vec<String>`.
        let specs: Vec<String> = excluded_update_specs(source, &request.settings.minimum_release_age_exclude).map_err(other)?;
        if !specs.is_empty() {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `let mut`: a binding that may change.
            // Typed binding: `args: Vec<String>`.
            // Macro `vec!`: builds or prints text (or a list) from a template.
            let mut args: Vec<String> = vec!["update".to_string()];
            args.extend(specs);
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            args.extend(request.forward.iter().cloned());
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Borrow `&args`: lends the value without copying or giving it away.
            let (updated, _update_claim): (Result<CargoOutcome, UpdateError>, Claim) = run_recording(request.root, crate_dir, produced, || { return run_or_fail(request, crate_dir, &args); }).map_err(other_text)?;
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            updated?;
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(fell_back);
}
