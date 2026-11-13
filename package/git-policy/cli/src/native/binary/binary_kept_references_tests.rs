//! What:
//!  `git monochromatic-git-policy-cli kept-references` and the pruning it lists,
//!  through the built executable:
//!  which of the references the wrapper keeps commits on go,
//!  which stay,
//!  what the log says about one that went,
//!  and what a person is told before it goes.
//! Why:
//!  The pruning removes the only handle on a commit no branch reaches,
//!  so its rules are promises about the program a caller runs as `git`,
//!  and every control here runs that program in a fixture of its own.
//!  The limits the release ships are a month and 64 MiB,
//!  which no control can reach in a fixture,
//!  so each control sets the three variables the release reads
//!  (`kept_reference_limits.rs`)
//!  and judges the same rules a repository would meet them by.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listing = run(['monochromatic-git-policy-cli', 'kept-references'], env); expect(namesItPrunes(listing)).toEqual(namesThePassPrunes());
//! ```

/// The fixtures and bounded process helpers every binary-level control shares.
use super::support::{Fixture, Observed, fixture, git, hermetic, observe, remove, repository, wrapped};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

/// The variable that sets the age limit, in whole seconds.
const MAX_AGE_VARIABLE: &str = "CLI_GIT_KEPT_REFERENCE_MAX_AGE_SECONDS";

/// The variable that sets the size limit, in bytes.
const MAX_BYTES_VARIABLE: &str = "CLI_GIT_KEPT_REFERENCE_MAX_BYTES";

/// The variable that sets the floor under which no reference is pruned for its size.
const FLOOR_VARIABLE: &str = "CLI_GIT_KEPT_REFERENCE_FLOOR_SECONDS";

/// The sentence a listing prints for a reference the next pass prunes.
const PRUNED_NEXT: &str = "the next pass prunes it:";

/// The command word of the wrapper's own namespace.
const COMMAND: &str = "monochromatic-git-policy-cli";

/// How long a control waits before it asks whether a reference kept a moment ago is old enough yet:
/// longer than the one-second age a control sets, and short enough that a control stays quick.
const A_MOMENT: Duration = Duration::from_millis(1_200);

/// How long a control waits for the commit service's upkeep to reach the pass it runs by itself.
const UPKEEP_PATIENCE: Duration = Duration::from_secs(120);

/// How often a control looks for what the upkeep did.
const UPKEEP_POLL: Duration = Duration::from_millis(100);

/// How long a control waits for a paused pass to reach its marker.
const MARKER_PATIENCE: Duration = Duration::from_secs(60);

/// How often a control looks for a marker.
const MARKER_POLL: Duration = Duration::from_millis(20);

/// A fixture and the one repository its controls keep references in.
struct World {
    /// The disposable fixture, removed by the control.
    fixture: Fixture,
    /// The repository's root.
    repo: PathBuf,
}

impl World {
    /// The repository's common Git directory,
    /// which is where the records and the log are.
    fn common_dir(&self) -> PathBuf {
        return self.repo.join(".git");
    }
}

/// A fixture with one repository whose `main` holds one empty commit.
fn world(name: &str) -> World {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    return World { fixture, repo };
}

/// A commit that no branch names,
/// made from the tree `HEAD` holds with a message of its own.
fn a_commit_no_branch_names(world: &World, message: &str) -> String {
    let tree: String = String::from_utf8_lossy(&git(&world.fixture, world.repo.as_path(), &["rev-parse", "HEAD^{tree}"]).stdout).trim().to_owned();
    let made: Output = git(&world.fixture, world.repo.as_path(), &["commit-tree", tree.as_str(), "-m", message]);
    return String::from_utf8_lossy(made.stdout.as_slice()).trim().to_owned();
}

/// Write one reference of the repository with real Git,
/// as a keeper of the wrapper does.
fn reference(world: &World, name: &str, commit: &str) {
    git(&world.fixture, world.repo.as_path(), &["update-ref", "--no-deref", name, commit]);
}

