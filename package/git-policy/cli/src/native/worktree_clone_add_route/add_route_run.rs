//! What:
//!  The ZFS add route's entry point and runner: the gate, the preflight, the
//!  claim and the record, the phases a resume continues from, and the end that
//!  repairs the new worktree and hands its trash over.
//! Why:
//!  The wrapper's one add path asks this before it builds a ladder,
//!  and `None` keeps every add on another filesystem, another checkout dataset,
//!  or whose source the fill's own restore would refuse,
//!  exactly as it was
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, step 10, and
//!  `super::add_source_the_restore_cannot_prove` for the sources the route leaves
//!  to the ladder until the removal's writing path is reachable:
//!  its Git settle is plan step 16's,
//!  and no command of this release reaches it).
//!  A failed or killed add keeps the claim, the slot and the locked
//!  registration, and the next add at the slot reads the record's phase and
//!  continues from it
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`, A0 to A12 and
//!  W5 and W6 of the fourth check).
//!  The fallback that finishes a failed add with Git's own checkout is step 11's,
//!  so this release refuses nothing it cannot resume.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finished = runAZfsAdd(realGit, args, overlay, facts); // null: the ordinary add
//! ```

use super::add_capture_store::clear_other_adds_records;
use super::add_plan_stop::refuse_while_a_plan_stands;
use super::add_preflight::{
    Preflight,
    preflight_the_add,
};
use super::add_route_gate::{
    RouteFacts,
    the_routes_add,
};
use super::add_route_plan::{
    AddNames,
    Stops,
    name_the_add,
    request_of,
};
use super::add_route_steps::{
    StepRefusal,
    Steps,
    step_clone,
    step_fill,
    step_git,
    step_mount,
    step_snapshot,
    write_the_record,
};
use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::transaction_git::GitContext;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::record_publication::{
    RecordRefusalAtTheFile,
    read_binding,
};
use crate::worktree_clone_binding_record::registration_home::{
    binding_path,
    claim_generation_of,
    claim_prefix,
    home_folder,
    open_or_create_home,
};
use crate::worktree_command_facts::CopyFacts;
use crate::worktree_copy_forward::ForwardedRun;
use crate::worktree_identity::WorktreeIdentity;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::ffi::OsString;
use std::io::Write as _;
use std::path::PathBuf;

/// The line a filled clone prints,
/// unless the person asked for no lines.
pub const FILLED_NOTE: &str = "monochromatic-git-policy-cli: made the worktree as a ZFS clone of the main checkout; the copied Git administration was exchanged for the saved pointer and left in the wrapper's trash.\n";

/// What:
///  Run the ZFS route for one add, or leave it to the ordinary path.
/// Why:
///  `None` is the answer for every add the route does not serve,
///  which is every add on another filesystem, every add in a checkout that is
///  not the design's dataset, and every add whose source the fill's own restore
///  would refuse until the settle that cleans up after it lands
///  (`super::add_source_the_restore_cannot_prove`).
pub fn run_a_zfs_add(
    real_git: &std::path::Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    facts: &CopyFacts,
) -> Option<ForwardedRun> {
    let WorktreeIdentity::MainWorktree {
        common_dir,
        ..
    } = &facts.identity
    else {
        return None;
    };
    let destination: &std::path::Path = facts.destination.as_deref()?;
    let plan: &crate::worktree_command_facts::AddPlan = facts.add_plan.as_ref()?;
    let route: RouteFacts = match the_routes_add(&facts.identity, destination, &LiveMountedDatasets)
    {
        Ok(None) => return None,
        Ok(Some(found)) => found,
        Err(reason) => {
            return Some(ForwardedRun::Failed {
                stderr: format!("monochromatic-git-policy-cli: {reason}\n"),
                code: REFUSED_EXIT_CODE,
            });
        }
    };
    // The account companion runs in this add's own process, after the gate above and before the claim,
    // so a refusal leaves no managed object (`doc/decision/cli-git-zfs-start-probe.md`, Q-a and Q6).
    // Its runtime folder is read here, once, and handed to the check as an input: a variable that names no folder,
    // or a folder that is not a directory, refuses the managed attach in the same words a person already reads.
    let runtime = crate::account_companion::the_accounts_runtime_folder()?;
    if !crate::account_companion::the_managed_attach_may_go_ahead(&runtime) {
        return None;
    }
    let context: GitContext = GitContext {
        real_git: real_git.to_path_buf(),
        overlay: overlay.to_vec(),
        global_prefix: Vec::new(),
        common_dir: Some(common_dir.clone()),
    };
    return match run_the_route(&route, plan, &context, real_git, arguments, overlay) {
        Ok(None) => None,
        Ok(Some(lines)) => {
            for line in lines {
                let _written: std::io::Result<()> = std::io::stderr().write_all(line.as_bytes());
            }
            Some(ForwardedRun::Ran(0))
        }
        Err(failure) => Some(match failure.message.is_empty() {
            true => ForwardedRun::Ran(failure.code),
            false => ForwardedRun::Failed {
                stderr: format!("monochromatic-git-policy-cli: {}\n", failure.message),
                code: failure.code,
            },
        }),
    };
}

