//! Which rendered pixels may move when virtual rows appear,
//!  change,
//!  or vanish,
//!  and which may not.
//!
//! Hints and diagnostic messages take rows above their code line,
//!  so they move text.
//!  These tests pin the
//! limits on real window frames:
//!  rows arriving above the view move nothing visible;
//!  rows arriving in view move
//! only what lies beneath them,
//!  by exactly their height;
//!  an external reload keeps the rows' space until
//! annotations of the new text return;
//!  and scrolling across an annotated line is rigid,
//!  pixel by pixel.

/// The window whose scroll the tests drive.
use super::AppWindow;
/// The production setter and the stamp of the displayed text.
use super::annotate::{displayed, set_annotations};
/// The frame reader.
use super::annotation_paint_tests::frame;
/// Snapshot builders and the text origin shared with the behavior tests.
use super::annotation_tests::{TEXT_LEFT, TEXT_TOP, hint, problem};
/// The complete reader fixture,
///  its opener,
///  and its bounded wait.
use super::find_tests::{Reader, reader};
/// The clock the scrolling rule reads,
/// which a test holds and moves.
use super::rows::hold_clock;
/// Two renders of one scene differ by one level in a few texels on the GPU.
use super::tile_reads::{same_pixels, same_texel};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{Facts, Waiting};
/// Snapshot records exactly as the Language module builds them.
use ide_app::language::{
    diagnostics::{Diagnostic, DiagnosticsSnapshot, Severity, SourceGroup},
    hints::{HintsSnapshot, InlayHint},
};
/// The vertical mapping and the heights virtual rows are built from.
use ide_app::{
    minimap::width_for,
    row_map::RowMap,
    virtual_row::{BLOCK_GAP, ROW_HEIGHT},
};
/// What:
///  Frames and pixels read back from the rendered window;
///  `update_timers_and_animations` runs the
///       toolkit's pending change handlers and timers.
/// Why:
///  Every assertion here is about rendered pixels.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Rgba8Pixel, type SharedPixelBuffer } from 'slint';
/// ```
use slint::{Rgba8Pixel, SharedPixelBuffer, platform::update_timers_and_animations};
/// Disposable fixtures and shared snapshot pointers.
use std::{fs, sync::Arc, time::Duration};

/// More than the 200 ms of stillness after which rows may move the view;
/// a test moves its held clock by this.
const QUIET_PASSED: Duration = Duration::from_millis(250);

/// Longer than the 200 ms quiet time,
/// in real time.
const STALL: Duration = Duration::from_millis(300);

/// Characters per line of the numbered fixture:
///  `line NNN: value` and its terminator.
const LINE: usize = 16;
/// The gutter,
///  severity letters and line numbers,
///  starts this far left of the text.
const GUTTER: f32 = super::sidebar_tests::GUTTER;
/// Advance of one line-number digit at the gutter's 15 px.
const DIGIT: f32 = 9.0;
/// Space between a line number and the text.
const NUMBER_GAP: f32 = 12.0;
/// Rows of a code row above its underline band;
///  glyphs without descenders end there.
const ABOVE_UNDERLINE: usize = 17;

/// What:
///  A reader over `text` saved as plain text in a fresh disposable project;
///  the directory lives as long
///       as the reader.
///  The pair is (directory,
///  reader).
/// Why:
///  A plain-text file gets no syntax colors,
///  so a highlighting answer arriving between two frames cannot
///      change a pixel;
///  every difference these tests see comes from virtual rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fixtureReader(text: string): [TempDir, Reader];
/// ```
fn fixture_reader(text: &str) -> (tempfile::TempDir, Reader) {
    let directory = crate::test_scratch::scratch().expect("disposable stability project");
    fs::write(directory.path().join("notes.txt"), text).expect("stability fixture");
    let opened = reader(directory.path(), "notes.txt");
    return (directory, opened);
}

