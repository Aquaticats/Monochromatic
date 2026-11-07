//! Answering a Search query from the index.
//!
//! What:
//! `SearchIndex::search` turns a query into the same `SearchState` the replaced `search_library` produced:
//! it picks the smallest bucket among the query terms,
//! tiers its members,
//! merges the matching folders,
//! keeps the best
//! five hundred in result order,
//! and only then builds rows with parent context and highlights.
//! Why:
//! D112 requires the answer to be identical to the old matcher's while costing milliseconds at one million
//! tracks.
//! The work per keystroke is therefore the bucket size plus five hundred rows,
//! not the library size.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function search(index: SearchIndex, query: string): SearchState { /* bucket, tier, best 500, rows */ }
//! ```

/// Imports the thread module whose scope splits a large tier across workers.
use std::thread;

/// Imports the label of a track's top-level folder.
use crate::pagination::root_level_folder_label;
/// Imports the index type and the bit,
/// slot and marker constants that tell folders from tracks.
use crate::search_index::{FOLDER_BIT, ROOT_FOLDER_SLOT, SearchIndex, UNFILED};
/// Imports the function that finds the folder part of a name.
use crate::search_index_batch::folder_of;
/// Imports the ranked candidate type,
/// the cheap tier test and the best-first selection.
use crate::search_index_rank::{Ranked, fast_tier, select_best};
/// Imports the single-character shortcut answer.
use crate::search_index_top::single_character_tracks;
/// Imports the matcher helpers that split a query and find its highlights.
use crate::search_matcher::{
    SearchHighlight, WORD_SEPARATORS, highlight_ranges_for_terms, search_terms, trim_kotlin_whitespace,
};
/// Imports the result types that a search returns.
use crate::search_results::{SEARCH_RESULT_LIMIT, SearchResult, SearchResultKind, SearchState};
/// Imports the title that a result shows.
use crate::track_entries::track_title_for;

/// What:
///      `pub(crate) const PARALLEL_SCAN_FROM: usize = 32_768;`.
///  The bucket size from which a query is tiered by several threads.
/// Why:
///       Below it the scan takes under a millisecond and a thread would cost about as much;
/// above it the scan is the
///  largest part of a keystroke.
pub(crate) const PARALLEL_SCAN_FROM: usize = 32_768;

/// What:
///      `const SCAN_THREADS: usize = 4;`.
///  How many slices a large bucket is cut into.
/// Why:
///       Four threads cut the scan of a large bucket to a fraction without taking the whole machine for a keystroke.
const SCAN_THREADS: usize = 4;

/// What:
///      `fn parent_context_for(display_path: &str) -> Option<String>`.
///  The folder path that holds a track,
/// or `None` for a root-level track.
/// Why:
///       The second line of a track result names its folder;
/// a root-level track has no parent.
fn parent_context_for(display_path: &str) -> Option<String> {
    let parent: &str = match display_path.rsplit_once('/') {
        Some((parent_path, _file_name)) => parent_path,
        None => "",
    };
    if parent.is_empty() {
        return None;
    }
    return Some(parent.to_string());
}

/// What:
///      `fn bucket_chunk(term: &str) -> Vec<char>`.
///  The first one or two characters of a term,
/// up to its first separator.
/// Why:
///       A match of the term starts at a word start and covers these characters,
/// so they select the bucket.
fn bucket_chunk(term: &str) -> Vec<char> {
    let mut chunk: Vec<char> = Vec::new();
    for character in term.chars() {
        if WORD_SEPARATORS.contains(&character) || chunk.len() == 2 {
            break;
        }
        chunk.push(character);
    }
    return chunk;
}

/// Methods of `SearchIndex` that answer queries.
impl SearchIndex {
    /// What:
    /// `candidate_members` is the smallest bucket among the terms,
    /// or `None` when every term starts with
    /// a separator and no bucket applies.
    /// Why:
    /// every term must match,
    /// so any term's bucket holds every match;
    /// the shortest is the cheapest to test.
    fn candidate_members(&self, terms: &[String]) -> Option<&[u32]> {
        let mut best: Option<&[u32]> = None;
        for term in terms {
            let chunk: Vec<char> = bucket_chunk(term);
            let list: &[u32] = match chunk.as_slice() {
                [] => continue,
                [only] => self.bucket_first(*only),
                [first, second, ..] => self.bucket_pair((*first, *second)),
            };
            let shorter: bool = match best {
                Some(current) => list.len() < current.len(),
                None => true,
            };
            if shorter {
                best = Some(list);
            }
        }
        return best;
    }

