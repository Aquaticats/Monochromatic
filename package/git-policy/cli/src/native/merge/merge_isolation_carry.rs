//! What:
//!  The durable half of carrying a person's own work across a merge:
//! reading what their worktree and index hold in
//!  the paths the merge changes,
//! writing every byte of it as an object,
//! keeping those objects on a ref of their own,
//! and
//!  dropping that ref once the merge is done with them.
//! Why:
//!  A merge replaces exactly these paths,
//! so their bytes must exist somewhere recovery can find them before the first
//!  one changes (`package/git-policy/cli/SPEC.md`,
//! "Merges and commits never fail";
//! owner's rule "never discard user
//!  data").
//!  The object store alone is not enough:
//! a blob nothing points at is collected,
//! so one commit on
//!  `refs/cli-git/carried/<n>` holds every captured object under a slot name,
//! and the merge's record names that ref
//!  and the bytes.
//!  A path whose worktree entry is a directory,
//! or whose index entry is unmerged,
//! is not one edit anybody can merge,
//!  so the capture refuses it and the command falls back to native Git's own answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const capture = await captureWork({ session, root, paths }); await keepObjects({ session, root, capture });
//! ```

/// The captured work,
/// and the slots that keep it.
use crate::merge_isolation_carry_parts::{CapturedFile, CapturedIndex, CapturedPath, Capture, slots};
/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// Real Git,
/// on either face.
use crate::lazy_runtime::run_on_runtime;
use crate::transaction_git::{GitContext, GitOutput, GitRequest};
use crate::transaction_git_awaiting::run_git_awaiting;
/// The worktree shapes.
use crate::merge_isolation_changes::worktree_path;
/// Owned operating-system text.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::Path;

/// What stands at one worktree path,
/// as a merge reads it,
/// in the file that reads it.
pub(crate) use crate::merge_isolation_worktree_entry::{WorktreeEntry, not_one_edit_a_merge_can_carry, worktree_entry_at};

/// The tag of this module in the diagnostic log.
const TAG: &str = "mergeCarry";

/// The prefix of the refs that keep a capture,
/// beside nothing else.
pub const CARRIED_REF_PREFIX: &str = "refs/cli-git/carried/";

/// How many ref names are tried before the capture is refused.
///  A name carries milliseconds and a process number,
/// so a collision means another merge in the same millisecond;
///  eight tries is a bound,
/// not a loop.
const REF_ATTEMPTS: u128 = 8;

/// The Git mode of a symbolic link.
const SYMBOLIC_LINK_MODE: &str = "120000";

/// The Git mode of an executable file.
const EXECUTABLE_MODE: &str = "100755";

/// The Git mode of an ordinary file.
const PLAIN_FILE_MODE: &str = "100644";

/// Whether one byte is the NUL that ends each field of Git's `-z` output.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether one byte is the tab that ends the header of a `-z` index field.
fn is_a_tab(byte: &u8) -> bool {
    return *byte == b'\t';
}

/// What:
///  The words for a path of the person's work that could not be read.
/// Why:
///   Two reads of one path answer with the same sentence,
///  and the path is the one the person names.
pub(crate) fn could_not_be_read(location: &Path, error: std::io::Error) -> String {
    return format!("{} could not be read: {error}", location.display());
}

/// What:
///  The index entry one path has,
/// from the entries one read returned.
/// Why:
///   The capture keeps the entry of each path it carries,
///  and a path with no entry is one the index never held.
fn entry_at<'a>(entries: &'a [Entry], path: &[u8]) -> Option<&'a Entry> {
    for entry in entries {
        if entry.path.as_slice() == path {
            return Some(entry);
        }
    }
    return None;
}

/// The capture's own record of one index entry.
fn captured_index_of(entry: &Entry) -> CapturedIndex {
    return CapturedIndex { mode: entry.mode.clone(), oid: entry.oid.clone() };
}

