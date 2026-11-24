//! What:
//!  Plan step 10's arms in the throwaway guest:
//!  the whole register-first add from the claimed slot to a verified fill,
//!  a second add at the filled slot,
//!  a kill at each phase and the resume after it,
//!  what each kill leaves,
//!  the pool's full answer at the snapshot,
//!  and the two arms a boot and a power cut split in half:
//!  T27's record read after the guest rebooted and after its dataset was
//!  exported and imported,
//!  and T28's flushed write read back after the guest lost power.
//! Why:
//!  The route writes a real registration, a real snapshot, a real clone and a
//!  real mount, so no control on a person's machine may run it:
//!  these arms run only where `CLI_GIT_ZFS_ADD_GUEST` names a disposable guest
//!  (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`, the
//!  `zfs-add-*` steps, on OpenZFS 2.4.4 in an `mvm` Ubuntu guest).
//!  A run without the variable prints `SKIP` and returns, and the driver's
//!  suite step refuses to count that as a pass.
//!  The arm mounts the clone with the kernel's own `mount` as root rather than
//!  through the handler's attach: the attach is step 4's arm, already green,
//!  and this step's subject is the order from the claim to the fill. The add's
//!  own wait and verification read the mount the table shows, whichever
//!  program made it.
//!  T27 and T28 each span a reboot or a power cut,
//!  so each is split into a `before` half and an `after` half,
//!  which `CLI_GIT_ZFS_ADD_PHASE` names.
//!  The `before` half writes what the `after` half reads,
//!  and the driver's own steps put the cut between them,
//!  at `zfs-add-boot-*` and at `zfs-add-power-*`.
//!  A run of the suite without that variable prints one `UNRUN` line per split arm and returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const add = runAdd(slot); await mountWhenTheCloneAppears(slot); assertFilled(add);
//! ```

#![cfg(target_os = "linux")]

use git_policy_cli::worktree_clone_binding_record::{BindingRecord, FolderIdentity, Phase, parse_binding};
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness::{ReadyState, ReadinessRecord, SweepState};
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use std::io::Write as _;
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::os::unix::process::ExitStatusExt as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The variable the guest driver sets.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_ADD_GUEST";

/// The checkout root the driver mounted the design's dataset at.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_ADD_ROOT";

/// The pool the driver made.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_ADD_POOL";

/// The OpenZFS userspace the arm's own reads use.
const SBIN_VARIABLE: &str = "CLI_GIT_ZFS_ADD_SBIN";

/// The stop switch the kills use.
const STOP_VARIABLE: &str = "CLI_GIT_TEST_ONLY_ZFS_ADD_STOP_AFTER";

/// The checkout dataset's leaf name the design fixes.
const CHECKOUT_LEAF: &str = "monochromatic-git-checkout";

/// The container's leaf name the design fixes.
const CONTAINER_LEAF: &str = "monochromatic-git-worktree-clone-datasets";

/// The wrapper program the driver built.
const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// How long the arm waits for one phase of the add or for a mount.
const WAIT: Duration = Duration::from_secs(60);

/// The variable naming which half of a split arm this run is.
const PHASE_VARIABLE: &str = "CLI_GIT_ZFS_ADD_PHASE";

/// The half that writes the record the next half reads.
const BEFORE: &str = "before";

/// The half that reads that record back on the other side of the boot or the cut.
const AFTER: &str = "after";

/// The slot the boot arm fills.
/// The red control replaces the folder of `BOOT_PLANT_SLOT` instead.
const BOOT_SLOT: &str = "boot";

/// The slot whose folder the red control replaces,
/// so that the record cannot match it.
const BOOT_PLANT_SLOT: &str = "boot-plant";

/// The slot the power-loss arm fills.
const POWER_SLOT: &str = "power";

/// The variable that makes the power-loss arm's before half cut the guest's power,
/// once its own work is done.
const CUT_VARIABLE: &str = "CLI_GIT_ZFS_ADD_CUT_POWER";

/// The OpenZFS release the guest's own module must answer,
/// as the driver's recipe pins it (`OPENZFS_RELEASE`).
const OPENZFS_RELEASE: &str = "2.4.4-1";

/// What the before halves keep on the guest's own root disk,
/// which a reboot and a cut both keep.
const KEPT_FOLDER: &str = "/root/zfs-add-kept";

/// The pause between two looks at the pool while the add runs.
const POLL: Duration = Duration::from_millis(100);

/// Whether this run is inside the guest,
/// or a skip line on every other machine.
fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP binary_zfs_add_guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest");
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname").map(|text| return text.trim().to_string()).unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// One required fixture variable.
fn required(variable: &str) -> String {
    return std::env::var(variable).unwrap_or_else(|_error| panic!("{variable} is unset inside the guest, so this arm has no fixture to run against"));
}

/// The fixture the arm drives.
struct Fixture {
    /// The checkout root.
    root: PathBuf,
    /// The pool's name.
    pool: String,
    /// The OpenZFS userspace.
    sbin: String,
}

impl Fixture {
    /// The fixture the driver prepared,
    /// with its repository and its handler readiness written.
    fn prepare() -> Fixture {
        let fixture: Fixture = Fixture::bare();
        fixture.adopt_the_guest();
        return fixture;
    }

    /// What:
    ///  Write what the guest must hold again after it rebooted or lost power:
    ///  the repository,
    ///  its worktrees folder,
    ///  the handler's readiness and the wrapper's link.
    /// Why:
    ///  The readiness report lives in `/run`,
    ///  which a boot empties,
    ///  and a half after a cut must publish this boot's own identifier before any add reads it.
    fn adopt_the_guest(&self) {
        self.make_the_repository();
        self.make_the_worktrees_folder();
        self.publish_the_readiness();
        self.make_the_wrapper_link();
    }

    /// The fixture's three names and nothing else,
    /// for the halves that must import the pool before they can read or write anything on it.
    fn bare() -> Fixture {
        let root: PathBuf = PathBuf::from(required(ROOT_VARIABLE));
        let pool: String = required(POOL_VARIABLE);
        let sbin: String = required(SBIN_VARIABLE);
        return Fixture { root, pool, sbin };
    }

