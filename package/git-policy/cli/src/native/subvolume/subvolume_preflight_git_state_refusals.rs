//! What:
//!  The refusals about Git's own state in a checkout and in each linked worktree:
//!  the files and directories whose presence means Git is mid-operation,
//!  a conversion of that worktree that is running now,
//!  and unfinished work of the commit service.
//! Why:
//!  Converting a checkout under a running Git command moves the index and the worktrees it is writing,
//!  so each of these is refused with its cause and every way forward,
//!  and a lock this migration holds itself is the one file that is not a refusal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusals = gitStateRefusals(layout, journal);
//! ```

use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::single_owner_claim::live_owner_of;
use crate::subvolume_journal::Journal;
use crate::subvolume_lock::{
    is_ours,
    token_of,
};
use crate::subvolume_preflight_paths::Layout;
use crate::subvolume_preflight_refusal::{
    Refusal,
    refuse,
};
use std::path::{
    Path,
    PathBuf,
};

/// The names whose presence in `.git` or in a linked worktree's administrative directory means Git is mid-operation.
/// Why a bisect has two of them:
///  `git bisect start` writes `BISECT_START`,
/// then `BISECT_NAMES`,
///  and opens `BISECT_LOG` only after both
///  (measured with Git 2.56 under `strace`),
///  so a process killed between those writes leaves a bisect whose mark is a file written before the kill,
///  and a list that knew only the log moved a checkout with that bisect still unfinished.
pub const IN_PROGRESS_NAMES: &[&str] = &[
    "index.lock",
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "BISECT_START",
    "BISECT_LOG",
    "rebase-merge",
    "rebase-apply",
    "sequencer",
    "HEAD.lock",
    "config.lock",
    "packed-refs.lock",
    "shallow.lock",
];

/// The refusal for a conversion of the linked worktree whose administrative directory this is,
/// when its command is running.
fn conversion_refusal(administrative: &Path) -> Option<Refusal> {
    return match live_owner_of(
        administrative,
        std::ffi::OsStr::new(crate::worktree_convert_journal::CLAIM_PREFIX),
    ) {
        Ok(None) => None,
        Ok(Some(owner)) => Some(refuse(
            format!(
                "process {} is converting the linked worktree registered in {} to a subvolume (it owns {})",
                owner.pid,
                administrative.display(),
                owner.path.display()
            ),
            "Wait for `git monochromatic-git-policy-cli convert-worktree-to-subvolume` to finish, then run the command again.",
        )),
        Err(error) => Some(refuse(
            format!(
                "whether the linked worktree registered in {} is being converted cannot be decided ({error})",
                administrative.display()
            ),
            "Look at the claim file the message names; if no monochromatic-git-policy-cli command is running, remove it, then run the command again.",
        )),
    };
}

/// The refusals about Git's own state in the checkout.
pub(super) fn git_state_refusals(
    layout: &Layout,
    journal: &Journal,
) -> Vec<Refusal> {
    let mut found: Vec<Refusal> = Vec::new();
    let git_directory: PathBuf = layout.root.join(".git");
    if !git_directory.is_dir() {
        found.push(refuse(
            format!("{} is not a directory, so this is not the main checkout of a repository", git_directory.display()),
            "Run the command from the main checkout (the directory that holds the repository's `.git` directory), not from a linked worktree.",
        ));
        return found;
    }
    let token: String = token_of(journal);
    let mut directories: Vec<PathBuf> = vec![git_directory.clone()];
    if let Ok(names) = std::fs::read_dir(git_directory.join("worktrees")) {
        for name in names.flatten() {
            directories.push(name.path());
        }
    }
    directories.sort();
    for directory in &directories {
        for &name in IN_PROGRESS_NAMES {
            let own_lock: bool =
                name == "index.lock" && is_ours(directory.join(name).as_path(), token.as_str());
            if directory.join(name).exists() && !own_lock {
                found.push(refuse(
                    format!("Git is in the middle of an operation: {} exists", directory.join(name).display()),
                    "Finish or abort that operation (commit, merge, rebase, cherry-pick, bisect), or, if no Git command is running, remove the stale file after checking, then run the command again.",
                ));
            }
        }
        if let Some(refusal) = conversion_refusal(directory.as_path()) {
            found.push(refusal);
        }
        for queue in ["pending", "claimed"] {
            let path: PathBuf =
                path_for_reading(directory.as_path(), "monochromatic-git-policy-cli/work")
                    .join(queue);
            if let Ok(mut entries) = std::fs::read_dir(&path)
                && entries.next().is_some()
            {
                found.push(refuse(
                    format!("the commit service has unfinished work in {}", path.display()),
                    "Wait for the service to finish landing its commits (`git monochromatic-git-policy-cli service status`), then run the command again.",
                ));
            }
        }
    }
    return found;
}
