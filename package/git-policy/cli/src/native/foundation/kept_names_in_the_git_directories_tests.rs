//! What:
//!  Disposable-repository controls for the settling of every name the program keeps in the Git directories.
//! Why:
//!  Each kind the wrapper keeps (saved copies, notices, work records, logs, merge records, copy records) must move in place
//!  with its content intact when only the earlier name exists, be used when only the current name exists, and leave the
//!  earlier one untouched and reported when both exist.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await plant(gitDir, 'cli-git/discarded/x'); settleKeptNames(gitDir, commonDir); expect(await read(gitDir, 'monochromatic-git-policy-cli/discarded/x')).toBe(...);
//! ```

/// The functions under test and the pairs they settle.
use super::{data_folder_for_reading, settle_kept_names};
use crate::wrapper_names_old_and_new::{
    COMMIT_TIMING_LOG, DATA_FOLDER, DISCARD_AUDIT_LOG, MERGE_INTENT_FOLDER, PUSH_TRIGGER_LOG, RenamedName, WORKTREE_ADD_TIMING_LOG,
    WORKTREE_COPY_RECORD_FOLDER,
};
use std::path::{Path, PathBuf};

/// Create one empty fixture directory owned by this test process and test name.
fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("native-kept-names", name);
}

/// The kinds that are folders,
///  with the file each holds in the test.
const FOLDER_KINDS: [RenamedName; 3] = [DATA_FOLDER, WORKTREE_COPY_RECORD_FOLDER, MERGE_INTENT_FOLDER];

/// The kinds that are files.
const FILE_KINDS: [RenamedName; 4] = [DISCARD_AUDIT_LOG, COMMIT_TIMING_LOG, WORKTREE_ADD_TIMING_LOG, PUSH_TRIGGER_LOG];

/// Plant a kind under its earlier name,
///  with content that names the kind.
fn plant_earlier(directory: &Path, kind: RenamedName) {
    let path: PathBuf = directory.join(kind.earlier);
    if FOLDER_KINDS.contains(&kind) {
        std::fs::create_dir_all(path.join("inner")).expect("create the folder");
        std::fs::write(path.join("inner").join("record"), kind.earlier).expect("write the record");
    } else {
        std::fs::write(path, kind.earlier).expect("write the file");
    }
}

/// Read back what `plant_earlier` wrote,
///  under the current name.
fn read_current(directory: &Path, kind: RenamedName) -> String {
    let path: PathBuf = directory.join(kind.current);
    if FOLDER_KINDS.contains(&kind) {
        return std::fs::read_to_string(path.join("inner").join("record")).expect("the record moved with the folder");
    }
    return std::fs::read_to_string(path).expect("the file moved");
}

