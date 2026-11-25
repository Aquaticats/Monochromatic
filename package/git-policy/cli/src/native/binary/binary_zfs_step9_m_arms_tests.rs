//! What:
//!  The staging design's M3 to M9 as guest arms:
//!  the private bind and what it copies,
//!  removing the bind from every namespace,
//!  the ordering against the `user-cache` dataset,
//!  interruption and the cleanup pass,
//!  two removals at once,
//!  a root-made target,
//!  and the refused namespace teardown.
//! Why:
//!  Each measurement states its own scene and a control,
//!  and no control on a person's machine may mount:
//!  these run only where `CLI_GIT_ZFS_M_GUEST` names a disposable guest and print
//!  `SKIP` everywhere else,
//!  so the gate says what it did not run
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "Owed measurements in a throwaway VM").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await m3(); await m4(); … // each builds its scene and asserts its own expectation
//! ```


use git_policy_cli::worktree_clone_mount_protocol::DatasetGuid;
use git_policy_cli::worktree_clone_unmount::staged_folder_identity::StagedFolderIdentity;
use git_policy_cli::worktree_clone_unmount::staged_folder_record::{write_record, StagedFolderRecord};
use git_policy_cli::worktree_clone_unmount::staged_unmount::{recover_a_staged_unmount, RecoveryOutcome, StagedSweep};
use git_policy_cli::worktree_clone_unmount::staging_calls::{LiveStagingCalls, StagingCalls};
use git_policy_cli::worktree_clone_unmount::staging_cleanup::{clean_up_staging_folder, CleanupOutcome, CleanupRequest};
use git_policy_cli::worktree_clone_unmount::staging_folder_name::staging_folder_name;
use git_policy_cli::worktree_clone_unmount::user_cache_pin::{open_cache, read_pin, UserCachePin, USER_CACHE_PIN_PATH};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const GUEST: &str = "CLI_GIT_ZFS_M_GUEST";
const POOL: &str = "CLI_GIT_ZFS_SWEEP_POOL";
const SBIN: &str = "CLI_GIT_ZFS_SWEEP_SBIN";
const CHECKOUT: &str = "CLI_GIT_ZFS_SWEEP_CHECKOUT";
const HOME: &str = "/home/ubuntu";
const CACHE: &str = "/home/ubuntu/.cache";

fn required(name: &str) -> String {
    return std::env::var(name).unwrap_or_else(|_| panic!("{name} is unset"));
}

fn in_guest() -> bool {
    let Ok(name) = std::env::var(GUEST) else {
        println!("SKIP M arms: {GUEST} unset");
        return false;
    };
    let running = std::fs::read_to_string("/etc/hostname").map(|t| return t.trim().to_string()).unwrap_or_default();
    assert_eq!(running, name);
    return true;
}

fn sh(script: &str) -> (bool, String) {
    let out = Command::new("sh").args(["-c", script]).output().expect("sh starts");
    let said = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    return (out.status.success(), said.trim().replace('\n', " / "));
}

/// One shell command whose output keeps its own line breaks,
/// for a table the arm splits itself.
fn sh_raw(script: &str) -> (bool, String) {
    let out = Command::new("sh").args(["-c", script]).output().expect("sh starts");
    return (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)));
}

fn as_account(script: &str) -> (bool, String) {
    return sh(format!("runuser -u ubuntu -- sh -c '{}'", script.replace('\'', "'\\''")).as_str());
}

fn zfs(sbin: &str, args: &[&str]) -> (bool, String) {
    let out = Command::new(format!("{sbin}/zfs")).args(args).output().expect("zfs starts");
    return (out.status.success(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)).trim().to_string());
}

fn zfs_ok(sbin: &str, args: &[&str]) {
    let (ok, said) = zfs(sbin, args);
    assert!(ok, "zfs {args:?}: {said}");
}

fn table() -> Vec<String> {
    return std::fs::read_to_string("/proc/self/mountinfo").expect("the table reads").lines().map(|l| return l.to_string()).collect();
}

fn rows_at(path: &Path) -> Vec<String> {
    let want = path.display().to_string();
    return table().into_iter().filter(|line| return line.split(' ').nth(4) == Some(want.as_str())).map(|line| return desc(&line)).collect();
}

fn desc(line: &str) -> String {
    let f: Vec<&str> = line.split(' ').collect();
    let dash = f.iter().position(|x| return *x == "-").unwrap_or(f.len());
    return format!("id={} parent={} root={} at={} fs={} src={} opt={}", f[0], f[1], f[3], f[4], f.get(dash + 1).unwrap_or(&"?"), f.get(dash + 2).unwrap_or(&"?"), f[6]);
}

fn rows_of_source(clone: &str) -> Vec<String> {
    let needle = format!(" - zfs {clone} ");
    return table().into_iter().filter(|line| return line.contains(&needle)).map(|line| return desc(&line)).collect();
}

fn optional_fields(line: &str) -> String {
    let f: Vec<&str> = line.split(' ').collect();
    return f.iter().skip(6).take_while(|x| return **x != "-").cloned().collect::<Vec<&str>>().join(" ");
}

