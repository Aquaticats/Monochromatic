//! The event stream is not trusted forever:
//!  overflow,
//!  errors,
//!  and lost or failed watches reread everything shown.

/// Waits,
///  kernel probes,
///  and fixture helpers shared by this crate.
use super::support::{Arrival, kernel_watches, not_watching, quiet, set, settle, start, watching};
/// Fixture files are created and moved through the ordinary filesystem API.
use std::fs;

/// A real inotify queue overflow (`IN_Q_OVERFLOW`,
///  notify's rescan flag) requests a full reread.
#[test]
fn queue_overflow_requests_a_full_reread() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let folder = root.join("burst");
    fs::create_dir(&folder).expect("burst folder");
    watcher.watch_only(&set(&[&root, &folder]), None);
    settle(&mut watcher, &[&root, &folder]);
    let limit: usize = fs::read_to_string("/proc/sys/fs/inotify/max_queued_events")
        .expect("inotify queue limit")
        .trim()
        .parse()
        .expect("numeric queue limit");
    {
        // While paused, notify's thread waits inside the handler and stops draining the kernel queue.
        let _pause = watcher.pause_delivery();
        // Each created and closed file queues create, open, and close-write: three events.
        for index in 0..(limit / 2 + 64) {
            fs::File::create(folder.join(format!("{index}.txt"))).expect("burst file");
        }
    }
    {
        let mut arrival = Arrival::begin(&mut watcher, "an overflowed queue's full reread");
        while !(arrival.record.everything) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
}

/// An error from notify's event stream requests a full reread.
#[test]
fn a_notification_error_requests_a_full_reread() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    watcher.watch_only(&set(&[&root]), None);
    settle(&mut watcher, &[&root]);
    // notify reports read failures of the inotify descriptor this way; they cannot be provoked on demand.
    watcher.deliver(Err(notify::Error::generic(
        "simulated inotify read failure",
    )));
    {
        let mut arrival = Arrival::begin(&mut watcher, "a notification error's full reread");
        while !(arrival.record.everything) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
}

/// Removing a watched folder requests a full reread and leaves it unwatched.
#[test]
fn a_removed_watched_folder_requests_a_full_reread() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let folder = root.join("folder");
    fs::create_dir(&folder).expect("fixture folder");
    watcher.watch_only(&set(&[&root, &folder]), None);
    settle(&mut watcher, &[&root, &folder]);
    fs::remove_dir(&folder).expect("remove watched folder");
    {
        let mut arrival = Arrival::begin(
            &mut watcher,
            "a removed folder's full reread and lost watch",
        );
        while !(arrival.record.everything && not_watching(&arrival.record, &folder)) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
}

/// A renamed watched folder leaves the watched set and requests a full reread,
///  and its new location is
/// not reported under the old name.
///  With the parent watched,
///  notify also drops the kernel watch itself.
#[test]
fn a_renamed_watched_folder_is_not_followed_to_its_new_name() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let folder = root.join("folder");
    let moved = root.join("moved");
    fs::create_dir(&folder).expect("fixture folder");
    watcher.watch_only(&set(&[&root, &folder]), None);
    settle(&mut watcher, &[&root, &folder]);
    fs::rename(&folder, &moved).expect("rename watched folder");
    {
        let mut arrival = Arrival::begin(
            &mut watcher,
            "a renamed folder's full reread and lost watch",
        );
        while !(arrival.record.everything && not_watching(&arrival.record, &folder)) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    assert!(
        !kernel_watches(&moved),
        "the moved folder is still watched under its old name"
    );
    quiet(&mut watcher);
    fs::write(moved.join("inside.txt"), "inside").expect("change in the moved folder");
    let silent = quiet(&mut watcher);
    assert!(
        !silent.directories.contains(&folder),
        "a change in the moved folder was reported under the old name: {silent:?}"
    );
}

/// The displayed file's folder can be watched while its parent is not;
///  renaming it then reports only
/// inotify's move-self event,
///  and notify keeps that watch on the moved directory unless the IDE removes it.
#[test]
fn a_renamed_folder_with_an_unwatched_parent_loses_its_kernel_watch() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let parent = root.join("collapsed");
    let folder = parent.join("folder");
    let moved = parent.join("moved");
    fs::create_dir_all(&folder).expect("nested fixture folder");
    let file = folder.join("view.txt");
    fs::write(&file, "view").expect("displayed file");
    watcher.watch_only(&set(&[&root]), Some(&file));
    settle(&mut watcher, &[&root, &folder]);
    fs::rename(&folder, &moved).expect("rename the displayed file's folder");
    {
        let mut arrival = Arrival::begin(
            &mut watcher,
            "a renamed folder's full reread and lost watch",
        );
        while !(arrival.record.everything && not_watching(&arrival.record, &folder)) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    assert!(
        !kernel_watches(&moved),
        "the moved folder kept a kernel watch that reports under its old name"
    );
}

/// A watch that cannot be added requests a full reread,
///  stays unwatched,
///  and is retried on request.
#[test]
fn a_failed_watch_requests_a_full_reread_and_is_retried_on_request() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let missing = root.join("missing");
    watcher.watch_only(&set(&[&root, &missing]), None);
    let failed = {
        let mut arrival = Arrival::begin(&mut watcher, "a failed watch's full reread");
        while !(arrival.record.everything && watching(&arrival.record, &root)) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    assert!(
        not_watching(&failed, &missing),
        "a missing folder was reported as watched"
    );
    fs::create_dir(&missing).expect("create the missing folder");
    // Without a retry request the failed folder stays on its timer.
    let waiting = quiet(&mut watcher);
    assert!(
        !watching(&waiting, &missing),
        "a failed watch was retried without a request"
    );
    watcher.retry();
    {
        let mut arrival = Arrival::begin(&mut watcher, "the retried watch and its extra read");
        while !(arrival.record.directories.contains(&missing)
            && watching(&arrival.record, &missing))
        {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
}

/// A watch that keeps failing the same way is reported once.
///  Retries,
///  which the native tick requests on
/// every 1 s sweep,
///  report nothing more until the failure changes;
///  here it ends when the folder appears.
#[test]
fn a_watch_failing_the_same_way_is_reported_once_until_it_changes() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let (mut watcher, workspace) = start(fixture.path());
    let root = workspace.root().to_path_buf();
    let missing = root.join("missing");
    watcher.watch_only(&set(&[&root, &missing]), None);
    {
        let mut arrival = Arrival::begin(&mut watcher, "the first failure's full reread");
        while !(arrival.record.everything && watching(&arrival.record, &root)) {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
    quiet(&mut watcher);
    for _ in 0..3 {
        watcher.retry();
        let repeated = quiet(&mut watcher);
        assert!(
            !repeated.everything && repeated.watched.is_none(),
            "an unchanged failure was reported again: {repeated:?}"
        );
    }
    fs::create_dir(&missing).expect("create the missing folder");
    quiet(&mut watcher);
    watcher.retry();
    {
        let mut arrival = Arrival::begin(&mut watcher, "the watch restored by a retry");
        while !(arrival.record.directories.contains(&missing)
            && watching(&arrival.record, &missing))
        {
            arrival.again(&mut watcher);
        }
        arrival.into_record()
    };
}
