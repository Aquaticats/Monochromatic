//! What:
//!  Controls of what a run checks before any attach:
//!  the probe's record for this boot,
//!  the pool's name and state,
//!  and whether the checkout walks.
//! Why:
//!  A failed prerequisite authorizes no attachment,
//!  and each failure sends a person somewhere else,
//!  so every answer of the record and every state is proved here without a
//!  device:
//!  a record is text and a checkout is a fixture folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(probeStanding(record, boot, path).kind).toBe('passed');
//! ```

use super::{BootFacts, ProbeRefusal, ProbeStanding, boot_identifier, checkout_state, pool_name_of, pool_state, prepare, probe_standing};
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ListedChild};
use crate::worktree_clone_mount_protocol::ReadyState;
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use std::path::{Path, PathBuf};

/// The pinned parent the controls name.
const PARENT: &str = "zpcachyos/nosnap/clones";

/// The checkout the controls walk.
const CHECKOUT: &str = "/fixture/checkout";

/// One record a probe that passed writes.
fn passed_record(boot: &str) -> String {
    return format!("state finished\nboot {boot}\ntime 2026-11-21T20:00:00-05:00\nverdict passed\nverdict-step none\n");
}

/// One record a probe that failed writes.
fn failed_record(boot: &str, step: &str) -> String {
    return format!("state finished\nboot {boot}\nverdict failed\nverdict-step {step}\n");
}

/// One fixture folder of this control's own,
/// removed however the control ends.
struct Fixture {
    /// The folder.
    path: PathBuf,
}

impl Fixture {
    /// A fresh folder under this process's own temporary root.
    fn new(name: &str) -> Self {
        let path: PathBuf = std::env::temp_dir().join(format!("sweep-preconditions-{name}-{}", std::process::id()));
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(path.as_path());
        std::fs::create_dir_all(path.as_path()).expect("the fixture folder is made");
        return Fixture { path };
    }

    /// The folder,
    /// with a `.claude/worktrees` chain inside it when asked,
    /// and the record file when one is given.
    fn with_checkout(self, record: Option<&str>) -> Self {
        let parent: PathBuf = self.path.join("checkout").join(".claude").join("worktrees");
        std::fs::create_dir_all(parent.as_path()).expect("the checkout chain is made");
        if let Some(text) = record {
            std::fs::write(self.path.join("probe-record"), text).expect("the record is written");
        }
        return self;
    }

    /// The fixture checkout's path.
    fn checkout(&self) -> PathBuf {
        return self.path.join("checkout");
    }

    /// The record's path.
    fn record(&self) -> PathBuf {
        return self.path.join("probe-record");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(self.path.as_path());
    }
}

/// The statistics a pool's answer carries,
/// which this control reads no field of.
fn stats() -> monochromatic_zfs_request_to_openzfs_kernel_device::DmuObjsetStats {
    return monochromatic_zfs_request_to_openzfs_kernel_device::DmuObjsetStats {
        dds_num_clones: 0,
        dds_creation_txg: 1,
        dds_guid: 1,
        dds_type: 2,
        dds_is_snapshot: 0,
        dds_inconsistent: 0,
        dds_redacted: 0,
        dds_origin: [0; 256],
        dds_flags: 0,
    };
}

/// The identity a pool's answer for the pinned parent carries.
fn identity_of_one_dataset() -> DatasetIdentity {
    return DatasetIdentity { stats: stats(), properties: monochromatic_zfs_request_to_openzfs_kernel_device::NvList::new() };
}

/// A pool that answers one dataset and nothing else.
struct OneDataset {
    /// The dataset it holds.
    held: Option<String>,
    /// The words it refuses with,
    /// for a pool that is not ready.
    refusal: String,
}

impl DatasetReader for OneDataset {
    fn identity(&self, dataset: &str) -> Result<DatasetIdentity, String> {
        return match &self.held {
            Some(held) if held == dataset => Ok(identity_of_one_dataset()),
            _ => Err(self.refusal.clone()),
        };
    }

    fn next_child(&self, _parent: &str, _cursor: u64) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}

/// This boot's identifier is the kernel's own,
/// and it is not empty.
#[test]
fn this_boots_identifier_is_the_kernels_own() {
    let boot: String = boot_identifier().expect("this host's boot identifier reads");
    assert!(!boot.is_empty());
    assert!(boot.len() <= 36, "the record's bound holds this boot: {boot}");
}

/// What:
///  A record that passed for this boot enables the managed route,
///  and every other verdict or boot blocks it.
/// Why:
///  An arm that could not run keeps the route blocked,
///  and a record another boot wrote says nothing about this kernel.
#[test]
fn a_record_enables_the_route_only_when_it_passed_for_this_boot() {
    let boot: String = boot_identifier().expect("this host's boot identifier reads");
    let path: &Path = Path::new("/fixture/probe-record");
    assert_eq!(probe_standing(passed_record(boot.as_str()).as_str(), boot.as_str(), path), Ok(ProbeStanding::Passed));
    let blocked: ProbeRefusal = probe_standing(failed_record(boot.as_str(), "attach-exclusive-create").as_str(), boot.as_str(), path).expect_err("a failed arm blocks the route");
    assert!(matches!(blocked, ProbeRefusal::Blocked { .. }), "{blocked:?}");
    assert!(blocked.message().contains("attach-exclusive-create"), "the refusal names the step: {}", blocked.message());
    assert!(blocked.message().contains("Nothing was enumerated"), "the refusal says nothing acted: {}", blocked.message());
    let skipped: ProbeRefusal = probe_standing(format!("boot {boot}\nverdict skipped\nverdict-step device\n").as_str(), boot.as_str(), path).expect_err("a skipped probe blocks the route");
    assert!(matches!(skipped, ProbeRefusal::Blocked { .. }), "{skipped:?}");
    let another: ProbeRefusal = probe_standing(passed_record("00000000-0000-4000-8000-000000000000").as_str(), boot.as_str(), path).expect_err("another boot's record is stale");
    assert!(matches!(another, ProbeRefusal::AnotherBoot { .. }), "{another:?}");
    assert!(another.message().contains("is another boot's"), "the refusal says the record is another boot's: {}", another.message());
    let no_verdict: ProbeRefusal = probe_standing(format!("boot {boot}\n").as_str(), boot.as_str(), path).expect_err("a record with no verdict enables nothing");
    assert!(matches!(no_verdict, ProbeRefusal::Unreadable { .. }), "{no_verdict:?}");
}

