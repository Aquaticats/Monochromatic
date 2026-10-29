//! Controls for the landed-or-not decision and index completion against real repositories.

use super::*;
use crate::content_digest::digest_file;
use crate::recovery_files::file_identity_of;
use crate::recovery_files::tests::lock_identity_of;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_journal::{Base, LandingOperation, LockIdentity};
use crate::transaction_journal_encode::encode_ref_updated;
use crate::transaction_journal_encode::tests::{sample_lock, sample_preparing};

/// The transaction ID the fixtures use.
const ID: &str = "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10";

/// The recorded identity of an artifact.
fn identity(path: &Path) -> FileIdentity {
    let (device, inode, _) = file_identity_of(path).expect("identity");
    return FileIdentity { device, inode };
}

/// A preparing record for a fixture repository.
fn preparing_for(repo: &Path) -> PreparingRecord {
    let mut record: PreparingRecord = sample_preparing();
    record.transaction_id = String::from(ID);
    record.target_ref = String::from("refs/heads/main");
    record.real_index_path = repo
        .join(".git")
        .join("index")
        .to_str()
        .expect("UTF-8")
        .to_string();
    return record;
}

/// A commit landing record naming `new_oid` with artifacts in `directory`.
fn landing_for(directory: &Path, new_oid: &str, lock: LockIdentity) -> LandingRecord {
    return LandingRecord {
        attempt: 1,
        operation: LandingOperation::Commit,
        expected_old: Base::Unborn,
        new_oid: Some(String::from(new_oid)),
        landed_tree_oid: String::from("4b825dc642cb6eb9a060e54bf8d69288fbee4904"),
        pre_landing_index: identity(directory.join("pre-landing-1.index").as_path()),
        post_index: identity(directory.join("post-1.index").as_path()),
        lock,
        pack_name: None,
        added_paths: Vec::new(),
        selected_worktree_paths: Vec::new(),
        pre_landing_digest: None,
        post_digest: None,
    };
}

/// A fixture repository with a transaction directory holding both index snapshots.
struct Scene {
    /// Fixture root.
    root: PathBuf,
    /// Repository.
    repo: PathBuf,
    /// Transaction directory.
    directory: PathBuf,
    /// Commit before the landing.
    first: String,
    /// Commit the landing created.
    second: String,
}

/// Build a scene;
///  the landing commit exists but `main` has not moved to it.
fn scene(name: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let first: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let second: String = git_text(
        repo.as_path(),
        &[
            "commit-tree",
            "-p",
            first.as_str(),
            "-m",
            "landed",
            tree.as_str(),
        ],
    );
    let directory: PathBuf = root.join("transaction");
    std::fs::create_dir(&directory).expect("transaction");
    std::fs::write(directory.join("pre-landing-1.index"), b"pre").expect("pre-landing");
    std::fs::write(directory.join("post-1.index"), b"post").expect("post");
    return Scene {
        root,
        repo,
        directory,
        first,
        second,
    };
}

/// A landing whose marker names its commit landed;
///  a marker naming another fails closed.
#[test]
fn markers_prove_a_landing() {
    let scene: Scene = scene("completion-marker");
    let preparing: PreparingRecord = preparing_for(scene.repo.as_path());
    let landing: LandingRecord = landing_for(
        scene.directory.as_path(),
        scene.second.as_str(),
        sample_lock(),
    );
    let context: GitContext = git_context();
    std::fs::write(
        scene.directory.join(REF_UPDATED_FILENAME),
        encode_ref_updated(scene.second.as_str()),
    )
    .expect("marker");
    assert!(
        commit_landed(
            &context,
            scene.repo.as_path(),
            scene.directory.as_path(),
            &preparing,
            &landing
        )
        .expect("landed")
    );
    std::fs::write(
        scene.directory.join(REF_UPDATED_FILENAME),
        encode_ref_updated(scene.first.as_str()),
    )
    .expect("other marker");
    assert_eq!(
        commit_landed(
            &context,
            scene.repo.as_path(),
            scene.directory.as_path(),
            &preparing,
            &landing
        )
        .expect_err("contradiction")
        .0,
        format!(
            "Landed marker names another commit than the landing record: {}",
            scene.directory.display()
        )
    );
    let mut without: LandingRecord = landing.clone();
    without.new_oid = None;
    assert!(
        commit_landed(
            &context,
            scene.repo.as_path(),
            scene.directory.as_path(),
            &preparing,
            &without
        )
        .is_err()
    );
    remove(scene.root.as_path());
}

