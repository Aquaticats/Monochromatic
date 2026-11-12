//! What:
//!  The paths whose privacy a preparation proved to itself.
//! Why:
//!  A proof is one metadata round trip to the device,
//!  and this device stalls such calls for seconds,
//!  so a control must be able to say which paths one preparation proved
//!  and which it did not need to.
//!  A directory this process just made belongs to this account with mode `0700` less the umask,
//!  so proving it cannot change the answer;
//!  a directory it found already there must be proved before anything of this account is written inside it.
//!  Kept in a file of its own so the shipped `scratch_location.rs` holds no state a release would carry
//!  (`rust/tests-in-sidecars`);
//!  it is declared there under `#[cfg(test)]`,
//!  so the release executable never contains it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a module that exists only in test builds
//! export const proofs = { note, forget, proved };
//! ```
use std::cell::RefCell;
use std::path::{Path, PathBuf};

thread_local! {
    static PROVED: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
}

/// What:
///  Records that a preparation proved `path` is a private directory of this account.
/// Why:
///  The call site is the only moment the proof is made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proved.push(path);
/// ```
pub(crate) fn note(path: &Path) {
    PROVED.with(|proved| { proved.borrow_mut().push(path.to_path_buf()); });
}

/// What:
///  Forgets every proof this thread recorded.
/// Why:
///  A control reads the proofs of one call of its own,
///  so it starts from nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proved.length = 0;
/// ```
pub(crate) fn forget() {
    PROVED.with(|proved| { proved.borrow_mut().clear(); });
}

/// What:
///  Every path this thread proved private since it last forgot them.
/// Why:
///  A control reads them after the preparation returns,
///  and compares them with the directory the call did not make and the tree it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return [...proved];
/// ```
pub(crate) fn proved() -> Vec<PathBuf> {
    return PROVED.with(|proved| { return proved.borrow().clone(); });
}
