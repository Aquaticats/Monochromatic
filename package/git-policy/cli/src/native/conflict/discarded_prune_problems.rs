//! What:
//!  What can go wrong in a pass over the folder of saved copies, the words that say it, and the listing that carries both.
//! Why:
//!  `discarded_prune.rs` finds, plans and removes; the problems and the listing are plain values that the doctor and the guard read, so they live in a file of their own that `discarded_prune.rs` imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = problem.message(); // the line the doctor prints
//! ```

use crate::discarded_prune_stamps::Stamped;
use std::path::PathBuf;

/// What went wrong with the folder or one of its directories; each is kept, never guessed at.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProblemKind {
    /// The folder cannot be used as a store: a link stands where it should be, or it cannot be opened.
    StoreUnusable,
    /// A directory could not be read, so its size is unknown and it is kept.
    EntryUnreadable,
    /// A directory was replaced or changed between the listing and the removal, and was kept.
    EntryChanged,
    /// A removal failed after some files were gone.
    PartiallyRemoved,
    /// A removal failed before anything was removed.
    NotRemoved,
    /// The audit file could not be opened for appending, so nothing was removed.
    AuditUnwritable,
    /// A directory was removed but its audit line could not be written.
    RemovedWithoutAudit,
}

/// One thing a pass or a listing could not do, with the path it concerns and the reason in plain words.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Problem {
    /// The folder or directory concerned.
    pub path: PathBuf,
    /// What kind of trouble.
    pub kind: ProblemKind,
    /// The reason, as the system gave it.
    pub detail: String,
}

/// What a listing found: the directories named like the wrapper's own, and what it could not read.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Listing {
    /// The directories, oldest first.
    pub entries: Vec<Stamped>,
    /// What could not be read.
    pub problems: Vec<Problem>,
    /// How many files the entries hold.
    pub files: usize,
    /// The files each entry held when it was listed, by the entry's name, so that a removal can tell a directory that only
    /// lost files (another pass removing them) from one that gained a file after it was judged.
    pub listed_files: std::collections::HashMap<String, std::collections::HashSet<String>>,
}

impl Problem {
    /// What:
    ///  The sentence that tells a person what was kept or not done, which input it concerns and what to do about it.
    /// Why:
    ///   The copies are what a person may need back, so every case where the limits could not be applied, or an audit line
    ///  is missing, is said where the person sees it (the notice of the command, and `git cli-git doctor`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// `${problem.path} was kept: ${problem.detail}`
    /// ```
    pub fn message(&self) -> String {
        let path = self.path.display();
        let detail: &str = self.detail.as_str();
        return match self.kind {
            ProblemKind::StoreUnusable => format!(
                "cli-git: the saved copies of replaced files in {path} were not pruned, and the age and size limits did not apply: {detail}. \
                 Make it a real directory (move the copies there and remove the link) and the next save prunes it again.\n"
            ),
            ProblemKind::EntryUnreadable => format!(
                "cli-git: {path} was kept and not counted: it cannot be read ({detail}). Fix its permissions or remove it yourself; \
                 until then the size limit sees the folder as smaller than it is.\n"
            ),
            ProblemKind::EntryChanged => format!(
                "cli-git: {path} was kept: it was replaced or changed while the pass ran ({detail}), so it is not the directory the pass judged.\n"
            ),
            ProblemKind::PartiallyRemoved => format!(
                "cli-git: {path} was only partly removed ({detail}). The audit file lists the copies that went and the rest are still there; \
                 fix the cause and the next pass finishes the removal.\n"
            ),
            ProblemKind::NotRemoved => format!("cli-git: {path} could not be removed and was left whole ({detail}). The next pass tries again.\n"),
            ProblemKind::AuditUnwritable => format!(
                "cli-git: no saved copy was removed because the audit file {path} cannot be appended to ({detail}); every removal needs its \
                 line there. Fix the file or its permissions and the next pass removes what the limits allow.\n"
            ),
            ProblemKind::RemovedWithoutAudit => format!(
                "cli-git: {path} was removed but its line in the audit file could not be written ({detail}). Its copies were older or \
                 larger than the limits allow.\n"
            ),
        };
    }
}
