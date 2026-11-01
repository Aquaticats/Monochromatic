//! Tests for the message builders in `analysis_notification_wire.rs`, pulled in by
//! `#[cfg(test)] #[path = "analysis_notification_wire_tests.rs"] mod tests;` at the bottom of that file.
//! The calls themselves are covered by the end-to-end run against a mock bus; these pin the payloads.

use super::*;

/// What: `identity` is the app identity every test uses.
/// Why: the values match the shipped desktop file, so a failing assertion reads like the real payload.
fn identity() -> Identity {
    return Identity {
        desktop_entry: "monochromatic.music-player".to_string(),
        app_name: "Music Player".to_string(),
        app_icon: "multimedia-player".to_string(),
    };
}

/// What: `view` builds a view.
/// Why: the builders take a whole view.
fn view(percent: u32, text: &str, paused: bool) -> IndicatorView {
    return IndicatorView {
        percent,
        text: text.to_string(),
        paused,
    };
}

// What: the job request hints carry the header name and icon and the transient flag.
// Why: without them a dev run with no installed desktop file has a blank header, and without `transient` a finished
// job stays in the history.
#[test]
fn job_request_hints_name_the_app_and_make_the_job_transient() {
    let who = identity();
    let hints = job_request_hints(&who);
    assert_eq!(hints.len(), 3);
    assert_eq!(hints.get("application-display-name"), Some(&Value::from("Music Player")));
    assert_eq!(hints.get("application-icon-name"), Some(&Value::from("multimedia-player")));
    assert_eq!(hints.get("transient"), Some(&Value::from(true)));
}

// What: the first update carries the title; later ones do not.
// Why: the title never changes, so it is sent once.
#[test]
fn only_the_first_job_update_carries_the_title() {
    let shown = view(33, "412 of 1,218", false);
    let first = job_update_properties(&shown, true);
    assert_eq!(first.get("title"), Some(&Value::from("Analyzing true peak")));
    let later = job_update_properties(&shown, false);
    assert!(!later.contains_key("title"));
}

// What: a job update reports the count, an unsigned percent and the suspended flag.
// Why: Plasma reads `percent` as an unsigned integer and flips its button only from `suspended`.
#[test]
fn job_updates_carry_count_percent_and_suspended() {
    let shown = view(33, "412 of 1,218", true);
    let paused = job_update_properties(&shown, false);
    assert_eq!(paused.get("infoMessage"), Some(&Value::from("412 of 1,218")));
    assert_eq!(paused.get("percent"), Some(&Value::U32(33)));
    assert_eq!(paused.get("suspended"), Some(&Value::Bool(true)));
    assert_eq!(paused.len(), 3);
}

// What: the requested capability is exactly Suspendable.
// Why: Killable would add a Cancel button, and D84 makes analysis non-optional.
#[test]
fn the_job_capability_is_suspendable_and_never_killable() {
    assert_eq!(CAPABILITY_SUSPENDABLE, 2);
    assert_eq!(CAPABILITY_SUSPENDABLE & 1, 0);
}

// What: each outcome maps to its terminate code and message.
// Why: 0 removes a transient job quietly, 1 is a stop, and 2 shows the message.
#[test]
fn outcomes_map_to_terminate_codes() {
    assert_eq!(terminate_arguments(&Outcome::Finished), (0, String::new()));
    assert_eq!(terminate_arguments(&Outcome::Stopped), (1, String::new()));
    assert_eq!(
        terminate_arguments(&Outcome::Failed("disk unreadable".to_string())),
        (2, "disk unreadable".to_string())
    );
}

// What: the notification hints are exactly the decided set with the decided types.
// Why: the byte urgency and the 32-bit value are silent failures if sent as other integer types.
#[test]
fn notification_hints_are_the_decided_set() {
    let who = identity();
    let hints = notification_hints(&who, &view(33, "412 of 1,218", false));
    assert_eq!(hints.len(), 6);
    assert_eq!(hints.get("value"), Some(&Value::I32(33)));
    assert_eq!(hints.get("urgency"), Some(&Value::U8(0)));
    assert_eq!(hints.get("suppress-sound"), Some(&Value::Bool(true)));
    assert_eq!(hints.get("category"), Some(&Value::from("transfer")));
    assert_eq!(hints.get("desktop-entry"), Some(&Value::from("monochromatic.music-player")));
    assert_eq!(hints.get("resident"), Some(&Value::Bool(true)));
    assert!(!hints.contains_key("transient"));
}

// What: the action pair is Pause when running, Resume when paused, and absent when not suspendable.
// Why: a button that does nothing must not be offered, and the label must say what a press does.
#[test]
fn notification_actions_follow_the_pause_state() {
    assert_eq!(notification_actions(true, &view(1, "t", false)), vec!["pause", "Pause"]);
    assert_eq!(notification_actions(true, &view(1, "t", true)), vec!["resume", "Resume"]);
    assert!(notification_actions(false, &view(1, "t", false)).is_empty());
}

// What: the title is the decided text.
// Why: D107 names it, with the American spelling.
#[test]
fn the_title_is_the_decided_text() {
    assert_eq!(TITLE, "Analyzing true peak");
}
