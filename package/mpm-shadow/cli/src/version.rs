//! What:
//! Splits a registry entry name such as `monochromatic-lint-0.4.0` into its crate name and version,
//! and orders
//!           two versions.
//! Why:
//! The registry stores `<crate>-<version>` with no separator that tells where the crate name ends,
//! and the
//!           prune must know both parts and which version is newer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // version.ts
//! export function splitEntryName(name: string): { crate: string; version: string } | undefined { ... }
//! export function compareVersions(left: string, right: string): number { ... }
//! ```

/// What:
/// `use std::cmp::Ordering;` imports the three-valued comparison result:
/// `Less`,
/// `Equal` or `Greater`.
/// Why:
/// `compare_versions` returns one,
/// the way a TS comparator returns a negative,
/// zero or positive number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // A TS comparator returns -1, 0 or 1 as a plain number.
/// ```
use std::cmp::Ordering;

/// Imports the shared digit check.
use crate::toolchain::util::all_ascii_digits;

/// What:
/// `pub struct EntryName { ... }` declares a record type with two owned fields.
///           - `crate_name: String`.
/// `String` is a heap-allocated growable UTF-8 text this record owns (sibling:
///             `&str`,
/// a borrowed view).
///           - `version: String`,
/// the same.
/// Why:
/// `String` and not `&str` because the record is handed back to callers that keep it after the borrowed
///           entry name they split has gone out of scope.
/// The field is `crate_name` because `crate` is a Rust keyword.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type EntryName = { crateName: string; version: string };
/// // TS strings are garbage-collected, so the owned-versus-borrowed question does not arise.
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryName {
    /// The crate's name,
    /// for example `monochromatic-lint`.
    pub crate_name: String,
    /// The crate's version text,
    /// for example `0.4.0` or `0.5.0-rc.1`.
    pub version: String,
}

/// What:
/// `fn is_allowed_suffix_char(character: char) -> bool` declares a private function over one `char` (a single
///           Unicode character;
/// sibling `u8` is a single byte).
/// Why:
/// A pre-release or build suffix may hold only ASCII letters,
/// digits,
/// dots,
/// plus signs and hyphens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isAllowedSuffixChar(character: string): boolean { return /[0-9A-Za-z.+-]/.test(character); }
/// ```
fn is_allowed_suffix_char(character: char) -> bool {
    return character.is_ascii_alphanumeric() || character == '.' || character == '+' || character == '-';
}

/// What:
/// `fn is_whole_version(text: &str) -> bool` declares a private function taking a borrowed text and
///           returning true or false.
/// Why:
/// This is the test that decides where the crate name ends:
/// the text after a hyphen must be a whole semantic
///           version (three numeric parts,
/// then an optional pre-release or build suffix),
/// or that hyphen belongs to
///           the crate name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isWholeVersion(text: string): boolean { return /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.+-]*)?$/.test(text); }
/// ```
fn is_whole_version(text: &str) -> bool {
    // What:     `text.find(['-', '+'])` returns an `Option<usize>`: `Some(index)` of the first hyphen or plus sign, or
    //           `None`. `usize` is the unsigned integer wide enough to index any byte (siblings: `u32`, `u64`).
    //           `unwrap_or(text.len())` takes the index, or the text length when there is no suffix.
    // Why:      Everything before that index is the numeric core, everything after it is the suffix.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const coreEnd = text.search(/[-+]/) === -1 ? text.length : text.search(/[-+]/);
    // ```
    let core_end: usize = text.find(['-', '+']).unwrap_or(text.len());

    // What:     `&text[..core_end]` borrows the part of the text before `core_end`. `&` lends it without copying.
    // Why:      The core is checked on its own.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const core = text.slice(0, coreEnd);
    // ```
    let core: &str = &text[..core_end];

    // What:     `core.split('.').collect()` cuts the core at each dot and gathers the pieces into a `Vec<&str>` (a
    //           growable list of borrowed texts; sibling `[&str; 3]` is a fixed-size array). The annotation names the
    //           list type so `collect` knows what to build.
    // Why:      A version core has exactly three numeric parts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parts: string[] = core.split('.');
    // ```
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    for part in &parts {
        // `part.is_empty()` is true for ``; `chars().all(...)` asks whether every character satisfies the test; the
        // `|c| c.is_ascii_digit()` is a closure (Rust's `(c) => c.isAsciiDigit()` arrow function).
        if part.is_empty() || !all_ascii_digits(part) {
            return false;
        }
    }

    // What:     The suffix is everything after the first hyphen or plus sign: `&text[core_end + 1..]`, which is empty
    //           when there is no suffix. `core_end + 1` skips the separator; `..` with nothing after it means "to the
    //           end".
    // Why:      Every suffix character must be in the allowed set.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const suffix = coreEnd < text.length ? text.slice(coreEnd + 1) : '';
    // ```
    if core_end < text.len() {
        let suffix: &str = &text[core_end + 1..];
        return suffix.chars().all(is_allowed_suffix_char);
    }
    return true;
}

