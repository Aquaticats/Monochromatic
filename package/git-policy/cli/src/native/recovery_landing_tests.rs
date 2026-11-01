//! Controls for dead-transaction recovery against real repositories,
//!  and the scene builder the
//! registry and entry tests share.

use super::*;
use crate::capture_records::{
    CAPTURED_FILENAME, CapturedRecord, encode_captured_record, landed_record_directory,
};
use crate::recovery_files::file_identity_of;
use crate::recovery_files::tests::lock_identity_of;
use crate::recovery_inspect::tests::owner_bytes;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_journal::{AddedPath, FileIdentity, IndexLockRecord, LockIdentity};
use crate::transaction_journal_encode::tests::sample_preparing;
use crate::transaction_journal_encode::{
    encode_index_lock, encode_landing, encode_preparing, encode_ref_updated,
};
use crate::transaction_registry::{
    TRANSACTION_ROOT_NAME, ensure_transaction_root, publish_transaction_directory,
};

/// The transaction ID the fixtures use.
pub(crate) const ID: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// A repository with a registry and one published transaction directory.
pub(crate) struct Scene {
    /// Fixture root.
    pub(crate) root: PathBuf,
    /// Repository worktree.
    pub(crate) repo: PathBuf,
    /// Its Git directory.
    pub(crate) git_dir: PathBuf,
    /// Registry root.
    pub(crate) registry: PathBuf,
    /// Transaction directory.
    pub(crate) directory: PathBuf,
    /// Shadow repository path.
    pub(crate) shadow: PathBuf,
    /// `HEAD` before the transaction.
    pub(crate) base: String,
    /// The commit the transaction prepared,
    ///  not on any branch yet.
    pub(crate) prepared: String,
}

/// Methods building transaction state in a scene.
impl Scene {
    /// What:
    ///  A fresh scene whose transaction is owned by a live or dead owner.
    /// Why:
    ///   Every recovery case starts from a published directory.
    pub(crate) fn new(name: &str, live: bool) -> Scene {
        let root: PathBuf = fixture(name);
        let repo: PathBuf = repository(root.as_path(), "repo");
        let git_dir: PathBuf = repo.join(".git");
        // An empty commit leaves no index file; landings always have one.
        git(repo.as_path(), &["read-tree", "HEAD"]);
        let registry: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
        ensure_transaction_root(registry.as_path()).expect("registry");
        let directory: PathBuf = publish_transaction_directory(
            registry.as_path(),
            ID,
            owner_bytes(ID, "2026-10-06T00:00:00.000Z", live).as_slice(),
        )
        .expect("published");
        let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
        let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
        let prepared: String = git_text(
            repo.as_path(),
            &[
                "commit-tree",
                "-p",
                base.as_str(),
                "-m",
                "prepared",
                tree.as_str(),
            ],
        );
        let shadow: PathBuf = git_dir.join("cli-git").join("shadow").join(ID);
        return Scene {
            root,
            repo,
            git_dir,
            registry,
            directory,
            shadow,
            base,
            prepared,
        };
    }

    /// What:
    ///  The preparing record of this scene.
    /// Why:
    ///   Recovery reads every path from it.
    pub(crate) fn preparing(&self) -> PreparingRecord {
        let mut record: PreparingRecord = sample_preparing();
        record.transaction_id = String::from(ID);
        record.base = Base::Commit(self.base.clone());
        record.repository_root = text(self.repo.as_path());
        record.git_dir = text(self.git_dir.as_path());
        record.common_dir = text(self.git_dir.as_path());
        record.real_index_path = text(self.git_dir.join("index").as_path());
        record.object_directory = text(self.git_dir.join("objects").as_path());
        record.shadow_path = text(self.shadow.as_path());
        return record;
    }

    /// What:
    ///  Write `preparing.json` and create the shadow repository directory.
    /// Why:
    ///   A transaction past capture.
    pub(crate) fn capture(&self) {
        std::fs::write(
            self.directory.join(PREPARING_FILENAME),
            encode_preparing(&self.preparing()),
        )
        .expect("preparing");
        std::fs::create_dir_all(self.shadow.join("objects").join("info")).expect("shadow");
        // The shadow store reaches the real objects through alternates, as a real shadow does.
        std::fs::write(
            self.shadow.join("objects").join("info").join("alternates"),
            format!("{}\n", self.git_dir.join("objects").display()),
        )
        .expect("alternates");
    }

