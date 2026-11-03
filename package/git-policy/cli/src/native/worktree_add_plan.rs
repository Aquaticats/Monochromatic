//! What:
//!  The plan for one `git worktree add` the cheap-worktree ladder may serve: where the new worktree goes and where in
//!  the argument list a rung inserts `--no-checkout`, or nothing when the invocation uses a form the ladder does not
//!  know.
//! Why:
//!  The ladder runs real Git once with the person's arguments, so it only needs the destination path and a promise that
//!  every other word means what the table says. An option the table does not list, a bundled short flag, a global
//!  option that changes how Git reads configuration, or a destination whose parent does not exist makes the whole
//!  invocation a plain Git run, which is always correct.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = parseAddPlan(['-C', 'repo', 'worktree', 'add', '-b', 'topic', '../topic'], 1, cwd); // undefined: plain Git
//! ```

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// What:
///  The facts of a `git worktree add` the ladder serves.
///  `usize` is an unsigned size or index; `PathBuf` is an owned file-system path.
/// Why:
///   The destination decides the subvolume to create and the worktree to settle; the insertion index decides where
///  `--no-checkout` goes in the arguments real Git receives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddPlan = { destination: string; insertAt: number; quiet: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddPlan {
    /// The destination as Git records it: the real path of its parent joined with its last component.
    pub destination: PathBuf,
    /// The index in the full argument list where options of `add` may be inserted: directly after the word `add`.
    pub insert_at: usize,
    /// Whether the person passed `--quiet`, which silences the line Git's checkout prints.
    pub quiet: bool,
}

/// The options of `add` that take no value and keep the checkout of a new worktree the ordinary one.
const FLAG_OPTIONS: &[&[u8]] = &[b"--detach", b"-d", b"--quiet", b"-q", b"--lock", b"--checkout"];

/// The options of `add` whose value is the next argument.
const VALUE_OPTIONS: &[&[u8]] = &[b"-b", b"--reason"];

/// The global options that cannot change how the ladder's own Git runs read the repository.
const HARMLESS_GLOBALS: &[&[u8]] = &[b"--no-pager", b"-P", b"--paginate", b"-p", b"--no-advice", b"--no-lazy-fetch"];

/// What:
///  The directory a command starts in: the process directory moved by every `-C <dir>` of the global prefix, or nothing
///  when the prefix holds an option the ladder does not serve.
/// Why:
///   Git changes directory at each `-C` in turn, and a relative destination is relative to the result. A `..`
///  component in a `-C` value is declined, because following it lexically differs from Git's directory changes
///  through a symbolic link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startDirectory(prefix: string[], cwd: string): string | undefined;
/// ```
pub fn start_directory(global_prefix: &[OsString], current: &Path) -> Option<PathBuf> {
    let mut start: PathBuf = current.to_path_buf();
    let mut tokens = global_prefix.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes == b"-C" {
            let target: &OsString = tokens.next()?;
            if Path::new(target).components().any(|part| { return part == std::path::Component::ParentDir; }) {
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
///  The real path of a destination as Git records it: the canonical parent joined with the last component.
/// Why:
///   Git stores `strbuf_add_real_path` of the new worktree after creating its parent directories; the ladder serves
///  only a parent that exists, so no directory has to be guessed. A last component of `.` or `..`, or one that is already
///  a symbolic link, is declined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordedDestination(start: string, operand: string): string | undefined;
/// ```
fn recorded_destination(start: &Path, operand: &OsStr) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let mut trimmed: &[u8] = operand.as_bytes();
    while let Some(rest) = trimmed.strip_suffix(b"/") {
        trimmed = rest;
    }
    let last: &[u8] = trimmed.rsplit(|byte| { return *byte == b'/'; }).next()?;
    if last.is_empty() || last == b"." || last == b".." {
        return None;
    }
    let joined: PathBuf = start.join(OsStr::from_bytes(trimmed));
    let parent: PathBuf = std::fs::canonicalize(joined.parent()?).ok()?;
    let recorded: PathBuf = parent.join(OsStr::from_bytes(last));
    // Git records the resolved path of a destination that is a link, so the directory Git fills and the path the ladder
    // would swap differ: decline, and let Git run alone.
    if std::fs::symlink_metadata(recorded.as_path()).is_ok_and(|found: std::fs::Metadata| { return found.file_type().is_symlink(); }) {
        return None;
    }
    return Some(recorded);
}

/// What:
///  Read the arguments after `add`: the operands and whether `--quiet` was given, or nothing when any option is outside
///  the table.
/// Why:
///   `--` ends the options. An operand count other than one or two is a usage error that Git words itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addOperands(region: string[]): { operands: string[]; quiet: boolean } | undefined;
/// ```
fn add_operands(region: &[OsString]) -> Option<(Vec<&OsString>, bool)> {
    let mut operands: Vec<&OsString> = Vec::new();
    let mut quiet: bool = false;
    let mut tokens = region.iter();
    let mut options_ended: bool = false;
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if options_ended || !bytes.starts_with(b"-") || bytes == b"-" {
            operands.push(token);
        } else if bytes == b"--" {
            options_ended = true;
        } else if VALUE_OPTIONS.contains(&bytes) {
            tokens.next()?;
        } else if bytes == b"-q" || bytes == b"--quiet" {
            quiet = true;
        } else if !bytes.starts_with(b"--reason=") && !FLAG_OPTIONS.contains(&bytes) {
            return None;
        }
    }
    return if (1..=2).contains(&operands.len()) { Some((operands, quiet)) } else { None };
}

/// What:
///  Plan one invocation: `arguments` is the whole argument list, `subcommand_index` the index of `worktree` in it, and
///  `start` the directory the command starts in.
/// Why:
///   The word directly after `worktree` must be `add` itself, because Git reads the subcommand there and never
///  abbreviates it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAddPlan(args: string[], subcommandIndex: number, start: string): AddPlan | undefined;
/// ```
pub fn parse_add_plan(arguments: &[OsString], subcommand_index: usize, start: &Path) -> Option<AddPlan> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree" {
        return None;
    }
    let word_index: usize = subcommand_index + 1;
    if arguments.get(word_index)?.as_encoded_bytes() != b"add" {
        return None;
    }
    let (operands, quiet): (Vec<&OsString>, bool) = add_operands(&arguments[word_index + 1..])?;
    let destination: PathBuf = recorded_destination(start, operands[0])?;
    return Some(AddPlan { destination, insert_at: word_index + 1, quiet });
}


