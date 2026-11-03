//! What:
//! Release-age and trust checks over a set of newly locked versions, one registry lookup per crate.
//! Why:
//! Both the install step and the update gate every version that no lockfile held before through these two checks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checks.ts
//! export async function checkReleaseAge({ versions, settings, registry, now }): Promise<{ problems: string[]; warnings: string[] }>
//! export async function trustRefusals({ versions, settings, registry, now }): Promise<TrustRefusal[]>
//! ```

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::collections::BTreeSet;

// Imports `monochromatic_dependency_catalog::{Settings...`.
use monochromatic_dependency_catalog::{Settings, TrustPolicy};

// Imports `crate::http::RegistryError`.
use crate::http::RegistryError;
// Imports `crate::lockfile::LockedVersion`.
use crate::lockfile::LockedVersion;
// Imports `crate::policy::{release_age_problems, trust...`.
use crate::policy::{release_age_problems, trust_problems};
// Imports `crate::registry::Registry`.
use crate::registry::Registry;

/// The findings of a check pass.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckOutcome {
    /// Findings that refuse the run.
    pub problems: Vec<String>,
    /// Findings reported without refusing (release age with `minimumReleaseAgeStrict = false`).
    pub warnings: Vec<String>,
}

/// One version the trust policy refuses, with every finding against it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustRefusal {
    /// The crate.
    pub name: String,
    /// The refused version.
    pub version: String,
    /// The findings.
    pub problems: Vec<String>,
}

/// Versions grouped by crate name, in first-seen order, so each crate costs one lookup.
pub fn group_by_name(versions: &[LockedVersion]) -> Vec<(String, Vec<String>)> {
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
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for locked in versions {
        // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
        // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
        // ```
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      The collection stays usable after the walk and no item is copied until it must be.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        match groups.iter_mut().find(|group| { return group.0 == locked.name; }) {
            // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
            // Why:      Absence is a value the caller must check; there is no `null`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Some(value) is  value
            // // None        is  undefined
            // ```
            // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
            // Why:      The copy can be stored or moved elsewhere while the original stays in place.
            // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // structuredClone(value)  // or [...list] / {...object}
            // ```
            Some(group) => group.1.push(locked.version.clone()),
            // `None`: the present or the absent value of an `Option`.
            // `.clone()`: an independent owned copy.
            // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
            // Why:      Text building and printing are written as one call instead of string concatenation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // `..${x}..`   // template literal
            // console.error(text);   // eprintln!
            // [a, b]            // vec![a, b]
            // ```
            None => groups.push((locked.name.clone(), vec![locked.version.clone()])),
        }
    }
    return groups;
}

