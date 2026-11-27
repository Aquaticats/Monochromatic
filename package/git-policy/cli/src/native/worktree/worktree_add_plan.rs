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

/// The Git global option table this reading was copied from,
/// which only a control reads.
#[cfg(test)]
pub(crate) use crate::worktree_add_plan_copy_destination::COPIED_GLOBAL_OPTION_TABLE_GIT_RELEASE;
/// Import the two words of that reading a control names,
/// which the release build never reads.
#[cfg(test)]
use crate::worktree_add_plan_copy_destination::{
    copy_add_operands,
    creating_destination,
};
/// Import the copy's destination reading,
/// and hand it on under this module's name,
/// which is the name every caller already writes.
pub use crate::worktree_add_plan_copy_destination::{
    copy_start_directory,
    parse_copy_destination,
};
/// Import the questions about a path that the plan's own destination asks.
use crate::worktree_add_plan_path_questions::{
    is_slash,
    is_symbolic_link,
    names_parent_directory,
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

/// Plan controls,
/// with a real Git control,
/// stay out of the release executable.
#[cfg(test)]
#[path = "worktree_add_plan_tests.rs"]
mod tests;
