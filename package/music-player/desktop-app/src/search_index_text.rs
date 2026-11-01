//! Text storage and normalization for the Search index.
//!
//! What: `TextArena` keeps many strings in one buffer, and `normalize_title_into` writes the trimmed normalized form
//! of a title into such a buffer without allocating per character.
//! Why: one million names stored as one `String` each cost a heap block per name; one buffer with end offsets costs
//! four bytes per name, and a batch of names can be moved into the index with two copies.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class TextArena { buf = ""; ends: number[] = []; get(i: number) { return this.buf.slice(this.start(i), this.ends[i]); } }
//! ```

use unicode_normalization::UnicodeNormalization;

use crate::non_spacing_mark::is_non_spacing_mark;
use crate::search_matcher::{is_kotlin_whitespace, trim_kotlin_whitespace};

/// What:
///      `pub(crate) struct TextArena { buf: String, ends: Vec<u32> }`.
///  Append-only list of strings, stored back to back.
/// Why:
///       The index appends batches and reads single entries by position, which is all this layout supports.
///  Offsets are `u32`, so one arena holds up to four gibibytes of text, far above one million names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TextArena = { buf: string; ends: number[] };
/// ```
#[derive(Debug, Default)]
pub(crate) struct TextArena {
    /// All entries concatenated in insertion order.
    buf: String,
    /// End offset in `buf` of each entry; entry `i` starts where entry `i - 1` ends.
    ends: Vec<u32>,
}

/// Methods of `TextArena`.
impl TextArena {
    /// What: `new` makes an empty arena.
    /// Why: an empty index starts with empty arenas.
    pub(crate) fn new() -> TextArena {
        return TextArena::default();
    }

    /// What: `byte_len` is the total text size in bytes.
    /// Why: memory reports and capacity estimates read it.
    pub(crate) fn byte_len(&self) -> usize {
        return self.buf.len();
    }

    /// What: `get` returns entry `position`, or an empty text when the position is past the end.
    /// Why: a caller that races a reset reads an empty name instead of panicking.
    pub(crate) fn get(&self, position: usize) -> &str {
        let Some(end) = self.ends.get(position) else {
            return "";
        };
        let start: usize = match position.checked_sub(1) {
            Some(previous) => self.ends[previous] as usize,
            None => 0,
        };
        return &self.buf[start..*end as usize];
    }

    /// What: `push` appends one entry.
    /// Why: batch preparation builds an arena entry by entry, outside any lock.
    ///
    /// # Panics
    /// When the arena would pass four gibibytes, which is far above the supported library size.
    pub(crate) fn push(&mut self, text: &str) {
        self.buf.push_str(text);
        self.close_entry();
    }

    /// What: `buffer_mut` lends the buffer so a normalizer can write the next entry in place.
    /// Why: writing into the buffer avoids one temporary `String` per name; the caller then calls `close_entry`.
    pub(crate) fn buffer_mut(&mut self) -> &mut String {
        return &mut self.buf;
    }

    /// What: `close_entry` ends the entry written through `buffer_mut`.
    /// Why: the entry boundary is recorded once, after the text is complete.
    ///
    /// # Panics
    /// When the arena would pass four gibibytes, which is far above the supported library size.
    pub(crate) fn close_entry(&mut self) {
        let end: u32 = u32::try_from(self.buf.len()).expect("a search arena holds under four gibibytes of text");
        self.ends.push(end);
    }

    /// What: `reserve` makes room for more entries and bytes.
    /// Why: reserving before the write lock is taken keeps buffer growth, which copies the whole buffer, out of the
    /// lock.
    pub(crate) fn reserve(&mut self, entries: usize, bytes: usize) {
        self.buf.reserve(bytes);
        self.ends.reserve(entries);
    }

    /// What: `append` moves every entry of `other` to the end of this arena.
    /// Why: this is the push the index performs under its write lock: two copies and an offset shift.
    ///
    /// # Panics
    /// When the arena would pass four gibibytes, which is far above the supported library size.
    pub(crate) fn append(&mut self, other: &TextArena) {
        let total: usize = self.buf.len() + other.buf.len();
        let shift: u32 = u32::try_from(self.buf.len()).expect("a search arena holds under four gibibytes of text");
        u32::try_from(total).expect("a search arena holds under four gibibytes of text");
        self.buf.push_str(&other.buf);
        self.ends.extend(other.ends.iter().map(|end| return end + shift));
    }
}

/// What:
///      `pub(crate) fn normalize_title_into(buffer: &mut String, title: &str) -> usize`.
///  Appends the search form of a title to `buffer` and returns its length in UTF-16 units.
/// Why:
///       The search form is the trimmed result of `normalize_for_search`: NFKD, non-spacing marks removed, one
///  code point lowercased at a time. ASCII code points are their own decomposition and have no mark, so they take a
///  direct lowercase path; every other code point goes through the same per-character rule as the matcher.
///  The UTF-16 length orders results, as Kotlin's `String.length` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const searchForm = (title: string) => normalizeForSearch(title).trim();
/// ```
pub(crate) fn normalize_title_into(buffer: &mut String, title: &str) -> usize {
    let start: usize = buffer.len();
    for character in title.chars() {
        if character.is_ascii() {
            buffer.push(character.to_ascii_lowercase());
            continue;
        }
        let mut single: [u8; 4] = [0; 4];
        let text: &str = character.encode_utf8(&mut single);
        buffer.extend(
            text.nfkd()
                .filter(|part| return !is_non_spacing_mark(*part))
                .flat_map(|part| return part.to_lowercase()),
        );
    }
    let written: &str = &buffer[start..];
    let leading: usize = written.len() - written.trim_start_matches(is_kotlin_whitespace).len();
    let kept: usize = trim_kotlin_whitespace(written).len();
    buffer.truncate(start + leading + kept);
    buffer.replace_range(start..start + leading, "");
    return buffer[start..].chars().map(char::len_utf16).sum();
}
