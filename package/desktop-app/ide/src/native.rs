//! Source-view binding gate;
//!  filesystem and language integration follow separately.

/// What:
///  Slint's procedural macro imports the actual UI while retaining generated-code provenance.
/// Why:
///  Clippy can distinguish compiler output from app bindings without relaxing source lint rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as Ui from '../ui/app.slint';
/// ```
mod ui {
    // Use the toolkit's supported re-export syntax rather than editing generated Rust.
    slint::slint! {
        export { AppWindow, SourceSelection } from "../ui/app.slint";
    }
}

/// Hint and diagnostic snapshots with their position index.
use ide_app::annotation::Annotations;
/// Accepted in-file matches carry the file generation and revision they describe.
use ide_app::find_navigation::FindResults;
/// The stamp naming a displayed text.
use ide_app::language::identity::DocumentStamp;
/// The reload record the state keeps until the Language module is told.
use ide_app::language::sync::DocumentReload;
/// Visual rows of every line,
/// as far as measured.
use ide_app::line_rows::LineRows;
/// The displayed file is reread on change notifications,
///  or on the old timer while unwatched.
use ide_app::refresh_policy::SourceRefresh;
/// The one vertical mapping between pixels and source lines.
use ide_app::row_map::RowMap;
/// Shared shaping replaces terminal-column assumptions in native hit testing.
use ide_app::shaped_text::{Clip, ShapedView, TextShaper};
/// Paint identity prevents caret movement from rebuilding source pixels.
use ide_app::source_frame::FrameStamp;
/// One line's block of virtual rows.
use ide_app::virtual_row::Block;
/// Source and display geometry use the same library interface tested headlessly.
use ide_app::{document::Document, source_style::SourceStyles};
/// Toolkit handles and models bridge owned Rust state to the window.
use slint::{Timer, VecModel};
/// What:
///  Rc shares one UI-thread owner;
///  RefCell permits checked mutable borrowing.
/// Why:
///  Callbacks need the same document without cross-thread Arc/Mutex overhead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shared = { current: state };
/// ```
use std::{path::PathBuf, rc::Rc, sync::Arc, time::Instant};
/// Native window and model row generated from the UI declaration.
use ui::AppWindow;

