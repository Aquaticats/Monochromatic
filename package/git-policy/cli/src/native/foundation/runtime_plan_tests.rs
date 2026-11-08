//! The runtime plan:
//! how many workers the budget and the limits allow,
//! and when the plan is the current-thread runtime.

use super::*;
use crate::parallel_budget::cpu_margin;

/// A host with this many threads,
/// memory not limiting.
fn host(parallelism: usize) -> HostResources {
    return HostResources { parallelism, memory_available: None };
}

/// Limits that bound nothing.
const NO_LIMITS: RuntimeLimits = RuntimeLimits { address_space_left: None, threads_left: None };

/// With no limits the workers are the budget's:
/// the CPUs less the margin.
#[test]
fn without_limits_the_workers_are_the_budgets() {
    let large: HostResources = host(32);
    let expected: usize = large.parallelism - cpu_margin(large.parallelism);
    assert_eq!(plan_runtime(&large, &NO_LIMITS), RuntimePlan::MultiThread { workers: expected, blocking_threads: expected });
}

/// A budget of one worker is a multi-thread runtime of one worker,
/// because the watchers of waits run as tasks beside a thread that
///  is blocked in a read or a wait,
/// and a current-thread runtime has no thread to run them on.
/// Only a limit that leaves no thread
///  makes the plan the current-thread runtime.
#[test]
fn a_budget_of_one_worker_is_a_multi_thread_runtime_of_one_worker() {
    assert_eq!(plan_runtime(&host(1), &NO_LIMITS), RuntimePlan::MultiThread { workers: 1, blocking_threads: 1 });
}

/// The process limit bounds the workers:
/// every worker and every blocking thread and the building thread must fit.
#[test]
fn the_threads_left_bound_the_workers() {
    let large: HostResources = host(32);
    let limits: RuntimeLimits = RuntimeLimits { address_space_left: None, threads_left: Some(9) };
    assert_eq!(plan_runtime(&large, &limits), RuntimePlan::MultiThread { workers: 4, blocking_threads: 4 });
    let none: RuntimeLimits = RuntimeLimits { address_space_left: None, threads_left: Some(0) };
    assert_eq!(plan_runtime(&large, &none), RuntimePlan::CurrentThread);
}

/// The address space bounds the workers:
/// each thread maps a stack,
/// and the rest of the command keeps its reserve.
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

/// A cap lowers the workers and the blocking pool of a larger plan together.
#[test]
fn a_cap_lowers_the_workers_and_the_blocking_pool_of_a_larger_plan() {
    let plan: RuntimePlan = RuntimePlan::MultiThread { workers: 25, blocking_threads: 25 };
    assert_eq!(capped_at(plan, 4), RuntimePlan::MultiThread { workers: 4, blocking_threads: 4 });
}

/// A plan smaller than the cap stands,
/// and so does the current-thread plan:
/// a cap only ever lowers.
#[test]
fn a_cap_never_raises_a_plan_and_leaves_the_current_thread_plan() {
    let small: RuntimePlan = RuntimePlan::MultiThread { workers: 2, blocking_threads: 2 };
    assert_eq!(capped_at(small, 8), small);
    assert_eq!(capped_at(RuntimePlan::CurrentThread, 8), RuntimePlan::CurrentThread);
}
