//! What:
//!  The linked worktrees a repository's common Git directory registers,
//!  with the branch each one has checked out,
//!  and the ones that were already registered at a moment the caller names.
//! Why:
//!   The registrations are the only list of worktrees a repository owns,
//!  and one file read per worktree names the branch that decides whether it is a candidate at all.
//!  This is the cheap half of the landed sweep:
//!  Git runs only for a branch a merge landed,
//!  never once per registered worktree.
//!  A sweep that reads the list after its merge commit exists would take a worktree another command
//!  added in between,
//!  which did not exist when the merge landed,
//!  so the command that runs the merge names the registrations it found before it started
//!  and the sweep removes only those.
//!  Every record of a registration is read by `worktree_registration_proof.rs`,
//!  which opens without waiting for a writer:
//!  a `gitdir` or a `HEAD` that a process names as a named pipe would otherwise hold the person's `git merge`
//!  for ever,
//!  and a registration whose record Git did not write is no registration this sweep can act on
//!  (a cross-vendor review of the landed sweep,
//!  round 208).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const registered = registrationsOf(commonDir); // throws the reason
//! const before = registeredBefore(commonDir); // the names that existed before the merge
//! ```

use crate::worktree_registration_proof::read_record_here;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// One registration of the repository,
/// with the branch its `HEAD` names when it names one.
/// Why:
///   A registration is the only list entry a repository owns,
/// and a detached `HEAD` is a state a worktree can have:
/// the sweep has no candidate there,
/// because nothing can land on a branch it does not have,
/// while a report about the worktree still has to name it as the worktree it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RegisteredWorktree = { admin: string; worktree: string; branch?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegisteredWorktree {
    /// The administrative directory.
    pub(crate) admin: PathBuf,
    /// The worktree root the registration names.
    pub(crate) worktree: PathBuf,
    /// The branch its `HEAD` names,
    /// as a full ref name,
    /// or nothing for a detached `HEAD`.
    pub(crate) branch: Option<String>,
}

/// What:
///  Every registration of the repository,
///  in one listing of its `worktrees` folder,
///  with the branch each one's `HEAD` names when it names one.
/// Why:
///   The sweep and the report ask the same files the same question,
/// and only what they keep differs:
/// one
///  needs a branch,
/// the other needs the registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registered = registeredWorktrees(commonDir); // throws the reason
/// ```
pub(crate) fn registered_worktrees(common_dir: &Path) -> Result<Vec<RegisteredWorktree>, String> {
    let folder: PathBuf = common_dir.join("worktrees");
    let listing = match std::fs::read_dir(folder.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("the registrations in {} cannot be listed ({error})", folder.display())),
    };
    let mut found: Vec<RegisteredWorktree> = Vec::new();
    for entry in listing.flatten() {
        let admin: PathBuf = entry.path();
        // A registration whose record cannot be read is no candidate of the sweep, which removes what it can identify;
        // a removal that takes registrations with it asks `registrations_a_removal_takes` instead.
        let Ok(Some(worktree)) = registered_worktree(admin.as_path()) else {
            continue;
        };
        let branch: Option<String> = head_branch(admin.as_path());
        found.push(RegisteredWorktree { admin, worktree, branch });
    }
    return Ok(found);
}

/// What:
///  The worktree root a registration's `gitdir` file names,
///  when that root is still a directory,
///  or the reason the record this decides by cannot be read.
/// Why:
///   A removal takes the registrations inside the worktree it removes,
///  so a registration whose record cannot be read may be one of them:
///  the reason is what makes that removal stop,
///  while a sweep that only removes what it can identify skips it
///  (a review of `5a2bd4bb6c`, finding 3).
///  A root that is no longer a directory is no `None` of the same kind:
///  a registration Git prunes names nothing a removal deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function registeredWorktree(admin: string): string | undefined; // throws the reason
/// ```
fn registered_worktree(admin: &Path) -> Result<Option<PathBuf>, String> {
    let text: Vec<u8> = read_record_here(admin.join("gitdir").as_path(), "the record that names its worktree").map_err(|refusal| return refusal.into_words())?;
    let recorded: PathBuf = PathBuf::from(String::from_utf8_lossy(text.trim_ascii_end()).into_owned());
    let pointer: PathBuf = if recorded.is_absolute() { recorded } else { admin.join(recorded) };
    let Some(parent) = pointer.parent() else { return Ok(None) };
    let Ok(root) = std::fs::canonicalize(parent) else { return Ok(None) };
    return Ok(root.is_dir().then_some(root));
}

