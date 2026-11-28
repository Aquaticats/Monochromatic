//! What:
//!  The state an arm commits into the fixture's checkout:
//!  the worktree's tracked files,
//!  the ignored folders one arm leaves for the next,
//!  and the readiness report,
//!  the probe record and the wrapper's own link that the route requires.
//! Why:
//!  The route serves an add from a clean checkout and from nowhere else,
//!  so every arm writes its state,
//!  commits it,
//!  and takes the previous arm's away through one place.
//!  The fixture itself is `binary_zfs_step_21_guest_fixture.rs`'s,
//!  and this file's methods are that struct's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fixture.commit([['package.json', manifest]], 'the node project'); fixture.removeIgnored(['node_modules']);
//! ```

#![cfg(target_os = "linux")]

use super::zfs_step_21_guest_fixture::Fixture;
use super::zfs_step_21_guest_layout::WRAPPER;
use super::zfs_sweep_guest::zfs_ignored;
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness::{
    ReadinessRecord,
    ReadyState,
    SweepState,
};
use git_policy_cli::worktree_clone_mount_protocol::clone_mount_readiness_file::ReadinessFile;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use std::time::Duration;

impl Fixture {
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
        if !self.something_changed() {
            return;
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

    /// What:
    ///  Whether the checkout holds anything Git would commit.
    /// Why:
    ///  Two arms prepare the same state,
    ///  and an arm that writes the bytes another arm already committed has
    ///  nothing to commit,
    ///  which Git refuses rather than accepting an empty commit.
    fn something_changed(&self) -> bool {
        let status: String =
            String::from_utf8_lossy(self.git(&["status", "--porcelain"]).stdout.as_slice())
                .to_string();
        return !status.is_empty();
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
        if !self.something_changed() {
            return;
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
    pub(super) fn zfs_ignored(
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
    pub(super) fn make_the_repository(&self) {
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
    pub(super) fn make_the_worktrees_folder(&self) {
        let parent: PathBuf = self.root.join(".claude").join("worktrees");
        std::fs::create_dir_all(parent.as_path()).expect("the worktrees folder");
        std::fs::set_permissions(parent.as_path(), std::fs::Permissions::from_mode(0o755))
            .expect("the folder's mode");
    }

    /// The readiness report the add's preflight requires,
    /// written as the boot unit would.
    pub(super) fn publish_the_readiness(&self) {
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
    pub(super) fn make_the_wrapper_link(&self) {
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
    pub(super) fn record(
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
    pub(super) fn origin_of(
        &self,
        name: &str,
    ) -> Option<String> {
        let record: String = self.record(name)?;
        return record
            .lines()
            .find_map(|line| return line.strip_prefix("originSnapshot="))
            .map(String::from);
    }
}
