//! Builds the calls the watcher makes to the user's service manager:
//! creating the updates slice,
//! creating a scope that adopts processes,
//! attaching processes to a scope that exists,
//! giving a live scope new limits,
//! and stopping an empty scope.
//! Each names `org.freedesktop.systemd1.Manager`;
//! the call itself goes over that manager's own private socket,
//! where the manager is the only process that can answer,
//! never over a bus name another process could take.

/// The property values and arguments of a call.
use crate::dbus_wire::{Argument, MethodCall, MANAGER_INTERFACE, Property, PropertyValue};
/// The limits a scope carries.
use crate::inherited_limits::InheritedLimits;
/// Escaping of untrusted text.
use crate::log_line::sanitize;
/// Why a process is kept outside the tunnel.
use crate::update_entry_points::MatchReason;

/// What `systemctl status` shows for the slice that holds the scopes.
const SLICE_DESCRIPTION: &str = "wg-quicker: scopes holding the processes whose traffic leaves outside the WireGuard tunnel";
/// How a unit of the earlier design was started,
/// which a slice the manager already holds keeps.
const TRANSIENT_MODE: &str = "fail";
/// How a stop applies to the named unit,
/// replacing a queued job for it.
const STOP_MODE: &str = "replace";

/// What `systemctl status` shows for one scope:
/// the cgroup it was adopted from and the method that moved it,
/// so no one reads a test run's scope as an update run of its own.
pub fn scope_description(origin: &str, reason: MatchReason) -> String {
    return sanitize(&format!("wg-quicker: adopted from {origin} (moved because it is {})", reason.method_label()));
}

/// The pids as the unsigned numbers the wire carries,
/// which a negative pid keeps its bytes through.
fn as_unsigned(pids: &[i32]) -> Vec<u32> {
    let mut numbers: Vec<u32> = Vec::with_capacity(pids.len());
    for pid in pids {
        numbers.push(u32::from_ne_bytes(pid.to_ne_bytes()));
    }
    return numbers;
}

/// One call of the manager's interface.
fn manager_call(member: &'static str, arguments: Vec<Argument>) -> MethodCall {
    return MethodCall { interface: MANAGER_INTERFACE, member, arguments };
}

/// One `(sv)` property holding a 64-bit number.
fn number_property(name: &str, value: u64) -> Property {
    return Property { name: name.to_string(), value: PropertyValue::Number(value) };
}

/// One `(sv)` property holding per-device limits.
fn devices_property(name: &str, entries: Vec<(String, u64)>) -> Property {
    return Property { name: name.to_string(), value: PropertyValue::Devices(entries) };
}

/// The properties that carry `limits` onto a new scope.
/// A limit the origin does not set is left out,
/// so the manager's own default applies instead of a value read from nowhere.
fn limit_properties(limits: &InheritedLimits) -> Vec<Property> {
    let mut properties: Vec<Property> = Vec::new();
    for (name, value) in limits.properties() {
        properties.push(number_property(name, value));
    }
    for (name, entries) in limits.device_properties() {
        if !entries.is_empty() {
            properties.push(devices_property(name, entries));
        }
    }
    return properties;
}

/// The call that creates the updates slice when the manager does not hold it.
pub fn ensure_slice_call(slice: &str) -> MethodCall {
    let properties = vec![Property { name: "Description".to_string(), value: PropertyValue::Text(SLICE_DESCRIPTION.to_string()) }];
    return manager_call(
        "StartTransientUnit",
        vec![Argument::Text(slice.to_string()), Argument::Text(TRANSIENT_MODE.to_string()), Argument::Properties(properties), Argument::NoAuxUnits],
    );
}

/// The call that creates a scope adopting `pids` under `slice`,
/// carrying `limits`,
/// and `description` naming the origin and the method that moved them.
/// The scope is delegated,
/// because the manager refuses `AttachProcessesToUnit` for a unit that is not
/// (`bus_unit_method_attach_processes` in systemd v259.10 `src/core/dbus-unit.c`,
/// line 1592),
/// and `CollectMode=inactive-or-failed` makes the manager forget the scope as soon as it stops or fails,
/// so a scope that failed to start leaves no unit behind.
pub fn create_call(scope_name: &str, slice: &str, pids: &[i32], limits: &InheritedLimits, description: &str) -> MethodCall {
    let adopted: Vec<u32> = as_unsigned(pids);
    let mut properties: Vec<Property> = vec![
        Property { name: "Slice".to_string(), value: PropertyValue::Text(slice.to_string()) },
        Property { name: "PIDs".to_string(), value: PropertyValue::Pids(adopted) },
        Property { name: "Description".to_string(), value: PropertyValue::Text(description.to_string()) },
        Property { name: "Delegate".to_string(), value: PropertyValue::Flag(true) },
        Property { name: "CollectMode".to_string(), value: PropertyValue::Text("inactive-or-failed".to_string()) },
    ];
    properties.extend(limit_properties(limits));
    return manager_call(
        "StartTransientUnit",
        vec![Argument::Text(scope_name.to_string()), Argument::Text(TRANSIENT_MODE.to_string()), Argument::Properties(properties), Argument::NoAuxUnits],
    );
}

/// The call that attaches `pids` to the scope's root subgroup.
/// A process that is already in the scope stays where it is.
pub fn attach_call(scope_name: &str, pids: &[i32]) -> MethodCall {
    let attached: Vec<u32> = as_unsigned(pids);
    return manager_call("AttachProcessesToUnit", vec![Argument::Text(scope_name.to_string()), Argument::Text(String::new()), Argument::Numbers(attached)]);
}

/// The call that gives a live scope every limit again,
/// because a property left out would keep its old value:
/// a limit the origin no longer sets goes back to unlimited,
/// the I/O weight back to the kernel default,
/// and a kind of per-device limit that no device has any more is sent as an empty array.
pub fn set_properties_call(scope_name: &str, limits: &InheritedLimits) -> MethodCall {
    let mut properties: Vec<Property> = Vec::new();
    for (name, value) in limits.all_properties() {
        properties.push(number_property(name, value));
    }
    for (name, entries) in limits.device_properties() {
        properties.push(devices_property(name, entries));
    }
    return manager_call("SetUnitProperties", vec![Argument::Text(scope_name.to_string()), Argument::Flag(true), Argument::Properties(properties)]);
}

/// The call that stops a scope.
pub fn stop_call(scope_name: &str) -> MethodCall {
    return manager_call("StopUnit", vec![Argument::Text(scope_name.to_string()), Argument::Text(STOP_MODE.to_string())]);
}
