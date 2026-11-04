//! What:
//!  Reading one file's changed new-side line spans out of a `git show --patch` text, so the review needs no
//!  `git diff -U0` start per excerpted file.
//! Why:
//!  The script runs `git diff -U0 <sha>^ <sha> -- <name>` for each of up to eight files. A hunk of the three-line-context
//!  patch is a list of change runs, and each run is one zero-context hunk, so the same spans follow from the patch the
//!  review already holds. The spans are on the new side because the excerpt is cut from the new-side file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const match of git('diff', '-U0', `${sha}^`, sha, '--', name).matchAll(/^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? /gm)) { ... }
//! ```

/// A diff section's opening line prefix.
const SECTION_PREFIX: &str = "diff --git ";

/// A hunk header's opening characters.
const HUNK_PREFIX: &str = "@@ ";

/// What:
///  One run of changed lines on the new side of a file: the first new line the change touches and the last.
///  A pure deletion touches the new line that now stands where the removed lines were.
/// Why:
///   The excerpt window is the union of these spans over the file's runs, cut from the new-side file the commit
///   produced. The script applied the old-side numbers to the new-side file, which shifts the window after an earlier
///   insertion; this port uses the new side on both ends.
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
///  Whether `header` and the extended lines after it describe `name` as the section's new path: either both sides
///  name it, or a `rename to` line does.
/// Why:
///   A renamed file is excerpted under its new name, like any other changed file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// header === `diff --git a/${name} b/${name}` || extended.includes(`rename to ${name}`)
/// ```
fn section_is_for(header: &str, extended: &[&str], name: &str) -> bool {
    let rename: String = format!("rename to {name}");
    return header == format!("{SECTION_PREFIX}a/{name} b/{name}") || extended.iter().any(|line| return *line == rename);
}

/// What:
///  Parse the new side of `@@ -a[,b] +c[,d] @@` into the first new line the hunk consumes.
/// Why:
///   A zero new length means the hunk sits after line `c`, so the first new line it reaches is `c + 1`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const m = /^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? /.exec(line);
/// ```
fn parse_new_start(line: &str) -> Option<u64> {
    let mut sides = line.strip_prefix("@@ -")?.split(' ');
    let _old = sides.next()?;
    let new: &str = sides.next()?.strip_prefix('+')?;
    let (start_text, length_text) = new.split_once(',').unwrap_or((new, "1"));
    let start: u64 = start_text.parse::<u64>().ok()?;
    let length: u64 = length_text.parse::<u64>().ok()?;
    return Some(if length == 0 { start + 1 } else { start });
}

/// What:
///  The new-side spans of `name` in `patch_lines`, in file order; empty when the patch holds no section for the path
///  or a section without hunks (a binary file, a mode change or a pure rename).
/// Why:
///   These replace the script's per-file `git diff -U0` start, and `standards_review_patch_tests.rs` checks them
///   against that command's new-side headers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function newSpans({ patchLines, name }): { first: number; last: number }[];
/// ```
pub fn new_spans(patch_lines: &[&str], name: &str) -> Vec<NewSpan> {
    let mut spans: Vec<NewSpan> = Vec::new();
    let mut index: usize = 0;
    while index < patch_lines.len() {
        let header: &str = patch_lines[index];
        index += 1;
        if !header.starts_with(SECTION_PREFIX) {
            continue;
        }
        let first_hunk: usize = patch_lines[index..]
            .iter()
            .position(|line| return line.starts_with(HUNK_PREFIX) || line.starts_with(SECTION_PREFIX))
            .map_or(patch_lines.len(), |offset| return index + offset);
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
///  Walk the hunks that start at `index` until the next section, appending one span per run of changed lines.
/// Why:
///   Context lines separate runs, and each run is what a zero-context diff would print as one hunk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readHunks({ patchLines, index, spans }): void;
/// ```
fn read_hunks(patch_lines: &[&str], index: &mut usize, spans: &mut Vec<NewSpan>) {
    let mut new_line: u64 = 0;
    let mut open: Option<NewSpan> = None;
    while *index < patch_lines.len() {
        let line: &str = patch_lines[*index];
        if line.starts_with(SECTION_PREFIX) {
            break;
        }
        *index += 1;
        if line.starts_with(HUNK_PREFIX) {
            close_run(&mut open, spans);
            new_line = parse_new_start(line).unwrap_or(new_line);
            continue;
        }
        match line.as_bytes().first() {
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
///  Move an open run, if any, into the finished spans.
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
