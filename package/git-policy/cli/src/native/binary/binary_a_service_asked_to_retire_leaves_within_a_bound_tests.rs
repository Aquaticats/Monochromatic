//! What:
//!  A service asked to retire while the lane of its slow work holds every place that lane may take and a landing's batch of
//!  records waits behind it:
//! how long the service takes to end,
//! what the client that asked it to retire sees,
//! what the commit that was landed leaves in the repository once everything is killed,
//! and a commit of a newer client landing while what the exit left is still there.
//! Why:
//!  A wrapper of a newer release asks the service of an older one to retire and waits for it for `RETIREMENT_WAIT` (ten
//!  seconds) before it stops its unit and gives up on the commit it was making
//!  (`service_provision_earlier_service_retirement.rs`).
//!  The serving loop leaves by waiting for the background syncs of the landings it answered (`service_durable::drain`), and
//!  a batch of those syncs waits for a place of the lane of slow work, so a service asked to retire while slow work holds
//!  that lane ends as late as the slow work (a read of the change under `2026-11-14`, observation O1 of
//!  `change-reviews/git-policy-cli-landing-never-waits-control-of-a-trash-that-reaches-its-ceiling/review.md`; nobody had run
//!  the sequence).
//!  The owner's rules are "nothing should ever wait forever or indefinitely" and "commits should never fail", so the exit
//!  must end within a bound whatever the slow lane holds, and must leave the commit it landed no less safe than a kill at
//!  that instruction does.
//!  The holds are states and not durations (`service_hold_a_step_for_a_control.rs`), and the premise this control needs,
//!  "no place of the lane is free", is checked here rather than assumed: a batch made after it says so by staying out of the
//!  service's own log, after an earlier batch proved that a batch that runs does write its line there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await holdTheLane(); await commit('one'); const last = await commit('waiting'); const asked = now(); await askToRetire(socket)
//! const gone = await waitForTheServiceToEnd(RETIREMENT_WAIT); killEverything(); expect(gitHas(last)).toBe(true); await commit('newer')
//! expect(gone).toBe(true)
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, FixtureService, SERVICE_LOG_FILE, direct, fixture, git, repository, service_log, start_fixture_service};
use git_policy_cli::service_client_listen_and_ask_to_retire::ask_to_retire;
use git_policy_cli::service_hold_a_step_for_a_control::{HOLD_DIRECTORY_VARIABLE, HOLD_VARIABLE, INSIDE_A_SCRATCH_REMOVAL, let_go_file, reached_file};
use git_policy_cli::service_instance::socket_for;
use git_policy_cli::service_provision::RETIREMENT_WAIT;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{Duration, Instant};

/// The most this control waits for a state it expects within a moment:
/// the greeting of the service,
/// a removal that was reached,
/// a commit.
/// It is far beyond any of them and is a hang guard, not a measurement.
const PATIENCE: Duration = Duration::from_secs(60);

/// How long the control watches, with the holds still in place, for the batch of a commit that must not be synced:
/// two seconds.
/// A batch takes a place of the lane and is written within milliseconds when one is free, and both removals that hold the
/// lane have already asked for their place when the next commit queues its batch,
/// so a batch that is written at all is written inside this watch and no figure here is a claim about how long a sync takes.
const THE_WATCH_FOR_A_SYNC: Duration = Duration::from_secs(2);

/// How many places of the lane of slow work this service's slow work may take:
/// two.
/// A service is held to `SERVICE_WORKER_CAP` (four) places of the process's pool and keeps `SLOW_WORK_RESERVE` (two) of them
/// out of the slow lane, so `places_a_round_may_take` is two.
/// The figure is stated here and not read from the crate, because the budget of this test process is not the service's;
/// nothing relies on it being right either:
/// the commit after the last one that filled the lane is the one whose batch waits, and this control says so loudly when it
/// arrives at the bound below instead of passing.
const THE_PLACES_OF_THE_LANE: usize = 2;

/// The most commits this control makes before it concludes that every place of the lane is held:
/// twice the places, so a commit whose batch was written before the removal of its own tree asked for a place (which fills
/// one place on the next commit) is room to try again.
const THE_MOST_COMMITS: usize = THE_PLACES_OF_THE_LANE * 2;

/// The line a batch of a landing's records writes when it is synced, which is how this control tells a batch that ran from
/// one that is still waiting for a place of the lane.
const A_BATCH_WAS_SYNCED: &str = "the background sync of";

/// A repository with a bare remote and a service of the repository's own that holds the removal of a scratch tree open.
struct Setup {
    /// The disposable fixture, removed at the end.
    fixture: Fixture,
    /// The working repository.
    repo: PathBuf,
    /// The service, stopped with this value.
    service: FixtureService,
    /// Where the held removal says it was reached and is let go.
    holds: PathBuf,
}

