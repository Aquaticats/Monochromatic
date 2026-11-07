//! Controls for the move of an earlier folder's entries into the current folder,
//! each in its own disposable folder.

use super::*;
use crate::earlier_log_lines_added_to_current_log::NeverStopped;
use crate::wrapper_names_old_and_new::DATA_FOLDER;
use std::path::PathBuf;

/// One empty fixture folder for a control.
fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("native-folder-fold", name);
}

/// A stop that ends the fold when it has been told of `limit` steps,
/// as a kill at that instant would.
struct StopAfter {
    /// How many steps may pass.
    limit: usize,
    /// How many steps have been told.
    seen: usize,
}

impl FoldSteps for StopAfter {
    fn may_continue(&mut self, _step: &'static str) -> bool {
        self.seen += 1;
        return self.seen <= self.limit;
    }
}

/// Write a file,
/// making its folders.
fn put(root: &Path, relative: &str, bytes: &str) {
    let path: PathBuf = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("folders");
    std::fs::write(path, bytes).expect("file");
}

/// Plant the shape measured in this repository:
/// both folders hold `work/done` with different records,
/// a file of the same name,
///  and a folder that only the earlier one holds.
fn plant(root: &Path) {
    put(root, "cli-git/work/done/earlier-record", "earlier record");
    put(root, "cli-git/work/pending/.keep", "");
    put(root, "cli-git/standards-review-shadow.md", "earlier shadow");
    put(root, "cli-git/only-earlier/inside/deep", "deep");
    put(root, "monochromatic-git-policy-cli/work/done/current-record", "current record");
    put(root, "monochromatic-git-policy-cli/standards-review-shadow.md", "current shadow");
}

