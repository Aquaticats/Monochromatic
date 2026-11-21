//! What:
//!  The `btf-rs` adapter's own controls:
//! an array's byte size,
//! which the reader
//!  leaves to this crate,
//! the one file the reader itself refuses,
//! and the five branches the
//!  reader's own structure hands this file:
//! a member typed and named by the base
//!  file,
//! a typedef chain and a qualifier chain over a member's type,
//! a base file the
//!  reader refuses,
//! and a negative enumerator whose sign the comparison prints.
//! Why:
//!  `btf-rs` states an array's element and count and its byte size nowhere,
//! so the
//!  multiplication is this crate's arithmetic and needs the same controls as
//! any other:
//!  the approved array passes,
//! one element short refuses at the member's size,
//! and
//!  the same bytes as wider elements refuse at the element's size,
//! which together say
//!  the product is neither the count alone nor the element alone.
//!  A file whose own type name no section holds passes the pre-scan and stops
//! the reader
//!  while it parses,
//! so the refusal names the module file rather than a type entry,
//! which is the one branch of this adapter no comparison reaches.
//!  Every other branch below is a lookup the reader answers for this file,
//! and each
//!  control runs on a machine with no kernel type information,
//! where only the
//!  guest arms would otherwise reach it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! checkPair('approved', emptyBase, moduleWith('none')); // a 4,096-byte array passes
//! ```

/// The approved declaration and the changes it can carry.
use super::approved_module_fixture::{Change, module_with};
/// The pair that types and names one member from a base file.
use super::base_file_fixture::module_typed_and_named_from_a_base_file;
/// The writer of the two files and the entry point's verdict.
use super::blob_fixture::{Blob, check_pair, check_raw};
/// The step one refusal names.
use super::refusal::Step;
/// The verdict and the refusal value.
use super::{Compatible, Refusal};

/// The 32-byte header every fixture file starts after.
const HEADER_BYTES: usize = 32;
/// One name offset past any string section this writer writes,
/// so the type entry that carries it names nothing.
const UNREADABLE_NAME: u32 = 0x7fff_fff0;

/// The gate's verdict for one change to the approved module,
/// over the empty base file every writer here starts from.
fn verdict(name: &str, change: Change) -> Result<Compatible, Refusal> {
    return check_pair(name, &Blob::new(), &module_with(change));
}

/// The refusal one change to the approved module must get.
fn refused(name: &str, change: Change) -> Refusal {
    let answer: Result<Compatible, Refusal> = verdict(name, change);
    let Err(refusal): Result<Compatible, Refusal> = answer else {
        panic!("the {name} module passed the gate");
    };
    return refusal;
}

/// One module file that must pass the gate,
/// naming what the fixture moved.
fn passed(name: &str, module: &Blob) -> Compatible {
    let answer: Result<Compatible, Refusal> = check_pair(name, &Blob::new(), module);
    let Ok(compatible): Result<Compatible, Refusal> = answer else {
        panic!("the {name} module refused: {}", answer.err().unwrap());
    };
    return compatible;
}

/// The approved module's first type entry renamed to an offset no section
/// holds,
/// so the reader cannot parse the file at all.
fn module_with_an_unreadable_type_name() -> Vec<u8> {
    let mut bytes: Vec<u8> = module_with(Change::None).bytes();
    let entry: usize = HEADER_BYTES;
    bytes[entry..entry + 4].copy_from_slice(&UNREADABLE_NAME.to_le_bytes());
    return bytes;
}

/// An array member reads its element size times its count:
/// `zc_name` is `unsigned char[4096]`,
/// so the approved module passes only if the adapter
///  multiplies the element's one byte by the array's 4,096 elements.
#[test]
fn an_array_member_reads_its_element_size_times_its_count() {
    let module: Blob = module_with(Change::None);
    let answer: Result<Compatible, Refusal> = verdict("adapter-array-approved", Change::None);
    let Ok(compatible): Result<Compatible, Refusal> = answer else {
        panic!("the approved array refused: {}", answer.err().unwrap());
    };
    assert_eq!(compatible.module_bytes, module.bytes().len());
}

/// One element short is a member refusal naming the size the product gave.
#[test]
fn an_array_one_element_short_refuses_at_its_member() {
    let refusal: Refusal = refused("adapter-array-one-short", Change::NameArrayOneShort);
    assert_eq!(refusal.step, Step::Member);
    assert!(refusal.detail.contains("zc_name"), "{}", refusal.detail);
    assert!(refusal.detail.contains("4095 bytes"), "{}", refusal.detail);
}

