//! What:
//!  Taking one side of a conflicted file during a real merge,
//! rebase (both backends),
//! cherry-pick and revert,
//! through the
//!  built executable:
//! the file is saved first and named,
//! a path that is not conflicted keeps the rejection,
//! a line that
//!  mixes both is rejected whole and names the clean paths,
//! and a copy that cannot be written rejects the command.
//! Why:
//!  The owner's decision of 2026-11-02 ("save,
//! then allow") lets `git checkout --theirs -- <path>` and its siblings
//!  through for conflicted paths only.
//! These controls make a real conflict with real Git,
//! edit the conflicted file by hand
//!  as a person resolving it would,
//! run the wrapper,
//! and compare the saved bytes with what the person had written.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await makeConflict('merge'); edit('a.rs'); const out = run(['checkout', '--theirs', '--', 'a.rs']);
//! expect(readFileSync(savedPath(out))).toEqual(handEdited);
//! ```

use super::support::{Fixture, Observed, fixture, git, remove, repository, run_direct, run_wrapped, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What a person had in the conflicted file when they ran the command:
/// not any side's content,
/// not valid UTF-8.
const HAND_EDITED: &[u8] = b"<<<<<<< ours\nmine \xff\n=======\ntheirs\n>>>>>>> topic\nand my own line\n";

/// Which operation a control stops in.
#[derive(Clone, Copy, Debug)]
enum Stop {
    /// `git merge topic`.
    Merge,
    /// `git rebase main` on `topic`,
    /// with the merge backend.
    RebaseMerge,
    /// `git rebase --apply main` on `topic`,
    /// with the apply backend.
    RebaseApply,
    /// `git cherry-pick topic`.
    CherryPick,
    /// `git revert` of the commit that changed `a.rs` on `main`,
    /// after `a.rs` changed again.
    Revert,
}

/// A repository stopped by `stop` with `a.rs` conflicted and `b.rs` and `c.rs` clean,
/// and `a.rs` hand-edited.
fn conflicted(fixture: &Fixture, stop: Stop) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    for (name, text) in [("a.rs", "base\n"), ("b.rs", "base\n"), ("c.rs", "keep\n")] {
        std::fs::write(repo.join(name), text).expect("file");
    }
    git(fixture, repo.as_path(), &["add", "--", "a.rs", "b.rs", "c.rs"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    std::fs::write(repo.join("a.rs"), "topic\n").expect("topic side");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=topic side"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("a.rs"), "main\n").expect("main side");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main side"]);
    let started: Observed = match stop {
        Stop::Merge => run_direct(fixture, repo.as_path(), &["merge", "--no-edit", "topic"]),
        Stop::CherryPick => run_direct(fixture, repo.as_path(), &["cherry-pick", "topic"]),
        Stop::RebaseMerge | Stop::RebaseApply => {
            git(fixture, repo.as_path(), &["switch", "--quiet", "topic"]);
            let arguments: Vec<&str> = if matches!(stop, Stop::RebaseApply) { vec!["rebase", "--apply", "main"] } else { vec!["rebase", "main"] };
            run_direct(fixture, repo.as_path(), arguments.as_slice())
        }
        Stop::Revert => {
            std::fs::write(repo.join("a.rs"), "later\n").expect("later change");
            git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=later change"]);
            run_direct(fixture, repo.as_path(), &["revert", "--no-edit", "HEAD~1"])
        }
    };
    assert_eq!(started.code, Some(1), "the operation stops with a conflict: {}{}", String::from_utf8_lossy(&started.stdout), stderr_of(&started));
    assert!(
        git(fixture, repo.as_path(), &["ls-files", "--unmerged"]).stdout.windows(4).any(|window| { return window == b"a.rs"; }),
        "a.rs is unmerged"
    );
    std::fs::write(repo.join("a.rs"), HAND_EDITED).expect("hand edit");
    return repo;
}

