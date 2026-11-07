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
pub fn is_whole_version(text: &str) -> bool {
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
/// `fn is_numeric_identifier(identifier: &str) -> bool` declares a private function that is true when a
///           pre-release identifier is made only of ASCII digits and is not empty.
/// Why:
/// Semantic Versioning 2.0.0,
/// item 11,
/// compares such identifiers as numbers and every other identifier as text,
/// so the comparison first has to tell the two kinds apart.
/// An empty identifier is not valid semver,
/// and counting it as text keeps it from equalling a number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isNumericIdentifier(identifier: string): boolean { return /^\d+$/.test(identifier); }
/// ```
fn is_numeric_identifier(identifier: &str) -> bool {
    return !identifier.is_empty() && all_ascii_digits(identifier);
}

/// What:
/// `fn compare_numeric_identifiers(left: &str, right: &str) -> Ordering` declares a private function comparing
///           two texts of digits by the number they spell,
/// without turning them into integers.
/// Why:
/// A pre-release number can have more digits than any machine integer holds,
/// and a conversion that fails or wraps would make two different numbers equal.
/// After leading zeros are dropped,
/// the number with more digits is the larger one,
/// and two numbers with as many digits compare digit by digit,
/// which is what comparing the texts does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compareNumericIdentifiers(left: string, right: string): number {
///   const a = left.replace(/^0+/, ''); const b = right.replace(/^0+/, '');
///   if (a.length !== b.length) return a.length < b.length ? -1 : 1;
///   return a < b ? -1 : a > b ? 1 : 0;
/// }
/// ```
fn compare_numeric_identifiers(left: &str, right: &str) -> Ordering {
    // What:     `trim_start_matches('0')` returns the borrowed text without its leading `0` characters.
    // Why:      `007` and `7` spell the same number.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const left_digits = left.replace(/^0+/, '');
    // ```
    let left_digits: &str = left.trim_start_matches('0');
    let right_digits: &str = right.trim_start_matches('0');

    // What:     `left_digits.len().cmp(&right_digits.len())` compares the two byte counts and returns an
    //           `Ordering`. `&` lends the right count without moving it. A digit is one byte, so the byte count is
    //           the digit count.
    // Why:      More digits means a larger number once leading zeros are gone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const by_length = Math.sign(left_digits.length - right_digits.length);
    // ```
    let by_length: Ordering = left_digits.len().cmp(&right_digits.len());
    if by_length != Ordering::Equal {
        return by_length;
    }
    return left_digits.cmp(right_digits);
}

/// What:
/// `fn compare_identifiers(left: &str, right: &str) -> Ordering` declares a private function comparing two
///           single pre-release identifiers by the rules of Semantic Versioning 2.0.0,
/// item 11.
/// Why:
/// Identifiers of digits only compare as numbers,
/// identifiers with letters or hyphens compare as ASCII text,
/// and a numeric identifier is always lower than a non-numeric one.
/// Rust's `str` comparison is by byte,
/// which for ASCII text is the ASCII order the specification asks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compareIdentifiers(left: string, right: string): number { ... }
/// ```
fn compare_identifiers(left: &str, right: &str) -> Ordering {
    // What:     `(is_numeric_identifier(left), is_numeric_identifier(right))` builds a pair of two booleans, and
    //           `match` picks the arm that fits the pair.
    // Why:      Four cases: two numbers, a number then text, text then a number, and two texts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (isNumeric(left) && isNumeric(right)) return compareNumeric(left, right);
    // if (isNumeric(left)) return -1;
    // if (isNumeric(right)) return 1;
    // return left < right ? -1 : left > right ? 1 : 0;
    // ```
    match (is_numeric_identifier(left), is_numeric_identifier(right)) {
        (true, true) => return compare_numeric_identifiers(left, right),
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        (false, false) => return left.cmp(right),
    }
}

/// What:
/// `fn compare_pre_releases(left: &str, right: &str) -> Ordering` declares a private function comparing two
///           pre-release texts (the part after the first hyphen,
/// such as `alpha.10`).
/// Why:
/// Semantic Versioning 2.0.0,
/// item 11:
/// the dot-separated identifiers compare from left to right until one differs,
/// and when every identifier of the shorter set equals the start of the longer one,
/// the larger set is higher.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function comparePreReleases(left: string, right: string): number {
///   const a = left.split('.'); const b = right.split('.');
///   for (let i = 0; i < Math.min(a.length, b.length); i++) {
///     const result = compareIdentifiers(a[i], b[i]);
///     if (result !== 0) return result;
///   }
///   return Math.sign(a.length - b.length);
/// }
/// ```
fn compare_pre_releases(left: &str, right: &str) -> Ordering {
    // What:     `left.split('.').collect()` cuts the text at each dot and gathers the pieces into a `Vec<&str>` (a
    //           growable list of borrowed texts; siblings `&[&str]`, a borrowed view of a list, and `[&str; N]`, a
    //           fixed-size array). The annotation names the list type so `collect` knows what to build.
    // Why:      The loop below walks the identifiers by position, and the lengths decide a tie.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const left_identifiers: string[] = left.split('.');
    // ```
    let left_identifiers: Vec<&str> = left.split('.').collect();
    let right_identifiers: Vec<&str> = right.split('.').collect();

    // What:     `left_identifiers.iter().zip(right_identifiers.iter())` pairs the lists element by element and
    //           stops at the end of the shorter one. `for (a, b) in ...` unpacks each pair. `&` is not needed
    //           because `iter()` already lends each element.
    // Why:      Compare identifiers from left to right until one differs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (let i = 0; i < Math.min(left_identifiers.length, right_identifiers.length); i++) { ... }
    // ```
    for (left_identifier, right_identifier) in left_identifiers.iter().zip(right_identifiers.iter()) {
        let by_identifier: Ordering = compare_identifiers(left_identifier, right_identifier);
        if by_identifier != Ordering::Equal {
            return by_identifier;
        }
    }

    // What:     `left_identifiers.len().cmp(&right_identifiers.len())` compares the two list lengths. `&` lends the
    //           right length without moving it.
    // Why:      Every shared identifier is equal, so the longer set is the higher one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return Math.sign(left_identifiers.length - right_identifiers.length);
    // ```
    return left_identifiers.len().cmp(&right_identifiers.len());
}

/// What:
/// `pub fn compare_versions(left: &str, right: &str) -> Ordering` declares a public function returning
///           `Less`,
/// `Equal` or `Greater`.
/// Why:
/// The prune keeps the newest cached version of a crate that cargo has no install record for,
/// and the resolver and the lock check pick the highest of several releases.
/// The order is the precedence of Semantic Versioning 2.0.0,
/// item 11:
/// the three numbers first,
/// then a release above its own pre-release,
/// then two pre-releases identifier by identifier (digits as numbers,
/// the rest as ASCII text,
/// numbers below text,
/// a shorter set below a longer one with the same start).
/// Build metadata after a plus sign does not take part.
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
    // Why:      Equal cores: no pre-release beats a pre-release; two pre-releases follow the identifier rules.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (leftPre === undefined && rightPre === undefined) return 0;
    // if (leftPre === undefined) return 1;
    // if (rightPre === undefined) return -1;
    // return comparePreReleases(leftPre, rightPre);
    // ```
    match (pre_release(left), pre_release(right)) {
        (None, None) => return Ordering::Equal,
        (None, Some(_)) => return Ordering::Greater,
        (Some(_), None) => return Ordering::Less,
        (Some(left_pre), Some(right_pre)) => return compare_pre_releases(left_pre, right_pre),
    }
}
