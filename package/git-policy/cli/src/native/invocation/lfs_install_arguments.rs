//! What:
//!  The `--skip-repo` the wrapper adds to `git lfs install` inside a repository,
//!  and the notice that says why.
//! Why:
//!  The wrapper starts every real Git with traditional hooks switched off
//! (`core.hooksPath` below `/dev/null` in `hooks_off.rs`),
//!  and `git lfs install` writes its four shims into that directory when it runs inside a repository:
//!  `mkdir /dev/null: not a directory`,
//!  status 2,
//!  and advice to run `git lfs update --manual` or `--force`,
//!  neither of which can work while the directory cannot exist.
//!  The global `filter.lfs.*` keys are written first and are what the wrapper's upload needs,
//!  and no hook is needed at all:
//!  the wrapper runs the upload itself before every push
//! (`package/git-policy/cli/SPEC.md`,
//! "Git LFS upload before a push").
//!  So the command the person typed runs with `--skip-repo`,
//!  which git-lfs's own help describes as "Skips installation of hooks into the local repository;
//!  use if you want to install the LFS filters but not make changes to the hooks",
//!  and the wrapper says in one notice what the person would otherwise have to work out:
//!  where this command's filters go and that no hook is needed here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [args, notice] = withSkipRepo(stripped.arguments, inRepository); // `git lfs install --skip-repo`
//! ```

/// The global-option layout,
/// to find the command word behind the caller's global options.
use crate::global_arguments::{GlobalOutcome, global_layout};
/// Owned operating-system text for the arguments.
use std::ffi::OsString;

/// The option git-lfs's `install` takes to write the filters and no hook.
pub const SKIP_REPO_OPTION: &str = "--skip-repo";

/// What:
///  The boolean one pflag value spells,
///  or nothing when it is not one.
/// Why:
///  git-lfs's `--local`, `--worktree`, `--system` and `--manual` are pflag booleans,
///  so `--local=true` and `--worktree=false` are read as its flag parser reads them.
///  The spellings are `strconv.ParseBool`'s, which pflag uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function booleanValue(text: string): boolean | undefined
/// ```
fn boolean_value(text: &str) -> Option<bool> {
    const TRUE_VALUES: [&str; 6] = ["1", "t", "T", "TRUE", "true", "True"];
    const FALSE_VALUES: [&str; 6] = ["0", "f", "F", "FALSE", "false", "False"];
    if TRUE_VALUES.contains(&text) {
        return Some(true);
    }
    if FALSE_VALUES.contains(&text) {
        return Some(false);
    }
    return None;
}

/// The value of one `--name=value` boolean option, or nothing for another word.
fn long_boolean(text: &str, option: &str) -> Option<bool> {
    let value: &str = text.strip_prefix(option)?.strip_prefix('=')?;
    return boolean_value(value);
}

/// What:
///  The booleans one group of short options sets,
///  in the order pflag reads them.
/// Why:
///   pflag reads a group letter by letter,
///  and a `=` after a letter gives that letter its value and ends the group:
///  `-l=false` withdraws `--local`,
///  and `-lw=false` sets `--local` while withdrawing `--worktree`.
///  A value no pflag boolean spells sets nothing, as `boolean_value` reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shortBooleanSettings(letters: string): Array<[letter: string, value: boolean]>
/// ```
fn short_boolean_settings(letters: &str) -> Vec<(char, bool)> {
    let mut settings: Vec<(char, bool)> = Vec::new();
    let mut remaining: &str = letters;
    while let Some(letter) = remaining.chars().next() {
        let after: &str = remaining.strip_prefix(letter).unwrap_or_default();
        if let Some(text) = after.strip_prefix('=') {
            if let Some(value) = boolean_value(text) {
                settings.push((letter, value));
            }
            return settings;
        }
        settings.push((letter, true));
        remaining = after;
    }
    return settings;
}

/// What:
///  The command with `--skip-repo` after its `install` word,
///  and the notice that says what that means,
///  or the command as it stands and no notice when it is not `git lfs install`.
/// Why:
///   A command outside a repository writes no hook,
///  so it needs no rewrite and no notice about hooks;
///  a help query answers what was asked and changes nothing;
///  and a caller who wrote the option already is told the same thing anyway,
///  because the reason is the wrapper's and not the caller's.
///  The notice is built from the words the caller typed,
/// so it names the configuration file git-lfs writes:
///  `--local`,
/// `--worktree`,
/// `--system` and `--file` each send the keys somewhere other than the global file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (words.slice(installIndex, installIndex + 2).join(' ') !== 'lfs install') return [args, '']; return [inserted, notice(scopeOf(args))];
/// ```
pub fn with_skip_repo(arguments: Vec<OsString>, in_a_repository: bool) -> (Vec<OsString>, String) {
    if !in_a_repository {
        return (arguments, String::new());
    }
    let Some(install_index) = install_word(&arguments) else {
        return (arguments, String::new());
    };
    let scope: InstallScope = InstallScope::of(arguments.as_slice(), install_index);
    let note: String = notice(scope, InstallScope::asked_for_manual(arguments.as_slice(), install_index));
    let already: bool = arguments.iter().skip(install_index + 1).any(|word| { return word == SKIP_REPO_OPTION; });
    if already {
        return (arguments, note);
    }
    let mut rewritten: Vec<OsString> = arguments;
    rewritten.insert(install_index + 1, OsString::from(SKIP_REPO_OPTION));
    return (rewritten, note);
}

