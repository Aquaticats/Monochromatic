//! What:
//!  The one root-owned folder the merge device keeps its records in,
//!  made when a fresh boot has none and proved through its own descriptor.
//! Why:
//!  The run claim,
//!  the mark and the pass budget are one folder under `/run`,
//!  so a boot clears all three and a claim means nothing across a boot
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The records",
//!  and veto entry V6,
//!  accepted as choice 11 in
//!  `doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
//!  "The seventeen choices as the project was shown them").
//!  The folder is proved the way the per-slot claim folder and the receipts
//!  folder are,
//!  because a fresh boot has none and the first event-driven pass must not
//!  refuse until something else made it.
//!  A link or another account's folder is refused without being written,
//!  which is why the proof opens no link and checks the owner before it changes
//!  the mode.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = SweepsFolder.live(); // '/run/git-worktree-clone-mount-sweeps'
//! folder.prepare(); // throws when another account owns it
//! ```

use crate::worktree_clone_mount_protocol::root_owned_directory::{
    RootDirectoryRefusal,
    make_and_prove,
};
use std::path::{
    Path,
    PathBuf,
};

/// The folder every merge-device record lives in.
pub const SWEEPS_FOLDER: &str = "/run/git-worktree-clone-mount-sweeps";

/// The folder's mode:
/// readable by everyone so the account's `doctor` can report it,
/// writable by root only.
pub const SWEEPS_FOLDER_MODE: u32 = 0o755;

/// The prefix of the run claim's numbered generations,
/// which the repository's claim publishes as `run.<generation>`.
pub const RUN_CLAIM_PREFIX: &str = "run";

/// The mark's file name inside the folder.
pub const PASS_MARK_NAME: &str = "another-pass-is-due";

/// The one line the mark's file holds.
/// Value:
///  `v1`,
/// a version word,
///  because presence is the whole meaning and a reader that found another
///  word must not take it for a mark this build wrote.
pub const PASS_MARK_TEXT: &str = "v1";

/// The mark's mode,
/// which lets the account's `doctor` read it.
pub const PASS_MARK_MODE: u32 = 0o644;

/// What:
///  Where the merge device's records are,
///  and which account must own them.
/// Why:
///   The live place is under `/run` and belongs to root,
///  and a control hands in a fixture folder it owns,
///  so the device is exercised without `/run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SweepsFolder { constructor(readonly directory: string, readonly owner: number) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SweepsFolder {
    /// The folder the claim generations,
    /// the mark and the budget live in.
    directory: PathBuf,
    /// The account every one of those must belong to.
    owner: u32,
}

impl SweepsFolder {
    /// The place the release handler keeps the device's records in.
    pub fn live() -> Self {
        return SweepsFolder {
            directory: PathBuf::from(SWEEPS_FOLDER),
            owner: 0,
        };
    }

    /// A place at a fixture folder,
    /// owned by the account the control expects.
    pub fn at(
        directory: PathBuf,
        owner: u32,
    ) -> Self {
        return SweepsFolder {
            directory,
            owner,
        };
    }

    /// The folder.
    pub fn directory(&self) -> &Path {
        return self.directory.as_path();
    }

    /// The mark's file.
    pub fn pass_mark_path(&self) -> PathBuf {
        return self.directory.join(PASS_MARK_NAME);
    }

    /// One generation of the run claim,
    /// which is the name the repository's claim publishes.
    pub fn run_claim_file(
        &self,
        generation: u64,
    ) -> PathBuf {
        return self
            .directory
            .join(format!("{RUN_CLAIM_PREFIX}.{generation}"));
    }

    /// Make the folder when it is absent,
    /// at its designed mode,
    /// and prove it is the designed one.
    ///
    /// # Errors
    /// The folder could not be made or given its mode,
    /// or it is owned by another account,
    /// each in the words a pass reports instead of refusing.
    pub fn prepare(&self) -> Result<(), String> {
        make_and_prove(self.directory.as_path(), self.owner, SWEEPS_FOLDER_MODE)
            .map_err(|refusal| return refusal_words(self.directory.as_path(), &refusal))?;
        return Ok(());
    }
}

/// What:
///  Why the folder could not be made or proved,
///  in the words a pass prints before it runs anyway.
/// Why:
///   The device is cost and never safety (V7's first option),
///  so a folder that cannot be proved is reported and the pass continues;
///  the sentence says both,
/// or a reader would take the report for a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = refusalWords(path, refusal); // 'the sweep merge folder /run/… is owned by user 1000 …'
/// ```
fn refusal_words(
    path: &Path,
    refusal: &RootDirectoryRefusal,
) -> String {
    let because: String = match refusal {
        RootDirectoryRefusal::Open {
            reason,
        } => format!("it could not be made or opened ({reason})"),
        RootDirectoryRefusal::Owner {
            found,
            wanted,
        } => format!("it is owned by user {found} and this route requires user {wanted}"),
        RootDirectoryRefusal::Mode {
            found,
            wanted,
        } => format!("it has mode {found:04o} and this route requires {wanted:04o}"),
    };
    return format!(
        "the sweep merge folder {} cannot be used: {because}",
        path.display()
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweeps_folder_tests.rs"]
mod tests;
