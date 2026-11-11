//! What:
//!  The resident commit service as a real process:
//! commits through it,
//! a kill at each point of a batch,
//! requests sent
//!  again,
//! a service that starts late or is older than the wrapper,
//! and hostile input.
//! Why:
//!  The service is the only writer of a commit,
//! so every state a kill can leave is checked here against real Git:
//!  each commit lands exactly once,
//! the index agrees with the branch,
//! `fsck` is clean,
//! and the client that was waiting
//!  gets an answer,
//! never a protocol detail (`doc/planning/cli-git-resident-service.md`,
//! "Kill-test plan").
//!  The service is started by the test (the harness),
//! never by the wrapper;
//! its address is an abstract name,
//! so a
//!  killed service leaves no file behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const point of FAULT_POINTS) await killAtPointAndExpectExactlyOnce(point);
//! ```

use super::commit::{git_text, remove_settled};
use super::support::{Fixture, ServiceLaunch, fixture, git, repository};
use git_policy_cli::service_client::{Activation, ClientConfig, Submitted, submit};
use git_policy_cli::service_fault::{FAULT_POINTS, FAULT_VARIABLE, FIRST_BATCH_VARIABLE, PRESTAGE_FLUSHED, PRESTAGE_STAGED, PRESTAGE_START};
use git_policy_cli::service_instance::{instance_of, socket_path};
use git_policy_cli::service_units_of_removed_worktrees::record_worktree;
use git_policy_cli::service_journal::{DONE_EXTENSION, DONE_RETAINED, landed_commits};
use git_policy_cli::service_messages::{Platform, ClientFailure, STARTING_NOTICE, UPDATING_NOTICE};
use git_policy_cli::service_protocol::{
    ALLOWED_MODES, Change, ClientMessage, CommitRequest, PROTOCOL_VERSION, ServiceMessage, decode_service, encode_client, encode_retire, encode_service,
};
use git_policy_cli::service_server::Bind;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Write};
use std::os::fd::FromRawFd;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const COMMITTER: &str = "T <t@example.invalid> 1793535889 +0000";

/// The longest a control waits for anything:
/// far beyond what a healthy run needs,
/// so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// How long the client of a killed service waits on the restarted one before it calls it stopped.
/// The restart settles the batch the kill left open before it accepts a connection (`serve_in_a_landing` calls
///  `settle_open_batches` before `start_acceptor_or_exit`),
/// and that settle waits up to `SETTLE_LOCK_CEILING` for the
///  `index.lock` the killed service left;
/// `LANDING_LOCK_CEILING` bounds the first landing after it.
/// Both are a minute,
/// which is
///  exactly this control's `PATIENCE`,
/// so a settle that used its whole ceiling ended the client's wait (`ServiceSilent` after
///  60 s,
/// 2026-11-08).
/// The event a killed-service control waits for is the answer;
/// a service that never answers is ended by the
///  per-test limit (five minutes),
/// so the client's silence bound is above every ceiling the service can meet.
const RESTART_PATIENCE: Duration = Duration::from_secs(240);

/// The text of a Git query,
/// without its final newline.
fn query(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    return git_text(fixture, repo, arguments).trim_end().to_string();
}

struct Service {
    child: Child,
}

/// A service a control started never outlives it,
/// whether the control ended or a failed assertion unwound it.
impl Drop for Service {
    fn drop(&mut self) {
        let _killed = self.child.kill();
        let _waited = self.child.wait();
    }
}

fn address(fixture: &Fixture, label: &str) -> String {
    return format!("cli-git-service-test-{}-{label}", fixture.root.file_name().and_then(OsStr::to_str).unwrap_or("x"));
}

fn start_service(fixture: &Fixture, name: &str, fault: Option<&str>) -> Service {
    let journal: PathBuf = fixture.root.join("journal");
    let mut command: Command = super::support::service_command();
    command.args(["--journal", journal.to_str().expect("UTF-8"), "--abstract", name]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::inherit());
    if let Some(point) = fault {
        command.env(FAULT_VARIABLE, point);
    }
    return Service { child: command.spawn().expect("the service starts") };
}

fn reachable(name: &str) -> bool {
    return SocketAddr::from_abstract_name(name.as_bytes()).and_then(|socket| { return UnixStream::connect_addr(&socket); }).is_ok();
}

