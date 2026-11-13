//! What:
//!  The real commit service,
//! a foreign program that holds the repository's index,
//! and clients whose waits reach their bound:
//!  what lands,
//! what never lands,
//! and what a restart of the service does.
//! Why:
//!  The owner,
//! 2026-11-03:
//! "nothing should ever wait forever or indefinitely",
//! and for a wait that reaches its bound,
//!  "Return with an error" with "No,
//! keep it for a retry (Recommended)":
//! the commit is withdrawn so that nothing lands later by
//!  itself,
//! the staged changes stay staged,
//! and the commit is kept on a side ref.
//! A withdrawal and a landing that start at the
//!  same moment must agree:
//! either the commit landed,
//! and the client reports success,
//! or it never will.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await submit({ ...config, answerCeiling: 1000 }, request); // 'landed' | 'withdrawn'
//! await restartService(); expect(await log()).not.toContain(request.message);
//! ```

use super::commit::{git_text, remove_settled};
use super::service_commit::runtime_for;
use super::service_landing_lock::{Held, foreign_commit_until_released, service_with_arguments};
use super::service_landing_selection::base_repository;
use super::support::{Fixture, fixture};
use git_policy_cli::service_client::{Activation, ClientConfig, Submitted, submit};
use git_policy_cli::service_messages::ClientFailure;
use git_policy_cli::service_protocol::{ALLOWED_MODES, Change, CommitRequest};
use git_policy_cli::service_server::Bind;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::Duration;

/// The longest a control waits for anything:
/// far beyond what a healthy run needs,
/// so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// The ceiling a client of the controls gives the service's answer:
/// short,
/// so a control ends in seconds.
const CLIENT_CEILING: Duration = Duration::from_secs(1);

/// The ceiling the client of the control whose answer is the service's own refusal gives the service:
/// five minutes,
/// five times the longest wait a landing has been measured to take outside the acquisition's own work.
/// Why such a ceiling,
/// where every other control of these rules answers in seconds:
///  the control proves that the service's own ceiling (3 s) withdraws the commit and names the holder,
/// and it read that answer as the client's `IndexInUse`.
/// A landing the machine stalled after its ceiling keeps the client waiting for an answer it is about to give:
/// the gate of 2026-11-11T10:29-05:00 answered that landing 95.8 s after the request,
/// of which 92.3 s was outside the wait the acquisition accounts for,
/// so a client bound of sixty seconds ended in `StillWithService` before the refusal could reach it.
/// A person's own client waits `ANSWER_CEILING` (twenty minutes) for this answer,
/// so the control's bound must be far past what the service's own 3 s ceiling can need,
/// and this one is.
const SERVICE_ANSWER_PATIENCE: Duration = Duration::from_secs(300);

/// The ceiling the controls give the service's wait for a foreign lock when a control needs the service to give up first.
const SERVICE_CEILING_MILLIS: &str = "3000";

/// The notice interval of the service in these controls:
/// far beyond them,
/// so only the first notice of a wait is heard.
const QUIET_NOTICE_SECONDS: &str = "3600";

/// The notice interval of the service in the control where the withdrawal must cross the client's ceiling:
/// one second,
/// so the wait for the lock repeats its line every second and one of those lines reaches the client after its ceiling.
/// Why not the service's own beat:
///  a beat is written by the connection's writer at its own timer,
/// and a writer the suite's load held back writes a beat only when the answer is already queued,
///  where `tokio::time::timeout` takes the ready message and no beat is written at all
/// (gate of 2026-11-11T10:29-05:00,
/// `a_landing_that_starts_as_the_bound_passes_is_reported_as_landed`:
/// `Ok(Landed { already: false })`
/// after 3.17 s,
/// where the landing's own answer crossed the client's one-second ceiling with no line before it).
/// A notice is *queued* by the landing,
/// so it is ahead of the answer on the connection whatever the writer's timing.
const RACE_NOTICE_SECONDS: &str = "1";

/// The ceiling the client of that control gives the service's answer:
/// half a second,
/// shorter than one notice interval,
/// so a notice is certainly due after it and never before it by more than half an interval.
const RACE_CLIENT_CEILING: Duration = Duration::from_millis(500);

/// How long the foreign program holds the lock past that ceiling:
/// five seconds,
/// five notice intervals,
/// so a notice is queued after the ceiling even when the landing took seconds to start,
/// and the landing can only answer after the program lets go,
/// which is past every one of those notices.
const HOLD_PAST_THE_CEILING: Duration = Duration::from_secs(5);

