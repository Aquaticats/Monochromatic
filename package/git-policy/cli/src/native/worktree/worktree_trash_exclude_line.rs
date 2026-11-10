//! What:
//!  The repository's own exclude line that keeps a standing trash out of every
//! `git add --all`,
//!  whatever the target commit ignores.
//! Why:
//!  The add leaves the directories it set aside in
//! `.claude/cli-git-trash/<id>` inside the new worktree until the commit service empties
//!  them,
//! and one of them can be another command's stage container with credentials in it.
//! A target commit that stops
//!  ignoring `.claude/` would make that trash untracked,
//! so the person's next `git add --all` would stage it.
//! The ignore rules
//!  of a commit are the wrong place to decide that,
//! because a commit is what changes them;
//! `$GIT_COMMON_DIR/info/exclude` is
//!  not written by any commit,
//! and Git reads it for every worktree of the repository.
//! Measured with Git 2.56.0
//!  on 2026-11-11
//! (`doc/planning/cli-git-worktree-add-cost-of-its-trash.md`):
//! in a linked worktree,
//!  a line in the common `info/exclude` makes `git check-ignore` name that file and `git add
//! --all` stages nothing of the trash,
//! while a line in the worktree's own `info/exclude` is not read at all;
//! a target commit's
//!  `.gitignore` cannot re-include a path below a directory the line excludes,
//! so a negation there does not open the trash again.
//! The
//!  line is added once and never removed:
//! the directory is the wrapper's own,
//! and a person who wants their `.claude/cli-git-trash` tracked
//!  has no trash to track.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const added = ensureTrashIsExcluded(commonDir); // append '/.claude/cli-git-trash/' when it is not there
//! ```

use std::path::{Path, PathBuf};

/// What:
///  The line that keeps the trash parent,
/// and everything below it,
/// out of every commit.
/// Why:
///   A leading slash anchors it at the top of the worktree,
/// so a directory of that name deeper down is not what the
///  wrapper leaves out,
/// and the trailing slash says the entry is a directory,
/// which is all a trash is.
/// The text is the one measured
///  against real Git in the planning document.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TRASH_EXCLUDE_LINE = '/.claude/cli-git-trash/';
/// ```
pub const TRASH_EXCLUDE_LINE: &str = "/.claude/cli-git-trash/";

/// What:
///  Add the wrapper's exclude line to the repository's `info/exclude` when it is not there;
/// whether it had to be
///  added,
/// and the reason when the file could not be read or written.
/// Why:
///   One line per repository decides whether a trash standing in any of its worktrees is
/// stageable,
/// so
///  the write happens before a snapshot makes a trash at all.
/// Nothing else in the file is touched:
/// the bytes the person
///  put there are kept exactly,
/// and a file that does not end in a newline is closed with one so the added line
///  is a line of its own.
/// A file that exists and cannot be read is a refusal rather than an append,
/// because appending
///  to bytes nobody could read could write the line twice or in the middle of a pattern.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ensureTrashIsExcluded = (common) => { const path = join(common, 'info/exclude'); const text = readOrEmpty(path); if (text.split('\n').includes(LINE)) return false; write(path, text + (text && !text.endsWith('\n') ? '\n' : '') + LINE + '\n'); return true; };
/// ```
pub fn ensure_trash_is_excluded(common_dir: &Path) -> Result<bool, String> {
    let path: PathBuf = common_dir.join("info").join("exclude");
    let existing: String = match std::fs::read(path.as_path()) {
        Ok(bytes) => String::from_utf8_lossy(bytes.as_slice()).into_owned(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("{} cannot be read ({error}), so nothing says whether a trash in this repository would be staged", path.display())),
    };
    if already_excluded(existing.as_str()) {
        return Ok(false);
    }
    let mut writing: String = existing;
    if !writing.is_empty() && !writing.ends_with('\n') {
        writing.push('\n');
    }
    writing.push_str(TRASH_EXCLUDE_LINE);
    writing.push('\n');
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        return Err(format!("{} cannot be made ({error}), so the trash line cannot be written", parent.display()));
    }
    return match std::fs::write(path.as_path(), writing.as_bytes()) {
        Ok(()) => Ok(true),
        Err(error) => Err(format!("{} cannot be written ({error}), so nothing says whether a trash in this repository would be staged", path.display())),
    };
}

/// What:
///  Whether the file's lines already hold the wrapper's own.
/// Why:
///   A blank line,
/// a comment beside it and a longer pattern that begins with it are not the line,
/// so the answer is
///  asked of whole lines,
/// never of the file's text as one blob.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alreadyExcluded = (text) => text.split('\n').map((line) => line.trim()).includes(TRASH_EXCLUDE_LINE);
/// ```
fn already_excluded(text: &str) -> bool {
    return text.lines().any(|line: &str| return line.trim() == TRASH_EXCLUDE_LINE);
}

/// Exclude-line controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_exclude_line_tests.rs"]
mod tests;