impl Setup {
    /// The repository's own Git directory.
    fn git_dir(&self) -> PathBuf {
        return self.repo.join(".git");
    }

    /// The socket the wrapper reaches this repository's service at.
    fn socket(&self) -> PathBuf {
        return socket_for(self.service.runtime.as_path(), self.git_dir().as_path());
    }

    /// Whether the service has reached the removal the control holds.
    fn reached(&self) -> bool {
        return reached_file(self.holds.as_path(), INSIDE_A_SCRATCH_REMOVAL).exists();
    }

    /// Let the held removal go on,
    /// and every later one with it.
    fn let_go(&self) {
        std::fs::write(let_go_file(self.holds.as_path(), INSIDE_A_SCRATCH_REMOVAL), b"").expect("the file that lets a held removal go");
    }

    /// How many batches of a landing's records the service has written a line for,
    /// counted over the whole log rather than the part a failure carries.
    fn batches_synced(&self) -> usize {
        let said: String = std::fs::read_to_string(self.fixture.root.join(SERVICE_LOG_FILE)).unwrap_or_default();
        return said.matches(A_BATCH_WAS_SYNCED).count();
    }

    /// Whether the service of this worktree has ended,
    /// forgetting it so that the next commit starts a new one as the manager would.
    fn service_ended(&self) -> bool {
        return self.service.forget_ended();
    }
}

/// What:
///  A repository tracking a bare remote,
///  and a service of its own started with the removal of a scratch tree held open.
/// Why:
///   The holds are named before the first command starts the service,
/// because the service reads them once for its own life.
fn setup(name: &str) -> Setup {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("the file");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    // The upkeep of a control is not what it measures, so it runs in front of the command rather than behind it.
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    git(&fixture, &repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&fixture, &repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let holds: PathBuf = fixture.root.join("holds");
    std::fs::create_dir_all(holds.as_path()).expect("the directory the holds go in");
    name_the_holds(&fixture, holds.as_path());
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, service, holds };
}

/// Name the step this control holds in the fixture service's environment.
fn name_the_holds(fixture: &Fixture, holds: &Path) {
    let mut environment = fixture.service_environment.lock().expect("the fixture's service environment");
    environment.push((OsString::from(HOLD_VARIABLE), OsString::from(INSIDE_A_SCRATCH_REMOVAL)));
    environment.push((OsString::from(HOLD_DIRECTORY_VARIABLE), holds.as_os_str().to_os_string()));
}

/// Commit `content` through the service and answer what the command said and how long it took.
fn commit_and_time(setup: &Setup, content: &str) -> (Output, Duration) {
    std::fs::write(setup.repo.join("a.txt"), content).expect("the edit");
    let mut command: std::process::Command = wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let began: Instant = Instant::now();
    let output: Output = command.output().expect("the wrapped commit runs");
    return (output, began.elapsed());
}

/// The commit the repository's branch names now.
fn head(setup: &Setup) -> String {
    let named: Output = direct(&setup.fixture).current_dir(setup.repo.as_path()).args(["rev-parse", "HEAD"]).output().expect("real Git answers");
    return String::from_utf8_lossy(named.stdout.as_slice()).trim().to_owned();
}

/// Whether the repository holds `commit`, read with real Git,
/// which is what the person's next command reads.
fn repository_has(setup: &Setup, commit: &str) -> bool {
    let asked: Output = direct(&setup.fixture)
        .current_dir(setup.repo.as_path())
        .args(["cat-file", "-e", format!("{commit}^{{commit}}").as_str()])
        .output()
        .expect("real Git answers");
    return asked.status.success();
}