/// The find and search boxes and their clear controls through Slint's element handles,
///  as assistive tools see them.
#[cfg(test)]
mod accessible_box_tests;
/// The sidebar divider through Slint's element handles:
///  slider role,
///  value,
///  bounds,
///  step,
///  actions,
///  and keys.
#[cfg(test)]
mod accessible_divider_tests;
/// Tree rows,
///  search results,
///  and the location list through element handles:
///  roles and selected states.
#[cfg(test)]
mod accessible_list_tests;
/// Inlay hints and diagnostics:
///  the snapshot setter,
///  the visible subset,
///  and the problems at the caret.
mod annotate;
/// The gutter's severity letters in front of line numbers,
///  in both schemes,
///  with measured contrast.
#[cfg(test)]
mod annotation_gutter_tests;
/// Rendered annotation pixels in both schemes,
///  visible-only repaints,
///  the scroll range,
///  and display scale.
#[cfg(test)]
mod annotation_paint_tests;
/// Hints a server places outside the lines it was asked about are not part of its answer.
#[cfg(test)]
mod annotation_range_tests;
/// The pixels the GPU draws for annotations:
/// one underline style per severity,
/// virtual rows,
/// and the scheme's inks.
#[cfg(test)]
mod annotation_scene_tests;
/// Which rendered pixels may move when virtual rows arrive,
///  change,
///  or vanish,
///  and which may not.
#[cfg(test)]
mod annotation_stability_tests;
/// Injected hint and diagnostic snapshots through real key and pointer events.
#[cfg(test)]
mod annotation_tests;
/// Real key events drive caret movement,
///  Shift selection,
///  paging,
///  and caret-following scroll.
#[cfg(test)]
mod caret_tests;
/// In-file find bar,
///  worker polling,
///  and selection-based match navigation.
mod find;
/// The find box's 48px clear cell:
///  every corner clears,
///  its edges end it,
///  and it follows focus and text.
#[cfg(test)]
mod find_clear_tests;
/// File switching and the modal search overlay must coexist with an open find bar.
#[cfg(test)]
mod find_interplay_tests;
/// Real key events drive the find bar,
///  reloads,
///  file switches,
///  and the search overlay together.
#[cfg(test)]
mod find_tests;
/// Tab traversal between tree,
///  source,
///  and find bar;
///  typing never edits the read-only source.
#[cfg(test)]
mod focus_tests;
/// Native font instance changes must repaint rather than reuse stale glyphs.
#[cfg(test)]
mod font_tests;
/// Debug records of the moments a window frame passes through.
mod frame_trace;
/// The native tests' GPU snapshot harness:
///  Slint windows drawn on a hardware Vulkan device and read back as pixels.
#[cfg(test)]
mod gpu_snapshot;
/// The rendering notifier fires on a harness snapshot,
///  and a Vello texture imported into the window appears in it.
#[cfg(test)]
mod gpu_texture_probe_tests;
/// The code view on the GPU:
/// the textures the tile and the wrap indicator are drawn into,
/// and the rendering notifier that draws them.
mod gpu_tile;
/// Tree rows that git ignores draw their name at 0.8 opacity and nothing else of the row changes.
#[cfg(test)]
mod ignored_row_tests;
/// Source selection and keyboard callbacks.
mod input;
/// Inspection-only hint and diagnostic injection from a JSON file;
///  debug builds only.
#[cfg(debug_assertions)]
mod inspect;
/// Go to definition,
///  references,
///  and hover through the Language module.
mod language;
/// The minimap beside the source text:
/// the scroll control,
/// and the picture of the whole text built off-thread.
mod minimap;
/// The densest slices of text the minimap draws render in the renderer's fixed buffers.
#[cfg(test)]
mod minimap_dense_tests;
/// The minimap's picture is the average of the window's own pixels,
/// for plain text,
/// selections,
/// matches,
/// wrapped lines,
/// and diagnostics.
#[cfg(test)]
mod minimap_reflection_tests;
/// The chunks the minimap's picture is rendered in,
///  stacked,
///  equal one render of the picture.
#[cfg(test)]
mod minimap_seam_tests;
/// The minimap's geometry,
/// picture,
/// and pointer paths.
#[cfg(test)]
mod minimap_tests;
/// The minimap's worker thread.
mod minimap_worker;
/// Project tree and asynchronous successful-file navigation.
mod navigation;
/// Missing targets and canonical aliases exercise reveal liveness and model identity.
#[cfg(test)]
mod navigation_reveal_tests;
/// Native project callbacks exercise actual reader/timer and source replacement boundaries.
#[cfg(test)]
mod navigation_tests;
/// Scrolling inside the overscan tile does no source work,
/// and leaving it re-centers the tile once.
#[cfg(test)]
mod overscan_tile_tests;
/// A picture asks for a frame of its own only when no frame draws it first.
#[cfg(test)]
mod picture_frame_tests;
/// Real pointer events select by character,
///  word,
///  and line,
///  extend with Shift,
///  and drag without panning.
#[cfg(test)]
mod pointer_tests;
/// Rendered search box in both schemes:
///  placeholder,
///  focus mark,
///  selection colors,
///  and clear states.
#[cfg(test)]
mod query_input_paint_tests;
/// The find box keeps the toolkit box's editing keys,
///  context menu,
///  and scrolling of long text.
#[cfg(test)]
mod query_input_tests;
/// Reads no write notification asked for do not show a save in progress.
#[cfg(test)]
mod quiet_read_tests;
/// External-write-to-window timings for the tree and the displayed source;
///  ignored by default.
#[cfg(test)]
mod refresh_latency_tests;
/// A tile the painter refuses never blanks the source,
///  and a worker job that failed is freed.
#[cfg(test)]
mod refused_tile_tests;
/// Background source reads apply correspondence to the latest UI reading state.
mod reload;
/// Native rendering and input are split by their invalidation boundary.
mod render;
/// The window thread's counted step when the wrap width changes.
mod rewrap;
/// The window's side of the vertical mapping between pixels and source lines.
mod rows;
/// The search box's 48px clear cell empties the query and the results and keeps focus.
#[cfg(test)]
mod search_clear_tests;
/// Replacement search queries cannot redirect an in-progress result click.
#[cfg(test)]
mod search_pointer_tests;
/// Combined search exercises native key capture,
///  real ripgrep,
///  scope,
///  and source-line navigation.
#[cfg(test)]
mod search_tests;
/// Selected rows of the tree and both lists use the ink chosen from the selection fill.
#[cfg(test)]
mod selected_row_tests;
/// Tree rows and source text beside the sidebar divider keep their own clicks at every width.
#[cfg(test)]
mod sidebar_adjacent_tests;
/// The sidebar divider is a Tab stop adjusted by keys,
///  and absent without a project.
#[cfg(test)]
mod sidebar_keyboard_tests;
/// Source repaint,
///  window resizing,
///  the find bar,
///  and the search overlay beside a resized sidebar.
#[cfg(test)]
mod sidebar_layout_tests;
/// Rendered divider states and tree painting inside the narrowest sidebar.
#[cfg(test)]
mod sidebar_paint_tests;
/// Real pointer drags on the sidebar divider resize within both bounds.
#[cfg(test)]
mod sidebar_tests;
/// Source typography reaches the actual image and selection bindings.
#[cfg(test)]
mod source_font_tests;
/// Starting one project window:
///  logging,
///  the language worker,
///  every binding,
///  and the event loop.
mod start;
/// The initial reading state of a window.
mod state;
/// Selections of dozens of rows in a tall tile keep every glyph:
///  the renderer's tile buffer is not used up by the clips of the selected rows.
#[cfg(test)]
mod tall_selection_tests;
/// The minimap of a tall window is a picture of glyphs,
///  not blocks.
#[cfg(test)]
mod tall_window_minimap_tests;
/// A tall window over wrapped lines no measurement has reached:
///  the tile is cut to the raster limit,
///  the text stays on screen,
///  and no window size or scale makes a raster past it.
#[cfg(test)]
mod tall_window_tests;
/// Consumer window events exercise the actual markup and source-image bindings.
#[cfg(test)]
mod tests;
/// A live system color-scheme change repaints source ink,
///  find overlays,
///  and window chrome.
#[cfg(test)]
mod theme_tests;
/// Scroll re-center bounds and the window-thread step that swaps a finished tile in.
mod tile;
/// Source tile construction and the worker thread that runs it.
mod tile_build;
/// The background measurement of every line at the wrap width.
mod tile_pass;
/// The native tests' reads of the GPU tile:
/// a frame is rendered and the tile's texture is read back.
#[cfg(test)]
mod tile_reads;
/// The code tile drawn across stacked textures shows what one texture shows.
#[cfg(test)]
mod tile_stack_tests;
/// A finished tile is swapped in by the next scroll step's frame,
/// and by the poll only when no step is due.
#[cfg(test)]
mod tile_step_tests;
/// Windowed rows must not redirect an in-progress click after scrolling.
#[cfg(test)]
mod tree_pointer_tests;
/// Long-distance tree reveal must show the whole target row after native layout.
#[cfg(test)]
mod tree_scroll_tests;
/// Fractional viewport movement and bounded tile materialization.
mod viewport;
/// Bounded waiting that advances native timers until a state the test checks holds.
#[cfg(test)]
mod waiting_tests;
/// An ignored measurement of watch counts and staleness when a changed folder is scrolled into view.
#[cfg(test)]
mod watch_scope_tests;
/// External changes reach the tree and source through inotify notifications,
///  faster than polling could.
#[cfg(test)]
mod watch_tests;
/// Hints and message rows on wrapped lines:
///  their place above the first visual row,
///  their width,
///  presses on them,
///  and new window widths.
#[cfg(test)]
mod wrap_annotation_tests;
/// A line wrapped to more rows than one raster holds is painted in a clip,
///  and scrolling inside a built tile of a wrapped file does no source work.
#[cfg(test)]
mod wrap_clip_tests;
/// Home,
/// End,
/// Up,
/// Down,
/// paging,
/// word motion,
/// drags,
/// and multi-clicks on wrapped lines.
#[cfg(test)]
mod wrap_keys_tests;
/// A new window width:
/// the window thread breaks the lines near the view again and paints the tile for it before the
///  next frame,
/// within a counted allowance,
/// and what does not fit goes to the tile worker.
#[cfg(test)]
mod wrap_resize_tests;
/// The pixels the GPU draws for wrapped lines,
/// clips,
/// selections,
/// and the wrap indicator.
#[cfg(test)]
mod wrap_scene_tests;
/// Always-on soft wrap in the window:
///  the wrap width,
///  the gutter's indicator,
///  the worker's measurement of every line,
///  resizes,
///  and clicks on continuation rows.
#[cfg(test)]
mod wrap_tests;
/// A save in progress is not shown before it finishes;
///  an ignored measurement finds the pause that is.
#[cfg(test)]
mod write_wait_tests;
/// An ignored measurement of saves that begin when the safety sweep comes due.
#[cfg(test)]
mod write_wait_timer_tests;
/// The handle the state holds on the GPU side of the code view.
use gpu_tile::GpuTile;
/// The wrap indicator's symbol.
use ide_app::wrap_sign::Sign;
/// Bind caret and selection callbacks.
use input::{bind_keys, bind_pointer};
/// Shared rendering entry point.
use render::{bind_appearance, render};
/// The one entry point of the native window.
pub use start::run;
/// Tile bounds of the displayed and requested windows.
use tile::TileBounds;
/// The worker thread that builds scroll re-centers,
/// held by the state.
use tile_build::TileWorker;
/// What a background measurement is valid for.
use tile_pass::PassKey;
/// Bind viewport changes without line-snapping native scrolling.
use viewport::bind_viewport;

