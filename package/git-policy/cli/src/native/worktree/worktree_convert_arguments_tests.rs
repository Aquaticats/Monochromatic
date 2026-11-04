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

/// The help states the risk the person accepts, and the notice names the same thing in one line.
#[test]
fn the_help_and_the_notice_state_the_risk() {
    assert!(CONVERT_HELP.contains("Risk, accepted by running the command"));
    assert!(CONVERT_HELP.contains("another process from changing the worktree"));
    assert!(!RISK_NOTICE.contains('\n'), "the notice is one line");
    assert!(RISK_NOTICE.contains("that risk is yours"));
}

/// An attribute source chosen before the command is found: `--attr-source` in both spellings, `-c attr.tree`, and the
///  environment-supplied form; other global options are not.
#[test]
fn a_command_line_attribute_source_is_found() {
    assert!(selects_attribute_source(&words(&["--attr-source=HEAD"])));
    assert!(selects_attribute_source(&words(&["--attr-source", "HEAD"])));
    assert!(selects_attribute_source(&words(&["-c", "attr.tree=HEAD"])));
    assert!(selects_attribute_source(&words(&["-C", "/x", "-c", "ATTR.TREE=HEAD"])));
    assert!(selects_attribute_source(&words(&["--config-env=attr.tree=NAME"])));
    assert!(!selects_attribute_source(&words(&["-C", "/x", "-c", "core.autocrlf=false"])));
    assert!(!selects_attribute_source(&words(&["--no-pager"])));
    assert!(!selects_attribute_source(&[]));
}

/// `--jobs` takes a count in both spellings, and a count that is not a positive number is refused with the word it refused.
#[test]
fn jobs_takes_a_positive_count() {
    assert_eq!(parse_convert_arguments(&words(&["--all", "--jobs", "3"])).expect("spaced").jobs, Some(3));
    assert_eq!(parse_convert_arguments(&words(&["--jobs=2", "--all"])).expect("joined").jobs, Some(2));
    assert_eq!(parse_convert_arguments(&words(&["--all"])).expect("absent").jobs, None);
    assert_eq!(parse_convert_arguments(&words(&["--all", "--jobs", "3", "--jobs", "5"])).expect("last wins").jobs, Some(5));
    for refused in [&["--all", "--jobs", "0"][..], &["--all", "--jobs=0"], &["--all", "--jobs", "many"], &["--all", "--jobs="], &["--all", "--jobs"]] {
        let reason: String = parse_convert_arguments(&words(refused)).expect_err("a count is required");
        assert!(reason.contains("--jobs"), "{refused:?}: {reason}");
    }
}

/// The help names `--jobs` and says worktrees are converted side by side, and that output stays in order.
#[test]
fn the_help_describes_jobs_and_the_order_of_the_report() {
    assert!(CONVERT_HELP.contains("--jobs"));
    assert!(CONVERT_HELP.contains("side by side"));
    assert!(!CONVERT_HELP.contains("one at a time"), "the old promise is gone");
}
