//! Tests for `analysis_notification_logic.rs`, pulled in by
//! `#[cfg(test)] #[path = "analysis_notification_logic_tests.rs"] mod tests;` at the bottom of that file.
//! Time is a parameter, so every rule is checked with plain `Instant` arithmetic and no sleeping.

use super::*;

/// What: `start` is the fixed origin every test measures from.
/// Why: `Instant` has no constructor from a number, so one captured instant plus offsets stands in for a clock.
fn start() -> Instant {
    return Instant::now();
}

/// What: `ms` is `origin` plus a number of milliseconds.
/// Why: it keeps the test bodies readable as a timeline.
fn ms(origin: Instant, millis: u64) -> Instant {
    return origin + Duration::from_millis(millis);
}

/// What: `view` builds a view for expected values.
/// Why: asserting whole views shows the percent, the text and the paused flag together.
fn view(percent: u32, text: &str, paused: bool) -> IndicatorView {
    return IndicatorView {
        percent,
        text: text.to_string(),
        paused,
    };
}

// What: `count_text` follows the app's grouping rule.
// Why: the text is the user's only count, and `1,218` must match the former banner.
#[test]
fn count_text_groups_thousands_like_the_former_banner() {
    assert_eq!(count_text(412, 1218), "412 of 1,218");
    assert_eq!(count_text(0, 5), "0 of 5");
    assert_eq!(count_text(1_000_000, 1_234_567), "1,000,000 of 1,234,567");
}

// What: a count beyond the signed range is clamped, not wrapped.
// Why: wrapping would show a negative count.
#[test]
fn count_text_clamps_a_count_above_the_signed_range() {
    assert_eq!(count_text(usize::MAX, usize::MAX), "2,147,483,647 of 2,147,483,647");
}

// What: percent rounds down and stays below 100 while work remains.
// Why: a bar at 100 would look finished while tracks remain.
#[test]
fn percent_rounds_down_and_never_reaches_100_while_running() {
    assert_eq!(percent_for(0, 1218), 0);
    assert_eq!(percent_for(412, 1218), 33);
    assert_eq!(percent_for(1217, 1218), 99);
    assert_eq!(percent_for(5000, 1218), 99);
    assert_eq!(percent_for(1, 0), 0);
    assert_eq!(percent_for(usize::MAX, usize::MAX), 99);
}

// What: the first unfinished report opens the indicator with its view.
// Why: D107 shows the indicator when analysis starts.
#[test]
fn the_first_unfinished_report_opens_the_indicator() {
    let origin = start();
    let mut machine = Machine::new(true);
    assert_eq!(
        machine.progress(origin, 0, 1218),
        vec![Effect::Open(view(0, "0 of 1,218", false))]
    );
}

// What: reports with an unknown total or from a finished sweep open nothing.
// Why: a fully cached library finishes at once and must not flash an indicator.
#[test]
fn an_unknown_total_or_a_finished_sweep_never_opens() {
    let origin = start();
    let mut machine = Machine::new(true);
    assert_eq!(machine.progress(origin, 0, 0), Vec::new());
    assert_eq!(machine.progress(origin, 7, 7), Vec::new());
    assert_eq!(machine.progress(origin, 9, 7), Vec::new());
}

// What: a report inside the interval is held and the held view is due at the interval's end.
// Why: at most two updates a second; the newest count must still arrive.
#[test]
fn an_update_inside_the_interval_is_held_until_the_deadline() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 0, 1000);
    assert_eq!(machine.progress(ms(origin, 100), 100, 1000), Vec::new());
    assert_eq!(machine.deadline(), Some(ms(origin, 500)));
    assert_eq!(machine.tick(ms(origin, 499)), Vec::new());
    assert_eq!(
        machine.tick(ms(origin, 500)),
        vec![Effect::Update(view(10, "100 of 1,000", false))]
    );
    assert_eq!(machine.deadline(), None);
}

// What: while a view is held, a newer report replaces it.
// Why: only the latest count matters, so held views never queue up.
#[test]
fn a_newer_held_view_replaces_the_older_one() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 0, 1000);
    machine.progress(ms(origin, 100), 100, 1000);
    machine.progress(ms(origin, 200), 200, 1000);
    assert_eq!(
        machine.tick(ms(origin, 500)),
        vec![Effect::Update(view(20, "200 of 1,000", false))]
    );
}

// What: a report after the interval is sent at once.
// Why: the limit is a gap between sends, not a fixed grid.
#[test]
fn a_report_after_the_interval_is_sent_at_once() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 0, 1000);
    assert_eq!(
        machine.progress(ms(origin, 500), 300, 1000),
        vec![Effect::Update(view(30, "300 of 1,000", false))]
    );
}