/// Without a marker,
///  the reflog nonce decides,
///  and a reachable commit without one fails closed.
#[test]
fn reflog_nonces_prove_a_landing() {
    let scene: Scene = scene("completion-reflog");
    let preparing: PreparingRecord = preparing_for(scene.repo.as_path());
    let landing: LandingRecord = landing_for(
        scene.directory.as_path(),
        scene.second.as_str(),
        sample_lock(),
    );
    let context: GitContext = git_context();
    let directory: &Path = scene.directory.as_path();
    // Never moved, not reachable: not landed.
    assert!(
        !commit_landed(
            &context,
            scene.repo.as_path(),
            directory,
            &preparing,
            &landing
        )
        .expect("not landed")
    );
    // Moved without the nonce: contradictory.
    git(
        scene.repo.as_path(),
        &["update-ref", "refs/heads/main", scene.second.as_str()],
    );
    let reachable: RecoveryError = commit_landed(
        &context,
        scene.repo.as_path(),
        directory,
        &preparing,
        &landing,
    )
    .expect_err("no nonce");
    assert_eq!(
        reachable.0,
        format!(
            "refs/heads/main contains the transaction commit {} but its reflog lacks the nonce entry; recovery retained at {}",
            scene.second,
            directory.display()
        )
    );
    // Moved back, then forward with the nonce: landed. Every update changes the value, so each
    // writes its own reflog entry.
    git(
        scene.repo.as_path(),
        &["update-ref", "refs/heads/main", scene.first.as_str()],
    );
    let action: String = format!("{}: landed", nonce_subject_prefix(ID));
    git(
        scene.repo.as_path(),
        &[
            "update-ref",
            "-m",
            action.as_str(),
            "refs/heads/main",
            scene.second.as_str(),
        ],
    );
    assert!(
        commit_landed(
            &context,
            scene.repo.as_path(),
            directory,
            &preparing,
            &landing
        )
        .expect("landed")
    );
    // The nonce names another commit than the record.
    let mut other: LandingRecord = landing.clone();
    other.new_oid = Some(scene.first.clone());
    assert_eq!(
        commit_landed(
            &context,
            scene.repo.as_path(),
            directory,
            &preparing,
            &other
        )
        .expect_err("other commit")
        .0,
        format!(
            "refs/heads/main reflog nonce names {}, not the recorded {}; recovery retained at {}",
            scene.second,
            scene.first,
            directory.display()
        )
    );
    // Two commits under one nonce.
    git(
        scene.repo.as_path(),
        &[
            "update-ref",
            "-m",
            action.as_str(),
            "refs/heads/main",
            scene.first.as_str(),
        ],
    );
    assert_eq!(
        commit_landed(
            &context,
            scene.repo.as_path(),
            directory,
            &preparing,
            &landing
        )
        .expect_err("several")
        .0,
        format!(
            "refs/heads/main reflog names several commits for transaction {ID}; recovery retained at {}",
            directory.display()
        )
    );
    remove(scene.root.as_path());
}

/// An unknown target ref is an unreadable reflog.
#[test]
fn unreadable_reflogs_fail_closed() {
    let scene: Scene = scene("completion-unreadable");
    let mut preparing: PreparingRecord = preparing_for(scene.repo.as_path());
    preparing.target_ref = String::from("refs/heads/none");
    let landing: LandingRecord = landing_for(
        scene.directory.as_path(),
        scene.second.as_str(),
        sample_lock(),
    );
    assert!(
        commit_landed(
            &git_context(),
            scene.repo.as_path(),
            scene.directory.as_path(),
            &preparing,
            &landing
        )
        .expect_err("unreadable")
        .0
        .starts_with("refs/heads/none reflog is unreadable")
    );
    remove(scene.root.as_path());
}

/// The post-index is installed through the owned lock while the real index is the pre-landing one.
#[test]
fn indexes_install_from_the_pre_landing_state() {
    let scene: Scene = scene("completion-install");
    let preparing: PreparingRecord = preparing_for(scene.repo.as_path());
    let real_index: PathBuf = PathBuf::from(preparing.real_index_path.clone());
    std::fs::write(&real_index, b"pre").expect("pre-landing real index");
    let lock: PathBuf = PathBuf::from(format!("{}.lock", preparing.real_index_path));
    std::fs::write(&lock, b"").expect("owned lock");
    let landing: LandingRecord = landing_for(
        scene.directory.as_path(),
        scene.second.as_str(),
        lock_identity_of(lock.as_path()),
    );
    assert_eq!(
        complete_index(scene.directory.as_path(), &preparing, &landing).expect("install"),
        IndexCompletion::Installed
    );
    assert_eq!(std::fs::read(&real_index).expect("installed"), b"post");
    assert!(!lock.exists());
    assert!(
        scene
            .directory
            .join("pre-landing-1.index.recovery")
            .exists()
    );
    assert!(scene.directory.join("post-1.index.recovery").exists());
    remove(scene.root.as_path());
}

