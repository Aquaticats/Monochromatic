//! Controls for what may leave the machine when a review is built:
//! the names Git reports for a landed commit,
//! the patch read,
//! and the bytes the request carries.
//! They run real Git,
//! the shipped reader and builder,
//! and the loopback mock,
//! so every assertion names a request the review really produced,
//! never a value a helper handed it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const review = await reviewCommit({ root, sha, answer: json({ choice: 'no' }) }); expect(review.seen).not.toContain(marker);
//! ```

use crate::standards_review_entry::SHORT_SHA_LENGTH;
use crate::standards_review_input::ChangedPath;
use crate::standards_review_mock::{ANSWER_WINDOW, Answer, CONTROL_CEILING, Mock, http_reply};
use crate::standards_review_run::{ReviewOptions, ReviewOutcome, review_landed};
use crate::standards_review_source::{read_landed_commit, read_message, read_names};
use crate::standards_review_text::commit_subject;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// A fixed clock,
/// so an entry's date and seconds never vary between runs.
const FIXED_MILLIS: u64 = 1_791_344_041_123;

/// The gateway's answer to a review that found no cost:
/// a complete verdict,
/// so a review that reaches the gateway proves it got all the way.
const NO_COST_ANSWER: &str = r#"{"answers":[{"choice":"no","probabilities":[{"value":"no","probability":0.9}]}]}"#;

/// What one review did,
/// as the controls read it.
struct Reviewed {
    /// The request the mock received,
    /// headers and body.
    seen: String,
    /// How the review ended.
    outcome: ReviewOutcome,
    /// The log the review appended to.
    log: String,
}

/// The fixed clock.
fn fixed_clock() -> u64 {
    return FIXED_MILLIS;
}

/// Options writing to `directory/log.md` and calling `endpoint` with an invented key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const options = { logPath, apiKey: 'invented', endpoint, quietWindow: ANSWER_WINDOW, callCeiling: CONTROL_CEILING };
/// ```
fn options(directory: &Path, endpoint: &str) -> ReviewOptions {
    return ReviewOptions {
        log_path: directory.join("log.md"),
        api_key: Some(String::from("invented-key-for-the-loopback-mock")),
        endpoint: String::from(endpoint),
        quiet_window: ANSWER_WINDOW,
        call_ceiling: CONTROL_CEILING,
        trusted_pem: None,
    };
}

/// What:
///  Review one landed commit of `root` against the loopback mock.
/// Why:
///   The names and the subject come from the same real-Git reads the service uses,
/// and the reader is the shipped
///  one,
/// so a defect in any of them shows up in the request the mock records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reviewCommit({ directory, root, sha, answer }): Reviewed
/// ```
fn review_commit(directory: &Path, root: &Path, sha: &str, answer: Answer) -> Reviewed {
    let context: GitContext = git_context();
    let names = read_names(&context, root, sha).expect("the changed names");
    let subject: String = commit_subject(read_message(&context, root, sha).expect("the message").as_str());
    let mock: Mock = Mock::start(answer);
    let given: ReviewOptions = options(directory, mock.url.as_str());
    let read = |files: &[ChangedPath]| return read_landed_commit(&context, root, sha, subject.as_str(), files);
    let outcome: ReviewOutcome = review_landed(names.as_slice(), sha, subject.as_str(), &read, &given, &fixed_clock);
    let seen: String = mock.finish().seen;
    let log: String = std::fs::read_to_string(given.log_path.as_path()).unwrap_or_default();
    return Reviewed { seen, outcome, log };
}

/// What:
///  Assert the review reached the gateway and its verdict reached the log.
/// Why:
///   A review that failed earlier would make the absence of a marker prove nothing,
/// so every provenance control proves the route ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(review.outcome).toEqual({ reviewed: ... }); expect(review.log).toContain('Named superlinear cost');
/// ```
fn assert_the_verdict_reached_the_log(review: &Reviewed, sha: &str) {
    assert!(matches!(&review.outcome, ReviewOutcome::Reviewed(judged) if judged.top == "no"), "the review never reached a verdict: {:?}", review.outcome);
    assert!(review.log.contains("cost_verdict: no (no 0.9)"), "the verdict never reached the log: {}", review.log);
    assert!(review.log.contains(&sha[..SHORT_SHA_LENGTH]), "the log names the reviewed commit: {}", review.log);
}

