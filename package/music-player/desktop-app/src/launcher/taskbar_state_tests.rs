// What:     Unit tests for `taskbar_state.rs`, pulled in by `#[cfg(test)] #[path = "taskbar_state_tests.rs"] mod tests;`.
// Why:      The taskbar decision has no window and no bus, so every rule is checked with plain values.

use super::{TaskbarInput, TaskbarProgress, TaskbarUpdate, MIN_UPDATE_GAP};
use crate::command::Update;
use std::time::Duration;

/// What: a shown bar at `percent`.
fn shown(percent: u8) -> TaskbarUpdate {
    return TaskbarUpdate {
        percent,
        visible: true,
    };
}

/// What: `Duration` of `ms` milliseconds since the app started.
fn at(ms: u64) -> Duration {
    return Duration::from_millis(ms);
}

/// What: a machine that is playing a track of `duration_secs`, at position zero, with the opening values consumed.
fn playing_track(duration_secs: f64) -> TaskbarProgress {
    let mut machine = TaskbarProgress::new();
    machine.apply(TaskbarInput::NowPlaying { duration_secs }, at(0));
    machine.apply(TaskbarInput::Playing(true), at(0));
    return machine;
}

/// What: the engine's message pair at a track change, `NowPlaying` then `Position(0)`.
fn now_playing(duration: f64) -> Update {
    return Update::NowPlaying {
        index: Some(0),
        name: String::from("track"),
        duration,
    };
}

#[test]
fn a_fresh_machine_hides_the_bar_once_and_then_stays_quiet() {
    let mut machine = TaskbarProgress::new();
    assert_eq!(machine.apply(TaskbarInput::Idle, at(0)), Some(TaskbarUpdate::HIDDEN));
    assert_eq!(machine.apply(TaskbarInput::Idle, at(1000)), None);
}

#[test]
fn a_loaded_track_that_has_not_started_shows_no_bar() {
    let mut machine = TaskbarProgress::new();
    machine.apply(TaskbarInput::Idle, at(0));
    assert_eq!(machine.apply(TaskbarInput::NowPlaying { duration_secs: 120.0 }, at(10)), None);
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 0.0 }, at(20)), None);
}

#[test]
fn playing_a_loaded_track_shows_the_bar_at_zero() {
    let mut machine = TaskbarProgress::new();
    machine.apply(TaskbarInput::NowPlaying { duration_secs: 120.0 }, at(0));
    assert_eq!(machine.apply(TaskbarInput::Playing(true), at(10)), Some(shown(0)));
}

#[test]
fn ticks_inside_one_percent_send_nothing() {
    let mut machine = playing_track(120.0);
    // 120 s long: one percent is 1.2 s, so ticks every 0.1 s change the percent about every twelfth tick.
    machine.apply(TaskbarInput::Position { secs: 0.1 }, at(300));
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 0.2 }, at(600)), None);
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 1.1 }, at(900)), None);
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 1.2 }, at(1200)), Some(shown(1)));
}

#[test]
fn ordinary_ticks_never_send_faster_than_the_gap_and_the_change_is_not_lost() {
    // 10 s long: one percent is 0.1 s, so every tick changes the percent.
    let mut machine = playing_track(10.0);
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 1.0 }, at(1000)), Some(shown(10)));
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 1.1 }, at(1100)), None);
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 1.2 }, at(1200)), None);
    let after_gap = machine.apply(TaskbarInput::Position { secs: 1.3 }, at(1000 + MIN_UPDATE_GAP.as_millis() as u64));
    assert_eq!(after_gap, Some(shown(13)));
}

#[test]
fn the_percent_is_floored_so_it_reaches_one_hundred_only_at_the_end() {
    let mut machine = playing_track(10.0);
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 9.99 }, at(1000)), Some(shown(99)));
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 10.0 }, at(2000)), Some(shown(100)));
    // A position past the end (a late tick) stays at one hundred and is not a change.
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 10.4 }, at(3000)), None);
}

#[test]
fn the_fraction_rounds_back_to_the_percent_under_plasmas_rounding() {
    // Plasma shows qRound(progress * 100): the fraction must survive that for every percent, including both ends.
    for percent in 0..=100u8 {
        let fraction = shown(percent).fraction();
        let seen_by_plasma = (fraction * 100.0).round() as u8;
        assert_eq!(seen_by_plasma, percent, "percent {percent} fraction {fraction}");
    }
    assert_eq!(shown(0).fraction(), 0.0);
    assert_eq!(shown(100).fraction(), 1.0);
}

