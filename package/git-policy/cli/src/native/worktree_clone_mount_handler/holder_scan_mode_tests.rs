//! What:
//!  Controls of the `holder-scan` mode:
//!  the answer carries the result's own lines and the counts,
//!  the handler's own namespace is the one its own task is in,
//!  a mount at the slot in another namespace is named as a holder,
//!  two scans of one clone answer under two nonces,
//!  the caller's own nonce is the one the answer carries,
//!  and a dataset whose identity or GUID cannot be read is refused by name.
//! Why:
//!  A guest arm and a later step call this mode,
//!  so its answer has to be the result a reader parses rather than a summary
//!  about one,
//!  a refusal has to say nothing was scanned,
//!  and every destroy request asks for its own fresh answer
//!  (`doc/decision/cli-git-zfs-holder-scan-names-every-holder-it-can-read.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = runWith({ dataset, nonce }, scriptedPool, fixtureProc, fixtureSelf);
//! expect(parseHolderScan(answer.split('\n').slice(0, -1).join('\n')).holders.length).toBe(1);
//! ```

use super::{
    HolderScanInvocation,
    run_with,
};
use crate::clone_holder_scan::{
    HolderScan,
    parse_holder_scan,
};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::DMU_OBJECT_SET_FILESYSTEM;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::{
    DatasetReader,
    ListedChild,
    ReadFailure,
};
use crate::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    DmuObjsetStats,
    NvList,
};
use std::cell::RefCell;
use std::path::{
    Path,
    PathBuf,
};

/// The dataset the fixture's clone is.
const CLONE: &str = "git-policy-cli-store/step9/monochromatic-git-worktree-clone-datasets/one";

/// The GUID the fixture's clone answers.
const GUID: u64 = 42;

/// The nonce the fixture's request carries.
const NONCE: &str = "00112233445566778899aabbccddeeff";

/// The table a namespace holding the clone answers.
fn mount_line(
    mount_id: u32,
    root: &str,
    mount_point: &str,
) -> String {
    return format!("{mount_id} 30 0:32 {root} {mount_point} ro,nosuid,nodev - zfs {CLONE} ro\n");
}

/// One fixture thread of one task in one namespace,
/// at the fixture's shared root.
fn task(
    proc_root: &Path,
    pid: &str,
    namespace: &str,
    table: Option<&str>,
    readers: &Path,
) {
    let folder: PathBuf = proc_root.join(pid).join("task").join(pid);
    std::fs::create_dir_all(folder.join("ns")).expect("the fixture task is made");
    std::os::unix::fs::symlink(namespace, folder.join("ns/mnt"))
        .expect("the namespace link is made");
    std::os::unix::fs::symlink(readers, folder.join("root")).expect("the root link is made");
    if let Some(text) = table {
        std::fs::write(folder.join("mountinfo"), text).expect("the fixture table is written");
    }
}

/// One property as OpenZFS writes it.
fn pair_text(
    value: &str,
    source: &str,
) -> NvList {
    let mut pair: NvList = NvList::new();
    pair.add_str("value", value);
    pair.add_str("source", source);
    return pair;
}

/// The reply the fixture's clone answers,
/// carrying `intent` as a local property of the clone itself.
fn reply(intent: Option<&str>) -> NvList {
    let mut reply: NvList = NvList::new();
    if let Some(value) = intent {
        reply.add_nvlist(DESIRED_STATE_PROPERTY, pair_text(value, CLONE));
    }
    return reply;
}

/// The statistics the fixture's clone answers.
fn stats(guid: u64) -> DmuObjsetStats {
    return DmuObjsetStats {
        dds_num_clones: 0,
        dds_creation_txg: 7,
        dds_guid: guid,
        dds_type: DMU_OBJECT_SET_FILESYSTEM,
        dds_is_snapshot: 0,
        dds_inconsistent: 0,
        dds_redacted: 0,
        dds_origin: [0; 256],
        dds_flags: 0,
    };
}

/// A pool that answers one dataset's identity from a script.
struct ScriptedPool {
    /// The datasets it holds and their replies.
    held: RefCell<Vec<(String, u64, NvList)>>,
}

impl ScriptedPool {
    /// A pool holding one clone with the given intent and GUID.
    fn of(
        intent: Option<&str>,
        guid: u64,
    ) -> Self {
        return ScriptedPool {
            held: RefCell::new(vec![(String::from(CLONE), guid, reply(intent))]),
        };
    }
}

