//! What:
//!  Plan step 22's guest measurements in one throwaway guest:
//!  the add against ordinary Git under the same workload with an interleaved
//!  control and its stages apart,
//!  the mount request's wait and the unmount's,
//!  the sweep's own cost and the lost-event behaviour,
//!  what expiration and removal free,
//!  the forced arm's three waits,
//!  and the largest snapshot-destroy request the binding sends.
//! Why:
//!  The plan hands every one of those to step 22
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 22:
//! The measurements that size waits and retention"),
//!  and each shipped limit must cite a measurement beside its value.
//!  The arms build a real pool, clone datasets and mounts as root,
//!  so no control on a person's machine may run them:
//!  they run only where `CLI_GIT_ZFS_STEP22_GUEST` names a disposable guest,
//!  on a file-backed pool,
//!  and they print `SKIP` everywhere else,
//!  which is why a run that proves nothing says so.
//!  The checkout is the dataset the step 19 design describes,
//!  made here rather than migrated,
//!  because the owner's migration has not run:
//!  every number is the guest's semantics on a file-backed pool and never this
//!  host's storage.
//!  The control for the add is ordinary Git under the same tracked tree,
//!  interleaved round by round.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! prepare(); for (let n = 0; n < ROUNDS; n += 1) { gitAdd(n); wrapperAdd(n); } sweeps(); waits(); retention(); budget();
//! ```

#![cfg(target_os = "linux")]

use super::zfs_sweep_guest::{clone_the_checkouts_snapshot, make_the_slot, prepare_the_store, required, sweep, write_configuration, write_passing_probe_record, zfs, zfs_ok};
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness::{ReadyState, ReadinessRecord, SweepState};
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use monochromatic_zfs_request_to_openzfs_kernel_device::{DeviceError, NvList, Request, ZfsDevice, destroy_snaps};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The variable the guest driver sets, so a host run skips instead of touching a pool.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_STEP22_GUEST";

/// The pool the driver made.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_STEP22_POOL";

/// The OpenZFS userspace the arm's own reads use.
const SBIN_VARIABLE: &str = "CLI_GIT_ZFS_STEP22_SBIN";

/// The checkout root the arm mounts the design's dataset at.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_STEP22_ROOT";

/// The tracked tree the workload is copied from, which the driver pushed.
const WORKLOAD_VARIABLE: &str = "CLI_GIT_ZFS_STEP22_WORKLOAD";

/// The checkout dataset's leaf name the step 19 design fixes.
const CHECKOUT_LEAF: &str = "monochromatic-git-checkout";

/// The container's leaf name the design fixes beside it.
const CONTAINER_LEAF: &str = "monochromatic-git-worktree-clone-datasets";

/// The desired-state property the add writes its mount request under.
const DESIRED_STATE: &str = "org.monochromatic:git-worktree-clone-mount-desired-state";

/// The wrapper program the driver built.
const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// The real Git the control arm runs, never the wrapper's link.
const REAL_GIT: &str = "/usr/bin/git";

/// How many interleaved rounds each arm runs.
/// Why:
///  An odd count gives each arm a median and leaves neither warmed by the
///  other at the end, while keeping the guest's whole arm inside its bound.
const ROUNDS: usize = 5;

/// How often a poller looks at a dataset, a record or a mount.
const POLL: Duration = Duration::from_millis(20);

/// How long one add may take before the arm gives up on it.
const ADD_WAIT: Duration = Duration::from_secs(120);

/// How long the lost-event window watches for a mount no trigger can make.
const LOST_EVENT_WINDOW: Duration = Duration::from_secs(5);

/// How long a transient unit may take to leave its cgroup.
const STOP_WAIT_BOUND: Duration = Duration::from_secs(30);

/// How many build-like mebibytes the retention arm writes into an ignored folder.
const BUILD_OUTPUT_MIB: usize = 12;

/// The snapshot-destroy request sizes the budget arm sends, in names.
/// Why:
///  The design's budget is bytes of encoded input,
///  and one name is the same flag pair at every count,
///  so three counts read the same rule at three scales.
const REQUEST_NAMES: [usize; 3] = [64, 512, 2048];

/// The account this arm runs as, whose delegation the route's preflight reads.
const ACCOUNT: &str = "0";

/// The binding record's phases in the order the add writes them.
const PHASES: [&str; 7] = ["claimed", "slot", "registered", "snapshotted", "cloned", "mounted", "filled"];

/// Whether this run is inside the guest, or a skip line on every other machine.
fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP binary_zfs_step_22_measurement: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest");
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname").map(|text| return text.trim().to_string()).unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// A nonce of the designed width, from a short seed that names the arm.
fn nonce(seed: &str) -> String {
    let mut text: String = String::new();
    while text.len() < 32 {
        text.push_str(seed);
    }
    text.truncate(32);
    return text;
}

/// The fixture the arm drives: the design's layout, a repository, and the
/// wrapper's own link.
struct Fixture {
    /// The checkout root.
    root: PathBuf,
    /// The pool's name.
    pool: String,
    /// The OpenZFS userspace.
    sbin: String,
    /// The checkout dataset the layout made.
    checkout: String,
    /// The clone container the layout made.
    container: String,
}

/// What one add round measured.
struct AddRound {
    /// The wall time from the command's start to its end.
    elapsed: Duration,
    /// The status the command exited with.
    status: Option<i32>,
    /// Each binding phase's first sighting, in milliseconds from the start.
    phases: Vec<(&'static str, u128)>,
    /// What the command printed.
    said: String,
}

impl Fixture {
    /// Make the design's layout, the repository and the wrapper's link.
    fn prepare() -> Fixture {
        let root: PathBuf = PathBuf::from(required(ROOT_VARIABLE));
        let pool: String = required(POOL_VARIABLE);
        let sbin: String = required(SBIN_VARIABLE);
        let (checkout, container): (String, String) = make_the_designs_layout(&sbin, &pool, root.as_path());
        let fixture: Fixture = Fixture { root, pool, sbin, checkout, container };
        fixture.copy_the_workload();
        fixture.make_the_repository();
        fixture.make_the_worktrees_folder();
        fixture.publish_the_readiness();
        write_passing_probe_record();
        prepare_the_store(fixture.root.as_path());
        write_configuration(fixture.root.as_path(), None, None);
        fixture.make_the_wrapper_link();
        return fixture;
    }

