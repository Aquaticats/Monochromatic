//! What:
//!  The complete `git clean` option table of Git 2.56.0 and the deletion facts read
//!       from it.
//! Why:
//!  The linked-worktree policy guards a clean that can delete files.
//!  Git's last
//!      `--dry-run`/`--no-dry-run` wins,
//!  in clusters (`-ndX`) and abbreviations (`--dry`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cleanChangesWorktree(parseCleanRegion(['-ndX'])) === false
//! ```

/// The reading every `git clean --dry-run` answer gets,
///  which one function owns because both a deletion and a kept repository are read from one.
use crate::command_clean_nested_repositories::dry_run_lines_from_answer;
use crate::command_options::{
    Arity, DEFAULT_MODE, OptionError, OptionSpec, ParsedOptions, UNREAD, parse_options, row,
};
/// The captured answer of a query Git ran.
use crate::git_metadata::MetadataOutput;
use crate::command_options_query::{
    WrapperFlags, is_enabled, positional_tokens, split_wrapper_flags, value_bytes,
};
/// What:
///  Bring the shared hatch spelling,
///  the tokenizer,
///  its table builder and its
///       questions into this file.
/// Why:
///   This module only declares Git's table and interprets what the tokenizer found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseOptions, row } from './command_options.ts';
/// ```
use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// The fact the reading of a clean's own dry run names when it fails.
/// Why:
///  A refusal means Git deleted nothing,
/// while a reading that never answered could hide a deletion;
/// the two fail apart because one lets the command run and the other stops it.
pub const CLEAN_DELETION_FACT: &str = "what the clean would delete";

/// The words that begin a failed reading of a clean's own dry run.
/// Why:
///  The message is this crate's own,
///  and it names the reading the person can run again;
///  a failed reading is never Git's refusal of the clean,
///  so no caller classifies by these words.
pub const UNREADABLE_CLEAN_DELETION: &str = "Git could not read what the clean would delete:";

/// The words that begin the reading of a clean's own dry run that never answered.
/// Why:
///  A read that ended without an exit status says nothing about what the clean would delete;
///  the two sentences differ so the person can tell which kind of failed reading stopped the command.
pub const CLEAN_READ_NEVER_ANSWERED: &str = "Git never answered what the clean would delete:";

/// `-n`,
///  `--dry-run`.
pub const DRY_RUN: u16 = 1;
/// `-i`,
///  `--interactive`.
pub const INTERACTIVE: u16 = 2;
/// `-x`,
///  which makes the clean delete ignored files as well as untracked ones.
pub const INCLUDE_IGNORED: u16 = 4;
/// `-X`,
///  which makes the clean delete ignored files only.
pub const IGNORED_ONLY: u16 = 8;
/// `-e`,
///  `--exclude`,
///  whose value adds one pattern to the ignore rules the clean applies.
pub const EXCLUDE: u16 = 16;
/// `-f`,
///  `--force`,
///  which a clean that deletes needs unless `clean.requireForce` says otherwise;
///  counted,
///  because the second and third `-f` also remove a nested repository.
pub const FORCE: u16 = 32;
/// `-d`,
///  which makes the clean descend into untracked directories instead of leaving them.
pub const DIRECTORIES: u16 = 64;

/// What:
///  Rows of `options[]` in `cmd_clean` (builtin/clean.c:936-948),
///  in source order.
///       `&[OptionSpec]` is a borrowed table baked into the program;
///  `None` in the long
///       position means the option has only a letter.
/// Why:
///   `-e`/`--exclude` takes a value and is `PARSE_OPT_NONEG`;
///  `-d`,
///  `-x` and `-X` have
///       no long spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const CLEAN_TABLE = [row(UNREAD, 'q', 'quiet', 'none', true), /* ... */] as const;
/// ```
pub const CLEAN_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(DRY_RUN, Some(b'n'), Some("dry-run"), Arity::None, true),
    row(FORCE, Some(b'f'), Some("force"), Arity::None, true),
    row(
        INTERACTIVE,
        Some(b'i'),
        Some("interactive"),
        Arity::None,
        true,
    ),
    row(DIRECTORIES, Some(b'd'), None, Arity::None, true),
    row(EXCLUDE, Some(b'e'), Some("exclude"), Arity::Required, false),
    row(INCLUDE_IGNORED, Some(b'x'), None, Arity::None, true),
    row(IGNORED_ONLY, Some(b'X'), None, Arity::None, true),
];