fn receipt(guid: &str) -> String {
    return match std::fs::read_to_string(Path::new("/run/git-worktree-clone-mount-results").join(guid)) {
        Ok(text) => text.lines().filter(|l| return l.starts_with("operation") || l.starts_with("outcome") || l.starts_with("nonce") || l.starts_with("failed-step")).collect::<Vec<&str>>().join(" ; "),
        Err(_) => String::from("none"),
    };
}

struct World {
    sbin: String,
    checkout: PathBuf,
    checkout_dataset: String,
    container: String,
    counter: std::cell::Cell<u32>,
}

fn world(tag: &str) -> World {
    let sbin = required(SBIN);
    let pool = required(POOL);
    let checkout = PathBuf::from(format!("{}-{tag}", required(CHECKOUT)));
    let _ = std::fs::remove_dir_all(&checkout);
    std::fs::create_dir_all(&checkout).expect("the checkout folder");
    let base = format!("{pool}/step8-{tag}");
    let checkout_dataset = format!("{base}/monochromatic-git-checkout");
    let container = format!("{base}/monochromatic-git-worktree-clone-datasets");
    let _ = sh(format!("export PATH={sbin}:$PATH; umount -R {0} 2>/dev/null; umount -R {1} 2>/dev/null; zfs destroy -r -f {2} 2>/dev/null; for d in $(zfs list -H -o name -r {2} 2>/dev/null | tac); do zfs destroy -f $d 2>/dev/null; done; true", checkout.display(), CACHE, base).as_str());
    std::fs::create_dir_all(&checkout).expect("the checkout folder again");
    let (base_there, _) = zfs(&sbin, &["list", "-H", "-o", "name", base.as_str()]);
    if !base_there {
        zfs_ok(&sbin, &["create", "-p", "-o", "mountpoint=none", base.as_str()]);
    }
    let (checkout_there, _) = zfs(&sbin, &["list", "-H", "-o", "name", checkout_dataset.as_str()]);
    if !checkout_there {
        zfs_ok(&sbin, &["create", "-o", format!("mountpoint={}", checkout.display()).as_str(), checkout_dataset.as_str()]);
    } else if std::fs::metadata(checkout.join(".git")).is_err() {
        let _ = sh(format!("zfs mount {}", checkout_dataset).as_str());
    }
    let _ = sh(format!("mount --make-private {}", checkout.display()).as_str());
    let (container_there, _) = zfs(&sbin, &["list", "-H", "-o", "name", container.as_str()]);
    if !container_there {
        zfs_ok(&sbin, &["create", "-o", "mountpoint=legacy", "-o", "canmount=off", "-o", "setuid=off", "-o", "devices=off", container.as_str()]);
    }
    std::fs::create_dir_all(checkout.join(".claude/worktrees")).expect("the chain");
    std::fs::create_dir_all(checkout.join(".git/worktrees")).expect("the git admin");
    let _ = sh(format!("chown --recursive ubuntu:ubuntu {}", checkout.display()).as_str());
    let config = format!("{{\n  \"cloneMounts\": {{ \"checkout\": \"{}\", \"parent\": \"{container}\", \"checkoutDataset\": \"{checkout_dataset}\" }}\n}}\n", checkout.display());
    std::fs::create_dir_all("/etc/monochromatic-git-policy-cli").expect("the config folder");
    std::fs::write("/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc", config).expect("the config");
    let live = git_policy_cli::start_probe::probe_fingerprint::ProbeFingerprint::read(&git_policy_cli::start_probe::FingerprintPaths::live()).expect("the fingerprint");
    std::fs::create_dir_all("/run/git-worktree-clone-mount-probe").expect("the probe folder");
    let record = format!(
        "v1\nkind probe-result\nstate finished\nboot {}\nkernel {}\nmodule {}\nsrcversion {}\nbuild-id {}\nhandler {}\ntime 2026-11-24T00:00:00+00:00\nverdict passed\nverdict-step none\nskip-reason none\n",
        live.boot_id, live.kernel_release, live.module_version.as_deref().unwrap_or("none"), live.module_srcversion.as_deref().unwrap_or("none"), live.module_build_id.as_deref().unwrap_or("none"), live.handler.as_deref().unwrap_or("none")
    );
    std::fs::write("/run/git-worktree-clone-mount-probe/result", record).expect("the probe record");
    std::fs::set_permissions("/run/git-worktree-clone-mount-probe/result", std::fs::Permissions::from_mode(0o644)).expect("mode");
    let store = "/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities";
    let _ = std::fs::remove_dir_all(store);
    std::fs::create_dir_all(store).expect("the store");
    std::fs::set_permissions(store, std::fs::Permissions::from_mode(0o755)).expect("mode");
    let parent = checkout.join(".claude/worktrees");
    let identity = git_policy_cli::worktree_clone_mount::CloneFolderIdentity::capture(&std::fs::File::open(&parent).expect("the parent")).expect("the identity");
    git_policy_cli::worktree_clone_mount::publish_parent_identity_under_root_control(Path::new(store), &identity, &identity).expect("the parent record");
    let cache_dataset = format!("{pool}/cache-{tag}");
    let _ = zfs(&sbin, &["destroy", "-r", "-f", cache_dataset.as_str()]);
    std::fs::create_dir_all(HOME).expect("the home");
    let (cache_there, _) = zfs(&sbin, &["list", "-H", "-o", "name", cache_dataset.as_str()]);
    if !cache_there {
        zfs_ok(&sbin, &["create", "-p", "-o", format!("mountpoint={CACHE}").as_str(), cache_dataset.as_str()]);
    } else if std::fs::metadata(CACHE).map(|m| return m.dev() == std::fs::metadata(HOME).map(|h| return h.dev()).unwrap_or(0)).unwrap_or(true) {
        let _ = sh(format!("zfs mount {cache_dataset}").as_str());
    }
    let _ = sh(format!("chown ubuntu:ubuntu {CACHE}").as_str());
    let facts = std::fs::metadata(CACHE).expect("the cache");
    let pin = format!("{{\"version\":\"v1\",\"kind\":\"user-cache-pin\",\"source\":\"{cache_dataset}\",\"root-inode\":\"{}\",\"device\":\"{}\"}}\n", facts.ino(), facts.dev());
    std::fs::create_dir_all(Path::new(USER_CACHE_PIN_PATH).parent().expect("the pin folder")).expect("the pin folder");
    std::fs::write(USER_CACHE_PIN_PATH, pin).expect("the pin");
    std::fs::set_permissions(USER_CACHE_PIN_PATH, std::fs::Permissions::from_mode(0o644)).expect("mode");
    let _ = cache_dataset;
    return World { sbin, checkout, checkout_dataset, container, counter: std::cell::Cell::new(0) };
}

