//! What:
//!  The repository format keys a layout read needs:
//!  the object format,
//! whether `extensions.worktreeConfig` is on,
//!  and the configuration entries once the last `core.bare` has settled.
//! Why:
//!  The plainness of a layout and the length of every object name come from these keys,
//! so they are read here once and named for the readers that need them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const format = readFormat(entries); // { objectFormat, worktreeConfig }
//! ```

use crate::repo_read_config_scan::read_following;
use crate::repo_read_config_text::{
    ConfigEntry,
    last_entry,
    parse_bool,
    parse_config_text,
};
use crate::repo_read_object_format::ObjectFormat;
use std::path::Path;

/// The per-worktree configuration file in a worktree's own Git directory.
pub(crate) const WORKTREE_CONFIG_NAME: &str = "config.worktree";

/// What:
///  The repository format keys the layout needs from the repository's own configuration file:
///  the object format,
/// and whether `extensions.worktreeConfig` turns the per-worktree file on.
/// Why:
///   The object format decides the length of a name;
/// any other extension may change how the repository is read.
///  Git respects `extensions.worktreeConfig` at every `core.repositoryFormatVersion`
///  (`Documentation/config/extensions.adoc`,
/// `handle_extension_v0` in `setup.c`);
/// the last entry decides,
/// as a boolean.
///  A repository whose configuration names no `core.repositoryformatversion` has no format at all to Git,
/// so the
///  extension is off there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const format: { objectFormat: ObjectFormat; worktreeConfig: boolean } | undefined = readFormat(entries); // undefined: ask Git
/// ```
pub(crate) fn format_of(entries: &[ConfigEntry]) -> Option<(ObjectFormat, bool)> {
    // A bare `core.repositoryformatversion` is an error in Git, not an absent key.
    let version: Option<&[u8]> = match last_entry(entries, "core.repositoryformatversion") {
        Some(entry) => Some(entry.value.as_deref()?),
        None => None,
    };
    let version_one: bool = match version {
        None | Some(b"0") => false,
        Some(b"1") => true,
        Some(_) => return None,
    };
    let mut format: ObjectFormat = ObjectFormat::Sha1;
    let mut worktree_config: bool = false;
    for entry in entries {
        if !entry.key.starts_with("extensions.") {
            continue;
        }
        if entry.key == "extensions.worktreeconfig" {
            worktree_config = parse_bool(entry.value.as_deref())?;
            continue;
        }
        if !version_one {
            return None;
        }
        let value: &[u8] = entry.value.as_deref().unwrap_or_default();
        match (entry.key.as_str(), value) {
            ("extensions.noop", _) => {}
            ("extensions.preciousobjects", _) if parse_bool(entry.value.as_deref()).is_some() => {}
            ("extensions.objectformat", b"sha1") => format = ObjectFormat::Sha1,
            ("extensions.objectformat", b"sha256") => format = ObjectFormat::Sha256,
            ("extensions.refstorage", b"files") => {}
            _ => return None,
        }
    }
    // Git clears the whole format when no version was found (`read_repository_format` in `setup.c`), the extension with
    // it, so an absent version never turns the per-worktree file on.
    return Some((format, worktree_config && version.is_some()));
}

/// What:
///  The entries of the worktree's own `config.worktree`,
/// or none when the extension is off or the file is absent;
///  `None` when the file cannot be read or parsed.
/// Why:
///   Git reads `<git dir>/config.worktree` (the worktree's own Git directory,
/// not the common one) after the common
///  file,
/// and only when `extensions.worktreeConfig` is on;
/// a missing file is an empty file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entries = enabled ? parse(await readIfExists(join(gitDir, 'config.worktree'))) : []; // undefined: ask Git
/// ```
pub(crate) fn worktree_entries(
    git_dir: &Path,
    enabled: bool,
) -> Option<Vec<ConfigEntry>> {
    if !enabled {
        return Some(Vec::new());
    }
    return match read_following(git_dir.join(WORKTREE_CONFIG_NAME).as_path())? {
        Some(bytes) => parse_config_text(bytes.as_slice()),
        None => Some(Vec::new()),
    };
}

/// What:
///  The entries with every direct `core.bare` removed when the last one leaves the repository non-bare;
///  `None` when it makes the repository bare or any of them is not a boolean.
/// Why:
///   Git takes the last `core.bare` of the common file then the per-worktree file (`read_worktree_config` in
///  `setup.c`);
/// a bare repository,
/// an unreadable boolean and a `core.worktree` are left to Git by the key scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = lastBare(files) === true ? undefined : files.map((entries) => entries.filter((e) => e.key !== 'core.bare'));
/// ```
pub(crate) fn without_false_bare(files: [&[ConfigEntry]; 2]) -> Option<[Vec<ConfigEntry>; 2]> {
    let mut last: Option<bool> = None;
    for entries in files {
        for entry in entries {
            if entry.key == "core.bare" {
                last = Some(parse_bool(entry.value.as_deref())?);
            }
        }
    }
    if last == Some(true) {
        return None;
    }
    let [common, per_worktree] = files;
    return Some([without_bare(common), without_bare(per_worktree)]);
}

/// The entries without `core.bare`,
/// in their order.
fn without_bare(entries: &[ConfigEntry]) -> Vec<ConfigEntry> {
    let mut kept: Vec<ConfigEntry> = Vec::new();
    for entry in entries {
        if entry.key != "core.bare" {
            kept.push(entry.clone());
        }
    }
    return kept;
}
