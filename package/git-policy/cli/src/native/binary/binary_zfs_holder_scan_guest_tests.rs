//! What:
//!  Plan step 9's holder-scan arms in the throwaway guest:
//!  the built handler's scan of a clone the built sweep attached,
//!  a second mount namespace holding the clone,
//!  a confined consumer holding a bind of it,
//!  task churn while the scan runs,
//!  a pinned namespace with no live member,
//!  a task with a divergent process root,
//!  and how one scan's cost grows with the number of live tasks.
//! Why:
//!  The scan reads every task's mount namespace and reports holders,
//!  so no control on a person's machine may run it against that machine's
//!  processes:
//!  these arms run only where `CLI_GIT_ZFS_SCAN_GUEST` names a disposable guest
//!  (`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`,
//!  on OpenZFS 2.4.4 in an `mvm` Ubuntu guest),
//!  and they print `SKIP` everywhere else,
//!  which is why the arm carries no `#[ignore]`:
//!  a harness that skips an ignored test would run it nowhere,
//!  so a run that did
//!  nothing says so.
//!  The fixture is the design's own layout and the clone is attached by the
//!  built sweep,
//!  because the scan's question is about real mounts
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Dataset and path layout",
//!  "Mounts held by other namespaces";
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 9,
//!  "VM measurements").
//!  Task churn and the processes that hold the clone run inside this guest
//!  only.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await sweep(); const scan = await holderScan(clone);
//! await unshareHolder(clone); expect(scan.holderNamespaces).toBe(1);
//! ```

use git_policy_cli::clone_holder_scan::{HolderScan, parse_holder_scan, scan_holders, HolderScanRequest};
use git_policy_cli::foundation::iso_time::now_iso;
use git_policy_cli::start_probe::probe_fingerprint::{FingerprintPaths, ProbeFingerprint};
use git_policy_cli::start_probe::probe_record::{ABSENT, RECORD_KIND, RECORD_VERSION};
use git_policy_cli::worktree_clone_mount::{CloneFolderIdentity, publish_parent_identity_under_root_control};
use git_policy_cli::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use git_policy_cli::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The variable the guest driver sets.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_SCAN_GUEST";

/// The pool the driver prepared.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_SCAN_POOL";

/// The OpenZFS userspace the guest's commands need.
const SBIN_VARIABLE: &str = "CLI_GIT_ZFS_SCAN_SBIN";

/// The checkout the sweep walks.
const CHECKOUT_VARIABLE: &str = "CLI_GIT_ZFS_SCAN_CHECKOUT";

/// The checkout dataset's leaf name the design gives it.
const CHECKOUT_LEAF: &str = "monochromatic-git-checkout";

/// The container's leaf name the design gives it.
const CONTAINER_LEAF: &str = "monochromatic-git-worktree-clone-datasets";

/// The slot the clone attaches at.
const SLOT: &str = "one";

/// The prefix the add route gives its origin snapshots.
const ORIGIN_PREFIX: &str = "git-worktree-clone-origin-";

/// The nonce the arm's mount request carries.
const NONCE: &str = "00112233445566778899aabbccddeeff";

/// The handler configuration the sweep and the scan read.
const CONFIGURATION: &str = "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc";

/// The start probe's record directory inside the guest.
const PROBE_RECORD_DIRECTORY: &str = "/run/git-worktree-clone-mount-probe";

/// The first attach's record,
/// which the sweep writes and this arm clears.
const FIRST_ATTACH: &str = "/run/git-worktree-clone-mount-probe/first-attach";

/// The desired-state property.
const DESIRED_STATE: &str = "org.monochromatic:git-worktree-clone-mount-desired-state";

/// The store the sweep records into.
const STORE: &str = "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities";

/// The folder a holder's own mount namespace attaches the clone at.
const HOLDER_FOLDER: &str = "/mnt/holder-scan-holder";

/// The folder a confined consumer binds the clone's root at.
const CONFINED_FOLDER: &str = "/mnt/holder-scan-confined";

/// The folder the guest's root is bound at for the divergent-root task,
/// which is the root that task runs under.
const ROOTED_BIND: &str = "/root/holder-scan-rootfs";

/// The path the clone is mounted at inside that task's own root.
const ROOTED_FOLDER: &str = "/mnt/holder-scan-rooted-in-root";

/// How many live tasks the large half of the timing arm adds.
const CHURN_TASKS: usize = 200;

