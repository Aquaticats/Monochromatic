//! What:
//!  Write one corrected file into the private commit index:
//! its bytes as a blob,
//!       then the index entry that names that blob.
//! Why:
//!  Corrections land in the private index,
//! never in the real index or the worktree.
//!  The incumbent patches the private index with `git apply --cached`;
//!       here the policy already holds the corrected bytes,
//! so the blob and the entry are written directly.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const blob = await git(['hash-object', '-w', '--stdin'], { input: after });
//! await git(['update-index', '--add', '--cacheinfo', mode, blob, path], { indexPath });
//! ```

use crate::candidate_object::CandidateMode;
use crate::path_bytes::os_from_path_bytes;
use crate::policy_content::Correction;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::path::{Path, PathBuf};

/// What:
///  The private index and the Git context its writes run in.
/// Why:
///   One value keeps the index,
/// object store and repository together at every write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrivateStage = { indexPath: string; objectDirectory?: string; cwd: string };
/// ```
#[derive(Clone, Copy)]
pub struct PrivateStage<'a> {
    /// Real Git executable and the invocation's overlay.
    pub context: &'a GitContext,
    /// Repository directory the commands run from.
    pub cwd: &'a Path,
    /// Private commit index receiving the entries.
    pub index: &'a Path,
    /// Object store receiving the blobs;
    /// the real store when absent.
    pub object_directory: Option<&'a Path>,
}

/// What:
///  Git's file mode text for a candidate mode.
/// Why:
///   `update-index --cacheinfo` takes the mode in Git's spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MODES = { regular: '100644', executable: '100755', symlink: '120000', gitlink: '160000' };
/// ```
pub fn git_mode(mode: CandidateMode) -> &'static str {
    match mode {
        CandidateMode::Regular => return "100644",
        CandidateMode::Executable => return "100755",
        CandidateMode::Symlink => return "120000",
        CandidateMode::Gitlink => return "160000",
    }
}

/// What:
///  Write one correction's bytes into the private index.
/// Why:
///   A correction the index cannot take is a failure of the pass,
/// reported with its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stageCorrection({ indexPath, correction });
/// ```
pub fn stage_correction(stage: &PrivateStage<'_>, correction: &Correction) -> Result<(), String> {
    let blob: String = write_blob(stage, correction.after.as_ref())?;
    let mode: &str = git_mode(correction.mode);
    return update_entry(stage, mode, blob.as_str(), correction.path.as_slice());
}


/// What:
///  Write every correction of one pass into the private index in two Git starts,
/// not two for each correction.
/// Why:
///  A formatter or a final-newline rule can correct thousands of files in one pass,
/// and a `hash-object` and an `update-index` for each of them cost a process per file:
/// a thousand corrected files took 12,181 ms against real Git's 136 ms on 2026-11-16,
/// of which the per-file `update-index` starts were about ten seconds.
/// The corrected bytes are written beside the index, one file each, because
/// `git hash-object --stdin-paths` is the one Git form that hashes many blobs in one start;
/// the entries then go in through `update-index --index-info`,
/// whose NUL-separated records carry a path of any bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stageCorrections({ indexPath, corrections }); // one hash-object, one update-index
/// ```
///
/// # Errors
///
/// The words of the Git step that failed, or of the batch directory that could not be made.
pub fn stage_corrections(stage: &PrivateStage<'_>, corrections: &[Correction]) -> Result<(), String> {
    if corrections.is_empty() {
        return Ok(());
    }
    let directory: PathBuf = stage.index.with_extension("corrected");
    if let Err(error) = std::fs::remove_dir_all(&directory)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        return Err(format!("removing {} failed: {error}", directory.display()));
    }
    if let Err(error) = std::fs::create_dir_all(&directory) {
        return Err(format!("making {} failed: {error}", directory.display()));
    }
    let outcome: Result<(), String> = stage_all_corrections(stage, corrections, directory.as_path());
    let _removed = std::fs::remove_dir_all(&directory);
    return outcome;
}

