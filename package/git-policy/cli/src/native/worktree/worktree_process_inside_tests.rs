//! What:
//!  Controls for the read that says whether a process works inside a folder.
//! Why:
//!   A worktree a person is working in must never be the one an automatic removal takes,
//!  and the answer comes from the process table,
//!  which a control can write itself instead of waiting for a real process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(processWorkingInside(worktree, table)).toEqual({ kind: 'someone', process: { pid } });
//! ```

use super::{ProcessWorking, WorkingFolder, WorkingProcess, process_working_inside, process_working_inside_until, started_before};
use std::time::{Duration, Instant};
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

/// What:
///  One process that keeps the worktree,
///  with the program name the control wrote beside it.
/// Why:
///   Every answer this read gives names the process and its program,
///  so the expected value is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// someone(4242, 'sleep', WorkingFolder.Inside)
/// ```
fn someone(pid: u32, program: &str, folder: WorkingFolder) -> ProcessWorking {
    return ProcessWorking::Someone(WorkingProcess { pid, program: Some(String::from(program)), folder });
}

#[test]
fn a_process_working_in_the_folder_is_named_by_its_id() {
    let root: PathBuf = fixture("process-inside");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let table: PathBuf = a_table_with_one_process_in(worktree.as_path(), 4242);
    std::fs::write(table.join("4242").join("comm"), b"sleep\n").expect("the program the process runs");
    assert_eq!(process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None), someone(4242, "sleep", WorkingFolder::Inside));
}

#[test]
fn a_process_in_a_folder_below_the_worktree_counts() {
    let root: PathBuf = fixture("process-below");
    let worktree: PathBuf = root.join("worktree");
    let below: PathBuf = worktree.join("package").join("cli");
    std::fs::create_dir_all(below.as_path()).expect("a folder below the worktree");
    let table: PathBuf = a_table_with_one_process_in(below.as_path(), 4343);
    std::fs::write(table.join("4343").join("comm"), b"zsh\n").expect("the program the process runs");
    assert_eq!(process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None), someone(4343, "zsh", WorkingFolder::Inside));
}

#[test]
fn a_process_working_elsewhere_does_not_count() {
    let root: PathBuf = fixture("process-elsewhere");
    let worktree: PathBuf = root.join("worktree");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    std::fs::create_dir_all(elsewhere.as_path()).expect("another folder");
    let table: PathBuf = a_table_with_one_process_in(elsewhere.as_path(), 4444);
    assert_eq!(process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None), ProcessWorking::Nobody);
    assert_eq!(
        process_working_inside(worktree.as_path(), &worktree.join(".git"), root.join("no-such-table").as_path(), None),
        ProcessWorking::Unfinished(format!(
            "the process table {:?} cannot be read (No such file or directory (os error 2)), so nothing proves no process works there",
            root.join("no-such-table")
        )),
        "no table proves nothing, so it is never read as an empty one"
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
    let found: ProcessWorking = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    let ProcessWorking::Someone(own_process) = &found else {
        panic!("the command itself works there: {found:?}");
    };
    assert_eq!(own_process.pid, own, "the command itself works there");
    assert_eq!(found.reason(), Some(String::from("this command is working in it, because it was started in that folder")));
}

#[test]
fn a_real_process_in_a_real_folder_is_found_in_the_hosts_own_table() {
    let root: PathBuf = fixture("process-real");
    let mut child: std::process::Child = spawn_owned(std::process::Command::new("sleep").arg("120").current_dir(root.as_path()))
        .expect("a helper working in the folder");
    // The folder was made a moment ago, so every process that hides its working folder started before it.
    let found: ProcessWorking = process_working_inside(root.as_path(), &root.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    let _ended = child.kill();
    let _waited = child.wait();
    let ProcessWorking::Someone(working) = &found else {
        panic!("the helper is found: {found:?}");
    };
    assert_eq!(working.pid, child.id(), "the helper is named by its own id");
    assert_eq!(working.folder, WorkingFolder::Inside, "its working folder is the fixture root");
    assert_eq!(working.program.as_deref(), Some("sleep"), "the kernel's name for the program is read");
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
    let found: ProcessWorking = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    assert_eq!(
        found,
        ProcessWorking::Someone(WorkingProcess { pid: 4545, program: None, folder: WorkingFolder::Unreadable }),
        "nothing proves it is outside the folder"
    );
    let why: String = found.reason().expect("the process keeps the folder");
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
    assert_eq!(
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None),
        ProcessWorking::Nobody,
        "a process that ended cannot be working there"
    );
}

