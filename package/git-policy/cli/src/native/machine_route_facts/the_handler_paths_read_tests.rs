//! What:
//!  The controls of the handler's installed paths and ZED's wiring:
//!  the two installed halves,
//!  the configuration and the two folders,
//!  ZED's unit as a source build and as a package place it,
//!  the wants link that enables it,
//!  the other zedlets an enablement would start,
//!  and a folder of zedlets nobody could list.
//! Why:
//!  An install that is incomplete fails only when a clone is made,
//!  so every one of these places must answer its own state from a fixture,
//!  and a folder nobody could list must answer the wiring check an unknown
//!  rather than "none besides the route's own".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const handler = theHandlerPathsRead({ places, unread }); // { installation, configuration, folders, wiring }
//! expect(handler.wiring.otherZedlets).toEqual(['zfs_scrub_finish']);
//! ```

/// The places a fixture names.
use super::super::the_route_places::TheRoutePlaces;
/// The read a control judges,
/// and the value it answers.
use super::{
    TheHandlerPaths,
    the_handler_paths_read,
};
/// The check a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// What one look at a path answered.
use crate::doctor_zfs_route::handler_checks::PathState;
/// One fixture folder of this call's own.
use crate::testing::test_support::fixture;
/// The account this process runs as.
use crate::worktree_clone_add_route::add_grants::this_account;
/// The owner and mode bits a fixture's files are given.
use std::os::unix::fs::PermissionsExt;
/// The owned paths these controls use.
use std::path::PathBuf;

/// What:
///  The executable and the zedlet answer files this account owns at their own
///  modes,
///  the configuration answers a file,
///  and the two folders answer folders.
/// Why:
///  A handler check reads presence and kind from these looks and its finding
///  from the owner and mode,
///  so a control must see the four apart without root and without a ZED
///  install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(handler.installation.executable.state).toEqual({ kind: 'file', owner: uid, mode: 0o755 });
/// ```
#[test]
fn the_installed_halves_and_the_folders_answer_their_looks() {
    let root: PathBuf = fixture("handler-paths");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    write_the_handlers(&places);
    let mut unread: Vec<CheckReport> = Vec::new();
    let handler: TheHandlerPaths = the_handler_paths_read(&places, &mut unread);
    let account: u32 = this_account();
    assert_eq!(
        handler.installation.executable.state,
        PathState::File {
            owner: account,
            mode: 0o755,
        }
    );
    assert_eq!(
        handler.configuration.file.state,
        PathState::File {
            owner: account,
            mode: 0o644,
        }
    );
    assert_eq!(
        handler.folders.readiness.state,
        PathState::Directory {
            owner: account,
            mode: 0o755,
        }
    );
    assert!(unread.is_empty(), "{unread:?}");
}

/// What:
///  The unit a source build installs at its prefix is found,
///  the wants link answers enabled,
///  and every other zedlet is named beside the route's own.
/// Why:
///  A guest builds OpenZFS from source and a package installs the unit
///  elsewhere,
///  and an enablement starts every other zedlet,
///  which is a person's decision before the link is made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(handler.wiring.zedEnabled).toBe(true);
/// expect(handler.wiring.otherZedlets).toEqual(['zfs_scrub_finish']);
/// ```
#[test]
fn the_unit_enablement_and_other_zedlets_answer_the_wiring() {
    let root: PathBuf = fixture("handler-wiring");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    write_the_handlers(&places);
    let prefix_unit: PathBuf = places
        .zed_units
        .last()
        .expect("the source build's place")
        .clone();
    write_file(prefix_unit.as_path(), "[Unit]\nDescription=ZED\n");
    let enablement: PathBuf = places
        .zed_enablements
        .first()
        .expect("the wants link's place")
        .clone();
    if let Some(folder) = enablement.parent() {
        std::fs::create_dir_all(folder).expect("the wants folder");
    }
    std::os::unix::fs::symlink(prefix_unit.as_path(), enablement.as_path())
        .expect("the wants link");
    write_file(
        places.zedlets_folder.join("zfs_scrub_finish").as_path(),
        "# a zedlet\n",
    );
    let mut unread: Vec<CheckReport> = Vec::new();
    let handler: TheHandlerPaths = the_handler_paths_read(&places, &mut unread);
    assert_eq!(handler.wiring.zed_unit.path, prefix_unit);
    assert!(handler.wiring.zed_enabled);
    assert_eq!(
        handler.wiring.other_zedlets,
        vec![String::from("zfs_scrub_finish")]
    );
    assert!(unread.is_empty(), "{unread:?}");
}

/// What:
///  A zedlets folder nobody could list answers the wiring check an unknown,
///  and leaves the enablement as it read it.
/// Why:
///  The sentence a person reads names the other zedlets an enablement would
///  start,
///  so a folder nobody listed must not answer "none besides the route's own".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(unread.map((r) => r.check)).toEqual(['zed-wiring']);
/// ```
#[test]
fn a_zedlets_folder_nobody_could_list_answers_an_unknown() {
    let root: PathBuf = fixture("handler-wiring-unreadable");
    let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
    write_the_handlers(&places);
    // A file where the folder belongs is the one state every account's read refuses.
    std::fs::remove_dir_all(places.zedlets_folder.as_path()).expect("the folder goes");
    write_file(places.zedlets_folder.as_path(), "not a folder\n");
    let mut unread: Vec<CheckReport> = Vec::new();
    let handler: TheHandlerPaths = the_handler_paths_read(&places, &mut unread);
    assert_eq!(
        unread
            .iter()
            .map(|report| return report.check)
            .collect::<Vec<Check>>(),
        vec![Check::ZedWiring]
    );
    assert!(!handler.wiring.zed_enabled);
    assert!(handler.wiring.other_zedlets.is_empty());
}

/// One fixture's installed halves,
/// its configuration and its two folders.
fn write_the_handlers(places: &TheRoutePlaces) {
    write_file(places.executable.as_path(), "#!/bin/sh\nexit 0\n");
    std::fs::set_permissions(
        places.executable.as_path(),
        std::fs::Permissions::from_mode(0o755),
    )
    .expect("the executable's mode");
    write_file(places.zedlet.as_path(), "#!/bin/sh\nexec true\n");
    std::fs::set_permissions(
        places.zedlet.as_path(),
        std::fs::Permissions::from_mode(0o755),
    )
    .expect("the zedlet's mode");
    write_file(places.configuration.as_path(), "{}\n");
    for folder in [places.readiness.as_path(), places.results.as_path()] {
        std::fs::create_dir_all(folder).expect("a folder");
        std::fs::set_permissions(folder, std::fs::Permissions::from_mode(0o755))
            .expect("a folder's mode");
    }
}

/// One file with the mode every fixture's file carries before a control changes
/// it.
fn write_file(
    path: &std::path::Path,
    text: &str,
) {
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).expect("a file's folder");
    }
    std::fs::write(path, text.as_bytes()).expect("a file");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644)).expect("a file's mode");
}
