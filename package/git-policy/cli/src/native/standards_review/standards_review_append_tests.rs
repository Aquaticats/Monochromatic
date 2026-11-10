//! Controls for appending to the shared live log:
//! append only,
//! created when missing,
//! once per commit,
//! and whole under
//! concurrent appenders.

use super::*;
use crate::standards_review_entry::SHORT_SHA_LENGTH;
use crate::test_support::{fixture, remove};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Threads that deliver at the same time,
/// each through its own descriptor,
/// as separate processes would.
const APPENDERS: usize = 6;

/// Entries each thread delivers.
const ENTRIES_PER_APPENDER: usize = 5;

/// Lines in one concurrent entry;
/// long enough that a writer splitting an entry into several writes is overtaken.
const LINES_PER_ENTRY: usize = 200;

/// A commit name whose first characters are `tag`.
fn sha(tag: &str) -> String {
    return format!("{tag:0<40}");
}

/// An entry text that names the commit and ends with its receipt line and one line feed,
/// as the renderer writes it;
/// the delivery supplies the line feed that separates it from the entry before it.
fn entry(tag: &str) -> String {
    return format!("- Date:\n  D\n  Subject:\n  {} `s`\n  reviewed: {}\n", &sha(tag)[..SHORT_SHA_LENGTH], sha(tag));
}

/// A log path inside a fixture directory that does not exist yet,
/// as the live log is before the first review.
fn absent_log(root: &Path) -> PathBuf {
    return root.join("monochromatic-git-policy-cli").join("standards-review-shadow.md");
}

/// An entry is added after what the log holds,
/// whatever its ending,
/// and nothing already there changes:
/// a last line without its line feed is finished with one,
/// so the entry starts on a line of its own.
#[test]
fn an_entry_goes_after_the_existing_content() {
    let root: PathBuf = fixture("append-after");
    let log: PathBuf = root.join("log.md");
    std::fs::write(log.as_path(), "# Log\nlast line without newline").expect("log");
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()), Ok(Delivery::Appended));
    let text: String = std::fs::read_to_string(log.as_path()).expect("log");
    assert_eq!(text, format!("# Log\nlast line without newline\n{}", entry("aaa")), "the entry's write begins with the line feed that finishes the unfinished line");
    assert!(text.starts_with("# Log\nlast line without newline"), "what was there is left as it was");
    remove(root.as_path());
}

/// A missing log and its missing directory are created by the first delivery,
/// and the next one appends to the same file.
#[test]
fn a_missing_log_and_its_directory_are_created() {
    let root: PathBuf = fixture("append-created");
    let log: PathBuf = absent_log(root.as_path());
    assert!(!log.parent().expect("parent").exists());
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()), Ok(Delivery::Appended));
    assert_eq!(deliver(log.as_path(), sha("bbb").as_str(), entry("bbb").as_str()), Ok(Delivery::Appended));
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("log"), format!("\n{}\n{}", entry("aaa"), entry("bbb")));
    remove(root.as_path());
}

/// A log the service creates is readable by its owner only,
/// because entries carry commit subjects.
#[cfg(unix)]
#[test]
fn a_created_log_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("append-private");
    let log: PathBuf = absent_log(root.as_path());
    deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()).expect("delivery");
    let mode: u32 = std::fs::metadata(log.as_path()).expect("log metadata").permissions().mode();
    let others: u32 = 0o077;
    assert_eq!(mode & others, 0, "{mode:o}");
    remove(root.as_path());
}

/// A receipt in an older generation is still a receipt:
/// a rotation keeps every entry, so a commit whose entry moved aside is never reviewed again.
#[test]
fn a_receipt_in_an_older_generation_is_still_a_receipt() {
    let root: PathBuf = fixture("append-generation-receipt");
    let log: PathBuf = absent_log(root.as_path());
    std::fs::create_dir_all(log.parent().expect("parent")).expect("the live log's folder");
    std::fs::write(root.join("monochromatic-git-policy-cli").join("standards-review-shadow.md.2"), format!("\n{}", entry("aaa"))).expect("an older generation");
    assert!(names_commit(log.as_path(), sha("aaa").as_str()), "the generation holds this commit's entry");
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()), Ok(Delivery::AlreadyThere));
    assert!(!log.exists(), "nothing is appended for a commit the history already names");
    remove(root.as_path());
}