fn wait_until_reachable(name: &str) {
    let started: Instant = Instant::now();
    while !reachable(name) {
        assert!(started.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn stop(mut service: Service) {
    let _killed = service.child.kill();
    let _waited = service.child.wait();
}

/// Waits at most `PATIENCE` for a service started with a fault point to end there by itself.
/// One still running then fails
///  the control with the point named,
/// instead of holding the whole run (a planted control once held one for three hours);
///  dropping the `Service` kills it.
fn ended_at_fault(service: &mut Service, point: &str) -> std::process::ExitStatus {
    let started: Instant = Instant::now();
    loop {
        if let Some(status) = service.child.try_wait().expect("the service's exit state") {
            return status;
        }
        assert!(started.elapsed() < PATIENCE, "{point}: the service did not end at its fault within {PATIENCE:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn config(name: &str) -> ClientConfig {
    return ClientConfig { address: Bind::Abstract(name.to_string()), activation: Activation::Managed, start_wait: PATIENCE, heartbeat_silence: PATIENCE, start_watch: None, keep_fresh: None, answer_ceiling: git_policy_cli::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: git_policy_cli::wait_ceilings::WITHDRAW_CEILING, extra_wait: git_policy_cli::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: git_policy_cli::wait_ceilings::START_CEILING, waiting_notice_after: PATIENCE, units_record_dir: None };
}

/// The client of a control whose service is killed and started again:
/// `config` with `RESTART_PATIENCE` as its silence bound.
fn restarted_service_client(name: &str) -> ClientConfig {
    return ClientConfig { heartbeat_silence: RESTART_PATIENCE, ..config(name) };
}

/// What:
///  The answer the client of a killed-service control ended with,
/// and what it was told while it waited.
/// Why:
///   `submit_quietly` throws the notices away,
/// and the failure this replaces said only `ServiceSilent` after sixty seconds,
///  with nothing about what the client had seen (2026-11-08).
///
/// # Panics
///  When the client ended without an answer,
/// with the failure and every notice it printed.
fn answered_after_the_restart(client: std::thread::JoinHandle<(Result<Submitted, ClientFailure>, String)>, point: &str) -> Submitted {
    let (answered, notices) = client.join().expect("client thread");
    return answered.unwrap_or_else(|failure| panic!("{point}: the client ended before the restarted service answered: {failure:?}; it was told:\n{notices}"));
}

/// A request that sets `path` to `content`,
/// prepared against the branch as it is now.
fn request(fixture: &Fixture, repo: &Path, id: &str, path: &str, content: &str) -> CommitRequest {
    std::fs::write(repo.join(path), content).expect("write");
    let blob: String = query(fixture, repo, &["hash-object", "-w", "--", path]);
    return CommitRequest {
        id: id.to_string(),
        repository: repo.to_path_buf(),
        common_dir: repo.join(".git"),
        git_dir: repo.join(".git"),
        reference: git_policy_cli::ref_name::RefName::from_text("refs/heads/main"),
        base: query(fixture, repo, &["rev-parse", "HEAD"]),
        changes: vec![Change { path: PathBuf::from(path), mode: ALLOWED_MODES[0], blob: Some(blob) }],
        message: format!("message of {id}\n"),
        author: None,
        committer: COMMITTER.to_string(),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: repo.join(".git/index"),
        scratch: repo.join(".git/monochromatic-git-policy-cli-scratch").join(format!("monochromatic-git-policy-cli-service-{id}-0")),
        captured: 0,
        holder: String::new(),
    };
}

fn landed_subjects(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return query(fixture, repo, &["log", "--format=%s", "main"]).lines().map(str::to_string).collect();
}

fn count_of(fixture: &Fixture, repo: &Path, subject: &str) -> usize {
    return landed_subjects(fixture, repo).iter().filter(|line| { return line.as_str() == subject; }).count();
}

fn assert_healthy(fixture: &Fixture, repo: &Path, what: &str) {
    assert_eq!(query(fixture, repo, &["status", "--short", "--untracked-files=no"]), "", "{what}: the index agrees with the branch");
    let checked = Command::new("/usr/bin/git").current_dir(repo).args(["fsck", "--strict", "--no-dangling"]).output().expect("fsck");
    assert!(checked.status.success() && checked.stderr.is_empty(), "{what}: fsck: {}", String::from_utf8_lossy(&checked.stderr));
}

/// What:
///  The references of the wrapper's two side namespaces,
/// as `name object` lines.
/// Why:
///   A kill leaves the batch's scratch reference and the commit it holds behind,
/// and every one of them must be accounted for
///  after the restart rather than being pinned for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['for-each-ref', '--format=%(refname) %(objectname)', namespace])
/// ```
fn side_refs(fixture: &Fixture, repo: &Path, namespace: &str) -> Vec<(String, String)> {
    return query(fixture, repo, &["for-each-ref", "--format=%(refname) %(objectname)", namespace])
        .lines()
        .filter(|line| { return !line.is_empty(); })
        .map(|line| {
            let (name, object) = line.split_once(' ').expect("a reference and its object");
            return (name.to_string(), object.to_string());
        })
        .collect();
}

fn submit_quietly(config: &ClientConfig, request: &CommitRequest) -> Result<Submitted, ClientFailure> {
    return submit(config, request, &mut std::io::sink());
}

/// A commit through the service lands once,
/// is acknowledged with its commit,
/// and leaves the index clean.
#[test]
fn a_commit_through_the_service_lands_and_is_acknowledged() {
    let fixture: Fixture = fixture("service-basic");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "basic");
    let service: Service = start_service(&fixture, name.as_str(), None);
    wait_until_reachable(name.as_str());
    let sent: CommitRequest = request(&fixture, &repo, "basic-1", "a.txt", "a\n");
    let answered: Submitted = submit_quietly(&config(name.as_str()), &sent).expect("answered");
    let Submitted::Landed { commit, already } = answered else { panic!("{answered:?}") };
    assert!(!already);
    assert_eq!(query(&fixture, &repo, &["rev-parse", "main"]), commit);
    assert_eq!(count_of(&fixture, &repo, "message of basic-1"), 1);
    assert_healthy(&fixture, &repo, "basic");
    stop(service);
    remove_settled(&fixture);
}

/// A service that starts on a journal holding more finished batches than it keeps removes the oldest records before it
///  answers its first request,
/// and a commit through it still lands.
#[test]
fn a_service_removes_the_records_of_old_finished_batches_when_it_starts() {
    let fixture: Fixture = fixture("service-prune");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let journal: PathBuf = fixture.root.join("journal");
    std::fs::create_dir_all(&journal).expect("journal directory");
    let surplus: u64 = 5;
    let kept: u64 = u64::try_from(DONE_RETAINED).expect("a count");
    for seq in 1..=(kept + surplus) {
        std::fs::write(journal.join(format!("{seq}.{DONE_EXTENSION}")), "not an intent").expect("an old finished batch");
    }
    let name: String = address(&fixture, "prune");
    let service: Service = start_service(&fixture, name.as_str(), None);
    wait_until_reachable(name.as_str());
    // The first answer comes after the service has started serving, so the pruning at its start has been done.
    let sent: CommitRequest = request(&fixture, &repo, "prune-1", "a.txt", "a\n");
    submit_quietly(&config(name.as_str()), &sent).expect("answered");
    for seq in 1..=surplus {
        assert!(!journal.join(format!("{seq}.{DONE_EXTENSION}")).exists(), "batch {seq} is among the oldest and is removed");
    }
    for seq in (surplus + 1)..=(kept + surplus) {
        assert!(journal.join(format!("{seq}.{DONE_EXTENSION}")).exists(), "batch {seq} is among the newest and is kept");
    }
    assert_eq!(count_of(&fixture, &repo, "message of prune-1"), 1);
    stop(service);
    remove_settled(&fixture);
}

/// The instance of one worktree also trims the journal the single service of the earlier releases left above its own:
/// nothing
///  reads that journal's finished batches,
/// and it grew by a file per commit.
#[test]
fn an_instance_removes_the_old_finished_batches_of_the_earlier_shared_journal() {
    let fixture: Fixture = fixture("service-prune-shared");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let instance: String = instance_of(repo.join(".git").as_path());
    let shared: PathBuf = fixture.root.join("shared");
    let journal: PathBuf = shared.join(instance.as_str());
    std::fs::create_dir_all(&journal).expect("journal directory");
    let surplus: u64 = 5;
    let kept: u64 = u64::try_from(DONE_RETAINED).expect("a count");
    for seq in 1..=(kept + surplus) {
        std::fs::write(shared.join(format!("{seq}.{DONE_EXTENSION}")), "not an intent").expect("an old finished batch");
    }
    let name: String = address(&fixture, "prune-shared");
    let mut command: Command = super::support::service_command();
    command
        .args(["--journal", journal.to_str().expect("UTF-8"), "--abstract", name.as_str(), "--instance", instance.as_str()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    let service: Service = Service { child: command.spawn().expect("the service starts") };
    wait_until_reachable(name.as_str());
    // The first answer comes after the service has started serving, so the pruning at its start has been done.
    let sent: CommitRequest = request(&fixture, &repo, "prune-shared-1", "a.txt", "a\n");
    submit_quietly(&config(name.as_str()), &sent).expect("answered");
    for seq in 1..=surplus {
        assert!(!shared.join(format!("{seq}.{DONE_EXTENSION}")).exists(), "batch {seq} of the shared journal is among the oldest and is removed");
    }
    for seq in (surplus + 1)..=(kept + surplus) {
        assert!(shared.join(format!("{seq}.{DONE_EXTENSION}")).exists(), "batch {seq} of the shared journal is among the newest and is kept");
    }
    stop(service);
    remove_settled(&fixture);
}

/// A service that starts and finds its own worktree gone serves nothing and ends without a failure status:
/// the manager starts a service on the first connection,
/// so a removal Git ran with the wrapper replaced took its units with it at that connection,
/// and a failure status would make `Restart=on-failure` start it again.
#[test]
fn a_service_whose_worktree_is_gone_serves_nothing() {
    let fixture: Fixture = fixture("service-gone-worktree");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let git_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("canonical Git directory");
    let instance: String = instance_of(git_dir.as_path());
    // The runtime directory a user's units live in, and the note the wrapper writes beside them.
    let runtime: PathBuf = fixture.root.join("runtime");
    std::fs::create_dir_all(runtime.join("monochromatic-git-policy-cli")).expect("socket directory");
    record_worktree(runtime.as_path(), instance.as_str(), git_dir.as_path()).expect("the note provisioning writes");
    std::fs::write(socket_path(runtime.as_path(), instance.as_str()), b"listening").expect("the socket file the manager leaves");
    // A worktree deleted without Git, as after a removal run by another Git.
    std::fs::remove_dir_all(git_dir.as_path()).expect("the Git directory is gone");
    let name: String = address(&fixture, "gone-worktree");
    let journal: PathBuf = runtime.join("state/monochromatic-git-policy-cli/service").join(instance.as_str());
    let mut command: Command = super::support::service_command();
    command
        .args(["--journal", journal.to_str().expect("UTF-8"), "--abstract", name.as_str(), "--instance", instance.as_str()])
        .env("XDG_RUNTIME_DIR", runtime.as_path())
        // The bus is looked for in the fixture's runtime directory, so no real user manager is asked anything.
        .env_remove("DBUS_SESSION_BUS_ADDRESS")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    let mut service: Service = Service { child: command.spawn().expect("the service starts") };
    let status: std::process::ExitStatus = ended_at_fault(&mut service, "a service of a gone worktree");
    assert!(status.success(), "serving nothing is not a failure, or `Restart=on-failure` starts the unit again: {status}");
    assert!(!reachable(name.as_str()), "the service listened for a worktree that is gone");
    remove_settled(&fixture);
}

/// Starts the service of one worktree with its journal in `journal`,
/// as the unit does with `--journal` and `--instance`,
///  and with the further words a control gives.
fn start_instance_service(name: &str, journal: &Path, instance: &str, further: &[&str]) -> Service {
    let mut command: Command = super::support::service_command();
    command
        .args(["--journal", journal.to_str().expect("UTF-8"), "--abstract", name, "--instance", instance])
        .args(further)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    return Service { child: command.spawn().expect("the service starts") };
}

/// Leaves a batch of the earlier helper open in `earlier`,
/// as after a kill between landing and finishing:
/// the earlier helper
///  lands a commit and is stopped,
/// and the batch's finished record is turned back into an open one.
/// Gives the open record and
///  the finished record it came from.
fn leave_a_batch_open(fixture: &Fixture, repo: &Path, instance: &str, earlier: &Path, label: &str) -> (PathBuf, PathBuf) {
    std::fs::create_dir_all(earlier).expect("the earlier journal");
    let first_name: String = address(fixture, format!("{label}-first").as_str());
    let first: Service = start_instance_service(first_name.as_str(), earlier, instance, &[]);
    wait_until_reachable(first_name.as_str());
    let sent: CommitRequest = request(fixture, repo, format!("{label}-1").as_str(), "a.txt", "a\n");
    submit_quietly(&config(first_name.as_str()), &sent).expect("answered");
    stop(first);
    let finished: Vec<PathBuf> = std::fs::read_dir(earlier)
        .expect("the earlier journal lists")
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.extension().is_some_and(|extension| { return extension == DONE_EXTENSION; }); })
        .collect();
    assert_eq!(finished.len(), 1, "one landed batch");
    let open: PathBuf = finished[0].with_extension(git_policy_cli::service_journal::OPEN_EXTENSION);
    std::fs::rename(&finished[0], &open).expect("reopen the batch");
    return (open, finished[0].clone());
}

/// The words after the program that make it settle the batches an earlier release left open.
const SETTLE_WORDS: [&str; 3] = ["monochromatic-git-policy-cli", "service", "settle-earlier-batches"];

/// How long a settle run waits before it asks again after a refusal that changed nothing.
const SETTLE_ASK_AGAIN_PAUSE: Duration = Duration::from_millis(20);

/// What:
///  Runs the settle command of the fixture's worktree and gives what the caller saw;
/// while the command refuses and
///  changes nothing,
/// it is asked again after a short pause,
/// for at most `PATIENCE`.
/// Why:
///   The look reads the command line of every process of the user,
/// so it answers from facts the control does not own,
/// and
///  two of its answers are one a person meets by running the command again:
/// a process whose `--journal` folder does not
///  exist yet (a service another control is starting,
/// one whose fixture was just removed,
/// a program another session
///  runs) may be an earlier helper,
/// and a look the machine was too busy to finish inside its ceiling proves
///  nothing either way.
/// Both are a refusal that changes nothing and leaves standard output empty,
/// which is
///  what every hold does;
/// its own words tell the person to run the command again,
/// so this does,
/// and a hold that
///  never clears fails the control with the last answer after `PATIENCE`.
/// Every answer that changed something,
/// or
///  found nothing to settle,
/// is what the control is about,
/// so it is returned at once.
/// Only the first of the two
///  holds was retried before,
/// so a busy host failed these controls on the first slow look while the patience went
///  unused.
/// A PID namespace of its own would keep other processes out of the look,
/// but the gate's container
///  refuses its `/proc` mount (`unshare: mount /proc failed:
///  Permission denied`),
/// so the control follows the product's
///  own instruction instead and this limit stands.
fn run_settle_past_a_refusal(fixture: &Fixture, repo: &Path) -> super::support::Observed {
    let begun: Instant = Instant::now();
    loop {
        let observed = super::support::run_wrapped(fixture, repo, &SETTLE_WORDS);
        // Every hold exits 1 with nothing on standard output and changes nothing on disk; anything else is the control's answer.
        let refused: bool = observed.code == Some(1) && observed.stdout.is_empty();
        if !refused || begun.elapsed() >= PATIENCE {
            return observed;
        }
        std::thread::sleep(SETTLE_ASK_AGAIN_PAUSE);
    }
}

/// The owner's rule is "never discard user data",
/// and the owner's pick of 2026-11-07 ("Off,
/// settle on request"):
/// a batch that
///  the helper of an earlier release accepted and left open is finished when the person runs
///  `git monochromatic-git-policy-cli service settle-earlier-batches`,
/// after the command has said the condition it is safe
///  under;
/// it is finished where it was,
/// remembered as finished,
/// and a second run finds nothing.
#[test]
fn the_settle_command_finishes_a_batch_the_earlier_helper_left_open() {
    let fixture: Fixture = fixture("service-settle-on-request");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let instance: String = instance_of(repo.join(".git").as_path());
    // The wrapper reads the state folder from the home folder the hermetic environment gives it.
    let earlier: PathBuf = fixture.root.join("home/.local/state/cli-git/service").join(instance.as_str());
    let (open, finished) = leave_a_batch_open(&fixture, &repo, instance.as_str(), earlier.as_path(), "settle");
    let settled = run_settle_past_a_refusal(&fixture, &repo);
    let stderr: String = String::from_utf8_lossy(settled.stderr.as_slice()).into_owned();
    assert_eq!(settled.code, Some(0), "{stderr}");
    assert!(stderr.contains("no wrapper of the earlier release is in use in this worktree"), "the condition is said before it acts: {stderr}");
    assert!(String::from_utf8_lossy(settled.stdout.as_slice()).contains("finished batch 1"), "{}", String::from_utf8_lossy(settled.stdout.as_slice()));
    assert!(!open.exists(), "the open batch of the earlier journal was finished");
    assert!(finished.exists(), "and is remembered as finished where it was");
    let again = run_settle_past_a_refusal(&fixture, &repo);
    assert_eq!(again.code, Some(0));
    assert!(String::from_utf8_lossy(again.stdout.as_slice()).contains("No batch of the earlier release is left open"), "{}", String::from_utf8_lossy(again.stdout.as_slice()));
    remove_settled(&fixture);
}

/// A batch the command cannot examine (here its repository is gone) is named with its journal,
/// stays on disk untouched,
/// and
///  makes the exit status nonzero,
/// even though another batch was finished in the same run.
#[test]
fn the_settle_command_names_a_batch_it_could_not_examine_and_exits_nonzero() {
    let fixture: Fixture = fixture("service-settle-unanswered");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let instance: String = instance_of(repo.join(".git").as_path());
    let earlier: PathBuf = fixture.root.join("home/.local/state/cli-git/service").join(instance.as_str());
    let (open, finished) = leave_a_batch_open(&fixture, &repo, instance.as_str(), earlier.as_path(), "unanswered");
    let mut intent = git_policy_cli::service_journal::parse_intent(std::fs::read_to_string(&open).expect("read").as_str()).expect("parse");
    intent.seq += 1;
    intent.repository = fixture.root.join("repository-that-is-not-there");
    git_policy_cli::service_journal::write_intent(earlier.as_path(), &intent).expect("an intent whose repository is gone");
    let unavailable: PathBuf = earlier.join(format!("{}.intent", intent.seq));
    let settled = run_settle_past_a_refusal(&fixture, &repo);
    let stdout: String = String::from_utf8_lossy(settled.stdout.as_slice()).into_owned();
    let stderr: String = String::from_utf8_lossy(settled.stderr.as_slice()).into_owned();
    assert_eq!(settled.code, Some(1), "{stdout}{stderr}");
    assert!(stdout.contains("finished batch 1"), "the batch that could be examined is finished: {stdout} (standard error: {stderr})");
    assert!(stdout.contains(format!("batch {}", intent.seq).as_str()) && stdout.contains("could not be examined"), "the other is named: {stdout}");
    assert!(unavailable.exists(), "its intent stays on disk");
    assert!(!open.exists() && finished.exists());
    remove_settled(&fixture);
}

/// Whether a process of any user has `folder` on its command line,
/// as the `--journal` word a service started for it carries.
fn a_process_names(folder: &Path) -> bool {
    let wanted: &[u8] = folder.as_os_str().as_encoded_bytes();
    let Ok(listing) = std::fs::read_dir("/proc") else { return false };
    return listing.flatten().any(|entry| {
        let Ok(bytes) = std::fs::read(entry.path().join("cmdline")) else { return false };
        return bytes.split(|byte| { return *byte == 0; }).any(|word| { return word == wanted; });
    });
}

/// How long the start of the other control's service is held back,
/// so that the settle command runs while that process already
///  shows its `--journal` word and the service has not made the folder yet.
/// It is the stand-in for the moments in which every
///  service another control starts (and every service whose fixture was just removed) is in that state.
const SERVICE_START_DELAY_SECONDS: &str = "3";

/// Review finding of 2026-11-07 ("passes alone,
/// fails in its suite"):
/// the look that the settle command makes reads the command
///  line of every process of the user,
/// and a process whose `--journal` folder cannot be resolved counts as a service that may be
///  running.
/// The services other controls start show that word for as long as their start takes,
/// so a settle command that ran
///  beside them stopped with nothing on standard output,
/// and every settle control did the same whenever one was starting.
/// The
///  hold is the product's answer and changes nothing;
/// the controls ask again,
/// as its words say,
/// so the batch must be finished
///  once the other service has made its folder,
/// and no control may end on the hold while that is only a start.
#[test]
fn the_settle_command_is_not_held_by_a_service_another_control_is_starting() {
    let fixture: Fixture = fixture("service-settle-beside-a-start");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let instance: String = instance_of(repo.join(".git").as_path());
    let earlier: PathBuf = fixture.root.join("home/.local/state/cli-git/service").join(instance.as_str());
    let (open, finished) = leave_a_batch_open(&fixture, &repo, instance.as_str(), earlier.as_path(), "beside");
    let other: Fixture = super::support::fixture("service-settle-the-other-start");
    let _other_repo: PathBuf = repository(&other, OsStr::new("repo"));
    *other.service_launch.lock().expect("the service launch") = ServiceLaunch {
        prefix: ["/usr/bin/sh", "-c", format!("sleep {SERVICE_START_DELAY_SECONDS}; exec \"$@\"").as_str(), "sh"].map(std::ffi::OsString::from).to_vec(),
        ..ServiceLaunch::default()
    };
    let other_journal: PathBuf = other.root.join("journal");
    std::thread::scope(|scope| {
        let starting = scope.spawn(|| { return super::support::service_runtime(&other); });
        let begun: Instant = Instant::now();
        while !a_process_names(other_journal.as_path()) {
            assert!(begun.elapsed() < PATIENCE, "the other control's service never started");
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(!other_journal.exists(), "the other service has not made its folder yet, so its word cannot be resolved");
        let settled = run_settle_past_a_refusal(&fixture, &repo);
        let stdout: String = String::from_utf8_lossy(settled.stdout.as_slice()).into_owned();
        let stderr: String = String::from_utf8_lossy(settled.stderr.as_slice()).into_owned();
        assert_eq!(settled.code, Some(0), "{stdout}{stderr}");
        assert!(stdout.contains("finished batch 1"), "{stdout} (standard error: {stderr})");
        assert!(!open.exists() && finished.exists());
        starting.join().expect("the other service starts");
    });
    remove_settled(&other);
    remove_settled(&fixture);
}

/// When the safety condition cannot be delivered (standard error is closed),
/// the command settles nothing:
/// the batch is still
///  open afterwards and the exit status is not success.
#[test]
fn the_settle_command_changes_nothing_when_its_condition_cannot_be_written() {
    let fixture: Fixture = fixture("service-settle-closed-stderr");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let instance: String = instance_of(repo.join(".git").as_path());
    let earlier: PathBuf = fixture.root.join("home/.local/state/cli-git/service").join(instance.as_str());
    let (open, finished) = leave_a_batch_open(&fixture, &repo, instance.as_str(), earlier.as_path(), "closed");
    // Standard error is a pipe whose read end was closed before the wrapper starts, so every write it makes meets a broken pipe
    //  (cross-vendor review of 2026-11-07: dropping the read end after `spawn` let the wrapper write and flush the condition first).
    //  The command changes nothing, so the same state is tried again: each run must end without success and leave the batch open.
    for attempt in 0..3 {
        let [read_end, write_end] = super::support::close_on_exec_pipe();
        // SAFETY: both descriptors were just made by `close_on_exec_pipe` and are owned here; each is used once.
        let (read_owned, write_owned) = unsafe { (std::os::fd::OwnedFd::from_raw_fd(read_end), std::os::fd::OwnedFd::from_raw_fd(write_end)) };
        drop(read_owned);
        let mut command: Command = super::support::wrapped(&fixture);
        command
            .current_dir(&repo)
            .args(SETTLE_WORDS)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(write_owned));
        let mut child: Child = command.spawn().expect("the wrapper starts");
        drop(command);
        let status = child.wait().expect("the wrapper ends");
        assert_ne!(status.code(), Some(0), "attempt {attempt}: {status:?}");
        assert!(open.exists(), "attempt {attempt}: the batch is still open");
        assert!(!finished.exists(), "attempt {attempt}: and was not marked finished");
    }
    remove_settled(&fixture);
}

/// Review finding of 2026-11-07 (cross-vendor,
/// rounds 5 and 6):
/// the look at the earlier sockets and processes cannot exclude an
///  earlier-release wrapper that sets its socket unit up and starts the helper between the look and the end of a recovery,
/// and
///  masking the unit would make that wrapper fail its commits.
/// So the service runs no adoption at all:
/// whatever options it is
///  started with,
/// including the one an earlier build of this branch took,
/// a batch of the earlier helper stays exactly where it is.
#[test]
fn the_service_has_no_start_option_that_settles_the_earlier_helpers_batches() {
    let fixture: Fixture = fixture("service-no-adoption-option");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let instance: String = instance_of(repo.join(".git").as_path());
    let state: PathBuf = fixture.root.join("state");
    let earlier: PathBuf = state.join("cli-git/service").join(instance.as_str());
    let current: PathBuf = state.join("monochromatic-git-policy-cli/service").join(instance.as_str());
    let (open, finished) = leave_a_batch_open(&fixture, &repo, instance.as_str(), earlier.as_path(), "held");
    for (index, further) in [vec![], vec!["--adopt-earlier-journals"], vec!["--adopt-earlier-journals", "true"], vec!["--adopt"]].into_iter().enumerate() {
        let name: String = address(&fixture, format!("held-second-{index}").as_str());
        let second: Service = start_instance_service(name.as_str(), current.as_path(), instance.as_str(), further.as_slice());
        wait_until_reachable(name.as_str());
        // The first answer comes after the start, where an adoption would have run.
        let next: CommitRequest = request(&fixture, &repo, format!("held-next-{index}").as_str(), format!("b{index}.txt").as_str(), "b\n");
        submit_quietly(&config(name.as_str()), &next).expect("answered");
        assert!(open.exists(), "{further:?}: the open batch of the earlier journal is left where it is");
        assert!(!finished.exists(), "{further:?}: and is not marked finished by the service of this release");
        stop(second);
    }
    remove_settled(&fixture);
}

/// Many clients at once all land,
/// each once,
/// and the journal never holds more files than commits.
#[test]
fn clients_sent_together_all_land_once() {
    let fixture: Fixture = fixture("service-together");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "together");
    let service: Service = start_service(&fixture, name.as_str(), None);
    wait_until_reachable(name.as_str());
    let clients: usize = 12;
    let requests: Vec<CommitRequest> = (0..clients).map(|n| { return request(&fixture, &repo, format!("t{n}").as_str(), format!("f{n}.txt").as_str(), "x\n"); }).collect();
    let handles: Vec<_> = requests.into_iter().map(|sent| {
        let client: ClientConfig = config(name.as_str());
        return std::thread::spawn(move || { return submit_quietly(&client, &sent); });
    }).collect();
    for handle in handles {
        assert!(matches!(handle.join().expect("client thread"), Ok(Submitted::Landed { already: false, .. })));
    }
    for n in 0..clients {
        assert_eq!(count_of(&fixture, &repo, format!("message of t{n}").as_str()), 1, "client {n}");
    }
    assert_healthy(&fixture, &repo, "together");
    let journal_files: usize = std::fs::read_dir(fixture.root.join("journal")).expect("journal").count();
    assert!(journal_files <= clients, "one journal file per batch, never more than one per commit: {journal_files}");
    stop(service);
    remove_settled(&fixture);
}

/// The service is killed at each point of a batch;
/// it is started again,
/// the client that was waiting reconnects with the
///  same identity,
/// and the commit is on the branch exactly once with the index and the repository healthy.
/// The references the kill leaves are accounted for as well:
/// no batch's scratch reference survives the restart,
/// and the commit of a batch that the branch never took is kept where the person can find it.
#[test]
fn a_service_killed_at_each_point_of_a_batch_lands_the_commit_exactly_once() {
    for point in FAULT_POINTS {
        let fixture: Fixture = fixture(format!("service-kill-{point}").as_str());
        let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
        let name: String = address(&fixture, "kill");
        let mut doomed: Service = start_service(&fixture, name.as_str(), Some(point));
        wait_until_reachable(name.as_str());
        let sent: CommitRequest = request(&fixture, &repo, "kill-1", "k.txt", "k\n");
        let client_config: ClientConfig = restarted_service_client(name.as_str());
        let client = std::thread::spawn(move || {
            let mut notices: Vec<u8> = Vec::new();
            let answered = submit(&client_config, &sent, &mut notices);
            return (answered, String::from_utf8_lossy(&notices).into_owned());
        });
        let status = ended_at_fault(&mut doomed, point);
        assert!(!status.success(), "{point}: the service was killed");
        // A kill at these points leaves the batch's objects on `refs/worktree/cli-git-service/<seq>` without an intent that
        //  names them; the branch takes the commit only from `after-ref` on.
        let before_the_branch_moved: bool = matches!(point, "after-objects" | "after-intent");
        // What:
        //  A point of the marker made ahead is reached by the job that makes it, beside the landing that started it.
        // Why:
        //  So a kill there may come down on either side of the branch's move,
        // and this control's per-point reading of the side refs is
        //  taken only where the point itself fixes that side;
        // what the marker's own points must leave is read below, of the lock and of the files it
        //  writes, and never of the order.
        let at_the_marker: bool = matches!(point, PRESTAGE_START | PRESTAGE_STAGED | PRESTAGE_FLUSHED);
        let scratch_at_the_kill: Vec<(String, String)> = side_refs(&fixture, &repo, "refs/worktree/cli-git-service/");
        if !at_the_marker {
            assert_eq!(scratch_at_the_kill.len(), usize::from(before_the_branch_moved), "{point}: the killed batch's scratch reference is what the kill left: {scratch_at_the_kill:?}");
        }
        let restarted: Service = start_service(&fixture, name.as_str(), None);
        let answered = answered_after_the_restart(client, point);
        assert!(matches!(answered, Submitted::Landed { .. }), "{point}: {answered:?}");
        assert_eq!(count_of(&fixture, &repo, "message of kill-1"), 1, "{point}: exactly once");
        assert_healthy(&fixture, &repo, point);
        // What:
        //  The marker made ahead is never the lock,
        //  so a kill anywhere in its life leaves no lock and nothing that blocks the next commit.
        // Why:
        //  The staging name it is written under is the sweep's to collect and the lock's place is `index.lock`,
        //  which only a rename makes;
        //  a commit after the restart proves the difference where it matters, by landing through the same service and the
        //  same repository.
        assert!(!repo.join(".git/index.lock").exists(), "{point}: no index lock is left");
        let after: CommitRequest = request(&fixture, &repo, "kill-2", "k2.txt", "k2\n");
        let answered_next = submit_quietly(&config(name.as_str()), &after);
        assert!(matches!(answered_next, Ok(Submitted::Landed { .. })), "{point}: the commit after the restart is not blocked by anything the kill left: {answered_next:?}");
        assert_eq!(side_refs(&fixture, &repo, "refs/worktree/cli-git-service/"), Vec::new(), "{point}: the restart removed the scratch reference of the batch no intent named");
        let kept: Vec<(String, String)> = side_refs(&fixture, &repo, "refs/cli-git/kept/");
        if at_the_marker {
            // Nothing of the batch is lost either way: the commit is on the branch exactly once above, and a reference kept for a
            //  batch the branch never took names that batch's own commit and nothing else.
            assert!(kept.len() <= 1, "{point}: at most one kept reference: {kept:?}");
        } else if before_the_branch_moved {
            assert_eq!(kept.len(), 1, "{point}: the commit of the batch that did not land is kept: {kept:?}");
            assert_eq!(query(&fixture, &repo, &["log", "-1", "--format=%s", kept[0].0.as_str()]), "message of kill-1", "{point}: the kept reference holds the batch's own commit");
        } else {
            assert_eq!(kept, Vec::new(), "{point}: the branch took the commit, so nothing was kept");
        }
        stop(restarted);
        remove_settled(&fixture);
    }
}

/// A request sent again after it landed is answered as already landed,
/// with the same commit,
/// and lands nothing more.
#[test]
fn a_request_sent_again_is_answered_as_already_landed() {
    let fixture: Fixture = fixture("service-again");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "again");
    let service: Service = start_service(&fixture, name.as_str(), None);
    wait_until_reachable(name.as_str());
    let sent: CommitRequest = request(&fixture, &repo, "again-1", "r.txt", "r\n");
    let Submitted::Landed { commit, .. } = submit_quietly(&config(name.as_str()), &sent).expect("first") else { panic!("first") };
    let Submitted::Landed { commit: again, already } = submit_quietly(&config(name.as_str()), &sent).expect("second") else { panic!("second") };
    assert!(already);
    assert_eq!(again, commit);
    assert_eq!(count_of(&fixture, &repo, "message of again-1"), 1);
    stop(service);
    // After a restart the journal still remembers it.
    let restarted: Service = start_service(&fixture, name.as_str(), None);
    wait_until_reachable(name.as_str());
    let Submitted::Landed { commit: third, already: third_already } = submit_quietly(&config(name.as_str()), &sent).expect("third") else { panic!("third") };
    assert!(third_already);
    assert_eq!(third, commit);
    assert_eq!(count_of(&fixture, &repo, "message of again-1"), 1);
    stop(restarted);
    remove_settled(&fixture);
}

/// A client killed right after sending leaves the commit landed once and the service serving.
#[test]
fn a_client_that_disappears_after_sending_still_lands_once() {
    let fixture: Fixture = fixture("service-vanish");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "vanish");
    let service: Service = start_service(&fixture, name.as_str(), None);
    wait_until_reachable(name.as_str());
    let sent: CommitRequest = request(&fixture, &repo, "vanish-1", "v.txt", "v\n");
    {
        let socket = SocketAddr::from_abstract_name(name.as_bytes()).expect("name");
        let mut stream: UnixStream = UnixStream::connect_addr(&socket).expect("connect");
        let mut line: String = encode_client(&ClientMessage::Commit(Box::new(sent)));
        line.push('\n');
        stream.write_all(line.as_bytes()).expect("send");
    }
    let started: Instant = Instant::now();
    while count_of(&fixture, &repo, "message of vanish-1") == 0 {
        assert!(started.elapsed() < PATIENCE, "the commit never landed");
        std::thread::sleep(Duration::from_millis(20));
    }
    let next: CommitRequest = request(&fixture, &repo, "vanish-2", "w.txt", "w\n");
    assert!(matches!(submit_quietly(&config(name.as_str()), &next), Ok(Submitted::Landed { .. })));
    assert_eq!(count_of(&fixture, &repo, "message of vanish-1"), 1);
    assert_healthy(&fixture, &repo, "vanish");
    stop(service);
    remove_settled(&fixture);
}

/// Hostile input does not stop the service:
/// a line that is not a message,
/// an oversized line,
/// and a request naming a
///  path outside the repository are all refused,
/// and the next honest client is served.
#[test]
fn hostile_input_is_refused_and_the_service_keeps_serving() {
    let fixture: Fixture = fixture("service-hostile");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "hostile");
    let service: Service = start_service(&fixture, name.as_str(), None);
    wait_until_reachable(name.as_str());
    let hostile_lines: Vec<String> = vec![
        String::from("not json at all"),
        "x".repeat(git_policy_cli::service_protocol::MAX_LINE_BYTES + 10),
        encode_client(&ClientMessage::Commit(Box::new({
            let mut escaping: CommitRequest = request(&fixture, &repo, "evil-1", "e.txt", "e\n");
            escaping.changes[0].path = PathBuf::from("../outside.txt");
            escaping
        }))),
    ];
    for line in hostile_lines {
        let socket = SocketAddr::from_abstract_name(name.as_bytes()).expect("name");
        let stream: UnixStream = UnixStream::connect_addr(&socket).expect("connect");
        let mut writer = stream.try_clone().expect("clone");
        let _sent = writer.write_all(format!("{line}\n").as_bytes());
        let mut reader = BufReader::new(stream);
        let mut welcome = String::new();
        reader.read_line(&mut welcome).expect("welcome");
        let mut answer = String::new();
        let _read = reader.read_line(&mut answer);
        if !answer.is_empty() {
            assert!(matches!(decode_service(answer.trim_end()), Ok(ServiceMessage::Failed { .. })), "{answer}");
        }
    }
    assert!(!repo.parent().expect("parent").join("outside.txt").exists());
    let honest: CommitRequest = request(&fixture, &repo, "honest-1", "h.txt", "h\n");
    assert!(matches!(submit_quietly(&config(name.as_str()), &honest), Ok(Submitted::Landed { .. })));
    assert_healthy(&fixture, &repo, "hostile");
    stop(service);
    remove_settled(&fixture);
}

