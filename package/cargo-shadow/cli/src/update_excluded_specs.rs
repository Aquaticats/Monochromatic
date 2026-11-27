//! What:
//! Which locked crates a catalog exclude selector exempts from the release-age cutoff.
//! A selector that names no version,
//! such as `serde` or `serde*`,
//! exempts every version of the crates it fits,
//! so those crates may move to their newest release;
//! a selector that names exact versions exempts only those versions and moves nothing.
//! Why:
//! A full update hands this list to `cargo update`,
//! and a crate missing from it stays at the release the cutoff chose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update_excluded_specs.ts: the name-only patterns of the selectors and the locked crates they fit
//! ```

/// What:
/// `use crate::lockfile::parse_registry_versions;` imports `parse_registry_versions` from another module of this crate so this file can name it unqualified.
/// Why:
/// The exemption is read off the lockfile's text,
/// and this parser yields the registry crate name and version of every entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parse_registry_versions } from 'crate/lockfile';
/// ```
use crate::lockfile::parse_registry_versions;

/// What:
/// `use crate::selectors::{matches_pattern, parse_selector};` imports `matches_pattern`,
/// `parse_selector` from another module of this crate so this file can name them unqualified.
/// Why:
/// `parse_selector` splits a selector into its pattern and the versions it names;
/// `matches_pattern` tells whether a crate name fits one of those patterns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { matches_pattern, parse_selector } from 'crate/selectors';
/// ```
use crate::selectors::{
    matches_pattern,
    parse_selector,
};

/// What:
/// `fn matches_any_pattern(patterns, name) -> bool` tells whether a crate name fits at least one name pattern.
/// Why:
/// A crate that an exemption pattern names is updated on its own,
/// outside the release-age cutoff.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const matchesAnyPattern = (patterns: string[], name: string) => patterns.some((p) => matchesPattern(p, name));
/// ```
fn matches_any_pattern(
    patterns: &[String],
    name: &str,
) -> bool {
    for pattern in patterns {
        if matches_pattern(pattern, name) {
            return true;
        }
    }
    return false;
}

/// What:
/// `pub fn excluded_update_specs(lock: &str, selectors: &[String]) -> Result<Vec<String>, String>` lists `name@version` for
/// each locked registry crate that a name-only selector exempts from the release age.
/// Why:
/// Those crates move to their newest release with `cargo update`;
/// a selector naming exact versions only exempts those
/// versions from the checks and does not move the crate,
/// because the cutoff already chose its version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function excludedUpdateSpecs({ lock, selectors }): string[]
/// ```
pub fn excluded_update_specs(
    lock: &str,
    selectors: &[String],
) -> Result<Vec<String>, String> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      `excluded_update_specs` collects the name-only patterns one selector at a time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `patterns: Vec<String>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      No pattern exists until a selector is read, and a catalog without name-only selectors yields none.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      Patterns are copied out of the selectors into owned text so the returned list owns what it holds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The pattern list is only read afterwards, so a `Vec` owned by this function is enough.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut patterns: Vec<String> = Vec::new();
    for selector in selectors {
        let parsed = parse_selector(selector);
        if parsed.versions.is_empty() {
            patterns.push(parsed.pattern);
        }
    }
    if patterns.is_empty() {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        return Ok(Vec::new());
    }
    // `let mut`: a binding that may change.
    // Typed binding: `specs: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut specs: Vec<String> = Vec::new();
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    for locked in parse_registry_versions(lock)? {
        // Closure `|pattern|`: an anonymous function, like a TS arrow function.
        // Borrow `&locked.name`: lends the value without copying or giving it away.
        // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
        // Why:      Matching each locked crate against every pattern reads the patterns without consuming them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const item of list) { ... }  // or list.map(...)
        // ```
        if matches_any_pattern(&patterns, &locked.name) {
            specs.push(locked.key());
        }
    }
    specs.sort();
    specs.dedup();
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(specs);
}
