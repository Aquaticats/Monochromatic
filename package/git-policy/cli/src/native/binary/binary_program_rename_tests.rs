//! What:
//!  The built program under its new name and its earlier names,
//! in real repositories.
//! Why:
//!  The owner renamed the program to `monochromatic-git-policy-cli`,
//! its command word,
//! its settings file and the folders
//!  and files it keeps in the Git directory,
//! and decided that the earlier names keep working:
//! the earlier command word with
//!  one notice line,
//! an earlier settings file read with one notice,
//! and a kept folder or file moved in place the first time
//!  it is needed,
//! with nothing lost when processes race to move it (`doc/planning/git-policy-cli-program-rename.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const earlier = await run(repo, ['cli-git', 'doctor']);
//! expect(earlier.stderr).toBe(NOTICE + (await run(repo, ['monochromatic-git-policy-cli', 'doctor'])).stderr);
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, ServiceLaunch, fixture, git, observe, remove, repository, start_fixture_service, wrapped};
use git_policy_cli::entry::EARLIER_COMMAND_WORD_NOTICE;
use git_policy_cli::wrapper_names_old_and_new::{
    COMMAND_WORD, COMMIT_TIMING_LOG, DATA_FOLDER, DISCARD_AUDIT_LOG, MERGE_INTENT_FOLDER, PUSH_TRIGGER_LOG, RenamedName, SETTINGS_FILE,
    WORKTREE_ADD_TIMING_LOG, WORKTREE_COPY_RECORD_FOLDER,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Run the wrapped `git` with `arguments` in `directory`.
fn run(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Observed {
    return observe(wrapped(fixture).current_dir(directory).args(arguments), b"");
}

/// The standard error of a run,
///  as text.
fn stderr_text(observed: &Observed) -> String {
    return String::from_utf8_lossy(&observed.stderr).into_owned();
}

/// Every management command that is safe in a plain repository,
///  with its options.
const COMMANDS: [&[&str]; 6] = [
    &["--help"],
    &["doctor"],
    &["check", "--all"],
    &["fix", "--all"],
    &["trust"],
    &["migrate-to-subvolume", "--help"],
];

/// The new word answers every management command without a notice,
///  and the earlier word answers it the same with exactly one line more on standard error.
#[test]
fn the_earlier_word_does_the_same_with_one_notice_for_every_command() {
    let fixture: Fixture = fixture("rename-word");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    for command in COMMANDS {
        let mut current: Vec<&str> = vec![COMMAND_WORD.current];
        current.extend(command);
        let mut earlier: Vec<&str> = vec![COMMAND_WORD.earlier];
        earlier.extend(command);
        let new_run: Observed = run(&fixture, repo.as_path(), current.as_slice());
        let old_run: Observed = run(&fixture, repo.as_path(), earlier.as_slice());
        assert_eq!(old_run.code, new_run.code, "{command:?}");
        assert_eq!(old_run.stdout, new_run.stdout, "{command:?}");
        assert_eq!(stderr_text(&old_run), format!("{EARLIER_COMMAND_WORD_NOTICE}{}", stderr_text(&new_run)), "{command:?}");
        assert!(!stderr_text(&new_run).contains(EARLIER_COMMAND_WORD_NOTICE), "{command:?}");
    }
    remove(&fixture);
}

/// How many lines of a run's standard error say that the settings file is read under its earlier name.
fn earlier_name_lines(observed: &Observed) -> usize {
    return stderr_text(observed).lines().filter(|line| { return line.contains("read under its earlier name"); }).count();
}

/// A repository whose only settings file has the earlier name:
///  the settings are read (a policy it sets to an error finds a violation),
///  and the one notice names the new file.
#[test]
fn an_earlier_settings_file_alone_is_read_with_one_notice() {
    let fixture: Fixture = fixture("rename-settings-earlier");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(SETTINGS_FILE.earlier), "{ \"policies\": { \"final-newline\": \"error\" } }").expect("an earlier file");
    std::fs::write(repo.join("a.txt"), "no newline at the end").expect("a file");
    git(&fixture, repo.as_path(), &["add", "a.txt"]);
    let checked: Observed = run(&fixture, repo.as_path(), &[COMMAND_WORD.current, "check", "--all"]);
    assert_eq!(checked.code, Some(1), "the earlier file was read, so its error policy found the file: {}", String::from_utf8_lossy(&checked.stdout));
    assert_eq!(earlier_name_lines(&checked), 1, "one notice: {}", stderr_text(&checked));
    assert!(stderr_text(&checked).contains(SETTINGS_FILE.current), "{}", stderr_text(&checked));
    assert!(repo.join(SETTINGS_FILE.earlier).is_file() && !repo.join(SETTINGS_FILE.current).exists(), "no file was renamed or made");
    remove(&fixture);
}

/// A guarded Git command reads the settings too,
///  and says the same one line once however many times the settings are read.
#[test]
fn a_guarded_git_command_says_the_settings_notice_once() {
    let fixture: Fixture = fixture("rename-settings-guarded");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(SETTINGS_FILE.earlier), "{}").expect("an earlier file");
    std::fs::write(repo.join("a.txt"), "a\n").expect("a file");
    let added: Observed = run(&fixture, repo.as_path(), &["add", "a.txt"]);
    assert_eq!(added.code, Some(0), "{}", stderr_text(&added));
    assert_eq!(earlier_name_lines(&added), 1, "{}", stderr_text(&added));
    remove(&fixture);
}

