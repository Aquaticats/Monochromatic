//! What:
//!  The controls of one registration's `gitdir` file:
//!  a path it names,
//!  a path written relative to the registration,
//!  a file that is not there,
//!  and a file nobody may read,
//!  which is unknown and never the half-made state a missing file means.
//! Why:
//!  A registration that stands with a link this account cannot see proves
//!  nothing about the link,
//!  so the three answers are kept apart here and the line that reports them
//!  says which it read.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theGitdirARegistrationNames(registration)).toEqual({ kind: 'named', path: slotGit });
//! ```

/// The reader and its two values.
use super::{
    ARegistrationsGitdir,
    GITDIR_RECORD_LIMIT,
    the_gitdir_a_registration_names,
};
/// One fixture folder of the control's own.
use crate::test_support::{
    fixture,
    remove,
};
use std::path::{
    Path,
    PathBuf,
};

/// A registration folder of the control's own,
/// holding the file the route reads.
fn a_registration(name: &str) -> PathBuf {
    let folder: PathBuf = fixture(name);
    let registration: PathBuf = folder.join("worktrees").join("topic");
    std::fs::create_dir_all(registration.as_path()).expect("the registration");
    return registration;
}

/// What:
///  A `gitdir` file naming an absolute path answers that path,
///  and the newline Git writes is not part of it.
#[test]
fn a_gitdir_file_names_the_path_it_holds() {
    let registration: PathBuf = a_registration("registration-gitdir-absolute");
    let named: PathBuf = registration.join("..").join("topic-git").join(".git");
    std::fs::write(
        registration.join("gitdir"),
        format!("{}\n", named.display()),
    )
    .expect("the gitdir file");
    assert_eq!(
        the_gitdir_a_registration_names(registration.as_path()),
        ARegistrationsGitdir::Named(named)
    );
    remove(
        registration
            .parent()
            .expect("the fixture root")
            .parent()
            .expect("the fixture root"),
    );
}

/// What:
///  A `gitdir` file written relative to the registration answers the path it
///  names inside the registration's own folder.
#[test]
fn a_relative_gitdir_is_resolved_against_the_registration() {
    let registration: PathBuf = a_registration("registration-gitdir-relative");
    std::fs::write(registration.join("gitdir"), b"../../elsewhere/.git\n")
        .expect("the gitdir file");
    assert_eq!(
        the_gitdir_a_registration_names(registration.as_path()),
        ARegistrationsGitdir::Named(registration.join("../../elsewhere/.git"))
    );
    remove(
        registration
            .parent()
            .expect("the fixture root")
            .parent()
            .expect("the fixture root"),
    );
}

/// What:
///  A registration with no `gitdir` file answers no file,
///  which is the state a kill inside Git's own `worktree add` leaves.
#[test]
fn a_registration_with_no_gitdir_file_answers_no_file() {
    let registration: PathBuf = a_registration("registration-gitdir-absent");
    assert_eq!(
        the_gitdir_a_registration_names(registration.as_path()),
        ARegistrationsGitdir::NoFile
    );
    remove(
        registration
            .parent()
            .expect("the fixture root")
            .parent()
            .expect("the fixture root"),
    );
}

/// What:
///  A directory at the file's name is no record,
///  and the answer says so rather than naming a path.
#[test]
fn a_directory_at_the_gitdir_name_is_unreadable() {
    let registration: PathBuf = a_registration("registration-gitdir-directory");
    std::fs::create_dir(registration.join("gitdir")).expect("the directory");
    let answer: ARegistrationsGitdir = the_gitdir_a_registration_names(registration.as_path());
    let ARegistrationsGitdir::Unreadable {
        why,
    } = answer
    else {
        panic!("a directory is no record: {answer:?}");
    };
    assert!(!why.is_empty(), "the answer says what it read");
    remove(
        registration
            .parent()
            .expect("the fixture root")
            .parent()
            .expect("the fixture root"),
    );
}

/// What:
///  A file past the bound is refused rather than read whole.
#[test]
fn a_gitdir_past_the_bound_is_unreadable() {
    let registration: PathBuf = a_registration("registration-gitdir-too-large");
    let text: Vec<u8> =
        vec![b'a'; usize::try_from(GITDIR_RECORD_LIMIT).expect("a small bound") + 1];
    std::fs::write(registration.join("gitdir"), text).expect("the gitdir file");
    let answer: ARegistrationsGitdir = the_gitdir_a_registration_names(registration.as_path());
    assert!(
        matches!(answer, ARegistrationsGitdir::Unreadable { .. }),
        "a file past the bound is unreadable: {answer:?}"
    );
    remove(
        registration
            .parent()
            .expect("the fixture root")
            .parent()
            .expect("the fixture root"),
    );
}

/// What:
///  A file whose last line has no newline still names its path,
///  because Git writes the newline and a hand-written file may not.
#[test]
fn a_gitdir_with_no_final_newline_names_its_path() {
    let registration: PathBuf = a_registration("registration-gitdir-no-newline");
    let named: PathBuf =
        Path::new("/home/user/Monochromatic/.git/worktrees/topic/.git").to_path_buf();
    std::fs::write(registration.join("gitdir"), named.display().to_string())
        .expect("the gitdir file");
    assert_eq!(
        the_gitdir_a_registration_names(registration.as_path()),
        ARegistrationsGitdir::Named(named)
    );
    remove(
        registration
            .parent()
            .expect("the fixture root")
            .parent()
            .expect("the fixture root"),
    );
}
