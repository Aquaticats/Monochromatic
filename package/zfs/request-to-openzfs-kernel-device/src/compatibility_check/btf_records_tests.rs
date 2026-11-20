//! What:
//!  The member list and the enumeration values:
//! names,
//! bit offsets,
//! empty names for
//!  anonymous containers and signed values decode as the module wrote them.
//! Why:
//!  The comparison matches an anonymous container by kind and position,
//! which it can
//!  only do when the empty name is read as no name and the offset is the container's,
//!  and a negative enumerator would be read as its unsigned form by a decoder
//! that  forgot the sign.

use super::blob_fixture::{Blob, WrittenMember};
use super::btf_header::{Sections, read_sections};
use super::btf_records::{Member, Members};
use super::btf_entry::Entry;
/// The indexed type entries.
use super::btf_types::TypeInformation;
use std::path::Path;

/// The file one arm reads,
/// whose path only names it in the refusal.
const FIXTURE: &str = "fixture";

/// The sections of one file,
/// cut and walked.
fn information<'a>(bytes: &'a [u8]) -> TypeInformation<'a> {
    let sections: Sections<'a> =
        read_sections(bytes, Path::new(FIXTURE)).expect("the header is read");
    return TypeInformation::alone(sections.types, sections.strings, Path::new(FIXTURE))
        .expect("the types are walked");
}

/// The member list of the only structure in one file.
fn members<'a>(types: &TypeInformation<'a>, id: u32) -> Members<'a> {
    let entry: Entry<'a> = types.entry(id).expect("the structure is declared");
    let Entry::Record { members, bitfield, .. } = entry else {
        panic!("the entry is no structure");
    };
    return Members::new(members, bitfield);
}

#[test]
fn a_structure_reads_each_member_name_and_offset() {
    let mut blob: Blob = Blob::new();
    let word: u32 = blob.int("unsigned int", 32);
    let structure: u32 = blob.structure(
        "record",
        16,
        &[
            WrittenMember {
                name: "first",
                type_id: word,
                bit_offset: 0,
            },
            WrittenMember {
                name: "",
                type_id: word,
                bit_offset: 32,
            },
            WrittenMember {
                name: "third",
                type_id: word,
                bit_offset: 64,
            },
        ],
    );
    let bytes: Vec<u8> = blob.bytes();
    let types: TypeInformation<'_> = information(&bytes);
    let list: Members<'_> = members(&types, structure);
    assert_eq!(list.len(), 3);
    assert_eq!(list.at(&types, 0).map(|member: Member<'_>| { return member.name; }), Some(Some("first")));
    assert_eq!(list.at(&types, 1).map(|member: Member<'_>| { return member.name; }), Some(None));
    let third: Member<'_> = list.at(&types, 2).expect("the third member");
    assert_eq!(third.bit_offset, 64);
    assert_eq!(third.bitfield, None);
    assert!(list.at(&types, 3).is_none());
}

#[test]
fn an_enumeration_reads_each_name_and_signed_value() {
    let mut blob: Blob = Blob::new();
    let enumeration: u32 = blob.enumeration(
        "signed",
        4,
        &[("one", 1), ("none", 0), ("minus one", -1)],
    );
    let bytes: Vec<u8> = blob.bytes();
    let types: TypeInformation<'_> = information(&bytes);
    let values: Vec<(String, i64)> = types
        .enumeration_values(enumeration)
        .expect("the values are read");
    assert_eq!(
        values,
        vec![
            ("one".to_string(), 1),
            ("none".to_string(), 0),
            ("minus one".to_string(), -1),
        ]
    );
}

#[test]
fn a_structure_is_no_enumeration_and_an_enumeration_is_no_member_list() {
    let mut blob: Blob = Blob::new();
    let word: u32 = blob.int("unsigned int", 32);
    let structure: u32 = blob.structure(
        "record",
        4,
        &[WrittenMember {
            name: "only",
            type_id: word,
            bit_offset: 0,
        }],
    );
    let bytes: Vec<u8> = blob.bytes();
    let types: TypeInformation<'_> = information(&bytes);
    assert!(types.enumeration_values(structure).is_none());
    assert!(types.entry(structure).is_some());
}
