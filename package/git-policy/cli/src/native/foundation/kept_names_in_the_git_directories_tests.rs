//! What:
//!  Disposable-repository controls for the settling of every name the program keeps in the Git directories.
//! Why:
//!  Each kind the wrapper keeps (saved copies,
//! notices,
//! work records,
//! logs,
//! merge records,
//! copy records) must move in place
//!  with its content intact when only the earlier name exists,
//! be used when only the current name exists,
//! and leave the
//!  earlier one untouched and reported when both exist.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await plant(gitDir, 'cli-git/discarded/x'); settleKeptNames(gitDir, commonDir); expect(await read(gitDir, 'monochromatic-git-policy-cli/discarded/x')).toBe(...);
//! ```

/// The functions under test and the pairs they settle.
use super::{data_folder_for_reading, settle_kept_names_within};
/// The surroundings the controls give the fold,
/// which name no process and no state folder of this host.
use crate::earlier_names_folded_into_current::FoldSurroundings;
/// The steps a fold reports through when nothing stops it.
use crate::earlier_log_lines_added_to_current_log::NeverStopped;
use crate::earlier_name_moved_to_current_in_place::place_kept_name;
use crate::wrapper_names_old_and_new::{
    COMMIT_TIMING_LOG, DATA_FOLDER, DISCARD_AUDIT_LOG, MERGE_INTENT_FOLDER, PUSH_TRIGGER_LOG, RenamedName, WORKTREE_ADD_TIMING_LOG,
    WORKTREE_COPY_RECORD_FOLDER,
};
use std::path::{Path, PathBuf};

/// Create one empty fixture directory owned by this test process and test name.
fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("native-kept-names", name);
}

