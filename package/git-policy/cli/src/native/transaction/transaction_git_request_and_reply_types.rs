//! What:
//!  The request that starts one real Git,
//! the reply it gets back,
//! and the failure that names the command:
//!  `GitContext`,
//! `GitRequest`,
//! `GitOutput`,
//! `GitFailure` and the text of a command line.
//! Why:
//!  The runner (`transaction_git.rs`),
//! the rules that say which steps may run twice or be ended,
//! and the step bounds
//!  all speak in these types;
//! keeping them in a file that uses none of them lets each of those files use it
//!  without any of them using another in a circle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // transactionGitTypes.ts: export type GitContext, GitRequest, GitOutput; export class GitFailure; export function commandText();
//! ```

/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The real Git executable and the variables every Git child receives.
/// Why:
///   Transaction code is handed this once and starts every Git through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitContext = { gitPath: string; overlay: [string, string][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitContext {
    /// Absolute real Git executable.
    pub real_git: PathBuf,
    /// Variables added to every child:
    ///  lock PID injection and the forward-target marker.
    pub overlay: Vec<(OsString, OsString)>,
    /// Global options of the invocation,
    ///  kept before every subcommand (`-C`,
    ///  `--git-dir`,
    ///  `-c`).
    pub global_prefix: Vec<OsString>,
    /// The repository's common Git directory,
    ///  where the input file of a request that carries input is made (`cli-git-scratch` inside it);
    ///  `None` where the repository is not yet known,
    ///  and a request with input then fails naming the defect.
    pub common_dir: Option<PathBuf>,
}

/// What:
///  One Git start:
///  where it runs,
///  its arguments after the global prefix,
///  and its
///       environment and input adjustments.
/// Why:
///   A builder keeps call sites short while every option stays explicit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitRequest = { cwd: string; args: string[]; indexPath?: string; objectDirectory?: string; environment?: Record<string, string>; unset?: string[]; input?: Uint8Array };
/// ```
#[derive(Clone, Debug, Default)]
pub struct GitRequest {
    /// Working directory.
    pub cwd: PathBuf,
    /// Arguments after the global prefix.
    pub arguments: Vec<OsString>,
    /// Whether the invocation's global prefix is left out (shadow and private-store commands).
    pub without_prefix: bool,
    /// Private index,
    ///  as `GIT_INDEX_FILE`.
    pub index: Option<PathBuf>,
    /// Object store receiving every written object,
    ///  as `GIT_OBJECT_DIRECTORY`.
    pub object_directory: Option<PathBuf>,
    /// Variables added after the overlay.
    pub environment: Vec<(OsString, OsString)>,
    /// Inherited variables removed before the additions apply.
    pub unset: Vec<OsString>,
    /// Bytes on standard input;
    ///  none means an empty input.
    pub input: Option<Vec<u8>>,
}

/// Builder methods for one request.
impl GitRequest {
    /// What:
    ///  A request running `arguments` in `cwd`.
    /// Why:
    ///   Most requests need nothing else.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { cwd, args }
    /// ```
    pub fn new<S: AsRef<OsStr>>(cwd: &Path, arguments: &[S]) -> Self {
        // `mut` allows collecting the arguments one at a time.
        let mut owned: Vec<OsString> = Vec::with_capacity(arguments.len());
        for argument in arguments {
            owned.push(argument.as_ref().to_os_string());
        }
        return Self {
            cwd: cwd.to_path_buf(),
            arguments: owned,
            ..Self::default()
        };
    }

    /// What:
    ///  This request with every pathspec-mode variable of the caller's environment removed.
    /// Why:
    ///   A pathspec the wrapper writes is the wrapper's own,
    /// so no mode the caller set may change which paths it selects:
    ///  with `GIT_ICASE_PATHSPECS=1`,
    /// `:(top,literal)a.txt` matches `A.txt` too,
    ///  and `git reset` or `git rm` then changes an entry the batch never selected.
    ///  No option turns case-insensitive matching off:
    /// measured against Git 2.56.0,
    /// `--no-icase-pathspecs` is an unknown global.
    ///  Git also refuses two modes at once:
    /// `GIT_LITERAL_PATHSPECS=1` beside `GIT_ICASE_PATHSPECS=1` stops it with
    ///  "global 'literal' pathspec setting is incompatible with all other global pathspec settings",
    /// and `GIT_GLOB_PATHSPECS=1` beside `GIT_NOGLOB_PATHSPECS=1` with
    ///  "global 'glob' and 'noglob' pathspec settings are incompatible".
    ///  `GIT_ICASE_PATHSPECS=1` alone makes `git ls-tree` refuse every pathspec
    ///  ("pathspec magic not supported by this command:
    /// 'icase'").
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const withoutPathspecModes = (request: GitRequest) => ({ ...request, unset: [...request.unset, ...PATHSPEC_MODE_VARIABLES] });
    /// ```
    pub fn without_pathspec_modes(mut self) -> Self {
        for name in crate::conflict_side_modes::MODE_VARIABLES {
            self.unset.push(OsString::from(name));
        }
        return self;
    }
}

/// What:
///  How one Git start ended and what it wrote.
/// Why:
///   Callers decide on the exit code and parse standard output as bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitOutput = { stdout: Uint8Array; stderr: string; exitCode: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitOutput {
    /// Exit code;
    ///  `None` when a signal ended Git.
    pub code: Option<i32>,
    /// Standard output bytes.
    pub stdout: Vec<u8>,
    /// Standard error bytes.
    pub stderr: Vec<u8>,
}

/// Methods over a finished Git start.
impl GitOutput {
    /// What:
    ///  Whether Git exited with status 0.
    /// Why:
    ///   Most callers only need success or failure.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// output.exitCode === 0
    /// ```
    pub fn succeeded(&self) -> bool {
        return self.code == Some(0);
    }

    /// What:
    ///  Standard error as trimmed,
    ///  lossily decoded text for diagnostics.
    /// Why:
    ///   Failure messages quote what Git said.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// output.stderr.trim()
    /// ```
    pub fn error_text(&self) -> String {
        return String::from_utf8_lossy(self.stderr.as_slice())
            .trim()
            .to_string();
    }
}

/// What:
///  Why a Git start failed.
/// Why:
///   A start that could not happen and a Git that exited unsuccessfully are reported with
///       the command named,
///  as `CommitTransactionGitError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CommitTransactionGitError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitFailure(pub String);

/// What:
///  The failure message as display text.
/// Why:
///   Diagnostics print it unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.message
/// ```
impl std::fmt::Display for GitFailure {
    /// Writes the message.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.0.as_str());
    }
}

/// What:
///  The arguments of a request as display text,
///  `git <arguments>`.
/// Why:
///   Failure messages name the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `git ${args.join(' ')}`
/// ```
pub fn command_text(arguments: &[OsString]) -> String {
    let mut text: String = String::from("git");
    for argument in arguments {
        text.push(' ');
        text.push_str(argument.to_string_lossy().as_ref());
    }
    return text;
}
