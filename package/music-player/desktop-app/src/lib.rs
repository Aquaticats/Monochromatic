//! Library root for the `music-player` app.
//!  The binary (`main.rs`) wires
//! these modules to the Slint UI;
//!  keeping the logic in a library lets the pure
//! parts (queue,
//!  session) be unit-tested without any audio or GUI.

/// What:
///      `pub mod command;` declares and re-exports the `command` module
///           (the file `src/command.rs`).
///  `pub` makes it visible to the binary
///           and to tests.
/// Why:
///       Shared command/update message types live there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as command from "./command";
/// ```
pub mod command;

/// What:
///      `pub mod cli;` declares and re-exports the `cli` module
///           (the file `src/cli.rs`).
///  `pub` makes the `Cli` parser visible to the
///           binary (`main.rs` calls `Cli::parse()`) and to its sibling tests.
/// Why:
///       Command-line parsing is pure logic,
///  so it lives in the library where it
///           can be unit-tested without the GUI or an audio backend.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as cli from "./cli";
/// ```
pub mod cli;

/// What:
///      `pub mod licenses;` declares the license-notice module (`src/licenses.rs`).
/// Why:
///       The binary carries the license notices of everything compiled into it and prints them for
///           `--licenses`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as licenses from "./licenses";
/// ```
pub mod licenses;

/// What:
///      `pub mod identity;` the platform identity-string module
///           (`src/identity.rs`).
/// Why:
///       One home for the Wayland app id,
///  the macOS bundle id,
///  and the config-
///           dir reverse-DNS triple,
///  so the three platforms' names never drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as identity from "./identity";
/// ```
pub mod identity;

/// What:
///      `pub mod queue;` the play-queue model module.
/// Why:
///       Pure traversal/shuffle/repeat logic,
///  unit-tested.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as queue from "./queue";
/// ```
pub mod queue;

/// What:
///      `pub mod session;` the save/restore module.
/// Why:
///       Persists the last session to disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as session from "./session";
/// ```
pub mod session;

/// What: `pub mod text_size;` the Text size setting module (`src/text_size.rs`).
/// Why: the four text elements, their limits and defaults, and the stepping rule are pure logic shared by the session
/// file and the Settings page, so they live in the library where they are tested without a window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as textSize from "./text_size";
/// ```
pub mod text_size;

/// What:
///      `pub mod error;` the shared error type module.
/// Why:
///       `PlayerError` is the one error all fallible functions return.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as error from "./error";
/// ```
pub mod error;

/// What:
///      `pub mod decode;` the decoding module (`src/decode.rs`).
/// Why:
///       Probes files and yields interleaved f32 PCM via the `Source` trait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as decode from "./decode";
/// ```
pub mod decode;

/// What:
///      `pub mod opus;` the Opus decode path (`src/opus.rs`).
/// Why:
///       libopus-backed `Source` for Opus,
///  which symphonia cannot decode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as opus from "./opus";
/// ```
pub mod opus;

/// What:
///      `#[cfg(target_os = "linux")]` is a CONDITIONAL-COMPILATION attribute:
///           the item right below it is compiled into the program ONLY when the
///           build target is Linux,
///  and skipped entirely otherwise.
///           `#[path = "output_pipewire.rs"]` overrides the default filename,
///  so
///           the module named `output` is read from `src/output_pipewire.rs`
///           instead of `src/output.rs`.
///  `pub mod output;` declares and
///           re-exports that module.
/// Why:
///       Linux gets the native PipeWire backend;
///  everything else in the
///           player (engine.rs,
///  controller.rs) only ever names `output::Output`
///           and never learns which backend is behind it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as output from "./output_pipewire"; // only on Linux builds
/// ```
#[cfg(all(target_os = "linux", not(feature = "alsa-fallback")))]
#[path = "output_pipewire.rs"]
pub mod output;