    /// What:
    /// `track_matches` ranks the tracks that match the terms and counts them.
    /// Why:
    /// a single-character query reads the cached answer;
    /// any other query tiers its bucket,
    /// or every track when
    /// no bucket applies.
    /// A bucket of `parallel_from` members or more is tiered by several threads,
    /// because that scan
    /// is the largest part of a keystroke and the members are independent;
    /// the answer is the same in either case.
    fn track_matches(&self, terms: &[String], joined: &str, parallel_from: usize) -> (Vec<Ranked>, usize) {
        if let [only] = terms {
            let mut characters = only.chars();
            if let (Some(character), None) = (characters.next(), characters.next())
                && !WORD_SEPARATORS.contains(&character)
            {
                return single_character_tracks(self, character);
            }
        }
        let ranked: Vec<Ranked> = match self.candidate_members(terms) {
            Some(members) if members.len() >= parallel_from => self.tier_in_parallel(members, terms, joined),
            Some(members) => self.tier_members(members, terms, joined),
            None => {
                let mut every: Vec<Ranked> = Vec::new();
                for position in 0..self.track_count() {
                    self.tier_one(position, terms, joined, &mut every);
                }
                every
            }
        };
        let matched: usize = ranked.len();
        return (ranked, matched);
    }

    /// What:
    /// `tier_one` appends the track at `position` to `into` when it matches the terms.
    /// Why:
    /// the one place that turns a match into a ranked entry,
    /// shared by every scan shape.
    fn tier_one(&self, position: usize, terms: &[String], joined: &str, into: &mut Vec<Ranked>) {
        if let Some(tier) = fast_tier(self.norm(position), terms, joined) {
            into.push(Ranked {
                tier,
                nlen: self.nlen(position),
                id: u32::try_from(position).unwrap_or(u32::MAX),
            });
        }
    }

    /// What:
    /// `tier_members` tiers a slice of bucket members on the calling thread.
    /// Why:
    /// the sequential scan,
    /// and the work each thread of the parallel scan does.
    fn tier_members(&self, members: &[u32], terms: &[String], joined: &str) -> Vec<Ranked> {
        let mut ranked: Vec<Ranked> = Vec::new();
        for member in members {
            self.tier_one(*member as usize, terms, joined, &mut ranked);
        }
        return ranked;
    }

