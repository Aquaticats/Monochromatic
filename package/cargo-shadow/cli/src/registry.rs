//! What:
//! The crates.io lookups the policy needs:
//! the sparse index for publish times and yanked flags,
//! the web API for the full
//! version history and the owner list,
//! with the history cached on disk.
//! Why:
//! A published version never changes,
//! so a history that already lists every version being checked is read from
//! `.cache/cargo-shadow/crates-io/<name>.json` with no request.
//! The API allows about one request per second,
//! so requests
//! are spaced and an HTTP 429 is retried after its `Retry-After`,
//! or a doubling backoff,
//! at most five times.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // registry.ts
//! export type Registry = { pubtimes(name): Map<string, string>; yanked(name): Set<string>; history({ name, versions }): VersionRecord[]; owners(name): Set<string> };
//! export function createRegistry({ root, fetchText }): Registry
//! ```

/// What:
/// `use crate::http::RegistryError;` imports the failure of a registry lookup.
/// Why:
/// Every method of the trait answers with it or with the records it asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegistryError } from 'crate/http';
/// ```
use crate::http::RegistryError;
/// What:
/// `use crate::policy::VersionRecord;` imports the record of one published release that the trust rule reads.
/// Why:
/// The history answer of the trait is a list of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { VersionRecord } from 'crate/policy';
/// ```
use crate::policy::VersionRecord;
/// What:
/// `use std::collections::{BTreeMap, BTreeSet};` imports the sorted map and the sorted set (siblings:
/// `HashMap`/`HashSet`,
/// unordered and faster).
/// Why:
/// The trait's four answers are these two collections,
/// so their types appear in its method signatures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // new Map() / new Set()
/// ```
use std::collections::{
    BTreeMap,
    BTreeSet,
};

/// What:
/// `pub trait Registry { ... }` is what the checks ask of crates.io.
/// Why:
/// Tests give the checks a fake with canned histories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Registry { pubtimes(name): Map<string, string>; yanked(name): Set<string>; history(args): VersionRecord[]; owners(name): Set<string> }
/// ```
pub trait Registry {
    /// Publish time (RFC 3339) of each version of a crate,
    /// from the sparse index;
    /// empty when the index has none.
    fn pubtimes(
        &self,
        name: &str,
    ) -> Result<BTreeMap<String, String>, RegistryError>;

    /// The yanked versions of a crate.
    fn yanked(
        &self,
        name: &str,
    ) -> Result<BTreeSet<String>, RegistryError>;
    /// Every release of a crate;
    /// read from the cache when it lists all of `versions`.
    fn history(
        &self,
        name: &str,
        versions: &[String],
    ) -> Result<Vec<VersionRecord>, RegistryError>;
    /// The account ids of the crate's listed individual owners.
    fn owners(
        &self,
        name: &str,
    ) -> Result<BTreeSet<String>, RegistryError>;
    /// Warms the history cache for these crates before they are asked for one by one;
    /// a registry with nothing to warm does nothing.
    fn prefetch(
        &self,
        _names: &BTreeSet<String>,
    ) -> Result<(), RegistryError> {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        return Ok(());
    }
}

/// crates.io through a fetcher (split out of this module).
pub use crate::registry_crates_io::CratesIo;
/// The addresses and limits of the two crates.io services (split out of this module).
pub use crate::registry_crates_io::{
    API_MIN_INTERVAL,
    DATABASE_DUMP_URL,
    DUMP_BREAKEVEN_CRATES,
    HISTORY_CACHE_DIR,
};
/// The readers of crates.io's answers and of the history cache (split out of this module).
pub use crate::registry_record_reading::{
    parse_index_pubtimes,
    parse_index_yanked,
    parse_owner_ids,
    parse_version_history,
};

/// The sparse index path of a crate:
/// `1/a`,
/// `2/ab`,
/// `3/a/abc`,
/// `ab/cd/abcdef`.
pub fn sparse_index_path(name: &str) -> String {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let lower: String = name.to_lowercase();
    if lower.len() <= 2 {
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        return format!("{}/{lower}", lower.len());
    }
    if lower.len() == 3 {
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // Macro `format!`: builds or prints text (or a list) from a template.
        return format!("3/{}/{lower}", &lower[..1]);
    }
    // Borrow `&lower`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return format!("{}/{}/{lower}", &lower[..2], &lower[2..4]);
}