/// What:
///      `#[cfg(all(target_os = "linux", feature = "alsa-fallback"))]` compiles the next item only on Linux builds
///           made with the `alsa-fallback` cargo feature (the Alpine build). `all(...)` is true when every listed
///           predicate is true. `#[path = "output_linux.rs"]` sources the `output` module from the file that tries
///           PipeWire first and falls back to ALSA.
/// Why:
///       Alpine and other minimal Linux installs run no sound server by default, so a PipeWire-only build is silent
///           there. The feature keeps every other Linux build exactly as it was (no new link-time dependency) while
///           exposing the IDENTICAL `output::Output` surface to the engine and the controller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as output from "./output_linux"; // only Linux builds with the alsa-fallback feature
/// ```
#[cfg(all(target_os = "linux", feature = "alsa-fallback"))]
#[path = "output_linux.rs"]
pub mod output;

/// What:
///      The non-Linux counterpart:
///  `#[cfg(not(target_os = "linux"))]` compiles
///           the next item on EVERY target except Linux (macOS and Windows here),
///           and `#[path = "output_cpal.rs"]` sources the `output` module from the
///           cpal file.
///  Same `pub mod output;` name as the Linux branch above;
///           exactly one of the two is ever compiled.
///  `not(...)` inverts a cfg
///           predicate,
///  so this is "compiled when NOT Linux".
/// Why:
///       Neither macOS nor Windows has PipeWire,
///  so both use cpal (CoreAudio on
///           macOS,
///  WASAPI on Windows) while exposing the IDENTICAL `output::Output`
///           surface,
///  keeping the rest of the player platform-agnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as output from "./output_cpal"; // on every non-Linux build
/// ```
#[cfg(not(target_os = "linux"))]
#[path = "output_cpal.rs"]
pub mod output;

/// What:
///      `pub mod audio_meter;` the underrun and late-callback counters (`src/audio_meter.rs`)
///           that the realtime output writes and the engine thread reports once a second.
/// Why:
///       Stutter must be measurable; the counters live apart from PipeWire so they compile on every platform.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as audioMeter from "./audioMeter";
/// ```
pub mod audio_meter;

/// What:
///      `pub mod thread_priority;` the decode thread's nice-level request to rtkit (Linux)
///           (`src/thread_priority.rs`).
/// Why:
///       Keeps the scheduling call in one place, with a no-op body on platforms without nice levels.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as threadPriority from "./threadPriority";
/// ```
pub mod thread_priority;

/// What:
///      `pub mod playback;` the device-free playback helpers (`src/playback.rs`):
///           the per-sample gain/clamp stage,
///  frame->seconds conversion,
///  and folder
///           expansion.
/// Why:
///       Pure logic kept apart so it is unit-tested and the files stay small.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as playback from "./playback";
/// ```
pub mod playback;

/// What:
///      `pub mod playback_clock;` the audible playback clock (`src/playback_clock.rs`):
///           position from the frames the output consumed,
///  and the end-of-track drain decision.
/// Why:
///       Pure arithmetic kept apart so it is unit-tested without an audio device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as playbackClock from "./playbackClock";
/// ```
pub mod playback_clock;

/// What:
///      `pub mod progress;` the progress-surface debounce helper module.
/// Why:
///       The binary uses this pure,
///  unit-tested timing gate for the Slint seek
///           bar and KDE taskbar progress.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as progress from "./progress";
/// ```
pub mod progress;

/// What:
///      `pub mod truepeak;` the true-peak measurement module (`src/truepeak.rs`).
/// Why:
///       Oversampled inter-sample peak measurement + the normalization gain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as truepeak from "./truepeak";
/// ```
pub mod truepeak;

/// What:
///      `pub mod peakcache;` the persistent peak cache (`src/peakcache.rs`).
/// Why:
///       Memoizes measured peaks on disk,
///  keyed by an opaque fingerprint.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as peakcache from "./peakcache";
/// ```
pub mod peakcache;

/// What:
///      `pub mod measure;` the measurement orchestration (`src/measure.rs`).
/// Why:
///       Per-track gain resolution + the background queue-measurement sweep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as measure from "./measure";
/// ```
pub mod measure;

/// What:
///      `pub mod peak_swap;` the current-track true-peak swap strategy
///           (`src/peak_swap.rs`).
/// Why:
///       Cache misses now start with a safe temporary gain,
///  start playback at once,
///           then swap to the measured gain when it lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as peakSwap from "./peak_swap";
/// ```
pub mod peak_swap;

