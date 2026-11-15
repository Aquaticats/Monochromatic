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
/// Imports what this module uses from `super::model`.
use super::model::{Postinstall, PostinstallLink};
/// Imports `render_lock` from `super::lock`.
use super::lock::render_lock;
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
    return ensure_launcher(&rig.scope, tool_id, name, &program, INSTALLED).expect("the launcher is made").launcher;
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
    // The tests' fixtures ask as if the record of postinstall links were readable, except the one that asks otherwise.
    return retire_undeclared_launchers(&rig.context(), &[], true);
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

//region Links no record names, and links that were never mpm-shadow's
// A link a killed install made before it wrote the record of postinstall links is a link no record names, and the
// ninth review's rule is that a name no record has is a person's: it is left, its launcher stays (a link reaches it),
// and the sentence says what no record proves. A link that does not resolve is judged by its stated target alone, so a
// person's own broken link stays with the launcher the statement keeps.

#[test]
fn a_link_no_record_names_is_left_with_its_launcher_and_said() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    // The state of a first install killed after it made its postinstall links and before it wrote the record of them:
    // nothing names this link, so nothing says this tool wrote that entry.
    let link_directory: PathBuf = rig.scope.root.join(".cache").join(format!("{RENAMED_BIN}-bin"));
    fs::create_dir_all(&link_directory).expect("make the postinstall link folder");
    let link: PathBuf = link_directory.join("linked");
    symlink(&launcher, &link).expect("the link a killed install made");

    let notices = retire(&rig);

    assert!(exists_at(&link), "a name no record has is a person's, and it stays: {notices:?}");
    assert!(
        exists_at(&launcher) && exists_at(&record_path(&launcher)),
        "the link reaches its launcher, so the launcher and the record stay: {notices:?}"
    );
    assert!(
        notices.warnings.iter().any(|line| return line.contains(&link.display().to_string())),
        "the sentence names the link that keeps the launcher: {notices:?}"
    );
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
    assert!(!exists_at(&launcher), "the launcher itself is still retired; the person's own broken link is not: {notices:?}");
}

//endregion

//region Judgement sets: what the lock declares, and what belongs to a tool's own postinstall command
// The third review found a declared postinstall link retired in the very install that made it, a person's broken link
// removed because its folder only shared a basename with a launcher folder, and warnings for links the lock's own
// postinstall command keeps.

#[test]
fn a_postinstall_link_the_lock_declares_inside_the_farm_stays() {
    let mut rig = rig_with_installed_shadow();
    // A postinstall link may name the farm as its link directory; the path, not a bin name, is then what declares it.
    let tool = rig.lock.tools.iter_mut().find(|tool| return tool.declaration.id == SHADOW_TOOL).expect("the sample lock holds the tool");
    tool.declaration.options.postinstall = Some(Postinstall::Link(PostinstallLink {
        source_bin: DECLARED_BIN.to_string(),
        link_name: "mpm".to_string(),
        link_dir: Some(".cache/toolchain-bin".to_string()),
    }));
    // The retirement reads the lock file now, so the declaration the test names must be on disk, as an install leaves it.
    fs::write(&rig.scope.lock_file, render_lock(&rig.lock)).expect("write the lock the test declares");
    let launcher: PathBuf = make_launcher(&rig, DECLARED_BIN, SHADOW_TOOL);
    let declared: PathBuf = make_link(&rig, "mpm", &launcher);

    let notices = retire(&rig);

    assert!(exists_at(&declared), "the lock declares this link at this path, so nothing decides about it: {notices:?}");
    assert!(exists_at(&launcher) && exists_at(&record_path(&launcher)));
}

