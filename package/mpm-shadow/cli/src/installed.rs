//! What:     Reads cargo's install record (`<cargo home>/.crates2.json`) to learn which version of each crate is
//!           installed right now.
//! Why:      The installed version is the one that was just provisioned, so every other cached version of that crate is
//!           stale. Reading cargo's own record needs no knowledge of this repository's tool plan.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // installed.ts
//! export function installedVersions(cargoHome: string): Map<string, string> { ... }
//! ```

/// What:     `use std::collections::BTreeMap;` imports a key-value map that keeps its keys sorted (sibling `HashMap`
///           has no order).
/// Why:      A sorted map gives the same output order on every run, which keeps messages and tests stable.
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

/// What:     `pub fn installed_versions(cargo_home: &Path) -> BTreeMap<String, String>` declares a public function that
///           borrows the cargo home and returns an owned map from crate name to installed version.
/// Why:      A missing or unreadable record returns an empty map: the prune then falls back to the newest cached
///           version, which is safe. `BTreeMap<String, String>` and not `HashMap` for the stable order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installedVersions(cargoHome: string): Map<string, string>
/// ```
pub fn installed_versions(cargo_home: &Path) -> BTreeMap<String, String> {
    let mut versions: BTreeMap<String, String> = BTreeMap::new();

    // What:     `fs::read_to_string(...)` reads a whole file as text and returns a `Result` (`Ok(text)` or
    //           `Err(error)`). `.join(".crates2.json")` appends the file name to the path. `let Ok(text) = ... else
    //           { ... };` binds `text` on success and runs the `else` block on failure; that block must leave the
    //           function (here with `return`).
    // Why:      No record means nothing is known as installed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let text; try { text = fs.readFileSync(path.join(cargoHome, '.crates2.json'), 'utf8'); } catch { return versions; }
    // ```
    let Ok(text) = fs::read_to_string(cargo_home.join(".crates2.json")) else {
        return versions;
    };

    // What:     `serde_json::from_str::<serde_json::Value>(&text)` parses JSON text into a loosely typed value tree
    //           (like `JSON.parse`); `::<serde_json::Value>` names the result type. `&text` lends the text.
    // Why:      The record is JSON whose shape cargo owns; a loose tree reads only the part we need.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let document; try { document = JSON.parse(text); } catch { return versions; }
    // ```
    let Ok(document) = serde_json::from_str::<serde_json::Value>(&text) else {
        return versions;
    };

    // What:     `document.get("installs")` looks up a key and returns an `Option`. `.and_then(|value| ...)` continues
    //           only when the key exists; the `|value| ...` is a closure (Rust's `(value) => ...` arrow function) that
    //           converts the value to a JSON object (also an `Option`). `let Some(installs) = ... else { return ... };`
    //           unwraps it or leaves the function.
    // Why:      Each key under `installs` reads like `"monochromatic-lint 0.4.0 (registry+https://...)"`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const installs = document.installs; if (typeof installs !== 'object' || installs === null) return versions;
    // ```
    let Some(installs) = document.get("installs").and_then(|value| return value.as_object()) else {
        return versions;
    };

    for key in installs.keys() {
        // What:     `key.split_whitespace()` yields the key's space-separated words lazily; `.next()` takes the next
        //           one as an `Option`. Two calls take the name then the version.
        // Why:      The first word is the crate name and the second is its version.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [name, version] = key.split(/\s+/);
        // ```
        let mut words = key.split_whitespace();
        if let Some(name) = words.next()
            && let Some(version) = words.next()
        {
            versions.insert(name.to_string(), version.to_string());
        }
    }
    return versions;
}
