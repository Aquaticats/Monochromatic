//! Controls for the compiled-in system configuration path:
//!  the learned path equals what the Git executable prints for itself, one start per executable identity,
//!  a replaced executable is asked again, and a path outside `/etc` is the file the readers look at.

use super::*;
use crate::repo_read_config_scan::ScopeEnvironment;
use crate::repo_read_layout::read_layout_with;
use crate::test_support::{REAL_GIT, fixture, git_output, remove, repository};
use std::cell::Cell;

/// Copy the host's Git to `root/name` so the copy is a real ELF executable named as asked.
fn copy_git(root: &Path, name: &str) -> PathBuf {
    let target: PathBuf = root.join(name);
    std::fs::copy(REAL_GIT, &target).expect("fixture");
    return target;
}

/// What the host Git prints for itself when the environment has no override.
fn printed_by_git() -> PathBuf {
    let output = std::process::Command::new(REAL_GIT).args(["var", "GIT_CONFIG_SYSTEM"]).env_clear().output().expect("git var");
    assert!(output.status.success());
    return parse_path_line(output.stdout.as_slice()).expect("an absolute path");
}

/// The learned path is the one Git prints, asked once for an executable and then read from the cache.
#[test]
fn the_learned_path_equals_what_git_prints_and_is_asked_once() {
    let root = fixture("system-config-learned");
    let git_copy = copy_git(root.as_path(), "git");
    let cache = root.join("xdg").join("monochromatic-git-policy-cli");
    let starts: Cell<usize> = Cell::new(0);
    let counting = |path: &Path| -> Option<Vec<u8>> {
        starts.set(starts.get() + 1);
        return ask_git(path);
    };
    let first = system_config_of(git_copy.as_path(), Some(cache.as_path()), &counting);
    assert_eq!(first, Some(printed_by_git()));
    assert_eq!(starts.get(), 1);
    assert_eq!(system_config_of(git_copy.as_path(), Some(cache.as_path()), &counting), first);
    assert_eq!(starts.get(), 1, "the second read must come from the cache");
    remove(root.as_path());
}

/// A replaced executable has another identity, so its path is asked again, and a Git whose prefix is not `/usr` is
///  believed: the path is whatever that executable says, not `/etc/gitconfig`.
#[test]
fn a_replaced_executable_is_asked_again_and_another_prefix_is_believed() {
    let root = fixture("system-config-replaced");
    let git_copy = copy_git(root.as_path(), "git");
    let cache = root.join("xdg").join("monochromatic-git-policy-cli");
    let elsewhere = root.join("opt").join("etc").join("gitconfig");
    let from_prefix = |_: &Path| -> Option<Vec<u8>> { return Some(format!("{}\n", elsewhere.display()).into_bytes()); };
    assert_eq!(system_config_of(git_copy.as_path(), Some(cache.as_path()), &from_prefix), Some(elsewhere.clone()));
    // Appending a byte changes the size and times, so the identity differs.
    let mut bytes = std::fs::read(&git_copy).expect("read");
    bytes.push(0);
    std::fs::write(&git_copy, bytes).expect("rewrite");
    let asked: Cell<bool> = Cell::new(false);
    let host = |path: &Path| -> Option<Vec<u8>> {
        asked.set(true);
        return ask_git(path);
    };
    assert_eq!(system_config_of(git_copy.as_path(), Some(cache.as_path()), &host), Some(printed_by_git()));
    assert!(asked.get(), "the stale entry must not be trusted");
    remove(root.as_path());
}

/// Output that is not one absolute path, an executable without an identity, and a failed start are unknown.
#[test]
fn unknown_answers_are_none() {
    for refused in ["", "\n", "relative/gitconfig\n", "/two\n/lines\n", "/no/newline", "/nul\0byte\n"] {
        assert_eq!(parse_path_line(refused.as_bytes()), None, "{refused:?}");
    }
    let root = fixture("system-config-unknown");
    let script = root.join("git");
    std::fs::write(&script, "#!/bin/sh\n").expect("script");
    let never = |_: &Path| -> Option<Vec<u8>> { panic!("a script has no identity, so Git is never asked"); };
    assert_eq!(system_config_of(script.as_path(), None, &never), None);
    let git_copy = copy_git(root.as_path(), "named-git");
    let failing = |_: &Path| -> Option<Vec<u8>> { return None; };
    // Only an executable named `git` has an identity.
    assert_eq!(system_config_of(git_copy.as_path(), None, &failing), None);
    // An executable with an identity whose start fails is unknown too.
    let named = copy_git(root.as_path(), "git-dir");
    let subdirectory = root.join("bin");
    std::fs::create_dir(&subdirectory).expect("directory");
    std::fs::rename(&named, subdirectory.join("git")).expect("move");
    assert_eq!(system_config_of(subdirectory.join("git").as_path(), None, &failing), None);
    remove(root.as_path());
}

/// The scope built from the child's variables learns the path of the Git the forward target names, and reads that file.
#[test]
fn the_scope_reads_the_file_the_git_names() {
    let root = fixture("system-config-scope");
    let git_copy = copy_git(root.as_path(), "git");
    let variables: Vec<(OsString, OsString)> = vec![
        (OsString::from(FORWARD_TARGET_VARIABLE), git_copy.clone().into_os_string()),
        (OsString::from("XDG_CACHE_HOME"), root.join("cache").into_os_string()),
        (OsString::from("HOME"), root.join("home").into_os_string()),
    ];
    let scope = ScopeEnvironment::from_variables(variables.as_slice());
    assert_eq!(scope.system_default, Some(printed_by_git()));
    assert!(!scope.unresolvable);
    assert_eq!(scope.labelled_files().first().map(|(label, file)| { return (*label, file.clone()); }), Some(("system", printed_by_git())));
    // Without the target the path is unknown, and the scope says so instead of assuming `/etc/gitconfig`.
    let unknown = ScopeEnvironment::from_variables(&[(OsString::from("HOME"), root.join("home").into_os_string())]);
    assert!(unknown.unresolvable);
    // With the system file removed or replaced, nothing needs to be learned.
    let removed = ScopeEnvironment::from_variables(&[(OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1"))]);
    assert!(!removed.unresolvable);
    let replaced = ScopeEnvironment::from_variables(&[(OsString::from("GIT_CONFIG_SYSTEM"), OsString::from("/replaced"))]);
    assert!(!replaced.unresolvable);
    remove(root.as_path());
}

/// A system file outside `/etc` that carries a decisive key makes the readers decline; the same repository with a clean
///  file there is answered, so the file that is read is the one the scope names.
#[test]
fn a_system_file_outside_etc_is_the_one_the_readers_check() {
    let root = fixture("system-config-outside-etc");
    let repo = repository(root.as_path(), "repo");
    let system = root.join("opt").join("etc").join("gitconfig");
    std::fs::create_dir_all(system.parent().expect("parent")).expect("directories");
    let scope = ScopeEnvironment {
        home: Some(root.join("no-home")),
        system_default: Some(system.clone()),
        global: Some(PathBuf::from("/nonexistent-global-config")),
        ..ScopeEnvironment::default()
    };
    std::fs::write(&system, "[core]\n\tbare = true\n").expect("system");
    assert!(read_layout_with(repo.as_path(), &[], &scope).is_none(), "a system file that sets core.bare must be declined");
    std::fs::write(&system, "[user]\n\tname = Someone\n").expect("system");
    assert!(read_layout_with(repo.as_path(), &[], &scope).is_some(), "a plain system file is answered");
    let real = git_output(repo.as_path(), &["rev-parse", "--git-dir"]);
    assert!(real.status.success());
    remove(root.as_path());
}
