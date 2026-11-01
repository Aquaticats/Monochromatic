//! The initial reading state of a window:
//!  the source state the application and the headless native event tests both start from.

/// The source state that is constructed,
///  the toolkit list models of the gutter,
///  and the minimap beside the text.
use super::{State, gpu_tile::GpuTile, minimap, rows, tile::TileBounds, wheel};
/// Hint and diagnostic snapshots with their position index.
use ide_app::annotation::Annotations;
/// Visual rows of every line,
///  as far as measured.
use ide_app::line_rows::LineRows;
/// The displayed file is reread on change notifications,
///  or on the old timer while unwatched.
use ide_app::refresh_policy::SourceRefresh;
/// The one vertical mapping between pixels and source lines.
use ide_app::row_map::RowMap;
/// Shared shaping replaces terminal-column assumptions in native hit testing.
use ide_app::shaped_text::TextShaper;
/// Source and display geometry use the same library interface tested headlessly.
use ide_app::{document::Document, source_style::SourceStyles};
/// The timer that polls the tile worker is created idle.
use slint::Timer;
/// The optional disk path of the displayed file.
use std::path::PathBuf;

/// Construct the same reading state for the application and headless native event tests.
impl State {
    /// Retain source ownership and initialize viewport resources without changing the filesystem.
    pub(super) fn new(source: &str, file_path: Option<PathBuf>) -> Self {
        let document = Document::new(source);
        // A new text has no virtual rows yet: every line starts at its number times one code row.
        let lines = document.text().len_lines();
        let row_map = RowMap::plain(lines);
        return Self {
            document,
            file_path,
            file_generation: 1,
            file_error: None,
            navigation_error: None,
            syntax_revision: None,
            syntax_error: None,
            styles: SourceStyles::from([]),
            first: 0,
            count: 32,
            clip: None,
            scale: 1.0,
            width: 0.0,
            wrap_rows: LineRows::plain(lines),
            wrap_text: None,
            wrap_version: 0,
            pass_posted: None,
            sign: None,
            sign_scale: None,
            shaper: TextShaper::new(),
            gpu: GpuTile::default(),
            shaped: None,
            presented_revision: None,
            frame_stamp: None,
            row_map,
            blocks: Vec::new(),
            row_key: None,
            offset: 0.0,
            scrolled_at: None,
            stepped_at: None,
            row_tops: rows::top_model(),
            row_numbers: rows::mark_model(),
            row_marks: rows::mark_model(),
            find: None,
            language_reload: None,
            annotations: Annotations::default(),
            outside_project: false,
            pending_jump: None,
            refresh: SourceRefresh::default(),
            shown: TileBounds {
                first: 0,
                count: 32,
                width: 0.0,
                clip: None,
            },
            scroll_wanted: None,
            rewrap: false,
            tile_job: None,
            tile_worker: None,
            tile_poll: Timer::default(),
            tile_wake: Timer::default(),
            wheel: wheel::Wheel::default(),
            minimap: minimap::MinimapState::new(),
            #[cfg(debug_assertions)]
            renders: 0,
            #[cfg(debug_assertions)]
            tiles_painted: 0,
            #[cfg(debug_assertions)]
            rows_shaped: 0,
            #[cfg(debug_assertions)]
            images_set: 0,
            #[cfg(debug_assertions)]
            rows_rebroken: 0,
            #[cfg(debug_assertions)]
            rewrapped: 0,
            #[cfg(debug_assertions)]
            rewrap_deferred: 0,
            #[cfg(debug_assertions)]
            rewrap_spent: (0, 0),
            #[cfg(debug_assertions)]
            worker_tiles: 0,
            #[cfg(debug_assertions)]
            worker_rows_shaped: 0,
            #[cfg(debug_assertions)]
            tiles_applied: 0,
            #[cfg(debug_assertions)]
            tiles_discarded: 0,
            #[cfg(debug_assertions)]
            worker_failures: Vec::new(),
            #[cfg(debug_assertions)]
            pass_applied: 0,
            #[cfg(debug_assertions)]
            pass_most_lines: 0,
            #[cfg(debug_assertions)]
            pass_lines: 0,
            #[cfg(debug_assertions)]
            pass_shaped: 0,
            #[cfg(debug_assertions)]
            pass_rebroken: 0,
        };
    }
}
