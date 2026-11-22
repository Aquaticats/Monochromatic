//! What:
//!  Reading one receipt file's text:
//!  the byte bound before any line,
//!  the exact key at each line,
//!  each value's own rule,
//!  and the mount fields that belong only to a done mount.
//! Why:
//!  The caller acts on this answer,
//!  so nothing is defaulted and no extra line is skipped:
//!  the keys are read in order,
//!  an absent mount field is `none` or a refusal,
//!  and a stale file is told apart from the answer by the nonce,
//!  the operation and the boot the caller compares afterwards
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  The bound comes first so a huge file is refused without being split into
//!  lines.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const receipt = parseReceipt(text); // throws ReceiptRefusal
//! ```

use crate::foundation::iso_time::instant_of_iso;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{
    ABSENT_WORD, CloneMountReceipt, MAXIMUM_DATASET_NAME_BYTES, MAXIMUM_PATH_BYTES, MAXIMUM_WORD_BYTES, RECEIPT_KEYS, RECEIPT_KIND,
    RECEIPT_VERSION, ReceiptMountIdentity, ReceiptOutcome,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipt_refusals::ReceiptRefusal;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// Read one receipt's text.
///
/// # Errors
/// The text is over the byte bound,
/// not text,
/// or any line's key or value does not fit the receipt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const receipt = parseReceipt(text);
/// ```
pub fn parse_receipt(text: &str) -> Result<CloneMountReceipt, ReceiptRefusal> {
    let bound: usize = crate::worktree_clone_mount_protocol::clone_mount_receipt::MAXIMUM_RECEIPT_BYTES;
    if text.len() > bound {
        return Err(ReceiptRefusal::TooLarge { bytes: text.len(), bound });
    }
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() != RECEIPT_KEYS.len() {
        return Err(ReceiptRefusal::LineCount { found: lines.len(), expected: RECEIPT_KEYS.len() });
    }
    if lines[0] != RECEIPT_VERSION {
        return Err(ReceiptRefusal::Key { number: 1, expected: RECEIPT_VERSION, found: String::from(lines[0]) });
    }
    let kind: &str = value_of(&lines, 1, RECEIPT_KEYS[1])?;
    if kind != RECEIPT_KIND {
        return Err(ReceiptRefusal::Value { key: RECEIPT_KEYS[1], shown: String::from(kind) });
    }
    let dataset: DatasetGuid = DatasetGuid::parse(value_of(&lines, 2, RECEIPT_KEYS[2])?).map_err(ReceiptRefusal::Dataset)?;
    let nonce: RequestNonce = RequestNonce::parse(value_of(&lines, 3, RECEIPT_KEYS[3])?).map_err(ReceiptRefusal::Nonce)?;
    let operation: MountOperation = MountOperation::from_word(value_of(&lines, 4, RECEIPT_KEYS[4])?)
        .ok_or_else(|| return ReceiptRefusal::Value { key: RECEIPT_KEYS[4], shown: String::from(value_of(&lines, 4, RECEIPT_KEYS[4]).unwrap_or_default()) })?;
    let outcome_word: &str = value_of(&lines, 5, RECEIPT_KEYS[5])?;
    let outcome: ReceiptOutcome =
        ReceiptOutcome::from_word(outcome_word).ok_or_else(|| return ReceiptRefusal::Value { key: RECEIPT_KEYS[5], shown: String::from(outcome_word) })?;
    let mount: Option<ReceiptMountIdentity> = read_mount(&lines, outcome, operation)?;
    let failed_step: Option<String> = read_word(&lines, 11, RECEIPT_KEYS[11])?;
    let errno: Option<String> = read_word(&lines, 12, RECEIPT_KEYS[12])?;
    check_outcome_words(outcome, &failed_step, &errno)?;
    let boot: &str = value_of(&lines, 13, RECEIPT_KEYS[13])?;
    if !crate::foundation::random_id::is_canonical_uuid(boot) {
        return Err(ReceiptRefusal::Value { key: RECEIPT_KEYS[13], shown: String::from(boot) });
    }
    let time: &str = value_of(&lines, 14, RECEIPT_KEYS[14])?;
    if instant_of_iso(time).is_none() {
        return Err(ReceiptRefusal::Time { value: String::from(time) });
    }
    return Ok(CloneMountReceipt {
        dataset,
        operation,
        nonce,
        outcome,
        mount,
        failed_step,
        errno,
        boot: String::from(boot),
        time: String::from(time),
    });
}

/// What:
///  The value after the key at one place.
/// Why:
///   Every line is `<key> <value>`,
///  and a line whose key differs is refused rather than searched for,
///  so a reordered or foreign file cannot be read as a receipt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = valueOf(lines, 2, 'dataset');
/// ```
fn value_of<'a>(lines: &[&'a str], index: usize, key: &'static str) -> Result<&'a str, ReceiptRefusal> {
    let line: &'a str = lines[index];
    let Some((found, value)) = line.split_once(' ') else {
        return Err(ReceiptRefusal::Key { number: index + 1, expected: key, found: String::from(line) });
    };
    if found != key {
        return Err(ReceiptRefusal::Key { number: index + 1, expected: key, found: String::from(found) });
    }
    if value.is_empty() {
        return Err(ReceiptRefusal::Value { key, shown: String::from("<empty>") });
    }
    return Ok(value);
}

/// What:
///  One optional step or errno word.
/// Why:
///   The two fields name a failure and nothing else,
///  so `none` is the only word without content and any other is checked
///  to a bounded set of printable characters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const word = readWord(lines, 11, 'failed-step'); // string | null
/// ```
fn read_word(lines: &[&str], index: usize, key: &'static str) -> Result<Option<String>, ReceiptRefusal> {
    let value: &str = value_of(lines, index, key)?;
    if value == ABSENT_WORD {
        return Ok(None);
    }
    if value.len() > MAXIMUM_WORD_BYTES || !value.bytes().all(|byte| return byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_') {
        return Err(ReceiptRefusal::Value { key, shown: String::from(value) });
    }
    return Ok(Some(String::from(value)));
}

