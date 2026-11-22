//! What:
//!  Plan step 8's arms in the throwaway guest:
//!  the built handler's own sweep against a real pool,
//!  a registered slot and a clone carrying a mount request,
//!  the same fixture with the pool's key unloaded,
//!  and the handler's refusals with account-owned slots.
//! Why:
//!  The sweep mounts a real clone as root,
//!  writes the store under `/var/lib` and files receipts under `/run`,
//!  so no control on a person's machine may run it:
//!  these arms run only where `CLI_GIT_ZFS_SWEEP_GUEST` names a disposable guest
//!  (`package/git-policy/cli/e2e/mount-git-worktree-clone-datasets-guest.ts`,
//!  on OpenZFS 2.4.4 in an `mvm` Ubuntu guest),
//!  and they print `SKIP` everywhere else,
//!  which is why the arm carries no `#[ignore]`: a harness that skips an ignored
//!  test would run it nowhere,
//!  so the gate says what it did not run
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 8,
//!  "Acceptance tests" and "VM measurements").
//!  The arm drives the built executable rather than a module,
//!  because the sweep's order and its receipts are what an operator and a
//!  waiting caller read.
//!  Item 1 and item 5 are repeated through the handler,
//!  and the key arm runs both halves:
//!  the refusal while the key is unloaded,
//!  and the attach after the load,
//!  so a run that skipped the unloaded half fails instead of reporting a green
//!  suite.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await sweep(slot('one'));            // item 1
//! await unloadKey(parent); await sweep(slot('one')); // item 5, refused
//! await loadKey(parent); await sweep(slot('one'));   // item 5, attached
//! ```

use git_policy_cli::worktree_clone_mount::{CloneFolderIdentity, SlotIdentityStore, publish_parent_identity_under_root_control};
use monochromatic_zfs_request_to_openzfs_kernel_device::ZfsDevice;
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Instant;

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

/// The unencrypted parent the first arm sweeps.
const PLAIN_PARENT_NAME: &str = "clones";

/// The encrypted parent the key arm sweeps.
const ENCRYPTED_PARENT_NAME: &str = "encrypted";

/// The slot the clone attaches at.
const SLOT: &str = "one";

/// The passphrase when the driver names none,
/// which the guest's throwaway pool accepts and no other machine ever sees.
const DEFAULT_PASSPHRASE: &str = "zfs-step-8-sweep";

/// The handler configuration the sweep reads inside the guest.
const CONFIGURATION: &str = "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc";

/// The start probe's record directory inside the guest.
const PROBE_RECORD_DIRECTORY: &str = "/run/git-worktree-clone-mount-probe";

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
    elapsed: std::time::Duration,
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

/// The line the sweep printed about one dataset,
/// or nothing when it printed none.
fn line_about<'a>(said: &'a str, dataset: &str) -> Option<&'a str> {
    return said.lines().find(|line| return line.contains(dataset) && (line.starts_with("attached ") || line.starts_with("refused ") || line.starts_with("nothing ") || line.starts_with("failed ") || line.starts_with("held ")));
}

/// Write the handler configuration the sweep reads.
fn write_configuration(parent: &str, checkout: &Path) {
    let directory: &Path = Path::new(CONFIGURATION).parent().expect("the configuration has a folder");
    std::fs::create_dir_all(directory).expect("the configuration folder is made");
    let text: String = format!(
        r#"{{
  // written by plan step 8's guest arm
  "cloneMounts": {{ "parent": "{parent}", "checkout": "{}" }}
}}
"#,
        checkout.display()
    );
    std::fs::write(CONFIGURATION, text).expect("the configuration is written");
    std::fs::set_permissions(CONFIGURATION, std::fs::Permissions::from_mode(0o644)).expect("the configuration holds its mode");
}

