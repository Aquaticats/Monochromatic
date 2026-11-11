//! Controls for the isolated merge's intent record.

use super::*;
use crate::merge_isolation_carry_parts as carry_parts;
use crate::private_storage::take_file_syncs;
use crate::test_support::{fixture, remove};

/// The intent every control uses.
fn intent(namespace: Option<&str>, target: Option<&str>) -> MergeIntent {
    return MergeIntent {
        owner: TransactionOwner {
            transaction_id: String::from("0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10"),
            owner_pid: 4242,
            owner_identity: String::from("linux:7"),
            created_at: String::from("2026-10-29T00:00:00.000Z"),
            owner_pid_namespace: namespace.map(String::from),
        },
        repository_root: PathBuf::from("/work/tree \"quoted\""),
        target_ref: target.map(String::from),
        old_oid: String::from("a".repeat(40)),
        new_oid: String::from("b".repeat(40)),
        side_ref: None,
        carried: None,
        landed: Vec::new(),
    };
}

/// A record reads back exactly,
/// with and without the optional fields.
#[test]
fn an_intent_round_trips() {
    for (namespace, target) in [
        (None, Some("refs/heads/main")),
        (Some("pid:[4026531836]"), Some("refs/heads/feature/x")),
        (None, None),
    ] {
        let original: MergeIntent = intent(namespace, target);
        let text: String = encode_merge_intent(&original);
        assert!(text.ends_with("}\n"));
        assert_eq!(parse_merge_intent(text.as_bytes()), Some(original));
    }
}

/// What:
///  A record keeps the bytes of the worktree root,
/// which is where recovery runs Git.
/// Why:
///   The merge is recovered in the worktree the merge was made in,
/// so a root that lost its bytes names a directory that does not exist,
/// and the recovery of a non-UTF-8 worktree fails.
#[test]
fn a_record_keeps_the_bytes_of_a_root_that_is_not_text() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let original: MergeIntent = MergeIntent { repository_root: PathBuf::from(OsStr::from_bytes(b"/work/tr\xffee")), ..intent(None, Some("refs/heads/main")) };
    let text: String = encode_merge_intent(&original);
    assert_eq!(parse_merge_intent(text.as_bytes()), Some(original));
}

/// What:
///  A record written by a release that spelled the root as plain text is still read,
/// and that text is the root.
/// Why:
///   A merge interrupted by an upgrade is recovered by the release that starts,
/// not dropped.
#[test]
fn a_record_of_a_plain_text_root_is_still_read() {
    let earlier: &str = "{\"schemaVersion\":1,\"state\":\"merge-intent\",\"transactionId\":\"0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10\",\"ownerPid\":4242,\"ownerIdentity\":\"linux:7\",\"createdAt\":\"2026-10-29T00:00:00.000Z\",\"repositoryRoot\":\"/work/tree\",\"targetRef\":\"refs/heads/main\",\"oldOid\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"newOid\":\"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"}\n";
    let read: MergeIntent = parse_merge_intent(earlier.as_bytes()).expect("an earlier record still reads");
    assert_eq!(read.repository_root, PathBuf::from("/work/tree"));
}

/// A damaged record is not an intent.
#[test]
fn a_malformed_intent_is_refused() {
    let good: String = encode_merge_intent(&intent(None, Some("refs/heads/main")));
    // The bytes of the root the record holds beside its plain text.
    let bytes: String = crate::path_bytes_in_records::hex_of_path(Path::new("/work/tree \"quoted\""));
    for broken in [
        String::new(),
        String::from("[]"),
        String::from("not json"),
        good.replace("\"schemaVersion\":2", "\"schemaVersion\":3"),
        good.replace("merge-intent", "landing"),
        good.replace("\"ownerPid\":4242", "\"ownerPid\":0"),
        good.replace("\"ownerIdentity\":\"linux:7\"", "\"ownerIdentity\":\"\""),
        good.replace(&"a".repeat(40), ""),
        good.replace("\"newOid\"", "\"other\""),
        // The plain text alone is not the root: the reader takes the bytes, and those must name one.
        good.replace(bytes.as_str(), "z"),
    ] {
        assert_eq!(parse_merge_intent(broken.as_bytes()), None, "{broken}");
    }
    assert_eq!(parse_merge_intent(&[0xff, 0xfe]), None);
}

