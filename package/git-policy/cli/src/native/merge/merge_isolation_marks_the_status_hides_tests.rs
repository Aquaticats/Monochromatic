//! Controls for reading the index marks that hide a person's edit from `git status`.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

/// A context whose Git runs in `repo`,
/// which is how a merge names the repository.
fn context_in(repo: &Path) -> GitContext {
    let mut context: GitContext = git_context();
    context.global_prefix = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    return context;
}

/// Every path of the merge's own whose mark hides an edit,
/// and nothing else:
/// an `assume-unchanged` path whose bytes the person changed is hidden work,
/// so is a `skip-worktree` one,
/// while a marked path that still holds the branch's bytes is not,
/// a marked path with no file is a
///  sparse checkout's own state,
/// and a path with no mark is status's business,
/// which already lists it.
#[test]
fn only_a_marked_path_whose_bytes_differ_is_hidden_work() {
    let parent: PathBuf = fixture("merge-hidden-marks");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    for name in ["hidden.txt", "clean.txt", "sparse.txt", "plain.txt"] {
        std::fs::write(repo.join(name), b"the branch's bytes\n").expect("file");
        git(repo.as_path(), &["add", "--", name]);
    }
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "hidden.txt"]);
    git(repo.as_path(), &["update-index", "--assume-unchanged", "--", "clean.txt"]);
    git(repo.as_path(), &["update-index", "--skip-worktree", "--", "sparse.txt"]);
    std::fs::write(repo.join("hidden.txt"), b"the person's own line\n").expect("file");
    std::fs::write(repo.join("plain.txt"), b"an edit no mark hides\n").expect("file");
    // The sparse checkout's own state: the mark says the worktree copy is not the index's, and no file stands there at all.
    std::fs::remove_file(repo.join("sparse.txt")).expect("the sparse path has no worktree copy");
    let status: String = git_text(repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]);
    assert_eq!(status, " M plain.txt", "Git's own status hides the marked edits: {status}");
    let touched: Vec<Vec<u8>> = ["hidden.txt", "clean.txt", "sparse.txt"].iter().map(|name| { return name.as_bytes().to_vec(); }).collect();
    let hidden: Vec<Vec<u8>> = paths_the_status_hides(&context_in(repo.as_path()), repo.as_path(), touched.as_slice()).expect("the marks are read");
    assert_eq!(hidden, vec![b"hidden.txt".to_vec()], "only the marked edit whose bytes differ is work to carry");
    remove(parent.as_path());
}

/// The marks Git prints for a `skip-worktree` and an `assume-unchanged` entry are read,
/// a plain entry is not,
/// an unmerged
///  entry is left to the merge's own answer,
/// and a path whose bytes are another field is kept whole.
#[test]
fn the_tags_git_prints_for_each_mark_are_read_and_no_other() {
    let output: Vec<u8> = b"S 100644 1111111111111111111111111111111111111111 0\tskip.txt\0\
        s 100644 2222222222222222222222222222222222222222 0\tboth.txt\0\
        h 100644 3333333333333333333333333333333333333333 0\tassumed.txt\0\
        H 100644 4444444444444444444444444444444444444444 0\tplain.txt\0\
        M 100644 5555555555555555555555555555555555555555 2\tconflicted.txt\0\
        s 100644 6666666666666666666666666666666666666666 0\ta\tpath\twith\ttabs.txt\0"
        .to_vec();
    let marked: Vec<Marked> = marked_entries(output.as_slice());
    let names: Vec<String> = marked.iter().map(|entry| { return String::from_utf8_lossy(entry.path.as_slice()).into_owned(); }).collect();
    assert_eq!(names, vec!["skip.txt", "both.txt", "assumed.txt", "a\tpath\twith\ttabs.txt"], "every hidden mark is read, unmerged and plain entries are not");
    assert_eq!(marked[0].mode, "100644");
    assert_eq!(marked[0].object, "1111111111111111111111111111111111111111");
}
