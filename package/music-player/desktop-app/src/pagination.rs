//! Pure queue pagination on two axes,
//!  picked per track,
//!  so a long library can be
//! browsed a page at a time.
//!  No Slint,
//!  audio,
//!  or I/O,
//!  so it is fully unit-tested;
//! the binary maps the result onto UI properties at the property edge.
//!
//! Each track's display string is its path relative to the queue's common root
//! (see the `relpath` module):
//!
//! - A track inside a subfolder (its relative path contains `/`) groups by its
//!   TOP-LEVEL folder under the loaded root (one level only):
//!    the page label is
//!   that single folder (e.g. `Artist`),
//!    while any deeper nesting stays visible
//!   in the row's full relative path.
//! - A track sitting directly at the root (no `/`) groups by first letter,
//!    with
//!   fixed buckets:
//!    the 26 English letters A-Z (case-insensitive),
//!    plus a single
//!   `#` catch-all for digits,
//!    symbols,
//!    CJK,
//!    and non-English letters.
//!
//! Pages come out sorted folder-pages-first (case-insensitively by path),
//!  then the
//! A-Z letter pages,
//!  then the `#` catch-all.
//!  Folder labels are case-folded for the
//! sort only (never for display or bucketing),
//!  so `daniwellP` and `r-906` interleave
//! with the capitalized folder names instead of trailing after `Zedd`:
//!  raw codepoint
//! order puts every lowercase letter (a-z,
//!  0x61+) after every uppercase one (A-Z,
//! 0x41-0x5A),
//!  which is the surprising "Zedd before daniwellP" ordering this avoids.

/// What:
///      `use std::collections::BTreeMap;`.
///  `BTreeMap<K, V>` is an ordered map that
///           keeps its keys SORTED (a balanced tree).
///  Sibling the reader might expect:
///           `HashMap<K, V>`,
///  which is faster but iterates in arbitrary order.
/// Why:
///       We group entries by a page key and want the pages to come out in sorted key
///           order for free;
///  `HashMap` would force a separate sort.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no built-in sorted map: a Map you always iterate via [...map.keys()].sort()
/// ```
use std::collections::BTreeMap;

/// What:
///      `const SEPARATOR: char = '/';`.
///  `char` is a single Unicode scalar value
///           (sibling:
///  `&str`,
///  a whole string slice).
///  The path separator the display
///           strings use.
/// Why:
///       `relpath` joins segments with `/`;
///  we split on the same char to find a
///           track's parent folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SEPARATOR = "/";
/// ```
const SEPARATOR: char = '/';

/// What:
///      `const FOLDER_GROUP: u8 = 0;`.
///  `u8` is an 8-bit unsigned integer (siblings:
///           `u16`,
///  `u32`,
///  `usize`).
///  The sort-group tag for folder pages.
/// Why:
///       The page key pairs this tag with a label so folder pages sort before letter
///           pages regardless of how the labels compare as text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FOLDER_GROUP = 0;
/// ```
const FOLDER_GROUP: u8 = 0;

/// What:
///      `const LETTER_GROUP: u8 = 1;`.
///  Sort-group tag for the A-Z letter pages.
/// Why:
///       Letter pages sort after folder pages,
///  before the catch-all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LETTER_GROUP = 1;
/// ```
const LETTER_GROUP: u8 = 1;

/// What:
///      `const CATCH_ALL_GROUP: u8 = 2;`.
///  Sort-group tag for the `#` page.
/// Why:
///       The catch-all sorts last,
///  after every A-Z letter page.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CATCH_ALL_GROUP = 2;
/// ```
const CATCH_ALL_GROUP: u8 = 2;

/// What:
///      `const CATCH_ALL_LABEL: &str = "#";`.
///  `&str` is a BORROWED string slice
///           (here pointing at text baked into the binary);
///  sibling:
///  the owned `String`.
///           The label of the catch-all page.
/// Why:
///       One spot defines the catch-all caption,
///  shared by the key and any test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CATCH_ALL_LABEL = "#";
/// ```
const CATCH_ALL_LABEL: &str = "#";

