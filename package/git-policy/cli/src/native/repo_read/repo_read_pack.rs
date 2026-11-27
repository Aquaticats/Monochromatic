//! What:
//!  A reader of one Git pack's index:
//! the version 2 index lookup by object name,
//!  and the offset each name answers with.
//! Why:
//!  Most objects of a real repository are packed.
//!  The reader opens the two files,
//! seeks for the few bytes a lookup needs,
//!  and returns `None` for an index it does not
//!  fully understand (a version 1 index,
//! an inconsistent size),
//!  which sends the caller to Git.
//!  What the bytes at one offset are is another job:
//! `the_entries_of_one_pack.rs` reads an entry and walks its delta chain,
//!  and `applying_a_git_delta.rs` holds what a delta means.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pack = openPack(idxPath, hashLength); const offset = pack.find(name);
//! ```

#![cfg(unix)]

use crate::repo_read_object::RawObject;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::Path;

/// The most decoded bytes one pack keeps for later reads:
/// a listing of a large tree reads thousands of trees that share
///  delta bases,
/// and a stop at a fixed size keeps a long listing from growing without bound.
pub(super) const CACHE_BYTE_LIMIT: usize = 32 * 1024 * 1024;

/// The bytes of the index header and fanout table:
/// magic,
/// version,
/// 256 counts.
const FANOUT_END: u64 = 8 + 256 * 4;

/// How many pack entries every pack of this process decoded,
/// for a control that compares the work of two ways to answer.
#[cfg(test)]
pub(crate) static DECODED_TOTAL: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// One open pack and its version 2 index.
#[derive(Debug)]
pub struct Pack {
    /// The index file.
    pub(super) index: File,
    /// The pack file.
    pub(super) data: File,
    /// The size in bytes of an object name.
    pub(super) hash_length: usize,
    /// How many objects the pack holds.
    pub(super) count: u64,
    /// The size in bytes of the pack file.
    pub(super) data_length: u64,
    /// The cumulative fanout counts by first byte.
    pub(super) fanout: Vec<u32>,
    /// How many entries were decoded;
    /// a stand-in for the work a read costs.
    pub(super) decoded: std::cell::Cell<u64>,
    /// The entries decoded so far by their offset,
    /// so the base that many versions of one chain share is decoded once.
    pub(super) cache: std::cell::RefCell<std::collections::HashMap<u64, RawObject>>,
    /// The bytes held in `cache`;
    /// once it passes `CACHE_BYTE_LIMIT` no entry is added.
    pub(super) cached_bytes: std::cell::Cell<usize>,
    /// How many reads of the index file the lookups made;
    /// a stand-in for the system calls a lookup costs.
    pub(super) index_reads: std::cell::Cell<u64>,
    /// The names of each first byte already read from the index,
    /// by that byte,
    /// so a second lookup in the same bucket reads
    ///  nothing:
    /// the lookups of one commit are many and spread over every first byte.
    pub(super) buckets: std::cell::RefCell<std::collections::HashMap<u8, std::rc::Rc<Vec<u8>>>>,
    /// The bytes held in `buckets`;
    /// once it passes `CACHE_BYTE_LIMIT` no bucket is added.
    pub(super) bucket_bytes: std::cell::Cell<usize>,
}

/// Read exactly `length` bytes at `position`,
/// or `None`.
pub(super) fn read_at(
    file: &File,
    position: u64,
    length: usize,
) -> Option<Vec<u8>> {
    let mut buffer: Vec<u8> = vec![0; length];
    file.read_exact_at(buffer.as_mut_slice(), position).ok()?;
    return Some(buffer);
}

pub use crate::applying_a_git_delta::{
    BaseByName,
    apply_delta,
};

