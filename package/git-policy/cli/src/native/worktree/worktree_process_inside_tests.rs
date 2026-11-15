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

use super::{LeftOut, ProcessWorking, WorkingFolder, WorkingProcess, process_working_inside, process_working_inside_until, started_before};
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

/// What:
///  A stand-in process table with one process whose working folder cannot be read,
///  a group the caller writes for it and for this command,
///  and the start time that decides whether it is older than the worktree.
/// Why:
///   A program that hides its folder leaves two facts a scan can read:
///  the group it runs in and when it started,
///  and this control has to hold both fixed to say which one decided the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableWithAHiddenProcess(4242, '/ours.scope', '/ours.scope/child.scope', 1000);
/// ```
fn a_table_with_a_hidden_process(pid: u32, our_group: &str, their_group: &str, started_seconds_ago: u64) -> PathBuf {
    let root: PathBuf = fixture(&format!("process-table-hidden-group-{pid}-{}", started_seconds_ago));
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("a process entry");
    std::fs::create_dir_all(table.join("self").as_path()).expect("this command's own entry");
    // A hundred ticks a second, and an uptime far enough ahead that the difference is the age written here.
    let uptime: u64 = started_seconds_ago + 100;
    std::fs::write(table.join("uptime"), format!("{uptime}.00 0.00\n")).expect("the machine's uptime");
    std::fs::write(table.join("self/cgroup"), format!("0::{our_group}\n")).expect("this command's own group");
    std::fs::write(entry.join("cgroup"), format!("0::{their_group}\n")).expect("the process's own group");
    std::fs::write(entry.join("cwd"), b"not a link, so nothing can be read from it").expect("a working folder that cannot be read");
    std::fs::write(entry.join("comm"), b"a program\n").expect("the kernel's name for the program");
    // State `S`, flags none, and the start time: field 3, field 9 and field 22 of the record.
    std::fs::write(
        entry.join("stat"),
        format!("{pid} (a program) S 1 {pid} {pid} 0 -1 0 0 0 0 0 0 0 0 20 0 1 0 {} 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n", started_seconds_ago * 100),
    )
    .expect("the process's record");
    return table;
}

/// The age a hidden process that began long before the worktree has to the scan,
/// which is the pair that used to be the whole answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// anAgeOlderThanTheWorktree() // Duration { seconds: 1000 }
/// ```
fn an_age_older_than_the_worktree() -> Duration {
    return Duration::from_secs(1000);
}

/// What:
///  A process of this account that hides its working folder,
///  began before the worktree was made,
///  and runs in this command's own control group,
///  keeps the worktree anyway.
/// Why:
///   A start time does not prove a process is outside a folder:
///  a process that began before the worktree existed can change its folder into it afterwards,
///  and the guard then removed a worktree around a live process
///  (a review of this change,
///  finding 4).
///  A process in this command's own group or below it is the work a worktree is used by,
///  and not the services a session starts with,
///  and it is the one kind of hidden process whose age decides nothing.
#[test]
fn a_hidden_process_of_this_commands_own_group_keeps_the_worktree_whatever_its_start() {
    let root: PathBuf = fixture("process-hidden-own-group");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let table: PathBuf = a_table_with_a_hidden_process(4242, "/user.slice/app.scope", "/user.slice/app.scope/child.scope", 1000);
    let found: ProcessWorking = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), Some(an_age_older_than_the_worktree()));
    assert_eq!(
        found,
        ProcessWorking::Someone(WorkingProcess { pid: 4242, program: Some(String::from("a program")), folder: WorkingFolder::Unreadable }),
        "a hidden process in this command's own group may be inside, so the worktree stays"
    );
}

