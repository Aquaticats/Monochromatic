//! What:
//!  The receipt a forced removal files for its own answer,
//!  and the filing itself beside the scan file and the progress file.
//! Why:
//!  The wrapper waits on a receipt,
//!  and the forced arm's answer travels in the receipt's own fixed grammar:
//!  the outcome word and,
//!  for a refusal or a failed step,
//!  one bounded failed-step word
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The refusal, and what it leaves",
//!  and F24).
//!  This design adds no key,
//!  so the words are the whole answer and the holders the act ended are the
//!  handler's journal alone,
//!  by the records record.
//!  The filing is the receipts folder's own:
//!  a root-owned `0644` file in a root-owned `0755` folder,
//!  written to a staging sibling and renamed over the dataset's name,
//!  so a waiter reads one whole answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const receipt = receiptOf(answer, bootId(), nowIso());   // the whole file's text
//! folder.publish(receipt);
//! ```

use crate::foundation::iso_time::now_iso;
use crate::worktree_clone_forced_removal::forced_removal_words::{FAILED_STEP_MAXIMUM_BYTES, RefusalWord};
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{CloneMountReceipt, ReceiptOutcome};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::ReceiptsRefusal;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_unmount::staging_calls::BOOT_ID_PATH;

/// What:
///  What one forced act answers,
///  in the receipt's own two words.
/// Why:
///  The outcome says whether the effect was observed and the failed step names
///  which refusal stopped it,
///  and the wrapper reads exactly those two to decide what it prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheActsAnswer = { dataset; nonce; outcome: 'done' | 'refused' | 'failed'; failedStep?: RefusalWord; errno?: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheActsAnswer {
    /// The clone's dataset GUID,
    /// which names the receipt's file.
    pub dataset: DatasetGuid,
    /// The request's nonce.
    pub nonce: RequestNonce,
    /// What the act came to.
    pub outcome: ReceiptOutcome,
    /// The word a refusal or a failed step carries,
    /// which a finished act carries none of.
    pub failed_step: Option<RefusalWord>,
    /// The errno the stopping call answered,
    /// where it answered one.
    pub errno: Option<i32>,
}

/// What:
///  Why no receipt was filed.
/// Why:
///  A word longer than the receipt's own bound,
///  a boot the machine would not name and the folder's own refusal are three
///  different things,
///  and a wrapper waiting on a receipt must be told which one stopped the
///  answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FilingRefusal = { said: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FilingRefusal {
    /// The word the answer names is longer than the receipt's field allows.
    WordTooLong {
        /// The word.
        word: &'static str,
    },
    /// The machine's boot identifier could not be read.
    Boot {
        /// The path that was read.
        path: &'static str,
        /// What the read answered.
        said: String,
    },
    /// The receipts folder refused the write.
    Folder(ReceiptsRefusal),
}

impl std::fmt::Display for FilingRefusal {
    /// One sentence naming what refused.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            FilingRefusal::WordTooLong { word } => {
                write!(formatter, "the failed step `{word}` is longer than the receipt's {FAILED_STEP_MAXIMUM_BYTES} bytes")
            }
            FilingRefusal::Boot { path, said } => write!(formatter, "the boot identifier at {path} could not be read: {said}"),
            FilingRefusal::Folder(refusal) => write!(formatter, "the receipts folder refused the answer: {refusal:?}"),
        };
    }
}

/// What:
///  The receipt one answer becomes.
/// Why:
///  The text is what the waiter parses,
///  so the answer's own fields are the only thing built here and the boot and
///  the time are passed in,
///  which is what lets a control read the exact file the handler would file.
///
/// # Errors
/// The answer's failed step is longer than the receipt's field.
pub fn receipt_of(answer: &TheActsAnswer, boot: &str, time: &str) -> Result<CloneMountReceipt, FilingRefusal> {
    let failed_step: Option<String> = match answer.failed_step {
        Some(word) => {
            let text: &'static str = word.word();
            if text.len() > FAILED_STEP_MAXIMUM_BYTES {
                return Err(FilingRefusal::WordTooLong { word: text });
            }
            Some(String::from(text))
        }
        None => None,
    };
    let errno: Option<String> = answer.errno.map(|number| return number.to_string());
    return Ok(CloneMountReceipt {
        dataset: answer.dataset,
        // The forced act runs under the standing unmount intent, which is the request whose nonce it answers.
        operation: MountOperation::Unmount,
        nonce: answer.nonce.clone(),
        outcome: answer.outcome,
        mount: None,
        failed_step,
        errno,
        boot: String::from(boot),
        time: String::from(time),
    });
}

/// What:
///  File one answer in the receipts folder this boot names.
/// Why:
///  The wrapper waits on this file and on nothing else,
///  so the answer is read back by the folder's own reader before a byte is
///  written,
///  and a refusal here leaves the wrapper's wait to its own deadline rather
///  than to a half-written file.
///
/// # Errors
/// The boot identifier could not be read,
/// the word is too long for the receipts field,
/// or the folder refused the write.
pub fn file_the_receipt(folder: &ReceiptsFolder, answer: &TheActsAnswer) -> Result<(), FilingRefusal> {
    let boot: String = std::fs::read_to_string(BOOT_ID_PATH)
        .map(|text| return text.trim().to_string())
        .map_err(|error| return FilingRefusal::Boot { path: BOOT_ID_PATH, said: error.to_string() })?;
    let receipt: CloneMountReceipt = receipt_of(answer, boot.as_str(), now_iso().as_str())?;
    return folder.publish(&receipt).map_err(|refusal| return FilingRefusal::Folder(refusal));
}

/// What:
///  The outcome word and the failed step a refusal is filed with.
/// Why:
///  Every refusal the design names is a `refused` answer with its word,
///  and one place makes that pair so no caller files a refusal as `failed`.
pub fn refusal_answer(dataset: DatasetGuid, nonce: RequestNonce, word: RefusalWord, errno: Option<i32>) -> TheActsAnswer {
    return TheActsAnswer { dataset, nonce, outcome: ReceiptOutcome::Refused, failed_step: Some(word), errno };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "forced_receipt_tests.rs"]
mod tests;
