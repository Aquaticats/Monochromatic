//! Combined project search keeps filename and content results distinct and preserves native path identities.

/// Partial search failures retain the other stream's useful results.
use anyhow::Result;
/// Native paths are not reconstructed from result labels or JSON display text.
use std::path::PathBuf;

/// Match metadata determines navigation without interpreting a presentation string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchKind {
    /// A smart-case substring matched the project-relative filename.
    Path,
    /// A regex matched one source line;
    ///  line numbers use ripgrep's one-based convention.
    Content {
        /// One-based source line to reveal after opening the file.
        line: usize,
        /// Bounded display preview,
        ///  never substituted for actual source on open.
        preview: String,
        /// The original matching line exceeded the preview's grapheme limit.
        truncated: bool,
    },
}

/// One result retains native bytes even when its visible label requires replacement characters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchHit {
    /// Absolute path validated lexically within the project;
    ///  activation rechecks canonical containment.
    pub path: PathBuf,
    /// Path or content match with source navigation metadata.
    pub kind: SearchKind,
}

/// Independent stream outcomes allow filename matches to remain useful when the content regex is invalid.
#[derive(Debug)]
pub struct SearchResults {
    /// Up to twenty smart-case filename matches,
    ///  displayed before content matches.
    pub paths: Result<Vec<SearchHit>>,
    /// Up to thirty matching lines,
    ///  at most one line per file.
    pub contents: Result<Vec<SearchHit>>,
}

/// Match the paused editor's filename-result cap.
pub const MAX_PATH_RESULTS: usize = 20;
/// Match the paused editor's content-result cap.
pub const MAX_CONTENT_RESULTS: usize = 30;
/// Bound JSON/path record buffering before parsing or allocating a result preview.
pub const MAX_SEARCH_RECORD: usize = 4 * 1024 * 1024;
/// Bound UI preview storage without cutting inside a shaped grapheme.
pub const MAX_PREVIEW_GRAPHEMES: usize = 300;
