//! When a picture is requested:
//!  the pure decision from two keys and the clock,
//!  and the two periods it uses.

/// The key a picture is valid for,
///  whose parts the decision compares.
use crate::native::minimap_worker::Key;
/// The quiet time and the throttle period are durations.
use std::time::Duration;

/// What:
///  Quiet time after the last change of what the picture is valid for before a new request is posted.
/// Why:
///  The measurement of the whole text changes the vertical mapping every few milliseconds while a file opens,
///  and a resize drag changes the size at every step;
/// one request after the changes stop is all the picture needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEBOUNCE_MS = 120;
/// ```
pub(in crate::native) const DEBOUNCE: Duration = Duration::from_millis(120);

/// What:
///  Longest time the picture may fall behind while only the vertical mapping changes.
/// Why:
///  The measurement of a long text changes the mapping for as long as it runs (a second or two for fifty thousand
///  lines);
/// a picture that waited for it to end would stay as it was drawn before the first measurement.
///  One request each period,
/// with the newest wins rule of the worker,
/// keeps the picture following it at a bounded cost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THROTTLE_MS = 250;
/// ```
pub(in crate::native) const THROTTLE: Duration = Duration::from_millis(250);

/// What:
///  Longest time the picture may fall behind while only the bands the track shows change.
/// Why:
///  A scroll through a long text names new bands every band's height or so;
/// the picture keeps one band on each side of the track,
/// so a request a few tens of milliseconds after the change
///  brings the next band in before the view reaches the edge of what the picture holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SLIDE_MS = 60;
/// ```
pub(in crate::native) const SLIDE: Duration = Duration::from_millis(60);

/// What:
///  What the window does about a key that is not the one last posted.
/// Why:
///  The decision is a pure function of the two keys and the clock,
/// so a test can hold it against every kind of change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plan = { kind: 'post' } | { kind: 'wait'; ms: number } | { kind: 'keep' };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::native) enum Plan {
    /// Post a request now.
    Post,
    /// Post after this long,
    /// unless a later change moves the time.
    Wait(
        /// How long to wait.
        Duration,
    ),
    /// A wait that is due soon enough is already running:
    /// do nothing.
    Keep,
}

/// What:
///  Decide when the picture for `key` is requested,
/// from what the newest request was for,
/// what the previous look at the
///  window was for,
/// how long ago the newest request was posted,
/// and whether a wait is already running.
///  Nothing posted yet,
/// another file,
/// a new palette,
/// or a new scale:
/// now.
///  Since the previous look only the vertical mapping changed:
/// at the throttle's pace,
/// and never while a wait is running.
///  Since the previous look only the bands the track shows changed (a scroll):
/// the same,
/// at the slide period.
///  Anything else,
/// such as a new size,
/// a new wrap width,
/// new text,
/// or new syntax classes:
/// after the quiet time.
/// Why:
///  The first picture and a picture of the wrong file or colors must not wait;
/// the measurement of a long text changes
///  the mapping for a second or two and the picture follows it steadily;
/// a resize drag changes the size at every step
///  and the mapping with it,
/// so the wait it starts is kept and one request after the drag ends is all the picture needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plan(posted: Key | undefined, noted: Key | undefined, key: Key, sincePost: number | undefined,
///   waiting: boolean): Plan;
/// ```
pub(in crate::native) fn plan(
    posted: Option<Key>,
    noted: Option<Key>,
    key: Key,
    since_post: Option<Duration>,
    waiting: bool,
) -> Plan {
    let Some(previous) = posted else {
        return Plan::Post;
    };
    if previous.stamp.file != key.stamp.file
        || previous.colors != key.colors
        || previous.scale != key.scale
    {
        return Plan::Post;
    }
    // Only the vertical mapping and the bands differ from the previous look: the measurement of the text is arriving,
    // or the view is scrolling.
    let (steady, scrolling) = match noted {
        Some(before) => (
            (Key {
                rows: key.rows,
                window: key.window,
                ..before
            }) == key,
            before.rows == key.rows,
        ),
        None => (false, false),
    };
    if !steady {
        return Plan::Wait(DEBOUNCE);
    }
    if waiting {
        return Plan::Keep;
    }
    let period = if scrolling { SLIDE } else { THROTTLE };
    let since = since_post.unwrap_or(period);
    if since >= period {
        return Plan::Post;
    }
    return Plan::Wait(period - since);
}