/// Every reference the repository holds under `refs/cli-git/`.
fn references(world: &World) -> Vec<String> {
    let listed: Output = git(&world.fixture, world.repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/cli-git/"]);
    return String::from_utf8_lossy(listed.stdout.as_slice()).lines().map(String::from).collect();
}

/// What one commit a reference names,
/// or nothing when the reference is gone.
fn names(world: &World, name: &str) -> Option<String> {
    let output: Output = super::support::native_git(&world.fixture, world.repo.as_path(), &["rev-parse", "--verify", "--quiet", name]);
    if output.status.success() {
        return Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned());
    }
    return None;
}

/// What:
///  One wrapped management command in the fixture,
///  with the variables a control sets for it.
/// Why:
///   Every control here judges one rule,
///  so it sets that rule's limit and leaves the others as the release ships them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// run(world, ['monochromatic-git-policy-cli', 'kept-references', '--prune'], { MAX_AGE: '1' })
/// ```
fn management(world: &World, arguments: &[&str], environment: &[(&str, &str)]) -> Observed {
    let mut command: Command = wrapped(&world.fixture);
    command.current_dir(world.repo.as_path()).args([COMMAND]).args(arguments);
    for (name, value) in environment {
        command.env(name, value);
    }
    return observe(&mut command, b"");
}

/// The listing of the repository's kept references.
fn listing(world: &World, environment: &[(&str, &str)]) -> String {
    let observed: Observed = management(world, &["kept-references"], environment);
    assert_eq!(observed.code, Some(0), "the listing answers: {}", String::from_utf8_lossy(observed.stderr.as_slice()));
    return String::from_utf8_lossy(observed.stdout.as_slice()).into_owned();
}

/// One pruning of the repository's kept references.
fn pruning(world: &World, environment: &[(&str, &str)]) -> String {
    let observed: Observed = management(world, &["kept-references", "--prune"], environment);
    assert_eq!(observed.code, Some(0), "the pass answers: {}", String::from_utf8_lossy(observed.stderr.as_slice()));
    return String::from_utf8_lossy(observed.stdout.as_slice()).into_owned();
}

/// The names a listing's blocks say the next pass prunes,
/// in the order the listing prints them.
fn names_the_listing_prunes(text: &str) -> Vec<String> {
    let mut pruned: Vec<String> = Vec::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if line.starts_with("refs/") {
            current = Some(String::from(line));
        } else if line.trim_start().starts_with(PRUNED_NEXT)
            && let Some(name) = current.take()
        {
            pruned.push(name);
        }
    }
    return pruned;
}

/// The names a pruning's report says it pruned,
/// in the order it printed them.
fn names_the_report_prunes(text: &str) -> Vec<String> {
    let mut pruned: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("pruned ")
            && let Some((name, _why)) = rest.split_once(' ')
        {
            pruned.push(String::from(name));
        }
    }
    return pruned;
}

/// The lines of the repository's log of pruned kept references,
/// or none when the log was never written.
fn log_lines(world: &World) -> Vec<String> {
    return std::fs::read_to_string(world.common_dir().join("monochromatic-git-policy-cli-kept-references.jsonl"))
        .unwrap_or_default()
        .lines()
        .filter(|line| { return !line.trim().is_empty(); })
        .map(String::from)
        .collect();
}

/// A pass over the repository that has listed its references and removed none yet.
struct Paused {
    /// The pass waiting for its release.
    child: Child,
    /// Where the marker and the release file go.
    markers: PathBuf,
}

/// Start a pruning that stops between its listing and its removals.
fn paused_pass(world: &World, environment: &[(&str, &str)]) -> Paused {
    let markers: PathBuf = world.fixture.root.join("markers-kept-references");
    std::fs::create_dir(&markers).expect("marker directory");
    let mut command: Command = wrapped(&world.fixture);
    command
        .current_dir(world.repo.as_path())
        .args([COMMAND, "kept-references", "--prune"])
        .env(PHASE_SIGNAL_VARIABLE, format!("kept-references-listed:pause:{}", markers.display()))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in environment {
        command.env(name, value);
    }
    let child: Child = command.spawn().expect("the pass starts");
    let paused: Paused = Paused { child, markers };
    paused.wait_for_the_listing();
    return paused;
}