/// What:
///  Let a quarter of a second pass while the toolkit's timers run.
/// Why:
///  A scroll offset the test assigns counts as the reader scrolling,
///  and rows above the view wait 200 ms
///      after the last scroll before they may move the offset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function idle(): Promise<void> { await sleep(250); }
/// ```
pub(super) fn idle() {
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_millis(250) {
        update_timers_and_animations();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// `count` numbered lines,
///  each [`LINE`] characters long with its terminator.
fn numbered(count: usize) -> String {
    let mut text = String::new();
    for index in 0..count {
        text.push_str(&format!("line {index:03}: value\n"));
    }
    return text;
}

/// Character offset of column `column` on line `line` of the numbered fixture.
fn at(line: usize, column: usize) -> usize {
    return line * LINE + column;
}

/// Install `hints` and `problems` for the text the reader displays now;
///  the hints answer for every line.
fn inject(reader: &Reader, hints: Vec<InlayHint>, problems: Vec<Diagnostic>) {
    let stamp = displayed(&reader.source.borrow());
    let lines = reader.source.borrow().document.text().len_lines();
    let hint_snapshot = Arc::new(HintsSnapshot {
        stamp,
        first_line: 0,
        last_line: lines,
        hints,
    });
    let problem_snapshot = Arc::new(DiagnosticsSnapshot {
        stamp,
        groups: vec![SourceGroup {
            source: "rustc".to_string(),
            items: problems,
        }],
    });
    // `Some(...)` hands both snapshots over as present values.
    set_annotations(
        &reader.window,
        &reader.source,
        Some(hint_snapshot),
        Some(problem_snapshot),
    );
    update_timers_and_animations();
}

/// The vertical mapping the reader paints with now.
fn map(reader: &Reader) -> RowMap {
    return reader.source.borrow().row_map.clone();
}

/// What:
///  The pixels of window row `y` from the start of the line numbers to the minimap;
///       `&[Rgba8Pixel]` lends that part of the frame.
/// Why:
///  A line is its number and its text;
///  both must move together or not at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function band(shown: Frame, y: number): Pixel[];
/// ```
fn band(shown: &SharedPixelBuffer<Rgba8Pixel>, y: f32) -> &[Rgba8Pixel] {
    let width = shown.width() as usize;
    let left = (TEXT_LEFT - GUTTER) as usize;
    // The minimap stands at the right edge of the view and moves its box with the scroll, so it is not text.
    let minimap = width_for((width - left) as f32) as usize;
    let start = y as usize * width + left;
    let end = (y as usize + 1) * width - minimap;
    return &shown.as_slice()[start..end];
}

/// What:
///  A sentence naming how many window rows of `current` are not the row below in `previous` (compared exactly,
///  from the viewport's second row to the one above the last),
///  and the first and last of them.
/// Why:
///  One lone row points at a renderer detail,
///  rows from one row down to the end point at content that changed between the two frames,
///  such as a tile that arrived;
///  the failure message tells them apart without a second run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unshiftedRows(current: Frame, previous: Frame, height: number): string;
/// ```
fn unshifted_rows(
    current: &SharedPixelBuffer<Rgba8Pixel>,
    previous: &SharedPixelBuffer<Rgba8Pixel>,
    height: f32,
) -> String {
    let mut rows = Vec::new();
    let mut largest = 0;
    let mut columns = (usize::MAX, 0);
    let mut y = TEXT_TOP + 1.0;
    while y < height - 2.0 {
        let now = band(current, y);
        let then = band(previous, y + 1.0);
        if now != then {
            rows.push(y as usize);
            for (column, (one, other)) in now.iter().zip(then.iter()).enumerate() {
                if one != other {
                    columns = (columns.0.min(column), columns.1.max(column));
                }
                largest = largest
                    .max(one.r.abs_diff(other.r))
                    .max(one.g.abs_diff(other.g))
                    .max(one.b.abs_diff(other.b))
                    .max(one.a.abs_diff(other.a));
            }
        }
        y += 1.0;
    }
    let contiguous = match (rows.first(), rows.last()) {
        (Some(first), Some(last)) => last - first + 1 == rows.len(),
        _ => true,
    };
    return format!(
        "{} rows differ exactly, the first {:?} and the last {:?}, contiguous {contiguous}, largest channel difference {largest}, columns {} to {} of the band, the differing rows {rows:?}",
        rows.len(),
        rows.first(),
        rows.last(),
        columns.0,
        columns.1
    );
}

/// What:
///  Whether two bands differ by at most one level in any channel of any pixel.
/// Why:
///  The viewport's bottom edge is a clip edge.
///  Measured on 2026-10-20 over 1256 one-row scrolls of this test:
///  the last window row differed from the same content one row higher in 123 scrolls,
///  334 pixels in all,
///  by one level at most;
///  the viewport's first row differed likewise in 5 scrolls (6 pixels);
///  every row between the two edges matched exactly in every scroll.
///  The likely cause,
///  from reading FemtoVG's fragment shader (`scissorMask` in `femtovg-0.27.0`),
///  is that the clip is a float mask multiplied into the pixels of a clipped row,
///  which need not be exactly one at the edge row;
///  that mechanism was not isolated.
///  Shifted content that moved one row is therefore compared exactly except in that last row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withinOneLevel(left: Pixel[], right: Pixel[]): boolean;
/// ```
fn within_one_level(left: &[Rgba8Pixel], right: &[Rgba8Pixel]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (first, second) in left.iter().zip(right.iter()) {
        for (one, other) in [
            (first.r, second.r),
            (first.g, second.g),
            (first.b, second.b),
            (first.a, second.a),
        ] {
            if one.abs_diff(other) > 1 {
                return false;
            }
        }
    }
    return true;
}

/// What:
///  Pixels of a band left of line `line`'s number,
///  where its severity letter stands,
///  in files of fewer than
///       1000 lines.
/// Why:
///  The letter stands 4 px before the line's own number,
///  so its place depends on the number's digit count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function letterRoom(line: number): number;
/// ```
fn letter_room(line: usize) -> usize {
    // `to_string().len()` counts the digits of the one-based line number.
    let digits = (line + 1).to_string().len();
    return (GUTTER - NUMBER_GAP - DIGIT * digits as f32) as usize;
}

/// What:
///  Check that line `line`'s code row has the same pixels in `after` as in `before`,
///  each at the place
///       its own mapping gives,
///  and answer how far the line moved.
///  `rows` limits the compared part of the row.
/// Why:
///  A line's number and text may move as a whole,
///  never change;
///  the distance is what the tests pin.
///  The
///      gutter's severity letter is left out:
///  a letter comes and goes with its line's diagnostics,
///  also
///      while a reload holds the space of rows that are not painted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moved(before: [Frame, RowMap], after: [Frame, RowMap], line: number, rows: number): number;
/// ```
fn moved(
    before: (&SharedPixelBuffer<Rgba8Pixel>, &RowMap, &str),
    after: (&SharedPixelBuffer<Rgba8Pixel>, &RowMap, &str),
    line: usize,
    rows: usize,
) -> f32 {
    let from = TEXT_TOP + before.1.code_top(line);
    let to = TEXT_TOP + after.1.code_top(line);
    // Everything left of the line's number is where its letter may stand.
    let skip = letter_room(line);
    for row in 0..rows {
        let old = &band(before.0, from + row as f32)[skip..];
        let new = &band(after.0, to + row as f32)[skip..];
        let kept = if same_pixels(old, new) {
            String::new()
        } else {
            keep_frames("moved", before.0, after.0)
        };
        assert!(
            same_pixels(old, new),
            "line {line} changed its pixels in row {row} of its code row (window rows {} and {}); {}; {kept}; \
             facts before {}; facts after {}",
            from + row as f32,
            to + row as f32,
            level_difference(old, new),
            before.2,
            after.2
        );
    }
    return to - from;
}

/// What:
///  Write two frames that a comparison found different to `target/flake-frames/`,
///  as PPM images,
///  and say where.
/// Why:
///  A difference that shows only now and then cannot be looked at again;
///  the files are the evidence of the next one.
fn keep_frames(
    name: &str,
    first: &SharedPixelBuffer<Rgba8Pixel>,
    second: &SharedPixelBuffer<Rgba8Pixel>,
) -> String {
    let directory = std::path::Path::new("target/flake-frames");
    if fs::create_dir_all(directory).is_err() {
        return "frames not kept".to_string();
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| return elapsed.as_millis());
    for (label, frame) in [("first", first), ("second", second)] {
        let mut bytes = format!("P6\n{} {}\n255\n", frame.width(), frame.height()).into_bytes();
        for pixel in frame.as_slice() {
            bytes.extend_from_slice(&[pixel.r, pixel.g, pixel.b]);
        }
        let _ = fs::write(directory.join(format!("{name}-{stamp}-{label}.ppm")), bytes);
    }
    return format!("frames kept as target/flake-frames/{name}-{stamp}-first and -second.ppm");
}

/// What:
///  The note a failure message carries about saved frames:
///  empty when the two compared parts of a row are the same pixels,
///  otherwise the frames are written to `target/flake-frames/` under `name` and the note names the files.
/// Why:
///  Every comparison of two frames in this file keeps its frames when it fails,
///  so the next failure of any of them can be looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keptIfDifferent(name: string, old: Pixel[], now: Pixel[], first: Frame, second: Frame): string;
/// ```
fn kept_if_different(
    name: &str,
    old: &[Rgba8Pixel],
    new: &[Rgba8Pixel],
    first: &SharedPixelBuffer<Rgba8Pixel>,
    second: &SharedPixelBuffer<Rgba8Pixel>,
) -> String {
    if same_pixels(old, new) {
        return String::new();
    }
    return keep_frames(name, first, second);
}

/// What:
///  A frame,
///  the vertical mapping the reader paints with now,
///  and the tile facts of that moment.
/// Why:
///  A failed comparison names the state each of its two frames was taken in.
fn shot_once(reader: &Reader) -> (SharedPixelBuffer<Rgba8Pixel>, RowMap, String) {
    return (
        frame(&reader.window),
        map(reader),
        Facts {
            window: &reader.window,
            source: &reader.source,
        }
        .to_string(),
    );
}

/// The tile and worker facts of the reader's window now.
fn facts_of(reader: &Reader) -> String {
    return Facts {
        window: &reader.window,
        source: &reader.source,
    }
    .to_string();
}

/// What:
///  `shot_once` plus a second frame of the same state,
///  compared with the first.
/// Why:
///  The rigid scroll takes hundreds of frames and keeps `shot_once`;
///  the comparisons with a few frames use this one.
fn shot(reader: &Reader) -> (SharedPixelBuffer<Rgba8Pixel>, RowMap, String) {
    let before = facts_of(reader);
    let first = frame(&reader.window);
    // A second frame of the same state tells a frame that is wrong from a state that is different:
    // when a comparison fails, the facts of both of its frames say whether each frame drew its state the same
    // way twice, and whether the tile counters moved between the two frames.
    let again = frame(&reader.window);
    let after = facts_of(reader);
    let repeatable = match difference(&first, &again) {
        Some(found) => format!("no, {found}"),
        None => "yes".to_string(),
    };
    let mut moved_between = String::new();
    if before != after {
        moved_between = format!("; facts before the two frames {before}");
    }
    return (
        first,
        map(reader),
        format!("{after} second frame of the same state equal: {repeatable}{moved_between}"),
    );
}

/// What:
///  How two runs of texels differ:
///  how many texels,
///  the largest difference of any channel,
///  and the first differing texel with both colors.
/// Why:
///  The size of a difference tells render noise (one level) from a changed picture (many levels).
fn level_difference(left: &[Rgba8Pixel], right: &[Rgba8Pixel]) -> String {
    let mut count = 0;
    let mut largest = 0;
    let mut first = None;
    let mut columns = (usize::MAX, 0);
    for (index, (one, other)) in left.iter().zip(right.iter()).enumerate() {
        let worst = one
            .r
            .abs_diff(other.r)
            .max(one.g.abs_diff(other.g))
            .max(one.b.abs_diff(other.b))
            .max(one.a.abs_diff(other.a));
        if worst > 0 {
            count += 1;
            largest = largest.max(worst);
            columns = (columns.0.min(index), columns.1.max(index));
            if first.is_none() {
                first = Some((index, *one, *other));
            }
        }
    }
    return format!(
        "{count} of {} texels differ, by {largest} levels at most, in columns {} to {} of the compared part, the first {first:?}",
        left.len(),
        columns.0,
        columns.1
    );
}

/// What:
///  The first pixel of the source view at which two frames differ,
///  with both colors,
///  or nothing when the
///       source view is the same in both;
///  `Option<String>` is that description or nothing.
/// Why:
///  A failing stability assertion must say where the frame changed,
///  not only that it did.
///  The project
///      tree beside the source view selects the opened file on its own schedule and is not compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function difference(first: Frame, second: Frame): string | undefined;
/// ```
fn difference(
    first: &SharedPixelBuffer<Rgba8Pixel>,
    second: &SharedPixelBuffer<Rgba8Pixel>,
) -> Option<String> {
    let mut y = TEXT_TOP;
    while y < first.height() as f32 {
        let before = band(first, y);
        let after = band(second, y);
        for (index, (old, new)) in before.iter().zip(after.iter()).enumerate() {
            if !same_texel(*old, *new) {
                // `Some(...)` carries the description of the first differing pixel.
                return Some(format!(
                    "pixel {},{y} changed from {old:?} to {new:?}",
                    index + (TEXT_LEFT - GUTTER) as usize
                ));
            }
        }
        y += 1.0;
    }
    return None;
}

/// Hints arriving for visible lines move only what lies beneath them,
///  by exactly their block's height,
///  and
/// diagnostics arriving later do the same again;
///  nothing above the first changed line moves,
///  and the caret
/// stays on its line.
#[test]
fn rows_arriving_in_view_move_only_lines_beneath_them() {
    let (_directory, reader) = fixture_reader(&numbered(14));
    let window = &reader.window;
    update_timers_and_animations();
    let plain = shot(&reader);
    let caret = (window.get_caret_x(), window.get_caret_y());
    // Hints arrive first: one above line 3 and one above line 6.
    let hints = vec![hint(at(3, 8), ": u32"), hint(at(6, 4), "n:")];
    inject(&reader, hints.clone(), Vec::new());
    let hinted = shot(&reader);
    let row = BLOCK_GAP + ROW_HEIGHT;
    let mut movement = Vec::new();
    for line in 0..14 {
        movement.push(moved(
            (&plain.0, &plain.1, &plain.2),
            (&hinted.0, &hinted.1, &hinted.2),
            line,
            24,
        ));
    }
    println!("movement when hints arrive for lines 3 and 6: {movement:?}");
    assert_eq!(
        movement[..3],
        [0.0, 0.0, 0.0],
        "a line above the first hint moved"
    );
    assert_eq!(movement[3..6], [row, row, row]);
    for distance in &movement[6..] {
        assert_eq!(*distance, 2.0 * row);
    }
    // Everything above the first block is the very same pixels, not only the code rows.
    let first_block = hinted.1.block_top(3);
    let mut y = TEXT_TOP;
    while y < TEXT_TOP + first_block {
        assert!(
            same_pixels(band(&plain.0, y), band(&hinted.0, y)),
            "a pixel above the first changed line moved in window row {y}; {}; {}; facts before {}; facts after {}",
            level_difference(band(&plain.0, y), band(&hinted.0, y)),
            kept_if_different(
                "above-hints",
                band(&plain.0, y),
                band(&hinted.0, y),
                &plain.0,
                &hinted.0
            ),
            plain.2,
            hinted.2
        );
        y += 1.0;
    }
    assert_eq!((window.get_caret_x(), window.get_caret_y()), caret);
    assert_eq!(window.get_scroll_y(), 0.0);
    // Diagnostics arrive seconds later for some servers: a message on line 6, which already has a hint row,
    // and one on line 9, which has no block yet.
    let problems = vec![
        problem(at(6, 10), at(6, 15), Severity::Error, "E1", "wrong value"),
        problem(at(9, 0), at(9, 4), Severity::Warning, "W1", "unused line"),
    ];
    inject(&reader, hints, problems);
    let diagnosed = shot(&reader);
    let mut later = Vec::new();
    for line in 0..14 {
        // Underlines appear on lines 6 and 9; the part of their rows above the underline band is compared.
        let mut rows = 24;
        if line == 6 || line == 9 {
            rows = ABOVE_UNDERLINE;
        }
        later.push(moved(
            (&hinted.0, &hinted.1, &hinted.2),
            (&diagnosed.0, &diagnosed.1, &diagnosed.2),
            line,
            rows,
        ));
    }
    println!("movement when diagnostics arrive later for lines 6 and 9: {later:?}");
    assert_eq!(later[..6], [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    assert_eq!(
        later[6..9],
        [ROW_HEIGHT, ROW_HEIGHT, ROW_HEIGHT],
        "a message joining a hint row adds one row, no second gap"
    );
    for distance in &later[9..] {
        assert_eq!(*distance, ROW_HEIGHT + row);
    }
    let first_changed = diagnosed.1.block_top(6);
    let mut above = TEXT_TOP;
    while above < TEXT_TOP + first_changed {
        assert!(
            same_pixels(band(&hinted.0, above), band(&diagnosed.0, above)),
            "a pixel above the first changed line moved in window row {above}; {}; {}; facts before {}; facts after {}",
            level_difference(band(&hinted.0, above), band(&diagnosed.0, above)),
            kept_if_different(
                "above-diagnostics",
                band(&hinted.0, above),
                band(&diagnosed.0, above),
                &hinted.0,
                &diagnosed.0
            ),
            hinted.2,
            diagnosed.2
        );
        above += 1.0;
    }
}

/// Rows arriving,
///  changing,
///  and vanishing above the first visible code row move no visible pixel and repaint
/// nothing:
///  the scroll offset follows them.
///  At the very top of the text the view stays at the top instead.
#[test]
fn rows_arriving_above_the_view_move_no_visible_pixel() {
    let (_directory, reader) = fixture_reader(&numbered(200));
    let window = &reader.window;
    // Line 100 is cut by the top edge, seven pixels in.
    let offset = 100.0 * 24.0 + 7.0;
    window.set_scroll_y(-offset);
    update_timers_and_animations();
    // The scroll re-centers the tile on the worker; the frames are compared only once it is swapped in.
    super::overscan_tile_tests::settle_tile(&reader.source);
    let before = frame(window);
    // The overscan tile may repaint when a diagnostic inside it changes; only visible pixels are asserted here.
    let row = BLOCK_GAP + ROW_HEIGHT;
    // Above the materialized lines: hints on lines 10 and 50 and an error on line 97; far below the view: a
    // hint on line 180.
    let hints = vec![
        hint(at(10, 4), "a:"),
        hint(at(50, 4), "b:"),
        hint(at(180, 4), "c:"),
    ];
    let error = vec![problem(
        at(97, 0),
        at(97, 4),
        Severity::Error,
        "E1",
        "wrong",
    )];
    // The test's own scroll counts as the reader scrolling; rows above the view wait until that is over.
    idle();
    inject(&reader, hints, error.clone());
    assert_eq!(
        -window.get_scroll_y(),
        offset + 3.0 * row,
        "the offset did not follow the rows that arrived above the view"
    );
    assert_eq!(
        difference(&before, &frame(window)),
        None,
        "a visible pixel moved when rows arrived above the view"
    );
    // A row above the view vanishes: again nothing visible moves.
    inject(
        &reader,
        vec![hint(at(50, 4), "b:"), hint(at(180, 4), "c:")],
        error,
    );
    assert_eq!(-window.get_scroll_y(), offset + 2.0 * row);
    assert_eq!(
        difference(&before, &frame(window)),
        None,
        "a visible pixel moved when a row vanished above the view"
    );
    // Line 99 is materialized as overscan but lies above the top edge: its rows repaint the tile, and still
    // no visible pixel moves.
    let both = vec![
        problem(at(97, 0), at(97, 4), Severity::Error, "E1", "wrong"),
        problem(at(99, 0), at(99, 4), Severity::Warning, "W1", "odd"),
    ];
    inject(
        &reader,
        vec![hint(at(50, 4), "b:"), hint(at(180, 4), "c:")],
        both,
    );
    assert_eq!(-window.get_scroll_y(), offset + 3.0 * row);
    assert_eq!(
        difference(&before, &frame(window)),
        None,
        "a visible pixel moved when rows arrived for the line just above the view"
    );
    // Positive control: a row for a visible line does change the frame.
    inject(&reader, vec![hint(at(103, 4), "d:")], Vec::new());
    assert!(
        difference(&before, &frame(window)).is_some(),
        "a row in view changed nothing (positive control)"
    );
    // At the very top the view stays at the top: rows for the first line push the text down.
    let (_other, top_reader) = fixture_reader(&numbered(40));
    update_timers_and_animations();
    inject(&top_reader, vec![hint(4, "first:")], Vec::new());
    assert_eq!(top_reader.window.get_scroll_y(), 0.0);
    assert_eq!(top_reader.source.borrow().row_map.code_top(0), row);
    assert_eq!(top_reader.window.get_caret_y(), row + 2.0);
}

/// Scroll one pixel away from `offset` and back,
///  as the reader's wheel would;
///  the scroll the rows must wait for has to be the latest thing that happened when they arrive.
fn scroll(window: &AppWindow, offset: f32) {
    window.set_scroll_y(-(offset + 1.0));
    update_timers_and_animations();
    window.set_scroll_y(-offset);
    update_timers_and_animations();
}

/// While the reader is scrolling,
///  rows arriving above the view wait:
///  assigning the scroll offset would cut the
/// toolkit's scroll animation short.
///  They are shown,
///  with the view kept still,
///  once scrolling has stopped.
/// Rows arriving in view never wait,
///  because they need no change of the offset.
#[test]
fn rows_above_the_view_wait_until_scrolling_has_stopped() {
    let (_directory, reader) = fixture_reader(&numbered(200));
    let window = &reader.window;
    let offset = 100.0 * 24.0 + 7.0;
    let row = BLOCK_GAP + ROW_HEIGHT;
    idle();
    window.set_scroll_y(-offset);
    update_timers_and_animations();
    // The scroll re-centers the tile on the worker; the frames are compared only once it is swapped in.
    super::overscan_tile_tests::settle_tile(&reader.source);
    let before = frame(window);
    let before_facts = Facts {
        window,
        source: &reader.source,
    }
    .to_string();
    // What: From here on the scrolling rule reads a clock this test holds and moves by exactly the time it means.
    // Why: Sleeping through the 200 ms quiet time and racing it in the other direction both depend on the host's
    //      load; a render that came 200 ms after a scroll step on a loaded host once showed the rows early.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let now = Date.now(); holdClock(now);
    // ```
    let mut now = std::time::Instant::now();
    hold_clock(Some(now));
    // The reader scrolls, and a hint for line 10 arrives at once.
    scroll(window, offset);
    inject(&reader, vec![hint(at(10, 4), "a:")], Vec::new());
    assert_eq!(
        -window.get_scroll_y(),
        offset,
        "rows above the view moved the offset while the reader was scrolling"
    );
    assert_eq!(reader.source.borrow().row_map.block_height(10), 0.0);
    let seen = frame(window);
    assert_eq!(
        difference(&before, &seen),
        None,
        "a frame of the same state differs; facts at the first frame {before_facts}; facts now {}",
        Facts {
            window,
            source: &reader.source
        }
    );
    // A hint for a visible line arrives together with the one above the view while the reader still scrolls.
    scroll(window, offset);
    inject(
        &reader,
        vec![hint(at(10, 4), "a:"), hint(at(104, 4), "b:")],
        Vec::new(),
    );
    assert_eq!(
        reader.source.borrow().row_map.block_height(104),
        0.0,
        "a change that also moves the offset waits as a whole"
    );
    // Scrolling has stopped: the rows are shown and nothing above line 104's block moves.
    now += QUIET_PASSED;
    hold_clock(Some(now));
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().row_map.block_height(10) == row) {
            waiting.step("rows that waited for scrolling were never shown");
        }
    }
    update_timers_and_animations();
    assert_eq!(-window.get_scroll_y(), offset + row);
    let after = frame(window);
    let shown = map(&reader);
    let mut y = TEXT_TOP;
    while y < TEXT_TOP + shown.block_top(104) - (offset + row) {
        assert!(
            same_pixels(band(&before, y), band(&after, y)),
            "a visible pixel above the new row moved in window row {y}; {}; {}; facts at the first frame {before_facts}; facts now {}",
            level_difference(band(&before, y), band(&after, y)),
            kept_if_different(
                "below-the-new-row",
                band(&before, y),
                band(&after, y),
                &before,
                &after
            ),
            Facts {
                window,
                source: &reader.source
            }
        );
        y += 1.0;
    }
    assert_eq!(shown.block_height(104), row);
    // A scroll step whose change handler has not run yet is scrolling too: rows that arrive in the render
    // right after it wait, and are shown once scrolling has stopped.
    now += QUIET_PASSED;
    hold_clock(Some(now));
    update_timers_and_animations();
    let resting = -window.get_scroll_y();
    window.set_scroll_y(-(resting + 3.0));
    inject(
        &reader,
        vec![
            hint(at(10, 4), "a:"),
            hint(at(20, 4), "c:"),
            hint(at(104, 4), "b:"),
        ],
        Vec::new(),
    );
    assert_eq!(
        map(&reader).block_height(20),
        0.0,
        "rows above the view were shown during a scroll step whose change handler had not run yet"
    );
    now += QUIET_PASSED;
    hold_clock(Some(now));
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().row_map.block_height(20) == row) {
            waiting.step("rows that waited for a scroll step were never shown");
        }
    }
}

