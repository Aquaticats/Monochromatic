//! What:
//!  The names' controls:
//!  the mount unit name a checkout path escapes to,
//!  the gate's own path,
//!  the drop-in folders,
//!  and what a unit line can carry as one word.
//! Why:
//!  each value is measured against `systemd-escape`'s own output,
//!  because a drop-in written under a name systemd does not give the unit is
//!  read by nothing and reports nothing.
//!  The four paths are the ones the design and its guest spell:
//!  this route's checkout,
//!  the guest's dashed path,
//!  a path with a space,
//!  and the root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(mountUnitName('/home/user/Monochromatic')).toBe('home-user-Monochromatic.mount');
//! ```

use crate::checkout_boot_units::boot_unit_names::{
    BOOT_GATE_UNIT_NAME, BOOT_GATE_UNIT_PATH, STOP_FAILURE_DROP_IN_NAME, SYSTEM_UNIT_DIRECTORY, drop_in_path, is_one_unit_name, is_one_word, mount_unit_name, nested_unit_name,
    path_as_one_word, propagation_service_path, unit_path,
};
use std::path::{Path, PathBuf};

#[test]
fn the_mount_unit_name_matches_systemd_escape_for_the_paths_this_route_names() {
    assert_eq!(mount_unit_name(Path::new("/home/user/Monochromatic")), "home-user-Monochromatic.mount");
    assert_eq!(
        mount_unit_name(Path::new("/srv/zfs-step-19-fixture/home/account/Monochromatic")),
        r"srv-zfs\x2dstep\x2d19\x2dfixture-home-account-Monochromatic.mount"
    );
    assert_eq!(mount_unit_name(Path::new("/home/user/Mono chrome")), r"home-user-Mono\x20chrome.mount");
    assert_eq!(mount_unit_name(Path::new("/")), "-.mount");
}

#[test]
fn a_trailing_slash_and_a_doubled_slash_change_no_name() {
    assert_eq!(mount_unit_name(Path::new("//home//user//Monochromatic//")), mount_unit_name(Path::new("/home/user/Monochromatic")));
    assert_eq!(mount_unit_name(Path::new("/home/user/mono-chrome")), r"home-user-mono\x2dchrome.mount", "the separator's own byte is escaped");
}

#[test]
fn the_gates_path_is_the_one_the_names_agree_on() {
    assert_eq!(unit_path(BOOT_GATE_UNIT_NAME), PathBuf::from(BOOT_GATE_UNIT_PATH));
    assert_eq!(unit_path(BOOT_GATE_UNIT_NAME), Path::new(SYSTEM_UNIT_DIRECTORY).join(BOOT_GATE_UNIT_NAME));
}

#[test]
fn a_drop_in_lives_in_the_unit_s_own_folder() {
    let path: PathBuf = drop_in_path(BOOT_GATE_UNIT_NAME, STOP_FAILURE_DROP_IN_NAME);
    assert_eq!(path, PathBuf::from("/etc/systemd/system/reconcile-git-worktree-clone-mounts.service.d/zz-checkout-stop-failure-mode.conf"));
    assert_eq!(propagation_service_path(), PathBuf::from("/etc/systemd/system/checkout-propagation.service"));
}

#[test]
fn a_nested_unit_name_carries_the_dataset_s_short_word() {
    assert_eq!(nested_unit_name("output"), "git-worktree-nested-output-mount.service");
}

#[test]
fn a_word_for_a_command_line_admits_no_space_quote_or_backslash() {
    assert!(is_one_word("/home/user/Monochromatic"));
    assert!(!is_one_word(""));
    assert!(!is_one_word("two words"));
    assert!(!is_one_word("a\tb"));
    assert!(!is_one_word("a\"b"));
    assert!(!is_one_word("a'b"));
    assert!(!is_one_word(r"a\b"));
}

#[test]
fn a_unit_name_admits_the_backslash_systemd_escape_writes() {
    assert!(is_one_unit_name(r"zfs-load-key@zfsstep19-home\x2dgit.service"));
    assert!(!is_one_unit_name(""));
    assert!(!is_one_unit_name("two units.service another.service"));
}

#[test]
fn a_path_with_a_space_is_no_unit_word_and_one_without_is() {
    assert_eq!(path_as_one_word(Path::new("/home/user/Monochromatic")), Some("/home/user/Monochromatic"));
    assert_eq!(path_as_one_word(Path::new("/home/user/Mono chrome")), None);
}
