//! What:
//!  Plan step 12's arms in the throwaway guest,
//!  on the design's layout:
//!  one real Git registration and one real clone,
//!  the mounted and the unmounted class of the same slot,
//!  a record that disagrees with the mount,
//!  a pending add and a pending removal,
//!  the wrapper's `git worktree move` refusal beside Git's own answer,
//!  which is `Device or resource busy` while mounted and success while
//!  unmounted,
//!  and the whole removal of a slot a clone is bound to,
//!  once by the command and once behind a shell alias,
//!  with what it destroyed read from the guest's own pool and files.
//! Why:
//!  The kernel refuses a move only while the clone is mounted,
//!  so the unmounted arm is the one that shows why the wrapper needs a refusal
//!  of its own
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  the measurement's item 6).
//!  The arm runs only where `CLI_GIT_ZFS_SLOT_STATE_GUEST` names a disposable
//!  guest and prints `SKIP` everywhere else,
//!  so a host run that ran nothing says so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mounted = reportOf(readSlot(common, 'one', slot, { mounts, intents })); // mounted managed clone
//! const refusal = refuseMoveOfABoundSlot(common, slot); // bound, mounted or not
//! ```

use super::the_destruction_plan_a_control_plants::{
    ACloneAPlanNames,
    APlanAControlPlants,
    plant,
};
use super::zfs_slot_state_guest_fixture::{
    Place,
    class_of,
    createtxg_of,
    git,
    guid_of,
    intent_word,
    publish,
    publish_for_the_snapshot,
    readings_of,
};
use super::zfs_sweep_guest::{
    CHECKOUT_VARIABLE,
    NONCE,
    ORIGIN_PREFIX,
    POOL_VARIABLE,
    SBIN_VARIABLE,
    clone_the_checkouts_snapshot,
    command_ok,
    make_the_designs_layout,
    mount_request,
    prepare_the_store,
    required,
    rows_naming,
    sweep,
    unmount_at,
    write_configuration,
    write_passing_probe_record,
    zfs,
    zfs_ignored,
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
    binding_name,
    clone_inventory_name,
    home_folder,
    open_or_create_home,
    origin_inventory_name,
    plans_folder,
};
use git_policy_cli::worktree_clone_mount_protocol::DatasetGuid;
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY;
use git_policy_cli::worktree_clone_move_refusal::managed_move_refusal::{
    ManagedMoveRefusal,
    ManagedMoveRefusalCause,
    refuse_move_of_a_bound_slot,
};
use git_policy_cli::worktree_clone_slot_state::slot_state_classification::classify_slot;
use git_policy_cli::worktree_clone_slot_state::slot_state_readings::SlotReadings;
use git_policy_cli::worktree_clone_slot_state::slot_state_report::report_of;
use git_policy_cli::worktree_clone_unmount::staging_folder_name::staging_folder_name;
use git_policy_cli::worktree_clone_unmount::user_cache_pin::{
    USER_CACHE_PIN_PATH,
    UserCachePin,
    passwd_home_of,
};
use git_policy_cli::worktree_clone_unmount::user_cache_pin_record::write_pin;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Output;

/// The variable the guest driver sets for these arms.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_SLOT_STATE_GUEST";

/// The slot this arm registers,
/// clones and moves.
const SLOT: &str = "one";

/// The second slot a pending add's record is filed for.
const SLOT_TWO: &str = "two";

/// The slot the removal step's first scene fills and removes.
const SLOT_FOR_THE_FIRST_REMOVAL: &str = "removal";

/// The slot the removal step's second scene fills and removes behind a shell
/// alias.
const SLOT_FOR_THE_ALIASED_REMOVAL: &str = "aliased-removal";

/// The nonce the first scene's snapshot,
/// mount request and unmount request carry.
/// Why:
///  A removal destroys the checkout snapshot its clone was made from,
///  so each scene takes a snapshot and a request number of its own.
const NONCE_FOR_THE_FIRST_REMOVAL: &str = "1a2b3c4d5e6f708192a3b4c5d6e7f809";

/// The nonce the second scene's snapshot,
/// mount request and unmount request carry.
const NONCE_FOR_THE_ALIASED_REMOVAL: &str = "2b3c4d5e6f708192a3b4c5d6e7f8091a";

/// Whether this run is inside the guest.
fn in_the_guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!(
            "SKIP worktree_clone_slot_state::guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest"
        );
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname")
        .map(|text| return text.trim().to_string())
        .unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// One checkout path as the argument Git takes.
fn shown(path: &Path) -> &str {
    return path.to_str().expect("the guest's paths are UTF-8");
}

