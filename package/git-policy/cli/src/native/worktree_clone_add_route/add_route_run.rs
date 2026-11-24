//! What:
//!  The ZFS add route's entry point and runner: the gate, the preflight, the
//!  claim and the record, the phases a resume continues from, and the end that
//!  repairs the new worktree and hands its trash over.
//! Why:
//!  The wrapper's one add path asks this before it builds a ladder,
//!  and `None` keeps every add on another filesystem or another checkout dataset
//!  exactly as it was
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, step 10).
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

use super::add_route_gate::{RouteFacts, the_routes_add};
use super::add_route_plan::{AddNames, Stops, name_the_add, request_of};
use super::add_route_steps::{StepRefusal, Steps, step_clone, step_fill, step_git, step_mount, step_snapshot, step_slot, write_the_record};
use super::add_zfs::{ObjectIdentity, OpenedDevice};
use crate::worktree_copy_forward::{CopyFacts, ForwardedRun};
use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::transaction_git::GitContext;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::plan_folder_read::refuse_while_a_plan_names_the_clone;
use crate::worktree_clone_binding_record::record_publication::{RecordRefusalAtTheFile, read_binding};
use crate::worktree_clone_binding_record::registration_home::{claim_generation_of, claim_prefix, home_folder, open_or_create_home};
use super::add_capture_store::clear_other_adds_records;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{ReadinessExpectation, ReadinessRecord};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_admission::{AdmissionRefusal, admit_attach};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_snapshot_source::{Refusal as SourceRefusal, SnapshotSource, snapshot_blocker, source_refusal};
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::ffi::OsString;
use std::io::Write as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::PathBuf;

/// The line a filled clone prints,
/// unless the person asked for no lines.
pub const FILLED_NOTE: &str = "monochromatic-git-policy-cli: made the worktree as a ZFS clone of the main checkout; the copied Git administration was exchanged for the saved pointer and left in the wrapper's trash.\n";

/// What:
///  Run the ZFS route for one add, or leave it to the ordinary path.
/// Why:
///  `None` is the answer for every add the route does not serve,
///  which is every add on another filesystem and every add in a checkout that
///  is not the design's dataset.
pub fn run_a_zfs_add(real_git: &std::path::Path, arguments: &[OsString], overlay: &[(OsString, OsString)], facts: &CopyFacts) -> Option<ForwardedRun> {
    let WorktreeIdentity::MainWorktree { common_dir, .. } = &facts.identity else { return None };
    let destination: &std::path::Path = facts.destination.as_deref()?;
    let plan: &crate::worktree_add_plan::AddPlan = facts.add_plan.as_ref()?;
    let route: RouteFacts = match the_routes_add(&facts.identity, destination, &LiveMountedDatasets) {
        Ok(None) => return None,
        Ok(Some(found)) => found,
        Err(reason) => {
            return Some(ForwardedRun::Failed { stderr: format!("monochromatic-git-policy-cli: {reason}\n"), code: REFUSED_EXIT_CODE });
        }
    };
    let context: GitContext = GitContext { real_git: real_git.to_path_buf(), overlay: overlay.to_vec(), global_prefix: Vec::new(), common_dir: Some(common_dir.clone()) };
    return match run_the_route(&route, plan, &context, real_git, arguments, overlay) {
        Ok(lines) => {
            for line in lines {
                let _written: std::io::Result<()> = std::io::stderr().write_all(line.as_bytes());
            }
            Some(ForwardedRun::Ran(0))
        }
        Err(failure) => Some(match failure.message.is_empty() {
            true => ForwardedRun::Ran(failure.code),
            false => ForwardedRun::Failed { stderr: format!("monochromatic-git-policy-cli: {}\n", failure.message), code: failure.code },
        }),
    };
}

