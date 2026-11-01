//! Controls for the unit templates the wrapper provisions.

use crate::service_provision::{OWN_MARKERS, SERVICE_UNIT, SOCKET_UNIT};
use crate::service_route::SOCKET_RELATIVE;


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

/// Both templates carry the marker that lets the wrapper recognize its own units, on the first line.
#[test]
fn the_templates_carry_the_marker_on_the_first_line() {
    for text in [SOCKET_UNIT, SERVICE_UNIT] {
        let first: &str = text.lines().next().expect("a first line");
        assert!(first.starts_with('#') && first.contains(OWN_MARKERS[0]), "{first}");
    }
}
