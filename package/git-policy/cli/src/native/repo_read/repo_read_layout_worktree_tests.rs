//! Differential controls for the per-worktree configuration (`extensions.worktreeConfig` and `config.worktree`):
//!  whenever the reader answers, `git rev-parse` prints the same five lines, and every spelling the reader cannot
//!  interpret exactly is declined.
//! The rules come from `Documentation/config/extensions.adoc` (the extension is respected at every repository format
//!  version; `config.worktree` overrides the other files) and from `setup.c` `read_and_verify_repository_format` of
//!  Git 2.56.0 (the file is read from the worktree's own Git directory; with the extension on, the common
//!  configuration's `core.bare` and `core.worktree` apply to linked worktrees too, and the per-worktree file overrides).

use super::tests::{answered_equal, declined, git_lines, read_here, spelled};
use crate::test_support::{fixture, git, remove, repository};
use std::path::{Path, PathBuf};

/// One repository with its linked worktree, and the Git directory of each.
struct Pair {
    /// The main worktree.
    main: PathBuf,
    /// The Git directory of the main worktree, which is the common directory.
    main_git: PathBuf,
    /// The linked worktree.
    linked: PathBuf,
    /// The linked worktree's own Git directory below the common one.
    linked_git: PathBuf,
}

impl Pair {
    /// The start directory (a nested one, so the prefix is exercised) and the Git directory of one side.
    fn side(&self, linked: bool) -> (PathBuf, &Path) {
        if linked {
            return (self.linked.join("sub"), self.linked_git.as_path());
        }
        return (self.main.join("sub"), self.main_git.as_path());
    }
}

/// A repository and a linked worktree whose common configuration got `common_extra` appended, at format version 0 or 1.
fn pair(root: &Path, name: &str, version_one: bool, common_extra: &str) -> Pair {
    let main = repository(root, name);
    let linked = root.join(format!("{name}-linked"));
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", format!("side-{name}").as_str(), linked.to_str().expect("UTF-8"), "HEAD"]);
    if version_one {
        git(main.as_path(), &["config", "--local", "core.repositoryformatversion", "1"]);
    }
    let config = main.join(".git/config");
    let mut text = std::fs::read_to_string(&config).expect("config");
    text.push_str(common_extra);
    std::fs::write(&config, text).expect("append to config");
    std::fs::create_dir(main.join("sub")).expect("sub");
    std::fs::create_dir(linked.join("sub")).expect("sub");
    let linked_git = main.join(".git/worktrees").join(format!("{name}-linked"));
    assert!(linked_git.join("commondir").exists(), "the linked worktree's Git directory");
    return Pair { main_git: main.join(".git"), main, linked, linked_git };
}

/// The extension switched on, spelled as a boolean Git reads as true.
const ENABLED: &str = "[extensions]\n\tworktreeConfig = true\n";

/// Whether the reader must answer, or must decline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Expect {
    /// The reader answers and equals Git.
    Answer,
    /// The reader declines.
    Decline,
}

/// Check one start directory against the expectation.
fn check(root: &Path, start: &Path, label: &str, expect: Expect) {
    match expect {
        Expect::Answer => {
            answered_equal(root, start, label);
        }
        Expect::Decline => declined(root, start, label),
    }
}

/// Check both sides of every pair for one `config.worktree` text and the files it includes.
fn check_file(root: &Path, pairs: &[&Pair], label: &str, text: &str, files: &[(&str, &str)], expect: Expect) {
    for (number, pair) in pairs.iter().enumerate() {
        for linked in [false, true] {
            let (start, git_dir) = pair.side(linked);
            std::fs::write(git_dir.join("config.worktree"), text).expect("config.worktree");
            for (name, content) in files {
                std::fs::write(git_dir.join(name), content).expect("included file");
            }
            check(root, start.as_path(), format!("{label} (pair {number}, linked {linked})").as_str(), expect);
            std::fs::remove_file(git_dir.join("config.worktree")).expect("remove config.worktree");
        }
    }
}

