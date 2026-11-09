//! What:
//!  The question "does this worktree hold uncommitted state?"
//! as one Git query and its reading.
//! Why:
//!  A command that can discard uncommitted state is judged by whether any exists:
//! a clean linked worktree has
//!  nothing to lose,
//! a dirty one does.
//! Tracked changes (staged or not) always count;
//! untracked files count only
//!  for the commands that delete them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git status --porcelain=v1 --untracked-files=no  => dirty when it prints anything
//! ```

use crate::git_metadata::MetadataOutput;
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// What:
///  Which files count as uncommitted state.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A `reset --hard` loses tracked changes only,
/// a `clean` loses untracked files,
/// and a `clean -x` also
///  loses the ignored files a person keeps beside the build output;
/// asking for the wrong set would
///  reject a harmless command or let a destructive one through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirtyScope = 'tracked' | 'tracked-and-untracked' | 'everything' | 'ignored-only';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirtyScope {
    /// Staged and unstaged changes to tracked files.
    Tracked,
    /// Tracked changes and untracked files that are not ignored.
    TrackedAndUntracked,
    /// Tracked changes,
    ///  untracked files and ignored files.
    Everything,
    /// Ignored files alone:
    ///  what `git clean -X` deletes.
    IgnoredOnly,
}

/// What:
///  The arguments of the status query,
/// after the invocation's global options.
///  `Vec<OsString>` is an owned list of byte-preserving text.
/// Why:
///   `--no-optional-locks` keeps a read from writing the index,
/// and the porcelain form is stable across
///  Git versions.
///  A reading that reaches ignored files asks for them with
///  `--ignored=matching`,
///  which reports an ignored directory once instead of walking into it,
/// and Git itself refuses
///  that setting together with `--untracked-files=no`,
/// so such a reading asks for untracked files as well and reads
///  only the lines the reading is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const args = [...preSubcommandArgs, '--no-optional-locks', 'status', '--porcelain=v1', '--untracked-files=no'];
/// ```
pub fn dirty_query_arguments(global_prefix: &[OsString], scope: DirtyScope) -> Vec<OsString> {
    // `.to_vec()` copies the borrowed prefix into a list this function owns.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for token in ["--no-optional-locks", "status", "--porcelain=v1"] {
        arguments.push(OsString::from(token));
    }
    arguments.push(OsString::from(match scope {
        DirtyScope::Tracked => "--untracked-files=no",
        DirtyScope::TrackedAndUntracked => "--untracked-files=normal",
        DirtyScope::Everything | DirtyScope::IgnoredOnly => "--untracked-files=normal",
    }));
    if matches!(scope, DirtyScope::Everything | DirtyScope::IgnoredOnly) {
        arguments.push(OsString::from("--ignored=matching"));
    }
    return arguments;
}

/// What:
///  The pathspec magic that reads a path exactly as it is,
/// anchored at the top of the worktree,
/// so no
///       path in it is read as a glob,
///  as `:(` magic or as relative to the caller's directory.
/// Why:
///   A tree path is data,
/// not a pattern:
/// `a*b` names one file,
/// and a path the
///  person keeps may hold any byte but NUL.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TOP_LITERAL_PATHSPEC = ':(top,literal)';
/// ```
pub const TOP_LITERAL_PATHSPEC: &[u8] = b":(top,literal)";

/// What:
///  The most bytes of pathspec one ignored-paths query carries.
/// Why:
///   One start carries what the system accepts (measured: 1860 pathspecs of 85 KB answered in 21 ms
///  in one start,
///  and this host's `getconf ARG_MAX` is 2,097,152 bytes),
/// and a worktree with millions of
///  tracked paths is asked in as many bounded starts as it needs instead of failing to start at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IGNORED_PATHS_QUERY_BYTES = 128 * 1024;
/// ```
pub const IGNORED_PATHS_QUERY_BYTES: usize = 128 * 1024;

/// What:
///  How many shared paths a rejection text names before it says how many more there are.
/// Why:
///   The text has to name the affected paths plainly (a person acts on them),
///  and a worktree that
///       ignores thousands of entries would turn it into a list nobody reads;
///  five keeps the sentence
///  within one screen.
pub const SHARED_PATHS_NAMED: usize = 5;

/// What:
///  The queries that list which of `paths` the worktree holds as ignored files,
/// one argument
///       list per bounded start,
///  and no query at all for no paths.
///  `&[Vec<u8>]` borrows the paths as their tree holds them.
/// Why:
///   Git answers for the named paths alone,
/// so a worktree with a million ignored files costs what its
///  tracked files cost and not a walk of everything,
///  and every path is asked exactly as it is:
///  the
///       caller's own `--literal-pathspecs` would turn the magic into a path,
///  so the query turns that
///       reading off again,
///  and `--full-name` answers paths from the top of the worktree whatever the caller's
///       directory.
///  `-z` keeps every answer's bytes exactly as they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ignoredPathsQueries(globalPrefix: string[], paths: string[]): string[][];
/// ```
pub fn ignored_paths_queries(global_prefix: &[OsString], paths: &[Vec<u8>]) -> Vec<Vec<OsString>> {
    let mut head: Vec<OsString> = global_prefix.to_vec();
    for token in [
        "--no-optional-locks",
        "--no-literal-pathspecs",
        "ls-files",
        "--others",
        "--ignored",
        "--exclude-standard",
        "-z",
        "--full-name",
        "--",
    ] {
        head.push(OsString::from(token));
    }
    let mut queries: Vec<Vec<OsString>> = Vec::<Vec<OsString>>::new();
    let mut current: Vec<OsString> = head.clone();
    let mut carried: usize = 0;
    for path in paths {
        let mut spec: Vec<u8> = TOP_LITERAL_PATHSPEC.to_vec();
        spec.extend_from_slice(path);
        // A chunk that already holds a path opens a new one rather than overrunning the system's limit.
        if carried > 0 && carried + spec.len() > IGNORED_PATHS_QUERY_BYTES {
            queries.push(current);
            current = head.clone();
            carried = 0;
        }
        carried += spec.len();
        current.push(spec_argument(spec.as_slice()));
    }
    if carried > 0 {
        queries.push(current);
    }
    return queries;
}

