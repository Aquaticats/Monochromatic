//! Search results for the Search page:
//!  the five page states and the ranked list of folder and track matches.
//!
//! Ported from `core/SearchResults.kt`.
//!  Folder names and track titles are the only matched fields (D60),
//!  and
//! folders and tracks share one relevance order (D61).
//!  The result cap applies after ordering,
//!  so the strongest
//! matches always survive it.
//!
//! Highlight offsets come from `search_matcher`:
//!  character offsets into the name or the parent context.
//! The ranking itself lives in `search_index_rank` and `search_index_query` (D112).
//! Ordering keys match Kotlin:
//!  tier,
//!  then the normalized name length in UTF-16 units (as Kotlin's `String.length`
//! counts them),
//!  then the collation key that stands in for the rail collator,
//!  then the display path by code point,
//! then the kind,
//!  then the library position.

/// The index that answers a query and the batch form it is built from.
use crate::search_index::SearchIndex;
use crate::search_index_batch::prepare_batch;
/// The highlight type used by each result.
use crate::search_matcher::SearchHighlight;
/// One track with its URI and display path.
use crate::track::Track;

/// What:
///      `pub const SEARCH_RESULT_LIMIT: usize = 500`.
///  The most results one search returns.
/// Why:
///       A huge library must not build an unbounded list,
///  and the state reports when the cap cut it short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_RESULT_LIMIT = 500;
/// ```
pub const SEARCH_RESULT_LIMIT: usize = 500;

/// What:
///      `pub enum SearchResultKind { Folder, Track }`.
///  The two kinds of entry a result can describe.
/// Why:
///       A row opens a folder (D72) or plays a track (D73),
///  so the kind decides the action.
///  Declaration order
///  makes `Folder` sort before `Track`,
///  which the relevance order relies on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchResultKind = "folder" | "track";
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SearchResultKind {
    /// A top-level folder whose own name matched the query.
    Folder,
    /// A track whose own title,
    ///  from its file name without the extension,
    ///  matched the query.
    Track,
}

/// What:
///      `pub struct SearchResult { ... }`.
///  One matched folder or track,
///  ready to draw.
/// Why:
///       A row needs the title,
///  the context line,
///  the highlights for both,
///  and the action target in one value.
///  `folder_name` is always set,
///  because a folder result names itself and a track names its top-level folder,
///  so the Kotlin nullable type is not needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchResult = { kind: SearchResultKind; name: string; parentContext: string | null; /* ... */ };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    /// Whether this result is a folder or a track.
    pub kind: SearchResultKind,
    /// Folder name,
    ///  or track title without its extension.
    pub name: String,
    /// Folder path that holds the track as displayed.
    ///  `None` for a folder and for a root-level track.
    pub parent_context: Option<String>,
    /// Position of the track in the library list,
    ///  or `None` for a folder.
    pub index_in_library: Option<usize>,
    /// Top-level folder that owns this result,
    ///  which is the folder itself for a folder result.
    pub folder_name: String,
    /// Relevance tier,
    ///  where 0 is the best match and a larger value is a weaker match.
    pub tier: i32,
    /// Ranges of the name to emphasize,
    ///  in the name's own character offsets.
    pub name_highlights: Vec<SearchHighlight>,
    /// Ranges of the parent context to emphasize,
    ///  in its own character offsets,
    ///  and empty for a folder.
    pub parent_highlights: Vec<SearchHighlight>,
}

/// What:
///      `pub enum SearchState`.
///  The five states the Search page can be in.
/// Why:
///       The page must tell an empty prompt,
///  an unavailable library,
///  a finished no-match,
///  and results apart (D69).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchState =
///   | { kind: "emptyQuery" }
///   | { kind: "unavailable" }
///   | { kind: "emptyInventory" }
///   | { kind: "noMatch"; query: string }
///   | { kind: "results"; items: SearchResult[]; truncated: boolean };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchState {
    /// No usable query was typed,
    ///  so the page shows its invitation (D70).
    EmptyQuery,
    /// The source is usable and confirmed to hold no tracks,
    ///  so no name can be searched.
    EmptyInventory,
    /// The library cannot be searched right now,
    ///  so no result claim is made (D71).
    Unavailable,
    /// The query was evaluated and no folder or track name matched it.
    ///  Carries the query,
    ///  trimmed (D70).
    NoMatch(
        /// The query text,
        ///  trimmed.
        String,
    ),
    /// Ranked matches,
    ///  with a flag that says whether the limit cut the list short.
    Results {
        /// Matched folders and tracks in relevance order,
        ///  at most `SEARCH_RESULT_LIMIT` of them.
        items: Vec<SearchResult>,
        /// Whether more matches existed than `items` holds.
        truncated: bool,
    },
}

/// What:
///      `pub fn search_library(query: &str, tracks: &[Track], available: bool) -> SearchState`.
///  Answers one Search
///  query against a list of tracks.
/// Why:
///       The page must know which state applies,
///  and a source failure must win over the query state (D69).
///  It builds a `SearchIndex` from the tracks and asks it,
///  so it suits small lists and tests;
///  the window keeps one index alive and appends to it instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function searchLibrary(query: string, tracks: Track[], available: boolean): SearchState { /* index + search */ }
/// ```
pub fn search_library(query: &str, tracks: &[Track], available: bool) -> SearchState {
    if !available {
        return SearchState::Unavailable;
    }
    let display_paths: Vec<&str> = tracks.iter().map(|track| return track.display_path.as_str()).collect();
    let mut index: SearchIndex = SearchIndex::new(0);
    index.append(prepare_batch(&display_paths));
    return index.search(query);
}

/// The replaced matcher, kept as the oracle for index parity tests.
#[cfg(test)]
#[path = "search_results_oracle.rs"]
pub(crate) mod oracle;

/// Unit tests in `search_results_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "search_results_tests.rs"]
mod tests;

/// Ordering and limit tests in `search_results_order_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "search_results_order_tests.rs"]
mod order_tests;