impl Paused {
    /// Wait until the pass has listed the references,
    /// which is what says a control may act between its two steps.
    fn wait_for_the_listing(&self) {
        let marker: PathBuf = self.markers.join("kept-references-listed.reached");
        let began: Instant = Instant::now();
        while !marker.exists() {
            assert!(began.elapsed() < MARKER_PATIENCE, "the pass did not list within {} seconds", MARKER_PATIENCE.as_secs());
            std::thread::sleep(MARKER_POLL);
        }
    }

    /// Let the pass go on and read everything it said.
    fn released(self) -> Output {
        std::fs::write(self.markers.join("kept-references-listed.release"), b"").expect("the release file");
        return self.child.wait_with_output().expect("the pass ends");
    }
}

/// The commit service prunes a kept reference by itself, in the upkeep that runs after a landed commit,
/// with no command asking for a pass.
#[test]
fn the_upkeep_prunes_with_no_command_asking_for_a_pass() {
    let world: World = world("kept-references-upkeep");
    // The service reads the limits from its own environment, which the fixture gives it.
    world
        .fixture
        .service_environment
        .lock()
        .expect("the fixture's service environment")
        .push((std::ffi::OsString::from(MAX_AGE_VARIABLE), std::ffi::OsString::from("1")));
    let commit: String = a_commit_no_branch_names(&world, "a commit the upkeep prunes");
    reference(&world, "refs/cli-git/kept/1794071191963960", commit.as_str());
    // A command of the wrapper dates the reference at that moment; the service's own limit then finds it past the age.
    assert!(pruning(&world, &[(MAX_AGE_VARIABLE, "3600")]).contains("No kept reference was pruned"));
    std::thread::sleep(A_MOMENT);
    // A person's commit is what brings the service to its upkeep, and the upkeep is where the pass runs by itself.
    let landed: Observed = super::commit::run_commit(&world.fixture, world.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message", "a commit that wakes the upkeep"]);
    assert_eq!(landed.code, Some(0), "the commit landed: {}", String::from_utf8_lossy(landed.stderr.as_slice()));
    let began: Instant = Instant::now();
    while names(&world, "refs/cli-git/kept/1794071191963960").is_some() {
        assert!(began.elapsed() < UPKEEP_PATIENCE, "the upkeep pruned the reference within {} seconds", UPKEEP_PATIENCE.as_secs());
        std::thread::sleep(UPKEEP_POLL);
    }
    let lines: Vec<String> = log_lines(&world);
    assert_eq!(lines.len(), 1, "the upkeep wrote the line about what it pruned: {lines:?}");
    assert!(lines[0].contains("too-old"), "{lines:?}");
    remove(&world.fixture);
}

/// That fixture's wrapper gets the fixture's own home and runtime directory,
/// so no control reaches the owner's.
#[test]
fn the_fixture_environment_is_the_controls_own() {
    let world: World = world("kept-references-environment");
    let mut command: Command = wrapped(&world.fixture);
    hermetic(&mut command, &world.fixture, OsStr::new("/usr/bin:/bin"));
    assert_eq!(command.get_envs().filter(|(name, _value)| { return *name == OsStr::new("HOME"); }).count(), 1);
    remove(&world.fixture);
}

/// A kept commit a branch has since reached holds nothing,
/// so its reference goes at once,
/// however young it is,
/// and the log says no commit was lost.
#[test]
fn a_kept_commit_a_branch_reaches_is_pruned() {
    let world: World = world("kept-references-reached");
    let commit: String = a_commit_no_branch_names(&world, "a commit a branch will reach");
    reference(&world, "refs/cli-git/kept/1794071191963902", commit.as_str());
    // The branch moves onto the kept commit, so the reference holds nothing any more.
    git(&world.fixture, world.repo.as_path(), &["update-ref", "refs/heads/main", commit.as_str()]);
    let report: String = pruning(&world, &[]);
    assert_eq!(names(&world, "refs/cli-git/kept/1794071191963902"), None, "the reference went: {report}");
    assert_eq!(names_the_report_prunes(report.as_str()), vec![String::from("refs/cli-git/kept/1794071191963902")], "{report}");
    let lines: Vec<String> = log_lines(&world);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains(commit.as_str()), "the line names the commit: {}", lines[0]);
    assert!(lines[0].contains("reached-a-branch"), "the line names the rule: {}", lines[0]);
    assert!(lines[0].contains("so nothing was lost"), "the line says nothing was lost: {}", lines[0]);
    remove(&world.fixture);
}

