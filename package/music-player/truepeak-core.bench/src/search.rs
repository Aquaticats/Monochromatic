//! The corrected-target parameter search over candidate window policies.
//!
//! This is deliberately more than the prior margin-sweep.
//!  The objective is the decided
//! one:
//!  among policies that keep every track inside the bounds and stay under the
//! decoded-seconds budget,
//!  minimize the worst-case too-quiet error first,
//!  then prefer
//! the simplest classifier.
//!  Because a probe never over-reads the true peak,
//!  with a
//! margin at or below the too-quiet bound the only failures are too-loud under-reads,
//! so the worst too-quiet error equals the margin and the search drives the margin down
//! until the violators it must full-scan would exceed the budget.

/// Imports the corpus track record.
use crate::corpus::Track;
/// Imports the per-candidate evaluation and its report.
use crate::evaluate::{Candidate, Report, evaluate};

/// One scored point in the search,
///  kept for ranking and reporting.
#[derive(Clone, Debug)]
pub struct Scored {
    /// The full evaluation of this candidate.
    pub report: Report,
    /// Whether the candidate's decoded cost is within the corrected target.
    pub within_budget: bool,
}

/// Sweep window seconds and probe margin for a fixed window count,
///  scoring each point.
///
/// What:
///  evaluates a grid of `(window_seconds, probe_margin_db)` for one count.
///  Why:
///  the
/// count is fixed by the policy (14);
///  the seconds set the threshold and the margin sets
/// the safety,
///  and together they trade worst-too-quiet against violator budget.
pub fn sweep(
    tracks: &[Track],
    window_count: usize,
    seconds_grid: &[f64],
    margin_grid: &[f64],
    target_secs: f64,
) -> Vec<Scored> {
    // Evaluate every grid point; the grids are small, so a full product is fine.
    let mut scored = Vec::new();
    for &window_seconds in seconds_grid {
        for &probe_margin_db in margin_grid {
            let candidate = Candidate {
                window_count,
                window_seconds,
                probe_margin_db,
            };
            let report = evaluate(tracks, candidate);
            let within_budget = report.decoded_secs <= target_secs;
            scored.push(Scored {
                report,
                within_budget,
            });
        }
    }
    return scored
}

/// Rank scored candidates by the decided objective,
///  best first.
///
/// What:
///  keep only in-budget points,
///  then order by least-bad worst-too-quiet,
///  then
/// fewest violators (a proxy for classifier simplicity),
///  then decoded seconds closest to
/// the target (spend the budget pulling tracks toward the ceiling).
///  Why:
///  this encodes
/// "loudest-safe,
///  then simplest" directly.
pub fn rank(mut scored: Vec<Scored>, target_secs: f64) -> Vec<Scored> {
    // Drop over-budget points: they cannot ship.
    scored.retain(|point| return point.within_budget);
    // Order by the objective; floats are compared through total_cmp to stay total.
    scored.sort_by(|left, right| {
        let quiet = right
            .report
            .worst_too_quiet_db
            .total_cmp(&left.report.worst_too_quiet_db);
        let violators = left
            .report
            .violators
            .len()
            .cmp(&right.report.violators.len());
        let budget = (target_secs - left.report.decoded_secs)
            .abs()
            .total_cmp(&(target_secs - right.report.decoded_secs).abs());
        return quiet.then(violators).then(budget)
    });
    return scored
}
