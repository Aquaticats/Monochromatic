//! The gain decision a resolve produces,
//!  and the kinds of decision.
//!
//! A decision is the shared service's answer for one track:
//!  the constant linear gain to
//! apply,
//!  how it was reached (a short full scan,
//!  a proportional probe with margin,
//!  or a
//! full exact scan),
//!  the peak it was based on,
//!  and the track length.
//!  The cache stores
//! these;
//!  playback applies the `gain`.

/// What:
///      `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum DecisionKind`.
///  The
///           three ways the policy reaches a gain.
///  `Eq` is valid because the variants
///           are plain tags.
///  Sibling shape:
///  a bool "exact or not";
///  an enum keeps the
///           three cases distinct for the cache and for debugging.
/// Why:
///       A reader (and the cache) must tell an exact decision from a probe estimate,
///           since only probe estimates carry the margin and can be improved by warming.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DecisionKind = "shortFullScan" | "probeEstimate" | "fullScanExact";
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionKind {
    /// What:
    ///      `ShortFullScan`.
    ///  The track was at or below the short-scan cutoff,
    ///  so it
    ///           was scanned in full;
    ///  the peak and gain are exact.
    /// Why:
    ///       Short tracks are cheap to scan exactly,
    ///  so they carry no probe error.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "shortFullScan"
    /// ```
    ShortFullScan,
    /// What:
    ///      `ProbeEstimate`.
    ///  A long track was probed at proportional coverage;
    ///  the
    ///           gain is from the sampled peak inflated by the fixed margin.
    /// Why:
    ///       This is the estimate that may be a little too quiet,
    ///  and that background
    ///           warming can later replace with an exact full scan.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "probeEstimate"
    /// ```
    ProbeEstimate,
    /// What:
    ///      `FullScanExact`.
    ///  A long track was scanned in full:
    ///  either its duration
    ///           was unknown so the policy could not probe,
    ///  or warming upgraded it.
    /// Why:
    ///       An exact decision for a long track,
    ///  strictly better evidence than a
    ///           probe estimate for the same track.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "fullScanExact"
    /// ```
    FullScanExact,
}

/// What:
///      `#[derive(Clone, Copy, Debug, PartialEq)] pub struct Decision`.
///  The service's
///           answer for one track:
///  the gain to apply and the evidence behind it.
/// Why:
///       Playback needs the gain;
///  the cache needs the kind and peak to decide whether a
///           later,
///  better decision may replace it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Decision = { gain: number; kind: DecisionKind; measuredPeak: number; durationSecs: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decision {
    /// What:
    ///      `pub gain: f32`.
    ///  The constant linear gain to multiply every sample by;
    ///           never above 1.0.
    ///  `f32` (sibling `f64`) to match the audio sample type.
    /// Why:
    ///       This is what playback applies.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// gain: number;
    /// ```
    pub gain: f32,
    /// What:
    ///      `pub kind: DecisionKind`.
    ///  How the gain was reached.
    /// Why:
    ///       Distinguishes an exact decision from an improvable probe estimate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// kind: DecisionKind;
    /// ```
    pub kind: DecisionKind,
    /// What:
    ///      `pub measured_peak: f32`.
    ///  The peak the gain was computed from:
    ///  the exact
    ///           true peak for a full scan,
    ///  or the sampled peak for a probe.
    ///  `f32`
    ///           (sibling `f64`) to match the meter.
    /// Why:
    ///       Lets a future reader see the basis of the gain and compare a later scan.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// measuredPeak: number;
    /// ```
    pub measured_peak: f32,
    /// What:
    ///      `pub duration_secs: f64`.
    ///  The decoded track length.
    ///  `f64` (sibling `f32`)
    ///           for precision on long tracks.
    /// Why:
    ///       Stored for debugging and to recompute the branch under a policy change.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// durationSecs: number;
    /// ```
    pub duration_secs: f64,
}

/// What:
///      `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum PeakKind`.
///  Whether a stored peak is the exact true peak or a sampled estimate.
/// Why:
///       A reader of a cached peak must know if the value is a full-scan maximum
///  or a probe's lower bound before showing it as the track's true peak.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PeakKind = "exact" | "sampled";
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeakKind {
    /// What:
    ///      `Exact`.
    ///  The peak came from a full scan (short or full).
    /// Why:
    ///       The value is the track's true peak.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "exact"
    /// ```
    Exact,
    /// What:
    ///      `Sampled`.
    ///  The peak came from a proportional probe and may sit below the true peak.
    /// Why:
    ///       The value is an estimate until a warming scan replaces it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// "sampled"
    /// ```
    Sampled,
}

impl Decision {
    /// What:
    ///      `pub fn peak_kind(&self) -> PeakKind`.
    ///  Classify the decision's peak as exact or sampled from its kind.
    /// Why:
    ///       Callers read the kind without matching the three decision tags themselves.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// peakKind(): PeakKind { return this.kind === "probeEstimate" ? "sampled" : "exact"; }
    /// ```
    pub fn peak_kind(&self) -> PeakKind {
        // What:     `match self.kind { ... }`. The two full scans are exact; the probe is sampled.
        // Why:      Only the probe carries the margin and the sampled-peak basis.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (this.kind) { case "probeEstimate": return "sampled"; default: return "exact"; }
        // ```
        match self.kind {
            DecisionKind::ShortFullScan | DecisionKind::FullScanExact => return PeakKind::Exact,
            DecisionKind::ProbeEstimate => return PeakKind::Sampled,
        }
    }
}

#[cfg(test)]
#[path = "decision_tests.rs"]
mod tests;
