//! What:
//!  Plan step 10's arms in the throwaway guest:
//!  the whole register-first add from the claimed slot to a verified fill,
//!  a second add at the filled slot,
//!  a kill at each phase and the resume after it,
//!  what each kill leaves,
//!  and the pool's full answer at the snapshot.
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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const add = runAdd(slot); await mountWhenTheCloneAppears(slot); assertFilled(add);
//! ```

#![cfg(target_os = "linux")]

use git_policy_cli::worktree_clone_binding_record::{BindingRecord, Phase, parse_binding};
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness::{ReadyState, ReadinessRecord, SweepState};
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
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
        let root: PathBuf = PathBuf::from(required(ROOT_VARIABLE));
        let pool: String = required(POOL_VARIABLE);
        let sbin: String = required(SBIN_VARIABLE);
        let fixture: Fixture = Fixture { root, pool, sbin };
        fixture.make_the_repository();
        fixture.make_the_worktrees_folder();
        fixture.publish_the_readiness();
        fixture.make_the_wrapper_link();
        return fixture;
    }

    /// The checkout's dataset name.
    fn checkout(&self) -> String {
        return format!("{}/{CHECKOUT_LEAF}", self.pool);
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
        return Command::new("zfs")
            .args(arguments)
            .env("PATH", format!("{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin", self.sbin))
            .env("LD_LIBRARY_PATH", format!("{}/lib", self.sbin.trim_end_matches("/sbin")))
            .output()
            .unwrap_or_else(|error| panic!("zfs {arguments:?} could not start: {error}"));
    }

    /// Run one `zfs` command and require success.
    fn zfs_ok(&self, arguments: &[&str]) {
        let output: Output = self.zfs(arguments);
        assert!(output.status.success(), "zfs {arguments:?} was refused: {}", String::from_utf8_lossy(output.stderr.as_slice()));
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
        let snapshots: Output = self.zfs(&["list", "-H", "-o", "name", "-d", "1", "-t", "snapshot", self.checkout().as_str()]);
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
    let arms: [(&str, Phase); 8] = [
        ("slot-identity", Phase::Slot),
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
fn a_full_container_at_the_clone_leaves_the_claim_the_slot_and_the_registration() {
    if !guest() {
        return;
    }
    let fixture: Fixture = Fixture::prepare();
    // The container's quota is the space every clone under it may use, so a
    // checkout larger than it cannot be cloned.
    let big: PathBuf = fixture.root.join("big.bin");
    let mut file: std::fs::File = std::fs::File::create(big.as_path()).expect("the big file");
    file.write_all(&vec![7u8; 8 * 1024 * 1024]).expect("the big file's bytes");
    drop(file);
    std::thread::sleep(std::time::Duration::from_secs(1));
    fixture.git(&["add", "big.bin"]);
    fixture.git(&["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "big"]);
    // The clone shares its blocks with the snapshot, so a quota on the
    // container does not charge it; the arm fills the pool itself with a
    // dataset of zeros and compression off.
    let filler: String = format!("{}/filler", fixture.pool);
    fixture.zfs_ok(&["create", "-o", "mountpoint=/root/zfs-filler", "-o", "compression=off", filler.as_str()]);
    let _filled: Output = Command::new("/usr/bin/dd").args(["if=/dev/zero", "of=/root/zfs-filler/fill", "bs=1M"]).output().expect("dd fills the pool");
    let output: Output = fixture.run_add("full");
    let _gone: Output = fixture.zfs(&["destroy", "-r", "-f", filler.as_str()]);
    assert!(!output.status.success(), "the add with a full pool is refused");
    let said: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert!(said.contains("ENOSPC") || said.contains("No space left") || said.contains("space"), "the refusal names the pool's space: {said}");
    let record: BindingRecord = fixture.binding("full").expect("the refused add's binding");
    assert_eq!(record.phase, Phase::Registered, "the snapshot's refusal leaves the registration phase");
    assert!(fixture.slot("full").join(".git").is_file(), "the slot and its pointer stand");
    assert!(!fixture.dataset_stands(fixture.clone("full").as_str()), "no clone was made");
    let resumed: Output = fixture.run_add("full");
    assert!(resumed.status.success(), "the resume after the full pool failed: {}", String::from_utf8_lossy(resumed.stderr.as_slice()));
    assert_eq!(fixture.binding("full").expect("the resumed binding").phase, Phase::Filled);
    fixture.clear_the_slot("full");
    // The checkout returns to the two tracked files, so an arm that runs after
    // this one reads the same source.
    let _big_gone: std::io::Result<()> = std::fs::remove_file(big.as_path());
    fixture.git(&["add", "-A"]);
    fixture.git(&["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "big gone"]);
}
