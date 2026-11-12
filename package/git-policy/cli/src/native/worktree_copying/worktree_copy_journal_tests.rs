//! Journal checks:
//! the exact encoding,
//! strict validation,
//! private storage and removal on disk.

use super::*;
use crate::path_bytes_in_records::hex_of_path;
use crate::worktree_copy_install_entry::InstalledPath;
use crate::worktree_copy_journal_encoding_and_validation::*;
use crate::path_bytes_in_records::hex_of_bytes;
use crate::worktree_copy_journal_removal::*;
use crate::private_storage::{take_directory_syncs, take_file_syncs};
use crate::test_support::{fixture, remove};
use std::os::unix::fs::PermissionsExt;

/// A valid staged journal whose paths live under one fixture root.
fn sample(root: &Path) -> Journal {
    let destination: PathBuf = root.join("topic");
    let stage: PathBuf = root.join(".cli-git-worktree-copy-abc");
    return Journal {
        created_entries: vec![InstalledPath {
            device: 7,
            inode: 42,
            relative_path: b"cache".to_vec(),
            selected: true,
        }],
        intended_entries: vec![b"cache".to_vec()],
        destination_root: destination,
        phase: Phase::Installing,
        selected_roots: vec![b"cache".to_vec()],
        source_root: root.join("source"),
        stage_container: stage.clone(),
        stage_root: stage.join("payload"),
    };
}

/// The encoding is the incumbent's `JSON.stringify` of the same object,
/// with each root's bytes in hexadecimal after its text and a final line feed.
#[test]
fn encoding_matches_the_incumbent_exactly() {
    let journal: Journal = sample(Path::new("/w"));
    let expected: String = concat!(
        "{\"createdEntries\":[{\"device\":\"7\",\"inode\":\"42\",\"relativePath\":\"cache\",\"selected\":true}],",
        "\"intendedEntries\":[\"cache\"],",
        "\"destinationRoot\":\"/w/topic\",",
        "\"destinationRootHex\":\"2f772f746f706963\",",
        "\"phase\":\"installing\",",
        "\"selectedRoots\":[\"cache\"],",
        "\"sourceRoot\":\"/w/source\",",
        "\"sourceRootHex\":\"2f772f736f75726365\",",
        "\"stageContainer\":\"/w/.cli-git-worktree-copy-abc\",",
        "\"stageContainerHex\":\"2f772f2e636c692d6769742d776f726b747265652d636f70792d616263\",",
        "\"stageRoot\":\"/w/.cli-git-worktree-copy-abc/payload\",",
        "\"stageRootHex\":\"2f772f2e636c692d6769742d776f726b747265652d636f70792d6162632f7061796c6f6164\",",
        "\"version\":1}\n"
    )
    .to_string();
    assert_eq!(encode_journal(&journal), expected);
}

/// Encoding then parsing returns the same journal.
#[test]
fn encoded_journal_parses_back_to_itself() {
    let root: PathBuf = fixture("journal-round-trip");
    let journal: Journal = sample(root.as_path());
    let journal_root: PathBuf = root.join("journals");
    let path: PathBuf = journal_root.join("id.json");
    let parsed: Journal = parse_journal(
        encode_journal(&journal).as_str(),
        path.as_path(),
        journal_root.as_path(),
    )
    .expect("valid");
    assert_eq!(parsed, journal);
    remove(root.as_path());
}

/// The same record with every bytes field removed,
/// as the release installed before them wrote it.
fn strip_bytes_fields(text: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(text).expect("a journal");
    let object: serde_json::Map<String, serde_json::Value> = value.as_object().expect("an object").clone();
    let mut earlier: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for (key, field) in object {
        if !key.ends_with("Hex") {
            earlier.insert(key, field);
        }
    }
    return format!("{}\n", serde_json::Value::Object(earlier));
}

