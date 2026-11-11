//! What:
//!  The rejection text of the shared-remote-settings refusal:
//!  a command that would change the remote settings every
//!  worktree of the repository shares.
//! Why:
//!  The person who ran the command needs three facts to undo the surprise the rule exists
//!  for:
//! the setting is shared by every worktree,
//! the main checkout is where it belongs,
//! and a real
//!  clone is the copy whose remote settings are its own.
//!  The words are plain on purpose:
//!  the
//!  failure this rule prevents looked like a broken remote and nothing on screen said which command
//!  had changed it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sharedRemoteSettingsMessage({ what: 'git remote set-url', configFile, mainCheckout })
//! //   === 'monochromatic-git-policy-cli: git remote set-url is rejected in this linked git worktree: ...'
//! ```

use crate::rule_shared_remote::SharedRemoteWrite;
/// `Path` borrows the two directories the text names.
use std::path::Path;

/// What:
///  How a text names the refused form.
/// Why:
///   A subcommand,
///  one key and a whole section are different things to name,
///  and the person should
///       recognize their own command in the words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describe(write: SharedRemoteWrite): string;
/// ```
fn describe(write: &SharedRemoteWrite) -> String {
    return match write {
        SharedRemoteWrite::RemoteSubcommand(name) => format!("git remote {name}"),
        SharedRemoteWrite::Key(key) => format!("the git config write to {key}"),
        SharedRemoteWrite::Section(section) => format!("the git config write to the {section} section"),
    };
}

/// What:
///  The complete rejection text.
///  `&Path` borrows the shared configuration file and the main checkout.
/// Why:
///   One text carries every fact and every way forward,
///  so a refusal is a step toward the goal
///       rather than a wall.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sharedRemoteSettingsMessage(write: SharedRemoteWrite, configFile: string, mainCheckout: string): string;
/// ```
pub fn shared_remote_settings_message(
    write: &SharedRemoteWrite,
    config_file: &Path,
    main_checkout: &Path,
) -> String {
    let what: String = describe(write);
    return format!(
        "monochromatic-git-policy-cli: {what} is rejected in this linked git worktree: every worktree of this repository \
         shares one Git configuration, {config}, so this command would change the remote settings the main checkout \
         uses too, and a push, fetch or pull from there would use the new setting with nothing recording which command \
         changed it. Nothing was changed.\n\
         To change a remote setting on purpose, run the same command in the main checkout, {main}, which owns that file.\n\
         For a copy of this repository whose remote settings are its own, clone it instead of adding a worktree: \
         git clone --no-hardlinks {main} <destination>.\n\
         If you have decided to change the shared settings from this worktree on purpose, pass --no-enforce-worktree in \
         option position of the same command, directly after git remote for a remote subcommand.",
        config = config_file.display(),
        main = main_checkout.display(),
    );
}