/// A kept commit no branch reaches,
/// younger than the age limit,
/// stays,
/// and the log says nothing.
#[test]
fn a_kept_commit_younger_than_the_age_stays() {
    let world: World = world("kept-references-young");
    let commit: String = a_commit_no_branch_names(&world, "a commit kept a moment ago");
    reference(&world, "refs/cli-git/kept/1794071191963903", commit.as_str());
    let report: String = pruning(&world, &[(MAX_AGE_VARIABLE, "3600")]);
    assert_eq!(names(&world, "refs/cli-git/kept/1794071191963903"), Some(commit), "the reference stays: {report}");
    assert!(report.contains("No kept reference was pruned"), "{report}");
    assert!(log_lines(&world).is_empty(), "nothing was pruned, so nothing is logged: {:?}", log_lines(&world));
    remove(&world.fixture);
}

/// A kept commit no branch reaches that is older than the age limit goes,
/// and the line about it says the reference,
/// the commit,
/// its age and the reason.
#[test]
fn a_kept_commit_older_than_the_age_goes_and_the_log_says_so() {
    let world: World = world("kept-references-old");
    let commit: String = a_commit_no_branch_names(&world, "a commit nothing reaches");
    reference(&world, "refs/cli-git/kept/1794071191963904", commit.as_str());
    // The first pass records when it first saw the reference, as it does for one an earlier release wrote.
    assert!(pruning(&world, &[(MAX_AGE_VARIABLE, "3600")]).contains("No kept reference was pruned"));
    std::thread::sleep(A_MOMENT);
    let report: String = pruning(&world, &[(MAX_AGE_VARIABLE, "1")]);
    assert_eq!(names(&world, "refs/cli-git/kept/1794071191963904"), None, "the reference went: {report}");
    let lines: Vec<String> = log_lines(&world);
    assert_eq!(lines.len(), 1, "{lines:?}");
    for expected in ["refs/cli-git/kept/1794071191963904", commit.as_str(), "\"age_seconds\":1", "too-old", "reached no branch"] {
        assert!(lines[0].contains(expected), "the line names {expected}: {}", lines[0]);
    }
    remove(&world.fixture);
}

/// While the objects only kept references reach weigh more than the size limit,
/// the oldest goes first,
/// and a reference younger than the floor stays even though the limit is still exceeded.
#[test]
fn over_the_size_the_oldest_go_first_and_a_young_one_stays() {
    let world: World = world("kept-references-size");
    let oldest: String = a_commit_no_branch_names(&world, "the oldest kept commit");
    let middle: String = a_commit_no_branch_names(&world, "the middle kept commit");
    let youngest: String = a_commit_no_branch_names(&world, "the youngest kept commit");
    // Two references are kept more than the two-second floor apart, and one after them, so the floor lets two go.
    reference(&world, "refs/cli-git/kept/1794071191963910", oldest.as_str());
    assert!(pruning(&world, &[(MAX_AGE_VARIABLE, "3600")]).contains("No kept reference was pruned"));
    std::thread::sleep(2 * A_MOMENT);
    reference(&world, "refs/cli-git/kept/1794071191963911", middle.as_str());
    assert!(pruning(&world, &[(MAX_AGE_VARIABLE, "3600")]).contains("No kept reference was pruned"));
    std::thread::sleep(2 * A_MOMENT);
    reference(&world, "refs/cli-git/kept/1794071191963912", youngest.as_str());
    assert!(pruning(&world, &[(MAX_AGE_VARIABLE, "3600")]).contains("No kept reference was pruned"));
    // One byte of limit: the rule fires. Two seconds of floor: the two oldest may go, the youngest may not.
    let report: String = pruning(&world, &[(MAX_AGE_VARIABLE, "3600"), (MAX_BYTES_VARIABLE, "1"), (FLOOR_VARIABLE, "2")]);
    assert_eq!(names(&world, "refs/cli-git/kept/1794071191963910"), None, "the oldest went: {report}");
    assert_eq!(names(&world, "refs/cli-git/kept/1794071191963911"), None, "the middle went: {report}");
    assert_eq!(names(&world, "refs/cli-git/kept/1794071191963912"), Some(youngest), "the youngest is under the floor: {report}");
    assert_eq!(names_the_report_prunes(report.as_str()), vec![String::from("refs/cli-git/kept/1794071191963910"), String::from("refs/cli-git/kept/1794071191963911")], "{report}");
    let lines: Vec<String> = log_lines(&world);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines.iter().all(|line| { return line.contains("too-much-kept"); }), "{lines:?}");
    remove(&world.fixture);
}

