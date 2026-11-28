//! What:
//!  The controls of the live pass:
//!  a repository the route never served costs one look,
//!  asks no reader and prints nothing,
//!  and the look reads the configured plan folder and no other.
//! Why:
//!  The pass runs before every forwarded command,
//!  so the healthy state is the state that matters most:
//!  it must not open a pool,
//!  write a record or create the folder it looked for.
//!  The reader is a seam here,
//!  so a control shows the guard missing without
//!  opening this machine's ZFS device.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(finishTheRemovalPlansOver(commonDir, recorder)).toEqual([]);
//! expect(recorder.asked).toBe(false);
//! ```

use super::{
    TheDatasetReaderOpener,
    finish_the_removal_plans_over,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::registration_home::{
    home_folder,
    plans_folder,
};
use crate::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader;
use std::cell::Cell;
use std::path::PathBuf;

/// What:
///  The reader opener a control hands the pass:
///  it records that it was asked and opens nothing.
/// Why:
///  The guard is about whether the pass asks for a reader at all,
///  and a control that answered with a real reader would open this machine's
///  ZFS device,
///  which no control may do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorder = { asked: false, open() { this.asked = true; throw new Error('no device'); } };
/// ```
struct TheReaderThatRecords {
    asked: Cell<bool>,
}

impl TheReaderThatRecords {
    fn new() -> TheReaderThatRecords {
        return TheReaderThatRecords {
            asked: Cell::new(false),
        };
    }
}

impl TheDatasetReaderOpener for TheReaderThatRecords {
    fn open(&self) -> Result<ZfsDatasetReader, String> {
        self.asked.set(true);
        return Err(String::from("a control opens no device"));
    }
}

#[test]
fn a_repository_with_no_plan_folder_prints_nothing_and_asks_no_reader() {
    let common: PathBuf = fixture("the_live_pass_with_no_plan_folder");
    let reader: TheReaderThatRecords = TheReaderThatRecords::new();
    let said: Vec<String> = finish_the_removal_plans_over(common.as_path(), &reader);
    assert!(
        !reader.asked.get(),
        "a repository the route never served opens no pool, and this one asked for a reader"
    );
    assert!(
        said.is_empty(),
        "a repository the route never served prints no line, and this one printed {said:?}"
    );
    assert!(
        !plans_folder(common.as_path()).exists(),
        "the pass looked for the plan folder and left it exactly as it found it"
    );
}

#[test]
fn the_look_reads_the_configured_plan_folder_and_no_other() {
    // The configured folder decides: with it standing the pass asks for the reader, and a reader that
    // answers an error is reported rather than reading as an empty folder.
    let configured: PathBuf = fixture("the_live_pass_configured_place");
    let folder: PathBuf = plans_folder(configured.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the configured plan folder is made");
    let asked: TheReaderThatRecords = TheReaderThatRecords::new();
    let said: Vec<String> = finish_the_removal_plans_over(configured.as_path(), &asked);
    assert!(
        asked.asked.get(),
        "the configured plan folder stands, so the pass asks the pool before it reads a plan"
    );
    assert!(
        said.iter()
            .any(|line| return line.contains("the pool could not be read")),
        "a reader that cannot open is one line about the pool: {said:?}"
    );
    assert!(
        folder.is_dir(),
        "a device that could not be opened removes nothing from the folder it did not list"
    );

    // Another place with the same leaf name decides nothing, because the pass reads the configured one.
    let elsewhere: PathBuf = fixture("the_live_pass_another_place");
    let decoy: PathBuf = home_folder(elsewhere.as_path()).join("plans");
    std::fs::create_dir_all(decoy.as_path()).expect("the other place's folder is made");
    let unasked: TheReaderThatRecords = TheReaderThatRecords::new();
    let quiet: Vec<String> = finish_the_removal_plans_over(elsewhere.as_path(), &unasked);
    assert!(
        !unasked.asked.get(),
        "a folder that is not the configured place stands, and the pass asked for the pool anyway"
    );
    assert!(
        quiet.is_empty(),
        "a folder that is not the configured place prints nothing: {quiet:?}"
    );
    assert!(
        decoy.is_dir(),
        "the pass never looked at the other place, so it removed nothing there"
    );
}
