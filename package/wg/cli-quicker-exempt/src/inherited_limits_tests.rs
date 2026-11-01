//! Verifies that the limits of the cgroups a process leaves are read from a fake cgroup tree,
//! merged to the smallest finite value,
//! and rendered as unit properties.

/// Reading and rendering under test.
use crate::inherited_limits::{parse_cpu_max, parse_limit, read_inherited_limits, InheritedLimits};
/// Fixture paths.
use std::path::{Path, PathBuf};

/// The common ancestor of origin and destination in these fixtures.
const APP_SLICE: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice";
/// The destination scope in these fixtures.
const DESTINATION: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/app-monochromatic\\x2dupdates.slice/wg-quicker-update-1.scope";

/// Makes an empty fake cgroup tree.
fn tree(name: &str) -> PathBuf {
    let root = crate::test_scratch::scratch_root().join(format!("wg-quicker-limits-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("make the tree");
    return root;
}

/// Writes limit files into one cgroup directory of the tree.
fn set(root: &Path, cgroup: &str, files: &[(&str, &str)]) {
    let directory = root.join(cgroup.trim_start_matches('/'));
    std::fs::create_dir_all(&directory).expect("make the cgroup");
    for (name, content) in files {
        std::fs::write(directory.join(name), format!("{content}\n")).expect("write the limit");
    }
}

/// Limit files read as numbers or unlimited,
/// and `cpu.max` as microseconds per second.
#[test]
fn limit_files_parse() {
    assert_eq!(parse_limit("max\n").expect("max"), None);
    assert_eq!(parse_limit("1073741824\n").expect("number"), Some(1_073_741_824));
    assert!(parse_limit("lots").is_err());
    assert_eq!(parse_cpu_max("max 100000\n").expect("unlimited"), None);
    assert_eq!(parse_cpu_max("800000 100000\n").expect("eight cpus"), Some(8_000_000));
    assert_eq!(parse_cpu_max("50000 100000\n").expect("half a cpu"), Some(500_000));
    assert!(parse_cpu_max("100000").is_err());
    assert!(parse_cpu_max("100000 0").is_err());
    assert!(parse_cpu_max("100000 many").is_err());
}

/// A CPU cap and a memory cap on the cgroup the process leaves reach the new scope.
#[test]
fn the_caps_of_the_origin_cgroup_are_carried() {
    let root = tree("carried");
    let origin = format!("{APP_SLICE}/claude-code-bash");
    set(&root, &origin, &[("cpu.max", "800000 100000"), ("memory.max", "17179869184"), ("memory.high", "max"), ("pids.max", "max")]);
    let limits = read_inherited_limits(&root, &origin, DESTINATION).expect("readable");
    assert_eq!(
        limits,
        InheritedLimits { cpu_quota_per_sec_usec: Some(8_000_000), memory_max: Some(17_179_869_184), ..InheritedLimits::default() }
    );
    assert_eq!(limits.properties(), vec![("CPUQuotaPerSecUSec", 8_000_000), ("MemoryMax", 17_179_869_184)]);
    assert_ne!(limits, InheritedLimits::default());
}

/// The smallest finite value on the way up wins,
/// and the common ancestor and above are left to keep applying by themselves.
#[test]
fn the_smallest_cap_below_the_common_ancestor_wins() {
    let root = tree("smallest");
    let outer = format!("{APP_SLICE}/app-build.slice");
    let origin = format!("{outer}/app-build-1.scope");
    set(&root, &outer, &[("cpu.max", "200000 100000"), ("pids.max", "512")]);
    set(&root, &origin, &[("cpu.max", "400000 100000"), ("pids.max", "max"), ("memory.swap.max", "0")]);
    set(&root, APP_SLICE, &[("cpu.max", "100000 100000"), ("memory.max", "1024")]);
    let limits = read_inherited_limits(&root, &origin, DESTINATION).expect("readable");
    assert_eq!(limits.cpu_quota_per_sec_usec, Some(2_000_000));
    assert_eq!(limits.tasks_max, Some(512));
    assert_eq!(limits.memory_swap_max, Some(0));
    assert_eq!(limits.memory_max, None, "the shared ancestor's own limit is not copied");
    assert_eq!(limits.properties()[0], ("CPUQuotaPerSecUSec", 2_000_000));
}

/// An uncapped origin and a controller that is absent leave nothing to carry.
#[test]
fn an_uncapped_origin_carries_nothing() {
    let root = tree("uncapped");
    let origin = format!("{APP_SLICE}/plain.scope");
    set(&root, &origin, &[("cpu.max", "max 100000"), ("memory.max", "max")]);
    let limits = read_inherited_limits(&root, &origin, DESTINATION).expect("readable");
    assert_eq!(limits, InheritedLimits::default());
    assert!(limits.properties().is_empty());
    let absent = read_inherited_limits(&root, "/user.slice/user-1000.slice/user@1000.service/app.slice/never-made.scope", DESTINATION).expect("a missing directory has no limits");
    assert_eq!(absent, InheritedLimits::default());
}

/// A limit file that cannot be understood is an error,
/// because assuming no limit could loosen the process.
#[test]
fn a_garbled_limit_file_is_an_error_naming_the_file() {
    let root = tree("garbled");
    let origin = format!("{APP_SLICE}/odd.scope");
    set(&root, &origin, &[("memory.max", "plenty")]);
    let error = read_inherited_limits(&root, &origin, DESTINATION).expect_err("a garbled file");
    assert!(error.to_string().contains("memory.max"), "{error}");
}
