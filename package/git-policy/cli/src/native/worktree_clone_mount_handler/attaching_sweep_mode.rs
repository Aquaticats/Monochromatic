//! What:
//!  The clone-mount sweep:
//!  what the installed ZED event runs when it starts the program with no argument,
//!  and what `boot-reconcile` runs after its own checks.
//! Why:
//!  The sweep enumerates the pinned parent's clone datasets,
//!  reads each clone's durable mount intent and attaches only what that intent
//!  asks for
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety",
//!  "Requests,
//! results and ordering";
//!  it is plan step 8).
//!  Before any attach it reads this boot's start-probe record,
//!  the pool and the checkout,
//!  because a failed prerequisite authorizes no attachment.
//!  It reads `.parent` and the record of each slot it reaches and lists no
//!  name:
//!  one folder listing per run is what the accepted design's choice 9 does not
//!  take,
//!  and a staging file a killed writer left is the adoption's and plan step 18's
//!  `doctor`'s to remove
//!  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
//!  "Who removes a killed writer's staging file").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = run(); // the run's summary and one line per clone it reached
//! ```

use super::per_slot_root_claim::SlotClaimDirectory;
use super::sweep_checkout_mount::LiveCheckoutMounts;
use super::sweep_configuration::{SweepConfiguration, read_configuration};
use super::sweep_first_attach::LiveFirstAttach;
use super::sweep_one_clone::LiveStandingMounts;
use super::sweep_dataset_reader::ZfsDatasetReader;
use super::sweep_one_clone::SweepContext;
use super::sweep_preconditions::{BootFacts, PreparedBoot, complete, prepare, probe_record_path};
use super::sweep_run::{SweepRun, sweep};
use super::handler_install_paths::CONFIGURATION_PATH;
use crate::diagnostic_log::debug;
use crate::worktree_clone_mount::SLOT_IDENTITY_STORE_PATH;
use crate::worktree_clone_mount_protocol::ReceiptsFolder;
use std::path::PathBuf;

/// Module tag for diagnostics.
const TAG: &str = "attachingSweepMode";

/// What:
///  Where one run reads and writes,
///  with the installed paths by default.
/// Why:
///   The release path takes the installed configuration,
///  the store under `/var/lib` and the receipts under `/run`,
///  while a control hands in fixture paths and a scripted pool,
///  so the paths travel as one value and no mode hardcodes them twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepPlaces = { configuration: string; store: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SweepPlaces {
    /// The root-owned handler configuration.
    pub configuration: PathBuf,
    /// The store of recorded identities.
    pub store: PathBuf,
    /// The start probe's record,
    /// which every run that makes a raw request reads first.
    pub probe_record: PathBuf,
}

impl SweepPlaces {
    /// The installed places.
    pub fn installed() -> Self {
        return SweepPlaces { configuration: PathBuf::from(CONFIGURATION_PATH), store: PathBuf::from(SLOT_IDENTITY_STORE_PATH), probe_record: probe_record_path() };
    }

    /// Fixture places,
    /// which a control hands in so nothing on a machine is read or written.
    pub fn at(configuration: PathBuf, store: PathBuf, probe_record: PathBuf) -> Self {
        return SweepPlaces { configuration, store, probe_record };
    }
}

/// What:
///  Run one sweep from the installed places,
///  with this machine's live pool and the installed receipts and claims.
/// Why:
///  The ZED event and the boot unit both call this,
///  so the route's own reads and its own places are decided once.
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
/// or nothing at all when the sweep ran and some clone refused.
pub fn run() -> Result<String, String> {
    return run_at(&SweepPlaces::installed());
}

/// What:
///  Run one sweep from the places it is given.
/// Why:
///  The release path takes the installed places and a control takes fixtures,
///  and the order the checks and the sweep run in is the same for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = runAt(places);
/// ```
///
/// # Errors
/// Everything `run` refuses.
pub fn run_at(places: &SweepPlaces) -> Result<String, String> {
    let mounts: LiveCheckoutMounts = LiveCheckoutMounts;
    let configuration: SweepConfiguration = read_configuration(places.configuration.as_path(), &mounts).map_err(|refusal| return refusal.message())?;
    let prepared: PreparedBoot = prepare(&configuration, places.probe_record.as_path(), &mounts).map_err(|refusal| return refusal.message())?;
    let reader: ZfsDatasetReader = ZfsDatasetReader::open().map_err(|error| return format!("the handler could not open the ZFS device: {error}"))?;
    let facts: BootFacts = complete(prepared, &reader, configuration.parent_dataset.as_str()).map_err(|refusal| return refusal.message())?;
    debug(TAG, format!("this boot is {} and the pool {} is ready", facts.boot, facts.pool_name).as_str());
    let receipts: ReceiptsFolder = ReceiptsFolder::live();
    let claims: SlotClaimDirectory = SlotClaimDirectory::live();
    let clock = || return crate::iso_time::now_iso();
    let attach: super::sweep_attach_step::RealAttachStep = super::sweep_attach_step::RealAttachStep::default();
    let first_attach: LiveFirstAttach = LiveFirstAttach::live();
    let standing: LiveStandingMounts = LiveStandingMounts::read();
    let context: SweepContext<'_> = SweepContext {
        parent_dataset: configuration.parent_dataset.as_str(),
        checkout: configuration.checkout.as_path(),
        checkout_dataset: configuration.checkout_dataset.as_str(),
        store: places.store.as_path(),
        claims: &claims,
        receipts: &receipts,
        boot: facts.boot.as_str(),
        time: &clock,
        reader: &reader,
        attach: &attach,
        first_attach: &first_attach,
        standing: &standing,
    };
    let run: SweepRun = sweep(&context);
    return Ok(run.report());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "attaching_sweep_mode_tests.rs"]
mod tests;