    /// What:
    ///  Create the real `index.lock` and record it as attempt 1.
    /// Why:
    ///   A transaction inside the landing critical section.
    pub(crate) fn lock_index(&self) -> LockIdentity {
        let lock: PathBuf = self.git_dir.join("index.lock");
        std::fs::write(&lock, b"").expect("index lock");
        let identity: LockIdentity = lock_identity_of(lock.as_path());
        std::fs::write(
            self.directory.join("index-lock-1.json"),
            encode_index_lock(&IndexLockRecord {
                attempt: 1,
                lock: identity.clone(),
            }),
        )
        .expect("index lock record");
        std::fs::write(
            self.git_dir.join("index~pid.lock"),
            format!("pid {}\n", std::process::id()),
        )
        .expect("pid file");
        return identity;
    }

    /// What:
    ///  Write landing attempt 1 for a commit (or a normalization) with both index snapshots.
    /// Why:
    ///   A transaction that reached the compare-and-swap.
    pub(crate) fn land(&self, operation: LandingOperation, added: Vec<AddedPath>) -> LandingRecord {
        let lock: LockIdentity = self.lock_index();
        let real_index: PathBuf = self.git_dir.join("index");
        std::fs::copy(&real_index, self.directory.join("pre-landing-1.index"))
            .expect("pre-landing");
        std::fs::write(self.directory.join("post-1.index"), b"post index").expect("post");
        let record: LandingRecord = LandingRecord {
            attempt: 1,
            operation,
            expected_old: Base::Commit(self.base.clone()),
            new_oid: if operation == LandingOperation::Commit {
                Some(self.prepared.clone())
            } else {
                None
            },
            landed_tree_oid: String::from("4b825dc642cb6eb9a060e54bf8d69288fbee4904"),
            pre_landing_index: identity(self.directory.join("pre-landing-1.index").as_path()),
            post_index: identity(self.directory.join("post-1.index").as_path()),
            lock,
            pack_name: None,
            added_paths: added,
            selected_worktree_paths: Vec::new(),
            pre_landing_digest: Some(
                crate::content_digest::digest_file(self.directory.join("pre-landing-1.index").as_path()).expect("pre-landing digest"),
            ),
            post_digest: Some(
                crate::content_digest::digest_file(self.directory.join("post-1.index").as_path()).expect("post digest"),
            ),
        };
        std::fs::write(
            self.directory.join("landing-1.json"),
            encode_landing(&record),
        )
        .expect("landing");
        return record;
    }

    /// What:
    ///  Move `main` to the prepared commit with the nonce reflog entry,
    ///  as the landing does.
    /// Why:
    ///   A transaction whose compare-and-swap succeeded.
    pub(crate) fn move_branch(&self) {
        let message: String = format!("commit (cli-git {ID}): prepared");
        git(
            self.repo.as_path(),
            &[
                "update-ref",
                "-m",
                message.as_str(),
                "refs/heads/main",
                self.prepared.as_str(),
                self.base.as_str(),
            ],
        );
    }

    /// What:
    ///  The recovery place of this scene.
    /// Why:
    ///   Recovery runs Git in the repository.
    pub(crate) fn place(&self) -> RecoveryPlace {
        return RecoveryPlace {
            git: git_context(),
            cwd: self.repo.clone(),
            common_dir: self.git_dir.clone(),
        };
    }

    /// What:
    ///  Remove the fixture.
    /// Why:
    ///   Every test cleans up after itself.
    pub(crate) fn finish(self) {
        remove(self.root.as_path());
    }
}

/// A path as UTF-8 text.
fn text(path: &Path) -> String {
    return path.to_str().expect("UTF-8 fixture path").to_string();
}

/// The recorded identity of an artifact.
fn identity(path: &Path) -> FileIdentity {
    let (device, inode, _) = file_identity_of(path).expect("identity");
    return FileIdentity { device, inode };
}

/// Action names are the incumbent's.
#[test]
fn actions_have_the_incumbent_names() {
    let pairs: [(RecoveryAction, &str); 9] = [
        (RecoveryAction::CommitNotCreated, "commit-not-created"),
        (
            RecoveryAction::NormalizationInstalled,
            "normalization-installed",
        ),
        (RecoveryAction::IndexInstalled, "index-installed"),
        (RecoveryAction::AlreadyInstalled, "already-installed"),
        (RecoveryAction::OwnerActive, "owner-active"),
        (RecoveryAction::Vanished, "vanished"),
        (RecoveryAction::StagingUnattributed, "staging-unattributed"),
        (RecoveryAction::StagingRetained, "staging-retained"),
        (RecoveryAction::RetiredRemoved, "retired-removed"),
    ];
    for (action, name) in pairs {
        assert_eq!(action_name(action), name);
    }
}