    /// The checkout's dataset name.
    fn checkout(&self) -> String {
        return format!("{}/ROOT/home/{CHECKOUT_LEAF}", self.pool);
    }

    /// The clone container's dataset name.
    fn container(&self) -> String {
        return format!("{}/ROOT/home/{CONTAINER_LEAF}", self.pool);
    }

    /// The common Git directory.
    fn common(&self) -> PathBuf {
        return self.root.join(".git");
    }

    /// The records' home.
    fn home(&self) -> PathBuf {
        return self.common().join("monochromatic-git-policy-cli").join("worktree-clones").join("v1");
    }

    /// The slot at one name.
    fn slot(&self, name: &str) -> PathBuf {
        return self.root.join(".claude").join("worktrees").join(name);
    }

    /// The clone dataset at one name.
    fn clone(&self, name: &str) -> String {
        return format!("{}/{}", self.container(), name);
    }

    /// A repository with two files, one commit, and the trash place ignored.
    /// The arms share one checkout, so the second arm finds the first's commit.
    fn make_the_repository(&self) {
        if self.root.join(".git").is_dir() {
            return;
        }
        std::fs::write(self.root.join(".gitignore"), b".claude/\n").expect("the ignore rule");
        std::fs::write(self.root.join("plain.txt"), b"plain\n").expect("the tracked file");
        std::fs::write(self.root.join("second.txt"), b"second\n").expect("the second file");
        // A file written in the same second as the index is a racy entry, which
        // the clean check refuses to vouch for; the fixture waits one second so
        // the index's record proves the bytes.
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.git(&["init", "--quiet", "--initial-branch=main"]);
        self.git(&["add", "."]);
        self.git(&["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "one"]);
    }

    /// The folder the route's slots live in.
    fn make_the_worktrees_folder(&self) {
        let parent: PathBuf = self.root.join(".claude").join("worktrees");
        std::fs::create_dir_all(parent.as_path()).expect("the worktrees folder");
        std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the folder's mode");
    }

    /// The readiness report the add's preflight requires,
    /// written as the boot unit would.
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

    /// The folder the wrapper is linked into, outside the checkout so the
    /// source stays clean.
    fn bin_folder(&self) -> PathBuf {
        return PathBuf::from("/root/zfs-add-bin");
    }

    /// The home folder the wrappers and Git run with, outside the checkout.
    fn home_folder(&self) -> PathBuf {
        return PathBuf::from("/root/zfs-add-home");
    }

    /// The link the arm starts as `git`.
    fn make_the_wrapper_link(&self) {
        let bin: PathBuf = self.bin_folder();
        std::fs::create_dir_all(bin.as_path()).expect("the bin folder");
        std::fs::create_dir_all(self.home_folder().as_path()).expect("the home folder");
        let link: PathBuf = bin.join("git");
        let _stale: std::io::Result<()> = std::fs::remove_file(link.as_path());
        std::os::unix::fs::symlink(WRAPPER, link.as_path()).expect("the wrapper's git link");
    }

    /// Run one `zfs` command and answer its output.
    fn zfs(&self, arguments: &[&str]) -> Output {
        return self.openzfs("zfs", arguments);
    }

    /// Run one OpenZFS program and answer its output.
    fn openzfs(&self, program: &str, arguments: &[&str]) -> Output {
        return Command::new(program)
            .args(arguments)
            .env("PATH", format!("{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin", self.sbin))
            .env("LD_LIBRARY_PATH", format!("{}/lib", self.sbin.trim_end_matches("/sbin")))
            .output()
            .unwrap_or_else(|error| panic!("{program} {arguments:?} could not start: {error}"));
    }

    /// Run one `zfs` command and require success.
    fn zfs_ok(&self, arguments: &[&str]) {
        let output: Output = self.zfs(arguments);
        assert!(output.status.success(), "zfs {arguments:?} was refused: {}", String::from_utf8_lossy(output.stderr.as_slice()));
    }

    /// Run one `zpool` command and require success.
    fn zpool_ok(&self, arguments: &[&str]) {
        let output: Output = self.openzfs("zpool", arguments);
        assert!(output.status.success(), "zpool {arguments:?} was refused: {}", String::from_utf8_lossy(output.stderr.as_slice()));
    }

    /// One `zfs get -H -o value <property>` answer,
    /// trimmed.
    fn zfs_value(&self, property: &str, dataset: &str) -> String {
        let output: Output = self.zfs(&["get", "-H", "-o", "value", property, dataset]);
        assert!(
            output.status.success(),
            "zfs get {property} {dataset} was refused: {}",
            String::from_utf8_lossy(output.stderr.as_slice())
        );
        return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    }

    /// The live GUID of one dataset,
    /// as the record writes it.
    fn dataset_guid(&self, dataset: &str) -> String {
        return self.zfs_value("guid", dataset);
    }

    /// The device number the kernel assigned the checkout's mount,
    /// which `F16` says is anonymous and need not come back,
    /// and which the record therefore never holds.
    fn device_of_the_slot(&self, name: &str) -> u64 {
        return std::fs::metadata(self.slot(name).as_path()).expect("the slot folder's metadata").dev();
    }

    /// Whether a dataset answers the pool.
    fn dataset_stands(&self, dataset: &str) -> bool {
        return self.zfs(&["list", "-H", "-o", "name", dataset]).status.success();
    }

    /// Run one real Git command in the checkout and require success.
    fn git(&self, arguments: &[&str]) {
        let output: Output = Command::new("/usr/bin/git")
            .current_dir(self.root.as_path())
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", self.home_folder())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .args(arguments)
            .output()
            .unwrap_or_else(|error| panic!("git {arguments:?} could not start: {error}"));
        assert!(
            output.status.success(),
            "git {arguments:?} was refused ({:?}): stdout `{}`, stderr `{}`",
            output.status.code(),
            String::from_utf8_lossy(output.stdout.as_slice()),
            String::from_utf8_lossy(output.stderr.as_slice())
        );
    }

    /// The binding one slot holds,
    /// or nothing.
    fn binding(&self, name: &str) -> Option<BindingRecord> {
        let path: PathBuf = self.home().join(format!("{name}.binding"));
        let text: String = std::fs::read_to_string(path.as_path()).ok()?;
        return Some(parse_binding(text.as_bytes()).unwrap_or_else(|refusal| panic!("the binding of {name} is refused: {refusal:?}")));
    }

    /// One add's command, with the harness's hermetic environment.
    fn add_command(&self, name: &str, stop_after: Option<&str>) -> Command {
        let mut command: Command = Command::new(self.bin_folder().join("git"));
        command
            .args(["worktree", "add", self.slot(name).to_str().expect("a text path")])
            .current_dir(self.root.as_path())
            .env_clear()
            .env("PATH", format!("{}:/usr/bin:/bin", self.bin_folder().display()))
            .env("HOME", self.home_folder())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .env_remove(STOP_VARIABLE)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(step) = stop_after {
            command.env(STOP_VARIABLE, step);
        }
        return command;
    }

    /// Run one add to its end,
    /// mounting the clone when it appears.
    fn run_add(&self, name: &str) -> Output {
        let mut child: Child = self.add_command(name, None).spawn().expect("the add starts");
        self.mount_when_the_clone_appears(name, &mut child);
        return child.wait_with_output().expect("the add ends");
    }

    /// While the add runs, mount the clone at the slot as root when it answers.
    fn mount_when_the_clone_appears(&self, name: &str, child: &mut Child) {
        let clone: String = self.clone(name);
        let slot: PathBuf = self.slot(name);
        let started: Instant = Instant::now();
        while started.elapsed() < WAIT {
            if let Some(_status) = child.try_wait().expect("the add is asked how it is") {
                return;
            }
            if self.dataset_stands(clone.as_str()) && self.intent_says_mount(clone.as_str()) && slot.is_dir() && !self.mounted_at(slot.as_path()) {
                let mounted: Output = Command::new("/usr/bin/mount")
                    .args(["-t", "zfs", "-o", "nosuid,nodev", clone.as_str(), slot.to_str().expect("a text path")])
                    .output()
                    .expect("mount starts");
                let _ = mounted;
            }
            std::thread::sleep(POLL);
        }
    }

    /// Whether the add wrote the clone's durable mount request.
    fn intent_says_mount(&self, clone: &str) -> bool {
        let output: Output = self.zfs(&["get", "-H", "-o", "value", "org.monochromatic:git-worktree-clone-mount-desired-state", clone]);
        return String::from_utf8_lossy(output.stdout.as_slice()).contains(":mount");
    }

    /// Whether a mount stands at one path in this process's table.
    fn mounted_at(&self, path: &Path) -> bool {
        let table: String = std::fs::read_to_string("/proc/self/mountinfo").expect("the mount table");
        let wanted: String = path.display().to_string();
        return table.lines().any(|line| return line.split(' ').nth(4) == Some(wanted.as_str()));
    }

    /// Unmount the slot, destroy the clone and the origin snapshot, remove the
    /// registration and the records, so the next arm starts from nothing.
    fn clear_the_slot(&self, name: &str) {
        let slot: PathBuf = self.slot(name);
        if self.mounted_at(slot.as_path()) {
            let _unmounted = Command::new("/usr/bin/umount").arg(slot.as_path()).status();
        }
        let clone: String = self.clone(name);
        let _destroyed: Output = self.zfs(&["destroy", clone.as_str()]);
        let snapshots: Output = self.zfs(&["list", "-H", "-o", "name", "-t", "snapshot", self.checkout().as_str()]);
        for snapshot in String::from_utf8_lossy(snapshots.stdout.as_slice()).lines() {
            if snapshot.contains("git-worktree-clone-origin-") {
                let _gone: Output = self.zfs(&["destroy", snapshot]);
            }
        }
        let _removed = Command::new("/usr/bin/git")
            .current_dir(self.root.as_path())
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", self.home_folder())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args(["worktree", "remove", "--force", "--force", slot.to_str().expect("a text path")])
            .status();
        let records: PathBuf = self.common().join("monochromatic-git-policy-cli").join("worktree-clones");
        let _cleared: std::io::Result<()> = std::fs::remove_dir_all(records.as_path());
    }

    /// What:
    ///  Clear one slot's own mount,
    ///  its clone,
    ///  its registration and its records,
    ///  so a split arm run twice starts from nothing.
    /// Why:
    ///  A split arm's `before` half is run again for the red control,
    ///  and the filled slot it left would refuse the second add,
    ///  while another slot's record and clone must stand.
    fn clear_one_slot(&self, name: &str) {
        self.unmount(name);
        let clone: String = self.clone(name);
        let _destroyed: Output = self.zfs(&["destroy", clone.as_str()]);
        let slot: PathBuf = self.slot(name);
        let _removed = Command::new("/usr/bin/git")
            .current_dir(self.root.as_path())
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", self.home_folder())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args(["worktree", "remove", "--force", "--force", slot.to_str().expect("a text path")])
            .status();
        for form in ["binding", "origin-inventory", "clone-inventory"] {
            let _gone: std::io::Result<()> = std::fs::remove_file(self.home().join(format!("{name}.{form}")));
        }
    }

    /// Unmount one slot's clone,
    /// so the folder below it answers again,
    /// and answer whether a mount was there.
    fn unmount(&self, name: &str) -> bool {
        let slot: PathBuf = self.slot(name);
        if !self.mounted_at(slot.as_path()) {
            return false;
        }
        let unmounted: Output = Command::new("/usr/bin/umount").arg(slot.as_path()).output().expect("the unmount runs");
        assert!(unmounted.status.success(), "the clone at {} cannot be unmounted: {}", slot.display(), String::from_utf8_lossy(unmounted.stderr.as_slice()));
        return true;
    }

    /// What:
    ///  Bring the pool back the way the driver's pool step left it:
    ///  the module,
    ///  the import,
    ///  the datasets' keys,
    ///  the checkout's mount and a private parent.
    /// Why:
    ///  The pool is made with `cachefile=none`,
    ///  so nothing imports it at boot,
    ///  and a dataset with no key loaded mounts nothing.
    ///  The import is skipped when the pool answers,
    ///  so the half after a cut may run against a pool another half already imported.
    fn import_the_pool(&self) {
        self.load_the_module();
        let listed: Output = self.openzfs("zpool", &["list", "-H", "-o", "name", self.pool.as_str()]);
        if !listed.status.success() {
            let folder: &Path = self.root.parent().expect("the folder the pool's image sits beside the checkout's mountpoint");
            // `-f` because a guest that did not export the pool before it went down leaves it importing only when it is
            // forced, and `-N` because the checkout's key is not loaded yet, so the mount is made below instead; the pool
            // is this guest's own, so nothing else can hold it.
            self.zpool_ok(&["import", "-f", "-N", "-d", folder.to_str().expect("a text path"), self.pool.as_str()]);
        }
        if self.zfs_value("keystatus", self.checkout().as_str()) != "available" {
            self.zfs_ok(&["load-key", "-a"]);
        }
        if self.zfs_value("mounted", self.checkout().as_str()) != "yes" {
            self.zfs_ok(&["mount", self.checkout().as_str()]);
        }
        self.make_the_checkout_private();
    }

    /// What:
    ///  Make the guest's own OpenZFS module the loaded one.
    /// Why:
    ///  A boot loads nothing,
    ///  and the first `zpool` or `zfs` command loads the distribution's own module through modprobe,
    ///  which answers 2.2.2 here and which the wrapper's compatibility gate refuses,
    ///  because only the module the driver built carries the type information at `/sys/kernel/btf/zfs`.
    ///  So a loaded module whose release is not the pinned one is removed and the driver's own two are inserted.
    fn load_the_module(&self) {
        let loaded: String = std::fs::read_to_string("/sys/module/zfs/version").unwrap_or_default().trim().to_string();
        if loaded == OPENZFS_RELEASE {
            return;
        }
        if !loaded.is_empty() {
            for module in ["zfs", "spl"] {
                let _removed: Output = Command::new("/usr/sbin/modprobe")
                    .args(["--remove", module])
                    .output()
                    .unwrap_or_else(|error| panic!("modprobe --remove {module} could not start: {error}"));
            }
        }
        let kernel: String = std::fs::read_to_string("/proc/sys/kernel/osrelease").expect("this kernel's release").trim().to_string();
        for module in ["spl", "zfs"] {
            let path: String = format!("/lib/modules/{kernel}/extra/{module}.ko");
            let inserted: Output = Command::new("/usr/sbin/insmod").arg(path.as_str()).output().unwrap_or_else(|error| panic!("insmod {path} could not start: {error}"));
            assert!(inserted.status.success(), "insmod {path} was refused: {}", String::from_utf8_lossy(inserted.stderr.as_slice()));
        }
        let now: String = std::fs::read_to_string("/sys/module/zfs/version").expect("the module's release").trim().to_string();
        assert_eq!(now, OPENZFS_RELEASE, "the guest runs the OpenZFS module the driver built");
    }

    /// Make the checkout's mount share nothing,
    /// as the driver's pool step did,
    /// because a clone mounted below it must not propagate to a peer.
    fn make_the_checkout_private(&self) {
        let made: Output = Command::new("/usr/bin/mount")
            .args(["--make-rprivate", self.root.to_str().expect("a text path")])
            .output()
            .expect("the mount runs");
        assert!(made.status.success(), "the checkout's mount cannot be made private: {}", String::from_utf8_lossy(made.stderr.as_slice()));
    }

    /// Unmount everything mounted below the checkout's mountpoint,
    /// which an export refuses while it stands.
    fn unmount_below_the_checkout(&self) {
        let prefix: String = format!("{}/", self.root.display());
        let table: String = std::fs::read_to_string("/proc/self/mountinfo").expect("the mount table");
        for line in table.lines() {
            let Some(point) = line.split(' ').nth(4) else {
                continue;
            };
            let path: String = point.replace("\\040", " ");
            if path.starts_with(prefix.as_str()) {
                let _unmounted: Output = Command::new("/usr/bin/umount").arg(path.as_str()).output().expect("the unmount runs");
            }
        }
    }

    /// What:
    ///  Export the pool and import it again,
    ///  which T27's second read needs.
    /// Why:
    ///  An export ends every mount and unloads every key,
    ///  so the import must bring back the keys and the checkout's mount,
    ///  which is exactly the state the record must survive.
    fn export_and_import_the_pool(&self) {
        self.unmount_below_the_checkout();
        self.zpool_ok(&["export", "-f", self.pool.as_str()]);
        self.import_the_pool();
    }

    /// One slot's binding bytes,
    /// or nothing.
    fn binding_bytes(&self, name: &str) -> Option<Vec<u8>> {
        return std::fs::read(self.home().join(format!("{name}.binding"))).ok();
    }

    /// What:
    ///  Keep one file on the guest's own root disk,
    ///  flushed,
    ///  for the half on the other side of a reboot or a cut.
    /// Why:
    ///  The kept copy is what the after half compares the pool's record with,
    ///  so it must survive the same loss,
    ///  and a write that is not flushed there would be lost by the very cut this arm makes.
    fn keep(&self, name: &str, bytes: &[u8]) {
        let folder: PathBuf = PathBuf::from(KEPT_FOLDER);
        std::fs::create_dir_all(folder.as_path()).expect("the kept folder");
        let path: PathBuf = folder.join(name);
        let mut file: std::fs::File = std::fs::File::create(path.as_path()).unwrap_or_else(|error| panic!("{} cannot be made: {error}", path.display()));
        file.write_all(bytes).unwrap_or_else(|error| panic!("{} cannot be written: {error}", path.display()));
        file.sync_all().unwrap_or_else(|error| panic!("{} cannot be flushed: {error}", path.display()));
    }

    /// One file kept before the reboot or the cut,
    /// or a panic naming what is missing.
    fn kept(&self, name: &str) -> Vec<u8> {
        let path: PathBuf = PathBuf::from(KEPT_FOLDER).join(name);
        return std::fs::read(path.as_path()).unwrap_or_else(|error| panic!("{} was kept before the reboot or the cut and cannot be read ({error})", path.display()));
    }
}

#[test]
fn the_add_makes_a_verified_clone_and_a_second_add_refuses() {
    if !guest() {
        return;
    }
    let fixture: Fixture = Fixture::prepare();
    let output: Output = fixture.run_add("whole");
    assert!(output.status.success(), "the whole add failed: {}", String::from_utf8_lossy(output.stderr.as_slice()));
    assert!(String::from_utf8_lossy(output.stderr.as_slice()).contains("made the worktree as a ZFS clone"), "the add says what it made");
    let record: BindingRecord = fixture.binding("whole").expect("the add's binding");
    assert_eq!(record.phase, Phase::Filled);
    assert!(record.clone_guid.is_some(), "the clone's identity is recorded");
    assert!(record.origin_snapshot_guid.is_some(), "the snapshot's identity is recorded");
    let slot: PathBuf = fixture.slot("whole");
    assert!(slot.join(".git").is_file(), "the slot holds the saved pointer");
    assert!(fixture.mounted_at(slot.as_path()), "the clone is mounted at the slot");
    assert_eq!(std::fs::read_to_string(slot.join("plain.txt")).expect("the tracked file"), "plain\n");
    let status: Output = Command::new("/usr/bin/git").current_dir(slot.as_path()).args(["status", "--porcelain"]).output().expect("status runs");
    assert_eq!(String::from_utf8_lossy(status.stdout.as_slice()).trim(), "", "the filled worktree is clean");
    let listed: Output = Command::new("/usr/bin/git").current_dir(fixture.root.as_path()).args(["worktree", "list", "--porcelain"]).output().expect("list runs");
    assert!(String::from_utf8_lossy(listed.stdout.as_slice()).contains(slot.display().to_string().as_str()), "Git lists the worktree");
    let trash: PathBuf = slot.join(".claude").join("cli-git-trash");
    assert!(trash.is_dir(), "the copied administration was left in the wrapper's trash");
    // T22: the origin inventory is the checkout's snapshots as `zfs list` reads them.
    let home: PathBuf = fixture.home();
    let origin: git_policy_cli::InventoryRecord = git_policy_cli::read_inventory(home.as_path(), "whole", git_policy_cli::InventoryKind::Origin).expect("the origin inventory reads");
    let snapshots: Output = fixture.zfs(&["list", "-H", "-o", "name", "-t", "snapshot", fixture.checkout().as_str()]);
    let listed_names: Vec<String> = String::from_utf8_lossy(snapshots.stdout.as_slice()).lines().map(|line| return line.rsplit('@').next().unwrap_or("").to_string()).filter(|name| return !name.is_empty()).collect();
    let recorded: Vec<String> = origin.entries.iter().filter(|entry| return entry.kind == git_policy_cli::EntryKind::Snapshot).map(|entry| return entry.name.clone()).collect();
    assert_eq!(recorded.len(), listed_names.len(), "the inventory counts the checkout's snapshots");
    for name in &listed_names {
        assert!(recorded.contains(name), "the inventory records {name}");
    }
    assert_eq!(origin.bounds, record.origin_snapshot.as_str(), "the inventory is bounded by the add's snapshot");
    // T25: prune keeps a locked registration whose underlay pointer is gone.
    let unmounted: Output = Command::new("/usr/bin/umount").arg(slot.as_path()).output().expect("the unmount runs");
    assert!(unmounted.status.success(), "the clone unmounts");
    let _pointer_gone: std::io::Result<()> = std::fs::remove_file(slot.join(".git"));
    let pruned: Output = Command::new("/usr/bin/git").current_dir(fixture.root.as_path()).args(["worktree", "prune", "--verbose"]).output().expect("prune runs");
    assert!(pruned.status.success(), "prune succeeds: {}", String::from_utf8_lossy(pruned.stderr.as_slice()));
    let registration: PathBuf = fixture.common().join("worktrees").join("whole");
    assert!(registration.is_dir(), "the managed lock keeps the registration through prune");
    assert!(fixture.dataset_stands(fixture.clone("whole").as_str()), "the clone stands through prune");
    assert_eq!(fixture.binding("whole").expect("the record reads after prune").phase, Phase::Filled);
    let again: Output = fixture.run_add("whole");
    assert!(!again.status.success(), "a second add at a filled slot is refused");
    assert!(String::from_utf8_lossy(again.stderr.as_slice()).contains("already a whole ZFS clone"), "the refusal says the worktree is whole");
    fixture.clear_the_slot("whole");
}

#[test]
fn a_kill_at_each_phase_leaves_that_state_and_the_next_add_resumes() {
    if !guest() {
        return;
    }
    let fixture: Fixture = Fixture::prepare();
    // Each arm kills the add after one step's write and then runs the same add
    // again, which resumes it. The phase the kill leaves is the last one that
    // step wrote.
    let arms: [(&str, Phase); 9] = [
        ("slot-identity", Phase::Slot),
        ("git-add", Phase::Slot),
        ("registered", Phase::Registered),
        ("clone", Phase::Snapshotted),
        ("clone-inventory", Phase::Cloned),
        ("mount", Phase::Cloned),
        ("index", Phase::Mounted),
        ("trash-entry", Phase::Mounted),
        ("exchange", Phase::Mounted),
    ];
    for (step, left) in arms {
        let name: String = format!("kill-{step}");
        let mut child: Child = fixture.add_command(name.as_str(), Some(step)).spawn().expect("the add starts");
        fixture.mount_when_the_clone_appears(name.as_str(), &mut child);
        let killed: Output = child.wait_with_output().expect("the killed add ends");
        assert_eq!(
            killed.status.signal(),
            Some(libc::SIGKILL),
            "the control's switch killed the add at {step} ({:?}); the add said `{}`",
            killed.status.code(),
            String::from_utf8_lossy(killed.stderr.as_slice())
        );
        let record: BindingRecord = fixture.binding(name.as_str()).unwrap_or_else(|| panic!("the {step} kill left a binding"));
        assert_eq!(record.phase, left, "the {step} kill left the phase the step before it wrote");
        match step {
            "slot-identity" => assert!(fixture.slot(name.as_str()).is_dir(), "the slot stands after the kill"),
            "git-add" => {
                let slot: PathBuf = fixture.slot(name.as_str());
                assert!(slot.join(".git").is_file(), "Git\'s pointer stands and no scan has adopted it");
                assert!(!fixture.dataset_stands(fixture.clone(name.as_str()).as_str()), "no clone was made");
            }
            "registered" => {
                let slot: PathBuf = fixture.slot(name.as_str());
                assert!(slot.join(".git").is_file(), "the registration's pointer stands");
                assert!(!fixture.dataset_stands(fixture.clone(name.as_str()).as_str()), "no clone was made");
            }
            "clone" => assert!(fixture.dataset_stands(fixture.clone(name.as_str()).as_str()), "the clone stands although its identity is not recorded"),
            _later => {
                assert!(fixture.dataset_stands(fixture.clone(name.as_str()).as_str()), "the clone stands");
            }
        }
        let resumed: Output = fixture.run_add(name.as_str());
        assert!(resumed.status.success(), "the resume after {step} failed: {}", String::from_utf8_lossy(resumed.stderr.as_slice()));
        let finished: BindingRecord = fixture.binding(name.as_str()).expect("the resumed add's binding");
        assert_eq!(finished.phase, Phase::Filled, "the resume after {step} filled the worktree");
        assert!(finished.clone_guid.is_some(), "the resume after {step} recorded the clone's identity");
        fixture.clear_the_slot(name.as_str());
    }
}

#[test]
fn a_full_pool_at_the_snapshot_leaves_the_claim_the_slot_and_the_registration() {
    if !guest() {
        return;
    }
    let fixture: Fixture = Fixture::prepare();
    std::fs::write(fixture.root.join("big.bin"), vec![7u8; 1024 * 1024]).expect("the big file");
    std::thread::sleep(std::time::Duration::from_secs(1));
    fixture.git(&["add", "big.bin"]);
    fixture.git(&["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "big"]);
    // The capture is written before the snapshot, so the pool fills at the one
    // moment no other state can tell the test about: the add holds itself after
    // the capture until the control's release file stands.
    // Outside the pool: the release is written while the pool is full.
    let release: PathBuf = PathBuf::from("/root/zfs-hold-release");
    let waiting: PathBuf = release.with_extension("waiting");
    let _stale: std::io::Result<()> = std::fs::remove_file(release.as_path());
    let _stale_waiting: std::io::Result<()> = std::fs::remove_file(waiting.as_path());
    let filler: String = format!("{}/filler", fixture.pool);
    fixture.zfs_ok(&["create", "-o", "mountpoint=/root/zfs-filler", "-o", "compression=off", filler.as_str()]);
    let mut command: Command = fixture.add_command("full", None);
    command.env("CLI_GIT_TEST_ONLY_ZFS_ADD_HOLD_STEP", "capture").env("CLI_GIT_TEST_ONLY_ZFS_ADD_HOLD_UNTIL", release.as_os_str());
    let child: Child = command.spawn().expect("the add starts");
    let started: Instant = Instant::now();
    while !waiting.exists() && started.elapsed() < WAIT {
        std::thread::sleep(POLL);
    }
    assert!(waiting.exists(), "the add reached the capture before the bound");
    // The filler takes the pool's free space with zeros that do not compress.
    let _filled: Output = Command::new("/usr/bin/dd").args(["if=/dev/zero", "of=/root/zfs-filler/fill", "bs=1M"]).output().expect("dd fills the pool");
    std::fs::write(release.as_path(), b"release\n").expect("the control releases the add");
    let output: Output = child.wait_with_output().expect("the add ends");
    // The pool is freed before any assertion, so a failed arm never leaves the
    // next one with no space.
    let _gone: Output = fixture.zfs(&["destroy", "-r", "-f", filler.as_str()]);
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert!(!output.status.success(), "the add with a full pool is refused: {said}");
    assert!(said.contains("ENOSPC") || said.contains("No space left"), "the refusal names the pool's space: {said}");
    let record: BindingRecord = fixture.binding("full").expect("the refused add's binding");
    assert_eq!(record.phase, Phase::Registered, "the snapshot's refusal leaves the registration phase");
    assert!(fixture.slot("full").join(".git").is_file(), "the slot and its pointer stand");
    assert!(!fixture.dataset_stands(fixture.clone("full").as_str()), "no clone was made");
    let _released: std::io::Result<()> = std::fs::remove_file(release.as_path());
    let resumed: Output = fixture.run_add("full");
    assert!(resumed.status.success(), "the resume after the full pool failed: {}", String::from_utf8_lossy(resumed.stderr.as_slice()));
    assert_eq!(fixture.binding("full").expect("the resumed binding").phase, Phase::Filled);
    fixture.clear_the_slot("full");
    let _big_gone: std::io::Result<()> = std::fs::remove_file(fixture.root.join("big.bin"));
    fixture.git(&["add", "-A"]);
    fixture.git(&["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "big gone"]);
}

//region The two arms a boot and a power cut split in half

/// This boot's identifier,
/// which a reboot and a power cut both change.
fn this_boot() -> String {
    return std::fs::read_to_string("/proc/sys/kernel/random/boot_id").expect("this boot's identifier").trim().to_string();
}

/// What:
///  The half of a split arm this run was asked for,
///  or nothing with one line saying the arm runs in its own steps.
/// Why:
///  A reboot and a power cut sit between the two halves of one arm,
///  so the suite's own run can hold neither,
///  and the driver's steps name the half they run.
fn requested_half(arm: &str) -> Option<String> {
    let Ok(half) = std::env::var(PHASE_VARIABLE) else {
        println!("UNRUN {arm} is split across a reboot or a power cut and runs only in its own driver step ({PHASE_VARIABLE} is unset)");
        return None;
    };
    assert!(half == BEFORE || half == AFTER, "{PHASE_VARIABLE}={half} names no half of {arm}; it takes {BEFORE} or {AFTER}");
    return Some(half);
}

/// What:
///  Read one slot's record back,
///  and compare its bytes and its phase and the identities it keeps with what was kept,
///  and with the live system.
/// Why:
///  The whole point of an identity is that it still names the same object after a boot or an export,
///  so a comparison against a value read in the same breath would prove nothing;
///  the record's bytes are compared with the copy taken before the reboot or the cut.
fn read_the_record_back(fixture: &Fixture, slot: &str, when: &str) -> BindingRecord {
    let kept: Vec<u8> = fixture.kept(format!("{slot}.binding").as_str());
    let path: PathBuf = fixture.home().join(format!("{slot}.binding"));
    let bytes: Vec<u8> = std::fs::read(path.as_path())
        .unwrap_or_else(|error| panic!("{} cannot be read {when} ({error}), so the record this arm wrote before it is gone", path.display()));
    assert_eq!(
        bytes, kept,
        "the record at {slot} differs from the copy this arm kept {when}, so the write the route flushed did not read back"
    );
    let record: BindingRecord = parse_binding(bytes.as_slice()).unwrap_or_else(|refusal| panic!("the record at {slot} is not this grammar's {when}: {refusal:?}"));
    assert_eq!(record.phase, Phase::Filled, "the record at {slot} still records the fill {when}");
    let recorded: FolderIdentity = record.slot_identity.unwrap_or_else(|| panic!("the record at {slot} keeps no slotIdentity= {when}"));
    let live: FolderIdentity = FolderIdentity::of_folder(fixture.slot(slot).as_path()).expect("the slot folder's identity");
    assert_eq!(
        recorded,
        live,
        "the record's slotIdentity= differs from the slot folder's own inode and birth time {when}: recorded {}, live {}",
        recorded.encode(),
        live.encode()
    );
    let guid: String = fixture.dataset_guid(fixture.checkout().as_str());
    assert_eq!(
        record.checkout_dataset_guid.file_name(),
        guid,
        "the record's checkoutDatasetGuid= differs from the live dataset's GUID {when}: recorded {}, live {guid}",
        record.checkout_dataset_guid.file_name()
    );
    println!(
        "MEASUREMENT read-back slot={slot} when=\"{when}\" slot-identity={} checkout-guid={guid} clone-guid={} device={} bytes-match=yes",
        recorded.encode(),
        record.clone_guid.map(|clone| return clone.file_name()).unwrap_or_else(|| return String::from("-")),
        fixture.device_of_the_slot(slot)
    );
    return record;
}

/// What:
///  The half before the reboot:
///  two whole adds,
///  each record's identities checked against the live folder and the live dataset,
///  and the record and this boot's identifier kept on the guest's own disk.
/// Why:
///  T27 must read a record whose `slotIdentity=` and `checkoutDatasetGuid=` are real,
///  because two wrong values compare equal just as well as two right ones.
///  The second slot is the red control's,
///  planted over after the reboot.
fn boot_before(fixture: &Fixture) {
    for name in [BOOT_SLOT, BOOT_PLANT_SLOT] {
        fixture.clear_one_slot(name);
        let output: Output = fixture.run_add(name);
        let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
        assert!(output.status.success(), "the add at {name} failed: {said}");
        assert!(
            said.contains("made the worktree as a ZFS clone"),
            "the add at {name} took the ordinary worktree path instead of the clone route, so no record was written: {said}"
        );
        let record: BindingRecord = fixture.binding(name).unwrap_or_else(|| panic!("the filled add at {name} wrote no binding"));
        assert_eq!(record.phase, Phase::Filled, "the add at {name} filled the worktree");
        // The recorded identity is the empty slot folder's, and the clone now covers that path:
        // the unmount makes the folder answer again, and nothing below writes to it.
        fixture.unmount(name);
        let recorded: FolderIdentity = record.slot_identity.unwrap_or_else(|| panic!("the record at {name} keeps no slotIdentity="));
        let live: FolderIdentity = FolderIdentity::of_folder(fixture.slot(name).as_path()).expect("the slot folder's identity");
        assert_eq!(recorded, live, "the record at {name} names the slot folder's own inode and birth time");
        let guid: String = fixture.dataset_guid(fixture.checkout().as_str());
        assert_eq!(record.checkout_dataset_guid.file_name(), guid, "the record at {name} names the checkout dataset's live GUID");
        let bytes: Vec<u8> = fixture.binding_bytes(name).unwrap_or_else(|| panic!("the record at {name} cannot be read"));
        fixture.keep(format!("{name}.binding").as_str(), bytes.as_slice());
        println!(
            "MEASUREMENT before-reboot slot={name} slot-identity={} checkout-guid={guid} clone-guid={} device={} bytes={} boot-id={}",
            recorded.encode(),
            record.clone_guid.map(|clone| return clone.file_name()).unwrap_or_else(|| return String::from("-")),
            fixture.device_of_the_slot(name),
            bytes.len(),
            this_boot()
        );
    }
    fixture.keep("boot-id", this_boot().as_bytes());
}

/// What:
///  The half after the reboot:
///  the record read back once the guest has booted again,
///  and again after its dataset was exported and imported.
/// Why:
///  That is T27's second sentence,
///  and the boot identifier kept before the reboot is what proves the guest really booted.
fn boot_after(fixture: &Fixture) {
    let before: String = String::from_utf8(fixture.kept("boot-id")).expect("the kept boot identifier");
    let now: String = this_boot();
    assert_ne!(now, before, "the boot identifier is still {now}, so this half ran without the guest booting");
    read_the_record_back(fixture, BOOT_SLOT, "after the guest rebooted");
    fixture.export_and_import_the_pool();
    read_the_record_back(fixture, BOOT_SLOT, "after the dataset was exported and imported");
}

/// What:
///  The red control's half:
///  the slot folder was replaced,
///  so the record's `slotIdentity=` must no longer match it,
///  and this must fail naming both.
/// Why:
///  An arm whose comparison cannot fail proves nothing;
///  the driver's own step plants the replacement and requires this run's refusal.
fn boot_plant(fixture: &Fixture) {
    let record: BindingRecord = fixture.binding(BOOT_PLANT_SLOT).unwrap_or_else(|| panic!("the planted slot's record cannot be read"));
    let recorded: FolderIdentity = record.slot_identity.unwrap_or_else(|| panic!("the planted slot's record keeps no slotIdentity="));
    let live: FolderIdentity = FolderIdentity::of_folder(fixture.slot(BOOT_PLANT_SLOT).as_path()).expect("the planted slot folder's identity");
    assert_eq!(
        recorded,
        live,
        "the record's slotIdentity= no longer names the slot folder, so this arm is what refuses a record whose identity did not survive: recorded {}, live {}",
        recorded.encode(),
        live.encode()
    );
}

/// What:
///  The half before the guest loses power:
///  one whole add,
///  its record kept and this boot's identifier kept,
///  then the guest's own power cut,
///  with no orderly shutdown behind it.
/// Why:
///  The record's last write is the routed one the arm reads back,
///  and it is the flushed one,
///  and a read or a second write after it would change what the cut is asked about.
///  The cut is made here rather than by a command after this run ends,
///  because ZFS commits a transaction group within `zfs_txg_timeout` seconds of its first change (F12)
///  and a cut that waits for a round trip can land after that commit,
///  which would answer a different question.
fn power_before(fixture: &Fixture) {
    fixture.clear_one_slot(POWER_SLOT);
    let output: Output = fixture.run_add(POWER_SLOT);
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert!(output.status.success(), "the add at {POWER_SLOT} failed: {said}");
    assert!(
        said.contains("made the worktree as a ZFS clone"),
        "the add at {POWER_SLOT} took the ordinary worktree path instead of the clone route, so there is no record to lose: {said}"
    );
    let bytes: Vec<u8> = fixture.binding_bytes(POWER_SLOT).unwrap_or_else(|| panic!("the filled add at {POWER_SLOT} wrote no binding"));
    let record: BindingRecord = parse_binding(bytes.as_slice()).unwrap_or_else(|refusal| panic!("the record at {POWER_SLOT} is not this grammar's: {refusal:?}"));
    assert_eq!(record.phase, Phase::Filled, "the add at {POWER_SLOT} filled the worktree, so its last record write is the filled one");
    fixture.keep("power.binding", bytes.as_slice());
    fixture.keep("power.boot-id", this_boot().as_bytes());
    println!(
        "MEASUREMENT before-cut slot={POWER_SLOT} phase={:?} bytes={} slot-identity={} device={} sysrq-mask={} boot-id={}",
        record.phase,
        bytes.len(),
        record.slot_identity.expect("the filled record keeps the slot's identity").encode(),
        fixture.device_of_the_slot(POWER_SLOT),
        std::fs::read_to_string("/proc/sys/kernel/sysrq").expect("the sysrq mask").trim(),
        this_boot()
    );
    if std::env::var(CUT_VARIABLE).is_ok() {
        cut_the_power();
    }
}

/// What:
///  Take the guest's power at once:
///  the kernel's own immediate reboot,
///  which does not sync and does not unmount anything.
/// Why:
///  `mvm` offers no verb that powers a guest off and keeps its disk
///  (`mvm destroy` runs `virsh destroy` and then deletes the storage),
///  so the deepest loss reachable here is the one the guest can make of itself,
///  and it is reachable only from inside through `mvm exec` anyway.
///  `b` is "immediately reboot the system without syncing or unmounting your disks"
///  (`Documentation/admin-guide/sysrq.rst`).
///  So the guest's page cache is gone with it,
///  and its ZFS in memory,
///  and every unwritten byte of the pool's image.
fn cut_the_power() {
    let written: std::io::Result<()> = std::fs::write("/proc/sysrq-trigger", b"b");
    panic!("the kernel did not take the power cut ({written:?}), so this half cannot answer T28");
}

/// What:
///  The half after the cut:
///  the record the route flushed must read back whole,
///  with the identities it keeps still naming the live folder and dataset.
/// Why:
///  That is T28,
///  and the guest's boot identifier is what proves it really lost power.
fn power_after(fixture: &Fixture) {
    let before: String = String::from_utf8(fixture.kept("power.boot-id")).expect("the kept boot identifier");
    let now: String = this_boot();
    assert_ne!(now, before, "the boot identifier is still {now}, so this half ran without the guest losing power");
    read_the_record_back(fixture, POWER_SLOT, "after the guest lost power");
}

/// What:
///  T27:
///  the boot arm reads a record written before the guest rebooted,
///  and again after the dataset was exported and imported.
/// Why:
///  The record keeps a folder's inode and birth time and the dataset's GUID,
///  and whether those still answer the same values after a boot is what step 8's boot arm owes,
///  and what this design's M7 owes,
///  and this arm runs it once for both.
#[test]
fn the_boot_arm_reads_a_record_written_before_the_guest_rebooted() {
    if !guest() {
        return;
    }
    let Some(half) = requested_half("the boot arm") else {
        return;
    };
    let fixture: Fixture = Fixture::bare();
    // The first half imports too, because a half is run again after a cut that left the pool unimported, and the import
    // is skipped when the pool already answers.
    fixture.import_the_pool();
    if half == BEFORE {
        fixture.adopt_the_guest();
        boot_before(&fixture);
        return;
    }
    fixture.adopt_the_guest();
    if std::env::var("CLI_GIT_ZFS_ADD_PLANT").is_ok() {
        boot_plant(&fixture);
        return;
    }
    boot_after(&fixture);
}

/// What:
///  T28:
///  a write the route flushed is read back after the guest loses power without an orderly shutdown.
/// Why:
///  The design flushes every record write on the strength of a source reading (`F12`),
///  and M6 is the run that decides whether the name and the bytes are still there after a cut.
#[test]
fn a_flushed_write_is_read_back_after_the_guest_loses_power() {
    if !guest() {
        return;
    }
    let Some(half) = requested_half("the power-loss arm") else {
        return;
    };
    let fixture: Fixture = Fixture::bare();
    // The first half imports too, because a half is run again after a cut that left the pool unimported, and the import
    // is skipped when the pool already answers.
    fixture.import_the_pool();
    if half == BEFORE {
        fixture.adopt_the_guest();
        power_before(&fixture);
        return;
    }
    fixture.adopt_the_guest();
    power_after(&fixture);
}

//endregion The two arms a boot and a power cut split in half
