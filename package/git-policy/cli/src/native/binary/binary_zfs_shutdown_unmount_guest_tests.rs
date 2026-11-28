//! What:
//!  Plan step 19's shutdown pass,
//!  driven as root against a real file-backed
//!  pool in a throwaway guest:
//!  an idle clone's mount is unmounted,
//!  a busy clone's mount is kept and its holder is named,
//!  this boot's readiness is withdrawn before either,
//!  no attach is admitted afterwards,
//!  and no clone's durable intent changes.
//! Why:
//!  The pass reads this boot's readiness,
//!  this process's mount table,
//!  the route's recorded `.claude/worktrees` identity and one dataset's GUID,
//!  and it unmounts a real mount:
//!  none of that exists on a machine where no pool is imported and no mount may
//!  be made,
//!  so the arm runs only where `CLI_GIT_ZFS_SHUTDOWN_UNMOUNT_GUEST` names a
//!  disposable guest
//!  (`package/git-policy/cli/e2e/shutdown-unmount-guest.ts`,
//!  on OpenZFS 2.4.4 in an `mvm` guest).
//!  Everywhere else it prints one `SKIPPED` line,
//!  which the driver reads as a failure when it meant the arm to run.
//!  The mode that will call the pass is still the stub that refuses,
//!  so the arm calls the pass itself:
//!  that is the round's whole claim,
//!  and the wiring round adds the mode's own arm.
//!  Every claim is a marker line the driver requires,
//!  so a section that silently stopped running fails the run instead of
//!  printing less.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!process.env.CLI_GIT_ZFS_SHUTDOWN_UNMOUNT_GUEST) return print('SKIPPED');
//! layout(); mountIdle(); mountBusy(); publishReadiness();
//! const outcome = runThePass(liveRequest()); expect(outcome.unmounted()).toBe(1); expect(outcome.held()).toBe(1);
//! ```

use super::zfs_sweep_guest::{
    STORE,
    clone_the_checkouts_snapshot,
    command_ok,
    make_the_designs_layout,
    make_the_slot,
    prepare_the_store,
    required,
};
use git_policy_cli::foundation::iso_time::now_iso;
use git_policy_cli::worktree_clone_mount_handler::sweep_dataset_reader::ZfsDatasetReader;
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness::{
    ReadinessRecord,
    ReadyState,
    SweepState,
};
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use git_policy_cli::worktree_clone_shutdown_unmount_pass::shutdown_unmount_answer::PassOutcome;
use git_policy_cli::worktree_clone_shutdown_unmount_pass::shutdown_unmount_places::ShutdownUnmountPlaces;
use git_policy_cli::worktree_clone_shutdown_unmount_pass::shutdown_unmount_request::ShutdownUnmountRequest;
use git_policy_cli::worktree_clone_shutdown_unmount_pass::shutdown_unmount_sources::{
    LiveDatasetGuids,
    LiveHolderScans,
    LiveSlotLookups,
    LiveSlotUnmounts,
    LiveWorktreesFolder,
};
use git_policy_cli::worktree_clone_shutdown_unmount_pass::shutdown_unmount_turn::run_the_pass;
use git_policy_cli::worktree_clone_unmount::slot_walk_calls::LiveSlotWalk;
use git_policy_cli::worktree_clone_unmount::staging_calls::LiveStagingCalls;
use git_policy_cli::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::io::{
    BufRead,
    BufReader,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    ChildStdout,
    Command,
    Stdio,
};

/// The variable the guest driver sets.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_SHUTDOWN_UNMOUNT_GUEST";

/// The pool variable the driver prepared.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_POOL";

/// The OpenZFS userspace variable the driver prepared.
const SBIN_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_SBIN";

/// The checkout variable the driver prepared.
const CHECKOUT_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_CHECKOUT";

/// The idle clone's slot,
/// which nothing holds.
const IDLE_SLOT: &str = "one";

/// The busy clone's slot,
/// which one child holds open.
const BUSY_SLOT: &str = "two";

/// The clone's durable intent property,
/// which the pass must leave exactly as it found it.
const DESIRED_STATE: &str = "org.monochromatic:git-worktree-clone-mount-desired-state";

