//! What:
//!  Plan step 9's arms in the throwaway guest,
//! on the design's dataset layout:
//!  a clone attached by the built sweep and unmounted by a handler process that
//!  did not attach it,
//!  a holder that makes the unmount busy and the checked reattach,
//!  two removals at once,
//!  and the receipt the plain unmount files.
//! Why:
//!  Every claim about a ZFS clone's unmount rests on the guest until this runs,
//!  and the design's M1 and M2 arms are the same scenes at the descriptor level.
//!  The arm runs only where `CLI_GIT_ZFS_UNMOUNT_GUEST` names a disposable guest
//!  and prints `SKIP` everywhere else,
//! so a host run that ran nothing says so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await sweep(); // attached
//! await unmount(); // a fresh handler process unmounts and files done
//! ```

use super::zfs_sweep_guest::{
    CONFIGURATION, DESIRED_STATE, NONCE, PROBE_RECORD_DIRECTORY, Run, STORE, clone_the_checkouts_snapshot, command_ok, line_about, make_the_designs_layout, make_the_slot,
    mount_request, prepare_the_store, required, sweep, write_configuration, write_passing_probe_record, zfs, zfs_ignored, zfs_ok,
};
use git_policy_cli::worktree_clone_mount_protocol::DatasetGuid;
use git_policy_cli::worktree_clone_unmount::staged_folder_identity::StagedFolderIdentity;
use git_policy_cli::worktree_clone_unmount::staged_folder_record::{write_record, StagedFolderRecord};
use git_policy_cli::worktree_clone_unmount::staged_record_reading::{read_staged_record, RecordReading};
use git_policy_cli::worktree_clone_unmount::staging_calls::{LiveStagingCalls, StagingCalls};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

/// The variable the guest driver sets for these arms.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_UNMOUNT_GUEST";

/// The pool variable the driver prepared.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_POOL";

/// The OpenZFS userspace variable the driver prepared.
const SBIN_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_SBIN";

/// The checkout variable the driver prepared.
const CHECKOUT_VARIABLE: &str = "CLI_GIT_ZFS_SWEEP_CHECKOUT";

/// The account's home.
const ACCOUNT_HOME: &str = "/home/ubuntu";

/// The pin's root-owned file.
const PIN: &str = "/var/lib/monochromatic-git-policy-cli/user-cache-pin";

/// The receipts folder.
const RECEIPTS: &str = "/run/git-worktree-clone-mount-results";

/// Whether this run is inside the guest.
fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP worktree_clone_unmount::guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest");
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname").map(|text| return text.trim().to_string()).unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// One clone's GUID.
fn guid_of(sbin: &str, clone: &str) -> String {
    let (ok, said, _error) = zfs(sbin, "zfs", &["get", "-H", "-o", "value", "guid", clone], None);
    assert!(ok, "the GUID of {clone} is read: {said}");
    return said.trim().to_string();
}

/// The staging folder's name for one GUID.
fn staging_name(guid: &str) -> String {
    return format!("monochromatic-git-policy-cli-staging-{guid}");
}

/// Make the account's `.cache` a dataset and record the pin the handler checks.
fn make_the_cache_and_pin(sbin: &str, pool: &str) -> String {
    let cache_dataset: String = format!("{pool}/cache");
    zfs_ignored(sbin, &["destroy", "-r", "-f", cache_dataset.as_str()]);
    std::fs::create_dir_all(ACCOUNT_HOME).expect("the account's home is made");
    zfs_ok(sbin, &["create", "-p", "-o", "mountpoint=/home/ubuntu/.cache", cache_dataset.as_str()]);
    let facts: std::fs::Metadata = std::fs::metadata("/home/ubuntu/.cache").expect("the cache's metadata reads");
    let pin: String = format!(
        "{{\"version\":\"v1\",\"kind\":\"user-cache-pin\",\"source\":\"{cache_dataset}\",\"root-inode\":\"{}\",\"device\":\"{}\"}}\n",
        facts.ino(),
        facts.dev()
    );
    let directory: &Path = Path::new(PIN).parent().expect("the pin has a folder");
    std::fs::create_dir_all(directory).expect("the pin's folder is made");
    std::fs::write(PIN, pin).expect("the pin is written");
    std::fs::set_permissions(PIN, std::fs::Permissions::from_mode(0o644)).expect("the pin holds its mode");
    return cache_dataset;
}

