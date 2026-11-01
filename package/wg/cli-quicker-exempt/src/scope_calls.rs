//! Builds the `busctl` arguments for the calls to the user's service manager:
//! creating the updates slice,
//! creating a scope that adopts processes,
//! attaching processes to a scope that exists,
//! giving a live scope new limits,
//! and stopping an empty scope.
//! All are calls of `org.freedesktop.systemd1.Manager` on the user bus.

/// The limits a scope carries.
use crate::inherited_limits::InheritedLimits;

/// What `systemctl status` shows for every scope the watcher creates.
const DESCRIPTION: &str = "wg-quicker update traffic that bypasses the tunnel";

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

/// Builds the arguments for `StartTransientUnit` of a slice.
pub fn ensure_slice_arguments(slice: &str) -> Vec<String> {
    let mut arguments = manager_call("StartTransientUnit", "ssa(sv)a(sa(sv))");
    arguments.push(slice.to_string());
    arguments.push("fail".to_string());
    arguments.push("1".to_string());
    arguments.extend(["Description", "s"].map(str::to_string));
    arguments.push(DESCRIPTION.to_string());
    arguments.push("0".to_string());
    return arguments;
}

/// Builds the arguments for `StartTransientUnit` of a scope that adopts `pids` and carries `limits`.
/// The scope is delegated,
/// because the manager refuses `AttachProcessesToUnit` for a unit that is not
/// (`bus_unit_method_attach_processes` in systemd v259.10 `src/core/dbus-unit.c`,
/// line 1592),
/// and `CollectMode=inactive-or-failed` makes the manager forget the scope as soon as it stops or fails,
/// so a scope that failed to start leaves no unit behind.
pub fn create_arguments(scope_name: &str, slice: &str, pids: &[i32], limits: &InheritedLimits) -> Vec<String> {
    let limit_properties = limits.properties();
    let mut arguments = manager_call("StartTransientUnit", "ssa(sv)a(sa(sv))");
    arguments.push(scope_name.to_string());
    arguments.push("fail".to_string());
    arguments.push((5 + limit_properties.len()).to_string());
    arguments.extend(["Slice", "s"].map(str::to_string));
    arguments.push(slice.to_string());
    arguments.extend(["PIDs", "au"].map(str::to_string));
    arguments.push(pids.len().to_string());
    arguments.extend(pids.iter().map(i32::to_string));
    arguments.extend(["Description", "s"].map(str::to_string));
    arguments.push(DESCRIPTION.to_string());
    arguments.extend(["Delegate", "b", "true"].map(str::to_string));
    arguments.extend(["CollectMode", "s", "inactive-or-failed"].map(str::to_string));
    for (name, value) in limit_properties {
        push_number(&mut arguments, name, value);
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
/// and the I/O weight back to the kernel default,
/// because leaving a property out would keep the old value.
pub fn set_properties_arguments(scope_name: &str, limits: &InheritedLimits) -> Vec<String> {
    let properties = limits.all_properties();
    let mut arguments = manager_call("SetUnitProperties", "sba(sv)");
    arguments.push(scope_name.to_string());
    arguments.push("true".to_string());
    arguments.push(properties.len().to_string());
    for (name, value) in properties {
        push_number(&mut arguments, name, value);
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
