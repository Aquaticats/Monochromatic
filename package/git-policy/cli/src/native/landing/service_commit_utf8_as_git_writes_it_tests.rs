//! What:
//!  The controls of Git's own byte repair:
//! a corpus of sequences that are and are not UTF-8 text,
//!  each repaired here and compared with the committer line real Git writes for the same bytes.
//! Why:
//!  The rule is Git's,
//! and only a real commit says which bytes Git records;
//!  a hand-written expectation could agree with this file and disagree with Git,
//! which is the failure that arrived here as an audit finding.

use super::{COMMIT_UTF8_WARN, repair_commit_utf8};
use crate::testing::test_support::{fixture, remove, repository};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// One source name and the bytes Git records for it.
struct Case {
    /// What the case covers, in words.
    what: &'static str,
    /// The name as a person's configuration holds it.
    given: &'static [u8],
    /// The bytes Git records,
    /// or nothing when the given bytes are text as they stand.
    repaired: Option<&'static [u8]>,
}

/// The corpus: text,
/// then every way a byte sequence fails to be text,
/// including the ones a screen of replacement characters would hide.
const CASES: &[Case] = &[
    Case { what: "plain text", given: b"Ren", repaired: None },
    Case { what: "text with a letter outside ASCII", given: "Ren\u{e9}".as_bytes(), repaired: None },
    Case { what: "a three-byte sequence", given: "\u{2007}".as_bytes(), repaired: None },
    Case { what: "a three-byte letter", given: "\u{4e2d}\u{6587}".as_bytes(), repaired: None },
    Case { what: "a four-byte sequence", given: "\u{1f4a9}".as_bytes(), repaired: None },
    Case { what: "a Latin-1 letter", given: b"Ren\xe9", repaired: Some(b"Ren\xc3\xa9") },
    Case { what: "two Latin-1 bytes", given: b"\xff\xfe", repaired: Some(b"\xc3\xbf\xc3\xbe") },
    Case { what: "an overlong sequence", given: b"\xc0\xaf", repaired: Some(b"\xc3\x80\xc2\xaf") },
    Case { what: "a surrogate", given: b"\xed\xa0\x80", repaired: Some(b"\xc3\xad\xc2\xa0\xc2\x80") },
    Case { what: "a non-character", given: b"\xef\xbf\xbe", repaired: Some(b"\xc3\xaf\xc2\xbf\xc2\xbe") },
    Case { what: "a truncated sequence", given: b"a\xe2\x82", repaired: Some(b"a\xc3\xa2\xc2\x82") },
    Case { what: "a lone continuation byte", given: b"\x82", repaired: Some(b"\xc2\x82") },
    Case { what: "a five-byte leader", given: b"\xf8\x88\x80\x80\x80", repaired: Some(b"\xc3\xb8\xc2\x88\xc2\x80\xc2\x80\xc2\x80") },
];

/// What real Git records as the committer's name for these bytes.
/// Why:
///   Git's repair is applied to the whole commit buffer it writes,
/// and the committer line is where a name that is not text reaches the object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = committerNameGitWrites(repo, Buffer.from('Ren\xe9'));
/// ```
fn committer_name_git_writes(repo: &Path, name: &[u8]) -> Vec<u8> {
    let mut commit: Command = Command::new("/usr/bin/git");
    commit
        .current_dir(repo)
        .args(["commit", "--quiet", "--allow-empty", "--message=x"])
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", OsStr::from_bytes(name))
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_AUTHOR_DATE", "2001-02-03T04:05:06+0000")
        .env("GIT_COMMITTER_DATE", "2001-02-03T04:05:06+0000")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config");
    let committed: Output = commit.output().expect("Git commits");
    assert!(committed.status.success(), "{}", String::from_utf8_lossy(committed.stderr.as_slice()));
    let raw: Output = Command::new("/usr/bin/git").current_dir(repo).args(["cat-file", "commit", "HEAD"]).output().expect("cat-file starts");
    let header: &[u8] = raw.stdout.split(|byte| { return *byte == b'\n'; }).find(|line| { return line.starts_with(b"committer "); }).expect("a committer line");
    let after: &[u8] = &header[b"committer ".len()..];
    let end: usize = after.iter().position(|byte| { return *byte == b'<'; }).expect("the email's opening bracket");
    return after[..end].strip_suffix(b" ").unwrap_or(&after[..end]).to_vec();
}

/// Every case's repair is the one Git records,
/// and a repair of a repaired name changes nothing.
#[test]
fn the_repair_is_the_one_real_git_records() {
    let root: PathBuf = fixture("utf8-repair");
    let repo: PathBuf = repository(root.as_path(), "repo");
    for case in CASES {
        let written: Vec<u8> = committer_name_git_writes(repo.as_path(), case.given);
        let (repaired, changed): (Vec<u8>, bool) = repair_commit_utf8(case.given);
        assert_eq!(repaired, written, "{}: the bytes Git records", case.what);
        assert_eq!(changed, case.repaired.is_some(), "{}: whether Git repaired it", case.what);
        if let Some(expected) = case.repaired {
            assert_eq!(repaired, expected, "{}: the repair Git 2.56.0 makes", case.what);
        }
        let (again, changed_again): (Vec<u8>, bool) = repair_commit_utf8(repaired.as_slice());
        assert_eq!(again, repaired, "{}: a repaired name is text", case.what);
        assert!(!changed_again, "{}: a repaired name needs no repair", case.what);
    }
    remove(root.as_path());
}

/// The warning is Git's, word for word.
#[test]
fn the_warning_is_gits_own() {
    assert_eq!(
        COMMIT_UTF8_WARN,
        "Warning: commit message did not conform to UTF-8.\nYou may want to amend it after fixing the message, or set the config\nvariable i18n.commitEncoding to the encoding your project uses.\n"
    );
}

/// What:
///  A message's non-character,
/// which a Rust string holds and Git does not read as text,
/// is repaired as Git repairs it.
/// Why:
///   The message reaches the service as text,
/// so a non-character is the one way a message can need the repair,
/// and the table's rows are the real commit that proves it.
#[test]
fn a_message_non_character_is_repaired_as_git_repairs_it() {
    let (repaired, changed): (Vec<u8>, bool) = repair_commit_utf8("\u{fffe}".as_bytes());
    assert!(changed);
    assert_eq!(repaired, b"\xc3\xaf\xc2\xbf\xc2\xbe");
}
