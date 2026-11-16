//! What:
//!  Every name this program answers to or keeps on disk,
//! as a pair:
//! the name of this release and the name an earlier
//!  release used before the program became `monochromatic-git-policy-cli`.
//! Why:
//!  The owner renamed the program,
//! its command word,
//! its settings file and the folders and files it keeps inside the Git
//!  directory (`doc/planning/git-policy-cli-program-rename.md`).
//! An older install,
//! a settings file in a repository that has
//!  not been renamed and a folder an older release wrote must all keep working,
//! so each place that reads or writes one of
//!  these names takes the pair from here and never spells either half itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const PROGRAM = { current: 'monochromatic-git-policy-cli', earlier: 'cli-git-native' } as const;
//! ```

/// What:
///  One renamed name:
///  the spelling this release writes and the spelling an earlier release wrote.
///  `&'static str` is text compiled into the executable.
/// Why:
///   Reading code needs both spellings,
///  and writing code needs only the current one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RenamedName = { readonly current: string; readonly earlier: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenamedName {
    /// The spelling this release writes.
    pub current: &'static str,
    /// The spelling an earlier release wrote,
    ///  still accepted for reading.
    pub earlier: &'static str,
}

/// The executable's file name,
///  which is the crate's name.
pub const PROGRAM: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli",
    earlier: "cli-git-native",
};

/// The word after `git` that the wrapper answers itself,
///  as in `git monochromatic-git-policy-cli doctor`.
pub const COMMAND_WORD: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli",
    earlier: "cli-git",
};

/// The settings file at a repository's top level.
pub const SETTINGS_FILE: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli.config.jsonc",
    earlier: "cli-git.config.jsonc",
};

/// The prefix of the wrapper's own lines on standard error,
///  and the name its messages call it by.
pub const MESSAGE_PREFIX: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli",
    earlier: "cli-git",
};

/// The folder inside a Git directory that holds the saved copies,
///  notices,
///  work records,
///  removal snapshots and the standards-review log.
pub const DATA_FOLDER: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli",
    earlier: "cli-git",
};

/// The log of discarded conflict sides and their prunes,
///  inside the common Git directory.
pub const DISCARD_AUDIT_LOG: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli-audit.jsonl",
    earlier: "cli-git-audit.jsonl",
};

/// The log of where the time of each commit went,
///  inside the common Git directory.
pub const COMMIT_TIMING_LOG: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli-commit-timing.jsonl",
    earlier: "cli-git-commit-timing.jsonl",
};

/// The log of where the time of each `git worktree add` went,
///  inside the common Git directory.
pub const WORKTREE_ADD_TIMING_LOG: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli-worktree-add-timing.jsonl",
    earlier: "cli-git-worktree-add-timing.jsonl",
};

/// The log of what triggered each automatic push,
///  inside the common Git directory.
pub const PUSH_TRIGGER_LOG: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli-push-trigger.jsonl",
    earlier: "cli-git-push-trigger.jsonl",
};

/// The records of a merge that was started and not yet concluded,
///  inside a Git directory.
pub const MERGE_INTENT_FOLDER: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli-merges",
    earlier: "cli-git-merges",
};

/// The folder of ignored-state copy records,
///  inside the common Git directory.
pub const WORKTREE_COPY_RECORD_FOLDER: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli-worktree-copy",
    earlier: "cli-git-worktree-copy",
};

/// The folder inside a Git directory that holds the repository scratch:
///  short-lived private files that a command makes and a later preparation sweeps once they are stale.
pub const REPOSITORY_SCRATCH: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli-scratch",
    earlier: "cli-git-scratch",
};

/// The folder inside a Git common directory that holds the finished shadows a commit can take instead of making one.
///  It sits beside the repository scratch directory,
/// not inside it,
///  because a member is standing state the owner asked for and not scratch:
///  the check that a commit leaves no scratch behind must keep meaning what it says.
///  This release is the first to keep one,
///  so it has no earlier spelling.
pub const SHADOW_POOL_FOLDER: &str = "monochromatic-git-policy-cli-shadow-pool";

/// The folder inside a Git common directory that holds the finished shadows the resident service drew
/// out of the pool ahead of the commits that ask for one.
///  It sits beside the pool's folder and never inside it:
///  a claim takes a member by renaming it out of the pool,
///  and the removal of a removed worktree takes the pool's folder out of its name with one rename before anything
///  of it is deleted,
///  while a drawn tree is used where it stands and must be left alone while a commit holds it.
///  This release is the first to keep one,
///  so it has no earlier spelling.
pub const SHADOW_DRAWN_FOLDER: &str = "monochromatic-git-policy-cli-shadow-drawn";

/// The prefix of a commit's scratch directory:
///  the private indexes Git's scratch repository is built from,
///  kept inside the repository scratch directory.
pub const SERVICE_SCRATCH_PREFIX: &str = "monochromatic-git-policy-cli-service-";

/// What:
///  Whether a byte may appear in a request identity or a scratch directory identity:
/// an ASCII letter or digit,
/// or a hyphen.
/// Why:
///  Both are used in file names,
///  so no identity can name a path,
///  and a name that holds something else is never taken for a scratch directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isIdentityByte = (byte: number) => /[A-Za-z0-9-]/.test(String.fromCharCode(byte));
/// ```
pub fn is_identity_byte(byte: &u8) -> bool {
    return byte.is_ascii_alphanumeric() || *byte == b'-';
}

/// The end of the name an entry of an earlier folder takes when the current folder holds an entry of the same name,
///  so that both are kept (`standards-review-shadow.md` and `standards-review-shadow.md.from-earlier-release`).
pub const KEPT_BESIDE_SUFFIX: &str = ".from-earlier-release";

/// The commit service's units and sockets,
///  by the name `systemd` knows them as.
pub const SERVICE_UNIT: RenamedName = RenamedName {
    current: "monochromatic-git-policy-cli",
    earlier: "cli-git",
};

#[cfg(test)]
#[path = "wrapper_names_old_and_new_tests.rs"]
mod tests;
