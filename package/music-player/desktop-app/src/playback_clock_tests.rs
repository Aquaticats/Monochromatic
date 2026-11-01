// What:     Unit tests for `playback_clock.rs`, pulled in by
//           `#[cfg(test)] #[path = "playback_clock_tests.rs"] mod tests;` at the bottom of `playback_clock.rs`.
// Why:      The audible position and the track-end decision are pure arithmetic, so they are checked without audio.
//           Each guard test fails if position goes back to counting pushed frames as played.

// What:     `use super::*;`. Bring the module's items into the test scope.
// Why:      Tests use `PlaybackClock`, `frames_of_samples` and the emit interval.
use super::*;

// What:     `use std::time::{Duration, Instant};`. Injected clock values for the drain hold.
// Why:      The hold depends only on the gaps between instants.
use std::time::{Duration, Instant};

// What:     `const RATE: u32 = 48_000;`. A common sample rate.
// Why:      One frame count per second keeps the expected values readable.
const RATE: u32 = 48_000;

#[test]
fn guard_pushed_frames_are_not_played_frames() {
    // The decoder filled the whole one-second ring and the output has taken nothing yet.
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    assert_eq!(clock.audible_frames(48_000), 0);
    assert!(clock.audible_secs(48_000, RATE).abs() < f64::EPSILON);
}

#[test]
fn guard_position_leads_by_nothing_after_the_output_takes_part_of_the_ring() {
    // 2.0 s pushed, 1.0 s still waiting: the listener has heard exactly 1.0 s.
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(96_000);
    assert_eq!(clock.audible_frames(48_000), 48_000);
    assert!((clock.audible_secs(48_000, RATE) - 1.0).abs() < 1e-9);
}

#[test]
fn consumed_everything_means_the_pushed_total_is_audible() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(30_000);
    clock.record_pushed(18_000);
    assert_eq!(clock.audible_frames(0), 48_000);
}

#[test]
fn restart_at_a_seek_target_starts_from_that_target() {
    // A seek to 5.0 s rebuilds the ring: the position is 5.0 s before the output consumed anything.
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(96_000);
    clock.restart(5 * 48_000);
    assert_eq!(clock.audible_frames(0), 240_000);
    assert!((clock.audible_secs(0, RATE) - 5.0).abs() < 1e-9);
    // Frames pushed after the seek count from the target, minus what is still queued.
    clock.record_pushed(48_000);
    assert_eq!(clock.audible_frames(48_000), 240_000);
    assert_eq!(clock.audible_frames(24_000), 264_000);
}

#[test]
fn restart_clears_the_pushed_count_and_the_drain() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    clock.begin_drain();
    clock.restart(0);
    assert!(!clock.is_draining());
    assert_eq!(clock.audible_frames(0), 0);
}

#[test]
fn queued_above_pushed_never_goes_below_the_base() {
    // A defensive case: the ring cannot hold more than was pushed, but the clock must not underflow.
    let mut clock = PlaybackClock::new();
    clock.restart(1_000);
    clock.record_pushed(10);
    assert_eq!(clock.audible_frames(99), 1_000);
}

#[test]
fn unknown_rate_reports_zero_seconds_and_no_update() {
    let mut clock = PlaybackClock::new();
    clock.record_pushed(48_000);
    assert!(clock.audible_secs(0, 0).abs() < f64::EPSILON);
    assert_eq!(clock.position_to_emit(0, 0), None);
}

#[test]
fn position_updates_wait_for_the_emit_interval_of_audible_progress() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(96_000);
    // Only 0.05 s consumed: below the interval.
    assert_eq!(clock.position_to_emit(96_000 - 2_400, RATE), None);
    // 0.1 s consumed: first update at 0.1 s.
    let first = clock.position_to_emit(96_000 - 4_800, RATE);
    assert!(first.is_some_and(|secs| return (secs - 0.1).abs() < 1e-9));
    // 0.15 s: too soon after the first update.
    assert_eq!(clock.position_to_emit(96_000 - 7_200, RATE), None);
    // 0.2 s: the next update.
    let second = clock.position_to_emit(96_000 - 9_600, RATE);
    assert!(second.is_some_and(|secs| return (secs - 0.2).abs() < 1e-9));
}

#[test]
fn guard_a_full_ring_emits_no_position_ahead_of_the_sound() {
    // The old behavior emitted 1.0 s of "progress" while the output had consumed nothing.
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    assert_eq!(clock.position_to_emit(48_000, RATE), None);
}

#[test]
fn a_restart_at_a_seek_target_does_not_emit_a_redundant_update() {
    let mut clock = PlaybackClock::new();
    clock.restart(10 * 48_000);
    clock.record_pushed(48_000);
    assert_eq!(clock.position_to_emit(48_000, RATE), None);
    let after = clock.position_to_emit(48_000 - 4_800, RATE);
    assert!(after.is_some_and(|secs| return (secs - 10.1).abs() < 1e-9));
}

#[test]
fn begin_drain_reports_only_the_first_call() {
    let mut clock = PlaybackClock::new();
    assert!(!clock.is_draining());
    assert!(clock.begin_drain());
    assert!(clock.is_draining());
    assert!(!clock.begin_drain());
}

#[test]
fn guard_the_track_end_waits_until_the_last_pushed_frame_is_consumed() {
    // The old behavior ended the track when the decoder finished, with the whole ring still queued.
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    clock.begin_drain();
    let now = Instant::now();
    assert!(!clock.drain_finished(48_000, now, Duration::ZERO));
    assert!(!clock.drain_finished(1, now, Duration::ZERO));
    assert!(clock.drain_finished(0, now, Duration::ZERO));
}