/// Whether the wrapper itself rejected the command.
fn rejected_by_policy(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// The places the notice names,
/// in order.
fn saved_places(stderr: &str) -> Vec<PathBuf> {
    let lines: Vec<&str> = stderr.lines().collect();
    return lines
        .iter()
        .enumerate()
        .filter(|(_, line)| { return line.starts_with("monochromatic-git-policy-cli: saved your copy of "); })
        .filter_map(|(at, _)| { return lines.get(at + 1).map(|place| { return PathBuf::from(place.trim()); }); })
        .collect();
}

/// The directory of the copies below the common Git directory of `repo`.
fn saved_root(repo: &Path) -> PathBuf {
    return repo.join(".git/monochromatic-git-policy-cli/discarded");
}

/// Every audit line of the repository.
fn audit_lines(repo: &Path) -> Vec<String> {
    return std::fs::read_to_string(repo.join(".git/cli-git-audit.jsonl")).unwrap_or_default().lines().map(String::from).collect();
}

/// Every control for one stop:
/// each allowed form saves the hand-edited file first and names the place;
/// the audit record names it.
///  Each form gets a conflict of its own,
/// because a form that names a revision resolves the path in the index,
/// which ends
///  its being conflicted.
fn allowed_forms_save_the_hand_edited_file(label: &str, stop: Stop, forms: &[&[&str]]) {
    for (round, arguments) in forms.iter().enumerate() {
        let fixture: Fixture = fixture(format!("{label}-{round}").as_str());
        let repo: PathBuf = conflicted(&fixture, stop);
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments);
        let said: String = stderr_of(&observed);
        assert!(!rejected_by_policy(&observed), "{stop:?} {arguments:?}: {said}");
        let places: Vec<PathBuf> = saved_places(said.as_str());
        assert_eq!(places.len(), 1, "{stop:?} {arguments:?}: one copy is named: {said}");
        assert!(places[0].starts_with(saved_root(repo.as_path())), "{stop:?} {arguments:?}: the copy is beside the audit log: {places:?}");
        assert!(places[0].ends_with("a.rs"), "{places:?}");
        assert_eq!(std::fs::read(&places[0]).expect("the copy exists"), HAND_EDITED, "{stop:?} {arguments:?}: the original bytes");
        assert!(said.contains("a.rs was conflicted"), "{said}");
        let audit: Vec<String> = audit_lines(repo.as_path());
        let saved: Vec<&String> = audit.iter().filter(|line| { return line.contains("\"outcome\":\"saved-conflict\""); }).collect();
        assert_eq!(saved.len(), 1, "{audit:?}");
        assert!(saved[0].contains("\"copies\":[\"a.rs => "), "{}", saved[0]);
        remove(&fixture);
    }
}

/// A real merge conflict:
/// every form that takes a side saves first.
#[test]
fn a_merge_conflict_side_is_taken_after_the_file_is_saved() {
    allowed_forms_save_the_hand_edited_file(
        "side-merge",
        Stop::Merge,
        &[
            &["checkout", "--theirs", "--", "a.rs"],
            &["checkout", "--ours", "--", "a.rs"],
            &["checkout", "topic", "--", "a.rs"],
            &["checkout", "--merge", "--", "a.rs"],
            &["restore", "--theirs", "--", "a.rs"],
            &["restore", "--ours", "--", "a.rs"],
            &["restore", "--source=topic", "--", "a.rs"],
        ],
    );
}

/// A real rebase conflict,
/// merge backend.
#[test]
fn a_rebase_conflict_side_is_taken_after_the_file_is_saved() {
    allowed_forms_save_the_hand_edited_file(
        "side-rebase-merge",
        Stop::RebaseMerge,
        &[&["checkout", "--theirs", "--", "a.rs"], &["checkout", "--ours", "--", "a.rs"], &["restore", "--theirs", "--", "a.rs"]],
    );
}

/// A real rebase conflict,
/// apply backend.
#[test]
fn a_rebase_conflict_of_the_apply_backend_side_is_taken_after_the_file_is_saved() {
    allowed_forms_save_the_hand_edited_file(
        "side-rebase-apply",
        Stop::RebaseApply,
        &[&["checkout", "--theirs", "--", "a.rs"], &["restore", "--ours", "--", "a.rs"]],
    );
}

/// A real cherry-pick conflict.
#[test]
fn a_cherry_pick_conflict_side_is_taken_after_the_file_is_saved() {
    allowed_forms_save_the_hand_edited_file(
        "side-cherry-pick",
        Stop::CherryPick,
        &[&["checkout", "--theirs", "--", "a.rs"], &["checkout", "topic", "--", "a.rs"], &["restore", "--theirs", "--", "a.rs"]],
    );
}

