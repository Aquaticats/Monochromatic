//! What:
//!  Turning a newly registered worktree into a durable copy request, for the wrapper that created it and for the service
//!  that finds what a killed wrapper left.
//! Why:
//!  Both write the same request for the same registration: the token, the marker and the request of a registration are
//!  looked up before anything is written, so a request is never made for a copy that is over or already waiting, and two
//!  writers of one registration agree. The service reads no output of Git here: the worktree's root is in the pointer file
//!  Git keeps in its administrative directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const published = await publishRequest({ commonDir, sourceRoot, selection, worktree, allowLocked });
//! ```

use super::json_record::quote;
use super::worktree_copy_error::WorktreeCopyError;
use super::worktree_copy_intent::{INTENT_KEPT_MILLISECONDS, Intent, read_intents, remove_intent};
use super::worktree_copy_registry::{
    CreatedWorktree, admin_identity, copied_for, ensure_registration_token, is_locked, read_admin_ids,
};
use super::worktree_copy_request::{CopyRequest, new_request_id, request_for_registration, write_request};
use super::worktree_copy_selection::CopySelection;
use std::path::{Path, PathBuf};

/// What came of asking for the copy of one registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Published {
    /// A request for this registration stands, written now or earlier; its identity.
    Request(String),
    /// The registration's copy is over already.
    Already,
    /// The registration is gone, or is no longer the directory that was observed.
    Gone,
}

/// What:
///  Writes the request for one registration unless its copy is over or a request for it already stands.
/// Why:
///   The token is written through a handle to the directory that was observed, so a replacement under the same name is
///  `Gone`; the marker and the pending requests are looked at first, so a second writer reuses the first one's work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const published = await publishRequest({ commonDir, sourceRoot, selection, worktree, allowLocked });
/// ```
pub fn publish_request(
    common_dir: &Path,
    source_root: &Path,
    selection: &CopySelection,
    worktree: &CreatedWorktree,
    allow_locked: bool,
) -> Result<Published, WorktreeCopyError> {
    let admin_root: PathBuf = common_dir.join("worktrees");
    let Some(token) = ensure_registration_token(admin_root.as_path(), worktree.admin_id.as_str(), Some(worktree.identity.as_str()))?
    else {
        return Ok(Published::Gone);
    };
    if copied_for(admin_root.as_path(), worktree.admin_id.as_str(), token.as_str())? {
        return Ok(Published::Already);
    }
    if let Some(existing) = request_for_registration(common_dir, worktree.admin_id.as_str(), token.as_str())? {
        return Ok(Published::Request(existing));
    }
    let request: CopyRequest = CopyRequest {
        id: new_request_id()?,
        admin_id: worktree.admin_id.clone(),
        registration: token,
        destination_root: worktree.root.clone(),
        source_root: source_root.to_path_buf(),
        selection: selection.clone(),
        tries: 0,
        identity: worktree.identity.clone(),
        allow_locked,
    };
    write_request(common_dir, &request)?;
    return Ok(Published::Request(request.id));
}

/// The canonical root of the worktree an administrative directory belongs to, from the pointer file Git keeps in it.
///  Nothing when the file is missing or unreadable, or the worktree is gone.
pub fn root_from_pointer(admin_dir: &Path) -> Option<PathBuf> {
    let text: String = std::fs::read_to_string(admin_dir.join("gitdir")).ok()?;
    let pointer: PathBuf = PathBuf::from(text.trim_end_matches(['\n', '\r']));
    let absolute: PathBuf = if pointer.is_absolute() { pointer } else { admin_dir.join(pointer) };
    if absolute.file_name()? != ".git" {
        return None;
    }
    return std::fs::canonicalize(absolute.parent()?).ok();
}

/// Whether the path the command line named is this root.
pub fn is_added_root(added_path: Option<&Path>, root: &Path) -> bool {
    return added_path
        .and_then(|path| return std::fs::canonicalize(path).ok())
        .is_some_and(|canonical| return canonical == root);
}

/// What:
///  Writes the requests for the worktrees a command created, from its intent alone.
/// Why:
///   The wrapper that wrote the intent may have been killed after Git returned; the intent holds what the wrapper knew
///  before Git ran, which is enough to find the worktrees Git added and to leave the ones another command is still
///  creating (locked) alone. An intent more than an hour old is removed after it was read: it can no longer be waiting
///  for a Git command to finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await reconcileIntents(commonDir, nowMilliseconds)
/// ```
pub fn reconcile_intents(common_dir: &Path, now_milliseconds: u128) -> Result<Vec<String>, WorktreeCopyError> {
    let (intents, unreadable) = read_intents(common_dir)?;
    let mut notices: Vec<String> = Vec::new();
    for description in unreadable {
        notices.push(format!(
            "cli-git: could not read the copy intent {}; it is left in the repository's copy directory.",
            quote(description.as_str())
        ));
    }
    let admin_root: PathBuf = common_dir.join("worktrees");
    for intent in intents {
        reconcile_intent(common_dir, admin_root.as_path(), &intent)?;
        let expired: bool = intent
            .written_at()
            .is_some_and(|written| return now_milliseconds.saturating_sub(written) > INTENT_KEPT_MILLISECONDS);
        if expired {
            remove_intent(common_dir, &intent)?;
        }
    }
    return Ok(notices);
}

fn reconcile_intent(common_dir: &Path, admin_root: &Path, intent: &Intent) -> Result<(), WorktreeCopyError> {
    let after: std::collections::BTreeSet<String> = read_admin_ids(admin_root)?;
    for admin_id in after.difference(&intent.before_admin_ids) {
        let Some(identity) = admin_identity(admin_root, admin_id.as_str())? else { continue };
        let Some(root) = root_from_pointer(admin_root.join(admin_id).as_path()) else { continue };
        let locked: bool = is_locked(admin_root, admin_id.as_str())?;
        let allow_locked: bool = is_added_root(intent.added_path.as_deref(), root.as_path());
        // A worktree that is locked and that this command did not ask for is another command's while it is created.
        if locked && !allow_locked {
            continue;
        }
        let worktree: CreatedWorktree = CreatedWorktree { admin_id: admin_id.clone(), root, locked, identity };
        publish_request(common_dir, intent.source_root.as_path(), &intent.selection, &worktree, allow_locked)?;
    }
    return Ok(());
}

#[cfg(test)]
#[path = "worktree_copy_publish_tests.rs"]
mod tests;
