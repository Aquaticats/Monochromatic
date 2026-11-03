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

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::collections::BTreeSet;
// Imports `std::path::Path`.
use std::path::Path;

// Imports `monochromatic_dependency_catalog::Settings`.
use monochromatic_dependency_catalog::Settings;

// Imports `crate::cargo_run::CargoRunner`.
use crate::cargo_run::CargoRunner;
// Imports `crate::checks::{TrustRefusal, group_by_name...`.
use crate::checks::{TrustRefusal, group_by_name, trust_refusals};
// Imports `crate::crates::{LockSnapshot, read_locks}`.
use crate::crates::{LockSnapshot, read_locks};
// Imports `crate::http::RegistryError`.
use crate::http::RegistryError;
// Imports `crate::lockfile::{LockedVersion, added_vers...`.
use crate::lockfile::{LockedVersion, added_versions, registry_versions};
// Imports `crate::policy::{VersionRecord, release_age_...`.
use crate::policy::{VersionRecord, release_age_problems, trust_problems};
// Imports `crate::registry::Registry`.
use crate::registry::Registry;
// Imports `crate::time_text::parse_rfc3339_millis`.
use crate::time_text::parse_rfc3339_millis;
// Imports `crate::version_order::{caret_compatible, co...`.
use crate::version_order::{caret_compatible, compare_texts, is_prerelease_text};

/// How many rounds of moving refused crates are tried.
pub const SET_ASIDE_ROUNDS: usize = 5;

/// How many candidates are tried per refused version and crate.
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
    pub cutoff_ms: i64,
    /// The cargo to run.
    pub runner: &'a dyn CargoRunner,
    /// Options forwarded to every cargo call (`--offline`,
    /// `--config`,
    /// ...).
    pub forward: &'a [String],
}

/// What:
/// `fn age_refusals(added, request) -> Result<Vec<TrustRefusal>, RegistryError>` lists the new versions that the release-age rule refuses,
/// as refusals.
/// `Result<T, E>` is Rust's success-or-failure value (siblings:
/// `Option<T>`,
/// a value or nothing);
/// `?` on one returns early with the failure.
/// Why:
/// Under a strict `minimumReleaseAge` a too-young version must be set aside like a version the trust rule refuses;
/// without a cutoff in `cargo update` the age rule cannot steer the resolution,
/// so it has to be a refusal here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ageRefusals(added: LockedVersion[], request: SetAsideRequest): TrustRefusal[]  // throws RegistryError
/// ```
fn age_refusals(added: &[LockedVersion], request: &SetAsideRequest) -> Result<Vec<TrustRefusal>, RegistryError> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut refusals: Vec<TrustRefusal> = Vec::new();
    if request.settings.minimum_release_age == 0 || !request.settings.minimum_release_age_strict {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        return Ok(refusals);
    }
    for (name, versions) in group_by_name(added) {
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        let times = request.registry.pubtimes(&name)?;
        for version in versions {
            // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
            // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // (a, b) => body
            // ```
            // Borrow `&str`: lends the value without copying or giving it away.
            // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
            // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
            // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // value ?? fallback
            // ```
            // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
            // Why:      They express one step of a pipeline without an index loop.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // list.map(f).filter(g)
            // ```
            // Typed binding: `pubtime: &str`.
            // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
            // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // TS has one type for this: string
            // ```
            let pubtime: &str = times.get(&version).map(|text| { return text.as_str(); }).unwrap_or("");
            // Borrow `&name`: lends the value without copying or giving it away.
            // Typed binding: `problems: Vec<String>`.
            let problems: Vec<String> = release_age_problems(&name, &version, pubtime, request.settings, request.now);
            if !problems.is_empty() {
                // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
                // Why:      The copy can be stored or moved elsewhere while the original stays in place.
                // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // structuredClone(value)  // or [...list] / {...object}
                // ```
                refusals.push(TrustRefusal { name: name.clone(), version, problems });
            }
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(refusals);
}

/// What:
/// `fn is_candidate(record, refused, prerelease_allowed, cutoff_ms, yanked, rejected) -> bool` says whether one published release may replace the refused version.
/// `&` marks a borrowed read-only argument;
/// `BTreeSet<String>` is an ordered set of owned text (sibling:
/// `HashSet`,
/// unordered).
/// Why:
/// A replacement must be older,
/// in the same caret class,
/// not yanked,
/// not already rejected,
/// published before the cutoff,
/// and a prerelease only when the refused version is one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCandidate(args): boolean
/// ```
fn is_candidate(record: &VersionRecord, refused: &TrustRefusal, prerelease_allowed: bool, cutoff_ms: i64, yanked: &BTreeSet<String>, rejected: &BTreeSet<String>) -> bool {
    // Closure `||`: an anonymous function, like a TS arrow function.
    // Borrow `&record.num`: lends the value without copying or giving it away.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    if yanked.contains(&record.num) || rejected.contains(&format!("{}@{}", refused.name, record.num)) {
        return false;
    }
    // Borrow `&record.created_at`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    if cutoff_ms != 0 && parse_rfc3339_millis(&record.created_at).unwrap_or(0) > cutoff_ms {
        return false;
    }
    // Borrow `&record.num`: lends the value without copying or giving it away.
    if !prerelease_allowed && is_prerelease_text(&record.num) {
        return false;
    }
    // Borrow `&record.num`: lends the value without copying or giving it away.
    return compare_texts(&record.num, &refused.version) == std::cmp::Ordering::Less && caret_compatible(&refused.version, &record.num);
}

