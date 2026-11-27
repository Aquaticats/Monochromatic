//! What:
//!  One progress file:
//!  where it lives,
//!  whose request it answers,
//!  how often its counter moves,
//!  and the folder every reader and writer of it proves first.
//! Why:
//!  The act and the wrapper each need to know the other is alive,
//!  and both reach the file through this one shape:
//!  the handler builds the live one beside the receipt,
//!  a control builds one at a folder it owns,
//!  and every write and read proves the folder is the designed root-owned
//!  `0755` one before it touches anything
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The act's liveness",
//!  L21).
//!  The five lines both sides speak are `the_progress_file_lines.rs`,
//!  what the handler writes is `the_progress_file_writes.rs`,
//!  and what the wrapper reads is `the_progress_file_reads.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const file = ProgressFile.live(guid, nonce, heartbeat);
//! const writing = file.begin();     // the first stage, and the counter starts moving
//! ```

use crate::worktree_clone_forced_removal::descriptor_calls::open_directory;
use crate::worktree_clone_forced_removal::forced_removal_words::{
    PROGRESS_FILE_SUFFIX,
    Stage,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::{
    DIRECTORY_MODE,
    RESULTS_DIRECTORY,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

// What:     the names the progress file's controls reach through this module,
//           whose grammar and reads now live in siblings; a test build only,
//           because no code here reads them.
// Why:      the control file is named for this module and reaches them with
//           `super::`, so the module must still answer to those names in the
//           run that checks them.
//
// In TS you'd write (pseudocode):
// ```ts
// import { parseProgress, progressText, ProgressReading } from './removalProgressFile.js';   // re-exported for the tests
// ```
#[cfg(test)]
use super::{
    ProgressReading,
    parse_progress,
    progress_text,
};

/// The stage a fresh file carries,
/// which the act writes before it reads anything.
pub(super) const FIRST_STAGE: Stage = Stage::Preflight;

/// What:
///  Why the progress file could not be written or read.
/// Why:
///  A folder that is not the designed one,
///  a staging write,
///  a mode a person changed and a file that answers another request are five
///  different things,
///  and the act's refusal names which one stopped it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProgressRefusal = { calling: 'folder' | 'staging' | 'read' | 'another-request'; errno?: number; said: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressRefusal {
    /// Which call refused,
    /// or which fact of the file did.
    pub calling: &'static str,
    /// The errno the call answered, where it answered one.
    pub errno: Option<i32>,
    /// What it said, in plain words.
    pub said: String,
}

impl std::fmt::Display for ProgressRefusal {
    /// One sentence naming the call, the errno and what it said.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self.errno {
            Some(errno) => write!(
                formatter,
                "{} answered errno {errno}: {}",
                self.calling, self.said
            ),
            None => write!(formatter, "{}: {}", self.calling, self.said),
        };
    }
}

/// What:
///  One progress file:
///  where it lives,
///  whose request it answers and how often its counter moves.
/// Why:
///  A control hands in a fixture folder it owns and the live handler the
///  design's own,
///  and the heartbeat is a value the act passes from its window rather than a
///  constant this module reads,
///  so a control can show a moving counter in milliseconds while the release
///  ships no period at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ProgressFile { constructor(folder, owner, guid, nonce, heartbeat) {} }
/// ```
#[derive(Clone, Debug)]
pub struct ProgressFile {
    pub(super) folder: PathBuf,
    pub(super) owner: u32,
    pub(super) dataset: DatasetGuid,
    pub(super) nonce: RequestNonce,
    pub(super) heartbeat: Duration,
    pub(super) stage_word: Arc<AtomicU64>,
}

impl ProgressFile {
    /// The file the root handler publishes beside the receipt.
    pub fn live(
        dataset: DatasetGuid,
        nonce: RequestNonce,
        heartbeat: Duration,
    ) -> ProgressFile {
        return ProgressFile {
            folder: PathBuf::from(RESULTS_DIRECTORY),
            owner: 0,
            dataset,
            nonce,
            heartbeat,
            stage_word: Arc::new(AtomicU64::new(FIRST_STAGE.order() as u64)),
        };
    }

    /// A file at a fixture folder,
    /// owned by the account a control expects.
    pub fn at(
        folder: PathBuf,
        owner: u32,
        dataset: DatasetGuid,
        nonce: RequestNonce,
        heartbeat: Duration,
    ) -> ProgressFile {
        return ProgressFile {
            folder,
            owner,
            dataset,
            nonce,
            heartbeat,
            stage_word: Arc::new(AtomicU64::new(FIRST_STAGE.order() as u64)),
        };
    }

    /// The file one dataset's progress is filed under.
    pub fn entry_name(&self) -> String {
        return format!("{}{PROGRESS_FILE_SUFFIX}", self.dataset.file_name());
    }

    /// Open the folder and prove it is the designed one.
    pub(super) fn open_the_folder(&self) -> Result<std::fs::File, ProgressRefusal> {
        let path: &Path = self.folder.as_path();
        let folder: std::fs::File = open_directory(path).map_err(|error| {
            return ProgressRefusal {
                calling: "folder",
                errno: error.raw_os_error(),
                said: format!("the results folder {}", path.display()),
            };
        })?;
        let facts: std::fs::Metadata = folder.metadata().map_err(|error| {
            return ProgressRefusal {
                calling: "folder",
                errno: error.raw_os_error(),
                said: format!("the facts of the results folder {}", path.display()),
            };
        })?;
        if facts.uid() != self.owner {
            return Err(ProgressRefusal {
                calling: "folder",
                errno: None,
                said: format!(
                    "{} is owned by account {} and the route requires {}",
                    path.display(),
                    facts.uid(),
                    self.owner
                ),
            });
        }
        if facts.mode() & 0o7777 != DIRECTORY_MODE {
            return Err(ProgressRefusal {
                calling: "folder",
                errno: None,
                said: format!(
                    "{}'s mode is {:o} and the route requires {DIRECTORY_MODE:o}",
                    path.display(),
                    facts.mode() & 0o7777
                ),
            });
        }
        return Ok(folder);
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_progress_file_tests.rs"]
mod tests;
