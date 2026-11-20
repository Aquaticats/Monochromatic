//! What:
//!  The format's kind numbers,
//! the bytes one entry's kind holds,
//! and one decoded entry.
//! Why:
//!  Three readers ask what a kind is and what it carries:
//! the walk that indexes every
//!  entry,
//! the decode that reads one and the comparison that matches a member,
//! so the
//!  numbers and the value they decode to live before all of them and use none.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const KIND_STRUCT = 4;
//! type Entry = { kind: number; name?: string; ... };
//! ```

/// `BTF_KIND_INT`:
/// one base integer type.
pub const KIND_INT: u8 = 1;
/// `BTF_KIND_PTR`:
/// one pointer.
pub const KIND_PTR: u8 = 2;
/// `BTF_KIND_ARRAY`:
/// one fixed array.
pub const KIND_ARRAY: u8 = 3;
/// `BTF_KIND_STRUCT`:
/// one structure.
pub const KIND_STRUCT: u8 = 4;
/// `BTF_KIND_UNION`:
/// one union.
pub const KIND_UNION: u8 = 5;
/// `BTF_KIND_ENUM`:
/// one enumeration of 32-bit values.
pub const KIND_ENUM: u8 = 6;
/// `BTF_KIND_FWD`:
/// a forward declaration of a structure or a union.
pub const KIND_FWD: u8 = 7;
/// `BTF_KIND_TYPEDEF`:
/// another name for one type.
pub const KIND_TYPEDEF: u8 = 8;
/// `BTF_KIND_VOLATILE`:
/// a qualifier over one type.
pub const KIND_VOLATILE: u8 = 9;
/// `BTF_KIND_CONST`:
/// a qualifier over one type.
pub const KIND_CONST: u8 = 10;
/// `BTF_KIND_RESTRICT`:
/// a qualifier over one type.
pub const KIND_RESTRICT: u8 = 11;
/// `BTF_KIND_FUNC`:
/// one function's own entry.
pub const KIND_FUNC: u8 = 12;
/// `BTF_KIND_FUNC_PROTO`:
/// one function's prototype.
pub const KIND_FUNC_PROTO: u8 = 13;
/// `BTF_KIND_VAR`:
/// one global variable.
pub const KIND_VAR: u8 = 14;
/// `BTF_KIND_DATASEC`:
/// one section of global variables.
pub const KIND_DATASEC: u8 = 15;
/// `BTF_KIND_FLOAT`:
/// one floating-point type.
pub const KIND_FLOAT: u8 = 16;
/// `BTF_KIND_DECL_TAG`:
/// one declaration's tag.
pub const KIND_DECL_TAG: u8 = 17;
/// `BTF_KIND_TYPE_TAG`:
/// a tag over one type.
pub const KIND_TYPE_TAG: u8 = 18;
/// `BTF_KIND_ENUM64`:
/// one enumeration of 64-bit values.
pub const KIND_ENUM64: u8 = 19;
/// The kinds one enumeration is.
pub const ENUM_KINDS: [u8; 2] = [KIND_ENUM, KIND_ENUM64];
/// The most aliases one name or size resolves through.
pub const ALIAS_HOPS: usize = 32;
/// The bytes one type entry holds before its kind's own bytes.
pub const ENTRY_HEAD_BYTES: usize = 12;
/// The bits of the `info` word that hold the member count.
pub const VLEN_MASK: u32 = 0x00ff_ffff;
/// The shift of the kind in the `info` word.
pub const KIND_SHIFT: u32 = 24;
/// The bits of the `info` word that hold the kind.
pub const KIND_MASK: u32 = 0x7f;
/// The bit of the `info` word that says a record's members carry bitfields.
pub const KIND_FLAG: u32 = 1 << 31;
/// The bits of a record member's offset that hold a bitfield's width.
pub const BITFIELD_SHIFT: u32 = 24;
/// The bits of a record member's offset that hold the member's bit offset.
pub const BIT_OFFSET_MASK: u32 = 0x00ff_ffff;
/// The bytes one record member's entry holds.
pub const MEMBER_BYTES: usize = 12;
/// The bytes one array entry holds after its head.
pub const ARRAY_BYTES: usize = 12;
/// The bytes one integer entry holds after its head.
pub const INT_BYTES: usize = 4;
/// The bytes one variable's entry holds after its head.
pub const VAR_BYTES: usize = 4;
/// The bytes one declaration tag's entry holds after its head.
pub const DECL_TAG_BYTES: usize = 4;
/// The width of the pointer type,
/// which this format states for no pointer.
pub const POINTER_BYTES: u64 = 8;

