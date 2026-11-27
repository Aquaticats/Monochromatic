//! What:
//! The choice of an older release that replaces one the policy refused:
//! the release-age and trust checks a replacement must pass,
//! and the order the candidates are tried in.
//! Why:
//! Split out of `set_aside.rs`,
//! which moves a refused version down to an older one,
//! to keep that file under the code-line limit.
//! The parent module calls these items,
//! so the round behaves as it did.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports the ordered set of owned text that holds the yanked and the already rejected versions.
use std::collections::BTreeSet;

/// Imports the request of a set-aside round:
/// the registry,
/// the settings and the time it judges by.
use super::SetAsideRequest;
/// Imports the refusal a release must replace and the reader that groups a run's new versions by crate name.
use crate::checks::{
    TrustRefusal,
    group_by_name,
};
/// Imports the registry failure a candidate lookup reports.
use crate::http::RegistryError;
/// Imports the locked version a refusal is about.
use crate::lockfile::LockedVersion;
/// Imports one published release of a crate and the release-age and trust checks a candidate must pass.
use crate::policy::{
    VersionRecord,
    release_age_problems,
    trust_problems,
};
/// Imports the version comparison and the prerelease and caret-class readers the choice orders by.
use crate::version_order::{
    caret_compatible,
    compare_texts,
    is_prerelease_text,
};

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
pub(crate) fn age_refusals(
    added: &[LockedVersion],
    request: &SetAsideRequest,
) -> Result<Vec<TrustRefusal>, RegistryError> {
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
            let pubtime: &str = times.get(&version).map(String::as_str).unwrap_or("");
            // Borrow `&name`: lends the value without copying or giving it away.
            // Typed binding: `problems: Vec<String>`.
            let problems: Vec<String> =
                release_age_problems(&name, &version, pubtime, request.settings, request.now);
            if !problems.is_empty() {
                // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
                // Why:      The copy can be stored or moved elsewhere while the original stays in place.
                // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // structuredClone(value)  // or [...list] / {...object}
                // ```
                refusals.push(TrustRefusal {
                    name: name.clone(),
                    version,
                    problems,
                });
            }
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(refusals);
}

/// What:
/// `fn age_allows(record, refused, request) -> bool` says whether a release passes the release-age condition of the choice.
/// Why:
/// The final check (`check_release_age`) refuses only under `minimumReleaseAgeStrict`,
/// skips crates listed in
/// `minimumReleaseAgeExclude`,
/// and refuses a release with no readable publish time unless
/// `minimumReleaseAgeIgnoreMissingTime` is set.
/// A candidate is judged by the same function,
/// so a release the final check accepts is never rejected here,
/// and one it would refuse is never chosen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ageAllows(record, refused, request): boolean {
///   return !strict || releaseAgeProblems(refused.name, record.num, record.createdAt, settings, now).length === 0;
/// }
/// ```
fn age_allows(
    record: &VersionRecord,
    refused: &TrustRefusal,
    request: &SetAsideRequest,
) -> bool {
    if request.settings.minimum_release_age == 0 || !request.settings.minimum_release_age_strict {
        return true;
    }
    // Borrow `&refused.name`: lends the value without copying or giving it away.
    // Typed binding: `problems: Vec<String>`.
    let problems: Vec<String> = release_age_problems(
        &refused.name,
        &record.num,
        &record.created_at,
        request.settings,
        request.now,
    );
    return problems.is_empty();
}

/// What:
/// `fn is_candidate(record, refused, prerelease_allowed, yanked, rejected, request) -> bool` says whether one published release may replace the refused version.
/// `&` marks a borrowed read-only argument;
/// `BTreeSet<String>` is an ordered set of owned text (sibling:
/// `HashSet`,
/// unordered).
/// Why:
/// A replacement must be older,
/// not yanked,
/// not already rejected,
/// pass the release-age condition as the final check applies it,
/// and be a prerelease only when the refused version is one.
/// Whether it fits the manifest's range is decided by cargo,
/// which refuses `--precise` for a version outside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCandidate(args): boolean
/// ```
fn is_candidate(
    record: &VersionRecord,
    refused: &TrustRefusal,
    prerelease_allowed: bool,
    yanked: &BTreeSet<String>,
    rejected: &BTreeSet<String>,
    request: &SetAsideRequest,
) -> bool {
    if yanked.contains(&record.num)
        || rejected.contains(&format!("{}@{}", refused.name, record.num))
    {
        return false;
    }
    if !age_allows(record, refused, request) {
        return false;
    }
    if !prerelease_allowed && is_prerelease_text(&record.num) {
        return false;
    }
    // Borrow `&record.num`: lends the value without copying or giving it away.
    return compare_texts(&record.num, &refused.version) == std::cmp::Ordering::Less;
}