/// What:
///  Where the `filter.lfs.*` keys of one `git lfs install` are written,
///  as git-lfs's `install` flags choose it.
/// Why:
///  git-lfs 3.8.1 writes the keys to one of five places (`commands/command_install.go:40-58` refuses more than one):
///  the global file by default,
///  `--local` this repository's `.git/config`,
///  `--worktree` this worktree's configuration while Git's `extensions.worktreeConfig` is set,
///  this repository's `.git/config` when it is not and the repository holds one worktree,
///  and nothing at all when it is not and the repository holds several (Git refuses that write and git-lfs ends with
///  status 2),
///  `--system` the system file,
///  and `--file <path>` that file.
///  The scope is read from the words so the notice can name where the keys go,
///  or, for `--worktree`, every place Git may put them,
///  rather than calling every scope global.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallScope = { kind: 'global' } | { kind: 'local' } | { kind: 'worktree' } | { kind: 'system' } | { kind: 'file', path: string }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
enum InstallScope {
    /// The global file git-lfs writes when no scope flag is given.
    Global,
    /// This repository's `.git/config`.
    Local,
    /// This worktree's configuration.
    Worktree,
    /// The system-wide configuration.
    System,
    /// The file a `--file` names.
    File(String),
}

impl InstallScope {
    /// What:
    ///  The scope one `git lfs install` asks for,
    ///  read from the words after its `install` word.
    /// Why:
    ///   git-lfs's own flags are the only source of this answer,
    ///  and `--` ends them,
    ///  so nothing after it is read as a flag.
    ///  `-l` and `-w` are the scope short forms git-lfs registers;
    ///  `-f` is `--force`,
    /// `-s` is `--skip-smudge` and `-m` is `--manual`,
    ///  so none of them chooses a scope,
    ///  and a group of short forms (`-lw`) is read letter by letter as Git's flag parser does.
    ///  A long option's `=value` is read as pflag reads it:
    ///  `--local=true` asks for that scope and `--worktree=false` withdraws it,
    ///  with the last value of one option winning.
    ///  A short form's `=value` is read as pflag reads it too:
    ///  `-l=false` withdraws `--local` and `-lw=false` sets `--local` while withdrawing `--worktree`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const word of words.slice(install + 1)) { if (word === '--') break; /* ... */ }
    /// ```
    fn of(arguments: &[OsString], install_index: usize) -> InstallScope {
        let mut local: bool = false;
        let mut worktree: bool = false;
        let mut system: bool = false;
        let mut file: Option<String> = None;
        let mut words = arguments.iter().skip(install_index + 1);
        while let Some(word) = words.next() {
            let text: String = word.to_string_lossy().into_owned();
            if text == "--" {
                break;
            }
            if let Some(path) = text.strip_prefix("--file=") {
                file = (!path.is_empty()).then(|| { return path.to_owned(); });
            } else if text == "--file" {
                if let Some(next) = words.next() {
                    let path: String = next.to_string_lossy().into_owned();
                    file = (!path.is_empty()).then_some(path);
                }
            } else if let Some(value) = long_boolean(text.as_str(), "--local") {
                local = value;
            } else if text == "--local" {
                local = true;
            } else if let Some(value) = long_boolean(text.as_str(), "--worktree") {
                worktree = value;
            } else if text == "--worktree" {
                worktree = true;
            } else if let Some(value) = long_boolean(text.as_str(), "--system") {
                system = value;
            } else if text == "--system" {
                system = true;
            } else if let Some(letters) = text.strip_prefix('-').filter(|letters| { return !letters.starts_with('-'); }) {
                for (letter, value) in short_boolean_settings(letters) {
                    match letter {
                        'l' => local = value,
                        'w' => worktree = value,
                        _other => {}
                    }
                }
            }
        }
        if let Some(path) = file {
            return InstallScope::File(path);
        }
        if worktree {
            return InstallScope::Worktree;
        }
        if local {
            return InstallScope::Local;
        }
        if system {
            return InstallScope::System;
        }
        return InstallScope::Global;
    }

    /// What:
    ///  Whether the caller asked git-lfs for its hook instructions with `--manual`.
    /// Why:
    ///   `--manual` changes only the hook step,
    /// which `--skip-repo` skips,
    ///  so the notice has to say why the instructions the person asked for are missing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// words.slice(install + 1).some((word) => word === '--manual' || /^-[a-z]*m[a-z]*$/.test(word))
    /// ```
    fn asked_for_manual(arguments: &[OsString], install_index: usize) -> bool {
        let mut manual: bool = false;
        for word in arguments.iter().skip(install_index + 1) {
            let text: String = word.to_string_lossy().into_owned();
            if text == "--" {
                break;
            }
            if let Some(value) = long_boolean(text.as_str(), "--manual") {
                manual = value;
            } else if text == "--manual" {
                manual = true;
            } else if let Some(letters) = text.strip_prefix('-').filter(|letters| { return !letters.starts_with('-'); }) {
                for (letter, value) in short_boolean_settings(letters) {
                    if letter == 'm' {
                        manual = value;
                    }
                }
            }
        }
        return manual;
    }

