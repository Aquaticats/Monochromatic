//! Stall-detection tests using injected times,
//!  with no display,
//!  sleeping,
//!  or host state.

use super::*;

/// A parent that presented within the threshold keeps ownership of the frame cadence.
#[test]
fn presenting_parent_needs_no_fallback() {
    assert!(!parent_stalled(false, Duration::ZERO));
    assert!(!parent_stalled(false, FRAME_INTERVAL));
    assert!(!parent_stalled(false, STALL_THRESHOLD - Duration::from_nanos(1)));
}

/// A locked or hidden parent stops presenting;
///  the hosted client must still be paced.
#[test]
fn silent_parent_triggers_fallback_from_the_threshold_on() {
    assert!(parent_stalled(false, STALL_THRESHOLD));
    assert!(parent_stalled(false, Duration::from_secs(3600)));
}

/// While recording,
///  the recorder's own timer defines the cadence,
///  however long the parent is silent.
#[test]
fn recorder_keeps_ownership_of_the_frame_cadence() {
    assert!(!parent_stalled(true, STALL_THRESHOLD));
    assert!(!parent_stalled(true, Duration::from_secs(3600)));
}

/// Only a recording that sees a real change of size or scale sends the callbacks early.
#[test]
fn early_callbacks_follow_a_geometry_change_while_recording() {
    assert!(resize_needs_early_callbacks(true, true));
    // The live redraw path already answers the client after every presentation.
    assert!(!resize_needs_early_callbacks(false, true));
    // A command that leaves the geometry as it was gives the client nothing new to draw.
    assert!(!resize_needs_early_callbacks(true, false));
    assert!(!resize_needs_early_callbacks(false, false));
}

/// The fallback starts once the parent goes silent and stops when the parent presents again.
#[test]
fn fallback_follows_parent_presentation() {
    let start = Instant::now();
    let mut pacing = FramePacing::new(start);
    assert!(!pacing.fallback_due(false, start + FRAME_INTERVAL));
    assert!(pacing.fallback_due(false, start + STALL_THRESHOLD));
    // The fallback's own ticks do not count as presentation, so it keeps pacing every frame.
    assert!(pacing.fallback_due(false, start + STALL_THRESHOLD + FRAME_INTERVAL));
    let resumed = start + Duration::from_secs(5);
    pacing.parent_presented(resumed);
    assert!(!pacing.fallback_due(false, resumed + FRAME_INTERVAL));
    assert!(pacing.fallback_due(false, resumed + STALL_THRESHOLD));
}

/// A time earlier than the last presentation is treated as no silence,
///  never as a panic.
#[test]
fn earlier_time_is_not_a_stall() {
    let start = Instant::now();
    let mut pacing = FramePacing::new(start + Duration::from_secs(1));
    assert!(!pacing.fallback_due(false, start));
}

/// The frame interval is derived from the advertised 60 Hz output refresh.
#[test]
fn frame_interval_matches_the_advertised_refresh() {
    assert_eq!(FRAME_INTERVAL, Duration::from_nanos(16_666_666));
    assert!(STALL_THRESHOLD >= FRAME_INTERVAL * 2);
}

/// Headless has no parent presentation,
///  so a callback is due on every tick from the first one,
///  with no stall threshold and for any amount of elapsed time.
#[test]
fn headless_sends_callbacks_on_every_tick_from_the_start() {
    assert!(callbacks_due(BackendKind::Headless, false, Duration::ZERO));
    assert!(callbacks_due(BackendKind::Headless, false, FRAME_INTERVAL));
    assert!(callbacks_due(BackendKind::Headless, false, Duration::from_secs(3600)));
    let start = Instant::now();
    let mut pacing = FramePacing::for_backend(BackendKind::Headless, start);
    assert!(pacing.fallback_due(false, start));
    assert!(pacing.fallback_due(false, start + FRAME_INTERVAL));
}

/// While recording,
///  the recorder's timer is the only source in headless mode too.
#[test]
fn headless_leaves_the_cadence_to_the_recorder_while_recording() {
    assert!(!callbacks_due(BackendKind::Headless, true, Duration::ZERO));
    assert!(!callbacks_due(BackendKind::Headless, true, Duration::from_secs(3600)));
    let start = Instant::now();
    let mut pacing = FramePacing::for_backend(BackendKind::Headless, start);
    assert!(!pacing.fallback_due(true, start + STALL_THRESHOLD));
    assert!(pacing.fallback_due(false, start + STALL_THRESHOLD));
}

/// The nested window keeps today's decision exactly:
///  due only once the parent has been silent for the threshold.
#[test]
fn nested_window_pacing_is_unchanged() {
    for recording in [false, true] {
        for millis in [0, 16, 49, 50, 51, 3_600_000] {
            let since = Duration::from_millis(millis);
            assert_eq!(
                callbacks_due(BackendKind::NestedWinit, recording, since),
                parent_stalled(recording, since),
                "recording={recording} since={millis}ms",
            );
        }
    }
    let start = Instant::now();
    let mut nested = FramePacing::for_backend(BackendKind::NestedWinit, start);
    assert!(!nested.fallback_due(false, start));
    assert!(nested.fallback_due(false, start + STALL_THRESHOLD));
}
