//! Batch preparation for the Search index: everything about a group of new tracks that can be computed without
//! touching the index.
//!
//! What: `prepare_batch` normalizes each title, counts its UTF-16 length, collects the first-character and
//! first-two-character keys of its word starts, and notes which top-level folders it names.
//! Why: D112 appends tracks under a write lock that holds only the push, so every cost that grows with the text
//! (normalization, key extraction, sorting keys into runs) happens here, on the caller's thread, before the lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const prepareBatch = (paths: string[]): PreparedBatch => ({ norm: paths.map(normalizeTitle), keys: wordStartKeys(...) });
//! ```

use std::collections::HashMap;

use crate::search_index_text::{TextArena, normalize_title_into};
use crate::search_matcher::WORD_SEPARATORS;
use crate::track_entries::track_title_for;

/// What:
///      `pub(crate) const UNFILED_PREFIX: &str = "Unfiled/"`.
///  The path prefix that marks a root-level track as filed under a folder named `Unfiled`.
/// Why:
///       The folder index treats the root-level group as a real folder only when one of its tracks starts with this
///  prefix, so the batch records whether any track does.
pub(crate) const UNFILED_PREFIX: &str = "Unfiled/";

/// What:
///      `pub(crate) struct FolderSighting { name, nested_or_prefixed }`.
///  A top-level folder named by a batch.
/// Why:
///       The index lists each folder once; the flag decides, for a folder named `Unfiled`, whether it is a real
///  folder or the root-level group's namesake.
#[derive(Debug)]
pub(crate) struct FolderSighting {
    /// First path segment of at least one track with two or more segments.
    pub(crate) name: String,
    /// Whether a track of this folder has a subfolder or a path that starts with `Unfiled/`.
    pub(crate) nested_or_prefixed: bool,
}

/// What:
///      `pub(crate) struct PreparedBatch`.
///  A group of tracks, ready to be appended with a push.
/// Why:
///       Holding the arenas, lengths, sorted key lists and folder notes lets the index append in time proportional
///  to a copy, not to normalization.
#[derive(Debug)]
pub(crate) struct PreparedBatch {
    /// Display paths of the batch tracks, in batch order.
    pub(crate) paths: TextArena,
    /// Trimmed normalized titles, in batch order.
    pub(crate) norm: TextArena,
    /// UTF-16 length of each normalized title.
    pub(crate) nlen: Vec<u32>,
    /// Word-start first characters with the batch-local track number, sorted by character then number.
    pub(crate) by_first: Vec<(char, u32)>,
    /// Word-start first character pairs with the batch-local track number, sorted by pair then number.
    pub(crate) by_pair: Vec<((char, char), u32)>,
    /// Top-level folders named by tracks with two or more segments.
    pub(crate) folders: Vec<FolderSighting>,
    /// Whether any track in the batch has fewer than two segments.
    pub(crate) has_root_track: bool,
    /// Whether a root-level track in the batch starts with `Unfiled/`.
    pub(crate) root_prefixed: bool,
}

/// What:
///      `fn push_word_start_keys(norm: &str, local: u32, firsts: &mut Vec<char>, pairs: &mut Vec<(char, char)>, ...)`.
///  Lists the distinct word-start keys of one normalized title.
/// Why:
///       A query term can match only where a word starts, so a track belongs in the bucket of each first character
///  and first character pair found at a word start. A separator never starts a bucket key.
fn word_start_keys(norm: &str, firsts: &mut Vec<char>, pairs: &mut Vec<(char, char)>) {
    firsts.clear();
    pairs.clear();
    let mut previous: Option<char> = None;
    let mut characters = norm.chars().peekable();
    while let Some(character) = characters.next() {
        let starts_word: bool = previous.is_none_or(|before| return WORD_SEPARATORS.contains(&before));
        if starts_word && !WORD_SEPARATORS.contains(&character) {
            firsts.push(character);
            if let Some(next) = characters.peek().filter(|next| return !WORD_SEPARATORS.contains(next)) {
                pairs.push((character, *next));
            }
        }
        previous = Some(character);
    }
    firsts.sort_unstable();
    firsts.dedup();
    pairs.sort_unstable();
    pairs.dedup();
}

/// What:
///      `pub(crate) fn folder_of(display_path: &str) -> (Option<&str>, usize)`.
///  The top-level folder of a display path and the number of its non-empty segments, counted up to three.
/// Why:
///       The folder index drops empty, `.` and `..` segments before it decides where a track lives, so the batch
///  must drop them too to find the same folder.
pub(crate) fn folder_of(display_path: &str) -> (Option<&str>, usize) {
    let mut first: Option<&str> = None;
    let mut count: usize = 0;
    for segment in display_path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            continue;
        }
        if first.is_none() {
            first = Some(segment);
        }
        count += 1;
        if count == 3 {
            break;
        }
    }
    return (first, count);
}

/// What:
///      `pub(crate) fn prepare_batch<S: AsRef<str>>(display_paths: &[S]) -> PreparedBatch`.
///  Builds the appendable form of a list of display paths.
/// Why:
///       This is the only part of an append that costs time proportional to the text, and it runs without any lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareBatch(displayPaths: string[]): PreparedBatch { /* normalize, key, sort */ }
/// ```
pub(crate) fn prepare_batch<S: AsRef<str>>(display_paths: &[S]) -> PreparedBatch {
    let mut paths: TextArena = TextArena::new();
    let mut norm: TextArena = TextArena::new();
    let mut nlen: Vec<u32> = Vec::with_capacity(display_paths.len());
    let mut by_first: Vec<(char, u32)> = Vec::new();
    let mut by_pair: Vec<((char, char), u32)> = Vec::new();
    let mut folders: HashMap<String, bool> = HashMap::new();
    let mut has_root_track: bool = false;
    let mut root_prefixed: bool = false;
    let mut firsts: Vec<char> = Vec::new();
    let mut pairs: Vec<(char, char)> = Vec::new();
    for (local, display_path) in display_paths.iter().enumerate() {
        let path: &str = display_path.as_ref();
        let local_id: u32 = u32::try_from(local).expect("a batch holds under four billion tracks");
        paths.push(path);
        let length: usize = normalize_title_into(norm.buffer_mut(), &track_title_for(path));
        norm.close_entry();
        nlen.push(u32::try_from(length).expect("a title is under four billion UTF-16 units"));
        word_start_keys(norm.get(local), &mut firsts, &mut pairs);
        by_first.extend(firsts.iter().map(|key| return (*key, local_id)));
        by_pair.extend(pairs.iter().map(|key| return (*key, local_id)));
        let (folder, segments) = folder_of(path);
        match folder {
            Some(name) if segments >= 2 => {
                let marker: bool = segments >= 3 || path.starts_with(UNFILED_PREFIX);
                match folders.get_mut(name) {
                    Some(flag) => *flag |= marker,
                    None => {
                        folders.insert(name.to_string(), marker);
                    }
                }
            }
            _ => {
                has_root_track = true;
                root_prefixed |= path.starts_with(UNFILED_PREFIX);
            }
        }
    }
    by_first.sort_by_key(|entry| return entry.0);
    by_pair.sort_by_key(|entry| return entry.0);
    let mut sightings: Vec<FolderSighting> = folders
        .into_iter()
        .map(|(name, nested_or_prefixed)| return FolderSighting { name, nested_or_prefixed })
        .collect();
    sightings.sort_by(|left, right| return left.name.cmp(&right.name));
    return PreparedBatch {
        paths,
        norm,
        nlen,
        by_first,
        by_pair,
        folders: sightings,
        has_root_track,
        root_prefixed,
    };
}
