//! What:
//! What one replacement answers with,
//! and the rule that says where the file it displaced is kept:
//! beside the target under a suffix,
//! or in one directory such as the quarantine.
//! Why:
//! Every caller reads one of these answers to decide whether its file is its own text,
//! another writer's file or a kept copy,
//! and the keep rule decides what a person finds on disk after a refusal or a rollback.
//! Split out of `journal_replace.rs`;
//! the two types are re-exported from there,
//! so their paths are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal_replace_outcome_and_keep_rule.ts: type ReplaceOutcome, interface KeepAs
//! ```

/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// A kept name is produced here and outlives the replacement that produced it,
/// so it is owned,
/// while the directory a quarantine is chosen in is only lent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{
    Path,
    PathBuf,
};

/// What a compare-and-replace did to one file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplaceOutcome {
    /// The file held what was expected and now holds the new text;
    /// the displaced file was kept under the name inside.
    Replaced(
        /// Where the displaced file was kept (always `Some` unless the target had vanished and nothing was displaced).
        Option<PathBuf>,
    ),
    /// The file held something else,
    /// or held it only for a moment and was put back:
    /// another writer's file is where it was.
    LeftAlone,
    /// The snapshot is live and the file it displaced is kept at this path
    /// (it was not exactly the expected one,
    /// or could only be kept under its temporary name).
    Displaced(
        /// Where the displaced file is kept.
        PathBuf,
    ),
    /// The exchange was undone:
    /// another writer's file is live and the replacement inode is kept at this path.
    Undone(
        /// Where the replacement is kept.
        PathBuf,
    ),
}

/// Where and under what suffix a displaced file is kept.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeepAs {
    /// The directory the kept names are made in;
    /// `None` means next to the target.
    pub directory: Option<PathBuf>,
    /// The suffix after the target's file name (`.displaced`,
    /// `.refused`).
    pub suffix: String,
}

/// The constructors of the places a displaced file is kept.
impl KeepAs {
    /// What:
    /// `pub fn next_to_target(suffix: &str) -> KeepAs` keeps displaced files in the target's own directory.
    /// Why:
    /// A refused lockfile saved by a recovery sits beside the lockfile,
    /// where the person looks for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static nextToTarget(suffix: string): KeepAs
    /// ```
    pub fn next_to_target(suffix: &str) -> KeepAs {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
        // ```
        return KeepAs {
            directory: None,
            suffix: suffix.to_string(),
        };
    }

    /// What:
    /// `pub fn in_directory(directory: &Path, suffix: &str) -> KeepAs` keeps displaced files in one directory
    /// (the quarantine under `.cache/cargo-shadow/`).
    /// Why:
    /// An ordinary rollback of this run's own output should not leave files in the working tree.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static inDirectory(directory: string, suffix: string): KeepAs
    /// ```
    pub fn in_directory(
        directory: &Path,
        suffix: &str,
    ) -> KeepAs {
        // `Some`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return KeepAs {
            directory: Some(directory.to_path_buf()),
            suffix: suffix.to_string(),
        };
    }
}
