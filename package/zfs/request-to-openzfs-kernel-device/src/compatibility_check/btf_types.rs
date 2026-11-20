//! What:
//!  The type entries of one BTF file:
//! the walk that indexes every entry by its type id,
//!  the split-file name and size lookups,
//! and the one place a decoded entry is
//!  asked for.
//! Why:
//!  A module's file is split:
//! its type ids start after the kernel's own,
//! and its
//!  name offsets start after the kernel's strings,
//! so both lookups must fall through to the
//!  base file and neither may read a module name as a base name.
//!  The walk runs before any parser sees a type,
//! with the widest encodings the format has
//!  (a 24-bit member count and a 7-bit kind),
//! so a file this reader cannot walk is a
//!  refusal rather than a misread,
//! and every later lookup is one bounds-checked slice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const base = TypeInformation.alone({ types, strings, path });
//! const module = TypeInformation.overBase({ types, strings, path }, base);
//! module.find('zfs_cmd', [KIND_STRUCT]);
//! ```

/// The entry lengths and the decode of one entry.
use super::btf_decode::{decode, entry_extra_bytes};
/// The three fields of one entry's `info` word,
/// the bytes its head holds,
/// and the most aliases a size resolves through.
use super::btf_entry::{ALIAS_HOPS, ENTRY_HEAD_BYTES, KIND_MASK, KIND_SHIFT, POINTER_BYTES, VLEN_MASK};
/// The step vocabulary and the refusal builders.
use super::refusal::{Refusal, Step, refusal_at};
/// One decoded entry,
/// which every other file of this folder reads through here.
pub use super::btf_entry::Entry;
/// The borrowed path type the refusal names.
use std::path::Path;

/// What:
///  One file's type entries,
/// its strings and the base file its ids and names follow.
/// Why:
///   Every lookup the comparison makes is answered here:
/// a type by id,
/// a type by
///  name and a byte size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TypeInformation<'a> { entry(id): Entry<'a> | undefined; find(name, kinds): number | undefined; }
/// ```
pub struct TypeInformation<'a> {
    /// The type section's bytes.
    types: &'a [u8],
    /// This file's own strings,
    /// which a module's offsets index past the base's.
    strings: &'a [u8],
    /// The base file this one extends,
    /// or nothing for the base itself.
    base: Option<&'a TypeInformation<'a>>,
    /// Where each local type entry starts.
    offsets: Vec<u32>,
    /// The first type id this file owns.
    start_id: u32,
}

/// The walk that indexes one file's type entries.
impl<'a> TypeInformation<'a> {
    /// Index a file that extends no other.
    ///
    /// # Errors
    /// A refusal at the entry's step naming a type entry this reader cannot walk.
    pub fn alone(types: &'a [u8], strings: &'a [u8], path: &Path) -> Result<Self, Refusal> {
        return Self::build(types, strings, None, 1, path);
    }

    /// Index a file whose ids and names follow `base`.
    ///
    /// # Errors
    /// A refusal at the entry's step naming a type entry this reader cannot walk.
    pub fn over_base(
        types: &'a [u8],
        strings: &'a [u8],
        base: &'a TypeInformation<'a>,
        path: &Path,
    ) -> Result<Self, Refusal> {
        let start_id: u32 = base.count().saturating_add(1);
        return Self::build(types, strings, Some(base), start_id, path);
    }

