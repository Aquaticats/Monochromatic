//! What:
//!  The live pool and repository one guest arm builds inside the throwaway `mvm` guest:
//!  a file-backed pool,
//!  a checkout with its origin snapshot,
//!  a clone with its own snapshots,
//!  and the records an add would have left for it.
//! Why:
//!  A live arm opens `/dev/zfs`,
//!  so no control on a person's machine may run it against a pool it did not build,
//!  and every live arm of this folder needs the same fixture:
//!  one pool,
//!  one registration,
//!  one binding record and one origin-side inventory.
//!  Each arm names its own pool,
//!  image folder and guest variable,
//!  because two live arms run at once in one crate run and a shared pool would be one arm
//!  destroying the other's objects.
//!  A run on this machine has no fixture and prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = LivePoolFixture.fromEnvironment('an-arm', variables);
//! fixture.build(['worktree-own']);
//! fixture.writeTheAddsRecords(fixture.recorded());
//! ```

use crate::worktree_clone_binding_record::binding_encoding::{FolderIdentity, bytes_as_hexadecimal, path_as_hexadecimal};
use crate::worktree_clone_binding_record::binding_inventory::{EntryKind, InventoryEntry, InventoryKind, InventoryRecord};
use crate::worktree_clone_binding_record::binding_words::BINDING_HEADER;
use crate::worktree_clone_binding_record::registration_home::{binding_name, home_folder, origin_inventory_name};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::{LiveRemovalReads, RemovalPoolReads};
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::SnapshotDestroyDevice;
use monochromatic_zfs_request_to_openzfs_kernel_device::{DeviceError, NvList};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The slot every guest arm removes.
pub(crate) const SLOT: &str = "fix-flaky-test";
/// The image the pool stands on,
/// in gibibytes.
pub(crate) const POOL_IMAGE_SIZE: &str = "1G";
/// The account the item-12 grant set is written for.
pub(crate) const ACCOUNT: &str = "ubuntu";

/// What:
///  The four environment variables and the two defaults one arm's fixture takes.
/// Why:
///  The driver exports one set per arm,
///  and the arm refuses to run anywhere but the guest its own variable names,
///  so the names are the arm's rather than a constant of the shared fixture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const variables = { guest: 'CLI_GIT_..._GUEST', pool: '...', prefix: '...', root: '...', defaultRoot, defaultPool };
/// ```
pub(crate) struct PoolVariables {
    /// The variable naming the guest this arm may run in.
    pub(crate) guest: &'static str,
    /// The variable naming the pool the arm builds.
    pub(crate) pool: &'static str,
    /// The variable naming the OpenZFS prefix the guest carries.
    pub(crate) prefix: &'static str,
    /// The variable naming the folder the pool's image and the repository live under.
    pub(crate) root: &'static str,
    /// The guest directory the fixture lives in when the driver names none.
    pub(crate) default_root: &'static str,
    /// The pool the arm builds when the driver names none.
    pub(crate) default_pool: &'static str,
}

/// What:
///  The fixture one live arm may run in:
///  the guest's name,
///  the pool it builds and the paths its commands come from.
/// Why:
///  A live request opens `/dev/zfs`,
///  so the arm refuses to run anywhere but the guest the recipe names,
///  and it says by name why it skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fixture = LivePoolFixture.fromEnvironment('an-arm', variables);
/// ```
pub(crate) struct LivePoolFixture {
    /// The guest's host name.
    pub(crate) guest: String,
    /// The pool the arm builds and destroys.
    pub(crate) pool: String,
    /// The OpenZFS prefix whose `sbin` and `bin` the commands come from.
    pub(crate) prefix: PathBuf,
    /// The folder the pool's image and the repository live under.
    pub(crate) root: PathBuf,
    /// The origin snapshot's own name,
    /// which the add names with its prefix and its nonce and the earlier arms call `origin`.
    pub(crate) origin_name: String,
}

