//! Removal of unused key folders on disposable cache folders,
//!  with injected times.

use super::{MARKER, Sweep, UNUSED_LIMIT, mark_used, remove_unused, unpack_marked};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
    time::{Duration, SystemTime},
};

/// The key of the build running in each test.
const CURRENT: &str = "00000000000000c0";

/// A span of whole days.
fn days(count: u64) -> Duration {
    return Duration::from_secs(count * 86_400);
}

/// Set the modification time of a file or folder,
///  without following a link at `path`'s end.
fn set_time(path: &Path, time: SystemTime) {
    let handle = fs::File::open(path).expect("open to set its time");
    handle.set_modified(time).expect("set modification time");
}

/// A key folder holding a cached library,
///  with its marker last renewed at `used`,
///  or no marker.
fn key_folder(runtime: &Path, key: &str, used: Option<SystemTime>) {
    let folder = runtime.join(key);
    fs::create_dir_all(folder.join("grammars")).expect("key folder");
    fs::write(folder.join("grammars/sql.so"), b"library").expect("cached library");
    if let Some(time) = used {
        fs::write(folder.join(MARKER), b"").expect("marker");
        set_time(&folder.join(MARKER), time);
    }
}

/// Entry names below `runtime`,
///  sorted.
fn names(runtime: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for entry in fs::read_dir(runtime).expect("runtime folder") {
        found.push(
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned(),
        );
    }
    found.sort();
    return found;
}

/// Sorted copy of a list of names,
///  for comparing sweeps regardless of listing order.
fn sorted(list: &[String]) -> Vec<String> {
    let mut copy = list.to_vec();
    copy.sort();
    return copy;
}

#[test]
fn a_folder_of_another_build_unused_for_more_than_30_days_is_removed() {
    let base = tempfile::tempdir().expect("disposable cache");
    let runtime = base.path().join("runtime");
    let now = SystemTime::now();
    key_folder(&runtime, CURRENT, Some(now));
    key_folder(&runtime, "0000000000000031", Some(now - days(31)));
    let sweep = remove_unused(&runtime, CURRENT, now, UNUSED_LIMIT);
    assert_eq!(sweep.removed, vec!["0000000000000031".to_string()]);
    assert_eq!(
        names(&runtime),
        vec![CURRENT.to_string()],
        "the old folder and its temporary removal name are gone"
    );
}

#[test]
fn a_folder_used_within_30_days_is_kept() {
    let base = tempfile::tempdir().expect("disposable cache");
    let runtime = base.path().join("runtime");
    let now = SystemTime::now();
    key_folder(&runtime, "0000000000000001", Some(now - days(1)));
    key_folder(&runtime, "0000000000000029", Some(now - days(29)));
    key_folder(&runtime, "0000000000000030", Some(now - days(30)));
    key_folder(&runtime, "00000000000000ff", Some(now + days(3)));
    let sweep = remove_unused(&runtime, CURRENT, now, UNUSED_LIMIT);
    assert_eq!(sweep.removed, Vec::<String>::new());
    assert_eq!(
        sorted(&sweep.kept),
        vec![
            "0000000000000001".to_string(),
            "0000000000000029".to_string(),
            "0000000000000030".to_string(),
            "00000000000000ff".to_string(),
        ],
        "used within 30 days (exactly 30 counts as within) or, after a clock change, in the future"
    );
    assert!(runtime.join("0000000000000029/grammars/sql.so").is_file());
}

#[test]
fn the_current_key_is_never_removed_however_long_unused() {
    let base = tempfile::tempdir().expect("disposable cache");
    let runtime = base.path().join("runtime");
    let now = SystemTime::now();
    key_folder(&runtime, CURRENT, Some(now - days(400)));
    key_folder(&runtime, "0000000000000400", Some(now - days(400)));
    let sweep = remove_unused(&runtime, CURRENT, now, UNUSED_LIMIT);
    assert_eq!(sweep.kept, vec![CURRENT.to_string()]);
    assert_eq!(sweep.removed, vec!["0000000000000400".to_string()]);
    assert_eq!(
        fs::read(runtime.join(CURRENT).join("grammars/sql.so")).expect("current library"),
        b"library"
    );
}

