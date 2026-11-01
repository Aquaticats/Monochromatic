//! Verifies the marker a running sentinel leaves:
//! its text,
//! reading it back,
//! and that a link planted at its name is replaced and never followed.

/// The marker under test.
use crate::sentinel_marker::{parse_marker, read_marker_in, write_marker_in, SentinelMarker};

/// The marker is written with the pid and start time of the running process,
/// read back,
/// and absent when missing or malformed.
#[test]
fn the_marker_round_trips_and_malformed_text_is_absent() {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-marker-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("make the directory");
    assert_eq!(read_marker_in(&directory), None);
    write_marker_in(&directory, 4242).expect("write the marker");
    let read = read_marker_in(&directory).expect("read the marker back");
    assert_eq!(read.watcher_pid, 4242);
    assert_eq!(u32::try_from(read.pid).expect("positive"), std::process::id());
    assert!(read.start_time > 0);
    assert_eq!(parse_marker("1 2 3\n"), Some(SentinelMarker { watcher_pid: 1, pid: 2, start_time: 3 }));
    assert_eq!(parse_marker("1 2"), None);
    assert_eq!(parse_marker("1 2 3 4"), None);
    assert_eq!(parse_marker("a b c"), None);
    assert_eq!(parse_marker(""), None);
    std::fs::write(directory.join("sentinel"), "garbage").expect("write garbage");
    assert_eq!(read_marker_in(&directory), None);
}

/// A link planted at the marker's name is replaced by the marker and never followed,
/// and no temporary file is left behind.
#[test]
fn a_link_at_the_marker_name_is_replaced_and_not_followed() {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-marker-link-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("make the directory");
    let victim = directory.join("victim");
    std::fs::write(&victim, "precious").expect("write the victim");
    std::os::unix::fs::symlink(&victim, directory.join("sentinel")).expect("plant the link");
    write_marker_in(&directory, 4242).expect("write the marker");
    assert_eq!(std::fs::read_to_string(&victim).expect("read the victim"), "precious");
    assert!(!std::fs::symlink_metadata(directory.join("sentinel")).expect("the marker exists").file_type().is_symlink());
    assert_eq!(read_marker_in(&directory).expect("read the marker back").watcher_pid, 4242);
    let leftovers: Vec<String> = std::fs::read_dir(&directory)
        .expect("list the directory")
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().into_owned())
        .filter(|name| return name.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}