/// A notice sink the control can read while the client is still writing to it.
struct SharedNotices(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl Write for SharedNotices {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("notices").extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// A service that starts a moment after the client asks is waited for,
/// with one notice;
/// a service that never starts is
///  reported in plain words;
/// where nothing can start it the client says so at once.
#[test]
fn a_late_service_is_waited_for_and_an_absent_one_is_reported_plainly() {
    let fixture: Fixture = fixture("service-late");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "late");
    let sent: CommitRequest = request(&fixture, &repo, "late-1", "l.txt", "l\n");
    let nobody = ClientConfig { start_wait: Duration::from_millis(300), ..config(name.as_str()) };
    assert_eq!(submit_quietly(&nobody, &sent), Err(ClientFailure::DidNotStart));
    // A client that waited for a service it cannot start would sit out this wait, so a result is the proof that it did not.
    let uninstalled = ClientConfig { activation: Activation::Unsupported(Platform::LinuxWithoutUserServices), start_wait: Duration::from_secs(3600), ..config(name.as_str()) };
    assert_eq!(submit_quietly(&uninstalled, &sent), Err(ClientFailure::NoActivation(Platform::LinuxWithoutUserServices)));
    let notices: std::sync::Arc<std::sync::Mutex<Vec<u8>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let started_late = std::thread::spawn({
        let client: ClientConfig = config(name.as_str());
        let for_thread: CommitRequest = sent.clone();
        let mut sink: SharedNotices = SharedNotices(std::sync::Arc::clone(&notices));
        move || { return submit(&client, &for_thread, &mut sink); }
    });
    // The service starts once the client has said that it is waiting for one: the notice is what the client prints after its
    //  grace, so no clock in this control decides whether the client has waited that long.
    let began: Instant = Instant::now();
    while !String::from_utf8_lossy(notices.lock().expect("notices").as_slice()).contains(STARTING_NOTICE) {
        assert!(began.elapsed() < Duration::from_secs(60), "the client never said that it waits for the service");
        std::thread::sleep(Duration::from_millis(10));
    }
    let service: Service = start_service(&fixture, name.as_str(), None);
    let answered = started_late.join().expect("client thread");
    let said: String = String::from_utf8(notices.lock().expect("notices").clone()).expect("UTF-8");
    assert!(matches!(answered, Ok(Submitted::Landed { .. })), "{answered:?}");
    assert_eq!(said.matches(STARTING_NOTICE).count(), 1, "{said:?}");
    stop(service);
    remove_settled(&fixture);
}

/// A service older than the wrapper is asked to retire and the client then reaches the current one,
/// with one notice and
///  no protocol message.
#[test]
fn an_older_service_is_replaced_without_the_user_seeing_it() {
    let fixture: Fixture = fixture("service-skew");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "skew");
    let sent: CommitRequest = request(&fixture, &repo, "skew-1", "s.txt", "s\n");
    // An old service: it greets with an older version and ends when it is asked to retire.
    let socket = SocketAddr::from_abstract_name(name.as_bytes()).expect("name");
    let old: UnixListener = UnixListener::bind_addr(&socket).expect("bind the old service");
    let client_config: ClientConfig = config(name.as_str());
    let client = std::thread::spawn({
        let for_thread: CommitRequest = sent.clone();
        move || {
            let mut notices: Vec<u8> = Vec::new();
            let answered = submit(&client_config, &for_thread, &mut notices);
            return (answered, String::from_utf8(notices).expect("UTF-8"));
        }
    });
    let (stream, _) = old.accept().expect("the client connects to the old service");
    let mut greeting: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION - 1, release: None });
    greeting.push('\n');
    (&stream).write_all(greeting.as_bytes()).expect("greet");
    let mut asked = String::new();
    BufReader::new(&stream).read_line(&mut asked).expect("the client asks it to retire");
    // Written in the version the old service announced, the only one it reads.
    assert_eq!(asked.trim_end(), encode_retire(PROTOCOL_VERSION - 1));
    drop(stream);
    drop(old);
    let service: Service = start_service(&fixture, name.as_str(), None);
    let (answered, notices) = client.join().expect("client thread");
    assert!(matches!(answered, Ok(Submitted::Landed { .. })), "{answered:?}");
    assert!(notices.contains(UPDATING_NOTICE), "{notices:?}");
    stop(service);
    remove_settled(&fixture);
}

