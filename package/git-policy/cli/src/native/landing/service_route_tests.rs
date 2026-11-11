//! Controls for the route decision:
//! an existing socket is used as it is,
//! the session bus and the wrapper's own executable together
//!  mean the wrapper provisions,
//! and anything less has no route.

use super::*;
use crate::service_instance::socket_for;
use crate::test_support::{fixture, remove};

const GIT: &str = "/usr/bin/git";

struct Machine {
    root: PathBuf,
    runtime: PathBuf,
    wrapper: PathBuf,
    /// The worktree's own Git directory.
    git_dir: PathBuf,
}

fn machine(name: &str) -> Machine {
    let root: PathBuf = fixture(name);
    let runtime: PathBuf = root.join("run");
    let programs: PathBuf = root.join("programs");
    std::fs::create_dir_all(&runtime).expect("runtime");
    std::fs::create_dir_all(&programs).expect("programs");
    let git_dir: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&git_dir).expect("git directory");
    return Machine { wrapper: programs.join("git"), runtime, root, git_dir };
}

fn inputs<'a>(machine: &'a Machine, bus_address: Option<&'a str>) -> RouteInputs<'a> {
    return RouteInputs {
        runtime_dir: Some(machine.runtime.as_path()),
        bus_address,
        config_home: None,
        home: Some(machine.root.as_path()),
        own_executable: Some(machine.wrapper.as_path()),
        real_git: Path::new(GIT),
        git_dir: machine.git_dir.as_path(),
    };
}

fn add_bus(machine: &Machine) -> String {
    let bus: PathBuf = machine.runtime.join("bus");
    std::fs::write(&bus, b"").expect("bus stand-in");
    return format!("unix:path={}", bus.display());
}

fn add_program(machine: &Machine) -> PathBuf {
    // The service is a mode of the wrapper's own executable, so the wrapper is the program the unit names.
    std::fs::write(&machine.wrapper, b"program").expect("the wrapper's own executable");
    return machine.wrapper.clone();
}

/// A socket that exists is used as it is,
/// with nothing to set up.
#[test]
fn an_existing_socket_is_used_as_it_is() {
    let machine: Machine = machine("route-socket");
    let socket: PathBuf = socket_for(machine.runtime.as_path(), machine.git_dir.as_path());
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("directory");
    std::fs::write(&socket, b"").expect("socket stand-in");
    assert_eq!(choose_route(&inputs(&machine, None)), Some(Route { address: Bind::Path(socket), provision: None }));
    remove(machine.root.as_path());
}

/// A socket that exists does not end the wrapper's care for the units:
/// the socket is the manager's and outlives an upgrade,
///  so with the bus and the program the provisioning check is still handed back,
/// and `provision` decides from the files.
#[test]
fn an_existing_socket_still_hands_back_the_provisioning_check() {
    let machine: Machine = machine("route-socket-check");
    let socket: PathBuf = socket_for(machine.runtime.as_path(), machine.git_dir.as_path());
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("directory");
    std::fs::write(&socket, b"").expect("socket stand-in");
    let address: String = add_bus(&machine);
    let program: PathBuf = add_program(&machine);
    let route: Option<Route> = choose_route(&inputs(&machine, Some(address.as_str())));
    let Some(Route { address: bind, provision: Some(host) }) = route else { panic!("the provisioning check expected") };
    assert_eq!(bind, Bind::Path(socket));
    assert_eq!(host.service_binary, program);
    remove(machine.root.as_path());
}

