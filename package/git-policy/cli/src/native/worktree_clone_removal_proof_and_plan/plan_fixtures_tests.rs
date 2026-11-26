//! What:
//!  The plan every control of this folder starts from,
//!  and the small file helpers they need.
//! Why:
//!  One record built from the design's own example keeps the controls about the grammar rather
//!  than about the numbers,
//!  and a control that plants a broken form changes exactly one thing from it
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The grammar").
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = samplePlan();
//! const broken = replaceLine(plan.encode(), 'head=present', 'head=elsewhere');
//! ```

use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanFacts,
    PlanSnapshot,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use std::path::PathBuf;

/// The clone's GUID the sample plan is filed under.
pub const SAMPLE_CLONE_GUID: u64 = 7_828_944_690_444_299_989;
/// The origin dataset's GUID the sample plan records.
pub const SAMPLE_ORIGIN_DATASET_GUID: u64 = 4_815_162_342_109_876_543;
/// The origin snapshot's GUID the sample plan records.
pub const SAMPLE_ORIGIN_SNAPSHOT_GUID: u64 = 11_050_373_805_378_911_967;
/// The clone's own transaction group.
pub const SAMPLE_CLONE_CREATETXG: u64 = 17;
/// The origin snapshot's transaction group.
pub const SAMPLE_ORIGIN_CREATETXG: u64 = 15;
/// The first own snapshot's GUID.
pub const SAMPLE_OWN_GUID: u64 = 17_157_080_670_377_245_572;
/// The sanoid snapshot's GUID.
pub const SAMPLE_SANOID_GUID: u64 = 9_332_110_456_720_018_274;

/// One dataset GUID built from a decimal number,
/// which is what the kernel reports.
pub fn guid(value: u64) -> DatasetGuid {
    return DatasetGuid::parse(value.to_string().as_str()).expect("a number is a canonical GUID");
}

/// One request nonce built from its digits.
pub fn nonce(digits: &str) -> RequestNonce {
    return RequestNonce::parse(digits).expect("the digits are a nonce");
}

/// The two own snapshots the sample plan verifies.
pub fn sample_snapshots() -> Vec<PlanSnapshot> {
    return vec![
        PlanSnapshot {
            name: String::from("tank/wt/fix-flaky-test@worktree-own"),
            guid: guid(SAMPLE_OWN_GUID),
            createtxg: 18,
        },
        PlanSnapshot {
            name: String::from("tank/wt/fix-flaky-test@sanoid-hourly"),
            guid: guid(SAMPLE_SANOID_GUID),
            createtxg: 20,
        },
    ];
}

/// The identities the sample plan records.
pub fn sample_facts() -> PlanFacts {
    return PlanFacts {
        run: nonce("6f1c9d2a7b4e40a1b8c1d3e5f7091a2b"),
        owner: String::from("48213:linux:9481733"),
        started: String::from("2026-11-22T18:40:12-05:00"),
        clone_dataset: String::from("tank/wt/fix-flaky-test"),
        clone_guid: guid(SAMPLE_CLONE_GUID),
        clone_createtxg: SAMPLE_CLONE_CREATETXG,
        origin_dataset: String::from("tank/mono-checkout"),
        origin_dataset_guid: guid(SAMPLE_ORIGIN_DATASET_GUID),
        origin_snapshot: String::from("tank/mono-checkout@origin"),
        origin_snapshot_guid: guid(SAMPLE_ORIGIN_SNAPSHOT_GUID),
        origin_snapshot_createtxg: SAMPLE_ORIGIN_CREATETXG,
        registration_id: Vec::from("fix-flaky-test".as_bytes()),
        admin: String::from("2049:131081:1763750412:987654321"),
        target: PathBuf::from("/home/u/repo/.claude/worktrees/fix-flaky-test"),
        unmount_nonce: nonce("0a1b2c3d4e5f60718293a4b5c6d7e8f9"),
        unmount_mount_id: None,
    };
}

/// The sample plan as its first write leaves it.
pub fn sample_plan() -> DestructionPlan {
    let mut plan: DestructionPlan =
        DestructionPlan::first_write(sample_facts(), sample_snapshots().as_slice());
    plan.head = HeadWord::Present;
    return plan;
}

/// The sample plan's bytes,
/// which every control that plants a malformed form edits.
pub fn sample_bytes() -> Vec<u8> {
    return sample_plan()
        .encode()
        .expect("the sample plan is inside every bound");
}

/// The sample plan's bytes with one text replaced,
/// which is how a control plants exactly one broken form.
pub fn replaced(
    bytes: &[u8],
    from: &str,
    to: &str,
) -> Vec<u8> {
    let text: String = String::from_utf8(bytes.to_vec()).expect("the sample is text");
    assert!(text.contains(from), "the sample holds `{from}`");
    return text.replacen(from, to, 1).into_bytes();
}
