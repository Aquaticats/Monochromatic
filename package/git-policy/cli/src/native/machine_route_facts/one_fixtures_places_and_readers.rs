//! What:
//!  One control's fixture:
//!  a folder of its own that holds every place this machine's route facts are
//!  read from,
//!  the layout's configuration,
//!  the caller's mount table,
//!  this boot's identifier,
//!  the identity store with the parent's and one slot's record,
//!  the readiness and results folders,
//!  and the account's cache.
//! Why:
//!  Every read of a machine fact is a read of a place,
//!  so one fixture that writes each place at the design's own state lets a
//!  control gather the whole route and read what each check answered
//!  without this machine's `/proc`,
//!  store,
//!  cache or `/dev/zfs`
//!  (`the_route_places::TheRoutePlaces::under`).
//!  The store's and the receipts' readers carry the owner term their folder
//!  must answer,
//!  and a fixture's folder belongs to the account running the control,
//!  so the readers are made at that account rather than at root.
//!  The two records are published through the release's own writer,
//!  so a record a check reads is the record a writer makes.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = TheFixturesReaders.made('the-gather');
//! const gathered = the_machines_route_facts({ places: fixture.places, pool: AScriptedPool.a_healthy_machine(), store: fixture.store(), readiness: fixture.readiness, receipts: fixture.receipts });
//! ```

/// The four places and the two readers a control gathers over.
use super::the_route_places::TheRoutePlaces;
/// One mount row in the kernel's grammar,
/// which the route's own test facts already build.
use crate::doctor_zfs_route::route_test_facts::mount_row;
/// The names the design's layout fixes.
use crate::doctor_zfs_route::route_test_facts::{
    CHECKOUT_DATASET,
    CONTAINER_DATASET,
    LIVE_BOOT,
};
/// One fixture folder of the run's own,
/// removed when the test thread ends.
use crate::testing::test_support::fixture;
/// The account this process runs as,
/// which is the one the store's and the receipts' owners are compared with.
use crate::worktree_clone_add_route::add_grants::this_account;
/// The release's own writers of the parent's and one slot's record.
use crate::worktree_clone_mount::slot_identity_store_publication::{
    publish_parent_identity,
    publish_slot_identity,
};
/// The store folder a control reads through the grammar's own open,
/// and the refusal of one that could not be opened.
use crate::worktree_clone_mount::{
    CloneFolderIdentity,
    SlotIdentityStore,
    StoreRefusal,
};
/// The handler's reader of the readiness record.
use crate::worktree_clone_mount_protocol::ReadinessFile;
/// The folder the handler files its receipts in.
use crate::worktree_clone_mount_protocol::ReceiptsFolder;
/// The owner and mode bits a fixture's own files carry.
use std::os::unix::fs::PermissionsExt;
/// A borrowed path,
/// and the owned ones this fixture keeps.
use std::path::{
    Path,
    PathBuf,
};

/// The slot this fixture records,
/// which is the one managed clone below the container.
pub(crate) const THE_SLOT: &str = "one";

/// The mode a fixture's file is written with,
/// which is a file this account alone writes and every reader may read.
const FILE_MODE: u32 = 0o644;

/// The mode a fixture's folder is made with.
const FOLDER_MODE: u32 = 0o755;

/// What:
///  One control's fixture:
///  its root,
///  the places under it,
///  the account's cache,
///  the checkout whose `.claude/worktrees` the walks reach,
///  and the two readers whose owner terms are the account's.
/// Why:
///  A control gathers with the places and the readers together,
///  so one value hands all of them over and no control writes a place of its
///  own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheFixturesReaders = { places; checkout; readiness; receipts };
/// ```
pub(crate) struct TheFixturesReaders {
    /// Every place under the fixture's root.
    pub places: TheRoutePlaces,
    /// The checkout the layout names,
    /// whose `.claude/worktrees` folder the walks reach.
    pub checkout: PathBuf,
    /// The handler's reader of the readiness record.
    pub readiness: ReadinessFile,
    /// The folder the handler files its receipts in.
    pub receipts: ReceiptsFolder,
}

