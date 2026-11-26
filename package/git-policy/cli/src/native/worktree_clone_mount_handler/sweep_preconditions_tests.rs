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

use super::{
    BootFacts,
    ProbeRefusal,
    ProbeStanding,
    boot_identifier,
    checkout_state,
    complete,
    pool_name_of,
    pool_state,
    prepare,
    probe_fingerprint_standing,
    probe_standing,
    read_probe_standing,
};
use crate::start_probe::probe_fingerprint::{
    FingerprintPaths,
    ProbeFingerprint,
};
use crate::start_probe::probe_record::{
    ABSENT,
    RECORD_KIND,
    RECORD_VERSION,
};
use crate::worktree_clone_mount_handler::sweep_checkout_mount::{
    CheckoutMount,
    CheckoutMountRefusal,
    CheckoutMounts,
};
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
};
use crate::worktree_clone_mount_protocol::ReadyState;
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;
use std::path::{
    Path,
    PathBuf,
};

/// The pinned parent the controls name.
const PARENT: &str = "zpcachyos/nosnap/clones";

/// The checkout the controls walk.
const CHECKOUT: &str = "/fixture/checkout";

/// The running machine's fingerprint,
/// which every record this control writes pins.
fn running() -> ProbeFingerprint {
    return ProbeFingerprint::read(&FingerprintPaths::live())
        .expect("this machine's fingerprint reads");
}

/// One record a probe that passed writes,
/// in the mode's own shape and with the running fingerprint.
fn passed_record(boot: &str) -> String {
    let live: ProbeFingerprint = running();
    return format!(
        "{RECORD_VERSION}\nkind {RECORD_KIND}\nstate finished\nboot {boot}\nkernel {}\nmodule {}\nsrcversion {}\nbuild-id {}\nhandler {}\ntime 2026-11-21T20:00:00-05:00\nverdict passed\nverdict-step none\n",
        live.kernel_release,
        live.module_version.as_deref().unwrap_or(ABSENT),
        live.module_srcversion.as_deref().unwrap_or(ABSENT),
        live.module_build_id.as_deref().unwrap_or(ABSENT),
        live.handler.as_deref().unwrap_or(ABSENT)
    );
}

/// One record a probe that failed writes.
fn failed_record(
    boot: &str,
    step: &str,
) -> String {
    return format!(
        "{RECORD_VERSION}\nkind {RECORD_KIND}\nstate finished\nboot {boot}\nverdict failed\nverdict-step {step}\n"
    );
}

/// One scripted checkout mount,
/// which the walk controls answer so no control reads this host's shared home.
struct ScriptedMounts {
    /// What it answers.
    answer: Result<CheckoutMount, CheckoutMountRefusal>,
}

impl CheckoutMounts for ScriptedMounts {
    fn mount_holding(
        &self,
        _checkout: &Path,
    ) -> Result<CheckoutMount, CheckoutMountRefusal> {
        return self.answer.clone();
    }
}

/// One checkout mount that is not shared.
fn a_private_mount() -> ScriptedMounts {
    return ScriptedMounts {
        answer: Ok(CheckoutMount {
            source: String::from("zpcachyos/ROOT/cos/home/monochromatic-git-checkout"),
            filesystem_type: String::from("zfs"),
            mount_point: PathBuf::from("/fixture/checkout"),
            shared: None,
        }),
    };
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
        return Fixture {
            path: crate::test_support::fixture(format!("sweep-preconditions-{name}").as_str()),
        };
    }

    /// The folder,
    /// with a `.claude/worktrees` chain inside it when asked,
    /// and the record file when one is given.
    fn with_checkout(
        self,
        record: Option<&str>,
    ) -> Self {
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
    return DatasetIdentity {
        stats: stats(),
        properties: monochromatic_zfs_request_to_openzfs_kernel_device::NvList::new(),
    };
}

/// A pool that answers one dataset and nothing else.
struct OneDataset {
    /// The dataset it holds.
    held: Option<String>,
    /// The words it refuses with,
    /// for a pool that is not ready.
    refusal: String,
    /// The errno the binding would carry,
    /// where a control names one.
    errno: Option<i32>,
}

