//! What:
//!  The two listings of a worktree's changed paths.
//! Why:
//!  The `.gitattributes` listing is a `:(glob)` pattern the wrapper wrote,
//! so no pathspec mode the caller set may reach it:
//!  under literal mode it would match nothing and a modified attributes file would go unnoticed,
//! and two glob modes at once make Git refuse the listing and fail the restore.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect((await changedPaths(withGlobModes(), repo)).attributesChanged).toBe(true);
//! ```

use super::{ChangedPaths, changed_paths};
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::ffi::OsString;
use std::path::PathBuf;

/// An untracked `.gitattributes` file below the top is a change,
/// whatever glob modes the caller's environment sets.
#[test]
fn an_untracked_attributes_file_is_seen_under_the_callers_conflicting_glob_modes() {
    let root: PathBuf = fixture("changed-paths-glob-modes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("file.txt"), b"one\n").expect("write file.txt");
    git(repo.as_path(), &["add", "--", "file.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base", "--", "file.txt"]);
    std::fs::create_dir(repo.join("sub")).expect("sub directory");
    std::fs::write(repo.join("sub/.gitattributes"), b"*.txt text\n").expect("write sub/.gitattributes");
    let mut inherited = git_context();
    inherited.overlay.push((OsString::from("GIT_GLOB_PATHSPECS"), OsString::from("1")));
    inherited.overlay.push((OsString::from("GIT_NOGLOB_PATHSPECS"), OsString::from("1")));
    let changed: ChangedPaths = changed_paths(&inherited, repo.as_path()).expect("the listings answer despite the caller's modes");
    assert!(changed.attributes_changed, "the untracked attributes file is seen: {changed:?}");
    remove(root.as_path());
}
