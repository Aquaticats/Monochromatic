//! Controls for `service settle-earlier-batches` and the note `doctor` adds about it:
//! what each answer says,
//! that the
//!  condition of the command is said before it acts,
//! and that nothing is changed while an earlier helper may run or start.

use super::*;
use crate::held_commit_record::{Holding, hold};
use crate::owed_index_update;
use crate::service_instance::instance_of;
use crate::service_journal::{Intent, OPEN_EXTENSION, batch_path, write_intent, write_start};
use crate::service_land_start::witness_of;
use crate::test_support::{REAL_GIT, fixture, git, git_text, isolated_overlay, remove, repository};
use crate::wrapper_names_old_and_new::DATA_FOLDER;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt as _;

/// The command word the notes name.
const COMMAND: &str = "git monochromatic-git-policy-cli service settle-earlier-batches";

/// One disposable world:
/// a state folder,
/// a Git directory,
/// a process table.
struct Setup {
    root: PathBuf,
    state: PathBuf,
    git_dir: PathBuf,
    proc_root: PathBuf,
}

fn setup(name: &str) -> Setup {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("repo/.git");
    std::fs::create_dir_all(&git_dir).expect("git directory");
    let proc_root: PathBuf = root.join("proc");
    std::fs::create_dir_all(proc_root.join("net")).expect("net folder");
    std::fs::write(proc_root.join("net/unix"), "Num       RefCount Protocol Flags    Type St Inode Path\n").expect("an empty table");
    return Setup { state: root.join("state"), root, git_dir, proc_root };
}

/// One disposable world whose repository is real,
/// because a held batch names commits that must exist,
/// with the process table and the state folder beside it.
fn setup_with_repository(name: &str) -> (Setup, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let proc_root: PathBuf = root.join("proc");
    std::fs::create_dir_all(proc_root.join("net")).expect("net folder");
    std::fs::write(proc_root.join("net/unix"), "Num       RefCount Protocol Flags    Type St Inode Path\n").expect("an empty table");
    let git_dir: PathBuf = repo.join(".git");
    return (Setup { state: root.join("state"), root, git_dir, proc_root }, repo);
}

/// The file the held commit changes.
const HELD_FILE: &str = "held.txt";

/// What:
///  A batch whose commit a reference lock held off its branch,
///  left in the world's earlier journal as a refused landing leaves it,
///  with the branch still at the commit before it and the person's bytes in the worktree.
/// Why:
///  The command's own path over a held batch is what these controls judge,
///  and it starts from the state a landing that met a lock writes:
///  the commit on the batch's side reference,
///  the intent,
///  the index witness,
///  the owed index record,
///  and the held record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = heldBatchIn(journal, repo); // branch at base, worktree bytes kept
/// ```
fn held_batch_in(journal: &Path, repo: &Path) -> crate::held_commit_record::Held {
    let git_dir: PathBuf = repo.join(".git");
    let base: String = git_text(repo, &["rev-parse", "HEAD"]);
    std::fs::write(repo.join(HELD_FILE), b"the held edit\n").expect("the held edit");
    git(repo, &["add", "--", HELD_FILE]);
    git(repo, &["commit", "--quiet", "--message", "the held edit"]);
    let held_commit: String = git_text(repo, &["rev-parse", "HEAD"]);
    // The branch write was refused, so the commit is only on the batch's own side reference and the branch stayed where it was.
    git(repo, &["update-ref", "--no-deref", crate::held_commit_settle::kept_reference(1).as_str(), held_commit.as_str()]);
    git(repo, &["reset", "--hard", "--quiet", base.as_str()]);
    // The person's worktree still holds the bytes the commit captured.
    std::fs::write(repo.join(HELD_FILE), b"the held edit\n").expect("the edit stays in the worktree");
    let intent: Intent = Intent {
        seq: 1,
        repository: repo.to_path_buf(),
        common_dir: git_dir.clone(),
        reference: String::from("refs/heads/main"),
        old: base,
        new: held_commit.clone(),
        ids: vec![String::from("request-1")],
        commits: vec![held_commit],
        holders: vec![String::new()],
        paths: vec![HELD_FILE.as_bytes().to_vec()],
        derived: false,
        cleanup: None,
        index: git_dir.join("index"),
        git_dir: git_dir.clone(),
    };
    write_intent(journal, &intent).expect("the intent");
    write_start(journal, 1, &witness_of(intent.index.as_path()).expect("the index file")).expect("the witness");
    // The landing writes the record of the index update it will owe before it moves the branch, so it is here whether or not the
    //  branch write was refused.
    owed_index_update::write(&owed_index_update::of_intent(&intent));
    return hold(Holding {
        journal,
        repository: repo,
        common_dir: git_dir.as_path(),
        git_dir: git_dir.as_path(),
        index: intent.index.as_path(),
        reference: intent.reference.as_str(),
        seq: 1,
        intent: &intent,
        locks: vec![git_dir.join("refs/heads/main.lock")],
        reflog: b"commit: the held edit".as_slice(),
    })
    .expect("the held record");
}

