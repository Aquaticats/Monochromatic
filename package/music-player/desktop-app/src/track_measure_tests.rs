// What:     Unit tests for `track_measure.rs`, pulled in by `#[cfg(test)] #[path = "track_measure_tests.rs"] mod tests;`.
// Why:      Keep the measure rules beside their code; the cases cover the exact and sampled paths, the
//           silent infinity branch, and the duration and peak fill of the template fields.

// What:     `use super::*;` imports the measure items into the test scope.
// Why:      The tests call the public functions of the parent module directly.
use super::*;

// What:     `use truepeak_core::{Decision, DecisionKind};` names the decision kinds the cases build.
// Why:      A decision is the input of `measure_from_decision`, and its kind picks the exact flag.
use truepeak_core::{Decision, DecisionKind};

// What:     `fn decision(kind, measured_peak, duration_secs) -> Decision` builds one cached decision.
// Why:      Only the peak, the kind and the length matter here, so the gain is a fixed placeholder.
fn decision(kind: DecisionKind, measured_peak: f32, duration_secs: f64) -> Decision {
    return Decision {
        gain: 1.0,
        kind,
        measured_peak,
        duration_secs,
    };
}

// What:     `fn fields() -> TrackFields` builds empty template fields for the fill cases.
// Why:      The fill only overwrites `len` and `peak`, so the other fields can stay blank.
fn fields() -> TrackFields {
    return TrackFields {
        title: "Song".to_string(),
        file: "Song".to_string(),
        ext: "flac".to_string(),
        folder: String::new(),
        path: "Song.flac".to_string(),
        len: None,
        peak: None,
    };
}

#[test]
fn formats_negative_peak_with_u_plus_2212_minus() {
    assert_eq!(format_peak_dbtp(-21.1), "\u{2212}21.1 dBTP");
}

#[test]
fn formats_positive_peak_with_plus_sign() {
    assert_eq!(format_peak_dbtp(1.26), "+1.3 dBTP");
}

#[test]
fn formats_zero_and_near_zero_without_sign() {
    assert_eq!(format_peak_dbtp(0.0), "0.0 dBTP");
    assert_eq!(format_peak_dbtp(-0.04), "0.0 dBTP");
}

#[test]
fn formats_minus_infinity_as_infinity_mark_with_minus() {
    assert_eq!(format_peak_dbtp(f64::NEG_INFINITY), "\u{2212}\u{221E} dBTP");
}

#[test]
fn formats_plus_infinity_as_infinity_mark_with_plus() {
    assert_eq!(format_peak_dbtp(f64::INFINITY), "+\u{221E} dBTP");
}

#[test]
fn exact_peak_text_formats_a_full_scan_peak() {
    let measure = TrackMeasure {
        duration_secs: Some(20.0),
        peak_dbtp: Some(-21.1),
        peak_exact: true,
    };
    assert_eq!(exact_peak_text(Some(&measure)), Some("\u{2212}21.1 dBTP".to_string()));
}

#[test]
fn exact_peak_text_drops_a_sampled_peak() {
    let measure = TrackMeasure {
        duration_secs: Some(20.0),
        peak_dbtp: Some(-21.1),
        peak_exact: false,
    };
    assert_eq!(exact_peak_text(Some(&measure)), None);
}

#[test]
fn exact_peak_text_is_none_without_a_peak_or_a_measure() {
    let no_peak = TrackMeasure {
        duration_secs: Some(20.0),
        peak_dbtp: None,
        peak_exact: true,
    };
    assert_eq!(exact_peak_text(Some(&no_peak)), None);
    assert_eq!(exact_peak_text(None), None);
}

#[test]
fn exact_peak_text_drops_a_nan_peak() {
    let measure = TrackMeasure {
        duration_secs: None,
        peak_dbtp: Some(f64::NAN),
        peak_exact: true,
    };
    assert_eq!(exact_peak_text(Some(&measure)), None);
}

