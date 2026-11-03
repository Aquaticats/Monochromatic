//! What:
//! Release-age and trust checks over a set of newly locked versions,
//! one registry lookup per crate.
//! Why:
//! Both the install step and the update gate every version that no lockfile held before through these two checks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checks.ts
//! export async function checkReleaseAge({ versions, settings, registry, now }): Promise<{ problems: string[]; warnings: string[] }>
//! export async function trustRefusals({ versions, settings, registry, now }): Promise<TrustRefusal[]>
//! ```

/// What:
/// `use std::collections::BTreeSet;` imports `BTreeSet` from the standard library so this file can name it unqualified.
/// Why:
/// `BTreeSet` is used by `crate_trust_refusals`,
/// `trust_refusals`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BTreeSet } from 'std/collections';
/// ```
use std::collections::BTreeSet;

/// What:
/// `use monochromatic_dependency_catalog::{Settings, TrustPolicy};` imports `Settings`,
/// `TrustPolicy` from the external crate `monochromatic_dependency_catalog` so this file can name them unqualified.
/// Why:
/// `Settings` is used by `check_release_age`,
/// `crate_trust_refusals`,
/// `trust_refusals`,
/// `check_trust`;
/// `TrustPolicy` is used by `trust_refusals`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Settings, TrustPolicy } from 'monochromatic_dependency_catalog';
/// ```
use monochromatic_dependency_catalog::{Settings, TrustPolicy};

/// What:
/// `use crate::http::RegistryError;` imports `RegistryError` from another module of this crate so this file can name it unqualified.
/// Why:
/// `RegistryError` is used by `check_release_age`,
/// `trust_refusals`,
/// `check_trust`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegistryError } from 'crate/http';
/// ```
use crate::http::RegistryError;
/// What:
/// `use crate::lockfile::LockedVersion;` imports `LockedVersion` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LockedVersion` is used by `group_by_name`,
/// `check_release_age`,
/// `trust_refusals`,
/// `check_trust`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion } from 'crate/lockfile';
/// ```
use crate::lockfile::LockedVersion;
/// What:
/// `use crate::policy::{owners_can_change_verdict, release_age_problems, trust_problems};` imports `release_age_problems`,
/// `trust_problems` from another module of this crate so this file can name them unqualified.
/// Why:
/// `release_age_problems` is used by `check_release_age`;
/// `trust_problems` is used by `crate_trust_refusals`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { release_age_problems, trust_problems } from 'crate/policy';
/// ```
use crate::policy::{owners_can_change_verdict, release_age_problems, trust_problems};
/// What:
/// `use crate::registry::Registry;` imports `Registry` from another module of this crate so this file can name it unqualified.
/// Why:
/// `Registry` is used by `check_release_age`,
/// `trust_refusals`,
/// `check_trust`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Registry } from 'crate/registry';
/// ```
use crate::registry::Registry;

/// The findings of a check pass.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckOutcome {
    /// Findings that refuse the run.
    pub problems: Vec<String>,
    /// Findings reported without refusing (release age with `minimumReleaseAgeStrict = false`).
    pub warnings: Vec<String>,
}

/// One version the trust policy refuses,
/// with every finding against it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustRefusal {
    /// The crate.
    pub name: String,
    /// The refused version.
    pub version: String,
    /// The findings.
    pub problems: Vec<String>,
}

/// What:
/// `fn group_for(groups, name) -> Option<&mut (String, Vec<String>)>` finds the group of one crate name so another version can be added to it.
/// Why:
/// Versions of the same crate are checked together,
/// so each crate name has one group.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const groupFor = (groups: [string, string[]][], name: string) => groups.find((g) => g[0] === name);
/// ```
fn group_for<'a>(groups: &'a mut [(String, Vec<String>)], name: &str) -> Option<&'a mut (String, Vec<String>)> {
    for group in groups {
        if group.0 == name {
            return Some(group);
        }
    }
    return None;
}

