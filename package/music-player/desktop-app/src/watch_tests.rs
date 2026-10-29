// What:     Integration test for `watch.rs`, pulled in by
//           `#[cfg(test)] #[path = "watch_tests.rs"] mod tests;` at the bottom of
//           `watch.rs`. Compiles only under `cargo nextest run` / `cargo test`.
// Why:      Cross the real notify->callback boundary: a SourceWatcher over a throwaway
//           directory must call its change callback when a file appears, proving the live
//           watcher actually fires (not just that it compiles).

// What:     `use super::*;` bring the parent `watch` module's items (SourceWatcher,
//           Duration) into scope.
// Why:      The test constructs a `SourceWatcher` and uses `Duration`.
use super::*;

// What:     `use std::sync::mpsc;`. Channel module.
// Why:      The change callback signals the test thread over a channel.
use std::sync::mpsc;

// What:     `use std::time::{SystemTime, UNIX_EPOCH};`. Clock and epoch reference.
// Why:      Build a unique throwaway directory name.
use std::time::{SystemTime, UNIX_EPOCH};

// What:     `#[test] fn watcher_fires_on_file_creation()`. Watch a temp dir and assert the
//           callback runs after a file is created in it.
// Why:      Verify the notify + debouncer pipeline delivers a change to our callback.
#[test]
fn watcher_fires_on_file_creation() {
    // What:     `let nanos = ...as_nanos();`. Unique suffix for the throwaway directory.
    // Why:      Isolate this run's scratch root (THR: verify on a throwaway).
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    // What:     `let dir = crate::test_scratch::scratch_root().join(format!("mp_watch_{nanos}"));`. The dir
    //           to watch.
    // Why:      A real directory the OS watcher can observe.
    let dir = crate::test_scratch::scratch_root().join(format!("mp_watch_{nanos}"));
    // What:     Create the source root before watching it.
    // Why:      The watch target must exist.
    std::fs::create_dir_all(&dir).expect("watch fixture directory should be created");

    // What:     `let (tx, rx) = mpsc::channel::<()>();`. A unit-signal channel.
    // Why:      The callback sends `()` so the test can wait for a change.
    let (tx, rx) = mpsc::channel::<()>();
    // What:     `let mut watcher = SourceWatcher::new(move |_change| { let _ = tx.send(()); }).expect("watcher");`.
    //           Build the watcher with a callback that signals the channel; `.expect` fails
    //           the test if the OS watcher cannot start.
    // Why:      Exercise the real constructor and handler wiring.
    let mut watcher = SourceWatcher::new(move |_change| {
        let _ = tx.send(());
    })
    .expect("OS watcher should start");
    // What:     `watcher.watch(&dir);`. Start watching the throwaway directory.
    // Why:      Arm the watch before creating a file.
    watcher.watch(&dir);
    assert!(watcher.wait_until_registered(1, Duration::from_secs(10)), "the watch should be registered");

    // What:     `std::thread::sleep(Duration::from_millis(ARM_SLACK_MS));`. Brief pause so the
    //           inotify/FSEvents watch is fully armed before the write.
    // Why:      A file created in the gap between watch() and the kernel arming the watch can
    //           be missed; this avoids that race in the test.
    const ARM_SLACK_MS: u64 = 200;
    std::thread::sleep(Duration::from_millis(ARM_SLACK_MS));
    // What:     Create an audio file in the watched directory.
    // Why:      The change the watcher should report.
    std::fs::write(dir.join("new.flac"), b"x").expect("watched fixture should be written");

    // What:     `let got = rx.recv_timeout(Duration::from_secs(WAIT_SECS));`. Wait (bounded)
    //           for the callback. The debounce window is 500ms; the timeout is generous.
    // Why:      A bounded wait is the correct way to await an inherently async OS event.
    const WAIT_SECS: u64 = 10;
    let got = rx.recv_timeout(Duration::from_secs(WAIT_SECS));
    // What:     `assert!(got.is_ok(), ...)`. The callback must have fired.
    // Why:      Proves the notify -> debouncer -> callback path delivers a change.
    assert!(
        got.is_ok(),
        "expected a change callback after creating a file in the watched root"
    );

    // What:     `drop(watcher);` then `std::fs::remove_dir_all(&dir).ok();`. Stop the watcher
    //           and clean up the scratch directory.
    // Why:      Release the OS watch and leave no test droppings.
    drop(watcher);
    std::fs::remove_dir_all(&dir).ok();
}