impl Setup {
    fn instance(&self) -> String {
        return instance_of(self.git_dir.as_path());
    }

    /// Leaves `count` open batches of this worktree in its earlier journal,
    /// and one of another worktree beside them.
    fn leave_open_batches(&self, count: u64) {
        let journal: PathBuf = self.state.join(DATA_FOLDER.earlier).join("service").join(self.instance());
        std::fs::create_dir_all(&journal).expect("the earlier journal");
        for seq in 1..=count {
            write_intent(journal.as_path(), &self.intent(seq, self.git_dir.as_path())).expect("an open batch");
        }
        let shared: PathBuf = self.state.join(DATA_FOLDER.earlier).join("service");
        write_intent(shared.as_path(), &self.intent(90, self.root.join("elsewhere/.git").as_path())).expect("another worktree's batch");
    }

    fn intent(&self, seq: u64, git_dir: &Path) -> Intent {
        return Intent {
            seq,
            repository: self.root.join("repo-that-is-not-there"),
            common_dir: git_dir.to_path_buf(),
            reference: String::from("refs/heads/main"),
            old: "a".repeat(40),
            new: "b".repeat(40),
            ids: vec![format!("id-{seq}")],
            commits: vec!["c".repeat(40)],
            holders: vec![String::new()],
            paths: vec![b"a.txt".to_vec()],
            derived: false,
            cleanup: None,
            index: git_dir.join("index"),
            git_dir: git_dir.to_path_buf(),
        };
    }

    fn request(&self) -> SettleRequest<'_> {
        return SettleRequest {
            selection: Selection::Found(self.git_dir.clone()),
            state_dir: Some(self.state.as_path()),
            git: Some(GitContext { real_git: PathBuf::from(REAL_GIT), overlay: isolated_overlay(), global_prefix: Vec::new(), common_dir: None }),
            proc_root: self.proc_root.as_path(),
        };
    }

    fn open_files(&self) -> usize {
        let journal: PathBuf = self.state.join(DATA_FOLDER.earlier).join("service").join(self.instance());
        return (1..=9).filter(|seq| { return batch_path(journal.as_path(), *seq, OPEN_EXTENSION).exists(); }).count();
    }
}

/// The parts of an action that exits.
fn parts(action: Action) -> (i32, String, String) {
    let Action::Exit { code, stdout, stderr } = action else { panic!("the command exits") };
    return (code, stdout, stderr);
}

/// Runs the command,
/// giving what it said before acting as well.
fn run(request: &SettleRequest<'_>) -> ((i32, String, String), String) {
    let mut before: Vec<u8> = Vec::new();
    let action: Action = settle_earlier_batches_action(request, &mut before);
    return (parts(action), String::from_utf8_lossy(before.as_slice()).into_owned());
}

