//! What:
//!  Reading one file's changed old-side line ranges out of a `git show --patch` text, so the review needs no
//!  `git diff -U0` start per excerpted file.
//! Why:
//!  The script runs `git diff -U0 <sha>^ <sha> -- <name>` for each of up to eight files and reads the old-side
//!  `@@ -start[,length]` headers. A hunk of the three-line-context patch is a list of change runs, and each run is one
//!  zero-context hunk, so the same ranges follow from the patch the review already holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const match of git('diff', '-U0', `${sha}^`, sha, '--', name).matchAll(/^@@ -(\d+)(?:,(\d+))? /gm)) { ... }
//! ```

/// A diff section's opening line prefix.
const SECTION_PREFIX: &str = "diff --git ";

/// A hunk header's opening characters.
const HUNK_PREFIX: &str = "@@ ";

/// What:
///  One zero-context hunk's old side: the line before the change for a pure insertion (which Git prints as
///  `-N,0`), otherwise the first removed line, and how many old lines the change removes.
/// Why:
///   The excerpt window is the union of `start` to `start + length` over the file's hunks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OldRange = { start: number; length: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OldRange {
    /// First old line of the change, or the line before an insertion.
    pub start: u64,
    /// Removed old lines; zero for a pure insertion.
    pub length: u64,
}

/// What:
///  Whether the diff section opened by `header` and the lines that follow it describes `name` as its new path.
/// Why:
///   A section whose header names the same path on both sides is an in-place change; a rename section names the new
///   path in its `rename to` line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// header === `diff --git a/${name} b/${name}` || section.includes(`rename to ${name}`)
/// ```
fn section_is_for(header: &str, extended: &[&str], name: &str) -> (bool, bool) {
    let same: bool = header == format!("{SECTION_PREFIX}a/{name} b/{name}");
    let rename: String = format!("rename to {name}");
    let renamed: bool = extended.iter().any(|line| return *line == rename);
    return (same || renamed, renamed);
}

/// What:
///  Parse `@@ -a[,b] +c[,d] @@` into the next old line to consume and the old length.
/// Why:
///   A zero old length means the hunk sits after line `a`, so the first old line it consumes is `a + 1`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const m = /^@@ -(\d+)(?:,(\d+))? /.exec(line);
/// ```
fn parse_old_side(line: &str) -> Option<(u64, u64)> {
    let rest: &str = line.strip_prefix("@@ -")?;
    let old: &str = rest.split(' ').next()?;
    let (start_text, length_text) = old.split_once(',').unwrap_or((old, "1"));
    let start: u64 = start_text.parse::<u64>().ok()?;
    let length: u64 = length_text.parse::<u64>().ok()?;
    return Some((start, length));
}

/// What:
///  The zero-context old ranges of `name` in `patch_lines`, in file order; empty when the patch holds no section for
///  the path, or a section without hunks (a binary file or a mode change).
///  A renamed file yields the one range `-0,0`, because `git diff -U0 <sha>^ <sha> -- <new name>` sees an addition.
/// Why:
///   These ranges replace the script's per-file `git diff -U0` start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function oldRanges({ patchLines, name }): { start: number; length: number }[];
/// ```
pub fn old_ranges(patch_lines: &[&str], name: &str) -> Vec<OldRange> {
    let mut ranges: Vec<OldRange> = Vec::new();
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
        let (matches, renamed) = section_is_for(header, &patch_lines[index..first_hunk], name);
        if !matches {
            index = first_hunk;
            continue;
        }
        if renamed {
            return vec![OldRange { start: 0, length: 0 }];
        }
        index = first_hunk;
        read_hunks(patch_lines, &mut index, &mut ranges);
        return ranges;
    }
    return ranges;
}

/// What:
///  Walk the hunks that start at `index` until the next section, appending one range per run of changed lines.
/// Why:
///   Context lines separate runs, and each run is what a zero-context diff would print as one hunk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readHunks({ patchLines, index, ranges }): void;
/// ```
fn read_hunks(patch_lines: &[&str], index: &mut usize, ranges: &mut Vec<OldRange>) {
    let mut old_line: u64 = 0;
    let mut open: Option<OldRange> = None;
    while *index < patch_lines.len() {
        let line: &str = patch_lines[*index];
        if line.starts_with(SECTION_PREFIX) {
            break;
        }
        *index += 1;
        if line.starts_with(HUNK_PREFIX) {
            close_run(&mut open, ranges);
            let Some((start, length)) = parse_old_side(line) else {
                continue;
            };
            old_line = if length == 0 { start + 1 } else { start };
            continue;
        }
        match line.as_bytes().first() {
            Some(b'-') => {
                let run: OldRange = open.unwrap_or(OldRange { start: old_line, length: 0 });
                open = Some(OldRange { start: run.start, length: run.length + 1 });
                old_line += 1;
            }
            Some(b'+') => {
                open = Some(open.unwrap_or(OldRange { start: old_line.saturating_sub(1), length: 0 }));
            }
            Some(b' ') => {
                close_run(&mut open, ranges);
                old_line += 1;
            }
            _ => {}
        }
    }
    close_run(&mut open, ranges);
}

/// What:
///  Move an open run, if any, into the finished ranges.
/// Why:
///   Both a context line and the end of a hunk finish a run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (open !== undefined) { ranges.push(open); open = undefined; }
/// ```
fn close_run(open: &mut Option<OldRange>, ranges: &mut Vec<OldRange>) {
    if let Some(run) = open.take() {
        ranges.push(run);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_patch_tests.rs"]
mod tests;