/// The session bus and the service program together mean the wrapper sets the service up itself,
/// from the facts it found.
#[test]
fn the_bus_and_the_program_mean_provisioning() {
    let machine: Machine = machine("route-provision");
    let address: String = add_bus(&machine);
    let program: PathBuf = add_program(&machine);
    let route: Option<Route> = choose_route(&inputs(&machine, Some(address.as_str())));
    let Some(Route { address: bind, provision: Some(host) }) = route else { panic!("provisioning expected") };
    assert_eq!(bind, Bind::Path(socket_for(machine.runtime.as_path(), machine.git_dir.as_path())));
    assert_eq!(host.service_binary, program);
    assert_eq!(host.git, PathBuf::from(GIT));
    assert_eq!(host.runtime_dir, machine.runtime);
    assert_eq!(host.bus, machine.runtime.join("bus"));
    assert_eq!(host.user_unit_dir, Some(machine.root.join(".config").join("systemd").join("user")));
    remove(machine.root.as_path());
}

/// Without the bus,
/// or without the program,
/// or without a runtime directory,
/// and with no socket,
/// there is no route.
#[test]
fn anything_less_has_no_route() {
    let machine: Machine = machine("route-none");
    assert_eq!(choose_route(&inputs(&machine, None)), None, "no bus, no program");
    let address: String = add_bus(&machine);
    assert_eq!(choose_route(&inputs(&machine, Some(address.as_str()))), None, "a bus but no program");
    add_program(&machine);
    assert_eq!(choose_route(&RouteInputs { runtime_dir: None, ..inputs(&machine, Some(address.as_str())) }), None, "no runtime directory");
    assert_eq!(choose_route(&RouteInputs { own_executable: None, ..inputs(&machine, Some(address.as_str())) }), None, "no executable to look beside");
    std::fs::remove_file(machine.runtime.join("bus")).expect("bus gone");
    assert_eq!(choose_route(&inputs(&machine, Some(address.as_str()))), None, "a program but no bus");
    remove(machine.root.as_path());
}

/// Each worktree has its own socket and its own instance,
/// so two worktrees of one repository are never routed to one service,
///  and one worktree is routed to the same one however its commit is started.
#[test]
fn two_worktrees_are_routed_to_two_services() {
    let machine: Machine = machine("route-two-worktrees");
    let address: String = add_bus(&machine);
    add_program(&machine);
    let linked: PathBuf = machine.git_dir.join("worktrees/linked");
    std::fs::create_dir_all(&linked).expect("linked worktree's git directory");
    let first = choose_route(&inputs(&machine, Some(address.as_str())));
    let second = choose_route(&RouteInputs { git_dir: linked.as_path(), ..inputs(&machine, Some(address.as_str())) });
    let Some(Route { address: first_socket, provision: Some(first_host) }) = first.clone() else { panic!("provisioning expected") };
    let Some(Route { address: second_socket, provision: Some(second_host) }) = second else { panic!("provisioning expected") };
    assert_ne!(first_socket, second_socket, "two worktrees, two sockets");
    assert_ne!(first_host.instance, second_host.instance, "two worktrees, two unit instances");
    assert_eq!(first, choose_route(&inputs(&machine, Some(address.as_str()))), "one worktree, one route");
    remove(machine.root.as_path());
}

/// The user unit directory is the configuration home's,
/// else the home's `.config`.
#[test]
fn the_user_unit_directory_follows_the_configuration_home() {
    assert_eq!(user_unit_dir(Some(Path::new("/cfg")), Some(Path::new("/home/a"))), Some(PathBuf::from("/cfg/systemd/user")));
    assert_eq!(user_unit_dir(None, Some(Path::new("/home/a"))), Some(PathBuf::from("/home/a/.config/systemd/user")));
    assert_eq!(user_unit_dir(None, None), None);
}

/// A Git directory that cannot be resolved has no route even when the runtime directory,
/// the bus and the program are all there:
/// no socket path or unit instance is named for it,
/// and nothing is created under the runtime directory.
#[test]
fn an_unreachable_git_directory_has_no_route_and_names_nothing() {
    let machine: Machine = machine("route-unreachable");
    let address: String = add_bus(&machine);
    add_program(&machine);
    let link: PathBuf = machine.root.join("dangling");
    std::os::unix::fs::symlink(machine.root.join("no-such-target"), &link).expect("dangling link");
    let git_dir: PathBuf = link.join(".git");
    let before: Vec<PathBuf> = entries_of(machine.runtime.as_path());
    let mut route_inputs: RouteInputs<'_> = inputs(&machine, Some(address.as_str()));
    route_inputs.git_dir = git_dir.as_path();
    assert_eq!(choose_route(&route_inputs), None);
    assert_eq!(entries_of(machine.runtime.as_path()), before, "nothing was created");
    remove(machine.root.as_path());
}

