//! What:
//! The check that an object name in a reference names a commit that exists,
//! made from the object files of a repository (a loose object, or an entry of a pack index) without starting Git.
//! Why:
//! Git's `X^{commit}` establishes that the object exists, is a commit (a tag is peeled), and is not replaced.
//! A reference file alone proves none of it,
//! so the reader accepts a commit only after it has seen the object's type,
//! and everything it cannot see (alternates, grafts, replace references, deltified or tag objects, a missing object)
//! is an error that leaves the answer to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-objects.ts
//! export function confirmCommit(commonDir: string, object: string): void  // throws when the object is not seen to be a commit
//! ```

use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

use flate2::read::ZlibDecoder;

/// The object name length of a SHA-1 repository, in hexadecimal digits.
pub const OBJECT_NAME_DIGITS: usize = 40;

/// The bytes of a SHA-1 object name.
const OBJECT_NAME_BYTES: usize = OBJECT_NAME_DIGITS / 2;

/// The signature and version that start a version 2 pack index.
const PACK_INDEX_HEADER: [u8; 8] = [0xff, b't', b'O', b'c', 0, 0, 0, 2];

/// The number of fan-out entries of a pack index.
const FANOUT_ENTRIES: usize = 256;

/// The pack entry type of a commit that is stored whole.
const PACK_TYPE_COMMIT: u8 = 1;

/// The most bytes of a loose object read to learn its type (`commit 123456789\0` is far shorter).
const LOOSE_HEADER_BYTES: usize = 32;

/// What:
/// `fn hex_to_bytes(hex: &str) -> Option<[u8; OBJECT_NAME_BYTES]>` decodes an object name.
/// Why:
/// The pack index stores names as raw bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hexToBytes = (hex: string): Uint8Array | undefined => ...
/// ```
fn hex_to_bytes(hex: &str) -> Option<[u8; OBJECT_NAME_BYTES]> {
    if hex.len() != OBJECT_NAME_DIGITS {
        return None;
    }
    let mut bytes: [u8; OBJECT_NAME_BYTES] = [0; OBJECT_NAME_BYTES];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(index * 2..index * 2 + 2)?, 16).ok()?;
    }
    return Some(bytes);
}

/// What:
/// `fn loose_type(path: &Path) -> Result<String, String>` reads the type word of a loose object from its compressed header.
/// Why:
/// A loose object starts with `<type> <size>\0` inside the zlib stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function looseType(path: string): string  // throws on an unreadable object
/// ```
fn loose_type(path: &Path) -> Result<String, String> {
    let file: File = File::open(path).map_err(|error| { return format!("{} cannot be read: {error}", path.display()); })?;
    let mut header: Vec<u8> = Vec::new();
    ZlibDecoder::new(file)
        .take(LOOSE_HEADER_BYTES as u64)
        .read_to_end(&mut header)
        .map_err(|error| { return format!("{} is not a readable object: {error}", path.display()); })?;
    let word: &[u8] = header.split(|byte| { return *byte == b' '; }).next().unwrap_or(&[]);
    return Ok(String::from_utf8_lossy(word).to_string());
}

/// What:
/// `fn read_at(file: &File, offset: u64, length: usize) -> std::io::Result<Vec<u8>>` reads exactly `length` bytes at `offset`.
/// Why:
/// A pack index is searched by positions,
/// and reading only the needed bytes keeps a large index out of memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readAt = (file: number, offset: number, length: number): Buffer => ...
/// ```
fn read_at(file: &File, offset: u64, length: usize) -> std::io::Result<Vec<u8>> {
    let mut bytes: Vec<u8> = vec![0; length];
    file.read_exact_at(&mut bytes, offset)?;
    return Ok(bytes);
}

/// What:
/// `fn big_endian(bytes: &[u8]) -> u64` decodes a big-endian unsigned integer.
/// Why:
/// Pack indexes store their numbers that way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bigEndian = (bytes: Uint8Array): bigint => ...
/// ```
fn big_endian(bytes: &[u8]) -> u64 {
    return bytes.iter().fold(0_u64, |total, byte| { return (total << 8) | u64::from(*byte); });
}

