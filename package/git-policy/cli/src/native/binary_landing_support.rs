//! What:
//!  A repository with one prepared commit waiting in a shadow repository, and the transaction around it,
//!  for controls that run the landing attempt.
//! Why:
//!  A landing needs a real repository, a shadow holding the prepared commit, the invocation's index snapshots,
//!  a published transaction directory with its preparing record, and the landing lock.
//!  The same builder serves controls that land in this process and controls that land in a child process which
//!  a phase marker then kills or pauses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const world = await buildLandingWorld(); const outcome = await landTransaction({ ...world });
//! ```

/// Shared fixtures and the bounded process helpers.
use super::support::{Fixture, REAL_GIT, TRANSACTION_ID, direct, git, repository};
use git_policy_cli::capture_invocation::{InvocationCapture, capture_invocation_base, capture_invocation_layout};
use git_policy_cli::landing_attempt::{
    LandingError, LandingOutcome, LandingPayload, LandingRequest, LandingWorkspace, land_transaction,
};
use git_policy_cli::owner_lock::OwnerLock;
use git_policy_cli::recovery_scan::acquire_registry_lock;
use git_policy_cli::shadow_git::shadow_repository_path;
use git_policy_cli::transaction_git::GitContext;
use git_policy_cli::transaction_journal::{Base, PREPARING_FILENAME, PreparingRecord, TransactionMode};
use git_policy_cli::transaction_journal_encode::encode_preparing;
use git_policy_cli::transaction_owner::{current_transaction_owner, encode_transaction_owner};
use git_policy_cli::transaction_registry::{LANDING_LOCK_NAME, TRANSACTION_ROOT_NAME, ensure_transaction_root, publish_transaction_directory};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Environment variable naming the world a child process lands in.
pub const WORLD_VARIABLE: &str = "CLI_GIT_LANDING_WORLD";

/// The paths and object IDs of one landing world.
#[derive(Clone, Debug)]
pub struct World {
    /// Fixture root.
    pub root: PathBuf,
    /// Repository worktree.
    pub repo: PathBuf,
    /// Its Git directory.
    pub git_dir: PathBuf,
    /// Transaction registry.
    pub registry: PathBuf,
    /// Transaction directory the registry publishes.
    pub directory: PathBuf,
    /// Shadow repository holding the prepared commit.
    pub shadow: PathBuf,
    /// Scratch files built before the transaction is published.
    pub stage: PathBuf,
    /// The branch's commit when the world was built.
    pub base: String,
    /// The prepared commit that changes `a.txt` from `one` to `two`.
    pub prepared: String,
    /// The prepared commit's tree.
    pub prepared_tree: String,
    /// The base commit's tree.
    pub base_tree: String,
    /// A prepared root commit with no parent, for an unborn target.
    pub root_commit: String,
}

/// What a transaction opened in a world carries.
pub struct Opened {
    /// The invocation facts, read from the repository as it is when the transaction opens.
    pub capture: InvocationCapture,
    /// The workspace a landing reads and marks.
    pub workspace: LandingWorkspace,
}

/// Real Git's trimmed standard output, run hermetically with extra environment.
pub fn git_with(fixture: &Fixture, directory: &Path, environment: &[(&str, &Path)], arguments: &[&str]) -> String {
    let mut command = direct(fixture);
    command.current_dir(directory).args(arguments);
    for (name, value) in environment {
        command.env(name, value);
    }
    let output = command.output().expect("Git starts");
    assert!(output.status.success(), "git {arguments:?} failed: {}", String::from_utf8_lossy(&output.stderr));
    return String::from_utf8_lossy(&output.stdout).trim_end().to_string();
}

/// Real Git's trimmed standard output.
pub fn git_text(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> String {
    return String::from_utf8_lossy(&git(fixture, directory, arguments).stdout).trim_end().to_string();
}

/// A path as UTF-8 text.
pub fn text(path: &Path) -> String {
    return path.to_str().expect("UTF-8 fixture path").to_string();
}

/// The Git context controls use: real Git with an isolating environment.
pub fn context() -> GitContext {
    let mut overlay: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in [
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ] {
        overlay.push((OsString::from(name), OsString::from(value)));
    }
    // The context names no repository, so the input files of its commands are made in the scratch directory beside
    // the fixtures, under the target directory and never in the system temporary directory.
    return GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay,
        global_prefix: Vec::new(),
        common_dir: Some(super::support::fixture_root()),
    };
}

