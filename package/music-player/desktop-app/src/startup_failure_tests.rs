// What:     Unit tests for `startup_failure.rs`, pulled in by `#[cfg(test)] #[path = "startup_failure_tests.rs"] mod tests;`.
// Why:      The message box exists on Windows only; its text, the box decision and the test hook are checked here on
//           every host.

use super::*;

const EXECUTABLE: &str = r"C:\Users\someone\Downloads\music-player-windows-x64.exe";
const ALL_STEPS: [StartupStep; 4] = [StartupStep::WindowSystem, StartupStep::Window, StartupStep::EventLoop, StartupStep::Panic];

#[test]
fn each_forced_value_names_its_step() {
    assert_eq!(forced_startup_failure(Some("window-system")), Some(StartupStep::WindowSystem));
    assert_eq!(forced_startup_failure(Some("window")), Some(StartupStep::Window));
    assert_eq!(forced_startup_failure(Some("event-loop")), Some(StartupStep::EventLoop));
    assert_eq!(forced_startup_failure(Some("panic")), Some(StartupStep::Panic));
}

#[test]
fn an_unset_variable_forces_nothing() {
    assert_eq!(forced_startup_failure(None), None);
}

#[test]
fn an_unknown_value_forces_nothing() {
    assert_eq!(forced_startup_failure(Some("Window")), None);
    assert_eq!(forced_startup_failure(Some("")), None);
}

#[test]
fn the_forced_detail_round_trips_through_the_variable_value() {
    for step in ALL_STEPS {
        let detail = forced_failure_detail(step);
        let value = detail
            .strip_prefix(&format!("forced by {FORCE_STARTUP_FAILURE_ENV}="))
            .and_then(|rest| return rest.split(':').next());
        assert_eq!(forced_startup_failure(value), Some(step), "{detail}");
    }
}

#[test]
fn failures_before_the_window_exists_are_titled_could_not_start() {
    assert_eq!(failure_title(StartupStep::WindowSystem), "Music Player could not start");
    assert_eq!(failure_title(StartupStep::Window), "Music Player could not start");
}

#[test]
fn failures_of_a_started_program_are_titled_stopped() {
    assert_eq!(failure_title(StartupStep::EventLoop), "Music Player stopped");
    assert_eq!(failure_title(StartupStep::Panic), "Music Player stopped");
}

#[test]
fn every_message_names_the_failure_carries_the_detail_and_the_program_path() {
    for step in ALL_STEPS {
        let message = failure_message(step, "the detail Windows gave", EXECUTABLE);
        assert!(message.starts_with("Music Player"), "{message}");
        assert!(message.contains("the detail Windows gave"), "{message}");
        assert!(message.contains(&format!("\"{EXECUTABLE}\"")), "the Command Prompt line quotes the path: {message}");
        assert!(message.contains("Start Music Player again"), "{message}");
    }
}

#[test]
fn the_window_system_message_names_the_desktop_connection_and_the_signed_in_start() {
    let message = failure_message(StartupStep::WindowSystem, "detail", EXECUTABLE);
    assert!(message.contains("connection to the Windows desktop"), "{message}");
    assert!(message.contains("scheduled task"), "{message}");
    assert!(message.contains("Restart Windows"), "{message}");
}

#[test]
fn the_window_message_names_the_window_and_the_graphics_driver() {
    let message = failure_message(StartupStep::Window, "detail", EXECUTABLE);
    assert!(message.contains("could not create its window"), "{message}");
    assert!(message.contains("graphics driver"), "{message}");
}

#[test]
fn only_the_event_loop_message_offers_the_software_renderer() {
    let message = failure_message(StartupStep::EventLoop, "detail", EXECUTABLE);
    assert!(message.contains("window stopped working"), "{message}");
    assert!(message.contains("set SLINT_BACKEND=software"), "{message}");
    for step in [StartupStep::WindowSystem, StartupStep::Window, StartupStep::Panic] {
        assert!(!failure_message(step, "detail", EXECUTABLE).contains("SLINT_BACKEND"), "{step:?}");
    }
}

#[test]
fn the_panic_message_calls_it_a_defect_of_the_program() {
    let message = failure_message(StartupStep::Panic, "detail", EXECUTABLE);
    assert!(message.contains("internal error"), "{message}");
    assert!(message.contains("not a problem with your computer"), "{message}");
}

#[test]
fn no_message_contains_a_dash_used_as_punctuation() {
    for step in ALL_STEPS {
        let message = failure_message(step, "detail", EXECUTABLE);
        assert!(!message.contains('\u{2014}') && !message.contains('\u{2013}') && !message.contains(" - "), "{message}");
    }
}

#[test]
fn the_box_is_shown_when_stderr_has_no_handle() {
    assert!(message_box_wanted(None));
}

#[test]
fn the_box_is_not_shown_when_stderr_has_a_handle() {
    assert!(!message_box_wanted(Some(0x1c4)));
}

#[test]
fn a_nul_inside_the_text_is_replaced() {
    assert_eq!(without_nul("before\0after"), "before\u{FFFD}after");
}

#[test]
fn text_without_nul_is_unchanged() {
    assert_eq!(without_nul("plain text\nsecond line"), "plain text\nsecond line");
}

#[test]
fn the_panic_detail_has_the_message_and_the_location() {
    assert_eq!(panic_detail(Some("index out of range"), Some("src/main.rs:10:5")), "index out of range (at src/main.rs:10:5)");
}

#[test]
fn the_panic_detail_without_a_location_is_the_message() {
    assert_eq!(panic_detail(Some("index out of range"), None), "index out of range");
}

#[test]
fn a_panic_without_text_says_so() {
    assert_eq!(panic_detail(None, None), "the panic carried no text");
}