/// Every rule of the incumbent's validation refuses the record with its own wording.
#[test]
fn validation_refuses_each_malformed_shape() {
    let root: PathBuf = fixture("journal-validation");
    let journal_root: PathBuf = root.join("journals");
    let path: PathBuf = journal_root.join("id.json");
    let base: String = encode_journal(&sample(root.as_path()));
    let corrupt = format!(
        "monochromatic-git-policy-cli: worktree-copy journal is corrupt: {}.",
        quote(path.to_string_lossy().as_ref())
    );
    let cases: Vec<(String, String)> = vec![
        (
            base.replace("\"version\":1", "\"version\":2"),
            corrupt.clone(),
        ),
        (
            base.replace("\"phase\":\"installing\"", "\"phase\":\"other\""),
            corrupt.clone(),
        ),
        (
            base.replace("\"device\":\"7\"", "\"device\":\"07\""),
            corrupt.clone(),
        ),
        (
            base.replace("\"selected\":true", "\"selected\":\"yes\""),
            corrupt.clone(),
        ),
        (
            base.replace("\"intendedEntries\":[\"cache\"]", "\"intendedEntries\":[1]"),
            corrupt.clone(),
        ),
        (
            // The bytes win over the text, so damage is a bytes field that is not the bytes of a path.
            base.replace(
                &format!(
                    "\"sourceRootHex\":{}",
                    quote(hex_of_path(root.join("source").as_path()).as_str())
                ),
                "\"sourceRootHex\":\"2f6g\"",
            ),
            corrupt.clone(),
        ),
        (
            // A record of the earlier shape, with no bytes fields, holds the text as the path.
            strip_bytes_fields(base.as_str()).replace(
                &format!(
                    "\"sourceRoot\":{}",
                    quote(root.join("source").to_string_lossy().as_ref())
                ),
                "\"sourceRoot\":\"relative/source\"",
            ),
            String::from(
                "monochromatic-git-policy-cli: worktree-copy journal has unsafe sourceRoot: \"relative/source\".",
            ),
        ),
        (
            base.replace(
                &format!(
                    "\"stageContainerHex\":{}",
                    quote(
                        hex_of_path(root.join(".cli-git-worktree-copy-abc").as_path()).as_str()
                    )
                ),
                &format!(
                    "\"stageContainerHex\":{}",
                    quote(
                        hex_of_path(root.join("elsewhere/.cli-git-worktree-copy-abc").as_path())
                            .as_str()
                    )
                ),
            ),
            format!(
                "monochromatic-git-policy-cli: worktree-copy journal has unsafe private stage relation: {}.",
                quote(path.to_string_lossy().as_ref())
            ),
        ),
        (
            base.replace(
                "\"intendedEntries\":[\"cache\"]",
                "\"intendedEntries\":[\"../escape\"]",
            ),
            String::from("monochromatic-git-policy-cli: Git returned unsafe ignored path \"../escape\"."),
        ),
        (
            base.replace(
                "\"selectedRoots\":[\"cache\"]",
                "\"selectedRoots\":[\"cache\",\"cache\"]",
            ),
            format!(
                "monochromatic-git-policy-cli: worktree-copy journal contains duplicate repository paths: {}.",
                quote(path.to_string_lossy().as_ref())
            ),
        ),
    ];
    for (text, expected) in cases {
        let failure: WorktreeCopyError =
            parse_journal(text.as_str(), path.as_path(), journal_root.as_path())
                .expect_err("refused");
        assert_eq!(failure.0, expected, "{text}");
    }
    let escaped: PathBuf = root.join("elsewhere/id.json");
    let failure: WorktreeCopyError =
        parse_journal(base.as_str(), escaped.as_path(), journal_root.as_path())
            .expect_err("escaped");
    assert_eq!(
        failure.0,
        format!(
            "monochromatic-git-policy-cli: worktree-copy journal escaped private root: {}.",
            quote(escaped.to_string_lossy().as_ref())
        )
    );
    let not_json: WorktreeCopyError =
        parse_journal("{", path.as_path(), journal_root.as_path()).expect_err("not json");
    // The JSON parser's own text differs from the incumbent's; the wording before the cause is the contract.
    assert!(
        not_json.0.starts_with(&format!(
            "monochromatic-git-policy-cli: could not read worktree-copy journal {}. Cause: ",
            quote(path.to_string_lossy().as_ref())
        )),
        "{}",
        not_json.0
    );
    remove(root.as_path());
}

/// Only an absolute path with no empty,
/// `.` or `..` component is canonical.
#[test]
fn only_canonical_absolute_paths_are_accepted() {
    for accepted in ["/", "/a", "/a/b", "/a/.hidden", "/a/b.c"] {
        assert!(is_canonical_absolute(Path::new(accepted)), "{accepted}");
    }
    for refused in [
        "", "a", "a/b", "//", "/a/", "/a//b", "/a/./b", "/./a", "/a/.", "/a/..", "/../a", "/a/b/..",
    ] {
        assert!(!is_canonical_absolute(Path::new(refused)), "{refused}");
    }
}

/// A device or inode is a decimal without a leading zero,
/// and `0` itself is one.
#[test]
fn identities_are_canonical_decimals() {
    let text = |value: &str| return serde_json::Value::String(String::from(value));
    assert_eq!(decimal_identity(&text("0")), Some(0));
    assert_eq!(decimal_identity(&text("7")), Some(7));
    assert_eq!(decimal_identity(&text("10")), Some(10));
    assert_eq!(decimal_identity(&text("18446744073709551615")), Some(u64::MAX));
    for refused in ["", "00", "01", "-1", "+1", "1.0", "a", "1 ", "18446744073709551616"] {
        assert_eq!(decimal_identity(&text(refused)), None, "{refused:?}");
    }
    assert_eq!(decimal_identity(&serde_json::Value::from(7_u64)), None);
}

/// Each relation between the stage and the destination refuses on its own,
///  and a repeated path refuses in whichever list it repeats.
#[test]
fn each_stage_relation_and_each_repeated_list_is_refused_alone() {
    let root: PathBuf = fixture("journal-relations");
    let journal_root: PathBuf = root.join("journals");
    let path: PathBuf = journal_root.join("id.json");
    let refuse = |journal: &Journal, expected: String| {
        let failure: WorktreeCopyError = parse_journal(
            encode_journal(journal).as_str(),
            path.as_path(),
            journal_root.as_path(),
        )
        .expect_err("refused");
        assert_eq!(failure.0, expected);
    };
    let relation: String = format!(
        "monochromatic-git-policy-cli: worktree-copy journal has unsafe private stage relation: {}.",
        quote(path.to_string_lossy().as_ref())
    );
    let duplicates: String = format!(
        "monochromatic-git-policy-cli: worktree-copy journal contains duplicate repository paths: {}.",
        quote(path.to_string_lossy().as_ref())
    );
    let place = |journal: &mut Journal, container: PathBuf| {
        journal.stage_container = container.clone();
        journal.stage_root = container.join("payload");
    };
    let mut unprefixed: Journal = sample(root.as_path());
    place(&mut unprefixed, root.join("plain-stage"));
    refuse(&unprefixed, relation.clone());
    let mut elsewhere: Journal = sample(root.as_path());
    place(&mut elsewhere, root.join("elsewhere/.cli-git-worktree-copy-abc"));
    refuse(&elsewhere, relation.clone());
    let mut misplaced_payload: Journal = sample(root.as_path());
    misplaced_payload.stage_root = root.join(".cli-git-worktree-copy-abc/other");
    refuse(&misplaced_payload, relation);
    let mut repeated_intent: Journal = sample(root.as_path());
    repeated_intent.intended_entries.push(b"cache".to_vec());
    refuse(&repeated_intent, duplicates.clone());
    let mut repeated_creation: Journal = sample(root.as_path());
    let first: InstalledPath = repeated_creation.created_entries[0].clone();
    repeated_creation.created_entries.push(first);
    refuse(&repeated_creation, duplicates);
    remove(root.as_path());
}