struct Scene {
    clone: String,
    guid: String,
    slot: PathBuf,
    folder: PathBuf,
}

fn sweep() -> (Option<i32>, String) {
    let out = Command::new(crate::support_installed_name::installed_name("step9")).output().expect("the sweep starts under the installed name");
    return (out.status.code(), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)));
}

fn line_about(said: &str, clone: &str) -> String {
    return said.lines().find(|l| return l.contains(clone) && (l.starts_with("unmounted ") || l.starts_with("reattached ") || l.starts_with("attached ") || l.starts_with("refused ") || l.starts_with("failed "))).unwrap_or("").to_string();
}

fn scene(world: &World, label: &str) -> Scene {
    world.counter.set(world.counter.get() + 1);
    let name = format!("{label}{}{}", world.counter.get(), std::process::id());
    let snapshot = format!("{}@git-worktree-clone-origin-{name}-{}", world.checkout_dataset, std::process::id());
    let _ = zfs(&world.sbin, &["destroy", "-r", "-f", format!("{snapshot}").as_str()]);
    zfs_ok(&world.sbin, &["snapshot", snapshot.as_str()]);
    let clone = format!("{}/{name}", world.container);
    zfs_ok(&world.sbin, &["clone", snapshot.as_str(), clone.as_str()]);
    zfs_ok(&world.sbin, &["set", "org.monochromatic:git-worktree-clone-mount-desired-state=v1:00112233445566778899aabbccddeeff:mount", clone.as_str()]);
    let slot = world.checkout.join(".claude/worktrees").join(&name);
    std::fs::create_dir_all(&slot).expect("the slot");
    std::fs::write(slot.join(".git"), format!("gitdir: {}\n", world.checkout.join(".git/worktrees").join(&name).display())).expect("the pointer");
    let _ = sh(format!("chown --recursive ubuntu:ubuntu {}", world.checkout.display()).as_str());
    let (_, said) = sweep();
    assert!(line_about(&said, &clone).starts_with("attached "), "the clone attaches: {said}");
    let (_, guid_text) = zfs(&world.sbin, &["get", "-H", "-o", "value", "guid", clone.as_str()]);
    let guid = guid_text.trim().to_string();
    let folder = Path::new(CACHE).join(staging_folder_name(DatasetGuid::parse(guid.as_str()).expect("guid")));
    return Scene { clone, guid, slot, folder };
}

fn make_folder(scene: &Scene) {
    let (ok, said) = as_account(format!("mkdir -p -m 700 {}", scene.folder.display()).as_str());
    assert!(ok, "the account makes the folder: {said}");
}

fn ask(world: &World, scene: &Scene, nonce: &str) {
    zfs_ok(&world.sbin, &["set", format!("org.monochromatic:git-worktree-clone-mount-desired-state=v1:{nonce}:unmount").as_str(), scene.clone.as_str()]);
}

fn cache() -> std::fs::File {
    return open_cache(Path::new(HOME)).expect("the cache opens");
}

fn pin() -> UserCachePin {
    return read_pin(Path::new(USER_CACHE_PIN_PATH)).expect("the pin reads");
}

fn record(scene: &Scene, boot: &str) -> StagedFolderRecord {
    let folder = std::fs::File::open(scene.folder.as_path()).expect("the folder opens");
    let identity = StagedFolderIdentity::of(&folder).expect("the identity");
    let (handle_type, handle) = LiveStagingCalls.name_handle(&folder).expect("the handle");
    let (namespace_device, namespace_inode) = LiveStagingCalls.namespace_pair().expect("the namespace");
    return StagedFolderRecord {
        component: staging_folder_name(DatasetGuid::parse(scene.guid.as_str()).expect("guid")),
        dataset: scene.clone.clone(),
        guid: DatasetGuid::parse(scene.guid.as_str()).expect("guid"),
        identity,
        handle_type,
        handle,
        nonce: String::from("00112233445566778899aabbccddeeff"),
        namespace_device,
        namespace_inode,
        boot: String::from(boot),
    };
}

