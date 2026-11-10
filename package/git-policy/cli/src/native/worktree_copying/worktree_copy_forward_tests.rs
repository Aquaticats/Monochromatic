//! Forwarding-run checks against real Git:
//! which invocations the copy applies to,
//! and how a settled run ends.

use super::*;
use crate::test_support::{REAL_GIT, fixture, git, isolated_overlay, remove, repository};
use std::path::Path;

/// A repository with an ignored rules file,
/// returned with its canonical root and common directory.
fn source(root: &Path) -> (PathBuf, PathBuf) {
    let main: PathBuf = repository(root, "main");
    std::fs::write(main.join(".gitignore"), b".cache/\n").expect("gitignore");
    std::fs::create_dir_all(main.join(".cache")).expect("cache");
    std::fs::write(main.join(".cache/rules.txt"), b"rules").expect("rules");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    return (std::fs::canonicalize(main).expect("canonical main"), common);
}

/// The identity of the main worktree of a source.
fn main_identity(main: &Path, common: &Path) -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: common.to_path_buf(),
        git_dir: common.to_path_buf(),
        worktree_root: main.to_path_buf(),
    };
}

/// Text arguments as owned operating-system arguments.
fn arguments(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(OsString::from).collect();
}

/// The copy applies to linked and bare repositories always,
/// to the main checkout only for a creation,
/// and never outside a repository.
#[test]
fn the_copy_applies_by_repository_shape() {
    let main: WorktreeIdentity = main_identity(Path::new("/r"), Path::new("/r/.git"));
    let linked: WorktreeIdentity = WorktreeIdentity::LinkedWorktree {
        common_dir: PathBuf::from("/r/.git"),
        git_dir: PathBuf::from("/r/.git/worktrees/w"),
        worktree_root: PathBuf::from("/w"),
    };
    let bare: WorktreeIdentity = WorktreeIdentity::BareRepository {
        common_dir: PathBuf::from("/b.git"),
        git_dir: PathBuf::from("/b.git"),
    };
    for creates in [true, false] {
        assert!(copy_applies(&linked, creates), "linked {creates}");
        assert!(copy_applies(&bare, creates), "bare {creates}");
        assert!(!copy_applies(&WorktreeIdentity::OutsideWorktree, creates));
    }
    assert!(copy_applies(&main, true));
    assert!(!copy_applies(&main, false), "a main checkout command that creates nothing");
}

/// Without a copy the run is plain Git:
/// its exit status passes through,
/// and an unstartable Git is an engine failure.
#[test]
fn a_plain_run_reports_git_status_or_the_start_failure() {
    let root: PathBuf = fixture("forward-plain");
    let (main, _common) = source(root.as_path());
    assert_eq!(
        run_forwarded_git(
            Path::new(REAL_GIT),
            arguments(&["-C", main.to_str().expect("utf8"), "rev-parse", "--git-dir"]).as_slice(),
            isolated_overlay().as_slice(),
            None,
        ),
        ForwardedRun::Ran(0)
    );
    assert_eq!(
        run_forwarded_git(
            Path::new(REAL_GIT),
            arguments(&["no-such-subcommand"]).as_slice(),
            isolated_overlay().as_slice(),
            None,
        ),
        ForwardedRun::Ran(1)
    );
    match run_forwarded_git(
        Path::new("/nonexistent/git"),
        arguments(&["status"]).as_slice(),
        &[],
        None,
    ) {
        ForwardedRun::Failed { stderr, code } => {
            assert_eq!(code, 2);
            assert!(stderr.starts_with("monochromatic-git-policy-cli: cannot start /nonexistent/git: "), "{stderr}");
        }
        other => panic!("expected a start failure, got {other:?}"),
    }
    remove(root.as_path());
}

