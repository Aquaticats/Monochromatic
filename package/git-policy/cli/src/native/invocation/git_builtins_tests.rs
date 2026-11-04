//! What:
//!  The built-in table against real Git's own list.
//! Why:
//!  A missing name would refuse a plain Git command in a linked worktree;
//!  an extra
//!      name would forward an alias that might create a worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect([...GIT_BUILTIN_COMMANDS]).toEqual(run('git --list-cmds=builtins').split('\n'));
//! ```

/// The table under test and the real-Git fixture helpers.
use super::{GIT_BUILTIN_COMMANDS, is_git_builtin};
use crate::test_support::{fixture, git_output, remove};
use std::path::PathBuf;
use std::process::Output;

/// The table is exactly what Git 2.56.0 prints,
///  name for name and in order.
#[test]
fn table_is_what_real_git_lists() {
    let root: PathBuf = fixture("builtins");
    let output: Output = git_output(root.as_path(), &["--list-cmds=builtins"]);
    assert!(output.status.success());
    let mut expected: Vec<u8> = Vec::<u8>::new();
    for name in GIT_BUILTIN_COMMANDS {
        expected.extend_from_slice(name);
        expected.push(b'\n');
    }
    assert_eq!(
        String::from_utf8_lossy(expected.as_slice()),
        String::from_utf8_lossy(output.stdout.as_slice())
    );
    remove(root.as_path());
}

/// Only an exact built-in name is built in.
#[test]
fn only_exact_names_are_built_in() {
    for word in ["add", "commit", "status", "worktree", "write-tree"] {
        assert!(is_git_builtin(word.as_bytes()), "{word}");
    }
    for word in [
        "",
        "st",
        "ad",
        "added",
        "ADD",
        "cli-git",
        "gui",
        "worktree ",
    ] {
        assert!(!is_git_builtin(word.as_bytes()), "{word}");
    }
    assert!(!is_git_builtin(b"add\xff"));
}