/// The intent value both clones carry,
/// which is a `mount` request the pass never rewrites.
const INTENT: &str = "v1:00112233445566778899aabbccddeeff:mount";

/// Whether this run is inside the guest.
fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!(
            "SKIPPED worktree_clone_shutdown_unmount_pass::guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest"
        );
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname")
        .map(|text| return text.trim().to_string())
        .unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// One clone's intent property as the pool reports it now.
fn intent_of(
    sbin: &str,
    clone: &str,
) -> String {
    let (status, said, _error) = super::zfs_sweep_guest::zfs(
        sbin,
        "zfs",
        &["get", "-H", "-o", "value", "-p", DESIRED_STATE, clone],
        None,
    );
    assert!(status, "the intent of {clone} is read: {said}");
    return said.trim().to_string();
}

/// One clone mounted at its slot the way a legacy mount stands,
/// which is the shape the route's own attach leaves in the mount table.
fn mount_the_clone(
    clone: &str,
    slot_path: &Path,
) {
    std::fs::create_dir_all(slot_path).expect("the slot folder is made");
    command_ok(
        "mount",
        &[
            "-t",
            "zfs",
            clone,
            slot_path.to_str().expect("the slot path is text"),
        ],
    );
}

/// Whether one dataset stands mounted anywhere this process can see.
fn mounted(clone: &str) -> bool {
    return super::zfs_sweep_guest::mounted(clone);
}

/// What:
///  One child that holds a file inside a mount open,
///  so the kernel answers `EBUSY` to an unmount of it.
/// Why:
///  A busy clone is the arm's second half,
///  and the only way to make one is a task with something open inside the
///  mount
///  (`umount(2)`:
///  "EBUSY ... a file on the filesystem is open").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holder = holdAFileIn(slot); // a child sleeping with the file open
/// ```
struct Holder {
    /// The child holding the file.
    child: Child,
}

impl Holder {
    /// Open one file inside the mount and sleep with it open,
    /// waiting for the line the arm reads before it removes the file.
    fn hold_a_file_in(slot_path: &Path) -> Holder {
        let file: PathBuf = slot_path.join("held-by-the-arm");
        std::fs::write(file.as_path(), b"the arm holds this\n").expect("the held file is written");
        let mut command: Command = Command::new("sh");
        command
            .arg("-c")
            .arg("exec 3<\"$1\"; echo ready; sleep 600")
            .arg("sh")
            .arg(file.to_str().expect("the file path is text"))
            .stdout(Stdio::piped());
        // The child is this test's to end, so it is started through the target's own owner and ended however the
        // assertions below go.
        let mut child: Child =
            super::support::spawn_owned(&mut command).expect("the holding child starts");
        // The child is ready once it has the file open, which it does before it
        // prints anything; a read of the pipe waits for the arm's own line so no
        // sleep guesses at it.
        let stdout: ChildStdout = child.stdout.take().expect("the child's output is piped");
        let mut reader: BufReader<ChildStdout> = BufReader::new(stdout);
        let mut line: String = String::new();
        let _read: std::io::Result<usize> = reader.read_line(&mut line);
        assert_eq!(
            line.trim(),
            "ready",
            "the holding child says it has the file open before the pass runs"
        );
        return Holder {
            child,
        };
    }

    /// End the child and wait for it,
    /// so no child outlives the assertion that reads the mount.
    fn end(mut self) {
        let _killed: std::io::Result<()> = self.child.kill();
        let _waited: std::io::Result<std::process::ExitStatus> = self.child.wait();
    }
}

/// Publish this boot's readiness record,
/// which is what says the pass may act on this boot's mounts.
fn publish_readiness(
    pool: &str,
    checkout: &Path,
) {
    let boot: String = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .expect("this boot's identifier is read")
        .trim()
        .to_string();
    let record: ReadinessRecord = ReadinessRecord {
        boot,
        pool_name: String::from(pool),
        pool_state: ReadyState::Ready,
        checkout_path: checkout.display().to_string(),
        checkout_state: ReadyState::Ready,
        sweep_nonce: None,
        sweep_state: SweepState::Finished,
        time: now_iso(),
    };
    ReadinessFile::live()
        .publish(&record)
        .expect("this boot's readiness record is published");
}

