//! What:
//!  What the handler writes into a progress file:
//!  the first stage and the heartbeat that starts with it,
//!  one stage word per stage the act reaches,
//!  the last word when the act ends,
//!  and the staging sibling every publish renames over the file.
//! Why:
//!  A timer thread advances the counter once per heartbeat period so a long
//!  preflight,
//!  wait or unmount does not read as a dead handler,
//!  the act also advances it at each stage so the wrapper's give-up window
//!  compares whole periods of a counter a stage may move sooner,
//!  and every write is a staging sibling renamed over the file,
//!  so a reader sees one whole file
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The act's liveness",
//!  L21).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const writing = file.begin();                  // first stage + a moving counter
//! writing.stage(Stage.Signals);                  // one word per stage
//! writing.finish();                              // the last word, and the thread stops
//! ```

use super::removal_progress_file::{
    FIRST_STAGE,
    ProgressFile,
    ProgressRefusal,
};
use super::the_progress_file_lines::progress_text;
use crate::worktree_clone_forced_removal::descriptor_calls::{
    create_at,
    remove_entry_at,
    rename_at,
    sync_directory,
};
use crate::worktree_clone_forced_removal::forced_removal_words::{
    PROGRESS_FILE_SUFFIX,
    Stage,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::RECEIPT_MODE;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};
use std::time::Duration;

/// Distinguishes the staging names of two publishes one process makes,
/// so no publish ever shares another's sibling.
static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

/// The sibling one publish is staged under before its rename.
/// Why:
///  The name carries this process's number and a counter of this process's
///  own attempts,
///  so two publishes never share a staging name.

impl ProgressFile {
    /// The sibling one publish is staged under before its rename.
    /// Why:
    ///  The name carries this process's number and a counter of this process's
    ///  own attempts,
    ///  so two publishes never share a staging name.
    pub(super) fn staging_name(&self) -> String {
        let attempt: u64 = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
        return format!(
            "{}{PROGRESS_FILE_SUFFIX}.staging.{}.{attempt}",
            self.dataset.file_name(),
            std::process::id()
        );
    }

    /// What:
    ///  Publish the file's first stage and start its heartbeat.
    /// Why:
    ///  The wrapper's first read must find a counter,
    ///  and the act's own first stage must be there before it reads anything,
    ///  so one call writes both and hands back the value that keeps the
    ///  counter moving, which stops when it is dropped.
    ///
    /// # Errors
    /// The folder is not the designed root-owned `0755` one,
    /// or the staging write and its rename answered an error.
    pub fn begin(&self) -> Result<ProgressWriting, ProgressRefusal> {
        let counter: Arc<AtomicU64> = Arc::new(AtomicU64::new(1));
        let folder: std::fs::File = self.open_the_folder()?;
        self.publish(&folder, FIRST_STAGE, 1)?;
        let (stop, listening) = std::sync::mpsc::channel::<()>();
        let thread_folder: std::fs::File = folder.try_clone().map_err(|error| {
            return ProgressRefusal {
                calling: "folder",
                errno: error.raw_os_error(),
                said: String::from(
                    "the results folder's descriptor could not be shared with its heartbeat",
                ),
            };
        })?;
        let period: Duration = self.heartbeat;
        let shared: Arc<AtomicU64> = Arc::clone(&counter);
        let held: ProgressFile = self.clone();
        let heartbeat =
            crate::foundation::scoped_start::spawn_named("forced-removal-progress", move || {
                // The thread ends when its channel is closed or when the act sends its stop, and each period it publishes
                // the next counter with the stage the act last set, which it reads from the shared word.
                while let Err(std::sync::mpsc::RecvTimeoutError::Timeout) =
                    listening.recv_timeout(period)
                {
                    let next: u64 = shared.fetch_add(1, Ordering::SeqCst) + 1;
                    let stage: Stage = Stage::of_order(
                        u8::try_from(held.stage_word.load(Ordering::SeqCst)).unwrap_or(0),
                    );
                    let _published: Result<(), ProgressRefusal> =
                        held.publish(&thread_folder, stage, next);
                }
            })
            .map_err(|error| {
                return ProgressRefusal {
                    calling: "thread",
                    errno: error.raw_os_error(),
                    said: crate::foundation::scoped_start::refused_thread_sentence(
                        "advance the forced removal's progress file",
                        &error,
                    ),
                };
            })?;
        return Ok(ProgressWriting {
            file: self.clone(),
            counter,
            stop,
            heartbeat,
            stage_word: self.stage_word.clone(),
        });
    }

