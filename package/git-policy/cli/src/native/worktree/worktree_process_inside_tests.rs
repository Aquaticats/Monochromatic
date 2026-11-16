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

use super::{ProcessLook, WorkingFolder, WorkingProcess, process_working_inside, started_before};
use std::time::Duration;
use crate::test_support::{fixture, spawn_owned};
use std::path::{Path, PathBuf};

/// What a read answers when it found a process,
/// as a control compares it.
/// Why:
///  The three answers are told apart,
///  so a control that means "one process works there" never passes on a read that could not be made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// working(4242, 'inside') // { working: { pid: 4242, folder: 'inside' } }
/// ```
fn working(pid: u32, folder: WorkingFolder) -> ProcessLook {
    return ProcessLook::Working(WorkingProcess { pid, folder });
}

/// What:
///  The process a read named,
///  or a failure that names what the read answered instead.
/// Why:
///  A control that reads the reason needs the process,
///  and an answer that found none has to fail here rather than in the words it compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theProcess(look) // look.working, or a failure naming look
/// ```
fn the_process(look: ProcessLook) -> WorkingProcess {
    return match look {
        ProcessLook::Working(found) => found,
        other => panic!("a process works there, and the read answered {other:?}"),
    };
}

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
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None),
        working(4242, WorkingFolder::Inside)
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
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None),
        working(4343, WorkingFolder::Inside)
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
    assert_eq!(
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None),
        ProcessLook::NothingWorksThere,
        "every process of the table is elsewhere"
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
    let found: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    assert_eq!(found, working(own, WorkingFolder::Inside), "the command itself works there");
    let why: String = the_process(found).why();
    assert_eq!(why, "this command is working in it, because it was started in that folder");
}

#[test]
fn a_real_process_in_a_real_folder_is_found_in_the_hosts_own_table() {
    let root: PathBuf = fixture("process-real");
    let mut child: std::process::Child = spawn_owned(std::process::Command::new("sleep").arg("120").current_dir(root.as_path()))
        .expect("a helper working in the folder");
    // The folder was made a moment ago, so every process that hides its working folder started before it.
    let found: ProcessLook = process_working_inside(root.as_path(), &root.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    let _ended = child.kill();
    let _waited = child.wait();
    assert_eq!(found, working(child.id(), WorkingFolder::Inside), "the helper is named by its own id");
}

/// What:
///  The same table,
/// saying the machine was up for `uptime_seconds` and the process started at `start_ticks`.
/// Why:
///   The ages are the whole question for whatever the read could not see,
///  and a fixture writes both numbers instead of waiting for a process to be born.
fn saying_it_started(table: PathBuf, pid: u32, uptime_seconds: u64, start_ticks: u64) -> PathBuf {
    std::fs::write(table.join("uptime"), format!("{uptime_seconds}.00 0.00\n")).expect("the machine's uptime");
    std::fs::write(table.join(pid.to_string()).join("stat"), stat_with_start_ticks(pid, start_ticks)).expect("the process's own row");
    return table;
}

/// What:
///  A stand-in process table with one process whose working folder is elsewhere
///  and whose descriptor table this account may not read.
/// Why:
///   A program can stop every other reader seeing what it holds open while it runs,
///  and no fixture can make a real process do that,
///  so the control writes a descriptor directory with no permission to read it:
///  the kernel refuses that read for a process that is there,
///  which is an answer the read must never turn into none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = aTableWithOneProcessWhoseDescriptorsCannotBeRead(elsewhere, 4242);
/// ```
fn a_table_with_one_process_whose_descriptors_cannot_be_read(elsewhere: &Path, pid: u32) -> PathBuf {
    use std::os::unix::fs::PermissionsExt as _;
    let table: PathBuf = a_table_with_one_process_in(elsewhere, pid);
    let descriptors: PathBuf = table.join(pid.to_string()).join("fd");
    std::fs::create_dir_all(descriptors.as_path()).expect("a descriptor directory");
    std::fs::set_permissions(descriptors.as_path(), std::fs::Permissions::from_mode(0o000)).expect("a descriptor directory this account may not read");
    return table;
}

/// What:
///  A process younger than the worktree whose descriptor table cannot be read keeps the worktree,
///  and one older than it does not.
/// Why:
///   A refused look at the files a program holds is never none,
///  so the read answers that the process may hold a file there;
///  and that answer is bounded the way a hidden working folder is:
///  a process that started before the worktree was made is taken to be outside it,
///  though one whose working folder stays elsewhere,
///  and that opens a file in it afterwards,
///  is the miss this rule leaves.
///  The bound is what keeps the walk from stopping at every program that hides its own memory,
///  a password manager that runs from login above all:
///  measured on this host,
///  its working folder read as the account's home folder while its descriptor table was refused,
///  and an unbounded answer kept every worktree a walk looked at.
#[test]
fn a_process_whose_descriptor_table_cannot_be_read_keeps_the_worktree_only_before_the_worktree_is_older_than_it() {
    let root: PathBuf = fixture("process-descriptors-hidden");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("a folder the process was started in");
    let pid: u32 = 5353;
    let described: PathBuf = a_table_with_one_process_whose_descriptors_cannot_be_read(elsewhere.as_path(), pid);
    // Up for a thousand seconds and started at the 99 900th tick, so the process is a second old: younger than a
    // folder made ten seconds ago, and older than one made half a second ago.
    let table: PathBuf = saying_it_started(described, pid, 1_000, 99_900);
    let younger: ProcessLook =
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), Some(Duration::from_secs(10)));
    let older: ProcessLook =
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), Some(Duration::from_millis(500)));
    // The permission is given back before the fixture goes, which needs to list the directory it holds.
    use std::os::unix::fs::PermissionsExt as _;
    let descriptors: PathBuf = table.join(pid.to_string()).join("fd");
    std::fs::set_permissions(descriptors.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the descriptor directory is readable again");
    assert_eq!(
        younger,
        working(pid, WorkingFolder::OpenFilesUnreadable),
        "a refused read of the files a process holds is never none"
    );
    assert_eq!(
        the_process(younger).why(),
        "process 5353 belongs to this account, works elsewhere, and the files it holds open could not be read, and nothing shows it started before this worktree was made, so nothing proves it holds none in it"
    );
    assert_eq!(
        older,
        ProcessLook::NothingWorksThere,
        "a process older than the worktree is taken to be outside it, though one whose working folder stays elsewhere and that opens a file in it afterwards is the miss this rule leaves"
    );
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
    let found: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    assert_eq!(found, working(4545, WorkingFolder::Unreadable), "nothing proves it is outside the folder");
    let why: String = the_process(found).why();
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
        ProcessLook::NothingWorksThere,
        "a process that ended cannot be working there"
    );
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
    return saying_it_started(a_table_with_a_process_whose_folder_cannot_be_read(pid), pid, uptime_seconds, start_ticks);
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
        ProcessLook::NothingWorksThere,
        "a process older than the folder is taken to be outside it, though one that changed into it afterwards is the miss this rule leaves"
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
        working(4949, WorkingFolder::Unreadable),
        "a process that started after the folder was made may be working in it"
    );
}