#[test]
fn guard_a_track_change_resets_the_percent_with_the_new_duration_and_never_uses_the_old_position() {
    // The old bug: NowPlaying read the old position (108 s) against the new duration (95 s) before Position(0) was applied.
    let mut machine = playing_track(120.0);
    machine.apply(TaskbarInput::Position { secs: 108.0 }, at(1000));
    let mut sent = Vec::new();
    // The engine sends NowPlaying and Position(0) back to back, well inside the rate-limit gap.
    for update in [now_playing(95.0), Update::Position(0.0)] {
        if let Some(value) = machine.apply_update(&update, at(1010)) {
            sent.push(value);
        }
    }
    assert_eq!(sent, vec![shown(0)], "the track change must send exactly one value, 0 percent");
    // The first tick of the new track is a fraction of the new duration only.
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 9.5 }, at(2000)), Some(shown(10)));
}

#[test]
fn guard_a_track_change_is_sent_even_when_the_last_send_was_a_moment_ago() {
    let mut machine = playing_track(10.0);
    machine.apply(TaskbarInput::Position { secs: 5.0 }, at(1000));
    let reset = machine.apply_update(&now_playing(10.0), at(1001));
    assert_eq!(reset, Some(shown(0)));
}

#[test]
fn a_tick_behind_the_position_is_dropped_so_the_percent_stays_monotonic() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 50.0 }, at(1000));
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 49.9 }, at(2000)), None);
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 50.0 }, at(3000)), None);
}

#[test]
fn a_seek_back_is_sent_at_once_even_inside_the_gap() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 50.0 }, at(1000));
    assert_eq!(machine.apply(TaskbarInput::Seek { secs: 10.0 }, at(1005)), Some(shown(10)));
}

#[test]
fn a_seek_forward_is_sent_at_once_even_inside_the_gap() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 1.0 }, at(1000));
    assert_eq!(machine.apply(TaskbarInput::Seek { secs: 80.0 }, at(1005)), Some(shown(80)));
}

#[test]
fn a_position_message_that_jumps_is_classified_as_a_seek_and_a_small_step_as_a_tick() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 30.0 }, at(1000));
    assert!(matches!(
        machine.input_for(&Update::Position(30.1)),
        Some(TaskbarInput::Position { .. })
    ));
    assert!(matches!(
        machine.input_for(&Update::Position(31.0)),
        Some(TaskbarInput::Position { .. })
    ));
    assert!(matches!(machine.input_for(&Update::Position(10.0)), Some(TaskbarInput::Seek { .. })));
    assert!(matches!(machine.input_for(&Update::Position(29.9)), Some(TaskbarInput::Seek { .. })));
    assert!(matches!(machine.input_for(&Update::Position(60.0)), Some(TaskbarInput::Seek { .. })));
}

#[test]
fn a_short_seek_back_on_a_short_track_moves_the_bar() {
    // One second back on a 10 s track is ten percent; it must not be mistaken for a stale tick.
    let mut machine = playing_track(10.0);
    machine.apply(TaskbarInput::Position { secs: 5.0 }, at(1000));
    assert_eq!(machine.apply_update(&Update::Position(4.0), at(1005)), Some(shown(40)));
}

#[test]
fn a_fast_backward_drag_while_playing_sends_once_and_the_next_tick_sends_the_latest() {
    let mut machine = playing_track(10.0);
    machine.apply(TaskbarInput::Position { secs: 5.0 }, at(1000));
    let mut sent = Vec::new();
    for step in 1..=20u64 {
        let secs = 5.0 - step as f64 * 0.1;
        if let Some(value) = machine.apply_update(&Update::Position(secs), at(1000 + step)) {
            sent.push(value);
        }
    }
    assert_eq!(sent, vec![shown(49)], "the first seek of a drag is sent, the rest are held");
    // Playback resumes from 3.0 s; the next tick, a gap later, sends the latest value.
    assert_eq!(machine.apply_update(&Update::Position(3.1), at(1300)), Some(shown(31)));
}

#[test]
fn a_drag_while_paused_sends_every_change_so_the_last_one_is_never_lost() {
    let mut machine = playing_track(10.0);
    machine.apply(TaskbarInput::Position { secs: 5.0 }, at(1000));
    machine.apply(TaskbarInput::Playing(false), at(1001));
    let mut sent = Vec::new();
    for secs in [4.0, 3.0, 2.0] {
        if let Some(value) = machine.apply_update(&Update::Position(secs), at(1002)) {
            sent.push(value);
        }
    }
    assert_eq!(sent, vec![shown(40), shown(30), shown(20)]);
    // A small forward step while paused is sent at once as well, because no tick would follow it.
    assert_eq!(machine.apply_update(&Update::Position(2.5), at(1003)), Some(shown(25)));
}

