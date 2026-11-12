//! What:
//!  The common Git directory a service drain looks at,
//! and whether it belongs to a repository with no main worktree:
//!  the directory a linked worktree's Git directory names in its `commondir` file,
//! and the answer `core.bare` gives for it.
//! Why:
//!   The drains of directories and the rounds that call them both need these answers
//! (`service_work.rs`,
//! `service_work_drains_awaiting.rs`),
//! and they belong to no single one of those callers:
//! one decides which directory holds a repository's records,
//! the other decides which instance of the repository may run them.
//!  They sat in `service_work_handlers_replanning_and_acceptance.rs`
//! until two branches' additions met there and put that file over `rust/max-lines`,
//! so they moved here whole,
//! their docs with them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! commonDirectoryOf(gitDir) ?? undefined; isBareCommonDirectory(commonDir)
//! ```

use crate::repo_read_config_text::{last_entry, parse_bool, parse_config_text};
use std::path::{Path, PathBuf};


/// What:
///  Whether a common Git directory belongs to a repository with no main worktree,
/// which `core.bare` in its own `config`,
/// or in its
///  `config.worktree` when `extensions.worktreeConfig` turns that file on,
/// says.
/// Why:
///   A bare repository has no instance whose Git directory is the common one,
/// so the instances of its linked worktrees are the
///  only ones that can run the records left there;
/// a repository with a main worktree has that instance,
/// which owns them.
///   Git moves `core.bare` into `config.worktree` when the extension is enabled (`git help config`,
/// "extensions.worktreeConfig"),
///  and reads that file after `config` only while the extension is on and the repository has a format version
///  (`repo_read_layout.rs`),
/// so the last `core.bare` of the two decides.
/// An unreadable or unparsable file,
/// or a spelling this
///  reader leaves to Git,
/// reads as not bare,
/// and the records then stay for the main worktree's instance,
/// which is the safe side
///  of a wrong answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isBareCommonDirectory(commonDir)
/// ```
pub(crate) fn is_bare_common_directory(common: &Path) -> bool {
    let Ok(text) = std::fs::read(common.join("config")) else {
        return false;
    };
    let Some(mut entries) = parse_config_text(text.as_slice()) else {
        return false;
    };
    let worktree_config: bool = matches!(last_entry(entries.as_slice(), "extensions.worktreeconfig"), Some(entry) if parse_bool(entry.value.as_deref()) == Some(true))
        && last_entry(entries.as_slice(), "core.repositoryformatversion").is_some();
    if worktree_config {
        // A missing file is an empty file; one that is there and cannot be read or parsed leaves the answer to Git.
        match std::fs::read(common.join("config.worktree")) {
            Ok(bytes) => match parse_config_text(bytes.as_slice()) {
                Some(more) => entries.extend(more),
                None => return false,
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return false,
        }
    }
    return matches!(last_entry(entries.as_slice(), "core.bare"), Some(entry) if parse_bool(entry.value.as_deref()) == Some(true));
}

/// What:
///  The common directory a linked worktree's Git directory names in its `commondir` file,
/// or nothing for any other directory.
/// Why:
///   The drains of directories and the rounds that call them both need this answer
/// (`service_work.rs`,
/// `service_work_drains_awaiting.rs`),
/// so it sits in the file that uses neither.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// commonDirectoryOf(gitDir) ?? undefined
/// ```
pub(crate) fn common_directory_of(git_dir: &Path) -> Option<PathBuf> {
    let text: String = std::fs::read_to_string(git_dir.join("commondir")).ok()?;
    let named: &Path = Path::new(text.trim_end_matches(['\n', '\r']));
    if named.as_os_str().is_empty() {
        return None;
    }
    // Resolved by the text alone, so the result spells the path the way the worktree's own Git directory does, which is how the
    //  records name their directory.
    let mut resolved: PathBuf = PathBuf::new();
    for part in git_dir.join(named).components() {
        match part {
            std::path::Component::ParentDir => {
                if !resolved.pop() {
                    return None;
                }
            }
            std::path::Component::CurDir => {}
            other => resolved.push(other.as_os_str()),
        }
    }
    return Some(resolved);
}