/// What:
/// `fn sort_newest_first(list: &mut [String])` orders version texts with the newer first,
/// in place.
/// `&mut [String]` is a borrowed view of a list that the function may reorder.
/// Why:
/// The candidates are tried newest first;
/// the lists are short,
/// so a plain insertion sort with the crate's version comparison does,
/// and it avoids a closure for `sort_by`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sortNewestFirst = (list: string[]): void => { list.sort((a, b) => compareVersions(b, a)); };
/// ```
fn sort_newest_first(list: &mut [String]) {
    // `let mut`: a binding that may change.
    // Typed binding: `position: usize`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let mut position: usize = 1;
    while position < list.len() {
        // `let mut`: a binding that may change.
        // Typed binding: `inner: usize`.
        let mut inner: usize = position;
        // Borrow `&list`: lends the value without copying or giving it away.
        while inner > 0 && compare_texts(&list[inner - 1], &list[inner]) == std::cmp::Ordering::Less
        {
            list.swap(inner - 1, inner);
            inner -= 1;
        }
        position += 1;
    }
}

/// What:
/// `pub fn set_aside_candidates(...) -> Result<Vec<String>, RegistryError>` lists the releases that may replace a refused
/// version,
/// the refused version's own caret class first and each group newest first.
/// Why:
/// A candidate must be older,
/// not yanked,
/// not already rejected,
/// pass the release-age condition,
/// and
/// pass the trust policy against the full history with the crate's owners,
/// as the final check does.
/// Cargo takes the first one that fits the manifest's requirements,
/// so the order puts the least surprising ones first;
/// the caret class is a preference,
/// not a boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function setAsideCandidates({ refusal, registry, settings, now, rejected }): Promise<string[]>
/// ```
pub fn set_aside_candidates(
    refused: &TrustRefusal,
    request: &SetAsideRequest,
    rejected: &BTreeSet<String>,
) -> Result<Vec<String>, RegistryError> {
    let history: Vec<VersionRecord> = request
        .registry
        .history(&refused.name, std::slice::from_ref(&refused.version))?;
    let yanked: BTreeSet<String> = request.registry.yanked(&refused.name)?;
    // Owners raise levels, so the candidate is judged with them, exactly as the final check judges it.
    let owners: BTreeSet<String> = request.registry.owners(&refused.name)?;
    let prerelease_allowed: bool = is_prerelease_text(&refused.version);
    // `let mut`: a binding that may change.
    // Typed binding: `same_class: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut same_class: Vec<String> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `other_class: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut other_class: Vec<String> = Vec::new();
    for record in &history {
        // Borrow `&yanked`: lends the value without copying or giving it away.
        if !is_candidate(
            record,
            refused,
            prerelease_allowed,
            &yanked,
            rejected,
            request,
        ) {
            continue;
        }
        // Borrow `&refused.name`: lends the value without copying or giving it away.
        // Typed binding: `trust: Vec<String>`.
        let trust: Vec<String> = trust_problems(
            &refused.name,
            &record.num,
            &history,
            request.settings,
            request.now,
            &owners,
        );
        if !trust.is_empty() {
            continue;
        }
        // Borrow `&refused.version`: lends the value without copying or giving it away.
        if caret_compatible(&refused.version, &record.num) {
            // `.clone()`: an independent owned copy.
            same_class.push(record.num.clone());
        } else {
            // `.clone()`: an independent owned copy.
            other_class.push(record.num.clone());
        }
    }
    // Borrow `&mut same_class`: lends the value without copying or giving it away.
    sort_newest_first(&mut same_class);
    // Borrow `&mut other_class`: lends the value without copying or giving it away.
    sort_newest_first(&mut other_class);
    same_class.extend(other_class);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(same_class);
}
