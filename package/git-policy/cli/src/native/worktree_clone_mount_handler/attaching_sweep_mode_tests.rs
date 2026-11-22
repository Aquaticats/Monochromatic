//! What:
//!  Controls of the sweep mode:
//!  the places it reads,
//!  the configuration refusal that comes before anything is opened,
//!  and the probe's record that a boot without one never passes.
//! Why:
//!  A sweep that took a guessed path or guessed a pool would attach another
//!  checkout's clones,
//!  and a host control may not open `/dev/zfs`,
//!  so what this host proves is the order:
//!  the configuration,
//!  then the probe's record and the checkout walk,
//!  and only then the device.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(SweepPlaces.installed().configuration).toBe('/etc/…/git-worktree-clone-mounts.jsonc');
//! ```

use super::{SweepPlaces, run_at};
use crate::start_probe::{RECORD_DIRECTORY, RESULT_NAME};
use crate::worktree_clone_mount::SLOT_IDENTITY_STORE_PATH;
use crate::worktree_clone_mount_handler::handler_install_paths::CONFIGURATION_PATH;
use crate::worktree_clone_mount_handler::sweep_configuration::{CHECKOUT_DATASET_KEY, CLONE_MOUNTS_KEY, CHECKOUT_KEY, PARENT_KEY};
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
        return Fixture { path: crate::test_support::fixture(format!("attaching-sweep-mode-{name}").as_str()) };
    }

    /// Write one configuration document and answer its path.
    fn configuration(&self, document: &str) -> PathBuf {
        let file: PathBuf = self.path.join("git-worktree-clone-mounts.jsonc");
        std::fs::write(file.as_path(), document).expect("the fixture configuration is written");
        return file;
    }
}

/// The installed places are the design's own four paths.
#[test]
fn the_installed_places_are_the_designs() {
    let places: SweepPlaces = SweepPlaces::installed();
    assert_eq!(places.configuration, PathBuf::from(CONFIGURATION_PATH));
    assert_eq!(places.store, PathBuf::from(SLOT_IDENTITY_STORE_PATH));
    assert_eq!(places.probe_record, PathBuf::from(RECORD_DIRECTORY).join(RESULT_NAME));
    let fixture: PathBuf = crate::test_support::fixture("a-fixture-configuration");
    let handed: SweepPlaces = SweepPlaces::at(fixture.clone(), PathBuf::from("/fixture/store"), PathBuf::from("/fixture/record"));
    assert_eq!(handed.configuration, fixture);
    assert_ne!(handed.store, places.store, "a control's store is never the installed one");
}

/// What:
///  A run whose configuration is not there answers the configuration's refusal,
///  naming the two keys to write,
///  and opens nothing.
/// Why:
///  The configuration is the first thing a run reads,
///  so a host with no installed file must learn what to write rather than that
///  a device failed.
#[test]
fn a_missing_configuration_is_refused_before_anything_is_opened() {
    let fixture: Fixture = Fixture::new("missing");
    let places: SweepPlaces = SweepPlaces::at(fixture.path.join("nothing.jsonc"), fixture.path.join("store"), fixture.path.join("record"));
    let Err(said): Result<String, String> = run_at(&places) else {
        panic!("a run with no configuration answered a report");
    };
    assert!(said.contains("is not there"), "{said}");
    assert!(said.contains(&format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}")), "{said}");
    assert!(said.contains(&format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}")), "{said}");
    assert!(!PathBuf::from(&places.store).exists(), "a refused run wrote no store");
}

/// What:
///  A configuration that names both keys and a boot with no probe record is
///  refused by the record,
///  before the device is opened.
/// Why:
///  No arm of a boot that never probed is evidence about this kernel,
///  so the run attaches nothing and says which mode writes the record.
#[test]
fn a_boot_with_no_probe_record_is_refused_before_the_device() {
    let fixture: Fixture = Fixture::new("no-record");
    let document: String = format!(
        r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "zpcachyos/nosnap/clones", "{CHECKOUT_KEY}": "{}", "{CHECKOUT_DATASET_KEY}": "zpcachyos/nosnap/checkout" }} }}"#,
        fixture.path.display()
    );
    let configuration: PathBuf = fixture.configuration(document.as_str());
    let places: SweepPlaces = SweepPlaces::at(configuration, fixture.path.join("store"), fixture.path.join("probe-record"));
    let Err(said): Result<String, String> = run_at(&places) else {
        panic!("a run with no probe record answered a report");
    };
    assert!(said.contains("start probe"), "the refusal is the record's own: {said}");
    assert!(said.contains("is not there"), "{said}");
    assert!(!PathBuf::from(&places.store).exists(), "a refused run wrote no store");
}