/// What:
///  The route's own run,
///  whose success is the lines the person reads
///  and whose `None` is the add the route declines.
/// Why:
///  The decline is the preflight's, and it stands before the claim,
///  so a declined add leaves nothing this route would resume and reaches the
///  ladder with nothing changed.
fn run_the_route(
    route: &RouteFacts,
    plan: &crate::worktree_command_facts::AddPlan,
    context: &GitContext,
    real_git: &std::path::Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> Result<Option<Vec<String>>, StepRefusal> {
    let stops: Stops = Stops::from_environment().map_err(StepRefusal::refused)?;
    let Some(preflight): Option<Preflight> = preflight_the_add(route, plan, context, arguments)?
    else {
        return Ok(None);
    };
    let home: std::fs::File = open_or_create_home(route.common_dir.as_path()).map_err(|error| {
        return StepRefusal::refused(format!(
            "the wrapper's record folder under {} cannot be opened ({error})",
            route.common_dir.display()
        ));
    })?;
    let home_path: PathBuf = home_folder(route.common_dir.as_path());
    let mut claim = take_the_claim(home_path.as_path(), route.slot.as_str())?;
    let generation: u64 = the_claims_generation(home_path.as_path(), route.slot.as_str())?;
    let mut record: BindingRecord = match read_binding(home_path.as_path(), route.slot.as_str()) {
        Ok(found) => resume_the_record(route, plan, arguments, found, generation)?,
        Err(RecordRefusalAtTheFile::Missing) => {
            fresh_the_record(route, plan, arguments, &preflight, &home, generation)?
        }
        Err(refusal) => {
            return Err(StepRefusal::refused(
                refusal.message(&home_path.join(format!("{}.binding", route.slot))),
            ));
        }
    };
    if record.phase == Phase::Filled {
        // The slot is a whole worktree, and a person reads its state, its dataset and the one next safe action rather
        // than a guess between a deleted worktree and a pending add
        // (`doc/planning/cli-git-zfs-worktree-design-proposal.md`, "Unmounted, pruned or moved worktrees"; step 12).
        let slot = crate::worktree_clone_link_repair::the_slot_at(
            route.common_dir.as_path(),
            route.slot.as_str(),
            route.destination.as_path(),
        );
        let mut message: String = format!(
            "the worktree {} is already a whole ZFS clone, and Git refuses a second worktree at a slot that holds one. Nothing was added; the slot's state follows.",
            route.destination.display()
        );
        for line in slot.lines() {
            message.push('\n');
            message.push_str(line.as_str());
        }
        return Err(StepRefusal::refused(message));
    }
    if record.phase == Phase::Settling {
        return Err(StepRefusal::refused(format!(
            "the worktree {} says an unfinished add is settling, which this release does not finish; its clone stands and the settling command is plan step 16's. Ways forward: run `git worktree remove {}` after inspecting it",
            route.destination.display(),
            route.slot
        )));
    }
    let names: AddNames = AddNames {
        nonce: record.nonce.clone(),
        origin_snapshot: record.origin_snapshot.clone(),
        clone_dataset: record.clone_dataset.clone(),
        request: record.request.clone(),
    };
    let steps: Steps<'_> = Steps {
        route,
        names: &names,
        source: &preflight.source,
        plan,
        context,
        home: &home,
        home_path: home_path.as_path(),
        device: &preflight.device,
        checkout_guid: preflight.checkout_guid,
        pool_guid: preflight.pool_guid,
        arguments,
        real_git,
        overlay,
        quiet: plan.quiet,
        stops: &stops,
        out: std::cell::RefCell::new(Vec::new()),
    };
    for warning in preflight.warnings.iter() {
        steps.out.borrow_mut().push(warning.clone());
    }
    let phase_before: Phase = record.phase;
    match phase_before {
        // The scan decides first: at phase `claimed` or `slot` Git's own add may
        // already have run and left a registration, and only a scan that finds
        // nothing lets the slot be verified and Git run again.
        Phase::Claimed | Phase::Slot => {}
        // Before the intent is written the slot must be the underlay the
        // registration names; from the intent on, a handler may have mounted the
        // clone there, and the design compares the mount's source instead.
        Phase::Cloned => {
            let mounted: Option<
                crate::worktree_clone_mount::attached_mount_identity::MountIdentity,
            > = super::add_mount::the_mount_at(
                route.destination.as_path(),
                record.clone_dataset.as_str(),
            )
            .map_err(StepRefusal::refused)?;
            if mounted.is_none() {
                verify_the_slot_identity(&steps, &record)?;
            }
        }
        Phase::Registered | Phase::Snapshotted => verify_the_slot_identity(&steps, &record)?,
        _mounted => {}
    }
    if phase_before == Phase::Claimed || phase_before == Phase::Slot {
        step_git(&steps, &mut record)?;
    }
    step_snapshot(&steps, &mut record)?;
    step_clone(&steps, &mut record)?;
    step_mount(&steps, &mut record)?;
    let trash = match record.phase {
        Phase::Mounted => step_fill(&steps, &mut record)?,
        _filled => None,
    };
    finish_the_add(&steps, trash, &mut record)?;
    claim.release().map_err(|error| return StepRefusal::refused(format!("the claim on {} cannot be released ({error:?}); the worktree is whole and the claim's own liveness rule lets the next add go on", route.slot)))?;
    let mut lines: Vec<String> = steps.out.borrow().clone();
    if !plan.quiet {
        lines.push(String::from(FILLED_NOTE));
    }
    return Ok(Some(lines));
}