#[test]
fn a_broken_link_to_a_folder_only_shaped_like_a_launcher_folder_stays() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    // A folder named like the launcher folder but somewhere else is not one this tool keeps.
    let elsewhere: PathBuf = rig.scope.root.join("elsewhere").join("toolchain-launchers").join(RENAMED_BIN);
    let link: PathBuf = make_link(&rig, RENAMED_BIN, &elsewhere);

    let notices = retire(&rig);

    assert!(exists_at(&link), "the stated folder only shares the basename, so nothing proves the link named a launcher: {notices:?}");
    assert_eq!(fs::read_link(&link).expect("still the person's link"), elsewhere);
    assert!(!exists_at(&launcher), "the launcher itself is still retired");
}

#[test]
fn a_link_a_tools_own_postinstall_command_keeps_is_neither_removed_nor_warned_about() {
    let rig = rig_with_installed_shadow();
    // A launcher folder with something in it, so the pass reaches the other link folders at all.
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    assert!(exists_at(&launcher));
    // The search shadow's postinstall command links `rg`, `grep` and seven more at the store program; nothing the
    // retirement can prove, and nothing a person can act on.
    let program: PathBuf = rig.scope.root.join("store").join("search-shadow");
    write_executable(&program);
    let directory: PathBuf = rig.scope.root.join(".cache").join("search-shadow-bin");
    fs::create_dir_all(&directory).expect("make the folder the postinstall command links into");
    let link: PathBuf = directory.join("rg");
    symlink(&program, &link).expect("the link that command keeps");

    let notices = retire(&rig);

    assert!(exists_at(&link), "the link is the postinstall command's, and it stays");
    assert!(notices.warnings.is_empty(), "no warning for a link its own postinstall command keeps: {notices:?}");
}

//endregion

//region A removal is decided by the lock on disk at that instant
// The fourth review found that a decision taken on the lock an install loaded deletes what a newer lock declares and
// another install has just placed.

#[test]
fn a_launcher_a_newer_lock_declares_is_kept_whatever_this_install_loaded() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    let record: PathBuf = record_path(&launcher);
    let link: PathBuf = make_link(&rig, RENAMED_BIN, &launcher);
    // Another install publishes a lock that declares this name; this install still holds the lock it loaded.
    let mut newer = rig.lock.clone();
    newer
        .tools
        .iter_mut()
        .find(|tool| return tool.declaration.id == SHADOW_TOOL)
        .expect("the sample lock holds the tool")
        .declaration
        .bins
        .push(RENAMED_BIN.to_string());
    fs::write(&rig.scope.lock_file, render_lock(&newer)).expect("publish the newer lock");

    let notices = retire(&rig);

    assert!(exists_at(&launcher), "the lock on disk declares this name, so nothing decides about its launcher: {notices:?}");
    assert!(exists_at(&record));
    assert!(exists_at(&link), "and nothing decides about its farm link");
}

//region The ownership rule, one control per rule
// The fifth review's findings and the rule the sixth brief asked for: a thing goes only when this tool's own record
// proves it placed it, the lock on disk read again does not declare it, no live process owns it, and the removal is one
// step that fails when the thing changed since the look.

/// One pass that takes the record of postinstall links as unreadable before this run wrote it.
fn retire_while_the_link_set_is_unknown(rig: &Rig) -> LauncherNotices {
    return retire_undeclared_launchers(&rig.context(), &[], false);
}

#[test]
fn a_launcher_is_kept_and_counted_while_the_link_set_is_unknown() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    let record: PathBuf = record_path(&launcher);

    let notices = retire_while_the_link_set_is_unknown(&rig);

    assert!(exists_at(&launcher) && exists_at(&record), "a link a killed run made may point at it and nothing can find that \
        folder, so the launcher stays: {notices:?}");
    assert_eq!(notices.retired.len(), 0, "{notices:?}");
    assert_eq!(notices.warnings.len(), 1, "one line and no more: {notices:?}");
    assert!(notices.warnings[0].contains("left 1 launcher(s)"), "{notices:?}");
}

