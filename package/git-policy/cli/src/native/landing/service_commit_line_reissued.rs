//! What:
//!  The command line a commit Git built is built again with once its editor has run and the branch moved:
//!  the line without the options whose content the preparation already settled.
//! Why:
//!  A commit that took its message and authorship from another commit
//!  (`-C`,
//! `-c`),
//!  or that named a fixup commit,
//!  was refused on a moved branch because nothing could be made of the line alone
//! (`doc/decision/cli-git-concurrent-commits.md`,
//! "Serial landing",
//! and the owner's rule "Commits should never fail.").
//!  Once the editor has run,
//!  the message is a file and the authorship is in the commit Git built,
//!  and the selection `--patch` or `--interactive` made is in the private index,
//!  so the rebuild reads all three instead of asking the line again:
//!  `-F <file>` for the message,
//!  `--author` and `--date` for a borrowed authorship,
//!  and the private index for the selection.
//!  A `--fixup=reword:` commit is the one line a rebuild cannot read back:
//!  Git throws the index away for it,
//!  so its commit is the branch's own tree and a replay of the captured tree would land changes the person did not choose.
//!  That commit is not refused for it:
//!  `rewords_a_commit` tells the commit loop that the line asks for an empty commit,
//!  and the commit a moved branch makes of it is written from the fields of the commit Git built
//!  (`service_commit.rs`,
//!  the arm for a branch that moved).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = await reissuedArguments({ context, cwd, arguments, subcommand, file, prepared, interactive });
//! ```

use crate::commit_args::{normalise_commit_args, takes_next_token};
use crate::commit_transaction::TransactionStop;
use crate::path_bytes::{argument_bytes, os_from_path_bytes};
use crate::service_amend::split_author;
use crate::service_amend_replayed::stored_commit_fields;
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// The token that ends the options of a commit region.
const SEPARATOR: &str = "--";

/// Short option letters a rebuilt line drops:
/// the selection the private index already holds.
const SELECTION_LETTERS: &[u8] = b"pi";

/// Short option letters whose value is attached,
/// so that what follows is never read as a flag.
const ATTACHED_VALUE_LETTERS: &[u8] = b"Su";

/// The authorship a commit reused from another commit,
/// which a rebuild supplies itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BorrowedAuthor {
    /// The author as Git's `--author` reads it,
    /// `Name <email>`.
    pub(crate) author: String,
    /// The author date in the spelling Git's `--date` reads,
    /// `seconds zone`.
    pub(crate) date: String,
}

/// What:
///  Whether one option takes the message and the authorship of the commit it names.
/// Why:
///   `-c`,
/// `-C`,
/// `--reuse-message` and `--reedit-message` take both,
///  so a rebuild that keeps the message in a file must supply the authorship itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const borrows = (token) => ['-c', '-C', '--reuse-message', '--reedit-message'].includes(nameOf(token));
/// ```
fn borrows_its_author(token: &OsStr) -> bool {
    return matches!(name_of(token), Some("-c" | "-C" | "--reuse-message" | "--reedit-message"));
}

/// The name half of an option split at the `=` that attached its value.
fn name_before_the_value<'a>(halves: (&'a str, &'a str)) -> &'a str {
    return halves.0;
}

/// What:
///  The name of an option,
/// without the value an `=` attached to it,
/// and nothing for a token that is not text.
/// Why:
///   An option's spelling is text,
/// so a token holding a byte that is not UTF-8 is no option of this list,
///  and it is kept as it is rather than respelled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = token.toString()?.split('=')[0]
/// ```
fn name_of(token: &OsStr) -> Option<&str> {
    let text: &str = token.to_str()?;
    return Some(text.split_once('=').map_or(text, name_before_the_value));
}

/// What:
///  Whether one option names the one fixup whose commit is not the private index.
/// Why:
///   `--fixup=reword:<commit>` throws the index away and makes a commit of the branch's own tree,
///  so the tree this preparation captured is not the tree that command would commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rewords = (token) => /^--(fixup|squash)=reword(:|$)/.test(token);
/// ```
fn rewords_another_commit(token: &OsStr) -> bool {
    let Some(text) = token.to_str() else { return false };
    let named: Option<&str> = match text.strip_prefix("--fixup=") {
        Some(value) => Some(value),
        None => text.strip_prefix("--squash="),
    };
    let Some(value) = named else { return false; };
    return value == "reword" || value.starts_with("reword:");
}