/// What:
///      `pub mod controller;` the playback state machine (`src/controller.rs`),
///           with its loading/audio half in `src/controller_audio.rs`.
/// Why:
///       Owns the queue + decoder + output;
///  turns commands into playback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as controller from "./controller";
/// ```
pub mod controller;

/// What:
///      `pub mod controller_audio;` the second `impl Controller` block
///           (`src/controller_audio.rs`):
///  loading and audio pumping.
/// Why:
///       Split out so each controller file stays within the line budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // more methods of the same `controller` class, in another file
/// ```
pub mod controller_audio;

/// What:
///      `pub mod controller_flow;` the third `impl Controller` block (`src/controller_flow.rs`).
/// Why:
///       Play with nothing selected, the stop at the end of a folder and the report of unplayable files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // more methods of the same `controller` class, in another file
/// ```
pub mod controller_flow;

/// What:
///      `pub mod default_library;` the choice of the folder to open when no session names one (`src/default_library.rs`).
/// Why:
///       D95: a library is always open, so the home `Music` folder is the last place to look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as defaultLibrary from "./default_library";
/// ```
pub mod default_library;

/// What:
///      `pub mod dialog_backend;` whether a folder chooser can open (`src/dialog_backend.rs`).
/// Why:
///       `Open folder` must say what to do when no chooser exists (the polish review finding PB5).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as dialogBackend from "./dialog_backend";
/// ```
pub mod dialog_backend;

/// What:
///      `pub mod engine;` the worker-thread front door (`src/engine.rs`).
/// Why:
///       Spawns the worker and drives a `Controller` from the command channel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as engine from "./engine";
/// ```
pub mod engine;

/// What:
///      `pub mod watch;` the Source Root file watcher (`src/watch.rs`).
/// Why:
///       Watches the current Source Root and reports the paths that changed (`Command::SourceChanged`) after a
///           debounce, or everything (`Command::Rescan`) when events were lost,
///  so the queue live-updates while the app runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as watch from "./watch";
/// ```
pub mod watch;

/// What:
///      `pub mod pagination;` the queue-pagination module (`src/pagination.rs`).
/// Why:
///       Pure grouping of display paths into folder pages (subfolder tracks) and
///           A-Z + `#` letter pages (root-level tracks, virtual folders named by their letter),
///  unit-tested.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as pagination from "./pagination";
/// ```
pub mod pagination;

/// What:
///      `pub mod natural_order;` the natural name order (`src/natural_order.rs`).
/// Why:
///       The folder walk sorts file and folder names so `Track 2` comes before `Track 10`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as naturalOrder from "./natural_order";
/// ```
pub mod natural_order;

/// What:
///      `pub mod title_prefix;` the shared filename prefix removal of D86 (`src/title_prefix.rs`).
/// Why:
///       Rows and the deck show titles without the text every file of the folder starts with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as titlePrefix from "./title_prefix";
/// ```
pub mod title_prefix;

/// What:
///      `pub mod page_index;` the page index built once per library (`src/page_index.rs`).
/// Why:
///       The window reads the selected page from this index instead of regrouping every name on each projection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as pageIndex from "./page_index";
/// ```
pub mod page_index;

/// What:
///      `pub mod relpath;` the relative-path display module (`src/relpath.rs`).
/// Why:
///       Pure stripping of the queue's common directory prefix,
///  so the UI shows
///           each track's path relative to the loaded root;
///  unit-tested.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as relpath from "./relpath";
/// ```
pub mod relpath;

/// What:
///      `pub mod launcher;` the desktop-shell integration module
///           (`src/launcher.rs`):
///  the Wayland app-id hook and the KDE taskbar
///           progress signal.
/// Why:
///       Stamps the window app id and emits LauncherEntry progress over D-Bus.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as launcher from "./launcher";
/// ```
pub mod launcher;