/// The state folder is `XDG_STATE_HOME` when set and not empty,
/// else `.local/state` of the home folder,
/// else unknown.
#[test]
fn the_state_folder_follows_the_environment() {
    let pair = |name: &str, value: &str| -> (OsString, OsString) { return (OsString::from(name), OsString::from(value)) };
    assert_eq!(state_dir_of(&[pair("XDG_STATE_HOME", "/s"), pair("HOME", "/h")]), Some(PathBuf::from("/s")));
    assert_eq!(state_dir_of(&[pair("XDG_STATE_HOME", ""), pair("HOME", "/h")]), Some(PathBuf::from("/h/.local/state")));
    assert_eq!(state_dir_of(&[pair("HOME", "/h")]), Some(PathBuf::from("/h/.local/state")));
    assert_eq!(state_dir_of(&[pair("HOME", "")]), None);
    assert_eq!(state_dir_of(&[]), None);
}

/// With no batch left,
/// the command says so and exits 0,
/// and says nothing of a condition because it acts on nothing.
#[test]
fn with_no_batch_left_the_command_says_so_and_acts_on_nothing() {
    let world: Setup = setup("settle-command-nothing");
    let ((code, stdout, stderr), before) = run(&world.request());
    assert_eq!(code, 0);
    assert!(stdout.contains("No batch of the earlier release is left open for this worktree"), "{stdout}");
    assert_eq!((stderr.as_str(), before.as_str()), ("", ""));
    remove(world.root.as_path());
}

/// A command that cannot tell which worktree,
/// or where the journals are,
/// settles nothing and says why.
#[test]
fn a_command_that_cannot_find_the_worktree_or_the_journals_says_so_and_changes_nothing() {
    let world: Setup = setup("settle-command-unknown");
    world.leave_open_batches(1);
    let mut outside = world.request();
    outside.selection = Selection::Outside;
    let ((outside_code, outside_out, outside_err), before) = run(&outside);
    assert_eq!((outside_code, outside_out.as_str(), before.as_str()), (1, "", ""));
    assert!(outside_err.contains("not inside a repository"), "{outside_err}");
    let mut unresolved = world.request();
    unresolved.selection = Selection::Unresolved(String::from("Git said: boom"));
    let ((unresolved_code, _, unresolved_err), _) = run(&unresolved);
    assert_eq!(unresolved_code, 1);
    assert!(unresolved_err.contains("Git said: boom"), "{unresolved_err}");
    let mut homeless = world.request();
    homeless.state_dir = None;
    let ((homeless_code, _, homeless_err), _) = run(&homeless);
    assert_eq!(homeless_code, 1);
    assert!(homeless_err.contains("state folder"), "{homeless_err}");
    assert_eq!(world.open_files(), 1);
    remove(world.root.as_path());
}

/// Appends a listening socket line to the fake table.
fn listen_at(world: &Setup, file: &str) {
    let table: PathBuf = world.proc_root.join("net/unix");
    let before: String = std::fs::read_to_string(&table).expect("table");
    std::fs::write(&table, format!("{before}0000000000000001: 00000002 00000000 00010000 0001 01 1 /run/user/1000/cli-git/{file}\n")).expect("a listening socket");
}

/// Puts a fake process with `words` in the table.
fn run_process(world: &Setup, words: &[String]) {
    let process: PathBuf = world.proc_root.join("77");
    std::fs::create_dir_all(&process).expect("process");
    let mut bytes: Vec<u8> = Vec::new();
    for word in words {
        bytes.extend(word.as_bytes());
        bytes.push(0);
    }
    std::fs::write(process.join("cmdline"), bytes).expect("command line");
}

/// What the kernel calls the fake process's program,
/// as `/proc/77/comm` spells it.
const FAKE_PROGRAM_NAME: &str = "unrelated-tool";

/// Names the fake process's program,
/// as every process on a machine has a name.
fn name_the_process(world: &Setup) {
    std::fs::write(world.proc_root.join("77/comm"), format!("{FAKE_PROGRAM_NAME}\n")).expect("the program's kernel name");
}

