//! What:
//!  The one repository,
//!  binding record,
//!  origin-side inventory and scripted pool the removal's controls drive,
//!  with a builder that breaks one field at a time.
//! Why:
//!  No control on a person's machine may open `/dev/zfs`,
//!  so the proof and the entry read a scripted pool through the same seam the release answers
//!  with,
//!  and one healthy fixture is what proves each refusal came from the field its control changed
//!  rather than from another comparison.
//!  The identities are the design's own example values,
//!  so a message reads as the design's does.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = RemovalFixture.healthy('a-control');
//! const pool = fixture.pool().withoutTheCloneOrigin();
//! expect(proveTheLineage(pool, fixture.recorded, fixture.originInventory)).toThrow();
//! ```

use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::binding_encoding::{
    FolderIdentity,
    bytes_as_hexadecimal,
    path_as_hexadecimal,
};
use crate::worktree_clone_binding_record::binding_inventory::{
    EntryKind,
    InventoryEntry,
    InventoryKind,
    InventoryRecord,
};
use crate::worktree_clone_binding_record::binding_words::BINDING_HEADER;
use crate::worktree_clone_binding_record::registration_home::{
    binding_name,
    home_folder,
    origin_inventory_name,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::{
    RecordedBinding,
    read_the_recorded_binding,
};
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_removal_deferred_origin_check::fixtures::{
    origin_properties,
    stats,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DmuObjsetStats,
    NvList,
};
use std::path::{
    Path,
    PathBuf,
};

/// The slot every control removes.
pub const SLOT: &str = "fix-flaky-test";
/// The clone's full dataset name.
pub const CLONE: &str = "tank/wt/fix-flaky-test";
/// The origin dataset's full name.
pub const ORIGIN_DATASET: &str = "tank/mono-checkout";
/// The origin snapshot's full name.
pub const ORIGIN_SNAPSHOT: &str = "tank/mono-checkout@origin";
/// The pool's dataset name.
pub const POOL: &str = "tank";
/// The clone's GUID.
pub const CLONE_GUID: u64 = 7_828_944_690_444_299_989;
/// The clone's own creation transaction group.
pub const CLONE_CREATETXG: u64 = 17;
/// The origin dataset's GUID.
pub const ORIGIN_DATASET_GUID: u64 = 4_815_162_342_109_876_543;
/// The origin snapshot's GUID.
pub const ORIGIN_SNAPSHOT_GUID: u64 = 11_050_373_805_378_911_967;
/// The origin snapshot's creation transaction group.
pub const ORIGIN_CREATETXG: u64 = 15;
/// The pool's GUID.
pub const POOL_GUID: u64 = 1_234_567_890_123_456_789;
/// The clone's own manual snapshot,
/// which the design's example calls `worktree-own`.
pub const OWN_SNAPSHOT: &str = "worktree-own";
/// The clone's own sanoid snapshot.
pub const SANOID_SNAPSHOT: &str = "sanoid-hourly";
/// The own snapshot's GUID.
pub const OWN_GUID: u64 = 17_157_080_670_377_245_572;
/// The own snapshot's creation transaction group.
pub const OWN_CREATETXG: u64 = 18;
/// The sanoid snapshot's GUID.
pub const SANOID_GUID: u64 = 9_332_110_456_720_018_274;
/// The sanoid snapshot's creation transaction group.
pub const SANOID_CREATETXG: u64 = 20;

/// One dataset GUID built from a decimal number.
pub fn guid(value: u64) -> DatasetGuid {
    return DatasetGuid::parse(value.to_string().as_str()).expect("a number is a canonical GUID");
}

/// The statistics one identity reply carries.
pub fn the_stats(
    guid_value: u64,
    is_snapshot: bool,
    creation_txg: u64,
    clones: u64,
    origin: &str,
) -> DmuObjsetStats {
    return stats(guid_value, is_snapshot, creation_txg, clones, origin);
}

/// One live object of the clone's inventory.
pub fn an_object(
    kind: EntryKind,
    name: &str,
    guid_value: u64,
    createtxg: u64,
) -> LiveObject {
    return LiveObject {
        kind,
        name: String::from(name),
        guid: guid(guid_value),
        createtxg,
        owning_dataset: String::from(CLONE),
        owning_dataset_guid: guid(CLONE_GUID),
    };
}

/// What:
///  The pool a control answers with:
///  one identity reply for any name the control scripts,
///  and the clone's two lists.
/// Why:
///  The proof reads exactly these three answers,
///  and a control that broke one field would otherwise have to write a whole pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ScriptedPool implements RemovalPoolReads { ... }
/// ```
pub struct ScriptedPool {
    /// One identity reply per name,
    /// in the order the control wrote them.
    pub identities: Vec<(String, DatasetIdentity)>,
    /// The clone's live snapshots.
    pub snapshots: Vec<LiveObject>,
    /// The clone's live bookmarks.
    pub bookmarks: Vec<LiveObject>,
    /// Why a listing refuses,
    /// when a control plants one.
    pub listing_refusal: Option<String>,
}

impl ScriptedPool {
    /// What:
    ///  A second pool with the same answers,
    ///  so one control breaks one field of one copy.
    /// Why:
    ///  A reply's property list is the kernel's decoded list and is not `Clone`,
    ///  so the copy is written where that type is known rather than derived.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const other = pool.copy(); other.with(name, anotherReply);
    /// ```
    pub fn copy(&self) -> ScriptedPool {
        let identities: Vec<(String, DatasetIdentity)> = self
            .identities
            .iter()
            .map(|(name, identity)| {
                return (
                    name.clone(),
                    DatasetIdentity {
                        stats: identity.stats,
                        properties: identity.properties.clone(),
                    },
                );
            })
            .collect();
        return ScriptedPool {
            identities,
            snapshots: self.snapshots.clone(),
            bookmarks: self.bookmarks.clone(),
            listing_refusal: self.listing_refusal.clone(),
        };
    }

    /// Replace one name's identity reply.
    #[allow(dead_code)]
    pub fn with(
        mut self,
        name: &str,
        identity: DatasetIdentity,
    ) -> ScriptedPool {
        for pair in &mut self.identities {
            if pair.0 == name {
                pair.1 = identity;
                return self;
            }
        }
        self.identities.push((String::from(name), identity));
        return self;
    }

    /// One name's identity as this pool would answer it.
    fn identity_at(
        &self,
        name: &str,
    ) -> Option<&DatasetIdentity> {
        for (scripted, identity) in &self.identities {
            if scripted == name {
                return Some(identity);
            }
        }
        return None;
    }
}

impl RemovalPoolReads for ScriptedPool {
    fn identity(
        &self,
        name: &str,
    ) -> Result<DatasetIdentity, String> {
        let Some(found) = self.identity_at(name) else {
            return Err(format!("`{name}` does not answer this scripted pool"));
        };
        return Ok(DatasetIdentity {
            stats: found.stats,
            properties: found.properties.clone(),
        });
    }

    fn snapshots_of(
        &self,
        _dataset: &str,
        _dataset_guid: DatasetGuid,
    ) -> Result<Vec<LiveObject>, String> {
        if let Some(reason) = &self.listing_refusal {
            return Err(reason.clone());
        }
        return Ok(self.snapshots.clone());
    }

    fn bookmarks_of(
        &self,
        _dataset: &str,
        _dataset_guid: DatasetGuid,
    ) -> Result<Vec<LiveObject>, String> {
        if let Some(reason) = &self.listing_refusal {
            return Err(reason.clone());
        }
        return Ok(self.bookmarks.clone());
    }
}

/// What:
///  One throwaway repository with a real registration,
///  a binding record and an origin-side inventory.
/// Why:
///  The entry reads three records from the common directory and compares the registration's own
///  identity with the one the binding recorded,
///  so a fixture that planted plausible numbers would fail on the identity check instead of on the
///  field its control changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = RemovalFixture.healthy('one-control');
/// fixture.binding = fixture.binding.replace('poolGuid=', 'poolGuid=9');
/// ```
pub struct RemovalFixture {
    /// The repository's common Git directory.
    pub common_dir: PathBuf,
    /// The slot's path.
    pub slot_path: PathBuf,
    /// The registration directory.
    pub registration: PathBuf,
    /// The binding record's lines,
    /// which a control edits before it writes them.
    pub binding_lines: Vec<String>,
    /// The origin snapshot's full name,
    /// which a control that needs a record's own name grammar changes.
    pub origin_snapshot: String,
}

impl RemovalFixture {
    /// What:
    ///  One healthy fixture:
    ///  a common directory,
    ///  a registration directory with its identity recorded,
    ///  a binding at phase `filled`,
    ///  and the origin-side inventory the add wrote.
    /// Why:
    ///  Every field the removal reads is present and agrees with the pool below,
    ///  so a control changes exactly one of them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fixture = RemovalFixture.healthy('a-control');
    /// ```
    pub fn healthy(name: &str) -> RemovalFixture {
        let root: PathBuf = fixture(&format!("removal-proof-and-plan-{name}"));
        let common_dir: PathBuf = root.join(".git");
        std::fs::create_dir_all(common_dir.as_path()).expect("the common directory");
        let slot_path: PathBuf = root.join(".claude").join("worktrees").join(SLOT);
        std::fs::create_dir_all(slot_path.as_path()).expect("the slot folder");
        let registration: PathBuf = common_dir.join("worktrees").join("fix-flaky-test");
        std::fs::create_dir_all(registration.as_path()).expect("the registration");
        let repository_identity: FolderIdentity = FolderIdentity::of_folder(common_dir.as_path())
            .expect("the common directory's identity");
        let registration_identity: FolderIdentity =
            FolderIdentity::of_folder(registration.as_path()).expect("the registration's identity");
        let lines: Vec<String> = vec![
            String::from(BINDING_HEADER),
            String::from("claim=1"),
            String::from("nonce=00112233445566778899aabbccddeeff"),
            format!("request=branch {}", bytes_as_hexadecimal(b"main")),
            String::from("phase=filled"),
            format!("checkoutDataset={ORIGIN_DATASET}"),
            format!("checkoutDatasetGuid={ORIGIN_DATASET_GUID}"),
            format!("repository={}", path_as_hexadecimal(common_dir.as_path())),
            format!("repositoryIdentity={}", repository_identity.encode()),
            format!("slot={}", path_as_hexadecimal(slot_path.as_path())),
            format!("slotIdentity={}", registration_identity.encode()),
            format!(
                "registration={}",
                path_as_hexadecimal(registration.as_path())
            ),
            format!("registrationIdentity={}", registration_identity.encode()),
            format!("pool={POOL}"),
            format!("poolGuid={POOL_GUID}"),
            format!("originSnapshot={ORIGIN_SNAPSHOT}"),
            format!("originSnapshotGuid={ORIGIN_SNAPSHOT_GUID}"),
            format!("originSnapshotCreatetxg={ORIGIN_CREATETXG}"),
            format!("originDataset={ORIGIN_DATASET}"),
            format!("originDatasetGuid={ORIGIN_DATASET_GUID}"),
            format!("cloneDataset={CLONE}"),
            format!("cloneGuid={CLONE_GUID}"),
            format!("cloneCreatetxg={CLONE_CREATETXG}"),
            format!("pointer={}", bytes_as_hexadecimal(b"gitdir: /nowhere\n")),
        ];
        let fixture: RemovalFixture = RemovalFixture {
            common_dir,
            slot_path,
            registration,
            binding_lines: lines,
            origin_snapshot: String::from(ORIGIN_SNAPSHOT),
        };
        fixture.write_the_binding();
        fixture.write_the_origin_inventory();
        return fixture;
    }

    /// What:
    ///  The same fixture with another origin snapshot name,
    ///  written into the binding and the origin-side inventory.
    /// Why:
    ///  The origin cleanup record is keyed by the snapshot's own name,
    ///  which begins with the add's prefix and the add's nonce,
    ///  and the step 14 fixture names its origin snapshot `origin`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fixture = RemovalFixture.healthy('x').withOriginSnapshot(fullName);
    /// ```
    pub fn with_origin_snapshot(
        mut self,
        full: &str,
    ) -> RemovalFixture {
        let prefix: String = String::from("originSnapshot=");
        for line in &mut self.binding_lines {
            if line.starts_with(prefix.as_str()) {
                *line = format!("{prefix}{full}");
            }
        }
        self.origin_snapshot = String::from(full);
        self.write_the_binding();
        self.write_the_origin_inventory();
        return self;
    }

    /// Write the binding as its lines stand now.
    pub fn write_the_binding(&self) {
        let home: PathBuf = home_folder(self.common_dir.as_path());
        std::fs::create_dir_all(home.as_path()).expect("the binding home");
        let path: PathBuf = home.join(binding_name(SLOT));
        let mut text: String = String::new();
        for line in &self.binding_lines {
            text.push_str(line.as_str());
            text.push('\n');
        }
        std::fs::write(path, text).expect("the binding record");
    }

    /// Drop one binding line.
    pub fn drop_binding_line(
        &mut self,
        key: &str,
    ) {
        let prefix: String = format!("{key}=");
        let before: usize = self.binding_lines.len();
        self.binding_lines
            .retain(|line| return !line.starts_with(prefix.as_str()));
        assert_eq!(
            self.binding_lines.len(),
            before - 1,
            "the binding holds one `{key}` line"
        );
    }

    /// The origin-side inventory the add wrote.
    pub fn the_origin_inventory(&self) -> InventoryRecord {
        return InventoryRecord {
            kind: InventoryKind::Origin,
            pool_guid: guid(POOL_GUID),
            dataset: String::from(ORIGIN_DATASET),
            dataset_guid: guid(ORIGIN_DATASET_GUID),
            bounds: self.origin_snapshot.clone(),
            bounds_guid: guid(ORIGIN_SNAPSHOT_GUID),
            bounds_createtxg: ORIGIN_CREATETXG,
            entries: vec![InventoryEntry {
                kind: EntryKind::Snapshot,
                guid: guid(ORIGIN_SNAPSHOT_GUID),
                createtxg: ORIGIN_CREATETXG,
                name: String::from("origin"),
            }],
        };
    }

    /// Write the origin-side inventory under its own name.
    pub fn write_the_origin_inventory(&self) {
        let home: PathBuf = home_folder(self.common_dir.as_path());
        std::fs::create_dir_all(home.as_path()).expect("the binding home");
        let path: PathBuf = home.join(origin_inventory_name(SLOT));
        std::fs::write(path, self.the_origin_inventory().encode()).expect("the origin inventory");
    }

    /// The identities the removal reads from the written binding.
    pub fn recorded(&self) -> RecordedBinding {
        return read_the_recorded_binding(self.common_dir.as_path(), SLOT)
            .expect("the fixture's binding reads");
    }

    /// What:
    ///  The healthy pool:
    ///  the clone with two own snapshots and no bookmark,
    ///  its origin,
    ///  and an unmarked origin snapshot.
    /// Why:
    ///  The design's own healthy tuple,
    ///  which each refusal arm breaks in one place.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pool = fixture.healthyPool();
    /// ```
    pub fn healthy_pool(&self) -> ScriptedPool {
        let identities: Vec<(String, DatasetIdentity)> = vec![
            (
                String::from(POOL),
                DatasetIdentity {
                    stats: the_stats(POOL_GUID, false, 1, 0, ""),
                    properties: NvList::new(),
                },
            ),
            (
                String::from(CLONE),
                DatasetIdentity {
                    stats: the_stats(CLONE_GUID, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT),
                    properties: NvList::new(),
                },
            ),
            (
                String::from(ORIGIN_DATASET),
                DatasetIdentity {
                    stats: the_stats(ORIGIN_DATASET_GUID, false, 2, 1, ""),
                    properties: NvList::new(),
                },
            ),
            (
                String::from(ORIGIN_SNAPSHOT),
                DatasetIdentity {
                    stats: the_stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_CREATETXG, 1, ""),
                    properties: origin_properties(Some(0), Some(0)),
                },
            ),
        ];
        return ScriptedPool {
            identities,
            snapshots: vec![
                an_object(EntryKind::Snapshot, OWN_SNAPSHOT, OWN_GUID, OWN_CREATETXG),
                an_object(
                    EntryKind::Snapshot,
                    SANOID_SNAPSHOT,
                    SANOID_GUID,
                    SANOID_CREATETXG,
                ),
            ],
            bookmarks: Vec::new(),
            listing_refusal: None,
        };
    }

    /// The folder the removal writes its plan into,
    /// which the entry makes.
    pub fn plan_folder(&self) -> PathBuf {
        return crate::worktree_clone_binding_record::registration_home::plans_folder(
            self.common_dir.as_path(),
        );
    }

    /// Whether the plan folder holds anything.
    pub fn the_plan_folder_holds(&self) -> usize {
        let folder: PathBuf = self.plan_folder();
        return std::fs::read_dir(folder.as_path()).map_or(0, |entries| return entries.count());
    }

    /// The common directory,
    /// for a control that needs its path alone.
    pub fn common(&self) -> &Path {
        return self.common_dir.as_path();
    }
}