/// How many scans the churn arm runs.
const CHURN_SCANS: usize = 20;

/// How long one child that holds the clone stays alive,
/// which outlives every arm here.
const HOLDER_SECONDS: u64 = 900;

/// Whether this run is inside the guest,
/// or a skip line on every other machine.
fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP clone_holder_scan::holder_scan_guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest");
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname").map(|text| return text.trim().to_string()).unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// One required fixture variable,
/// or a panic naming it.
fn required(variable: &str) -> String {
    return std::env::var(variable).unwrap_or_else(|_error| panic!("{variable} is unset inside the guest, so this arm has no fixture to run against"));
}

/// Run one `zfs` or `zpool` command as root,
/// returning its status and both streams.
fn zfs(sbin: &str, program: &str, arguments: &[&str], stdin: Option<&str>) -> (bool, String, String) {
    let path: String = format!("{sbin}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
    let mut child = Command::new(program)
        .args(arguments)
        .env("PATH", path)
        .env("LD_LIBRARY_PATH", format!("{}/lib", sbin.trim_end_matches("/sbin")))
        .stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("{program} {arguments:?} could not start: {error}"));
    if let (Some(text), Some(mut pipe)) = (stdin, child.stdin.take()) {
        pipe.write_all(text.as_bytes()).expect("the input reaches the command");
    }
    let output: Output = child.wait_with_output().unwrap_or_else(|error| panic!("{program} {arguments:?} could not be waited for: {error}"));
    return (
        output.status.success(),
        String::from_utf8_lossy(output.stdout.as_slice()).to_string(),
        String::from_utf8_lossy(output.stderr.as_slice()).to_string(),
    );
}

/// One `zfs` command the arm requires to succeed.
fn zfs_ok(sbin: &str, arguments: &[&str]) {
    let (ok, _out, error): (bool, String, String) = zfs(sbin, "zfs", arguments, None);
    assert!(ok, "zfs {arguments:?} was refused: {error}");
}

/// One `zfs` command whose failure the arm expects.
fn zfs_ignored(sbin: &str, arguments: &[&str]) {
    let (_ok, _out, _error): (bool, String, String) = zfs(sbin, "zfs", arguments, None);
}

/// Run one program and require it succeeded.
fn command_ok(program: &str, arguments: &[&str]) {
    let status: std::process::ExitStatus = Command::new(program).args(arguments).status().unwrap_or_else(|error| panic!("{program} {arguments:?} could not start: {error}"));
    assert!(status.success(), "{program} {arguments:?} was refused");
}

/// The mount's own row for one folder,
/// or nothing when no mount stands there.
fn mount_row(folder: &Path) -> Option<String> {
    let table: String = std::fs::read_to_string("/proc/self/mountinfo").expect("the arm reads this process's mount table");
    let wanted: String = folder.display().to_string();
    return table.lines().find(|line| return line.split(' ').nth(4) == Some(wanted.as_str())).map(|line| return String::from(line));
}

/// Take one mount down by its path,
/// which is this arm's own cleanup and never the route's unmount.
fn unmount_at(folder: &Path) {
    let status: std::process::ExitStatus = Command::new("umount").arg(folder).status().expect("the cleanup unmount starts");
    assert!(status.success(), "the control's own cleanup could not unmount {}", folder.display());
}

/// Whether any mount in this process's table names the dataset as its source.
fn mounted(dataset: &str) -> bool {
    let table: String = std::fs::read_to_string("/proc/self/mountinfo").expect("the arm reads this process's mount table");
    return table.lines().any(|line| return line.contains(&format!(" - zfs {dataset} ")));
}

/// Write the handler configuration the sweep reads.
fn write_configuration(checkout: &Path) {
    let directory: &Path = Path::new(CONFIGURATION).parent().expect("the configuration has a folder");
    std::fs::create_dir_all(directory).expect("the configuration folder is made");
    let text: String = format!("{{\n  // written by plan step 9's holder-scan arm\n  \"cloneMounts\": {{ \"checkout\": \"{}\" }}\n}}\n", checkout.display());
    std::fs::write(CONFIGURATION, text).expect("the configuration is written");
    std::fs::set_permissions(CONFIGURATION, std::fs::Permissions::from_mode(0o644)).expect("the configuration holds its mode");
}

