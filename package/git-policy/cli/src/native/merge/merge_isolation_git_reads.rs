//! What:
//!  The questions an isolated merge asks Git and the terminal:
//! the merge settings,
//! the unsaved paths,
//! the fetch head,
//! and whether an editor would open.
//! Why:
//!  These steps only read,
//! and the merge and its apply step both need them,
//! so they live in a file of their own that imports none of the files that use them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const config = readConfig(session); const unsaved = unsavedPaths(session);
//! ```

/// Names and sets of unsaved work.
use crate::auto_push_dirty_paths::status_paths;
/// The merge messages.
use crate::merge_isolation_message::fetch_head_merge_heads;
/// Real Git.
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// Owned operating-system text.
use crate::merge_isolation_run_types::{Bail, MergeConfig};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::Path;

/// Whether one byte is the NUL byte that ends each entry of a `-z` listing.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Read Git's boolean spellings.
/// Why:
///   Settings accept `true`,
/// `yes`,
/// `on`,
/// `1` and their opposites in any letter case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMaybeBool(text: string): boolean | undefined;
/// ```
pub fn parse_maybe_bool(text: &str) -> Option<bool> {
    match text.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => return Some(true),
        "false" | "no" | "off" | "0" | "" => return Some(false),
        _ => return None,
    }
}

/// What:
///  Whether `git merge` would open an editor for the message.
/// Why:
///   `GIT_MERGE_AUTOEDIT` decides when it is set;
/// otherwise the editor opens when standard input and output are one
///  terminal.
///  A merge that would open one over a checkout that holds nothing of the person's own work is still native Git's,
///  which opens the editor itself;
/// over that work the wrapper opens it (`merge_isolation_editor.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function editorWouldOpen({ autoEdit, sameTerminal }): boolean | undefined;
/// ```
pub fn editor_would_open(auto_edit: Option<&str>, same_terminal: bool) -> Option<bool> {
    match auto_edit {
        Some(text) => return parse_maybe_bool(text),
        None => return Some(same_terminal),
    }
}

/// The device and inode a descriptor names,
/// to tell whether two descriptors are the same file;
/// nothing when it cannot be read.
#[cfg(unix)]
fn device_and_inode(descriptor: std::os::fd::BorrowedFd<'_>) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let file = std::fs::File::from(descriptor.try_clone_to_owned().ok()?);
    let metadata = file.metadata().ok()?;
    return Some((metadata.dev(), metadata.ino()));
}

/// What:
///  Whether standard input and standard output are the same terminal.
/// Why:
///   Git's `default_edit_option` asks exactly this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.stdin.isTTY && process.stdout.isTTY && sameDevice(0, 1)
/// ```
pub fn stdin_and_stdout_are_one_terminal() -> bool {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::fd::AsFd;
        return device_and_inode(std::io::stdin().as_fd()) == device_and_inode(std::io::stdout().as_fd());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return true;
    }
}

/// One session of queries against the invocation's repository.
pub struct Session {
    /// Real Git with the invocation's global options as a prefix.
    pub context: GitContext,
}

/// Queries.
impl Session {
    /// Run Git and return its output whatever the exit status.
    pub fn run(&self, arguments: &[&str]) -> Result<GitOutput, String> {
        return match run_git(&self.context, &GitRequest::new(Path::new("."), arguments)) {
            Ok(output) => Ok(output),
            Err(error) => Err(format!("git {} could not start: {error}", arguments.join(" "))),
        };
    }

    /// Run Git with bytes on standard input.
    pub fn run_with_input(&self, arguments: &[&str], input: Vec<u8>) -> Result<GitOutput, String> {
        let mut request: GitRequest = GitRequest::new(Path::new("."), arguments);
        request.input = Some(input);
        return match run_git(&self.context, &request) {
            Ok(output) => Ok(output),
            Err(error) => Err(format!("git {} could not start: {error}", arguments.join(" "))),
        };
    }

    /// Run Git,
    /// require success,
    /// and return its trimmed standard output.
    pub fn text(&self, arguments: &[&str]) -> Result<String, String> {
        let output: GitOutput = self.run(arguments)?;
        if !output.succeeded() {
            return Err(format!("git {} failed: {}", arguments.join(" "), output.error_text()));
        }
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }
}