impl DatasetReader for OneDataset {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        return match &self.held {
            Some(held) if held == dataset => Ok(identity_of_one_dataset()),
            _ => Err(ReadFailure {
                said: self.refusal.clone(),
                errno: self.errno,
            }),
        };
    }

    fn next_child(
        &self,
        _parent: &str,
        _cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}

/// This boot's identifier is the kernel's own,
/// and it is not empty.
#[test]
fn this_boots_identifier_is_the_kernels_own() {
    let boot: String = boot_identifier().expect("this host's boot identifier reads");
    assert!(!boot.is_empty());
    assert!(
        boot.len() <= 36,
        "the record's bound holds this boot: {boot}"
    );
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
    assert_eq!(
        probe_standing(passed_record(boot.as_str()).as_str(), boot.as_str(), path),
        Ok(ProbeStanding::Passed)
    );
    let blocked: ProbeRefusal = probe_standing(
        failed_record(boot.as_str(), "attach-exclusive-create").as_str(),
        boot.as_str(),
        path,
    )
    .expect_err("a failed arm blocks the route");
    assert!(
        matches!(blocked, ProbeRefusal::Blocked { .. }),
        "{blocked:?}"
    );
    assert!(
        blocked.message().contains("attach-exclusive-create"),
        "the refusal names the step: {}",
        blocked.message()
    );
    assert!(
        blocked.message().contains("Nothing was enumerated"),
        "the refusal says nothing acted: {}",
        blocked.message()
    );
    let skipped: ProbeRefusal = probe_standing(
        format!("boot {boot}\nverdict skipped\nverdict-step device\n").as_str(),
        boot.as_str(),
        path,
    )
    .expect_err("a skipped probe blocks the route");
    assert!(
        matches!(skipped, ProbeRefusal::Blocked { .. }),
        "{skipped:?}"
    );
    let another: ProbeRefusal = probe_standing(
        passed_record("00000000-0000-4000-8000-000000000000").as_str(),
        boot.as_str(),
        path,
    )
    .expect_err("another boot's record is stale");
    assert!(
        matches!(another, ProbeRefusal::AnotherBoot { .. }),
        "{another:?}"
    );
    assert!(
        another.message().contains("is another boot's"),
        "the refusal says the record is another boot's: {}",
        another.message()
    );
    let no_verdict: ProbeRefusal =
        probe_standing(format!("boot {boot}\n").as_str(), boot.as_str(), path)
            .expect_err("a record with no verdict enables nothing");
    assert!(
        matches!(no_verdict, ProbeRefusal::Unreadable { .. }),
        "{no_verdict:?}"
    );
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
    let configuration: SweepConfiguration = SweepConfiguration {
        parent_dataset: String::from(PARENT),
        checkout: fixture.checkout(),
        checkout_dataset: String::from("zpcachyos/nosnap/checkout"),
    };
    let refusal: ProbeRefusal = prepare(
        &configuration,
        fixture.record().as_path(),
        &a_private_mount(),
    )
    .expect_err("a missing record refuses the run");
    assert!(
        matches!(refusal, ProbeRefusal::Missing { .. }),
        "{refusal:?}"
    );
    assert!(
        refusal.message().contains("start-probe"),
        "the refusal names the mode that writes it: {}",
        refusal.message()
    );
    assert!(
        refusal
            .message()
            .contains("every clone's mount intent stands untouched"),
        "the refusal says every intent stands: {}",
        refusal.message()
    );
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
    let answering: OneDataset = OneDataset {
        held: Some(String::from(PARENT)),
        refusal: String::from("no such pool"),
        errno: Some(libc::ENOENT),
    };
    assert_eq!(pool_state(&answering, PARENT), ReadyState::Ready);
    let gone: OneDataset = OneDataset {
        held: None,
        refusal: String::from("dataset does not exist"),
        errno: Some(libc::ENOENT),
    };
    assert_eq!(pool_state(&gone, PARENT), ReadyState::NotReady);
    let unreadable: OneDataset = OneDataset {
        held: None,
        refusal: String::from("the device answered EIO"),
        errno: Some(libc::EIO),
    };
    assert_eq!(pool_state(&unreadable, PARENT), ReadyState::Unknown);
    // A sentence that says a pool is gone is not the kernel's answer, so a read that kept no errno admits nothing.
    let words_only: OneDataset = OneDataset {
        held: None,
        refusal: String::from("dataset does not exist"),
        errno: None,
    };
    assert_eq!(pool_state(&words_only, PARENT), ReadyState::Unknown);
}

