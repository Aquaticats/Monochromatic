//! Virtual rows:
//!  what a source line shows above its code row.
//!  Inlay hints come first,
//!  each at the pixel x of
//! the character it annotates;
//!  diagnostic messages follow,
//!  one row per message line,
//!  starting at the pixel x of
//! the diagnostic.
//!  The rows are not source text:
//!  nothing here changes a code row or a source position.
//!
//! A line's rows form a block.
//!  The block sits tight against its own code row and is separated from the previous
//! line by [`BLOCK_GAP`],
//!  so the rows read as belonging to the line beneath them.

/// Diagnostics as the annotation store indexes them,
///  and the text one message shows.
use crate::annotation::{Problem, describe, severity_name};
/// Severities as the Language module names them.
use crate::language::diagnostics::Severity;

/// What:
///  Height of one virtual row in logical pixels;
///  `f32` is a 32-bit float (sibling `f64`).
/// Why:
///  The 13 px row text needs 16 px:
///  its font's own ascent plus descent.
///  A row lower than a 24 px code row
///      also shows at a glance that it is not source text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const ROW_HEIGHT = 16;
/// ```
pub const ROW_HEIGHT: f32 = 16.0;
/// Empty space above a block.
///  It makes the distance to the previous line's text visibly larger than the
/// distance to the block's own code row,
///  which is the only thing that says which line the rows belong to.
pub const BLOCK_GAP: f32 = 10.0;
/// Least horizontal space between two hints on one row.
pub const HINT_GAP: f32 = 8.0;
/// Font size of virtual-row text in logical pixels,
///  smaller than the 15 px source text.
pub const ROW_TEXT: f32 = 13.0;
/// How far the second and later rows of one message start right of its first row.
pub const CONTINUATION_INDENT: f32 = 16.0;
/// What:
///  A message row holds at most this many columns;
///  `usize` is the count type (siblings `u32`,
///  `u64`).
/// Why:
///  Long messages wrap instead of running on without bound.
///  The limit is counted in
///      characters,
///  so this first break never depends on the window or on shaping;
///  a row wider than the text area is then broken again to fit it
///  (`Annotations::assemble`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const WRAP_COLUMNS = 80;
/// ```
pub const WRAP_COLUMNS: usize = 80;
/// At most this many rows are shown for one message;
///  the last one then says how many lines are left out.
pub const MESSAGE_ROWS: usize = 12;
/// At most this many messages are shown above one line;
///  one more row then says how many are left out.
pub const LINE_MESSAGES: usize = 8;

/// What:
///  One inlay hint placed on a hint row;
///  `String` owns the label (sibling:
///  borrowed `&str`).
///       `PartialEq` lets two placements be compared,
///  which decides whether a frame must be repainted.
/// Why:
///  The row and the pixel x are decided once per snapshot by packing;
///  every frame paints from them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HintPlace = { row: number; position: number; x: number; width: number; text: string };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct HintPlace {
    /// Zero-based hint row inside the block,
    ///  counted from the top.
    pub row: usize,
    /// Source character the hint annotates;
    ///  `usize` is the index type ropes use (siblings `u32`,
    ///  `u64`).
    pub position: usize,
    /// Left edge in logical pixels from the start of the line's text:
    ///  the caret x of the annotated position.
    pub x: f32,
    /// Shaped width of the label in logical pixels.
    pub width: f32,
    /// The label as the server sent it,
    ///  without padding spaces.
    pub text: String,
}

/// What:
///  One row of a diagnostic message.
/// Why:
///  Rows are derived from the message text alone,
///  so a line's row count at the first break is known without shaping;
///  a row wider than the text area is broken again by shaping when the blocks are assembled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MessageRow = { start: number; continued: boolean; severity: Severity; text: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageRow {
    /// Source character whose pixel x the row is aligned to:
    ///  the diagnostic's start.
    pub start: usize,
    /// A second or later row of its message;
    ///  it is indented and does not start with the severity word.
    pub continued: bool,
    /// Severity,
    ///  which selects the row's ink.
    pub severity: Severity,
    /// What:
    ///  How the row's text follows the row before it,
    ///  or nothing when it starts a line of the message.
    /// Why:
    ///  The rows made by the break at 80 columns are one line of prose;
    ///  a row wider than the text area is broken again from the whole line,
    /// not from each row alone,
    ///  which would leave a short tail on every row.
    ///  The joiner is a blank where the break was at a blank and nothing where a word was cut.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// gap?: ' ' | '';
    /// ```
    pub gap: Option<&'static str>,
    /// Row text.
    pub text: String,
}

