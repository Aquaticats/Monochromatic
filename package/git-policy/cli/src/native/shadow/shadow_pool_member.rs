//! What:
//!  What a finished shadow in the pool must be for this invocation to take it:
//!  exactly what a creation would have made,
//! or a refusal that names what differs.
//! Why:
//!  A commit takes a shadow another process made,
//! so the one new risk is trusting a tree made earlier.
//!  That risk has one answer a test can hold:
//! a member is either the tree a creation makes,
//! or it is refused
//! (`doc/handover/questions-for-the-owner.md`,
//! "May a commit take its private repository from a pool made ahead?").
//!  Everything here is a read:
//! the comparison costs no directory-entry creation on the volume a commit already waits on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = await resizeMember(template, member); // undefined when the member is what a creation makes
//! ```

/// The creation's own `HEAD`,
/// before any commit names its branch.
use crate::shadow_skeleton::PLACEHOLDER_HEAD;
/// The creation's skeleton,
/// which a member must hold whole.
use crate::shadow_skeleton::SKELETON_DIRECTORIES;
/// The facts a creation is made from.
use crate::shadow_create::ShadowTemplate;
/// The entries a creation links,
/// and the per-worktree file it copies.
use crate::shadow_links::{has_worktree_config, shared_entries_of};
/// The directories of a shadow that stay the creation's own.
use std::path::Path;

/// What:
///  The entries of a member that the skeleton and the creation always make,
/// which no link and no copy stands in for.
/// Why:
///  A member holds these plus the shared entries and,
/// for a linked worktree,
/// one copied file,
/// and anything else is a leftover of a use that would be trusted by the next commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OWN_ENTRIES = ['HEAD', 'config', 'objects', 'refs'];
/// ```
const OWN_ENTRIES: [&str; 4] = ["HEAD", "config", "objects", "refs"];

/// What:
///  The per-worktree file a creation copies into a shadow.
/// Why:
///   It is the one entry that is neither the skeleton's nor a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WORKTREE_CONFIG = 'config.worktree';
/// ```
const WORKTREE_CONFIG: &str = "config.worktree";

/// What:
///  Why this member is not what a creation would have made,
/// or nothing when it is.
/// Why:
///   The caller refuses a named member and removes it,
/// so a member made for another worktree,
/// another object directory or another release never reaches a commit.
///  A reftable repository is never pooled at all,
/// so the entries Git's own `init` adds there are not considered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await refusal(template, member); // 'its config is not the one this invocation would write' | undefined
/// ```
pub fn refusal(template: &ShadowTemplate, member: &Path) -> std::io::Result<Option<String>> {
    if read_text(member.join("config").as_path())? != template.config_text() {
        return Ok(Some(String::from("its config is not the one this invocation would write")));
    }
    if read_text(member.join("objects").join("info").join("alternates").as_path())? != template.alternates_text() {
        return Ok(Some(String::from("its alternates do not name this invocation's object directory")));
    }
    if read_text(member.join("HEAD").as_path())? != PLACEHOLDER_HEAD {
        return Ok(Some(String::from("its HEAD is not the one a creation leaves before a commit names its branch")));
    }
    for relative in SKELETON_DIRECTORIES {
        let path: std::path::PathBuf = member.join(relative);
        let Ok(metadata) = std::fs::symlink_metadata(path.as_path()) else {
            return Ok(Some(format!("the skeleton directory {relative} is not a directory")));
        };
        if !metadata.is_dir() {
            return Ok(Some(format!("the skeleton directory {relative} is not a directory")));
        }
    }
    let (shared, keeps_its_own_info) = shared_entries_of(template.common_dir.as_path(), template.git_dir.as_path())?;
    if keeps_its_own_info {
        return Ok(Some(String::from("this worktree keeps its own info directory, so its shadows are made where the commit is")));
    }
    let copies_worktree_config: bool = has_worktree_config(template.git_dir.as_path())?;
    // Every entry of the member must be one this invocation's creation makes.
    for listed in std::fs::read_dir(member)? {
        let entry: std::fs::DirEntry = listed?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let own: bool = OWN_ENTRIES.contains(&name.as_str()) || (copies_worktree_config && name == WORKTREE_CONFIG);
        let mut linked: bool = false;
        for candidate in &shared {
            if candidate.name.to_string_lossy() == name {
                linked = true;
                break;
            }
        }
        if !own && !linked {
            return Ok(Some(format!("it holds {name}, which this invocation's creation does not make")));
        }
    }
    // Every entry the creation shares is here, and links to the same real entry.
    for entry in &shared {
        let link: std::path::PathBuf = member.join(&entry.name);
        let shared_named: String = entry.name.to_string_lossy().into_owned();
        let Ok(metadata) = std::fs::symlink_metadata(link.as_path()) else {
            return Ok(Some(format!("the shared entry {shared_named} is not in the member")));
        };
        if !metadata.file_type().is_symlink() {
            return Ok(Some(format!("the shared entry {shared_named} is not a link in the member")));
        }
        if std::fs::read_link(link.as_path())? != entry.source {
            return Ok(Some(format!("the shared entry {shared_named} links somewhere other than the repository's own entry")));
        }
    }
    if copies_worktree_config
        && read_text(member.join(WORKTREE_CONFIG).as_path())?
            != read_text(template.git_dir.join(WORKTREE_CONFIG).as_path())?
    {
        return Ok(Some(String::from("its copy of the worktree's configuration is not the one a creation would copy")));
    }
    // Nothing of an earlier commit's work survives in a member.
    if !is_empty(member.join("objects").join("pack").as_path())? {
        return Ok(Some(String::from("it holds a pack, which no creation leaves")));
    }
    if !is_empty(member.join("refs").join("heads").as_path())?
        || !is_empty(member.join("refs").join("tags").as_path())?
    {
        return Ok(Some(String::from("it holds a ref, which no creation leaves")));
    }
    return Ok(None);
}

/// What:
///  A file's bytes as text,
///  and nothing for one that is not there.
/// Why:
///   A missing entry is a difference the caller refuses,
/// not an error it has to tell from a real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readText(path) // '' when absent
/// ```
fn read_text(path: &Path) -> std::io::Result<String> {
    return match std::fs::read(path) {
        Ok(bytes) => Ok(String::from_utf8_lossy(bytes.as_slice()).into_owned()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error),
    };
}

/// What:
///  Whether a directory holds no entry.
/// Why:
///  A pack and a ref are what a used shadow holds;
/// a directory of them is what a member never has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await isEmpty(path)
/// ```
fn is_empty(path: &Path) -> std::io::Result<bool> {
    return match std::fs::read_dir(path) {
        Ok(mut entries) => Ok(entries.next().is_none()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(error),
    };
}

/// Member checks stay out of the release executable.
#[cfg(test)]
#[path = "shadow_pool_member_tests.rs"]
mod tests;