/// A checkout whose `.claude/worktrees` folder walks is ready,
/// and one that is not there is not ready.
#[test]
fn the_checkout_is_ready_only_when_the_walk_reaches_it() {
    let fixture: Fixture = Fixture::new("checkout").with_checkout(None);
    let (state, _said): (ReadyState, String) =
        checkout_state(fixture.checkout().as_path(), &a_private_mount());
    assert_eq!(state, ReadyState::Ready);
    let absent: PathBuf = fixture.path.join("nothing-here");
    let (missing, said): (ReadyState, String) =
        checkout_state(absent.as_path(), &a_private_mount());
    assert_eq!(missing, ReadyState::NotReady, "{said}");
    assert!(
        said.contains("nothing-here"),
        "the refusal names the checkout: {said}"
    );
}

/// A run whose prerequisites all hold is prepared,
/// and one whose checkout does not walk is refused before any device is opened.
#[test]
fn prepare_reads_the_record_and_the_checkout_before_any_device() {
    let boot: String = boot_identifier().expect("this host's boot identifier reads");
    let fixture: Fixture =
        Fixture::new("prepared").with_checkout(Some(passed_record(boot.as_str()).as_str()));
    let configuration: SweepConfiguration = SweepConfiguration {
        parent_dataset: String::from(PARENT),
        checkout: fixture.checkout(),
        checkout_dataset: String::from("zpcachyos/nosnap/checkout"),
    };
    let prepared = prepare(
        &configuration,
        fixture.record().as_path(),
        &a_private_mount(),
    )
    .expect("every check passed");
    assert_eq!(prepared.boot, boot);
    assert_eq!(prepared.checkout_state, ReadyState::Ready);
    assert_eq!(prepared.standing, ProbeStanding::Passed);
    let absent: SweepConfiguration = SweepConfiguration {
        parent_dataset: String::from(PARENT),
        checkout: fixture.path.join("nowhere"),
        checkout_dataset: String::from("zpcachyos/nosnap/checkout"),
    };
    let refused = prepare(&absent, fixture.record().as_path(), &a_private_mount())
        .expect("the record passes, so the checkout is what the admission refuses");
    assert_eq!(refused.checkout_state, ReadyState::NotReady);
    let facts: BootFacts = BootFacts {
        boot: String::from("boot"),
        pool_name: String::from("zpcachyos"),
        pool_state: ReadyState::Ready,
        checkout: PathBuf::from(CHECKOUT),
        checkout_state: ReadyState::NotReady,
        checkout_said: String::from(
            "the walk from the checkout did not reach its worktrees folder",
        ),
    };
    let admission: ProbeRefusal = facts
        .admit()
        .expect_err("a checkout that is not ready admits nothing");
    assert!(
        matches!(admission, ProbeRefusal::CheckoutNotReady { .. }),
        "{admission:?}"
    );
    let unknown: BootFacts = BootFacts {
        checkout_state: ReadyState::Ready,
        pool_state: ReadyState::Unknown,
        ..facts
    };
    assert!(
        matches!(unknown.admit(), Err(ProbeRefusal::PoolNotReady { .. })),
        "an unknown pool admits nothing"
    );
}

