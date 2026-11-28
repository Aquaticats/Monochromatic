//! What:
//!  The layout plan step 21's guest arms add worktrees from:
//!  the design's checkout dataset and the clone container beside it,
//!  the repository the arms commit their fixtures into,
//!  the wrapper's own link,
//!  and the one add the arms drive,
//!  which mounts the clone the route asks for.
//! Why:
//!  Every arm here prepares dependencies in a worktree the ZFS clone route made,
//!  and that route serves an add only on the step 19 design's layout,
//!  so the fixture is the design's datasets rather than a folder on the guest's
//!  root
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19's layout,
//!  and `SPEC.md` "The clone route's repair and its watches").
//!  The checkout dataset is made here rather than migrated,
//!  because the owner's migration has not run,
//!  so every fact these arms prove is the guest's semantics on a file-backed
//!  pool and never this host's storage.
//!  The arms add and remove worktrees in that one repository,
//!  so one fixture owns the layout for a whole run and each arm clears the slot
//!  it made.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = prepare(); const added = fixture.add('pnpm-0'); fixture.clear('pnpm-0');
//! ```

#![cfg(target_os = "linux")]

use super::zfs_sweep_guest::{
    prepare_the_store,
    required,
    unmount_at,
    write_configuration,
    write_passing_probe_record,
    zfs,
    zfs_ignored,
    zfs_ok,
};
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness::{
    ReadinessRecord,
    ReadyState,
    SweepState,
};
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    Output,
    Stdio,
};
use std::time::{
    Duration,
    Instant,
};

/// The variable the guest driver sets,
/// so a host run skips instead of touching a pool.
pub(super) const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_STEP21_GUEST";

/// The pool the driver made.
pub(super) const POOL_VARIABLE: &str = "CLI_GIT_ZFS_STEP21_POOL";

/// The OpenZFS userspace the arm's own reads use.
pub(super) const SBIN_VARIABLE: &str = "CLI_GIT_ZFS_STEP21_SBIN";

/// The checkout root the arm mounts the design's dataset at.
pub(super) const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_STEP21_ROOT";

/// The checkout dataset's leaf name the step 19 design fixes.
const CHECKOUT_LEAF: &str = "monochromatic-git-checkout";

/// The container's leaf name the design fixes beside it.
const CONTAINER_LEAF: &str = "monochromatic-git-worktree-clone-datasets";

/// The wrapper program the driver built.
pub(super) const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// The real Git the fixture commits with,
/// never the wrapper's link.
pub(super) const REAL_GIT: &str = "/usr/bin/git";

/// How often a poller looks at a dataset,
/// a record or a mount.
const POLL: Duration = Duration::from_millis(20);

/// How long one add may take before the arm gives up on it.
const ADD_WAIT: Duration = Duration::from_secs(180);

/// How many interleaved rounds each measured arm runs.
/// Why:
///  An odd count gives each arm a median,
///  and alternating which arm goes first keeps neither warmed by the other at
///  the end.
pub(super) const ROUNDS: usize = 3;

/// The desired-state property the add writes its mount request under.
const DESIRED_STATE: &str = "org.monochromatic:git-worktree-clone-mount-desired-state";

/// The account this arm runs as,
/// whose delegation the route's preflight reads.
const ACCOUNT: &str = "0";

/// Whether this run is inside the guest,
/// or a skip line on every other machine.
pub(super) fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!(
            "SKIP binary_zfs_step_21: {GUEST_VARIABLE} is unset, so these arms run only in the throwaway mvm guest"
        );
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname")
        .map(|text| return text.trim().to_string())
        .unwrap_or_default();
    assert_eq!(running, name, "these arms run only in the guest {name}");
    return true;
}

/// What one add came to.
pub(super) struct AddRound {
    /// The wall time from the command's start to its end.
    pub(super) elapsed: Duration,
    /// The status the command exited with.
    pub(super) status: Option<i32>,
    /// What the command printed on both streams.
    pub(super) said: String,
}

/// The fixture one run of the arms holds:
/// the design's layout,
/// a repository,
/// and the wrapper's own link.
pub(super) struct Fixture {
    /// The checkout root,
    /// which is the mount point of the checkout dataset.
    root: PathBuf,
    /// The pool's name.
    pool: String,
    /// The OpenZFS userspace.
    sbin: String,
    /// The checkout dataset the layout made.
    pub(super) checkout: String,
    /// The clone container the layout made.
    pub(super) container: String,
}

