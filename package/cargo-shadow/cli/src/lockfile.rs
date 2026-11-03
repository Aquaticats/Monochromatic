//! What:
//! Reads the registry packages out of `Cargo.lock` text, and finds which `name@version` pairs one set of lockfiles holds
//! that another does not.
//! Why:
//! Only a version no lockfile held before needs the release-age and trust checks, so one run costs one registry lookup per
//! moved crate instead of one per locked crate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lockfile.ts
//! export function registryVersions(sources: string[]): LockedVersion[]
//! export function addedVersions({ before, after }): LockedVersion[]
//! ```

use std::collections::BTreeSet;

use toml_edit::DocumentMut;

/// The start of a `source` value for a crates.io-style registry package.
const REGISTRY_SOURCE_PREFIX: &str = "registry+";

/// One registry crate version.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LockedVersion {
    /// The crate name.
    pub name: String,
    /// The exact locked version.
    pub version: String,
}

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
/// Workspace members and path dependencies have no source; git dependencies have a `git+` source. Neither goes through the
/// registry's release-age and trust rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseRegistryVersions(source: string): LockedVersion[]  // throws on invalid TOML
/// ```
pub fn parse_registry_versions(source: &str) -> Result<Vec<LockedVersion>, String> {
    let document: DocumentMut = source.parse::<DocumentMut>().map_err(|error| { return error.to_string(); })?;
    let mut versions: Vec<LockedVersion> = Vec::new();
    let packages = match document.get("package").and_then(|item| { return item.as_array_of_tables(); }) {
        Some(found) => found,
        None => return Ok(versions),
    };
    for package in packages.iter() {
        let name: Option<&str> = package.get("name").and_then(|item| { return item.as_str(); });
        let version: Option<&str> = package.get("version").and_then(|item| { return item.as_str(); });
        let source_text: &str = package.get("source").and_then(|item| { return item.as_str(); }).unwrap_or("");
        if let (Some(found_name), Some(found_version)) = (name, version)
            && source_text.starts_with(REGISTRY_SOURCE_PREFIX)
        {
            versions.push(LockedVersion { name: found_name.to_string(), version: found_version.to_string() });
        }
    }
    return Ok(versions);
}

/// The distinct registry versions of several lockfiles, sorted by key.
pub fn registry_versions(sources: &[String]) -> Result<Vec<LockedVersion>, String> {
    let mut unique: BTreeSet<LockedVersion> = BTreeSet::new();
    for source in sources {
        for version in parse_registry_versions(source)? {
            unique.insert(version);
        }
    }
    let mut sorted: Vec<LockedVersion> = unique.into_iter().collect();
    sorted.sort_by_key(|version| { return version.key(); });
    return Ok(sorted);
}

/// What:
/// `pub fn added_versions(before: &[String], after: &[String]) -> Result<Vec<LockedVersion>, String>` returns the registry
/// versions present after that no lockfile held before.
/// Why:
/// A version some other lockfile already held passed the policy when it arrived; it costs no registry request now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addedVersions({ before, after }): LockedVersion[]
/// ```
pub fn added_versions(before: &[String], after: &[String]) -> Result<Vec<LockedVersion>, String> {
    let previous: BTreeSet<String> = registry_versions(before)?.iter().map(|version| { return version.key(); }).collect();
    let mut added: Vec<LockedVersion> = Vec::new();
    for version in registry_versions(after)? {
        if !previous.contains(&version.key()) {
            added.push(version);
        }
    }
    return Ok(added);
}
