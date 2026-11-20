//! What:
//!  The disposable encrypted pool a live test builds:
//! a 1 GiB image under the
//!  fixture root,
//! a raw key file beside it,
//! the checkout and clone-parent
//!  datasets the design names,
//! the grants of its delegated scope,
//! and the
//!  snapshots and clones each arm needs.
//! Why:
//!  Every measurement of this route ran on a separate file-backed pool with
//!  `aes-256-gcm`,
//! a raw key,
//! `recordsize=32K` and `compression=zstd-3`;
//! building
//!  the same shape here means a live test never depends on a pool it did not
//!  create,
//! and the pool is destroyed as the test ends.

/// The fixture that runs the programs,
/// and the release every live test needs.
use super::fixture::{Fixture, OPENZFS_RELEASE};
/// The path type the image and key need.
use std::path::PathBuf;

/// The encryption the measurements' fixture used.
const ENCRYPTION: &str = "aes-256-gcm";
/// The record size the measurements' fixture used.
const RECORDSIZE: &str = "32K";
/// The compression the measurements' fixture used.
const COMPRESSION: &str = "zstd-3";
/// The image size every item's pool held in the measurements.
const POOL_IMAGE_SIZE: &str = "1G";
/// The bytes an `aes-256-gcm` raw key holds.
const RAW_KEY_BYTES: usize = 32;
/// The prefix every pool this fixture creates carries,
/// so destroying a leftover
/// can never reach a pool it did not make.
pub(crate) const FIXTURE_POOL_PREFIX: &str = "zfsstep2";

/// One file-backed encrypted pool in the guest fixture.
pub(crate) struct FixturePool<'a> {
    /// The fixture that runs the programs.
    pub(crate) fixture: &'a Fixture,
    /// The pool's name.
    pub(crate) name: String,
    /// The dataset the checkout and the clone parent sit under.
    pub(crate) home: String,
    /// The checkout dataset.
    pub(crate) checkout: String,
    /// The unmounted container the clones are immediate children of.
    pub(crate) clones: String,
    /// Whether the pool still exists.
    armed: bool,
}