    /// Walk every entry,
    /// refusing one whose length this reader cannot compute.
    ///
    /// # Errors
    /// A refusal at the entry's step naming the byte and the kind.
    fn build(
        types: &'a [u8],
        strings: &'a [u8],
        base: Option<&'a TypeInformation<'a>>,
        start_id: u32,
        path: &Path,
    ) -> Result<Self, Refusal> {
        let step: Step = Step::Header;
        let mut offsets: Vec<u32> = Vec::<u32>::new();
        let mut at: usize = 0;
        while at < types.len() {
            let head: &[u8] = &types[at..];
            if head.len() < ENTRY_HEAD_BYTES {
                return Err(refusal_at(
                    step,
                    path,
                    format!(
                        "a type entry at byte {at} holds {} bytes, fewer than the {ENTRY_HEAD_BYTES}-byte head",
                        head.len()
                    ),
                ));
            }
            let info: u32 = u32::from_le_bytes([head[4], head[5], head[6], head[7]]);
            let kind: u8 = ((info >> KIND_SHIFT) & KIND_MASK) as u8;
            let vlen: u32 = info & VLEN_MASK;
            let extra: usize = entry_extra_bytes(kind, vlen).ok_or_else(|| {
                return refusal_at(
                    step,
                    path,
                    format!("a type entry at byte {at} is of kind {kind}, which this reader cannot walk"),
                );
            })?;
            let length: usize = ENTRY_HEAD_BYTES.saturating_add(extra);
            if length > head.len() {
                return Err(refusal_at(
                    step,
                    path,
                    format!(
                        "a type entry at byte {at} needs {length} bytes and {} remain",
                        head.len()
                    ),
                ));
            }
            offsets.push(at as u32);
            at = at.saturating_add(length);
        }
        return Ok(TypeInformation {
            types,
            strings,
            base,
            offsets,
            start_id,
        });
    }

    /// How many type ids this file and its base answer.
    pub fn count(&self) -> u32 {
        return self.start_id.saturating_add(self.offsets.len() as u32).saturating_sub(1);
    }

    /// The type id `index` entries into this file.
    fn local_id(&self, index: usize) -> u32 {
        return self.start_id.saturating_add(index as u32);
    }

    /// The entry one type id names.
    pub fn entry(&self, id: u32) -> Option<Entry<'a>> {
        if id == 0 {
            return None;
        }
        if let Some(base) = self.base
            && id < self.start_id
        {
            return base.entry(id);
        }
        let index: usize = (id - self.start_id) as usize;
        let offset: usize = *self.offsets.get(index)? as usize;
        return Some(decode(&self.types[offset..], |at: u32| {
            return self.name_at(at);
        }));
    }

    /// Every name this file itself carries,
    /// for the two named types the comparison looks up.
    pub fn find(&self, name: &str, kinds: &[u8]) -> Option<u32> {
        for index in 0..self.offsets.len() {
            let id: u32 = self.local_id(index);
            let entry: Entry<'a> = self.entry(id)?;
            if kinds.contains(&entry.kind()) && entry.name() == Some(name) {
                return Some(id);
            }
        }
        return None;
    }

    /// One name at one string offset,
    /// which every reader in this folder asks for.
    pub(super) fn name_at(&self, offset: u32) -> Option<&'a str> {
        return self.string(offset);
    }

    /// One name at one string offset,
    /// which a module's file reads past the base's strings.
    fn string(&self, offset: u32) -> Option<&'a str> {
        if let Some(base) = self.base {
            if (offset as usize) < base.strings.len() {
                return base.within(offset as usize);
            }
            return self.within(offset as usize - base.strings.len());
        }
        return self.within(offset as usize);
    }

    /// One name inside this file's own strings.
    fn within(&self, index: usize) -> Option<&'a str> {
        let rest: &[u8] = self.strings.get(index..)?;
        let end: usize = rest.iter().position(|byte| {
            return *byte == 0;
        })?;
        return std::str::from_utf8(&rest[..end]).ok();
    }

    /// The size in bytes one type id describes,
    /// through every alias over it.
    pub fn byte_size(&self, id: u32) -> Option<u64> {
        let mut current: u32 = id;
        for _hop in 0..ALIAS_HOPS {
            match self.entry(current)? {
                Entry::Integer { bits, .. } => return Some(u64::from(bits) / 8),
                Entry::Pointer { .. } => return Some(POINTER_BYTES),
                Entry::Array { element, count, .. } => {
                    return Some(self.byte_size(element)?.saturating_mul(u64::from(count)));
                }
                Entry::Record { bytes, .. }
                | Entry::Enumeration { bytes, .. }
                | Entry::Float { bytes, .. } => return Some(u64::from(bytes)),
                Entry::Alias { target, .. } => current = target,
                Entry::Other { .. } => return None,
            }
        }
        return None;
    }

}
