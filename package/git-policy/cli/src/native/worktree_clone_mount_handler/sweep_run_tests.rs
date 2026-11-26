//! What:
//!  Controls of one whole run:
//!  a clone that refused does not stop the run,
//!  a listing that stopped says where,
//!  and the counts and lines are the run's own.
//! Why:
//!  One malformed clone must not leave every later clone unmounted,
//!  and a run that stopped mid-listing must not read as a finished one,
//!  so both are proved inside a user namespace where the fixtures are root's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!inARootNamespace(...)) return;
//! const run = sweep(context); expect(run.attached()).toBe(1);
//! ```

use super::{
    SweepRun,
    sweep,
};
use crate::testing::root_namespace_test_support::in_a_root_namespace;
use crate::worktree_clone_mount::CloneFolderIdentity;
use crate::worktree_clone_mount::publish_parent_identity_under_root_control;
use crate::worktree_clone_mount::{
    MountIdentity,
    SlotIdentityStore,
};
use crate::worktree_clone_mount_handler::per_slot_root_claim::SlotClaimDirectory;
use crate::worktree_clone_mount_handler::sweep_attach_step::{
    AttachRequest,
    AttachStepOutcome,
    CloneAttachStep,
};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    DEVICES_PROPERTY,
    DMU_OBJECT_SET_FILESYSTEM,
    ENCRYPTION_PROPERTY,
    ENCRYPTION_ROOT_PROPERTY,
    KEY_STATUS_PROPERTY,
    MOUNTPOINT_PROPERTY,
    SETUID_PROPERTY,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ReadFailure;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
};
use crate::worktree_clone_mount_handler::sweep_one_clone::SweepContext;
use crate::worktree_clone_mount_protocol::{
    DESIRED_STATE_PROPERTY,
    MOUNT_WORD,
    ReceiptsFolder,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DmuObjsetStats,
    NvList,
};
use std::cell::Cell;
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};

/// The variable the inner half of a control sets.
const INSIDE: &str = "CLI_GIT_ZFS_SWEEP_RUN_ROOT";

/// The pinned clone parent.
const PARENT: &str = "zpcachyos/nosnap/clones";

/// The checkout dataset a clone's origin is a snapshot of.
const CHECKOUT_DATASET: &str = "zpcachyos/nosnap/checkout";

/// The child that asks for its mount.
const MOUNTING: &str = "zpcachyos/nosnap/clones/mounting";

/// The child that asks for nothing.
const QUIET: &str = "zpcachyos/nosnap/clones/quiet";

/// The child whose intent is malformed.
const MALFORMED: &str = "zpcachyos/nosnap/clones/malformed";

/// The nonce the mounting child asks under.
const NONCE: &str = "fedcba9876543210fedcba9876543210";

/// The boot the receipts name.
const BOOT: &str = "00000000-0000-4000-8000-000000000000";

/// The time every receipt carries.
const TIME: &str = "2026-11-21T20:00:00-05:00";

/// One property as OpenZFS writes it.
fn pair_number(
    value: u64,
    source: &str,
) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_u64("value", value);
    pair.add_str("source", source);
    return pair;
}

/// One text property in the same shape.
fn pair_text(
    value: &str,
    source: &str,
) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", value);
    pair.add_str("source", source);
    return pair;
}

/// The reply one managed clone answers with,
/// whose intent is local to `name` when one is given.
fn reply(
    name: &str,
    intent: Option<&str>,
) -> NvList {
    let mut reply: NvList = NvList::new();
    reply.add_nvlist(MOUNTPOINT_PROPERTY, pair_text("legacy", PARENT));
    reply.add_nvlist(SETUID_PROPERTY, pair_number(0, PARENT));
    reply.add_nvlist(DEVICES_PROPERTY, pair_number(0, PARENT));
    reply.add_nvlist(ENCRYPTION_PROPERTY, pair_number(0, PARENT));
    reply.add_nvlist(ENCRYPTION_ROOT_PROPERTY, pair_text("zpcachyos", PARENT));
    reply.add_nvlist(KEY_STATUS_PROPERTY, pair_number(0, PARENT));
    if let Some(value) = intent {
        reply.add_nvlist(DESIRED_STATE_PROPERTY, pair_text(value, name));
    }
    return reply;
}

/// The statistics of one child,
/// whose origin is a snapshot of the parent.
fn stats(guid: u64) -> DmuObjsetStats {
    let origin: String = format!("{CHECKOUT_DATASET}@git-worktree-clone-origin-00112233445566");
    let mut bytes: [u8; 256] = [0; 256];
    bytes[..origin.len()].copy_from_slice(origin.as_bytes());
    return DmuObjsetStats {
        dds_num_clones: 0,
        dds_creation_txg: 9,
        dds_guid: guid,
        dds_type: DMU_OBJECT_SET_FILESYSTEM,
        dds_is_snapshot: 0,
        dds_inconsistent: 0,
        dds_redacted: 0,
        dds_origin: bytes,
        dds_flags: 0,
    };
}