/// A real revert conflict.
#[test]
fn a_revert_conflict_side_is_taken_after_the_file_is_saved() {
    allowed_forms_save_the_hand_edited_file("side-revert", Stop::Revert, &[&["checkout", "--theirs", "--", "a.rs"], &["restore", "--ours", "--", "a.rs"]]);
}

/// The side really is taken:
/// the file holds their content after the command,
/// and only after the copy was made.
#[test]
fn taking_their_side_replaces_the_file_once_it_is_saved() {
    let fixture: Fixture = fixture("side-replaces");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), b"topic\n");
    let place: PathBuf = saved_places(stderr_of(&observed).as_str()).remove(0);
    assert_eq!(std::fs::read(place).expect("the copy"), HAND_EDITED);
    remove(&fixture);
}

/// A path that is not conflicted keeps the rejection,
/// a line that mixes both is rejected whole naming the clean path,
/// and
///  nothing is copied or changed.
#[test]
fn clean_paths_keep_the_rejection_and_a_mixed_line_is_rejected_whole() {
    for stop in [Stop::Merge, Stop::RebaseMerge, Stop::CherryPick] {
        let fixture: Fixture = fixture(format!("side-clean-{stop:?}").to_lowercase().as_str());
        let repo: PathBuf = conflicted(&fixture, stop);
        for arguments in [
            vec!["checkout", "--theirs", "--", "c.rs"],
            vec!["checkout", "topic", "--", "b.rs"],
            vec!["restore", "--theirs", "--", "c.rs"],
            vec!["checkout", "--theirs", "--", "."],
            vec!["checkout", "--theirs", "--", "*.rs"],
        ] {
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
            assert!(rejected_by_policy(&observed), "{stop:?} {arguments:?}: {}", stderr_of(&observed));
        }
        for arguments in [vec!["checkout", "--theirs", "--", "a.rs", "c.rs"], vec!["restore", "--ours", "--", "c.rs", "a.rs"]] {
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{stop:?} {arguments:?}: {said}");
            assert!(said.contains("`c.rs`") && said.contains("not conflicted files"), "{stop:?} {arguments:?} names the clean path: {said}");
            assert!(!said.contains("`a.rs`"), "the conflicted path is not named as clean: {said}");
        }
        assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "the hand edit is untouched");
        assert!(!saved_root(repo.as_path()).exists(), "no copy was made for a rejected line");
        remove(&fixture);
    }
}

/// The forms the decision does not cover keep their rejection even for a conflicted path.
#[test]
fn forms_outside_the_decision_keep_the_rejection() {
    let fixture: Fixture = fixture("side-uncovered");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    for arguments in [
        vec!["checkout", "-f", "--", "a.rs"],
        vec!["checkout", "--theirs", "a.rs"],
        vec!["checkout", "--patch", "--", "a.rs"],
        vec!["restore", "--staged", "--theirs", "--", "a.rs"],
        vec!["restore", "--theirs", "a.rs"],
        vec!["checkout", "--theirs", "--", "sub/"],
        vec!["reset", "--hard"],
        vec!["merge", "--abort"],
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert!(rejected_by_policy(&observed), "{arguments:?}: {}", stderr_of(&observed));
    }
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED);
    assert!(!saved_root(repo.as_path()).exists());
    remove(&fixture);
}

/// With no operation in progress a conflict-shaped line is an ordinary discarding form and is rejected,
/// even for a path that
///  still has unmerged entries (what a conflicting `git stash pop` leaves).
#[test]
fn without_an_operation_in_progress_the_rejection_stays() {
    let fixture: Fixture = fixture("side-no-operation");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    std::fs::remove_file(repo.join(".git/MERGE_HEAD")).expect("end the merge's state, leaving the unmerged entries");
    for arguments in [vec!["checkout", "--theirs", "--", "a.rs"], vec!["restore", "--source=topic", "--", "a.rs"]] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert!(rejected_by_policy(&observed), "{arguments:?}: {}", stderr_of(&observed));
    }
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED);
    assert!(!saved_root(repo.as_path()).exists());
    remove(&fixture);
}

