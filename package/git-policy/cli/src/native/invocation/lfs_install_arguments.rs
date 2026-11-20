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
//!  where this command asks git-lfs to put its filters,
//! or why git-lfs refuses the command before it writes a key,
//!  and that no hook is needed here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [args, notice] = withSkipRepo(stripped.arguments, inRepository); // `git lfs install --skip-repo`
//! ```

/// The global-option layout,
/// to find the command word behind the caller's global options.
use crate::global_arguments::{GlobalOutcome, global_layout};
/// The request one `git lfs install` makes,
/// read from the words after its `install` word.
use super::lfs_install_request::{InstallAsk, InstallRequest};
/// Owned operating-system text for the arguments.
use std::ffi::OsString;

/// The option git-lfs's `install` takes to write the filters and no hook.
pub const SKIP_REPO_OPTION: &str = "--skip-repo";

/// What:
///  The command with `--skip-repo` after the word the caller left it off at,
///  and the notice that says what that means,
///  or the command as it stands and no notice when it is not `git lfs install`.
/// Why:
///   A command outside a repository writes no hook,
///  so it needs no rewrite and no notice about hooks;
///  a help query answers what was asked and changes nothing;
///  and a caller who wrote the option already,
/// with a true value,
/// is told the same thing anyway,
///  because the reason is the wrapper's and not the caller's.
///  The notice is built from the words the caller typed,
///  so it names the destination this command asks git-lfs for,
/// and for `--worktree` every place Git may put the keys,
///  rather than calling every scope global;
///  and it states that request and not a write,
///  because git-lfs can stop after it reads the words well,
/// and the sentence would then be false.
///  A command whose words git-lfs refuses before it writes a key is told that refusal instead,
///  because the notice names no destination that nothing reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (words.slice(installIndex, installIndex + 2).join(' ') !== 'lfs install') return [args, '']; const request = read(args); if (request.asksForHelp()) return [args, '']; const at = request.skipRepoInsertion(installIndex); return [at === undefined ? args : inserted(args, at), notice(request)];
/// ```
pub fn with_skip_repo(arguments: Vec<OsString>, in_a_repository: bool) -> (Vec<OsString>, String) {
    if !in_a_repository {
        return (arguments, String::new());
    }
    let Some(install_index) = install_word(&arguments) else {
        return (arguments, String::new());
    };
    let request: InstallRequest = InstallRequest::read(arguments.as_slice(), install_index);
    if request.asks_for_help() {
        return (arguments, String::new());
    }
    let note: String = notice(&request);
    let Some(at) = request.skip_repo_insertion(install_index) else {
        return (arguments, note);
    };
    let mut rewritten: Vec<OsString> = arguments;
    rewritten.insert(at, OsString::from(SKIP_REPO_OPTION));
    return (rewritten, note);
}

/// What:
///  What a person is told when the wrapper added `--skip-repo` to their `git lfs install`.
/// Why:
///   The command then says only "Git LFS initialized.",
///  which does not say that no hook was installed or that none is needed,
///  and the person who ran it in a repository is the one who would wonder.
///  It states the destination this command asks git-lfs for,
///  the hooks that were not,
///  and the step that replaces them;
///  and it states that request rather than a completed write,
///  because git-lfs can stop after it reads the words well and then write nothing.
///  A command git-lfs refuses before it writes a key is told the refusal instead,
///  because naming a destination there would name a write that does not happen,
///  and the reason is left out when a word this reading does not name may be the one git-lfs stops at.
///  A run with `--manual` that git-lfs runs is told why the instructions that option asks for are not printed:
///  they are instructions for merging a hook this wrapper installs nowhere and never runs,
///  and a refused command prints none of them because it is refused,
/// so this is not added to it.
///  A caller's own `--skip-repo=false` is answered by the wrapper's copy after it,
///  so the notice says that word did not win.
///  `request` is the one reading of the caller's words,
///  which is where the destination,
/// the refusal,
/// `--manual` and that word come from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: \`git lfs install\` ran with \`--skip-repo\` ...
/// ${ask.kind === 'writes'
///   ? `This command asks git-lfs to put the \`filter.lfs.*\` keys in ${where}`
///   : `git-lfs refuses this command, because ${reason}`} ...`
/// ```
fn notice(request: &InstallRequest) -> String {
    let ask: InstallAsk = request.ask();
    let asked: String = match &ask {
        InstallAsk::Writes(scope) => {
            format!("This command asks git-lfs to put the `filter.lfs.*` keys in {}. ", scope.where_it_writes())
        }
        InstallAsk::Refused(refusal) => match request.named_reason(refusal) {
            Some(reason) => format!(
                "git-lfs refuses this command, because {reason}, so no `filter.lfs.*` key is written to any configuration \
                 file. "
            ),
            None => String::from(
                "git-lfs refuses this command, so no `filter.lfs.*` key is written to any configuration file. ",
            ),
        },
    };
    let mut text: String = format!(
        "monochromatic-git-policy-cli: `git lfs install` ran with `--skip-repo`, which the wrapper adds. {asked}The wrapper \
         does not run traditional hooks, and it uploads the Git LFS objects itself before each push, so no `pre-push` hook is \
         installed in this repository and none is needed; the `post-checkout`, `post-commit` and `post-merge` hooks it would \
         install are not needed here either. Without the option git-lfs tries to write its hooks into the directory the \
         wrapper disables, which fails and ends the command with status 2 and advice that cannot work."
    );
    if request.answers_a_word_that_turned_it_off() {
        text.push_str(
            " The last word that set `--skip-repo` turned it off, and the hooks it asks for cannot be installed here: the \
             directory git-lfs writes them into is disabled, so the wrapper's copy follows that word and they are skipped \
             all the same.",
        );
    }
    text.push('\n');
    if request.manual && matches!(ask, InstallAsk::Writes(_)) {
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
///  a help query is left to the one reading of the words,
/// which alone knows whether they ask for help,
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
    if arguments.get(install + 1).is_some_and(|word| { return word == "hooks"; }) {
        return None;
    }
    return Some(install);
}

/// The rewrite is controlled in the test module.
#[cfg(test)]
#[path = "lfs_install_arguments_tests.rs"]
mod tests;
