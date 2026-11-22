//! What:
//!  Controls for the option table of `git worktree add`
//!  and the rules that read it:
//! a long word resolves as Git's own parser resolves it,
//! an exact name wins over the names it is a prefix of,
//!  and the table lists exactly the options `git worktree add -h` prints.
//! Why:
//!   Every reading of the command line takes its options from this one table,
//! so a table that drifts from Git silently changes which words are operands,
//! and a resolution rule that drifts changes which option a word is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(table).toEqual(optionsFromGitHelp());
//! ```

use super::*;
use crate::test_support::{fixture, remove, repository};
use std::path::PathBuf;
use std::process::Output;

/// Long options resolve as Git's `parse_long_opt` does:
/// an exact name wins over longer names,
/// a unique prefix is the option,
/// the `--no-` form negates,
/// and two candidates,
/// a prefix of `no-` and a double negation resolve to nothing.
#[test]
fn a_long_option_resolves_the_way_gits_parser_does() {
    let name = |text: &str| -> Option<(&'static str, bool, Option<String>)> {
        return resolve_long(text).map(|found| { return (found.option.long.unwrap_or("-"), found.unset, found.attached.map(String::from)); });
    };
    assert_eq!(name("detach"), Some(("detach", false, None)));
    assert_eq!(name("det"), Some(("detach", false, None)), "a unique prefix");
    assert_eq!(name("d"), Some(("detach", false, None)));
    assert_eq!(name("no-che"), Some(("checkout", true, None)), "a negated prefix");
    assert_eq!(name("no-checkout"), Some(("checkout", true, None)));
    assert_eq!(name("reas=why"), Some(("reason", false, Some(String::from("why")))), "a prefix with its value");
    assert_eq!(name("reason="), Some(("reason", false, Some(String::new()))), "an empty value is a value");
    assert_eq!(name("re"), None, "reason and relative-paths");
    assert_eq!(name("rel"), Some(("relative-paths", false, None)));
    assert_eq!(name("f"), Some(("force", false, None)));
    assert_eq!(name("g"), Some(("guess-remote", false, None)));
    assert_eq!(name("l"), Some(("lock", false, None)));
    assert_eq!(name("o"), Some(("orphan", false, None)));
    assert_eq!(name("t"), Some(("track", false, None)));
    assert_eq!(name("n"), None, "a prefix of no- abbreviates every negation");
    assert_eq!(name("no"), None);
    assert_eq!(name("no-"), None);
    assert_eq!(name("no-no-checkout"), None, "a double negation");
    assert_eq!(name("nonsense"), None);
    assert_eq!(name("detachx"), None, "a longer text than the name is not a prefix");
    assert_eq!(name("c"), Some(("checkout", false, None)));
}

/// The table of options is the list `git worktree add -h` prints,
/// so it cannot drift from Git:
/// every option Git lists is in
/// the table with the same letter,
/// name and value,
/// and the table lists nothing Git does not.
#[test]
fn the_option_table_matches_what_git_prints_for_dash_h() {
    // `git worktree` sets up a repository before it reads `-h`, so it runs inside a fixture repository: outside one, as in
    // the container gate's working directory, Git stops with "not a git repository" and prints no usage.
    let root: PathBuf = fixture("add-plan-dash-h");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let output: Output = std::process::Command::new(crate::test_support::REAL_GIT)
        .args(["worktree", "add", "-h"])
        .current_dir(repo.as_path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("git runs");
    remove(root.as_path());
    // `-h` prints the usage on standard output with status 129, and the option lines follow a blank line.
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned() + String::from_utf8_lossy(output.stderr.as_slice()).as_ref();
    let listed: Vec<(Option<u8>, Option<String>, bool)> = text
        .lines()
        .skip_while(|line| { return !line.is_empty(); })
        .filter(|line| { return line.starts_with("    -"); })
        .map(|line| {
            // The spelling is dash words and `<placeholder>` words; the description starts at the first other word.
            let mut short: Option<u8> = None;
            let mut long: Option<String> = None;
            let mut takes: bool = false;
            for word in line.split_whitespace() {
                let head: &str = word.trim_end_matches(',');
                if let Some(rest) = head.strip_prefix("--") {
                    long = Some(rest.replace("[no-]", ""));
                } else if let Some(letter) = head.strip_prefix('-') {
                    short = letter.bytes().next();
                } else if head.starts_with('<') {
                    takes = true;
                } else {
                    break;
                }
            }
            return (short, long, takes);
        })
        .collect();
    let table: Vec<(Option<u8>, Option<String>, bool)> = ADD_OPTIONS
        .iter()
        .map(|option| { return (option.short, option.long.map(String::from), option.takes == Takes::Required); })
        .collect();
    assert!(!listed.is_empty(), "the usage lists options: {text}");
    for entry in &listed {
        assert!(table.contains(entry), "Git lists {entry:?}, which the table lacks: {text}");
    }
    for entry in &table {
        assert!(listed.contains(entry), "the table lists {entry:?}, which Git does not: {text}");
    }
}

/// An exact name wins over a longer name it is a prefix of,
/// even when the longer name comes first;
/// the real table has no such
/// pair,
/// so a table that has one shows the rule.
#[test]
fn an_exact_name_wins_over_the_names_it_is_a_prefix_of() {
    static TABLE: &[AddOption] = &[
        AddOption { short: None, long: Some("reason"), takes: Takes::Nothing },
        AddOption { short: None, long: Some("re"), takes: Takes::Required },
        AddOption { short: None, long: Some("reset"), takes: Takes::Nothing },
    ];
    let exact = resolve_in(TABLE, "re").expect("the exact name");
    assert_eq!(exact.option.long, Some("re"));
    assert!(resolve_in(TABLE, "res").is_some_and(|prefix| { return prefix.option.long == Some("reset"); }), "a unique prefix");
    assert!(resolve_in(TABLE, "rea").is_some_and(|prefix| { return prefix.option.long == Some("reason"); }));
    assert!(resolve_in(TABLE, "r").is_none(), "three names share the prefix");
    assert!(resolve_in(TABLE, "re=1").is_some_and(|valued| { return valued.attached == Some("1"); }));
}
