//! Controls for the route decision: an existing socket is used as it is, the session bus and the service program together
//!  mean the wrapper provisions, and anything less keeps the earlier path.

use super::*;
use crate::test_support::{fixture, remove};
use crate::service_provision::SERVICE_PROGRAM;

const GIT: &str = "/usr/bin/git";

struct Machine {
    root: PathBuf,
    runtime: PathBuf,
    wrapper: PathBuf,
}

fn machine(name: &str) -> Machine {
    let root: PathBuf = fixture(name);
    let runtime: PathBuf = root.join("run");
    let programs: PathBuf = root.join("programs");
    std::fs::create_dir_all(&runtime).expect("runtime");
    std::fs::create_dir_all(&programs).expect("programs");
    return Machine { wrapper: programs.join("git"), runtime, root };
}

fn inputs<'a>(machine: &'a Machine, bus_address: Option<&'a str>) -> RouteInputs<'a> {
    return RouteInputs {
        runtime_dir: Some(machine.runtime.as_path()),
        bus_address,
        config_home: None,
        home: Some(machine.root.as_path()),
        own_executable: Some(machine.wrapper.as_path()),
        real_git: Path::new(GIT),
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
    let socket: PathBuf = machine.runtime.join(SOCKET_RELATIVE);
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("directory");
    std::fs::write(&socket, b"").expect("socket stand-in");
    assert_eq!(choose_route(&inputs(&machine, None)), Route::Service { address: Bind::Path(socket), provision: None });
    remove(machine.root.as_path());
}

/// The session bus and the service program together mean the wrapper sets the service up itself, from the facts it found.
#[test]
fn the_bus_and_the_program_mean_provisioning() {
    let machine: Machine = machine("route-provision");
    let address: String = add_bus(&machine);
    let program: PathBuf = add_program(&machine);
    let route: Route = choose_route(&inputs(&machine, Some(address.as_str())));
    let Route::Service { address: bind, provision: Some(host) } = route else { panic!("provisioning expected") };
    assert_eq!(bind, Bind::Path(machine.runtime.join(SOCKET_RELATIVE)));
    assert_eq!(host.service_binary, program);
    assert_eq!(host.git, PathBuf::from(GIT));
    assert_eq!(host.runtime_dir, machine.runtime);
    assert_eq!(host.bus, machine.runtime.join("bus"));
    assert_eq!(host.user_unit_dir, Some(machine.root.join(".config").join("systemd").join("user")));
    remove(machine.root.as_path());
}

/// Without the bus, or without the program, or without a runtime directory, the earlier path stays.
#[test]
fn anything_less_keeps_the_earlier_path() {
    let machine: Machine = machine("route-earlier");
    assert_eq!(choose_route(&inputs(&machine, None)), Route::Earlier, "no bus, no program");
    let address: String = add_bus(&machine);
    assert_eq!(choose_route(&inputs(&machine, Some(address.as_str()))), Route::Earlier, "a bus but no program");
    add_program(&machine);
    assert_eq!(choose_route(&RouteInputs { runtime_dir: None, ..inputs(&machine, Some(address.as_str())) }), Route::Earlier, "no runtime directory");
    assert_eq!(choose_route(&RouteInputs { own_executable: None, ..inputs(&machine, Some(address.as_str())) }), Route::Earlier, "no executable to look beside");
    std::fs::remove_file(machine.runtime.join("bus")).expect("bus gone");
    assert_eq!(choose_route(&inputs(&machine, Some(address.as_str()))), Route::Earlier, "a program but no bus");
    remove(machine.root.as_path());
}

/// The user unit directory is the configuration home's, else the home's `.config`.
#[test]
fn the_user_unit_directory_follows_the_configuration_home() {
    assert_eq!(user_unit_dir(Some(Path::new("/cfg")), Some(Path::new("/home/a"))), Some(PathBuf::from("/cfg/systemd/user")));
    assert_eq!(user_unit_dir(None, Some(Path::new("/home/a"))), Some(PathBuf::from("/home/a/.config/systemd/user")));
    assert_eq!(user_unit_dir(None, None), None);
}