// What:     Reading a watched directory does not request a rescan, and a real change afterwards does.
// Why:      The rescan's own walk reads every directory. When reads counted as changes each rescan requested another,
//           and at one million tracks that kept a core busy. The change at the end is the positive control: it proves
//           the watcher was able to report anything at all during the quiet period.
#[test]
fn reading_a_watched_directory_is_not_a_change_but_writing_is() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let dir = crate::test_scratch::scratch_root().join(format!("mp_watch_reads_{nanos}"));
    std::fs::create_dir_all(dir.join("sub")).expect("watch fixture directory should be created");
    std::fs::write(dir.join("sub").join("a.flac"), b"x").expect("fixture file should be written");
    let (tx, rx) = mpsc::channel::<()>();
    let mut watcher = SourceWatcher::new(move |_change| {
        let _ = tx.send(());
    })
    .expect("OS watcher should start");
    watcher.watch(&dir);
    assert!(watcher.wait_until_registered(1, Duration::from_secs(10)), "the watch should be registered");
    assert!(rx.recv_timeout(Duration::from_secs(10)).is_ok(), "the registration asks for one catch-up rescan");
    for _ in 0..3 {
        let walked: usize = std::fs::read_dir(&dir).expect("root reads").count() + std::fs::read_dir(dir.join("sub")).expect("subfolder reads").count();
        assert_eq!(walked, 2);
        let contents: Vec<u8> = std::fs::read(dir.join("sub").join("a.flac")).expect("file reads");
        assert_eq!(contents, b"x");
    }
    let during_reads = rx.recv_timeout(Duration::from_millis(2 * DEBOUNCE_MS + 500));
    assert!(during_reads.is_err(), "reading directories and files must not request a rescan, but one arrived: {during_reads:?}");
    std::fs::write(dir.join("sub").join("b.flac"), b"y").expect("watched fixture should be written");
    assert!(rx.recv_timeout(Duration::from_secs(10)).is_ok(), "a created file must request a rescan");
    drop(watcher);
    std::fs::remove_dir_all(&dir).ok();
}

// What:     A burst of changes requests one rescan, not one per change.
// Why:      Copying an album writes many files; each rescan walks the whole library.
#[test]
fn a_burst_of_changes_requests_one_rescan() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let dir = crate::test_scratch::scratch_root().join(format!("mp_watch_burst_{nanos}"));
    std::fs::create_dir_all(&dir).expect("watch fixture directory should be created");
    let (tx, rx) = mpsc::channel::<()>();
    let mut watcher = SourceWatcher::new(move |_change| {
        let _ = tx.send(());
    })
    .expect("OS watcher should start");
    watcher.watch(&dir);
    assert!(watcher.wait_until_registered(1, Duration::from_secs(10)), "the watch should be registered");
    assert!(rx.recv_timeout(Duration::from_secs(10)).is_ok(), "the registration asks for one catch-up rescan");
    for number in 0..20 {
        std::fs::write(dir.join(format!("{number}.flac")), b"x").expect("burst fixture should be written");
    }
    assert!(rx.recv_timeout(Duration::from_secs(10)).is_ok(), "the burst must request a rescan");
    assert!(rx.recv_timeout(Duration::from_millis(2 * DEBOUNCE_MS)).is_err(), "and only one");
    drop(watcher);
    std::fs::remove_dir_all(&dir).ok();
}

