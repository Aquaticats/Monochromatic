//! What:
//!   The ref of its own that keeps a capture's objects from `git gc`,
//!   and the dropping of that ref once nothing needs it.
//! Why:
//!   A blob nothing points at is collected,
//! so the capture's objects are written as one commit on `refs/cli-git/carried/<n>` before the checkout changes,
//! and the record of the merge names that ref and the bytes.
//! A ref left behind is a name a person has to clean up by hand,
//! so it is dropped by the pass that removes the last record whose carry still needs it,
//! and a deletion that failed leaves the record in place so a later command retries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = await keepCapture(context, root, capture); // keeps refs/cli-git/carried/<n>
//! await dropCaptureRef(context, gitDir, root, kept, finishing);
//! ```

/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// Real Git,
/// on the runtime that may already be entered.
use crate::lazy_runtime::run_on_runtime;
/// One run of Git whose arguments carry bytes that need not be text.
use crate::merge_isolation_carry_objects::run_bytes_awaiting;
/// The captured work,
/// and the slots that keep it.
use crate::merge_isolation_carry_parts::{
    Capture,
    slots,
};
use crate::transaction_git::{
    GitContext,
    GitOutput,
};
/// Owned operating-system text.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// The tag of this module in the diagnostic log.
const TAG: &str = "mergeCarry";

/// The prefix of the refs that keep a capture,
/// beside nothing else.
pub const CARRIED_REF_PREFIX: &str = "refs/cli-git/carried/";

/// How many ref names are tried before the capture is refused.
///  A name carries milliseconds and a process number,
/// so a collision means another merge in the same millisecond;
///  eight tries is a bound,
/// not a loop.
const REF_ATTEMPTS: u128 = 8;

/// What:
///  The words for a clock this process could not read.
/// Why:
///   The capture's reference names the moment it was made,
///  and a clock before the epoch cannot say when that was.
fn the_clock_could_not_be_read(error: std::time::SystemTimeError) -> String {
    return format!("the clock could not be read: {error}");
}

/// What:
///  The tree that holds every captured object under a slot name.
/// Why:
///   A flat list of numbered entries is a tree `git mktree` writes without a scratch index,
/// and the slot names carry
///  no path,
/// so a path that cannot be a tree entry still captures its bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureTree(context, root, capture): Promise<string>;
/// ```
async fn capture_tree_awaiting(
    context: &GitContext,
    root: &Path,
    capture: &Capture,
) -> Result<String, String> {
    let mut listing: Vec<u8> = Vec::new();
    for slot in slots(capture.paths.as_slice()) {
        listing.extend_from_slice(format!("100644 blob {}\t{}\n", slot.oid, slot.name).as_bytes());
    }
    let arguments: Vec<OsString> = vec![OsString::from("mktree")];
    let output: GitOutput =
        run_bytes_awaiting(context, root, arguments.as_slice(), Some(listing)).await?;
    if !output.succeeded() {
        return Err(format!(
            "the capture's tree could not be written: {}",
            output.error_text()
        ));
    }
    return Ok(String::from_utf8_lossy(output.stdout.as_slice())
        .trim()
        .to_owned());
}

/// What:
///  Write the capture's objects,
/// one commit on a ref of its own,
/// and return the capture with the commit and ref that hold it.
/// Why:
///   Until the ref exists the bytes are loose objects nothing points at,
/// and a `git gc --prune=now` would
///  collect them;
/// the ref is written before the first byte of the checkout changes,
/// and the record is written
///  between the two,
/// so a kill leaves either loose objects with a record that names them or a ref that
///  keeps them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function keepCapture({ context, root, capture }): Promise<Capture>;
/// ```
pub fn keep_capture(
    context: &GitContext,
    root: &Path,
    capture: &Capture,
) -> Result<Capture, String> {
    return match run_on_runtime(keep_capture_awaiting(context, root, capture)) {
        Ok(found) => found,
        Err(refusal) => Err(format!("{refusal}")),
    };
}

