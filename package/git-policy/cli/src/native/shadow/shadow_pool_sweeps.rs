//! What:
//!  Where a worktree's pool folder is named from,
//!  what proves a name on the pool's path is this wrapper's own,
//!  and the two sweeps:
//! the pool of a worktree Git no longer lists,
//! and the half-made members a killed fill left.
//! Why:
//!  A member's config names the worktree root,
//! so a linked worktree's members are its own,
//! and a sweep and a fill must name that folder by one rule,
//! or a sweep would leave the folder a commit takes from.
//!  A link or a file anywhere on the pool's path is nobody's pool:
//! a claim's `rename` and a sweep's `remove_tree` would land in whatever tree it names.
//!  Kept apart from the claim and the fill so all three stay under the file limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // poolKey(commonDir, gitDir); thePoolPathIsTheWrappersOwn(root); sweepGoneWorktrees(commonDir); sweepAbandoned(root, now)
//! ```

/// Removing a tree made of the wrapper's own entries.
use crate::private_storage::remove_tree;
/// What the pool's folder is called,
/// beside the repository scratch directory.
use crate::wrapper_names_old_and_new::SHADOW_POOL_FOLDER;
/// Paths and time.
use std::path::{
    Path,
    PathBuf,
};
use std::time::{
    Duration,
    SystemTime,
};

/// How old a half-made member must be before a fill removes it.
/// Measured:
/// a fill is about 40 calls and took 2 to 10 ms in the traces of
/// `doc/planning/cli-git-commit-latency-under-many-agents.md`
/// ("The creation is,
/// counted"),
/// and the volume has held one call for 64 s,
/// so this covers a fill that waits and leaves nothing a commit could take.
pub const FILLING_STALE_AFTER: Duration = Duration::from_secs(900);

/// The pool folder of the main checkout,
/// whose own Git directory is the common directory the pool lives in.
const MAIN_KEY: &str = "main";

/// The prefix of a linked worktree's pool folder,
/// followed by the name of its own Git directory.
const WORKTREE_KEY_PREFIX: &str = "worktree-";

/// The prefix of a member that is still being made.
pub(super) const FILLING_PREFIX: &str = "filling-";

/// What:
///  The folder name one worktree's pool has.
/// Why:
///  The sweep of a gone worktree's pool and the name a fill and a claim use must be one rule,
///  or a sweep would look at a folder nothing fills and leave the one a commit takes from.
///  The name is lossy text of the Git directory's own name,
///  so both sides compare the same text whatever the path's bytes are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// poolKey(commonDir, gitDir) // 'main' or 'worktree-<name>'
/// ```
pub(crate) fn pool_key(
    common_dir: &Path,
    git_dir: &Path,
) -> String {
    return match git_dir == common_dir {
        true => String::from(MAIN_KEY),
        false => format!(
            "{WORKTREE_KEY_PREFIX}{}",
            git_dir.file_name().unwrap_or_default().to_string_lossy()
        ),
    };
}

/// What:
///  Whether one name of the pool's path is a directory this wrapper made,
///  or is not there yet.
/// Why:
///   A name that is there and is a link or a file is not the wrapper's:
/// the wrapper makes its folders with `create_private_directory_all` and keeps its members inside them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theWrappersOwnDirectory(path) // false for a link and for a file, true for a directory and for nothing
/// ```
fn the_wrappers_own_directory(path: &Path) -> bool {
    return match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata.is_dir(),
        Err(error) => error.kind() == std::io::ErrorKind::NotFound,
    };
}

/// What:
///  Whether the two names the wrapper makes above its members are directories it made,
///  or are not there yet.
/// Why:
///   The pool's path is the pool's folder and the worktree's folder inside it,
///  and a link at either one sends a creation,
/// a claim and the sweep of a killed fill into
///  whatever tree somebody else chose,
///  where the sweep removes whole trees.
///  A path that is not there is the empty pool every caller already handles,
/// which `create_private_directory_all` makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePoolPathIsTheWrappersOwn(root) // false when either name is a link or a file
/// ```
pub(crate) fn the_pool_path_is_the_wrappers_own(root: &Path) -> bool {
    let folder_is_ours: bool = root.parent().is_none_or(the_wrappers_own_directory);
    return folder_is_ours && the_wrappers_own_directory(root);
}

