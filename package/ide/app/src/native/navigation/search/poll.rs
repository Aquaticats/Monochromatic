//! Debounce and worker polling share the existing native navigation timer.

/// Presentation is the only point that replaces a result model.
use super::{Search, present};
/// Native handles remain on the UI thread while filesystem and subprocess work stays on the worker.
use crate::native::AppWindow;
/// Worker disconnects and admission failures remain visible.
use anyhow::Result;
/// The reference overlay waits 150ms after its most recent input edit.
use std::time::Duration;

/// Advance a live query without waiting on subprocess work or starting repeated empty requests.
fn tick(window: &AppWindow, search: &mut Search) -> Result<()> {
    if !window.get_search_open() || !window.get_search_busy() {
        return Ok(());
    }
    if let Some(edited) = search.edited {
        if edited.elapsed() < Duration::from_millis(150) {
            return Ok(());
        }
        search.edited = None;
        if let Some(input) = &search.input {
            search
                .worker
                .request_scoped(input.query.clone(), search.scope.clone())?;
            tracing::debug!(query = input.query, scope = %search.scope.display(), "submitted debounced native search");
        }
    }
    let Some(reply) = search.worker.try_take()? else {
        return Ok(());
    };
    let content_only = match search.input.as_ref() {
        Some(input) => input.content_only,
        None => false,
    };
    let (hits, error) = present::results(&reply.results, content_only);
    search.hits = hits;
    window.set_search_error(error.into());
    window.set_search_busy(false);
    present::rows(window, &search.hits, &reply.scope);
    tracing::debug!(
        generation = reply.generation,
        results = search.hits.len(),
        "presented native search results"
    );
    return Ok(());
}

/// A stopped worker reports once;
///  it does not alternate busy/error states on every timer tick.
pub(in crate::native::navigation) fn update(window: &AppWindow, search: &mut Search) {
    if let Err(error) = tick(window, search) {
        tracing::warn!(%error, "native search could not advance");
        search.edited = None;
        window.set_search_busy(false);
        window.set_search_error(format!("{error:#}").into());
    }
}
