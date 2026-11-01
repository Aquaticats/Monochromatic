//! Controls for the choice that turns the service path on.

use super::*;

const RUNTIME: &str = "/run/user/1000";

/// A unit directory, with the socket unit in it when `installed`; one per control so the two states never share files.
fn units(control: &str, installed: bool) -> PathBuf {
    let directory: PathBuf = crate::test_support::fixture(format!("service-choice-{control}-{installed}").as_str());
    if installed {
        std::fs::write(directory.join(SOCKET_UNIT_NAME), b"unit").expect("socket unit");
    }
    return directory;
}

/// Unset uses the service exactly when the activation is installed; the answer comes from the files, not a process.
#[test]
fn unset_follows_the_installed_activation() {
    let runtime: PathBuf = PathBuf::from(RUNTIME);
    let present: PathBuf = units("unset", true);
    let absent: PathBuf = units("unset", false);
    assert_eq!(
        choose_service(None, Some(runtime.as_path()), Some(present.as_path())),
        Some((Bind::Path(runtime.join(SOCKET_RELATIVE)), Activation::Managed))
    );
    assert_eq!(choose_service(None, Some(runtime.as_path()), Some(absent.as_path())), None, "an uninstalled machine keeps its path");
    assert_eq!(choose_service(None, Some(runtime.as_path()), None), None);
    assert_eq!(choose_service(None, None, Some(present.as_path())), None, "no runtime directory, no address");
}

/// `1` asks for the service and gets the plain not-installed stop; `0` and unknown spellings never use it.
#[test]
fn the_explicit_values_decide_regardless_of_the_installation() {
    let runtime: PathBuf = PathBuf::from(RUNTIME);
    let present: PathBuf = units("explicit", true);
    let absent: PathBuf = units("explicit", false);
    let address: Bind = Bind::Path(runtime.join(SOCKET_RELATIVE));
    assert_eq!(choose_service(Some("1"), Some(runtime.as_path()), Some(present.as_path())), Some((address.clone(), Activation::Managed)));
    assert_eq!(choose_service(Some("1"), Some(runtime.as_path()), Some(absent.as_path())), Some((address, Activation::NotInstalled)));
    assert_eq!(choose_service(Some("1"), None, Some(present.as_path())), None);
    assert_eq!(choose_service(Some("abstract:name"), None, None), Some((Bind::Abstract(String::from("name")), Activation::Managed)));
    for value in ["0", "", "yes", "true", "abstract:", "path:/x"] {
        assert_eq!(choose_service(Some(value), Some(runtime.as_path()), Some(present.as_path())), None, "{value:?}");
    }
}

/// A socket that exists counts as installed, whoever made it.
#[test]
fn an_existing_socket_counts_as_installed() {
    let runtime: PathBuf = crate::test_support::fixture("service-choice-socket");
    let socket: PathBuf = runtime.join(SOCKET_RELATIVE);
    std::fs::create_dir_all(socket.parent().expect("parent")).expect("directory");
    std::fs::write(&socket, b"").expect("socket stand-in");
    assert_eq!(choose_service(None, Some(runtime.as_path()), None), Some((Bind::Path(socket), Activation::Managed)));
}

const SOCKET_UNIT: &str = include_str!("service_units/cli-git.socket");
const SERVICE_UNIT: &str = include_str!("service_units/cli-git.service");

/// The socket unit listens where the wrapper looks, privately, and the service unit is bounded and ordered after it.
#[test]
fn the_activation_units_match_the_wrapper_and_the_resource_rules() {
    assert!(SOCKET_UNIT.lines().any(|line| { return line == format!("ListenStream=%t/{SOCKET_RELATIVE}"); }), "{SOCKET_UNIT}");
    assert!(SOCKET_UNIT.contains("SocketMode=0600") && SOCKET_UNIT.contains("DirectoryMode=0700"));
    assert!(SERVICE_UNIT.contains("Requires=cli-git.socket") && SERVICE_UNIT.contains("After=cli-git.socket"));
    for property in ["MemoryMax=", "TasksMax=", "CPUQuota=", "IOWeight="] {
        assert!(SERVICE_UNIT.lines().any(|line| { return line.starts_with(property); }), "{property}");
    }
    let weight: u32 = SERVICE_UNIT
        .lines()
        .find_map(|line| { return line.strip_prefix("IOWeight="); })
        .and_then(|value| { return value.trim().parse().ok(); })
        .expect("a numeric IOWeight");
    assert!(weight <= 50, "the owner's rule: never above 50");
    assert!(SERVICE_UNIT.contains("@SERVICE_BINARY@") && SERVICE_UNIT.contains("@GIT@"), "the installer fills these in");
}