/// What:
///      `pub mod logging;`.
///  The tracing subscriber setup (`src/logging.rs`):
///  a stderr
///           sink with an env-filter the binary installs once at startup.
/// Why:
///       Gives every `tracing` event from this crate and `truepeak-core` a sink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as logging from "./logging";
/// ```
pub mod logging;

/// What:
///      `pub mod console_attach;` the pure part of the Windows console attachment (`src/console_attach.rs`):
///           which standard handles need repair after attaching to the parent console, and the startup report.
/// Why:
///       The Windows executable is a GUI-subsystem program (no console window on double-click) that still has to print
///           in a terminal; the decisions are pure, so they are tested on every host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as consoleAttach from "./consoleAttach";
/// ```
pub mod console_attach;

/// What:
///      `#[cfg(windows)] pub mod console_attach_win32;` the Win32 calls of the console attachment
///           (`src/console_attach_win32.rs`).
/// Why:
///       `AttachConsole`, `GetStdHandle` and `SetStdHandle` exist on Windows only, so the module is gated out of the
///           Linux and macOS builds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.platform === "win32") await import("./consoleAttachWin32");
/// ```
#[cfg(windows)]
pub mod console_attach_win32;

/// What:
///      `pub mod console_color;` whether the Windows build writes ANSI colors on stdout and stderr (`src/console_color.rs`).
/// Why:
///       A Windows console without VT processing shows escape sequences as text and a file keeps them, so the logger and
///           clap write colors only to a console that interprets them; the decisions are pure and tested on every host.
pub mod console_color;

/// What:
///      `pub mod startup_failure;` the text and the decisions of the startup failure report (`src/startup_failure.rs`).
/// Why:
///       A double-click start on Windows has no console, so a failed start shows its text in a message box; the text
///           and the decisions are pure and tested on every host.
pub mod startup_failure;

/// What:
///      `#[cfg(windows)] pub mod startup_failure_win32;` the message box and the panic hook of the startup failure report
///           (`src/startup_failure_win32.rs`).
/// Why:
///       `MessageBoxW` exists on Windows only, so the module is gated out of the Linux and macOS builds.
#[cfg(windows)]
pub mod startup_failure_win32;

/// What:
///      `pub mod template_model;` the supporting-text template vocabulary (`src/template_model.rs`):
///  fields,
///  the two templates and their defaults,
///  the function table,
///  track value sources,
///  and results.
/// Why:
///       Every other template module reads these shared definitions,
///  and `TrackFields` is the one track value type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as templateModel from "./template_model";
/// ```
pub mod template_model;

/// What:
///      `pub mod template_format;` the JavaScript number text,
///  `tf` formatting,
///  and code-point cuts
///  (`src/template_format.rs`).
/// Why:
///       `tf` and `tc` must match the reference's number rules exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as templateFormat from "./template_format";
/// ```
pub mod template_format;

/// What:
///      `pub mod template_tokens;` splits one formula's text into tokens (`src/template_tokens.rs`).
/// Why:
///       Token offsets are UTF-16 positions in the whole template,
///  as the editor needs them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as templateTokens from "./template_tokens";
/// ```
pub mod template_tokens;

/// What:
///      `pub mod template_parse;` builds formula trees and splits a template into parts
///  (`src/template_parse.rs`).
/// Why:
///       Evaluation and help walk the same trees and parts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as templateParse from "./template_parse";
/// ```
pub mod template_parse;

/// What:
///      `pub mod template_plain;` writes the editor's mistake lines as plain sentences
///  (`src/template_plain.rs`).
/// Why:
///       The engine keeps its short notes,
///  and the editor shows one plain sentence per problem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as templatePlain from "./template_plain";
/// ```
pub mod template_plain;

/// What:
///      `pub mod template_eval;` shows a template's text for one track,
///  or the mistakes that refuse it
///  (`src/template_eval.rs`).
/// Why:
///       Evaluation is the only place that reads field values and runs functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as templateEval from "./template_eval";
/// ```
pub mod template_eval;

/// What:
///      `pub mod template_help;` finds the call and argument under the caret (`src/template_help.rs`).
/// Why:
///       The typing help is a pure function of the text and the caret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as templateHelp from "./template_help";
/// ```
pub mod template_help;

