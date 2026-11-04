//! Unit tests of the retirement of postinstall links the lock no longer declares:
//! a link declared and then dropped,
//! a link renamed or moved,
//! and a path that now holds something a person put there.
//! Each runs against the fake runner inside a disposable scope.

use std::fs;
use std::path::{Path, PathBuf};

use super::cargo_installs::current_link;
use super::fakes::{Rig, write_executable};
use super::model::{LockedTool, Postinstall, PostinstallLink};
use super::postinstall_links::{LinkNotices, sync_postinstall_links};
use super::sync::{SyncReport, format_sync_report};

/// The sample lock's tool that declares a postinstall link.
const LINKED: &str = "cargo:monochromatic-mpm-shadow";

/// The program the link points at.
const PROGRAM: &str = "mpm-shadow";

/// A rig whose linked tool has its program installed.
fn rig_with_program() -> Rig {
    let rig = Rig::new();
    write_executable(&current_link(&rig.scope.toolchain_root, "monochromatic-mpm-shadow").join("bin").join(PROGRAM));
    return rig;
}

/// The linked tool of the rig's lock.
fn linked_tool(rig: &mut Rig) -> &mut LockedTool {
    return rig.lock.tools.iter_mut().find(|tool| return tool.declaration.id == LINKED).expect("the sample lock holds the tool");
}

/// The link the sample lock declares.
fn declared_link(rig: &mut Rig) -> PostinstallLink {
    let Some(Postinstall::Link(link)) = linked_tool(rig).declaration.options.postinstall.clone() else {
        panic!("the sample tool declares a postinstall link");
    };
    return link;
}

/// Where the declared link lives.
fn link_path(rig: &Rig, link: &PostinstallLink) -> PathBuf {
    return rig.scope.root.join(link.link_dir.as_deref().expect("a link directory")).join(&link.link_name);
}

/// Declares `link` on the linked tool instead of what it declared.
fn declare(rig: &mut Rig, link: Option<PostinstallLink>) {
    linked_tool(rig).declaration.options.postinstall = link.map(Postinstall::Link);
}

/// One postinstall pass over every tool of the rig's lock.
fn sync(rig: &Rig) -> LinkNotices {
    let tools: Vec<&LockedTool> = rig.lock.tools.iter().collect();
    return sync_postinstall_links(&rig.context(), &tools).expect("the pass succeeds");
}

/// Whether anything (even a dangling link) exists at `path`.
fn exists_at(path: &Path) -> bool {
    return fs::symlink_metadata(path).is_ok();
}

#[test]
fn a_link_declared_and_then_dropped_is_removed_by_the_next_install() {
    let mut rig = rig_with_program();
    let link = declared_link(&mut rig);
    let path = link_path(&rig, &link);
    assert!(sync(&rig).retired.is_empty());
    assert!(exists_at(&path), "the first pass makes the link");

    declare(&mut rig, None);
    let notices = sync(&rig);
    assert!(!exists_at(&path), "the dropped link is gone");
    assert_eq!(notices.retired.len(), 1, "{notices:?}");
    assert!(notices.retired[0].contains(&path.display().to_string()), "{notices:?}");
    assert!(notices.warnings.is_empty(), "{notices:?}");
    assert_eq!(sync(&rig), LinkNotices::default(), "a later pass has nothing left to say");
}

#[test]
fn a_renamed_or_moved_link_leaves_no_old_link_behind() {
    let mut rig = rig_with_program();
    let link = declared_link(&mut rig);
    let old_path = link_path(&rig, &link);
    sync(&rig);

    let renamed = PostinstallLink { link_name: format!("{}-renamed", link.link_name), ..link.clone() };
    declare(&mut rig, Some(renamed.clone()));
    let notices = sync(&rig);
    assert!(!exists_at(&old_path), "the old name is retired");
    assert!(exists_at(&link_path(&rig, &renamed)), "the new name is made");
    assert_eq!(notices.retired.len(), 1, "{notices:?}");

    let moved = PostinstallLink { link_dir: Some(format!("{}-moved", link.link_dir.as_deref().expect("a directory"))), ..renamed.clone() };
    declare(&mut rig, Some(moved.clone()));
    sync(&rig);
    assert!(!exists_at(&link_path(&rig, &renamed)), "the old directory's link is retired");
    assert!(exists_at(&link_path(&rig, &moved)), "the link is made in the new directory");
}

#[test]
fn a_persons_file_at_a_dropped_links_path_is_left_untouched_with_a_notice_once() {
    let mut rig = rig_with_program();
    let link = declared_link(&mut rig);
    let path = link_path(&rig, &link);
    sync(&rig);
    fs::remove_file(&path).expect("remove the link");
    fs::write(&path, "a person's own script").expect("write the person's file");

    declare(&mut rig, None);
    let notices = sync(&rig);
    assert_eq!(fs::read_to_string(&path).expect("still there"), "a person's own script");
    assert!(notices.retired.is_empty(), "{notices:?}");
    assert_eq!(notices.warnings.len(), 1, "{notices:?}");
    assert!(notices.warnings[0].contains(&path.display().to_string()) && notices.warnings[0].contains("not a symlink"), "{notices:?}");
    assert_eq!(sync(&rig), LinkNotices::default(), "the notice appears once");
}

#[test]
fn a_symlink_a_person_pointed_elsewhere_is_left_untouched_with_a_notice() {
    let mut rig = rig_with_program();
    let link = declared_link(&mut rig);
    let path = link_path(&rig, &link);
    sync(&rig);
    let elsewhere = rig.scope.root.join("somewhere-else");
    fs::write(&elsewhere, "x").expect("write the target");
    fs::remove_file(&path).expect("remove the link");
    std::os::unix::fs::symlink(&elsewhere, &path).expect("a person's link");

    declare(&mut rig, None);
    let notices = sync(&rig);
    assert_eq!(fs::read_link(&path).expect("still a link"), elsewhere);
    assert_eq!(notices.warnings.len(), 1, "{notices:?}");
    assert!(notices.warnings[0].contains(&elsewhere.display().to_string()), "{notices:?}");
}

#[test]
fn a_link_that_was_never_recorded_is_never_removed() {
    let mut rig = rig_with_program();
    let link = declared_link(&mut rig);
    let path = link_path(&rig, &link);
    sync(&rig);
    // A copy of the situation before the record existed: the same link, no memory of having made it.
    fs::remove_file(rig.scope.launcher_dir.join(".postinstall-links.json")).expect("forget the record");

    declare(&mut rig, None);
    assert_eq!(sync(&rig), LinkNotices::default());
    assert!(exists_at(&path), "a link nothing recorded is not this tool's to remove");
}

#[test]
fn a_pass_over_a_subset_of_tools_keeps_the_links_of_the_others() {
    let mut rig = rig_with_program();
    let link = declared_link(&mut rig);
    let path = link_path(&rig, &link);
    sync(&rig);

    let none: Vec<&LockedTool> = Vec::new();
    let notices = sync_postinstall_links(&rig.context(), &none).expect("the pass succeeds");
    assert_eq!(notices, LinkNotices::default());
    assert!(exists_at(&path), "the lock still declares the link");
    declare(&mut rig, None);
    assert_eq!(sync(&rig).retired.len(), 1, "the record still held it, so the later full pass retires it");
}

#[test]
fn the_install_report_prints_each_retired_link() {
    let report = SyncReport { retired_links: vec!["retired the postinstall link /x/cargo".to_string()], ..SyncReport::default() };
    assert!(format_sync_report(&report).contains("toolchain install: retired the postinstall link /x/cargo\n"));
}