    /// What:
    ///  Write one whole file at the dataset's name,
    ///  through a staging sibling.
    /// Why:
    ///  A reader sees the whole earlier file or the whole new one,
    ///  and the sibling is removed on every way out so this process's next
    ///  publish is never blocked by its own failure.
    pub(super) fn publish(
        &self,
        folder: &std::fs::File,
        stage: Stage,
        counter: u64,
    ) -> Result<(), ProgressRefusal> {
        let name: String = self.entry_name();
        let staging: String = self.staging_name();
        let text: String = progress_text(self.dataset, &self.nonce, stage, counter);
        let staged: Result<(), ProgressRefusal> = (|| {
            let mut file: std::fs::File = create_at(folder, staging.as_str(), RECEIPT_MODE)
                .map_err(|error| {
                    return ProgressRefusal {
                        calling: "staging",
                        errno: error.raw_os_error(),
                        said: format!("the progress file's staging sibling {staging}"),
                    };
                })?;
            file.write_all(text.as_bytes()).map_err(|error| {
                return ProgressRefusal {
                    calling: "staging",
                    errno: error.raw_os_error(),
                    said: format!("the write of {staging}"),
                };
            })?;
            file.sync_all().map_err(|error| {
                return ProgressRefusal {
                    calling: "staging",
                    errno: error.raw_os_error(),
                    said: format!("the sync of {staging}"),
                };
            })?;
            drop(file);
            rename_at(folder, staging.as_str(), name.as_str()).map_err(|error| {
                return ProgressRefusal {
                    calling: "staging",
                    errno: error.raw_os_error(),
                    said: format!("the rename of {staging} over {name}"),
                };
            })?;
            return sync_directory(folder).map_err(|error| {
                return ProgressRefusal {
                    calling: "staging",
                    errno: error.raw_os_error(),
                    said: format!("the sync of the folder after {name}"),
                };
            });
        })();
        if staged.is_err() {
            let _removed: Result<(), std::io::Error> = remove_entry_at(folder, staging.as_str());
        }
        return staged;
    }
}

/// What:
///  One progress file being advanced:
///  the counter the heartbeat raises and the stage word the act sets.
/// Why:
///  A stage that was set and then overwritten by the heartbeat must not lose
///  its word,
///  so the thread reads the act's own stage word rather than a stale copy,
///  and dropping this value stops the thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ProgressWriting { stage(next: Stage): void; counter(): number; finish(): void }
/// ```
pub struct ProgressWriting {
    file: ProgressFile,
    counter: Arc<AtomicU64>,
    stop: std::sync::mpsc::Sender<()>,
    heartbeat: std::thread::JoinHandle<()>,
    stage_word: Arc<AtomicU64>,
}

impl ProgressWriting {
    /// What:
    ///  Publish one stage word with the counter as it stands.
    /// Why:
    ///  The act's own progress is what the wrapper's report prints,
    ///  and each stage's write also advances the counter,
    ///  which is the design's own "the act also advances it at each stage".
    ///
    /// # Errors
    /// The folder or the staging write answered an error.
    pub fn stage(
        &self,
        stage: Stage,
    ) -> Result<(), ProgressRefusal> {
        let counter: u64 = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        self.stage_word
            .store(stage.order() as u64, Ordering::SeqCst);
        let folder: std::fs::File = self.file.open_the_folder()?;
        return self.file.publish(&folder, stage, counter);
    }

    /// The counter as it stands.
    pub fn counter(&self) -> u64 {
        return self.counter.load(Ordering::SeqCst);
    }

    /// What:
    ///  Publish the last stage and stop the heartbeat.
    /// Why:
    ///  A counter that kept moving after the act ended would tell the wrapper a
    ///  handler that is gone still lives.
    ///
    /// # Errors
    /// The last write answered an error, which the act reports beside its own
    /// answer.
    pub fn finish(self) -> Result<(), ProgressRefusal> {
        let ProgressWriting {
            file,
            counter,
            stop,
            heartbeat,
            stage_word,
        } = self;
        let next: u64 = counter.fetch_add(1, Ordering::SeqCst) + 1;
        stage_word.store(Stage::Done.order() as u64, Ordering::SeqCst);
        let folder: std::fs::File = file.open_the_folder()?;
        let published: Result<(), ProgressRefusal> = file.publish(&folder, Stage::Done, next);
        // Closing the channel ends the heartbeat's wait, and joining it here means the act's own end is not read as a
        // live handler.
        drop(stop);
        let _joined: Result<(), Box<dyn std::any::Any + Send>> = heartbeat.join();
        return published;
    }
}
