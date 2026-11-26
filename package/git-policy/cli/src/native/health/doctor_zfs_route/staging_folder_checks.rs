//! What:
//!  The staging folders a removal leaves under the account's cache,
//!  and the propagation of the private bind each one gets.
//! Why:
//!  A removal moves the clone to a staging folder under `~/.cache` and unmounts
//!  it from there,
//!  so a folder another account could write,
//!  one holding content,
//!  or one whose name no removal derives is a state a person must see,
//!  and the bind must stay private or a mount made on it reaches another
//!  namespace
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staging target and who creates it" and
//!  "The private bind";
//!  plan step 18,
//!  "the staging folder and its live bind's propagation").
//!  The account is trusted with its own cache by the accepted threat model,
//!  so these are reports of what is there rather than refusals,
//!  and each fact is handed in so a control reads no cache.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = check_staging_folder(facts); // pass | finding('owner'|'mode'|'name'|'content') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use super::handler_checks::{
    PathFacts,
    PathState,
};
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_unmount::staging_folder_name::{
    STAGING_FOLDER_PREFIX,
    guid_of_folder_name,
};

/// The mode every staging folder is made with,
/// owned by the account.
pub const STAGING_FOLDER_MODE: u32 = 0o700;

/// The optional fields the kernel writes for a mount shared with another
/// namespace.
pub const SHARED_FIELD_PREFIX: &str = "shared:";

/// The optional field the kernel writes for a slave mount.
pub const MASTER_FIELD_PREFIX: &str = "master:";

/// What:
///  One folder under the cache whose name begins with the staging prefix.
/// Why:
///  A name that carries the prefix but no derived component is a folder no
///  removal made,
///  and it must be reported rather than removed,
///  so the name's own derivation is part of the fact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedFolder = { name: string; state: PathState; derived: boolean; empty: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedFolder {
    /// The folder's name under the cache.
    pub name: String,
    /// What the look at it answered.
    pub state: PathState,
    /// Whether the name away from the prefix is a dataset GUID.
    pub derived: bool,
    /// Whether the folder holds no entry.
    pub empty: bool,
}

/// What:
///  The cache folder and every staging-shaped name under it.
/// Why:
///  One value carries the account,
///  the cache and the folders,
///  so a report cannot compare one account's folder with another's record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingFolderFacts = { cache: PathFacts; account: number; folders: StagedFolder[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagingFolderFacts {
    /// The account's cache folder.
    pub cache: PathFacts,
    /// The managed account's user id.
    pub account: u32,
    /// Every name under the cache that begins with the staging prefix.
    pub folders: Vec<StagedFolder>,
}

/// What:
///  Whether every staging folder under the cache is one a removal made,
///  owned by the account,
///  at the design's mode,
///  and empty.
/// Why:
///  A folder holding content is what the design calls the account's
///  substitution,
///  which only a report can surface,
///  and a folder at another owner or mode was not made by the wrapper this
///  account runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_staging_folder(facts); // pass | finding('owner'|'mode'|'name'|'content') | unknown
/// ```
pub fn check_staging_folder(facts: &StagingFolderFacts) -> CheckOutcome {
    let cache: String = facts.cache.path.display().to_string();
    match &facts.cache.state {
        PathState::Absent => {
            return pass(format!(
                "{cache} is not there, so no staging folder stands under it"
            ));
        }
        PathState::Unreadable {
            why,
        } => {
            return unknown(format!(
                "{cache} could not be listed ({why}), so no staging folder is proved absent"
            ));
        }
        PathState::WrongKind {
            found,
        } => return finding("cache", format!("{cache} is not a directory ({found})")),
        PathState::File {
            ..
        } => {
            return finding(
                "cache",
                format!("{cache} is a file rather than the account's cache folder"),
            );
        }
        PathState::Directory {
            ..
        } => {}
    }
    if facts.folders.is_empty() {
        return pass(format!("no staging folder stands under {cache}"));
    }
    let mut empty_folders: usize = 0;
    for folder in &facts.folders {
        match &folder.state {
            PathState::Unreadable {
                why,
            } => {
                return unknown(format!(
                    "{cache}/{} could not be looked at ({why}), so its ownership and content are unproved",
                    folder.name
                ));
            }
            PathState::Directory {
                owner,
                mode,
            } => {
                if !folder.derived {
                    return finding(
                        "name",
                        format!(
                            "{cache}/{} begins with `{STAGING_FOLDER_PREFIX}` and the rest of the name is no dataset GUID, so no removal derived it; it is reported and left in place",
                            folder.name
                        ),
                    );
                }
                if *owner != facts.account {
                    return finding(
                        "owner",
                        format!(
                            "{cache}/{} is owned by user {owner} and a staging folder is the managed account's ({})",
                            folder.name, facts.account
                        ),
                    );
                }
                if *mode != STAGING_FOLDER_MODE {
                    return finding(
                        "mode",
                        format!(
                            "{cache}/{} is mode {mode:04o} and the design makes a staging folder {STAGING_FOLDER_MODE:04o}",
                            folder.name
                        ),
                    );
                }
                if !folder.empty {
                    return finding(
                        "content",
                        format!(
                            "{cache}/{} holds an entry, and a staging folder is empty before the clone moves onto it; a person looks at what stands there before any removal removes it",
                            folder.name
                        ),
                    );
                }
                empty_folders += 1;
            }
            PathState::Absent => {
                return finding(
                    "folder",
                    format!(
                        "{cache}/{} is gone between the listing and the look",
                        folder.name
                    ),
                );
            }
            PathState::WrongKind {
                found,
            } => {
                return finding(
                    "folder",
                    format!(
                        "{cache}/{} is no staging folder: it is {found}",
                        folder.name
                    ),
                );
            }
            PathState::File {
                ..
            } => {
                return finding(
                    "folder",
                    format!("{cache}/{} is a file rather than a folder", folder.name),
                );
            }
        }
    }
    return pass(format!(
        "{empty_folders} staging folder(s) under {cache} are the account's, mode {STAGING_FOLDER_MODE:04o} and empty"
    ));
}

