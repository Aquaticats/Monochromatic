//! Controls of the run's directory:
//!  the folder the caller names is where the run is made and its removal takes
//!  it,
//!  a name that is no directory refuses in the sentence a person reads,
//!  and the folder a name resolves to is the folder the caller named rather than
//!  one the environment holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const runtime = RuntimeFolder.at(fixture('run')); const run = makeRunDirectory(runtime); removeRunDirectory(run);
//! ```

use super::{
    COMPANION_FOLDER,
    RUN_PREFIX,
    RunDirectory,
    RuntimeFolder,
    ScratchRefusal,
    make_run_directory,
    remove_run_directory,
};
use std::path::PathBuf;

/// The fixture root one control keeps its runtime folder in.
fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("account-companion-scratch", name);
}

/// What:
///  The run is made under the folder the caller named,
///  its scratch is a directory inside it,
///  and its removal takes the run and leaves the folder the sweep shares.
#[test]
fn the_run_is_made_under_the_folder_the_caller_named_and_its_removal_takes_it() {
    let root: PathBuf = fixture("run-under-the-folder-the-caller-named");
    let runtime: RuntimeFolder =
        RuntimeFolder::at(root.as_path()).expect("the fixture root is a directory");
    let run: RunDirectory = make_run_directory(&runtime).expect("one run directory is made");
    assert!(
        run.root.starts_with(root.join(COMPANION_FOLDER)),
        "the run is under the companion's folder in the folder the caller named: {}",
        run.root.display()
    );
    let name: String = run
        .root
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    assert!(
        name.starts_with(RUN_PREFIX),
        "the run's name carries the prefix the sweep reads: {name}"
    );
    assert!(
        run.scratch.is_dir(),
        "the scratch mount point is a directory: {}",
        run.scratch.display()
    );
    assert_eq!(
        run.scratch,
        run.root.join("scratch"),
        "the scratch mount point is inside the run's own directory"
    );
    remove_run_directory(&run);
    assert!(
        !run.root.exists(),
        "the removal takes the run's directory: {}",
        run.root.display()
    );
    assert!(
        root.join(COMPANION_FOLDER).is_dir(),
        "the folder the sweep shares stays for the next add and keeps its neighbours"
    );
}

/// What:
///  A name that is no directory refuses the scratch in the sentence a person
///  reads,
///  and the sentence names the path that was tried.
/// Why:
///  A session whose runtime directory a logout removed meets this refusal,
///  and it is told what to do rather than given an error number.
#[test]
fn a_name_that_is_no_directory_refuses_in_the_sentence_a_person_reads() {
    let root: PathBuf = fixture("name-is-no-directory");
    let missing: PathBuf = root.join("no-such-runtime-folder");
    let refusal: ScratchRefusal = RuntimeFolder::at(missing.as_path())
        .expect_err("a path that is no directory names no runtime folder");
    let expected: String = format!(
        "XDG_RUNTIME_DIR names {}, which is no directory, so the account companion cannot keep its scratch there. Ways forward: log in again so the session makes it, then run the add again",
        missing.display()
    );
    assert_eq!(refusal.message, expected);
}

/// What:
///  The folder a name resolves to is the folder that name holds,
///  which is what makes the add route's one read of the variable the whole of
///  what the environment contributes.
#[test]
fn the_folder_a_name_resolves_to_is_the_folder_the_caller_named() {
    let root: PathBuf = fixture("name-resolves-to-the-folder-it-holds");
    let named: std::ffi::OsString = root.clone().into_os_string();
    let runtime: RuntimeFolder =
        RuntimeFolder::from_name(Some(named)).expect("the fixture root is a directory");
    assert_eq!(runtime.path(), root.as_path());
}