/// What:
///  The mount identity a done mount carries and the absence every other result
///  carries.
/// Why:
///   A mount identity on a refused or failed result would invite a caller to
///  confirm a mount the handler never reported,
///  and a done mount without one cannot be confirmed.
///  An unmount that verified absence has no mount to identify,
///  so a done unmount carries `none` there,
///  and its receipt names the clone's GUID alone:
///  the design asks for the identity "where applicable"
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Requests,
/// results and ordering").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mount = readMount(lines, outcome, operation); // identity | null
/// ```
fn read_mount(lines: &[&str], outcome: ReceiptOutcome, operation: MountOperation) -> Result<Option<ReceiptMountIdentity>, ReceiptRefusal> {
    let wanted: bool = outcome == ReceiptOutcome::Done && operation == MountOperation::Mount;
    let present_keys: [&'static str; 5] = [RECEIPT_KEYS[6], RECEIPT_KEYS[7], RECEIPT_KEYS[8], RECEIPT_KEYS[9], RECEIPT_KEYS[10]];
    let words: [&str; 5] = [
        value_of(lines, 6, present_keys[0])?,
        value_of(lines, 7, present_keys[1])?,
        value_of(lines, 8, present_keys[2])?,
        value_of(lines, 9, present_keys[3])?,
        value_of(lines, 10, present_keys[4])?,
    ];
    for (key, word) in present_keys.iter().zip(words.iter()) {
        if (*word == ABSENT_WORD) == wanted {
            return Err(ReceiptRefusal::MountField { key, wanted });
        }
    }
    if !wanted {
        return Ok(None);
    }
    let mount_id: u64 = decimal_of(present_keys[0], words[0])?;
    let device: u64 = decimal_of(present_keys[1], words[1])?;
    let inode: u64 = decimal_of(present_keys[2], words[2])?;
    let source: String = dataset_name_of(present_keys[3], words[3])?;
    let canonical_path: String = text_of(present_keys[4], words[4], MAXIMUM_PATH_BYTES)?;
    if !canonical_path.starts_with('/') {
        return Err(ReceiptRefusal::Value { key: RECEIPT_KEYS[10], shown: canonical_path });
    }
    return Ok(Some(ReceiptMountIdentity { mount_id, device, inode, source, canonical_path }));
}

/// What:
///  A decimal count or the refusal naming its field.
/// Why:
///   The mount identity's three numbers are the kernel's own,
///  and a sign or a prefix would name another number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const count = decimalOf('mount-id', '41');
/// ```
fn decimal_of(key: &'static str, word: &str) -> Result<u64, ReceiptRefusal> {
    if word.is_empty() || !word.bytes().all(|byte| return byte.is_ascii_digit()) {
        return Err(ReceiptRefusal::Value { key, shown: String::from(word) });
    }
    return word.parse::<u64>().map_err(|_| return ReceiptRefusal::Value { key, shown: String::from(word) });
}

/// What:
///  A bounded dataset name.
/// Why:
///   The source is a ZFS dataset's own name,
///  whose letters,
///  digits and `_-.:/@` are the kernel's alphabet,
///  so a space or a control character is another value and no name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = datasetNameOf('source', word);
/// ```
fn dataset_name_of(key: &'static str, word: &str) -> Result<String, ReceiptRefusal> {
    let allowed = |byte: u8| return byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':' | b'/' | b'@' | b'$' | b'%');
    if word.is_empty() || word.len() > MAXIMUM_DATASET_NAME_BYTES || !word.bytes().all(allowed) || word.starts_with('/') {
        return Err(ReceiptRefusal::Value { key, shown: String::from(word) });
    }
    return Ok(String::from(word));
}

/// What:
///  A bounded text value with no control character.
/// Why:
///   The source and the canonical path are written one per line,
///  and a control character or a newline would break the file's own shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = textOf('source', word, 255);
/// ```
fn text_of(key: &'static str, word: &str, bound: usize) -> Result<String, ReceiptRefusal> {
    if word.is_empty() || word.len() > bound || word.chars().any(char::is_control) {
        return Err(ReceiptRefusal::Value { key, shown: String::from(word) });
    }
    return Ok(String::from(word));
}

/// What:
///  Whether a result's outcome agrees with its failure words.
/// Why:
///   A done result names no failing step,
///  and a refused or failed one names the step,
///  so the two fields cannot quietly disagree with the outcome.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkOutcomeWords(outcome, failedStep, errno); // throws
/// ```
fn check_outcome_words(outcome: ReceiptOutcome, failed_step: &Option<String>, errno: &Option<String>) -> Result<(), ReceiptRefusal> {
    return match outcome {
        ReceiptOutcome::Done | ReceiptOutcome::Reattached => {
            if failed_step.is_some() {
                return Err(ReceiptRefusal::Value { key: RECEIPT_KEYS[11], shown: String::from("a step on a done result") });
            }
            if errno.is_some() {
                return Err(ReceiptRefusal::Value { key: RECEIPT_KEYS[12], shown: String::from("an errno on a done result") });
            }
            Ok(())
        }
        ReceiptOutcome::Refused | ReceiptOutcome::Failed => {
            if failed_step.is_none() {
                return Err(ReceiptRefusal::MountField { key: RECEIPT_KEYS[11], wanted: true });
            }
            Ok(())
        }
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_receipt_parse_tests.rs"]
mod tests;
