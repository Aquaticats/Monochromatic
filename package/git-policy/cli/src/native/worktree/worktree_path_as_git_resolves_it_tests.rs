//! Controls for Git's own resolution of a path and its own comparison of two paths:
//! the last component that is not there,
//! the components that name no directory of their own,
//! the link that is followed,
//! and the case the setting folds.
//!
//! Each control states a rule of `real_pathdup` in Git's `abspath.c` or of `fspathcmp` in its `dir.c`,
//! because the guard names the worktree Git removes only when it resolves the name Git resolves.

use super::*;
use crate::test_support::{fixture, remove};

/// What:
///  Everything below `root`, spelled in the fixture's own directory.
/// Why:
///   The fixture root is where a control makes its directories and links,
///  and a path written by hand would name another machine's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const under = (root: string, rest: string) => `${root}/${rest}`;
/// ```
fn under(root: &Path, rest: &str) -> PathBuf {
    return root.join(rest);
}

/// What:
///  A last component that is not there is kept as it was spelled,
///  and only a last component may be missing.
/// Why:
///   Git allows exactly one missing component, at the end (`REALPATH_MANY_MISSING` is not set for
///  `real_pathdup`),
///  and that is the rule that made a case-differing name a worktree Git removes while this program's
///  own resolution named nothing (a review of `e615fb599b`, finding 1).
///  A separator after the missing component fails instead, which is why Git refuses the same name
///  with a trailing slash.
#[test]
fn a_last_component_that_is_not_there_is_kept_as_it_was_spelled() {
    let root: PathBuf = fixture("realpath-missing-last");
    std::fs::create_dir_all(under(&root, "there/nested").as_path()).expect("a directory");
    assert_eq!(git_real_path(under(&root, "ABSENT").as_path()), Some(under(&root, "ABSENT")), "the last component is kept");
    assert_eq!(
        git_real_path(under(&root, "./Absent").as_path()),
        Some(under(&root, "Absent")),
        "and a `.` before it names no component of its own"
    );
    assert_eq!(git_real_path(under(&root, "absent/child").as_path()), None, "a component before the last that is missing");
    assert_eq!(git_real_path(under(&root, "ABSENT/").as_path()), None, "a separator after the missing component");
    assert_eq!(git_real_path(under(&root, "there/").as_path()), Some(under(&root, "there")), "a separator after a directory");
    assert_eq!(git_real_path(under(&root, "there/nested").as_path()), Some(under(&root, "there/nested")), "a directory there");
    remove(root.as_path());
}

/// What:
///  A `.` names no component,
///  a `..` takes the last component back,
///  and a path may not be taken back past its root.
/// Why:
///   Git resolves both of these on the bytes it has already resolved (`strbuf_realpath_1` in `abspath.c`)
///  instead of asking the kernel,
///  so a `..` after a link leaves the directory the link stood in.
#[test]
fn a_dot_names_no_component_and_two_dots_take_one_back() {
    let root: PathBuf = fixture("realpath-dot-and-dotdot");
    std::fs::create_dir_all(under(&root, "here/there").as_path()).expect("a directory");
    assert_eq!(git_real_path(under(&root, "here/./there").as_path()), Some(under(&root, "here/there")), "a `.`");
    assert_eq!(git_real_path(under(&root, "here/there/..").as_path()), Some(under(&root, "here")), "a `..`");
    assert_eq!(git_real_path(Path::new("/../..")), Some(PathBuf::from("/")), "`..` at the root leaves the root");
    assert_eq!(
        git_real_path(under(&root, "here/there/.././absent").as_path()),
        Some(under(&root, "here/absent")),
        "both together, with the last component kept"
    );
    remove(root.as_path());
}

