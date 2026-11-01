//! Page selection on the screen state: the cached page index, the selected page, and the pages' measures.
//!
//! What: `impl ScreenState` methods that set the queue names (building the page index once), find the selected page,
//! and watch which library paths belong to it.
//! Why: every projection used to regroup the whole queue by name to find one page, and every sweep chunk used to
//! redo the projection; with the index built when the queue arrives, a projection reads the selected page only, and a
//! measure report is checked against that page's paths instead of repainting.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! state.setNames(names);                    // builds the page index once
//! const page = state.currentPages().page(state.selectedPagePosition());
//! ```

use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use music_player::page_index::PageIndex;
use music_player::track_measure::TrackMeasure;

use crate::ui_screen_view::ScreenState;

/// What:
///      `pub struct WatchedPage { generation, page, paths }`.
///  The absolute paths of the selected page, remembered for the library and page they were taken from.
/// Why:
///       A measure report repaints the window only when one of its paths is on the shown page; the set is built once
///  per page, not once per report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WatchedPage {
    /// Queue generation the set was taken from.
    generation: u64,
    /// Display position of the page.
    page: usize,
    /// Paths of the page's tracks.
    paths: HashSet<PathBuf>,
}

/// Methods of `ScreenState` about pages.
impl ScreenState {
    /// What: `set_names` replaces the queue names, starts a new queue generation and rebuilds the page index.
    /// Why: the index is keyed by the queue generation, so it is built once per queue and never again per
    /// projection. Returns the shared names so the Search index can be fed from them without another copy.
    pub fn set_names(&mut self, names: &[String]) -> Arc<Vec<String>> {
        let shared: Arc<Vec<String>> = Arc::new(names.to_vec());
        self.names = Arc::clone(&shared);
        self.queue_generation += 1;
        self.page_index = PageIndex::build(&shared);
        self.watched_page = None;
        return shared;
    }


    /// What: `set_names_keeping_place` replaces the queue names after a rescan and, when the selected folder is gone,
    /// selects the folder now at the same place in the list.
    /// Why: deleting the last file of the folder on screen used to switch the view to the first folder with no word
    /// (the polish review finding PB2). The nearest folder keeps the user in the same part of the list.
    pub fn set_names_keeping_place(&mut self, names: &[String]) -> Arc<Vec<String>> {
        let previous_position: usize = self.selected_page_position();
        let had_selection: bool = !self.selected_key.is_empty();
        let shared: Arc<Vec<String>> = self.set_names(names);
        if !had_selection || self.page_index.position_of_identity(&self.selected_key).is_some() {
            return shared;
        }
        let count: usize = self.page_index.page_count();
        if count == 0 {
            return shared;
        }
        let nearest: usize = previous_position.min(count - 1);
        if let Some(page) = self.page_index.page(nearest) {
            tracing::info!(from = %self.selected_key, to = %page.identity, "the selected folder is gone; its neighbor is shown");
            self.selected_key = page.identity.clone();
        }
        return shared;
    }

    /// What: `set_source_root_name` records the opened folder's name.
    /// Why: the first-run copy and the header name the folder; the pages are built from the names alone (loose files sit in
    /// the virtual folders of their letters, as in Search), so the name does not touch them.
    pub fn set_source_root_name(&mut self, name: Option<String>) {
        self.root_known = true;
        self.source_root_name = name;
    }

    /// What: `current_pages` is the page index of the current names.
    /// Why: a state whose names were assigned without `set_names` (a test fixture) still gets correct pages: the
    /// index is rebuilt for that call only, which is slow but never wrong.
    pub fn current_pages(&self) -> Cow<'_, PageIndex> {
        if self.page_index.covered() == self.names.len() {
            return Cow::Borrowed(&self.page_index);
        }
        return Cow::Owned(PageIndex::build(&self.names));
    }

    /// What: `selected_page_position` is the display position of the selected page, or the first page.
    /// Why: a queue rebuild keeps the same folder selected when its identity survives.
    pub fn selected_page_position(&self) -> usize {
        return self
            .current_pages()
            .position_of_identity(&self.selected_key)
            .unwrap_or(0);
    }

    /// What: `page_key_of_index` gives the identity of the page holding queue index `index`.
    /// Why: a track that starts playing moves the picker to its folder, and finding that page must not scan the queue.
    pub fn page_key_of_index(&self, index: usize) -> Option<String> {
        let pages: Cow<'_, PageIndex> = self.current_pages();
        let position: usize = pages.page_of_index(index)?;
        return pages.page(position).map(|page| return page.identity.clone());
    }

    /// What: `selected_page_scope_indices` lists the queue indices of the selected page, in page order.
    /// Why: the engine plays shuffle-this-page and in-order from the page the user is looking at.
    pub fn selected_page_scope_indices(&self) -> Vec<usize> {
        let pages: Cow<'_, PageIndex> = self.current_pages();
        let position: usize = pages.position_of_identity(&self.selected_key).unwrap_or(0);
        return pages.page(position).map_or_else(Vec::new, |page| {
            return page.indices().iter().map(|index| return *index as usize).collect();
        });
    }

    /// What: `measures_touch_selected_page` tells whether any reported path is on the selected page.
    /// Why: a sweep reports hundreds of thousands of tracks in chunks, and only a chunk that holds a shown row can
    /// change what the window draws.
    pub fn measures_touch_selected_page(&mut self, reported: &[(PathBuf, TrackMeasure)]) -> bool {
        let position: usize = self.selected_page_position();
        let stale: bool = !matches!(&self.watched_page, Some(watched) if watched.generation == self.queue_generation && watched.page == position);
        if stale {
            let indices: Vec<usize> = self.selected_page_scope_indices();
            let paths: HashSet<PathBuf> = indices.iter().filter_map(|index| return self.paths.get(*index).cloned()).collect();
            self.watched_page = Some(WatchedPage {
                generation: self.queue_generation,
                page: position,
                paths,
            });
        }
        return self
            .watched_page
            .as_ref()
            .is_some_and(|watched| return reported.iter().any(|(path, _measure)| return watched.paths.contains(path)));
    }
}

/// What:
///      `pub fn selected_page_scope(state: &ScreenState) -> Vec<usize>`.
///  The queue indices of the selected page, in page order.
/// Why:
///       Callers pass it to `read_state` as a function; it is the method with the old name.
pub fn selected_page_scope(state: &ScreenState) -> Vec<usize> {
    return state.selected_page_scope_indices();
}

/// What:
///      `pub fn select_folder_label(state: &mut ScreenState, label: &str) -> Option<Vec<usize>>`.
///  Selects the first page named `label`, closes the picker, and returns its scope.
/// Why:
///       A folder pick changes the selected page and the engine scope together; an unknown label changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selectFolderLabel = (state: ScreenState, label: string): number[] | null => { ... };
/// ```
pub fn select_folder_label(state: &mut ScreenState, label: &str) -> Option<Vec<usize>> {
    let identity: String = {
        let pages: Cow<'_, PageIndex> = state.current_pages();
        let position: usize = pages.position_of_label(label)?;
        pages.page(position)?.identity.clone()
    };
    state.selected_key = identity;
    state.picker_open = false;
    return Some(state.selected_page_scope_indices());
}
