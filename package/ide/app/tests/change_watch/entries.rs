//! Entry changes invalidate the watched directory that lists them;
//!  collapse removes the kernel watch.

/// Waits,
///  kernel probes,
///  and fixture helpers shared by this crate.
use super::support::{Arrival, kernel_watches, not_watching, quiet, set, settle, start, watching};
/// The watcher under test,
///  and how it classifies the displayed file's changes.
use ide_app::change_watch::{ChangeWatcher, SourceChange};
/// What:
///  `PermissionsExt` adds `from_mode`,
///  the Unix way to build permission bits from an octal number.
/// Why:
///  A mode change on a watched folder is one of the reported changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { chmodSync } from 'node:fs';
/// ```
use std::os::unix::fs::PermissionsExt;
/// Fixture writes and renames happen through the ordinary filesystem API.
use std::{fs, path::Path};

/// Require a report for `folder` without a full reread,
///  after the caller drained earlier notifications and made one filesystem change.
///
/// What:
///  The caller drains with `quiet` and makes the change itself before calling this.
/// Why:
///  Each filesystem step is checked in isolation from the notifications of the previous step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function expectReport(watcher, folder, name) { ... }
/// ```
fn expect_report(watcher: &mut ChangeWatcher, folder: &Path, name: &str) {
    let record = {
        let mut arrival = Arrival::begin(watcher, name);
        while !(arrival.record.directories.contains(folder)) {
            arrival.again(watcher);
        }
        arrival.into_record()
    };
    assert!(
        !record.everything,
        "{name} requested a full reread: {record:?}"
    );
}

/// Create,
///  rename,
///  and remove inside a watched folder each report that folder.
#[test]
fn created_renamed_and_removed_entries_invalidate_their_folder() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let folder = root.join("folder");
    fs::create_dir(&folder).expect("fixture folder");
    watcher.watch_only(&set(&[&root, &folder]), None);
    settle(&mut watcher, &[&root, &folder]);
    quiet(&mut watcher);
    {
        fs::write(folder.join("a.txt"), "a").expect("create");
    }
    expect_report(&mut watcher, &folder, "a created file");
    quiet(&mut watcher);
    {
        fs::rename(folder.join("a.txt"), folder.join("b.txt")).expect("rename");
    }
    expect_report(&mut watcher, &folder, "a renamed file");
    quiet(&mut watcher);
    {
        fs::remove_file(folder.join("b.txt")).expect("remove");
    }
    expect_report(&mut watcher, &folder, "a removed file");
    quiet(&mut watcher);
    {
        fs::create_dir(folder.join("nested")).expect("create folder");
    }
    expect_report(&mut watcher, &folder, "a created folder");
}

/// A permission change on a watched folder reports that folder,
///  because it decides whether it can be listed.
#[test]
fn a_permission_change_on_a_watched_folder_reports_it() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let folder = root.join("folder");
    fs::create_dir(&folder).expect("fixture folder");
    watcher.watch_only(&set(&[&root, &folder]), None);
    settle(&mut watcher, &[&root, &folder]);
    quiet(&mut watcher);
    {
        // What: `Permissions::from_mode(0o700)` builds Unix permission bits; `0o` marks an octal literal.
        // Why: Any mode change makes inotify report an attribute event for the folder itself.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // chmodSync(folder, 0o700);
        // ```
        fs::set_permissions(&folder, fs::Permissions::from_mode(0o700)).expect("change mode");
    }
    expect_report(&mut watcher, &folder, "a permission change");
}