/// What:
///  A shared checkout's own refusal reaches the person who reads the admission,
///  instead of a sentence about a walk that did reach the folder.
/// Why:
///  The checkout's check builds the words that say why it is not ready,
///  and a person who reads "the walk did not reach its worktrees folder" about
///  a shared mount looks at the wrong thing.
#[test]
fn a_shared_checkout_is_refused_with_its_own_reason() {
    let boot: String = boot_identifier().expect("this host's boot identifier reads");
    let fixture: Fixture =
        Fixture::new("shared-reason").with_checkout(Some(passed_record(boot.as_str()).as_str()));
    let configuration: SweepConfiguration = SweepConfiguration {
        parent_dataset: String::from(PARENT),
        checkout: fixture.checkout(),
        checkout_dataset: String::from("zpcachyos/nosnap/checkout"),
    };
    let shared: ScriptedMounts = ScriptedMounts {
        answer: Ok(CheckoutMount {
            source: String::from("zpcachyos/ROOT/cos/home/monochromatic-git-checkout"),
            filesystem_type: String::from("zfs"),
            mount_point: fixture.checkout(),
            shared: Some(String::from("shared:217")),
        }),
    };
    let prepared = prepare(&configuration, fixture.record().as_path(), &shared)
        .expect("the record passes, so the checkout is what the admission refuses");
    assert_eq!(prepared.checkout_state, ReadyState::NotReady);
    assert!(
        prepared.checkout_said.contains("shared:217"),
        "the prepared facts keep the checkout's own words: {}",
        prepared.checkout_said
    );
    let answering: OneDataset = OneDataset {
        held: Some(String::from(PARENT)),
        refusal: String::from("no such pool"),
        errno: Some(libc::ENOENT),
    };
    let refusal: ProbeRefusal =
        complete(prepared, &answering, PARENT).expect_err("a shared checkout admits nothing");
    assert!(
        matches!(refusal, ProbeRefusal::CheckoutNotReady { .. }),
        "{refusal:?}"
    );
    let said: String = refusal.message();
    assert!(
        said.contains("shared:217"),
        "the refusal names the shared tag a person must clear: {said}"
    );
    assert!(
        !said.contains("did not reach its worktrees folder"),
        "the refusal does not name a walk that did reach the folder: {said}"
    );
}

/// What:
///  A record that names the live boot and a module other than the running one
///  is stale when it is read through the read the sweep makes.
/// Why:
///  The fingerprint helper's own control never reaches `read_probe_standing`,
///  so a read that stopped calling it would still pass every other control.
#[test]
fn a_record_of_another_module_is_stale_through_the_read_the_sweep_makes() {
    let boot: String = boot_identifier().expect("this host's boot identifier reads");
    let live: ProbeFingerprint = running();
    let record: String = passed_record(boot.as_str());
    let running_module: String = format!(
        "module {}",
        live.module_version.as_deref().unwrap_or(ABSENT)
    );
    let wrong: String = record.replacen(
        running_module.as_str(),
        "module 0.0.0-not-the-running-module",
        1,
    );
    assert_ne!(wrong, record, "the control changed the module line");
    let fixture: Fixture = Fixture::new("stale-module").with_checkout(Some(wrong.as_str()));
    let refusal: ProbeRefusal =
        read_probe_standing(fixture.record().as_path()).expect_err("another module is stale");
    assert!(
        matches!(
            &refusal,
            ProbeRefusal::Stale {
                line: "module",
                ..
            }
        ),
        "{refusal:?}"
    );
    assert!(
        refusal.message().contains("0.0.0-not-the-running-module"),
        "the refusal names the record's module: {}",
        refusal.message()
    );
}

