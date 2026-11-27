//! What:
//!  The directory one commit command runs in:
//!  the effective invocation directory after every `-C` of the global options,
//!  and the real Git context and directory the commit's own Git starts run in.
//! Why:
//!  Relative pathspecs and the transaction's Git queries resolve from where Git itself would run.
//!  A `-C` moves Git before it runs a command,
//!  so the wrapper starts its own Git in the directory the chain already reached and drops the `-C` tokens:
//!  forwarding one would move Git a second time,
//!  and a relative `-C sub` would then be read inside `sub` itself.
//!  `--git-dir`,
//!  `--work-tree` and `--bare` stay,
//!  because Git resolves their relative values after its last `-C`;
//!  a `GIT_DIR` or `GIT_WORK_TREE` in the environment stays for the same reason.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const whereTheCommitRuns = (command) => ({ context: withoutDirectoryChanges(command.context), cwd: effectiveDirectory(...) });
//! ```

/// What the command is handed.
use crate::commit_command_input::CommitCommand;
/// The `-C` tokens are dropped from the global options Git starts with,
/// because the directory already holds what they moved.
use crate::global_arguments::without_directory_changes;
/// The transaction Git wrapper,
///  whose context the directory changes are taken out of.
use crate::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The effective invocation directory:
/// the process directory moved by every `-C` of the global options.
/// Why:
///   Relative pathspecs and the transaction's Git queries resolve from where Git itself would run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseGlobalOptions(args).effectiveCwd
/// ```
pub fn effective_directory(
    start: &Path,
    global_prefix: &[OsString],
) -> PathBuf {
    let mut directory: PathBuf = start.to_path_buf();
    let mut index: usize = 0;
    while index < global_prefix.len() {
        if global_prefix[index] == "-C"
            && let Some(value) = global_prefix.get(index + 1)
        {
            directory = directory.join(value);
            index += 2;
            continue;
        }
        index += 1;
    }
    return directory;
}

/// What:
///  The real Git context and the directory the commit's own Git runs in,
///  worked out once for every start this command makes.
/// Why:
///  A `-C` moves Git before it runs a command,
///  so the wrapper starts its own Git in the directory the chain already reached
///  and drops the `-C` tokens:
///  forwarding one would move Git a second time,
///  and a relative `-C sub` would then be read inside `sub` itself.
///  `--git-dir`,
///  `--work-tree` and `--bare` stay,
///  because Git resolves their relative values after its last `-C`,
///  which is the directory this returns;
///  a `GIT_DIR` or `GIT_WORK_TREE` in the environment stays for the same reason.
///  Every start of this commit reads one answer,
///  so the question asked for a refusal is asked where the commit itself runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whereTheCommitRuns = (command) => ({
///   context: { ...command.context, globalPrefix: withoutDirectoryChanges(command.context.globalPrefix) },
///   cwd: effectiveDirectory(process.cwd(), command.context.globalPrefix),
/// });
/// ```
pub(crate) fn where_the_commit_runs(command: &CommitCommand<'_>) -> (GitContext, PathBuf) {
    // The directory the command was started in, and `.` when the caller named none, which is where Git reads its own
    //  relative options.
    let start: &Path = command.start_directory.path().unwrap_or(Path::new("."));
    let cwd: PathBuf = effective_directory(start, command.context.global_prefix.as_slice());
    let context: GitContext = GitContext {
        global_prefix: without_directory_changes(command.context.global_prefix.as_slice()),
        ..command.context.clone()
    };
    return (context, cwd);
}