impl LivePoolFixture {
    /// The fixture the environment names,
    /// or nothing on a host run,
    /// where no pool command may be made.
    pub(crate) fn from_environment(test: &str, variables: &PoolVariables) -> Option<LivePoolFixture> {
        let Some(named) = std::env::var_os(variables.guest) else {
            println!("SKIPPED {test}: {} is not set, so this machine has no guest fixture and no live arm ran", variables.guest);
            return None;
        };
        let guest: String = named.to_string_lossy().into_owned();
        let host: String = Self::hostname();
        if host != guest {
            panic!("{test}: {} names the guest `{guest}` and this machine calls itself `{host}`, so a live arm would reach the wrong pool", variables.guest);
        }
        return Some(LivePoolFixture {
            guest,
            pool: std::env::var(variables.pool).unwrap_or_else(|_| return String::from(variables.default_pool)),
            prefix: PathBuf::from(std::env::var(variables.prefix).unwrap_or_else(|_| return String::from("/opt/openzfs-2.4.4"))),
            root: PathBuf::from(std::env::var(variables.root).unwrap_or_else(|_| return String::from(variables.default_root))),
            origin_name: String::from("origin"),
        });
    }

    /// What:
    ///  The same fixture whose checkout snapshots the add's own name,
    ///  which the origin cleanup record's grammar requires.
    /// Why:
    ///  The record is keyed by the snapshot's own name,
    ///  and an arm that measured the cleanup against a snapshot named `origin` would measure a
    ///  record the release refuses to write.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fixture = fixture.withOriginName('git-worktree-clone-origin-2f2f…');
    /// ```
    pub(crate) fn with_origin_name(mut self, name: &str) -> LivePoolFixture {
        self.origin_name = String::from(name);
        return self;
    }

    /// This machine's own host name.
    fn hostname() -> String {
        let output: Output = Command::new("hostname").output().expect("the guest has `hostname`");
        return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    }

    /// The `PATH` the guest's OpenZFS userspace needs.
    pub(crate) fn search_path(&self) -> String {
        return format!("{}:{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin", self.prefix.join("sbin").display(), self.prefix.join("bin").display());
    }

    /// One pool command,
    /// whose failure names the command and what it wrote.
    pub(crate) fn run(&self, program: &str, arguments: &[&str]) -> String {
        let output: Output = Command::new(program)
            .args(arguments)
            .env("PATH", self.search_path())
            .output()
            .unwrap_or_else(|error| panic!("`{program} {}` could not start in the guest: {error}", arguments.join(" ")));
        assert!(
            output.status.success(),
            "`{program} {}` exited {}: {}{}",
            arguments.join(" "),
            output.status,
            String::from_utf8_lossy(output.stdout.as_slice()),
            String::from_utf8_lossy(output.stderr.as_slice())
        );
        return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    }