// What:     `#[derive(Debug, Clone, PartialEq, Eq)]` runs the listed macros to
//           auto-implement behaviour: `Debug` enables `{:?}` formatting (used by
//           `assert_eq!` messages), `Clone` allows duplicating the value, and
//           `PartialEq`/`Eq` enable `==`.
// Why:      Tests compare whole `PageEntry` values with `assert_eq!`, which needs
//           equality and debug formatting; the binary clones names into entries.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation: a plain object is comparable, cloneable, and loggable
// ```
#[derive(Debug, Clone, PartialEq, Eq)]
/// What:
///      `pub struct PageEntry { ... }` declares a public record type:
///  one track on a
///           page,
///  carrying its LOAD-ORDER index plus its display string.
/// Why:
///       Filtering hides other tracks,
///  so a clicked row must still know its real
///           position in the full queue;
///  the index carries that through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PageEntry = { index: number; name: string };
/// ```
pub struct PageEntry {
    /// What:
    ///      `pub index: usize`.
    ///  `usize` is the pointer-sized unsigned integer used
    ///           for array indices (siblings:
    ///  `u32`,
    ///  `u64`,
    ///  `i32`).
    ///  The track's position
    ///           in the full queue,
    ///  in load order.
    /// Why:
    ///       `usize` because it indexes the queue's `Vec`;
    ///  that is the type Rust
    ///           indexing uses,
    ///  so no casts are needed on the queue side.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// index: number;
    /// ```
    pub index: usize,
    /// What:
    ///      `pub name: String`.
    ///  `String` is the OWNED,
    ///  growable UTF-8 buffer
    ///           (sibling:
    ///  the borrowed `&str`).
    ///  The display string:
    ///  a folder-relative
    ///           path (`Artist/Album/01.flac`) for a subfolder track,
    ///  or a bare filename
    ///           for a root-level track.
    /// Why:
    ///       Owned,
    ///  not borrowed,
    ///  because the entry outlives the input slice it was
    ///           copied from (the UI keeps it after `paginate` returns).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// name: string;
    /// ```
    pub name: String,
}

// What:     `#[derive(Debug, Clone, PartialEq, Eq)]`. Same derives as `PageEntry`, for
//           the same reasons (compare and debug-print whole pages in tests).
// Why:      Tests assert on `Vec<Page>` equality.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation: free in TS
// ```
#[derive(Debug, Clone, PartialEq, Eq)]
/// What:
///      `pub struct Page { ... }` declares one page:
///  a label plus the tracks that
///           belong to it,
///  in load order.
/// Why:
///       The UI shows one tab per page (its label) and lists the page's tracks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Page = { label: string; entries: PageEntry[] };
/// ```
pub struct Page {
    /// What:
    ///      `pub label: String`.
    ///  The page caption (owned):
    ///  a relative folder path
    ///           (`Artist/Album`),
    ///  a single A-Z letter,
    ///  or `#`.
    /// Why:
    ///       `String` not `&str` because the label is built fresh (sliced from a path
    ///           or produced by uppercasing),
    ///  not borrowed from the input.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// label: string;
    /// ```
    pub label: String,
    /// What:
    ///      `pub entries: Vec<PageEntry>`.
    ///  `Vec<T>` is the owned,
    ///  growable array
    ///           (sibling:
    ///  the borrowed slice `&[T]`).
    ///  This page's tracks.
    /// Why:
    ///       Owned because the page is built up as names are scanned and handed back
    ///           to the caller.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// entries: PageEntry[];
    /// ```
    pub entries: Vec<PageEntry>,
}

/// Returns a stable page identity that distinguishes folder and root-letter pages.
pub fn page_identity(page: &Page) -> String {
    let folder_prefix = format!("{}/", page.label);
    let is_folder = page
        .entries
        .first()
        .is_some_and(|entry| return entry.name.starts_with(&folder_prefix));
    if is_folder {
        return format!("folder:{}", page.label);
    }
    return format!("root:{}", page.label);
}

