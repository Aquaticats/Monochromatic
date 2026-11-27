//! What:
//!  The destruction plan a control plants in a fixture's plan folder:
//!  the plan a removal's first write leaves,
//!  or one whose head word and attempt lines the control names,
//!  published by the removal's own writer.
//! Why:
//!  A control that writes a plan by hand writes a plan the design's grammar may
//!  refuse,
//!  and a refused plan says nothing to the reader the control meant to feed:
//!  the slot-state guest arm planted the header and the clone's two fields alone,
//!  the add's reader refused it for the `head=` line it never carried,
//!  and every assertion after that plan went unread (issue 6595).
//!  `DestructionPlan::encode` and `PlanFolder::publish` are what a removal writes
//!  its own record with,
//!  and `publish` reads the record back through the grammar's parser before it
//!  lands,
//!  so a plan planted this way is one the grammar accepts
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The grammar").
//! Gotcha:
//!  The library's unit tests and the `native_binary` crate both include this file
//!  by `#[path]`,
//!  and each names the library's items its own way,
//!  so each crate declares the writer's own names as
//!  `the_destruction_plans_writer` and this file reaches them through `super::`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = aRemovalsFirstWrite(aFixtureClone(common, clone, guid));
//! const path = plant(plan); // <common>/monochromatic-git-policy-cli/clone-destruction-plans/<guid>
//! ```

use super::the_destruction_plans_writer::{
    DatasetGuid,
    DestructionPlan,
    HeadWord,
    PlanAttempt,
    PlanFacts,
    PlanFolder,
    RequestNonce,
    now_with_offset,
};
use std::path::{
    Path,
    PathBuf,
};

/// The removal run's own nonce a planted plan records,
/// which is the design's example record's.
const THE_REMOVAL_RUN: &str = "6f1c9d2a7b4e40a1b8c1d3e5f7091a2b";

/// The writer a planted plan names as its own,
/// spelled `<pid>:linux:<start time>`.
///
/// Why:
///  No removal wrote a control's fixture,
///  and the grammar checks this field's shape and nothing else,
///  so this is a stand-in for the process a plan records.
const THE_PROCESS_THAT_WROTE_IT: &str = "48213:linux:9481733";

/// The registration directory's identity a planted plan records,
/// `device:inode:seconds:nanoseconds`.
///
/// Why:
///  The identity belongs to a registration a fixture may never make,
///  so this is the design's example value,
///  and `publish` refuses the record when the grammar's `ObjectId::decode`
///  cannot read it.
const THE_REGISTRATION_IDENTITY: &str = "2049:131081:1763750412:987654321";

/// The unmount nonce a planted plan records,
/// which is the design's example record's.
const THE_UNMOUNT_NONCE: &str = "0a1b2c3d4e5f60718293a4b5c6d7e8f9";

/// The origin dataset every fixture plan names,
/// which the design's example record holds.
const THE_ORIGIN_DATASET: &str = "tank/checkouts/checkout";

/// The origin snapshot every fixture plan names,
/// which is a snapshot of [`THE_ORIGIN_DATASET`].
const THE_ORIGIN_SNAPSHOT: &str = "tank/checkouts/checkout@origin";

/// The GUID every fixture plan records for its origin dataset.
const THE_ORIGIN_DATASET_GUID: u64 = 4_815_162_342_109_876_543;

/// The GUID every fixture plan records for its origin snapshot.
const THE_ORIGIN_SNAPSHOT_GUID: u64 = 11_050_373_805_378_911_967;

/// The transaction group every fixture plan records for its origin snapshot,
/// which the design's example puts below the clone's own group.
const THE_ORIGIN_SNAPSHOTS_CREATION_GROUP: u64 = 15;

/// The transaction group a fixture plan records for a clone no pool made,
/// which is the design's own example clone's group.
const A_FIXTURE_CLONES_CREATION_GROUP: u64 = 17;

/// The clone a planted plan names,
/// and the origin its removal was made against.
pub struct ACloneAPlanNames {
    /// The repository's common Git directory,
    /// whose plan folder holds the record.
    pub(crate) common_dir: PathBuf,
    /// The clone's full dataset name.
    pub(crate) clone_dataset: String,
    /// The clone's own GUID,
    /// which is also the plan's key and file name.
    pub(crate) clone_guid: DatasetGuid,
    /// The clone's own creation transaction group.
    pub(crate) clone_createtxg: u64,
    /// The origin dataset the removal proved the clone against.
    pub(crate) origin_dataset: String,
    /// The origin snapshot the clone was made from.
    pub(crate) origin_snapshot: String,
}