/// A settled creation from the main checkout runs Git,
/// copies its ignored state,
/// and keeps Git's status.
#[cfg(unix)]
#[test]
fn a_settled_creation_from_the_main_checkout_copies_its_ignored_state() {
    let root: PathBuf = fixture("forward-settled");
    let (main, common) = source(root.as_path());
    let topic: PathBuf = root.join("topic");
    let facts: CopyFacts = CopyFacts {
        identity: main_identity(main.as_path(), common.as_path()),
        creates: true,
        destination: Some(topic.clone()),
        add_plan: None,
        remove_plan: None,
    };
    let created: ForwardedRun = run_forwarded_git(
        Path::new(REAL_GIT),
        arguments(&[
            "-C",
            main.to_str().expect("utf8"),
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            topic.to_str().expect("utf8"),
        ])
        .as_slice(),
        isolated_overlay().as_slice(),
        Some(&facts),
    );
    assert_eq!(created, ForwardedRun::Ran(0));
    assert_eq!(std::fs::read(topic.join(".cache/rules.txt")).expect("copied"), b"rules");
    // Git's own failure keeps its status and copies nothing.
    let failed: ForwardedRun = run_forwarded_git(
        Path::new(REAL_GIT),
        arguments(&[
            "-C",
            main.to_str().expect("utf8"),
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            root.join("again").to_str().expect("utf8"),
        ])
        .as_slice(),
        isolated_overlay().as_slice(),
        Some(&facts),
    );
    assert_eq!(failed, ForwardedRun::Ran(255));
    assert!(!root.join("again").exists());
    remove(root.as_path());
}

/// A settled run whose repository cannot be asked about again ends as an engine failure with the reason.
#[cfg(unix)]
#[test]
fn a_settled_run_that_cannot_ask_git_fails_with_the_reason() {
    let root: PathBuf = fixture("forward-unstartable");
    let (main, common) = source(root.as_path());
    git(main.as_path(), &["config", "user.name", "t"]);
    let facts: CopyFacts = CopyFacts {
        identity: main_identity(main.as_path(), common.as_path()),
        creates: true,
        destination: None,
        add_plan: None,
        remove_plan: None,
    };
    match run_forwarded_git(
        Path::new("/nonexistent/git"),
        arguments(&["worktree", "list"]).as_slice(),
        &[],
        Some(&facts),
    ) {
        ForwardedRun::Failed { stderr, code } => {
            assert_eq!(code, 2);
            assert!(stderr.ends_with('\n'), "{stderr}");
            assert!(stderr.contains("/nonexistent/git"), "{stderr}");
        }
        other => panic!("expected a failure, got {other:?}"),
    }
    remove(root.as_path());
}

/// A forwarded removal stops the units of the worktree it removed:
/// the instance is the one the registration the removal deletes names,
/// and it is asked for only after the removal ended well.
#[cfg(unix)]
#[test]
fn a_forwarded_removal_asks_for_the_removed_worktrees_units_to_be_stopped() {
    let root: PathBuf = fixture("forward-removal-units");
    let (main, common) = source(root.as_path());
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical worktree");
    let registered: PathBuf = std::fs::canonicalize(common.join("worktrees/wt")).expect("registration");
    let facts: CopyFacts = CopyFacts {
        identity: main_identity(main.as_path(), common.as_path()),
        creates: false,
        destination: None,
        add_plan: None,
        remove_plan: Some(RemovePlan { target: worktree.clone(), operand: worktree.clone().into_os_string() }),
    };
    let asked: std::cell::RefCell<Vec<WorktreeAboutToGo>> = std::cell::RefCell::new(Vec::new());
    let finished: ForwardedRun = run_forwarded_git_with_unit_stop(
        Path::new(REAL_GIT),
        arguments(&[
            "-C",
            main.to_str().expect("utf8"),
            "worktree",
            "remove",
            "--force",
            worktree.to_str().expect("utf8"),
        ])
        .as_slice(),
        isolated_overlay().as_slice(),
        Some(&facts),
        &|about: &WorktreeAboutToGo| asked.borrow_mut().push(about.clone()),
    );
    assert_eq!(finished, ForwardedRun::Ran(0), "the removal itself must end well");
    assert!(!worktree.exists(), "the removal took the worktree");
    let stops: Vec<WorktreeAboutToGo> = asked.into_inner();
    assert_eq!(stops.len(), 1, "one removal, one stop");
    assert_eq!(stops[0].git_dir, registered);
    assert_eq!(stops[0].instance, crate::service_instance::instance_of(registered.as_path()));
    remove(root.as_path());
}