#[test]
fn an_age_that_cannot_be_read_keeps_the_worktree() {
    assert!(!started_before(None, Some(Duration::from_secs(5))), "a process whose age is unknown may be inside");
    assert!(!started_before(Some(Duration::from_secs(5)), None), "a folder whose age is unknown may hold the process");
    assert!(started_before(Some(Duration::from_secs(6)), Some(Duration::from_secs(5))), "an older process is taken to be outside");
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
    let found: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    assert_eq!(found, working(5151, WorkingFolder::OpenFile), "the descriptor into the worktree is found");
    assert_eq!(
        the_process(found).why(),
        "process 5151 has a file open in it, though its working folder is elsewhere"
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
        ProcessLook::NothingWorksThere,
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
    assert_eq!(
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None),
        working(5252, WorkingFolder::OpenFile),
        "the text the kernel adds after an unlinked target does not hide it"
    );
}

/// What:
///  A shell started in `elsewhere` that holds `held` open on descriptor 3 and says so once it does.
/// Why:
///   A descriptor is the kernel's own record of what a program holds,
///  and only a real process proves the reading finds what the kernel writes there.
///  The shell says it holds the file by making `said`,
///  which happens only after the redirect that opens `held`:
///  a shell that has not reached its redirect is a process holding nothing,
///  so the control waits for that file instead of sleeping.
///  Its input is a pipe this control keeps open,
///  so the `read` after the marker waits until the control ends the helper,
///  and descriptor 3 stays open for every walk taken meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const helper = startAHolder({ script, held, said, elsewhere });
/// ```
fn start_a_holder(script: &str, held: &Path, said: &Path, elsewhere: &Path) -> std::process::Child {
    return spawn_owned(
        std::process::Command::new("sh")
            .args(["-c", script, held.to_str().expect("a UTF-8 file path"), said.to_str().expect("a UTF-8 marker path")])
            .current_dir(elsewhere)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null()),
    )
    .expect("a helper holding a file in the folder");
}