impl TheFixturesReaders {
    /// What:
    ///  One fixture at the design's own state,
    ///  written into a folder of this call's own.
    /// Why:
    ///  Every place is a file or a folder a control may then change,
    ///  and the two records are published through the release's writer,
    ///  so the gathering code reads what a writer of this crate makes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fixture = TheFixturesReaders.made('the-gather');
    /// ```
    pub(crate) fn made(name: &str) -> Self {
        let root: PathBuf = fixture(name);
        let places: TheRoutePlaces = TheRoutePlaces::under(root.as_path());
        let checkout: PathBuf = root.join("checkout");
        let cache: PathBuf = places.the_cache_or_why().expect("a fixture names a home");
        for folder in [
            checkout.as_path(),
            checkout.join(".claude/worktrees").as_path(),
            checkout.join(".claude/worktrees").join(THE_SLOT).as_path(),
            places.readiness.as_path(),
            places.results.as_path(),
            cache.as_path(),
            places.store.as_path(),
            places
                .zedlet
                .parent()
                .expect("the zedlet's folder")
                .to_path_buf()
                .as_path(),
        ] {
            make_folder(folder);
        }
        let account: u32 = this_account();
        let readiness: ReadinessFile = ReadinessFile::at(places.readiness.clone(), account);
        let receipts: ReceiptsFolder = ReceiptsFolder::at(places.results.clone(), account);
        let fixture: TheFixturesReaders = TheFixturesReaders {
            places,
            checkout,
            readiness,
            receipts,
        };
        fixture.write_the_layout();
        fixture.write_the_mount_table();
        fixture.write_the_boot_identity();
        fixture.record_the_identities();
        return fixture;
    }

    /// What:
    ///  The store folder,
    ///  opened the way the grammar opens one a control reads,
    ///  and the refusal of a folder that could not be opened.
    /// Why:
    ///  The route opens the store under root control and a fixture's folder is
    ///  the account's,
    ///  so a control opens it through the open that checks no control term,
    ///  and the same reading code then answers a record's facts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const store = fixture.store(); // SlotIdentityStore | StoreRefusal
    /// ```
    pub(crate) fn store(&self) -> Result<SlotIdentityStore, StoreRefusal> {
        return SlotIdentityStore::open(self.places.store.as_path());
    }

    /// What:
    ///  The configuration that names this fixture's three layout facts.
    /// Why:
    ///  The layout is read from the file the handler installs,
    ///  so a control's names must arrive through that file rather than through
    ///  a constant of the gather.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fixture.writeTheLayout(); // { cloneMounts: { checkout, parent, checkoutDataset } }
    /// ```
    pub(crate) fn write_the_layout(&self) -> PathBuf {
        let document: String = format!(
            "{{\n  \"cloneMounts\": {{\n    \"checkout\": \"{}\",\n    \"parent\": \"{CONTAINER_DATASET}\",\n    \"checkoutDataset\": \"{CHECKOUT_DATASET}\"\n  }}\n}}\n",
            self.checkout.display()
        );
        write_file(self.places.configuration.as_path(), document.as_str());
        return self.places.configuration.clone();
    }

    /// What:
    ///  The caller's mount table with the checkout mounted at the fixture's
    ///  path and the cache under it.
    /// Why:
    ///  The mount checks compare the table's rows with the layout's path and
    ///  dataset,
    ///  so a fixture's table must name the same two and one private row for
    ///  each.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fixture.writeTheMountTable(); // rows for /, checkout, /run, cache
    /// ```
    pub(crate) fn write_the_mount_table(&self) -> PathBuf {
        let cache: PathBuf = self
            .places
            .the_cache_or_why()
            .expect("a fixture names a home");
        let text: String = [
            mount_row(1, 0, "/", "/", "zfs", "zpcachyos/ROOT/cos/root", ""),
            mount_row(
                40,
                1,
                "/",
                self.checkout.display().to_string().as_str(),
                "zfs",
                CHECKOUT_DATASET,
                "",
            ),
            mount_row(41, 1, "/", "/run", "tmpfs", "tmpfs", ""),
            mount_row(
                42,
                1,
                "/",
                cache.display().to_string().as_str(),
                "zfs",
                "zpcachyos/nosnap/user-cache",
                "",
            ),
        ]
        .join("\n");
        write_file(self.places.mount_table.as_path(), text.as_str());
        return self.places.mount_table.clone();
    }