#[test]
fn measure_from_full_scan_decision_is_exact_with_its_length() {
    let measure = measure_from_decision(&decision(DecisionKind::FullScanExact, 0.0891, 20.0));
    assert!(measure.peak_exact);
    assert_eq!(measure.duration_secs, Some(20.0));
    let peak = measure.peak_dbtp.expect("a full scan carries a peak");
    assert!((peak + 21.0).abs() < 0.05, "peak was {peak}");
}

#[test]
fn measure_from_probe_estimate_decision_is_sampled() {
    let measure = measure_from_decision(&decision(DecisionKind::ProbeEstimate, 0.5, 20.0));
    assert!(!measure.peak_exact);
    assert!(measure.peak_dbtp.is_some());
}

#[test]
fn measure_from_silent_decision_keeps_minus_infinity() {
    let measure = measure_from_decision(&decision(DecisionKind::FullScanExact, 0.0, 20.0));
    assert_eq!(measure.peak_dbtp, Some(f64::NEG_INFINITY));
    assert_eq!(exact_peak_text(Some(&measure)), Some("\u{2212}\u{221E} dBTP".to_string()));
}

#[test]
fn measure_from_decision_drops_a_zero_length() {
    let measure = measure_from_decision(&decision(DecisionKind::FullScanExact, 0.5, 0.0));
    assert_eq!(measure.duration_secs, None);
}

#[test]
fn measure_from_duration_carries_only_the_length() {
    let measure = measure_from_duration(Some(200.0));
    assert_eq!(measure.duration_secs, Some(200.0));
    assert_eq!(measure.peak_dbtp, None);
    assert!(!measure.peak_exact);
}

#[test]
fn measure_from_duration_drops_missing_and_negative_lengths() {
    assert_eq!(measure_from_duration(None).duration_secs, None);
    assert_eq!(measure_from_duration(Some(-1.0)).duration_secs, None);
}

#[test]
fn merge_keeps_earlier_paths_and_overwrites_updated_ones() {
    let first = PathBuf::from("/music/a.flac");
    let second = PathBuf::from("/music/b.flac");
    let old = TrackMeasure {
        duration_secs: Some(10.0),
        ..TrackMeasure::default()
    };
    let new = TrackMeasure {
        duration_secs: Some(20.0),
        ..TrackMeasure::default()
    };
    let current = HashMap::from([(first.clone(), old)]);
    let merged = merge_measures(&current, &[(first.clone(), new), (second.clone(), old)]);
    assert_eq!(merged.get(&first), Some(&new));
    assert_eq!(merged.get(&second), Some(&old));
    assert_eq!(current.get(&first), Some(&old));
}

#[test]
fn lookup_by_name_keys_each_known_measure_by_its_queue_name() {
    let names = vec!["a.flac".to_string(), "b.flac".to_string()];
    let paths = vec![PathBuf::from("/x/a.flac"), PathBuf::from("/x/b.flac")];
    let known = TrackMeasure {
        duration_secs: Some(5.0),
        ..TrackMeasure::default()
    };
    let measures = HashMap::from([(PathBuf::from("/x/b.flac"), known)]);
    let lookup = lookup_by_name(&names, &paths, &measures);
    assert_eq!(lookup, HashMap::from([("b.flac".to_string(), known)]));
}

#[test]
fn fields_with_exact_measure_carry_length_and_peak() {
    let measure = TrackMeasure {
        duration_secs: Some(20.0),
        peak_dbtp: Some(-21.1),
        peak_exact: true,
    };
    let filled = fields_with_measure(fields(), Some(&measure));
    assert_eq!(filled.len, Some(20.0));
    assert_eq!(filled.peak, Some("\u{2212}21.1 dBTP".to_string()));
    assert_eq!(filled.title, "Song");
}

#[test]
fn fields_with_sampled_measure_carry_length_only() {
    let measure = TrackMeasure {
        duration_secs: Some(20.0),
        peak_dbtp: Some(-21.1),
        peak_exact: false,
    };
    let filled = fields_with_measure(fields(), Some(&measure));
    assert_eq!(filled.len, Some(20.0));
    assert_eq!(filled.peak, None);
}

#[test]
fn fields_without_measure_stay_unknown() {
    let filled = fields_with_measure(fields(), None);
    assert_eq!(filled.len, None);
    assert_eq!(filled.peak, None);
}