/// How long a command line of another program may be:
/// a word is at most 32 pages (`MAX_ARG_STRLEN`,
/// Linux `include/uapi/linux/binfmts.h`),
/// and the kernel refuses a command line and environment together above 6 MiB (`fs/exec.c`,
/// `bprm_stack_limits`).
/// This control's line is 200 words of 1 KiB,
/// 204,800 bytes:
/// longer than the 65,536 bytes the look used to read whole,
/// and far below what a program may carry.
const LONG_COMMAND_LINE_WORD_BYTES: usize = 1024;
const LONG_COMMAND_LINE_WORDS: usize = 200;

/// The words of a long command line of a program that is no earlier helper:
/// a commit whose message travels in its arguments,
/// as a person's `git commit -m` does.
fn long_command_line() -> Vec<String> {
    let mut words: Vec<String> = vec![String::from("/usr/bin/git"), String::from("commit"), String::from("-m")];
    for at in 0..LONG_COMMAND_LINE_WORDS {
        words.push(format!("{at:07}").chars().cycle().take(LONG_COMMAND_LINE_WORD_BYTES).collect());
    }
    return words;
}

/// What:
///  A program of the user whose command line is longer than the look's first limit holds nothing back:
/// the command says the condition,
/// examines the batch,
/// and settles it.
/// Why:
///   A command line of tens of kilobytes is ordinary:
/// a commit carries its message in its arguments,
/// a script may be passed with `-e`,
/// and a container run line carries its mounts,
/// and one program's measured 44,174 bytes on this host on 2026-11-10,
/// while the look read only 65,536.
/// A process whose line was longer counted as one that may be the earlier helper,
/// so it refused the command with the words of a look that ran out of time,
/// for as long as that program ran,
/// and no retry could clear it,
/// because nothing about it changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(runSettleWithAProgramCarryingALongCommandLine()).toSettleTheBatch()
/// ```
#[test]
#[cfg(unix)]
fn a_program_whose_command_line_is_long_holds_nothing_back() {
    let world: Setup = setup("settle-command-long-command-line");
    world.leave_open_batches(1);
    run_process(&world, long_command_line().as_slice());
    name_the_process(&world);
    let ((code, stdout, stderr), before) = run(&world.request());
    assert!(!stderr.contains(crate::earlier_service_process_look::Hold::CannotLook.words()), "the look proved the line whole: {stderr}");
    assert!(before.contains("no wrapper of the earlier release is in use in this worktree"), "the look passed, so the condition is said: {before}");
    assert_eq!(code, 1, "the fixture's batch names a repository that is not there, which Git cannot answer for: {stdout}{stderr}");
    assert!(stdout.contains("batch 1") && stdout.contains("could not be examined"), "the batch was examined: {stdout}");
    remove(world.root.as_path());
}

/// What:
///  A process whose command line this check may not read is named,
/// with what the kernel calls it and what the kernel answered,
/// and with the two ways past it.
/// Why:
///   Such a process cannot be ruled out as the earlier commit service,
/// so the command refuses;
/// a refusal that
///  names neither the process nor the answer leaves the person with "try again in a moment",
/// which nothing
///  about that process ever clears.
/// The kernel's name of its program is shown,
/// never a word of its command line,
/// because any of those can be a secret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(stderr).toContain('could not read the command line of process 77')
/// ```
#[test]
#[cfg(unix)]
fn a_process_whose_command_line_cannot_be_read_is_named_with_the_answer_and_the_way_on() {
    let world: Setup = setup("settle-command-names-an-unreadable-process");
    world.leave_open_batches(1);
    run_process(&world, [String::from("/usr/bin/unrelated-tool"), String::from("--journal"), String::from("/gone/for/good")].as_slice());
    name_the_process(&world);
    let cmdline: PathBuf = world.proc_root.join("77/cmdline");
    std::fs::set_permissions(cmdline.as_path(), std::fs::Permissions::from_mode(0o0)).expect("a command line no one may read");
    let ((code, stdout, stderr), before) = run(&world.request());
    assert_eq!((code, stdout.as_str(), before.as_str()), (1, "", ""), "the command refused and acted on nothing: {stderr}");
    assert!(stderr.contains(crate::earlier_service_process_look::Hold::CannotLook.words()), "the reason's words stay: {stderr}");
    for wanted in ["could not read the command line of process 77", FAKE_PROGRAM_NAME, "Permission denied", "cannot be ruled out", "wait for process 77", "stop it", "again", COMMAND] {
        assert!(stderr.contains(wanted), "{wanted}: {stderr}");
    }
    assert!(!stderr.contains("/gone/for/good"), "no word of a command line it may not read is shown: {stderr}");
    assert_eq!(world.open_files(), 1, "nothing was settled");
    remove(world.root.as_path());
}

