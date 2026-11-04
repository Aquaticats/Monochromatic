//! What:
//!  The abbreviated name Git prints for an object (`git commit`'s `[main 30b2211]`, `git log --oneline`), computed in
//!  process: the length from `core.abbrev` or from the number of packed objects, then lengthened until no other object
//!  shares the name.
//! Why:
//!  A fixed seven characters turns ambiguous as a repository grows (Git itself prints ten in a repository of some hundred
//!  thousand objects), and a person may copy the name. Git's rules are `odb_find_abbrev_len` (`odb.c`) for the length and
//!  `find_abbrev_len_for_pack` (`odb/source-packed.c`) and `odb_source_loose_find_abbrev_len` (`odb/source-loose.c`) for
//!  the uniqueness, all read from the Git 2.56.0 sources. The count is the approximate one: the objects of the pack
//!  indexes (a multi-pack-index counts for the packs it covers), loose objects not counted, alternates included. Reading
//!  the indexes needs no Git process; a shape this reader does not model (an incremental multi-pack-index chain, a
//!  version 1 pack index with a name it cannot read) answers nothing and the caller asks Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const short = abbreviate({ objectDirectory, setting: settingOf(config['core.abbrev']), name }); // undefined: ask Git
//! ```

#![cfg(unix)]

use crate::repo_read_odb::alternates_of;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

/// Git's `FALLBACK_DEFAULT_ABBREV`: the shortest automatic length.
pub const FALLBACK_ABBREVIATION: usize = 7;

/// Git's `minimum_abbrev`: the shortest length `core.abbrev` may name.
pub const MINIMUM_ABBREVIATION: i64 = 4;

/// Git's `GIT_MAX_HEXSZ`: what a false word in `core.abbrev` asks for, which means the whole name.
const LONGEST_NAME: usize = 64;

/// The bytes of a version 2 pack index before the names: magic, version and the fanout table.
const FANOUT_END: u64 = 8 + 256 * 4;

/// The bytes of a version 1 pack index before its entries: the fanout table.
const FANOUT_END_V1: u64 = 256 * 4;

/// How a repository asks for the length of an abbreviation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Setting {
    /// `core.abbrev` is unset, `auto`, or unusable: the length follows the size of the repository.
    Auto,
    /// The shortest length `core.abbrev` asks for; the name is still lengthened until it is unique.
    Length(usize),
}

/// What:
///  The setting the value of `core.abbrev` stands for.
/// Why:
///   Git reads `auto` as the automatic length, a false word (or an empty value) as the whole name and a number from four up
///  as that many characters; anything else makes Git report the key and keep the automatic length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settingOf(value) // Auto | { length }
/// ```
pub fn setting_of(value: Option<&[u8]>) -> Setting {
    let Some(bytes) = value else { return Setting::Auto };
    let Ok(text) = std::str::from_utf8(bytes) else { return Setting::Auto };
    let lowered: String = text.trim().to_ascii_lowercase();
    if lowered == "auto" {
        return Setting::Auto;
    }
    if lowered.is_empty() || ["no", "false", "off"].contains(&lowered.as_str()) {
        return Setting::Length(LONGEST_NAME);
    }
    return match lowered.parse::<i64>() {
        Ok(number) if number >= MINIMUM_ABBREVIATION => Setting::Length(usize::try_from(number).unwrap_or(LONGEST_NAME)),
        _ => Setting::Auto,
    };
}

/// The most significant bit of a count, as Git's `msb`: the index of the highest set bit.
fn highest_bit(count: u64) -> usize {
    return if count == 0 { 0 } else { 63 - count.leading_zeros() as usize };
}

/// What:
///  The length Git starts from for a repository of `count` packed objects.
/// Why:
///   About `2^bits` objects expect a collision at `2^(bits/2)`; four bits make a character, and small repositories keep the
///  fallback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.max(7, Math.ceil((msb(count) + 1) / 2))
/// ```
pub fn automatic_length(count: u64) -> usize {
    return (highest_bit(count) + 1).div_ceil(2).max(FALLBACK_ABBREVIATION);
}

