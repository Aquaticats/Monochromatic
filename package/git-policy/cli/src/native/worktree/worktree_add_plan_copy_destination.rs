//! What:
//!  The destination of a `git worktree add` whose worktree Git's own checkout fills,
//! read from the command line by the grammar of Git's option parser:
//! the global prefix,
//! and the operands after `add`.
//! Why:
//!  The ignored-state copy an add makes goes to the worktree that add created,
//! and it finds that worktree from the command line
//! instead of comparing the repository's registrations before and after,
//! which another add running at the same time would disturb.
//! The destination is known only when every global option in the prefix is one Git accepts:
//! an option this reading does not know,
//! or an operand it cannot settle,
//! leaves the destination unknown,
//! and the copy then does not run.
//! The ladder's own plan reads a narrower grammar
//! (`worktree_add_plan.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const destination = parseCopyDestination(argv, subcommandIndex, copyStartDirectory(argv, cwd)); // undefined: no copy
//! ```

/// Import the option table of `git worktree add`,
/// which says which long and short words take a value.
use crate::worktree_add_option_table::{
    AddOption,
    LongOption,
    Takes,
    option_for_short_letter,
    resolve_long,
    take_value,
};
/// Import the questions about a path that both destinations ask.
use crate::worktree_add_plan_path_questions::{
    is_slash,
    is_symbolic_link,
    nearest_existing_ancestor,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  Whether a global option word is a valued global in its attached form,
/// `<name>=<value>`.
/// Why:
///   Such a word carries its value,
/// so the word after it is not a value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// COPY_VALUED_GLOBALS.some((name) => word.startsWith(`${name}=`))
/// ```
fn is_valued_global_with_equals(word: &[u8]) -> bool {
    for name in COPY_VALUED_GLOBALS {
        if let Some(rest) = word.strip_prefix(*name)
            && rest.first() == Some(&b'=')
        {
            return true;
        }
    }
    return false;
}

/// What:
///  The Git release whose `handle_options` the tables `COPY_NEUTRAL_GLOBALS`,
///  `COPY_VALUED_GLOBALS` and `COPY_SEPARATE_VALUE_GLOBALS` copy:
///  its major and minor numbers,
///  one `u64` each.
///  `u64` and not `u32` or `i64`,
///  the siblings a reader might expect,
///  because `parse_git_version` answers with `u64` numbers,
///  so the control compares like with like.
/// Why:
///   A release that adds a global option makes the copy incomplete:
///  the reading then cannot tell where that option's value ends,
///  so a relative destination after it is left unread,
///  and a slot's removal is forwarded to Git.
///  The control in `worktree_add_plan_tests.rs` refuses a Git newer than this release,
///  so these tables are compared with that Git's `handle_options` before this constant moves.
///  Only the major and minor numbers are compared:
///  a later patch release is taken to add no global option,
///  and these tables stand for it.
///  It is `pub` because the release build never reads it:
///  those three tables are used by `copy_start_directory`,
///  and this release fact is read by the control and by anyone comparing the tables with a Git,
///  so it is part of this module's API and not a private constant the release holds unused.
/// Source:
///  `git.c` at tag `v2.56.0` of `git/git`,
///  `handle_options`,
///  lines 157 to 370,
///  and the same arms in that file at `v2.56.1`.
///  The arm that ends its chain decides that Git accepts nothing else:
///  `} else {` /
///  `fprintf(stderr, _("unknown option: %s\n"), cmd);` /
///  `usage(git_usage_string);`
///  (lines 361 to 364).
///  Five arms end the run before any command word is read,
///  so they stand in no command's global prefix and none is in those three tables:
///  `--exec-path` without `=` prints the path and exits (lines 178 to 185),
///  `--html-path`,
///  `--man-path` and
///  `--info-path` print one and exit (lines 186 to 197),
///  and `--list-cmds=` lists commands and exits (lines 329 to 341).
///  `--help`,
///  `-h`,
///  `--version` and
///  `-v` end the loop as the command word itself (lines 171 to 173),
///  so they name no global option either.
///  Two forms are read by `copy_start_directory` itself rather than by a table:
///  `-C`,
///  which moves the start,
///  and `--exec-path=<path>`,
///  which sets the path and lets the command run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COPIED_GLOBAL_OPTION_TABLE_GIT_RELEASE: [number, number] = [2, 56];
/// ```
#[cfg(test)]
pub(crate) const COPIED_GLOBAL_OPTION_TABLE_GIT_RELEASE: (u64, u64) = (2, 56);

/// The global options that take no value and change nothing about where a relative destination lies.
const COPY_NEUTRAL_GLOBALS: &[&[u8]] = &[
    b"--no-pager",
    b"-P",
    b"--paginate",
    b"-p",
    b"--no-replace-objects",
    b"--no-lazy-fetch",
    b"--no-optional-locks",
    b"--no-advice",
    b"--literal-pathspecs",
    b"--no-literal-pathspecs",
    b"--glob-pathspecs",
    b"--noglob-pathspecs",
    b"--icase-pathspecs",
    b"--bare",
];

/// The global options that take a value in the next token or after `=`,
/// and change nothing about where a relative
/// destination lies (`git.c` of Git 2.56.0,
/// `handle_options`:
/// `--git-dir`,
/// `--work-tree`,
/// `--namespace`,
/// `--config-env`,
/// `--attr-source`).
/// A relative destination lies against the working directory whatever they say:
/// Git changes into the work
/// tree and prefixes the operand when the working directory is inside it,
/// and leaves it alone when it is outside
/// (`setup.c`,
/// lines 1222 to 1231).
const COPY_VALUED_GLOBALS: &[&[u8]] = &[
    b"--git-dir",
    b"--work-tree",
    b"--namespace",
    b"--config-env",
    b"--attr-source",
];

/// The global options that take a value in the next token only.
const COPY_SEPARATE_VALUE_GLOBALS: &[&[u8]] = &[b"-c", b"--shallow-file"];

/// What:
///  The directory a command's relative destination is resolved against,
/// after the global options.
/// Why:
///   Only `-C` moves it;
/// the others change nothing about it.
/// An option this reading does not know leaves the destination
///  unknown,
/// and the copy then says it did not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyStartDirectory(globalPrefix: string[], current: string): string | undefined;
/// ```
pub fn copy_start_directory(
    global_prefix: &[OsString],
    current: &Path,
) -> Option<PathBuf> {
    let mut start: PathBuf = current.to_path_buf();
    let mut tokens = global_prefix.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes == b"-C" {
            let target: &OsString = tokens.next()?;
            if !target.is_empty() {
                start = start.join(target);
            }
        } else if COPY_SEPARATE_VALUE_GLOBALS.contains(&bytes)
            || COPY_VALUED_GLOBALS.contains(&bytes)
        {
            tokens.next()?;
        } else if is_valued_global_with_equals(bytes) || bytes.starts_with(b"--exec-path=") {
            continue;
        } else if !COPY_NEUTRAL_GLOBALS.contains(&bytes) {
            return None;
        }
    }
    return Some(start);
}

