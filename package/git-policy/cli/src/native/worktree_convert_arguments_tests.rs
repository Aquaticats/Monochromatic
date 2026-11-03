//! The grammar of the worktree conversion command.

use super::*;

/// The words of a command line as owned strings.
fn words(list: &[&str]) -> Vec<std::ffi::OsString> {
    return list.iter().map(|word: &&str| { return std::ffi::OsString::from(*word); }).collect();
}

/// Options and named worktrees parse; the modes exclude each other and a target is required.
#[test]
fn options_paths_and_refusals_parse() {
    let all: ConvertArguments = parse_convert_arguments(&words(&["--all", "--dry-run"])).expect("all");
    assert!(all.all && all.dry_run && !all.abandon && all.paths.is_empty());
    let named: ConvertArguments = parse_convert_arguments(&words(&["a", "b"])).expect("named");
    assert_eq!(named.paths, words(&["a", "b"]));
    assert!(parse_convert_arguments(&words(&["--dry-run", "--abandon", "a"])).is_err());
    assert!(parse_convert_arguments(&words(&["--dry-run"])).is_err(), "a target is required");
    assert!(parse_convert_arguments(&words(&["--all", "a"])).is_err(), "--all excludes named worktrees");
    assert!(parse_convert_arguments(&words(&["--bogus", "a"])).unwrap_err().contains("--bogus"));
    assert!(parse_convert_arguments(&words(&["--help"])).expect("help").help);
}