/// What:
///      `fn letter_key(name: &str) -> (u8, String)`.
///  The page key for a root-level
///           track (one with no folder):
///  a `(group, label)` pair using the first letter.
///           `(u8, String)` is a TUPLE (a fixed pair of two types).
///  Private helper.
/// Why:
///       Fixed A-Z buckets plus a `#` catch-all,
///  so a flat folder is browsable by
///           first letter without exploding into one page per distinct character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function letterKey(name: string): [number, string] { ... }
/// ```
fn letter_key(name: &str) -> (u8, String) {
    // What:     `match name.chars().next() { ... }`. `name.chars()` iterates the string's
    //           Unicode characters; `.next()` pulls the first as `Option<char>` (`Some(c)`
    //           or `None` for an empty string).
    // Why:      The first character decides the letter bucket.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const c = [...name][0]; // may be undefined
    // ```
    match name.chars().next() {
        // What:     `Some(c) if c.is_ascii_alphabetic() => (LETTER_GROUP, c.to_ascii_uppercase().to_string())`.
        //           A match arm with a GUARD (`if ...`): taken only when the first char is
        //           one of the 26 English letters a-z/A-Z. `c.to_ascii_uppercase()` returns
        //           the uppercase `char` (identity for A-Z); `.to_string()` makes an owned
        //           one-character `String`.
        // Why:      Case-fold so `a` and `A` share the `A` page; non-letters fall through
        //           to the catch-all arm below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (/[a-z]/i.test(c)) return [LETTER_GROUP, c.toUpperCase()];
        // ```
        Some(c) if c.is_ascii_alphabetic() => {
            return (LETTER_GROUP, c.to_ascii_uppercase().to_string())
        }
        // What:     `_ => (CATCH_ALL_GROUP, CATCH_ALL_LABEL.to_string())`. The wildcard `_`
        //           matches everything else: a digit, symbol, CJK or accented/non-English
        //           letter, or an empty name. `.to_string()` copies the `&str` constant
        //           into an owned `String`.
        // Why:      Everything that is not a plain English letter lands on `#`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return [CATCH_ALL_GROUP, CATCH_ALL_LABEL];
        // ```
        _ => return (CATCH_ALL_GROUP, CATCH_ALL_LABEL.to_string()),
    }
}

/// What:
///      `pub fn root_level_folder_label(display_name: &str) -> String`.
///  The name of the virtual folder that a root-level track sits in: its first letter in upper case for an ASCII
///  letter, otherwise `#`.
/// Why:
///       The picker lists loose files in virtual folders named by their rail letter. Search names the folder of a loose
///  track result the same way, so opening it selects the page that shows it (the polish review finding LS1: Search
///  said `Unfiled` while the picker said `A`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rootLevelFolderLabel = (name: string) => /^[a-z]/i.test(name) ? name[0].toUpperCase() : "#";
/// ```
pub fn root_level_folder_label(display_name: &str) -> String {
    return letter_key(display_name).1;
}

/// What:
///      `fn page_key(name: &str) -> (u8, String)`.
///  Decide a track's page:
///  the
///           `(sort-group, label)` pair used both to bucket it and to caption its tab.
///           Private helper.
/// Why:
///       One spot defines grouping,
///  so the bucket key and the displayed label can
///           never drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pageKey(name: string): [number, string] { ... }
/// ```
fn page_key(name: &str) -> (u8, String) {
    // What:     `match name.find(SEPARATOR) { ... }`. `name.find(c)` returns
    //           `Option<usize>`: the BYTE index of the FIRST `/`, or `None` when the name
    //           has no folder. (`find` with a `char` is a plain forward scan, not a regex.)
    // Why:      A `/` means the track lives in a subfolder; the segment before the FIRST
    //           `/` is its top-level folder (one level only), otherwise it is a root-level
    //           (letter-page) track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const slash = name.indexOf("/");
    // ```
    match name.find(SEPARATOR) {
        // What:     `Some(slash) => (FOLDER_GROUP, name[..slash].to_string())`.
        //           `name[..slash]` slices the bytes BEFORE the FIRST `/` (a range index;
        //           `..slash` means "up to, not including"), yielding the top-level folder
        //           as a `&str`; `.to_string()` owns it as the page label.
        // Why:      Group by one folder level only (the top folder under the loaded root);
        //           deeper nesting shows in the row path, not the tab.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (slash >= 0) return [FOLDER_GROUP, name.slice(0, slash)];
        // ```
        Some(slash) => return (FOLDER_GROUP, name[..slash].to_string()),
        // What:     `None => letter_key(name)`. No folder: fall back to the first-letter
        //           bucket.
        // Why:      Root-level tracks paginate by letter.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return letterKey(name);
        // ```
        None => return letter_key(name),
    }
}

