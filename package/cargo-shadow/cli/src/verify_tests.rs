//! The unit tests of the check of every locked entry:
//! what pnpm's install does to a lockfile unless `trustLockfile` is set.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use monochromatic_dependency_catalog::{Settings, TrustPolicy};

use crate::crates::LockSnapshot;
use crate::http::RegistryError;
use crate::policy::VersionRecord;
use crate::registry::Registry;
use crate::test_scratch::scratch;
use crate::verify::{verify_added_since_base, verify_locked_entries};

const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";

fn settings() -> Settings {
    return Settings {
        catalog_prune: false,
        minimum_release_age: 0,
        minimum_release_age_exclude: Vec::new(),
        minimum_release_age_exclude_prune: false,
        minimum_release_age_ignore_missing_time: true,
        minimum_release_age_strict: true,
        trust_policy: TrustPolicy::NoDowngrade,
        trust_policy_exclude: Vec::new(),
        trust_policy_exclude_prune: false,
        trust_policy_ignore_after: None,
        trust_lockfile: false,
    };
}

/// A registry that counts its history lookups.
struct CountingRegistry {
    lookups: Cell<u32>,
}

impl Registry for CountingRegistry {
    fn pubtimes(&self, _name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        return Ok(BTreeMap::new());
    }
    fn yanked(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return Ok(BTreeSet::new());
    }
    fn history(&self, _name: &str, _versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        self.lookups.set(self.lookups.get() + 1);
        let trusted = VersionRecord {
            num: "1.0.0".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            publisher_id: String::new(),
            trustpub_repository: "o/r".to_string(),
            repository: String::new(),
        };
        let downgraded = VersionRecord {
            num: "1.0.1".to_string(),
            created_at: "2026-02-01T00:00:00Z".to_string(),
            publisher_id: "9".to_string(),
            trustpub_repository: String::new(),
            repository: String::new(),
        };
        return Ok(vec![trusted, downgraded]);
    }
    fn owners(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return Ok(BTreeSet::new());
    }
}

fn snapshot(version: &str) -> LockSnapshot {
    return LockSnapshot {
        crate_dir: "package/v/one".to_string(),
        source: format!("version = 4\n\n[[package]]\nname = \"dep\"\nversion = \"{version}\"\nsource = \"{REGISTRY}\"\n"),
    };
}

#[test]
fn every_locked_version_is_checked_not_only_the_ones_added_since_a_base() {
    let root = scratch();
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };

    let problems = verify_locked_entries(root.path(), &[snapshot("1.0.1")], &settings(), &registry, 0).expect("verifies");

    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("dep@1.0.1 has trust level 1"));
}

#[test]
fn a_passing_lockfile_is_remembered_and_costs_no_lookup_the_next_time() {
    let root = scratch();
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };

    assert!(verify_locked_entries(root.path(), &[snapshot("1.0.0")], &settings(), &registry, 0).expect("verifies").is_empty());
    let after_first: u32 = registry.lookups.get();
    assert!(verify_locked_entries(root.path(), &[snapshot("1.0.0")], &settings(), &registry, 0).expect("verifies").is_empty());

    assert!(after_first > 0);
    assert_eq!(registry.lookups.get(), after_first, "the second call asked the registry again");
}

#[test]
fn a_failing_lockfile_is_never_remembered() {
    let root = scratch();
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };

    verify_locked_entries(root.path(), &[snapshot("1.0.1")], &settings(), &registry, 0).expect("verifies");
    let after_first: u32 = registry.lookups.get();
    let again = verify_locked_entries(root.path(), &[snapshot("1.0.1")], &settings(), &registry, 0).expect("verifies");

    assert_eq!(again.len(), 1);
    assert!(registry.lookups.get() > after_first);
}

