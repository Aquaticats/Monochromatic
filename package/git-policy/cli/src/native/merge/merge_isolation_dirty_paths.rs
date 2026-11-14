//! What:
//!  The paths a merge that writes or deletes them would discard.
//! Why:
//!  `git status` lists what changed in its own reading of a worktree:
//!  it compares the executable bit alone,
//!  and it never lists a file an ignore rule covers,
//! so an ignored untracked file is work it
//!  does not show,
//! and a merge that deletes or writes over one takes the only copy
//!  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
//! its second finding).
//!  A permission bit Git does not record is not work here:
//! real Git writes over such a file with the mode its entry and the umask give,
//!  and so does this command
//!  (`SPEC.md`,
//!  "Carry edits forward").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const paths = [...statusPaths, ...ignoredUntrackedAmong];
//! ```

/// Names and sets of unsaved work.
use crate::auto_push_dirty_paths::status_paths;
/// The failure a command hands back to native Git.
use crate::merge_isolation_run_types::Bail;
/// The ignored untracked files of the person's that stand at the paths a merge writes,
/// which `git status` never lists.
use crate::merge_isolation_ignored_paths::ignored_untracked_among;
/// One run of Git under the command's own configuration.
use crate::merge_isolation_git_reads::Session;
/// What a Git start wrote.
use crate::transaction_git::GitOutput;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Every path whose worktree or index differs from the branch,
/// untracked files included,
/// and the ignored untracked
///  files of the person's that stand at the paths the merge touches.
/// Why:
///   These are exactly the paths a merge that writes them would discard,
/// and the untracked ones are why native
///  Git refuses a merge that adds a path a file already stands at.
///  `git status` never lists a file an ignore rule covers,
/// so those paths are read from Git's own list of the untracked files the
///  rules cover (`merge_isolation_ignored_paths.rs`);
/// the merge writes over the only copy of their bytes as it would over
///  any other untracked file's.
///  A `chmod 0600` on a tracked `0644` file is work no reading here treats as the person's,
///  because Git records the executable bit alone and a merge writes the file with the mode its
///  entry and the umask give
///  (`SPEC.md`,
///  "Carry edits forward").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dirty = statusPaths(await git(['--no-optional-locks', 'status', '--porcelain=v1', '-z', '--untracked-files=all'])).concat(ignoredUntrackedAmong(context, root, touched));
/// ```
pub(crate) fn dirty_paths(session: &Session, root: &Path, touched: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, Bail> {
    let status: GitOutput = session.run(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
    ])?;
    if !status.succeeded() {
        return Err(Bail::from(format!("git status failed: {}", status.error_text())));
    }
    let mut paths: Vec<Vec<u8>> = status_paths(status.stdout.as_slice());
    for path in ignored_untracked_among(&session.context, root, touched).map_err(Bail::from)? {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    return Ok(paths);
}