/// What:
///      `fn sort_key(label: &str) -> String`.
///  The case-folded form of a page label,
///           used ONLY to order pages,
///  never to display or bucket them.
///           `label.to_uppercase()` is Unicode-aware (folds accented and non-English
///           letters,
///  not just ASCII) and returns a fresh owned `String`.
///  A single linear
///           pass over `label`,
///  no recursion or rescanning.
///  Private helper.
/// Why:
///       Folder labels are raw folder names in mixed case;
///  ordering them as raw
///           `String`s is codepoint order,
///  which sorts every uppercase letter (A-Z,
///           0x41-0x5A) before every lowercase one (a-z,
///  0x61+),
///  so `Zedd` lands before
///           `daniwellP`.
///  Folding case first gives the human "ignore case" order the tab
///           bar wants.
///  Letter pages (`A`-`Z`) and the `#` catch-all are already uppercase,
///           so this is the identity for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sortKey(label: string): string { return label.toUpperCase(); }
/// ```
fn sort_key(label: &str) -> String {
    // What:     `label.to_uppercase()`. Uppercase every character (Unicode-aware). Tail
    //           expression -> return value.
    // Why:      Collapse case so the ordering ignores it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return label.toUpperCase();
    // ```
    return label.to_uppercase()
}

/// What:
///      `pub fn paginate(names: &[String]) -> Vec<Page>`.
///  Group the display strings
///           into pages.
///  `&[String]` is a BORROWED slice of owned strings (read-only;
///  we
///           copy out of it,
///  never mutate it).
/// Why:
///       The binary calls this whenever the queue changes to rebuild the tabs and the
///           visible page.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function paginate(names: readonly string[]): Page[] {
///   const groups = new Map<string, PageEntry[]>(); // keyed by `${group} ${label}`
///   names.forEach((name, index) => {
///     const [group, label] = pageKey(name);
///     const key = `${group} ${label}`;
///     (groups.get(key) ?? groups.set(key, []).get(key)!).push({ index, name });
///   });
///   return [...groups.keys()].sort().map(key => ({ label: key.split(" ")[1], entries: groups.get(key)! }));
/// }
/// ```
pub fn paginate(names: &[String]) -> Vec<Page> {
    // What:     `let mut groups: BTreeMap<(u8, String, String), Vec<PageEntry>> = BTreeMap::new();`.
    //           A fresh empty sorted map keyed by `(sort-group, sort-key, label)` to that
    //           page's entries. `mut` marks it mutable. Tuples sort lexicographically: the
    //           `u8` group first, then the case-folded `sort-key`, then the original
    //           `label` as a tiebreaker. The label still rides in the key so two distinct
    //           folders that case-fold alike (`Reol` and `REOL`) stay separate buckets,
    //           ordered deterministically.
    // Why:      Accumulate entries per page; the tree sorts folder pages, then A-Z, then
    //           `#`, case-insensitively within each, with no extra sort step.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const groups = new Map(); // sorted by (group, sortKey, label)
    // ```
    let mut groups: BTreeMap<(u8, String, String), Vec<PageEntry>> = BTreeMap::new();

    // What:     `for (index, name) in names.iter().enumerate() { ... }`. `names.iter()`
    //           borrows each element as `&String`; `.enumerate()` pairs each with its
    //           position, yielding `(usize, &String)`. The `(index, name)` pattern
    //           destructures that pair.
    // Why:      We need both the load-order index (for `PageEntry.index`) and the name
    //           itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // names.forEach((name, index) => { ... });
    // ```
    for (index, name) in names.iter().enumerate() {
        // What:     `let (group, label) = page_key(name);`. Compute this name's
        //           `(group, label)` pair and destructure it. `name` is a `&String`, which
        //           auto-derefs to the `&str` the helper takes.
        // Why:      Decide which page this name belongs to.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const [group, label] = pageKey(name);
        // ```
        let (group, label) = page_key(name);
        // What:     `let key = (group, sort_key(&label), label);`. Build the
        //           `(group, sort-key, label)` map key: the case-folded label sits between
        //           the group and the original label so ordering ignores case while the
        //           original label still distinguishes buckets. `label` MOVES into the
        //           tuple last (after `sort_key` borrows it).
        // Why:      Case-insensitive page order without losing the display label.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const key = `${group} ${sortKey(label)} ${label}`;
        // ```
        let key = (group, sort_key(&label), label);
        // What:     `let entry = PageEntry { index, name: name.clone() };`. Build the
        //           entry. `index` uses field-init shorthand (variable name matches the
        //           field). `name.clone()` makes an OWNED copy of the borrowed string,
        //           since the entry must own its name.
        // Why:      The slice is only borrowed; the page keeps its own copy.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const entry = { index, name };
        // ```
        let entry = PageEntry {
            index,
            name: name.clone(),
        };
        // What:     `groups.entry(key).or_default().push(entry);`. `.entry(key)` looks up
        //           the key's slot (creating it if absent, MOVING `key` in);
        //           `.or_default()` returns a mutable reference to the slot's `Vec`,
        //           inserting an empty one on first sight; `.push(entry)` appends it.
        // Why:      Bucket the entry under its page key, creating the bucket on demand.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (groups.get(key) ?? groups.set(key, []).get(key)).push(entry);
        // ```
        groups.entry(key).or_default().push(entry);
    }

    // What:     `groups.into_iter().map(|((_, _, label), entries)| Page { label, entries }).collect()`.
    //           `.into_iter()` CONSUMES the map, yielding `((u8, String, String), Vec<PageEntry>)`
    //           pairs in sorted-key order; `.map(|((_, _, label), entries)| ...)`
    //           destructures each, DISCARDING the `u8` group and the case-folded sort-key
    //           (the two `_`s) and keeping the original label, then builds a `Page`
    //           (field-init shorthand); `.collect()` gathers them into the `Vec<Page>` the
    //           return type names. Tail expression -> return value.
    // Why:      Materialize the sorted buckets as the ordered list of pages; the sort group
    //           and sort-key were only needed to order them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [...groups].map(([[, , label], entries]) => ({ label, entries }));
    // ```
    return groups
        .into_iter()
        .map(|((_, _, label), entries)| return Page { label, entries })
        .collect()
}

