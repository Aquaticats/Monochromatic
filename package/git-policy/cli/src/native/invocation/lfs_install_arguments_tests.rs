//! Controls for the `--skip-repo` a `git lfs install` runs with inside a repository,
//! and the notice that comes with it.

use super::*;

/// An argument list from words.
fn words(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(OsString::from).collect();
}

/// What:
///  `git lfs install` inside a repository runs with `--skip-repo` right after its `install` word,
///  and the notice says where the filters go and that no hook is needed;
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
    assert!(notice.contains("This command asks git-lfs to put the `filter.lfs.*` keys in your global configuration"), "{notice}");
    assert!(!notice.contains("keys were written"), "the notice runs before git-lfs and never promises a write: {notice}");
    assert!(!notice.contains("keys go, as this command asked, to"), "git-lfs can fail after a good parse, so the notice states a request: {notice}");
    assert!(!notice.contains("run `git lfs update --manual`"), "no advice that cannot work is offered: {notice}");

    let (with_a_caller_setting, setting_notice) = with_skip_repo(words(&["-c", "lfs.skipdownloaderrors=true", "lfs", "install", "--force"]), true);
    assert_eq!(with_a_caller_setting, words(&["-c", "lfs.skipdownloaderrors=true", "lfs", "install", SKIP_REPO_OPTION, "--force"]));
    assert!(!setting_notice.is_empty());

    for untouched in [
        vec!["lfs", "install"],
        vec!["lfs", "install", "--skip-smudge"],
        vec!["lfs", "install", "--help"],
        vec!["lfs", "install", "-h"],
        vec!["lfs", "install", "--help=true"],
        vec!["lfs", "install", "-lh"],
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
///  `--worktree` writes `config.worktree` in the worktree's Git directory while `extensions.worktreeConfig` is set
///  (`<repo>/.git/config.worktree` in the main worktree and `<repo>/.git/worktrees/<name>/config.worktree` in a
///  linked one),
///  `<repo>/.git/config` when it is not and the repository holds one worktree,
///  and nothing at all when it is not and the repository holds several, where Git refuses the write and git-lfs ends
///  with status 2,
///  `--file=<path>` writes that file,
///  and the global file stays empty in all three;
///  the notice said "the global `filter.lfs.*` keys were written as usual" for every one of them,
///  which is false for three.
///  A boolean option's `=value` is read too: `--local=true` names the repository's file,
///  and a false value resets the scope that option had set;
///  the short form reads it as pflag does, so `-l=false` withdraws `--local` and `-lw=false` sets `--local` while
///  withdrawing `--worktree`.
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
        ("--worktree names the worktree file and its condition", vec!["lfs", "install", "--worktree"], "`config.worktree` in this worktree's Git directory"),
        ("--system names the system file", vec!["lfs", "install", "--system"], "system configuration"),
        ("--file names the file it was given", vec!["lfs", "install", "--file=/work/elsewhere.conf"], "`/work/elsewhere.conf`"),
        ("--file's value may be the next word", vec!["lfs", "install", "--file", "elsewhere.conf"], "`elsewhere.conf`"),
        (
            "--file takes a flag-like next word as its value",
            vec!["lfs", "install", "--file", "--local"],
            "keys in `--local`",
        ),
        ("--local=true names the repository's own file", vec!["lfs", "install", "--local=true"], "this repository's `.git/config`"),
        ("--worktree=false leaves the global file", vec!["lfs", "install", "--worktree=false"], "global configuration"),
        ("--system=true names the system file", vec!["lfs", "install", "--system=true"], "system configuration"),
        ("a later false value resets --local", vec!["lfs", "install", "--local", "--local=false"], "global configuration"),
        ("a later false value resets --system", vec!["lfs", "install", "--system", "--system=false"], "global configuration"),
        ("a false --worktree leaves the earlier --local", vec!["lfs", "install", "--local", "--worktree=false"], "this repository's `.git/config`"),
        ("--file= resets the file scope", vec!["lfs", "install", "--file=/work/elsewhere.conf", "--file="], "global configuration"),
        ("-l=false withdraws --local", vec!["lfs", "install", "-l=false"], "global configuration"),
        ("-l=0 withdraws --local", vec!["lfs", "install", "-l=0"], "global configuration"),
        ("-l=true keeps --local", vec!["lfs", "install", "-l=true"], "this repository's `.git/config`"),
        ("-w=0 withdraws --worktree", vec!["lfs", "install", "-w=0"], "global configuration"),
        ("-lw=false sets --local and withdraws --worktree", vec!["lfs", "install", "-lw=false"], "this repository's `.git/config`"),
        ("-lm=false asks for no hook instructions", vec!["lfs", "install", "-lm=false"], "this repository's `.git/config`"),
        ("-m=false names no scope of its own", vec!["lfs", "install", "-m=false"], "global configuration"),
        ("-l=false before -w leaves the worktree scope", vec!["lfs", "install", "-l=false", "-w"], "`config.worktree` in this worktree's Git directory"),
    ] {
        let (_, notice) = with_skip_repo(words(typed.as_slice()), true);
        assert!(notice.contains(wanted), "{case}: the notice names the scope: {notice}");
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
    for off in [
        vec!["lfs", "install", "--manual=false"],
        vec!["lfs", "install", "--manual", "--manual=false"],
        vec!["lfs", "install", "-m=false"],
        vec!["lfs", "install", "-lm=false"],
    ] {
        let (_, silent) = with_skip_repo(words(off.as_slice()), true);
        assert!(!silent.contains("none are printed here"), "{off:?} asks for no hook instructions: {silent}");
    }
}

