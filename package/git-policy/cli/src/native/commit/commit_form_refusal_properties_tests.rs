//! What:
//!  Two properties over every generated `git commit` command line:
//!  the form rule refuses exactly what real Git refuses,
//!  and the question the wrapper asks leaves `HEAD`,
//!  every reference,
//!  the person's index and the worktree as they were.
//! Why:
//!  A rule wider than Git refuses a commit Git makes,
//!  and a rule narrower lets a mistyped command through;
//!  a question that is not a dry run once made a real commit,
//!  in a repository the person's command never named.
//!  Each of those is one statement about every command line,
//!  so the control generates them rather than naming the cases someone
//!  thought of:
//!  every selection option,
//!  every option that changes whether an empty change is allowed,
//!  and every way of naming paths.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const line of everyLine()) { expect(ruleFlags(line)).toBe(gitRefusesWithAFormMessage(line)); expect(snapshot()).toEqual(before); }
//! ```

use crate::command_commit::parse_commit_region;
use crate::commit_form_refusal_from_git::{Answer, ask_git};
use crate::rule_commit_region_git_refuses::form_git_refuses;
use crate::testing::test_support::{fixture, git, git_context, git_text, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The selection options,
/// which decide the compatibility check and the pathless-selection rule.
const SELECTIONS: [&[&str]; 5] = [&[], &["-a"], &["-i"], &["-o"], &["--patch"]];

/// The options that decide whether an empty change is allowed,
/// and which fixup prefix is in force.
const EMPTY_CHANGE_OPTIONS: [&[&str]; 7] = [
    &[],
    &["--interactive"],
    &["--allow-empty"],
    &["--amend"],
    &["--fixup=HEAD"],
    &["--fixup=amend:HEAD"],
    &["--fixup=reword:HEAD"],
];

/// The ways of naming paths,
/// including the two that make Git read the count itself,
/// and one that matches nothing.
const PATHS: [&[&str]; 6] = [
    &[],
    &["--", "a.txt"],
    &["--", "nothing.txt"],
    &["--pathspec-from-file=paths.txt"],
    &["--pathspec-file-nul"],
    &["--pathspec-from-file=paths.txt", "--pathspec-file-nul"],
];

/// The command lines whose `--` is an option's value rather than the separator,
///  which is the shape that once made the question a real commit.
const SEPARATORS_AS_VALUES: [&[&str]; 4] = [
    &["-m", "--", "-i"],
    &["-m", "--", "--", "a.txt"],
    &["-F", "--", "-i"],
    &["--pathspec-from-file", "--", "-a"],
];

/// One generated command line:
/// the person's options,
/// after `commit`.
fn every_line() -> Vec<Vec<String>> {
    let mut lines: Vec<Vec<String>> = Vec::new();
    for selection in SELECTIONS {
        for empty_change in EMPTY_CHANGE_OPTIONS {
            for paths in PATHS {
                let mut line: Vec<String> = Vec::new();
                for token in [selection, empty_change].concat() {
                    line.push(token.to_owned());
                }
                line.push("-m".to_owned());
                line.push("x".to_owned());
                for token in paths {
                    line.push((*token).to_owned());
                }
                lines.push(line);
            }
        }
    }
    // A `--` that is an option's value is the separator to nothing but a reading that skips values, and a question that
    //  placed its own option before that token made a commit instead of asking one.
    for separator_as_value in SEPARATORS_AS_VALUES {
        lines.push(separator_as_value.iter().map(|token| { return (*token).to_owned(); }).collect());
    }
    return lines;
}

/// What:
///  Whether Git's words are one of the messages the form rules stand for.
/// Why:
///  Git refuses other things too (`--squash` with `--fixup`,
/// a commit that is not
///  there),
///  so a control that counted every fatal status would pass a rule that is wider
///  than Git.
///  Each pattern below is one of `builtin/commit.c`'s own sentences for the forms
///  this file names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAFormMessage = (text) => FORMS.some((pattern) => text.startsWith(pattern));
/// ```
fn is_a_form_message(stderr: &[u8]) -> bool {
    let printed: String = String::from_utf8_lossy(stderr).into_owned();
    let first: &str = printed.lines().next().unwrap_or("");
    // The compatibility check names the options it counts, so a selection conflict always names one of them.
    if first.starts_with("fatal: options '")
        && ["-i/--include", "-o/--only", "-a/--all", "--interactive/-p/--patch"]
            .iter()
            .any(|option| { return first.contains(option); })
    {
        return true;
    }
    let patterns: [&str; 6] = [
        "fatal: No paths with --include/--only does not make sense.",
        "fatal: '--pathspec-from-file' and pathspec arguments cannot be used together",
        "fatal: options '--pathspec-from-file' and",
        "fatal: reword option of '--fixup' and",
        "fatal: the option '--pathspec-file-nul' requires '--pathspec-from-file'",
        "fatal: paths '",
    ];
    return patterns.iter().any(|pattern| { return first.starts_with(pattern); });
}

/// What:
///  Everything the question may not change:
///  the commit `HEAD` names,
///  every reference,
///  the person's index bytes,
///  and every file of the worktree with its bytes.
/// Why:
///  "No commit,
/// no reference and no record" is one statement about all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshot = () => ({ head, refs, index: read(indexPath), worktree: walk() });
/// ```
#[derive(Debug, Eq, PartialEq)]
struct Unchanged {
    /// The commit `HEAD` names.
    head: String,
    /// Every reference with the object it names.
    refs: String,
    /// The person's index,
    /// byte for byte.
    index: Vec<u8>,
    /// Every file of the worktree,
    /// by name,
    /// with its bytes.
    worktree: Vec<(String, Vec<u8>)>,
}

/// The state of the repository,
/// for the comparison a row makes.
fn unchanged(repo: &Path) -> Unchanged {
    let mut worktree: Vec<(String, Vec<u8>)> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![repo.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            let name: String = path.strip_prefix(repo).unwrap_or(path.as_path()).display().to_string();
            // The object store is not read: a dry run writes no object, and reading a pack is no part of this property.
            //  Everything else is, the question's own scratch index and the repository's logs included, so a leftover from
            //  the question shows up as a difference.
            if name == ".git/objects" {
                continue;
            }
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let bytes: Vec<u8> = std::fs::read(path.as_path()).unwrap_or_default();
            worktree.push((name, bytes));
        }
    }
    worktree.sort();
    return Unchanged {
        head: git_text(repo, &["rev-parse", "HEAD"]),
        refs: git_text(repo, &["for-each-ref", "--format=%(refname) %(objectname)"]),
        index: std::fs::read(repo.join(".git/index")).unwrap_or_default(),
        worktree,
    };
}