/// What:
/// `pub fn set_aside_candidates(...) -> Result<Vec<String>, RegistryError>` lists the releases that may replace a refused
/// version,
/// newest first.
/// Why:
/// A candidate must be older,
/// in the same caret class,
/// not yanked,
/// not already rejected,
/// published before the cutoff,
/// and
/// pass the trust policy against the full history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function setAsideCandidates({ refusal, registry, settings, now, cutoffMs, rejected }): Promise<string[]>
/// ```
pub fn set_aside_candidates(refused: &TrustRefusal, request: &SetAsideRequest, rejected: &BTreeSet<String>) -> Result<Vec<String>, RegistryError> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&refused.name`: lends the value without copying or giving it away.
    // Typed binding: `history: Vec<VersionRecord>`.
    let history: Vec<VersionRecord> = request.registry.history(&refused.name, std::slice::from_ref(&refused.version))?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&refused.name`: lends the value without copying or giving it away.
    // Typed binding: `yanked: BTreeSet<String>`.
    // What:     `BTreeSet<T>`/`BTreeMap<K, V>` are sorted sets/maps (siblings: `HashSet`/`HashMap`, unordered and faster).
    // Why:      Sorted iteration makes the output and the tests deterministic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let yanked: BTreeSet<String> = request.registry.yanked(&refused.name)?;
    // Borrow `&refused.version`: lends the value without copying or giving it away.
    // Typed binding: `prerelease_allowed: bool`.
    let prerelease_allowed: bool = is_prerelease_text(&refused.version);
    // `let mut`: a binding that may change.
    // Typed binding: `candidates: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut candidates: Vec<String> = Vec::new();
    // Borrow `&history`: lends the value without copying or giving it away.
    for record in &history {
        // Borrow `&yanked`: lends the value without copying or giving it away.
        if !is_candidate(record, refused, prerelease_allowed, request.cutoff_ms, &yanked, rejected) {
            continue;
        }
        // Borrow `&refused.name`: lends the value without copying or giving it away.
        // Typed binding: `trust: Vec<String>`.
        // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
        let trust: Vec<String> = trust_problems(&refused.name, &record.num, &history, request.settings, request.now, &BTreeSet::new());
        if trust.is_empty() {
            // `.clone()`: an independent owned copy.
            candidates.push(record.num.clone());
        }
    }
    // Closure `|left, right|`: an anonymous function, like a TS arrow function.
    candidates.sort_by(|left, right| { return compare_texts(right, left); });
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(candidates);
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
fn crates_locking(snapshots: &[LockSnapshot], key: &str) -> Vec<String> {
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
            && versions.iter().any(|locked| { return locked.key() == key; })
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
fn move_crate(request: &SetAsideRequest, crate_dir: &str, refused: &TrustRefusal, candidates: &[String]) -> Result<String, String> {
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
        let outcome = request.runner.run(&request.root.join(crate_dir), &args)?;
        if outcome.ok {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.clone()`: an independent owned copy.
            return Ok(candidate.clone());
        }
        // Macro `eprintln!`: builds or prints text (or a list) from a template.
        eprintln!("cargo-shadow: {crate_dir}: cargo refused {} {candidate} ({})", refused.name, outcome.output.trim());
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
        let current: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(|error| { return error.to_string(); })?;
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
        let sources: Vec<String> = current.iter().map(|snapshot| { return snapshot.source.clone(); }).collect();
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
        let mut pending: Vec<TrustRefusal> = age_refusals(&added, request).map_err(|error| { return error.message; })?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&added`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        for refusal in trust_refusals(&added, request.settings, request.registry, request.now).map_err(|error| { return error.message; })? {
            // Closure `|known|`: an anonymous function, like a TS arrow function.
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            if !pending.iter().any(|known| { return known.name == refusal.name && known.version == refusal.version; }) {
                pending.push(refusal);
            }
        }
        // Closure `|refusal|`: an anonymous function, like a TS arrow function.
        // Borrow `&format`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        pending.retain(|refusal| { return !rejected.contains(&format!("{}@{}", refusal.name, refusal.version)); });
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
            let candidates: Vec<String> = set_aside_candidates(refusal, request, &rejected).map_err(|error| { return error.message; })?;
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
            let snapshots: Vec<LockSnapshot> = read_locks(request.root, request.crate_dirs).map_err(|error| { return error.to_string(); })?;
            // Borrow `&snapshots`: lends the value without copying or giving it away.
            for crate_dir in crates_locking(&snapshots, &key) {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Borrow `&crate_dir`: lends the value without copying or giving it away.
                // Typed binding: `instead: String`.
                let instead: String = move_crate(request, &crate_dir, refusal, &candidates)?;
                if instead.is_empty() {
                    // Macro `eprintln!`: builds or prints text (or a list) from a template.
                    eprintln!("cargo-shadow: {crate_dir}: no candidate for {key} resolved; its finding stands");
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
