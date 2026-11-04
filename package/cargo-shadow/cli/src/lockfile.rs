//! What:
//! Reads the registry packages out of `Cargo.lock` text,
//! and finds which `name@version` pairs one set of lockfiles holds
//! that another does not.
//! Why:
//! Only a version no lockfile held before needs the release-age and trust checks,
//! so one run costs one registry lookup per
//! moved crate instead of one per locked crate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lockfile.ts
//! export function registryVersions(sources: string[]): LockedVersion[]
//! export function addedVersions({ before, after }): LockedVersion[]
//! ```

/// What:
/// `use std::collections::BTreeSet;` imports `BTreeSet` from the standard library so this file can name it unqualified.
/// Why:
/// `BTreeSet` is used by `registry_versions`,
/// `added_versions`,
/// `direct_dependency_names`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BTreeSet } from 'std/collections';
/// ```
use std::collections::BTreeSet;

/// What:
/// `use toml_edit::DocumentMut;` imports `DocumentMut` from the external crate `toml_edit` so this file can name it unqualified.
/// Why:
/// `DocumentMut` is used by `parse_registry_versions`,
/// `direct_dependency_names`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DocumentMut } from 'toml_edit';
/// ```
use toml_edit::DocumentMut;
/// What:
/// `use crate::adapt::text_of;` imports `text_of` from another module of this crate so this file can name it unqualified.
/// Why:
/// `text_of` is used by `parse_registry_versions`,
/// `direct_dependency_names`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { text_of } from 'crate/adapt';
/// ```
use crate::adapt::text_of;
/// What:
/// `use toml_edit::Item;` imports `Item` from the external crate `toml_edit` so this file can name it unqualified.
/// Why:
/// `Item` is used by `parse_registry_versions`,
/// `direct_dependency_names`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Item } from 'toml_edit';
/// ```
use toml_edit::Item;

/// The start of a `source` value for a registry package of the index-style spelling.
const REGISTRY_SOURCE_PREFIX: &str = "registry+";

/// The start of a `source` value for a registry package of the sparse spelling.
const SPARSE_SOURCE_PREFIX: &str = "sparse+";

/// The words in the message about a registry other than crates.io;
/// a caller that must treat that rejection as a policy refusal,
/// not as a parse problem,
/// recognizes the message by them.
pub const UNSUPPORTED_REGISTRY_MARK: &str = "which the policy checks do not support";

/// What:
/// The `source` values that mean crates.io:
/// the index URL cargo writes for crates.io whichever protocol it used,
/// and the sparse index URL spelled out.
/// Why:
/// Every lookup of the policy checks goes to crates.io,
/// so a package may be checked only when its identity is crates.io;
/// the registry is not carried through the comparisons and caches,
/// so any other registry is refused by name instead of being approved by an unrelated crates.io entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CRATES_IO_SOURCES = ['registry+https://github.com/rust-lang/crates.io-index', 'sparse+https://index.crates.io/'] as const;
/// ```
const CRATES_IO_SOURCES: [&str; 2] = ["registry+https://github.com/rust-lang/crates.io-index", "sparse+https://index.crates.io/"];

/// One registry crate version.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LockedVersion {
    /// The crate name.
    pub name: String,
    /// The exact locked version.
    pub version: String,
}

/// What:
/// `impl LockedVersion { ... }` attaches `key()` to the struct.
/// Why:
/// Everything that compares or groups versions uses the same `name@version` text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LockedVersion { key(): string { return `${this.name}@${this.version}`; } }
/// ```
impl LockedVersion {
    /// The joined key such as `anyhow@1.0.104`.
    pub fn key(&self) -> String {
        return format!("{}@{}", self.name, self.version);
    }
}