/// What:
///  Read the settings in one Git call.
/// Why:
///   The settings that change what a merge does but that this path does not reproduce defer the whole command to
///  native Git;
/// the others (`merge.ff`,
/// `merge.stat`,
/// `commit.gpgSign`) are honored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readMergeConfig(session): Promise<MergeConfig>;
/// ```
pub fn read_config(session: &Session) -> Result<MergeConfig, String> {
    let output: GitOutput = session.run(&[
        "config",
        "--null",
        "--get-regexp",
        "^(merge\\.(ff|stat|verifysignatures|autostash)|branch\\..*\\.mergeoptions|commit\\.gpgsign)$",
    ])?;
    if output.code != Some(0) && output.code != Some(1) {
        return Err(format!("git config failed: {}", output.error_text()));
    }
    let mut config: MergeConfig = MergeConfig { stat: true, ..MergeConfig::default() };
    for entry in output.stdout.split(is_nul_byte) {
        if entry.is_empty() {
            continue;
        }
        let text: String = String::from_utf8_lossy(entry).into_owned();
        let (key, value) = text.split_once('\n').unwrap_or((text.as_str(), "true"));
        match key {
            "merge.ff" if value.eq_ignore_ascii_case("only") => config.fast_forward_only = true,
            "merge.ff" => config.fast_forward = parse_maybe_bool(value),
            "merge.stat" => config.stat = parse_maybe_bool(value).unwrap_or(true),
            "commit.gpgsign" => config.sign_commits = parse_maybe_bool(value).unwrap_or(false),
            "merge.verifysignatures" | "merge.autostash" => {
                config.deferred |= parse_maybe_bool(value).unwrap_or(true);
            }
            other if other.starts_with("branch.") => config.deferred = true,
            _ => {}
        }
    }
    return Ok(config);
}

/// What:
///  The object IDs on a Git output's lines.
/// Why:
///   `rev-parse` and `merge-base` print one ID per line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// output.stdout.toString().split('\n').filter(Boolean)
/// ```
pub fn lines_of(output: &GitOutput) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let trimmed: &str = line.trim();
        if !trimmed.is_empty() {
            lines.push(String::from(trimmed));
        }
    }
    return lines;
}

/// What:
///  The result of the staged-work check:
/// the paths of everyone else's unsaved work,
/// or a reason to forward.
/// Why:
///   An unmerged path means a conflict is in progress,
/// which native Git reports itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function unsavedPaths(session): Promise<Uint8Array[]>;
/// ```
pub fn unsaved_paths(session: &Session) -> Result<Vec<Vec<u8>>, Bail> {
    let status: GitOutput = session.run(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=no",
    ])?;
    if !status.succeeded() {
        return Err(Bail::from(format!("git status failed: {}", status.error_text())));
    }
    let mut entries = status.stdout.split(is_nul_byte);
    while let Some(entry) = entries.next() {
        if entry.len() >= 2 && (entry[0] == b'U' || entry[1] == b'U' || entry[..2] == *b"AA" || entry[..2] == *b"DD") {
            return Err(Bail::Forward(None));
        }
        // A rename or copy is followed by its original path, which carries no status letters.
        if entry.len() >= 2 && (matches!(entry[0], b'R' | b'C') || matches!(entry[1], b'R' | b'C')) {
            entries.next();
        }
    }
    return Ok(status_paths(status.stdout.as_slice()));
}

/// What:
///  The head a pull's `FETCH_HEAD` lists for merging,
/// with the file's text that names it in the merge message.
/// Why:
///   `git merge FETCH_HEAD` merges the lines marked for merging and words the message from the whole file
///  (`handle_fetch_head`,
/// `builtin/merge.c`);
/// this completes the merge only for exactly one such head and leaves several,
/// or
///  none,
/// to native Git,
/// which words that refusal itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readFetchHead(gitDir: string): { oid: string; text: string }; // throws Bail
/// ```
pub fn read_fetch_head(git_dir: &Path) -> Result<(String, String), Bail> {
    // A missing or unreadable file is native Git's to report.
    let Ok(text) = std::fs::read_to_string(git_dir.join("FETCH_HEAD")) else {
        return Err(Bail::Forward(None));
    };
    let mut heads: Vec<String> = fetch_head_merge_heads(text.as_str());
    if heads.len() != 1 {
        return Err(Bail::Forward(None));
    }
    return Ok((heads.remove(0), text));
}
