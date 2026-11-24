//! What:
//!  Plan step 8's arms in the throwaway guest:
//!  the built handler's own sweep against the design's layout,
//!  a clone of the checkout dataset's snapshot carrying a mount request,
//!  the same sweep again with the clone left attached,
//!  the pool's key unloaded and loaded,
//!  the handler's refusals,
//!  the descriptor unmount a rollback makes,
//!  the child bound the listing takes,
//!  the four shapes that name no checkout,
//!  with a second pool holding the design's leaf name,
//!  and the `snapdir` the route's own dataset-stats request carries.
//! Why:
//!  The sweep mounts a real clone as root,
//!  writes the store under `/var/lib` and files receipts under `/run`,
//!  so no control on a person's machine may run it:
//!  these arms run only where `CLI_GIT_ZFS_SWEEP_GUEST` names a disposable guest
//!  (`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`,
//!  on OpenZFS 2.4.4 in an `mvm` Ubuntu guest),
//!  and they print `SKIP` everywhere else,
//!  which is why the arm carries no `#[ignore]`:
//! a harness that skips an ignored
//!  test would run it nowhere,
//!  so the gate says what it did not run
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 8,
//!  "Acceptance tests" and "VM measurements").
//!  The arm drives the built executable rather than a module,
//!  because the sweep's order and its receipts are what an operator and a
//!  waiting caller read.
//!  The fixture is the design's own layout:
//!  a checkout dataset mounted at the configured path,
//!  a sibling container,
//!  and a clone of the checkout's `git-worktree-clone-origin-` snapshot,
//!  which is what the add route makes
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Dataset and path layout").
//!  The configuration names only the checkout,
//!  so the run reads the two dataset names through the design's fallback,
//!  and the checkout's own mount is made private because the design's route
//!  requires a not-shared immediate parent.
//!  Four further shapes name no checkout at all,
//!  one of them no file,
//!  and each has to attach the designed clone on its own run
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "With no configuration file").
//!  Item 1 and item 5 are repeated through the handler,
//!  and the key arm runs both halves:
//!  the refusal while the key is unloaded,
//!  and the attach after the load,
//!  so a run that skipped the unloaded half fails instead of reporting a green
//!  suite.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await sweep('one');                  // item 1, the design's origin
//! await sweep('one');                  // the same clone left attached
//! await unloadKey(encrypted); await sweep('two'); await loadKey(encrypted); await sweep('two'); // item 5
//! ```

use git_policy_cli::start_probe::probe_fingerprint::{FingerprintPaths, ProbeFingerprint};
use git_policy_cli::worktree_clone_mount_handler::sweep_dataset_facts::{PropertyValue, value_of};
use git_policy_cli::worktree_clone_mount_handler::sweep_dataset_reader::{DatasetReader, ZfsDatasetReader};
use git_policy_cli::start_probe::probe_record::{ABSENT, RECORD_KIND, RECORD_VERSION};
use git_policy_cli::worktree_clone_mount::{
    CloneFolderIdentity, HoldCloneSlotRequest, SlotIdentityStore, attach_the_clone, observe_the_clone_slot_under_root_control, publish_parent_identity_under_root_control, unmount_through_descriptor,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::ZfsDevice;
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The variable the guest driver sets.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_GUEST";

/// The pool the driver prepared.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_POOL";

/// The OpenZFS userspace the guest's commands need.
const SBIN_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_SBIN";

/// The checkout the sweep walks.
const CHECKOUT_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_CHECKOUT";

/// The passphrase the encrypted fixture's key is loaded with.
const PASSPHRASE_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_PASSPHRASE";

/// The checkout dataset's leaf name the design gives it.
const CHECKOUT_LEAF: &str = "monochromatic-git-checkout";

/// The container's leaf name the design gives it.
const CONTAINER_LEAF: &str = "monochromatic-git-worktree-clone-datasets";

/// The first slot,
/// which the design's clone attaches at.
const SLOT: &str = "one";

/// The second slot the key arm uses.
const SLOT_TWO: &str = "two";

/// The prefix the add route gives its origin snapshots.
const ORIGIN_PREFIX: &str = "git-worktree-clone-origin-";

/// The nonce the first arm's request carries.
const NONCE: &str = "00112233445566778899aabbccddeeff";

/// The passphrase when the driver names none,
/// which the guest's throwaway pool accepts and no other machine ever sees.
const DEFAULT_PASSPHRASE: &str = "zfs-step-8-sweep";

/// The handler configuration the sweep reads inside the guest.
const CONFIGURATION: &str = "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc";

/// The start probe's record directory inside the guest.
const PROBE_RECORD_DIRECTORY: &str = "/run/git-worktree-clone-mount-probe";

/// The first attach's record,
/// beside the probe's.
const FIRST_ATTACH: &str = "/run/git-worktree-clone-mount-probe/first-attach";

/// The desired-state property.
const DESIRED_STATE: &str = "org.monochromatic:git-worktree-clone-mount-desired-state";

/// The store the sweep records into.
const STORE: &str = "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities";

/// Whether this run is inside the guest,
/// or a skip line on every other machine.
fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP worktree_clone_mount_handler::sweep_guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest");
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
        pipe.write_all(text.as_bytes()).expect("the passphrase reaches the key command");
    }
    let output: Output = child.wait_with_output().unwrap_or_else(|error| panic!("{program} {arguments:?} could not be waited for: {error}"));
    return (
        output.status.success(),
        String::from_utf8_lossy(output.stdout.as_slice()).to_string(),
        String::from_utf8_lossy(output.stderr.as_slice()).to_string(),
    );
}

/// Run one `zfs` command and require it succeeded.
fn zfs_ok(sbin: &str, arguments: &[&str]) {
    let (ok, _out, error): (bool, String, String) = zfs(sbin, "zfs", arguments, None);
    assert!(ok, "zfs {arguments:?} was refused: {error}");
}

/// Run one `zfs` command whose failure this arm expects.
fn zfs_ignored(sbin: &str, arguments: &[&str]) {
    let (_ok, _out, _error): (bool, String, String) = zfs(sbin, "zfs", arguments, None);
}

/// Run one program and require it succeeded.
fn command_ok(program: &str, arguments: &[&str]) {
    let status: std::process::ExitStatus = Command::new(program).args(arguments).status().unwrap_or_else(|error| panic!("{program} {arguments:?} could not start: {error}"));
    assert!(status.success(), "{program} {arguments:?} was refused");
}

