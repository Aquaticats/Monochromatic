//! Controls for the Git reads against real Git in a fixture repository.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// The batch parser reads by size,
/// reports a missing name as absent,
/// and rejects malformed output.
#[test]
fn batch_output_is_read_by_size_with_missing_names_absent() {
    let names: Vec<ChangedPath> = ["a.ts", "gone.ts", "b.rs"].iter().map(|name| return ChangedPath::from_bytes(name.as_bytes().to_vec())).collect();
    let objects: Vec<RequestedObject> = requested_objects("s", names.as_slice());
    let output: String = format!("oid1 blob 5\nx\ny\nz\n{}oid2 blob 0\n\n", String::from_utf8(missing_line(objects[1].spec.as_slice())).expect("ascii"));
    let texts: BTreeMap<ChangedPath, String> = parse_batch(output.as_bytes(), objects.as_slice()).expect("parsed");
    let named = |text: &str| return ChangedPath::from_bytes(text.as_bytes().to_vec());
    assert_eq!(texts.get(&named("a.ts")).map(String::as_str), Some("x\ny\nz"));
    assert_eq!(texts.get(&named("b.rs")).map(String::as_str), Some(""));
    assert!(!texts.contains_key(&named("gone.ts")));
    assert!(parse_batch(b"oid1 blob 5\nx\ny\nz\noid2 blob 5\nx\ny\nz\n", &objects[..1]).is_err(), "a reply nothing asked for is not text for the last name");
    assert!(parse_batch(b"", objects.as_slice()).is_err());
    assert!(parse_batch(b"oid1 tree 3\nabc\n", &objects[..1]).is_err());
    assert!(parse_batch(b"oid1 blob 9\nabc\n", &objects[..1]).is_err());
    assert!(parse_batch(b"oid1 blob x\nabc\n", &objects[..1]).is_err());
    // A missing object whose name holds a line feed is absent, not a header the parser cannot read.
    let awkward: Vec<ChangedPath> = vec![ChangedPath::from_bytes(b"gone\nHEAD:x.ts".to_vec())];
    let awkward_objects: Vec<RequestedObject> = requested_objects("s", awkward.as_slice());
    let mut echoed: Vec<u8> = missing_line(awkward_objects[0].spec.as_slice());
    echoed.extend_from_slice(b"oid2 blob 0\n\n");
    let parsed: BTreeMap<ChangedPath, String> = parse_batch(echoed.as_slice(), &[awkward_objects[0].clone(), RequestedObject { name: ChangedPath::from_bytes(b"x.ts".to_vec()), spec: b"s:x.ts".to_vec() }]).expect("a missing awkward name parses");
    assert_eq!(parsed.len(), 1);
}