/// What:
///  A record that ended is removed without flushing its directory,
///  and a record the power loss brings back names a
///  merge the branch already holds,
///  which recovery only loses again.
/// Why:
///  The flush after the removal was a third of the merge's fsyncs,
/// and it sits on the path the person waits for:
/// a
///  merge that needs no resolution took 22 to 41 seconds in a measured disk stall where real Git took 9 to 16
///  milliseconds (finding 1 of the merge-and-pull audit of 2026-11-09).
///  It buys no safety:
/// a record that comes back after a power loss names a merge whose commit the branch already
///  holds,
/// and `settle_failed_merge` drops it as already landed (`a_landed_merge_only_loses_its_record`),
/// or
///  one whose branch moved,
///  which it settles the same way a later command always could.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeMergeIntent(gitDir, intent); takeDirectorySyncs(); removeMergeIntent(path); expect(takeDirectorySyncs()).toEqual([]);
/// ```
#[test]
fn a_record_that_ended_is_removed_without_flushing_its_directory() {
    let root: PathBuf = fixture("merge-intent-removal-syncs");
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(git_dir.as_path()).expect("git dir");
    let path: PathBuf = write_merge_intent(git_dir.as_path(), &intent(None, Some("refs/heads/main"))).expect("the record");
    // The write's own flushes are the price of "the record survives a crash before Git starts".
    let _written: Vec<PathBuf> = crate::private_storage::take_directory_syncs();
    take_file_syncs();
    remove_merge_intent(path.as_path()).expect("removed");
    assert!(!path.exists(), "the record is gone from the directory");
    let flushed: Vec<PathBuf> = crate::private_storage::take_directory_syncs();
    assert!(flushed.is_empty(), "the removal is not flushed: a record that comes back after a power loss is settled by the next command, so nothing waits on this flush: {flushed:?}");
    remove(&root);
}

/// A record is written once,
/// listed,
/// and removed;
/// a second write of the same merge is refused.
#[test]
fn a_record_is_written_listed_and_removed() {
    let root: PathBuf = fixture("merge-intent-files");
    let git_dir: PathBuf = root.join("git-dir");
    std::fs::create_dir(git_dir.as_path()).expect("git dir");
    assert_eq!(list_merge_intents(git_dir.as_path()).expect("absent directory"), Vec::<PathBuf>::new());
    let first: MergeIntent = intent(None, Some("refs/heads/main"));
    let path: PathBuf = write_merge_intent(git_dir.as_path(), &first).expect("first write");
    assert_eq!(path, intent_path(git_dir.as_path(), first.owner.transaction_id.as_str()));
    assert_eq!(
        parse_merge_intent(std::fs::read(path.as_path()).expect("read").as_slice()),
        Some(first.clone())
    );
    assert!(write_merge_intent(git_dir.as_path(), &first).is_err(), "records are never rewritten");
    let mut second: MergeIntent = intent(None, None);
    second.owner.transaction_id = String::from("11111111-6f5b-4d0e-9a55-3f5d8e2f6a10");
    let other: PathBuf = write_merge_intent(git_dir.as_path(), &second).expect("second write");
    std::fs::write(intent_directory(git_dir.as_path()).join("notes.txt"), b"not a record").expect("stray file");
    assert_eq!(list_merge_intents(git_dir.as_path()).expect("listing"), vec![path.clone(), other.clone()]);
    remove_merge_intent(path.as_path()).expect("remove");
    remove_merge_intent(path.as_path()).expect("removing twice is not an error");
    assert_eq!(list_merge_intents(git_dir.as_path()).expect("listing"), vec![other]);
    remove(&root);
}

/// What:
///  A record that carries the person's own work reads back byte for byte,
///  including a path that is not text.
/// Why:
///  A merge interrupted with the checkout cleared is finished from these bytes,
/// so a path that lost one byte would
///  put the person's work at a name nothing reads.
#[test]
fn a_capture_round_trips() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let capture: Capture = Capture {
        commit: String::from("c".repeat(40)),
        reference: Some(String::from("refs/cli-git/carried/0b6c2c1e")),
        paths: vec![
            carry_parts::CapturedPath {
                path: b"dir/a b.txt".to_vec(),
                worktree: Some(carry_parts::CapturedFile {
                    mode: String::from("100755"),
                    raw: String::from("d".repeat(40)),
                    blob: String::from("e".repeat(40)),
                }),
                index: Some(carry_parts::CapturedIndex { mode: String::from("100644"), oid: String::from("f".repeat(40)) }),
            },
            carry_parts::CapturedPath {
                path: OsStr::from_bytes(b"not text \xff").as_bytes().to_vec(),
                worktree: None,
                index: Some(carry_parts::CapturedIndex { mode: String::from("100644"), oid: String::from("a".repeat(40)) }),
            },
        ],
    };
    let original: MergeIntent = MergeIntent { carried: Some(capture.clone()), ..intent(None, Some("refs/heads/main")) };
    let text: String = encode_merge_intent(&original);
    assert_eq!(parse_merge_intent(text.as_bytes()), Some(original));
    assert!(text.contains("\"carried\""), "the record names the capture: {text}");
}