/// What:
///  The words for a clock this process could not read.
/// Why:
///   The capture's reference names the moment it was made,
///  and a clock before the epoch cannot say when that was.
fn the_clock_could_not_be_read(error: std::time::SystemTimeError) -> String {
    return format!("the clock could not be read: {error}");
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
pub fn git_cwd<'a>(context: &GitContext, root: &'a Path) -> &'a Path {
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
async fn run_bytes_awaiting(context: &GitContext, root: &Path, arguments: &[OsString], input: Option<Vec<u8>>) -> Result<GitOutput, String> {
    let mut request: GitRequest = GitRequest::new::<&str>(git_cwd(context, root), &[]);
    request.arguments = arguments.to_vec();
    request.input = input;
    return match run_git_awaiting(context, &request).await {
        Ok(output) => Ok(output),
        Err(error) => Err(format!("git could not start: {error}")),
    };
}

/// What:
///  The mode Git records for what stands at a worktree path.
/// Why:
///   A link's bytes are its target text and Git records no mode of the file it points at,
/// so the kind is read before
///  the bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function modeOf(metadata): '120000' | '100755' | '100644';
/// ```
pub(crate) fn mode_of(metadata: &std::fs::Metadata) -> &'static str {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.file_type().is_symlink() {
            return SYMBOLIC_LINK_MODE;
        }
        if metadata.permissions().mode() & 0o111 != 0 {
            return EXECUTABLE_MODE;
        }
        return PLAIN_FILE_MODE;
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = metadata;
        return PLAIN_FILE_MODE;
    }
}

