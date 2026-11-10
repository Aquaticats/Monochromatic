//! What:
//!  The command line a commit Git built is built again with once its editor has run and the branch moved:
//!  the line without the options whose content the preparation already settled.
//! Why:
//!  A commit that took its message and authorship from another commit (`-C`, `-c`),
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
//!  A `--fixup=reword:` commit is the one line left out:
//!  Git throws the index away for it,
//!  so its commit is the branch's own tree and a replay of the captured tree would land changes the person did not choose.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = await reissuedArguments({ context, cwd, arguments, subcommand, file, prepared, interactive });
//! ```

use crate::commit_args::{normalise_commit_args, takes_next_token};
use crate::commit_transaction::TransactionStop;
use crate::service_amend::split_author;
use crate::service_amend_replayed::stored_commit_fields;
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
use std::ffi::OsString;
use std::path::Path;

/// The token that ends the options of a commit region.
const SEPARATOR: &str = "--";

/// Short option letters a rebuilt line drops:
/// the selection the private index already holds.
const SELECTION_LETTERS: [char; 2] = ['p', 'i'];

/// Short option letters whose value is attached, so that what follows is never read as a flag.
const ATTACHED_VALUE_LETTERS: [char; 2] = ['S', 'u'];

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
///   `-c`, `-C`, `--reuse-message` and `--reedit-message` take both,
///  so a rebuild that keeps the message in a file must supply the authorship itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const borrows = (token) => ['-c', '-C', '--reuse-message', '--reedit-message'].includes(nameOf(token));
/// ```
fn borrows_its_author(token: &str) -> bool {
    let name: &str = name_of(token);
    return matches!(name, "-c" | "-C" | "--reuse-message" | "--reedit-message");
}

/// The name of an option,
/// without the value an `=` attached to it.
fn name_of(token: &str) -> &str {
    return token.split_once('=').map_or(token, |(name, _)| { return name; });
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
fn rewords_another_commit(token: &str) -> bool {
    let Some(value) = token.strip_prefix("--fixup=").or_else(|| { return token.strip_prefix("--squash="); }) else {
        return false;
    };
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
fn without_selection_letters(token: &str) -> Option<String> {
    let letters: Vec<char> = token.chars().skip(1).collect();
    let mut kept: String = String::from("-");
    let mut at: usize = 0;
    while at < letters.len() {
        let letter: char = letters[at];
        at += 1;
        if ATTACHED_VALUE_LETTERS.contains(&letter) {
            kept.push(letter);
            kept.extend(letters[at..].iter());
            break;
        }
        if SELECTION_LETTERS.contains(&letter) {
            continue;
        }
        kept.push(letter);
    }
    return if kept == "-" { None } else { Some(kept) };
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
    /// with the authorship the walk was given where it needs one.
    Line(Vec<String>),
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
fn without_settled_options(region: &[String], interactive: bool, borrowed: Option<&BorrowedAuthor>) -> Settled {
    let mut kept: Vec<String> = Vec::new();
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
        if token == SEPARATOR {
            kept.push(token.clone());
            state = State::Verbatim;
            continue;
        }
        if rewords_another_commit(token.as_str()) {
            return Settled::Cannot;
        }
        let name: &str = name_of(token.as_str());
        if borrows_its_author(token.as_str()) {
            if borrowed.is_none() {
                return Settled::NeedsTheAuthorship;
            }
            state = dropped_after(token.as_str());
            continue;
        }
        if matches!(name, "--fixup" | "--squash") {
            state = dropped_after(token.as_str());
            continue;
        }
        if matches!(name, "--author" | "--date") && borrowed.is_some() {
            state = dropped_after(token.as_str());
            continue;
        }
        if interactive && matches!(name, "-p" | "-i" | "--patch" | "--interactive") {
            state = dropped_after(token.as_str());
            continue;
        }
        if interactive && token.starts_with('-') && !token.starts_with("--") && token.len() > 1 {
            match without_selection_letters(token.as_str()) {
                Some(cluster) => kept.push(cluster),
                None => {}
            }
            state = kept_after(token.as_str());
            continue;
        }
        kept.push(token.clone());
        state = kept_after(token.as_str());
    }
    return Settled::Line(kept);
}

/// What the walk reads after an option it keeps.
fn kept_after(token: &str) -> State {
    return if takes_next_token(token) { State::Value } else { State::Option };
}

/// What the walk reads after an option it drops.
fn dropped_after(token: &str) -> State {
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
    let text: Vec<String> = arguments.iter().map(|token| { return token.to_string_lossy().into_owned(); }).collect();
    let region: Vec<String> = normalise_commit_args(text.get(subcommand + 1..).unwrap_or(&[]));
    let (mut rebuilt, borrowed): (Vec<String>, Option<BorrowedAuthor>) = match without_settled_options(region.as_slice(), interactive, None) {
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
        rebuilt.insert(0, format!("--author={}", found.author));
        rebuilt.insert(1, format!("--date={}", found.date));
    }
    let mut whole: Vec<OsString> = arguments.get(..=subcommand).unwrap_or(&[]).to_vec();
    whole.extend(rebuilt.into_iter().map(OsString::from));
    return Ok(crate::service_commit_args::with_fixed_message(whole.as_slice(), subcommand, file));
}

/// Controls for the line a moved branch rebuilds stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_line_reissued_tests.rs"]
mod tests;