/// The options of `git worktree add` that take no value, as a flag in either polarity.
const COPY_FLAG_OPTIONS: &[&[u8]] = &[
    b"-f", b"--force", b"--no-force", b"--orphan", b"--no-orphan", b"-d", b"--detach", b"--no-detach", b"--checkout",
    b"--no-checkout", b"--lock", b"--no-lock", b"-q", b"--quiet", b"--no-quiet", b"--track", b"--no-track",
    b"--guess-remote", b"--no-guess-remote", b"--relative-paths", b"--no-relative-paths", b"--no-reason",
];

/// The long options of `git worktree add` that take a value in the next token; `-b` and `-B` are read as short options.
const COPY_VALUE_OPTIONS: &[&[u8]] = &[b"--reason"];

/// The global options that change nothing about where a relative destination lies.
const COPY_NEUTRAL_GLOBALS: &[&[u8]] = &[
    b"--no-pager", b"-P", b"--paginate", b"-p", b"--no-replace-objects", b"--no-lazy-fetch", b"--no-optional-locks",
    b"--no-advice", b"--literal-pathspecs", b"--glob-pathspecs", b"--noglob-pathspecs", b"--icase-pathspecs",
];

/// What:
///  The directory a command's relative destination is resolved against, after the global options.
/// Why:
///   Only `-C` moves it; `-c` and the neutral flags leave it. Any other global option (`--git-dir`, `--work-tree`, a
///  namespace) changes how Git finds the repository, so the destination is then not read and the copy says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyStartDirectory(globalPrefix: string[], current: string): string | undefined;
/// ```
pub fn copy_start_directory(global_prefix: &[OsString], current: &Path) -> Option<PathBuf> {
    let mut start: PathBuf = current.to_path_buf();
    let mut tokens = global_prefix.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes == b"-C" {
            let target: &OsString = tokens.next()?;
            if !target.is_empty() {
                start = start.join(target);
            }
        } else if bytes == b"-c" || bytes == b"--config-env" {
            tokens.next()?;
        } else if !bytes.starts_with(b"--config-env=") && !COPY_NEUTRAL_GLOBALS.contains(&bytes) {
            return None;
        }
    }
    return Some(start);
}

/// What:
///  The operands of a `git worktree add`: the path and the optional commit-ish, with every option Git documents skipped.
/// Why:
///   The destination of a copy must be known from the command line, so the reading is complete for the documented options
///  (short options may be bundled, `-b` and `-B` take their value attached or in the next token) and refuses what it does not
///  know, such as an abbreviated long option, rather than guess an operand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyAddOperands(region: string[]): string[] | undefined;
/// ```
fn copy_add_operands(region: &[OsString]) -> Option<Vec<&OsString>> {
    let mut operands: Vec<&OsString> = Vec::new();
    let mut tokens = region.iter();
    let mut options_ended: bool = false;
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if options_ended || !bytes.starts_with(b"-") || bytes == b"-" {
            operands.push(token);
        } else if bytes == b"--" {
            options_ended = true;
        } else if COPY_VALUE_OPTIONS.contains(&bytes) {
            tokens.next()?;
        } else if bytes.starts_with(b"--reason=") || bytes.starts_with(b"--track=") || COPY_FLAG_OPTIONS.contains(&bytes) {
            continue;
        } else if bytes.starts_with(b"-") && !bytes.starts_with(b"--") {
            // Bundled short options: flags stand alone, and `b` or `B` takes the rest of the token or the next token.
            for (position, letter) in bytes[1..].iter().enumerate() {
                match letter {
                    b'f' | b'd' | b'q' => {}
                    b'b' | b'B' => {
                        if position + 2 == bytes.len() {
                            tokens.next()?;
                        }
                        break;
                    }
                    _ => return None,
                }
            }
        } else {
            return None;
        }
    }
    return if (1..=2).contains(&operands.len()) { Some(operands) } else { None };
}

/// What:
///  The destination a `git worktree add` names, in the form Git records it, for every form of the documented options.
/// Why:
///   The ignored-state copy goes to the worktree this command created, found from the command line instead of by comparing
///  the repository's registrations before and after, which another add running at the same time would disturb.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCopyDestination(args: string[], subcommandIndex: number, start: string): string | undefined;
/// ```
pub fn parse_copy_destination(arguments: &[OsString], subcommand_index: usize, start: &Path) -> Option<PathBuf> {
    if arguments.get(subcommand_index)?.as_encoded_bytes() != b"worktree" {
        return None;
    }
    let word_index: usize = subcommand_index + 1;
    if arguments.get(word_index)?.as_encoded_bytes() != b"add" {
        return None;
    }
    let operands: Vec<&OsString> = copy_add_operands(&arguments[word_index + 1..])?;
    return recorded_destination(start, operands[0]);
}

/// Plan controls, with a real Git control, stay out of the release executable.
#[cfg(test)]
#[path = "worktree_add_plan_tests.rs"]
mod tests;
