//! Verifies the shape of the calls the watcher makes to a user's service manager:
//! the method,
//! the order and the signatures of the arguments,
//! the properties that carry the limits,
//! and the description that says which origin and which method moved a process.
//! A signature the manager does not expect is refused by the manager,
//! so the shape is part of the contract and is checked without a manager.

/// The call builders under test.
use crate::scope_calls::{attach_call, create_call, ensure_slice_call, scope_description, set_properties_call, stop_call};
/// The values a call carries.
use crate::dbus_wire::{Argument, Property, PropertyValue, body_signature};
/// The limits a scope carries.
use crate::inherited_limits::{DeviceLimits, InheritedLimits};
/// Why a process matched,
/// which the scope description names.
use crate::update_entry_points::MatchReason;

/// Slice used by the tests.
const SLICE: &str = "app-monochromatic\\x2dupdates.slice";
/// Scope used by the tests.
const SCOPE: &str = "wg-quicker-update-app.slice-claude\\x2dcode\\x2dbash.scope";
/// Origin used by the tests.
const ORIGIN: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash";
/// The description a created scope carries in these tests.
const DESCRIPTION: &str = "wg-quicker: adopted from somewhere (moved because it is a test)";

/// The limits of the cgroup the processes left,
/// all of the kinds a scope carries.
fn all_limits() -> InheritedLimits {
    return InheritedLimits {
        cpu_quota_per_sec_usec: Some(8_000_000),
        memory_max: Some(1_073_741_824),
        memory_high: Some(2_147_483_648),
        memory_swap_max: Some(0),
        tasks_max: Some(500),
        io_weight: Some(10),
        ..InheritedLimits::default()
    };
}

/// The properties of the third argument.
fn properties(arguments: &[Argument]) -> &[Property] {
    for argument in arguments {
        if let Argument::Properties(properties) = argument {
            return properties;
        }
    }
    panic!("no property list in {arguments:?}");
}

/// The property of that name.
fn property<'a>(properties: &'a [Property], name: &str) -> &'a Property {
    for property in properties {
        if property.name == name {
            return property;
        }
    }
    panic!("no {name} in {properties:?}");
}

/// Creating a scope names the slice,
/// carries the pids as a property,
/// is delegated and collected when it stops,
/// gives the description it was handed,
/// and sends the empty array of auxiliary units the method requires.
#[test]
fn a_created_scope_carries_its_pids_its_limits_and_its_description() {
    let call = create_call(SCOPE, SLICE, &[10, 11], &all_limits(), DESCRIPTION);
    assert_eq!(call.member, "StartTransientUnit");
    assert_eq!(body_signature(&call.arguments), "ssa(sv)a(sa(sv))");
    assert_eq!(call.arguments.first(), Some(&Argument::Text(SCOPE.to_string())));
    assert_eq!(call.arguments.get(1), Some(&Argument::Text("fail".to_string())));
    assert_eq!(call.arguments.last(), Some(&Argument::NoAuxUnits));
    let properties = properties(&call.arguments);
    assert_eq!(property(properties, "Slice").value, PropertyValue::Text(SLICE.to_string()));
    assert_eq!(property(properties, "PIDs").value, PropertyValue::Pids(vec![10, 11]));
    assert_eq!(property(properties, "Description").value, PropertyValue::Text(DESCRIPTION.to_string()));
    assert_eq!(property(properties, "Delegate").value, PropertyValue::Flag(true));
    assert_eq!(property(properties, "CollectMode").value, PropertyValue::Text("inactive-or-failed".to_string()));
    assert_eq!(property(properties, "CPUQuotaPerSecUSec").value, PropertyValue::Number(8_000_000));
    assert_eq!(property(properties, "MemoryMax").value, PropertyValue::Number(1_073_741_824));
    assert_eq!(property(properties, "MemoryHigh").value, PropertyValue::Number(2_147_483_648));
    assert_eq!(property(properties, "MemorySwapMax").value, PropertyValue::Number(0));
    assert_eq!(property(properties, "TasksMax").value, PropertyValue::Number(500));
    assert_eq!(property(properties, "IOWeight").value, PropertyValue::Number(10));
}

/// A limit the origin does not set is left off a new scope,
/// so the manager's default applies instead of a value read from nowhere.
#[test]
fn a_limit_the_origin_does_not_set_is_left_off_a_new_scope() {
    let call = create_call(SCOPE, SLICE, &[10], &InheritedLimits::default(), DESCRIPTION);
    let properties = properties(&call.arguments);
    for name in ["CPUQuotaPerSecUSec", "MemoryMax", "MemoryHigh", "MemorySwapMax", "TasksMax", "IOWeight"] {
        assert!(!properties.iter().any(|property| return property.name == name), "{name} was sent though the origin sets nothing");
    }
}