/// The wrapper program the driver built,
/// which the arm reaches through a `git` link so it runs as the person's `git`.
const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// What:
///  Run one Git command through the built wrapper,
///  the way a person's shell starts it.
/// Why:
///  Plan step 12's wiring is the command path itself,
///  so the arm proves the refusal,
///  the restore and the report a person meets rather than the function behind
///  them.
fn wrapper_git(
    checkout: &Path,
    arguments: &[&str],
) -> Output {
    let bin: PathBuf = PathBuf::from("/root/slot-state-bin");
    std::fs::create_dir_all(bin.as_path()).expect("the wrapper's bin folder");
    let home: PathBuf = PathBuf::from("/root/slot-state-home");
    std::fs::create_dir_all(home.as_path()).expect("the wrapper's home folder");
    let runtime: PathBuf = PathBuf::from("/root/slot-state-runtime");
    std::fs::create_dir_all(runtime.as_path()).expect("the wrapper's runtime folder");
    let link: PathBuf = bin.join("git");
    let _stale: std::io::Result<()> = std::fs::remove_file(link.as_path());
    std::os::unix::fs::symlink(WRAPPER, link.as_path()).expect("the wrapper's git link");
    return std::process::Command::new(link.as_path())
        .args(arguments)
        .current_dir(checkout)
        .env_clear()
        .env(
            "PATH",
            format!(
                "{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
                bin.display()
            ),
        )
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

/// One command's whole answer as text.
fn said_of(output: &Output) -> String {
    return format!(
        "{}{}",
        String::from_utf8_lossy(output.stdout.as_slice()),
        String::from_utf8_lossy(output.stderr.as_slice())
    );
}

/// One scene the removal step runs on:
/// where its slot stands,
/// the clone the record names,
/// and the facts the removal must leave gone.
/// Why:
///  A finished removal destroys its own scene,
///  so the clone's GUID,
///  the snapshot it was made from and the checkout it stands beside are read
///  before the command and asserted afterwards.
struct AScene {
    /// Where the slot,
    /// its registration and its record stand.
    place: Place,
    /// The clone dataset the record names.
    clone: String,
    /// The clone's GUID,
    /// as the pool answered before the removal.
    clone_guid: DatasetGuid,
    /// The checkout's snapshot the clone was made from,
    /// which the removal's origin attempt destroys.
    origin_snapshot: String,
    /// The container the clone was made in.
    container: String,
    /// The checkout's dataset,
    /// which the removal keeps.
    checkout_dataset: String,
    /// The OpenZFS userspace every read of the guest's pool runs.
    sbin: String,
}

/// What:
///  The fixture one removal scene is made from.
/// Why:
///  Two scenes are made,
///  each under a nonce of its own,
///  and the facts they share travel as one value so neither call site can pass
///  them in another order.
struct ASceneToMake<'a> {
    /// The checkout the slot is registered in.
    checkout: &'a Path,
    /// The pool the clone is made in.
    pool: &'a str,
    /// The OpenZFS userspace the arm runs.
    sbin: &'a str,
    /// The checkout's dataset.
    checkout_dataset: &'a str,
    /// The container the clone is made in.
    container: &'a str,
    /// The slot's name.
    slot_name: &'a str,
    /// The nonce the scene's snapshot and both of the arm's requests carry.
    nonce: &'a str,
}