/// What:
///  Everything one line shows above its code row;
///  `Vec<T>` is a growable list (siblings `[T; N]`,
///  `&[T]`).
///       `held` is a pair (tuple) of reserved heights.
/// Why:
///  The vertical mapping needs each block's height,
///  and the frame paints its rows;
///  both read this record.
///      After an external change the previous rows' space is held empty until annotations of the new text
///      arrive,
///  so the text does not jump twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Block = { line: number; hintRows: number; hints: HintPlace[]; messages: MessageRow[];
///   held: [hints: number, messages: number] };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    /// Zero-based source line the block belongs to.
    pub line: usize,
    /// Number of hint rows.
    pub hint_rows: usize,
    /// Hints with their rows and positions,
    ///  in source order.
    pub hints: Vec<HintPlace>,
    /// Message rows,
    ///  top to bottom.
    pub messages: Vec<MessageRow>,
    /// Height held for hint rows and for message rows of the previous text revision,
    ///  in logical pixels.
    pub held: (f32, f32),
}

/// Heights and row positions of one block.
impl Block {
    /// Height of the hint part:
    ///  its rows,
    ///  or the held space when that is taller.
    pub fn hint_part(&self) -> f32 {
        return (self.hint_rows as f32 * ROW_HEIGHT).max(self.held.0);
    }

    /// Height of the message part:
    ///  its rows,
    ///  or the held space when that is taller.
    pub fn message_part(&self) -> f32 {
        return (self.messages.len() as f32 * ROW_HEIGHT).max(self.held.1);
    }

    /// Height of the whole block with the gap above it;
    ///  zero for a block without rows or held space.
    pub fn height(&self) -> f32 {
        let rows = self.hint_part() + self.message_part();
        if rows <= 0.0 {
            return 0.0;
        }
        return BLOCK_GAP + rows;
    }

    /// What:
    ///  How far above the top of the code row hint row `row` starts.
    /// Why:
    ///  Rows are stacked upwards from the code row,
    ///  so the block stays tight against its own line
    ///      whatever space is held above the rows.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hintRise(row: number): number;
    /// ```
    pub fn hint_rise(&self, row: usize) -> f32 {
        // `saturating_sub` stops at zero for a row index past the last hint row.
        let below = self.hint_rows.saturating_sub(row);
        return self.message_part() + below as f32 * ROW_HEIGHT;
    }

    /// How far above the top of the code row message row `index` starts.
    pub fn message_rise(&self, index: usize) -> f32 {
        let below = self.messages.len().saturating_sub(index);
        return below as f32 * ROW_HEIGHT;
    }
}

/// What:
///  Columns one character takes for wrapping:
///  two for the wide scripts from the CJK radicals on,
///  one
///       otherwise.
///  `char` is one Unicode scalar;
///  `as u32` reads its number.
/// Why:
///  A row of eighty CJK characters would be about twice as wide as eighty Latin ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function columns(character: string): number { return character.codePointAt(0) >= 0x2e80 ? 2 : 1; }
/// ```
fn columns(character: char) -> usize {
    if character as u32 >= 0x2E80 {
        return 2;
    }
    return 1;
}

/// What:
///  Break one message line into rows of at most `limit` columns at blanks;
///  a word longer than a row is
///       cut.
///  Leading blanks stay on the first row,
///  a tab counting as two.
///  `&str` lends the line.
/// Why:
///  Messages nest their explanations by indentation,
///  which must stay readable;
///  everything else is prose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wrap(line: string, limit: number): string[];
/// ```
pub fn wrap(line: &str, limit: usize) -> Vec<String> {
    // Each row keeps its text and drops how it follows the row before it.
    let mut rows: Vec<String> = Vec::new();
    for (text, _) in wrap_gapped(line, limit) {
        rows.push(text);
    }
    return rows;
}