impl DatasetReader for ScriptedPool {
    fn identity(
        &self,
        dataset: &str,
    ) -> Result<DatasetIdentity, ReadFailure> {
        let held = self.held.borrow();
        let Some((_name, guid, properties)) = held
            .iter()
            .find(|(name, _guid, _properties)| return name == dataset)
        else {
            return Err(ReadFailure::at(format!(
                "this scripted pool holds no `{dataset}`"
            )));
        };
        return Ok(DatasetIdentity {
            stats: stats(*guid),
            properties: properties.clone(),
        });
    }

    fn next_child(
        &self,
        _parent: &str,
        _cursor: u64,
    ) -> Result<Option<ListedChild>, String> {
        return Ok(None);
    }
}

/// The fixture `/proc` root and the task that names the handler's own
/// namespace.
fn proc_root(
    root: &Path,
    own_namespace: &str,
) -> (PathBuf, PathBuf) {
    let proc: PathBuf = root.join("proc");
    let readers: PathBuf = proc.join("reader-root");
    std::fs::create_dir_all(readers.as_path()).expect("the fixture root is made");
    // The grouping lists the tasks the kernel names by number, so a fixture task is named by a pid and the mode is
    // given that task's folder as its own; its root is the one the fixture's threads share.
    task(
        proc.as_path(),
        "11",
        own_namespace,
        Some(mount_line(41, "/", "/home/user/Monochromatic/.claude/worktrees/one").as_str()),
        readers.as_path(),
    );
    task(
        proc.as_path(),
        "21",
        "mnt:[4026532001]",
        Some(mount_line(42, "/", "/srv/holder").as_str()),
        readers.as_path(),
    );
    return (proc.clone(), proc.join("11").join("task").join("11"));
}

/// One invocation of the fixture's clone.
fn invocation() -> HolderScanInvocation {
    return HolderScanInvocation {
        dataset: String::from(CLONE),
        nonce: None,
    };
}

/// One invocation that names its own nonce.
fn named_invocation(nonce: &str) -> HolderScanInvocation {
    return HolderScanInvocation {
        dataset: String::from(CLONE),
        nonce: Some(RequestNonce::parse(nonce).expect("the fixture nonce is one")),
    };
}

/// The result's own lines of one answer,
/// which are every line but the summary.
fn result_text(answer: &str) -> String {
    let lines: Vec<&str> = answer.lines().collect();
    let kept: &[&str] = &lines[..lines.len() - 1];
    return format!("{}\n", kept.join("\n"));
}

/// The answer carries the result a reader parses,
/// the holder it found and the counts in its last line.
#[test]
fn the_answer_carries_the_result_and_the_counts() {
    let root: PathBuf = fixture("holder-scan-mode-answer");
    let (proc, self_task): (PathBuf, PathBuf) = proc_root(root.as_path(), "mnt:[4026531001]");
    let nonce: String = format!("v1:{NONCE}:mount");
    let pool: ScriptedPool = ScriptedPool::of(Some(nonce.as_str()), GUID);
    let answer: String = run_with(&invocation(), &pool, proc.as_path(), self_task.as_path())
        .expect("the scripted pool answers");
    let read: HolderScan = parse_holder_scan(result_text(answer.as_str()).as_str())
        .expect("the answer's own lines are the result");
    assert_eq!(read.dataset.file_name(), String::from("42"));
    assert_eq!(read.own_namespace.as_deref(), Some("mnt:[4026531001]"));
    assert_eq!(read.holder_namespaces(), 1);
    assert_eq!(read.holders[0].namespace(), "mnt:[4026532001]");
    let summary: &str = answer.lines().last().unwrap_or_default();
    assert!(
        summary.contains("1 holder namespace(s) holding 1 mount(s)"),
        "{summary}"
    );
    assert!(
        summary.contains("it mounted, unmounted and recorded nothing"),
        "{summary}"
    );
    assert!(
        summary.contains("fresh, because the caller named none"),
        "{summary}"
    );
}

