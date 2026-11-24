//! What:
//!  Controls for the one snapshot-destroy request:
//! its operation,
//! its pool,
//! its
//!  `snaps` list,
//! the absence of `defer` in the bytes the kernel reads,
//! and the
//!  detector that would see the key if one were added.
//! Why:
//!  The absence of `defer` is the design's decision,
//! and the host control must be
//!  able to fail:
//! the last control here adds the key to a request and shows the
//!  detector report it,
//! so the green control proves the bytes rather than the
//!  detector's optimism.
//!  These controls are device-free:
//! a request is built and its own input
//!  bytes are read,
//! which is exactly what the kernel reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const request = destroyRequest('pool', names);
//! expect(carriesNoDefer(request)).toBe(true);
//! expect(carriesNoDefer(requestWithDefer())).toBe(false);
//! ```

use crate::worktree_clone_snapshot_batch::request_refusal::BatchRefusal;
use crate::worktree_clone_snapshot_batch::snapshot_destroy_request::{DEFER_KEY, carries_no_defer, destroy_request, encoded_request_bytes};
use monochromatic_zfs_request_to_openzfs_kernel_device::requests::{checked_name, flag_list};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    NvData, NvList, NameField, Operation, Request,
};

/// The pool the controls' requests name.
const POOL: &str = "zfsstep16";

/// The full snapshot names one verified set holds.
fn names() -> Vec<String> {
    return ["one", "two", "three"]
        .iter()
        .map(|suffix| {
            return format!("{POOL}/clones/removal@{suffix}");
        })
        .collect();
}

/// The request names the destroy-snaps operation,
/// the pool and every snapshot.
#[test]
fn the_request_is_one_destroy_snaps_request_over_the_pool() {
    let set: Vec<String> = names();
    let request: Request = destroy_request(POOL, set.as_slice()).expect("the names fit");
    assert_eq!(request.operation(), Operation::DestroySnaps);
    assert_eq!(request.command().name(), POOL);
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    let snaps: &NvList = input.get_list("snaps").expect("the request carries a snaps list");
    let submitted: Vec<String> = snaps
        .iter()
        .map(|pair| {
            return pair.name.clone();
        })
        .collect();
    assert_eq!(submitted, set);
    assert!(snaps.iter().all(|pair| {
        return pair.data == NvData::BooleanFlag;
    }));
    assert_eq!(input.get(DEFER_KEY), None);
}

/// The encoded request carries no `defer` key at all.
#[test]
fn the_encoded_request_carries_no_defer_key() {
    let request: Request = destroy_request(POOL, names().as_slice()).expect("the names fit");
    assert!(
        carries_no_defer(&request),
        "a deferred destroy would mark a snapshot the removal cannot account for"
    );
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    assert_eq!(input.get(DEFER_KEY), None);
    // The list the kernel reads holds the pool's `snaps` list and nothing else, so no other
    //  top-level key can be mistaken for `defer`.
    let top_level: Vec<String> = input
        .iter()
        .map(|pair| {
            return pair.name.clone();
        })
        .collect();
    assert_eq!(top_level, vec![String::from("snaps")]);
}

/// The detector sees a `defer` key when a request carries one.
///
/// This is the green control's own red half:
/// without it a detector that always
///  answered `true` would pass the control above.
#[test]
fn the_defer_detector_sees_a_key_when_one_is_present() {
    let name = checked_name(
        Operation::DestroySnaps,
        POOL,
        NameField::CommandName,
        POOL,
    )
    .expect("the pool fits");
    let mut input: NvList = NvList::new();
    input.add_nvlist("snaps", flag_list(&["zfsstep16/clones/removal@one"]));
    input.push(DEFER_KEY, NvData::BooleanFlag);
    let mut request: Request = Request::new(Operation::DestroySnaps, name);
    request.attach_input(&input).expect("the list packs");
    assert!(
        !carries_no_defer(&request),
        "a request carrying the key must never read as defer-free"
    );
}

/// A snapshot name carrying a NUL is refused before a request exists.
#[test]
fn a_snapshot_name_with_an_embedded_nul_is_refused() {
    let refusal: BatchRefusal = destroy_request(POOL, &[String::from("zfsstep16/clones/removal@on\0e")])
        .expect_err("a NUL would shorten the name the kernel reads");
    assert!(matches!(refusal, BatchRefusal::Unbuildable { .. }));
}

/// The encoded size is the length of the bytes the kernel would receive.
#[test]
fn the_encoded_size_is_the_requests_own_length() {
    let set: Vec<String> = names();
    let request: Request = destroy_request(POOL, set.as_slice()).expect("the names fit");
    assert_eq!(
        encoded_request_bytes(POOL, set.as_slice()).expect("the names fit"),
        request.input_bytes().len()
    );
}
