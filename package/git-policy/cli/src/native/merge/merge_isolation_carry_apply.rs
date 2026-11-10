//! What:
//!  The writing half of carrying a person's own work across a merge:
//! putting the merge's paths back to the branch before
//!  Git updates them,
//! merging each of the person's two layers with the merge afterwards,
//! and putting their
//!  layers back when the merge does not land.
//! Why:
//!  Git's own two-way update refuses a path that holds anything but the branch's content,
//! so the checkout is
//!  cleared in one step whose every path is judged before anything is written;
//! and after the branch moved the
//!  person's layers are merged with the merge's (`merge_isolation_carry_layers.rs`),
//! so an edit away from the
//!  merge's lines comes back where it was and an edit on them keeps both sides.
//! A path that holds bytes neither the capture
//!  nor the change explains belongs to someone else and is never written:
//! it is named and left alone.
//!  The worktree of one path is written under a private name in its own directory and renamed into place,
//! and every
//!  index entry of one step goes in one `git update-index` call,
//! so a kill leaves either what was there or the whole
//!  result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await clear({ context, root, changes, capture }); await carry({ context, root, changes, capture });
//! ```

use crate::merge_isolation_carry::git_cwd;
use crate::merge_isolation_carry_layers::{Decision, Layer, Outcome, Side, decide, write_layer_bytes};
use crate::merge_isolation_carry_notice::Report;
use crate::merge_isolation_carry_parts::{CapturedFile, CapturedIndex, Capture};
use crate::merge_isolation_changes::{Change, WorktreeState, worktree_path, worktree_state};
use crate::merge_second_pass::{ContextRunner, GitRunner};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The mode an `update-index --index-info` record gives a path it removes.
const REMOVAL_MODE: &str = "0";

/// The mode a plain file is written with when neither side names one.
const PLAIN_FILE_MODE: &str = "100644";

/// The mode of a symbolic link.
const SYMBOLIC_LINK_MODE: &str = "120000";

/// Whether one byte is the NUL that ends each field of Git's `-z` output.
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// What one layer of one path writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexWrite = { kind: 'leave' } | { kind: 'entry'; mode: string; oid: string } | { kind: 'remove' };
/// ```
enum IndexWrite {
    /// The layer already holds it.
    Leave,
    /// The entry to write.
    Entry(String, String),
    /// Nothing stands at the path.
    Remove,
}

/// What the worktree of one path writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileWrite = { kind: 'leave' } | { kind: 'bytes'; bytes: Uint8Array; mode: string } | { kind: 'remove' };
/// ```
enum FileWrite {
    /// The layer already holds it.
    Leave,
    /// The file form and the mode to write.
    Bytes(Vec<u8>, String),
    /// Nothing stands at the path.
    Remove,
}

/// One path's writes and what the report says of it.
struct Step {
    path: Vec<u8>,
    index: IndexWrite,
    worktree: FileWrite,
}

