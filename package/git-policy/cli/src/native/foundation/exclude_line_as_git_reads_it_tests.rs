//! Controls of the rule that reads one exclude-file line the way Git reads it:
//! which forms of the wrapper's line Git honours,
//! measured with real Git on this host,
//! and a second control that asks Git itself for each of them.

use super::{the_pattern_git_reads, without_the_line_ending};
use crate::test_support::{fixture, remove, repository};
use std::path::PathBuf;

/// The line the wrapper keeps the trash out of every commit with,
/// spelled here as the bytes a person's exclude file holds,
/// so this control never depends on the module that writes it.
const THE_LINE: &[u8] = b"/.claude/cli-git-trash/";

/// What one form of the line is,
/// and whether Git ignores the trash with it:
/// measured on this host with Git 2.56.0 on 2026-11-13,
/// `git check-ignore --no-index --verbose .claude/cli-git-trash/credentials.txt`.
const FORMS: [( &[u8], bool); 11] = [
    (b"/.claude/cli-git-trash/\n", true),
    (b"/.claude/cli-git-trash/", true),
    (b"/.claude/cli-git-trash/\r\n", true),
    (b"/.claude/cli-git-trash/ \n", true),
    (b"/.claude/cli-git-trash/  \n", true),
    (b"/.claude/cli-git-trash/\t\n", false),
    (b"/.claude/cli-git-trash/\t \n", false),
    (b"/.claude/cli-git-trash/ \t\n", false),
    (b" /.claude/cli-git-trash/\n", false),
    (b"\t/.claude/cli-git-trash/\n", false),
    (b"/.claude/cli-git-trash/\\ \n", false),
];

/// What:
///  The rule agrees with Git for every form a person's file can hold:
///  the pattern it reads is the wrapper's line exactly when Git ignores the trash,
///  and the line ending never survives.
/// Why:
///   A wrapper that took a form Git does not honour for its own would append nothing and leave the trash
///   stageable,
///   which is the whole reason the line exists.
///   The eleven forms are the ones the rule has to get right,
///   and the `FORMS` table is the measurement they come from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [line, ignored] of FORMS) expect(thePatternGitReads(line).equals(THE_LINE)).toBe(ignored)
/// ```
#[test]
fn the_rule_agrees_with_git_for_every_form_of_the_line() {
    for (line, ignored) in FORMS {
        let pattern: &[u8] = the_pattern_git_reads(line);
        let shown: String = String::from_utf8_lossy(line).into_owned();
        assert_eq!(pattern == THE_LINE, ignored, "for {shown:?} the pattern is {pattern:?} and Git ignores the trash: {ignored}");
        assert!(!pattern.contains(&b'\n'), "the line ending never survives: {shown:?}");
    }
}

/// What:
///  Git itself agrees, for each of those forms,
///  about whether the trash is ignored,
///  and about it agreeing with the rule.
/// Why:
///   The table above is a measurement someone took once;
/// this control asks the Git on this host every time,
/// so a Git whose rule differs from the table fails here rather than in a person's repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [line, ignored] of FORMS) { write(exclude, line); expect(gitCheckIgnore(repo) === ignored).toBe(true) }
/// ```
#[test]
fn real_git_ignores_the_trash_for_exactly_the_forms_the_rule_calls_its_own() {
    let root: PathBuf = fixture("exclude-line-forms");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir_all(repo.join(".claude/cli-git-trash")).expect("the trash place");
    std::fs::write(repo.join(".claude/cli-git-trash/credentials.txt"), b"private bytes\n").expect("a file in the trash");
    let exclude: PathBuf = repo.join(".git/info/exclude");
    for (line, ignored) in FORMS {
        std::fs::write(exclude.as_path(), line).expect("the exclude file a person's editor left");
        let named: std::process::Output = std::process::Command::new("git")
            .args(["-C"])
            .arg(repo.as_path())
            .args(["check-ignore", "--no-index", "--verbose", ".claude/cli-git-trash/credentials.txt"])
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .output()
            .expect("git runs");
        let git_ignores: bool = named.status.success();
        let shown: String = String::from_utf8_lossy(line).into_owned();
        assert_eq!(git_ignores, ignored, "Git on this host, for {shown:?}: {shown:?} {}", String::from_utf8_lossy(named.stdout.as_slice()));
        assert_eq!(the_pattern_git_reads(line) == THE_LINE, git_ignores, "the rule and Git agree for {shown:?}");
    }
    remove(root.as_path());
}

/// What:
///  A line's ending is removed whether the caller handed it in with the ending or without,
///  and a carriage return is removed only as part of one.
/// Why:
///   One caller splits a file at its line ends and another hands a whole line in,
/// and a `\r` inside a pattern is a byte Git keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(withoutTheEnding("a\r\n")).toBe("a"); expect(withoutTheEnding("a\rb")).toBe("a\rb")
/// ```
#[test]
fn the_line_ending_is_the_only_thing_removed_before_the_spaces() {
    assert_eq!(without_the_line_ending(b"a\r\n"), b"a");
    assert_eq!(without_the_line_ending(b"a\n"), b"a");
    assert_eq!(without_the_line_ending(b"a"), b"a");
    assert_eq!(without_the_line_ending(b"a\rb"), b"a\rb");
    assert_eq!(without_the_line_ending(b"a\r"), b"a");
}
