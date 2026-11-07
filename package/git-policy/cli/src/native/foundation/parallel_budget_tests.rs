//! The worker budget:
//! what bounds it,
//! what never lets it reach zero,
//! and how the host's facts are read.

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

/// The margin is an eighth of the threads,
/// at least one,
/// and nothing for a process that may run only two.
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

/// Fewer items than workers:
/// one worker per item.
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

/// A host with almost no memory still runs one item,
/// so the run makes progress.
#[test]
fn a_starved_host_still_gets_one_worker() {
    let starved: HostResources = host(CPU_MARGIN_DIVISOR * 4, Some(0));
    assert_eq!(worker_budget(&request(usize::MAX, &starved)), 1);
    let nothing: HostResources = host(0, None);
    assert_eq!(worker_budget(&request(usize::MAX, &nothing)), 1);
    assert_eq!(worker_budget(&request(0, &nothing)), 1);
}

/// The measured ceiling and the person's cap bound the workers,
/// and the smaller of the two wins.
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

/// The real host reports at least one thread.
#[test]
fn this_host_has_a_thread() {
    assert!(HostResources::read().parallelism >= 1);
}

/// File-tree jobs take the host's budget up to their ceiling:
/// the margin and the memory bound them on a small host,
/// the ceiling on a
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

/// The owner's ruling of 2026-11-07 ("Memory headroom:
/// sysinfo") moves the reading of the system's available memory and of the
///  control groups' limits to the published crate,
/// so this file holds neither the `/proc/meminfo` label nor the group files.
#[test]
fn the_budget_reads_no_memory_file_itself() {
    let source: &str = include_str!("parallel_budget.rs");
    let code: String = source.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
    // Quoted, so the method call `per_item_memory.max(1)` is not mistaken for the group file `memory.max`.
    for needle in ["\"MemAvailable", "\"memory.max\"", "\"memory.current\"", "/proc/meminfo"] {
        assert!(!code.contains(needle), "parallel_budget.rs still holds `{needle}`: the memory figures come from the sysinfo crate");
    }
}

/// The available memory the budget reads is a figure no larger than the system's total,
/// and the read gives one on this host.
#[test]
fn the_read_memory_is_a_figure_below_the_system_total() {
    let host: HostResources = HostResources::read();
    let total: u64 = std::fs::read_to_string("/proc/meminfo")
        .expect("meminfo")
        .lines()
        .find_map(|line| { return line.strip_prefix("MemTotal:")?.split_whitespace().next()?.parse::<u64>().ok(); })
        .expect("MemTotal")
        * 1024;
    let available: u64 = host.memory_available.expect("this host reports its available memory");
    assert!(available > 0 && available <= total, "{available} of {total}");
}
