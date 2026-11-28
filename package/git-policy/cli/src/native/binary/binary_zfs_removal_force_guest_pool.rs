//! What:
//!  What one arm's pool and its handler answer,
//!  and the sweeper a forced removal waits beside.
//! Why:
//!  Every one of these reads the guest's own mount table and dataset list,
//!  so an arm judges the pool's answer rather than a command's words,
//!  and the sweeper is the one child an arm keeps beside the wrapper it drives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sweeper = arm.startTheSweeper();
//! arm.stopTheSweeper(sweeper);
//! ```

use super::zfs_removal_force_guest_fixture::{
    Arm,
    shown,
};
use super::zfs_sweep_guest::{
    command_ok,
    mounted,
    rows_naming,
    sweep,
    zfs,
    zfs_ok,
};
use git_policy_cli::worktree_clone_binding_record::registration_home::plans_folder;
use git_policy_cli::worktree_clone_unmount::user_cache_pin::{
    UserCachePin,
    write_pin,
};
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    Stdio,
};

/// The folder the release's own pin record lives in,
/// which is `USER_CACHE_PIN_PATH`'s parent.
pub(super) const THE_PIN_FOLDER: &str = "/var/lib/monochromatic-git-policy-cli";

impl Arm {
    /// What:
    ///  Mount this arm's cache dataset at the account's cache and pin it.
    /// Why:
    ///  The handler's unmount turn opens the account's cache and checks it
    ///  against the pin before either `umount2`,
    ///  so every arm whose removal asks the handler for an unmount needs a
    ///  pinned cache and not only the arm about a staged clone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// arm.pinTheCache();
    /// ```
    pub(super) fn pin_the_cache(&self) {
        let dataset: String = format!("{}/cache", self.pool);
        zfs_ok(
            self.sbin.as_str(),
            &["create", "-o", "mountpoint=legacy", dataset.as_str()],
        );
        let cache_path: PathBuf = PathBuf::from("/root/.cache");
        std::fs::create_dir_all(cache_path.as_path()).expect("the account's cache folder is there");
        command_ok(
            "mount",
            &["-t", "zfs", dataset.as_str(), shown(cache_path.as_path())],
        );
        let facts: std::fs::Metadata =
            std::fs::metadata(cache_path.as_path()).expect("the cache's own facts read");
        let pin: UserCachePin = UserCachePin {
            source: dataset,
            root_inode: facts.ino(),
            device: facts.dev(),
        };
        std::fs::create_dir_all(Path::new(THE_PIN_FOLDER)).expect("the pin's folder is made");
        let folder: std::fs::File =
            std::fs::File::open(Path::new(THE_PIN_FOLDER)).expect("the pin's folder opens");
        write_pin(&folder, &pin).expect("the pin is written");
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
            mounted(self.clone.as_str()),
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
        // The handler's own words are kept beside the arm's log, because a removal that waits for a receipt is a step
        // whose answer only the handler can give.
        let log: String = format!("/mnt/shared/sweeps-{}.log", self.pool);
        return Command::new("sh")
            .args([
                "-c",
                "while :; do \"$0\" >> \"$1\" 2>&1; sleep 1; done",
                shown(binary.as_path()),
                log.as_str(),
            ])
            .env("PATH", self.search_path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the sweeper starts under the installed name");
    }

    /// End the sweeper this arm started,
    /// and wait for it.
    pub(super) fn stop_the_sweeper(
        &self,
        mut sweeper: Child,
    ) {
        let _ended: std::io::Result<std::process::ExitStatus> =
            sweeper.kill().and_then(|()| return sweeper.wait());
    }
    /// Make one snapshot of this arm's clone,
    /// named below `OWN_SNAPSHOT_PREFIX`.
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

    /// The plan folder's entries,
    /// which a refusal never creates.
    pub(super) fn plan_entries(&self) -> usize {
        return std::fs::read_dir(plans_folder(self.place.common.as_path()))
            .map_or(0, |entries| return entries.count());
    }

    /// Every mount row naming this arm's clone,
    /// by the pool's own source name.
    pub(super) fn mounts_of_the_clone(&self) -> Vec<String> {
        return rows_naming(self.clone.as_str());
    }
}
