//! What:
//!  The facts one audit line carries about a `git worktree remove`:
//!  the directory it names,
//!  whether the
//!       person forced it,
//!  the `HEAD` the removed worktree had,
//!  and how many tracked paths differed from it.
//! Why:
//!  The owner chose "Log only" for this command,
//!  with the note that agents are just as likely to run
//!       `rm -rf`,
//!  and asked for one line naming the worktree,
//!  whether `--force` was given,
//!  and what the
//!       removal took,
//!  without making the removal slow.
//!  So the line records what a removal can read cheaply:
//!  the target's
//!       `HEAD` from two file reads,
//!  and the number of tracked paths that differ,
//!  from one `git status` that reads the
//!       index and never walks the untracked or ignored files
//!       (measured:
//! 20 ms on this repository's own worktree).
//!  What it does not
//!       count,
//! it says it does not count,
//!  so a reader is never told the worktree was clean.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // measureRemoval({ context, target, force }) => [{ name: 'force', value: true }, ...]
//! ```

use crate::discard_audit_facts::AuditFact;
use crate::git_metadata::{MetadataOutput, path_from_git_bytes, run_metadata_git};
/// A pathname from the bytes a pointer file holds.
use crate::hooks_path_values::path_answer;
use crate::transaction_git::GitContext;
/// `OsString`/`OsStr` are owned/borrowed operating-system text of raw bytes,
///  and `Path`/`PathBuf` owned/borrowed paths.
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::path::{Path, PathBuf};

/// The sentence the record carries about what it counted.
const COUNTED_TRACKED_ONLY: &str = "tracked changes only";

/// The most bytes a pointer file may hold,
///  more than any `gitdir:` line or `HEAD`.
/// Why:
///   A record is written before a removal and never holds one up,
///  so the read is bounded as well as
///       non-blocking.
const POINTER_BYTES: u64 = 4096;

/// What:
///  The Git directory a linked worktree's `.git` names,
///  read as Git resolves it:
///  a relative
///       pointer is relative to the worktree that holds it.
///  `Option<PathBuf>` is "the directory or nothing".
/// Why:
///   A `.git` that is a directory names a main worktree,
///  which a removal never names,
///  and a pointer that is not
///       `gitdir:` is one Git refuses;
///  resolving a relative pointer against the process's directory
///  would read
///       another worktree's facts or none.
///  The line is read as bytes and the path it names is built from them,
///  because a Git directory whose own path is not UTF-8 is still the one this worktree uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeGitDirectory(target: Buffer): Buffer | undefined;
/// ```
fn worktree_git_directory(target: &Path) -> Option<PathBuf> {
    let pointer: Vec<u8> = read_pointer_file(target.join(".git").as_path())?;
    let named: &[u8] = pointer.trim_ascii().strip_prefix(b"gitdir:")?.trim_ascii();
    if named.is_empty() {
        return None;
    }
    let directory: PathBuf = path_answer(named);
    if directory.is_absolute() {
        return Some(directory);
    }
    return Some(target.join(directory));
}

/// What:
///  The bytes of a pointer file,
///  or nothing when the path is not a regular file or cannot be read.
/// Why:
///   Opening a FIFO with no writer waits for ever,
///  and this read happens before a removal the person asked for:
///  the type is checked
///  first and the open cannot wait,
///  so a best-effort fact never holds the command.
///  The bytes are returned as they are,
///  because the path the file names may hold bytes that are not UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPointerFile(path: Buffer): Buffer | undefined;
/// ```
fn read_pointer_file(path: &Path) -> Option<Vec<u8>> {
    // `metadata` follows a link, so a link to a regular file is still a regular file; anything else is not read.
    let metadata: std::fs::Metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    let opened: std::fs::File = open_without_waiting(path).ok()?;
    let mut bytes: Vec<u8> = Vec::new();
    // `.take(n)` bounds the read; a file longer than that is not a pointer file.
    opened.take(POINTER_BYTES).read_to_end(&mut bytes).ok()?;
    return Some(bytes);
}

/// What:
///  A file opened for reading without waiting for anything.
/// Why:
///   A path can be replaced by a FIFO between the type check and this open,
///  and a blocking open of a
///       FIFO with no writer never returns;
///  `O_NONBLOCK` makes that open fail instead,
///  and a read of a
///       pipe with no bytes ready fails rather than waiting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openWithoutWaiting(path: string): FileHandle;
/// ```
fn open_without_waiting(path: &Path) -> std::io::Result<std::fs::File> {
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    return options.open(path);
}