/// What:
///      `pub fn page_of_index(pages: &[Page], index: usize) -> Option<usize>`.
///  Find
///           which page holds a given load-order track index.
///  `&[Page]` borrows the pages
///           read-only;
///  the result is `Some(page_position)` or `None`.
/// Why:
///       Auto-follow needs to switch the visible page to the one containing the
///           now-playing track when the track changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pageOfIndex(pages: readonly Page[], index: number): number | null {
///   const p = pages.findIndex(page => page.entries.some(e => e.index === index));
///   return p < 0 ? null : p;
/// }
/// ```
pub fn page_of_index(pages: &[Page], index: usize) -> Option<usize> {
    // What:     `pages.iter().position(|page| page.entries.iter().any(|entry| entry.index == index))`.
    //           `.iter()` borrows each page; `.position(|page| ...)` returns the index of
    //           the FIRST page for which the closure is true, as `Option<usize>`. Inside,
    //           `page.entries.iter().any(|entry| ...)` is true when ANY entry on that page
    //           has the matching load-order index. `|page|` and `|entry|` are closures
    //           taking a borrowed page / entry. Tail expression -> return value.
    // Why:      One linear scan locates the page; `position` already yields the `Option`
    //           shape the caller wants.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const p = pages.findIndex((page) => page.entries.some((e) => e.index === index));
    // return p < 0 ? null : p;
    // ```
    return pages
        .iter()
        .position(|page| return page.entries.iter().any(|entry| return entry.index == index))
}

