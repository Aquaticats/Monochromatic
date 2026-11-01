//! What:
//!  Ask real Git which repository and worktree an invocation selects.
//! Why:
//!  Configuration lives at the worktree top level,
//!  and worktree policies need to
//!      know main versus linked.
//!  Git applies `-C`,
//!  `--git-dir`,
//!  `--work-tree` and its
//!      environment itself,
//!  so the wrapper never re-implements that selection.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const identity = await resolveGitWorktreeIdentity({ args, gitPath });
//! ```

/// Import the captured-query runner and its byte helpers.
use super::git_metadata::{MetadataOutput, path_from_git_bytes, run_metadata_git, strip_git_line};
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   The caller's global options are replayed to Git unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string[] argv, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};

/// What:
///  The repository shape an invocation selects.
///       `#[derive(...)]` generates cloning,
///  debug printing and `==`.
/// Why:
///   Each shape needs different handling:
///  no repository means no configuration,
///       a bare repository has no worktree,
///  and main versus linked decides worktree
///       policies.
///  All paths are Git's own canonical absolute paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeIdentity = { kind: 'outside-worktree' } | { kind: 'bare-repository'; ... } | ...;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorktreeIdentity {
    /// No repository,
    ///  or a repository location without a worktree (inside `.git`).
    OutsideWorktree,
    /// A bare repository:
    ///  Git directories but no worktree.
    BareRepository {
        /// Canonical common Git directory.
        common_dir: PathBuf,
        /// Canonical invocation-specific Git directory.
        git_dir: PathBuf,
    },
    /// The main worktree:
    ///  its Git directory is the common directory.
    MainWorktree {
        /// Canonical common Git directory.
        common_dir: PathBuf,
        /// Canonical invocation-specific Git directory.
        git_dir: PathBuf,
        /// Canonical worktree top level,
        ///  where `cli-git.config.jsonc` lives.
        worktree_root: PathBuf,
    },
    /// A linked worktree:
    ///  its Git directory is separate from the common directory.
    LinkedWorktree {
        /// Canonical common Git directory.
        common_dir: PathBuf,
        /// Canonical invocation-specific Git directory.
        git_dir: PathBuf,
        /// Canonical worktree top level,
        ///  where `cli-git.config.jsonc` lives.
        worktree_root: PathBuf,
    },
}

/// What:
///  The `rev-parse` options of the identity query,
///  in output order.
///       `&[&str]` is a borrowed list of borrowed strings compiled into the program.
/// Why:
///   One query answers everything.
///  `--show-toplevel` is last because Git stops
///       there in a bare repository after already printing the first three lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const IDENTITY_QUERY = ['rev-parse', '--path-format=absolute', ...] as const;
/// ```
pub const IDENTITY_QUERY: &[&str] = &[
    "rev-parse",
    "--path-format=absolute",
    "--is-bare-repository",
    "--git-dir",
    "--git-common-dir",
    "--show-toplevel",
];

/// What:
///  Build the error for identity output this module cannot interpret.
///       `std::io::Error::other` wraps a message as an input/output error.
/// Why:
///   Guessing a repository identity from unexpected output could apply the wrong
///       configuration;
///  failing names the limitation instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function malformed(detail: string): Error;
/// ```
fn malformed(detail: &str) -> std::io::Error {
    // `format!` builds the owned message text.
    return std::io::Error::other(format!(
        "cli-git could not classify the Git worktree: git rev-parse returned {detail}. \
         Repository paths containing line feeds are not supported."
    ));
}

