//! What:
//! Reads cargo's install record (`<cargo home>/.crates.toml`) to learn which version of each crate is
//!           installed right now.
//! Why:
//! The installed version is the one that was just provisioned,
//! so every other cached version of that crate is
//!           stale.
//! `.crates.toml` is the file both `cargo install` and `cargo binstall` update;
//! `.crates2.json` is
//!           updated by cargo only,
//! so after a binstall it still names the previous version.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // installed.ts
//! export function installedVersions(cargoHome: string): Map<string, string> { ... }
//! ```

/// What:
/// `use std::collections::BTreeMap;` imports a key-value map that keeps its keys sorted (sibling `HashMap`
///           has no order).
/// Why:
/// A sorted map gives the same output order on every run,
/// which keeps messages and tests stable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // A TS Map keeps insertion order; a sorted map has no built-in TS counterpart.
/// ```
use std::collections::BTreeMap;

/// Imports the file-system module (reads the record).
use std::fs;

/// Imports the borrowed path type.
use std::path::Path;

/// What:
/// `pub fn installed_versions(cargo_home: &Path) -> BTreeMap<String, String>` declares a public function that
///           borrows the cargo home and returns an owned map from crate name to installed version.
/// Why:
/// A missing or unreadable record returns an empty map:
/// the prune then falls back to the newest cached
///           version,
/// which is safe.
/// `BTreeMap<String, String>` and not `HashMap` for the stable order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installedVersions(cargoHome: string): Map<string, string>
/// ```
pub fn installed_versions(cargo_home: &Path) -> BTreeMap<String, String> {
    let mut versions: BTreeMap<String, String> = BTreeMap::new();

    // What:     `fs::read_to_string(...)` reads a whole file as text and returns a `Result` (`Ok(text)` or
    //           `Err(error)`). `.join(".crates.toml")` appends the file name to the path. `let Ok(text) = ... else
    //           { ... };` binds `text` on success and runs the `else` block on failure; that block must leave the
    //           function (here with `return`).
    // Why:      No record means nothing is known as installed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let text; try { text = fs.readFileSync(path.join(cargoHome, '.crates.toml'), 'utf8'); } catch { return versions; }
    // ```
    let Ok(text) = fs::read_to_string(cargo_home.join(".crates.toml")) else {
        return versions;
    };

    for line in text.lines() {
        // What:     Each install line looks like `"monochromatic-lint 0.4.0 (registry+https://...)" = ["bin"]`.
        //           `line.strip_prefix('"')` removes a leading double quote and returns `Some(rest)`, or `None` when
        //           the line does not start with one (the `[v1]` header and blank lines). `let Some(rest) = ... else
        //           { continue; };` skips such lines. `rest.find('"')` returns `Some(index)` of the closing quote.
        // Why:      The quoted key holds the crate name, the version and the source, separated by spaces; it never
        //           contains a quote, so no TOML parser is needed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const match = line.match(/^"([^"]+)"/); if (match === null) continue;
        // ```
        let Some(rest) = line.strip_prefix('"') else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            continue;
        };

        // `rest[..end]` borrows the text before the closing quote; `split_whitespace()` yields its space-separated
        // words lazily and `.next()` takes the next one as an `Option`. The first word is the crate name and the
        // second is its version.
        let mut words = rest[..end].split_whitespace();
        if let Some(name) = words.next()
            && let Some(version) = words.next()
        {
            versions.insert(name.to_string(), version.to_string());
        }
    }
    return versions;
}
