//! Controls for conclusion cleanup against real files-backend and reftable repositories.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};

/// A real worktree Git directory,
///  a shadow and a transaction directory with copies.
struct Scene {
    /// Fixture root.
    root: PathBuf,
    /// Owning worktree.
    repo: PathBuf,
    /// Owning worktree Git directory.
    git_dir: PathBuf,
    /// Shadow repository after native Git.
    shadow: PathBuf,
    /// Transaction directory.
    transaction: PathBuf,
}

/// Build a files-backend scene with an empty copy directory.
fn scene(name: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let shadow_work: PathBuf = repository(root.as_path(), "shadow");
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir_all(transaction.join(CONCLUSION_COPY_DIRECTORY)).expect("copies");
    std::fs::write(
        transaction
            .join(CONCLUSION_COPY_DIRECTORY)
            .join(STORE_RECORD_FILENAME),
        b"{}",
    )
    .expect("store record");
    return Scene {
        git_dir: repo.join(".git"),
        repo,
        shadow: shadow_work.join(".git"),
        root,
        transaction,
    };
}

/// Run the cleanup for a scene.
fn clean(scene: &Scene, format: RefFormat) -> Result<(), RecoveryError> {
    return reproduce_conclusion_cleanup(
        &git_context(),
        scene.repo.as_path(),
        scene.git_dir.as_path(),
        scene.shadow.as_path(),
        scene.transaction.as_path(),
        format,
    );
}

/// The name lists are the incumbent's.
#[test]
fn names_match_the_incumbent() {
    assert_eq!(CONCLUSION_STATE_FILES.len(), 8);
    assert!(is_store_held(RefFormat::Reftable, "AUTO_MERGE"));
    assert!(is_store_held(RefFormat::Reftable, "CHERRY_PICK_HEAD"));
    assert!(is_store_held(RefFormat::Reftable, "REVERT_HEAD"));
    assert!(!is_store_held(RefFormat::Reftable, "MERGE_HEAD"));
    assert!(!is_store_held(RefFormat::Files, "AUTO_MERGE"));
}

/// Without copies nothing changes.
#[test]
fn absent_copies_change_nothing() {
    let scene: Scene = scene("conclusion-absent");
    std::fs::remove_dir_all(scene.transaction.join(CONCLUSION_COPY_DIRECTORY)).expect("no copies");
    std::fs::write(scene.git_dir.join("MERGE_MSG"), b"msg").expect("real entry");
    clean(&scene, RefFormat::Files).expect("cleanup");
    assert!(scene.git_dir.join("MERGE_MSG").exists());
    remove(scene.root.as_path());
}

/// An entry is removed only while copied,
///  gone from the shadow,
///  and unchanged in the worktree.
#[test]
fn entries_are_removed_only_while_unchanged() {
    let scene: Scene = scene("conclusion-files");
    let copies: PathBuf = scene.transaction.join(CONCLUSION_COPY_DIRECTORY);
    // Removed: copied, gone from the shadow, unchanged.
    std::fs::write(copies.join("MERGE_HEAD"), b"oid\n").expect("copy");
    std::fs::write(scene.git_dir.join("MERGE_HEAD"), b"oid\n").expect("real");
    // Kept: changed after preparation.
    std::fs::write(copies.join("MERGE_MSG"), b"old").expect("copy");
    std::fs::write(scene.git_dir.join("MERGE_MSG"), b"new").expect("real");
    // Kept: native Git kept it in the shadow.
    std::fs::write(copies.join("SQUASH_MSG"), b"s").expect("copy");
    std::fs::write(scene.shadow.join("SQUASH_MSG"), b"s").expect("shadow");
    std::fs::write(scene.git_dir.join("SQUASH_MSG"), b"s").expect("real");
    // Kept: never copied.
    std::fs::write(scene.git_dir.join("MERGE_MODE"), b"no-ff").expect("real");
    // Kept: ORIG_HEAD is never cleaned.
    std::fs::write(copies.join("ORIG_HEAD"), b"o").expect("copy");
    std::fs::write(scene.git_dir.join("ORIG_HEAD"), b"o").expect("real");
    // Store-held names are files under the files backend.
    std::fs::write(copies.join("CHERRY_PICK_HEAD"), b"c").expect("copy");
    std::fs::write(scene.git_dir.join("CHERRY_PICK_HEAD"), b"c").expect("real");
    // Copied but already gone from the worktree.
    std::fs::write(copies.join("REVERT_HEAD"), b"r").expect("copy");
    clean(&scene, RefFormat::Files).expect("cleanup");
    assert!(!scene.git_dir.join("MERGE_HEAD").exists());
    assert!(!scene.git_dir.join("CHERRY_PICK_HEAD").exists());
    assert_eq!(
        std::fs::read(scene.git_dir.join("MERGE_MSG")).expect("kept"),
        b"new"
    );
    assert!(scene.git_dir.join("SQUASH_MSG").exists());
    assert!(scene.git_dir.join("MERGE_MODE").exists());
    assert!(scene.git_dir.join("ORIG_HEAD").exists());
    assert!(!scene.git_dir.join("REVERT_HEAD").exists());
    remove(scene.root.as_path());
}