/// Take the slot's claim, or refuse with the live owner's process id.
///
/// Public to the crate so the removal's controls can show an add meeting the claim a
/// removal holds through this route's own call rather than a copy of its arguments.
pub(crate) fn take_the_claim(
    home: &std::path::Path,
    slot: &str,
) -> Result<crate::single_owner_claim::OwnerClaim, StepRefusal> {
    return match crate::single_owner_claim::take_claim(home, claim_prefix(slot).as_os_str()) {
        Ok(crate::single_owner_claim::ClaimOutcome::Held(claim)) => Ok(claim),
        Ok(crate::single_owner_claim::ClaimOutcome::Busy(owner)) => {
            Err(StepRefusal::refused(format!(
                "another command holds the claim on {slot} (process {}), so this add changed nothing. Ways forward: wait for that command to end, or stop it if it is not running",
                owner.pid
            )))
        }
        Err(error) => Err(StepRefusal::refused(format!(
            "the claim on {slot} cannot be taken ({error:?}), so nothing was changed"
        ))),
    };
}

/// The generation the claim this process holds took.
fn the_claims_generation(
    home: &std::path::Path,
    slot: &str,
) -> Result<u64, StepRefusal> {
    let listing: std::fs::ReadDir = std::fs::read_dir(home).map_err(|error| {
        return StepRefusal::refused(format!(
            "the record home {} cannot be listed ({error})",
            home.display()
        ));
    })?;
    let mut newest: u64 = 0;
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| {
            return StepRefusal::refused(format!(
                "the record home {} cannot be listed ({error})",
                home.display()
            ));
        })?;
        let Some(name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if let Some(generation) = claim_generation_of(name.as_str(), slot) {
            newest = newest.max(generation);
        }
    }
    if newest == 0 {
        return Err(StepRefusal::refused(format!(
            "no claim generation for {slot} stands after this command took one"
        )));
    }
    return Ok(newest);
}

/// Read a standing record and prove it is this add's continuation.
fn resume_the_record(
    route: &RouteFacts,
    plan: &crate::worktree_command_facts::AddPlan,
    arguments: &[OsString],
    mut record: BindingRecord,
    generation: u64,
) -> Result<BindingRecord, StepRefusal> {
    if record.repository_path().as_deref() != Some(route.common_dir.as_path())
        || record.slot_path().as_deref() != Some(route.destination.as_path())
    {
        let shown: String = record.repository_path().map_or_else(
            || return String::from("none"),
            |path| return path.display().to_string(),
        );
        return Err(StepRefusal::refused(format!(
            "the record at the slot names repository {shown} and another slot's path, so it is not this repository's record for {}. Ways forward: inspect it and remove it if it is not the add's",
            route.destination.display()
        )));
    }
    if record.checkout_dataset != route.checkout_dataset {
        return Err(StepRefusal::refused(format!(
            "the record names the checkout dataset {} and this checkout is {}, so nothing in it can be trusted; Ways forward: inspect the record and remove it, then run the add again",
            record.checkout_dataset, route.checkout_dataset
        )));
    }
    let request = request_of(plan, arguments).map_err(StepRefusal::refused)?;
    if record.request != request {
        return Err(StepRefusal::refused(format!(
            "the record was written by an add given `{}` and this add was given `{}`; Ways forward: run the same add again, which continues that record, or remove the worktree with `git worktree remove {}` and then delete the binding record at {} yourself, because Git's removal leaves that record and no command of this release removes it",
            record.request.encode(),
            request.encode(),
            route.slot,
            binding_path(route.common_dir.as_path(), route.slot.as_str()).display()
        )));
    }
    record.claim = generation;
    return Ok(record);
}