/// What else lives under `refs/cli-git/` is no kept commit,
/// and no pass touches it.
#[test]
fn a_reference_that_is_no_kept_commit_is_never_touched() {
    let world: World = world("kept-references-other");
    let carried: String = a_commit_no_branch_names(&world, "a commit an isolated merge carries");
    let fetched: String = a_commit_no_branch_names(&world, "a commit a private fetch names");
    let kept: String = a_commit_no_branch_names(&world, "a commit the wrapper keeps");
    reference(&world, "refs/cli-git/carried/7", carried.as_str());
    reference(&world, "refs/cli-git/auto-push/fetch-4242", fetched.as_str());
    reference(&world, "refs/cli-git/kept/1794071191963920", kept.as_str());
    assert!(pruning(&world, &[(MAX_AGE_VARIABLE, "3600")]).contains("No kept reference was pruned"));
    std::thread::sleep(A_MOMENT);
    let report: String = pruning(&world, &[(MAX_AGE_VARIABLE, "1")]);
    assert_eq!(names(&world, "refs/cli-git/kept/1794071191963920"), None, "the kept reference went: {report}");
    assert_eq!(names(&world, "refs/cli-git/carried/7"), Some(carried), "the carried work stays: {report}");
    assert_eq!(names(&world, "refs/cli-git/auto-push/fetch-4242"), Some(fetched), "the private fetch stays: {report}");
    remove(&world.fixture);
}

/// A reference written after a pass has listed the repository is not one that pass removes,
/// and a listed reference another command moves is left at its new commit.
#[test]
fn a_reference_written_during_a_pass_survives_it() {
    let world: World = world("kept-references-during");
    let old: String = a_commit_no_branch_names(&world, "a commit the pass lists");
    let moved: String = a_commit_no_branch_names(&world, "a commit another command writes");
    let later: String = a_commit_no_branch_names(&world, "a commit written while the pass pauses");
    reference(&world, "refs/cli-git/kept/1794071191963930", old.as_str());
    assert!(pruning(&world, &[(MAX_AGE_VARIABLE, "3600")]).contains("No kept reference was pruned"));
    std::thread::sleep(A_MOMENT);
    let paused: Paused = paused_pass(&world, &[(MAX_AGE_VARIABLE, "1")]);
    // The pass has listed the references and removed none. Both writes below happen inside that window.
    reference(&world, "refs/cli-git/kept/1794071191963931", later.as_str());
    reference(&world, "refs/cli-git/kept/1794071191963930", moved.as_str());
    let output: Output = paused.released();
    assert_eq!(output.status.code(), Some(0), "the pass ends well: {}", String::from_utf8_lossy(output.stderr.as_slice()));
    assert_eq!(names(&world, "refs/cli-git/kept/1794071191963931"), Some(later), "the reference written during the pass stays");
    assert_eq!(names(&world, "refs/cli-git/kept/1794071191963930"), Some(moved), "the listed name keeps the commit another command put there");
    remove(&world.fixture);
}