/// The extension on, without any `config.worktree`, is read at both format versions for the main and the linked worktree.
#[test]
fn an_enabled_extension_without_a_file_is_read() {
    let root = fixture("repo-read-wt-enabled");
    for (number, version_one) in [false, true].into_iter().enumerate() {
        for spelling in ["true", "yes", "on", "1", "TRUE"] {
            let name = format!("e{number}-{spelling}");
            let both = pair(root.as_path(), name.as_str(), version_one, format!("[extensions]\n\tworktreeConfig = {spelling}\n").as_str());
            for linked in [false, true] {
                let (start, _) = both.side(linked);
                let layout = answered_equal(root.as_path(), start.as_path(), format!("{name} linked={linked}").as_str());
                assert_eq!(layout.prefix, b"sub/".to_vec());
            }
        }
    }
    // A bare name is a true boolean to Git.
    let bare_name = pair(root.as_path(), "bare-name", false, "[extensions]\n\tworktreeConfig\n");
    answered_equal(root.as_path(), bare_name.side(false).0.as_path(), "bare name, main");
    answered_equal(root.as_path(), bare_name.side(true).0.as_path(), "bare name, linked");
    remove(root.as_path());
}

/// A disabled or absent extension means Git never reads `config.worktree`, so a file that would break a read is ignored.
#[test]
fn a_file_is_ignored_when_the_extension_is_off() {
    let root = fixture("repo-read-wt-off");
    let ignored = "[core]\n\tbare = true\n\tworktree = /elsewhere\n\tfilemode = maybe\n";
    for (number, extension) in ["", "[extensions]\n\tworktreeConfig = false\n", "[extensions]\n\tworktreeConfig = no\n"].into_iter().enumerate() {
        let both = pair(root.as_path(), format!("off-{number}").as_str(), number > 0, extension);
        check_file(root.as_path(), &[&both], format!("extension variant {number}").as_str(), ignored, &[], Expect::Answer);
    }
    remove(root.as_path());
}

/// An extension value that is not a boolean is declined, whatever Git makes of it.
#[test]
fn a_non_boolean_extension_value_is_declined() {
    let root = fixture("repo-read-wt-nonbool");
    for (number, value) in ["maybe", "2", ""].into_iter().enumerate() {
        let both = pair(root.as_path(), format!("nb-{number}").as_str(), true, format!("[extensions]\n\tworktreeConfig = {value}\n").as_str());
        for linked in [false, true] {
            declined(root.as_path(), both.side(linked).0.as_path(), format!("value {value:?}, linked {linked}").as_str());
        }
    }
    remove(root.as_path());
}

/// A `config.worktree` that sets only values the layout does not depend on is read, for every format version and side,
///  and the values it sets really are in force for Git (so the reader answered about the file Git reads).
#[test]
fn values_only_the_worktree_file_sets_are_read() {
    let root = fixture("repo-read-wt-values");
    let v0 = pair(root.as_path(), "values-v0", false, ENABLED);
    let v1 = pair(root.as_path(), "values-v1", true, ENABLED);
    let text = "[user]\n\tname = Only Here\n[core]\n\tfilemode = true\n\thooksPath = /nowhere\n[alias]\n\tx = status\n";
    check_file(root.as_path(), &[&v0, &v1], "worktree-only values", text, &[], Expect::Answer);
    // The value is Git's too: the file is the one Git reads in each worktree.
    for linked in [false, true] {
        let (start, git_dir) = v1.side(linked);
        std::fs::write(git_dir.join("config.worktree"), text).expect("config.worktree");
        let output = git(start.as_path(), &["config", "--get", "user.name"]);
        assert_eq!(String::from_utf8(output.stdout).expect("UTF-8").trim(), "Only Here", "linked {linked}");
    }
    remove(root.as_path());
}

/// A bad typed value in the per-worktree file fails every Git command, so the reader declines; so it does for the
///  common file when the worktree file has a good value for the same key (Git types every entry).
#[test]
fn typed_values_in_both_files_are_checked() {
    let root = fixture("repo-read-wt-typed");
    let both = pair(root.as_path(), "typed", true, ENABLED);
    check_file(root.as_path(), &[&both], "good typed value", "[core]\n\tfilemode = false\n", &[], Expect::Answer);
    check_file(root.as_path(), &[&both], "bad typed value", "[core]\n\tfilemode = maybe\n", &[], Expect::Decline);
    for linked in [false, true] {
        let (start, _) = both.side(linked);
        assert!(git_lines(start.as_path()).is_some(), "the fixture is healthy without the bad file");
    }
    let conflict = pair(root.as_path(), "typed-conflict", true, format!("{ENABLED}[core]\n\tfilemode = maybe\n").as_str());
    check_file(root.as_path(), &[&conflict], "bad common value overridden", "[core]\n\tfilemode = true\n", &[], Expect::Decline);
    remove(root.as_path());
}

