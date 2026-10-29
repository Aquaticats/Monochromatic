//! Project mounts:
//!  the canonical root always,
//!  other spellings only where the sandbox replaced the
//! directory,
//!  all with one mount option,
//!  and all after the replacements.

use super::{PROJECT_MOUNT, project_binds, read_only_binds, replaced_location};
use crate::language::confine::recipe::recipe;
use crate::language::launch::{LaunchRequest, scratch_name};
use std::{collections::HashMap, path::Path, path::PathBuf};

fn request(project: &str, spellings: &[&str]) -> LaunchRequest {
    return LaunchRequest {
        server: "rust-analyzer".to_string(),
        executable: PathBuf::from("/opt/servers/bin/server"),
        args: Vec::new(),
        environment: HashMap::new(),
        settings: None,
        project_root: PathBuf::from(project),
        project_spellings: spellings.iter().map(PathBuf::from).collect(),
        read_only_binds: Vec::new(),
        state_root: Some(PathBuf::from("/state")),
    };
}

/// Position of the first argument equal to `wanted`.
fn position_of(args: &[String], wanted: &str) -> Option<usize> {
    for (index, part) in args.iter().enumerate() {
        if part == wanted {
            return Some(index);
        }
    }
    return None;
}

/// Position of the first argument triple `option source destination`.
fn triple(args: &[String], option: &str, source: &str, destination: &str) -> Option<usize> {
    for (index, window) in args.windows(3).enumerate() {
        if window[0] == option && window[1] == source && window[2] == destination {
            return Some(index);
        }
    }
    return None;
}

#[test]
fn project_below_tmp_and_run_is_bound_back_after_the_replacements() {
    let wanted = request(
        "/tmp/work/project",
        &["/run/media/someone/project", "/home/someone/project"],
    );
    let launch = recipe(
        &wanted,
        Path::new("/state/p/rust-analyzer"),
        "/usr/bin/bwrap",
        &[],
    )
    .expect("recipe");
    let args = &launch.args;
    let mut run = None;
    for (index, window) in args.windows(2).enumerate() {
        if window[0] == "--tmpfs" && window[1] == "/run" {
            run = Some(index);
            break;
        }
    }
    let tmp = triple(
        args,
        "--bind",
        &format!("/state/p/rust-analyzer/{}", scratch_name()),
        "/tmp",
    );
    assert!(
        run.is_some() && tmp.is_some(),
        "the replacements are missing: {args:?}"
    );
    let canonical = triple(args, "--ro-bind", "/tmp/work/project", "/tmp/work/project");
    let media = triple(
        args,
        "--ro-bind",
        "/tmp/work/project",
        "/run/media/someone/project",
    );
    assert!(
        canonical.is_some() && media.is_some(),
        "the project is not bound back at both spellings: {args:?}"
    );
    assert!(
        run < canonical && tmp < canonical && run < media,
        "a replacement comes after the project bind and would hide it: {args:?}"
    );
    assert_eq!(
        triple(
            args,
            "--ro-bind",
            "/tmp/work/project",
            "/home/someone/project"
        ),
        None,
        "a spelling outside the replaced locations resolves through the root and needs no bind"
    );
    let clear = position_of(args, "--clearenv");
    assert!(media < clear, "project binds must precede the environment");
}

#[test]
fn every_project_mount_uses_the_one_mount_option() {
    assert_eq!(
        PROJECT_MOUNT, "--ro-bind",
        "0.x servers must only read the project"
    );
    let options = project_binds(
        Path::new("/var/home/someone/project"),
        &[
            PathBuf::from("/var/home/someone/project"),
            PathBuf::from("/dev/shm/project"),
        ],
    )
    .expect("binds");
    assert_eq!(
        options,
        vec![
            "--ro-bind",
            "/var/home/someone/project",
            "/var/home/someone/project",
            "--ro-bind",
            "/var/home/someone/project",
            "/dev/shm/project",
        ],
        "the canonical root is always bound, a duplicate spelling is not bound twice"
    );
}

