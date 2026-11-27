//! What:
//!  The type entries a test asks a blob to hold:
//! an integer of a stated byte width,
//! a pointer,
//! a typedef and each qualifier over another type,
//! a fixed array,
//! a structure,
//! a union,
//! and an enumeration.
//! Why:
//!  A test that refuses a changed module has to write the module's declaration
//!  itself,
//! so each entry is stated in the words the format uses rather than as raw
//!  bytes,
//! and a reader can check one against the design's `pahole` output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const byte: number = blob.int('zc_simple', 1); const record: number = blob.structure('zfs_cmd', 13744, members);
//! ```

/// The blob the entries are written into,
/// and the member a structure or union is written from.
use super::blob_fixture::{
    Blob,
    WrittenMember,
};
/// The kind numbers the entries and their record are written with.
use super::blob_fixture::{
    KIND_CONST,
    KIND_RESTRICT,
    KIND_STRUCT,
    KIND_TYPE_TAG,
    KIND_TYPEDEF,
    KIND_UNION,
    KIND_VOLATILE,
};

/// The type entries one blob hands out.
impl Blob {
    /// One base integer type,
    /// whose size is stated in bytes.
    pub fn int(
        &mut self,
        name: &str,
        bytes: u32,
    ) -> u32 {
        let entry: Vec<u8> = self.entry(name, 1, 0, bytes, &[0, 0, 0, 0]);
        return self.push(entry);
    }

    /// One pointer,
    /// whose width the format states nowhere.
    pub fn pointer(
        &mut self,
        name: &str,
    ) -> u32 {
        let entry: Vec<u8> = self.entry(name, 2, 0, 0, &[]);
        return self.push(entry);
    }

    /// One typedef of the type the writer is about to hand it,
    /// so the chain over it never leaves its own entry.
    pub fn self_alias(
        &mut self,
        name: &str,
    ) -> u32 {
        let id: u32 = self.next_id();
        return self.typedef(name, id);
    }

    /// One typedef of another type,
    /// whose own id is the one this file hands it.
    pub fn typedef(
        &mut self,
        name: &str,
        of: u32,
    ) -> u32 {
        let entry: Vec<u8> = self.entry(name, KIND_TYPEDEF, 0, of, &[]);
        return self.push(entry);
    }

    /// One `volatile` over another type.
    pub fn volatile(
        &mut self,
        of: u32,
    ) -> u32 {
        return self.qualified(KIND_VOLATILE, of);
    }

    /// One `const` over another type.
    pub fn constant(
        &mut self,
        of: u32,
    ) -> u32 {
        return self.qualified(KIND_CONST, of);
    }

    /// One `restrict` over another type.
    pub fn restrict(
        &mut self,
        of: u32,
    ) -> u32 {
        return self.qualified(KIND_RESTRICT, of);
    }

    /// One type tag over another type.
    pub fn type_tag(
        &mut self,
        of: u32,
    ) -> u32 {
        return self.qualified(KIND_TYPE_TAG, of);
    }

    /// One qualifier of the kind number given,
    /// over another type.
    fn qualified(
        &mut self,
        kind: u32,
        of: u32,
    ) -> u32 {
        let entry: Vec<u8> = self.entry("", kind, 0, of, &[]);
        return self.push(entry);
    }

    /// One fixed array of `count` elements of `element`.
    pub fn array(
        &mut self,
        element: u32,
        count: u32,
    ) -> u32 {
        let mut extra: Vec<u8> = Vec::<u8>::new();
        extra.extend_from_slice(&element.to_le_bytes());
        extra.extend_from_slice(&0_u32.to_le_bytes());
        extra.extend_from_slice(&count.to_le_bytes());
        let entry: Vec<u8> = self.entry("", 3, 0, 0, &extra);
        return self.push(entry);
    }

    /// One fixed array whose element is its own entry,
    /// which no reader can give a size without asking itself again.
    pub fn array_naming_itself(
        &mut self,
        count: u32,
    ) -> u32 {
        let id: u32 = self.next_id();
        return self.array(id, count);
    }

    /// One structure.
    pub fn structure(
        &mut self,
        name: &str,
        bytes: u32,
        members: &[WrittenMember],
    ) -> u32 {
        return self.record(name, KIND_STRUCT, bytes, members);
    }

    /// One union.
    pub fn union(
        &mut self,
        name: &str,
        bytes: u32,
        members: &[WrittenMember],
    ) -> u32 {
        return self.record(name, KIND_UNION, bytes, members);
    }

    /// One enumeration of 32-bit values.
    pub fn enumeration(
        &mut self,
        name: &str,
        bytes: u32,
        values: &[(&str, i32)],
    ) -> u32 {
        let mut extra: Vec<u8> = Vec::<u8>::new();
        for (value_name, value) in values {
            let offset: u32 = self.name(value_name);
            extra.extend_from_slice(&offset.to_le_bytes());
            extra.extend_from_slice(&value.to_le_bytes());
        }
        let entry: Vec<u8> = self.entry(name, 6, values.len() as u32, bytes, &extra);
        return self.push(entry);
    }

    /// One structure or union with its members.
    fn record(
        &mut self,
        name: &str,
        kind: u32,
        bytes: u32,
        members: &[WrittenMember],
    ) -> u32 {
        let mut extra: Vec<u8> = Vec::<u8>::new();
        for member in members {
            let offset: u32 = match member.name_offset {
                Some(written) => written,
                None => self.name(member.name),
            };
            extra.extend_from_slice(&offset.to_le_bytes());
            extra.extend_from_slice(&member.type_id.to_le_bytes());
            extra.extend_from_slice(&member.bit_offset.to_le_bytes());
        }
        let entry: Vec<u8> = self.entry(name, kind, members.len() as u32, bytes, &extra);
        return self.push(entry);
    }
}