/// A request that sets `path` to `content`,
/// prepared against the branch as it is now.
fn request_for(fixture: &Fixture, repo: &Path, id: &str, path: &str, content: &str) -> CommitRequest {
    std::fs::write(repo.join(path), content).expect("write");
    let blob: String = git_text(fixture, repo, &["hash-object", "-w", "--", path]).trim().to_string();
    let git_dir: PathBuf = repo.join(".git");
    return CommitRequest {
        id: id.to_string(),
        repository: repo.to_path_buf(),
        common_dir: git_dir.clone(),
        git_dir: git_dir.clone(),
        reference: git_policy_cli::ref_name::RefName::from_text("refs/heads/main"),
        base: git_text(fixture, repo, &["rev-parse", "HEAD"]).trim().to_string(),
        changes: vec![Change { path: PathBuf::from(path), mode: ALLOWED_MODES[0], blob: Some(blob) }],
        message: format!("message of {id}\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: git_dir.join("index"),
        scratch: git_dir.join("monochromatic-git-policy-cli-scratch").join(format!("monochromatic-git-policy-cli-service-{id}-0")),
        shadow: None,
        captured: 0,
        holder: String::new(),
    };
}

/// The client configuration for the service of `name`,
/// with the given ceiling for the answer.
fn config_for(fixture: &Fixture, name: &str, answer_ceiling: Duration) -> ClientConfig {
    let git_dir: PathBuf = fixture.root.join("repo/.git");
    let socket: PathBuf = git_policy_cli::service_instance::socket_for(runtime_for(name).as_path(), git_dir.as_path());
    return ClientConfig {
        address: Bind::Path(socket),
        activation: Activation::Managed,
        start_wait: PATIENCE,
        heartbeat_silence: PATIENCE,
        start_watch: None,
        keep_fresh: None,
        answer_ceiling,
        withdraw_ceiling: PATIENCE,
        waiting_notice_after: PATIENCE,
        extra_wait: git_policy_cli::wait_ceilings::SHORT_EXTRA_WAIT,
        start_ceiling: PATIENCE,
        units_record_dir: None,
    };
}

/// A client that submits `request` on a thread of its own and returns what it ended with and printed.
fn submit_on_a_thread(config: ClientConfig, request: CommitRequest) -> std::thread::JoinHandle<(Result<Submitted, ClientFailure>, String)> {
    return std::thread::spawn(move || {
        let mut notices: Vec<u8> = Vec::new();
        let answered = submit(&config, &request, &mut notices);
        return (answered, String::from_utf8(notices).expect("UTF-8"));
    });
}

/// The subjects on the branch after `base`,
/// oldest first.
fn subjects_since(fixture: &Fixture, repo: &Path, base: &str) -> String {
    return git_text(fixture, repo, &["log", "--reverse", "--format=%s", format!("{base}..HEAD").as_str()]);
}

/// How many batches the service's journal still holds open:
/// an open batch is one a restart would finish.
fn open_batches(fixture: &Fixture) -> usize {
    let journal: PathBuf = fixture.root.join("journal");
    return std::fs::read_dir(journal).map_or(0, |entries| {
        return entries.flatten().filter(|entry| { return entry.file_name().to_string_lossy().ends_with(".open"); }).count();
    });
}

/// A request that is queued behind a landing that waits for a foreign lock,
/// and whose client reaches its bound meanwhile,
/// is
/// withdrawn:
/// when the foreign program lets go,
/// the landing in progress ends,
/// the withdrawal is answered before the queued request is
/// looked at,
/// the request never lands,
/// and a restart of the service lands nothing either.
#[test]
fn a_request_withdrawn_at_its_bound_never_lands_not_even_after_a_restart() {
    let fixture: Fixture = fixture("withdraw-never-lands");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("wd-never-{}", std::process::id());
    let service: Held = service_with_arguments(&fixture, name.as_str(), QUIET_NOTICE_SECONDS, &[]);
    let base: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let (holder, release): (Child, PathBuf) = foreign_commit_until_released(&fixture, &repo, 0);
    // The first request waits for the foreign lock with the service's long ceiling; it has no client ceiling of note.
    let first = submit_on_a_thread(config_for(&fixture, name.as_str(), PATIENCE), request_for(&fixture, &repo, "first", "a.txt", "first\n"));
    std::thread::sleep(CLIENT_CEILING);
    // The second is queued behind the first landing, and its client reaches its bound while it waits.
    let second = submit_on_a_thread(config_for(&fixture, name.as_str(), CLIENT_CEILING), request_for(&fixture, &repo, "second", "c.txt", "second\n"));
    std::thread::sleep(CLIENT_CEILING * 3);
    std::fs::write(&release, b"").expect("the foreign commit is told to let go");
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "{}", String::from_utf8_lossy(&held.stderr));
    let (first_answer, first_notices) = first.join().expect("first client");
    assert!(matches!(first_answer, Ok(Submitted::Landed { .. })), "{first_answer:?} {first_notices}");
    let (second_answer, _second_notices) = second.join().expect("second client");
    assert!(matches!(second_answer, Err(ClientFailure::StillWithService { .. })), "{second_answer:?}");
    // The service gets time to land anything it is going to land before the branch is read.
    std::thread::sleep(CLIENT_CEILING * 3);
    let landed: String = subjects_since(&fixture, &repo, base.as_str());
    assert_eq!(landed, "foreign\nmessage of first\n", "the withdrawn request is not on the branch");
    // A restart: the old service is killed and a new one settles the journal and serves.
    drop(service);
    let _restarted: Held = service_with_arguments(&fixture, name.as_str(), QUIET_NOTICE_SECONDS, &[]);
    std::thread::sleep(CLIENT_CEILING * 2);
    assert_eq!(subjects_since(&fixture, &repo, base.as_str()), landed, "a restart of the service landed nothing");
    assert_eq!(open_batches(&fixture), 0, "no batch is left open for a restart to finish");
    remove_settled(&fixture);
}