impl ASceneToMake<'_> {
    /// The account's cache folder,
    /// reached from the home of the account that owns the checkout.
    /// Why:
    ///  The handler reaches the cache from the account's home and never from a
    ///  variable of this process,
    ///  so the fixture reads it the same way.
    fn the_accounts_cache(&self) -> PathBuf {
        let owner: u32 = std::fs::metadata(self.checkout)
            .expect("the checkout's owner reads")
            .uid();
        let home: PathBuf = passwd_home_of(owner).expect("the checkout's account has a home");
        return home.join(".cache");
    }

    /// What:
    ///  The account's cache dataset and the pin that names it,
    ///  which the mount handler reads before it unmounts anything.
    /// Why:
    ///  The handler answers no unmount without that pin,
    ///  and the one-time setup plan step 19 owns is what writes it:
    ///  nothing in this release writes one,
    ///  so a launch that never ran that setup answers a removal's unmount
    ///  request with a refusal and no removal finishes there.
    ///  The record is the producer's own,
    ///  written through its writer.
    ///
    /// # Panics
    /// The account whose checkout this is,
    /// its home,
    /// the dataset,
    /// or the pin that could not be made.
    fn pin_the_accounts_cache(&self) {
        let cache: String = format!("{}/cache", self.pool);
        let mountpoint: PathBuf = self.the_accounts_cache();
        let (stands, _said, _error): (bool, String, String) = zfs(
            self.sbin,
            "zfs",
            &["list", "-H", "-o", "name", cache.as_str()],
            None,
        );
        if !stands {
            zfs_ok(
                self.sbin,
                &[
                    "create",
                    "-o",
                    format!("mountpoint={}", mountpoint.display()).as_str(),
                    cache.as_str(),
                ],
            );
        }
        let facts: std::fs::Metadata =
            std::fs::metadata(mountpoint.as_path()).expect("the cache's own folder reads");
        let folder: PathBuf = Path::new(USER_CACHE_PIN_PATH)
            .parent()
            .expect("the pin names a folder")
            .to_path_buf();
        std::fs::create_dir_all(folder.as_path()).expect("the pin's folder is made");
        let held: std::fs::File =
            std::fs::File::open(folder.as_path()).expect("the pin's folder opens");
        write_pin(
            &held,
            &UserCachePin {
                source: cache,
                root_inode: facts.ino(),
                device: facts.dev(),
            },
        )
        .expect("the pin writes");
    }

    /// What:
    ///  The empty staging folder the handler binds over before it moves the
    ///  clone's mount,
    ///  at the account's cache and under the clone's GUID.
    /// Why:
    ///  The handler's unmount turn opens that folder before anything else and
    ///  answers `cancelled` at `staging-folder` when it is absent,
    ///  and the design gives its creation to the wrapper:
    ///  `worktree_clone_unmount::staging_cleanup::create_the_staging_folder`
    ///  carries that duty and this release calls it from nowhere,
    ///  which this round reports from the guest's own receipt.
    ///  The scene makes the folder the design says the wrapper makes,
    ///  so the order this step asserts can be reached at all,
    ///  and a round that wires the creation in deletes this.
    ///
    /// # Panics
    /// The folder that could not be made or given its mode.
    fn make_the_staging_folder(
        &self,
        clone_guid: DatasetGuid,
    ) {
        let staging: PathBuf = self
            .the_accounts_cache()
            .join(staging_folder_name(clone_guid));
        std::fs::create_dir(staging.as_path()).expect("the scene's staging folder is made");
        std::fs::set_permissions(staging.as_path(), std::fs::Permissions::from_mode(0o700))
            .expect("the staging folder holds the route's mode");
    }

    /// What:
    ///  Publish the origin-side inventory the add records for this scene's
    ///  slot:
    ///  the checkout's snapshots at or below the origin snapshot's transaction
    ///  group,
    ///  and every bookmark it holds,
    ///  through the add's own writer.
    /// Why:
    ///  The removal's third check compares the clone's live GUIDs with this
    ///  record,
    ///  so the scene carries the add's own shape rather than a hand-written
    ///  list.
    ///
    /// # Panics
    /// The pool's listing,
    /// a GUID that is not canonical,
    /// or a publication that refused.
    fn publish_the_origin_inventory(
        &self,
        place: &Place,
    ) {
        let sbin: &str = self.sbin;
        let slot: &str = self.slot_name;
        let snapshot_suffix: &str = self.nonce;
        let origin_snapshot: String = format!(
            "{}@{ORIGIN_PREFIX}{snapshot_suffix}",
            place.checkout_dataset
        );
        let bounds_createtxg: u64 = createtxg_of(sbin, origin_snapshot.as_str());
        let mut entries: Vec<InventoryEntry> = Vec::new();
        let (listed_ok, listed, listed_error): (bool, String, String) = zfs(
            sbin,
            "zfs",
            &[
                "list",
                "-H",
                "-p",
                "-o",
                "name,guid,createtxg",
                "-t",
                "snapshot",
                "-r",
                place.checkout_dataset.as_str(),
            ],
            None,
        );
        assert!(
            listed_ok,
            "the checkout's snapshots list: {listed}{listed_error}"
        );
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
                .expect("a snapshot's group is a number");
            if group > bounds_createtxg {
                continue;
            }
            entries.push(InventoryEntry {
                kind: EntryKind::Snapshot,
                guid: DatasetGuid::parse(guid.trim()).expect("a snapshot's GUID is canonical"),
                createtxg: group,
                name: String::from(
                    name.split_once('@')
                        .expect("a snapshot names its dataset")
                        .1,
                ),
            });
        }
        let (bookmarks_ok, bookmarks, bookmarks_error): (bool, String, String) = zfs(
            sbin,
            "zfs",
            &[
                "list",
                "-H",
                "-p",
                "-o",
                "name,guid,createtxg",
                "-t",
                "bookmark",
                "-r",
                place.checkout_dataset.as_str(),
            ],
            None,
        );
        assert!(
            bookmarks_ok,
            "the checkout's bookmarks list: {bookmarks}{bookmarks_error}"
        );
        for line in bookmarks.lines() {
            let mut fields = line.split('\t');
            let (Some(name), Some(guid), Some(createtxg)) =
                (fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            entries.push(InventoryEntry {
                kind: EntryKind::Bookmark,
                guid: DatasetGuid::parse(guid.trim()).expect("a bookmark's GUID is canonical"),
                createtxg: createtxg
                    .trim()
                    .parse()
                    .expect("a bookmark's group is a number"),
                name: String::from(
                    name.split_once('#')
                        .expect("a bookmark names its dataset")
                        .1,
                ),
            });
        }
        let home: std::fs::File =
            open_or_create_home(place.common.as_path()).expect("the record home opens");
        let record: InventoryRecord = InventoryRecord {
            kind: InventoryKind::Origin,
            pool_guid: guid_of(sbin, place.pool.as_str()),
            dataset: place.checkout_dataset.clone(),
            dataset_guid: guid_of(sbin, place.checkout_dataset.as_str()),
            bounds: origin_snapshot.clone(),
            bounds_guid: guid_of(sbin, origin_snapshot.as_str()),
            bounds_createtxg,
            entries,
        };
        publish_inventory(&home, slot, InventoryKind::Origin, &record)
            .expect("the origin inventory publishes once");
    }

    /// What:
    ///  One slot registered,
    ///  cloned,
    ///  attached,
    ///  recorded and unmounted again by
    ///  the handler,
    ///  which is the state the removal step runs on.
    /// Why:
    ///  The clone is attached by the sweep and unmounted by the handler's own
    ///  request,
    ///  because a removal asks the handler for an unmount and the handler
    ///  answers that request only for a clone it unmounted itself;
    ///  nothing mounted at the slot is what the class this step is about says.
    ///  The account's cache is pinned first,
    ///  because the handler stages the unmount inside it.
    ///
    /// # Panics
    /// A registration,
    /// a clone,
    /// an attach,
    /// a record,
    /// the cache's pin or the unmount that did not happen.
    fn make(&self) -> AScene {
        let common: PathBuf = self.checkout.join(".git");
        let slot: PathBuf = self
            .checkout
            .join(".claude")
            .join("worktrees")
            .join(self.slot_name);
        command_ok(
            "git",
            &[
                "-C",
                shown(self.checkout),
                "worktree",
                "add",
                "--quiet",
                "--no-checkout",
                shown(slot.as_path()),
                "HEAD",
            ],
        );
        let slot_identity: FolderIdentity = FolderIdentity::of_folder(slot.as_path())
            .expect("the scene's underlay identity reads before the mount");
        let pointer: Vec<u8> = std::fs::read(slot.join(".git"))
            .expect("the scene's underlay pointer reads before the mount");
        let clone: String = clone_the_checkouts_snapshot(
            self.sbin,
            self.checkout_dataset,
            self.container,
            self.slot_name,
            self.nonce,
            Some(mount_request(self.nonce).as_str()),
        );
        let clone_guid: DatasetGuid = guid_of(self.sbin, clone.as_str());
        let place: Place = Place {
            common: common.clone(),
            pool: String::from(self.pool),
            checkout_dataset: String::from(self.checkout_dataset),
            clone_dataset: clone.clone(),
            slot: slot.clone(),
            slot_identity,
            pointer,
            registration: common.join("worktrees").join(self.slot_name),
        };
        self.pin_the_accounts_cache();
        self.make_the_staging_folder(clone_guid);
        let attached = sweep();
        assert_eq!(
            attached.status,
            Some(0),
            "the sweep attached the scene's clone at `{}`: {}",
            self.slot_name,
            attached.said
        );
        publish_for_the_snapshot(&place, Phase::Filled, self.sbin, self.slot_name, self.nonce);
        self.publish_the_origin_inventory(&place);
        // The arm asks the handler for the unmount, under the same nonce: a receipt is matched on the dataset,
        //  the operation and the nonce, so one nonce may name both of a scene's requests.
        let intent: String = format!("v1:{}:unmount", self.nonce);
        zfs_ok(
            self.sbin,
            &[
                "set",
                format!("{DESIRED_STATE_PROPERTY}={intent}").as_str(),
                clone.as_str(),
            ],
        );
        let unmounted = sweep();
        assert_eq!(
            unmounted.status,
            Some(0),
            "the handler answered the scene's unmount request at `{}`: {}",
            self.slot_name,
            unmounted.said
        );
        assert!(
            rows_naming(clone.as_str()).is_empty(),
            "the scene's clone is unmounted before the removal runs: {}",
            unmounted.said
        );
        let readings: SlotReadings = readings_of(&place, self.slot_name, slot.as_path());
        let class: String = String::from(class_of(&classify_slot(&readings)));
        assert_eq!(
            class,
            "known unmounted clone",
            "the scene's slot is a known unmounted clone: {:?}",
            report_of(&readings).lines()
        );
        println!(
            "SLOT-STATE scene slot={} class={class} intent={}",
            self.slot_name,
            intent_word(&readings.intent)
        );
        return AScene {
            place,
            clone,
            clone_guid,
            origin_snapshot: format!("{}@{ORIGIN_PREFIX}{}", self.checkout_dataset, self.nonce),
            container: String::from(self.container),
            checkout_dataset: String::from(self.checkout_dataset),
            sbin: String::from(self.sbin),
        };
    }
}

