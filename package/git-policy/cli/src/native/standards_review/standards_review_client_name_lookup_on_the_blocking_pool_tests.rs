//! Controls for the name lookup:
//! a name is looked up on the blocking pool,
//! and where the system refuses every thread the lookup is an error and not a panic.

use super::*;
use std::str::FromStr;
use tower_service::Service;

/// The name of this host's own loopback.
const LOOPBACK_NAME: &str = "localhost";

/// A name is looked up and gives addresses.
#[test]
fn a_name_is_looked_up_on_the_blocking_pool() {
    let found = crate::lazy_runtime::run_on_runtime(async {
        let name: Name = Name::from_str(LOOPBACK_NAME).expect("a name");
        return PoolResolver.call(name).await;
    })
    .expect("the runtime")
    .expect("the lookup");
    assert!(found.count() > 0, "the loopback name has an address");
}

/// Where the system refuses every thread,
/// the lookup is an error with the pool's plain words,
/// and tokio's own panic of `spawn_blocking` is never reached.
#[test]
fn where_no_thread_can_start_the_lookup_is_an_error_and_not_a_panic() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_the_lookup_is_an_error_and_not_a_panic") {
        return;
    }
    let outcome = crate::lazy_runtime::run_on_runtime(async {
        let name: Name = Name::from_str(LOOPBACK_NAME).expect("a name");
        return PoolResolver.call(name).await;
    })
    .expect("a current-thread runtime stands in where no thread can start");
    let error: std::io::Error = outcome.expect_err("no thread, so no lookup");
    assert!(error.to_string().contains("refused a thread"), "{error}");
}
