//! What:
//!  Reading the values of `run:` and `shell:` keys.
//! Why:
//!  Only a block literal and a plain one-line command are read;
//!  every other form is refused with a message that says what to write,
//!  because a form the reader skipped would hide a long script from the rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function captureRun(lines: Line[], index: number, column: number, value: string): [RunBody, number];
//! function captureShell(line: Line, value: string, offset: number): Shell;
//! ```

/// Import the line records and the result records.
use crate::workflow_lines::{Line, without_comment};
use crate::workflow_model::{BodyLine, RunBody, Shell, WalkError};

/// Build a refusal at a workflow byte offset.
fn refuse(offset: usize, message: &str) -> WalkError {
    return WalkError {
        offset,
        message: String::from(message),
    };
}

/// What:
///  Read the lines of a block literal that follows its `|` header.
/// Why:
///  The body is every line indented deeper than the key,
///  stripped of the first line's indentation,
///  with blank lines kept between content and dropped at the end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blockLines(lines: Line[], header: number, column: number): [BodyLine[], number];
/// ```
fn block_lines(
    lines: &[Line<'_>],
    header: usize,
    column: usize,
) -> Result<(Vec<BodyLine>, usize), WalkError> {
    let mut body: Vec<BodyLine> = Vec::new();
    let mut pending: Vec<usize> = Vec::new();
    let mut body_indent: Option<usize> = None;
    let mut next: usize = header + 1;
    while next < lines.len() {
        let line: &Line<'_> = &lines[next];
        if line.is_blank() {
            pending.push(next);
            next += 1;
            continue;
        }
        if line.indent <= column {
            break;
        }
        let indent: usize = *body_indent.get_or_insert(line.indent);
        if line.indent < indent {
            // A comment less indented than the body ends the scalar; other text is malformed.
            if line.text.starts_with('#') {
                break;
            }
            return Err(refuse(
                line.text_start(),
                "a line of this run script is indented less than its first line",
            ));
        }
        for blank in pending.drain(..) {
            let gap: &Line<'_> = &lines[blank];
            body.push(BodyLine {
                envelope: gap.start,
                payload: (gap.start + indent).min(gap.start + gap.indent + gap.text.len()),
                end: gap.end,
            });
        }
        body.push(BodyLine {
            envelope: line.start,
            payload: line.start + indent,
            end: line.end,
        });
        next += 1;
    }
    return Ok((body, next));
}

/// What:
///  Read the value of a step's `run:` key.
/// Why:
///  Returns the script and the index of the first line after it,
///  so the walker resumes after the body.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function captureRun(lines: Line[], index: number, column: number, value: string, valueOffset: number): [RunBody, number];
/// ```
pub(crate) fn capture_run(
    lines: &[Line<'_>],
    index: usize,
    column: usize,
    value: &str,
    value_offset: usize,
) -> Result<(RunBody, usize), WalkError> {
    let line: &Line<'_> = &lines[index];
    let at: usize = line.text_start() + value_offset;
    let visible: &str = without_comment(value);
    if let Some(header) = visible.strip_prefix('|') {
        if !header.is_empty() && header != "-" && header != "+" {
            return Err(refuse(
                at,
                "this block literal has an indentation indicator; write `run: |` with the body indented under it",
            ));
        }
        let (body, next): (Vec<BodyLine>, usize) = block_lines(lines, index, column)?;
        return Ok((RunBody::Block { lines: body }, next));
    }
    if visible.is_empty() {
        return Err(refuse(
            at,
            "this run value does not start on the key's line; write `run: |` or `run: command`",
        ));
    }
    let first: u8 = visible.as_bytes()[0];
    if b">\"'&*![{%@`".contains(&first) {
        return Err(refuse(
            at,
            "this run value is a folded, quoted, anchored, aliased, tagged or flow form; write `run: |` or an unquoted `run: command`",
        ));
    }
    for later in &lines[index + 1..] {
        if later.is_skippable() {
            continue;
        }
        if later.indent > column {
            return Err(refuse(
                at,
                "this run command continues on the next line; write `run: |` and put the lines under it",
            ));
        }
        break;
    }
    return Ok((
        RunBody::Plain {
            start: at,
            end: at + visible.len(),
        },
        index + 1,
    ));
}

/// What:
///  Read the value of a `shell:` key.
/// Why:
///  A plain or quoted one-line value is enough;
///  anything else cannot be classified and is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function captureShell(line: Line, value: string, valueOffset: number): Shell;
/// ```
pub(crate) fn capture_shell(
    line: &Line<'_>,
    value: &str,
    value_offset: usize,
) -> Result<Shell, WalkError> {
    let at: usize = line.text_start() + value_offset;
    let visible: &str = without_comment(value);
    let first: u8 = visible.as_bytes().first().copied().unwrap_or(b' ');
    if first == b'"' || first == b'\'' {
        let quote: char = first as char;
        if visible.len() >= 2
            && visible.ends_with(quote)
            && !visible[1..visible.len() - 1].contains(quote)
        {
            return Ok(Shell {
                value: String::from(&visible[1..visible.len() - 1]),
                offset: at,
            });
        }
        return Err(refuse(
            at,
            "this shell value is a quoted string the reader cannot place",
        ));
    }
    if visible.is_empty() || b"|>&*![{%@`".contains(&first) {
        return Err(refuse(
            at,
            "this shell value is empty or a block, anchor, alias, tag or flow form; write the shell on the key's line",
        ));
    }
    return Ok(Shell {
        value: String::from(visible),
        offset: at,
    });
}

/// Run and shell value controls stay outside release artifacts.
#[cfg(test)]
#[path = "workflow_run_tests.rs"]
mod tests;