/// Only the new file:
///  read,
///  nothing said.
#[test]
fn the_new_settings_file_alone_is_read_without_a_notice() {
    let fixture: Fixture = fixture("rename-settings-new");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(SETTINGS_FILE.current), "{ \"unknown\": 1 }").expect("an invalid new file");
    let refused: Observed = run(&fixture, repo.as_path(), &[COMMAND_WORD.current, "check", "--all"]);
    assert!(String::from_utf8_lossy(&refused.stdout).contains("Unknown configuration key: unknown."));
    assert_eq!(stderr_text(&refused), "");
    remove(&fixture);
}

/// Both files:
///  the new one is read,
///  and the earlier one is reported as ignored.
#[test]
fn both_settings_files_read_the_new_one_and_report_the_earlier() {
    let fixture: Fixture = fixture("rename-settings-both");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join(SETTINGS_FILE.current), "{}").expect("a valid new file");
    std::fs::write(repo.join(SETTINGS_FILE.earlier), "{ \"unknown\": 1 }").expect("an invalid earlier file");
    let checked: Observed = run(&fixture, repo.as_path(), &[COMMAND_WORD.current, "check", "--all"]);
    assert_eq!(checked.code, Some(0), "the earlier file's error is not read: {}", String::from_utf8_lossy(&checked.stdout));
    assert!(stderr_text(&checked).contains(SETTINGS_FILE.earlier) && stderr_text(&checked).contains("ignored"), "{}", stderr_text(&checked));
    remove(&fixture);
}

/// Neither file:
///  nothing is said and the command behaves as before.
#[test]
fn no_settings_file_says_nothing_about_names() {
    let fixture: Fixture = fixture("rename-settings-neither");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let checked: Observed = run(&fixture, repo.as_path(), &[COMMAND_WORD.current, "check", "--all"]);
    assert_eq!(checked.code, Some(0));
    assert_eq!(stderr_text(&checked), "");
    remove(&fixture);
}

/// Plant a kind under its earlier name in `.git`,
///  with content that names the kind.
fn plant_earlier(git_dir: &Path, kind: RenamedName, is_folder: bool) {
    let path: PathBuf = git_dir.join(kind.earlier);
    if is_folder {
        std::fs::create_dir_all(path.join("inner")).expect("folder");
        std::fs::write(path.join("inner").join("record"), kind.earlier).expect("record");
    } else {
        std::fs::write(path, kind.earlier).expect("file");
    }
}

/// Read what `plant_earlier` wrote under the current name.
fn read_current(git_dir: &Path, kind: RenamedName, is_folder: bool) -> String {
    let path: PathBuf = git_dir.join(kind.current);
    if is_folder {
        return std::fs::read_to_string(path.join("inner").join("record")).unwrap_or_else(|error| { return format!("missing: {error}"); });
    }
    return std::fs::read_to_string(path).unwrap_or_else(|error| { return format!("missing: {error}"); });
}

