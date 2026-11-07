//! Controls for the fold of every kept name that exists under both spellings, each in its own disposable folders and with a
//!  process table of its own, so no control looks at this machine's processes or touches its Git directories.

use super::*;
use crate::service_instance::instance_of;
use crate::wrapper_names_old_and_new::{COMMIT_TIMING_LOG, DATA_FOLDER, PUSH_TRIGGER_LOG, WORKTREE_COPY_RECORD_FOLDER};
use std::path::PathBuf;

/// One empty fixture folder for a control.
fn fixture(name: &str) -> PathBuf {
    return crate::test_support::fixture_with_prefix("native-names-fold", name);
}

/// The names the controls fold.
const NAMES: [KeptName; 3] = [
    KeptName { name: DATA_FOLDER, kind: KeptKind::Folder },
    KeptName { name: PUSH_TRIGGER_LOG, kind: KeptKind::Log { rotates: false } },
    KeptName { name: COMMIT_TIMING_LOG, kind: KeptKind::Log { rotates: true } },
];

/// A stop that ends the fold when it has been told of `limit` steps, as a kill at that instant would.
struct StopAfter {
    /// How many steps may pass.
    limit: usize,
    /// How many steps have been told.
    seen: usize,
}

impl FoldSteps for StopAfter {
    fn may_continue(&mut self, _step: &'static str) -> bool {
        self.seen += 1;
        return self.seen <= self.limit;
    }
}

/// A Git directory with every kept name under both spellings, and a process table with no process in it.
struct World {
    /// The Git directory.
    git_dir: PathBuf,
    /// The surroundings the fold looks at.
    surroundings: FoldSurroundings,
}

/// Plant both spellings of the three kinds, and an empty process table.
fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let git_dir: PathBuf = root.join("git");
    let proc_root: PathBuf = root.join("proc");
    std::fs::create_dir_all(git_dir.join("cli-git/work/done")).expect("earlier folder");
    std::fs::create_dir_all(git_dir.join("monochromatic-git-policy-cli/work/done")).expect("current folder");
    std::fs::create_dir_all(proc_root.as_path()).expect("process table");
    std::fs::write(git_dir.join("cli-git/work/done/earlier"), "earlier record").expect("earlier record");
    std::fs::write(git_dir.join("monochromatic-git-policy-cli/work/done/current"), "current record").expect("current record");
    std::fs::write(git_dir.join(PUSH_TRIGGER_LOG.earlier), "earlier-push\n").expect("earlier log");
    std::fs::write(git_dir.join(PUSH_TRIGGER_LOG.current), "current-push\n").expect("current log");
    std::fs::write(git_dir.join(COMMIT_TIMING_LOG.earlier), "earlier-timing\n").expect("earlier log");
    std::fs::write(git_dir.join(COMMIT_TIMING_LOG.current), "current-timing\n").expect("current log");
    return World { git_dir, surroundings: FoldSurroundings { state_dir: None, proc_root } };
}

/// Plant a process whose start arguments are the earlier commit service of the world's Git directory.
fn start_earlier_service(world: &World) {
    let instance: String = instance_of(world.git_dir.as_path());
    let process: PathBuf = world.surroundings.proc_root.join("4242");
    std::fs::create_dir_all(process.as_path()).expect("process");
    let mut bytes: Vec<u8> = Vec::new();
    for word in ["/home/user/.cache/cli-git-bin/cli-git-native", "cli-git", "service", "run", "--instance", instance.as_str()] {
        bytes.extend(word.as_bytes());
        bytes.push(0);
    }
    std::fs::write(process.join("cmdline"), bytes).expect("command line");
}

/// Everything is where it should be after a fold.
fn assert_folded(world: &World) {
    let git: &Path = world.git_dir.as_path();
    for earlier in [DATA_FOLDER.earlier, PUSH_TRIGGER_LOG.earlier, COMMIT_TIMING_LOG.earlier] {
        assert!(!git.join(earlier).exists(), "{earlier} is gone");
    }
    assert_eq!(std::fs::read_to_string(git.join("monochromatic-git-policy-cli/work/done/earlier")).expect("moved"), "earlier record");
    assert_eq!(std::fs::read_to_string(git.join("monochromatic-git-policy-cli/work/done/current")).expect("kept"), "current record");
    assert_eq!(std::fs::read_to_string(git.join(PUSH_TRIGGER_LOG.current)).expect("log"), "current-push\nearlier-push\n");
    assert_eq!(std::fs::read_to_string(git.join(COMMIT_TIMING_LOG.current)).expect("log"), "current-timing\nearlier-timing\n");
}

