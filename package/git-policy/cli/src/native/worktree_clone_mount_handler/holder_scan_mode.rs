//! What:
//!  The `holder-scan` mode:
//!  read one dataset's GUID and durable intent,
//!  run the all-namespace holder scan of it,
//!  and answer the result's own text and one line that counts what it found.
//! Why:
//!  Plan step 9 builds the scan and its result,
//!  step 16 consumes the result before each snapshot-destroy request and
//!  before the head destroy,
//!  and step 18 reports its holders through `doctor`
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 9;
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces").
//!  A guest arm and a later step need one way in that needs no library caller,
//!  so the mode is that door,
//!  and it is the same door the other modes are:
//!  one word,
//!  two options that name the dataset and the checkout the slot hangs below,
//!  and no path,
//!  dataset or flag taken from an intent.
//!  The mode only reads:
//!  it mounts,
//!  unmounts,
//!  destroys and records nothing,
//!  so running it before a decision changes no state the decision reads.
//!  The nonce is the clone's own request's when it carries one,
//!  so a later reader can match this answer with the request that asked for it,
//!  and a fresh one otherwise,
//!  so even a mode invoked by hand answers under a value no earlier result
//!  carries.
//!  The result is written where every other mode's answer is written and not on
//!  the program's standard output:
//!  step 17 owns how a finding reaches the wrapper,
//!  and this mode is the reading,
//!  not the carrying.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = run({ dataset: '…/clones/one', checkout: '/home/user/Monochromatic' }); // the result's text and the counts
//! ```

use super::handler_mode_arguments::HolderScanInvocation;
use super::sweep_dataset_facts::facts_of;
use super::sweep_dataset_reader::{DatasetReader, ZfsDatasetReader};
use crate::clone_holder_scan::holder_scan::{HolderScan, HolderScanRequest, scan_holders};
use crate::clone_holder_scan::standing_unmount::StandingUnmount;
use crate::foundation::iso_time::now_iso;
use crate::mount_namespace_scan::mount_namespace_of;
use crate::worktree_clone_mount::{WORKTREES_PARENT_NAMES, slot_name_refusal};
use crate::worktree_clone_mount_protocol::clone_mount_intent::{IntentReading, MountIntent, read_intent};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::{Path, PathBuf};

/// The `/proc` root a live scan reads.
pub const PROC_ROOT: &str = "/proc";

/// The task whose namespace is the handler's own.
pub const SELF_TASK: &str = "/proc/self";

/// What:
///  The GUID one dataset's identity reply carries,
///  or the refusal that says why it is no dataset this route can name.
/// Why:
///   The result is bound to the GUID,
///  so a zero GUID or one that does not spell canonically stops the scan before
///  it answers anything a reader would have to compare.
fn guid_of(identity: &monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity, dataset: &str) -> Result<DatasetGuid, String> {
    if identity.stats.dds_guid == 0 {
        return Err(format!(
            "`{dataset}` answered GUID 0, and no dataset this route names has none. Nothing was scanned. Ways forward: name the clone whose GUID `zfs get -H -o value guid {dataset}` prints."
        ));
    }
    return DatasetGuid::parse(identity.stats.dds_guid.to_string().as_str()).map_err(|refusal| {
        let reason: String = refusal.message();
        return format!(
            "`{dataset}` answered a GUID this route cannot spell as one number ({reason}), so no result can be bound to it. Nothing was scanned. Ways forward: read the GUID again with `zfs get -H -o value guid {dataset}`."
        );
    });
}

/// What:
///  The path the clone's managed attach stands at,
///  or nothing when its name gives no slot.
/// Why:
///  The exclusion a standing `unmount` intent makes compares a mount's point
///  with this path,
///  and a name that is no slot name means the dataset is not one this route
///  attaches,
///  so nothing is set aside and every mount stays a holder.
fn slot_path_of(invocation: &HolderScanInvocation) -> Option<PathBuf> {
    let slot: &str = invocation.dataset.rsplit('/').next()?;
    if slot_name_refusal(slot).is_some() {
        return None;
    }
    return Some(invocation.checkout.join(WORKTREES_PARENT_NAMES[0]).join(WORKTREES_PARENT_NAMES[1]).join(slot));
}

/// What:
///  Which nonce the answer carries and where it came from,
///  in the words the summary repeats.
/// Why:
///  A result answers one request,
///  and a person reading the mode's line has to know which request that was.
fn nonce_of(intent: &IntentReading) -> Result<(RequestNonce, &'static str), String> {
    if let IntentReading::Request(request) = intent {
        return Ok((request.nonce.clone(), "from the clone's own request"));
    }
    let fresh: RequestNonce = RequestNonce::fresh().map_err(|refusal| return refusal.message())?;
    return Ok((fresh, "fresh, because the clone carries no request this build reads"));
}

/// What:
///  What the exclusion did,
///  in one sentence of the summary.
/// Why:
///  A reader that sees no holder has to be able to tell a clone whose managed
///  attach was set aside from one that carried no intent at all.
fn exclusion_sentence(intent: &IntentReading, standing: Option<&StandingUnmount>, slot: Option<&Path>) -> String {
    if let Some(standing) = standing {
        return format!("the clone's standing unmount intent set aside any managed attach at {}", standing.slot().display());
    }
    return match (intent, slot) {
        (IntentReading::Refused(refusal), _) => format!("nothing was set aside: the clone's intent could not be read ({})", refusal.message()),
        (IntentReading::Request(request), _) => format!("nothing was set aside: the clone asks for `{}` rather than an unmount", request.operation.word()),
        (IntentReading::Inherited { from }, _) => format!("nothing was set aside: the clone's intent is inherited from {from} and authorizes nothing"),
        (IntentReading::Absent, None) => String::from("nothing was set aside: the clone carries no intent and its name gives no slot"),
        (IntentReading::Absent, Some(_slot)) => String::from("nothing was set aside: the clone carries no intent"),
    };
}

