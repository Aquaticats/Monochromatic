//! What:
//! The command `cargo-shadow acknowledge <journal directory>`:
//! a person asserts that every process or worker of the run that left the journal has stopped.
//! Why:
//! Contract `doc/planning/cargo-shadow-guarantees.md`,
//! "The weaker path":
//! a journal of a run without a delegated control group,
//! or whose containment cannot be examined,
//! is refused by every recovery and
//! every new transaction until a person acknowledges it.
//! The acknowledgement is written under the workspace lock,
//! to one journal,
//! by an atomic publication,
//! and it overrides only the uncertainty of the containment context:
//! a recorded scope or process group that is positively observed to have a member still blocks,
//! and every other check of the recovery (listing,
//! snapshots,
//! durability) still runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // acknowledge.ts
//! export function acknowledge(journal: string): string  // throws the reason it did not
//! ```

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

/// Imports `std::time::{SystemTime, UNIX_EPOCH}` for this file.
use std::time::{SystemTime, UNIX_EPOCH};

/// Imports the journal names and the publication helpers for this file.
use crate::journal::{ACKNOWLEDGED_FILE, ACKNOWLEDGED_HEADER, DONE_SUFFIX, JOURNAL_DIR, atomic_write, sync_directory};

/// Imports the workspace lock for this file.
use crate::journal_transaction::take_workspace;

/// What:
/// `fn workspace_of(journal: &Path) -> Result<PathBuf, String>` finds the workspace root of a journal directory.
/// Why:
/// A journal lives at `<root>/.cache/cargo-shadow/journal/<name>`;
/// anything else is not a journal this command acknowledges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function workspaceOf(journal: string): string  // throws the reason
/// ```
fn workspace_of(journal: &Path) -> Result<PathBuf, String> {
    let parent: &Path = journal.parent().ok_or_else(|| { return format!("{} has no parent directory", journal.display()); })?;
    let mut root: PathBuf = parent.to_path_buf();
    for _component in Path::new(JOURNAL_DIR).components() {
        if !root.pop() {
            return Err(format!("{} is not a journal directory of a workspace (expected <root>/{JOURNAL_DIR}/<name>)", journal.display()));
        }
    }
    if parent != root.join(JOURNAL_DIR) {
        return Err(format!("{} is not a journal directory of a workspace (expected <root>/{JOURNAL_DIR}/<name>)", journal.display()));
    }
    return Ok(root);
}

/// What:
/// `pub fn acknowledge(journal: &Path) -> Result<String, String>` acknowledges exactly one journal and returns what to tell the
/// person.
/// Why:
/// The journal is named by its directory,
/// resolved first so a link cannot point the acknowledgement at another journal.
/// The workspace lock is tried,
/// never waited for.
/// The file is published whole by an atomic rename and the directory is synced,
/// so an interrupted publication leaves the journal
/// unacknowledged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function acknowledge(journal: string): string  // throws the reason
/// ```
pub fn acknowledge(journal: &Path) -> Result<String, String> {
    let resolved: PathBuf = std::fs::canonicalize(journal).map_err(|error| { return format!("{} cannot be examined ({error})", journal.display()); })?;
    if resolved.to_string_lossy().ends_with(DONE_SUFFIX) {
        return Err(format!("{} is a journal that is already finished", resolved.display()));
    }
    if !resolved.is_dir() {
        return Err(format!("{} is not a directory", resolved.display()));
    }
    let root: PathBuf = workspace_of(&resolved)?;
    let _workspace = match take_workspace(&root) {
        Ok(Ok(held)) => held,
        Ok(Err(holder)) => return Err(format!("{holder}; nothing was acknowledged")),
        Err(error) => return Err(format!("the workspace lock could not be tried ({error}); nothing was acknowledged")),
    };
    let seconds: u64 = SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| { return elapsed.as_secs(); }).unwrap_or(0);
    let text: String = format!("{ACKNOWLEDGED_HEADER}\n{}\n{seconds}\n", std::process::id());
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::AcknowledgementPublish) {
        return Err("the acknowledgement could not be written (injected failure); nothing was acknowledged".to_string());
    }
    atomic_write(&resolved.join(ACKNOWLEDGED_FILE), &text).map_err(|error| { return format!("the acknowledgement could not be written ({error}); nothing was acknowledged"); })?;
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::AcknowledgementDurability) {
        return Err("the acknowledgement could not be made durable (injected failure); it counts only once it is, so run the command again".to_string());
    }
    sync_directory(&resolved).map_err(|error| { return format!("the acknowledgement could not be made durable ({error}); it counts only once it is, so run the command again"); })?;
    return Ok(format!(
        "acknowledged {}: you asserted that every process or worker of that run that can modify the workspace has stopped; the next cargo-shadow run settles the journal",
        resolved.display()
    ));
}
