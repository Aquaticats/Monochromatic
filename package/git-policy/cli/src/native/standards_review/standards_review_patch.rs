//! What:
//!  Reading one file's changed new-side line spans out of a `git show --patch` text,
//! so the review needs no
//!  `git diff -U0` start per excerpted file.
//! Why:
//!  The script runs `git diff -U0 <sha>^ <sha> -- <name>` for each of up to eight files.
//! A hunk of the three-line-context
//!  patch is a list of change runs,
//! and each run is one zero-context hunk,
//! so the same spans follow from the patch the
//!  review already holds.
//! The spans are on the new side because the excerpt is cut from the new-side file.
//! The patch is read and matched as the bytes Git wrote:
//! with `core.quotePath` off Git prints a path's bytes as
//!  they are,
//! so a path that is not UTF-8 reaches the header as those bytes and decoding the patch first would
//!  lose the section it names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const match of git('diff', '-U0', `${sha}^`, sha, '--', name).matchAll(/^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? /gm)) { ... }
//! ```

/// A diff section's opening line prefix.
const SECTION_PREFIX: &[u8] = b"diff --git ";

/// A hunk header's opening characters.
const HUNK_PREFIX: &[u8] = b"@@ ";

/// What:
///  Whether one byte is the line feed that ends a patch line.
///  `&u8` borrows the byte,
/// as `split` hands it over.
/// Why:
///   `split` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLineFeed = (byte: number) => byte === '\n'.charCodeAt(0);
/// ```
pub(crate) fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  One run of changed lines on the new side of a file:
/// the first new line the change touches and the last.
///  A pure deletion touches the new line that now stands where the removed lines were.
/// Why:
///   The excerpt window is the union of these spans over the file's runs,
/// cut from the new-side file the commit
///   produced.
/// The script applied the old-side numbers to the new-side file,
/// which shifts the window after an earlier
///   insertion;
/// this port uses the new side on both ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NewSpan = { first: number; last: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NewSpan {
    /// First changed new line.
    pub first: u64,
    /// Last changed new line.
    pub last: u64,
}

/// What:
///  The path a patch line prints,
/// as bytes,
/// with Git's own quoting decoded when the line quotes it.
/// Why:
///   Git quotes a path that holds a quote,
/// a backslash,
/// a control character or,
/// with `core.quotePath` on,
///  a byte above 0x7f,
/// and the header then carries escapes where the name carries bytes.
/// A path it does not quote is
///  printed as it is,
/// spaces included,
/// so nothing needs splitting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const printedPath = (text) => text.startsWith('"') ? unquoteCStyle(text) : Buffer.from(text);
/// ```
fn printed_path(text: &[u8]) -> Option<Vec<u8>> {
    return match take_quoted(text) {
        Some((decoded, after)) if after.is_empty() => Some(decoded),
        Some(_unfinished) => None,
        None => Some(text.to_vec()),
    };
}

/// What:
///  The C-quoted text at the start of `text`,
/// decoded,
/// and the text after its closing quote.
/// Why:
///   This is Git's `unquote_c_style`:
/// the escapes are the short forms and up to three octal digits,
///  and a quoted path's bytes are all ASCII in the patch text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeQuoted(text) { /* parse "..." and decode C escapes */ }
/// ```
fn take_quoted(text: &[u8]) -> Option<(Vec<u8>, &[u8])> {
    if text.first() != Some(&b'"') {
        return None;
    }
    let mut decoded: Vec<u8> = Vec::new();
    let mut index: usize = 1;
    while index < text.len() {
        match text[index] {
            b'"' => return Some((decoded, text.get(index + 1..).unwrap_or_default())),
            b'\\' => {
                let escape: u8 = *text.get(index + 1)?;
                let (byte, used): (u8, usize) = match escape {
                    b'a' => (7, 2),
                    b'b' => (8, 2),
                    b'f' => (12, 2),
                    b'n' => (10, 2),
                    b'r' => (13, 2),
                    b't' => (9, 2),
                    b'v' => (11, 2),
                    b'\\' => (b'\\', 2),
                    b'"' => (b'"', 2),
                    b'0'..=b'7' => {
                        let mut value: u32 = 0;
                        let mut digits: usize = 0;
                        while digits < 3 {
                            let Some(&next) = text.get(index + 1 + digits) else { break; };
                            let Some(digit) = char::from(next).to_digit(8) else { break; };
                            value = value * 8 + digit;
                            digits += 1;
                        }
                        if digits == 0 {
                            return None;
                        }
                        (u8::try_from(value).ok()?, 1 + digits)
                    }
                    _other => return None,
                };
                decoded.push(byte);
                index += used;
            }
            other => {
                decoded.push(other);
                index += 1;
            }
        }
    }
    return None;
}