/// A pool holding the three children,
/// with an optional stop at one cursor.
struct ScriptedPool {
    /// The children the listing gives.
    children: Vec<String>,
    /// The replies by child.
    replies: Vec<(String, DmuObjsetStats, NvList)>,
    /// The cursor the listing refuses at,
    /// or nothing.
    fail_at: Option<u64>,
}

impl DatasetReader for ScriptedPool {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        let Some((_name, stats, properties)) = self
            .replies
            .iter()
            .find(|(name, _, _)| return name == dataset)
        else {
            return Err(ReadFailure::at(format!(
                "this scripted pool holds no `{dataset}`"
            )));
        };
        return Ok(DatasetIdentity {
            stats: *stats,
            properties: properties.clone(),
        });
    }

    fn next_child(
        &self,
        _parent: &str,
        cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        if self.fail_at == Some(cursor) {
            return Err(String::from("the device answered EIO"));
        }
        let index: usize = usize::try_from(cursor).unwrap_or(usize::MAX);
        return Ok(self.children.get(index).map(|name| {
            return ListedChild {
                name: name.clone(),
                cursor: cursor + 1,
            };
        }));
    }
}

/// An attach step that says the mount landed at the slot.
struct MountingAttach {
    /// How many times it was asked.
    asked: Cell<usize>,
}

impl CloneAttachStep for MountingAttach {
    fn attach(
        &self,
        request: &AttachRequest<'_>,
    ) -> AttachStepOutcome {
        self.asked.set(self.asked.get() + 1);
        return AttachStepOutcome::Attached {
            mount: MountIdentity {
                mount_id: 41,
                parent_mount_id: 40,
                mount_point: PathBuf::from(format!(
                    "/fixture/checkout/.claude/worktrees/{}",
                    request.slot
                )),
                source: String::from(request.clone),
                file_system_type: String::from("zfs"),
                device: 57,
                inode: 1_337,
                observed_path: PathBuf::from(format!(
                    "/fixture/checkout/.claude/worktrees/{}",
                    request.slot
                )),
            },
        };
    }
}

/// What one run's first attach answers:
/// nothing,
/// because this control proves the run's counts and its lines.
struct SilentFirstAttach;

impl crate::worktree_clone_mount_handler::sweep_first_attach::FirstAttachRecorder
    for SilentFirstAttach
{
    fn claim(&self) -> Option<crate::start_probe::first_attach_record::FirstAttachClaim> {
        return None;
    }

    fn record(
        &self,
        _claim: &crate::start_probe::first_attach_record::FirstAttachClaim,
        _outcome: &crate::start_probe::first_attach_record::FirstAttachOutcome,
    ) {
    }
}

/// What the canonical slot holds before an attach:
/// nothing,
/// because each slot in this fixture is unmounted.
struct NoStandingMount;

impl crate::worktree_clone_mount_handler::sweep_one_clone::StandingMounts for NoStandingMount {
    fn standing_mount_of(
        &self,
        _request: &crate::worktree_clone_mount::LookAgainAtSlotRequest<'_>,
        _clone: &str,
    ) -> Option<MountIdentity> {
        return None;
    }
}

/// One fixture:
/// a checkout with three slots and a root-owned store.
struct Fixture {
    /// The checkout.
    checkout: PathBuf,
    /// The store.
    store: PathBuf,
    /// The claims folder.
    claims: PathBuf,
    /// The receipts folder.
    receipts: PathBuf,
}

impl Fixture {
    /// Build the fixture under this run's own temporary root.
    fn build(name: &str) -> Self {
        let root: PathBuf = crate::test_support::fixture(format!("sweep-run-{name}").as_str());
        let checkout: PathBuf = root.join("checkout");
        let parent: PathBuf = checkout.join(".claude").join("worktrees");
        for slot in ["mounting", "quiet", "malformed"] {
            std::fs::create_dir_all(parent.join(slot)).expect("the slot folder is made");
            let pointer: String = format!(
                "gitdir: {}\n",
                checkout.join(".git").join("worktrees").join(slot).display()
            );
            std::fs::write(parent.join(slot).join(".git"), pointer)
                .expect("the slot's Git pointer is written");
        }
        let store: PathBuf = root.join("store");
        std::fs::create_dir(store.as_path()).expect("the store folder is made");
        std::fs::set_permissions(store.as_path(), std::fs::Permissions::from_mode(0o755))
            .expect("the store is root's");
        let identity: CloneFolderIdentity = CloneFolderIdentity::capture(
            &std::fs::File::open(parent.as_path()).expect("the parent opens"),
        )
        .expect("the identity reads");
        publish_parent_identity_under_root_control(store.as_path(), &identity, &identity)
            .expect("the parent record is published");
        let claims: PathBuf = root.join("claims");
        let receipts: PathBuf = root.join("receipts");
        return Fixture {
            checkout,
            store,
            claims,
            receipts,
        };
    }

