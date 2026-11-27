//! What:
//!  The record the mode compares and writes:
//!  the fields that differ from the proved facts,
//!  the value each side of the comparison printed,
//!  and the write that a `--replace` token admits.
//! Why:
//!  A runbook that runs twice must be harmless,
//!  and a differing field is a refusal rather than a write,
//!  so the only admitted change is the GUID a restore moved and the caller
//!  typed from the kernel's own answer,
//!  and every other difference is named with the record's value beside the
//!  kernel's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = writeOrAgree(places, proved, request); // throws RecordDiffers
//! const field = differingField(recorded, proved);     // undefined when every field agrees
//! ```

use super::record_checkout_mode_places::RecordCheckoutPlaces;
use crate::worktree_clone_mount_handler::handler_mode_arguments::RecordCheckoutRequest;
use crate::worktree_clone_mount_handler::record_checkout_mode_record::{
    DifferingField,
    RecordCheckoutFailure,
    RecordedCheckout,
};
use crate::worktree_clone_mount_handler::record_checkout_mode_record_file::{
    already_recorded_line,
    read_record,
    recorded_line,
    replaced_line,
    write_record,
};

/// What:
///  Compare the proved facts with the record,
///  admit the GUID a `--replace` names,
///  and write only when the GUID is the whole of what changed.
/// Why:
///  A runbook that runs twice must be harmless,
///  and a differing field is a refusal rather than a write,
///  so the only admitted change is the GUID a restore moved and the caller
///  typed from the kernel's own answer.
///  The record the command writes is the dataset this command named:
///  a container that also differs is refused,
///  because no token names it and the line says the container was kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = writeOrAgree(places, proved, request);
/// ```
///
/// # Errors
/// Everything a differing field,
/// a wrong token,
/// a record that is not one root alone may write,
/// and a write that fails.
pub(super) fn write_or_agree(
    places: &RecordCheckoutPlaces<'_>,
    proved: &RecordedCheckout,
    request: &RecordCheckoutRequest,
) -> Result<String, RecordCheckoutFailure> {
    let existing: Option<RecordedCheckout> = read_record(places.record.as_path())?;
    let Some(recorded) = existing else {
        if request.replace.is_some() {
            return Err(RecordCheckoutFailure::NothingToReplace {
                record: places.record.clone(),
            });
        }
        write_record(places.record.as_path(), proved)?;
        return Ok(recorded_line(proved, places.record.as_path()));
    };
    let difference: Option<DifferingField> = differing_field(&recorded, proved);
    let Some(field) = difference else {
        return Ok(already_recorded_line(places.record.as_path()));
    };
    let Some(typed) = request.replace.as_deref() else {
        return Err(RecordCheckoutFailure::RecordDiffers {
            recorded: recorded_value(&field),
            kernel: proved_value(proved, &field),
            field,
        });
    };
    if typed != proved.guid {
        return Err(RecordCheckoutFailure::ReplaceTokenDiffers {
            kernel: proved.guid.clone(),
            typed: String::from(typed),
        });
    }
    // The token admits the GUID, so the record with that GUID is what the write would leave: any field that still
    // differs is a change no token names, and it refuses rather than being written under a line that says it was kept.
    let was: String = recorded.guid.clone();
    let admitted: RecordedCheckout = RecordedCheckout {
        guid: proved.guid.clone(),
        ..recorded
    };
    if let Some(also) = differing_field(&admitted, proved) {
        return Err(RecordCheckoutFailure::RecordDiffers {
            recorded: recorded_value(&also),
            kernel: proved_value(proved, &also),
            field: also,
        });
    }
    write_record(places.record.as_path(), proved)?;
    return Ok(replaced_line(proved, was.as_str(), places.record.as_path()));
}

/// What:
///  The fields of the record that differ from the proved facts,
///  one at a time.
/// Why:
///  Each field's repair is another command,
///  so the first difference is the one a person is sent to fix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const field = differingField(recorded, proved); // undefined when every field agrees
/// ```
fn differing_field(
    recorded: &RecordedCheckout,
    proved: &RecordedCheckout,
) -> Option<DifferingField> {
    if recorded.checkout != proved.checkout {
        return Some(DifferingField::Path {
            recorded: recorded.checkout.display().to_string(),
        });
    }
    if recorded.dataset != proved.dataset {
        return Some(DifferingField::Dataset {
            recorded: recorded.dataset.clone(),
        });
    }
    if recorded.guid != proved.guid {
        return Some(DifferingField::Guid {
            recorded: recorded.guid.clone(),
        });
    }
    if recorded.container != proved.container {
        return Some(DifferingField::Container {
            recorded: recorded.container.clone(),
        });
    }
    return None;
}

/// The record's value of one field,
/// which a refusal prints beside the kernel's.
fn recorded_value(field: &DifferingField) -> String {
    return match field {
        DifferingField::Path {
            recorded,
        }
        | DifferingField::Dataset {
            recorded,
        }
        | DifferingField::Guid {
            recorded,
        }
        | DifferingField::Container {
            recorded,
        } => recorded.clone(),
    };
}

/// The kernel's value of one field,
/// which a refusal prints beside the record's.
fn proved_value(
    proved: &RecordedCheckout,
    field: &DifferingField,
) -> String {
    return match field {
        DifferingField::Path {
            ..
        } => proved.checkout.display().to_string(),
        DifferingField::Dataset {
            ..
        } => proved.dataset.clone(),
        DifferingField::Guid {
            ..
        } => proved.guid.clone(),
        DifferingField::Container {
            ..
        } => proved.container.clone(),
    };
}
