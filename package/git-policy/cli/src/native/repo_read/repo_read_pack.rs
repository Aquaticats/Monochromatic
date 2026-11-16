//! What:
//!  A reader of one Git pack:
//!  the version 1 and version 2 index lookup by object name,
//!  and the entry read with `OFS_DELTA` and `REF_DELTA` resolution.
//! Why:
//!  Most objects of a real repository are packed.
//!  The reader opens the two files,
//! seeks for the few bytes a lookup needs,
//!  and follows a delta chain down to its base through a stack of its own,
//!  so a chain is read as deep as the pack holds it,
//!  loops and lengths the pack cannot hold end the walk.
//!  Anything it does not fully understand (an index of another version,
//! an inconsistent size,
//!  a chain that comes back to an entry it already looked at,
//! a bad delta)
//!  answers `None`,
//!  which sends the caller to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pack = openPack(idxPath, hashLength); const object = pack.read(pack.find(name), resolveByName);
//! ```

#![cfg(unix)]

use crate::repo_read_object::{ObjectKind, OBJECT_SIZE_LIMIT, RawObject, inflate_exact};
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::Path;

/// The most decoded bytes one pack keeps for later reads:
/// a listing of a large tree reads thousands of trees that share
///  delta bases,
/// and a stop at a fixed size keeps a long listing from growing without bound.
const CACHE_BYTE_LIMIT: usize = 32 * 1024 * 1024;

/// The bytes of a version 2 index's header and its table of counts:
/// magic,
/// version,
/// 256 counts.
const FANOUT_END: u64 = 8 + 256 * 4;

/// The bytes of a version 1 index's table of counts,
/// which has no header in front of it.
const FANOUT_ONLY: u64 = 256 * 4;

/// What:
///  Which of the two index layouts a pack's index holds.
/// Why:
///   Both are stores Git itself reads,
/// and they differ in where a name and its entry's offset sit,
/// so the lookup asks the layout where to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexLayout = { version: 1 | 2; recordsAt: number; recordLength: number; nameOffset: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexLayout {
    /// Every entry is its own offset word and its name,
    /// sorted by name,
    /// with no table of offsets and no large offsets.
    Version1,
    /// A table of names with a checksum each,
    /// a table of offsets,
    /// and a table of large offsets for an offset the one word cannot hold.
    Version2,
}

/// The version 2 index's magic,
/// which a version 1 index does not have.
const VERSION_TWO_MAGIC: &[u8] = b"\xfftOc\0\0\0\x02";

impl IndexLayout {
    /// Where the first entry starts.
    fn records_at(self) -> u64 {
        return match self {
            IndexLayout::Version1 => FANOUT_ONLY,
            IndexLayout::Version2 => FANOUT_END,
        };
    }

    /// How many bytes one entry takes.
    fn record_length(self, hash_length: usize) -> u64 {
        return match self {
            IndexLayout::Version1 => 4 + hash_length as u64,
            IndexLayout::Version2 => hash_length as u64,
        };
    }

    /// Where the object's name sits inside its entry.
    fn name_offset(self) -> u64 {
        return match self {
            IndexLayout::Version1 => 4,
            IndexLayout::Version2 => 0,
        };
    }

    /// Where the offset word of the `position`th name sits,
    /// given how many names the index holds.
    fn offset_at(self, position: u64, count: u64, hash_length: usize) -> u64 {
        return match self {
            IndexLayout::Version1 => FANOUT_ONLY + position * self.record_length(hash_length),
            IndexLayout::Version2 => FANOUT_END + count * (hash_length as u64 + 4) + position * 4,
        };
    }

    /// The smallest an index of this layout can be,
    /// given how many names it holds:
    /// a version 1 entry is its offset word and its name,
    /// a version 2 entry is its name,
    ///  its checksum and its offset,
    /// then the pack's own checksum and the index's checksum.
    fn smallest(self, count: u64, hash_length: usize) -> u64 {
        let entries: u64 = match self {
            IndexLayout::Version1 => count * (4 + hash_length as u64),
            IndexLayout::Version2 => count * (hash_length as u64 + 8),
        };
        return self.records_at() + entries + 2 * hash_length as u64;
    }
}