/// The same 4,096 bytes as four-byte elements is an element refusal,
/// so neither the count nor the element alone is the array's byte size.
#[test]
fn an_array_of_wider_elements_refuses_at_the_element_size() {
    let refusal: Refusal = refused("adapter-array-wider", Change::NameArrayWiderElements);
    assert_eq!(refusal.step, Step::BaseType);
    assert!(refusal.detail.contains("zc_name"), "{}", refusal.detail);
    assert!(
        refusal.detail.contains("elements of 4 bytes"),
        "{}",
        refusal.detail
    );
}

/// `zc_guid` under a chain of typedefs passes,
/// so every hop of the alias walk is the module's own entry.
///
/// A reader that answers nothing for a typedef refuses here at `zc_guid`'s
/// size,
/// naming the member whose alias chain it could not follow.
#[test]
fn a_member_under_a_typedef_chain_reads_its_integer() {
    let module: Blob = module_with(Change::TypedefChain);
    let compatible: Compatible = passed("adapter-typedef-chain", &module);
    assert_eq!(compatible.module_bytes, module.bytes().len());
}

/// `zc_guid` under a type tag,
/// a `const`,
/// a `restrict` and a `volatile` passes,
/// so every qualifier is followed rather than answered as another kind.
///
/// A reader that drops one qualifier refuses here at `zc_guid`'s size.
#[test]
fn a_member_under_a_qualifier_chain_reads_its_integer() {
    let module: Blob = module_with(Change::QualifierChain);
    let compatible: Compatible = passed("adapter-qualifier-chain", &module);
    assert_eq!(compatible.module_bytes, module.bytes().len());
}

/// A member of the module may be typed and named by the base file it extends:
/// `zc_simple` is the base file's own integer,
/// written under the base file's own name,
/// and the pair passes.
///
/// A reader that answers a handle from the module's own section alone refuses
/// here,
/// because the base file's id and name are not the module's.
#[test]
fn a_member_typed_and_named_from_the_base_file_reads_through_it() {
    let (base, module): (Blob, Blob) = module_typed_and_named_from_a_base_file();
    let answer: Result<Compatible, Refusal> = check_pair("adapter-base-member", &base, &module);
    let Ok(compatible): Result<Compatible, Refusal> = answer else {
        panic!("the module over a base file refused: {}", answer.err().unwrap());
    };
    assert_eq!(compatible.module_bytes, module.bytes().len());
    assert_eq!(compatible.base_bytes, base.bytes().len());
}

/// A base file the pre-scan passes and the reader cannot parse refuses at the
/// base file,
/// so a caller reads which of the two files stopped the gate.
#[test]
fn a_base_file_the_reader_cannot_parse_refuses_at_the_base_file() {
    let name: &str = "adapter-unreadable-base-name";
    let answer: Result<Compatible, Refusal> = check_raw(
        name,
        &module_with_an_unreadable_type_name(),
        &module_with(Change::None).bytes(),
    );
    let Err(refusal): Result<Compatible, Refusal> = answer else {
        panic!("a base file with no readable type name passed");
    };
    assert_eq!(refusal.step, Step::BaseBtf);
    assert!(refusal.detail.contains("btf-rs refused it"), "{}", refusal.detail);
    assert!(refusal.to_string().contains("vmlinux"), "{}", refusal.to_string());
}

/// A negative enumerator is compared as the signed number the format states:
/// `ZFS_IOC_OBJSET_STATS` holding `0xffff_fffe` refuses as `-2`,
/// not as the unsigned value its 32 bits also spell.
///
/// A reader that widens the value without the sign prints the unsigned number
/// here.
#[test]
fn a_negative_enumerator_is_compared_signed() {
    let refusal: Refusal = refused("adapter-negative-enumerator", Change::SignedEnumerator);
    assert_eq!(refusal.step, Step::Enum);
    assert!(refusal.detail.contains("ZFS_IOC_OBJSET_STATS"), "{}", refusal.detail);
    assert!(refusal.detail.contains("is -2 "), "{}", refusal.detail);
}

/// A file the pre-scan passes and the reader cannot parse refuses at the
/// module file,
/// naming the reader's own reason.
#[test]
fn a_file_the_reader_cannot_parse_refuses_at_the_module_file() {
    let name: &str = "adapter-unreadable-type-name";
    let answer: Result<Compatible, Refusal> = check_raw(
        name,
        &Blob::new().bytes(),
        &module_with_an_unreadable_type_name(),
    );
    let Err(refusal): Result<Compatible, Refusal> = answer else {
        panic!("a file with no readable type name passed");
    };
    assert_eq!(refusal.step, Step::ModuleBtf);
    assert!(refusal.detail.contains("btf-rs refused it"), "{}", refusal.detail);
}