/// What:
///  The current time as the filesystem stamps it:
///  the modification time of a file written now.
/// Why:
///  The kernel stamps file times from a coarse clock,
///  and `SystemTime::now` reads a finer one,
///      so a file written right after `SystemTime::now` can carry an earlier time.
///  A start time
///      taken from the same file clock is never later than a file written after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fileClockNow(directory: string): Date { writeFileSync(probe, ''); return statSync(probe).mtime; }
/// ```
fn file_clock_now(directory: &Path) -> SystemTime {
    let probe = directory.join("file-clock");
    fs::write(&probe, b"").expect("file clock probe");
    return fs::metadata(&probe)
        .expect("file clock probe")
        .modified()
        .expect("file clock time");
}

#[test]
fn a_marker_renewed_by_a_running_copy_keeps_its_folder() {
    let base = tempfile::tempdir().expect("disposable cache");
    let runtime = base.path().join("runtime");
    let older = "00000000000000a1";
    let started = file_clock_now(base.path());
    key_folder(&runtime, older, Some(started - days(31)));
    // The older copy loads a parser: the marker is renewed before the library is compared.
    let library = unpack_marked(&runtime.join(older), "sql", b"library").expect("parser load");
    assert_eq!(library, runtime.join(older).join("grammars/sql.so"));
    let renewed = fs::metadata(runtime.join(older).join(MARKER))
        .expect("marker")
        .modified()
        .expect("time");
    assert!(renewed >= started, "the parser load renewed the marker");
    let now = SystemTime::now();
    assert_eq!(
        remove_unused(&runtime, CURRENT, now, UNUSED_LIMIT).kept,
        vec![older.to_string()]
    );
    // A start renews it in the same way; 31 days after the last renewal the folder goes.
    set_time(&runtime.join(older).join(MARKER), started - days(31));
    mark_used(&runtime.join(older)).expect("start of the older copy");
    assert_eq!(
        remove_unused(&runtime, CURRENT, now, UNUSED_LIMIT).kept,
        vec![older.to_string()]
    );
    assert_eq!(
        remove_unused(&runtime, CURRENT, now + days(31), UNUSED_LIMIT).removed,
        vec![older.to_string()]
    );
}

#[test]
fn a_folder_without_a_marker_is_aged_by_its_own_time() {
    let base = tempfile::tempdir().expect("disposable cache");
    let runtime = base.path().join("runtime");
    let now = SystemTime::now();
    key_folder(&runtime, "00000000000000b1", None);
    key_folder(&runtime, "00000000000000b2", None);
    set_time(&runtime.join("00000000000000b1"), now - days(1));
    set_time(&runtime.join("00000000000000b2"), now - days(31));
    let sweep = remove_unused(&runtime, CURRENT, now, UNUSED_LIMIT);
    assert_eq!(sweep.kept, vec!["00000000000000b1".to_string()]);
    assert_eq!(sweep.removed, vec!["00000000000000b2".to_string()]);
}

