//! What:
//!  The guest arm of plan step 14's piece:
//!  the healthy removal entering against a real pool,
//!  the promote and reverse-promote controls of VM item 11,
//!  the live and stale bookmark controls of VM item 13,
//!  and the same-name replacement and grant controls of VM item 12.
//! Why:
//!  The proof reads `/dev/zfs`,
//!  so no control on a person's machine may run it against a pool it did not build,
//!  and item 11's loss is only visible on a real pool:
//!  a promoted clone's transferred snapshot is history the checkout still depends on
//!  (`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
//! items 11,
//! 12 and 13).
//!  The arm runs inside the throwaway `mvm` guest the package's e2e recipe drives,
//!  and a run on this machine prints one `SKIPPED` line,
//!  which the recipe reads as a failure when it meant the arm to run.
//!  The unmount is a scripted answer here:
//!  step 9's live path needs the root handler the guest fixture does not start,
//!  and this arm judges the proof and the plan,
//! never the mount work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('the removal lineage guest arm', () => { /* zpool, checkout, clone, promote, bookmarks */ });
//! ```

use crate::worktree_clone_binding_record::binding_encoding::{FolderIdentity, bytes_as_hexadecimal, path_as_hexadecimal};
use crate::worktree_clone_binding_record::binding_inventory::{EntryKind, InventoryEntry, InventoryKind, InventoryRecord};
use crate::worktree_clone_binding_record::binding_words::BINDING_HEADER;
use crate::worktree_clone_binding_record::registration_home::{binding_name, home_folder, origin_inventory_name};
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::bookmark_rule::{explanations_of, refuse_the_unexplained_bookmarks};
use crate::worktree_clone_removal_proof_and_plan::lineage_proof::{ProvenRemoval, prove_the_lineage};
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_the_plan;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{EnteredRemoval, RemovalEntry, TheLiveClock, enter_the_removal};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::{LiveRemovalReads, RemovalPoolReads};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{RemovalUnmount, UnmountAnswer};
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The variable naming the guest a live arm may run in.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_LINEAGE_GUEST";
/// The variable naming the file-backed pool the arm builds.
const POOL_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_LINEAGE_POOL";
/// The variable naming the OpenZFS 2.4.4 prefix the guest carries.
const PREFIX_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_LINEAGE_PREFIX";
/// The variable naming the folder the pool's image and the fixture repository live under.
const ROOT_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_LINEAGE_ROOT";

/// The guest directory the fixture lives in when the recipe names none.
const DEFAULT_ROOT: &str = "/home/zfs-step-14-fixture";
/// The guest's OpenZFS 2.4.4 prefix when the recipe names none.
const DEFAULT_PREFIX: &str = "/opt/openzfs-2.4.4";
/// The pool the arm builds when the recipe names none.
const DEFAULT_POOL: &str = "zfsstep14";
/// The image the pool stands on,
/// in gibibytes.
const POOL_IMAGE_SIZE: &str = "1G";
/// The slot the arm's clone takes.
const SLOT: &str = "fix-flaky-test";
/// The clone's own snapshot.
const OWN_SNAPSHOT: &str = "worktree-own";
/// The account the item-12 grant set is written for.
const ACCOUNT: &str = "ubuntu";

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
/// const fixture = Fixture.fromEnvironment('the-removal-lineage-guest-arms');
/// ```
struct Fixture {
    /// The guest's host name.
    guest: String,
    /// The pool the arm builds and destroys.
    pool: String,
    /// The OpenZFS prefix whose `sbin` and `bin` the commands come from.
    prefix: PathBuf,
    /// The folder the pool's image and the repository live under.
    root: PathBuf,
}

impl Fixture {
    /// The fixture the environment names,
    /// or nothing on a host run,
    /// where no pool command may be made.
    fn from_environment(test: &str) -> Option<Fixture> {
        let Some(named) = std::env::var_os(GUEST_VARIABLE) else {
            println!("SKIPPED {test}: {GUEST_VARIABLE} is not set, so this machine has no guest fixture and no live arm ran");
            return None;
        };
        let guest: String = named.to_string_lossy().into_owned();
        let host: String = Self::hostname();
        if host != guest {
            panic!("{test}: {GUEST_VARIABLE} names the guest `{guest}` and this machine calls itself `{host}`, so a live arm would reach the wrong pool");
        }
        return Some(Fixture {
            guest,
            pool: std::env::var(POOL_VARIABLE).unwrap_or_else(|_| return String::from(DEFAULT_POOL)),
            prefix: PathBuf::from(std::env::var(PREFIX_VARIABLE).unwrap_or_else(|_| return String::from(DEFAULT_PREFIX))),
            root: PathBuf::from(std::env::var(ROOT_VARIABLE).unwrap_or_else(|_| return String::from(DEFAULT_ROOT))),
        });
    }

