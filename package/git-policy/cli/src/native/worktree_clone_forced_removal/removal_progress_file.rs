//! What:
//!  The progress file the handler advances while the forced act lives,
//!  and the reads the wrapper makes of it.
//! Why:
//!  The act and the wrapper each need to know the other is alive,
//!  and the handler's half is a root-owned file beside the receipt carrying the
//!  request's nonce,
//!  the stage word and a counter
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The act's liveness",
//!  L21).
//!  A timer thread advances the counter once per heartbeat period so a long
//!  preflight,
//!  wait or unmount does not read as a dead handler,
//!  and the act also advances it at its start and at each stage,
//!  so the wrapper's give-up window compares whole periods of a counter that a
//!  stage may move sooner.
//!  The counter alone decides liveness:
//!  the stage word is what a person and the wrapper's report read.
//!  Every write is a staging sibling renamed over the file,
//!  so a reader sees one whole file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const writing = ProgressFile.live(guid, nonce, window.heartbeat()).begin();  // throws when the folder is not the designed one
//! writing.stage(Stage.Signals); … writing.finish();
//! ```

use crate::worktree_clone_forced_removal::descriptor_calls::{
    create_at,
    open_directory,
    open_file_below,
    remove_entry_at,
    rename_at,
    sync_directory,
};
use crate::worktree_clone_forced_removal::forced_arm_numbers::GIVE_UP_HEARTBEATS;
use crate::worktree_clone_forced_removal::forced_removal_words::{
    PROGRESS_FILE_SUFFIX,
    Stage,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::{
    DIRECTORY_MODE,
    RECEIPT_MODE,
    RESULTS_DIRECTORY,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::io::{
    Read,
    Write,
};
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};
use std::time::Duration;

/// The file's format version,
/// which the reader checks before anything else.
pub const PROGRESS_VERSION: &str = "v1";

/// The file's kind,
/// which tells it from the receipt beside it.
pub const PROGRESS_KIND: &str = "removal-progress";

/// The most bytes one progress file takes:
/// five short lines,
/// the stage word and a counter of this machine's own.
pub const MAXIMUM_PROGRESS_BYTES: u64 = 512;

/// The stage a fresh file carries,
/// which the act writes before it reads anything.
const FIRST_STAGE: Stage = Stage::Preflight;

/// Distinguishes the staging names of two publishes one process makes,
/// so no publish ever shares another's sibling.
static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    folder: PathBuf,
    owner: u32,
    dataset: DatasetGuid,
    nonce: RequestNonce,
    heartbeat: Duration,
    stage_word: Arc<AtomicU64>,
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

    /// The sibling one publish is staged under before its rename.
    /// Why:
    ///  The name carries this process's number and a counter of this process's
    ///  own attempts,
    ///  so two publishes never share a staging name.
    fn staging_name(&self) -> String {
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

    /// Open the folder and prove it is the designed one.
    fn open_the_folder(&self) -> Result<std::fs::File, ProgressRefusal> {
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

    /// What:
    ///  Write one whole file at the dataset's name,
    ///  through a staging sibling.
    /// Why:
    ///  A reader sees the whole earlier file or the whole new one,
    ///  and the sibling is removed on every way out so this process's next
    ///  publish is never blocked by its own failure.
    fn publish(
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

/// What:
///  One progress file's whole text.
/// Why:
///  The reader must find the four values by name,
///  and the nonce is data beside the dataset's name and never part of it.
pub fn progress_text(
    dataset: DatasetGuid,
    nonce: &RequestNonce,
    stage: Stage,
    counter: u64,
) -> String {
    let lines: [String; 6] = [
        String::from(PROGRESS_VERSION),
        format!("kind {PROGRESS_KIND}"),
        format!("dataset {}", dataset.file_name()),
        format!("nonce {}", nonce.as_str()),
        format!("stage {}", stage.word()),
        format!("counter {counter}"),
    ];
    return format!("{}\n", lines.join("\n"));
}

/// What:
///  Read one progress file's four values.
/// Why:
///  A half-written or foreign file is refused by its own grammar rather than
///  read as a counter,
///  which is what keeps a torn file from reading as a dead handler.
///
/// # Errors
/// A line is absent or names no value this build writes.
pub fn parse_progress(text: &str) -> Result<(String, Stage, u64), String> {
    let mut nonce: Option<String> = None;
    let mut stage: Option<Stage> = None;
    let mut counter: Option<u64> = None;
    let mut version_seen: bool = false;
    let mut kind_seen: bool = false;
    let mut dataset_seen: bool = false;
    for line in text.lines() {
        if line == PROGRESS_VERSION {
            version_seen = true;
            continue;
        }
        if let Some(kind) = line.strip_prefix("kind ") {
            if kind != PROGRESS_KIND {
                return Err(format!("the kind `{kind}` is not one this build writes"));
            }
            kind_seen = true;
            continue;
        }
        if let Some(name) = line.strip_prefix("dataset ") {
            if name.is_empty() {
                return Err(String::from("the dataset's name is empty"));
            }
            dataset_seen = true;
            continue;
        }
        if let Some(value) = line.strip_prefix("nonce ") {
            nonce = Some(String::from(value));
            continue;
        }
        if let Some(word) = line.strip_prefix("stage ") {
            let Some(found) = Stage::parse_word(word) else {
                return Err(format!("the stage `{word}` is not one this build writes"));
            };
            stage = Some(found);
            continue;
        }
        if let Some(number) = line.strip_prefix("counter ") {
            counter = Some(
                number
                    .parse::<u64>()
                    .map_err(|_error| return String::from("the counter is not a number"))?,
            );
            continue;
        }
        return Err(format!("the line `{line}` is not one this build writes"));
    }
    if !(version_seen && kind_seen && dataset_seen) {
        return Err(String::from(
            "a progress file names its version, its kind and its dataset",
        ));
    }
    let (Some(found_nonce), Some(found_stage), Some(found_counter)) = (nonce, stage, counter)
    else {
        return Err(String::from(
            "a progress file names its nonce, its stage and its counter",
        ));
    };
    return Ok((found_nonce, found_stage, found_counter));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_progress_file_tests.rs"]
mod tests;
