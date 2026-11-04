//! The unit tests of the command grammar:
//! pnpm's flag names,
//! and no words pnpm does not have.

use clap::Parser;

use crate::cli::{Cli, Command};

fn parse(words: &[&str]) -> Result<Cli, clap::Error> {
    let mut all: Vec<&str> = vec!["cargo-shadow"];
    all.extend(words);
    return Cli::try_parse_from(all);
}

#[test]
fn install_takes_pnpms_frozen_flags_and_dry_run() {
    match parse(&["install", "--frozen-lockfile", "--dry-run"]).expect("parses").command {
        Command::Install { frozen_lockfile, no_frozen_lockfile, dry_run } => {
            assert!(frozen_lockfile);
            assert!(!no_frozen_lockfile);
            assert!(dry_run);
        }
        _ => panic!("expected install"),
    }
    assert!(parse(&["install", "--no-frozen-lockfile"]).is_ok());
}

#[test]
fn the_two_frozen_flags_together_are_refused() {
    assert!(parse(&["install", "--frozen-lockfile", "--no-frozen-lockfile"]).is_err());
}

#[test]
fn there_is_no_reconcile_word_and_no_verify_command() {
    assert!(parse(&["install", "--reconcile"]).is_err());
    assert!(parse(&["reconcile"]).is_err());
    assert!(parse(&["--verify"]).is_err());
    assert!(parse(&["verify"]).is_err());
}

#[test]
fn update_takes_dry_run_and_nothing_else() {
    assert!(parse(&["update", "--dry-run"]).is_ok());
    assert!(parse(&["update", "--verify"]).is_err());
}