/// What:
///      `pub mod template_editor;` the editor values,
///  constants,
///  and shared helpers (`src/template_editor.rs`).
/// Why:
///       The editor state imports them from one module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as templateEditor from "./template_editor";
/// ```
pub mod template_editor;

/// What:
///      `pub mod template_editor_state;` the editor state and its immutable transitions
///  (`src/template_editor_state.rs`).
/// Why:
///       Its fields stay private so the preview track list is never empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as templateEditorState from "./template_editor_state";
/// ```
pub mod template_editor_state;

/// What:
///      `#[cfg(test)] mod template_fixtures;` the shared track and editor fixtures (`src/template_fixtures.rs`).
/// Why:
///       The module is compiled only under `cargo test`,
///  so the fixtures never reach the binary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // templateFixtures.ts, imported only by the test runner
/// ```
#[cfg(test)]
mod template_fixtures;

/// What:
///      `pub mod track_supporting_line;` builds the supporting-line function from a track rows template
///  (`src/track_supporting_line.rs`).
/// Why:
///       `track_list_entries` and `build_player_content` take this function,
///  so the template engine
///  is reached through one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as trackSupportingLine from "./track_supporting_line";
/// ```
pub mod track_supporting_line;

/// What:
///      `pub mod track;` the one-track value (`src/track.rs`) with its URI and display path.
/// Why:
///       Folder and search modules group tracks by display path,
///  so they share this record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as track from "./track";
/// ```
pub mod track;

/// What:
///      `pub mod display_path;` joins and sanitizes display paths (`src/display_path.rs`).
/// Why:
///       Folder rows and search results both show the same `/`-joined path text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as displayPath from "./display_path";
/// ```
pub mod display_path;

/// What:
///      `pub mod folder_rail;` the folder rail sections and collation keys (`src/folder_rail.rs`).
/// Why:
///       The folder picker and the search page both sort folder names with the same collation rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as folderRail from "./folder_rail";
/// ```
pub mod folder_rail;

/// What:
///      `pub mod folder_index;` groups the library into folders and their flat track lists (`src/folder_index.rs`).
/// Why:
///       The folder picker,
///  the search page,
///  and the player content all read the same folder layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as folderIndex from "./folder_index";
/// ```
pub mod folder_index;

/// What:
///      `pub mod track_entries;` the flat track rows for one folder,
///  with titles and supporting lines
///  (`src/track_entries.rs`).
/// Why:
///       The pane draws rows in folder order,
///  and the supporting line comes from `track_supporting_line`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as trackEntries from "./track_entries";
/// ```
pub mod track_entries;

/// What:
///      `pub mod player_content;` the pure builder for the player's rail,
///  folder rows,
///  and deck text
///  (`src/player_content.rs`).
/// Why:
///       One builder derives every player surface from the same inputs,
///  so the screen updates in one step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as playerContent from "./player_content";
/// ```
pub mod player_content;

/// What:
///      `pub mod non_spacing_mark;` tests for non-spacing marks,
///  used by the search matcher
///  (`src/non_spacing_mark.rs`).
/// Why:
///       Search ignores combining marks when it compares text,
///  so the matcher needs this predicate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as nonSpacingMark from "./non_spacing_mark";
/// ```
pub mod non_spacing_mark;

/// What:
///      `pub mod search_matcher;` the case- and mark-insensitive search matcher (`src/search_matcher.rs`).
/// Why:
///       Search results and the search page both need the same match and highlight offsets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as searchMatcher from "./search_matcher";
/// ```
pub mod search_matcher;

/// What:
///      `pub mod search_results;` the ranked search results and their state (`src/search_results.rs`).
/// Why:
///       The search page renders these results,
///  and their order and limit are fixed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as searchResults from "./search_results";
/// ```
pub mod search_results;

/// What:
///      `pub mod search_index;` the in-memory Search index (`src/search_index.rs`).
/// Why:
///       D112 moves Search onto normalized names in memory with first-character buckets,
///  so a keystroke tests a bucket instead of the whole library.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as searchIndex from "./search_index";
/// ```
pub mod search_index;

