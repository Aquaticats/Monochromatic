//! What:
//!  A commit killed at a phase marker whose transaction files are then damaged the way a power loss can damage them,
//!  and the outcome of the next wrapped command: recovery completes the landing, discards it, or fails closed.
//! Why:
//!  A `SIGKILL` leaves the page cache intact, so a kill alone never shows what a power loss does to files that were
//!  written but not yet synced. The damage step is the test: each file the paused commit left is, in turn, emptied,
//!  cut to half its length, or deleted while the later records stay (a flush that reached the disk out of order).
//!  The table of outcomes decides which further syncs the journal may drop; it is recorded in
//!  `doc/planning/cli-git-spawn-and-sync-reduction.md`. An outcome that is not the recorded one fails the control,
//!  so a change to recovery has to update the record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of filesLeftAt(phase)) for (const damage of ['empty', 'half', 'delete']) {
//!   const repo = await killCommitAt(phase); await damageFile(file, damage);
//!   expect(await classify(repo)).toBe(recorded[`${phase}|${file}|${damage}`]);
//! }
//! ```

/// The recorded outcome of every case.
#[path = "binary_recovery_tearing_expected.rs"]
mod expected;

use super::commit::{git_text, keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::support::{Fixture, bounded_for, direct, fixture, git, repository, wrapper_first_path};
use git_policy_cli::transaction_registry::is_transaction_id;
use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Environment variable naming a file that receives one line per case instead of the assertions,
///  which is how the recorded table is regenerated.
const REPORT_VARIABLE: &str = "CLI_GIT_TEARING_REPORT";

/// How long one probe may run before it counts as hung, in seconds.
const PROBE_BOUND_SECONDS: &str = "30";

/// One kind of damage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Damage {
    /// The file keeps its name and loses every byte.
    Empty,
    /// The file keeps its name and the first half of its bytes.
    Half,
    /// The file is gone while the records written after it stay.
    Dropped,
}

/// Every damage kind, in the order the table lists them.
const DAMAGES: [Damage; 3] = [Damage::Empty, Damage::Half, Damage::Dropped];

impl Damage {
    /// What:
    ///  The name the table uses.
    /// Why:
    ///   Keys of the recorded outcomes spell it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// damage // 'empty' | 'half' | 'delete'
    /// ```
    fn name(self) -> &'static str {
        match self {
            Damage::Empty => return "empty",
            Damage::Half => return "half",
            Damage::Dropped => return "delete",
        }
    }
}

/// What a probe saw.
struct Probe {
    /// Exit code, absent when the probe was killed by its time bound.
    code: Option<i32>,
    /// Standard error as text.
    stderr: String,
    /// Standard output as text.
    stdout: String,
}

/// A repository whose commit was killed at a phase.
struct Paused {
    /// The fixture, removed with the case.
    fixture: Fixture,
    /// The repository.
    repo: PathBuf,
    /// `HEAD` before the commit.
    base: String,
}

/// What:
///  Replace each canonical UUID in a path with a placeholder, each run of 40 hexadecimal digits (a commit name) with a
///  second, and each run of six or more digits with a third.
/// Why:
///   Transaction and lock-candidate names carry UUIDs, process IDs and clock readings that change from run to run;
///  the table keys must not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.replaceAll(/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/g, '<id>').replaceAll(/\d{6,}/g, '<n>')
/// ```
fn normalized(path: &str) -> String {
    let mut text: String = String::new();
    let mut index: usize = 0;
    while index < path.len() {
        let window: Option<&str> = path.get(index..index + 36);
        let digits: usize = path[index..].bytes().take_while(u8::is_ascii_digit).count();
        let object_name: bool = path.get(index..index + 40).is_some_and(|name| { return name.bytes().all(|byte| { return byte.is_ascii_hexdigit(); }); });
        if window.is_some_and(is_transaction_id) {
            text.push_str("<id>");
            index += 36;
        } else if object_name {
            // A landed-capture record is named by the commit it recorded.
            text.push_str("<commit>");
            index += 40;
        } else if digits >= 6 {
            // Process IDs and nanosecond clock readings in scratch names.
            text.push_str("<n>");
            index += digits;
        } else {
            let next: usize = index + path[index..].chars().next().map_or(1, char::len_utf8);
            text.push_str(&path[index..next]);
            index = next;
        }
    }
    return text;
}