/// What:
///  The whole arm:
///  the design's layout,
///  an idle clone and a busy one,
///  this boot's readiness,
///  one pass,
///  and every claim the driver requires.
/// Why:
///  The pass's claims are about one machine at one moment,
///  so the whole scene is built,
///  read and taken down in one test rather than in arms that would each need
///  the other's leftover.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theArm();
/// ```
#[test]
fn the_shutdown_unmount_guest_arm() {
    if !guest() {
        return;
    }
    let pool: String = required(POOL_VARIABLE);
    let sbin: String = required(SBIN_VARIABLE);
    let checkout: PathBuf = PathBuf::from(required(CHECKOUT_VARIABLE));
    std::fs::create_dir_all(checkout.as_path()).expect("the checkout folder is made");
    let (checkout_dataset, container): (String, String) =
        make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
    prepare_the_store(checkout.as_path());
    super::zfs_sweep_guest::write_configuration(
        checkout.as_path(),
        Some(container.as_str()),
        Some(checkout_dataset.as_str()),
    );
    make_the_slot(checkout.as_path(), IDLE_SLOT);
    make_the_slot(checkout.as_path(), BUSY_SLOT);
    let idle: String = clone_the_checkouts_snapshot(
        sbin.as_str(),
        checkout_dataset.as_str(),
        container.as_str(),
        IDLE_SLOT,
        "shutdown-idle",
        Some(INTENT),
    );
    let busy: String = clone_the_checkouts_snapshot(
        sbin.as_str(),
        checkout_dataset.as_str(),
        container.as_str(),
        BUSY_SLOT,
        "shutdown-busy",
        Some(INTENT),
    );
    let idle_slot: PathBuf = checkout.join(".claude").join("worktrees").join(IDLE_SLOT);
    let busy_slot: PathBuf = checkout.join(".claude").join("worktrees").join(BUSY_SLOT);
    mount_the_clone(idle.as_str(), idle_slot.as_path());
    mount_the_clone(busy.as_str(), busy_slot.as_path());
    assert!(
        mounted(idle.as_str()) && mounted(busy.as_str()),
        "both clones stand mounted before the pass runs"
    );
    println!("SHUTDOWN-UNMOUNT layout=two-mounts");
    let intent_before_idle: String = intent_of(sbin.as_str(), idle.as_str());
    let intent_before_busy: String = intent_of(sbin.as_str(), busy.as_str());
    let holder: Holder = Holder::hold_a_file_in(busy_slot.as_path());
    publish_readiness(pool.as_str(), checkout.as_path());
    println!("SHUTDOWN-UNMOUNT readiness=published");
    let reader: ZfsDatasetReader = ZfsDatasetReader::open().expect("the guest's ZFS device opens");
    let places: ShutdownUnmountPlaces = ShutdownUnmountPlaces::live();
    let worktrees: LiveWorktreesFolder = LiveWorktreesFolder::open(Path::new(STORE))
        .expect("the store this arm published opens under root control");
    let mounts: LiveMountedDatasets = LiveMountedDatasets;
    let calls: LiveStagingCalls = LiveStagingCalls;
    let walk: LiveSlotWalk = LiveSlotWalk;
    let unmounts: LiveSlotUnmounts = LiveSlotUnmounts;
    let lookups: LiveSlotLookups = LiveSlotLookups;
    let guids: LiveDatasetGuids<'_> = LiveDatasetGuids {
        reader: &reader,
    };
    let holders: LiveHolderScans = LiveHolderScans {
        proc_root: PathBuf::from("/proc"),
        self_task: PathBuf::from("/proc/self"),
    };
    let request: ShutdownUnmountRequest<'_> = ShutdownUnmountRequest {
        places: &places,
        mounts: &mounts,
        calls: &calls,
        walk: &walk,
        unmounts: &unmounts,
        lookups: &lookups,
        worktrees: &worktrees,
        guids: &guids,
        holders: &holders,
    };
    let outcome: PassOutcome = run_the_pass(&request).expect("the pass answers");
    let lines: Vec<String> = outcome.lines();
    for line in &lines {
        println!("SHUTDOWN-UNMOUNT pass-line: {line}");
    }
    let summary: &str = lines.first().expect("the pass answers a summary line");
    assert!(
        summary.contains(
            "2 mount(s) of this boot's clone container stood at a slot, 1 unmounted, 1 held, 0 left"
        ),
        "the summary counts the two mounts and what became of each, and this one says {summary}"
    );
    assert!(
        !mounted(idle.as_str()),
        "the idle clone's mount is gone after the pass: the row the pool reports for {idle}"
    );
    assert!(
        mounted(busy.as_str()),
        "a busy clone is never torn down by force, so its mount still stands"
    );
    println!("SHUTDOWN-UNMOUNT idle=unmounted");
    println!("SHUTDOWN-UNMOUNT busy=held");
    let unmounted_line: &String = lines
        .iter()
        .find(|line| return line.contains("unmounted slot `one`"))
        .expect("the pass answered a line about the idle slot");
    assert!(
        unmounted_line.contains(&format!("of `{idle}`")),
        "the idle line names the clone it unmounted, and this one says {unmounted_line}"
    );
    let held_line: &String = lines
        .iter()
        .find(|line| return line.contains("held slot `two`"))
        .expect("the pass answered a line about the busy slot");
    assert!(
        held_line.contains("stays exactly as it was"),
        "the busy line says the mount is where it was, and this one says {held_line}"
    );
    assert!(
        held_line.contains("the all-namespace scan")
            || held_line.contains("no holder scan could be made"),
        "the busy line names what the scan found or why no scan was made, and this one says {held_line}"
    );
    assert!(
        !held_line.contains("named no namespace holding a mount of it"),
        "the scan names the namespace this process's own table holds the clone in, and this line says {held_line}"
    );
    println!("SHUTDOWN-UNMOUNT holder=named");
    // What the wrapper's own add preflight decides from: an absent record is "no
    // handler readiness report stands for this boot, so no attach is admitted"
    // (`worktree_clone_add_route/add_preflight.rs`, `admit_the_handler`).
    assert_eq!(
        ReadinessFile::live()
            .read()
            .expect("the readiness directory reads"),
        None,
        "this boot's readiness is withdrawn, so no attach is admitted after the pass"
    );
    println!("SHUTDOWN-UNMOUNT readiness=withdrawn");
    assert_eq!(
        intent_of(sbin.as_str(), idle.as_str()),
        intent_before_idle,
        "the idle clone's durable intent is exactly what it was"
    );
    assert_eq!(
        intent_of(sbin.as_str(), busy.as_str()),
        intent_before_busy,
        "the busy clone's durable intent is exactly what it was"
    );
    assert_eq!(intent_before_idle, INTENT);
    println!("SHUTDOWN-UNMOUNT intent=unchanged");
    // A second pass finds no record of this boot, which is what says the
    // withdrawal came before anything was drained: it unmounts nothing.
    let second: PassOutcome = run_the_pass(&request).expect("a second pass answers");
    let second_lines: Vec<String> = second.lines();
    for line in &second_lines {
        println!("SHUTDOWN-UNMOUNT second-line: {line}");
    }
    assert_eq!(
        second_lines.len(),
        1,
        "a boot whose readiness is withdrawn reaches no mount, and this pass answered {second_lines:?}"
    );
    assert!(
        second_lines[0].contains("this boot had published no readiness record"),
        "the second pass says what was missing, and this one says {}",
        second_lines[0]
    );
    println!("SHUTDOWN-UNMOUNT second-pass=proves-nothing");
    holder.end();
    // The arm takes its own fixture down, so the guest is ready for the next
    // run and no mount outlives the arm.
    unmount_ignoring(busy_slot.as_path());
    println!("SHUTDOWN-UNMOUNT done");
}

/// Unmount one folder this arm mounted,
/// and nothing else.
fn unmount_ignoring(folder: &Path) {
    let _taken: std::io::Result<std::process::ExitStatus> =
        Command::new("umount").arg(folder).status();
}