/// The marker an unchanged committed file carries:
/// it may never appear in a review of a commit that did not touch that file.
const UNCHANGED_PRIVATE_MARKER: &str = "PROVENANCE_UNCHANGED_PRIVATE_MARKER";

/// The text of the file whose name carries a line feed.
const NEWLINE_OWN_TEXT: &str = "PROVENANCE_NEWLINE_OWN_TEXT";

/// The text of the plain file after it.
const AFTER_OWN_TEXT: &str = "PROVENANCE_AFTER_OWN_TEXT";

/// A changed name containing a line feed is requested as one object,
/// so a file the commit never touched is not read,
/// labelled as the changed file and sent.
/// The name's bytes also run on into `HEAD:private.ts`,
/// which Git would resolve as a second object of the same request.
#[test]
fn a_changed_name_with_a_line_feed_never_sends_another_committed_file() {
    let directory: PathBuf = fixture("provenance-newline");
    let root: PathBuf = repository(directory.as_path(), "repo");
    std::fs::write(root.join("a.ts"), "public\n").expect("write a.ts");
    std::fs::write(root.join("private.ts"), format!("{UNCHANGED_PRIVATE_MARKER}\n")).expect("write private.ts");
    git(root.as_path(), &["add", "--", "a.ts", "private.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=base"]);
    // One line feed in the name, then a revision syntax Git accepts, then another changed file.
    let awkward: &str = "a.ts\nHEAD:private.ts";
    std::fs::write(root.join(awkward), format!("{NEWLINE_OWN_TEXT}\n")).expect("write the awkward name");
    std::fs::write(root.join("z.ts"), format!("{AFTER_OWN_TEXT}\n")).expect("write z.ts");
    git(root.as_path(), &["add", "--", awkward, "z.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=change"]);
    let sha: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    let review: Reviewed = review_commit(directory.as_path(), root.as_path(), sha.as_str(), Answer::Bytes(http_reply("200 OK", NO_COST_ANSWER)));
    assert_the_verdict_reached_the_log(&review, sha.as_str());
    assert!(!review.seen.contains(UNCHANGED_PRIVATE_MARKER), "`{UNCHANGED_PRIVATE_MARKER}` of a file the reviewed commit never touched left the machine");
    assert!(review.seen.contains(NEWLINE_OWN_TEXT), "the changed file's own text is the excerpt");
    assert!(review.seen.contains(AFTER_OWN_TEXT), "the changed file after it keeps its own text");
    remove(directory.as_path());
}

/// A changed name whose bytes are not UTF-8 is read by those bytes,
/// so the file Git would return for its lossy text,
/// which this commit never touched,
/// is not read and not sent.
#[cfg(unix)]
#[test]
fn a_changed_name_git_cannot_decode_never_sends_another_committed_file() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStringExt as _;
    let directory: PathBuf = fixture("provenance-native-bytes");
    let root: PathBuf = repository(directory.as_path(), "repo");
    // The replacement character and the byte `ff` are two distinct Linux names.
    let alias: String = format!("bad-{}.rs", '\u{fffd}');
    let native: PathBuf = PathBuf::from(std::ffi::OsString::from_vec(vec![b'b', b'a', b'd', b'-', 0xff, b'.', b'r', b's']));
    std::fs::write(root.join(alias.as_str()), format!("{UNCHANGED_PRIVATE_MARKER}\n")).expect("write the alias");
    git(root.as_path(), &["add", "--", alias.as_str()]);
    git(root.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(root.join(native.as_path()), format!("{NEWLINE_OWN_TEXT}\n")).expect("write the native name");
    git(root.as_path(), &[OsStr::new("add"), OsStr::new("--"), native.as_os_str()]);
    git(root.as_path(), &["commit", "--quiet", "--message=change"]);
    let sha: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    let review: Reviewed = review_commit(directory.as_path(), root.as_path(), sha.as_str(), Answer::Bytes(http_reply("200 OK", NO_COST_ANSWER)));
    assert_the_verdict_reached_the_log(&review, sha.as_str());
    assert!(!review.seen.contains(UNCHANGED_PRIVATE_MARKER), "`{UNCHANGED_PRIVATE_MARKER}` of the unchanged file whose lossy name the commit's changed name runs to left the machine");
    assert!(review.seen.contains(NEWLINE_OWN_TEXT), "the changed file's own text is the excerpt");
    remove(directory.as_path());
}

