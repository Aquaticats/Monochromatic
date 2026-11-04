//! What:
//!  The `FSMN` (fsmonitor) index extension: the last-update token and the bitmap of entries not known to be unchanged
//!  (`fsmonitor.c`: `read_fsmonitor_extension`, `write_fsmonitor_extension`, `fill_fsmonitor_bitmap`).
//! Why:
//!  When fsmonitor is on, Git keeps the extension across an index write and rebuilds its bitmap from the entries: an entry
//!  it just made is not marked valid, so it is listed as dirty, and the positions follow the entries that remain. The token
//!  is kept as read; nothing here asks the monitor anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { token, dirty } = readFsmonitor(bytes); const out = writeFsmonitor(token, dirtyPositions);
//! ```

use crate::index_ewah::Ewah;
use crate::index_stat::be32;

/// The extension version Git writes, which carries the token as text.
const VERSION_TEXT_TOKEN: u32 = 2;

/// The older version, which carries a 64-bit timestamp that Git reads as a decimal token.
const VERSION_TIMESTAMP: u32 = 1;

/// The extension as read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fsmonitor {
    /// The token of the last query, kept as read.
    pub token: Vec<u8>,
    /// The positions of the entries that were not valid when the index was written.
    pub dirty: Vec<usize>,
}

/// The extension from its bytes, or `None` for a form Git would reject; its bitmap may name at most `entries` entries (Git
///  stops with an error on a longer one, `assert_index_minimum`).
pub fn read_fsmonitor(data: &[u8], entries: usize) -> Option<Fsmonitor> {
    let version: u32 = be32(data, 0)?;
    let (token, after): (Vec<u8>, usize) = if version == VERSION_TIMESTAMP {
        let stamp: u64 = u64::from_be_bytes(data.get(4..12)?.try_into().ok()?);
        (stamp.to_string().into_bytes(), 12)
    } else if version == VERSION_TEXT_TOKEN {
        let nul: usize = data.get(4..)?.iter().position(|byte| { return *byte == 0; })?;
        (data[4..4 + nul].to_vec(), 4 + nul + 1)
    } else {
        return None;
    };
    let size: usize = be32(data, after)? as usize;
    let bitmap_bytes: &[u8] = data.get(after + 4..after + 4 + size)?;
    let (bitmap, used) = Ewah::read(bitmap_bytes, entries)?;
    // Git requires the bitmap to use exactly the size the extension states, and nothing after it is read.
    if used != size || after + 4 + size != data.len() {
        return None;
    }
    return Some(Fsmonitor { token, dirty: bitmap.bits() });
}

/// The extension's bytes in version 2, for the entries at `dirty` (increasing) not being valid.
pub fn write_fsmonitor(token: &[u8], dirty: &[usize]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(&VERSION_TEXT_TOKEN.to_be_bytes());
    out.extend_from_slice(token);
    out.push(0);
    let mut bitmap: Vec<u8> = Vec::new();
    Ewah::from_bits(dirty).serialize(&mut bitmap);
    out.extend_from_slice(&(bitmap.len() as u32).to_be_bytes());
    out.extend_from_slice(bitmap.as_slice());
    return out;
}

#[cfg(test)]
#[path = "index_fsmonitor_tests.rs"]
mod tests;