#[test]
fn a_live_installs_temporary_is_left_and_keeps_its_launcher() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    // The link an install is placing this instant: `atomic_symlink`'s temporary, carrying that install's process number.
    // This process stands in for it, so the number is alive and the temporary is in flight.
    let directory: PathBuf = rig.scope.root.join(".cache").join("placing-bin");
    fs::create_dir_all(&directory).expect("make the link folder");
    let temporary: PathBuf = directory.join(format!(".mpm.{}.tmp", std::process::id()));
    symlink(&launcher, &temporary).expect("the temporary an install is placing");

    let notices = retire(&rig);

    assert!(exists_at(&temporary), "a live install's temporary is never a leftover: {notices:?}");
    assert!(exists_at(&launcher) && exists_at(&record_path(&launcher)), "and its target is not removed from under it: {notices:?}");
}

#[test]
fn a_removal_that_the_lock_declares_puts_the_entry_back() {
    let directory = crate::test_scratch::scratch();
    let target: PathBuf = directory.path().join("target");
    fs::write(&target, "x").expect("write the target");
    let entry: PathBuf = directory.path().join("entry");
    symlink(&target, &entry).expect("the entry");
    let judged = super::removal::judged_of(&entry).expect("the entry is inspected");

    // The closure is the seam: this is where the pass asks the lock on disk again, and a lock that declares the entry
    // means keep it.
    let outcome = super::removal::remove_if_still(&entry, judged, &|| return false);

    assert_eq!(outcome.expect("the removal runs"), super::removal::Removal::Kept);
    assert!(exists_at(&entry), "the entry was put back, not deleted");
}

//endregion

#[test]
fn a_temporary_is_judged_only_when_its_maker_is_gone() {
    // `atomic_symlink` writes `.<name>.<pid>.tmp` and a removal writes `.<name>.<pid>.<n>.retired`; the number in both is
    // the maker. A maker that is alive owns its temporary; one that is gone does not.
    assert_eq!(super::stale_launchers::maker_of(&format!(".mpm.{}.tmp", std::process::id())), Some(std::process::id()));
    assert_eq!(super::stale_launchers::maker_of(&format!(".mpm.{}.0.retired", std::process::id())), Some(std::process::id()));
    assert!(super::stale_launchers::a_live_installs_temporary(&format!(".mpm.{}.tmp", std::process::id())));
    // A number no process can have: the maker is gone, so the temporary is a leftover.
    assert!(!super::stale_launchers::a_live_installs_temporary(".mpm.4294967.tmp"));
    assert!(!super::stale_launchers::a_live_installs_temporary("mpm"));
}

#[test]
fn the_removal_asks_the_lock_on_disk_at_that_instant() {
    let rig = rig_with_installed_shadow();
    let mut newer = rig.lock.clone();
    let tool = newer.tools.iter_mut().find(|tool| return tool.declaration.id == SHADOW_TOOL).expect("the sample lock holds the tool");
    tool.declaration.bins.push(RENAMED_BIN.to_string());
    fs::write(&rig.scope.lock_file, render_lock(&newer)).expect("publish the lock that declares the name");
    let context = rig.context();

    // The rule, asked on its own: a name the lock on disk declares is not one a removal may take.
    assert_eq!(super::stale_launchers::still_undeclared(&context, &[], RENAMED_BIN, None), Some(false));
    assert!(!super::stale_launchers::may_remove(&context, &[], RENAMED_BIN, None));

    // And with a lock that declares it nowhere, the removal may go.
    fs::write(&rig.scope.lock_file, render_lock(&rig.lock)).expect("publish a lock without the name");
    let without = rig.context();
    assert_eq!(super::stale_launchers::still_undeclared(&without, &[], RENAMED_BIN, None), Some(true));
    assert!(super::stale_launchers::may_remove(&without, &[], RENAMED_BIN, None));
    assert_eq!(super::stale_launchers::still_undeclared(&without, &[], RENAMED_BIN, None), Some(true));
}

