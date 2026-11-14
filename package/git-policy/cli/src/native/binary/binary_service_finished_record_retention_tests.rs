//! What:
//!  How long the commit service remembers a finished request,
//!  through the real wrapper and the real service:
//!  a request sent again after more landings than the journal keeps,
//!  and the two conditions a record must meet before it is removed.
//! Why:
//!  A client whose answer was lost sends its request again with the same identity,
//! and the record its batch left is the only thing that makes the service answer it as already landed.
//! The journal is therefore allowed to forget a record only when two things hold:
//! no client of it may still read it,
//! and it is beyond the newest records the journal keeps.
//! Neither a burst of landings nor a long quiet time alone may take it,
//! and two services that prune at once may take nothing that should stay
//! (`doc/planning/cli-git-resident-service.md`,
//! "Durability and crash recovery").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const first = await wrappedCommit('one-request'); const id = recordId();
//! await landMoreThanTheJournalKeeps(); await restart(); expect(await resend(id)).toEqual({ already: true, commit: first });
//! ```

use super::commit::{remove_settled, run_commit};
use super::support::{Fixture, FixtureService, fixture, git, repository, start_fixture_service};
use git_policy_cli::claim_time::ClaimTime;
use git_policy_cli::service_client::{Activation, ClientConfig, Submitted, submit};
use git_policy_cli::service_instance::{instance_of, socket_for};
use git_policy_cli::service_journal::{DONE_EXTENSION, DONE_RETAINED, Intent, OPEN_EXTENSION, batch_path, intent_text, list_intents, mark_done};
use git_policy_cli::service_journal_record_age::stamp_line;
use git_policy_cli::service_protocol::CommitRequest;
use git_policy_cli::service_server::Bind;
use git_policy_cli::wait_ceilings::A_FINISHED_RECORD_IS_READ;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

/// The committer every request of a control is landed as.
const COMMITTER: &str = "T <t@example.invalid> 1793535889 +0000";

/// The longest a control waits for anything but a landing,
/// far beyond a healthy run.
const PATIENCE: Duration = Duration::from_secs(60);

/// The boot a control writes a record of when it needs one no client of this boot may read:
/// 32 hexadecimal digits that are not this boot's.
const AN_EARLIER_BOOT: &str = "0000000000000000000000000000000b";

/// The message of the request the controls send again.
const ONE_REQUEST: &str = "one-request";

/// The finished batches of a journal,
/// by their numbers,
/// in order.
fn finished(journal: &Path) -> Vec<u64> {
    return list_intents(journal, DONE_EXTENSION)
        .iter()
        .filter_map(|(_, parsed)| { return parsed.as_ref().map(|intent| { return intent.seq; }); })
        .collect();
}

/// A time longer than the whole life of any client of a record,
/// by a minute,
/// so nothing a control states sits at the edge.
fn longer_than_a_client_s_life() -> Duration {
    return A_FINISHED_RECORD_IS_READ + Duration::from_secs(60);
}

