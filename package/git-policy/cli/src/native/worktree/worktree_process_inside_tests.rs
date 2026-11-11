//! What:
//!  Controls for the read that says whether a process works inside a folder.
//! Why:
//!   A worktree a person is working in must never be the one an automatic removal takes,
//!  and the answer comes from the process table,
//!  which a control can write itself instead of waiting for a real process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(processWorkingInside(worktree, table)).toBe(pid);
//! ```

use super::{WorkingFolder, WorkingProcess, process_working_inside, started_before};
use std::time::Duration;
use crate::test_support::{fixture, spawn_owned};
use std::path::{Path, PathBuf};

/// What:
///  A stand-in process table with one process whose working folder is `working`.
/// Why:
///   The real table cannot be made to hold a process in a folder that a control chose,
///  so the control writes the two things the read looks at:
/// a numbered entry and its `cwd` link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableWithOneProcessIn(working, 4242); // path to the table
/// ```
fn a_table_with_one_process_in(working: &Path, pid: u32) -> PathBuf {
    let root: PathBuf = fixture(&format!("process-table-{pid}"));
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("a process entry");
    std::os::unix::fs::symlink(working, entry.join("cwd")).expect("the working folder of the process");
    return table;
}

#[test]
fn a_process_working_in_the_folder_is_named_by_its_id() {
    let root: PathBuf = fixture("process-inside");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let table: PathBuf = a_table_with_one_process_in(worktree.as_path(), 4242);
    assert_eq!(
        process_working_inside(worktree.as_path(), table.as_path(), None),
        Some(WorkingProcess { pid: 4242, folder: WorkingFolder::Inside })
    );
}

#[test]
fn a_process_in_a_folder_below_the_worktree_counts() {
    let root: PathBuf = fixture("process-below");
    let worktree: PathBuf = root.join("worktree");
    let below: PathBuf = worktree.join("package").join("cli");
    std::fs::create_dir_all(below.as_path()).expect("a folder below the worktree");
    let table: PathBuf = a_table_with_one_process_in(below.as_path(), 4343);
    assert_eq!(
        process_working_inside(worktree.as_path(), table.as_path(), None),
        Some(WorkingProcess { pid: 4343, folder: WorkingFolder::Inside })
    );
}

#[test]
fn a_process_working_elsewhere_does_not_count() {
    let root: PathBuf = fixture("process-elsewhere");
    let worktree: PathBuf = root.join("worktree");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    std::fs::create_dir_all(elsewhere.as_path()).expect("another folder");
    let table: PathBuf = a_table_with_one_process_in(elsewhere.as_path(), 4444);
    assert_eq!(process_working_inside(worktree.as_path(), table.as_path(), None), None);
    assert_eq!(
        process_working_inside(worktree.as_path(), root.join("no-such-table").as_path(), None),
        None,
        "no table, no answer"
    );
}

/// What:
///  This command's own process working in the folder keeps it,
///  and is named as this command rather than by its number.
/// Why:
///  `-C` names the repository a command works on,
/// never its own working folder,
///  so a program that spawns a wrapped merge with the worktree a landed branch holds as its working folder
///  is the only process there,
///  and skipping the command's own number removed that folder
///  (a cross-vendor audit of the landed sweep).
///  The table is written,
/// not read from the machine,
///  because a control cannot choose where its own process works.
#[test]
fn this_command_working_in_the_folder_keeps_it() {
    let root: PathBuf = fixture("process-own");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let own: u32 = std::process::id();
    let table: PathBuf = a_table_with_one_process_in(worktree.as_path(), own);
    let found: Option<WorkingProcess> = process_working_inside(worktree.as_path(), table.as_path(), None);
    assert_eq!(found, Some(WorkingProcess { pid: own, folder: WorkingFolder::Inside }), "the command itself works there");
    let why: String = found.expect("the command's own process").why();
    assert_eq!(why, "this command is working in it, because it was started in that folder");
}

#[test]
fn a_real_process_in_a_real_folder_is_found_in_the_hosts_own_table() {
    let root: PathBuf = fixture("process-real");
    let mut child: std::process::Child = spawn_owned(std::process::Command::new("sleep").arg("120").current_dir(root.as_path()))
        .expect("a helper working in the folder");
    // The folder was made a moment ago, so every process that hides its working folder started before it.
    let found: Option<WorkingProcess> = process_working_inside(root.as_path(), Path::new("/proc"), Some(Duration::ZERO));
    let _ended = child.kill();
    let _waited = child.wait();
    assert_eq!(found, Some(WorkingProcess { pid: child.id(), folder: WorkingFolder::Inside }), "the helper is named by its own id");
}