/// An owner that died before capture leaves only its directory and maybe a shadow.
#[test]
fn deaths_before_capture_remove_everything() {
    let scene: Scene = Scene::new("recovery-landing-precapture", false);
    std::fs::create_dir_all(scene.shadow.as_path()).expect("shadow");
    let action: RecoveryAction =
        recover_dead_transaction(&scene.place(), scene.directory.as_path(), ID, 1)
            .expect("recovered");
    assert_eq!(action, RecoveryAction::CommitNotCreated);
    assert!(!scene.directory.exists());
    assert!(!scene.shadow.exists());
    scene.finish();
}

/// An unreleased journal format stops recovery and keeps the directory.
#[test]
fn unreleased_journals_fail_closed() {
    let scene: Scene = Scene::new("recovery-landing-unreleased", false);
    std::fs::write(scene.directory.join(UNRELEASED_JOURNAL_FILENAME), b"{}").expect("journal");
    assert_eq!(
        recover_dead_transaction(&scene.place(), scene.directory.as_path(), ID, 1)
            .expect_err("unreleased")
            .0,
        format!(
            "Transaction directory uses an unreleased journal format; inspect it before removing it: {}",
            scene.directory.display()
        )
    );
    assert!(scene.directory.join(UNRELEASED_JOURNAL_FILENAME).exists());
    scene.finish();
}

/// Without a landing record the owned lock,
///  PID file,
///  keeps and shadow go.
#[test]
fn deaths_before_landing_discard_the_attempt() {
    let scene: Scene = Scene::new("recovery-landing-prelanding", false);
    scene.capture();
    scene.lock_index();
    let pack: PathBuf = scene.git_dir.join("objects").join("pack");
    std::fs::create_dir_all(&pack).expect("pack directory");
    std::fs::write(pack.join("pack-x.keep"), format!("cli-git {ID}\n")).expect("keep");
    let index_before: Vec<u8> = std::fs::read(scene.git_dir.join("index")).expect("index");
    let action: RecoveryAction = recover_dead_transaction(
        &scene.place(),
        scene.directory.as_path(),
        ID,
        i64::from(std::process::id()),
    )
    .expect("recovered");
    assert_eq!(action, RecoveryAction::CommitNotCreated);
    assert!(!scene.git_dir.join("index.lock").exists());
    assert!(!scene.git_dir.join("index~pid.lock").exists());
    assert!(!pack.join("pack-x.keep").exists());
    assert!(!scene.shadow.exists());
    assert!(!scene.directory.exists());
    assert_eq!(
        std::fs::read(scene.git_dir.join("index")).expect("index"),
        index_before
    );
    assert_eq!(
        git_text(scene.repo.as_path(), &["rev-parse", "HEAD"]),
        scene.base
    );
    scene.finish();
}

/// A landing that never moved the branch is discarded.
#[test]
fn unlanded_attempts_are_discarded() {
    let scene: Scene = Scene::new("recovery-landing-unlanded", false);
    scene.capture();
    scene.land(LandingOperation::Commit, Vec::new());
    let action: RecoveryAction = recover_dead_transaction(
        &scene.place(),
        scene.directory.as_path(),
        ID,
        i64::from(std::process::id()),
    )
    .expect("recovered");
    assert_eq!(action, RecoveryAction::CommitNotCreated);
    assert!(!scene.git_dir.join("index.lock").exists());
    assert!(!scene.directory.exists());
    assert_eq!(
        git_text(scene.repo.as_path(), &["rev-parse", "HEAD"]),
        scene.base
    );
    scene.finish();
}