/// The shadow's `MERGE_RR` replaces the worktree's.
#[test]
fn merge_rr_is_copied_back() {
    let scene: Scene = scene("conclusion-merge-rr");
    std::fs::write(scene.git_dir.join("MERGE_RR"), b"old").expect("real");
    clean(&scene, RefFormat::Files).expect("cleanup without shadow MERGE_RR");
    assert_eq!(
        std::fs::read(scene.git_dir.join("MERGE_RR")).expect("kept"),
        b"old"
    );
    std::fs::write(scene.shadow.join("MERGE_RR"), b"resolved\0path").expect("shadow");
    clean(&scene, RefFormat::Files).expect("cleanup");
    assert_eq!(
        std::fs::read(scene.git_dir.join("MERGE_RR")).expect("copied"),
        b"resolved\0path"
    );
    let mut leftovers: usize = 0;
    for entry in std::fs::read_dir(&scene.git_dir).expect("list") {
        let name: String = entry
            .expect("entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        if name.starts_with("MERGE_RR.cli-git-") {
            leftovers += 1;
        }
    }
    assert_eq!(leftovers, 0);
    remove(scene.root.as_path());
}

/// The sequencer goes only when native Git removed it and the worktree copy is unchanged.
#[test]
fn sequencer_state_follows_native_git() {
    let scene: Scene = scene("conclusion-sequencer");
    let copy: PathBuf = scene
        .transaction
        .join(CONCLUSION_COPY_DIRECTORY)
        .join(SEQUENCER_DIRECTORY);
    let real: PathBuf = scene.git_dir.join(SEQUENCER_DIRECTORY);
    for directory in [&copy, &real] {
        std::fs::create_dir_all(directory.join("nested")).expect("sequencer");
        std::fs::write(directory.join("todo"), b"pick a").expect("todo");
        std::fs::write(directory.join("nested").join("opts"), b"o").expect("nested");
    }
    // Native Git kept the shadow sequencer: kept.
    std::fs::create_dir(scene.shadow.join(SEQUENCER_DIRECTORY)).expect("shadow sequencer");
    clean(&scene, RefFormat::Files).expect("cleanup");
    assert!(real.exists());
    std::fs::remove_dir(scene.shadow.join(SEQUENCER_DIRECTORY)).expect("native removed it");
    // The worktree sequencer changed: kept.
    std::fs::write(real.join("nested").join("opts"), b"changed").expect("changed");
    clean(&scene, RefFormat::Files).expect("cleanup");
    assert!(real.exists());
    std::fs::write(real.join("extra"), b"x").expect("extra");
    std::fs::write(real.join("nested").join("opts"), b"o").expect("restored");
    clean(&scene, RefFormat::Files).expect("cleanup");
    assert!(real.exists());
    std::fs::remove_file(real.join("extra")).expect("extra removed");
    clean(&scene, RefFormat::Files).expect("cleanup");
    assert!(!real.exists());
    // An empty copy never matches.
    std::fs::remove_dir_all(&copy).expect("remove copy");
    std::fs::create_dir(&copy).expect("empty copy");
    std::fs::create_dir(&real).expect("empty real");
    clean(&scene, RefFormat::Files).expect("cleanup");
    assert!(real.exists());
    assert!(!same_tree(copy.as_path(), real.as_path()).expect("empty trees"));
    assert!(
        !same_tree(scene.root.join("missing").as_path(), real.as_path()).expect("missing tree")
    );
    remove(scene.root.as_path());
}

/// A missing or malformed store record fails closed.
#[test]
fn store_records_must_be_json() {
    let scene: Scene = scene("conclusion-store-record");
    let record: PathBuf = scene
        .transaction
        .join(CONCLUSION_COPY_DIRECTORY)
        .join(STORE_RECORD_FILENAME);
    std::fs::write(&record, b"{").expect("malformed");
    assert_eq!(
        clean(&scene, RefFormat::Files).expect_err("malformed").0,
        format!("{} is not JSON.", record.display())
    );
    std::fs::write(&record, [0xff]).expect("binary");
    assert_eq!(
        clean(&scene, RefFormat::Files).expect_err("binary").0,
        format!("{} is not UTF-8.", record.display())
    );
    std::fs::remove_file(&record).expect("missing");
    assert!(
        clean(&scene, RefFormat::Files)
            .expect_err("missing")
            .0
            .starts_with("reading ")
    );
    remove(scene.root.as_path());
}

