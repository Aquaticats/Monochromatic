//! What:
//!  The unified-diff parser that replay's subsumption reads:
//!       `git diff-tree -p` output over single-level trees named by decimal index,
//!       split into hunks per entry name.
//! Why:
//!  Subsumption decides from hunk lines and their markers;
//!       the parser keeps every byte exact,
//!       so a hunk line round-trips to the blob bytes it came from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! parseIndexedPatch(text: string): ReadonlyMap<string, readonly PatchHunk[]>
//! ```

/// `BTreeMap` keeps the parsed sections in a deterministic order for tests.
use std::collections::BTreeMap;
/// Owned byte strings for lines.
use std::fmt;

/// What:
///  One patch line:
///       its marker (`b' '`, `b'-'`, `b'+'`) and its bytes.
/// Why:
///   The reverse check and the containment check both read the markers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PatchLine = { op: ' ' | '-' | '+'; text: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatchLine {
    /// Context, removal, or addition.
    pub op: u8,
    /// Line bytes with their newline,
    ///  without it for an incomplete last line.
    pub text: Vec<u8>,
}

/// What:
///  One unified-diff hunk:
///       its first old and new line and its lines in order.
/// Why:
///   A zero-based start is derived from `oldStart` by the callers that need it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PatchHunk = { oldStart: number; newStart: number; lines: readonly PatchLine[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatchHunk {
    /// First line of the old side, `0` for an empty old side.
    pub old_start: i64,
    /// First line of the new side, `0` for an empty new side.
    pub new_start: i64,
    /// Hunk lines in order.
    pub lines: Vec<PatchLine>,
}

/// What:
///  Patch output that does not have the shape `git diff-tree -p` writes.
/// Why:
///   Callers fail the replay rather than guess at a truncated or foreign patch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReplayPatchFormatError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayPatchFormatError(pub String);

impl fmt::Display for ReplayPatchFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}", self.0);
    }
}

/// Section header of one file in `diff-tree -p` output.
const SECTION_HEADER: &[u8] = b"diff --git a/";
/// Hunk header prefix.
const HUNK_HEADER: &[u8] = b"@@ -";
/// Marker of a `\ No newline at end of file` line.
const INCOMPLETE_MARKER: u8 = b'\\';

/// What:
///  Split bytes into lines,
///       each keeping its newline;
///       an unterminated last line stays as it is.
/// Why:
///   Hunk bodies and blob contents are compared line by line,
///       newline included, the way Git's `xdiff` compares them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// splitKeepingNewlines('a\nb'); // ['a\n', 'b']
/// ```
pub fn split_keeping_newlines(text: &[u8]) -> Vec<Vec<u8>> {
    // `mut` allows collecting lines while walking the bytes once.
    let mut lines: Vec<Vec<u8>> = Vec::new();
    let mut start: usize = 0;
    while start < text.len() {
        let stop: usize = match text[start..].iter().position(|byte| { return *byte == b'\n'; }) {
            Some(offset) => start + offset + 1,
            None => text.len(),
        };
        lines.push(text[start..stop].to_vec());
        start = stop;
    }
    return lines;
}

/// What:
///  Parse one non-negative decimal number,
///       rejecting anything but ASCII digits.
/// Why:
///   Git prints only digits in hunk ranges;
///       any other byte means the patch is not one Git wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(text) // must be a safe integer
/// ```
fn parse_number(text: &[u8]) -> Option<i64> {
    if text.is_empty() || !text.iter().all(|byte| { return byte.is_ascii_digit(); }) {
        return None;
    }
    // `to_str` is safe here: every byte is an ASCII digit.
    let digits: &str = std::str::from_utf8(text).ok()?;
    return digits.parse::<i64>().ok();
}

/// What:
///  Parse one `-start[,count]` or `+start[,count]` range;
///       a missing count is `1`.
/// Why:
///   Hunk headers name both sides this way,
///       and the counts tell the body parser how many lines to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseRange('-3,2'); // { start: 3, count: 2 }
/// ```
fn parse_range(range: &[u8]) -> Result<(i64, i64), ReplayPatchFormatError> {
    let body: &[u8] = &range[1..];
    let (start_text, count_text): (&[u8], &[u8]) = match body.iter().position(|byte| { return *byte == b','; }) {
        Some(comma) => (&body[..comma], &body[comma + 1..]),
        None => (body, b"1"),
    };
    return match (parse_number(start_text), parse_number(count_text)) {
        (Some(start), Some(count)) => Ok((start, count)),
        _ => Err(ReplayPatchFormatError(format!(
            "malformed hunk range {}",
            String::from_utf8_lossy(range)
        ))),
    };
}

/// What:
///  Drop the newline of the last parsed line,
///       as a following `\ No newline at end of file` marker says.
/// Why:
///   An incomplete last line compares as a prefix of the file line that has the newline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// markIncomplete(parsed); // throws when no line precedes the marker
/// ```
fn mark_incomplete(parsed: &mut [PatchLine]) -> Result<(), ReplayPatchFormatError> {
    let last: &mut PatchLine = match parsed.last_mut() {
        Some(line) => line,
        None => {
            return Err(ReplayPatchFormatError(String::from(
                "an incomplete-line marker precedes every hunk line",
            )));
        }
    };
    if last.text.last() == Some(&b'\n') {
        last.text.pop();
    }
    return Ok(());
}

