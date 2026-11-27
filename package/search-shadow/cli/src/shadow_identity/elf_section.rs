//! What:
//! The reader that finds the identity section in an open ELF file
//! and returns the text inside it.
//! Why:
//! The check reads the open file,
//! not a path,
//! so a path swapped after the check cannot change what was checked;
//! a few small reads find the section whatever the file's size.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function identityOfOpenFile(file: FileHandle): string | undefined { /* ELF64 little-endian section table */ }
//! ```
/// Imports the file-system module the identity lives in.
use std::fs;
/// Imports the positioned-read trait for an open file.
use std::os::unix::fs::FileExt;

/// Imports the identity text's section name and prefix.
use super::identity_text::{
    IDENTITY_PREFIX,
    IDENTITY_SECTION,
};

/// The most sections an ELF file may have and still be read for the identity.
/// A release build of each shadow has fewer than one hundred (`readelf --section-headers`,
/// 2026-11-07);
/// this is a bound on memory.
const MAX_SECTIONS: usize = 2048;

/// The most bytes read for the section names and for the identity text.
const MAX_SECTION_BYTES: u64 = 64 * 1024;

/// Reads a little-endian 16-bit number out of a header.
fn u16_at(
    bytes: &[u8],
    offset: usize,
) -> usize {
    return usize::from(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]));
}

/// Reads a little-endian 32-bit number out of a header.
fn u32_at(
    bytes: &[u8],
    offset: usize,
) -> usize {
    return u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]) as usize;
}

/// Reads a little-endian 64-bit number out of a header.
fn u64_at(
    bytes: &[u8],
    offset: usize,
) -> u64 {
    let mut eight: [u8; 8] = [0; 8];
    eight.copy_from_slice(&bytes[offset..offset + 8]);
    return u64::from_le_bytes(eight);
}

/// One section header of an ELF section table:
/// the `index`-th run of `entry_size` bytes.
fn section_entry(
    table: &[u8],
    entry_size: usize,
    index: usize,
) -> &[u8] {
    return &table[index * entry_size..(index + 1) * entry_size];
}

/// Whether a byte ends a section name (a NUL).
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
/// `pub fn identity_of_open_file(file: &fs::File) -> Option<String>` is the crate name and version of the shadow that the open file is,
/// or `None` when it carries no identity text or is not an ELF file this can read.
/// Why:
/// The check reads the open file,
/// not a path,
/// so a path swapped after the check cannot change what was checked.
/// Only a 64-bit little-endian ELF file is read (every shadow is one);
/// anything else is not a shadow.
/// A few small reads find the section,
/// so a candidate costs a fraction of a millisecond whatever its size.
pub fn identity_of_open_file(file: &fs::File) -> Option<String> {
    let mut header: [u8; 64] = [0; 64];
    file.read_exact_at(&mut header, 0).ok()?;
    let is_elf64_little_endian: bool =
        header[..4] == *b"\x7fELF" && header[4] == 2 && header[5] == 1;
    if !is_elf64_little_endian {
        return None;
    }
    let table_offset: u64 = u64_at(&header, 0x28);
    let entry_size: usize = u16_at(&header, 0x3A);
    let entry_count: usize = u16_at(&header, 0x3C);
    let names_index: usize = u16_at(&header, 0x3E);
    if entry_size < 64
        || entry_count == 0
        || entry_count > MAX_SECTIONS
        || names_index >= entry_count
    {
        return None;
    }
    let mut table: Vec<u8> = vec![0; entry_size * entry_count];
    file.read_exact_at(&mut table, table_offset).ok()?;
    let names_entry: &[u8] = section_entry(&table, entry_size, names_index);
    let names_size: u64 = u64_at(names_entry, 0x20);
    if names_size > MAX_SECTION_BYTES {
        return None;
    }
    let mut names: Vec<u8> = vec![0; usize::try_from(names_size).ok()?];
    file.read_exact_at(&mut names, u64_at(names_entry, 0x18))
        .ok()?;
    for index in 0..entry_count {
        let candidate: &[u8] = section_entry(&table, entry_size, index);
        let name_start: usize = u32_at(candidate, 0);
        let name_bytes: &[u8] = names.get(name_start..)?;
        let name_end: usize = name_bytes
            .iter()
            .position(is_nul_byte)
            .unwrap_or(name_bytes.len());
        if &name_bytes[..name_end] != IDENTITY_SECTION.as_bytes() {
            continue;
        }
        let size: u64 = u64_at(candidate, 0x20);
        if size > MAX_SECTION_BYTES {
            return None;
        }
        let mut text: Vec<u8> = vec![0; usize::try_from(size).ok()?];
        file.read_exact_at(&mut text, u64_at(candidate, 0x18))
            .ok()?;
        let whole: String = String::from_utf8(text).ok()?;
        return whole
            .trim_end_matches('\0')
            .strip_prefix(IDENTITY_PREFIX)
            .map(str::to_string);
    }
    return None;
}
