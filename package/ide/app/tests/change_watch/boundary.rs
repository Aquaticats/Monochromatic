//! Containment at the project root,
//!  and shutdown that leaves no watch behind.

/// Waits,
///  kernel probes,
///  and fixture helpers shared by this crate.
use super::support::{Arrival, kernel_watches, quiet, set, settle, start, watching};
/// What:
///  notify's event constructors build a synthetic event exactly like the inotify backend's.
/// Why:
///  A path outside the root can only reach the handler through a moved watch;
///  the seam shows the filter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const event = new NotifyEvent({ kind: 'modify-data', paths: [outside] });
/// ```
use notify::{
    Event, EventKind,
    event::{DataChange, ModifyKind},
};
/// What:
///  `std::os::unix::fs::symlink` creates a symbolic link (siblings:
///  `fs::hard_link`,
///  `fs::copy`).
/// Why:
///  A folder inside the root that links outside must not be watched,
///  because inotify follows links.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// symlinkSync(target, link);
/// ```
use std::{
    fs,
    os::unix::fs::symlink,
    time::{Duration, Instant},
};

/// Folders outside the root,
///  directly or through a symbolic link,
///  are refused,
///  and their changes are never reported.
#[test]
fn folders_outside_the_root_are_never_watched_or_reported() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let elsewhere = tempfile::tempdir().expect("disposable outside folder");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let outside = elsewhere
        .path()
        .canonicalize()
        .expect("canonical outside folder");
    let link = root.join("link");
    symlink(&outside, &link).expect("symbolic link to the outside folder");
    // An alias inside the root would report the real folder's changes under a second name.
    let real = root.join("real");
    fs::create_dir(&real).expect("real folder");
    let alias = root.join("alias");
    symlink(&real, &alias).expect("symbolic link inside the root");
    watcher.watch_only(&set(&[&root, &outside, &link, &alias]), None);
    let refused = {
        let mut arrival = Arrival::begin(
            &mut watcher,
            "refused watches falling back to a full reread",
        );
        while !(arrival.record.everything && watching(&arrival.record, &root)) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    let watched = refused.watched.expect("published watched set");
    assert!(
        !watched.contains(&outside),
        "a folder outside the root was watched"
    );
    assert!(
        !watched.contains(&link),
        "a symbolic link leading outside the root was watched"
    );
    assert!(
        !watched.contains(&alias),
        "a symbolic-link alias inside the root was watched"
    );
    assert!(
        !kernel_watches(&real),
        "the alias target was watched through its alias"
    );
    assert!(
        !kernel_watches(&outside),
        "the outside folder has a kernel watch"
    );
    quiet(&mut watcher);
    fs::write(outside.join("outside.txt"), "outside").expect("change outside the root");
    let silent = quiet(&mut watcher);
    assert!(
        silent.directories.is_empty() && !silent.everything,
        "a change outside the root was reported: {silent:?}"
    );
}

/// Even a displayed path outside the root is not reported when an event names it.
#[test]
fn events_naming_paths_outside_the_root_are_ignored() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let elsewhere = tempfile::tempdir().expect("disposable outside folder");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let outside = elsewhere
        .path()
        .canonicalize()
        .expect("canonical outside folder")
        .join("view.txt");
    watcher.watch_only(&set(&[&root]), Some(&outside));
    settle(&mut watcher, &[&root]);
    quiet(&mut watcher);
    let event =
        Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any))).add_path(outside.clone());
    // What: `Ok(event)` wraps the event as notify's handler receives it.
    // Why: The same recording path as real notifications runs, without watching outside the root.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // watcher.deliver(event);
    // ```
    watcher.deliver(Ok(event));
    let silent = quiet(&mut watcher);
    assert_eq!(
        silent.source, None,
        "an event outside the root reached the displayed file: {silent:?}"
    );
}

/// Dropping the watcher joins its thread and removes every kernel watch.
#[test]
fn dropping_the_watcher_removes_its_kernel_watches() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    watcher.watch_only(&set(&[&root]), None);
    settle(&mut watcher, &[&root]);
    assert!(kernel_watches(&root), "the root has no kernel watch");
    drop(watcher);
    // notify's own loop thread is detached; it removes watches and closes its descriptor after Drop.
    let start_time = Instant::now();
    while kernel_watches(&root) {
        assert!(
            start_time.elapsed() < Duration::from_secs(2),
            "a kernel watch outlived the watcher"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// A displayed file outside the root (a go-to-definition target in the standard library) is not a
/// watch failure:
///  its folder is never asked for,
///  so nothing is refused,
///  logged,
///  or reread in full.
#[test]
fn a_displayed_file_outside_the_root_is_not_watched_and_not_a_failure() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let elsewhere = tempfile::tempdir().expect("disposable outside folder");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let outside = elsewhere
        .path()
        .canonicalize()
        .expect("canonical outside folder");
    let file = outside.join("vec.rs");
    fs::write(&file, "pub struct Vec;\n").expect("outside file");
    watcher.watch_only(&set(&[&root]), Some(&file));
    let first = {
        let mut arrival = Arrival::begin(&mut watcher, "the root's watch");
        while !(watching(&arrival.record, &root)) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    let later = quiet(&mut watcher);
    assert!(
        !first.everything && !later.everything,
        "the outside file's folder was treated as a failed watch: {first:?} {later:?}"
    );
    assert!(
        !kernel_watches(&outside),
        "the outside file's folder has a kernel watch"
    );
    watcher.retry();
    let retried = quiet(&mut watcher);
    assert!(
        !retried.everything && retried.watched.is_none(),
        "a retry reported the outside folder: {retried:?}"
    );
}