/// Versions grouped by crate name,
/// in first-seen order,
/// so each crate costs one lookup.
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
        match group_for(&mut groups, &locked.name) {
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
/// Findings are problems when `minimumReleaseAgeStrict` is true,
/// warnings otherwise;
/// age zero checks nothing.
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
            let pubtime: &str = times.get(&version).map(String::as_str).unwrap_or("");
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
/// `fn group_name(group: &(String, Vec<String>)) -> String` returns the crate name of one group of locked versions.
/// Why:
/// The registry prefetches the index of every crate name once,
/// before the checks read them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const groupName = (group: [string, string[]]) => group[0];
/// ```
fn group_name(group: &(String, Vec<String>)) -> String {
    return group.0.clone();
}

/// What:
/// `pub fn trust_refusals_partial(...) -> (Vec<TrustRefusal>, Option<RegistryError>)` lists the trust refusals found before a registry
/// request failed,
/// and the failure if one did.
/// Why:
/// `trust_refusals` (which returns an error and drops what it found) and `check_trust` (which keeps it) both start from this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trustRefusalsPartial(versions, settings, registry, now): [TrustRefusal[], Error | undefined]
/// ```
pub fn trust_refusals_partial(
    versions: &[LockedVersion],
    settings: &Settings,
    registry: &dyn Registry,
    now: i64,
) -> (Vec<TrustRefusal>, Option<RegistryError>) {
    // `let mut`: a binding that may change.
    // Typed binding: `refusals: Vec<TrustRefusal>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut refusals: Vec<TrustRefusal> = Vec::new();
    if settings.trust_policy == TrustPolicy::Off {
        // `None`: the present or the absent value of an `Option`.
        return (refusals, None);
    }
    let groups: Vec<(String, Vec<String>)> = group_by_name(versions);
    if groups.is_empty() {
        // `None`: the present or the absent value of an `Option`.
        return (refusals, None);
    }
    eprintln!("cargo-shadow: checking the trust policy for {} versions of {} crates", versions.len(), groups.len());
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    // Borrow `&groups.iter`: lends the value without copying or giving it away.
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    // Adapter `.map(`: transforms or selects items with a closure.
    if let Err(error) = registry.prefetch(&groups.iter().map(group_name).collect::<BTreeSet<String>>()) {
        // `Some`: the present or the absent value of an `Option`.
        return (refusals, Some(error));
    }
    for (name, name_versions) in groups {
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&name`: lends the value without copying or giving it away.
        let history = match registry.history(&name, &name_versions) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(found) => found,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `Some`: the present or the absent value of an `Option`.
            Err(error) => return (refusals, Some(error)),
        };
        let without_owners = crate_trust_refusals(&name, &name_versions, &history, settings, now, &BTreeSet::new());
        // The owner list only raises levels, so it can add a refusal as well as remove one: it is asked for whenever the
        // ownerless evaluation refused, or could be wrong because an earlier release might be raised above the checked one.
        if without_owners.is_empty() && !owners_can_change_verdict(&name, &name_versions, &history, settings, now) {
            continue;
        }
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&name`: lends the value without copying or giving it away.
        match registry.owners(&name) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Borrow `&name`: lends the value without copying or giving it away.
            Ok(owner_ids) => refusals.extend(crate_trust_refusals(&name, &name_versions, &history, settings, now, &owner_ids)),
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => {
                // Review finding (3c P2-1): the findings without the owners are provisional (the owners could clear them), so they are
                // not carried as refusals; only completed evaluations are.
                drop(without_owners);
                // `Some`: the present or the absent value of an `Option`.
                return (refusals, Some(error));
            }
        }
    }
    // `None`: the present or the absent value of an `Option`.
    return (refusals, None);
}