impl Pack {
    /// Open the pack whose index is at `index_path` (the pack is the same name ending `.pack`);
    /// `None` unless the index
    /// is version 2 and its size agrees with its own count.
    pub fn open(
        index_path: &Path,
        hash_length: usize,
    ) -> Option<Pack> {
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
        let pack_count: u32 = u32::from_be_bytes([
            pack_header[8],
            pack_header[9],
            pack_header[10],
            pack_header[11],
        ]);
        if &pack_header[..4] != b"PACK"
            || !matches!(pack_header[7], 2 | 3)
            || pack_header[4..7] != [0, 0, 0]
            || u64::from(pack_count) != count
        {
            return None;
        }
        return Some(Pack {
            index,
            data,
            hash_length,
            count,
            data_length,
            fanout,
            decoded: std::cell::Cell::new(0),
            cache: std::cell::RefCell::default(),
            cached_bytes: std::cell::Cell::new(0),
            index_reads: std::cell::Cell::new(reads),
            buckets: std::cell::RefCell::default(),
            bucket_bytes: std::cell::Cell::new(0),
        });
    }

    /// How many times the lookups of this pack read the index file since it was opened.
    pub fn index_reads(&self) -> u64 {
        return self.index_reads.get();
    }

    /// `length` bytes of the index at `position`,
    /// counted as one read.
    fn read_index(
        &self,
        position: u64,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.index_reads.set(self.index_reads.get() + 1);
        return read_at(&self.index, position, length);
    }

    /// The pack offset of the object named `name`,
    /// or `None` when this pack does not hold it.
    pub fn find(
        &self,
        name: &[u8],
    ) -> Option<u64> {
        if name.len() != self.hash_length {
            return None;
        }
        let first: usize = usize::from(name[0]);
        let low: u64 = if first == 0 {
            0
        } else {
            u64::from(self.fanout[first - 1])
        };
        let high: u64 = u64::from(self.fanout[first]);
        if low >= high {
            return None;
        }
        // The names that share the first byte lie side by side, so one read brings all of them and the search below
        //  runs in memory; probing name by name cost a system call for each step of the search. The bucket is kept, because
        //  the next lookup of the commit lands in one of 256 buckets and the same one again soon.
        let kept: Option<std::rc::Rc<Vec<u8>>> =
            self.buckets.borrow().get(&name[0]).map(std::rc::Rc::clone);
        let bucket: std::rc::Rc<Vec<u8>> = match kept {
            Some(found) => found,
            None => {
                let read: std::rc::Rc<Vec<u8>> = std::rc::Rc::new(self.read_index(
                    FANOUT_END + low * self.hash_length as u64,
                    ((high - low) as usize) * self.hash_length,
                )?);
                if self.bucket_bytes.get() + read.len() <= CACHE_BYTE_LIMIT {
                    self.bucket_bytes.set(self.bucket_bytes.get() + read.len());
                    self.buckets
                        .borrow_mut()
                        .insert(name[0], std::rc::Rc::clone(&read));
                }
                read
            }
        };
        let mut left: usize = 0;
        let mut right: usize = bucket.len() / self.hash_length;
        while left < right {
            let middle: usize = left + (right - left) / 2;
            let candidate: &[u8] =
                &bucket[middle * self.hash_length..(middle + 1) * self.hash_length];
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
    fn offset_of(
        &self,
        position: u64,
    ) -> Option<u64> {
        let offsets_at: u64 = FANOUT_END + self.count * (self.hash_length as u64 + 4);
        let bytes_word: Vec<u8> = self.read_index(offsets_at + position * 4, 4)?;
        let word: u32 =
            u32::from_be_bytes([bytes_word[0], bytes_word[1], bytes_word[2], bytes_word[3]]);
        if word & 0x8000_0000 == 0 {
            return Some(u64::from(word));
        }
        let large_at: u64 = offsets_at + self.count * 4 + u64::from(word & 0x7fff_ffff) * 8;
        let bytes: Vec<u8> = self.read_index(large_at, 8)?;
        let mut array: [u8; 8] = [0; 8];
        array.copy_from_slice(bytes.as_slice());
        return Some(u64::from_be_bytes(array));
    }
}

#[cfg(test)]
#[path = "repo_read_pack_tests.rs"]
mod tests;
