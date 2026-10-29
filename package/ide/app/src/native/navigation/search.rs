//! Native search owns debounce,
//!  immutable result identity,
//!  and one cancellable worker.

/// Worker startup failures remain explicit rather than silently disabling the shortcut.
use anyhow::Result;
/// Search presentation never needs a second project boundary or a mutable source snapshot.
use ide_app::{
    search::SearchHit,
    search_input::{DoubleShift, SearchInput},
    search_worker::SearchWorker,
    workspace::Workspace,
};
/// Native paths are kept separately from lossy labels;
///  monotonic clocks drive debounce and gestures.
use std::{path::PathBuf, time::Instant};

/// Input and overlay lifecycle callbacks retain the navigation owner.
mod bindings;
/// Polling publishes only the latest query and preserves independent stream diagnostics.
mod poll;
/// Convert immutable hits to native display rows without reconstructing their paths.
mod present;
/// Open,
///  edit,
///  close,
///  and choose transitions own cancellation and source navigation.
mod session;

/// Share the binding entry point without exposing search state beyond native navigation.
pub(super) use bindings::bind;
/// The existing navigation timer also advances search without a second polling loop.
pub(super) use poll::update;

/// Window-local search state contains no cross-thread mutable source data.
pub(super) struct Search {
    /// Only this worker starts search children;
    ///  closing or editing cancels its current generation.
    worker: SearchWorker,
    /// Last tree interaction's directory,
    ///  independent of programmatic file reveal.
    directory: Option<PathBuf>,
    /// Scope captured when opening the overlay,
    ///  stable while background tree refresh continues.
    scope: PathBuf,
    /// Parsed input is absent when no useful pattern exists.
    input: Option<SearchInput>,
    /// Debounce starts at the most recent edit,
    ///  not at worker completion.
    edited: Option<Instant>,
    /// Native hit identities align exactly with the displayed row indices.
    hits: Vec<SearchHit>,
    /// Global key capture recognizes editord's release-based double-Shift gesture.
    gesture: DoubleShift,
    /// Elapsed monotonic time is injected into the pure gesture model.
    clock: Instant,
    /// Closing without choosing a result restores the tree when it previously owned focus.
    return_tree: bool,
}

/// Construct worker ownership once per window,
///  without scanning any project files.
impl Search {
    /// What:
    ///  `Result<Self>` carries either the new state or a startup error;
    ///  `?` forwards that error.
    /// Why:
    ///  Window startup must report a missing worker instead of offering a nonfunctional overlay.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const search = { worker: createWorker(workspace), scope: workspace.root, hits: [] };
    /// ```
    pub(super) fn new(workspace: Workspace) -> Result<Self> {
        let scope = workspace.root().to_path_buf();
        return Ok(Self {
            worker: SearchWorker::new(workspace)?,
            directory: None,
            scope,
            input: None,
            edited: None,
            hits: Vec::new(),
            gesture: DoubleShift::default(),
            clock: Instant::now(),
            return_tree: false,
        });
    }
}