    /// The workload's own folder inside the guest, which the driver pushed.
    fn workload(&self) -> PathBuf {
        return PathBuf::from(required(WORKLOAD_VARIABLE));
    }

    /// The folder Git's common directory is.
    fn common(&self) -> PathBuf {
        return self.root.join(".git");
    }

    /// The records' home folder.
    fn home(&self) -> PathBuf {
        return self.common().join("monochromatic-git-policy-cli");
    }

    /// One slot's folder, which the design keeps under the checkout.
    fn slot(&self, name: &str) -> PathBuf {
        return self.root.join(".claude").join("worktrees").join(name);
    }

    /// One slot's clone dataset.
    fn clone(&self, name: &str) -> String {
        return format!("{}/{}", self.container, name);
    }

    /// The wrapper started as `git`, in a folder of its own.
    fn wrapper_git(&self) -> PathBuf {
        return PathBuf::from("/root/step22-bin").join("git");
    }

    /// The home folder each arm runs with, outside the checkout so the source
    /// stays clean.
    fn home_folder(&self) -> PathBuf {
        return PathBuf::from("/root/step22-home");
    }

    /// The session's runtime folder, which the companion reads.
    fn runtime_folder(&self) -> PathBuf {
        return PathBuf::from("/root/step22-runtime");
    }

    /// Copy the tracked workload into the checkout.
    /// Why:
    ///  Both arms must fill the same tracked tree,
    ///  and the wrapper's own sources are the one real tree the driver already
    ///  pushed into the guest,
    ///  so the fill is timed on files a person's checkout really holds.
    fn copy_the_workload(&self) {
        let from: PathBuf = self.workload();
        let to: PathBuf = self.root.join("src");
        copy_tree(from.as_path(), to.as_path());
        std::fs::write(self.root.join(".gitignore"), b".claude/\n").expect("the ignore rule");
        std::fs::write(self.root.join("README.md"), b"step 22's workload\n").expect("the readme");
    }