/// What:
///      `mod search_index_text;` the text arena and title normalizer of the index (`src/search_index_text.rs`).
/// Why:
///       One buffer per list of names keeps a million names to a few allocations.
mod search_index_text;

/// What:
///      `mod search_index_batch;` batch preparation for the index (`src/search_index_batch.rs`).
/// Why:
///       Normalization and key extraction run before the write lock is taken.
mod search_index_batch;

/// What:
///      `mod search_index_rank;` the tier test and best-five-hundred selection (`src/search_index_rank.rs`).
/// Why:
///       Ranking needs integer keys first and string keys only for ties.
mod search_index_rank;

/// What:
///      `mod search_index_top;` the single-character answer cache (`src/search_index_top.rs`).
/// Why:
///       A one-character query matches a large share of the library,
///  so its answer is kept and merged with later tracks.
mod search_index_top;

/// What:
///      `mod search_index_query;` query answering for the index (`src/search_index_query.rs`).
/// Why:
///       Keeps `SearchIndex::search` and row building apart from the index storage.
mod search_index_query;

/// What:
///      `pub mod search_service;` the Search worker thread, append path and refresh signal (`src/search_service.rs`).
/// Why:
///       D111 and D112 need queries answered off the window thread while a scan appends tracks under a short lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as searchService from "./search_service";
/// ```
pub mod search_service;

/// What:
///      `mod search_worker;` the threads behind the Search service (`src/search_worker.rs`).
/// Why:
///       Keeps the query worker and the warm-up thread apart from the public handle.
mod search_worker;

/// What:
///      `mod search_stream;` the thread that appends streamed scan batches (`src/search_stream.rs`).
/// Why:
///       The engine thread only hands names over; preparing and pushing them happen here.
mod search_stream;

/// What:
///      `pub mod search_feed;` feeding a whole library into the Search service (`src/search_feed.rs`).
/// Why:
///       A library that arrives as one list must reach the index in batches, off the window thread.
pub mod search_feed;

/// What:
///      `pub(crate) mod scan_stream;` the folder walk that reports tracks while it walks (`src/scan_stream.rs`).
/// Why:
///       D111: Search must work while the library is still being scanned, so the walk hands out batches as it goes.
pub(crate) mod scan_stream;

/// Synthetic libraries for the scale tests (`src/scale_library_tests.rs`), compiled for tests only.
#[cfg(test)]
#[path = "scale_library_tests.rs"]
mod scale_library_tests;

/// Search index scale tests (`src/search_scale_tests.rs`): 1,000 to 1,000,000 tracks.
#[cfg(test)]
#[path = "search_scale_tests.rs"]
mod search_scale_tests;

/// What:
///      `pub mod search_page_model;` the search page's view model (`src/search_page_model.rs`).
/// Why:
///       One pure builder turns the query and results into the rows and states the page draws.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as searchPageModel from "./search_page_model";
/// ```
pub mod search_page_model;

/// What:
///      `pub mod keymap;` the keyboard shortcut map (`src/keymap.rs`).
/// Why:
///       Shortcuts are data,
///  so the platform-specific bindings stay in one tested module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as keymap from "./keymap";
/// ```
pub mod keymap;

/// What:
///      `pub mod keymap_bindings;` the D106 binding table (`src/keymap_bindings.rs`).
/// Why:
///       One table of chords feeds the classifier,
///  the key hint badges,
///  the tooltips and the menu rows,
///  so they cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as keymapBindings from "./keymap_bindings";
/// ```
pub mod keymap_bindings;

/// What:
///      `pub mod double_shift;` the double tap of Shift detector (`src/double_shift.rs`).
/// Why:
///       Search opens on two quick taps of Shift (D106),
///  and the timing rule is pure logic with injected time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as doubleShift from "./double_shift";
/// ```
pub mod double_shift;

/// What:
///      `pub mod key_hints;` the key hint badge state and texts (`src/key_hints.rs`).
/// Why:
///       Badges appear after the modifiers of a shortcut have been held for 300 ms (D106),
///  and the timing and narrowing rules are pure logic with injected time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as keyHints from "./key_hints";
/// ```
pub mod key_hints;

