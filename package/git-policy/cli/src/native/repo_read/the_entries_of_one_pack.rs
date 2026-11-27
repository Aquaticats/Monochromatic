//! What:
//!  One pack's entries:
//!  the object stored at an offset,
//!  the base a `REF_DELTA` or an `OFS_DELTA` names,
//!  and the window each entry is inflated from.
//! Why:
//!  The pack's index says where an object starts (`repo_read_pack.rs`),
//!  and this file says what the bytes at that offset are.
//!  A delta chain is followed one entry at a time,
//!  and an entry decoded once is kept for the questions that follow.
//!  A delta this reader cannot trust
//!  (a chain deeper than the limit,
//! a bad delta) returns `None`,
//!  which sends the caller to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const object = pack.read(offset, byName, depth); // undefined: start Git
//! ```

#![cfg(unix)]

use crate::applying_a_git_delta::{
    BaseByName,
    apply_delta,
};
use crate::repo_read_object::{
    OBJECT_SIZE_LIMIT,
    ObjectKind,
    RawObject,
    inflate_exact,
};
use crate::repo_read_pack::{
    CACHE_BYTE_LIMIT,
    Pack,
    read_at,
};

/// The deepest delta chain this reader follows.
const DELTA_DEPTH_LIMIT: usize = 100;

impl Pack {
    /// How many entries of this pack were decoded (inflated,
    /// with their delta applied) since it was opened.
    pub fn decoded_entries(&self) -> u64 {
        return self.decoded.get();
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
    pub fn read(
        &self,
        offset: u64,
        by_name: &dyn BaseByName,
        depth: usize,
    ) -> Option<RawObject> {
        if let Some(kept) = self.cache.borrow().get(&offset) {
            return Some(kept.clone());
        }
        let object: RawObject = self.decode(offset, by_name, depth)?;
        if self.cached_bytes.get() + object.bytes.len() <= CACHE_BYTE_LIMIT {
            self.cached_bytes
                .set(self.cached_bytes.get() + object.bytes.len());
            self.cache.borrow_mut().insert(offset, object.clone());
        }
        return Some(object);
    }

    /// The entry at `offset` rebuilt from the pack file,
    /// as `read` does it when the entry is not kept.
    fn decode(
        &self,
        offset: u64,
        by_name: &dyn BaseByName,
        depth: usize,
    ) -> Option<RawObject> {
        if depth > DELTA_DEPTH_LIMIT || offset < 12 || offset >= self.data_length {
            return None;
        }
        self.decoded.set(self.decoded.get() + 1);
        #[cfg(test)]
        crate::repo_read_pack::DECODED_TOTAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // Room for the size and distance varints and, for a `REF_DELTA`, the base name that follows them.
        let head: Vec<u8> = read_at(
            &self.data,
            offset,
            (32 + self.hash_length).min((self.data_length - offset) as usize),
        )?;
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
                let base: RawObject =
                    self.read(offset.checked_sub(distance)?, by_name, depth + 1)?;
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
    fn inflated(
        &self,
        start: u64,
        size: usize,
    ) -> Option<Vec<u8>> {
        // The size comes from the pack; it is checked before any window is allocated for it.
        if size > OBJECT_SIZE_LIMIT {
            return None;
        }
        let remaining: u64 = self
            .data_length
            .checked_sub(start)?
            .checked_sub(self.hash_length as u64)?;
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
    fn base_object(
        &self,
        kind: ObjectKind,
        start: u64,
        size: usize,
    ) -> Option<RawObject> {
        return Some(RawObject {
            kind,
            bytes: self.inflated(start, size)?,
        });
    }

    /// A delta entry over `base`.
    fn apply(
        &self,
        base: RawObject,
        start: u64,
        delta_length: usize,
    ) -> Option<RawObject> {
        let delta: Vec<u8> = self.inflated(start, delta_length)?;
        return Some(RawObject {
            kind: base.kind,
            bytes: apply_delta(base.bytes.as_slice(), delta.as_slice())?,
        });
    }
}