/// How many pack entries every pack of this process decoded,
/// for a control that compares the work of two ways to answer.
#[cfg(test)]
pub(crate) static DECODED_TOTAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// One open pack and its version 2 index.
#[derive(Debug)]
pub struct Pack {
    /// The index file.
    index: File,
    /// The pack file.
    data: File,
    /// The size in bytes of an object name.
    hash_length: usize,
    /// How many objects the pack holds.
    count: u64,
    /// The size in bytes of the pack file.
    data_length: u64,
    /// The cumulative fanout counts by first byte.
    fanout: Vec<u32>,
    /// Which index layout this pack's index holds.
    layout: IndexLayout,
    /// How many entries were decoded;
    /// a stand-in for the work a read costs.
    decoded: std::cell::Cell<u64>,
    /// The entries decoded so far by their offset,
    /// so the base that many versions of one chain share is decoded once.
    cache: std::cell::RefCell<std::collections::HashMap<u64, RawObject>>,
    /// The bytes held in `cache`;
    /// once it passes `CACHE_BYTE_LIMIT` no entry is added.
    cached_bytes: std::cell::Cell<usize>,
    /// How many reads of the index file the lookups made;
    /// a stand-in for the system calls a lookup costs.
    index_reads: std::cell::Cell<u64>,
    /// The names of each first byte already read from the index,
    /// by that byte,
    /// so a second lookup in the same bucket reads
    ///  nothing:
    /// the lookups of one commit are many and spread over every first byte.
    buckets: std::cell::RefCell<std::collections::HashMap<u8, std::rc::Rc<Vec<u8>>>>,
    /// The bytes held in `buckets`;
    /// once it passes `CACHE_BYTE_LIMIT` no bucket is added.
    bucket_bytes: std::cell::Cell<usize>,
}

/// Read exactly `length` bytes at `position`,
/// or `None`.
fn read_at(file: &File, position: u64, length: usize) -> Option<Vec<u8>> {
    let mut buffer: Vec<u8> = vec![0; length];
    file.read_exact_at(buffer.as_mut_slice(), position).ok()?;
    return Some(buffer);
}

/// One delta size varint at `cursor`:
/// seven bits per byte,
/// low first.
fn delta_size(delta: &[u8], cursor: &mut usize) -> Option<usize> {
    let mut size: usize = 0;
    let mut shift: u32 = 0;
    loop {
        let byte: u8 = *delta.get(*cursor)?;
        *cursor += 1;
        size |= usize::from(byte & 0x7f).checked_shl(shift)?;
        if byte & 0x80 == 0 {
            return Some(size);
        }
        shift += 7;
        if shift > 56 {
            return None;
        }
    }
}

/// What:
///  Apply a Git delta to its base.
/// Why:
///   A delta states the base size and the result size and then copies from the base or inserts bytes;
/// every number is
///  checked,
/// because a corrupt delta must be refused and not read past a buffer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyDelta(base: Uint8Array, delta: Uint8Array): Uint8Array | undefined;
/// ```
pub fn apply_delta(base: &[u8], delta: &[u8]) -> Option<Vec<u8>> {
    let mut cursor: usize = 0;
    if delta_size(delta, &mut cursor)? != base.len() {
        return None;
    }
    let result_size: usize = delta_size(delta, &mut cursor)?;
    if result_size > OBJECT_SIZE_LIMIT {
        return None;
    }
    let mut result: Vec<u8> = Vec::with_capacity(result_size);
    while cursor < delta.len() {
        let op: u8 = delta[cursor];
        cursor += 1;
        if op & 0x80 != 0 {
            let mut offset: usize = 0;
            let mut length: usize = 0;
            for bit in 0..4 {
                if op & (1 << bit) != 0 {
                    offset |= usize::from(*delta.get(cursor)?) << (8 * bit);
                    cursor += 1;
                }
            }
            for bit in 0..3 {
                if op & (0x10 << bit) != 0 {
                    length |= usize::from(*delta.get(cursor)?) << (8 * bit);
                    cursor += 1;
                }
            }
            if length == 0 {
                length = 0x10000;
            }
            result.extend_from_slice(base.get(offset..offset.checked_add(length)?)?);
        } else if op != 0 {
            let end: usize = cursor.checked_add(usize::from(op))?;
            result.extend_from_slice(delta.get(cursor..end)?);
            cursor = end;
        } else {
            return None;
        }
        if result.len() > result_size {
            return None;
        }
    }
    if result.len() != result_size {
        return None;
    }
    return Some(result);
}

