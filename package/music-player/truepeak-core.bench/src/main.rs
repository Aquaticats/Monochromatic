//! Corpus evaluation and corrected-target search for the shared true-peak policy.
//!
//! Reads a per-track measurement corpus (full true peak plus per-second bin peaks),
//! computes the corrected decoded-seconds target (`total decodable seconds / 4`),
//!  and
//! ranks window policies by the decided objective using the shared truepeak-core math.
//! Usage:
//!  `truepeak-core-bench <tracks.jsonl> [window_count]`.

/// The per-track corpus record and loader.
mod corpus;
/// The per-candidate evaluation against the corpus.
mod evaluate;
/// The classifier that routes risky long tracks to a full scan from probe features.
mod classify;
/// The feasible no-classifier model:
///  minimize the fixed margin by probe density.
mod feasible;
/// The decided proportional-coverage policy and the margin/clamp tradeoff.
mod proportional;
/// The corrected-target parameter search and ranking.
mod search;
/// The bucket-first composite with the FLAC bones-guided probe.
mod buckets;
/// The quarter-measure answer's probe:
///  even pass one plus frontier zoom.
mod zoom;

/// Imports `anyhow` helpers for application-level error returns.
use anyhow::{Context, Result};
/// Imports the process argument reader.
use std::env;
/// Imports the borrowed path type for the corpus location.
use std::path::Path;

/// Imports the corpus loader,
///  the track record,
///  and the safe-provenance loader.
use crate::corpus::{Track, load_safe_paths, load_tracks};
/// Imports the classifier search,
///  the feature diagnostic,
///  and the per-track dump.
use crate::classify::{diagnose, fit_full_scan_rule, write_long_features};
/// Imports the feasible no-classifier density search and the provenance margin.
use crate::feasible::{best_feasible, provenance_margin};
/// Imports the candidate policy type for the provenance report.
use crate::evaluate::Candidate;
/// Imports the decided proportional-coverage policy evaluation.
use crate::proportional::{evaluate_proportional, margin_clamp_table, under_read_quantile};
/// Imports the safe-provenance path set default.
use std::collections::HashSet;
/// Imports the sweep and the objective ranking.
use crate::search::{rank, sweep};
/// Imports the frontier-zoom evaluation and its measures.
use crate::zoom::{evaluate_zoom, measure_zoom, zoom_under_read_quantile};
/// Imports the bucket-composite report.
use crate::buckets::report_buckets;

/// The plan's quarter-library divisor for the benchmark target.
const TARGET_DIVISOR: f64 = 4.0;
/// Window counts to sweep:
///  at a fixed threshold (decode budget) more,
///  shorter windows
/// cover more distinct regions,
///  shrinking the gaps that cause under-read violators.
const WINDOW_COUNTS: &[usize] = &[14, 20, 28, 40];

/// Build an inclusive-ish float grid from `start` to `end` stepping by `step`.
///
/// What:
///  a small helper so the sweep grids read declaratively.
///  Why:
///  avoids hand-listing
/// dozens of thresholds and margins.
fn frange(start: f64, end: f64, step: f64) -> Vec<f64> {
    // Count the steps, then map each index to its value (functional, no mutable cursor).
    let steps = ((end - start) / step).round() as i64;
    return (0..=steps).map(|index| return start + index as f64 * step).collect()
}

/// The stage-two shipped policy's coverage,
///  kept locally so the ledger stays
/// reproducible now that the engine ships the bucket-zoom policy instead.
const LEGACY_COVERAGE_FRACTION: f64 = 0.2;
/// The stage-two shipped policy's probe window length in seconds.
const LEGACY_PROBE_WINDOW_SECS: f64 = 0.3;
/// The stage-two shipped policy's fixed margin in dB.
const LEGACY_PROBE_MARGIN_DB: f64 = 0.8;