/// `core.bare` and `core.worktree`: the per-worktree file overrides the common one, the last entry wins, and any
///  state that makes the repository bare or moves the worktree is declined.
#[test]
fn bare_and_worktree_settings_follow_the_precedence_git_applies() {
    let root = fixture("repo-read-wt-precedence");
    let bare_common = pair(root.as_path(), "common-bare", true, format!("{ENABLED}[core]\n\tbare = true\n").as_str());
    let false_common = pair(root.as_path(), "common-false", true, format!("{ENABLED}[core]\n\tbare = false\n").as_str());
    let moved_common = pair(root.as_path(), "common-moved", true, format!("{ENABLED}[core]\n\tworktree = /elsewhere\n").as_str());
    let plain = pair(root.as_path(), "plain", true, ENABLED);
    // The common file says bare, the worktree file overrides it: not bare, and Git agrees.
    check_file(root.as_path(), &[&bare_common], "common true, worktree false", "[core]\n\tbare = false\n", &[], Expect::Answer);
    check_file(root.as_path(), &[&bare_common], "common true, worktree absent", "", &[], Expect::Decline);
    check_file(root.as_path(), &[&bare_common], "common true, worktree true", "[core]\n\tbare = true\n", &[], Expect::Decline);
    // The reverse conflict makes the worktree bare.
    check_file(root.as_path(), &[&false_common], "common false, worktree true", "[core]\n\tbare = true\n", &[], Expect::Decline);
    check_file(root.as_path(), &[&false_common], "common false, worktree false", "[core]\n\tbare = false\n", &[], Expect::Answer);
    // Last entry wins inside one file.
    check_file(root.as_path(), &[&plain], "true then false", "[core]\n\tbare = true\n\tbare = false\n", &[], Expect::Answer);
    check_file(root.as_path(), &[&plain], "false then true", "[core]\n\tbare = false\n\tbare = true\n", &[], Expect::Decline);
    check_file(root.as_path(), &[&plain], "bad boolean overridden", "[core]\n\tbare = maybe\n\tbare = false\n", &[], Expect::Decline);
    // `core.worktree` anywhere moves the worktree: declined.
    check_file(root.as_path(), &[&plain], "worktree file moves the worktree", "[core]\n\tworktree = /elsewhere\n", &[], Expect::Decline);
    check_file(root.as_path(), &[&moved_common], "common moves the worktree", "[core]\n\tbare = false\n", &[], Expect::Decline);
    check_file(root.as_path(), &[&plain], "bare and worktree together", "[core]\n\tbare = true\n\tworktree = /elsewhere\n", &[], Expect::Decline);
    remove(root.as_path());
}

/// Without the extension a `core.bare = true` then `false` in one file is not bare either (the last entry wins).
#[test]
fn the_last_core_bare_wins_without_the_extension_too() {
    let root = fixture("repo-read-wt-last");
    let both = pair(root.as_path(), "last", false, "[core]\n\tbare = true\n\tbare = false\n");
    for linked in [false, true] {
        let (start, _) = both.side(linked);
        // A linked worktree ignores the common `core.bare`; both sides must equal Git when the reader answers.
        if let Some(layout) = read_here(root.as_path(), start.as_path()) {
            assert_eq!(Some(spelled(&layout)), git_lines(start.as_path()), "linked {linked}");
        }
    }
    answered_equal(root.as_path(), both.side(false).0.as_path(), "main, last entry false");
    remove(root.as_path());
}