/// What:
///  A ladder step that waits on the checkout's conversion filter is ended at its ceiling,
/// so the add returns
///  instead of waiting for the filter.
/// Why:
///   The wiring is what this proves:
/// the `read-tree` of a rung and the fallback `git reset --hard` are started by
///  the forwarded runner,
/// and a smudge filter that never answers must not hold the add (owner:
/// "nothing should ever wait
///  forever or indefinitely").
/// The attribute selects a filter whose driver is not the standard LFS definition,
/// so `prepare_target`
///  refuses the rung and the fallback checkout is the step that would run the filter,
/// and the filter is a
///  `git-lfs` on the overlay's `PATH` that never answers.
/// The source is a subvolume root,
/// because no rung serves a source that is not one,
/// and the ceiling is
///  shortened for this thread,
/// so the control ends in a moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shortenCeiling(500); const began = now(); const run = runForwardedGit(...); expect(now() - began).toBeLessThan(20_000)
/// ```
#[cfg(target_os = "linux")]
#[test]
fn btrfs_a_ladder_step_that_waits_on_a_filter_ends_at_its_ceiling() {
    use crate::fs_btrfs_ioctl::subvolume_create_at;
    use crate::fs_btrfs_test_support::btrfs_fixture;
    use crate::ladder_step_ceilings::CEILING_FOR_THIS_THREAD;
    use crate::worktree_ladder_test_support::{commit_all, tracked};
    use std::time::Duration;
    let Some(root) = btrfs_fixture("forward-ladder-ceiling") else { return };
    let source_root: PathBuf = root.join("main");
    subvolume_create_at(source_root.as_path()).expect("the source is a subvolume root");
    git(source_root.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    tracked(source_root.as_path(), ".gitignore", b".cache/\n");
    tracked(source_root.as_path(), ".gitattributes", b"stored.txt filter=lfs\n");
    tracked(source_root.as_path(), "stored.txt", b"one\n");
    commit_all(source_root.as_path(), "first");
    git(source_root.as_path(), &["checkout", "--quiet", "-b", "other"]);
    // A different length, so the aged file cannot pass for the committed one by size and time alone.
    tracked(source_root.as_path(), "stored.txt", b"two, longer\n");
    commit_all(source_root.as_path(), "second");
    git(source_root.as_path(), &["checkout", "--quiet", "main"]);
    let main: PathBuf = std::fs::canonicalize(source_root.as_path()).expect("canonical main");
    let common: PathBuf = main.join(".git");
    // A smudge filter that never answers, on the overlay's PATH, and a definition `prepare_target` refuses: the rung ends
    // and the fallback checkout is the step that would run it. No `clean` command is configured, so the clean check does not
    // start the filter.
    let filters: PathBuf = root.join("filters");
    std::fs::create_dir_all(filters.as_path()).expect("filter directory");
    crate::test_support::executable(filters.join("git-lfs").as_path(), b"#!/bin/sh\nsleep 5\n");
    git(main.as_path(), &["config", "filter.lfs.clean", "cat"]);
    git(main.as_path(), &["config", "filter.lfs.smudge", "git-lfs smudge -- %f"]);
    git(main.as_path(), &["config", "filter.lfs.required", "true"]);
    let topic: PathBuf = root.join("topic");
    let facts: CopyFacts = CopyFacts {
        identity: main_identity(main.as_path(), common.as_path()),
        creates: true,
        destination: Some(topic.clone()),
        add_plan: Some(AddPlan { destination: topic.clone(), insert_at: 4, quiet: true }),
        remove_plan: None,
    };
    let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
    overlay.push((OsString::from("PATH"), OsString::from(format!("{}:/usr/bin:/bin", filters.display()))));
    CEILING_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<Duration>>| slot.set(Some(Duration::from_millis(500))));
    let run: ForwardedRun = run_forwarded_git(
        Path::new(REAL_GIT),
        arguments(&[
            "-C",
            main.to_str().expect("utf8"),
            "worktree",
            "add",
            "--quiet",
            "-b",
            "topic",
            topic.to_str().expect("utf8"),
        ])
        .as_slice(),
        overlay.as_slice(),
        Some(&facts),
    );
    CEILING_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<Duration>>| slot.set(None));
    // Ended at its ceiling, the step cannot complete the add; waiting for the filter would have.
    assert_ne!(run, ForwardedRun::Ran(0), "a step ended at its ceiling cannot complete the add");
    // A step ended at its ceiling leaves the registration Git made, without a checkout.
    assert!(!topic.join("stored.txt").exists(), "the add was not completed");
    remove(root.as_path());
}