/// Make one clone's staging folder and return its path.
fn make_the_staging_folder(cache: &str, guid: &str) -> PathBuf {
    let folder: PathBuf = Path::new(cache).join(staging_name(guid));
    let _ = std::fs::remove_dir_all(folder.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the staging folder is made");
    std::fs::set_permissions(folder.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the staging folder holds its mode");
    return folder;
}

/// Write the unmount intent on one clone.
fn ask_to_unmount(sbin: &str, clone: &str, nonce: &str) {
    zfs_ok(sbin, &["set", format!("{DESIRED_STATE}=v1:{nonce}:unmount").as_str(), clone]);
}

/// One receipt's text,
/// or nothing when the handler filed none.
fn receipt(guid: &str) -> Option<String> {
    return std::fs::read_to_string(Path::new(RECEIPTS).join(guid)).ok();
}

/// A source row at the slot,
/// as the guest's mount table prints it.
fn row_at(folder: &Path) -> Option<String> {
    let text: String = std::fs::read_to_string("/proc/self/mountinfo").expect("the mount table reads");
    return text.lines().find(|line| return line.split(' ').nth(4) == Some(folder.to_string_lossy().as_ref())).map(String::from);
}

/// Whether any line begins with `prefix` and names `dataset`,
/// which a concurrent handler's later line about the same clone does not hide.
fn any_line(said: &str, prefix: &str, dataset: &str) -> bool {
    return said.lines().any(|line| return line.starts_with(prefix) && line.contains(dataset));
}

/// Start a holder whose working directory is inside `folder`,
/// and wait until it is there.
fn hold_inside(folder: &Path) -> Child {
    let script: String = format!("cd {} && exec sleep 60", folder.display());
    let mut holder: Child = Command::new("sh").args(["-c", script.as_str()]).spawn().expect("the holder starts");
    let wanted: PathBuf = std::fs::canonicalize(folder).expect("the folder resolves");
    let deadline: Instant = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let link: PathBuf = PathBuf::from(format!("/proc/{}/cwd", holder.id()));
        if std::fs::read_link(link).ok() == Some(wanted.clone()) {
            return holder;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let _ = holder.kill();
    let _ = holder.wait();
    panic!("the holder never reached {}", folder.display());
}

/// What:
///  The whole step 9 guest run:
///  the design's layout,
///  one plain unmount by a fresh handler,
///  one busy unmount that reattaches,
///  and two removals at once.
/// Why:
///  Each arm owns its clones and slots for the whole run,
///  and the driver's markers say which arm ran.
#[test]
fn zfs_unmount_guest() {
    if !guest() {
        return;
    }
    let sbin: String = required(SBIN_VARIABLE);
    let pool: String = required(POOL_VARIABLE);
    let checkout: PathBuf = PathBuf::from(required(CHECKOUT_VARIABLE));
    let _ = std::fs::remove_dir_all(checkout.as_path());
    std::fs::create_dir_all(checkout.as_path()).expect("the checkout folder is made");
    let (checkout_dataset, container): (String, String) = make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
    // The handler finds the account's home from the checkout's owner, so the fixture checkout is the account's, as the design's repository is.
    command_ok("chown", &["--recursive", "ubuntu:ubuntu", checkout.display().to_string().as_str()]);
    write_configuration(checkout.as_path(), Some(container.as_str()), Some(checkout_dataset.as_str()));
    write_passing_probe_record();
    prepare_the_store(checkout.as_path());
    std::fs::create_dir_all(checkout.join(".git/worktrees").as_path()).expect("the Git administration folder is made");
    let cache: String = make_the_cache_and_pin(sbin.as_str(), pool.as_str());

    // M1's positive arm and M2's covered-folder arm: one clone, attached by the sweep, then unmounted by a fresh handler.
    let one: String = clone_the_checkouts_snapshot(sbin.as_str(), checkout_dataset.as_str(), container.as_str(), "one", "nine-one", Some(mount_request(NONCE).as_str()));
    make_the_slot(checkout.as_path(), "one");
    let attached: Run = sweep();
    assert!(line_about(attached.said.as_str(), one.as_str()).is_some_and(|line| return line.starts_with("attached ")), "the built sweep attaches the design's clone: {}", attached.said);
    println!("UNMOUNT attached clone={one}");
    let one_guid: String = guid_of(sbin.as_str(), one.as_str());
    let one_folder: PathBuf = make_the_staging_folder("/home/ubuntu/.cache", one_guid.as_str());
    ask_to_unmount(sbin.as_str(), one.as_str(), "11111111111111111111111111111111");
    let unmounted: Run = sweep();
    assert!(line_about(unmounted.said.as_str(), one.as_str()).is_some_and(|line| return line.starts_with("unmounted ")), "a fresh handler process unmounts the clone: {}", unmounted.said);
    let done: String = receipt(one_guid.as_str()).expect("the unmount files its done receipt");
    assert!(done.contains("outcome done"), "the receipt is done: {done}");
    assert!(done.contains("operation unmount"), "the receipt answers an unmount: {done}");
    assert!(row_at(checkout.join(".claude/worktrees/one").as_path()).is_none(), "no row is left at the slot");
    assert!(std::fs::read_dir(one_folder.as_path()).expect("the staging folder lists").next().is_none(), "the staging folder is empty");
    zfs_ok(sbin.as_str(), &["destroy", one.as_str()]);
    println!("UNMOUNT plain=unmounted receipt=done destroy=ok");

    // M1's busy arm and the checked reattach: a holder in the clone keeps the unmount busy, and the handler moves the same mount back to its slot.
    let two: String = clone_the_checkouts_snapshot(sbin.as_str(), checkout_dataset.as_str(), container.as_str(), "two", "nine-two", Some(mount_request(NONCE).as_str()));
    make_the_slot(checkout.as_path(), "two");
    let two_attached: Run = sweep();
    assert!(line_about(two_attached.said.as_str(), two.as_str()).is_some_and(|line| return line.starts_with("attached ")), "the second clone attaches: {}", two_attached.said);
    let two_guid: String = guid_of(sbin.as_str(), two.as_str());
    make_the_staging_folder("/home/ubuntu/.cache", two_guid.as_str());
    let two_slot: PathBuf = checkout.join(".claude/worktrees/two");
    let mut holder: Child = hold_inside(two_slot.as_path());
    ask_to_unmount(sbin.as_str(), two.as_str(), "22222222222222222222222222222222");
    let busy: Run = sweep();
    assert!(line_about(busy.said.as_str(), two.as_str()).is_some_and(|line| return line.starts_with("reattached ")), "the busy unmount moves the same mount back: {}", busy.said);
    assert!(row_at(two_slot.as_path()).is_some(), "the clone's row is back at its slot");
    let cancelled: String = receipt(two_guid.as_str()).expect("a busy reattach files its cancelled answer");
    assert!(cancelled.contains("operation unmount"), "the answer answers the unmount: {cancelled}");
    assert!(cancelled.contains("outcome cancelled"), "the answer is cancelled: {cancelled}");
    assert!(cancelled.contains("failed-step bind-gone"), "the answer names the bind gone: {cancelled}");
    let _ = holder.kill();
    let _ = holder.wait();
    println!("UNMOUNT busy=reattached holder=ended");
    ask_to_unmount(sbin.as_str(), two.as_str(), "33333333333333333333333333333333");
    let after: Run = sweep();
    assert!(line_about(after.said.as_str(), two.as_str()).is_some_and(|line| return line.starts_with("unmounted ")), "the clone unmounts after its holder ends: {}", after.said);
    zfs_ok(sbin.as_str(), &["destroy", two.as_str()]);
    println!("UNMOUNT reattach-then-unmounted=true");

    // Two removals at once: each handler removes only its own clone.
    let three: String = clone_the_checkouts_snapshot(sbin.as_str(), checkout_dataset.as_str(), container.as_str(), "three", "nine-three", Some(mount_request(NONCE).as_str()));
    let four: String = clone_the_checkouts_snapshot(sbin.as_str(), checkout_dataset.as_str(), container.as_str(), "four", "nine-four", Some(mount_request(NONCE).as_str()));
    make_the_slot(checkout.as_path(), "three");
    make_the_slot(checkout.as_path(), "four");
    let both: Run = sweep();
    assert!(line_about(both.said.as_str(), three.as_str()).is_some_and(|line| return line.starts_with("attached ")), "the third clone attaches: {}", both.said);
    assert!(line_about(both.said.as_str(), four.as_str()).is_some_and(|line| return line.starts_with("attached ")), "the fourth clone attaches: {}", both.said);
    let three_guid: String = guid_of(sbin.as_str(), three.as_str());
    let four_guid: String = guid_of(sbin.as_str(), four.as_str());
    make_the_staging_folder("/home/ubuntu/.cache", three_guid.as_str());
    make_the_staging_folder("/home/ubuntu/.cache", four_guid.as_str());
    ask_to_unmount(sbin.as_str(), three.as_str(), "44444444444444444444444444444444");
    ask_to_unmount(sbin.as_str(), four.as_str(), "55555555555555555555555555555555");
    let binary: &str = env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets");
    let first: std::thread::JoinHandle<std::process::Output> = std::thread::spawn(move || return Command::new(binary).output().expect("the first concurrent handler starts"));
    let second: std::thread::JoinHandle<std::process::Output> = std::thread::spawn(move || return Command::new(binary).output().expect("the second concurrent handler starts"));
    let first_output: std::process::Output = first.join().expect("the first handler ends");
    let second_output: std::process::Output = second.join().expect("the second handler ends");
    let said: String = format!(
        "{}{}{}{}",
        String::from_utf8_lossy(first_output.stdout.as_slice()),
        String::from_utf8_lossy(first_output.stderr.as_slice()),
        String::from_utf8_lossy(second_output.stdout.as_slice()),
        String::from_utf8_lossy(second_output.stderr.as_slice())
    );
    assert!(any_line(said.as_str(), "unmounted ", three.as_str()), "the first removal unmounts: {said}");
    assert!(any_line(said.as_str(), "unmounted ", four.as_str()), "the second removal unmounts: {said}");
    zfs_ok(sbin.as_str(), &["destroy", three.as_str()]);
    zfs_ok(sbin.as_str(), &["destroy", four.as_str()]);
    println!("UNMOUNT two-at-once=both-unmounted");

    zfs_ignored(sbin.as_str(), &["destroy", "-r", "-f", cache.as_str()]);
    let _ = std::fs::remove_file(PIN);
    let _ = std::fs::remove_file(CONFIGURATION);
    let _ = std::fs::remove_dir_all(PROBE_RECORD_DIRECTORY);
    let _ = std::fs::remove_dir_all(STORE);
    let _ = std::fs::remove_dir_all(RECEIPTS);
    println!("UNMOUNT done");
}

/// A fixture folder under `/root`,
/// cleared first.
fn folder(name: &str) -> PathBuf {
    let path: PathBuf = Path::new("/root").join(name);
    let _ = std::fs::remove_dir_all(path.as_path());
    std::fs::create_dir_all(path.as_path()).expect("the fixture folder is made");
    return path;
}

/// The record one handle arm writes before its reopen.
fn record(guid: DatasetGuid, folder_path: &Path, _cache: &std::fs::File, boot: &str) -> StagedFolderRecord {
    return record_of(guid, folder_path, "tmpfs", boot);
}

/// The record one handle or kill-point arm writes, for a named dataset.
fn record_of(guid: DatasetGuid, folder_path: &Path, dataset: &str, boot: &str) -> StagedFolderRecord {
    let held: std::fs::File = std::fs::File::open(folder_path).expect("the staging folder opens");
    let identity: StagedFolderIdentity = StagedFolderIdentity::of(&held).expect("the staging folder's identity is read");
    let (handle_type, handle): (i32, Vec<u8>) = LiveStagingCalls.name_handle(&held).expect("the folder's handle is read");
    let (namespace_device, namespace_inode): (u64, u64) = LiveStagingCalls.namespace_pair().expect("the namespace pair is read");
    return StagedFolderRecord {
        component: staging_name(guid.file_name().as_str()),
        dataset: String::from(dataset),
        guid,
        identity,
        handle_type,
        handle,
        nonce: String::from("00112233445566778899aabbccddeeff"),
        namespace_device,
        namespace_inode,
        boot: String::from(boot),
    };
}

/// What:
///  The handle one process writes is reopened from the record by another process.
/// Why:
///  The kernel's `name_to_handle_at` and `open_by_handle_at` are one pair,
///  and only a fresh process can show the record alone is enough to reopen the
///  folder;
///  a host control's recording calls cannot see the kernel's own handle.
#[test]
#[allow(clippy::disallowed_methods, reason = "starts this test binary, not the wrapper, for the fresh-process reopen control")]
fn zfs_unmount_guest_handle_round_trip() {
    if !guest() {
        return;
    }
    let cache_path: PathBuf = folder("unmount-handle");
    let cache: std::fs::File = std::fs::File::open(cache_path.as_path()).expect("the handle cache opens");
    let guid: DatasetGuid = DatasetGuid::parse("9002").expect("the handle GUID parses");
    let folder_path: PathBuf = cache_path.join(staging_name("9002"));
    std::fs::create_dir_all(folder_path.as_path()).expect("the handle folder is made");
    std::fs::set_permissions(folder_path.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the handle folder holds 0700");
    let boot: String = LiveStagingCalls.boot().expect("this boot is read");
    let written: StagedFolderRecord = record(guid, folder_path.as_path(), &cache, boot.as_str());
    write_record(&cache, &written).expect("the handle record is written");
    let exe = std::env::current_exe().expect("this test binary");
    let child = Command::new(exe)
        .args(["--exact", "zfs_unmount_guest::zfs_unmount_guest_handle_child", "--nocapture"])
        .env("STEP9_HANDLE_CACHE", cache_path.as_path())
        .env("STEP9_HANDLE_GUID", "9002")
        .env("STEP9_HANDLE_BOOT", boot.as_str())
        .output()
        .expect("the handle child runs");
    let said: String = String::from_utf8_lossy(child.stdout.as_slice()).to_string() + String::from_utf8_lossy(child.stderr.as_slice()).as_ref();
    assert!(child.status.success(), "the handle child failed: {said}");
    assert!(said.contains("HANDLE reopened=true"), "the child reopened the folder from the record: {said}");
    println!("UNMOUNT handle-reopened=true");
}

/// The child half of the handle round trip:
///  it reads only the record,
/// the cache descriptor and the boot.
#[test]
fn zfs_unmount_guest_handle_child() {
    let Ok(cache_path) = std::env::var("STEP9_HANDLE_CACHE") else {
        return;
    };
    let cache_folder: PathBuf = PathBuf::from(cache_path);
    let guid: DatasetGuid = DatasetGuid::parse(std::env::var("STEP9_HANDLE_GUID").expect("the child GUID").as_str()).expect("the child GUID parses");
    let boot: String = std::env::var("STEP9_HANDLE_BOOT").expect("the child boot");
    let cache: std::fs::File = std::fs::File::open(cache_folder.as_path()).expect("the child cache opens");
    let calls = LiveStagingCalls;
    let reading = read_staged_record(&cache, guid, boot.as_str(), cache_folder.join(staging_name(guid.file_name().as_str())).as_path(), &calls);
    assert!(matches!(reading, RecordReading::Current { .. }), "the child reads a current record: {}", reading.word());
    println!("HANDLE reopened=true");
}

/// What:
///  The four states a killed handler leaves,
/// each finished by a later handler run.
/// Why:
///  The record's whole point is that a later run finishes what an interrupted one
///  left;
///  the states are built directly,
/// because the shipped handler carries no pause
///  seam a kill could be aimed at,
///  and each state is exactly what the pass measured a kill leaving:
///  a staged clone,
/// a left bind,
/// a bind over the slot's clone,
/// and nothing
///  mounted but the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const state of ['staged','between-unmounts','bind-over-slot','none']) { build(state); expect(sweep()).toContain('unmounted'); }
/// ```
#[test]
fn zfs_unmount_guest_kill_points() {
    if !guest() {
        return;
    }
    let sbin: String = required(SBIN_VARIABLE);
    let pool: String = required(POOL_VARIABLE);
    let checkout: PathBuf = PathBuf::from(required(CHECKOUT_VARIABLE));
    let (checkout_dataset, container): (String, String) = make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
    command_ok("chown", &["--recursive", "ubuntu:ubuntu", checkout.display().to_string().as_str()]);
    write_configuration(checkout.as_path(), Some(container.as_str()), Some(checkout_dataset.as_str()));
    write_passing_probe_record();
    prepare_the_store(checkout.as_path());
    std::fs::create_dir_all(checkout.join(".git/worktrees").as_path()).expect("the Git administration folder is made");
    let _ = make_the_cache_and_pin(sbin.as_str(), pool.as_str());
    let _wall: std::path::PathBuf = std::path::PathBuf::from("/home/ubuntu/.cache");
    let cache: std::fs::File = git_policy_cli::worktree_clone_unmount::user_cache_pin::open_cache(Path::new("/home/ubuntu")).expect("the cache opens");
    let boot: String = LiveStagingCalls.boot().expect("this boot is read");
    for (label, slot) in [("staged", "five"), ("between-unmounts", "six"), ("bind-over-slot", "seven"), ("none", "eight")] {
        let clone: String = clone_the_checkouts_snapshot(sbin.as_str(), checkout_dataset.as_str(), container.as_str(), slot, format!("nine-{slot}").as_str(), Some(mount_request(NONCE).as_str()));
        make_the_slot(checkout.as_path(), slot);
        let attached: Run = sweep();
        assert!(line_about(attached.said.as_str(), clone.as_str()).is_some_and(|line| return line.starts_with("attached ")), "the {label} clone attaches: {}", attached.said);
        let guid: DatasetGuid = DatasetGuid::parse(guid_of(sbin.as_str(), clone.as_str()).as_str()).expect("the clone GUID parses");
        let folder_path: PathBuf = _wall.join(staging_name(guid.file_name().as_str()));
        let _ = std::fs::remove_dir_all(folder_path.as_path());
        std::fs::create_dir_all(folder_path.as_path()).expect("the staging folder is made");
        std::fs::set_permissions(folder_path.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the staging folder holds 0700");
        let slot_path: PathBuf = checkout.join(".claude/worktrees").join(slot);
        let written: StagedFolderRecord = record_of(guid, folder_path.as_path(), clone.as_str(), boot.as_str());
        write_record(&cache, &written).expect("the record is written");
        let folder: std::fs::File = std::fs::File::open(folder_path.as_path()).expect("the staging folder opens");
        if label != "none" {
            let bind: std::fs::File = LiveStagingCalls.bind_the_folder(&folder, &cache, std::ffi::OsStr::new(staging_name(guid.file_name().as_str()).as_str())).expect("the folder binds");
            LiveStagingCalls.make_private(&bind).expect("the bind is private");
            drop(bind);
            if label != "bind-over-slot" {
                let clone_at_slot: std::fs::File = std::fs::File::open(slot_path.as_path()).expect("the clone's slot opens");
                let bind_again: std::fs::File = LiveStagingCalls.open_by_name(&cache, std::ffi::OsStr::new(staging_name(guid.file_name().as_str()).as_str())).expect("the bind opens");
                LiveStagingCalls.move_clone(&clone_at_slot, &bind_again).expect("the clone moves onto the bind");
                drop(clone_at_slot);
                drop(bind_again);
                if label == "between-unmounts" {
                    LiveStagingCalls.unmount_top(&folder).expect("the clone's unmount works");
                }
            }
        }
        ask_to_unmount(sbin.as_str(), clone.as_str(), "77777777777777777777777777777777");
        let finished: Run = sweep();
        let line: &str = line_about(finished.said.as_str(), clone.as_str()).unwrap_or("");
        assert!(line.starts_with("unmounted "), "the later run finishes the {label} state: {}", finished.said);
        assert!(receipt(guid.file_name().as_str()).is_some_and(|text| return text.contains("outcome done")), "the later run files done for {label}");
        assert!(row_at(slot_path.as_path()).is_none(), "the {label} state leaves no row at the slot");
        zfs_ok(sbin.as_str(), &["destroy", clone.as_str()]);
        println!("UNMOUNT kill={label} finished=unmounted");
    }
    println!("UNMOUNT kill-points=done");
}