/// What:
///  A finished record of batch `seq`,
/// in a fixture's journal,
/// that is `older` than now,
///  in the reading of the clocks it carries and in its modification time.
/// Why:
///   A journal of a stated size is a fixture's own,
/// so a control that needs more records than a person would wait for writes them,
/// in the text this release writes,
/// and finishes each batch the way a landing finishes it.
/// Both the reading and the modification time say the age,
/// because a fresh modification time alone keeps a record whatever its reading says.
fn write_finished(journal: &Path, repository: &Path, seq: u64, older: Duration) {
    let intent: Intent = Intent {
        seq,
        repository: repository.to_path_buf(),
        common_dir: repository.join(".git"),
        reference: git_policy_cli::ref_name::RefName::from_text("refs/heads/main"),
        old: String::from("a".repeat(40)),
        new: String::from("b".repeat(40)),
        ids: vec![format!("request-{seq}")],
        commits: vec![format!("{:040x}", seq)],
        holders: vec![String::new()],
        paths: Vec::new(),
        derived: false,
        index: repository.join(".git/index"),
        git_dir: repository.join(".git"),
        cleanup: None,
    };
    // A record of this boot a moment old, or one of an earlier boot that far back, which are the two ways time passes.
    let written: ClaimTime = match older.is_zero() {
        true => ClaimTime::read(),
        false => ClaimTime::read().in_boot(AN_EARLIER_BOOT).rewound(older),
    };
    std::fs::write(batch_path(journal, seq, OPEN_EXTENSION), format!("{}{}", intent_text(&intent), stamp_line(&written))).expect("an intent");
    mark_done(journal, seq).expect("the batch finishes");
    let when: std::time::SystemTime = std::time::SystemTime::now().checked_sub(older).expect("a fixture time after the epoch");
    std::fs::File::open(batch_path(journal, seq, DONE_EXTENSION)).expect("a fixture record to age").set_modified(when).expect("a fixture time to age");
}

/// A journal of `count` finished records whose numbers start at `first`,
/// each `older` than now.
fn write_finished_from(journal: &Path, repository: &Path, first: u64, count: usize, older: Duration) {
    for offset in 0..count {
        write_finished(journal, repository, first + u64::try_from(offset).expect("a count"), older);
    }
}

/// What the client of a control sends with,
/// on the socket of a fixture's service.
fn config(runtime: &Path, git_dir: &Path) -> ClientConfig {
    return ClientConfig {
        address: Bind::Path(socket_for(runtime, git_dir)),
        activation: Activation::StartedByTheWrapper,
        start_wait: PATIENCE,
        heartbeat_silence: PATIENCE,
        starter: None,
        keep_fresh: None,
        answer_ceiling: git_policy_cli::wait_ceilings::ANSWER_CEILING,
        withdraw_ceiling: git_policy_cli::wait_ceilings::WITHDRAW_CEILING,
        waiting_notice_after: PATIENCE,
        extra_wait: git_policy_cli::wait_ceilings::SHORT_EXTRA_WAIT,
        start_ceiling: git_policy_cli::wait_ceilings::START_CEILING,
    };
}

/// What:
///  A request that lands an empty commit on `main` of the repository,
///  prepared against `base`,
///  with identity `id`.
/// Why:
///   The landings a control needs beyond the count are the service's own work,
/// and a request with no change is the cheapest commit it can land.
fn empty_request(repository_root: &Path, id: &str, base: &str) -> CommitRequest {
    let scratch: PathBuf = repository_root
        .join(".git/monochromatic-git-policy-cli-scratch")
        .join(format!("monochromatic-git-policy-cli-service-{id}-0"));
    std::fs::create_dir_all(scratch.as_path()).expect("the tree the request names");
    return CommitRequest {
        id: id.to_string(),
        repository: repository_root.to_path_buf(),
        common_dir: repository_root.join(".git"),
        git_dir: repository_root.join(".git"),
        reference: git_policy_cli::ref_name::RefName::from_text("refs/heads/main"),
        base: base.to_string(),
        changes: Vec::new(),
        message: format!("message of {id}\n"),
        author: None,
        committer: COMMITTER.to_string(),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: repository_root.join(".git/index"),
        scratch,
        shadow: None,
        captured: 0,
        holder: String::new(),
    };
}

/// What a landing answered.
fn landed(config: &ClientConfig, request: &CommitRequest, what: &str) -> Submitted {
    let answered: Result<Submitted, git_policy_cli::service_messages::ClientFailure> = submit(config, request, &mut std::io::sink());
    return answered.unwrap_or_else(|failure| panic!("{what}: the service did not answer: {failure:?}"));
}

/// The commit an answer landed,
/// and whether the service said it had landed it before.
fn commit_and_already(answer: &Submitted, what: &str) -> (String, bool) {
    let Submitted::Landed { commit, already } = answer else { panic!("{what}: expected a landing, got {answer:?}") };
    return (commit.clone(), *already);
}