/// A live log past the size limit moves aside to the next free generation and a new file starts,
/// as every other log of the wrapper does,
/// so the live log never grows without a bound.
#[test]
fn a_full_standards_log_moves_aside_to_a_generation() {
    let root: PathBuf = fixture("append-rotate");
    let log: PathBuf = absent_log(root.as_path());
    std::fs::create_dir_all(log.parent().expect("parent")).expect("the live log's folder");
    let mut filler: Vec<u8> = vec![b'x'; crate::discard_audit::ROTATE_AT_BYTES as usize];
    filler[crate::discard_audit::ROTATE_AT_BYTES as usize - 1] = b'\n';
    std::fs::write(log.as_path(), filler.as_slice()).expect("a full live log");
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()), Ok(Delivery::Appended));
    assert_eq!(std::fs::read(root.join("monochromatic-git-policy-cli").join("standards-review-shadow.md.1")).expect("the generation"), filler);
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("the live log"), format!("\n{}", entry("aaa")));
    remove(root.as_path());
}

/// Delivering the same commit again adds nothing,
/// even when its entry was reflowed by another tool in between.
#[test]
fn a_commit_is_added_once_even_after_a_reflow() {
    let root: PathBuf = fixture("append-once");
    let log: PathBuf = absent_log(root.as_path());
    deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()).expect("first");
    let reflowed: String = std::fs::read_to_string(log.as_path()).expect("log").replace("  Subject:\n  ", "  Subject: ");
    std::fs::write(log.as_path(), reflowed.as_str()).expect("reflow");
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str()), Ok(Delivery::AlreadyThere));
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("log"), reflowed);
    remove(root.as_path());
}

/// A log that exists but cannot be read is reported and appended to nowhere,
/// because appending blind could repeat an entry.
#[test]
fn an_unreadable_log_is_reported_and_left_alone() {
    let root: PathBuf = fixture("append-unreadable");
    // A directory where the log should be: reading it fails with something other than "not found".
    let log: PathBuf = root.join("log.md");
    std::fs::create_dir(log.as_path()).expect("directory in place of the log");
    let result: Result<Delivery, String> = deliver(log.as_path(), sha("aaa").as_str(), entry("aaa").as_str());
    assert!(matches!(&result, Err(message) if message.contains("reading")), "{result:?}");
    assert!(log.is_dir());
    remove(root.as_path());
}

/// The expected entry of appender `thread`'s `index`th delivery:
/// a header naming it and many lines that each name it again,
/// so an entry that was split across writes shows another entry's lines inside it.
fn concurrent_entry(thread: usize, index: usize) -> (String, String) {
    let tag: String = format!("{thread:06}{index:06}");
    let lines: String = (0..LINES_PER_ENTRY).map(|line| return format!("  {tag} line {line}\n")).collect::<String>();
    return (tag.clone(), format!("- Entry {tag}:\n{lines}"));
}

/// Several threads deliver at once to one log that does not exist yet,
/// each through its own open file,
/// and every entry
/// lands whole,
/// once,
/// with no lock:
/// no entry has another entry's lines inside it,
/// none is missing or repeated,
/// and every write begins with the line feed that keeps it clear of the write before it.
#[test]
fn concurrent_appends_land_whole() {
    let root: PathBuf = fixture("append-concurrent");
    let log: PathBuf = absent_log(root.as_path());
    let start: std::sync::Barrier = std::sync::Barrier::new(APPENDERS);
    std::thread::scope(|scope| {
        for thread in 0..APPENDERS {
            let (shared, barrier) = (log.as_path(), &start);
            scope.spawn(move || {
                barrier.wait();
                for index in 0..ENTRIES_PER_APPENDER {
                    let (tag, text) = concurrent_entry(thread, index);
                    let delivered: Result<Delivery, String> = deliver(shared, tag.as_str(), text.as_str());
                    assert_eq!(delivered, Ok(Delivery::Appended));
                }
            });
        }
    });
    let written: String = std::fs::read_to_string(log.as_path()).expect("log");
    // Each expected entry, by tag.
    let mut expected: BTreeMap<String, String> = BTreeMap::new();
    for thread in 0..APPENDERS {
        for index in 0..ENTRIES_PER_APPENDER {
            let (tag, text) = concurrent_entry(thread, index);
            expected.insert(tag, text);
        }
    }
    // The log must be a line feed and one whole expected entry, again and again: every entry exactly once, and no byte of its own outside it.
    let mut remaining: BTreeMap<String, String> = expected.clone();
    let mut rest: &str = written.as_str();
    while !rest.is_empty() {
        let Some(after_feed) = rest.strip_prefix('\n') else {
            panic!("a write begins without a line feed of its own: {:?}", &rest[..rest.len().min(40)]);
        };
        let found: Option<(String, String)> = remaining.iter().find(|(_tag, text)| return after_feed.starts_with(text.as_str())).map(|(tag, text)| return (tag.clone(), text.clone()));
        let Some((tag, text)) = found else {
            panic!("no expected entry starts here: {:?}", &after_feed[..after_feed.len().min(40)]);
        };
        remaining.remove(&tag);
        rest = &after_feed[text.len()..];
    }
    assert!(remaining.is_empty(), "entries missing from the log: {:?}", remaining.keys().collect::<Vec<&String>>());
    remove(root.as_path());
}