/// What:
///  Whether the section header names `name` on both sides.
/// Why:
///   Git quotes both sides or neither,
/// so a header that quotes is read as its two printed paths,
/// and one that does not
///  is compared with the name's exact bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [from, to] = printedPaths(header); return eq(from, `a/${name}`) && eq(to, `b/${name}`);
/// ```
fn header_is_for(header: &[u8], name: &[u8]) -> bool {
    let Some(rest) = header.strip_prefix(SECTION_PREFIX) else {
        return false;
    };
    let Some((from, after_from)) = take_quoted(rest) else {
        let mut expected: Vec<u8> = Vec::from(&b"a/"[..]);
        expected.extend_from_slice(name);
        expected.extend_from_slice(b" b/");
        expected.extend_from_slice(name);
        return rest == expected.as_slice();
    };
    let Some((to, after_to)) = after_from.strip_prefix(b" ").and_then(take_quoted) else {
        return false;
    };
    let mut expected_from: Vec<u8> = Vec::from(&b"a/"[..]);
    expected_from.extend_from_slice(name);
    let mut expected_to: Vec<u8> = Vec::from(&b"b/"[..]);
    expected_to.extend_from_slice(name);
    return after_to.is_empty() && from == expected_from && to == expected_to;
}

/// What:
///  Whether one of the extended header lines is `rename to` and then `name`.
///  `&[&[u8]]` borrows a list of borrowed lines.
/// Why:
///   `section_is_for` asks it of the lines after a section header,
/// and a loop with an early return stands where
///  a closure given to `any` stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listsRenameTo = (extended: string[], name) => extended.some((line) => printedPath(line.slice('rename to '.length)) == name);
/// ```
fn lists_rename_to(extended: &[&[u8]], name: &[u8]) -> bool {
    for line in extended {
        let Some(printed) = line.strip_prefix(b"rename to ").and_then(printed_path) else {
            continue;
        };
        if printed.as_slice() == name {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a patch line starts a hunk or starts the next file section.
///  `&&[u8]` is a borrow of an element of a list of borrowed lines,
/// as `position` hands it over.
/// Why:
///   The extended header lines of a section end at the first such line,
/// and `position` takes this function by path
///  where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isHunkOrSectionLine = (line: string) => line.startsWith(HUNK_PREFIX) || line.startsWith(SECTION_PREFIX);
/// ```
fn is_hunk_or_section_line(line: &&[u8]) -> bool {
    return line.starts_with(HUNK_PREFIX) || line.starts_with(SECTION_PREFIX);
}

/// What:
///  Whether `header` and the extended lines after it describe `name` as the section's new path:
/// either both sides
///  name it,
/// or a `rename to` line does.
/// Why:
///   A renamed file is excerpted under its new name,
/// like any other changed file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// headerIsFor(header, name.bytes) || extended.some((line) => renameNames(line, name.bytes))
/// ```
fn section_is_for(header: &[u8], extended: &[&[u8]], name: &[u8]) -> bool {
    return header_is_for(header, name) || lists_rename_to(extended, name);
}

/// What:
///  Parse the new side of `@@ -a[,b] +c[,d] @@` into the first new line the hunk consumes.
/// Why:
///   A zero new length means the hunk sits after line `c`,
/// so the first new line it reaches is `c + 1`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const m = /^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? /.exec(line);
/// ```
fn parse_new_start(line: &[u8]) -> Option<u64> {
    let text: &str = std::str::from_utf8(line).ok()?;
    let mut sides = text.strip_prefix("@@ -")?.split(' ');
    let _old = sides.next()?;
    let new: &str = sides.next()?.strip_prefix('+')?;
    let (start_text, length_text) = new.split_once(',').unwrap_or((new, "1"));
    let start: u64 = start_text.parse::<u64>().ok()?;
    let length: u64 = length_text.parse::<u64>().ok()?;
    return Some(if length == 0 { start + 1 } else { start });
}

/// What:
///  The new-side spans of `name` in `patch_lines`,
/// in file order;
/// empty when the patch holds no section for the path
///  or a section without hunks (a binary file,
/// a mode change or a pure rename).
/// Why:
///   These replace the script's per-file `git diff -U0` start,
/// and `standards_review_patch_tests.rs` checks them
///   against that command's new-side headers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function newSpans({ patchLines, name }): { first: number; last: number }[];
/// ```
pub fn new_spans(patch_lines: &[&[u8]], name: &[u8]) -> Vec<NewSpan> {
    let mut spans: Vec<NewSpan> = Vec::new();
    let mut index: usize = 0;
    while index < patch_lines.len() {
        let header: &[u8] = patch_lines[index];
        index += 1;
        if !header.starts_with(SECTION_PREFIX) {
            continue;
        }
        let first_hunk: usize = match patch_lines[index..].iter().position(is_hunk_or_section_line) {
            Some(offset) => index + offset,
            None => patch_lines.len(),
        };
        let matches: bool = section_is_for(header, &patch_lines[index..first_hunk], name);
        index = first_hunk;
        if matches {
            read_hunks(patch_lines, &mut index, &mut spans);
            return spans;
        }
    }
    return spans;
}

/// What:
///  Walk the hunks that start at `index` until the next section,
/// appending one span per run of changed lines.
/// Why:
///   Context lines separate runs,
/// and each run is what a zero-context diff would print as one hunk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readHunks({ patchLines, index, spans }): void;
/// ```
fn read_hunks(patch_lines: &[&[u8]], index: &mut usize, spans: &mut Vec<NewSpan>) {
    let mut new_line: u64 = 0;
    let mut open: Option<NewSpan> = None;
    while *index < patch_lines.len() {
        let line: &[u8] = patch_lines[*index];
        if line.starts_with(SECTION_PREFIX) {
            break;
        }
        *index += 1;
        if line.starts_with(HUNK_PREFIX) {
            close_run(&mut open, spans);
            new_line = parse_new_start(line).unwrap_or(new_line);
            continue;
        }
        match line.first() {
            Some(b'-') => {
                open = Some(open.unwrap_or(NewSpan { first: new_line, last: new_line }));
            }
            Some(b'+') => {
                let run: NewSpan = open.unwrap_or(NewSpan { first: new_line, last: new_line });
                open = Some(NewSpan { first: run.first, last: new_line });
                new_line += 1;
            }
            Some(b' ') => {
                close_run(&mut open, spans);
                new_line += 1;
            }
            _ => {}
        }
    }
    close_run(&mut open, spans);
}

/// What:
///  Move an open run,
/// if any,
/// into the finished spans.
/// Why:
///   Both a context line and the end of a hunk finish a run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (open !== undefined) { spans.push(open); open = undefined; }
/// ```
fn close_run(open: &mut Option<NewSpan>, spans: &mut Vec<NewSpan>) {
    if let Some(run) = open.take() {
        spans.push(run);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_patch_tests.rs"]
mod tests;