impl Fixture {
    /// Make the design's layout,
    /// the repository and the wrapper's link.
    pub(super) fn prepare() -> Fixture {
        let root: PathBuf = PathBuf::from(required(ROOT_VARIABLE));
        let pool: String = required(POOL_VARIABLE);
        let sbin: String = required(SBIN_VARIABLE);
        let (checkout, container): (String, String) = make_the_layout(&sbin, &pool, root.as_path());
        let fixture: Fixture = Fixture {
            root,
            pool,
            sbin,
            checkout,
            container,
        };
        fixture.make_the_repository();
        fixture.make_the_worktrees_folder();
        fixture.publish_the_readiness();
        write_passing_probe_record();
        prepare_the_store(fixture.root.as_path());
        write_configuration(fixture.root.as_path(), None, None);
        fixture.make_the_wrapper_link();
        return fixture;
    }

    /// The checkout root.
    pub(super) fn root(&self) -> &Path {
        return self.root.as_path();
    }

    /// The folder Git's common directory is.
    pub(super) fn common(&self) -> PathBuf {
        return self.root.join(".git");
    }

    /// The records' home folder.
    pub(super) fn home_records(&self) -> PathBuf {
        return self.common().join("monochromatic-git-policy-cli");
    }

    /// The folder each arm runs with as `HOME`,
    /// outside the checkout so the source stays clean and so every install of
    /// one round reads one store.
    pub(super) fn home_folder(&self) -> PathBuf {
        return PathBuf::from("/root/step21-home");
    }

    /// The session's runtime folder,
    /// short enough that the service's socket fits one address.
    pub(super) fn runtime_folder(&self) -> PathBuf {
        return PathBuf::from("/root/r");
    }

    /// The folder the wrapper's link sits in.
    pub(super) fn bin_folder(&self) -> PathBuf {
        return PathBuf::from("/root/step21-bin");
    }

    /// One slot's folder,
    /// which the design keeps under the checkout.
    pub(super) fn slot(
        &self,
        name: &str,
    ) -> PathBuf {
        return self.root.join(".claude").join("worktrees").join(name);
    }

    /// One slot's clone dataset.
    pub(super) fn clone(
        &self,
        name: &str,
    ) -> String {
        return format!("{}/{}", self.container, name);
    }

    /// The folder a control arm works in,
    /// on the checkout dataset,
    /// so a control and the clone's own preparation differ only by the
    /// filesystem the work happens on.
    pub(super) fn control(
        &self,
        arm: &str,
        round: usize,
    ) -> PathBuf {
        return self
            .root
            .join(".step21")
            .join(arm)
            .join(format!("round-{round}"));
    }

    /// The wrapper started as `git`,
    /// in a folder of its own.
    pub(super) fn wrapper_git(&self) -> PathBuf {
        return self.bin_folder().join("git");
    }

    /// The environment every command runs with,
    /// so no host variable reaches it and every arm searches the same path.
    pub(super) fn environment(
        &self,
        program_folder: &Path,
    ) -> Vec<(String, String)> {
        return vec![
            (
                String::from("PATH"),
                format!(
                    "{}:{}:/usr/local/bin:/root/.cargo/bin:/usr/bin:/bin",
                    program_folder.display(),
                    self.bin_folder().display()
                ),
            ),
            (
                String::from("HOME"),
                self.home_folder().display().to_string(),
            ),
            (
                String::from("XDG_RUNTIME_DIR"),
                self.runtime_folder().display().to_string(),
            ),
            // The toolchain the guest was prepared with lives under root's own folders, and the fixture's home is not root's.
            (String::from("CARGO_HOME"), String::from("/root/.cargo")),
            (String::from("RUSTUP_HOME"), String::from("/root/.rustup")),
            // The repair says which branch it took only in this log, and an arm reads branches as evidence.
            (String::from("MONOCHROMATIC_VERBOSE"), String::from("true")),
            (String::from("GIT_CONFIG_NOSYSTEM"), String::from("1")),
            (
                String::from("GIT_CONFIG_GLOBAL"),
                String::from("/nonexistent-global-config"),
            ),
            (String::from("GIT_AUTHOR_NAME"), String::from("Fixture")),
            (
                String::from("GIT_AUTHOR_EMAIL"),
                String::from("fixture@example.invalid"),
            ),
            (String::from("GIT_COMMITTER_NAME"), String::from("Fixture")),
            (
                String::from("GIT_COMMITTER_EMAIL"),
                String::from("fixture@example.invalid"),
            ),
        ];
    }