/// What:
///  The two steps of `stage_corrections`,
///  with the batch directory already made.
/// Why:
///  The directory is removed whatever these answer,
///  so the caller has one place where a failed batch leaves nothing.
fn stage_all_corrections(stage: &PrivateStage<'_>, corrections: &[Correction], directory: &Path) -> Result<(), String> {
    let mut files: Vec<PathBuf> = Vec::with_capacity(corrections.len());
    for (position, correction) in corrections.iter().enumerate() {
        let file: PathBuf = directory.join(position.to_string());
        std::fs::write(&file, correction.after.as_ref()).map_err(|error| format!("writing the corrected bytes of {} failed: {error}", file.display()))?;
        files.push(file);
    }
    let blobs: Vec<String> = write_blobs_through_git(stage, files.as_slice())?;
    return update_entries_through_git(stage, corrections, blobs.as_slice());
}

/// What:
///  Hash and write every file of a batch as a blob in one Git start,
///  answering the object names in the order the files were given.
/// Why:
///  The per-correction form started Git once for each blob;
/// a batch of corrected bytes is already on the disk, so one `hash-object --stdin-paths` writes them all.
fn write_blobs_through_git(stage: &PrivateStage<'_>, files: &[PathBuf]) -> Result<Vec<String>, String> {
    let mut input: Vec<u8> = Vec::new();
    for file in files {
        let Some(line) = path_line(file) else {
            return Err(String::from("a batch path holds a newline, which hash-object --stdin-paths cannot read"));
        };
        input.extend_from_slice(line.as_slice());
        input.push(b'\n');
    }
    let began: std::time::Instant = std::time::Instant::now();
    let mut request: GitRequest = GitRequest::new(stage.cwd, &["hash-object", "-w", "--no-filters", "--stdin-paths"]);
    request.without_prefix = true;
    request.repository_selection = crate::repository_selection::from_global_options(stage.cwd, stage.context.global_prefix.as_slice());
    request.object_directory = stage.object_directory.map(Path::to_path_buf);
    request.input = Some(input);
    let output: GitOutput = private_git(stage, &request, "hash-object")?;
    crate::prepare_work_spans::note_span(crate::prepare_work_spans::POLICY_BLOB_WRITE, began.elapsed());
    let blobs: Vec<String> = String::from_utf8_lossy(output.stdout.as_slice()).lines().map(str::trim).filter(|line| return !line.is_empty()).map(str::to_string).collect();
    if blobs.len() != files.len() {
        return Err(format!("git hash-object answered {} names for {} files", blobs.len(), files.len()));
    }
    return Ok(blobs);
}

/// What:
///  Point every corrected path's entry at its blob in one Git start.
/// Why:
///  `--index-info` reads one mode, object and path per NUL-terminated record,
/// so a path with a tab or a newline in it reaches Git as the bytes it is,
/// and a pass of thousands of corrections costs one child.
fn update_entries_through_git(stage: &PrivateStage<'_>, corrections: &[Correction], blobs: &[String]) -> Result<(), String> {
    let mut input: Vec<u8> = Vec::new();
    for (correction, blob) in corrections.iter().zip(blobs.iter()) {
        input.extend_from_slice(format!("{} {}\t", git_mode(correction.mode), blob).as_bytes());
        input.extend_from_slice(correction.path.as_slice());
        input.push(0);
    }
    let began: std::time::Instant = std::time::Instant::now();
    let mut request: GitRequest = GitRequest::new(stage.cwd, &["update-index", "--add", "-z", "--index-info"]);
    request.without_prefix = true;
    request.repository_selection = crate::repository_selection::from_global_options(stage.cwd, stage.context.global_prefix.as_slice());
    request.index = Some(stage.index.to_path_buf());
    request.object_directory = stage.object_directory.map(Path::to_path_buf);
    request.input = Some(input);
    private_git(stage, &request, "update-index")?;
    crate::prepare_work_spans::note_span(crate::prepare_work_spans::POLICY_ENTRY_WRITE, began.elapsed());
    return Ok(());
}

/// What:
///  One path's bytes and a newline, as a line `hash-object --stdin-paths` reads.
/// Why:
///  The line format has no escape, so a path holding a newline is refused here
///  and the caller stages the corrections one at a time instead.
#[cfg(unix)]
fn path_line(path: &Path) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    let bytes: &[u8] = path.as_os_str().as_bytes();
    if bytes.contains(&b'\n') {
        return None;
    }
    return Some(bytes.to_vec());
}