/// What:
///  The same commit on a ref of its own,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The tree,
/// the commit and the ref are each a Git step,
/// and a synchronous start registers the step's pipes by entering the runtime,
///  which is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function keepCapture({ context, root, capture }): Promise<Capture>;
/// ```
pub async fn keep_capture_awaiting(
    context: &GitContext,
    root: &Path,
    capture: &Capture,
) -> Result<Capture, String> {
    let tree: String = capture_tree_awaiting(context, root, capture).await?;
    let mut arguments: Vec<OsString> =
        vec![OsString::from("commit-tree"), OsString::from(tree.as_str())];
    arguments.push(OsString::from("-m"));
    arguments.push(OsString::from(
        "monochromatic-git-policy-cli: the work a merge carries",
    ));
    let created: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await?;
    if !created.succeeded() {
        return Err(format!(
            "the capture could not be committed: {}",
            created.error_text()
        ));
    }
    let commit: String = String::from_utf8_lossy(created.stdout.as_slice())
        .trim()
        .to_owned();
    let first: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(the_clock_could_not_be_read)?
        .as_millis();
    let zeros: OsString = OsString::from("0".repeat(commit.len()).as_str());
    for step in 0..REF_ATTEMPTS {
        let reference: String = format!(
            "{CARRIED_REF_PREFIX}{}-{}-{step}",
            first,
            std::process::id()
        );
        let mut write: Vec<OsString> = vec![
            OsString::from("update-ref"),
            OsString::from("--no-deref"),
            OsString::from(reference.as_str()),
            OsString::from(commit.as_str()),
            zeros.clone(),
        ];
        let written: GitOutput = run_bytes_awaiting(context, root, write.as_slice(), None).await?;
        if written.succeeded() {
            return Ok(Capture {
                commit,
                reference: Some(reference),
                paths: capture.paths.clone(),
            });
        }
        debug(
            TAG,
            format!(
                "the capture could not take {reference}: {}",
                written.error_text()
            )
            .as_str(),
        );
        write.clear();
    }
    return Err(String::from("no name for the capture was free"));
}

/// What:
///  Whether an intent record that still needs this capture names it:
/// one whose carry has not finished.
/// Why:
///  A merge that saves its result for a later finish writes a second record beside the first,
/// and both name one capture
/// (`save_for_a_later_finish`).
/// The capture's ref is that record's claim on the person's bytes while its carry is still to run:
/// `git gc` collects a commit nothing
///  points at,
/// and the record is what finishes the merge later
/// (a qwen4-max-preview audit of the saved merge,
/// its second finding).
/// Once a record's carry has finished,
/// its bytes are in the checkout and it needs nothing from the objects;
/// its sibling marker says so (`merge_isolation_journal.rs`,
/// `mark_carry_finished`),
/// and it must not hold the ref for the other record,
/// or two records whose carries have both finished each keep the other's ref alive for ever
/// (an audit of 0.6.1-alpha.14,
/// its third finding),
/// and a pass that removes its own record while counting on this one leaves the ref with nobody to drop it
/// (the second review of this branch,
/// its second finding).
/// A listing that failed proves nothing,
/// so it answers yes and the ref stays:
/// a name that outlives its bytes costs a person one `rm`,
/// a name dropped too early costs the bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anotherRecordStillNeedsTheCapture(gitDir, capture, finishing) { return listMergeIntents().some((one) => one.path !== finishing && !existsSync(settledOf(one)) && one.carried?.reference === capture.reference) }
/// ```
pub(crate) fn another_record_still_needs_the_capture(
    git_dir: &Path,
    capture: &Capture,
    finishing: Option<&Path>,
) -> bool {
    let Some(reference) = capture.reference.as_deref() else {
        return false;
    };
    let Ok(records) = crate::merge_isolation_journal::list_merge_intents(git_dir) else {
        return true;
    };
    for record in records {
        // The record this command is finishing is not another one, and a caller that deletes the refs before the record
        //  names it here: the ref is only left standing when a record other than that one still needs it.
        if finishing == Some(record.as_path()) {
            continue;
        }
        // A record whose carry has finished needs nothing from the capture, so it is no claim on the ref.
        if crate::merge_isolation_journal::carry_finished(record.as_path()) {
            continue;
        }
        // A record that is a pipe is no record this command can read, so it is no claim on the ref and the
        //  read answers at once rather than waiting for a writer (`crate::plain_file`).
        let Ok(bytes) = crate::plain_file::read_plain_file(
            record.as_path(),
            crate::plain_file::SMALL_FILE_LIMIT,
        ) else {
            continue;
        };
        let Some(intent) = crate::merge_isolation_journal::parse_merge_intent(bytes.as_slice())
        else {
            continue;
        };
        let Some(carried) = intent.carried.as_ref() else {
            continue;
        };
        if carried.reference.as_deref() == Some(reference) && carried.commit == capture.commit {
            return true;
        }
    }
    return false;
}

