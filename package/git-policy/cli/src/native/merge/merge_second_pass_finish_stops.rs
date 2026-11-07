//! What:
//!  Which options of a merge,
//! a pull,
//! a cherry-pick or a revert make the finish stop where Git would stop,
//! and the operation they belong to.
//! Why:
//!  `merge_second_pass_finish.rs` plans and runs the finish and stays under the file-size limit;
//! reading the options needs nothing from it,
//! so it lives in a file of its own that the finish imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stops = mergeStops(["--no-commit", "feature"]); // { noCommit: true, ... }
//! ```

use std::ffi::OsString;
use std::path::Path;

/// What stops a clean run before it commits,
/// and what shapes its message.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Stops {
    /// `--no-commit` (or `-n` of a pick):
    /// stop with the result staged.
    pub no_commit: bool,
    /// `--squash`:
    /// stop with the result staged and no merge state.
    pub squash: bool,
    /// `--edit` or `--no-edit`;
    /// absent when neither was given.
    pub edit: Option<bool>,
    /// `-m` or `-F` was given,
    /// which turns the default editor off.
    pub message: bool,
    /// Signing and sign-off options,
    /// handed on to the commit as typed.
    pub passed_on: Vec<OsString>,
}

/// Which operation stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// A merge,
    /// with `MERGE_HEAD` (or `--squash`).
    Merge,
    /// A rebase,
    /// either backend.
    Rebase,
    /// A cherry-pick or revert.
    Pick,
}

/// The options shared by merge and pull that change nothing the finish needs to know.
pub const HARMLESS_MERGE_OPTIONS: [&str; 21] = [
    "-v", "--verbose", "--no-verbose", "--progress", "--no-progress", "--no-verify", "--verify", "--summary", "--no-summary",
    "--no-signoff", "--no-verify-signatures", "--ff", "--no-ff", "--ff-only", "--allow-unrelated-histories", "--stat",
    "--no-stat", "--log", "--no-log", "--quiet", "--autostash",
];

/// The options of pull that only fetch,
/// or choose merge or rebase,
/// which the state files decide.
pub const HARMLESS_PULL_OPTIONS: [&str; 17] = [
    "--rebase", "-r", "--no-rebase", "--rebase=true", "--rebase=false", "--rebase=merges", "--all", "--prune", "-p", "--no-prune",
    "--tags", "-t", "--no-tags", "--force", "-f", "--no-recurse-submodules", "--ipv4",
];

/// The options of a cherry-pick or revert that change nothing the finish needs to know.
pub const HARMLESS_PICK_OPTIONS: [&str; 13] = [
    "-x", "--ff", "--allow-empty", "--allow-empty-message", "--keep-redundant-commits", "--rerere-autoupdate",
    "--no-rerere-autoupdate", "--reference", "-q", "--quiet", "-v", "--verbose", "--no-verbose",
];

/// The options that take the next token as their value and change nothing the finish needs to know.
pub const VALUE_OPTIONS: [&str; 4] = ["-s", "--strategy", "-X", "--strategy-option"];

/// What:
///  Read the arguments after the subcommand of a merge or pull,
/// by exact spelling.
/// Why:
///   An option this does not know might be an abbreviation of `--no-commit`;
/// the finish must not guess,
/// so it declines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeStops(region: string[], pull: boolean): Stops | undefined;
/// ```
pub fn merge_stops(region: &[OsString], pull: bool) -> Option<Stops> {
    let mut stops: Stops = Stops::default();
    let mut tokens = region.iter();
    while let Some(token) = tokens.next() {
        let text: &str = token.to_str()?;
        match text {
            "--no-commit" => stops.no_commit = true,
            "--commit" => stops.no_commit = false,
            "--squash" => stops.squash = true,
            "--no-squash" => stops.squash = false,
            "-e" | "--edit" => stops.edit = Some(true),
            "--no-edit" => stops.edit = Some(false),
            "-m" | "--message" | "-F" | "--file" => {
                tokens.next()?;
                stops.message = true;
            }
            "--signoff" | "--no-gpg-sign" => stops.passed_on.push(token.clone()),
            other if other.starts_with("--message=") || other.starts_with("--file=") => stops.message = true,
            other if other.starts_with("-S") || other.starts_with("--gpg-sign") => stops.passed_on.push(token.clone()),
            other if VALUE_OPTIONS.contains(&other) => {
                tokens.next()?;
            }
            other if other.starts_with("--strategy=") || other.starts_with("--strategy-option=") => {}
            other if HARMLESS_MERGE_OPTIONS.contains(&other) => {}
            other if pull && HARMLESS_PULL_OPTIONS.contains(&other) => {}
            other if other.starts_with('-') && other != "-" => return None,
            _ => {}
        }
    }
    return Some(stops);
}

/// What:
///  Read the arguments after the subcommand of a cherry-pick or revert,
/// by exact spelling,
/// and say whether they continue a
///  sequence (`--continue`,
/// `--skip`),
/// whose options Git saved.
/// Why:
///   Same reason as `merge_stops`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pickStops(region: string[]): { stops: Stops; continuing: boolean } | undefined;
/// ```
pub fn pick_stops(region: &[OsString]) -> Option<(Stops, bool)> {
    let mut stops: Stops = Stops::default();
    let mut continuing: bool = false;
    let mut tokens = region.iter();
    while let Some(token) = tokens.next() {
        let text: &str = token.to_str()?;
        match text {
            "-n" | "--no-commit" => stops.no_commit = true,
            "-e" | "--edit" => stops.edit = Some(true),
            "--no-edit" => stops.edit = Some(false),
            "--continue" | "--skip" => continuing = true,
            "-s" | "--signoff" => stops.passed_on.push(OsString::from("--signoff")),
            "-m" | "--mainline" => {
                tokens.next()?;
            }
            other if other.starts_with("--mainline=") || other.starts_with("--empty=") => {}
            other if other.starts_with("-S") || other.starts_with("--gpg-sign") || other == "--no-gpg-sign" => stops.passed_on.push(token.clone()),
            other if VALUE_OPTIONS.contains(&other) => {
                tokens.next()?;
            }
            other if other.starts_with("--strategy=") || other.starts_with("--strategy-option=") => {}
            other if HARMLESS_PICK_OPTIONS.contains(&other) => {}
            other if other.starts_with('-') && other != "-" => return None,
            _ => {}
        }
    }
    return Some((stops, continuing));
}

/// What:
///  The options Git saved for a cherry-pick or revert sequence,
/// read from `sequencer/opts`.
/// Why:
///   `--continue` takes the options of the first command (`save_opts`,
/// `sequencer.c` lines 3734 to 3760);
/// the lines of the
///  file are `name = value` under `[options]`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function savedPickStops(gitDir: string): Stops | undefined;
/// ```
pub fn saved_pick_stops(git_dir: &Path) -> Option<Stops> {
    let text: String = std::fs::read_to_string(git_dir.join("sequencer/opts")).ok()?;
    let mut stops: Stops = Stops::default();
    for line in text.lines() {
        let Some((name, value)) = line.split_once('=') else { continue };
        match (name.trim(), value.trim()) {
            ("no-commit", "true") => stops.no_commit = true,
            ("edit", "true") => stops.edit = Some(true),
            ("edit", "false") => stops.edit = Some(false),
            ("signoff", "true") => stops.passed_on.push(OsString::from("--signoff")),
            _ => {}
        }
    }
    return Some(stops);
}
