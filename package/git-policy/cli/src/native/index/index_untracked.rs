//! What:
//!  The `UNTR` (untracked cache) index extension as a tree of directories,
//! and its invalidation along a path
//!  (`dir.c`:
//! `invalidate_one_component`,
//! `lookup_untracked`,
//! `invalidate_one_directory`).
//! Why:
//!  Git clears the cached listing of the directory of every path it adds or removes in the index,
//! and past it up to the top
//!  when the cache lists whole untracked directories;
//! an in-process index update must clear exactly those,
//! creating the
//!  directories along the path that the cache did not know,
//! as `lookup_untracked` does.
//! Reading and writing the extension
//!  are in `index_untracked_io.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! cache.invalidatePath(path); // clears `valid` and the listing of each affected directory
//! ```

use crate::index_byte_search::first_index_of_byte;
use crate::index_stat::StatData;

/// The directory-flags bit that makes the cache list untracked directories as a whole (`DIR_SHOW_OTHER_DIRECTORIES`).
pub const SHOW_OTHER_DIRECTORIES: u32 = 1 << 1;

/// One directory of the cache.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UntrackedDir {
    /// The directory's name,
    /// without a slash;
    /// empty for the top.
    pub name: Vec<u8>,
    /// Whether the directory is written out;
    /// a directory made while invalidating a path is not.
    pub recurse: bool,
    /// Whether the cached listing and `stat` are trusted.
    pub valid: bool,
    /// Whether the directory was listed only to see whether it holds anything untracked.
    pub check_only: bool,
    /// The directory's stat data when `valid`.
    pub stat: StatData,
    /// The object name of its ignore file,
    /// all zero when none.
    pub exclude_id: Vec<u8>,
    /// The untracked files listed in it.
    pub untracked: Vec<Vec<u8>>,
    /// The subdirectories in the byte order of their names.
    pub dirs: Vec<UntrackedDir>,
}

/// The whole extension.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Untracked {
    /// The identification of the machine that made the cache.
    pub ident: Vec<u8>,
    /// Stat data and object name of `.git/info/exclude`.
    pub info_exclude: (StatData, Vec<u8>),
    /// Stat data and object name of the excludes file.
    pub excludes_file: (StatData, Vec<u8>),
    /// The directory flags the cache was made with.
    pub dir_flags: u32,
    /// The per-directory ignore file name (`.gitignore`).
    pub exclude_per_dir: Vec<u8>,
    /// The top directory;
    /// none until a status run has listed the worktree.
    pub root: Option<UntrackedDir>,
}

impl UntrackedDir {
    /// A directory `lookup_untracked` makes:
    /// unknown,
    /// so neither valid nor written.
    fn created(name: &[u8], hash_length: usize) -> UntrackedDir {
        return UntrackedDir {
            name: name.to_vec(),
            recurse: false,
            valid: false,
            check_only: false,
            stat: StatData::default(),
            exclude_id: vec![0; hash_length],
            untracked: Vec::new(),
            dirs: Vec::new(),
        };
    }

    /// Forget the listing of this directory (`invalidate_one_directory`).
    fn invalidate_one(&mut self) {
        self.valid = false;
        self.untracked.clear();
    }

    /// The position of the subdirectory `name`,
    /// or the position to insert it at,
    /// by the same halving as `lookup_untracked`.
    fn position_of(&self, name: &[u8]) -> Result<usize, usize> {
        let (mut first, mut last): (usize, usize) = (0, self.dirs.len());
        while last > first {
            let next: usize = first + ((last - first) >> 1);
            match name.cmp(self.dirs[next].name.as_slice()) {
                std::cmp::Ordering::Equal => return Ok(next),
                std::cmp::Ordering::Less => last = next,
                std::cmp::Ordering::Greater => first = next + 1,
            }
        }
        return Err(first);
    }

    /// Invalidate along `path`;
    /// whether the parent directory must be invalidated too (`invalidate_one_component`).
    fn invalidate_component(&mut self, path: &[u8], show_other: bool, hash_length: usize) -> bool {
        let Some(slash) = first_index_of_byte(path, b'/') else {
            self.invalidate_one();
            return show_other;
        };
        let at: usize = match self.position_of(&path[..slash]) {
            Ok(found) => found,
            Err(insert) => {
                self.dirs.insert(insert, UntrackedDir::created(&path[..slash], hash_length));
                insert
            }
        };
        let below: bool = self.dirs[at].invalidate_component(&path[slash + 1..], show_other, hash_length);
        if below {
            self.invalidate_one();
        }
        return below;
    }
}

impl Untracked {
    /// Clear the cached listings a change of the index entry at `path` makes stale (`untracked_cache_invalidate_path`).
    pub fn invalidate_path(&mut self, path: &[u8]) {
        let show_other: bool = self.dir_flags & SHOW_OTHER_DIRECTORIES != 0;
        let hash_length: usize = self.info_exclude.1.len();
        if let Some(root) = self.root.as_mut() {
            root.invalidate_component(path, show_other, hash_length);
        }
    }
}

#[cfg(test)]
#[path = "index_untracked_tests.rs"]
mod tests;
