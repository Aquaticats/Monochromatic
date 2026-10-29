//! Inlay placement probe:
//!  how far inline hint text would move source glyphs,
//!  tab ends,
//!  and vertical caret
//! targets,
//!  measured with the production shaper on the hints real servers returned for the inspection fixtures.
//!
//! Inline placement is simulated by splicing each label into the line at its position and shaping the result,
//! which is what a widened line looks like.
//!  Run with `--nocapture` to print the measurements the README cites.

/// Canonical source,
///  the production shaper,
///  its tab stops,
///  and the production vertical movement.
use ide_app::{
    document::{Document, ReadingPosition},
    shaped_text::TextShaper,
    tab_stop::TAB_SPACES,
    vertical_motion::vertical,
};

/// What:
///  One hint as the inspection run recorded it:
///  the line text before it,
///  its label,
///  and its right padding.
///       `&'static str` is text baked into the test binary (sibling:
///  owned `String`).
/// Why:
///  The real hints come from `inspect:language` evidence;
///  keeping them as data keeps the probe honest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordedHint = { before: string; label: string; paddingRight: boolean };
/// ```
struct RecordedHint {
    /// Line text in front of the hint,
    ///  as recorded in the event log.
    before: &'static str,
    /// Label as the server sent it.
    label: &'static str,
    /// The server asked for a space after the label.
    padding_right: bool,
}

/// What:
///  Splice the labels into `line` at their positions,
///  returning the widened text and,
///  for every source
///       character boundary of `line`,
///  its character index in the widened text.
///  `Vec<usize>` is a growable list
///       of indices (siblings:
///  fixed `[usize; N]`,
///  borrowed `&[usize]`).
/// Why:
///  Inline virtual text is shaped as part of the line;
///  the map finds each source character in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splice(line: string, hints: RecordedHint[]): [string, number[]];
/// ```
fn splice(line: &str, hints: &[RecordedHint]) -> (String, Vec<usize>) {
    // What: `.chars().count()` counts Unicode scalar values, the unit of every source position here.
    // Why: The recorded `before` text ends exactly at the hint's position on the line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const positions = hints.map(hint => [...hint.before].length);
    // ```
    let mut positions = Vec::new();
    for hint in hints {
        positions.push(hint.before.chars().count());
    }
    let mut widened = String::new();
    let mut map = Vec::new();
    let mut shown = 0;
    // What: `enumerate` pairs each character with its index.
    // Why: A label goes in front of the character whose index equals its position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // [...line].forEach((character, index) => { ... });
    // ```
    for (index, character) in line.chars().enumerate() {
        for (ordinal, position) in positions.iter().enumerate() {
            if *position == index {
                let label = hints[ordinal].label;
                widened.push_str(label);
                shown += label.chars().count();
                if hints[ordinal].padding_right {
                    widened.push(' ');
                    shown += 1;
                }
            }
        }
        map.push(shown);
        widened.push(character);
        shown += 1;
    }
    map.push(shown);
    return (widened, map);
}

/// Logical x of every source boundary of one line,
///  shaped alone with the production shaper.
fn boundaries(shaper: &mut TextShaper, text: &str, map: &[usize]) -> Vec<f32> {
    let document = Document::new(text);
    let row = shaper.row(&document, 0, 1.0);
    let mut result = Vec::new();
    for position in map {
        result.push(row.caret_x(*position, 1.0));
    }
    return result;
}

/// What:
///  Largest horizontal movement of any source boundary when the labels are spliced in,
///  and the movement
///       of the line end.
///  The answer is a pair (tuple) of logical pixel distances.
/// Why:
///  A late hint snapshot,
///  or one dropped as stale on reload,
///  moves every glyph after its first hint by this much.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function displacement(line: string, hints: RecordedHint[]): [number, number];
/// ```
fn displacement(line: &str, hints: &[RecordedHint]) -> (f32, f32) {
    let mut shaper = TextShaper::new();
    let mut identity = Vec::new();
    for index in 0..=line.chars().count() {
        identity.push(index);
    }
    let plain = boundaries(&mut shaper, line, &identity);
    let (widened, map) = splice(line, hints);
    let inline = boundaries(&mut shaper, &widened, &map);
    let mut largest: f32 = 0.0;
    // What: `zip` walks both boundary lists in step.
    // Why: Boundary `n` of the plain line corresponds to boundary `n` of the widened line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // plain.forEach((x, index) => { largest = Math.max(largest, inline[index] - x); });
    // ```
    for (before, after) in plain.iter().zip(inline.iter()) {
        largest = largest.max(after - before);
    }
    // What: `last()` returns the final element or nothing; `expect` fails the test if the list is empty.
    // Why: Every line has at least one boundary, its start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const end = inline.at(-1)! - plain.at(-1)!;
    // ```
    let end = inline.last().expect("line end") - plain.last().expect("line end");
    return (largest, end);
}