/// Shared UI-thread state;
///  there is no project-writing operation.
struct State {
    /// Canonical source and current reading position.
    document: Document,
    /// Optional authoritative disk file;
    ///  absent only for the explicit in-memory fixture.
    file_path: Option<PathBuf>,
    /// File-open identity rejects worker replies after future navigation.
    file_generation: u64,
    /// A failed refresh retains source and displays an actionable error only while needed.
    file_error: Option<String>,
    /// New-file open failures remain visible independently of the displayed file's refresh result.
    navigation_error: Option<String>,
    /// Last revision whose highlighting result was accepted,
    ///  including plain-text or failed results.
    syntax_revision: Option<u64>,
    /// Highlight failures remain distinct from file-read failures.
    syntax_error: Option<String>,
    /// Highlight ranges,
    ///  populated by the syntax integration.
    styles: SourceStyles,
    /// First materialized source line,
    ///  including viewport overscan.
    first: usize,
    /// Bounded number of materialized source lines.
    count: usize,
    /// The part of those lines the next tile holds,
    /// when they are too tall for one raster.
    clip: Option<Clip>,
    /// Physical pixels per logical pixel as of the last render or viewport report,
    /// which the tile's size limit uses.
    scale: f32,
    /// What:
    ///  The wrap width in logical pixels every line is broken at,
    /// or zero before the window reported one.
    /// Why:
    ///  The text area's width minus the caret room;
    /// a width of zero or less keeps the last width.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// width: number; // 0 until the view has a width
    /// ```
    width: f32,
    /// How many visual rows every line takes,
    /// as far as it is known:
    ///  exact for lines measured at `width`,
    /// the last known count for the rest,
    /// one for a line never measured.
    wrap_rows: LineRows,
    /// The text `wrap_rows` describes.
    wrap_text: Option<DocumentStamp>,
    /// Counter of changes to `wrap_rows`,
    /// part of the row key.
    wrap_version: u64,
    /// What the pass last posted to the worker was for,
    /// and the syntax snapshot it used.
    pass_posted: Option<(PassKey, usize)>,
    /// The symbol the gutter shows on a continuation row,
    /// chosen once from the fonts.
    sign: Option<Sign>,
    /// The display scale the gutter's sign image was painted for.
    sign_scale: Option<u32>,
    /// Reusable paragraph/font resources.
    shaper: TextShaper,
    /// The GPU side of the code view:
    /// the textures the tile and the wrap indicator are drawn into,
    /// and the scenes waiting for the next frame.
    gpu: GpuTile,
    /// Exact displayed geometry used by pointer input.
    shaped: Option<ShapedView>,
    /// Avoid cloning whole source for accessibility on each selection change.
    presented_revision: Option<u64>,
    /// Last materialized image inputs;
    ///  reset when changing the displayed file.
    frame_stamp: Option<FrameStamp>,
    /// Where every line of the displayed text is vertically;
    ///  kept current by `rows::refresh`.
    row_map: RowMap,
    /// What:
    ///  The blocks of virtual rows of every annotated line,
    ///  in line order;
    ///  `Arc` shares each block
    ///       with the frames that paint it (siblings:
    ///  single-thread `Rc`,
    ///  owning `Box`).
    /// Why:
    ///  The map takes its block heights from these,
    ///  and a frame takes the blocks of its lines.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// blocks: Readonly<Block>[];
    /// ```
    blocks: Vec<Arc<Block>>,
    /// What:
    ///  What `blocks` and `row_map` were built from:
    ///  the displayed text's stamp,
    ///  the annotation store's
    ///       change counter,
    ///  and the display scale as exact bits;
    ///  `Option<...>` is nothing before the first build.
    /// Why:
    ///  Blocks are assembled again only when one of the three changed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rowKey?: [stamp: DocumentStamp, version: number, scale: number];
    /// ```
    row_key: Option<rows::RowKey>,
    /// Vertical scroll offset the materialized lines were last chosen for.
    offset: f32,
    /// What:
    ///  When the reader last scrolled vertically;
    ///  `Option<Instant>` is that moment or nothing.
    /// Why:
    ///  A change of rows that would move the scroll offset waits until the toolkit's scroll animation,
    ///      which runs through a binding on that offset,
    ///  has had time to finish.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scrolledAt?: number;
    /// ```
    scrolled_at: Option<Instant>,
    /// What:
    ///  When the view last moved by a scroll step,
    ///  on the system clock.
    /// Why:
    ///  The poll of the tile worker leaves a finished tile for the next step
    ///  while a step is due,
    ///  so the tile is swapped in by the frame of that step.
    ///  `scrolled_at` runs on a clock the tests hold still.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// steppedAt?: number;
    /// ```
    stepped_at: Option<Instant>,
    /// What:
    ///  `Rc<VecModel<f32>>` is a shared,
    ///  growable toolkit list of floats.
    /// Why:
    ///  The window draws one gutter entry per visual row of the frame at that vertical position;
    ///  the list is updated in place while scrolling.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rowTops: ArrayModel<number>;
    /// ```
    row_tops: Rc<VecModel<f32>>,
    /// What:
    ///  `Rc<VecModel<i32>>` is a shared toolkit list of 32-bit integers.
    /// Why:
    ///  Per gutter entry,
    ///  the one-based line number on the first visual row of a line and zero on a continuation row,
    ///  where the wrap indicator stands instead.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rowNumbers: ArrayModel<number>;
    /// ```
    row_numbers: Rc<VecModel<i32>>,
    /// What:
    ///  `Rc<VecModel<i32>>` is a shared toolkit list of 32-bit integers.
    /// Why:
    ///  Per gutter entry,
    ///  the severity letter:
    ///  0 for none,
    ///  otherwise one more than the rank of
    ///      the worst diagnostic starting on the line;
    ///  only the first visual row of a line carries one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rowMarks: ArrayModel<number>;
    /// ```
    row_marks: Rc<VecModel<i32>>,
    /// Accepted in-file matches;
    ///  painted only while they describe the displayed file and revision.
    find: Option<FindResults>,
    /// The latest accepted external reload the Language module has not been told about yet.
    language_reload: Option<DocumentReload>,
    /// Latest accepted hint and diagnostic snapshots,
    ///  stored by the language poll;
    /// painted and handed out only while they describe the displayed file and revision.
    annotations: Annotations,
    /// The displayed file lies outside the project;
    ///  it was opened read-only from a language target.
    outside_project: bool,
    /// Where the caret goes once the file of a language target is installed.
    pending_jump: Option<language::Jump>,
    /// When to reread the displayed file:
    ///  on change notifications,
    ///  or on a timer while unwatched.
    refresh: SourceRefresh,
    /// Bounds of the tile on screen:
    /// the frame,
    /// its gutter,
    /// and its line numbers.
    /// It stays on screen while a scroll re-center is built,
    /// so it can differ from the requested window.
    shown: TileBounds,
    /// Scroll re-center the reader's last viewport change asked for,
    /// until it is shown or superseded.
    scroll_wanted: Option<TileBounds>,
    /// A viewport report changed the wrap width and the next render is to take it on the window thread within an
    /// allowance;
    /// the render clears it.
    rewrap: bool,
    /// Bounds of the tile last queued for the worker and not cancelled;
    /// its result is accepted only while set.
    tile_job: Option<TileBounds>,
    /// The tile worker thread,
    /// started on the first scroll re-center.
    tile_worker: Option<TileWorker>,
    /// Polls the tile worker's replies on the window thread while a re-center is queued.
    tile_poll: Timer,
    /// What:
    ///  A single wake-up for the end of the wait after a scroll step,
    ///  set by a poll that left a finished tile for the next step.
    /// Why:
    ///  The next repeated poll is a whole period later,
    ///  and the tile would wait most of it.
    ///  The wake-up runs the same function as the poll,
    ///  and every poll stops it.
    tile_wake: Timer,
    /// The minimap beside the source text:
    /// its size,
    /// worker,
    /// timers,
    /// and the drag in progress.
    minimap: minimap::MinimapState,
    /// What:
    ///  Calls of `render`,
    /// counted in debug builds for the structural tests of the overscan tile.
    /// Why:
    ///  A scroll inside the tile must not even enter `render`,
    /// which would still update models.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// renders?: number;
    /// ```
    #[cfg(debug_assertions)]
    renders: u64,
    /// Source tiles rasterized,
    /// counted in debug builds:
    /// one per re-center.
    #[cfg(debug_assertions)]
    tiles_painted: u64,
    /// Source rows shaped,
    /// counted in debug builds:
    /// a row taken from the cache is not counted.
    #[cfg(debug_assertions)]
    rows_shaped: u64,
    /// Source images set on the window,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    images_set: u64,
    /// Kept rows the window thread broke again at a new width,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    rows_rebroken: u64,
    /// Changes of width the window thread took at once,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    rewrapped: u64,
    /// Changes of width left to the tile worker because they did not fit the allowance,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    rewrap_deferred: u64,
    /// Lines and work the last immediate change of width was charged,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    rewrap_spent: (usize, usize),
    /// Scroll re-center tiles completed by the tile worker,
    /// counted in debug builds when their result arrives.
    #[cfg(debug_assertions)]
    worker_tiles: u64,
    /// Source rows shaped by the tile worker,
    /// counted in debug builds from its results.
    #[cfg(debug_assertions)]
    worker_rows_shaped: u64,
    /// Worker tiles swapped in,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    tiles_applied: u64,
    /// Worker tiles discarded as stale,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    tiles_discarded: u64,
    /// The messages named for tiles the worker failed to build,
    /// kept in debug builds because the next render clears the window's error message,
    /// which can happen between two polls of a test.
    #[cfg(debug_assertions)]
    worker_failures: Vec<String>,
    /// Outputs of the background pass applied on the window thread,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    pass_applied: u64,
    /// Most lines of the pass's output applied in one window-thread callback,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    pass_most_lines: usize,
    /// Lines of the pass's output applied on the window thread,
    /// counted in debug builds.
    #[cfg(debug_assertions)]
    pass_lines: u64,
    /// Lines the worker shaped for the pass,
    /// counted in debug builds from its outputs.
    #[cfg(debug_assertions)]
    pass_shaped: u64,
    /// Kept rows the worker broke again for the pass,
    /// counted in debug builds from its outputs.
    #[cfg(debug_assertions)]
    pass_rebroken: u64,
}
