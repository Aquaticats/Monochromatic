//! What:
//!  Fixtures shared by the controls of the cheap-worktree ladder:
//! a main checkout that is a btrfs subvolume with two
//!  commits,
//! aged files and an ignored directory,
//! its identity,
//! and a plan for a destination beside it.
//! Why:
//!  Every rung starts from a clean subvolume source whose index is provably unchanged,
//! which needs files older than the
//!  index;
//! building it once keeps the controls about the rung and not about the fixture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const repo = await subvolumeRepository(root); const identity = mainIdentity(repo);
//! ```

use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::subvolume_index::walk_entries;
use crate::subvolume_sha::{sha1, sha256};
use crate::test_support::{age, git, git_output};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_identity::WorktreeIdentity;
use std::path::{Path, PathBuf};
use std::os::unix::fs::MetadataExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Write a tracked file with its parents and age it.
pub(crate) fn tracked(repo: &Path, relative: &str, content: &[u8]) {
    let path: PathBuf = repo.join(relative);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
    std::fs::write(path.as_path(), content).expect("tracked file");
    age(path.as_path());
}

/// Commit everything and refresh the index so every entry is provably unchanged.
pub(crate) fn commit_all(repo: &Path, message: &str) {
    git(repo, &["add", "--all"]);
    git(repo, &["commit", "--quiet", format!("--message={message}").as_str()]);
    git(repo, &["update-index", "--refresh"]);
}

/// A main checkout that is a subvolume with two commits:
/// the first holds `old.txt`,
/// `shared.txt` and `src/deep/leaf.txt`,
///  the second changes `shared.txt`,
/// adds `new/added.txt` and deletes `old.txt`;
/// `ignored/pkg/index.js` is ignored state.
pub(crate) fn subvolume_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = root.join("repo");
    subvolume_create_at(repo.as_path()).expect("repository subvolume");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    tracked(repo.as_path(), ".gitignore", b"ignored/\n");
    tracked(repo.as_path(), "old.txt", b"old\n");
    tracked(repo.as_path(), "shared.txt", b"first\n");
    tracked(repo.as_path(), "src/deep/leaf.txt", &vec![b'x'; 5_000]);
    commit_all(repo.as_path(), "first");
    std::fs::remove_file(repo.join("old.txt")).expect("delete");
    tracked(repo.as_path(), "shared.txt", b"second, longer\n");
    tracked(repo.as_path(), "new/added.txt", b"added\n");
    commit_all(repo.as_path(), "second");
    std::fs::create_dir_all(repo.join("ignored/pkg")).expect("ignored directory");
    std::fs::write(repo.join("ignored/pkg/index.js"), b"module.exports = 1;\n").expect("ignored file");
    return std::fs::canonicalize(repo).expect("canonical");
}

/// The identity of the main worktree at `repo`.
pub(crate) fn main_identity(repo: &Path) -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree { common_dir: repo.join(".git"), git_dir: repo.join(".git"), worktree_root: repo.to_path_buf() };
}

/// A plan for a destination,
/// with the insertion point after `worktree add`.
pub(crate) fn plan_to(destination: PathBuf) -> AddPlan {
    return AddPlan { destination, insert_at: 2, quiet: false };
}

/// The modification time every fixture that hides an edit gives its file:
/// a whole second in the past,
/// so the
///  entry is not racy against the index file.
pub(crate) const HIDDEN_EDIT_SECONDS: u64 = 1_577_934_245;

/// What:
///  Give `path` the modification time `seconds`.
/// Why:
///   A fixture that hides an edit puts the file's times back after writing it,
/// so nothing about the file's
///  length or modification time says it changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const seconds = 1_577_934_245; const when = new Date(seconds * 1000); utimesSync(path, when, when);
/// ```
fn set_modified(path: &Path, seconds: u64) {
    let when: SystemTime = UNIX_EPOCH + Duration::from_secs(seconds);
    let times: std::fs::FileTimes = std::fs::FileTimes::new().set_modified(when);
    std::fs::File::open(path).expect("the file whose times are put back").set_times(times).expect("the modification time");
}