// What:     Registering a watch is followed by exactly one rescan request, with no change made.
// Why:      A change made while the registration was still running is not seen by the watch; at one million tracks the
//           registration ends long after the scan, so the one rescan catches up. A second request would mean the
//           rescan's own reads are being counted as changes again.
#[test]
fn a_registration_is_followed_by_exactly_one_catch_up_rescan() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let dir = crate::test_scratch::scratch_root().join(format!("mp_watch_catchup_{nanos}"));
    std::fs::create_dir_all(dir.join("sub")).expect("watch fixture directory should be created");
    std::fs::write(dir.join("sub").join("a.flac"), b"x").expect("fixture file should be written");
    let (tx, rx) = mpsc::channel::<()>();
    let mut watcher = SourceWatcher::new(move |_change| {
        let _ = tx.send(());
    })
    .expect("OS watcher should start");
    watcher.watch(&dir);
    assert!(rx.recv_timeout(Duration::from_secs(10)).is_ok(), "one catch-up rescan follows the registration");
    // The rescan this triggers reads every directory, as the real one does.
    assert_eq!(std::fs::read_dir(dir.join("sub")).expect("subfolder reads").count(), 1);
    let second = rx.recv_timeout(Duration::from_millis(2 * DEBOUNCE_MS + 500));
    assert!(second.is_err(), "and no second one, but this arrived: {second:?}");
    drop(watcher);
    std::fs::remove_dir_all(&dir).ok();
}

// What:     `fn watched(tag, files) -> (PathBuf, SourceWatcher, Receiver<Change>)`. A throwaway library with the given
//           files, a watcher on it, and the changes it reports, after the catch-up that follows the registration.
// Why:      The tests below read what the real notify backend names for each kind of change.
fn watched(tag: &str, files: &[&str]) -> (std::path::PathBuf, SourceWatcher, mpsc::Receiver<Change>) {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock should be after Unix epoch").as_nanos();
    let dir = crate::test_scratch::scratch_root().join(format!("mp_watch_{tag}_{nanos}"));
    for file in files {
        std::fs::create_dir_all(dir.join(file).parent().expect("a file has a folder")).expect("watch fixture directory should be created");
        std::fs::write(dir.join(file), b"first").expect("fixture file should be written");
    }
    let (tx, rx) = mpsc::channel::<Change>();
    let mut watcher = SourceWatcher::new(move |change| {
        let _ = tx.send(change);
    })
    .expect("OS watcher should start");
    watcher.watch(&dir);
    assert!(watcher.wait_until_registered(1, Duration::from_secs(10)), "the watch should be registered");
    assert_eq!(rx.recv_timeout(Duration::from_secs(10)), Ok(Change::Everything), "the registration asks for one catch-up of everything");
    return (dir, watcher, rx);
}

// What:     `assert_names_only_plain_writes_to(named, files)` checks that a report names each of `files` as a plain write and
//           nothing else. On Windows the report may also name a folder that holds one of the files, as a plain write too,
//           because Windows reports a folder whose entries were just written as modified; elsewhere the report is exactly
//           the files.
// Why:      The refresh looks at the named paths only, so a stray structural entry would read a whole folder.
fn assert_names_only_plain_writes_to(named: &[(std::path::PathBuf, bool)], files: &[std::path::PathBuf]) {
    if !cfg!(windows) {
        let expected: Vec<(std::path::PathBuf, bool)> = files.iter().map(|file| return (file.clone(), false)).collect();
        assert_eq!(named, expected.as_slice());
        return;
    }
    for file in files {
        assert!(named.contains(&(file.clone(), false)), "{} is named as a plain write: {named:?}", file.display());
    }
    for (path, structural) in named {
        assert!(!structural, "{} is a plain write, not structural: {named:?}", path.display());
        assert!(files.iter().any(|file| return file.starts_with(path)), "{} is one of the files or a folder holding one: {named:?}", path.display());
    }
}

// What:     A file written in place is reported by its own path and as a write, not as everything.
// Why:      D113: the refresh looks at the named path only, so one replaced file must not name the library.
#[test]
fn a_file_written_in_place_is_reported_by_its_own_path() {
    let (dir, watcher, rx) = watched("write", &["sub/a.flac", "sub/b.flac"]);
    std::fs::write(dir.join("sub").join("a.flac"), b"second, longer").expect("the file is replaced");
    let change = rx.recv_timeout(Duration::from_secs(10)).expect("a change is reported");
    let Change::Paths(set) = change else { panic!("a write is not everything: {change:?}") };
    assert_names_only_plain_writes_to(&set.sorted(), &[dir.join("sub").join("a.flac")]);
    drop(watcher);
    std::fs::remove_dir_all(&dir).ok();
}