/// The journal root and its parent are created private,
/// and a journal written there reads back exactly.
#[cfg(unix)]
#[test]
fn journal_is_written_privately_and_read_back() {
    let root: PathBuf = fixture("journal-disk");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let journal_root: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    assert_eq!(
        journal_root,
        common.join("monochromatic-git-policy-cli-worktree-copy").join("v1")
    );
    assert_eq!(
        std::fs::metadata(journal_root.as_path())
            .expect("root")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    let journal: Journal = sample(root.as_path());
    let path: PathBuf = journal_root.join("id.json");
    write_journal(path.as_path(), &journal).expect("write");
    assert_eq!(
        std::fs::metadata(path.as_path())
            .expect("file")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let pending: Vec<PendingJournal> = read_pending_journals(common.as_path()).expect("pending");
    assert_eq!(
        pending,
        vec![PendingJournal {
            path: path.clone(),
            journal
        }]
    );
    remove(root.as_path());
}

/// Without a journal root there is nothing pending.
#[test]
fn absent_journal_root_has_no_pending_journals() {
    let root: PathBuf = fixture("journal-absent");
    assert!(
        read_pending_journals(root.as_path())
            .expect("none")
            .is_empty()
    );
    remove(root.as_path());
}

/// A journal whose file is group-writable is unsafe and fails closed without being deleted.
#[cfg(unix)]
#[test]
fn unsafe_journal_mode_fails_closed_and_is_kept() {
    let root: PathBuf = fixture("journal-unsafe-mode");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let journal_root: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let path: PathBuf = journal_root.join("id.json");
    std::fs::write(path.as_path(), encode_journal(&sample(root.as_path()))).expect("journal");
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o620)).expect("mode");
    let failure: WorktreeCopyError = read_pending_journals(common.as_path()).expect_err("unsafe");
    assert_eq!(
        failure.0,
        format!(
            "monochromatic-git-policy-cli: worktree-copy journal file is unsafe: {}.",
            quote(path.to_string_lossy().as_ref())
        )
    );
    assert!(path.exists());
    remove(root.as_path());
}

/// A journal file with a second hard link is unsafe,
/// because its content could be changed elsewhere.
#[cfg(unix)]
#[test]
fn journal_with_extra_hard_link_is_unsafe() {
    let root: PathBuf = fixture("journal-link-count");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let journal_root: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let path: PathBuf = journal_root.join("id.json");
    std::fs::write(path.as_path(), encode_journal(&sample(root.as_path()))).expect("journal");
    // The mode is private, so only the second link makes the file unsafe.
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o600)).expect("mode");
    std::fs::hard_link(path.as_path(), root.join("alias")).expect("alias");
    let failure: WorktreeCopyError = read_pending_journals(common.as_path()).expect_err("linked");
    assert!(
        failure
            .0
            .starts_with("monochromatic-git-policy-cli: worktree-copy journal file is unsafe: ")
    );
    remove(root.as_path());
}

/// A journal that cannot be written fails with the persist wording and leaves no temporary file.
#[test]
fn unwritable_journal_leaves_no_temporary() {
    let root: PathBuf = fixture("journal-persist");
    let path: PathBuf = root.join("absent-directory").join("id.json");
    let failure: WorktreeCopyError =
        write_journal(path.as_path(), &sample(root.as_path())).expect_err("persist");
    assert_eq!(
        failure.0,
        format!(
            "monochromatic-git-policy-cli: could not persist worktree-copy journal {}. Cause: No such file or directory (os error 2)",
            quote(path.to_string_lossy().as_ref())
        )
    );
    remove(root.as_path());
}

/// Removal writes the complete phase,
/// deletes the stage with its read-only subdirectories,
/// and deletes the journal.
#[cfg(unix)]
#[test]
fn removal_deletes_stage_and_journal() {
    let root: PathBuf = fixture("journal-remove");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let journal_root: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let journal: Journal = sample(root.as_path());
    let stage: PathBuf = journal.stage_container.clone();
    std::fs::create_dir_all(stage.join("payload/locked")).expect("stage");
    std::fs::write(stage.join("payload/locked/file"), b"x").expect("file");
    std::fs::set_permissions(
        stage.join("payload/locked"),
        std::fs::Permissions::from_mode(0o500),
    )
    .expect("read-only");
    let path: PathBuf = journal_root.join("id.json");
    write_journal(path.as_path(), &journal).expect("write");
    let pending: PendingJournal = PendingJournal {
        path: path.clone(),
        journal,
    };
    remove_pending_journal(&pending).expect("removed");
    assert!(!stage.exists());
    assert!(!path.exists());
    remove(root.as_path());
}

/// The complete phase reaches the disk before the stage is touched,
///  so a removal that stops part way resumes as cleanup.
#[cfg(unix)]
#[test]
fn the_complete_phase_is_written_before_the_stage_is_removed() {
    let root: PathBuf = fixture("journal-remove-order");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let journal_root: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let file: PathBuf = root.join("not-a-directory");
    std::fs::write(&file, b"x").expect("file");
    // A stage below a file cannot be inspected, which stops the removal right after the phase is written.
    let mut journal: Journal = sample(root.as_path());
    journal.stage_container = file.join("stage");
    let path: PathBuf = journal_root.join("id.json");
    write_journal(path.as_path(), &journal).expect("write");
    let pending: PendingJournal = PendingJournal {
        path: path.clone(),
        journal,
    };
    assert!(remove_pending_journal(&pending).is_err());
    let written: String = std::fs::read_to_string(&path).expect("the journal stays");
    assert!(written.contains("\"phase\":\"complete\""), "{written}");
    remove(root.as_path());
}