/// What:
///  One slot registered,
///  cloned,
///  attached,
///  recorded and unmounted again by
///  the handler,
///  which is the state the removal step runs on.
/// Why:
///  The scene is made from the facts of the guest's own pool and checkout,
///  and each scene takes a snapshot and a request number of its own.
fn a_known_unmounted_clone(scene: ASceneToMake<'_>) -> AScene {
    return scene.make();
}

impl AScene {
    /// What:
    ///  Prove one finished removal destroyed what `SPEC.md`'s "Removal of a
    ///  ZFS worktree clone" says it destroys,
    ///  from the guest's own pool and files after the command.
    /// Why:
    ///  The command's own words are the report,
    ///  and what they claim is what the pool and the files answer:
    ///  the clone's name and its GUID under the container,
    ///  the checkout's snapshot the clone was made from,
    ///  the registration,
    ///  the slot's folder,
    ///  and the removal's own records.
    ///  The checkout's dataset stands besides them,
    ///  because what the checkout kept is half of what the report says.
    fn assert_the_removal_destroyed(
        &self,
        removal: &Output,
    ) {
        let said: String = said_of(removal);
        let slot: String = String::from(
            self.place
                .slot
                .file_name()
                .expect("the slot names a folder")
                .to_string_lossy(),
        );
        assert_eq!(
            removal.status.code(),
            Some(0),
            "a finished removal is no failure: {said}"
        );
        assert!(
            said.contains(&format!(
                "the ZFS worktree clone at `{slot}` (dataset `{}`) is destroyed",
                self.clone
            )),
            "{said}"
        );
        assert!(
            said.contains("the clone's head destroy answered and its absence is proved."),
            "{said}"
        );
        assert!(
            said.contains(&format!(
                "the origin cleanup destroyed the snapshot `{}`",
                self.origin_snapshot
            )),
            "{said}"
        );
        assert!(
            said.contains("the Git registration and the slot's pointer were removed, the leftover underlay folder was removed, and the managed lock was released."),
            "{said}"
        );
        assert!(
            said.contains("the removal's plan, its binding and both inventories are removed"),
            "{said}"
        );
        assert!(
            said.contains(&format!(
                "the slot's per-slot claim is released, so nothing of this removal keeps another command off `{slot}`."
            )),
            "{said}"
        );
        assert!(
            !self.stands_in_the_pool(self.clone.as_str()),
            "the clone's name answered the pool after the removal"
        );
        let left_under_the_container: String = self.the_pool_below(self.container.as_str());
        assert!(
            !left_under_the_container.contains(self.clone_guid.file_name().as_str()),
            "no dataset under `{}` carries the clone's GUID {}: {left_under_the_container}",
            self.container,
            self.clone_guid.file_name()
        );
        assert!(
            !self.stands_in_the_pool(self.origin_snapshot.as_str()),
            "the checkout snapshot the clone was made from answered the pool after the removal"
        );
        assert!(
            self.stands_in_the_pool(self.checkout_dataset.as_str()),
            "the checkout the clone was made beside still stands"
        );
        assert!(
            !self.place.registration.exists(),
            "the registration is removed"
        );
        assert!(
            !self.place.slot.exists(),
            "the slot's folder is gone with the pointer it held"
        );
        let home: PathBuf = home_folder(self.place.common.as_path());
        for name in [
            binding_name(slot.as_str()),
            origin_inventory_name(slot.as_str()),
            clone_inventory_name(slot.as_str()),
        ] {
            assert!(
                !home.join(name.as_str()).exists(),
                "the removal's record `{name}` is removed"
            );
        }
        assert!(
            !plans_folder(self.place.common.as_path())
                .join(self.clone_guid.file_name())
                .exists(),
            "no destruction plan naming the clone stands after a finished removal"
        );
    }