    /// One pool command whose answer is printed whole,
    /// whether it worked or not.
    pub(crate) fn ask(&self, program: &str, arguments: &[&str]) -> String {
        let output: Output = Command::new(program)
            .args(arguments)
            .env("PATH", self.search_path())
            .output()
            .unwrap_or_else(|error| panic!("`{program} {}` could not start in the guest: {error}", arguments.join(" ")));
        return format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice()));
    }

    /// Whether the named dataset,
    /// snapshot or bookmark exists.
    pub(crate) fn exists(&self, name: &str) -> bool {
        let mut arguments: Vec<&str> = vec!["list", "-H", "-o", "name"];
        if name.contains('#') {
            arguments.extend_from_slice(&["-t", "bookmark"]);
        }
        arguments.push(name);
        let output: Output = Command::new("zfs")
            .args(arguments)
            .env("PATH", self.search_path())
            .output()
            .expect("the guest has `zfs`");
        return output.status.success();
    }

    /// One dataset's live GUID and creation transaction group.
    ///
    /// A `zfs get` of two properties prints one line per property,
    /// so the values are read as lines and never as one separated row.
    pub(crate) fn identity(&self, name: &str) -> (u64, u64) {
        let printed: String = self.run("zfs", &["get", "-H", "-p", "-o", "value", "guid,createtxg", name]);
        let mut values = printed.lines();
        let guid: u64 = values.next().expect("the GUID is printed").trim().parse().expect("the GUID is a number");
        let createtxg: u64 = values.next().expect("the group is printed").trim().parse().expect("the group is a number");
        return (guid, createtxg);
    }

    /// The full name of one dataset in this pool.
    pub(crate) fn named(&self, below: &str) -> String {
        return format!("{}/{below}", self.pool);
    }

    /// What:
    ///  Build the pool,
    ///  the checkout,
    ///  its origin snapshot and bookmark,
    ///  the clone and its own snapshots.
    /// Why:
    ///  The checkout's older history and one bookmark are what the lineage proof's origin-side
    ///  comparison rests on,
    ///  and the own snapshots are the set a removal records and submits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fixture.build(['manual-2026-11-21', 'sanoid-hourly-2026-11-22']);
    /// ```
    pub(crate) fn build(&self, snapshots: &[&str]) {
        self.run("mkdir", &["--parents", self.root.to_str().expect("the fixture root is text")]);
        let image: PathBuf = self.root.join("pool.img");
        let image_text: &str = image.to_str().expect("the image path is text");
        if Path::new(image_text).exists() {
            self.teardown();
        }
        self.run("truncate", &["--size", POOL_IMAGE_SIZE, image_text]);
        self.run("zpool", &["create", "-f", self.pool.as_str(), image_text]);
        let checkout: String = self.named("checkout");
        let clones: String = self.named("clones");
        self.run("zfs", &["create", "-o", "mountpoint=legacy", checkout.as_str()]);
        self.run("zfs", &["create", "-o", "mountpoint=legacy", "-o", "canmount=off", clones.as_str()]);
        self.run("zfs", &["snapshot", format!("{checkout}@earliest").as_str()]);
        self.run("zfs", &["bookmark", format!("{checkout}@earliest").as_str(), format!("{checkout}#from-earliest").as_str()]);
        self.run("zfs", &["snapshot", format!("{checkout}@{}", self.origin_name).as_str()]);
        let clone: String = self.clone();
        self.run("zfs", &["clone", self.origin().as_str(), clone.as_str()]);
        for snapshot in snapshots {
            self.run("zfs", &["snapshot", format!("{clone}@{snapshot}").as_str()]);
        }
    }

    /// What:
    ///  Tear the whole fixture down,
    ///  whatever the arm left.
    /// Why:
    ///  An arm that rebuilds the pool must also rebuild the add's records:
    ///  a repository left from an earlier arm holds that arm's registration and its destruction
    ///  plan,
    ///  and a control that counted the plan folder would then count another clone's plan.
    pub(crate) fn teardown(&self) {
        let _ = Command::new("zpool")
            .args(["destroy", "-f", self.pool.as_str()])
            .env("PATH", self.search_path())
            .output();
        let image: PathBuf = self.root.join("pool.img");
        let _ = std::fs::remove_file(image.as_path());
        let repository: PathBuf = self.root.join("repo");
        let _ = std::fs::remove_dir_all(repository.as_path());
    }

    /// The clone's full name.
    pub(crate) fn clone(&self) -> String {
        return format!("{}/{SLOT}", self.named("clones"));
    }

    /// The checkout's full name.
    pub(crate) fn checkout(&self) -> String {
        return self.named("checkout");
    }

    /// The origin snapshot's full name.
    pub(crate) fn origin(&self) -> String {
        return format!("{}@{}", self.checkout(), self.origin_name);
    }

    /// What:
    ///  The identities a removal recorded at the add,
    ///  read from the live objects as the add would have read them.
    /// Why:
    ///  An arm judges a proof,
    ///  so its record is the live tuple before any lineage change,
    ///  and an arm that changed the lineage changes the pool and not the record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const recorded = fixture.recorded();
    /// ```
    pub(crate) fn recorded(&self) -> RecordedBinding {
        let (clone_guid, clone_createtxg) = self.identity(self.clone().as_str());
        let (origin_dataset_guid, _) = self.identity(self.checkout().as_str());
        let (origin_snapshot_guid, origin_snapshot_createtxg) = self.identity(self.origin().as_str());
        let (pool_guid, _) = self.identity(self.pool.as_str());
        return RecordedBinding {
            slot: String::from(SLOT),
            slot_path: self.slot_path(),
            pool: self.pool.clone(),
            pool_guid: guid_of(pool_guid),
            clone_dataset: self.clone(),
            clone_guid: guid_of(clone_guid),
            clone_createtxg,
            origin_dataset: self.checkout(),
            origin_dataset_guid: guid_of(origin_dataset_guid),
            origin_snapshot: self.origin(),
            origin_snapshot_guid: guid_of(origin_snapshot_guid),
            origin_snapshot_createtxg,
            registration_id: Vec::from(SLOT.as_bytes()),
            // The records writer reads the registration's own identity once the folder stands; the proof compares
            // none of it, so a placeholder here keeps the identity read in one place.
            registration_identity: FolderIdentity::decode("1:1:1").expect("a decodable identity"),
        };
    }

    /// The fixture repository's common Git directory.
    pub(crate) fn common(&self) -> PathBuf {
        return self.root.join("repo").join(".git");
    }

    /// The fixture repository's slot path.
    pub(crate) fn slot_path(&self) -> PathBuf {
        return self.root.join("repo").join(".claude").join("worktrees").join(SLOT);
    }

    /// The registration directory the fixture repository holds.
    pub(crate) fn registration(&self) -> PathBuf {
        return self.common().join("worktrees").join(SLOT);
    }

    /// How many entries the removal's plan folder holds,
    /// which is zero before a removal writes one.
    pub(crate) fn plan_entries(&self) -> usize {
        let folder: PathBuf = crate::worktree_clone_binding_record::registration_home::plans_folder(self.common().as_path());
        return std::fs::read_dir(folder.as_path()).map_or(0, |entries| return entries.count());
    }

    /// What:
    ///  The records an add would have left for this clone:
    ///  the repository's folders,
    ///  the registration,
    ///  the binding and the origin-side inventory.
    /// Why:
    ///  The entry reads them by the slot's name,
    ///  and a fixture that skipped one would stop before the arm reached the pool.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fixture.writeTheAddsRecords(recorded);
    /// ```
    pub(crate) fn write_the_adds_records(&self, recorded: &RecordedBinding) {
        std::fs::create_dir_all(self.slot_path().as_path()).expect("the slot folder");
        std::fs::create_dir_all(self.registration().as_path()).expect("the registration");
        let registration_identity: FolderIdentity = FolderIdentity::of_folder(self.registration().as_path()).expect("the registration's identity");
        // The settle compares the underlay's identity with the one the binding records, so the record carries the
        // slot folder's own identity and not the registration's.
        let slot_identity: FolderIdentity = FolderIdentity::of_folder(self.slot_path().as_path()).expect("the slot's identity");
        let home: PathBuf = home_folder(self.common().as_path());
        std::fs::create_dir_all(home.as_path()).expect("the binding home");
        let lines: Vec<String> = vec![
            String::from(BINDING_HEADER),
            String::from("claim=1"),
            String::from("nonce=00112233445566778899aabbccddeeff"),
            format!("request=branch {}", bytes_as_hexadecimal(b"main")),
            String::from("phase=filled"),
            format!("checkoutDataset={}", recorded.origin_dataset),
            format!("checkoutDatasetGuid={}", recorded.origin_dataset_guid.file_name()),
            format!("repository={}", path_as_hexadecimal(self.common().as_path())),
            format!("repositoryIdentity={}", FolderIdentity::of_folder(self.common().as_path()).expect("the common directory's identity").encode()),
            format!("slot={}", path_as_hexadecimal(self.slot_path().as_path())),
            format!("slotIdentity={}", slot_identity.encode()),
            format!("registration={}", path_as_hexadecimal(self.registration().as_path())),
            format!("registrationIdentity={}", registration_identity.encode()),
            format!("pool={}", recorded.pool),
            format!("poolGuid={}", recorded.pool_guid.file_name()),
            format!("originSnapshot={}", recorded.origin_snapshot),
            format!("originSnapshotGuid={}", recorded.origin_snapshot_guid.file_name()),
            format!("originSnapshotCreatetxg={}", recorded.origin_snapshot_createtxg),
            format!("originDataset={}", recorded.origin_dataset),
            format!("originDatasetGuid={}", recorded.origin_dataset_guid.file_name()),
            format!("cloneDataset={}", recorded.clone_dataset),
            format!("cloneGuid={}", recorded.clone_guid.file_name()),
            format!("cloneCreatetxg={}", recorded.clone_createtxg),
            format!("pointer={}", bytes_as_hexadecimal(b"gitdir: /nowhere\n")),
        ];
        std::fs::write(home.join(binding_name(SLOT)), format!("{}\n", lines.join("\n"))).expect("the binding record");
        let inventory: InventoryRecord = self.the_origin_inventory(recorded);
        std::fs::write(home.join(origin_inventory_name(SLOT)), inventory.encode()).expect("the origin inventory");
    }

    /// What:
    ///  The origin-side inventory the add recorded:
    ///  the checkout's snapshots at or below the origin snapshot's group and its bookmarks.
    /// Why:
    ///  The third check compares the live clone against it,
    ///  and the transferred bookmark is the object a snapshot-only inventory misses.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const inventory = fixture.theOriginInventory(recorded);
    /// ```
    pub(crate) fn the_origin_inventory(&self, recorded: &RecordedBinding) -> InventoryRecord {
        let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");
        let mut entries: Vec<InventoryEntry> = Vec::new();
        for snapshot in reads.snapshots_of(self.checkout().as_str(), recorded.origin_dataset_guid).expect("the checkout's snapshots list") {
            if snapshot.createtxg <= recorded.origin_snapshot_createtxg {
                entries.push(InventoryEntry { kind: EntryKind::Snapshot, guid: snapshot.guid, createtxg: snapshot.createtxg, name: snapshot.name });
            }
        }
        for bookmark in reads.bookmarks_of(self.checkout().as_str(), recorded.origin_dataset_guid).expect("the checkout's bookmarks list") {
            entries.push(InventoryEntry { kind: EntryKind::Bookmark, guid: bookmark.guid, createtxg: bookmark.createtxg, name: bookmark.name });
        }
        return InventoryRecord {
            kind: InventoryKind::Origin,
            pool_guid: recorded.pool_guid,
            dataset: recorded.origin_dataset.clone(),
            dataset_guid: recorded.origin_dataset_guid,
            bounds: recorded.origin_snapshot.clone(),
            bounds_guid: recorded.origin_snapshot_guid,
            bounds_createtxg: recorded.origin_snapshot_createtxg,
            entries,
        };
    }

    /// What:
    ///  Every snapshot and bookmark tuple the checkout and the clone hold,
    ///  by GUID and creation transaction group,
    ///  sorted and without repeats.
    /// Why:
    ///  Promotion moves objects between the two datasets without changing their tuples,
    ///  so a set of tuples is what proves no protected object was lost,
    ///  where a set of names would read a move as a loss.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const guids = fixture.protectedGuids();
    /// ```
    pub(crate) fn protected_guids(&self) -> Vec<String> {
        let mut found: Vec<String> = Vec::new();
        for dataset in [self.checkout(), self.clone()] {
            for kind in ["snapshot", "bookmark"] {
                let printed: String = self.ask("zfs", &["list", "-H", "-p", "-o", "guid,createtxg", "-r", "-t", kind, dataset.as_str()]);
                for line in printed.lines() {
                    if line.trim().is_empty() || line.contains("no datasets available") {
                        continue;
                    }
                    // A bookmark carries its source snapshot's tuple, so the kind is what tells the two apart:
                    // without it a bookmark and the snapshot it marks are one entry and a lost bookmark reads as a
                    // shape the control expects.
                    found.push(format!("{kind}:{}", line.trim()));
                }
            }
        }
        found.sort();
        found.dedup();
        return found;
    }
}