    /// What:
    /// `tier_in_parallel` tiers a bucket in `SCAN_THREADS` slices at once and joins them in slice order.
    /// Why:
    /// the joined vector equals the sequential one entry for entry,
    /// so the ranking and the counts are unchanged.
    fn tier_in_parallel(&self, members: &[u32], terms: &[String], joined: &str) -> Vec<Ranked> {
        let slice_len: usize = members.len().div_ceil(SCAN_THREADS);
        tracing::debug!(members = members.len(), slice_len, "tiering a large bucket on several threads");
        #[cfg_attr(
            dylint_lib = "monochromatic_lint_closure",
            expect(
                avoidable_closure,
                reason = "std::thread::scope takes only a callable bounded by the scope, and the scoped spawn takes only a callable that borrows self, the slice and the terms; no named function can carry them and no loop can start the threads"
            )
        )]
        return thread::scope(|scope| {
            let mut workers: Vec<std::thread::ScopedJoinHandle<'_, Vec<Ranked>>> = Vec::new();
            for slice in members.chunks(slice_len) {
                workers.push(scope.spawn(move || return self.tier_members(slice, terms, joined)));
            }
            let mut all: Vec<Ranked> = Vec::new();
            for worker in workers {
                match worker.join() {
                    Ok(part) => all.extend(part),
                    Err(panic) => std::panic::resume_unwind(panic),
                }
            }
            return all;
        });
    }

    /// What:
    /// `folder_matches` ranks the listed folders that match the terms.
    /// Why:
    /// folders are few,
    /// so each query tests all of them,
    /// and a folder never enters the cached track answers.
    fn folder_matches(&self, terms: &[String], joined: &str) -> Vec<Ranked> {
        let mut ranked: Vec<Ranked> = Vec::new();
        for slot in self.listed_folders() {
            let Some(record) = self.folder(slot) else {
                continue;
            };
            if let Some(tier) = fast_tier(&record.norm, terms, joined) {
                ranked.push(Ranked {
                    tier,
                    nlen: record.nlen,
                    id: FOLDER_BIT | u32::try_from(slot).unwrap_or(0),
                });
            }
        }
        if self.root_group_listed()
            && let Some(tier) = fast_tier("unfiled", terms, joined)
        {
            ranked.push(Ranked {
                tier,
                nlen: 7,
                id: FOLDER_BIT | ROOT_FOLDER_SLOT,
            });
        }
        return ranked;
    }

    /// What:
    /// `result_for` builds the row of one ranked candidate.
    /// Why:
    /// titles,
    /// parent context and highlights are computed for the five hundred rows that are shown,
    /// not for
    /// every match.
    fn result_for(&self, ranked: Ranked, terms: &[String]) -> SearchResult {
        let tier: i32 = i32::from(ranked.tier);
        if ranked.id & FOLDER_BIT != 0 {
            let slot: u32 = ranked.id & !FOLDER_BIT;
            let name: String = match self.folder(slot as usize) {
                Some(record) if slot != ROOT_FOLDER_SLOT => record.name.clone(),
                _ => UNFILED.to_string(),
            };
            return SearchResult {
                kind: SearchResultKind::Folder,
                name_highlights: highlight_ranges_for_terms(&name, terms),
                folder_name: name.clone(),
                name,
                parent_context: None,
                index_in_library: None,
                tier,
                parent_highlights: Vec::new(),
            };
        }
        let position: usize = ranked.id as usize;
        let path: &str = self.path(position);
        let title: String = track_title_for(path);
        let parent_context: Option<String> = parent_context_for(path);
        let parent_highlights: Vec<SearchHighlight> = match &parent_context {
            Some(parent_path) => highlight_ranges_for_terms(parent_path, terms),
            None => Vec::new(),
        };
        let (top_folder, segments) = folder_of(path);
        let folder_name: String = match top_folder {
            Some(name) if segments >= 2 => name.to_string(),
            _ => root_level_folder_label(path),
        };
        return SearchResult {
            kind: SearchResultKind::Track,
            name_highlights: highlight_ranges_for_terms(&title, terms),
            name: title,
            parent_context,
            index_in_library: Some(position),
            folder_name,
            tier,
            parent_highlights,
        };
    }

    /// What:
    /// `search` answers one query over every track appended so far.
    /// Why:
    /// it is the whole of Search:
    /// the same states,
    /// order,
    /// cap and highlights as the replaced matcher.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const state = index.search("cam"); // { kind: "results", items, truncated }
    /// ```
    #[must_use]
    pub fn search(&self, query: &str) -> SearchState {
        return self.search_with(query, PARALLEL_SCAN_FROM);
    }

    /// What:
    /// `search_with` answers one query,
    /// tiering buckets of `parallel_from` members or more on several threads.
    /// Why:
    /// tests force the parallel scan on small libraries,
    /// and the sequential scan on large ones,
    /// to prove both
    /// give one answer.
    #[must_use]
    pub(crate) fn search_with(&self, query: &str, parallel_from: usize) -> SearchState {
        let terms: Vec<String> = search_terms(query);
        if terms.is_empty() {
            return SearchState::EmptyQuery;
        }
        if self.track_count() == 0 {
            return SearchState::EmptyInventory;
        }
        let joined: String = terms.join(" ");
        let (mut candidates, track_total) = self.track_matches(&terms, &joined, parallel_from);
        let folders: Vec<Ranked> = self.folder_matches(&terms, &joined);
        let total: usize = track_total + folders.len();
        if total == 0 {
            return SearchState::NoMatch(trim_kotlin_whitespace(query).to_string());
        }
        candidates.extend(folders);
        let best: Vec<Ranked> = select_best(self, candidates, SEARCH_RESULT_LIMIT);
        let mut items: Vec<SearchResult> = Vec::with_capacity(best.len());
        for ranked in best {
            items.push(self.result_for(ranked, &terms));
        }
        return SearchState::Results {
            items,
            truncated: total > SEARCH_RESULT_LIMIT,
        };
    }

    /// What:
    /// `warm` brings the cached answer of one first character up to date.
    /// Why:
    /// a background pass calls it so the first keystroke of a letter reads a cache entry.
    pub fn warm(&self, character: char) {
        if !WORD_SEPARATORS.contains(&character) {
            let _answer = single_character_tracks(self, character);
        }
    }
}
