//! What:
//!  After a forwarded command that moves the worktree to other files (`merge`,
//! `pull`,
//! `checkout`,
//! `switch`,
//! `rebase`,
//! `reset`)
//! the client compares the text of the forbidden-strings rules files it would load before and after,
//! and asks the commit service
//! that already runs to compile the new text when it differs.
//! Why:
//!  The first commit after such a command meets rules text with no compiled form in the shared cache and compiles it itself,
//! 1.7 s of processor time and about 20 s under load.
//! Only these commands read the files (a command that cannot change the worktree adds nothing),
//! a command whose rules text did
//! not change sends nothing,
//! and no service is ever started for this (`service_warm_request.rs`).
//! Each rules file is opened without blocking and through a link,
//! so a named pipe at a rules name ends the
//!  command with a line naming the path instead of holding it for ever,
//! and a file past `SMALL_FILE_LIMIT` is not
//!  marked at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mark = markRulesText(command, identity); await runGit(); mark?.askIfChanged();
//! ```

use crate::config_file::load_repository_config;
use crate::forwarded_command::ResolvedCommand;
use crate::plain_file::{SMALL_FILE_LIMIT, read_plain_file_through_links};
use crate::scanner_selection::rules_source;
use crate::service_warm_request::ask_running_service_to_warm;
use crate::worktree_identity::{WorktreeIdentity, git_directory, worktree_root};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

/// The commands that move the worktree to other files,
/// and so may change the rules text.
pub const WORKTREE_MOVING_COMMANDS: [&[u8]; 6] = [b"merge", b"pull", b"checkout", b"switch", b"rebase", b"reset"];

/// The rules text as it stood before the command.
pub struct RulesTextMark {
    /// The worktree's own Git directory.
    git_dir: PathBuf,
    /// The worktree's top level.
    root: PathBuf,
    /// A digest of the rules files' bytes,
    /// with an absent file told from an empty one.
    digest: u64,
}

/// A digest of the bytes of every rules file the configuration names,
/// or why there is none.
fn digest_of_rules_text(root: &Path) -> Result<u64, NoDigest> {
    let loaded = load_repository_config(root).map_err(|_not_selected| { return NoDigest::Unselectable; })?;
    let options = &loaded.config.policies.forbidden_strings;
    let rules = rules_source(options.rules_files.as_deref(), std::env::var_os("FORBIDDEN_STRINGS_RULES").as_deref(), root)
        .map_err(|_not_selected| { return NoDigest::Unselectable; })?;
    let mut hasher: DefaultHasher = DefaultHasher::new();
    for entry in &rules.entries {
        entry.path.hash(&mut hasher);
        // A rules file is opened without blocking and through a link of the person's own: a named pipe at its name
        // must not be waited on, and a link is the path the configuration means.
        match read_plain_file_through_links(entry.path.as_path(), SMALL_FILE_LIMIT) {
            Ok(bytes) => {
                1_u8.hash(&mut hasher);
                bytes.hash(&mut hasher);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0_u8.hash(&mut hasher),
            Err(reason) => return Err(NoDigest::Unreadable { path: entry.path.clone(), reason }),
        }
    }
    return Ok(hasher.finish());
}

/// Why the rules text has no digest.
#[derive(Debug)]
enum NoDigest {
    /// The selection cannot be made:
    /// no configuration,
    /// or a rules list that does not parse.
    /// Nothing is marked, and the commit that needs the rules compiles them itself.
    Unselectable,
    /// A rules file the selection names is not a plain file to read,
    /// so its text is unknown.
    Unreadable {
        /// The file the selection names.
        path: PathBuf,
        /// What the open found there,
        /// or the read's own failure.
        reason: std::io::Error,
    },
}

/// What:
///  The line for the person when a rules file cannot be read:
/// it names the path,
/// says what was found there,
/// and says what follows for the command.
/// Why:
///   A command that cannot read the rules text says so instead of leaving the person with a
/// file that does nothing;
/// the commit that reads the rules itself then reports what it finds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `... the rules file ${path} is not text this command can read (${reason}) ...`
/// ```
fn unreadable_rules_line(path: &Path, reason: &std::io::Error) -> String {
    return format!(
        "monochromatic-git-policy-cli: the forbidden-strings rules file {} is not text this command can read ({reason}), so the rules text \
         is not marked here; the next commit reads it and reports what it finds.\n",
        path.display()
    );
}

/// The digest of the rules text,
/// with a rules file that cannot be read said on the terminal,
/// naming it.
/// A selection that cannot be made stays silent,
/// as it was:
/// nothing is lost there,
/// because the commit that needs the rules compiles them itself.
fn digest_or_report(root: &Path) -> Option<u64> {
    return match digest_of_rules_text(root) {
        Ok(digest) => Some(digest),
        Err(NoDigest::Unselectable) => None,
        Err(NoDigest::Unreadable { path, reason }) => {
            let line: String = unreadable_rules_line(path.as_path(), &reason);
            let _said: std::io::Result<()> = std::io::Write::write_all(&mut std::io::stderr(), line.as_bytes());
            None
        }
    };
}

/// Marks the rules text before `command` runs,
/// when it is one of the commands that move the worktree.
pub fn mark_rules_text(command: &ResolvedCommand, identity: &WorktreeIdentity) -> Option<RulesTextMark> {
    if !WORKTREE_MOVING_COMMANDS.contains(&command.word()) {
        return None;
    }
    let root: &Path = worktree_root(identity)?;
    let git_dir: &Path = git_directory(identity)?;
    let digest: u64 = digest_or_report(root)?;
    return Some(RulesTextMark { git_dir: git_dir.to_path_buf(), root: root.to_path_buf(), digest });
}

impl RulesTextMark {
    /// Asks the running service to compile the rules when their text is not what it was;
    /// says whether it asked.
    pub fn ask_if_changed(&self) -> bool {
        let Some(after) = digest_or_report(self.root.as_path()) else { return false };
        if after == self.digest {
            return false;
        }
        return ask_running_service_to_warm(self.git_dir.as_path(), self.root.as_path());
    }
}

/// The warm request is controlled in the test module.
#[cfg(test)]
#[path = "entry_rules_text_change_warm_tests.rs"]
mod tests;
