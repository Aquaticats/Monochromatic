//! What:
//!  The one place where a binary-level control states that it needs a fixture root on which a file can be
//!  reflinked (`FICLONE`), and what it does where there is none.
//! Why:
//!  A control that asserts the clone rung served its add needs a root on which `FICLONE` works:
//! the gate image's fixture root is a tmpfs the container makes (`bin/native-gate-fixture-root.ts`),
//! which answers no reflink,
//!  so a control there must not pass as coverage it has not got.
//! It prints one line naming itself and the reason it did not run;
//! a root that can reflink needs no btrfs,
//! so the same controls run on a btrfs host,
//! a ZFS host and an XFS host with reflink.
//! The release pass over alpha.45 found the end-to-end conversion controls gated on ZFS alone,
//! so a btrfs host never ran them
//! (`~/temp/agent/release-pass-wrapper-alpha-45-2026-11-23/clone-rung/keep/review.md`,
//! finding M4).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // if (!onAReflinkCapableFixture(fixture, 'the_control')) return;
//! ```

use super::support::Fixture;
use git_policy_cli::fs_clone_exchange::clone_file;
use std::path::PathBuf;

/// What:
///  Whether a file on the fixture root can be reflinked,
/// or `false` after one line naming the control and the reason it did not run.
/// Why:
///   The probe is the mechanism the rung uses,
/// `FICLONE` of one file onto another,
/// so a root that answers it is the root such a control needs,
///  and no filesystem name stands in for the answer.
/// The probe's two files are removed,
/// whether the clone worked or not,
/// so the control that follows sees the root it expects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onAReflinkCapableFixture(fixture: Fixture, control: string): boolean;
/// ```
pub fn on_a_reflink_capable_fixture(fixture: &Fixture, control: &str) -> bool {
    let from: PathBuf = fixture.root.join("reflink-probe-from");
    let to: PathBuf = fixture.root.join("reflink-probe-to");
    std::fs::write(from.as_path(), b"probe\n").expect("the probe's file");
    std::fs::write(to.as_path(), b"").expect("the probe's clone");
    let opened = (std::fs::File::open(from.as_path()), std::fs::OpenOptions::new().write(true).open(to.as_path()));
    let cloned: bool = match opened {
        (Ok(source), Ok(destination)) => clone_file(&source, &destination, to.as_path()).is_ok(),
        _unreadable => false,
    };
    let _removed = (std::fs::remove_file(from.as_path()), std::fs::remove_file(to.as_path()));
    if cloned {
        return true;
    }
    eprintln!("CLONE CONTROL NOT RUN: {control} (the fixture root {} cannot reflink a file, so the clone rung cannot serve an add there)", fixture.root.display());
    return false;
}
