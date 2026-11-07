//! What:
//!  A reader of one Git pack:
//! the version 2 index lookup by object name and the entry read,
//! with `OFS_DELTA` and
//!  `REF_DELTA` resolution.
//! Why:
//!  Most objects of a real repository are packed.
//!  The reader opens the two files,
//! seeks for the few bytes a lookup needs,
//! and returns `None` for anything it does not
//!  fully understand (a version 1 index,
//! an inconsistent size,
//! a delta chain deeper than the limit,
//! a bad delta),
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

/// The deepest delta chain this reader follows.
const DELTA_DEPTH_LIMIT: usize = 100;

/// The most decoded bytes one pack keeps for later reads:
/// a listing of a large tree reads thousands of trees that share
///  delta bases,
/// and a stop at a fixed size keeps a long listing from growing without bound.
const CACHE_BYTE_LIMIT: usize = 32 * 1024 * 1024;

/// The bytes of the index header and fanout table:
/// magic,
/// version,
/// 256 counts.
const FANOUT_END: u64 = 8 + 256 * 4;

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
pub trait BaseByName {
    /// The object named `name`,
    /// or nothing when it cannot be read.
    fn base_named(&self, name: &[u8]) -> Option<RawObject>;
}

impl Pack {
    /// Open the pack whose index is at `index_path` (the pack is the same name ending `.pack`);
    /// `None` unless the index
    /// is version 2 and its size agrees with its own count.
    pub fn open(index_path: &Path, hash_length: usize) -> Option<Pack> {
        let index: File = File::open(index_path).ok()?;
        let data: File = File::open(index_path.with_extension("pack")).ok()?;
        // The header and the table of counts that follows it are one stretch of the file, read at once.
        let table: Vec<u8> = read_at(&index, 0, FANOUT_END as usize)?;
        let reads: u64 = 1;
        if table[..8] != *b"\xfftOc\0\0\0\x02" {
            return None;
        }
        let mut fanout: Vec<u32> = Vec::new();
        for word in table[8..].as_chunks::<4>().0 {
            fanout.push(u32::from_be_bytes(*word));
        }
        for pair in fanout.windows(2) {
            if pair[0] > pair[1] {
                return None;
            }
        }
        let count: u64 = u64::from(fanout[255]);
        let index_length: u64 = index.metadata().ok()?.len();
        let minimum: u64 = FANOUT_END + count * (hash_length as u64 + 8) + 2 * hash_length as u64;
        if index_length < minimum {
            return None;
        }
        let data_length: u64 = data.metadata().ok()?.len();
        // The pack states its own version and object count; they must agree with the index.
        let pack_header: Vec<u8> = read_at(&data, 0, 12)?;
        let pack_count: u32 = u32::from_be_bytes([pack_header[8], pack_header[9], pack_header[10], pack_header[11]]);
        if &pack_header[..4] != b"PACK" || !matches!(pack_header[7], 2 | 3) || pack_header[4..7] != [0, 0, 0] || u64::from(pack_count) != count {
            return None;
        }
        return Some(Pack { index, data, hash_length, count, data_length, fanout, decoded: std::cell::Cell::new(0), cache: std::cell::RefCell::default(), cached_bytes: std::cell::Cell::new(0), index_reads: std::cell::Cell::new(reads), buckets: std::cell::RefCell::default(), bucket_bytes: std::cell::Cell::new(0) });
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
        let kept: Option<std::rc::Rc<Vec<u8>>> = self.buckets.borrow().get(&name[0]).map(std::rc::Rc::clone);
        let bucket: std::rc::Rc<Vec<u8>> = match kept {
            Some(found) => found,
            None => {
                let read: std::rc::Rc<Vec<u8>> = std::rc::Rc::new(self.read_index(FANOUT_END + low * self.hash_length as u64, ((high - low) as usize) * self.hash_length)?);
                if self.bucket_bytes.get() + read.len() <= CACHE_BYTE_LIMIT {
                    self.bucket_bytes.set(self.bucket_bytes.get() + read.len());
                    self.buckets.borrow_mut().insert(name[0], std::rc::Rc::clone(&read));
                }
                read
            }
        };
        let mut left: usize = 0;
        let mut right: usize = bucket.len() / self.hash_length;
        while left < right {
            let middle: usize = left + (right - left) / 2;
            let candidate: &[u8] = &bucket[middle * self.hash_length..(middle + 1) * self.hash_length];
            match candidate.cmp(name) {
                std::cmp::Ordering::Equal => return self.offset_of(low + middle as u64),
                std::cmp::Ordering::Less => left = middle + 1,
                std::cmp::Ordering::Greater => right = middle,
            }
        }
        return None;
    }

