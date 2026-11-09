//! What:
//!  The names a control saw this module hand to a removal.
//! Why:
//!  A removal runs on a thread of its own and keeps the pathname it was given,
//!  so what it was given cannot be asked of it afterwards without racing it.
//!  The name is recorded where the removal is handed over,
//!  and a control compares it against the name the attempt that follows makes:
//!  a removal that outlives its ceiling must never be able to name
//!  the tree a new attempt writes.
//!  Kept in a file of its own so the shipped `scratch_location.rs` holds no state a release would carry
//!  (`rust/tests-in-sidecars`);
//!  it is declared there under `#[cfg(test)]`,
//!  so the release executable never contains it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a module that exists only in test builds
//! export const removals = { note, forget, named };
//! ```
use std::cell::RefCell;
use std::path::{Path, PathBuf};

thread_local! {
    static NAMED: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
}

/// What:
///  Records that a removal started here was given `path`.
/// Why:
///  The call site is the only moment the name is known to this thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// named.push(path);
/// ```
pub(crate) fn note(path: &Path) {
    NAMED.with(|named| { named.borrow_mut().push(path.to_path_buf()); });
}

/// What:
///  Every name this thread handed to a removal since it last forgot them.
/// Why:
///  A control reads them after the preparation returns,
///  when the removals themselves may already be gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return [...named];
/// ```
pub(crate) fn named() -> Vec<PathBuf> {
    return NAMED.with(|named| { return named.borrow().clone(); });
}

/// What:
///  Forgets the names recorded so far.
/// Why:
///  Each control starts from an empty record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// named.length = 0;
/// ```
pub(crate) fn forget() {
    NAMED.with(|named| { named.borrow_mut().clear(); });
}