/// What:
///  A command git-lfs refuses before it writes a `filter.lfs.*` key is told what it refuses and why,
///  and the notice names no destination and no write for it,
///  because the words alone show that nothing reaches one.
/// Why:
///   Measured with git-lfs 3.8.1 through the wrapper, the global file and the repository's `.git/config` read before and
///  after each run:
///  two destination flags (`-l --worktree`, `-lw`, `--local --system`, `-lw --system`) end with status 2 and
///  "Only one of the --local, --system, --worktree, and --file options can be specified.",
///  a word pflag cannot read as a boolean (`-l=yes`, `-l=`, `-w=`, `--local=yes`, `--worktree=maybe`, `--manual=yes`,
///  `--force=yes`) ends with status 127 and the parser's own error,
///  and `--file` with no word after it ends with status 127 and "flag needs an argument: --file";
///  none of them wrote a `filter.lfs.*` key anywhere,
///  and the repository's `.git/config` gained only the `lfs.repositoryformatversion = 0` git-lfs records while setting
///  up a `--local` or `--worktree` scope, before it refuses,
///  while the notice named a destination in every one of those states.
///  A false value still chooses no second destination, so `--local --worktree=false` and `-l=false -w` are run,
///  as measured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(refusedNotice).toContain('git-lfs refuses this command'); expect(refusedNotice).not.toContain('asks git-lfs to put')
/// ```
#[test]
fn the_notice_names_the_refusal_instead_of_a_destination() {
    for (case, typed, wanted) in [
        (
            "two flags -l --worktree",
            vec!["lfs", "install", "-l", "--worktree"],
            "only one of `--local`, `--system`, `--worktree` and `--file` can be specified, and this command asks for more than one",
        ),
        ("a short group -lw", vec!["lfs", "install", "-lw"], "more than one"),
        ("--local --system", vec!["lfs", "install", "--local", "--system"], "more than one"),
        ("three flags -lw --system", vec!["lfs", "install", "-lw", "--system"], "more than one"),
        ("a rejected short value", vec!["lfs", "install", "-l=yes"], "`-l=yes`"),
        ("a short group that ends in =", vec!["lfs", "install", "-l="], "`-l=`"),
        ("a short group ending in = on --worktree", vec!["lfs", "install", "-w="], "`-w=`"),
        ("a rejected long value", vec!["lfs", "install", "--local=yes"], "`--local=yes`"),
        ("a rejected value on a boolean that chooses no scope", vec!["lfs", "install", "--force=yes"], "`--force=yes`"),
        ("a rejected value on a short boolean that chooses no scope", vec!["lfs", "install", "-f=yes"], "`-f=yes`"),
        ("--file with no path, beside a destination flag", vec!["lfs", "install", "--local", "--file"], "`--file` needs a path and none followed it"),
        ("a --file path counts as a destination beside --local", vec!["lfs", "install", "--file=elsewhere.conf", "--local"], "more than one"),
    ] {
        let (_, notice) = with_skip_repo(words(typed.as_slice()), true);
        assert!(notice.contains("git-lfs refuses this command"), "{case}: the notice says git-lfs refuses it: {notice}");
        assert!(notice.contains(wanted), "{case}: the notice names the refusal: {notice}");
        assert!(!notice.contains("asks git-lfs to put"), "{case}: the notice names no destination: {notice}");
        assert!(
            notice.contains("so no `filter.lfs.*` key is written to any configuration file"),
            "{case}: the notice claims no write: {notice}"
        );
    }
    for (case, typed) in [
        ("a false second destination is no second destination", vec!["lfs", "install", "--local", "--worktree=false"]),
        ("a withdrawn flag before another leaves one destination", vec!["lfs", "install", "-l=false", "-w"]),
        ("a withdrawn flag alone leaves the global file", vec!["lfs", "install", "--local", "--local=false"]),
        ("a group whose value withdraws one flag leaves one destination", vec!["lfs", "install", "-lw=false"]),
        ("short booleans of their own change no destination", vec!["lfs", "install", "-fs"]),
        ("a word after -- is no option", vec!["lfs", "install", "--", "-lw"]),
    ] {
        let (_, notice) = with_skip_repo(words(typed.as_slice()), true);
        assert!(!notice.contains("git-lfs refuses this command"), "{case}: git-lfs runs it: {notice}");
    }
    let (_, refused_manual) = with_skip_repo(words(&["lfs", "install", "-lw", "--manual"]), true);
    assert!(refused_manual.contains("git-lfs refuses this command"), "a refused command is told so: {refused_manual}");
    assert!(
        !refused_manual.contains("none are printed here"),
        "a refused command prints no hook instructions because it is refused, not because `--skip-repo` skipped them: {refused_manual}"
    );
    // An option this file does not read is left to git-lfs rather than modelled, so the destination sentence stands for it:
    //  measured, `git lfs install --bogus` ends with status 127 and "Error: unknown flag: --bogus" and writes nothing.
    //  A list of every option git-lfs registers would go stale the day git-lfs adds one, and that would claim a refusal
    //  git-lfs does not make; git-lfs's own message answers an option it does not know.
    let (_, unread_option) = with_skip_repo(words(&["lfs", "install", "--bogus"]), true);
    assert!(
        unread_option.contains("This command asks git-lfs to put the `filter.lfs.*` keys in"),
        "an option this reading does not name keeps the destination its words select: {unread_option}"
    );
    let (_, unread_letter) = with_skip_repo(words(&["lfs", "install", "-z"]), true);
    assert!(
        unread_letter.contains("This command asks git-lfs to put the `filter.lfs.*` keys in your global configuration"),
        "an unknown short letter is passed through like an unknown long option: {unread_letter}"
    );
}