#[test]
fn finish_drain_clears_the_flag_without_touching_the_counts() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    clock.begin_drain();
    clock.finish_drain();
    assert!(!clock.is_draining());
    assert!(!clock.drain_finished(0, Instant::now(), Duration::ZERO));
    assert_eq!(clock.audible_frames(0), 48_000);
}

#[test]
fn nothing_is_finished_while_the_decoder_is_still_running() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    assert!(!clock.drain_finished(0, Instant::now(), Duration::ZERO));
}

#[test]
fn frames_of_samples_divides_samples_by_channels() {
    assert_eq!(frames_of_samples(96_000, 2), 48_000);
    assert_eq!(frames_of_samples(48_000, 1), 48_000);
    assert_eq!(frames_of_samples(5, 2), 2);
}

#[test]
fn frames_of_samples_of_an_unknown_channel_count_is_zero() {
    assert_eq!(frames_of_samples(96_000, 0), 0);
}

#[test]
fn the_emit_interval_is_a_tenth_of_a_second() {
    assert!((POSITION_EMIT_INTERVAL_SECS - 0.1).abs() < f64::EPSILON);
}

#[test]
fn a_boundary_is_not_reached_while_frames_before_it_are_still_queued() {
    // Track one pushed 48 000 frames, then the join, then track two pushed 24 000 frames.
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    clock.mark_boundary();
    clock.record_pushed(24_000);
    assert!(clock.has_boundary());
    // 72 000 pushed and 72 000 queued: nothing consumed.
    assert!(!clock.boundary_reached(72_000));
    // One frame before the join is still queued.
    assert!(!clock.boundary_reached(24_001));
    // Everything before the join was consumed; 24 000 frames of track two wait in the ring.
    assert!(clock.boundary_reached(24_000));
}

#[test]
fn the_old_track_keeps_counting_until_the_boundary_is_crossed() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    clock.mark_boundary();
    clock.record_pushed(24_000);
    // 36 000 consumed: still inside track one.
    assert_eq!(clock.audible_frames(36_000), 36_000);
    // 48 000 consumed: track one is complete.
    assert_eq!(clock.audible_frames(24_000), 48_000);
}

#[test]
fn crossing_the_boundary_restarts_the_position_at_the_new_track() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    clock.mark_boundary();
    clock.record_pushed(24_000);
    clock.cross_boundary();
    assert!(!clock.has_boundary());
    // Right at the join the new track is at zero; 12 000 frames later it is at 12 000.
    assert_eq!(clock.audible_frames(24_000), 0);
    assert_eq!(clock.audible_frames(12_000), 12_000);
}

#[test]
fn crossing_resets_the_emit_baseline_so_the_new_track_reports_from_zero() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(96_000);
    let late = clock.position_to_emit(96_000 - 48_000, RATE);
    assert!(late.is_some_and(|secs| return (secs - 1.0).abs() < 1e-9));
    clock.mark_boundary();
    clock.record_pushed(48_000);
    clock.cross_boundary();
    // Just after the join the new track is at 0.1 s: it must be reported even though the old baseline was 1.0 s.
    let early = clock.position_to_emit(144_000 - (96_000 + 4_800), RATE);
    assert!(early.is_some_and(|secs| return (secs - 0.1).abs() < 1e-9));
}

#[test]
fn restart_forgets_a_pending_boundary() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    clock.mark_boundary();
    clock.restart(0);
    assert!(!clock.has_boundary());
    assert!(!clock.boundary_reached(0));
}

#[test]
fn crossing_without_a_boundary_changes_nothing() {
    let mut clock = PlaybackClock::new();
    clock.restart(10 * 48_000);
    clock.record_pushed(48_000);
    clock.cross_boundary();
    assert_eq!(clock.audible_frames(24_000), 10 * 48_000 + 24_000);
}

#[test]
fn the_drain_hold_starts_when_the_ring_is_first_seen_empty() {
    // The output keeps some consumed audio of its own: the next track waits `hold` after the ring emptied.
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    clock.begin_drain();
    let start = Instant::now();
    let hold = Duration::from_millis(50);
    assert!(!clock.drain_finished(0, start, hold));
    assert!(!clock.drain_finished(0, start + Duration::from_millis(49), hold));
    assert!(clock.drain_finished(0, start + Duration::from_millis(50), hold));
}

#[test]
fn guard_a_ring_that_refills_restarts_the_drain_hold() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    clock.begin_drain();
    let start = Instant::now();
    let hold = Duration::from_millis(50);
    assert!(!clock.drain_finished(0, start, hold));
    // Frames were queued again (a pause and resume race): the hold must start over from the next empty ring.
    assert!(!clock.drain_finished(10, start + Duration::from_millis(40), hold));
    assert!(!clock.drain_finished(0, start + Duration::from_millis(60), hold));
    assert!(!clock.drain_finished(0, start + Duration::from_millis(109), hold));
    assert!(clock.drain_finished(0, start + Duration::from_millis(110), hold));
}

#[test]
fn drain_wait_reports_the_time_left_of_the_hold() {
    let mut clock = PlaybackClock::new();
    clock.restart(0);
    clock.record_pushed(48_000);
    clock.begin_drain();
    let start = Instant::now();
    let hold = Duration::from_millis(50);
    // No hold is running while frames are queued.
    assert_eq!(clock.drain_wait(start, hold), None);
    assert!(!clock.drain_finished(0, start, hold));
    assert_eq!(clock.drain_wait(start, hold), Some(Duration::from_millis(50)));
    assert_eq!(clock.drain_wait(start + Duration::from_millis(20), hold), Some(Duration::from_millis(30)));
    assert_eq!(clock.drain_wait(start + Duration::from_millis(80), hold), Some(Duration::ZERO));
}