/// Reftable pseudorefs are deleted through Git while they still hold the copied value.
#[test]
fn reftable_pseudorefs_are_deleted_through_git() {
    let root: PathBuf = fixture("conclusion-reftable");
    let repo: PathBuf = root.join("repo");
    let shadow_work: PathBuf = root.join("shadow");
    for directory in [&repo, &shadow_work] {
        std::fs::create_dir(directory).expect("directory");
        git(
            directory.as_path(),
            &[
                "init",
                "--quiet",
                "--initial-branch=main",
                "--ref-format=reftable",
            ],
        );
        git(
            directory.as_path(),
            &["commit", "--quiet", "--allow-empty", "--message=initial"],
        );
    }
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(
        repo.as_path(),
        &["update-ref", "CHERRY_PICK_HEAD", head.as_str()],
    );
    git(
        repo.as_path(),
        &["update-ref", "REVERT_HEAD", head.as_str()],
    );
    git(repo.as_path(), &["update-ref", "AUTO_MERGE", head.as_str()]);
    // A second commit makes the shadow's value differ from the worktree's, which two identical
    // initial commits made in the same second would not.
    git(
        shadow_work.as_path(),
        &["commit", "--quiet", "--allow-empty", "--message=shadow"],
    );
    let shadow_head: String = git_text(shadow_work.as_path(), &["rev-parse", "HEAD"]);
    assert_ne!(shadow_head, head);
    git(
        shadow_work.as_path(),
        &["update-ref", "AUTO_MERGE", shadow_head.as_str()],
    );
    let transaction: PathBuf = root.join("transaction");
    std::fs::create_dir_all(transaction.join(CONCLUSION_COPY_DIRECTORY)).expect("copies");
    // CHERRY_PICK_HEAD: copied and unchanged, gone from the shadow: deleted.
    // REVERT_HEAD: copied value differs from the current one: kept.
    // AUTO_MERGE: still in the shadow: kept.
    std::fs::write(
        transaction.join(CONCLUSION_COPY_DIRECTORY).join(STORE_RECORD_FILENAME),
        format!(
            "{{\"CHERRY_PICK_HEAD\":\"{head}\",\"REVERT_HEAD\":\"{shadow_head}\",\"AUTO_MERGE\":\"{head}\",\"MERGE_HEAD\":1}}"
        ),
    )
    .expect("store record");
    let scene: Scene = Scene {
        git_dir: repo.join(".git"),
        repo: repo.clone(),
        shadow: shadow_work.join(".git"),
        root: root.clone(),
        transaction,
    };
    clean(&scene, RefFormat::Reftable).expect("cleanup");
    let verify: std::process::Output = crate::test_support::git_output(
        repo.as_path(),
        &["rev-parse", "--verify", "--quiet", "CHERRY_PICK_HEAD"],
    );
    assert!(!verify.status.success());
    assert_eq!(
        git_text(repo.as_path(), &["rev-parse", "--verify", "REVERT_HEAD"]),
        head
    );
    assert_eq!(
        git_text(repo.as_path(), &["rev-parse", "--verify", "AUTO_MERGE"]),
        head
    );
    // A value that is not text is no copy.
    std::fs::write(
        scene
            .transaction
            .join(CONCLUSION_COPY_DIRECTORY)
            .join(STORE_RECORD_FILENAME),
        "{\"REVERT_HEAD\":7,\"AUTO_MERGE\":null}",
    )
    .expect("non-text values");
    clean(&scene, RefFormat::Reftable).expect("cleanup");
    assert_eq!(
        git_text(repo.as_path(), &["rev-parse", "--verify", "REVERT_HEAD"]),
        head
    );
    // A non-object record copies nothing.
    std::fs::write(
        scene
            .transaction
            .join(CONCLUSION_COPY_DIRECTORY)
            .join(STORE_RECORD_FILENAME),
        b"[]",
    )
    .expect("array record");
    clean(&scene, RefFormat::Reftable).expect("cleanup");
    assert_eq!(
        git_text(repo.as_path(), &["rev-parse", "--verify", "REVERT_HEAD"]),
        head
    );
    assert_eq!(
        resolve_pseudoref(
            &git_context(),
            GitRequest::new(
                repo.as_path(),
                &["rev-parse", "--verify", "--quiet", "NONE_HEAD"]
            )
        ),
        None
    );
    remove(root.as_path());
}
