//! What:
//!  Controls for the words a refused set prints.
//! Why:
//!  Every refusal names the input to correct and says no snapshot was destroyed,
//!  so a person does not read a build refusal as a partial or unknown removal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(overTheBudget().message).toContain('No snapshot was destroyed');
//! ```

use crate::worktree_clone_snapshot_batch::request_refusal::BatchRefusal;
use crate::worktree_clone_snapshot_batch::snapshot_destroy_request::destroy_request;
use monochromatic_zfs_request_to_openzfs_kernel_device::layout::MAXPATHLEN;

/// A budget refusal names the snapshot,
/// both numbers and the way forward.
#[test]
fn an_over_budget_name_is_named_with_both_numbers() {
    let refusal: BatchRefusal = BatchRefusal::OneNameOverTheBudget {
        name: String::from("pool/clone@one"),
        request_bytes: 80,
        budget_bytes: 40,
    };
    let said: String = refusal.to_string();
    assert!(said.contains("`pool/clone@one`"));
    assert!(said.contains("needs 80 bytes"));
    assert!(said.contains("budget is 40 bytes"));
    assert!(said.contains("No snapshot was destroyed"));
}

/// A repeated name says which name the set holds twice.
#[test]
fn a_repeated_name_is_named() {
    let said: String = BatchRefusal::RepeatedName {
        name: String::from("pool/clone@one"),
    }
    .to_string();
    assert!(said.contains("`pool/clone@one` appears more than once"));
    assert!(said.contains("No snapshot was destroyed"));
}

/// A deferring request is refused by name,
/// with what a deferral would have done.
#[test]
fn a_deferring_request_says_what_it_would_have_marked() {
    let said: String = BatchRefusal::WouldDefer {
        pool: String::from("pool"),
    }
    .to_string();
    assert!(said.contains("`defer` key"));
    assert!(said.contains("deferred destruction"));
    assert!(said.contains("The request was not sent"));
}

/// The binding's own refusal is the cause of an unbuildable request,
/// and the message names the pool.
#[test]
fn an_unbuildable_request_carries_the_bindings_refusal() {
    // `zc_name` holds the OpenZFS `MAXPATHLEN` bytes, NUL included, so one byte more cannot fit it.
    let long_pool: String = "p".repeat(MAXPATHLEN + 1);
    let refusal: BatchRefusal = destroy_request(long_pool.as_str(), &[])
        .expect_err("a pool name past zc_name's bound cannot fit the command field");
    let said: String = refusal.to_string();
    assert!(said.contains("could not be built"));
    assert!(said.contains("No snapshot was destroyed"));
    assert!(std::error::Error::source(&refusal).is_some());
}
