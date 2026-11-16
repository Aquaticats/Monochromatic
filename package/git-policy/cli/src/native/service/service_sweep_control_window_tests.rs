//! Controls for the instant a control may hold the service's last sweep at:
//!  the two files it leaves and waits for are named beside the path the control chose,
//!  and a sweep that carries no variable goes on at once.

use super::*;

/// The asked file and the release file sit beside the path the control named,
/// each with its own suffix,
/// so one control's directory holds both and nothing else.
#[test]
fn the_two_files_of_one_control_sit_beside_its_path() {
    let held: PathBuf = PathBuf::from("/a/control/held");
    assert_eq!(marker(held.as_path(), "asked"), PathBuf::from("/a/control/held.asked"));
    assert_eq!(marker(held.as_path(), "release"), PathBuf::from("/a/control/held.release"));
    let named_like_a_folder: PathBuf = PathBuf::from("/a/control/held/");
    assert_eq!(marker(named_like_a_folder.as_path(), "release"), PathBuf::from("/a/control/held.release"));
}

/// A sweep nobody named holds nothing and leaves no file:
/// the control's variable is the only thing that can park it.
#[test]
fn a_sweep_no_control_armed_goes_on_at_once() {
    let began: Instant = Instant::now();
    the_sweep_waits_where_a_control_says();
    assert!(began.elapsed() < Duration::from_secs(1), "nothing is armed, so the sweep does not wait");
}