/// Evaluate the superseded stage-two proportional policy on the corpus and print its
/// budget,
///  the under-read distribution,
///  and the margin/clamp tradeoff.
///
/// What:
///  replays the stage-two even probe from local legacy constants (the engine now
/// ships the bucket-zoom policy),
///  optionally loading provenance from a metadata
/// argument.
///  Why:
///  the letter's ledger must stay reproducible for comparison.
fn report_proportional(
    tracks: &[Track],
    full_secs: f64,
    target_secs: f64,
    args: &[String],
) -> Result<()> {
    let policy = truepeak_core::default_policy();
    // The optional metadata argument (a non-flag after the corpus path) enables the safe split.
    let safe: HashSet<String> = match args.iter().skip(2).find(|arg| return !arg.starts_with("--")) {
        Some(meta) => load_safe_paths(Path::new(meta))?,
        None => HashSet::new(),
    };
    let (decoded, under_reads) = evaluate_proportional(
        tracks,
        policy.short_scan_max_secs,
        LEGACY_COVERAGE_FRACTION,
        LEGACY_PROBE_WINDOW_SECS,
        policy.ceiling_dbtp,
        &safe,
    );
    println!(
        "\nlegacy stage-two proportional policy: short_scan<={}s coverage={} window={}s margin={}dB",
        policy.short_scan_max_secs, LEGACY_COVERAGE_FRACTION, LEGACY_PROBE_WINDOW_SECS, LEGACY_PROBE_MARGIN_DB
    );
    println!(
        "decoded={:.0}s ({:.1}% of corpus) target={:.0}s {}",
        decoded,
        100.0 * decoded / full_secs,
        target_secs,
        if decoded <= target_secs { "IN BUDGET" } else { "OVER" }
    );
    print!("loud long tracks={} safe-provenance={} | under-read percentiles dB:", under_reads.len(), safe.len());
    for fraction in [0.5, 0.9, 0.95, 0.99, 0.995, 1.0] {
        print!(" p{:.1}={:.2}", fraction * 100.0, under_read_quantile(&under_reads, fraction));
    }
    println!();
    println!("margin/clamp tradeoff (clamped = cold-start tracks the realtime clamp catches, warming later corrects):");
    for row in margin_clamp_table(&under_reads, &[0.5, 0.8, 1.0, 1.2, 1.5], policy.max_too_loud_db) {
        let percent = 100.0 * row.clamped as f64 / row.total.max(1) as f64;
        println!(
            "  margin={:.1}dB worst_quiet={:+.1}dB -> {} clamped ({percent:.2}%, {} safe) of {}",
            row.margin_db, row.worst_quiet_db, row.clamped, row.clamped_safe, row.total
        );
    }
    return Ok(())
}

/// The zoom pass-one coverage fraction the answer fixes (a tenth of each long track).
const ZOOM_PASS1_COVERAGE: f64 = 0.1;

/// The margins the zoom report tables:
///  the dial between loud-kept and cold-start clamps.
const ZOOM_MARGINS: &[f64] = &[0.3, 0.4, 0.5, 0.8];

/// The provenance-split margins the zoom report shows when metadata is present.
const ZOOM_SPLIT_SAFE_DB: f64 = 0.3;
/// The hot-side margin of the provenance split.
const ZOOM_SPLIT_HOT_DB: f64 = 0.5;

