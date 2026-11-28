//! What:
//!  One arm's fixture for the forced removal's guest work:
//!  a pool made from a file,
//!  the design's checkout above it with one real registration and one real
//!  clone,
//!  the binding and origin-side records an add writes,
//!  and the two children an arm drives,
//!  the handler's sweep running beside the wrapper and the wrapper itself.
//! Why:
//!  Every claim these arms make is a pool's own answer,
//!  and no control on a person's machine may open `/dev/zfs` or make a mount,
//!  so the fixture is built only where the guest's variable names the machine
//!  (`doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`,
//!  "All guest work runs on OVHcloud,
//! 2026-11-28").
//!  The layout is the shipped route's:
//!  the checkout dataset with its private propagation,
//!  the sibling clone container,
//!  and a clone of the checkout dataset's own snapshot,
//!  so a removal's proof reads what the design built and not a stand-in.
//!  Each arm takes its own pool and its own checkout,
//!  so an arm that dies leaves the next one a machine it cannot see.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const arm = Arm.make(1);
//! const said = arm.wrapperGit(['worktree', 'remove', '--force', shown(arm.place.slot)]);
//! arm.tearDown();
//! ```

use super::zfs_slot_state_guest_fixture::{
    Place,
    createtxg_of,
    guid_of,
    publish,
};
use super::zfs_sweep_guest::{
    NONCE,
    ORIGIN_PREFIX,
    SBIN_VARIABLE,
    clone_the_checkouts_snapshot,
    command_ok,
    make_the_designs_layout,
    mount_request,
    prepare_the_store,
    required,
    sweep,
    write_configuration,
    write_passing_probe_record,
    zfs,
    zfs_ok,
};
use git_policy_cli::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use git_policy_cli::worktree_clone_binding_record::binding_inventory::{
    EntryKind,
    InventoryEntry,
    InventoryKind,
    InventoryRecord,
};
use git_policy_cli::worktree_clone_binding_record::binding_words::Phase;
use git_policy_cli::worktree_clone_binding_record::record_publication::publish_inventory;
use git_policy_cli::worktree_clone_binding_record::registration_home::{
    home_folder,
    open_or_create_home,
    plans_folder,
};
use git_policy_cli::worktree_clone_mount_protocol::DatasetGuid;
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

/// The variable the guest driver sets for these arms.
pub(super) const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_FORCE_GUEST";

/// The slot every arm registers and clones.
pub(super) const SLOT: &str = "fix-flaky-test";

/// The prefix of the snapshots an arm makes of its own clone,
/// which are the names a removal's report lists as destroyed.
pub(super) const OWN_SNAPSHOT_PREFIX: &str = "arm-own-";

/// The wrapper program the driver built,
/// which every arm reaches through a `git` link so it runs as the person's `git`.
pub(super) const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// The file Git writes a person's lock into under one registration,
/// which is the lock design's own name.
pub(super) const THE_LOCKS_FILE: &str = "locked";

/// Whether this run is inside the guest the driver named.
pub(super) fn in_the_guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!(
            "SKIP worktree_clone_removal_proof_and_plan::removal_force_guest: {GUEST_VARIABLE} is unset, so these arms run only in the throwaway mvm guest"
        );
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname")
        .map(|text| return text.trim().to_string())
        .unwrap_or_default();
    assert_eq!(running, name, "these arms run only in the guest {name}");
    return true;
}

/// One checkout path as the argument Git takes.
pub(super) fn shown(path: &Path) -> &str {
    return path.to_str().expect("the guest's paths are UTF-8");
}