/// The race:
/// the withdrawal and a landing of the same request meet,
/// and the answer is that the commit landed:
/// the command reports success and
/// the commit is on the branch exactly once.
/// The bound passes while the foreign program still holds the lock,
/// and a repeated notice of the wait reaches the client after that bound and before the answer,
/// so the withdrawal is decided while the landing is in progress:
/// nothing here depends on when the connection's writer gets a turn.
#[test]
fn a_landing_that_starts_as_the_bound_passes_is_reported_as_landed() {
    let fixture: Fixture = fixture("withdraw-race-landed");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("wd-race-{}", std::process::id());
    let _service: Held = service_with_arguments(&fixture, name.as_str(), RACE_NOTICE_SECONDS, &[]);
    let base: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let (holder, release): (Child, PathBuf) = foreign_commit_until_released(&fixture, &repo, 0);
    let mut client: ClientConfig = config_for(&fixture, name.as_str(), RACE_CLIENT_CEILING);
    // The wait a client gives a write it is inside is the owner's short one (`SHORT_EXTRA_WAIT`, seven seconds), which a landing
    //  the machine stalled spends before it answers; this control is about the withdrawal meeting a landing, not about that wait
    //  (which `service_heartbeat` proves), so its client waits like a person's.
    client.extra_wait = PATIENCE;
    let waiting = submit_on_a_thread(client, request_for(&fixture, &repo, "racer", "a.txt", "racer\n"));
    // The foreign program holds the lock past the bound, so the withdrawal is decided while the landing waits for it.
    std::thread::sleep(RACE_CLIENT_CEILING + HOLD_PAST_THE_CEILING);
    std::fs::write(&release, b"").expect("the foreign commit is told to let go");
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "{}", String::from_utf8_lossy(&held.stderr));
    let (answered, notices) = waiting.join().expect("client");
    assert!(matches!(answered, Ok(Submitted::Landed { already: true, .. })), "{answered:?} {notices}");
    assert_eq!(subjects_since(&fixture, &repo, base.as_str()), "foreign\nmessage of racer\n", "the commit landed once");
    remove_settled(&fixture);
}

/// The service's own wait for a foreign lock reaches its ceiling before the client's:
/// the landing is withdrawn before anything
/// moved,
/// the built commit is kept on a side ref that the message names,
/// the person's staged changes stand,
/// and the client ends
/// with the error that names the foreign program and says what to run;
/// a restart lands nothing.
#[test]
fn a_foreign_lock_that_outlasts_the_service_ceiling_withdraws_the_commit_and_keeps_it() {
    let fixture: Fixture = fixture("withdraw-service-ceiling");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("wd-lock-{}", std::process::id());
    let service: Held = service_with_arguments(&fixture, name.as_str(), QUIET_NOTICE_SECONDS, &["--lock-ceiling-millis", SERVICE_CEILING_MILLIS]);
    let base: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_string();
    let (holder, release): (Child, PathBuf) = foreign_commit_until_released(&fixture, &repo, 0);
    let request: CommitRequest = request_for(&fixture, &repo, "blocked", "a.txt", "blocked\n");
    let (answered, notices) = submit_on_a_thread(config_for(&fixture, name.as_str(), SERVICE_ANSWER_PATIENCE), request).join().expect("client");
    assert!(matches!(answered, Err(ClientFailure::IndexInUse(_))), "{answered:?} {notices}");
    assert!(notices.contains("refs/cli-git/kept/") && notices.contains("still held") && notices.contains("was not made"), "{notices}");
    let kept: String = git_text(&fixture, &repo, &["for-each-ref", "--format=%(refname)", "refs/cli-git/kept/"]);
    assert!(kept.trim().starts_with("refs/cli-git/kept/"), "the built commit is kept: {kept:?}");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%s", kept.trim()]).trim(), "message of blocked");
    assert_eq!(open_batches(&fixture), 0);
    drop(service);
    let _restarted: Held = service_with_arguments(&fixture, name.as_str(), QUIET_NOTICE_SECONDS, &[]);
    std::fs::write(&release, b"").expect("the foreign commit is told to let go");
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "{}", String::from_utf8_lossy(&held.stderr));
    std::thread::sleep(CLIENT_CEILING * 2);
    assert_eq!(subjects_since(&fixture, &repo, base.as_str()), "foreign\n", "nothing landed by itself after the foreign program let go");
    remove_settled(&fixture);
}