/// What:
///  [`wrap`] with,
/// for each row,
/// how its text follows the row before it:
///  nothing for the first row,
///  a blank where the break was at a blank,
///  and an empty joiner where a word was cut.
/// Why:
///  A row broken again from the whole line needs the line back,
///  and joining the rows with blanks would put a blank into a cut word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wrapGapped(line: string, limit: number): [text: string, gap?: ' ' | ''][];
/// ```
fn wrap_gapped(line: &str, limit: usize) -> Vec<(String, Option<&'static str>)> {
    // `Vec::new()` and `String::new()` create the empty list and the empty row the loops fill.
    let mut rows = Vec::new();
    let mut gap: Option<&'static str> = None;
    let mut current = String::new();
    let mut width: usize = 0;
    for character in line.chars() {
        if character == ' ' {
            current.push(' ');
            width += 1;
        } else if character == '\t' {
            current.push_str("  ");
            width += 2;
        } else {
            break;
        }
    }
    let mut started = false;
    // `split_whitespace` yields the words between runs of blanks.
    for word in line.split_whitespace() {
        let mut needed = 0;
        for character in word.chars() {
            needed += columns(character);
        }
        if started && width + 1 + needed > limit {
            rows.push((current, gap));
            gap = Some(" ");
            current = String::new();
            width = 0;
            started = false;
        }
        if started {
            current.push(' ');
            width += 1;
        }
        for character in word.chars() {
            if width + columns(character) > limit && width > 0 {
                rows.push((current, gap));
                gap = Some("");
                current = String::new();
                width = 0;
            }
            current.push(character);
            width += columns(character);
        }
        started = true;
    }
    if started {
        rows.push((current, gap));
    }
    return rows;
}

/// What:
///  The rows of one message:
///  its text as [`describe`] spells it,
///  each line wrapped,
///  blank lines dropped,
///       and at most [`MESSAGE_ROWS`] rows,
///  the last of which then counts the lines left out.
///       `&Problem` lends the diagnostic.
/// Why:
///  Every message is readable in place;
///  only a pathological one is cut,
///  and its full text stays in the
///      source view's accessible description while the caret is in its range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rowsOf(problem: Problem): MessageRow[];
/// ```
fn rows_of(problem: &Problem) -> Vec<MessageRow> {
    let mut texts = Vec::new();
    for line in describe(problem).lines() {
        // `extend` appends every wrapped row of this message line.
        texts.extend(wrap_gapped(line.trim_end(), WRAP_COLUMNS));
    }
    let total = texts.len();
    let mut rows = Vec::new();
    for (index, (text, gap)) in texts.into_iter().enumerate() {
        if total > MESSAGE_ROWS && index + 1 == MESSAGE_ROWS {
            rows.push(MessageRow {
                start: problem.mark.start,
                continued: true,
                severity: problem.mark.severity,
                gap: None,
                // `format!` builds the text from a template, like a TS template literal.
                text: format!("… {} more lines", total + 1 - MESSAGE_ROWS),
            });
            break;
        }
        rows.push(MessageRow {
            start: problem.mark.start,
            continued: index > 0,
            severity: problem.mark.severity,
            gap,
            text,
        });
    }
    return rows;
}

/// What:
///  The message rows of one line from the diagnostics that start on it,
///  already ordered worst first.
///       `&[&Problem]` lends the list of lent diagnostics.
///  Past [`LINE_MESSAGES`] messages one row counts
///       the rest and names their worst severity.
/// Why:
///  Worst first means a cap never hides an error behind a hint;
///  the count keeps a pile from burying the code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function messageRows(problems: Problem[]): MessageRow[];
/// ```
pub fn message_rows(problems: &[&Problem]) -> Vec<MessageRow> {
    let mut rows = Vec::new();
    for (index, problem) in problems.iter().enumerate() {
        if index == LINE_MESSAGES {
            let hidden = problems.len() - LINE_MESSAGES;
            rows.push(MessageRow {
                start: problem.mark.start,
                continued: false,
                severity: problem.mark.severity,
                gap: None,
                text: format!(
                    "{hidden} more on this line, the worst: {}",
                    severity_name(problem.mark.severity)
                ),
            });
            break;
        }
        rows.extend(rows_of(problem));
    }
    return rows;
}