/// What:
///      `pub fn row_display<'a>(label: &str, name: &'a str) -> &'a str`.
///  Given a page's
///           LABEL and one of that page's track display NAMES,
///  return the text a row should
///           SHOW.
///  `<'a>` is a LIFETIME parameter:
///  it ties the returned `&str` to the same
///           `name` that came in,
///  so the result borrows from `name` and lives exactly as long
///           as it.
///  `&str` is a BORROWED string slice (sibling:
///  the owned `String`);
///  we hand
///           back a slice INTO `name`,
///  never a fresh allocation.
///  `label` needs no lifetime
///           because we never return a piece of it.
/// Why:
///       A FOLDER tab already names its top-level folder,
///  so repeating it on every row
///           (`Ado/B/C.opus` under the `Ado` tab) is noise;
///  show `B/C.opus` instead.
///  A LETTER
///           or `#` tab groups loose root-level files that have no folder segment to strip,
///  so
///           their names stay whole.
///  One pure helper keeps both flavours' trimming identical
///           and unit-tested.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rowDisplay(label: string, name: string): string {
///   const prefix = label + "/";
///   return name.startsWith(prefix) ? name.slice(prefix.length) : name;
/// }
/// ```
pub fn row_display<'a>(label: &str, name: &'a str) -> &'a str {
    // What:     `match name.strip_prefix(label) { ... }`. `name.strip_prefix(label)` returns
    //           `Option<&str>`: `Some(rest)` (the text AFTER `label`, a slice borrowed from
    //           `name`) when `name` starts with `label`, otherwise `None`. It is a plain
    //           forward comparison, not a regex.
    // Why:      A folder-page name is exactly `<label>/...`, so a label match is the first
    //           half of detecting that shape; the `/` check below is the second half.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rest = name.startsWith(label) ? name.slice(label.length) : undefined;
    // ```
    match name.strip_prefix(label) {
        // What:     `Some(rest) => rest.strip_prefix('/').unwrap_or(name)`. We matched the
        //           label; `rest` is the remainder. `rest.strip_prefix('/')` returns
        //           `Some(after_slash)` only when `rest` begins with a `/`, else `None`;
        //           `.unwrap_or(name)` extracts that slash-stripped slice or, when there was
        //           no leading `/`, FALLS BACK to the whole original `name`.
        // Why:      `Ado/B/C.opus` under label `Ado` becomes `B/C.opus`; a root file that
        //           merely shares the label's leading letters (label `A`, name `Apple.flac`,
        //           no `/` after the `A`) is returned untouched, so letter tabs never trim.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return rest !== undefined && rest.startsWith("/") ? rest.slice(1) : name;
        // ```
        Some(rest) => return rest.strip_prefix('/').unwrap_or(name),
        // What:     `None => name`. `name` did not start with `label` at all (e.g. the `#`
        //           catch-all label against `#tag.flac` once `#` is stripped there is no `/`,
        //           but a name like `élan.flac` under `#` never matched `#` to begin with).
        //           Return the whole `name`.
        // Why:      Nothing to strip when the label is not a prefix.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return name;
        // ```
        None => return name,
    }
}

/// What:
///      `#[cfg(test)] #[path = "pagination_tests.rs"] mod tests;` declares the
///           test-only child module and aims it at the flat sibling file instead of the
///           default `pagination/tests.rs` lookup.
///  `#[cfg(test)]` compiles it only under
///           `cargo test` / `cargo nextest run`.
///  The file stays the `tests` CHILD of
///           pagination,
///  so its `use super::*` still reaches the private module items.
/// Why:
///       Keep `pagination.rs` to production code;
///  the tests live beside it without
///           inflating this file or its max-lines budget (sibling `*_tests.rs` files are
///           exempt from the linter),
///  matching every other module's convention.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // pagination.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "pagination_tests.rs"]
mod tests;
