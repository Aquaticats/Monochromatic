//! What:
//!  Where a forwarded command's landing lock and real index live:
//!  one `git rev-parse` that
//!       reports the git directory,
//!  the index Git would use,
//!  and the transaction registry.
//! Why:
//!  An index writer coordinates with landings through the registry's landing lock and the real
//!      index's lock.
//!  A command outside a repository,
//!  or one writing another index,
//!  coordinates with
//!      nothing (`src/index-lock/landing-location.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [gitDir, indexPath, registryRoot] = (await nanoSpawn(gitPath, [...globalArgs, 'rev-parse', ...])).stdout.split('\n');
//! ```

/// Running real Git for the location query.
use super::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The landing lock and index paths of one repository selection.
/// Why:
///   `LandingLocation`;
///  the real index is the worktree's `<git-dir>/index`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingLocation = { cwd: string; gitDir: string; indexPath: string; realIndexPath: string; registryRoot: string };
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
    /// Transaction registry directory that holds the landing lock.
    pub registry_root: PathBuf,
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
/// ['rev-parse', '--path-format=absolute', '--absolute-git-dir', '--git-path', 'index', '--git-path', 'cli-git-transactions']
/// ```
fn layout_arguments() -> Vec<OsString> {
    return ["rev-parse", "--path-format=absolute", "--absolute-git-dir", "--git-path", "index", "--git-path", "cli-git-transactions"]
        .into_iter()
        .map(OsString::from)
        .collect();
}

/// What:
///  Split the layout output into its three fixed lines,
///  refusing any other shape.
/// Why:
///   A missing line would point the lock at the wrong directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [gitDir, indexPath, registryRoot] = stdout.split('\n'); if (!registryRoot) throw new TypeError(...);
/// ```
pub fn parse_layout(stdout: &[u8]) -> Result<LandingLocation, String> {
    let text: String = String::from_utf8_lossy(stdout).into_owned();
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.len() < 3 || lines[0].is_empty() || lines[1].is_empty() || lines[2].is_empty() {
        return Err(format!("Unexpected rev-parse layout: {text:?}"));
    }
    let git_dir: PathBuf = PathBuf::from(lines[0]);
    return Ok(LandingLocation {
        real_index_path: git_dir.join("index"),
        index_path: PathBuf::from(lines[1]),
        registry_root: PathBuf::from(lines[2]),
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
