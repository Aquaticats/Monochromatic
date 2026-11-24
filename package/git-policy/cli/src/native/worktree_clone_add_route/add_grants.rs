//! What:
//!  The account's effective ZFS delegation over the route's two datasets,
//!  read in the preflight and refused when it is not the decided one.
//! Why:
//!  The design's A0 has the shared preflight prove the grants before the claim,
//!  and the proposal's first step puts "effective grants" in the list a refusal
//!  must answer
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  A0;
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Add and Git registration" step 1).
//!  The judgement is `doctor`'s own check and not a second rule:
//!  one machine's delegation is either the decided shape or it is not,
//!  and an add that measured it differently from `doctor` would refuse where
//!  `doctor` passes or the other way round
//!  (`crate::doctor_zfs_route`,
//!  "The effective delegation and the
//!  lineage-changing grants' absence").
//!  The read needs no privilege:
//!  OpenZFS `zfs-2.4.4` registers `ZFS_IOC_GET_FSACL` with
//!  `zfs_secpolicy_read`,
//!  which answers zero in the global zone
//!  (`module/zfs/zfs_ioctl.c:390` to `:398` and `:7825` to `:7826`),
//!  so the account reads its own delegation through the device the route has
//!  already opened.
//!  `doctor` does not print this read yet,
//!  so the refusal does not send a person there:
//!  it prints the finding's own `zfs allow` or `zfs unallow` command,
//!  and the plan's step 18 is what reports the same bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! refuseWithoutTheDecidedGrants(device, route); // throws the finding and its zfs allow command
//! ```

use super::add_route_gate::RouteFacts;
use super::add_route_steps::StepRefusal;
use super::add_zfs::OpenedDevice;
use crate::doctor_zfs_route::{Check, CheckReport, DelegationFacts, Grant, check_delegation};

/// What:
///  Refuse the add unless the account's delegation is the shape the design
///  scopes,
///  reading it through the device the preflight opened.
/// Why:
///  A refused snapshot or clone would report the kernel's own error after the
///  claim,
///  the first record write and the pool's name were already committed,
///  and a grant the design forbids is a fact no later step can repair.
///
/// # Errors
/// The pool refused the delegation read,
/// or the delegation is not the decided one,
/// each with the sentence the person reads before anything is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseWithoutTheDecidedGrants(device, route);
/// ```
pub fn refuse_without_the_decided_grants(device: &OpenedDevice, route: &RouteFacts) -> Result<(), StepRefusal> {
    let facts: DelegationFacts = read_the_grants(device, route)?;
    return refuse_unless_the_decided_ones(&facts);
}

/// What:
///  The account's effective delegation over the checkout dataset and its sibling
///  clone container,
///  as `doctor`'s check reads it.
/// Why:
///  One reply walks the dataset's ancestors,
///  so two replies cover both places the design grants a permission,
///  and the account is this process's,
///  which is the one the kernel judges.
///
/// # Errors
/// A delegation read the pool refused or answered in a shape this reader cannot
/// represent,
/// and the account's own groups when the kernel will not name them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = readTheGrants(device, route); // { account, groups, checkoutDataset, containerDataset, grants }
/// ```
pub fn read_the_grants(device: &OpenedDevice, route: &RouteFacts) -> Result<DelegationFacts, StepRefusal> {
    let mut grants: Vec<Grant> = device.grants_at(route.checkout_dataset.as_str()).map_err(StepRefusal::refused)?;
    grants.extend(device.grants_at(route.container_dataset.as_str()).map_err(StepRefusal::refused)?);
    let account: u32 = this_account();
    return Ok(DelegationFacts { account, groups: groups_of_this_account()?, checkout_dataset: route.checkout_dataset.clone(), container_dataset: route.container_dataset.clone(), grants });
}

/// What:
///  Refuse when `doctor`'s delegation check does not pass on the facts.
/// Why:
///  The finding already names every failing permission with the command that
///  settles it,
///  so the refusal adds only what to do after that command and changes nothing.
///
/// # Errors
/// A finding or an unproved delegation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseUnlessTheDecidedOnes(facts); // throws the check's line
/// ```
pub fn refuse_unless_the_decided_ones(facts: &DelegationFacts) -> Result<(), StepRefusal> {
    let report: CheckReport = CheckReport::new(Check::Delegation, check_delegation(facts));
    if report.passed() {
        return Ok(());
    }
    return Err(StepRefusal::refused(format!(
        "the ZFS clone route cannot run with this account's delegation: {}. Ways forward: run the command the finding names, then run the add again; the add changed nothing.",
        report.line()
    )));
}

/// The account this process runs as,
/// which is the one the kernel judges a delegation against.
fn this_account() -> u32 {
    // SAFETY: `geteuid` reads this process's own identity and has no preconditions.
    return unsafe { libc::geteuid() };
}

/// Every group this process belongs to,
/// which is what the kernel matches a group's grant against.
fn groups_of_this_account() -> Result<Vec<u32>, StepRefusal> {
    // SAFETY: a zero size with a null buffer only asks the kernel how many groups there are.
    let counted: libc::c_int = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if counted < 0 {
        return Err(StepRefusal::refused(format!(
            "this account's groups cannot be read ({}), so nothing proves which group's ZFS grant reaches the account. Ways forward: run the add from a session whose groups the kernel will name.",
            std::io::Error::last_os_error()
        )));
    }
    if counted == 0 {
        return Ok(Vec::new());
    }
    let mut filled: Vec<libc::gid_t> = vec![0; counted as usize];
    // SAFETY: the buffer holds exactly `counted` ids, which is the count the kernel just answered.
    let read: libc::c_int = unsafe { libc::getgroups(counted, filled.as_mut_ptr()) };
    if read < 0 {
        return Err(StepRefusal::refused(format!(
            "this account's groups cannot be read ({}), so nothing proves which group's ZFS grant reaches the account. Ways forward: run the add from a session whose groups the kernel will name.",
            std::io::Error::last_os_error()
        )));
    }
    filled.truncate(read as usize);
    return Ok(filled.into_iter().map(|group| return group as u32).collect());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_grants_tests.rs"]
mod tests;