/// A service of another release that goes on answering after it was asked to retire ends the command in plain words,
///  within the start wait and not after it,
/// with the command that restarts the service;
/// nothing is sent to it but the request
///  to retire.
#[test]
fn a_service_of_another_release_that_will_not_retire_is_reported_plainly() {
    let fixture: Fixture = fixture("service-stubborn");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "stubborn");
    let sent: CommitRequest = request(&fixture, &repo, "stubborn-1", "t.txt", "t\n");
    let socket = SocketAddr::from_abstract_name(name.as_bytes()).expect("name");
    let stubborn: UnixListener = UnixListener::bind_addr(&socket).expect("bind");
    stubborn.set_nonblocking(true).expect("non-blocking");
    let finished: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let acceptor = std::thread::spawn({
        let stop: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&finished);
        move || {
            let mut asked: Vec<String> = Vec::new();
            let mut held: Vec<UnixStream> = Vec::new();
            while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                let Ok((stream, _)) = stubborn.accept() else {
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                };
                stream.set_nonblocking(false).expect("blocking");
                let mut greeting: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from("0.0.1-earlier")) });
                greeting.push('\n');
                let _sent = (&stream).write_all(greeting.as_bytes());
                let mut line: String = String::new();
                let _read = BufReader::new(&stream).read_line(&mut line);
                asked.push(line.trim_end().to_string());
                held.push(stream);
            }
            return asked;
        }
    });
    let impatient: ClientConfig = ClientConfig { start_wait: Duration::from_millis(400), ..config(name.as_str()) };
    assert_eq!(submit_quietly(&impatient, &sent), Err(ClientFailure::OtherRelease));
    finished.store(true, std::sync::atomic::Ordering::SeqCst);
    let asked: Vec<String> = acceptor.join().expect("acceptor");
    // The connection the client gave up on says nothing; every other line is the request to retire.
    let spoken: Vec<&String> = asked.iter().filter(|line| { return !line.is_empty(); }).collect();
    assert!(!spoken.is_empty() && spoken.iter().all(|line| { return **line == encode_client(&ClientMessage::Retire); }), "{asked:?}");
    git(&fixture, &repo, &["status", "--short"]);
    remove_settled(&fixture);
}

