//! What:
//!  The device-free proof that every operation carries the request number the
//!  design quotes from OpenZFS `zfs-2.4.4`,
//! that the numbers are unique,
//! and
//!  that the two queries (changes state,
//! drives a listing) answer the design's
//!  own classification.
//! Why:
//!  The number is the whole request:
//! a changed digit sends a snapshot where a
//!  destroy was meant,
//! and no device-free test could see it from the fields.

use super::*;

#[test]
fn every_request_number_is_the_openzfs_one() {
    let expected: [(Operation, u64); 12] = [
        (Operation::ObjsetStats, 0x5a12),
        (Operation::DatasetListNext, 0x5a14),
        (Operation::SnapshotListNext, 0x5a15),
        (Operation::SetProp, 0x5a16),
        (Operation::Destroy, 0x5a18),
        (Operation::Promote, 0x5a22),
        (Operation::Snapshot, 0x5a23),
        (Operation::GetFsacl, 0x5a29),
        (Operation::DestroySnaps, 0x5a3b),
        (Operation::Clone, 0x5a42),
        (Operation::GetBookmarks, 0x5a44),
        (Operation::ChannelProgram, 0x5a48),
    ];
    for (operation, code) in expected {
        assert_eq!(operation.code(), code, "{operation} request number");
    }
}

#[test]
fn no_two_operations_share_a_number() {
    let mut seen: Vec<u64> = Vec::new();
    for operation in Operation::ALL {
        assert!(
            !seen.contains(&operation.code()),
            "{operation} repeats a number"
        );
        seen.push(operation.code());
    }
    assert_eq!(seen.len(), Operation::ALL.len());
}

#[test]
fn every_operation_appears_once_in_the_order() {
    for operation in Operation::ALL {
        let count: usize = Operation::ALL
            .iter()
            .filter(|candidate| {
                return **candidate == operation;
            })
            .count();
        assert_eq!(count, 1, "{operation} appears once");
    }
}

#[test]
fn the_state_changing_split_is_the_designs() {
    let changing: Vec<Operation> = Operation::ALL
        .iter()
        .copied()
        .filter(|operation| {
            return operation.mutates();
        })
        .collect();
    assert_eq!(
        changing,
        [
            Operation::SetProp,
            Operation::Destroy,
            Operation::Promote,
            Operation::Snapshot,
            Operation::DestroySnaps,
            Operation::Clone,
            Operation::ChannelProgram,
        ]
    );
}

#[test]
fn only_the_two_cur_requests_list() {
    let listing: Vec<Operation> = Operation::ALL
        .iter()
        .copied()
        .filter(|operation| {
            return operation.lists();
        })
        .collect();
    assert_eq!(
        listing,
        [Operation::DatasetListNext, Operation::SnapshotListNext]
    );
}

#[test]
fn a_diagnostic_prints_the_c_name() {
    assert_eq!(Operation::DestroySnaps.as_str(), "DESTROY_SNAPS");
    assert_eq!(Operation::DestroySnaps.to_string(), "DESTROY_SNAPS");
    assert_eq!(Operation::ChannelProgram.to_string(), "CHANNEL_PROGRAM");
}
