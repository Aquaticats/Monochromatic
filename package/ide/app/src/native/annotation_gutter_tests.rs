//! The gutter's severity letters:
//!  the worst severity of the diagnostics starting on a line,
//!  as `E`,
//!  `W`,
//!  `I`,
//!  or
//! `H` in that severity's ink,
//!  4 px before that line's own number.
//!  Room for a letter exists on every line,
//!  so text
//! and line numbers keep their x when diagnostics arrive;
//!  the number column is as wide as the file's widest line
//! number with at least three digits,
//!  so text moves only when the line count gains a digit past 999.

/// The production setter and the stamp of the displayed text.
use super::annotate::{displayed, set_annotations};
/// Rendered frames,
///  scheme switching,
///  pixel reading,
///  contrast,
///  and readers over disposable files.
use super::annotation_paint_tests::{contrast, fixture_reader, frame, near, pixel, switch};
/// The shared fixture,
///  its snapshots,
///  the text origin,
///  a click,
///  and the diagnostic builder.
use super::annotation_tests::{FIXTURE, TEXT_LEFT, TEXT_TOP, annotate, click, problem};
/// Width of the whole gutter for files of fewer than 1000 lines.
use super::sidebar_tests::GUTTER;
/// Texel comparison that accepts the GPU's one-level difference between two renders.
use super::tile_reads::same_pixels;
/// Snapshot records.
use ide_app::language::diagnostics::{DiagnosticsSnapshot, Severity, SourceGroup};
/// The toolkit's scheme values,
///  reached through its unstable re-export module as the paint tests do.
use slint::private_unstable_api::re_exports::ColorScheme;
/// What:
///  `Model` gives the window's list its `iter`;
///  the rest reads frames,
///  toolkit colors,
///  and window points.
/// Why:
///  The tests check what native code hands over,
///  what the window draws,
///  and where a click lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Model, type Color, LogicalPosition } from 'slint';
/// ```
use slint::{
    Color, LogicalPosition, Model, Rgba8Pixel, SharedPixelBuffer,
    platform::update_timers_and_animations,
};
/// Shared snapshot pointers,
///  and the bound on waiting for the first highlighting answer.
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Advance of one JetBrains Mono digit or letter at the gutter's 15 px.
const DIGIT: f32 = 9.0;
/// Space between a line number and the text.
const NUMBER_GAP: f32 = 12.0;

/// What:
///  Longest wait for the first highlighting answer:
///  a hang detector,
///  not a latency budget,
///  and the same
///       bound the Language integration tests use for any expected state (`PATIENCE` in
///       `tests/language/support.rs`).
/// Why:
///  The answer needs the syntax engine and the Rust queries,
///  built once per reader on its worker;
///  measured
///      on a loaded machine with two processors,
///  that took up to 4.2 s.
///  No product bound exists for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HIGHLIGHT_PATIENCE_MS = 20_000;
/// ```
const HIGHLIGHT_PATIENCE: Duration = Duration::from_secs(20);