    /// One command in the fixture's environment,
    /// which must succeed.
    pub(super) fn run(
        &self,
        program: &Path,
        arguments: &[&str],
        directory: &Path,
    ) -> Output {
        let mut command_line: Command = Command::new(program);
        command_line
            .args(arguments)
            .current_dir(directory)
            .env_clear();
        for (key, value) in self.environment(Path::new("/usr/bin")) {
            command_line.env(key, value);
        }
        let output: Output = command_line
            .output()
            .unwrap_or_else(|error| panic!("{program:?} could not start: {error}"));
        assert!(
            output.status.success(),
            "{program:?} {arguments:?} in {} was refused: {}",
            directory.display(),
            String::from_utf8_lossy(output.stderr.as_slice())
        );
        return output;
    }

    /// One real Git command in the checkout,
    /// which must succeed.
    pub(super) fn git(
        &self,
        arguments: &[&str],
    ) -> Output {
        return self.run(Path::new(REAL_GIT), arguments, self.root.as_path());
    }

    /// Write the fixture's tracked and ignored state,
    /// commit it,
    /// and leave the checkout clean,
    /// so the route's own checks pass and the clone's snapshot carries exactly
    /// this.
    pub(super) fn commit(
        &self,
        files: &[(&str, &str)],
        message: &str,
    ) {
        for (relative, text) in files {
            let path: PathBuf = self.root.join(relative);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("the fixture file's folder");
            }
            std::fs::write(path.as_path(), text.as_bytes()).expect("one fixture file");
        }
        self.git(&["add", "--all"]);
        self.git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--quiet",
            "--no-verify",
            "-m",
            message,
        ]);
    }

    /// Take a tracked path away and commit the removal.
    pub(super) fn forget(
        &self,
        paths: &[&str],
        message: &str,
    ) {
        if paths.is_empty() {
            return;
        }
        for relative in paths {
            let path: PathBuf = self.root.join(relative);
            let _removed: std::io::Result<()> = std::fs::remove_file(path.as_path());
        }
        self.git(&["add", "--all"]);
        self.git(&[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "--quiet",
            "--no-verify",
            "-m",
            message,
        ]);
    }

    /// One `zfs` command whose failure the arm expects.
    fn zfs_ignored(
        &self,
        arguments: &[&str],
    ) {
        zfs_ignored(self.sbin.as_str(), arguments);
    }

    /// What:
    ///  Take ignored state away,
    ///  which the clone's snapshot would otherwise carry into the next arm.
    /// Why:
    ///  Each arm's add must carry exactly the state that arm is about,
    ///  and an ignored folder one arm made is no part of the next one.
    pub(super) fn remove_ignored(
        &self,
        names: &[&str],
    ) {
        for name in names {
            let path: PathBuf = self.root.join(name);
            let _gone: std::io::Result<()> = if path.is_dir() {
                std::fs::remove_dir_all(path.as_path())
            } else {
                std::fs::remove_file(path.as_path())
            };
        }
    }

    /// What:
    ///  Refuse a checkout the route's own clean check would refuse.
    /// Why:
    ///  The route serves an add from a clean checkout and from nowhere else,
    ///  so an arm that left a change behind would fail at the add and not at
    ///  its own line.
    pub(super) fn assert_clean(&self) {
        let status: String =
            String::from_utf8_lossy(self.git(&["status", "--porcelain"]).stdout.as_slice())
                .to_string();
        assert_eq!(status, "", "the checkout is not clean: {status}");
    }

    /// The repository with the fixture's ignore rules committed,
    /// and no file that the route's cleanliness check would refuse.
    fn make_the_repository(&self) {
        if self.common().is_dir() {
            return;
        }
        // A file written in the same second as the index is a racy entry, which the route's clean check refuses to vouch for.
        std::thread::sleep(Duration::from_secs(1));
        self.git(&["init", "--quiet", "--initial-branch=main"]);
        self.commit(
            &[
                (
                    ".gitignore",
                    ".claude/\nnode_modules/\ntarget/\ntarget.monochromatic-prewarm/\n.cache/\n.step21/\nelsewhere/\n",
                ),
                ("src/main.txt", "step 21's fixture\n"),
            ],
            "one",
        );
    }

    /// The folder the route's slots live in,
    /// at the mode the design's unit sets.
    fn make_the_worktrees_folder(&self) {
        let parent: PathBuf = self.root.join(".claude").join("worktrees");
        std::fs::create_dir_all(parent.as_path()).expect("the worktrees folder");
        std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o755))
            .expect("the folder's mode");
    }

    /// The readiness report the add's preflight requires,
    /// written as the boot unit would.
    fn publish_the_readiness(&self) {
        let file: ReadinessFile = ReadinessFile::live();
        file.prepare().expect("the readiness directory");
        let boot: String = std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .expect("this boot's identifier")
            .trim()
            .to_string();
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

    /// The link every arm starts as `git`,
    /// outside the checkout.
    fn make_the_wrapper_link(&self) {
        std::fs::create_dir_all(self.bin_folder().as_path()).expect("the bin folder");
        std::fs::create_dir_all(self.home_folder().as_path()).expect("the home folder");
        std::fs::create_dir_all(self.runtime_folder().as_path()).expect("the runtime folder");
        std::fs::set_permissions(
            self.runtime_folder().as_path(),
            std::fs::Permissions::from_mode(0o700),
        )
        .expect("the runtime folder's mode");
        let link: PathBuf = self.wrapper_git();
        let _stale: std::io::Result<()> = std::fs::remove_file(link.as_path());
        std::os::unix::fs::symlink(WRAPPER, link.as_path()).expect("the wrapper's link");
    }

    /// One slot's binding record,
    /// or nothing when no phase wrote it.
    fn record(
        &self,
        name: &str,
    ) -> Option<String> {
        return std::fs::read_to_string(
            self.home_records()
                .join("worktree-clones")
                .join("v1")
                .join(format!("{name}.binding")),
        )
        .ok();
    }

    /// The origin snapshot the add recorded,
    /// or nothing.
    fn origin_of(
        &self,
        name: &str,
    ) -> Option<String> {
        let record: String = self.record(name)?;
        return record
            .lines()
            .find_map(|line| return line.strip_prefix("originSnapshot="))
            .map(String::from);
    }

    /// Run the wrapper's own add,
    /// which is the ZFS clone route,
    /// mounting the clone when its intent appears and calling `before` first
    /// when the arm asks.
    pub(super) fn add(
        &self,
        name: &str,
        before_the_mount: Option<&mut dyn FnMut(&Path)>,
    ) -> AddRound {
        let slot: PathBuf = self.slot(name);
        let clone: String = self.clone(name);
        let mut hook: Option<&mut dyn FnMut(&Path)> = before_the_mount;
        let started: Instant = Instant::now();
        let mut command_line: Command = Command::new(self.wrapper_git());
        command_line
            .args(["worktree", "add", slot.to_str().expect("a text path")])
            .current_dir(self.root.as_path())
            .env_clear();
        for (key, value) in self.environment(Path::new("/usr/bin")) {
            command_line.env(key, value);
        }
        command_line.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child: Child = command_line.spawn().expect("the wrapper starts");
        let mut attach_made: bool = false;
        while started.elapsed() < ADD_WAIT {
            if let Ok(Some(_status)) = child.try_wait() {
                break;
            }
            if !attach_made
                && slot.is_dir()
                && dataset_stands(self.sbin.as_str(), clone.as_str())
                && the_intent_says_mount(self.sbin.as_str(), clone.as_str())
            {
                if let Some(call) = hook.take() {
                    call(slot.as_path());
                }
                let output: Output = Command::new("/usr/bin/mount")
                    .args([
                        "-t",
                        "zfs",
                        "-o",
                        "nosuid,nodev",
                        clone.as_str(),
                        slot.to_str().expect("a text path"),
                    ])
                    .output()
                    .expect("mount starts");
                assert!(
                    output.status.success(),
                    "the mount of {clone} at the slot was refused: {}",
                    String::from_utf8_lossy(output.stderr.as_slice())
                );
                attach_made = true;
            }
            std::thread::sleep(POLL);
        }
        let output: Output = child.wait_with_output().expect("the add ends");
        return AddRound {
            elapsed: started.elapsed(),
            status: output.status.code(),
            said: format!(
                "{}{}",
                String::from_utf8_lossy(output.stdout.as_slice()),
                String::from_utf8_lossy(output.stderr.as_slice())
            ),
        };
    }

    /// Take a slot's clone,
    /// mount,
    /// folder,
    /// registration and records away,
    /// so the next round starts from nothing.
    pub(super) fn clear(
        &self,
        name: &str,
    ) {
        let slot: PathBuf = self.slot(name);
        let clone: String = self.clone(name);
        unmount_at(slot.as_path());
        let origin: Option<String> = self.origin_of(name);
        self.zfs_ignored(&["destroy", "-r", "-f", clone.as_str()]);
        if let Some(snapshot) = origin {
            self.zfs_ignored(&["destroy", "-f", snapshot.as_str()]);
        }
        let _folder: std::io::Result<()> = std::fs::remove_dir_all(slot.as_path());
        let _records: std::io::Result<()> = std::fs::remove_dir_all(self.home_records().as_path());
        let _pruned: Output = self.run(
            Path::new(REAL_GIT),
            &["worktree", "prune"],
            self.root.as_path(),
        );
    }
}