/// What:
///  The person's own `git read-tree --reset -u` is forwarded without a ceiling,
/// even though a ladder's `read-tree` has
///  one.
/// Why:
///   The ceiling is for the steps the wrapper starts itself,
/// and native Git gives the person's own command no bound:
/// a
///  conversion filter that takes minutes is the person's own work,
/// and ending their Git would leave a partial checkout and an
///  `index.lock` behind.
/// The ceiling for this thread is zero,
/// so a step that consulted it would end at once,
/// and the
///  person's command must still complete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shortenCeiling(0); const run = runForwardedGit({ args: ['-C', linked, 'read-tree', '--reset', '-u', 'HEAD'], facts }); expect(run).toEqual({ exitCode: 0 })
/// ```
#[test]
fn a_persons_own_read_tree_is_not_ended_at_the_ladder_ceiling() {
    use crate::ladder_step_ceilings::CEILING_FOR_THIS_THREAD;
    use std::time::Duration;
    let root: PathBuf = fixture("forward-person-read-tree");
    let (main, common) = source(root.as_path());
    std::fs::write(main.join("kept.txt"), b"kept\n").expect("a tracked file");
    git(main.as_path(), &["add", "--all"]);
    git(main.as_path(), &["commit", "--quiet", "--message=first"]);
    let made: PathBuf = root.join("linked");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "linked", made.to_str().expect("utf8")]);
    let linked: PathBuf = std::fs::canonicalize(made.as_path()).expect("canonical linked");
    let facts: CopyFacts = CopyFacts {
        identity: WorktreeIdentity::LinkedWorktree { common_dir: common.clone(), git_dir: common.join("worktrees/linked"), worktree_root: linked.clone() },
        creates: false,
        destination: None,
        add_plan: None,
        remove_plan: None,
    };
    CEILING_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<Duration>>| slot.set(Some(Duration::ZERO)));
    let run: ForwardedRun = run_forwarded_git(
        Path::new(REAL_GIT),
        arguments(&["-C", linked.to_str().expect("utf8"), "read-tree", "--reset", "-u", "HEAD"]).as_slice(),
        isolated_overlay().as_slice(),
        Some(&facts),
    );
    CEILING_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<Duration>>| slot.set(None));
    assert_eq!(run, ForwardedRun::Ran(0), "the person's own read-tree runs to its end");
    assert!(linked.join("kept.txt").is_file(), "its checkout wrote the file");
    remove(root.as_path());
}

/// What:
///  The step a stopped run names reaches a shell as the words it ran,
/// whatever the person's path holds.
/// Why:
///   The line says the step did not finish and names it so a person can run it again;
/// a path holding a
///  space,
/// a `$` or a backquote has to arrive as one word and with nothing expanded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(wordsOf(['-C', '/work/$bucket/a b', 'read-tree'])).toBe("-C '/work/$bucket/a b' read-tree")
/// ```
#[test]
fn a_stopped_step_is_named_as_the_shell_reads_it() {
    let words: Vec<OsString> = vec![OsString::from("-C"), OsString::from("/work/$bucket/a b"), OsString::from("read-tree")];
    let printed: String = words_of(words.as_slice());
    let read: std::process::Output = std::process::Command::new("bash").arg("-c").arg(format!("printf '%s|' {printed}")).output().expect("start bash");
    assert_eq!(String::from_utf8_lossy(&read.stdout), "-C|/work/$bucket/a b|read-tree|", "the shell reads the three words, unexpanded: {printed}");
}