/// With no earlier program running, every name that exists twice is folded and nothing is held or reported.
#[test]
fn with_no_earlier_program_running_every_name_is_folded() {
    let world: World = world("folds");
    let report: FoldReport = fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut NeverStopped);
    assert_eq!(report.held, None);
    assert_eq!(report.outcomes.len(), NAMES.len());
    assert!(report.outcomes.iter().all(|(_, outcome)| return *outcome == Outcome::Folded), "{report:?}");
    assert_folded(&world);
    let again: FoldReport = fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut NeverStopped);
    assert_eq!(again, FoldReport { held: None, outcomes: Vec::new() }, "a folded directory costs a look at names and nothing else");
    std::fs::remove_dir_all(world.git_dir.parent().expect("root")).expect("remove only the fixture");
}

/// An earlier commit service of the worktree is running: nothing is folded, every byte stays, and the hold says why;
///  once it stops, the next command folds.
#[test]
fn an_earlier_program_that_runs_holds_the_fold_until_it_stops() {
    let world: World = world("held");
    start_earlier_service(&world);
    let report: FoldReport = fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut NeverStopped);
    assert_eq!(report.held, Some(Hold::InstanceHelperRuns));
    assert!(report.outcomes.is_empty());
    assert_eq!(std::fs::read_to_string(world.git_dir.join(PUSH_TRIGGER_LOG.earlier)).expect("earlier"), "earlier-push\n");
    assert_eq!(std::fs::read_to_string(world.git_dir.join(PUSH_TRIGGER_LOG.current)).expect("current"), "current-push\n");
    assert!(world.git_dir.join("cli-git/work/done/earlier").exists());
    let line: String = waiting_line(world.git_dir.as_path(), Hold::InstanceHelperRuns);
    assert!(line.ends_with('\n') && line.matches('\n').count() == 1, "one line: {line:?}");
    assert!(line.contains("systemctl --user stop"), "{line}");
    std::fs::remove_dir_all(world.surroundings.proc_root.join("4242")).expect("the program stops");
    let later: FoldReport = fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut NeverStopped);
    assert_eq!(later.held, None);
    assert_folded(&world);
    std::fs::remove_dir_all(world.git_dir.parent().expect("root")).expect("remove only the fixture");
}

/// A socket that listens for the earlier service, with no process behind it, does not hold the fold: such a socket stays for
///  good and writes nothing.
#[test]
fn a_listening_socket_without_a_process_does_not_hold_the_fold() {
    let world: World = world("socket-only");
    let instance: String = instance_of(world.git_dir.as_path());
    let table: PathBuf = world.surroundings.proc_root.join("net/unix");
    std::fs::create_dir_all(table.parent().expect("net")).expect("net folder");
    std::fs::write(
        table,
        format!("Num RefCount Protocol Flags Type St Inode Path\n0: 2 0 00010000 1 1 1 /run/user/1000/cli-git/{instance}.sock\n"),
    )
    .expect("socket table");
    let report: FoldReport = fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut NeverStopped);
    assert_eq!(report.held, None);
    assert_folded(&world);
    std::fs::remove_dir_all(world.git_dir.parent().expect("root")).expect("remove only the fixture");
}

/// A kill at every step of the whole fold, then one more fold: everything ends up where a fold with no kill puts it.
#[test]
fn a_kill_at_any_step_of_the_whole_fold_then_a_second_fold_gives_the_same_result() {
    let mut limit: usize = 0;
    loop {
        let world: World = world(format!("kill-{limit}").as_str());
        let first: FoldReport = fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut StopAfter { limit, seen: 0 });
        let finished: bool = first.outcomes.iter().all(|(_, outcome)| return !matches!(outcome, Outcome::Stopped(_))) && first.outcomes.len() == NAMES.len();
        fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut NeverStopped);
        assert_folded(&world);
        std::fs::remove_dir_all(world.git_dir.parent().expect("root")).expect("remove only the fixture");
        if finished {
            break;
        }
        limit += 1;
    }
}

