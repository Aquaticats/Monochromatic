//! What:
//!  The words one line says about a record of an interrupted copy that cannot be read:
//!  what the record was,
//!  where it went,
//!  and what may still stand on disk because of it.
//! Why:
//!  The person reads one line per record,
//!  and it is the only place a stage folder nothing names any more,
//!  or a worktree nothing names as one holding less than its source ignores,
//!  is named at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = settledNotice({ kind, record, moved, reason, left });
//! ```

use crate::json_record::quote;
use crate::worktree_copy_selection::STAGE_PREFIX;
use std::path::Path;

/// What:
///  The line that names one record that cannot be read,
/// where it was moved,
/// and what may still stand on disk because of it.
/// Why:
///   The person reads what the wrapper could not read,
/// that nothing was deleted,
/// and what is left where,
/// so the leftovers are settled rather than searched for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${kind} ${quote(record)} could not be read (${reason}), so it was moved to ${quote(moved)}; ${left}.`
/// ```
pub(crate) fn settled_notice(kind: &str, record: &Path, moved: &Path, reason: &str, left: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: the {kind} {} could not be read ({reason}), so it was moved to {} and nothing was deleted. {left}",
        quote(record.to_string_lossy().as_ref()),
        quote(moved.to_string_lossy().as_ref())
    );
}

/// What:
///  What a journal that cannot be read may leave on disk.
/// Why:
///   The journal is the only record that names the stage it filled,
/// so a stage folder can stand beside the worktree with nothing left to find it;
/// and the worktree itself is named by the record of an unfilled copy where the copy got that far.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The ignored-state copy it belonged to was interrupted and nothing in any worktree was changed; the private stage folder it named (whose name begins ${STAGE_PREFIX} beside the worktree it was filling) may still stand with the ignored state it holds, nothing finds it again, and \`rm -r\` removes it once nothing needs it. A worktree the copy never finished is named again by the record of an unfilled copy where that record stands.`
/// ```
pub(crate) fn what_a_lost_journal_leaves() -> String {
    return format!(
        "The ignored-state copy it belonged to was interrupted, and nothing in any worktree was changed: the private stage folder it named (a folder whose name begins {STAGE_PREFIX} beside the worktree it was filling) may still stand with the ignored state it holds, nothing of the wrapper's finds it again, and `rm -r` removes it once nothing needs it. A worktree the copy never finished is named again by the record of an unfilled copy where that record stands."
    );
}

/// What:
///  What a claim that cannot be read may leave on disk when its stage folder cannot be recovered.
/// Why:
///   Nothing beside the claim names the folder any more,
/// and the destination record beside it is settled on its own,
/// so both are said rather than left to be guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The stage folder it named (a private folder whose name begins ${STAGE_PREFIX} beside the worktree the add registered) may still stand with the ignored state it holds and nothing finds it again, and the record of the destination beside the claim is named and settled by the commands that read it.`
/// ```
pub(crate) fn what_a_lost_claim_leaves() -> String {
    return format!(
        "The stage folder it named (a private folder whose name begins {STAGE_PREFIX} beside the worktree that add registered) may still stand with the ignored state it holds, and nothing of the wrapper's finds it again; the record of the destination beside the claim is named and settled on its own by the commands that read it."
    );
}