#[test]
fn a_seek_message_sent_through_update_moves_the_bar_immediately() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 30.0 }, at(1000));
    assert_eq!(machine.apply_update(&Update::Position(70.0), at(1002)), Some(shown(70)));
}

#[test]
fn pausing_a_track_that_has_moved_keeps_the_bar_at_its_position() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 40.0 }, at(1000));
    // Nothing changes on screen: the bar keeps its value and visibility.
    assert_eq!(machine.apply(TaskbarInput::Playing(false), at(1010)), None);
    assert_eq!(machine.apply(TaskbarInput::Playing(true), at(1020)), None);
}

#[test]
fn pausing_flushes_a_change_that_the_rate_limit_was_holding() {
    let mut machine = playing_track(10.0);
    machine.apply(TaskbarInput::Position { secs: 1.0 }, at(1000));
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 1.5 }, at(1050)), None);
    assert_eq!(machine.apply(TaskbarInput::Playing(false), at(1060)), Some(shown(15)));
}

#[test]
fn pausing_at_the_start_keeps_the_bar_shown_at_zero() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 0.0 }, at(10));
    assert_eq!(machine.apply(TaskbarInput::Playing(false), at(20)), None);
    assert_eq!(machine.apply(TaskbarInput::Playing(true), at(30)), None);
}

#[test]
fn a_track_that_was_never_played_shows_no_bar_when_the_engine_says_not_playing() {
    let mut machine = TaskbarProgress::new();
    machine.apply(TaskbarInput::NowPlaying { duration_secs: 100.0 }, at(0));
    assert_eq!(machine.apply(TaskbarInput::Playing(false), at(10)), None);
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 0.0 }, at(20)), None);
}

#[test]
fn a_restored_position_on_a_paused_track_shows_the_bar() {
    let mut machine = TaskbarProgress::new();
    machine.apply(TaskbarInput::NowPlaying { duration_secs: 100.0 }, at(0));
    machine.apply(TaskbarInput::Playing(false), at(1));
    assert_eq!(machine.apply_update(&Update::Position(40.0), at(10)), Some(shown(40)));
}

#[test]
fn guard_selecting_a_row_while_playing_sends_one_value_and_never_hides_the_bar() {
    // The engine's order for a row click: NowPlaying, Position(0), Playing(false), then the window's Play: Playing(true).
    // The old code hid the bar and showed it again with the old position (3.86 s of the old 120 s track) over the new 10 s length.
    let mut machine = playing_track(120.0);
    machine.apply(TaskbarInput::Position { secs: 3.86 }, at(1000));
    let mut sent = Vec::new();
    for update in [now_playing(10.0), Update::Position(0.0), Update::Playing(false), Update::Playing(true)] {
        if let Some(value) = machine.apply_update(&update, at(1100)) {
            sent.push(value);
        }
    }
    assert_eq!(sent, vec![shown(0)]);
}

#[test]
fn a_backlog_replayed_after_a_stall_sends_once_and_the_next_tick_sends_the_latest() {
    // A stalled window thread replays about 100 queued ticks within 20 ms; the old code sent 87 of them.
    let mut machine = playing_track(120.0);
    machine.apply(TaskbarInput::Position { secs: 10.0 }, at(1000));
    let mut sent = Vec::new();
    for tick in 1..=100u64 {
        let secs = 10.0 + tick as f64 * 0.3;
        if let Some(value) = machine.apply(TaskbarInput::Position { secs }, at(2000 + tick / 5)) {
            sent.push(value);
        }
    }
    assert_eq!(sent.len(), 1, "a replayed backlog must produce one send, got {sent:?}");
    // The next live tick, a gap later, sends the latest value (40.3 s of 120 s is 33 percent).
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 40.3 }, at(2300)), Some(shown(33)));
}

#[test]
fn a_stopped_track_at_its_end_hides_the_bar() {
    let mut machine = playing_track(10.0);
    machine.apply(TaskbarInput::Position { secs: 9.9 }, at(1000));
    // The end of the queue: the engine reports only "not playing" and leaves the last track loaded.
    assert_eq!(machine.apply(TaskbarInput::Playing(false), at(1010)), Some(TaskbarUpdate::HIDDEN));
}

#[test]
fn going_idle_hides_the_bar_and_forgets_the_position() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 40.0 }, at(1000));
    assert_eq!(machine.apply(TaskbarInput::Idle, at(1001)), Some(TaskbarUpdate::HIDDEN));
    // A late position with no track is ignored.
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 41.0 }, at(2000)), None);
    assert_eq!(machine.apply(TaskbarInput::Seek { secs: 41.0 }, at(2001)), None);
}

