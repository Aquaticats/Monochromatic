//! What:
//!  The proof that the stage equals what Git's checkout of the target commit makes,
//! plus the ignored paths the restore carried:
//! the index equals the commit's tree,
//! Git shows no untracked file besides those,
//! and no tracked file differs from the index.
//! Why:
//!   The restore trusts Git's reset and clean,
//! so the proof asks Git again from the other side,
//! and a state either of them left ends the rung instead of reaching the person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await proveRestored({ context, target, workTree, carried }); // throws the first thing that is not as Git's checkout leaves it
//! ```

use crate::transaction_git::GitContext;
use crate::worktree_dirty_restore_stage_git_questions::{
    ask,
    is_nonempty_name,
    is_nul_byte,
};
use crate::worktree_ladder_target::Target;
use std::path::Path;

/// What:
///  Whether a name Git lists as untracked is one the restore carried through:
/// the same name,
/// or a file below a
///  carried directory.
/// Why:
///   A carried directory is one name ending in `/`,
/// while a path inside it is listed whole when a tracked file
///  of the new tree also lives there,
/// so the prefix decides for the rest of one directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// carried.some((kept) => name.equals(kept) || (kept.endsWith('/') && name.startsWith(kept)))
/// ```
fn is_carried(
    name: &[u8],
    carried: &[Vec<u8>],
) -> bool {
    for kept in carried {
        if name == kept.as_slice() || (kept.ends_with(b"/") && name.starts_with(kept.as_slice())) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a name Git lists as untracked is a whole directory that holds a path the restore carried.
/// Why:
///   `--directory` collapses a directory whose every path is untracked into that one name,
/// so
///  `cache/private/token` carried by the restore is reported as `cache/`;
/// the name matches no carried
///  path and its content decides,
/// which needs the same query without the collapse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name.endsWith('/') && carried.some((kept) => kept.startsWith(name))
/// ```
fn holds_carried(
    name: &[u8],
    carried: &[Vec<u8>],
) -> bool {
    if !name.ends_with(b"/") {
        return false;
    }
    for kept in carried {
        if kept.starts_with(name) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a name lies below one of the directories Git reported as one collapsed name.
/// Why:
///   Only the paths inside such a directory are judged when the collapse is opened;
/// every other name was
///  carried already.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// directories.some((directory) => name.startsWith(directory))
/// ```
fn lies_below(
    name: &[u8],
    directories: &[Vec<u8>],
) -> bool {
    for directory in directories {
        if name.starts_with(directory.as_slice()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Prove that the stage equals what Git's checkout of the target commit makes,
/// plus the ignored paths the
///  restore carried:
/// the index equals the commit's tree,
/// Git shows no untracked file besides those,
/// and no tracked file
///  differs from the index.
/// Why:
///   The restore trusts Git's reset and clean;
/// the proof asks Git again from the other side,
/// so a state either of them left
///  (a path the reset skipped,
/// a directory the clean could not remove) ends the rung instead of reaching the person.
/// A carried path is what the source ignored and the new commit does not,
/// which the person keeps and the
///  ignored-state copy hands over on every other filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await proveRestored({ context, target, workTree, carried }); // throws the first thing that is not as Git's checkout leaves it
/// ```
pub fn prove_restored(
    context: &GitContext,
    target: &Target,
    work_tree: &Path,
    carried: &[Vec<u8>],
) -> Result<(), String> {
    let cached: Option<i32> = ask(
        context,
        target,
        work_tree,
        &["diff-index", "--cached", "--quiet", target.commit.as_str()],
    )?
    .0;
    if cached != Some(0) {
        return Err(String::from(
            "the restored index does not equal the tree of the new HEAD",
        ));
    }
    let (listed, untracked) = ask(
        context,
        target,
        work_tree,
        &[
            "ls-files",
            "--others",
            "--exclude-standard",
            "--directory",
            "--no-empty-directory",
            "-z",
        ],
    )?;
    if listed != Some(0) {
        return Err(String::from(
            "Git could not list the untracked files of the restored worktree",
        ));
    }
    let names: Vec<Vec<u8>> = untracked
        .split(is_nul_byte)
        .filter(is_nonempty_name)
        .map(<[u8]>::to_vec)
        .collect();
    // The first untracked name the restore did not carry; the loop stops there, as `find` did.
    let mut first_untracked: Option<Vec<u8>> = None;
    let mut collapsed: Vec<Vec<u8>> = Vec::new();
    for name in &names {
        if is_carried(name.as_slice(), carried) {
            continue;
        }
        if holds_carried(name.as_slice(), carried) {
            collapsed.push(name.clone());
            continue;
        }
        first_untracked = Some(name.clone());
        break;
    }
    // A whole directory that holds carried paths is decided by its content, which the collapsed listing does not carry:
    //  the same query without `--directory` names every path below it.
    if first_untracked.is_none() && !collapsed.is_empty() {
        let (expanded_code, expanded) = ask(
            context,
            target,
            work_tree,
            &["ls-files", "--others", "--exclude-standard", "-z"],
        )?;
        if expanded_code != Some(0) {
            return Err(String::from(
                "Git could not list the untracked files below the carried directories of the restored worktree",
            ));
        }
        // Inside a reported directory only its own paths are judged; every other name was carried already.
        for name in expanded.split(is_nul_byte).filter(is_nonempty_name) {
            if lies_below(name, collapsed.as_slice()) && !is_carried(name, carried) {
                first_untracked = Some(name.to_vec());
                break;
            }
        }
    }
    if let Some(name) = first_untracked {
        return Err(format!(
            "{} is still untracked in the restored worktree",
            crate::worktree_snapshot_source::plain(
                String::from_utf8_lossy(name.as_slice()).as_ref()
            )
        ));
    }
    let files: Option<i32> = ask(context, target, work_tree, &["diff-files", "--quiet"])?.0;
    if files != Some(0) {
        return Err(String::from(
            "a tracked file of the restored worktree differs from its index entry",
        ));
    }
    return Ok(());
}
