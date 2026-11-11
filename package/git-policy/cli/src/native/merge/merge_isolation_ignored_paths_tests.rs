//! Controls for the read of the ignored untracked files the merge writes over:
//! the batches one Git start carries,
//! and the paths of a real repository the read answers with.
//! Each control that needs a repository runs real Git in a disposable one.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use std::path::PathBuf;

/// What:
///  The batches cover the list once,
/// in order,
/// and no batch carries more path text than one call may.
/// Why:
///   A merge of many paths must not fail to start Git over its own argument list,
/// and a path dropped from a batch is a
///  file the merge would write over.
#[test]
fn the_batches_cover_every_path_once_under_the_argument_bound() {
    let per_path: usize = PATH_TEXT_PER_CALL / 4;
    let many: Vec<Vec<u8>> = (0..20)
        .map(|index| {
            let mut path: Vec<u8> = format!("dir/{index}-").into_bytes();
            path.resize(per_path, b'x');
            return path;
        })
        .collect();
    let spans: Vec<(usize, usize)> = batches(many.as_slice());
    let mut covered: usize = 0;
    for (start, end) in spans.iter().copied() {
        assert_eq!(start, covered, "the batches are adjacent and ordered");
        assert!(end > start, "a batch holds at least one path");
        let bytes: usize = many[start..end].iter().map(Vec::len).sum();
        assert!(bytes <= PATH_TEXT_PER_CALL, "a batch's path text stays under the bound: {bytes}");
        if end < many.len() {
            assert!(bytes + many[end].len() > PATH_TEXT_PER_CALL, "a batch ends only where the next path would break the bound");
        }
        covered = end;
    }
    assert_eq!(covered, many.len(), "every path is in a batch exactly once");
    assert!(spans.len() > 1, "path text this long takes more than one call");
    assert_eq!(batches(&[]), [(0usize, 0usize)], "an empty list is one empty batch");
}

/// What:
///  A repository where `.gitignore` covers `added.txt` and a whole `build/` directory.
/// Why:
///  The read must answer for a file a rule names and for one below a directory a rule ignores whole,
/// and it must
///  leave out a path the index tracks even when a rule covers it.
fn a_repository_with_ignored_paths(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join(".gitignore"), "added.txt\nbuild/\ntracked-ignored.txt\n").expect(".gitignore");
    std::fs::write(repo.join("tracked-ignored.txt"), b"tracked\n").expect("tracked-ignored.txt");
    std::fs::write(repo.join("plain.txt"), b"plain\n").expect("plain.txt");
    git(repo.as_path(), &["add", "--", ".gitignore", "plain.txt"]);
    git(repo.as_path(), &["add", "--force", "--", "tracked-ignored.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::create_dir(repo.join("build")).expect("build");
    std::fs::write(repo.join("build").join("out.txt"), b"the person's build output\n").expect("build/out.txt");
    std::fs::write(repo.join("added.txt"), b"the person's ignored file\n").expect("added.txt");
    return (parent, repo);
}

/// What:
///  The ignored untracked files of the paths given are answered,
/// and a path the index tracks,
/// a path no rule
///  covers,
/// a path that stands outside the list,
/// and a path that is not there are all left out.
/// Why:
///   Only a path the merge writes and an ignore rule covers holds work the capture must carry;
/// anything else would be
///  captured for nothing or,
/// worse,
/// misread as the person's.
#[test]
fn only_the_ignored_untracked_files_among_the_paths_given_are_answered() {
    let (parent, repo): (PathBuf, PathBuf) = a_repository_with_ignored_paths("merge-ignored-paths-read");
    let context: GitContext = git_context();
    let written: Vec<Vec<u8>> = ["added.txt", "build/out.txt", "tracked-ignored.txt", "plain.txt", "missing.txt"]
        .iter()
        .map(|name| { return name.as_bytes().to_vec(); })
        .collect();
    let mut found: Vec<Vec<u8>> = ignored_untracked_among(&context, repo.as_path(), written.as_slice()).expect("the read answers");
    found.sort();
    let expected: Vec<Vec<u8>> = [b"added.txt".to_vec(), b"build/out.txt".to_vec()].to_vec();
    assert_eq!(found, expected, "the files a rule covers, and nothing else");
    let outside: Vec<Vec<u8>> = [b"other.txt".to_vec()].to_vec();
    assert_eq!(ignored_untracked_among(&context, repo.as_path(), outside.as_slice()), Ok(Vec::new()), "a path no rule covers is no work of theirs");
    assert_eq!(ignored_untracked_among(&context, repo.as_path(), &[]), Ok(Vec::new()), "a merge that writes nothing asks nothing");
    remove(parent.as_path());
}