/// The hexadecimal characters two names share from the start; equal names share all.
fn shared_prefix(first: &[u8], second: &[u8]) -> usize {
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
fn be32(file: &File, position: u64) -> Option<u32> {
    let mut bytes: [u8; 4] = [0; 4];
    file.read_exact_at(&mut bytes, position).ok()?;
    return Some(u32::from_be_bytes(bytes));
}

/// A big-endian `u64` at `position`.
fn be64(file: &File, position: u64) -> Option<u64> {
    let mut bytes: [u8; 8] = [0; 8];
    file.read_exact_at(&mut bytes, position).ok()?;
    return Some(u64::from_be_bytes(bytes));
}

/// A file holding object names in sorted order at a fixed stride.
struct SortedNames {
    file: File,
    first: u64,
    stride: u64,
    count: u64,
    length: usize,
}

impl SortedNames {
    /// The `position`th name.
    fn name(&self, position: u64) -> Option<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![0; self.length];
        self.file.read_exact_at(bytes.as_mut_slice(), self.first + position * self.stride).ok()?;
        return Some(bytes);
    }

    /// What:
    ///  The longest shared prefix between `target` and any other name of the file.
    /// Why:
    ///   The names are sorted, so the closest ones are the two next to where `target` sits (or sat); a name equal to
    ///  `target` is the object itself and is skipped.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// longestShared(names, target)
    /// ```
    fn longest_shared(&self, target: &[u8]) -> Option<usize> {
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

/// One pack index opened for its count and its names; `None` when it is not a plain index.
fn open_index(path: &Path, length: usize) -> Option<SortedNames> {
    let file: File = File::open(path).ok()?;
    let mut magic: [u8; 8] = [0; 8];
    file.read_exact_at(&mut magic, 0).ok()?;
    if magic == *b"\xfftOc\0\0\0\x02" {
        let count: u64 = u64::from(be32(&file, 8 + 255 * 4)?);
        return Some(SortedNames { file, first: FANOUT_END, stride: length as u64, count, length });
    }
    // A version 1 index has no header: the fanout comes first, and each entry is a four-byte offset and then the name.
    let count: u64 = u64::from(be32(&file, 255 * 4)?);
    return Some(SortedNames { file, first: FANOUT_END_V1 + 4, stride: 4 + length as u64, count, length });
}

/// The chunk identifiers of a multi-pack-index that are read here.
const CHUNK_FANOUT: u32 = 0x4f49_4446;
const CHUNK_NAMES: u32 = 0x4f49_444c;
const CHUNK_PACKS: u32 = 0x504e_414d;

/// The covering multi-pack-index of one object directory: its names and the pack indexes it covers.
struct Midx {
    names: SortedNames,
    covered: Vec<String>,
}

/// What:
///  The multi-pack-index file of a pack directory, when there is one this reader models.
/// Why:
///   A pack the index covers is counted and searched through it, not on its own, as Git does. A chain of incremental
///  layers is not modelled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readMidx(packDirectory, length) // undefined: none; null: not modelled
/// ```
fn read_midx(pack_directory: &Path, length: usize) -> Result<Option<Midx>, ()> {
    if pack_directory.join("multi-pack-index.d").exists() {
        return Err(());
    }
    let path: PathBuf = pack_directory.join("multi-pack-index");
    let Ok(file) = File::open(&path) else { return Ok(None) };
    let mut header: [u8; 12] = [0; 12];
    file.read_exact_at(&mut header, 0).map_err(|_| {})?;
    // Signature, version, object format, chunk count, number of base files, number of packs.
    if &header[..4] != b"MIDX" || header[4] != 1 || header[7] != 0 {
        return Err(());
    }
    let chunks: u64 = u64::from(header[6]);
    let packs: u64 = u64::from(u32::from_be_bytes([header[8], header[9], header[10], header[11]]));
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
    let (fanout_at, names_at, (packs_start, packs_end)) = (fanout_chunk.ok_or(())?, names_chunk.ok_or(())?, packs_chunk.ok_or(())?);
    let count: u64 = u64::from(be32(&file, fanout_at + 255 * 4).ok_or(())?);
    let mut listed: Vec<u8> = vec![0; usize::try_from(packs_end.saturating_sub(packs_start)).map_err(|_| {})?];
    file.read_exact_at(listed.as_mut_slice(), packs_start).map_err(|_| {})?;
    let covered: Vec<String> = listed
        .split(|byte| { return *byte == 0; })
        .filter(|name| { return !name.is_empty(); })
        .map(|name| { return String::from_utf8_lossy(name).into_owned(); })
        .take(usize::try_from(packs).map_err(|_| {})?)
        .collect();
    return Ok(Some(Midx { names: SortedNames { file, first: names_at, stride: length as u64, count, length }, covered }));
}

/// The pack indexes and the multi-pack-index of one object directory.
struct Packed {
    midx: Option<Midx>,
    indexes: Vec<SortedNames>,
}

/// Opens what a root's pack directory holds; `Err` when a shape is not modelled.
fn open_packed(root: &Path, length: usize) -> Result<Packed, ()> {
    let directory: PathBuf = root.join("pack");
    let listing = match std::fs::read_dir(&directory) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Packed { midx: None, indexes: Vec::new() }),
        Err(_) => return Err(()),
    };
    let midx: Option<Midx> = read_midx(directory.as_path(), length)?;
    let mut indexes: Vec<SortedNames> = Vec::new();
    for entry in listing {
        let path: PathBuf = entry.map_err(|_| {})?.path();
        let Some(name) = path.file_name().and_then(|name| { return name.to_str(); }).map(str::to_string) else { continue };
        if !name.ends_with(".idx") || midx.as_ref().is_some_and(|covering| { return covering.covered.contains(&name); }) {
            continue;
        }
        indexes.push(open_index(path.as_path(), length).ok_or(())?);
    }
    return Ok(Packed { midx, indexes });
}