/// Write the start probe's record for this boot as a passed one,
/// in the mode's own shape and with the live fingerprint.
fn write_passing_probe_record() {
    let live: ProbeFingerprint = ProbeFingerprint::read(&FingerprintPaths::live()).expect("the guest's fingerprint reads");
    std::fs::create_dir_all(PROBE_RECORD_DIRECTORY).expect("the probe record's folder is made");
    let text: String = format!(
        "{RECORD_VERSION}\nkind {RECORD_KIND}\nstate finished\nboot {}\nkernel {}\nmodule {}\nsrcversion {}\nbuild-id {}\nhandler {}\ntime 2026-11-22T00:00:00+00:00\nverdict passed\nverdict-step none\nskip-reason none\n",
        live.boot_id,
        live.kernel_release,
        live.module_version.as_deref().unwrap_or(ABSENT),
        live.module_srcversion.as_deref().unwrap_or(ABSENT),
        live.module_build_id.as_deref().unwrap_or(ABSENT),
        live.handler.as_deref().unwrap_or(ABSENT)
    );
    let record: PathBuf = Path::new(PROBE_RECORD_DIRECTORY).join("result");
    std::fs::write(record.as_path(), text).expect("the probe record is written");
    std::fs::set_permissions(record.as_path(), std::fs::Permissions::from_mode(0o644)).expect("the probe record holds its mode");
    let _cleared: std::io::Result<()> = std::fs::remove_file(FIRST_ATTACH);
}

/// Publish the parent record the setup run would have written,
/// and clear every slot record so this arm starts from no record.
fn prepare_the_store(checkout: &Path) {
    let _cleared: std::io::Result<()> = std::fs::remove_dir_all(STORE);
    std::fs::create_dir_all(STORE).expect("the store folder is made");
    std::fs::set_permissions(STORE, std::fs::Permissions::from_mode(0o755)).expect("the store folder is root's and readable");
    let parent: PathBuf = checkout.join(".claude").join("worktrees");
    let identity: CloneFolderIdentity = CloneFolderIdentity::capture(&std::fs::File::open(parent.as_path()).expect("the parent folder opens")).expect("the parent identity reads");
    publish_parent_identity_under_root_control(Path::new(STORE), &identity, &identity).expect("the parent record is published");
}

/// A slot folder Git registered,
/// holding the pointer this checkout's registration gives it.
fn make_the_slot(checkout: &Path) {
    let folder: PathBuf = checkout.join(".claude").join("worktrees").join(SLOT);
    std::fs::create_dir_all(folder.as_path()).expect("the slot folder is made");
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the slot folder holds its mode");
    let pointer: String = format!("gitdir: {}\n", checkout.join(".git").join("worktrees").join(SLOT).display());
    std::fs::write(folder.join(".git"), pointer).expect("the slot's Git pointer is written");
}

/// The design's layout in this guest:
/// a checkout dataset mounted at the configured path,
/// its private propagation,
/// and a sibling container.
fn make_the_designs_layout(sbin: &str, pool: &str, checkout: &Path) -> (String, String) {
    let base: String = format!("{pool}/step9");
    let checkout_dataset: String = format!("{base}/{CHECKOUT_LEAF}");
    let container: String = format!("{base}/{CONTAINER_LEAF}");
    zfs_ignored(sbin, &["destroy", "-r", "-f", base.as_str()]);
    zfs_ok(sbin, &["create", "-p", "-o", "mountpoint=none", base.as_str()]);
    let mountpoint: String = format!("mountpoint={}", checkout.display());
    zfs_ok(sbin, &["create", "-o", mountpoint.as_str(), checkout_dataset.as_str()]);
    command_ok("mount", &["--make-private", checkout.display().to_string().as_str()]);
    zfs_ok(sbin, &["create", "-o", "mountpoint=legacy", "-o", "canmount=off", "-o", "setuid=off", "-o", "devices=off", container.as_str()]);
    std::fs::create_dir_all(checkout.join(".claude").join("worktrees").as_path()).expect("the checkout chain is made");
    return (checkout_dataset, container);
}

/// One clone of the checkout dataset's fresh snapshot at the slot,
/// asking for its mount,
/// which is the shape the design's add route makes.
fn clone_the_checkouts_snapshot(sbin: &str, checkout_dataset: &str, container: &str) -> String {
    let snapshot: String = format!("{checkout_dataset}@{ORIGIN_PREFIX}{NONCE}");
    zfs_ok(sbin, &["snapshot", snapshot.as_str()]);
    let clone: String = format!("{container}/{SLOT}");
    zfs_ok(sbin, &["clone", snapshot.as_str(), clone.as_str()]);
    zfs_ok(sbin, &["set", format!("{DESIRED_STATE}=v1:{NONCE}:mount").as_str(), clone.as_str()]);
    return clone;
}

