//! Controls for the fold of every kept name that exists under both spellings,
//! each in its own disposable folders and with a
//!  process table of its own,
//! so no control looks at this machine's processes or touches its Git directories.

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
    KeptName { name: PUSH_TRIGGER_LOG, kind: KeptKind::Log },
    KeptName { name: COMMIT_TIMING_LOG, kind: KeptKind::Log },
];

/// A stop that ends the fold when it has been told of `limit` steps,
/// as a kill at that instant would.
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

/// A Git directory with every kept name under both spellings,
/// and a process table with no process in it.
struct World {
    /// The Git directory.
    git_dir: PathBuf,
    /// The surroundings the fold looks at.
    surroundings: FoldSurroundings,
}

/// Plant both spellings of the three kinds,
/// and an empty process table.
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

/// What:
///  The steps a fold with no kill takes,
/// in order,
/// and a fold that never stops.
/// Why:
///   One fold of the fixture says how many steps there are before the kill points are walked,
/// so each limit can be checked against the step it is meant to land on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names: string[] = []; // foldKeptNames(dir, names, surroundings, { mayContinue: (step) => { names.push(step); return true; } })
/// ```
struct RecordingSteps {
    /// Every step the fold reached,
    /// in order.
    names: Vec<&'static str>,
}

impl FoldSteps for RecordingSteps {
    fn may_continue(&mut self, step: &'static str) -> bool {
        self.names.push(step);
        return true;
    }
}

/// The steps of one fold with no kill,
/// from a fixture of its own,
/// so that every kill point can be named.
fn steps_of_one_fold() -> Vec<&'static str> {
    // The sweep's subject is the fold's steps and their outcomes, not what a crash leaves, so its syncs are no-ops here.
    crate::private_storage::skip_the_durability_syncs_here();
    let world: World = world("kill-points");
    let mut recording: RecordingSteps = RecordingSteps { names: Vec::new() };
    let report: FoldReport = fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut recording);
    assert!(report.outcomes.iter().all(|(_, outcome)| return !matches!(outcome, Outcome::Stopped(_))), "nothing stops this fold: {report:?}");
    std::fs::remove_dir_all(world.git_dir.parent().expect("root")).expect("remove only the fixture");
    return recording.names;
}

/// One kill point of the whole fold:
/// how many steps may pass,
/// and the steps of a fold that no kill interrupts.
struct KillPoint<'a> {
    /// How many steps may pass before the fold ends where a kill would have ended it.
    limit: usize,
    /// Every step of the whole fold,
    /// in order,
    /// so `limit` names the step this kill lands on.
    steps: &'a [&'static str],
}

/// What:
///  End the fold at one step,
/// fold again,
/// and check that everything ended where a fold with no kill puts it.
/// Why:
///   See the control that walks every step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// killThenFold({ limit, steps }); // one kill point of the sweep
/// ```
fn kill_then_fold(point: KillPoint<'_>) {
    let KillPoint { limit, steps } = point;
    // The stop this point makes is the fold's own step boundary and not a crash, so what the fold leaves on the disk is not
    //  what this control asks about, and its syncs are no-ops here.
    crate::private_storage::skip_the_durability_syncs_here();
    let world: World = world(format!("kill-{limit}").as_str());
    let mut stop: StopAfter = StopAfter { limit, seen: 0 };
    let first: FoldReport = fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut stop);
    let stopped: Option<&'static str> = first.outcomes.last().and_then(|(_, outcome)| {
        return match outcome {
            Outcome::Stopped(step) => Some(*step),
            _ => None,
        };
    });
    // Each limit below the number of steps ends the first fold at exactly that step, so every kill point of the whole fold is
    //  walked once and none twice; the last limit lets every step pass, which is the fold no kill interrupts.
    if limit < steps.len() {
        assert_eq!(stopped, Some(steps[limit]), "the kill at step {limit} lands on that step: {first:?}");
    } else {
        assert_eq!(stopped, None, "the last limit lets every step pass: {first:?}");
    }
    fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut NeverStopped);
    assert_folded(&world);
    std::fs::remove_dir_all(world.git_dir.parent().expect("root")).expect("remove only the fixture");
}

/// With no earlier program running,
/// every name that exists twice is folded and nothing is held or reported.
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

/// An earlier commit service of the worktree is running:
/// nothing is folded,
/// every byte stays,
/// and the hold says why;
///  once it stops,
/// the next command folds.
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

/// A socket that listens for the earlier service,
/// with no process behind it,
/// does not hold the fold:
/// such a socket stays for
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

