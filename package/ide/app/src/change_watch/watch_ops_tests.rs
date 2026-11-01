//! Failure descriptions for watches that could not be added.

/// The function under test.
use super::describe;
/// What:
///  notify's error type and its kinds;
///  `MaxFilesWatch` is what `ENOSPC` from `inotify_add_watch` maps to.
/// Why:
///  The watch limit is reported without exhausting `fs.inotify.max_user_watches` on the test host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { NotifyError, ErrorKind } from 'notify';
/// ```
use notify::{Error, ErrorKind};
/// Borrowed native paths name the affected directory.
use std::path::Path;

/// The watch limit names the directory and the sysctl that sets the limit.
#[test]
fn watch_limit_names_the_directory_and_the_sysctl() {
    let message = describe(
        &Error::new(ErrorKind::MaxFilesWatch),
        Path::new("/project/src"),
    );
    assert!(
        message.contains("/project/src") && message.contains("fs.inotify.max_user_watches"),
        "the watch-limit message lost its directory or the sysctl that sets the limit: {message}"
    );
}

/// Other failures keep notify's own description beside the directory.
#[test]
fn other_failures_keep_notifys_description() {
    let message = describe(
        &Error::new(ErrorKind::PathNotFound),
        Path::new("/project/gone"),
    );
    assert!(message.contains("/project/gone"), "{message}");
    assert!(!message.contains("max_user_watches"), "{message}");
}
