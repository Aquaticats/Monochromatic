//! Controls for the route decision: an existing socket is used as it is, the session bus and the service program together
//!  mean the wrapper provisions, and anything less has no route.

use super::*;
use crate::test_support::{fixture, remove};
use crate::service_provision::SERVICE_PROGRAM;

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
    let program: PathBuf = machine.wrapper.parent().expect("programs").join(SERVICE_PROGRAM);
    std::fs::write(&program, b"program").expect("service program");
    return program;
}

/// A socket that exists is used as it is, with nothing to set up.
#[test]
fn an_existing_socket_is_used_as_it_is() {
    let machine: Machine = machine("route-socket");
    let socket: PathBuf = socket_for(machine.runtime.as_path(), machine.git_dir.as_path());
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("directory");
    std::fs::write(&socket, b"").expect("socket stand-in");
    assert_eq!(choose_route(&inputs(&machine, None)), Some(Route { address: Bind::Path(socket), provision: None }));
    remove(machine.root.as_path());
}

/// A socket that exists does not end the wrapper's care for the units: the socket is the manager's and outlives an upgrade,
///  so with the bus and the program the provisioning check is still handed back, and `provision` decides from the files.
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

/// The session bus and the service program together mean the wrapper sets the service up itself, from the facts it found.
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

/// Without the bus, or without the program, or without a runtime directory, and with no socket, there is no route.
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

/// Each worktree has its own socket and its own instance, so two worktrees of one repository are never routed to one service,
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

/// The user unit directory is the configuration home's, else the home's `.config`.
#[test]
fn the_user_unit_directory_follows_the_configuration_home() {
    assert_eq!(user_unit_dir(Some(Path::new("/cfg")), Some(Path::new("/home/a"))), Some(PathBuf::from("/cfg/systemd/user")));
    assert_eq!(user_unit_dir(None, Some(Path::new("/home/a"))), Some(PathBuf::from("/home/a/.config/systemd/user")));
    assert_eq!(user_unit_dir(None, None), None);
}