/// The identity,
/// the commit and the batch number of the one finished record a scene's first request left.
fn first_request_of(journal: &Path) -> (String, String, u64) {
    let listed = list_intents(journal, DONE_EXTENSION);
    let (_, Some(intent)) = listed.first().cloned().expect("one finished record") else { panic!("the first finished record parses") };
    let id: String = intent.ids.first().cloned().expect("the record names its request");
    let commit: String = intent.commits.first().cloned().expect("the record names the commit it landed");
    return (id, commit, intent.seq);
}

/// A repository with a file committed,
/// a running service and the journal it writes.
struct Scene {
    fixture: Fixture,
    repo: PathBuf,
    service: FixtureService,
    journal: PathBuf,
}

fn scene(name: &str) -> Scene {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    let journal: PathBuf = fixture.root.join("journal");
    // A control that writes its journal before the service starts makes the folder the service would make.
    std::fs::create_dir_all(journal.as_path()).expect("the journal's folder");
    let service: FixtureService = start_fixture_service(&fixture);
    return Scene { fixture, repo, service, journal };
}

impl Scene {
    /// The service of this worktree,
    /// started as the manager starts it on the first connection.
    fn serve(&self) {
        self.service.serve(&self.fixture, self.repo.join(".git").as_path());
    }

    /// The client this scene's service answers.
    fn client(&self) -> ClientConfig {
        return config(self.service.runtime.as_path(), self.repo.join(".git").as_path());
    }

    /// A wrapped commit that lands on `main`,
    /// and the message it was given.
    fn wrapped_commit(&self, message: &str) {
        std::fs::write(self.repo.join("a.txt"), format!("{message}\n")).expect("a file to commit");
        git(&self.fixture, &self.repo, &["add", "--", "a.txt"]);
        let observed = run_commit(&self.fixture, &self.repo, &["commit", "--quiet", &format!("--message={message}"), "a.txt"]);
        assert_eq!(observed.code, Some(0), "{message}: the wrapped commit landed: {}", String::from_utf8_lossy(observed.stderr.as_slice()));
    }
}

