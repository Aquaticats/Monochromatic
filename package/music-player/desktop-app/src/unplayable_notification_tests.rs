//! Tests for the notifier thread in `unplayable_notification.rs`, pulled in by
//! `#[cfg(test)] #[path = "unplayable_notification_tests.rs"] mod tests;` at the bottom of that file.
//! They run against a private bus with a recording notification service; the user's session bus is never used.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use super::*;
use crate::unplayable_notification_bus_support::{Behavior, PrivateBus};

fn identity() -> Identity {
    return Identity {
        desktop_entry: "monochromatic.music-player".to_string(),
        app_name: "Music Player".to_string(),
        app_icon: "multimedia-player".to_string(),
    };
}

fn config_for(bus: &PrivateBus, timeout: Duration) -> UnplayableConfig {
    let address = bus.address.clone();
    return UnplayableConfig {
        identity: identity(),
        connect: Box::new(move || {
            return zbus::blocking::connection::Builder::address(address.as_str())?.method_timeout(timeout).build();
        }),
    };
}

fn files(names: &[&str]) -> Vec<PathBuf> {
    return names.iter().map(|name| return PathBuf::from("/invented/library").join(name)).collect();
}

#[test]
fn a_report_becomes_exactly_one_notification_and_a_repeat_adds_none() {
    let bus = PrivateBus::start("thread_once");
    let (_service, calls) = bus.serve_notifications(Duration::ZERO);
    let notifier = UnplayableNotifier::spawn(config_for(&bus, Duration::from_secs(3)));
    notifier.report(&files(&["Example Track.flac"]));
    notifier.report(&files(&["Example Track.flac"]));
    notifier.shutdown();
    let seen = calls.lock().expect("not poisoned").clone();
    assert_eq!(seen.len(), 1, "the repeat of the same file posts nothing: {seen:?}");
    assert_eq!(seen[0].summary, "Cannot play a file");
    assert_eq!(seen[0].body, "Example Track.flac can't be played.");
}

#[test]
fn several_files_in_one_report_are_one_notification_with_a_count() {
    let bus = PrivateBus::start("thread_batch");
    let (_service, calls) = bus.serve_notifications(Duration::ZERO);
    let notifier = UnplayableNotifier::spawn(config_for(&bus, Duration::from_secs(3)));
    notifier.report(&files(&["a.wav", "b.wav", "c.wav"]));
    notifier.shutdown();
    let seen = calls.lock().expect("not poisoned").clone();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].body, "3 files can't be played, including a.wav.");
}

#[test]
fn an_empty_report_posts_nothing_and_does_not_even_connect() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&attempts);
    let notifier = UnplayableNotifier::spawn(UnplayableConfig {
        identity: identity(),
        connect: Box::new(move || {
            counted.fetch_add(1, Ordering::SeqCst);
            return Err(zbus::Error::Failure("no bus in this test".to_string()));
        }),
    });
    notifier.report(&[]);
    notifier.shutdown();
    assert_eq!(attempts.load(Ordering::SeqCst), 0, "nothing to say, so the bus is never opened");
}

#[test]
fn a_new_file_after_an_earlier_report_posts_a_second_notification_for_that_file_only() {
    let bus = PrivateBus::start("thread_two");
    let (_service, calls) = bus.serve_notifications(Duration::ZERO);
    let notifier = UnplayableNotifier::spawn(config_for(&bus, Duration::from_secs(3)));
    notifier.report(&files(&["a.wav"]));
    notifier.report(&files(&["a.wav", "b.wav"]));
    notifier.shutdown();
    let bodies: Vec<String> = calls.lock().expect("not poisoned").iter().map(|call| return call.body.clone()).collect();
    assert_eq!(bodies, vec!["a.wav can't be played.".to_string(), "b.wav can't be played.".to_string()]);
}

#[test]
fn a_refused_post_is_not_retried_and_a_later_report_still_reaches_the_service() {
    let bus = PrivateBus::start("thread_refused");
    let (_service, calls) = bus.serve_notifications_with(Behavior { delay: Duration::ZERO, refuse_first: 1 });
    let notifier = UnplayableNotifier::spawn(config_for(&bus, Duration::from_secs(3)));
    notifier.report(&files(&["early.wav"]));
    notifier.report(&files(&["late.wav"]));
    notifier.shutdown();
    let bodies: Vec<String> = calls.lock().expect("not poisoned").iter().map(|call| return call.body.clone()).collect();
    assert_eq!(bodies, vec!["late.wav can't be played.".to_string()], "the refused early file is not told again");
}