/// The protocol version of the build before a changed path carried its bytes.
/// The release text cannot name that build apart from this one,
/// because both name one crate version.
const PROTOCOL_BEFORE_CHANGED_PATH_BYTES: u32 = 7;

/// What:
///  A service of the protocol that ignores `pathHex`,
/// even one that names this release,
/// is asked to retire and is never
///  given a commit.
/// Why:
///   Such a service takes the readable spelling of a name that is not text and commits it,
/// and reports success,
/// which is another file from the one the person staged;
/// the version both sides compare is the only thing that keeps the request from being handed to it.
#[test]
fn a_service_that_ignores_changed_path_bytes_is_retired_and_never_given_a_commit() {
    let fixture: Fixture = fixture("service-same-release-old-protocol");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "same-release");
    let sent: CommitRequest = request(&fixture, &repo, "same-release-1", "c.txt", "c\n");
    let socket = SocketAddr::from_abstract_name(name.as_bytes()).expect("name");
    let stubborn: UnixListener = UnixListener::bind_addr(&socket).expect("bind");
    stubborn.set_nonblocking(true).expect("non-blocking");
    let finished: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let acceptor = std::thread::spawn({
        let stop: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&finished);
        move || {
            let mut asked: Vec<String> = Vec::new();
            let mut held: Vec<UnixStream> = Vec::new();
            while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                let Ok((stream, _)) = stubborn.accept() else {
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                };
                stream.set_nonblocking(false).expect("blocking");
                let mut greeting: String = encode_service(&ServiceMessage::Welcome {
                    version: PROTOCOL_BEFORE_CHANGED_PATH_BYTES,
                    release: Some(String::from(git_policy_cli::service_protocol::RELEASE)),
                });
                greeting.push('\n');
                let _sent = (&stream).write_all(greeting.as_bytes());
                let mut line: String = String::new();
                let _read = BufReader::new(&stream).read_line(&mut line);
                // A service that ignored the bytes would answer the commit: that answer is the success this control denies.
                if line.contains("\"type\":\"commit\"") {
                    let mut answer: String = encode_service(&ServiceMessage::Landed {
                        id: String::from("same-release-1"),
                        commit: String::from("0123456789abcdef0123456789abcdef01234567"),
                        already: false,
                        landing: None,
                    });
                    answer.push('\n');
                    let _answered = (&stream).write_all(answer.as_bytes());
                }
                asked.push(line.trim_end().to_string());
                held.push(stream);
            }
            return asked;
        }
    });
    let impatient: ClientConfig = ClientConfig { start_wait: Duration::from_millis(400), ..config(name.as_str()) };
    let answered = submit_quietly(&impatient, &sent);
    finished.store(true, std::sync::atomic::Ordering::SeqCst);
    let asked: Vec<String> = acceptor.join().expect("acceptor");
    let spoken: Vec<&String> = asked.iter().filter(|line| { return !line.is_empty(); }).collect();
    assert_eq!(answered, Err(ClientFailure::OtherRelease), "a service that ignores the bytes is retired, not served");
    assert!(!spoken.is_empty(), "the client asked it to retire: {asked:?}");
    assert!(spoken.iter().all(|line| { return line.contains("\"type\":\"retire\""); }), "no commit line is handed to it: {asked:?}");
    git(&fixture, &repo, &["status", "--short"]);
    remove_settled(&fixture);
}