/// A landed commit gets its index,
///  landed record,
///  added-path copies and cleanup.
#[test]
fn landed_commits_are_completed() {
    let scene: Scene = Scene::new("recovery-landing-landed", false);
    scene.capture();
    std::fs::write(scene.repo.join("added.txt"), b"x").expect("worktree copy");
    let original: String = git_text(scene.repo.as_path(), &["hash-object", "-w", "added.txt"]);
    std::fs::write(scene.root.join("intended"), b"x\n").expect("intended input");
    let intended_path: String = text(scene.root.join("intended").as_path());
    let intended: String = git_text(
        scene.repo.as_path(),
        &["hash-object", "-w", intended_path.as_str()],
    );
    let added: AddedPath = AddedPath {
        path: b"added.txt".to_vec(),
        git_mode: String::from("100644"),
        original_oid: original,
        intended_oid: intended,
    };
    std::fs::write(
        scene.directory.join(CAPTURED_FILENAME),
        encode_captured_record(&CapturedRecord {
            worktree_id: String::from("w"),
            sequence: 1,
            next_sequence_before_base: 1,
            worktree_paths: vec![String::from("added.txt")],
        }),
    )
    .expect("captured");
    scene.land(LandingOperation::Commit, vec![added]);
    scene.move_branch();
    std::fs::write(
        scene.directory.join(REF_UPDATED_FILENAME_FOR_TEST),
        encode_ref_updated(scene.prepared.as_str()),
    )
    .expect("marker");
    let action: RecoveryAction = recover_dead_transaction(
        &scene.place(),
        scene.directory.as_path(),
        ID,
        i64::from(std::process::id()),
    )
    .expect("recovered");
    assert_eq!(action, RecoveryAction::IndexInstalled);
    assert_eq!(
        std::fs::read(scene.git_dir.join("index")).expect("index"),
        b"post index"
    );
    assert!(!scene.git_dir.join("index.lock").exists());
    assert_eq!(
        std::fs::read(scene.repo.join("added.txt")).expect("completed copy"),
        b"x\n"
    );
    assert!(
        landed_record_directory(scene.git_dir.as_path())
            .join(format!("{}.json", scene.prepared))
            .exists()
    );
    assert!(!scene.shadow.exists());
    assert!(!scene.directory.exists());
    scene.finish();
}

/// The marker filename,
///  spelled out so a renamed constant fails this test.
const REF_UPDATED_FILENAME_FOR_TEST: &str = "ref-updated.json";

/// A landed commit whose index was already installed is recognized.
#[test]
fn installed_indexes_are_recognized() {
    let scene: Scene = Scene::new("recovery-landing-installed", false);
    scene.capture();
    scene.land(LandingOperation::Commit, Vec::new());
    scene.move_branch();
    std::fs::copy(
        scene.directory.join("post-1.index"),
        scene.git_dir.join("index"),
    )
    .expect("installed");
    let action: RecoveryAction = recover_dead_transaction(
        &scene.place(),
        scene.directory.as_path(),
        ID,
        i64::from(std::process::id()),
    )
    .expect("recovered");
    assert_eq!(action, RecoveryAction::AlreadyInstalled);
    assert!(!scene.git_dir.join("index.lock").exists());
    assert!(!scene.directory.exists());
    scene.finish();
}

/// A normalization is completed only while the target did not move.
#[test]
fn normalizations_need_an_unmoved_target() {
    let scene: Scene = Scene::new("recovery-landing-normalize", false);
    scene.capture();
    scene.land(LandingOperation::NormalizeOnly, Vec::new());
    git(
        scene.repo.as_path(),
        &["update-ref", "refs/heads/main", scene.prepared.as_str()],
    );
    assert_eq!(
        recover_dead_transaction(&scene.place(), scene.directory.as_path(), ID, 1)
            .expect_err("moved")
            .0,
        format!(
            "refs/heads/main moved after an interrupted normalization; recovery retained at {}",
            scene.directory.display()
        )
    );
    assert!(scene.directory.exists());
    git(
        scene.repo.as_path(),
        &["update-ref", "refs/heads/main", scene.base.as_str()],
    );
    assert_eq!(
        recover_dead_transaction(&scene.place(), scene.directory.as_path(), ID, 1)
            .expect("unmoved"),
        RecoveryAction::NormalizationInstalled
    );
    assert_eq!(
        std::fs::read(scene.git_dir.join("index")).expect("index"),
        b"post index"
    );
    assert!(!scene.directory.exists());
    scene.finish();
}

/// An unborn target compares as unborn.
#[test]
fn unborn_targets_compare_as_unborn() {
    let scene: Scene = Scene::new("recovery-landing-unborn", false);
    let mut preparing: PreparingRecord = scene.preparing();
    preparing.target_ref = String::from("refs/heads/none");
    assert!(target_unmoved(&scene.place(), &preparing, &Base::Unborn));
    assert!(!target_unmoved(
        &scene.place(),
        &preparing,
        &Base::Commit(scene.base.clone())
    ));
    preparing.target_ref = String::from("refs/heads/main");
    assert!(target_unmoved(
        &scene.place(),
        &preparing,
        &Base::Commit(scene.base.clone())
    ));
    scene.finish();
}
