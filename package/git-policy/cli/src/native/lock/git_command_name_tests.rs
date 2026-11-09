//! Controls for the command-name test both lock judgements share:
//! Git's own names are Git's,
//! and a program of another name is not.

use super::*;

/// Git and its helpers are Git's names.
#[test]
fn git_and_its_helpers_are_named_like_git() {
    for name in ["git", "git-remote-https", "git-nuisance"] {
        assert!(names_git_command(name), "{name} is named like Git");
    }
}

/// A program that is not Git is not named like Git.
#[test]
fn another_program_is_not_named_like_git() {
    for name in ["sleep", "", "sh", "gIt", "Git"] {
        assert!(!names_git_command(name), "{name:?} is not named like Git");
    }
}