/// A path that cannot be inspected or removed is a failure,
/// never an absent path.
#[cfg(unix)]
#[test]
fn removals_report_what_they_cannot_inspect() {
    let root: PathBuf = fixture("journal-uninspectable");
    let file: PathBuf = root.join("file");
    std::fs::write(&file, b"x").expect("file");
    let below: PathBuf = file.join("below");
    assert!(prepare_stage_removal(below.as_path()).is_err());
    assert!(remove_tree_if_present(below.as_path()).is_err());
    assert!(remove_file_if_present(below.as_path()).is_err());
    let absent: PathBuf = root.join("absent");
    prepare_stage_removal(absent.as_path()).expect("an absent stage needs no preparation");
    remove_tree_if_present(absent.as_path()).expect("an absent tree is removed");
    remove_file_if_present(absent.as_path()).expect("an absent file is removed");
    let common: PathBuf = file.clone();
    assert!(
        read_pending_journals(common.as_path()).is_err(),
        "a common directory that is a file cannot hold journals"
    );
    remove(root.as_path());
}

/// A journal root that cannot be created for a reason other than already existing names that reason.
#[cfg(unix)]
#[test]
fn a_refused_creation_names_the_refusal() {
    let root: PathBuf = fixture("journal-create-refused");
    let common: PathBuf = std::fs::canonicalize(&root).expect("canonical").join("common");
    std::fs::create_dir(&common).expect("common");
    std::fs::set_permissions(&common, std::fs::Permissions::from_mode(0o500)).expect("lock");
    let outcome = ensure_journal_root(common.as_path());
    std::fs::set_permissions(&common, std::fs::Permissions::from_mode(0o700)).expect("unlock");
    // A superuser creates the directory anyway, so only an ordinary user shows the refusal.
    if let Err(failure) = outcome {
        assert!(failure.0.contains("Permission denied"), "{}", failure.0);
    }
    remove(root.as_path());
}

/// A private path is refused for the wrong kind,
/// a second link,
/// and a route through a link.
#[cfg(unix)]
#[test]
fn private_paths_are_refused_for_kind_links_and_route() {
    let root: PathBuf = std::fs::canonicalize(fixture("journal-private-paths")).expect("canonical");
    let directory: PathBuf = root.join("directory");
    std::fs::create_dir(&directory).expect("directory");
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).expect("mode");
    let file: PathBuf = root.join("file");
    std::fs::write(&file, b"x").expect("file");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).expect("mode");
    assert_private_path(directory.as_path(), Role::JournalDirectory).expect("a private directory");
    assert_private_path(directory.as_path(), Role::PrivateStage).expect("a private stage");
    assert_private_path(file.as_path(), Role::JournalFile).expect("a private file");
    assert!(assert_private_path(file.as_path(), Role::JournalDirectory).is_err());
    assert!(assert_private_path(file.as_path(), Role::PrivateStage).is_err());
    assert!(assert_private_path(directory.as_path(), Role::JournalFile).is_err());
    std::fs::hard_link(&file, root.join("second-link")).expect("second link");
    assert!(assert_private_path(file.as_path(), Role::JournalFile).is_err());
    // A private directory reached through a link to its parent is the same directory by another route.
    let route: PathBuf = root.join("route");
    std::os::unix::fs::symlink(&root, &route).expect("link to the parent");
    let indirect: PathBuf = route.join("directory");
    assert!(assert_private_path(indirect.as_path(), Role::JournalDirectory).is_err());
    remove(root.as_path());
}

/// Only the temporaries of the one journal are removed:
/// not the journal,
/// not another journal's temporaries,
///  not files that merely share its prefix or the temporary suffix.
#[test]
fn only_a_journals_own_temporaries_are_removed() {
    let root: PathBuf = fixture("journal-own-temporaries");
    let journal: PathBuf = root.join("one.json");
    let own: PathBuf = root.join("one.json.abc.tmp");
    let kept: Vec<PathBuf> = vec![
        journal.clone(),
        root.join("two.json.abc.tmp"),
        root.join("one.json.abc.keep"),
        root.join("loose.tmp"),
    ];
    std::fs::write(own.as_path(), b"cut off").expect("own temporary");
    for path in &kept {
        std::fs::write(path.as_path(), b"stays").expect("kept file");
    }
    remove_unfinished_writes(journal.as_path()).expect("removed");
    assert!(!own.exists(), "the journal's own temporary is removed");
    for path in &kept {
        assert!(path.exists(), "{} stays", path.display());
    }
    remove(root.as_path());
}

/// The removal's workers are jobs of the process's blocking pool:
/// a worker asks for the one place of the process's bound before its subtree is removed,
/// which work on a thread of its own never does.
#[test]
fn the_removal_workers_take_places_of_the_processs_bound() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::RecvTimeoutError;
    let (given_back, wanted): (std::sync::Arc<AtomicBool>, std::sync::mpsc::Receiver<bool>) = crate::parallel_test_support::hold_the_only_place();
    let root: PathBuf = fixture("journal-split-removal-places");
    let stage: PathBuf = root.join("stage");
    for subtree in 0..9 {
        let directory: PathBuf = stage.join(format!("payload/root/group/subtree{subtree}/deeper"));
        std::fs::create_dir_all(directory.as_path()).expect("directories");
        std::fs::write(directory.join("file"), b"x").expect("file");
    }
    remove_tree_split(stage.as_path()).expect("removed");
    assert!(!stage.exists(), "every subtree is removed");
    // The holder gives its place back when a job of the work asks for one, so this answer is the proof that the
    //  removals reached the process's bound instead of starting at once.
    match wanted.recv_timeout(crate::parallel_test_support::PATIENCE) {
        Ok(true) => {}
        Ok(false) => panic!("the job that holds the only place saw no ask, so no worker of the split removal took a place of the process's bound"),
        Err(RecvTimeoutError::Timeout) => panic!("no worker of the split removal asked for a place of the process's bound within the patience, so the subtrees were not removed by jobs of the blocking pool"),
        Err(RecvTimeoutError::Disconnected) => panic!("the job that holds the only place ended without answering"),
    }
    assert!(given_back.load(Ordering::SeqCst), "the place of the process's bound was never given back to a worker of the removal");
    remove(root.as_path());
}