/// What one carried path adds to the report.
enum Note {
    /// Nothing worth a line.
    None,
    /// The word pass merged it.
    Merged,
    /// Both sides stand in the file between markers.
    Conflict,
    /// The person's bytes stand whole,
    /// for this reason.
    Kept(&'static str),
}

/// Which wait for the index lock a step makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockWait = 'before-the-apply' | 'while-finishing';
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum LockWait {
    /// The merge's own apply is about to run,
    /// so the wait's words are the merge's own and a dead owner's lock is cleared.
    BeforeTheApply,
    /// A merge that is being finished,
    /// where nothing is written when the lock stays held and the next command tries again.
    WhileFinishing,
}

/// Why a carry or a clear stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CarryFailure = { kind: 'index-held'; detail: string } | { kind: 'failed'; detail: string } | { kind: 'foreign'; path: Uint8Array };
/// ```
#[derive(Debug)]
pub enum CarryFailure {
    /// Another program held `index.lock` until the wait's bound,
    /// so nothing was written.
    IndexHeld(String),
    /// A read or a write failed;
    /// the capture still holds the bytes.
    Failed(String),
    /// One path holds bytes neither the capture nor the change explains.
    Foreign(Vec<u8>),
}

/// What:
///  Run Git with arguments that need not be text,
/// and bytes on standard input.
/// Why:
///   A path that is not text cannot be spelled in a `&str`,
/// so every argument here is owned operating-system text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBytes(context, root, args: OsString[], input?: Uint8Array): Promise<GitOutput>;
/// ```
fn run_bytes(context: &GitContext, root: &Path, arguments: &[OsString], input: Option<Vec<u8>>) -> Result<GitOutput, String> {
    let mut request: GitRequest = GitRequest::new::<&str>(git_cwd(context, root), &[]);
    request.arguments = arguments.to_vec();
    request.input = input;
    return run_git(context, &request).map_err(|error| { return format!("git could not start: {error}"); });
}

/// One path spelled for a Git argument.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argument = (path: Uint8Array) => worktreePath('', path);
/// ```
fn path_argument(path: &[u8]) -> OsString {
    return worktree_path(Path::new(""), path).into_os_string();
}

/// What:
///  The file form of one object: the bytes a checkout writes for it.
/// Why:
///  A file's bytes in the worktree are not the object's bytes when a filter applies,
/// so a content the person reads
///  is made the way Git makes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fileForm(context, root, path, oid): Promise<Uint8Array>;
/// ```
fn file_form(context: &GitContext, root: &Path, path: &[u8], oid: &str) -> Result<Vec<u8>, String> {
    let mut named: OsString = OsString::from("--path=");
    named.push(path_argument(path));
    let arguments: Vec<OsString> = vec![OsString::from("cat-file"), OsString::from("--filters"), named, OsString::from(oid)];
    let output: GitOutput = run_bytes(context, root, arguments.as_slice(), None)?;
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
/// async function rawBytes(context, root, oid): Promise<Uint8Array>;
/// ```
fn raw_bytes(context: &GitContext, root: &Path, oid: &str) -> Result<Vec<u8>, String> {
    let arguments: Vec<OsString> = vec![OsString::from("cat-file"), OsString::from("blob"), OsString::from(oid)];
    let output: GitOutput = run_bytes(context, root, arguments.as_slice(), None)?;
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
/// async function indexEntry(context, root, path): Promise<{ mode, oid } | null>;
/// ```
fn index_entry(context: &GitContext, root: &Path, path: &[u8]) -> Result<Option<(String, String)>, String> {
    let mut arguments: Vec<OsString> = vec![
        OsString::from("ls-files"),
        OsString::from("--stage"),
        OsString::from("-z"),
        OsString::from("--"),
    ];
    arguments.push(path_argument(path));
    let output: GitOutput = run_bytes(context, root, arguments.as_slice(), None)?;
    if !output.succeeded() {
        return Err(format!("git ls-files failed: {}", output.error_text()));
    }
    for field in output.stdout.split(is_nul) {
        let Some(split) = field.iter().position(|byte| { return *byte == b'\t'; }) else { continue };
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
///  Whether a worktree state is one the capture or the change explains.
/// Why:
///   Anything else was written after this command judged the path,
/// and replacing it would discard that
///  edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const explained = (state, objects) => state.kind === 'missing' ? objects.includes(null) : objects.includes(state.oid);
/// ```
fn explained(state: &WorktreeState, objects: &[Option<&str>]) -> bool {
    return match state {
        WorktreeState::Missing => objects.contains(&None),
        WorktreeState::Content { object, .. } => objects.contains(&Some(object.as_str())),
        WorktreeState::Other => false,
    };
}

/// What:
///  Whether an index entry is one the capture or the change explains.
/// Why:
///   A staged entry nothing explains was staged after this command judged it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entryExplained = (entry, allowed) => (entry === null && allowed.includes(null)) || allowed.some((a) => a !== null && a.mode === entry.mode && a.oid === entry.oid);
/// ```
fn entry_explained(entry: &Option<(String, String)>, allowed: &[Option<(&str, &str)>]) -> bool {
    return match entry {
        None => allowed.contains(&None),
        Some((mode, oid)) => allowed.iter().any(|candidate| { return *candidate == Some((mode.as_str(), oid.as_str())); }),
    };
}

/// The side of the change at one path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const side = (change, after) => after ? (change.existsAfter ? { mode: change.newMode, oid: change.newOid } : null) : (change.existed ? { mode: change.oldMode, oid: change.oldOid } : null);
/// ```
fn side(change: &Change, after: bool) -> Option<Side> {
    if after {
        return change.exists_after().then(|| { return Side { mode: change.new_mode.clone(), oid: change.new_oid.clone() }; });
    }
    return change.existed().then(|| { return Side { mode: change.old_mode.clone(), oid: change.old_oid.clone() }; });
}

/// What:
///  The paths the merge changes,
/// with the mode and object each side holds.
/// Why:
///   The clear and the carry both read them,
/// and they are Git's own raw difference between the two commits,
/// so
///  a path the merge renames is told from one it changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function changesBetween(context, root, oldOid, newOid): Promise<Change[]>;
/// ```
pub fn changes_between(context: &GitContext, root: &Path, old_oid: &str, new_oid: &str) -> Result<Vec<Change>, String> {
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
    let output: GitOutput = run_bytes(context, root, arguments.as_slice(), None)?;
    if !output.succeeded() {
        return Err(format!("git diff-tree failed: {}", output.error_text()));
    }
    return crate::merge_isolation_changes::parse_raw_changes(output.stdout.as_slice());
}

/// The runner that merges one layer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runner = { run: (args, input, index) => startGit(context, root, args, input, index) };
/// ```
fn rooted(context: &GitContext) -> GitContext {
    // The worktree's own shapes are read from the top of the worktree, where the invocation's global options do not reach.
    return GitContext { global_prefix: Vec::new(), ..context.clone() };
}

/// What:
///  One awaited step of the word-level pass,
/// run to its end from this synchronous path.
/// Why:
///   The pass awaits every Git start it makes
/// (`merge_second_pass_git_runner.rs`),
/// and the merge that
///  carries a person's work is a command's synchronous step,
/// so this one entry of the process runtime is taken for it.
/// Everything else the carry does is a plain start,
/// which must stay outside the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decision = runOnRuntime(decide(runner, path, layer));
/// ```
fn on_the_runtime<T>(future: impl std::future::Future<Output = Result<T, String>>) -> Result<T, CarryFailure> {
    return match crate::lazy_runtime::run_on_runtime(future) {
        Ok(found) => found.map_err(CarryFailure::Failed),
        Err(error) => Err(CarryFailure::Failed(format!(
            "the word-level pass over a path this merge carries could not run: {error}"
        ))),
    };
}

/// The runner that merges one layer,
/// from the top of the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runner = { run: (args, input, index) => startGit(rooted(context), root, args, input, index) };
/// ```
fn runner_of<'a>(context: &'a GitContext, root: &'a Path) -> ContextRunner<'a> {
    return ContextRunner { context, cwd: root, without_prefix: true, object_directory: None, environment: Vec::new() };
}

/// What:
///  The index records that write these entries and remove these paths.
/// Why:
///  One `update-index` call writes every entry,
/// so the index moves between two states in one step and a kill cannot
///  leave half of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexRecords(steps: Step[], width: number): Uint8Array;
/// ```
fn index_records(steps: &[Step], width: usize) -> Vec<u8> {
    let mut records: Vec<u8> = Vec::new();
    for step in steps {
        match &step.index {
            IndexWrite::Leave => continue,
            IndexWrite::Entry(mode, oid) => records.extend_from_slice(format!("{mode} {oid}\t").as_bytes()),
            IndexWrite::Remove => records.extend_from_slice(format!("{REMOVAL_MODE} {}\t", "0".repeat(width)).as_bytes()),
        }
        records.extend_from_slice(step.path.as_slice());
        records.push(0);
    }
    return records;
}

/// What:
///  Write one path's file under a private name in its own directory and rename it into place,
/// or make the link,
/// or remove what stands there.
/// Why:
///   A rename in the same directory is one step on whatever file system the worktree is on,
/// so a kill leaves either
///  the file that was there or the whole new one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeFile(root, path, write: FileWrite): Promise<void>;
/// ```
fn write_file(root: &Path, path: &[u8], write: &FileWrite) -> std::io::Result<()> {
    let location: PathBuf = worktree_path(root, path);
    match write {
        FileWrite::Leave => return Ok(()),
        FileWrite::Remove => {
            return match std::fs::remove_file(location.as_path()) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            };
        }
        FileWrite::Bytes(bytes, mode) => {
            if let Some(parent) = location.parent()
                && let Err(error) = std::fs::create_dir_all(parent)
            {
                return Err(error);
            }
            if mode == SYMBOLIC_LINK_MODE {
                return make_link(location.as_path(), bytes);
            }
            let aside: PathBuf = private_name(location.as_path());
            std::fs::write(aside.as_path(), bytes)?;
            set_mode(aside.as_path(), mode)?;
            return std::fs::rename(aside.as_path(), location.as_path());
        }
    }
}

/// What:
///  Replace whatever stands at a path with a symbolic link to `target`.
/// Why:
///   A link cannot be renamed into place as a file can,
/// so the path is freed and the link made;
/// a kill between the
///  two leaves the path absent and the capture still holds its target text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function makeLink(location, target: Uint8Array): Promise<void>;
/// ```
fn make_link(location: &Path, target: &[u8]) -> std::io::Result<()> {
    match std::fs::remove_file(location) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return std::os::unix::fs::symlink(std::ffi::OsStr::from_bytes(target), location);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return std::os::unix::fs::symlink(String::from_utf8_lossy(target).into_owned(), location);
    }
}

/// The private name a file is written under before it takes its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${dirname(location)}/.cli-git-carry-${pid}-${count}-${nanos}-${basename(location)}`
/// ```
fn private_name(location: &Path) -> PathBuf {
    static COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let count: u64 = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    let name: String = location.file_name().map(|found| { return found.to_string_lossy().into_owned(); }).unwrap_or_default();
    let process: u32 = std::process::id();
    return location.with_file_name(format!(".cli-git-carry-{process}-{}-{count}-{nanos:x}-{name}", 0u8));
}

/// The mode a written file carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function setMode(file, mode: string): Promise<void>; // 0o755 for 100755, else 0o644
/// ```
fn set_mode(file: &Path, mode: &str) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bits: u32 = if mode == "100755" { 0o755 } else { 0o644 };
        return std::fs::set_permissions(file, std::fs::Permissions::from_mode(bits));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = (file, mode);
        return Ok(());
    }
}

/// What:
///  Write every planned entry and file,
/// after a bounded wait for `index.lock`.
/// Why:
///   The index is the one thing two commands cannot write at once,
/// so the wait is named and bounded,
/// and a lock
///  that outlasts it leaves everything as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeSteps({ context, root, gitDir, steps, out }): Promise<void>;
/// ```
fn write_steps(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    steps: &[Step],
    wait: LockWait,
    out: &mut dyn std::io::Write,
) -> Result<(), CarryFailure> {
    let waited = match wait {
        LockWait::BeforeTheApply => crate::index_holder_wait::wait_for_index_release_before_a_merge_that_clears_a_dead_lock(git_dir.join("index").as_path(), out),
        LockWait::WhileFinishing => crate::index_holder_wait::wait_for_index_release_while_finishing_a_merge(git_dir.join("index").as_path(), out),
    };
    if !matches!(
        waited,
        crate::index_holder_wait::IndexWait::NotHeld | crate::index_holder_wait::IndexWait::Released(_)
    ) {
        return Err(CarryFailure::IndexHeld(git_dir.join("index.lock").display().to_string()));
    }
    let width: usize = steps
        .iter()
        .find_map(|step| {
            return match &step.index {
                IndexWrite::Entry(_, oid) => Some(oid.len()),
                IndexWrite::Remove | IndexWrite::Leave => None,
            };
        })
        .unwrap_or(40);
    let records: Vec<u8> = index_records(steps, width);
    if !records.is_empty() {
        let arguments: Vec<OsString> = vec![OsString::from("update-index"), OsString::from("-z"), OsString::from("--index-info")];
        let written: GitOutput = run_bytes(context, root, arguments.as_slice(), Some(records)).map_err(CarryFailure::Failed)?;
        if !written.succeeded() {
            let detail: String = written.error_text();
            if detail.contains("index.lock") {
                return Err(CarryFailure::IndexHeld(detail));
            }
            return Err(CarryFailure::Failed(detail));
        }
    }
    let mut wrote_a_file: bool = false;
    for step in steps {
        if !matches!(step.worktree, FileWrite::Leave) {
            wrote_a_file = true;
        }
        if let Err(error) = write_file(root, step.path.as_slice(), &step.worktree) {
            let path: String = String::from_utf8_lossy(step.path.as_slice()).into_owned();
            let _ = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: {path} could not be written ({error}); the bytes this merge carries are kept, and the next \
                     git command in this worktree finishes carrying them.\n"
                )
                .as_bytes(),
            );
            return Err(CarryFailure::Failed(format!("{path} could not be written: {error}")));
        }
    }
    // A file this step wrote carries a new modification time and size, and Git compares those with the index's record of the
    //  file before it updates a path (`read-tree` refuses an entry that is not up to date). The refresh re-reads the record for
    //  every entry whose content still hashes to what the index says, so the next step reads the checkout as it is.
    if wrote_a_file {
        let arguments: Vec<OsString> = vec![OsString::from("update-index"), OsString::from("-q"), OsString::from("--refresh")];
        let refreshed: GitOutput = run_bytes(context, root, arguments.as_slice(), None).map_err(CarryFailure::Failed)?;
        if !refreshed.succeeded() && refreshed.code != Some(1) {
            return Err(CarryFailure::Failed(refreshed.error_text()));
        }
    }
    return Ok(());
}