/// A linked worktree's Git directory keeps the data folder and the merge records,
///  and the common Git directory keeps the rest;
///  every kind planted under its earlier name is renamed in place with its content.
#[test]
fn every_kind_planted_under_its_earlier_name_is_renamed_in_place_with_its_content() {
    let root: PathBuf = fixture("every-kind");
    let common: PathBuf = root.join("common");
    let own: PathBuf = common.join("worktrees").join("linked");
    std::fs::create_dir_all(own.as_path()).expect("directories");
    for kind in [DATA_FOLDER, MERGE_INTENT_FOLDER] {
        plant_earlier(own.as_path(), kind);
    }
    for kind in [DATA_FOLDER, WORKTREE_COPY_RECORD_FOLDER, DISCARD_AUDIT_LOG, COMMIT_TIMING_LOG, WORKTREE_ADD_TIMING_LOG, PUSH_TRIGGER_LOG] {
        plant_earlier(common.as_path(), kind);
    }
    settle_kept_names(own.as_path(), common.as_path());
    for kind in [DATA_FOLDER, MERGE_INTENT_FOLDER] {
        assert_eq!(read_current(own.as_path(), kind), kind.earlier, "{kind:?} in the worktree directory");
        assert!(!own.join(kind.earlier).exists(), "{kind:?}: no second name");
    }
    for kind in [DATA_FOLDER, WORKTREE_COPY_RECORD_FOLDER, DISCARD_AUDIT_LOG, COMMIT_TIMING_LOG, WORKTREE_ADD_TIMING_LOG, PUSH_TRIGGER_LOG] {
        assert_eq!(read_current(common.as_path(), kind), kind.earlier, "{kind:?} in the common directory");
        assert!(!common.join(kind.earlier).exists(), "{kind:?}: no second name");
    }
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// The main worktree's Git directory is the common directory:
///  settling it once for both roles is the same as for each.
#[test]
fn a_main_worktree_directory_serves_both_roles() {
    let root: PathBuf = fixture("main-worktree");
    for kind in [DATA_FOLDER, MERGE_INTENT_FOLDER, DISCARD_AUDIT_LOG, PUSH_TRIGGER_LOG] {
        plant_earlier(root.as_path(), kind);
    }
    settle_kept_names(root.as_path(), root.as_path());
    for kind in [DATA_FOLDER, MERGE_INTENT_FOLDER, DISCARD_AUDIT_LOG, PUSH_TRIGGER_LOG] {
        assert_eq!(read_current(root.as_path(), kind), kind.earlier, "{kind:?}");
    }
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Names already under the current spelling are used as they are,
///  and a second settling changes nothing.
#[test]
fn current_names_are_left_alone_and_settling_twice_is_the_same() {
    let root: PathBuf = fixture("current-only");
    for kind in FOLDER_KINDS.iter().chain(FILE_KINDS.iter()) {
        let path: PathBuf = root.join(kind.current);
        if FOLDER_KINDS.contains(kind) {
            std::fs::create_dir_all(path.join("inner")).expect("folder");
            std::fs::write(path.join("inner").join("record"), "current").expect("record");
        } else {
            std::fs::write(path, "current").expect("file");
        }
    }
    settle_kept_names(root.as_path(), root.as_path());
    settle_kept_names(root.as_path(), root.as_path());
    for kind in FOLDER_KINDS.iter().chain(FILE_KINDS.iter()) {
        assert_eq!(read_current(root.as_path(), *kind), "current", "{kind:?}");
        assert!(!root.join(kind.earlier).exists(), "{kind:?}: nothing was created under the earlier name");
    }
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Both spellings of a kind exist:
///  the current one is used and the earlier one keeps every byte.
#[test]
fn both_spellings_leave_the_earlier_one_untouched() {
    let root: PathBuf = fixture("both");
    plant_earlier(root.as_path(), DATA_FOLDER);
    std::fs::create_dir_all(root.join(DATA_FOLDER.current)).expect("the current folder");
    settle_kept_names(root.as_path(), root.as_path());
    assert_eq!(std::fs::read_to_string(root.join(DATA_FOLDER.earlier).join("inner").join("record")).expect("untouched"), DATA_FOLDER.earlier);
    assert!(root.join(DATA_FOLDER.current).is_dir());
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// The audit log's numbered older copies move with it,
///  each keeping its number,
///  because their lines name saved files still on disk.
#[test]
fn the_numbered_older_copies_of_a_rotated_log_move_with_it() {
    let root: PathBuf = fixture("generations");
    std::fs::write(root.join(DISCARD_AUDIT_LOG.earlier), "newest").expect("log");
    std::fs::write(root.join(format!("{}.1", DISCARD_AUDIT_LOG.earlier)), "one").expect("first");
    std::fs::write(root.join(format!("{}.12", DISCARD_AUDIT_LOG.earlier)), "twelve").expect("twelfth");
    std::fs::write(root.join(format!("{}.old", DISCARD_AUDIT_LOG.earlier)), "not a number").expect("other");
    settle_kept_names(root.as_path(), root.as_path());
    assert_eq!(std::fs::read_to_string(root.join(DISCARD_AUDIT_LOG.current)).expect("log"), "newest");
    assert_eq!(std::fs::read_to_string(root.join(format!("{}.1", DISCARD_AUDIT_LOG.current))).expect("first"), "one");
    assert_eq!(std::fs::read_to_string(root.join(format!("{}.12", DISCARD_AUDIT_LOG.current))).expect("twelfth"), "twelve");
    assert!(root.join(format!("{}.old", DISCARD_AUDIT_LOG.earlier)).exists(), "a name that is not a number is not a copy");
    assert!(!root.join(format!("{}.1", DISCARD_AUDIT_LOG.earlier)).exists());
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A reader that visits another worktree's directory finds the earlier folder when only that exists,
///  and does not rename it.
#[test]
fn a_reader_of_another_worktree_does_not_move_its_folder() {
    let root: PathBuf = fixture("reader");
    plant_earlier(root.as_path(), DATA_FOLDER);
    assert_eq!(data_folder_for_reading(root.as_path()), root.join(DATA_FOLDER.earlier));
    assert!(root.join(DATA_FOLDER.earlier).is_dir(), "not renamed by a reader");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Directories that do not exist cost nothing and fail nothing.
#[test]
fn directories_that_do_not_exist_are_not_an_error() {
    let root: PathBuf = fixture("absent");
    settle_kept_names(root.join("no-git-dir").as_path(), root.join("no-common-dir").as_path());
    assert_eq!(std::fs::read_dir(root.as_path()).expect("list").count(), 0);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Awkward directory names (spaces,
///  quotes,
///  a newline) are ordinary paths to the settling.
#[test]
fn awkward_directory_names_are_settled_like_any_other() {
    let root: PathBuf = fixture("awkward");
    let directory: PathBuf = root.join("a repo's \"git\"\ndir");
    std::fs::create_dir_all(directory.as_path()).expect("directory");
    plant_earlier(directory.as_path(), DATA_FOLDER);
    plant_earlier(directory.as_path(), PUSH_TRIGGER_LOG);
    settle_kept_names(directory.as_path(), directory.as_path());
    assert_eq!(read_current(directory.as_path(), DATA_FOLDER), DATA_FOLDER.earlier);
    assert_eq!(read_current(directory.as_path(), PUSH_TRIGGER_LOG), PUSH_TRIGGER_LOG.earlier);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}