/// What:
///  A request sent again after more landings than the journal keeps is answered as already landed.
/// Why:
///   This is the audit's reproduction of release 0.6.1-alpha.12:
/// a request whose answer was lost was sent again after a thousand other landings had pushed its record past the count,
/// and the service landed it a second time.
/// The record was a hundred seconds old,
/// far inside the window in which its client may still resend it.
#[test]
fn a_request_resent_after_more_landings_than_the_journal_keeps_is_recognised() {
    let scene: Scene = scene("service-finished-resend");
    scene.serve();
    scene.wrapped_commit(ONE_REQUEST);
    let (id, commit, first_seq): (String, String, u64) = first_request_of(scene.journal.as_path());
    // What:
    //  The service that lands the rest starts after the one that landed the wrapped commit.
    // Why:
    //   A batch number comes from the reading of the clocks at its service's start,
    // so every record of a later service is above every record of an earlier one,
    // and the wrapped commit's record is then the oldest the count would push out first.
    // A commit through the wrapper starts a service of its own when the fixture offers none where it looks,
    // so where its record would otherwise be the newest, this is what states the order the audit met.
    scene.service.stop_all();
    scene.serve();
    let client: ClientConfig = scene.client();

    // More landings than the journal keeps, one after another, each a batch of its own.
    let mut tip: String = commit.clone();
    let mut landings: usize = 0;
    // The wrapped commit left the first record, and each landing is a batch of its own, so one more than the count
    // leaves the oldest beyond it. The journal is read once at the end: reading it after every landing would cost the
    // square of the records.
    while landings < DONE_RETAINED {
        let filler: CommitRequest = empty_request(scene.repo.as_path(), format!("filler-{landings}").as_str(), tip.as_str());
        let answer: Submitted = landed(&client, &filler, "a filler landing");
        let (made, already) = commit_and_already(&answer, "a filler landing");
        assert!(!already, "a filler request the service never saw lands for the first time");
        tip = made;
        landings += 1;
    }
    assert_eq!(finished(scene.journal.as_path()).len(), DONE_RETAINED + 1, "the journal holds one record more than it keeps, so the oldest is beyond the count");
    assert_eq!(finished(scene.journal.as_path()).first().copied(), Some(first_seq), "the record of the wrapped commit is the oldest by its batch number");

    // The restart whose start is where the journal decides what it remembers.
    scene.service.stop_all();
    scene.serve();

    // The same identity again, prepared against the branch as it is now, as a client that rebuilt its attempt sends it.
    let again: CommitRequest = empty_request(scene.repo.as_path(), id.as_str(), tip.as_str());
    let answer: Submitted = landed(&client, &again, "the request sent again");
    let (made, already) = commit_and_already(&answer, "the request sent again");
    assert!(already, "the record a client may still send its request against is kept, whatever the count");
    assert_eq!(made, commit, "the answer names the commit the request made the first time");
    let subjects: String = super::commit::git_text(&scene.fixture, scene.repo.as_path(), &["log", "--format=%s", "main"]);
    assert_eq!(subjects.lines().filter(|line| { return *line == ONE_REQUEST; }).count(), 1, "the request landed once");
    scene.service.stop_all();
    remove_settled(&scene.fixture);
}
/// What:
///  A finished record older than the age but within the count stays.
/// Why:
///   The count is the other half of the rule:
/// a client that was stalled past any age the service states is still recognised
/// while the journal holds fewer records than it keeps.
#[test]
fn a_finished_record_older_than_the_age_but_within_the_count_stays() {
    let scene: Scene = scene("service-finished-old-within");
    write_finished_from(scene.journal.as_path(), scene.repo.as_path(), 1, DONE_RETAINED, longer_than_a_client_s_life());
    scene.serve();
    scene.wrapped_commit("after-the-restart");
    let kept: Vec<u64> = finished(scene.journal.as_path());
    assert_eq!(kept.len(), DONE_RETAINED + 1, "every record of a journal within the count stays, and the landing added one");
    for seq in 1..=u64::try_from(DONE_RETAINED).expect("a count") {
        assert!(kept.contains(&seq), "batch {seq} is past every client of it and within the count, so it stays");
    }
    scene.service.stop_all();
    remove_settled(&scene.fixture);
}

/// What:
///  A finished record beyond the count but younger than the age stays.
/// Why:
///   A burst of landings pushes the record of a request whose client is still waiting past the count,
/// and a service that forgot it there would land that request a second time.
#[test]
fn a_finished_record_beyond_the_count_but_younger_than_the_age_stays() {
    let scene: Scene = scene("service-finished-young-beyond");
    let beyond: usize = 4;
    write_finished_from(scene.journal.as_path(), scene.repo.as_path(), 1, DONE_RETAINED + beyond, Duration::ZERO);
    scene.serve();
    scene.wrapped_commit("after-the-restart");
    let kept: Vec<u64> = finished(scene.journal.as_path());
    assert_eq!(kept.len(), DONE_RETAINED + beyond + 1, "a client of every young record may still read it, so none goes");
    for seq in 1..=u64::try_from(DONE_RETAINED + beyond).expect("a count") {
        assert!(kept.contains(&seq), "batch {seq} is younger than the age, so it stays however far beyond the count it is");
    }
    scene.service.stop_all();
    remove_settled(&scene.fixture);
}

