//! What:
//!  How the word-level second pass starts Git:
//! what a start gave back,
//! the thing that runs a start,
//! and the one that runs it in the invocation's context.
//! Why:
//!  `merge_second_pass.rs` runs the pass and stays under the file-size limit;
//! the way Git is started needs nothing from it,
//! so it lives in a file of its own that the pass imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reply = await runner.run(["merge-tree", "--write-tree"], undefined, undefined); // { code, stdout, stderr }
//! ```

use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What a Git start gave back.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitReply {
    /// The exit status,
    /// or nothing when a signal ended Git.
    pub code: Option<i32>,
    /// Standard output.
    pub stdout: Vec<u8>,
    /// Standard error.
    pub stderr: Vec<u8>,
}

/// How a caller starts Git for the second pass:
/// in the repository it works in,
/// with its environment and global options.
pub trait GitRunner {
    /// Start Git with `arguments`,
    /// `input` on standard input and `index_file` as the index when given.
    /// `Err` means Git could
    /// not be started;
    /// an exit status other than 0 is a reply.
    fn run(&self, arguments: &[&str], input: Option<&[u8]>, index_file: Option<&Path>) -> Result<GitReply, String>;
}

/// The runner for callers that start Git through `run_git`:
/// one context,
/// one directory and a fixed set of request options.
pub struct ContextRunner<'a> {
    /// The real Git,
    /// its overlay and the global options of the invocation.
    pub context: &'a GitContext,
    /// The directory every start runs in.
    pub cwd: &'a Path,
    /// Whether the invocation's global options are left out (the commit replay's private runs).
    pub without_prefix: bool,
    /// The object store that receives written objects,
    /// when it is not the repository's own.
    pub object_directory: Option<&'a Path>,
    /// Variables added to every start.
    pub environment: Vec<(OsString, OsString)>,
}

/// Starting Git through the shared runner.
impl GitRunner for ContextRunner<'_> {
    fn run(&self, arguments: &[&str], input: Option<&[u8]>, index_file: Option<&Path>) -> Result<GitReply, String> {
        let mut request: GitRequest = GitRequest::new(self.cwd, arguments);
        request.without_prefix = self.without_prefix;
        request.object_directory = self.object_directory.map(Path::to_path_buf);
        request.environment.clone_from(&self.environment);
        request.input = input.map(<[u8]>::to_vec);
        request.index = index_file.map(PathBuf::from);
        let output = match run_git(self.context, &request) {
            Ok(done) => done,
            Err(error) => return Err(format!("git {} could not start: {error}", arguments.join(" "))),
        };
        return Ok(GitReply { code: output.code, stdout: output.stdout, stderr: output.stderr });
    }
}

/// What:
///  The configuration options among an invocation's global options,
/// and nothing else:
/// `-c <name>=<value>`,
/// `--config-env=<name>=<variable>` or `--config-env <name>=<variable>`,
/// and `--attr-source=<tree>` or `--attr-source <tree>`.
/// Why:
///   The second pass runs from the top of the worktree,
/// because `git check-attr` and `git checkout-index` read the paths
///  they are given relative to the directory they run in;
/// `-C` and `--git-dir` would move them again.
/// The settings that change
///  what a checkout writes (`core.autocrlf`,
/// filters) still apply,
/// and so does every way the invocation names an attribute tree:
/// leaving one
///  out made the pass read attributes from another source than the person chose,
/// and merge a file their rules
///  exclude.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configOptions(prefix: string[]): string[]; // ['-c', 'a=b', '--config-env', 'attr.tree=RULES']
/// ```
pub fn config_options(prefix: &[OsString]) -> Vec<OsString> {
    let mut kept: Vec<OsString> = Vec::new();
    let mut tokens = prefix.iter();
    while let Some(token) = tokens.next() {
        // Two of these options carry their value in the next token, and the other two after `=` in the same one.
        if token == "-c" || token == "--config-env" || token == "--attr-source" {
            let Some(setting) = tokens.next() else { continue };
            kept.push(token.clone());
            kept.push(setting.clone());
            continue;
        }
        let Some(text) = token.to_str() else { continue };
        if text.starts_with("--config-env=") || text.starts_with("--attr-source=") {
            kept.push(token.clone());
        }
    }
    return kept;
}
