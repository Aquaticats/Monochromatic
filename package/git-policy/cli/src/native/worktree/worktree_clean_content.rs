//! What:
//!  The decision,
//! by content and without a Git process,
//! whether a tracked file that its index entry's stat data cannot
//!  vouch for (a racily clean entry,
//! a stale stat) still equals the committed blob:
//! hash the file as a blob and compare it
//!  with the entry's object name,
//! refusing where a content conversion could make the two differ.
//! Why:
//!  Git settles those entries by hashing the file;
//! stat data alone cannot tell a modified file from a copied or cloned one
//!  that kept the index's old inode and times.
//! The hash of the raw bytes is conclusive only when Git's own conversion on the
//!  way into the index (`filter`,
//! `ident`,
//! `working-tree-encoding`,
//! or a line-ending conversion that strips a carriage
//!  return) leaves the bytes alone,
//! so a path such an attribute applies to,
//! and a file with a carriage return,
//! are refused
//!  with that reason instead of guessed.
//! The file is read through a handle opened below the worktree root without following a
//!  link,
//! its type and mode are judged on that handle,
//! and its state is compared before and after the read,
//! so the bytes that
//!  were hashed are the bytes of the one file that was judged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const judge = new ContentJudge({ root, gitDir, commonDir, environment, format });
//! judge.agrees({ directory, parents, name, entry, id, indexSeconds }); // throws the reason the file cannot be proven equal
//! ```

use crate::fs_beneath::{entry_metadata_at, open_file_at, read_link_at};
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_sha::{sha1, sha256};
pub use crate::worktree_clean_content_attribute_sources::{ConversionEnvironment, SystemAttributes, system_attributes_from};
use crate::worktree_clean_content_attribute_sources::{AttributeFile, FixedSources};
use crate::worktree_clean_stat_verdicts::{StatEntry, StatVerdict, metadata_verdict};
use crate::worktree_path_failure_message::subject_failure;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The largest file read into memory to be hashed;
/// a larger one is refused with that reason.
pub const CONTENT_LIMIT: u64 = 32 << 20;

/// The largest attributes file read;
/// Git ignores a larger one,
/// which here is a refusal.
pub const ATTRIBUTES_LIMIT: u64 = 4 << 20;

/// The object name of `data` stored as a blob.
fn blob_name(data: &[u8], format: ObjectFormat) -> Vec<u8> {
    let mut framed: Vec<u8> = format!("blob {}\0", data.len()).into_bytes();
    framed.extend_from_slice(data);
    return match format {
        ObjectFormat::Sha1 => sha1(framed.as_slice()).to_vec(),
        ObjectFormat::Sha256 => sha256(framed.as_slice()).to_vec(),
    };
}

/// What:
///  Everything `reader` yields,
/// or nothing when it yields more than `limit` bytes;
/// it is never asked for more than
///  `limit + 1`.
/// Why:
///   A file can grow after its size was looked at;
/// the bound is on the read,
/// not on an earlier look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readAtMost(reader: Reader, limit: number): Buffer | undefined;
/// ```
pub fn read_at_most(reader: impl Read, limit: u64) -> std::io::Result<Option<Vec<u8>>> {
    let mut data: Vec<u8> = Vec::new();
    reader.take(limit + 1).read_to_end(&mut data)?;
    return Ok(if data.len() as u64 > limit { None } else { Some(data) });
}

/// Whether two metadata describe one unchanged file:
/// same object,
/// size,
/// mode and times.
fn same_state(first: &std::fs::Metadata, second: &std::fs::Metadata) -> bool {
    return first.dev() == second.dev()
        && first.ino() == second.ino()
        && first.size() == second.size()
        && first.mode() == second.mode()
        && first.mtime() == second.mtime()
        && first.mtime_nsec() == second.mtime_nsec()
        && first.ctime() == second.ctime()
        && first.ctime_nsec() == second.ctime_nsec();
}

/// What:
///  The refusal for a file that changed while it was examined.
/// Why:
///   A file that moves under the check cannot be proven equal to the committed content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${shown} changed while it was examined, so it cannot be proven equal to the committed content`
/// ```
fn changed_while_examined(shown: &str) -> String {
    return format!("{shown} changed while it was examined, so it cannot be proven equal to the committed content");
}

/// What:
///  The refusal for a file larger than the content check reads.
/// Why:
///   A bound on the bytes read keeps one huge file from holding the add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${shown} is larger than ${CONTENT_LIMIT} bytes, ...`
/// ```
fn too_large_to_compare(shown: &str) -> String {
    return format!("{shown} is larger than {CONTENT_LIMIT} bytes, which is more than is read to compare it by content");
}

/// What:
///  The refusal for attributes Git takes from a tree and not from the files read here,
/// with the reason it does.
/// Why:
///   What such a tree sets cannot be ruled out by reading files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${reason}, so Git takes its attributes from a tree ...`
/// ```
pub(super) fn unmodelled(reason: &str) -> String {
    return format!("{reason}, so Git takes its attributes from a tree and not from the files read here, and what it sets cannot be ruled out");
}