/// What:
///  One arm's pool, checkout, registration, clone and records.
/// Why:
///  An arm names the pool it destroys and the checkout whose records the
///  wrapper reads,
///  and the two must be the ones the clone was made in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Arm = { sbin, pool, root, checkout, place, clone, cloneGuid };
/// ```
pub(super) struct Arm {
    /// The OpenZFS userspace prefix the guest's recipe installed.
    pub(super) sbin: String,
    /// The pool one arm makes and destroys.
    pub(super) pool: String,
    /// The folder this arm's pool image and checkout live under.
    pub(super) root: PathBuf,
    /// The checkout the wrapper runs in.
    pub(super) checkout: PathBuf,
    /// The slot, its clone and the records the add wrote.
    pub(super) place: Place,
    /// The clone dataset, which is the name the record holds.
    pub(super) clone: String,
    /// The clone's canonical GUID as the pool answered it.
    pub(super) clone_guid: DatasetGuid,
}

impl Arm {
    /// What:
    ///  Build one arm's whole fixture:
    ///  the pool from a file,
    ///  the design's checkout,
    ///  one registration,
    ///  one clone wanting its mount,
    ///  and the records an add leaves.
    /// Why:
    ///  A removal refuses a slot whose records it cannot read,
    ///  so a fixture that skipped the binding or the origin-side inventory would
    ///  prove nothing about the pool.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const arm = Arm.make(3);
    /// ```
    pub(super) fn make(number: usize) -> Arm {
        let sbin: String = required(SBIN_VARIABLE);
        let root: PathBuf = PathBuf::from(format!("/root/removal-force-arm-{number}"));
        let pool: String = format!("removalforce{number}");
        std::fs::create_dir_all(root.as_path()).expect("the arm's folder is made");
        let image: PathBuf = root.join("pool.img");
        let file: std::fs::File =
            std::fs::File::create(image.as_path()).expect("the pool's image is made");
        file.set_len(1_073_741_824)
            .expect("the pool's image is one gibibyte");
        drop(file);
        let made: (bool, String, String) = zfs(
            sbin.as_str(),
            "zpool",
            &[
                "create",
                "-f",
                "-O",
                "mountpoint=none",
                pool.as_str(),
                shown(image.as_path()),
            ],
            None,
        );
        assert!(
            made.0,
            "the pool `{pool}` is made from a file: {}{}",
            made.1, made.2
        );
        let checkout: PathBuf = root.join("checkout");
        let (checkout_dataset, container): (String, String) =
            make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
        command_ok("git", &["init", "--quiet", shown(checkout.as_path())]);
        std::fs::write(checkout.join("a.txt"), "the arm's own file\n")
            .expect("the checkout's one file is written");
        command_ok("git", &["-C", shown(checkout.as_path()), "add", "a.txt"]);
        command_ok(
            "git",
            &[
                "-C",
                shown(checkout.as_path()),
                "-c",
                "user.name=the arm",
                "-c",
                "user.email=arm@example.invalid",
                "commit",
                "--quiet",
                "--message",
                "the arm's base",
            ],
        );
        let slot: PathBuf = checkout.join(".claude").join("worktrees").join(SLOT);
        command_ok(
            "git",
            &[
                "-C",
                shown(checkout.as_path()),
                "worktree",
                "add",
                "--quiet",
                shown(slot.as_path()),
                "HEAD",
            ],
        );
        let slot_identity: FolderIdentity =
            FolderIdentity::of_folder(slot.as_path()).expect("the underlay's identity reads");
        let pointer: Vec<u8> =
            std::fs::read(slot.join(".git")).expect("the underlay pointer reads");
        write_passing_probe_record();
        prepare_the_store(checkout.as_path());
        write_configuration(checkout.as_path(), None, None);
        let clone: String = clone_the_checkouts_snapshot(
            sbin.as_str(),
            checkout_dataset.as_str(),
            container.as_str(),
            SLOT,
            NONCE,
            Some(mount_request(NONCE).as_str()),
        );
        let common: PathBuf = checkout.join(".git");
        let registration: PathBuf = common.join("worktrees").join(SLOT);
        let place: Place = Place {
            common,
            pool: pool.clone(),
            checkout_dataset: checkout_dataset.clone(),
            clone_dataset: clone.clone(),
            slot,
            slot_identity,
            pointer,
            registration,
        };
        publish(&place, Phase::Filled, sbin.as_str(), SLOT);
        write_the_origin_inventory(&place, sbin.as_str());
        let clone_guid: DatasetGuid = guid_of(sbin.as_str(), clone.as_str());
        return Arm {
            sbin,
            pool,
            root,
            checkout,
            place,
            clone,
            clone_guid,
        };
    }