/// A socket that listens,
/// a process that runs,
/// and a look that cannot be made each hold the command back with their own words,
///  the count of open batches,
/// what was not done,
/// and the way on that fits the cause;
/// nothing is changed and nothing was said
///  as if acting.
/// This worktree's units are named for this worktree's socket and helper;
/// the single service of the first
///  releases is named as serving every worktree,
/// with the units that exist for it;
/// a way on never names units that are not the
///  cause,
/// and the wait for an idle exit is offered only where no socket listens that would start the helper again.
#[test]
fn an_earlier_helper_that_may_run_holds_the_command_back_with_the_reason_and_the_way_that_fits() {
    let world: Setup = setup("settle-command-held");
    world.leave_open_batches(2);
    let instance: String = world.instance();
    let own_units: String = format!("cli-git@{instance}.socket cli-git@{instance}.service");
    let table: PathBuf = world.proc_root.join("net/unix");
    let header: String = std::fs::read_to_string(&table).expect("header");
    listen_at(&world, format!("{instance}.sock").as_str());
    let ((socket_code, socket_out, socket_err), before) = run(&world.request());
    assert_eq!((socket_code, socket_out.as_str(), before.as_str()), (1, "", ""));
    for wanted in ["2 batches", "listens", "Nothing was changed", own_units.as_str(), COMMAND] {
        assert!(socket_err.contains(wanted), "{wanted}: {socket_err}");
    }
    assert!(!socket_err.contains("three hours") && !socket_err.contains("cli-git.socket"), "a listening socket does not stop by waiting: {socket_err}");
    std::fs::write(&table, header.as_str()).expect("an empty table");
    listen_at(&world, "service.sock");
    let ((shared_code, _, shared_err), _) = run(&world.request());
    assert_eq!(shared_code, 1);
    for wanted in ["every worktree", "systemctl --user stop cli-git.socket cli-git.service", "Nothing was changed", "next commit made with this release", "asks the service to finish its batch", "still working after ten seconds"] {
        assert!(shared_err.contains(wanted), "{wanted}: {shared_err}");
    }
    assert!(!shared_err.contains(own_units.as_str()) && !shared_err.contains("three hours"), "units that are not the cause, or a wait that clears nothing: {shared_err}");
    std::fs::write(&table, header.as_str()).expect("an empty table");
    run_process(&world, &[String::from("/p/cli-git"), String::from("cli-git"), String::from("service"), String::from("run"), String::from("--instance"), instance.clone()]);
    let ((process_code, _, process_err), _) = run(&world.request());
    assert_eq!(process_code, 1);
    for wanted in ["is running as the commit service of this worktree", "Nothing was changed", own_units.as_str(), "three hours"] {
        assert!(process_err.contains(wanted), "{wanted}: {process_err}");
    }
    run_process(&world, &[String::from("/p/cli-git-native"), String::from("--journal"), world.state.join("cli-git/service").display().to_string()]);
    let ((single_code, _, single_err), _) = run(&world.request());
    assert_eq!(single_code, 1);
    assert!(single_err.contains("every worktree") && single_err.contains("systemctl --user stop cli-git.socket cli-git.service") && !single_err.contains(own_units.as_str()), "{single_err}");
    run_process(&world, &[String::from("/p/cli-git-native"), String::from("--journal"), String::from("not/yet/there")]);
    let ((unsure_code, _, unsure_err), _) = run(&world.request());
    assert_eq!(unsure_code, 1);
    assert!(unsure_err.contains(own_units.as_str()) && unsure_err.contains("cli-git.socket cli-git.service"), "both sets of units, because the cause is not known: {unsure_err}");
    std::fs::remove_dir_all(world.proc_root.join("77")).expect("process gone");
    std::fs::remove_file(&table).expect("no table");
    let ((blind_code, _, blind_err), _) = run(&world.request());
    assert_eq!(blind_code, 1);
    assert!(blind_err.contains("could not be looked at") && blind_err.contains("Nothing was changed"), "{blind_err}");
    assert_eq!(world.open_files(), 2, "nothing was settled or removed");
    remove(world.root.as_path());
}

