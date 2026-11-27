//! What:
//!  One pack index and one multi-pack-index of an object directory,
//!  read for the count and the sorted names they hold.
//! Why:
//!  The abbreviated name Git prints must be long enough that no other object of the
//!  repository shares it,
//! so the abbreviation asks every pack of the root, and the multi-pack-index that
//!  covers some of them,
//! for their names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const packed: Packed = openPacked(root, nameLength); // throws: a shape this reader does not model
//! ```

#![cfg(unix)]

use crate::object_abbreviation_byte_and_prefix_checks::is_nul;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::{
    Path,
    PathBuf,
};

/// The bytes of a version 2 pack index before the names:
/// magic,
/// version and the fanout table.
const FANOUT_END: u64 = 8 + 256 * 4;

/// The bytes of a version 1 pack index before its entries:
/// the fanout table.
const FANOUT_END_V1: u64 = 256 * 4;

/// The hexadecimal characters two names share from the start;
/// equal names share all.
fn shared_prefix(
    first: &[u8],
    second: &[u8],
) -> usize {
    let mut shared: usize = 0;
    for (left, right) in first.iter().zip(second) {
        if left == right {
            shared += 2;
            continue;
        }
        if left >> 4 == right >> 4 {
            shared += 1;
        }
        break;
    }
    return shared;
}

/// A big-endian `u32` at `position`.
fn be32(
    file: &File,
    position: u64,
) -> Option<u32> {
    let mut bytes: [u8; 4] = [0; 4];
    file.read_exact_at(&mut bytes, position).ok()?;
    return Some(u32::from_be_bytes(bytes));
}

/// A big-endian `u64` at `position`.
fn be64(
    file: &File,
    position: u64,
) -> Option<u64> {
    let mut bytes: [u8; 8] = [0; 8];
    file.read_exact_at(&mut bytes, position).ok()?;
    return Some(u64::from_be_bytes(bytes));
}

/// A file holding object names in sorted order at a fixed stride.
pub(crate) struct SortedNames {
    file: File,
    first: u64,
    stride: u64,
    pub(crate) count: u64,
    length: usize,
}

impl SortedNames {
    /// The `position`th name.
    pub(crate) fn name(
        &self,
        position: u64,
    ) -> Option<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![0; self.length];
        self.file
            .read_exact_at(bytes.as_mut_slice(), self.first + position * self.stride)
            .ok()?;
        return Some(bytes);
    }

    /// What:
    ///  The longest shared prefix between `target` and any other name of the file.
    /// Why:
    ///   The names are sorted,
    /// so the closest ones are the two next to where `target` sits (or sat);
    /// a name equal to
    ///  `target` is the object itself and is skipped.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// longestShared(names, target)
    /// ```
    fn longest_shared(
        &self,
        target: &[u8],
    ) -> Option<usize> {
        let mut low: u64 = 0;
        let mut high: u64 = self.count;
        let mut found: bool = false;
        while low < high {
            let middle: u64 = low + (high - low) / 2;
            match self.name(middle)?.as_slice().cmp(target) {
                std::cmp::Ordering::Equal => {
                    found = true;
                    low = middle;
                    break;
                }
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
            }
        }
        let mut longest: usize = 0;
        if low > 0 {
            longest = longest.max(shared_prefix(self.name(low - 1)?.as_slice(), target));
        }
        let after: u64 = if found { low + 1 } else { low };
        if after < self.count {
            longest = longest.max(shared_prefix(self.name(after)?.as_slice(), target));
        }
        return Some(longest);
    }
}

/// One pack index opened for its count and its names;
/// `None` when it is not a plain index.
pub(crate) fn open_index(
    path: &Path,
    length: usize,
) -> Option<SortedNames> {
    let file: File = File::open(path).ok()?;
    let mut magic: [u8; 8] = [0; 8];
    file.read_exact_at(&mut magic, 0).ok()?;
    if magic == *b"\xfftOc\0\0\0\x02" {
        let count: u64 = u64::from(be32(&file, 8 + 255 * 4)?);
        return Some(SortedNames {
            file,
            first: FANOUT_END,
            stride: length as u64,
            count,
            length,
        });
    }
    // A version 1 index has no header: the fanout comes first, and each entry is a four-byte offset and then the name.
    let count: u64 = u64::from(be32(&file, 255 * 4)?);
    return Some(SortedNames {
        file,
        first: FANOUT_END_V1 + 4,
        stride: 4 + length as u64,
        count,
        length,
    });
}

/// The chunk identifiers of a multi-pack-index that are read here.
const CHUNK_FANOUT: u32 = 0x4f49_4446;
const CHUNK_NAMES: u32 = 0x4f49_444c;
const CHUNK_PACKS: u32 = 0x504e_414d;

/// The covering multi-pack-index of one object directory:
/// its names and the pack indexes it covers.
struct Midx {
    names: SortedNames,
    covered: Vec<String>,
}

