//! What:
//!  The one place where a binary-level control states that it needs a btrfs fixture root,
//! and what it does where there is none.
//! Why:
//!  The container gate runs on an overlay filesystem,
//! where the btrfs ioctls cannot run,
//! so a control there must not pass
//!  as coverage it does not have.
//! It fails with the filesystem it found unless the run states that the absence is
//!  known,
//! and then it prints one line naming itself and the reason it did not run.
//! Three copies of that decision
//!  lived in the test files themselves and had already drifted apart in visibility and in wording.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // if (!onBtrfsOrAcknowledged(fixture, 'the_control')) return;
//! ```

use super::support::Fixture;
use git_policy_cli::fs_kind::{FilesystemKind, filesystem_kind};

/// The environment variable that acknowledges "this run has no btrfs fixture root".
/// The container gate image sets it;
/// the host task `native:test:host:btrfs` unsets it,
/// so no control on btrfs can
///  quietly pass.
pub const ABSENT_ACKNOWLEDGED_VARIABLE: &str = "CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED";

/// What:
///  Whether the fixture root is on btrfs,
/// or `false` after one line naming the control and the filesystem it found.
/// Why:
///   A control that needs the ioctls returns early on `false`,
/// and a run that never said it has no btrfs fails the
///  control with both ways forward instead of reporting a pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onBtrfsOrAcknowledged(fixture: Fixture, control: string): boolean;
/// ```
pub fn on_btrfs_or_acknowledged(fixture: &Fixture, control: &str) -> bool {
    let kind: Result<FilesystemKind, _> = filesystem_kind(fixture.root.as_path());
    if kind == Ok(FilesystemKind::Btrfs) {
        return true;
    }
    let acknowledged: bool = std::env::var_os(ABSENT_ACKNOWLEDGED_VARIABLE).is_some_and(|value| { return !value.is_empty(); });
    assert!(
        acknowledged,
        "the control {control} needs a btrfs fixture root, but {} is on {kind:?}; set CLI_GIT_TEST_ROOT to a directory on \
         btrfs, or set {ABSENT_ACKNOWLEDGED_VARIABLE}=1 to state that this run has none",
        fixture.root.display()
    );
    eprintln!("BTRFS CONTROL NOT RUN: {control} (fixture root {} is on {kind:?}; the host task native:test:host:btrfs covers it)", fixture.root.display());
    return false;
}
