//! What:
//!  The batches an earlier service left open:
//!  which of them this instance finishes,
//!  or drops,
//!  or holds,
//!  and the word each decision is logged with.
//! Why:
//!   An open batch means an earlier service stopped between its text and its end,
//!  and a batch whose branch moved but whose index update gave up left its intent open;
//!  settling them first puts the index right before anything else changes it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reports = instance ? recoverInstance(context, instance) : recover(context);
//! ```

use crate::journald_native::Priority;
use crate::service_journal::Recovery;
use crate::service_land::{
    LandContext,
    recover,
    recover_instance,
};
use crate::service_timing::log_entry;
/// Finishes the open batches this instance owns:
/// those of its worktree when it serves one,
/// all of them in a development run.
pub(crate) fn settle_open_batches(
    context: &LandContext,
    instance: Option<&str>,
) -> Vec<crate::service_land::RecoveryReport> {
    let reports: Vec<crate::service_land::RecoveryReport> = match instance {
        Some(mine) => recover_instance(context, mine),
        None => recover(context),
    };
    // An open batch means an earlier service stopped between its intent and its end; each one settled is said, with the
    //  repository whose batch it is, because one journal can be read across a whole machine.
    for report in &reports {
        let repository: String =
            crate::service_timing::in_the_repository_at(report.repository.as_path());
        log_entry(
            Priority::Warning,
            format!(
                "settled the batch an earlier service left open {repository} batch={} decision={}",
                report.seq,
                decision_word(report.decision)
            )
            .as_str(),
            &[("CLI_GIT_BATCH", report.seq.to_string())],
        );
    }
    return reports;
}

/// The word a settled batch is logged with,
/// so the record says which decision was made and not only that one was.
pub(crate) fn decision_word(decision: Recovery) -> &'static str {
    return match decision {
        Recovery::Finish => "finish",
        Recovery::Drop => "drop",
        Recovery::Hold => "hold",
    };
}