/// What:
///  Do nothing;
/// the seams of the content read run this unless a control puts work there.
/// Why:
///   The real read has nothing to do between its steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const doNothing = () => {};
/// ```
fn do_nothing() {}

/// What:
///  Decides,
/// one entry at a time,
/// whether a file equals its committed blob by content.
/// Why:
///   The attributes files are read once,
/// on the first entry that needs them,
/// and the per-directory ones are cached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ContentJudge { agrees(args: Place & { entry: StatEntry; id: Uint8Array; indexSeconds: number }): void /* throws the reason */ }
/// ```
pub struct ContentJudge<'a> {
    /// The worktree root,
    /// held open.
    pub(super) root: &'a std::fs::File,
    /// The worktree's Git directory.
    pub(super) git_dir: &'a Path,
    /// The shared Git directory.
    pub(super) common_dir: &'a Path,
    /// Which files Git reads in this environment.
    pub(super) environment: &'a ConversionEnvironment,
    /// The object format of the repository.
    format: ObjectFormat,
    /// The attributes files that do not depend on the path.
    pub(super) fixed: Option<Result<FixedSources, String>>,
    /// The `.gitattributes` of each directory,
    /// keyed by its path,
    /// read at most once;
    /// `None` when it has none.
    pub(super) directories: HashMap<Vec<u8>, Option<AttributeFile>>,
    /// The bytes a caller has read so far,
    /// when it bounds what it reads (`CONTENT_LIMIT` shared with the landing),
    /// so the attributes read here are part of that bound.
    pub(super) budget: Option<&'a std::cell::Cell<u64>>,
    /// The read an attributes file is taken with,
    /// or nothing for the operating system's own;
    /// a control sets one that hands over the bytes of a file that grew under the read.
    pub(super) attribute_read: Option<&'a dyn Fn(&std::fs::File, &mut [u8]) -> std::io::Result<usize>>,
}

/// The moments of a judgment at which a control changes the file under it.
pub struct ReadSeams<'a> {
    /// Called after the file's state was first read and before it is opened.
    pub before_open: &'a mut dyn FnMut(),
    /// Called after the open handle was judged and before the bytes are read.
    pub before_read: &'a mut dyn FnMut(),
    /// Called after the bytes were read and before the state is read again.
    pub after_read: &'a mut dyn FnMut(),
}

/// Where a tracked file is:
/// the held directory that contains it,
/// the names above it and its own name.
pub struct Place<'a> {
    /// The directory holding the file,
    /// opened below the root without following a link.
    pub directory: &'a std::fs::File,
    /// The names of the directories between the root and the file.
    pub parents: &'a [OsString],
    /// The file's own name.
    pub name: &'a OsStr,
}