/// A read-only Git directory: nothing is folded, nothing is lost, each name reports a failure, and the failure line is one line.
#[cfg(unix)]
#[test]
fn a_read_only_git_directory_fails_every_name_and_loses_nothing() {
    use std::os::unix::fs::PermissionsExt;
    let world: World = world("read-only");
    let git: PathBuf = world.git_dir.clone();
    std::fs::set_permissions(git.as_path(), std::fs::Permissions::from_mode(0o500)).expect("read-only");
    let probe = std::fs::File::create(git.join("probe"));
    let report: FoldReport = fold_kept_names(git.as_path(), &NAMES, &world.surroundings, &mut NeverStopped);
    std::fs::set_permissions(git.as_path(), std::fs::Permissions::from_mode(0o700)).expect("writable again");
    if probe.is_ok() {
        // The control: this user can write there anyway, so the case cannot be made.
        std::fs::remove_dir_all(git.parent().expect("root")).expect("remove only the fixture");
        return;
    }
    assert_eq!(report.held, None);
    // The folder's entries move below the folders, which stay writable; only the removal of the emptied earlier folder fails.
    for (name, outcome) in &report.outcomes {
        assert!(matches!(outcome, Outcome::Failed(_)), "{name:?}: {outcome:?}");
        if name.current == DATA_FOLDER.current {
            let Outcome::Failed(message) = outcome else { unreachable!("matched above") };
            assert!(message.contains("could not be removed"), "the line names the removal, not a name collision: {message}");
            assert!(!message.contains("already in use"), "{message}");
        }
        let line: String = failure_line(git.as_path(), *name, outcome).expect("a failure is reported");
        assert!(line.ends_with('\n') && line.matches('\n').count() == 1 && line.contains("tries again"), "{line:?}");
    }
    assert_eq!(std::fs::read_to_string(git.join(PUSH_TRIGGER_LOG.earlier)).expect("earlier"), "earlier-push\n");
    assert_eq!(std::fs::read_to_string(git.join(COMMIT_TIMING_LOG.earlier)).expect("earlier"), "earlier-timing\n");
    assert_eq!(std::fs::read_to_string(git.join(PUSH_TRIGGER_LOG.current)).expect("current"), "current-push\n");
    assert_eq!(std::fs::read_to_string(git.join("monochromatic-git-policy-cli/work/done/earlier")).expect("moved"), "earlier record");
    std::fs::remove_dir_all(git.parent().expect("root")).expect("remove only the fixture");
}

/// A kept name that exists once needs no fold, and the look at processes is never made for it.
#[test]
fn a_name_that_exists_once_costs_no_look() {
    let root: PathBuf = fixture("once");
    std::fs::create_dir_all(root.join(WORKTREE_COPY_RECORD_FOLDER.current)).expect("current only");
    // A process table that cannot be read would hold the fold if the look were made.
    let surroundings: FoldSurroundings = FoldSurroundings { state_dir: None, proc_root: root.join("no-such-proc") };
    let names: [KeptName; 1] = [KeptName { name: WORKTREE_COPY_RECORD_FOLDER, kind: KeptKind::Folder }];
    let report: FoldReport = fold_kept_names(root.as_path(), &names, &surroundings, &mut NeverStopped);
    assert_eq!(report, FoldReport { held: None, outcomes: Vec::new() });
    std::fs::remove_dir_all(root.as_path()).expect("remove only the fixture");
}

/// The numbered older copies of an earlier rotating log go beside the current ones, and none replaces another.
#[test]
fn numbered_copies_of_an_earlier_rotating_log_are_kept_beside_the_current_ones() {
    let world: World = world("numbered");
    let git: &Path = world.git_dir.as_path();
    std::fs::write(git.join(format!("{}.1", COMMIT_TIMING_LOG.earlier)), "earlier copy").expect("earlier copy");
    std::fs::write(git.join(format!("{}.2", COMMIT_TIMING_LOG.earlier)), "earlier second copy").expect("earlier copy");
    std::fs::write(git.join(format!("{}.1", COMMIT_TIMING_LOG.current)), "current copy").expect("current copy");
    fold_kept_names(git, &NAMES, &world.surroundings, &mut NeverStopped);
    assert_eq!(std::fs::read_to_string(git.join(format!("{}.1", COMMIT_TIMING_LOG.current))).expect("kept"), "current copy");
    assert_eq!(
        std::fs::read_to_string(git.join(format!("{}.1{KEPT_BESIDE_SUFFIX}", COMMIT_TIMING_LOG.current))).expect("beside"),
        "earlier copy"
    );
    assert_eq!(std::fs::read_to_string(git.join(format!("{}.2", COMMIT_TIMING_LOG.current))).expect("moved"), "earlier second copy");
    std::fs::remove_dir_all(git.parent().expect("root")).expect("remove only the fixture");
}