/// Quotes, escapes, comments, continuation lines, odd capitalisation and a cut-off value are read as Git reads them:
///  an answer equals Git, and a spelling that sets `core.bare` or `core.worktree` to anything but false is declined.
#[test]
fn adversarial_spellings_are_read_as_git_reads_them() {
    let root = fixture("repo-read-wt-spelling");
    let v0 = pair(root.as_path(), "spell-v0", false, ENABLED);
    let v1 = pair(root.as_path(), "spell-v1", true, ENABLED);
    let pairs: [&Pair; 2] = [&v0, &v1];
    let cases: [(&str, &str, Expect); 18] = [
        ("key text inside a quoted value", "[user]\n\tname = \"core.bare = true\"\n", Expect::Answer),
        ("escaped quote then comment", "[user]\n\tname = a\\\"b # core.bare = true\n", Expect::Answer),
        ("keys only in comments", "# core.bare = true\n; core.worktree = x\n[user]\n\tname = x\n", Expect::Answer),
        ("false", "[core]\n\tbare = false\n", Expect::Answer),
        ("false with trailing comment", "[core]\n\tbare = false ; was true\n", Expect::Answer),
        ("false in split quotes", "[core]\n\tbare = \"fa\"\"lse\"\n", Expect::Answer),
        ("false in a quoted word", "[core]\n\tbare = \"false\"\n", Expect::Answer),
        ("true split by a continuation line", "[core]\n\tbare = tr\\\nue\n", Expect::Decline),
        ("upper-case section and key", "[CORE]\n\tBare = TRUE\n", Expect::Decline),
        ("bare name is true", "[core]\n\tbare\n", Expect::Decline),
        ("not a boolean", "[core]\n\tbare = maybe\n", Expect::Decline),
        ("a subsection is another key", "[core \"sub\"]\n\tbare = true\n", Expect::Answer),
        ("worktree path", "[core]\n\tworktree = /elsewhere\n", Expect::Decline),
        ("worktree path in quotes", "[core]\n\tworktree = \"/elsewhere\"\n", Expect::Decline),
        ("open quote", "[core]\n\tbare = \"false\n", Expect::Decline),
        ("section header broken", "[core\n\tbare = false\n", Expect::Decline),
        ("byte order mark", "\u{feff}[user]\n\tname = x\n", Expect::Answer),
        ("carriage returns", "[user]\r\n\tname = x\r\n", Expect::Answer),
    ];
    for (label, text, expect) in cases {
        check_file(root.as_path(), &pairs, label, text, &[], expect);
    }
    remove(root.as_path());
}

/// Includes in `config.worktree` are not interpreted, only scanned: a file that mentions `core.bare` or `core.worktree`
///  makes the reader decline whatever its condition says, an include without them is read, and anything the scan
///  cannot follow is declined.
#[test]
fn includes_in_the_worktree_file_are_scanned_and_never_interpreted() {
    let root = fixture("repo-read-wt-include");
    let v1 = pair(root.as_path(), "include", true, ENABLED);
    let pairs: [&Pair; 1] = [&v1];
    let plain = ("inc-plain", "[user]\n\tname = Included\n");
    let bare = ("inc-bare", "[core]\n\tbare = true\n");
    let false_bare = ("inc-false", "[core]\n\tbare = false\n");
    let moved = ("inc-moved", "[core]\n\tworktree = /elsewhere\n");
    let nested = ("inc-nested", "[include]\n\tpath = inc-bare\n");
    let bad_type = ("inc-typed", "[core]\n\tfilemode = maybe\n");
    let cases: [(&str, &str, Vec<(&str, &str)>, Expect); 14] = [
        ("include without the keys", "[include]\n\tpath = inc-plain\n", vec![plain], Expect::Answer),
        ("include with core.bare", "[include]\n\tpath = inc-bare\n", vec![bare], Expect::Decline),
        ("include with core.bare = false", "[include]\n\tpath = inc-false\n", vec![false_bare], Expect::Decline),
        ("include with core.worktree", "[include]\n\tpath = inc-moved\n", vec![moved], Expect::Decline),
        ("nested include with core.bare", "[include]\n\tpath = inc-nested\n", vec![nested, bare], Expect::Decline),
        ("include with a bad typed value", "[include]\n\tpath = inc-typed\n", vec![bad_type], Expect::Decline),
        ("includeIf that cannot match, without keys", "[includeIf \"gitdir:/nowhere/\"]\n\tpath = inc-plain\n", vec![plain], Expect::Answer),
        ("includeIf that cannot match, with core.bare", "[includeIf \"gitdir:/nowhere/\"]\n\tpath = inc-bare\n", vec![bare], Expect::Decline),
        ("includeIf onbranch with core.bare", "[includeIf \"onbranch:nothing\"]\n\tpath = inc-bare\n", vec![bare], Expect::Decline),
        ("include of a missing file", "[include]\n\tpath = inc-missing\n", vec![], Expect::Answer),
        ("include in quotes", "[include]\n\tpath = \"inc-bare\"\n", vec![bare], Expect::Decline),
        ("include of itself", "[include]\n\tpath = config.worktree\n", vec![], Expect::Decline),
        ("include with a user path", "[include]\n\tpath = ~nobody/x\n", vec![], Expect::Decline),
        ("include without a value", "[include]\n\tpath\n", vec![], Expect::Decline),
    ];
    for (label, text, files, expect) in cases {
        check_file(root.as_path(), &pairs, label, text, files.as_slice(), expect);
    }
    // The included file is the one beside `config.worktree`, not beside the common file: a relative include that only
    // the common directory holds is a missing file for the worktree file of a linked worktree.
    std::fs::write(v1.main_git.join("only-common"), "[core]\n\tbare = true\n").expect("only common");
    let (linked_start, linked_git) = v1.side(true);
    std::fs::write(linked_git.join("config.worktree"), "[include]\n\tpath = only-common\n").expect("config.worktree");
    answered_equal(root.as_path(), linked_start.as_path(), "relative include resolves beside config.worktree");
    remove(root.as_path());
}

