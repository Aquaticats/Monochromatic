//! Fitting a line's message to the text area:
//!  the rows a message is drawn in,
//!  broken where they do not fit,
//!  and cut to the one row cap.

/// The shaper that measures a row and breaks it.
use crate::shaped_text::TextShaper;
/// One message row of a line,
///  the width its soft-wrap continuation starts at,
///  the width its text has,
///  and the one cap that cuts a message to its row limit.
use crate::virtual_row::{
    CONTINUATION_INDENT,
    MessageRow,
    ROW_TEXT,
    cap_message_rows,
};

/// What:
///  The rows of one line's messages broken so that each fits `limit` logical pixels,
///  capped again at [`crate::virtual_row::MESSAGE_ROWS`] rows for each message by the one cap,
///  with the number of rows that had to be broken by shaping.
///  `widths` holds the shaped width of every row of `rows`;
///  only a row wider than the limit is shaped again.
/// Why:
///  The source view has no horizontal scrolling,
///  so a message must fit the text area it stands in.
///  The first row of a message may use the whole width,
///  and every later row starts [`CONTINUATION_INDENT`] further right,
///  so the pieces of a row that is not itself continued break in two steps.
///  A cap that counted rows before they were broken would let a narrow window show many times more rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fitMessages(rows: MessageRow[], widths: number[], shaper: TextShaper, scale: number,
///   limit: number): [rows: MessageRow[], broken: number];
/// ```
pub(super) fn fit_messages(
    rows: &[MessageRow],
    widths: &[f32],
    shaper: &mut TextShaper,
    scale: f32,
    limit: f32,
) -> (Vec<MessageRow>, u64) {
    let mut broken: u64 = 0;
    let narrowest = ROW_TEXT;
    let mut fitted: Vec<MessageRow> = Vec::new();
    let mut from = 0;
    while from < rows.len() {
        // What: A line of the message is the row that starts it and the rows that follow it softly.
        // Why: The rows of one line were cut at 80 columns; breaking each of them alone would leave a short tail
        //      on every one, so a line with a row wider than the limit is joined and broken again as a whole.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const line = rows.slice(from, rows.findIndex((row, i) => i > from && row.gap === undefined));
        // ```
        let mut to = from + 1;
        while to < rows.len() && rows[to].gap.is_some() {
            to += 1;
        }
        let head = &rows[from];
        let indent = if head.continued {
            CONTINUATION_INDENT
        } else {
            0.0
        };
        // Every row of the line must fit the limit for the line to stay as it was.
        let mut fits = true;
        for index in from..to {
            let own = if rows[index].continued {
                CONTINUATION_INDENT
            } else {
                0.0
            };
            if !(own + widths[index] <= limit) {
                fits = false;
                break;
            }
        }
        if fits {
            fitted.extend_from_slice(&rows[from..to]);
            from = to;
            continue;
        }
        broken += (to - from) as u64;
        let mut joined = head.text.clone();
        for row in &rows[from + 1..to] {
            joined.push_str(row.gap.unwrap_or(" "));
            joined.push_str(&row.text);
        }
        let mut pieces = shaper.wrap_row_text(&joined, scale, (limit - indent).max(narrowest));
        if !head.continued && pieces.len() > 1 {
            // The rest of the line starts a continuation, which is indented, so it breaks again at its own width.
            let rest = joined[pieces[0].len()..].trim_start();
            let tail =
                shaper.wrap_row_text(rest, scale, (limit - CONTINUATION_INDENT).max(narrowest));
            pieces.truncate(1);
            pieces.extend(tail);
        }
        if pieces.is_empty() {
            fitted.extend_from_slice(&rows[from..to]);
            from = to;
            continue;
        }
        for (index, piece) in pieces.into_iter().enumerate() {
            fitted.push(MessageRow {
                start: head.start,
                continued: head.continued || index > 0,
                severity: head.severity,
                gap: if index == 0 { head.gap } else { Some(" ") },
                text: piece,
            });
        }
        from = to;
    }
    // Each message is its first row and the continued rows after it, cut by the one row cap.
    let mut capped = Vec::new();
    let mut first = 0;
    while first < fitted.len() {
        let mut end = first + 1;
        while end < fitted.len() && fitted[end].continued {
            end += 1;
        }
        capped.extend(cap_message_rows(&fitted[first..end]));
        first = end;
    }
    return (capped, broken);
}
