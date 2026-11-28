//! What:
//!  The guest fixture plan step 21's arms add worktrees from:
//!  the repository the arms commit their state into,
//!  the environment every command runs with,
//!  and the one add the arms drive,
//!  which mounts the clone when the route's own intent asks for it.
//! Why:
//!  Each arm needs a clean checkout,
//!  a wrapper it starts as `git`,
//!  and an add whose mount the arm makes at the moment it chooses,
//!  so one struct owns those and the records a slot leaves behind.
//!  The layout it stands on,
//!  the guest variables and the readings of a mount are
//!  `binary_zfs_step_21_guest_layout.rs`'s,
//!  and the state an arm commits is `binary_zfs_step_21_guest_repository.rs`'s.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fixture = Fixture.prepare(); const added = fixture.add('pnpm-0'); fixture.clear('pnpm-0');
//! ```

#![cfg(target_os = "linux")]

use super::zfs_step_21_guest_layout::{
    ADD_WAIT,
    POLL,
    POOL_VARIABLE,
    REAL_GIT,
    ROOT_VARIABLE,
    SBIN_VARIABLE,
    dataset_stands,
    make_the_layout,
    the_intent_says_mount,
};
use super::zfs_sweep_guest::{
    prepare_the_store,
    required,
    unmount_at,
    write_configuration,
    write_passing_probe_record,
};
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
use std::time::{
    Duration,
    Instant,
};

/// What one add came to.
pub(super) struct AddRound {
    /// The wall time from the command's start to its end.
    pub(super) elapsed: Duration,
    /// The status the command exited with.
    pub(super) status: Option<i32>,
    /// What the command printed on both streams.
    pub(super) said: String,
}

/// The fixture one run of the arms holds:
/// the design's layout,
/// a repository,
/// and the wrapper's own link.
pub(super) struct Fixture {
    /// The checkout root,
    /// which is the mount point of the checkout dataset.
    pub(super) root: PathBuf,
    /// The pool's name.
    pub(super) pool: String,
    /// The OpenZFS userspace.
    pub(super) sbin: String,
    /// The checkout dataset the layout made.
    pub(super) checkout: String,
    /// The clone container the layout made.
    pub(super) container: String,
}

impl Fixture {
    /// Make the design's layout,
    /// the repository and the wrapper's link.
    pub(super) fn prepare() -> Fixture {
        let root: PathBuf = PathBuf::from(required(ROOT_VARIABLE));
        let pool: String = required(POOL_VARIABLE);
        let sbin: String = required(SBIN_VARIABLE);
        let (checkout, container): (String, String) = make_the_layout(&sbin, &pool, root.as_path());
        let fixture: Fixture = Fixture {
            root,
            pool,
            sbin,
            checkout,
            container,
        };
        fixture.make_the_repository();
        fixture.make_the_worktrees_folder();
        fixture.publish_the_readiness();
        write_passing_probe_record();
        prepare_the_store(fixture.root.as_path());
        write_configuration(fixture.root.as_path(), None, None);
        fixture.make_the_wrapper_link();
        return fixture;
    }

    /// The checkout root.
    pub(super) fn root(&self) -> &Path {
        return self.root.as_path();
    }

    /// The folder Git's common directory is.
    pub(super) fn common(&self) -> PathBuf {
        return self.root.join(".git");
    }

    /// The records' home folder.
    pub(super) fn home_records(&self) -> PathBuf {
        return self.common().join("monochromatic-git-policy-cli");
    }

    /// The folder each arm runs with as `HOME`,
    /// outside the checkout so the source stays clean and so every install of
    /// one round reads one store.
    pub(super) fn home_folder(&self) -> PathBuf {
        return PathBuf::from("/root/step21-home");
    }

    /// The session's runtime folder,
    /// short enough that the service's socket fits one address.
    pub(super) fn runtime_folder(&self) -> PathBuf {
        return PathBuf::from("/root/r");
    }

    /// The folder the wrapper's link sits in.
    pub(super) fn bin_folder(&self) -> PathBuf {
        return PathBuf::from("/root/step21-bin");
    }

    /// One slot's folder,
    /// which the design keeps under the checkout.
    pub(super) fn slot(
        &self,
        name: &str,
    ) -> PathBuf {
        return self.root.join(".claude").join("worktrees").join(name);
    }

    /// One slot's clone dataset.
    pub(super) fn clone(
        &self,
        name: &str,
    ) -> String {
        return format!("{}/{}", self.container, name);
    }

    /// The folder a control arm works in,
    /// on the checkout dataset,
    /// so a control and the clone's own preparation differ only by the
    /// filesystem the work happens on.
    pub(super) fn control(
        &self,
        arm: &str,
        round: usize,
    ) -> PathBuf {
        return self
            .root
            .join(".step21")
            .join(arm)
            .join(format!("round-{round}"));
    }

    /// The wrapper started as `git`,
    /// in a folder of its own.
    pub(super) fn wrapper_git(&self) -> PathBuf {
        return self.bin_folder().join("git");
    }

