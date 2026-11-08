//! Controls for the name of a worktree's service instance.

use super::*;
use crate::test_support::{fixture, remove};

/// Two worktrees of one repository have two instances,
/// and one worktree has one however its directory is spelled.
#[test]
fn an_instance_belongs_to_one_worktrees_own_git_directory() {
    let root = fixture("instance-names");
    let main = root.join("main/.git");
    let linked = root.join("main/.git/worktrees/linked");
    std::fs::create_dir_all(&linked).expect("directories");
    assert_ne!(instance_of(main.as_path()), instance_of(linked.as_path()), "two worktrees, two instances");
    assert_eq!(instance_of(main.as_path()), instance_of(main.as_path()));
    let spelled = root.join("main/.git/worktrees/../");
    assert_eq!(instance_of(main.as_path()), instance_of(spelled.as_path()), "a relative part names the same directory");
    let link = root.join("link-to-main");
    std::os::unix::fs::symlink(&main, &link).expect("link");
    assert_eq!(instance_of(main.as_path()), instance_of(link.as_path()), "a link names the same directory");
    remove(root.as_path());
}

/// The suite names a worktree's service in TypeScript (`e2e/service-launch-fixture.ts`);
/// this vector is pinned there too,
/// so
///  the two cannot drift apart.
#[test]
fn the_name_of_a_path_that_cannot_be_resolved_is_pinned() {
    assert_eq!(instance_of(Path::new("/nonexistent/instance-vector")), "b82882c15388721fd60c7ae28508d85b");
}

/// The name fits a unit name and a socket path whatever the directory is called.
#[test]
fn the_name_is_short_and_plain_for_any_directory() {
    let long = Path::new("/home/someone/a/very/long/path").join("x".repeat(400));
    let name = instance_of(long.as_path());
    assert_eq!(name.len(), INSTANCE_LENGTH);
    assert!(name.bytes().all(|byte| { return byte.is_ascii_hexdigit(); }), "{name}");
    assert!(socket_path(Path::new("/run/user/1000"), name.as_str()).as_os_str().len() < 108, "a socket path must fit sun_path");
    assert!(socket_unit_instance(name.as_str()).len() < 255 && service_unit_instance(name.as_str()).len() < 255);
}

/// The socket is the instance's,
/// under the runtime directory,
/// and never the shared one an earlier release listened on.
#[test]
fn the_socket_is_the_instances_own() {
    let runtime = Path::new("/run/user/1000");
    let socket = socket_for(runtime, Path::new("/work/repo/.git"));
    assert!(socket.starts_with("/run/user/1000/monochromatic-git-policy-cli"));
    assert_ne!(socket, runtime.join(LEGACY_SOCKET_RELATIVE));
    assert_eq!(socket, socket_path(runtime, instance_of(Path::new("/work/repo/.git")).as_str()));
}

/// A Git directory that resolves has the same name from the checked and the unchecked function,
/// so no existing instance changes name.
#[test]
fn the_checked_name_equals_the_unchecked_name_for_a_directory_that_resolves() {
    let root = fixture("instance-checked-same");
    let git_dir = root.join("main/.git");
    std::fs::create_dir_all(&git_dir).expect("directories");
    assert_eq!(try_instance_of(git_dir.as_path()), Ok(instance_of(git_dir.as_path())));
    remove(root.as_path());
}

/// A Git directory that cannot be resolved is refused with its path and the cause,
/// where the unchecked function would hash the text as given.
#[test]
fn the_checked_name_refuses_a_directory_that_cannot_be_resolved() {
    let root = fixture("instance-checked-dangling");
    let link = root.join("link");
    std::os::unix::fs::symlink(root.join("no-such-target"), &link).expect("dangling link");
    for unreachable in [root.join("absent/.git"), link.join(".git")] {
        let refusal: GitDirUnreachable = try_instance_of(unreachable.as_path()).expect_err("no name for an unreachable directory");
        assert_eq!(refusal.path, unreachable);
        let text: String = refusal.to_string();
        assert!(text.contains(unreachable.to_string_lossy().as_ref()) && text.contains("No such file or directory"), "{text}");
    }
    remove(root.as_path());
}