/// What:
///  Nothing the leave-out set aside survives in the new worktree of a snapshot,
/// even when the target commit
///  stops ignoring the place the trash lived,
/// so another command's stage container cannot be staged from it.
/// Why:
///   The trash held the directories the leave-out moved out of the snapshot,
/// another command's stage container with its
///  credentials among them,
/// and it sat inside the new worktree.
/// A commit that no longer ignores that path makes it untracked
///  there,
/// so `git add --all` stages another command's state.
/// The source has uncommitted changes,
/// so the
///  dirty restore the review's sequence ran is part of this control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runForwardedGit(['-C', main, 'worktree', 'add', '-b', 'topic', topic, 'later'], facts);
/// expect(existsSync(join(topic, '.claude/cli-git-trash'))).toBe(false);
/// await git(topic, ['add', '--all']); expect(staged(topic).filter((p) => p.startsWith('.claude/'))).toEqual([]);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn btrfs_a_snapshot_leaves_nothing_left_out_in_the_new_worktree() {
    use crate::fs_btrfs_ioctl::subvolume_create_at;
    use crate::fs_btrfs_test_support::btrfs_fixture;
    use crate::worktree_copy_selection::STAGE_PREFIX;
    let Some(root) = btrfs_fixture("forward-trash-not-carried") else { return };
    let source_root: PathBuf = root.join("main");
    subvolume_create_at(source_root.as_path()).expect("the source is a subvolume root");
    git(source_root.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let ignore_with: &[u8] = b".claude/\nnode_modules/\n";
    let ignore_without: &[u8] = b"node_modules/\n";
    std::fs::write(source_root.join(".gitignore"), ignore_with).expect("the ignore rule that hides the trash");
    std::fs::write(source_root.join("tracked.txt"), b"one\n").expect("a tracked file");
    git(source_root.as_path(), &["add", "."]);
    git(source_root.as_path(), &["commit", "--quiet", "--message=one"]);
    // The target commit stops ignoring `.claude/`, which is the place the trash is made in.
    git(source_root.as_path(), &["checkout", "--quiet", "-b", "later"]);
    std::fs::write(source_root.join(".gitignore"), ignore_without).expect("the ignore rule removed");
    git(source_root.as_path(), &["add", ".gitignore"]);
    git(source_root.as_path(), &["commit", "--quiet", "--message=two"]);
    git(source_root.as_path(), &["checkout", "--quiet", "main"]);
    // A command that works at once needs the ignored state, and another command's stage is filling its own beside it.
    std::fs::create_dir_all(source_root.join("node_modules/pkg")).expect("ignored state");
    std::fs::write(source_root.join("node_modules/pkg/index.js"), b"module\n").expect("ignored state");
    let container: String = format!(".claude/worktrees/{STAGE_PREFIX}8f14e45fceea167a5a36dedd4bea2543");
    std::fs::create_dir_all(source_root.join(container.as_str()).join("payload")).expect("the other command's stage");
    std::fs::write(source_root.join(container.as_str()).join("payload/credentials"), b"another worktree's token\n").expect("its credentials");
    // An uncommitted change, so the dirty restore runs before the proof.
    std::fs::write(source_root.join("tracked.txt"), b"one, edited\n").expect("an edit");
    let main: PathBuf = std::fs::canonicalize(source_root.as_path()).expect("canonical main");
    let topic: PathBuf = root.join("topic");
    let facts: CopyFacts = CopyFacts {
        identity: main_identity(main.as_path(), main.join(".git").as_path()),
        creates: true,
        destination: Some(topic.clone()),
        add_plan: Some(crate::worktree_add_plan::AddPlan { destination: topic.clone(), insert_at: 4, quiet: true }),
        remove_plan: None,
    };
    let made: ForwardedRun = run_forwarded_git(
        Path::new(REAL_GIT),
        arguments(&["-C", main.to_str().expect("utf8"), "worktree", "add", "--quiet", "-b", "topic", topic.to_str().expect("utf8"), "later"]).as_slice(),
        isolated_overlay().as_slice(),
        Some(&facts),
    );
    assert_eq!(made, ForwardedRun::Ran(0), "the add ran");
    // The positive control on the rung: the record of the add's steps names the leave-out, which only a snapshot runs.
    let timing: String = std::fs::read_to_string(main.join(".git").join(crate::add_timing::ADD_TIMING_FILE)).expect("the add's timing record");
    assert!(timing.contains("\"prune\""), "the snapshot rung served this add, so a trash was made: {timing}");
    assert!(topic.join("node_modules/pkg/index.js").is_file(), "the ignored state a new worktree needs is carried");
    assert!(!topic.join(container.as_str()).exists(), "the other command's stage container is not in the new worktree");
    // The trash may stand in the new worktree until the commit service empties it: what must not happen is that the
    //  person's next `git add --all` stages any of it, which the repository's own exclude line is what prevents here.
    //  The target commit does not ignore `.claude/`, so anything still there would be staged without that line.
    assert!(std::fs::symlink_metadata(topic.join(crate::worktree_trash::TRASH_PARENT)).is_ok(), "the trash stands until the service empties it, so the exclusion below is what keeps it out of the index");
    git(topic.as_path(), &["add", "--all"]);
    let staged: String = String::from_utf8(git(topic.as_path(), &["diff", "--cached", "--name-only"]).stdout).expect("utf8");
    assert!(!staged.lines().any(|path: &str| { return path.starts_with(".claude/"); }), "nothing of another command's state is staged: {staged}");
    assert!(!topic.join("tracked.txt").is_file() || std::fs::read(topic.join("tracked.txt")).expect("tracked") != b"one, edited\n", "the person's uncommitted edit stays in the source");
    remove(root.as_path());
}