/// The status and both streams of one handler run.
struct Run {
    /// The status the handler exited with.
    status: Option<i32>,
    /// What it wrote on both streams.
    said: String,
}

/// Run the built handler once with the given words.
fn handler(arguments: &[&str]) -> Run {
    let binary: &str = env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets");
    let output: Output = Command::new(binary).args(arguments).output().expect("the handler starts");
    return Run {
        status: output.status.code(),
        said: format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice())),
    };
}

/// The result's own lines of one answer,
/// which are every line but the summary the mode adds.
fn result_text(said: &str) -> Option<String> {
    let lines: Vec<&str> = said.lines().collect();
    let start: usize = lines.iter().position(|line| return *line == "v1")?;
    let end_of_time: usize = lines.iter().rposition(|line| return line.starts_with("time "))?;
    return Some(format!("{}\n", lines[start..=end_of_time].join("\n")));
}

/// One scan of the dataset the mode answers about,
/// read back from the result's own lines.
fn scan_of(clone: &str, checkout: &Path) -> (Run, Option<HolderScan>) {
    let checkout_text: String = checkout.display().to_string();
    let run: Run = handler(&["holder-scan", "--dataset", clone, "--checkout", checkout_text.as_str()]);
    let read: Option<HolderScan> = result_text(run.said.as_str()).and_then(|text| return parse_holder_scan(text.as_str()).ok());
    return (run, read);
}

/// The number of live tasks the kernel lists,
/// which is what one scan groups.
fn live_tasks() -> usize {
    let entries: std::fs::ReadDir = std::fs::read_dir("/proc").expect("the arm lists /proc");
    return entries.filter_map(|entry| return entry.ok()).filter(|entry| return entry.file_name().to_string_lossy().bytes().all(|byte| return byte.is_ascii_digit())).count();
}

/// What:
///  One task the arm started in a mount namespace of its own.
/// Why:
///  `unshare` forks the task and waits for it, so the launcher's pid is in the
///  host's namespace and killing it leaves the task that keeps the namespace
///  alive.
///  The task writes its own pid into a file and then execs into its work,
///  so the arm signals the task itself and a namespace ends when the arm means
///  it to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Task = { launcher: ChildProcess; pid: number; namespace: string };
/// ```
struct Task {
    /// The launcher,
    /// which waits for the task and exits with it.
    launcher: Child,
    /// The task's own pid,
    /// which is in the namespace the launcher made.
    pid: u32,
    /// The task's mount namespace identity,
    /// read while the task is alive.
    namespace: String,
}