/// Whether any mount claims the dataset as its source.
fn mounted(dataset: &str) -> bool {
    let table: String = std::fs::read_to_string("/proc/self/mountinfo").expect("the arm reads this process's mount table");
    return table.lines().any(|line| return line.contains(&format!(" - zfs {dataset} ")));
}

/// The mount's own row for one folder,
/// or nothing when no mount stands there.
fn mount_row(folder: &Path) -> Option<String> {
    let table: String = std::fs::read_to_string("/proc/self/mountinfo").expect("the arm reads this process's mount table");
    let wanted: String = folder.display().to_string();
    return table.lines().find(|line| return line.split(' ').nth(4) == Some(wanted.as_str())).map(|line| return String::from(line));
}

/// Take one mount down by its path where the arm cannot be sure it stands,
/// which is cleanup of a fixture rather than a proof.
fn unmount_if_it_stands(folder: &Path) {
    let _ignored: std::io::Result<std::process::ExitStatus> = Command::new("umount").arg(folder).status();
}

/// Take one mount down by its path,
/// which is this arm's own cleanup and never the route's unmount.
fn unmount_at(folder: &Path) {
    let status: std::process::ExitStatus = Command::new("umount").arg(folder).status().expect("the cleanup unmount starts");
    assert!(status.success(), "the control's own cleanup could not unmount {}", folder.display());
}

/// What one sweep run answered.
struct Run {
    /// The status the handler exited with.
    status: Option<i32>,
    /// Its standard output and error,
    /// joined.
    said: String,
    /// How long it took.
    elapsed: Duration,
}

/// Run the built handler's default invocation,
/// which is the sweep.
fn sweep() -> Run {
    let binary: &str = env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets");
    let started: Instant = Instant::now();
    let output: Output = Command::new(binary).output().expect("the handler starts");
    return Run {
        status: output.status.code(),
        said: format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice())),
        elapsed: started.elapsed(),
    };
}

/// Run the built handler's `start-probe` mode,
/// which writes this boot's record.
fn start_probe() -> Run {
    let binary: &str = env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets");
    let started: Instant = Instant::now();
    let output: Output = Command::new(binary).arg("start-probe").output().expect("the probe mode starts");
    return Run {
        status: output.status.code(),
        said: format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice())),
        elapsed: started.elapsed(),
    };
}

/// The probe's record for this boot,
/// as the file it publishes holds it.
fn probe_record() -> String {
    return std::fs::read_to_string(Path::new(PROBE_RECORD_DIRECTORY).join("result")).expect("the probe wrote its record");
}

/// One line of the probe's record,
/// or nothing when it holds none.
fn record_line<'a>(record: &'a str, key: &str) -> Option<&'a str> {
    let prefix: String = format!("{key} ");
    return record.lines().find(|line| return line.starts_with(prefix.as_str())).map(|line| return &line[prefix.len()..]);
}

/// The line the sweep printed about one dataset,
/// or nothing when it printed none.
fn line_about<'a>(said: &'a str, dataset: &str) -> Option<&'a str> {
    return said.lines().find(|line| {
        return line.contains(dataset) && (line.starts_with("attached ") || line.starts_with("refused ") || line.starts_with("nothing ") || line.starts_with("failed ") || line.starts_with("held "));
    });
}

