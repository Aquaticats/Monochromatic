//! What:
//!  Controls for the plan:
//! no request for an empty set,
//! one request when the set
//!  fits,
//! the largest fitting prefix per chunk when it does not,
//! the sort that
//!  makes a plan repeatable,
//! and the refusals a single name or a repeated name earns.
//! Why:
//!  The split is the piece's own decision and its control must fail without it:
//!  the split controls assert every chunk's exact encoded size against the
//!  budget and assert that one more name would not have fitted,
//! so a plan that sent
//!  the whole set in one request,
//! or one name per request,
//! fails them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planRequests('zfsstep16', names, RequestBudget.ofInputBytes(sizeOfTwo));
//! expect(plan.chunks().map(c => c.length)).toEqual([2, 2, 1]);
//! ```

use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::request_refusal::BatchRefusal;
use crate::worktree_clone_snapshot_batch::snapshot_batch_plan::{
    RequestPlan,
    plan_requests,
};
use crate::worktree_clone_snapshot_batch::snapshot_destroy_request::encoded_request_bytes;

/// The pool every plan names.
const POOL: &str = "zfsstep16";

/// The full snapshot names of a set,
/// with the given suffixes.
fn names(suffixes: &[&str]) -> Vec<String> {
    return suffixes
        .iter()
        .map(|suffix| {
            return format!("{POOL}/clones/removal@{suffix}");
        })
        .collect();
}

/// A set of five names whose suffixes sort in their given order.
fn five() -> Vec<String> {
    return names(&["s1", "s2", "s3", "s4", "s5"]);
}

/// The budget that exactly fits the given names' own request.
fn budget_for(set: &[String]) -> RequestBudget {
    return RequestBudget::of_input_bytes(encoded_request_bytes(POOL, set).expect("the names fit"));
}

/// The names one request would carry for the first `take` names of `set`.
fn prefix(
    set: &[String],
    take: usize,
) -> Vec<String> {
    return set[..take].to_vec();
}

/// An empty set makes no request at all.
#[test]
fn no_request_is_planned_for_an_empty_set() {
    let plan: RequestPlan = plan_requests(POOL, &[], RequestBudget::of_input_bytes(4096))
        .expect("an empty set refuses nothing");
    assert!(plan.is_empty());
    assert!(plan.chunks().is_empty());
    assert!(plan.encoded_sizes().is_empty());
    assert_eq!(plan.name_count(), 0);
}

/// A set whose own request fits the budget is one request.
#[test]
fn a_set_that_fits_is_one_request() {
    let set: Vec<String> = five();
    let budget: RequestBudget = budget_for(set.as_slice());
    let plan: RequestPlan = plan_requests(POOL, set.as_slice(), budget).expect("the set fits");
    assert_eq!(plan.chunks().len(), 1);
    assert_eq!(plan.chunks()[0], set);
    assert_eq!(plan.encoded_sizes()[0], budget.input_bytes());
    assert_eq!(plan.name_count(), set.len());
}

/// A set over the budget is split by the largest fitting prefix,
/// repeatedly.
#[test]
fn a_set_over_the_budget_is_split_by_the_largest_fitting_prefix() {
    let set: Vec<String> = five();
    let budget: RequestBudget = budget_for(prefix(set.as_slice(), 2).as_slice());
    let plan: RequestPlan = plan_requests(POOL, set.as_slice(), budget).expect("the set plans");
    assert_eq!(plan.chunks().len(), 3);
    assert_eq!(plan.chunks()[0].len(), 2);
    assert_eq!(plan.chunks()[1].len(), 2);
    assert_eq!(plan.chunks()[2].len(), 1);
    for (index, _chunk) in plan.chunks().iter().enumerate() {
        assert!(
            plan.encoded_sizes()[index] <= budget.input_bytes(),
            "chunk {index} must fit the budget"
        );
    }
    // Each chunk but the last is the *largest* prefix that fits: the next name of the
    //  remaining set would have crossed the budget.
    let mut remaining: &[String] = set.as_slice();
    for chunk in plan.chunks() {
        if chunk.len() < remaining.len() {
            let one_more: usize =
                encoded_request_bytes(POOL, &remaining[..chunk.len() + 1]).expect("the names fit");
            assert!(
                one_more > budget.input_bytes(),
                "the split must take the largest fitting prefix, and one more name fits"
            );
        }
        remaining = &remaining[chunk.len()..];
    }
}

/// The split sorts by name,
/// so the same set always makes the same requests.
#[test]
fn the_split_sorts_the_names_by_name() {
    let set: Vec<String> = names(&["s3", "s1", "s2"]);
    let sorted: Vec<String> = names(&["s1", "s2", "s3"]);
    let budget: RequestBudget = budget_for(prefix(sorted.as_slice(), 2).as_slice());
    let plan: RequestPlan = plan_requests(POOL, set.as_slice(), budget).expect("the set plans");
    assert_eq!(plan.chunks().len(), 2);
    assert_eq!(plan.chunks()[0], prefix(sorted.as_slice(), 2));
    assert_eq!(plan.chunks()[1], sorted[2..].to_vec());
    let flattened: Vec<String> = plan
        .chunks()
        .iter()
        .flat_map(|chunk| {
            return chunk.iter().cloned();
        })
        .collect();
    assert_eq!(flattened, sorted);
}

/// The boundary is the encoded request,
/// never a count of names.
#[test]
fn the_boundary_is_the_encoded_request() {
    let set: Vec<String> = names(&["s1", "s2", "s3", "s4"]);
    let three: Vec<String> = prefix(set.as_slice(), 3);
    let budget: RequestBudget = budget_for(three.as_slice());
    let plan: RequestPlan = plan_requests(POOL, set.as_slice(), budget).expect("the set plans");
    assert_eq!(plan.chunks()[0], three);
    assert_eq!(plan.chunks()[1], prefix(set.as_slice(), 4)[3..].to_vec());
    assert!(
        encoded_request_bytes(POOL, &prefix(set.as_slice(), 4)).expect("the names fit")
            > budget.input_bytes()
    );
}

/// One name larger than the whole budget is refused before any request is
/// made.
#[test]
fn one_name_over_the_budget_is_refused_before_any_request() {
    let set: Vec<String> = five();
    let refusal: BatchRefusal =
        plan_requests(POOL, set.as_slice(), RequestBudget::of_input_bytes(1))
            .expect_err("one byte cannot carry a request");
    match refusal {
        BatchRefusal::OneNameOverTheBudget {
            name,
            request_bytes,
            budget_bytes,
        } => {
            assert_eq!(name, set[0]);
            assert_eq!(budget_bytes, 1);
            assert!(request_bytes > budget_bytes);
        }
        other => panic!("the refusal must be the over-budget one, not {other:?}"),
    }
}

/// A set that holds one name twice is refused rather than sent twice.
#[test]
fn a_repeated_name_is_refused() {
    let set: Vec<String> = names(&["s2", "s1", "s2"]);
    let refusal: BatchRefusal = plan_requests(POOL, set.as_slice(), budget_for(set.as_slice()))
        .expect_err("a repeated name has no single per-name outcome");
    assert!(matches!(refusal, BatchRefusal::RepeatedName { .. }));
    if let BatchRefusal::RepeatedName {
        name,
    } = refusal
    {
        assert_eq!(name, set[0]);
    }
}