/// Start one task in its own mount namespace,
/// bounded: it writes its pid first, and the arm reads that pid and the
/// namespace it is in.
fn start_task(script: &str, pid_file: &str, sbin: &str, unshare_arguments: &[&str]) -> Option<Task> {
    let _removed: std::io::Result<()> = std::fs::remove_file(pid_file);
    let written: String = format!("echo $$ > {pid_file}; {script}");
    let path: String = format!("{sbin}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
    let mut words: Vec<&str> = unshare_arguments.to_vec();
    words.extend(["--mount", "sh", "-c", written.as_str()]);
    let launcher: Child = Command::new("unshare")
        .args(words.as_slice())
        .env("PATH", path)
        .env("LD_LIBRARY_PATH", format!("{}/lib", sbin.trim_end_matches("/sbin")))
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|error| panic!("the task's unshare could not start: {error}"));
    let mut task: Task = Task { launcher, pid: 0, namespace: String::new() };
    let deadline: Instant = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        let found: Option<(u32, std::path::PathBuf)> = std::fs::read_to_string(pid_file)
            .ok()
            .and_then(|text| return text.trim().parse::<u32>().ok())
            .and_then(|pid| return std::fs::read_link(format!("/proc/{pid}/ns/mnt")).ok().map(|link| return (pid, link)));
        if let Some((pid, link)) = found {
            task.pid = pid;
            task.namespace = link.to_string_lossy().into_owned();
            return Some(task);
        }
        if let Ok(Some(_status)) = task.launcher.try_wait() {
            return None;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    return None;
}

/// Wait,
/// bounded,
/// for a task's own mount table to mention one folder,
/// which is how an arm knows the task it started has made its own mount and
/// that the scan is not racing that mount.
fn wait_for_the_task_table_to_mention(task: &Task, needle: &str) -> bool {
    let deadline: Instant = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        let mentions: bool = std::fs::read_to_string(format!("/proc/{}/mountinfo", task.pid)).is_ok_and(|text| return text.contains(needle));
        if mentions {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    return false;
}

/// End one task the arm started,
/// and reap the launcher that waited for it.
fn end(task: &mut Task) {
    let _signalled: std::process::ExitStatus = Command::new("sh").args(["-c", format!("kill -TERM {}", task.pid).as_str()]).status().expect("the signal starts");
    let _ended: bool = wait_for_the_end(task.pid);
    let _reaped = task.launcher.wait();
}

/// Wait,
/// bounded,
/// for a task to be gone,
/// which is how the arm knows the namespace it was in has no live member left.
fn wait_for_the_end(pid: u32) -> bool {
    let deadline: Instant = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if !Path::new(format!("/proc/{pid}").as_str()).exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    return false;
}

/// The task number of one launcher,
/// which is a pid in this namespace and never the pid of a task it started.
fn pid_of(child: &Child) -> u32 {
    return child.id();
}

/// Stop one child this arm started,
/// and wait for it.
fn stop(child: &mut Child) {
    let _killed = child.kill();
    let _waited = child.wait();
}

/// What:
///  Every holder arm of plan step 9 in one run of the built programs.
/// Why:
///  The arms change the guest's mounts and datasets,
///  so one arm owns them for the whole run and the driver's markers say which
///  arm ran.
#[test]
fn the_holder_scan_finds_each_holder_the_guest_can_make() {
    if !guest() {
        return;
    }
    let pool: String = required(POOL_VARIABLE);
    let sbin: String = required(SBIN_VARIABLE);
    let checkout: PathBuf = PathBuf::from(required(CHECKOUT_VARIABLE));
    let (checkout_dataset, container): (String, String) = make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
    make_the_slot(checkout.as_path());
    write_passing_probe_record();
    prepare_the_store(checkout.as_path());
    write_configuration(checkout.as_path());
    let clone: String = clone_the_checkouts_snapshot(sbin.as_str(), checkout_dataset.as_str(), container.as_str());

    // The clone is attached by the built sweep, which is where the scan's question starts.
    let sweep: Run = handler(&[]);
    assert_eq!(sweep.status, Some(0), "the built sweep attached the design's clone: {}", sweep.said);
    assert!(mounted(clone.as_str()), "the clone is mounted in the handler's own namespace: {}", sweep.said);
    assert!(mount_row(checkout.join(".claude").join("worktrees").join(SLOT).as_path()).is_some(), "the slot is the mount point");

    // Arm 1: the handler's own namespace is not a foreign holder, and the scan answers under the clone's GUID.
    let (baseline_run, baseline): (Run, Option<HolderScan>) = scan_of(clone.as_str(), checkout.as_path());
    assert_eq!(baseline_run.status, Some(0), "the mode answers: {}", baseline_run.said);
    let baseline_scan: HolderScan = baseline.unwrap_or_else(|| panic!("the mode's answer holds a result this build parses: {}", baseline_run.said));
    assert_eq!(baseline_scan.own_mounts.len(), 1, "the clone is in the handler's own namespace");
    assert_eq!(baseline_scan.holder_namespaces(), 0, "no other namespace holds it yet");
    let unknown: Vec<&git_policy_cli::clone_holder_scan::IncompleteRead> =
        baseline_scan.incomplete.iter().filter(|read| return !matches!(read, git_policy_cli::clone_holder_scan::IncompleteRead::TaskEnded { .. })).collect();
    assert!(unknown.is_empty(), "no table was left unread in the baseline: {unknown:?}");
    let guid: String = zfs(sbin.as_str(), "zfs", &["get", "-H", "-o", "value", "guid", clone.as_str()], None).1.trim().to_string();
    assert_eq!(baseline_scan.dataset.file_name(), guid, "the result is bound to the clone's live GUID");
    println!(
        "SCAN baseline own={} holders={} incomplete={} guid={guid}",
        baseline_scan.own_mounts.len(),
        baseline_scan.holder_namespaces(),
        baseline_scan.incomplete_reads()
    );

    // Arm 2: a second mount namespace holding the clone is found, under its own namespace identity.
    std::fs::create_dir_all(HOLDER_FOLDER).expect("the holder's folder is made");
    let holder_script: String = format!("mount -t zfs {clone} {HOLDER_FOLDER} && exec sleep {HOLDER_SECONDS}");
    let mut holder: Task = start_task(holder_script.as_str(), "/root/holder-scan-holder.pid", sbin.as_str(), &[])
        .unwrap_or_else(|| panic!("the holder's mount namespace could not be made; see the arm's log for the mount's own refusal"));
    assert!(wait_for_the_task_table_to_mention(&holder, HOLDER_FOLDER), "the holder's own table shows its mount at {HOLDER_FOLDER}");
    let (holder_run, held): (Run, Option<HolderScan>) = scan_of(clone.as_str(), checkout.as_path());
    assert_eq!(holder_run.status, Some(0), "the mode answers: {}", holder_run.said);
    let held_scan: HolderScan = held.unwrap_or_else(|| panic!("the mode's answer holds a result this build parses: {}", holder_run.said));
    let named_holder: Option<&git_policy_cli::clone_holder_scan::NamespaceHolder> = held_scan.holders.iter().find(|found| return found.namespace() == holder.namespace);
    let foreign: &git_policy_cli::clone_holder_scan::NamespaceHolder =
        named_holder.unwrap_or_else(|| panic!("the holder's own namespace {} is a holder: {:?}", holder.namespace, held_scan.holders));
    assert_ne!(foreign.namespace(), held_scan.own_namespace.as_deref().unwrap_or_default(), "the holder is not the handler's own namespace");
    println!("SCAN HOLDER other-namespace={} namespace={} mount={}", held_scan.holder_namespaces(), foreign.namespace(), foreign.mounts[0].mount_point.display());

    // Arm 3: a confined consumer holding a bind of the clone,
    // or a line saying this guest could not make one.
    let confined: Option<Task> = confined_consumer(clone.as_str(), checkout.as_path(), sbin.as_str());
    match confined {
        Some(mut consumer) => {
            assert!(wait_for_the_task_table_to_mention(&consumer, CONFINED_FOLDER), "the confined consumer's own table shows its bind at {CONFINED_FOLDER}");
            let (confined_run, after): (Run, Option<HolderScan>) = scan_of(clone.as_str(), checkout.as_path());
            assert_eq!(confined_run.status, Some(0), "the mode answers: {}", confined_run.said);
            let after_scan: HolderScan = after.unwrap_or_else(|| panic!("the mode's answer holds a result this build parses: {}", confined_run.said));
            let found: Option<&git_policy_cli::clone_holder_scan::NamespaceHolder> = after_scan.holders.iter().find(|found| return found.namespace() == consumer.namespace);
            assert!(found.is_some(), "the confined consumer's own namespace {} is a holder: {:?}", consumer.namespace, after_scan.holders);
            assert_eq!(after_scan.holder_namespaces(), 2, "the confined consumer is a second holder: {:?}", after_scan.holders);
            println!("SCAN confined-consumer=held holders={} namespace={}", after_scan.holder_namespaces(), consumer.namespace);
            end(&mut consumer);
        }
        None => {
            println!("SCAN confined-consumer=unavailable this guest refused an unprivileged user namespace");
        }
    };

    // Arm 4: task churn while the scan runs never hides the holder the scan already saw.
    let mut churn: Child = start_churn();
    let mut churned_holder_seen: usize = 0;
    let mut churned_incomplete: usize = 0;
    for _scan in 0..CHURN_SCANS {
        let (_run, scanned): (Run, Option<HolderScan>) = scan_of(clone.as_str(), checkout.as_path());
        let scanned_scan: HolderScan = scanned.unwrap_or_else(|| panic!("every answer under churn holds a result this build parses"));
        assert_eq!(scanned_scan.own_mounts.len(), 1, "the clone stays in the handler's own namespace under churn");
        if scanned_scan.holder_namespaces() >= 1 {
            churned_holder_seen += 1;
        }
        churned_incomplete += scanned_scan.incomplete.iter().filter(|read| return matches!(read, git_policy_cli::clone_holder_scan::IncompleteRead::TaskEnded { .. })).count();
    }
    stop(&mut churn);
    assert_eq!(churned_holder_seen, CHURN_SCANS, "churn never hid a holder the scan had already found");
    println!("SCAN churn scans={CHURN_SCANS} holder-kept={churned_holder_seen} task-ended-named={churned_incomplete}");

    // Arm 5: how one scan's cost grows with the number of live tasks.
    let small_tasks: usize = live_tasks();
    let small: Duration = time_a_scan(clone.as_str(), 5);
    let mut many: Vec<Child> = Vec::with_capacity(CHURN_TASKS);
    for _task in 0..CHURN_TASKS {
        many.push(Command::new("sleep").arg(HOLDER_SECONDS.to_string().as_str()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("a task starts"));
    }
    let large_tasks: usize = live_tasks();
    let large: Duration = time_a_scan(clone.as_str(), 5);
    for child in many.iter_mut() {
        stop(child);
    }
    println!(
        "MEASUREMENT holder-scan-ms=small={} large={} tasks-small={small_tasks} tasks-large={large_tasks}",
        small.as_millis(),
        large.as_millis()
    );

    // Arm 6: a namespace pinned by a handle and no task is invisible,
    // which is the residual the destroy-anyway decision accepts rather than hides.
    let held_pid: u32 = holder.pid;
    let held_namespace: String = holder.namespace.clone();
    let mut pinned: Child = pin_the_namespace(held_pid);
    assert!(wait_for_the_namespace_handle(pid_of(&pinned), held_namespace.as_str()), "the pinning task holds the namespace's handle");
    end(&mut holder);
    assert!(wait_for_the_end(held_pid), "the holder's task has ended");
    let (pinned_run, pinned_scan): (Run, Option<HolderScan>) = scan_of(clone.as_str(), checkout.as_path());
    assert_eq!(pinned_run.status, Some(0), "the mode answers: {}", pinned_run.said);
    let pinned_result: HolderScan = pinned_scan.unwrap_or_else(|| panic!("the mode's answer holds a result this build parses: {}", pinned_run.said));
    assert_eq!(pinned_result.holder_namespaces(), 0, "a namespace with no live task is in no file the scan reads: {:?}", pinned_result.holders);
    println!(
        "SCAN pinned-namespace invisible=true holders={} incomplete={} note=the-scan-cannot-see-a-namespace-no-task-holds",
        pinned_result.holder_namespaces(),
        pinned_result.incomplete_reads()
    );
    stop(&mut pinned);

    // Arm 7: a task with a divergent process root still names its namespace's mount of the clone.
    let mut rooted: Task = divergent_root(sbin.as_str(), clone.as_str())
        .unwrap_or_else(|| panic!("the divergent-root task's mount namespace could not be made; see the arm's log for the refusal"));
    if !wait_for_the_task_table_to_mention(&rooted, ROOTED_FOLDER) {
        let table: String = std::fs::read_to_string(format!("/proc/{}/mountinfo", rooted.pid)).unwrap_or_else(|error| return format!("unreadable: {error}"));
        println!(
            "SCAN divergent-root=unavailable pid={} alive={} mount-table={table}",
            rooted.pid,
            Path::new(format!("/proc/{}", rooted.pid).as_str()).exists()
        );
    }
    assert!(wait_for_the_task_table_to_mention(&rooted, ROOTED_FOLDER), "the divergent-root task's own table shows its mount at {ROOTED_FOLDER}");
    let (rooted_run, root_scan): (Run, Option<HolderScan>) = scan_of(clone.as_str(), checkout.as_path());
    assert_eq!(rooted_run.status, Some(0), "the mode answers: {}", rooted_run.said);
    let rooted_scan: HolderScan = root_scan.unwrap_or_else(|| panic!("the mode's answer holds a result this build parses: {}", rooted_run.said));
    let named: Option<&git_policy_cli::clone_holder_scan::NamespaceHolder> = rooted_scan.holders.iter().find(|found| return found.namespace() == rooted.namespace);
    let rooted_holder: &git_policy_cli::clone_holder_scan::NamespaceHolder = named
        .unwrap_or_else(|| panic!("a holder read through a divergent root is a holder under its own namespace {}: {:?}", rooted.namespace, rooted_scan.holders));
    let observed: String = rooted_holder.mounts.iter().map(|mount| return mount.mount_point.display().to_string()).collect::<Vec<String>>().join(",");
    println!("SCAN divergent-root holders={} namespace={} mount-points={observed}", rooted_scan.holder_namespaces(), rooted.namespace);
    end(&mut rooted);

    // Cleanup this arm owns: its own holders are gone, and the clone is left for the guest's own destruction.
    if mounted(clone.as_str()) {
        let slot: PathBuf = checkout.join(".claude").join("worktrees").join(SLOT);
        unmount_at(slot.as_path());
    }
    println!("SCAN done");
}

/// The duration of one scan of the dataset,
/// averaged over a few runs.
fn time_a_scan(clone: &str, rounds: usize) -> Duration {
    let nonce: RequestNonce = RequestNonce::parse(NONCE).expect("the fixture nonce is one");
    let dataset: DatasetGuid = DatasetGuid::parse("42").expect("a GUID that only times the scan");
    let started: Instant = Instant::now();
    for _round in 0..rounds {
        let scan: HolderScan = scan_holders(&HolderScanRequest {
            proc_root: Path::new("/proc"),
            source: clone,
            own_namespace: None,
            standing_unmount: None,
            dataset,
            nonce: &nonce,
            time: now_iso(),
        });
        assert!(!scan.source.is_empty(), "each timed scan answers a result");
    }
    return started.elapsed() / u32::try_from(rounds).unwrap_or(1);
}

/// A confined consumer:
/// an unprivileged user namespace whose own mount namespace binds the clone's
/// root somewhere else,
/// which is a holder this route cannot end without --force.
fn confined_consumer(clone: &str, checkout: &Path, sbin: &str) -> Option<Task> {
    let _unused_clone = clone;
    let slot: PathBuf = checkout.join(".claude").join("worktrees").join(SLOT);
    let _made: std::io::Result<()> = std::fs::create_dir_all(CONFINED_FOLDER);
    let script: String = format!("mount --bind {} {CONFINED_FOLDER} && exec sleep {HOLDER_SECONDS}", slot.display());
    return start_task(script.as_str(), "/root/holder-scan-confined.pid", sbin, &["--user", "--map-root-user"]);
}




/// A task churn:
/// short-lived processes arriving and leaving while the scan runs.
fn start_churn() -> Child {
    return Command::new("sh")
        .args(["-c", "while : ; do /bin/true; done"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the churn's shell starts");
}

/// A holder whose process root is not `/`:
/// the clone is mounted,
/// the guest's root is bound inside it,
/// and the task then chroots into that bind,
/// so its mount table's paths are rendered from another root.
fn divergent_root(sbin: &str, clone: &str) -> Option<Task> {
    std::fs::create_dir_all(ROOTED_BIND).expect("the divergent root's folder is made");
    // The task's root becomes a bind of the guest's root, so every path its mount table renders is rendered from
    // that root, and the clone is mounted inside it so the mount itself is a mount of that root's tree.
    // `mount` searches its own helper folders and not the path, and this guest's OpenZFS is under /opt, so the task
    // calls the helper by its own path, which the bind carries into the root it runs in.
    let inner: String = format!("{sbin}/mount.zfs {clone} {ROOTED_FOLDER} && exec sleep {HOLDER_SECONDS}");
    // The mount point is made inside the bind, because the task that mounts the clone runs there.
    let script: String = format!("mount --bind / {ROOTED_BIND} && mkdir --parents {ROOTED_BIND}{ROOTED_FOLDER} && exec chroot {ROOTED_BIND} sh -c '{inner}'");
    return start_task(script.as_str(), "/root/holder-scan-rooted.pid", sbin, &[]);
}

/// A task that holds another task's mount namespace open by a handle,
/// so the namespace outlives its last task.
fn pin_the_namespace(held_pid: u32) -> Child {
    let script: String = format!("exec 3</proc/{held_pid}/ns/mnt; exec sleep {HOLDER_SECONDS}");
    return Command::new("sh").args(["-c", script.as_str()]).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("the pinning task starts");
}

/// Wait,
/// bounded,
/// for the pinning task to hold the namespace's handle.
fn wait_for_the_namespace_handle(pinning_pid: u32, namespace: &str) -> bool {
    let deadline: Instant = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        let folder: String = format!("/proc/{pinning_pid}/fd");
        if let Ok(entries) = std::fs::read_dir(folder.as_str()) {
            for entry in entries.filter_map(|entry| return entry.ok()) {
                let holds: bool = std::fs::read_link(entry.path()).is_ok_and(|target| return target.to_string_lossy() == namespace);
                if holds {
                    return true;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    return false;
}
