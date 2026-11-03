//! The worker budget: what bounds it, what never lets it reach zero, and how the host's facts are read.

use super::*;

/// The bytes one item holds in these controls.
const ITEM_BYTES: u64 = 1 << 28;

/// A host with this many threads and this much available memory.
fn host(parallelism: usize, memory_available: Option<u64>) -> HostResources {
    return HostResources { parallelism, memory_available };
}

/// A request for `items` items on `host` with no ceiling and no cap.
fn request(items: usize, host: &HostResources) -> BudgetRequest<'_> {
    return BudgetRequest { items, per_item_memory: ITEM_BYTES, ceiling: None, cap: None, host };
}

/// The margin is an eighth of the threads, at least one, and nothing for a process that may run only two.
#[test]
fn the_margin_is_an_eighth_of_the_threads_and_nothing_for_two() {
    assert_eq!(cpu_margin(1), 0);
    assert_eq!(cpu_margin(2), 0);
    assert_eq!(cpu_margin(3), 1);
    assert_eq!(cpu_margin(CPU_MARGIN_DIVISOR), 1);
    assert_eq!(cpu_margin(CPU_MARGIN_DIVISOR * 4), 4);
}

/// With plenty of items and memory the CPUs minus the margin bound the workers.
#[test]
fn the_cpus_minus_the_margin_bound_the_workers() {
    let large: HostResources = host(CPU_MARGIN_DIVISOR * 4, None);
    assert_eq!(worker_budget(&request(usize::MAX, &large)), large.parallelism - cpu_margin(large.parallelism));
    let small: HostResources = host(2, None);
    assert_eq!(worker_budget(&request(usize::MAX, &small)), 2);
}

/// Fewer items than workers: one worker per item.
#[test]
fn the_items_bound_the_workers() {
    let large: HostResources = host(CPU_MARGIN_DIVISOR * 4, None);
    assert_eq!(worker_budget(&request(3, &large)), 3);
}

/// Memory that is free bounds the workers to the items a share of it holds.
#[test]
fn the_free_memory_bounds_the_workers() {
    let fits: u64 = 5;
    let tight: HostResources = host(CPU_MARGIN_DIVISOR * 4, Some(ITEM_BYTES * fits * MEMORY_SHARE_DIVISOR));
    assert_eq!(worker_budget(&request(usize::MAX, &tight)), usize::try_from(fits).expect("a small count"));
}

/// A host with almost no memory still runs one item, so the run makes progress.
#[test]
fn a_starved_host_still_gets_one_worker() {
    let starved: HostResources = host(CPU_MARGIN_DIVISOR * 4, Some(0));
    assert_eq!(worker_budget(&request(usize::MAX, &starved)), 1);
    let nothing: HostResources = host(0, None);
    assert_eq!(worker_budget(&request(usize::MAX, &nothing)), 1);
    assert_eq!(worker_budget(&request(0, &nothing)), 1);
}

/// The measured ceiling and the person's cap bound the workers, and the smaller of the two wins.
#[test]
fn the_ceiling_and_the_cap_bound_the_workers() {
    let large: HostResources = host(CPU_MARGIN_DIVISOR * 4, None);
    let ceiling: usize = 4;
    let cap: usize = 2;
    assert_eq!(worker_budget(&BudgetRequest { ceiling: Some(ceiling), ..request(usize::MAX, &large) }), ceiling);
    assert_eq!(worker_budget(&BudgetRequest { cap: Some(cap), ..request(usize::MAX, &large) }), cap);
    assert_eq!(worker_budget(&BudgetRequest { ceiling: Some(ceiling), cap: Some(cap), ..request(usize::MAX, &large) }), cap);
    assert_eq!(worker_budget(&BudgetRequest { ceiling: Some(cap), cap: Some(ceiling), ..request(usize::MAX, &large) }), cap);
}

/// The available memory comes from the `MemAvailable` line of the system's text, in bytes.
#[test]
fn the_available_memory_is_read_from_the_system_text() {
    let text: &str = "MemTotal:       65536000 kB\nMemFree:         1000 kB\nMemAvailable:   2048 kB\nBuffers: 1 kB\n";
    assert_eq!(memory_available_from(text), Some(2048 * 1024));
    assert_eq!(memory_available_from("MemTotal: 1 kB\n"), None);
    assert_eq!(memory_available_from("MemAvailable: lots kB\n"), None);
}

/// The control group is the version 2 line, and the headroom is the limit minus the use, nothing when there is no limit.
#[test]
fn the_control_group_headroom_is_read_from_the_group_files() {
    assert_eq!(control_group_path("12:cpu:/old\n0::/user.slice/session.scope\n"), Some(String::from("/user.slice/session.scope")));
    assert_eq!(control_group_path("12:cpu:/old\n"), None);
    assert_eq!(headroom_from("1000\n", "400\n"), Some(600));
    assert_eq!(headroom_from("1000\n", "4000\n"), Some(0));
    assert_eq!(headroom_from("max\n", "400\n"), None);
}

/// The least headroom of a group and the groups above it wins, and a hierarchy without limits has none.
#[test]
fn the_least_headroom_up_the_hierarchy_wins() {
    let root: PathBuf = crate::test_support::fixture("parallel-budget-groups");
    let outer: PathBuf = root.join("outer");
    let inner: PathBuf = outer.join("inner");
    std::fs::create_dir_all(&inner).expect("directories");
    std::fs::write(inner.join("memory.max"), "max\n").expect("inner limit");
    std::fs::write(inner.join("memory.current"), "10\n").expect("inner use");
    assert_eq!(group_headroom(root.as_path(), "/outer/inner"), None);
    std::fs::write(outer.join("memory.max"), "100\n").expect("outer limit");
    std::fs::write(outer.join("memory.current"), "30\n").expect("outer use");
    assert_eq!(group_headroom(root.as_path(), "/outer/inner"), Some(70));
    std::fs::write(inner.join("memory.max"), "50\n").expect("inner limit");
    assert_eq!(group_headroom(root.as_path(), "/outer/inner"), Some(40));
    crate::test_support::remove(root.as_path());
}

/// The real host reports at least one thread.
#[test]
fn this_host_has_a_thread() {
    assert!(HostResources::read().parallelism >= 1);
}

/// File-tree jobs take the host's budget up to their ceiling: the margin and the memory bound them on a small host, the ceiling on a
///  large one.
#[test]
fn sliced_jobs_take_the_budget_up_to_their_ceiling() {
    let ceiling: usize = 8;
    let large: HostResources = host(CPU_MARGIN_DIVISOR * 8, None);
    assert_eq!(sliced_workers(usize::MAX, ceiling, &large), ceiling);
    let small: HostResources = host(ceiling, None);
    assert_eq!(sliced_workers(usize::MAX, ceiling, &small), ceiling - cpu_margin(ceiling));
    let tight: HostResources = host(large.parallelism, Some(SLICE_MEMORY * MEMORY_SHARE_DIVISOR * 3));
    assert_eq!(sliced_workers(usize::MAX, ceiling, &tight), 3);
}