/// A snapshot that differs from the digest its landing record holds is never installed as the real index,
///  whichever way it differs, and the real index, the owned lock and the snapshots stay as they were.
///  A record without digests installs as before, and a landing whose install already happened needs neither snapshot.
#[test]
fn a_torn_snapshot_is_never_installed() {
    let scene: Scene = scene("completion-torn");
    let preparing: PreparingRecord = preparing_for(scene.repo.as_path());
    let real_index: PathBuf = PathBuf::from(preparing.real_index_path.clone());
    std::fs::write(&real_index, b"pre").expect("pre-landing real index");
    let lock: PathBuf = PathBuf::from(format!("{}.lock", preparing.real_index_path));
    std::fs::write(&lock, b"").expect("owned lock");
    let mut landing: LandingRecord = landing_for(
        scene.directory.as_path(),
        scene.second.as_str(),
        lock_identity_of(lock.as_path()),
    );
    landing.pre_landing_digest = Some(digest_file(scene.directory.join("pre-landing-1.index").as_path()).expect("pre digest"));
    landing.post_digest = Some(digest_file(scene.directory.join("post-1.index").as_path()).expect("post digest"));
    let damages: [(&str, &[u8]); 3] = [("emptied", b""), ("halved", b"po"), ("zero-filled", &[0; 4])];
    for file in ["post-1.index", "pre-landing-1.index"] {
        let intact: Vec<u8> = std::fs::read(scene.directory.join(file)).expect("intact snapshot");
        for (name, bytes) in damages {
            // A damaged copy is a new file under the recorded name, as a torn write leaves it.
            std::fs::write(scene.directory.join(file), bytes).expect("damage");
            let error: String = complete_index(scene.directory.as_path(), &preparing, &landing)
                .expect_err(format!("{file} {name}").as_str())
                .0;
            assert!(
                error.contains(&scene.directory.join(file).display().to_string()) && error.contains("landing-1.json"),
                "{file} {name}: {error}"
            );
            assert_eq!(std::fs::read(&real_index).expect("real index"), b"pre", "{file} {name}");
            assert!(lock.exists(), "{file} {name}: the owned lock is kept");
        }
        std::fs::write(scene.directory.join(file), &intact).expect("restore");
    }
    // The install marker means the install happened: a torn snapshot no longer matters.
    std::fs::write(scene.directory.join("post-1.index"), b"").expect("torn after install");
    std::fs::write(scene.directory.join(INDEX_INSTALLED_FILENAME), b"").expect("marker");
    assert_eq!(
        complete_index(scene.directory.as_path(), &preparing, &landing).expect("marker"),
        IndexCompletion::AlreadyInstalled
    );
    remove(scene.root.as_path());
}

/// An installed post-index is recognized and the owned lock released;
///  a changed index fails closed.
#[test]
fn installed_or_changed_indexes_are_recognized() {
    let scene: Scene = scene("completion-installed");
    let preparing: PreparingRecord = preparing_for(scene.repo.as_path());
    let real_index: PathBuf = PathBuf::from(preparing.real_index_path.clone());
    let lock: PathBuf = PathBuf::from(format!("{}.lock", preparing.real_index_path));
    std::fs::write(&lock, b"").expect("owned lock");
    let landing: LandingRecord = landing_for(
        scene.directory.as_path(),
        scene.second.as_str(),
        lock_identity_of(lock.as_path()),
    );
    std::fs::write(&real_index, b"post").expect("already installed");
    assert_eq!(
        complete_index(scene.directory.as_path(), &preparing, &landing).expect("recognized"),
        IndexCompletion::AlreadyInstalled
    );
    assert!(!lock.exists());
    std::fs::write(&real_index, b"someone else").expect("changed index");
    std::fs::write(scene.directory.join(INDEX_INSTALLED_FILENAME), b"").expect("marker");
    assert_eq!(
        complete_index(scene.directory.as_path(), &preparing, &landing).expect("marker"),
        IndexCompletion::AlreadyInstalled
    );
    std::fs::remove_file(scene.directory.join(INDEX_INSTALLED_FILENAME)).expect("remove marker");
    assert_eq!(
        complete_index(scene.directory.as_path(), &preparing, &landing)
            .expect_err("changed")
            .0,
        format!(
            "Real index no longer matches the pre-landing snapshot; recovery retained at {}",
            scene.directory.display()
        )
    );
    // A replaced artifact is not the recorded one.
    std::fs::remove_file(scene.directory.join("post-1.index")).expect("remove post");
    std::fs::write(scene.directory.join("post-1.index"), b"post").expect("replaced post");
    assert!(complete_index(scene.directory.as_path(), &preparing, &landing).is_err());
    remove(scene.root.as_path());
}