/// Evaluate the frontier-zoom answer on the corpus and print the three measures.
///
/// What:
///  runs the zoom probe at the full quarter budget and prints the under-read
/// distribution plus,
///  per candidate margin (and a provenance split when metadata is
/// given),
///  the letter's three measures.
///  Why:
///  the committed,
///  reproducible evaluation
/// of the quarter-measure answer.
fn report_zoom(
    tracks: &[Track],
    full_secs: f64,
    target_secs: f64,
    args: &[String],
) -> Result<()> {
    let policy = truepeak_core::default_policy();
    // The optional metadata argument (a non-flag after the corpus path) enables the split.
    let safe: HashSet<String> = match args.iter().skip(2).find(|arg| return !arg.starts_with("--")) {
        Some(meta) => load_safe_paths(Path::new(meta))?,
        None => HashSet::new(),
    };
    let (decoded, rows) = evaluate_zoom(
        tracks,
        policy.short_scan_max_secs,
        ZOOM_PASS1_COVERAGE,
        target_secs,
        &safe,
    );
    println!(
        "\nfrontier zoom: pass1={} short_scan<={}s budget={:.0}s",
        ZOOM_PASS1_COVERAGE, policy.short_scan_max_secs, target_secs
    );
    println!(
        "decoded={:.0}s ({:.2}% of corpus) {}",
        decoded,
        100.0 * decoded / full_secs,
        if decoded <= target_secs { "IN BUDGET" } else { "OVER" }
    );
    print!("under-read percentiles dB:");
    for fraction in [0.5, 0.9, 0.95, 0.99, 0.995, 1.0] {
        print!(
            " p{:.1}={:.2}",
            fraction * 100.0,
            zoom_under_read_quantile(&rows, policy.ceiling_dbtp, fraction)
        );
    }
    println!();
    for &margin in ZOOM_MARGINS {
        let m = measure_zoom(&rows, &|_| return margin, policy.max_too_loud_db, policy.ceiling_dbtp, tracks.len());
        println!(
            "  margin={margin:.1}: clamped={} ({} safe) avg_quiet={:.3}dB worst_quiet={:.2}dB worst_over={:.2}dB",
            m.clamped, m.clamped_safe, m.avg_quiet_db, m.worst_quiet_db, m.worst_over_db
        );
    }
    if !safe.is_empty() {
        let split = measure_zoom(
            &rows,
            &|row| if row.safe { return ZOOM_SPLIT_SAFE_DB } else { return ZOOM_SPLIT_HOT_DB },
            policy.max_too_loud_db,
            policy.ceiling_dbtp,
            tracks.len(),
        );
        println!(
            "  split safe={ZOOM_SPLIT_SAFE_DB:.1}/hot={ZOOM_SPLIT_HOT_DB:.1}: clamped={} ({} safe) avg_quiet={:.3}dB worst_quiet={:.2}dB worst_over={:.2}dB",
            split.clamped, split.clamped_safe, split.avg_quiet_db, split.worst_quiet_db, split.worst_over_db
        );
    }
    return Ok(())
}