/// The methods of `FixturePool`.
impl<'a> FixturePool<'a> {
    /// Build `name` at the fixture root and create the three datasets the
    /// design's layout names.
    ///
    /// # Panics
    ///
    /// When the guest's userspace cannot create the pool or a dataset,
    /// with the
    /// program's standard error in the message.
    pub(crate) fn create(fixture: &'a Fixture, name: &str) -> FixturePool<'a> {
        assert!(
            name.starts_with(FIXTURE_POOL_PREFIX),
            "a fixture pool is named '{FIXTURE_POOL_PREFIX}...', and the name is '{name}'"
        );
        let mut leftover: FixturePool<'a> = FixturePool::adopt(fixture, name);
        leftover.armed = true;
        leftover.destroy();
        let folder: PathBuf = super::fixture::pool_folder(&fixture.root, name);
        std::fs::create_dir_all(&folder).expect("the pool's folder is created");
        let image: String = folder.join("pool.img").to_string_lossy().into_owned();
        let key: PathBuf = folder.join("raw-key");
        std::fs::write(&key, raw_key()).expect("the raw key file is written");
        let truncate: std::process::Output = std::process::Command::new("truncate")
            .arg(size_argument())
            .arg(&image)
            .output()
            .expect("truncate runs in the guest");
        assert!(
            truncate.status.success(),
            "creating the pool image failed: {truncate:?}"
        );
        let encryption: String = format!("encryption={ENCRYPTION}");
        let keylocation: String = format!("keylocation=file://{}", key.display());
        let recordsize: String = format!("recordsize={RECORDSIZE}");
        let compression: String = format!("compression={COMPRESSION}");
        fixture.zpool(&[
            "create",
            "-o",
            "cachefile=none",
            "-O",
            "mountpoint=none",
            "-O",
            encryption.as_str(),
            "-O",
            "keyformat=raw",
            "-O",
            keylocation.as_str(),
            "-O",
            recordsize.as_str(),
            "-O",
            compression.as_str(),
            name,
            &image,
        ]);
        let home: String = format!("{name}/home");
        let checkout: String = format!("{home}/checkout");
        let clones: String = format!("{home}/worktree-clone-datasets");
        let home_mountpoint: String = format!("mountpoint={}", folder.join("home").display());
        let checkout_mountpoint: String =
            format!("mountpoint={}", folder.join("home/checkout").display());
        fixture.zfs(&[
            "create",
            "-o",
            home_mountpoint.as_str(),
            "-o",
            "acltype=posix",
            &home,
        ]);
        fixture.zfs(&["create", "-o", checkout_mountpoint.as_str(), &checkout]);
        fixture.zfs(&[
            "create",
            "-o",
            "mountpoint=legacy",
            "-o",
            "canmount=off",
            &clones,
        ]);
        return FixturePool {
            fixture,
            name: name.to_string(),
            home,
            checkout,
            clones,
            armed: true,
        };
    }

    /// Take a handle on a pool another process created,
    /// which the delegated-account
    /// arms need and which nothing destroys on drop.
    pub(crate) fn adopt(fixture: &'a Fixture, name: &str) -> FixturePool<'a> {
        let home: String = format!("{name}/home");
        return FixturePool {
            fixture,
            name: name.to_string(),
            home: home.clone(),
            checkout: format!("{home}/checkout"),
            clones: format!("{home}/worktree-clone-datasets"),
            armed: false,
        };
    }

    /// Leave the pool in place when this handle drops,
    /// so the account arm can use
    /// what the root arm prepared.
    pub(crate) fn keep(mut self) {
        self.armed = false;
    }

    /// The design's proposed grants and nothing else:
    /// pool `delegation` on,
    /// local
    /// `create,mount` on the clone parent,
    /// descendent `clone` on the checkout,
    /// descendent
    /// `mount,destroy,userprop` on the clone parent.
    pub(crate) fn grant_the_proposed_scope(&self) {
        let account: &str = self.fixture.account.as_str();
        self.fixture.zpool(&["set", "delegation=on", &self.name]);
        self.fixture
            .zfs(&["allow", "-l", "-u", account, "create,mount", &self.clones]);
        self.fixture
            .zfs(&["allow", "-d", "-u", account, "clone", &self.checkout]);
        self.fixture.zfs(&[
            "allow",
            "-d",
            "-u",
            account,
            "mount,destroy,userprop",
            &self.clones,
        ]);
    }

    /// Snapshot `dataset` with each name.
    ///
    /// One `zfs snapshot` call per name:
    /// the program refuses several snapshots of
    /// one filesystem in one command,
    /// which is its own rule,
    /// not the kernel's.
    pub(crate) fn snapshot(&self, dataset: &str, names: &[&str]) {
        for name in names {
            let full: String = format!("{dataset}@{name}");
            self.fixture.zfs(&["snapshot", &full]);
        }
    }

    /// Clone `origin` as `<clones>/<name>` and return the new dataset's name.
    pub(crate) fn clone_of(&self, origin: &str, name: &str) -> String {
        let target: String = format!("{}/{name}", self.clones);
        self.fixture.zfs(&["clone", origin, &target]);
        return target;
    }

    /// Set `count` user properties of `length` bytes each on `dataset`,
    /// so a read's
    /// reply grows past the initial buffer.
    pub(crate) fn pad_user_properties(&self, dataset: &str, count: usize, length: usize) {
        let value: String = "p".repeat(length);
        for index in 0..count {
            let assignment: String = format!("monochromatic:pad{index}={value}");
            self.fixture.zfs(&["set", assignment.as_str(), dataset]);
        }
    }

    /// Whether `dataset` exists,
    /// asked of the pool itself.
    pub(crate) fn dataset_exists(&self, dataset: &str) -> bool {
        return self
            .fixture
            .zfs_succeeds(&["list", "-H", "-o", "name", dataset]);
    }

    /// The GUID and creation transaction group of every object `selection`
    /// names under `dataset`.
    pub(crate) fn identity_rows(&self, selection: &str, dataset: &str) -> Vec<(String, u64, u64)> {
        return self.fixture.identity_rows(selection, dataset);
    }

    /// Assert the loaded module is the design's release.
    pub(crate) fn assert_the_module_release(&self) {
        let version: String =
            std::fs::read_to_string("/sys/module/zfs/version").expect("the module is loaded");
        assert_eq!(version.trim(), OPENZFS_RELEASE);
    }

    /// Destroy the pool,
    /// every dataset under it,
    /// and the folder that held its
    /// image and raw key,
    /// so a run leaves the guest's fixture root empty.
    pub(crate) fn destroy(&mut self) {
        if !self.armed {
            return;
        }
        self.armed = false;
        let _ = std::process::Command::new(self.fixture.sbin.join("zpool"))
            .args(["destroy", "-f", &self.name])
            .output();
        let folder: std::path::PathBuf =
            super::fixture::pool_folder(&self.fixture.root, &self.name);
        let _ = std::fs::remove_dir_all(folder);
    }
}

/// The pool is destroyed when the test ends,
/// however it ends,
/// so a failed
/// assertion leaves no dataset behind.
impl Drop for FixturePool<'_> {
    /// Destroys the pool unless the test already did.
    fn drop(&mut self) {
        self.destroy();
    }
}

/// The argument that gives the pool image its size.
fn size_argument() -> String {
    return format!("--size={POOL_IMAGE_SIZE}");
}

/// A fresh raw key,
/// long enough for `aes-256-gcm` and never reused across pools.
fn raw_key() -> Vec<u8> {
    /// The trait that fills the key's bytes from the random device.
    use std::io::Read;
    let mut key: Vec<u8> = vec![0; RAW_KEY_BYTES];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut source| {
            return source.read_exact(&mut key);
        })
        .expect("the guest's random device fills the raw key");
    return key;
}