/// A stage with several subtrees below the split depth is removed whole,
/// by workers sharing the subtrees.
#[test]
fn a_stage_with_many_subtrees_is_removed_whole() {
    let root: PathBuf = fixture("journal-split-removal");
    let stage: PathBuf = root.join("stage");
    for subtree in 0..9 {
        let directory: PathBuf = stage.join(format!("payload/root/group/subtree{subtree}/deeper"));
        std::fs::create_dir_all(directory.as_path()).expect("directories");
        std::fs::write(directory.join("file"), b"x").expect("file");
        std::fs::write(stage.join(format!("payload/root/group/subtree{subtree}/near")), b"y").expect("near file");
    }
    std::fs::write(stage.join("payload/top"), b"z").expect("shallow file");
    remove_tree_if_present(stage.as_path()).expect("removed");
    assert!(!stage.exists());
    // An absent tree is the normal answer for a retried removal.
    remove_tree_if_present(stage.as_path()).expect("absent");
    remove(root.as_path());
}

/// A journal named by a listing that is gone by the time it is read is skipped,
/// while a journal that is there and corrupt is
/// still an error:
/// the skip is for a vanished file only.
#[test]
fn a_vanished_journal_is_skipped_and_a_corrupt_one_is_not() {
    let root: PathBuf = fixture("journal-vanished");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    assert_eq!(read_listed_journal(journals.as_path(), "gone.json").expect("skipped"), None);
    crate::private_storage::write_private_file(journals.join("corrupt.json").as_path(), b"not json").expect("corrupt journal");
    read_listed_journal(journals.as_path(), "corrupt.json").expect_err("a present corrupt journal is reported");
    remove(root.as_path());
}

/// A journal's name carries its owner,
/// the owner reads back from the name,
/// and a name without one has no owner.
#[test]
fn a_journal_names_its_owner() {
    let owner = crate::worktree_copy_owner::current_copy_owner().expect("owner");
    let path: PathBuf = PathBuf::from(format!("/journals/abc.{}.json", owner.token()));
    assert_eq!(owner_token_of(path.as_path()), Some(owner.token()));
    assert_eq!(owner_token_of(Path::new("/journals/abc.json")), None, "a name an earlier release wrote names no owner");
    assert_eq!(owner_token_of(Path::new("/journals/abc.def.stage")), None, "a file that is no journal");
    assert_eq!(owner_token_of(Path::new("/journals/abc..json")), None, "a name without a token field names no owner");
}

/// A temporary file names the process that writes it,
/// the name reads back to the writer,
/// and a name of another shape has none.
#[test]
fn a_temporary_names_its_writer() {
    let owner = crate::worktree_copy_owner::current_copy_owner().expect("owner");
    let temporary: PathBuf = temporary_path(Path::new("/journals/abc.stage"), &owner, "0f1e");
    let name: String = temporary.file_name().expect("name").to_string_lossy().into_owned();
    assert_eq!(temporary_writer_token(name.as_str()), Some(owner.token().as_str()));
    assert_eq!(temporary_writer_token("abc.stage.0f1e.tmp"), Some("stage"), "an older release's name reads as a token that names nobody");
    assert_eq!(temporary_writer_token("abc..0f1e.tmp"), None, "a name without a token field names no writer");
    assert_eq!(temporary_writer_token("abc.tmp"), None);
    assert_eq!(temporary_writer_token("abc.json"), None, "only temporaries have writers");
    assert!(crate::worktree_copy_owner::CopyOwner::from_token("stage").is_none());
}

/// A write onto a name that cannot be replaced fails and removes its own temporary.
#[test]
fn a_failed_write_removes_its_temporary() {
    let root: PathBuf = fixture("journal-failed-write");
    let target: PathBuf = root.join("item.stage");
    std::fs::create_dir(target.as_path()).expect("a directory where the file belongs");
    write_atomic(target.as_path(), b"x", "item").expect_err("the rename onto a directory fails");
    let leftovers: Vec<String> = std::fs::read_dir(root.as_path()).expect("listing").flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).filter(|name| { return name.ends_with(".tmp"); }).collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    remove(root.as_path());
}

/// A journal name with its owner and a temporary of that journal fit a file name,
/// whatever the platform's identity is.
#[test]
fn the_longest_names_fit_a_file_name() {
    let owner = crate::worktree_copy_owner::CopyOwner { pid: 4_194_304, birth_identity: String::from("linux:123456789012"), pid_namespace: Some(String::from("pid:[4026531836]")) };
    let identifier: String = "0".repeat(36);
    let journal: String = format!("{identifier}.{}{JOURNAL_SUFFIX}", owner.token());
    let temporary: String = format!("{journal}.{}.{identifier}{TEMPORARY_SUFFIX}", owner.token());
    assert!(temporary.len() <= 255, "{} bytes", temporary.len());
}

/// Where the system starts no thread the removal workers' subtrees are removed in turn.
/// The body runs in a child that is refused
/// every thread.
#[test]
fn a_split_removal_where_no_thread_can_start_removes_the_whole_tree() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "a_split_removal_where_no_thread_can_start_removes_the_whole_tree") {
        return;
    }
    let root: std::path::PathBuf = crate::test_support::fixture("journal-removal-no-threads");
    let tree: std::path::PathBuf = root.join("stage");
    for branch in 0..4 {
        let deep: std::path::PathBuf = tree.join(format!("a{branch}/b/c/d"));
        std::fs::create_dir_all(deep.as_path()).expect("directories");
        std::fs::write(deep.join("file"), b"x").expect("file");
    }
    remove_tree_split(tree.as_path()).expect("every subtree is removed here");
    assert!(!tree.exists());
    crate::test_support::remove(root.as_path());
}

