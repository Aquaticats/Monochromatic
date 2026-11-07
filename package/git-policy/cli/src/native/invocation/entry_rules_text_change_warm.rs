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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mark = markRulesText(command, identity); await runGit(); mark?.askIfChanged();
//! ```

use crate::config_file::load_repository_config;
use crate::forwarded_command::ResolvedCommand;
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
/// or nothing when the selection cannot be made.
fn digest_of_rules_text(root: &Path) -> Option<u64> {
    let loaded = load_repository_config(root).ok()?;
    let options = &loaded.config.policies.forbidden_strings;
    let rules = rules_source(options.rules_files.as_deref(), std::env::var_os("FORBIDDEN_STRINGS_RULES").as_deref(), root).ok()?;
    let mut hasher: DefaultHasher = DefaultHasher::new();
    for entry in &rules.entries {
        entry.path.hash(&mut hasher);
        match std::fs::read(entry.path.as_path()) {
            Ok(bytes) => {
                1_u8.hash(&mut hasher);
                bytes.hash(&mut hasher);
            }
            Err(_absent) => 0_u8.hash(&mut hasher),
        }
    }
    return Some(hasher.finish());
}

/// Marks the rules text before `command` runs,
/// when it is one of the commands that move the worktree.
pub fn mark_rules_text(command: &ResolvedCommand, identity: &WorktreeIdentity) -> Option<RulesTextMark> {
    if !WORKTREE_MOVING_COMMANDS.contains(&command.word()) {
        return None;
    }
    let root: &Path = worktree_root(identity)?;
    let git_dir: &Path = git_directory(identity)?;
    return Some(RulesTextMark { git_dir: git_dir.to_path_buf(), root: root.to_path_buf(), digest: digest_of_rules_text(root)? });
}

impl RulesTextMark {
    /// Asks the running service to compile the rules when their text is not what it was;
    /// says whether it asked.
    pub fn ask_if_changed(&self) -> bool {
        let Some(after) = digest_of_rules_text(self.root.as_path()) else { return false };
        if after == self.digest {
            return false;
        }
        return ask_running_service_to_warm(self.git_dir.as_path(), self.root.as_path());
    }
}