/// What:
///  Where a pack finds the base of a `REF_DELTA`,
/// which is named by object and may live outside the pack.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The database that owns the pack also owns the other packs and the loose objects,
/// and it follows a delta chain one step
///  deeper each time;
/// a type that holds the database and the depth so far implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface BaseByName { baseNamed(name: Uint8Array): RawObject | undefined }
/// ```
/// What:
///  One pack entry's header,
///  as the bytes of the pack state it.
/// Why:
///   A base entry states its object's kind and content size,
/// and a delta states the distance to the entry below it or the name of the base it stands on,
/// and how long its own delta bytes are.
pub enum Entry {
    /// An object stored whole.
    Base {
        /// What the object is.
        kind: ObjectKind,
        /// How many bytes its content holds.
        size: usize,
    },
    /// A delta over the entry `distance` bytes before this one.
    OfsDelta {
        /// How many bytes back the base's entry starts.
        distance: u64,
        /// How many bytes this delta holds.
        size: usize,
    },
    /// A delta over the object `name` names,
    /// which may sit in another pack or be loose.
    RefDelta {
        /// The base object's name.
        name: Vec<u8>,
        /// How many bytes this delta holds.
        size: usize,
    },
}

pub trait BaseByName {
    /// The object named `name`,
    /// or nothing when it cannot be read.
    fn base_named(&self, name: &[u8]) -> Option<RawObject>;
}

impl Pack {
    /// What:
    ///  Open the pack whose index is at `index_path` (the pack is the same name ending `.pack`).
    /// Why:
    ///   Git writes a version 2 index unless a repository asks for version 1,
    /// and reads both,
    ///  so both are read here:
    /// the version 2 magic says which one the file holds,
    /// and a file without it starts with the table of counts a version 1 index begins with.
    ///  `None` unless the layout is one of those two,
    /// its size agrees with its own count,
    /// and the pack states the same count.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pack = Pack.open(idxPath, hashLength); // undefined: not a version 1 or 2 index, or not that pack's index
    /// ```
    pub fn open(index_path: &Path, hash_length: usize) -> Option<Pack> {
        let index: File = File::open(index_path).ok()?;
        let data: File = File::open(index_path.with_extension("pack")).ok()?;
        // The header and the table of counts that follows it are one stretch of the file, read at once.
        let table: Vec<u8> = read_at(&index, 0, FANOUT_END as usize)?;
        let reads: u64 = 1;
        let (layout, counts): (IndexLayout, &[u8]) = match table.starts_with(VERSION_TWO_MAGIC) {
            true => (IndexLayout::Version2, &table[8..]),
            // A version 1 index has no header, so its own counts begin the file and the header read above runs past them.
            false => (IndexLayout::Version1, &table[..FANOUT_ONLY as usize]),
        };
        // A version 1 index holds 20-byte names only, and Git writes none for a SHA-256 repository.
        if layout == IndexLayout::Version1 && hash_length != 20 {
            return None;
        }
        let mut fanout: Vec<u32> = Vec::new();
        for word in counts.as_chunks::<4>().0 {
            fanout.push(u32::from_be_bytes(*word));
        }
        for pair in fanout.windows(2) {
            if pair[0] > pair[1] {
                return None;
            }
        }
        let count: u64 = u64::from(fanout[255]);
        let index_length: u64 = index.metadata().ok()?.len();
        if index_length < layout.smallest(count, hash_length) {
            return None;
        }
        let data_length: u64 = data.metadata().ok()?.len();
        // The pack states its own version and object count; they must agree with the index.
        let pack_header: Vec<u8> = read_at(&data, 0, 12)?;
        let pack_count: u32 = u32::from_be_bytes([pack_header[8], pack_header[9], pack_header[10], pack_header[11]]);
        if &pack_header[..4] != b"PACK" || !matches!(pack_header[7], 2 | 3) || pack_header[4..7] != [0, 0, 0] || u64::from(pack_count) != count {
            return None;
        }
        return Some(Pack { index, data, hash_length, count, data_length, fanout, layout, decoded: std::cell::Cell::new(0), cache: std::cell::RefCell::default(), cached_bytes: std::cell::Cell::new(0), index_reads: std::cell::Cell::new(reads), buckets: std::cell::RefCell::default(), bucket_bytes: std::cell::Cell::new(0) });
    }