/// What:
///  The multi-pack-index file of a pack directory,
/// when there is one this reader models.
/// Why:
///   A pack the index covers is counted and searched through it,
/// not on its own,
/// as Git does.
/// A chain of incremental
///  layers is not modelled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readMidx(packDirectory, length) // undefined: none; null: not modelled
/// ```
fn read_midx(
    pack_directory: &Path,
    length: usize,
) -> Result<Option<Midx>, ()> {
    if pack_directory.join("multi-pack-index.d").exists() {
        return Err(());
    }
    let path: PathBuf = pack_directory.join("multi-pack-index");
    let Ok(file) = File::open(&path) else {
        return Ok(None);
    };
    let mut header: [u8; 12] = [0; 12];
    if file.read_exact_at(&mut header, 0).is_err() {
        return Err(());
    }
    // Signature, version, object format, chunk count, number of base files, number of packs.
    if &header[..4] != b"MIDX" || header[4] != 1 || header[7] != 0 {
        return Err(());
    }
    let chunks: u64 = u64::from(header[6]);
    let packs: u64 = u64::from(u32::from_be_bytes([
        header[8], header[9], header[10], header[11],
    ]));
    let mut fanout_chunk: Option<u64> = None;
    let mut names_chunk: Option<u64> = None;
    let mut packs_chunk: Option<(u64, u64)> = None;
    let mut previous: Option<(u32, u64)> = None;
    for index in 0..=chunks {
        let at: u64 = 12 + index * 12;
        let id: u32 = be32(&file, at).ok_or(())?;
        let offset: u64 = be64(&file, at + 4).ok_or(())?;
        if let Some((earlier, start)) = previous {
            match earlier {
                CHUNK_FANOUT => fanout_chunk = Some(start),
                CHUNK_NAMES => names_chunk = Some(start),
                CHUNK_PACKS => packs_chunk = Some((start, offset)),
                _ => {}
            }
        }
        previous = Some((id, offset));
    }
    let (fanout_at, names_at, (packs_start, packs_end)) = (
        fanout_chunk.ok_or(())?,
        names_chunk.ok_or(())?,
        packs_chunk.ok_or(())?,
    );
    let count: u64 = u64::from(be32(&file, fanout_at + 255 * 4).ok_or(())?);
    let Ok(listed_length) = usize::try_from(packs_end.saturating_sub(packs_start)) else {
        return Err(());
    };
    let mut listed: Vec<u8> = vec![0; listed_length];
    if file
        .read_exact_at(listed.as_mut_slice(), packs_start)
        .is_err()
    {
        return Err(());
    }
    let Ok(pack_limit) = usize::try_from(packs) else {
        return Err(());
    };
    let mut covered: Vec<String> = Vec::new();
    for name in listed.split(is_nul) {
        if covered.len() >= pack_limit {
            break;
        }
        if !name.is_empty() {
            covered.push(String::from_utf8_lossy(name).into_owned());
        }
    }
    return Ok(Some(Midx {
        names: SortedNames {
            file,
            first: names_at,
            stride: length as u64,
            count,
            length,
        },
        covered,
    }));
}

/// The pack indexes and the multi-pack-index of one object directory.
pub(crate) struct Packed {
    midx: Option<Midx>,
    indexes: Vec<SortedNames>,
}

/// Opens what a root's pack directory holds;
/// `Err` when a shape is not modelled.
pub(crate) fn open_packed(
    root: &Path,
    length: usize,
) -> Result<Packed, ()> {
    let directory: PathBuf = root.join("pack");
    let listing = match std::fs::read_dir(&directory) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Packed {
                midx: None,
                indexes: Vec::new(),
            });
        }
        Err(_) => return Err(()),
    };
    let midx: Option<Midx> = read_midx(directory.as_path(), length)?;
    let mut indexes: Vec<SortedNames> = Vec::new();
    for entry in listing {
        let Ok(found) = entry else {
            return Err(());
        };
        let path: PathBuf = found.path();
        let Some(name) = path
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_string)
        else {
            continue;
        };
        if !name.ends_with(".idx") || midx_covers(&midx, &name) {
            continue;
        }
        indexes.push(open_index(path.as_path(), length).ok_or(())?);
    }
    return Ok(Packed {
        midx,
        indexes,
    });
}

/// Whether the multi-pack-index,
/// when there is one,
/// already lists the pack index `name`.
fn midx_covers(
    midx: &Option<Midx>,
    name: &String,
) -> bool {
    return match midx {
        Some(covering) => covering.covered.contains(name),
        None => false,
    };
}

impl Packed {
    /// What:
    ///  The number of names this root's pack indexes hold,
    ///  the multi-pack-index counted once where one covers the packs.
    /// Why:
    ///   Git sizes the automatic abbreviation from the count of packed objects,
    /// and the count is read here,
    /// where the fields are known.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// packed.countedNames(); // number
    /// ```
    pub(crate) fn counted_names(&self) -> u64 {
        let mut counted: u64 = 0;
        if let Some(midx) = self.midx.as_ref() {
            counted += midx.names.count;
        }
        for index in &self.indexes {
            counted += index.count;
        }
        return counted;
    }

    /// What:
    ///  The longest prefix `target` shares with any other name this root's packs hold,
    ///  the object it names itself excluded.
    /// Why:
    ///   One abbreviation must be longer than every shared prefix in the repository,
    /// so the caller takes the longest over the multi-pack-index and the pack indexes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// packed.longestShared(target); // number | undefined
    /// ```
    pub(crate) fn longest_shared(
        &self,
        target: &[u8],
    ) -> Option<usize> {
        let mut longest: usize = 0;
        if let Some(midx) = self.midx.as_ref() {
            longest = longest.max(midx.names.longest_shared(target)?);
        }
        for index in &self.indexes {
            longest = longest.max(index.longest_shared(target)?);
        }
        return Some(longest);
    }
}