/// What:
///  Final-state facts of one `git clean` region.
///  `bool` is true or false.
/// Why:
///   The policy decides from the final dry-run state;
///  `interactive` is reported for
///       its diagnostics.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CleanRegion = { dryRunActive: boolean; interactiveActive: boolean; hasEscapeHatch: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanRegion {
    /// `-n`/`--dry-run` is on after every option was applied.
    pub dry_run: bool,
    /// `-i`/`--interactive` is on after every option was applied.
    pub interactive: bool,
    /// The clean deletes ignored files,
    ///  whether it also deletes untracked ones (`-x`) or only ignored ones (`-X`).
    pub deletes_ignored: bool,
    /// The clean deletes ignored files and nothing else (`-X` without `-x`).
    pub ignored_only: bool,
    /// The clean asks for `-x` and `-X` together,
    ///  which Git refuses before it reads anything and therefore deletes nothing.
    ///  The wrapper says so itself instead of reading a dry run's silence as a refusal.
    pub both_ignored_modes: bool,
    /// The clean descends into untracked directories (`-d`).
    pub descends: bool,
    /// How many `-f`/`--force` were written;
    ///  the second and third also remove a nested repository.
    pub forces: usize,
    /// The patterns `-e`/`--exclude` added to the ignore rules this clean applies,
    ///  exactly as their bytes were written.
    pub exclusions: Vec<Vec<u8>>,
    /// Wrapper-only flags in option position;
    ///  `escape` is `--no-enforce-worktree`.
    pub wrapper: WrapperFlags,
}