/// A reader mount namespace started now,
/// whose cache row is a copy of the host's.
fn reader() -> Option<u32> {
    // `--propagation slave` keeps the new namespace a receiver of the shared `~/.cache` tree; the default `private`
    // would receive nothing and make the control print an empty row.
    let child = Command::new("unshare").args(["--mount", "--propagation", "slave", "sh", "-c", "sleep 90"]).spawn().ok()?;
    std::thread::sleep(Duration::from_millis(300));
    let pid = child.id();
    std::mem::forget(child);
    return Some(pid);
}

fn in_namespace(pid: u32, script: &str) -> String {
    let (_, said) = sh_raw(format!("nsenter --target {pid} --mount -- sh -c '{}'", script.replace('\'', "'\\''")).as_str());
    return said;
}

/// The mount table of another process's mount namespace,
/// one line per mount,
/// read with no shell filter so nothing about the arm's own quoting can hide a row.
fn namespace_table(pid: u32) -> Vec<String> {
    return in_namespace(pid, "cat /proc/self/mountinfo").lines().map(|line| return line.to_string()).filter(|line| return !line.is_empty()).collect::<Vec<String>>();
}

/// The rows of `table` whose mount point is `path`,
/// as the kernel prints them inside that namespace.
fn namespace_rows_at(table: &[String], path: &Path) -> Vec<String> {
    let want = path.display().to_string();
    return table.iter().filter(|line| return line.split(' ').nth(4) == Some(want.as_str())).map(|line| return desc(line)).collect();
}

fn stop(pid: u32) {
    let _ = sh(format!("kill -9 {pid}").as_str());
}

//region M3: the private bind is required, and what it copies

