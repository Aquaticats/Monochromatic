//! Unit tests of the retirement of the launchers,
//! records and exposed links of names the lock declares nowhere:
//! a bin renamed at its release,
//! a file that is not provably this tool's own,
//! and the states a killed run leaves.
//! Each runs against the fake runner inside a disposable scope.

/// Imports `fs` from `std`.
use std::fs;
/// Imports `PermissionsExt` from `std::os::unix::fs`,
/// for the one test that takes the right to remove a file away.
use std::os::unix::fs::PermissionsExt;
/// Imports `symlink` from `std::os::unix::fs`.
use std::os::unix::fs::symlink;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `current_link` from `super::cargo_installs`.
use super::cargo_installs::current_link;
/// Imports `rebuild_exposure` from `super::exposure`.
use super::exposure::rebuild_exposure;
/// Imports what this module uses from `super::fakes`.
use super::fakes::{Rig, write_executable};
/// Imports what this module uses from `super::launcher`.
use super::launcher::{LaunchRecord, ensure_launcher, record_path, render_record};
/// Imports `ScopeKind` from `super::scope`.
use super::scope::ScopeKind;
/// Imports what this module uses from `super::stale_launchers`.
use super::stale_launchers::{LauncherNotices, retire_undeclared_launchers};

/// The repository-published tool of the sample lock,
/// the one tool the fixture installs a launcher for.
const SHADOW_TOOL: &str = "cargo:monochromatic-mpm-shadow";

/// The crate the sample plan locks for that tool.
const SHADOW_CRATE: &str = "monochromatic-mpm-shadow";

/// The name every sentence of this tool gives the running program.
const DECLARED_BIN: &str = "mpm-shadow";

/// The name the wrapper had before its 0.6.0 release renamed it:
/// a name no lock of this shape declares, standing beside a record whose tool the lock still declares,
/// which is the leftover the wrapper's relink counted as stale.
const RENAMED_BIN: &str = "cli-git-native";

/// The version the fixture installs.
const INSTALLED: &str = "0.2.0";

/// A rig whose store holds a finished `cargo-binstall` install of the launcher-owning tool,
/// so `rebuild_exposure` finds its program and makes its launcher and farm link.
fn rig_with_installed_shadow() -> Rig {
    let rig = Rig::new();
    let version_dir: PathBuf = rig.scope.toolchain_root.join("cargo").join(SHADOW_CRATE).join(INSTALLED);
    let program: PathBuf = version_dir.join("bin").join(DECLARED_BIN);
    write_executable(&program);
    fs::write(
        version_dir.join(".crates.toml"),
        format!("[v1]\n\"{SHADOW_CRATE} {INSTALLED} (registry+https://example.invalid/index)\" = [\"{DECLARED_BIN}\"]\n"),
    )
    .expect("write cargo's record");
    symlink(INSTALLED, current_link(&rig.scope.toolchain_root, SHADOW_CRATE)).expect("link current");
    return rig;
}

/// A launcher and its record for `name` with `tool_id` in the record,
/// made the way `install` makes them.
fn make_launcher(rig: &Rig, name: &str, tool_id: &str) -> PathBuf {
    let program: PathBuf = rig.scope.root.join("store").join(name);
    write_executable(&program);
    return ensure_launcher(&rig.scope, tool_id, name, &program).expect("the launcher is made");
}

/// The exposed link `install` makes for `name`.
fn make_link(rig: &Rig, name: &str, target: &Path) -> PathBuf {
    fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let link: PathBuf = rig.scope.exposure_dir.join(name);
    symlink(target, &link).expect("link the name");
    return link;
}

/// One pass over the rig's folders.
fn retire(rig: &Rig) -> LauncherNotices {
    return retire_undeclared_launchers(&rig.context(), &[]);
}

/// Whether anything (even a dangling link) exists at `path`.
fn exists_at(path: &Path) -> bool {
    return fs::symlink_metadata(path).is_ok();
}

#[test]
fn a_launcher_a_record_and_a_link_of_a_renamed_bin_are_all_retired() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    let record: PathBuf = record_path(&launcher);
    let link: PathBuf = make_link(&rig, RENAMED_BIN, &launcher);
    assert!(exists_at(&launcher) && exists_at(&record) && exists_at(&link));

    let notices = retire(&rig);

    assert!(!exists_at(&launcher), "the old launcher is gone");
    assert!(!exists_at(&record), "its record is gone");
    assert!(!exists_at(&link), "the exposed link is gone");
    assert_eq!(notices.retired.len(), 3, "{notices:?}");
    assert!(notices.warnings.is_empty(), "{notices:?}");
    assert!(notices.retired.iter().all(|line| return line.contains(RENAMED_BIN)), "{notices:?}");
    assert_eq!(retire(&rig), LauncherNotices::default(), "a later pass has nothing left to do");
}