/// What:
/// `pub fn trust_refusals(...) -> Result<Vec<TrustRefusal>, RegistryError>` lists the trust refusals,
/// or the registry failure.
/// Why:
/// The set-aside needs structured refusals and treats a lookup failure as an error of the whole round.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trustRefusals(versions, settings, registry, now): TrustRefusal[]  // throws RegistryError
/// ```
pub fn trust_refusals(
    versions: &[LockedVersion],
    settings: &Settings,
    // `dyn` trait object `dyn Registry`: any type that implements the trait.
    registry: &dyn Registry,
    now: i64,
) -> Result<Vec<TrustRefusal>, RegistryError> {
    let (refusals, failure): (Vec<TrustRefusal>, Option<RegistryError>) = trust_refusals_partial(versions, settings, registry, now);
    // `match`: picks the arm whose pattern fits the value.
    return match failure {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        Some(error) => Err(error),
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        None => Ok(refusals),
    };
}

/// What:
/// `pub fn check_trust(versions, settings, registry, now) -> Result<Vec<String>, RegistryError>` returns the trust findings for
/// versions.
/// Why:
/// When a registry request fails after some findings are known,
/// the findings are returned with a line saying the rest is unknown,
/// so a known refusal is never erased by a later lookup error;
/// with no finding yet,
/// the error is returned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function checkTrust(versions, settings, registry, now): Promise<string[]>  // throws only when nothing is known
/// ```
pub fn check_trust(
    versions: &[LockedVersion],
    settings: &Settings,
    registry: &dyn Registry,
    now: i64,
) -> Result<Vec<String>, RegistryError> {
    let mut messages: Vec<String> = Vec::new();
    let (refusals, failure): (Vec<TrustRefusal>, Option<RegistryError>) = trust_refusals_partial(versions, settings, registry, now);
    for refusal in refusals {
        messages.extend(refusal.problems);
    }
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    if let Some(error) = failure {
        if messages.is_empty() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(error);
        }
        // Macro `format!`: builds or prints text (or a list) from a template.
        messages.push(format!("a later registry lookup failed ({}), so only the findings above are known", error.message));
    }
    return Ok(messages);
}

/// What:
/// `pub fn check_policy(versions, settings, registry, now) -> Result<CheckOutcome, RegistryError>` returns the release-age and trust
/// findings for versions together,
/// with the release-age warnings.
/// Why:
/// Review finding (2c 2):
/// a lookup error in one check must not erase a refusal the other already found.
/// With a known finding the error becomes one more finding;
/// with none,
/// it is returned as the operational error it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function checkPolicy(versions, settings, registry, now): Promise<CheckOutcome>
/// ```
pub fn check_policy(
    versions: &[LockedVersion],
    settings: &Settings,
    // `dyn` trait object `dyn Registry`: any type that implements the trait.
    registry: &dyn Registry,
    now: i64,
) -> Result<CheckOutcome, RegistryError> {
    // Typed binding: `age: Result<CheckOutcome, RegistryError>`.
    let age: Result<CheckOutcome, RegistryError> = check_release_age(versions, settings, registry, now);
    // Typed binding: `trust: Result<Vec<String>, RegistryError>`.
    let trust: Result<Vec<String>, RegistryError> = check_trust(versions, settings, registry, now);
    // `match`: picks the arm whose pattern fits the value.
    return match (age, trust) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        (Ok(mut found), Ok(more)) => {
            found.problems.extend(more);
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(found)
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        (Ok(mut found), Err(error)) => {
            if found.problems.is_empty() {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                return Err(error);
            }
            // Macro `format!`: builds or prints text (or a list) from a template.
            found.problems.push(format!("the trust lookup failed after the release-age finding ({})", error.message));
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(found)
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        (Err(error), Ok(more)) => {
            if more.is_empty() {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                return Err(error);
            }
            // `let mut`: a binding that may change.
            // Typed binding: `problems: Vec<String>`.
            let mut problems: Vec<String> = more;
            // Macro `format!`: builds or prints text (or a list) from a template.
            problems.push(format!("the release-age lookup failed after the trust finding ({})", error.message));
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Constructor `Vec::new()`: an empty owned collection or a boxed value.
            Ok(CheckOutcome { problems, warnings: Vec::new() })
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        (Err(error), Err(_)) => Err(error),
    };
}