/// What:
///  The rule and the question agree,
/// for every generated command line,
///  and the question changes nothing.
/// Why:
///  Both properties are about every line and not about the ones someone named;
///  a line where they disagree says which line and what Git answered.
#[test]
fn the_rule_refuses_exactly_what_git_refuses_and_the_question_changes_nothing() {
    let root: PathBuf = fixture("commit-form-properties");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), "a\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), "a changed\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    std::fs::write(repo.join("paths.txt"), "a.txt\n").expect("pathspecs");
    let mut context: crate::transaction_git::GitContext = git_context();
    // Git's own words are English whatever locale the machine runs with, so the classification below reads them.
    context.overlay.push((OsString::from("LC_ALL"), OsString::from("C")));
    let mut counted: usize = 0;
    for line in every_line() {
        let region_args: Vec<OsString> = line.iter().map(OsString::from).collect();
        let Ok(region) = parse_commit_region(region_args.as_slice(), &[]) else {
            continue;
        };
        let before: Unchanged = unchanged(repo.as_path());
        let answer: Answer = ask_git(&context, repo.as_path(), region_args.as_slice());
        let after: Unchanged = unchanged(repo.as_path());
        let flagged: bool = form_git_refuses(&region).is_some();
        let git_refused: bool = match &answer {
            Answer::Answered { status, stderr } => *status == 128 && is_a_form_message(stderr.as_slice()),
            Answer::Unanswered(_) => false,
        };
        assert_eq!(
            flagged,
            git_refused,
            "{line:?}: the rule says {flagged}, real Git says {git_refused} ({answer:?})"
        );
        assert_eq!(after, before, "{line:?}: the question changed the repository ({answer:?})");
        counted += 1;
    }
    assert!(counted > 100, "the table covers the generated lines: {counted}");
}