    /// What:
    ///  Run one arm's clone attached at its slot,
    ///  which is the state every removal that reaches a pool starts from.
    /// Why:
    ///  The handler is what unmounts a clone,
    ///  and its sweep publishes the slot's identity record on the first
    ///  observation,
    ///  so an arm that never ran it would meet a handler that refuses the slot it
    ///  was never told about.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// arm.attachAtTheSlot();
    /// ```
    pub(super) fn attach_at_the_slot(&self) {
        let attached: super::zfs_sweep_guest::Run = sweep();
        assert_eq!(
            attached.status,
            Some(0),
            "the handler's sweep attaches the clone: {}",
            attached.said
        );
        assert!(
            super::zfs_sweep_guest::mounted(self.clone.as_str()),
            "the clone stands at its slot before the arm asks anything of the removal: {}",
            attached.said
        );
    }

    /// What:
    ///  Start the handler's sweep in a loop beside the wrapper,
    ///  so the removal's one `unmount` request is answered.
    /// Why:
    ///  The removal files one request and waits for its receipt,
    ///  and no single sweep can answer a request the wrapper has not filed yet.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const sweeper = arm.startTheSweeper();
    /// arm.stopTheSweeper(sweeper);
    /// ```
    pub(super) fn start_the_sweeper(&self) -> Child {
        let binary: PathBuf = super::support_installed_name::installed_name("sweep");
        return Command::new("sh")
            .args([
                "-c",
                "while :; do \"$0\" >/dev/null 2>&1; sleep 1; done",
                shown(binary.as_path()),
            ])
            .env("PATH", self.search_path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the sweeper starts under the installed name");
    }

    /// End the sweeper this arm started, and wait for it.
    pub(super) fn stop_the_sweeper(
        &self,
        mut sweeper: Child,
    ) {
        let _ended: std::io::Result<std::process::ExitStatus> =
            sweeper.kill().and_then(|()| return sweeper.wait());
    }

    /// The `PATH` the guest's OpenZFS userspace needs.
    pub(super) fn search_path(&self) -> String {
        let prefix: &str = self.sbin.trim_end_matches("/sbin");
        return format!(
            "{prefix}/sbin:{prefix}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
        );
    }

    /// What:
    ///  Run one Git command through the built wrapper,
    ///  the way a person's shell starts it.
    /// Why:
    ///  The arm's subject is the command a person meets,
    ///  so the words are the person's and the wrapper is the program the driver
    ///  built.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// arm.wrapperGit(['worktree', 'remove', shown(slot)]);
    /// ```
    pub(super) fn wrapper_git(
        &self,
        arguments: &[&str],
    ) -> Output {
        let bin: PathBuf = self.root.join("bin");
        std::fs::create_dir_all(bin.as_path()).expect("the wrapper's bin folder");
        let home: PathBuf = self.root.join("home");
        std::fs::create_dir_all(home.as_path()).expect("the wrapper's home folder");
        let runtime: PathBuf = self.root.join("runtime");
        std::fs::create_dir_all(runtime.as_path()).expect("the wrapper's runtime folder");
        let link: PathBuf = bin.join("git");
        let _stale: std::io::Result<()> = std::fs::remove_file(link.as_path());
        std::os::unix::fs::symlink(WRAPPER, link.as_path()).expect("the wrapper's git link");
        return Command::new(link.as_path())
            .args(arguments)
            .current_dir(self.checkout.as_path())
            .env_clear()
            .env("PATH", format!("{}:{}", bin.display(), self.search_path()))
            .env("HOME", home.as_path())
            .env("XDG_RUNTIME_DIR", runtime.as_path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .output()
            .unwrap_or_else(|error| {
                panic!("the wrapper `git {arguments:?}` could not start: {error}")
            });
    }

    /// The wrapper's standard output and error, joined.
    pub(super) fn said(output: &Output) -> String {
        return format!(
            "{}{}",
            String::from_utf8_lossy(output.stdout.as_slice()),
            String::from_utf8_lossy(output.stderr.as_slice())
        );
    }

    /// Make one snapshot of this arm's clone, named below `OWN_SNAPSHOT_PREFIX`.
    pub(super) fn snapshot(
        &self,
        name: &str,
    ) {
        zfs_ok(
            self.sbin.as_str(),
            &["snapshot", format!("{}@{name}", self.clone).as_str()],
        );
    }

    /// What:
    ///  Every snapshot the pool lists under this arm's clone,
    ///  in the pool's own order and without the dataset's name.
    /// Why:
    ///  The report a removal prints is compared with the pool's list and not
    ///  with a list the arm wrote,
    ///  so the arm holds the answer the pool gave at that moment.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// arm.the_snapshots(); // ['arm-own-a', 'arm-own-b']
    /// ```
    pub(super) fn the_snapshots(&self) -> Vec<String> {
        let (ok, listed, error): (bool, String, String) = zfs(
            self.sbin.as_str(),
            "zfs",
            &[
                "list",
                "-H",
                "-p",
                "-o",
                "name",
                "-t",
                "snapshot",
                "-r",
                self.clone.as_str(),
            ],
            None,
        );
        assert!(ok, "the clone's snapshots list: {listed}{error}");
        let mut names: Vec<String> = Vec::new();
        for line in listed.lines() {
            let trimmed: &str = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let Some((_, snapshot)) = trimmed.split_once('@') else {
                continue;
            };
            names.push(String::from(snapshot));
        }
        return names;
    }

    /// Whether the pool answers for one dataset name.
    pub(super) fn exists(
        &self,
        dataset: &str,
    ) -> bool {
        let listed: (bool, String, String) = zfs(
            self.sbin.as_str(),
            "zfs",
            &["list", "-H", "-o", "name", dataset],
            None,
        );
        return listed.0;
    }

    /// The plan folder's entries, which a refusal never creates.
    pub(super) fn plan_entries(&self) -> usize {
        return std::fs::read_dir(plans_folder(self.place.common.as_path()))
            .map_or(0, |entries| return entries.count());
    }

    /// Every mount row naming this arm's clone, by the pool's own source name.
    pub(super) fn mounts_of_the_clone(&self) -> Vec<String> {
        return super::zfs_sweep_guest::rows_naming(self.clone.as_str());
    }

    /// What:
    ///  Take this arm's fixture away:
    ///  every mount it left,
    ///  the pool, and the folder.
    /// Why:
    ///  A pool whose dataset is mounted cannot be destroyed,
    ///  and a leftover mount would make the next arm's checkout dataset busy.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// arm.tearDown();
    /// ```
    pub(super) fn tear_down(&self) {
        let (unmounted, said, error): (bool, String, String) = zfs(
            self.sbin.as_str(),
            "zfs",
            &["unmount", "-f", self.place.checkout_dataset.as_str()],
            None,
        );
        assert!(
            unmounted,
            "the checkout dataset unmounts for this arm's own cleanup: {said}{error}"
        );
        let destroyed: (bool, String, String) = zfs(
            self.sbin.as_str(),
            "zpool",
            &["destroy", "-f", self.pool.as_str()],
            None,
        );
        assert!(
            destroyed.0,
            "the pool `{}` goes: {}{}",
            self.pool, destroyed.1, destroyed.2
        );
        std::fs::remove_dir_all(self.root.as_path()).expect("the arm's folder goes");
    }
}

/// What:
///  The origin-side inventory the add records for one slot:
///  the checkout's snapshots at or below the origin snapshot's group and every
///  bookmark it holds.
/// Why:
///  A removal refuses a slot whose origin-side inventory cannot be read,
///  and the clone's proof compares the live objects with this set,
///  so the fixture must be the add's own record rather than a hand-written
///  list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeTheOriginInventory(place, sbin);
/// ```
fn write_the_origin_inventory(
    place: &Place,
    sbin: &str,
) {
    let origin_snapshot: String = format!("{}@{ORIGIN_PREFIX}{NONCE}", place.checkout_dataset);
    let bounds_createtxg: u64 = createtxg_of(sbin, origin_snapshot.as_str());
    let bounds_guid: DatasetGuid = guid_of(sbin, origin_snapshot.as_str());
    let mut entries: Vec<InventoryEntry> = Vec::new();
    for (kind, separator) in [("snapshot", '@'), ("bookmark", '#')] {
        let (ok, listed, error): (bool, String, String) = zfs(
            sbin,
            "zfs",
            &[
                "list",
                "-H",
                "-p",
                "-o",
                "name,guid,createtxg",
                "-t",
                kind,
                "-r",
                place.checkout_dataset.as_str(),
            ],
            None,
        );
        assert!(ok, "the checkout's {kind}s list: {listed}{error}");
        for line in listed.lines() {
            let mut fields = line.split('\t');
            let (Some(name), Some(guid), Some(createtxg)) =
                (fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            let group: u64 = createtxg
                .trim()
                .parse()
                .expect("the object's group is a number");
            let entry_kind: EntryKind = match kind {
                "snapshot" => EntryKind::Snapshot,
                _ => EntryKind::Bookmark,
            };
            if entry_kind == EntryKind::Snapshot && group > bounds_createtxg {
                continue;
            }
            entries.push(InventoryEntry {
                kind: entry_kind,
                guid: DatasetGuid::parse(guid.trim()).expect("the object's GUID is canonical"),
                createtxg: group,
                name: String::from(
                    name.split_once(separator)
                        .expect("the object names its dataset")
                        .1,
                ),
            });
        }
    }
    let home: std::fs::File =
        open_or_create_home(place.common.as_path()).expect("the record home opens");
    let record: InventoryRecord = InventoryRecord {
        kind: InventoryKind::Origin,
        pool_guid: guid_of(sbin, place.pool.as_str()),
        dataset: place.checkout_dataset.clone(),
        dataset_guid: guid_of(sbin, place.checkout_dataset.as_str()),
        bounds: origin_snapshot,
        bounds_guid,
        bounds_createtxg,
        entries,
    };
    publish_inventory(&home, SLOT, InventoryKind::Origin, &record)
        .expect("the origin inventory publishes once");
}

/// The record home's folder, for an arm that checks the records went.
pub(super) fn the_record_home(common: &Path) -> PathBuf {
    return home_folder(common);
}

/// What:
///  Write one unreadable destruction plan where a removal's plan for this clone
///  would stand,
///  in the folder and at the name the reader derives.
/// Why:
///  The state the project decided to refuse is a plan whose file cannot be read,
///  so the bytes are ones the grammar refuses and the name is the clone GUID's,
///  which is the only name the removal looks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = plantAnUnreadablePlan(common, cloneGuid);
/// ```
pub(super) fn plant_an_unreadable_plan(
    common: &Path,
    clone_guid: DatasetGuid,
) -> PathBuf {
    let folder: PathBuf = plans_folder(common);
    std::fs::create_dir_all(folder.as_path()).expect("the plan folder is made");
    std::fs::set_permissions(
        folder.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .expect("the plan folder holds the design's mode");
    let path: PathBuf = folder.join(clone_guid.file_name());
    std::fs::write(path.as_path(), "this file is no destruction plan\n")
        .expect("the unreadable plan is written");
    std::fs::set_permissions(
        path.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    )
    .expect("the unreadable plan holds a mode the reader accepts");
    return path;
}
