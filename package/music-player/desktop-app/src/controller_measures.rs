//! Controller side of the per-track measures: forwards sweep reports, refreshes them when files change, and names the opened folder.
//!
//! What: `impl Controller` methods that turn sweep reports into `Update::Measures` and `Update::Analysis`,
//! and announce the opened folder's display name.
//! Why: the controller owns the queue generation and the cache handle, so stale reports are dropped here.

use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Arc;

use truepeak_core::Decision;

use crate::command::Update;
use crate::controller::Controller;
use crate::sweep_report::SweepReport;
use crate::track_measure::measure_from_decision;

/// What:
///      `fn display_name_of(root: &Path) -> String`.
///  The folder's last path component, or its whole path when it has none.
/// Why:
///       A root such as `/` has no file name, and the first-run copy still needs a name to show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const displayNameOf = (root: string) => basename(root) || root;
/// ```
pub(crate) fn display_name_of(root: &Path) -> String {
    return root
        .file_name()
        .map_or_else(|| return root.display().to_string(), |name| return name.to_string_lossy().into_owned());
}

impl Controller {
    /// What:
    ///      `pub(crate) fn emit_source_root(&self)`.
    ///  Sends the opened folder's display name, or `None` when no folder is open.
    /// Why:
    ///       The first-run copy depends on whether a folder is open and what it is called.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// emitSourceRoot(): void { this.emit({ kind: "sourceRoot", name: this.sourceRoot ? displayNameOf(this.sourceRoot) : null }); }
    /// ```
    pub(crate) fn emit_source_root(&self) {
        let name = self.source_root.as_deref().map(display_name_of);
        self.emit(Update::SourceRoot(name));
    }

    /// What:
    ///      `pub(crate) fn handle_sweep_report(&mut self, report: SweepReport)`.
    ///  Forwards a sweep report to the window when it belongs to the newest queue load.
    /// Why:
    ///       Sweeps are never cancelled, so a report from a replaced queue must not update the rows or the bar.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handleSweepReport(report: SweepReport): void { if (report.generation === this.analysisGeneration) this.emit(toUpdate(report)); }
    /// ```
    pub(crate) fn handle_sweep_report(&mut self, report: SweepReport) {
        match report {
            SweepReport::Measures { generation, measures } => {
                if generation != self.analysis_generation {
                    tracing::debug!(generation, current = self.analysis_generation, "dropped measures from a replaced queue");
                    return;
                }
                self.forget_reports_of_decoded(&measures);
                self.emit(Update::Measures(measures));
            }
            SweepReport::Progress { generation, done, total } => {
                if generation != self.analysis_generation {
                    tracing::debug!(generation, current = self.analysis_generation, "dropped progress from a replaced queue");
                    return;
                }
                self.emit(Update::Analysis { done, total });
            }
            SweepReport::RefreshDone { id, .. } => self.finish_refresh(id),
            SweepReport::Unreadable { generation, files } => {
                if generation != self.analysis_generation {
                    tracing::debug!(generation, current = self.analysis_generation, "dropped unreadable files from a replaced queue");
                    return;
                }
                self.report_unplayable_once(files);
            }
        }
    }

    /// What:
    ///      `pub(crate) fn publish_decision_measure(&self, path: &Path, decision: &Decision)`.
    ///  Sends the measure of the track that was just measured on the current-track worker.
    /// Why:
    ///       The playing track is measured outside the sweep, so its row and deck would otherwise stay blank until the next load.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// publishDecisionMeasure(path: string, decision: Decision): void { this.emit({ kind: "measures", measures: [[path, measureFromDecision(decision)]] }); }
    /// ```
    pub(crate) fn publish_decision_measure(&self, path: &Path, decision: &Decision) {
        self.emit(Update::Measures(vec![(path.to_path_buf(), measure_from_decision(decision))]));
    }

    /// What:
    ///      `pub(crate) fn report_unplayable_once(&self, files: Vec<PathBuf>)`.
    ///  Tells the window about the files it has not yet been told about.
    /// Why:
    ///       D113 and D83: a failure is reported once per file in plain words. The same broken file fails again at every
    ///  later sweep, and a person who already saw the message does not need it again. A file that is still being written
    ///  fails with each partial state it passes through, so the report is per file and not per state of the file; it is
    ///  forgotten when a peak is measured for the file (it decodes now), so a file that breaks again is told again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reportUnplayableOnce(files: string[]): void { const fresh = files.filter(f => reported.add(f)); if (fresh.length) emit({ kind: "unplayable", paths: fresh }); }
    /// ```
    pub(crate) fn report_unplayable_once(&self, files: Vec<PathBuf>) {
        let fresh: Vec<PathBuf> = match self.reported_unplayable.lock() {
            Ok(mut reported) => files.into_iter().filter(|file| return reported.insert(file.clone())).collect(),
            Err(_poisoned) => files,
        };
        if fresh.is_empty() {
            tracing::debug!("every unplayable file was already reported");
            return;
        }
        self.emit_unplayable(fresh);
    }

    /// What:
    ///      `fn forget_reports_of_decoded(&self, measures: &[(PathBuf, TrackMeasure)])`.
    ///  Forgets the failure report of every file that now has a measured peak.
    /// Why:
    ///       A file that decodes again was fixed or replaced; if it breaks later the person is told again (D113).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// forgetReportsOfDecoded(measures: [string, TrackMeasure][]): void { for (const [p, m] of measures) if (m.peakDbtp !== null) reported.delete(p); }
    /// ```
    fn forget_reports_of_decoded(&self, measures: &[(PathBuf, crate::track_measure::TrackMeasure)]) {
        if let Ok(mut reported) = self.reported_unplayable.lock() {
            for (path, measure) in measures {
                if measure.peak_dbtp.is_some() {
                    reported.remove(path);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "controller_measures_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "controller_refresh_tests.rs"]
mod refresh_tests;

#[cfg(test)]
#[path = "refresh_cost_tests.rs"]
mod refresh_cost_tests;