/// What:
/// `pub fn check_release_age(...) -> Result<CheckOutcome, RegistryError>` checks each version's publish time from the sparse
/// index against `minimumReleaseAge`.
/// Why:
/// Findings are problems when `minimumReleaseAgeStrict` is true, warnings otherwise; age zero checks nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function checkReleaseAge({ versions, settings, registry, now }): Promise<CheckOutcome>
/// ```
pub fn check_release_age(
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    versions: &[LockedVersion],
    // Borrow `&Settings`: lends the value without copying or giving it away.
    settings: &Settings,
    // Borrow `&dyn`: lends the value without copying or giving it away.
    // What:     `&dyn Trait` is a borrowed value of any type that implements the trait (a TS interface type); the call goes through a lookup table at run time.
    // Why:      The caller may pass the real implementation or a test fake.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function f(registry: Registry) { ... }  // an interface type
    // ```
    registry: &dyn Registry,
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    now: i64,
) -> Result<CheckOutcome, RegistryError> {
    if settings.minimum_release_age == 0 {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        return Ok(CheckOutcome::default());
    }
    // `let mut`: a binding that may change.
    // Typed binding: `findings: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut findings: Vec<String> = Vec::new();
    for (name, name_versions) in group_by_name(versions) {
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        // Borrow `&name`: lends the value without copying or giving it away.
        let times = registry.pubtimes(&name)?;
        for version in name_versions {
            // Closure `|text|`: an anonymous function, like a TS arrow function.
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
            let pubtime: &str = times.get(&version).map(|text| { return text.as_str(); }).unwrap_or("");
            // Borrow `&name`: lends the value without copying or giving it away.
            findings.extend(release_age_problems(&name, &version, pubtime, settings, now));
        }
    }
    if settings.minimum_release_age_strict {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        return Ok(CheckOutcome { problems: findings, warnings: Vec::new() });
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    return Ok(CheckOutcome { problems: Vec::new(), warnings: findings });
}

/// The refusals of several versions of one crate under one owner list.
fn crate_trust_refusals(
    // Borrow `&str`: lends the value without copying or giving it away.
    name: &str,
    // Borrow `&[`: lends the value without copying or giving it away.
    versions: &[String],
    // Borrow `&[`: lends the value without copying or giving it away.
    history: &[crate::policy::VersionRecord],
    // Borrow `&Settings`: lends the value without copying or giving it away.
    settings: &Settings,
    now: i64,
    // Borrow `&BTreeSet`: lends the value without copying or giving it away.
    // What:     `BTreeSet<T>`/`BTreeMap<K, V>` are sorted sets/maps (siblings: `HashSet`/`HashMap`, unordered and faster).
    // Why:      Sorted iteration makes the output and the tests deterministic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    owner_ids: &BTreeSet<String>,
) -> Vec<TrustRefusal> {
    // `let mut`: a binding that may change.
    // Typed binding: `refusals: Vec<TrustRefusal>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut refusals: Vec<TrustRefusal> = Vec::new();
    for version in versions {
        // Typed binding: `problems: Vec<String>`.
        let problems: Vec<String> = trust_problems(name, version, history, settings, now, owner_ids);
        if !problems.is_empty() {
            // `.clone()`: an independent owned copy.
            // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
            // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // String(value)  // JS strings are always owned values
            // ```
            refusals.push(TrustRefusal { name: name.to_string(), version: version.clone(), problems });
        }
    }
    return refusals;
}

/// What:
/// `pub fn trust_refusals(...) -> Result<Vec<TrustRefusal>, RegistryError>` checks the trust policy: one history lookup per
/// crate, plus an owners lookup only for a crate that would otherwise be refused.
/// Why:
/// A listed owner's first release ranks as a known publisher, so a refusal is rechecked against the current owner list
/// before it stands; most crates pass without that request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function trustRefusals({ versions, settings, registry, now }): Promise<TrustRefusal[]>
/// ```
pub fn trust_refusals(
    // Borrow `&[`: lends the value without copying or giving it away.
    versions: &[LockedVersion],
    // Borrow `&Settings`: lends the value without copying or giving it away.
    settings: &Settings,
    // Borrow `&dyn`: lends the value without copying or giving it away.
    // `dyn` trait object `dyn Registry`: any type that implements the trait.
    registry: &dyn Registry,
    now: i64,
) -> Result<Vec<TrustRefusal>, RegistryError> {
    if settings.trust_policy == TrustPolicy::Off {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        return Ok(Vec::new());
    }
    // Typed binding: `groups: Vec<(String, Vec<String>)>`.
    let groups: Vec<(String, Vec<String>)> = group_by_name(versions);
    if groups.is_empty() {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        return Ok(Vec::new());
    }
    // Macro `eprintln!`: builds or prints text (or a list) from a template.
    eprintln!("cargo-shadow: checking the trust policy for {} versions of {} crates", versions.len(), groups.len());
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|group|`: an anonymous function, like a TS arrow function.
    // Borrow `&groups.iter`: lends the value without copying or giving it away.
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
    registry.prefetch(&groups.iter().map(|group| { return group.0.clone(); }).collect::<BTreeSet<String>>())?;
    // `let mut`: a binding that may change.
    // Typed binding: `refusals: Vec<TrustRefusal>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut refusals: Vec<TrustRefusal> = Vec::new();
    for (name, name_versions) in groups {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&name`: lends the value without copying or giving it away.
        let history = registry.history(&name, &name_versions)?;
        // Borrow `&name`: lends the value without copying or giving it away.
        // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
        let without_owners = crate_trust_refusals(&name, &name_versions, &history, settings, now, &BTreeSet::new());
        if without_owners.is_empty() {
            continue;
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&name`: lends the value without copying or giving it away.
        // Typed binding: `owner_ids: BTreeSet<String>`.
        let owner_ids: BTreeSet<String> = registry.owners(&name)?;
        // Borrow `&name`: lends the value without copying or giving it away.
        refusals.extend(crate_trust_refusals(&name, &name_versions, &history, settings, now, &owner_ids));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(refusals);
}

/// The findings of `trust_refusals`, flattened.
pub fn check_trust(
    // Borrow `&[`: lends the value without copying or giving it away.
    versions: &[LockedVersion],
    // Borrow `&Settings`: lends the value without copying or giving it away.
    settings: &Settings,
    // Borrow `&dyn`: lends the value without copying or giving it away.
    // `dyn` trait object `dyn Registry`: any type that implements the trait.
    registry: &dyn Registry,
    now: i64,
) -> Result<Vec<String>, RegistryError> {
    // `let mut`: a binding that may change.
    // Typed binding: `messages: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut messages: Vec<String> = Vec::new();
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    for refusal in trust_refusals(versions, settings, registry, now)? {
        messages.extend(refusal.problems);
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(messages);
}
