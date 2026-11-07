//! What:
//!  A real-Git stand-in for unit tests that writes one line per start into a log file and then runs the real Git.
//! Why:
//!  A replacement that saves a Git process is only proven when a control sees the start did not happen,
//!  while the real Git remains the oracle for what the answer must be.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const context = { gitPath: recordingShim, overlay, globalPrefix: [] }; // shim appends argv to a log, then execs git
//! ```

/// The fixture root,
///  the real-Git paths and the isolating overlay.
use crate::test_support::{REAL_GIT, executable, isolated_overlay};
/// The Git context the production runners take.
use crate::transaction_git::GitContext;
/// Borrowed and owned paths.
use std::path::{Path, PathBuf};

/// What:
///  A Git context whose executable logs its arguments to a file in `root` and then runs the real Git,
///  with the log's path.
/// Why:
///   Controls count and name the Git starts a function made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { context, log } = recordingContext(root);
/// ```
#[cfg(unix)]
pub(crate) fn recording_context(root: &Path) -> (GitContext, PathBuf) {
    let log: PathBuf = root.join("git-starts.log");
    let shim: PathBuf = root.join("recording-git");
    let script: String = format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec {REAL_GIT} \"$@\"\n", log.display());
    executable(shim.as_path(), script.as_bytes());
    let context: GitContext = GitContext {
        real_git: shim,
        // The in-process readers are off: the recorder exists to see the Git starts of the process path.
        overlay: isolated_overlay().into_iter().chain(crate::repo_read_hook::switch_off()).collect(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    return (context, log);
}

/// What:
///  The arguments of every Git start the log holds,
///  one string per start.
/// Why:
///   A log that was never written means no start happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readFile(log, 'utf8').catch(() => '')).split('\n').filter(Boolean)
/// ```
pub(crate) fn recorded_starts(log: &Path) -> Vec<String> {
    let text: String = std::fs::read_to_string(log).unwrap_or_default();
    let mut starts: Vec<String> = Vec::new();
    for line in text.lines() {
        starts.push(String::from(line));
    }
    return starts;
}

/// The commands whose `--quiet` changes the exit status or what is computed,
/// so a start of one keeps it.
///  Git 2.56.0:
/// `builtin/rev-parse.c` `die_no_single_rev` exits 1 where it dies with 128;
/// `builtin/symbolic-ref.c`
///  `check_symref` returns 1 where it dies;
/// `diff.c` sets `exit_with_status`,
/// no renames and an early quit for `--quiet`.
const RESULT_CHANGING_COMMANDS: [&str; 4] = ["rev-parse", "symbolic-ref", "diff-index", "diff-files"];

/// What:
///  The recorded starts that carry `--quiet` or `-q` as a whole argument,
/// other than the probes whose option changes the answer.
/// Why:
///   The owner's rule is that no Git start of the wrapper or the service gets either option when it only hides messages,
/// so
///  a control names the starts that did,
/// and an empty answer is only worth something next to a check that the start it looks
///  for happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordedStarts(log).filter((start) => hasWord(start, ['--quiet', '-q']) && !RESULT_CHANGING.some((name) => hasWord(start, [name])))
/// ```
pub(crate) fn quiet_starts(log: &Path) -> Vec<String> {
    let mut quiet: Vec<String> = Vec::new();
    for start in recorded_starts(log) {
        let words: Vec<&str> = start.split(' ').collect();
        let carries: bool = words.iter().any(|word| { return *word == "--quiet" || *word == "-q"; });
        let probe: bool = words.iter().any(|word| { return RESULT_CHANGING_COMMANDS.contains(word); });
        if carries && !probe {
            quiet.push(start);
        }
    }
    return quiet;
}
