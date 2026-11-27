//! What:
//!  The words a finish of a stopped operation reads:
//!  the `branch.<name>.mergeoptions` value split the way Git splits it,
//!  the command that ends the operation the Git directory shows in progress,
//!  and the commit message without the conflict hint Git appended when it stopped.
//! Why:
//!  A merge or a pull applies that setting to the merge it runs,
//!  so the finish applies the same words to the options it reads,
//!  and a stop appends a blank line,
//!  a `Conflicts:` line and one line per conflicted path
//!  (`append_conflicts_hint`, `sequencer.c` lines 722 to 745),
//!  which a clean run's message never holds.
//!  A pull runs either a merge or a rebase as a child (`builtin/pull.c` lines 525 and 750),
//!  so the word `pull` alone names no ending command;
//!  the state files Git writes do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const options = splitCmdline('--no-commit -m "a message"'); // ['--no-commit', '-m', 'a message']
//! const ending = endingCommandOf(gitDir); // '`git rebase --continue`' | undefined
//! const message = withoutConflictsHint(bytes);
//! ```

use std::ffi::OsString;
use std::path::Path;
/// What:
///  The words of a `branch.<name>.mergeoptions` value,
/// split the way Git splits it.
/// Why:
///   `parse_branch_merge_options` (`builtin/merge.c` line 641) hands the value to `split_cmdline` (`alias.c` lines 126 to 180):
///  white space separates words,
/// single and double quotes group (a backslash escapes inside double quotes and outside quotes),
///  and an unclosed quote or a trailing backslash is an error,
/// which Git dies on and this reads as "not read".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitCmdline(text: string): string[] | undefined;
/// ```
pub fn split_cmdline(text: &str) -> Option<Vec<OsString>> {
    let mut words: Vec<String> = Vec::new();
    let mut current: String = String::new();
    let mut started: bool = false;
    let mut quoted: Option<char> = None;
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if quoted.is_none() && character.is_whitespace() {
            if started {
                words.push(std::mem::take(&mut current));
                started = false;
            }
        } else if quoted.is_none() && (character == '\'' || character == '"') {
            quoted = Some(character);
            started = true;
        } else if Some(character) == quoted {
            quoted = None;
        } else {
            let mut kept: char = character;
            if character == '\\' && quoted != Some('\'') {
                kept = characters.next()?;
            }
            current.push(kept);
            started = true;
        }
    }
    if quoted.is_some() {
        return None;
    }
    if started {
        words.push(current);
    }
    return Some(words.into_iter().map(OsString::from).collect());
}
/// What:
///  The command that ends the operation the Git directory shows in progress,
/// or nothing when no state file names one.
/// Why:
///   The words after a finish that did not end the operation must name a command that works for it,
/// and which operation it is shows
///  in the state files Git writes:
/// a `git pull` runs either a merge or a rebase as a child (`builtin/pull.c` lines 525 and 750),
/// so the
///  word `pull` alone names no ending command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endingCommandOf(gitDir: string): string | undefined;
/// ```
pub fn ending_command_of(git_dir: &Path) -> Option<&'static str> {
    if git_dir.join("rebase-merge").is_dir() || git_dir.join("rebase-apply").is_dir() {
        return Some("`git rebase --continue`");
    }
    if git_dir.join("CHERRY_PICK_HEAD").exists() {
        return Some("`git cherry-pick --continue`");
    }
    if git_dir.join("REVERT_HEAD").exists() {
        return Some("`git revert --continue`");
    }
    if git_dir.join("MERGE_HEAD").exists() {
        return Some("`git commit`");
    }
    return None;
}
/// Whether one byte is the line feed that ends a line of a message.
/// `&u8` borrows the byte,
/// as `split_inclusive` hands it over.
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  `message` without the conflict hint Git appended when it stopped,
/// so it reads as the message of a clean run.
/// Why:
///   A stop appends a blank line,
/// a comment line `Conflicts:` and one comment line with a tab and a path per conflicted file
///  (`append_conflicts_hint`,
/// `sequencer.c` lines 722 to 745).
/// A clean run without an editor keeps the message as prepared,
/// so the
///  hint is cut here;
/// the comment string is whatever precedes `Conflicts:` on the hint's line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withoutConflictsHint(message: Uint8Array): Uint8Array;
/// ```
pub fn without_conflicts_hint(message: &[u8]) -> Vec<u8> {
    let lines: Vec<&[u8]> = message.split_inclusive(is_line_feed_byte).collect();
    let mut start: usize = lines.len();
    let mut comment: Option<&[u8]> = None;
    // Walk back over the path lines, then expect the `Conflicts:` line.
    while start > 0 {
        let line: &[u8] = lines[start - 1]
            .strip_suffix(b"\n")
            .unwrap_or(lines[start - 1]);
        if let Some(prefix) = line.strip_suffix(b"Conflicts:") {
            let same_comment: bool = match comment {
                Some(known) => known == prefix,
                None => true,
            };
            if same_comment && !prefix.is_empty() {
                comment = Some(prefix);
                start -= 1;
                break;
            }
            return message.to_vec();
        }
        let path_line: Option<&[u8]> = match comment {
            Some(known) => line.strip_prefix(known),
            None => Some(line),
        };
        let Some(tab_path) = path_line else {
            return message.to_vec();
        };
        if comment.is_none() && !tab_path.contains(&b'\t') {
            return message.to_vec();
        }
        start -= 1;
    }
    if comment.is_none() {
        return message.to_vec();
    }
    // The blank line the hint starts with goes too.
    if start > 0 && lines[start - 1] == b"\n" {
        start -= 1;
    }
    return lines[..start].concat();
}