/// A source repository with one ignored `cache` file,
/// and a destination registered by real Git whose parent holds `parent_bytes`.
fn staged_for_destination(name: &str, parent_bytes: &[u8]) -> (PathBuf, PathBuf, PathBuf, StagedSnapshot) {
    use crate::test_support::{git, git_context, repository};
    use crate::worktree_copy_selection::CopySelection;
    use crate::worktree_copy_stage::{new_stage_container, stage_ignored_snapshot};
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "source");
    std::fs::write(main.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(main.join("cache")).expect("cache");
    std::fs::write(main.join("cache/data"), b"payload").expect("data");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    let parent: PathBuf = root.join(OsStr::from_bytes(parent_bytes));
    std::fs::create_dir_all(parent.as_path()).expect("a parent that is not text");
    let destination: PathBuf = parent.join("topic");
    let add: Vec<std::ffi::OsString> = ["worktree", "add", "--quiet", "-b", "topic"]
        .iter()
        .map(std::ffi::OsString::from)
        .chain([destination.clone().into_os_string()])
        .collect();
    git(main.as_path(), add.as_slice());
    let canonical: PathBuf = std::fs::canonicalize(destination).expect("canonical destination");
    let canonical_source: PathBuf = std::fs::canonicalize(main).expect("canonical source");
    let snapshot: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        canonical_source.as_path(),
        new_stage_container(canonical.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    return (root, common, canonical, snapshot);
}

/// The fields of a journal that the release installed before the bytes fields read as text,
/// copied from `read_shape` in `worktree_copy_journal_encoding_and_validation.rs` at commit 9462675cd8.
/// Why:
///  A record this build writes must still be read by that release when the path is text,
///  and must make it refuse rather than act on the replacement spelling when it is not.
fn earlier_release_roots(text: &str) -> Option<Vec<String>> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let object = value.as_object()?;
    if object.get("version")?.as_f64()? != 1.0 {
        return None;
    }
    let mut roots: Vec<String> = Vec::new();
    for field in ["destinationRoot", "sourceRoot", "stageContainer", "stageRoot"] {
        roots.push(String::from(object.get(field)?.as_str()?));
    }
    return Some(roots);
}

/// The temporary a write publishes from keeps every byte of the path it stands beside,
/// so a write under a directory that is not text renames onto the real name.
#[test]
fn a_temporary_beside_a_path_that_is_not_text_keeps_its_bytes() {
    use crate::test_support::dead_copy_owner;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("journal-temporary-bytes");
    let path: PathBuf = root.join(OsStr::from_bytes(b"journals\xff")).join("id.json");
    let owner: crate::worktree_copy_owner::CopyOwner = dead_copy_owner();
    let temporary: PathBuf = temporary_path(path.as_path(), &owner, "unique");
    let mut expected: std::ffi::OsString = path.clone().into_os_string();
    expected.push(format!(".{}.unique.tmp", owner.token()));
    assert_eq!(temporary, PathBuf::from(expected), "the suffix follows the path's own bytes");
    assert_eq!(temporary.parent(), path.parent(), "the temporary stands beside the journal");
    remove(root.as_path());
}

/// A journal keeps the bytes of its four roots,
/// so recovery reads the destination the copy was made for,
/// and installs the ignored state there.
#[test]
fn recovery_installs_into_a_destination_whose_path_is_not_text() {
    use crate::worktree_copy_recovery::recover_worktree_copy_transactions;
    use crate::test_support::dead_copy_owner;
    let (root, common, destination, snapshot) = staged_for_destination("journal-recovery-bytes", b"par\xffnt");
    let pending: PendingJournal = create_journal_for(common.as_path(), destination.as_path(), &snapshot, &dead_copy_owner()).expect("journal");
    let text: String = std::fs::read_to_string(pending.path.as_path()).expect("journal file");
    assert!(
        text.contains(format!("\"destinationRootHex\":{}", quote(hex_of_path(destination.as_path()).as_str())).as_str()),
        "the record keeps the destination's bytes: {text}"
    );
    assert!(
        !text.contains("\"destinationRoot\":\""),
        "a path that is not text has no plain spelling that names it: {text}"
    );
    let report = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 1, "{report:?}");
    assert_eq!(std::fs::read(destination.join("cache/data")).expect("copied"), b"payload");
    let replacement: PathBuf = PathBuf::from(destination.to_string_lossy().into_owned());
    assert!(!replacement.exists(), "nothing is installed at the replacement spelling");
    remove(root.as_path());
}

