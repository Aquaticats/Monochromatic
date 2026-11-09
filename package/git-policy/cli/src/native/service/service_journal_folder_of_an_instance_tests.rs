//! Controls for the one spelling of a worktree's journal folder:
//! the unit names it,
//! and a journal folder read back names the state folder that holds it.

use super::*;

/// What:
///  The folder the unit gives an instance is the folder the wrapper's state folder holds.
/// Why:
///   The unit's `--journal`,
/// the sweep's inverse and the client's look must agree,
/// or the look finds no journal and reports nothing.
#[test]
fn the_journal_folder_is_the_one_the_unit_names() {
    let wrapper_state: PathBuf = PathBuf::from("/home/a/.local/state/monochromatic-git-policy-cli");
    let instance: &str = "3f9a3f9a3f9a3f9a3f9a3f9a3f9a3f9a";
    let journal: PathBuf = journal_of(wrapper_state.as_path(), instance);
    assert_eq!(journal, wrapper_state.join(JOURNAL_FOLDER).join(instance), "the unit's --journal names this folder");
    assert_eq!(state_dir_of_journal(journal.as_path(), instance), Some(PathBuf::from("/home/a/.local/state")), "and it reads back as the state folder");
}

/// What:
///  A journal folder of another shape names no state folder,
///  and neither does one of another instance.
/// Why:
///   A development run names any folder,
/// and reading a state folder out of it would let the sweep act on a place no unit names.
#[test]
fn a_journal_folder_of_another_shape_names_no_state_folder() {
    let instance: &str = "3f9a3f9a3f9a3f9a3f9a3f9a3f9a3f9a";
    let other: &str = "0d1e0d1e0d1e0d1e0d1e0d1e0d1e0d1e";
    let elsewhere: PathBuf = PathBuf::from("/tmp/somewhere/service").join(instance);
    let another_instance: PathBuf = journal_of(PathBuf::from("/home/a/.local/state/monochromatic-git-policy-cli").as_path(), other);
    assert_eq!(state_dir_of_journal(elsewhere.as_path(), instance), None, "a development run's folder names no state folder");
    assert_eq!(state_dir_of_journal(another_instance.as_path(), instance), None, "another instance's journal is not this one's");
}
