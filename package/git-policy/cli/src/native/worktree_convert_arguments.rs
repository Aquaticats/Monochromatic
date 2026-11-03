//! What:
//!  The grammar of `git cli-git convert-worktree-to-subvolume`.
//! Why:
//!  The grammar is parsed on every platform by the management namespace; only the run needs Linux and btrfs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const arguments = parseConvertArguments(['--dry-run', '--all']);
//! ```

/// The text of `--help`.
pub const CONVERT_HELP: &str = "git cli-git convert-worktree-to-subvolume [--dry-run | --abandon] (--all | <worktree>...)

Convert linked worktrees that are ordinary directories into btrfs subvolumes at the same path, one at a time.
A snapshot of the main checkout then shows each converted worktree as an empty directory instead of a full copy.

For each worktree the command refuses unless it is quiet: not locked, no index.lock, no operation in progress, tracked files
unmodified, no process with its working directory or an open file inside it, and no worktree registered inside it.
It then copies the worktree with cloned data into <worktree>.cli-git-converting, proves the copy equal, swaps the two with
one atomic exchange, rewrites the index stat fields so the first `git status` stays fast, and removes the old tree only
after the swap is verified. The registration and the bytes of the .git file do not change.

  --all        every linked worktree registered inside the main checkout
  --dry-run    list each worktree and what would happen, or why it would be skipped; change nothing
  --abandon    undo an unfinished conversion that has not reached its swap (destroys only the subvolume it created)

An interrupted run is finished by running the same command again.
";

/// What:
///  The parsed options.
/// Why:
///   The parse is separate from the run so the grammar is controlled without a filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvertArguments = { dryRun: boolean; abandon: boolean; all: boolean; paths: string[]; help: boolean };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConvertArguments {
    /// Preview only.
    pub dry_run: bool,
    /// Undo an unfinished conversion.
    pub abandon: bool,
    /// Every linked worktree inside the main checkout.
    pub all: bool,
    /// Worktrees named on the command line.
    pub paths: Vec<std::ffi::OsString>,
    /// Print the help.
    pub help: bool,
}

/// What:
///  Parse the words after `convert-worktree-to-subvolume`.
/// Why:
///   A refusal names the word it refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseConvertArguments(rest: string[]): ConvertArguments; // throws the reason
/// ```
pub fn parse_convert_arguments(rest: &[std::ffi::OsString]) -> Result<ConvertArguments, String> {
    let mut parsed: ConvertArguments = ConvertArguments::default();
    for token in rest {
        let Some(word) = token.to_str() else {
            parsed.paths.push(token.clone());
            continue;
        };
        match word {
            "--help" | "-h" => parsed.help = true,
            "--dry-run" => parsed.dry_run = true,
            "--abandon" => parsed.abandon = true,
            "--all" => parsed.all = true,
            other if other.starts_with('-') => return Err(format!("unknown option {other}; see `git cli-git convert-worktree-to-subvolume --help`")),
            _ => parsed.paths.push(token.clone()),
        }
    }
    if parsed.dry_run && parsed.abandon {
        return Err(String::from("--dry-run and --abandon cannot be combined"));
    }
    if !parsed.help && !parsed.all && parsed.paths.is_empty() {
        return Err(String::from("name a worktree or pass --all; see `git cli-git convert-worktree-to-subvolume --help`"));
    }
    if parsed.all && !parsed.paths.is_empty() {
        return Err(String::from("--all and named worktrees cannot be combined"));
    }
    return Ok(parsed);
}

/// Argument grammar controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_arguments_tests.rs"]
mod tests;
