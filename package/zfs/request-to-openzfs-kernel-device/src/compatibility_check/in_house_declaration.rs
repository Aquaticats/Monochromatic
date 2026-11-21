//! What:
//!  The one adapter that maps this crate's in-house decoder onto the
//! reader-neutral
//!  declaration,
//! and the call that compares one module's file through it.
//! Why:
//!  The project has ruled an in-house reader ineligible and has not picked the
//!  profile reader yet,
//! so this decoder stands behind the declaration until the pick
//!  lands and is the only file the pick replaces.
//!  The member list and the enumeration values live here too,
//! because they are
//!  the decoder's reading of the file and nothing outside this adapter needs
//!  them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const module = inHouseDeclaration(base, moduleFile); compare(module); compareRequests(module);
//! ```

/// The bytes one enumeration's value holds.
use super::btf_decode::enum_value_bytes;
/// The entry vocabulary,
/// the member bytes and a member's two fields.
use super::btf_entry::{
    ALIAS_HOPS, BITFIELD_SHIFT, BIT_OFFSET_MASK, ENUM_KINDS, Entry, KIND_STRUCT, KIND_UNION,
    MEMBER_BYTES, POINTER_BYTES,
};
/// The section cut of each file.
use super::btf_header::Sections;
/// The indexed type entries.
use super::btf_types::TypeInformation;
/// The walk that compares the declared layout.
use super::comparison::compare;
/// The walk that compares the request numbers.
use super::comparison_requests::compare_requests;
/// The reader-neutral vocabulary this adapter fills.
use super::declaration::{ContainerKind, DeclaredMember, DeclaredType, Declaration};
/// The one difference value either walk returns.
use super::mismatch::Mismatch;
/// The refusal value and the step one difference is reported as.
use super::refusal::{Refusal, refusal_at};
/// The borrowed path type a refusal names.
use std::path::Path;

/// Read one file pair through the in-house decoder and compare it.
///
/// # Errors
/// A refusal naming the step and the file of the read that failed,
/// or of the first difference.
pub fn compare_declared_layout(
    base: &Sections<'_>,
    module: &Sections<'_>,
    base_path: &Path,
    module_path: &Path,
) -> Result<(), Refusal> {
    let base_types: TypeInformation<'_> = TypeInformation::alone(base.types, base.strings, base_path)?;
    let module_types: TypeInformation<'_> =
        TypeInformation::over_base(module.types, module.strings, &base_types, module_path)?;
    let declaration: InHouseDeclaration<'_> = InHouseDeclaration { module: module_types };
    compare(&declaration).map_err(|mismatch: Mismatch| {
        return refusal_at(mismatch.step, module_path, mismatch.detail);
    })?;
    compare_requests(&declaration).map_err(|mismatch: Mismatch| {
        return refusal_at(mismatch.step, module_path, mismatch.detail);
    })?;
    return Ok(());
}

/// The in-house decoder seen as the reader-neutral declaration.
struct InHouseDeclaration<'a> {
    /// The indexed module entries,
    /// whose base answers the kernel's own ids and names.
    module: TypeInformation<'a>,
}

/// The in-house decoder read through the declaration.
impl Declaration for InHouseDeclaration<'_> {
    /// One named structure or union.
    fn find_container(&self, name: &str, kind: ContainerKind) -> Option<u32> {
        return self.module.find(name, &[kind_number(kind)]);
    }

    /// One named enumeration.
    fn find_enumeration(&self, name: &str) -> Option<u32> {
        return self.module.find(name, &ENUM_KINDS);
    }

    /// One entry,
    /// through every alias over it.
    fn declared(&self, id: u32) -> Option<DeclaredType> {
        let mut current: u32 = id;
        for _hop in 0..ALIAS_HOPS {
            let entry: Entry<'_> = self.module.entry(current)?;
            match entry {
                Entry::Alias { target, .. } => current = target,
                Entry::Integer { bytes, .. } => {
                    return Some(DeclaredType::Integer {
                        byte_size: u64::from(bytes),
                    });
                }
                Entry::Pointer { .. } => {
                    return Some(DeclaredType::Pointer {
                        byte_size: POINTER_BYTES,
                    });
                }
                Entry::Array { element, count, .. } => {
                    return Some(DeclaredType::Array {
                        element,
                        byte_size: self
                            .module
                            .byte_size(element)?
                            .saturating_mul(u64::from(count)),
                    });
                }
                Entry::Record {
                    kind,
                    bytes,
                    members,
                    bitfield,
                    ..
                } => {
                    return Some(DeclaredType::Record {
                        kind: container_kind(kind),
                        byte_size: u64::from(bytes),
                        members: members_of(&self.module, members, bitfield),
                    });
                }
                Entry::Enumeration { bytes, .. } => {
                    return Some(DeclaredType::Enumeration {
                        byte_size: u64::from(bytes),
                        values: self.module.enumeration_values(current)?,
                    });
                }
                Entry::Float { bytes, .. } => {
                    return Some(DeclaredType::Float {
                        byte_size: u64::from(bytes),
                    });
                }
                Entry::Other { .. } => return Some(DeclaredType::Other),
            }
        }
        return None;
    }
}

