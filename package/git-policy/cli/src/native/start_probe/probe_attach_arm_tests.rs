//! What:
//!  Controls of the attach arm that a machine without the source can hold:
//!  a source no pool names is refused at the exclusive create with `ENOENT`,
//!  and the failure names that step.
//! Why:
//!  The record must attribute a missing dataset or snapshot to the arm and the
//!  call rather than inventing a cause,
//!  and this control runs only where a `zfs` context can be opened at all,
//!  which the guest arms cover.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(attachSnapshot(slot, 'no-such-pool/ds@ready').step).toBe('attach-exclusive-create');
//! ```

use super::attach_snapshot;
use crate::testing::test_support::fixture;
use std::os::fd::OwnedFd;
use std::path::PathBuf;

/// A source the pool does not hold is refused at the exclusive create,
/// and the failure names that step and its errno.
#[test]
fn a_missing_source_is_refused_at_the_exclusive_create() {
    if super::super::probe_descriptor_calls::fsopen("zfs").is_err() {
        // The kernel this control runs on has no `zfs` filesystem, so the
        // attach's first call refuses and the guest arms carry the rest.
        return;
    }
    let root: PathBuf = fixture("start-probe-attach-arm");
    let slot: PathBuf = root.join("slot");
    std::fs::create_dir(slot.as_path()).expect("the fixture slot is made");
    let handle: OwnedFd = super::super::probe_open_beneath::open_path_handle(slot.as_path()).expect("the slot handle opens");
    let refusal = attach_snapshot(&handle, slot.as_path(), "no-such-pool/git-worktree-probe@ready", false, |_| {}).expect_err("the source is refused");
    assert_eq!(refusal.step, super::super::probe_arms::ProbeStep::AttachExclusiveCreate);
    assert_eq!(refusal.errno, Some(libc::ENOENT));
}
