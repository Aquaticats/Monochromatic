//! Controls for one process record:
//!  the fields are read after the last closing parenthesis,
//!  a record with a command name holding parentheses and spaces reads whole,
//!  a short record is blind rather than gone,
//!  a control byte in a command name is escaped,
//!  and the two errnos that mean "gone" are told from a refused read.

use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::process_session_record::*;

/// The fields a record carries after the command name,
/// in the kernel's own order.
fn stat_line(command: &str, state: char, parent: i32, group: i32, session: i32, started: u64) -> String {
    let mut fields: Vec<String> = vec![String::from(state)];
    fields.push(parent.to_string());
    fields.push(group.to_string());
    fields.push(session.to_string());
    // tty, tpgid, flags, the four faults, the two times and the sixteen other fields before the start time.
    for _filler in 0..(STARTED_FIELD - 4) {
        fields.push(String::from("0"));
    }
    fields.push(started.to_string());
    return format!("{} ({command}) {}\n", 4242, fields.join(" "));
}

/// A record whose command name holds a space and a closing parenthesis reads its fields from the last
/// parenthesis,
/// which is what the kernel's own format needs.
#[test]
fn a_command_name_holding_parentheses_does_not_move_the_fields() {
    let text: String = stat_line("a) b", 'S', 11, 12, 13, 987);
    let outcome: ReadOutcome = parse_process_record(4242, text.as_str());
    let ReadOutcome::Found(record) = outcome else {
        panic!("the record reads: {outcome:?}");
    };
    assert_eq!(record.pid, 4242);
    assert_eq!(record.state, 'S');
    assert_eq!(record.parent, 11);
    assert_eq!(record.group, 12);
    assert_eq!(record.session, 13);
    assert_eq!(record.started, 987);
    assert_eq!(record.command.text(), "a) b");
}

/// A record that stops before the start-time field is a read this process cannot use,
/// which is blind and not gone.
#[test]
fn a_short_record_is_blind() {
    let outcome: ReadOutcome = parse_process_record(1, "1 (sh) S 0 0 0");
    let ReadOutcome::Blind { word, .. } = outcome else {
        panic!("a short record is not found: {outcome:?}");
    };
    assert_eq!(word, RefusalWord::HolderBlind);
}

/// A command name carrying a control byte is escaped,
/// and one longer than the kernel's own bound is cut to it.
#[test]
fn a_command_name_is_bounded_and_escaped() {
    let escaped: CommandWord = CommandWord::of("a\nb");
    assert_eq!(escaped.text(), "a\\x0ab");
    let long: CommandWord = CommandWord::of("abcdefghijklmnopqrst");
    assert_eq!(long.text().len(), COMMAND_MAXIMUM_BYTES);
}

/// The two errnos a read of a task that ended gives mean gone,
/// and every other refusal is a blind read.
#[test]
fn only_a_missing_task_is_gone() {
    assert_eq!(cause_of(2), None);
    assert_eq!(cause_of(3), None);
    assert_eq!(cause_of(13), Some(RefusalWord::HolderBlind));
    assert_eq!(cause_of(1), Some(RefusalWord::HolderBlind));
    assert_eq!(cause_of(0), Some(RefusalWord::HolderBlind));
}

/// A live process's record is read from a fixture tree,
/// and a number no task has is gone.
#[test]
fn a_record_reads_from_a_fixture_tree() {
    let tree: std::path::PathBuf = crate::testing::test_support::fixture_root().join("process-record-reads");
    let _removed: std::io::Result<()> = std::fs::remove_dir_all(&tree);
    let folder: std::path::PathBuf = tree.join("777");
    std::fs::create_dir_all(folder.as_path()).expect("the task's folder");
    std::fs::write(folder.join("stat"), stat_line("worker", 'R', 1, 777, 777, 5)).expect("the record");
    let ReadOutcome::Found(record) = read_process_record(tree.as_path(), 777) else {
        panic!("the fixture's record reads");
    };
    assert_eq!(record.pid, 777);
    assert_eq!(record.command.text(), "worker");
    assert_eq!(read_process_record(tree.as_path(), 778), ReadOutcome::Gone);
    std::fs::remove_dir_all(&tree).expect("the fixture is removed");
}

/// A zombie holds nothing,
/// so its record reads as a task that has gone.
#[test]
fn a_zombie_is_gone() {
    let tree: std::path::PathBuf = crate::testing::test_support::fixture_root().join("process-record-zombie");
    let _removed: std::io::Result<()> = std::fs::remove_dir_all(&tree);
    let folder: std::path::PathBuf = tree.join("9");
    std::fs::create_dir_all(folder.as_path()).expect("the task's folder");
    std::fs::write(folder.join("stat"), stat_line("zombie", 'Z', 1, 9, 9, 5)).expect("the record");
    assert_eq!(read_process_record(tree.as_path(), 9), ReadOutcome::Gone);
    let text: String = stat_line("zombie", 'Z', 1, 9, 9, 5);
    let ReadOutcome::Found(record) = parse_process_record(9, text.as_str()) else {
        panic!("the parse answers the record itself");
    };
    assert!(record.has_exited());
    std::fs::remove_dir_all(&tree).expect("the fixture is removed");
}