#[test]
fn links_files_and_other_names_are_left_alone_and_never_followed() {
    let base = tempfile::tempdir().expect("disposable cache");
    let runtime = base.path().join("runtime");
    let outside = base.path().join("outside");
    let now = SystemTime::now();
    key_folder(&outside, "00000000000000d1", Some(now - days(90)));
    fs::create_dir_all(&runtime).expect("runtime folder");
    // A link named like a key, pointing at an old key folder outside the runtime folder.
    symlink(
        outside.join("00000000000000d1"),
        runtime.join("00000000000000d1"),
    )
    .expect("link to a folder");
    fs::write(runtime.join("00000000000000d2"), b"a file named like a key").expect("file");
    fs::create_dir_all(runtime.join("notes")).expect("other folder");
    set_time(&runtime.join("notes"), now - days(90));
    fs::create_dir_all(runtime.join("00000000000000D3")).expect("upper-case name");
    set_time(&runtime.join("00000000000000D3"), now - days(90));
    // An old key folder that holds a link to the outside folder: the folder goes, the target stays.
    key_folder(&runtime, "00000000000000d4", None);
    symlink(&outside, runtime.join("00000000000000d4/escape")).expect("link inside");
    set_time(&runtime.join("00000000000000d4"), now - days(90));
    let sweep = remove_unused(&runtime, CURRENT, now, UNUSED_LIMIT);
    assert_eq!(sweep.removed, vec!["00000000000000d4".to_string()]);
    assert_eq!(
        sorted(&sweep.ignored),
        vec![
            "00000000000000D3".to_string(),
            "00000000000000d1".to_string(),
            "00000000000000d2".to_string(),
            "notes".to_string(),
        ]
    );
    assert!(
        fs::symlink_metadata(runtime.join("00000000000000d1"))
            .expect("link")
            .file_type()
            .is_symlink()
    );
    assert!(
        outside.join("00000000000000d1/grammars/sql.so").is_file(),
        "nothing outside the runtime folder was removed"
    );
    assert!(outside.join("00000000000000d1").join(MARKER).is_file());
}

#[test]
fn a_runtime_folder_that_is_a_link_is_not_swept() {
    let base = tempfile::tempdir().expect("disposable cache");
    let real = base.path().join("elsewhere");
    let now = SystemTime::now();
    key_folder(&real, "00000000000000e1", Some(now - days(90)));
    symlink(&real, base.path().join("runtime")).expect("runtime folder as a link");
    let sweep = remove_unused(&base.path().join("runtime"), CURRENT, now, UNUSED_LIMIT);
    assert_eq!(sweep, Sweep::default());
    assert!(real.join("00000000000000e1/grammars/sql.so").is_file());
}

#[test]
fn the_rest_of_a_removal_cut_short_is_removed() {
    let base = tempfile::tempdir().expect("disposable cache");
    let runtime = base.path().join("runtime");
    let now = SystemTime::now();
    let leftover = ".00000000000000f1.removing-4242-0";
    fs::create_dir_all(runtime.join(leftover).join("grammars")).expect("leftover");
    fs::create_dir_all(runtime.join(".00000000000000f2.removing-")).expect("not a leftover");
    fs::create_dir_all(runtime.join(".notakey.removing-1-0")).expect("not a leftover");
    let sweep = remove_unused(&runtime, CURRENT, now, UNUSED_LIMIT);
    assert_eq!(sweep.removed, vec![leftover.to_string()]);
    assert_eq!(
        names(&runtime),
        vec![
            ".00000000000000f2.removing-".to_string(),
            ".notakey.removing-1-0".to_string(),
        ]
    );
}

#[test]
fn the_marker_is_private_and_a_link_at_its_name_is_replaced_not_followed() {
    let base = tempfile::tempdir().expect("disposable cache");
    let folder = base.path().join("runtime").join(CURRENT);
    let target = base.path().join("someone-elses-file");
    fs::write(&target, b"unchanged").expect("target");
    fs::create_dir_all(&folder).expect("key folder");
    symlink(&target, folder.join(MARKER)).expect("link at the marker's name");
    mark_used(&folder).expect("renew");
    assert_eq!(fs::read(&target).expect("target"), b"unchanged");
    let marker = fs::symlink_metadata(folder.join(MARKER)).expect("marker");
    assert!(marker.is_file(), "the link was replaced by a regular file");
    assert_eq!(marker.permissions().mode() & 0o777, 0o600);
    let fresh = base.path().join("fresh").join(CURRENT);
    mark_used(&fresh).expect("first start");
    assert_eq!(
        fs::metadata(&fresh).expect("folder").permissions().mode() & 0o777,
        0o700
    );
}