// What:     A file replaced by a rename is reported by the target path, as an entry that appeared.
// Why:      Editors and downloaders write a temporary file and rename it over the target.
#[test]
fn a_file_replaced_by_a_rename_is_reported_by_the_target_path() {
    let (dir, watcher, rx) = watched("rename", &["a.flac"]);
    std::fs::write(dir.join("a.flac.tmp"), b"replacement").expect("the temporary file is written");
    std::fs::rename(dir.join("a.flac.tmp"), dir.join("a.flac")).expect("renamed over the target");
    let change = rx.recv_timeout(Duration::from_secs(10)).expect("a change is reported");
    let Change::Paths(set) = change else { panic!("a rename is not everything: {change:?}") };
    let named = set.sorted();
    assert!(named.contains(&(dir.join("a.flac"), true)), "the target is named: {named:?}");
    drop(watcher);
    std::fs::remove_dir_all(&dir).ok();
}

// What:     A folder that is created and filled is reported by the folder, and a file removed by its path.
// Why:      The watch of a new folder does not exist yet when its files are written, so the folder is what is named.
#[test]
fn a_new_folder_and_a_removed_file_are_reported_as_entries_that_appeared_and_went() {
    let (dir, watcher, rx) = watched("folder", &["a.flac", "b.flac"]);
    std::fs::create_dir_all(dir.join("album")).expect("the folder is created");
    std::fs::write(dir.join("album").join("x.flac"), b"x").expect("the file is written");
    std::fs::remove_file(dir.join("b.flac")).expect("the file is removed");
    let change = rx.recv_timeout(Duration::from_secs(10)).expect("a change is reported");
    let Change::Paths(set) = change else { panic!("not everything: {change:?}") };
    let named = set.sorted();
    assert!(named.contains(&(dir.join("album"), true)), "the new folder is named: {named:?}");
    assert!(named.contains(&(dir.join("b.flac"), true)), "the removed file is named: {named:?}");
    assert!(!named.iter().any(|(path, _)| return path.file_name().is_some_and(|name| return name == "a.flac")), "an untouched file is not named");
    drop(watcher);
    std::fs::remove_dir_all(&dir).ok();
}

// What:     Reading the files and folders of a watched library reports nothing, and a write afterwards does.
// Why:      The refresh's own reads must never request another refresh; the write at the end is the positive control.
#[test]
fn reads_report_no_change_but_the_write_after_them_does() {
    let (dir, watcher, rx) = watched("reads", &["sub/a.flac"]);
    for _ in 0..3 {
        assert_eq!(std::fs::read_dir(dir.join("sub")).expect("reads").count(), 1);
        assert_eq!(std::fs::read(dir.join("sub").join("a.flac")).expect("reads"), b"first");
        assert!(std::fs::symlink_metadata(dir.join("sub").join("a.flac")).is_ok());
    }
    let during_reads = rx.recv_timeout(Duration::from_millis(2 * DEBOUNCE_MS + 500));
    assert!(during_reads.is_err(), "reads are not changes, but this arrived: {during_reads:?}");
    std::fs::write(dir.join("sub").join("a.flac"), b"changed").expect("the file is written");
    assert!(matches!(rx.recv_timeout(Duration::from_secs(10)), Ok(Change::Paths(_))));
    drop(watcher);
    std::fs::remove_dir_all(&dir).ok();
}

// What:     A burst of writes to the same few files is one report that names each file once.
// Why:      The debounce merges the burst into the union of what was named.
#[test]
fn a_burst_of_writes_is_one_report_naming_each_file_once() {
    let (dir, watcher, rx) = watched("burst", &["a.flac", "b.flac"]);
    for round in 0..50 {
        std::fs::write(dir.join("a.flac"), format!("round {round}")).expect("written");
        std::fs::write(dir.join("b.flac"), format!("round {round}")).expect("written");
    }
    let Ok(Change::Paths(set)) = rx.recv_timeout(Duration::from_secs(10)) else { panic!("paths are reported") };
    assert_names_only_plain_writes_to(&set.sorted(), &[dir.join("a.flac"), dir.join("b.flac")]);
    assert!(rx.recv_timeout(Duration::from_millis(2 * DEBOUNCE_MS)).is_err(), "and only one report");
    drop(watcher);
    std::fs::remove_dir_all(&dir).ok();
}