/// Two journals whose roots differ only in a byte that is not text are two records:
/// recovery fills each destination with its own copy and neither takes the other's.
#[test]
fn two_journals_whose_roots_differ_only_in_bytes_are_two_records() {
    use crate::test_support::dead_copy_owner;
    use crate::worktree_copy_recovery::recover_worktree_copy_transactions;
    use crate::worktree_copy_selection::CopySelection;
    use crate::worktree_copy_stage::{new_stage_container, stage_ignored_snapshot};
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("journal-two-byte-names");
    let main: PathBuf = crate::test_support::repository(root.as_path(), "source");
    std::fs::write(main.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(main.join("cache")).expect("cache");
    std::fs::write(main.join("cache/data"), b"payload").expect("data");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    // Two destinations of one repository, under parents that are the same text and differ in one byte.
    let mut destinations: Vec<PathBuf> = Vec::new();
    for (bytes, branch) in [(b"one\xff".as_slice(), "first"), (b"one\xfe".as_slice(), "second")] {
        let parent: PathBuf = root.join(OsStr::from_bytes(bytes));
        std::fs::create_dir_all(parent.as_path()).expect("parent");
        let destination: PathBuf = parent.join("topic");
        let arguments: Vec<std::ffi::OsString> = ["worktree", "add", "--quiet", "-b", branch]
            .iter()
            .map(std::ffi::OsString::from)
            .chain([destination.clone().into_os_string()])
            .collect();
        crate::test_support::git(main.as_path(), arguments.as_slice());
        destinations.push(std::fs::canonicalize(destination).expect("canonical destination"));
    }
    let source: PathBuf = std::fs::canonicalize(main.as_path()).expect("canonical source");
    let mut journals: Vec<PendingJournal> = Vec::new();
    for destination in &destinations {
        let snapshot: StagedSnapshot = stage_ignored_snapshot(
            &crate::test_support::git_context(),
            source.as_path(),
            new_stage_container(destination.as_path()).expect("container").as_path(),
            &[],
            &CopySelection::Ignored,
        )
        .expect("staged");
        journals.push(create_journal_for(common.as_path(), destination.as_path(), &snapshot, &dead_copy_owner()).expect("journal"));
    }
    let first_text: String = std::fs::read_to_string(journals[0].path.as_path()).expect("first file");
    let second_text: String = std::fs::read_to_string(journals[1].path.as_path()).expect("second file");
    assert_ne!(first_text, second_text, "the two destinations are two records");
    assert!(
        first_text.contains(format!("\"destinationRootHex\":{}", quote(hex_of_path(destinations[0].as_path()).as_str())).as_str()),
        "the first names its own bytes: {first_text}"
    );
    assert!(
        second_text.contains(format!("\"destinationRootHex\":{}", quote(hex_of_path(destinations[1].as_path()).as_str())).as_str()),
        "the second names its own bytes: {second_text}"
    );
    let report = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 2, "{report:?}");
    for destination in &destinations {
        assert_eq!(std::fs::read(destination.join("cache/data")).expect("copied"), b"payload", "{}", destination.display());
        let replacement: PathBuf = PathBuf::from(destination.to_string_lossy().into_owned());
        assert!(!replacement.exists(), "nothing is written at the replacement spelling");
    }
    remove(root.as_path());
}

/// The release now installed reads a journal this build writes for text roots,
/// and refuses one whose roots hold other bytes instead of acting on the replacement spelling.
#[test]
fn an_earlier_release_reads_a_text_journal_and_refuses_one_that_names_other_bytes() {
    use crate::test_support::dead_copy_owner;
    let (text_root, text_common, text_destination, text_snapshot) = staged_for_destination("journal-earlier-text", b"parent");
    let text_journal: PendingJournal = create_journal_for(text_common.as_path(), text_destination.as_path(), &text_snapshot, &dead_copy_owner()).expect("journal");
    let text: String = std::fs::read_to_string(text_journal.path.as_path()).expect("journal file");
    let roots: Vec<String> = earlier_release_roots(text.as_str()).expect("the installed release reads a text journal");
    assert_eq!(roots[0], text_destination.to_string_lossy(), "the destination is the one this build named");
    assert_eq!(roots[2], text_snapshot.stage_container.to_string_lossy(), "the stage is the one this build named");
    remove(text_root.as_path());
    let (bytes_root, bytes_common, bytes_destination, bytes_snapshot) = staged_for_destination("journal-earlier-bytes", b"par\xffnt");
    let bytes_journal: PendingJournal = create_journal_for(bytes_common.as_path(), bytes_destination.as_path(), &bytes_snapshot, &dead_copy_owner()).expect("journal");
    let bytes_text: String = std::fs::read_to_string(bytes_journal.path.as_path()).expect("journal file");
    assert_eq!(
        earlier_release_roots(bytes_text.as_str()),
        None,
        "the installed release refuses rather than naming the replacement character"
    );
    remove(bytes_root.as_path());
}

/// A journal file whose bytes are not text is damage:
/// reading it with replacement characters would name another directory,
/// so it is refused by name and kept.
#[test]
fn a_journal_file_that_is_not_text_is_refused_and_kept() {
    use crate::private_storage::write_private_file;
    let root: PathBuf = fixture("journal-not-text");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let journals: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let path: PathBuf = journals.join("id.json");
    let record: &[u8] = b"{\"createdEntries\":[],\"intendedEntries\":[],\"destinationRoot\":\"/w/a\xffb\",\"phase\":\"staged\",\"selectedRoots\":[],\"sourceRoot\":\"/w/s\",\"stageContainer\":\"/w/.cli-git-worktree-copy-x\",\"stageRoot\":\"/w/.cli-git-worktree-copy-x/payload\",\"version\":1}\n";
    write_private_file(path.as_path(), record).expect("journal");
    let failure: WorktreeCopyError = read_pending_journals(common.as_path()).expect_err("not text");
    assert!(failure.0.contains("it is not text"), "{}", failure.0);
    assert!(path.exists(), "a journal that cannot be read is kept");
    remove(root.as_path());
}

