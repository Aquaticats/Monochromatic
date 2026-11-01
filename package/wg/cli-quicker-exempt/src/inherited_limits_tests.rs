//! Verifies that the limits of the cgroups a process leaves are read from a fake cgroup tree,
//! merged to the smallest finite value,
//! and rendered as unit properties.

/// Reading and rendering under test.
use crate::inherited_limits::{parse_cpu_max, parse_io_max, parse_io_weight, parse_limit, read_inherited_limits, DeviceLimits, InheritedLimits};
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

/// The `default` line of `io.weight` is the weight,
/// device lines are not read,
/// the kernel default is nothing to carry,
/// and a file without a default line or with a garbled one is an error.
#[test]
fn the_io_weight_default_line_is_read() {
    assert_eq!(parse_io_weight("default 10\n").expect("ten"), Some(10));
    assert_eq!(parse_io_weight("default 10\n8:0 50\n").expect("device lines are skipped"), Some(10));
    assert_eq!(parse_io_weight("default 100\n").expect("the default"), None);
    assert_eq!(parse_io_weight("default 500\n").expect("a boost"), Some(500));
    assert!(parse_io_weight("8:0 50\n").is_err());
    assert!(parse_io_weight("default heavy\n").is_err());
    assert!(parse_io_weight("").is_err());
}

/// The origin of the cap daemon has `io.weight` 10,
/// and the scope carries it.
#[test]
fn the_io_weight_of_the_origin_is_carried() {
    let root = tree("io-carried");
    let origin = format!("{APP_SLICE}/claude-code-bash");
    set(&root, &origin, &[("cpu.max", "2800000 100000"), ("io.weight", "default 10")]);
    let limits = read_inherited_limits(&root, &origin, DESTINATION).expect("readable");
    assert_eq!(limits.io_weight, Some(10));
    assert_eq!(limits.properties(), vec![("CPUQuotaPerSecUSec", 28_000_000), ("IOWeight", 10)]);
}

/// The lightest weight on the way up wins,
/// a weight above the default is never carried,
/// and cgroups at the default carry nothing.
#[test]
fn the_lightest_io_weight_wins_and_boosts_are_dropped() {
    let outer = format!("{APP_SLICE}/app-build.slice");
    let origin = format!("{outer}/app-build-1.scope");
    let weights = |outer_weight: &str, origin_weight: &str| {
        let root = tree(&format!("io-{}-{}", outer_weight.replace(' ', "_"), origin_weight.replace(' ', "_")));
        set(&root, &outer, &[("io.weight", outer_weight)]);
        set(&root, &origin, &[("io.weight", origin_weight)]);
        return read_inherited_limits(&root, &origin, DESTINATION).expect("readable").io_weight;
    };
    assert_eq!(weights("default 50", "default 20"), Some(20));
    assert_eq!(weights("default 20", "default 100"), Some(20));
    assert_eq!(weights("default 500", "default 100"), None);
    assert_eq!(weights("default 100", "default 100"), None);
    assert_eq!(weights("default 500", "default 30"), Some(30));
}

/// A live scope is sent every limit:
/// the absent ones as unlimited and the weight as the default.
#[test]
fn all_properties_name_every_limit() {
    let none = InheritedLimits::default().all_properties();
    assert_eq!(none.len(), 6);
    assert!(none.iter().take(5).all(|(_name, value)| return *value == u64::MAX));
    assert_eq!(none[5], ("IOWeight", 100));
    let some = InheritedLimits { memory_max: Some(7), io_weight: Some(10), ..InheritedLimits::default() }.all_properties();
    assert_eq!(some[1], ("MemoryMax", 7));
    assert_eq!(some[5], ("IOWeight", 10));
}

/// `io.max` lines name a device and its four limits,
/// `max` is no limit,
/// a device with nothing finite is not listed,
/// a key this parser does not know is skipped,
/// and a line that cannot be understood is an error.
#[test]
fn io_max_lines_parse_per_device() {
    let parsed = parse_io_max("259:0 rbps=1048576 wbps=max riops=max wiops=500\n8:0 rbps=max wbps=max riops=max wiops=max\n8:16 rbps=10 future=7\n").expect("parses");
    assert_eq!(parsed.len(), 2, "{parsed:?}");
    assert_eq!(parsed["259:0"], DeviceLimits { read_bandwidth: Some(1_048_576), write_iops: Some(500), ..DeviceLimits::default() });
    assert_eq!(parsed["8:16"], DeviceLimits { read_bandwidth: Some(10), ..DeviceLimits::default() });
    assert!(parse_io_max("").expect("empty").is_empty());
    for garbled in ["sda rbps=1\n", "8:0 rbps\n", "8:0 rbps=fast\n", ":0 rbps=1\n"] {
        assert!(parse_io_max(garbled).is_err(), "{garbled:?}");
    }
}

/// The per-device limits of the origin reach the scope,
/// keyed by device,
/// the smallest value of each kind on the way up wins,
/// and a missing file is no limit.
#[test]
fn the_per_device_limits_of_the_origin_are_carried() {
    let root = tree("io-max");
    let outer = format!("{APP_SLICE}/app-build.slice");
    let origin = format!("{outer}/app-build-1.scope");
    set(&root, &outer, &[("io.max", "259:0 rbps=2000000 wbps=max riops=max wiops=max")]);
    set(&root, &origin, &[("io.max", "259:0 rbps=3000000 wbps=500000 riops=max wiops=max\n8:0 rbps=max wbps=max riops=100 wiops=max")]);
    let limits = read_inherited_limits(&root, &origin, DESTINATION).expect("readable");
    assert_eq!(limits.io_devices["259:0"], DeviceLimits { read_bandwidth: Some(2_000_000), write_bandwidth: Some(500_000), ..DeviceLimits::default() });
    assert_eq!(limits.io_devices["8:0"], DeviceLimits { read_iops: Some(100), ..DeviceLimits::default() });
    let properties = limits.device_properties();
    assert_eq!(properties[0], ("IOReadBandwidthMax", vec![("/dev/block/259:0".to_string(), 2_000_000)]));
    assert_eq!(properties[1], ("IOWriteBandwidthMax", vec![("/dev/block/259:0".to_string(), 500_000)]));
    assert_eq!(properties[2], ("IOReadIOPSMax", vec![("/dev/block/8:0".to_string(), 100)]));
    assert_eq!(properties[3], ("IOWriteIOPSMax", Vec::new()));
    let plain = tree("io-max-plain");
    set(&plain, &origin, &[("cpu.max", "max 100000")]);
    assert!(read_inherited_limits(&plain, &origin, DESTINATION).expect("readable").io_devices.is_empty());
    let garbled = tree("io-max-garbled");
    set(&garbled, &origin, &[("io.max", "8:0 rbps=fast")]);
    let error = read_inherited_limits(&garbled, &origin, DESTINATION).expect_err("a garbled file");
    assert!(error.to_string().contains("io.max"), "{error}");
}