/// What:
///  Removes the pool folder of every worktree Git no longer lists,
///  and answers which folders it removed.
/// Why:
///  A worktree's pool is named after that worktree's own Git directory,
///  and the members of a folder whose Git directory is gone are private repositories with their links into this one:
///  no commit can ever claim them.
///  A `git worktree prune`,
///  a registration deleted by hand and a worktree destroyed without Git all leave that folder,
///  and the sweep is what collects it,
///  so a pool left unused does not keep a copy of the repository's shape for ever.
///  The question is asked of the repository's own `worktrees` folder and never of Git,
///  so a command pays one `read_dir` that usually finds no pool folder at all.
///  A worktree Git is adding keeps its folder:
///  Git makes the registration before the checkout,
///  so a registration is the worktree's presence and not a leftover.
///  A folder the wrapper did not name stays:
///  only a folder named for a worktree can be proven to belong to one,
///  and the pool holds state of the person's own otherwise.
///  A name that is no directory stays as well:
/// the pool's folder and every pool in it are directories this program
///  made,
/// and a link at either name is somebody else's,
/// whose target this sweep never reads and never removes
///  (a review finding,
/// F4).
///  Nothing here can fail a command:
///  a folder that cannot be removed is named in the diagnostic log and left for the next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = sweepGoneWorktrees(commonDir); // the pool folders of worktrees Git no longer lists
/// ```
pub fn sweep_gone_worktrees(common_dir: &Path) -> Vec<PathBuf> {
    let root: PathBuf = common_dir.join(SHADOW_POOL_FOLDER);
    // Both names this program made above a pool are read before any entry below them is: a link at the pool's folder
    // would send the listing and every removal below it into whatever tree it names (a review finding, F4).
    if !the_pool_path_is_the_wrappers_own(root.as_path()) {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(root.as_path()) else {
        return Vec::new();
    };
    // The keys of the worktrees Git lists, made with the same rule the folders were named by.
    let mut listed: Vec<String> = Vec::new();
    if let Ok(worktrees) = std::fs::read_dir(common_dir.join("worktrees")) {
        for entry in worktrees.flatten() {
            listed.push(pool_key(
                common_dir,
                &common_dir.join("worktrees").join(entry.file_name()),
            ));
        }
    }
    let mut removed: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(WORKTREE_KEY_PREFIX) || listed.contains(&name) {
            continue;
        }
        // What the entry is, never what it is named: `file_type` answers for the entry itself and follows no link.
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => {}
            _not_a_pool => continue,
        }
        let folder: PathBuf = entry.path();
        match remove_tree(folder.as_path()) {
            Ok(()) => {
                crate::diagnostic_log::debug(
                    "shadowPool",
                    format!("removed the pool of the worktree Git no longer lists: {name}")
                        .as_str(),
                );
                removed.push(folder);
            }
            Err(error) => {
                crate::diagnostic_log::warn("shadowPool", format!("the pool of the worktree Git no longer lists, {}, was not removed: {error}", folder.display()).as_str());
            }
        }
    }
    return removed;
}

/// What:
///  Removes the half-made members a killed fill left,
///  as of the moment given.
/// Why:
///   A fill publishes its member with one rename,
/// so anything still named as a fill belongs to a process that stopped,
/// and nothing else ever claims it.
///  The moment is a parameter so a control can show the age that removes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweepAbandoned(root, now)
/// ```
pub(super) fn sweep_abandoned(
    root: &Path,
    now: SystemTime,
) {
    // Only the wrapper's own folder is swept.
    if !the_pool_path_is_the_wrappers_own(root) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(FILLING_PREFIX) {
            continue;
        }
        let old: bool = match entry.metadata().and_then(modification_time) {
            Ok(modified) => now
                .duration_since(modified)
                .is_ok_and(older_than_the_fill_age),
            Err(_) => false,
        };
        if old {
            let _removed: std::io::Result<()> = remove_tree(entry.path().as_path());
            crate::diagnostic_log::debug(
                "shadowPool",
                format!("removed the abandoned fill {name}").as_str(),
            );
        }
    }
}

/// What:
///  When one entry of the pool was last written.
/// Why:
///  A fill that was killed leaves a folder whose age says whether a fill is still writing into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modificationTime = (metadata) => metadata.modified();
/// ```
fn modification_time(metadata: std::fs::Metadata) -> std::io::Result<SystemTime> {
    return metadata.modified();
}

/// What:
///  Whether a fill has been unfinished for longer than any fill takes.
/// Why:
///  `sweep_abandoned` removes what this answers yes for,
/// and names the age it uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const olderThanTheFillAge = (age) => age > FILLING_STALE_AFTER;
/// ```
fn older_than_the_fill_age(age: Duration) -> bool {
    return age > FILLING_STALE_AFTER;
}