/// What:
///  One decoded type entry of the kinds the comparison asks about.
/// Why:
///   The comparison reads names,
/// sizes and members,
/// and everything else is skipped by the walk
///  that indexed the entry,
/// so a kind outside this list keeps only its kind number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { kind: number; name?: string; ... };
/// ```
pub enum Entry<'a> {
    /// A base integer type,
    /// whose size this format states in bits.
    Integer {
        /// The type's name.
        name: Option<&'a str>,
        /// Its width in bits.
        bits: u32,
    },
    /// A pointer,
    /// whose width is the target's and whose target adds no size.
    Pointer {
        /// The type's name.
        name: Option<&'a str>,
    },
    /// A fixed array.
    Array {
        /// The type's name.
        name: Option<&'a str>,
        /// The element type.
        element: u32,
        /// How many elements it holds.
        count: u32,
    },
    /// A structure or a union.
    Record {
        /// The type's name,
        /// empty for an anonymous container.
        name: Option<&'a str>,
        /// Which of the two it is.
        kind: u8,
        /// Its size in bytes.
        bytes: u32,
        /// Its members,
        /// each `MEMBER_BYTES` long.
        members: &'a [u8],
        /// Whether its members carry bitfields.
        bitfield: bool,
    },
    /// An enumeration of 32-bit or 64-bit values.
    Enumeration {
        /// The type's name.
        name: Option<&'a str>,
        /// Its size in bytes.
        bytes: u32,
        /// Which width its values have.
        kind: u8,
        /// Its values.
        values: &'a [u8],
    },
    /// A floating-point type.
    Float {
        /// The type's name.
        name: Option<&'a str>,
        /// Its size in bytes.
        bytes: u32,
    },
    /// Another name or qualifier for one type.
    Alias {
        /// The type's name.
        name: Option<&'a str>,
        /// Which alias it is.
        kind: u8,
        /// The type it names.
        target: u32,
    },
    /// A kind the comparison never reads.
    Other {
        /// The kind number.
        kind: u8,
    },
}

/// The kind number of one decoded entry.
impl<'a> Entry<'a> {
    /// The kind this entry is,
    /// as the format numbers it.
    pub fn kind(&self) -> u8 {
        return match self {
            Entry::Integer { .. } => KIND_INT,
            Entry::Pointer { .. } => KIND_PTR,
            Entry::Array { .. } => KIND_ARRAY,
            Entry::Record { kind, .. } => *kind,
            Entry::Enumeration { kind, .. } => *kind,
            Entry::Float { .. } => KIND_FLOAT,
            Entry::Alias { kind, .. } => *kind,
            Entry::Other { kind } => *kind,
        };
    }

    /// The entry's name,
    /// or nothing for an anonymous one.
    pub fn name(&self) -> Option<&'a str> {
        return match self {
            Entry::Integer { name, .. }
            | Entry::Pointer { name, .. }
            | Entry::Array { name, .. }
            | Entry::Record { name, .. }
            | Entry::Enumeration { name, .. }
            | Entry::Float { name, .. }
            | Entry::Alias { name, .. } => *name,
            Entry::Other { .. } => None,
        };
    }
}