/// What:
///  The operands of a `git worktree add`:
/// the path and the optional commit-ish,
/// with every option skipped the way Git's
///  `parse_options` skips it.
/// Why:
///   The destination of a copy must be known from the command line.
/// Long options are resolved by `resolve_long`,
/// short
///  options bundle (`-fq`,
/// `-bfeature`,
/// `-b feature`),
/// `--` and `--end-of-options` end the options,
/// and everything Git
///  rejects makes the reading refuse,
/// so the copy never guesses an operand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyAddOperands(region: string[]): string[] | undefined;
/// ```
pub(crate) fn copy_add_operands(region: &[OsString]) -> Option<Vec<&OsString>> {
    let mut operands: Vec<&OsString> = Vec::new();
    let mut tokens = region.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !bytes.starts_with(b"-") || bytes == b"-" {
            operands.push(token);
        } else if bytes == b"--" || bytes == b"--end-of-options" {
            operands.extend(tokens.by_ref());
        } else if let Some(long) = bytes.strip_prefix(b"--") {
            let resolved: LongOption<'_> = resolve_long(std::str::from_utf8(long).ok()?)?;
            take_value(
                resolved.option,
                resolved.unset,
                resolved.attached,
                &mut tokens,
            )?;
        } else {
            // Bundled short options: flags stand alone, and `b` or `B` takes the rest of the token or the next token.
            let letters: &[u8] = &bytes[1..];
            for (position, letter) in letters.iter().enumerate() {
                let option: &AddOption = option_for_short_letter(*letter)?;
                if option.takes == Takes::Required {
                    if position + 1 == letters.len() {
                        tokens.next()?;
                    }
                    break;
                }
            }
        }
    }
    return if (1..=2).contains(&operands.len()) {
        Some(operands)
    } else {
        None
    };
}

/// What:
///  The destination of a path operand in the form Git records it,
/// whether or not its parent directories exist yet.
/// Why:
///   Git creates the leading directories of the destination,
/// so `.claude/worktrees/topic` names a destination before
///  `.claude/worktrees` does;
/// the nearest existing ancestor is resolved to its real path and the rest is appended as written.
///  A destination that is a link is declined,
/// as in the ladder's reading,
/// and a `..` in the part that does not exist yet is
///  declined because it cannot be resolved without the directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function creatingDestination(start: string, operand: string): string | undefined;
/// ```
pub(crate) fn creating_destination(
    start: &Path,
    operand: &OsStr,
) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let mut trimmed: &[u8] = operand.as_bytes();
    while let Some(rest) = trimmed.strip_suffix(b"/") {
        trimmed = rest;
    }
    let last: &[u8] = trimmed.rsplit(is_slash).next()?;
    if last.is_empty() || last == b"." || last == b".." {
        return None;
    }
    let joined: PathBuf = start.join(OsStr::from_bytes(trimmed));
    if is_symbolic_link(joined.as_path()) {
        return None;
    }
    let existing: &Path = nearest_existing_ancestor(joined.as_path())?;
    let mut resolved: PathBuf = std::fs::canonicalize(existing).ok()?;
    for part in joined.strip_prefix(existing).ok()?.components() {
        match part {
            std::path::Component::Normal(name) => resolved.push(name),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    return Some(resolved);
}

/// What:
///  The destination a `git worktree add` names,
/// in the form Git records it,
/// for every form Git's option parser accepts.
/// Why:
///   The ignored-state copy goes to the worktree this command created,
/// found from the command line instead of by comparing
///  the repository's registrations before and after,
/// which another add running at the same time would disturb.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCopyDestination(args: string[], subcommandIndex: number, start: string): string | undefined;
/// ```
pub fn parse_copy_destination(
    arguments: &[OsString],
    subcommand_index: usize,
    start: &Path,
) -> Option<PathBuf> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree" {
        return None;
    }
    let word_index: usize = subcommand_index + 1;
    if arguments.get(word_index)?.as_encoded_bytes() != b"add" {
        return None;
    }
    let operands: Vec<&OsString> = copy_add_operands(&arguments[word_index + 1..])?;
    return creating_destination(start, operands[0]);
}
