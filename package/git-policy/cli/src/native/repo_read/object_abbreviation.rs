//! What:
//!  The abbreviated name Git prints for an object (`git commit`'s `[main 30b2211]`,
//! `git log --oneline`),
//! computed in
//!  process:
//! the length from `core.abbrev` or from the number of packed objects,
//! then lengthened until no other object
//!  shares the name.
//! Why:
//!  A fixed seven characters turns ambiguous as a repository grows (Git itself prints ten in a repository of some hundred
//!  thousand objects),
//! and a person may copy the name.
//! Git's rules are `odb_find_abbrev_len` (`odb.c`) for the length and
//!  `find_abbrev_len_for_pack` (`odb/source-packed.c`) and `odb_source_loose_find_abbrev_len` (`odb/source-loose.c`) for
//!  the uniqueness,
//! all read from the Git 2.56.0 sources.
//! The count is the approximate one:
//! the objects of the pack
//!  indexes (a multi-pack-index counts for the packs it covers),
//! loose objects not counted,
//! alternates included.
//! Reading
//!  the indexes needs no Git process;
//! a shape this reader does not model (an incremental multi-pack-index chain,
//! a
//!  version 1 pack index with a name it cannot read) answers nothing and the caller asks Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const short = abbreviate({ objectDirectory, setting: settingOf(config['core.abbrev']), name }); // undefined: ask Git
//! ```

#![cfg(unix)]

use crate::object_abbreviation_byte_and_prefix_checks::{
    is_hex_digit,
    shared_prefix_length,
};
use crate::object_abbreviation_pack_index::{
    Packed,
    open_packed,
};
use crate::repo_read_odb::alternates_of;
use std::path::{
    Path,
    PathBuf,
};

/// Git's `FALLBACK_DEFAULT_ABBREV`:
/// the shortest automatic length.
pub const FALLBACK_ABBREVIATION: usize = 7;

/// Git's `minimum_abbrev`:
/// the shortest length `core.abbrev` may name.
pub const MINIMUM_ABBREVIATION: i64 = 4;

/// Git's `GIT_MAX_HEXSZ`:
/// what a false word in `core.abbrev` asks for,
/// which means the whole name.
const LONGEST_NAME: usize = 64;

/// How a repository asks for the length of an abbreviation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Setting {
    /// `core.abbrev` is unset,
    /// `auto`,
    /// or unusable:
    /// the length follows the size of the repository.
    Auto,
    /// The shortest length `core.abbrev` asks for;
    /// the name is still lengthened until it is unique.
    Length(usize),
}

/// What:
///  The setting the value of `core.abbrev` stands for.
/// Why:
///   Git reads `auto` as the automatic length,
/// a false word (or an empty value) as the whole name and a number from four up
///  as that many characters;
/// anything else makes Git report the key and keep the automatic length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settingOf(value) // Auto | { length }
/// ```
pub fn setting_of(value: Option<&[u8]>) -> Setting {
    let Some(bytes) = value else {
        return Setting::Auto;
    };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Setting::Auto;
    };
    let lowered: String = text.trim().to_ascii_lowercase();
    if lowered == "auto" {
        return Setting::Auto;
    }
    if lowered.is_empty() || ["no", "false", "off"].contains(&lowered.as_str()) {
        return Setting::Length(LONGEST_NAME);
    }
    return match lowered.parse::<i64>() {
        Ok(number) if number >= MINIMUM_ABBREVIATION => {
            Setting::Length(usize::try_from(number).unwrap_or(LONGEST_NAME))
        }
        _ => Setting::Auto,
    };
}

/// The most significant bit of a count,
/// as Git's `msb`:
/// the index of the highest set bit.
fn highest_bit(count: u64) -> usize {
    return if count == 0 {
        0
    } else {
        63 - count.leading_zeros() as usize
    };
}

/// What:
///  The length Git starts from for a repository of `count` packed objects.
/// Why:
///   About `2^bits` objects expect a collision at `2^(bits/2)`;
/// four bits make a character,
/// and small repositories keep the
///  fallback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.max(7, Math.ceil((msb(count) + 1) / 2))
/// ```
pub fn automatic_length(count: u64) -> usize {
    return (highest_bit(count) + 1)
        .div_ceil(2)
        .max(FALLBACK_ABBREVIATION);
}

/// The longest prefix `name` shares with a loose object of the root (the object itself excluded).
fn loose_shared(
    root: &Path,
    hex: &str,
) -> Option<usize> {
    let (directory, rest): (&str, &str) = hex.split_at(2);
    let listing = match std::fs::read_dir(root.join(directory)) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(0),
        Err(_) => return None,
    };
    let mut longest: usize = 0;
    for entry in listing {
        let file_name = entry.ok()?.file_name();
        let Some(other) = file_name.to_str() else {
            continue;
        };
        if other.len() != rest.len() || !other.bytes().all(is_hex_digit) || other == rest {
            continue;
        }
        let shared: usize = shared_prefix_length(rest, other);
        longest = longest.max(2 + shared);
    }
    return Some(longest);
}

/// The bytes of a lower-case hexadecimal name.
fn bytes_of(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 || !hex.bytes().all(is_hex_digit) {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::new();
    for at in 0..hex.len() / 2 {
        bytes.push(u8::from_str_radix(&hex[at * 2..at * 2 + 2], 16).ok()?);
    }
    return Some(bytes);
}

/// What:
///  The abbreviation of `hex` that Git prints:
/// at least the length the setting asks for,
/// and longer until no other object
///  of the repository or its alternates starts with the same characters.
/// Why:
///   The answer is Git's own without starting Git.
/// Nothing is answered when the object directory,
/// an alternate or a pack
///  index has a shape this reader does not model,
/// so the caller can ask Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// abbreviate({ objectDirectory, setting, name }) // 'main' commit 30b2211...
/// ```
pub fn abbreviate(
    object_directory: &Path,
    setting: Setting,
    hex: &str,
) -> Option<String> {
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
            let mut counted: u64 = 0;
            for (_, packed) in &opened {
                counted += packed.counted_names();
            }
            automatic_length(counted)
        }
    };
    if length >= hex.len() || length == 0 {
        return Some(hex.to_string());
    }
    for (root, packed) in &opened {
        let mut longest: usize = loose_shared(root.as_path(), hex)?;
        longest = longest.max(packed.longest_shared(target.as_slice())?);
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
