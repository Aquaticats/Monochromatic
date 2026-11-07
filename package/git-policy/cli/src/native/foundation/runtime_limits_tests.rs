//! The limits a runtime build reads: what each source's text says, and that an unset limit never bounds the build.

use super::*;

/// The mappings come from the `VmSize` line, in bytes, and a text without it gives no figure.
#[test]
fn the_mapped_bytes_come_from_the_vm_size_line() {
    let status: &str = "Name:\tcli\nVmPeak:\t  9000 kB\nVmSize:\t  2048 kB\nVmRSS:\t 100 kB\n";
    assert_eq!(mapped_bytes_from(status), Some(2048 * 1024));
    assert_eq!(mapped_bytes_from("Name:\tcli\n"), None);
    assert_eq!(mapped_bytes_from("VmSize:\tlots kB\n"), None);
}

/// The system thread total is the part after the slash of the fourth `/proc/loadavg` field.
#[test]
fn the_system_threads_come_from_the_fourth_load_field() {
    assert_eq!(system_threads_from("0.10 0.20 0.30 3/4321 99999\n"), Some(4321));
    assert_eq!(system_threads_from("0.10 0.20 0.30\n"), None);
    assert_eq!(system_threads_from("0.10 0.20 0.30 three 9\n"), None);
}

/// A limit that is set leaves what the use has not taken, never below zero; a limit that is not set leaves nothing to bound by.
#[test]
fn a_set_limit_leaves_the_rest_and_an_unset_limit_bounds_nothing() {
    assert_eq!(left_under(Some(100), 30), Some(70));
    assert_eq!(left_under(Some(100), 300), Some(0));
    assert_eq!(left_under(None, 300), None);
}

/// A control group's thread room is `pids.max` minus `pids.current`, and `max` is no limit.
#[test]
fn a_groups_thread_room_is_its_maximum_less_its_use() {
    assert_eq!(group_threads_left_from("512\n", "12\n"), Some(500));
    assert_eq!(group_threads_left_from("512\n", "900\n"), Some(0));
    assert_eq!(group_threads_left_from("max\n", "12\n"), None);
}

/// The least of two optional figures treats nothing as unlimited.
#[test]
fn the_least_of_two_figures_ignores_an_unlimited_one() {
    assert_eq!(least_of(Some(3), Some(9)), Some(3));
    assert_eq!(least_of(None, Some(9)), Some(9));
    assert_eq!(least_of(Some(3), None), Some(3));
    assert_eq!(least_of(None, None), None);
}

/// The smallest room up the hierarchy wins, and a hierarchy with no `pids.max` has none.
#[test]
fn the_least_thread_room_up_the_hierarchy_wins() {
    let root: PathBuf = crate::test_support::fixture("runtime-limits-groups");
    let outer: PathBuf = root.join("outer");
    let inner: PathBuf = outer.join("inner");
    std::fs::create_dir_all(&inner).expect("directories");
    std::fs::write(inner.join("pids.max"), "max\n").expect("inner limit");
    std::fs::write(inner.join("pids.current"), "10\n").expect("inner use");
    assert_eq!(group_threads_left(root.as_path(), "/outer/inner"), None);
    std::fs::write(outer.join("pids.max"), "100\n").expect("outer limit");
    std::fs::write(outer.join("pids.current"), "30\n").expect("outer use");
    assert_eq!(group_threads_left(root.as_path(), "/outer/inner"), Some(70));
    std::fs::write(inner.join("pids.max"), "50\n").expect("inner limit");
    assert_eq!(group_threads_left(root.as_path(), "/outer/inner"), Some(40));
    crate::test_support::remove(root.as_path());
}

/// The real process reports an address-space figure no larger than its own limit, and the read does not panic.
#[test]
fn this_process_reads_its_own_limits() {
    let limits: RuntimeLimits = RuntimeLimits::read();
    let own: rustix::process::Rlimit = rustix::process::getrlimit(rustix::process::Resource::As);
    if let (Some(left), Some(limit)) = (limits.address_space_left, own.current) {
        assert!(left <= limit);
    }
}