/// The kind number the in-house decoder gives one container kind.
fn kind_number(kind: ContainerKind) -> u8 {
    return match kind {
        ContainerKind::Struct => KIND_STRUCT,
        ContainerKind::Union => KIND_UNION,
    };
}

/// The container kind one decoder kind number is.
fn container_kind(kind: u8) -> ContainerKind {
    if kind == KIND_UNION {
        return ContainerKind::Union;
    }
    return ContainerKind::Struct;
}

/// One record's members,
/// read from the bytes the module wrote them as.
fn members_of(module: &TypeInformation<'_>, bytes: &[u8], bitfield: bool) -> Vec<DeclaredMember> {
    let mut members: Vec<DeclaredMember> = Vec::<DeclaredMember>::with_capacity(bytes.len() / MEMBER_BYTES);
    let list: Members<'_> = Members::new(bytes, bitfield);
    for index in 0..list.len() {
        let Some(member): Option<Member<'_>> = list.at(module, index) else {
            continue;
        };
        members.push(DeclaredMember {
            name: member.name.map(str::to_string),
            type_id: member.type_id,
            bit_offset: member.bit_offset,
            bitfield_bits: member.bitfield,
        });
    }
    return members;
}

/// The members of one record,
/// as the bytes the module wrote them.
pub(super) struct Members<'a> {
    /// The member bytes,
    /// each `MEMBER_BYTES` long.
    pub(super) bytes: &'a [u8],
    /// Whether the container's members carry bitfields.
    pub(super) bitfield: bool,
}

/// One record member as the module declares it.
pub(super) struct Member<'a> {
    /// The member's name,
    /// and nothing for an anonymous container.
    pub(super) name: Option<&'a str>,
    /// The member's type id.
    pub(super) type_id: u32,
    /// Bits from the container's first bit.
    pub(super) bit_offset: u32,
    /// A bitfield's width in bits,
    /// when the member is one.
    pub(super) bitfield: Option<u8>,
}

/// The members of one record.
impl<'a> Members<'a> {
    /// The member list one record's bytes hold.
    pub(super) fn new(bytes: &'a [u8], bitfield: bool) -> Self {
        return Members { bytes, bitfield };
    }

    /// How many members the record declares.
    pub(super) fn len(&self) -> usize {
        return self.bytes.len() / MEMBER_BYTES;
    }

    /// One member,
    /// with its name read through the file's strings.
    pub(super) fn at(&self, types: &TypeInformation<'a>, index: usize) -> Option<Member<'a>> {
        let start: usize = index.checked_mul(MEMBER_BYTES)?;
        let bytes: &'a [u8] = self.bytes.get(start..start.saturating_add(MEMBER_BYTES))?;
        let name: &'a str = types.name_at(word(bytes, 0))?;
        let raw_offset: u32 = word(bytes, 8);
        let (bit_offset, bitfield): (u32, Option<u8>) = if self.bitfield {
            let width: u8 = (raw_offset >> BITFIELD_SHIFT) as u8;
            (
                raw_offset & BIT_OFFSET_MASK,
                if width > 0 { Some(width) } else { None },
            )
        } else {
            (raw_offset, None)
        };
        return Some(Member {
            name: if name.is_empty() { None } else { Some(name) },
            type_id: word(bytes, 4),
            bit_offset,
            bitfield,
        });
    }
}

/// The values of one enumeration,
/// in the order the module declared them.
impl<'a> TypeInformation<'a> {
    /// One enumeration's values,
    /// or nothing when the type is no enumeration.
    pub(super) fn enumeration_values(&self, id: u32) -> Option<Vec<(String, i64)>> {
        let entry: Entry<'a> = self.entry(id)?;
        let Entry::Enumeration { kind, values, .. } = entry else {
            return None;
        };
        let width: usize = enum_value_bytes(kind);
        let mut decoded: Vec<(String, i64)> = Vec::<(String, i64)>::new();
        for value in values.chunks_exact(width) {
            let name: &'a str = self.name_at(word(value, 0))?;
            let number: i64 = enumeration_value(value, width);
            decoded.push((name.to_string(), number));
        }
        return Some(decoded);
    }
}

/// One enumeration's value,
/// whose 64-bit form is two 32-bit words.
fn enumeration_value(value: &[u8], width: usize) -> i64 {
    let low: u32 = word(value, 4);
    if width == 8 {
        return i64::from(low as i32);
    }
    let high: u32 = word(value, 8);
    return i64::from(low) | (i64::from(high) << 32);
}

/// One little-endian 32-bit word at `at`.
fn word(bytes: &[u8], at: usize) -> u32 {
    let mut field: [u8; 4] = [0; 4];
    let Some(source): Option<&[u8]> = bytes.get(at..at + 4) else {
        return 0;
    };
    field.copy_from_slice(source);
    return u32::from_le_bytes(field);
}