/// A service newer than the wrapper cannot be talked to,
/// and the user is told what to do,
/// in plain words.
#[test]
fn a_newer_service_leaves_the_user_a_plain_instruction() {
    let fixture: Fixture = fixture("service-newer");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "newer");
    let sent: CommitRequest = request(&fixture, &repo, "newer-1", "n.txt", "n\n");
    let socket = SocketAddr::from_abstract_name(name.as_bytes()).expect("name");
    let future: UnixListener = UnixListener::bind_addr(&socket).expect("bind");
    let acceptor = std::thread::spawn(move || {
        if let Ok((stream, _)) = future.accept() {
            let mut greeting: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION + 1, release: Some(String::from(git_policy_cli::service_protocol::RELEASE)) });
            greeting.push('\n');
            let _sent = (&stream).write_all(greeting.as_bytes());
            std::thread::sleep(Duration::from_millis(200));
        }
    });
    assert_eq!(submit_quietly(&config(name.as_str()), &sent), Err(ClientFailure::NoCommonVersion));
    acceptor.join().expect("acceptor");
    git(&fixture, &repo, &["status", "--short"]);
    remove_settled(&fixture);
}

/// The service is killed at an arbitrary moment while many clients are in flight;
/// it is started again and every client
///  is answered,
/// each commit on the branch exactly once,
/// with a clean index and a clean `fsck`.
#[test]
fn a_service_killed_at_arbitrary_moments_under_load_loses_and_duplicates_nothing() {
    let delays_ms: [u64; 4] = [1, 5, 12, 25];
    for (round, delay) in delays_ms.into_iter().enumerate() {
        let fixture: Fixture = fixture(format!("service-random-kill-{round}").as_str());
        let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
        let name: String = address(&fixture, "random");
        let mut doomed: Service = start_service(&fixture, name.as_str(), None);
        wait_until_reachable(name.as_str());
        let clients: usize = 16;
        let requests: Vec<CommitRequest> = (0..clients).map(|n| { return request(&fixture, &repo, format!("r{n}").as_str(), format!("f{n}.txt").as_str(), "x\n"); }).collect();
        let handles: Vec<_> = requests.into_iter().map(|sent| {
            let client: ClientConfig = config(name.as_str());
            return std::thread::spawn(move || { return submit_quietly(&client, &sent); });
        }).collect();
        std::thread::sleep(Duration::from_millis(delay));
        let _killed = doomed.child.kill();
        let _waited = doomed.child.wait();
        let restarted: Service = start_service(&fixture, name.as_str(), None);
        for (n, handle) in handles.into_iter().enumerate() {
            let answered = handle.join().expect("client thread").expect("answered after the restart");
            assert!(matches!(answered, Submitted::Landed { .. }), "client {n} after a kill at {delay} ms: {answered:?}");
        }
        for n in 0..clients {
            assert_eq!(count_of(&fixture, &repo, format!("message of r{n}").as_str()), 1, "client {n}, kill at {delay} ms");
        }
        assert_healthy(&fixture, &repo, "random kill");
        stop(restarted);
        remove_settled(&fixture);
    }
}