/// A `config.worktree` that is a directory is declined, and a dangling link is declined or equal to Git.
#[test]
fn an_unreadable_worktree_file_is_declined_or_equal_git() {
    let root = fixture("repo-read-wt-unreadable");
    let both = pair(root.as_path(), "unreadable", true, ENABLED);
    for linked in [false, true] {
        let (start, git_dir) = both.side(linked);
        let path = git_dir.join("config.worktree");
        std::fs::create_dir(&path).expect("directory");
        assert!(read_here(root.as_path(), start.as_path()).is_none(), "a directory, linked {linked}");
        std::fs::remove_dir(&path).expect("remove directory");
        std::os::unix::fs::symlink("nowhere", &path).expect("dangling link");
        if let Some(layout) = read_here(root.as_path(), start.as_path()) {
            assert_eq!(Some(spelled(&layout)), git_lines(start.as_path()), "dangling link, linked {linked}");
        }
        std::fs::remove_file(&path).expect("remove link");
    }
    remove(root.as_path());
}

/// Git clears the repository format when `core.repositoryformatversion` is absent (`read_repository_format` in
///  `setup.c`), so `extensions.worktreeConfig` switches nothing on and `config.worktree` is never read: a file that
///  would make the repository bare or break a typed value is ignored, and the reader must answer as Git does.
#[test]
fn an_absent_format_version_never_turns_the_worktree_file_on() {
    let root = fixture("repo-read-wt-no-version");
    let both = pair(root.as_path(), "no-version", false, ENABLED);
    git(both.main.as_path(), &["config", "--local", "--unset", "core.repositoryformatversion"]);
    assert!(
        !std::fs::read_to_string(both.main_git.join("config")).expect("config").contains("repositoryformatversion"),
        "the fixture has no format version"
    );
    for (label, text) in [("a bare repository", "[core]\n\tbare = true\n"), ("a bad typed value", "[core]\n\tfilemode = maybe\n")] {
        check_file(root.as_path(), &[&both], format!("absent version, {label}").as_str(), text, &[], Expect::Answer);
    }
    // The same file is read once a version is present, so the answer above is about the missing version alone.
    git(both.main.as_path(), &["config", "--local", "core.repositoryformatversion", "0"]);
    check_file(root.as_path(), &[&both], "version 0, a bad typed value", "[core]\n\tfilemode = maybe\n", &[], Expect::Decline);
    remove(root.as_path());
}