/// A program that carries `--journal` with a folder the check cannot resolve holds the command back,
/// and the message names
///  that program,
/// so a person with an unrelated tool that happens to take the option can tell what blocks them:
/// its process
///  ID,
/// the program it runs (the executable path of its first word) and the folder that did not resolve with the system's
///  reason,
/// why the program cannot be ruled out,
/// and what to do:
/// wait for it,
/// stop it,
/// or run the command again after it ends.
///  No other word of its command line is shown,
/// because any of them can be a secret that no masker knows (`--token`,
///  `--password`,
/// a URL's credentials);
/// the message says that they are not shown.
/// A folder the check cannot look up at all holds the command back;
/// a word that is absolute and names a
///  folder that is not there holds nothing back,
/// because such a folder holds no batch of this worktree.
#[test]
fn a_program_with_an_unresolvable_journal_is_named_by_program_and_folder_and_shows_no_other_argument() {
    let world: Setup = setup("settle-command-names-the-program");
    world.leave_open_batches(1);
    let words: Vec<String> = [
        "/usr/bin/unrelated-tool",
        "--journal",
        "gone/for/good",
        "--remote",
        "https://user:s3cret@example.invalid/x",
        "--token",
        "s3cret-token",
        "--password",
        "hunter2",
    ]
    .iter()
    .map(|word| { return String::from(*word); })
    .collect();
    run_process(&world, words.as_slice());
    let ((code, stdout, stderr), before) = run(&world.request());
    assert_eq!((code, stdout.as_str(), before.as_str()), (1, "", ""));
    assert!(stderr.contains(crate::earlier_service_process_look::Hold::HelperUnsure.words()), "the reason's words stay: {stderr}");
    for wanted in ["process 77", "`/usr/bin/unrelated-tool`", "`gone/for/good`", "No such file or directory", "cannot be ruled out", "wait for process 77", "stop it", "again", COMMAND, "Nothing was changed", "other arguments are not shown"] {
        assert!(stderr.contains(wanted), "{wanted}: {stderr}");
    }
    for secret in ["s3cret", "s3cret-token", "hunter2", "--token", "--password", "--remote", "example.invalid"] {
        assert!(!stderr.contains(secret), "{secret} of another argument must not be printed: {stderr}");
    }
    assert_eq!(stderr.matches('\n').count(), 1, "one line: {stderr:?}");
    run_process(&world, &[String::from("tool-by-name"), String::from("--journal"), String::from("two\nlines"), String::from("--token"), String::from("x")]);
    let ((named_code, _, named_err), _) = run(&world.request());
    assert_eq!(named_code, 1);
    assert!(named_err.contains("`tool-by-name`") && named_err.contains("`two\\nlines`"), "a bare program name, a control character escaped: {named_err}");
    assert_eq!(named_err.matches('\n').count(), 1, "one line: {named_err:?}");
    run_process(&world, &[String::from("/usr/bin/unrelated-tool"), String::from("--journal"), String::from("not/yet/there")]);
    let ((relative_code, _, relative_err), _) = run(&world.request());
    assert_eq!(relative_code, 1);
    assert!(relative_err.contains("`not/yet/there`") && relative_err.contains("working directory of process 77"), "a relative folder says what it is relative to: {relative_err}");
    std::fs::create_dir_all(world.root.join("exists")).expect("a folder that exists");
    run_process(&world, &[String::from("/usr/bin/unrelated-tool"), String::from("--journal"), world.root.join("exists").display().to_string()]);
    let ((resolved_code, _, resolved_err), _) = run(&world.request());
    assert_eq!(resolved_code, 1, "the command goes on and Git gives no answer for the fixture's batch, which it names: {resolved_err}");
    assert!(!resolved_err.contains("process 77"), "{resolved_err}");
    // A word that is absolute and names a folder that is not there names no folder that holds a batch of this worktree.
    run_process(&world, &[String::from("/usr/bin/unrelated-tool"), String::from("--journal"), world.root.join("gone/for/good").display().to_string()]);
    let ((absent_code, _, absent_err), _) = run(&world.request());
    assert_eq!(absent_code, 1, "the command goes on and Git gives no answer for the fixture's batch, which it names: {absent_err}");
    assert!(absent_err.contains("Some batches were not settled") && !absent_err.contains("process 77") && !absent_err.contains("Nothing was changed"), "a folder that is not there holds nothing back: {absent_err}");
    assert_eq!(world.open_files(), 1);
    remove(world.root.as_path());
}

