//! Controls for the discovery-key scan of the configuration scopes:
//!  a key in the file or in any included file is found, a missing file passes, and an include loop is declined.

use super::*;
use crate::test_support::{fixture, remove};

/// A file that sets `core.bare` is found, and so is one that includes such a file by a relative path.
#[test]
fn a_key_is_found_in_the_file_and_in_its_includes() {
    let root = fixture("repo-read-scan-keys");
    std::fs::write(root.join("bare"), "[core]\n\tbare = false\n").expect("bare");
    std::fs::write(root.join("plain"), "[core]\n\teditor = vi\n").expect("plain");
    std::fs::write(root.join("via"), "[include]\n\tpath = bare\n").expect("via");
    assert_eq!(file_avoids_discovery_keys(root.join("bare").as_path(), None, 0), Some(false));
    assert_eq!(file_avoids_discovery_keys(root.join("plain").as_path(), None, 0), Some(true));
    assert_eq!(file_avoids_discovery_keys(root.join("via").as_path(), None, 0), Some(false));
    remove(root.as_path());
}

/// A missing file passes, like an empty one in Git, and a `~/` include is read below the given home.
#[test]
fn a_missing_file_passes_and_a_home_include_is_followed() {
    let root = fixture("repo-read-scan-home");
    assert_eq!(file_avoids_discovery_keys(root.join("absent").as_path(), None, 0), Some(true));
    std::fs::write(root.join("target"), "[core]\n\tworktree = /x\n").expect("target");
    std::fs::write(root.join("entry"), "[include]\n\tpath = ~/target\n").expect("entry");
    assert_eq!(file_avoids_discovery_keys(root.join("entry").as_path(), Some(root.as_path()), 0), Some(false));
    assert_eq!(file_avoids_discovery_keys(root.join("entry").as_path(), None, 0), None, "no home to expand `~/`");
    remove(root.as_path());
}

/// A file that includes itself is declined once the depth limit is crossed.
#[test]
fn an_include_loop_is_declined() {
    let root = fixture("repo-read-scan-loop");
    std::fs::write(root.join("loop"), format!("[include]\n\tpath = {}\n", root.join("loop").display())).expect("loop");
    assert_eq!(file_avoids_discovery_keys(root.join("loop").as_path(), None, 0), None);
    remove(root.as_path());
}

/// `GIT_CONFIG_NOSYSTEM` is a boolean, and a relative path cannot be settled without the directory Git runs in.
#[test]
fn the_scope_environment_reads_booleans_and_refuses_relative_paths() {
    let variables = |items: &[(&str, &str)]| -> Vec<(OsString, OsString)> {
        return items.iter().map(|(name, value)| { return (OsString::from(*name), OsString::from(*value)); }).collect();
    };
    assert!(ScopeEnvironment::from_variables(variables(&[("GIT_CONFIG_NOSYSTEM", "true")]).as_slice()).no_system);
    assert!(!ScopeEnvironment::from_variables(variables(&[("GIT_CONFIG_NOSYSTEM", "false")]).as_slice()).no_system);
    assert!(!ScopeEnvironment::from_variables(variables(&[("GIT_CONFIG_NOSYSTEM", "0")]).as_slice()).no_system);
    assert!(!ScopeEnvironment::from_variables(variables(&[("GIT_CONFIG_NOSYSTEM", "")]).as_slice()).no_system);
    assert!(ScopeEnvironment::from_variables(variables(&[("GIT_CONFIG_NOSYSTEM", "perhaps")]).as_slice()).unresolvable);
    assert!(ScopeEnvironment::from_variables(variables(&[("GIT_CONFIG_GLOBAL", "relative.config")]).as_slice()).unresolvable);
    assert!(!ScopeEnvironment::from_variables(variables(&[("GIT_CONFIG_GLOBAL", "/absolute.config"), ("GIT_CONFIG_NOSYSTEM", "1")]).as_slice()).unresolvable);
    let unresolved = ScopeEnvironment { unresolvable: true, ..ScopeEnvironment::default() };
    assert_eq!(outer_scopes_avoid_keys(&unresolved, &discovery_key), None);
}

/// The scope files follow `GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_SYSTEM`, `GIT_CONFIG_GLOBAL` and the XDG directory.
#[test]
fn the_scope_environment_chooses_the_files() {
    let home = PathBuf::from("/home/someone");
    let base = ScopeEnvironment { home: Some(home.clone()), system_default: Some(PathBuf::from("/etc/gitconfig")), ..ScopeEnvironment::default() };
    assert_eq!(base.files(), vec![PathBuf::from("/etc/gitconfig"), home.join(".config/git/config"), home.join(".gitconfig")]);
    let moved = ScopeEnvironment { no_system: true, xdg: Some(PathBuf::from("/xdg")), ..base.clone() };
    assert_eq!(moved.files(), vec![PathBuf::from("/xdg/git/config"), home.join(".gitconfig")]);
    let replaced = ScopeEnvironment { system: Some(PathBuf::from("/s")), global: Some(PathBuf::from("/g")), ..base };
    assert_eq!(replaced.files(), vec![PathBuf::from("/s"), PathBuf::from("/g")]);
}