/// What:
///  Parse one hunk starting at its header line;
///       returns the hunk and the index of the line after it.
/// Why:
///   Every body line is read once,
///       counted down from the header's old and new counts,
///       so a truncated or overlong hunk fails instead of swallowing the next section.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseHunk({ lines, start }); // { hunk, next }
/// ```
fn parse_hunk(lines: &[Vec<u8>], start: usize) -> Result<(PatchHunk, usize), ReplayPatchFormatError> {
    let header: &[u8] = &lines[start];
    let end: usize = match find_bytes(header, b" @@", 4) {
        Some(found) => found,
        None => {
            return Err(ReplayPatchFormatError(format!(
                "malformed hunk header {}",
                String::from_utf8_lossy(header).trim_end()
            )));
        }
    };
    let ranges: &[u8] = &header[3..end];
    let (old_range, new_range): (&[u8], &[u8]) = match ranges.iter().position(|byte| { return *byte == b' '; }) {
        Some(space) => (&ranges[..space], &ranges[space + 1..]),
        None => {
            return Err(ReplayPatchFormatError(format!(
                "malformed hunk header {}",
                String::from_utf8_lossy(header).trim_end()
            )));
        }
    };
    let (old_start, old_count): (i64, i64) = parse_range(old_range)?;
    let (new_start, new_count): (i64, i64) = parse_range(new_range)?;
    // `mut` allows the counts and the cursor to count down while the body is read.
    let mut parsed: Vec<PatchLine> = Vec::new();
    let mut old_left: i64 = old_count;
    let mut new_left: i64 = new_count;
    let mut cursor: usize = start + 1;
    while old_left > 0 || new_left > 0 {
        let line: &[u8] = match lines.get(cursor) {
            Some(found) => found,
            None => {
                return Err(ReplayPatchFormatError(format!(
                    "hunk {} is truncated",
                    String::from_utf8_lossy(header).trim_end()
                )));
            }
        };
        cursor += 1;
        let marker: u8 = line_marker(line)?;
        if marker == INCOMPLETE_MARKER {
            mark_incomplete(&mut parsed)?;
            continue;
        }
        if marker != b'+' {
            old_left -= 1;
        }
        if marker != b'-' {
            new_left -= 1;
        }
        let text: Vec<u8> = if line == b"\n" { b"\n".to_vec() } else { line[1..].to_vec() };
        parsed.push(PatchLine { op: marker, text });
    }
    if lines.get(cursor).is_some_and(|line| { return line.starts_with(b"\\"); }) {
        mark_incomplete(&mut parsed)?;
        cursor += 1;
    }
    return Ok((
        PatchHunk {
            old_start,
            new_start,
            lines: parsed,
        },
        cursor,
    ));
}

/// What:
///  The marker of one hunk body line:
///       a bare newline is an empty context line,
///       and an incomplete-line marker is reported as `\`.
/// Why:
///   A line without one of the four markers cannot belong to a hunk body.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lineMarker('\n'); // ' '
/// ```
fn line_marker(line: &[u8]) -> Result<u8, ReplayPatchFormatError> {
    let first: u8 = if line == b"\n" { b' ' } else { line.first().copied().unwrap_or(0) };
    if matches!(first, b' ' | b'-' | b'+' | b'\\') {
        return Ok(first);
    }
    return Err(ReplayPatchFormatError(format!(
        "hunk line {} has no diff marker",
        String::from_utf8_lossy(line)
    )));
}

/// What:
///  The first index at or after `from` where `needle` starts in `haystack`.
/// Why:
///   The hunk header's range end is searched from after the `@@ -` prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// haystack.indexOf(needle, from)
/// ```
fn find_bytes(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if haystack.len() < needle.len() {
        return None;
    }
    return (from..=haystack.len() - needle.len()).find(|index| { return &haystack[*index..*index + needle.len()] == needle; });
}

/// What:
///  Parse `git diff-tree -p` output over index-named single-level trees
///       into hunks per entry name;
///       an entry without a section has no change.
/// Why:
///   Subsumption reads each candidate's landed and prepared hunks by its index name,
///       so no patch header carries a user path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseIndexedPatch(text); // ReadonlyMap<string, readonly PatchHunk[]>
/// ```
pub fn parse_indexed_patch(
    text: &[u8],
) -> Result<BTreeMap<Vec<u8>, Vec<PatchHunk>>, ReplayPatchFormatError> {
    let lines: Vec<Vec<u8>> = split_keeping_newlines(text);
    // `mut` allows the sections and the cursor to advance.
    let mut files: BTreeMap<Vec<u8>, Vec<PatchHunk>> = BTreeMap::new();
    let mut section: Option<Vec<u8>> = None;
    let mut hunks: Vec<PatchHunk> = Vec::new();
    let mut cursor: usize = 0;
    while cursor < lines.len() {
        let line: &[u8] = &lines[cursor];
        if line.starts_with(HUNK_HEADER) {
            let (hunk, next): (PatchHunk, usize) = parse_hunk(&lines, cursor)?;
            hunks.push(hunk);
            cursor = next;
            continue;
        }
        if line.starts_with(SECTION_HEADER) {
            if let Some(name) = section.take() {
                files.insert(name, std::mem::take(&mut hunks));
            }
            hunks.clear();
            let rest: &[u8] = &line[SECTION_HEADER.len()..];
            let name_end: usize = match rest.iter().position(|byte| { return *byte == b' '; }) {
                Some(space) => space,
                None => {
                    return Err(ReplayPatchFormatError(format!(
                        "section header {} has no destination",
                        String::from_utf8_lossy(line).trim_end()
                    )));
                }
            };
            section = Some(rest[..name_end].to_vec());
        }
        cursor += 1;
    }
    if let Some(name) = section {
        files.insert(name, hunks);
    }
    return Ok(files);
}

#[cfg(test)]
#[path = "replay_patch_tests.rs"]
mod tests;
