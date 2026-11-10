//! Tests for the candidate sweep and the objective ranking that picked the
//! shipped policy.

use super::*;
use crate::evaluate::Violator;
use crate::test_fixtures::{flat_bins, levels_match, track};

/// Seconds in every fixture track,
///  long enough to be probed under every threshold the tests sweep.
const TRACK_SECS: u64 = 100;

/// The window count every swept candidate uses.
const WINDOW_COUNT: usize = 2;

/// Window seconds the sweep tries.
const SECONDS_GRID: &[f64] = &[1.0, 2.0];

/// Probe margins the sweep tries.
const MARGIN_GRID: &[f64] = &[0.0, 0.5];

/// The flat track the sweep scores.
fn flat_track() -> Track {
    return track("flat.flac", TRACK_SECS, 0.5, flat_bins(TRACK_SECS as usize, 0.5))
}

/// One scored point built straight from the fields the ranking reads.
fn scored(decoded_secs: f64, worst_too_quiet_db: f64, violators: usize) -> Scored {
    let candidate = Candidate {
        window_count: WINDOW_COUNT,
        window_seconds: 1.0,
        probe_margin_db: 0.0,
    };
    let violator = Violator {
        path: "violator.flac".to_owned(),
        error_db: worst_too_quiet_db,
        sampled_max: 0.5,
        duration_secs: TRACK_SECS as f64,
    };
    return Scored {
        report: Report {
            candidate,
            threshold_secs: WINDOW_COUNT as f64,
            short_count: 0,
            accepted_count: 1,
            violators: vec![violator; violators],
            decoded_secs,
            worst_too_loud_db: 0.0,
            worst_too_quiet_db,
        },
        within_budget: true,
    }
}

/// The order a ranking produced,
///  as the decoded seconds that identify each fixture point.
fn decoded_order(ranked: &[Scored]) -> Vec<f64> {
    return ranked.iter().map(|point| return point.report.decoded_secs).collect()
}

/// What:
///  the sweep evaluates every point of the grid product,
///  carrying that point's candidate into its report.
#[test]
fn the_sweep_covers_every_point_of_the_grid() {
    let tracks = [flat_track()];
    let points = sweep(&tracks, WINDOW_COUNT, SECONDS_GRID, MARGIN_GRID, 1.0e9);
    assert_eq!(points.len(), SECONDS_GRID.len() * MARGIN_GRID.len());
    for (index, point) in points.iter().enumerate() {
        let expected_seconds = SECONDS_GRID[index / MARGIN_GRID.len()];
        let expected_margin = MARGIN_GRID[index % MARGIN_GRID.len()];
        assert_eq!(point.report.candidate.window_count, WINDOW_COUNT);
        assert_eq!(point.report.candidate.window_seconds, expected_seconds);
        assert_eq!(point.report.candidate.probe_margin_db, expected_margin);
        assert!(
            levels_match(point.report.threshold_secs, WINDOW_COUNT as f64 * expected_seconds),
            "the threshold follows the grid point",
        );
    }
}

/// What:
///  a point is in budget exactly when its decoded cost fits the target.
#[test]
fn the_sweep_flags_a_point_by_its_decoded_cost() {
    let tracks = [flat_track()];
    let target_secs = 0.0;
    for point in sweep(&tracks, WINDOW_COUNT, SECONDS_GRID, MARGIN_GRID, target_secs) {
        assert!(!point.within_budget, "no probe fits a zero target");
        assert!(point.report.decoded_secs > target_secs);
    }
}

/// What:
///  an empty grid sweeps nothing and ranks nothing.
#[test]
fn an_empty_grid_sweeps_and_ranks_nothing() {
    let tracks = [flat_track()];
    assert!(sweep(&tracks, WINDOW_COUNT, &[], MARGIN_GRID, 1.0).is_empty());
    assert!(rank(Vec::new(), 1.0).is_empty());
}

/// What:
///  the ranking drops every point the budget cannot afford.
#[test]
fn the_ranking_drops_the_points_over_the_budget() {
    let mut over_budget = scored(TRACK_SECS as f64, 0.0, 0);
    over_budget.within_budget = false;
    let ranked = rank(vec![over_budget, scored(1.0, 0.0, 0)], 1.0e9);
    assert_eq!(decoded_order(&ranked), vec![1.0], "only the affordable point survives");
}

/// What:
///  the ranking reads least-bad worst-too-quiet first,
///  then fewest violators,
/// then the decoded cost closest to the target.
#[test]
fn the_ranking_orders_by_the_decided_objective() {
    let target_secs = TRACK_SECS as f64;
    let ranked = rank(
        vec![
            // Identified by its decoded seconds in the expectation below.
            scored(target_secs, -1.0, 0),           // the quietest and so the worst
            scored(target_secs, -0.5, 9),           // good quiet, too many violators
            scored(target_secs + 20.0, -0.5, 1),    // good on both, spends 20 s too much
            scored(target_secs - 10.0, -0.5, 1),    // good on both, closest to the target
        ],
        target_secs,
    );
    assert_eq!(
        decoded_order(&ranked),
        vec![target_secs - 10.0, target_secs + 20.0, target_secs, target_secs],
        "fewest violators and closest budget first, the quietest last",
    );
    assert_eq!(ranked[2].report.worst_too_quiet_db, -0.5, "both -0.5 points precede the -1.0 one");
    assert_eq!(ranked[3].report.worst_too_quiet_db, -1.0);
}