/// What:
///  Advance timers until the reader shows the highlighting of its displayed revision.
/// Why:
///  The reload binding asks for highlighting on a timer and applies the answer on a later tick,
///  so a frame
///      taken right after the reader opens shows plain text,
///  and the answer can land between two frames a test
///      compares;
///  it colors the text and changes pixels the test expects to stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function untilHighlighted(reader: Reader): void { while (reader.syntaxRevision !== reader.revision) tick(); }
/// ```
fn until_highlighted(reader: &super::find_tests::Reader) {
    let start = Instant::now();
    loop {
        update_timers_and_animations();
        let current = reader.source.borrow();
        if current.syntax_revision == Some(current.document.revision()) {
            return;
        }
        drop(current);
        assert!(
            start.elapsed() < HIGHLIGHT_PATIENCE,
            "the first highlighting answer was not applied within {HIGHLIGHT_PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Window x of the left edge of the line number of source line `line` (0-based) in a file of fewer than 1000 lines.
fn number_left(line: usize) -> f32 {
    // `to_string().len()` counts the digits of the one-based line number.
    let digits = (line + 1).to_string().len();
    return TEXT_LEFT - NUMBER_GAP - DIGIT * digits as f32;
}

/// What:
///  Pixels left of line `line`'s number in its code row starting at `code_top` (the letter's place) that match
///       `ink`,
///  and the strongest contrast any of them reaches against `background`.
/// Why:
///  A letter is drawn when its ink appears before the number;
///  its legibility is measured on the same pixels.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function letterCell(shown: Frame, line: number, codeTop: number, ink: Color, background: Pixel): [number, number];
/// ```
fn letter_cell(
    shown: &SharedPixelBuffer<Rgba8Pixel>,
    line: usize,
    code_top: f32,
    ink: Color,
    background: Rgba8Pixel,
) -> (usize, f32) {
    let mut matching = 0;
    let mut strongest: f32 = 1.0;
    for dy in 0..24 {
        for x in (TEXT_LEFT - GUTTER) as usize..(number_left(line) - 1.0) as usize {
            let found = pixel(shown, x as f32, TEXT_TOP + code_top + dy as f32);
            if near(found, ink) {
                matching += 1;
            }
            strongest = strongest.max(contrast(found, background));
        }
    }
    return (matching, strongest);
}

/// What:
///  The leftmost window column at or right of `from` with ink different from `background` in the code row
///       starting at window y `top`,
///  or nothing.
/// Why:
///  The first ink right of the letter is the line number,
///  and right of the gutter the text;
///  comparing them
///      before and after letters appear shows that nothing moved sideways.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstInk(shown: Frame, top: number, from: number, background: Pixel): number | undefined;
/// ```
fn first_ink(
    shown: &SharedPixelBuffer<Rgba8Pixel>,
    top: f32,
    from: f32,
    background: Rgba8Pixel,
) -> Option<usize> {
    for x in from as usize..(TEXT_LEFT + 80.0) as usize {
        for dy in 4..20 {
            if contrast(pixel(shown, x as f32, top + dy as f32), background) > 1.5 {
                // `Some(x)` is the first column with ink.
                return Some(x);
            }
        }
    }
    return None;
}

/// What:
///  The runs of window columns with any ink,
///  antialiased edges included,
///  from the gutter's left edge to the
///       text in the code row starting at window y `top`;
///  each run is its first and last column.
/// Why:
///  The blank columns between the letter's run and the number's first run are the gap the user sees,
///  and the
///      blank columns between two digits of a number are what that gap is compared with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inkRuns(shown: Frame, top: number, background: Pixel): Array<[number, number]>;
/// ```
fn ink_runs(
    shown: &SharedPixelBuffer<Rgba8Pixel>,
    top: f32,
    background: Rgba8Pixel,
) -> Vec<(usize, usize)> {
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for x in (TEXT_LEFT - GUTTER) as usize..(TEXT_LEFT - NUMBER_GAP) as usize {
        // `any` asks whether some pixel of the column differs visibly from the background.
        let mut inked = false;
        for dy in 2..22 {
            if contrast(pixel(shown, x as f32, top + dy as f32), background) > 1.05 {
                inked = true;
                break;
            }
        }
        if !inked {
            continue;
        }
        // `last_mut` lends the latest run, which grows when this column continues it.
        match runs.last_mut() {
            Some(run) if run.1 + 1 == x => run.1 = x,
            _ => runs.push((x, x)),
        }
    }
    return runs;
}

/// `count` numbered lines,
///  `line NNNN: value`.
fn numbered(count: usize) -> String {
    let mut text = String::new();
    for index in 0..count {
        text.push_str(&format!("line {index:04}: value\n"));
    }
    return text;
}

/// Logical x of the caret before character `position` of the first line,
///  measured from where text starts.
fn first_row_caret_x(reader: &super::find_tests::Reader, position: usize) -> f32 {
    let reading = reader.source.borrow();
    let view = reading.shaped.as_ref().expect("shaped source");
    return view.rows[0].caret_x(position, view.viewport.scale);
}

/// An error diagnostic on `value`,
///  columns 11 to 16,
///  of each of `lines` of a [`numbered`] file.
fn errors_on(reader: &super::find_tests::Reader, lines: &[usize]) {
    let stamp = displayed(&reader.source.borrow());
    let mut items = Vec::new();
    for line in lines {
        // Each numbered line is 17 characters with its terminator; the range is `value`, from column 11, so its
        // squiggle stays right of the line's first ten characters.
        items.push(problem(
            line * 17 + 11,
            line * 17 + 16,
            Severity::Error,
            "E1",
            "wrong",
        ));
    }
    let snapshot = DiagnosticsSnapshot {
        stamp,
        groups: vec![SourceGroup {
            source: "rustc".to_string(),
            items,
        }],
    };
    // `None` installs no hints; `Arc::new` shares the snapshot.
    set_annotations(
        &reader.window,
        &reader.source,
        None,
        Some(Arc::new(snapshot)),
    );
    update_timers_and_animations();
}

/// In both schemes the gutter shows `E` beside a line with an error and a milder problem,
///  `W` beside a line whose
/// worst problem is a warning (also when a warning range continues onto the next line,
///  which shows nothing for
/// it),
///  `I` beside a line with information,
///  nothing beside a clean line,
///  and `H` once a hint is the only problem;
/// each letter is in its severity's ink at least 4.5:1 against the background.
///  Line numbers and text keep their x.
#[test]
fn gutter_letters_show_the_worst_severity_in_front_of_the_line_number() {
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let (_directory, reader) = fixture_reader(FIXTURE);
        let window = &reader.window;
        switch(window, scheme);
        update_timers_and_animations();
        let plain = frame(window);
        // The clean last line's gutter shows the background left of its number in both frames.
        let empty_top = reader.source.borrow().row_map.code_top(3);
        let background = pixel(
            &plain,
            TEXT_LEFT - GUTTER + 8.0,
            TEXT_TOP + empty_top + 12.0,
        );
        let number_before = first_ink(&plain, TEXT_TOP, number_left(0) - 1.0, background);
        let text_before = first_ink(&plain, TEXT_TOP, TEXT_LEFT - 4.0, background);
        annotate(&reader);
        update_timers_and_animations();
        let shown = frame(window);
        // `iter().collect()` copies the window's list into a plain list.
        let marks: Vec<i32> = window.get_row_marks().iter().collect();
        assert_eq!(
            &marks[..4],
            [1, 2, 3, 0],
            "{scheme:?}: marks handed to the window"
        );
        assert_eq!(
            window.get_gutter_width(),
            GUTTER,
            "{scheme:?}: the gutter's width, measured from the 15 px JetBrains Mono letter and digits"
        );
        let mut tops: Vec<f32> = Vec::new();
        for line in 0..4 {
            tops.push(reader.source.borrow().row_map.code_top(line));
        }
        let inks = [
            window.get_error_ink(),
            window.get_warning_ink(),
            window.get_information_ink(),
        ];
        for (line, ink) in inks.iter().enumerate() {
            let (matching, strongest) = letter_cell(&shown, line, tops[line], *ink, background);
            println!(
                "gutter {scheme:?}: line {line} letter pixels in its ink {matching}, contrast {strongest:.2}"
            );
            assert!(
                matching > 5,
                "{scheme:?}: line {line} shows no letter in its severity's ink"
            );
            assert!(
                strongest >= 4.5,
                "{scheme:?}: line {line}'s letter reaches only {strongest:.2}:1"
            );
        }
        let (_, clean) = letter_cell(&shown, 3, tops[3], inks[0], background);
        assert!(clean < 1.05, "{scheme:?}: a clean line shows a letter");
        assert_eq!(
            first_ink(&shown, TEXT_TOP + tops[0], number_left(0) - 1.0, background),
            number_before,
            "{scheme:?}: the line number moved sideways when diagnostics arrived"
        );
        assert_eq!(
            first_ink(&shown, TEXT_TOP + tops[0], TEXT_LEFT - 4.0, background),
            text_before,
            "{scheme:?}: text moved sideways when diagnostics arrived"
        );
        // A hint as the only problem, on the last line, shows `H` there and nothing elsewhere.
        let stamp = displayed(&reader.source.borrow());
        let only_hint = DiagnosticsSnapshot {
            stamp,
            groups: vec![SourceGroup {
                source: "rustc".to_string(),
                items: vec![problem(55, 55, Severity::Hint, "H2", "a note")],
            }],
        };
        set_annotations(window, &reader.source, None, Some(Arc::new(only_hint)));
        update_timers_and_animations();
        let hint_marks: Vec<i32> = window.get_row_marks().iter().collect();
        assert_eq!(&hint_marks[..4], [0, 0, 0, 4], "{scheme:?}: a hint's mark");
        let hinted = frame(window);
        let last_top = reader.source.borrow().row_map.code_top(3);
        let (matching, _) = letter_cell(
            &hinted,
            3,
            last_top,
            window.get_suggestion_ink(),
            background,
        );
        assert!(
            matching > 5,
            "{scheme:?}: the hint letter is not in the hint severity's ink"
        );
    }
}

/// The letter stands the same distance before the numbers 1,
///  10,
///  and 100,
///  and a line's number and text keep every
/// pixel when its letter appears.
#[test]
fn the_letter_stands_the_same_gap_before_one_two_and_three_digit_numbers() {
    let (_directory, reader) = fixture_reader(&numbered(300));
    let window = &reader.window;
    // Both frames show the highlighted text, so they differ only by the diagnostics.
    until_highlighted(&reader);
    let plain = frame(window);
    let background = pixel(&plain, TEXT_LEFT - GUTTER + 2.0, TEXT_TOP + 12.0);
    errors_on(&reader, &[0, 9, 99]);
    let error = window.get_error_ink();
    let shown = frame(window);
    let mut runs = Vec::new();
    for line in [0, 9] {
        let before = TEXT_TOP + line as f32 * 24.0;
        let after = TEXT_TOP + reader.source.borrow().row_map.code_top(line);
        let width = shown.width() as usize;
        // The line's number and its first ten characters, at the line's own place in each frame; the squiggle
        // under `value` starts further right.
        let from = number_left(line) as usize;
        let to = (TEXT_LEFT + 90.0) as usize;
        for dy in 0..24 {
            let old_start = (before as usize + dy) * width;
            let new_start = (after as usize + dy) * width;
            assert!(
                same_pixels(
                    &plain.as_slice()[old_start + from..old_start + to],
                    &shown.as_slice()[new_start + from..new_start + to]
                ),
                "line {line}'s number or text changed when its letter appeared"
            );
        }
        let (matching, _) = letter_cell(&shown, line, after - TEXT_TOP, error, background);
        assert!(matching > 5, "line {line} shows no letter in the error ink");
        runs.push(ink_runs(&shown, after, background));
    }
    // Line 99 is below the view: scroll its code row into the view first, and read back the offset taken.
    let top = reader.source.borrow().row_map.code_top(99);
    window.set_scroll_y(-(top - 100.0));
    update_timers_and_animations();
    // A scroll re-center is built on the tile worker, so the frame is read only once it is swapped in.
    super::overscan_tile_tests::settle_tile(&reader.source);
    let row_y = TEXT_TOP + top + window.get_scroll_y();
    let scrolled = frame(window);
    let (matching, _) = letter_cell(&scrolled, 99, row_y - TEXT_TOP, error, background);
    assert!(matching > 5, "line 99 shows no letter in the error ink");
    runs.push(ink_runs(&scrolled, row_y, background));
    // The letter is the first run and the number's first digit the second; `10` has its digits in two runs.
    let mut gaps: Vec<usize> = Vec::new();
    for found in &runs {
        gaps.push(found[1].0 - found[0].1 - 1);
    }
    assert_eq!(
        runs[1].len(),
        3,
        "the gutter of line 10 holds a letter and two digits: {runs:?}"
    );
    let digits = runs[1][2].0 - runs[1][1].1 - 1;
    println!(
        "gutter: blank columns between the letter and the number for 1, 10, 100: {gaps:?}; between the digits of 10: {digits}; runs {runs:?}"
    );
    assert_eq!(
        runs[0].len(),
        2,
        "the gutter of line 1 holds a letter and one digit"
    );
    assert_eq!(
        runs[2].len(),
        4,
        "the gutter of line 100 holds a letter and three digits"
    );
    assert_eq!(
        gaps[0], gaps[1],
        "the gap before a two-digit number differs"
    );
    assert_eq!(
        gaps[0], gaps[2],
        "the gap before a three-digit number differs"
    );
    assert!(
        gaps[0] > digits && gaps[0] <= 8,
        "the letter stands {} blank columns from its number, digits {digits} apart",
        gaps[0]
    );
}

/// A file of 1000 lines has one digit more than the three every smaller file gets,
///  so its text starts one digit
/// further right;
///  a click there still lands on the character under the pointer.
///  Both files end without a line
/// terminator:
///  after a final terminator the empty line that follows gets a number of its own.
#[test]
fn the_gutter_gains_a_digit_past_999_lines_and_the_pointer_follows() {
    // `trim_end_matches` drops the final terminator.
    let (_small_directory, small) = fixture_reader(numbered(999).trim_end_matches('\n'));
    let (_large_directory, large) = fixture_reader(numbered(1000).trim_end_matches('\n'));
    update_timers_and_animations();
    assert_eq!(
        small.window.get_gutter_width(),
        GUTTER,
        "999 lines keep the three-digit gutter"
    );
    assert_eq!(
        large.window.get_gutter_width(),
        GUTTER + DIGIT,
        "a fourth digit widens the gutter by one digit"
    );
    let small_frame = frame(&small.window);
    let large_frame = frame(&large.window);
    let background = pixel(&small_frame, TEXT_LEFT - GUTTER + 2.0, TEXT_TOP + 12.0);
    let small_text =
        first_ink(&small_frame, TEXT_TOP, TEXT_LEFT - 4.0, background).expect("text of 999 lines");
    let large_text =
        first_ink(&large_frame, TEXT_TOP, TEXT_LEFT - 4.0, background).expect("text of 1000 lines");
    assert_eq!(
        large_text - small_text,
        DIGIT as usize,
        "a fourth digit widens the gutter by one digit"
    );
    let caret_x = first_row_caret_x(&large, 4);
    click(
        &large.window,
        LogicalPosition::new(TEXT_LEFT + DIGIT + caret_x + 1.0, TEXT_TOP + 12.0),
    );
    update_timers_and_animations();
    assert_eq!(
        large.source.borrow().document.position().head,
        4,
        "a click beside a four-digit gutter did not land on its character"
    );
}