/// What:
///  The bytes the worktree holds at one path:
/// a file's content,
/// or a link's target text.
/// Why:
///   These are the bytes written back,
/// so they are read exactly and never through a filter.
///  A name that holds a named pipe,
/// a socket,
/// a device or a directory holds no bytes a merge can carry,
/// so a capture
///  refuses it rather than holding part of what stands there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bytesAt(location: string): { bytes: Uint8Array; mode: string } | null;
/// ```
pub(crate) fn bytes_at(location: &Path) -> Result<Option<(Vec<u8>, &'static str)>, String> {
    return match worktree_entry_at(location)? {
        WorktreeEntry::Holds { bytes, mode } => Ok(Some((bytes, mode))),
        WorktreeEntry::Absent => Ok(None),
        WorktreeEntry::NoBytes(kind) => Err(not_one_edit_a_merge_can_carry(location, &kind)),
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
pub(crate) async fn object_for_awaiting(context: &GitContext, root: &Path, path: &[u8], bytes: &[u8], filtered: bool) -> Result<String, String> {
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
pub(crate) async fn object_name_awaiting(context: &GitContext, root: &Path, path: &[u8], bytes: &[u8]) -> Result<String, String> {
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
    let output: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), Some(bytes.to_vec())).await?;
    if !output.succeeded() {
        return Err(format!("the bytes of {} could not be read as an object: {}", String::from_utf8_lossy(path), output.error_text()));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned());
}

/// One index entry as the index prints it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { mode: string; oid: string; path: Uint8Array };
/// ```
struct Entry {
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
async fn staged_entries_awaiting(context: &GitContext, root: &Path) -> Result<Vec<Entry>, String> {
    let arguments: Vec<OsString> = vec![OsString::from("ls-files"), OsString::from("--stage"), OsString::from("-z")];
    let output: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await?;
    if !output.succeeded() {
        return Err(format!("git ls-files failed: {}", output.error_text()));
    }
    let mut found: Vec<Entry> = Vec::new();
    for field in output.stdout.split(is_nul) {
        let Some(split) = field.iter().position(is_a_tab) else { continue };
        let header: String = String::from_utf8_lossy(&field[..split]).into_owned();
        let parts: Vec<&str> = header.split(' ').collect();
        let [mode, oid, stage] = parts.as_slice() else { continue };
        if *stage != "0" {
            return Err(String::from("the index holds unmerged entries, which native Git reports itself"));
        }
        found.push(Entry { mode: String::from(*mode), oid: String::from(*oid), path: field[split + 1..].to_vec() });
    }
    return Ok(found);
}

/// What:
///  The person's own work in the paths the merge changes,
/// read once.
/// Why:
///   A path the merge changes and the person has not touched is no work to carry,
/// so it is left out and the merge
///  takes it as it always did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureWork({ context, root, paths }): Promise<Capture>;
/// ```
pub fn capture_work(context: &GitContext, root: &Path, paths: &[Vec<u8>]) -> Result<Capture, String> {
    return match run_on_runtime(capture_work_awaiting(context, root, paths)) {
        Ok(found) => found,
        Err(refusal) => Err(format!("{refusal}")),
    };
}

/// What:
///  The same capture of the person's work,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Every byte is written as an object through Git,
/// and a synchronous start registers the step's pipes by entering the runtime,
///  which is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureWork({ context, root, paths }): Promise<Capture>;
/// ```
pub async fn capture_work_awaiting(context: &GitContext, root: &Path, paths: &[Vec<u8>]) -> Result<Capture, String> {
    if paths.is_empty() {
        return Err(String::from("no path holds work to carry"));
    }
    let staged: Vec<Entry> = staged_entries_awaiting(context, root).await?;
    let mut captured: Vec<CapturedPath> = Vec::new();
    for path in paths {
        let location = worktree_path(root, path.as_slice());
        let worktree: Option<CapturedFile> = match bytes_at(location.as_path())? {
            Some((bytes, mode)) => {
                let raw: String = object_for_awaiting(context, root, path.as_slice(), bytes.as_slice(), false).await?;
                let blob: String = if mode == SYMBOLIC_LINK_MODE {
                    raw.clone()
                } else {
                    object_for_awaiting(context, root, path.as_slice(), bytes.as_slice(), true).await?
                };
                Some(CapturedFile { mode: String::from(mode), raw, blob })
            }
            None => None,
        };
        let index: Option<CapturedIndex> = entry_at(staged.as_slice(), path.as_slice()).map(captured_index_of);
        // Both layers absent is the person's own deletion of a path the merge changes, which is work to carry as much as an
        //  edit is: the merge would put a file back they removed.
        captured.push(CapturedPath { path: path.clone(), worktree, index });
    }
    if captured.is_empty() {
        return Err(String::from("no path holds work to carry"));
    }
    return Ok(Capture { commit: String::new(), reference: None, paths: captured });
}

/// What:
///  The tree that holds every captured object under a slot name.
/// Why:
///   A flat list of numbered entries is a tree `git mktree` writes without a scratch index,
/// and the slot names carry
///  no path,
/// so a path that cannot be a tree entry still captures its bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureTree(context, root, capture): Promise<string>;
/// ```
async fn capture_tree_awaiting(context: &GitContext, root: &Path, capture: &Capture) -> Result<String, String> {
    let mut listing: Vec<u8> = Vec::new();
    for slot in slots(capture.paths.as_slice()) {
        listing.extend_from_slice(format!("100644 blob {}\t{}\n", slot.oid, slot.name).as_bytes());
    }
    let arguments: Vec<OsString> = vec![OsString::from("mktree")];
    let output: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), Some(listing)).await?;
    if !output.succeeded() {
        return Err(format!("the capture's tree could not be written: {}", output.error_text()));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned());
}

/// What:
///  Write the capture's objects,
/// one commit on a ref of its own,
/// and return the capture with the commit and ref that hold it.
/// Why:
///   Until the ref exists the bytes are loose objects nothing points at,
/// and a `git gc --prune=now` would
///  collect them;
/// the ref is written before the first byte of the checkout changes,
/// and the record is written
///  between the two,
/// so a kill leaves either loose objects with a record that names them or a ref that
///  keeps them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function keepCapture({ context, root, capture }): Promise<Capture>;
/// ```
pub fn keep_capture(context: &GitContext, root: &Path, capture: &Capture) -> Result<Capture, String> {
    return match run_on_runtime(keep_capture_awaiting(context, root, capture)) {
        Ok(found) => found,
        Err(refusal) => Err(format!("{refusal}")),
    };
}