/// A copy that cannot be written rejects the command,
/// says why,
/// changes nothing,
/// and leaves the file as the person had it.
#[test]
fn a_copy_that_cannot_be_written_rejects_the_command() {
    let fixture: Fixture = fixture("side-copy-fails");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    // `monochromatic-git-policy-cli` is a file, so no directory for the copies can be made below it.
    std::fs::write(repo.join(".git/monochromatic-git-policy-cli"), b"in the way").expect("blocker");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert!(rejected_by_policy(&observed), "{said}");
    assert!(said.contains("your copy of it could not be saved first"), "{said}");
    assert!(said.contains("discarded"), "the place is named: {said}");
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "the file was not replaced");
    assert!(audit_lines(repo.as_path()).iter().all(|line| { return !line.contains("saved-conflict"); }));
    // Once the place is free the same command goes through.
    std::fs::remove_file(repo.join(".git/monochromatic-git-policy-cli")).expect("remove blocker");
    let again: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    assert_eq!(again.code, Some(0), "{}", stderr_of(&again));
    remove(&fixture);
}

/// A write that fails part way (a file-size limit stands in for a full disk:
/// the write returns `EFBIG` as a full disk
///  returns `ENOSPC`) rejects the command,
/// leaves no cut-short copy,
/// and leaves the file as the person had it.
#[test]
fn a_copy_cut_short_by_a_failing_write_rejects_the_command_and_leaves_no_partial_copy() {
    use std::os::unix::process::CommandExt;
    let fixture: Fixture = fixture("side-write-fails");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    let mut command = super::support::wrapped(&fixture);
    command.current_dir(&repo).args(["checkout", "--theirs", "--", "a.rs"]);
    // SAFETY: between fork and exec only `signal` and `setrlimit`, both async-signal-safe, run.
    unsafe {
        command.pre_exec(|| {
            libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
            let cap: libc::rlimit = libc::rlimit { rlim_cur: 16, rlim_max: 16 };
            if libc::setrlimit(libc::RLIMIT_FSIZE, &raw const cap) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    let observed: Observed = super::support::observe(&mut command, b"");
    let said: String = stderr_of(&observed);
    assert!(rejected_by_policy(&observed), "{said}");
    assert!(said.contains("your copy of it could not be saved first"), "{said}");
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "the file was not replaced");
    let leftovers: Vec<PathBuf> = walk(saved_root(repo.as_path()).as_path()).into_iter().filter(|path| { return path.is_file(); }).collect();
    assert!(leftovers.is_empty(), "no partial copy stays: {leftovers:?}");
    remove(&fixture);
}

/// Every path below `directory`,
/// directories included;
/// none when it does not exist.
fn walk(directory: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        found.push(entry.path());
        if entry.path().is_dir() {
            found.extend(walk(entry.path().as_path()));
        }
    }
    return found;
}

/// A conflicted path deleted in the worktree has nothing to copy:
/// the notice says so and the command is allowed.
#[test]
fn a_conflicted_path_deleted_in_the_worktree_is_allowed_with_no_copy() {
    let fixture: Fixture = fixture("side-deleted");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    std::fs::remove_file(repo.join("a.rs")).expect("delete the conflicted file");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(said.contains("a.rs was conflicted and is deleted in your worktree, so there is no copy to save"), "{said}");
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs is back"), b"topic\n");
    assert!(!saved_root(repo.as_path()).exists(), "nothing was copied");
    remove(&fixture);
}

/// The same holds in a linked worktree,
/// where the copy goes to the common Git directory beside the shared audit log.
#[test]
fn a_linked_worktree_saves_beside_the_common_audit_log() {
    let fixture: Fixture = fixture("side-linked");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.rs"), "base\n").expect("a.rs");
    git(&fixture, repo.as_path(), &["add", "--", "a.rs"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["branch", "topic"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join("a.rs"), "side\n").expect("side edit");
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "topic"]);
    std::fs::write(repo.join("a.rs"), "topic\n").expect("topic edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=topic"]);
    let merged: Observed = run_direct(&fixture, linked.as_path(), &["merge", "--no-edit", "topic"]);
    assert_eq!(merged.code, Some(1), "{}", stderr_of(&merged));
    std::fs::write(linked.join("a.rs"), HAND_EDITED).expect("hand edit");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let place: PathBuf = saved_places(said.as_str()).remove(0);
    assert!(place.starts_with(std::fs::canonicalize(repo.join(".git")).expect("common directory").join("monochromatic-git-policy-cli/discarded")), "{place:?}");
    assert_eq!(std::fs::read(place).expect("copy"), HAND_EDITED);
    remove(&fixture);
}