#[test]
fn what_a_killed_run_left_under_a_hidden_name_is_put_back_and_said() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    // The state of a run killed right after the move-aside: the entry is only under a hidden name, and the run that made
    // it is gone (4294967 is a number no process can have).
    let hidden: PathBuf = launcher.with_file_name(format!(".{RENAMED_BIN}.4294967.0.retired"));
    fs::rename(&launcher, &hidden).expect("the killed run's move-aside");
    // The lock on disk declares the name, so the pass this repair runs in front of keeps it once it is back.
    let mut newer = rig.lock.clone();
    newer
        .tools
        .iter_mut()
        .find(|tool| return tool.declaration.id == SHADOW_TOOL)
        .expect("the sample lock holds the tool")
        .declaration
        .bins
        .push(RENAMED_BIN.to_string());
    fs::write(&rig.scope.lock_file, render_lock(&newer)).expect("publish the lock that declares the name");

    let notices = retire(&rig);

    assert!(exists_at(&launcher), "the next pass puts it back at its own name: {notices:?}");
    assert!(!exists_at(&hidden));
    assert!(notices.warnings.iter().any(|line| return line.contains("put back")), "{notices:?}");
}

#[test]
fn a_link_that_names_the_launcher_without_resolving_keeps_the_launcher_and_its_record() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    let record: PathBuf = record_path(&launcher);
    // A copied checkout's link: it names that checkout's launcher, and that checkout is gone.
    let gone: PathBuf = rig.scope.root.join("gone-checkout").join(".cache").join("toolchain-launchers").join(RENAMED_BIN);
    let link: PathBuf = make_link(&rig, RENAMED_BIN, &gone);
    // The farm cannot be written, so the link cannot be removed in this pass.
    fs::set_permissions(&rig.scope.exposure_dir, fs::Permissions::from_mode(0o555)).expect("make the farm unwritable");

    let first = retire(&rig);

    fs::set_permissions(&rig.scope.exposure_dir, fs::Permissions::from_mode(0o755)).expect("make the farm writable again");
    assert!(exists_at(&link), "the link could not be removed: {first:?}");
    assert!(exists_at(&launcher) && exists_at(&record), "and the launcher the link names stays, with the record that proves it: {first:?}");

    // The next pass can still show the link was this tool's, and finishes the cleanup.
    let second = retire(&rig);
    assert!(!exists_at(&link), "the link is retired once the farm is writable: {second:?}");
    assert!(!exists_at(&launcher) && !exists_at(&record));
}

//region The seventh review: a second name of one entry converges in one pass
// The killed state is the file system's (one entry under two names, whose maker is gone), never a sleep.

#[test]
fn a_second_name_of_one_entry_left_by_a_kill_is_gone_after_one_pass() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    // The state a kill between a hard link and the removal of the other name left, which every version before the one
    // that moves entries with a single no-replace move made on every removal a kill landed in.
    let hidden: PathBuf = launcher.with_file_name(format!(".{RENAMED_BIN}.4294967.0.retired"));
    fs::hard_link(&launcher, &hidden).expect("the second name a kill between the link and the unlink leaves");
    // The lock declares the visible launcher, so the pass keeps it and the repair has the duplicate to clean.
    let mut declared = rig.lock.clone();
    declared
        .tools
        .iter_mut()
        .find(|tool| return tool.declaration.id == SHADOW_TOOL)
        .expect("the sample lock holds the tool")
        .declaration
        .bins
        .push(RENAMED_BIN.to_string());
    fs::write(&rig.scope.lock_file, render_lock(&declared)).expect("declare the visible launcher");

    let first: LauncherNotices = retire(&rig);

    assert!(exists_at(&launcher), "the visible entry is kept: {first:?}");
    assert!(!exists_at(&hidden), "one pass removes the second name of the entry that stands: {first:?}");
    assert!(
        first.warnings.iter().any(|line| return line.contains("a second name of the entry that already stands at")),
        "the pass says what it removed: {first:?}"
    );
    assert_eq!(retire(&rig), LauncherNotices::default(), "a later pass has nothing left to say");
}

//endregion