    /// The pack offset stored for the `position`th name,
    /// following the large-offset table.
    fn offset_of(&self, position: u64) -> Option<u64> {
        let offsets_at: u64 = FANOUT_END + self.count * (self.hash_length as u64 + 4);
        let bytes_word: Vec<u8> = self.read_index(offsets_at + position * 4, 4)?;
        let word: u32 = u32::from_be_bytes([bytes_word[0], bytes_word[1], bytes_word[2], bytes_word[3]]);
        if word & 0x8000_0000 == 0 {
            return Some(u64::from(word));
        }
        let large_at: u64 = offsets_at + self.count * 4 + u64::from(word & 0x7fff_ffff) * 8;
        let bytes: Vec<u8> = self.read_index(large_at, 8)?;
        let mut array: [u8; 8] = [0; 8];
        array.copy_from_slice(bytes.as_slice());
        return Some(u64::from_be_bytes(array));
    }

    /// What:
    ///  Read the object stored at `offset`,
    /// resolving deltas;
    /// `by_name` finds the base of a `REF_DELTA`.
    /// Why:
    ///   A packed object is a base object or a delta over another entry of the pack or over any object by name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read(offset: number, byName: BaseByName, depth = 0): RawObject | undefined;
    /// ```
    pub fn read(&self, offset: u64, by_name: &dyn BaseByName, depth: usize) -> Option<RawObject> {
        if let Some(kept) = self.cache.borrow().get(&offset) {
            return Some(kept.clone());
        }
        let object: RawObject = self.decode(offset, by_name, depth)?;
        if self.cached_bytes.get() + object.bytes.len() <= CACHE_BYTE_LIMIT {
            self.cached_bytes.set(self.cached_bytes.get() + object.bytes.len());
            self.cache.borrow_mut().insert(offset, object.clone());
        }
        return Some(object);
    }

    /// The entry at `offset` rebuilt from the pack file,
    /// as `read` does it when the entry is not kept.
    fn decode(&self, offset: u64, by_name: &dyn BaseByName, depth: usize) -> Option<RawObject> {
        if depth > DELTA_DEPTH_LIMIT || offset < 12 || offset >= self.data_length {
            return None;
        }
        self.decoded.set(self.decoded.get() + 1);
        #[cfg(test)]
        DECODED_TOTAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
        match code {
            1 => return self.base_object(ObjectKind::Commit, offset + cursor as u64, size),
            2 => return self.base_object(ObjectKind::Tree, offset + cursor as u64, size),
            3 => return self.base_object(ObjectKind::Blob, offset + cursor as u64, size),
            4 => return self.base_object(ObjectKind::Tag, offset + cursor as u64, size),
            6 => {
                let mut c: u8 = *head.get(cursor)?;
                cursor += 1;
                let mut distance: u64 = u64::from(c & 0x7f);
                while c & 0x80 != 0 {
                    c = *head.get(cursor)?;
                    cursor += 1;
                    distance = distance.checked_add(1)?.checked_shl(7)? | u64::from(c & 0x7f);
                }
                let base: RawObject = self.read(offset.checked_sub(distance)?, by_name, depth + 1)?;
                return self.apply(base, offset + cursor as u64, size);
            }
            7 => {
                let name: &[u8] = head.get(cursor..cursor + self.hash_length)?;
                cursor += self.hash_length;
                let base: RawObject = by_name.base_named(name)?;
                return self.apply(base, offset + cursor as u64, size);
            }
            _ => return None,
        }
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

    /// A delta entry over `base`.
    fn apply(&self, base: RawObject, start: u64, delta_length: usize) -> Option<RawObject> {
        let delta: Vec<u8> = self.inflated(start, delta_length)?;
        return Some(RawObject { kind: base.kind, bytes: apply_delta(base.bytes.as_slice(), delta.as_slice())? });
    }
}

#[cfg(test)]
#[path = "repo_read_pack_tests.rs"]
mod tests;