/// What:
///  A link is followed where Git follows it,
///  and a `..` after the link leaves the directory the link names.
/// Why:
///   Git replaces the link's own component with the bytes it reads from the link,
///  resolved from the root when the link is absolute and from the link's directory when it is relative,
///  and the difference decides a prefix a person spelled through `/home/user` while the registration
///  is canonical under `/var/mnt/encrypted`.
#[test]
fn a_link_is_followed_and_two_dots_after_it_leave_where_it_names() {
    let root: PathBuf = fixture("realpath-links");
    std::fs::create_dir_all(under(&root, "deep/nested").as_path()).expect("a directory");
    std::fs::create_dir_all(under(&root, "beside").as_path()).expect("a directory");
    std::os::unix::fs::symlink(under(&root, "deep/nested").as_path(), under(&root, "relative-link").as_path()).expect("a relative link");
    std::os::unix::fs::symlink(under(&root, "beside").as_path(), under(&root, "absolute-link").as_path()).expect("an absolute link");
    // A link whose own name differs in case from the name Git resolves it to: the shape a review of `e615fb599b` reproduced.
    std::os::unix::fs::symlink(under(&root, "deep/nested").as_path(), under(&root, "Link-Through-It").as_path()).expect("a link to follow");
    assert_eq!(
        git_real_path(under(&root, "relative-link").as_path()),
        Some(under(&root, "deep/nested")),
        "a relative link names what it says"
    );
    assert_eq!(
        git_real_path(under(&root, "absolute-link/beside-nothing").as_path()),
        Some(under(&root, "beside/beside-nothing")),
        "an absolute link starts again at the root it names"
    );
    assert_eq!(
        git_real_path(under(&root, "Link-Through-It/../beside").as_path()),
        Some(under(&root, "deep/beside")),
        "a `..` after a link takes back the last component the link named, not the link's own name"
    );
    assert_eq!(
        git_real_path(under(&root, "relative-link/ABSENT").as_path()),
        Some(under(&root, "deep/nested/ABSENT")),
        "and a last component after a link is kept as spelled"
    );
    remove(root.as_path());
}

/// What:
///  A link whose target is not there resolves to that target's own spelling,
///  and a path of links that never ends resolves to nothing.
/// Why:
///   Git reads a link and then keeps a last component it cannot look at,
///  so a dangling link resolves where `realpath(3)` fails,
///  and Git refuses a path of more than thirty-two links.
#[test]
fn a_dangling_link_resolves_to_its_target_and_a_loop_resolves_to_nothing() {
    let root: PathBuf = fixture("realpath-dangling-and-loop");
    std::fs::create_dir_all(root.as_path()).expect("a directory");
    std::os::unix::fs::symlink("nowhere", under(&root, "dangling").as_path()).expect("a dangling link");
    std::os::unix::fs::symlink("loop", under(&root, "loop").as_path()).expect("a loop");
    assert_eq!(git_real_path(under(&root, "dangling").as_path()), Some(root.join("nowhere")), "the target, kept as spelled");
    assert_eq!(git_real_path(under(&root, "loop").as_path()), None, "a loop");
    assert_eq!(git_real_path(Path::new("")), None, "no path at all");
    remove(root.as_path());
}

/// What:
///  The comparison is over the bytes,
///  and over ASCII letters only when the repository folds case.
/// Why:
///   Git's `fspathcmp` is `strcasecmp` exactly when `core.ignorecase` is set,
///  and neither setting folds a letter outside ASCII,
///  so a name that differs in `Ä` names another worktree under either setting.
#[test]
fn the_comparison_folds_ascii_case_exactly_when_the_setting_says_so() {
    assert!(the_same_path(Path::new("/a/topic"), Path::new("/a/topic"), false), "the same bytes");
    assert!(!the_same_path(Path::new("/a/topic"), Path::new("/a/Topic"), false), "case differs, no folding");
    assert!(the_same_path(Path::new("/a/topic"), Path::new("/a/Topic"), true), "case differs, folding");
    assert!(!the_same_path(Path::new("/a/topic"), Path::new("/a/topic/"), true), "a separator is not case");
    assert!(!the_same_path(Path::new("/a/Ä"), Path::new("/a/ä"), true), "a letter outside ASCII is not folded");
}
