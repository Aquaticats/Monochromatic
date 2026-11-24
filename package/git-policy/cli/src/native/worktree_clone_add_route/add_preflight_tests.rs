//! Controls for the preflight's own shape:
//!  it makes the delegation read once,
//!  and that read answers before the claim is taken.
//!
//! Why a source scan:
//!  the preflight opens the kernel's device and reads a live pool,
//!  so no host control can drive it,
//!  and this crate's own idiom for the shape a release file must have is
//!  `source_guard`.
//!  The arm fails when the read is dropped from the preflight,
//!  which is the defect the read was built for.

/// The preflight makes the delegation read once,
/// inside itself,
/// before the runner takes the claim.
#[test]
fn the_preflight_refuses_through_the_grant_read() {
    let source: &str = include_str!("add_preflight.rs");
    let code: Vec<&str> = source
        .lines()
        .map(str::trim_start)
        .filter(|line| return !line.starts_with("//"))
        .collect();
    let joined: String = code.join("\n");
    assert_eq!(joined.matches("refuse_without_the_decided_grants(").count(), 1, "the preflight makes the delegation read once: {joined}");
    let read: usize = joined.find("refuse_without_the_decided_grants(").expect("the read is in the preflight");
    let preflight_function: usize = joined.find("fn preflight_the_add(").expect("the preflight is a function of this file");
    assert!(preflight_function < read, "the read sits inside the preflight's own function");
    let runner: &str = include_str!("add_route_run.rs");
    let runner_code: Vec<&str> = runner
        .lines()
        .map(str::trim_start)
        .filter(|line| return !line.starts_with("//"))
        .collect();
    let runner_joined: String = runner_code.join("\n");
    let preflight_call: usize = runner_joined.find("preflight_the_add(").expect("the runner calls the preflight");
    let claim_taken: usize = runner_joined.find("take_the_claim(").expect("the runner takes the claim");
    assert!(preflight_call < claim_taken, "the runner calls the preflight before it takes the claim");
}