/// The directory name the wrapper gives a save made `age_millis` before now.
fn name_before_now(age_millis: i64) -> String {
    let now: i64 = i64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("a clock").as_millis()).expect("fits");
    return git_policy_cli::iso_time::format_iso_milliseconds(now - age_millis).chars().take(19).map(|each| { return if each == ':' { '-' } else { each }; }).collect();
}

/// Plants a save of `bytes` bytes made `age_millis` ago below the folder of copies,
/// and gives its directory.
fn plant_save(repo: &Path, age_millis: i64, bytes: usize) -> PathBuf {
    let directory: PathBuf = saved_root(repo).join(name_before_now(age_millis));
    std::fs::create_dir_all(&directory).expect("planted save");
    std::fs::write(directory.join("planted.txt"), vec![b'p'; bytes]).expect("planted copy");
    return directory;
}

/// A day in milliseconds.
const A_DAY: i64 = 24 * 60 * 60 * 1000;

/// Saving a copy applies the limits:
/// a save past thirty days goes with an audit line,
/// a younger one and the new copy stay.
#[test]
fn a_new_save_removes_the_saves_past_the_age_limit_and_audits_them() {
    let fixture: Fixture = fixture("side-prune-age");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    let old: PathBuf = plant_save(repo.as_path(), 45 * A_DAY, 100);
    let recent: PathBuf = plant_save(repo.as_path(), 5 * A_DAY, 100);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(!old.exists(), "the save past thirty days is removed");
    assert!(recent.exists(), "a save inside the limit stays");
    let place: PathBuf = saved_places(said.as_str()).remove(0);
    assert_eq!(std::fs::read(place).expect("the new copy"), HAND_EDITED);
    let pruned: Vec<String> = audit_lines(repo.as_path()).into_iter().filter(|line| { return line.contains("\"outcome\":\"pruned\""); }).collect();
    assert_eq!(pruned.len(), 1, "{pruned:?}");
    assert!(pruned[0].contains("\"reason\":\"age\"") && pruned[0].contains("planted.txt") && pruned[0].contains(old.to_string_lossy().as_ref()), "{}", pruned[0]);
    remove(&fixture);
}

/// The `discardedCopies` settings of `monochromatic-git-policy-cli.config.jsonc` change the limits:
/// a longer age keeps the old save,
/// and a small size
/// removes an older save but never one younger than a day.
#[test]
fn the_configuration_file_changes_the_limits_and_a_young_save_stays() {
    let fixture: Fixture = fixture("side-prune-config");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    std::fs::write(repo.join("monochromatic-git-policy-cli.config.jsonc"), r#"{ "discardedCopies": { "maxAgeDays": 100, "maxSize": "1k" } }"#).expect("configuration");
    let old: PathBuf = plant_save(repo.as_path(), 45 * A_DAY, 4096);
    let older_than_a_day: PathBuf = plant_save(repo.as_path(), 3 * A_DAY, 4096);
    let young: PathBuf = plant_save(repo.as_path(), A_DAY / 4, 4096);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert!(!old.exists() && !older_than_a_day.exists(), "the size limit of one kibibyte takes the older saves, not the age limit of a hundred days");
    assert!(young.exists(), "a save a quarter of a day old is out of reach of the size limit");
    let reasons: Vec<String> = audit_lines(repo.as_path()).into_iter().filter(|line| { return line.contains("\"outcome\":\"pruned\""); }).collect();
    assert_eq!(reasons.len(), 2, "{reasons:?}");
    assert!(reasons.iter().all(|line| { return line.contains("\"reason\":\"size\"") && line.contains("\"limit\":1024"); }), "{reasons:?}");
    remove(&fixture);
}

/// A longer age limit keeps a save the default would have removed.
#[test]
fn a_longer_age_setting_keeps_the_old_save() {
    let fixture: Fixture = fixture("side-prune-keeps");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    std::fs::write(repo.join("monochromatic-git-policy-cli.config.jsonc"), r#"{ "discardedCopies": { "maxAgeDays": 400 } }"#).expect("configuration");
    let old: PathBuf = plant_save(repo.as_path(), 45 * A_DAY, 100);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert!(old.exists());
    assert!(audit_lines(repo.as_path()).iter().all(|line| { return !line.contains("\"outcome\":\"pruned\""); }));
    remove(&fixture);
}

/// `git monochromatic-git-policy-cli doctor` states the folder's size,
/// copy count and oldest save.
#[test]
fn the_doctor_reports_the_folder_of_saved_copies() {
    let fixture: Fixture = fixture("side-prune-doctor");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    plant_save(repo.as_path(), 9 * A_DAY, 300);
    let oldest: String = name_before_now(12 * A_DAY);
    let directory: PathBuf = saved_root(repo.as_path()).join(oldest.as_str());
    std::fs::create_dir_all(&directory).expect("older save");
    std::fs::write(directory.join("older.txt"), vec![b'o'; 200]).expect("older copy");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "doctor"]);
    let said: String = stderr_of(&observed);
    assert!(said.contains("git monochromatic-git-policy-cli doctor: discarded copies:") && said.contains("500 bytes in 2 copies from 2 saves"), "{said}");
    assert!(said.contains(format!("the oldest save is {oldest}").as_str()), "{said}");
    remove(&fixture);
}