/// What:
///  A hidden process that began before the worktree and runs in another group does not keep it,
///  and this is the cost the module's description names.
/// Why:
///   Every removal on a machine whose session starts services that hide their folders
///  would otherwise be refused,
///  measured on this host at eleven such processes,
///  so the scan takes the group as the line:
///  a process of this command's work may be inside, a service of the session is taken as outside.
///  A control states that answer plainly,
///  because a reader of this code has to know which process it is wrong about.
#[test]
fn a_hidden_process_of_another_group_and_an_older_start_is_taken_as_outside() {
    let root: PathBuf = fixture("process-hidden-other-group");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let table: PathBuf = a_table_with_a_hidden_process(4343, "/user.slice/app.scope", "/user.slice/user@1000.service/init.scope", 1000);
    let found: ProcessWorking = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), Some(an_age_older_than_the_worktree()));
    assert_eq!(found, ProcessWorking::Nobody, "a session service that began before the worktree is taken as outside it");
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
    let found: ProcessWorking = process_working_inside_until(worktree.as_path(), &worktree.join(".git"), table.as_path(), None, Instant::now(), &LeftOut::none());
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
    // The kernel writes the path it resolved into `/proc/<pid>/fd`, so both the scan and this control compare the
    //  canonical spelling: a fixture root reached through a link is the same directory under another name.
    let root: PathBuf = std::fs::canonicalize(fixture("process-real-open-file").as_path()).expect("the fixture's real path");
    let held: PathBuf = root.join("held.txt");
    std::fs::write(held.as_path(), b"the file the helper holds\n").expect("the file the helper holds open");
    let elsewhere: PathBuf = std::fs::canonicalize(fixture("process-real-open-file-elsewhere").as_path()).expect("the helper's real path");
    // `tee` opens its argument for writing and keeps it open while it copies its input, which is held open here.
    let mut child: std::process::Child = spawn_owned(
        std::process::Command::new("tee")
            .arg(held.as_path())
            .current_dir(elsewhere.as_path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null()),
    )
    .expect("a helper holding a file in the folder");
    // The start returns before the helper has opened its argument: measured on 2026-11-14, the descriptor was missing at
    //  that instant in 59 of 60 rounds and appeared 0.20 to 0.35 ms later, which is the window this control scanned in when
    //  the container gate of 0.6.1-alpha.35 failed it. The control holds the helper at its open before it scans, as the
    //  control above holds its helper at the working folder.
    let start: std::time::Instant = std::time::Instant::now();
    while !descriptors_name(child.id(), held.as_path()) {
        assert!(start.elapsed() < Duration::from_secs(10), "the helper never opened {}", held.display());
        std::thread::sleep(Duration::from_millis(5));
    }
    // The folder was made a moment ago, so every process that hides its working folder started before it.
    let found: ProcessWorking = process_working_inside(root.as_path(), &root.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    let _ended = child.kill();
    let _waited = child.wait();
    assert_eq!(found, someone(child.id(), "tee", WorkingFolder::OpenFile), "the helper's own descriptor is named");
    std::fs::remove_dir_all(elsewhere.as_path()).expect("the helper's own folder");
}

/// What:
///  Whether one process holds this file open,
///  read from its descriptor links.
/// Why:
///  A control that starts a helper must not scan before the helper has opened what it is there to hold:
///  the start returns before the helper's own `open`, and a scan in that window finds nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const open = descriptorsName(pid, held) // readdirSync(`/proc/${pid}/fd`).some((fd) => readlinkSync(...) === held)
/// ```
fn descriptors_name(pid: u32, file: &Path) -> bool {
    let Ok(descriptors) = std::fs::read_dir(format!("/proc/{pid}/fd")) else {
        return false;
    };
    for descriptor in descriptors.flatten() {
        if std::fs::read_link(descriptor.path()).ok().as_deref() == Some(file) {
            return true;
        }
    }
    return false;
}

/// What:
///  A stand-in process table holding `rows`:
///  each `(pid, ppid, start ticks, working folder)`.
/// Why:
///  The chain of parents is read from the same table the scan reads,
///  so a control writes the chain it wants instead of arranging real parents.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// a_table(name, [[4242, 4343, 100, worktree], [4343, 4444, 90, worktree]])
/// ```
fn a_table_of_rows(name: &str, rows: &[(u32, u32, u64, &Path)]) -> PathBuf {
    let root: PathBuf = fixture(name);
    let table: PathBuf = root.join("proc");
    std::fs::create_dir_all(table.as_path()).expect("the process table");
    for (pid, parent, started, working) in rows {
        let entry: PathBuf = table.join(pid.to_string());
        std::fs::create_dir_all(entry.as_path()).expect("a process entry");
        std::os::unix::fs::symlink(working, entry.join("cwd")).expect("the working folder of the process");
        let mut fields: Vec<String> = vec![String::from("S"), parent.to_string()];
        // The fields between the parent's number and the start time are numbers this read never looks at: the start is
        //  the twentieth after the program's name.
        for _unused in 0..17 {
            fields.push(String::from("0"));
        }
        fields.push(started.to_string());
        std::fs::write(entry.join("stat"), format!("{pid} (a helper) {}\n", fields.join(" "))).expect("the process's own row");
        std::fs::write(entry.join("comm"), b"a helper\n").expect("the program the process runs");
    }
    std::fs::write(table.join("uptime"), b"100000.00 0.00\n").expect("the machine's uptime");
    return table;
}

