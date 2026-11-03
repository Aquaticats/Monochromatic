//! The unit tests of the plain-file reader of references and branch configuration,
//! over hand-made repository directories.

use std::fs;
use std::path::{Path, PathBuf};

use crate::git_config::parse_config;
use crate::git_refs::{GitLayout, Head, NativeEnvironment, find_layout, lookup_ref, lookup_short, native_base, read_config, read_head, tracking_ref};
use crate::test_scratch::scratch;

const FIRST: &str = "1111111111111111111111111111111111111111";
const SECOND: &str = "2222222222222222222222222222222222222222";

fn environment(root: &Path) -> NativeEnvironment {
    return NativeEnvironment { home: None, xdg_config_home: None, system_config: root.join("none"), blocking_variable: None };
}

/// A plain repository directory with the given HEAD text and configuration.
fn repository(head: &str, config: &str) -> (tempfile::TempDir, PathBuf) {
    let root = scratch();
    let dot_git: PathBuf = root.path().join(".git");
    fs::create_dir_all(dot_git.join("refs/heads")).expect("refs");
    fs::write(dot_git.join("HEAD"), head).expect("head");
    fs::write(dot_git.join("config"), config).expect("config");
    let path: PathBuf = root.path().to_path_buf();
    return (root, path);
}

fn write_ref(root: &Path, name: &str, object: &str) {
    let path: PathBuf = root.join(".git").join(name);
    fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
    fs::write(path, format!("{object}\n")).expect("ref");
}

#[test]
fn head_is_a_branch_or_a_detached_object_and_anything_else_is_refused() {
    let (root, path) = repository("ref: refs/heads/main\n", "");
    let layout: GitLayout = find_layout(&path, &environment(root.path())).expect("layout");
    assert_eq!(read_head(&layout), Ok(Head::Branch("refs/heads/main".to_string())));
    fs::write(path.join(".git/HEAD"), format!("{FIRST}\n")).expect("detach");
    assert_eq!(read_head(&layout), Ok(Head::Detached(FIRST.to_string())));
    fs::write(path.join(".git/HEAD"), "garbage\n").expect("garbage");
    assert!(read_head(&layout).is_err());
}

#[test]
fn a_loose_reference_wins_over_a_packed_one_and_a_packed_one_is_found_by_name() {
    let (root, path) = repository("ref: refs/heads/main\n", "");
    let layout: GitLayout = find_layout(&path, &environment(root.path())).expect("layout");
    fs::write(path.join(".git/packed-refs"), format!("# pack-refs with: peeled\n{FIRST} refs/heads/main\n^{SECOND}\n{SECOND} refs/heads/other\n")).expect("packed");
    assert_eq!(lookup_ref(&layout, "refs/heads/main"), Ok(Some(FIRST.to_string())));
    assert_eq!(lookup_ref(&layout, "refs/heads/other"), Ok(Some(SECOND.to_string())));
    write_ref(&path, "refs/heads/main", SECOND);
    assert_eq!(lookup_ref(&layout, "refs/heads/main"), Ok(Some(SECOND.to_string())));
    assert_eq!(lookup_ref(&layout, "refs/heads/absent"), Ok(None));
}

#[test]
fn a_symbolic_loose_reference_and_a_name_that_escapes_are_refused() {
    let (root, path) = repository("ref: refs/heads/main\n", "");
    let layout: GitLayout = find_layout(&path, &environment(root.path())).expect("layout");
    fs::write(path.join(".git/refs/heads/alias"), "ref: refs/heads/main\n").expect("symbolic");
    assert!(lookup_ref(&layout, "refs/heads/alias").is_err());
    assert!(lookup_ref(&layout, "refs/../config").is_err());
    assert!(lookup_ref(&layout, "/etc/passwd").is_err());
}

#[test]
fn a_short_name_is_the_remote_tracking_branch_unless_something_shadows_it() {
    let (root, path) = repository("ref: refs/heads/main\n", "");
    let layout: GitLayout = find_layout(&path, &environment(root.path())).expect("layout");
    write_ref(&path, "refs/remotes/origin/main", FIRST);
    assert_eq!(lookup_short(&layout, "origin/main"), Ok(Some(FIRST.to_string())));
    assert_eq!(lookup_short(&layout, "origin/master"), Ok(None));
    write_ref(&path, "refs/tags/origin/main", SECOND);
    assert!(lookup_short(&layout, "origin/main").is_err());
}

#[test]
fn the_upstream_of_a_branch_comes_from_its_remote_merge_and_the_fetch_refspec() {
    let config = parse_config("[remote \"origin\"]\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n[branch \"main\"]\n\tremote = origin\n\tmerge = refs/heads/trunk\n[branch \"local\"]\n\tremote = .\n\tmerge = refs/heads/main\n[branch \"lonely\"]\n\tremote = origin\n").expect("parses");

    assert_eq!(tracking_ref(&config, "main"), Ok(Some("refs/remotes/origin/trunk".to_string())));
    assert_eq!(tracking_ref(&config, "local"), Ok(Some("refs/heads/main".to_string())));
    assert_eq!(tracking_ref(&config, "lonely"), Ok(None), "a remote without a merge reference is no upstream");
    assert_eq!(tracking_ref(&config, "unknown"), Ok(None));
}