/// An amend request for `path` set to `content`:
/// the tree is the tip's tree with that file,
/// written to the object store,
///  and the real index is left as it was.
fn amend_request(fixture: &Fixture, repo: &Path, id: &str, path: &str, content: &str) -> CommitRequest {
    let mut amend: CommitRequest = request(fixture, repo, id, path, content);
    git(fixture, repo, &["add", "--", path]);
    let tree: String = query(fixture, repo, &["write-tree"]);
    git(fixture, repo, &["reset", "--quiet"]);
    amend.amend = true;
    amend.tree = Some(tree);
    return amend;
}

/// Another program holds Git's index lock:
/// the landing waits for it before the branch moves,
/// so the index never shows a file as
///  reversed.
/// Once the other program lets go,
/// the commit lands,
/// the index agrees with the branch,
/// and no intent is left open.
#[test]
fn a_landing_waits_for_the_index_lock_before_the_branch_moves() {
    let fixture: Fixture = fixture("service-index-wait");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "wait");
    let service: Service = start_service(&fixture, name.as_str(), None);
    wait_until_reachable(name.as_str());
    let first: CommitRequest = request(&fixture, &repo, "wait-1", "a.txt", "a\n");
    let lock: PathBuf = repo.join(".git/index.lock");
    // A lock with content that nothing the service can see holds, as a person's `git commit --all` in its editor leaves one;
    //  an empty lock that nothing holds is removed by the service (`service_index_unheld_removal.rs`).
    std::fs::write(&lock, b"another program's lock\n").expect("another program holds the index");
    let before: String = query(&fixture, &repo, &["rev-parse", "HEAD"]);
    let holder_lock: PathBuf = lock.clone();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(800));
        std::fs::remove_file(holder_lock).expect("the other program lets go");
    });
    let began: Instant = Instant::now();
    let landed = submit_quietly(&config(name.as_str()), &first).expect("answered");
    assert!(began.elapsed() >= Duration::from_millis(700), "the commit waited for the holder: {:?}", began.elapsed());
    assert!(matches!(landed, Submitted::Landed { already: false, .. }), "{landed:?}");
    release.join().expect("thread");
    assert_ne!(query(&fixture, &repo, &["rev-parse", "HEAD"]), before);
    let open: usize = std::fs::read_dir(fixture.root.join("journal")).map_or(0, |entries| {
        return entries.flatten().filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == git_policy_cli::service_journal::OPEN_EXTENSION; }); }).count();
    });
    assert_eq!(open, 0, "no intent is left open");
    assert_healthy(&fixture, &repo, "landed after the wait");
    stop(service);
    remove_settled(&fixture);
}

