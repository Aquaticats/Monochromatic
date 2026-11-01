//! Unit tests for the option value.

/// Brings the mode into scope.
use super::NotificationMode;

/// The three spellings parse and anything else is refused with the choices named.
#[test]
fn parses_exactly_the_three_spellings() {
    assert_eq!(NotificationMode::parse("none").expect("none is a valid notification mode"), NotificationMode::None);
    assert_eq!(NotificationMode::parse("freedesktop").expect("freedesktop is a valid notification mode"), NotificationMode::Freedesktop);
    assert_eq!(NotificationMode::parse("kde").expect("kde is a valid notification mode"), NotificationMode::Kde);
    for bad in ["", "KDE", "plasma", "freedesktop,kde", "none "] {
        let message = NotificationMode::parse(bad).unwrap_err().to_string();
        assert!(message.contains("none, freedesktop, or kde"), "{bad:?}: {message}");
    }
}

/// Each mode serves what the README says.
#[test]
fn modes_serve_the_documented_services() {
    assert!(!NotificationMode::None.serves_notifications());
    assert!(NotificationMode::Freedesktop.serves_notifications());
    assert!(!NotificationMode::Freedesktop.serves_jobs());
    assert!(NotificationMode::Kde.serves_notifications() && NotificationMode::Kde.serves_jobs());
    assert_eq!(NotificationMode::default(), NotificationMode::None);
}