//region The eighth review: a link while its launcher is retired, a replacement at a hidden name, and an unreadable record entry
// Every second actor is placed by the code's own seam or is the file system's state, never by a sleep; the second
// install's step is the code's own step, not a bare `symlink`.

#[test]
fn a_retirement_between_a_placement_and_its_link_is_never_left_dangling() {
    use std::sync::{Arc, Mutex};

    let rig = rig_with_installed_shadow();
    let tool = rig.tool(SHADOW_TOOL);
    let name: &str = DECLARED_BIN;
    let link: PathBuf = rig.scope.exposure_dir.join(name);
    let launcher: PathBuf = rig.scope.launcher_dir.join(name);
    let scope = rig.scope.clone();
    let env = rig.env.clone();
    // Install A's lock (the rig's, in memory) declares this bin; the lock on disk no longer does, so install B's
    // retirement of the launcher is the one that would take it away.
    let mut without_the_bin = rig.lock.clone();
    let declared = &mut without_the_bin
        .tools
        .iter_mut()
        .find(|each| return each.declaration.id == SHADOW_TOOL)
        .expect("the sample lock holds the tool")
        .declaration;
    declared.bins.clear();
    declared.options.postinstall = None;
    fs::write(&rig.scope.lock_file, render_lock(&without_the_bin)).expect("publish the lock that dropped the bin");
    let removal: Arc<Mutex<LauncherNotices>> = Arc::new(Mutex::new(LauncherNotices::default()));
    let removal_in_hook: Arc<Mutex<LauncherNotices>> = Arc::clone(&removal);
    // Install B's whole retirement runs between install A's placement of the launcher (and its record) and install A's
    // link: the order the eighth review reproduced. In this code those two instructions are one claimed section, so the
    // retirement waits for the claim and leaves the name to the next install.
    super::exposure::seam::set_before_the_link(Some(Box::new(move || {
        let runner = super::fakes::FakeRunner::new();
        let http = super::fakes::FakeHttp::new();
        let host = super::context::Host { platform: "linux-x64".to_string(), family: Some(super::model::OsFamily::Linux) };
        let tools: Vec<super::model::LockedTool> = Vec::new();
        let context = super::context::Context { scope: &scope, env: &env, runner: &runner, http: &http, host: &host, tools: &tools };
        let notices = retire_undeclared_launchers(&context, &[], true);
        *removal_in_hook.lock().expect("the notices are not poisoned") = notices;
    })));

    let exposure = super::exposure::rebuild_exposure(&rig.context(), &[&tool]);
    super::exposure::seam::set_before_the_link(None);

    let pass = exposure.expect("install A's link pass runs");
    let retirement: LauncherNotices = removal.lock().expect("the notices are not poisoned").clone();
    assert!(
        !(exists_at(&link) && !exists_at(&launcher)),
        "install A's link dangles: a retirement took the launcher between the placement and the link; \
         pass: {pass:?}, retirement: {retirement:?}"
    );
}

#[test]
fn an_install_holds_the_claim_and_a_second_install_leaves_the_link_to_the_next_one() {
    let rig = rig_with_installed_shadow();
    let tool = rig.tool(SHADOW_TOOL);
    let name: &str = DECLARED_BIN;
    fs::create_dir_all(&rig.scope.launcher_dir).expect("make the launcher folder");
    // Another install is inside its launcher-and-link section for this folder.
    let held = super::launcher::claim_launcher_names(&rig.scope).expect("the claim is tested").expect("the claim is free");

    let report = super::exposure::rebuild_exposure(&rig.context(), &[&tool]).expect("the exposure pass runs");

    drop(held);
    assert!(
        !exists_at(&rig.scope.exposure_dir.join(name)),
        "a link made without the claim could point at a launcher the other install is retiring: {report:?}"
    );
    assert!(
        report.warnings.iter().any(|line| return line.contains("holds the claim on this launcher folder's names")),
        "the pass says why the link is left to the next install: {report:?}"
    );
}