/// The folders and files the wrapper keeps in the Git directory of a plain repository.
const KEPT: [(RenamedName, bool); 7] = [
    (DATA_FOLDER, true),
    (MERGE_INTENT_FOLDER, true),
    (WORKTREE_COPY_RECORD_FOLDER, true),
    (DISCARD_AUDIT_LOG, false),
    (COMMIT_TIMING_LOG, false),
    (WORKTREE_ADD_TIMING_LOG, false),
    (PUSH_TRIGGER_LOG, false),
];

/// Every kind planted under its earlier name is renamed in place by the first Git command that needs it,
///  with its content intact,
///  and the command succeeds without a word about it.
#[test]
fn every_kept_kind_is_renamed_in_place_by_an_ordinary_command() {
    let fixture: Fixture = fixture("rename-kept");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let git_dir: PathBuf = repo.join(".git");
    for (kind, folder) in KEPT {
        plant_earlier(git_dir.as_path(), kind, folder);
    }
    let status: Observed = run(&fixture, repo.as_path(), &["status"]);
    assert_eq!(status.code, Some(0), "{}", stderr_text(&status));
    assert_eq!(stderr_text(&status), "", "a move in place is silent");
    for (kind, folder) in KEPT {
        assert_eq!(read_current(git_dir.as_path(), kind, folder), kind.earlier, "{kind:?}");
        assert!(!git_dir.join(kind.earlier).exists(), "{kind:?}: no second name");
    }
    remove(&fixture);
}

/// Both names of a kind:
/// an ordinary command folds the earlier one into the new one without a word,
/// and every entry survives.
///  The fold looks at this user's processes,
/// so a process of another test that has the earlier service's journal option with a
///  folder that cannot be resolved makes the look say "may be running" (the conservative answer the fold is built on):
/// the
///  command then prints exactly one line that says it will fold later,
/// and nothing has moved.
/// Both ends are checked,
/// and the
///  fold itself with a process table of its own is checked by `earlier_names_folded_into_current_tests.rs`.
#[test]
fn both_names_of_a_kept_kind_are_folded_into_the_new_one_without_a_word_or_wait_with_one_line() {
    let fixture: Fixture = fixture("rename-kept-both");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let git_dir: PathBuf = repo.join(".git");
    plant_earlier(git_dir.as_path(), DATA_FOLDER, true);
    std::fs::create_dir_all(git_dir.join(DATA_FOLDER.current)).expect("the new folder");
    let status: Observed = run(&fixture, repo.as_path(), &["status"]);
    assert_eq!(status.code, Some(0), "{}", stderr_text(&status));
    let said: String = stderr_text(&status);
    if said.is_empty() {
        assert_eq!(read_current(git_dir.as_path(), DATA_FOLDER, true), DATA_FOLDER.earlier);
        assert!(!git_dir.join(DATA_FOLDER.earlier).exists(), "no second name");
    } else {
        assert_eq!(said.lines().count(), 1, "one line: {said}");
        assert!(said.contains("a later command folds them into the new names"), "{said}");
        assert_eq!(std::fs::read_to_string(git_dir.join(DATA_FOLDER.earlier).join("inner").join("record")).expect("untouched"), DATA_FOLDER.earlier);
    }
    remove(&fixture);
}

/// How many wrapper processes start together in one round of the race,
///  and how many rounds run:
///  enough that two of them reach the rename in the same instant on a loaded host.
const RACING_PROCESSES: usize = 8;

/// The rounds of the race.
const RACING_ROUNDS: usize = 4;