/// What:
///  A reader that stands still for longer than the quiet time between a scroll step and the arrival of rows above the
///  view (a loaded host that does not run the window thread for 300 ms does exactly this) still has them wait,
///  because the scrolling rule reads the clock the test holds and not the real one.
/// Why:
///  This is the deterministic stand-in for the load that made `rows_above_the_view_wait_until_scrolling_has_stopped`
///  fail before the rule's clock could be held:
///  with the real clock,
///  the rows are shown at once and the offset
///  moves (the ledger in `doc/handover/ide-native-flakes.md` records the control that fails this way).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdClock(now); scroll(); await sleep(300); inject(); expect(rowsShown()).toBe(false);
/// ```
#[test]
fn a_reader_that_stalls_past_the_quiet_time_still_has_rows_wait() {
    let (_directory, reader) = fixture_reader(&numbered(200));
    let window = &reader.window;
    let offset = 100.0 * 24.0 + 7.0;
    let row = BLOCK_GAP + ROW_HEIGHT;
    idle();
    window.set_scroll_y(-offset);
    update_timers_and_animations();
    super::overscan_tile_tests::settle_tile(&reader.source);
    let mut now = std::time::Instant::now();
    hold_clock(Some(now));
    scroll(window, offset);
    // The window thread is not run for longer than the 200 ms of stillness.
    std::thread::sleep(STALL);
    inject(&reader, vec![hint(at(10, 4), "a:")], Vec::new());
    assert_eq!(
        -window.get_scroll_y(),
        offset,
        "rows above the view moved the offset after the real time passed while the held clock did not"
    );
    assert_eq!(reader.source.borrow().row_map.block_height(10), 0.0);
    // The held clock moving by the quiet time is what shows them.
    now += QUIET_PASSED;
    hold_clock(Some(now));
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().row_map.block_height(10) == row) {
            waiting.step("rows that waited for the held clock were never shown");
        }
    }
}