/// What:
///  A finished record both older than the age and beyond the count goes,
///  and the one after it stays.
/// Why:
///   This is the only case where the journal spends the memory of a landed request,
/// so a rule that never removed anything would grow without end.
#[test]
fn a_finished_record_older_than_the_age_and_beyond_the_count_goes() {
    let scene: Scene = scene("service-finished-past-beyond");
    let gone: usize = 4;
    write_finished_from(scene.journal.as_path(), scene.repo.as_path(), 1, gone, longer_than_a_client_s_life());
    write_finished_from(scene.journal.as_path(), scene.repo.as_path(), u64::try_from(gone).expect("a count") + 1, DONE_RETAINED, Duration::ZERO);
    scene.serve();
    scene.wrapped_commit("after-the-restart");
    let kept: Vec<u64> = finished(scene.journal.as_path());
    for seq in 1..=u64::try_from(gone).expect("a count") {
        assert!(!kept.contains(&seq), "batch {seq} is past every client of it and beyond the count, so it goes");
    }
    assert_eq!(kept.len(), DONE_RETAINED + 1, "every young record stays and the landing added one");
    scene.service.stop_all();
    remove_settled(&scene.fixture);
}

/// What:
///  Two services that prune one journal at the same moment lose no record that should stay.
/// Why:
///   Two instances of one worktree can start together while an earlier one is still serving,
/// and each decides from its own listing of the journal.
/// A pruner that counted the records another pruner had already removed as its own would take records no client is done with.
#[test]
fn two_services_pruning_at_once_lose_no_record_that_should_stay() {
    let scene: Scene = scene("service-finished-two-services");
    let gone: usize = 4;
    let beyond: usize = 4;
    write_finished_from(scene.journal.as_path(), scene.repo.as_path(), 1, gone, longer_than_a_client_s_life());
    write_finished_from(
        scene.journal.as_path(),
        scene.repo.as_path(),
        u64::try_from(gone).expect("a count") + 1,
        DONE_RETAINED + beyond,
        Duration::ZERO,
    );
    let git_dir: PathBuf = scene.repo.join(".git");
    let mut services: Vec<Child> = Vec::new();
    let mut sockets: Vec<PathBuf> = Vec::new();
    for copy in 0..2 {
        let socket: PathBuf = scene.service.runtime.join(format!("pruning-service-{copy}.sock"));
        let child: Child = super::support::service_command()
            .args(["--journal"])
            .arg(scene.journal.as_path())
            .arg("--path")
            .arg(&socket)
            .args(["--instance", instance_of(git_dir.as_path()).as_str()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("a service starts");
        services.push(child);
        sockets.push(socket);
    }
    let began: Instant = Instant::now();
    for socket in &sockets {
        while std::os::unix::net::UnixStream::connect(socket).is_err() {
            assert!(began.elapsed() < PATIENCE, "the service at {} never listened", socket.display());
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    // An answer from each service says that its start, and the pruning in it, is over.
    let mut tip: String = super::commit::git_text(&scene.fixture, scene.repo.as_path(), &["rev-parse", "HEAD"]).trim_end().to_string();
    for (at, socket) in sockets.iter().enumerate() {
        let client = ClientConfig { address: Bind::Path(socket.clone()), ..scene.client() };
        let request: CommitRequest = empty_request(scene.repo.as_path(), format!("answer-{at}").as_str(), tip.as_str());
        let (made, _already) = commit_and_already(&landed(&client, &request, "an answer"), "an answer");
        tip = made;
    }
    let kept: Vec<u64> = finished(scene.journal.as_path());
    for seq in 1..=u64::try_from(gone).expect("a count") {
        assert!(!kept.contains(&seq), "batch {seq} is past every client of it and beyond the count, so it goes");
    }
    let first_kept: u64 = u64::try_from(gone).expect("a count") + 1;
    for offset in 0..DONE_RETAINED + beyond {
        let seq: u64 = first_kept + u64::try_from(offset).expect("a count");
        assert!(kept.contains(&seq), "batch {seq} is younger than the age, so no pruner may take it");
    }
    for child in &mut services {
        let _killed = child.kill();
        let _waited = child.wait();
    }
    remove_settled(&scene.fixture);
}