/// What:
///  The letters of a short cluster without the selection letters,
///  nothing when none is left.
/// Why:
///   The private index holds the selection `--patch` or `--interactive` made,
///  and a line holding `-p` again would ask the same question a second time at a terminal that is no longer there.
///  A letter with an attached value ends the cluster:
///  what follows it is that value and never a flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withoutSelection = (token) => [...token].filter((letter) => letter !== 'p' && letter !== 'i').join('');
/// ```
fn without_selection_letters(token: &OsStr) -> Option<OsString> {
    let bytes: &[u8] = argument_bytes(token);
    let letters: &[u8] = &bytes[1..];
    let mut kept: Vec<u8> = vec![b'-'];
    let mut at: usize = 0;
    while at < letters.len() {
        let letter: u8 = letters[at];
        at += 1;
        if ATTACHED_VALUE_LETTERS.contains(&letter) {
            kept.push(letter);
            kept.extend_from_slice(&letters[at..]);
            break;
        }
        if SELECTION_LETTERS.contains(&letter) {
            continue;
        }
        kept.push(letter);
    }
    return if kept == b"-" { None } else { Some(os_from_path_bytes(kept.as_slice())) };
}

/// What:
///  Whether the command line names the one fixup whose commit Git makes from the branch rather than from the index.
/// Why:
///   `--fixup=reword:<commit>` throws the index away,
///  so the commit it asks for carries the branch's own tree and the message the editor settled:
///  an empty commit is what the person asked for,
///  and the commit a moved branch makes of it is written from the fields of the commit Git built
///  (`service_commit.rs`,
///  the arm for a branch that moved).
///  The region is read the way the rebuild reads it,
///  so a `--fixup=reword:` that is the value of another option is not one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rewordsACommit(args, subcommand) // boolean
/// ```
pub(crate) fn rewords_a_commit(arguments: &[OsString], subcommand: usize) -> bool {
    let region: Vec<OsString> = normalise_commit_args(arguments.get(subcommand + 1..).unwrap_or(&[]));
    let mut state: State = State::Option;
    for token in &region {
        match state {
            State::Value | State::DroppedValue => {
                state = State::Option;
                continue;
            }
            State::Verbatim => continue,
            State::Option => {}
        }
        if token.as_os_str() == OsStr::new(SEPARATOR) {
            state = State::Verbatim;
            continue;
        }
        if rewords_another_commit(token.as_os_str()) {
            return true;
        }
        state = if takes_next_token(token.as_os_str()) { State::Value } else { State::Option };
    }
    return false;
}

/// What one token of the region leaves for the tokens after it.
/// Why:
///   The value of an option is the person's text and is never read as an option,
///  and a dropped option takes its value with it whether it was detached or attached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = 'option' | 'value' | 'dropped-value' | 'verbatim';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    /// The next token is an option.
    Option,
    /// The next token is the value of an option that stays.
    Value,
    /// The next token is the value of an option that goes.
    DroppedValue,
    /// Every token from here is a path and is copied as it is.
    Verbatim,
}

/// What a line that reads a settled authorship leaves behind.
/// Why:
///   The authorship a `-C` borrowed is read from the commit Git built only when the line needs it,
///  so the walk names that need instead of reading Git itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Settled = { kind: 'line'; tokens } | { kind: 'needs-the-authorship' } | { kind: 'cannot' };
/// ```
enum Settled {
    /// Everything the line keeps,
    /// each token with its own bytes.
    Line(Vec<OsString>),
    /// The line borrows an authorship the walk was not given.
    NeedsTheAuthorship,
    /// The line cannot be made again from what the preparation captured.
    Cannot,
}

/// What:
///  The options of the region as a rebuilt line keeps them.
/// Why:
///   The selection options and a reused authorship are what the captured bytes and the built commit already hold,
///  and a `reword:` fixup is the one command whose commit is not the captured tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// withoutSettledOptions({ region, interactive, borrowed });
/// ```
fn without_settled_options(region: &[OsString], interactive: bool, borrowed: Option<&BorrowedAuthor>) -> Settled {
    let mut kept: Vec<OsString> = Vec::new();
    let mut state: State = State::Option;
    for token in region {
        match state {
            State::Value => {
                kept.push(token.clone());
                state = State::Option;
                continue;
            }
            State::Verbatim => {
                kept.push(token.clone());
                continue;
            }
            State::DroppedValue => {
                state = State::Option;
                continue;
            }
            State::Option => {}
        }
        if token.as_os_str() == OsStr::new(SEPARATOR) {
            kept.push(token.clone());
            state = State::Verbatim;
            continue;
        }
        if rewords_another_commit(token.as_os_str()) {
            return Settled::Cannot;
        }
        let name: Option<&str> = name_of(token.as_os_str());
        if borrows_its_author(token.as_os_str()) {
            if borrowed.is_none() {
                return Settled::NeedsTheAuthorship;
            }
            state = dropped_after(token.as_os_str());
            continue;
        }
        if matches!(name, Some("--fixup" | "--squash")) {
            state = dropped_after(token.as_os_str());
            continue;
        }
        if matches!(name, Some("--author" | "--date")) && borrowed.is_some() {
            state = dropped_after(token.as_os_str());
            continue;
        }
        if interactive && matches!(name, Some("-p" | "-i" | "--patch" | "--interactive")) {
            state = dropped_after(token.as_os_str());
            continue;
        }
        let bytes: &[u8] = argument_bytes(token.as_os_str());
        if interactive && bytes.starts_with(b"-") && !bytes.starts_with(b"--") && bytes.len() > 1 {
            if let Some(cluster) = without_selection_letters(token.as_os_str()) {
                kept.push(cluster);
            }
            state = kept_after(token.as_os_str());
            continue;
        }
        kept.push(token.clone());
        state = kept_after(token.as_os_str());
    }
    return Settled::Line(kept);
}