/// The route's own run, whose success is the lines the person reads.
fn run_the_route(route: &RouteFacts, plan: &crate::worktree_add_plan::AddPlan, context: &GitContext, real_git: &std::path::Path, arguments: &[OsString], overlay: &[(OsString, OsString)]) -> Result<Vec<String>, StepRefusal> {
    let stops: Stops = Stops::from_environment().map_err(StepRefusal::refused)?;
    let preflight: Preflight = preflight_the_add(route, plan, context, arguments)?;
    let home: std::fs::File = open_or_create_home(route.common_dir.as_path()).map_err(|error| return StepRefusal::refused(format!("the wrapper's record folder under {} cannot be opened ({error})", route.common_dir.display())))?;
    let home_path: PathBuf = home_folder(route.common_dir.as_path());
    let mut claim = take_the_claim(home_path.as_path(), route.slot.as_str())?;
    let generation: u64 = the_claims_generation(home_path.as_path(), route.slot.as_str())?;
    let mut record: BindingRecord = match read_binding(home_path.as_path(), route.slot.as_str()) {
        Ok(found) => resume_the_record(route, plan, arguments, found, generation)?,
        Err(RecordRefusalAtTheFile::Missing) => fresh_the_record(route, plan, arguments, &preflight, &home, generation)?,
        Err(refusal) => return Err(StepRefusal::refused(refusal.message(&home_path.join(format!("{}.binding", route.slot))))),
    };
    if record.phase == Phase::Filled {
        return Err(StepRefusal::refused(format!(
            "the worktree {} is already a whole ZFS clone, and Git refuses a second worktree at a slot that holds one. Ways forward: use it, or remove it with `git worktree remove {}`",
            route.destination.display(),
            route.slot
        )));
    }
    if record.phase == Phase::Settling {
        return Err(StepRefusal::refused(format!(
            "the worktree {} says an unfinished add is settling, which this release does not finish; its clone stands and the settling command is plan step 16's. Ways forward: run `git worktree remove {}` after inspecting it",
            route.destination.display(),
            route.slot
        )));
    }
    let names: AddNames = AddNames { nonce: record.nonce.clone(), origin_snapshot: record.origin_snapshot.clone(), clone_dataset: record.clone_dataset.clone(), request: record.request.clone() };
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
    let phase_before: Phase = record.phase;
    match phase_before {
        Phase::Claimed | Phase::Slot => step_slot(&steps, &mut record)?,
        // Before the intent is written the slot must be the underlay the
        // registration names; from the intent on, a handler may have mounted the
        // clone there, and the design compares the mount's source instead.
        Phase::Cloned => {
            let mounted: Option<crate::worktree_clone_mount::attached_mount_identity::MountIdentity> = super::add_mount::the_mount_at(route.destination.as_path(), record.clone_dataset.as_str()).map_err(StepRefusal::refused)?;
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
    return Ok(lines);
}

/// What:
///  Everything the preflight proves before the claim.
/// Why:
///  A0 writes nothing,
///  and each of its answers is handed to the steps rather than read twice.
struct Preflight {
    /// The proven clean source.
    source: SnapshotSource,
    /// The device opened behind the gate.
    device: OpenedDevice,
    /// The checkout dataset's GUID.
    checkout_guid: DatasetGuid,
    /// The pool's GUID.
    pool_guid: DatasetGuid,
}

/// Prove the supported form, the clean source, the datasets, the handler's
/// readiness and the destination.
fn preflight_the_add(route: &RouteFacts, plan: &crate::worktree_add_plan::AddPlan, context: &GitContext, arguments: &[OsString]) -> Result<Preflight, StepRefusal> {
    if let Some(word) = super::add_git_words::person_lock_word(arguments) {
        return Err(StepRefusal::refused(format!(
            "the add names `{}`, and the ZFS clone route takes the managed lock itself. Ways forward: run the add again without `--lock`, `--reason` or `--checkout`",
            word.to_string_lossy()
        )));
    }
    let identity: WorktreeIdentity = WorktreeIdentity::MainWorktree { common_dir: route.common_dir.clone(), git_dir: route.git_dir.clone(), worktree_root: route.source_root.clone() };
    let source: SnapshotSource = source_refusal(context, &identity).map_err(|refusal: SourceRefusal| return StepRefusal::refused(format!("the main checkout cannot be the source of a clone: {}", refusal.reason)))?;
    if let Some(reason) = snapshot_blocker(&source) {
        return Err(StepRefusal::refused(format!("the main checkout cannot be the source of a clone: {reason}")));
    }
    if !source.trash_ignored {
        return Err(StepRefusal::refused(String::from(
            "the place the add sets the copied Git administration aside in, `.claude/cli-git-trash`, is not ignored by this repository's Git, so the new worktree would carry an untracked folder. Ways forward: add that path to `.gitignore` or `.git/info/exclude`, then run the add again",
        )));
    }
    let _plan_read: () = refuse_while_a_plan_names_the_clone(route.common_dir.as_path(), None, route.clone_dataset().as_str())
        .map_err(|stop| return StepRefusal::refused(format!("{stop:?}")))?;
    verify_the_slot_parent(route)?;
    let device: OpenedDevice = OpenedDevice::open().map_err(StepRefusal::refused)?;
    let checkout: ObjectIdentity = device.identity_of(route.checkout_dataset.as_str()).map_err(StepRefusal::refused)?;
    let pool: ObjectIdentity = device.identity_of(route.pool.as_str()).map_err(StepRefusal::refused)?;
    device.identity_of(route.container_dataset.as_str()).map_err(|reason| {
        return StepRefusal::refused(format!(
            "{reason}; the clone container {} the design places beside the checkout does not answer, so the route's layout is half-made. Ways forward: mount the container's dataset as setup does, then run the add again",
            route.container_dataset
        ));
    })?;
    admit_the_handler(route)?;
    let _ = plan;
    return Ok(Preflight { source, device, checkout_guid: checkout.guid, pool_guid: pool.guid });
}

/// Verify the folder the slot goes in, when it stands.
fn verify_the_slot_parent(route: &RouteFacts) -> Result<(), StepRefusal> {
    let parent: PathBuf = route.worktrees_parent();
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(parent.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(StepRefusal::refused(format!("the worktrees folder {} cannot be examined ({error})", parent.display()))),
    };
    if !metadata.is_dir() {
        return Err(StepRefusal::refused(format!("the worktrees folder {} is no folder", parent.display())));
    }
    // SAFETY: `geteuid` reads this process's own identity and has no preconditions.
    let ours: u32 = unsafe { libc::geteuid() };
    if metadata.uid() != ours {
        return Err(StepRefusal::refused(format!("the worktrees folder {} belongs to account {}, and this is account {ours}", parent.display(), metadata.uid())));
    }
    return Ok(());
}

/// Require a current handler readiness report for this boot, pool and checkout.
fn admit_the_handler(route: &RouteFacts) -> Result<(), StepRefusal> {
    let boot: String = std::fs::read_to_string("/proc/sys/kernel/random/boot_id").map_err(|error| return StepRefusal::refused(format!("this boot's identifier cannot be read ({error}), so the handler's readiness cannot be judged")))?;
    let answer: Option<ReadinessRecord> = ReadinessFile::live().read().map_err(|refusal| return StepRefusal::refused(refusal.message()))?;
    let Some(record) = answer else {
        return Err(StepRefusal::refused(format!(
            "no handler readiness report stands for this boot, so no attach is admitted ({}). Ways forward: start the boot unit's `boot-reconcile` and run the add again",
            ReadinessFile::live().record().display()
        )));
    };
    let checkout: String = route.source_root.display().to_string();
    let expectation: ReadinessExpectation<'_> = ReadinessExpectation { boot: boot.trim(), pool: route.pool.as_str(), checkout: checkout.as_str() };
    return admit_attach(&record, &expectation).map_err(|refusal: AdmissionRefusal| return StepRefusal::refused(refusal.message()));
}

/// Take the slot's claim, or refuse with the live owner's process id.
fn take_the_claim(home: &std::path::Path, slot: &str) -> Result<crate::single_owner_claim::OwnerClaim, StepRefusal> {
    return match crate::single_owner_claim::take_claim(home, claim_prefix(slot).as_os_str()) {
        Ok(crate::single_owner_claim::ClaimOutcome::Held(claim)) => Ok(claim),
        Ok(crate::single_owner_claim::ClaimOutcome::Busy(owner)) => Err(StepRefusal::refused(format!(
            "another command holds the claim on {slot} (process {}), so this add changed nothing. Ways forward: wait for that command to end, or stop it if it is not running",
            owner.pid
        ))),
        Err(error) => Err(StepRefusal::refused(format!("the claim on {slot} cannot be taken ({error:?}), so nothing was changed"))),
    };
}

/// The generation the claim this process holds took.
fn the_claims_generation(home: &std::path::Path, slot: &str) -> Result<u64, StepRefusal> {
    let listing: std::fs::ReadDir = std::fs::read_dir(home).map_err(|error| return StepRefusal::refused(format!("the record home {} cannot be listed ({error})", home.display())))?;
    let mut newest: u64 = 0;
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| return StepRefusal::refused(format!("the record home {} cannot be listed ({error})", home.display())))?;
        let Some(name) = entry.file_name().to_str().map(String::from) else { continue };
        if let Some(generation) = claim_generation_of(name.as_str(), slot) {
            newest = newest.max(generation);
        }
    }
    if newest == 0 {
        return Err(StepRefusal::refused(format!("no claim generation for {slot} stands after this command took one")));
    }
    return Ok(newest);
}