/// The name a remote answer can spell:
/// a commit's own name inside another entry's verdict text.
#[test]
fn a_commit_name_inside_another_entry_is_not_a_receipt() {
    let root: PathBuf = fixture("append-inside-text");
    let log: PathBuf = root.join("log.md");
    let named: String = sha("bbb");
    std::fs::write(log.as_path(), format!("- Date:\n  D\n  Subject:\n  {} `s`\n  cost_verdict: no ({named} 0.9)\n\n", &sha("aaa")[..SHORT_SHA_LENGTH])).expect("log");
    assert!(!names_commit(log.as_path(), named.as_str()), "text inside another entry is not this commit's receipt");
    assert_eq!(deliver(log.as_path(), named.as_str(), entry("bbb").as_str()), Ok(Delivery::Appended), "so the review of the named commit still runs");
    remove(root.as_path());
}

/// An entry the operating system cut short before its receipt is not a finished review:
/// the retry appends the whole entry after the fragment,
/// keeps the fragment,
/// and finishes the fragment's unfinished line with a line feed of the entry's own.
#[test]
fn a_fragment_before_the_receipt_is_not_a_finished_review() {
    let root: PathBuf = fixture("append-fragment");
    let log: PathBuf = root.join("log.md");
    let whole: String = entry("aaa");
    let short: &str = &sha("aaa")[..SHORT_SHA_LENGTH];
    let cut: usize = whole.find(short).expect("the entry names the commit") + short.len();
    let fragment: &str = &whole[..cut];
    std::fs::write(log.as_path(), fragment).expect("seed the fragment");
    assert!(!names_commit(log.as_path(), sha("aaa").as_str()), "a fragment is not a receipt");
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), whole.as_str()), Ok(Delivery::Appended));
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("log"), format!("{fragment}\n{whole}"), "the fragment keeps its line and the whole entry follows it");
    remove(root.as_path());
}

/// The first byte of the two a subject's `é` is written as,
/// left behind by a write the operating system cut short.
const TORN_UTF8_FRAGMENT: [u8; 1] = [0xc3];

/// A torn UTF-8 tail is one cut-short write,
/// never a reason to stop logging:
/// a later review still appends its entry,
/// the torn bytes are kept,
/// and an intact receipt before them still counts.
#[test]
fn a_torn_utf8_tail_does_not_stop_later_deliveries() {
    let root: PathBuf = fixture("append-torn-utf8");
    let log: PathBuf = root.join("log.md");
    let mut seeded: Vec<u8> = entry("aaa").into_bytes();
    seeded.extend_from_slice(&TORN_UTF8_FRAGMENT);
    std::fs::write(log.as_path(), seeded.as_slice()).expect("seed the log");
    assert_eq!(deliver(log.as_path(), sha("bbb").as_str(), entry("bbb").as_str()), Ok(Delivery::Appended), "a later review is still logged");
    assert!(names_commit(log.as_path(), sha("aaa").as_str()), "the complete entry before the torn bytes still counts");
    let written: Vec<u8> = std::fs::read(log.as_path()).expect("log");
    assert!(written.starts_with(seeded.as_slice()), "the torn bytes are kept as they were");
    remove(root.as_path());
}

/// A receipt the cut-short write never gave a line feed is not a receipt:
/// the commit is reviewed again,
/// the fragment keeps a line of its own,
/// and a later entry cannot join it and destroy the evidence.
#[test]
fn an_unterminated_receipt_is_not_a_receipt_and_keeps_its_line() {
    let root: PathBuf = fixture("append-unterminated-receipt");
    let log: PathBuf = root.join("log.md");
    let whole: String = entry("aaa");
    let receipt: String = receipt_line(sha("aaa").as_str());
    let cut: usize = whole.find(receipt.as_str()).expect("the entry carries its receipt") + receipt.len();
    let fragment: &str = &whole[..cut];
    std::fs::write(log.as_path(), fragment).expect("seed the fragment");
    assert!(!names_commit(log.as_path(), sha("aaa").as_str()), "a receipt without its line feed is not a receipt");
    assert_eq!(deliver(log.as_path(), sha("aaa").as_str(), whole.as_str()), Ok(Delivery::Appended), "so the commit is reviewed again");
    assert_eq!(std::fs::read_to_string(log.as_path()).expect("log"), format!("{fragment}\n{whole}"), "the fragment keeps its line and the whole entry follows it");
    assert_eq!(deliver(log.as_path(), sha("bbb").as_str(), entry("bbb").as_str()), Ok(Delivery::Appended));
    let after: String = std::fs::read_to_string(log.as_path()).expect("log");
    assert!(after.lines().any(|line| return line == receipt), "a later entry leaves the first receipt a whole line: {after}");
    remove(root.as_path());
}
