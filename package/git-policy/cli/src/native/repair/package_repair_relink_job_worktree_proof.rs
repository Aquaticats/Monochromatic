//! What:
//!  What a delayed relink proves about the worktree its record names,
//!  before it starts that record's first tool and before every tool after it.
//! Why:
//!  A record outlives the worktree it was written for:
//!  a service that died holds a claim,
//!  a claim that ran out is queued again,
//!  and a person may remove the worktree and put their own directory at that path meanwhile.
//!  The tools rewrite link layouts in place,
//!  so the directory is proved to be a registration of this repository before any of them starts,
//!  and the answer names what was left exactly as it is.
//!  One record names one directory,
//!  so that directory is held open:
//!  the pointer is read through the descriptor rather than through the path,
//!  and the proof is repeated before every tool,
//!  because the tool before it can take a minute and the path may hold another directory by then.
//!  A worktree removed and added again at the same path before this record was drained is a registration
//!  of this repository too,
//!  and each tool decides its need again from the directory it finds,
//!  so the run there is the one that worktree's own record asks for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = HeldWorktree.open(entry.root);
//! for (const tool of entry.tools) { const refusal = refusalOf(held, item, entry); if (refusal) return refusal; await tool(); }
//! ```

use crate::service_work_handlers_replanning_and_acceptance::Verdict;
use crate::work_item::WorkItem;
use crate::worktree_registration_proof::{RegistrationRefusal, is_absent, read_record, registration_of_pointer};
use std::path::{Path, PathBuf};

/// What:
///  Why this record is not acted on now:
/// it is no longer the directory that was proved to be the record's worktree,
/// or that directory is no longer this repository's registration at that path.
/// Why:
///   The tools rewrite link layouts in place,
/// so a directory this repository no longer registers there is left exactly as it is,
/// and one that could not be read now may be readable a moment later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = (held, item, entry) => held.stillAt(entry.root) ? registrationOfPointer(item.gitDir, entry.root, held.pointer()).error : 'the directory was replaced';
/// ```
pub(super) fn refusal_of_a_replaced_or_unregistered_worktree(item: &WorkItem, entry: &crate::package_repair_relink::RelinkEntry, held: &HeldWorktree) -> Option<Verdict> {
    if !held.still_at(entry.root.as_path()) {
        return Some(Verdict::Skipped(format!(
            "{} is no longer the directory this relink was proved on, so it was left exactly as it is",
            entry.root.display()
        )));
    }
    let pointer: Vec<u8> = match read_record(held.inside(".git").as_path(), "the worktree's pointer") {
        Ok(bytes) => bytes,
        Err(refusal) => return Some(verdict_for_a_refusal(refusal)),
    };
    return match registration_of_pointer(item.git_dir.as_path(), entry.root.as_path(), pointer.as_slice()) {
        Ok(_registration) => None,
        Err(refusal) => Some(verdict_for_a_refusal(refusal)),
    };
}

/// What:
///  The verdict one refusal of the registration is.
/// Why:
///   A directory that is no worktree of this repository any more is left exactly as it is and its record is done,
/// while a proof that could not be read for a reason that may pass
/// (a permission,
/// an input-output error,
/// a descriptor table that is full)
/// keeps the record's claim,
/// which queues the same work again when the claim runs out,
/// so a transient refusal costs one try and never the promised repair.
/// The words name the reason the proof gave, so the service's journal says what was left and why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = (refusal) => refusal.kind === 'not-this-repository'
///   ? { skipped: `this directory is no worktree of the repository any more (${refusal.reason})` }
///   : { retry: `this directory's registration could not be read now (${refusal.reason})` };
/// ```
pub(super) fn verdict_for_a_refusal(refusal: RegistrationRefusal) -> Verdict {
    return match refusal {
        RegistrationRefusal::NotThisRepository(reason) => Verdict::Skipped(format!("this directory is no worktree of the repository any more ({reason}), so it was left exactly as it is")),
        RegistrationRefusal::Undecided(reason) => Verdict::Retry(format!("this directory's registration could not be read now ({reason})")),
    };
}

