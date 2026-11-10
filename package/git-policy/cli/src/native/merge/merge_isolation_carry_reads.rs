//! What:
//!  What a carry reads from the repository:
//!  one object's bytes in the form a checkout writes,
//!  the same object's stored bytes,
//!  one path's index entry,
//!  and the raw changes between two commits.
//! Why:
//!   Every one of them is Git's own answer,
//!  and they share one entry point that carries bytes a command line cannot spell;
//!  the judgements and the writes that use them are `merge_isolation_carry_apply.rs`,
//!  which is at its code-line limit.
//!  Each read is awaited,
//!  because a start that registers the step's pipes enters the runtime,
//!  which is refused from inside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = await fileFormAwaiting(context, root, path, oid); const entry = await indexEntryAwaiting(context, root, path);
//! ```

use crate::lazy_runtime::run_on_runtime;
use crate::merge_isolation_carry::git_cwd;
use crate::merge_isolation_changes::{Change, worktree_path};
use crate::transaction_git::{GitContext, GitOutput, GitRequest};
use crate::transaction_git_awaiting::run_git_awaiting;
use std::ffi::OsString;
use std::path::Path;

/// Whether one byte is the NUL that ends each field of Git's `-z` output.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether one byte is the tab that ends the header of a `-z` index field.
fn is_a_tab(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// What:
///  The words for a Git that could not be started at all.
/// Why:
///   A start that failed is not Git's own failure,
///  and the caller reads it as a read that answered nothing.
fn git_start_failure(error: std::io::Error) -> String {
    return format!("git could not start: {error}");
}

/// What:
///  Run Git with arguments that need not be text,
/// and bytes on standard input.
/// Why:
///   A path that is not text cannot be spelled in a `&str`,
/// so every argument here is owned operating-system text,
///  and the run is awaited because a start enters the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBytesAwaiting(context, root, args: OsString[], input?: Uint8Array): Promise<GitOutput>;
/// ```
pub(crate) async fn run_bytes_awaiting(context: &GitContext, root: &Path, arguments: &[OsString], input: Option<Vec<u8>>) -> Result<GitOutput, String> {
    let mut request: GitRequest = GitRequest::new::<&str>(git_cwd(context, root), &[]);
    request.arguments = arguments.to_vec();
    request.input = input;
    return run_git_awaiting(context, &request).await.map_err(git_start_failure);
}

/// One path spelled for a Git argument.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argument = (path: Uint8Array) => worktreePath('', path);
/// ```
pub(crate) fn path_argument(path: &[u8]) -> OsString {
    return worktree_path(Path::new(""), path).into_os_string();
}

/// What:
///  The file form of one object:
/// the bytes a checkout writes for it.
/// Why:
///  A file's bytes in the worktree are not the object's bytes when a filter applies,
/// so a content the person reads
///  is made the way Git makes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fileFormAwaiting(context, root, path, oid): Promise<Uint8Array>;
/// ```
pub(crate) async fn file_form_awaiting(context: &GitContext, root: &Path, path: &[u8], oid: &str) -> Result<Vec<u8>, String> {
    let mut named: OsString = OsString::from("--path=");
    named.push(path_argument(path));
    let arguments: Vec<OsString> = vec![OsString::from("cat-file"), OsString::from("--filters"), named, OsString::from(oid)];
    let output: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await?;
    if !output.succeeded() {
        return Err(format!("the bytes of {} could not be read: {}", String::from_utf8_lossy(path), output.error_text()));
    }
    return Ok(output.stdout);
}

/// What:
///  One object's bytes as they are stored,
/// with no filter applied.
/// Why:
///   A capture keeps the person's bytes exactly,
/// so they are put back exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function rawBytesAwaiting(context, root, oid): Promise<Uint8Array>;
/// ```
pub(crate) async fn raw_bytes_awaiting(context: &GitContext, root: &Path, oid: &str) -> Result<Vec<u8>, String> {
    let arguments: Vec<OsString> = vec![OsString::from("cat-file"), OsString::from("blob"), OsString::from(oid)];
    let output: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await?;
    if !output.succeeded() {
        return Err(format!("the bytes this merge carries could not be read: {}", output.error_text()));
    }
    return Ok(output.stdout);
}

/// What:
///  The index entry of one path,
/// as the index prints it.
/// Why:
///   Every judgement is made against the entry that is there now,
/// and an entry the capture or the change explains is
///  not someone else's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function indexEntryAwaiting(context, root, path): Promise<{ mode, oid } | null>;
/// ```
pub(crate) async fn index_entry_awaiting(context: &GitContext, root: &Path, path: &[u8]) -> Result<Option<(String, String)>, String> {
    let mut arguments: Vec<OsString> = vec![
        OsString::from("ls-files"),
        OsString::from("--stage"),
        OsString::from("-z"),
        OsString::from("--"),
    ];
    arguments.push(path_argument(path));
    let output: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await?;
    if !output.succeeded() {
        return Err(format!("git ls-files failed: {}", output.error_text()));
    }
    for field in output.stdout.split(is_nul) {
        let Some(split) = field.iter().position(is_a_tab) else { continue };
        let header: String = String::from_utf8_lossy(&field[..split]).into_owned();
        let parts: Vec<&str> = header.split(' ').collect();
        let [mode, oid, stage] = parts.as_slice() else { continue };
        if *stage != "0" {
            return Err(String::from("the index holds an unmerged entry for a path this merge carries"));
        }
        return Ok(Some((String::from(*mode), String::from(*oid))));
    }
    return Ok(None);
}

/// What:
///  The paths the merge changes,
/// with the mode and object each side holds.
/// Why:
///   The clear and the carry both read them,
/// and they are Git's own raw difference between the two commits,
/// so
///  a path the merge renames is told from one it changes.
///  A synchronous caller takes one entry of the runtime for the read;
///  a caller that is itself a task uses `changes_between_awaiting`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function changesBetween(context, root, oldOid, newOid): Promise<Change[]>;
/// ```
pub fn changes_between(context: &GitContext, root: &Path, old_oid: &str, new_oid: &str) -> Result<Vec<Change>, String> {
    return match run_on_runtime(changes_between_awaiting(context, root, old_oid, new_oid)) {
        Ok(found) => found,
        Err(refusal) => Err(format!("{refusal}")),
    };
}

/// What:
///  The same tree difference,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   A synchronous start registers the step's pipes by entering the runtime,
/// which is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function changesBetweenAwaiting(context, root, oldOid, newOid): Promise<Change[]>;
/// ```
pub async fn changes_between_awaiting(context: &GitContext, root: &Path, old_oid: &str, new_oid: &str) -> Result<Vec<Change>, String> {
    let arguments: Vec<OsString> = vec![
        OsString::from("diff-tree"),
        OsString::from("-r"),
        OsString::from("-z"),
        OsString::from("--raw"),
        OsString::from("--no-renames"),
        OsString::from("--no-abbrev"),
        OsString::from(old_oid),
        OsString::from(new_oid),
    ];
    let output: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await?;
    if !output.succeeded() {
        return Err(format!("git diff-tree failed: {}", output.error_text()));
    }
    return crate::merge_isolation_changes::parse_raw_changes(output.stdout.as_slice());
}