/// What:
///  This command and its parents are left out of the scan,
///  and a stranger that works in the folder still keeps it.
/// Why:
///  The owner's decision of 2026-11-14: a removal leaves out the command itself and each of its parents.
///  Read with that set the scan finds nobody though every one of those stands in the folder,
///  and without it the same table still reports the command's own process,
///  which is what a caller that reads its own process like any other gets (the automatic removal).
#[test]
fn this_command_and_its_parents_are_left_out_of_the_scan() {
    let root: PathBuf = fixture("process-chain-left-out");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let own: u32 = std::process::id();
    // This command, its parent and its grandparent all stand in the folder, and a stranger does too.
    let table: PathBuf = a_table_of_rows(
        "process-chain-left-out-table",
        &[
            (own, 4343, 100, worktree.as_path()),
            (4343, 4444, 90, worktree.as_path()),
            (4444, 1, 80, worktree.as_path()),
            (4545, 1, 70, worktree.as_path()),
        ],
    );
    let left_out: LeftOut = LeftOut::of_this_command(table.as_path(), Instant::now() + std::time::Duration::from_secs(2));
    let found: ProcessWorking = process_working_inside_until(
        worktree.as_path(),
        &worktree.join(".git"),
        table.as_path(),
        Some(Duration::from_secs(1)),
        Instant::now() + std::time::Duration::from_secs(2),
        &left_out,
    );
    assert_eq!(found, someone(4545, "a helper", WorkingFolder::Inside), "only the stranger that is not a parent keeps it: {found:?}");
    let without: ProcessWorking = process_working_inside_until(
        worktree.as_path(),
        &worktree.join(".git"),
        table.as_path(),
        Some(Duration::from_secs(1)),
        Instant::now() + std::time::Duration::from_secs(2),
        &LeftOut::none(),
    );
    let ProcessWorking::Someone(working) = &without else { panic!("the command's own process is read when nothing is left out: {without:?}") };
    assert_eq!(working.pid, own, "and it is this command");
}

/// What:
///  A parent number taken again after the real parent ended ends the chain,
///  and the stranger that holds the number keeps the folder.
/// Why:
///  A parent starts before its child;
///  a record that starts after the child's cannot be that child's parent,
///  so leaving it out would leave out a process that may work anywhere.
#[test]
fn a_parent_number_taken_again_ends_the_chain() {
    let root: PathBuf = fixture("process-chain-reused");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let own: u32 = std::process::id();
    // The number the command's record names as its parent belongs to a process that started after the command.
    let table: PathBuf = a_table_of_rows(
        "process-chain-reused-table",
        &[(own, 4343, 100, worktree.as_path()), (4343, 1, 200, worktree.as_path())],
    );
    let left_out: LeftOut = LeftOut::of_this_command(table.as_path(), Instant::now() + std::time::Duration::from_secs(2));
    assert!(!left_out.holds(4343), "a number whose record starts after the command's is no parent of it");
    let found: ProcessWorking = process_working_inside_until(
        worktree.as_path(),
        &worktree.join(".git"),
        table.as_path(),
        Some(Duration::from_secs(1)),
        Instant::now() + std::time::Duration::from_secs(2),
        &left_out,
    );
    assert_eq!(found, someone(4343, "a helper", WorkingFolder::Inside), "the process holding that number keeps the folder: {found:?}");
}

/// What:
///  A parent whose own record cannot be read is left out,
///  and the chain ends there.
/// Why:
///  The child's record named it as its parent,
///  so it is one of the command's own chain;
///  its own record is what would name the next ancestor,
///  and without it the chain cannot go on.
#[test]
fn a_parent_whose_record_cannot_be_read_is_left_out_and_ends_the_chain() {
    let root: PathBuf = fixture("process-chain-unreadable-parent");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let own: u32 = std::process::id();
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join(own.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("this command's entry");
    std::fs::write(entry.join("stat"), format!("{own} (a helper) S 4343 0 0 0\n")).expect("a record with a parent and no start");
    std::fs::write(entry.join("comm"), b"a helper\n").expect("the program this command runs");
    let parent: PathBuf = table.join("4343");
    std::fs::create_dir_all(parent.as_path()).expect("the parent's entry");
    std::os::unix::fs::symlink(worktree.as_path(), parent.join("cwd")).expect("its working folder");
    std::fs::write(parent.join("comm"), b"a helper\n").expect("the program the parent runs");
    std::fs::write(table.join("uptime"), b"100000.00 0.00\n").expect("the machine's uptime");
    let left_out: LeftOut = LeftOut::of_this_command(table.as_path(), Instant::now() + std::time::Duration::from_secs(2));
    assert!(left_out.holds(4343), "the parent the command's record names stays left out");
    assert!(left_out.holds(own), "and so does the command");
}
