//! What:
//!  The removal a person's `git worktree remove` makes through the built wrapper,
//!  on a real file-backed pool,
//!  with the mount handler answering the unmount it asks for:
//!  the clone,
//!  its own snapshots,
//!  its binding and plan and its registration all gone,
//!  and the loss printed in the same invocation;
//!  beside the refusals that stay refusals.
//! Why:
//!  The proof reads `/dev/zfs`,
//!  and a promotion is a real pool's own lineage change:
//!  a promoted clone's transferred snapshots are history the checkout still
//!  depends on,
//!  and item 11 measured that destroying one is a loss no promotion restores
//!  (`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
//!  items 11,
//!  12 and 13).
//!  The host controls answer every seam with a scripted value,
//!  so only this arm shows the order's own writes on the answers a kernel gives,
//!  and the removal's readiness proof is the handler's own:
//!  a sweeper runs beside the wrapper and files the `unmount` receipt the removal
//!  waits for,
//!  and the fixture gives the handler what it opens before that receipt:
//!  the account's `.cache` as a dataset of the guest's pool and the root-owned
//!  `user-cache` pin that records it,
//!  because the handler opens the checkout owner's `.cache` and checks the pin
//!  before it moves the clone.
//!  It runs where `CLI_GIT_ZFS_REMOVAL_REFUSAL_GUEST` names a disposable guest,
//!  printing `SKIP` everywhere else.
//!  A refusing arm asserts what a refusal leaves:
//!  the plan folder holds no entry,
//!  the clone's intent property still holds what it held,
//!  the binding record's bytes are the ones the add wrote,
//!  and every snapshot and bookmark still stands.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const said = wrapperGit(checkout, ['worktree', 'remove', slot]);
//! expect(said).toContain('is destroyed');
//! ```

