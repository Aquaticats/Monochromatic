//! Controls for the temporary file of a write, and the sweep of what a killed one left.

use super::*;
use crate::test_support::{fixture, remove};
use std::time::{Duration, SystemTime};

/// How long a file of no writer must be before the sweep treats it as a killed write's remains.
const GRACE: Duration = Duration::from_secs(10 * 60);

/// A process number no process of this machine has.
/// Why:
///   The highest number the kernel hands out is far below it,
/// so a file naming it is the remains of a writer that is gone,
/// and no live process can take the number while this control runs.
fn a_number_no_process_has() -> u32 {
    return 4_000_000_000;
}

/// The name of the file a write of `target` makes carries that write's own process number,
/// and that number is the one read back from the name.
#[test]
fn the_temporary_of_a_write_carries_its_writer() {
    let target: PathBuf = PathBuf::from("/state/journal/7.intent");
    let temporary: PathBuf = name_of(target.as_path());
    let name: &str = temporary.file_name().expect("a name").to_str().expect("text");
    assert!(name.starts_with(format!("7.intent.{}-", std::process::id()).as_str()) && name.ends_with(".tmp"), "the writer's number is in the name: {name}");
    assert_eq!(writer_of(temporary.file_name().expect("a name").to_str().expect("text")), Some(std::process::id()));
    assert!(!writer_is_gone(std::process::id()), "the process writing this control is alive");
}

/// A name of another shape carries no writer, so nothing about it can be proven.
#[test]
fn a_name_of_another_shape_names_no_writer() {
    for name in ["7.tmp", "7.intent.tmp", "notatemporary", "7.intent.12x.tmp", "0f1e2d3c.tmp"] {
        assert_eq!(writer_of(name), None, "{name:?} names no writer");
    }
    assert_eq!(writer_of("7.intent.tmp-1234-0"), Some(1234), "a staging name of the release before this one names its writer after the marker");
    assert_eq!(writer_of("7.intent.1234.tmp"), Some(1234));
    assert_eq!(writer_of("7.intent.1234-7.tmp"), Some(1234), "the count of one write follows its writer's number");
    assert_eq!(writer_of(".7.intent.abcdef.1234.partial"), Some(1234), "a staging file of the private storage names its writer too");
    assert_eq!(writer_of("monochromatic-git-policy-cli@x.socket.tmp-1234-0"), Some(1234), "a unit file's staging name of the release before this one names its writer");
    assert_eq!(writer_of("service.linktmp-1234-0"), Some(1234), "and so does the staging name of the link beside it");
}

/// What:
///  The sweep removes the temporary of a writer that is gone,
///  and leaves the one the process writing now made,
///  however old the folder's other files are.
/// Why:
///   Removing a live writer's temporary makes its rename fail and costs the batch;
/// the kernel's answer about the number in the name is the only proof that a write will not finish.
#[test]
fn the_sweep_removes_a_dead_writers_temporary_and_leaves_a_live_ones() {
    let root: PathBuf = fixture("temporary-of-a-dead-writer");
    let live: PathBuf = name_of(root.join("7.intent").as_path());
    std::fs::write(&live, "seq=7\n").expect("the write of this process");
    let dead: PathBuf = root.join(format!("8.intent.{}-0.tmp", a_number_no_process_has()));
    std::fs::write(&dead, "seq=8\n").expect("the write of a process that is gone");
    let record: PathBuf = root.join("9.intent");
    std::fs::write(&record, "seq=9\n").expect("a record");
    let swept: Swept = sweep_temporaries(root.as_path(), GRACE);
    assert_eq!(swept.removed, 1, "one temporary is the remains of a killed write");
    assert!(live.exists(), "the temporary of a writer that is still there is left alone");
    assert!(!dead.exists(), "the temporary of a writer that is gone is removed");
    assert!(record.exists(), "no record is removed");
    remove(root.as_path());
}

/// What:
///  A temporary of a release that named no writer is removed only once it is far older than any write,
///  and a young one is left,
///  because nothing else about it can be proven.
/// Why:
///   The release before this one wrote `<seq>.tmp`,
/// so its remains are indistinguishable from a write in progress except by their age.
#[test]
fn a_temporary_of_no_writer_goes_only_when_it_is_old() {
    let root: PathBuf = fixture("temporary-of-no-writer");
    let young: PathBuf = root.join("1.tmp");
    std::fs::write(&young, "seq=1\n").expect("a write in progress");
    let old: PathBuf = root.join("2.tmp");
    std::fs::write(&old, "seq=2\n").expect("a write that was cut off");
    let long_ago: SystemTime = SystemTime::now() - GRACE - Duration::from_secs(60);
    std::fs::File::options().append(true).open(&old).and_then(|file| { return file.set_modified(long_ago); }).expect("aged");
    let swept: Swept = sweep_temporaries(root.as_path(), GRACE);
    assert_eq!(swept.removed, 1);
    assert!(young.exists(), "a young file of no writer may be a write in progress");
    assert!(!old.exists(), "an old file of no writer is the remains of a killed write");
    remove(root.as_path());
}

/// What:
///  A temporary that is a folder is removed whole,
///  and a folder the sweep did not write is left.
/// Why:
///   The copy of a batch's conclusion state is built in a folder beside its name,
/// so a kill between its copy and its rename leaves a tree, not a file.
#[test]
fn a_temporary_that_is_a_folder_goes_whole() {
    let root: PathBuf = fixture("temporary-folder");
    let dead: PathBuf = root.join(format!("3.cleanup.{}-0.tmp", a_number_no_process_has()));
    std::fs::create_dir_all(dead.join("shadow")).expect("the tree of a copy");
    std::fs::write(dead.join("shadow/MERGE_RR"), "rr\n").expect("a file of the tree");
    let kept: PathBuf = root.join("3.cleanup");
    std::fs::create_dir_all(&kept).expect("the copy it was published as");
    let swept: Swept = sweep_temporaries(root.as_path(), GRACE);
    assert_eq!(swept.removed, 1);
    assert!(!dead.exists(), "the tree of a killed copy is removed");
    assert!(kept.exists(), "the copy it was published as is a record, and stays");
    remove(root.as_path());
}
