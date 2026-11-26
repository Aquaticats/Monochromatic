//! What:
//!  The plan for one `git worktree add` the cheap-worktree ladder may serve:
//! where the new worktree goes and where in
//!  the argument list a rung inserts `--no-checkout`,
//! or nothing when the invocation uses a form the ladder does not
//!  know.
//! Why:
//!  The ladder runs real Git once with the person's arguments,
//! so it only needs the destination path and a promise that
//!  every other word means what the table says.
//! An option the table does not list,
//! a bundled short flag,
//! a global
//!  option that changes how Git reads configuration,
//! or a destination whose parent does not exist makes the whole
//!  invocation a plain Git run,
//! which is always correct.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = parseAddPlan(['-C', 'repo', 'worktree', 'add', '-b', 'topic', '../topic'], 1, cwd); // undefined: plain Git
//! ```

use crate::worktree_add_option_table::{
    AddOption,
    LongOption,
    Takes,
    option_for_short_letter,
    resolve_long,
    take_value,
};
use crate::worktree_add_plan_path_questions::{
    is_slash,
    is_symbolic_link,
    names_parent_directory,
    nearest_existing_ancestor,
};
/// The plan these questions fill in,
/// which the forwarded run's facts carry.
use crate::worktree_command_facts::AddPlan;
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::{
    Path,
    PathBuf,
};

/// The options of `add` that take no value and keep the checkout of a new worktree the ordinary one.
const FLAG_OPTIONS: &[&[u8]] = &[
    b"--detach",
    b"-d",
    b"--quiet",
    b"-q",
    b"--lock",
    b"--checkout",
];

/// The options of `add` whose value is the next argument.
const VALUE_OPTIONS: &[&[u8]] = &[b"-b", b"--reason"];

/// The global options that cannot change how the ladder's own Git runs read the repository.
const HARMLESS_GLOBALS: &[&[u8]] = &[
    b"--no-pager",
    b"-P",
    b"--paginate",
    b"-p",
    b"--no-advice",
    b"--no-lazy-fetch",
];

/// What:
///  The directory a command starts in:
/// the process directory moved by every `-C <dir>` of the global prefix,
/// or nothing
///  when the prefix holds an option the ladder does not serve.
/// Why:
///   Git changes directory at each `-C` in turn,
/// and a relative destination is relative to the result.
/// A `..`
///  component in a `-C` value is declined,
/// because following it lexically differs from Git's directory changes
///  through a symbolic link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startDirectory(prefix: string[], cwd: string): string | undefined;
/// ```
pub fn start_directory(
    global_prefix: &[OsString],
    current: &Path,
) -> Option<PathBuf> {
    let mut start: PathBuf = current.to_path_buf();
    let mut tokens = global_prefix.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes == b"-C" {
            let target: &OsString = tokens.next()?;
            if names_parent_directory(Path::new(target)) {
                return None;
            }
            if !target.is_empty() {
                start = start.join(target);
            }
        } else if !HARMLESS_GLOBALS.contains(&bytes) {
            return None;
        }
    }
    return Some(start);
}

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
///  The real path of a destination as Git records it:
/// the canonical parent joined with the last component.
/// Why:
///   Git stores `strbuf_add_real_path` of the new worktree after creating its parent directories;
/// the ladder serves
///  only a parent that exists,
/// so no directory has to be guessed.
/// A last component of `.` or `..`,
/// or one that is already
///  a symbolic link,
/// is declined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordedDestination(start: string, operand: string): string | undefined;
/// ```
fn recorded_destination(
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
    let parent: PathBuf = std::fs::canonicalize(joined.parent()?).ok()?;
    let recorded: PathBuf = parent.join(OsStr::from_bytes(last));
    // Git records the resolved path of a destination that is a link, so the directory Git fills and the path the ladder
    // would swap differ: decline, and let Git run alone.
    if is_symbolic_link(recorded.as_path()) {
        return None;
    }
    return Some(recorded);
}

/// What:
///  The words after `add`:
/// its operands,
/// whether `--quiet` was given,
/// and the two options that decide what Git checks out.
/// Why:
///   Every reading takes this one pass,
/// so the plan,
/// the quiet answer and the tree the add names cannot disagree about which words were written.
///  `-b` and `--detach` are the only two options of the table that change what the new worktree's `HEAD` is
///  (`builtin/worktree.c`,
///  `add_worktree`),
///  and `-B` never reaches this pass because the table does not read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddWords = { operands: string[]; quiet: boolean; newBranch: boolean; detach: boolean };
/// ```
struct AddWords<'a> {
    /// The path and,
    /// when the add names one,
    /// the commit-ish.
    operands: Vec<&'a OsString>,
    /// Whether `--quiet` was written.
    quiet: bool,
    /// Whether `-b` was written,
    /// which makes Git create a branch at the commit-ish and check that out.
    new_branch: bool,
    /// Whether `--detach` was written,
    /// which makes Git detach at the commit-ish.
    detach: bool,
}

