//! What:
//!  One arm's fixture for the forced removal's guest work:
//!  a pool made from a file,
//!  the design's checkout above it with one real registration and one real
//!  clone,
//!  and the names every arm of this file's siblings reads.
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
//!  so an arm that died leaves the next one a machine it cannot see.
//!  What the pool answers an arm is `binary_zfs_removal_force_guest_pool.rs`,
//!  how a person's wrapper is started is
//!  `binary_zfs_removal_force_guest_wrapper.rs`,
//!  and the records an add leaves are
//!  `binary_zfs_removal_force_guest_records.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const arm = Arm.make(1);
//! arm.attachAtTheSlot();
//! arm.tearDown();
//! ```

use super::zfs_removal_force_guest_records::write_the_origin_inventory;
use super::zfs_slot_state_guest_fixture::{
    Place,
    guid_of,
    publish,
};
use super::zfs_sweep_guest::{
    NONCE,
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
};
use git_policy_cli::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use git_policy_cli::worktree_clone_binding_record::binding_words::Phase;
use git_policy_cli::worktree_clone_mount_protocol::DatasetGuid;
use std::path::{
    Path,
    PathBuf,
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
///  One arm's pool,
///  checkout,
///  registration,
///  clone and records.
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
    /// The slot,
    /// its clone and the records the add wrote.
    pub(super) place: Place,
    /// The clone dataset,
    /// which is the name the record holds.
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
        // A run that died before its cleanup left this arm's own pool behind, and a pool cannot be made twice.
        let _leftover: (bool, String, String) = zfs(
            sbin.as_str(),
            "zpool",
            &["destroy", "-f", pool.as_str()],
            None,
        );
        let _wreckage: std::io::Result<()> = std::fs::remove_dir_all(root.as_path());
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
        // The checkout's slot chain is inside the checkout's own dataset and is no tracked content, so the clone a
        // removal meets at a slot would hold one untracked entry (`.claude/`) and every arm without force would be
        // refused for a change nobody made. The ignore file is the checkout's own.
        std::fs::write(checkout.join(".gitignore"), ".claude/\n")
            .expect("the checkout's ignore file is written");
        command_ok(
            "git",
            &[
                "-C",
                shown(checkout.as_path()),
                "add",
                "a.txt",
                ".gitignore",
            ],
        );
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
        // The design's add writes no checkout, so the underlay holds Git's pointer and nothing else. This fixture checks
        // the slot out, because the cleanliness reading needs the registration's index, so the files it wrote into the
        // underlay are taken out again: the clone carries the checkout's files and the underlay is the empty folder the
        // settle removes.
        for listed in std::fs::read_dir(slot.as_path()).expect("the underlay lists") {
            let entry: std::fs::DirEntry = listed.expect("one underlay entry");
            if entry.file_name() == std::ffi::OsStr::new(".git") {
                continue;
            }
            let path: PathBuf = entry.path();
            if entry.file_type().expect("the entry's type").is_dir() {
                std::fs::remove_dir_all(path.as_path()).expect("a checked-out folder goes");
            } else {
                std::fs::remove_file(path.as_path()).expect("a checked-out file goes");
            }
        }
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
        let arm: Arm = Arm {
            sbin,
            pool,
            root,
            checkout,
            place,
            clone,
            clone_guid,
        };
        // The handler's unmount turn checks the account's cache against the pin before either `umount2`, so the
        // fixture pins its own cache dataset for every arm and not only the one about a staged clone.
        arm.pin_the_cache();
        return arm;
    }

    /// The `PATH` the guest's OpenZFS userspace needs.
    pub(super) fn search_path(&self) -> String {
        let prefix: &str = self.sbin.trim_end_matches("/sbin");
        return format!(
            "{prefix}/sbin:{prefix}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
        );
    }

    /// What:
    ///  Take this arm's fixture away:
    ///  every mount it left,
    ///  the pool,
    ///  and the folder.
    /// Why:
    ///  A pool whose dataset is mounted cannot be destroyed,
    ///  and a leftover mount would make the next arm's checkout dataset busy.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// arm.tearDown();
    /// ```
    pub(super) fn tear_down(&self) {
        // A clone still mounted anywhere is what makes its pool's datasets busy, and an arm that panicked left one: the
        // cleanup takes every mount of the clone down, then the checkout's own.
        let _clone_gone: (bool, String, String) = zfs(
            self.sbin.as_str(),
            "zfs",
            &["unmount", "-f", self.clone.as_str()],
            None,
        );
        let _unmounted: (bool, String, String) = zfs(
            self.sbin.as_str(),
            "zfs",
            &["unmount", "-f", self.place.checkout_dataset.as_str()],
            None,
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
