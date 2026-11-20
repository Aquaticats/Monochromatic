//! What:
//!  The target allowlist:
//! `x86_64` passes and every other target refuses before a file is read.
//! Why:
//!  The proof is compiled for one architecture,
//! and the design refuses the other
//!  until a guest qualifies it.

use super::gate_target::{QUALIFIED_TARGETS, current_target, target_refusal};
use monochromatic_zfs_request_to_openzfs_kernel_device::CompatibilityStep as Step;

#[test]
fn this_binary_runs_on_a_qualified_target() {
    assert_eq!(QUALIFIED_TARGETS, ["x86_64"]);
    assert!(target_refusal(current_target()).is_none());
}

#[test]
fn another_target_refuses_at_the_type_step() {
    let refusal = target_refusal("aarch64").expect("aarch64 is not qualified");
    assert_eq!(refusal.step, Step::Type);
    assert!(refusal.file.is_none());
    assert!(refusal.detail.contains("aarch64"), "{}", refusal.detail);
    assert!(refusal.detail.contains("x86_64"), "{}", refusal.detail);
}
