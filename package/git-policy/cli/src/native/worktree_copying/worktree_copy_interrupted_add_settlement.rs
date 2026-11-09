//! What:
//!  What recovery settles an interrupted `git worktree add` to:
//! the record it keeps,
//! the notice it gives,
//! and the proof that decides whether the checkout it left is whole or gone.
//! Why:
//!  A record that cannot be read and a checkout that is not proven whole keep the record,
//! so the next command reports the same obligation again;
//! only a destination that is gone,
//! or one whose index and empty worktree together prove the add finished,
//! lets the record go.
//! The proof reads Git's own pointer and index rather than running Git,
//! because recovery never runs a Git command and never waits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { notice, keep } = await reportInterruptedAdd(record);
//! const whole = !exists(destination) || (exists(join(admin, 'index')) && !exists(join(admin, 'index.lock')) && (entryCount(index) === 0 || entries(destination).length > 1))
//! ```

use crate::json_record::quote;
use crate::worktree_add_binding::{Recorded, decode_record};
use crate::worktree_copy_stage_claim::COMPLETE_SUFFIX;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// What recovery decided about one add's record.
pub struct Settlement {
    /// What to tell the person,
    /// if anything.
    pub notice: Option<String>,
    /// Whether the record and the claim stay because the state is unknown or the add is not proven whole,
    /// so the
    ///  next command reports it again.
    pub keep: bool,
}

/// The name of the lock Git takes to write an index,
/// in the administrative directory of a worktree.
pub(crate) const INDEX_LOCK: &str = "index.lock";

/// The name of the index itself,
/// in the administrative directory of a worktree.
pub(crate) const INDEX: &str = "index";