/// Impl block: building a world and opening transactions in it.
impl World {
    /// What:
    ///  Build a repository whose `main` holds `a.txt` as `one`,
    ///  and a shadow repository holding a prepared commit that changes it to `two`.
    /// Why:
    ///   The shadow reaches the real objects through alternates,
    ///  so the prepared commit is the only object the landing has to migrate.
    pub fn build(fixture: &Fixture) -> World {
        let repo: PathBuf = repository(fixture, std::ffi::OsStr::new("repo"));
        std::fs::write(repo.join("a.txt"), b"one\n").expect("a.txt");
        git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
        git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
        let git_dir: PathBuf = repo.join(".git");
        let base: String = git_text(fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        let base_tree: String = git_text(fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
        let shadow: PathBuf = shadow_repository_path(git_dir.as_path(), TRANSACTION_ID);
        std::fs::create_dir_all(shadow.parent().expect("shadow parent")).expect("shadow parent");
        git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", text(shadow.as_path()).as_str()]);
        std::fs::create_dir_all(shadow.join("objects/info")).expect("shadow info");
        std::fs::write(shadow.join("objects/info/alternates"), format!("{}\n", git_dir.join("objects").display()))
            .expect("alternates");
        let stage: PathBuf = fixture.root.join("stage");
        std::fs::create_dir(&stage).expect("stage");
        std::fs::copy(git_dir.join("index"), stage.join("captured.index")).expect("captured index");
        std::fs::copy(git_dir.join("index"), stage.join("commit.index")).expect("commit index");
        std::fs::write(stage.join("a-two.txt"), b"two\n").expect("blob source");
        let shadow_dir: String = format!("--git-dir={}", shadow.display());
        let blob: String = git_text(
            fixture,
            repo.as_path(),
            &[shadow_dir.as_str(), "hash-object", "-w", text(stage.join("a-two.txt").as_path()).as_str()],
        );
        let commit_index: PathBuf = stage.join("commit.index");
        let cacheinfo: String = format!("100644,{blob},a.txt");
        git_with(
            fixture,
            repo.as_path(),
            &[("GIT_INDEX_FILE", commit_index.as_path())],
            &[shadow_dir.as_str(), "update-index", "--cacheinfo", cacheinfo.as_str()],
        );
        let prepared_tree: String = git_with(
            fixture,
            repo.as_path(),
            &[("GIT_INDEX_FILE", commit_index.as_path())],
            &[shadow_dir.as_str(), "write-tree"],
        );
        let prepared: String = git_text(
            fixture,
            repo.as_path(),
            &[shadow_dir.as_str(), "commit-tree", prepared_tree.as_str(), "-p", base.as_str(), "-m", "prepared"],
        );
        let root_commit: String = git_text(
            fixture,
            repo.as_path(),
            &[shadow_dir.as_str(), "commit-tree", prepared_tree.as_str(), "-m", "prepared root"],
        );
        // The worktree already holds what the prepared commit lands.
        std::fs::write(repo.join("a.txt"), b"two\n").expect("worktree a.txt");
        let world: World = World::attach_with(fixture.root.as_path(), base, prepared, prepared_tree, base_tree, root_commit);
        std::fs::write(
            world.stage.join("oids"),
            format!(
                "{}\n{}\n{}\n{}\n{}\n",
                world.base, world.prepared, world.prepared_tree, world.base_tree, world.root_commit
            ),
        )
        .expect("recorded object IDs");
        return world;
    }

    /// What:
    ///  The world's paths from its fixture root and object IDs.
    /// Why:
    ///   Every path derives from the root and the fixed transaction ID.
    fn attach_with(root: &Path, base: String, prepared: String, prepared_tree: String, base_tree: String, root_commit: String) -> World {
        let repo: PathBuf = root.join("repo");
        let git_dir: PathBuf = repo.join(".git");
        let registry: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
        let world: World = World {
            root: root.to_path_buf(),
            directory: registry.join(TRANSACTION_ID),
            shadow: shadow_repository_path(git_dir.as_path(), TRANSACTION_ID),
            stage: root.join("stage"),
            repo,
            git_dir,
            registry,
            base,
            prepared,
            prepared_tree,
            base_tree,
            root_commit,
        };
        return world;
    }

    /// What:
    ///  The world a parent process built, read back by a child process.
    /// Why:
    ///   The child must open its own transaction so that it owns the lock and the directory.
    pub fn attach(root: &Path) -> World {
        let recorded: String = std::fs::read_to_string(root.join("stage/oids")).expect("recorded object IDs");
        let ids: Vec<&str> = recorded.lines().collect();
        return World::attach_with(
            root,
            ids[0].to_string(),
            ids[1].to_string(),
            ids[2].to_string(),
            ids[3].to_string(),
            ids[4].to_string(),
        );
    }

    /// What:
    ///  Publish the transaction directory owned by this process,
    ///  with the private indexes and `preparing.json`,
    ///  and capture the invocation as the repository is now.
    /// Why:
    ///   The state a commit reaches before it lands.
    pub fn open(&self) -> Opened {
        let git: GitContext = context();
        ensure_transaction_root(self.registry.as_path()).expect("registry");
        let owner = current_transaction_owner(TRANSACTION_ID, "2026-10-11T00:00:00.000Z").expect("owner");
        publish_transaction_directory(
            self.registry.as_path(),
            TRANSACTION_ID,
            encode_transaction_owner(&owner).as_bytes(),
        )
        .expect("published");
        std::fs::copy(self.stage.join("captured.index"), self.directory.join("captured.index")).expect("captured index");
        std::fs::copy(self.stage.join("commit.index"), self.directory.join("commit.index")).expect("commit index");
        let layout = capture_invocation_layout(&git, self.repo.as_path(), false).expect("layout");
        let capture: InvocationCapture = capture_invocation_base(&git, self.repo.as_path(), layout).expect("base");
        let record: PreparingRecord = PreparingRecord {
            transaction_id: TRANSACTION_ID.to_string(),
            mode: TransactionMode::Index,
            base: capture.base.clone(),
            symbolic_head: capture.layout.symbolic_head.clone(),
            target_ref: capture.layout.target_ref.clone(),
            conclusion: capture.layout.conclusion,
            repository_root: text(capture.layout.repository_root.as_path()),
            git_dir: text(capture.layout.git_dir.as_path()),
            common_dir: text(capture.layout.common_dir.as_path()),
            real_index_path: text(capture.layout.real_index_path.as_path()),
            object_directory: text(capture.layout.object_directory.as_path()),
            ref_format: capture.layout.ref_format,
            empty_tree_oid: capture.layout.empty_tree_oid.clone(),
            shadow_path: text(self.shadow.as_path()),
            selected_pathspecs: Vec::new(),
            invoked_at: capture.layout.invoked_at.clone(),
        };
        std::fs::write(self.directory.join(PREPARING_FILENAME), encode_preparing(&record)).expect("preparing");
        let workspace: LandingWorkspace = LandingWorkspace {
            transaction_id: TRANSACTION_ID.to_string(),
            directory: self.directory.clone(),
            shadow: self.shadow.clone(),
            captured_index: self.directory.join("captured.index"),
            commit_index: self.directory.join("commit.index"),
            preserved: false,
        };
        return Opened { capture, workspace };
    }

    /// What:
    ///  The landing lock of the registry,
    ///  held by this process.
    /// Why:
    ///   The caller of a landing holds it.
    pub fn landing_lock(&self) -> OwnerLock {
        return acquire_registry_lock(self.registry.as_path(), LANDING_LOCK_NAME).expect("landing lock");
    }

    /// What:
    ///  The payload landing the prepared commit onto the base.
    /// Why:
    ///   The common case; controls change one field to reach the others.
    pub fn commit_payload(&self) -> LandingPayload {
        return LandingPayload::Commit {
            new_oid: self.prepared.clone(),
            landed_tree_oid: self.prepared_tree.clone(),
            expected_old: Base::Commit(self.base.clone()),
            landed_index: self.directory.join("commit.index"),
            exact_private_index: true,
            guarded_paths: Vec::new(),
        };
    }

    /// The real index path.
    pub fn index(&self) -> PathBuf {
        return self.git_dir.join("index");
    }

    /// The real index lock path.
    pub fn index_lock(&self) -> PathBuf {
        return self.git_dir.join("index.lock");
    }

    /// The real index lock's PID file path.
    pub fn pid_file(&self) -> PathBuf {
        return self.git_dir.join("index~pid.lock");
    }

    /// Every `.keep` file of the real object store.
    pub fn keeps(&self) -> Vec<String> {
        let pack: PathBuf = self.git_dir.join("objects/pack");
        let mut names: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(pack) {
            for entry in entries.flatten() {
                let name: String = entry.file_name().to_string_lossy().into_owned();
                if name.ends_with(".keep") {
                    names.push(name);
                }
            }
        }
        return names;
    }
}

/// What:
///  Run one landing attempt of the payload in a world,
///  holding the landing lock for its duration.
/// Why:
///   The caller of a landing holds the landing lock,
///  so every control lands through the same entry.
pub fn land(world: &World, opened: &mut Opened, payload: &LandingPayload) -> Result<LandingOutcome, LandingError> {
    let lock: OwnerLock = world.landing_lock();
    let request: LandingRequest<'_> = LandingRequest {
        capture: &opened.capture,
        mode: TransactionMode::Index,
        payload,
        committed_paths: &[],
        added_paths: &[],
        selected_worktree_paths: &[],
        index_lock_timeout_ms: 300,
        attempt: 1,
    };
    let result: Result<LandingOutcome, LandingError> =
        land_transaction(&context(), world.repo.as_path(), &lock, &mut opened.workspace, &request);
    drop(lock);
    return result;
}