#[test]
fn a_file_renamed_onto_a_hidden_name_is_not_unlinked_by_the_repair() {
    let rig = Rig::new();
    fs::create_dir_all(&rig.scope.launcher_dir).expect("make the launcher folder");
    let target: PathBuf = rig.scope.launcher_dir.join("entry");
    let hidden: PathBuf = rig.scope.launcher_dir.join(".entry.4294967.0.retired");
    fs::write(&target, "the killed run's original entry").expect("write the visible entry");
    fs::hard_link(&target, &hidden).expect("make the killed run's second name");
    let replacement: PathBuf = rig.scope.launcher_dir.join("replacement");
    fs::write(&replacement, "a different file of a person's").expect("write the replacement");
    let replacement_in_hook: PathBuf = replacement.clone();
    let hidden_in_hook: PathBuf = hidden.clone();
    // The replacement lands between the repair's look at the hidden name and the repair's unlink of it.
    super::removal::seam::set_before_the_move(Some(Box::new(move || {
        fs::rename(&replacement_in_hook, &hidden_in_hook).expect("another command renames its file onto the hidden name");
    })));
    let mut notices: LauncherNotices = LauncherNotices::default();
    super::stale_launchers::repair_what_a_kill_left(&rig.context(), &mut notices);
    super::removal::seam::set_before_the_move(None);

    assert_eq!(
        fs::read_to_string(&hidden).expect("the file renamed onto the hidden name must survive"),
        "a different file of a person's",
        "final state: {notices:?}"
    );
}

#[test]
fn a_record_entry_that_cannot_be_read_keeps_the_launcher_its_link_names() {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    let old_directory: PathBuf = rig.scope.root.join(".cache").join("old-links");
    fs::create_dir_all(&old_directory).expect("make the dropped postinstall directory");
    let link: PathBuf = old_directory.join("old-alias");
    symlink(&launcher, &link).expect("make the link that only the damaged record names");
    let record: PathBuf = super::postinstall_links::record_file(&rig.context());
    fs::write(&record, format!("{{\"links\":{{\"{}\":7}}}}", link.display())).expect("write a record with a non-path value");

    let link_set_known: bool = super::postinstall_links::record_is_readable(&rig.context());
    let notices: LauncherNotices = retire_undeclared_launchers(&rig.context(), &[], link_set_known);

    assert!(!link_set_known, "a record with an entry it cannot read names an unknown set of links: {notices:?}");
    assert!(exists_at(&link), "the fixture's link must remain: {notices:?}");
    assert!(
        exists_at(&launcher),
        "a record entry that cannot be read must keep the launcher its link names; final state: {notices:?}"
    );
}

//endregion

//region The ninth review: every folder a link can live in, a person's alias, and the record's read bound

/// A rig with a stale launcher and its record, and the postinstall record readable, so retirement may retire it.
fn rig_with_a_stale_launcher() -> (Rig, PathBuf) {
    let rig = rig_with_installed_shadow();
    let launcher: PathBuf = make_launcher(&rig, RENAMED_BIN, SHADOW_TOOL);
    fs::write(rig.scope.launcher_dir.join(".postinstall-links.json"), "{\"links\":{}}\n").expect("write a readable link record");
    return (rig, launcher);
}

/// One state a folder of links can be in, and whether the launcher must survive it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LinkFolderState {
    /// A link stands in a folder that can be listed and cannot be written: the link is seen and cannot go, so the
    /// launcher stays.
    ListedWithALink,
    /// No folder holds a link: the launcher goes.
    Absent,
    /// A link stands in a folder that cannot be listed: this pass cannot tell, so the launcher stays.
    UnlistableWithALink,
}