#[test]
fn replaced_locations_compare_whole_components() {
    assert_eq!(replaced_location(Path::new("/tmp")), Some("/tmp"));
    assert_eq!(
        replaced_location(Path::new("/run/user/1000/p")),
        Some("/run")
    );
    assert_eq!(replaced_location(Path::new("/dev/shm/p")), Some("/dev"));
    assert_eq!(replaced_location(Path::new("/tmpfoo/p")), None);
    assert_eq!(replaced_location(Path::new("/var/tmp/p")), None);
    assert_eq!(replaced_location(Path::new("/proc/1")), None);
}

/// The home folder opened as the project holds the private state below `~/.cache`;
///  bubblewrap
/// applies mounts in order and a later one covers an earlier one,
///  so the writable state bind must
/// come after the read-only project bind or the server could not write its own state.
#[test]
fn state_inside_the_project_is_bound_writable_after_the_read_only_project() {
    let wanted = request("/var/home/someone", &["/home/someone"]);
    let state = "/var/home/someone/.cache/monochromatic-ide/language/someone-0/rust-analyzer";
    let launch = recipe(&wanted, Path::new(state), "/usr/bin/bwrap", &[]).expect("recipe");
    let args = &launch.args;
    let project = triple(args, "--ro-bind", "/var/home/someone", "/var/home/someone");
    let writable = triple(args, "--bind", state, state);
    assert!(
        project.is_some() && writable.is_some(),
        "the project or the state bind is missing: {args:?}"
    );
    assert!(
        project < writable,
        "the read-only project bind comes after the state bind and would cover it: {args:?}"
    );
    let clear = position_of(args, "--clearenv");
    assert!(writable < clear, "binds must precede the environment");
}

/// Tool directories of mise below a replaced location exist inside only if they are bound again,
/// after the replacements and before the state bind,
/// and never writable.
#[test]
fn read_only_directories_below_replaced_locations_are_bound_back_read_only() {
    let mut wanted = request("/work/project", &[]);
    wanted.read_only_binds = vec![
        PathBuf::from("/tmp/mise data/installs/node/bin"),
        PathBuf::from("/run/user/1000/tools"),
        PathBuf::from("/usr/local/tools"),
    ];
    let state = "/state/p/rust-analyzer";
    let launch = recipe(&wanted, Path::new(state), "/usr/bin/bwrap", &[]).expect("recipe");
    let args = &launch.args;
    let tmp = triple(
        args,
        "--bind",
        &format!("/state/p/rust-analyzer/{}", scratch_name()),
        "/tmp",
    );
    let project = triple(args, "--ro-bind", "/work/project", "/work/project");
    let node = triple(
        args,
        "--ro-bind",
        "/tmp/mise data/installs/node/bin",
        "/tmp/mise data/installs/node/bin",
    );
    let tools = triple(
        args,
        "--ro-bind",
        "/run/user/1000/tools",
        "/run/user/1000/tools",
    );
    let writable = triple(args, "--bind", state, state);
    assert!(
        node.is_some() && tools.is_some(),
        "a tool directory below /tmp or /run is missing: {args:?}"
    );
    assert!(
        tmp < node && project < node && node < writable && tools < writable,
        "the tool directories must come after the replacements and before the state bind: {args:?}"
    );
    // A directory elsewhere is visible through the read-only root and gets no mount of its own.
    assert!(!args.contains(&"/usr/local/tools".to_string()), "{args:?}");
    // No tool directory is ever mounted writable.
    let mut writable_tool = false;
    for window in args.windows(3) {
        if window[0] == "--bind" && window[1].contains("/mise data/") {
            writable_tool = true;
        }
    }
    assert!(!writable_tool, "{args:?}");
}

#[test]
fn only_replaced_locations_need_a_read_only_mount() {
    let directories = vec![
        PathBuf::from("/tmp/a b/-c"),
        PathBuf::from("/dev/shm/tools"),
        PathBuf::from("/tmpfoo/tools"),
        PathBuf::from("/opt/tools"),
    ];
    let options = read_only_binds(&directories).expect("options");
    assert_eq!(
        options,
        [
            "--ro-bind",
            "/tmp/a b/-c",
            "/tmp/a b/-c",
            "--ro-bind",
            "/dev/shm/tools",
            "/dev/shm/tools"
        ]
    );
    assert!(read_only_binds(&[]).expect("none").is_empty());
}