#[test]
fn a_track_without_a_usable_length_shows_no_bar() {
    for bad in [0.0, -3.0, f64::NAN, f64::INFINITY] {
        let mut machine = playing_track(100.0);
        machine.apply(TaskbarInput::Position { secs: 40.0 }, at(1000));
        assert_eq!(
            machine.apply(TaskbarInput::NowPlaying { duration_secs: bad }, at(1001)),
            Some(TaskbarUpdate::HIDDEN),
            "duration {bad}"
        );
        assert_eq!(machine.apply(TaskbarInput::Position { secs: 1.0 }, at(2000)), None);
    }
}

#[test]
fn not_a_number_positions_are_dropped() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 40.0 }, at(1000));
    assert_eq!(machine.apply(TaskbarInput::Position { secs: f64::NAN }, at(2000)), None);
    assert_eq!(machine.apply(TaskbarInput::Seek { secs: f64::NAN }, at(2001)), None);
    assert_eq!(machine.apply(TaskbarInput::Seek { secs: -5.0 }, at(2002)), None);
    assert_eq!(machine.apply(TaskbarInput::Position { secs: 41.0 }, at(3000)), Some(shown(41)));
}

#[test]
fn the_next_track_while_playing_keeps_the_bar_shown_and_restarts_it_at_zero() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 99.9 }, at(1000));
    assert_eq!(machine.apply_update(&now_playing(50.0), at(1100)), Some(shown(0)));
    assert_eq!(machine.apply_update(&Update::Position(0.0), at(1101)), None);
}

#[test]
fn engine_messages_without_a_track_are_idle() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 40.0 }, at(1000));
    let no_track = Update::NowPlaying {
        index: None,
        name: String::new(),
        duration: 0.0,
    };
    assert_eq!(machine.apply_update(&no_track, at(1001)), Some(TaskbarUpdate::HIDDEN));
}

#[test]
fn a_rescan_that_keeps_the_track_changes_nothing_and_one_that_loses_it_hides_the_bar() {
    let mut machine = playing_track(100.0);
    machine.apply(TaskbarInput::Position { secs: 40.0 }, at(1000));
    let same_track = Update::Reconciled {
        names: vec![String::from("a")],
        index: Some(0),
        name: String::from("a"),
        duration: 100.0,
    };
    assert_eq!(machine.apply_update(&same_track, at(1001)), None);
    assert_eq!(machine.apply_update(&Update::Position(40.2), at(1300)), None);
    let lost_track = Update::Reconciled {
        names: Vec::new(),
        index: None,
        name: String::new(),
        duration: 0.0,
    };
    assert_eq!(machine.apply_update(&lost_track, at(1400)), Some(TaskbarUpdate::HIDDEN));
}

#[test]
fn messages_that_do_not_concern_the_bar_are_ignored() {
    let mut machine = playing_track(100.0);
    assert_eq!(machine.apply_update(&Update::Volume(0.5), at(10)), None);
    assert_eq!(machine.apply_update(&Update::Analysis { done: 1, total: 9 }, at(11)), None);
}

#[test]
fn a_whole_session_never_moves_backward_except_through_a_seek_or_a_new_track() {
    // Ten minutes of ticks every 0.1 s on a 120 s track, then a new 95 s track: the sent percents only rise within a track.
    let mut machine = TaskbarProgress::new();
    let mut sent: Vec<(u64, TaskbarUpdate)> = Vec::new();
    let mut ms = 0u64;
    for update in [now_playing(120.0), Update::Position(0.0), Update::Playing(true)] {
        if let Some(value) = machine.apply_update(&update, at(ms)) {
            sent.push((ms, value));
        }
    }
    for tick in 1..=1200u64 {
        ms += 100;
        let update = Update::Position(tick as f64 / 10.0);
        if let Some(value) = machine.apply_update(&update, at(ms)) {
            sent.push((ms, value));
        }
    }
    let percents: Vec<u8> = sent.iter().map(|(_, value)| return value.percent).collect();
    assert!(percents.windows(2).all(|pair| return pair[0] <= pair[1]), "not monotonic: {percents:?}");
    assert_eq!(percents.last(), Some(&100));
    let gaps: Vec<u64> = sent[2..].windows(2).map(|pair| return pair[1].0 - pair[0].0).collect();
    assert!(gaps.iter().all(|gap| return *gap >= 250), "a send came sooner than the gap: {gaps:?}");
    assert!(sent.len() <= 102, "one send per percent at most, got {}", sent.len());
}
