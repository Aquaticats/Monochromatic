//! Read-only document and workspace logic,
//!  independent of the native window.

/// What:
///  Export the document module without exporting a writable filesystem API.
/// Why:
///  Tests and the native window share the same state transitions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as document from './document';
/// ```
pub mod document;

/// The one JSON-building macro whose expansion may call `Result::unwrap`.
mod json_value;

/// Semantic source classifications do not depend on terminal-cell geometry.
pub mod source_style;

/// Session-local Ctrl+digit history with editord's promotion semantics.
pub mod recent;

/// Tab projection retains exact source/display correspondence.
pub mod text_projection;

/// Pixel tab stops at multiples of two space advances,
///  independent of the preceding script.
pub mod tab_stop;
/// Soft-wrap break rules from IntelliJ,
/// as Parley line-break overrides,
///  and the continuation indent of a wrapped line.
pub mod wrap;
/// The wrap indicator of the gutter:
///  IntelliJ's symbol chain checked against the fonts,
/// its color,
/// and the arrow drawn when no symbol fits.
pub mod wrap_sign;

/// Two-pass shaping that widens each tab's stand-in space to its stop.
mod tab_layout;

/// How many visual rows each source line takes,
/// in chunks that copies share.
pub mod line_rows;
/// The minimap that scrolls the source view:
///  its geometry and the picture of the whole text it shows.
pub mod minimap;
/// The one mapping between vertical pixels and source lines,
///  with blocks of virtual rows above annotated lines.
pub mod row_map;

/// One shaped row with its caret,
///  hit-test,
///  and range geometry.
pub mod shaped_row;
/// Shaped source rows kept by line,
/// so a re-centered tile shapes only lines it has not shaped.
pub mod shaped_row_cache;

/// Shared shaping for mixed-script source rows,
///  caret,
///  and hit testing.
pub mod shaped_text;

/// Grapheme,
///  word,
///  line,
///  and document caret movement over source characters.
pub mod caret_motion;

/// Home and End on the visual rows of a soft-wrapped line.
pub mod line_boundary;

/// Up,
///  Down,
///  and page movement by shaped pixel position with a remembered column.
pub mod vertical_motion;

/// Click counting and unit-wise drag extension for pointer selection.
pub mod pointer_selection;

/// The palette of source ink:
///  syntax roles,
///  default and selected foreground,
///  per color scheme.
pub mod code_colors;

/// The GPU code view's picture of a shaped frame:
///  paint items and the Vello scene they encode to.
pub mod tile_scene;

/// Paint identity separates source-image updates from caret presentation.
pub mod source_frame;

/// Disk reads prepare correspondence without mutating project files or UI state.
pub mod file_reload;

/// One bounded background read/diff job keeps the native input loop independent.
pub mod reload_worker;

/// Worker-side target resolution and revision-bound source classification.
mod reload_read;

/// The application's private cache directory below `$XDG_CACHE_HOME`.
pub mod app_cache;
/// The fixed FNV-1a digest shared by the build script and the embedded-runtime checks.
pub mod content_digest;
/// The application's own language runtime:
///  embedded in the binary,
///  or a directory for tests and tools.
pub mod runtime;
/// Helix-backed source classifications use canonical character offsets.
pub mod syntax;
/// The highlighting loader that reads grammars and queries only from the application's own runtime.
mod syntax_loader;

/// User-facing parser diagnostics distinguish limits from missing assets.
mod syntax_error;

/// Language-server feature paths run on one worker thread and are polled;
///  server edits are refused.
pub mod language;

/// The log:
///  warnings by default and `RUST_LOG` for detail,
///  helix-lsp records labelled by what they
/// are,
///  and a writer thread so no logging thread waits for the output.
pub mod logging;

/// Inlay hints and diagnostics of the displayed file,
///  grouped into blocks per line;
///  stale snapshots paint nothing.
pub mod annotation;

/// Underline runs and the texts of virtual rows,
///  positioned from the shaped rows;
///  hint rows are packed here.
pub mod annotation_layout;

/// The rows a line shows above its code row:
///  placed hints,
///  wrapped diagnostic messages,
///  and their spacing.
pub mod virtual_row;

/// The measures of the diagnostic underlines,
///  one line style per severity.
pub mod annotation_paint;

/// Selected-text ink follows the selection background's lightness,
///  not the color scheme.
pub mod selection_ink;

/// Sibling names in the file tree sort mixed,
/// case folded,
/// with numbers compared by value.
pub mod name_order;

/// One local project boundary exposes directory snapshots without mutation operations.
pub mod workspace;

/// Lazy tree rows and expansion state consume background directory snapshots without I/O.
pub mod file_tree;

/// Which listed entries git ignores,
///  decided from the rule files with ripgrep's `ignore` crate and without a git process.
pub mod git_ignore;

/// Bounded background directory reads apply only current tree-request replies.
pub mod directory_worker;

/// inotify notifications for shown directories and the displayed file,
///  reported as invalidations only.
pub mod change_watch;

/// When to reread:
///  on notifications,
///  on today's timers while unwatched,
///  and on a slow safety sweep.
pub mod refresh_policy;

/// Startup argument grammar is independent of filesystem and native display initialization.
pub mod cli;

/// Latest-request-wins project source opening preserves the displayed document until success.
pub mod file_open;

/// Bounded combined-search results retain independent filename and content outcomes.
pub mod search;

/// Ripgrep JSON and native filename payloads are validated before entering result models.
pub mod search_protocol;

/// Filename matching follows editord's smart-case literal-substring rule.
pub mod search_query;

/// Overlay query parsing and shortcut timing are independent of native event delivery.
pub mod search_input;

/// One-way cancellation reaches both search children without waiting for another output record.
pub mod search_cancel;

/// One latest query and one latest reply keep search independent of native event-loop timing.
pub mod search_worker;

/// Child output is bounded before parsing or diagnostic retention.
mod search_io;

/// Streaming collectors preserve result order and stop at the approved caps.
mod search_collect;

/// Read-only ripgrep subprocesses are killed and reaped on cancellation or output limits.
mod search_process;

/// Plain literal,
///  case-insensitive in-file matching kept in one replaceable function.
pub mod find;

/// Active,
///  next,
///  previous,
///  and visible matches derived from the reading selection.
pub mod find_navigation;

/// One bounded in-file find job;
///  replies are tagged by file generation,
///  revision,
///  and query.
pub mod find_worker;

/// Match rectangles for materialized rows share selection's shaped geometry.
pub mod find_paint;

/// Unmodified variable and real italic font assets with stable source-font identities.
pub mod font_asset;

/// Validated immutable source weight,
///  italic,
///  and OpenType feature settings.
pub mod source_typography;
