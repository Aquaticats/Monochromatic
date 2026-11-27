//! What:
//!  The five lines one progress file holds:
//!  its version and kind,
//!  the dataset it belongs to,
//!  the request's nonce,
//!  the stage the act last wrote and the counter it last published,
//!  with the writer that builds them and the reader that takes them apart.
//! Why:
//!  A half-written or foreign file must be refused by its own grammar rather
//!  than read as a counter,
//!  which is what keeps a torn file from reading as a dead handler,
//!  and the reader must find the values by name,
//!  so the two halves of the grammar sit together
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The act's liveness",
//!  L21).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = progressText(dataset, nonce, stage, counter);   // six lines plus their newline
//! const [nonceBack, stageBack, counterBack] = parseProgress(text);
//! ```

use crate::worktree_clone_forced_removal::forced_removal_words::Stage;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

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