/// What:
///      `pub mod list_focus;` the focus ring position inside the folder and track lists (`src/list_focus.rs`).
/// Why:
///       Arrow keys,
///  Home,
///  End,
///  the page keys and letters move the ring by pure rules that are unit-tested.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as listFocus from "./list_focus";
/// ```
pub mod list_focus;

/// What:
///      `pub mod first_run;` the first-run walkthrough state (`src/first_run.rs`).
/// Why:
///       The walkthrough's steps and their persistence are pure logic,
///  so they are unit-tested.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as firstRun from "./first_run";
/// ```
pub mod first_run;

/// What:
///      `pub mod track_menu;` the per-track action menu model (`src/track_menu.rs`).
/// Why:
///       The menu's actions and their enabled states are decided here,
///  not in the UI code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as trackMenu from "./track_menu";
/// ```
pub mod track_menu;

/// What:
///      `pub mod feedback;` the transient feedback message queue (`src/feedback.rs`).
/// Why:
///       Messages expire by time,
///  and the queue's order and expiry rules are pure logic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as feedback from "./feedback";
/// ```
pub mod feedback;

/// What: `pub mod track_measure;` the per-track length and true peak, and their formatting.
/// Why: the rows and the deck read these values through the template fields.
pub mod track_measure;

/// What: `pub(crate) mod track_measure_page;` the in-place measure writer and the page lookup.
/// Why: they live beside `track_measure`, which re-exports them, to keep that file short.
pub(crate) mod track_measure_page;

/// What: `pub mod duration_probe;` reads a file length from its container header.
/// Why: rows show lengths before the slow true-peak measurement finishes.
pub mod duration_probe;

/// What: `pub mod measure_peek;` reads each track's cached decision and header length without decoding.
/// Why: the sweep reports lengths and cached peaks as soon as a queue loads.
pub mod measure_peek;

/// What: `pub mod change_set;` what the file watcher saw change: some paths, or everything (D113).
/// Why: the refresh looks at the named paths instead of walking the library.
pub mod change_set;

/// What: `pub(crate) mod walk_order;` the folder walk's order as a comparison, for binary search in the queue.
/// Why: deciding whether a named path is listed must cost a few comparisons at a million tracks.
pub(crate) mod walk_order;

/// What: `pub(crate) mod disk_view;` the questions the refresh planner asks the disk.
/// Why: the planner is tested at a million tracks without a million files.
pub(crate) mod disk_view;

/// What: `pub(crate) mod change_plan;` from named paths to queue edits.
/// Why: a replaced file must cost lookups for that file only.
pub(crate) mod change_plan;

/// What: `pub(crate) mod refresh_coalescer;` at most one refresh sweep runs, with one merged follow-up.
/// Why: overlapping sweeps would stat the same files twice.
pub(crate) mod refresh_coalescer;

/// What: `pub mod controller_refresh;` the controller side of refreshing the library when the watcher reports changes.
/// Why: the cost of a refresh follows what changed.
pub mod controller_refresh;

/// What: `refresh_test_support`, shared helpers of the refresh tests (test builds only).
/// Why: the planner, coalescer and controller tests share a disposable folder, a random generator and an in-memory disk.
#[cfg(test)]
pub(crate) mod refresh_test_support;

/// What: `refresh_scale_support`, the million-track library of the refresh tests (test builds only).
/// Why: the planner and the controller are measured at normal operation's largest size.
#[cfg(test)]
pub(crate) mod refresh_scale_support;

/// What: `refresh_scale_tests`, one refresh event at a million tracks (test builds only).
/// Why: 1,000,000 tracks is normal operation, so this test belongs to the normal suite.
#[cfg(test)]
#[path = "refresh_scale_tests.rs"]
mod refresh_scale_tests;

/// What: `pub mod controller_reload;` loads the selected track again where it was when its file was replaced.
/// Why: a replaced file that plays keeps its place, peak and gain current without a toast.
pub mod controller_reload;