/// What the walk reads after an option it keeps.
fn kept_after(token: &OsStr) -> State {
    return if takes_next_token(token) { State::Value } else { State::Option };
}

/// What the walk reads after an option it drops.
fn dropped_after(token: &OsStr) -> State {
    return if takes_next_token(token) { State::DroppedValue } else { State::Option };
}

/// What:
///  The author and author date of the commit a line reused,
///  read from the commit Git built for it.
/// Why:
///   `-C` and `-c` copy the authorship of the commit they name,
///  and no other record of what the line meant survives the build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const borrowed = await borrowedAuthorOf({ context, cwd, commit });
/// ```
///
/// # Errors
///
/// `TransactionStop::Message` when the commit cannot be read.
pub(crate) fn borrowed_author_of(context: &GitContext, cwd: &Path, commit: &str) -> Result<Option<BorrowedAuthor>, TransactionStop> {
    let read = run_git_checked(context, &GitRequest::new(cwd, &["cat-file", "commit", commit]))?;
    let Some(fields) = stored_commit_fields(read.stdout.as_slice()) else { return Ok(None) };
    let Some((name, email, date)) = split_author(fields.authored_by()) else { return Ok(None) };
    return Ok(Some(BorrowedAuthor { author: format!("{name} <{email}>"), date }));
}

/// What:
///  The command line the commit is built again with after its editor has run and the branch moved,
///  nothing when the line cannot be made again from what the preparation captured.
/// Why:
///   The message is the file the person's typing was saved in,
///  and everything else the line borrowed is read back from the commit Git built,
///  so the rebuild asks the person nothing and invents nothing.
///  A line whose selection is settled reads no `--patch`,
///  a line that borrowed its authorship reads `--author` and `--date` from that commit,
///  and a `reword:` fixup gives nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = await reissuedArguments({ context, cwd, arguments, subcommand, file, prepared, interactive });
/// ```
///
/// # Errors
///
/// `TransactionStop::Message` when the commit the line reused cannot be read back.
pub(crate) fn reissued_arguments(
    context: &GitContext,
    cwd: &Path,
    arguments: &[OsString],
    subcommand: usize,
    file: &Path,
    prepared: Option<&str>,
    interactive: bool,
) -> Result<Option<Vec<OsString>>, TransactionStop> {
    let region: Vec<OsString> = normalise_commit_args(arguments.get(subcommand + 1..).unwrap_or(&[]));
    let (mut rebuilt, borrowed): (Vec<OsString>, Option<BorrowedAuthor>) = match without_settled_options(region.as_slice(), interactive, None) {
        Settled::Line(kept) => (kept, None),
        Settled::Cannot => return Ok(None),
        Settled::NeedsTheAuthorship => {
            let Some(commit) = prepared else { return Ok(None) };
            let Some(found) = borrowed_author_of(context, cwd, commit)? else { return Ok(None) };
            match without_settled_options(region.as_slice(), interactive, Some(&found)) {
                Settled::Line(kept) => (kept, Some(found)),
                // A line that names its own author and a commit to reuse it from cannot be both.
                Settled::NeedsTheAuthorship | Settled::Cannot => return Ok(None),
            }
        }
    };
    // The authorship the line borrowed goes where the option that borrowed it was, and Git reads it from there.
    if let Some(found) = borrowed.as_ref() {
        rebuilt.insert(0, OsString::from(format!("--author={}", found.author)));
        rebuilt.insert(1, OsString::from(format!("--date={}", found.date)));
    }
    let mut whole: Vec<OsString> = arguments.get(..=subcommand).unwrap_or(&[]).to_vec();
    whole.extend(rebuilt);
    return Ok(crate::service_commit_args::with_fixed_message(whole.as_slice(), subcommand, file));
}

/// Controls for the line a moved branch rebuilds stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_line_reissued_tests.rs"]
mod tests;