#[test]
fn an_exact_refspec_maps_one_reference_and_an_uncovered_reference_has_no_tracking_branch() {
    let config = parse_config("[remote \"origin\"]\n\tfetch = refs/heads/main:refs/remotes/origin/primary\n[branch \"main\"]\n\tremote = origin\n\tmerge = refs/heads/main\n[branch \"side\"]\n\tremote = origin\n\tmerge = refs/heads/side\n").expect("parses");

    assert_eq!(tracking_ref(&config, "main"), Ok(Some("refs/remotes/origin/primary".to_string())));
    assert_eq!(tracking_ref(&config, "side"), Ok(None));
}

#[test]
fn a_merge_value_that_is_not_a_full_reference_and_a_one_sided_pattern_are_refused() {
    let short = parse_config("[remote \"origin\"]\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n[branch \"main\"]\n\tremote = origin\n\tmerge = main\n").expect("parses");
    let lopsided = parse_config("[remote \"origin\"]\n\tfetch = refs/heads/*:refs/remotes/origin/main\n[branch \"main\"]\n\tremote = origin\n\tmerge = refs/heads/main\n").expect("parses");

    assert!(tracking_ref(&short, "main").is_err());
    assert!(tracking_ref(&lopsided, "main").is_err());
}

#[test]
fn a_linked_worktree_finds_its_head_in_its_own_directory_and_the_rest_in_the_common_one() {
    let root = scratch();
    let common: PathBuf = root.path().join("main/.git");
    let own: PathBuf = common.join("worktrees/linked");
    let linked: PathBuf = root.path().join("linked");
    fs::create_dir_all(common.join("refs/heads")).expect("common");
    fs::create_dir_all(&own).expect("own");
    fs::create_dir_all(&linked).expect("linked");
    fs::write(common.join("config"), "").expect("config");
    fs::write(own.join("HEAD"), "ref: refs/heads/topic\n").expect("head");
    fs::write(own.join("commondir"), "../..\n").expect("commondir");
    fs::write(linked.join(".git"), format!("gitdir: {}\n", own.display())).expect("gitfile");
    fs::write(common.join("refs/heads/topic"), format!("{FIRST}\n")).expect("topic");

    let layout: GitLayout = find_layout(&linked, &environment(root.path())).expect("layout");

    assert_eq!(read_head(&layout), Ok(Head::Branch("refs/heads/topic".to_string())));
    assert_eq!(lookup_ref(&layout, "refs/heads/topic"), Ok(Some(FIRST.to_string())));
    assert_eq!(native_base(&linked, true, &environment(root.path())).map(|base| { return base.commit; }), Ok(Ok(FIRST.to_string())));
}

#[test]
fn a_missing_dot_git_a_dot_git_file_without_gitdir_and_a_blocking_variable_are_refused() {
    let root = scratch();
    let mut blocked: NativeEnvironment = environment(root.path());
    assert!(find_layout(root.path(), &blocked).is_err());
    fs::write(root.path().join(".git"), "not a pointer\n").expect("file");
    assert!(find_layout(root.path(), &blocked).is_err());
    blocked.blocking_variable = Some("GIT_DIR".to_string());
    assert!(find_layout(root.path(), &blocked).unwrap_err().contains("GIT_DIR"));
}

#[test]
fn the_configuration_layers_override_in_gits_order_and_an_unreadable_file_is_refused() {
    let (root, path) = repository("ref: refs/heads/main\n", "[core]\n\tx = repository\n");
    let home: PathBuf = root.path().join("home");
    fs::create_dir_all(&home).expect("home");
    fs::write(home.join(".gitconfig"), "[core]\n\tx = user\n\ty = user\n").expect("user");
    let layered: NativeEnvironment = NativeEnvironment { home: Some(home.clone()), xdg_config_home: None, system_config: root.path().join("none"), blocking_variable: None };
    let layout: GitLayout = find_layout(&path, &layered).expect("layout");

    let config = read_config(&layout, &layered).expect("reads");

    assert_eq!(config.last("core", None, "x"), Some("repository"));
    assert_eq!(config.last("core", None, "y"), Some("user"));
    fs::write(home.join(".gitconfig"), "[core\n").expect("broken");
    assert!(read_config(&layout, &layered).is_err());
}

#[test]
fn a_repository_that_asks_for_reftable_or_a_worktree_configuration_is_refused() {
    let (root, path) = repository("ref: refs/heads/main\n", "[extensions]\n\trefStorage = reftable\n");
    let layout: GitLayout = find_layout(&path, &environment(root.path())).expect("layout");
    assert!(read_config(&layout, &environment(root.path())).unwrap_err().contains("reftable"));
    fs::write(path.join(".git/config"), "[extensions]\n\tworktreeConfig = true\n").expect("worktree config");
    assert!(read_config(&layout, &environment(root.path())).unwrap_err().contains("worktree"));
}