/// What:
///  The worktree's directory held open for one relink.
/// Why:
///   The record outlives the worktree it was written for,
/// so the run reads the pointer through this descriptor
/// and asks the path again before every tool,
/// and stops the run when the path names another directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = HeldWorktree.open(root); // { directory, identity }
/// if (!held.stillAt(root)) return skipped('the directory was replaced');
/// ```
pub(super) struct HeldWorktree {
    /// The directory's descriptor,
    /// which keeps naming the directory it was opened on.
    directory: std::fs::File,
    /// The identity the directory had when it was opened.
    identity: crate::fs_beneath::ObjectId,
}

impl HeldWorktree {
    /// What:
    ///  Open `root` and take its identity.
    /// Why:
    ///   `open_root` refuses a path reached through a link or by a non-canonical spelling,
    /// which a worktree Git registered under its own canonical path never is,
    /// and it holds the directory so the names inside it are read through this descriptor rather than through the path.
    /// A missing directory is this record's worktree gone,
    /// while a directory the system refused to open for another reason may be readable a moment later.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const held = HeldWorktree.open(root);
    /// ```
    pub(super) fn open(root: &Path) -> Result<HeldWorktree, RegistrationRefusal> {
        let directory: std::fs::File = match crate::fs_beneath::open_root(root) {
            Ok(opened) => opened,
            Err(error) if is_absent(&error) || error.kind() == std::io::ErrorKind::InvalidInput => {
                return Err(RegistrationRefusal::NotThisRepository(format!("{} is no directory of this repository ({error})", root.display())))
            }
            Err(error) => return Err(RegistrationRefusal::Undecided(format!("{} cannot be opened ({error})", root.display()))),
        };
        return match crate::fs_beneath::identity_of(&directory) {
            Ok(identity) => Ok(HeldWorktree { directory, identity }),
            Err(error) => Err(RegistrationRefusal::Undecided(format!("{} cannot be examined ({error})", root.display()))),
        };
    }

    /// What:
    ///  The path of `name` inside the held directory,
    ///  addressed through its descriptor.
    /// Why:
    ///   A path such as `<worktree>/.git` is resolved again for every call,
    /// so a directory replaced at that path between two calls would have its own files read.
    /// `/proc/self/fd/<descriptor>/<name>` names the directory that was opened, whatever its path names later.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pointerPath = held.inside('.git');
    /// ```
    pub(super) fn inside(&self, name: &str) -> PathBuf {
        return self.descriptor().join(name);
    }

    /// The path that names the held directory itself.
    fn descriptor(&self) -> PathBuf {
        use std::os::unix::io::AsRawFd as _;
        return PathBuf::from(format!("/proc/self/fd/{}", self.directory.as_raw_fd()));
    }

    /// What:
    ///  Whether `root` still names the directory this was opened on.
    /// Why:
    ///   The proof about the registration is a proof about one directory,
    /// so a tool may only start in it:
    /// a directory removed and replaced at the same path is another directory, whatever its pointer says.
    /// Two questions are asked because neither answers alone:
    /// the descriptor's own name says whether the directory it holds is still the one at that path
    /// (a removed one reads `<path> (deleted)`,
    /// and a renamed one reads its new path),
    /// and the identity says whether that name holds the same directory rather than another one whose inode the
    /// filesystem reused.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const same = readlink(held.descriptor) === root && identityOfPath(root) === held.identity;
    /// ```
    pub(super) fn still_at(&self, root: &Path) -> bool {
        let held_named: PathBuf = self.descriptor();
        let Ok(named_at) = std::fs::read_link(held_named.as_path()) else { return false };
        if named_at.as_os_str() != root.as_os_str() {
            return false;
        }
        return crate::worktree_add_binding::path_identity(root).as_ref() == Some(&self.identity);
    }
}
