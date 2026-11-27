//! What:
//!  What proves the checkout of an interrupted `git worktree add` whole or gone:
//!  the destination's fate,
//!  the index Git wrote in its administrative directory,
//!  the lock a Git killed inside the index install left,
//!  and what the destination holds beside Git's pointer.
//! Why:
//!  Recovery never runs Git and never waits,
//! so only what those two directories say may settle an add:
//!  a destination that is gone is nothing to finish,
//!  an index with no entries belongs to a tree with nothing to write,
//!  and anything else keeps the record for the next command to report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const whole = !exists(destination) || (exists(join(admin, 'index')) && !exists(join(admin, 'index.lock')) && (entryCount(index) === 0 || entries(destination).length > 1));
//! ```

use std::path::Path;

/// The name of the lock Git takes to write an index,
/// in the administrative directory of a worktree.
pub(crate) const INDEX_LOCK: &str = "index.lock";

/// The name of the index itself,
/// in the administrative directory of a worktree.
pub(crate) const INDEX: &str = "index";

/// What:
///  How an index file starts:
/// its magic,
/// its version and its entry count.
/// Why:
///   The count alone answers whether a checkout had anything to write,
/// and reading it needs no object
///  name length,
/// so recovery reads a dozen bytes instead of parsing every entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = readSync(index).subarray(0, 12); // 'DIRC' | version | count
/// ```
const INDEX_HEADER_BYTES: usize = 12;

/// The index's magic at the start of its header (`include/git/read-cache-ll.h`).
const INDEX_MAGIC: [u8; 4] = *b"DIRC";

/// Where the entry count sits in the index header.
const INDEX_COUNT_AT: usize = 8;

/// What:
///  How many entries an index file holds,
/// or nothing when the file does not start like an index.
/// Why:
///   The count is the header's,
/// so a file that cannot be read or does not begin with the index magic proves nothing,
/// and a count of zero says the tree the checkout wrote has nothing in it to write into the worktree.
/// The index is Git's file in the destination's administrative directory,
/// so it is opened by the
///  plain-file reader,
/// which waits for no writer and follows no link:
/// a name left as a named pipe in a
///  directory this command did not make must end the proof at once,
/// which is a record kept,
/// never a
///  command that waits for a writer that may never come ("nothing should ever wait forever or indefinitely").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const header = openPlainFile(index).read(12); return header.toString('latin1', 0, 4) === 'DIRC' ? header.readUInt32BE(8) : undefined;
/// ```
fn index_entry_count(index: &Path) -> Option<usize> {
    let mut header = [0_u8; INDEX_HEADER_BYTES];
    let mut file = crate::plain_file::open_plain_file(index).ok()?;
    std::io::Read::read_exact(&mut file, &mut header).ok()?;
    if header[..INDEX_MAGIC.len()] != INDEX_MAGIC {
        return None;
    }
    let count: [u8; 4] = header[INDEX_COUNT_AT..INDEX_HEADER_BYTES].try_into().ok()?;
    return Some(u32::from_be_bytes(count) as usize);
}

/// What:
///  Whether the checkout at the destination is proven whole or gone,
/// which is when nothing is left for a person
///  to finish.
/// Why:
///   Recovery never runs Git and never waits,
/// so the proof is what the two directories say:
/// the destination
///  is gone,
/// its administrative directory holds an index,
/// and no `index.lock` stands in the way of the next Git
///  write there.
/// Then the index decides what a whole checkout holds beside Git's pointer:
/// an index with entries belongs to
///  a tree with files,
/// so a destination holding only its `.git` file is the shape a kill between real Git and
///  the checkout leaves;
/// an index with no entries belongs to a tree with nothing to write,
/// so a destination holding only its
///  `.git` file is whole there,
/// which is what a worktree at a commit with an empty tree looks like after
///  every step finished.
/// A lock with no index beside it is the shape a kill inside the index install leaves.
/// What the proof cannot see:
/// a process filling the destination beside an index that was written before it started
///  (another program's `git read-tree` without `-u`,
/// then a copy),
/// and the contents of the files it counts.
/// Git's
///  own `read-tree -u` and `reset --hard` hold `index.lock` for as long as they write,
/// so the lock
///  refuses those while they run,
/// and one such command writes the index last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// !exists(destination) || (exists(join(admin, 'index')) && !exists(join(admin, 'index.lock')) && (entryCount(index) === 0 || entries(destination).length > 1))
/// ```
pub(crate) fn add_is_finished(
    destination: &Path,
    admin: &Path,
) -> bool {
    // `match` on the lookup: a destination that is gone is nothing to finish, while one that cannot be examined is kept.
    match std::fs::symlink_metadata(destination) {
        Ok(_present) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return true,
        Err(_refused) => return false,
    }
    if !admin.join(INDEX).exists() || admin.join(INDEX_LOCK).exists() {
        return false;
    }
    // An index with no entries is a tree with nothing to check out, so Git's pointer alone is the whole worktree.
    if index_entry_count(admin.join(INDEX).as_path()) == Some(0) {
        return true;
    }
    // More than the `.git` file Git wrote: a checkout put something beside it.
    let held_listing = match std::fs::read_dir(destination) {
        Ok(listing) => listing,
        Err(_unreadable) => return false,
    };
    return held_listing.count() > 1;
}
