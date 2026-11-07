//! Controls for the Linux platform's primitives on a btrfs fixture:
//! they are the kernel's answers,
//! not constants.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::remove;

/// A subvolume made through the platform is a subvolume root,
/// a directory beside it is not,
/// and a path that is not there is
///  an error;
/// the platform reports what the kernel did,
/// so a platform that always said yes or no would fail here.
#[test]
fn btrfs_the_platform_creates_a_subvolume_and_tells_it_from_a_directory() {
    let Some(root) = btrfs_fixture("platform-subvolume") else { return };
    let platform: LinuxPlatform = LinuxPlatform::default();
    std::fs::create_dir(root.join("plain")).expect("directory");
    assert_eq!(platform.is_subvolume_root(root.join("plain").as_path()), Ok(false));
    assert!(!root.join("made").exists());
    platform.create_subvolume(root.join("made").as_path()).expect("create");
    assert!(root.join("made").is_dir(), "the subvolume exists");
    assert_eq!(platform.is_subvolume_root(root.join("made").as_path()), Ok(true));
    assert!(platform.create_subvolume(root.join("made").as_path()).is_err(), "a second creation is refused by the kernel");
    assert!(platform.is_subvolume_root(root.join("absent").as_path()).is_err());
    remove(root.as_path());
}

/// The platform returns the kernel's error for a path that is not there,
/// on any filesystem:
/// asking whether it is a subvolume
///  root is an error,
/// and creating a subvolume below a parent that does not exist is an error that creates nothing.
/// A platform
///  that answered `Ok` to either would fail here without a btrfs fixture.
#[test]
fn the_platform_reports_the_error_for_a_path_that_is_not_there() {
    let root = crate::test_support::fixture("platform-absent");
    let platform: LinuxPlatform = LinuxPlatform::default();
    assert!(platform.is_subvolume_root(root.join("absent").as_path()).is_err());
    let created = platform.create_subvolume(root.join("absent/below").as_path());
    assert!(created.is_err(), "{created:?}");
    assert!(!root.join("absent").exists(), "nothing was created");
    remove(root.as_path());
}

/// A directory made the ordinary way is not a subvolume root,
/// on any filesystem;
/// a platform that answered `true` would fail
///  here,
/// and the creation of a subvolume where the filesystem has none is refused by the kernel and leaves nothing behind.
#[test]
fn an_ordinary_directory_is_no_subvolume_root_whatever_the_filesystem() {
    let root = crate::test_support::fixture("platform-ordinary");
    let platform: LinuxPlatform = LinuxPlatform::default();
    std::fs::create_dir(root.join("plain")).expect("directory");
    assert_eq!(platform.is_subvolume_root(root.join("plain").as_path()), Ok(false));
    std::fs::write(root.join("file"), b"x").expect("file");
    assert_eq!(platform.is_subvolume_root(root.join("file").as_path()), Ok(false), "a file is not a subvolume root either");
    let existing = platform.create_subvolume(root.join("plain").as_path());
    assert!(existing.is_err(), "a name that is taken is refused: {existing:?}");
    assert!(root.join("plain").is_dir(), "the directory is untouched");
    remove(root.as_path());
}

/// The variables of a disposable home:
/// the cache of the compiled-in paths goes inside the fixture,
/// and the process
///  environment (which has no forward-target variable) is not consulted.
fn disposable_variables(root: &Path) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    return vec![
        (std::ffi::OsString::from("HOME"), root.join("home").into_os_string()),
        (std::ffi::OsString::from("XDG_CACHE_HOME"), root.join("cache").into_os_string()),
    ];
}

/// A platform that names the real Git learns where that Git reads its system configuration and attributes files from,
/// though
///  neither the process environment nor the variables it was given carry the forward-target variable;
/// a platform that names
///  none cannot know,
/// so the content decision stays on the refusing side.
#[test]
fn the_platform_learns_the_system_files_from_the_git_it_forwards_to() {
    let root = crate::test_support::fixture("platform-names-git");
    let variables = disposable_variables(root.as_path());
    assert!(!variables.iter().any(|(name, _value)| { return name == crate::child_environment::FORWARD_TARGET_VARIABLE; }));
    assert!(std::env::var_os(crate::child_environment::FORWARD_TARGET_VARIABLE).is_none(), "the control needs a process without the variable");
    let knowing: ConversionEnvironment = LinuxPlatform::forwarding_to(variables.as_slice(), Some(Path::new(crate::test_support::REAL_GIT))).conversion_environment();
    assert!(!knowing.scope.unresolvable, "the system configuration file is learned from the Git named");
    assert!(knowing.scope.system_default.is_some_and(|path| { return path.is_absolute(); }));
    assert!(matches!(&knowing.system_attributes, crate::worktree_clean_content::SystemAttributes::File(path) if path.is_absolute()), "{:?}", knowing.system_attributes);
    let unaware: ConversionEnvironment = LinuxPlatform::forwarding_to(variables.as_slice(), None).conversion_environment();
    assert!(unaware.scope.unresolvable && unaware.scope.system_default.is_none());
    assert_eq!(unaware.system_attributes, crate::worktree_clean_content::SystemAttributes::Unknown);
    let default: ConversionEnvironment = LinuxPlatform::default().conversion_environment();
    assert!(default.scope.unresolvable && default.variables.is_empty());
    remove(root.as_path());
}

/// A variable the caller already set to name a Git wins over the one resolved,
/// as it does for every other reader,
/// and the
///  environment the facts were read from is the one the configuration listing reads its entries from.
#[test]
fn the_environment_the_facts_come_from_is_the_one_kept() {
    let root = crate::test_support::fixture("platform-keeps-environment");
    let mut variables = disposable_variables(root.as_path());
    variables.push((std::ffi::OsString::from("GIT_CONFIG_COUNT"), std::ffi::OsString::from("0")));
    let environment: ConversionEnvironment = LinuxPlatform::forwarding_to(variables.as_slice(), Some(Path::new(crate::test_support::REAL_GIT))).conversion_environment();
    assert!(environment.variables.iter().any(|(name, value)| { return name == "GIT_CONFIG_COUNT" && value == "0"; }));
    assert_eq!(environment.variables.iter().filter(|(name, _value)| { return name == crate::child_environment::FORWARD_TARGET_VARIABLE; }).count(), 1);
    let named_already: Vec<(std::ffi::OsString, std::ffi::OsString)> = [variables, vec![(std::ffi::OsString::from(crate::child_environment::FORWARD_TARGET_VARIABLE), std::ffi::OsString::from("/named/by/the/caller"))]].concat();
    let kept: ConversionEnvironment = LinuxPlatform::forwarding_to(named_already.as_slice(), Some(Path::new(crate::test_support::REAL_GIT))).conversion_environment();
    assert!(kept.variables.iter().any(|(name, value)| { return name == crate::child_environment::FORWARD_TARGET_VARIABLE && value == "/named/by/the/caller"; }));
    remove(root.as_path());
}
