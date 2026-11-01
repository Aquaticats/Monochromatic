//! Tests for the peak classification of a decision.

use super::*;

// A decision with the given kind; the gain and peak do not affect the classification.
fn decision(kind: DecisionKind) -> Decision {
    return Decision { gain: 0.5, kind, measured_peak: 0.9, duration_secs: 200.0 }
}

// Both full-scan kinds are exact peaks.
#[test]
fn full_scans_are_exact() {
    assert_eq!(decision(DecisionKind::ShortFullScan).peak_kind(), PeakKind::Exact);
    assert_eq!(decision(DecisionKind::FullScanExact).peak_kind(), PeakKind::Exact);
}

// The probe is the one sampled peak, and a positive control proves the mapping can differ.
#[test]
fn probe_estimate_is_sampled() {
    assert_eq!(decision(DecisionKind::ProbeEstimate).peak_kind(), PeakKind::Sampled);
    assert_ne!(
        decision(DecisionKind::ProbeEstimate).peak_kind(),
        decision(DecisionKind::FullScanExact).peak_kind(),
    );
}
