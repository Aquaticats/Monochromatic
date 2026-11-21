//! What:
//!  The install of a first index into a new worktree:
//! written into `index.lock`,
//! linked under the name `index` without
//!  replacing anything,
//! and the lock released only if the name still holds the file made here.
//! Why:
//!  The index of a new worktree must never be written in place and never replace one that exists;
//! the steps and their
//!  reasons are on `install_index_in`.
//! The code sits in a file of its own so `subvolume_index_files.rs` stays under the
//!  line limit,
//! and it uses nothing from that file (that file re-exports it).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! installIndexIn(admin, seed, stats, hashLength); // throws a reason
//! ```

use crate::subvolume_file_write_steps::write_all_then_sync;
use crate::subvolume_index::{AcceptEveryStat, IndexOutcome, RewriteContext, StatLookup, rewrite_index_stat_with};
use std::ffi::OsStr;

/// The call the install makes at a seam when no control wants one:
/// none.
fn do_nothing() {}

/// What:
///  Install `seed` as the index of the new worktree whose administrative directory is the open handle `admin`,
/// with the stat
///  fields of `stats` for the entries it knows,
/// and nothing for the others.
/// Why:
///   The index of a new worktree must never be written in place and never replace one that exists:
/// `index.lock` is created
///  first (exclusively),
/// the absence of an index is checked under it,
/// the finished index is written into the lock,
/// and
///  that open file is linked under the name of the index without replacing anything;
/// the lock name is released afterwards
///  only if it still holds that file,
/// because once it is released it is another Git process's to take.
/// The stat fields come from the objects the caller made,
/// so an entry whose file was
///  replaced afterwards no longer matches its inode and Git hashes it.
/// Everything is done through the handle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// installIndexIn(admin, seed, stats, hashLength); // throws a reason
/// ```
pub fn install_index_in(admin: &std::fs::File, seed: &[u8], stats: &mut dyn StatLookup, hash_length: usize) -> Result<String, String> {
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
pub fn install_index_in_with(admin: &std::fs::File, seed: &[u8], stats: &mut dyn StatLookup, hash_length: usize, seams: &mut InstallSeams<'_>) -> Result<String, String> {
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
    let (rewritten, report): (Vec<u8>, String) = match rewritten_seed(seed, stats, hash_length) {
        Ok(found) => found,
        Err(reason) => return give_up(admin, lock, ours, reason),
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
    return Ok(report);
}

/// What:
///  The seed with the stat fields of `stats` rewritten into it,
/// and the sentence the install reports.
/// Why:
///   The rewrite is the same step whether the index is linked under the worktree's own name or written under a name of
///  its own until the paths Git must write are written,
/// so it is done once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [bytes, report] = rewrittenSeed(seed, stats, hashLength); // throws a reason
/// ```
fn rewritten_seed(seed: &[u8], stats: &mut dyn StatLookup, hash_length: usize) -> Result<(Vec<u8>, String), String> {
    let now: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let context: RewriteContext = RewriteContext { hash_length: Some(hash_length), index_mtime: Some((now.as_secs() as u32, now.subsec_nanos())) };
    return match rewrite_index_stat_with(seed, &context, stats, &mut AcceptEveryStat) {
        IndexOutcome::Rewritten { bytes, updated, total } => Ok((bytes, format!("{updated} of {total} entries refreshed"))),
        IndexOutcome::Skipped(reason) => Err(format!("the index cannot be refreshed ({reason})")),
    };
}

/// What:
///  Write the new worktree's index under the name `name`,
///  which is not `index`,
/// and hand back the open file that holds it,
///  without publishing anything.
/// Why:
///   A rung that still has paths to write through Git must not publish an index before that write:
/// a kill between the publication and the write leaves an index whose entries no `git status` reports as changed,
///  because the clean form of the source's bytes equals the blob the entry names
/// (`git status` reads the file,
/// finds its clean form equal and refreshes the entry),
///  so the worktree passes as whole while a converted path holds the source's bytes
///  (issue 6323,
/// reached by a kill;
/// `worktree_clone_conversion_beside_the_clone_tests.rs` holds the control).
/// The file under its own name is what Git is pointed at while it writes (`GIT_INDEX_FILE`),
///  so its `-u` records the stat data of what it wrote,
/// and `publish_index_under_a_name` gives it the worktree's own name once the write is done.
/// The absence of an index is checked here under the name,
///  because a worktree that already has one is not this run's to replace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [held, report] = writeIndexUnderAName(admin, seed, stats, hashLength, 'index.pending'); // throws a reason
/// ```
pub fn write_index_under_a_name(admin: &std::fs::File, seed: &[u8], stats: &mut dyn StatLookup, hash_length: usize, name: &OsStr) -> Result<(std::fs::File, String), String> {
    use crate::fs_beneath::{create_file_at, entry_metadata_at, identity_of};
    let index: &OsStr = OsStr::new("index");
    let mut created: std::fs::File = match create_file_at(admin, name, 0o600) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Err(format!("{} exists: an interrupted run left its index there, and this one will not replace it", name.to_string_lossy())),
        Err(error) => return Err(format!("the index's own name cannot be created ({error})")),
    };
    let ours = match identity_of(&created) {
        Ok(identity) => identity,
        Err(error) => return Err(format!("the index's own name cannot be examined ({error})")),
    };
    match entry_metadata_at(admin, index) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return give_up(admin, name, ours, format!("the index cannot be examined ({error})")),
        Ok(_) => return give_up(admin, name, ours, String::from("an index appeared in the new worktree while it was made")),
    }
    let (rewritten, report): (Vec<u8>, String) = match rewritten_seed(seed, stats, hash_length) {
        Ok(found) => found,
        Err(reason) => return give_up(admin, name, ours, reason),
    };
    if let Err(error) = write_all_then_sync(&mut created, rewritten.as_slice()) {
        return give_up(admin, name, ours, format!("the new index cannot be written ({error})"));
    }
    return Ok((created, report));
}