/// Moving a file into and out of a watched folder reports both folders.
#[test]
fn moves_into_and_out_of_a_watched_folder_report_both_folders() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let folder = root.join("folder");
    fs::create_dir(&folder).expect("fixture folder");
    fs::write(root.join("outer.txt"), "outer").expect("outer file");
    watcher.watch_only(&set(&[&root, &folder]), None);
    settle(&mut watcher, &[&root, &folder]);
    fs::rename(root.join("outer.txt"), folder.join("outer.txt")).expect("move in");
    {
        let mut arrival = Arrival::begin(&mut watcher, "a move into the folder, for both folders");
        while !(arrival.record.directories.contains(&folder)
            && arrival.record.directories.contains(&root))
        {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    quiet(&mut watcher);
    fs::rename(folder.join("outer.txt"), root.join("outer.txt")).expect("move out");
    {
        let mut arrival =
            Arrival::begin(&mut watcher, "a move out of the folder, for both folders");
        while !(arrival.record.directories.contains(&folder)
            && arrival.record.directories.contains(&root))
        {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
}

/// A new watch reports its folder once,
///  so a change made before the watch existed is reread.
/// The displayed file's directory also reports the displayed file as changed.
#[test]
fn a_new_watch_reports_its_folder_and_the_displayed_file_once() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let folder = root.join("folder");
    fs::create_dir(&folder).expect("fixture folder");
    let file = folder.join("view.txt");
    fs::write(&file, "view").expect("displayed file");
    watcher.watch_only(&set(&[&root]), None);
    let first = {
        let mut arrival = Arrival::begin(&mut watcher, "the root's new watch");
        while !(arrival.record.directories.contains(&root)) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    assert_eq!(first.source, None, "no file is displayed yet");
    // The displayed file's directory is watched even though the tree does not show it.
    watcher.watch_only(&set(&[&root]), Some(&file));
    let second = {
        let mut arrival = Arrival::begin(&mut watcher, "the displayed file's new directory watch");
        while !(arrival.record.directories.contains(&folder) && arrival.record.source.is_some()) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    assert!(watching(&second, &folder));
}

/// Switching the displayed file inside an already watched folder reports the new file once:
/// a change made to it before the switch reached the watcher was not recorded.
#[test]
fn a_newly_displayed_file_in_a_watched_folder_is_reported_once() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let first = root.join("first.txt");
    let second = root.join("second.txt");
    fs::write(&first, "first").expect("first file");
    fs::write(&second, "second").expect("second file");
    watcher.watch_only(&set(&[&root]), Some(&first));
    settle(&mut watcher, &[&root]);
    watcher.watch_only(&set(&[&root]), Some(&second));
    let switched = {
        let mut arrival = Arrival::begin(&mut watcher, "the newly displayed file");
        while !(arrival.record.source.is_some()) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    // No write was seen, so the read it asks for must accept only a file that has been quiet.
    assert_eq!(
        switched.source,
        Some(SourceChange::Reread),
        "the newly displayed file was reported as a finished write"
    );
    // Positive control for the silence that follows: nothing else reports the file again.
    let silent = quiet(&mut watcher);
    assert_eq!(
        silent.source, None,
        "the displayed file was reported again without a change"
    );
}

/// A displayed file whose folder watch succeeds only on a retry is reported when that watch starts.
#[test]
fn a_displayed_file_is_reported_when_its_retried_folder_watch_starts() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let folder = root.join("later");
    let file = folder.join("view.txt");
    watcher.watch_only(&set(&[&root]), Some(&file));
    {
        let mut arrival = Arrival::begin(&mut watcher, "the failed watch of a missing folder");
        while !(arrival.record.everything && watching(&arrival.record, &root)) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    fs::create_dir(&folder).expect("create the folder");
    fs::write(&file, "view").expect("create the displayed file");
    quiet(&mut watcher);
    watcher.retry();
    {
        let mut arrival = Arrival::begin(
            &mut watcher,
            "the displayed file after its folder's retried watch",
        );
        while !(watching(&arrival.record, &folder) && arrival.record.source.is_some()) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
}

/// Collapsing a folder removes its kernel watch;
///  showing it again adds one back.
#[test]
fn collapsing_a_folder_removes_its_kernel_watch() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let folder = root.join("folder");
    fs::create_dir(&folder).expect("fixture folder");
    watcher.watch_only(&set(&[&root, &folder]), None);
    settle(&mut watcher, &[&root, &folder]);
    assert!(
        kernel_watches(&folder),
        "the shown folder has no kernel watch"
    );
    watcher.watch_only(&set(&[&root]), None);
    {
        let mut arrival = Arrival::begin(
            &mut watcher,
            "the collapsed folder's removal from the watched set",
        );
        while !(not_watching(&arrival.record, &folder)) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    assert!(
        !kernel_watches(&folder),
        "the collapsed folder kept its kernel watch"
    );
    assert!(
        kernel_watches(&root),
        "collapsing a folder removed the root's watch"
    );
    fs::write(folder.join("hidden.txt"), "hidden").expect("change inside the collapsed folder");
    let silent = quiet(&mut watcher);
    assert!(
        !silent.directories.contains(&folder),
        "a collapsed folder was reported: {silent:?}"
    );
    watcher.watch_only(&set(&[&root, &folder]), None);
    settle(&mut watcher, &[&root, &folder]);
    assert!(
        kernel_watches(&folder),
        "re-expanding the folder did not watch it again"
    );
}

/// A rule file edited in place reports its folder,
///  although no name changed;
///  an in-place edit of any other file does not.
#[test]
fn an_ignore_rule_file_edited_in_place_reports_its_folder_and_other_files_do_not() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let info = root.join(".git").join("info");
    fs::create_dir_all(&info).expect("git info folder");
    fs::write(root.join(".gitignore"), "*.log\n").expect("rule file");
    fs::write(root.join("other.txt"), "other\n").expect("other file");
    fs::write(info.join("exclude"), "# none\n").expect("exclude file");
    watcher.watch_only(&set(&[&root, &info]), None);
    settle(&mut watcher, &[&root, &info]);
    // An in-place edit of an ordinary file leaves its folder's listing alone.
    fs::write(root.join("other.txt"), "other, edited\n").expect("edit other file");
    let silent = quiet(&mut watcher);
    assert!(
        silent.directories.is_empty() && !silent.everything,
        "a content edit of an ordinary file reported its folder: {silent:?}"
    );
    quiet(&mut watcher);
    {
        fs::write(root.join(".gitignore"), "*.log\n*.tmp\n").expect("edit rule file");
    }
    expect_report(&mut watcher, &root, "an in-place edit of .gitignore");
    quiet(&mut watcher);
    {
        fs::write(info.join("exclude"), "# edited\n").expect("edit exclude file");
    }
    expect_report(&mut watcher, &info, "an in-place edit of info/exclude");
}
