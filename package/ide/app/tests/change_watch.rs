//! The inotify change watcher against disposable directories:
//!  what is reported,
//!  what is ignored,
//! and how lost or failed watches fall back to rereading everything shown.

/// What:
///  `#[path = "..."]` names the file a module is read from;
///  this crate's modules live under
///       `tests/change_watch/`.
/// Why:
///  One test crate links the library once and shares its waits without unused-code warnings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as support from './change_watch/support';
/// ```
#[path = "change_watch/support.rs"]
mod support;

/// Entry changes,
///  moves,
///  a new watch's extra read,
///  and watch removal on collapse.
#[path = "change_watch/entries.rs"]
mod entries;

/// Displayed-file classification and the watcher's own reads.
#[path = "change_watch/source.rs"]
mod source;

/// Queue overflow,
///  notification errors,
///  removed and renamed watched directories,
///  failed watches.
#[path = "change_watch/recovery.rs"]
mod recovery;

/// Containment at the project root,
///  and shutdown.
#[path = "change_watch/boundary.rs"]
mod boundary;