/// What:
///  A real process that holds a file open in a folder and was started elsewhere is found in the host's own table.
/// Why:
///  The stand-in table writes the links this control assumes;
///  only a real child proves the kernel writes them that way,
///  which is what the sweep reads on a person's machine.
///  The walk waits for the helper's own word that the file is open:
///  a walk taken straight after the start reads a table in which the helper holds nothing yet,
///  which is how this control failed in the gate on `0.6.1-alpha.38`,
///  answering `left: None` for a helper that was alive and wrote nothing to its standard error.
#[test]
fn a_real_process_holding_a_file_open_is_found_in_the_hosts_own_table() {
    let root: PathBuf = fixture("process-real-open-file");
    let elsewhere: PathBuf = fixture("process-real-open-file-elsewhere");
    let held: PathBuf = root.join("held.txt");
    std::fs::write(held.as_path(), b"the file the helper holds\n").expect("the file the helper holds open");
    // The helper opens `held` on descriptor 3 and then makes this file, which is the word the wait below is for.
    let said: PathBuf = elsewhere.join("it-holds-the-file");
    let mut helper: std::process::Child =
        start_a_holder("exec 3<\"$0\"; : > \"$1\"; read x", held.as_path(), said.as_path(), elsewhere.as_path());
    assert!(crate::test_support::wait_until(&|| { return said.exists(); }), "the helper says it holds the file");
    // The folder was made a moment ago, so every process that hides its working folder started before it.
    let found: ProcessLook = process_working_inside(root.as_path(), &root.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    let _ended = helper.kill();
    let _waited = helper.wait();
    assert_eq!(found, working(helper.id(), WorkingFolder::OpenFile), "the helper's own descriptor is named");
}

/// What:
///  The two orders the gate's failure falls between:
///  a walk taken before the helper opens the file names it for nothing,
///  and a walk taken after the helper said it holds the file names it.
/// Why:
///  The control above failed in two gate runs by reading the table too early,
///  and neither the reading nor the control is shown right by repeating a race:
///  this control makes each order happen once,
///  so the order the table answers nothing in is seen every time this runs,
///  and the answer the other order earns is seen in the same run.
///  The helper opens nothing until this control says so on the helper's own input,
///  so no sleep and no luck decides which order the first walk meets.
#[test]
fn a_walk_before_the_helper_opens_the_file_names_nothing_and_a_walk_after_it_names_the_helper() {
    let root: PathBuf = fixture("process-real-open-file-orders");
    let elsewhere: PathBuf = fixture("process-real-open-file-orders-elsewhere");
    let held: PathBuf = root.join("held.txt");
    std::fs::write(held.as_path(), b"the file the helper holds\n").expect("the file the helper holds open");
    let said: PathBuf = elsewhere.join("it-holds-the-file");
    let mut helper: std::process::Child =
        start_a_holder("read go || exit 1; exec 3<\"$0\"; : > \"$1\"; read x", held.as_path(), said.as_path(), elsewhere.as_path());
    let before: ProcessLook = process_working_inside(root.as_path(), &root.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    assert!(
        !matches!(before, ProcessLook::Working(working) if working.pid == helper.id()),
        "the file is not open yet, so nothing the table says names the helper: {before:?}"
    );
    {
        use std::io::Write as _;
        let go: &mut std::process::ChildStdin = helper.stdin.as_mut().expect("the helper's own input");
        go.write_all(b"go\n").expect("the helper is told to open the file");
        go.flush().expect("the word reaches the helper");
    }
    assert!(crate::test_support::wait_until(&|| { return said.exists(); }), "the helper says it holds the file");
    let after: ProcessLook = process_working_inside(root.as_path(), &root.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    let _ended = helper.kill();
    let _waited = helper.wait();
    assert_eq!(
        after,
        working(helper.id(), WorkingFolder::OpenFile),
        "the same walk names the helper once it holds the file"
    );
}

/// What:
///  A table with nothing in it answers that nothing works there,
///  which is the one answer a removal may take as its proof.
/// Why:
///  Every control here about a table that could not be read needs this one beside it:
///  a read that answered "could not look" for every table would pass them all and remove nothing.
#[test]
fn a_table_with_nothing_in_it_answers_that_nothing_works_there() {
    let root: PathBuf = fixture("process-table-empty");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let table: PathBuf = root.join("proc");
    std::fs::create_dir(table.as_path()).expect("an empty process table");
    assert_eq!(
        process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None),
        ProcessLook::NothingWorksThere,
        "a table that was read to its end with nothing in it is the proof a removal needs"
    );
}

/// What:
///  A table that is not there is never a table with nothing in it.
/// Why:
///  A machine whose process table cannot be found hides every process it holds,
///  and answering "nothing works there" for it removed a worktree under a process working in it.
#[test]
fn a_table_that_is_not_there_is_never_nothing_works_there() {
    let root: PathBuf = fixture("process-table-absent");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let table: PathBuf = root.join("no-such-table");
    let answered: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    let ProcessLook::CannotLook(reason) = answered else {
        panic!("a table that is not there answered {answered:?}");
    };
    assert!(reason.contains(table.display().to_string().as_str()), "the reason names the table: {reason}");
    assert!(reason.contains("could not be listed"), "the reason says the listing is what failed: {reason}");
}

/// What:
///  A table this account may not list answers that the look could not be made,
///  and the same table with the permission given back names the process working in the folder.
/// Why:
///  The permission is the only difference between the two arms,
///  so the control shows the answer rests on the listing and not on what the table holds:
///  a table whose entries were never read holds every process that works in the worktree,
///  and the removal took the worktree while one did.
#[test]
fn a_table_this_account_may_not_list_is_never_nothing_works_there() {
    use std::os::unix::fs::PermissionsExt as _;
    let root: PathBuf = fixture("process-table-unlistable");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let pid: u32 = 5454;
    let table: PathBuf = root.join("proc");
    let entry: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("a process entry");
    std::os::unix::fs::symlink(worktree.as_path(), entry.join("cwd")).expect("the working folder of the process");
    std::fs::set_permissions(table.as_path(), std::fs::Permissions::from_mode(0o000)).expect("a table this account may not list");
    let refused: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    std::fs::set_permissions(table.as_path(), std::fs::Permissions::from_mode(0o700)).expect("the table is listable again");
    let listed: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    assert_eq!(listed, working(pid, WorkingFolder::Inside), "the same table, listable, names the process working there");
    let ProcessLook::CannotLook(reason) = refused else {
        panic!("a table nobody could list answered {refused:?}");
    };
    assert!(reason.contains(table.display().to_string().as_str()), "the reason names the table: {reason}");
    assert!(reason.contains("could not be listed"), "the reason says what failed: {reason}");
}

/// What:
///  A listing that stops answering before its end answers that the look could not be made,
///  though every entry it did name held nothing.
/// Why:
///  A real listing answers an error for an entry only when the kernel refuses to name it,
///  which no directory of a fixture can make it do,
///  so the walk is written over the listing's items and this control names them:
///  one entry whose process works elsewhere,
///  then a read that refused.
///  The entries the listing never named are processes nobody looked at,
///  so the answer is never that nothing works there.
///  The same first entry alone answers the other way in the same control.
#[test]
fn a_listing_that_stops_answering_before_its_end_is_never_nothing_works_there() {
    let root: PathBuf = fixture("process-listing-stopped");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir(elsewhere.as_path()).expect("a folder another process works in");
    let table: PathBuf = root.join("proc");
    let pid: u32 = 5656;
    let entry: PathBuf = table.join(pid.to_string());
    std::fs::create_dir_all(entry.as_path()).expect("a process entry");
    std::os::unix::fs::symlink(elsewhere.as_path(), entry.join("cwd")).expect("the working folder of the process");
    let whole: ProcessLook =
        super::look_at_each(std::iter::once(Ok::<PathBuf, std::io::Error>(entry.clone())), worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    let stopped: ProcessLook = super::look_at_each(
        vec![Ok(entry), Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))].into_iter(),
        worktree.as_path(),
        &worktree.join(".git"),
        table.as_path(),
        None,
    );
    assert_eq!(whole, ProcessLook::NothingWorksThere, "the one entry that was read holds nothing in the worktree");
    let ProcessLook::CannotLook(reason) = stopped else {
        panic!("a listing that stopped answering answered {stopped:?}");
    };
    assert!(reason.contains(table.display().to_string().as_str()), "the reason names the table: {reason}");
    assert!(reason.contains("stopped answering"), "the reason says the listing stopped before its end: {reason}");
}

/// What the read answered,
/// as the words a control of a probe compares.
/// Why:
///  A probe runs in a process of its own and answers on its standard output,
///  so its answer travels as text and the shape of it is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// describe(look) // 'nothing works there' | 'a process works there' | 'cannot look'
/// ```
fn describe(look: &ProcessLook) -> String {
    return match look {
        ProcessLook::Working(found) => format!("a process works there: process {} answered {:?}", found.pid, found.folder),
        ProcessLook::NothingWorksThere => String::from("nothing works there"),
        ProcessLook::CannotLook(reason) => format!("cannot look: {reason}"),
    };
}

/// What:
///  The harness name of one control in this module,
///  as the parent that starts this same test binary on it names it.
/// Why:
///  A probe runs in a child of the test binary,
///  and the child is given the one control it must run,
///  so the name is built from this module's own path rather than written out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theTestPath('probe') // 'worktree::worktree_process_inside::tests::probe'
/// ```
fn the_test_path(name: &str) -> String {
    let module: &str = module_path!();
    let without_crate: &str = module.split_once("::").map_or(module, |(_crate, rest)| { return rest; });
    return format!("{without_crate}::{name}");
}

/// What:
///  The variable that names the worktree a bind-mount probe binds at another spelling.
/// Why:
///  A probe runs only when its variable is set,
///  so the same control is inert in the suite's own process.
const BIND_MOUNT_WORKTREE_VARIABLE: &str = "CLI_GIT_PROCESS_BIND_MOUNT_WORKTREE";

/// The variable that names the folder a bind-mount probe binds the worktree at.
const BIND_MOUNT_ALIAS_VARIABLE: &str = "CLI_GIT_PROCESS_BIND_MOUNT_ALIAS";

/// What:
///  Bind the folder `source` at the folder `at`,
///  inside the private mount namespace this process runs in.
/// Why:
///  A bind mount is the spelling the removal cannot resolve:
///  it is no symbolic link, so no canonical path leads from one spelling to the other,
///  and the two names name one folder by its device and inode alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bindMount(source: string, at: string): void // mount(source, at, NULL, MS_BIND, NULL)
/// ```
fn bind_mount(source: &Path, at: &Path) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt as _;
    let from: std::ffi::CString = std::ffi::CString::new(source.as_os_str().as_bytes()).map_err(|_error| {
        return std::io::Error::from(std::io::ErrorKind::InvalidInput);
    })?;
    let to: std::ffi::CString = std::ffi::CString::new(at.as_os_str().as_bytes()).map_err(|_error| {
        return std::io::Error::from(std::io::ErrorKind::InvalidInput);
    })?;
    // SAFETY: both names are NUL-terminated and outlive the call, and a bind mount reads no file system type and no data.
    let called: libc::c_int = unsafe { libc::mount(from.as_ptr(), to.as_ptr(), std::ptr::null(), libc::MS_BIND, std::ptr::null()) };
    return if called == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
}