#[test]
fn the_launcher_and_link_the_lock_declares_stay() {
    let rig = rig_with_installed_shadow();
    let tool = rig.tool(SHADOW_TOOL);
    rebuild_exposure(&rig.context(), &[&tool]).expect("the farm is rebuilt");
    let launcher: PathBuf = rig.scope.launcher_dir.join(DECLARED_BIN);
    let link: PathBuf = rig.scope.exposure_dir.join(DECLARED_BIN);
    assert!(exists_at(&launcher) && exists_at(&link), "the fixture made the declared launcher and link");

    assert_eq!(retire(&rig), LauncherNotices::default(), "the lock declares this name, so nothing is decided about it");
    assert!(exists_at(&launcher) && exists_at(&link));
}

#[test]
fn a_persons_file_where_the_old_launcher_stood_stays_and_the_install_says_why() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    fs::remove_file(&launcher).expect("remove the launcher");
    fs::write(&launcher, "#!/bin/sh\necho a person's own script\n").expect("write a person's file");

    let notices = retire(&rig);

    assert_eq!(fs::read_to_string(&launcher).expect("still there"), "#!/bin/sh\necho a person's own script\n");
    assert!(exists_at(&record_path(&launcher)), "the record stays with the file it belongs to");
    assert!(notices.retired.is_empty(), "{notices:?}");
    assert_eq!(notices.warnings.len(), 1, "{notices:?}");
    assert!(
        notices.warnings[0].contains(&launcher.display().to_string()) && notices.warnings[0].contains("no mpm-shadow program"),
        "{notices:?}"
    );
}

#[test]
fn a_launcher_with_no_record_stays() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    fs::remove_file(record_path(&launcher)).expect("forget the record");

    let notices = retire(&rig);

    assert!(exists_at(&launcher), "without a record nothing says mpm-shadow made it");
    assert_eq!(notices.warnings.len(), 1, "{notices:?}");
    assert!(notices.warnings[0].contains("record"), "{notices:?}");
}

#[test]
fn a_record_naming_a_tool_the_lock_declares_nowhere_keeps_its_launcher() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, "cargo:a-tool-this-lock-no-longer-declares");

    let notices = retire(&rig);

    assert!(exists_at(&launcher), "the tool the record names is gone from the lock, so the record is not proven");
    assert!(exists_at(&record_path(&launcher)));
    assert_eq!(notices.warnings.len(), 1, "{notices:?}");
    assert!(notices.warnings[0].contains("declares that tool nowhere"), "{notices:?}");
}

#[test]
fn a_record_left_without_its_launcher_is_retired_once_the_tool_is_declared() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    fs::remove_file(&launcher).expect("the launcher is already gone");

    let notices = retire(&rig);

    assert!(!exists_at(&record_path(&launcher)), "the record alone is not left behind");
    assert_eq!(notices.retired.len(), 1, "{notices:?}");
}

#[test]
fn a_link_to_a_program_whose_name_the_lock_declares_nowhere_stays() {
    let rig = rig_with_installed_shadow();
    let program: PathBuf = rig.scope.root.join("store").join("an-old-tool");
    write_executable(&program);
    let link: PathBuf = make_link(&rig, "an-old-tool", &program);

    let notices = retire(&rig);

    assert!(exists_at(&link), "a link to a real program is not provably a launcher mpm-shadow made");
    assert_eq!(notices.warnings.len(), 1, "{notices:?}");
    assert!(
        notices.warnings[0].contains("no launcher record") && notices.warnings[0].contains("stands beside it"),
        "{notices:?}"
    );
}