/// What:
///  Every regular file under the places a landing keeps its state, as paths relative to the Git directory.
/// Why:
///   The journal, the lock candidates, the capture store and the real index lock are what a power loss can tear;
///  links (the hook dispatcher entries) and directories are not files that carry bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const files = await Promise.all(roots.map(walkRegularFiles));
/// ```
fn state_files(git_dir: &Path) -> Vec<PathBuf> {
    let mut pending: Vec<PathBuf> = ["cli-git-transactions", "cli-git-captures", "cli-git/hook.lock", "index.lock", "index~pid.lock"]
        .iter()
        .map(|name| { return git_dir.join(name); })
        .collect();
    let mut found: Vec<PathBuf> = Vec::new();
    while let Some(path) = pending.pop() {
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.is_file() {
            found.push(path);
        } else if metadata.is_dir() {
            for entry in std::fs::read_dir(&path).expect("state directory").flatten() {
                pending.push(entry.path());
            }
        }
    }
    found.sort();
    return found;
}

/// What:
///  The key of a state file: its path under the Git directory with UUIDs replaced.
/// Why:
///   One name per file kind across runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// normalize(relative(gitDir, file))
/// ```
fn relative_key(git_dir: &Path, file: &Path) -> String {
    return normalized(file.strip_prefix(git_dir).expect("under the Git directory").to_string_lossy().as_ref());
}

/// What:
///  Run a probe command in the repository with the wrapper first on PATH and a short bound.
/// Why:
///   A recovery that waits forever on damaged lock evidence must end as a finding, not as a stuck suite.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync('git', args, { cwd: repo, timeout: 30_000 })
/// ```
fn probe(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Probe {
    let mut command: Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        PROBE_BOUND_SECONDS,
    );
    command.current_dir(repo).args(arguments).stdin(Stdio::null());
    let output = command.output().expect("the probe runs");
    return Probe {
        code: output.status.code().filter(|code| { return *code != 137; }),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    };
}

