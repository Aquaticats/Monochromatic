// What:     Unit test for the desktop `truepeak.rs`, pulled in by
//           `#[cfg(test)] #[path = "truepeak_tests.rs"] mod tests;` at the bottom of
//           `truepeak.rs`. Compiles only under `cargo nextest run` / `cargo test`; reaches
//           the module items via `use super::*` because this file is the `tests` CHILD of
//           truepeak.
// Why:      Keep the test beside the code without inflating `truepeak.rs` or its max-lines
//           budget (sibling `*_tests.rs` files are exempt from the linter).
//
// Only the end-to-end fixture decode test remains here. The gain math
// (`normalization_gain`) and the Catmull-Rom spline are now owned by the shared
// `truepeak-core` crate and tested there (its `gain_tests.rs` and `meter_tests.rs`), so the
// former desktop copies of those tests were removed rather than duplicated.

// What:     `use super::*;`. Bring the parent module's items into the test scope, including
//           its `pub(crate)` `resolve_full` and its imported `Path`.
// Why:      The test calls `resolve_full(Path::new(...))`.
use super::*;

// What:     `use truepeak_core::DecisionKind;`. The decision-kind tag.
// Why:      A short fixture full-scans, so the returned kind must be `ShortFullScan`.
use truepeak_core::DecisionKind;

// What:     `#[test]` resolving a committed fixture end-to-end.
// Why:      Prove the desktop decode loop plus the shared resolver produce a sane exact
//           decision on real audio (this is the desktop-specific coverage the shared crate
//           cannot give, because it does not own the desktop decoder).
#[test]
fn resolve_full_of_fixture_is_sane() {
    // What:     Full-scan the committed 440 Hz tone.
    // Why:      Exercise a real decode through the desktop adapter into the shared resolver.
    let decision =
        resolve_full(Path::new("fixture/tone.flac")).expect("tone fixture should resolve");
    // What:     `assert_eq!(decision.kind, DecisionKind::ShortFullScan);`. The tiny fixture is
    //           short, so the exact scan tags it short.
    // Why:      Confirm the desktop adapter reports a real duration the policy reads as short.
    assert_eq!(decision.kind, DecisionKind::ShortFullScan);
    // What:     `assert!(decision.measured_peak > 0.05 && decision.measured_peak < 0.2, ...)`.
    //           The committed tone sits at about -21 dBFS (verified with `ffmpeg
    //           volumedetect`: max_volume -21.1 dB, about 0.088 linear); the measured true
    //           peak is just above the raw sample peak, exactly the inter-sample overshoot we
    //           look for.
    // Why:      Confirm the scan decodes end-to-end and reports the real level (not silence,
    //           not an absurd value).
    assert!(
        decision.measured_peak > 0.05 && decision.measured_peak < 0.2,
        "measured peak was {}",
        decision.measured_peak
    );
    // What:     `assert_eq!(decision.gain, 1.0);`. The tone is far below the ceiling, so the
    //           attenuate-only gain is unity.
    // Why:      A quiet track must pass through unchanged.
    assert_eq!(decision.gain, 1.0);
}
