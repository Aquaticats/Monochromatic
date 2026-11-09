//! What:
//!  The grammar of `git monochromatic-git-policy-cli migrate-to-subvolume`,
//! the wrapper's command that converts the main checkout into a
//!  btrfs subvolume,
//! and the help text.
//! Why:
//!  The command name says what it does and follows the other multi-word `git monochromatic-git-policy-cli` spellings (`service status`).
//!  Running it migrates after the preflight;
//! `--dry-run` previews;
//! `--rollback` and `--remove-old` are the two ways
//!  out;
//! the rest only switch off a part of the migration.
//! Modes that exclude each other,
//! and switches that belong to
//!  one mode,
//! are refused with the reason.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! parseMigrateArguments(['--dry-run']) // => { mode: 'migrate', dryRun: true, nestedWorktrees: true, refreshIndexes: true }
//! ```

use std::ffi::OsString;

/// The usage and the meaning of each option,
/// printed for `--help` and after a refused invocation.
pub const MIGRATE_HELP: &str = "\
Usage: git monochromatic-git-policy-cli migrate-to-subvolume [--dry-run] [--no-nested-worktrees] [--no-refresh-indexes]
       git monochromatic-git-policy-cli migrate-to-subvolume --rollback [--discard-unreadable] [--discard-left-out] [--dry-run]
       git monochromatic-git-policy-cli migrate-to-subvolume --remove-old [--discard-unreadable] [--discard-left-out] [--dry-run]

Convert the repository's main checkout into a btrfs subvolume at the same path, so that creating a worktree can start
from a snapshot. Nothing is spawned: the command calls the btrfs ioctls, clones file data, copies the tree, swaps the
old and new directories atomically in one step, and rewrites the Git index stat fields so the first `git status` stays
fast. Every step is recorded beside the checkout; if the command is stopped at any point, run it again and it resumes.
The original tree is kept until you delete it.

Options:
  --dry-run               Check the conditions and print the plan; change nothing.
  --rollback              Undo a migration, finished or not: carry back newer files, restore the original directory
                          and delete the new subvolume.
  --remove-old            Delete the original tree of a finished migration (slow: every file is unlinked).
  --discard-unreadable    With --remove-old: also delete the files this user may not read, which were not copied.
                          With --rollback: also delete the files only the new subvolume holds that this user may not
                          read, which cannot be carried back.
  --discard-left-out      With --remove-old: also delete the entries of the old tree that the checkout lacks and that
                          are older than the migration's catch-up. --remove-old names them and deletes nothing without
                          this flag, because they are either files you deleted from the checkout on purpose or files
                          you restored into the old tree (the list is written beside the checkout).
                          With --rollback: also delete the entries of the new subvolume that could not be carried back
                          because the original tree holds another kind of entry at their path. --rollback names them
                          and deletes nothing without this flag, because the new subvolume is the only copy of them.
  --no-nested-worktrees   Keep the linked worktrees inside the checkout as ordinary directories (default: each becomes
                          a nested subvolume).
  --no-refresh-indexes    Leave the Git index files as copied (the first git status of each worktree then hashes every
                          file).
  --help                  Show this help.

Before you run it, stop agent sessions and editors that have their working directory inside the checkout: after the
swap they would keep writing into the old tree. The dry run lists them. Run it on a machine with the checkout's
filesystem mounted with enough free space; `git monochromatic-git-policy-cli doctor` explains the mount option that makes removal fast.

Risk, accepted by running the command: it replaces or deletes files of the checkout while other programs may be
changing the same files. A program that writes through an open file or a mapping, or that renames or replaces a path,
between a check and the step that relies on it, can lose that change, and nothing undoes the deletion of a tree. Stop
the programs working inside the checkout first. The command says so once on standard error before it changes anything,
and in this help.
";

/// The one line printed on standard error before a run changes the checkout.
pub const MIGRATE_RISK_NOTICE: &str = "monochromatic-git-policy-cli: this command replaces or deletes files of the checkout while other programs may be changing them at the same moment. A change made between a check and the step that relies on it is lost, and that risk is yours; stop the programs working inside the checkout first.";