// What: sixty reports in three seconds produce at most six sends.
// Why: this is the two-a-second limit stated as a count over a long run.
#[test]
fn a_busy_sweep_is_limited_to_two_sends_a_second() {
    let origin = start();
    let mut machine = Machine::new(true);
    let mut sends = 0;
    let mut previous: Option<Instant> = None;
    for step in 0..60_u64 {
        let now = ms(origin, step * 50);
        let mut effects = machine.progress(now, step as usize, 1000);
        effects.extend(machine.tick(now));
        for effect in effects {
            if matches!(effect, Effect::Open(_) | Effect::Update(_)) {
                sends += 1;
                if let Some(earlier) = previous {
                    assert!(now - earlier >= MIN_UPDATE_INTERVAL, "two sends closer than the interval");
                }
                previous = Some(now);
            }
        }
    }
    assert!(sends <= 6, "sent {sends} times in three seconds");
    assert!(sends >= 5, "sent only {sends} times in three seconds");
}

// What: a report that repeats the last sent view sends nothing and drops a held view.
// Why: the percent and the text are unchanged, so the bus has nothing to learn.
#[test]
fn a_repeated_view_is_skipped() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 0, 1000);
    assert_eq!(machine.progress(ms(origin, 600), 0, 1000), Vec::new());
    machine.progress(ms(origin, 700), 5, 1000);
    machine.progress(ms(origin, 800), 6, 1000);
    assert!(machine.deadline().is_some());
    assert_eq!(machine.progress(ms(origin, 900), 5, 1000), Vec::new());
    assert_eq!(machine.deadline(), None);
}

// What: a count change that leaves the percent alone still sends once the interval allows.
// Why: the count text is part of the view, so 412 to 413 of 1,218 is a change.
#[test]
fn a_new_count_with_the_same_percent_is_sent() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 412, 1218);
    assert_eq!(
        machine.progress(ms(origin, 500), 413, 1218),
        vec![Effect::Update(view(33, "413 of 1,218", false))]
    );
}

// What: the finishing report sends the full bar inside the interval and then the close.
// Why: D107 requires the final value and the close, and neither may be rate limited.
#[test]
fn the_final_view_and_the_close_are_never_held_back() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 0, 1218);
    machine.progress(ms(origin, 10), 1000, 1218);
    assert_eq!(
        machine.progress(ms(origin, 20), 1218, 1218),
        vec![
            Effect::Update(view(100, "1,218 of 1,218", false)),
            Effect::Close(Outcome::Finished),
        ]
    );
    assert_eq!(machine.deadline(), None);
}

// What: the finishing report sends the full bar even for a tiny sweep.
// Why: running views stop at 99, so the final view always differs from the last running one.
#[test]
fn the_finishing_report_always_sends_a_full_bar() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 0, 4);
    assert_eq!(
        machine.progress(ms(origin, 1), 4, 4),
        vec![
            Effect::Update(view(100, "4 of 4", false)),
            Effect::Close(Outcome::Finished),
        ]
    );
}

// What: a second sweep after a finished one opens a fresh indicator.
// Why: a queue reload starts a new sweep, and its progress must be shown again.
#[test]
fn a_new_sweep_after_a_finished_one_opens_again() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 0, 3);
    machine.progress(ms(origin, 10), 3, 3);
    assert_eq!(
        machine.progress(ms(origin, 20), 0, 8),
        vec![Effect::Open(view(0, "0 of 8", false))]
    );
}

// What: a pause request tells the engine and flips the indicator at once, even inside the interval.
// Why: the button only changes when the app reports the new state, and a click must feel immediate.
#[test]
fn pause_tells_the_engine_and_updates_the_indicator_at_once() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 100, 1000);
    assert_eq!(
        machine.set_paused(ms(origin, 10), true),
        vec![Effect::SetEnginePause(true), Effect::Update(view(10, "100 of 1,000", true))]
    );
    assert_eq!(
        machine.set_paused(ms(origin, 20), false),
        vec![Effect::SetEnginePause(false), Effect::Update(view(10, "100 of 1,000", false))]
    );
}

// What: a repeated pause request does nothing.
// Why: the signal and the click can both arrive; the engine must be told once.
#[test]
fn a_repeated_pause_request_is_ignored() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 100, 1000);
    machine.set_paused(origin, true);
    assert_eq!(machine.set_paused(ms(origin, 5), true), Vec::new());
}

// What: a pause change sends the held count, not an older one.
// Why: the pause update must not move the bar backwards.
#[test]
fn pause_sends_the_newest_known_count() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 100, 1000);
    machine.progress(ms(origin, 100), 250, 1000);
    assert_eq!(
        machine.set_paused(ms(origin, 150), true),
        vec![Effect::SetEnginePause(true), Effect::Update(view(25, "250 of 1,000", true))]
    );
    assert_eq!(machine.deadline(), None);
}

