//! What:
//! The context one journal's containment records were made in:
//! whether the record is there at all,
//! the boot and machine identity it was written under,
//! and the mount and inode view its directory had when it was written.
//! Why:
//! `journal_unready_assessment.rs` judges the records themselves;
//! reading the record and its context was split out of it to keep each file under the code-line limit.
//! A reboot of the same machine ends every process of the run,
//! and a journal seen through another mount view may not hold the scopes it recorded,
//! so both are answered before any record is judged.
//! `Finding` lives here and is re-exported from `journal_unready_assessment`,
//! so its path there is unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-unready-record-context.ts
//! type RecordContext = { text: string; uncertain: string | undefined }
//! function recordAndContext(directory: string, hint: string): RecordContext  // throws the verdict that ends the assessment
//! ```

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the control group records and queries for this file.
use crate::control_group::{
    Identity,
    Record,
    current_identity,
    parse_record,
    populated_at,
    read_identity_text,
    view_of,
};

/// Imports the journal file names for this file.
use crate::journal::{
    GROUPS_FILE,
    VIEW_FILE,
};

/// Imports the verdict on a journal that has no ready record for this file.
use crate::journal_unready_verdict::note_uncertain;

/// What a check of one journal found.
pub(crate) enum Finding {
    /// A recorded scope or process group is positively observed to have a member:
    /// nothing overrides this.
    Live(
        /// The message.
        String,
    ),
    /// The containment context cannot establish that the run's writers stopped:
    /// an acknowledgement overrides this.
    Uncertain(
        /// The message.
        String,
    ),
}

/// The record of one journal,
/// with the uncertainty its context already raised.
pub(crate) struct RecordContext {
    /// The texts of the containment records,
    /// one record per line.
    pub(crate) text: String,
    /// What the journal's own making left uncertain,
    /// or `None` when nothing did.
    pub(crate) uncertain: Option<String>,
}

/// What:
/// `fn record_and_context(directory: &Path, hint: &str) -> Result<RecordContext, Option<Finding>>` reads one journal's
/// containment record and the context it was written in.
/// Why:
/// The caller judges the records only when this answers `Ok`;
/// an `Err` is the verdict the whole assessment gives,
/// because a missing record or a reboot decided it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordAndContext(directory: string, hint: string): RecordContext  // throws Finding | undefined
/// ```
pub(crate) fn record_and_context(
    directory: &Path,
    hint: &str,
) -> Result<RecordContext, Option<Finding>> {
    let path: PathBuf = directory.join(GROUPS_FILE);
    let text: String = match fs::read_to_string(&path) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(match fs::read_dir(directory) {
                Ok(mut entries) => match entries.next() {
                    None => None,
                    Some(Ok(_)) => Some(Finding::Uncertain(format!(
                        "{}: the record of the containment of the killed run is missing, so a child of that run may still be writing; {hint}",
                        directory.display()
                    ))),
                    Some(Err(listing_error)) => Some(Finding::Uncertain(format!(
                        "{}: the journal could not be listed ({listing_error}), so a child of the killed run may still be writing; {hint}",
                        directory.display()
                    ))),
                },
                Err(listing_error) => Some(Finding::Uncertain(format!(
                    "{}: the journal could not be listed ({listing_error}), so a child of the killed run may still be writing; {hint}",
                    directory.display()
                ))),
            });
        }
        Err(error) => {
            return Err(Some(Finding::Uncertain(format!(
                "{}: the record of the containment of the killed run could not be read ({error}), so a child of that run may still be writing; {hint}",
                directory.display()
            ))));
        }
    };
    // An empty record shows that no child ever started.
    if text.lines().next().is_none() {
        return Err(None);
    }
    let mut uncertain: Option<String> = None;
    let view_text: io::Result<String> = fs::read_to_string(directory.join(VIEW_FILE));
    // A reboot of the same machine ended every process of the run (assumption a: the run's writers are inside it, and a boot ends them
    // all). The journal was made by one process in one boot, so its identity covers every record. A different boot id with the same
    // machine id discharges only the question whether the run's processes live; every other check of a recovery still runs.
    let recorded_identity: Option<Identity> = match view_text.as_ref() {
        Ok(recorded) => read_identity_text(recorded),
        Err(_) => None,
    };
    // Both identities must be well formed (the reader already refuses a malformed or repeated record): an id that is not an id differs
    // from every real one and is no evidence of a reboot (round 10, 9a D2).
    let rebooted: bool = match (&recorded_identity, current_identity()) {
        (Some(recorded), Ok(now)) => {
            recorded.is_well_formed()
                && now.is_well_formed()
                && recorded.machine == now.machine
                && recorded.boot != now.boot
        }
        _ => false,
    };
    if rebooted {
        // The one contradiction that is kept: a recorded scope (its name carries a random token, so it cannot be a reused name)
        // that has a member after a reboot is not explained by the reboot.
        for line in text.lines() {
            if let Ok(Record::Scope {
                path: scope_path,
                ..
            }) = parse_record(line)
                && populated_at(&scope_path).unwrap_or(false)
            {
                return Err(Some(Finding::Live(format!(
                    "{}: the scope {} of the killed run has a member although the boot id changed since the run; the journal was kept for a later run",
                    directory.display(),
                    scope_path.display()
                ))));
            }
        }
        return Err(None);
    }
    // The view of the journal directory: another mount view of the same path cannot be trusted to hold the same scopes.
    match &view_text {
        Ok(recorded) => {
            let expected: String = recorded
                .lines()
                .next()
                .unwrap_or("")
                .strip_prefix("view ")
                .unwrap_or("")
                .trim()
                .to_string();
            match view_of(directory) {
                Ok(seen) if seen.to_string() == expected => {}
                Ok(seen) => note_uncertain(
                    &mut uncertain,
                    format!(
                        "{}: the journal is seen through another mount or path (mount and inode {seen} here, {expected:?} when it was made), so the containment it recorded may not be the one visible here; {hint}",
                        directory.display()
                    ),
                ),
                Err(error) => note_uncertain(
                    &mut uncertain,
                    format!(
                        "{}: the view of the journal cannot be examined ({error}); {hint}",
                        directory.display()
                    ),
                ),
            }
        }
        Err(error) => note_uncertain(
            &mut uncertain,
            format!(
                "{}: the record of the journal's mount view could not be read ({error}); {hint}",
                directory.display()
            ),
        ),
    }
    return Ok(RecordContext {
        text,
        uncertain,
    });
}