/// The design's dataset layout for the guest,
/// with the delegation the route's preflight reads:
/// the checkout dataset mounted at the configured path and private,
/// and the clone container beside it as a legacy dataset.
fn make_the_layout(
    sbin: &str,
    pool: &str,
    checkout: &Path,
) -> (String, String) {
    let base: String = format!("{pool}/step21");
    let checkout_dataset: String = format!("{base}/{CHECKOUT_LEAF}");
    let container: String = format!("{base}/{CONTAINER_LEAF}");
    zfs_ignored(sbin, &["destroy", "-r", "-f", base.as_str()]);
    // The parent is made without a mountpoint, because `zfs create -p -o` applies its options to every parent it makes and a
    // second mount at the same path would hide the checkout.
    zfs_ok(
        sbin,
        &["create", "-p", "-o", "mountpoint=none", base.as_str()],
    );
    let mountpoint: String = format!("mountpoint={}", checkout.display());
    zfs_ok(
        sbin,
        &[
            "create",
            "-o",
            mountpoint.as_str(),
            checkout_dataset.as_str(),
        ],
    );
    std::fs::create_dir_all(checkout).expect("the checkout's mount point");
    // The design requires a not-shared immediate parent, which its boot unit sets; this arm sets it for the checkout alone.
    let _private: Output = Command::new("/usr/bin/mount")
        .args(["--make-private", checkout.to_str().expect("a text path")])
        .output()
        .expect("the private mount starts");
    zfs_ok(
        sbin,
        &[
            "create",
            "-o",
            "mountpoint=legacy",
            "-o",
            "canmount=off",
            "-o",
            "setuid=off",
            "-o",
            "devices=off",
            container.as_str(),
        ],
    );
    // The permissions the design scopes, handed to the account this arm runs as, which is root: the route's preflight reads
    // the caller's own delegation and refuses without them.
    for arguments in [
        vec![
            "allow",
            "-l",
            "-u",
            ACCOUNT,
            "snapshot",
            checkout_dataset.as_str(),
        ],
        vec![
            "allow",
            "-d",
            "-u",
            ACCOUNT,
            "clone,mount,destroy",
            checkout_dataset.as_str(),
        ],
        vec![
            "allow",
            "-l",
            "-u",
            ACCOUNT,
            "create,mount",
            container.as_str(),
        ],
        vec![
            "allow",
            "-d",
            "-u",
            ACCOUNT,
            "mount,destroy,userprop",
            container.as_str(),
        ],
    ] {
        zfs_ok(sbin, arguments.as_slice());
    }
    return (checkout_dataset, container);
}