    /// A repository with the workload committed, and the trash place ignored.
    fn make_the_repository(&self) {
        if self.common().is_dir() {
            return;
        }
        // A file written in the same second as the index is a racy entry, which
        // the route's clean check refuses to vouch for.
        std::thread::sleep(Duration::from_secs(1));
        self.git(&["init", "--quiet", "--initial-branch=main"]);
        self.git(&["add", "."]);
        self.git(&["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "--no-verify", "-m", "one"]);
    }

    /// The folder the route's slots live in, at the mode the design's unit sets.
    fn make_the_worktrees_folder(&self) {
        let parent: PathBuf = self.root.join(".claude").join("worktrees");
        std::fs::create_dir_all(parent.as_path()).expect("the worktrees folder");
        std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the folder's mode");
    }

    /// The readiness report the add's preflight requires, written as the boot
    /// unit would.
    fn publish_the_readiness(&self) {
        let file: ReadinessFile = ReadinessFile::live();
        file.prepare().expect("the readiness directory");
        let boot: String = std::fs::read_to_string("/proc/sys/kernel/random/boot_id").expect("this boot's identifier").trim().to_string();
        let record: ReadinessRecord = ReadinessRecord {
            boot,
            pool_name: self.pool.clone(),
            pool_state: ReadyState::Ready,
            checkout_path: self.root.display().to_string(),
            checkout_state: ReadyState::Ready,
            sweep_nonce: None,
            sweep_state: SweepState::Never,
            time: git_policy_cli::iso_time::now_iso(),
        };
        file.publish(&record).expect("the readiness report");
    }

    /// The link every arm starts as `git`, outside the checkout.
    fn make_the_wrapper_link(&self) {
        let bin: PathBuf = PathBuf::from("/root/step22-bin");
        std::fs::create_dir_all(bin.as_path()).expect("the bin folder");
        std::fs::create_dir_all(self.home_folder().as_path()).expect("the home folder");
        std::fs::create_dir_all(self.runtime_folder().as_path()).expect("the runtime folder");
        std::fs::set_permissions(self.runtime_folder().as_path(), std::fs::Permissions::from_mode(0o700)).expect("the runtime folder's mode");
        let link: PathBuf = self.wrapper_git();
        let _stale: std::io::Result<()> = std::fs::remove_file(link.as_path());
        std::os::unix::fs::symlink(WRAPPER, link.as_path()).expect("the wrapper's link");
    }

    /// The environment every command runs with, so no host variable reaches it.
    fn environment(&self, program_folder: &Path) -> Vec<(String, String)> {
        return vec![
            (String::from("PATH"), format!("{}:/usr/bin:/bin", program_folder.display())),
            (String::from("HOME"), self.home_folder().display().to_string()),
            (String::from("XDG_RUNTIME_DIR"), self.runtime_folder().display().to_string()),
            (String::from("GIT_CONFIG_NOSYSTEM"), String::from("1")),
            (String::from("GIT_CONFIG_GLOBAL"), String::from("/nonexistent-global-config")),
            (String::from("GIT_AUTHOR_NAME"), String::from("Fixture")),
            (String::from("GIT_AUTHOR_EMAIL"), String::from("fixture@example.invalid")),
            (String::from("GIT_COMMITTER_NAME"), String::from("Fixture")),
            (String::from("GIT_COMMITTER_EMAIL"), String::from("fixture@example.invalid")),
        ];
    }

    /// One real Git command in the checkout, which must succeed.
    fn git(&self, arguments: &[&str]) -> Output {
        let output: Output = command(REAL_GIT, arguments, self.root.as_path(), &self.environment(Path::new("/usr/bin")));
        assert!(output.status.success(), "git {arguments:?} was refused: {}", String::from_utf8_lossy(output.stderr.as_slice()));
        return output;
    }

    /// One `zfs` command as root, with its status, output and error.
    fn zfs(&self, arguments: &[&str]) -> (bool, String, String) {
        return zfs(self.sbin.as_str(), "zfs", arguments, None);
    }

    /// One `zfs` command that must succeed.
    fn zfs_ok(&self, arguments: &[&str]) {
        zfs_ok(self.sbin.as_str(), arguments);
    }

    /// One dataset's property, in bytes, where the property is a number.
    fn bytes(&self, property: &str, dataset: &str) -> u64 {
        let (_ok, said, _error): (bool, String, String) = self.zfs(&["get", "-H", "-p", "-o", "value", property, dataset]);
        return said.trim().parse().unwrap_or(0);
    }

    /// One slot's binding record, or nothing when no phase wrote it.
    fn record(&self, name: &str) -> Option<String> {
        return std::fs::read_to_string(self.home().join("worktree-clones").join("v1").join(format!("{name}.binding"))).ok();
    }

    /// The origin snapshot the add recorded, or nothing.
    fn origin_of(&self, name: &str) -> Option<String> {
        let record: String = self.record(name)?;
        return record.lines().find_map(|line| return line.strip_prefix("originSnapshot=")).map(String::from);
    }

    /// Run ordinary Git's own `git worktree add`, the control arm.
    fn git_add(&self, name: &str) -> AddRound {
        let slot: PathBuf = self.slot(name);
        let started: Instant = Instant::now();
        let output: Output = command(REAL_GIT, &["worktree", "add", slot.to_str().expect("a text path")], self.root.as_path(), &self.environment(Path::new("/usr/bin")));
        return AddRound {
            elapsed: started.elapsed(),
            status: output.status.code(),
            phases: Vec::new(),
            said: String::from_utf8_lossy(output.stderr.as_slice()).to_string(),
        };
    }

    /// Run the wrapper's own add, which is the ZFS clone route, watching its
    /// binding record and mounting the clone when it asks.
    fn wrapper_add(&self, name: &str, by_sweep: bool) -> AddRound {
        let slot: PathBuf = self.slot(name);
        let clone: String = self.clone(name);
        let record: PathBuf = self.home().join("worktree-clones").join("v1").join(format!("{name}.binding"));
        let started: Instant = Instant::now();
        let mut command_line: Command = Command::new(self.wrapper_git());
        command_line.args(["worktree", "add", slot.to_str().expect("a text path")]).current_dir(self.root.as_path()).env_clear();
        for (key, value) in self.environment(Path::new("/root/step22-bin")) {
            command_line.env(key, value);
        }
        command_line.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child: Child = command_line.spawn().expect("the wrapper starts");
        let mut phases: Vec<(&'static str, u128)> = Vec::new();
        let mut attach_made: bool = false;
        let mut ended: bool = false;
        while started.elapsed() < ADD_WAIT {
            for word in PHASES {
                if phases.iter().any(|(already, _at)| return *already == word) {
                    continue;
                }
                if record_lines(record.as_path()).iter().any(|line| return *line == format!("phase={word}")) {
                    phases.push((word, started.elapsed().as_millis()));
                }
            }
            if let Ok(Some(_status)) = child.try_wait() {
                ended = true;
                break;
            }
            // The probe stops once the mount stands: a poll that kept asking
            // while the fill ran would slow the arm it is timing.
            if !attach_made && slot.is_dir() && dataset_stands(self.sbin.as_str(), clone.as_str()) && intent_says_mount(self.sbin.as_str(), clone.as_str()) && !mounted_at_path(slot.as_path()) {
                let attach: Instant = Instant::now();
                if by_sweep {
                    let run: super::zfs_sweep_guest::Run = sweep();
                    assert_eq!(run.status, Some(0), "the sweep that mounts the add's clone reached its end: {}", run.said);
                    println!("STEP22 add-attach=sweep slot={name} sweep-ms={}", run.elapsed.as_millis());
                } else {
                    let output: Output = Command::new("/usr/bin/mount")
                        .args(["-t", "zfs", "-o", "nosuid,nodev", clone.as_str(), slot.to_str().expect("a text path")])
                        .output()
                        .expect("mount starts");
                    assert!(output.status.success(), "the mount of {clone} at the slot was refused: {}", String::from_utf8_lossy(output.stderr.as_slice()));
                    println!("STEP22 add-attach=direct slot={name} mount-call-ms={}", attach.elapsed().as_millis());
                }
                attach_made = true;
            }
            std::thread::sleep(POLL);
        }
        let output: Output = child.wait_with_output().expect("the add ends");
        let _ended: bool = ended;
        for word in PHASES {
            if phases.iter().any(|(already, _at)| return *already == word) {
                continue;
            }
            if record_lines(record.as_path()).iter().any(|line| return *line == format!("phase={word}")) {
                phases.push((word, started.elapsed().as_millis()));
            }
        }
        return AddRound {
            elapsed: started.elapsed(),
            status: output.status.code(),
            phases,
            said: format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice())),
        };
    }

    /// Take a slot's clone, mount, folder, registration and record away, so the
    /// next round starts from nothing.
    fn clear(&self, name: &str) {
        let slot: PathBuf = self.slot(name);
        let clone: String = self.clone(name);
        unmount_quiet(slot.as_path());
        let origin: Option<String> = self.origin_of(name);
        let _destroyed: (bool, String, String) = self.zfs(&["destroy", "-r", "-f", clone.as_str()]);
        if let Some(snapshot) = origin {
            let _gone: (bool, String, String) = self.zfs(&["destroy", "-f", snapshot.as_str()]);
        }
        let _folder: std::io::Result<()> = std::fs::remove_dir_all(slot.as_path());
        let _records: std::io::Result<()> = std::fs::remove_dir_all(self.home().as_path());
        let output: Output = command(REAL_GIT, &["worktree", "prune"], self.root.as_path(), &self.environment(Path::new("/usr/bin")));
        let _pruned: bool = output.status.success();
    }

    /// One clone of a fresh checkout snapshot, asking to be mounted at a slot
    /// Git registered.
    fn make_a_clone_asking(&self, name: &str, seed: &str) {
        let intent: String = format!("v1:{}:mount", nonce(seed));
        let _clone: String = clone_the_checkouts_snapshot(self.sbin.as_str(), self.checkout.as_str(), self.container.as_str(), name, nonce(seed).as_str(), Some(intent.as_str()));
        make_the_slot(self.root.as_path(), name);
    }

    /// Take one of the arm's own clones away, with the checkout snapshot it was
    /// cloned from, without touching a record the arm still reads.
    fn forget(&self, name: &str, seed: &str) {
        let clone: String = self.clone(name);
        let snapshot: String = format!("{}@git-worktree-clone-origin-{}", self.checkout, nonce(seed));
        let _destroyed: (bool, String, String) = self.zfs(&["destroy", "-r", "-f", clone.as_str()]);
        let _snapshot: (bool, String, String) = self.zfs(&["destroy", "-f", snapshot.as_str()]);
        let _folder: std::io::Result<()> = std::fs::remove_dir_all(self.slot(name).as_path());
        let _inherited: (bool, String, String) = self.zfs(&["inherit", DESIRED_STATE, self.container.as_str()]);
        let output: Output = command(REAL_GIT, &["worktree", "prune"], self.root.as_path(), &self.environment(Path::new("/usr/bin")));
        let _pruned: bool = output.status.success();
    }
}

/// What:
///  The design's dataset layout for the guest:
///  the pool's home dataset,
///  the checkout dataset below it with the leaf name step 19 fixes,
///  mounted at the configured path and private,
///  and the clone container beside it as a legacy dataset.
/// Why:
///  The plan's storage half has not run and the owner's migration has not
///  either,
///  so the guest makes the checkout dataset the step 19 design describes rather
///  than finding one.
fn make_the_designs_layout(sbin: &str, pool: &str, checkout: &Path) -> (String, String) {
    let home: String = format!("{pool}/ROOT/cos/home");
    let checkout_dataset: String = format!("{home}/{CHECKOUT_LEAF}");
    let container: String = format!("{home}/{CONTAINER_LEAF}");
    let created: (bool, String, String) = zfs(sbin, "zfs", &["create", "-p", "-o", "mountpoint=none", home.as_str()], None);
    assert!(created.0, "the home dataset is made: {}", created.2);
    let _replaced: (bool, String, String) = zfs(sbin, "zfs", &["destroy", "-r", "-f", checkout_dataset.as_str()], None);
    let mountpoint: String = format!("mountpoint={}", checkout.display());
    let made: (bool, String, String) = zfs(sbin, "zfs", &["create", "-o", mountpoint.as_str(), checkout_dataset.as_str()], None);
    assert!(made.0, "the checkout dataset is made: {}", made.2);
    std::fs::create_dir_all(checkout).expect("the checkout's mount point");
    let _private: Output = Command::new("/usr/bin/mount").args(["--make-private", checkout.to_str().expect("a text path")]).output().expect("the private mount starts");
    let container_made: (bool, String, String) = zfs(
        sbin,
        "zfs",
        &["create", "-o", "mountpoint=legacy", "-o", "canmount=off", "-o", "setuid=off", "-o", "devices=off", container.as_str()],
        None,
    );
    assert!(container_made.0, "the clone container is made: {}", container_made.2);
    // The nine permissions the design scopes, handed to the account this arm
    // runs as, which is root: the route's preflight reads the caller's own
    // delegation and refuses without them.
    for arguments in [
        vec!["allow", "-l", "-u", ACCOUNT, "snapshot", checkout_dataset.as_str()],
        vec!["allow", "-d", "-u", ACCOUNT, "clone,mount,destroy", checkout_dataset.as_str()],
        vec!["allow", "-l", "-u", ACCOUNT, "create,mount", container.as_str()],
        vec!["allow", "-d", "-u", ACCOUNT, "mount,destroy,userprop", container.as_str()],
    ] {
        let granted: (bool, String, String) = zfs(sbin, "zfs", arguments.as_slice(), None);
        assert!(granted.0, "the decided delegation is granted: {}", granted.2);
    }
    return (checkout_dataset, container);
}

/// The lines one binding record holds, or nothing while no phase wrote it.
fn record_lines(record: &Path) -> Vec<String> {
    return std::fs::read_to_string(record).map(|text| return text.lines().map(String::from).collect()).unwrap_or_default();
}

/// Whether a dataset answers `zfs list`.
fn dataset_stands(sbin: &str, dataset: &str) -> bool {
    return zfs(sbin, "zfs", &["list", "-H", "-o", "name", dataset], None).0;
}

/// Whether one clone's durable intent asks for its mount.
fn intent_says_mount(sbin: &str, clone: &str) -> bool {
    let (_ok, said, _error): (bool, String, String) = zfs(sbin, "zfs", &["get", "-H", "-o", "value", DESIRED_STATE, clone], None);
    return said.contains(":mount");
}

/// Whether a mount stands at one path in this process's table.
fn mounted_at_path(path: &Path) -> bool {
    let table: String = std::fs::read_to_string("/proc/self/mountinfo").expect("the mount table");
    let wanted: String = path.display().to_string();
    return table.lines().any(|line| return line.split(' ').nth(4) == Some(wanted.as_str()));
}

/// Take one mount down where it stands, without insisting that it did.
fn unmount_quiet(folder: &Path) {
    let _unmounted: Output = Command::new("/usr/bin/umount").arg(folder).output().expect("the cleanup unmount starts");
}

/// Run one program with an environment of its own and no inherited variable.
fn command(program: &str, arguments: &[&str], directory: &Path, environment: &[(String, String)]) -> Output {
    let mut started: Command = Command::new(program);
    started.args(arguments).current_dir(directory).env_clear();
    for (key, value) in environment {
        started.env(key, value);
    }
    return started.output().unwrap_or_else(|error| panic!("{program} {arguments:?} could not start: {error}"));
}

/// Copy one whole folder tree.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the copy's folder");
    for entry in std::fs::read_dir(from).expect("the tree reads").flatten() {
        let path: PathBuf = entry.path();
        let target: PathBuf = to.join(entry.file_name());
        if path.is_dir() {
            copy_tree(path.as_path(), target.as_path());
        } else {
            std::fs::copy(path.as_path(), target.as_path()).expect("the file copies");
        }
    }
}

/// The middle reading of a copy, which is the median of an odd count.
fn middle(values: &[u128]) -> u128 {
    let mut sorted: Vec<u128> = values.to_vec();
    sorted.sort_unstable();
    return sorted[sorted.len() / 2];
}

/// Count the files below one folder.
fn count_files(root: &Path) -> u64 {
    let mut count: u64 = 0;
    for entry in std::fs::read_dir(root).expect("the folder reads").flatten() {
        if entry.path().is_dir() {
            count += count_files(entry.path().as_path());
        } else {
            count += 1;
        }
    }
    return count;
}

/// The bytes the files below one folder hold.
fn tree_bytes(root: &Path) -> u64 {
    let mut bytes: u64 = 0;
    for entry in std::fs::read_dir(root).expect("the folder reads").flatten() {
        let metadata: std::fs::Metadata = entry.metadata().expect("the entry's metadata");
        if metadata.is_dir() {
            bytes += tree_bytes(entry.path().as_path());
        } else {
            bytes += metadata.len();
        }
    }
    return bytes;
}

/// Print one arm's control, treatment, spread and ratio.
fn report_pairs(arm: &str, controls: &[Duration], treatments: &[Duration]) {
    let control: Vec<u128> = controls.iter().map(|value| return value.as_millis()).collect();
    let treatment: Vec<u128> = treatments.iter().map(|value| return value.as_millis()).collect();
    let control_mid: u128 = middle(&control);
    let treatment_mid: u128 = middle(&treatment);
    println!(
        "STEP22 {arm} control-ms={control:?} control-mid={control_mid} treatment-ms={treatment:?} treatment-mid={treatment_mid} ratio-percent={:.1}",
        (treatment_mid as f64 / control_mid as f64) * 100.0
    );
}

/// What:
///  The add against ordinary Git, interleaved round by round, with the binding
///  record's phases timed so the Git fill, the snapshot, the clone, the mount
///  wait and the fill's own work are separate.
/// Why:
///  The plan's acceptance text asks for exactly those stages apart under one
///  workload with an interleaved control,
///  because a single total would hide which stage a step changes.
fn add_rounds(fixture: &Fixture) {
    let mut controls: Vec<Duration> = Vec::new();
    let mut treatments: Vec<Duration> = Vec::new();
    for round in 0..ROUNDS {
        let control_name: String = format!("git-{round}");
        let control: AddRound = fixture.git_add(control_name.as_str());
        assert_eq!(control.status, Some(0), "round {round}: Git's own add made the worktree: {}", control.said);
        println!("STEP22 add arm=git round={round} ms={}", control.elapsed.as_millis());
        if round == 0 {
            // The control's own registration is the same administration the
            // route copies and discards, so the discard is timed here once.
            discard_arm(fixture);
        }
        fixture.clear(control_name.as_str());

        let treatment_name: String = format!("clone-{round}");
        // Even rounds let the built sweep attach, which is what ZED's start
        // does; odd rounds mount at once, the smallest wait the route can see,
        // so the two shapes bracket what a handler adds to the wait.
        let by_sweep: bool = round % 2 == 0;
        let treatment: AddRound = fixture.wrapper_add(treatment_name.as_str(), by_sweep);
        assert_eq!(treatment.status, Some(0), "round {round}: the clone route made the worktree: {}", treatment.said);
        // A wrapper add that fell back to Git's own checkout would also exit
        // zero, so the record's last phase is what proves the route ran.
        assert!(
            treatment.phases.iter().any(|(word, _at)| return *word == "filled"),
            "round {round}: the add reached the clone route's filled phase rather than the ladder's fallback: {}",
            treatment.said
        );
        let stages: String = treatment.phases.iter().map(|(word, at)| return format!("{word}={at}")).collect::<Vec<String>>().join(",");
        println!(
            "STEP22 add arm=clone round={round} ms={} attached-by={} phases-ms={stages}",
            treatment.elapsed.as_millis(),
            match by_sweep {
                true => "sweep",
                false => "mount-call",
            }
        );
        controls.push(control.elapsed);
        treatments.push(treatment.elapsed);
        fixture.clear(treatment_name.as_str());
    }
    report_pairs("add-total", &controls, &treatments);
}

/// What:
///  The discard the route's fill pays and the ordinary Git arm has no stage for:
///  deleting the copied Git administration, at the bytes and file count the
///  route's own copy holds.
/// Why:
///  The acceptance text asks for the disposal to be reported as a contribution
///  against the control's total,
///  and the route leaves no trash behind to time,
///  so the arm deletes a copy of the same administration and times it.
fn discard_arm(fixture: &Fixture) {
    let mut times: Vec<u128> = Vec::new();
    let mut files: u64 = 0;
    let mut bytes: u64 = 0;
    for round in 0..ROUNDS {
        let from: PathBuf = fixture.common();
        let to: PathBuf = fixture.root.join(".claude").join("discard-measure");
        let _stale: std::io::Result<()> = std::fs::remove_dir_all(to.as_path());
        copy_tree(from.as_path(), to.as_path());
        files = count_files(to.as_path());
        bytes = tree_bytes(to.as_path());
        let started: Instant = Instant::now();
        std::fs::remove_dir_all(to.as_path()).expect("the copied administration goes");
        times.push(started.elapsed().as_millis());
        println!("STEP22 discard round={round} files={files} bytes={bytes} ms={}", times[times.len() - 1]);
    }
    println!("STEP22 discard files={files} bytes={bytes} ms={times:?} mid={}", middle(&times));
}

/// What:
///  The sweep's own cost with one clone and with sixteen, and what a lost event
///  leaves: a request no waiter and no handler sees stays unmounted until a
///  trigger fires.
/// Why:
///  The design says a bound on how many sweeps may run in a period needs the
///  sweep's measured cost first,
///  and the lost-event behaviour decides whether that bound is even the right
///  shape.
fn sweep_cost_arm(fixture: &Fixture) {
    fixture.make_a_clone_asking("sweep-one", "aa");
    let one: super::zfs_sweep_guest::Run = sweep();
    assert_eq!(one.status, Some(0), "the one-clone sweep reached its end: {}", one.said);
    println!("STEP22 sweep clones=1 ms={} attached={}", one.elapsed.as_millis(), mounted_at_path(fixture.slot("sweep-one").as_path()));
    unmount_quiet(fixture.slot("sweep-one").as_path());
    fixture.forget("sweep-one", "aa");

    // The sweep's cost against the number of clones it walks, at two sizes a
    // parent could hold, so the period a heartbeat could use has a curve rather
    // than one point.
    sweep_over(fixture, 16, "bb");
    sweep_over(fixture, 64, "bc");

    // The lost event: a request is written while no sweep runs and no caller
    // waits, and the arm watches a bounded window for a mount no trigger can
    // make.
    fixture.make_a_clone_asking("lost-event", "cc");
    let watched: Instant = Instant::now();
    let mut appeared: bool = false;
    while watched.elapsed() < LOST_EVENT_WINDOW {
        if mounted_at_path(fixture.slot("lost-event").as_path()) {
            appeared = true;
            break;
        }
        std::thread::sleep(POLL);
    }
    let after: super::zfs_sweep_guest::Run = sweep();
    println!(
        "STEP22 lost-event window-ms={} mounted-in-window={appeared} then-sweep-ms={} mounted-after-sweep={}",
        LOST_EVENT_WINDOW.as_millis(),
        after.elapsed.as_millis(),
        mounted_at_path(fixture.slot("lost-event").as_path())
    );
    unmount_quiet(fixture.slot("lost-event").as_path());
    fixture.forget("lost-event", "cc");
}

/// What:
///  One sweep's cost over `count` clones of the checkout's snapshot, each
///  asking for its mount, and how many it attached.
/// Why:
///  The cost is what the sweep pays per run, and a period a heartbeat uses must
///  exceed it, so the arm reads it at two sizes.
fn sweep_over(fixture: &Fixture, count: usize, seed: &str) {
    for index in 0..count {
        fixture.make_a_clone_asking(format!("sweep-{seed}-{index}").as_str(), format!("{seed}{index:02}").as_str());
    }
    let run: super::zfs_sweep_guest::Run = sweep();
    assert_eq!(run.status, Some(0), "the {count}-clone sweep reached its end: {}", run.said);
    let attached: usize = (0..count).filter(|index| return mounted_at_path(fixture.slot(format!("sweep-{seed}-{index}").as_str()).as_path())).count();
    println!("STEP22 sweep clones={count} ms={} attached={attached}", run.elapsed.as_millis());
    assert_eq!(attached, count, "the sweep attached every clone it was given");
    for index in 0..count {
        unmount_quiet(fixture.slot(format!("sweep-{seed}-{index}").as_str()).as_path());
        fixture.forget(format!("sweep-{seed}-{index}").as_str(), format!("{seed}{index:02}").as_str());
    }
}

/// What:
///  The unmount's own wait: how long `umount` takes with no holder (the floor),
///  how long it refuses with a holder (the path a deadline must not paper over),
///  and how long the holder takes to leave after its signal, which is the
///  forced arm's `T_wait`.
/// Why:
///  Step 9's unmount deadline must cover the verified unmount,
///  and step 17's `T_wait` value is the time a holder really takes to leave.
fn unmount_arm(fixture: &Fixture) {
    let mut plain: Vec<u128> = Vec::new();
    let mut busy: Vec<u128> = Vec::new();
    let mut leaving: Vec<u128> = Vec::new();
    for round in 0..ROUNDS {
        let name: String = format!("unmount-{round}");
        fixture.make_a_clone_asking(name.as_str(), format!("dd{round:02}").as_str());
        let attached: super::zfs_sweep_guest::Run = sweep();
        assert_eq!(attached.status, Some(0), "round {round}: the sweep attached the clone: {}", attached.said);
        let slot: PathBuf = fixture.slot(name.as_str());
        assert!(mounted_at_path(slot.as_path()), "round {round}: the clone stood at its slot");
        let started: Instant = Instant::now();
        let unmounted: (bool, String, String) = fixture.zfs(&["unmount", slot.to_str().expect("a text path")]);
        plain.push(started.elapsed().as_millis());
        assert!(unmounted.0, "round {round}: the plain unmount was accepted: {}", unmounted.2);
        assert!(!mounted_at_path(slot.as_path()), "round {round}: the plain unmount took the mount down");
        fixture.forget(name.as_str(), format!("dd{round:02}").as_str());
    }
    for round in 0..ROUNDS {
        let name: String = format!("held-{round}");
        fixture.make_a_clone_asking(name.as_str(), format!("ab{round:02}").as_str());
        let attached: super::zfs_sweep_guest::Run = sweep();
        assert_eq!(attached.status, Some(0), "round {round}: the sweep attached the held clone: {}", attached.said);
        let slot: PathBuf = fixture.slot(name.as_str());
        assert!(mounted_at_path(slot.as_path()), "round {round}: the held clone stood at its slot");
        let mut holder: Child = Command::new("/bin/sleep")
            .arg("600")
            .current_dir(slot.as_path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the holder starts");
        std::thread::sleep(Duration::from_millis(200));
        let refused: Instant = Instant::now();
        let while_held: Output = Command::new("/usr/bin/umount").arg(slot.to_str().expect("a text path")).output().expect("the busy unmount starts");
        busy.push(refused.elapsed().as_millis());
        assert!(!while_held.status.success(), "round {round}: a working directory inside the mount refuses the unmount");
        let signalled: Instant = Instant::now();
        let _signal: Output = Command::new("/usr/bin/kill").args(["--signal", "TERM", holder.id().to_string().as_str()]).output().expect("the signal starts");
        let waited: std::io::Result<std::process::ExitStatus> = holder.wait();
        leaving.push(signalled.elapsed().as_millis());
        assert!(waited.is_ok(), "round {round}: the holder ended after its signal");
        unmount_quiet(slot.as_path());
        assert!(!mounted_at_path(slot.as_path()), "round {round}: the unmount after the holder ended was accepted");
        fixture.forget(name.as_str(), format!("ab{round:02}").as_str());
    }
    println!("STEP22 unmount holder=none ms={plain:?} mid={}", middle(&plain));
    println!("STEP22 unmount holder=present refused-ms={busy:?} mid={}", middle(&busy));
    println!("STEP22 forced-arm t-wait-ms={leaving:?} mid={}", middle(&leaving));

    // The holder that ignores the polite signal: the arm reads that a `SIGKILL`
    // is what ends it, which is why `T_wait` is a wait and not a promise.
    let name: &str = "unmount-stubborn";
    fixture.make_a_clone_asking(name, "ee");
    let attached: super::zfs_sweep_guest::Run = sweep();
    assert_eq!(attached.status, Some(0), "the stubborn arm's sweep attached the clone: {}", attached.said);
    let slot: PathBuf = fixture.slot(name);
    let mut stubborn: Child = Command::new("/bin/sh")
        .args(["-c", "trap '' TERM; exec sleep 600"])
        .current_dir(slot.as_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the stubborn holder starts");
    std::thread::sleep(Duration::from_millis(200));
    let _signal: Output = Command::new("/usr/bin/kill").args(["--signal", "TERM", stubborn.id().to_string().as_str()]).output().expect("the signal starts");
    std::thread::sleep(Duration::from_millis(500));
    let still: Option<std::process::ExitStatus> = stubborn.try_wait().expect("the stubborn holder is asked how it is");
    let killed: Instant = Instant::now();
    let _kill: Output = Command::new("/usr/bin/kill").args(["--signal", "KILL", stubborn.id().to_string().as_str()]).output().expect("the kill starts");
    let _ended: std::io::Result<std::process::ExitStatus> = stubborn.wait();
    println!("STEP22 forced-arm stubborn-alive-after-term={} kill-ms={}", still.is_none(), killed.elapsed().as_millis());
    unmount_quiet(slot.as_path());
    fixture.forget(name, "ee");
}

/// What:
///  `T_stop`: how long a transient unit takes to leave its cgroup after its
///  manager is asked to stop it.
/// Why:
///  The forced arm gives a locked unit `T_stop` to leave before it names it,
///  and the design has no provisional value at all.
fn stop_wait_arm() {
    let mut stops: Vec<u128> = Vec::new();
    for round in 0..ROUNDS {
        let unit: String = format!("step22-holder-{round}");
        let started: Output = Command::new("/usr/bin/systemd-run")
            .args(["--unit", unit.as_str(), "--property", "Type=exec", "/bin/sleep", "600"])
            .output()
            .expect("systemd-run starts");
        assert!(started.status.success(), "round {round}: the unit started: {}", String::from_utf8_lossy(started.stderr.as_slice()));
        std::thread::sleep(Duration::from_millis(200));
        let asked: Instant = Instant::now();
        let _stop: Output = Command::new("/usr/bin/systemctl").args(["stop", unit.as_str()]).output().expect("the stop starts");
        let cgroup: PathBuf = PathBuf::from(format!("/sys/fs/cgroup/system.slice/{unit}.service"));
        while asked.elapsed() < STOP_WAIT_BOUND && cgroup.exists() {
            std::thread::sleep(POLL);
        }
        stops.push(asked.elapsed().as_millis());
        println!("STEP22 forced-arm unit={unit} gone={} stop-ms={}", !cgroup.exists(), asked.elapsed().as_millis());
        let _reset: Output = Command::new("/usr/bin/systemctl").args(["reset-failed", unit.as_str()]).output().expect("the reset starts");
    }
    println!("STEP22 forced-arm t-stop-ms={stops:?} mid={}", middle(&stops));
}

/// What:
///  What expiration, removal and the guard's current scope free:
///  the checkout dataset's own bytes, what its snapshot pins, what the clone
///  holds, and what destroying first the clone and then the snapshot gives back.
/// Why:
///  The design asks what expiration,
///  removal and the current guard can free,
///  and the guard prunes only `<dataset>@autosnap_*` of its one configured
///  dataset,
///  so a descendant's hourly and a clone's pinned origin are outside its reach
///  and their measured sizes say what that leaves.
fn retention_arm(fixture: &Fixture) {
    let name: &str = "retention";
    fixture.make_a_clone_asking(name, "ff");
    let attached: super::zfs_sweep_guest::Run = sweep();
    assert_eq!(attached.status, Some(0), "the retention arm's sweep attached the clone: {}", attached.said);
    let checkout: String = fixture.checkout.clone();
    let clone: String = fixture.clone(name);
    let used_before: u64 = fixture.bytes("used", checkout.as_str());
    println!("STEP22 retention checkout-used={used_before} clone-used={}", fixture.bytes("used", clone.as_str()));

    // A build-like output the checkout does not track, which an hourly snapshot
    // would pin, written into an ignored folder.
    let output_folder: PathBuf = fixture.root.join(".claude").join("build-output");
    std::fs::create_dir_all(output_folder.as_path()).expect("the build output's folder");
    let block: Vec<u8> = vec![b'x'; 1024 * 1024];
    for index in 0..BUILD_OUTPUT_MIB {
        std::fs::write(output_folder.join(format!("block-{index}")), block.as_slice()).expect("one write of the build-like output");
    }
    std::thread::sleep(Duration::from_millis(500));
    let written: u64 = fixture.bytes("usedbydataset", checkout.as_str());
    let hourly: String = format!("{checkout}@autosnap_2026-11-25_00:00:00_hourly");
    fixture.zfs_ok(&["snapshot", hourly.as_str()]);
    let with_hourly: u64 = fixture.bytes("usedbysnapshots", checkout.as_str());
    println!("STEP22 retention checkout-written={written} after-hourly-snapshot-pinned={with_hourly}");

    // Removal frees the clone; expiry frees the hourly; the guard could free
    // only a snapshot named like this one on its own configured dataset.
    unmount_quiet(fixture.slot(name).as_path());
    fixture.zfs_ok(&["destroy", "-f", clone.as_str()]);
    let after_removal: u64 = fixture.bytes("used", checkout.as_str());
    fixture.zfs_ok(&["destroy", "-f", hourly.as_str()]);
    let after_expiry: u64 = fixture.bytes("used", checkout.as_str());
    std::fs::remove_dir_all(output_folder.as_path()).expect("the arm takes its own build output away");
    let after_output: u64 = fixture.bytes("used", checkout.as_str());
    println!(
        "STEP22 retention after-removal={after_removal} after-expiry={after_expiry} after-output-removed={after_output} freed-by-removal={} freed-by-expiry={}",
        used_before.saturating_sub(after_removal),
        with_hourly
    );
    fixture.forget(name, "ff");
}

/// What:
///  The largest snapshot-destroy request the binding sends, in names and in
///  bytes of encoded input, at three sizes.
/// Why:
///  The design's split needs the largest encoded input one request may carry,
///  and this arm reads it as the request's own bytes rather than estimating
///  it.
fn request_budget_arm(fixture: &Fixture) {
    let device: ZfsDevice = ZfsDevice::open().expect("the guest permits the device");
    let dataset: String = format!("{}/step22-batch", fixture.pool);
    let _destroyed: (bool, String, String) = fixture.zfs(&["destroy", "-r", "-f", dataset.as_str()]);
    fixture.zfs_ok(&["create", "-o", "mountpoint=none", dataset.as_str()]);
    for count in REQUEST_NAMES {
        let names: Vec<String> = (0..count).map(|index| return format!("{dataset}@step22-{index:05}")).collect();
        for name in &names {
            fixture.zfs_ok(&["snapshot", name.as_str()]);
        }
        let borrowed: Vec<&str> = names.iter().map(|name| return name.as_str()).collect();
        let request: Request = destroy_snaps(fixture.pool.as_str(), borrowed.as_slice()).expect("the request encodes");
        let input_bytes: usize = request.input_bytes().len();
        let started: Instant = Instant::now();
        let answer: Result<Option<NvList>, DeviceError> = device.destroy_snaps(fixture.pool.as_str(), borrowed.as_slice());
        let elapsed: Duration = started.elapsed();
        let outcome: String = match &answer {
            Ok(_answer) => String::from("accepted"),
            Err(error) => format!("refused: {error}"),
        };
        let (_ok, remaining, _error): (bool, String, String) = fixture.zfs(&["list", "-H", "-o", "name", "-t", "snapshot", "-r", dataset.as_str()]);
        println!(
            "STEP22 request-budget names={count} input-bytes={input_bytes} outcome={outcome} ms={} snapshots-left={}",
            elapsed.as_millis(),
            remaining.lines().filter(|line| return !line.trim().is_empty()).count()
        );
    }
    let _destroyed: (bool, String, String) = fixture.zfs(&["destroy", "-r", "-f", dataset.as_str()]);
}

/// The arm's whole run, in the order a reader reads the record.
#[test]
fn the_step_22_guest_measurements() {
    if !guest() {
        return;
    }
    let fixture: Fixture = Fixture::prepare();
    println!("STEP22 pool={} checkout={} container={}", fixture.pool, fixture.checkout, fixture.container);
    println!("STEP22 workload files={} bytes={}", count_files(fixture.root.join("src").as_path()), tree_bytes(fixture.root.join("src").as_path()));
    add_rounds(&fixture);
    sweep_cost_arm(&fixture);
    unmount_arm(&fixture);
    stop_wait_arm();
    retention_arm(&fixture);
    request_budget_arm(&fixture);
    println!("STEP22 done");
}