/// One live GUID as this route holds it.
pub(crate) fn guid_of(value: u64) -> DatasetGuid {
    return DatasetGuid::parse(value.to_string().as_str()).expect("the kernel reports a canonical GUID");
}

/// What:
///  A device that sends every request to the guest's own kernel,
///  and runs one hook after the first request returns,
///  so an arm can change the pool in the window between two requests.
/// Why:
///  The owed measurement's second half needs the mark set after the batch's request and before
///  the head check,
///  which is the window the design's pause route reaches in a shipped build;
///  this round reaches it from the arm's own answer instead,
///  because the pause route is plan step 16's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const device = { calls: 0, destroySnaps: (pool, names) => { const a = live.destroySnaps(pool, names); if (++this.calls === 1) hook(); return a; } };
/// ```
pub(crate) struct HookedDevice<'a> {
    /// The guest's own device,
    /// which every request reaches.
    pub(crate) live: &'a dyn SnapshotDestroyDevice,
    /// What the arm does once the first request has returned.
    pub(crate) after_the_first: Box<dyn Fn() + 'a>,
    /// How many requests have reached the kernel.
    pub(crate) calls: std::cell::Cell<usize>,
}

impl SnapshotDestroyDevice for HookedDevice<'_> {
    fn destroy_snaps(&self, pool: &str, names: &[&str]) -> Result<Option<NvList>, DeviceError> {
        let answer: Result<Option<NvList>, DeviceError> = self.live.destroy_snaps(pool, names);
        let seen: usize = self.calls.get();
        self.calls.set(seen + 1);
        if seen == 0 {
            (self.after_the_first)();
        }
        return answer;
    }
}