/// After an external change the old rows are not painted,
///  but their space stays,
///  so no line moves;
///  when the
/// same annotations return for the new text the frame is the one from before the change.
///  Nothing jumps twice.
#[test]
fn a_reload_holds_row_space_until_annotations_return() {
    let (directory, reader) = fixture_reader(&numbered(14));
    let hints = vec![hint(at(3, 8), ": u32"), hint(at(6, 4), "n:")];
    let problems = vec![problem(
        at(6, 10),
        at(6, 15),
        Severity::Error,
        "E1",
        "wrong value",
    )];
    inject(&reader, hints.clone(), problems.clone());
    let annotated = shot(&reader);
    let revision = reader.source.borrow().document.revision();
    // Line 12 changes without changing its length, so every annotated position stays valid.
    let changed = numbered(14).replace("line 012: value", "line 012: other");
    fs::write(directory.path().join("notes.txt"), &changed).expect("external change");
    {
        let mut waiting = Waiting::eventually();
        while !(reader.source.borrow().document.revision() > revision) {
            waiting.step("the external change was not reloaded");
        }
    }
    update_timers_and_animations();
    let held = shot(&reader);
    assert_eq!(held.1, annotated.1, "the reload changed where lines are");
    let mut movement = Vec::new();
    for line in 0..12 {
        // The underline of line 6 is stale and gone; the rest of every code row is unchanged.
        let mut rows = 24;
        if line == 6 {
            rows = ABOVE_UNDERLINE;
        }
        movement.push(moved(
            (&annotated.0, &annotated.1, &annotated.2),
            (&held.0, &held.1, &held.2),
            line,
            rows,
        ));
    }
    println!("movement at an external reload, rows held: {movement:?}");
    for distance in &movement {
        assert_eq!(*distance, 0.0, "a line moved at the reload");
    }
    // The held space is empty: every pixel of line 6's block is the row's own first pixel, the background.
    let block_top = TEXT_TOP + held.1.block_top(6);
    let mut y = block_top;
    while y < TEXT_TOP + held.1.code_top(6) {
        let pixels = band(&held.0, y);
        for found in &pixels[GUTTER as usize..] {
            assert!(
                *found == pixels[GUTTER as usize],
                "a stale row was painted for the new text in window row {y}"
            );
        }
        y += 1.0;
    }
    // The same annotations arrive for the new text: every line above the changed one is as before the change.
    inject(&reader, hints, problems);
    let returned = shot(&reader);
    let mut back = TEXT_TOP;
    while back < TEXT_TOP + returned.1.block_top(12) {
        assert!(
            same_pixels(band(&annotated.0, back), band(&returned.0, back)),
            "window row {back} differs after the annotations returned; {}; {}; facts before {}; facts after {}",
            level_difference(band(&annotated.0, back), band(&returned.0, back)),
            kept_if_different(
                "reload",
                band(&annotated.0, back),
                band(&returned.0, back),
                &annotated.0,
                &returned.0
            ),
            annotated.2,
            returned.2
        );
        back += 1.0;
    }
}