/// Two scans of one clone answer under two nonces,
/// never under the clone's request nonce,
/// so an answer read at one destroy request is refused at the next.
#[test]
fn two_scans_of_one_clone_answer_under_their_own_nonces() {
    let root: PathBuf = fixture("holder-scan-mode-two-nonces");
    let (proc, self_task): (PathBuf, PathBuf) = proc_root(root.as_path(), "mnt:[4026531001]");
    let unmount: String = format!("v1:{NONCE}:unmount");
    let pool: ScriptedPool = ScriptedPool::of(Some(unmount.as_str()), GUID);
    let first: String = run_with(&invocation(), &pool, proc.as_path(), self_task.as_path())
        .expect("the scripted pool answers");
    let second: String = run_with(&invocation(), &pool, proc.as_path(), self_task.as_path())
        .expect("the scripted pool answers");
    let first_scan: HolderScan = parse_holder_scan(result_text(first.as_str()).as_str())
        .expect("the first answer's lines are a result");
    let second_scan: HolderScan = parse_holder_scan(result_text(second.as_str()).as_str())
        .expect("the second answer's lines are a result");
    assert_ne!(
        first_scan.nonce.as_str(),
        second_scan.nonce.as_str(),
        "each scan answers under its own nonce"
    );
    assert_ne!(
        first_scan.nonce.as_str(),
        NONCE,
        "and never under the clone's request nonce"
    );
    let expectation = crate::clone_holder_scan::ScanExpectation {
        dataset: second_scan.dataset,
        nonce: &second_scan.nonce,
    };
    assert!(
        first_scan.accept(&expectation).is_err(),
        "the earlier scan's answer is refused at the later request"
    );
    assert!(second_scan.accept(&expectation).is_ok());
}

/// A mount of the clone at the slot in another namespace is named as a holder
/// while the clone's intent stands at `unmount`,
/// which is the state a removal scans in.
#[test]
fn a_mount_at_the_slot_in_another_namespace_is_named_under_an_unmount_intent() {
    let root: PathBuf = fixture("holder-scan-mode-unmount");
    let (proc, self_task): (PathBuf, PathBuf) = proc_root(root.as_path(), "mnt:[4026531001]");
    let readers: PathBuf = proc.join("reader-root");
    task(
        proc.as_path(),
        "31",
        "mnt:[4026532002]",
        Some(mount_line(43, "/", "/home/user/Monochromatic/.claude/worktrees/one").as_str()),
        readers.as_path(),
    );
    let unmount: String = format!("v1:{NONCE}:unmount");
    let pool: ScriptedPool = ScriptedPool::of(Some(unmount.as_str()), GUID);
    let answer: String = run_with(&invocation(), &pool, proc.as_path(), self_task.as_path())
        .expect("the scripted pool answers");
    let read: HolderScan = parse_holder_scan(result_text(answer.as_str()).as_str())
        .expect("the answer's own lines are the result");
    assert_eq!(
        read.own_mounts.len(),
        1,
        "the handler's own mount is its own"
    );
    assert_eq!(
        read.holder_namespaces(),
        2,
        "both other namespaces are named: {:?}",
        read.holders
    );
    let summary: &str = answer.lines().last().unwrap_or_default();
    assert!(summary.contains("it sets nothing aside"), "{summary}");
}

/// The caller's own nonce is the one the answer carries,
/// and the summary says where it came from.
#[test]
fn the_callers_nonce_is_the_one_the_answer_carries() {
    let root: PathBuf = fixture("holder-scan-mode-named-nonce");
    let (proc, self_task): (PathBuf, PathBuf) = proc_root(root.as_path(), "mnt:[4026531001]");
    let pool: ScriptedPool = ScriptedPool::of(None, GUID);
    let named: &str = "fedcba9876543210fedcba9876543210";
    let answer: String = run_with(
        &named_invocation(named),
        &pool,
        proc.as_path(),
        self_task.as_path(),
    )
    .expect("the scripted pool answers");
    let read: HolderScan = parse_holder_scan(result_text(answer.as_str()).as_str())
        .expect("the answer's own lines are the result");
    assert_eq!(read.nonce.as_str(), named);
    let summary: &str = answer.lines().last().unwrap_or_default();
    assert!(summary.contains("named by the caller"), "{summary}");
}

/// A dataset this pool does not hold,
/// and one whose GUID is zero,
/// are each refused by name with nothing scanned.
#[test]
fn a_dataset_that_cannot_be_read_or_named_is_refused() {
    let absent: ScriptedPool = ScriptedPool::of(None, GUID);
    let other: HolderScanInvocation = HolderScanInvocation {
        dataset: String::from("git-policy-cli-store/step9/clones/two"),
        nonce: None,
    };
    let refusal: String = run_with(&other, &absent, Path::new("/proc"), Path::new("/proc/self"))
        .expect_err("a dataset the pool does not hold is refused");
    assert!(refusal.contains("could not read the dataset"), "{refusal}");
    assert!(refusal.contains("Nothing was scanned"), "{refusal}");
    let zero: ScriptedPool = ScriptedPool::of(None, 0);
    let unnamed: String = run_with(
        &invocation(),
        &zero,
        Path::new("/proc"),
        Path::new("/proc/self"),
    )
    .expect_err("a zero GUID is refused");
    assert!(unnamed.contains("answered GUID 0"), "{unnamed}");
}
