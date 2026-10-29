//! What:
//!      Unit tests for fuzzer-driven address selection.
//! Why:
//!       The edit target only exercises edits if the addresses it draws actually resolve.
//!  A
//!           picker that returned mostly missing addresses would turn that campaign into a
//!           not-found test and hide the interesting failures.

/// What:
///      Import the unstructured-input type.
/// Why:
///       Tests drive selection from explicit byte buffers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Unstructured } from 'arbitrary';
/// ```
use arbitrary::Unstructured;

/// What:
///      Import the picker under test.
/// Why:
///       Its guarantees are resolution,
///  bounded length and coverage of both segment kinds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { randomPath } from './pathPick';
/// ```
use crate::path_pick::random_path;

/// What:
///      Import the crate's address model and queries.
/// Why:
///       Resolution is checked with the same `jsonc_has` a consumer would call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { jsoncHas, parseJsonc, JsoncPathSegment } from 'monochromatic-jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{jsonc_has, parse_jsonc, JsoncPathSegment};

/// What:
///      Draw one address from a seed.
/// Why:
///       Every case needs a fresh budget,
///  and the seed makes the draw reproducible.
fn draw(document: &monochromatic_jsonc_edit::JsoncValue, seed: &[u8]) -> Vec<JsoncPathSegment> {
    let mut unstructured = Unstructured::new(seed);
    return random_path(document, &mut unstructured).expect("selection succeeds");
}

/// Every drawn address must resolve,
///  and must stay within the documented length cap.
#[test]
fn drawn_addresses_resolve_and_stay_bounded() {
    let document = parse_jsonc("{\"a\":{\"b\":[10,{\"c\":true}]},\"d\":null}").expect("test document parses");
    let mut saw_key = false;
    let mut saw_index = false;
    let mut drew = 0;
    // Three varying bytes, not one: the walk spends a byte per descend decision and per index, so
    // a seed that fixes the later bytes can never reach the array and the coverage assertion below
    // would report a picker defect that is really a seed defect.
    for first in 0u8..=255 {
        for second in [0u8, 1, 2, 9, 250] {
            for third in [0u8, 1, 128, 255] {
            let path = draw(&document, &[first, second, third, third | 1, second | 1, 255, 7, 8]);
            assert!(path.len() <= 6, "drawn address exceeded the cap: {path:?}");
            assert!(jsonc_has(&document, &path), "drawn address does not resolve: {path:?}");
            for segment in &path {
                if matches!(segment, JsoncPathSegment::Key { .. }) {
                    saw_key = true;
                }
                if matches!(segment, JsoncPathSegment::Index { .. }) {
                    saw_index = true;
                }
            }
            drew += 1;
            }
        }
    }
    assert!(drew > 1000, "enumeration covered only {drew} seeds");
    assert!(saw_key, "no key segment was ever drawn");
    assert!(saw_index, "no index segment was ever drawn");
}

/// An empty container has no members,
///  so the only address is the root.
#[test]
fn empty_containers_yield_the_root_address() {
    for source in ["{}", "[]"] {
        let document = parse_jsonc(source).expect("test document parses");
        for seed in [[0u8, 0, 0], [255, 255, 255], [1, 0, 255]] {
            let path = draw(&document, &seed);
            assert!(path.is_empty(), "{source} produced a non-root address: {path:?}");
        }
    }
}