/// Wait until a new whole second of the wall clock has begun.
fn wait_for_a_new_second() {
    let now: Duration = SystemTime::now().duration_since(UNIX_EPOCH).expect("a clock after 1970");
    std::thread::sleep(Duration::from_secs(1) - Duration::from_nanos(u64::from(now.subsec_nanos())) + Duration::from_millis(2));
}

/// What:
///  Replace the tracked file `relative` with `edited`,
/// which is as long as `original`,
/// in a way Git's
///  stat check cannot see,
/// and answer whether it cannot.
/// Why:
///   Git decides whether a tracked file changed by its length,
/// its modification time and the second of its
///  change time.
/// A file written at the start of a whole second,
/// recorded in the index and then replaced with bytes
///  of the same length and the same modification time keeps all three,
/// so `git status`,
/// `git diff-files`,
///  `read-tree --reset -u` and `reset --hard` all take it for the entry's content.
/// The write and the
///  index record are retried when the second rolled over between them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hideAnEdit(repo, relative, original, edited) { ...; return git(repo, ['diff-files', '--quiet']).status === 0 }
/// ```
pub(crate) fn hide_an_edit(repo: &Path, relative: &str, original: &[u8], edited: &[u8]) -> bool {
    assert_eq!(original.len(), edited.len(), "an edit the stat check cannot see keeps the length of the file it replaces");
    let path: PathBuf = repo.join(relative);
    for _attempt in 0..5 {
        std::fs::write(path.as_path(), original).expect("the file before the edit");
        wait_for_a_new_second();
        set_modified(path.as_path(), HIDDEN_EDIT_SECONDS);
        git(repo, &["add", "--", relative]);
        let recorded: std::fs::Metadata = std::fs::metadata(path.as_path()).expect("the file's stat when the index recorded it");
        std::fs::write(path.as_path(), edited).expect("the edit");
        set_modified(path.as_path(), HIDDEN_EDIT_SECONDS);
        let hidden: bool = git_output(repo, &["diff-files", "--quiet"]).status.success();
        let same_second: bool = std::fs::metadata(path.as_path()).expect("the file's stat after the edit").ctime() == recorded.ctime();
        if hidden && same_second {
            return true;
        }
    }
    return false;
}

/// What:
///  The index `index` as a Git built without nanosecond support writes it:
/// every entry's change and modification
///  nanoseconds zero, and the checksum recomputed over the new bytes.
/// Why:
///   Zero is how such a Git records a time it cannot express,
/// and the comparison of recorded stat data reads zero
///  as "any value",
/// so a whole entry set loses the only field that could prove which bytes a file holds.
/// A control that
///  needs that proof has to write this view of an index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// asAGitWithoutNanosecondsWroteIt(readFileSync(index), 20); // the counts zero, the checksum over the new bytes
/// ```
pub(crate) fn as_a_git_without_nanoseconds_wrote_it(index: &[u8], hash_length: usize) -> Vec<u8> {
    let Some((entries, _end)) = walk_entries(index, hash_length) else { panic!("the fixture's index has a layout the walk knows") };
    let Some(checksum_at) = index.len().checked_sub(hash_length) else { panic!("the fixture's index is longer than its checksum") };
    let mut zeroed: Vec<u8> = index.to_vec();
    for entry in &entries {
        zeroed[entry.start + 4..entry.start + 8].fill(0);
        zeroed[entry.start + 12..entry.start + 16].fill(0);
    }
    if hash_length == 32 {
        let digest: [u8; 32] = sha256(&zeroed[..checksum_at]);
        zeroed[checksum_at..].copy_from_slice(&digest);
    } else {
        let digest: [u8; 20] = sha1(&zeroed[..checksum_at]);
        zeroed[checksum_at..].copy_from_slice(&digest);
    }
    return zeroed;
}
