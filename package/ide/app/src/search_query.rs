//! Editord-compatible smart-case filename filtering,
//!  independent of content-regex semantics.

/// Matching uses project-relative names;
///  absolute root text must not create a filename hit.
use std::path::Path;

/// Immutable normalized query avoids repeating its lowercase conversion for every candidate.
#[derive(Clone, Debug)]
pub struct PathQuery {
    /// Preserve case whenever lowercasing changes the user's query.
    case_sensitive: bool,
    /// Original or lowercased substring according to the smart-case rule.
    needle: String,
}

/// Path queries are literal substrings,
///  not regular expressions or fuzzy patterns.
impl PathQuery {
    /// Keep the same uppercase-detection rule used by editord's filename search.
    pub fn new(query: &str) -> Self {
        let lowered = query.to_lowercase();
        let case_sensitive = lowered != query;
        // to_string owns the original query only when the lowercase copy cannot be used.
        let needle = if case_sensitive {
            query.to_string()
        } else {
            lowered
        };
        return Self {
            case_sensitive,
            needle,
        };
    }

    /// Test a project-relative native path without changing the path identity stored in the result.
    pub fn matches(&self, relative: &Path) -> bool {
        // Conversion is only a matching/display view; result activation retains the original native bytes.
        let candidate = relative.to_string_lossy();
        if self.case_sensitive {
            return candidate.contains(&self.needle);
        }
        return candidate.to_lowercase().contains(&self.needle);
    }
}
