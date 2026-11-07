//! Focused tests for the properties of the isolated app scope.

use super::*;

/// What: `fn isolation(...)` builds an `Isolation` value for a test.
/// Why:  Each test names only the cap it cares about.
fn isolation(cpu_quota_percent: Option<u32>, cpu_weight: Option<u32>) -> Isolation {
    return Isolation { enabled: true, cpu_quota_percent, cpu_weight, memory_max: None, tasks_max: None };
}

/// The CPU cap and share the caller names reach the scope as properties.
#[test]
fn given_cpu_cap_and_weight_become_scope_properties() {
    let properties = isolation_properties(&isolation(Some(200), Some(20)));
    assert!(properties.contains(&"--property=CPUQuota=200%".to_string()));
    assert!(properties.contains(&"--property=CPUWeight=20".to_string()));
}

/// Highest `IOWeight` a per-command cgroup may carry: the owner's rule in `CLAUDE.md` (RXS) says never above 10.
/// The test states it itself so a change to the implementation's own constant cannot move it.
const OWNER_MAXIMUM_IO_WEIGHT: u32 = 10;

/// What: `fn values_of(...)` lists the values one `--property=NAME=` prefix has in the arguments.
/// Why:  Each test asks how many times and with what value a limit is set.
fn values_of(properties: &[String], name: &str) -> Vec<String> {
    let prefix = format!("--property={name}=");
    return properties
        .iter()
        .filter_map(|property| return property.strip_prefix(prefix.as_str()))
        .map(|value| return value.to_string())
        .collect();
}

/// A scope without a named limit still carries a memory, a CPU, a task and a disk-weight limit, one of each.
#[test]
fn default_scope_carries_memory_cpu_task_and_disk_weight_limits_once_each() {
    let properties = isolation_properties(&isolation(None, None));
    for name in ["MemoryMax", "CPUQuota", "TasksMax", "IOWeight"] {
        assert_eq!(values_of(&properties, name).len(), 1, "{name} in {properties:?}");
    }
}

/// The disk weight of the scope never exceeds the owner's limit.
#[test]
fn scope_disk_weight_never_exceeds_the_owner_limit() {
    let properties = isolation_properties(&isolation(None, None));
    let weights = values_of(&properties, "IOWeight");
    let weight: u32 = weights.first().and_then(|text| return text.parse().ok()).unwrap_or(u32::MAX);
    assert!(weight > 0 && weight <= OWNER_MAXIMUM_IO_WEIGHT, "IOWeight {weight} in {properties:?}");
}

/// The memory cap and the task limit the caller names replace the defaults.
#[test]
fn named_memory_and_task_limits_replace_the_defaults() {
    let named = Isolation { enabled: true, cpu_quota_percent: None, cpu_weight: None, memory_max: Some("3G".to_string()), tasks_max: Some(77) };
    let properties = isolation_properties(&named);
    assert_eq!(values_of(&properties, "MemoryMax"), vec!["3G".to_string()]);
    assert_eq!(values_of(&properties, "TasksMax"), vec!["77".to_string()]);
}

/// The fixed disk weight stays within the owner's limit.
#[test]
fn fixed_disk_weight_constant_stays_within_the_owner_limit() {
    assert!(APP_IO_WEIGHT > 0 && APP_IO_WEIGHT <= OWNER_MAXIMUM_IO_WEIGHT);
}
