//! Controls for the paths whose entries hold no bytes a merge can carry,
//! and for the sentence that stops the merge there.

use super::*;
use crate::test_support::{fixture, remove};
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

/// What:
///  A named pipe stands at one path and every other kind of entry stands at another,
///  and only the pipe is named.
/// Why:
///   A plain file and a link hold bytes the carry keeps,
/// and a directory is judged by its own module,
/// so naming
///  any of them here would stop a merge that loses nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(aSpecialFileTheMergeChanges(root, [pipe, plain, link, dir])).toEqual([pipe]);
/// ```
#[cfg(unix)]
#[test]
fn a_named_pipe_is_named_and_a_plain_file_a_link_and_a_directory_are_not() {
    let parent: PathBuf = fixture("merge-special-file-scan");
    std::fs::write(parent.join("plain.txt"), b"plain\n").expect("file");
    std::fs::create_dir(parent.join("directory")).expect("directory");
    std::os::unix::fs::symlink("plain.txt", parent.join("link.txt")).expect("link");
    let made: std::process::ExitStatus = std::process::Command::new("mkfifo").arg(parent.join("pipe")).status().expect("mkfifo");
    assert!(made.success(), "the fixture holds a named pipe");
    let paths: Vec<Vec<u8>> = ["plain.txt", "directory", "link.txt", "pipe", "absent.txt"].iter().map(|name| { return name.as_bytes().to_vec(); }).collect();
    let found: Vec<Vec<u8>> = a_special_file_the_merge_changes(parent.as_path(), paths.as_slice());
    assert_eq!(found, vec![b"pipe".to_vec()], "only the entry that holds no bytes is named");
    remove(parent.as_path());
}

/// What:
///  The sentence names the paths and says where they still are and what to do.
/// Why:
///   The merge stops without changing anything,
/// so the person has to know their entries were kept and how the merge can
///  be run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theLine('feature', [b'pipe'])).toContain('pipe');
/// ```
#[test]
fn the_line_names_the_paths_kept_where_they_are_and_the_way_to_run_the_merge() {
    let said: String = the_line("feature", &[b"pipe".to_vec(), b"dev/socket".to_vec()]);
    assert!(said.starts_with("monochromatic-git-policy-cli: git merge feature: "), "{said}");
    assert!(said.contains("pipe") && said.contains("dev/socket"), "every path is named: {said}");
    assert!(said.contains("a named pipe, a socket or a device"), "what stands there is said: {said}");
    assert!(said.contains("the merge was not made"), "{said}");
    assert!(said.contains("nothing in the worktree, the index or the branch changed"), "{said}");
    assert!(said.contains("still in your checkout as they were"), "{said}");
    assert!(said.contains("run the merge again"), "{said}");
}
