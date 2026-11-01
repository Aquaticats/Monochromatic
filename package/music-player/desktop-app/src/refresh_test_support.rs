// What:     Helpers shared by the refresh tests: a disposable library folder, a random generator, and an in-memory disk.
// Why:      The planner, the coalescer and the controller are tested over real folders (the oracle is the folder walk
//           itself) and over a million in-memory tracks (the scale test), and each needs the same small pieces.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use crate::disk_view::{Disk, EntryKind};

/// What: `TempTree` is a unique folder under the system temporary directory that is removed when it drops.
/// Why: tests change files, so they must never touch the user's music or each other's files.
pub(crate) struct TempTree {
    /// The folder every test file lives in.
    pub(crate) root: PathBuf,
}

/// Methods of `TempTree`.
impl TempTree {
    /// What: `new` makes the folder.
    /// Why: the process id, the tag and the clock make the name unique across parallel tests.
    pub(crate) fn new(tag: &str) -> TempTree {
        let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed| return elapsed.as_nanos());
        let root: PathBuf = crate::test_scratch::scratch_root().join(format!("mp-refresh-{}-{nanos}-{tag}", std::process::id()));
        std::fs::create_dir_all(&root).expect("the test folder is created");
        return TempTree { root };
    }

    /// What: `path` is the absolute path of a relative name.
    /// Why: tests talk in relative names.
    pub(crate) fn path(&self, relative: &str) -> PathBuf {
        return self.root.join(relative);
    }

    /// What: `write` creates or replaces a file with the given text, making its folders.
    /// Why: replacing a file keeps its path; the text decides its size.
    pub(crate) fn write(&self, relative: &str, text: &str) -> PathBuf {
        let path: PathBuf = self.path(relative);
        std::fs::create_dir_all(path.parent().expect("a file has a folder")).expect("the folder is created");
        std::fs::write(&path, text).expect("the file is written");
        return path;
    }

    /// What: `try_write` creates a file when its path is free: false when a file is in the way of a folder or a folder is in
    /// the way of the file.
    /// Why: random trees collide on names, and a collision is skipped, not a failure.
    pub(crate) fn try_write(&self, relative: &str, text: &str) -> bool {
        let path: PathBuf = self.path(relative);
        let Some(parent) = path.parent() else { return false };
        return std::fs::create_dir_all(parent).is_ok() && std::fs::write(&path, text).is_ok();
    }

    /// What: `walk` is the folder walk of the whole library: the oracle for every queue.
    /// Why: the queue is, by definition, this list.
    pub(crate) fn walk(&self) -> Vec<PathBuf> {
        return crate::playback::expand_paths(vec![self.root.clone()]);
    }
}

/// Removes the folder when the test ends.
impl Drop for TempTree {
    /// What: `drop` removes the folder with everything in it.
    /// Why: tests leave no files behind.
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
    }
}

/// What: `folder_names_ignore_case` is true when this machine's temporary folder treats `a` and `A` as one name.
/// Why: Windows and default macOS volumes do; the random trees below then must not hold two names that differ only by case.
///      The answer is probed once, in a throwaway folder.
fn folder_names_ignore_case() -> bool {
    static ANSWER: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    return *ANSWER.get_or_init(|| {
        let probe = TempTree::new("case-probe");
        probe.write("probe-a", "x");
        return probe.path("PROBE-A").exists();
    });
}

/// What: `pick_stem` is a random one of `all`, leaving out later names that differ from an earlier one only by case when the
/// disk ignores case.
/// Why: `a` and `A` name two folders on Linux but one on Windows, and a random tree that wrote both would not match the
///      walk's paths. On a case-sensitive disk the list is used as given, so the sequences of random numbers stay as they were.
pub(crate) fn pick_stem(all: &[&'static str], rng: &mut Rng) -> &'static str {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let usable: Vec<&'static str> = all.iter().copied().filter(|stem| return !folder_names_ignore_case() || seen.insert(stem.to_lowercase())).collect();
    return usable[rng.below(usable.len())];
}

/// What: `Rng` is a 64-bit xorshift generator.
/// Why: a fixed seed repeats a failing sequence of random edits exactly.
pub(crate) struct Rng(pub(crate) u64);

/// Methods of `Rng`.
impl Rng {
    /// What: `below` is a number from zero up to `bound`, exclusive.
    /// Why: every random choice is a bounded index.
    pub(crate) fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        return usize::try_from(self.0 % (bound as u64)).unwrap_or(0);
    }
}

/// What: `MemoryDisk` answers the planner's questions from a set of file paths and counts the questions.
/// Why: the million-track test cannot create a million files, and the counts prove the planner asked few questions.
pub(crate) struct MemoryDisk {
    /// Every track on the pretend disk.
    pub(crate) files: BTreeSet<PathBuf>,
    /// How many times each question was asked: `kind`, `entries`, `walk`.
    pub(crate) asked: std::cell::RefCell<HashMap<&'static str, usize>>,
    /// The root, so folders can be derived from file paths.
    pub(crate) root: PathBuf,
}

/// Methods of the pretend disk.
impl Disk for MemoryDisk {
    /// What: `kind` is `Track` for a listed file, `Folder` for a prefix of one, else `Missing`.
    /// Why: the pretend disk has only files; folders exist as their parents.
    fn kind(&self, path: &Path) -> EntryKind {
        *self.asked.borrow_mut().entry("kind").or_insert(0) += 1;
        if self.files.contains(path) {
            return EntryKind::Track;
        }
        let prefix: PathBuf = path.join("");
        let is_folder: bool = self.files.range(prefix.clone()..).next().is_some_and(|first| return first.starts_with(path) && first != path);
        return if is_folder { EntryKind::Folder } else { EntryKind::Missing };
    }

    /// What: `entries` lists the files and folders directly under a folder.
    /// Why: a folder event is answered from the folder's own entries.
    fn entries(&self, folder: &Path) -> Vec<(PathBuf, EntryKind)> {
        *self.asked.borrow_mut().entry("entries").or_insert(0) += 1;
        let mut found: BTreeSet<(PathBuf, bool)> = BTreeSet::new();
        for file in self.files.range(folder.to_path_buf()..).take_while(|file| return file.starts_with(folder)) {
            let Ok(below) = file.strip_prefix(folder) else { continue };
            let mut parts = below.components();
            let Some(first) = parts.next() else { continue };
            let is_file: bool = parts.next().is_none();
            found.insert((folder.join(first.as_os_str()), is_file));
        }
        return found.into_iter().map(|(path, is_file)| return (path, if is_file { EntryKind::Track } else { EntryKind::Folder })).collect();
    }

    /// What: `walk` lists every file under a folder in queue order.
    /// Why: a new folder is read once.
    fn walk(&self, folder: &Path) -> Vec<PathBuf> {
        *self.asked.borrow_mut().entry("walk").or_insert(0) += 1;
        let mut found: Vec<PathBuf> = self.files.iter().filter(|file| return file.starts_with(folder)).cloned().collect();
        found.sort_by(|left, right| return crate::walk_order::walk_cmp(&self.root, left, right));
        return found;
    }
}
