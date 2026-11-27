//! What:
//!  The source a landing names,
//!  which proves many paths against one worktree held open,
//!  and the two entry functions that prove one path.
//! Why:
//!  A held directory makes every look-up below it immune to a swap of an
//!  ancestor,
//!  which a path built from the worktree root is not,
//!  and the attribute files of each directory and the read budget are kept
//!  here,
//!  so a landing proves every path of one commit against the same worktree,
//!  the same attributes and the same bound,
//!  and no closure carries either side's state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const source = new LandingStat({ root, target, stamp, hashed, indexedAttributes });
//! const stat = source.recorded(path, blob, mode);
//! ```

/// The proofs a landing makes about one file,
/// and the object name it compares.
use super::index_reset_worktree_stat_proofs::{
    blob_name,
    mode_agrees,
    read_by_chunks,
    same_state,
    stamp_proves_one_version,
    stat_of,
};
/// The stat fields an index entry records,
/// and the request and seams one path is asked with.
use super::index_reset_worktree_stat_request::{
    ProofSeams,
    do_nothing,
    read_some,
};
/// The trait a proof implements for the reset's writer.
use super::index_reset_worktree_stat_source::WorktreeStatSource;
/// The held directories,
/// the files opened below them and the components of one path.
use crate::fs_beneath::{
    entry_metadata_at,
    open_file_at,
    plain_components,
};
/// The directory opened below the held root without following a link.
use crate::fs_beneath_walk_chain::open_directory_beneath;
/// The stat fields and the index file's date.
use crate::index_stat::{
    IndexStamp,
    StatData,
};
/// The object hash of the repository.
use crate::repo_read_layout::ObjectFormat;
/// The content decision's judge and the bytes one landing may read.
use crate::worktree_clean_content::{
    CONTENT_LIMIT,
    ContentJudge,
};
/// The worktree,
/// its Git directories and the attributes Git would apply.
use crate::worktree_clean_tree_walk::ContentTarget;
/// The attribute files a landing reads,
/// and the read budget it charges them to.
use std::cell::{
    Cell,
    RefCell,
};
/// The names of a path's components and of one directory's `.gitattributes`.
use std::ffi::{
    OsStr,
    OsString,
};
/// A path's bytes,
/// which the index spells `/` separated.
use std::os::unix::ffi::OsStrExt;
/// The change and modification times and the file kind a look answers.
use std::os::unix::fs::MetadataExt;
/// The path whose bytes are read from the held root.
use std::path::Path;
/// What:
///  The content decision's judge for a landing,
/// with its attribute reads charged to the landing's bound.
/// Why:
///   The attributes a path's bytes are judged against are read under the landing's index lock,
/// so those bytes are part of
///  what the landing bounds,
/// and a file that does not fit is refused (a refresh) rather than read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judge = new ContentJudge({ root, gitDir, commonDir, environment, format }); judge.chargeAttributesTo(hashed);
/// ```
fn charged_judge<'a>(
    root: &'a std::fs::File,
    target: &ContentTarget<'a>,
    hashed: &'a Cell<u64>,
) -> ContentJudge<'a> {
    let mut judge: ContentJudge<'a> = ContentJudge::new(
        root,
        target.git_dir,
        target.common_dir,
        target.environment,
        target.format,
    );
    judge.charge_attributes_to(hashed);
    return judge;
}

/// What:
///  The held directory out of the cached entry.
/// Why:
///   The cache holds the names it was opened for beside the handle,
/// and only the handle is read out of the entry.
/// It is duplicated for the caller there,
/// so no borrow of the cache outlives the look-up that asked for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theDirectory = (held) => held?.opened;
/// ```
fn the_directory(held: &Option<(Vec<OsString>, std::fs::File)>) -> Option<&std::fs::File> {
    let (_names, opened) = held.as_ref()?;
    return Some(opened);
}

/// What:
///  The source a landing names:
/// the worktree held open,
/// the content decision's own attributes,
/// the object hash and the date of the index file.
/// Why:
///   A landing proves many paths against one worktree,
/// so the held root,
/// the attribute files of each directory and the read
///  budget are kept here,
/// and no closure carries either side's state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = new LandingStat({ root, target, stamp });
/// ```
pub struct LandingStat<'a> {
    /// The worktree's top level,
    /// held open.
    root: &'a std::fs::File,
    /// The attributes Git would apply to a path,
    /// which decide whether a conversion may change its bytes.
    judge: RefCell<ContentJudge<'a>>,
    /// The directories opened so far:
    /// the names below the root and the held directory,
    /// because the entries come in path order and most share one.
    directory: RefCell<Option<(Vec<OsString>, std::fs::File)>>,
    /// The object hash of the repository.
    format: ObjectFormat,
    /// The date of the index file the entry is written into.
    stamp: IndexStamp,
    /// The bytes this landing has read so far.
    hashed: &'a Cell<u64>,
    /// The directories whose `.gitattributes` the index holds.
    indexed_attributes: &'a [Vec<u8>],
}

