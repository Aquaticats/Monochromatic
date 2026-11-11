//! Unit tests of which `mpm-shadow` a launcher is a copy of:
//! the newest installed one through its `current` link in either toolchain root,
//! the running program when it belongs to no install,
//! and the `/proc/self/exe` read that makes a deleted running program copyable.
//! Starting a program and passing its streams and exit status through are covered by `tests/launcher.rs`,
//! which runs the built binary.

use std::fs;
use std::path::{Path, PathBuf};

use super::fakes::Rig;
use super::launcher_source::{PROC_SELF_EXE, launcher_source, newest_installed_shadow, usable_source};

/// Where an install of `mpm-shadow` at `version` puts the program,
/// in the layout a release archive unpacks into.
fn shadow_program(toolchain_root: &Path, version: &str) -> PathBuf {
    return toolchain_root.join("mpm-shadow").join(version).join(format!("monochromatic-mpm-shadow-{version}-linux")).join("mpm-shadow");
}

/// The program of `mpm-shadow` at `version` as the launcher source names it:
/// the shortest path,
/// through the root's `current` link,
/// so a launcher made from it stays valid when a newer version replaces the folder behind the link.
fn shadow_through_current(toolchain_root: &Path, version: &str) -> PathBuf {
    return toolchain_root
        .join("mpm-shadow")
        .join("current")
        .join(format!("monochromatic-mpm-shadow-{version}-linux"))
        .join("mpm-shadow");
}