    /// How many entries of this pack were decoded (inflated,
    /// with their delta applied) since it was opened.
    pub fn decoded_entries(&self) -> u64 {
        return self.decoded.get();
    }

    /// How many times the lookups of this pack read the index file since it was opened.
    pub fn index_reads(&self) -> u64 {
        return self.index_reads.get();
    }

    /// `length` bytes of the index at `position`,
    /// counted as one read.
    fn read_index(&self, position: u64, length: usize) -> Option<Vec<u8>> {
        self.index_reads.set(self.index_reads.get() + 1);
        return read_at(&self.index, position, length);
    }

    /// The pack offset of the object named `name`,
    /// or `None` when this pack does not hold it.
    pub fn find(&self, name: &[u8]) -> Option<u64> {
        if name.len() != self.hash_length {
            return None;
        }
        let first: usize = usize::from(name[0]);
        let low: u64 = if first == 0 { 0 } else { u64::from(self.fanout[first - 1]) };
        let high: u64 = u64::from(self.fanout[first]);
        if low >= high {
            return None;
        }
        // The names that share the first byte lie side by side, so one read brings all of them and the search below
        //  runs in memory; probing name by name cost a system call for each step of the search. The bucket is kept, because
        //  the next lookup of the commit lands in one of 256 buckets and the same one again soon.
        let record: u64 = self.layout.record_length(self.hash_length);
        let kept: Option<std::rc::Rc<Vec<u8>>> = self.buckets.borrow().get(&name[0]).map(std::rc::Rc::clone);
        let bucket: std::rc::Rc<Vec<u8>> = match kept {
            Some(found) => found,
            None => {
                let read: std::rc::Rc<Vec<u8>> = std::rc::Rc::new(self.read_index(self.layout.records_at() + low * record, usize::try_from(high - low).ok()?.checked_mul(usize::try_from(record).ok()?)?)?);
                if self.bucket_bytes.get() + read.len() <= CACHE_BYTE_LIMIT {
                    self.bucket_bytes.set(self.bucket_bytes.get() + read.len());
                    self.buckets.borrow_mut().insert(name[0], std::rc::Rc::clone(&read));
                }
                read
            }
        };
        let stride: usize = usize::try_from(record).ok()?;
        let inside: usize = usize::try_from(self.layout.name_offset()).ok()?;
        let mut left: usize = 0;
        let mut right: usize = bucket.len() / stride;
        while left < right {
            let middle: usize = left + (right - left) / 2;
            let at: usize = middle * stride + inside;
            let candidate: &[u8] = &bucket[at..at + self.hash_length];
            match candidate.cmp(name) {
                std::cmp::Ordering::Equal => return self.offset_of(low + middle as u64),
                std::cmp::Ordering::Less => left = middle + 1,
                std::cmp::Ordering::Greater => right = middle,
            }
        }
        return None;
    }

