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

/// Every name in a folder is this package's own, for a folder this package alone writes in.
fn every_name(_name: &str) -> bool {
    return true;
}

/// What:
///  The name of the file a write makes carries its writer's process number,
///  and the number read back from it is the one writing now.
/// Why:
///   That number is the only thing a sweep can ask the kernel about,
/// so a name that did not carry it could not be judged at all.
#[test]
fn the_temporary_of_a_write_carries_its_writer() {
    let target: PathBuf = PathBuf::from("/state/journal/7.intent");
    let temporary: PathBuf = name_of(target.as_path());
    let name: &str = temporary.file_name().expect("a name").to_str().expect("text");
    assert!(name.starts_with(format!("7.intent.{}_", std::process::id()).as_str()) && name.ends_with(".tmp"), "the writer's number is in the name: {name}");
    let writer: Writer = writer_of(name).expect("the name names its writer");
    assert_eq!(writer.pid, std::process::id());
    assert_eq!(writer.birth, crate::process_identity::process_birth_identity(i64::from(std::process::id())).ok().flatten(), "and its birth identity, which no later process shares");
    assert!(!writer_is_gone(&writer), "the process writing this control is alive");
}

/// What:
///  Two writes of one process never share a name,
///  and a name the kernel could give to another process is not made again.
/// Why:
///   A rename that found its file already published by another thread would fail,
/// and a sweep that judged a name a second process had just made again would remove a live writer's file.
#[test]
fn two_writes_of_one_process_never_share_a_name() {
    let target: PathBuf = PathBuf::from("/state/journal/7.intent");
    assert_ne!(name_of(target.as_path()), name_of(target.as_path()), "the count of the write keeps two writes of one process apart");
    let with_a_birth: &str = "7.intent.4242_linux:99_0.tmp";
    let without: &str = "7.intent.4242_0.tmp";
    assert_eq!(writer_of(with_a_birth), Some(Writer { pid: 4242, birth: Some(String::from("linux:99")) }));
    assert_eq!(writer_of(without), Some(Writer { pid: 4242, birth: None }), "a name that carries no birth identity names the process alone");
    assert_ne!(with_a_birth, without, "so a number the kernel gives again makes another name, not this one");
}

/// A name of another shape carries no writer, so nothing about it can be proven.
#[test]
fn a_name_of_another_shape_names_no_writer() {
    for name in ["7.tmp", "7.intent.tmp", "notatemporary", "7.intent.12x.tmp", "0f1e2d3c.tmp"] {
        assert_eq!(writer_of(name), None, "{name:?} names no writer");
    }
    assert_eq!(writer_of("7.intent.1234.tmp"), Some(Writer { pid: 1234, birth: None }));
    assert_eq!(writer_of("monochromatic-git-policy-cli@x.socket.tmp-1234-0"), Some(Writer { pid: 1234, birth: None }), "a unit file's staging name of the release before this one names its writer");
    assert_eq!(writer_of("service.linktmp-1234-0"), Some(Writer { pid: 1234, birth: None }), "and so does the staging name of the link beside it");
    assert_eq!(writer_of(".7.intent.abcdef.1234.partial"), Some(Writer { pid: 1234, birth: None }), "a staging file of the private storage names its writer too");
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
    let dead: PathBuf = root.join(format!("8.intent.{}.tmp", writer_fields().replace(std::process::id().to_string().as_str(), a_number_no_process_has().to_string().as_str())));
    std::fs::write(&dead, "seq=8\n").expect("the write of a process that is gone");
    let record: PathBuf = root.join("9.intent");
    std::fs::write(&record, "seq=9\n").expect("a record");
    let swept: Swept = sweep_temporaries(root.as_path(), GRACE, &every_name);
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
///   The release before this one wrote names without a writer in them,
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
    let swept: Swept = sweep_temporaries(root.as_path(), GRACE, &every_name);
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
    let dead: PathBuf = root.join(format!("3.cleanup.{}_0.tmp", a_number_no_process_has()));
    std::fs::create_dir_all(dead.join("shadow")).expect("the tree of a copy");
    std::fs::write(dead.join("shadow/MERGE_RR"), "rr\n").expect("a file of the tree");
    let kept: PathBuf = root.join("3.cleanup");
    std::fs::create_dir_all(&kept).expect("the copy it was published as");
    let swept: Swept = sweep_temporaries(root.as_path(), GRACE, &every_name);
    assert_eq!(swept.removed, 1);
    assert!(!dead.exists(), "the tree of a killed copy is removed");
    assert!(kept.exists(), "the copy it was published as is a record, and stays");
    remove(root.as_path());
}

/// What:
///  A sweep of a folder that another program writes into removes only the names the caller calls its own.
/// Why:
///   The manager's own directory holds other programs' files,
/// so a temporary that ends the way one of this package's does is not proof that this package wrote it.
#[test]
fn a_sweep_leaves_a_name_the_caller_does_not_call_its_own() {
    let root: PathBuf = fixture("temporary-of-another-program");
    let ours: PathBuf = root.join(format!("monochromatic-git-policy-cli@x.socket.{}_0.tmp", a_number_no_process_has()));
    std::fs::write(&ours, "[Unit]\n").expect("a unit's staging file");
    let theirs: PathBuf = root.join(format!("other-application.state.{}_0.tmp", a_number_no_process_has()));
    std::fs::write(&theirs, "[Unit]\n").expect("another program's staging file");
    let swept: Swept = sweep_temporaries(root.as_path(), GRACE, &|name| { return name.starts_with("monochromatic-git-policy-cli"); });
    assert_eq!(swept.removed, 1, "only the name the caller calls its own is judged");
    assert!(!ours.exists(), "this package's staging file is removed");
    assert!(theirs.exists(), "another program's file is left alone");
    remove(root.as_path());
}