/// Read a standing record and prove it is this add's continuation.
fn resume_the_record(route: &RouteFacts, plan: &crate::worktree_add_plan::AddPlan, arguments: &[OsString], mut record: BindingRecord, generation: u64) -> Result<BindingRecord, StepRefusal> {
    if record.repository_path().as_deref() != Some(route.common_dir.as_path()) || record.slot_path().as_deref() != Some(route.destination.as_path()) {
        let shown: String = record.repository_path().map_or_else(|| return String::from("none"), |path| return path.display().to_string());
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
            "the record was written by an add given `{}` and this add was given `{}`; Ways forward: run the same add again, or remove the worktree and its records with `git worktree remove {}` first",
            record.request.encode(),
            request.encode(),
            route.slot
        )));
    }
    record.claim = generation;
    return Ok(record);
}

/// Write the record's first version before any mutation beyond the claim.
fn fresh_the_record(route: &RouteFacts, plan: &crate::worktree_add_plan::AddPlan, arguments: &[OsString], preflight: &Preflight, home: &std::fs::File, generation: u64) -> Result<BindingRecord, StepRefusal> {
    // A plan that names the clone comes from a removal's record design and can
    // stand only from step 14, so the read finds nothing today and refuses at
    // once when it does.
    let clone_dataset: String = route.clone_dataset();
    refuse_while_a_plan_names_the_clone(route.common_dir.as_path(), None, clone_dataset.as_str()).map_err(|stop| return StepRefusal::refused(format!("{stop:?}")))?;
    let names: AddNames = name_the_add(route, plan, arguments).map_err(StepRefusal::refused)?;
    clear_other_adds_records(home_folder(route.common_dir.as_path()).as_path(), route.slot.as_str(), &names.nonce).map_err(StepRefusal::refused)?;
    let repository_identity: FolderIdentity = FolderIdentity::of_folder(route.common_dir.as_path()).map_err(|error| return StepRefusal::refused(format!("the common Git directory cannot be identified ({error})")))?;
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
fn verify_the_slot_identity(steps: &Steps<'_>, record: &BindingRecord) -> Result<(), StepRefusal> {
    let Some(recorded) = record.slot_identity else {
        return Err(StepRefusal::refused(String::from("the record names no slot identity, so nothing proves the slot Git registered is this add's")));
    };
    let live: FolderIdentity = FolderIdentity::of_folder(steps.route.destination.as_path()).map_err(|error| return StepRefusal::refused(format!("the slot {} cannot be identified ({error})", steps.route.destination.display())))?;
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
fn finish_the_add(steps: &Steps<'_>, trash: Option<crate::worktree_trash::PendingTrash>, record: &mut BindingRecord) -> Result<(), StepRefusal> {
    if let Some(pending) = trash {
        let mut notices = std::io::stderr();
        if let Err(reason) = crate::worktree_trash::hand_over_to_the_service(steps.context, &pending, &mut notices) {
            let _written: std::io::Result<()> = writeln!(std::io::stderr(), "monochromatic-git-policy-cli: the trash of {} could not be handed to the service ({reason}); its record still names it", steps.route.destination.display());
        }
    }
    #[cfg(target_os = "linux")]
    {
        let environment: crate::package_repair::RepairEnvironment = crate::package_repair::RepairEnvironment::from_process();
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