    /// What:
    ///  The pack offset stored for the `position`th name.
    /// Why:
    ///   A version 1 index keeps the offset beside the name,
    /// and a version 2 index's word is itself,
    ///  or the number of an entry in its large-offset table when the word's highest bit is set.
    fn offset_of(&self, position: u64) -> Option<u64> {
        let at: u64 = self.layout.offset_at(position, self.count, self.hash_length);
        let bytes_word: Vec<u8> = self.read_index(at, 4)?;
        let word: u32 = u32::from_be_bytes([bytes_word[0], bytes_word[1], bytes_word[2], bytes_word[3]]);
        if word & 0x8000_0000 == 0 {
            return Some(u64::from(word));
        }
        if self.layout == IndexLayout::Version1 {
            return None;
        }
        let large_at: u64 = FANOUT_END + self.count * (self.hash_length as u64 + 4) + self.count * 4 + u64::from(word & 0x7fff_ffff) * 8;
        let bytes: Vec<u8> = self.read_index(large_at, 8)?;
        let mut array: [u8; 8] = [0; 8];
        array.copy_from_slice(bytes.as_slice());
        return Some(u64::from_be_bytes(array));
    }

    /// What:
    ///  Read the object stored at `offset`,
    /// following its chain of deltas down to the entry that stores a base whole;
    /// `by_name` finds the base of a `REF_DELTA`.
    /// Why:
    ///   A packed object is a base object or a delta over another entry of the pack or over any object by name,
    /// and Git follows a chain as deep as the pack holds it:
    /// a repack at `--depth=250` (which `git gc --aggressive` uses) leaves trees at the end of more than a hundred deltas,
    /// so a ceiling of this reader's own would refuse a tree Git reads.
    ///  The walk keeps a stack of its own instead of recursing,
    /// and it ends on three proofs:
    /// every delta states a base at a lower offset,
    ///  so a walk only ends,
    /// and a distance of zero ends it at once (a chain that would come back to its own entry);
    /// a walk past the number of entries the pack states it holds ends it (a chain the pack cannot hold);
    /// and an offset outside the pack ends it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read(offset: number, byName: BaseByName): RawObject | undefined;
    /// ```
    pub fn read(&self, offset: u64, by_name: &dyn BaseByName) -> Option<RawObject> {
        if let Some(kept) = self.cache.borrow().get(&offset) {
            return Some(kept.clone());
        }
        let object: RawObject = self.rebuild(offset, by_name)?;
        self.keep(offset, &object);
        return Some(object);
    }

    /// Keep one rebuilt entry for later reads,
    /// until the cache holds `CACHE_BYTE_LIMIT` bytes.
    fn keep(&self, offset: u64, object: &RawObject) {
        if self.cached_bytes.get() + object.bytes.len() > CACHE_BYTE_LIMIT {
            return;
        }
        self.cached_bytes.set(self.cached_bytes.get() + object.bytes.len());
        self.cache.borrow_mut().insert(offset, object.clone());
    }

    /// What:
    ///  The entry at `offset` rebuilt from the pack file,
    /// as `read` does it when the entry is not kept:
    /// the chain is walked down to a base first,
    /// then each delta is applied on the way back up.
    /// Why:
    ///   A stack of offsets replaces the recursion,
    /// and every entry rebuilt on the way back up is kept,
    /// so one read of a chain's end still costs one decode per entry (the control `a_delta_chain_is_decoded_about_once_for_all_its_versions`).
    fn rebuild(&self, offset: u64, by_name: &dyn BaseByName) -> Option<RawObject> {
        // Each frame: the entry's own offset, where its delta begins, and how many bytes the delta holds.
        let mut frames: Vec<(u64, u64, usize)> = Vec::new();
        let mut walked: u64 = 0;
        let mut at: u64 = offset;
        let mut base: RawObject = loop {
            if let Some(kept) = self.cache.borrow().get(&at) {
                break kept.clone();
            }
            // Every entry of the walk is an entry of the pack, and the pack states how many it holds:
            // a walk longer than that is a chain no pack of this size can hold.
            walked += 1;
            if walked > self.count {
                return None;
            }
            if at < 12 || at >= self.data_length {
                return None;
            }
            self.decoded.set(self.decoded.get() + 1);
            #[cfg(test)]
            DECODED_TOTAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let (entry, header): (Entry, u64) = self.entry_at(at)?;
            match entry {
                Entry::Base { kind, size } => break self.base_object(kind, at.checked_add(header)?, size)?,
                Entry::OfsDelta { distance, size } => {
                    frames.push((at, at.checked_add(header)?, size));
                    if distance == 0 {
                        // A base at the delta's own offset is a chain that never ends; Git's own index refuses it too.
                        return None;
                    }
                    at = at.checked_sub(distance)?;
                }
                Entry::RefDelta { name, size } => {
                    frames.push((at, at.checked_add(header)?, size));
                    break by_name.base_named(name.as_slice())?;
                }
            }
        };
        for (frame, start, length) in frames.iter().rev() {
            let delta: Vec<u8> = self.inflated(*start, *length)?;
            base = RawObject { kind: base.kind, bytes: apply_delta(base.bytes.as_slice(), delta.as_slice())? };
            self.keep(*frame, &base);
        }
        return Some(base);
    }