/// What:
/// `fn pack_offset(index: &File, name: &[u8; OBJECT_NAME_BYTES]) -> Result<Option<u64>, String>` finds the offset of an object in
/// a version 2 pack index, or `None` when the index does not list it.
/// Why:
/// The fan-out table narrows the search to the names that start with the same byte,
/// then a binary search over the sorted names finds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function packOffset(index: number, name: Uint8Array): number | undefined
/// ```
fn pack_offset(index: &File, name: &[u8; OBJECT_NAME_BYTES]) -> Result<Option<u64>, String> {
    let failed = |error: std::io::Error| { return format!("a pack index cannot be read: {error}"); };
    let header: Vec<u8> = read_at(index, 0, PACK_INDEX_HEADER.len()).map_err(failed)?;
    if header != PACK_INDEX_HEADER {
        return Err("a pack index is not of version 2".to_string());
    }
    let fanout_start: u64 = PACK_INDEX_HEADER.len() as u64;
    let fanout: Vec<u8> = read_at(index, fanout_start, FANOUT_ENTRIES * 4).map_err(failed)?;
    let entry = |position: usize| { return big_endian(&fanout[position * 4..position * 4 + 4]); };
    let total: u64 = entry(FANOUT_ENTRIES - 1);
    let first_byte: usize = usize::from(name[0]);
    let mut low: u64 = if first_byte == 0 { 0 } else { entry(first_byte - 1) };
    let mut high: u64 = entry(first_byte);
    let names_start: u64 = fanout_start + (FANOUT_ENTRIES as u64) * 4;
    while low < high {
        let middle: u64 = low + (high - low) / 2;
        let found: Vec<u8> = read_at(index, names_start + middle * OBJECT_NAME_BYTES as u64, OBJECT_NAME_BYTES).map_err(failed)?;
        match found.as_slice().cmp(name.as_slice()) {
            std::cmp::Ordering::Equal => {
                let offsets_start: u64 = names_start + total * OBJECT_NAME_BYTES as u64 + total * 4;
                let small: u64 = big_endian(&read_at(index, offsets_start + middle * 4, 4).map_err(failed)?);
                if small & 0x8000_0000 == 0 {
                    return Ok(Some(small));
                }
                let large_start: u64 = offsets_start + total * 4;
                let large: Vec<u8> = read_at(index, large_start + (small & 0x7fff_ffff) * 8, 8).map_err(failed)?;
                return Ok(Some(big_endian(&large)));
            }
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle,
        }
    }
    return Ok(None);
}

/// What:
/// `fn packed_commit(objects: &Path, name: &[u8; OBJECT_NAME_BYTES]) -> Result<bool, String>` says whether some pack stores the
/// object as a whole commit.
/// Why:
/// The type is in the top bits of the first byte of the pack entry;
/// a tag, tree, blob or deltified entry is not a commit this reader can vouch for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function packedCommit(objects: string, name: Uint8Array): boolean  // throws when a pack cannot be read
/// ```
fn packed_commit(objects: &Path, name: &[u8; OBJECT_NAME_BYTES]) -> Result<bool, String> {
    let pack_directory: PathBuf = objects.join("pack");
    let listing = match fs::read_dir(&pack_directory) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("{} cannot be listed: {error}", pack_directory.display())),
    };
    for item in listing {
        let path: PathBuf = item.map_err(|error| { return format!("{} cannot be listed: {error}", pack_directory.display()); })?.path();
        if path.extension().and_then(|extension| { return extension.to_str(); }) != Some("idx") {
            continue;
        }
        let index: File = File::open(&path).map_err(|error| { return format!("{} cannot be read: {error}", path.display()); })?;
        let Some(offset) = pack_offset(&index, name)? else {
            continue;
        };
        let pack: File = File::open(path.with_extension("pack")).map_err(|error| { return format!("a pack cannot be read: {error}"); })?;
        let first: Vec<u8> = read_at(&pack, offset, 1).map_err(|error| { return format!("a pack cannot be read: {error}"); })?;
        return Ok((first[0] >> 4) & 0x7 == PACK_TYPE_COMMIT);
    }
    return Ok(false);
}

/// What:
/// `fn refs_replace_in_use(common_dir: &Path) -> Result<bool, String>` says whether any `refs/replace/` reference exists, loose or
/// packed.
/// Why:
/// A replace reference changes what Git reads for an object name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceRefsInUse(commonDir: string): boolean
/// ```
fn replace_refs_in_use(common_dir: &Path) -> Result<bool, String> {
    let loose: PathBuf = common_dir.join("refs/replace");
    match fs::read_dir(&loose) {
        Ok(mut entries) => {
            if entries.next().is_some() {
                return Ok(true);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("{} cannot be listed: {error}", loose.display())),
    }
    let packed: PathBuf = common_dir.join("packed-refs");
    return match fs::read_to_string(&packed) {
        Ok(text) => Ok(text.contains(" refs/replace/")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("{} cannot be read: {error}", packed.display())),
    };
}

/// What:
/// `pub fn confirm_commit(common_dir: &Path, object: &str) -> Result<(), String>` succeeds only when the object is seen to be a
/// commit stored whole, in the repository's own object files, with no alternates, grafts or replace references.
/// Why:
/// This is the part of `X^{commit}` that a reference file cannot supply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function confirmCommit(commonDir: string, object: string): void  // throws when the object is not seen to be a commit
/// ```
pub fn confirm_commit(common_dir: &Path, object: &str) -> Result<(), String> {
    let Some(name) = hex_to_bytes(object) else {
        return Err(format!("{object:?} is not an object name of a SHA-1 repository"));
    };
    for unsupported in ["objects/info/alternates", "info/grafts"] {
        if common_dir.join(unsupported).exists() {
            return Err(format!("the repository has {unsupported}, which only Git applies"));
        }
    }
    if replace_refs_in_use(common_dir)? {
        return Err("the repository has replace references, which only Git applies".to_string());
    }
    let objects: PathBuf = common_dir.join("objects");
    let loose: PathBuf = objects.join(&object[..2]).join(&object[2..]);
    if loose.exists() {
        let kind: String = loose_type(&loose)?;
        if kind == "commit" {
            return Ok(());
        }
        return Err(format!("{object} is a {kind}, not a commit"));
    }
    if packed_commit(&objects, &name)? {
        return Ok(());
    }
    return Err(format!("{object} was not seen as a commit in the object files"));
}