    /// What:
    ///  This boot's identifier,
    ///  as the kernel would write it.
    /// Why:
    ///  The readiness record and every receipt are judged against the boot the
    ///  kernel names,
    ///  so a fixture must write the file the gather reads rather than state a
    ///  string the gather never sees.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fixture.writeTheBootIdentity(); // '4c4c…\n'
    /// ```
    pub(crate) fn write_the_boot_identity(&self) -> PathBuf {
        write_file(
            self.places.boot_identity.as_path(),
            format!("{LIVE_BOOT}\n").as_str(),
        );
        return self.places.boot_identity.clone();
    }

    /// What:
    ///  The checkout's `.claude/worktrees` folder,
    ///  which the parent's walk and every slot's walk start at.
    /// Why:
    ///  A control that adds a slot folder or changes one writes it here,
    ///  so the walk reaches the folder the control made.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const worktrees = fixture.worktrees(); // `${root}/checkout/.claude/worktrees`
    /// ```
    pub(crate) fn worktrees(&self) -> PathBuf {
        return self.checkout.join(".claude").join("worktrees");
    }

    /// What:
    ///  The parent's record and one slot's record,
    ///  captured from the fixture's own folders.
    /// Why:
    ///  The identity checks compare records with live folders,
    ///  and a record a control wrote by hand would prove only that the reader
    ///  parses it,
    ///  so the records are the ones the release's writer publishes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// fixture.recordTheIdentities(); // .parent and one, each captured from its folder
    /// ```
    pub(crate) fn record_the_identities(&self) {
        let parent: PathBuf = self.worktrees();
        let slot: PathBuf = parent.join(THE_SLOT);
        let parent_identity: CloneFolderIdentity =
            capture(parent.as_path()).expect("the parent folder's identity");
        publish_parent_identity(
            self.places.store.as_path(),
            &parent_identity,
            &parent_identity,
        )
        .expect("the parent's record");
        let slot_identity: CloneFolderIdentity =
            capture(slot.as_path()).expect("the slot folder's identity");
        publish_slot_identity(
            self.places.store.as_path(),
            THE_SLOT,
            &slot_identity,
            &parent_identity,
        )
        .expect("the slot's record");
    }
}

/// What:
///  One folder's identity,
///  read through an open descriptor as the route reads it.
/// Why:
///  The record must hold the identity the walk will read,
///  and the identity is read from the folder's own descriptor rather than from
///  a path looked at twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = capture(folder); // { device, inode, owner, mode, acls }
/// ```
pub(crate) fn capture(folder: &Path) -> std::io::Result<CloneFolderIdentity> {
    let opened: std::fs::File = std::fs::File::open(folder)?;
    return CloneFolderIdentity::capture(&opened);
}

/// What:
///  Make a folder with the mode this fixture's folders carry,
///  and its parents.
/// Why:
///  The store's and the receipts' readers judge a folder's mode,
///  so a fixture's folder is made with the one mode the design names rather
///  than the account's umask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// makeFolder('/root/…/run'); // mkdir -p -m 0755
/// ```
pub(crate) fn make_folder(folder: &Path) {
    std::fs::create_dir_all(folder).expect("a fixture folder");
    std::fs::set_permissions(folder, std::fs::Permissions::from_mode(FOLDER_MODE))
        .expect("a fixture folder's mode");
}

/// What:
///  Write one file with the mode a fixture's files carry.
/// Why:
///  A configuration the account alone may write and a table every account may
///  read are the two shapes the reads judge,
///  and the umask would leave the mode to the shell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFile('/root/…/boot_id', '4c4c…\n', { mode: 0o644 });
/// ```
pub(crate) fn write_file(
    path: &Path,
    text: &str,
) {
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).expect("a fixture file's folder");
    }
    std::fs::write(path, text.as_bytes()).expect("a fixture file");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(FILE_MODE))
        .expect("a fixture file's mode");
}
