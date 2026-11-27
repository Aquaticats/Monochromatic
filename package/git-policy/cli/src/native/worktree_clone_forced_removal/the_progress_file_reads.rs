//! What:
//!  What the wrapper reads out of a progress file:
//!  one request's nonce,
//!  the stage the act last wrote and the counter it last published,
//!  when the read was taken,
//!  and whether a counter that has not moved reads as a dead handler.
//! Why:
//!  The wrapper asks whether the act lives,
//!  and a file for another request,
//!  another mode or another owner is never read as its own,
//!  so every fact the answer rests on is checked here before the counter is
//!  compared
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The act's liveness",
//!  L21).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reading = file.read();                       // nonce, stage, counter, readAt
//! const dead = file.givesUp(now - reading.readAt);   // whole heartbeat periods only
//! ```

use super::removal_progress_file::{
    ProgressFile,
    ProgressRefusal,
};
use super::the_progress_file_lines::{
    MAXIMUM_PROGRESS_BYTES,
    parse_progress,
};
use crate::worktree_clone_forced_removal::descriptor_calls::open_file_below;
use crate::worktree_clone_forced_removal::forced_arm_numbers::GIVE_UP_HEARTBEATS;
use crate::worktree_clone_forced_removal::forced_removal_words::Stage;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::RECEIPT_MODE;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::time::Duration;

/// What:
///  What one read of the file answered.
/// Why:
///  The wrapper reads the nonce to know the file is its own request's,
///  the stage to say what the act was doing and the counter to know it lives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProgressReading = { nonce: string; stage: Stage; counter: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressReading {
    /// The request's nonce.
    pub nonce: String,
    /// The stage the act last wrote.
    pub stage: Stage,
    /// The counter the act last published.
    pub counter: u64,
    /// When this read was taken,
    /// as the reader's own clock,
    /// so the give-up window is compared in one clock's terms.
    pub read_at: std::time::Instant,
}

impl ProgressFile {
    /// What:
    ///  Read one request's progress file.
    /// Why:
    ///  The wrapper asks whether the act lives,
    ///  and a file for another request,
    ///  another mode or another owner is never read as its own.
    ///
    /// # Errors
    /// The folder, the file or its bytes are not the designed ones,
    /// or the file answers another nonce.
    pub fn read(&self) -> Result<ProgressReading, ProgressRefusal> {
        let folder: std::fs::File = self.open_the_folder()?;
        let name: String = self.entry_name();
        let mut file: std::fs::File = open_file_below(&folder, name.as_str()).map_err(|error| {
            return ProgressRefusal {
                calling: "read",
                errno: error.raw_os_error(),
                said: format!("the progress file {name}"),
            };
        })?;
        let facts: std::fs::Metadata = file.metadata().map_err(|error| {
            return ProgressRefusal {
                calling: "read",
                errno: error.raw_os_error(),
                said: format!("the facts of the progress file {name}"),
            };
        })?;
        if !facts.is_file() {
            return Err(ProgressRefusal {
                calling: "read",
                errno: None,
                said: format!("{name} is not a regular file"),
            });
        }
        if facts.uid() != self.owner {
            return Err(ProgressRefusal {
                calling: "read",
                errno: None,
                said: format!(
                    "{name} is owned by account {} and the route requires {}",
                    facts.uid(),
                    self.owner
                ),
            });
        }
        if facts.mode() & 0o7777 != RECEIPT_MODE {
            return Err(ProgressRefusal {
                calling: "read",
                errno: None,
                said: format!(
                    "{name}'s mode is {:o} and the route requires {RECEIPT_MODE:o}",
                    facts.mode() & 0o7777
                ),
            });
        }
        let mut bytes: Vec<u8> = Vec::new();
        Read::by_ref(&mut file)
            .take(MAXIMUM_PROGRESS_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| {
                return ProgressRefusal {
                    calling: "read",
                    errno: error.raw_os_error(),
                    said: format!("the bytes of {name}"),
                };
            })?;
        if bytes.len() as u64 > MAXIMUM_PROGRESS_BYTES {
            return Err(ProgressRefusal {
                calling: "read",
                errno: None,
                said: format!("{name} is longer than a progress file may be"),
            });
        }
        let Ok(text) = String::from_utf8(bytes) else {
            return Err(ProgressRefusal {
                calling: "read",
                errno: None,
                said: format!("{name} is not text"),
            });
        };
        let (nonce, stage, counter) = parse_progress(text.as_str()).map_err(|said| {
            return ProgressRefusal {
                calling: "read",
                errno: None,
                said,
            };
        })?;
        if nonce != self.nonce.as_str() {
            return Err(ProgressRefusal {
                calling: "another-request",
                errno: None,
                said: format!(
                    "{name} answers nonce {nonce} and this request is {}",
                    self.nonce.as_str()
                ),
            });
        }
        return Ok(ProgressReading {
            nonce,
            stage,
            counter,
            read_at: std::time::Instant::now(),
        });
    }

    /// What:
    ///  Whether a counter that has not moved for this long reads as a dead
    ///  handler.
    /// Why:
    ///  The design fixes the give-up window as whole heartbeat periods,
    ///  and the wrapper's own wait compares nothing else.
    pub fn gives_up(
        &self,
        unchanged_for: Duration,
    ) -> bool {
        return unchanged_for >= self.heartbeat.saturating_mul(GIVE_UP_HEARTBEATS);
    }
}
