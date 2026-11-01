//! What:
//!  The length and a 64-bit FNV-1a hash of a file's bytes,
//!  the evidence a landing record keeps about each index snapshot.
//! Why:
//!  The index snapshots of a landing are written without a sync.
//!  A power loss can leave one of them with fewer bytes than were written,
//!  or with bytes that never reached the disk,
//!  while the synced landing record and the moved branch survive.
//!  Recovery installs the post-index as the real index,
//!  so it compares the file with this evidence first and never installs a snapshot that differs.
//!  FNV-1a is specified by its two constants and needs no dependency,
//!  and it is stable across Rust versions and builds,
//!  which `std::hash::DefaultHasher` is not;
//!  it detects damage and is not meant to resist someone who edits the file on purpose.
//!  Measured in a release build over a 521,631-byte index (5,000 entries): 0.38 ms per file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const digest = { length: bytes.length, fnv1a64: fnv1a64(bytes).toString(16).padStart(16, '0') };
//! ```

/// `Read` pulls bytes from a file in chunks.
use std::io::Read;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// The FNV-1a 64-bit offset basis.
const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// The FNV-1a 64-bit prime.
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// How many bytes one read takes,
///  so a snapshot of any size is hashed in bounded memory.
const CHUNK_BYTES: usize = 64 * 1024;

/// What:
///  The length and hash of one file's bytes.
/// Why:
///   A truncated file differs in length and a file with unwritten or foreign bytes differs in hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ContentDigest = { length: number; fnv1a64: bigint };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContentDigest {
    /// Number of bytes.
    pub length: u64,
    /// FNV-1a 64-bit hash of the bytes.
    pub fnv1a64: u64,
}

impl ContentDigest {
    /// What:
    ///  The hash as the 16 lowercase hexadecimal digits a record stores.
    /// Why:
    ///   JSON numbers cannot carry 64 bits exactly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// digest.fnv1a64.toString(16).padStart(16, '0')
    /// ```
    pub fn hash_text(&self) -> String {
        return format!("{:016x}", self.fnv1a64);
    }

    /// What:
    ///  The digest a record's length and hexadecimal hash text describe,
    ///  or nothing when the text is not exactly 16 lowercase hexadecimal digits.
    /// Why:
    ///   A record that spells the hash any other way is damaged,
    ///  and the caller reports it as a malformed record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// /^[0-9a-f]{16}$/.test(text) ? { length, fnv1a64: BigInt(`0x${text}`) } : undefined
    /// ```
    pub fn from_record(length: u64, hash_text: &str) -> Option<ContentDigest> {
        let well_formed: bool = hash_text.len() == 16
            && hash_text
                .bytes()
                .all(|byte| return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if !well_formed {
            return None;
        }
        let fnv1a64: u64 = u64::from_str_radix(hash_text, 16).ok()?;
        return Some(ContentDigest { length, fnv1a64 });
    }
}

/// What:
///  Fold more bytes into a running FNV-1a hash.
/// Why:
///   A file is hashed chunk by chunk with the same result as hashing it whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const byte of bytes) { hash ^= BigInt(byte); hash = (hash * PRIME) & MASK_64; }
/// ```
fn fold(hash: u64, bytes: &[u8]) -> u64 {
    let mut running: u64 = hash;
    for byte in bytes {
        running ^= u64::from(*byte);
        running = running.wrapping_mul(PRIME);
    }
    return running;
}

/// What:
///  The digest of in-memory bytes.
/// Why:
///   Tests pin the published FNV-1a vectors through it,
///  and the file reader below must agree with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const digest = digestBytes(bytes);
/// ```
pub fn digest_bytes(bytes: &[u8]) -> ContentDigest {
    return ContentDigest {
        length: bytes.len() as u64,
        fnv1a64: fold(OFFSET_BASIS, bytes),
    };
}

/// What:
///  The digest of a file's current bytes,
///  read in bounded chunks.
/// Why:
///   The landing records it for both snapshots after they are written,
///  and recovery recomputes it from the pinned snapshot before installing anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const digest = await digestFile(path);
/// ```
pub fn digest_file(path: &Path) -> std::io::Result<ContentDigest> {
    let mut file: std::fs::File = std::fs::File::open(path)?;
    let mut buffer: Vec<u8> = vec![0; CHUNK_BYTES];
    let mut hash: u64 = OFFSET_BASIS;
    let mut length: u64 = 0;
    loop {
        let read: usize = match file.read(buffer.as_mut_slice()) {
            Ok(count) => count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        if read == 0 {
            break;
        }
        hash = fold(hash, &buffer[..read]);
        length += read as u64;
    }
    return Ok(ContentDigest {
        length,
        fnv1a64: hash,
    });
}

/// Digest controls stay out of the release executable.
#[cfg(test)]
#[path = "content_digest_tests.rs"]
mod tests;