/// What:
///  Write the start probe's record for this boot as a passed one,
///  so the sweep's first check passes for the reason a probed boot gives.
/// Why:
///  The arm proves the sweep,
///  and step 6's arms prove the probe;
///  a record written here is the fixture the sweep reads,
///  and its `verdict passed` line is the one the mode itself writes.
fn write_passing_probe_record(variable: &str) {
    let boot: String = std::fs::read_to_string("/proc/sys/kernel/random/boot_id").expect("the boot identifier is readable").trim().to_string();
    std::fs::create_dir_all(PROBE_RECORD_DIRECTORY).expect("the probe record's folder is made");
    let text: String = format!("v1\nkind git-worktree-clone-mount-probe\nboot {boot}\nstate finished\ntime 2026-11-22T00:00:00+00:00\nverdict passed\nverdict-step none\nskip-reason none\n");
    let record: PathBuf = Path::new(PROBE_RECORD_DIRECTORY).join("result");
    std::fs::write(record.as_path(), text).expect("the probe record is written");
    std::fs::set_permissions(record.as_path(), std::fs::Permissions::from_mode(0o644)).expect("the probe record holds its mode");
    let _ = variable;
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
///  Create one clone parent with the two safety properties and a legacy
///  mountpoint,
///  and the snapshot its clones are made from.
/// Why:
///  The sweep verifies exactly these facts,
///  so the fixture is the shape the design's container leaves.
///  A parent an earlier attempt left is destroyed first,
///  so a re-run in the same guest starts from the same state.
fn reset_parent(sbin: &str, parent: &str) {
    let _ignored: (bool, String, String) = zfs(sbin, "zfs", &["destroy", "-r", "-f", parent], None);
    zfs_ok(sbin, &["create", parent]);
    zfs_ok(sbin, &["set", "mountpoint=legacy", "canmount=off", "setuid=off", "devices=off", parent]);
    zfs_ok(sbin, &["snapshot", &format!("{parent}@origin")]);
}

/// One clone of `parent`'s origin at the slot,
/// asking for its mount with `intent` when one is given.
fn clone_at_the_slot(sbin: &str, parent: &str, intent: Option<&str>) -> String {
    let clone: String = format!("{parent}/{SLOT}");
    zfs_ok(sbin, &["clone", &format!("{parent}@origin"), &clone]);
    if let Some(value) = intent {
        let assignment: String = format!("{DESIRED_STATE}={value}");
        zfs_ok(sbin, &["set", &assignment, &clone]);
    }
    return clone;
}

/// The nonce one arm's request carries,
/// which a receipt must match.
const NONCE: &str = "00112233445566778899aabbccddeeff";

/// The mount request one clone carries.
fn mount_request() -> String {
    return format!("v1:{NONCE}:mount");
}

/// What:
///  Item 1 and item 5 repeated through the built handler,
///  and the handler's own refusals:
///  one sweep mounts a real clone at its slot,
///  records the slot's first observation and files a receipt,
///  the same clone refuses while the pool's key is unloaded,
///  the same sweep attaches after the load,
///  and a malformed,
///  an inherited and a foreign intent each authorize nothing.
#[test]
fn the_built_sweep_attaches_a_clone_and_refuses_each_mismatch() {
    if !guest() {
        return;
    }
    let pool: String = required(POOL_VARIABLE);
    let sbin: String = required(SBIN_VARIABLE);
    let checkout: PathBuf = PathBuf::from(required(CHECKOUT_VARIABLE));
    let passphrase: String = std::env::var(PASSPHRASE_VARIABLE).unwrap_or_else(|_error| return String::from(DEFAULT_PASSPHRASE));
    let parent: PathBuf = checkout.join(".claude").join("worktrees");
    std::fs::create_dir_all(parent.join(SLOT)).expect("the slot folder is made");
    std::fs::set_permissions(parent.join(SLOT), std::fs::Permissions::from_mode(0o755)).expect("the slot folder holds its mode");
    write_passing_probe_record(GUEST_VARIABLE);
    prepare_the_store(checkout.as_path());
    let _device = ZfsDevice::open().expect("the guest permits the device");

    // Item 1: a clone that asks for its mount is attached by one sweep run.
    write_configuration(format!("{pool}/{PLAIN_PARENT_NAME}").as_str(), checkout.as_path());
    let plain_parent: String = format!("{pool}/{PLAIN_PARENT_NAME}");
    reset_parent(sbin.as_str(), plain_parent.as_str());
    let clone: String = clone_at_the_slot(sbin.as_str(), plain_parent.as_str(), Some(mount_request().as_str()));
    let first: Run = sweep();
    assert_eq!(first.status, Some(0), "a sweep that reached the end exits successfully: {}", first.said);
    let attached: &str = line_about(first.said.as_str(), clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {clone}: {}", first.said));
    assert!(attached.starts_with("attached "), "item 1's clone is attached: {attached}");
    println!("MEASUREMENT sweep-attach-ms={}", first.elapsed.as_millis());
    println!("SWEEP attached item=1 slot={SLOT} mount-row={}", mount_row(parent.join(SLOT).as_path()).unwrap_or_default());
    let row: String = mount_row(parent.join(SLOT).as_path()).expect("the slot is a mount point after the attach");
    assert!(row.contains(",nosuid,") || row.contains("nosuid"), "the mount carries nosuid: {row}");
    assert!(row.contains(",nodev,") || row.contains("nodev"), "the mount carries nodev: {row}");
    assert!(row.contains(&format!(" - zfs {clone} ")), "the mount's source is the clone: {row}");
    let store: SlotIdentityStore = SlotIdentityStore::open_under_root_control(Path::new(STORE)).expect("the store opens");
    let recorded: CloneFolderIdentity = store.slot(SLOT).expect("the sweep recorded the slot's first observation");
    // The record is the account's underlay folder, taken before the mount landed on it: reading the same path now
    // yields the mounted clone's own root, so the two must differ. A record equal to this would mean the sweep recorded
    // the clone's root rather than the folder it judged.
    let mounted_root: CloneFolderIdentity = CloneFolderIdentity::capture(&std::fs::File::open(parent.join(SLOT).as_path()).expect("the slot opens")).expect("the slot's identity reads");
    assert_ne!(recorded, mounted_root, "the record is the underlay the sweep observed before the attach, and the slot now shows the clone's root");
    let guid: String = zfs(sbin.as_str(), "zfs", &["get", "-H", "-o", "value", "guid", clone.as_str()], None).1.trim().to_string();
    let receipt: String = std::fs::read_to_string(format!("/run/git-worktree-clone-mount-results/{guid}")).expect("the sweep filed a receipt under the clone's GUID");
    assert!(receipt.contains("outcome done"), "the receipt says the mount was observed: {receipt}");
    assert!(receipt.contains(NONCE), "the receipt carries the request's nonce: {receipt}");
    unmount_at(parent.join(SLOT).as_path());
    assert!(!mounted(clone.as_str()), "the control's cleanup took the mount down");

    // Item 5: the same fixture with the pool's key unloaded refuses as a key
    // question, and the same sweep attaches after the load.
    let encrypted: String = format!("{pool}/{ENCRYPTED_PARENT_NAME}");
    let _ignored: (bool, String, String) = zfs(sbin.as_str(), "zfs", &["destroy", "-r", "-f", encrypted.as_str()], None);
    // A dataset created with `keylocation=prompt` asks for its passphrase as it is made, so the passphrase is fed to this
    // call as well as to the load.
    let created: (bool, String, String) = zfs(
        sbin.as_str(),
        "zfs",
        &["create", "-o", "encryption=on", "-o", "keyformat=passphrase", "-o", "keylocation=prompt", encrypted.as_str()],
        Some(format!("{passphrase}\n{passphrase}\n").as_str()),
    );
    assert!(created.0, "the encrypted fixture is created: {}", created.2);
    let loaded: (bool, String, String) = zfs(sbin.as_str(), "zfs", &["load-key", encrypted.as_str()], Some(format!("{passphrase}\n").as_str()));
    assert!(loaded.0, "the encrypted fixture's key loads: {}", loaded.2);
    zfs_ok(sbin.as_str(), &["set", "mountpoint=legacy", "canmount=off", "setuid=off", "devices=off", encrypted.as_str()]);
    zfs_ok(sbin.as_str(), &["snapshot", &format!("{encrypted}@origin")]);
    let encrypted_clone: String = clone_at_the_slot(sbin.as_str(), encrypted.as_str(), Some(mount_request().as_str()));
    write_configuration(encrypted.as_str(), checkout.as_path());
    let _encrypted_slot_record_cleared: std::io::Result<()> = std::fs::remove_file(Path::new(STORE).join(SLOT));
    prepare_the_store(checkout.as_path());
    zfs_ok(sbin.as_str(), &["unload-key", encrypted.as_str()]);
    let unloaded: Run = sweep();
    let refused: &str = line_about(unloaded.said.as_str(), encrypted_clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {encrypted_clone}: {}", unloaded.said));
    assert!(refused.starts_with("refused ") && refused.contains("step=key-unavailable"), "an unloaded key is refused as itself: {refused}");
    assert!(!mounted(encrypted_clone.as_str()), "an unloaded key mounts nothing: {}", unloaded.said);
    assert!(mount_row(parent.join(SLOT).as_path()).is_none(), "an unloaded key leaves the slot unmounted");
    println!("SWEEP refusal item=5 step=key-unavailable");
    let loaded_again: (bool, String, String) = zfs(sbin.as_str(), "zfs", &["load-key", encrypted.as_str()], Some(format!("{passphrase}\n").as_str()));
    assert!(loaded_again.0, "the key loads again: {}", loaded_again.2);
    let second: Run = sweep();
    let attached_again: &str = line_about(second.said.as_str(), encrypted_clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {encrypted_clone}: {}", second.said));
    assert!(attached_again.starts_with("attached "), "item 5's clone is attached after the key load: {attached_again}");
    println!("SWEEP attached item=5 slot={SLOT} mount-row={}", mount_row(parent.join(SLOT).as_path()).unwrap_or_default());
    unmount_at(parent.join(SLOT).as_path());
    assert!(!mounted(encrypted_clone.as_str()));

    // The handler's own refusals: a malformed intent, an inherited one and a
    // clone of another parent's snapshot each authorize nothing.
    let malformed: String = String::from("v1:nothex:mount");
    zfs_ok(sbin.as_str(), &["set", format!("{DESIRED_STATE}={malformed}").as_str(), encrypted_clone.as_str()]);
    let malformed_run: Run = sweep();
    let refused_malformed: &str = line_about(malformed_run.said.as_str(), encrypted_clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {encrypted_clone}: {}", malformed_run.said));
    assert!(refused_malformed.starts_with("refused ") && refused_malformed.contains("step=intent"), "a malformed intent is refused before any work: {refused_malformed}");
    assert!(!mounted(encrypted_clone.as_str()), "a malformed intent mounts nothing");
    println!("SWEEP refusal item=2 step=intent");
    zfs_ok(sbin.as_str(), &["inherit", DESIRED_STATE, encrypted_clone.as_str()]);
    zfs_ok(sbin.as_str(), &["set", format!("{DESIRED_STATE}={}", mount_request()).as_str(), encrypted.as_str()]);
    let inherited_run: Run = sweep();
    let nothing: &str = line_about(inherited_run.said.as_str(), encrypted_clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {encrypted_clone}: {}", inherited_run.said));
    assert!(nothing.starts_with("nothing ") && nothing.contains("inherited"), "an inherited intent authorizes nothing: {nothing}");
    assert!(!mounted(encrypted_clone.as_str()), "an inherited intent mounts nothing");
    println!("SWEEP nothing inherited=true");
    zfs_ok(sbin.as_str(), &["inherit", DESIRED_STATE, encrypted.as_str()]);
    let foreign: String = format!("{pool}/foreign");
    reset_parent(sbin.as_str(), foreign.as_str());
    let foreign_clone: String = format!("{encrypted}/{SLOT}");
    zfs_ok(sbin.as_str(), &["destroy", &foreign_clone]);
    zfs_ok(sbin.as_str(), &["clone", &format!("{foreign}@origin"), &foreign_clone]);
    zfs_ok(sbin.as_str(), &["set", format!("{DESIRED_STATE}={}", mount_request()).as_str(), foreign_clone.as_str()]);
    let _foreign_slot_record_cleared: std::io::Result<()> = std::fs::remove_file(Path::new(STORE).join(SLOT));
    let foreign_run: Run = sweep();
    let refused_foreign: &str = line_about(foreign_run.said.as_str(), foreign_clone.as_str()).unwrap_or_else(|| panic!("the sweep printed no line about {foreign_clone}: {}", foreign_run.said));
    assert!(refused_foreign.starts_with("refused ") && refused_foreign.contains("step=clone-origin"), "a clone of another parent is refused: {refused_foreign}");
    assert!(!mounted(foreign_clone.as_str()), "a clone of another parent mounts nothing");
    println!("SWEEP refusal item=2 step=clone-origin");
    println!("SWEEP done pool={pool} slot={SLOT}");
}