/// What:
///  Interpret the identity query's exit state and output.
///       `std::io::Result<T>` is `Result<T, std::io::Error>`.
/// Why:
///   Success prints four lines.
///  Failure after three lines is a repository without
///       a worktree (bare when the first line says so).
///  Failure with no output is
///       "not a repository".
///  Any other shape is rejected rather than guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseIdentityOutput(success: boolean, stdout: Buffer): WorktreeIdentity;
/// ```
pub fn parse_identity_output(success: bool, stdout: &[u8]) -> std::io::Result<WorktreeIdentity> {
    if stdout.is_empty() {
        if success {
            // `Err(...)` is the failure variant.
            return Err(malformed("no output"));
        }
        // `Ok(...)` is the success variant.
        return Ok(WorktreeIdentity::OutsideWorktree);
    }
    // What: `Vec<&[u8]>` is an owned list of borrowed byte lines; `.split(...)` with a
    //       named predicate cuts at each line feed.
    // Why:  Each value Git printed occupies one line once the final line break is removed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lines = stripGitLine(stdout).split('\n');
    // ```
    let mut lines: Vec<&[u8]> = Vec::<&[u8]>::new();
    for line in strip_git_line(stdout).split(is_line_feed) {
        // A Windows Git may end lines with `\r\n`; the `\r` is not part of the value.
        lines.push(line.strip_suffix(b"\r").unwrap_or(line));
    }
    let expected: usize = if success { 4 } else { 3 };
    if lines.len() != expected {
        return Err(malformed(
            format!("{} lines where {expected} were expected", lines.len()).as_str(),
        ));
    }
    let bare: bool = lines[0] == b"true";
    if !bare && lines[0] != b"false" {
        return Err(malformed("an unknown bare-repository flag"));
    }
    // What: `let (Some(a), Some(b)) = (x, y) else { ... };` unwraps both or exits.
    // Why:  An empty directory line is not a path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (gitDir === '' || commonDir === '') throw malformed('an empty Git directory');
    // ```
    let (Some(git_dir), Some(common_dir)) =
        (path_from_git_bytes(lines[1]), path_from_git_bytes(lines[2]))
    else {
        return Err(malformed("an empty Git directory"));
    };
    if !success {
        if bare {
            return Ok(WorktreeIdentity::BareRepository {
                common_dir,
                git_dir,
            });
        }
        return Ok(WorktreeIdentity::OutsideWorktree);
    }
    let Some(worktree_root) = path_from_git_bytes(lines[3]) else {
        return Err(malformed("an empty worktree top level"));
    };
    if git_dir == common_dir {
        return Ok(WorktreeIdentity::MainWorktree {
            common_dir,
            git_dir,
            worktree_root,
        });
    }
    return Ok(WorktreeIdentity::LinkedWorktree {
        common_dir,
        git_dir,
        worktree_root,
    });
}

/// Named predicate for splitting output into lines;
///  `&u8` borrows one byte.
fn is_line_feed(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Resolve the identity selected by the caller's global Git options.
///       `&[OsString]` borrows the arguments before the subcommand,
///  unchanged.
/// Why:
///   Replaying the complete global prefix lets Git apply `-C` chains through
///       symbolic links,
///  `--git-dir`,
///  `--work-tree` and environment selection exactly
///       as it will for the forwarded command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveWorktreeIdentity(realGit, globalPrefix, overlay): Promise<WorktreeIdentity>;
/// ```
pub fn resolve_worktree_identity(
    real_git: &Path,
    global_prefix: &[OsString],
    overlay: &[(OsString, OsString)],
) -> std::io::Result<WorktreeIdentity> {
    // `.to_vec()` copies the borrowed prefix into an owned, growable argument list.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    // `for part in IDENTITY_QUERY` visits each compiled-in option.
    for part in IDENTITY_QUERY {
        arguments.push(OsString::from(part));
    }
    // A trailing `?` returns a start failure to our caller, or unwraps the output.
    let output: MetadataOutput = run_metadata_git(real_git, arguments.as_slice(), overlay)?;
    return parse_identity_output(output.success, output.stdout.as_slice());
}

/// What:
///  The worktree top level of an identity,
///  when it has one.
///       `Option<&Path>` is "a borrowed path or nothing".
/// Why:
///   Configuration is loaded only for main and linked worktrees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeRoot(identity: WorktreeIdentity): string | undefined;
/// ```
pub fn worktree_root(identity: &WorktreeIdentity) -> Option<&Path> {
    // What: `match` on the borrowed identity; `{ worktree_root, .. }` binds one field
    //       and ignores the rest; `|` joins two patterns sharing one arm.
    // Why:  Only the two worktree shapes carry a top level.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return 'worktreeRoot' in identity ? identity.worktreeRoot : undefined;
    // ```
    match identity {
        WorktreeIdentity::MainWorktree { worktree_root, .. }
        | WorktreeIdentity::LinkedWorktree { worktree_root, .. } => {
            return Some(worktree_root.as_path());
        }
        WorktreeIdentity::OutsideWorktree | WorktreeIdentity::BareRepository { .. } => {
            return None;
        }
    }
}

/// What:
///  The common Git directory of an identity,
///  when the invocation is in a repository.
/// Why:
///   Private repository storage such as the scratch directory lives there for every repository shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commonDirectory(identity: WorktreeIdentity): string | undefined;
/// ```
pub fn common_directory(identity: &WorktreeIdentity) -> Option<&Path> {
    match identity {
        WorktreeIdentity::BareRepository { common_dir, .. }
        | WorktreeIdentity::MainWorktree { common_dir, .. }
        | WorktreeIdentity::LinkedWorktree { common_dir, .. } => {
            return Some(common_dir.as_path());
        }
        WorktreeIdentity::OutsideWorktree => {
            return None;
        }
    }
}

/// Disposable-repository controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_identity_tests.rs"]
mod tests;