/// What:
///  A record whose kernel,
///  module,
///  source version or build ID differs from the running one is stale,
///  and a file that is no `v1` `probe-result` is refused as such.
/// Why:
///  The SPEC's rule is that a record whose fingerprint differs from the running
///  kernel,
///  module or boot is stale,
///  and comparing the boot and the verdict alone would accept a record written
///  before a module reload within one boot.
#[test]
fn a_record_of_another_kernel_or_module_is_stale() {
    let path: &Path = Path::new("/fixture/probe-record");
    let live: ProbeFingerprint = running();
    let record: String = passed_record(live.boot_id.as_str());
    probe_fingerprint_standing(record.as_str(), &live, path)
        .expect("the record pins the running machine");

    let another_kernel: ProbeFingerprint = ProbeFingerprint {
        kernel_release: String::from("0.0.0-other"),
        ..live.clone()
    };
    let stale: ProbeRefusal = probe_fingerprint_standing(record.as_str(), &another_kernel, path)
        .expect_err("another kernel is stale");
    assert!(
        matches!(
            &stale,
            ProbeRefusal::Stale {
                line: "kernel",
                ..
            }
        ),
        "{stale:?}"
    );
    assert!(
        stale.message().contains("0.0.0-other"),
        "the refusal names the record's kernel: {}",
        stale.message()
    );

    let another_module: ProbeFingerprint = ProbeFingerprint {
        module_version: Some(String::from("2.4.5-1")),
        ..live.clone()
    };
    let module_stale: ProbeRefusal =
        probe_fingerprint_standing(record.as_str(), &another_module, path)
            .expect_err("another module is stale");
    assert!(
        matches!(
            &module_stale,
            ProbeRefusal::Stale {
                line: "module",
                ..
            }
        ),
        "{module_stale:?}"
    );

    let another_srcversion: ProbeFingerprint = ProbeFingerprint {
        module_srcversion: Some(String::from("FFFFFFFF")),
        ..live.clone()
    };
    let srcversion_stale: ProbeRefusal =
        probe_fingerprint_standing(record.as_str(), &another_srcversion, path)
            .expect_err("another srcversion is stale");
    assert!(
        matches!(
            &srcversion_stale,
            ProbeRefusal::Stale {
                line: "srcversion",
                ..
            }
        ),
        "{srcversion_stale:?}"
    );

    let another_build: ProbeFingerprint = ProbeFingerprint {
        module_build_id: Some(String::from("ffffffff")),
        ..live.clone()
    };
    let build_stale: ProbeRefusal =
        probe_fingerprint_standing(record.as_str(), &another_build, path)
            .expect_err("another build is stale");
    assert!(
        matches!(
            &build_stale,
            ProbeRefusal::Stale {
                line: "build-id",
                ..
            }
        ),
        "{build_stale:?}"
    );

    let wrong_kind: String = record.replacen(
        "kind probe-result",
        "kind git-worktree-clone-mount-probe",
        1,
    );
    let refused: ProbeRefusal = probe_fingerprint_standing(wrong_kind.as_str(), &live, path)
        .expect_err("another kind is no record this route reads");
    assert!(
        matches!(refused, ProbeRefusal::NotARecord { .. }),
        "{refused:?}"
    );
    let no_version: String = record.replacen("v1\n", "", 1);
    assert!(
        matches!(
            probe_fingerprint_standing(no_version.as_str(), &live, path),
            Err(ProbeRefusal::NotARecord { .. })
        ),
        "a record without v1 is refused"
    );
}

/// What:
///  A checkout whose own mount is shared is not ready,
///  and a mount that cannot be read is unknown,
///  which admits nothing.
/// Why:
///  The design's measured move refused `EINVAL` under a shared checkout and
///  passed once only the checkout was private,
///  so this route requires a not-shared immediate parent.
#[test]
fn a_shared_checkout_is_not_ready() {
    let fixture: Fixture = Fixture::new("shared-checkout").with_checkout(None);
    let shared: ScriptedMounts = ScriptedMounts {
        answer: Ok(CheckoutMount {
            source: String::from("zpcachyos/ROOT/cos/home/monochromatic-git-checkout"),
            filesystem_type: String::from("zfs"),
            mount_point: fixture.checkout(),
            shared: Some(String::from("shared:217")),
        }),
    };
    let (state, said): (ReadyState, String) = checkout_state(fixture.checkout().as_path(), &shared);
    assert_eq!(state, ReadyState::NotReady, "{said}");
    assert!(
        said.contains("shared:217"),
        "the refusal names the propagation tag: {said}"
    );
    let unreadable: ScriptedMounts = ScriptedMounts {
        answer: Err(CheckoutMountRefusal::Unreadable {
            checkout: fixture.checkout(),
            reason: String::from("the table did not read"),
        }),
    };
    let (unknown, why): (ReadyState, String) =
        checkout_state(fixture.checkout().as_path(), &unreadable);
    assert_eq!(unknown, ReadyState::Unknown, "{why}");
    assert!(
        why.contains("the table did not read"),
        "the refusal keeps the read's words: {why}"
    );
}