    /// What:
    ///  Where this scope writes the keys,
    ///  as a person reads it.
    /// Why:
    ///   The notice names the file the keys go to,
    ///  so a person who reads their global file and finds nothing knows where to look.
    ///  `--worktree` is the one scope the wrapper cannot answer without asking Git,
    ///  and it asks nothing on every `git lfs install`:
    ///  so it names each outcome Git chooses instead,
    ///  including the one where Git refuses and writes no file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function where(scope: InstallScope): string
    /// ```
    fn where_it_writes(&self) -> String {
        return match self {
            InstallScope::Global => String::from("your global configuration"),
            InstallScope::Local => String::from("this repository's `.git/config`"),
            InstallScope::Worktree => String::from(
                "this worktree's configuration (`config.worktree` in this worktree's Git directory) while \
                 `extensions.worktreeConfig` is set, this repository's `.git/config` when it is not and the repository \
                 holds one worktree, or no file at all when it is not and the repository holds several, where Git \
                 refuses the write and git-lfs ends with status 2",
            ),
            InstallScope::System => String::from("the system configuration"),
            InstallScope::File(path) => format!("`{path}`"),
        };
    }
}

/// What:
///  What a person is told when the wrapper added `--skip-repo` to their `git lfs install`.
/// Why:
///   The command then says only "Git LFS initialized.",
///  which does not say that no hook was installed or that none is needed,
///  and the person who ran it in a repository is the one who would wonder.
///  It names where the keys go,
///  the hooks that were not,
///  and the step that replaces them;
///  and a run with `--manual` says why the instructions that option asks for are not printed:
///  they are instructions for merging a hook this wrapper installs nowhere and never runs.
///  `manual` is whether the caller asked for those instructions with `--manual`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: \`git lfs install\` ran with \`--skip-repo\` ... The \`filter.lfs.*\` keys go,
/// as this command asked, to ${where} ...`
/// ```
fn notice(scope: InstallScope, manual: bool) -> String {
    let mut text: String = format!(
        "monochromatic-git-policy-cli: `git lfs install` ran with `--skip-repo`, which the wrapper adds. The `filter.lfs.*` \
         keys go, as this command asked, to {}. The wrapper does not run traditional hooks, \
         and it uploads the Git LFS objects itself before each push, so no `pre-push` hook is installed in this repository and \
         none is needed; the `post-checkout`, `post-commit` and `post-merge` hooks it would install are not needed here \
         either. Without the option git-lfs tries to write its hooks into the directory the wrapper disables, which fails \
         and ends the command with status 2 and advice that cannot work.\n",
        scope.where_it_writes()
    );
    if manual {
        text.push_str(manual_notice().as_str());
    }
    return text;
}

/// What:
///  What a run with `--manual` is told about the hook instructions it asked for.
/// Why:
///   `--manual` makes git-lfs print instructions for merging its hooks into this repository by hand,
///  and that printing happens in the step `--skip-repo` skips (`commands/command_install.go:30-32`),
///  so the person who asked for them gets none and is owed the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// 'monochromatic-git-policy-cli: `--manual` asks git-lfs for the instructions that merge its hooks by hand; none are printed ...'
/// ```
fn manual_notice() -> String {
    return String::from(
        "monochromatic-git-policy-cli: `--manual` asks git-lfs for the instructions that merge its hooks into this \
         repository by hand, and none are printed here: they are instructions for a hook the wrapper installs nowhere and \
         never runs.\n",
    );
}

/// What:
///  The index of the `install` word of a `git lfs install` command,
///  or nothing for every other command.
/// Why:
///   The command word and its subcommand are read through the same layout the rest of the wrapper reads,
///  so a global option before `lfs` cannot hide it;
///  a help query is answered as it was typed,
///  and `git lfs install hooks` is left alone,
///  because the option changes nothing for that form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = globalLayout(args); if (layout.outcome !== 'command') return undefined; if (args[layout.prefixLen] !== 'lfs') return undefined; ...
/// ```
fn install_word(arguments: &[OsString]) -> Option<usize> {
    let layout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command {
        return None;
    }
    let command: usize = layout.prefix_len;
    if arguments.get(command)? != "lfs" {
        return None;
    }
    let install: usize = command + 1;
    if arguments.get(install)? != "install" {
        return None;
    }
    for word in arguments.iter().skip(install + 1) {
        if word == "--help" || word == "-h" {
            return None;
        }
    }
    if arguments.get(install + 1).is_some_and(|word| { return word == "hooks"; }) {
        return None;
    }
    return Some(install);
}

/// The rewrite is controlled in the test module.
#[cfg(test)]
#[path = "lfs_install_arguments_tests.rs"]
mod tests;
