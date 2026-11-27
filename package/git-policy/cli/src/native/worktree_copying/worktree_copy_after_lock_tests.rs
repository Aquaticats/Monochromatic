//! What happens once the ignored state of a new worktree is in place:
//! the trash a snapshot left is handed to the commit service and left standing,
//! its record outlives the
//!  command,
//! and no command waits for the emptying.

use super::*;
use crate::fs_beneath::{
    ObjectId,
    identity_at,
    identity_of,
    open_directory_at,
    open_root,
    plain_components,
};
use crate::fs_beneath_walk_chain::open_directory_beneath;
/// The switch a build made for tests takes the route's runtime directory from, which this control hands in.
use crate::session_runtime_directory_and_bus_address::RUNTIME_DIRECTORY_SWITCH;
use crate::test_support::{
    fixture,
    git_context,
    remove,
};
use crate::testing::process_wide_state_child_test_support::run_the_body_with_an_environment_of_its_own;
use crate::work_item::{
    PENDING,
    WorkItem,
    WorkKind,
    state_directory,
};
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_trash::{
    PendingTrash,
    TRASH_PARENT,
    TRASH_REFERENCE,
    TrashManifest,
    entry_name,
};
use std::path::Path;

/// A worktree whose trash holds `count` entries,
/// the administrative directory its `.git` file names,
/// and the manifest a leave-out would have written for it.
fn worktree_with_planted_trash(
    name: &str,
    trash_name: &str,
    count: usize,
) -> (PathBuf, PathBuf, PathBuf, TrashManifest) {
    let root: PathBuf = fixture(name);
    let worktree: PathBuf = root.join("topic");
    let common: PathBuf = root.join("common");
    let git_dir: PathBuf = common.join("worktrees").join("topic");
    std::fs::create_dir_all(&git_dir).expect("admin");
    std::fs::create_dir_all(&worktree).expect("worktree");
    std::fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", git_dir.display()),
    )
    .expect(".git file");
    for index in 0..count {
        let entry: PathBuf = worktree
            .join(TRASH_PARENT)
            .join(trash_name)
            .join(entry_name(index));
        std::fs::create_dir_all(&entry).expect("trash entry");
        std::fs::write(entry.join("file"), b"left out\n").expect("entry file");
    }
    let held: std::fs::File = open_root(worktree.as_path()).expect("the worktree");
    let names = plain_components(Path::new(TRASH_PARENT)).expect("plain parent");
    let parent: std::fs::File =
        open_directory_beneath(&held, names.as_slice()).expect("the trash parent");
    let trash: std::fs::File =
        open_directory_at(&parent, std::ffi::OsStr::new(trash_name)).expect("the trash");
    let entries: Vec<ObjectId> = (0..count)
        .map(|index: usize| {
            return identity_at(&trash, std::ffi::OsStr::new(entry_name(index).as_str()))
                .expect("entry identity");
        })
        .collect();
    let manifest: TrashManifest = TrashManifest {
        name: String::from(trash_name),
        trash: identity_of(&trash).expect("trash identity"),
        entries,
    };
    return (root, worktree, common, manifest);
}

/// The trash of a snapshot waiting to be handed over,
/// in the worktree that holds it.
fn pending_of(
    worktree: &Path,
    common: &Path,
    manifest: TrashManifest,
) -> PendingTrash {
    return PendingTrash {
        destination: worktree.to_path_buf(),
        common_dir: common.to_path_buf(),
        manifest,
        taken_over: 0,
        quiet: false,
    };
}

/// The invocation of a creating command,
/// which decides only what the repair may print.
fn creating_invocation() -> Invocation {
    return Invocation {
        identity: WorktreeIdentity::OutsideWorktree,
        effective_cwd: PathBuf::from("/"),
        creates: true,
        destination: None,
        opted_out: false,
        short_circuit: false,
        context: git_context(),
        add_plan: None,
        repair: crate::package_repair::RepairSwitch::default(),
        quiet: false,
    };
}

