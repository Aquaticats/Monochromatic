//! What:
//!  Compare a plan's outputs with the real files and list what differs.
//! Why:
//!  `check` reports exactly this list, `write` publishes exactly this list, and `--expected-dir` renders it, so one
//!  computation serves all three and `check` needs no prior write.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function computeDrift(report: PlanReport): Drift[]
//! ```

use crate::diagnostic::Diagnostic;
use crate::event::Event;
use crate::plan::PlanReport;
use crate::real_fs::RealState;
use crate::view::Planned;
use serde_json::Value;

/// What:
///  Why one path differs from its planned state.
/// Why:
///  Consumers branch on the reason (a mode change is repaired differently from a content change).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DriftReason = 'missing' | 'content' | 'mode' | 'extra' | 'extra-directory' | 'replace-link';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriftReason {
    /// The file is planned but absent.
    Missing,
    /// The file exists with other content.
    Content,
    /// Content matches but the permission bits differ.
    Mode,
    /// The file exists but a rule wants it removed.
    Extra,
    /// The directory exists but a rule wants it removed.
    ExtraDirectory,
    /// A link or special file stands where a regular file is planned; publishing replaces it.
    ReplaceLink,
}

impl DriftReason {
    /// The kebab-case name used in events.
    pub fn name(&self) -> &'static str {
        match self {
            DriftReason::Missing => return "missing",
            DriftReason::Content => return "content",
            DriftReason::Mode => return "mode",
            DriftReason::Extra => return "extra",
            DriftReason::ExtraDirectory => return "extra-directory",
            DriftReason::ReplaceLink => return "replace-link",
        }
    }
}

/// What:
///  One path whose real state differs from the plan.
/// Why:
///  The unit `check` reports and `write` repairs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Drift = { path: string; rule: string; reason: DriftReason; actual?: Uint8Array; expected?: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Drift {
    /// Repo-relative path.
    pub path: String,
    /// Id of the rule that owns the planned state.
    pub rule: String,
    /// Why it differs.
    pub reason: DriftReason,
    /// Current bytes, when the file exists.
    pub actual: Option<Vec<u8>>,
    /// Planned bytes, when the file will exist.
    pub expected: Option<Vec<u8>>,
}

impl Drift {
    /// What:
    ///  Render the drift as a `drift` event.
    /// Why:
    ///  `check` prints one event per drifted path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toEvent(): Event
    /// ```
    pub fn to_event(&self) -> Event {
        let message = format!("{} differs from what rule {} generates ({})", self.path, self.rule, self.reason.name());
        let mut event = Event::new("drift", &format!("drift-{}", self.reason.name()), &message);
        event.set_field("path", Value::String(self.path.clone()));
        event.set_field("rule", Value::String(self.rule.clone()));
        event.set_field("reason", Value::String(self.reason.name().to_string()));
        event.hint = Some("run `file-enforcer write` to regenerate it".to_string());
        return event;
    }
}

/// What:
///  Compare every planned output and planned removal with the real filesystem.
///  The comparison observes each real path through the view, so revalidation covers outputs as well as inputs.
/// Why:
///  Drift is exactly the set of paths `write` must touch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function computeDrift(report: PlanReport): Drift[]
/// ```
pub fn compute_drift(report: &mut PlanReport) -> Result<Vec<Drift>, Diagnostic> {
    let mut drifts: Vec<Drift> = Vec::new();
    let planned: Vec<(String, Planned)> = report.view.overlay().iter().map(|pair| return (pair.0.clone(), pair.1.clone())).collect();
    for (path, state) in planned {
        let rule = report.owners.get(&path).cloned().unwrap_or_default();
        let real = report.view.observe_file(&path)?;
        match (&state, real) {
            (Planned::File { content, .. }, RealState::Absent) => {
                drifts.push(Drift { path, rule, reason: DriftReason::Missing, actual: None, expected: Some(content.clone()) });
            }
            (Planned::File { content, mode }, RealState::File { content: actual, mode: actual_mode }) => {
                if &actual != content {
                    drifts.push(Drift { path, rule, reason: DriftReason::Content, actual: Some(actual), expected: Some(content.clone()) });
                } else if mode.is_some() && *mode != Some(actual_mode) {
                    drifts.push(Drift { path, rule, reason: DriftReason::Mode, actual: Some(actual), expected: Some(content.clone()) });
                }
            }
            (Planned::File { content, .. }, RealState::Symlink | RealState::Other) => {
                drifts.push(Drift { path, rule, reason: DriftReason::ReplaceLink, actual: None, expected: Some(content.clone()) });
            }
            (Planned::File { .. }, RealState::Directory) => {
                let message = format!("{path} is a directory where rule {rule} wants a file; remove the directory first");
                return Err(Diagnostic::new("path-blocked", &message).with_path(&path).with_rule(&rule));
            }
            (Planned::Removed, RealState::Absent) => {}
            (Planned::Removed, RealState::File { content: actual, .. }) => {
                drifts.push(Drift { path, rule, reason: DriftReason::Extra, actual: Some(actual), expected: None });
            }
            (Planned::Removed, RealState::Symlink | RealState::Other) => {
                drifts.push(Drift { path, rule, reason: DriftReason::Extra, actual: None, expected: None });
            }
            (Planned::Removed, RealState::Directory) => {}
        }
    }
    let directories = report.removed_directories.clone();
    for (path, rule) in directories {
        if report.view.observe_file(&path)? == RealState::Directory {
            drifts.push(Drift { path, rule, reason: DriftReason::ExtraDirectory, actual: None, expected: None });
        }
    }
    return Ok(drifts);
}
