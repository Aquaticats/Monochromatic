//! What:
//!  The common Git directory of a worktree's own Git directory,
//!  through each of the three answers a Git directory can give.
//! Why:
//!  The timing record of a linked worktree must land in the repository's own directory,
//!  and a `commondir` with a trailing carriage return,
//!  a relative path or a place that is gone
//!  must not send it anywhere but where Git would.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(commonDirectoryOf(join(common, 'worktrees/w'))).toBe(common);
//! ```

/// The rule under test.
use super::{
    bytes_named,
    common_directory_named_by,
    common_directory_of,
};
use crate::command_test_support::{
    fixture,
    remove,
};
/// What:
///  `std::ffi::OsStr` is a borrowed piece of an operating system's own text, the type a path is made of,
///  whose sibling `str` holds UTF-8 alone and so cannot spell every name a file system holds.
/// Why:
///  A control that plants a directory whose name is no UTF-8 needs one path part of raw bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OsStr = Buffer; // a path part is bytes
/// ```
use std::ffi::OsStr;
/// What:
///  `std::os::unix::ffi::OsStrExt` is the Unix-only extension trait that makes an `OsStr` out of bytes
///  (`OsStr::from_bytes`) and reads its bytes back out; the siblings it beats here are `OsString`'s own Unix trait
///  (an owned name, which this control does not need) and `String::from_utf8`, which refuses these bytes.
/// Why:
///  The control builds the name Git wrote in `commondir` exactly, byte for byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = Buffer.from([0xff, 0xfe]).toString('latin1'); // only for the assertion below
/// ```
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

/// A Git directory with no `commondir` file is its own common directory.
#[test]
fn a_git_directory_without_a_commondir_file_is_its_own() {
    let root: PathBuf = fixture("common-dir-own");
    let git_dir: PathBuf = root.join(".git");
    std::fs::create_dir_all(&git_dir).expect("git directory");
    assert_eq!(common_directory_of(git_dir.as_path()), git_dir);
    remove(root.as_path());
}

/// A linked worktree's Git directory answers with the directory its `commondir` names,
/// however many line endings the file ends with.
#[test]
fn a_linked_worktrees_commondir_names_the_repositorys_directory() {
    let root: PathBuf = fixture("common-dir-linked");
    let common: PathBuf = root.join("main/.git");
    let own: PathBuf = common.join("worktrees/w");
    std::fs::create_dir_all(&own).expect("worktree git directory");
    for ending in ["../..\n", "../..\r\n", "../..\r\n\r\n"] {
        std::fs::write(own.join("commondir"), ending).expect("commondir");
        assert_eq!(
            common_directory_of(own.as_path()),
            std::fs::canonicalize(&common).expect("canonical"),
            "{ending:?}"
        );
    }
    remove(root.as_path());
}

/// An empty `commondir` file leaves the Git directory its own common directory,
/// and so does one that names a place that is gone,
/// which is used as written.
#[test]
fn an_empty_or_missing_place_leaves_a_path_that_is_used() {
    let root: PathBuf = fixture("common-dir-empty");
    let own: PathBuf = root.join("w");
    std::fs::create_dir_all(&own).expect("git directory");
    std::fs::write(own.join("commondir"), b"\n").expect("commondir");
    assert_eq!(common_directory_of(own.as_path()), own);
    std::fs::write(own.join("commondir"), b"../gone\n").expect("commondir");
    // A place that cannot be resolved is used as written, so a record still lands under the name the file gives rather than
    //  nowhere at all.
    assert_eq!(common_directory_of(own.as_path()), own.join("../gone"));
    remove(root.as_path());
}

/// A relative `commondir` is resolved against the Git directory,
/// so a record lands in the repository rather than beside the worktree.
#[test]
fn a_relative_commondir_is_resolved_against_the_git_directory() {
    let root: PathBuf = fixture("common-dir-relative");
    let common: PathBuf = root.join("main/.git");
    let own: PathBuf = common.join("worktrees/w");
    std::fs::create_dir_all(&own).expect("worktree git directory");
    std::fs::write(own.join("commondir"), b"../..\n").expect("commondir");
    let answered: PathBuf = common_directory_of(own.as_path());
    assert!(answered.ends_with(".git"), "{}", answered.display());
    assert!(
        !answered.ends_with(Path::new("worktrees/w")),
        "{}",
        answered.display()
    );
    remove(root.as_path());
}

/// A `commondir` whose bytes are no UTF-8 text names the directory it points at,
/// so a repository kept in a directory whose name no text can spell still finds its own record.
/// A reader that needed text answered the Git directory itself instead,
/// which is the worktree's own directory rather than the repository's (alpha.47's release pass, Low 4).
/// The directory the file names is not made here, because the file system this suite runs on refuses to make one
/// (`Invalid or incomplete multibyte or wide character`); an unresolvable path is used as written, which is the answer below.
#[test]
fn a_commondir_that_is_not_text_names_the_directory_it_points_at() {
    let root: PathBuf = fixture("common-dir-not-text");
    let own: PathBuf = root.join("w");
    std::fs::create_dir_all(&own).expect("git directory");
    // The two bytes that are no UTF-8 text, and no text at all beside them.
    let named: &OsStr = OsStr::from_bytes(b"../\xff\xfe-common");
    std::fs::write(own.join("commondir"), b"../\xff\xfe-common\n")
        .expect("a commondir whose bytes are no text");
    assert_eq!(
        common_directory_of(own.as_path()),
        own.join(named),
        "the bytes the file holds, joined onto the Git directory, not the Git directory itself"
    );
    assert_eq!(
        common_directory_named_by(own.as_path()),
        Some(root.join(OsStr::from_bytes(b"\xff\xfe-common"))),
        "the name is resolved from its bytes, so a directory no text can spell is still the one named"
    );
    remove(root.as_path());
}

/// A `commondir` that is a named pipe names no common directory,
/// and it answers at once:
/// opening a named pipe for reading waits for a writer that may never come,
/// which held every command that asked this question before the file was read as a plain one (issue 6415).
#[test]
fn a_commondir_that_is_a_named_pipe_names_nothing_at_once() {
    let root: PathBuf = fixture("common-dir-named-pipe");
    let own: PathBuf = root.join("w");
    std::fs::create_dir_all(&own).expect("git directory");
    crate::subvolume_sync_attrs::make_fifo(own.join("commondir").as_path(), 0o600)
        .expect("a named pipe where a commondir goes");
    let inside: PathBuf = own.clone();
    let answered = crate::test_support::finished_within(move || {
        return (
            common_directory_of(inside.as_path()),
            common_directory_named_by(inside.as_path()),
            bytes_named(inside.as_path()).is_err(),
        );
    });
    let Some((own_answer, named, refused)) = answered else {
        panic!("a `commondir` that is a named pipe held the read")
    };
    assert_eq!(
        own_answer, own,
        "a Git directory whose `commondir` is a pipe is its own common directory"
    );
    assert!(
        named.is_none(),
        "a pipe names no common directory a caller can use"
    );
    assert!(
        refused,
        "the reader reports the refusal rather than a missing file"
    );
    // The read this module made before, on the same fixture under the same bound, so the bound above is shown to see a wait.
    let pointer: PathBuf = own.join("commondir");
    let waited =
        crate::test_support::finished_within(move || return std::fs::read_to_string(pointer).ok());
    assert!(
        waited.is_none(),
        "a plain read of the same name waits, so the bound above can see a wait"
    );
    remove(root.as_path());
}