    /// Whether the pool still answers for one dataset or snapshot name.
    fn stands_in_the_pool(
        &self,
        name: &str,
    ) -> bool {
        let (answered, _said, _error): (bool, String, String) = zfs(
            self.sbin.as_str(),
            "zfs",
            &["get", "-H", "-o", "value", "guid", name],
            None,
        );
        return answered;
    }

    /// Every name and GUID the pool lists below one dataset.
    fn the_pool_below(
        &self,
        dataset: &str,
    ) -> String {
        let (listed, said, error): (bool, String, String) = zfs(
            self.sbin.as_str(),
            "zfs",
            &["list", "-H", "-p", "-o", "name,guid", "-r", dataset],
            None,
        );
        assert!(listed, "the pool lists below `{dataset}`: {said}{error}");
        return said;
    }
}

/// What:
///  The command one removal step runs through the built wrapper.
/// Why:
///  The two spellings this step proves differ only in the words the wrapper reads,
///  so the sweeper that answers the removal's unmount request is started once
///  for both.
struct ARemovalToRun<'a> {
    /// The checkout the command runs in.
    checkout: &'a Path,
    /// The command's own words.
    arguments: &'a [&'a str],
}

/// What:
///  Run one removal through the built wrapper with a sweeper running beside it.
/// Why:
///  The removal files one `unmount` request on the clone and waits for the
///  handler's receipt,
///  and this arm starts no handler of its own,
///  so a sweep loop answers that request and ends with the command.
///
/// # Panics
/// The sweeper that could not start.
fn the_removal_beside_a_sweeper(removal: ARemovalToRun<'_>) -> Output {
    let sweeper_binary: PathBuf = super::support_installed_name::installed_name("sweep");
    let mut sweeper: std::process::Child = std::process::Command::new("sh")
        .args([
            "-c",
            "while :; do \"$0\" >/dev/null 2>&1; sleep 1; done",
            shown(sweeper_binary.as_path()),
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("the sweeper starts under the installed name");
    let answer: Output = wrapper_git(removal.checkout, removal.arguments);
    let _ended: std::io::Result<std::process::ExitStatus> =
        sweeper.kill().and_then(|()| return sweeper.wait());
    return answer;
}

/// What:
///  Plan step 12's arms in the guest.
/// Why:
///  Each class the host controls prove on scripted readings is proved once on a
///  real pool,
///  and the refusal is shown beside Git's own answer in both arms.
#[test]
fn the_slots_class_and_the_move_refusal_on_a_real_pool() {
    if !in_the_guest() {
        return;
    }
    let pool: String = required(POOL_VARIABLE);
    let sbin: String = required(SBIN_VARIABLE);
    let checkout: PathBuf = PathBuf::from(required(CHECKOUT_VARIABLE));
    let (checkout_dataset, container): (String, String) =
        make_the_designs_layout(sbin.as_str(), pool.as_str(), checkout.as_path());
    let worktrees: PathBuf = checkout.join(".claude").join("worktrees");
    std::fs::create_dir_all(worktrees.as_path()).expect("the slot parent is made");
    // A real repository and real registrations: the move this arm refuses is Git's own, not a fixture's.
    command_ok("git", &["init", "--quiet", shown(checkout.as_path())]);
    command_ok(
        "git",
        &[
            "-c",
            "user.name=the arm",
            "-c",
            "user.email=arm@example.invalid",
            "-C",
            shown(checkout.as_path()),
            "commit",
            "--quiet",
            "--allow-empty",
            "--message",
            "the arm's base",
        ],
    );
    let slot: PathBuf = worktrees.join(SLOT);
    let second: PathBuf = worktrees.join(SLOT_TWO);
    for place in [&slot, &second] {
        command_ok(
            "git",
            &[
                "-C",
                shown(checkout.as_path()),
                "worktree",
                "add",
                "--quiet",
                "--no-checkout",
                shown(place.as_path()),
                "HEAD",
            ],
        );
    }
    let slot_identity: FolderIdentity = FolderIdentity::of_folder(slot.as_path())
        .expect("the underlay's identity reads before the mount");
    // The pointer the add keeps for prune is the underlay's own, and the clone's copy replaces it under the mount.
    let pointer: Vec<u8> =
        std::fs::read(slot.join(".git")).expect("the underlay pointer reads before the mount");
    let second_pointer: Vec<u8> =
        std::fs::read(second.join(".git")).expect("the second slot's underlay pointer reads");
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
    let place: Place = Place {
        common: common.clone(),
        pool: pool.clone(),
        checkout_dataset: checkout_dataset.clone(),
        clone_dataset: clone.clone(),
        slot: slot.clone(),
        slot_identity,
        pointer,
        registration: common.join("worktrees").join(SLOT),
    };
    let attached = sweep();
    assert_eq!(
        attached.status,
        Some(0),
        "the sweep attached the clone: {}",
        attached.said
    );
    // The add's own exchange, measured as item 1's order: the clone's copy of the checkout's administration is moved
    // aside under the mount and the registration's pointer installed in its place, so the slot Git validates is the
    // clone's root rather than the copied repository.
    std::fs::rename(slot.join(".git"), slot.join("git-administered-copy"))
        .expect("the copied administration moves aside");
    std::fs::write(
        slot.join(".git"),
        format!("gitdir: {}\n", place.registration.display()),
    )
    .expect("the registration pointer is installed under the mount");
    let two: Place = Place {
        common: common.clone(),
        pool: pool.clone(),
        checkout_dataset: checkout_dataset.clone(),
        clone_dataset: format!("{container}/{SLOT_TWO}"),
        slot: second.clone(),
        slot_identity: FolderIdentity::of_folder(second.as_path())
            .expect("the second slot's identity reads"),
        pointer: second_pointer,
        registration: common.join("worktrees").join(SLOT_TWO),
    };

    // The mounted class, the report, and a record that disagrees with the mount.
    publish(&place, Phase::Filled, sbin.as_str(), SLOT);
    let mounted: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    let mounted_report = report_of(&mounted);
    assert_eq!(
        class_of(&classify_slot(&mounted)),
        "mounted managed clone",
        "{:?}",
        mounted_report.lines()
    );
    assert!(
        mounted_report
            .next_safe_action
            .contains("`git worktree move` is refused"),
        "{}",
        mounted_report.next_safe_action
    );
    println!(
        "SLOT-STATE mounted class={} dataset={}",
        mounted_report.state, mounted_report.dataset
    );
    publish(&place, Phase::Cloned, sbin.as_str(), SLOT);
    let behind: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    assert_eq!(class_of(&classify_slot(&behind)), "cannot be proved");
    println!(
        "SLOT-STATE record-behind-mount class={}",
        class_of(&classify_slot(&behind))
    );
    publish(&place, Phase::Filled, sbin.as_str(), SLOT);

    // Mounted: the wrapper refuses and Git's own move meets the kernel's refusal.
    let bound: ManagedMoveRefusal = refuse_move_of_a_bound_slot(common.as_path(), slot.as_path())
        .expect("a bound slot refuses");
    assert!(
        matches!(bound.cause, ManagedMoveRefusalCause::Bound { .. }),
        "{:?}",
        bound.cause
    );
    println!("MOVE-REFUSAL mounted cause=bound");
    let moved: PathBuf = worktrees.join("moved");
    let refused: Output = git(
        checkout.as_path(),
        &[
            "worktree",
            "move",
            shown(slot.as_path()),
            shown(moved.as_path()),
        ],
    );
    let said: String = format!(
        "{}{}",
        String::from_utf8_lossy(refused.stdout.as_slice()),
        String::from_utf8_lossy(refused.stderr.as_slice())
    );
    assert!(
        !refused.status.success(),
        "a mounted clone's move is refused by the kernel: {said}"
    );
    assert!(
        said.to_lowercase().contains("busy"),
        "the refusal is the busy mount: {said}"
    );
    println!("MOVE-REFUSAL mounted git=refused-busy");
    // The wrapper's own command path refuses the same move before Git runs.
    let by_the_wrapper: Output = wrapper_git(
        checkout.as_path(),
        &[
            "worktree",
            "move",
            shown(slot.as_path()),
            shown(moved.as_path()),
        ],
    );
    assert_eq!(
        by_the_wrapper.status.code(),
        Some(1),
        "the wrapper's refusal status: {}",
        said_of(&by_the_wrapper)
    );
    assert!(
        said_of(&by_the_wrapper).contains("`git worktree move` is refused"),
        "{}",
        said_of(&by_the_wrapper)
    );
    assert!(slot.is_dir(), "the wrapper moved nothing");
    println!("MOVE-REFUSAL wrapper-mounted cause=bound status=refused");

    // Unmounted: the kernel no longer refuses and Git's own move succeeds, which is the hole the wrapper closes.
    unmount_at(slot.as_path());
    let unmounted: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    assert_eq!(
        class_of(&classify_slot(&unmounted)),
        "known unmounted clone"
    );
    assert_eq!(intent_word(&unmounted.intent), "mount-requested");
    println!(
        "SLOT-STATE unmounted class={} intent={}",
        class_of(&classify_slot(&unmounted)),
        intent_word(&unmounted.intent)
    );
    let unmounted_refusal: ManagedMoveRefusal =
        refuse_move_of_a_bound_slot(common.as_path(), slot.as_path())
            .expect("the unmounted slot refuses too");
    assert!(matches!(
        unmounted_refusal.cause,
        ManagedMoveRefusalCause::Bound { .. }
    ));
    println!("MOVE-REFUSAL unmounted cause=bound");
    let refused_unmounted: Output = wrapper_git(
        checkout.as_path(),
        &[
            "worktree",
            "move",
            shown(slot.as_path()),
            shown(moved.as_path()),
        ],
    );
    assert_eq!(
        refused_unmounted.status.code(),
        Some(1),
        "the wrapper refuses the unmounted move: {}",
        said_of(&refused_unmounted)
    );
    assert!(
        said_of(&refused_unmounted).contains("`git worktree move` is refused"),
        "{}",
        said_of(&refused_unmounted)
    );
    assert!(
        slot.is_dir(),
        "the wrapper moved nothing while the clone was unmounted"
    );
    println!("MOVE-REFUSAL wrapper-unmounted cause=bound status=refused");
    let allowed: Output = git(
        checkout.as_path(),
        &[
            "worktree",
            "move",
            shown(slot.as_path()),
            shown(moved.as_path()),
        ],
    );
    assert!(
        allowed.status.success(),
        "Git's own move succeeds while the clone is unmounted: {}{}",
        String::from_utf8_lossy(allowed.stdout.as_slice()),
        String::from_utf8_lossy(allowed.stderr.as_slice())
    );
    println!("MOVE-REFUSAL unmounted git=succeeded");
    command_ok(
        "git",
        &[
            "-C",
            shown(checkout.as_path()),
            "worktree",
            "move",
            shown(moved.as_path()),
            shown(slot.as_path()),
        ],
    );

    // The removal of a slot a whole managed clone is bound to is its own step at the end of this arm: it destroys
    //  the clone and the registration, so the prune and repair steps below keep the slot the move arms made.

    // The pointer the add keeps for prune: real Git's own prune keeps the registration while the pointer stands.
    let pruned_intact: Output = git(checkout.as_path(), &["worktree", "prune", "--verbose"]);
    assert!(
        pruned_intact.status.success(),
        "prune ran: {}",
        said_of(&pruned_intact)
    );
    assert!(
        place.registration.is_dir(),
        "the registration stands after a prune with the pointer intact"
    );
    assert!(
        slot.join(".git").is_file(),
        "the underlay pointer stands after the prune"
    );
    println!("PRUNE-POINTER intact registration=kept");

    // A pointer an account deleted: the wrapper's prune writes the recorded bytes back before Git starts, and the
    // registration it would have dropped stands.
    std::fs::remove_file(slot.join(".git")).expect("the underlay pointer goes");
    let restored: Output = wrapper_git(checkout.as_path(), &["worktree", "prune"]);
    assert!(
        restored.status.success(),
        "the wrapped prune ran: {}",
        said_of(&restored)
    );
    assert!(
        said_of(&restored).contains("restored the pointer"),
        "{}",
        said_of(&restored)
    );
    assert!(
        place.registration.is_dir(),
        "the registration stands after the restore"
    );
    assert_eq!(
        std::fs::read(slot.join(".git")).expect("the restored pointer reads"),
        place.pointer
    );
    println!("PRUNE-POINTER restored registration=kept");

    // A damaged link with the registration surviving: the report says what a repair restores, and Git rebuilds it.
    std::fs::remove_file(slot.join(".git")).expect("the underlay pointer goes");
    let repaired: Output = wrapper_git(
        checkout.as_path(),
        &["worktree", "repair", shown(slot.as_path())],
    );
    assert!(
        said_of(&repaired).contains("link: damaged, and the registration stands"),
        "{}",
        said_of(&repaired)
    );
    assert!(
        said_of(&repaired).contains("next safe command: `git worktree repair"),
        "{}",
        said_of(&repaired)
    );
    assert!(
        slot.join(".git").is_file(),
        "Git rebuilt the pointer from the registration"
    );
    println!("REPAIR damaged registration=stands");

    // A pending add at a slot the add has not mounted, and a pending removal whose plan names this clone.
    publish(&two, Phase::Snapshotted, sbin.as_str(), SLOT_TWO);
    let adding: SlotReadings = readings_of(&two, SLOT_TWO, second.as_path());
    assert_eq!(class_of(&classify_slot(&adding)), "pending add");
    println!(
        "SLOT-STATE pending-add class={}",
        class_of(&classify_slot(&adding))
    );
    // The removal's own plan, published by the removal's own writer: the state its first write leaves, with the
    // head present and every line planned. The arm made this clone and never snapshotted it, so its inventory is empty.
    let plan: APlanAControlPlants = APlanAControlPlants::a_removals_first_write(ACloneAPlanNames {
        clone_createtxg: createtxg_of(sbin.as_str(), clone.as_str()),
        origin_dataset: checkout_dataset.clone(),
        origin_snapshot: format!("{checkout_dataset}@{ORIGIN_PREFIX}{NONCE}"),
        ..ACloneAPlanNames::a_fixture_clone(
            common.as_path(),
            clone.as_str(),
            guid_of(sbin.as_str(), clone.as_str()),
        )
    });
    let plan_path: PathBuf = plant(&plan);
    let removing: SlotReadings = readings_of(&place, SLOT, slot.as_path());
    assert_eq!(class_of(&classify_slot(&removing)), "pending removal");
    println!(
        "SLOT-STATE pending-removal class={}",
        class_of(&classify_slot(&removing))
    );

    // The plan goes first, so the prune below reads a slot no removal is in flight for.
    std::fs::remove_file(plan_path.as_path()).expect("the plan is removed");

    // A registration Git already pruned: the report says what is lost, and a pointer restored from the record points at
    // nothing, because Git does not rebuild a registration.
    std::fs::remove_file(slot.join(".git")).expect("the underlay pointer goes");
    let pruned: Output = git(checkout.as_path(), &["worktree", "prune"]);
    assert!(pruned.status.success(), "prune ran: {}", said_of(&pruned));
    assert!(!place.registration.exists(), "the registration is gone");
    let pruned_report: Output = wrapper_git(
        checkout.as_path(),
        &["worktree", "repair", shown(slot.as_path())],
    );
    assert!(
        said_of(&pruned_report).contains("link: pruned, and the registration is gone"),
        "{}",
        said_of(&pruned_report)
    );
    assert!(
        said_of(&pruned_report).contains("cannot rebuild a registration"),
        "{}",
        said_of(&pruned_report)
    );
    println!("REPAIR pruned registration=gone");

    // The first fixture is done with, and its clone is this arm's own debris: it stands no longer, so a sweep of the
    //  scenes below attaches what they make and nothing else.
    zfs_ok(sbin.as_str(), &["destroy", "-f", clone.as_str()]);

    // What a removal of a slot the arm has just classified a known unmounted clone does: `SPEC.md`'s "Removal of a
    //  ZFS worktree clone" is one order, `git worktree remove` of such a slot runs it, and that one command destroys the
    //  clone, its own snapshots, its records and its registration and prints what it destroyed. The scene is this
    //  step's own, because a removal destroys whatever it runs on and the steps above read the slot the move arms made.
    let removed: AScene = a_known_unmounted_clone(ASceneToMake {
        checkout: checkout.as_path(),
        pool: pool.as_str(),
        sbin: sbin.as_str(),
        checkout_dataset: checkout_dataset.as_str(),
        container: container.as_str(),
        slot_name: SLOT_FOR_THE_FIRST_REMOVAL,
        nonce: NONCE_FOR_THE_FIRST_REMOVAL,
    });
    let removal: Output = the_removal_beside_a_sweeper(ARemovalToRun {
        checkout: checkout.as_path(),
        arguments: &[
            "worktree",
            "remove",
            "--force",
            shown(removed.place.slot.as_path()),
        ],
    });
    removed.assert_the_removal_destroyed(&removal);
    println!("REMOVAL wrapper class=known-unmounted-clone status=removed clone=gone records=gone");

    // The same behind a shell alias: Git runs the alias's shell at the top of the working tree, and the operand is a
    //  path from there with a `.` step in it, so no registered path ends with it and the removal has to resolve it
    //  against the directory the alias runs the command in. The host controls hold that rule
    //  (`a_bound_slots_removal_reads_its_operand_where_git_runs_the_alias`); this arm repeats it once on a real pool.
    let through_the_alias: AScene = a_known_unmounted_clone(ASceneToMake {
        checkout: checkout.as_path(),
        pool: pool.as_str(),
        sbin: sbin.as_str(),
        checkout_dataset: checkout_dataset.as_str(),
        container: container.as_str(),
        slot_name: SLOT_FOR_THE_ALIASED_REMOVAL,
        nonce: NONCE_FOR_THE_ALIASED_REMOVAL,
    });
    let by_an_alias: Output = the_removal_beside_a_sweeper(ARemovalToRun {
        checkout: checkout.as_path(),
        arguments: &[
            "-c",
            "alias.rmwt=!git worktree remove --force",
            "rmwt",
            format!(".claude/worktrees/./{SLOT_FOR_THE_ALIASED_REMOVAL}").as_str(),
        ],
    });
    through_the_alias.assert_the_removal_destroyed(&by_an_alias);
    println!(
        "REMOVAL wrapper-through-a-shell-alias class=known-unmounted-clone status=removed clone=gone records=gone"
    );

    // The fixture's own cleanup: the base dataset, the account's cache the scene pinned, and the repository.
    let _unmounted = zfs(
        sbin.as_str(),
        "zfs",
        &["unmount", checkout_dataset.as_str()],
        None,
    );
    zfs_ignored(
        sbin.as_str(),
        &["destroy", "-r", "-f", format!("{pool}/step8").as_str()],
    );
    zfs_ignored(
        sbin.as_str(),
        &["destroy", "-r", "-f", format!("{pool}/cache").as_str()],
    );
    let _cleared: std::io::Result<()> = std::fs::remove_dir_all(checkout.as_path());
    println!("SLOT-STATE done");
}
