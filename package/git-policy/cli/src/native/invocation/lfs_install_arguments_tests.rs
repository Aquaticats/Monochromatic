//! Controls for the `--skip-repo` a `git lfs install` runs with inside a repository,
//! and the notice that comes with it.

use super::*;

/// An argument list from words.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// What:
///  `git lfs install` inside a repository runs with `--skip-repo` right after its `install` word,
///  and the notice says the filters were written and no hook is needed;
///  the same command outside a repository,
///  a help query,
///  and every other command are left as they were.
/// Why:
///   The wrapper starts Git with traditional hooks off,
///  so git-lfs's hook write fails with `mkdir /dev/null: not a directory`,
///  status 2,
///  and advice that cannot work;
///  the global `filter.lfs.*` keys it writes first are what the wrapper's own upload needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(withSkipRepo(words('lfs install'), true)[0]).toEqual(['lfs', 'install', '--skip-repo'])
/// ```
#[test]
fn an_install_inside_a_repository_runs_with_skip_repo_and_its_notice() {
    let (rewritten, notice) = with_skip_repo(words(&["lfs", "install"]), true);
    assert_eq!(rewritten, words(&["lfs", "install", SKIP_REPO_OPTION]));
    assert!(notice.contains("no `pre-push` hook is installed in this repository and none is needed"), "{notice}");
    assert!(notice.contains("`filter.lfs.*` keys were written"), "{notice}");
    assert!(!notice.contains("run `git lfs update --manual`"), "no advice that cannot work is offered: {notice}");

    let (with_a_caller_setting, setting_notice) = with_skip_repo(words(&["-c", "lfs.skipdownloaderrors=true", "lfs", "install", "--force"]), true);
    assert_eq!(with_a_caller_setting, words(&["-c", "lfs.skipdownloaderrors=true", "lfs", "install", SKIP_REPO_OPTION, "--force"]));
    assert!(!setting_notice.is_empty());

    for untouched in [
        vec!["lfs", "install"],
        vec!["lfs", "install", "--skip-smudge"],
        vec!["lfs", "install", "--help"],
        vec!["lfs", "install", "-h"],
        vec!["lfs", "install", "hooks"],
        vec!["lfs", "update"],
        vec!["lfs", "uninstall"],
        vec!["push", "origin", "main"],
        vec!["status"],
    ] {
        let (same, silent) = with_skip_repo(words(untouched.as_slice()), false);
        assert_eq!(same, words(untouched.as_slice()), "{untouched:?} outside a repository");
        assert!(silent.is_empty(), "{untouched:?} outside a repository says nothing");
    }
    let (help, silent) = with_skip_repo(words(&["lfs", "install", "--help"]), true);
    assert_eq!(help, words(&["lfs", "install", "--help"]), "a help query is answered as it was typed");
    assert!(silent.is_empty());
}

/// What:
///  A caller who already wrote `--skip-repo` keeps one copy of it and is told the same thing,
///  because the reason is the wrapper's.
/// Why:
///   Writing it twice is harmless to git-lfs,
///  but a person who asked for it should still read why the wrapper would have added it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(withSkipRepo(words('lfs install --skip-repo'), true)[0]).toEqual(['lfs', 'install', '--skip-repo'])
/// ```
#[test]
fn a_caller_who_wrote_the_option_already_keeps_one_copy_and_gets_the_notice() {
    let (rewritten, notice) = with_skip_repo(words(&["lfs", "install", SKIP_REPO_OPTION, "--local"]), true);
    assert_eq!(rewritten, words(&["lfs", "install", SKIP_REPO_OPTION, "--local"]));
    assert!(notice.contains("`filter.lfs.*` keys were written"), "{notice}");
}