/// The longest prefix `name` shares with a loose object of the root (the object itself excluded).
fn loose_shared(root: &Path, hex: &str) -> Option<usize> {
    let (directory, rest): (&str, &str) = hex.split_at(2);
    let listing = match std::fs::read_dir(root.join(directory)) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(0),
        Err(_) => return None,
    };
    let mut longest: usize = 0;
    for entry in listing {
        let file_name = entry.ok()?.file_name();
        let Some(other) = file_name.to_str() else { continue };
        if other.len() != rest.len() || !other.bytes().all(|byte| { return byte.is_ascii_hexdigit(); }) || other == rest {
            continue;
        }
        let shared: usize = rest.bytes().zip(other.bytes()).take_while(|(left, right)| { return left == right; }).count();
        longest = longest.max(2 + shared);
    }
    return Some(longest);
}

/// The bytes of a lower-case hexadecimal name.
fn bytes_of(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 || !hex.bytes().all(|byte| { return byte.is_ascii_hexdigit(); }) {
        return None;
    }
    return (0..hex.len() / 2).map(|at| { return u8::from_str_radix(&hex[at * 2..at * 2 + 2], 16).ok(); }).collect();
}

/// What:
///  The abbreviation of `hex` that Git prints: at least the length the setting asks for, and longer until no other object
///  of the repository or its alternates starts with the same characters.
/// Why:
///   The answer is Git's own without starting Git. Nothing is answered when the object directory, an alternate or a pack
///  index has a shape this reader does not model, so the caller can ask Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// abbreviate({ objectDirectory, setting, name }) // 'main' commit 30b2211...
/// ```
pub fn abbreviate(object_directory: &Path, setting: Setting, hex: &str) -> Option<String> {
    let target: Vec<u8> = bytes_of(hex)?;
    if hex != hex.to_ascii_lowercase() {
        return None;
    }
    let own: PathBuf = std::fs::canonicalize(object_directory).ok()?;
    let mut roots: Vec<PathBuf> = vec![own.clone()];
    alternates_of(own.as_path(), 0, &mut roots)?;
    let name_length: usize = target.len();
    let mut opened: Vec<(PathBuf, Packed)> = Vec::new();
    for root in &roots {
        opened.push((root.clone(), open_packed(root, name_length).ok()?));
    }
    let mut length: usize = match setting {
        Setting::Length(asked) => asked,
        Setting::Auto => {
            let counted: u64 = opened
                .iter()
                .map(|(_, packed)| { return packed.midx.as_ref().map_or(0, |midx| { return midx.names.count; }) + packed.indexes.iter().map(|index| { return index.count; }).sum::<u64>(); })
                .sum();
            automatic_length(counted)
        }
    };
    if length >= hex.len() || length == 0 {
        return Some(hex.to_string());
    }
    for (root, packed) in &opened {
        let mut longest: usize = loose_shared(root.as_path(), hex)?;
        for names in packed.midx.iter().map(|midx| { return &midx.names; }).chain(packed.indexes.iter()) {
            longest = longest.max(names.longest_shared(&target)?);
        }
        if longest >= length {
            length = longest + 1;
        }
    }
    return Some(hex.get(..length.min(hex.len()))?.to_string());
}

/// Abbreviation controls stay out of the release executable.
#[cfg(test)]
#[path = "object_abbreviation_tests.rs"]
mod tests;
