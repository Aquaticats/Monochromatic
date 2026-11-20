//! What:
//!  One structure's or union's member list and one enumeration's values,
//! as the module
//!  declares them.
//! Why:
//!  The comparison reaches the anonymous union and the anonymous structure
//! inside
//!  it by kind and position,
//! because the format gives them no name,
//! so the member
//!  list must keep the bit offsets and the empty names the module wrote rather
//!  than the names this crate's Rust type gave its own fields.
//!  A member that carries a bitfield is kept with its width,
//! so the comparison can
//!  refuse it instead of comparing a type size that is not the field's width.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const members = module.members(zfsCmdId);
//! for (const member of members) if (member.name === 'zc_zoneid') check(member.bitOffset);
//! ```

/// The entry vocabulary the member list reads.
/// The bytes one enumeration's value holds.
use super::btf_decode::enum_value_bytes;
/// The entry vocabulary,
/// the member bytes and a member's two fields.
use super::btf_entry::{BITFIELD_SHIFT, BIT_OFFSET_MASK, Entry, MEMBER_BYTES};
/// The indexed type entries.
use super::btf_types::TypeInformation;

/// What:
///  One record member as the module declares it.
/// Why:
///   A member is matched by name,
/// or by kind and position for an anonymous
///  container,
/// then compared by bit offset and type size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Member<'a> = { name?: &'a str; typeId: number; bitOffset: number; bitfield?: number };
/// ```
pub struct Member<'a> {
    /// The member's name,
    /// and nothing for an anonymous container.
    pub name: Option<&'a str>,
    /// The member's type id.
    pub type_id: u32,
    /// Bits from the container's first bit.
    pub bit_offset: u32,
    /// A bitfield's width in bits,
    /// when the container carries bitfields.
    pub bitfield: Option<u8>,
}

/// What:
///  One record's members,
/// as the bytes the module wrote them.
/// Why:
///   The bytes are borrowed from the file the read already holds,
/// so reading a
///  member's name,
/// type and offset is a decode on demand rather than a copy of
///  every member of every type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Members<'a> { length: number; at(index, types): Member<'a> }
/// ```
pub struct Members<'a> {
    /// The member bytes,
    /// each `MEMBER_BYTES` long.
    bytes: &'a [u8],
    /// Whether the container's members carry bitfields.
    bitfield: bool,
}

/// The member list of one record.
impl<'a> TypeInformation<'a> {
    /// The values of one enumeration,
    /// in the order the module declared them.
    pub fn enumeration_values(&self, id: u32) -> Option<Vec<(String, i64)>> {
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

/// The members of one record.
impl<'a> Members<'a> {
    /// The member list one record's bytes hold.
    pub fn new(bytes: &'a [u8], bitfield: bool) -> Self {
        return Members { bytes, bitfield };
    }

    /// How many members the record declares.
    pub fn len(&self) -> usize {
        return self.bytes.len() / MEMBER_BYTES;
    }

    /// One member,
    /// with its name read through the file's strings.
    pub fn at(&self, types: &TypeInformation<'a>, index: usize) -> Option<Member<'a>> {
        let start: usize = index.checked_mul(MEMBER_BYTES)?;
        let bytes: &'a [u8] = self.bytes.get(start..start.saturating_add(MEMBER_BYTES))?;
        let name: &'a str = types.name_at(word(bytes, 0))?;
        let raw_offset: u32 = word(bytes, 8);
        let (bit_offset, bitfield): (u32, Option<u8>) = if self.bitfield {
            (
                raw_offset & BIT_OFFSET_MASK,
                Some((raw_offset >> BITFIELD_SHIFT) as u8),
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