/// Wait, bounded, until `condition` holds.
fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened within {PATIENCE:?}: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// What:
///  Make commits until one of them queues a batch of records that the lane of slow work holds out,
///  then ask the service to retire while that batch waits,
///  and answer how long the service took to end,
///  or nothing when it was still there after the wait a newer release gives it.
/// Why:
///   This is the sequence a read of the change under review describes and nobody had run:
/// a landing queues a batch of its records,
/// the lane of slow work holds every place that batch waits for,
/// and a client asks the service to retire.
/// A batch says whether a place was free by writing a line of its own, so commits are made until one's line does not appear,
/// and that commit's records are the ones waiting.
/// Each commit is answered before the next is made and before the retire is asked, so no landing is in flight when the
/// retire arrives and the waiting batch is the last thing the service queued.
fn retire_with_a_batch_waiting_behind_the_lane(setup: &Setup) -> Option<Duration> {
    let mut attempts: usize = 0;
    loop {
        attempts += 1;
        assert!(
            attempts <= THE_MOST_COMMITS,
            "{attempts} commits and every one of them had its records synced, so this control never reached the state it measures: {}",
            service_log(setup.fixture.root.as_path())
        );
        // A batch whose records were synced proves that this control can see a batch that ran, so the batch whose line does
        //  not appear below is not a probe that could never say anything.
        let synced_before: usize = setup.batches_synced();
        let (made, took): (Output, Duration) = commit_and_time(setup, format!("attempt {attempts}\n").as_str());
        assert!(made.status.success(), "{}", String::from_utf8_lossy(made.stderr.as_slice()));
        wait_until("a removal of a landing's scratch tree was reached", &|| { return setup.reached(); });
        std::thread::sleep(THE_WATCH_FOR_A_SYNC);
        let synced_after: usize = setup.batches_synced();
        eprintln!(
            "MEASURE commit_{attempts}_ms={} landed={} batches_before={synced_before} batches_after={synced_after}",
            took.as_millis(),
            head(setup)
        );
        if synced_after == synced_before {
            // No place of the lane was free, so this commit's batch of records is waiting for one.
            break;
        }
    }
    // A newer release's client asks the service of this one to retire, and that ask waits for no bound of its own:
    // the greeting, the one line, and the client is gone.
    let ask_began: Instant = Instant::now();
    ask_to_retire(setup.socket().as_path());
    eprintln!("MEASURE the_ask_itself_ms={}", ask_began.elapsed().as_millis());
    // The wait a newer release gives the service it asked: `RETIREMENT_WAIT`, and not one moment more.
    let asked: Instant = Instant::now();
    while asked.elapsed() < RETIREMENT_WAIT {
        if setup.service_ended() {
            return Some(asked.elapsed());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    return if setup.service_ended() { Some(asked.elapsed()) } else { None };
}

/// What:
///  A service asked to retire leaves within the wait a newer release gives it,
///  whatever the lane of its slow work holds,
///  and the commit it landed is still in the repository once everything is killed,
///  and a commit of a newer client lands while what the exit left is still there.
/// Why:
///   This is the promise the task is about:
/// the service's exit has a bound the lane of slow work cannot move,
/// and what it gives up when that bound is reached is what a kill at that instruction gives up:
/// the records of a landing reach the disk with the operating system's own write-back,
/// while the commit itself is in the repository,
/// on the branch,
/// and readable by the next command.
/// The check on the landed commit runs after the kill and before the assertion about the bound, so a run that fails the
///  bound still shows what the kill left.
#[test]
fn a_service_asked_to_retire_leaves_within_a_bound_while_slow_work_holds_its_lane() {
    let setup: Setup = setup("a-service-asked-to-retire-with-a-batch-waiting");
    let ended: Option<Duration> = retire_with_a_batch_waiting_behind_the_lane(&setup);
    let landed: String = head(&setup);
    eprintln!("MEASURE the_service_ended_within={ended:?} of {RETIREMENT_WAIT:?}");
    // A kill of everything: what the commit is left as when the process dies at that instruction is what the exit leaves.
    setup.service.stop_all();
    assert!(repository_has(&setup, landed.as_str()), "the commit the client was answered about is no longer in the repository after everything was killed");
    assert_eq!(head(&setup), landed, "the branch no longer names the commit the client was answered about after everything was killed");
    // A newer client's commit lands while what the exit left is still there:
    // the fixture has no manager, so it starts the instance the manager would start once the old one is gone, and the held
    //  removal is let go afterwards so the fixture's own state is complete when it is removed.
    setup.service.serve(&setup.fixture, setup.git_dir().as_path());
    let (newer, newer_ms): (Output, Duration) = commit_and_time(&setup, "newer\n");
    let newer_landed: String = head(&setup);
    eprintln!("MEASURE newer_client_commit_ms={} landed={newer_landed}", newer_ms.as_millis());
    assert!(
        newer.status.success(),
        "a commit of a newer client did not land after the service was asked to retire: {}",
        String::from_utf8_lossy(newer.stderr.as_slice())
    );
    assert_ne!(newer_landed, landed, "the newer client's commit is a commit of its own");
    setup.let_go();
    assert!(
        ended.is_some(),
        "the service was still there {RETIREMENT_WAIT:?} after it was asked to retire, while slow work held the lane its records wait behind, so a newer release's commit fails with a service \"still running\""
    );
    let took: Duration = ended.unwrap_or_default();
    assert!(took < RETIREMENT_WAIT, "the service ended {took:?} after it was asked to retire, which is past the {RETIREMENT_WAIT:?} a newer release waits for it");
    remove_settled(&setup.fixture);
}