/// Writes `content` as the program of `mpm-shadow` at `version` in `toolchain_root`
/// and points that root's `current` link at the version.
fn install_shadow(toolchain_root: &Path, version: &str, content: &str) -> PathBuf {
    let program: PathBuf = shadow_program(toolchain_root, version);
    fs::create_dir_all(program.parent().expect("a parent")).expect("make the install");
    fs::write(&program, content).expect("write the program");
    fs::set_permissions(&program, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("make the program executable");
    let link: PathBuf = toolchain_root.join("mpm-shadow").join("current");
    let _ = fs::remove_file(&link);
    std::os::unix::fs::symlink(version, &link).expect("link current");
    return program;
}

#[test]
fn the_newest_install_in_the_same_root_is_the_source() {
    let rig = Rig::new();
    let running: PathBuf = install_shadow(&rig.scope.toolchain_root, "0.2.0", "the running one");
    install_shadow(&rig.scope.toolchain_root, "0.2.1", "the newest one");
    let newest: PathBuf = shadow_through_current(&rig.scope.toolchain_root, "0.2.1");
    assert_eq!(newest_installed_shadow(&rig.scope, &running), Some(newest.clone()));
    assert_eq!(launcher_source(&rig.scope, running, true).expect("a source"), newest, "the newest installed shadow, not the installer");
}

#[test]
fn a_newer_install_in_the_other_root_is_the_source() {
    let rig = Rig::new();
    let running: PathBuf = install_shadow(&rig.scope.toolchain_root, "0.2.0", "the running one");
    install_shadow(&rig.scope.other_toolchain_root, "0.2.1", "the newest one");
    let newest: PathBuf = shadow_through_current(&rig.scope.other_toolchain_root, "0.2.1");
    assert_eq!(
        launcher_source(&rig.scope, running, true).expect("a source"),
        newest,
        "the newest copy wins wherever it is, the rule the repository's own tools already follow"
    );
}

#[test]
fn an_older_install_in_the_other_root_does_not_replace_the_one_that_runs() {
    let rig = Rig::new();
    let running: PathBuf = install_shadow(&rig.scope.toolchain_root, "0.2.1", "the running one");
    install_shadow(&rig.scope.other_toolchain_root, "0.2.0", "the older one");
    let newest: PathBuf = shadow_through_current(&rig.scope.toolchain_root, "0.2.1");
    assert_eq!(newest_installed_shadow(&rig.scope, &running), Some(newest), "the newer version stays the source");
}

#[test]
fn the_root_whose_current_link_names_no_program_falls_back_to_the_other_one() {
    let rig = Rig::new();
    let running: PathBuf = install_shadow(&rig.scope.toolchain_root, "0.2.1", "the running one");
    fs::remove_file(shadow_program(&rig.scope.toolchain_root, "0.2.1")).expect("remove the newest program");
    install_shadow(&rig.scope.other_toolchain_root, "0.2.0", "the one that is there");
    let other: PathBuf = shadow_through_current(&rig.scope.other_toolchain_root, "0.2.0");
    assert_eq!(newest_installed_shadow(&rig.scope, &running), Some(other), "a version whose program is gone is skipped");
}

#[test]
fn a_deleted_note_on_the_running_program_leaves_its_file_name_readable() {
    let rig = Rig::new();
    let running: PathBuf = install_shadow(&rig.scope.toolchain_root, "0.2.0", "the running one");
    install_shadow(&rig.scope.toolchain_root, "0.2.1", "the newest one");
    let newest: PathBuf = shadow_through_current(&rig.scope.toolchain_root, "0.2.1");
    // What the kernel reports for a running program whose file a newer install deleted.
    let reported: PathBuf = PathBuf::from(format!("{} (deleted)", running.display()));
    assert_eq!(newest_installed_shadow(&rig.scope, &reported), Some(newest));
}

#[test]
fn a_running_program_that_belongs_to_no_install_is_the_source() {
    let rig = Rig::new();
    let newest: PathBuf = install_shadow(&rig.scope.toolchain_root, "0.2.1", "an install that is not where this one runs from");
    let running: PathBuf = rig._directory.path().join("build").join("debug").join("mpm-shadow");
    fs::create_dir_all(running.parent().expect("a parent")).expect("make the build folder");
    fs::write(&running, "the running one").expect("write the program");
    assert_eq!(newest_installed_shadow(&rig.scope, &running), None, "a build folder is no install");
    assert_eq!(launcher_source(&rig.scope, running.clone(), true).expect("a source"), running);
    assert!(newest.exists(), "the install is still there and is not chosen for a program outside both roots");
}

#[test]
fn a_file_that_exists_is_used_as_it_is_and_a_deleted_one_is_read_through_proc() {
    let rig = Rig::new();
    let present = rig._directory.path().join("present");
    fs::write(&present, "x").expect("write");
    assert_eq!(usable_source(present.clone(), true).expect("used"), present, "a file that exists is used as it is");
    assert_eq!(usable_source(present.clone(), false).expect("used"), present, "on every platform");
    let gone = rig._directory.path().join("deleted (deleted)");
    assert_eq!(usable_source(gone.clone(), true).expect("through proc"), PathBuf::from(PROC_SELF_EXE), "a replaced running binary is read through /proc");
    let error = usable_source(gone.clone(), false).expect_err("no /proc");
    assert!(error.message.contains(&gone.display().to_string()), "the error names the vanished path: {error}");
    assert!(error.message.contains("no longer exists") && !error.message.contains("/proc"), "and does not blame a /proc path: {error}");
}

#[test]
fn an_install_on_a_machine_whose_home_is_a_link_is_still_found() {
    let (rig, canonical_home) = Rig::through_symlinked_home();
    // The install sits under the physical home, and the running program reports that resolved spelling,
    // while the scope spells every path through the link (the machine's `/home` is a link to `/var/home`).
    let physical_root: PathBuf = canonical_home.join(".local").join("share").join("monochromatic-dev").join("toolchain");
    let running: PathBuf = install_shadow(&physical_root, "0.2.1", "the newest one");
    let newest: PathBuf = shadow_through_current(&rig.scope.toolchain_root, "0.2.1");
    assert_eq!(
        newest_installed_shadow(&rig.scope, &running),
        Some(newest),
        "an install is found by the file it is, never by the spelling of the path"
    );
}