/// Two passes at once end where one pass alone would:
/// both answer,
/// every reference that was old enough is gone once,
/// and the repository holds what a single pass leaves.
#[test]
fn two_passes_at_once_end_where_one_would() {
    let two: World = world("kept-references-two-pass");
    let one: World = world("kept-references-one-pass");
    for world in [&two, &one] {
        for (at, message) in ["the first kept commit", "the second kept commit"].iter().enumerate() {
            let name: String = format!("refs/cli-git/kept/17940711919639{}", 40 + at);
            let commit: String = a_commit_no_branch_names(world, message);
            reference(world, name.as_str(), commit.as_str());
        }
        assert!(pruning(world, &[(MAX_AGE_VARIABLE, "3600")]).contains("No kept reference was pruned"));
    }
    std::thread::sleep(A_MOMENT);
    let paused: Paused = paused_pass(&two, &[(MAX_AGE_VARIABLE, "1")]);
    // The second pass runs while the first waits between its listing and its removals.
    assert!(pruning(&two, &[(MAX_AGE_VARIABLE, "1")]).contains("pruned "));
    let first: Output = paused.released();
    assert_eq!(first.status.code(), Some(0), "the paused pass ends well: {}", String::from_utf8_lossy(first.stderr.as_slice()));
    assert!(pruning(&one, &[(MAX_AGE_VARIABLE, "1")]).contains("pruned "));
    assert_eq!(references(&two), references(&one), "two passes leave what one leaves");
    assert!(references(&two).is_empty(), "both kept references went: {:?}", references(&two));
    let lines: Vec<String> = log_lines(&two);
    assert_eq!(lines.len(), 2, "each reference that went was logged once: {lines:?}");
    remove(&two.fixture);
    remove(&one.fixture);
}

/// The listing names exactly the references the pass that follows it removes,
/// and says what the pass leaves.
#[test]
fn the_listing_says_what_the_next_pass_does() {
    let world: World = world("kept-references-listing");
    let reached: String = a_commit_no_branch_names(&world, "a commit a branch reaches");
    let old: String = a_commit_no_branch_names(&world, "a commit no branch reaches");
    let young: String = a_commit_no_branch_names(&world, "a commit kept a moment ago");
    reference(&world, "refs/cli-git/kept/1794071191963950", reached.as_str());
    reference(&world, "refs/cli-git/kept/1794071191963951", old.as_str());
    // The pass records the references it meets for the first time, as it does for those an earlier release wrote.
    assert!(pruning(&world, &[(MAX_AGE_VARIABLE, "3600")]).contains("No kept reference was pruned"));
    // The branch moves onto one of them, so that reference holds nothing, and the other is left older than the age limit.
    git(&world.fixture, world.repo.as_path(), &["update-ref", "refs/heads/main", reached.as_str()]);
    std::thread::sleep(A_MOMENT);
    reference(&world, "refs/cli-git/kept/1794071191963952", young.as_str());
    let text: String = listing(&world, &[(MAX_AGE_VARIABLE, "1"), (MAX_BYTES_VARIABLE, "1000000")]);
    let said: Vec<String> = names_the_listing_prunes(text.as_str());
    assert_eq!(said, vec![String::from("refs/cli-git/kept/1794071191963950"), String::from("refs/cli-git/kept/1794071191963951")], "the listing says what goes: {text}");
    assert!(text.contains("The next pass prunes 2 of them"), "{text}");
    assert!(text.contains("refs/cli-git/kept/1794071191963952\n"), "the young reference is listed: {text}");
    let report: String = pruning(&world, &[(MAX_AGE_VARIABLE, "1"), (MAX_BYTES_VARIABLE, "1000000")]);
    assert_eq!(names_the_report_prunes(report.as_str()), said, "the pass removes what the listing said: {report}");
    assert_eq!(references(&world), vec![String::from("refs/cli-git/kept/1794071191963952")], "the young reference stays: {report}");
    remove(&world.fixture);
}
