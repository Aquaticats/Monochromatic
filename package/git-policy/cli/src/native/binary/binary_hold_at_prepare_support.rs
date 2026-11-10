//! What:
//!  Holding a wrapped commit at the marker between its preparation and its landing,
//!  landing another session's commit in exactly that window,
//!  and reading what the held commit came to.
//! Why:
//!  The window is one instant,
//! and every control that needs it would otherwise copy the same scaffolding:
//!  the phase signal,
//!  the bound on the held command,
//!  the release files,
//!  and the hang guard's idle hold while the control keeps the commit waiting on purpose.
//!  The controls of `binary_conclusion_on_a_moved_branch_tests.rs` and
//!  `binary_moved_branch_commit_tests.rs` share it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = heldCommit(fixture, repo, name, markers, { arguments: ['commit', '-m', 'x', '--', 'a.txt'] });
//! await waitHeld(held); await anotherSessionCommits(); release(held);
//! ```

use super::commit::git_text;
use super::service_commit::runtime_for;
use super::support::{Fixture, bounded_for, wrapper_first_path};
use super::supervisor::{IdleHold, hold_idle_judgment};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// The longest a control waits for a marker,
/// in seconds,
/// far beyond a healthy run.
/// Why:
///   The preparation of a conclusion runs several Git starts and a policy pass,
/// and this host under a load average above a hundred took most of a minute for one of them;
/// the wait is bounded,
/// never a hang,
/// and the command's own bound is the one that ends work that really is stuck.
const PATIENCE_SECONDS: u64 = 300;

/// The time bound on the held commit of a control,
/// in seconds.
/// Why:
///   A commit prepared again builds itself twice and replays a tree between the builds,
/// and a host under a load average above a hundred took more than the commit bound of every other control,
/// which ended the command with `SIGKILL` and hid what the commit came to.
const HELD_BOUND_SECONDS: &str = "600";

/// How many arrivals at the phase marker the control releases in advance:
/// the bound on how often a commit is prepared again.
const LATER_ARRIVALS: u32 = 4;

/// What a finished wrapped commit came to.
pub(super) struct Ended {
    /// The status the wrapper ended with.
    pub(super) code: Option<i32>,
    /// Everything it wrote to its standard error.
    pub(super) stderr: String,
}

/// A wrapped commit held where it has prepared its commit and has not yet sent it.
pub(super) struct Held {
    child: Child,
    /// Everything the commit wrote to its standard error,
    /// once it has ended.
    stderr: Option<std::thread::JoinHandle<String>>,
    markers: PathBuf,
    /// The hold that suspends the hang guard's idle rule while this control keeps the commit waiting on purpose.
    /// The commit waits for a file this control writes,
    /// which the guard cannot tell from an endless wait on something outside the tree.
    hold: Option<IdleHold>,
}

impl Held {
    /// Everything the commit wrote to its standard error,
    /// read once the commit has ended.
    fn ended(&mut self) -> String {
        let Some(reader) = self.stderr.take() else { return String::new() };
        return reader.join().expect("the reader of standard error");
    }
}

/// What a control gives the held commit:
/// its command line,
/// what its standard input carries,
/// and the environment variables it reads
/// (the editor a line opens, above all).
pub(super) struct Hold<'a> {
    /// The arguments after the wrapper's own name.
    pub(super) arguments: &'a [&'a str],
    /// What standard input carries,
    /// nothing for an empty one.
    pub(super) stdin: Option<&'a [u8]>,
    /// The variables the commit reads,
    /// beyond the hermetic environment every fixture command gets.
    pub(super) environment: &'a [(&'a str, &'a str)],
}

/// The directory a control's phase markers live in,
/// with every later arrival released in advance.
pub(super) fn marker_directory(fixture: &Fixture) -> PathBuf {
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("the marker directory");
    for number in 2..=LATER_ARRIVALS {
        std::fs::write(markers.join(format!("service-prepared-{number}.release")), "").expect("release of a later arrival");
    }
    return markers;
}

