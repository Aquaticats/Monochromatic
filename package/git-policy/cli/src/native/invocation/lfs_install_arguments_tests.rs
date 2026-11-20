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
///  The notice names the scope the caller asked for,
///  because `--local`,
/// `--worktree`,
/// `--system` and `--file` write the keys somewhere else than the global file,
///  and `--manual`'s hook instructions are not printed by a run that installs no hook.
/// Why:
///   Measured with git-lfs 3.8.1 through the wrapper, read back with `git config --file <p> --list`:
///  `--local` writes `<repo>/.git/config`,
///  `--worktree` writes `<repo>/.git/config.worktree` while `extensions.worktreeConfig` is set and
///  `<repo>/.git/config` when it is not,
///  `--file=<path>` writes that file,
///  and the global file stays empty in all three;
///  the notice said "the global `filter.lfs.*` keys were written as usual" for every one of them,
///  which is false for three.
///  A boolean option's `=value` is read too: `--local=true` names the repository's file,
///  and a false value resets the scope that option had set.
///  `--manual` asks git-lfs for the instructions that merge its hooks into this repository by hand,
///  and this run installs no hook,
/// so none are printed and the notice has to say why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(withSkipRepo(words('lfs install --local'), true)[1]).toContain("this repository's `.git/config`")
/// ```
#[test]
fn the_notice_names_the_scope_the_caller_asked_for() {
    for (case, typed, wanted) in [
        ("the default scope is the global file", vec!["lfs", "install"], "global configuration"),
        ("--force keeps the global scope", vec!["lfs", "install", "--force"], "global configuration"),
        ("--local names the repository's own file", vec!["lfs", "install", "--local"], "this repository's `.git/config`"),
        ("-l names it too", vec!["lfs", "install", "-l"], "this repository's `.git/config`"),
        ("--worktree names the worktree file and its condition", vec!["lfs", "install", "--worktree"], "`.git/config.worktree`"),
        ("--system names the system file", vec!["lfs", "install", "--system"], "system configuration"),
        ("--file names the file it was given", vec!["lfs", "install", "--file=/work/elsewhere.conf"], "`/work/elsewhere.conf`"),
        ("--file's value may be the next word", vec!["lfs", "install", "--file", "elsewhere.conf"], "`elsewhere.conf`"),
        ("--local=true names the repository's own file", vec!["lfs", "install", "--local=true"], "this repository's `.git/config`"),
        ("--worktree=false leaves the global file", vec!["lfs", "install", "--worktree=false"], "global configuration"),
        ("--system=true names the system file", vec!["lfs", "install", "--system=true"], "system configuration"),
        ("a later false value resets --local", vec!["lfs", "install", "--local", "--local=false"], "global configuration"),
        ("a later false value resets --system", vec!["lfs", "install", "--system", "--system=false"], "global configuration"),
        ("a false --worktree leaves the earlier --local", vec!["lfs", "install", "--local", "--worktree=false"], "this repository's `.git/config`"),
        ("--file= resets the file scope", vec!["lfs", "install", "--file=/work/elsewhere.conf", "--file="], "global configuration"),
    ] {
        let (_, notice) = with_skip_repo(words(typed.as_slice()), true);
        assert!(notice.contains(wanted), "{case}: the notice names the scope: {notice}");
        if !wanted.contains("global") {
            assert!(!notice.contains("The global `filter.lfs.*` keys"), "{case}: a local scope is not called global: {notice}");
        }
    }
    let (_, worktree_notice) = with_skip_repo(words(&["lfs", "install", "--worktree"]), true);
    assert!(worktree_notice.contains("extensions.worktreeConfig"), "the worktree notice says when Git reads that file: {worktree_notice}");
    assert!(
        worktree_notice.contains("this repository's `.git/config`"),
        "the worktree notice names the extension-unset file too: {worktree_notice}"
    );
    let (_, manual) = with_skip_repo(words(&["lfs", "install", "--manual"]), true);
    assert!(manual.contains("`--manual`"), "the notice answers what `--manual` asked for: {manual}");
    assert!(manual.contains("none are printed here"), "the notice says the hook instructions are not printed: {manual}");
    let (_, force_manual) = with_skip_repo(words(&["lfs", "install", "-m"]), true);
    assert!(force_manual.contains("`--manual`"), "the short form of `--manual` is read too: {force_manual}");
    let (_, manual_on_value) = with_skip_repo(words(&["lfs", "install", "--manual=true"]), true);
    assert!(manual_on_value.contains("none are printed here"), "--manual=true asks for the instructions too: {manual_on_value}");
    for off in [vec!["lfs", "install", "--manual=false"], vec!["lfs", "install", "--manual", "--manual=false"]] {
        let (_, silent) = with_skip_repo(words(off.as_slice()), true);
        assert!(!silent.contains("none are printed here"), "{off:?} asks for no hook instructions: {silent}");
    }
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
    assert!(notice.contains("this repository's `.git/config`"), "the notice still names the scope: {notice}");
}
