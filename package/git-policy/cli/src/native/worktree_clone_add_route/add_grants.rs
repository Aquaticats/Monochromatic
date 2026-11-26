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
use crate::doctor_zfs_route::{
    Check,
    CheckOutcome,
    CheckReport,
    DelegationFacts,
    Grant,
    GrantPlace,
    Inheritance,
    RequiredGrant,
    Who,
    allow_command,
    check_delegation,
    missing_grants,
    place_dataset,
};

/// The permissions this add's own steps use,
/// which the preflight refuses a delegation for.
/// Why:
///  The add takes a snapshot,
///  clones it,
///  names the clone in the container,
///  writes the mount request,
///  and fills the mounted clone,
///  so a missing one of these is a step that cannot run at all
///  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
///  choice 3C).
///  No `destroy` grant is here yet:
///  a failed add keeps what it made,
///  and the settle that destroys the clone and the origin snapshot is plan
///  step 14's
///  (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
///  "Whether a failed add may finish with Git's own checkout by itself",
///  whose fallback the build deferred;
///  `super::add_route_run`,
///  "the fallback that finishes a failed add with Git's own checkout is step
///  11's, so this release refuses nothing it cannot resume";
///  `super::add_source_the_restore_cannot_prove`,
///  "the settle that destroys them is plan step 14's").
///  Choice 3C moves the two `destroy` grants into this table at the step that
///  makes the add's own failure path destroy,
///  and `source_guard/add_route_issues_no_destroy_tests.rs` is the check that
///  fails when that step names a destroy without moving them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_ADDS_OWN_STEPS = [['checkout', 'snapshot'], ..., ['container-descendants', 'userprop']];
/// ```
pub const THE_ADDS_OWN_STEPS: &[(GrantPlace, &str)] = &[
    (GrantPlace::Checkout, "snapshot"),
    (GrantPlace::CheckoutDescendants, "clone"),
    (GrantPlace::Container, "create"),
    (GrantPlace::Container, "mount"),
    (GrantPlace::ContainerDescendants, "mount"),
    (GrantPlace::ContainerDescendants, "userprop"),
];

/// The permissions only a removal's steps use,
/// which the preflight warns about and never refuses for.
/// Why:
///  Removing a clone destroys the clone,
///  and clearing an origin snapshot after a removal mounts a snapshot of the
///  checkout and destroys it,
///  which no step of an add does,
///  so a missing one of these is the removal's command and not this add's
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Scoped delegation").
///  The two `destroy` grants are the pair choice 3C keeps here until the add's
///  own failure path destroys,
///  which is plan step 14's settle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_REMOVALS_STEPS = [['container-descendants', 'destroy'], ['checkout-descendants', 'mount'], ['checkout-descendants', 'destroy']];
/// ```
pub const A_REMOVALS_STEPS: &[(GrantPlace, &str)] = &[
    (GrantPlace::ContainerDescendants, "destroy"),
    (GrantPlace::CheckoutDescendants, "mount"),
    (GrantPlace::CheckoutDescendants, "destroy"),
];

/// What:
///  Why one required permission is in the add's own set or the removal's.
/// Why:
///  The table is the one place the split is decided,
///  so a permission that moves between the steps moves one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTheAddsOwn = (required) => THE_ADDS_OWN_STEPS.some(([place, permission]) => place === required.place && permission === required.permission);
/// ```
fn the_adds_own(required: &RequiredGrant) -> bool {
    return THE_ADDS_OWN_STEPS.iter().any(|(place, permission)| {
        return *place == required.place && *permission == required.permission;
    });
}

