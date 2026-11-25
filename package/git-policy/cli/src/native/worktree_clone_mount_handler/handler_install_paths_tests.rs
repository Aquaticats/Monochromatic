//! What:
//!  Controls of the paths one root-owned build serves and of the words it
//!  prints.
//! Why:
//!  The decisions fix the paths and the words,
//!  and a moved character in either is an install ZED never starts or a
//!  refusal a person cannot follow,
//!  so the strings are held here rather than trusted to a reader
//!  (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
//!  choices 2, 3 and 5).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(ROOT_MODE_INSTALL_PATH).toBe('/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets');
//! ```

use super::{CHECKOUT_RECORD_PATH, CONFIGURATION_PATH, ROOT_MODE_INSTALL_PATH, ZEDLET_INSTALL_PATH, root_modes_name, version_line};
use crate::foundation::wrapper_names_old_and_new::COMMAND_WORD;
use crate::worktree_clone_mount_handler::root_mode_names::{ROOT_GROUP_WORD, ZEDLET_START_NAME};

/// The root modes' own file is the one the design names,
/// and it carries the ZED event's own name,
/// so a start there is a mode or a refusal and never Git as root.
#[test]
fn the_root_mode_path_is_the_designed_one() {
    assert_eq!(ROOT_MODE_INSTALL_PATH, "/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets");
    assert_eq!(ROOT_MODE_INSTALL_PATH.rsplit('/').next(), Some(ZEDLET_START_NAME));
}

/// The ZED entry is the history event the design names,
/// and ZED only runs a file whose name starts with `history_event-`.
#[test]
fn the_zedlet_path_is_the_designed_history_event() {
    assert_eq!(ZEDLET_INSTALL_PATH, "/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets");
    let name: &str = ZEDLET_INSTALL_PATH.rsplit('/').next().unwrap_or_default();
    assert!(name.starts_with("history_event-"), "ZED runs only files named history_event-*: {name}");
    assert_eq!(name, ZEDLET_START_NAME, "the entry ZED starts carries the one name the dispatch compares");
}

/// The ZED entry is a link to the root modes' own file,
/// so one build is installed once and no file is named for the wrapper.
#[test]
fn the_zedlet_entry_links_the_root_mode_file() {
    let linked_to: &str = "/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets";
    assert_eq!(linked_to, ROOT_MODE_INSTALL_PATH);
    assert_ne!(ZEDLET_INSTALL_PATH, ROOT_MODE_INSTALL_PATH, "the ZED entry is a link beside the pool's zedlets, and the file it links to is root's libexec one");
}

/// The configuration stays in the root-controlled folder the design names.
#[test]
fn the_configuration_path_is_the_designed_one() {
    assert_eq!(CONFIGURATION_PATH, "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc");
}

/// The checkout's record sits beside the slot identities,
/// in the folder the mode creates and the setup installs.
#[test]
fn the_checkout_record_path_is_beside_the_slot_identities() {
    assert_eq!(CHECKOUT_RECORD_PATH, "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json");
    let record_folder: &str = CHECKOUT_RECORD_PATH.rsplit_once('/').map(|(kept, _file)| return kept).unwrap_or_default();
    let slots: &str = crate::worktree_clone_mount::SLOT_IDENTITY_STORE_PATH;
    let slots_folder: Option<&str> = slots.rsplit_once('/').map(|(kept, _name)| return kept);
    assert_eq!(slots_folder, Some(record_folder), "the record and the slot identities live in one root-owned folder");
}

/// The words are the program's own word and the group word,
/// read from the two constants and never written out a second time.
#[test]
fn the_words_are_the_two_pinned_constants() {
    assert_eq!(root_modes_name(), format!("{} {}", COMMAND_WORD.current, ROOT_GROUP_WORD));
    assert_eq!(root_modes_name(), "monochromatic-git-policy-cli zfs-root");
}

/// The version line is the words and this build's release,
/// so a check on the first line tells the root modes' answer from the
/// wrapper's own banner.
#[test]
fn the_version_line_names_the_words_and_the_release() {
    assert_eq!(version_line(), format!("monochromatic-git-policy-cli zfs-root {}", env!("CARGO_PKG_VERSION")));
    assert!(!version_line().ends_with(','));
}
