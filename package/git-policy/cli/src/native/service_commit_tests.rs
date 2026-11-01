//! Controls for the opt-in that turns the service path on.

use super::*;

/// Only the two explicit spellings turn it on, and only with what they need.
#[test]
fn only_the_explicit_values_turn_the_service_path_on() {
    let runtime: PathBuf = PathBuf::from("/run/user/1000");
    assert_eq!(service_address("1", Some(runtime.as_path())), Some(Bind::Path(runtime.join(SOCKET_RELATIVE))));
    assert_eq!(service_address("1", None), None, "no runtime directory, no address");
    assert_eq!(service_address("abstract:name", None), Some(Bind::Abstract(String::from("name"))));
    for value in ["", "0", "yes", "true", "abstract:", "path:/x"] {
        assert_eq!(service_address(value, Some(runtime.as_path())), None, "{value:?}");
    }
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