/// What:
///  The line that counts what one scan found.
/// Why:
///  The result's own text says which namespaces hold the dataset,
///  and this line says how much was read and what was set aside,
///  which is what a person and a guest arm read first.
fn summary_of(scan: &HolderScan, dataset: &str, nonce_source: &str, exclusion: &str) -> String {
    return format!(
        "the holder scan of {dataset} (GUID {}, nonce {} {nonce_source}) read {} mount namespaces: {} mount(s) of it in the handler's own namespace, {} holder namespace(s) holding {} mount(s), {} managed attach(es) set aside and {} incomplete read(s); it mounted, unmounted and recorded nothing. {exclusion}",
        scan.dataset.file_name(),
        scan.nonce.as_str(),
        scan.namespaces_scanned,
        scan.own_mounts.len(),
        scan.holder_namespaces(),
        scan.holder_mounts(),
        scan.excluded.iter().map(|holder| return holder.mount_count()).sum::<usize>(),
        scan.incomplete_reads(),
    );
}

/// What:
///  Run the mode against this machine.
/// Why:
///  The release path opens the pool's device once,
///  and every other input is this machine's.
///
/// # Errors
/// The device could not be opened,
/// or the scan could not be bound to a GUID and a nonce.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = run({ dataset, checkout });
/// ```
pub fn run(invocation: &HolderScanInvocation) -> Result<String, String> {
    let reader: ZfsDatasetReader = ZfsDatasetReader::open().map_err(|error| {
        return format!(
            "the holder scan of `{}` could not open the ZFS device ({error}), so no dataset was read. Nothing was scanned. Ways forward: run the mode where OpenZFS is loaded and its device is present, as the handler's units do.",
            invocation.dataset
        );
    })?;
    return run_with(invocation, &reader, Path::new(PROC_ROOT), Path::new(SELF_TASK));
}

/// What:
///  Run the mode against a pool that answers from a script and a `/proc` root
///  the caller names.
/// Why:
///  A control may not open the pool's device or read another account's
///  processes,
///  so the scripted pool and the fixture tree are the mode's own seams.
///
/// # Errors
/// The dataset's identity could not be read,
/// or no GUID or nonce could be bound to the result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = runWith({ dataset, checkout }, scriptedPool, fixtureProc, '/proc/self');
/// ```
pub fn run_with(invocation: &HolderScanInvocation, reader: &dyn DatasetReader, proc_root: &Path, self_task: &Path) -> Result<String, String> {
    if !invocation.checkout.is_absolute() {
        return Err(format!(
            "`{}` is not an absolute path, and the checkout a slot hangs below is one. Nothing was scanned. Ways forward: give `--checkout` the absolute path the wrapper works from, as `--checkout /home/user/Monochromatic`.",
            invocation.checkout.display()
        ));
    }
    let identity = reader.identity(invocation.dataset.as_str()).map_err(|failure| {
        return format!(
            "the holder scan of `{}` could not read the dataset ({failure}). Nothing was scanned. Ways forward: name a dataset this machine holds, and check that the pool is imported.",
            invocation.dataset
        );
    })?;
    let dataset: DatasetGuid = guid_of(&identity, invocation.dataset.as_str())?;
    let facts = facts_of(invocation.dataset.as_str(), &identity);
    let slot: Option<PathBuf> = slot_path_of(invocation);
    let intent: IntentReading = read_intent(Some(&facts.properties), invocation.dataset.as_str());
    let request: Option<&MountIntent> = match &intent {
        IntentReading::Request(request) => Some(request),
        IntentReading::Absent | IntentReading::Inherited { .. } | IntentReading::Refused(_) => None,
    };
    let standing: Option<StandingUnmount> = StandingUnmount::of(request, slot.as_deref());
    let (nonce, nonce_source): (RequestNonce, &str) = nonce_of(&intent)?;
    let own_namespace: Option<String> = mount_namespace_of(self_task);
    let scan: HolderScan = scan_holders(&HolderScanRequest {
        proc_root,
        source: invocation.dataset.as_str(),
        own_namespace: own_namespace.as_deref(),
        standing_unmount: standing.as_ref(),
        dataset,
        nonce: &nonce,
        time: now_iso(),
    });
    return Ok(answer_of(&scan, invocation.dataset.as_str(), nonce_source, exclusion_sentence(&intent, standing.as_ref(), slot.as_deref()).as_str()));
}

/// What:
///  The result's own text followed by the line that counts what it found.
/// Why:
///  A reader that parses the result reads its lines,
///  and a person reads the last line;
///  both come from one value,
///  so the counts and the findings cannot disagree.
fn answer_of(scan: &HolderScan, dataset: &str, nonce_source: &str, exclusion: &str) -> String {
    return format!("{}\n{}", scan.text().trim_end(), summary_of(scan, dataset, nonce_source, exclusion));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_scan_mode_tests.rs"]
mod tests;