/// What:
///  The registrations a removal of `outer` takes with it,
///  or the reason one of them cannot be read.
/// Why:
///   A removal of a worktree deletes every registration whose worktree stands inside it,
///  with their checkouts and their own Git directories,
///  so the question about Git work must reach all of them:
///  the listing is read strictly, and one `gitdir` that cannot be read is the reason nothing is removed,
///  because it may name a worktree inside the one being removed
///  (a review of `5a2bd4bb6c`, finding 3).
///  A registration whose worktree is no longer a directory names nothing a removal deletes,
///  and one that names a path outside `outer` is the sweep's and the typed guard's business, not this question's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inside = registrationsARemovalTakes(commonDir, outer); // throws the reason
/// ```
///
/// # Errors
///
/// The words for a `worktrees` folder that cannot be listed,
/// and for a record that is there and cannot be read.
pub(crate) fn registrations_a_removal_takes(common_dir: &Path, outer: &Path) -> Result<Vec<RegisteredWorktree>, String> {
    let folder: PathBuf = common_dir.join("worktrees");
    let listing = match std::fs::read_dir(folder.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("the registrations in {} cannot be listed ({error})", folder.display())),
    };
    let mut inside: Vec<RegisteredWorktree> = Vec::new();
    for entry in listing.flatten() {
        let admin: PathBuf = entry.path();
        match registered_worktree(admin.as_path()) {
            Ok(Some(worktree)) => {
                if worktree != outer && worktree.starts_with(outer) {
                    let branch: Option<String> = head_branch(admin.as_path());
                    inside.push(RegisteredWorktree { admin, worktree, branch });
                }
            }
            // A registration that names no directory any more is a record Git prunes, and no worktree of this removal.
            Ok(None) => continue,
            Err(reason) => {
                return Err(format!(
                    "the registration in {} cannot be read ({reason}), and it may name a worktree inside the one being removed",
                    admin.display()
                ));
            }
        }
    }
    return Ok(inside);
}

/// What:
///  The branch a registration's `HEAD` names,
///  as a full ref name,
///  or nothing for a detached `HEAD`.
/// Why:
///   A removal that keeps a worktree names it the way the sweep does,
///  by its path and the branch it has checked out,
///  and the registration is the only thing that still says which branch that is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// branchOfRegistration(admin) // 'refs/heads/feature' | undefined
/// ```
pub(crate) fn branch_of_registration(admin: &Path) -> Option<String> {
    return head_branch(admin);
}

/// The branch the registration's `HEAD` names,
/// as a full ref name,
/// or nothing for a detached `HEAD`.
fn head_branch(admin: &Path) -> Option<String> {
    let recorded: Vec<u8> = read_record_here(admin.join("HEAD").as_path(), "the record that names its branch").ok()?;
    let text: String = String::from_utf8(recorded).ok()?;
    let line: &str = text.trim_end();
    let branch: &str = line.strip_prefix("ref: ")?;
    if !branch.starts_with("refs/heads/") {
        return None;
    }
    return Some(branch.to_owned());
}

/// The administrative directories a repository registered at one moment.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegisteredBefore {
    /// Their names,
    /// which is what a registration's directory is called and what a removal deletes.
    admins: HashSet<PathBuf>,
}

impl RegisteredBefore {
    /// Whether this registration was already there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// before.holds(registration) // boolean
    /// ```
    pub(crate) fn holds(&self, one: &RegisteredWorktree) -> bool {
        return self.admins.contains(&one.admin);
    }
}

/// What:
///  The names of the administrative directories the repository registers now,
///  for a sweep that runs after a merge that has not happened yet.
/// Why:
///   A merge that lands a branch removes the worktrees whose branches it landed,
///  and a worktree added while the merge ran,
///  or after it landed,
///  did not exist when the merge did,
///  so only the names read before it started may be removed.
///  This is one directory read and no file read:
///  the branch of each registration is read after the merge,
///  for the candidates alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// registeredBefore(commonDir) // { admins }, or the reason they cannot be listed
/// ```
///
/// # Errors
///
/// The words for a `worktrees` folder that cannot be listed.
pub(crate) fn registered_before(common_dir: &Path) -> Result<RegisteredBefore, String> {
    let folder: PathBuf = common_dir.join("worktrees");
    let listing = match std::fs::read_dir(folder.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(RegisteredBefore::default()),
        Err(error) => return Err(format!("the registrations in {} cannot be listed ({error})", folder.display())),
    };
    let mut admins: HashSet<PathBuf> = HashSet::new();
    for entry in listing.flatten() {
        admins.insert(entry.path());
    }
    return Ok(RegisteredBefore { admins });
}
