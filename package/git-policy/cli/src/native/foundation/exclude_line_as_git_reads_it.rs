//! What:
//!  One line of a repository's exclude file as Git reads it as a pattern.
//! Why:
//!   The wrapper both writes and recognises lines in `$GIT_COMMON_DIR/info/exclude`,
//! and a line it takes for its own while Git reads something else is a line that ignores nothing:
//! the trash,
//! or the staging copy of a build directory,
//! then shows in `git status` and can be staged.
//! Git's own rule is narrower than "the spaces around a line":
//! the line ending is stripped,
//! and so are the trailing spaces of a line,
//! except a space that a backslash escapes;
//! leading spaces and tabs are part of the pattern,
//! and a trailing tab is part of it too.
//! Measured with Git 2.56.0 on this host on 2026-11-13,
//! with `git check-ignore --no-index --verbose` on each form of `/.claude/cli-git-trash/`:
//! a leading space or tab,
//! a trailing tab,
//! a tab before or after a trailing space,
//! and an escaped trailing space all leave the trash ignored by nothing,
//! while trailing spaces,
//! a carriage return before the line end and a missing line end leave it ignored.
//! The rule below is Git's `trim_trailing_spaces`, applied to the line with its ending removed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git's dir.c: trim_trailing_spaces(), after the line reader dropped the ending
//! const thePatternGitReads = (line) => { const bytes = stripTheLineEnding(line); let lastSpace = -1; for (let at = 0; at < bytes.length; at += 1) { if (bytes[at] === 0x20) { if (lastSpace < 0) lastSpace = at; } else if (bytes[at] === 0x5c) { at += 1; if (at >= bytes.length) return bytes; lastSpace = -1; } else { lastSpace = -1; } } return lastSpace < 0 ? bytes : bytes.subarray(0, lastSpace); };
//! ```

/// What:
///  The bytes of one line of an exclude file with Git's line ending removed:
///  its `\n`,
///  and the `\r` before it,
///  when the caller handed the line in with either.
/// Why:
///   Git reads its exclude file through a line reader that drops both,
/// so a carriage return a person's editor left is not part of the pattern.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutTheEnding = (line) => line.at(-1) === 0x0a ? line.subarray(0, line.at(-2) === 0x0d ? -2 : -1) : line;
/// ```
fn without_the_line_ending(line: &[u8]) -> &[u8] {
    let mut end: usize = line.len();
    if end > 0 && line[end - 1] == b'\n' {
        end -= 1;
    }
    if end > 0 && line[end - 1] == b'\r' {
        end -= 1;
    }
    return &line[..end];
}

/// What:
///  The pattern Git reads out of one line of an exclude file.
/// Why:
///   Git strips the trailing spaces of a pattern,
/// never the leading ones and never a tab,
/// and never a space a backslash escapes,
/// so a caller that compared a trimmed line would take a line Git does not ignore for its own
/// and append nothing.
pub(crate) fn the_pattern_git_reads(line: &[u8]) -> &[u8] {
    let bytes: &[u8] = without_the_line_ending(line);
    let mut trailing_space: Option<usize> = None;
    let mut at: usize = 0;
    while at < bytes.len() {
        match bytes[at] {
            b' ' => {
                if trailing_space.is_none() {
                    trailing_space = Some(at);
                }
            }
            b'\\' => {
                // A backslash takes the next byte with it, and there is nothing left to strip when it ends the line.
                at += 1;
                if at >= bytes.len() {
                    return bytes;
                }
                trailing_space = None;
            }
            _ => trailing_space = None,
        }
        at += 1;
    }
    return match trailing_space {
        Some(first) => &bytes[..first],
        None => bytes,
    };
}

#[cfg(test)]
#[path = "exclude_line_as_git_reads_it_tests.rs"]
mod tests;