/// What:
///  A repository with a tracked file holding an uncommitted edit, and its commit killed at a phase.
/// Why:
///   The kill marker ends the wrapper with `SIGKILL` the moment the phase is reached, with every lock and record of
///  that moment on disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = await editedRepository(); await commitKilledAt(repo, phase);
/// ```
fn pause_at(phase: &str) -> Paused {
    let case_fixture: Fixture = fixture(format!("tearing-{phase}").as_str());
    let repo: PathBuf = repository(&case_fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(&case_fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(&case_fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
    keep_maintenance_in_the_foreground(&case_fixture, repo.as_path());
    let base: String = git_text(&case_fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let markers: PathBuf = case_fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("marker directory");
    let killed = wrapped_commit(&case_fixture, repo.as_path(), &["commit", "--message=killed", "a.txt"])
        .env("CLI_GIT_TEST_ONLY_PHASE_SIGNAL", format!("{phase}:kill:{}", markers.display()))
        .stdin(Stdio::null())
        .output()
        .expect("the killed commit runs");
    assert!(
        markers.join(format!("{phase}.reached")).exists(),
        "the commit reached {phase}: {}",
        String::from_utf8_lossy(&killed.stderr)
    );
    return Paused { fixture: case_fixture, repo, base };
}

/// What:
///  Apply one damage to one file and prove it took effect.
/// Why:
///   A damage step that silently did nothing would make every outcome look like the undamaged one.
///  Returns false when the damage does not apply (cutting a file shorter than two bytes to half is emptying it).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await truncate(file, length / 2); expect((await stat(file)).size).toBe(length / 2);
/// ```
fn apply(damage: Damage, file: &Path) -> bool {
    let length: u64 = std::fs::metadata(file).expect("the file to damage").len();
    match damage {
        Damage::Empty => {
            std::fs::OpenOptions::new().write(true).open(file).expect("open").set_len(0).expect("truncate");
            assert_eq!(std::fs::metadata(file).expect("emptied").len(), 0);
            return length > 0;
        }
        Damage::Half => {
            if length < 2 {
                return false;
            }
            std::fs::OpenOptions::new().write(true).open(file).expect("open").set_len(length / 2).expect("truncate");
            assert_eq!(std::fs::metadata(file).expect("halved").len(), length / 2);
            return true;
        }
        Damage::Dropped => {
            std::fs::remove_file(file).expect("delete");
            assert!(!file.exists());
            return true;
        }
    }
}

/// What:
///  Whether anything of the commit is left behind.
/// Why:
///   A recovery that reports success but leaves a transaction, a lock, a keep or a shadow behind has not finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await leftovers(gitDir)).length > 0
/// ```
fn has_residue(git_dir: &Path) -> bool {
    let registry_entries: usize = std::fs::read_dir(git_dir.join("cli-git-transactions")).map_or(0, Iterator::count);
    let shadows: usize = std::fs::read_dir(git_dir.join("cli-git/shadow")).map_or(0, Iterator::count);
    let keeps: usize = std::fs::read_dir(git_dir.join("objects/pack")).map_or(0, |entries| {
        return entries.flatten().filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".keep"); }).count();
    });
    return registry_entries + shadows + keeps > 0 || git_dir.join("index.lock").exists();
}

/// What:
///  `HEAD` as real Git reads it, or nothing when real Git cannot say.
/// Why:
///   The check must not take locks a stuck recovery left behind, and a failure is a finding, not a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await execFile('/usr/bin/git', ['rev-parse', 'HEAD'])).stdout.trim()
/// ```
fn real_git_head(paused: &Paused) -> Option<String> {
    let output = direct(&paused.fixture)
        .current_dir(paused.repo.as_path())
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("real Git runs");
    if !output.status.success() {
        return None;
    }
    return Some(String::from_utf8_lossy(&output.stdout).trim().to_owned());
}

/// What:
///  The outcome of the next wrapped commands after a damage, as one word and a first line of what the user was told.
/// Why:
///   The word is what the table records: `completes` (the commit landed and the index matches),
///  `discards` (nothing landed and the edit is intact), `fails-closed` (exit 2, then a later command works),
///  `sticks` (exit 2 on a repeat), `hangs` (the bound ended it), `inconsistent` (exit 0 with a repository that
///  is neither of the first two), or `exit-N`. A `+residue` suffix marks leftovers, and `!` marks an unexpected
///  result of the commit that follows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await classify(paused); // 'completes' | 'discards' | 'fails-closed' | 'sticks' | ...
/// ```
fn classify(paused: &Paused) -> (String, String) {
    let git_dir: PathBuf = paused.repo.join(".git");
    let first: Probe = probe(&paused.fixture, paused.repo.as_path(), &["status", "--porcelain=v1"]);
    let note: String = first.stderr.lines().next().unwrap_or("").chars().take(700).collect();
    let word: &str = match first.code {
        None => "hangs",
        Some(0) => {
            // The porcelain status shows the index against `HEAD` in its first column and the worktree against
            // the index in its second, so `HEAD` and the status together say which of the two states this is.
            let head: Option<String> = real_git_head(paused);
            if head.as_deref() == Some(paused.base.as_str()) && first.stdout == " M a.txt\n" {
                "discards"
            } else if head.as_deref().is_some_and(|moved| { return moved != paused.base; }) && first.stdout.is_empty() {
                "completes"
            } else {
                "inconsistent"
            }
        }
        Some(2) => {
            let second: Probe = probe(&paused.fixture, paused.repo.as_path(), &["status", "--porcelain=v1"]);
            if second.code == Some(2) { "sticks" } else { "fails-closed" }
        }
        Some(_) => "other-exit",
    };
    let mut cell: String = match (word, first.code) {
        ("other-exit", Some(code)) => format!("exit-{code}"),
        (kept, _) => kept.to_owned(),
    };
    if matches!(word, "completes" | "discards") && has_residue(git_dir.as_path()) {
        cell.push_str("+residue");
    }
    std::fs::write(paused.repo.join("a.txt"), b"one\ntwo\nthree\n").expect("another edit");
    let next: Probe = probe(&paused.fixture, paused.repo.as_path(), &["commit", "--message=after", "a.txt"]);
    let healthy: bool = matches!(word, "completes" | "discards");
    match (healthy, next.code) {
        (true, Some(0)) | (false, Some(2)) => {}
        (_, Some(code)) => cell.push_str(format!("!commit-exit-{code}").as_str()),
        (_, None) => cell.push_str("!commit-hangs"),
    }
    return (cell, note);
}

/// What:
///  Run one case: kill at the phase, damage one file (or none), and classify what the next command does.
/// Why:
///   Every case starts from a fresh repository so no case sees another's recovery.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paused = await pauseAt(phase); damage && await apply(damage, file); return classify(paused);
/// ```
fn run_case(phase: &str, target: Option<(&str, Damage)>) -> Option<(String, String)> {
    let paused: Paused = pause_at(phase);
    let git_dir: PathBuf = paused.repo.join(".git");
    if let Some((key, damage)) = target {
        let matching: Vec<PathBuf> = state_files(git_dir.as_path())
            .into_iter()
            .filter(|file| { return relative_key(git_dir.as_path(), file) == key; })
            .collect();
        assert_eq!(matching.len(), 1, "exactly one file is named {key} at {phase}: {matching:?}");
        if !apply(damage, matching[0].as_path()) {
            remove_settled(&paused.fixture);
            return None;
        }
    }
    let outcome: (String, String) = classify(&paused);
    remove_settled(&paused.fixture);
    return Some(outcome);
}

/// What:
///  Record or check one case's outcome.
/// Why:
///   With the report variable set the line is appended to the report file; otherwise the outcome must equal the
///  recorded one, and a case with no record fails so a new file kind is classified on purpose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// report ? append(report, line) : expect(cell).toBe(recorded[key]);
/// ```
fn settle(key: &str, cell: &str, note: &str) {
    if let Some(report) = std::env::var_os(REPORT_VARIABLE) {
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(report).expect("report file");
        writeln!(file, "{key}\t{cell}\t{note}").expect("report line");
        return;
    }
    let recorded: Option<&str> = expected::OUTCOMES
        .iter()
        .find(|(name, _)| { return *name == key; })
        .map(|(_, outcome)| { return *outcome; });
    assert_eq!(recorded, Some(cell), "{key}: {note}");
}

/// What:
///  Run every case of one phase: the undamaged control, then each state file under each damage.
/// Why:
///   The control proves what the phase does without damage, so a damaged case that differs is the damage's doing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const file of await filesLeftAt(phase)) for (const damage of DAMAGES) settle(await runCase(phase, file, damage));
/// ```
fn sweep(phase: &str) {
    let control: Option<(String, String)> = run_case(phase, None);
    let (control_cell, control_note): (String, String) = control.expect("the control always runs");
    settle(format!("{phase}|(undamaged)|none").as_str(), control_cell.as_str(), control_note.as_str());
    // The control counts as one case.
    let mut settled: usize = 1;
    let listing: Paused = pause_at(phase);
    let git_dir: PathBuf = listing.repo.join(".git");
    let mut keys: Vec<String> = state_files(git_dir.as_path())
        .iter()
        .map(|file| { return relative_key(git_dir.as_path(), file); })
        .collect();
    remove_settled(&listing.fixture);
    keys.sort();
    for key in &keys {
        for damage in DAMAGES {
            if let Some((cell, note)) = run_case(phase, Some((key.as_str(), damage))) {
                settle(format!("{phase}|{key}|{}", damage.name()).as_str(), cell.as_str(), note.as_str());
                settled += 1;
            }
        }
    }
    if std::env::var_os(REPORT_VARIABLE).is_none() {
        let recorded: usize = expected::OUTCOMES.iter().filter(|(name, _)| { return name.starts_with(format!("{phase}|").as_str()); }).count();
        assert_eq!(settled, recorded, "every recorded case of {phase} ran, and no other");
    }
}

/// What:
///  A damaged journal record stops the next wrapped command with a message that names the file and the transaction
///  directory, says what the person can do and what the wrapper then does, and carries no hint about fetching history.
/// Why:
///   The recorded outcome says a command sticks; this says what the person who is stuck reads.
///  Each case damages one record of a real killed commit and reads the standard error of the next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [phase, file, phrase] of cases) expect(await stderrAfterDamage(phase, file)).toContain(phrase);
/// ```
#[test]
fn the_messages_of_damaged_records_name_the_file_and_the_ways_out() {
    let cases: [(&str, &str, &str); 4] = [
        ("capture-locked", "preparing.json", "preparing.json is the first record of a transaction"),
        ("landing-locked", "index-lock-1.json", "This record names the index.lock file the transaction created"),
        ("ref-updated", "landing-1.json", "A landing record names the post-index"),
        ("ref-updated", "ref-updated.json", "This record is written after the branch moved"),
    ];
    for (phase, file_name, phrase) in cases {
        for damage in [Damage::Empty, Damage::Half] {
            let paused: Paused = pause_at(phase);
            let git_dir: PathBuf = paused.repo.join(".git");
            let matching: Vec<PathBuf> = state_files(git_dir.as_path())
                .into_iter()
                .filter(|file| { return file.file_name().is_some_and(|name| { return name == file_name; }); })
                .collect();
            assert_eq!(matching.len(), 1, "exactly one {file_name} at {phase}: {matching:?}");
            assert!(apply(damage, matching[0].as_path()), "{file_name} can be damaged by {}", damage.name());
            let first: Probe = probe(&paused.fixture, paused.repo.as_path(), &["status", "--porcelain=v1"]);
            let second: Probe = probe(&paused.fixture, paused.repo.as_path(), &["status", "--porcelain=v1"]);
            let directory: &Path = matching[0].parent().expect("the record sits in a transaction directory");
            let context: String = format!("{phase} {file_name} {}: {}", damage.name(), first.stderr);
            assert_eq!(first.code, Some(2), "{context}");
            assert_eq!(second.code, Some(2), "{context}");
            assert!(first.stderr.contains(matching[0].to_string_lossy().as_ref()), "the file is named: {context}");
            assert!(first.stderr.contains(directory.to_string_lossy().as_ref()), "the directory is named: {context}");
            assert!(first.stderr.contains(phrase), "the way out is named: {context}");
            assert!(first.stderr.contains("Every guarded command stops") || first.stderr.contains("every guarded command stops"), "{context}");
            assert!(!first.stderr.contains("git fetch") && !first.stderr.contains("\"hint\""), "no inapplicable hint: {context}");
            remove_settled(&paused.fixture);
        }
    }
}

/// A commit killed with its capture lock held.
#[test]
fn damage_at_capture_locked() {
    sweep("capture-locked");
}

/// A commit killed after preparation.
#[test]
fn damage_at_preparation_done() {
    sweep("preparation-done");
}

/// A commit killed with the landing and index locks held.
#[test]
fn damage_at_landing_locked() {
    sweep("landing-locked");
}

/// A commit killed after the objects migrated into the kept pack.
#[test]
fn damage_at_objects_migrated() {
    sweep("objects-migrated");
}

/// A commit killed after the branch advanced.
#[test]
fn damage_at_ref_updated() {
    sweep("ref-updated");
}

/// A commit killed after the index was installed.
#[test]
fn damage_at_index_installed() {
    sweep("index-installed");
}