/// What:
///  The `HEAD` of a worktree,
/// read from the Git directory the worktree's `.git` names.
/// Why:
///   Two file reads cost nothing beside a removal,
///  and they name the branch or commit the worktree was on,
///  which is
///       what brings the work back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeHead(target: string): string | undefined;
/// ```
pub fn worktree_head(target: &Path) -> Option<String> {
    // The Git directory the worktree's own `.git` names; a main worktree names none a removal could take.
    let directory: PathBuf = worktree_git_directory(target)?;
    // `HEAD` names a ref or a commit, which is text; a pointer file names a pathname, which is bytes.
    let head: String = String::from_utf8(read_pointer_file(directory.join("HEAD").as_path())?).ok()?;
    let trimmed: &str = head.trim();
    let named: &str = trimmed.strip_prefix("ref: ").unwrap_or(trimmed);
    if named.is_empty() {
        return None;
    }
    return Some(named.to_string());
}

/// What:
///  How many tracked paths differ from `HEAD` in the worktree at `target`,
/// or nothing when Git
///  cannot answer.
/// Why:
///   This is the one fact worth a Git start,
///  and it is the cheapest one:
///  `--untracked-files=no`
///       reads the index and the tracked files,
///  and no reading of ignored files is added because walking a large
///       ignored tree is what would make a removal slow.
///  The query names the target with its own `-C`,
///  so the invocation's
///       global options cannot point it at another repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runGit(['-C', target, '--no-optional-locks', 'status', '--porcelain=v1', '--untracked-files=no']);
/// ```
pub fn changed_paths(context: &GitContext, target: &Path) -> Option<usize> {
    let arguments: Vec<OsString> = vec![
        OsString::from("-C"),
        target.as_os_str().to_os_string(),
        OsString::from("--no-optional-locks"),
        OsString::from("status"),
        OsString::from("--porcelain=v1"),
        OsString::from("--untracked-files=no"),
    ];
    let output: MetadataOutput =
        run_metadata_git(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice()).ok()?;
    if !output.success {
        return None;
    }
    return Some(output.stdout.split(is_newline).filter(is_entry).count());
}

/// What:
///  Whether a byte of the status answer ends a line.
/// Why:
///   The count is one per entry,
///  and a named function keeps that pass free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0x0a
/// ```
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Whether one piece of the answer is an entry rather than the text after the last newline.
/// Why:
///   A trailing newline leaves one empty piece,
///  which is not an entry,
///  and `filter` hands the predicate a
///       reference to the piece.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (line: Buffer) => line.length > 0
/// ```
fn is_entry(line: &&[u8]) -> bool {
    return !line.is_empty();
}

/// What:
///  The facts one record of a removed worktree carries,
/// best effort:
///  a fact Git could not answer is left out
///  rather than reported as an answer.
///  `Path` is the resolved target and `&OsStr` its spelling as the person typed it.
/// Why:
///   A record that cannot be completed must not stop the removal the person asked for,
///  and a fact that is absent is
///       honest where a guessed one is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removalFacts({ context, target, typed, force }): AuditFact[];
/// ```
pub fn removal_facts(context: &GitContext, target: &Path, typed: &OsStr, force: bool) -> Vec<AuditFact> {
    let mut facts: Vec<AuditFact> = vec![
        AuditFact::Text("worktree", typed.to_string_lossy().into_owned()),
        AuditFact::Flag("force", force),
    ];
    if let Some(head) = worktree_head(target) {
        facts.push(AuditFact::Text("head", head));
    }
    if let Some(changed) = changed_paths(context, target) {
        facts.push(AuditFact::Text("changed", changed.to_string()));
    }
    facts.push(AuditFact::Text("counted", COUNTED_TRACKED_ONLY.to_string()));
    return facts;
}

