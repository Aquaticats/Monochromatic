//! Builds the `busctl` arguments for the calls to the user's service manager:
//! creating the updates slice,
//! creating a scope that adopts processes,
//! attaching processes to a scope that exists,
//! giving a live scope new limits,
//! and stopping an empty scope.
//! All are calls of `org.freedesktop.systemd1.Manager` on the user manager's private socket.

/// The limits a scope carries.
use crate::inherited_limits::InheritedLimits;
/// Escaping of untrusted text.
use crate::log_line::sanitize;
/// Why a process is kept outside the tunnel.
use crate::update_entry_points::MatchReason;

/// What `systemctl status` shows for the slice that holds the scopes.
const SLICE_DESCRIPTION: &str = "wg-quicker: scopes holding the processes whose traffic leaves outside the WireGuard tunnel";

/// What `systemctl status` shows for one scope:
/// the cgroup it was adopted from and the method that moved it,
/// so no one reads a test run's scope as an update run of its own.
pub fn scope_description(origin: &str, reason: MatchReason) -> String {
    return sanitize(&format!("wg-quicker: adopted from {origin} (moved because it is {})", reason.method_label()));
}

/// The arguments every manager call starts with.
fn manager_call(method: &str, signature: &str) -> Vec<String> {
    return vec![
        "--user".to_string(),
        "call".to_string(),
        "org.freedesktop.systemd1".to_string(),
        "/org/freedesktop/systemd1".to_string(),
        "org.freedesktop.systemd1.Manager".to_string(),
        method.to_string(),
        signature.to_string(),
    ];
}

/// Appends one `t` (unsigned 64-bit) property to a `StartTransientUnit` or `SetUnitProperties` call.
fn push_number(arguments: &mut Vec<String>, name: &str, value: u64) {
    arguments.push(name.to_string());
    arguments.push("t".to_string());
    arguments.push(value.to_string());
}

/// Appends one per-device property:
/// the name,
/// the type `a(st)`,
/// the number of entries,
/// and each entry as a device path and a number.
fn push_device_array(arguments: &mut Vec<String>, name: &str, entries: &[(String, u64)]) {
    arguments.push(name.to_string());
    arguments.push("a(st)".to_string());
    arguments.push(entries.len().to_string());
    for (path, value) in entries {
        arguments.push(path.clone());
        arguments.push(value.to_string());
    }
}

/// Builds the arguments for `StartTransientUnit` of a slice.
pub fn ensure_slice_arguments(slice: &str) -> Vec<String> {
    let mut arguments = manager_call("StartTransientUnit", "ssa(sv)a(sa(sv))");
    arguments.push(slice.to_string());
    arguments.push("fail".to_string());
    arguments.push("1".to_string());
    arguments.extend(["Description", "s"].map(str::to_string));
    arguments.push(SLICE_DESCRIPTION.to_string());
    arguments.push("0".to_string());
    return arguments;
}

/// Builds the arguments for `StartTransientUnit` of a scope that adopts `pids` and carries `limits`.
/// `description` names the origin cgroup and the method that moved the processes,
/// which is what `systemctl status` shows for the scope.
/// The scope is delegated,
/// because the manager refuses `AttachProcessesToUnit` for a unit that is not
/// (`bus_unit_method_attach_processes` in systemd v259.10 `src/core/dbus-unit.c`,
/// line 1592),
/// and `CollectMode=inactive-or-failed` makes the manager forget the scope as soon as it stops or fails,
/// so a scope that failed to start leaves no unit behind.
pub fn create_arguments(scope_name: &str, slice: &str, pids: &[i32], limits: &InheritedLimits, description: &str) -> Vec<String> {
    let limit_properties = limits.properties();
    let mut device_properties: Vec<(&'static str, Vec<(String, u64)>)> = Vec::new();
    for (name, entries) in limits.device_properties() {
        if !entries.is_empty() {
            device_properties.push((name, entries));
        }
    }
    let mut arguments = manager_call("StartTransientUnit", "ssa(sv)a(sa(sv))");
    arguments.push(scope_name.to_string());
    arguments.push("fail".to_string());
    arguments.push((5 + limit_properties.len() + device_properties.len()).to_string());
    arguments.extend(["Slice", "s"].map(str::to_string));
    arguments.push(slice.to_string());
    arguments.extend(["PIDs", "au"].map(str::to_string));
    arguments.push(pids.len().to_string());
    arguments.extend(pids.iter().map(i32::to_string));
    arguments.extend(["Description", "s"].map(str::to_string));
    arguments.push(description.to_string());
    arguments.extend(["Delegate", "b", "true"].map(str::to_string));
    arguments.extend(["CollectMode", "s", "inactive-or-failed"].map(str::to_string));
    for (name, value) in limit_properties {
        push_number(&mut arguments, name, value);
    }
    for (name, entries) in &device_properties {
        push_device_array(&mut arguments, name, entries);
    }
    arguments.push("0".to_string());
    return arguments;
}

/// Builds the arguments for `AttachProcessesToUnit`:
/// the unit name,
/// an empty subgroup,
/// and the pids.
/// A process that is already in the scope stays where it is.
pub fn attach_arguments(scope_name: &str, pids: &[i32]) -> Vec<String> {
    let mut arguments = manager_call("AttachProcessesToUnit", "ssau");
    arguments.push(scope_name.to_string());
    arguments.push(String::new());
    arguments.push(pids.len().to_string());
    arguments.extend(pids.iter().map(i32::to_string));
    return arguments;
}

/// Builds the arguments for `SetUnitProperties` with `runtime` set,
/// so the live scope gets every limit again:
/// a limit the origin no longer sets goes back to unlimited,
/// the I/O weight back to the kernel default,
/// and a kind of per-device limit that no device has any more is sent as an empty array,
/// because leaving a property out would keep the old value.
pub fn set_properties_arguments(scope_name: &str, limits: &InheritedLimits) -> Vec<String> {
    let properties = limits.all_properties();
    let device_properties = limits.device_properties();
    let mut arguments = manager_call("SetUnitProperties", "sba(sv)");
    arguments.push(scope_name.to_string());
    arguments.push("true".to_string());
    arguments.push((properties.len() + device_properties.len()).to_string());
    for (name, value) in properties {
        push_number(&mut arguments, name, value);
    }
    for (name, entries) in &device_properties {
        push_device_array(&mut arguments, name, entries);
    }
    return arguments;
}

/// Builds the arguments for `StopUnit` of a scope.
pub fn stop_arguments(scope_name: &str) -> Vec<String> {
    let mut arguments = manager_call("StopUnit", "ss");
    arguments.push(scope_name.to_string());
    arguments.push("replace".to_string());
    return arguments;
}
