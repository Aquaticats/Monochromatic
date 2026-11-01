//! What:
//!  Write one corrected file into the private commit index: its bytes as a blob,
//!       then the index entry that names that blob.
//! Why:
//!  Corrections land in the private index, never in the real index or the worktree.
//!  The incumbent patches the private index with `git apply --cached`;
//!       here the policy already holds the corrected bytes, so the blob and the entry are written directly.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const blob = await git(['hash-object', '-w', '--stdin'], { input: after });
//! await git(['update-index', '--add', '--cacheinfo', mode, blob, path], { indexPath });
//! ```

use super::candidate_object::CandidateMode;
use super::path_bytes::os_from_path_bytes;
use super::policy_content::Correction;
use super::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::path::Path;

/// What:
///  The private index and the Git context its writes run in.
/// Why:
///   One value keeps the index, object store and repository together at every write.
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
    /// Object store receiving the blobs; the real store when absent.
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
///   A correction the index cannot take is a failure of the pass, reported with its path.
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
///  Store bytes as a blob and return its object ID.
/// Why:
///   The blob must exist before the index entry names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oid = await git(['hash-object', '-w', '--stdin'], { input: bytes });
/// ```
fn write_blob(stage: &PrivateStage<'_>, bytes: &[u8]) -> Result<String, String> {
    let mut request: GitRequest = GitRequest::new(stage.cwd, &["hash-object", "-w", "--stdin"]);
    request.without_prefix = true;
    request.object_directory = stage.object_directory.map(Path::to_path_buf);
    request.input = Some(bytes.to_vec());
    let output: GitOutput = private_git(stage, &request, "hash-object")?;
    return Ok(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
}

/// What:
///  Point one index entry at a blob.
/// Why:
///   `--add` creates the entry for a path that was not staged, and `--cacheinfo` names the mode and blob directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['update-index', '--add', '--cacheinfo', mode, oid, path], { indexPath });
/// ```
fn update_entry(stage: &PrivateStage<'_>, mode: &str, oid: &str, path: &[u8]) -> Result<(), String> {
    let mut request: GitRequest = GitRequest::new(
        stage.cwd,
        &["update-index", "--add", "--cacheinfo", mode, oid],
    );
    // The path is the exact bytes Git reported, so a name that is not UTF-8 reaches Git unchanged.
    request.arguments.push(os_from_path_bytes(path));
    request.without_prefix = true;
    request.index = Some(stage.index.to_path_buf());
    request.object_directory = stage.object_directory.map(Path::to_path_buf);
    private_git(stage, &request, "update-index")?;
    return Ok(());
}

/// What:
///  Run one private Git command and turn a failure into its message.
/// Why:
///   Every write reports the same way, so a failed write names the command that failed.
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
