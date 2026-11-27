//! What:
//! The cargo versions and release-archive tools the known locks name:
//! what one lock's JSON text names,
//! and what the locks found for the repository name together with the ones a caller named.
//! Why:
//! Other worktrees may sit on a commit whose lock is in an older format,
//! so only the fields the frozen surface carries are read;
//! and the two lists of paths differ in what an absent file means,
//! which decides whether this deletion command refuses.
//! A caller imports from `prune_cargo` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function namedVersions(lockText: string, origin: string): Map<string, Set<string>>;
//! function namedByAll(discovered: string[], explicit: string[]): [Map<string, Set<string>>, Map<string, NamedArchive>];
//! ```

/// Imports the sorted collections a lock's rows are gathered into.
use std::collections::{
    BTreeMap,
    BTreeSet,
};
/// Imports `fs` from `std`.
use std::fs;
/// Imports `ErrorKind` from `std::io`.
use std::io::ErrorKind;
/// Imports the directory type a lock path is held in.
use std::path::PathBuf;

/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports the release-archive tools a lock names.
use super::super::published_archive::{
    NamedArchive,
    named_archives,
};
/// The crate name a lock entry installs:
/// `packageId`,
/// else the origin of a `cargo:<crate>@<range>` request,
/// else the id without its `cargo:` prefix.
fn crate_of_entry(
    id: &str,
    entry: &Value,
) -> String {
    if let Some(package) = entry.get("packageId").and_then(Value::as_str) {
        return package.to_string();
    }
    if let Some(request) = entry.get("request").and_then(Value::as_str)
        && let Some(rest) = request.strip_prefix("cargo:")
    {
        return rest.split('@').next().unwrap_or(rest).to_string();
    }
    return id.strip_prefix("cargo:").unwrap_or(id).to_string();
}

/// What:
/// The `cargo-binstall` crates and versions one lock names,
/// read from its JSON text without decoding the rest of the model.
/// Why:
/// Other worktrees may sit on a commit whose lock is in an older format;
/// only `tools.<id>.version` and `route` are needed,
/// and both are part of the lock's frozen surface.
/// A `cargo-binstall` row without a `version` text is not such a surface:
/// it is refused rather than read as naming nothing,
/// because naming nothing is what lets the prune delete the crate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function namedVersions(lockText: string): Map<string, Set<string>>  // throws when not a lock
/// ```
pub fn named_versions(
    lock_text: &str,
    origin: &str,
) -> Result<BTreeMap<String, BTreeSet<String>>, ToolchainError> {
    let lock: Value = match serde_json::from_str(lock_text) {
        Ok(value) => value,
        Err(error) => {
            return Err(ToolchainError::failure(format!(
                "{origin} is not JSON: {error}"
            )));
        }
    };
    let Some(tools) = lock.get("tools").and_then(Value::as_object) else {
        return Err(ToolchainError::failure(format!(
            "{origin} has no `tools` object"
        )));
    };
    let mut named: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (id, entry) in tools {
        if entry.get("route").and_then(Value::as_str) != Some("cargo-binstall") {
            continue;
        }
        let Some(version) = entry.get("version").and_then(Value::as_str) else {
            // A row this command cannot read would protect nothing, and this is a deletion command: refusing keeps
            // the crate it installs instead of reading the lock as if the row were not there.
            return Err(ToolchainError::failure(format!(
                "{origin} names a cargo-binstall tool without a version, so nothing was deleted; tools.{id} has no `version` text"
            )));
        };
        named
            .entry(crate_of_entry(id, entry))
            .or_default()
            .insert(version.to_string());
    }
    return Ok(named);
}

/// What:
/// The cargo versions and the release-archive tools every lock the two lists give names.
/// Why:
/// The two lists differ in what an absent file means.
/// A lock `git worktree list` reported may be gone by the time it is read (another run moved it on),
/// so a missing one names none.
/// A lock the caller named with `--lock` may not be missing:
/// reading it as naming nothing would let this deletion command delete the version it names,
/// so the refusal is raised where the file is read,
/// never by a check that runs before the read,
/// which a concurrent rename can outrun.
pub(crate) fn named_by_all(
    discovered: &[PathBuf],
    explicit: &[PathBuf],
) -> Result<
    (
        BTreeMap<String, BTreeSet<String>>,
        BTreeMap<String, NamedArchive>,
    ),
    ToolchainError,
> {
    let mut named: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut archives: BTreeMap<String, NamedArchive> = BTreeMap::new();
    let mut locks_to_read: Vec<(&PathBuf, bool)> =
        discovered.iter().map(|file| return (file, false)).collect();
    locks_to_read.extend(explicit.iter().map(|file| return (file, true)));
    for (file, named_by_the_caller) in locks_to_read {
        let text: String = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound && !named_by_the_caller => continue,
            Err(error) if named_by_the_caller => {
                return Err(ToolchainError::failure(format!(
                    "the --lock file {} cannot be read: {error}; nothing was deleted",
                    file.display()
                )));
            }
            Err(error) => {
                return Err(ToolchainError::failure(format!(
                    "cannot read the lock {}: {error}; nothing was deleted",
                    file.display()
                )));
            }
        };
        for (crate_name, versions) in named_versions(&text, &file.display().to_string())? {
            named.entry(crate_name).or_default().extend(versions);
        }
        for (id, archive) in named_archives(&text, &file.display().to_string())? {
            let slot: &mut NamedArchive = archives.entry(id).or_default();
            slot.versions.extend(archive.versions);
            slot.first_bin = archive.first_bin;
        }
    }
    return Ok((named, archives));
}