/// What:
///  The same commit on a ref of its own,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The tree,
/// the commit and the ref are each a Git step,
/// and a synchronous start registers the step's pipes by entering the runtime,
///  which is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function keepCapture({ context, root, capture }): Promise<Capture>;
/// ```
pub async fn keep_capture_awaiting(context: &GitContext, root: &Path, capture: &Capture) -> Result<Capture, String> {
    let tree: String = capture_tree_awaiting(context, root, capture).await?;
    let mut arguments: Vec<OsString> = vec![OsString::from("commit-tree"), OsString::from(tree.as_str())];
    arguments.push(OsString::from("-m"));
    arguments.push(OsString::from("monochromatic-git-policy-cli: the work a merge carries"));
    let created: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await?;
    if !created.succeeded() {
        return Err(format!("the capture could not be committed: {}", created.error_text()));
    }
    let commit: String = String::from_utf8_lossy(created.stdout.as_slice()).trim().to_owned();
    let first: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(the_clock_could_not_be_read)?
        .as_millis();
    let zeros: OsString = OsString::from("0".repeat(commit.len()).as_str());
    for step in 0..REF_ATTEMPTS {
        let reference: String = format!("{CARRIED_REF_PREFIX}{}-{}-{step}", first, std::process::id());
        let mut write: Vec<OsString> = vec![
            OsString::from("update-ref"),
            OsString::from("--no-deref"),
            OsString::from(reference.as_str()),
            OsString::from(commit.as_str()),
            zeros.clone(),
        ];
        let written: GitOutput = run_bytes_awaiting(context, root, write.as_slice(), None).await?;
        if written.succeeded() {
            return Ok(Capture { commit, reference: Some(reference), paths: capture.paths.clone() });
        }
        debug(TAG, format!("the capture could not take {reference}: {}", written.error_text()).as_str());
        write.clear();
    }
    return Err(String::from("no name for the capture was free"));
}

/// What:
///  Drop the ref that kept a capture,
/// once nothing needs its objects,
/// and say whether the name is gone.
/// Why:
///   A ref left behind keeps the person's bytes reachable for ever,
/// and a name pointing at nothing anybody uses is
///  what a person has to clean up by hand.
///  The answer is what lets the caller keep the merge's record when the name stayed:
/// the record is the only thing
///  that names the ref,
/// so a later command's recovery retries the deletion instead of the name being
///  orphaned (a cross-vendor audit of alpha.12,
/// "Capture-ref cleanup is never retried after a transient failure").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropCaptureRef({ context, root, capture }): Promise<boolean>;
/// ```
pub fn drop_capture_ref(context: &GitContext, root: &Path, capture: &Capture) -> bool {
    return match run_on_runtime(drop_capture_ref_awaiting(context, root, capture)) {
        Ok(dropped) => dropped,
        // A deletion that could not be started leaves the ref where it is, which is what the awaited face reports too.
        Err(refusal) => {
            debug(TAG, format!("the capture's ref was left where it is: {refusal}").as_str());
            false
        }
    };
}

/// What:
///  The same deletion of a capture's ref,
/// awaited by a caller that is itself a task of the runtime,
///  answering whether the name is gone.
/// Why:
///   A deletion that fails is logged and answered,
/// and the caller keeps the record that names the ref so the
///  next command retries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropCaptureRef({ context, root, capture }): Promise<boolean>;
/// ```
pub async fn drop_capture_ref_awaiting(context: &GitContext, root: &Path, capture: &Capture) -> bool {
    let Some(reference) = capture.reference.as_deref() else { return true };
    let arguments: Vec<OsString> = vec![
        OsString::from("update-ref"),
        OsString::from("--no-deref"),
        OsString::from("-d"),
        OsString::from(reference),
        OsString::from(capture.commit.as_str()),
    ];
    return match run_bytes_awaiting(context, root, arguments.as_slice(), None).await {
        Ok(output) if output.succeeded() => true,
        Ok(output) => {
            debug(TAG, format!("the capture's ref {reference} was left where it is: {}", output.error_text()).as_str());
            false
        }
        Err(error) => {
            debug(TAG, format!("the capture's ref {reference} was left where it is: {error}").as_str());
            false
        }
    };
}

/// Reading what the person's work is,
/// and keeping it outside the checkout.
#[cfg(test)]
#[path = "merge_isolation_carry_tests.rs"]
mod tests;