    /// The header of the entry at `offset` and how many bytes it took.
    fn entry_at(&self, offset: u64) -> Option<(Entry, u64)> {
        // Room for the size and distance varints and, for a `REF_DELTA`, the base name that follows them.
        let head: Vec<u8> = read_at(&self.data, offset, (32 + self.hash_length).min((self.data_length - offset) as usize))?;
        let mut cursor: usize = 0;
        let first: u8 = head[0];
        cursor += 1;
        let code: u8 = (first >> 4) & 7;
        let mut size: usize = usize::from(first & 15);
        let mut shift: u32 = 4;
        let mut byte: u8 = first;
        while byte & 0x80 != 0 {
            byte = *head.get(cursor)?;
            cursor += 1;
            size |= usize::from(byte & 0x7f).checked_shl(shift)?;
            shift += 7;
            if shift > 60 {
                return None;
            }
        }
        let taken: u64 = cursor as u64;
        return match code {
            1 => Some((Entry::Base { kind: ObjectKind::Commit, size }, taken)),
            2 => Some((Entry::Base { kind: ObjectKind::Tree, size }, taken)),
            3 => Some((Entry::Base { kind: ObjectKind::Blob, size }, taken)),
            4 => Some((Entry::Base { kind: ObjectKind::Tag, size }, taken)),
            6 => {
                let mut c: u8 = *head.get(cursor)?;
                cursor += 1;
                let mut distance: u64 = u64::from(c & 0x7f);
                while c & 0x80 != 0 {
                    c = *head.get(cursor)?;
                    cursor += 1;
                    distance = distance.checked_add(1)?.checked_shl(7)? | u64::from(c & 0x7f);
                }
                Some((Entry::OfsDelta { distance, size }, cursor as u64))
            }
            7 => {
                let name: &[u8] = head.get(cursor..cursor + self.hash_length)?;
                Some((Entry::RefDelta { name: name.to_vec(), size }, (cursor + self.hash_length) as u64))
            }
            _ => None,
        };
    }

    /// Inflate the stream at `start` that must be `size` bytes.
    fn inflated(&self, start: u64, size: usize) -> Option<Vec<u8>> {
        // The size comes from the pack; it is checked before any window is allocated for it.
        if size > OBJECT_SIZE_LIMIT {
            return None;
        }
        let remaining: u64 = self.data_length.checked_sub(start)?.checked_sub(self.hash_length as u64)?;
        for slack in [size / 4 + 4096, size * 2 + 65536] {
            let want: usize = usize::try_from(remaining).ok()?.min(size + slack);
            let window: Vec<u8> = read_at(&self.data, start, want)?;
            if let Some(found) = inflate_exact(window.as_slice(), size) {
                return Some(found);
            }
            if want as u64 == remaining {
                break;
            }
        }
        return None;
    }

    /// A non-delta entry.
    fn base_object(&self, kind: ObjectKind, start: u64, size: usize) -> Option<RawObject> {
        return Some(RawObject { kind, bytes: self.inflated(start, size)? });
    }
}

#[cfg(test)]
#[path = "repo_read_pack_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "repo_read_written_packs_tests.rs"]
mod written_tests;