/// What:
///  Drop the ref that kept a capture,
/// once nothing needs its objects,
/// and say whether the name is gone.
/// Why:
///   A ref left behind keeps the person's bytes reachable for ever,
/// and a name pointing at nothing anybody uses is
///  what a person has to clean up by hand.
///  The answer is what lets the caller keep the merge's record when the name stayed:
/// the record is the only thing
///  that names the ref,
/// so a later command's recovery retries the deletion instead of the name being
///  orphaned (a cross-vendor audit of alpha.12,
/// "Capture-ref cleanup is never retried after a transient failure").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropCaptureRef({ context, root, capture }): Promise<boolean>;
/// ```
pub fn drop_capture_ref(
    context: &GitContext,
    git_dir: &Path,
    root: &Path,
    capture: &Capture,
    finishing: Option<&Path>,
) -> bool {
    return match run_on_runtime(drop_capture_ref_awaiting(
        context, git_dir, root, capture, finishing,
    )) {
        Ok(dropped) => dropped,
        // A deletion that could not be started leaves the ref where it is, which is what the awaited face reports too.
        Err(refusal) => {
            debug(
                TAG,
                format!("the capture's ref was left where it is: {refusal}").as_str(),
            );
            false
        }
    };
}

/// What:
///  Whether a ref names nothing at all,
///  asked after a deletion of it was refused.
/// Why:
///   A deletion that compares the old value is refused when the name is already gone,
/// and that state is what the deletion wanted:
/// a kill between dropping a record's ref and removing the record leaves exactly it,
/// and a pass that retried the deletion for ever kept the record for ever,
/// so every later command printed that a ref its record keeps could not be removed and,
/// once `git gc --prune=now` had collected the capture's objects,
/// refused every command through the carry
/// (the second review of this branch,
/// its first finding).
/// The answer is read from Git rather than from the error text,
/// so a Git that words the refusal differently still answers,
/// and a read that fails is not proof of absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function theRefNamesNothing(context, root, name): Promise<boolean>; // for-each-ref --count=1 --format=%(objectname) <name>
/// ```
async fn the_ref_names_nothing(
    context: &GitContext,
    root: &Path,
    name: &str,
) -> bool {
    let arguments: Vec<OsString> = vec![
        OsString::from("for-each-ref"),
        OsString::from("--count=1"),
        OsString::from("--format=%(objectname)"),
        OsString::from(name),
    ];
    return match run_bytes_awaiting(context, root, arguments.as_slice(), None).await {
        Ok(output) if output.succeeded() => output.stdout.iter().all(|byte| {
            return byte.is_ascii_whitespace();
        }),
        Ok(_) | Err(_) => false,
    };
}

/// What:
///  Drop the ref that kept a capture,
/// once nothing needs its objects,
/// and answer whether this record may go.
/// Why:
///   A deletion that fails is logged and answered false,
/// so the caller keeps the record that names the ref and the next command retries
/// (a cross-vendor audit of alpha.12,
/// "Capture-ref cleanup is never retried after a transient failure").
/// Two answers are successes even though nothing was deleted:
/// a name another record still needs
/// (`another_record_still_needs_the_capture`),
/// whose own finish drops it,
/// and a name already gone,
/// which is what this deletion wanted
/// (`the_ref_names_nothing`).
/// The two together are what make the order of two commands' finishes immaterial:
/// whichever pass removes the last record that needs the capture deletes the ref,
/// and one that finds it deleted has done its part.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropCaptureRef({ context, root, capture }): Promise<boolean>;
/// ```
pub async fn drop_capture_ref_awaiting(
    context: &GitContext,
    git_dir: &Path,
    root: &Path,
    capture: &Capture,
    finishing: Option<&Path>,
) -> bool {
    let Some(reference) = capture.reference.as_deref() else {
        return true;
    };
    // The record this command is finishing may not be the only one whose carry still names this capture. While one does, its
    //  bytes are the person's only copy of that work, so the name stays and that record's finish drops it.
    if another_record_still_needs_the_capture(git_dir, capture, finishing) {
        debug(TAG, format!("the capture's ref {reference} is still needed by another merge record, which drops it").as_str());
        return true;
    }
    let arguments: Vec<OsString> = vec![
        OsString::from("update-ref"),
        OsString::from("--no-deref"),
        OsString::from("-d"),
        OsString::from(reference),
        OsString::from(capture.commit.as_str()),
    ];
    return match run_bytes_awaiting(context, root, arguments.as_slice(), None).await {
        Ok(output) if output.succeeded() => true,
        Ok(output) => {
            if the_ref_names_nothing(context, root, reference).await {
                debug(TAG, format!("the capture's ref {reference} was already gone, which is what its deletion wanted: {}", output.error_text()).as_str());
                return true;
            }
            debug(
                TAG,
                format!(
                    "the capture's ref {reference} was left where it is: {}",
                    output.error_text()
                )
                .as_str(),
            );
            false
        }
        Err(error) => {
            if the_ref_names_nothing(context, root, reference).await {
                debug(TAG, format!("the capture's ref {reference} was already gone, which is what its deletion wanted: {error}").as_str());
                return true;
            }
            debug(
                TAG,
                format!("the capture's ref {reference} was left where it is: {error}").as_str(),
            );
            false
        }
    };
}
