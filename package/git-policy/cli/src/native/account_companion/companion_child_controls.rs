//! What:
//!  The four refusal controls the companion runs inside the tree it owns,
//!  each of which must refuse and must refuse with the answer the design names.
//! Why:
//!  A control that attached would make the walk's refusals unproven,
//!  and the walk above is their success control,
//!  so a companion that refuses everything fails here rather than passing
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The companion's arms" and M5's shape).
//!  A refusal with another errno is a failure of this step and names the
//!  control,
//! so a kernel that changed an answer is visible.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(openat2(refusal, 'link')).rejects(ELOOP);
//! expect(openat2(root, '../escape')).rejects(EXDEV);
//! expect(walk('refusal/bound/slot')).rejects(EXDEV);
//! expect(identityAfterReplace).not.toEqual(recorded);
//! ```

use super::companion_buffers::CompanionBuffers;
use super::companion_calls::{CompanionCall, Failure, NO_ERRNO};
use super::companion_child_tree::Slots;
use super::companion_descriptor_calls::open_directory_beneath;
use super::companion_file_calls::{Identity, close, identity_of, make_directory, make_symlink, rename_within};
use super::companion_mount_calls::bind_mount;

/// What:
///  Run every refusal control,
///  each of which must refuse with its own answer.
/// Why:
///  The controls are what make the walk's refusals a proof rather than a hope,
///  and each one names the answer it must give.
///
/// # Errors
/// A control did not refuse as designed,
///  or a call it needed refused,
///  and the failure names the control and what it answered.
pub fn run_the_controls(buffers: &CompanionBuffers, slots: Slots) -> Result<(), Failure> {
    let refusal: std::os::fd::RawFd = open_directory_beneath(slots.root, buffers.refusal_name.as_c_str())?;
    let answered: Result<(), Failure> = run_inside(buffers, slots, refusal);
    close(refusal);
    return answered;
}

/// The controls inside the refusal tree,
/// which needs that folder's own descriptor.
fn run_inside(buffers: &CompanionBuffers, slots: Slots, refusal: std::os::fd::RawFd) -> Result<(), Failure> {
    make_symlink(buffers.slot_name.as_c_str(), refusal, buffers.link_name.as_c_str())?;
    expect_errno(open_directory_beneath(refusal, buffers.link_name.as_c_str()).map(|opened| return close(opened)), "link", libc::ELOOP)?;
    expect_errno(open_directory_beneath(slots.root, buffers.escape_name.as_c_str()).map(|opened| return close(opened)), "escape", libc::EXDEV)?;
    bind_mount(buffers.slot.as_c_str(), buffers.bound_slot.as_c_str())?;
    let crossing: Result<(), Failure> = crossed(buffers, slots.root);
    expect_errno(crossing, "crossing", libc::EXDEV)?;
    let recorded: Identity = identity_of(open_directory_beneath(refusal, buffers.replaced_name.as_c_str())?)?;
    rename_within(refusal, buffers.replaced_name.as_c_str(), buffers.replaced_elsewhere_name.as_c_str())?;
    make_directory(buffers.replaced.as_c_str())?;
    let now: Identity = identity_of(open_directory_beneath(refusal, buffers.replaced_name.as_c_str())?)?;
    if now == recorded {
        return Err(Failure { call: CompanionCall::MountFactsCheck, errno: NO_ERRNO });
    }
    return Ok(());
}

/// Walk the three components the bind mount sits across,
/// which must refuse at the crossing.
fn crossed(buffers: &CompanionBuffers, root: std::os::fd::RawFd) -> Result<(), Failure> {
    let refusal = open_directory_beneath(root, buffers.refusal_name.as_c_str())?;
    let bound = open_directory_beneath(refusal, buffers.bound_name.as_c_str())?;
    let crossed = open_directory_beneath(bound, buffers.slot_name.as_c_str());
    close(bound);
    close(refusal);
    return crossed.map(|opened| return close(opened));
}

/// What:
///  Require one control's answer to be the refusal the design names.
/// Why:
///  A control that attached or answered another errno is this step's failure,
///  and its number is what the line prints.
fn expect_errno(answer: Result<(), Failure>, control: &'static str, wanted: i32) -> Result<(), Failure> {
    let _named: &'static str = control;
    return match answer {
        Err(Failure { errno, .. }) if errno == wanted => Ok(()),
        Err(failure) => Err(failure),
        Ok(()) => Err(Failure { call: CompanionCall::Openat2, errno: NO_ERRNO }),
    };
}