/// A platform whose paths this batch does not spell: every batch falls back to one correction at a time.
#[cfg(not(unix))]
fn path_line(_path: &Path) -> Option<Vec<u8>> {
    return None;
}

/// What:
///  Store bytes as a blob and return its object ID.
/// Why:
///   The blob must exist before the index entry names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oid = await git(['hash-object', '-w', '--stdin'], { input: bytes });
/// ```
fn write_blob(stage: &PrivateStage<'_>, bytes: &[u8]) -> Result<String, String> {
    // A correction's bytes become a blob before any entry can name them; the write is the preparation's own work and is named.
    let began: std::time::Instant = std::time::Instant::now();
    let written: Result<String, String> = write_blob_through_git(stage, bytes);
    crate::prepare_work_spans::note_span(crate::prepare_work_spans::POLICY_BLOB_WRITE, began.elapsed());
    return written;
}

/// What:
///  `write_blob` without the note of how long the write took.
/// Why:
///   A write that failed is still time the preparation spent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oid = await git(['hash-object', '-w', '--stdin'], { input: bytes });
/// ```
fn write_blob_through_git(stage: &PrivateStage<'_>, bytes: &[u8]) -> Result<String, String> {
    let mut request: GitRequest = GitRequest::new(stage.cwd, &["hash-object", "-w", "--stdin"]);
    request.without_prefix = true;
    // The private state is the invocation's, and so is the repository: a Git directory beside its work tree is named here, because
    //  this write keeps none of the caller's other options.
    request.repository_selection = crate::repository_selection::from_global_options(stage.cwd, stage.context.global_prefix.as_slice());
    request.object_directory = stage.object_directory.map(Path::to_path_buf);
    request.input = Some(bytes.to_vec());
    let output: GitOutput = private_git(stage, &request, "hash-object")?;
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// What:
///  Point one index entry at a blob.
/// Why:
///   `--add` creates the entry for a path that was not staged,
/// and `--cacheinfo` names the mode and blob directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['update-index', '--add', '--cacheinfo', mode, oid, path], { indexPath });
/// ```
fn update_entry(stage: &PrivateStage<'_>, mode: &str, oid: &str, path: &[u8]) -> Result<(), String> {
    // One `git update-index` per corrected path per pass, so a record that names it says whether the rewrites are the cost.
    let began: std::time::Instant = std::time::Instant::now();
    let pointed: Result<(), String> = update_entry_through_git(stage, mode, oid, path);
    crate::prepare_work_spans::note_span(crate::prepare_work_spans::POLICY_ENTRY_WRITE, began.elapsed());
    return pointed;
}

/// What:
///  `update_entry` without the note of how long the write took.
/// Why:
///   A write that failed is still time the preparation spent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['update-index', '--add', '--cacheinfo', mode, oid, path], { indexPath });
/// ```
fn update_entry_through_git(stage: &PrivateStage<'_>, mode: &str, oid: &str, path: &[u8]) -> Result<(), String> {
    let mut request: GitRequest = GitRequest::new(
        stage.cwd,
        &["update-index", "--add", "--cacheinfo", mode, oid],
    );
    // The path is the exact bytes Git reported, so a name that is not UTF-8 reaches Git unchanged.
    request.arguments.push(os_from_path_bytes(path));
    request.without_prefix = true;
    request.repository_selection = crate::repository_selection::from_global_options(stage.cwd, stage.context.global_prefix.as_slice());
    request.index = Some(stage.index.to_path_buf());
    request.object_directory = stage.object_directory.map(Path::to_path_buf);
    private_git(stage, &request, "update-index")?;
    return Ok(());
}

/// What:
///  Run one private Git command and turn a failure into its message.
/// Why:
///   Every write reports the same way,
/// so a failed write names the command that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await git(args, { indexPath });
/// ```
fn private_git(stage: &PrivateStage<'_>, request: &GitRequest, name: &str) -> Result<GitOutput, String> {
    let output: GitOutput = match run_git(stage.context, request) {
        Ok(found) => found,
        Err(error) => return Err(format!("starting git {name}: {error}")),
    };
    if !output.succeeded() {
        return Err(format!("git {name} failed: {}", output.error_text()));
    }
    return Ok(output);
}
