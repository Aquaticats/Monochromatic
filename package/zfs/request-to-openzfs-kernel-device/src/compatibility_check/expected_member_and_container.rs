//! What:
//!  One member or container the module must declare,
//! the three ways a table states one,
//! and the byte width of the `zfs_cmd` fields the tables measure against.
//! Why:
//!  Every table beside this file is written through these three builders,
//! so a member's bit offset and byte size come from `offset_of!` and `size_of!`
//!  rather than a number copied out of a header,
//! and the two record types carry the names the comparison reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ExpectedMember = { name?: string; container?: ContainerKind; bitOffset: number; byteSize: number; elementBytes?: number; interior?: ExpectedContainer };
//! type ExpectedContainer = { name: string; kind: ContainerKind; byteSize: number; members: ExpectedMember[] };
//! ```

/// The container kind the declaration names.
use super::declaration::ContainerKind;
/// The lengths the struct's fixed-size name fields are declared with.
use crate::layout::{
    MAX_VALUE_LEN,
    MAXNAMELEN,
    MAXPATHLEN,
};
/// The operator the byte widths are measured with.
use std::mem::size_of;

/// The bytes the struct's first field holds,
/// kept as one number so the table below reads as one list.
pub(super) const NAME_BYTES: usize = size_of::<[u8; MAXPATHLEN]>();
/// The bytes the struct's secondary name holds.
pub(super) const VALUE_BYTES: usize = size_of::<[u8; MAX_VALUE_LEN]>();
/// The bytes the struct's short string holds.
pub(super) const STRING_BYTES: usize = size_of::<[u8; MAXNAMELEN]>();

/// What:
///  One member the module must declare,
/// with where it sits and the type size it holds.
/// Why:
///   A named member is matched by name;
/// an anonymous container is matched by its kind
///  and bit offset,
/// because the format writes it no name.
///  The byte size is the member type's own size,
/// and `element_bytes` is the size of
///  an array member's element,
/// so a module that keeps a total size while changing the
///  element cannot pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExpectedMember = { name?: string; container?: ContainerKind; bitOffset: number; byteSize: number; elementBytes?: number; interior?: ExpectedContainer };
/// ```
pub struct ExpectedMember {
    /// The member's name,
    /// and nothing for an anonymous container.
    pub name: Option<&'static str>,
    /// The container kind,
    /// for an anonymous container only.
    pub container: Option<ContainerKind>,
    /// Bits from the container's first bit.
    pub bit_offset: u32,
    /// The member type's size in bytes.
    pub byte_size: u32,
    /// An array member's element size in bytes.
    pub element_bytes: Option<u32>,
    /// The members of a container compared member by member.
    pub interior: Option<&'static ExpectedContainer>,
}

/// What:
///  One container the module must declare,
/// and the members it must hold.
/// Why:
///   The container's own size is compared before any member,
/// because a container that
///  grew is the one fact a moved member alone would not show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExpectedContainer = { name: string; kind: ContainerKind; byteSize: number; members: ExpectedMember[] };
/// ```
pub struct ExpectedContainer {
    /// The name the module declares it with.
    pub name: &'static str,
    /// Which of the two record kinds it is.
    pub kind: ContainerKind,
    /// Its size in bytes.
    pub byte_size: u32,
    /// Its members.
    pub members: &'static [ExpectedMember],
}

/// One whole member at the offset `offset_of!` gives its field.
pub(super) const fn member(
    name: &'static str,
    bit_offset: usize,
    byte_size: usize,
) -> ExpectedMember {
    return ExpectedMember {
        name: Some(name),
        container: None,
        bit_offset: bit_offset as u32,
        byte_size: byte_size as u32,
        element_bytes: None,
        interior: None,
    };
}

/// One array member,
/// with the element size its total hides.
pub(super) const fn array_member(
    name: &'static str,
    bit_offset: usize,
    byte_size: usize,
    element_bytes: usize,
) -> ExpectedMember {
    return ExpectedMember {
        name: Some(name),
        container: None,
        bit_offset: bit_offset as u32,
        byte_size: byte_size as u32,
        element_bytes: Some(element_bytes as u32),
        interior: None,
    };
}

/// One named member whose record is compared member by member.
pub(super) const fn descended(
    name: &'static str,
    bit_offset: usize,
    byte_size: usize,
    interior: &'static ExpectedContainer,
) -> ExpectedMember {
    return ExpectedMember {
        name: Some(name),
        container: None,
        bit_offset: bit_offset as u32,
        byte_size: byte_size as u32,
        element_bytes: None,
        interior: Some(interior),
    };
}