impl<'a> LandingStat<'a> {
    /// The source of the worktree held open as `root`,
    /// whose index file is dated `stamp`.
    pub fn new(
        root: &'a std::fs::File,
        target: &ContentTarget<'a>,
        stamp: IndexStamp,
        hashed: &'a Cell<u64>,
        indexed_attributes: &'a [Vec<u8>],
    ) -> LandingStat<'a> {
        return LandingStat {
            root,
            judge: RefCell::new(charged_judge(root, target, hashed)),
            directory: RefCell::new(None),
            format: target.format,
            stamp,
            hashed,
            indexed_attributes,
        };
    }

    /// What:
    ///  The directory holding the entry at `parents`,
    /// opened below the held root without following a link and kept for the next entry,
    /// as a handle of the caller's own.
    /// Why:
    ///   A held directory makes every look-up below it immune to a swap of an ancestor,
    /// which a path built from the
    ///  worktree root is not.
    /// The handle is duplicated for the caller rather than lent out of the cache:
    /// this method replaces the cache whenever another directory is asked for,
    /// and a borrow of the cache alive across that call panics
    ///  (`RefCell` holds one mutable borrow,
    /// and the third review found exactly that with the root and an ancestor's `.gitattributes`).
    /// A duplicate costs one `dup` a path and cannot outlive the call it is made for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const directory = (cache.get(parents) ?? openBelow(heldRoot, parents)).dup();
    /// ```
    fn directory(
        &self,
        parents: &[OsString],
    ) -> Option<std::fs::File> {
        let missed: bool = match self.directory.borrow().as_ref() {
            Some((known, _opened)) => known.as_slice() != parents,
            None => true,
        };
        if missed {
            let opened: std::fs::File = open_directory_beneath(self.root, parents).ok()?;
            self.directory.replace(Some((parents.to_vec(), opened)));
        }
        let cache: std::cell::Ref<'_, Option<(Vec<OsString>, std::fs::File)>> =
            self.directory.borrow();
        let cached: &std::fs::File = the_directory(&cache)?;
        return cached.try_clone().ok();
    }

    /// What:
    ///  The bytes of the regular file the entry names,
    /// read at most up to `room`,
    /// with what was read charged to the landing whatever becomes of it.
    /// Why:
    ///   A read that is later refused still reached the disk,
    /// so the bound is on the reading;
    /// the file is opened without waiting (`O_NONBLOCK`,
    /// `fs_beneath::open_file_at`),
    /// so a fifo
    ///  renamed over the path cannot hold the landing's index lock;
    /// the handle is judged by its type and by the inode the first look saw,
    /// so a swap
    ///  between the two is refused.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function readCharged(directory, name, before, room) {
    ///   seams.beforeOpen(); const file = openFileAt(directory, name); const held = fstat(file);
    ///   if (!held.isFile() || held.ino !== before.ino) return; const data = readAtMost(file, room);
    ///   hashed += data === undefined ? room : data.length; if (data === undefined) return;
    ///   seams.afterRead(); ... stability checks ...; return data;
    /// }
    /// ```
    fn read_charged(
        &self,
        directory: &std::fs::File,
        name: &OsStr,
        before: &std::fs::Metadata,
        room: u64,
        seams: &mut ProofSeams<'_>,
    ) -> Option<(Vec<u8>, std::fs::Metadata)> {
        (seams.before_open)();
        let file: std::fs::File = open_file_at(directory, name).ok()?;
        let held: std::fs::Metadata = file.metadata().ok()?;
        if !held.file_type().is_file() || held.dev() != before.dev() || held.ino() != before.ino() {
            return None;
        }
        // A file whose change time has no nanoseconds is on a file system that cannot tell two rewrites in one second
        // apart, so no state it reports can prove the bytes were one version (a trial review's seventh finding).
        if !stamp_proves_one_version((held.ctime(), held.ctime_nsec())) {
            return None;
        }
        // A file whose handle already reports more than the room the landing has is not read at all:
        // the size the handle gives decides, not an earlier look at the path.
        if held.len() > room {
            return None;
        }
        let data: Vec<u8> = read_by_chunks(&file, room, self.hashed, seams)?;
        (seams.after_read)();
        let after: std::fs::Metadata = file.metadata().ok()?;
        let by_name: std::fs::Metadata = entry_metadata_at(directory, name).ok()?;
        if !same_state(&held, &after) || !same_state(&held, &by_name) {
            return None;
        }
        return Some((data, after));
    }

    /// What:
    ///  Whether a directory whose `.gitattributes` the index holds has the worktree's copy.
    /// Why:
    ///   Git reads the index's copy when the worktree's is missing (`attr.c`,
    /// `prepare_attr_stack` falls from the file to the index),
    /// measured on Git 2.56.0 on 2026-11-09:
    /// `git check-attr filter -- a.txt` still answers from the index after the worktree's `.gitattributes` is
    ///  removed,
    /// and a sparse checkout that leaves the file outside its cone reads the index the same way.
    /// A proof that reads only the worktree sees no attributes there,
    /// so the path keeps the zero stat data instead.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const attributesMissing = (parents) => !exists(join(heldDirectory(parents), '.gitattributes'));
    /// ```
    fn attributes_missing(
        &self,
        parents: &[OsString],
    ) -> Option<bool> {
        let directory: std::fs::File = self.directory(parents)?;
        return Some(entry_metadata_at(&directory, OsStr::new(".gitattributes")).is_err());
    }

    /// What:
    ///  The refusal for a path whose attributes could come from the index because the worktree's copy is gone.
    /// Why:
    ///   Refusing costs a refresh and recording wrongly hides an edit,
    /// so a directory the index can supply attributes for is checked from the root
    ///  down before any of its files is trusted.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const directory of ancestors(parents)) if (indexed.has(directory) && attributesMissing(directory)) return undefined;
    /// ```
    fn attributes_come_only_from_the_index(
        &self,
        parents: &[OsString],
    ) -> bool {
        if self.indexed_attributes.is_empty() {
            return false;
        }
        if self.indexed_attributes.binary_search(&Vec::new()).is_ok()
            && matches!(self.attributes_missing(&[]), None | Some(true))
        {
            return true;
        }
        for depth in 1..=parents.len() {
            let mut prefix: Vec<u8> = Vec::new();
            for name in &parents[..depth] {
                if !prefix.is_empty() {
                    prefix.push(b'/');
                }
                prefix.extend_from_slice(name.as_os_str().as_bytes());
            }
            if self.indexed_attributes.binary_search(&prefix).is_err() {
                continue;
            }
            if matches!(
                self.attributes_missing(&parents[..depth]),
                None | Some(true)
            ) {
                return true;
            }
        }
        return false;
    }

    /// What:
    ///  The stat data the landing may record for one path,
    /// or nothing when the file cannot prove it holds the committed bytes.
    /// Why:
    ///  `index_reset.rs` calls this for every entry it writes,
    /// and the answer decides between the file's stat data and the all-zero
    ///  value `git reset --no-refresh` writes,
    /// so every doubt keeps the zero value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stat = source.recorded(path, blob, mode);
    /// ```
    pub(super) fn record(
        &self,
        path: &[u8],
        blob: &[u8],
        mode: u32,
        seams: &mut ProofSeams<'_>,
    ) -> Option<StatData> {
        let names: Vec<OsString> = plain_components(Path::new(OsStr::from_bytes(path))).ok()?;
        let (name, parents): (&OsString, &[OsString]) = names.split_last()?;
        let directory: std::fs::File = self.directory(parents)?;
        let before: std::fs::Metadata = entry_metadata_at(&directory, name.as_os_str()).ok()?;
        if !mode_agrees(mode, &before) || self.stamp.is_racy(&stat_of(&before)) {
            return None;
        }
        // A conversion Git may apply can leave these bytes looking equal to the blob while Git's view of the file is not.
        if self.attributes_come_only_from_the_index(parents) {
            return None;
        }
        if self
            .judge
            .borrow_mut()
            .conversion_is_the_identity(path, parents)
            .is_err()
        {
            return None;
        }
        let room: u64 = CONTENT_LIMIT.saturating_sub(self.hashed.get());
        if before.len() > room {
            return None;
        }
        let (data, examined): (Vec<u8>, std::fs::Metadata) =
            self.read_charged(&directory, name.as_os_str(), &before, room, seams)?;
        // A line-ending rule Git applies whatever the attributes say may strip a carriage return on the way into the index.
        if data.contains(&b'\r') || blob_name(data.as_slice(), self.format) != blob {
            return None;
        }
        return Some(stat_of(&examined));
    }
}

impl WorktreeStatSource for LandingStat<'_> {
    fn recorded(
        &self,
        path: &[u8],
        blob: &[u8],
        mode: u32,
    ) -> Option<StatData> {
        return self.record(
            path,
            blob,
            mode,
            &mut ProofSeams {
                before_open: &mut do_nothing,
                after_read: &mut do_nothing,
                read: &mut read_some,
            },
        );
    }
}