/// Write the record's first version before any mutation beyond the claim.
fn fresh_the_record(
    route: &RouteFacts,
    plan: &crate::worktree_command_facts::AddPlan,
    arguments: &[OsString],
    preflight: &Preflight,
    home: &std::fs::File,
    generation: u64,
) -> Result<BindingRecord, StepRefusal> {
    // A plan that names the clone comes from a removal's record design, and the
    // plan's head word and the clone the pool can find decide the one command the
    // person reads beside it (plan step 10's choice 2C). The same call stands in the
    // preflight, so one state prints one sentence.
    refuse_while_a_plan_stands(route, &preflight.device)?;
    let names: AddNames = name_the_add(route, plan, arguments).map_err(StepRefusal::refused)?;
    clear_other_adds_records(
        home_folder(route.common_dir.as_path()).as_path(),
        route.slot.as_str(),
        &names.nonce,
    )
    .map_err(StepRefusal::refused)?;
    let repository_identity: FolderIdentity = FolderIdentity::of_folder(route.common_dir.as_path())
        .map_err(|error| {
            return StepRefusal::refused(format!(
                "the common Git directory cannot be identified ({error})"
            ));
        })?;
    let record: BindingRecord = BindingRecord {
        claim: generation,
        nonce: names.nonce,
        request: names.request,
        phase: Phase::Claimed,
        settle_reason: None,
        checkout_dataset: route.checkout_dataset.clone(),
        checkout_dataset_guid: preflight.checkout_guid,
        repository: route.common_dir.as_os_str().as_encoded_bytes().to_vec(),
        repository_identity,
        slot: route.destination.as_os_str().as_encoded_bytes().to_vec(),
        slot_identity: None,
        registration: None,
        registration_identity: None,
        pool: route.pool.clone(),
        pool_guid: preflight.pool_guid,
        origin_snapshot: names.origin_snapshot,
        origin_snapshot_guid: None,
        origin_snapshot_createtxg: None,
        origin_dataset: None,
        origin_dataset_guid: None,
        clone_dataset: names.clone_dataset,
        clone_guid: None,
        clone_createtxg: None,
        pointer: None,
    };
    crate::worktree_clone_binding_record::record_publication::write_binding(home, route.slot.as_str(), &record).map_err(|error| return StepRefusal::refused(format!("the add's first record write was refused ({error}), so nothing beyond the claim was changed")))?;
    return Ok(record);
}

/// Verify the unmounted slot's identity before a step that relies on it.
fn verify_the_slot_identity(
    steps: &Steps<'_>,
    record: &BindingRecord,
) -> Result<(), StepRefusal> {
    let Some(recorded) = record.slot_identity else {
        return Err(StepRefusal::refused(String::from(
            "the record names no slot identity, so nothing proves the slot Git registered is this add's",
        )));
    };
    let live: FolderIdentity = FolderIdentity::of_folder(steps.route.destination.as_path())
        .map_err(|error| {
            return StepRefusal::refused(format!(
                "the slot {} cannot be identified ({error})",
                steps.route.destination.display()
            ));
        })?;
    if live != recorded {
        return Err(StepRefusal::refused(format!(
            "the slot {} is another folder than the one the registration was made in ({} instead of {}); Ways forward: inspect the slot, then remove the worktree and run the add again",
            steps.route.destination.display(),
            live.encode(),
            recorded.encode()
        )));
    }
    return Ok(());
}

/// Repair the new worktree and hand its trash over; neither fails the add.
fn finish_the_add(
    steps: &Steps<'_>,
    trash: Option<crate::worktree_trash::PendingTrash>,
    record: &mut BindingRecord,
) -> Result<(), StepRefusal> {
    if let Some(pending) = trash {
        let mut notices = std::io::stderr();
        if let Err(reason) =
            crate::worktree_trash::hand_over_to_the_service(steps.context, &pending, &mut notices)
        {
            let _written: std::io::Result<()> = writeln!(
                std::io::stderr(),
                "monochromatic-git-policy-cli: the trash of {} could not be handed to the service ({reason}); its record still names it",
                steps.route.destination.display()
            );
        }
    }
    #[cfg(target_os = "linux")]
    {
        let environment: crate::package_repair::RepairEnvironment =
            crate::package_repair::RepairEnvironment::from_process();
        let request = crate::package_repair::RepairRequest {
            git: steps.context,
            source: steps.route.source_root.as_path(),
            destination: steps.route.destination.as_path(),
            switch: crate::package_repair::RepairSwitch::from_arguments(steps.arguments),
            environment: &environment,
        };
        let mut findings = std::io::stderr();
        crate::package_repair::repair_worktree(&request, &mut findings);
    }
    record.phase = Phase::Filled;
    return write_the_record(steps, record);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_route_run_tests.rs"]
mod tests;
