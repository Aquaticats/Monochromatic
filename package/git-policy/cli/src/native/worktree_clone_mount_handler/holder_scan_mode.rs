//! What:
//!  The `holder-scan` mode:
//!  read one dataset's GUID,
//!  run the all-namespace holder scan of it under a nonce of its own,
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
//!  The project decided on 2026-11-23 that the wrapper asks root for a fresh
//!  scan before each destroy and names the holders itself,
//!  so every scan answers under a nonce of its own:
//!  a nonce the caller names,
//!  which the wrapper's request will carry,
//!  or a fresh one,
//!  and never the clone's request nonce,
//!  which two scans of one removal would share
//!  (`doc/decision/cli-git-zfs-holder-scan-names-every-holder-it-can-read.md`).
//!  A guest arm and a later step need one way in that needs no library caller,
//!  so the mode is that door,
//!  and it is the same door the other modes are:
//!  one word,
//!  one required option naming the dataset and one optional nonce,
//!  and no path,
//!  dataset or flag taken from an intent.
//!  The mode only reads:
//!  it mounts,
//!  unmounts,
//!  destroys and records nothing,
//!  so running it before a decision changes no state the decision reads.
//!  The result is written where every other mode's answer is written and not on
//!  the program's standard output:
//!  step 17 owns how a finding reaches the wrapper,
//!  and this mode is the reading,
//!  not the carrying.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = run({ dataset: '…/clones/one', nonce: '…' }); // the result's text and the counts
//! ```

use super::handler_mode_arguments::HolderScanInvocation;
use super::sweep_dataset_reader::{DatasetReader, ZfsDatasetReader};
use crate::clone_holder_scan::holder_scan::{HolderScan, HolderScanRequest, scan_holders};
use crate::foundation::iso_time::now_iso;
use crate::mount_namespace_scan::mount_namespace_tasks::RootIdentity;
use crate::mount_namespace_scan::namespace_target;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::Path;

/// The `/proc` root a live scan reads.
pub const PROC_ROOT: &str = "/proc";

/// The task whose namespace and root are the handler's own.
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
///  The nonce this scan answers under,
///  and where it came from in the words the summary repeats.
/// Why:
///  Every destroy request asks for its own fresh answer,
///  so a result is bound to the scan and never to the removal's request:
///  the caller's own nonce when it named one,
///  and a fresh one otherwise.
fn nonce_of(invocation: &HolderScanInvocation) -> Result<(RequestNonce, &'static str), String> {
    if let Some(named) = &invocation.nonce {
        return Ok((named.clone(), "named by the caller"));
    }
    let fresh: RequestNonce = RequestNonce::fresh().map_err(|refusal| return refusal.message())?;
    return Ok((fresh, "fresh, because the caller named none"));
}

/// What:
///  The line that counts what one scan found.
/// Why:
///  The result's own text says which namespaces hold the dataset,
///  and this line says how much was read,
///  which is what a person and a guest arm read first.
fn summary_of(scan: &HolderScan, dataset: &str, nonce_source: &str) -> String {
    return format!(
        "the holder scan of {dataset} (GUID {}, nonce {} {nonce_source}) read {} mount namespaces: {} mount(s) of it in the handler's own namespace, {} holder namespace(s) holding {} mount(s) and {} incomplete read(s); it mounted, unmounted and recorded nothing, and it sets nothing aside.",
        scan.dataset.file_name(),
        scan.nonce.as_str(),
        scan.namespaces_scanned,
        scan.own_mounts.len(),
        scan.holder_namespaces(),
        scan.holder_mounts(),
        scan.incomplete_reads(),
    );
}

/// What:
///  The reader's own namespace and root,
///  as the kernel names them.
/// Why:
///  The namespace says which rows are the reader's own,
///  and the root is the one a namespace made after the reader's inherits,
///  so the scan reads each namespace's table through a task at its own root
///  (`crate::mount_namespace_scan::mount_namespace_tasks::NamespaceRoot`).
fn readers_place(self_task: &Path) -> (Option<String>, Option<RootIdentity>) {
    let namespace: Option<String> = namespace_target(self_task.join("ns").join("mnt").as_path());
    let root: Option<RootIdentity> = RootIdentity::of(self_task.join("root").as_path());
    return (namespace, root);
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
/// const answer = run({ dataset, nonce });
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
/// const answer = runWith({ dataset, nonce }, scriptedPool, fixtureProc, '/proc/self');
/// ```
pub fn run_with(invocation: &HolderScanInvocation, reader: &dyn DatasetReader, proc_root: &Path, self_task: &Path) -> Result<String, String> {
    let identity = reader.identity(invocation.dataset.as_str()).map_err(|failure| {
        return format!(
            "the holder scan of `{}` could not read the dataset ({failure}). Nothing was scanned. Ways forward: name a dataset this machine holds, and check that the pool is imported.",
            invocation.dataset
        );
    })?;
    let dataset: DatasetGuid = guid_of(&identity, invocation.dataset.as_str())?;
    let (own_namespace, readers_root): (Option<String>, Option<RootIdentity>) = readers_place(self_task);
    let (nonce, nonce_source): (RequestNonce, &str) = nonce_of(invocation)?;
    let scan: HolderScan = scan_holders(&HolderScanRequest {
        proc_root,
        source: invocation.dataset.as_str(),
        own_namespace: own_namespace.as_deref(),
        readers_root,
        dataset,
        nonce: &nonce,
        time: now_iso(),
    });
    return Ok(answer_of(&scan, invocation.dataset.as_str(), nonce_source));
}

/// What:
///  The result's own text followed by the line that counts what it found.
/// Why:
///  A reader that parses the result reads its lines,
///  and a person reads the last line;
///  both come from one value,
///  so the counts and the findings cannot disagree.
fn answer_of(scan: &HolderScan, dataset: &str, nonce_source: &str) -> String {
    return format!("{}\n{}", scan.text().trim_end(), summary_of(scan, dataset, nonce_source));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_scan_mode_tests.rs"]
mod tests;
