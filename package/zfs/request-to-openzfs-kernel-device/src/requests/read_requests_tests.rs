//! What:
//!  The device-free proof of the five read requests:
//! the operation,
//! the target
//!  in `zc_name`,
//! the cursor in `zc_cookie`,
//! and the bookmark property list as
//!  flags in the input.
//! Why:
//!  A listing that replayed the wrong cursor would skip or repeat entries,
//! and
//!  a bookmark read that passed no property names returns a different shape than
//!  the one the caller asked for.

use super::*;
use crate::nvlist::{NvData, NvList};

#[test]
fn objset_stats_names_the_dataset_and_carries_no_input() {
    let request: Request = objset_stats("pool/dataset").expect("the name fits");
    assert_eq!(request.operation(), Operation::ObjsetStats);
    assert_eq!(request.command().name(), "pool/dataset");
    assert_eq!(request.command().zc_cookie, 0);
    assert_eq!(request.command().zc_nvlist_src, 0);
}

#[test]
fn a_dataset_listing_carries_its_cursor() {
    let request: Request = dataset_list_next("pool/dataset", 17).expect("the name fits");
    assert_eq!(request.operation(), Operation::DatasetListNext);
    assert_eq!(request.command().zc_cookie, 17);
    assert_eq!(request.command().name(), "pool/dataset");
}

#[test]
fn a_snapshot_listing_carries_its_cursor() {
    let request: Request = snapshot_list_next("pool/dataset", 3).expect("the name fits");
    assert_eq!(request.operation(), Operation::SnapshotListNext);
    assert_eq!(request.command().zc_cookie, 3);
}

#[test]
fn a_bookmark_read_passes_each_property_as_a_flag() {
    let request: Request =
        get_bookmarks("pool/dataset", &["guid", "createtxg"]).expect("the name fits");
    assert_eq!(request.operation(), Operation::GetBookmarks);
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    assert_eq!(input.get("guid"), Some(&NvData::BooleanFlag));
    assert_eq!(input.get("createtxg"), Some(&NvData::BooleanFlag));
    assert_eq!(input.pairs.len(), 2);
}

#[test]
fn a_bookmark_read_without_properties_carries_no_input() {
    let request: Request = get_bookmarks("pool/dataset", &[]).expect("the name fits");
    assert!(request.input_bytes().is_empty());
    assert_eq!(request.command().zc_nvlist_src, 0);
}

#[test]
fn the_delegation_read_names_the_dataset() {
    let request: Request = get_fsacl("pool/dataset").expect("the name fits");
    assert_eq!(request.operation(), Operation::GetFsacl);
    assert_eq!(request.command().name(), "pool/dataset");
}

#[test]
fn a_target_too_long_for_zc_name_is_refused_before_a_request_exists() {
    let long: String = "d".repeat(crate::layout::MAXPATHLEN);
    let error = objset_stats(&long).expect_err("zc_name cannot hold it");
    assert_eq!(error.operation(), Operation::ObjsetStats);
    assert_eq!(error.errno(), Some(libc::ENAMETOOLONG));
}