/// What:
///  Publish the index held open as the worktree's `index`,
///  and remove the name it was written under.
/// Why:
///   This is the moment the worktree starts vouching for the stat data of what is on disk,
/// so it comes after the last write this rung makes through Git;
/// the rename replaces nothing (`RENAME_NOREPLACE`),
///  so an index another writer made in the meantime is neither replaced nor removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publishIndexUnderAName(admin, held, 'index.pending'); // throws a reason
/// ```
pub fn publish_index_under_a_name(admin: &std::fs::File, held: &std::fs::File, name: &OsStr) -> Result<(), String> {
    use crate::fs_beneath::{entry_metadata_at, identity_of, rename_at};
    let index: &OsStr = OsStr::new("index");
    match entry_metadata_at(admin, index) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("the index cannot be examined ({error})")),
        Ok(_) => return Err(String::from("an index appeared in the new worktree while it was made")),
    }
    if let Err(error) = rename_at(admin, name, admin, index, false) {
        return Err(format!("the new index cannot be put in place ({error})"));
    }
    return match identity_of(held) {
        Ok(ours) => {
            let _removed = crate::fs_beneath::remove_if_identical(admin, name, ours);
            Ok(())
        }
        Err(error) => Err(format!("the published index cannot be examined ({error})")),
    };
}

/// What:
///  Remove the index written under `name`,
/// if the name still holds the file that was written there.
/// Why:
///   A rung that ends after the index was written under its own name leaves that file behind,
///  and Git's own checkout writes the worktree's `index` from scratch,
/// so the file is this run's to remove and nothing else's;
/// a name another process took is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// discardIndexUnderAName(admin, held, 'index.pending');
/// ```
pub fn discard_index_under_a_name(admin: &std::fs::File, held: &std::fs::File, name: &OsStr) {
    let Ok(ours) = crate::fs_beneath::identity_of(held) else { return };
    let _removed = crate::fs_beneath::remove_if_identical(admin, name, ours);
}

/// On a failure the name this install made is ours to remove,
/// but only if it still holds the file made here;
/// the error is `reason`.
fn give_up<T>(admin: &std::fs::File, name: &OsStr, ours: crate::fs_beneath::ObjectId, reason: String) -> Result<T, String> {
    let _kept = crate::fs_beneath::remove_if_identical(admin, name, ours);
    return Err(reason);
}