#[test]
fn a_bus_with_no_notification_service_ends_quietly_and_shows_nothing_to_wait_for() {
    let bus = PrivateBus::start("thread_none");
    let notifier = UnplayableNotifier::spawn(config_for(&bus, Duration::from_secs(3)));
    let started = Instant::now();
    notifier.report(&files(&["a.wav"]));
    notifier.report(&files(&["b.wav", "c.wav"]));
    notifier.shutdown();
    assert!(started.elapsed() < Duration::from_secs(2), "a missing service answers at once with an error, so nothing waits");
}

#[test]
fn a_connection_that_cannot_be_made_never_panics_and_report_returns_at_once() {
    let notifier = UnplayableNotifier::spawn(UnplayableConfig {
        identity: identity(),
        connect: Box::new(|| return Err(zbus::Error::Failure("no bus in this test".to_string()))),
    });
    let started = Instant::now();
    notifier.report(&files(&["a.wav"]));
    notifier.report(&files(&["b.wav"]));
    assert!(started.elapsed() < Duration::from_millis(500), "report only posts a message");
    notifier.shutdown();
}

#[test]
fn a_service_that_never_answers_does_not_block_the_caller_and_shutdown_waits_only_the_method_timeout() {
    let bus = PrivateBus::start("thread_wedged");
    let (_service, _calls) = bus.serve_notifications(Duration::from_secs(5));
    let notifier = UnplayableNotifier::spawn(config_for(&bus, Duration::from_millis(300)));
    let started = Instant::now();
    notifier.report(&files(&["a.wav"]));
    assert!(started.elapsed() < Duration::from_millis(200), "report returns before the service answers");
    notifier.shutdown();
    assert!(started.elapsed() < Duration::from_secs(3), "shutdown ends after the bounded call, not the service delay");
}

#[test]
fn reporting_after_shutdown_is_harmless() {
    let notifier = UnplayableNotifier::spawn(UnplayableConfig {
        identity: identity(),
        connect: Box::new(|| return Err(zbus::Error::Failure("no bus in this test".to_string()))),
    });
    notifier.shutdown();
    notifier.report(&files(&["a.wav"]));
    notifier.shutdown();
}

#[test]
fn dropping_every_handle_without_shutdown_still_delivers_what_was_posted_and_ends_the_thread() {
    let bus = PrivateBus::start("thread_drop");
    let (_service, calls) = bus.serve_notifications(Duration::ZERO);
    let notifier = UnplayableNotifier::spawn(config_for(&bus, Duration::from_secs(3)));
    notifier.report(&files(&["a.wav"]));
    drop(notifier);
    let end = Instant::now() + Duration::from_secs(5);
    while calls.lock().expect("not poisoned").is_empty() && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(calls.lock().expect("not poisoned").len(), 1, "the report posted before the drop is still delivered");
}

fn worker_with(connect: Box<dyn Fn() -> zbus::Result<Connection> + Send + 'static>) -> Worker {
    return Worker { identity: identity(), connect, connection: None, ledger: Ledger::new(), warned: false };
}

#[test]
fn the_first_failed_post_sets_the_warned_flag_and_a_report_with_nothing_new_does_not() {
    let mut worker = worker_with(Box::new(|| return Err(zbus::Error::Failure("no bus in this test".to_string()))));
    worker.handle(&[]);
    assert!(!worker.warned, "an empty report posts nothing, so nothing fails");
    worker.handle(&files(&["a.wav"]));
    assert!(worker.warned, "the first failure is logged as the one warning");
    worker.handle(&files(&["a.wav"]));
    assert!(worker.warned, "the flag stays set");
}

#[test]
fn a_successful_post_leaves_the_warned_flag_clear() {
    let bus = PrivateBus::start("thread_flag");
    let (_service, _calls) = bus.serve_notifications(Duration::ZERO);
    let address = bus.address.clone();
    let mut worker = worker_with(Box::new(move || {
        return zbus::blocking::connection::Builder::address(address.as_str())?.method_timeout(Duration::from_secs(3)).build();
    }));
    worker.handle(&files(&["a.wav"]));
    assert!(!worker.warned);
    assert!(worker.connection.is_some(), "the connection is kept for the next report");
}
