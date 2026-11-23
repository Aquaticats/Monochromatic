//! What:
//!  The boot unit's reconciliation:
//!  the start-time compatibility gate the dispatch ran,
//!  this boot's start-probe record,
//!  the keys and the checkout,
//!  the durable intent of every clone,
//!  and this boot's readiness.
//! Why:
//!  ZED alone cannot order a sweep after the checkout is ready,
//!  so a systemd unit runs this mode at boot
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit";
//!  the build order is `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 8).
//!  The mode's order is the design's:
//!  its own checks,
//!  then the sweep,
//!  then this boot's readiness record,
//!  which is what step 19's unit makes a boot wait for.
//!  A readiness record published before the sweep would let a caller read
//!  readiness for a boot whose clones were never reconciled,
//!  and one published instead of a failed prerequisite would promise a route the
//!  boot does not have:
//!  a failed prerequisite therefore publishes nothing and authorizes no
//!  attachment.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = run(); // the readiness line, then the sweep's report
//! ```

use super::per_slot_root_claim::SlotClaimDirectory;
use super::sweep_checkout_mount::LiveCheckoutMounts;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use super::sweep_configuration::{SweepConfiguration, read_configuration};
use super::sweep_first_attach::LiveFirstAttach;
use super::sweep_one_clone::LiveStandingMounts;
use super::sweep_dataset_reader::ZfsDatasetReader;
use super::sweep_one_clone::SweepContext;
use super::sweep_preconditions::{BootFacts, PreparedBoot, complete, prepare, probe_record_path};
use super::sweep_run::{SweepRun, sweep};
use crate::diagnostic_log::{debug, warn};
use crate::worktree_clone_mount::SLOT_IDENTITY_STORE_PATH;
use crate::worktree_clone_mount_protocol::{ReadinessFile, ReadinessRecord, ReceiptsFolder, SweepState};
use crate::worktree_clone_mount_handler::handler_install_paths::CONFIGURATION_PATH;
use std::path::PathBuf;

/// Module tag for diagnostics.
const TAG: &str = "bootReconcileMode";

/// What:
///  Where one reconciliation reads and writes,
///  with the installed paths by default.
/// Why:
///   The unit runs the installed configuration and the store under `/var/lib`,
///  while a control hands in fixtures,
///  so the places are one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReconcilePlaces = { configuration: string; store: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReconcilePlaces {
    /// The root-owned handler configuration.
    pub configuration: PathBuf,
    /// The store of recorded identities.
    pub store: PathBuf,
    /// The start probe's record,
    /// which every run that makes a raw request reads first.
    pub probe_record: PathBuf,
}

impl ReconcilePlaces {
    /// The installed places.
    pub fn installed() -> Self {
        return ReconcilePlaces { configuration: PathBuf::from(CONFIGURATION_PATH), store: PathBuf::from(SLOT_IDENTITY_STORE_PATH), probe_record: probe_record_path() };
    }

    /// Fixture places,
    /// which a control hands in so nothing on a machine is read or written.
    pub fn at(configuration: PathBuf, store: PathBuf, probe_record: PathBuf) -> Self {
        return ReconcilePlaces { configuration, store, probe_record };
    }
}

/// What:
///  Run the boot reconciliation from the installed places.
/// Why:
///  Step 19's unit runs exactly this,
///  so the mode's checks and its sweep and its readiness are one implementation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = run();
/// ```
///
/// # Errors
/// The configuration,
/// a failed prerequisite,
/// the device,
/// or a readiness record the writer would not read back.
pub fn run() -> Result<String, String> {
    let places: ReconcilePlaces = ReconcilePlaces::installed();
    let readiness: ReadinessFile = ReadinessFile::live();
    return run_at(&places, &readiness);
}