#[test]
fn a_process_whose_folder_was_read_elsewhere_says_so_in_its_own_words() {
    let inside: WorkingProcess = WorkingProcess { pid: 4747, program: Some(String::from("sleep")), folder: WorkingFolder::Inside };
    assert_eq!(inside.why(), "process 4747 (`sleep`) is working in it");
    let unnamed: WorkingProcess = WorkingProcess { pid: 4748, program: None, folder: WorkingFolder::Inside };
    assert_eq!(unnamed.why(), "process 4748 is working in it");
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
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), Some(Duration::from_secs(5))),
        ProcessWorking::Nobody,
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
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), Some(Duration::from_secs(10))),
        ProcessWorking::Someone(WorkingProcess { pid: 4949, program: None, folder: WorkingFolder::Unreadable }),
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

/// What:
///  A stand-in process table with one process whose working folder is elsewhere
///  and whose one open file is `held`.
/// Why:
///   A descriptor is what a program that opened a file in the worktree and went on holds,
///  and a fixture writes the link the kernel would write instead of starting such a program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableWithOneProcessHolding(elsewhere, 'wt/work.txt', 4242);
/// ```
fn a_table_with_one_process_holding(elsewhere: &Path, held: &Path, pid: u32) -> PathBuf {
    let table: PathBuf = a_table_with_one_process_in(elsewhere, pid);
    let descriptors: PathBuf = table.join(pid.to_string()).join("fd");
    std::fs::create_dir_all(descriptors.as_path()).expect("a descriptor directory");
    std::os::unix::fs::symlink(held, descriptors.join("3")).expect("the file the process holds open");
    return table;
}

/// What:
///  A process whose working folder is elsewhere and that holds a file open inside the worktree keeps it.
/// Why:
///   An editor,
///  a pager or a build started with `-C` from another directory works in a worktree without a working folder in it,
///  and removing the worktree pulls the file out from under it:
///  the working-folder read alone removed it
///  (a qwen4-max-preview audit of 0.6.1-alpha.10,
/// `574063773d`,
///  finding 2,
///  reproduced with a `tail -f` of a file in the worktree whose working folder was the fixture root).
#[test]
fn a_process_holding_a_file_inside_keeps_the_worktree_though_its_folder_is_elsewhere() {
    let root: PathBuf = fixture("process-open-file");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("a folder the process was started in");
    let held: PathBuf = worktree.join("work.txt");
    std::fs::write(held.as_path(), b"the file the process holds\n").expect("the file the process holds open");
    let table: PathBuf = a_table_with_one_process_holding(elsewhere.as_path(), held.as_path(), 5151);
    std::fs::write(table.join("5151").join("comm"), b"tee\n").expect("the program the process runs");
    let found: ProcessWorking = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    assert_eq!(found, someone(5151, "tee", WorkingFolder::OpenFile), "the descriptor into the worktree is found");
    assert_eq!(
        found.reason(),
        Some(String::from("process 5151 (`tee`) has a file open in it, though its working folder is elsewhere"))
    );
}

/// What:
///  A descriptor into the worktree's own Git directory does not keep it.
/// Why:
///   The wrapper's commit service holds its marker file inside its worktree's Git directory for as long as it lives
///  (measured on this host:
/// `.../.git/index.lock.cli-git-marker-<pid>...`),
///  so counting those descriptors would keep every landed worktree whose service is up,
///  and the sweep would remove nothing.
#[test]
fn a_descriptor_into_the_worktrees_own_git_directory_does_not_count() {
    let root: PathBuf = fixture("process-open-file-git-dir");
    let worktree: PathBuf = root.join("worktree");
    let git_dir: PathBuf = worktree.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the worktree's own Git directory");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("a folder the process was started in");
    let marker: PathBuf = git_dir.join("index.lock.cli-git-marker-5151");
    std::fs::write(marker.as_path(), b"the service's own marker\n").expect("the marker a service holds");
    let table: PathBuf = a_table_with_one_process_holding(elsewhere.as_path(), marker.as_path(), 5151);
    assert_eq!(
        process_working_inside(worktree.as_path(), git_dir.as_path(), table.as_path(), None),
        ProcessWorking::Nobody,
        "a service's own marker is not a person working in the worktree"
    );
}

/// What:
///  A descriptor whose file is already unlinked still keeps the worktree it was in.
/// Why:
///   The kernel names such a target with ` (deleted)` after it,
///  and a removal would take the directory the file was in while the program still reads it.
#[test]
fn a_descriptor_whose_file_is_already_unlinked_counts() {
    let root: PathBuf = fixture("process-open-file-deleted");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("a folder the process was started in");
    let gone: PathBuf = worktree.join("gone.txt");
    std::fs::write(gone.as_path(), b"the file the process holds\n").expect("the file the process holds open");
    let named: String = format!("{} (deleted)", gone.display());
    let table: PathBuf = a_table_with_one_process_holding(elsewhere.as_path(), Path::new(named.as_str()), 5252);
    std::fs::write(table.join("5252").join("comm"), b"less\n").expect("the program the process runs");
    assert_eq!(
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None),
        someone(5252, "less", WorkingFolder::OpenFile),
        "the text the kernel adds after an unlinked target does not hide it"
    );
}