/// Entry point:
///  load the corpus,
///  compute the target,
///  search,
///  and print the report.
fn main() -> Result<()> {
    // Send tracing events (including truepeak-core's) to stderr; the report is stdout.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| return tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();
    let args: Vec<String> = env::args().collect();
    let corpus_path = args
        .get(1)
        .context("usage: truepeak-core-bench <tracks.jsonl>")?;

    let tracks = load_tracks(Path::new(corpus_path))?;
    let full_secs: f64 = tracks.iter().map(|track| return track.duration_secs).sum();
    let target_secs = full_secs / TARGET_DIVISOR;

    println!("corpus: {} tracks", tracks.len());
    println!("full decodable seconds: {full_secs}");
    println!("corrected target (/{TARGET_DIVISOR}): {target_secs}");

    // The decided policy: `--proportional [metadata.jsonl]` evaluates it on the corpus and
    // prints the under-read distribution and the margin/clamp tradeoff, then returns.
    if args.iter().any(|arg| return arg == "--proportional") {
        return report_proportional(&tracks, full_secs, target_secs, &args);
    }

    // The quarter-measure answer: `--zoom [metadata.jsonl]` evaluates the frontier-zoom
    // probe at the full budget and prints the letter's three measures per margin.
    if args.iter().any(|arg| return arg == "--zoom") {
        return report_zoom(&tracks, full_secs, target_secs, &args);
    }

    // The bucket-first composite: `--buckets <tags-full.jsonl> [flac-profiles.jsonl]`
    // evaluates the decided per-provenance coverage/margin table with FLAC bones.
    if args.iter().any(|arg| return arg == "--buckets") {
        return report_buckets(&tracks, full_secs, target_secs, &args);
    }

    println!("window counts swept: {WINDOW_COUNTS:?}");

    // Sweep thresholds (converted to window seconds) and probe margins, per window count.
    let thresholds = frange(44.0, 60.0, 0.25);
    let margin_grid = frange(0.30, 2.00, 0.02);
    let mut scored = Vec::new();
    for &window_count in WINDOW_COUNTS {
        let seconds_grid: Vec<f64> = thresholds
            .iter()
            .map(|&threshold| return threshold / window_count as f64)
            .collect();
        let mut points = sweep(&tracks, window_count, &seconds_grid, &margin_grid, target_secs);
        scored.append(&mut points);
    }
    let ranked = rank(scored, target_secs);

    println!("\nin-budget candidates ranked by objective (loudest-safe, then simplest):");
    for point in ranked.iter().take(8) {
        let report = &point.report;
        println!(
            "  count={} thr={:.2}s ws={:.4}s margin={:.3}dB | decoded={:.1}s delta={:+.1}s | violators={} accepted={} short={} | worst_loud={:+.3}dB worst_quiet={:+.3}dB",
            report.candidate.window_count,
            report.threshold_secs,
            report.candidate.window_seconds,
            report.candidate.probe_margin_db,
            report.decoded_secs,
            report.decoded_secs - target_secs,
            report.violators.len(),
            report.accepted_count,
            report.short_count,
            report.worst_too_loud_db,
            report.worst_too_quiet_db,
        );
    }

    // For the best in-budget point, fit the probe-only full-scan classifier and report
    // the real routed budget (the oracle above assumes a perfect classifier).
    if let Some(best) = ranked.first() {
        println!();
        diagnose(&tracks, best.report.candidate);
        // Dump per-track features so a metadata-aware classifier can be evaluated offline.
        write_long_features(&tracks, best.report.candidate, Path::new("out/long_features.jsonl"))?;
        println!("wrote per-track features to out/long_features.jsonl");
        let rule = fit_full_scan_rule(&tracks, best.report.candidate, target_secs);
        println!(
            "\nbest candidate (count={}) + fitted classifier:",
            best.report.candidate.window_count
        );
        println!("  {rule}");
        // Exception list the classifier must catch (full paths, per the plan).
        println!("\nviolators to catch ({}):", best.report.violators.len());
        for violator in &best.report.violators {
            println!(
                "  err={:+.3}dB sampled_max={:.4} dur={:.0}s {}",
                violator.error_db, violator.sampled_max, violator.duration_secs, violator.path
            );
        }
    } else {
        println!("\nno candidate fits the corrected target for any swept window count");
    }

    // The feasible no-classifier model: the budget cannot afford a full-scan router, so
    // ship a single fixed margin and spend the budget on probe density to shrink it.
    let quiet_bound_db = truepeak_core::default_policy().max_too_quiet_db;
    let dense_counts = [20usize, 28, 40, 56, 80];
    let dense_thresholds = frange(40.0, 56.0, 1.0);
    let feasibles = best_feasible(&tracks, &dense_counts, &dense_thresholds, target_secs, quiet_bound_db);
    println!("\nfeasible no-classifier model (no full scans; single fixed margin), best densities:");
    for point in feasibles.iter().filter(|point| return point.feasible).take(30) {
        let threshold = point.candidate.window_count as f64 * point.candidate.window_seconds;
        println!(
            "  count={} thr={:.1}s ws={:.4}s | margin={:.3}dB worst_quiet={:+.3}dB | probe_decoded={:.0}s delta={:+.0}s",
            point.candidate.window_count,
            threshold,
            point.candidate.window_seconds,
            point.margin_db,
            point.worst_quiet_db,
            point.probe_decoded_secs,
            point.probe_decoded_secs - target_secs,
        );
    }

    // Provenance-dependent margin: a smaller margin for reliably-not-hot sources. Pass the
    // metadata pass (path, lossless, ytdlp JSONL) as the second argument to enable it.
    if let Some(meta_path) = args.get(2) {
        let safe = load_safe_paths(Path::new(meta_path))?;
        let candidate = Candidate {
            window_count: 28,
            window_seconds: 56.0 / 28.0,
            probe_margin_db: 0.0,
        };
        let split = provenance_margin(&truepeak_core::default_policy(), &tracks, candidate, &safe);
        println!(
            "\nprovenance-dependent margin at count=28 thr=56s ({} safe-provenance tracks):",
            safe.len()
        );
        println!(
            "  margin_safe={:.3}dB margin_unsafe={:.3}dB worst_quiet={:+.3}dB",
            split.margin_safe_db, split.margin_unsafe_db, split.worst_quiet_db
        );
    }

    return Ok(())
}