/// What:
///  A trash that no commit service can be told about is named by a record under the repository's common
///  directory,
/// and it is left in the worktree,
/// so the service empties it when it next runs.
/// Why:
///   The add no longer empties anything,
/// so the record is the only thing that says the directories exist;
/// it goes
///  into the common work directory because that is the one the repository's service drains,
/// and a trash left
///  without one would be disk nobody knows about.
/// A fixture's administrative directory reaches no service at all,
/// because the child this control
///  runs its body in hands the route a runtime directory of the fixture's own and no session bus,
/// which a build made for tests reads from that hand-in alone,
/// so this is the path every
///  add takes on a machine whose units are not installed.
/// That child is what
///  keeps the reach inside the fixture:
/// a body that handed in nothing would answer the same,
/// but the world this control describes is a session that names a runtime directory of its own,
/// and naming it is how the control states that world (issues 6582 and 6586).
#[cfg(target_os = "linux")]
#[test]
fn a_trash_no_service_can_take_is_named_by_a_record_and_left_standing() {
    // What:     a fixture directory the child hands the route as its runtime directory, so the hand-over finds no
    //           socket and no session bus to provision one through.
    // Why:      a build made for tests reads that fact from this switch alone, and a threaded run shares one process.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const runtime = fixture('after-lock-record-runtime');
    // ```
    let runtime: PathBuf = fixture("after-lock-record-runtime");
    // What:     the one variable the child is given, as a name and a value of raw operating-system text.
    // Why:      the runtime directory is where the route looks for the service's socket, and a build made for
    //           tests takes it from this switch alone, so naming the fixture's keeps every reach of this body
    //           inside the fixture (issue 6586).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const named = [[RUNTIME_DIRECTORY_SWITCH, runtime]];
    // ```
    let named: [(&str, &std::ffi::OsStr); 1] = [(RUNTIME_DIRECTORY_SWITCH, runtime.as_os_str())];
    // What:     the body, run in a child of this test binary whose environment is `named` and which inherits no
    //           session bus address.
    // Why:      a fixture's own runtime directory does not keep a program off this account's bus, because the
    //           address is inherited: a child that kept it would still ask this account's manager, and every
    //           test of the run is one process, so the body cannot unset it here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // runTheBodyWithAnEnvironmentOfItsOwn(modulePath(), 'a_trash_no_service_can_take_is_named_by_a_record_and_left_standing', named, ['DBUS_SESSION_BUS_ADDRESS'], () => { /* the body below */ });
    // ```
    run_the_body_with_an_environment_of_its_own(
        module_path!(),
        "a_trash_no_service_can_take_is_named_by_a_record_and_left_standing",
        named.as_slice(),
        &["DBUS_SESSION_BUS_ADDRESS"],
        || {
            let name: String = "6".repeat(40);
            let (root, worktree, common, manifest) =
                worktree_with_planted_trash("after-lock-record", name.as_str(), 2);
            let pending: PendingTrash =
                pending_of(worktree.as_path(), common.as_path(), manifest.clone());
            let after: AfterLock = AfterLock {
                source_root: None,
                created: Vec::new(),
                trash: Some(pending),
            };
            let invocation: Invocation = creating_invocation();
            let mut stderr: Vec<u8> = Vec::new();
            finish_trash_and_repair(&invocation, &after, &mut stderr);
            assert!(
                worktree
                    .join(TRASH_PARENT)
                    .join(&name)
                    .join("d1/file")
                    .is_file(),
                "the command returns without emptying anything"
            );
            let record: PathBuf = state_directory(
                crate::service_work::work_directory(common.as_path()).as_path(),
                PENDING,
            )
            .join(format!("trash-{name}"));
            let text: String =
                std::fs::read_to_string(record.as_path()).expect("the record the hand-over wrote");
            let item: WorkItem = WorkItem::decode(text.as_str()).expect("a work item");
            assert_eq!(item.kind, WorkKind::Trash);
            assert_eq!(
                item.git_dir, common,
                "the record is the repository's, so one service empties its trashes one at a time"
            );
            assert_eq!(item.repository, worktree);
            assert_eq!(item.oid, name);
            assert_eq!(item.parent, manifest.encode());
            assert_eq!(item.branch_ref, TRASH_REFERENCE);
            assert_eq!(
                stderr,
                Vec::<u8>::new(),
                "nothing is repaired here, and the reason the wake failed is a warning of its own"
            );
            remove(root.as_path());
        },
    );
    // What:     the runtime directory goes once the child that named it has ended.
    // Why:      a fixture that nobody removes is left under this run's fixture root for every later run to meet.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // remove(runtime);
    // ```
    remove(runtime.as_path());
}

/// A hand-over with nothing to hand over writes nothing and says nothing.
#[cfg(target_os = "linux")]
#[test]
fn an_add_that_set_nothing_aside_hands_nothing_over() {
    let root: PathBuf = fixture("after-lock-nothing");
    let common: PathBuf = root.join("common");
    std::fs::create_dir_all(&common).expect("the common directory");
    let after: AfterLock = AfterLock {
        source_root: None,
        created: Vec::new(),
        trash: None,
    };
    let invocation: Invocation = creating_invocation();
    let mut stderr: Vec<u8> = Vec::new();
    finish_trash_and_repair(&invocation, &after, &mut stderr);
    assert_eq!(stderr, Vec::<u8>::new());
    assert!(
        !crate::service_work::work_directory(common.as_path()).exists(),
        "no work directory is made for nothing"
    );
    remove(root.as_path());
}