/// The per-device limits of `io.max` are sent as arrays of device path and number,
/// one property per kind that has any device.
#[test]
fn per_device_limits_are_sent_as_arrays_of_device_paths_and_numbers() {
    let mut limits = InheritedLimits::default();
    limits.io_devices.insert("259:0".to_string(), DeviceLimits { read_bandwidth: Some(1_048_576), write_iops: Some(500), ..DeviceLimits::default() });
    limits.io_devices.insert("8:0".to_string(), DeviceLimits { read_bandwidth: Some(2_097_152), ..DeviceLimits::default() });
    let call = create_call(SCOPE, SLICE, &[10], &limits, DESCRIPTION);
    let properties = properties(&call.arguments);
    let read = property(properties, "IOReadBandwidthMax");
    assert_eq!(read.value, PropertyValue::Devices(vec![("/dev/block/259:0".to_string(), 1_048_576), ("/dev/block/8:0".to_string(), 2_097_152)]));
    assert_eq!(property(properties, "IOWriteIOPSMax").value, PropertyValue::Devices(vec![("/dev/block/259:0".to_string(), 500)]));
    assert!(!properties.iter().any(|property| return property.name == "IOWriteBandwidthMax" || property.name == "IOReadIOPSMax"), "a kind no device has is left off a new scope");
}

/// Attaching names the unit,
/// passes an empty subgroup,
/// and lists the pids.
#[test]
fn attaching_names_the_unit_and_the_pids() {
    let call = attach_call(SCOPE, &[10, 11]);
    assert_eq!(call.member, "AttachProcessesToUnit");
    assert_eq!(body_signature(&call.arguments), "ssau");
    assert_eq!(call.arguments, vec![Argument::Text(SCOPE.to_string()), Argument::Text(String::new()), Argument::Numbers(vec![10, 11])]);
}

/// A limit change sends every limit,
/// the unlimited and default ones as explicit values,
/// because a property left out would keep its old value on the live scope,
/// and every kind of per-device limit,
/// an empty one clearing that kind.
#[test]
fn a_limit_change_sends_every_limit_with_unlimited_for_the_absent_ones() {
    let call = set_properties_call(SCOPE, &InheritedLimits { cpu_quota_per_sec_usec: Some(250_000), ..InheritedLimits::default() });
    assert_eq!(call.member, "SetUnitProperties");
    assert_eq!(body_signature(&call.arguments), "sba(sv)");
    assert_eq!(call.arguments.first(), Some(&Argument::Text(SCOPE.to_string())));
    assert_eq!(call.arguments.get(1), Some(&Argument::Flag(true)));
    let properties = properties(&call.arguments);
    assert_eq!(property(properties, "CPUQuotaPerSecUSec").value, PropertyValue::Number(250_000));
    for name in ["MemoryMax", "MemoryHigh", "MemorySwapMax", "TasksMax"] {
        assert_eq!(property(properties, name).value, PropertyValue::Number(u64::MAX), "{name}");
    }
    assert_eq!(property(properties, "IOWeight").value, PropertyValue::Number(100));
    for name in ["IOReadBandwidthMax", "IOWriteBandwidthMax", "IOReadIOPSMax", "IOWriteIOPSMax"] {
        assert_eq!(property(properties, name).value, PropertyValue::Devices(Vec::new()), "{name}");
    }
}

/// Stopping a scope names it and the job mode.
#[test]
fn stopping_names_the_scope_and_the_mode() {
    let call = stop_call(SCOPE);
    assert_eq!(call.member, "StopUnit");
    assert_eq!(body_signature(&call.arguments), "ss");
    assert_eq!(call.arguments, vec![Argument::Text(SCOPE.to_string()), Argument::Text("replace".to_string())]);
}

/// The slice is requested with the description a person reads in `systemctl status`,
/// and the same empty auxiliary array the method requires.
#[test]
fn the_slice_is_requested_with_its_description() {
    let call = ensure_slice_call(SLICE);
    assert_eq!(call.member, "StartTransientUnit");
    assert_eq!(body_signature(&call.arguments), "ssa(sv)a(sa(sv))");
    assert_eq!(call.arguments.first(), Some(&Argument::Text(SLICE.to_string())));
    let properties = properties(&call.arguments);
    match &property(properties, "Description").value {
        PropertyValue::Text(text) => assert!(text.contains("wg-quicker") && text.contains("tunnel"), "{text}"),
        other => panic!("the description is not text: {other:?}"),
    }
}

/// The description of a scope names the origin cgroup and the method that moved the process,
/// so no one reads a test run's scope as an update run of its own.
#[test]
fn the_description_names_the_origin_and_the_method() {
    let description = scope_description(ORIGIN, MatchReason::ToolchainPath);
    assert!(description.contains(ORIGIN), "{description}");
    assert!(description.contains(MatchReason::ToolchainPath.method_label()), "{description}");
    let pi = scope_description(ORIGIN, MatchReason::PiAgent);
    assert!(pi.contains(MatchReason::PiAgent.method_label()), "{pi}");
    assert_ne!(description, pi);
    for reason in [MatchReason::MiseTask, MatchReason::MpmShadowVerb, MatchReason::MpmShim, MatchReason::ToolchainScript] {
        assert!(!reason.method_label().is_empty());
    }
}