// What: while paused, later views carry the paused flag.
// Why: a stray late report after the pause must not flip the button back.
#[test]
fn views_after_a_pause_stay_paused() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 100, 1000);
    machine.set_paused(origin, true);
    assert_eq!(
        machine.progress(ms(origin, 600), 101, 1000),
        vec![Effect::Update(view(10, "101 of 1,000", true))]
    );
}

// What: a machine without a pause to offer ignores pause requests.
// Why: while the engine has no pause, advertising one would be a button that does nothing.
#[test]
fn a_machine_without_pause_ignores_pause_requests() {
    let origin = start();
    let mut machine = Machine::new(false);
    machine.progress(origin, 100, 1000);
    assert_eq!(machine.set_paused(origin, true), Vec::new());
}

// What: a pause request before the open or after the close is ignored.
// Why: a click on a stale indicator must not pause an analysis nobody can resume.
#[test]
fn a_pause_request_outside_an_open_indicator_is_ignored() {
    let origin = start();
    let mut machine = Machine::new(true);
    assert_eq!(machine.set_paused(origin, true), Vec::new());
    machine.progress(origin, 0, 10);
    machine.progress(ms(origin, 5), 10, 10);
    assert_eq!(machine.set_paused(ms(origin, 6), true), Vec::new());
}

// What: finishing while paused resumes the engine after the close.
// Why: a pause must never outlive the indicator that could release it.
#[test]
fn finishing_while_paused_releases_the_pause() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 1, 10);
    machine.set_paused(origin, true);
    let effects = machine.progress(ms(origin, 5), 10, 10);
    assert_eq!(effects.last(), Some(&Effect::SetEnginePause(false)));
    assert!(effects.contains(&Effect::Close(Outcome::Finished)));
}

// What: a dismissal ends the indicator, releases a pause, and no later report reopens it in the same sweep.
// Why: the user removed it; resurrecting it would be a nag, and a stuck pause would stall the analysis.
#[test]
fn a_dismissal_is_respected_until_the_sweep_ends() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 1, 10);
    machine.set_paused(origin, true);
    assert_eq!(machine.dismissed(), vec![Effect::SetEnginePause(false)]);
    assert_eq!(machine.progress(ms(origin, 700), 2, 10), Vec::new());
    assert_eq!(machine.progress(ms(origin, 800), 10, 10), Vec::new());
    assert_eq!(
        machine.progress(ms(origin, 900), 0, 20),
        vec![Effect::Open(view(0, "0 of 20", false))]
    );
}

// What: a dismissal with no open indicator does nothing.
// Why: the close signal also follows the app's own close, which must not be read as a user action.
#[test]
fn a_dismissal_without_an_open_indicator_is_ignored() {
    let mut machine = Machine::new(true);
    assert_eq!(machine.dismissed(), Vec::new());
}

// What: an engine failure closes with the message and releases a pause.
// Why: the user is told why the analysis stopped.
#[test]
fn a_failure_closes_with_its_message() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 1, 10);
    assert_eq!(
        machine.failed("disk unreadable".to_string()),
        vec![Effect::Close(Outcome::Failed("disk unreadable".to_string()))]
    );
    assert_eq!(machine.failed("again".to_string()), Vec::new());
}

// What: stopping closes an open indicator without a result and does nothing otherwise.
// Why: shutdown must not leave a job or notification behind, and must not invent one.
#[test]
fn stop_closes_an_open_indicator_only() {
    let origin = start();
    let mut machine = Machine::new(true);
    assert_eq!(machine.stop(), Vec::new());
    machine.progress(origin, 1, 10);
    assert_eq!(machine.stop(), vec![Effect::Close(Outcome::Stopped)]);
    assert_eq!(machine.stop(), Vec::new());
}

// What: after the bus fails the machine is disabled for good and releases a pause.
// Why: no retry storm, and no stuck pause.
#[test]
fn a_bus_failure_disables_the_indicator_for_the_rest_of_the_run() {
    let origin = start();
    let mut machine = Machine::new(true);
    machine.progress(origin, 1, 10);
    machine.set_paused(origin, true);
    assert_eq!(machine.bus_failed(), vec![Effect::SetEnginePause(false)]);
    assert!(machine.is_disabled());
    assert_eq!(machine.progress(ms(origin, 900), 2, 10), Vec::new());
    assert_eq!(machine.progress(ms(origin, 1900), 0, 99), Vec::new());
    assert_eq!(machine.bus_failed(), Vec::new());
    assert_eq!(machine.deadline(), None);
}

// What: a tick with nothing held, or before the interval, sends nothing.
// Why: the bus thread may wake early or for another input.
#[test]
fn a_tick_with_nothing_due_sends_nothing() {
    let origin = start();
    let mut machine = Machine::new(true);
    assert_eq!(machine.tick(origin), Vec::new());
    machine.progress(origin, 0, 10);
    assert_eq!(machine.tick(ms(origin, 900)), Vec::new());
}
