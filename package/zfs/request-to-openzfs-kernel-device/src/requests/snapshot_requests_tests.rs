//! What:
//!  The device-free proof of the two snapshot writes:
//! both carry a `snaps` list
//!  of boolean flags,
//! the create request carries optional `props`,
//! and the
//!  destroy request carries no `defer` member at all.
//! Why:
//!  The absence of `defer` is the design's decision:
//! a deferred destroy marks a
//!  snapshot the removal's later accounting would still meet.
//! Asserting it on
//!  the encoder is where the request the kernel receives is decided.

use super::*;
use crate::nvlist::{NvData, NvList};
use crate::requests::Operation;

#[test]
fn snapshot_carries_each_name_as_a_flag() {
    let request: Request =
        snapshot("pool", &["pool/ds@s1", "pool/ds@s2"], None).expect("the name fits");
    assert_eq!(request.operation(), Operation::Snapshot);
    assert_eq!(request.command().name(), "pool");
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    let snaps: &NvList = input
        .get_list("snaps")
        .expect("the request carries a snaps list");
    assert_eq!(snaps.get("pool/ds@s1"), Some(&NvData::BooleanFlag));
    assert_eq!(snaps.get("pool/ds@s2"), Some(&NvData::BooleanFlag));
    assert_eq!(input.get("props"), None);
}

#[test]
fn snapshot_carries_its_optional_properties() {
    let mut properties: NvList = NvList::new();
    properties.add_str("monochromatic:purpose", "before-merge");
    let request: Request =
        snapshot("pool", &["pool/ds@s1"], Some(&properties)).expect("the name fits");
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    assert_eq!(input.get_list("props"), Some(&properties));
}

#[test]
fn destroy_snaps_carries_each_name_as_a_flag() {
    let request: Request =
        destroy_snaps("pool", &["pool/ds@s1", "pool/ds@s2"]).expect("the name fits");
    assert_eq!(request.operation(), Operation::DestroySnaps);
    assert_eq!(request.command().name(), "pool");
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    let snaps: &NvList = input
        .get_list("snaps")
        .expect("the request carries a snaps list");
    assert_eq!(snaps.pairs.len(), 2);
}

#[test]
fn the_encoded_destroy_snaps_request_carries_no_defer_key() {
    let request: Request =
        destroy_snaps("pool", &["pool/ds@s1", "pool/ds@s2"]).expect("the name fits");
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    assert_eq!(
        input.get("defer"),
        None,
        "a deferred destroy would mark a snapshot the removal then cannot account for"
    );
    // The packed bytes are what the kernel reads: no pair anywhere in them may
    // be named `defer`, including inside the `snaps` list.
    assert!(
        !request
            .input_bytes()
            .windows("defer\0".len())
            .any(|window| {
                return window == b"defer\0";
            }),
        "the encoded bytes must not name a defer key"
    );
}

#[test]
fn a_nul_in_a_snapshot_name_is_refused_while_packing() {
    let error = destroy_snaps("pool", &["pool/ds@s1\0"]).expect_err("a NUL would shorten the name");
    assert_eq!(error.operation(), Operation::DestroySnaps);
    assert!(error.to_string().contains("embedded NUL"));
}