/// The tolerance of the compares that straddle a re-render is the measured render noise and nothing more:
///  one level in a texel passes,
///  two levels do not,
///  and a frame shifted by one row,
///  the smallest real movement,
///  does not.
/// This is the positive control of every compare in this file that accepts the noise,
///  such as `moved` and the rows above a changed line.
#[test]
fn the_noise_tolerance_tells_a_shifted_row_from_a_re_render() {
    let (_directory, reader) = fixture_reader(&numbered(14));
    update_timers_and_animations();
    let shown = frame(&reader.window);
    let top = TEXT_TOP + map(&reader).code_top(0);
    // A glyph row of the first line: the first row of its code row that differs from the row below it by more than noise.
    let mut found = None;
    for row in 0..ROW_HEIGHT as usize {
        let y = top + row as f32;
        if !same_pixels(band(&shown, y), band(&shown, y + 1.0)) {
            found = Some(y);
            break;
        }
    }
    let y = found.expect("a code row with glyph pixels");
    let original = band(&shown, y).to_vec();
    // The first pixel with room to move in both directions.
    let at = original
        .iter()
        .position(|pixel| return pixel.r >= 2 && pixel.r <= 253)
        .expect("a pixel whose red can move by two levels");
    let moved_by = |levels: i16| {
        let mut changed = original.clone();
        changed[at].r = (i16::from(changed[at].r) + levels) as u8;
        return changed;
    };
    assert!(
        same_pixels(&original, &moved_by(1)) && same_pixels(&original, &moved_by(-1)),
        "one level of render noise must be accepted"
    );
    assert!(
        !same_pixels(&original, &moved_by(2)) && !same_pixels(&original, &moved_by(-2)),
        "two levels are more than the render noise"
    );
    assert!(
        !same_pixels(&original, band(&shown, y + 1.0)),
        "a frame shifted by one row must differ by more than the render noise"
    );
}