/// What:
///  Starts a wrapped commit and holds it at the marker between its preparation and the service.
/// Why:
///  The other command must land while this one has captured its commit and not landed it,
/// which is the one instant the failure needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = heldCommit(fixture, repo, name, markers, { arguments: ['commit', '-m', 'conclude'], stdin: undefined, environment: [] });
/// await waitHeld(held);
/// ```
pub(super) fn held_commit(fixture: &Fixture, repo: &Path, name: &str, markers: &Path, hold: &Hold<'_>) -> Held {
    let mut command: Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        HELD_BOUND_SECONDS,
    );
    command.current_dir(repo).args(hold.arguments);
    for (variable, value) in hold.environment {
        command.env(variable, value);
    }
    command
        .env("XDG_RUNTIME_DIR", runtime_for(name))
        .env(PHASE_SIGNAL_VARIABLE, format!("service-prepared:pause:{}", markers.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    match hold.stdin {
        Some(_) => {
            command.stdin(Stdio::piped());
        }
        None => {
            command.stdin(Stdio::null());
        }
    }
    let mut child: Child = command.spawn().expect("the wrapped commit starts");
    if let Some(input) = hold.stdin
        && let Some(mut pipe) = child.stdin.take()
    {
        use std::io::Write;
        pipe.write_all(input).expect("the held commit reads what this control gives it");
        // Dropped here, which is how a selection following the person's answers learns there are no more.
        drop(pipe);
    }
    let stderr = child.stderr.take().expect("piped standard error");
    let reader = std::thread::spawn(move || {
        let mut all: String = String::new();
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            all.push_str(&line);
            all.push('\n');
        }
        return all;
    });
    let idle: IdleHold = hold_idle_judgment(child.id());
    return Held { child, stderr: Some(reader), markers: markers.to_path_buf(), hold: Some(idle) };
}

/// Waits until the commit is held at the marker between its preparation and its landing,
/// and fails the control with the commit's own words when it instead ended.
pub(super) fn wait_held(held: &mut Held) {
    let reached: PathBuf = held.markers.join("service-prepared-1.reached");
    let began: Instant = Instant::now();
    while !reached.exists() {
        if let Some(status) = held.child.try_wait().expect("the commit can be asked whether it ended") {
            let words: String = held.ended();
            panic!("the commit ended ({status:?}) before it was held: {words}");
        }
        assert!(
            began.elapsed() < Duration::from_secs(PATIENCE_SECONDS),
            "never happened: the commit prepared and is held before the service"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Lets the held commit go on to the service and returns what it came to.
pub(super) fn release(mut held: Held) -> Ended {
    std::fs::write(held.markers.join("service-prepared-1.release"), "").expect("release");
    let status = held.child.wait().expect("the commit ends");
    // The guard judges the commit again once this control is no longer holding it.
    let _held_again: Option<IdleHold> = held.hold.take();
    let stderr: String = held.ended();
    return Ended { code: status.code(), stderr };
}

/// What one commit of a branch is,
/// so a control can say a commit another command made was left as it is.
pub(super) fn stored(fixture: &Fixture, repo: &Path, revision: &str) -> Vec<String> {
    let formats: [&str; 6] = ["%H", "%P", "%T", "%B", "%an <%ae>|%cn <%ce>", "%aI|%cI"];
    return formats.map(|format| { return git_text(fixture, repo, &["log", "-1", "--date=iso-strict", format!("--format={format}").as_str(), revision]); }).to_vec();
}

/// The author of a commit,
/// with the date the commit records,
/// as the person reads it.
pub(super) fn author_of(fixture: &Fixture, repo: &Path, revision: &str) -> String {
    return git_text(fixture, repo, &["log", "-1", "--date=iso-strict", "--format=%an <%ae>|%aI", revision]);
}

/// The parents of `HEAD`,
/// as object names.
pub(super) fn parents(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return git_text(fixture, repo, &["rev-list", "--parents", "-n", "1", "HEAD"]).split_whitespace().skip(1).map(String::from).collect();
}

/// Writes one file of the repository.
pub(super) fn write(repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("the file");
}