/// Real rust-analyzer and TypeScript 7 hints move source glyphs by tens of pixels when placed inline.
#[test]
fn inline_hints_from_real_servers_move_source_glyphs() {
    // rust-analyzer, `inspect-passing/rust.events.jsonl`, revision 0, line 5: four hints on one line.
    let rust_line = "    let label = \"héllo 😀\"; let total = area(2, 3);";
    let rust_hints = [
        RecordedHint {
            before: "    let label",
            label: ": &str",
            padding_right: false,
        },
        RecordedHint {
            before: "    let label = \"héllo 😀\"; let total",
            label: ": u32",
            padding_right: false,
        },
        RecordedHint {
            before: "    let label = \"héllo 😀\"; let total = area(",
            label: "width:",
            padding_right: true,
        },
        RecordedHint {
            before: "    let label = \"héllo 😀\"; let total = area(2, ",
            label: "height:",
            padding_right: true,
        },
    ];
    // TypeScript 7, `inspect-passing/typescript.events.jsonl`, revision 0, line 2. Its left padding is a space.
    let ts_line = "const label = \"héllo 😀\"; const message = greet(\"wörld\");";
    let ts_hints = [
        RecordedHint {
            before: "const label = \"héllo 😀\"; const message",
            label: " : string",
            padding_right: false,
        },
        RecordedHint {
            before: "const label = \"héllo 😀\"; const message = greet(",
            label: "name:",
            padding_right: true,
        },
    ];
    let (rust_largest, rust_end) = displacement(rust_line, &rust_hints);
    let (ts_largest, ts_end) = displacement(ts_line, &ts_hints);
    println!(
        "placement inline-displacement rust: largest {rust_largest:.1} px, line end {rust_end:.1} px; \
         typescript: largest {ts_largest:.1} px, line end {ts_end:.1} px"
    );
    assert!(rust_largest > 100.0 && rust_end == rust_largest);
    assert!(ts_largest > 50.0 && ts_end == ts_largest);
}

/// A hint before a tab either changes the tab's width or stops the tab from ending on a pixel stop.
#[test]
fn inline_hint_before_a_tab_changes_tab_width_or_breaks_the_stop() {
    let mut shaper = TextShaper::new();
    let space = boundaries(&mut shaper, " x", &[0, 1])[1];
    let stop = space * TAB_SPACES;
    let line = "\tlet x\t= 1;";
    let hints = [RecordedHint {
        before: "\tlet x",
        label: ": i32",
        padding_right: false,
    }];
    let mut identity = Vec::new();
    for index in 0..=line.chars().count() {
        identity.push(index);
    }
    let plain = boundaries(&mut shaper, line, &identity);
    let (widened, map) = splice(line, &hints);
    let inline = boundaries(&mut shaper, &widened, &map);
    // Boundary 6 starts the second tab and boundary 7 ends it.
    let plain_tab = plain[7] - plain[6];
    // Shaping the widened line recomputes the stop after the label.
    let recomputed_tab = inline[7] - inline[6];
    // Keeping the hint-free tab width instead moves the tab's end by the label's width.
    let label_width = inline[6] - plain[6];
    let frozen_end = plain[7] + label_width;
    let frozen_remainder = frozen_end.rem_euclid(stop);
    println!(
        "placement tab: stop {stop:.2} px, hint-free tab {plain_tab:.2} px ending at {:.2}; \
         inline tab recomputed {recomputed_tab:.2} px; frozen width ends at {frozen_end:.2} px, \
         {frozen_remainder:.2} px past a stop",
        plain[7]
    );
    let width_changed = (recomputed_tab - plain_tab).abs() > 0.01;
    let off_stop = frozen_remainder > 0.01 && (stop - frozen_remainder) > 0.01;
    assert!(width_changed && off_stop);
}

/// Down from a line with an inline hint lands on another source position than without the hint.
#[test]
fn inline_hint_changes_where_down_lands() {
    let above = "let total = area(2, 3);";
    let below = "let other = compute(4, 5, 6);";
    let hints = [RecordedHint {
        before: "let total",
        label: ": u32",
        padding_right: false,
    }];
    let text = format!("{above}\n{below}");
    let document = Document::new(&text);
    let mut shaper = TextShaper::new();
    let (widened, map) = splice(above, &hints);
    let shown = Document::new(&widened);
    let painted = shaper.row(&shown, 0, 1.0);
    let target = shaper.row(&document, 1, 1.0);
    let mut differing = 0;
    // What: `enumerate` pairs each source boundary with its index in the widened line; `*widened_at` reads it.
    // Why: The caret stands on source boundary `position`, painted at the widened index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // map.forEach((widenedAt, position) => { ... });
    // ```
    for (position, widened_at) in map.iter().enumerate() {
        let mut moving = Document::new(&text);
        // What: `ReadingPosition { .. }` builds the collapsed caret record the production key handler moves.
        // Why: `vertical` reads the caret from the document, as it does for a real key press.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // moving.select({ anchor: position, head: position, viewport: 0 });
        // ```
        moving.select(ReadingPosition {
            anchor: position,
            head: position,
            viewport: 0,
            affinity: ide_app::document::ReadingAffinity::Downstream,
        });
        // `None` means no remembered column: the aim is the caret's own x on hint-free geometry.
        let plain = vertical(&mut shaper, &moving, 1.0, 0.0, 1, None).head;
        // With inline hints the aim is the painted caret x, which includes every label before it.
        let aim = painted.caret_x(*widened_at, 1.0);
        let inline = target.hit(aim, 1.0);
        if plain != inline {
            differing += 1;
        }
    }
    let total = map.len();
    println!(
        "placement vertical: {differing} of {total} caret positions land elsewhere with the hint inline"
    );
    assert!(differing > 0);
}