    /// The context a run sweeps with.
    fn context<'a>(
        &'a self,
        pool: &'a ScriptedPool,
        attach: &'a MountingAttach,
        receipts: &'a ReceiptsFolder,
        claims: &'a SlotClaimDirectory,
    ) -> SweepContext<'a> {
        return SweepContext {
            parent_dataset: PARENT,
            checkout: self.checkout.as_path(),
            checkout_dataset: CHECKOUT_DATASET,
            store: self.store.as_path(),
            claims,
            receipts,
            boot: BOOT,
            time: &|| return String::from(TIME),
            reader: pool,
            attach,
            first_attach: &SilentFirstAttach,
            standing: &NoStandingMount,
        };
    }
}

/// The pool holding one mounting child,
/// one quiet child and one malformed child.
fn three_children() -> ScriptedPool {
    let intent: String = format!("v1:{NONCE}:{MOUNT_WORD}");
    return ScriptedPool {
        children: vec![
            String::from(MOUNTING),
            String::from(QUIET),
            String::from(MALFORMED),
        ],
        replies: vec![
            (
                String::from(MOUNTING),
                stats(3),
                reply(MOUNTING, Some(intent.as_str())),
            ),
            (String::from(QUIET), stats(4), reply(QUIET, None)),
            (
                String::from(MALFORMED),
                stats(5),
                reply(MALFORMED, Some("v1:nothex:mount")),
            ),
        ],
        fail_at: None,
    };
}

/// What:
///  Every child of the parent takes its turn,
///  a refused child does not stop the run,
///  and the counts are the run's own.
/// Why:
///  One malformed clone must not leave the clones after it unmounted,
///  and a run that stopped at the first refusal would look complete.
#[test]
fn every_child_takes_its_turn_and_a_refusal_does_not_stop_the_run() {
    if !in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_run::tests::every_child_takes_its_turn_and_a_refusal_does_not_stop_the_run",
    ) {
        return;
    }
    let fixture: Fixture = Fixture::build("three");
    let pool: ScriptedPool = three_children();
    let attach: MountingAttach = MountingAttach {
        asked: Cell::new(0),
    };
    let receipts: ReceiptsFolder = ReceiptsFolder::at(fixture.receipts.clone(), 0);
    let claims: SlotClaimDirectory = SlotClaimDirectory::at(fixture.claims.clone(), 0);
    let run: SweepRun = sweep(&fixture.context(&pool, &attach, &receipts, &claims));
    assert!(run.completed(), "the listing reached the end");
    assert_eq!(run.children, 3);
    assert_eq!(run.reached.len(), 3);
    assert_eq!(run.attached(), 1);
    assert_eq!(run.nothing_asked(), 1);
    assert_eq!(run.refused(), 1);
    assert_eq!(run.failed(), 0);
    assert_eq!(run.held(), 0);
    assert_eq!(
        attach.asked.get(),
        1,
        "only the child that asked for a mount was attached"
    );
    let summary: String = run.summary();
    assert!(summary.contains("1 attached"), "{summary}");
    assert!(summary.contains("1 refused"), "{summary}");
    assert!(summary.contains("reached the end"), "{summary}");
    assert!(
        run.report().contains(MALFORMED),
        "the refused child's own line is in the report: {}",
        run.report()
    );
    let store: SlotIdentityStore =
        SlotIdentityStore::open_under_root_control(fixture.store.as_path())
            .expect("the store opens");
    assert!(
        store.slot("mounting").is_ok(),
        "the attached clone's slot is recorded"
    );
    assert!(
        matches!(
            store.slot("quiet"),
            Err(crate::worktree_clone_mount::StoreRefusal::NoSlotRecord { .. })
        ),
        "a clone that asked for nothing records nothing"
    );
}

/// What:
///  A listing that stopped says where it stopped,
///  keeps the clone it attached,
///  and reads as incomplete.
/// Why:
///  Clones already attached keep their mounts,
///  and a run that could not finish must not be counted as one that did.
#[test]
fn a_listing_that_stopped_says_where_and_keeps_what_it_attached() {
    if !in_a_root_namespace(
        INSIDE,
        "worktree_clone_mount_handler::sweep_run::tests::a_listing_that_stopped_says_where_and_keeps_what_it_attached",
    ) {
        return;
    }
    let fixture: Fixture = Fixture::build("stopped");
    let mut pool: ScriptedPool = three_children();
    pool.fail_at = Some(1);
    pool.children.truncate(1);
    let attach: MountingAttach = MountingAttach {
        asked: Cell::new(0),
    };
    let receipts: ReceiptsFolder = ReceiptsFolder::at(fixture.receipts.clone(), 0);
    let claims: SlotClaimDirectory = SlotClaimDirectory::at(fixture.claims.clone(), 0);
    let run: SweepRun = sweep(&fixture.context(&pool, &attach, &receipts, &claims));
    assert!(!run.completed(), "a listing that refused did not finish");
    assert_eq!(run.children, 1);
    assert_eq!(
        run.attached(),
        1,
        "the clone the listing did reach was attached"
    );
    let summary: String = run.summary();
    assert!(summary.contains("The listing stopped"), "{summary}");
    assert!(
        summary.contains("EIO"),
        "the stop names what the read answered: {summary}"
    );
    let _path: &Path = fixture.checkout.as_path();
}