/// A commit that changes,
/// adds and deletes code files reads its own patch and the surviving files' texts.
#[test]
fn a_landed_commit_is_read_in_two_starts_worth_of_data() {
    let parent: PathBuf = fixture("source-read");
    let root: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(root.join("keep.ts"), "one\ntwo\n").expect("write");
    std::fs::write(root.join("gone.rs"), "bye\n").expect("write");
    git(root.as_path(), &["add", "--", "keep.ts", "gone.rs"]);
    git(root.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(root.join("keep.ts"), "one\nchanged\n").expect("write");
    std::fs::write(root.join("new.mjs"), "fresh\n").expect("write");
    git(root.as_path(), &["rm", "--quiet", "--", "gone.rs"]);
    git(root.as_path(), &["add", "--", "keep.ts", "new.mjs"]);
    git(root.as_path(), &["commit", "--quiet", "--message=change"]);
    let sha: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    let names: Vec<ChangedPath> = read_names(&git_context(), root.as_path(), sha.as_str()).expect("names");
    let commit: LandedCommit = read_landed_commit(&git_context(), root.as_path(), sha.as_str(), "change", names.as_slice()).expect("read");
    assert_eq!(commit.patch, (git_text(root.as_path(), &["show", "--format=", "--patch", "HEAD"]) + "\n").into_bytes(), "the patch is kept as the bytes Git wrote");
    let named = |text: &str| return ChangedPath::from_bytes(text.as_bytes().to_vec());
    assert_eq!(commit.contents.get(&named("keep.ts")).map(String::as_str), Some("one\nchanged\n"));
    assert_eq!(commit.contents.get(&named("new.mjs")).map(String::as_str), Some("fresh\n"));
    assert!(!commit.contents.contains_key(&named("gone.rs")), "a deleted file has no new-side text");
    assert_eq!(commit.sha, sha);
    remove(parent.as_path());
}

/// A commit name Git does not know is a failure that names the command.
#[test]
fn an_unknown_commit_fails_with_the_command_named() {
    let parent: PathBuf = fixture("source-unknown");
    let root: PathBuf = repository(parent.as_path(), "repo");
    let missing: String = "0".repeat(40);
    let failure: String = read_landed_commit(&git_context(), root.as_path(), missing.as_str(), "s", &[ChangedPath::from_bytes(b"a.ts".to_vec())]).expect_err("unknown");
    assert!(failure.contains("git show"), "{failure}");
    remove(parent.as_path());
}

/// The names are the raw changed paths (not quoted,
/// even with spaces and non-ASCII),
/// and the message folds to the subject.
#[test]
fn names_and_message_are_read_raw() {
    let parent: PathBuf = fixture("source-names");
    let root: PathBuf = repository(parent.as_path(), "repo");
    std::fs::create_dir_all(root.join("dir name")).expect("dir");
    let awkward: [&str; 3] = ["dir name/a b.ts", "dir name/\u{e9}.rs", "plain.md"];
    for name in awkward {
        std::fs::write(root.join(name), "x\n").expect("write");
        git(root.as_path(), &["add", "--", name]);
    }
    git(root.as_path(), &["commit", "--quiet", "--message=feat(a): subject line", "--message=body text"]);
    let sha: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    let mut found: Vec<ChangedPath> = read_names(&git_context(), root.as_path(), sha.as_str()).expect("names");
    found.sort();
    let mut expected: Vec<ChangedPath> = awkward.iter().map(|name| return ChangedPath::from_bytes(name.as_bytes().to_vec())).collect();
    expected.sort();
    assert_eq!(found, expected);
    let message: String = read_message(&git_context(), root.as_path(), sha.as_str()).expect("message");
    assert_eq!(crate::standards_review_text::commit_subject(message.as_str()), "feat(a): subject line");
    let root_commit: String = git_text(root.as_path(), &["rev-list", "--max-parents=0", "HEAD"]);
    assert!(read_names(&git_context(), root.as_path(), root_commit.as_str()).expect("root").is_empty(), "a root commit reports no names");
    remove(parent.as_path());
}

/// A commit that deletes a code file whose name holds a line feed still reads:
/// Git echoes the whole path in its `missing` reply,
/// so a request that an earlier name continued into must not be
/// split at a line feed,
/// and the deleted file is absent from the texts rather than a read failure.
#[test]
fn a_deleted_name_with_a_line_feed_is_absent() {
    let parent: PathBuf = fixture("source-deleted-newline");
    let root: PathBuf = repository(parent.as_path(), "repo");
    let awkward: &str = "gone\nnot-a-revision.ts";
    std::fs::write(root.join(awkward), "a\n").expect("write the awkward name");
    std::fs::write(root.join("keep.ts"), "one\n").expect("write keep.ts");
    git(root.as_path(), &["add", "--", awkward, "keep.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=base"]);
    git(root.as_path(), &["rm", "--quiet", "--", awkward]);
    std::fs::write(root.join("keep.ts"), "two\n").expect("change keep.ts");
    git(root.as_path(), &["add", "--", "keep.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=change"]);
    let sha: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    let names: Vec<ChangedPath> = read_names(&git_context(), root.as_path(), sha.as_str()).expect("names");
    assert!(names.iter().any(|name| return name.as_bytes() == awkward.as_bytes()), "the deleted name is reported");
    let commit: LandedCommit = read_landed_commit(&git_context(), root.as_path(), sha.as_str(), "change", names.as_slice()).expect("a deleted awkward name is not a read failure");
    let named = |text: &str| return ChangedPath::from_bytes(text.as_bytes().to_vec());
    assert_eq!(commit.contents.get(&named("keep.ts")).map(String::as_str), Some("two\n"));
    assert_eq!(commit.contents.len(), 1, "the deleted file has no text: {:?}", commit.contents.keys().map(|name| return name.to_string()).collect::<Vec<String>>());
    remove(parent.as_path());
}

/// The line of the fixture file a commit changes,
/// far enough from the head that the head window and the changed window cannot be confused.
const CHANGED_LINE: usize = 123;

/// Lines in the fixture file whose name Git prints unquoted.
const LONG_FILE_LINES: usize = 200;

/// What:
///  A commit that changes one line of a file whose path bytes are not UTF-8,
/// in a repository whose `core.quotePath` is off,
/// so Git prints those bytes in the patch header as they are.
/// Why:
///   The patch read decodes its text,
/// and a path that does not decode must still find its own section,
/// or the
///  excerpt silently falls back to the head of the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const input = buildInput(readLandedCommit({ ... }), names); expect(input).toContain(`lines 93 to 153 of 201`);
/// ```
#[cfg(unix)]
#[test]
fn an_unquoted_non_utf8_path_keeps_its_changed_line() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStringExt as _;
    let parent: PathBuf = fixture("source-unquoted-bytes");
    let root: PathBuf = repository(parent.as_path(), "repo");
    let config: String = std::fs::read_to_string(root.join(".git/config")).expect("read the fixture config");
    std::fs::write(root.join(".git/config"), format!("{config}\n[core]\n\tquotePath = false\n")).expect("configure the fixture");
    let name: PathBuf = PathBuf::from(std::ffi::OsString::from_vec(vec![b'b', b'a', b'd', b'-', 0xff, b'.', b'r', b's']));
    let mut lines: Vec<String> = (1..=LONG_FILE_LINES).map(|number| return format!("l{number}")).collect();
    std::fs::write(root.join(name.as_path()), format!("{}\n", lines.join("\n"))).expect("write the file");
    git(root.as_path(), &[OsStr::new("add"), OsStr::new("--"), name.as_os_str()]);
    git(root.as_path(), &["commit", "--quiet", "--message=base"]);
    lines[CHANGED_LINE - 1] = String::from("changed");
    std::fs::write(root.join(name.as_path()), format!("{}\n", lines.join("\n"))).expect("change the file");
    git(root.as_path(), &[OsStr::new("add"), OsStr::new("--"), name.as_os_str()]);
    git(root.as_path(), &["commit", "--quiet", "--message=change"]);
    let sha: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    let names: Vec<ChangedPath> = read_names(&git_context(), root.as_path(), sha.as_str()).expect("names");
    let commit: LandedCommit = read_landed_commit(&git_context(), root.as_path(), sha.as_str(), "change", names.as_slice()).expect("read");
    let input: crate::standards_review_input::ReviewInput = crate::standards_review_input::build_input(&commit, names.as_slice());
    let changed: i64 = i64::try_from(CHANGED_LINE).expect("a line number that fits");
    let (from, to) = (changed - crate::standards_review_input::EXCERPT_CONTEXT, changed + crate::standards_review_input::EXCERPT_CONTEXT);
    let header: &str = input.model_input.lines().find(|line| return line.starts_with("// candidate excerpt")).unwrap_or_default();
    assert!(input.model_input.contains(&format!("lines {from} to {to} of 201")), "the excerpt is the changed line's window, not {header}");
    remove(parent.as_path());
}