/// The count of open batches agrees with the verb in every hold:
/// one batch "was" not settled,
/// several "were" not.
#[test]
fn the_held_message_agrees_in_number_with_the_count_of_open_batches() {
    for (count, wanted) in [(1_u64, "1 batch the earlier release's commit service left open for this worktree was not settled"), (2, "2 batches the earlier release's commit service left open for this worktree were not settled"), (3, "3 batches the earlier release's commit service left open for this worktree were not settled")] {
        let world: Setup = setup(format!("settle-command-number-{count}").as_str());
        world.leave_open_batches(count);
        run_process(&world, &[String::from("/usr/bin/unrelated-tool"), String::from("--journal"), String::from("gone/for/good")]);
        let ((code, _, stderr), _) = run(&world.request());
        assert_eq!(code, 1);
        assert!(stderr.starts_with(wanted), "{wanted}: {stderr}");
        remove(world.root.as_path());
    }
}

/// Before it acts,
/// the command says the condition it is safe under,
/// in plain words,
/// and what an earlier-release wrapper does
///  to it.
/// Here Git gives no answer for the batch,
/// so the batch is named as one the command could not examine,
/// stays on disk,
///  and the exit status says that something was not done.
#[test]
fn the_condition_is_said_before_the_command_acts_and_a_batch_git_cannot_answer_for_is_named() {
    let world: Setup = setup("settle-command-condition");
    world.leave_open_batches(1);
    let ((code, stdout, stderr), before) = run(&world.request());
    for wanted in ["no wrapper of the earlier release is in use in this worktree", "sets its commit service up again", "start the earlier helper"] {
        assert!(before.contains(wanted), "{wanted}: {before}");
    }
    assert_eq!(code, 1);
    for wanted in ["batch 1", "could not be examined", "stays on disk"] {
        assert!(stdout.contains(wanted), "{wanted}: {stdout}");
    }
    assert!(stderr.contains("nothing was discarded"), "{stderr}");
    assert_eq!(world.open_files(), 1);
    remove(world.root.as_path());
}

/// A writer that fails,
/// as standard error does when it is closed or its reader is gone.
struct FailingWriter {
    fail_flush_only: bool,
}

impl std::io::Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.fail_flush_only {
            return Ok(bytes.len());
        }
        return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "standard error is closed"));
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "standard error is closed"));
    }
}