/// Several wrapper processes start at once on a repository that holds only the earlier folder:
///  one moves it,
///  the rest find the result,
///  nothing is lost and no process fails or complains.
#[test]
fn wrapper_processes_racing_to_move_a_kept_folder_lose_nothing() {
    let fixture: Fixture = fixture("rename-kept-race");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let git_dir: PathBuf = repo.join(".git");
    for round in 0..RACING_ROUNDS {
        plant_earlier(git_dir.as_path(), DATA_FOLDER, true);
        let observed: Vec<Observed> = std::thread::scope(|scope| {
            let mut workers = Vec::new();
            for _ in 0..RACING_PROCESSES {
                workers.push(scope.spawn(|| { return run(&fixture, repo.as_path(), &["status"]); }));
            }
            return workers.into_iter().map(|worker| { return worker.join().expect("a worker"); }).collect();
        });
        for run_result in &observed {
            assert_eq!(run_result.code, Some(0), "round {round}: {}", stderr_text(run_result));
            assert_eq!(stderr_text(run_result), "", "round {round}: no process complains");
        }
        assert_eq!(read_current(git_dir.as_path(), DATA_FOLDER, true), DATA_FOLDER.earlier, "round {round}");
        assert!(!git_dir.join(DATA_FOLDER.earlier).exists(), "round {round}");
        std::fs::remove_dir_all(git_dir.join(DATA_FOLDER.current)).expect("reset for the next round");
    }
    remove(&fixture);
}

/// A repository path with a space,
/// quotes and a newline in it:
///  the earlier command word,
/// the earlier settings file and the earlier folder all work as in any other path.
#[test]
fn an_awkward_repository_path_changes_nothing() {
    let fixture: Fixture = fixture("rename-awkward");
    let repo: PathBuf = repository(&fixture, OsStr::new("a repo's \"top\"\nlevel"));
    let git_dir: PathBuf = repo.join(".git");
    std::fs::write(repo.join(SETTINGS_FILE.earlier), "{}").expect("an earlier settings file");
    plant_earlier(git_dir.as_path(), DATA_FOLDER, true);
    let checked: Observed = run(&fixture, repo.as_path(), &[COMMAND_WORD.earlier, "check", "--all"]);
    assert_eq!(checked.code, Some(0), "{}", stderr_text(&checked));
    let lines: Vec<String> = stderr_text(&checked).lines().map(str::to_owned).collect();
    assert_eq!(lines.len(), 2, "the word notice and the settings notice, one line each: {lines:?}");
    assert_eq!(read_current(git_dir.as_path(), DATA_FOLDER, true), DATA_FOLDER.earlier);
    remove(&fixture);
}

/// Write one commit's worth of change to `a.txt` and commit it through the service of the fixture,
///  then count the batches the service finished.
fn commit_through_the_fixture_service(fixture: &Fixture, repo: &Path, message: &str) -> usize {
    std::fs::write(repo.join("a.txt"), format!("base\n{message}\n")).expect("edit");
    let service = start_fixture_service(fixture);
    service.serve(fixture, repo.join(".git").as_path());
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "-m", message, "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &service.runtime);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let finished: usize = std::fs::read_dir(fixture.root.join("journal"))
        .expect("journal")
        .flatten()
        .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == git_policy_cli::service_journal::DONE_EXTENSION; }); })
        .count();
    drop(service);
    return finished;
}

/// A repository with `a.txt` committed and Git's maintenance kept in the foreground.
fn repository_with_a_committed_file(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(fixture, &repo, &["add", "a.txt"]);
    git(fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    return repo;
}

/// The service this release starts,
/// under the new command word,
/// lands a commit.
#[test]
fn a_service_started_with_the_new_command_word_lands_a_commit() {
    let fixture: Fixture = fixture("rename-service-new");
    let repo: PathBuf = repository_with_a_committed_file(&fixture);
    assert_eq!(commit_through_the_fixture_service(&fixture, repo.as_path(), "through the new word"), 1);
    remove_settled(&fixture);
}

/// A helper that an older program started has the earlier command word in its unit file;
///  the executable answers that word as the service,
/// and the helper lands the commits it is sent.
#[test]
fn a_helper_started_with_the_earlier_command_word_keeps_landing_commits() {
    let fixture: Fixture = fixture("rename-service-earlier");
    *fixture.service_launch.lock().expect("the service launch") = ServiceLaunch { earlier_command_word: true, ..ServiceLaunch::default() };
    let repo: PathBuf = repository_with_a_committed_file(&fixture);
    assert_eq!(commit_through_the_fixture_service(&fixture, repo.as_path(), "through the earlier word"), 1);
    remove_settled(&fixture);
}