/// What: `pub mod sweep_report;` the reports a background sweep sends to the engine.
/// Why: the controller needs the generation of each report to drop stale ones.
pub mod sweep_report;

/// What: `pub mod color_scheme;` the environment override of the window color scheme.
/// Why: verification runs need a fixed light or dark window.
pub mod color_scheme;

/// What: `pub mod oklch;` OKLCH color math, gamut mapping and WCAG contrast (D108).
/// Why: every derived theme color is computed there, in one pure place.
pub mod oklch;

/// What: `pub mod accent_override;` the environment override of the theme root color.
/// Why: renders and verification runs need a deterministic accent.
pub mod accent_override;

/// What: `pub mod theme_roles;` the theme roles derived from one root color (A3, D108).
/// Why: the accent-following palette and its contrast guarantee are pure logic that is unit-tested.
pub mod theme_roles;

/// What: `pub mod controller_measures;` the controller side of the per-track measures.
/// Why: sweep reports are filtered by queue generation there, before they reach the window.
pub mod controller_measures;

/// What:
///      `pub mod scan_indicator;` the library scan progress indicator model (`src/scan_indicator.rs`).
/// Why:
///       The indicator's text and visibility follow the scan state,
///  which is pure logic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as scanIndicator from "./scan_indicator";
/// ```
pub mod scan_indicator;

/// What: `pub mod sweep_gate;` the pause gate of the background true-peak sweep (`src/sweep_gate.rs`).
/// Why: the analysis indicator's Pause and Resume stop and restart the sweep workers through it (D107).
pub mod sweep_gate;

/// What: `pub mod analysis_notification_logic;` the pure rules of the analysis indicator.
/// Why: what to show, when to send it and when to open, pause or close it need no bus, so they are tested without one.
pub mod analysis_notification_logic;

/// What: `pub mod analysis_notification_wire;` the D-Bus messages of the analysis indicator, Linux only.
/// Why: the Plasma job view and the freedesktop notification exist only on Linux desktops; `zbus` is a Linux-only
/// dependency.
#[cfg(target_os = "linux")]
pub mod analysis_notification_wire;

/// What: `pub mod analysis_notification;` the analysis indicator's handle and its D-Bus thread (D107).
/// Why: the system notification replaces the in-app banner; off Linux the handle does nothing.
pub mod analysis_notification;

/// What: `pub mod unplayable_notification_logic;` the wording and the told-before rule of the "cannot be played" report.
/// Why: D122 sends that report to the desktop instead of drawing it in the window; what it says and when it repeats need
/// no bus, so they are tested without one.
pub mod unplayable_notification_logic;

/// What: `pub mod unplayable_notification_wire;` the D-Bus call of the "cannot be played" notification, Linux only.
/// Why: the freedesktop notification service exists only on Linux desktops; `zbus` is a Linux-only dependency.
#[cfg(target_os = "linux")]
pub mod unplayable_notification_wire;

/// What: `pub mod unplayable_notification;` the "cannot be played" notification's handle and its D-Bus thread (D122).
/// Why: the report leaves the engine thread without waiting on the bus; off Linux the handle does nothing.
pub mod unplayable_notification;

/// What: `unplayable_notification_bus_support`, a private session bus and a recording notification service (test builds).
/// Why: the wire and thread tests of the "cannot be played" notification must never touch the user's session bus.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod unplayable_notification_bus_support;

/// What:
///      `pub mod settings_layout;` the Settings page's layout numbers (`src/settings_layout.rs`).
/// Why:
///       The pane widths and row geometry of D109 and D110 live once as pure functions,
///  and the window tests compare the real layout with them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as settingsLayout from "./settings_layout";
/// ```
pub mod settings_layout;

/// What:
///      `pub mod row_activation;` the rule for a click, Enter or Search result on a track (`src/row_activation.rs`).
/// Why:
///       Auto-play on select (D129) changes what that activation does, and the window wiring asks one pure function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as rowActivation from "./row_activation";
/// ```
pub mod row_activation;

/// Registers the unit-test scratch root, placed in the build directory instead of `/tmp`.
#[cfg(test)]
mod test_scratch;