impl<'a> ContentJudge<'a> {
    /// A judge for the worktree held open as `root`.
    pub fn new(root: &'a std::fs::File, git_dir: &'a Path, common_dir: &'a Path, environment: &'a ConversionEnvironment, format: ObjectFormat) -> ContentJudge<'a> {
        return ContentJudge { root, git_dir, common_dir, environment, format, fixed: None, directories: HashMap::new(), budget: None, attribute_read: None };
    }

    /// What:
    ///  Charge the attributes this judge reads to `spent`,
    /// and hold them inside what is left of `CONTENT_LIMIT`.
    /// Why:
    ///  The attributes of a worktree are read under the landing's index lock,
    /// so a landing bounds them with the files it proves;
    /// a worktree check that reads no files passes nothing and keeps the per-file bound.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// judge.chargeAttributesTo(spent);
    /// ```
    pub fn charge_attributes_to(&mut self, spent: &'a std::cell::Cell<u64>) {
        self.budget = Some(spent);
    }

    /// What:
    ///  Whether Git's conversion of the file at `path` is the identity on its bytes:
    /// no attribute that converts anything applies,
    /// and the attributes files could be read.
    /// Why:
    ///   A caller that records a file's stat data must be sure a later reader's trust in that data cannot hide a
    /// difference between the file and the committed bytes;
    /// a clean filter,
    /// `ident` or a `working-tree-encoding` can make
    ///  those differ while the raw bytes look equal,
    /// so such a path is refused here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// judge.conversionIsTheIdentity(path, parents); // throws the reason the conversion may change the bytes
    /// ```
    ///
    /// # Errors
    ///
    /// The reason a conversion may change these bytes,
    /// or the files it would be read from could not be.
    pub fn conversion_is_the_identity(&mut self, path: &[u8], parents: &[OsString]) -> Result<(), String> {
        let shown: String = String::from_utf8_lossy(path).into_owned();
        return match self.converting_attribute(path, parents)? {
            None => Ok(()),
            Some((token, label)) if token == "a quoted line" => Err(format!("{shown} cannot be proven unconverted: {label} has a quoted line, whose pattern and attributes are not modelled")),
            Some((token, label)) => Err(format!("{shown} cannot be proven unconverted: the attribute {token} from {label} may change its bytes on the way into the index")),
        };
    }

    /// What:
    ///  The bytes of the file at `place`,
    /// judged to be the one file its entry describes:
    /// opened below the held directory
    ///  without following a link,
    /// its type and mode judged on the open handle,
    /// and its state the same before and after.
    /// Why:
    ///   A swap,
    /// a `chmod` or a write while the file is read must refuse instead of passing,
    /// and the read is bounded.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function readStable(place: Place, entry: StatEntry, indexSeconds: number): Buffer; // throws the reason
    /// ```
    fn read_stable(place: &Place<'_>, entry: &StatEntry, index_seconds: u32, shown: &str, seams: &mut ReadSeams<'_>) -> Result<Vec<u8>, String> {
        let named: std::fs::Metadata = subject_failure(entry_metadata_at(place.directory, place.name), &shown, "cannot be examined")?;
        if let StatVerdict::Differs(reason) = metadata_verdict(entry, &named, index_seconds) {
            return Err(reason);
        }
        if named.file_type().is_symlink() {
            let target: PathBuf = subject_failure(read_link_at(place.directory, place.name), &shown, "cannot be read")?;
            let Ok(again) = entry_metadata_at(place.directory, place.name) else {
                return Err(changed_while_examined(shown));
            };
            if !same_state(&named, &again) {
                return Err(changed_while_examined(shown));
            }
            return Ok(target.into_os_string().into_vec());
        }
        (seams.before_open)();
        let file: std::fs::File = subject_failure(open_file_at(place.directory, place.name), &shown, "cannot be opened without following a link")?;
        let held: std::fs::Metadata = subject_failure(file.metadata(), &shown, "cannot be examined")?;
        if held.dev() != named.dev() || held.ino() != named.ino() {
            return Err(changed_while_examined(shown));
        }
        if let StatVerdict::Differs(reason) = metadata_verdict(entry, &held, index_seconds) {
            return Err(reason);
        }
        if held.len() > CONTENT_LIMIT {
            return Err(too_large_to_compare(shown));
        }
        (seams.before_read)();
        let Some(data) = subject_failure(read_at_most(&file, CONTENT_LIMIT), &shown, "cannot be read")? else {
            return Err(too_large_to_compare(shown));
        };
        (seams.after_read)();
        let Ok(after) = file.metadata() else {
            return Err(changed_while_examined(shown));
        };
        let Ok(by_name) = entry_metadata_at(place.directory, place.name) else {
            return Err(changed_while_examined(shown));
        };
        if !same_state(&held, &after) || !same_state(&held, &by_name) {
            return Err(changed_while_examined(shown));
        }
        return Ok(data);
    }

    /// What:
    ///  Whether the file at `place` equals the blob named `id`;
    /// the reason it cannot be proven so when it does not.
    /// Why:
    ///   Where a conversion applies,
    /// or the file has a carriage return,
    /// the bytes alone prove nothing and the answer is a
    ///  refusal that says so;
    /// a file that really differs says it differs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// judge.agrees(place, entry, id, indexSeconds); // throws the reason
    /// ```
    pub fn agrees(&mut self, place: &Place<'_>, entry: &StatEntry, id: &[u8], index_seconds: u32) -> Result<(), String> {
        return self.agrees_seamed(place, entry, id, index_seconds, &mut ReadSeams { before_open: &mut do_nothing, before_read: &mut do_nothing, after_read: &mut do_nothing });
    }

    /// `agrees` with the moments at which a control may change the file.
    pub fn agrees_seamed(&mut self, place: &Place<'_>, entry: &StatEntry, id: &[u8], index_seconds: u32, seams: &mut ReadSeams<'_>) -> Result<(), String> {
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        if let Some((token, label)) = self.converting_attribute(entry.path.as_slice(), place.parents)? {
            return Err(if token == "a quoted line" {
                format!("{shown} cannot be compared by content: {label} has a quoted line, whose pattern and attributes are not modelled, so what it sets cannot be ruled out")
            } else {
                format!("{shown} cannot be compared by content: the attribute {token} from {label} may change its bytes on the way into the index")
            });
        }
        let data: Vec<u8> = Self::read_stable(place, entry, index_seconds, shown.as_str(), seams)?;
        if blob_name(data.as_slice(), self.format) != id {
            return Err(format!("{shown} differs from the committed content (its recorded size and times could not vouch for it, so its content was compared)"));
        }
        if data.contains(&b'\r') {
            return Err(format!("{shown} cannot be compared by content: it contains a carriage return, so a line-ending conversion may change its bytes on the way into the index"));
        }
        return Ok(());
    }
}

#[cfg(test)]
#[path = "worktree_clean_content_tests.rs"]
mod tests;

/// Controls for the findings of the review of the content decision,
/// on real Git repositories.
#[cfg(test)]
#[path = "worktree_clean_content_safety_tests.rs"]
mod safety_tests;