impl ACloneAPlanNames {
    /// What:
    ///  The clone a control names by its dataset and its GUID,
    ///  against the origin every fixture plan records.
    /// Why:
    ///  A control whose subject is a reader asks only who the record names,
    ///  so the clone's creation group and its origin come from one place.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const clone = aFixtureClone(common, 'tank/clones/topic', guid);
    /// ```
    pub fn a_fixture_clone(
        common_dir: &Path,
        clone_dataset: &str,
        clone_guid: DatasetGuid,
    ) -> ACloneAPlanNames {
        return ACloneAPlanNames {
            common_dir: PathBuf::from(common_dir),
            clone_dataset: String::from(clone_dataset),
            clone_guid,
            clone_createtxg: A_FIXTURE_CLONES_CREATION_GROUP,
            origin_dataset: String::from(THE_ORIGIN_DATASET),
            origin_snapshot: String::from(THE_ORIGIN_SNAPSHOT),
        };
    }

    /// What:
    ///  The slot this clone's path ends in,
    ///  which is the name the plan's registration bytes and its target path hold.
    /// Why:
    ///  The design puts the clone's dataset under the container and the slot under
    ///  the worktrees folder,
    ///  so the last component of the one name is the last component of the other.
    fn slot(&self) -> &str {
        return self
            .clone_dataset
            .rsplit_once('/')
            .map(|(_container, slot)| return slot)
            .unwrap_or(self.clone_dataset.as_str());
    }

    /// The slot's whole path,
    /// which the design's `targetHex` field holds.
    fn target(&self) -> PathBuf {
        return self
            .common_dir
            .parent()
            .unwrap_or(self.common_dir.as_path())
            .join(".claude")
            .join("worktrees")
            .join(self.slot());
    }

    /// The identities and the verified inventory this clone's removal records.
    fn facts(&self) -> PlanFacts {
        return PlanFacts {
            run: RequestNonce::parse(THE_REMOVAL_RUN).expect("the fixture run nonce reads"),
            owner: String::from(THE_PROCESS_THAT_WROTE_IT),
            started: now_with_offset(),
            clone_dataset: self.clone_dataset.clone(),
            clone_guid: self.clone_guid,
            clone_createtxg: self.clone_createtxg,
            origin_dataset: self.origin_dataset.clone(),
            origin_dataset_guid: a_guid(THE_ORIGIN_DATASET_GUID),
            origin_snapshot: self.origin_snapshot.clone(),
            origin_snapshot_guid: a_guid(THE_ORIGIN_SNAPSHOT_GUID),
            origin_snapshot_createtxg: THE_ORIGIN_SNAPSHOTS_CREATION_GROUP,
            registration_id: Vec::from(self.slot().as_bytes()),
            admin: String::from(THE_REGISTRATION_IDENTITY),
            target: self.target(),
            unmount_nonce: RequestNonce::parse(THE_UNMOUNT_NONCE)
                .expect("the fixture unmount nonce reads"),
            unmount_mount_id: None,
        };
    }
}

/// The plan a control plants:
/// the clone it names,
/// the head destroy's state and every line of every snapshot's history.
pub struct APlanAControlPlants {
    /// The clone the plan names.
    pub(crate) clone: ACloneAPlanNames,
    /// The head destroy's state.
    pub(crate) head: HeadWord,
    /// Every line of every snapshot's history,
    /// in the order the file writes them.
    pub(crate) attempts: Vec<PlanAttempt>,
}

impl APlanAControlPlants {
    /// What:
    ///  The plan a removal's first write leaves:
    ///  the head still present and one line for every snapshot of an empty
    ///  history,
    ///  which is the state a removal that destroyed nothing records.
    /// Why:
    ///  This is the state the design's own first write gives
    ///  (`DestructionPlan::first_write`),
    ///  so a control whose subject is a slot with a removal in flight plants the
    ///  record such a removal would have left.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const plan = aRemovalsFirstWrite(clone); // head present, no attempt line
    /// ```
    pub fn a_removals_first_write(clone: ACloneAPlanNames) -> APlanAControlPlants {
        return APlanAControlPlants {
            clone,
            head: HeadWord::Present,
            attempts: Vec::new(),
        };
    }
}

/// What:
///  Publish one planted plan into its clone's repository,
///  under the clone's GUID,
///  and answer the file it landed in.
/// Why:
///  A plan's name is its key,
///  so the removal's own publication is what files it where every reader looks,
///  and what refuses a record its own reader would refuse.
///
/// # Panics
/// The folder the plan lives in cannot be made,
/// or the record this build writes is one this build refuses,
/// which is a defect of the fixture and never a state a control means to plant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = plant(plan);
/// ```
pub fn plant(plan: &APlanAControlPlants) -> PathBuf {
    let folder: PlanFolder = PlanFolder::open_or_create(plan.clone.common_dir.as_path())
        .expect("the fixture's plan folder opens");
    return folder
        .publish(&DestructionPlan {
            facts: plan.clone.facts(),
            head: plan.head,
            attempts: plan.attempts.clone(),
        })
        .expect("the planted plan is one this build's reader accepts");
}

/// One dataset GUID built from the decimal number a fixture names.
pub(crate) fn a_guid(value: u64) -> DatasetGuid {
    return DatasetGuid::parse(value.to_string().as_str())
        .expect("a decimal number is a canonical dataset GUID");
}