/// What:
///  The paths of the repository's worktrees,
///  as `git worktree list --porcelain` prints them,
///  or nothing when Git cannot be asked.
/// Why:
///   Git resolves the name of a removal against the worktrees the repository registers,
///  and that list is
///       the only place the resolved path can come from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runGit([...global, 'worktree', 'list', '--porcelain', '-z']);
/// ```
fn worktree_paths(context: &GitContext) -> Option<Vec<PathBuf>> {
    let mut arguments: Vec<OsString> = context.global_prefix.clone();
    for token in ["worktree", "list", "--porcelain", "-z"] {
        arguments.push(OsString::from(token));
    }
    let output: MetadataOutput =
        run_metadata_git(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice()).ok()?;
    if !output.success {
        return None;
    }
    // `-z` ends every field with a NUL and every record with one more, so a path holding a line break stays whole.
    let mut paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    for field in output.stdout.split(is_nul_byte) {
        if field.starts_with(WORKTREE_FIELD) {
            paths.push(path_from_git_bytes(&field[WORKTREE_FIELD.len()..])?);
        }
    }
    return Some(paths);
}

/// The word a `git worktree list --porcelain -z` field puts before a worktree's path.
const WORKTREE_FIELD: &[u8] = b"worktree ";

/// What:
///  Whether a byte of a `-z` answer ends a record.
/// Why:
///   The NUL-terminated form separates records with zero bytes,
/// and a named function keeps that pass free of
///  a closure that captures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether `short` names the components `long` ends with.
/// Why:
///   Git accepts a unique path suffix of a registered worktree,
///  so `topic` names
///  `/r/worktrees/topic` without naming `/r/main/topic`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endsWithComponents(long: string[], short: string[]): boolean;
/// ```
fn ends_with_components(long: &Path, short: &Path) -> bool {
    let tail: Vec<OsString> = short.iter().map(part_text).collect();
    if tail.is_empty() {
        return false;
    }
    let head: Vec<OsString> = long.iter().map(part_text).collect();
    if tail.len() > head.len() {
        return false;
    }
    let offset: usize = head.len() - tail.len();
    return head[offset..] == tail[..];
}

/// What:
///  Whether a registered worktree is the directory the name resolved to.
/// Why:
///   The same directory can be spelled with `..`,
///  a link or an absolute path,
///  and the record has to
///       name the worktree that spelling means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sameDirectory(worktree: string, resolved?: string): boolean;
/// ```
fn same_directory(worktree: &Path, resolved: Option<&Path>) -> bool {
    let Some(target) = resolved else {
        return false;
    };
    let Ok(other) = std::fs::canonicalize(worktree) else {
        return false;
    };
    return other == target;
}

/// What:
///  One path component as owned operating-system text.
/// Why:
///   Comparing two paths component by component needs owned parts,
/// and a named function keeps the
///  comparison free of a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (part: string) => part
/// ```
fn part_text(part: &std::ffi::OsStr) -> OsString {
    return part.to_os_string();
}

/// What:
///  The path of the worktree a `git worktree remove` names,
///  as Git resolves it:
///  the named path,
///  or the one registered worktree its
///  components end.
///  `&OsStr` is the name as the person typed it and `&Path` the directory Git works in.
/// Why:
///   A removal by a name Git resolves as a suffix
///  (`git worktree remove topic` for
///       `/r/worktrees/topic`) read against the command's own directory would describe a directory that is not
///  a worktree at all:
///  the record then counted the enclosing main worktree and claimed the removal took
///  nothing.
///  Every read of the target has to happen before Git removes it,
///  so the resolution is one list of the registered worktrees,
///  asked first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removalTarget({ context, typed, directory }): string;
/// ```
pub fn removal_target(context: &GitContext, typed: &OsStr, directory: &Path) -> PathBuf {
    let joined: PathBuf = directory.join(typed);
    let Some(registered) = worktree_paths(context) else {
        return joined;
    };
    let resolved: Option<PathBuf> = std::fs::canonicalize(&joined).ok();
    for path in &registered {
        // An exact path is the worktree itself, whether the name is absolute, relative or holds `..`.
        if *path == joined || same_directory(path, resolved.as_deref()) {
            return path.clone();
        }
    }
    let mut found: Option<&PathBuf> = None;
    let mut several: bool = false;
    for path in &registered {
        if !ends_with_components(path, Path::new(typed)) {
            continue;
        }
        several = found.is_some();
        found = Some(path);
        if several {
            break;
        }
    }
    // Git refuses a name that matches several worktrees, so only one resolution is a worktree it will remove.
    if !several
        && let Some(path) = found
    {
        return path.clone();
    }
    return joined;
}

/// The removal facts and the real-Git controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_audit_tests.rs"]
mod tests;
