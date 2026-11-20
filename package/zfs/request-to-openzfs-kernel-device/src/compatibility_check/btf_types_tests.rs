//! What:
//!  The type-entry walk and the lookups:
//! a kind this reader cannot walk refuses,
//! a head shorter
//!  than the format's refuses,
//! a member count wider than sixteen bits is walked,
//! sizes
//!  resolve through arrays and typedefs,
//! and a module's names and ids fall through to its
//!  base.
//! Why:
//!  The walk is what keeps every later lookup inside the file,
//! and the widened
//!  member count is the one encoding a reader built on the older masks would
//!  misread.

use super::blob_fixture::Blob;
use super::btf_header::{Sections, read_sections};
use super::btf_entry::{Entry, KIND_STRUCT};
/// The indexed type entries.
use super::btf_types::TypeInformation;
use super::refusal::{Refusal, Step};
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

/// The refusal one file's type entries must get.
fn refused(bytes: &[u8]) -> Refusal {
    let sections: Sections<'_> =
        read_sections(bytes, Path::new(FIXTURE)).expect("the header is read");
    let answer: Result<TypeInformation<'_>, Refusal> =
        TypeInformation::alone(sections.types, sections.strings, Path::new(FIXTURE));
    let Err(refusal): Result<TypeInformation<'_>, Refusal> = answer else {
        panic!("the entries were walked");
    };
    return refusal;
}

#[test]
fn a_kind_this_reader_cannot_walk_refuses() {
    let mut blob: Blob = Blob::new();
    blob.int("unsigned char", 8);
    let mut bytes: Vec<u8> = blob.bytes();
    bytes[32 + 7] = 30;
    let refusal: Refusal = refused(&bytes);
    assert_eq!(refusal.step, Step::Header);
    assert!(refusal.detail.contains("kind 30"), "{}", refusal.detail);
}

#[test]
fn a_head_shorter_than_the_format_refuses() {
    let mut bytes: Vec<u8> = Vec::<u8>::new();
    bytes.extend_from_slice(&0xeb9f_u16.to_le_bytes());
    bytes.push(1);
    bytes.push(0);
    bytes.extend_from_slice(&32_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&5_u32.to_le_bytes());
    bytes.extend_from_slice(&5_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.extend_from_slice(&5_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&[0, 0, 0, 0, 0]);
    bytes.push(0);
    let refusal: Refusal = refused(&bytes);
    assert!(refusal.detail.contains("fewer than the"), "{}", refusal.detail);
}

#[test]
fn a_member_count_wider_than_sixteen_bits_is_walked() {
    let mut blob: Blob = Blob::new();
    let mut members: Vec<super::blob_fixture::WrittenMember> = Vec::new();
    for index in 0..70_000_u32 {
        members.push(super::blob_fixture::WrittenMember {
            name: "",
            type_id: 0,
            bit_offset: index,
        });
    }
    blob.structure("wide", 16, members.as_slice());
    let bytes: Vec<u8> = blob.bytes();
    let types: TypeInformation<'_> = information(&bytes);
    let found: Option<u32> = types.find("wide", &[KIND_STRUCT]);
    assert_eq!(found, Some(1));
}

#[test]
fn a_size_resolves_through_an_array_and_a_typedef() {
    let mut blob: Blob = Blob::new();
    let byte: u32 = blob.int("unsigned char", 8);
    let array: u32 = blob.array(byte, 256);
    let alias: u32 = blob.alias("char_array_t", array);
    let bytes: Vec<u8> = blob.bytes();
    let types: TypeInformation<'_> = information(&bytes);
    assert_eq!(types.byte_size(array), Some(256));
    assert_eq!(types.byte_size(alias), Some(256));
}

#[test]
fn a_module_reads_its_bases_type_by_id_and_its_name_by_offset() {
    let mut base_blob: Blob = Blob::new();
    let base_int: u32 = base_blob.int("unsigned long", 64);
    let mut module_blob: Blob = Blob::over(base_blob.string_bytes());
    let stats: u32 = module_blob.structure(
        "dmu_objset_stats",
        288,
        &[super::blob_fixture::WrittenMember {
            name: "dds_guid",
            type_id: base_int,
            bit_offset: 0,
        }],
    );
    let base_bytes: Vec<u8> = base_blob.bytes();
    let module_bytes: Vec<u8> = module_blob.bytes();
    let base_sections: Sections<'_> =
        read_sections(&base_bytes, Path::new(FIXTURE)).expect("the base header");
    let base: TypeInformation<'_> =
        TypeInformation::alone(base_sections.types, base_sections.strings, Path::new(FIXTURE))
            .expect("the base types");
    let module_sections: Sections<'_> =
        read_sections(&module_bytes, Path::new(FIXTURE)).expect("the module header");
    let module: TypeInformation<'_> = TypeInformation::over_base(
        module_sections.types,
        module_sections.strings,
        &base,
        Path::new(FIXTURE),
    )
    .expect("the module types");
    assert_eq!(module.byte_size(base_int), Some(8));
    assert_eq!(module.entry(base_int).map(|entry| { return entry.kind(); }), Some(1));
    let found: Option<u32> = module.find("dmu_objset_stats", &[KIND_STRUCT]);
    assert_eq!(found, Some(stats.saturating_add(1)));
    let names: Option<&str> = module.entry(found.unwrap()).and_then(|entry| {
        return entry.name();
    });
    assert_eq!(names, Some("dmu_objset_stats"));
    assert_eq!(base.find("dmu_objset_stats", &[KIND_STRUCT]), None);
}

#[test]
fn a_zero_id_names_no_type() {
    let bytes: Vec<u8> = Blob::new().bytes();
    let types: TypeInformation<'_> = information(&bytes);
    assert!(types.entry(0).is_none());
    assert_eq!(types.count(), 0);
}

#[test]
fn an_entry_of_no_known_kind_keeps_its_number() {
    let entry: Entry<'_> = Entry::Other { kind: 30 };
    assert_eq!(entry.kind(), 30);
    assert_eq!(entry.name(), None);
}
