//! What:
//!  A reader of one Git pack: the version 2 index lookup by object name and the entry read, with `OFS_DELTA` and
//!  `REF_DELTA` resolution.
//! Why:
//!  Most objects of a real repository are packed.
//!  The reader opens the two files, seeks for the few bytes a lookup needs, and returns `None` for anything it does not
//!  fully understand (a version 1 index, an inconsistent size, a delta chain deeper than the limit, a bad delta),
//!  which sends the caller to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pack = openPack(idxPath, hashLength); const object = pack.read(pack.find(name), resolveByName);
//! ```

#![cfg(unix)]

use super::repo_read_object::{ObjectKind, OBJECT_SIZE_LIMIT, RawObject, inflate_exact};
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::Path;

/// The deepest delta chain this reader follows.
const DELTA_DEPTH_LIMIT: usize = 100;

/// The bytes of the index header and fanout table: magic, version, 256 counts.
const FANOUT_END: u64 = 8 + 256 * 4;

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
}

/// Read exactly `length` bytes at `position`, or `None`.
fn read_at(file: &File, position: u64, length: usize) -> Option<Vec<u8>> {
    let mut buffer: Vec<u8> = vec![0; length];
    file.read_exact_at(buffer.as_mut_slice(), position).ok()?;
    return Some(buffer);
}

/// A big-endian `u32` at `position`.
fn be32(file: &File, position: u64) -> Option<u32> {
    let bytes: Vec<u8> = read_at(file, position, 4)?;
    return Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
}

/// One delta size varint at `cursor`: seven bits per byte, low first.
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
///   A delta states the base size and the result size and then copies from the base or inserts bytes; every number is
///  checked, because a corrupt delta must be refused and not read past a buffer.
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

impl Pack {
    /// Open the pack whose index is at `index_path` (the pack is the same name ending `.pack`); `None` unless the index
    /// is version 2 and its size agrees with its own count.
    pub fn open(index_path: &Path, hash_length: usize) -> Option<Pack> {
        let index: File = File::open(index_path).ok()?;
        let data: File = File::open(index_path.with_extension("pack")).ok()?;
        let header: Vec<u8> = read_at(&index, 0, 8)?;
        if header != b"\xfftOc\0\0\0\x02" {
            return None;
        }
        let mut fanout: Vec<u32> = Vec::with_capacity(256);
        for slot in 0..256_u64 {
            fanout.push(be32(&index, 8 + slot * 4)?);
        }
        if fanout.windows(2).any(|pair| { return pair[0] > pair[1]; }) {
            return None;
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
        return Some(Pack { index, data, hash_length, count, data_length, fanout });
    }

    /// The pack offset of the object named `name`, or `None` when this pack does not hold it.
    pub fn find(&self, name: &[u8]) -> Option<u64> {
        if name.len() != self.hash_length {
            return None;
        }
        let first: usize = usize::from(name[0]);
        let mut low: u64 = if first == 0 { 0 } else { u64::from(self.fanout[first - 1]) };
        let mut high: u64 = u64::from(self.fanout[first]);
        let names_at: u64 = FANOUT_END;
        while low < high {
            let middle: u64 = low + (high - low) / 2;
            let candidate: Vec<u8> = read_at(&self.index, names_at + middle * self.hash_length as u64, self.hash_length)?;
            match candidate.as_slice().cmp(name) {
                std::cmp::Ordering::Equal => return self.offset_of(middle),
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
            }
        }
        return None;
    }

    /// The pack offset stored for the `position`th name, following the large-offset table.
    fn offset_of(&self, position: u64) -> Option<u64> {
        let offsets_at: u64 = FANOUT_END + self.count * (self.hash_length as u64 + 4);
        let word: u32 = be32(&self.index, offsets_at + position * 4)?;
        if word & 0x8000_0000 == 0 {
            return Some(u64::from(word));
        }
        let large_at: u64 = offsets_at + self.count * 4 + u64::from(word & 0x7fff_ffff) * 8;
        let bytes: Vec<u8> = read_at(&self.index, large_at, 8)?;
        let mut array: [u8; 8] = [0; 8];
        array.copy_from_slice(bytes.as_slice());
        return Some(u64::from_be_bytes(array));
    }

    /// What:
    ///  Read the object stored at `offset`, resolving deltas; `by_name` finds the base of a `REF_DELTA`.
    /// Why:
    ///   A packed object is a base object or a delta over another entry of the pack or over any object by name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read(offset: number, byName: (name: Uint8Array) => RawObject | undefined, depth = 0): RawObject | undefined;
    /// ```
    pub fn read(&self, offset: u64, by_name: &dyn Fn(&[u8]) -> Option<RawObject>, depth: usize) -> Option<RawObject> {
        if depth > DELTA_DEPTH_LIMIT || offset < 12 || offset >= self.data_length {
            return None;
        }
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
                let base: RawObject = by_name(name)?;
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