/// What:
///  A process that runs a program from the worktree keeps it, though its working folder is elsewhere.
/// Why:
///  A program started by absolute path from another folder is the ordinary way a tool drives a build,
///  and deleting the image under it ends that program:
///  neither the working folder nor a descriptor shows it.
#[test]
fn a_process_running_a_program_from_the_worktree_keeps_it() {
    let root: PathBuf = fixture("process-executable");
    let worktree: PathBuf = root.join("worktree");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(worktree.join("target/debug").as_path()).expect("a program in the worktree");
    std::fs::create_dir(elsewhere.as_path()).expect("a folder the process was started in");
    let program: PathBuf = worktree.join("target/debug/helper");
    std::fs::write(program.as_path(), b"not run by this control\n").expect("the program the process runs");
    let table: PathBuf = a_table_with_one_process_in(elsewhere.as_path(), 5353);
    std::os::unix::fs::symlink(program.as_path(), table.join("5353").join("exe")).expect("the program the process runs");
    std::fs::write(table.join("5353").join("comm"), b"helper\n").expect("the program's own name");
    let found: ProcessWorking = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    assert_eq!(found, someone(5353, "helper", WorkingFolder::Executable), "the program in the worktree is found");
    assert_eq!(
        found.reason(),
        Some(String::from("process 5353 (`helper`) runs a program from it, though its working folder is elsewhere"))
    );
}

/// What:
///  A process running a program outside the worktree does not count.
/// Why:
///   Almost every program on the machine runs from somewhere,
///  and none of them is working in this folder for it.
#[test]
fn a_process_running_a_program_from_elsewhere_does_not_count() {
    let root: PathBuf = fixture("process-executable-elsewhere");
    let worktree: PathBuf = root.join("worktree");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    std::fs::create_dir(elsewhere.as_path()).expect("a folder with a program in it");
    let program: PathBuf = elsewhere.join("helper");
    std::fs::write(program.as_path(), b"not run by this control\n").expect("the program the process runs");
    let table: PathBuf = a_table_with_one_process_in(elsewhere.as_path(), 5454);
    std::os::unix::fs::symlink(program.as_path(), table.join("5454").join("exe")).expect("the program the process runs");
    assert_eq!(process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None), ProcessWorking::Nobody);
}

/// What:
///  A scan whose moment has passed says so and never answers that nobody is there.
/// Why:
///   Nothing a bound cut short proves the folder empty,
///  and a caller that removed on it would take the folder an unreached process is working in.
///  The moment is given as one already past,
///  which is what a table of thousands of entries under load reaches.
#[test]
fn a_scan_that_reached_its_bound_answers_that_it_did_not_finish() {
    let root: PathBuf = fixture("process-bound");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("a folder no process works in the worktree from");
    let table: PathBuf = a_table_with_one_process_in(elsewhere.as_path(), 5555);
    let found: ProcessWorking = process_working_inside_until(worktree.as_path(), &worktree.join(".git"), table.as_path(), None, Instant::now());
    let ProcessWorking::Unfinished(why) = &found else {
        panic!("a scan past its moment did not finish: {found:?}");
    };
    assert!(why.contains("stopped at its bound of 2 seconds"), "{why}");
    assert!(why.starts_with("the scan of the process table "), "{why}");
    assert_eq!(found.reason(), Some(why.clone()), "the reason a caller reads is the scan's own words");
}

/// What:
///  A real process that holds a file open in a folder and was started elsewhere is found in the host's own table.
/// Why:
///  The stand-in table writes the links this control assumes;
///  only a real child proves the kernel writes them that way,
///  which is what the sweep reads on a person's machine.
#[test]
fn a_real_process_holding_a_file_open_is_found_in_the_hosts_own_table() {
    let root: PathBuf = fixture("process-real-open-file");
    let held: PathBuf = root.join("held.txt");
    std::fs::write(held.as_path(), b"the file the helper holds\n").expect("the file the helper holds open");
    let elsewhere: PathBuf = fixture("process-real-open-file-elsewhere");
    // `tee` opens its argument for writing and keeps it open while it copies its input, which is held open here.
    let mut child: std::process::Child = spawn_owned(
        std::process::Command::new("tee")
            .arg(held.as_path())
            .current_dir(elsewhere.as_path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null()),
    )
    .expect("a helper holding a file in the folder");
    // The folder was made a moment ago, so every process that hides its working folder started before it.
    let found: ProcessWorking = process_working_inside(root.as_path(), &root.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    let _ended = child.kill();
    let _waited = child.wait();
    assert_eq!(found, someone(child.id(), "tee", WorkingFolder::OpenFile), "the helper's own descriptor is named");
    std::fs::remove_dir_all(elsewhere.as_path()).expect("the helper's own folder");
}
