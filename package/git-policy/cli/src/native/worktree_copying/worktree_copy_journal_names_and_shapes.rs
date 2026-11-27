//! What:
//!  The names a copy journal lives under and the shapes it is written as:
//!  the journal's parent directory,
//!  its version directory,
//!  the suffix of a journal and of an unpublished write,
//!  the phase a copy reached,
//!  one journal's record,
//!  and a record with the file it was read from.
//! Why:
//!  The encoder,
//! the validator and every reader of a journal must spell the same names
//! and pass the same shapes,
//!  and the shapes are what recovery reads before it acts on any path,
//! so they live where the encoding and the validation of the fields both reach them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type WorktreeCopyJournal = { phase: 'staged' | 'installing' | 'complete'; destinationRoot: string; /* ... */ };
//! const journalPath = join(commonDir, JOURNAL_PARENT_NAME, JOURNAL_VERSION_NAME, `${id}.${owner}${JOURNAL_SUFFIX}`);
//! ```

use crate::worktree_copy_install_entry::InstalledPath;
use crate::wrapper_names_old_and_new::WORKTREE_COPY_RECORD_FOLDER;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The parent directory name of the journals,
/// beneath the common Git directory.
/// Why:
///   Every linked worktree of one repository shares the same journal location.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const JOURNAL_PARENT_NAME = 'monochromatic-git-policy-cli-worktree-copy';
/// ```
pub const JOURNAL_PARENT_NAME: &str = WORKTREE_COPY_RECORD_FOLDER.current;

/// What:
///  The schema directory of the journals.
/// Why:
///   A later schema gets its own directory,
/// so an older wrapper never reads a newer record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const JOURNAL_VERSION_NAME = 'v1';
/// ```
pub const JOURNAL_VERSION_NAME: &str = "v1";

/// What:
///  The suffix of a final journal file.
/// Why:
///   Only files with this suffix are journals;
/// anything else in the directory is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const JOURNAL_SUFFIX = '.json';
/// ```
pub const JOURNAL_SUFFIX: &str = ".json";

/// What:
///  The suffix of an unpublished temporary journal write.
/// Why:
///   An interrupted write leaves only files with this suffix,
/// which recovery removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPORARY_SUFFIX = '.tmp';
/// ```
pub const TEMPORARY_SUFFIX: &str = ".tmp";

/// What:
///  The durable phase a copy has reached.
/// Why:
///   Recovery resumes an installation and finishes a completed cleanup differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyJournalPhase = 'complete' | 'installing' | 'staged';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// The stage is complete and nothing has been installed.
    Staged,
    /// Installation has begun;
    /// the intents and creations in the install log say what it touched.
    Installing,
    /// Installation finished;
    /// only cleanup remains.
    Complete,
}

impl Phase {
    /// The phase's name as the incumbent writes it.
    pub(crate) fn as_str(self) -> &'static str {
        // `match` keeps each phase's spelling next to its variant.
        match self {
            Phase::Staged => return "staged",
            Phase::Installing => return "installing",
            Phase::Complete => return "complete",
        }
    }
}

/// What:
///  One durable journal record.
/// Why:
///   Every field is read back by recovery before any filesystem path is used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyJournal = { createdEntries; intendedEntries; destinationRoot; phase; selectedRoots; sourceRoot; stageContainer; stageRoot; version: 1 };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Journal {
    /// Paths proven created,
    /// with their post-creation identities.
    pub created_entries: Vec<InstalledPath>,
    /// Selected paths claimed before installation.
    pub intended_entries: Vec<Vec<u8>>,
    /// The worktree receiving the copy.
    pub destination_root: PathBuf,
    /// The durable phase.
    pub phase: Phase,
    /// The ignored roots the staged payload represents.
    pub selected_roots: Vec<Vec<u8>>,
    /// The source worktree.
    pub source_root: PathBuf,
    /// The private container beside the destination.
    pub stage_container: PathBuf,
    /// The payload root inside the container.
    pub stage_root: PathBuf,
}

/// What:
///  A journal and the path it was read from or written to.
/// Why:
///   Removal and recovery need both the record and the file that must be deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PendingWorktreeCopyJournal = { path: string; record: WorktreeCopyJournal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingJournal {
    /// Absolute journal file path.
    pub path: PathBuf,
    /// The validated record.
    pub journal: Journal,
}

impl PendingJournal {
    /// The owner token the journal's file name carries,
    /// or nothing for a name no owner is written into.
    pub fn owner_token(&self) -> Option<String> {
        return owner_token_of(self.path.as_path());
    }
}

/// What:
///  The owner token in a journal file name `<id>.<owner>.json`,
/// if the name has that shape.
/// Why:
///   The owner is part of the name so that taking a journal over is one atomic rename.
///  A name of that shape has a token field,
///  so a stranger's name whose token field is empty names no owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownerTokenOf('0a1b.2c3d.json') === '2c3d';
/// ```
pub fn owner_token_of(path: &Path) -> Option<String> {
    let name: String = path.file_name()?.to_string_lossy().into_owned();
    let stem: &str = name.strip_suffix(JOURNAL_SUFFIX)?;
    let (_identifier, token) = stem.rsplit_once('.')?;
    // The writer fills both fields, so an empty token field is a stranger's file that must name no owner.
    if token.is_empty() {
        return None;
    }
    return Some(token.to_string());
}