/// The permissions the removal's steps own,
/// which are the decided ones the add's table does not name.
fn a_removals_own(required: &RequiredGrant) -> bool {
    return A_REMOVALS_STEPS.iter().any(|(place, permission)| {
        return *place == required.place && *permission == required.permission;
    });
}

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
pub fn refuse_without_the_decided_grants(
    device: &OpenedDevice,
    route: &RouteFacts,
) -> Result<Vec<String>, StepRefusal> {
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
pub fn read_the_grants(
    device: &OpenedDevice,
    route: &RouteFacts,
) -> Result<DelegationFacts, StepRefusal> {
    let mut grants: Vec<Grant> = device
        .grants_at(route.checkout_dataset.as_str())
        .map_err(StepRefusal::refused)?;
    grants.extend(
        device
            .grants_at(route.container_dataset.as_str())
            .map_err(StepRefusal::refused)?,
    );
    let account: u32 = this_account();
    return Ok(DelegationFacts {
        account,
        groups: groups_of_this_account()?,
        checkout_dataset: route.checkout_dataset.clone(),
        container_dataset: route.container_dataset.clone(),
        grants,
    });
}

/// What:
///  Refuse when `doctor`'s delegation check does not pass on the facts,
///  except for a missing permission only a removal's steps use,
///  which the add warns about and goes on past.
/// Why:
///  The finding already names every failing permission with the command that
///  settles it,
///  so the refusal adds only what to do after that command and changes nothing.
///  A missing permission the add never uses would cost a person a root command
///  and a second add before their first one finished,
///  so it is printed once and the add goes on
///  (`doc/planning/cli-git-zfs-step-10-open-choices.md`,
///  choice 3C).
///
/// # Errors
/// A finding about a permission the add itself uses,
/// a wider grant,
/// or an unproved delegation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const warnings = refuseUnlessTheDecidedOnes(facts); // throws the check's line
/// ```
pub fn refuse_unless_the_decided_ones(facts: &DelegationFacts) -> Result<Vec<String>, StepRefusal> {
    let report: CheckReport = CheckReport::new(Check::Delegation, check_delegation(facts));
    if report.passed() {
        return Ok(Vec::new());
    }
    let outcome: CheckOutcome = check_delegation(facts);
    let CheckOutcome::Finding {
        field,
        ..
    } = &outcome
    else {
        return Err(the_refusal(report.line()));
    };
    // The check answers a missing permission for the whole decided set at once,
    // so the split is decided from the values rather than from its sentence.
    if field != "grant" {
        return Err(the_refusal(report.line()));
    }
    let missing: Vec<RequiredGrant> = missing_grants(facts);
    if missing.is_empty() {
        return Err(the_refusal(report.line()));
    }
    let ours: Vec<String> = missing
        .iter()
        .filter(|required| return the_adds_own(required))
        .map(|required| {
            return format!(
                "`{}` on `{}` ({})",
                required.permission,
                place_dataset(required.place, facts),
                allow_command(required, facts)
            );
        })
        .collect();
    if !ours.is_empty() {
        return Err(StepRefusal::refused(format!(
            "the ZFS clone route cannot run with this account's delegation: {} decided permission(s) the add's own steps use are not granted to the account: {}. Ways forward: run the command the finding names, then run the add again; the add changed nothing.",
            ours.len(),
            ours.join("; ")
        )));
    }
    let theirs: Vec<String> = missing
        .iter()
        .filter(|required| return a_removals_own(required))
        .map(|required| return format!("{}\n", allow_command(required, facts)))
        .collect();
    // What:
    //  A missing permission of the removal's group is one line the add prints
    //  and goes on past.
    // Why:
    //  No step of this add destroys a clone or an origin snapshot,
    //  so refusing here would cost a person two root commands and a second add
    //  for permissions nothing of the add uses
    //  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
    //  choice 3C).
    if theirs.is_empty() {
        // A decided permission the two tables do not name is a table that has
        // drifted from the decided set, and the add refuses rather than warn.
        return Err(the_refusal(report.line()));
    }
    // What:
    //  Every judgement the check makes after a missing permission is asked
    //  again about a delegation that has the removal's permission in the
    //  decided shape,
    //  so a grant that is effective only through a wider ancestor still refuses
    //  instead of riding along behind the warning.
    // Why:
    //  The check stops at the first kind it finds,
    //  and a removal's permission granted on an ancestor defeats the breadth the
    //  design's own option bought,
    //  which the record says refuses whatever group the permission belongs to
    //  (`doc/planning/cli-git-zfs-step-10-open-choices.md`,
    //  choice 3C,
    //  "What it does for each outcome kind").
    let settled: DelegationFacts = with_the_removals_granted(facts, &missing);
    let after: CheckReport = CheckReport::new(Check::Delegation, check_delegation(&settled));
    if !after.passed() {
        return Err(the_refusal(after.line()));
    }
    return Ok(vec![format!(
        "monochromatic-git-policy-cli: this add will finish, and a removal cannot do its part until you run, as root:\n{}\nThe add changed nothing about that yet.\n",
        theirs.join("")
    )]);
}

/// What:
///  The facts with one grant of the decided shape for every missing permission
///  only a removal uses,
///  so the check's narrower judgements can be asked about them.
/// Why:
///  The add goes on past those permissions,
///  and the rest of the delegation still has to be the decided one:
///  a wider grant, a grant to everyone or an unproved one refuses on its own.
fn with_the_removals_granted(
    facts: &DelegationFacts,
    missing: &[RequiredGrant],
) -> DelegationFacts {
    let mut settled: DelegationFacts = facts.clone();
    for required in missing
        .iter()
        .filter(|required| return a_removals_own(required))
    {
        settled.grants.push(Grant {
            carried_by: String::from(place_dataset(required.place, facts)),
            permission: String::from(required.permission),
            who: Who::User(facts.account),
            inheritance: the_inheritance(required.place),
        });
    }
    return settled;
}

/// How widely the design grants a place's permission.
fn the_inheritance(place: GrantPlace) -> Inheritance {
    return match place {
        GrantPlace::Checkout | GrantPlace::Container => Inheritance::Local,
        GrantPlace::CheckoutDescendants | GrantPlace::ContainerDescendants => {
            Inheritance::Descendants
        }
    };
}

/// The sentence a delegation the add cannot run under prints.
fn the_refusal(line: String) -> StepRefusal {
    return StepRefusal::refused(format!(
        "the ZFS clone route cannot run with this account's delegation: {line}. Ways forward: run the command the finding names, then run the add again; the add changed nothing."
    ));
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
        return Err(the_groups_cannot_be_read());
    }
    if counted == 0 {
        return Ok(Vec::new());
    }
    let mut filled: Vec<libc::gid_t> = vec![0; counted as usize];
    // SAFETY: the buffer holds exactly `counted` ids, which is the count the kernel just answered.
    let read: libc::c_int = unsafe { libc::getgroups(counted, filled.as_mut_ptr()) };
    if read < 0 {
        return Err(the_groups_cannot_be_read());
    }
    filled.truncate(read as usize);
    return Ok(filled
        .into_iter()
        .map(|group| return group as u32)
        .collect());
}

/// Why this account's groups could not be read,
/// and what a person does next.
fn the_groups_cannot_be_read() -> StepRefusal {
    return StepRefusal::refused(format!(
        "this account's groups cannot be read ({}), so nothing proves which group's ZFS grant reaches the account. Ways forward: run the add from a session whose groups the kernel will name.",
        std::io::Error::last_os_error()
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_grants_tests.rs"]
mod tests;
