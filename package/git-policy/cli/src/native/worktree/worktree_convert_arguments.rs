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
pub const CONVERT_HELP: &str = "git cli-git convert-worktree-to-subvolume [--dry-run | --abandon] [--jobs <count>] (--all | <worktree>...)

Convert linked worktrees that are ordinary directories into btrfs subvolumes at the same path, side by side.
A snapshot of the main checkout then shows each converted worktree as an empty directory instead of a full copy.

For each worktree the command refuses unless it is quiet: not locked, no index.lock, no operation in progress, tracked files
unmodified, no process with its working directory or an open file inside it, and no worktree registered inside it.
It then copies the worktree with cloned data into <worktree>.cli-git-converting, proves the copy equal, swaps the two with
one atomic exchange, rewrites the index stat fields so the first `git status` stays fast, and removes the old tree only
after the swap is verified. The registration and the bytes of the .git file do not change.

  --all        every linked worktree registered inside the main checkout
  --dry-run    list each worktree and what would happen, or why it would be skipped; change nothing
  --abandon    undo an unfinished conversion that has not reached its swap (destroys only the subvolume it created)
  --jobs <n>   convert at most n worktrees at once (also --jobs=<n>); without it the command uses the threads this process
               may run, less a margin, and no more worktrees than the free memory holds

The worktrees are named in the order given (with --all, sorted by path) and each one's lines come together, in that order,
whichever finishes first. A worktree that fails does not stop the others: it keeps its journal, the others are converted, the
summary counts the failures, and the status is 2. Run the same command again to finish the failed ones.

An interrupted run is finished by running the same command again.

Risk, accepted by running the command: it deletes the old directory once the copy is verified, and it cannot stop
another process from changing the worktree while it works. A process that writes through an open file or a mapping, or
renames or replaces a path, between a check and the step that relies on it can lose its data; stop such processes first.
";

/// The one line printed on standard error when a conversion runs, before anything is removed.
pub const RISK_NOTICE: &str = "cli-git: this deletes the old directory of each worktree after the copy is verified. A process that changes a worktree while it is converted can lose data, and that risk is yours; stop such processes first.";

/// What:
///  The parsed options.
/// Why:
///   The parse is separate from the run so the grammar is controlled without a filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvertArguments = { dryRun: boolean; abandon: boolean; all: boolean; paths: string[]; help: boolean; commandLineAttributes: boolean; jobs?: number };
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
    /// The invocation chose an attribute source before the command (`--attr-source`, `-c attr.tree`).
    pub command_line_attributes: bool,
    /// The most worktrees converted at once (`--jobs`), or nothing for the budget the host allows.
    pub jobs: Option<usize>,
}

/// The count after `--jobs`: a whole number of at least one.
fn parse_jobs(value: &str) -> Result<usize, String> {
    return match value.parse::<usize>() {
        Ok(count) if count >= 1 => Ok(count),
        _ => Err(format!("--jobs needs a whole number of at least one, but got {value:?}")),
    };
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
    let mut tokens = rest.iter();
    while let Some(token) = tokens.next() {
        let Some(word) = token.to_str() else {
            parsed.paths.push(token.clone());
            continue;
        };
        match word {
            "--jobs" => {
                // The word after `--jobs`, as text; a missing word and a word that is not text are the same refusal.
                let next_text: Option<&str> = match tokens.next() {
                    Some(next) => next.to_str(),
                    None => None,
                };
                let Some(value) = next_text else {
                    return Err(String::from("--jobs needs a count, as in --jobs 4"));
                };
                parsed.jobs = Some(parse_jobs(value)?);
            }
            other if other.starts_with("--jobs=") => parsed.jobs = Some(parse_jobs(&other["--jobs=".len()..])?),
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

/// What:
///  Whether the global options before `cli-git` choose an attribute source: `--attr-source`, `-c attr.tree=...` or
///  `--config-env=attr.tree=...`.
/// Why:
///   Git would then take its attributes from a tree, which the conversion's content check does not read, so the check refuses
///  to decide by content.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selectsAttributeSource(globals: string[]): boolean;
/// ```
pub fn selects_attribute_source(globals: &[std::ffi::OsString]) -> bool {
    let mut words = globals.iter();
    while let Some(word) = words.next() {
        let text: String = word.to_string_lossy().to_lowercase();
        if text.starts_with("--attr-source") || text.starts_with("--config-env=attr.tree") {
            return true;
        }
        if text == "-c" {
            // The value is read only after a `-c`, as the short-circuit of `&&` did.
            let names_tree: bool = match words.next() {
                Some(value) => value.to_string_lossy().to_lowercase().starts_with("attr.tree"),
                None => false,
            };
            if names_tree {
                return true;
            }
        }
    }
    return false;
}

/// Argument grammar controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_arguments_tests.rs"]
mod tests;