#[test]
fn every_state_a_kill_can_leave_converges_and_never_leaves_a_dangling_link() {
    // A kill between removing the link and removing the launcher leaves the launcher and its record.
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    let record: PathBuf = record_path(&launcher);
    let link: PathBuf = make_link(&rig, RENAMED_BIN, &launcher);
    fs::remove_file(&link).expect("the killed run removed the link first");
    let notices = retire(&rig);
    assert!(!exists_at(&launcher) && !exists_at(&record), "the next install finishes the retirement");
    assert_eq!(notices.warnings.len(), 0, "{notices:?}");

    // A kill between removing the launcher and removing its record leaves the record alone.
    let second = rig_with_installed_shadow();
    let its_launcher: PathBuf = make_launcher(&second, RENAMED_BIN, SHADOW_TOOL);
    let its_record: PathBuf = record_path(&its_launcher);
    let its_link: PathBuf = make_link(&second, RENAMED_BIN, &its_launcher);
    fs::remove_file(&its_link).expect("the killed run removed the link first");
    fs::remove_file(&its_launcher).expect("and then the launcher");
    assert!(exists_at(&its_record), "the state a kill between the two leaves");
    let second_notices = retire(&second);
    assert!(!exists_at(&its_record), "the record alone is retired");
    assert_eq!(second_notices.retired.len(), 1, "{second_notices:?}");
}

#[test]
fn a_link_that_cannot_be_removed_keeps_its_launcher() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    let link: PathBuf = make_link(&rig, RENAMED_BIN, &launcher);
    let farm: &Path = &rig.scope.exposure_dir;
    fs::set_permissions(farm, fs::Permissions::from_mode(0o555)).expect("take the right to remove away");
    let notices = retire(&rig);
    fs::set_permissions(farm, fs::Permissions::from_mode(0o755)).expect("give it back");

    assert!(exists_at(&link), "the link is still there");
    assert!(exists_at(&launcher), "a launcher is never removed while a link points at it");
    assert!(exists_at(&record_path(&launcher)));
    assert!(notices.warnings.iter().any(|line| return line.contains(&link.display().to_string())), "{notices:?}");
}

#[test]
fn a_launcher_whose_installer_is_gone_is_proven_by_the_copy_the_other_scope_holds() {
    let rig = rig_with_installed_shadow();
    // A launcher placed by an installer version the store no longer keeps: not a program this tool can name, and its own bytes
    // carry no identity, so only the copy of the same bytes in the machine-global folder can prove it.
    let launcher: PathBuf = rig.scope.launcher_dir.join(RENAMED_BIN);
    fs::create_dir_all(&rig.scope.launcher_dir).expect("make the launcher folder");
    fs::write(&launcher, "an installer build this store no longer keeps").expect("write the old copy");
    let record: LaunchRecord = LaunchRecord {
        tool_id: SHADOW_TOOL.to_string(),
        lock_file: rig.scope.lock_file.clone(),
        real: rig.scope.root.join("store").join(RENAMED_BIN),
        scope: ScopeKind::Repo,
    };
    fs::write(record_path(&launcher), render_record(&record)).expect("write its record");

    let alone = retire(&rig);
    assert!(exists_at(&launcher), "no proof at all, so the file stays");
    assert!(alone.warnings[0].contains("no mpm-shadow program"), "{alone:?}");

    let global: super::scope::Scope = super::scope::global_scope(&rig.env).expect("a global scope");
    fs::create_dir_all(&global.launcher_dir).expect("make the machine-global launcher folder");
    let twin: PathBuf = global.launcher_dir.join(RENAMED_BIN);
    fs::write(&twin, "an installer build this store no longer keeps").expect("write the copy the other scope holds");
    fs::write(record_path(&twin), render_record(&record)).expect("write its record");

    let proven = retire(&rig);
    assert!(!exists_at(&launcher), "the copy the machine holds proves the bytes are one of mpm-shadow's own");
    assert!(!exists_at(&record_path(&launcher)));
    assert!(exists_at(&twin), "the machine-global folder is not this checkout's to clean");
    assert!(proven.retired.iter().any(|line| return line.contains("the same as the launcher")), "{proven:?}");
}

#[test]
fn a_launcher_a_link_still_reaches_is_kept_and_the_link_that_cannot_go_is_named() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    // A link of a name the lock dropped, in a folder a tool's postinstall step links into, that this pass cannot remove:
    // the launcher it points at must stay, and the link is named.
    let postinstall_dir: PathBuf = rig.scope.root.join(".cache").join("old-bin");
    fs::create_dir_all(&postinstall_dir).expect("make the postinstall link folder");
    let postinstall: PathBuf = postinstall_dir.join("git");
    symlink(&launcher, &postinstall).expect("the postinstall link");
    fs::set_permissions(&postinstall_dir, fs::Permissions::from_mode(0o555)).expect("take the right to remove away");

    let notices = retire(&rig);

    fs::set_permissions(&postinstall_dir, fs::Permissions::from_mode(0o755)).expect("give it back");
    assert!(exists_at(&postinstall), "the link is still there");
    assert!(exists_at(&launcher), "a launcher is never removed while a link points at it");
    assert!(exists_at(&record_path(&launcher)));
    assert!(notices.warnings.iter().any(|line| return line.contains(&postinstall.display().to_string())), "{notices:?}");
}