    /// This machine's own host name.
    fn hostname() -> String {
        let output: Output = Command::new("hostname").output().expect("the guest has `hostname`");
        return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    }

    /// The `PATH` the guest's OpenZFS userspace needs.
    fn search_path(&self) -> String {
        return format!("{}:{}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin", self.prefix.join("sbin").display(), self.prefix.join("bin").display());
    }

    /// One pool command,
    /// whose failure names the command and what it wrote.
    fn run(&self, program: &str, arguments: &[&str]) -> String {
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
    fn ask(&self, program: &str, arguments: &[&str]) -> String {
        let output: Output = Command::new(program)
            .args(arguments)
            .env("PATH", self.search_path())
            .output()
            .unwrap_or_else(|error| panic!("`{program} {}` could not start in the guest: {error}", arguments.join(" ")));
        return format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice()));
    }

    /// Whether the named dataset,
    /// snapshot or bookmark exists.
    fn exists(&self, name: &str) -> bool {
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
    fn identity(&self, name: &str) -> (u64, u64) {
        let printed: String = self.run("zfs", &["get", "-H", "-p", "-o", "value", "guid,createtxg", name]);
        let mut values = printed.lines();
        let guid: u64 = values.next().expect("the GUID is printed").trim().parse().expect("the GUID is a number");
        let createtxg: u64 = values.next().expect("the group is printed").trim().parse().expect("the group is a number");
        return (guid, createtxg);
    }

    /// The full name of one dataset in this pool.
    fn named(&self, below: &str) -> String {
        return format!("{}/{below}", self.pool);
    }

    /// Whether the named object exists,
    /// for the arm that asks about a bookmark.
    fn keeps(&self, name: &str) -> bool {
        return self.exists(name);
    }

    /// Build the pool,
    /// the checkout,
    /// its snapshots and bookmark,
    /// the clone and its own snapshot.
    fn build(&self) {
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
        self.run("zfs", &["snapshot", format!("{checkout}@origin").as_str()]);
        let clone: String = format!("{clones}/{SLOT}");
        self.run("zfs", &["clone", format!("{checkout}@origin").as_str(), clone.as_str()]);
        self.run("zfs", &["snapshot", format!("{clone}@{OWN_SNAPSHOT}").as_str()]);
    }

    /// Tear the fixture down,
    /// whatever the arms left.
    fn teardown(&self) {
        let _ = Command::new("zpool")
            .args(["destroy", "-f", self.pool.as_str()])
            .env("PATH", self.search_path())
            .output();
        let image: PathBuf = self.root.join("pool.img");
        let _ = std::fs::remove_file(image.as_path());
    }

    /// The clone's full name.
    fn clone(&self) -> String {
        return format!("{}/{SLOT}", self.named("clones"));
    }

    /// The checkout's full name.
    fn checkout(&self) -> String {
        return self.named("checkout");
    }

    /// The origin snapshot's full name,
    /// which is the checkout's `@origin`.
    fn origin(&self) -> String {
        return format!("{}@origin", self.checkout());
    }

    /// What:
    ///  The identities a removal recorded at the add,
    ///  read from the live objects as the add would have read them.
    /// Why:
    ///  The arm judges the proof,
    ///  so its record is the live tuple before any lineage change,
    ///  and a control that changed the lineage changes the pool and not the record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const recorded = fixture.recorded(cloneName);
    /// ```
    fn recorded(&self) -> RecordedBinding {
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
    fn common(&self) -> PathBuf {
        return self.root.join("repo").join(".git");
    }

    /// The fixture repository's slot path.
    fn slot_path(&self) -> PathBuf {
        return self.root.join("repo").join(".claude").join("worktrees").join(SLOT);
    }

    /// The registration directory the fixture repository holds.
    fn registration(&self) -> PathBuf {
        return self.common().join("worktrees").join(SLOT);
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
    fn write_the_adds_records(&self, recorded: &RecordedBinding) {
        std::fs::create_dir_all(self.slot_path().as_path()).expect("the slot folder");
        std::fs::create_dir_all(self.registration().as_path()).expect("the registration");
        let registration_identity: FolderIdentity = FolderIdentity::of_folder(self.registration().as_path()).expect("the registration's identity");
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
            format!("slotIdentity={}", registration_identity.encode()),
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
    fn the_origin_inventory(&self, recorded: &RecordedBinding) -> InventoryRecord {
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
    fn protected_guids(&self) -> Vec<String> {
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
fn guid_of(value: u64) -> DatasetGuid {
    return DatasetGuid::parse(value.to_string().as_str()).expect("the kernel reports a canonical GUID");
}

/// What:
///  The unmount answer the guest gives:
///  the intent's own nonce and the live clone's GUID,
///  so the arm judges the proof and the plan and never the mount work.
/// Why:
///  Step 9's live unmount needs the root handler the guest fixture does not start,
///  and a control that skipped the unmount would not exercise the entry's order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unmount = { requestTheUnmount: (intent) => ({ ...done, nonce: intent.nonce }) };
/// ```
struct ADoneUnmount {
    clone_guid: DatasetGuid,
}

impl RemovalUnmount for ADoneUnmount {
    fn request_the_unmount(&self, intent: &MountIntent) -> Result<UnmountAnswer, RemovalStop> {
        return Ok(UnmountAnswer { dataset: self.clone_guid, operation: intent.operation, nonce: intent.nonce.clone(), outcome: ReceiptOutcome::Done, mount_id: None, failed_step: None, errno: None });
    }
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_removal_lineage_guest_arms() {
    let Some(fixture) = Fixture::from_environment("the_removal_lineage_guest_arms") else {
        return;
    };
    println!("REMOVAL guest={} pool={}", fixture.guest, fixture.pool);
    fixture.build();
    let reads: LiveRemovalReads = LiveRemovalReads::open().expect("the guest has /dev/zfs");

    // Arm 1: the healthy removal enters, writes its plan and prints its one own snapshot.
    let recorded: RecordedBinding = fixture.recorded();
    fixture.write_the_adds_records(&recorded);
    let unmount: ADoneUnmount = ADoneUnmount { clone_guid: recorded.clone_guid };
    let common: PathBuf = fixture.common();
    let entry: RemovalEntry<'_> = RemovalEntry { common_dir: common.as_path(), slot: SLOT };
    let entered: EnteredRemoval = enter_the_removal(&entry, &reads, &unmount, &TheLiveClock).expect("the healthy removal enters");
    assert_eq!(entered.plan.attempts.len(), 1, "the clone's one own snapshot is the plan's one tuple");
    assert_eq!(entered.plan.facts.clone_guid, recorded.clone_guid);
    let read_back = read_the_plan(common.as_path(), recorded.clone_guid).expect("the plan reads back");
    assert_eq!(read_back, entered.plan);
    assert!(entered.lost.text().contains(&format!("{}@{OWN_SNAPSHOT}", fixture.clone())));
    println!("REMOVAL healthy=entered snapshots={} bookmarks={}", entered.plan.attempts.len(), entered.lost.bookmarks.len());

    // Arm 2: promotion moves the checkout's older history under the clone, and the proof refuses.
    let before: Vec<String> = fixture.protected_guids();
    // The fixture makes three snapshots and one bookmark, and a bookmark shares its source snapshot's tuple:
    // the kind is what keeps the four objects four entries here.
    assert_eq!(before.len(), 4, "the arm read the fixture's three snapshots and one bookmark: {before:?}");
    fixture.run("zfs", &["promote", fixture.clone().as_str()]);
    let origin_inventory: InventoryRecord = fixture.the_origin_inventory(&recorded);
    let refused: RemovalStop = prove_the_lineage(&reads, &recorded, &origin_inventory, &RemovalHistory::NothingDestroyed).expect_err("a promoted clone's lineage is not the recorded one");
    let after: Vec<String> = fixture.protected_guids();
    assert_eq!(before, after, "promotion moved tuples between the datasets without losing one, and the refusal destroyed nothing");
    println!("REMOVAL promoted=refused cause={:?} protected=survived", refusal_name(&refused));

    // Reverse promotion restores the map, and the recorded tuple proves again.
    fixture.run("zfs", &["promote", fixture.checkout().as_str()]);
    let proven: ProvenRemoval = prove_the_lineage(&reads, &recorded, &fixture.the_origin_inventory(&recorded), &RemovalHistory::NothingDestroyed).expect("the reverse promotion restores the recorded lineage");
    assert_eq!(proven.own_snapshots.len(), 1, "the clone owns exactly its own snapshot again");
    println!("REMOVAL reverse-promotion=proved origin={}", fixture.origin());

    // Arm 3: a bookmark a live snapshot explains passes, and one whose snapshot went refuses.
    fixture.run("zfs", &["bookmark", format!("{}@{OWN_SNAPSHOT}", fixture.clone()).as_str(), format!("{}#from-own", fixture.clone()).as_str()]);
    let live: ProvenRemoval = prove_the_lineage(&reads, &recorded, &fixture.the_origin_inventory(&recorded), &RemovalHistory::NothingDestroyed).expect("a bookmark of an own snapshot keeps the lineage proved");
    let explanations = explanations_of(&live.inventory, None);
    let refused_live = refuse_the_unexplained_bookmarks(&explanations, live.inventory.bookmarks().as_slice(), &fixture.the_origin_inventory(&recorded));
    assert!(refused_live.is_empty(), "a bookmark its own live snapshot explains passes");
    println!("REMOVAL bookmark-live=passed");
    fixture.run("zfs", &["snapshot", format!("{}@gone", fixture.clone()).as_str()]);
    fixture.run("zfs", &["bookmark", format!("{}@gone", fixture.clone()).as_str(), format!("{}#from-gone", fixture.clone()).as_str()]);
    fixture.run("zfs", &["destroy", format!("{}@gone", fixture.clone()).as_str()]);
    assert!(fixture.keeps(format!("{}#from-gone", fixture.clone()).as_str()), "the kernel keeps a bookmark whose snapshot it destroyed");
    let after_the_destroy: ProvenRemoval = prove_the_lineage(&reads, &recorded, &fixture.the_origin_inventory(&recorded), &RemovalHistory::NothingDestroyed).expect("the clone's own history still proves");
    let stale = explanations_of(&after_the_destroy.inventory, None);
    let refused_stale = refuse_the_unexplained_bookmarks(&stale, after_the_destroy.inventory.bookmarks().as_slice(), &fixture.the_origin_inventory(&recorded));
    assert_eq!(refused_stale.len(), 1, "one bookmark's snapshot is gone and nothing explains it");
    assert_eq!(refused_stale[0].name, format!("{}#from-gone", fixture.clone()));
    println!("REMOVAL bookmark-stale=refused name={}", refused_stale[0].name);

    // Arm 4: the item-12 grant set carries no lineage-changing grant, and a same-name replacement refuses.
    fixture.run("zfs", &["allow", "-d", "-u", ACCOUNT, "mount,destroy,userprop", fixture.named("clones").as_str()]);
    fixture.run("zfs", &["allow", "-u", ACCOUNT, "create,mount", fixture.named("clones").as_str()]);
    fixture.run("zfs", &["allow", "-d", "-u", ACCOUNT, "clone", fixture.checkout().as_str()]);
    let grants: String = fixture.ask("zfs", &["allow", fixture.named("clones").as_str()]);
    for forbidden in ["promote", "rename", "receive"] {
        assert!(!grants.contains(forbidden), "the decided grant set carries no `{forbidden}`, and it reads: {grants}");
    }
    println!("REMOVAL grants=lineage-changing-absent");
    fixture.run("zfs", &["destroy", format!("{}@{OWN_SNAPSHOT}", fixture.clone()).as_str()]);
    fixture.run("zfs", &["destroy", fixture.clone().as_str()]);
    fixture.run("zfs", &["clone", fixture.origin().as_str(), fixture.clone().as_str()]);
    match prove_the_lineage(&reads, &recorded, &fixture.the_origin_inventory(&recorded), &RemovalHistory::NothingDestroyed) {
        Err(RemovalStop::IdentityDiffers { field, .. }) => {
            assert_eq!(field, "cloneGuid", "a replacement clone answers another GUID");
            println!("REMOVAL replacement=refused field={field}");
        }
        Err(other) => panic!("a replacement clone refuses on its GUID, not {other:?}"),
        Ok(_) => panic!("a replacement clone must not prove the recorded lineage"),
    }

    fixture.teardown();
    println!("REMOVAL done");
}

/// The short name of one refusal,
/// for the log line the recipe greps.
fn refusal_name(stop: &RemovalStop) -> &'static str {
    return match stop {
        RemovalStop::TransferredHistory { .. } => "transferred-history",
        RemovalStop::OriginIsNotTheRecordedOne { .. } => "origin-is-not-the-recorded-one",
        RemovalStop::IdentityDiffers { .. } => "identity-differs",
        RemovalStop::OriginSideHit { .. } => "origin-side-hit",
        RemovalStop::OriginMarked { .. } => "origin-marked",
        _ => "another-refusal",
    };
}