/// Scrolling pixel by pixel across consecutive annotated lines moves the whole frame rigidly:
///  each frame is
/// the previous one shifted by one row,
///  also where an annotated line and its block leave the top of the view
/// and the materialized tile changes.
#[test]
fn scrolling_across_annotated_lines_is_rigid() {
    let (_directory, reader) = fixture_reader(&numbered(80));
    let window = &reader.window;
    let hints = vec![hint(at(5, 8), ": u32"), hint(at(6, 4), "n:")];
    let problems = vec![problem(
        at(5, 10),
        at(5, 15),
        Severity::Error,
        "E1",
        "wrong value\nexpected another",
    )];
    inject(&reader, hints, problems);
    let height = frame(window).height() as f32;
    // From a few pixels above line 5's block on for two view heights: the overscan tile re-centers only after
    // about one view height of scrolling, so the range must pass one re-center as well as the annotated lines.
    let start = map(&reader).block_top(5) as usize - 6;
    let end = start + (window.get_viewport_height() * 2.0) as usize;
    window.set_scroll_y(-(start as f32));
    update_timers_and_animations();
    let mut previous_shot = shot_once(&reader);
    let first_tile = reader.source.borrow().first;
    let mut tiles = 0;
    for offset in start + 1..=end {
        let rendered_before = reader.source.borrow().gpu.tiles_rendered();
        window.set_scroll_y(-(offset as f32));
        update_timers_and_animations();
        let current_shot = shot_once(&reader);
        let (previous, current) = (&previous_shot.0, &current_shot.0);
        // A frame that drew a new tile scene is a second render of the text: Vello's render noise applies to it.
        // Scrolling inside a tile shows the very same texture, so those frames are compared exactly.
        let re_rendered = reader.source.borrow().gpu.tiles_rendered() != rendered_before;
        let mut y = TEXT_TOP + 1.0;
        while y < height - 1.0 {
            // The previous frame's last row is the viewport's clip edge; its pixels may be a level off (see
            // `within_one_level`). Every other row is the previous frame's row shifted by one, exactly,
            // unless the frame drew a new tile.
            let shifted = band(previous, y + 1.0);
            let moved = if re_rendered {
                same_pixels(band(current, y), shifted)
            } else if y + 2.0 < height {
                band(current, y) == shifted
            } else {
                within_one_level(band(current, y), shifted)
            };
            let kept = if moved {
                String::new()
            } else {
                keep_frames("rigid", previous, current)
            };
            assert!(
                moved,
                "the frame at offset {offset} is not the previous one shifted by one row (window row {y}); {kept}; {}; \
                 re_rendered {re_rendered}, tiles_rendered before the step {rendered_before}; facts at the \
                 previous frame {}; facts at this frame {}",
                unshifted_rows(current, previous, height),
                previous_shot.2,
                current_shot.2
            );
            y += 1.0;
        }
        if reader.source.borrow().first != first_tile {
            tiles += 1;
        }
        previous_shot = current_shot;
    }
    assert!(
        tiles > 0,
        "the materialized tile never changed, so the test did not cross a tile boundary"
    );
    println!(
        "scrolled {} offsets across the blocks of lines 5 and 6 with rigid frames",
        end - start
    );
}
