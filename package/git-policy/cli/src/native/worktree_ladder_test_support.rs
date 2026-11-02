//! What:
//!  Fixtures shared by the controls of the cheap-worktree ladder: a main checkout that is a btrfs subvolume with two
//!  commits, aged files and an ignored directory, its identity, and a plan for a destination beside it.
//! Why:
//!  Every rung starts from a clean subvolume source whose index is provably unchanged, which needs files older than the
//!  index; building it once keeps the controls about the rung and not about the fixture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const repo = await subvolumeRepository(root); const identity = mainIdentity(repo);
//! ```

use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::test_support::{age, git};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_identity::WorktreeIdentity;
use std::path::{Path, PathBuf};

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

/// A main checkout that is a subvolume with two commits: the first holds `old.txt`, `shared.txt` and `src/deep/leaf.txt`,
///  the second changes `shared.txt`, adds `new/added.txt` and deletes `old.txt`; `ignored/pkg/index.js` is ignored state.
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

/// A plan for a destination, with the insertion point after `worktree add`.
pub(crate) fn plan_to(destination: PathBuf) -> AddPlan {
    return AddPlan { destination, insert_at: 2, quiet: false };
}
