//! Controls for the instant a control may hold the warm request at:
//!  the two files it leaves and waits for are named beside the path the control chose.

use super::*;

/// The answered file and the release file sit beside the path the control named,
/// each with its own suffix,
/// so one control's directory holds both and nothing else.
#[test]
fn the_two_files_of_one_control_sit_beside_its_path() {
    let held: PathBuf = PathBuf::from("/a/control/held");
    assert_eq!(marker(held.as_path(), "answered"), PathBuf::from("/a/control/held.answered"));
    assert_eq!(marker(held.as_path(), "release"), PathBuf::from("/a/control/held.release"));
    let named_like_a_folder: PathBuf = PathBuf::from("/a/control/held/");
    assert_eq!(marker(named_like_a_folder.as_path(), "release"), PathBuf::from("/a/control/held.release"));
}
