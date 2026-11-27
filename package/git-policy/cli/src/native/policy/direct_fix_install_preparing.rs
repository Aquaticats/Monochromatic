//! What:
//!  Preparing every correction of one fix:
//!  proving each selected file still holds the bytes the fix read,
//!  recording what the file was,
//!  and writing the corrected bytes into a journal inside the Git directory.
//! Why:
//!  Nothing in the worktree changes until every file is prepared,
//!  so a file that changed under the fix stops the whole install before any file
//!       is replaced,
//!  and a kill leaves a record of what to do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const prepared = prepareReplacements(root, gitDir, changes);
//! ```

/// The journal a correction is installed through.
use crate::direct_fix_install_journal::{
    content_of,
    open_journal,
    remove,
};
/// The judgement of what a name holds.
use crate::direct_fix_install_judging::{
    SiteState,
    state_of,
};
/// The sentences the install reports.
use crate::direct_fix_install_messages::{
    install_failure,
    journal_refusal,
    unchanged_failure,
};
/// The correction to prepare,
///  and the site it is prepared as.
use crate::direct_fix_install_records::{
    InstallChange,
    Prepared,
};
/// The write of a new file that installs the corrected bytes.
use crate::direct_fix_install_writes::write_new;
/// The conversion of Git pathname bytes to a native path.
use crate::git_metadata::path_from_git_bytes;
/// The proof that this process can be named as the journal's writer.
use crate::worktree_copy_owner::current_copy_owner;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;
/// What:
///  The token that names this process in a journal,
///  or nothing where the platform cannot report it.
/// Why:
///   The sweep removes a journal only on a proof about its writer,
///  and a writer nothing can name is never
///       taken for dead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const token = currentCopyOwner()?.token();
/// ```
fn writer_token() -> Option<String> {
    return match current_copy_owner() {
        Ok(owner) => Some(owner.token()),
        Err(_unreported) => None,
    };
}
/// What:
///  Prepare every replacement:
///  check the file still holds the bytes the fix read,
///       record what the
///       file was,
///  and write the corrected bytes into the journal inside the Git directory.
/// Why:
///   Nothing in the worktree changes until every file is prepared,
///  and every change is recorded before
///       it is made,
///  so a file that changed under the fix stops it before any file is replaced and a
///       kill leaves a record of what to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareReplacements(root, gitDir, changes): Promise<Prepared[]>;
/// ```
pub fn prepare_replacements(
    root: &Path,
    git_dir: &Path,
    changes: &[InstallChange],
) -> Result<Vec<Prepared>, String> {
    let mut prepared: Vec<Prepared> = Vec::new();
    for change in changes {
        match prepare_one(root, git_dir, change) {
            Ok(site) => prepared.push(site),
            Err(message) => {
                for earlier in &prepared {
                    remove(earlier.journal.as_path());
                }
                return Err(message);
            }
        }
    }
    return Ok(prepared);
}

/// What:
///  `prepare_replacements` for one change.
/// Why:
///   The failure path of the whole preparation is one place,
///  so each refusal here just says what happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareOne(root, gitDir, change): Promise<Prepared>;
/// ```
fn prepare_one(
    root: &Path,
    git_dir: &Path,
    change: &InstallChange,
) -> Result<Prepared, String> {
    let Some(relative) = path_from_git_bytes(change.path.as_slice()) else {
        return Err(install_failure(
            "Git named a corrected file this system cannot represent, so no file was changed.",
        ));
    };
    let destination: PathBuf = root.join(relative);
    let found: SiteState =
        state_of(destination.as_path(), "reading a corrected file").map_err(unchanged_failure)?;
    let SiteState::File {
        bytes,
        kept,
    } = found
    else {
        if let SiteState::Absent = found {
            return Err(install_failure(
                "a selected file is gone, and putting corrected bytes at its path would create a file the fix never read, so no file was changed.",
            ));
        }
        // `symlink_metadata` describes the name itself, so a symbolic link is seen as a link and not followed; reading
        // through it would read another file, and the swap would put bytes where the link is.
        return Err(install_failure(
            "a selected file is not a regular file (a symbolic link, a directory or a device), and putting corrected bytes at its path would remove what is there, so no file was changed.",
        ));
    };
    if bytes.as_slice() != &*change.original {
        return Err(install_failure(
            "a selected file changed while monochromatic-git-policy-cli fix ran, so no file was changed. Run monochromatic-git-policy-cli fix again.",
        ));
    }
    let journal: PathBuf = open_journal(
        git_dir,
        destination.as_path(),
        &kept,
        &change.replacement,
        writer_token().as_deref(),
    )
    .map_err(journal_refusal)?;
    let content: PathBuf = content_of(journal.as_path());
    match write_new(content.as_path(), &change.replacement, &kept) {
        Ok(()) => {}
        Err(failure) => {
            remove(journal.as_path());
            return Err(install_failure(
                format!("writing a corrected file into its journal failed ({failure}), so no file was changed.")
                    .as_str(),
            ));
        }
    }
    return Ok(Prepared {
        destination,
        journal,
        original: Rc::clone(&change.original),
        replacement: Rc::clone(&change.replacement),
        kept,
    });
}