/// Write the handler configuration the sweep reads.
/// A name this arm does not give is left out,
/// which is the file the design's fallback reads.
fn write_configuration(checkout: &Path, parent: Option<&str>, checkout_dataset: Option<&str>) {
    let directory: &Path = Path::new(CONFIGURATION).parent().expect("the configuration has a folder");
    std::fs::create_dir_all(directory).expect("the configuration folder is made");
    let mut members: Vec<String> = vec![format!(r#""checkout": "{}""#, checkout.display())];
    if let Some(container) = parent {
        members.push(format!(r#""parent": "{container}""#));
    }
    if let Some(dataset) = checkout_dataset {
        members.push(format!(r#""checkoutDataset": "{dataset}""#));
    }
    let text: String = format!(
        r#"{{
  // written by plan step 8's guest arm
  "cloneMounts": {{ {} }}
}}
"#,
        members.join(", ")
    );
    std::fs::write(CONFIGURATION, text).expect("the configuration is written");
    std::fs::set_permissions(CONFIGURATION, std::fs::Permissions::from_mode(0o644)).expect("the configuration holds its mode");
}

/// What:
///  Write one whole configuration document the sweep reads.
/// Why:
///  The four shapes that name no checkout are documents of their own,
///  and a shape with no file is written by removing it.
fn write_document(text: &str) {
    let directory: &Path = Path::new(CONFIGURATION).parent().expect("the configuration has a folder");
    std::fs::create_dir_all(directory).expect("the configuration folder is made");
    std::fs::write(CONFIGURATION, text).expect("the configuration is written");
    std::fs::set_permissions(CONFIGURATION, std::fs::Permissions::from_mode(0o644)).expect("the configuration holds its mode");
}

/// Remove the handler configuration,
/// which is the shape the project's answer of 2026-11-22 is about.
fn remove_configuration() {
    let _removed: std::io::Result<()> = std::fs::remove_file(CONFIGURATION);
}

/// The dataset the design gives the checkout,
/// under the pool this guest made.
fn checkout_dataset_in(pool: &str) -> String {
    return format!("{pool}/step8/{CHECKOUT_LEAF}");
}

/// The container the design places beside it,
/// under the same pool.
fn container_in(pool: &str) -> String {
    return format!("{pool}/step8/{CONTAINER_LEAF}");
}

/// What:
///  Write the start probe's record for this boot as a passed one,
///  in the mode's own shape and with the live fingerprint.
/// Why:
///  The sweep compares the record's boot and its four fingerprint lines with
///  the running machine,
///  so a hand-written record that named no kernel or module would be stale and
///  the arm would prove nothing about the sweep.
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

/// What:
///  Publish the parent record the setup run would have written,
///  and clear every slot record so one arm does not inherit another's.
/// Why:
///  Plan step 19 owns the setup run,
///  and this arm is the sweep's:
///  the parent record is its input,
///  and a slot record left by an earlier arm would refuse the clone this arm
///  creates.
fn prepare_the_store(checkout: &Path) {
    let _cleared: std::io::Result<()> = std::fs::remove_dir_all(STORE);
    std::fs::create_dir_all(STORE).expect("the store folder is made");
    std::fs::set_permissions(STORE, std::fs::Permissions::from_mode(0o755)).expect("the store folder is root's and readable");
    let parent: PathBuf = checkout.join(".claude").join("worktrees");
    let identity: CloneFolderIdentity = CloneFolderIdentity::capture(&std::fs::File::open(parent.as_path()).expect("the parent folder opens")).expect("the parent identity reads");
    publish_parent_identity_under_root_control(Path::new(STORE), &identity, &identity).expect("the parent record is published");
}

/// What:
///  A slot folder Git registered,
///  holding the pointer this checkout's registration gives it.
/// Why:
///  The first observation reads the slot's `.git` and refuses a slot that holds
///  no registration of this checkout,
///  so an empty folder is not a fixture the sweep mounts.
fn make_the_slot(checkout: &Path, slot: &str) {
    let folder: PathBuf = checkout.join(".claude").join("worktrees").join(slot);
    std::fs::create_dir_all(folder.as_path()).expect("the slot folder is made");
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the slot folder holds its mode");
    let pointer: String = format!("gitdir: {}\n", checkout.join(".git").join("worktrees").join(slot).display());
    std::fs::write(folder.join(".git"), pointer).expect("the slot's Git pointer is written");
}

/// What:
///  The design's layout in this guest:
///  a checkout dataset mounted at the configured path,
///  its private propagation,
///  and a sibling container.
/// Why:
///  The origin rule is the checkout dataset's snapshot,
///  and the fallback finds the container beside the checkout dataset,
///  so the fixture is the design's layout rather than a folder on the guest's
///  root.
fn make_the_designs_layout(sbin: &str, pool: &str, checkout: &Path) -> (String, String) {
    let base: String = format!("{pool}/step8");
    let checkout_dataset: String = format!("{base}/{CHECKOUT_LEAF}");
    let container: String = format!("{base}/{CONTAINER_LEAF}");
    zfs_ignored(sbin, &["destroy", "-r", "-f", base.as_str()]);
    // The parent is made without the mountpoint, because `zfs create -p -o` applies its options to every parent it makes
    // and a second mount at the same path would hide the checkout.
    zfs_ok(sbin, &["create", "-p", "-o", "mountpoint=none", base.as_str()]);
    let mountpoint: String = format!("mountpoint={}", checkout.display());
    zfs_ok(sbin, &["create", "-o", mountpoint.as_str(), checkout_dataset.as_str()]);
    // The design requires a not-shared immediate parent, which its boot unit sets; this arm sets it for the checkout
    // alone, exactly as item 3 of the design's measurements did.
    command_ok("mount", &["--make-private", checkout.display().to_string().as_str()]);
    zfs_ok(sbin, &["create", "-o", "mountpoint=legacy", "-o", "canmount=off", "-o", "setuid=off", "-o", "devices=off", container.as_str()]);
    std::fs::create_dir_all(checkout.join(".claude").join("worktrees").as_path()).expect("the checkout chain is made");
    return (checkout_dataset, container);
}

/// What:
///  One clone of the checkout dataset's fresh snapshot at the slot,
///  asking for its mount,
///  which is the shape the design's add route makes.
/// Why:
///  The origin must be a snapshot of the checkout dataset under the add route's
///  prefix,
///  so the fixture is that snapshot and this is the clone of it.
fn clone_the_checkouts_snapshot(sbin: &str, checkout_dataset: &str, container: &str, slot: &str, snapshot_suffix: &str, intent: Option<&str>) -> String {
    let snapshot: String = format!("{checkout_dataset}@{ORIGIN_PREFIX}{snapshot_suffix}");
    zfs_ok(sbin, &["snapshot", snapshot.as_str()]);
    let clone: String = format!("{container}/{slot}");
    zfs_ok(sbin, &["clone", snapshot.as_str(), clone.as_str()]);
    if let Some(value) = intent {
        zfs_ok(sbin, &["set", format!("{DESIRED_STATE}={value}").as_str(), clone.as_str()]);
    }
    return clone;
}

/// The mount request one clone carries.
fn mount_request(nonce: &str) -> String {
    return format!("v1:{nonce}:mount");
}

/// What:
///  Every arm of plan step 8 in one run of the built handler:
///  the design's clone attached and its `done` receipt kept across a second
///  sweep with the mount standing,
///  the pool's key unloaded and loaded,
///  the handler's refusals,
///  the descriptor unmount a rollback makes,
///  and the child bound the listing takes.
/// Why:
///  Each arm changes the guest's datasets,
///  so one arm owns them for the whole run and the driver's markers say which
///  arm ran.
#[test]
fn the_built_sweep_attaches_the_designs_clone_and_refuses_each_mismatch() {
    if !guest() {
        return;
    }
    let pool: String = required(POOL_VARIABLE);
    let sbin: String = required(SBIN_VARIABLE);
    let checkout: PathBuf = PathBuf::from(required(CHECKOUT_VARIABLE));
    let passphrase: String = std::env::var(PASSPHRASE_VARIABLE).unwrap_or_else(|_error| return String::from(DEFAULT_PASSPHRASE));
    let (checkout_dataset, container): (String, String) = make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
    make_the_slot(checkout.as_path(), SLOT);
    write_passing_probe_record();
    prepare_the_store(checkout.as_path());
    let _device = ZfsDevice::open().expect("the guest permits the device");

    // The configuration names only the checkout, so the two dataset names come from the design's fallback.
    write_configuration(checkout.as_path(), None, None);
    let nonce: &str = NONCE;
    let clone: String = clone_the_checkouts_snapshot(sbin.as_str(), checkout_dataset.as_str(), container.as_str(), SLOT, nonce, Some(mount_request(nonce).as_str()));
    let slot_folder: PathBuf = checkout.join(".claude").join("worktrees").join(SLOT);

    // Item 1 and H1: a clone of the checkout's snapshot, at the design's layout, is attached by one sweep run.
    let first: Run = sweep();
    assert_eq!(first.status, Some(0), "a sweep that reached the end exits successfully: {}", first.said);
    let attached: &str = line_about(first.said.as_str(), clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {clone}: {}", first.said));
    assert!(attached.starts_with("attached "), "item 1's designed clone is attached: {attached}");
    println!("MEASUREMENT sweep-attach-ms={}", first.elapsed.as_millis());
    println!("SWEEP attached item=1 slot={SLOT} origin=checkout-snapshot mount-row={}", mount_row(slot_folder.as_path()).unwrap_or_default());
    let row: String = mount_row(slot_folder.as_path()).expect("the slot is a mount point after the attach");
    assert!(row.contains("nosuid"), "the mount carries nosuid: {row}");
    assert!(row.contains("nodev"), "the mount carries nodev: {row}");
    assert!(row.contains(&format!(" - zfs {clone} ")), "the mount's source is the clone: {row}");
    let store: SlotIdentityStore = SlotIdentityStore::open_under_root_control(Path::new(STORE)).expect("the store opens");
    let recorded: CloneFolderIdentity = store.slot(SLOT).expect("the sweep recorded the slot's first observation");
    let mounted_root: CloneFolderIdentity = CloneFolderIdentity::capture(&std::fs::File::open(slot_folder.as_path()).expect("the slot opens")).expect("the slot's identity reads");
    assert_ne!(recorded, mounted_root, "the record is the underlay the sweep observed before the attach");
    let guid: String = zfs(sbin.as_str(), "zfs", &["get", "-H", "-o", "value", "guid", clone.as_str()], None).1.trim().to_string();
    let receipt_path: String = format!("/run/git-worktree-clone-mount-results/{guid}");
    let receipt: String = std::fs::read_to_string(receipt_path.as_str()).expect("the sweep filed a receipt under the clone's GUID");
    assert!(receipt.contains("outcome done"), "the receipt says the mount was observed: {receipt}");
    assert!(receipt.contains(nonce), "the receipt carries the request's nonce: {receipt}");
    // M5: this boot's first attach records the verified mount and the flags read back.
    let first_attach: String = std::fs::read_to_string(FIRST_ATTACH).expect("this boot's first attach is recorded");
    assert!(first_attach.contains("kind operation-outcome"), "{first_attach}");
    assert!(first_attach.contains("outcome attached"), "the first attach records the mount: {first_attach}");
    assert!(first_attach.contains(&format!("mount ")), "{first_attach}");
    assert!(first_attach.contains("flags rw"), "the first attach records the flags read back: {first_attach}");
    assert!(first_attach.contains(&format!("slot {SLOT}")), "{first_attach}");
    assert!(first_attach.contains(&format!("clone {clone}")), "{first_attach}");
    assert!(first_attach.contains(&format!("guid {guid}")), "{first_attach}");
    println!("SWEEP first-attach outcome=attached slot={SLOT}");

    // H2: the second sweep with the mount standing answers attached and keeps the done receipt.
    let before: String = std::fs::read_to_string(receipt_path.as_str()).expect("the receipt reads");
    let second: Run = sweep();
    let again: &str = line_about(second.said.as_str(), clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {clone}: {}", second.said));
    assert!(again.starts_with("attached "), "a clone that stays attached is still attached: {again}");
    assert!(mount_row(slot_folder.as_path()).is_some(), "the sweep left the standing mount where it was");
    let after: String = std::fs::read_to_string(receipt_path.as_str()).expect("the receipt reads");
    assert!(after.contains("outcome done"), "the done receipt is kept rather than replaced by a refusal: {after}");
    assert!(after.contains(nonce), "the kept receipt carries the same nonce: {after}");
    assert!(after.contains("mount-id"), "the kept receipt still names the mount: {after}");
    assert!(before.contains("outcome done"), "the first receipt was a done one: {before}");
    println!("SWEEP attached twice=1 slot={SLOT} standing=true");
    println!("SWEEP twice-swept receipt=done nonce={nonce}");
    unmount_at(slot_folder.as_path());
    assert!(!mounted(clone.as_str()), "the control's cleanup took the mount down");
    // The clone keeps its request after the arm, so it is taken off before the later arms sweep this container.
    zfs_ok(sbin.as_str(), &["inherit", DESIRED_STATE, clone.as_str()]);
    let _slot_record_cleared: std::io::Result<()> = std::fs::remove_file(Path::new(STORE).join(SLOT));

    // The project's answer of 2026-11-22: a file that names no checkout still lets the sweep find the mounted checkout
    // dataset, the container beside it and the checkout path from that row. Each of the four shapes attaches the
    // designed clone on its own run.
    zfs_ok(sbin.as_str(), &["set", format!("{DESIRED_STATE}={}", mount_request(nonce)).as_str(), clone.as_str()]);
    let shapes: [(&str, Option<String>); 4] = [
        ("no-file", None),
        ("empty-object", Some(String::from("{}"))),
        (
            "only-start-probe",
            Some(format!(
                r#"{{ "startProbe": {{ "dataset": "{pool}/nosnap/git-worktree-probe", "checkout": "{}" }} }}"#,
                checkout.display()
            )),
        ),
        (
            "dataset-names",
            Some(format!(
                r#"{{ "cloneMounts": {{ "parent": "{}", "checkoutDataset": "{}" }} }}"#,
                container_in(pool.as_str()),
                checkout_dataset_in(pool.as_str())
            )),
        ),
    ];
    for (shape, document) in shapes {
        match document {
            Some(text) => write_document(text.as_str()),
            None => remove_configuration(),
        }
        let shaped: Run = sweep();
        assert_eq!(shaped.status, Some(0), "the shape {shape} reached the end of a sweep: {}", shaped.said);
        let found: &str = line_about(shaped.said.as_str(), clone.as_str()).unwrap_or_else(|| panic!("the shape {shape} printed no line about {clone}: {}", shaped.said));
        assert!(found.starts_with("attached "), "the shape {shape} attaches the designed clone: {found}");
        assert!(mount_row(slot_folder.as_path()).is_some(), "the shape {shape} left the mount standing: {}", shaped.said);
        println!("SWEEP no-configuration shape={shape} attached slot={SLOT}");
        unmount_at(slot_folder.as_path());
        let _cleared: std::io::Result<()> = std::fs::remove_file(Path::new(STORE).join(SLOT));
    }
    zfs_ok(sbin.as_str(), &["inherit", DESIRED_STATE, clone.as_str()]);

    // The "several" that no account needs root for: a second pool that holds a dataset of the design's own leaf name.
    // The sweep refuses, names both instances, and attaches nothing.
    let second_pool: &str = "git-policy-cli-second";
    let second_image: &str = "/home/git-policy-cli-second-pool.img";
    let second_path: &str = "/home/second-checkout";
    command_ok("truncate", &["--size", "256M", second_image]);
    let created: (bool, String, String) = zfs(sbin.as_str(), "zpool", &["create", "-f", "-O", "mountpoint=none", second_pool, second_image], None);
    assert!(created.0, "the second pool is created: {}", created.2);
    let second_dataset: String = format!("{second_pool}/{CHECKOUT_LEAF}");
    zfs_ok(sbin.as_str(), &["create", "-o", format!("mountpoint={second_path}").as_str(), second_dataset.as_str()]);
    remove_configuration();
    let several: Run = sweep();
    assert_eq!(several.status, Some(2), "two instances of the leaf name are a refusal: {}", several.said);
    for named in [checkout_dataset.as_str(), second_dataset.as_str()] {
        assert!(several.said.contains(named), "the refusal lists {named}: {}", several.said);
    }
    assert!(several.said.contains("will not choose between them"), "the refusal says it chose nothing: {}", several.said);
    assert!(several.said.contains(CHECKOUT_LEAF), "the refusal names the leaf it looked for: {}", several.said);
    assert!(mount_row(slot_folder.as_path()).is_none(), "a refusal attaches nothing: {}", several.said);
    assert!(!mounted(clone.as_str()), "a refusal mounts nothing: {}", several.said);
    println!("SWEEP no-configuration several=two-pools refused slot={SLOT} mounted=false");
    let removed: (bool, String, String) = zfs(sbin.as_str(), "zpool", &["destroy", "-f", second_pool], None);
    assert!(removed.0, "the second pool is destroyed: {}", removed.2);
    let _image_removed: std::io::Result<()> = std::fs::remove_file(second_image);

    // Item 5: an encrypted checkout dataset whose key is unloaded refuses as a key question and files a receipt naming
    // it, and the same sweep attaches after the load.
    let encrypted_base: String = format!("{pool}/step8-encrypted");
    zfs_ignored(sbin.as_str(), &["destroy", "-r", "-f", encrypted_base.as_str()]);
    let encrypted_checkout: String = format!("{encrypted_base}/{CHECKOUT_LEAF}");
    let encrypted_created: (bool, String, String) = zfs(
        sbin.as_str(),
        "zfs",
        // The dataset is never mounted, so its key can be unloaded: a mounted dataset's key cannot be.
        &["create", "-p", "-o", "mountpoint=none", "-o", "encryption=on", "-o", "keyformat=passphrase", "-o", "keylocation=prompt", encrypted_checkout.as_str()],
        Some(format!("{passphrase}\n{passphrase}\n").as_str()),
    );
    assert!(encrypted_created.0, "the encrypted fixture is created: {}", encrypted_created.2);
    let encrypted_container: String = format!("{encrypted_base}/{CONTAINER_LEAF}");
    zfs_ok(sbin.as_str(), &["create", "-o", "mountpoint=legacy", "-o", "canmount=off", "-o", "setuid=off", "-o", "devices=off", encrypted_container.as_str()]);
    make_the_slot(checkout.as_path(), SLOT_TWO);
    let encrypted_nonce: &str = "00112233445566778899aabbccddee01";
    let encrypted_clone: String = clone_the_checkouts_snapshot(sbin.as_str(), encrypted_checkout.as_str(), encrypted_container.as_str(), SLOT_TWO, encrypted_nonce, Some(mount_request(encrypted_nonce).as_str()));
    write_configuration(checkout.as_path(), Some(encrypted_container.as_str()), Some(encrypted_checkout.as_str()));
    // The checkout dataset is the encryption root, and it is not mounted, so its key can be unloaded and its clone's
    // keystatus reads unavailable.
    zfs_ok(sbin.as_str(), &["unload-key", encrypted_checkout.as_str()]);
    let unloaded: Run = sweep();
    let refused: &str = line_about(unloaded.said.as_str(), encrypted_clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {encrypted_clone}: {}", unloaded.said));
    assert!(refused.starts_with("refused ") && refused.contains("step=key-unavailable"), "an unloaded key is refused as itself: {refused}");
    assert!(!mounted(encrypted_clone.as_str()), "an unloaded key mounts nothing: {}", unloaded.said);
    let encrypted_guid: String = zfs(sbin.as_str(), "zfs", &["get", "-H", "-o", "value", "guid", encrypted_clone.as_str()], None).1.trim().to_string();
    let key_receipt: String = std::fs::read_to_string(format!("/run/git-worktree-clone-mount-results/{encrypted_guid}")).expect("an unavailable key files a receipt");
    assert!(key_receipt.contains("outcome refused"), "the key refusal files a refused receipt: {key_receipt}");
    assert!(key_receipt.contains("step key-unavailable"), "the receipt names the key step: {key_receipt}");
    println!("SWEEP refusal item=5 step=key-unavailable receipt=refused");
    let loaded_again: (bool, String, String) = zfs(sbin.as_str(), "zfs", &["load-key", encrypted_checkout.as_str()], Some(format!("{passphrase}\n").as_str()));
    assert!(loaded_again.0, "the key loads again: {}", loaded_again.2);
    let second_key: Run = sweep();
    let attached_again: &str = line_about(second_key.said.as_str(), encrypted_clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {encrypted_clone}: {}", second_key.said));
    assert!(attached_again.starts_with("attached "), "item 5's clone is attached after the key load: {attached_again}");
    println!("SWEEP attached item=5 slot={SLOT_TWO} mount-row={}", mount_row(checkout.join(".claude").join("worktrees").join(SLOT_TWO).as_path()).unwrap_or_default());
    unmount_at(checkout.join(".claude").join("worktrees").join(SLOT_TWO).as_path());
    assert!(!mounted(encrypted_clone.as_str()));
    let _second_record_cleared: std::io::Result<()> = std::fs::remove_file(Path::new(STORE).join(SLOT_TWO));

    // The handler's own refusals, on clones of the design's checkout snapshot in the plain container: a malformed
    // intent and an inherited one each authorize nothing.
    write_configuration(checkout.as_path(), None, None);
    let refusing_nonce: &str = "00112233445566778899aabbccddee03";
    let refusing_slot: &str = "six";
    make_the_slot(checkout.as_path(), refusing_slot);
    let refusing_clone: String = clone_the_checkouts_snapshot(sbin.as_str(), checkout_dataset.as_str(), container.as_str(), refusing_slot, refusing_nonce, None);
    let malformed: String = String::from("v1:nothex:mount");
    zfs_ok(sbin.as_str(), &["set", format!("{DESIRED_STATE}={malformed}").as_str(), refusing_clone.as_str()]);
    let malformed_run: Run = sweep();
    let refused_malformed: &str = line_about(malformed_run.said.as_str(), refusing_clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {refusing_clone}: {}", malformed_run.said));
    assert!(refused_malformed.starts_with("refused ") && refused_malformed.contains("step=intent"), "a malformed intent is refused before any work: {refused_malformed}");
    assert!(!mounted(refusing_clone.as_str()), "a malformed intent mounts nothing");
    println!("SWEEP refusal item=2 step=intent");
    zfs_ok(sbin.as_str(), &["inherit", DESIRED_STATE, refusing_clone.as_str()]);
    zfs_ok(sbin.as_str(), &["set", format!("{DESIRED_STATE}={}", mount_request(refusing_nonce)).as_str(), container.as_str()]);
    let inherited_run: Run = sweep();
    let nothing: &str = line_about(inherited_run.said.as_str(), refusing_clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {refusing_clone}: {}", inherited_run.said));
    assert!(nothing.starts_with("nothing ") && nothing.contains("inherited"), "an inherited intent authorizes nothing: {nothing}");
    println!("SWEEP nothing inherited=true");
    zfs_ok(sbin.as_str(), &["inherit", DESIRED_STATE, container.as_str()]);

    // A clone of the container's own snapshot is the shape H1 refused every designed clone as; it is not the add
    // route's origin, so it is refused as one.
    let nested_slot: &str = "three";
    make_the_slot(checkout.as_path(), nested_slot);
    zfs_ok(sbin.as_str(), &["snapshot", &format!("{container}@origin")]);
    let nested: String = format!("{container}/{nested_slot}");
    zfs_ok(sbin.as_str(), &["clone", &format!("{container}@origin"), nested.as_str()]);
    zfs_ok(sbin.as_str(), &["set", format!("{DESIRED_STATE}={}", mount_request(nonce)).as_str(), nested.as_str()]);
    let nested_run: Run = sweep();
    let refused_nested: &str = line_about(nested_run.said.as_str(), nested.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {nested}: {}", nested_run.said));
    assert!(refused_nested.starts_with("refused ") && refused_nested.contains("step=clone-origin"), "a clone of the container's snapshot is refused: {refused_nested}");
    assert!(!mounted(nested.as_str()), "a clone of the container's snapshot mounts nothing");
    println!("SWEEP refusal item=2 step=clone-origin");

    // A clone of another checkout dataset's snapshot is another lineage.
    let foreign_checkout: String = format!("{pool}/step8-foreign/{CHECKOUT_LEAF}");
    zfs_ignored(sbin.as_str(), &["destroy", "-r", "-f", format!("{pool}/step8-foreign").as_str()]);
    // The foreign checkout is a fixture and never mounted: a mounted dataset of the checkout's leaf name would be a
    // second instance for the arms after this one.
    zfs_ok(sbin.as_str(), &["create", "-p", "-o", "mountpoint=none", foreign_checkout.as_str()]);
    zfs_ignored(sbin.as_str(), &["unmount", foreign_checkout.as_str()]);
    let foreign_slot: &str = "four";
    make_the_slot(checkout.as_path(), foreign_slot);
    zfs_ok(sbin.as_str(), &["snapshot", &format!("{foreign_checkout}@{ORIGIN_PREFIX}{nonce}")]);
    let foreign: String = format!("{container}/{foreign_slot}");
    zfs_ok(sbin.as_str(), &["clone", &format!("{foreign_checkout}@{ORIGIN_PREFIX}{nonce}"), foreign.as_str()]);
    zfs_ok(sbin.as_str(), &["set", format!("{DESIRED_STATE}={}", mount_request(nonce)).as_str(), foreign.as_str()]);
    let foreign_run: Run = sweep();
    let refused_foreign: &str = line_about(foreign_run.said.as_str(), foreign.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {foreign}: {}", foreign_run.said));
    assert!(refused_foreign.starts_with("refused ") && refused_foreign.contains("step=clone-origin"), "a clone of another checkout dataset is refused: {refused_foreign}");
    println!("SWEEP refusal item=2 step=clone-origin-foreign");

    // The pass's stoppage: root's file names another container and leaves out the checkout entries, the account makes its
    // clone inside that container and asks for its mount, and the route mounts it. The next sweep and the probe pass,
    // because a row inside the container this run uses is no instance of the checkout
    // (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`, "Four further answers on the look-up with no configuration
    // file", the first answer).
    // The probe needs its own dataset and its empty `ready` snapshot, which plan step 19's setup creates.
    zfs_ignored(sbin.as_str(), &["destroy", "-r", "-f", format!("{pool}/nosnap").as_str()]);
    zfs_ok(sbin.as_str(), &["create", "-p", "-o", "mountpoint=legacy", "-o", "canmount=off", format!("{pool}/nosnap").as_str()]);
    zfs_ok(sbin.as_str(), &["create", "-o", "mountpoint=legacy", "-o", "canmount=off", "-o", "com.sun:auto-snapshot=false", format!("{pool}/nosnap/git-worktree-probe").as_str()]);
    zfs_ok(sbin.as_str(), &["snapshot", format!("{pool}/nosnap/git-worktree-probe@ready").as_str()]);
    let named_container: String = format!("{pool}/account-clones");
    zfs_ignored(sbin.as_str(), &["destroy", "-r", "-f", named_container.as_str()]);
    zfs_ok(sbin.as_str(), &["create", "-o", "mountpoint=legacy", "-o", "canmount=off", "-o", "setuid=off", "-o", "devices=off", named_container.as_str()]);
    // The design's own grants, as "Scoped delegation" names them: snapshot local to the checkout, clone/destroy/mount on
    // its descendants, create and mount on the container, destroy and userprop on its descendants.
    zfs_ok(sbin.as_str(), &["allow", "-l", "-u", "ubuntu", "snapshot", checkout_dataset.as_str()]);
    zfs_ok(sbin.as_str(), &["allow", "-d", "-u", "ubuntu", "clone,destroy,mount", checkout_dataset.as_str()]);
    zfs_ok(sbin.as_str(), &["allow", "-l", "-u", "ubuntu", "create,mount", named_container.as_str()]);
    zfs_ok(sbin.as_str(), &["allow", "-d", "-u", "ubuntu", "destroy,userprop", named_container.as_str()]);
    let as_account = |arguments: &[&str]| -> (bool, String, String) {
        let mut prefixed: Vec<String> = vec![String::from("-u"), String::from("ubuntu"), String::from("--"), format!("{sbin}/zfs")];
        prefixed.extend(arguments.iter().map(|one| return String::from(*one)));
        let borrowed: Vec<&str> = prefixed.iter().map(|one| return one.as_str()).collect();
        return zfs(sbin.as_str(), "runuser", borrowed.as_slice(), None);
    };
    let in_use_nonce: &str = "00112233445566778899aabbccddee04";
    let account_snapshot: String = format!("{checkout_dataset}@{ORIGIN_PREFIX}{in_use_nonce}");
    let snapshot_allowed: (bool, String, String) = as_account(&["snapshot", account_snapshot.as_str()]);
    assert!(snapshot_allowed.0, "the account may snapshot the checkout under the route's own origin prefix: {}", snapshot_allowed.2);
    let account_clone: String = format!("{named_container}/{CHECKOUT_LEAF}");
    let clone_allowed: (bool, String, String) = as_account(&["clone", account_snapshot.as_str(), account_clone.as_str()]);
    assert!(clone_allowed.0, "the account may clone into the container root's file names: {}", clone_allowed.2);
    let intent_written: (bool, String, String) = as_account(&["set", format!("{DESIRED_STATE}={}", mount_request(in_use_nonce)).as_str(), account_clone.as_str()]);
    assert!(intent_written.0, "the account may write the mount request on its own clone: {}", intent_written.2);
    make_the_slot(checkout.as_path(), CHECKOUT_LEAF);
    write_document(&format!(r#"{{ "cloneMounts": {{ "parent": "{named_container}" }} }}"#));
    let attached_account: Run = sweep();
    assert_eq!(attached_account.status, Some(0), "the sweep reaches its end with the account's clone in the named container: {}", attached_account.said);
    let account_line: &str = line_about(attached_account.said.as_str(), account_clone.as_str())
        .unwrap_or_else(|| panic!("the sweep printed no line about {account_clone}: {}", attached_account.said));
    assert!(account_line.starts_with("attached "), "the route mounts the clone the account asked for: {account_line}");
    println!("STOPPAGE account-clone attached slot={CHECKOUT_LEAF} container={named_container}");
    let after_attach = mount_row(checkout.join(".claude").join("worktrees").join(CHECKOUT_LEAF).as_path());
    assert!(after_attach.is_some(), "the account's clone stands at its slot: {}", attached_account.said);

    // The probe, with that row standing: it reads the same container key, so the clone is no second instance and the
    // record passes. This is the pass's B1 and B2 stopped.
    let probe_with_the_rule: Run = start_probe();
    let record: String = probe_record();
    assert_eq!(record_line(record.as_str(), "verdict"), Some("passed"), "the probe passes with the account's clone mounted: {}\n{}", record, probe_with_the_rule.said);
    assert_eq!(probe_with_the_rule.status, Some(0), "a probe that records a verdict exits successfully: {}", probe_with_the_rule.said);
    println!("STOPPAGE probe passed container={named_container}");

    // Without the key, the same table is several for the probe, and it prints every row it found with its path (the
    // project's second answer of 2026-11-22).
    remove_configuration();
    let probe_without_the_key: Run = start_probe();
    let several_record: String = probe_record();
    assert_eq!(record_line(several_record.as_str(), "verdict"), Some("blocked"), "the probe blocks on several without the key: {several_record}");
    // The word sits on the checkout-root step's line; the record's own source-refusal field names the source step, which
    // fails for want of a checkout to take the pool from.
    assert!(
        several_record.lines().any(|line| return line.starts_with("step checkout-root not-exercised") && line.contains("reason several-mounted-checkouts")),
        "the probe's checkout-root step names several: {several_record}"
    );
    for named in [checkout_dataset.as_str(), account_clone.as_str()] {
        assert!(probe_without_the_key.said.contains(named), "the printed listing names {named} with its path: {}", probe_without_the_key.said);
    }
    println!("PROBE listing several names-every-row=true container={named_container}");

    // With nothing mounted under the leaf name, the probe prints the none listing.
    unmount_at(checkout.join(".claude").join("worktrees").join(CHECKOUT_LEAF).as_path());
    zfs_ok(sbin.as_str(), &["unmount", checkout_dataset.as_str()]);
    let probe_with_nothing: Run = start_probe();
    let none_record: String = probe_record();
    assert_eq!(record_line(none_record.as_str(), "verdict"), Some("blocked"), "the probe blocks when nothing carries the leaf name: {none_record}");
    assert!(
        none_record.lines().any(|line| return line.starts_with("step checkout-root not-exercised") && line.contains("reason no-mounted-checkout")),
        "the probe's checkout-root step names none: {none_record}"
    );
    assert!(
        probe_with_nothing.said.contains("no row's source carried it") || probe_with_nothing.said.contains("is an instance of the checkout dataset"),
        "the printed listing says what the table held: {}",
        probe_with_nothing.said
    );
    println!("PROBE listing none names-what-was-found=true");
    let remounted: (bool, String, String) = zfs(sbin.as_str(), "zfs", &["mount", checkout_dataset.as_str()], None);
    assert!(remounted.0, "the checkout dataset is mounted again for the arms after this one: {}", remounted.2);
    command_ok("mount", &["--make-private", checkout.display().to_string().as_str()]);

    // The account's clone, its snapshot and the container go away, so no later arm meets a second leaf row.
    unmount_if_it_stands(checkout.join(".claude").join("worktrees").join(CHECKOUT_LEAF).as_path());
    zfs_ok(sbin.as_str(), &["destroy", "-r", "-f", named_container.as_str()]);
    let _snapshot_gone: (bool, String, String) = zfs(sbin.as_str(), "zfs", &["destroy", account_snapshot.as_str()], None);
    let _in_use_record_cleared: std::io::Result<()> = std::fs::remove_file(Path::new(STORE).join(CHECKOUT_LEAF));
    let _in_use_slot_cleared: std::io::Result<()> = std::fs::remove_dir_all(checkout.join(".claude").join("worktrees").join(CHECKOUT_LEAF));
    zfs_ignored(sbin.as_str(), &["destroy", "-r", "-f", format!("{pool}/nosnap").as_str()]);
    // This arm's probes wrote their own verdicts, the last of them `blocked`; the arms after this one need the passing
    // record the earlier arms handed in.
    write_passing_probe_record();
    println!("STOPPAGE cleanup done container={named_container}");

    // The descriptor unmount a rollback makes: the mount a held descriptor placed stands at the moved path while a
    // holder remains, the unmount answers EBUSY and takes nothing down, and the same call succeeds after the holder
    // leaves.
    let held_slot: &str = "five";
    make_the_slot(checkout.as_path(), held_slot);
    let rollback_clone: String = clone_the_checkouts_snapshot(sbin.as_str(), checkout_dataset.as_str(), container.as_str(), held_slot, "00112233445566778899aabbccddee02", None);
    let store_handle: SlotIdentityStore = SlotIdentityStore::open_under_root_control(Path::new(STORE)).expect("the store opens");
    let _ = store_handle;
    let walk_request: HoldCloneSlotRequest<'_> = HoldCloneSlotRequest { checkout: checkout.as_path(), slot: held_slot, store: Path::new(STORE) };
    let held = observe_the_clone_slot_under_root_control(&walk_request).expect("the fixture slot is held by descriptor");
    let worktrees_parent: PathBuf = checkout.join(".claude");
    let moved_parent: PathBuf = checkout.join(".claude-moved-by-probe");
    std::fs::rename(worktrees_parent.as_path(), moved_parent.as_path()).expect("the walk's parent is renamed under the held descriptors");
    let attached_mount = attach_the_clone(rollback_clone.as_str(), &held.target).expect("the clone attaches through the held descriptor");
    let moved_slot: PathBuf = moved_parent.join("worktrees").join(held_slot);
    let canonical_slot: PathBuf = worktrees_parent.join("worktrees").join(held_slot);
    assert!(mount_row(moved_slot.as_path()).is_some(), "the mount landed at the moved path the descriptor names");
    assert!(mount_row(canonical_slot.as_path()).is_none(), "no mount stands at the canonical slot");
    // The shell replaces itself with the sleeper, so killing the holder kills the process whose working directory is
    // the mount; a shell that kept a sleeping child would leave that child holding the mount busy.
    let mut holder = Command::new("sh").arg("-c").arg(format!("cd {} && exec sleep 60", moved_slot.display())).spawn().expect("the holder starts");
    std::thread::sleep(Duration::from_millis(250));
    let busy = unmount_through_descriptor(&held.target).expect_err("a mount a holder stands in answers EBUSY");
    assert_eq!(busy.errno, libc::EBUSY, "the unmount answered the kernel's EBUSY: {busy:?}");
    assert!(mount_row(moved_slot.as_path()).is_some(), "the busy unmount took nothing down");
    println!("SWEEP rollback unmount=EBUSY mount={} standing=true", attached_mount.mount_id);
    let _ = holder.kill();
    let _ = holder.wait();
    std::thread::sleep(Duration::from_millis(250));
    unmount_through_descriptor(&held.target).expect("the same call succeeds after the holder left");
    assert!(mount_row(moved_slot.as_path()).is_none(), "the mount left the kernel");
    println!("SWEEP rollback unmount=ok mount={} standing=false", attached_mount.mount_id);
    std::fs::rename(moved_parent.as_path(), worktrees_parent.as_path()).expect("the parent folder is restored");

    // The child bound the listing takes: one sweep over more children than it visits stops at the bound and says so,
    // and the time it took is the number beside the constant.
    let bound_parent: String = format!("{pool}/step8-bound/clones");
    zfs_ignored(sbin.as_str(), &["destroy", "-r", "-f", format!("{pool}/step8-bound").as_str()]);
    zfs_ok(sbin.as_str(), &["create", "-p", "-o", "mountpoint=legacy", "-o", "canmount=off", bound_parent.as_str()]);
    let children: usize = git_policy_cli::worktree_clone_mount_handler::sweep_dataset_reader::MAXIMUM_CHILDREN + 1;
    let children_created: (bool, String, String) = zfs(
        sbin.as_str(),
        "sh",
        &["-c", format!("i=0; while [ $i -lt {children} ]; do zfs create {bound_parent}/c$i || exit 1; i=$((i+1)); done").as_str()],
        None,
    );
    assert!(children_created.0, "the bound's children are created: {}", children_created.2);
    write_configuration(checkout.as_path(), Some(bound_parent.as_str()), Some(checkout_dataset.as_str()));
    let bounded: Run = sweep();
    assert_eq!(bounded.status, Some(0), "a listing that stopped is still a run that reached its end: {}", bounded.said);
    assert!(bounded.said.contains("The listing stopped:"), "the run says the listing stopped: {}", bounded.said);
    assert!(bounded.said.contains(&format!("more than the {} children", git_policy_cli::worktree_clone_mount_handler::sweep_dataset_reader::MAXIMUM_CHILDREN)), "the stop names the bound: {}", bounded.said);
    println!("MEASUREMENT child-bound-listing-ms={} children={} bound={}", bounded.elapsed.as_millis(), children, git_policy_cli::worktree_clone_mount_handler::sweep_dataset_reader::MAXIMUM_CHILDREN);
    println!("SWEEP bound stopped=true children={children}");
    // What the route's own dataset-stats request says about `snapdir`, which the documents once said it could not
    // report (the second pass's finding B1). Three states: a value set locally, a value inherited from the parent, and
    // the default nobody set.
    let snapdir_parent: String = format!("{pool}/step8-snapdir");
    zfs_ignored(sbin.as_str(), &["destroy", "-r", "-f", snapdir_parent.as_str()]);
    zfs_ok(sbin.as_str(), &["create", "-p", "-o", "mountpoint=legacy", "-o", "canmount=off", "-o", "snapdir=disabled", snapdir_parent.as_str()]);
    zfs_ok(sbin.as_str(), &["create", "-o", "mountpoint=legacy", "-o", "canmount=off", format!("{snapdir_parent}/local-visible").as_str()]);
    zfs_ok(sbin.as_str(), &["set", "snapdir=visible", format!("{snapdir_parent}/local-visible").as_str()]);
    zfs_ok(sbin.as_str(), &["create", "-o", "mountpoint=legacy", "-o", "canmount=off", format!("{snapdir_parent}/inherited-disabled").as_str()]);
    zfs_ok(sbin.as_str(), &["create", "-p", "-o", "mountpoint=legacy", "-o", "canmount=off", format!("{pool}/step8-snapdir-default").as_str()]);
    let reader: ZfsDatasetReader = ZfsDatasetReader::open().expect("the guest permits the device");
    let read = |dataset: &str| -> PropertyValue {
        let identity = reader.identity(dataset).unwrap_or_else(|failure| panic!("the reader answers for {dataset}: {}", failure.said));
        return value_of(&identity.properties, "snapdir");
    };
    let locally_set: PropertyValue = read(format!("{snapdir_parent}/local-visible").as_str());
    let inherited: PropertyValue = read(format!("{snapdir_parent}/inherited-disabled").as_str());
    let default: PropertyValue = read(format!("{pool}/step8-snapdir-default").as_str());
    assert_eq!(locally_set, PropertyValue::Number(1), "a locally set snapdir is carried as its number: {locally_set:?}");
    assert_eq!(inherited, PropertyValue::Number(2), "an inherited snapdir is carried as its number: {inherited:?}");
    assert_eq!(default, PropertyValue::Absent, "the default is the one state the reply leaves out: {default:?}");
    println!("SNAPDIR-REPLY local-visible=1 inherited-disabled=2 default=absent");
    println!("SWEEP done pool={pool} layout=design slot={SLOT}");
}
