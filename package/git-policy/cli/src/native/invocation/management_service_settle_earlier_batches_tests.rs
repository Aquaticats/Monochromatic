//! Controls for `service settle-earlier-batches` and the note `doctor` adds about it: what each answer says, that the
//!  condition of the command is said before it acts, and that nothing is changed while an earlier helper may run or start.

use super::*;
use crate::service_instance::instance_of;
use crate::service_journal::{Intent, OPEN_EXTENSION, batch_path, write_intent};
use crate::test_support::{REAL_GIT, fixture, isolated_overlay, remove};
use crate::wrapper_names_old_and_new::DATA_FOLDER;

/// The command word the notes name.
const COMMAND: &str = "git monochromatic-git-policy-cli service settle-earlier-batches";

/// One disposable world: a state folder, a Git directory, a process table.
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

impl Setup {
    fn instance(&self) -> String {
        return instance_of(self.git_dir.as_path());
    }

    /// Leaves `count` open batches of this worktree in its earlier journal, and one of another worktree beside them.
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
            paths: vec![String::from("a.txt")],
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

/// Runs the command, giving what it said before acting as well.
fn run(request: &SettleRequest<'_>) -> ((i32, String, String), String) {
    let mut before: Vec<u8> = Vec::new();
    let action: Action = settle_earlier_batches_action(request, &mut before);
    return (parts(action), String::from_utf8_lossy(before.as_slice()).into_owned());
}

/// The state folder is `XDG_STATE_HOME` when set and not empty, else `.local/state` of the home folder, else unknown.
#[test]
fn the_state_folder_follows_the_environment() {
    let pair = |name: &str, value: &str| -> (OsString, OsString) { return (OsString::from(name), OsString::from(value)) };
    assert_eq!(state_dir_of(&[pair("XDG_STATE_HOME", "/s"), pair("HOME", "/h")]), Some(PathBuf::from("/s")));
    assert_eq!(state_dir_of(&[pair("XDG_STATE_HOME", ""), pair("HOME", "/h")]), Some(PathBuf::from("/h/.local/state")));
    assert_eq!(state_dir_of(&[pair("HOME", "/h")]), Some(PathBuf::from("/h/.local/state")));
    assert_eq!(state_dir_of(&[pair("HOME", "")]), None);
    assert_eq!(state_dir_of(&[]), None);
}

/// With no batch left, the command says so and exits 0, and says nothing of a condition because it acts on nothing.
#[test]
fn with_no_batch_left_the_command_says_so_and_acts_on_nothing() {
    let world: Setup = setup("settle-command-nothing");
    let ((code, stdout, stderr), before) = run(&world.request());
    assert_eq!(code, 0);
    assert!(stdout.contains("No batch of the earlier release is left open for this worktree"), "{stdout}");
    assert_eq!((stderr.as_str(), before.as_str()), ("", ""));
    remove(world.root.as_path());
}

/// A command that cannot tell which worktree, or where the journals are, settles nothing and says why.
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

/// A socket that listens, a process that runs, and a look that cannot be made each hold the command back with their own
///  words, the count of open batches, what was not done, and what to do; nothing is changed and nothing was said as if acting.
#[test]
fn an_earlier_helper_that_may_run_holds_the_command_back_with_the_reason_and_the_way_on() {
    let world: Setup = setup("settle-command-held");
    world.leave_open_batches(2);
    let instance: String = world.instance();
    let table: PathBuf = world.proc_root.join("net/unix");
    let header: String = std::fs::read_to_string(&table).expect("header");
    std::fs::write(&table, format!("{header}0000000000000001: 00000002 00000000 00010000 0001 01 1 /run/user/1000/cli-git/{instance}.sock\n")).expect("a listening socket");
    let ((socket_code, socket_out, socket_err), before) = run(&world.request());
    assert_eq!((socket_code, socket_out.as_str(), before.as_str()), (1, "", ""));
    for wanted in ["2 batches", "listens", "Nothing was changed", "systemctl --user stop", "three hours", COMMAND] {
        assert!(socket_err.contains(wanted), "{wanted}: {socket_err}");
    }
    std::fs::write(&table, header).expect("an empty table");
    let process: PathBuf = world.proc_root.join("77");
    std::fs::create_dir_all(&process).expect("process");
    std::fs::write(process.join("cmdline"), format!("/p/cli-git\0cli-git\0service\0run\0--instance\0{instance}\0")).expect("command line");
    let ((process_code, _, process_err), _) = run(&world.request());
    assert_eq!(process_code, 1);
    assert!(process_err.contains("runs as the earlier helper") && process_err.contains("Nothing was changed"), "{process_err}");
    std::fs::remove_file(&table).expect("no table");
    let ((blind_code, _, blind_err), _) = run(&world.request());
    assert_eq!(blind_code, 1);
    assert!(blind_err.contains("could not be looked at") && blind_err.contains("Nothing was changed"), "{blind_err}");
    assert_eq!(world.open_files(), 2, "nothing was settled or removed");
    remove(world.root.as_path());
}

/// Before it acts, the command says the condition it is safe under, in plain words, and what an earlier-release wrapper does
///  to it. Here Git gives no answer for the batch, so the batch stays open and nothing is discarded.
#[test]
fn the_condition_is_said_before_the_command_acts_and_a_batch_git_cannot_answer_for_stays_open() {
    let world: Setup = setup("settle-command-condition");
    world.leave_open_batches(1);
    let ((code, _, stderr), before) = run(&world.request());
    for wanted in ["no wrapper of the earlier release is in use in this worktree", "sets its commit service up again", "start the earlier helper"] {
        assert!(before.contains(wanted), "{wanted}: {before}");
    }
    assert_eq!(code, 1);
    assert!(stderr.contains("could not be settled now") && stderr.contains("nothing was discarded"), "{stderr}");
    assert_eq!(world.open_files(), 1);
    remove(world.root.as_path());
}

/// The doctor's note counts the open batches of the worktree without changing a file, names the command, says it only
///  reports, and is empty when there is nothing to say or nothing to read.
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
