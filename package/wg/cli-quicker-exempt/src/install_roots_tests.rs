//! Verifies that no link is followed at an install root,
//! with allowed and rejected fixtures on a disposable tree:
//! a root that is a link is refused wherever it points,
//! a link on the way to a root is refused,
//! and plain directories,
//! roots that do not exist yet,
//! and a home reached through a link are not.
//! The rejected cases include the ones that would make every program of a user,
//! or of someone else,
//! leave the tunnel.

/// The roots and their refusals,
/// and the listing from a handle.
use crate::install_roots::{directory_names_from_handle, install_roots, installed_tool_names, InstallRoots};
/// The matcher that trusts the roots.
use crate::update_entry_points::{classify, MatchReason};
/// Fixture paths.
use std::path::{Path, PathBuf};

/// A disposable tree with a home and another user's home next to it.
struct Tree {
    /// The home as the passwd entry would spell it.
    home: PathBuf,
    /// The resolved home.
    canonical_home: PathBuf,
    /// Another user's home,
    /// beside the first.
    other_home: PathBuf,
    /// The directory that holds `toolchain` and `toolchain-global`.
    data: PathBuf,
}

/// Makes a fresh tree for one test.
fn tree(name: &str) -> Tree {
    let base = crate::test_scratch::scratch_root().join(format!("wg-quicker-roots-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&base);
    let home = base.join("home");
    let other_home = base.join("other-home");
    let data = home.join(".local/share/monochromatic-dev");
    std::fs::create_dir_all(&data).expect("make the data directory");
    std::fs::create_dir_all(other_home.join(".local/share/monochromatic-dev/toolchain/bin")).expect("make the other home");
    let canonical_home = std::fs::canonicalize(&home).expect("resolve the home");
    return Tree { home, canonical_home, other_home, data };
}

/// Links the `toolchain` root of the tree to `target`.
fn link_toolchain(tree: &Tree, target: &Path) {
    std::os::unix::fs::symlink(target, tree.data.join("toolchain")).expect("link the root");
}

/// Computes the roots of the tree the way the watcher does.
fn roots_of(tree: &Tree) -> InstallRoots {
    return install_roots(&tree.home, Some(&tree.canonical_home));
}

/// Classifies an executable path against the accepted roots.
fn classify_exe(roots: &InstallRoots, exe: &Path) -> Option<MatchReason> {
    return classify(exe, &[b"tool".to_vec()], &roots.roots);
}

/// Asserts that exactly the `toolchain` root was left out because it is a link.
fn assert_toolchain_refused_as_a_link(roots: &InstallRoots) {
    assert_eq!(roots.refused.len(), 1, "{roots:?}");
    assert!(roots.refused[0].why.contains("it is a link; links are not followed"), "{roots:?}");
    assert_eq!(roots.refused[0].root.file_name().and_then(|name| return name.to_str()), Some("toolchain"));
}

/// A link to a directory inside the home is refused too:
/// the owner chose not to follow any link,
/// so executables under what it points to do not match,
/// and the report names where the link points.
#[test]
fn a_link_to_a_place_inside_the_home_is_refused_and_nothing_matches_through_it() {
    let tree = tree("inside");
    let target = tree.home.join("tools/real");
    std::fs::create_dir_all(target.join("node/bin")).expect("make the target");
    link_toolchain(&tree, &target);
    let roots = roots_of(&tree);
    let resolved = std::fs::canonicalize(&target).expect("resolve the target");
    assert!(!roots.roots.contains(&resolved), "{roots:?}");
    assert!(!roots.roots.contains(&tree.data.join("toolchain")), "a refused root is not matched under its own name either: {roots:?}");
    assert_toolchain_refused_as_a_link(&roots);
    assert!(roots.refused[0].why.contains("it points to `"), "{roots:?}");
    assert_eq!(classify_exe(&roots, &resolved.join("node/bin/node")), None);
    assert_eq!(classify_exe(&roots, Path::new("/usr/bin/curl")), None);
}

/// A link to the home's data directory,
/// the case that would make everything below it match,
/// is refused like every other link.
#[test]
fn a_link_to_the_data_directory_of_the_home_is_refused() {
    let tree = tree("share");
    let share = tree.home.join(".local/share");
    std::fs::create_dir_all(share.join("something/bin")).expect("make a program directory");
    link_toolchain(&tree, &share);
    let roots = roots_of(&tree);
    assert_toolchain_refused_as_a_link(&roots);
    let resolved_share = std::fs::canonicalize(&share).expect("resolve the share directory");
    assert_eq!(classify_exe(&roots, &resolved_share.join("something/bin/tool")), None);
}

/// A link to the filesystem root is refused,
/// and no program outside the home matches because of it.
/// This is the case that made `/usr/bin/curl` a toolchain program.
#[test]
fn a_link_to_the_filesystem_root_is_refused_and_nothing_matches_through_it() {
    let tree = tree("root");
    link_toolchain(&tree, Path::new("/"));
    let roots = roots_of(&tree);
    assert!(!roots.roots.contains(&PathBuf::from("/")), "{roots:?}");
    assert_toolchain_refused_as_a_link(&roots);
    assert_eq!(classify_exe(&roots, Path::new("/usr/bin/curl")), None);
    assert_eq!(classify_exe(&roots, Path::new("/bin/sh")), None);
}

/// A link to the home itself is refused,
/// because every program run from the home would match.
#[test]
fn a_link_to_the_home_itself_is_refused() {
    let tree = tree("self");
    link_toolchain(&tree, &tree.home);
    let roots = roots_of(&tree);
    assert!(!roots.roots.contains(&tree.canonical_home), "{roots:?}");
    assert_toolchain_refused_as_a_link(&roots);
    assert_eq!(classify_exe(&roots, &tree.canonical_home.join("bin/anything")), None);
}

/// A link to another user's home is refused.
#[test]
fn a_link_to_another_users_home_is_refused() {
    let tree = tree("other");
    link_toolchain(&tree, &tree.other_home);
    let roots = roots_of(&tree);
    let other = std::fs::canonicalize(&tree.other_home).expect("resolve the other home");
    assert!(!roots.roots.contains(&other), "{roots:?}");
    assert_toolchain_refused_as_a_link(&roots);
    assert_eq!(classify_exe(&roots, &other.join(".local/share/monochromatic-dev/toolchain/bin/x")), None);
}

/// A link loop is a link like any other and is refused loudly,
/// not skipped as if the root did not exist.
#[test]
fn a_link_loop_is_refused_loudly() {
    let tree = tree("loop");
    std::os::unix::fs::symlink(tree.data.join("toolchain-global"), tree.data.join("toolchain")).expect("link one way");
    std::os::unix::fs::symlink(tree.data.join("toolchain"), tree.data.join("toolchain-global")).expect("link the other way");
    let roots = roots_of(&tree);
    assert_eq!(roots.refused.len(), 2, "{roots:?}");
    assert!(roots.refused.iter().all(|entry| return entry.why.contains("it is a link; links are not followed")), "{roots:?}");
}

/// A link to a place that does not exist is still a link,
/// and is reported loudly like the others.
#[test]
fn a_dangling_link_is_refused_like_any_other_link() {
    let tree = tree("dangling");
    link_toolchain(&tree, &tree.home.join("not-there"));
    let roots = roots_of(&tree);
    assert_toolchain_refused_as_a_link(&roots);
}

/// A link is refused whatever the home resolves to,
/// and with no resolved home at all:
/// nothing about the answer depends on where the link points.
#[test]
fn a_link_is_refused_whatever_the_home_resolves_to() {
    let tree = tree("any-home");
    let target = tree.home.join("tools");
    std::fs::create_dir_all(&target).expect("make the target");
    link_toolchain(&tree, &target);
    assert_toolchain_refused_as_a_link(&install_roots(&tree.home, Some(Path::new("/"))));
    assert_toolchain_refused_as_a_link(&install_roots(&tree.home, None));
    assert_toolchain_refused_as_a_link(&roots_of(&tree));
}

/// A link on the way to a root is refused too,
/// and the report names the link below the home and not the home's own spelling,
/// so both spellings of the home give one reason.
#[test]
fn a_link_on_the_way_to_a_root_is_refused_and_named() {
    let tree = tree("way");
    let elsewhere = tree.home.join("elsewhere");
    std::fs::create_dir_all(elsewhere.join("toolchain/node/bin")).expect("make a real root elsewhere");
    std::fs::remove_dir_all(&tree.data).expect("remove the data directory");
    std::os::unix::fs::symlink(&elsewhere, &tree.data).expect("link the data directory");
    let roots = roots_of(&tree);
    assert_eq!(roots.refused.len(), 2, "both roots lie behind the link: {roots:?}");
    assert!(roots.refused.iter().all(|entry| return entry.why.contains("`.local/share/monochromatic-dev` on the way to it is a link; links are not followed")), "{roots:?}");
    let resolved = std::fs::canonicalize(&elsewhere).expect("resolve the target");
    assert_eq!(classify_exe(&roots, &resolved.join("toolchain/node/bin/node")), None);
}

/// A home that is itself reached through a link,
/// such as `/home` pointing to `/var/home`,
/// is fine:
/// only what lies below the home is checked,
/// and both spellings of the home are roots.
#[test]
fn a_home_reached_through_a_link_is_not_a_reason_to_refuse() {
    let tree = tree("home-link");
    let spelled = tree.home.parent().expect("the tree has a base").join("home-link");
    std::os::unix::fs::symlink(&tree.home, &spelled).expect("link the home");
    std::fs::create_dir_all(tree.data.join("toolchain/node/bin")).expect("make a real root");
    let roots = install_roots(&spelled, Some(&tree.canonical_home));
    assert!(roots.refused.is_empty(), "{roots:?}");
    assert!(roots.roots.contains(&spelled.join(".local/share/monochromatic-dev/toolchain")), "{roots:?}");
    let exe = tree.canonical_home.join(".local/share/monochromatic-dev/toolchain/node/bin/node");
    assert_eq!(classify_exe(&roots, &exe), Some(MatchReason::ToolchainPath));
}

/// Plain directories and roots that do not exist yet are not refused.
#[test]
fn plain_and_absent_roots_report_nothing() {
    let tree = tree("plain");
    let absent = roots_of(&tree);
    assert!(absent.refused.is_empty(), "{absent:?}");
    std::fs::create_dir_all(tree.data.join("toolchain/node/bin")).expect("make a real root");
    let plain = roots_of(&tree);
    assert!(plain.refused.is_empty(), "{plain:?}");
    let exe = tree.canonical_home.join(".local/share/monochromatic-dev/toolchain/node/bin/node");
    assert_eq!(classify_exe(&plain, &exe), Some(MatchReason::ToolchainPath));
    assert!(install_roots(&tree.home, None).refused.is_empty(), "a plain root needs no resolved home");
}

/// The listing works from the opened handle:
/// a handle inside the home is listed,
/// with only real directories and no dot names,
/// and a handle outside the home is refused even when the caller had meant a path inside it,
/// which is what a path swapped after the check looks like.
#[test]
fn the_listing_checks_and_reads_the_handle_and_not_the_path() {
    let tree = tree("handle");
    let inside = tree.canonical_home.join("tools");
    std::fs::create_dir_all(inside.join("node")).expect("make a tool");
    std::fs::create_dir_all(inside.join(".hidden")).expect("make a dot directory");
    std::fs::write(inside.join("file"), "x").expect("make a file");
    std::os::unix::fs::symlink("/", inside.join("link")).expect("make a link entry");
    let handle = std::fs::File::open(&inside).expect("open the inside directory");
    assert_eq!(directory_names_from_handle(&handle, &tree.canonical_home), Some(vec!["node".to_string()]));
    let outside = std::fs::canonicalize(&tree.other_home).expect("resolve the other home");
    let swapped = std::fs::File::open(&outside).expect("open the directory the path was swapped to");
    assert_eq!(directory_names_from_handle(&swapped, &tree.canonical_home), None);
    let home_itself = std::fs::File::open(&tree.canonical_home).expect("open the home");
    assert_eq!(directory_names_from_handle(&home_itself, &tree.canonical_home), None);
}

/// The notice lists tools from a root inside the home and from nothing else.
#[test]
fn installed_tool_names_lists_only_roots_strictly_inside_the_home() {
    let tree = tree("listing");
    std::fs::create_dir_all(tree.data.join("toolchain/node")).expect("make a tool");
    let roots = roots_of(&tree);
    assert_eq!(installed_tool_names(&roots.roots, &tree.home), vec!["node".to_string()]);
    let outside = std::fs::canonicalize(&tree.other_home).expect("resolve the other home");
    std::fs::create_dir_all(outside.join("secret-tool")).expect("make a tool elsewhere");
    assert!(installed_tool_names(&[outside], &tree.home).is_empty());
    assert!(installed_tool_names(&[PathBuf::from("/")], &tree.home).is_empty());
}

/// The listing never reads through a link,
/// not even one that appeared after the roots were examined:
/// a root that did not exist at the start and became a link to a directory inside the home,
/// and a link on the way to a root,
/// list nothing.
#[test]
fn the_listing_does_not_read_through_a_link_that_appeared_later() {
    let tree = tree("late-link");
    let roots = roots_of(&tree);
    assert!(roots.refused.is_empty(), "{roots:?}");
    let secrets = tree.home.join("tools");
    std::fs::create_dir_all(secrets.join("secret-tool")).expect("make a tool elsewhere in the home");
    link_toolchain(&tree, &secrets);
    assert!(installed_tool_names(&roots.roots, &tree.home).is_empty(), "a root that became a link is not read");
    std::fs::remove_dir_all(&tree.data).expect("remove the data directory");
    let behind = tree.home.join("behind");
    std::fs::create_dir_all(behind.join("toolchain/secret-tool")).expect("make a real root behind a link");
    std::os::unix::fs::symlink(&behind, &tree.data).expect("link the data directory");
    assert!(installed_tool_names(&roots.roots, &tree.home).is_empty(), "a link on the way to a root is not followed");
}
