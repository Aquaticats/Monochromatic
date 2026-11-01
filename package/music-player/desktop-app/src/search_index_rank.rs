//! Tier test and best-five-hundred selection for the Search index.
//!
//! What: `fast_tier` gives the relevance tier of a normalized name without allocating, and `select_best` keeps the
//! best candidates in the result order, computing collation keys only for candidates the integer keys cannot separate.
//! Why: ranking runs on every keystroke over every member of a bucket. Allocating a character list per name, or a
//! collation key per match, was most of the old per-keystroke cost; the integer keys (tier, then UTF-16 length)
//! decide almost every comparison, so the string keys are needed for a few candidates only.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const best = (matches: Ranked[]) => matches.sort(byTierLengthKeyPathKindIndex).slice(0, 501);
//! ```

use std::cmp::Ordering;

use crate::folder_rail::collation_key;
use crate::search_index::{FOLDER_BIT, ROOT_FOLDER_SLOT, SearchIndex, UNFILED};
use crate::search_matcher::WORD_SEPARATORS;
use crate::track_entries::track_title_for;

/// What:
///      `pub(crate) struct Ranked { tier, nlen, id }`.
///  A matched candidate with the two integer ordering keys.
/// Why:
///       Selecting the best candidates by these two integers first leaves the collation key, the display path, the
///  kind and the position to break ties only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Ranked {
    /// Relevance tier: 0 whole name, 1 name prefix, 2 later word.
    pub(crate) tier: u8,
    /// UTF-16 length of the normalized name.
    pub(crate) nlen: u32,
    /// Track position, or `FOLDER_BIT` plus a folder slot.
    pub(crate) id: u32,
}

/// What:
///      `pub(crate) fn fast_tier(norm: &str, terms: &[String], joined: &str) -> Option<u8>`.
///  The relevance tier of a normalized name for already-normalized terms, or `None` when a term does not start a word.
/// Why:
///       Same answer as `match_tier`: every term must occur at a word start; the name equal to the terms joined by
///  a space is tier 0, a name that starts with the first term is tier 1, any other match is tier 2. It searches
///  bytes in place, so it does not allocate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fastTier = (norm: string, terms: string[], joined: string) => allStartAWord(norm, terms) ? tierOf(norm, terms, joined) : null;
/// ```
pub(crate) fn fast_tier(norm: &str, terms: &[String], joined: &str) -> Option<u8> {
    let bytes: &[u8] = norm.as_bytes();
    for term in terms {
        let mut from: usize = 0;
        let mut found: bool = false;
        while let Some(offset) = norm[from..].find(term.as_str()) {
            let start: usize = from + offset;
            let previous_is_separator: bool = start == 0 || WORD_SEPARATORS.contains(&char::from(bytes[start - 1]));
            if previous_is_separator {
                found = true;
                break;
            }
            from = start + norm[start..].chars().next().map_or(1, char::len_utf8);
        }
        if !found {
            return None;
        }
    }
    if norm == joined {
        return Some(0);
    }
    if terms.first().is_some_and(|first| return norm.starts_with(first.as_str())) {
        return Some(1);
    }
    return Some(2);
}

/// What:
///      `fn name_and_sort_path(index: &SearchIndex, id: u32) -> (String, &str)`.
///  The displayed name of a candidate and the path it sorts by.
/// Why:
///       A track is named by the file name without its extension and sorts by its whole display path; a folder
///  is named and sorted by its own name.
fn name_and_sort_path(index: &SearchIndex, id: u32) -> (String, &str) {
    if id & FOLDER_BIT == 0 {
        let path: &str = index.path(id as usize);
        return (track_title_for(path), path);
    }
    let slot: u32 = id & !FOLDER_BIT;
    if slot == ROOT_FOLDER_SLOT {
        return (UNFILED.to_string(), UNFILED);
    }
    return match index.folder(slot as usize) {
        Some(record) => (record.name.clone(), record.name.as_str()),
        None => (String::new(), ""),
    };
}