/// The entries of both folders end up below the current folder:
/// a record only one holds moves in,
/// a folder both hold is merged,
///  a file both hold is kept twice,
/// and the earlier folder is gone.
#[test]
fn every_entry_of_both_folders_is_below_the_current_folder_and_the_earlier_folder_is_gone() {
    let root: PathBuf = fixture("merge");
    plant(root.as_path());
    let done: FolderFold = fold_folder(root.as_path(), DATA_FOLDER, &mut NeverStopped).expect("fold");
    assert!(matches!(done, FolderFold::Folded { .. }), "{done:?}");
    let current: PathBuf = root.join(DATA_FOLDER.current);
    assert_eq!(std::fs::read_to_string(current.join("work/done/earlier-record")).expect("moved"), "earlier record");
    assert_eq!(std::fs::read_to_string(current.join("work/done/current-record")).expect("kept"), "current record");
    assert_eq!(std::fs::read_to_string(current.join("only-earlier/inside/deep")).expect("moved folder"), "deep");
    assert!(current.join("work/pending/.keep").exists());
    assert_eq!(std::fs::read_to_string(current.join("standards-review-shadow.md")).expect("current"), "current shadow");
    let beside: String = format!("standards-review-shadow.md{KEPT_BESIDE_SUFFIX}");
    assert_eq!(std::fs::read_to_string(current.join(beside.as_str())).expect("earlier kept beside"), "earlier shadow");
    assert!(!root.join(DATA_FOLDER.earlier).exists(), "the earlier folder is gone");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// Both names hold the same folder entry name as a file:
/// both are kept,
/// the current one under its own name.
#[test]
fn an_entry_name_that_both_folders_hold_is_kept_twice() {
    let root: PathBuf = fixture("same-entry");
    put(root.as_path(), "cli-git/same", "from the earlier folder");
    put(root.as_path(), "monochromatic-git-policy-cli/same", "from the current folder");
    fold_folder(root.as_path(), DATA_FOLDER, &mut NeverStopped).expect("fold");
    let current: PathBuf = root.join(DATA_FOLDER.current);
    assert_eq!(std::fs::read_to_string(current.join("same")).expect("current"), "from the current folder");
    assert_eq!(std::fs::read_to_string(current.join(format!("same{KEPT_BESIDE_SUFFIX}"))).expect("earlier"), "from the earlier folder");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// The name with the suffix is taken too:
/// the entry stays in the earlier folder,
/// the fold says so,
/// and nothing is replaced.
#[test]
fn an_entry_whose_second_name_is_taken_stays_and_is_reported() {
    let root: PathBuf = fixture("second-name-taken");
    put(root.as_path(), "cli-git/same", "earlier");
    put(root.as_path(), "monochromatic-git-policy-cli/same", "current");
    put(root.as_path(), format!("monochromatic-git-policy-cli/same{KEPT_BESIDE_SUFFIX}").as_str(), "older fold");
    let result: FolderFold = fold_folder(root.as_path(), DATA_FOLDER, &mut NeverStopped).expect("fold");
    let FolderFold::Left { kept } = result else { panic!("expected Left, got {result:?}") };
    assert_eq!(kept, vec![root.join("cli-git/same")]);
    assert_eq!(std::fs::read_to_string(root.join("cli-git/same")).expect("earlier"), "earlier");
    assert_eq!(std::fs::read_to_string(root.join("monochromatic-git-policy-cli/same")).expect("current"), "current");
    assert_eq!(std::fs::read_to_string(root.join(format!("monochromatic-git-policy-cli/same{KEPT_BESIDE_SUFFIX}"))).expect("older"), "older fold");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A kill at every step,
/// then one more fold:
/// every file of both folders exists once,
/// and no earlier folder is left.
#[test]
fn a_kill_at_any_step_then_a_second_fold_leaves_every_file_once() {
    let mut limit: usize = 0;
    loop {
        let root: PathBuf = fixture(format!("kill-{limit}").as_str());
        plant(root.as_path());
        let first: FolderFold = fold_folder(root.as_path(), DATA_FOLDER, &mut StopAfter { limit, seen: 0 }).expect("first");
        let finished: bool = matches!(first, FolderFold::Folded { .. });
        let second: FolderFold = fold_folder(root.as_path(), DATA_FOLDER, &mut NeverStopped).expect("second");
        assert!(matches!(second, FolderFold::Folded { .. } | FolderFold::Nothing), "kill after {limit} steps: {second:?}");
        let current: PathBuf = root.join(DATA_FOLDER.current);
        for (relative, bytes) in [
            ("work/done/earlier-record", "earlier record"),
            ("work/done/current-record", "current record"),
            ("only-earlier/inside/deep", "deep"),
            ("standards-review-shadow.md", "current shadow"),
            (format!("standards-review-shadow.md{KEPT_BESIDE_SUFFIX}").as_str(), "earlier shadow"),
        ] {
            assert_eq!(std::fs::read_to_string(current.join(relative)).unwrap_or_default(), bytes, "kill after {limit} steps: {relative}");
        }
        assert!(!root.join(DATA_FOLDER.earlier).exists(), "kill after {limit} steps: the earlier folder is gone");
        std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
        if finished {
            assert!(limit > 4, "the fold has several steps to stop at: {limit}");
            break;
        }
        limit += 1;
    }
}

/// A link in the earlier folder moves as a link and is never followed.
#[cfg(unix)]
#[test]
fn a_link_moves_as_a_link() {
    let root: PathBuf = fixture("link");
    put(root.as_path(), "cli-git/real", "target");
    std::os::unix::fs::symlink("real", root.join("cli-git/pointer")).expect("link");
    std::fs::create_dir_all(root.join(DATA_FOLDER.current)).expect("current folder");
    fold_folder(root.as_path(), DATA_FOLDER, &mut NeverStopped).expect("fold");
    let moved: PathBuf = root.join(DATA_FOLDER.current).join("pointer");
    assert!(std::fs::symlink_metadata(moved.as_path()).expect("moved").file_type().is_symlink());
    assert_eq!(std::fs::read_link(moved.as_path()).expect("target"), PathBuf::from("real"));
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// A Git directory that cannot be written:
/// the fold fails and every entry stays in the earlier folder.
#[cfg(unix)]
#[test]
fn a_read_only_folder_fails_the_fold_and_keeps_every_entry() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("read-only");
    plant(root.as_path());
    let current: PathBuf = root.join(DATA_FOLDER.current);
    std::fs::set_permissions(current.as_path(), std::fs::Permissions::from_mode(0o500)).expect("read-only");
    let probe = std::fs::File::create(current.join("probe"));
    let result = fold_folder(root.as_path(), DATA_FOLDER, &mut NeverStopped);
    std::fs::set_permissions(current.as_path(), std::fs::Permissions::from_mode(0o700)).expect("writable again");
    if probe.is_ok() {
        // The control: this user can write there anyway, so the case cannot be made.
        std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
        return;
    }
    assert!(result.is_err(), "{result:?}");
    assert_eq!(std::fs::read_to_string(root.join("cli-git/work/done/earlier-record")).expect("kept"), "earlier record");
    assert_eq!(std::fs::read_to_string(root.join("cli-git/only-earlier/inside/deep")).expect("kept"), "deep");
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// An earlier folder with nothing in it is removed,
/// and a name that does not exist is nothing.
#[test]
fn an_empty_earlier_folder_is_removed_and_a_missing_one_is_nothing() {
    let root: PathBuf = fixture("empty");
    std::fs::create_dir_all(root.join("cli-git/v1")).expect("empty tree");
    std::fs::create_dir_all(root.join(DATA_FOLDER.current).join("v1")).expect("current tree");
    assert!(matches!(fold_folder(root.as_path(), DATA_FOLDER, &mut NeverStopped).expect("fold"), FolderFold::Folded { .. }));
    assert!(!root.join("cli-git").exists());
    assert_eq!(fold_folder(root.as_path(), DATA_FOLDER, &mut NeverStopped).expect("again"), FolderFold::Nothing);
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}
