//! What:
//!  The three duties every reader owes the declaration,
//! each one control through the
//!  entry point:
//! a record whose kind flag is set and whose widths are zero is no bitfield,
//!  a typedef that names itself refuses instead of looping,
//! and a member whose name cannot be read
//!  refuses instead of vanishing from the list.
//! Why:
//!  The project has not picked the profile reader,
//! and the adapter that stands
//!  in for it goes with the pick,
//! so each duty needs a control written against
//!  `check_raw`:
//! a fixture this folder's writer builds and a verdict,
//! and nothing from
//!  the decoder the pick deletes.
//!  A reader that breaks a duty is refused by these
//!  controls,
//! which is how the declaration's contract outlives the stand-in adapter.

use super::approved_module_fixture::{
    COMMAND_BYTES, EMPTY_BASE_STRINGS, module_with_an_unreadable_extra_member, module_with_the_kind_flag_set,
};
use super::blob_fixture::{Blob, WrittenMember, check_pair};
use super::refusal::Step;
use super::{Compatible, Refusal};

/// The gate's verdict for one module file over the empty base file
/// every writer here starts from.
fn verdict(name: &str, module: &Blob) -> Result<Compatible, Refusal> {
    return check_pair(name, &Blob::new(), module);
}

/// The refusal one module file must get.
fn refused(name: &str, module: &Blob) -> Refusal {
    let answer: Result<Compatible, Refusal> = verdict(name, module);
    let Err(refusal): Result<Compatible, Refusal> = answer else {
        panic!("the {name} module passed the gate");
    };
    return refusal;
}

/// A module whose `zfs_cmd` carries one member,
/// `zc_name`,
/// whose type is a typedef that names itself.
fn module_with_a_self_naming_typedef() -> Blob {
    let mut module: Blob = Blob::over(EMPTY_BASE_STRINGS);
    let chain: u32 = module.self_alias("zc_objset_stats");
    module.structure(
        "zfs_cmd",
        COMMAND_BYTES,
        &[WrittenMember {
            name: "zc_name",
            type_id: chain,
            bit_offset: 0,
        }],
    );
    return module;
}

/// A zero width is no bitfield:
/// the approved layout passes with every record's kind flag set,
/// which is the shape a whole member's width of zero takes.
///
/// A reader that answers a zero-width bitfield refuses here,
/// naming a member of a record whose layout never moved.
#[test]
fn a_record_whose_kind_flag_is_set_and_whose_widths_are_zero_passes() {
    let module: Blob = module_with_the_kind_flag_set();
    let answer: Result<Compatible, Refusal> = verdict("contract-kind-flag", &module);
    let Ok(compatible): Result<Compatible, Refusal> = answer else {
        panic!("the kind-flagged approved declaration refused: {}", answer.err().unwrap());
    };
    assert_eq!(compatible.module_bytes, module.bytes().len());
}

/// An alias chain is bounded:
/// a typedef that names itself ends in a refusal,
/// and the call returns rather than looping.
///
/// A reader that walks aliases without a bound never answers here.
#[test]
fn a_typedef_that_names_itself_refuses_and_returns() {
    let refusal: Refusal = refused("contract-alias-loop", &module_with_a_self_naming_typedef());
    assert_eq!(refusal.step, Step::BaseType);
    assert!(refusal.detail.contains("zc_name"), "{}", refusal.detail);
    assert!(refusal.detail.contains("cannot compute"), "{}", refusal.detail);
}

/// An unreadable member is a refusal,
/// never a member dropped:
/// the approved layout with one more member whose name no reader can read
/// refuses naming the record it sits in,
/// where a dropped member would leave the count the comparison accepts.
#[test]
fn a_member_whose_name_cannot_be_read_refuses() {
    let module: Blob = module_with_an_unreadable_extra_member();
    let refusal: Refusal = refused("contract-unreadable-member", &module);
    assert_eq!(refusal.step, Step::BaseType);
    assert!(refusal.detail.contains("zc_objset_stats"), "{}", refusal.detail);
    assert!(refusal.detail.contains("cannot compute"), "{}", refusal.detail);
}