/// A folder of copies that is a link to a directory elsewhere is never followed by the pass:
/// the command still runs and says that
/// the limits did not apply,
/// and an old save the link reaches stays.
#[test]
fn a_linked_folder_of_copies_is_not_pruned_and_the_notice_says_so() {
    let fixture: Fixture = fixture("side-prune-link");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    let elsewhere: PathBuf = fixture.root.join("elsewhere");
    let old: PathBuf = elsewhere.join(name_before_now(45 * A_DAY));
    std::fs::create_dir_all(&old).expect("an old save elsewhere");
    std::fs::write(old.join("planted.txt"), "not the wrapper's to remove").expect("planted copy");
    let folder: PathBuf = saved_root(repo.as_path());
    std::fs::create_dir_all(folder.parent().expect("the wrapper's directory")).expect("directory");
    std::os::unix::fs::symlink(&elsewhere, &folder).expect("the link");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert!(old.join("planted.txt").exists(), "a save the link reaches was removed");
    assert!(said.contains("were not pruned") && said.contains("symbolic link"), "{said}");
    assert!(audit_lines(repo.as_path()).iter().all(|line| { return !line.contains("\"outcome\":\"pruned\""); }));
    remove(&fixture);
}

/// A limit that is no limit stops the command with the configuration failure that names the key,
/// and removes nothing.
#[test]
fn a_zero_limit_is_a_configuration_failure_and_removes_nothing() {
    let fixture: Fixture = fixture("side-prune-zero");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    std::fs::write(repo.join("monochromatic-git-policy-cli.config.jsonc"), r#"{ "discardedCopies": { "maxAgeDays": 0 } }"#).expect("configuration");
    let old: PathBuf = plant_save(repo.as_path(), 45 * A_DAY, 100);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(2), "{said}");
    assert!(said.contains("discardedCopies.maxAgeDays"), "{said}");
    assert!(old.exists(), "nothing was removed");
    remove(&fixture);
}

/// The doctor states the limits that the configuration file sets.
#[test]
fn the_doctor_names_the_limits_the_configuration_sets() {
    let fixture: Fixture = fixture("side-prune-doctor-limits");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("monochromatic-git-policy-cli.config.jsonc"), r#"{ "discardedCopies": { "maxAgeDays": 90, "maxSize": "2k" } }"#).expect("configuration");
    plant_save(repo.as_path(), 9 * A_DAY, 300);
    let said: String = stderr_of(&run_wrapped(&fixture, repo.as_path(), &["monochromatic-git-policy-cli", "doctor"]));
    assert!(said.contains("older than 90 days") && said.contains("past 2048 bytes"), "{said}");
    remove(&fixture);
}

/// Command lines that look like one conflicted file to the guard and are more to Git,
/// found by a cross-vendor review.
#[path = "binary_conflict_side_hardening_tests.rs"]
mod hardening;
