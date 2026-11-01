//! What:
//!  Argument and output-shape controls for the one location query.
//! Why:
//!  Whether a command runs from the repository root is read from the last line of this
//!      query.
//!  A misread line turns a subdirectory into the root,
//!  or the reverse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseLocationOutput(true, 'false\n/r/.git\n/r/.git\n/r\nsub/\n').prefix).toEqual(Buffer.from('sub/'));
//! ```
#![cfg(unix)]

/// The functions under test and the identity type they return.
use super::{
    PREFIX_OPTION, RepositoryLocation, effective_directory, location_query_arguments,
    parse_location_output,
};
use crate::command_test_support::os_arguments;
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

/// The main-worktree identity of a repository rooted at `/r`.
fn main_at_r() -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: PathBuf::from("/r/.git"),
        git_dir: PathBuf::from("/r/.git"),
        worktree_root: PathBuf::from("/r"),
    };
}

/// On success the last line is the prefix:
///  empty at the top level,
///  the exact bytes below it.
#[test]
fn successful_output_ends_with_the_prefix() {
    assert_eq!(PREFIX_OPTION, "--show-prefix");
    assert_eq!(
        parse_location_output(true, b"false\n/r/.git\n/r/.git\n/r\n\n").expect("top level"),
        RepositoryLocation {
            identity: main_at_r(),
            prefix: Vec::<u8>::new(),
        }
    );
    assert_eq!(
        parse_location_output(true, b"false\n/r/.git\n/r/.git\n/r\nsub/dir/\n").expect("nested"),
        RepositoryLocation {
            identity: main_at_r(),
            prefix: b"sub/dir/".to_vec(),
        }
    );
    assert_eq!(
        parse_location_output(true, b"false\n/r/.git\n/r/.git\n/r\ns\xff b/\n").expect("bytes"),
        RepositoryLocation {
            identity: main_at_r(),
            prefix: b"s\xff b/".to_vec(),
        }
    );
    assert_eq!(
        parse_location_output(true, b"false\n/r/.git/worktrees/w\n/r/.git\n/w\nx/\n")
            .expect("linked"),
        RepositoryLocation {
            identity: WorktreeIdentity::LinkedWorktree {
                common_dir: PathBuf::from("/r/.git"),
                git_dir: PathBuf::from("/r/.git/worktrees/w"),
                worktree_root: PathBuf::from("/w"),
            },
            prefix: b"x/".to_vec(),
        }
    );
}

/// A failed query has an empty prefix:
///  Git stopped at the missing worktree before printing one.
#[test]
fn failed_output_has_no_prefix() {
    assert_eq!(
        parse_location_output(false, b"").expect("outside"),
        RepositoryLocation {
            identity: WorktreeIdentity::OutsideWorktree,
            prefix: Vec::<u8>::new(),
        }
    );
    assert_eq!(
        parse_location_output(false, b"true\n/b.git\n/b.git\n").expect("bare"),
        RepositoryLocation {
            identity: WorktreeIdentity::BareRepository {
                common_dir: PathBuf::from("/b.git"),
                git_dir: PathBuf::from("/b.git"),
            },
            prefix: Vec::<u8>::new(),
        }
    );
    assert_eq!(
        parse_location_output(false, b"false\n/r/.git\n/r/.git\n").expect("inside .git"),
        RepositoryLocation {
            identity: WorktreeIdentity::OutsideWorktree,
            prefix: Vec::<u8>::new(),
        }
    );
}

/// Output shapes Git never produces for this query are rejected instead of guessed.
#[test]
fn unexpected_output_shapes_are_rejected() {
    for stdout in [
        b"".as_slice(),
        b"\n",
        b"false",
        // The prefix line is missing: the fourth line would be read as the prefix.
        b"false\n/r/.git\n/r/.git\n/r\n",
        // A directory name with a line break makes one line too many.
        b"false\n/r/.git\n/r/.git\n/r\nsub\ndir/\n",
        b"maybe\n/r/.git\n/r/.git\n/r\n\n",
    ] {
        let error = parse_location_output(true, stdout).expect_err("malformed");
        assert!(
            error.to_string().starts_with("cli-git could not "),
            "{error}"
        );
    }
    assert!(parse_location_output(false, b"true\n").is_err());
    assert_eq!(
        parse_location_output(true, b"x")
            .expect_err("no line break")
            .to_string(),
        "cli-git could not locate the command inside the Git worktree: git rev-parse printed \
         no line for the path below the top level."
    );
}

/// The query is the caller's global options,
///  the identity options,
///  then the prefix option last.
#[test]
fn the_query_puts_the_global_prefix_first_and_the_prefix_option_last() {
    assert_eq!(
        location_query_arguments(os_arguments(&["-C", "somewhere", "--git-dir=x"]).as_slice()),
        os_arguments(&[
            "-C",
            "somewhere",
            "--git-dir=x",
            "rev-parse",
            "--path-format=absolute",
            "--is-bare-repository",
            "--git-dir",
            "--git-common-dir",
            "--show-toplevel",
            "--show-prefix",
        ])
    );
    assert_eq!(location_query_arguments(&[]).len(), 7);
}

/// The effective directory is the top level itself for an empty prefix,
///  and the joined path otherwise.
#[test]
fn effective_directory_joins_the_prefix() {
    assert_eq!(
        effective_directory(Path::new("/r"), b""),
        PathBuf::from("/r")
    );
    assert_eq!(
        effective_directory(Path::new("/r"), b"a/b/"),
        PathBuf::from("/r/a/b")
    );
    // The text shown in diagnostics carries no trailing slash, with or without Git's.
    assert_eq!(
        effective_directory(Path::new("/r"), b"a/b/").as_os_str(),
        "/r/a/b"
    );
    assert_eq!(
        effective_directory(Path::new("/r"), b"a/b").as_os_str(),
        "/r/a/b"
    );
    assert_eq!(effective_directory(Path::new("/r"), b"").as_os_str(), "/r");
    assert_ne!(
        effective_directory(Path::new("/r"), b"a/"),
        PathBuf::from("/r")
    );
    assert_eq!(
        effective_directory(Path::new("/r"), b"x\xff/"),
        PathBuf::from(OsString::from_vec(b"/r/x\xff".to_vec()))
    );
}