/// What:
///  Settle a fixture's names with surroundings of its own.
/// Why:
///   The fold defers while an earlier release's service may be running,
/// and the look reads a process table and a state folder.
/// Reading the host's made four of these controls defer a fold
/// whenever this machine's own services were met (2026-11-08),
/// so the control gives an empty process table and no state folder instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settleWithin(fixture, fixture, { procRoot: emptyDir, stateDir: null });
/// ```
fn settle_within(git_dir: &Path, common_dir: &Path) {
    // The fixture root holds only directories named after tests, so it is a process table that lists no process.
    let proc_root: PathBuf = crate::test_support::fixture_root();
    let surroundings: FoldSurroundings = FoldSurroundings { state_dir: None, proc_root };
    settle_kept_names_within(git_dir, common_dir, &surroundings, &mut NeverStopped);
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
    settle_within(own.as_path(), common.as_path());
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
    settle_within(root.as_path(), root.as_path());
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
    settle_within(root.as_path(), root.as_path());
    settle_within(root.as_path(), root.as_path());
    for kind in FOLDER_KINDS.iter().chain(FILE_KINDS.iter()) {
        assert_eq!(read_current(root.as_path(), *kind), "current", "{kind:?}");
        assert!(!root.join(kind.earlier).exists(), "{kind:?}: nothing was created under the earlier name");
    }
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Both spellings of a kind exist:
///  the current one is used and the earlier one is folded into it with every byte.
#[test]
fn both_spellings_are_folded_into_the_current_one_with_every_byte() {
    let root: PathBuf = fixture("both");
    plant_earlier(root.as_path(), DATA_FOLDER);
    std::fs::create_dir_all(root.join(DATA_FOLDER.current)).expect("the current folder");
    settle_within(root.as_path(), root.as_path());
    assert_eq!(std::fs::read_to_string(root.join(DATA_FOLDER.current).join("inner").join("record")).expect("moved"), DATA_FOLDER.earlier);
    assert!(!root.join(DATA_FOLDER.earlier).exists(), "the earlier name is gone");
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
    settle_within(root.as_path(), root.as_path());
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
    settle_within(root.join("no-git-dir").as_path(), root.join("no-common-dir").as_path());
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
    settle_within(directory.as_path(), directory.as_path());
    assert_eq!(read_current(directory.as_path(), DATA_FOLDER), DATA_FOLDER.earlier);
    assert_eq!(read_current(directory.as_path(), PUSH_TRIGGER_LOG), PUSH_TRIGGER_LOG.earlier);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

//region Both spellings exist: the earlier name is folded into the current one
// Measured in this repository's main Git directory on 2026-11-07T08:37:00-05:00: development builds of the renamed wrapper
//  created the current names on 2026-11-03 and 2026-11-04, the installed 0.5.34 kept writing the earlier names until
//  2026-11-07T08:24:40-05:00, so six names existed twice and every command printed six lines.

/// The kinds that live in the common Git directory and exist twice in the measurement.
const KINDS_EXISTING_TWICE: [RenamedName; 6] =
    [DATA_FOLDER, WORKTREE_COPY_RECORD_FOLDER, DISCARD_AUDIT_LOG, COMMIT_TIMING_LOG, WORKTREE_ADD_TIMING_LOG, PUSH_TRIGGER_LOG];

/// The lines a log of `name` holds under one spelling:
/// each carries a time stamp,
/// as the real records do.
fn log_lines(spelling: &str, first_minute: u32) -> Vec<String> {
    return (0..3)
        .map(|index: u32| {
            return format!("{{\"time\":\"2026-11-05T03:{:02}:00.000Z\",\"from\":\"{spelling}\",\"index\":{index}}}\n", first_minute + index);
        })
        .collect();
}

/// Plant every kind under both spellings:
/// a log with its own lines under each,
/// a folder with its own entry under each,
///  and one entry name that both folders hold.
fn plant_both_spellings(directory: &Path) {
    for kind in KINDS_EXISTING_TWICE {
        if FOLDER_KINDS.contains(&kind) {
            for spelling in [kind.earlier, kind.current] {
                let folder: PathBuf = directory.join(spelling);
                std::fs::create_dir_all(folder.join("inner")).expect("folder");
                std::fs::write(folder.join("inner").join(format!("only-in-{spelling}")), spelling).expect("own entry");
                std::fs::write(folder.join("same-name"), spelling).expect("shared entry name");
            }
        } else {
            std::fs::write(directory.join(kind.earlier), log_lines(kind.earlier, 10).concat()).expect("earlier log");
            std::fs::write(directory.join(kind.current), log_lines(kind.current, 20).concat()).expect("current log");
        }
    }
}

/// Every file below `root`,
/// as its bytes,
/// in no particular order.
fn all_file_contents(root: &Path) -> Vec<Vec<u8>> {
    let mut found: Vec<Vec<u8>> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in std::fs::read_dir(next.as_path()).expect("list").flatten() {
            if entry.file_type().expect("type").is_dir() {
                pending.push(entry.path());
            } else {
                found.push(std::fs::read(entry.path()).expect("read"));
            }
        }
    }
    return found;
}

/// The measured state:
/// after one settling no earlier name is left beside its current one,
///  so the six lines that every command printed are gone.
#[test]
fn both_spellings_of_every_kept_name_leave_no_earlier_name_and_no_notice() {
    let root: PathBuf = fixture("fold-no-notice");
    plant_both_spellings(root.as_path());
    settle_within(root.as_path(), root.as_path());
    for kind in KINDS_EXISTING_TWICE {
        assert!(!root.join(kind.earlier).exists(), "{kind:?}: the earlier name is gone");
        assert!(root.join(kind.current).exists(), "{kind:?}: the current name stays");
        assert_eq!(place_kept_name(root.as_path(), kind).notice, None, "{kind:?}: nothing is printed");
    }
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// The history that the split left under the earlier names is not orphaned:
///  every line of both logs,
/// byte for byte,
/// is in the log under the current name.
#[test]
fn every_line_of_both_logs_survives_under_the_current_name() {
    let root: PathBuf = fixture("fold-lines");
    plant_both_spellings(root.as_path());
    settle_within(root.as_path(), root.as_path());
    for kind in FILE_KINDS {
        let mut wanted: Vec<String> = log_lines(kind.earlier, 10);
        wanted.extend(log_lines(kind.current, 20));
        wanted.sort();
        let text: String = std::fs::read_to_string(root.join(kind.current)).expect("current log");
        let mut found: Vec<String> = text.split_inclusive('\n').map(str::to_owned).collect();
        found.sort();
        assert_eq!(found, wanted, "{kind:?}: every line of both, once");
    }
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// The folders keep every entry of both:
/// a name only the earlier folder holds moves in,
/// and a name both hold is kept twice.
#[test]
fn every_entry_of_both_folders_survives_under_the_current_name() {
    let root: PathBuf = fixture("fold-folders");
    plant_both_spellings(root.as_path());
    settle_within(root.as_path(), root.as_path());
    for kind in FOLDER_KINDS.iter().filter(|kind| return KINDS_EXISTING_TWICE.contains(kind)) {
        let found: Vec<Vec<u8>> = all_file_contents(root.join(kind.current).as_path());
        for spelling in [kind.earlier, kind.current] {
            let entries: usize = found.iter().filter(|bytes| return bytes.as_slice() == spelling.as_bytes()).count();
            // One entry only that folder holds, and one under a name both hold.
            assert_eq!(entries, 2, "{kind:?}: both entries of {spelling} are below the current folder");
        }
    }
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A linked worktree's own Git directory holds the data folder and the merge records;
///  both spellings of those fold there too.
#[test]
fn both_spellings_in_a_linked_worktrees_directory_fold_there_too() {
    let root: PathBuf = fixture("fold-worktree");
    let own: PathBuf = root.join("worktrees").join("linked");
    std::fs::create_dir_all(own.as_path()).expect("directory");
    for kind in [DATA_FOLDER, MERGE_INTENT_FOLDER] {
        for spelling in [kind.earlier, kind.current] {
            std::fs::create_dir_all(own.join(spelling)).expect("folder");
            std::fs::write(own.join(spelling).join(format!("only-in-{spelling}")), spelling).expect("own entry");
        }
    }
    settle_within(own.as_path(), root.as_path());
    for kind in [DATA_FOLDER, MERGE_INTENT_FOLDER] {
        assert!(!own.join(kind.earlier).exists(), "{kind:?}: the earlier name is gone");
        for spelling in [kind.earlier, kind.current] {
            assert_eq!(std::fs::read_to_string(own.join(kind.current).join(format!("only-in-{spelling}"))).expect("entry"), spelling);
        }
    }
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}
//endregion
