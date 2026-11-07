//! The runtime plan: how many workers the budget and the limits allow, and when the plan is the current-thread runtime.

use super::*;

/// A host with this many threads, memory not limiting.
fn host(parallelism: usize) -> HostResources {
    return HostResources { parallelism, memory_available: None };
}

/// Limits that bound nothing.
const NO_LIMITS: RuntimeLimits = RuntimeLimits { address_space_left: None, threads_left: None };

/// With no limits the workers are the budget's: the CPUs less the margin.
#[test]
fn without_limits_the_workers_are_the_budgets() {
    let large: HostResources = host(32);
    let expected: usize = large.parallelism - cpu_margin(large.parallelism);
    assert_eq!(plan_runtime(&large, &NO_LIMITS), RuntimePlan::MultiThread { workers: expected, blocking_threads: expected });
}

/// A budget of one worker is the current-thread runtime, because a pool of one worker only adds a thread.
#[test]
fn a_budget_of_one_worker_is_the_current_thread_runtime() {
    assert_eq!(plan_runtime(&host(1), &NO_LIMITS), RuntimePlan::CurrentThread);
}

/// The process limit bounds the workers: every worker and every blocking thread and the building thread must fit.
#[test]
fn the_threads_left_bound_the_workers() {
    let large: HostResources = host(32);
    let limits: RuntimeLimits = RuntimeLimits { address_space_left: None, threads_left: Some(9) };
    assert_eq!(plan_runtime(&large, &limits), RuntimePlan::MultiThread { workers: 4, blocking_threads: 4 });
    let none: RuntimeLimits = RuntimeLimits { address_space_left: None, threads_left: Some(0) };
    assert_eq!(plan_runtime(&large, &none), RuntimePlan::CurrentThread);
}

/// The address space bounds the workers: each thread maps a stack, and the rest of the command keeps its reserve.
#[test]
fn the_address_space_left_bounds_the_workers() {
    let large: HostResources = host(32);
    let room_for_three_pairs: u64 = ADDRESS_SPACE_RESERVE_BYTES + 3 * 2 * THREAD_VIRTUAL_BYTES;
    let limits: RuntimeLimits = RuntimeLimits { address_space_left: Some(room_for_three_pairs), threads_left: None };
    assert_eq!(plan_runtime(&large, &limits), RuntimePlan::MultiThread { workers: 3, blocking_threads: 3 });
    let below_reserve: RuntimeLimits = RuntimeLimits { address_space_left: Some(ADDRESS_SPACE_RESERVE_BYTES - 1), threads_left: None };
    assert_eq!(plan_runtime(&large, &below_reserve), RuntimePlan::CurrentThread);
}

/// The smaller of the two limits wins.
#[test]
fn the_smaller_limit_wins() {
    let large: HostResources = host(32);
    let limits: RuntimeLimits = RuntimeLimits {
        address_space_left: Some(ADDRESS_SPACE_RESERVE_BYTES + 10 * 2 * THREAD_VIRTUAL_BYTES),
        threads_left: Some(2 * 5 + 1),
    };
    assert_eq!(plan_runtime(&large, &limits), RuntimePlan::MultiThread { workers: 5, blocking_threads: 5 });
}
