//! What:
//!  Disposable filesystem fixtures owned by one test.
//! Why:
//!  Native I/O verification must not modify a user's files or reuse another test's directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // A temporary directory disposed at the end of each test.
//! ```

/// Import native paths and the atomic counter used only for unique fixture names.
use std::path::PathBuf;
/// Import the atomic counter that gives every fixture directory a unique name.
use std::sync::atomic::{AtomicU64, Ordering};

/// What:
///  A process-local sequence shared safely by concurrent tests.
/// Why:
///  PID plus sequence separates fixtures without depending on clock resolution.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Atomics.add(sequence, 0, 1) reserves one fixture identity.
/// ```
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// What:
///  A newly created directory and its cleanup ownership.
/// Why:
///  Only a successful create grants permission to delete this path on disposal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Fixture { readonly path: Path; [Symbol.dispose]() { removeOwnedDirectory(this.path); } }
/// ```
pub(crate) struct Fixture {
    /// Root created exclusively for this fixture.
    pub(crate) path: PathBuf,
}

/// What:
///  Create a directory whose ownership belongs to this test alone.
/// Why:
///  Existing paths produce an error rather than being reused or deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Fixture();
/// ```
impl Fixture {
    /// Allocate and exclusively create one native fixture directory.
    pub(crate) fn new() -> Fixture {
        // Relaxed ordering still gives each caller a distinct number; no other memory is coordinated.
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let name = format!("monochromatic-lint-test-{}-{sequence}", std::process::id());
        let path = crate::test_scratch::scratch_root().join(name);
        std::fs::create_dir(&path).expect("exclusively create test directory");
        return Fixture { path };
    }
}

/// What:
///  Remove the owned fixture when its scope exits,
///  including assertion unwinding.
/// Why:
///  Disposable tests must not leave ordinary successful-run artifacts in the host temporary directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [Symbol.dispose]() { removeOwnedDirectory(this.path); }
/// ```
impl Drop for Fixture {
    /// Delete only the path that this instance successfully created.
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).expect("remove owned test directory");
    }
}