/// What:
///  One live mount under the cache whose name begins with the staging prefix.
/// Why:
///  The design binds the staging folder to itself and makes the bind private
///  before the clone moves onto it,
///  so the bind's source and its propagation are both part of the report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagingBind = { mountPoint: string; source: string; shared: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagingBind {
    /// Where the bind stands.
    pub mount_point: std::path::PathBuf,
    /// The source the kernel lists.
    pub source: String,
    /// The optional fields that say the mount is not private.
    pub sharing: Vec<String>,
}

/// What:
///  Every live mount under the cache whose name begins with the staging
///  prefix.
/// Why:
///  A staging bind is the one mount a removal leaves behind when it stops,
///  and the clone it carried is a second row on the same folder,
///  so the rows under the prefix are the set a person checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const binds = staging_binds(rows, cache); // rows whose mount point is `<cache>/monochromatic-git-policy-cli-staging-...`
/// ```
pub fn staging_binds(
    rows: &[MountFacts],
    cache: &std::path::Path,
) -> Vec<StagingBind> {
    let mut binds: Vec<StagingBind> = Vec::new();
    for row in rows {
        if !row.mount_point.starts_with(cache) || row.mount_point == cache {
            continue;
        }
        let Some(name) = row
            .mount_point
            .file_name()
            .and_then(|name| return name.to_str())
        else {
            continue;
        };
        if !name.starts_with(STAGING_FOLDER_PREFIX) {
            continue;
        }
        let mut sharing: Vec<String> = Vec::new();
        for field in &row.optional_fields {
            if field.starts_with(SHARED_FIELD_PREFIX) || field.starts_with(MASTER_FIELD_PREFIX) {
                sharing.push(field.clone());
            }
        }
        binds.push(StagingBind {
            mount_point: row.mount_point.clone(),
            source: row.source.clone(),
            sharing,
        });
    }
    return binds;
}

/// What:
///  Whether each live staging bind is private.
/// Why:
///  A shared bind delivers a mount made on it to another namespace,
///  which is what the guest measured as the cost of the shared arm,
///  and propagation belongs to the mount instance,
///  so the live row decides.
///  The row's source is not judged here:
///  the kernel prints a bind's source as the filesystem's own source, never as
///  the folder it was bound from,
///  so only the path and the optional fields say what the bind is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_staging_bind_propagation(binds); // pass | finding('propagation')
/// ```
pub fn check_staging_bind_propagation(binds: &[StagingBind]) -> CheckOutcome {
    if binds.is_empty() {
        return pass(String::from("no staging bind is live"));
    }
    for bind in binds {
        let path: String = bind.mount_point.display().to_string();
        if !bind.sharing.is_empty() {
            return finding(
                "propagation",
                format!(
                    "the staging bind at {path} is not private ({}), so a mount made on it reaches another namespace; set it back with `mount --make-private {path}`",
                    bind.sharing.join(", ")
                ),
            );
        }
    }
    return pass(format!(
        "{} staging bind(s) are private and bind their own folder",
        binds.len()
    ));
}

/// What:
///  Whether a name under the cache is one a removal derives.
/// Why:
///  The wrapper tells a folder it made from a stranger's by the component
///  after the prefix,
///  which is the clone's dataset GUID,
///  so the same rule answers whether a listing is a report or a removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const derived = is_derived_folder_name('monochromatic-git-policy-cli-staging-1234');
/// ```
pub fn is_derived_folder_name(name: &str) -> bool {
    return guid_of_folder_name(name).is_ok();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "staging_folder_checks_tests.rs"]
mod tests;