/// What:
/// `pub fn split_entry_name(name: &str) -> Option<EntryName>` declares a public function that takes a
///           borrowed entry name and returns an `Option<EntryName>`:
/// `Some(record)` when it splits,
/// `None` when it
///           does not (Rust's replacement for returning `undefined`).
/// Why:
/// It splits at the first hyphen whose remainder is a whole version,
/// so a pre-release hyphen
///           (`0.5.0-rc.1`) stays in the version and a longer crate name (`monochromatic-lint-closure`) stays whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function splitEntryName(name: string): EntryName | undefined { ... }
/// ```
pub fn split_entry_name(name: &str) -> Option<EntryName> {
    // What:     `name.char_indices()` yields each character with its byte index; `(index, character)` unpacks the pair
    //           in the `for`. `index` is a `usize` byte offset.
    // Why:      A hyphen is one byte, so `index + 1` is the first byte after it and slicing there is always valid.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (let index = 0; index < name.length; index++) { const character = name[index]; ... }
    // ```
    for (index, character) in name.char_indices() {
        if character != '-' {
            continue;
        }
        let rest: &str = &name[index + 1..];
        if is_whole_version(rest) {
            // What:     `Some(EntryName { ... })` builds the present case of `Option` wrapping a new record.
            //           `name[..index].to_string()` copies the borrowed slice into an owned `String`; `.to_string()`
            //           allocates a new text with the same characters.
            // Why:      Hand the caller the crate name and the version as owned texts. `String` and not `&str` so the
            //           result outlives the borrowed input.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { crateName: name.slice(0, index), version: rest };
            // ```
            return Some(EntryName {
                crate_name: name[..index].to_string(),
                version: rest.to_string(),
            });
        }
    }
    // What:     `None` is the absent case of `Option`.
    // Why:      No hyphen was followed by a whole version, so this is not a `<crate>-<version>` name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return undefined;
    // ```
    return None;
}

/// What:
/// `fn numeric_core(version: &str) -> [u64; 3]` declares a private function returning a fixed-size array of
///           three `u64`s (unsigned 64-bit integers;
/// siblings `u32`,
/// `usize`).
/// Why:
/// Versions order by their numeric parts as numbers,
/// not text (`0.1.10` is newer than `0.1.9`);
/// `u64`
///           because a version part can be large and never negative,
/// and a fixed array because a core has exactly three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function numericCore(version: string): [number, number, number] { ... }
/// ```
fn numeric_core(version: &str) -> [u64; 3] {
    // `find(['-', '+']).unwrap_or(len)` finds where the numeric core ends, as in `is_whole_version`.
    let core_end: usize = version.find(['-', '+']).unwrap_or(version.len());
    let mut parts: [u64; 3] = [0, 0, 0];

    // What:     `.enumerate()` pairs each piece with its position; `.take(3)` stops after three pieces. `piece
    //           .parse::<u64>()` turns the text into a number and returns a `Result`; `.unwrap_or(0)` takes the number
    //           or `0` when the text is not a number (which `is_whole_version` has already ruled out for real entries).
    // Why:      Fill the array from the version text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // version.slice(0, coreEnd).split('.').slice(0, 3).forEach((piece, i) => { parts[i] = Number(piece) || 0; });
    // ```
    for (position, piece) in version[..core_end].split('.').enumerate().take(3) {
        parts[position] = piece.parse::<u64>().unwrap_or(0);
    }
    return parts;
}

/// What:
/// `fn pre_release(version: &str) -> Option<&str>` declares a private function returning the pre-release
///           text borrowed from the input,
/// or `None`.
/// Why:
/// A release (`0.5.0`) is newer than its own pre-release (`0.5.0-rc.1`),
/// so the order needs to know whether
///           a pre-release is there.
/// `&str` and not `String` because the result is a view into the input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function preRelease(version: string): string | undefined { ... }
/// ```
fn pre_release(version: &str) -> Option<&str> {
    // What:     `version.split_once('-')` cuts at the first hyphen and returns `Some((before, after))` or `None`.
    //           `(_, after)` unpacks the pair and ignores the first half with `_`. Build metadata after a plus sign
    //           is dropped with a second `split('+').next()`.
    // Why:      Only the text after the first hyphen and before any plus sign orders pre-releases.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const dash = version.indexOf('-'); if (dash === -1) return undefined; return version.slice(dash + 1).split('+')[0];
    // ```
    if let Some((_, after)) = version.split_once('-') {
        return after.split('+').next();
    }
    return None;
}

/// What:
/// `pub fn compare_versions(left: &str, right: &str) -> Ordering` declares a public function returning
///           `Less`,
/// `Equal` or `Greater`.
/// Why:
/// The prune keeps the newest cached version of a crate that cargo has no install record for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function compareVersions(left: string, right: string): number { ... }
/// ```
pub fn compare_versions(left: &str, right: &str) -> Ordering {
    // What:     `numeric_core(left).cmp(&numeric_core(right))` compares two arrays element by element, first to last,
    //           and returns an `Ordering`. `&` lends the right array without moving it.
    // Why:      The numeric parts decide first.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const byCore = compareArrays(numericCore(left), numericCore(right));
    // ```
    let by_core: Ordering = numeric_core(left).cmp(&numeric_core(right));
    if by_core != Ordering::Equal {
        return by_core;
    }

    // What:     `match (a, b) { ... }` looks at two `Option`s together and picks the arm whose shape fits; `(None,
    //           None)` means neither side has a pre-release. `Some(x)` binds the inner text.
    // Why:      Equal cores: no pre-release beats a pre-release; two pre-releases compare as text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (leftPre === undefined && rightPre === undefined) return 0;
    // if (leftPre === undefined) return 1;
    // if (rightPre === undefined) return -1;
    // return leftPre < rightPre ? -1 : leftPre > rightPre ? 1 : 0;
    // ```
    match (pre_release(left), pre_release(right)) {
        (None, None) => return Ordering::Equal,
        (None, Some(_)) => return Ordering::Greater,
        (Some(_), None) => return Ordering::Less,
        (Some(left_pre), Some(right_pre)) => return left_pre.cmp(right_pre),
    }
}