/// What:
///  Run the reconciliation against the given places and readiness file.
/// Why:
///  The release path takes the installed places and a control takes fixtures,
///  and the order the checks and the sweep and the two readiness writes run in is
///  the same for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = runAt(places, readiness);
/// ```
///
/// # Errors
/// Everything `run` refuses.
pub fn run_at(places: &ReconcilePlaces, readiness: &ReadinessFile) -> Result<String, String> {
    let mounts: LiveCheckoutMounts = LiveCheckoutMounts;
    let table: LiveMountedDatasets = LiveMountedDatasets;
    let configuration: SweepConfiguration = read_configuration(places.configuration.as_path(), &table).map_err(|refusal| return refusal.message())?;
    let prepared: PreparedBoot = prepare(&configuration, places.probe_record.as_path(), &mounts).map_err(|refusal| return refusal.message())?;
    let reader: ZfsDatasetReader = ZfsDatasetReader::open().map_err(|error| return format!("the handler could not open the ZFS device, so this boot's clones are not reconciled: {error}"))?;
    let facts: BootFacts = complete(prepared, &reader, configuration.parent_dataset.as_str()).map_err(|refusal| return refusal.message())?;
    let boot: String = facts.boot.clone();
    readiness.publish(&record_of(&facts, SweepState::Running)).map_err(|refusal| return refusal.message())?;
    let receipts: ReceiptsFolder = ReceiptsFolder::live();
    let claims: SlotClaimDirectory = SlotClaimDirectory::live();
    let clock = || return crate::iso_time::now_iso();
    let attach: super::sweep_attach_step::RealAttachStep = super::sweep_attach_step::RealAttachStep::default();
    let first_attach: LiveFirstAttach = LiveFirstAttach::live();
    let standing: LiveStandingMounts = LiveStandingMounts;
    let context: SweepContext<'_> = SweepContext {
        parent_dataset: configuration.parent_dataset.as_str(),
        checkout: configuration.checkout.as_path(),
        checkout_dataset: configuration.checkout_dataset.as_str(),
        store: places.store.as_path(),
        claims: &claims,
        receipts: &receipts,
        boot: boot.as_str(),
        time: &clock,
        reader: &reader,
        attach: &attach,
        first_attach: &first_attach,
        standing: &standing,
    };
    let run: SweepRun = sweep(&context);
    let state: SweepState = if run.completed() && run.failed() == 0 { SweepState::Finished } else { SweepState::Failed };
    if state == SweepState::Failed {
        warn(TAG, run.summary().as_str());
    }
    readiness.publish(&record_of(&facts, state)).map_err(|refusal| return refusal.message())?;
    debug(TAG, "this boot's readiness is published after the sweep");
    return Ok(format!("{}\n{}", readiness_line(&facts, state), run.report()));
}

/// What:
///  This boot's readiness record,
///  built from the facts the checks read.
/// Why:
///  The record's fields are exactly those facts,
///  and a field longer than the record's bound is refused before the writer is
///  asked,
///  so a record the reader would refuse is never published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = recordOf(facts, 'finished', 'none');
/// ```
fn record_of(facts: &BootFacts, sweep_state: SweepState) -> ReadinessRecord {
    return ReadinessRecord {
        boot: facts.boot.clone(),
        pool_name: facts.pool_name.clone(),
        pool_state: facts.pool_state,
        checkout_path: facts.checkout.display().to_string(),
        checkout_state: facts.checkout_state,
        sweep_nonce: None,
        sweep_state,
        time: crate::iso_time::now_iso(),
    };
}

/// The one line the mode says about the readiness it published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = `readiness published for boot ${boot} with the sweep ${state}`;
/// ```
fn readiness_line(facts: &BootFacts, state: SweepState) -> String {
    return format!(
        "this boot's readiness is published: boot {}, pool {} {}, checkout {} {}, sweep {}. Managed attaches are admitted only when both the pool and the checkout are ready for this boot.",
        facts.boot,
        facts.pool_state.word(),
        facts.pool_name,
        facts.checkout_state.word(),
        facts.checkout.display(),
        state.word()
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "boot_reconcile_mode_tests.rs"]
mod tests;
