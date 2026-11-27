//! What:
//!   The Git plumbing a capture runs:
//!   the bytes of one path written as one object,
//!   the index's own entries,
//!   and the directory a Git start runs in.
//! Why:
//!   A capture writes every byte of a person's work as an object before the checkout changes,
//! and every step of it is one Git start whose arguments carry bytes,
//! so one file holds those starts and the reading of what they print.
//! The worktree reading itself,
//! and the capture that uses this,
//! stay in `merge_isolation_carry.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const name = await objectName(context, root, path, bytes); // then keepCapture() writes the ref
//! ```

/// The capture's own record of one index entry,
/// which is the shape its reading answers with.
use crate::merge_isolation_carry_parts::CapturedIndex;
/// The worktree path a filter name is built from.
use crate::merge_isolation_changes::worktree_path;
/// Real Git,
/// on either face.
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
};
use crate::transaction_git_awaiting::run_git_awaiting;
/// Owned operating-system text.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
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
///  The index entry one path has,
/// from the entries one read returned.
/// Why:
///   The capture keeps the entry of each path it carries,
///  and a path with no entry is one the index never held.
pub(crate) fn entry_at<'a>(
    entries: &'a [Entry],
    path: &[u8],
) -> Option<&'a Entry> {
    for entry in entries {
        if entry.path.as_slice() == path {
            return Some(entry);
        }
    }
    return None;
}

/// The capture's own record of one index entry.
pub(crate) fn captured_index_of(entry: &Entry) -> CapturedIndex {
    return CapturedIndex {
        mode: entry.mode.clone(),
        oid: entry.oid.clone(),
    };
}

/// What:
///  The directory a Git start runs in.
/// Why:
///   The invocation's own global options can name the repository relatively
/// (`git -C repo merge`),
/// and those are relative to
///  the directory the person ran the command in,
/// which is the process directory;
/// a context without them (recovery) runs
///  from the worktree itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cwd = (context, root) => context.globalPrefix.length === 0 ? root : '.';
/// ```
pub fn git_cwd<'a>(
    context: &GitContext,
    root: &'a Path,
) -> &'a Path {
    if context.global_prefix.is_empty() {
        return root;
    }
    return Path::new(".");
}

/// One run of Git whose arguments carry bytes that need not be text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBytes(context, root: string, args: OsString[], input?: Uint8Array): Promise<GitOutput>;
/// ```
pub(crate) async fn run_bytes_awaiting(
    context: &GitContext,
    root: &Path,
    arguments: &[OsString],
    input: Option<Vec<u8>>,
) -> Result<GitOutput, String> {
    let mut request: GitRequest = GitRequest::new::<&str>(git_cwd(context, root), &[]);
    request.arguments = arguments.to_vec();
    request.input = input;
    return match run_git_awaiting(context, &request).await {
        Ok(output) => Ok(output),
        Err(error) => Err(format!("git could not start: {error}")),
    };
}

/// What:
///  Write the bytes as a blob,
/// with the path's clean filters when `filtered` says so.
/// Why:
///   The raw form is what the person sees and is written back byte for byte;
/// the filtered form is what Git records and is
///  what a three-way merge can compare with a commit's blob.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function objectFor(context, root, path: Uint8Array, bytes: Uint8Array, filtered: boolean): Promise<string>;
/// ```
pub(crate) async fn object_for_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    bytes: &[u8],
    filtered: bool,
) -> Result<String, String> {
    return object_with_awaiting(context, root, path, bytes, filtered, true).await;
}

/// What:
///  The name of the object the given bytes make at `path`,
///  with that path's clean filters,
///  and without writing
///  it.
/// Why:
///   Comparing what the worktree holds with what the index records asks for the name alone,
/// and a comparison that
///  writes an object would change the repository to answer a question.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function objectName(context, root, path, bytes): Promise<string>;
/// ```
pub(crate) async fn object_name_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    bytes: &[u8],
) -> Result<String, String> {
    return object_with_awaiting(context, root, path, bytes, true, false).await;
}

/// What:
///  The object of the bytes,
/// written only when `write` says so.
/// Why:
///   One body decides how the filters are named and what an unavailable object store says,
/// so the capture and the
///  comparison cannot differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function objectWith(context, root, path, bytes, filtered, write): Promise<string>;
/// ```
async fn object_with_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    bytes: &[u8],
    filtered: bool,
    write: bool,
) -> Result<String, String> {
    let mut arguments: Vec<OsString> = vec![OsString::from("hash-object")];
    if write {
        arguments.push(OsString::from("-w"));
    }
    if filtered {
        let mut named: OsString = OsString::from("--path=");
        named.push(worktree_path(Path::new(""), path).into_os_string());
        arguments.push(named);
    } else {
        arguments.push(OsString::from("--no-filters"));
    }
    arguments.push(OsString::from("--stdin"));
    let output: GitOutput =
        run_bytes_awaiting(context, root, arguments.as_slice(), Some(bytes.to_vec())).await?;
    if !output.succeeded() {
        return Err(format!(
            "the bytes of {} could not be read as an object: {}",
            String::from_utf8_lossy(path),
            output.error_text()
        ));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice())
        .trim()
        .to_owned());
}

/// One index entry as the index prints it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { mode: string; oid: string; path: Uint8Array };
/// ```
pub(crate) struct Entry {
    mode: String,
    oid: String,
    path: Vec<u8>,
}

/// What:
///  Every stage-0 index entry,
/// which is what the person's staged work is.
/// Why:
///   One read serves every path,
/// and an entry of another stage is a conflict the merge must not be run over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function stagedEntries(context, root): Promise<Entry[]>;
/// ```
pub(crate) async fn staged_entries_awaiting(
    context: &GitContext,
    root: &Path,
) -> Result<Vec<Entry>, String> {
    let arguments: Vec<OsString> = vec![
        OsString::from("ls-files"),
        OsString::from("--stage"),
        OsString::from("-z"),
    ];
    let output: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await?;
    if !output.succeeded() {
        return Err(format!("git ls-files failed: {}", output.error_text()));
    }
    let mut found: Vec<Entry> = Vec::new();
    for field in output.stdout.split(is_nul) {
        let Some(split) = field.iter().position(is_a_tab) else {
            continue;
        };
        let header: String = String::from_utf8_lossy(&field[..split]).into_owned();
        let parts: Vec<&str> = header.split(' ').collect();
        let [mode, oid, stage] = parts.as_slice() else {
            continue;
        };
        if *stage != "0" {
            return Err(String::from(
                "the index holds unmerged entries, which native Git reports itself",
            ));
        }
        found.push(Entry {
            mode: String::from(*mode),
            oid: String::from(*oid),
            path: field[split + 1..].to_vec(),
        });
    }
    return Ok(found);
}