/// What:
///  A record whose capture is damaged is refused,
///  rather than read as a merge with nothing carried.
/// Why:
///   Reading damage as absence would make the merge replace the person's paths with no copy of their
///  bytes.
#[test]
fn a_damaged_capture_is_refused() {
    let capture: Capture = Capture {
        commit: String::from("c".repeat(40)),
        reference: Some(String::from("refs/cli-git/carried/0b6c2c1e")),
        paths: vec![carry_parts::CapturedPath {
            path: b"a.txt".to_vec(),
            worktree: Some(carry_parts::CapturedFile {
                mode: String::from("100644"),
                raw: String::from("d".repeat(40)),
                blob: String::from("e".repeat(40)),
            }),
            index: None,
        }],
    };
    let text: String = encode_merge_intent(&MergeIntent { carried: Some(capture), ..intent(None, Some("refs/heads/main")) });
    assert!(parse_merge_intent(text.as_bytes()).is_some());
    for broken in [
        text.replace("100644", ""),
        text.replace("\"raw\"", "\"other\""),
        text.replace("612e747874", ""),
        text.replace("\"paths\":[", "\"paths\":[{\"index\":{\"mode\":\"100644\",\"oid\":\"ff\"}},"),
    ] {
        assert_eq!(parse_merge_intent(broken.as_bytes()), None, "{broken}");
    }
}

/// What:
///  A record keeps what Git's own update left at each path a merge changes,
///  and a path whose entry is not text is kept as bytes.
/// Why:
///   The carry after a kill asks whether the file standing there is still the one Git wrote,
/// and the record is the only place
///  a recovery can read that from
/// (a cross-vendor review of round two,
/// its first finding).
#[test]
fn a_record_keeps_what_git_wrote_at_each_path() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let landed: Vec<LandedFile> = vec![
        LandedFile {
            path: b"a.txt".to_vec(),
            identity: Some(crate::merge_isolation_changes::FileIdentity {
                device: 2049,
                inode: 1_234_567,
                size: 19,
                mode: 33_188,
                modified_seconds: 1_760_000_000,
                modified_nanos: 123_456_789,
            }),
            changed_seconds: 1_760_000_001,
            changed_nanos: 987_654_321,
        },
        LandedFile { path: OsStr::from_bytes(b"gone \xff.txt").as_bytes().to_vec(), identity: None, changed_seconds: 0, changed_nanos: 0 },
    ];
    let original: MergeIntent = MergeIntent { landed: landed.clone(), ..intent(None, Some("refs/heads/main")) };
    let text: String = encode_merge_intent(&original);
    assert_eq!(parse_merge_intent(text.as_bytes()), Some(original), "the reading reads back whole");
    assert!(text.contains("\"landed\""), "the record names the reading: {text}");
}

/// What:
///  A record whose reading is damaged is refused,
///  rather than read as a merge that read nothing.
/// Why:
///   Reading damage as nothing makes the next command judge a path by its bytes alone,
/// which is what replaces a save that
///  accepts the whole merge.
#[test]
fn a_damaged_reading_is_refused() {
    let landed: Vec<LandedFile> = vec![LandedFile {
        path: b"a.txt".to_vec(),
        identity: Some(crate::merge_isolation_changes::FileIdentity { device: 2049, inode: 7, size: 19, mode: 33_188, modified_seconds: 1, modified_nanos: 2 }),
        changed_seconds: 3,
        changed_nanos: 4,
    }];
    let text: String = encode_merge_intent(&MergeIntent { landed, ..intent(None, Some("refs/heads/main")) });
    assert!(parse_merge_intent(text.as_bytes()).is_some(), "{text}");
    for broken in [text.replace("\"inode\":7", "\"inode\":\"seven\""), text.replace("\"changedNanos\":4", "\"changedNanos\":null"), text.replace("\"pathHex\":\"612e747874\"", "")] {
        assert_eq!(parse_merge_intent(broken.as_bytes()), None, "{broken}");
    }
}
