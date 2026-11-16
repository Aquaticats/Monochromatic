//! What:
//!  Controls of the record a command writes of itself and of the processes it starts:
//!  the one spelling the writer and the reader share,
//!  the two-space form older records carry,
//!  and the folder that must not grow with every command.
//! Why:
//!  The writer wrote `self  <pid>` with two spaces and the reader parsed only one,
//!  so a forwarded command's own record named nobody and its lock could never be proved abandoned,
//!  and a record with no parseable holder was never swept:
//!  every forwarded command left a file (a review of `675771e759`,
//! finding 1).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(holdersOf(recordText([['self', 42, id]]))).toEqual([[42, id]]);
//! ```

use crate::lock_holder_record_reads::{identities_recorded, sweep};
use crate::lock_holder_records::{CHILD_KIND, RECORD_HEADER, SELF_KIND, folder_of, identity_of_birth, identity_of_process, record_line, record_text};
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// What:
///  A record's own line is written in one spelling,
///  and that line reads back as the holder it names.
/// Why:
///  This is the whole defect:
///  the writer's `record_text` and the reader's parse must agree,
///  and the same line must be the one the append path writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(recordText([['self', 42, id]])).toContain(`self 42 ${id}`);
/// ```
#[test]
fn the_self_line_reads_back_as_the_process_it_names() {
    let identity: String = identity_of_birth("linux:4242");
    let text: String = record_text(&[(SELF_KIND, 42, identity.clone())]);
    assert!(text.contains(format!("{SELF_KIND} 42 {identity}").as_str()), "one space after the kind: {text:?}");
    let written: String = record_line(CHILD_KIND, 7, identity.as_str());
    assert_eq!(written, format!("{CHILD_KIND} 7 {identity}\n"), "the append path writes the same form");
    let root: PathBuf = fixture("record-self-line");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    std::fs::write(folder.join("record"), format!("{text}{written}").as_bytes()).expect("a record");
    let named: Vec<String> = identities_recorded(root.as_path(), 42);
    assert_eq!(named, vec![identity.clone()], "the self line names the process it was written for: {text:?}");
    let child: Vec<String> = identities_recorded(root.as_path(), 7);
    assert_eq!(child, vec![identity], "and a child line names its own");
    remove(root.as_path());
}

/// What:
///  A record of the two-space form,
/// which this branch wrote before the fix,
/// still names its holder.
/// Why:
///  Records of that form are on disk wherever the branch has run,
///  and a reader that dropped them would keep a killed command's lock for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(holdersOf(`self  ${pid} ${id}\n`)).toEqual([[pid, id]]);
/// ```
#[test]
fn the_two_space_form_of_a_record_still_names_its_holder() {
    let identity: String = identity_of_birth("linux:4343");
    let root: PathBuf = fixture("record-two-spaces");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    std::fs::write(folder.join("record"), format!("{RECORD_HEADER}\nself  99 {identity}\nchild  98 {identity}\n").as_bytes()).expect("a record of the older form");
    assert_eq!(identities_recorded(root.as_path(), 99), vec![identity.clone()], "the two-space self line reads back");
    assert_eq!(identities_recorded(root.as_path(), 98), vec![identity], "and so does the two-space child line");
    remove(root.as_path());
}

/// What:
///  A line that is not one of the two kinds,
///  and one whose number is not a number,
///  name nobody.
/// Why:
///  A record a person edited decides nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(holdersOf('other 1 x\nself nope x\n')).toEqual([]);
/// ```
#[test]
fn a_record_line_that_is_not_one_names_nobody() {
    let root: PathBuf = fixture("record-not-a-line");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    std::fs::write(folder.join("record"), format!("{RECORD_HEADER}\nother 1 {}\nself nope {}\nbogus\n", identity_of_birth("linux:1"), identity_of_birth("linux:2")).as_bytes()).expect("a record");
    assert!(identities_recorded(root.as_path(), 1).is_empty(), "a kind the reader does not know names nobody");
    assert!(identities_recorded(root.as_path(), 0).is_empty(), "a number that is not one names nobody");
    remove(root.as_path());
}

/// What:
///  A folder of records whose holders are all gone and which no standing lock names is emptied,
///  including the records a forwarded command writes of itself.
/// Why:
///  The sweep is what keeps the folder small enough that every lookup reads it whole;
///  records of the self form were never swept before,
/// so every command left a file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(selfRecordOfADeadProcess); sweep(folder, []); expect(readdir(folder)).toEqual([]);
/// ```
#[test]
fn a_record_of_a_finished_command_is_swept_and_one_a_lock_needs_is_kept() {
    let root: PathBuf = fixture("record-sweep");
    let folder: PathBuf = folder_of(root.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    let mut helper: std::process::Child = std::process::Command::new("talk").stdin(std::process::Stdio::null()).spawn().ok().unwrap_or_else(|| { return std::process::Command::new("sleep").arg("0").spawn().expect("a helper"); });
    let gone: i64 = i64::from(helper.id());
    let identity: String = identity_of_process(gone).expect("its identity while it runs");
    helper.wait().expect("the helper ends");
    // One record of a command that ended, and one whose number a standing lock still names.
    std::fs::write(folder.join("finished"), record_text(&[(SELF_KIND, gone, identity.clone())]).as_bytes()).expect("a finished command's record");
    std::fs::write(folder.join("needed"), record_text(&[(CHILD_KIND, gone, identity.clone())]).as_bytes()).expect("a record a lock needs");
    sweep(root.as_path(), &[gone]);
    assert!(folder.join("finished").exists() && folder.join("needed").exists(), "a number a standing lock names keeps every record that names it");
    sweep(root.as_path(), &[]);
    assert!(!folder.join("finished").exists(), "a record of a command that ended and that no lock names goes");
    assert!(!folder.join("needed").exists(), "and so does the other, once no lock names the number");
    remove(root.as_path());
}