/// An amend through the real service replaces the tip,
/// and an amend prepared on a tip the branch has left is refused with
///  the plain failure and changes nothing.
#[test]
fn an_amend_lands_through_the_service_and_a_stale_one_is_refused() {
    let fixture: Fixture = fixture("service-amend-process");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "amend");
    let service: Service = start_service(&fixture, name.as_str(), None);
    wait_until_reachable(name.as_str());
    let first: CommitRequest = request(&fixture, &repo, "amend-first", "first.txt", "first\n");
    assert!(matches!(submit_quietly(&config(name.as_str()), &first), Ok(Submitted::Landed { .. })));
    let parent: String = query(&fixture, &repo, &["rev-parse", "HEAD^"]);
    let stale: CommitRequest = amend_request(&fixture, &repo, "amend-stale", "stale.txt", "stale\n");
    let live: CommitRequest = amend_request(&fixture, &repo, "amend-live", "live.txt", "live\n");
    let Submitted::Landed { commit, .. } = submit_quietly(&config(name.as_str()), &live).expect("answered") else { panic!("not landed") };
    assert_eq!(query(&fixture, &repo, &["rev-parse", "HEAD"]), commit);
    assert_eq!(query(&fixture, &repo, &["rev-parse", "HEAD^"]), parent, "the replaced commit's parent");
    assert_eq!(count_of(&fixture, &repo, "message of amend-first"), 0, "the first commit was replaced");
    let refused = submit_quietly(&config(name.as_str()), &stale);
    assert_eq!(refused, Ok(Submitted::Moved));
    assert_eq!(query(&fixture, &repo, &["rev-parse", "HEAD"]), commit, "nothing changed");
    assert_healthy(&fixture, &repo, "amend");
    stop(service);
    remove_settled(&fixture);
}

/// An amend sent in the middle of plain commits to one branch is answered on its own,
/// and no plain commit is refused
///  for sharing a batch with it:
/// the plain commits all land,
/// and the amend,
/// prepared on a tip that has since moved,
/// is
///  told the branch moved.
#[test]
fn an_amend_among_plain_commits_neither_refuses_them_nor_lands_stale() {
    let fixture: Fixture = fixture("service-amend-among");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let name: String = address(&fixture, "among");
    let service: Service = start_service(&fixture, name.as_str(), None);
    wait_until_reachable(name.as_str());
    let plain: usize = 12;
    let mut requests: Vec<CommitRequest> = (0..plain).map(|n| { return request(&fixture, &repo, format!("p{n}").as_str(), format!("f{n}.txt").as_str(), "x\n"); }).collect();
    let amend: CommitRequest = amend_request(&fixture, &repo, "among-amend", "amended.txt", "amended\n");
    requests.insert(plain / 2, amend);
    let mut handles: Vec<std::thread::JoinHandle<Result<Submitted, ClientFailure>>> = Vec::new();
    let mut first_answer: Option<Result<Submitted, ClientFailure>> = None;
    for (position, sent) in requests.into_iter().enumerate() {
        let client: ClientConfig = config(name.as_str());
        let handle = std::thread::spawn(move || { return submit_quietly(&client, &sent); });
        if position == 0 {
            // The first lands before the rest are sent, so the branch has moved when the amend arrives, on any host: the
            //  amend was prepared on the tip before it, and the rest reach the service together behind a landing that is over.
            first_answer = Some(handle.join().expect("client thread"));
        } else {
            handles.push(handle);
        }
    }
    let mut answers: Vec<Result<Submitted, ClientFailure>> = vec![first_answer.expect("the first answer")];
    answers.extend(handles.into_iter().map(|handle| { return handle.join().expect("client thread"); }));
    for (position, answer) in answers.iter().enumerate() {
        if position == plain / 2 {
            assert_eq!(answer, &Ok(Submitted::Moved), "the amend");
        } else {
            assert!(matches!(answer, Ok(Submitted::Landed { already: false, .. })), "plain commit {position}: {answer:?}");
        }
    }
    assert_eq!(count_of(&fixture, &repo, "message of among-amend"), 0);
    assert_healthy(&fixture, &repo, "amend among plain commits");
    stop(service);
    remove_settled(&fixture);
}

/// The service is killed at each point of an amend's landing;
/// after the restart the client's amend is on the branch exactly
///  once,
/// with the parent of the replaced commit and a healthy repository.
#[test]
fn a_service_killed_at_each_point_of_an_amend_lands_it_exactly_once() {
    for point in FAULT_POINTS {
        let fixture: Fixture = fixture(format!("service-amend-kill-{point}").as_str());
        let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
        let name: String = address(&fixture, "amendkill");
        let calm: Service = start_service(&fixture, name.as_str(), None);
        wait_until_reachable(name.as_str());
        let first: CommitRequest = request(&fixture, &repo, "ak-first", "first.txt", "first\n");
        assert!(matches!(submit_quietly(&config(name.as_str()), &first), Ok(Submitted::Landed { .. })));
        stop(calm);
        let parent: String = query(&fixture, &repo, &["rev-parse", "HEAD^"]);
        let sent: CommitRequest = amend_request(&fixture, &repo, "ak-amend", "amended.txt", "amended\n");
        let mut doomed: Service = start_service(&fixture, name.as_str(), Some(point));
        wait_until_reachable(name.as_str());
        let client_config: ClientConfig = restarted_service_client(name.as_str());
        let client = std::thread::spawn(move || {
            let mut notices: Vec<u8> = Vec::new();
            let answered = submit(&client_config, &sent, &mut notices);
            return (answered, String::from_utf8_lossy(&notices).into_owned());
        });
        let status = ended_at_fault(&mut doomed, point);
        assert!(!status.success(), "{point}: the service was killed");
        let restarted: Service = start_service(&fixture, name.as_str(), None);
        let answered = answered_after_the_restart(client, point);
        assert!(matches!(answered, Submitted::Landed { .. }), "{point}: {answered:?}");
        assert_eq!(count_of(&fixture, &repo, "message of ak-amend"), 1, "{point}: exactly once");
        assert_eq!(count_of(&fixture, &repo, "message of ak-first"), 0, "{point}: the replaced commit is off the branch");
        assert_eq!(query(&fixture, &repo, &["rev-parse", "HEAD^"]), parent, "{point}: the replaced commit's parent");
        assert_healthy(&fixture, &repo, point);
        stop(restarted);
        remove_settled(&fixture);
    }
}

/// A service that starts on a clock reading another service of the journal also took,
/// and a service whose count another service
///  has overtaken,
/// must not number a batch with one the journal already holds:
/// each landing keeps its own record,
/// so a request
///  sent again after a broken connection is still found landed.
#[test]
fn two_services_of_one_journal_never_number_their_batches_alike() {
    let fixture: Fixture = fixture("service-shared-numbers");
    let first_repo: PathBuf = repository(&fixture, OsStr::new("first"));
    let second_repo: PathBuf = repository(&fixture, OsStr::new("second"));
    let pinned: String = String::from("1000");
    let mut services: Vec<Service> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for label in ["numbers-a", "numbers-b"] {
        let name: String = address(&fixture, label);
        let journal: PathBuf = fixture.root.join("journal");
        let mut command: Command = super::support::service_command();
        command
            .args(["--journal", journal.to_str().expect("UTF-8"), "--abstract", name.as_str()])
            .env(FIRST_BATCH_VARIABLE, pinned.as_str())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        services.push(Service { child: command.spawn().expect("the service starts") });
        names.push(name);
    }
    for name in &names {
        wait_until_reachable(name.as_str());
    }
    let first: CommitRequest = request(&fixture, &first_repo, "numbers-1", "a.txt", "a\n");
    let second: CommitRequest = request(&fixture, &second_repo, "numbers-2", "a.txt", "a\n");
    assert!(matches!(submit_quietly(&config(names[0].as_str()), &first), Ok(Submitted::Landed { already: false, .. })));
    assert!(matches!(submit_quietly(&config(names[1].as_str()), &second), Ok(Submitted::Landed { already: false, .. })));
    let finished: usize = std::fs::read_dir(fixture.root.join("journal"))
        .expect("journal")
        .flatten()
        .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == DONE_EXTENSION; }); })
        .count();
    assert_eq!(finished, 2, "each landing leaves its own finished batch");
    let known: std::collections::HashMap<String, String> = landed_commits(fixture.root.join("journal").as_path());
    assert!(known.contains_key("numbers-1") && known.contains_key("numbers-2"), "both requests are remembered as landed: {known:?}");
    for service in services {
        stop(service);
    }
    remove_settled(&fixture);
}
