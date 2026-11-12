//! Controls for the repository link gate at the entry of an invocation:
//! each way of naming a repository whose link to the 860 dangles prints the plain message and exits 128,
//! where Git alone prints its own `fatal:` line,
//! and nothing is created.
#![cfg(unix)]

use crate::action::Action;
use crate::control_terminal::Terminal;
use crate::entry::plan_invocation;
use crate::real_git::{Platform, ResolutionInputs};
use crate::repository_link_identity::IDENTITY_RELATIVE_PATH;
use crate::test_support::{executable, fixture, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Inputs selecting the image's real Git,
/// with a fixture file standing in for this wrapper.
fn inputs(root: &Path) -> ResolutionInputs {
    let own: PathBuf = root.join("own-wrapper");
    executable(own.as_path(), b"\x7fELF fixture wrapper");
    return ResolutionInputs {
        platform: Platform::Unix,
        path: OsString::from("/usr/bin"),
        path_extensions: OsString::new(),
        current_directory: root.to_path_buf(),
        common_paths: Vec::<PathBuf>::new(),
        own_executable: own,
        service_unit: None,
        // A command typed at a terminal; these controls stop before an overlay is planned, so the answer only has to be stated.
        terminal: Terminal::Present,
    };
}

/// Owned arguments from text.
fn text(values: &[&str]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        result.push(OsString::from(value));
    }
    return result;
}

/// The exit status of a Git fatal error.
const GIT_FATAL: i32 = 128;

/// A scratch root with a home that holds an identity file in phase `after`,
/// a link at `repolink` to a folder on a drive that is not mounted,
/// and the environment that names the home.
struct World {
    root: PathBuf,
    link: PathBuf,
    environment: Vec<(OsString, OsString)>,
}

fn world(name: &str) -> World {
    let root: PathBuf = fixture(name).canonicalize().expect("root");
    let home: PathBuf = root.join("home");
    let mount_point: PathBuf = root.join("not-a-mount");
    let link: PathBuf = root.join("repolink");
    let target: PathBuf = mount_point.join("repository/Monochromatic");
    std::fs::create_dir_all(mount_point.as_path()).expect("mount point stand-in");
    std::os::unix::fs::symlink(target.as_path(), link.as_path()).expect("dangling link");
    let identity: PathBuf = home.join(IDENTITY_RELATIVE_PATH);
    std::fs::create_dir_all(identity.parent().expect("parent")).expect("config folder");
    std::fs::write(
        identity,
        format!(
            "{{\"linkPath\":\"{}\",\"linkText\":\"{}\",\"canonicalRoot\":\"{}\",\"mountPoint\":\"{}\",\"driveUuid\":\"D\",\"nvmeUuid\":\"N\",\"rootCommit\":\"abc\",\"phase\":\"after\"}}",
            link.display(),
            target.display(),
            target.display(),
            mount_point.display()
        ),
    )
    .expect("identity file");
    return World { root, link, environment: vec![(OsString::from("HOME"), home.into_os_string())] };
}

/// The message and status the invocation stops with.
fn stopped_with(world: &World, arguments: &[&str]) -> (i32, String) {
    let mut owned: Vec<OsString> = text(arguments);
    // `LINK` stands for the link path wherever an argument holds it.
    for argument in &mut owned {
        let spelled: String = argument.to_string_lossy().replace("LINK", world.link.to_string_lossy().as_ref());
        *argument = OsString::from(spelled);
    }
    return match plan_invocation(owned.as_slice(), world.environment.as_slice(), &inputs(world.root.as_path())) {
        Action::Exit { code, stderr, .. } => (code, stderr),
        other => panic!("expected the gate to stop, got {other:?}"),
    };
}

/// The four ways the traced baseline binary let Git print its own `fatal:` line each print the plain message and exit 128.
#[test]
fn each_way_of_naming_a_dangling_repository_prints_the_plain_message() {
    let world: World = world("entry-link-dangling");
    for arguments in [
        vec!["-C", "LINK", "status"],
        vec!["-C", "LINK/sub", "status"],
        vec!["--git-dir=LINK/.git", "status"],
        vec!["-C", "LINK", "rev-parse", "--show-toplevel"],
        vec!["--work-tree", "LINK", "status"],
    ] {
        let (code, stderr) = stopped_with(&world, arguments.as_slice());
        assert_eq!(code, GIT_FATAL, "{arguments:?}: {stderr}");
        assert!(stderr.starts_with("fatal: the 860 drive is not mounted yet ("), "{arguments:?}: {stderr}");
        assert!(stderr.contains(world.link.to_string_lossy().as_ref()), "{arguments:?}: {stderr}");
    }
    remove(world.root.as_path());
}

/// A command elsewhere is not stopped:
/// it is planned as before.
#[test]
fn a_command_elsewhere_is_planned_as_before() {
    let world: World = world("entry-link-elsewhere");
    let elsewhere: PathBuf = world.root.join("elsewhere");
    std::fs::create_dir_all(elsewhere.as_path()).expect("folder");
    let arguments: Vec<OsString> = vec![OsString::from("-C"), elsewhere.into_os_string(), OsString::from("--version")];
    let action: Action = plan_invocation(arguments.as_slice(), world.environment.as_slice(), &inputs(world.root.as_path()));
    assert!(!matches!(action, Action::Exit { code: GIT_FATAL, .. }), "{action:?}");
    remove(world.root.as_path());
}

/// With the identity file removed the same invocation goes on to Git:
/// the gate is not the only reason a command fails.
#[test]
fn without_the_identity_file_the_dangling_path_goes_on_to_git() {
    let world: World = world("entry-link-no-identity");
    let home: PathBuf = world.root.join("home");
    std::fs::remove_file(home.join(IDENTITY_RELATIVE_PATH)).expect("remove identity");
    let arguments: Vec<OsString> = vec![OsString::from("-C"), world.link.clone().into_os_string(), OsString::from("status")];
    let action: Action = plan_invocation(arguments.as_slice(), world.environment.as_slice(), &inputs(world.root.as_path()));
    assert!(!matches!(&action, Action::Exit { code: GIT_FATAL, stderr, .. } if stderr.contains("860")), "{action:?}");
    let _unused: &Path = home.as_path();
    remove(world.root.as_path());
}