/// What:
///  A boot with no probe record attaches nothing,
///  and the refusal names the record and the modes a person runs.
/// Why:
///  The record is the only evidence that this boot's kernel passed the arms,
///  so its absence is a boot that attaches nothing rather than a boot that
///  guesses.
#[test]
fn a_boot_with_no_record_attaches_nothing() {
    let fixture: Fixture = Fixture::new("no-record").with_checkout(None);
    let configuration: SweepConfiguration = SweepConfiguration { parent_dataset: String::from(PARENT), checkout: fixture.checkout() };
    let refusal: ProbeRefusal = prepare(&configuration, fixture.record().as_path()).expect_err("a missing record refuses the run");
    assert!(matches!(refusal, ProbeRefusal::Missing { .. }), "{refusal:?}");
    assert!(refusal.message().contains("start-probe"), "the refusal names the mode that writes it: {}", refusal.message());
    assert!(refusal.message().contains("every clone's mount intent stands untouched"), "the refusal says every intent stands: {}", refusal.message());
}

/// The pool's name is the pinned parent's first component,
/// which is what the readiness record carries.
#[test]
fn the_pools_name_is_the_parents_first_component() {
    assert_eq!(pool_name_of(PARENT), "zpcachyos");
    assert_eq!(pool_name_of("pool"), "pool");
}

/// A pool whose parent answers is ready,
/// one whose parent is gone is not ready,
/// and one whose answer this route cannot read is unknown.
#[test]
fn the_pool_is_ready_only_when_the_parent_answers() {
    let answering: OneDataset = OneDataset { held: Some(String::from(PARENT)), refusal: String::from("no such pool") };
    assert_eq!(pool_state(&answering, PARENT), ReadyState::Ready);
    let gone: OneDataset = OneDataset { held: None, refusal: String::from("dataset does not exist") };
    assert_eq!(pool_state(&gone, PARENT), ReadyState::NotReady);
    let unreadable: OneDataset = OneDataset { held: None, refusal: String::from("the device answered EIO") };
    assert_eq!(pool_state(&unreadable, PARENT), ReadyState::Unknown);
}

/// A checkout whose `.claude/worktrees` folder walks is ready,
/// and one that is not there is not ready.
#[test]
fn the_checkout_is_ready_only_when_the_walk_reaches_it() {
    let fixture: Fixture = Fixture::new("checkout").with_checkout(None);
    let (state, _said): (ReadyState, String) = checkout_state(fixture.checkout().as_path());
    assert_eq!(state, ReadyState::Ready);
    let absent: PathBuf = fixture.path.join("nothing-here");
    let (missing, said): (ReadyState, String) = checkout_state(absent.as_path());
    assert_eq!(missing, ReadyState::NotReady, "{said}");
    assert!(said.contains("nothing-here"), "the refusal names the checkout: {said}");
}

/// A run whose prerequisites all hold is prepared,
/// and one whose checkout does not walk is refused before any device is opened.
#[test]
fn prepare_reads_the_record_and_the_checkout_before_any_device() {
    let boot: String = boot_identifier().expect("this host's boot identifier reads");
    let fixture: Fixture = Fixture::new("prepared").with_checkout(Some(passed_record(boot.as_str()).as_str()));
    let configuration: SweepConfiguration = SweepConfiguration { parent_dataset: String::from(PARENT), checkout: fixture.checkout() };
    let prepared = prepare(&configuration, fixture.record().as_path()).expect("every check passed");
    assert_eq!(prepared.boot, boot);
    assert_eq!(prepared.checkout_state, ReadyState::Ready);
    assert_eq!(prepared.standing, ProbeStanding::Passed);
    let absent: SweepConfiguration = SweepConfiguration { parent_dataset: String::from(PARENT), checkout: fixture.path.join("nowhere") };
    let refused = prepare(&absent, fixture.record().as_path()).expect("the record passes, so the checkout is what the admission refuses");
    assert_eq!(refused.checkout_state, ReadyState::NotReady);
    let facts: BootFacts = BootFacts { boot: String::from("boot"), pool_name: String::from("zpcachyos"), pool_state: ReadyState::Ready, checkout: PathBuf::from(CHECKOUT), checkout_state: ReadyState::NotReady };
    let admission: ProbeRefusal = facts.admit().expect_err("a checkout that is not ready admits nothing");
    assert!(matches!(admission, ProbeRefusal::CheckoutNotReady { .. }), "{admission:?}");
    let unknown: BootFacts = BootFacts { checkout_state: ReadyState::Ready, pool_state: ReadyState::Unknown, ..facts };
    assert!(matches!(unknown.admit(), Err(ProbeRefusal::PoolNotReady { .. })), "an unknown pool admits nothing");
}
