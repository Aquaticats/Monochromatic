// Copyright (c) 2026 Mikko Tanner. All rights reserved.
// Licensed under the MIT License or the Apache License, Version 2.0.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Adapted from `fuzz/fuzz_targets/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at
// commit 4559ffc711; see the crate's `PROVENANCE.md`. Changes: the property,
// stats and vdev decoders are gone because this crate ships only the codec, and
// the walk descends nested lists so every decoded list is packed again.

/*!
`NvList::unpack`,
then `pack` on whatever decoded.
Whatever the bytes,
the
decoder must either refuse them cleanly or produce a list whose own packing
decodes and re-packs to the same bytes (compared as bytes,
not `PartialEq`,
because a NaN double never equals itself).
A decoded list must pack:
the
decoder cannot produce a name or string with an interior NUL,
nor a nesting
deeper than the bound both sides share.
*/

#![no_main]

use libfuzzer_sys::fuzz_target;
use monochromatic_zfs_request_to_openzfs_kernel_device::{NvData, NvList};

/// Assert the fixed point for `list`,
/// then for every list nested inside it.
fn check_round_trip(list: &NvList) {
    let packed: Vec<u8> = list.pack().expect("a decoded list must pack again");
    let again: NvList = NvList::unpack(&packed).expect("our own packing must decode");
    let repacked: Vec<u8> = again.pack().expect("a decoded packing must re-pack");
    assert!(packed == repacked, "pack is not a fixed point");
    for pair in list.iter() {
        match &pair.data {
            NvData::List(inner) => check_round_trip(inner),
            NvData::ListArray(lists) => {
                for inner in lists {
                    check_round_trip(inner);
                }
            }
            _ => {}
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(list) = NvList::unpack(data) else {
        return;
    };
    check_round_trip(&list);
});