/// The condition must reach the person before anything is changed:
/// when it cannot be written (or flushed),
/// the command refuses,
///  says so,
/// and has settled and examined nothing,
/// so no intent is touched and nothing is logged as examined.
#[test]
fn a_condition_that_cannot_be_delivered_stops_the_command_before_it_acts() {
    for fail_flush_only in [false, true] {
        let world: Setup = setup("settle-command-writer");
        world.leave_open_batches(1);
        let entries = crate::service_timing::capture::entries_of(|| {
            let action: Action = settle_earlier_batches_action(&world.request(), &mut FailingWriter { fail_flush_only });
            let (code, stdout, stderr) = parts(action);
            assert_eq!((code, stdout.as_str()), (1, ""));
            assert!(stderr.contains("could not be written") && stderr.contains("nothing was settled"), "{stderr}");
            assert!(!stderr.contains("could not be examined"), "the recovery did not start: {stderr}");
        });
        assert!(entries.iter().all(|entry| { return !entry.message.contains("Git gave no answer"); }), "{entries:?}");
        assert_eq!(world.open_files(), 1);
        remove(world.root.as_path());
    }
}

/// The doctor's note counts the open batches of the worktree without changing a file,
/// names the command,
/// says it only
///  reports,
/// and is empty when there is nothing to say or nothing to read.
#[test]
fn the_doctor_note_counts_the_batches_names_the_command_and_is_empty_otherwise() {
    let world: Setup = setup("settle-command-note");
    assert_eq!(earlier_batches_note(&Selection::Found(world.git_dir.clone()), Some(world.state.as_path())), "");
    world.leave_open_batches(2);
    let note: String = earlier_batches_note(&Selection::Found(world.git_dir.clone()), Some(world.state.as_path()));
    for wanted in ["2 batches", COMMAND, "nothing was changed"] {
        assert!(note.contains(wanted), "{wanted}: {note}");
    }
    assert_eq!(earlier_batches_note(&Selection::Outside, Some(world.state.as_path())), "");
    assert_eq!(earlier_batches_note(&Selection::Unresolved(String::from("x")), Some(world.state.as_path())), "");
    assert_eq!(earlier_batches_note(&Selection::Found(world.git_dir.clone()), None), "");
    assert_eq!(world.open_files(), 2);
    remove(world.root.as_path());
}

/// What:
///  A held batch the command settles completely is named in its output,
///  with the journal it was found in,
///  and the command exits 0.
/// Why:
///  The command's whole answer is one line per batch of the recovery's report,
///  so a batch missing from the report is printed nowhere:
///  here the person would read the safety condition,
///  then nothing at all,
///  and exit 0 after their held commit was moved onto the branch and the index was updated.
///  A retry cannot supply the missing line,
///  because no batch stays open
///  (a Kimi K3.1 Pro review of the jam-fix integration,
/// its finding that a fully settled held batch is omitted from the recovery report).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [[code, stdout]] = run(request); expect([code, stdout]).toEqual([0, `finished batch 1 in ${journal}\n`]);
/// ```
#[test]
fn a_held_batch_the_command_finishes_is_named_in_its_output() {
    let (world, repo) = setup_with_repository("settle-command-held-finished");
    let journal: PathBuf = world.state.join(DATA_FOLDER.earlier).join("service").join(world.instance());
    std::fs::create_dir_all(journal.as_path()).expect("the earlier journal");
    let held: crate::held_commit_record::Held = held_batch_in(journal.as_path(), repo.as_path());
    let ((code, stdout, stderr), _before) = run(&world.request());
    assert_eq!(stdout, format!("finished batch 1 in {}\n", journal.display()), "the batch the command finished is named");
    assert_eq!((code, stderr.as_str()), (0, ""), "nothing was left undone");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim(), held.new, "the held commit is on the branch");
    assert_eq!(git_text(repo.as_path(), &["status", "--porcelain=v1"]), "", "the index caught up with the landing");
    remove(world.root.as_path());
}