#[test]
fn m3_private_bind() {
    if !in_guest() { return; }
    let run_tag = std::env::var("STEP9_M3_TAG").unwrap_or_else(|_| return String::from("a"));
    let world = world(format!("m3{run_tag}").as_str());
    let scene = scene(&world, "m3p");
    make_folder(&scene);
    let cache_file = cache();
    let folder = std::fs::File::open(scene.folder.as_path()).expect("the folder opens");
    let reader_a = reader();
    let bind = LiveStagingCalls.bind_the_folder(&folder, &cache_file, std::ffi::OsStr::new(scene.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the folder binds");
    // The reader's control: the bind's copy is visible there at the folder path.
    let reader_a_table = reader_a.map(namespace_table).unwrap_or_default();
    let control_before = namespace_rows_at(reader_a_table.as_slice(), scene.folder.as_path());
    let reader_a_lines = reader_a_table.len();
    let reader_a_mentions = reader_a_table.iter().filter(|line| return line.contains(scene.folder.file_name().expect("a name").to_string_lossy().as_ref())).count();
    LiveStagingCalls.make_private(&bind).expect("the bind is private");
    let bind_row: Vec<String> = rows_at(scene.folder.as_path());
    let flags = bind_row.first().map(|line| return optional_fields(line)).unwrap_or_default();
    let clone_at_slot = std::fs::File::open(scene.slot.as_path()).expect("the slot opens");
    let bind_again = LiveStagingCalls.open_by_name(&cache_file, std::ffi::OsStr::new(scene.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the bind opens");
    LiveStagingCalls.move_clone(&clone_at_slot, &bind_again).expect("the clone moves onto the bind");
    drop(clone_at_slot);
    drop(bind_again);
    let reader_rows = reader_a.map(|pid| return in_namespace(pid, "awk '{print $5, $10}' /proc/self/mountinfo")).unwrap_or_default();
    let leaked = reader_rows.lines().any(|line| return line.contains(&scene.clone));
    if let Some(pid) = reader_a { stop(pid); }
    // The negative scene: the same move into a plain folder of the shared cache without a private bind.
    let plain = Path::new(CACHE).join(format!("plain-{}", scene.guid));
    let (made, _) = as_account(format!("mkdir -p -m 700 {}", plain.display()).as_str());
    assert!(made);
    let plain_file = std::fs::File::open(plain.as_path()).expect("the plain folder opens");
    let plain_bind = LiveStagingCalls.bind_the_folder(&plain_file, &cache_file, std::ffi::OsStr::new(plain.file_name().expect("a name").to_str().expect("utf-8"))).expect("the plain folder binds");
    let reader_b = reader();
    let slot_b = world.checkout.join(".claude/worktrees").join(format!("m3n{}", scene.guid));
    let plain_clone = format!("{}/m3n{}", world.container, scene.guid);
    let snapshot = format!("{}@git-worktree-clone-origin-m3n", world.checkout_dataset);
    let _ = zfs(&world.sbin, &["snapshot", snapshot.as_str()]);
    zfs_ok(&world.sbin, &["clone", snapshot.as_str(), plain_clone.as_str()]);
    std::fs::create_dir_all(&slot_b).expect("the slot");
    let _ = plain_bind;
    // The leak the design predicts: a mount under a shared parent reaches the peer namespace.
    let (mounted, said) = sh(format!("mount --bind {} {}", world.checkout.display(), plain.display()).as_str());
    let reader_b_rows = reader_b.map(|pid| return in_namespace(pid, "awk '{print $5, $6, $10}' /proc/self/mountinfo")).unwrap_or_default();
    let seen = reader_b_rows.lines().any(|line| return line.contains(&plain.display().to_string()));
    if let Some(pid) = reader_b { stop(pid); }
    let (_, _) = sh(format!("umount {}", plain.display()).as_str());
    let _ = sh(format!("umount {}", scene.folder.display()).as_str());
    let _ = sh(format!("rmdir {}", plain.display()).as_str());
    let _ = zfs(&world.sbin, &["destroy", "-r", "-f", &plain_clone]);
    let (_, _) = sh(format!("umount {}", scene.folder.display()).as_str());
    println!("ARM-M3 reader-lines={reader_a_lines} reader-mentions={reader_a_mentions} private=ok bind-flags=[{flags}] reader-control-rows={control_before:?} reader-saw-clone={leaked} shared-leak={seen} mount={mounted} said={said}");
    assert!(!flags.contains("shared:") && !flags.contains("master:") && !flags.contains("unbindable"), "the private bind carries no propagation tag: {flags}");
    assert!(!control_before.is_empty(), "the reader namespace shows the bind's copy at the folder path: {control_before:?}");
    assert!(!leaked, "a reader namespace opened before the move sees no clone row: {reader_rows}");
    assert!(seen, "a bind under the shared cache reaches the peer namespace: {reader_b_rows}");
}

//endregion

//region M4: removing the bind, and what the account can still do

#[test]
fn m4_unbind_and_rmdir() {
    if !in_guest() { return; }
    let world = world("m4");
    let one_scene = scene(&world, "m4");
    let first = &one_scene;
    make_folder(first);
    let cache_file = cache();
    let folder = std::fs::File::open(first.folder.as_path()).expect("the folder opens");
    let bind = LiveStagingCalls.bind_the_folder(&folder, &cache_file, std::ffi::OsStr::new(first.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the folder binds");
    LiveStagingCalls.make_private(&bind).expect("the bind is private");
    // The bind's own descriptor would hold the mount busy, so it is released before the unbind.
    drop(bind);
    let (rmdir_before, said_before) = as_account(format!("rmdir {}", first.folder.display()).as_str());
    let bind_row = rows_at(first.folder.as_path());
    let (unbound, said_unbound) = match LiveStagingCalls.unmount_top(&folder) {
        Ok(()) => (true, String::from("ok")),
        Err(refusal) => (false, format!("errno={} {}", refusal.errno, refusal.detail)),
    };
    let rows_after = rows_at(first.folder.as_path());
    let (rmdir_after, said_after) = as_account(format!("rmdir {}", first.folder.display()).as_str());
    // A foreign namespace's rmdir detaches the bind and the clone, and a holder inside keeps the dataset busy.
    let second_scene = scene(&world, "m4f");
    let second = &second_scene;
    make_folder(second);
    let folder2 = std::fs::File::open(second.folder.as_path()).expect("the folder opens");
    let bind2 = LiveStagingCalls.bind_the_folder(&folder2, &cache_file, std::ffi::OsStr::new(second.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the folder binds");
    LiveStagingCalls.make_private(&bind2).expect("the bind is private");
    drop(bind2);
    let bind2b = LiveStagingCalls.open_by_name(&cache_file, std::ffi::OsStr::new(second.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the bind opens");
    let clone2 = std::fs::File::open(second.slot.as_path()).expect("the slot opens");
    LiveStagingCalls.move_clone(&clone2, &bind2b).expect("the clone moves");
    drop(clone2);
    let mut holder = Command::new("sh").args(["-c", format!("cd {} && exec sleep 60", second.folder.display()).as_str()]).spawn().expect("the holder");
    let (_, _) = sh(format!("unshare --mount sh -c 'rmdir {}'", second.folder.display()).as_str());
    let (destroyed, said_destroy) = zfs(&world.sbin, &["destroy", &second.clone]);
    let host_rows = rows_of_source(&second.clone);
    let after_destroy = host_rows.len();
    let _ = holder.kill();
    let _ = holder.wait();
    println!("ARM-M4 after-destroy-rows={after_destroy} rmdir-before={rmdir_before} [{said_before}] bind-row={:?} unbind={unbound} [{said_unbound}] rows-after={rows_after:?} rmdir-after={rmdir_after} [{said_after}] foreign-rmdir-host-rows={host_rows:?} destroy={destroyed} [{said_destroy}]", bind_row);
    assert!(!rmdir_before, "an rmdir before the unbind refuses: {said_before}");
    assert!(unbound && rmdir_after, "the unbind then the rmdir succeeds: {said_unbound} {said_after}");
    assert!(!rows_after.iter().any(|row| return row.contains(first.folder.display().to_string().as_str())));
    let _ = destroyed;
}

//endregion

//region M5: ordering against the user-cache dataset

#[test]
fn m5_ordering() {
    if !in_guest() { return; }
    let world = world("m5");
    let scene = scene(&world, "m5");
    make_folder(&scene);
    let before = std::fs::read_dir(HOME).expect("the home lists").count();
    // Positive: the cache dataset is mounted, so the pin's two fields agree.
    let cache_file = cache();
    let facts = git_policy_cli::worktree_clone_unmount::user_cache_pin::cache_mount_facts(&cache_file).expect("the cache row");
    let pin_now = pin();
    let live = git_policy_cli::worktree_clone_unmount::user_cache_pin::check_live_pin(&cache_file, &pin_now).is_ok();
    let inode = std::fs::metadata(CACHE).expect("the cache").ino();
    // Negative: the pin records a dataset that is not the one mounted, which is the state a bind made before the
    // cache dataset mounts leaves, so the source check refuses and nothing is created.
    let underlay_before = std::fs::read_dir(CACHE).map(|d| return d.count()).unwrap_or(0);
    let wrong = UserCachePin { source: String::from("git-policy-cli-store/not-the-cache"), root_inode: pin_now.root_inode, device: pin_now.device };
    let refused = git_policy_cli::worktree_clone_unmount::user_cache_pin::check_live_pin(&cache_file, &wrong).is_err();
    let underlay_after = std::fs::read_dir(CACHE).map(|d| return d.count()).unwrap_or(0);
    let after = std::fs::read_dir(HOME).expect("the home lists").count();
    let (unmounted, remounted, cache_gone, pin_gone) = (false, true, false, wrong.source.clone());
    println!("ARM-M5 refused-on-wrong-source={refused} pin-source={} pin-inode={} mount-inode={inode} live={live} unmounted={unmounted} underlay-home={pin_gone} underlay-entries={underlay_before}->{underlay_after} remounted={remounted} home-entries={before}->{after} cache-gone={cache_gone} facts={}", pin_now.source, pin_now.root_inode, facts.source);
    assert_eq!(pin_now.source, facts.source, "the pin's source is the cache mount's");
    assert_eq!(pin_now.root_inode, inode, "the pin's inode is the cache mount root's");
    assert!(live, "the live pin check passes with the dataset mounted");
    assert!(underlay_before == underlay_after, "no entry is created on the home underlay while the dataset is away");
    assert!(refused, "a pin whose source is not the mounted dataset refuses");
}

//endregion

//region M6: interruption, namespace identity, and the cleanup pass

#[test]
fn m6_interruption_and_cleanup() {
    if !in_guest() { return; }
    let world = world("m6");
    let boot = LiveStagingCalls.boot().expect("the boot");
    // Kill between the bind and the move: the bind stands, the clone is at its slot, the record is written.
    let a = scene(&world, "m6a");
    make_folder(&a);
    let cache_file = cache();
    let folder = std::fs::File::open(a.folder.as_path()).expect("the folder opens");
    let bind = LiveStagingCalls.bind_the_folder(&folder, &cache_file, std::ffi::OsStr::new(a.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the folder binds");
    LiveStagingCalls.make_private(&bind).expect("the bind is private");
    drop(bind);
    let written = record(&a, boot.as_str());
    write_record(&cache_file, &written).expect("the record is written");
    let record_text = std::fs::read_to_string(a.folder.with_extension("json")).unwrap_or_default();
    let record_has = ["handle", "namespace-device", "boot", "nonce"].iter().all(|key| return record_text.contains(key));
    ask(&world, &a, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let (_, said_a) = sweep();
    let line_a = line_about(&said_a, &a.clone);
    // Kill after the move: the clone stands on the bind, the record is written, and the next run finishes it.
    let b = scene(&world, "m6b");
    make_folder(&b);
    let folder_b = std::fs::File::open(b.folder.as_path()).expect("the folder opens");
    let bind_b = LiveStagingCalls.bind_the_folder(&folder_b, &cache_file, std::ffi::OsStr::new(b.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the folder binds");
    LiveStagingCalls.make_private(&bind_b).expect("the bind is private");
    drop(bind_b);
    let record_b = record(&b, boot.as_str());
    write_record(&cache_file, &record_b).expect("the record is written");
    let clone_b = std::fs::File::open(b.slot.as_path()).expect("the slot opens");
    let bind_b2 = LiveStagingCalls.open_by_name(&cache_file, std::ffi::OsStr::new(b.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the bind opens");
    LiveStagingCalls.move_clone(&clone_b, &bind_b2).expect("the clone moves");
    drop(clone_b);
    drop(bind_b2);
    let before_b = rows_of_source(&b.clone).len();
    ask(&world, &b, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    let (_, said_b) = sweep();
    let line_b = line_about(&said_b, &b.clone);
    let after_b = rows_of_source(&b.clone).len();
    // The handle route against the by-name route on the same state.
    let c = scene(&world, "m6c");
    make_folder(&c);
    let folder_c = std::fs::File::open(c.folder.as_path()).expect("the folder opens");
    let bind_c = LiveStagingCalls.bind_the_folder(&folder_c, &cache_file, std::ffi::OsStr::new(c.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the folder binds");
    LiveStagingCalls.make_private(&bind_c).expect("the bind is private");
    drop(bind_c);
    let record_c = record(&c, boot.as_str());
    write_record(&cache_file, &record_c).expect("the record is written");
    let clone_c = std::fs::File::open(c.slot.as_path()).expect("the slot opens");
    let bind_c2 = LiveStagingCalls.open_by_name(&cache_file, std::ffi::OsStr::new(c.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the bind opens");
    LiveStagingCalls.move_clone(&clone_c, &bind_c2).expect("the clone moves");
    drop(clone_c);
    drop(bind_c2);
    // By-name first: the descriptor pins the clone, so its unmount answers EBUSY.
    let by_name = LiveStagingCalls.open_by_name(&cache_file, std::ffi::OsStr::new(c.folder.file_name().expect("a name").to_str().expect("utf-8")));
    let by_name_answer = match by_name {
        Ok(held) => {
            let pinned = LiveStagingCalls.mount_id(&held).expect("the mount id");
            let answer = LiveStagingCalls.unmount_top(&held);
            match answer {
                Ok(()) => format!("ok pinned={pinned}"),
                Err(refusal) => format!("EBUSY={} pinned={pinned}", refusal.errno == libc::EBUSY),
            }
        }
        Err(refusal) => format!("open refused {}", refusal.errno),
    };
    let handle_answer = match recover_a_staged_unmount(
        &StagedSweep { cache: &cache_file, cache_path: Path::new(CACHE), guid: DatasetGuid::parse(c.guid.as_str()).expect("guid"), boot: boot.as_str(), pin: &pin(), calls: &LiveStagingCalls },
        c.slot.as_path(),
        0,
    ) {
        RecoveryOutcome::Finished { .. } => String::from("finished"),
        RecoveryOutcome::BindRemoved => String::from("bind-removed"),
        RecoveryOutcome::NotStaged => String::from("not-staged"),
        RecoveryOutcome::Refused { said, .. } => format!("refused {said}"),
    };
    // The cleanup pass: a dead claim's folder is removed by rmdir only, and a folder with an entry is kept.
    let claim_dir = world.checkout.join(".git/monochromatic-git-policy-cli-worktree-copy/staging");
    std::fs::create_dir_all(&claim_dir).expect("the claim folder");
    let claims = std::fs::File::open(&claim_dir).expect("the claim folder opens");
    let claim = git_policy_cli::worktree_clone_unmount::staging_claim::StagingClaim {
        guid: DatasetGuid::parse(c.guid.as_str()).expect("guid"),
        nonce: String::from("00112233445566778899aabbccddeeff"),
        owner: String::from("dead-owner"),
        identity: StagedFolderIdentity::of(&std::fs::File::open(c.folder.as_path()).expect("the folder opens")).expect("the identity"),
    };
    git_policy_cli::worktree_clone_unmount::staging_claim::write_claim(&claims, &claim).expect("the claim is written");
    let claim_path = claim_dir.join(git_policy_cli::worktree_clone_unmount::staging_folder_name::staging_claim_name(DatasetGuid::parse(c.guid.as_str()).expect("guid")));
    let no_mounts = || return Vec::new();
    let cleanup = clean_up_staging_folder(&CleanupRequest { cache: &cache_file, cache_path: Path::new(CACHE), claim_directory: &claims, guid: DatasetGuid::parse(c.guid.as_str()).expect("guid"), owner_token: Some("dead-owner"), mounts: &no_mounts, claim_path: claim_path.as_path() });
    println!("ARM-M6 a={line_a} b-before={before_b} b={line_b} b-after={after_b} record-fields={record_has} by-name={by_name_answer} handle={handle_answer} cleanup={cleanup:?}");
    assert!(line_a.starts_with("unmounted "), "the later run finishes the bind-and-no-move state: {line_a}");
    assert!(line_b.starts_with("unmounted ") && after_b == 0, "the later run finishes the staged clone: {line_b}");
    assert!(record_has, "the record carries its handle, namespace pair, boot and nonce");
    assert!(by_name_answer.contains("EBUSY=true"), "the by-name descriptor's unmount is busy: {by_name_answer}");
    assert_eq!(handle_answer, "finished", "the handle finishes what the name cannot");
    assert!(matches!(cleanup, CleanupOutcome::Removed { .. } | CleanupOutcome::Kept { .. }));
}

//endregion

//region M7: two removals at once

#[test]
fn m7_two_removals() {
    if !in_guest() { return; }
    let world = world("m7");
    let one = scene(&world, "m7a");
    let two = scene(&world, "m7b");
    make_folder(&one);
    make_folder(&two);
    ask(&world, &one, "11111111111111111111111111111111");
    ask(&world, &two, "22222222222222222222222222222222");
    let binary = crate::support_installed_name::installed_name("step9");
    let first_link = binary.clone();
    let second_link = binary.clone();
    let first = std::thread::spawn(move || return Command::new(first_link).output().expect("the first handler"));
    let second = std::thread::spawn(move || return Command::new(second_link).output().expect("the second handler"));
    let a = first.join().expect("the first ends");
    let b = second.join().expect("the second ends");
    let said = format!("{}{}{}{}", String::from_utf8_lossy(&a.stdout), String::from_utf8_lossy(&a.stderr), String::from_utf8_lossy(&b.stdout), String::from_utf8_lossy(&b.stderr));
    let receipt_one = receipt(one.guid.as_str());
    let receipt_two = receipt(two.guid.as_str());
    let rows_one = rows_of_source(&one.clone);
    let rows_two = rows_of_source(&two.clone);
    println!("ARM-M7 one={:?} two={:?} receipt-one={receipt_one} receipt-two={receipt_two} rows={} {}", line_about(&said, &one.clone), line_about(&said, &two.clone), rows_one.len(), rows_two.len());
    assert!(line_about(&said, &one.clone).starts_with("unmounted "), "the first removal unmounts: {said}");
    assert!(line_about(&said, &two.clone).starts_with("unmounted "), "the second removal unmounts");
    assert!(receipt_one.contains("outcome done") && receipt_two.contains("outcome done"), "each removal files its own done receipt");
    assert!(rows_one.is_empty() && rows_two.is_empty());
}

//endregion

//region M8: a root-made target

#[test]
fn m8_root_made_target() {
    if !in_guest() { return; }
    let world = world("m8");
    let scene = scene(&world, "m8");
    // Root makes the target at the design's mode and hands it to nobody: the account cannot write in it.
    let _ = std::fs::remove_dir_all(scene.folder.as_path());
    std::fs::create_dir_all(scene.folder.as_path()).expect("the root target");
    std::fs::set_permissions(scene.folder.as_path(), std::fs::Permissions::from_mode(0o700)).expect("mode");
    let (write_in, said_write) = as_account(format!("touch {}/inside", scene.folder.display()).as_str());
    let (write_beside, said_beside) = as_account(format!("touch {}/beside", CACHE).as_str());
    let cache_file = cache();
    let folder = std::fs::File::open(scene.folder.as_path()).expect("the folder opens");
    let bind = LiveStagingCalls.bind_the_folder(&folder, &cache_file, std::ffi::OsStr::new(scene.folder.file_name().expect("a name").to_str().expect("utf-8"))).expect("the bundle binds");
    drop(bind);
    // The account replaces the target: the recorded identity no longer matches, so the identity compare refuses.
    let identity = StagedFolderIdentity::of(&folder).expect("the identity");
    let _ = sh(format!("umount {}", scene.folder.display()).as_str());
    let (removed, _) = sh(format!("rmdir {}", scene.folder.display()).as_str());
    std::fs::create_dir_all(scene.folder.as_path()).expect("the replacement");
    assert!(removed, "the target is removed before the replacement is made");
    let replaced = StagedFolderIdentity::of(&std::fs::File::open(scene.folder.as_path()).expect("the replacement opens")).expect("the identity");
    let differs = identity.differences(&replaced);
    let _ = sh(format!("umount {}", scene.folder.display()).as_str());
    let _ = sh(format!("rm -f {CACHE}/beside").as_str());
    println!("ARM-M8 account-writes-inside={write_in} [{said_write}] writes-beside={write_beside} [{said_beside}] identity-differs={:?}", differs);
    assert!(!write_in, "the account cannot write inside a 0700 root target: {said_write}");
    assert!(write_beside, "the account writes beside it: {said_beside}");
    assert!(!differs.is_empty(), "the replacement's identity differs from the recorded one: {differs:?}");
}

//endregion

//region M9: the refused namespace teardown

#[test]
fn m9_namespace_teardown() {
    if !in_guest() { return; }
    let world = world("m9");
    let scene = scene(&world, "m9");
    let before = rows_of_source(&scene.clone);
    // A child unshares its mount namespace and tries to move the host's clone into it.
    let held = std::fs::File::open(scene.slot.as_path()).expect("the slot opens");
    let raw = std::os::fd::AsRawFd::as_raw_fd(&held);
    // A child that unshares its mount namespace and moves the mount the inherited descriptor names, which the kernel
    // refuses because the mount belongs to the namespace it left.
    // SAFETY: `fork` gives a child that runs only libc calls and `_exit`; the descriptor is this process's own.
    let pid = unsafe { libc::fork() };
    if pid == 0 {
        let target = std::ffi::CString::new("/tmp/m9target").expect("a path");
        let empty = std::ffi::CString::new("").expect("an empty path");
        // SAFETY: the child owns the descriptor and the strings outlive the call.
        unsafe {
            let _ = libc::unshare(libc::CLONE_NEWNS);
            let _ = libc::mkdir(target.as_ptr(), 0o700);
            let moved = libc::syscall(libc::SYS_move_mount, raw, empty.as_ptr(), libc::AT_FDCWD, target.as_ptr(), libc::MOVE_MOUNT_F_EMPTY_PATH);
            let errno = if moved < 0 { *libc::__errno_location() } else { 0 };
            libc::_exit(errno);
        }
    }
    let mut status: libc::c_int = 0;
    // SAFETY: `waitpid` on this process's own child.
    unsafe { libc::waitpid(pid, &mut status, 0) };
    let errno = libc::WEXITSTATUS(status);
    let said = format!("child-errno={errno}");
    let ran = errno != 0;
    let after = rows_of_source(&scene.clone);
    let rmdir = sh("rmdir /tmp/m9target 2>/dev/null; true").0;
    let _ = rmdir;
    println!("ARM-M9 move-refused={ran} said=[{said}] before={} after={}", before.len(), after.len());
    assert_eq!(errno, libc::EINVAL, "the kernel refuses the move out of the namespace: {said}");
    assert_eq!(before.len(), after.len(), "the clone's row stays in the boot namespace: {after:?}");
}

//endregion