use super::zfs_slot_state_guest_fixture::{
    Place,
    guid_of,
    publish,
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
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY;
use std::os::unix::fs::PermissionsExt;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Output;

/// The variable the guest driver sets for these arms.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_REMOVAL_REFUSAL_GUEST";

/// The slot this arm registers and clones.
const SLOT: &str = "fix-flaky-test";

/// The clone's own snapshot,
/// which is the design's example name for the loss a removal prints.
const OWN_SNAPSHOT: &str = "worktree-own";

/// The account's home,
/// which is the checkout owner's in this guest and the home the handler reads.
const ACCOUNT_HOME: &str = "/root";

/// The root-owned pin the handler checks the account's `.cache` against.
const PIN: &str = "/var/lib/monochromatic-git-policy-cli/user-cache-pin";

/// The wrapper program the driver built,
/// which the arm reaches through a `git` link so it runs as the person's `git`.
const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// Whether this run is inside the guest.
fn in_the_guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!(
            "SKIP worktree_clone_removal_proof_and_plan::refusal_guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest"
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

/// What:
///  Run one Git command through the built wrapper,
///  the way a person's shell starts it.
/// Why:
///  Plan step 14's wiring is the removal's command path itself,
///  so the arm proves the refusal a person meets rather than the function
///  behind it.
fn wrapper_git(
    checkout: &Path,
    arguments: &[&str],
) -> Output {
    let bin: PathBuf = PathBuf::from("/root/removal-refusal-bin");
    std::fs::create_dir_all(bin.as_path()).expect("the wrapper's bin folder");
    let home: PathBuf = PathBuf::from("/root/removal-refusal-home");
    std::fs::create_dir_all(home.as_path()).expect("the wrapper's home folder");
    let runtime: PathBuf = PathBuf::from("/root/removal-refusal-runtime");
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

/// What:
///  The wrapper's removal of the bound slot,
///  which the removal's own checks must stop,
///  with the lines it printed.
/// Why:
///  Every refusing arm asks the same command and reads a different stop,
///  and a command that ran Git instead would have changed the slot.
fn the_stopped_removal(
    checkout: &Path,
    slot: &Path,
) -> String {
    let removed: Output = wrapper_git(checkout, &["worktree", "remove", shown(slot)]);
    let said: String = said_of(&removed);
    assert_eq!(
        removed.status.code(),
        Some(1),
        "a removal the checks stop ends in the wrapper's refusal status: {said}"
    );
    assert!(
        !said.contains("this release has no removal for a clone"),
        "no sentence of this release says a removal does not exist: {said}"
    );
    assert!(
        slot.is_dir(),
        "the slot still holds its pointer and nothing was removed: {said}"
    );
    return said;
}

/// What:
///  Every protected snapshot's and bookmark's identity in one pool subtree,
///  as a sorted set.
/// Why:
///  Promotion moves objects between datasets without changing their GUIDs,
///  so a set of GUIDs is what proves no protected object was lost,
///  where a set of names would read a move as a loss.
fn protected_guids(
    sbin: &str,
    datasets: &[&str],
) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for dataset in datasets {
        for kind in ["snapshot", "bookmark"] {
            let (ok, said, error): (bool, String, String) = zfs(
                sbin,
                "zfs",
                &["list", "-H", "-p", "-o", "guid", "-r", "-t", kind, dataset],
                None,
            );
            assert!(ok, "the {kind}s of `{dataset}` list: {said}{error}");
            for line in said.lines() {
                let trimmed: &str = line.trim();
                if trimmed.is_empty() || trimmed.contains("no datasets available") {
                    continue;
                }
                // A bookmark carries its source snapshot's GUID, so the kind is what keeps the two apart.
                found.push(format!("{kind}:{trimmed}"));
            }
        }
    }
    found.sort();
    found.dedup();
    return found;
}

/// One dataset's creation transaction group.
fn createtxg_of(
    sbin: &str,
    dataset: &str,
) -> u64 {
    let (ok, said, error): (bool, String, String) = zfs(
        sbin,
        "zfs",
        &["get", "-H", "-p", "-o", "value", "createtxg", dataset],
        None,
    );
    assert!(ok, "`createtxg` of `{dataset}` reads: {said}{error}");
    return said
        .trim()
        .parse()
        .expect("the creation transaction group is a number");
}

/// What:
///  The origin-side inventory the add records for one slot:
///  the checkout's snapshots at or below the origin snapshot's group,
///  and every bookmark it holds.
/// Why:
///  The removal's third check compares the clone's live GUIDs with these,
///  so the fixture must be the add's own record rather than a hand-written
///  list,
///  and the transferred snapshot a promotion moves carries one of these GUIDs.
fn write_the_origin_inventory(
    place: &Place,
    sbin: &str,
) -> InventoryRecord {
    let origin_snapshot: String = format!("{}@{ORIGIN_PREFIX}{NONCE}", place.checkout_dataset);
    let origin_createtxg: u64 = createtxg_of(sbin, origin_snapshot.as_str());
    let mut entries: Vec<InventoryEntry> = Vec::new();
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
            "snapshot",
            "-r",
            place.checkout_dataset.as_str(),
        ],
        None,
    );
    assert!(ok, "the checkout's snapshots list: {listed}{error}");
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
            .expect("the snapshot's group is a number");
        if group > origin_createtxg {
            continue;
        }
        entries.push(InventoryEntry {
            kind: EntryKind::Snapshot,
            guid: DatasetGuid::parse(guid.trim()).expect("the snapshot's GUID is canonical"),
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
            guid: DatasetGuid::parse(guid.trim()).expect("the bookmark's GUID is canonical"),
            createtxg: createtxg
                .trim()
                .parse()
                .expect("the bookmark's group is a number"),
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
        bounds_createtxg: origin_createtxg,
        entries,
    };
    publish_inventory(&home, SLOT, InventoryKind::Origin, &record)
        .expect("the origin inventory publishes once");
    return record;
}

/// One dataset's,
/// snapshot's or bookmark's GUID and creation transaction group,
/// which is the tuple a promotion moves and a bookmark carries.
fn identity_of(
    sbin: &str,
    name: &str,
) -> (u64, u64) {
    let (ok, said, error): (bool, String, String) = zfs(
        sbin,
        "zfs",
        &["get", "-H", "-p", "-o", "value", "guid,createtxg", name],
        None,
    );
    assert!(ok, "`guid,createtxg` of `{name}` reads: {said}{error}");
    let mut values = said.lines();
    let guid: u64 = values
        .next()
        .expect("the GUID is printed")
        .trim()
        .parse()
        .expect("the GUID is a number");
    let group: u64 = values
        .next()
        .expect("the group is printed")
        .trim()
        .parse()
        .expect("the group is a number");
    return (guid, group);
}

/// The clone's current intent property,
/// which no refusal of this release may change.
fn the_intent_of(
    sbin: &str,
    clone: &str,
) -> String {
    let (ok, said, error): (bool, String, String) = zfs(
        sbin,
        "zfs",
        &[
            "get",
            "-H",
            "-p",
            "-o",
            "value",
            DESIRED_STATE_PROPERTY,
            clone,
        ],
        None,
    );
    assert!(ok, "the clone's intent property reads: {said}{error}");
    return said.trim().to_string();
}

/// The binding record's bytes,
/// which no refusal of this release may change.
fn the_binding_bytes(common: &Path) -> Vec<u8> {
    let home: PathBuf = home_folder(common);
    let name: String =
        git_policy_cli::worktree_clone_binding_record::registration_home::binding_name(SLOT);
    return std::fs::read(home.join(name)).expect("the binding record the add wrote reads");
}

/// What:
///  The plan folder's entries,
///  which a refusal of this release never creates.
fn the_plan_folder_holds(common: &Path) -> usize {
    return std::fs::read_dir(plans_folder(common)).map_or(0, |entries| return entries.count());
}

/// What:
///  The account's `.cache` as a dataset of the guest's pool,
///  and the root-owned pin that records it.
/// Why:
///  The handler opens the checkout owner's `.cache` and refuses with the
///  `user-cache` step when no dataset stands there,
///  and it reads the pin to prove the mount at `.cache` is that dataset's root
///  (`worktree_clone_unmount/user_cache_pin.rs`,
///  `open_cache` and `read_pin`).
///  The wrapper makes the per-clone staging folder under this dataset before it
///  asks for the unmount,
///  so the folder the wrapper writes and the mount the handler reads are the
///  same one.
fn make_the_cache_and_pin(
    sbin: &str,
    pool: &str,
) -> String {
    let cache_dataset: String = format!("{pool}/cache");
    let _destroyed: (bool, String, String) = zfs(
        sbin,
        "zfs",
        &["destroy", "-r", "-f", cache_dataset.as_str()],
        None,
    );
    std::fs::create_dir_all(ACCOUNT_HOME).expect("the account's home is made");
    zfs_ok(
        sbin,
        &[
            "create",
            "-p",
            "-o",
            format!("mountpoint={ACCOUNT_HOME}/.cache").as_str(),
            cache_dataset.as_str(),
        ],
    );
    let facts: std::fs::Metadata =
        std::fs::metadata(format!("{ACCOUNT_HOME}/.cache")).expect("the cache's metadata reads");
    let pin: String = format!(
        "{{\"version\":\"v1\",\"kind\":\"user-cache-pin\",\"source\":\"{cache_dataset}\",\"root-inode\":\"{}\",\"device\":\"{}\"}}\n",
        std::os::unix::fs::MetadataExt::ino(&facts),
        std::os::unix::fs::MetadataExt::dev(&facts)
    );
    let directory: &Path = Path::new(PIN).parent().expect("the pin has a folder");
    std::fs::create_dir_all(directory).expect("the pin's folder is made");
    std::fs::write(PIN, pin).expect("the pin is written");
    std::fs::set_permissions(PIN, std::fs::Permissions::from_mode(0o644))
        .expect("the pin holds its mode");
    return cache_dataset;
}

/// What:
///  Plan step 14's refusal arms on a real pool,
///  through the wrapper's own `git worktree remove`:
///  the healthy proof and its loss,
///  a promotion's refusal with every protected object surviving,
///  the reverse promotion's proof,
///  the transferred bookmark and snapshot a promotion leaves,
///  a stale bookmark refused by name,
///  and a same-name replacement clone refused on its GUID.
/// Why:
///  Each arm is a pool's own fact the host controls cannot make,
///  and each changes the pool and not the records,
///  so a refusal that wrote nothing is visible in the folder and the property
///  beside it.
#[test]
fn zfs_removal_refusal_guest() {
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
    command_ok(
        "git",
        &[
            "-C",
            shown(checkout.as_path()),
            "worktree",
            "add",
            "--quiet",
            "--no-checkout",
            shown(slot.as_path()),
            "HEAD",
        ],
    );
    let slot_identity: FolderIdentity =
        FolderIdentity::of_folder(slot.as_path()).expect("the underlay's identity reads");
    let pointer: Vec<u8> = std::fs::read(slot.join(".git")).expect("the underlay pointer reads");
    write_passing_probe_record();
    prepare_the_store(checkout.as_path());
    write_configuration(checkout.as_path(), None, None);
    // The handler reads the account's home from the checkout's owner, so the fixture's cache and its pin are made for
    // that account: the wrapper runs as root here, and root owns the checkout this arm just made.
    let cache: String = make_the_cache_and_pin(sbin.as_str(), pool.as_str());
    let clone: String = clone_the_checkouts_snapshot(
        sbin.as_str(),
        checkout_dataset.as_str(),
        container.as_str(),
        SLOT,
        NONCE,
        Some(mount_request(NONCE).as_str()),
    );
    // The clone's own snapshot, which is the name a removal prints as lost.
    zfs_ok(
        sbin.as_str(),
        &["snapshot", format!("{clone}@{OWN_SNAPSHOT}").as_str()],
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
    publish(&place, Phase::Filled, sbin.as_str(), SLOT);
    let origin_snapshot: String = format!("{checkout_dataset}@{ORIGIN_PREFIX}{NONCE}");
    // The checkout's own bookmark is part of the history a removal refuses to inherit, and it is what the origin-side
    // inventory records beside the snapshots.
    zfs_ok(
        sbin.as_str(),
        &[
            "bookmark",
            origin_snapshot.as_str(),
            format!("{checkout_dataset}#from-checkout").as_str(),
        ],
    );
    write_the_origin_inventory(&place, sbin.as_str());
    let intent: String = the_intent_of(sbin.as_str(), clone.as_str());
    let binding: Vec<u8> = the_binding_bytes(common.as_path());

    // Arm 2: an administrator promotes the clone, and the next removal refuses before any delete with every protected
    // snapshot and bookmark still there.
    // The recorded tuple is read before the promotion, because a promotion moves the origin snapshot under the clone.
    let recorded_tuple: (u64, u64) = identity_of(sbin.as_str(), origin_snapshot.as_str());
    let protected: Vec<String> =
        protected_guids(sbin.as_str(), &[checkout_dataset.as_str(), clone.as_str()]);
    assert!(
        protected.len() >= 3,
        "the arm read the checkout's snapshot and the clone's own: {protected:?}"
    );
    zfs_ok(sbin.as_str(), &["promote", clone.as_str()]);
    let promoted: String = the_stopped_removal(checkout.as_path(), slot.as_path());
    assert!(
        promoted.contains("Nothing was destroyed, and no plan was written."),
        "{promoted}"
    );
    assert!(
        promoted.contains(&format!("the clone `{clone}` has origin")),
        "{promoted}"
    );
    assert!(
        promoted.contains(&format!("the add recorded `{origin_snapshot}`")),
        "{promoted}"
    );
    assert!(
        !promoted.contains("the lineage proof a removal makes passed"),
        "a promoted clone's proof does not pass: {promoted}"
    );
    assert_eq!(
        protected_guids(sbin.as_str(), &[checkout_dataset.as_str(), clone.as_str()]),
        protected,
        "promotion moved tuples without losing one, and the refusal destroyed nothing"
    );
    assert_eq!(the_plan_folder_holds(common.as_path()), 0);
    println!(
        "REMOVAL-ARMS promoted=refused cause=origin-is-not-the-recorded-one protected=survived"
    );

    // Arm 3: the promotion left the origin snapshot's own tuple under the clone, and a bookmark made from it carries that
    // tuple unchanged, which is item 13's measurement and what the host controls refuse as inherited history.
    let transferred: String = format!("{clone}@{ORIGIN_PREFIX}{NONCE}");
    let (transferred_guid, transferred_group): (u64, u64) =
        identity_of(sbin.as_str(), transferred.as_str());
    assert_eq!(
        (transferred_guid, transferred_group),
        recorded_tuple,
        "a promotion moves the origin snapshot's tuple under the clone without changing it"
    );
    assert!(
        transferred_group <= createtxg_of(sbin.as_str(), clone.as_str()),
        "the transferred snapshot is older than the clone, which is how the proof tells it from the clone's own"
    );
    zfs_ok(
        sbin.as_str(),
        &[
            "bookmark",
            transferred.as_str(),
            format!("{clone}#from-origin").as_str(),
        ],
    );
    assert_eq!(
        identity_of(sbin.as_str(), format!("{clone}#from-origin").as_str()),
        recorded_tuple,
        "a bookmark made from a transferred snapshot carries that snapshot's tuple"
    );
    let inherited: String = the_stopped_removal(checkout.as_path(), slot.as_path());
    assert!(
        inherited.contains("Nothing was destroyed, and no plan was written."),
        "{inherited}"
    );
    // The bookmark carries the transferred snapshot's GUID, so a set of identities cannot grow by it: what the arm can
    // show is that every identity it read before the promotion is still there, which is what "every protected object
    // survives" means.
    let after: Vec<String> =
        protected_guids(sbin.as_str(), &[checkout_dataset.as_str(), clone.as_str()]);
    for identity in &protected {
        assert!(
            after.contains(identity),
            "`{identity}` survived the refusal, and the pool now lists {after:?}"
        );
    }
    assert_eq!(the_plan_folder_holds(common.as_path()), 0);
    assert_eq!(
        the_intent_of(sbin.as_str(), clone.as_str()),
        intent,
        "a stopped removal filed no intent of its own"
    );
    assert_eq!(
        the_binding_bytes(common.as_path()),
        binding,
        "a stopped removal rewrote no binding record"
    );
    println!(
        "REMOVAL-ARMS inherited-tuple=measured guid={transferred_guid} group={transferred_group} refused=before-any-write"
    );

    // Arm 4: the reverse promotion restores the recorded lineage.
    // The bookmarks follow the snapshot they were made from, so the reverse promotion puts the transferred one back
    // under the checkout.
    zfs_ok(sbin.as_str(), &["promote", checkout_dataset.as_str()]);
    zfs_ok(
        sbin.as_str(),
        &[
            "destroy",
            format!("{checkout_dataset}#from-origin").as_str(),
        ],
    );
    assert_eq!(
        identity_of(sbin.as_str(), origin_snapshot.as_str()),
        recorded_tuple,
        "the reverse promotion puts the origin snapshot's tuple back under the checkout"
    );
    println!("REMOVAL-ARMS reverse-promotion=restored-lineage");

    // Arm 5: a bookmark one of the clone's own live snapshots explains is a loss the healthy removal prints; one whose
    // snapshot the kernel destroyed is refused by name.
    zfs_ok(
        sbin.as_str(),
        &[
            "bookmark",
            format!("{clone}@{OWN_SNAPSHOT}").as_str(),
            format!("{clone}#from-own").as_str(),
        ],
    );
    zfs_ok(
        sbin.as_str(),
        &["snapshot", format!("{clone}@gone").as_str()],
    );
    zfs_ok(
        sbin.as_str(),
        &[
            "bookmark",
            format!("{clone}@gone").as_str(),
            format!("{clone}#from-gone").as_str(),
        ],
    );
    zfs_ok(
        sbin.as_str(),
        &["destroy", format!("{clone}@gone").as_str()],
    );
    let refused: String = the_stopped_removal(checkout.as_path(), slot.as_path());
    assert!(
        refused.contains(&format!("`{clone}#from-gone`")),
        "the refused bookmark is named: {refused}"
    );
    assert!(
        refused.contains(
            "the removal stops before its first delete with every snapshot and bookmark still there"
        ),
        "{refused}"
    );
    println!("REMOVAL-ARMS bookmark-stale=refused name={clone}#from-gone");
    // The refused bookmark is this arm's own debris: it goes, so the arm below starts from a clone the rule passes.
    zfs_ok(
        sbin.as_str(),
        &["destroy", format!("{clone}#from-gone").as_str()],
    );

    // Arm 6: the whole removal through the built wrapper, with the mount handler answering the unmount it asks for.
    // The clone is attached first, so the unmount is a real unmount at a real slot rather than a state the arm assumed.
    let attached: super::zfs_sweep_guest::Run = super::zfs_sweep_guest::sweep();
    assert_eq!(
        attached.status,
        Some(0),
        "the handler's sweep attaches the clone: {}",
        attached.said
    );
    let mounted: Output = std::process::Command::new("findmnt")
        .args([
            "--noheadings",
            "--output",
            "SOURCE",
            "--target",
            shown(slot.as_path()),
        ])
        .output()
        .expect("findmnt reads this guest's mount table");
    assert!(
        String::from_utf8_lossy(mounted.stdout.as_slice()).contains(clone.as_str()),
        "the clone stands at the slot before the removal asks for its unmount"
    );
    // A sweeper runs beside the wrapper, because the removal files one `unmount` request and waits for the handler's
    // receipt: no second pass of this arm can answer a request the wrapper has not filed yet.
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
    let removal: Output = wrapper_git(
        checkout.as_path(),
        &["worktree", "remove", shown(slot.as_path())],
    );
    let said: String = said_of(&removal);
    // The sweeper is this arm's own child, and it ends here whether the assertions pass or not.
    let _ended: std::io::Result<std::process::ExitStatus> =
        sweeper.kill().and_then(|()| return sweeper.wait());
    assert_eq!(
        removal.status.code(),
        Some(0),
        "the removal of a slot a whole managed clone is bound to finishes: {said}"
    );
    assert!(
        said.contains(&format!(
            "the ZFS worktree clone at `{SLOT}` (dataset `{clone}`) is destroyed"
        )),
        "the loss names the slot, its dataset and the destroy, in the same invocation: {said}"
    );
    assert!(
        said.contains(&format!("destroyed snapshot {clone}@{OWN_SNAPSHOT}")),
        "every snapshot the removal destroyed is named: {said}"
    );
    assert!(
        said.contains("bookmark(s) went with the clone's head and cannot be restored"),
        "the bookmark the head took is named as a loss: {said}"
    );
    assert!(
        said.contains("the removal's plan, its binding and both inventories are removed"),
        "the records are removed and the slot is free for the next add: {said}"
    );
    let clone_gone: (bool, String, String) = zfs(
        sbin.as_str(),
        "zfs",
        &["list", "-H", "-o", "name", clone.as_str()],
        None,
    );
    assert!(!clone_gone.0, "the clone dataset is gone: {}", clone_gone.1);
    let snapshot_gone: (bool, String, String) = zfs(
        sbin.as_str(),
        "zfs",
        &[
            "list",
            "-H",
            "-o",
            "name",
            "-t",
            "snapshot",
            format!("{clone}@{OWN_SNAPSHOT}").as_str(),
        ],
        None,
    );
    assert!(
        !snapshot_gone.0,
        "the clone's own snapshot is gone: {}",
        snapshot_gone.1
    );
    assert_eq!(
        the_plan_folder_holds(common.as_path()),
        0,
        "no destruction plan stands after a finished removal"
    );
    assert!(
        !home_folder(common.as_path())
            .join(
                git_policy_cli::worktree_clone_binding_record::registration_home::binding_name(
                    SLOT
                )
            )
            .exists(),
        "the binding record is removed with the other records"
    );
    assert!(
        !common.join("worktrees").join(SLOT).exists(),
        "the Git registration is removed"
    );
    assert!(
        !slot.exists(),
        "the slot folder is gone with the underlay pointer it held"
    );
    println!(
        "REMOVAL-ARMS whole-removal=done clone=gone snapshots=gone records=gone unmount=the-handler"
    );

    // The fixture's own cleanup, which is this arm's and no record's.
    let _unmounted: (bool, String, String) = zfs(
        sbin.as_str(),
        "zfs",
        &["unmount", checkout_dataset.as_str()],
        None,
    );
    zfs_ok(
        sbin.as_str(),
        &["destroy", "-r", "-f", format!("{pool}/step8").as_str()],
    );
    zfs_ok(sbin.as_str(), &["destroy", "-r", "-f", cache.as_str()]);
    let _cleared: std::io::Result<()> = std::fs::remove_dir_all(checkout.as_path());
    println!("REMOVAL-ARMS done");
}