#[test]
fn a_link_that_names_another_checkouts_launcher_is_retired_with_the_name_this_lock_dropped() {
    let source = rig_with_installed_shadow();
    let elsewhere: PathBuf = make_launcher(&source, RENAMED_BIN, SHADOW_TOOL);
    let rig = rig_with_installed_shadow();
    // The copied `.cache` of a new worktree: its own copy of the launcher, and a farm link that still names the checkout
    // the copy was made from.
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    let link: PathBuf = make_link(&rig, RENAMED_BIN, &elsewhere);

    let notices = retire(&rig);

    assert!(!exists_at(&link), "the link names a launcher, whichever checkout placed it: {notices:?}");
    assert!(!exists_at(&launcher) && !exists_at(&record_path(&launcher)));
    assert!(exists_at(&elsewhere), "another checkout's launcher is not this checkout's to remove");
}

#[test]
fn a_link_whose_launcher_is_gone_from_the_checkout_it_was_copied_from_is_retired_by_this_checkouts_own() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    // The source checkout cleaned up first, so the link names a file that is gone; the copy still holds the same launcher.
    let gone: PathBuf = rig.scope.root.join("the-checkout-this-was-copied-from").join(".cache").join("toolchain-launchers").join(RENAMED_BIN);
    let link: PathBuf = make_link(&rig, RENAMED_BIN, &gone);

    let notices = retire(&rig);

    assert!(!exists_at(&link), "the launcher of that name here proves what the link named: {notices:?}");
    assert!(!exists_at(&launcher));
}

#[test]
fn a_link_whose_launcher_is_gone_everywhere_stays() {
    let rig = rig_with_installed_shadow();
    let gone: PathBuf = rig.scope.root.join("somewhere").join(RENAMED_BIN);
    let link: PathBuf = make_link(&rig, RENAMED_BIN, &gone);

    let notices = retire(&rig);

    assert!(exists_at(&link), "nothing proves what it named");
    assert_eq!(notices.warnings.len(), 1, "{notices:?}");
    assert!(notices.warnings[0].contains("in no launcher folder"), "{notices:?}");
}

//region Links no record knows about, and links that were never mpm-shadow's
// The review of the first commit found both: a postinstall link a killed install made before it wrote the record was
// invisible to the links this pass asked about, so its launcher was removed and the link dangled; and a link that did
// not resolve was judged by the name of its target alone, so a person's own broken link went with it.

#[test]
fn a_postinstall_link_no_record_knows_of_is_retired_with_its_launcher() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    // The state of a first install killed after it made its postinstall links and before it wrote the record of them,
    // and the same state with an unreadable record: nothing names this link.
    let link_directory: PathBuf = rig.scope.root.join(".cache").join(format!("{RENAMED_BIN}-bin"));
    fs::create_dir_all(&link_directory).expect("make the postinstall link folder");
    let link: PathBuf = link_directory.join("linked");
    symlink(&launcher, &link).expect("the link a killed install made");

    let notices = retire(&rig);

    assert!(!exists_at(&link), "the link names a launcher of a name the lock dropped, so it goes too: {notices:?}");
    assert!(!exists_at(&launcher) && !exists_at(&record_path(&launcher)));
}

#[test]
fn a_broken_link_that_names_nothing_in_a_launcher_folder_is_left_alone() {
    let rig = rig_with_installed_shadow();
    // The stale launcher this pass would judge, with the name the person's link happens to share.
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    let elsewhere: PathBuf = rig.scope.root.join("elsewhere").join(RENAMED_BIN);
    let link: PathBuf = make_link(&rig, RENAMED_BIN, &elsewhere);

    let notices = retire(&rig);

    assert!(exists_at(&link), "the link named a path in no launcher folder, so nothing proves mpm-shadow made it: {notices:?}");
    assert_eq!(fs::read_link(&link).expect("still the person's link"), elsewhere);
    assert!(!exists_at(&launcher), "the launcher itself is still retired; the person's own broken link is not");
}

//endregion