/// What:
/// `pub fn parse_registry_versions(source: &str) -> Result<Vec<LockedVersion>, String>` returns every `[[package]]` whose
/// `source` starts with `registry+`.
/// Why:
/// Workspace members and path dependencies have no source;
/// git dependencies have a `git+` source.
/// Neither goes through the
/// registry's release-age and trust rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseRegistryVersions(source: string): LockedVersion[]  // throws on invalid TOML
/// ```
pub fn parse_registry_versions(source: &str) -> Result<Vec<LockedVersion>, String> {
    let document: DocumentMut = source.parse::<DocumentMut>().map_err(text_of)?;
    let mut versions: Vec<LockedVersion> = Vec::new();
    let packages = match document.get("package").and_then(Item::as_array_of_tables) {
        Some(found) => found,
        None => return Ok(versions),
    };
    for package in packages.iter() {
        let name: Option<&str> = package.get("name").and_then(Item::as_str);
        let version: Option<&str> = package.get("version").and_then(Item::as_str);
        let source_text: &str = package.get("source").and_then(Item::as_str).unwrap_or("");
        // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
        // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name: Type = value;
        // ```
        let from_registry: bool = source_text.starts_with(REGISTRY_SOURCE_PREFIX) || source_text.starts_with(SPARSE_SOURCE_PREFIX);
        if !from_registry {
            continue;
        }
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        if let (Some(found_name), Some(found_version)) = (name, version) {
            // Registry identity is not carried end to end, so a registry other than crates.io is refused explicitly.
            if !CRATES_IO_SOURCES.contains(&source_text) {
                // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
                // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // // Ok(value)  is  return value;
                // // Err(error) is  throw error;   (the caller sees it as a returned value)
                // ```
                // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
                // Why:      Text building and printing are written as one call instead of string concatenation.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // `..${x}..`   // template literal
                // console.error(text);   // eprintln!
                // [a, b]            // vec![a, b]
                // ```
                return Err(format!(
                    "{found_name}@{found_version} comes from the registry {source_text}, which the policy checks do not support: they ask crates.io only, so they would approve it by an unrelated crates.io release of the same name and version. Use crates.io, or a path or git dependency"
                ));
            }
            versions.push(LockedVersion { name: found_name.to_string(), version: found_version.to_string() });
        }
    }
    return Ok(versions);
}

/// The distinct registry versions of several lockfiles,
/// sorted by key.
pub fn registry_versions(sources: &[String]) -> Result<Vec<LockedVersion>, String> {
    let mut unique: BTreeSet<LockedVersion> = BTreeSet::new();
    for source in sources {
        for version in parse_registry_versions(source)? {
            unique.insert(version);
        }
    }
    let mut sorted: Vec<LockedVersion> = unique.into_iter().collect();
    sorted.sort_by_key(LockedVersion::key);
    return Ok(sorted);
}

/// What:
/// `pub fn added_versions(before: &[String], after: &[String]) -> Result<Vec<LockedVersion>, String>` returns the registry
/// versions present after that no lockfile held before.
/// Why:
/// A version some other lockfile already held passed the policy when it arrived;
/// it costs no registry request now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addedVersions({ before, after }): LockedVersion[]
/// ```
pub fn added_versions(before: &[String], after: &[String]) -> Result<Vec<LockedVersion>, String> {
    let previous: BTreeSet<String> = registry_versions(before)?.iter().map(LockedVersion::key).collect();
    let mut added: Vec<LockedVersion> = Vec::new();
    for version in registry_versions(after)? {
        if !previous.contains(&version.key()) {
            added.push(version);
        }
    }
    return Ok(added);
}

/// What:
/// `pub fn direct_dependency_names(source: &str) -> Result<BTreeSet<String>, String>` lists the names that a workspace member
/// or path package depends on directly,
/// read from the `dependencies` of every `[[package]]` without a `source`.
/// Why:
/// pnpm refuses `pnpm update <name>@<version>` for a package that is not a direct dependency
/// (`ERR_PNPM_UPDATE_VERSION_ON_INDIRECT_DEP`,
/// <https://pnpm.io/cli/update>);
/// the shadow applies the same rule to `--precise`.
/// A lock entry reads `name` or,
/// when two versions exist,
/// `name version` or `name version (source)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function directDependencyNames(source: string): Set<string>
/// ```
pub fn direct_dependency_names(source: &str) -> Result<BTreeSet<String>, String> {
    let document: DocumentMut = source.parse::<DocumentMut>().map_err(text_of)?;
    let mut names: BTreeSet<String> = BTreeSet::new();
    let packages = match document.get("package").and_then(Item::as_array_of_tables) {
        Some(found) => found,
        None => return Ok(names),
    };
    for package in packages.iter() {
        if package.get("source").is_some() {
            continue;
        }
        let dependencies = match package.get("dependencies").and_then(Item::as_array) {
            Some(found) => found,
            None => continue,
        };
        for entry in dependencies.iter() {
            if let Some(text) = entry.as_str()
                && let Some(name) = text.split(' ').next()
            {
                names.insert(name.to_string());
            }
        }
    }
    return Ok(names);
}