/// A kill at every step of the whole fold,
/// then one more fold:
/// everything ends up where a fold with no kill puts it.
#[test]
fn a_kill_at_any_step_of_the_whole_fold_then_a_second_fold_gives_the_same_result() {
    let steps: Vec<&'static str> = steps_of_one_fold();
    assert!(!steps.is_empty(), "the fixture's fold has steps to kill it at");
    // A kill point shares nothing with another: each has its own fixture, its own process table and its own surroundings,
    //  so the points walk side by side.
    // Their folds sync nothing, because what a crash leaves is not what this sweep asks:
    // measured on 2026-11-08 with `strace --summary-only --trace=fsync`,
    // one walk of the 21 kill points made 306 `fsync` calls and spent 2.999 s inside them of a 10.61 s run,
    // with single calls 1.755, 1.211 and 1.071 s apart,
    // and a disk having a bad minute is what once carried the same walk past the run's own 120 s bound.
    // The fixtures, the checks and the removals together are under 1 ms per point.
    let points: &[&'static str] = steps.as_slice();
    let mut failed: Vec<usize> = Vec::new();
    std::thread::scope(|scope| {
        let handles: Vec<std::thread::ScopedJoinHandle<'_, ()>> = (0..=points.len())
            .map(|limit| { return scope.spawn(move || { kill_then_fold(KillPoint { limit, steps: points }); }); })
            .collect();
        for (limit, handle) in handles.into_iter().enumerate() {
            if handle.join().is_err() {
                failed.push(limit);
            }
        }
    });
    assert!(failed.is_empty(), "the kill points that failed: {failed:?}");
}

/// An earlier program that still has a log open for writing:
/// that name is left for a later command with no line to the person,
/// and
///  the later command,
/// run once the program has written its last line and closed the log,
/// folds the line too.
#[test]
fn a_log_that_an_earlier_program_still_writes_is_left_silently_and_folded_after_it_closes() {
    use std::io::Write;
    let world: World = world("writer-open");
    let mut writer = std::fs::OpenOptions::new().append(true).open(world.git_dir.join(COMMIT_TIMING_LOG.earlier)).expect("writer");
    let report: FoldReport = fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut NeverStopped);
    let timing: Option<&Outcome> = report.outcomes.iter().find(|(name, _)| return name.current == COMMIT_TIMING_LOG.current).map(|(_, outcome)| return outcome);
    assert_eq!(timing, Some(&Outcome::LeftForALaterCommand), "{report:?}");
    assert_eq!(failure_line(world.git_dir.as_path(), COMMIT_TIMING_LOG, &Outcome::LeftForALaterCommand), None, "nothing is printed for it");
    writer.write_all(b"late-timing\n").expect("late line");
    drop(writer);
    let later: FoldReport = fold_kept_names(world.git_dir.as_path(), &NAMES, &world.surroundings, &mut NeverStopped);
    assert!(later.outcomes.iter().all(|(_, outcome)| return *outcome == Outcome::Folded), "{later:?}");
    assert_eq!(std::fs::read_to_string(world.git_dir.join(COMMIT_TIMING_LOG.current)).expect("current"), "current-timing\nearlier-timing\nlate-timing\n");
    std::fs::remove_dir_all(world.git_dir.parent().expect("root")).expect("remove only the fixture");
}

/// A read-only Git directory:
/// nothing is folded,
/// nothing is lost,
/// each name reports a failure,
/// and the failure line is one line.
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

/// A kept name that exists once needs no fold,
/// and the look at processes is never made for it.
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

/// The numbered older copies of an earlier rotating log go beside the current ones,
/// and none replaces another.
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

/// The numbered older copies of an earlier push trigger log go beside the current ones too:
/// every log of the wrapper keeps generations, so the copies the rename left behind are history a reader needs.
#[test]
fn numbered_copies_of_an_earlier_push_trigger_log_are_kept_beside_the_current_ones() {
    let world: World = world("numbered-push");
    let git: &Path = world.git_dir.as_path();
    std::fs::write(git.join(format!("{}.1", PUSH_TRIGGER_LOG.earlier)), "earlier copy").expect("earlier copy");
    std::fs::write(git.join(format!("{}.1", PUSH_TRIGGER_LOG.current)), "current copy").expect("current copy");
    fold_kept_names(git, &NAMES, &world.surroundings, &mut NeverStopped);
    assert_eq!(std::fs::read_to_string(git.join(format!("{}.1", PUSH_TRIGGER_LOG.current))).expect("kept"), "current copy");
    assert_eq!(
        std::fs::read_to_string(git.join(format!("{}.1{KEPT_BESIDE_SUFFIX}", PUSH_TRIGGER_LOG.current))).expect("beside"),
        "earlier copy"
    );
    std::fs::remove_dir_all(git.parent().expect("root")).expect("remove only the fixture");
}
