//! What:
//!  The check that a path the linked-worktree copy acts on is one it may act on:
//!  canonical,
//!  owned by this account,
//!  private,
//!  and of the kind its role says.
//! Why:
//!  A journal,
//!  a stage container or a stage claim that another account can write,
//!  or one that a link redirects,
//!  is never acted on,
//!  so every reader of the copy's records asks this one question.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await assertPrivateWorktreeCopyPath({ path, role });
//! ```

use crate::json_record::quote;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    failure_with_cause,
    raw_failure_of,
};
use std::path::Path;
/// What:
///  The group and other permission bits a private path must not have.
/// Why:
///   A journal or stage readable or writable by others could be planted or read by another account.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NON_PRIVATE_MODE_BITS = 0o077;
/// ```
const NON_PRIVATE_MODE_BITS: u32 = 0o077;

/// What:
///  The role a private path plays,
/// which selects its expected entry kind.
/// Why:
///   A journal file must be a single-link file;
/// a directory must be a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrivateWorktreeCopyPathRole = 'journal directory' | 'journal file' | 'private stage';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    /// A journal parent or root directory.
    JournalDirectory,
    /// A journal file.
    JournalFile,
    /// A private stage container or payload.
    PrivateStage,
}

impl Role {
    /// The role's name in diagnostics.
    fn name(self) -> &'static str {
        // `match` keeps each role's diagnostic spelling next to its variant.
        match self {
            Role::JournalDirectory => return "journal directory",
            Role::JournalFile => return "journal file",
            Role::PrivateStage => return "private stage",
        }
    }
}

/// What:
///  Checks that a path is canonical,
/// owned by this account,
/// private,
/// and of the role's kind.
/// Why:
///   A journal or stage that another account can write,
/// or that a link redirects,
/// is never acted on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await assertPrivateWorktreeCopyPath({ path, role });
/// ```
pub fn assert_private_path(
    path: &Path,
    role: Role,
) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::MetadataExt;
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).map_err(raw_failure_of)?;
    let shape: bool = match role {
        Role::JournalFile => metadata.is_file() && metadata.nlink() == 1,
        Role::JournalDirectory | Role::PrivateStage => metadata.is_dir(),
    };
    // SAFETY: `geteuid` has no preconditions and cannot fail.
    let effective_user: u32 = unsafe { libc::geteuid() };
    let canonical: bool = std::fs::canonicalize(path)
        .map_err(raw_failure_of)?
        .as_path()
        == path;
    if !shape
        || (metadata.mode() & NON_PRIVATE_MODE_BITS) != 0
        || metadata.uid() != effective_user
        || !canonical
    {
        return Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: worktree-copy {} is unsafe: {}.",
                role.name(),
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    return Ok(());
}