/// The name Git gives the pointer file that makes a directory a worktree.
const POINTER: &str = ".git";

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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const header = readSync(index).subarray(0, 12); return header.toString('latin1', 0, 4) === 'DIRC' ? header.readUInt32BE(8) : undefined;
/// ```
fn index_entry_count(index: &Path) -> Option<usize> {
    let mut header = [0_u8; INDEX_HEADER_BYTES];
    let mut file = std::fs::File::open(index).ok()?;
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
fn add_is_finished(destination: &Path, admin: &Path) -> bool {
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

/// What:
///  The administrative directory the destination's `.git` file names,
/// when that directory exists.
/// Why:
///   A record written before the add was bound carries only the destination,
/// so the pointer Git wrote is what names the
///  registration and the index the proof reads;
/// a pointer that is not a `gitdir:` line,
/// or one whose target is
///  not there,
/// proves nothing and is no registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = readFileSync(join(destination, '.git'), 'utf8'); return line.startsWith('gitdir: ') ? resolve(destination, line.slice(8).trim()) : undefined;
/// ```
pub(crate) fn registration_directory(destination: &Path) -> Option<PathBuf> {
    // Git's own reader, on bytes (`worktree_add_binding::pointer_target`): the rest of the file after the prefix, with only
    //  its trailing newlines removed, so a path holding a newline is whole and the remedy never names another repository.
    let pointer: Vec<u8> = std::fs::read(destination.join(POINTER)).ok()?;
    let resolved: PathBuf = crate::worktree_add_binding::pointer_target(destination, pointer.as_slice()).ok()?;
    return resolved.is_dir().then_some(resolved);
}

/// What:
///  What finishes or removes an interrupted add,
/// with the stale lock named first when one stands in the way.
/// Why:
///   `git -C <destination> reset --hard` cannot start while that lock exists,
/// and Git refuses it with
/// `fatal: Unable to create '.../index.lock': File exists`;
/// so the report names the file and the condition for removing it;
///  the worktree's own `git status` still works,
/// so nothing else would tell a person why every write to it fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = join(admin, 'index.lock')
/// return exists(lock) ? lockedSentence(lock, finish) : finish
/// ```
pub(crate) fn finish_instructions(destination: &Path, admin: &Path) -> String {
    // Both paths are read by the person's shell, so they are quoted the way a shell reads a word back, not the way JSON
    //  spells a string: `$bucket` and a backquote inside double quotes would be expanded by the shell that runs `rm`.
    let shown: String = crate::shell_quote::shell_quote(destination.as_os_str().as_bytes());
    let finish: String = format!("`git -C {shown} reset --hard` completes its checkout, and `git worktree remove --force {shown}` removes it.");
    let lock: PathBuf = admin.join(INDEX_LOCK);
    if !lock.exists() {
        return finish;
    }
    let lock_shown: String = crate::shell_quote::shell_quote(lock.as_os_str().as_bytes());
    return format!("Its Git directory holds the lock {lock_shown} that a Git killed inside the index install left, and every Git write to the worktree fails while it stands: once no Git command is running there, `rm {lock_shown}` removes it, and then {finish}");
}

/// What:
///  Report an add that was interrupted before it was marked complete,
/// and change nothing.
/// Why:
///   Recovery never deletes or unregisters a worktree:
/// a path,
/// a registration or a name can be changed by another process
///  between any check and any act,
/// so nothing the wrapper recorded earlier is a right to remove something later.
/// The report
///  names the worktree and the two commands that finish or remove it,
/// as Git's own interrupted add leaves it.
/// A record that
///  cannot be read,
/// and a marker that cannot be looked up,
/// are not "incomplete" and not "complete":
/// the record stays and the
///  report says that the state is unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { notice, keep } = await reportInterruptedAdd(record);
/// ```
pub fn settle_interrupted_add(record: &Path) -> Settlement {
    let marker: std::io::Result<bool> = match std::fs::symlink_metadata(record.with_extension(&COMPLETE_SUFFIX[1..])) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    };
    // The record is read only when the marker is absent, as the lazy read of `settle_from` did.
    if let Some(settled) = settle_marker(record, marker) {
        return settled;
    }
    return settle_text(record, std::fs::read_to_string(record));
}

/// What:
///  The settlement the completion marker alone decides,
/// or nothing when the record must be read.
/// Why:
///   A marker means the add finished,
/// so its record is dropped;
/// a marker that cannot be checked keeps the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (marker === true) return { keep: false }; if (marker instanceof Error) return { keep: true, notice }; return undefined;
/// ```
pub fn settle_marker(record: &Path, marker: std::io::Result<bool>) -> Option<Settlement> {
    let shown_record: String = quote(record.to_string_lossy().as_ref());
    return match marker {
        Ok(true) => Some(Settlement { notice: None, keep: false }),
        Ok(false) => None,
        Err(error) => Some(Settlement {
            notice: Some(format!("monochromatic-git-policy-cli: whether the `git worktree add` of {shown_record} completed cannot be checked ({error}); nothing was changed and the record stays.")),
            keep: true,
        }),
    };
}

/// What:
///  The settlement of an interrupted add from what its record said,
/// or the failure to read it.
/// Why:
///   A record that is gone,
/// unreadable,
/// planned or registered each ends differently,
/// and none deletes what is not proven.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settlement = settleText(record, await readFile(record).catch((error) => error));
/// ```
pub fn settle_text(record: &Path, read: std::io::Result<String>) -> Settlement {
    let shown_record: String = quote(record.to_string_lossy().as_ref());
    let text: String = match read {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Settlement { notice: None, keep: false },
        Err(error) => {
            return Settlement {
                notice: Some(format!("monochromatic-git-policy-cli: the record {shown_record} of an interrupted `git worktree add` cannot be read ({error}); nothing was changed and the record stays.")),
                keep: true,
            };
        }
    };
    return match decode_record(text.as_str()) {
        None => Settlement {
            notice: Some(format!("monochromatic-git-policy-cli: the record {shown_record} of an interrupted `git worktree add` is unreadable; nothing was changed.")),
            keep: false,
        },
        Some(Recorded::Planned(destination)) => {
            // The path is printed inside commands the person runs, so it is quoted for the shell.
            // The path is printed inside commands the person runs, so it is quoted for the shell.
            let shown: String = crate::shell_quote::shell_quote(destination.as_os_str().as_bytes());
            // Nothing at the path, or nothing Git's pointer there: the add made nothing that is left to finish.
            if !destination.join(POINTER).exists() {
                return Settlement { notice: None, keep: false };
            }
            // A kill between real Git's registration and the wrapper's record of the binding leaves the pointer without the
            //  plan, so the pointer is what names the administrative directory the proof reads.
            let registered: Option<PathBuf> = registration_directory(destination.as_path());
            if let Some(admin) = registered.as_ref()
                && add_is_finished(destination.as_path(), admin.as_path())
            {
                return Settlement { notice: None, keep: false };
            }
            let notice: String = match registered.as_ref() {
                Some(admin) => format!(
                    "monochromatic-git-policy-cli: an interrupted `git worktree add` left {shown} registered and possibly incomplete; it was not changed. {}",
                    finish_instructions(destination.as_path(), admin)
                ),
                None => format!("monochromatic-git-policy-cli: an interrupted `git worktree add` may have left {shown}; nothing proves it is the add's, so it was not changed. `git worktree list` shows whether it is registered, and `git worktree remove --force {shown}` removes it."),
            };
            // The record stays while something stands at the recorded path that is not proven whole, so every later command
            //  that runs the recovery says the same instead of leaving the obligation in one command's output.
            Settlement { notice: Some(notice), keep: true }
        }
        Some(Recorded::Registered(binding)) => {
            let shown: String = crate::shell_quote::shell_quote(binding.destination.as_os_str().as_bytes());
            // A worktree that is whole or gone settles silently; everything else keeps its record so the next command says
            //  the same again, because nothing else names the worktree, the stage or what finishes them.
            if add_is_finished(binding.destination.as_path(), binding.admin.as_path()) {
                return Settlement { notice: None, keep: false };
            }
            Settlement {
                notice: Some(format!(
                    "monochromatic-git-policy-cli: an interrupted `git worktree add` left {shown} registered and possibly incomplete; it was not changed. {}",
                    finish_instructions(binding.destination.as_path(), binding.admin.as_path())
                )),
                keep: true,
            }
        }
    };
}