/// What:
///  Put the branch's own content back at every captured path,
/// so Git's two-way update takes them.
/// Why:
///   Git refuses a merge whose paths hold anything but the branch's content,
/// and the person's bytes are already
///  durable elsewhere.
/// Every path is judged before anything is written,
/// so a path that changed after the capture
///  stops the clear with nothing written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function clear({ context, root, gitDir, changes, capture, out }): Promise<void>;
/// ```
pub fn clear(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: &Capture,
    wait: LockWait,
    out: &mut dyn std::io::Write,
) -> Result<(), CarryFailure> {
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let Some(change) = changes.iter().find(|change| { return change.path == entry.path; }) else { continue };
        let state: WorktreeState = worktree_state(&rooted(context), root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(|file| { return file.blob.as_str(); });
        let old_object: Option<&str> = change.existed().then(|| { return change.old_oid.as_str(); });
        if !explained(&state, &[captured_object, old_object]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let current: Option<(String, String)> = index_entry(context, root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(|staged| { return (staged.mode.as_str(), staged.oid.as_str()); });
        let old_entry: Option<(&str, &str)> = change.existed().then(|| { return (change.old_mode.as_str(), change.old_oid.as_str()); });
        if !entry_explained(&current, &[captured_entry, old_entry]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let index: IndexWrite = match old_entry {
            Some((mode, oid)) => IndexWrite::Entry(String::from(mode), String::from(oid)),
            None => IndexWrite::Remove,
        };
        let worktree: FileWrite = match old_object {
            Some(oid) => FileWrite::Bytes(file_form(context, root, entry.path.as_slice(), oid).map_err(CarryFailure::Failed)?, change.old_mode.clone()),
            None => FileWrite::Remove,
        };
        steps.push(Step { path: entry.path.clone(), index, worktree });
    }
    return write_steps(context, root, git_dir, steps.as_slice(), wait, out);
}

/// What:
///  Merge each captured layer with the merge and write it back,
/// and name every path the person must look at.
/// Why:
///   An edit away from the merge's lines comes back as it was,
/// a staged edit stays staged,
/// and an edit on the
///  merge's lines keeps both sides between markers,
/// so the command exits 0 with the merge made and the person's work
///  whole (`SPEC.md`,
/// "Merges and commits never fail").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function carry({ context, root, gitDir, changes, capture, out }): Promise<Report>;
/// ```
pub fn carry(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: &Capture,
    wait: LockWait,
    out: &mut dyn std::io::Write,
) -> Result<Report, CarryFailure> {
    // The merge of one layer runs from the top of the worktree, where a relative global option of the invocation does not reach.
    let rooted_context: GitContext = rooted(context);
    let runner: ContextRunner<'_> = runner_of(&rooted_context, root);
    let mut report: Report = Report::default();
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let Some(change) = changes.iter().find(|change| { return change.path == entry.path; }) else { continue };
        let state: WorktreeState = worktree_state(&rooted(context), root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(|file| { return file.blob.as_str(); });
        let merged_object: Option<&str> = change.exists_after().then(|| { return change.new_oid.as_str(); });
        let (worktree, note, target) = plan_worktree((&runner, context, root), change, entry)?;
        // What stands there is the capture, the merge's own, or the bytes this carry writes:
        // a carry that was killed after writing a path finds its
        //  own bytes and writes the same ones again. Anything else was written by someone else since the capture.
        if !explained(&state, &[captured_object, merged_object, target.as_deref()]) {
            report.changed.push(entry.path.clone());
            continue;
        }
        let current: Option<(String, String)> = index_entry(context, root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(|staged| { return (staged.mode.as_str(), staged.oid.as_str()); });
        let merged_entry: Option<(&str, &str)> = change.exists_after().then(|| { return (change.new_mode.as_str(), change.new_oid.as_str()); });
        let old_entry: Option<(&str, &str)> = change.existed().then(|| { return (change.old_mode.as_str(), change.old_oid.as_str()); });
        if !entry_explained(&current, &[captured_entry, merged_entry, old_entry]) {
            report.changed.push(entry.path.clone());
            continue;
        }
        let index: IndexWrite = plan_index(&runner, change, entry)?;
        match note {
            Note::Merged => report.merged.push(entry.path.clone()),
            Note::Conflict => report.conflicts.push(entry.path.clone()),
            Note::Kept(why) => report.kept.push((entry.path.clone(), why)),
            Note::None => {}
        }
        steps.push(Step { path: entry.path.clone(), index, worktree });
    }
    write_steps(context, root, git_dir, steps.as_slice(), wait, out)?;
    return Ok(report);
}

/// What:
///  The worktree side of one captured path.
/// Why:
///  The layer's decision and what the report says of it are one answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function planWorktree(places, change, entry, state): Promise<[FileWrite, Note]>;
/// ```
fn plan_worktree(
    places: (&dyn GitRunner, &GitContext, &Path),
    change: &Change,
    entry: &crate::merge_isolation_carry_parts::CapturedPath,
) -> Result<(FileWrite, Note, Option<String>), CarryFailure> {
    let (runner, context, root) = places;
    let layer: Layer = Layer { base: side(change, false), ours: entry.worktree.as_ref().map(side_of_file), theirs: side(change, true) };
    let decision: Decision = on_the_runtime(decide(runner, entry.path.as_slice(), &layer))?;
    let mode: String = decision.mode.clone().unwrap_or_else(|| { return String::from(PLAIN_FILE_MODE); });
    return match decision.outcome {
        Outcome::Merges => Ok((FileWrite::Leave, Note::None, None)),
        Outcome::Ours => match &entry.worktree {
            Some(file) => Ok((
                FileWrite::Bytes(raw_bytes(context, root, file.raw.as_str()).map_err(CarryFailure::Failed)?, mode),
                Note::None,
                Some(file.blob.clone()),
            )),
            None => Ok((FileWrite::Remove, Note::None, None)),
        },
        Outcome::Merged { bytes, conflict, .. } => {
            let oid: String = on_the_runtime(write_layer_bytes(runner, bytes.as_slice()))?;
            let form: Vec<u8> = file_form(context, root, entry.path.as_slice(), oid.as_str()).map_err(CarryFailure::Failed)?;
            return Ok((FileWrite::Bytes(form, mode), if conflict { Note::Conflict } else { Note::Merged }, Some(oid)));
        }
        Outcome::Kept(why) => match &entry.worktree {
            Some(file) => Ok((
                FileWrite::Bytes(raw_bytes(context, root, file.raw.as_str()).map_err(CarryFailure::Failed)?, mode),
                Note::Kept(why),
                Some(file.blob.clone()),
            )),
            None => Ok((FileWrite::Remove, Note::Kept(why), None)),
        },
        Outcome::Gone(why) => Ok((FileWrite::Remove, Note::Kept(why), None)),
    };
}

/// What:
///  The index side of one captured path.
/// Why:
///   A staged edit is a layer of its own:
/// it stays staged over the merged file when it merges,
/// and holds both
///  sides when it does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function planIndex(runner, context, root, change, entry): Promise<IndexWrite>;
/// ```
fn plan_index(
    runner: &dyn GitRunner,
    change: &Change,
    entry: &crate::merge_isolation_carry_parts::CapturedPath,
) -> Result<IndexWrite, CarryFailure> {
    let base: Option<Side> = side(change, false);
    let ours: Option<Side> = entry.index.as_ref().map(side_of_index);
    if let (Some(found), Some(person)) = (&base, &ours)
        && found.oid == person.oid
        && found.mode == person.mode
    {
        return Ok(IndexWrite::Leave);
    }
    // Nothing of the person's is in this layer, so the merge's entry is the layer's own.
    if base.is_none() && ours.is_none() {
        return Ok(IndexWrite::Leave);
    }
    let layer: Layer = Layer { base, ours: ours.clone(), theirs: side(change, true) };
    let decision: Decision = on_the_runtime(decide(runner, entry.path.as_slice(), &layer))?;
    let mode: String = decision.mode.clone().unwrap_or_else(|| { return String::from(PLAIN_FILE_MODE); });
    return match decision.outcome {
        Outcome::Merges => Ok(IndexWrite::Leave),
        Outcome::Ours | Outcome::Kept(_) => match &entry.index {
            Some(staged) => Ok(IndexWrite::Entry(staged.mode.clone(), staged.oid.clone())),
            None => Ok(IndexWrite::Remove),
        },
        Outcome::Merged { bytes, .. } => {
            let oid: String = on_the_runtime(write_layer_bytes(runner, bytes.as_slice()))?;
            Ok(IndexWrite::Entry(mode, oid))
        }
        Outcome::Gone(_) => Ok(IndexWrite::Remove),
    };
}

/// What:
///  Put the person's layers back exactly as they were,
/// for a merge that did not land.
/// Why:
///   A merge killed after the clear leaves the branch's content where the person's work was;
/// the capture is what puts
///  it back,
/// and nothing of the merge is applied,
/// so each layer is written as captured with no merge at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function restore({ context, root, gitDir, capture, out }): Promise<void>;
/// ```
pub fn restore(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: &Capture,
    wait: LockWait,
    out: &mut dyn std::io::Write,
) -> Result<(), CarryFailure> {
    let mut steps: Vec<Step> = Vec::with_capacity(capture.paths.len());
    for entry in &capture.paths {
        let state: WorktreeState = worktree_state(&rooted(context), root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(|file| { return file.blob.as_str(); });
        // The branch's own content is what a clear leaves, so it is what a restore may replace.
        let change: Option<&Change> = changes.iter().find(|change| { return change.path == entry.path; });
        let old_object: Option<&str> = change.and_then(|found| { return found.existed().then(|| { return found.old_oid.as_str(); }); });
        let merged_object: Option<&str> = change.and_then(|found| { return found.exists_after().then(|| { return found.new_oid.as_str(); }); });
        if !explained(&state, &[captured_object, old_object, merged_object]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let current: Option<(String, String)> = index_entry(context, root, entry.path.as_slice()).map_err(CarryFailure::Failed)?;
        let captured_entry: Option<(&str, &str)> = entry.index.as_ref().map(|staged| { return (staged.mode.as_str(), staged.oid.as_str()); });
        let old_entry: Option<(&str, &str)> = change.and_then(|found| { return found.existed().then(|| { return (found.old_mode.as_str(), found.old_oid.as_str()); }); });
        let merged_entry: Option<(&str, &str)> = change.and_then(|found| { return found.exists_after().then(|| { return (found.new_mode.as_str(), found.new_oid.as_str()); }); });
        if !entry_explained(&current, &[captured_entry, old_entry, merged_entry]) {
            return Err(CarryFailure::Foreign(entry.path.clone()));
        }
        let index: IndexWrite = match entry.index.as_ref() {
            Some(staged) => IndexWrite::Entry(staged.mode.clone(), staged.oid.clone()),
            None => IndexWrite::Remove,
        };
        let worktree: FileWrite = match entry.worktree.as_ref() {
            Some(file) => FileWrite::Bytes(raw_bytes(context, root, file.raw.as_str()).map_err(CarryFailure::Failed)?, file.mode.clone()),
            None => FileWrite::Remove,
        };
        steps.push(Step { path: entry.path.clone(), index, worktree });
    }
    return write_steps(context, root, git_dir, steps.as_slice(), wait, out);
}

/// Whether nothing but the branch's own content is at a captured path,
/// which is what a merge that never landed leaves behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function onlyTheBranchesContent(context, root, changes, capture): Promise<boolean>;
/// ```
pub fn only_the_branches_content(context: &GitContext, root: &Path, changes: &[Change], capture: &Capture) -> Result<bool, String> {
    for entry in &capture.paths {
        let Some(change) = changes.iter().find(|change| { return change.path == entry.path; }) else { continue };
        let state: WorktreeState = worktree_state(&rooted(context), root, entry.path.as_slice())?;
        let captured_object: Option<&str> = entry.worktree.as_ref().map(|file| { return file.blob.as_str(); });
        let old_object: Option<&str> = change.existed().then(|| { return change.old_oid.as_str(); });
        if !explained(&state, &[captured_object, old_object]) {
            return Ok(false);
        }
    }
    return Ok(true);
}

/// The side a captured worktree file is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sideOfFile = (file: CapturedFile): Side => ({ mode: file.mode, oid: file.blob });
/// ```
fn side_of_file(file: &CapturedFile) -> Side {
    return Side { mode: file.mode.clone(), oid: file.blob.clone() };
}

/// The side a captured index entry is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sideOfIndex = (staged: CapturedIndex): Side => ({ mode: staged.mode, oid: staged.oid });
/// ```
fn side_of_index(staged: &CapturedIndex) -> Side {
    return Side { mode: staged.mode.clone(), oid: staged.oid.clone() };
}

/// The writing half of a carry,
/// judged and written one path at a time.
#[cfg(test)]
#[path = "merge_isolation_carry_apply_tests.rs"]
mod tests;