#[test]
fn a_folder_of_links_that_cannot_be_listed_keeps_the_launcher_and_says_why() {
    for state in [LinkFolderState::ListedWithALink, LinkFolderState::Absent, LinkFolderState::UnlistableWithALink] {
        let (rig, launcher) = rig_with_a_stale_launcher();
        let folder: PathBuf = rig.scope.root.join(".cache").join("old-bin");
        let link: Option<PathBuf> = match state {
            LinkFolderState::ListedWithALink | LinkFolderState::UnlistableWithALink => {
                fs::create_dir_all(&folder).expect("make the link folder");
                let link: PathBuf = folder.join("alias");
                symlink(&launcher, &link).expect("make the link the folder holds");
                Some(link)
            }
            LinkFolderState::Absent => None,
        };
        let mode: u32 = match state {
            LinkFolderState::ListedWithALink => 0o555,
            LinkFolderState::Absent => 0o755,
            LinkFolderState::UnlistableWithALink => 0o000,
        };
        if link.is_some() {
            fs::set_permissions(&folder, std::os::unix::fs::PermissionsExt::from_mode(mode)).expect("give the folder its mode");
        }

        let notices: LauncherNotices = retire(&rig);

        let _ = fs::set_permissions(&folder, std::os::unix::fs::PermissionsExt::from_mode(0o755));
        let launcher_survives: bool = exists_at(&launcher);
        let must_survive: bool = match state {
            LinkFolderState::Absent => false,
            LinkFolderState::ListedWithALink | LinkFolderState::UnlistableWithALink => true,
        };
        assert_eq!(
            launcher_survives, must_survive,
            "folder state {:?}: the launcher's survival must follow what this pass could find out; {notices:?}",
            state
        );
        if let (LinkFolderState::UnlistableWithALink, Some(kept)) = (&state, &link) {
            assert!(exists_at(kept), "the pass cannot remove a link it cannot see: {notices:?}");
            assert!(
                notices.warnings.iter().any(|line| return line.contains("cannot be listed")),
                "the sentence names the folder it could not list: {notices:?}"
            );
        }
        if let (LinkFolderState::ListedWithALink, Some(kept)) = (&state, &link) {
            assert!(exists_at(kept), "a link the folder does not let this pass remove stays: {notices:?}");
        }
    }
}

#[test]
fn a_link_no_record_names_is_a_person_s_and_is_left_alone() {
    let (rig, launcher) = rig_with_a_stale_launcher();
    fs::create_dir_all(&rig.scope.exposure_dir).expect("make the farm");
    let farm_alias: PathBuf = rig.scope.exposure_dir.join("a-person-s-alias");
    symlink(&launcher, &farm_alias).expect("a person makes an alias in the farm");
    let bin_folder: PathBuf = rig.scope.root.join(".cache").join("a-tool-bin");
    fs::create_dir_all(&bin_folder).expect("make a tool's link folder");
    let tool_alias: PathBuf = bin_folder.join("another-alias");
    symlink(&launcher, &tool_alias).expect("a person makes an alias in a tool's folder");

    let notices: LauncherNotices = retire(&rig);

    assert!(exists_at(&farm_alias), "a link no record names is a person's, and it stays: {notices:?}");
    assert!(exists_at(&tool_alias), "a link no record names is a person's, and it stays: {notices:?}");
}

#[test]
fn a_launcher_record_past_the_read_bound_is_refused_as_unreadable() {
    let (rig, launcher) = rig_with_a_stale_launcher();
    let record: PathBuf = record_path(&launcher);
    let past_the_bound: fs::File = fs::File::create(&record).expect("replace the record with a larger file");
    // Sparse: past the bound without costing the disk it names.
    past_the_bound.set_len(super::ordinary_file::MAX_ORDINARY_BYTES + 1).expect("make the record past the read bound");
    drop(past_the_bound);

    let notices: LauncherNotices = retire(&rig);

    assert!(exists_at(&launcher), "a record this tool cannot read keeps its launcher: {notices:?}");
    assert!(
        notices.warnings.iter().any(|line| return line.contains("larger than the")),
        "the sentence names the bound the read stopped at: {notices:?}"
    );
}

//endregion