/// What:
///  A stand-in process table with one process of this account whose working folder cannot be read.
/// Why:
///   A program can hide its working folder from every other reader,
///  and a fixture cannot make a link unreadable,
///  so the control writes a `cwd` that is no link at all:
///  the read fails,
/// the entry is this account's,
///  and the answer must be the same as for a hidden folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableWithOneProcessWhoseFolderCannotBeRead(4242);
/// ```
fn a_table_with_a_process_whose_folder_cannot_be_read(pid: u32) -> PathBuf {
    let root: PathBuf = fixture(&format!("process-table-unreadable-{pid}"));
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("a process entry");
    std::fs::write(entry.join("cwd"), b"not a link, so nothing can be read from it").expect("a working folder that cannot be read");
    return table;
}

#[test]
fn a_process_of_this_account_whose_folder_cannot_be_read_keeps_the_folder() {
    let root: PathBuf = fixture("process-hidden");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let table: PathBuf = a_table_with_a_process_whose_folder_cannot_be_read(4545);
    let found: Option<WorkingProcess> = process_working_inside(worktree.as_path(), table.as_path(), None);
    assert_eq!(found, Some(WorkingProcess { pid: 4545, folder: WorkingFolder::Unreadable }), "nothing proves it is outside the folder");
    let why: String = found.expect("the process").why();
    assert!(why.contains("process 4545"), "{why}");
    assert!(why.contains("hides its working folder"), "{why}");
    assert!(why.contains("nothing proves it is outside it"), "{why}");
}

#[test]
fn a_process_that_ended_while_it_was_read_does_not_count() {
    let root: PathBuf = fixture("process-ended");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join("4646");
    std::fs::create_dir_all(entry.as_path()).expect("a process entry whose working folder went with it");
    assert_eq!(process_working_inside(worktree.as_path(), table.as_path(), None), None, "a process that ended cannot be working there");
}

#[test]
fn a_process_whose_folder_was_read_elsewhere_says_so_in_its_own_words() {
    let inside: WorkingProcess = WorkingProcess { pid: 4747, folder: WorkingFolder::Inside };
    assert_eq!(inside.why(), "process 4747 is working in it");
}

/// The start time one row of a process table names,
/// in the words `/proc/<pid>/stat` uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// statWithStartTicks(99600) // '1234 (a helper) R 0 0 ... 99600'
/// ```
fn stat_with_start_ticks(pid: u32, start_ticks: u64) -> String {
    let mut fields: Vec<String> = vec![String::from("R")];
    // Every field between the state and the start time is a number this read never looks at.
    for _unused in 0..18 {
        fields.push(String::from("0"));
    }
    fields.push(start_ticks.to_string());
    return format!("{pid} (a helper) {}\n", fields.join(" "));
}

/// What:
///  A stand-in process table with one process of this account that hides its working folder
///  and one `uptime`,
///  so the start time decides whether it could be inside.
/// Why:
///   The ages are the whole question for a hidden folder,
///  and a fixture can write both numbers instead of waiting for a process to be born.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableOfAHiddenProcessStartedAt(pid, uptimeSeconds, startTicks);
/// ```
fn a_table_of_a_hidden_process_started_at(pid: u32, uptime_seconds: u64, start_ticks: u64) -> PathBuf {
    let table: PathBuf = a_table_with_a_process_whose_folder_cannot_be_read(pid);
    std::fs::write(table.join("uptime"), format!("{uptime_seconds}.00 0.00\n")).expect("the machine's uptime");
    std::fs::write(table.join(pid.to_string()).join("stat"), stat_with_start_ticks(pid, start_ticks)).expect("the process's own row");
    return table;
}

#[test]
fn a_hidden_process_older_than_the_worktree_does_not_count() {
    let root: PathBuf = fixture("process-hidden-older");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    // Up for a thousand seconds, started at the second, so it is older than a folder made five seconds ago.
    let table: PathBuf = a_table_of_a_hidden_process_started_at(4848, 1_000, 100);
    assert_eq!(
        process_working_inside(worktree.as_path(), table.as_path(), Some(Duration::from_secs(5))),
        None,
        "a process older than the folder cannot have changed into it"
    );
}

#[test]
fn a_hidden_process_younger_than_the_worktree_keeps_it() {
    let root: PathBuf = fixture("process-hidden-younger");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    // Up for a thousand seconds and started at the 999th, so it is younger than a folder made ten seconds ago.
    let table: PathBuf = a_table_of_a_hidden_process_started_at(4949, 1_000, 99_900);
    assert_eq!(
        process_working_inside(worktree.as_path(), table.as_path(), Some(Duration::from_secs(10))),
        Some(WorkingProcess { pid: 4949, folder: WorkingFolder::Unreadable }),
        "a process that started after the folder was made may be working in it"
    );
}

#[test]
fn an_age_that_cannot_be_read_keeps_the_worktree() {
    assert!(!started_before(None, Some(Duration::from_secs(5))), "a process whose age is unknown may be inside");
    assert!(!started_before(Some(Duration::from_secs(5)), None), "a folder whose age is unknown may hold the process");
    assert!(started_before(Some(Duration::from_secs(6)), Some(Duration::from_secs(5))), "an older process is outside");
    assert!(!started_before(Some(Duration::from_secs(4)), Some(Duration::from_secs(5))), "a younger one may be inside");
}