/// What:
///  Read the arguments after `add`,
/// or nothing when any option is outside
///  the table.
/// Why:
///   `--` ends the options.
/// An operand count other than one or two is a usage error that Git words itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addOperands(region: string[]): AddWords | undefined;
/// ```
fn add_operands(region: &[OsString]) -> Option<AddWords<'_>> {
    let mut operands: Vec<&OsString> = Vec::new();
    let mut quiet: bool = false;
    let mut new_branch: bool = false;
    let mut detach: bool = false;
    let mut tokens = region.iter();
    let mut options_ended: bool = false;
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if options_ended || !bytes.starts_with(b"-") || bytes == b"-" {
            operands.push(token);
        } else if bytes == b"--" {
            options_ended = true;
        } else if VALUE_OPTIONS.contains(&bytes) {
            new_branch = new_branch || bytes == b"-b";
            tokens.next()?;
        } else if bytes == b"-q" || bytes == b"--quiet" {
            quiet = true;
        } else if bytes == b"-d" || bytes == b"--detach" {
            detach = true;
        } else if !bytes.starts_with(b"--reason=") && !FLAG_OPTIONS.contains(&bytes) {
            return None;
        }
    }
    return if (1..=2).contains(&operands.len()) {
        Some(AddWords {
            operands,
            quiet,
            new_branch,
            detach,
        })
    } else {
        None
    };
}

/// What:
///  Plan one invocation:
/// `arguments` is the whole argument list,
/// `subcommand_index` the index of `worktree` in it,
/// and
///  `start` the directory the command starts in.
/// Why:
///   The word directly after `worktree` must be `add` itself,
/// because Git reads the subcommand there and never
///  abbreviates it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAddPlan(args: string[], subcommandIndex: number, start: string): AddPlan | undefined;
/// ```
pub fn parse_add_plan(
    arguments: &[OsString],
    subcommand_index: usize,
    start: &Path,
) -> Option<AddPlan> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree" {
        return None;
    }
    let word_index: usize = subcommand_index + 1;
    if arguments.get(word_index)?.as_encoded_bytes() != b"add" {
        return None;
    }
    let words: AddWords<'_> = add_operands(&arguments[word_index + 1..])?;
    let destination: PathBuf = recorded_destination(start, words.operands[0])?;
    let target: Option<OsString> = words.operands.get(1).map(|named| return (*named).clone());
    return Some(AddPlan {
        destination,
        insert_at: word_index + 1,
        quiet: words.quiet,
        target,
        checks_out_a_branch: !words.new_branch && !words.detach,
    });
}

/// What:
///  Why a `git worktree add` the ladder does not serve runs as plain Git,
/// in the clause the notice reads.
/// Why:
///   An add the wrapper cannot serve is still an add a person waits for,
/// and the one thing the wrapper
///  knows about it is which fact put it on the plain route:
/// the folder the worktree goes in does not exist yet,
///  which is what the ladder's own reading of the destination refuses,
/// or a form of the command the wrapper does not read at all.
/// The clause is built here so the words and the reading that refuses share one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whyPlain = (destination) => parentMissing(destination) ? `because the folder it goes in, ${quoted}, does not exist yet` : 'because the form of the command is not one it reads';
/// ```
pub fn why_a_plain_add(destination: Option<&Path>) -> String {
    if let Some(parent) = destination.and_then(Path::parent)
        && matches!(std::fs::symlink_metadata(parent), Err(ref error) if error.kind() == std::io::ErrorKind::NotFound)
    {
        return format!(
            "because the folder it goes in, {}, does not exist yet",
            crate::json_record::quote(parent.to_string_lossy().as_ref())
        );
    }
    return String::from("because the command's form is not one the wrapper reads");
}

/// What:
///  Whether the person asked for `--quiet` on a `git worktree add`,
///  whatever the ladder makes of the form.
/// Why:
///   The words after `add` are read once,
/// by [`add_operands`],
/// and the plan carries the answer only for a form the ladder
///  serves:
/// an add whose destination the ladder refuses still has a person who may have asked for no
///  lines,
/// so the answer is taken from the same reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quiet = (args, subcommandIndex) => addOperands(args.slice(subcommandIndex + 2))?.[1] ?? false;
/// ```
pub fn add_is_quiet(
    arguments: &[OsString],
    subcommand_index: usize,
) -> bool {
    let word_is = |index: usize, wanted: &[u8]| {
        return arguments.get(index).is_some_and(|word| {
            return word.as_encoded_bytes() == wanted;
        });
    };
    if !word_is(subcommand_index, b"worktree") {
        return false;
    }
    let word_index: usize = subcommand_index + 1;
    if !word_is(word_index, b"add") {
        return false;
    }
    return add_operands(&arguments[word_index + 1..]).is_some_and(|words: AddWords<'_>| {
        return words.quiet;
    });
}

/// What:
///  The Git release whose `handle_options` the global option tables below copy:
///  its major and minor numbers, one `u64` each.
///  `u64` and not `u32` or `i64`, the siblings a reader might expect,
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
///  a later patch release is taken to add no global option, and these tables stand for it.
/// Source:
///  `git.c` at tag `v2.56.0` of `git/git`,
///  `handle_options`, lines 157 to 370,
///  and the same arms in that file at `v2.56.1`.
///  The arm that ends its chain decides that Git accepts nothing else:
///  `} else {` /
///  `fprintf(stderr, _("unknown option: %s\n"), cmd);` /
///  `usage(git_usage_string);`
///  (lines 361 to 364).
///  Five arms end the run before any command word is read,
///  so they stand in no command's global prefix and are listed nowhere below:
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
///  `-C`, which moves the start,
///  and `--exec-path=<path>`, which sets the path and lets the command run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COPIED_GLOBAL_OPTION_TABLE_GIT_RELEASE: [number, number] = [2, 56];
/// ```
const COPIED_GLOBAL_OPTION_TABLE_GIT_RELEASE: (u64, u64) = (2, 56);

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
fn copy_add_operands(region: &[OsString]) -> Option<Vec<&OsString>> {
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
fn creating_destination(
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

/// Plan controls,
/// with a real Git control,
/// stay out of the release executable.
#[cfg(test)]
#[path = "worktree_add_plan_tests.rs"]
mod tests;
