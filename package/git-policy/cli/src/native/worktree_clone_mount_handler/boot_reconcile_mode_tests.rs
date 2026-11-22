//! What:
//!  Controls of the boot reconciliation:
//!  the places it reads,
//!  and that a boot whose probe record is not there publishes no readiness and
//!  attaches nothing.
//! Why:
//!  The mode's first job is its own checks,
//!  and a readiness record published instead of a failed prerequisite would
//!  promise a route the boot does not have,
//!  which is what a host control can prove without a device or `/run`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(ReconcilePlaces.installed().store).toBe('/var/lib/…/git-worktree-clone-slot-identities');
//! ```

use super::{ReconcilePlaces, run_at};
use crate::start_probe::{RECORD_DIRECTORY, RESULT_NAME};
use crate::worktree_clone_mount::SLOT_IDENTITY_STORE_PATH;
use crate::worktree_clone_mount_handler::handler_install_paths::CONFIGURATION_PATH;
use crate::worktree_clone_mount_handler::sweep_configuration::{CLONE_MOUNTS_KEY, CHECKOUT_KEY, PARENT_KEY};
use crate::worktree_clone_mount_protocol::{READINESS_NAME, ReadinessFile};
use std::path::PathBuf;

/// One fixture folder of this control's own,
/// removed however the control ends.
struct Fixture {
    /// The folder.
    path: PathBuf,
}

impl Fixture {
    /// A fresh folder under this process's own temporary root.
    fn new(name: &str) -> Self {
        return Fixture { path: crate::test_support::fixture(format!("boot-reconcile-mode-{name}").as_str()) };
    }

    /// Write one configuration document and answer its path.
    fn configuration(&self, document: &str) -> PathBuf {
        let file: PathBuf = self.path.join("git-worktree-clone-mounts.jsonc");
        std::fs::write(file.as_path(), document).expect("the fixture configuration is written");
        return file;
    }
}

/// The installed places are the design's own three paths.
#[test]
fn the_installed_places_are_the_designs() {
    let places: ReconcilePlaces = ReconcilePlaces::installed();
    assert_eq!(places.configuration, PathBuf::from(CONFIGURATION_PATH));
    assert_eq!(places.store, PathBuf::from(SLOT_IDENTITY_STORE_PATH));
    assert_eq!(places.probe_record, PathBuf::from(RECORD_DIRECTORY).join(RESULT_NAME));
}

/// What:
///  A boot whose probe record is not there is refused,
///  and no readiness record is written for it.
/// Why:
///  A readiness record published instead of a failed prerequisite would promise
///  a boot whose clones were never reconciled,
///  and the design's order is the checks,
///  the sweep,
///  then readiness.
#[test]
fn a_boot_with_no_probe_record_publishes_no_readiness() {
    let fixture: Fixture = Fixture::new("no-record");
    let document: String = format!(
        r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "zpcachyos/nosnap/clones", "{CHECKOUT_KEY}": "{}" }} }}"#,
        fixture.path.display()
    );
    let configuration: PathBuf = fixture.configuration(document.as_str());
    let places: ReconcilePlaces = ReconcilePlaces::at(configuration, fixture.path.join("store"), fixture.path.join("probe-record"));
    let readiness: ReadinessFile = ReadinessFile::at(fixture.path.join("readiness"), 0);
    let Err(said): Result<String, String> = run_at(&places, &readiness) else {
        panic!("a boot with no probe record answered a report");
    };
    assert!(said.contains("start probe"), "the refusal is the record's own: {said}");
    assert!(said.contains("Nothing was enumerated"), "the refusal says nothing acted: {said}");
    assert!(!readiness.directory().join(READINESS_NAME).exists(), "no readiness record was published for a boot that failed its checks");
    assert!(!PathBuf::from(&places.store).exists(), "a refused boot wrote no store");
}