/// The marker of the ignored,
/// unstaged file the fixture's converter reads.
const IGNORED_MARKER: &str = "PROVENANCE_IGNORED_UNSTAGED_MARKER";

/// The marker of the file beside the repository that the fixture's converter reads.
const OUTSIDE_MARKER: &str = "PROVENANCE_OUTSIDE_REPOSITORY_MARKER";

/// The changed line of the reviewed file,
/// which is the only text of it the review may carry.
const CONVERTED_OWN_TEXT: &str = "PROVENANCE_CONVERTED_OWN_TEXT";

/// A `diff.<driver>.textconv` command this repository's `.gitattributes` selects
/// is a local presentation command:
/// its output is not committed candidate bytes,
/// so the patch read may not carry it.
/// The converter here reads a file the person ignored and did not stage,
/// and one outside the repository entirely,
/// so a review that runs it sends both.
#[test]
fn a_local_text_converter_never_sends_ignored_or_outside_text() {
    let directory: PathBuf = fixture("provenance-textconv");
    let root: PathBuf = repository(directory.as_path(), "repo");
    // Beside the repository, which is where a saved copy or a sibling checkout sits.
    std::fs::write(directory.join("outside.txt"), format!("{OUTSIDE_MARKER}\n")).expect("write outside.txt");
    std::fs::write(root.join("ignored.txt"), format!("{IGNORED_MARKER}\n")).expect("write ignored.txt");
    std::fs::write(root.join(".gitignore"), "ignored.txt\n").expect("write .gitignore");
    std::fs::write(root.join(".gitattributes"), "*.ts diff=audit\n").expect("write .gitattributes");
    std::fs::write(root.join("a.ts"), "before\n").expect("write a.ts");
    git(root.as_path(), &["add", "--", ".gitignore", ".gitattributes", "a.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(root.join("a.ts"), format!("{CONVERTED_OWN_TEXT}\n")).expect("change a.ts");
    git(root.as_path(), &["add", "--", "a.ts"]);
    // The converter is this repository's own configuration, as a person's would be.
    let config: String = std::fs::read_to_string(root.join(".git/config")).expect("read the fixture config");
    std::fs::write(root.join(".git/config"), format!("{config}\n[diff \"audit\"]\n\ttextconv = cat ignored.txt ../outside.txt\n")).expect("configure the converter");
    git(root.as_path(), &["commit", "--quiet", "--message=change"]);
    let sha: String = git_text(root.as_path(), &["rev-parse", "HEAD"]);
    let review: Reviewed = review_commit(directory.as_path(), root.as_path(), sha.as_str(), Answer::Bytes(http_reply("200 OK", NO_COST_ANSWER)));
    assert_the_verdict_reached_the_log(&review, sha.as_str());
    let carried: Vec<&str> = [IGNORED_MARKER, OUTSIDE_MARKER].into_iter().filter(|marker| return review.seen.contains(marker)).collect();
    assert!(carried.is_empty(), "the review's patch carried {carried:?}, text of no committed file");
    assert!(review.seen.contains(CONVERTED_OWN_TEXT), "the changed line itself is the review's evidence");
    remove(directory.as_path());
}
