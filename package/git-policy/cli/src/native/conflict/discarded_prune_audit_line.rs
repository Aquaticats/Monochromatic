//! What:
//!  The audit line one removed directory of saved copies leaves: the outcome, the reason, the limit, the sizes and the copy names.
//! Why:
//!  `discarded_prune.rs` lists, plans and removes and stays under the file-size limit; the text of the line depends only on
//!  what was removed and the limits, so it lives in a file of its own that `discarded_prune.rs` imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = pruneLine(removed, copies, limits, folderBytes, nowMillis); // JSON text ended by a line feed
//! ```

use crate::discarded_prune_limits::Limits;
use crate::discarded_prune_plan::Reason;
use crate::iso_time::format_iso_milliseconds;
use crate::json_record::{quote, string_array};
use crate::service_protocol::RELEASE;
use std::path::PathBuf;

/// How many copy names one audit line lists; the line also states the total.
///  Exempt from a citation under `LQM` (class: display cap): it limits how much text a message, log field or record
///  names, and no command result or duration depends on it. Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`, "Exempt limits"; the coordinator's pick under the owner's delegation, open to the owner's veto.
const LISTED_COPIES: usize = 100;

/// One removed directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Removed {
    /// The directory that was removed.
    pub path: PathBuf,
    /// Its bytes.
    pub bytes: u128,
    /// Why.
    pub reason: Reason,
}

/// What:
///  The audit line for one removed directory: the time, the path, the bytes, the reason, the limit it was judged against,
///  the size of the folder before the pass, and the names of the copies in it.
/// Why:
///   A person looking for a file that vanished finds here that the wrapper removed its copy, when, and by which rule. The
///  common fields are those of the other audit lines, so the file reads the same throughout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ schemaVersion: 1, time, form: 'PruneDiscarded', outcome: 'pruned', reason, path, bytes, copies }) + '\n'
/// ```
pub fn prune_line(removed: &Removed, copies: &[String], limits: Limits, folder_bytes: u128, now_millis: i64) -> String {
    return audit_line("pruned", removed, copies, limits, folder_bytes, now_millis);
}

/// The audit line of one directory with the outcome word `pruned` (all of it went) or `partial` (some of it went and the
/// rest was kept).
pub fn audit_line(outcome: &str, removed: &Removed, copies: &[String], limits: Limits, folder_bytes: u128, now_millis: i64) -> String {
    let limit: u64 = match removed.reason {
        Reason::Age => limits.max_age_days,
        Reason::Size => limits.max_bytes,
    };
    let listed: Vec<String> = copies.iter().take(LISTED_COPIES).cloned().collect();
    return format!(
        "{{\"schemaVersion\":1,\"time\":{},\"level\":\"info\",\"timeMillis\":{},\"release\":{},\"pid\":{},\"form\":\"PruneDiscarded\",\
         \"outcome\":{},\"reason\":{},\"limit\":{limit},\"folderBytes\":{folder_bytes},\"path\":{},\"bytes\":{},\
         \"copiesTotal\":{},\"copies\":{}}}\n",
        quote(&format_iso_milliseconds(now_millis)),
        quote(&now_millis.to_string()),
        quote(RELEASE),
        std::process::id(),
        quote(outcome),
        quote(removed.reason.word()),
        quote(&removed.path.to_string_lossy()),
        removed.bytes,
        copies.len(),
        string_array(&listed),
    );
}
