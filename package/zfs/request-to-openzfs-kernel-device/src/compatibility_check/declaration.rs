//! What:
//!  The reader-neutral declaration of one module's type information:
//! a named container,
//!  its byte size and its members in declaration order,
//! a type handle's own size,
//!  whether it is a base integer or an array,
//! its array element,
//! and an enumeration's values.
//! Why:
//!  The project has not picked the profile reader yet,
//! so everything that
//!  compares or reports a layout depends on this declaration alone,
//! and one adapter
//!  file maps a reader onto it.
//!  The vocabulary is the one the finalists give:
//! `btf-rs` resolves a named type,
//!  a chained type and a member's name and size,
//! `pahole` prints each member's
//!  offset and size,
//! and `bpftool` dumps the same facts with the base file supplied,
//!  so a reader picked later fills one implementation rather than the comparison.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface Declaration { findContainer(name, kind): Handle | undefined; declared(handle): DeclaredType | undefined; }
//! ```

/// What:
///  Which of the two record kinds a container is.
/// Why:
///   The format gives each kind its own number,
/// and an anonymous container is matched
///  by kind and position,
/// so the kind is part of the declaration and not of the reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ContainerKind = 'struct' | 'union';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainerKind {
    /// A structure.
    Struct,
    /// A union,
    /// whose members share their first bit.
    Union,
}

/// The words one container kind is read as.
impl ContainerKind {
    /// The kind in the words a mismatch prints.
    pub fn words(self) -> &'static str {
        return match self {
            ContainerKind::Struct => "a structure",
            ContainerKind::Union => "a union",
        };
    }
}

/// What:
///  One member of a record,
/// as the module declares it.
/// Why:
///   A member is matched by name,
/// or by kind and position for an anonymous
///  container,
/// then compared by bit offset and type size;
///  a bitfield keeps its
///  width so the comparison can refuse it instead of comparing a type size that
///  is not the field's width.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeclaredMember = { name?: string; typeId: number; bitOffset: number; bitfieldBits?: number };
/// ```
pub struct DeclaredMember {
    /// The member's name,
    /// and nothing for an anonymous container.
    pub name: Option<String>,
    /// The member's type handle.
    pub type_id: u32,
    /// Bits from the container's first bit.
    pub bit_offset: u32,
    /// A bitfield's width in bits,
    /// when the member is one.
    pub bitfield_bits: Option<u8>,
}

/// What:
///  One type handle resolved through every alias over it.
/// Why:
///   The comparison asks a type for its size,
/// its kind and its members,
/// and
///  the module may write it under a name or a qualifier,
/// so the declaration answers
///  the resolved type rather than the alias.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DeclaredType = { kind: 'record'; byteSize: number; members: DeclaredMember[] } | { kind: 'integer'; byteSize: number } | ...;
/// ```
pub enum DeclaredType {
    /// A base integer type.
    Integer {
        /// Its width in bytes.
        byte_size: u64,
    },
    /// A pointer.
    Pointer {
        /// Its width in bytes.
        byte_size: u64,
    },
    /// A fixed array.
    Array {
        /// The element type.
        element: u32,
        /// Its whole width in bytes.
        byte_size: u64,
    },
    /// A structure or a union.
    Record {
        /// Which of the two it is.
        kind: ContainerKind,
        /// Its size in bytes.
        byte_size: u64,
        /// Its members,
        /// in declaration order.
        members: Vec<DeclaredMember>,
    },
    /// An enumeration of 32-bit or 64-bit values.
    Enumeration {
        /// Its size in bytes.
        byte_size: u64,
        /// Its values,
        /// in declaration order.
        values: Vec<(String, i64)>,
    },
    /// A floating-point type.
    Float {
        /// Its width in bytes.
        byte_size: u64,
    },
    /// A kind the comparison never reads.
    Other,
}

/// The words one declared type is read as.
impl DeclaredType {
    /// The type in the words a mismatch prints.
    pub fn words(&self) -> &'static str {
        return match self {
            DeclaredType::Integer { .. } => "an integer",
            DeclaredType::Pointer { .. } => "a pointer",
            DeclaredType::Array { .. } => "an array",
            DeclaredType::Record { kind, .. } => kind.words(),
            DeclaredType::Enumeration { .. } => "an enumeration",
            DeclaredType::Float { .. } => "a floating-point type",
            DeclaredType::Other => "another kind",
        };
    }
}

/// What:
///  What the comparison asks of one module's declared types.
/// Why:
///   The comparison,
/// the request walk,
/// the expectation table and the entry
///  point name only this trait,
/// so the reader that fills it is one file and the walk
///  stays the same whichever reader is picked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Declaration { findContainer(name, kind): number | undefined; findEnumeration(name): number | undefined; declared(id): DeclaredType | undefined; }
/// ```
pub trait Declaration {
    /// The handle of the named structure or union,
    /// or nothing when the module declares none.
    fn find_container(&self, name: &str, kind: ContainerKind) -> Option<u32>;

    /// The handle of the named enumeration,
    /// or nothing when the module declares none.
    fn find_enumeration(&self, name: &str) -> Option<u32>;

    /// One handle's type,
    /// through every alias over it.
    fn declared(&self, id: u32) -> Option<DeclaredType>;

    /// The size in bytes one handle describes.
    fn byte_size(&self, id: u32) -> Option<u64> {
        return match self.declared(id)? {
            DeclaredType::Integer { byte_size }
            | DeclaredType::Pointer { byte_size }
            | DeclaredType::Array { byte_size, .. }
            | DeclaredType::Record { byte_size, .. }
            | DeclaredType::Enumeration { byte_size, .. }
            | DeclaredType::Float { byte_size } => Some(byte_size),
            DeclaredType::Other => None,
        };
    }

    /// Whether one handle is a base integer or an array of them.
    fn is_base_integer(&self, id: u32) -> bool {
        return matches!(
            self.declared(id),
            Some(DeclaredType::Integer { .. } | DeclaredType::Array { .. })
        );
    }

    /// Whether one handle is the given container kind.
    fn is_container(&self, id: u32, kind: ContainerKind) -> bool {
        return matches!(
            self.declared(id),
            Some(DeclaredType::Record { kind: actual, .. }) if actual == kind
        );
    }
}