#[test]
fn a_changed_lockfile_or_a_changed_policy_is_checked_again() {
    let root = scratch();
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };
    verify_locked_entries(root.path(), &[snapshot("1.0.0")], &settings(), &registry, 0).expect("verifies");
    let after_first: u32 = registry.lookups.get();

    verify_locked_entries(root.path(), &[snapshot("1.0.1")], &settings(), &registry, 0).expect("verifies");
    let after_new_lock: u32 = registry.lookups.get();
    let mut stricter: Settings = settings();
    stricter.trust_policy_exclude = vec!["other".to_string()];
    verify_locked_entries(root.path(), &[snapshot("1.0.0")], &stricter, &registry, 0).expect("verifies");

    assert!(after_new_lock > after_first);
    assert!(registry.lookups.get() > after_new_lock);
}

#[test]
fn since_the_base_only_versions_no_lockfile_held_there_are_checked() {
    let root = scratch();
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };
    let current: LockSnapshot = snapshot("1.0.1");
    let base_with_it: String = snapshot("1.0.1").source;
    let base_without_it: String = snapshot("1.0.0").source;
    let _ = root;

    let held = verify_added_since_base(std::slice::from_ref(&current), &|_dir| { return base_with_it.clone(); }, &settings(), &registry, 0).expect("verifies");
    let lookups_when_held: u32 = registry.lookups.get();
    let new = verify_added_since_base(std::slice::from_ref(&current), &|_dir| { return base_without_it.clone(); }, &settings(), &registry, 0).expect("verifies");

    assert!(held.is_empty());
    assert_eq!(lookups_when_held, 0, "a version the base held costs no registry lookup");
    assert_eq!(new.len(), 1);
    assert!(new[0].contains("dep@1.0.1 has trust level 1"));
}

#[test]
fn since_the_base_a_lockfile_absent_there_counts_as_new_in_full() {
    let registry: CountingRegistry = CountingRegistry { lookups: Cell::new(0) };

    let problems = verify_added_since_base(&[snapshot("1.0.1")], &|_dir| { return String::new(); }, &settings(), &registry, 0).expect("verifies");

    assert_eq!(problems.len(), 1);
}

/// A registry whose one release was published three hours before the fixed clock.
struct YoungRegistry;

impl Registry for YoungRegistry {
    fn pubtimes(&self, _name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        return Ok(BTreeMap::from([("1.0.0".to_string(), "2026-10-29T09:00:00Z".to_string())]));
    }
    fn yanked(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
        // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const list: string[] = [];   // or new Set() / new Map()
        // ```
        return Ok(BTreeSet::new());
    }
    fn history(&self, _name: &str, _versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        return Ok(vec![VersionRecord {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            num: "1.0.0".to_string(),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            created_at: "2026-10-29T09:00:00Z".to_string(),
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
            // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // TS has one type for this: string
            // ```
            publisher_id: String::new(),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            trustpub_repository: "o/r".to_string(),
            // Constructor `String::new()`: an empty owned collection or a boxed value.
            repository: String::new(),
        }]);
    }
    fn owners(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
        return Ok(BTreeSet::new());
    }
}

/// Delegated pick 2 (2026-11-03):
/// the release age and the trust policy both apply to the versions new since the base,
/// on the frozen path as on the local one.
#[test]
fn since_the_base_the_release_age_is_checked_as_well_as_the_trust() {
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
    let mut one_day: Settings = settings();
    one_day.minimum_release_age = 1_440;
    one_day.minimum_release_age_ignore_missing_time = false;
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // Typed binding: `now: i64`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let now: i64 = crate::time_text::parse_rfc3339_millis("2026-10-29T12:00:00Z").expect("a fixed time");

    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Constructor `String::new()`: an empty owned collection or a boxed value.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let problems = verify_added_since_base(&[snapshot("1.0.0")], &|_dir| { return String::new(); }, &one_day, &YoungRegistry, now).expect("verifies");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(problems.len(), 1, "{problems:?}");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(problems[0].contains("minimumReleaseAge"), "{problems:?}");
}