/// What:
///  The worktree's Git directory asked of Git is the bytes Git printed,
///  so the service instance has a name for a repository whose directory is not UTF-8.
/// Why:
///   The instance is named from that path,
///  and a spelling of it named a directory that does not exist,
///  so the commit was refused as "the Git directory cannot be resolved".
#[test]
fn the_worktree_git_directory_keeps_a_path_that_is_not_text() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("route-git-dir-not-utf8");
    let repo: PathBuf = root.join(std::ffi::OsString::from_vec(b"rep\xefo".to_vec()));
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    crate::test_support::git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    // Global options name the repository, so the in-process reader leaves the answer to Git.
    let context: GitContext = GitContext {
        real_git: PathBuf::from(GIT),
        overlay: crate::test_support::isolated_overlay(),
        global_prefix: vec![std::ffi::OsString::from("--git-dir"), repo.join(".git").into_os_string()],
        common_dir: None,
    };
    assert_eq!(
        worktree_git_dir(&context, root.as_path()),
        Some(repo.join(".git")),
        "the Git directory keeps the bytes Git printed"
    );
    remove(root.as_path());
}

/// What:
///  The worktree's Git directory asked of Git keeps a name that ends in a space.
/// Why:
///   Every ASCII whitespace byte was taken from the answer,
///  so the route named a directory that does not exist
/// and the commit had no instance to hand its work to.
#[test]
fn the_worktree_git_directory_keeps_a_name_that_ends_in_a_space() {
    let root: PathBuf = fixture("route-git-dir-space");
    let repo: PathBuf = root.join("repository");
    let administered: PathBuf = root.join("own ");
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    crate::test_support::git(
        root.as_path(),
        &[
            "init",
            "--quiet",
            "--initial-branch=main",
            "--separate-git-dir",
            administered.to_str().expect("a UTF-8 fixture path"),
            repo.to_str().expect("a UTF-8 fixture path"),
        ],
    );
    // Global options name the repository, so the in-process reader leaves the answer to Git.
    let context: GitContext = GitContext {
        real_git: PathBuf::from(GIT),
        overlay: crate::test_support::isolated_overlay(),
        global_prefix: vec![std::ffi::OsString::from("--git-dir"), administered.clone().into_os_string()],
        common_dir: None,
    };
    assert_eq!(
        worktree_git_dir(&context, root.as_path()),
        Some(std::fs::canonicalize(administered.as_path()).expect("the directory Git made")),
        "the Git directory keeps the space that belongs to its name"
    );
    remove(root.as_path());
}

/// The status of a worktree whose Git directory cannot be resolved says the cause,
/// and does not connect to any socket.
#[test]
fn the_status_of_an_unreachable_git_directory_says_the_cause() {
    let machine: Machine = machine("route-status-unreachable");
    let git_dir: PathBuf = machine.root.join("absent/.git");
    let report: StatusReport = service_status(Some(machine.runtime.as_path()), Some(git_dir.as_path()));
    assert_eq!(report.code, 1);
    assert!(report.stderr.contains(git_dir.to_string_lossy().as_ref()), "{}", report.stderr);
    assert!(report.stderr.contains("cannot be resolved"), "{}", report.stderr);
    assert!(!report.stderr.contains("sets itself up"), "{}", report.stderr);
    remove(machine.root.as_path());
}

/// The sorted entries of a folder.
fn entries_of(folder: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(folder).expect("folder").flatten().map(|entry| return entry.path()).collect();
    found.sort();
    return found;
}