/// What:
///  The probe a bind-mount control runs in a private mount namespace:
///  bind the worktree at the alias,
///  start a helper working in the alias,
///  and answer what the read says.
/// Why:
///  A mount may not be made in this machine's own mount table,
///  so the parent starts this same test binary with `unshare --user --map-root-user --mount --propagation private`,
///  and only a process inside that namespace sees the alias.
///  The probe says whether the kernel named the alias for the helper's working folder,
///  which is the case that makes the miss real rather than assumed.
#[test]
fn a_bind_mount_probe() {
    let Some(worktree) = std::env::var_os(BIND_MOUNT_WORKTREE_VARIABLE) else {
        return;
    };
    let alias: PathBuf = PathBuf::from(std::env::var_os(BIND_MOUNT_ALIAS_VARIABLE).expect("the alias the parent named"));
    let bound: PathBuf = PathBuf::from(worktree);
    bind_mount(bound.as_path(), alias.as_path()).expect("the worktree is bound at the alias in this namespace");
    let mut helper: std::process::Child = spawn_owned(std::process::Command::new("sleep").arg("120").current_dir(alias.as_path()))
        .expect("a helper working in the alias");
    let named: bool = crate::test_support::wait_until(&|| {
        return std::fs::read_link(format!("/proc/{}/cwd", helper.id())).is_ok_and(|there| { return there == alias; });
    });
    let through: PathBuf = std::fs::read_link(format!("/proc/{}/cwd", helper.id())).unwrap_or_default();
    let answered: ProcessLook = process_working_inside(bound.as_path(), &bound.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    let _ended = helper.kill();
    let _waited = helper.wait();
    println!("bind-mount-named {}", if named { "yes" } else { "no" });
    println!("bind-mount-cwd {}", through.display());
    println!("bind-mount-answer {}", describe(&answered));
}

/// What:
///  A process that works in the worktree through a bind mount is found,
///  so a removal's proof does not let the worktree go.
/// Why:
///  The read compared the text of the folder a process works in with the text of the worktree,
///  and a bind mount is a second spelling of one folder that no canonical path resolves,
///  which a symbolic link is not (`a_process_that_entered_the_worktree_through_a_link_is_found`).
///  A person whose repository is reachable at two spellings, one of them a bind mount,
///  and who works in a worktree through the second, lost that worktree with the files they had open in it.
///  The probe binds the worktree at the alias inside a private user, mount and process namespace of its own,
///  starts a real helper working there,
///  and the kernel names the alias for that helper's working folder.
#[test]
fn a_process_that_works_in_the_worktree_through_a_bind_mount_is_found() {
    let root: PathBuf = fixture("process-bind-mount");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let alias: PathBuf = root.join("through-a-bind-mount");
    std::fs::create_dir(alias.as_path()).expect("a folder the worktree is bound at");
    let Some(said) = run_the_probe_in_a_private_namespace(
        "a_bind_mount_probe",
        &[(BIND_MOUNT_WORKTREE_VARIABLE, worktree.as_os_str()), (BIND_MOUNT_ALIAS_VARIABLE, alias.as_os_str())],
    ) else {
        return;
    };
    assert!(said.contains("bind-mount-named yes"), "the kernel named the alias for the helper's working folder:\n{said}");
    assert!(
        said.contains(alias.display().to_string().as_str()),
        "the probe's precondition names the alias the helper works in:\n{said}"
    );
    assert!(
        said.contains("bind-mount-answer a process works there"),
        "the read found no process in a worktree one works in through a bind mount:\n{said}"
    );
    assert!(said.contains("answered Inside"), "the process works in the folder itself, which the alias names:\n{said}");
}

/// What:
///  Run one probe of this module in a private user, mount and process namespace,
///  and answer what it printed,
///  or nothing when this machine refused to make them.
/// Why:
///  A mount may not be made in this machine's own mount table,
///  and the read must see only the processes the probe started:
///  the machine's own table holds programs of this account that hide what they do,
///  and one of those that ends while it is read answers for a control that meant to read nothing.
///  A refusal is told from a probe that failed by the harness line a process prints when it starts:
///  a probe the namespace never started prints none of it.
fn run_the_probe_in_a_private_namespace(name: &str, variables: &[(&str, &std::ffi::OsStr)]) -> Option<String> {
    return run_the_probe_in_namespaces(
        name,
        variables,
        &["--user", "--map-root-user", "--mount", "--propagation", "private", "--pid", "--fork", "--mount-proc"],
    );
}

/// What:
///  The same probe run with the namespaces the control names,
///  and the words a failure printed.
/// Why:
///  One control needs a process file system of its own and a mount of its own (the bind-mount probes),
///  and another needs only a process namespace of its own,
///  keeping the machine's process file system on purpose (the table from another PID namespace),
///  so the namespaces are the control's to choose.
///  A refusal is told from a probe that failed by the harness line a process prints when it starts.
fn run_the_probe_in_namespaces(name: &str, variables: &[(&str, &std::ffi::OsStr)], namespaces: &[&str]) -> Option<String> {
    let mut command: std::process::Command = std::process::Command::new("unshare");
    command.args(namespaces);
    command.arg(crate::test_support::this_test_binary().expect("the test binary"));
    command.args(["--exact", the_test_path(name).as_str(), "--nocapture", "--test-threads=1"]);
    for (variable, value) in variables {
        command.env(variable, value);
    }
    let output: std::process::Output = command.stdin(std::process::Stdio::null()).output().expect("unshare starts the probe");
    let said: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
    let refused: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    if !said.contains("running 1 test") {
        println!("CONTROL NOT RUN: this machine refused a private namespace: {refused}");
        return None;
    }
    assert!(output.status.success(), "the probe in its own namespace failed:\n{said}\n{refused}");
    return Some(said);
}

/// The variable that names the worktree a mapped-and-closed probe maps a file in.
const MAPPED_WORKTREE_VARIABLE: &str = "CLI_GIT_PROCESS_MAPPED_WORKTREE";

/// What:
///  The probe a mapped-and-closed control runs:
///  map a file of the worktree the parent names,
///  close every descriptor on it,
///  and answer what the read says,
///  with the two preconditions it checked itself.
/// Why:
///  A mapping outlives the descriptor it was made from,
///  and only a process that really maps the file and closes it proves the read cannot see it.
///  The preconditions are read from this process's own table before the answer,
///  because the control's parent is in another process namespace and cannot see these numbers:
///  the file is in this process's mappings,
///  and no descriptor of it names the file.
#[test]
fn a_mapped_and_closed_file_probe() {
    use std::os::unix::io::AsRawFd as _;
    let Some(worktree) = std::env::var_os(MAPPED_WORKTREE_VARIABLE) else {
        return;
    };
    let holding: PathBuf = PathBuf::from(worktree);
    let held: PathBuf = holding.join("mapped.txt");
    let length: usize = usize::try_from(std::fs::metadata(held.as_path()).expect("the file the probe maps").len()).expect("a file this machine holds");
    let file: std::fs::File = std::fs::File::open(held.as_path()).expect("the file the probe maps");
    // SAFETY: the descriptor is open for the call, the length is the file's own, and the mapping is read-only and private.
    let mapped: *mut libc::c_void = unsafe { libc::mmap(std::ptr::null_mut(), length, libc::PROT_READ, libc::MAP_PRIVATE, file.as_raw_fd(), 0) };
    assert_ne!(mapped, libc::MAP_FAILED, "the file is mapped: {}", std::io::Error::last_os_error());
    drop(file);
    let descriptors: std::fs::ReadDir = std::fs::read_dir("/proc/self/fd").expect("this process's own descriptors");
    let named: bool = descriptors.flatten().any(|one| { return std::fs::read_link(one.path()).is_ok_and(|target| { return target == held; }); });
    let maps: String = std::fs::read_to_string("/proc/self/maps").expect("this process's own mappings");
    let answered: ProcessLook = process_working_inside(holding.as_path(), &holding.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    println!("mapped-file {}", if maps.contains(held.display().to_string().as_str()) { "yes" } else { "no" });
    println!("descriptor-names-it {}", if named { "yes" } else { "no" });
    println!("mapped-answer {}", describe(&answered));
}

/// What:
///  A process that mapped a file in the worktree and closed its descriptor is found,
///  so a removal's proof does not let the worktree go.
/// Why:
///  The read looked at the files a process holds open,
///  and a mapping outlives the descriptor it was made from:
///  the program still reads the file, and a shared mapping can still write it,
///  while the removal takes the directory it was in.
///  An editor or a viewer that maps what it reads, or a library that does it for the program,
///  lost the files it still held when its worktree went.
///  The probe maps a file of the worktree, closes its descriptor, and says from its own table that the file is
///  in `/proc/self/maps` and that no descriptor names it.
#[test]
fn a_process_that_mapped_a_file_in_the_worktree_and_closed_it_is_found() {
    let root: PathBuf = fixture("process-mapped-and-closed");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    std::fs::write(worktree.join("mapped.txt").as_path(), b"the file the process maps\n").expect("the file the probe maps");
    let Some(said) = run_the_probe_in_a_private_namespace("a_mapped_and_closed_file_probe", &[(MAPPED_WORKTREE_VARIABLE, worktree.as_os_str())]) else {
        return;
    };
    assert!(said.contains("mapped-file yes"), "the probe really holds the file mapped:\n{said}");
    assert!(said.contains("descriptor-names-it no"), "no descriptor of the probe names the file:\n{said}");
    assert!(
        said.contains("mapped-answer a process works there"),
        "the read found no process in a worktree whose file one holds mapped:\n{said}"
    );
    assert!(said.contains("answered MappedFile"), "the answer says a file of the folder is mapped:\n{said}");
}

/// What:
///  A process that entered the worktree through a symbolic link is found,
///  because the kernel names the folder it works in with the link resolved.
/// Why:
///  A link is the spelling a person meets most,
///  `/home` for `/var/home` above all,
///  and the read is safe for it without asking:
///  `/proc/<pid>/cwd` is the kernel's own answer,
///  and a process that entered through the link works in the folder it leads to.
#[test]
fn a_process_that_entered_the_worktree_through_a_link_is_found() {
    let root: PathBuf = fixture("process-through-a-link");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let link: PathBuf = root.join("through-the-link");
    std::os::unix::fs::symlink(worktree.as_path(), link.as_path()).expect("a link to the worktree");
    let mut worker: std::process::Child =
        spawn_owned(std::process::Command::new("sleep").arg("120").current_dir(link.as_path())).expect("a helper that entered through the link");
    let found: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    let _ended = worker.kill();
    let _waited = worker.wait();
    assert_eq!(found, working(worker.id(), WorkingFolder::Inside), "the kernel names the folder the link leads to, so the process is found");
}

/// What:
///  A worktree named through a link hides a process that really works in it,
///  and the same process is found when the worktree is named by its own path.
/// Why:
///  The read compares the text of the folder a process works in with the text of the worktree,
///  so a caller that names the worktree through a link is answered "nothing works there".
///  What this costs:
///  a removal that named the worktree that way would take it under a live process.
///  It cannot through the landed sweep, which reads the registration's own folder
///  (`worktree_landed_registrations.rs` `registered_worktree` canonicalizes it),
///  so this is a limit of the read's own contract and not of the removal that exists today:
///  a caller added later that passes a path a person spelled would meet it.
#[test]
fn a_root_spelled_through_a_link_hides_a_process_that_really_works_in_it() {
    let root: PathBuf = fixture("process-root-through-a-link");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let link: PathBuf = root.join("through-the-link");
    std::os::unix::fs::symlink(worktree.as_path(), link.as_path()).expect("a link to the worktree");
    let mut worker: std::process::Child =
        spawn_owned(std::process::Command::new("sleep").arg("120").current_dir(worktree.as_path())).expect("a helper working in the worktree");
    let through_the_link: ProcessLook = process_working_inside(link.as_path(), &link.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    let by_its_own_path: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    let _ended = worker.kill();
    let _waited = worker.wait();
    assert_eq!(
        by_its_own_path,
        working(worker.id(), WorkingFolder::Inside),
        "the same process is found when the worktree is named by its own path"
    );
    assert_eq!(
        through_the_link,
        ProcessLook::NothingWorksThere,
        "a worktree named through a link hides the process that works in it"
    );
}

/// What:
///  A table that is no directory is never a table with nothing in it.
/// Why:
///  A process file system that is there and is not a directory hides every process the same way one that is absent
///  does,
///  and the read is answered by `read_dir`'s own refusal,
///  which is the same arm as a table this account may not list.
#[test]
fn a_table_that_is_no_directory_is_never_nothing_works_there() {
    let root: PathBuf = fixture("process-table-no-directory");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let table: PathBuf = root.join("proc");
    std::fs::write(table.as_path(), b"a file where the process table should be\n").expect("a table that is no directory");
    let answered: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    let ProcessLook::CannotLook(reason) = answered else {
        panic!("a table that is no directory answered {answered:?}");
    };
    assert!(reason.contains(table.display().to_string().as_str()), "the reason names the table: {reason}");
    assert!(reason.contains("could not be listed"), "the reason says what failed: {reason}");
}

/// What:
///  A table that is not this process's own view of the machine's processes is never a table with nothing in it.
/// Why:
///  A process file system of another PID namespace numbers this process differently from the operating system,
///  and it lists only that namespace's processes:
///  a person's shell working in a worktree outside it is not in the table at all,
///  and a removal that read the table to its end removed the worktree under that shell
///  (an independent review of 2026-11-16 through the built wrapper).
///  The table is the fixture's own here,
///  so the answer does not depend on which processes this machine runs.
#[test]
fn a_table_that_is_not_this_processes_own_view_is_never_nothing_works_there() {
    let root: PathBuf = fixture("process-table-other-view");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let table: PathBuf = root.join("proc");
    let self_entry: PathBuf = table.join("self");
    std::fs::create_dir_all(self_entry.as_path()).expect("the entry a process table makes for its reader");
    let numbered: u32 = std::process::id() + 1;
    std::fs::write(self_entry.join("stat"), format!("{numbered} (a shell) S 0 0 0 0 0\n")).expect("a record that numbers this process as another namespace does");
    let answered: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    let ProcessLook::CannotLook(reason) = answered else {
        panic!("a table that numbers this process {numbered} where the operating system numbers it {} answered {answered:?}", std::process::id());
    };
    assert!(reason.contains(table.display().to_string().as_str()), "the reason names the table: {reason}");
    assert!(reason.contains("own view"), "the reason says the table is not this machine's view of its processes: {reason}");
}

/// The variable that names the worktree a probe of another view of the process table reads for.
const OTHER_VIEW_WORKTREE_VARIABLE: &str = "CLI_GIT_PROCESS_OTHER_VIEW_WORKTREE";

/// What:
///  The probe an other-view control runs inside a private PID namespace that keeps the machine's own process file system:
///  say how the operating system numbers this process,
///  how the table numbers it,
///  and what the read answers.
/// Why:
///  Only a process in a namespace of its own can hold one number and be named by another,
///  which is the state the removal must never read as a table holding nothing.
#[test]
fn an_other_view_probe() {
    let Some(worktree) = std::env::var_os(OTHER_VIEW_WORKTREE_VARIABLE) else {
        return;
    };
    let worktree: PathBuf = PathBuf::from(worktree);
    let own: u32 = std::process::id();
    let named: String = std::fs::read_to_string("/proc/self/stat").map(|stat| { return stat.split(' ').next().unwrap_or_default().to_string(); }).unwrap_or_default();
    let answered: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), Path::new("/proc"), Some(Duration::ZERO));
    println!("other-view-own {own}");
    println!("other-view-named {named}");
    println!("other-view-answer {}", describe(&answered));
}

/// What:
///  A process table of another PID namespace is never a table with nothing in it,
///  read from inside that namespace against the machine's own process file system.
/// Why:
///  This is the state an agent's container or a person's own namespace puts the wrapper in,
///  and the table then lists none of the machine's other processes.
///  The probe runs with `unshare --user --map-root-user --pid --fork` and no new process file system,
///  so its `/proc` is this machine's and its own number is its namespace's:
///  the two disagree, which is what the read must say instead of "nothing works there".
#[test]
fn a_table_of_another_pid_namespace_is_never_nothing_works_there() {
    let root: PathBuf = fixture("process-table-another-namespace");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process may work in");
    let Some(said) = run_the_probe_in_namespaces(
        "an_other_view_probe",
        &[(OTHER_VIEW_WORKTREE_VARIABLE, worktree.as_os_str())],
        &["--user", "--map-root-user", "--pid", "--fork"],
    ) else {
        return;
    };
    let own: String = said.lines().find_map(|line| { return line.strip_prefix("other-view-own ").map(str::to_string); }).expect("the probe said its own number");
    let named: String = said.lines().find_map(|line| { return line.strip_prefix("other-view-named ").map(str::to_string); }).expect("the probe said the number the table gives it");
    assert_ne!(own, named, "the machine's process file system numbers the probe differently from its own namespace:\n{said}");
    assert!(
        said.contains("other-view-answer cannot look"),
        "a table of another PID namespace answered that nothing works there:\n{said}"
    );
}

/// What:
///  A process whose first thread has ended, while another of its threads works in the worktree,
///  is found rather than taken for one that went.
/// Why:
///  The kernel answers `/proc/<pid>/cwd` with `NotFound` once a process's first thread has ended,
///  and lists no descriptor for that process,
///  while `/proc/<pid>/task/<thread>/cwd` still names the folder every thread works in.
///  The comment that read `NotFound` as "the process went" was wrong for this state,
///  which an independent review of 2026-11-16 reproduced at the function.
///  The table is the fixture's own, so the control does not depend on the machine's programs.
#[test]
fn a_process_whose_first_thread_ended_may_still_work_there() {
    let root: PathBuf = fixture("process-first-thread-ended");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let table: PathBuf = root.join("proc");
    let pid: u32 = 6060;
    let thread: u32 = 6061;
    let entry: PathBuf = table.join(pid.to_string());
    let task: PathBuf = entry.join("task").join(thread.to_string());
    std::fs::create_dir_all(task.as_path()).expect("a thread of the process");
    std::fs::write(entry.join("stat"), stat_with_start_ticks(pid, 100)).expect("the process's own row");
    std::os::unix::fs::symlink(worktree.as_path(), task.join("cwd")).expect("the folder the thread works in");
    let found: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    assert_eq!(
        found,
        working(pid, WorkingFolder::Inside),
        "a process whose first thread ended and whose other thread works there is working there"
    );
}

/// What:
///  A process entry whose account cannot be read may be working there,
///  and is never taken for another account's process.
/// Why:
///  `owned_by_this_account` answered "not this account's" for any failed `stat`,
///  and the process was then skipped in silence;
///  an entry that cannot be read as a link at all is such a `stat`,
///  which the fixture writes as a link to itself (the kernel answers `ELOOP`).
///  Nothing proves the process is not this account's, so the worktree stays.
#[test]
fn a_process_entry_whose_account_cannot_be_read_may_be_working_there() {
    let root: PathBuf = fixture("process-entry-account-unreadable");
    let worktree: PathBuf = root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("a folder a process works in");
    let table: PathBuf = root.join("proc");
    std::fs::create_dir_all(table.as_path()).expect("a process table");
    let pid: u32 = 6262;
    let name: String = pid.to_string();
    std::os::unix::fs::symlink(std::ffi::OsStr::new(name.as_str()), table.join(name.as_str())).expect("an entry whose own record cannot be read");
    let found: ProcessLook = process_working_inside(worktree.as_path(), &worktree.join(".git"), table.as_path(), None);
    assert!(
        matches!(found, ProcessLook::Working(process) if process.pid == pid),
        "an entry whose account could not be read answered {found:?}"
    );
}

/// What:
///  A process the two readings cannot place on either side of the folder's birth is never taken as older than it.
/// Why:
///  `/proc/uptime` counts hundredths of a second and a start counts ticks of ten milliseconds,
///  so a process that started just after a folder can be read as having started before it,
///  and the removal then took the worktree under it
///  (measured by an independent review of 2026-11-16:
/// four of a hundred at no delay and two of a hundred at four milliseconds).
///  Only a difference wider than both readings can show is an order.
#[test]
fn a_process_within_the_readings_of_the_folder_is_never_taken_as_older() {
    let folder: Duration = Duration::from_secs(5_000);
    assert!(
        !started_before(Some(folder + Duration::from_millis(1)), Some(folder)),
        "a process one millisecond older than the folder cannot be ordered against it"
    );
    assert!(!started_before(Some(folder - Duration::from_millis(1)), Some(folder)), "a younger process is not older");
    assert!(
        started_before(Some(folder + Duration::from_secs(1)), Some(folder)),
        "a process a second older than the folder is outside it"
    );
}
