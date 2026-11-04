//! What:
//! Helpers the tests of the control group share:
//! a group made under the test process's own group that is stopped and removed when the test ends, however it ends.
//! Why:
//! A test that needs a delegated group must fail loudly where none can be had, not pass without proving anything;
//! `CARGO_SHADOW_ALLOW_NO_CGROUP=1` turns that into a printed skip for hosts that cannot delegate (some CI runners).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function testGroup(): TestGroup | undefined
//! ```

use std::path::PathBuf;
use std::time::Duration;

use crate::control_group::{ControlGroup, own_group_directory};

/// The environment variable that lets the tests that need a control group skip, with a printed reason, where none can be had.
pub(crate) const ALLOW_NO_CGROUP: &str = "CARGO_SHADOW_ALLOW_NO_CGROUP";

/// How long a test gives a group to stop.
const TEST_STOP: Duration = Duration::from_secs(10);

/// A group that is stopped and removed when it drops.
pub(crate) struct TestGroup {
    /// The group.
    pub(crate) group: ControlGroup,
    /// The group the test process was in when the group was made.
    pub(crate) parent: PathBuf,
}

/// How the guard cleans up.
impl Drop for TestGroup {
    /// What:
    /// `fn drop(&mut self)` stops every member and removes the group.
    /// Why:
    /// A failed assertion must not leave a process or a directory behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        let _ = self.group.stop(TEST_STOP);
        let _ = self.group.remove();
    }
}

/// What:
/// `pub(crate) fn test_group() -> Option<TestGroup>` makes a group under the test process's own group,
/// or panics with the missing capability,
/// or returns `None` after printing a skip when `CARGO_SHADOW_ALLOW_NO_CGROUP` is set.
/// Why:
/// See the module comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function testGroup(): TestGroup | undefined
/// ```
pub(crate) fn test_group() -> Option<TestGroup> {
    let outcome: Result<TestGroup, String> = match own_group_directory() {
        Ok(parent) => match ControlGroup::create_in(&parent) {
            Ok(group) => Ok(TestGroup { group, parent }),
            Err(error) => Err(error.to_string()),
        },
        Err(error) => Err(error.to_string()),
    };
    return match outcome {
        Ok(found) => Some(found),
        Err(reason) => {
            assert!(std::env::var_os(ALLOW_NO_CGROUP).is_some(), "this host cannot give the test a control group ({reason}); set {ALLOW_NO_CGROUP}=1 to skip the tests that need one");
            eprintln!("SKIPPED: no control group ({reason})");
            None
        }
    };
}
