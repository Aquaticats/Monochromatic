//! What:
//!  The install of a first index into a new worktree: written into `index.lock`, linked under the name `index` without
//!  replacing anything, and the lock released only if the name still holds the file made here.
//! Why:
//!  The index of a new worktree must never be written in place and never replace one that exists; the steps and their
//!  reasons are on `install_index_in`. The code sits in a file of its own so `subvolume_index_files.rs` stays under the
//!  line limit, and it uses nothing from that file (that file re-exports it).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! installIndexIn(admin, seed, stats, hashLength); // throws a reason
//! ```

use crate::subvolume_file_write_steps::write_all_then_sync;
use crate::subvolume_index::{IndexOutcome, RewriteContext, StatFields, accept_every_stat, rewrite_index_stat_with};
use std::ffi::OsStr;

/// The call the install makes at a seam when no control wants one: none.
fn do_nothing() {}

/// What:
///  Install `seed` as the index of the new worktree whose administrative directory is the open handle `admin`, with the stat
///  fields of `stats` for the entries it knows, and nothing for the others.
/// Why:
///   The index of a new worktree must never be written in place and never replace one that exists: `index.lock` is created
///  first (exclusively), the absence of an index is checked under it, the finished index is written into the lock, and
///  that open file is linked under the name of the index without replacing anything; the lock name is released afterwards
///  only if it still holds that file, because once it is released it is another Git process's to take. The stat fields come from the objects the caller made, so an entry whose file was
///  replaced afterwards no longer matches its inode and Git hashes it. Everything is done through the handle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// installIndexIn(admin, seed, stats, hashLength); // throws a reason
/// ```
pub fn install_index_in(admin: &std::fs::File, seed: &[u8], stats: &mut dyn FnMut(&[u8]) -> Option<StatFields>, hash_length: usize) -> Result<String, String> {
    return install_index_in_with(admin, seed, stats, hash_length, &mut InstallSeams { before_publish: &mut do_nothing, after_publish: &mut do_nothing });
}

/// The moments of an index install at which the controls move the tree.
pub struct InstallSeams<'a> {
    /// Called after the index was written into the lock and before it is linked under its name.
    pub before_publish: &'a mut dyn FnMut(),
    /// Called right after the index got its name and before the lock is released.
    pub after_publish: &'a mut dyn FnMut(),
}

/// `install_index_in` with the moments at which a control may move the tree.
pub fn install_index_in_with(admin: &std::fs::File, seed: &[u8], stats: &mut dyn FnMut(&[u8]) -> Option<StatFields>, hash_length: usize, seams: &mut InstallSeams<'_>) -> Result<String, String> {
    use crate::fs_beneath::{create_file_at, entry_metadata_at, identity_of, link_open_file_at};
    let lock: &OsStr = OsStr::new("index.lock");
    let index: &OsStr = OsStr::new("index");
    let mut created: std::fs::File = match create_file_at(admin, lock, 0o600) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Err(String::from("index.lock exists: a Git command is running there")),
        Err(error) => return Err(format!("the lock cannot be created ({error})")),
    };
    let ours = match identity_of(&created) {
        Ok(identity) => identity,
        Err(error) => return Err(format!("the lock cannot be examined ({error})")),
    };
    match entry_metadata_at(admin, index) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return give_up(admin, lock, ours, format!("the index cannot be examined ({error})")),
        Ok(_) => return give_up(admin, lock, ours, String::from("an index appeared in the new worktree while it was made")),
    }
    let now: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let context: RewriteContext = RewriteContext { hash_length: Some(hash_length), index_mtime: Some((now.as_secs() as u32, now.subsec_nanos())) };
    let (rewritten, updated, total): (Vec<u8>, usize, usize) = match rewrite_index_stat_with(seed, &context, stats, &mut accept_every_stat) {
        IndexOutcome::Rewritten { bytes, updated, total } => (bytes, updated, total),
        IndexOutcome::Skipped(reason) => return give_up(admin, lock, ours, format!("the index cannot be refreshed ({reason})")),
    };
    if let Err(error) = write_all_then_sync(&mut created, rewritten.as_slice()) {
        return give_up(admin, lock, ours, format!("the new index cannot be written ({error})"));
    }
    (seams.before_publish)();
    // The open file is linked under the name of the index, which fails if the name exists: what is published is the file
    // written here, whatever the lock name refers to by now. The lock stays until the index has its name.
    if let Err(error) = link_open_file_at(&created, admin, index) {
        return give_up(admin, lock, ours, format!("the new index cannot be put in place ({error})"));
    }
    (seams.after_publish)();
    // The lock name and the index are the same file now; the lock is released only if the name still holds it, so the lock
    // of the next Git command is never removed.
    let _released = crate::fs_beneath::remove_if_identical(admin, lock, ours);
    return Ok(format!("{updated} of {total} entries refreshed"));
}

/// Before the publication the lock is ours to remove, but only if the name still holds the file made here; the error is `reason`.
fn give_up(admin: &std::fs::File, lock: &OsStr, ours: crate::fs_beneath::ObjectId, reason: String) -> Result<String, String> {
    let _kept = crate::fs_beneath::remove_if_identical(admin, lock, ours);
    return Err(reason);
}
