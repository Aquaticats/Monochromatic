//! What:
//!  The controls of one look at one path:
//!  a file,
//!  a folder,
//!  nothing,
//!  and an object of another kind,
//!  each read from a fixture the control wrote.
//! Why:
//!  The handler and staging checks judge presence,
//!  kind,
//!  owner and mode,
//!  and only a look that tells all four apart lets a check name the one a
//!  person repairs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(thePathState('/fixture/file')).toEqual({ kind: 'file', owner: process.geteuid(), mode: 0o644 });
//! ```

/// The two looks a control reads.
use super::{
    the_path_facts,
    the_path_state,
};
/// The states a look answers,
/// and the path facts it travels in.
use crate::doctor_zfs_route::handler_checks::{
    PathFacts,
    PathState,
};
/// One fixture folder of this call's own.
use crate::testing::test_support::fixture;
/// The account this process runs as,
/// which owns every file the control writes.
use crate::worktree_clone_add_route::add_grants::this_account;
/// The owner and mode bits the control's own files carry.
use std::os::unix::fs::PermissionsExt;
/// The owned and borrowed paths these controls use.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  A file and a folder answer their own kinds with the owner and the
///  permission bits they carry.
/// Why:
///  A handler check compares a file's mode with the install's and a folder's
///  with the design's,
///  so the look must keep the two kinds apart and report the bits without the
///  file type's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(thePathState(file)).toEqual({ kind: 'file', owner, mode: 0o644 });
/// ```
#[test]
fn a_file_and_a_folder_answer_their_kinds_owners_and_modes() {
    let root: PathBuf = fixture("one-look-kinds");
    let file: PathBuf = root.join("executable");
    std::fs::write(file.as_path(), b"#!/bin/sh\n").expect("the file");
    std::fs::set_permissions(file.as_path(), std::fs::Permissions::from_mode(0o755))
        .expect("the file's mode");
    let folder: PathBuf = root.join("readiness");
    std::fs::create_dir(folder.as_path()).expect("the folder");
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(0o700))
        .expect("the folder's mode");
    let account: u32 = this_account();
    assert_eq!(
        the_path_state(file.as_path()),
        PathState::File {
            owner: account,
            mode: 0o755,
        }
    );
    assert_eq!(
        the_path_state(folder.as_path()),
        PathState::Directory {
            owner: account,
            mode: 0o700,
        }
    );
}

/// What:
///  Nothing at a path answers absent,
///  and a link answers the kind it is rather than the file it names.
/// Why:
///  A link at an install path is the one state a look must not follow,
///  and a path nothing stands at is the install nobody ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(thePathState('/nonexistent')).toEqual({ kind: 'absent' });
/// expect(thePathState(link)).toEqual({ kind: 'wrong-kind', found: 'a symbolic link' });
/// ```
#[test]
fn nothing_is_absent_and_a_link_is_the_kind_it_is() {
    let root: PathBuf = fixture("one-look-link");
    let target: PathBuf = root.join("target");
    std::fs::write(target.as_path(), b"x").expect("the target");
    let link: PathBuf = root.join("zedlet");
    std::os::unix::fs::symlink(target.as_path(), link.as_path()).expect("the link");
    assert_eq!(
        the_path_state(root.join("gone").as_path()),
        PathState::Absent
    );
    let state: PathState = the_path_state(link.as_path());
    let PathState::WrongKind {
        found,
    } = state
    else {
        panic!("a link is another kind: {state:?}");
    };
    assert_eq!(found, "a symbolic link");
}

/// What:
///  The facts a look travels in name the same path the look judged.
/// Why:
///  A finding prints the path a person opens,
///  so the path and the state must be the two halves of one look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(thePathFacts('/fixture/x')).toEqual({ path: '/fixture/x', state: { kind: 'absent' } });
/// ```
#[test]
fn the_facts_name_the_path_the_look_judged() {
    let path: &Path = Path::new("/nonexistent/monochromatic-git-policy-cli");
    let facts: PathFacts = the_path_facts(path);
    assert_eq!(facts.path, PathBuf::from(path));
    assert_eq!(facts.state, PathState::Absent);
}