    /// The environment every command runs with,
    /// so no host variable reaches it and every arm searches the same path.
    pub(super) fn environment(
        &self,
        program_folder: &Path,
    ) -> Vec<(String, String)> {
        return vec![
            (
                String::from("PATH"),
                format!(
                    "{}:{}:/usr/local/bin:/root/.cargo/bin:/usr/bin:/bin",
                    program_folder.display(),
                    self.bin_folder().display()
                ),
            ),
            (
                String::from("HOME"),
                self.home_folder().display().to_string(),
            ),
            (
                String::from("XDG_RUNTIME_DIR"),
                self.runtime_folder().display().to_string(),
            ),
            // The toolchain the guest was prepared with lives under root's own folders, and the fixture's home is not root's.
            (String::from("CARGO_HOME"), String::from("/root/.cargo")),
            (String::from("RUSTUP_HOME"), String::from("/root/.rustup")),
            // The repair says which branch it took only in this log, and an arm reads branches as evidence.
            (String::from("MONOCHROMATIC_VERBOSE"), String::from("true")),
            (String::from("GIT_CONFIG_NOSYSTEM"), String::from("1")),
            (
                String::from("GIT_CONFIG_GLOBAL"),
                String::from("/nonexistent-global-config"),
            ),
            (String::from("GIT_AUTHOR_NAME"), String::from("Fixture")),
            (
                String::from("GIT_AUTHOR_EMAIL"),
                String::from("fixture@example.invalid"),
            ),
            (String::from("GIT_COMMITTER_NAME"), String::from("Fixture")),
            (
                String::from("GIT_COMMITTER_EMAIL"),
                String::from("fixture@example.invalid"),
            ),
        ];
    }

    /// One command in the fixture's environment,
    /// which must succeed.
    pub(super) fn run(
        &self,
        program: &Path,
        arguments: &[&str],
        directory: &Path,
    ) -> Output {
        let mut command_line: Command = Command::new(program);
        command_line
            .args(arguments)
            .current_dir(directory)
            .env_clear();
        for (key, value) in self.environment(Path::new("/usr/bin")) {
            command_line.env(key, value);
        }
        let output: Output = command_line
            .output()
            .unwrap_or_else(|error| panic!("{program:?} could not start: {error}"));
        assert!(
            output.status.success(),
            "{program:?} {arguments:?} in {} was refused: {}",
            directory.display(),
            String::from_utf8_lossy(output.stderr.as_slice())
        );
        return output;
    }

    /// One real Git command in the checkout,
    /// which must succeed.
    pub(super) fn git(
        &self,
        arguments: &[&str],
    ) -> Output {
        return self.run(Path::new(REAL_GIT), arguments, self.root.as_path());
    }

    /// Run the wrapper's own add,
    /// which is the ZFS clone route,
    /// mounting the clone when its intent appears and calling `before` first
    /// when the arm asks.
    pub(super) fn add(
        &self,
        name: &str,
        before_the_mount: Option<&mut dyn FnMut(&Path)>,
    ) -> AddRound {
        let slot: PathBuf = self.slot(name);
        let clone: String = self.clone(name);
        let mut hook: Option<&mut dyn FnMut(&Path)> = before_the_mount;
        let started: Instant = Instant::now();
        let mut command_line: Command = Command::new(self.wrapper_git());
        command_line
            .args(["worktree", "add", slot.to_str().expect("a text path")])
            .current_dir(self.root.as_path())
            .env_clear();
        for (key, value) in self.environment(Path::new("/usr/bin")) {
            command_line.env(key, value);
        }
        command_line.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child: Child = command_line.spawn().expect("the wrapper starts");
        let mut attach_made: bool = false;
        while started.elapsed() < ADD_WAIT {
            if let Ok(Some(_status)) = child.try_wait() {
                break;
            }
            if !attach_made
                && slot.is_dir()
                && dataset_stands(self.sbin.as_str(), clone.as_str())
                && the_intent_says_mount(self.sbin.as_str(), clone.as_str())
            {
                if let Some(call) = hook.take() {
                    call(slot.as_path());
                }
                let output: Output = Command::new("/usr/bin/mount")
                    .args([
                        "-t",
                        "zfs",
                        "-o",
                        "nosuid,nodev",
                        clone.as_str(),
                        slot.to_str().expect("a text path"),
                    ])
                    .output()
                    .expect("mount starts");
                assert!(
                    output.status.success(),
                    "the mount of {clone} at the slot was refused: {}",
                    String::from_utf8_lossy(output.stderr.as_slice())
                );
                attach_made = true;
            }
            std::thread::sleep(POLL);
        }
        let output: Output = child.wait_with_output().expect("the add ends");
        return AddRound {
            elapsed: started.elapsed(),
            status: output.status.code(),
            said: format!(
                "{}{}",
                String::from_utf8_lossy(output.stdout.as_slice()),
                String::from_utf8_lossy(output.stderr.as_slice())
            ),
        };
    }

    /// Take a slot's clone,
    /// mount,
    /// folder,
    /// registration and records away,
    /// so the next round starts from nothing.
    pub(super) fn clear(
        &self,
        name: &str,
    ) {
        let slot: PathBuf = self.slot(name);
        let clone: String = self.clone(name);
        unmount_at(slot.as_path());
        let origin: Option<String> = self.origin_of(name);
        self.zfs_ignored(&["destroy", "-r", "-f", clone.as_str()]);
        if let Some(snapshot) = origin {
            self.zfs_ignored(&["destroy", "-f", snapshot.as_str()]);
        }
        let _folder: std::io::Result<()> = std::fs::remove_dir_all(slot.as_path());
        let _records: std::io::Result<()> = std::fs::remove_dir_all(self.home_records().as_path());
        let _pruned: Output = self.run(
            Path::new(REAL_GIT),
            &["worktree", "prune"],
            self.root.as_path(),
        );
    }
}
