//! Focused tests for the properties of the isolated app scope.

use super::*;

/// What: `fn isolation(...)` builds an `Isolation` value for a test.
/// Why:  Each test names only the cap it cares about.
fn isolation(cpu_quota_percent: Option<u32>, cpu_weight: Option<u32>) -> Isolation {
    return Isolation { enabled: true, cpu_quota_percent, cpu_weight };
}

/// The CPU cap and share the caller names reach the scope as properties.
#[test]
fn given_cpu_cap_and_weight_become_scope_properties() {
    let properties = isolation_properties(&isolation(Some(200), Some(20)));
    assert!(properties.contains(&"--property=CPUQuota=200%".to_string()));
    assert!(properties.contains(&"--property=CPUWeight=20".to_string()));
}
