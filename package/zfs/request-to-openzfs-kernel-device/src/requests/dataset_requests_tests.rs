//! What:
//!  The device-free proof of the four dataset writes:
//! properties travel in the
//!  input list,
//! the defer mark travels in the command union,
//! promote names only
//!  the clone,
//! and clone carries the origin and optional properties inside its
//!  input list.
//! Why:
//!  A defer mark set on the wrong request,
//! or an origin packed as a name field
//!  instead of a list member,
//! would make the kernel answer with an error the
//!  caller could not map back to its own input.

use super::*;
use crate::nvlist::NvList;
use crate::requests::Operation;

#[test]
fn set_prop_carries_the_properties_in_the_input_list() {
    let mut properties: NvList = NvList::new();
    properties.add_str("monochromatic:desired-state", "mounted");
    let request: Request = set_prop("pool/dataset", &properties).expect("the name fits");
    assert_eq!(request.operation(), Operation::SetProp);
    assert_eq!(request.command().name(), "pool/dataset");
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    assert_eq!(input, properties);
}

#[test]
fn destroy_leaves_the_defer_mark_clear_unless_asked() {
    let request: Request = destroy("pool/dataset", false).expect("the name fits");
    assert_eq!(request.command().defer_destroy(), 0);
    assert_eq!(request.operation(), Operation::Destroy);
}

#[test]
fn destroy_sets_the_defer_mark_when_asked() {
    let request: Request = destroy("pool/dataset", true).expect("the name fits");
    assert_eq!(request.command().defer_destroy(), 1);
    assert_eq!(request.command().flags(), 0);
}

#[test]
fn promote_names_only_the_clone() {
    let request: Request = promote("pool/clone").expect("the name fits");
    assert_eq!(request.operation(), Operation::Promote);
    assert_eq!(request.command().name(), "pool/clone");
    assert_eq!(request.command().zc_nvlist_src, 0);
    assert_eq!(request.command().defer_destroy(), 0);
}

#[test]
fn clone_carries_the_origin_and_the_properties() {
    let mut properties: NvList = NvList::new();
    properties.add_boolean("monochromatic:clone", true);
    let request: Request =
        clone("pool/clone", "pool/origin@snapshot", Some(&properties)).expect("the name fits");
    assert_eq!(request.operation(), Operation::Clone);
    assert_eq!(request.command().name(), "pool/clone");
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    assert_eq!(input.get_str("origin"), Some("pool/origin@snapshot"));
    assert_eq!(input.get_list("props"), Some(&properties));
}

#[test]
fn clone_without_properties_omits_the_props_member() {
    let request: Request =
        clone("pool/clone", "pool/origin@snapshot", None).expect("the name fits");
    let input: NvList = NvList::unpack(request.input_bytes()).expect("the input decodes");
    assert_eq!(input.get_str("origin"), Some("pool/origin@snapshot"));
    assert_eq!(input.get("props"), None);
}

#[test]
fn a_nul_in_the_origin_is_refused_while_packing() {
    let error = clone("pool/clone", "pool/origin\0@snapshot", None)
        .expect_err("a NUL would shorten the origin");
    assert_eq!(error.operation(), Operation::Clone);
    assert!(error.to_string().contains("embedded NUL"));
}

#[test]
fn a_name_too_long_for_zc_name_is_refused() {
    let long: String = "d".repeat(crate::layout::MAXPATHLEN);
    let error = destroy(&long, false).expect_err("zc_name cannot hold it");
    assert_eq!(error.operation(), Operation::Destroy);
    assert_eq!(error.errno(), Some(libc::ENAMETOOLONG));
}

#[test]
fn the_defer_mark_reads_back_zero_on_a_request_that_never_set_it() {
    let request: Request = set_prop("pool", &NvList::new()).expect("the name fits");
    assert_eq!(request.command().defer_destroy(), 0);
    assert_eq!(request.command().flags(), 0);
    assert_eq!(request.command().action_handle(), 0);
}