/// What:
///      `fn ordering_key(name: &str) -> String`.
///  The collation key of a displayed name.
/// Why:
///       Names that are plain ASCII lowercase to themselves under the rail collation, so they skip decomposition.
fn ordering_key(name: &str) -> String {
    if name.is_ascii() {
        return name.to_ascii_lowercase();
    }
    return collation_key(name);
}

/// What:
///      `struct Keyed<'a> { ranked, key, sort_path }`.
///  A candidate with the string keys of the result order.
/// Why:
///       The string keys are built only for candidates that survive the integer keys.
struct Keyed<'a> {
    /// The candidate.
    ranked: Ranked,
    /// Collation key of the displayed name.
    key: String,
    /// Display path of a track, or name of a folder.
    sort_path: &'a str,
}

/// What:
///      `fn keyed<'a>(index: &'a SearchIndex, ranked: Ranked) -> Keyed<'a>`.
///  Adds the string keys to a candidate.
/// Why:
///       Ordering needs them, and building them is deferred until it does.
fn keyed(index: &SearchIndex, ranked: Ranked) -> Keyed<'_> {
    let (name, sort_path) = name_and_sort_path(index, ranked.id);
    return Keyed {
        ranked,
        key: ordering_key(&name),
        sort_path,
    };
}

/// What:
///      `fn full_order(left: &Keyed<'_>, right: &Keyed<'_>) -> Ordering`.
///  The result order: tier, name length, collation key, display path, folder before track, then position.
/// Why:
///       It is the order the replaced matcher used, so results stay identical.
fn full_order(left: &Keyed<'_>, right: &Keyed<'_>) -> Ordering {
    let left_kind: bool = left.ranked.id & FOLDER_BIT == 0;
    let right_kind: bool = right.ranked.id & FOLDER_BIT == 0;
    let left_position: Option<u32> = left_kind.then_some(left.ranked.id);
    let right_position: Option<u32> = right_kind.then_some(right.ranked.id);
    return left
        .ranked
        .tier
        .cmp(&right.ranked.tier)
        .then_with(|| return left.ranked.nlen.cmp(&right.ranked.nlen))
        .then_with(|| return left.key.cmp(&right.key))
        .then_with(|| return left.sort_path.cmp(right.sort_path))
        .then_with(|| return left_kind.cmp(&right_kind))
        .then_with(|| return left_position.cmp(&right_position));
}

/// What:
///      `pub(crate) fn select_best(index: &SearchIndex, matched: Vec<Ranked>, keep: usize) -> Vec<Ranked>`.
///  The best `keep` candidates in result order.
/// Why:
///       Partitioning by the integer keys finds the cut without string keys. Candidates strictly better than the
///  cut are all kept; those equal to the cut are ordered by the string keys to fill the remaining places. Only those
///  and the kept ones get string keys, instead of every match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selectBest(matched: Ranked[], keep: number): Ranked[] { /* nth element by (tier, len), then tie-break */ }
/// ```
pub(crate) fn select_best(index: &SearchIndex, mut matched: Vec<Ranked>, keep: usize) -> Vec<Ranked> {
    if keep == 0 {
        return Vec::new();
    }
    let mut survivors: Vec<Keyed<'_>> = if matched.len() > keep {
        matched.select_nth_unstable_by_key(keep - 1, |candidate| return (candidate.tier, candidate.nlen));
        let cut: (u8, u32) = (matched[keep - 1].tier, matched[keep - 1].nlen);
        let mut better: Vec<Keyed<'_>> = Vec::with_capacity(keep);
        let mut tied: Vec<Keyed<'_>> = Vec::new();
        for candidate in matched {
            let pair: (u8, u32) = (candidate.tier, candidate.nlen);
            if pair < cut {
                better.push(keyed(index, candidate));
            } else if pair == cut {
                tied.push(keyed(index, candidate));
            }
        }
        let places: usize = keep - better.len();
        tied.sort_by(full_order);
        tied.truncate(places);
        better.extend(tied);
        better
    } else {
        matched.into_iter().map(|candidate| return keyed(index, candidate)).collect()
    };
    survivors.sort_by(full_order);
    return survivors.into_iter().map(|entry| return entry.ranked).collect();
}
