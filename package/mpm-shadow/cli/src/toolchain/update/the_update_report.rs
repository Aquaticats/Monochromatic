//! What:
//! The text `update` and `outdated` print:
//! one line per tool,
//! then one summary line that says whether the lock was written,
//! what would move,
//! or how many lookups failed.
//! Why:
//! The same words go to a person's terminal and to the tests that read them,
//! so the whole text is built in one place and never printed from the update itself.
//! A caller imports from `update` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function formatUpdateReport(report: UpdateReport, check: boolean): string;
//! ```

/// Imports what happened to one tool,
/// and the report the text is built from.
use super::what_an_update_did::{
    OutcomeKind,
    UpdateReport,
};
/// The text `update` prints:
/// one line per tool and a summary.
pub fn format_update_report(
    report: &UpdateReport,
    check: bool,
) -> String {
    let mut text: String = String::new();
    for id in &report.synced {
        text.push_str(&format!("synced    {id}  (was missing from the machine)\n"));
    }
    for outcome in &report.outcomes {
        let line: String = match &outcome.kind {
            OutcomeKind::Unchanged => format!("unchanged {}", outcome.id),
            OutcomeKind::Pinned => format!("pinned    {}", outcome.id),
            OutcomeKind::Kept => format!("kept      {}", outcome.id),
            OutcomeKind::Updated {
                old_version,
                new_version,
            } => format!("updated   {}  {old_version} -> {new_version}", outcome.id),
            OutcomeKind::Added {
                version,
            } => format!("added     {}  {version}", outcome.id),
            OutcomeKind::Failed {
                reason,
            } => format!("failed    {}  {reason}", outcome.id),
        };
        text.push_str(&line);
        text.push('\n');
    }
    let mut moved: usize = 0;
    let mut failed: usize = 0;
    for outcome in &report.outcomes {
        match outcome.kind {
            OutcomeKind::Updated {
                ..
            }
            | OutcomeKind::Added {
                ..
            } => moved += 1,
            OutcomeKind::Failed {
                ..
            } => failed += 1,
            _ => {}
        }
    }
    if failed > 0 {
        text.push_str(&format!(
            "toolchain update: {failed} lookups failed; toolchain.lock.json was not written\n"
        ));
    } else if check {
        text.push_str(&format!(
            "toolchain outdated: {moved} tools would move; nothing was written or installed\n"
        ));
    } else if report.wrote {
        text.push_str(&format!(
            "toolchain update: wrote toolchain.lock.json ({moved} tools moved)\n"
        ));
    } else {
        text.push_str(&format!(
            "toolchain update: toolchain.lock.json is already current\n"
        ));
    }
    return text;
}