/// What:
///  One argument's exact bytes,
/// as the system reads them.
/// Why:
///   A path is bytes and not text,
/// so a path that is not UTF-8 reaches Git unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argument = Buffer.from(spec);
/// ```
fn spec_argument(spec: &[u8]) -> OsString {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        return OsString::from_vec(spec.to_vec());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return OsString::from(String::from_utf8_lossy(spec).into_owned());
    }
}

/// What:
///  The ignored paths one query answered,
/// each exactly as Git printed it.
///  `Result<Vec<Vec<u8>>, String>` is "the paths,
/// or why Git could not give one".
/// Why:
///   A query that failed says nothing,
/// and a safety check must not read a failure as "nothing is ignored".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ignoredPathsFromAnswer(answer: MetadataOutput): Buffer[]; // throws
/// ```
pub fn ignored_paths_from_answer(answer: &MetadataOutput) -> Result<Vec<Vec<u8>>, String> {
    if !answer.success {
        let detail: String = String::from_utf8_lossy(&answer.stderr)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(format!("Git could not read the ignored files: {detail}"));
    }
    let mut paths: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in answer.stdout.split(is_nul) {
        if !path.is_empty() {
            paths.push(path.to_vec());
        }
    }
    return Ok(paths);
}

/// What:
///  Whether a byte of a `-z` answer ends a path.
/// Why:
///   The NUL-terminated form separates paths with zero bytes,
/// and a named function keeps that pass free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0
/// ```
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Read the query:
/// any output line means uncommitted state exists.
///  `Result<bool, String>` is "the answer,
/// or why Git could not give one".
/// Why:
///   A status that failed says nothing,
/// and a safety check must not read a failure as "clean".
///  `IgnoredOnly` counts the entries Git marks with `!!`,
/// because the untracked entries of the same
///  answer are not what `git clean -X` deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dirtyFromOutput(output: MetadataOutput, scope: DirtyScope): boolean; // throws
/// ```
pub fn dirty_from_output(output: &MetadataOutput, scope: DirtyScope) -> Result<bool, String> {
    if !output.success {
        let detail: String = String::from_utf8_lossy(&output.stderr)
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(format!("Git could not read the worktree status: {detail}"));
    }
    if scope == DirtyScope::IgnoredOnly {
        return Ok(output.stdout.split(is_newline).any(is_ignored_entry));
    }
    return Ok(!output.stdout.is_empty());
}

/// What:
///  Whether a byte of the status answer ends a line.
/// Why:
///   The ignored-only reading looks at whole entries,
/// and a named function keeps that pass free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0x0a
/// ```
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Whether one line of `--porcelain=v1` is an ignored entry.
/// Why:
///   Git marks an ignored path with `!!` and a space,
/// and the untracked entries of the same answer are
///  not what `git clean -X` deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (line: Buffer) => line.subarray(0, 3).toString() === '!! '
/// ```
fn is_ignored_entry(line: &[u8]) -> bool {
    return line.starts_with(b"!! ");
}

/// What:
///  Whether a Git directory has never had an index file:
/// the state `git clone --no-checkout` leaves.
///  `Option<&Path>` is "a borrowed path or nothing";
/// the second argument is the value of `GIT_INDEX_FILE`,
/// injected so tests
///  never read the process environment.
/// Why:
///   With no index file Git sees an initial checkout and has nothing of the user's to protect:
/// `builtin/checkout.c` sets
///  `initial_checkout = is_index_unborn(...)` and `is_index_unborn` in `read-cache.c` is `!istate->cache_nr && !istate->timestamp.sec`,
///  so `unpack-trees.c` skips the rejection of a staged deletion (`if (oldtree && !o->initial_checkout)`).
/// `git status` still lists every
///  committed file as a staged deletion there,
/// which the guard read as uncommitted work.
/// The definition stays as narrow as Git's:
/// an index
///  file that exists,
/// empty or not,
/// is never absent,
/// so a staged deletion of every file still counts;
/// a `GIT_INDEX_FILE` that names
///  another index,
/// an unreadable Git directory and a Git directory that does not exist all answer false and leave the question to `git status`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexFileIsAbsent(gitDirectory?: string, indexFileVariable?: string): boolean;
/// ```
pub fn index_file_is_absent(git_directory: Option<&Path>, index_file_variable: Option<&OsStr>) -> bool {
    if index_file_variable.is_some() {
        return false;
    }
    let Some(directory) = git_directory else {
        return false;
    };
    // `symlink_metadata` does not follow a link, so a dangling link named `index` is present, never absent.
    return match std::fs::symlink_metadata(directory.join("index")) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => directory.is_dir(),
        Err(_) | Ok(_) => false,
    };
}

/// Arguments of each scope and the reading of a successful,
/// empty,
/// non-empty and failed status.
#[cfg(test)]
#[path = "rule_worktree_dirty_tests.rs"]
mod tests;
