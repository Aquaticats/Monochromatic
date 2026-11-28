//! Controls for the path Git's own reading resolves one operand to.

use super::*;
use crate::test_support::{
    fixture,
    remove,
};

/// What:
///  A path that stands resolves to itself with every link on the way followed,
///  and a missing last component is read off the folder above it.
/// Why:
///  A slot whose folder is gone keeps its registration and its binding record,
///  and the path those name is the one the command means.
#[test]
fn a_path_whose_last_component_is_gone_is_read_from_the_folder_above_it() {
    let root: PathBuf = fixture("operand-path-gone");
    let standing: PathBuf = root.join("standing");
    std::fs::create_dir(standing.as_path()).expect("the folder that stands");
    let canonical: PathBuf = std::fs::canonicalize(standing.as_path()).expect("it resolves");
    assert_eq!(
        path_git_resolves_an_operand_to(standing.as_path()),
        Some(canonical.clone()),
        "a path that stands answers itself"
    );
    let gone: PathBuf = standing.join("gone");
    assert_eq!(
        path_git_resolves_an_operand_to(gone.as_path()),
        Some(canonical.join("gone")),
        "the last component may be missing, as Git allows it"
    );
    assert_eq!(
        path_git_resolves_an_operand_to(standing.join("gone/deeper").as_path()),
        None,
        "only the last component may be missing, and Git refuses the rest"
    );
    remove(root.as_path());
}

/// What:
///  A symbolic link whose target is gone resolves to the path the link names.
/// Why:
///  A person's link to a slot outlives the folder,
///  and Git follows the link and reads the registered path it names,
///  so a removal through the link is the removal of that slot
///  (`abspath.c` at Git `v2.56.1` reads a link with `lstat` and `readlink` before it
///  asks whether the component it named stands).
#[test]
fn a_link_whose_target_is_gone_names_the_path_it_points_at() {
    let root: PathBuf = fixture("operand-path-link");
    let canonical_root: PathBuf =
        std::fs::canonicalize(root.as_path()).expect("the fixture resolves");
    let target: PathBuf = root.join("slot");
    std::fs::create_dir(target.as_path()).expect("the slot");
    let link: PathBuf = root.join("link");
    std::os::unix::fs::symlink(target.as_path(), link.as_path()).expect("the link");
    assert_eq!(
        path_git_resolves_an_operand_to(link.as_path()),
        Some(canonical_root.join("slot")),
        "a link that stands names its standing target"
    );
    std::fs::rename(target.as_path(), root.join("moved-aside")).expect("the slot moves aside");
    assert_eq!(
        path_git_resolves_an_operand_to(link.as_path()),
        Some(canonical_root.join("slot")),
        "a link whose target is gone still names the path the slot had"
    );
    remove(root.as_path());
}

/// What:
///  A `..` after a component that is a link resolves against what the link
///  resolved to,
///  as Git resolves it,
///  and a link that leads back to itself answers nothing.
/// Why:
///  A person types the operand in a folder they stand in,
///  so `..` and links meet in one path,
///  and a reader that resolved `..` textually would name a folder Git never
///  looks at.
#[test]
fn a_dotdot_component_resolves_against_what_the_link_named() {
    let root: PathBuf = fixture("operand-path-dotdot");
    let canonical_root: PathBuf =
        std::fs::canonicalize(root.as_path()).expect("the fixture resolves");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(elsewhere.join("inside").as_path()).expect("the folder a link names");
    let link: PathBuf = root.join("link");
    std::os::unix::fs::symlink(elsewhere.as_path(), link.as_path()).expect("the link");
    assert_eq!(
        path_git_resolves_an_operand_to(root.join("link/../outside").as_path()),
        Some(canonical_root.join("outside")),
        ".. applies to the folder the link resolved to"
    );
    let looped: PathBuf = root.join("ring");
    std::os::unix::fs::symlink(looped.as_path(), looped.as_path()).expect("the loop");
    assert_eq!(
        path_git_resolves_an_operand_to(looped.as_path()),
        None,
        "a link that leads back to itself is refused, as Git refuses it"
    );
    remove(root.as_path());
}
