//! The unit tests of the release-age and trust checks over newly locked versions.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::collections::{BTreeMap, BTreeSet};

// Imports `monochromatic_dependency_catalog::{Settings...`.
use monochromatic_dependency_catalog::{Settings, TrustPolicy};

// Imports `crate::checks::check_trust`.
use crate::checks::check_trust;
// Imports `crate::http::RegistryError`.
use crate::http::RegistryError;
// Imports `crate::lockfile::LockedVersion`.
use crate::lockfile::LockedVersion;
// Imports `crate::policy::VersionRecord`.
use crate::policy::VersionRecord;
// Imports `crate::registry::Registry`.
use crate::registry::Registry;

fn settings() -> Settings {
    return Settings {
        catalog_prune: false,
        minimum_release_age: 0,
        // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
        // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const list: string[] = [];   // or new Set() / new Map()
        // ```
        minimum_release_age_exclude: Vec::new(),
        minimum_release_age_exclude_prune: false,
        minimum_release_age_ignore_missing_time: true,
        minimum_release_age_strict: true,
        trust_policy: TrustPolicy::NoDowngrade,
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        trust_policy_exclude: Vec::new(),
        trust_policy_exclude_prune: false,
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        trust_policy_ignore_after: None,
        trust_lockfile: false,
    };
}

/// A registry with one canned history and one canned owner set.
struct OneCrate {
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    history: Vec<VersionRecord>,
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    // What:     `BTreeSet<T>`/`BTreeMap<K, V>` are sorted sets/maps (siblings: `HashSet`/`HashMap`, unordered and faster).
    // Why:      Sorted iteration makes the output and the tests deterministic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    owners: BTreeSet<String>,
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    // What:     `Cell<T>` holds a value that can be replaced through a shared reference (interior mutability; sibling: `RefCell<T>`, for non-copy values).
    // Why:      A method that takes `&self` still has to remember when it last made a request.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    owner_lookups: std::cell::Cell<u32>,
}

impl Registry for OneCrate {
    fn pubtimes(&self, _name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // Constructor `BTreeMap::new()`: an empty owned collection or a boxed value.
        return Ok(BTreeMap::new());
    }
    fn yanked(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
        return Ok(BTreeSet::new());
    }
    fn history(&self, _name: &str, _versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        return Ok(self.history.clone());
    }
    fn owners(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        self.owner_lookups.set(self.owner_lookups.get() + 1);
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `.clone()`: an independent owned copy.
        return Ok(self.owners.clone());
    }
}

fn record(num: &str, created_at: &str, publisher_id: &str) -> VersionRecord {
    return VersionRecord {
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        num: num.to_string(),
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        created_at: created_at.to_string(),
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        publisher_id: publisher_id.to_string(),
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        trustpub_repository: String::new(),
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        repository: String::new(),
    };
}

/// Review finding (lazy owner lookup):
/// without the owner list both releases rank as first-time publishers,
/// so the
/// ownerless evaluation finds nothing;
/// with it the earlier release (account 1 is a listed owner) ranks higher than the
/// candidate (account 2 is not),
/// which is a downgrade.
/// The owners must be asked even when the ownerless pass is clean.
#[test]
fn an_owner_who_raises_the_earlier_release_makes_a_later_release_by_a_stranger_a_downgrade() {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let registry: OneCrate = OneCrate {
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        history: vec![record("1.0.0", "2024-01-01T00:00:00Z", "1"), record("1.0.1", "2025-01-01T00:00:00Z", "2")],
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
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
        owners: ["1".to_string()].into_iter().collect(),
        // Constructor `Cell::new(`: an empty owned collection or a boxed value.
        owner_lookups: std::cell::Cell::new(0),
    };
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `candidate: Vec<LockedVersion>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let candidate: Vec<LockedVersion> = vec![LockedVersion { name: "dep".to_string(), version: "1.0.1".to_string() }];

    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    let messages = check_trust(&candidate, &settings(), &registry, 0).expect("checks");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(messages.len(), 1, "{messages:?}");
}

/// The lookup stays lazy where the owner list cannot matter:
/// a release by an account that already published an earlier
/// release ranks as a known publisher without owners,
/// so no owner can change the verdict and the rate-limited call is skipped.
#[test]
fn the_owner_list_is_not_asked_when_it_cannot_change_the_verdict() {
    // Typed binding: `registry: OneCrate`.
    let registry: OneCrate = OneCrate {
        // Macro `vec!`: builds or prints text (or a list) from a template.
        history: vec![record("1.0.0", "2024-01-01T00:00:00Z", "1"), record("1.0.1", "2025-01-01T00:00:00Z", "1")],
        // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
        owners: BTreeSet::new(),
        // Constructor `Cell::new(`: an empty owned collection or a boxed value.
        owner_lookups: std::cell::Cell::new(0),
    };
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `candidate: Vec<LockedVersion>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let candidate: Vec<LockedVersion> = vec![LockedVersion { name: "dep".to_string(), version: "1.0.1".to_string() }];

    // Borrow `&candidate`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let messages = check_trust(&candidate, &settings(), &registry, 0).expect("checks");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(messages.is_empty(), "{messages:?}");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(registry.owner_lookups.get(), 0);
}
