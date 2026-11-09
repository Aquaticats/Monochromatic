//! What:
//!  Where a forwarded command's real index lives:
//!  one `git rev-parse` that
//!       reports the git directory
//!  and the index Git would use.
//! Why:
//!  An index writer waits for the commit service's hold on the real index.
//!  A command outside a repository,
//!  or one writing another index,
//!  waits for
//!      nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [gitDir, indexPath] = (await nanoSpawn(gitPath, [...globalArgs, 'rev-parse', ...])).stdout.split('\n');
//! ```

/// Running real Git for the location query.
use crate::path_bytes::os_from_path_bytes;
use crate::repo_read_adapter::landing_of;
use crate::git_system_config::with_real_git;
use crate::repo_read_layout::read_layout;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The index paths of one repository selection.
/// Why:
///   `LandingLocation`;
///  the real index is the worktree's `<git-dir>/index`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingLocation = { cwd: string; gitDir: string; indexPath: string; realIndexPath: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LandingLocation {
    /// Per-worktree Git directory.
    pub git_dir: PathBuf,
    /// Index Git would write,
    ///  honouring `GIT_INDEX_FILE`.
    pub index_path: PathBuf,
    /// The worktree's real index,
    ///  `<git-dir>/index`.
    pub real_index_path: PathBuf,
}

/// What:
///  The arguments of the layout query,
///  after the selection's global options.
/// Why:
///   One process reports every path the coordination needs,
///  in a fixed order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['rev-parse', '--path-format=absolute', '--absolute-git-dir', '--git-path', 'index']
/// ```
fn layout_arguments() -> Vec<OsString> {
    return ["rev-parse", "--path-format=absolute", "--absolute-git-dir", "--git-path", "index"]
        .into_iter()
        .map(OsString::from)
        .collect();
}

/// What:
///  Whether a byte ends a line of Git's layout output.
/// Why:
///   `split` takes a named predicate,
///  which keeps the parser free of a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0x0a
/// ```
fn is_line_feed(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Split the layout output into its two fixed lines,
///  refusing any other shape.
/// Why:
///   A missing line would point the lock at the wrong directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [gitDir, indexPath] = stdout.split('\n'); if (!indexPath) throw new TypeError(...);
/// ```
pub fn parse_layout(stdout: &[u8]) -> Result<LandingLocation, String> {
    let mut lines = stdout.split(is_line_feed);
    let git_dir_bytes: &[u8] = lines.next().unwrap_or_default();
    let index_bytes: &[u8] = lines.next().unwrap_or_default();
    if git_dir_bytes.is_empty() || index_bytes.is_empty() {
        // A refusal names the text a person reads, with replacement characters for bytes that are not UTF-8.
        return Err(format!("Unexpected rev-parse layout: {:?}", String::from_utf8_lossy(stdout)));
    }
    // The paths stay bytes (`path_bytes.rs`): a lock or a transaction registry placed at the
    //  text with replacement characters would sit where no file is, and two different names
    //  would become the same one.
    let git_dir: PathBuf = PathBuf::from(os_from_path_bytes(git_dir_bytes));
    return Ok(LandingLocation {
        real_index_path: git_dir.join("index"),
        index_path: PathBuf::from(os_from_path_bytes(index_bytes)),
        git_dir,
    });
}

/// What:
///  Resolve the landing location for the global options of one command.
/// Why:
///   `resolveLandingLocation`:
///  outside a repository the answer is nothing,
///  not an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveLandingLocation({ gitPath, globalArgs }): Promise<LandingLocation | typeof LANDING_LOCATION_ABSENT>;
/// ```
pub fn resolve_landing_location(
    context: &GitContext,
    global_arguments: &[OsString],
) -> Result<Option<LandingLocation>, String> {
    // `mut` lets the request carry the layout arguments after the global options.
    // A plain layout is read from the file system; Git is asked only when the reader declines it.
    if let Some(found) = read_layout(global_arguments, with_real_git(context.overlay.as_slice(), context.real_git.as_path()).as_slice()).as_ref().and_then(landing_of) {
        return Ok(Some(found));
    }
    let mut arguments: Vec<OsString> = global_arguments.to_vec();
    arguments.extend(layout_arguments());
    let mut request: GitRequest = GitRequest::new(Path::new("."), arguments.as_slice());
    request.without_prefix = true;
    match run_git(context, &request) {
        Ok(output) => return answer_of(&output),
        Err(error) => return Err(error.to_string()),
    }
}

/// What:
///  The landing location a finished query answers with:
///  none when Git refused,
///  otherwise the parsed layout.
/// Why:
///   A refusal from Git means the command is not inside a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (error instanceof SubprocessError) return LANDING_LOCATION_ABSENT;
/// ```
fn answer_of(output: &GitOutput) -> Result<Option<LandingLocation>, String> {
    if !output.succeeded() {
        return Ok(None);
    }
    return parse_layout(output.stdout.as_slice()).map(Some);
}

/// Location controls stay out of the release executable.
#[cfg(test)]
#[path = "landing_location_tests.rs"]
mod tests;
