//! Controls for the grammar of `git monochromatic-git-policy-cli migrate-to-subvolume`:
//! the defaults,
//! every option,
//! every refusal and its
//!  wording,
//! and the help.

use super::*;
use std::os::unix::ffi::OsStringExt;

/// Parse text arguments.
fn parse(values: &[&str]) -> Result<MigrateArguments, String> {
    let tokens: Vec<OsString> = values.iter().map(OsString::from).collect();
    return parse_migrate_arguments(tokens.as_slice());
}

/// The parse of no argument:
/// a migration with both parts on.
const DEFAULTS: MigrateArguments = MigrateArguments {
    mode: MigrateMode::Migrate,
    dry_run: false,
    nested_worktrees: true,
    refresh_indexes: true,
    discard_unreadable: false,
    discard_left_out: false,
    help: false,
};

/// No argument migrates with the nested worktrees and the index rewrite on.
#[test]
fn no_argument_migrates_with_everything_on() {
    assert_eq!(parse(&[]), Ok(DEFAULTS));
}

/// Each option sets exactly its own field.
#[test]
fn each_option_sets_its_own_field() {
    assert_eq!(parse(&["--dry-run"]), Ok(MigrateArguments { dry_run: true, ..DEFAULTS }));
    assert_eq!(parse(&["--no-nested-worktrees"]), Ok(MigrateArguments { nested_worktrees: false, ..DEFAULTS }));
    assert_eq!(parse(&["--no-refresh-indexes"]), Ok(MigrateArguments { refresh_indexes: false, ..DEFAULTS }));
    assert_eq!(parse(&["--rollback"]), Ok(MigrateArguments { mode: MigrateMode::Rollback, ..DEFAULTS }));
    assert_eq!(parse(&["--remove-old"]), Ok(MigrateArguments { mode: MigrateMode::RemoveOld, ..DEFAULTS }));
    assert_eq!(
        parse(&["--remove-old", "--discard-unreadable", "--dry-run"]),
        Ok(MigrateArguments { mode: MigrateMode::RemoveOld, discard_unreadable: true, dry_run: true, ..DEFAULTS })
    );
    assert_eq!(parse(&["--rollback", "--dry-run"]), Ok(MigrateArguments { mode: MigrateMode::Rollback, dry_run: true, ..DEFAULTS }));
    assert_eq!(
        parse(&["--remove-old", "--discard-left-out"]),
        Ok(MigrateArguments { mode: MigrateMode::RemoveOld, discard_left_out: true, ..DEFAULTS })
    );
}

/// Both spellings of help win over every other mistake in the same invocation.
#[test]
fn help_wins_over_other_options() {
    assert_eq!(parse(&["--help"]), Ok(MigrateArguments { help: true, ..DEFAULTS }));
    assert_eq!(parse(&["-h"]), Ok(MigrateArguments { help: true, ..DEFAULTS }));
    assert!(parse(&["--rollback", "--remove-old", "--help"]).expect("help").help);
}

/// An unknown word is refused,
/// named,
/// and pointed at the help.
#[test]
fn an_unknown_argument_is_named() {
    let reason: String = parse(&["--force"]).expect_err("refused");
    assert!(reason.contains("unknown argument --force") && reason.contains("--help"), "{reason}");
}

/// Text that is not UTF-8 is refused with the reason,
/// not lossily guessed.
#[test]
fn an_argument_that_is_not_utf8_is_refused() {
    let bad: OsString = OsString::from_vec(vec![0x2d, 0x2d, 0xff]);
    let reason: String = parse_migrate_arguments(&[bad]).expect_err("refused");
    assert!(reason.contains("not valid UTF-8"), "{reason}");
}

/// The two modes exclude each other,
/// and the reason says what each does.
#[test]
fn rollback_and_remove_old_exclude_each_other() {
    let reason: String = parse(&["--rollback", "--remove-old"]).expect_err("refused");
    assert!(reason.contains("exclude each other") && reason.contains("restores") && reason.contains("deletes"), "{reason}");
}

/// The migration switches belong to the migration alone,
/// in either order and with either mode.
#[test]
fn migration_switches_are_refused_with_the_other_modes() {
    for arguments in [
        vec!["--rollback", "--no-nested-worktrees"],
        vec!["--no-refresh-indexes", "--rollback"],
        vec!["--remove-old", "--no-nested-worktrees"],
        vec!["--no-refresh-indexes", "--remove-old"],
    ] {
        let reason: String = parse(arguments.as_slice()).expect_err("refused");
        assert!(reason.contains("apply only without --rollback and --remove-old"), "{arguments:?}: {reason}");
    }
}

/// Discarding the unreadable files has meaning only with the deletion of the old tree or the rollback.
#[test]
fn discard_unreadable_needs_remove_old_or_rollback() {
    let reason: String = parse(&["--discard-unreadable"]).expect_err("refused");
    assert!(reason.contains("applies only with --remove-old or --rollback"), "{reason}");
    assert_eq!(
        parse(&["--rollback", "--discard-unreadable"]),
        Ok(MigrateArguments { mode: MigrateMode::Rollback, discard_unreadable: true, ..DEFAULTS })
    );
}

/// Discarding what the carry left out has meaning only with the deletion of the old tree or the rollback.
#[test]
fn discard_left_out_needs_remove_old_or_rollback() {
    let reason: String = parse(&["--discard-left-out"]).expect_err("refused");
    assert!(reason.contains("applies only with --remove-old or --rollback"), "{reason}");
    assert_eq!(
        parse(&["--remove-old", "--discard-left-out"]),
        Ok(MigrateArguments { mode: MigrateMode::RemoveOld, discard_left_out: true, ..DEFAULTS })
    );
    assert_eq!(
        parse(&["--rollback", "--discard-left-out"]),
        Ok(MigrateArguments { mode: MigrateMode::Rollback, discard_left_out: true, ..DEFAULTS })
    );
}

/// The help names every option the grammar accepts,
/// so the two cannot drift apart.
#[test]
fn the_help_names_every_option() {
    for option in ["--dry-run", "--rollback", "--remove-old", "--discard-unreadable", "--discard-left-out", "--no-nested-worktrees", "--no-refresh-indexes", "--help"] {
        assert!(MIGRATE_HELP.contains(option), "{option} missing from the help");
    }
}

/// The help states the risk the person accepts by running the command,
/// and the notice says the same thing in one line.
#[test]
fn the_help_and_the_notice_state_the_risk() {
    assert!(MIGRATE_HELP.contains("Risk, accepted by running the command"));
    assert!(MIGRATE_HELP.contains("other programs may be"));
    assert!(MIGRATE_HELP.contains("changing the same files"));
    assert!(MIGRATE_HELP.contains("says so once on standard error"));
    assert!(!MIGRATE_RISK_NOTICE.contains('\n'), "the notice is one line");
    assert!(MIGRATE_RISK_NOTICE.contains("that risk is yours"));
}
