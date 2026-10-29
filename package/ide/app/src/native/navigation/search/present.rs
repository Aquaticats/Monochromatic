//! Search result labels are projections,
//!  never filesystem identities.

/// The window model owns copied labels while native search retains the original paths.
use crate::native::{AppWindow, ui::SearchEntry};
/// Content line numbers and truncation state describe navigation and preview fidelity separately.
use ide_app::search::{SearchHit, SearchKind, SearchResults};
/// Bounded immutable rows become one native model replacement per accepted reply.
use slint::{ModelRc, SharedString, VecModel};
/// Scope-relative display paths keep the searched directory visible without repeating its full prefix.
use std::{path::Path, rc::Rc};

/// Preserve useful results from either stream and report each actual failure.
pub(super) fn results(results: &SearchResults, content_only: bool) -> (Vec<SearchHit>, String) {
    let mut hits = Vec::new();
    let mut messages = Vec::new();
    if !content_only {
        match &results.paths {
            Ok(paths) => hits.extend(paths.iter().cloned()),
            Err(error) => messages.push(format!("Filename search: {error:#}")),
        }
    }
    match &results.contents {
        Ok(contents) => hits.extend(contents.iter().cloned()),
        Err(error) => messages.push(format!("Content search: {error:#}")),
    }
    return (hits, messages.join("\n"));
}

/// What:
///  `&[SearchHit]` borrows a variable-length list instead of owning `Vec<SearchHit>` or requiring `[T; N]`.
/// Why:
///  Rendering labels must not consume the paths later used by the open callback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// window.rows = hits.map(hit => present(hit, scope));
/// ```
pub(super) fn rows(window: &AppWindow, hits: &[SearchHit], scope: &Path) {
    let mut rows = Vec::with_capacity(hits.len());
    for hit in hits {
        let path = hit.path.strip_prefix(scope).unwrap_or(&hit.path);
        let (detail, preview) = match &hit.kind {
            SearchKind::Path => ("File".to_string(), String::new()),
            SearchKind::Content {
                line,
                preview,
                truncated,
            } => (
                format!("Line {line}"),
                if *truncated {
                    format!("{preview}…")
                } else {
                    preview.clone()
                },
            ),
        };
        rows.push(SearchEntry {
            path: SharedString::from(path.to_string_lossy().as_ref()),
            detail: SharedString::from(detail),
            preview: SharedString::from(preview),
        });
    }
    window.set_search_entries(ModelRc::from(Rc::new(VecModel::from(rows))));
    window.set_search_selected(if hits.is_empty() { -1 } else { 0 });
    window.invoke_reset_search_scroll();
}
