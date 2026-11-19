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
//! (`package/git-policy/cli/SPEC.md`, "Git LFS upload before a push").
//!  So the command the person typed runs with `--skip-repo`,
//!  which git-lfs's own help describes as "Skips installation of hooks into the local repository;
//!  use if you want to install the LFS filters but not make changes to the hooks",
//!  and the wrapper says in one notice what the person would otherwise have to work out:
//!  the filters were written and no hook is needed here.
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
///  The command with `--skip-repo` after its `install` word,
///  and the notice that says what that means,
///  or the command as it stands and no notice when it is not `git lfs install`.
/// Why:
///   A command outside a repository writes no hook,
///  so it needs no rewrite and no notice about hooks;
///  a help query answers what was asked and changes nothing;
///  and a caller who wrote the option already is told the same thing anyway,
///  because the reason is the wrapper's and not the caller's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (words.slice(installIndex, installIndex + 2).join(' ') !== 'lfs install') return [args, '']; return [inserted, notice];
/// ```
pub fn with_skip_repo(arguments: Vec<OsString>, in_a_repository: bool) -> (Vec<OsString>, String) {
    if !in_a_repository {
        return (arguments, String::new());
    }
    let Some(install_index) = install_word(&arguments) else {
        return (arguments, String::new());
    };
    let already: bool = arguments.iter().skip(install_index + 1).any(|word| { return word == SKIP_REPO_OPTION; });
    if already {
        return (arguments, notice());
    }
    let mut rewritten: Vec<OsString> = arguments;
    rewritten.insert(install_index + 1, OsString::from(SKIP_REPO_OPTION));
    return (rewritten, notice());
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

/// What:
///  What a person is told when the wrapper added `--skip-repo` to their `git lfs install`.
/// Why:
///   The command then says only "Git LFS initialized.",
///  which does not say that no hook was installed or that none is needed,
///  and the person who ran it in a repository is the one who would wonder.
///  It names the keys that were written,
///  the hooks that were not,
///  and the step that replaces them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// 'monochromatic-git-policy-cli: `git lfs install` ran with `--skip-repo` ...'
/// ```
pub fn notice() -> String {
    return String::from(
        "monochromatic-git-policy-cli: `git lfs install` ran with `--skip-repo`, which the wrapper adds. The global \
         `filter.lfs.*` keys were written as usual. The wrapper does not run traditional hooks, and it uploads the Git LFS \
         objects itself before each push, so no `pre-push` hook is installed in this repository and none is needed; the \
         `post-checkout`, `post-commit` and `post-merge` hooks it would install are not needed here either. Without the \
         option git-lfs tries to write its hooks into the directory the wrapper disables, which fails and ends the command \
         with status 2 and advice that cannot work.\n",
    );
}

/// The rewrite is controlled in the test module.
#[cfg(test)]
#[path = "lfs_install_arguments_tests.rs"]
mod tests;