/// What:
///  The `--worktree` notice states each outcome Git chooses,
///  because the wrapper asks Git nothing and cannot know which one happens:
///  the worktree's own `config.worktree` while the extension is set,
///  this repository's `.git/config` when it is not and there is one worktree,
///  and no file at all when it is not and there are several, where Git refuses and git-lfs ends with status 2.
/// Why:
///   Measured with git-lfs 3.8.1 through the wrapper and plain `git` 2.56.1,
///  every file read with `git config --file <p> --list`:
///  with the extension set, a linked worktree's keys landed in
///  `<repo>/.git/worktrees/<name>/config.worktree`, not in the `.git/config.worktree` beside this repository's
///  `.git/config` the notice printed;
///  with two worktrees and the extension unset, `git-lfs install --worktree --skip-repo` exited 2 and wrote nothing,
///  while the notice promised the keys were written to a file.
///  The worktree name in a linked checkout is in that Git directory and not in `.git` of the checkout, where `.git` is
///  a file;
///  `binary_lfs_install_tests.rs` pins all four layouts against Git's own `git config --worktree`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(worktreeNotice).toContain('no file at all when it is not and the repository holds several')
/// ```
#[test]
fn the_worktree_notice_states_every_outcome_git_chooses() {
    let (_, notice) = with_skip_repo(words(&["lfs", "install", "--worktree"]), true);
    for (outcome, wanted) in [
        (
            "the extension is set, in this worktree's Git directory",
            "`config.worktree` in this worktree's Git directory) while `extensions.worktreeConfig` is set",
        ),
        (
            "the extension is unset and one worktree exists",
            "this repository's `.git/config` when it is not and the repository holds one worktree",
        ),
        (
            "the extension is unset and several worktrees exist",
            "no file at all when it is not and the repository holds several",
        ),
        ("Git refuses that write and git-lfs ends with status 2", "Git refuses the write and git-lfs ends with status 2"),
    ] {
        assert!(notice.contains(wanted), "{outcome}: the notice states it: {notice}");
    }
    assert!(
        !notice.contains("`.git/config.worktree`"),
        "a linked worktree's file is under that worktree's Git directory, so the notice names no path in `.git` that is not a file there: {notice}"
    );
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

/// What:
///  A help query is answered as it was typed and gets no notice,
///  however the caller spells it,
///  while a word that only looks like one is read as pflag reads it.
/// Why:
///   cobra registers `--help` with shorthand `h` as a boolean (`command.go:1219-1230` of cobra v1.10.2)
///  and reads every flag before it looks at that flag (`:919-936`),
///  so `--help=true` and a group such as `-lh` print help with status 0 and write no key,
///  measured through the wrapper with git-lfs 3.8.1,
///  and a second `--skip-repo` or a destination sentence there would answer a command that writes nothing.
///  `--help=false` and `-lh=false` run the command, a parse error beats help,
///  and a word after `--` is no help query at all:
///  `git lfs install -- -h` ran into the hook write the wrapper's own option prevents, with status 2.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(withSkipRepo(words('lfs install -lh'), true)).toEqual([words('lfs install -lh'), ''])
/// ```
#[test]
fn a_help_query_is_answered_as_it_was_typed() {
    for typed in [
        vec!["lfs", "install", "--help"],
        vec!["lfs", "install", "-h"],
        vec!["lfs", "install", "--help=true"],
        vec!["lfs", "install", "-lh"],
        vec!["lfs", "install", "-lh=true"],
        vec!["lfs", "install", "--local", "-h"],
        vec!["lfs", "install", "-h", "--"],
    ] {
        let (same, silent) = with_skip_repo(words(typed.as_slice()), true);
        assert_eq!(same, words(typed.as_slice()), "{typed:?}: a help query is answered as it was typed");
        assert!(silent.is_empty(), "{typed:?}: nothing is said about a command that writes no key");
    }
    for typed in [vec!["lfs", "install", "--help=false"], vec!["lfs", "install", "-lh=false"]] {
        let (rewritten, notice) = with_skip_repo(words(typed.as_slice()), true);
        assert_eq!(
            rewritten,
            words(&["lfs", "install", SKIP_REPO_OPTION, typed[2]]),
            "{typed:?}: a false help value runs the command, so the wrapper adds its option"
        );
        assert!(notice.contains("asks git-lfs to put the `filter.lfs.*` keys in"), "{typed:?}: the notice names the request: {notice}");
    }
    let (positional, positional_notice) = with_skip_repo(words(&["lfs", "install", "--", "-h"]), true);
    assert_eq!(
        positional,
        words(&["lfs", "install", SKIP_REPO_OPTION, "--", "-h"]),
        "a word after `--` is no help query, and the wrapper's option still goes right after `install`"
    );
    assert!(positional_notice.contains("your global configuration"), "the positional word asks for no destination: {positional_notice}");
    let (_, refused_help) = with_skip_repo(words(&["lfs", "install", "-lh", "--local=yes"]), true);
    assert!(
        refused_help.contains("git-lfs refuses this command, because its flag parser rejects `--local=yes`"),
        "a parse error beats help, as cobra reads them: {refused_help}"
    );
}

/// What:
///  A `--skip-repo` after `--` is a positional word and not the option,
///  so the wrapper adds its own copy rather than trusting that word.
/// Why:
///   pflag stops reading options at `--` (`flag.go:1141-1145` of pflag v1.0.9),
///  so git-lfs never saw that word as the option either:
///  measured through the wrapper, `git lfs install -- --skip-repo` ran into the hook write with status 2 and
///  `mkdir /dev/null: not a directory`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(withSkipRepo(words('lfs install -- --skip-repo'), true)[0]).toEqual(words('lfs install --skip-repo -- --skip-repo'))
/// ```
#[test]
fn a_word_after_the_double_dash_is_no_option() {
    let (rewritten, notice) = with_skip_repo(words(&["lfs", "install", "--", SKIP_REPO_OPTION]), true);
    assert_eq!(
        rewritten,
        words(&["lfs", "install", SKIP_REPO_OPTION, "--", SKIP_REPO_OPTION]),
        "the positional word is not the option, so the wrapper adds one"
    );
    assert!(notice.contains("your global configuration"), "the positional word asks for no destination: {notice}");
}

/// What:
///  A caller's own `--skip-repo=false` is answered by the wrapper's copy right after it,
///  and the notice says the caller's word did not win.
/// Why:
///   pflag sets a flag again for each word that names it (`flag.go:484-517` of pflag v1.0.9), and the last value wins,
///  so the copy the wrapper used to put right after `install` was undone by that later word:
///  measured through the wrapper, `git lfs install --skip-repo=false` wrote the global keys and then ended with status 2 and
///  `mkdir /dev/null: not a directory`,
///  the failure the option exists to prevent.
///  The hooks cannot be installed under the wrapper at all,
///  because the directory git-lfs writes them into is disabled,
///  so the copy follows the word and the notice says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(withSkipRepo(words('lfs install --skip-repo=false'), true)[0]).toEqual(words('lfs install --skip-repo=false --skip-repo'))
/// ```
#[test]
fn a_caller_who_turned_skip_repo_off_gets_the_wrappers_copy_after_it() {
    for spelled in [String::from("--skip-repo=false"), String::from("--skip-repo=0"), String::from("--skip-repo=FALSE")] {
        let typed: Vec<&str> = vec!["lfs", "install", spelled.as_str()];
        let (rewritten, notice) = with_skip_repo(words(typed.as_slice()), true);
        assert_eq!(
            rewritten,
            words(&["lfs", "install", spelled.as_str(), SKIP_REPO_OPTION]),
            "{spelled}: the wrapper's copy follows the word that would undo it"
        );
        assert!(
            notice.contains("The last word that set `--skip-repo` turned it off"),
            "{spelled}: the notice says which word won: {notice}"
        );
        assert!(
            notice.contains("asks git-lfs to put the `filter.lfs.*` keys in your global configuration"),
            "{spelled}: the destination is unchanged by that word: {notice}"
        );
    }
    let (kept, kept_notice) = with_skip_repo(words(&["lfs", "install", "--skip-repo=false", SKIP_REPO_OPTION]), true);
    assert_eq!(
        kept,
        words(&["lfs", "install", "--skip-repo=false", SKIP_REPO_OPTION]),
        "the caller's own copy stands when its value is already on"
    );
    assert!(!kept_notice.contains("turned it off"), "nothing was turned off: {kept_notice}");
}

/// What:
///  A refusal is told the reason git-lfs acts on,
///  and no reason at all when an option this reading does not name came before it.
/// Why:
///   pflag stops at the first word it cannot read (`flag.go:1153-1155` of pflag v1.0.9),
///  and that word may be one this reading passes through,
///  so the word this reading refuses is not the word git-lfs refuses:
///  measured through the wrapper, `git lfs install -z -l=yes` answered `unknown shorthand flag: 'z' in -z`
///  while the notice named `-l=yes`,
///  and the same happened for `--bogus --file` and `-lw --bogus`.
///  A word git-lfs reaches is still named, and no `filter.lfs.*` key is written in either case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(withSkipRepo(words('lfs install -z -l=yes'), true)[1]).not.toContain('because')
/// ```
#[test]
fn a_refusal_after_an_option_the_reading_does_not_name_states_no_reason() {
    for typed in [
        vec!["lfs", "install", "-z", "-l=yes"],
        vec!["lfs", "install", "--bogus", "--file"],
        vec!["lfs", "install", "--bogus", "--local=yes"],
        vec!["lfs", "install", "-zl=yes"],
        vec!["lfs", "install", "-lw", "--bogus"],
    ] {
        let (_, notice) = with_skip_repo(words(typed.as_slice()), true);
        assert!(notice.contains("git-lfs refuses this command, so no `filter.lfs.*` key is written"), "{typed:?}: {notice}");
        assert!(!notice.contains("because"), "{typed:?}: git-lfs stops at a word this reading cannot name: {notice}");
    }
    for (case, typed, wanted) in [
        ("a word git-lfs reaches is named", vec!["lfs", "install", "--local=yes", "--bogus"], "because its flag parser rejects `--local=yes`"),
        ("a short group's bad value", vec!["lfs", "install", "-f=yes"], "because its flag parser rejects `-f=yes`"),
        ("the count is the reason when nothing was passed through", vec!["lfs", "install", "-lw"], "because only one of"),
        ("a file without a path is the reason", vec!["lfs", "install", "--file"], "because `--file` needs a path"),
    ] {
        let (_, notice) = with_skip_repo(words(typed.as_slice()), true);
        assert!(notice.contains(wanted), "{case}: {notice}");
    }
}
