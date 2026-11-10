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

use std::io::Write as _;
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
/// The line is appended and the file is never opened for truncation:
/// the bytes the person wrote are not rewritten,
///  not even when the call fails,
/// so a full disk,
/// a quota,
/// or a kill after the open cannot lose an exclude rule that another
///  file's staging depends on.
/// The file's bytes are read and compared as bytes,
/// because a pattern that is not UTF-8 is a pattern
///  Git honours,
/// and a text round trip replaces the bytes that make it work with others that spell nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ensureTrashIsExcluded = (common) => { const path = join(common, 'info/exclude'); const bytes = readOrEmpty(path); if (alreadyThere(bytes)) return false; appendFileSync(path, (bytes.length && bytes.at(-1) !== 0x0a ? '\n' : '') + LINE + '\n'); return true; };
/// ```
pub fn ensure_trash_is_excluded(common_dir: &Path) -> Result<bool, String> {
    let path: PathBuf = common_dir.join("info").join("exclude");
    let existing: Vec<u8> = match std::fs::read(path.as_path()) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(format!("{} cannot be read ({error}), so nothing says whether a trash in this repository would be staged", path.display())),
    };
    if already_excluded(existing.as_slice()) {
        return Ok(false);
    }
    let mut adding: Vec<u8> = Vec::new();
    if !existing.is_empty() && !existing.ends_with(b"\n") {
        adding.push(b'\n');
    }
    adding.extend_from_slice(TRASH_EXCLUDE_LINE.as_bytes());
    adding.push(b'\n');
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        return Err(format!("{} cannot be made ({error}), so the trash line cannot be written", parent.display()));
    }
    // The file is opened for appending and for nothing else: a write that fails or is killed leaves every byte that was there
    //  where it was, and the worst a half-written addition can leave is a truncated pattern of the wrapper's own that ignores
    //  nothing.
    let mut opened: std::fs::File = match std::fs::OpenOptions::new().append(true).create(true).open(path.as_path()) {
        Ok(file) => file,
        Err(error) => return Err(format!("{} cannot be written ({error}), so nothing says whether a trash in this repository would be staged", path.display())),
    };
    return match opened.write_all(adding.as_slice()) {
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
/// never of the file's text as one blob,
/// and of the bytes,
/// because a pattern that
///  is not text is one of the person's lines too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const alreadyExcluded = (bytes) => bytes.toString('latin1').split('\n').map((line) => line.trim()).includes(TRASH_EXCLUDE_LINE);
/// ```
fn already_excluded(bytes: &[u8]) -> bool {
    return bytes.split(|byte: &u8| return *byte == b'\n').any(|line: &[u8]| return line.trim_ascii() == TRASH_EXCLUDE_LINE.as_bytes());
}

/// Exclude-line controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_exclude_line_tests.rs"]
mod tests;
