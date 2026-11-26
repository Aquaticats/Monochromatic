//! What:
//!  The reader's freshness controls:
//!  the result of one request is accepted by the request that made it,
//!  while a result for another dataset,
//!  and a result for an earlier request of the same dataset,
//!  are each refused by name.
//! Why:
//!  A removal destroys snapshots on the strength of this answer,
//!  so an answer to another clone's question or to the previous scan of this
//!  clone must never be taken for a fresh one
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces",
//!  which asks for a fresh,
//!  nonce- and GUID-bound result).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(read(scan).accept({ dataset, nonce })).toEqual(expect.objectContaining({ kind: 'another-nonce' }));
//! ```

use super::super::holder_scan::HolderScan;
use super::super::holder_scan_parse::parse_holder_scan;
use super::{
    ScanExpectation,
    StaleResult,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The nonce the fixture's request carries.
const NONCE: &str = "00112233445566778899aabbccddeeff";

/// The nonce of an earlier request of the same dataset.
const OLD_NONCE: &str = "ffeeddccbbaa99887766554433221100";

/// One result written by a scan and read back by its caller,
/// which is the whole path from the handler to the reader.
fn read_back() -> HolderScan {
    let scan: HolderScan = HolderScan {
        dataset: DatasetGuid::parse("42").expect("the fixture GUID is canonical"),
        source: String::from("zpcachyos/nosnap/clones/one"),
        nonce: RequestNonce::parse(NONCE).expect("the fixture nonce is one"),
        own_namespace: Some(String::from("mnt:[7]")),
        namespaces_scanned: 1,
        own_mounts: Vec::new(),
        holders: Vec::new(),
        incomplete: Vec::new(),
        time: String::from("2026-11-22T16:00:00-05:00"),
    };
    return parse_holder_scan(scan.text().as_str())
        .expect("the result this route wrote reads back");
}

/// The request that made a result accepts it,
/// while another dataset's GUID and an earlier request's nonce are refused.
#[test]
fn a_result_for_another_dataset_or_an_older_request_is_refused() {
    let read: HolderScan = read_back();
    let dataset: DatasetGuid = DatasetGuid::parse("42").expect("the fixture GUID is canonical");
    let nonce: RequestNonce = RequestNonce::parse(NONCE).expect("the fixture nonce is one");
    assert_eq!(
        read.accept(&ScanExpectation {
            dataset,
            nonce: &nonce
        }),
        Ok(())
    );
    let other_dataset: DatasetGuid = DatasetGuid::parse("43").expect("the other GUID is canonical");
    match read.accept(&ScanExpectation {
        dataset: other_dataset,
        nonce: &nonce,
    }) {
        Err(StaleResult::ForeignDataset {
            expected,
            found,
        }) => {
            assert_eq!(expected.file_name(), String::from("43"));
            assert_eq!(found.file_name(), String::from("42"));
        }
        other => panic!("{other:?}"),
    }
    let old: RequestNonce = RequestNonce::parse(OLD_NONCE).expect("the older nonce is one");
    match read.accept(&ScanExpectation {
        dataset,
        nonce: &old,
    }) {
        Err(StaleResult::AnotherNonce {
            expected,
            found,
        }) => {
            assert_eq!(expected, OLD_NONCE);
            assert_eq!(found, NONCE);
        }
        other => panic!("{other:?}"),
    }
}

/// Each refusal names both values and says nothing was read from the result.
#[test]
fn the_refusals_name_both_values_and_what_to_do() {
    let foreign: String = StaleResult::ForeignDataset {
        expected: DatasetGuid::parse("43").expect("the expected GUID is canonical"),
        found: DatasetGuid::parse("42").expect("the found GUID is canonical"),
    }
    .message();
    assert!(foreign.contains("GUID 42"), "{foreign}");
    assert!(foreign.contains("GUID 43"), "{foreign}");
    assert!(foreign.contains("Nothing was read from it"), "{foreign}");
    let stale: String = StaleResult::AnotherNonce {
        expected: String::from(OLD_NONCE),
        found: String::from(NONCE),
    }
    .message();
    assert!(stale.contains(OLD_NONCE), "{stale}");
    assert!(
        stale.contains("no earlier scan answers a later request"),
        "{stale}"
    );
}