/// Whether a dataset answers `zfs list`.
fn dataset_stands(
    sbin: &str,
    dataset: &str,
) -> bool {
    return zfs(sbin, "zfs", &["list", "-H", "-o", "name", dataset], None).0;
}

/// Whether one clone's durable intent asks for its mount,
/// so the arm mounts only once the route's own state says a mount belongs there.
fn the_intent_says_mount(
    sbin: &str,
    clone: &str,
) -> bool {
    let (_ok, said, _error): (bool, String, String) = zfs(
        sbin,
        "zfs",
        &["get", "-H", "-o", "value", DESIRED_STATE, clone],
        None,
    );
    return said.trim().ends_with(":mount");
}

/// Whether a mount stands at one folder of this process's table.
pub(super) fn mounted_at(folder: &Path) -> bool {
    let table: String = std::fs::read_to_string("/proc/self/mountinfo")
        .expect("the arm reads this process's mount table");
    let wanted: String = folder.display().to_string();
    return table
        .lines()
        .any(|line| return line.split(' ').nth(4) == Some(wanted.as_str()));
}

/// The device a folder lives on,
/// which is the number that decides whether a rename can exchange two files.
pub(super) fn device_of(folder: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt as _;
    return std::fs::metadata(folder)
        .expect("the folder's own metadata")
        .dev();
}