/// What:
///  Which of the three things the command does.
/// Why:
///   They exclude each other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MigrateMode = 'migrate' | 'rollback' | 'remove-old';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MigrateMode {
    /// Convert the checkout.
    Migrate,
    /// Undo a conversion.
    Rollback,
    /// Delete the original tree.
    RemoveOld,
}

/// What:
///  A parsed invocation.
/// Why:
///   The command acts on a typed value;
/// nothing downstream re-reads raw arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MigrateArguments = { mode: MigrateMode; dryRun: boolean; nestedWorktrees: boolean; refreshIndexes: boolean; discardUnreadable: boolean; help: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MigrateArguments {
    /// What to do.
    pub mode: MigrateMode,
    /// Preview only.
    pub dry_run: bool,
    /// Make linked worktrees nested subvolumes.
    pub nested_worktrees: bool,
    /// Rewrite the index stat fields.
    pub refresh_indexes: bool,
    /// With remove-old or rollback:
    /// delete the unreadable files too.
    pub discard_unreadable: bool,
    /// With remove-old or rollback:
    /// delete the entries the late-write carry left out of the checkout too,
    /// or those the rollback's carry-back could not put into the original tree.
    pub discard_left_out: bool,
    /// Print the help.
    pub help: bool,
}

/// What:
///  Parse the arguments after `migrate-to-subvolume`.
///  `Result<MigrateArguments, String>` is the parse,
/// or the reason the invocation is refused.
/// Why:
///   Each refusal names the option and says what to use instead,
/// so the person is never left to guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMigrateArguments(args: string[]): MigrateArguments;
/// ```
pub fn parse_migrate_arguments(arguments: &[OsString]) -> Result<MigrateArguments, String> {
    let mut parsed: MigrateArguments = MigrateArguments {
        mode: MigrateMode::Migrate,
        dry_run: false,
        nested_worktrees: true,
        refresh_indexes: true,
        discard_unreadable: false,
        discard_left_out: false,
        help: false,
    };
    let (mut rollback, mut remove_old, mut migrate_switch): (bool, bool, bool) = (false, false, false);
    for token in arguments {
        let Some(text) = token.to_str() else {
            return Err(String::from("an argument is not valid UTF-8 text; the options are ASCII"));
        };
        match text {
            "--dry-run" => parsed.dry_run = true,
            "--rollback" => rollback = true,
            "--remove-old" => remove_old = true,
            "--discard-unreadable" => parsed.discard_unreadable = true,
            "--discard-left-out" => parsed.discard_left_out = true,
            "--no-nested-worktrees" => {
                parsed.nested_worktrees = false;
                migrate_switch = true;
            }
            "--no-refresh-indexes" => {
                parsed.refresh_indexes = false;
                migrate_switch = true;
            }
            "--help" | "-h" => parsed.help = true,
            other => return Err(format!("unknown argument {other}; the options are listed by `git monochromatic-git-policy-cli migrate-to-subvolume --help`")),
        }
    }
    if parsed.help {
        return Ok(parsed);
    }
    if rollback && remove_old {
        return Err(String::from("--rollback and --remove-old exclude each other: --rollback restores the original tree, --remove-old deletes it. Run one, then the other if you still want it."));
    }
    if (rollback || remove_old) && migrate_switch {
        return Err(String::from("--no-nested-worktrees and --no-refresh-indexes change how the migration runs, so they apply only without --rollback and --remove-old."));
    }
    if parsed.discard_unreadable && !remove_old && !rollback {
        return Err(String::from("--discard-unreadable applies only with --remove-old or --rollback."));
    }
    if parsed.discard_left_out && !remove_old && !rollback {
        return Err(String::from("--discard-left-out applies only with --remove-old or --rollback."));
    }
    if rollback {
        parsed.mode = MigrateMode::Rollback;
    }
    if remove_old {
        parsed.mode = MigrateMode::RemoveOld;
    }
    return Ok(parsed);
}

/// Argument controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_arguments_tests.rs"]
mod tests;