/// A journal keeps the bytes of the ignored roots and the claimed paths,
/// so a recovered installation writes each entry under its own name,
/// and nothing appears at the spelling a conversion would have made.
#[cfg(unix)]
#[test]
fn recovery_installs_an_ignored_name_that_is_not_text_under_its_own_name() {
    use crate::worktree_copy_recovery::recover_worktree_copy_transactions;
    use crate::test_support::dead_copy_owner;
    use crate::worktree_copy_selection::CopySelection;
    use std::os::unix::ffi::OsStrExt;
    let root: PathBuf = fixture("journal-ignored-name-bytes");
    let source: PathBuf = crate::test_support::repository(root.as_path(), "source");
    std::fs::write(source.join(".gitignore"), b"cache/\n").expect("gitignore");
    std::fs::create_dir_all(source.join("cache")).expect("cache");
    let mut awkward: PathBuf = source.join("cache");
    awkward.push(std::ffi::OsStr::from_bytes(b"bad\xff.bin"));
    std::fs::write(awkward.as_path(), b"payload").expect("file that is not text");
    let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
    let parent: PathBuf = root.join("destinations");
    std::fs::create_dir_all(parent.as_path()).expect("parent");
    let named: PathBuf = parent.join("topic");
    crate::test_support::git(source.as_path(), &["worktree", "add", "--quiet", "-b", "one", named.to_str().expect("utf8")]);
    let destination: PathBuf = std::fs::canonicalize(named).expect("canonical destination");
    let canonical_source: PathBuf = std::fs::canonicalize(source.as_path()).expect("canonical source");
    let snapshot: StagedSnapshot = crate::worktree_copy_stage::stage_ignored_snapshot(
        &crate::test_support::git_context(),
        canonical_source.as_path(),
        crate::worktree_copy_stage::new_stage_container(destination.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect("staged");
    let pending: PendingJournal = create_journal_for(common.as_path(), destination.as_path(), &snapshot, &dead_copy_owner()).expect("journal");
    let text: String = std::fs::read_to_string(pending.path.as_path()).expect("journal file");
    assert!(
        text.contains("\"selectedRoots\":[\"cache\"]"),
        "a root whose name is text keeps the text form the earlier release reads: {text}"
    );
    assert!(!text.contains("\"selectedRootsHex\""), "the bytes form is written only where the text is not exact: {text}");
    let report = recover_worktree_copy_transactions(common.as_path()).expect("recovered");
    assert_eq!(report.recovered, 1, "{report:?}");
    let mut installed: PathBuf = destination.join("cache");
    installed.push(std::ffi::OsStr::from_bytes(b"bad\xff.bin"));
    assert_eq!(std::fs::read(installed.as_path()).expect("copied under its own name"), b"payload");
    let replacement: PathBuf = destination.join("cache/bad\u{FFFD}.bin");
    assert!(!replacement.exists(), "nothing is installed at the replacement spelling");
    remove(root.as_path());
}

/// What:
///  A journal whose selected root and claimed path are not text keeps their bytes,
/// and the release before the bytes form refuses it rather than reading a replacement spelling.
/// Why:
///   The earlier release reads those two lists as arrays of text and has no field for their bytes,
/// so a record that carries bytes only must make it refuse.
#[cfg(unix)]
#[test]
fn a_journal_keeps_the_bytes_of_its_lists_and_an_earlier_release_refuses_one_that_has_no_text() {
    let root: PathBuf = fixture("journal-list-bytes");
    let journal_root: PathBuf = root.join("journals");
    let path: PathBuf = journal_root.join("id.json");
    let awkward: Vec<u8> = b"cache/bad\xff.bin".to_vec();
    let mut journal: Journal = sample(root.as_path());
    journal.selected_roots = vec![awkward.clone()];
    journal.intended_entries = vec![awkward.clone()];
    let text: String = encode_journal(&journal);
    assert!(
        text.contains(&format!("\"selectedRootsHex\":[{}]", quote(hex_of_bytes(awkward.as_slice()).as_str()))),
        "the root's bytes are in the record: {text}"
    );
    assert!(
        text.contains(&format!("\"intendedEntriesHex\":[{}]", quote(hex_of_bytes(awkward.as_slice()).as_str()))),
        "the claim's bytes are in the record: {text}"
    );
    assert!(!text.contains("\"selectedRoots\":["), "a root that is not text has no plain spelling: {text}");
    assert!(!text.contains("\"intendedEntries\":["), "a claim that is not text has no plain spelling: {text}");
    let read_back: Journal =
        parse_journal(text.as_str(), path.as_path(), journal_root.as_path()).expect("a journal whose lists are bytes reads");
    assert_eq!(read_back.selected_roots, vec![awkward.clone()]);
    assert_eq!(read_back.intended_entries, vec![awkward.clone()]);
    let earlier: String = strip_bytes_fields(text.as_str());
    assert!(
        parse_journal(earlier.as_str(), path.as_path(), journal_root.as_path()).is_err(),
        "the earlier release's fields name no list, so it refuses the record: {earlier}"
    );
    remove(root.as_path());
}

/// What:
///  A journal,
/// and the removal that settles it,
/// are published whole and flush nothing.
/// Why:
///   A journal names a copy into a worktree Git has just made,
/// whose every byte the source still holds,
/// so a power cut costs the rollback of a partial installation and never a byte of a person's;
/// native Git flushes nothing for the worktree add it serves,
/// and a flush here is a commit of the whole btrfs filesystem on the add's waiting path
/// (`SPEC.md`,
/// "The records an add writes for itself").
/// The control that the recorders see a flush is a durable private file and a directory sync.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeJournal(); settle(); expect(fileSyncs()).toEqual([]); expect(directorySyncs()).toEqual([]);
/// ```
#[test]
fn the_journal_and_its_removal_flush_nothing() {
    let root: PathBuf = fixture("journal-flushes-nothing");
    let common: PathBuf = root.join("common");
    std::fs::create_dir(common.as_path()).expect("common");
    let journal_root: PathBuf = ensure_journal_root(common.as_path()).expect("root");
    let journal: Journal = sample(root.as_path());
    let stage: PathBuf = journal.stage_container.clone();
    std::fs::create_dir_all(stage.as_path()).expect("stage");
    let path: PathBuf = journal_root.join("id.json");
    take_file_syncs();
    take_directory_syncs();
    write_journal(path.as_path(), &journal).expect("write");
    let written: String = std::fs::read_to_string(path.as_path()).expect("the journal is there");
    assert!(written.contains("\"phase\":\"installing\""), "the journal is published whole: {written}");
    remove_pending_journal(&PendingJournal { path: path.clone(), journal }).expect("removed");
    assert_eq!(take_file_syncs(), Vec::<PathBuf>::new(), "a journal flushes no file");
    assert_eq!(take_directory_syncs(), Vec::<PathBuf>::new(), "a journal flushes no directory");
    assert!(!stage.exists() && !path.exists(), "the journal and its stage are gone");
    let control: PathBuf = root.join("durable-control");
    crate::private_storage::write_private_file(control.as_path(), b"control").expect("control");
    crate::private_storage::sync_directory(root.as_path()).expect("control directory");
    assert!(!take_file_syncs().is_empty() && !take_directory_syncs().is_empty(), "the recorders see a durable write");
    remove(root.as_path());
}
