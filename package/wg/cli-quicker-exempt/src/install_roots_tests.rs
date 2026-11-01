//! Verifies that a link at an install root counts only when it points strictly inside the home,
//! with allowed and rejected fixtures on a disposable tree.
//! The rejected cases are the ones that would make every program of a user,
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

/// A link to a directory strictly inside the home is accepted,
/// and executables under what it points to match.
#[test]
fn a_link_to_a_place_inside_the_home_is_accepted_and_matches() {
    let tree = tree("inside");
    let target = tree.home.join("tools/real");
    std::fs::create_dir_all(target.join("node/bin")).expect("make the target");
    link_toolchain(&tree, &target);
    let roots = roots_of(&tree);
    let resolved = std::fs::canonicalize(&target).expect("resolve the target");
    assert!(roots.roots.contains(&resolved), "{roots:?}");
    assert!(roots.refused.is_empty(), "{roots:?}");
    assert_eq!(classify_exe(&roots, &resolved.join("node/bin/node")), Some(MatchReason::ToolchainPath));
    assert_eq!(classify_exe(&roots, Path::new("/usr/bin/curl")), None);
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
    assert_eq!(roots.refused.len(), 1, "{roots:?}");
    assert!(roots.refused[0].why.contains("not strictly inside the home"), "{roots:?}");
    assert_eq!(roots.refused[0].root.file_name().and_then(|name| return name.to_str()), Some("toolchain"));
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
    assert_eq!(roots.refused.len(), 1, "{roots:?}");
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
    assert_eq!(roots.refused.len(), 1, "{roots:?}");
    assert_eq!(classify_exe(&roots, &other.join(".local/share/monochromatic-dev/toolchain/bin/x")), None);
}

/// A link loop cannot be resolved and is refused loudly,
/// not skipped as if the root did not exist.
#[test]
fn a_link_loop_is_refused_loudly() {
    let tree = tree("loop");
    std::os::unix::fs::symlink(tree.data.join("toolchain-global"), tree.data.join("toolchain")).expect("link one way");
    std::os::unix::fs::symlink(tree.data.join("toolchain"), tree.data.join("toolchain-global")).expect("link the other way");
    let roots = roots_of(&tree);
    assert_eq!(roots.refused.len(), 2, "{roots:?}");
    assert!(roots.refused.iter().all(|entry| return entry.why.contains("cannot be resolved")), "{roots:?}");
}

/// A link to a place that does not exist is the same as a root that does not exist yet.
#[test]
fn a_dangling_link_is_ignored_without_a_report() {
    let tree = tree("dangling");
    link_toolchain(&tree, &tree.home.join("not-there"));
    let roots = roots_of(&tree);
    assert!(roots.refused.is_empty(), "{roots:?}");
}

/// A home that resolves to the filesystem root has everything below it,
/// so no link is accepted.
#[test]
fn a_home_that_resolves_to_the_filesystem_root_accepts_no_link() {
    let tree = tree("home-is-root");
    let target = tree.home.join("tools");
    std::fs::create_dir_all(&target).expect("make the target");
    link_toolchain(&tree, &target);
    let roots = install_roots(&tree.home, Some(Path::new("/")));
    assert_eq!(roots.refused.len(), 1, "{roots:?}");
    assert!(!roots.roots.contains(&std::fs::canonicalize(&target).expect("resolve the target")), "{roots:?}");
}

/// Without a resolved home a link cannot be checked and is refused.
#[test]
fn without_a_resolved_home_no_link_is_followed() {
    let tree = tree("no-home");
    let target = tree.home.join("tools");
    std::fs::create_dir_all(&target).expect("make the target");
    link_toolchain(&tree, &target);
    let roots = install_roots(&tree.home, None);
    assert_eq!(roots.refused.len(), 1, "{roots:?}");
    assert!(roots.refused[0].why.contains("cannot be checked"), "{roots:?}");
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