/// What:
///  Parse the region after `clean`.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A region Git itself would refuse yields the refusal;
///  Git then deletes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCleanRegion(region: string[], wrapperFlags: string[]): CleanRegion; // throws
/// ```
pub fn parse_clean_region(
    region: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<CleanRegion, OptionError> {
    // The command's own escape hatch is flag 0; the caller's flags follow.
    let mut flags: Vec<&[u8]> = Vec::<&[u8]>::with_capacity(wrapper_flags.len() + 1);
    flags.push(WORKTREE_ENFORCEMENT_ESCAPE_HATCH.as_bytes());
    for flag in wrapper_flags {
        // `*flag` reads the `&[u8]` the loop variable points at.
        flags.push(*flag);
    }
    // `?` returns Git's refusal to our caller, or unwraps the tokenized region.
    let parsed: ParsedOptions = parse_options(region, CLEAN_TABLE, DEFAULT_MODE, flags.as_slice())?;
    // `Ok(x)` is the success case of `Result`.
    // `git clean` refuses `-x` together with `-X`; whichever reading it took, both delete ignored files, so the
    // measurement covers both.
    let include_ignored: bool = is_enabled(&parsed, INCLUDE_IGNORED);
    let ignored_only: bool = is_enabled(&parsed, IGNORED_ONLY);
    let mut forces: usize = 0;
    let mut exclusions: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for occurrence in &parsed.occurrences {
        if occurrence.id == FORCE && !occurrence.negated {
            forces += 1;
        }
        // `--exclude` is `PARSE_OPT_NONEG`, so a value is the pattern whichever spelling carried it.
        if occurrence.id == EXCLUDE
            && let Some(value) = occurrence.value
        {
            exclusions.push(value_bytes(region, value).to_vec());
        }
    }
    return Ok(CleanRegion {
        dry_run: is_enabled(&parsed, DRY_RUN),
        interactive: is_enabled(&parsed, INTERACTIVE),
        deletes_ignored: include_ignored || ignored_only,
        ignored_only: ignored_only && !include_ignored,
        both_ignored_modes: include_ignored && ignored_only,
        descends: is_enabled(&parsed, DIRECTORIES),
        forces,
        exclusions,
        wrapper: split_wrapper_flags(parsed.wrapper.as_slice()),
    });
}

/// What:
///  The `git clean` arguments that make Git print exactly what this clean would delete,
///  or nothing
///  when Git would refuse the region.
///  `&[OsString]` borrows the invocation's global options and the clean's own tokens.
/// Why:
///  `-e`/`--exclude` adds its patterns to the ignore rules the clean applies,
/// and the worktree status
///  cannot see them,
///  so the only reading that is exactly what Git deletes is Git's own dry run of the
///  same command.
///  The region is rebuilt from the parsed facts rather than edited,
///  because `-i`
///  would make the dry run read the terminal and `-q` would silence it;
///  `-f` is written as often as
///  the caller wrote it (at least once),
///  because the second and third also remove a nested repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanDryRunArguments(globalPrefix: string[], region: string[]): string[] | undefined;
/// ```
pub fn clean_dry_run_arguments(global_prefix: &[OsString], region: &[OsString]) -> Option<Vec<OsString>> {
    return dry_run_arguments(global_prefix, region, true);
}

/// What:
///  The same dry run without the patterns the clean's own `-e`/`--exclude` tokens add to its ignore rules.
///  `&[OsString]` borrows the invocation's global options and the clean's own tokens.
/// Why:
///  A pattern can pull into a clean a path the worktree itself does not ignore,
///  and the difference between
///  this answer and the typed clean's is exactly what the patterns add
///  (`paths_only_the_exclusions_add`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanDryRunArgumentsWithoutExclusions(globalPrefix: string[], region: string[]): string[] | undefined;
/// ```
pub fn clean_dry_run_arguments_without_exclusions(global_prefix: &[OsString], region: &[OsString]) -> Option<Vec<OsString>> {
    return dry_run_arguments(global_prefix, region, false);
}

/// What:
///  The dry-run arguments of one clean,
/// with or without its own exclusion patterns.
///  `bool` says whether the patterns are written.
/// Why:
///  Every reading is the same command,
///  so they share one builder and differ in one step each;
///  a second copy of the
///  rebuild would drift from this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dryRunArguments(globalPrefix: string[], region: string[], withExclusions: boolean): string[] | undefined;
/// ```
fn dry_run_arguments(
    global_prefix: &[OsString],
    region: &[OsString],
    with_exclusions: bool,
) -> Option<Vec<OsString>> {
    // The region Git refuses is nothing to ask about; the `?` is that refusal.
    let parsed: ParsedOptions = parse_options(region, CLEAN_TABLE, DEFAULT_MODE, &[]).ok()?;
    let include_ignored: bool = is_enabled(&parsed, INCLUDE_IGNORED);
    let ignored_only: bool = is_enabled(&parsed, IGNORED_ONLY);
    let descends: bool = is_enabled(&parsed, DIRECTORIES);
    let mut forces: usize = 0;
    let mut exclusions: Vec<&[u8]> = Vec::<&[u8]>::new();
    for occurrence in &parsed.occurrences {
        if occurrence.id == FORCE && !occurrence.negated {
            forces += 1;
        }
        if with_exclusions
            && occurrence.id == EXCLUDE
            && let Some(value) = occurrence.value
        {
            exclusions.push(value_bytes(region, value));
        }
    }
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for token in ["--no-optional-locks", "clean", "-n"] {
        arguments.push(OsString::from(token));
    }
    for _ in 0..forces.max(1) {
        arguments.push(OsString::from("-f"));
    }
    if descends {
        arguments.push(OsString::from("-d"));
    }
    // Every ignored-mode token the person wrote is written too, in Git's own order: `git clean` refuses `-x` with `-X`
    //  inside its own process (`cmd_clean`), and a rebuilt reading that dropped one of them succeeded where the command
    //  the person typed cannot, so the wrapper moved a repository for a clean Git never ran.
    if include_ignored {
        arguments.push(OsString::from("-x"));
    }
    if ignored_only {
        arguments.push(OsString::from("-X"));
    }
    for pattern in exclusions {
        arguments.push(OsString::from("-e"));
        arguments.push(bytes_argument(pattern));
    }
    arguments.push(OsString::from("--"));
    for index in positional_tokens(&parsed, region.len()) {
        arguments.push(region[index].clone());
    }
    return Some(arguments);
}

/// What:
///  The paths a `git clean --dry-run` answer says it would delete,
/// each exactly as Git printed it.
///  `Result<Vec<Vec<u8>>, String>` is "the paths,
/// or why Git could not give one".
/// Why:
///   A dry run prints one line per path it would remove and nothing at all when it would remove nothing,
/// and a clean Git
///  refuses says nothing either,
///  so a failure is an error and never "nothing to delete".
///  Two dry runs of the same clean name a path the same way,
///  which is what makes
///       their two answers comparable line by line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanDeletedPathsFromAnswer(answer: MetadataOutput): Buffer[]; // throws
/// ```
pub fn clean_deleted_paths_from_answer(answer: &MetadataOutput) -> Result<Vec<Vec<u8>>, String> {
    return dry_run_lines_from_answer(answer, CLEAN_DELETION_FACT);
}

/// What:
///  The paths one dry-run answer names that the same clean without its exclusion patterns would not delete.
///  `deleted` and `otherwise_deleted` borrow both answers as Git printed them.
/// Why:
///   Git adds the `-e`/`--exclude` patterns to the ignore rules the clean applies,
///  so a path only the first
///  answer names is one those patterns call ignored and the worktree does not:
///  an untracked
///  file this clean would lose.
///  A set keeps the comparison linear in the larger answer,
///  because a
///  worktree can ignore millions of paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathsOnlyTheExclusionsAdd(deleted: Buffer[], otherwiseDeleted: Buffer[]): Buffer[];
/// ```
pub fn paths_only_the_exclusions_add(deleted: &[Vec<u8>], otherwise_deleted: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let known: std::collections::BTreeSet<&[u8]> = otherwise_deleted.iter().map(Vec::as_slice).collect();
    let mut added: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in deleted {
        if !known.contains(path.as_slice()) {
            added.push(path.clone());
        }
    }
    return added;
}

/// What:
///  One pattern's exact bytes,
/// as the system reads them.
/// Why:
///   A pattern is bytes and not text,
/// so a pattern that is not UTF-8 reaches Git unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const argument = Buffer.from(pattern);
/// ```
fn bytes_argument(pattern: &[u8]) -> OsString {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        return OsString::from_vec(pattern.to_vec());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return OsString::from(String::from_utf8_lossy(pattern).into_owned());
    }
}

/// What:
///  Whether the clean can delete files.
///  `&CleanRegion` borrows the facts.
/// Why:
///   Every `unlink` and `rmdir` in Git's clean is skipped under dry run,
///  also after
///       the interactive menu chose what to delete (builtin/clean.c:191,
///  223,
///  267,
///       1050-1078).
/// Gotcha:
///  Divergence from the incumbent,
///  which treated every interactive clean as
///         deleting:
///  `git clean -i -n` deletes nothing in Git 2.56.0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanChangesWorktree(region: CleanRegion): boolean;
/// ```
pub fn clean_changes_worktree(region: &CleanRegion) -> bool {
    return !region.dry_run;
}

/// Facts in every accepted spelling,
///  and real-Git controls of the table and of dry runs.
#[cfg(test)]
#[path = "command_clean_tests.rs"]
mod tests;
