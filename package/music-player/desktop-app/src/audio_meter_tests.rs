// What:     Unit tests for `audio_meter.rs`, pulled in by
//           `#[cfg(test)] #[path = "audio_meter_tests.rs"] mod tests;` at
//           the bottom of `audio_meter.rs`.
// Why:      The callback's counting and the once-per-second reporting are
//           pure logic once the clock is passed in, so they can be checked without audio.

// What:     `use super::*;`. Bring the module's items into the test scope.
// Why:      Tests use `AudioMeter`, `MeterReporter`, and `MeterSnapshot`.
use super::*;

// What:     `use std::time::{Duration, Instant};`. Fixed clock values for deterministic tests.
// Why:      Cadence depends only on the gaps between instants.
use std::time::{Duration, Instant};

#[test]
fn record_short_counts_callbacks_and_silent_frames() {
    let meter = AudioMeter::new();
    meter.record_short(64);
    meter.record_short(16);
    let snap = meter.snapshot();
    assert_eq!(snap.short_callbacks, 2);
    assert_eq!(snap.silent_frames, 80);
}

#[test]
fn record_buffer_keeps_the_latest_sizes() {
    let meter = AudioMeter::new();
    meter.record_buffer(1024, 4096);
    meter.record_buffer(2048, 2048);
    let snap = meter.snapshot();
    assert_eq!(snap.last_requested_frames, 2048);
    assert_eq!(snap.last_buffer_frames, 2048);
}

#[test]
fn since_subtracts_counters_and_keeps_latest_sizes() {
    let earlier = MeterSnapshot {
        short_callbacks: 3,
        silent_frames: 100,
        last_requested_frames: 1024,
        last_buffer_frames: 1024,
    };
    let later = MeterSnapshot {
        short_callbacks: 5,
        silent_frames: 300,
        last_requested_frames: 2048,
        last_buffer_frames: 4096,
    };
    let change = later.since(earlier);
    assert_eq!(change.short_callbacks, 2);
    assert_eq!(change.silent_frames, 200);
    assert_eq!(change.last_requested_frames, 2048);
    assert_eq!(change.last_buffer_frames, 4096);
}

#[test]
fn since_never_goes_negative() {
    let earlier = MeterSnapshot {
        short_callbacks: 9,
        ..MeterSnapshot::default()
    };
    let change = MeterSnapshot::default().since(earlier);
    assert_eq!(change.short_callbacks, 0);
}

#[test]
fn reporter_reads_the_first_time_and_skips_quiet_intervals() {
    let mut reporter = MeterReporter::new();
    let start = Instant::now();
    let first = reporter.take_change(start, MeterSnapshot::default);
    assert!(first.is_none());
}

#[test]
fn reporter_does_not_read_before_the_interval_elapses() {
    let mut reporter = MeterReporter::new();
    let start = Instant::now();
    reporter.take_change(start, MeterSnapshot::default);
    let mut read_called = false;
    let result = reporter.take_change(start + Duration::from_millis(999), || {
        read_called = true;
        return MeterSnapshot::default();
    });
    assert!(result.is_none());
    assert!(!read_called);
}

#[test]
fn reporter_returns_only_the_change_since_the_last_read() {
    let mut reporter = MeterReporter::new();
    let start = Instant::now();
    let after_first = MeterSnapshot {
        short_callbacks: 2,
        silent_frames: 40,
        ..MeterSnapshot::default()
    };
    let after_second = MeterSnapshot {
        short_callbacks: 5,
        silent_frames: 100,
        ..MeterSnapshot::default()
    };
    reporter.take_change(start, || return after_first);
    let change = reporter
        .take_change(start + REPORT_INTERVAL, || return after_second)
        .expect("a change was recorded in the second interval");
    assert_eq!(change.short_callbacks, 3);
    assert_eq!(change.silent_frames, 60);
}

#[test]
fn reporter_announces_buffer_sizes_once_then_stays_silent_without_underruns() {
    let mut reporter = MeterReporter::new();
    let start = Instant::now();
    let quiet_buffers = MeterSnapshot {
        last_requested_frames: 1024,
        last_buffer_frames: 4096,
        ..MeterSnapshot::default()
    };
    let announced = reporter.take_change(start, || return quiet_buffers);
    assert_eq!(announced.map(|change| return change.last_buffer_frames), Some(4096));
    let result = reporter.take_change(start + REPORT_INTERVAL, || return quiet_buffers);
    assert!(result.is_none());
}
