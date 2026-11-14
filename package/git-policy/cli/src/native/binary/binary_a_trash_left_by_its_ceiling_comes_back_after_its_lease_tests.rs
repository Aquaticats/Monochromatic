//! What:
//!  A trash whose emptying cannot finish inside its ceiling:
//! the item is queued again for a later look, with the tries it had,
//! the look that follows removes what is left and touches nothing it already removed,
//! a trash of more rounds than the tries a failure gets is emptied rather than given up on,
//! and a commit lands in its usual time through all of it.
//! Why:
//!  `HOUSEKEEPING_OF_ONE_ROUND` bounds only what a round starts, and a trash that began before it ran out was the
//! one item a round of `2026-11-12` spent forty minutes on
//! (`doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`).
//! The repair is a bound of the item's own (`TRASH_ITEM_CEILING`) that stops it between waves of entries and leaves the
//! rest to the next round, and the path from that answer to the round that takes it up again was judged by reading only
//! (`/var/home/user/temp/agent/change-reviews/…/review.md`, "Limits of this review").
//! These controls walk that path with a ceiling a control states in seconds (`CLI_GIT_TRASH_ITEM_SECONDS`),
//! so nothing here is a large tree or a slow disk.
//! The emptying is held inside each of its waves by a hold of the test's own
//! (`service_hold_a_step_for_a_control.rs`, `BEFORE_A_TRASH_WAVE`), which is the one state a bound can stop.
//! A round that removed entries never spends a try: the review of `af0b824df3` found a trash of a removed worktree
//! given up on in `failed` after three bounded rounds, where nothing reads it again
//! (`/home/user/temp/agent/change-reviews/git-policy-cli-landing-never-waits-answers-after-its-third-review/`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! holdBeforeTheFirstWave(); await commit(); await reached; await sleep(ceiling + 1); letGo();
//! expect(firstWaveGone && restThere && recordClaimed); await leaseEnds(); expect(trashGone && recordDone.byTryOne); await commit(); expect(commitMs < bound)
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, FixtureService, fixture, git, repository, start_fixture_service};
use git_policy_cli::fs_beneath::{ObjectId, identity_at, identity_of, open_directory_at, open_directory_beneath, open_root, plain_components};
use git_policy_cli::service_hold_a_step_for_a_control::{BEFORE_A_TRASH_WAVE, HOLD_DIRECTORY_VARIABLE, HOLD_VARIABLE, let_go_file, reached_file};
use git_policy_cli::service_work::work_directory;
use git_policy_cli::housekeeping_of_one_round::LOOK_AGAIN_AFTER_DEFERRING;
use git_policy_cli::wait_ceilings::TRASH_ITEM_CEILING_VARIABLE;
use git_policy_cli::work_item::{CLAIMED, DONE, FAILED, PENDING, WorkItem, id_of_entry, state_directory};
use git_policy_cli::worktree_trash::{PendingTrash, TRASH_PARENT, TrashManifest, entry_name, record_the_trash};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::{Duration, Instant};

/// The most this control waits for a state it expects within a moment:
/// the greeting of the service, a wave that was held, a commit.
/// It is far beyond any of them and is a hang guard, not a measurement.
const PATIENCE: Duration = Duration::from_secs(60);

/// One trash item's ceiling, as this control states it to the fixture's service:
/// two seconds.
/// It has to be shorter than the lease by far, so the round that takes the item up again is the lease that brings it back
/// and not the bound; and long enough that a hold of the wave inside it is a state and not a race.
const THE_CEILING: Duration = Duration::from_secs(2);

/// How long the control holds the first wave:
/// longer than the ceiling, so the item cannot finish inside its bound whatever else the machine does.
const THE_HOLD: Duration = Duration::from_secs(3);

/// How long past the pause a look asks for the control waits for the round that takes the item up again:
/// a quarter of a minute, which is a wide margin over one look's scheduling.
/// The pause itself is read from the shipped `LOOK_AGAIN_AFTER_DEFERRING`, so no figure here is a guess about the service.
const THE_LOOK_AGAIN_MARGIN: Duration = Duration::from_secs(15);

/// How many entries the trash holds.
/// Why:
///   A wave of one trash is at most `places_a_round_may_take()` entries, which is two for a service whose bound is
///   `SERVICE_WORKER_CAP`, so four entries are always more than one wave and the bound always has something left to stop.
/// A host whose wave is all four fails this control's premise loudly rather than passing it by accident.
const THE_ENTRIES: usize = 4;

/// The reference every trash record carries (`worktree_trash_dispose::TRASH_REFERENCE`).
const TRASH_REFERENCE: &str = "refs/cli-git/trash";

/// How long a commit may take while the trash path cycles:
/// twenty seconds, as in `binary_a_commit_lands_while_every_kind_of_work_is_held_tests.rs`, which is four of the
/// client's five-second silence windows.
const A_WAIT_NO_OTHER_WORK_MAY_CAUSE: Duration = Duration::from_secs(20);

/// A repository with a bare remote,
/// a linked worktree holding a planted trash,
/// and a service of the repository's own that holds the trash at its first wave.
struct Setup {
    /// The disposable fixture, removed at the end.
    fixture: Fixture,
    /// The working repository.
    repo: PathBuf,
    /// The linked worktree whose trash this control empties.
    worktree: PathBuf,
    /// The service, stopped with this value.
    service: FixtureService,
    /// Where the held wave says it was reached and is let go.
    holds: PathBuf,
    /// The trash's recorded name.
    name: String,
    /// How many entries this trash holds.
    entries: usize,
}

impl Setup {
    /// The repository's own Git directory,
    /// which is the common directory of its linked worktree and the Git directory of the service.
    fn git_dir(&self) -> PathBuf {
        return self.repo.join(".git");
    }

    /// The work directory the service drains.
    fn work(&self) -> PathBuf {
        return work_directory(self.git_dir().as_path());
    }

    /// The trash directory the record names.
    fn trash(&self) -> PathBuf {
        return self.worktree.join(TRASH_PARENT).join(self.name.as_str());
    }

    /// The record's id,
    /// which is the trash's name.
    fn id(&self) -> String {
        return format!("trash-{}", self.name);
    }

    /// Whether one entry of the trash is still there.
    fn entry_present(&self, index: usize) -> bool {
        return self.trash().join(entry_name(index)).exists();
    }

    /// How many of the trash's entries are still there.
    fn entries_present(&self) -> usize {
        let mut present: usize = 0;
        for index in 0..self.entries {
            if self.entry_present(index) {
                present += 1;
            }
        }
        return present;
    }

    /// Where the record stands now:
    /// a state its name is in,
    /// or `claimed`,
    /// whose file names carry the reading of the claim as well as the id.
    fn record_state(&self) -> &'static str {
        for state in [PENDING, DONE, FAILED] {
            if state_directory(self.work().as_path(), state).join(self.id()).is_file() {
                return state;
            }
        }
        let claimed: bool = std::fs::read_dir(state_directory(self.work().as_path(), CLAIMED)).is_ok_and(|entries| {
            return entries.flatten().any(|entry| { return id_of_entry(entry.file_name().to_string_lossy().as_ref()) == self.id().as_str(); });
        });
        return if claimed { CLAIMED } else { "no state" };
    }

    /// The record in `state`,
    /// read back,
    /// and nothing when it is not there.
    fn record_in(&self, state: &str) -> Option<WorkItem> {
        let directory: PathBuf = state_directory(self.work().as_path(), state);
        let text: String = std::fs::read_to_string(directory.join(self.id())).ok()?;
        return WorkItem::decode(text.as_str()).ok();
    }

    /// Whether the service has reached the wave the control holds.
    fn reached(&self) -> bool {
        return reached_file(self.holds.as_path(), BEFORE_A_TRASH_WAVE).exists();
    }

    /// Let the held wave go on,
    /// and every later wave of every later round,
    /// which is what a let-go file that stays does.
    fn let_go(&self) {
        std::fs::write(let_go_file(self.holds.as_path(), BEFORE_A_TRASH_WAVE), b"").expect("the file that lets a held wave go");
    }
}

/// What:
///  A repository tracking a bare remote,
///  a linked worktree of it with a planted trash of `THE_ENTRIES` entries,
///  the trash's record in the repository's own work directory,
///  and a service started to hold the emptying at its first wave.
/// Why:
///   The trash must be a linked worktree's, because the emptying proves the worktree it opens is the one whose
/// administrative directory the record was read from (`worktree_trash_dispose::verify_association`), and the record must
/// sit under the common directory, because one repository's service drains one work directory.
/// The holds are named before the first command starts the service, because it reads them once for its own life.
fn setup(name: &str, entries: usize, ceiling: Duration) -> Setup {
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
    let trash_name: String = "7".repeat(40);
    let worktree: PathBuf = fixture.root.join("topic");
    let manifest: TrashManifest = plant_the_trash(repo.join(".git").as_path(), worktree.as_path(), trash_name.as_str(), entries);
    let pending: PendingTrash = PendingTrash { destination: worktree.clone(), common_dir: repo.join(".git"), manifest, taken_over: 0, quiet: true };
    record_the_trash(&pending).expect("the trash's record is queued");
    let holds: PathBuf = fixture.root.join("holds");
    std::fs::create_dir_all(holds.as_path()).expect("the directory the holds go in");
    name_the_holds(&fixture, holds.as_path(), ceiling);
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, worktree, service, holds, name: trash_name, entries };
}

/// What:
///  A linked worktree of `common` whose `.git` file points into it,
///  with a trash of `THE_ENTRIES` entries under the trash parent,
///  and the manifest a leave-out would have written for it.
/// Why:
///   This is the shape `git worktree add` leaves behind, written by hand so the control does not need a snapshot:
/// what the emptying proves is the `.git` pointer, each entry's identity and the trash's own identity, which are all here.
fn plant_the_trash(common: &Path, worktree: &Path, name: &str, entries: usize) -> TrashManifest {
    let git_dir: PathBuf = common.join("worktrees").join("topic");
    std::fs::create_dir_all(git_dir.as_path()).expect("the administrative directory");
    std::fs::create_dir_all(worktree).expect("the worktree");
    std::fs::write(worktree.join(".git"), format!("gitdir: {}\n", git_dir.display())).expect("the .git file");
    for index in 0..entries {
        let entry: PathBuf = worktree.join(TRASH_PARENT).join(name).join(entry_name(index));
        std::fs::create_dir_all(entry.join("payload")).expect("a trash entry");
        std::fs::write(entry.join("payload/credentials"), b"another worktree's token\n").expect("an entry file");
    }
    let held: std::fs::File = open_root(worktree).expect("the worktree");
    let names = plain_components(Path::new(TRASH_PARENT)).expect("a plain parent");
    let parent: std::fs::File = open_directory_beneath(&held, names.as_slice()).expect("the trash parent");
    let trash: std::fs::File = open_directory_at(&parent, OsStr::new(name)).expect("the trash");
    let identities: Vec<ObjectId> = (0..entries).map(|index: usize| { return identity_at(&trash, OsStr::new(entry_name(index).as_str())).expect("an entry identity"); }).collect();
    return TrashManifest { name: String::from(name), trash: identity_of(&trash).expect("the trash identity"), entries: identities };
}

/// Name the hold and the ceiling this control states in the fixture service's environment.
fn name_the_holds(fixture: &Fixture, holds: &Path, ceiling: Duration) {
    let mut environment = fixture.service_environment.lock().expect("the fixture's service environment");
    environment.push((OsString::from(HOLD_VARIABLE), OsString::from(BEFORE_A_TRASH_WAVE)));
    environment.push((OsString::from(HOLD_DIRECTORY_VARIABLE), holds.as_os_str().to_os_string()));
    environment.push((OsString::from(TRASH_ITEM_CEILING_VARIABLE), OsString::from(ceiling.as_secs().to_string())));
}

/// Commit `content` through the service and answer how long the command took.
fn commit_and_time(setup: &Setup, content: &str) -> (Output, Duration) {
    std::fs::write(setup.repo.join("a.txt"), content).expect("the edit");
    let mut command: std::process::Command = wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let began: Instant = Instant::now();
    let output: Output = command.output().expect("the wrapped commit runs");
    return (output, began.elapsed());
}

/// Wait, bounded, until `condition` holds.
fn wait_until(what: &str, patience: Duration, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < patience, "never happened within {patience:?}: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// What:
///  A trash left by its ceiling is taken up again by the look that follows,
///  with the tries it had,
///  the round that follows removes what the first one left,
///  and a commit lands in its usual time while both rounds run.
/// Why:
///   This is the whole path a bounded trash item depends on:
/// the ceiling answers `LeftForALaterRound` with entries removed,
/// the record is queued again rather than retried,
/// the next look runs it with the try count it had,
/// and it proves each entry it removes against the manifest the record still carries.
/// An entry the first wave already removed is handed to the removal again and comes back absent, which is not a refusal:
/// the whole trash is gone and the record is done, so nothing was removed twice and nothing was reported as a failure
/// for an entry that was already gone.
#[test]
fn a_trash_left_by_its_ceiling_is_taken_up_by_the_next_look_and_a_commit_lands_meanwhile() {
    let setup: Setup = setup("trash-left-by-its-ceiling", THE_ENTRIES, THE_CEILING);
    // The commit wakes the service, and its lane claims the trash and stops at the wave this control holds.
    let (first, first_ms): (Output, Duration) = commit_and_time(&setup, "one\n");
    assert!(first.status.success(), "{}", String::from_utf8_lossy(first.stderr.as_slice()));
    eprintln!("MEASURE waking_commit_ms={} alongside the first wave", first_ms.as_millis());
    wait_until("the emptying reached the wave the control holds", PATIENCE, &|| { return setup.reached(); });
    // The wave is held longer than the ceiling, so the item cannot finish inside its bound.
    std::thread::sleep(THE_HOLD);
    setup.let_go();
    // The wave that began is finished, and the ceiling stops the next one.
    wait_until("the wave the control held removed the entries it named", PATIENCE, &|| { return setup.entries_present() < THE_ENTRIES; });
    let state_after_the_wave: &'static str = setup.record_state();
    assert_eq!(
        state_after_the_wave, PENDING,
        "a bound that ran out with entries removed queues the record again for a later look, and spends no try; the record stands in {state_after_the_wave}"
    );
    let removed_before_the_retry: usize = THE_ENTRIES - setup.entries_present();
    assert!(removed_before_the_retry > 0, "the wave that began was finished, so the entries it named are gone");
    assert!(
        setup.entries_present() > 0,
        "the ceiling stopped the next wave, so its entries are left for a later round; {removed_before_the_retry} of {THE_ENTRIES} were removed and the rest went with them"
    );
    assert!(setup.trash().is_dir(), "the trash directory keeps the identity its record proves it by, so the later round finds it");
    assert_eq!(setup.record_in(PENDING).map(|item| { return item.tries; }), Some(0), "the record queued again is the first try's: a round that removed entries spends none");
    assert_eq!(setup.record_in(DONE), None, "the record is not done while entries are left");
    assert_eq!(setup.record_in(FAILED), None, "a bound that ran out is not a failure");
    // The look the record asked for runs it again, and that round removes what is left.
    wait_until("the look that followed emptied the trash", LOOK_AGAIN_AFTER_DEFERRING + THE_LOOK_AGAIN_MARGIN, &|| {
        return !setup.trash().exists() && setup.record_state() == DONE;
    });
    assert_eq!(setup.entries_present(), 0, "every entry the first wave left is gone");
    let done: WorkItem = setup.record_in(DONE).expect("the record that is done");
    assert_eq!(done.tries, 0, "the rounds that emptied it were taken up again for later looks, and no round spent a try of a failure");
    assert_eq!(done.kind.name(), "trash");
    assert_eq!(done.branch_ref, TRASH_REFERENCE);
    // A commit lands in its usual time after both rounds, as it did between them.
    let (second, second_ms): (Output, Duration) = commit_and_time(&setup, "two\n");
    assert!(second.status.success(), "{}", String::from_utf8_lossy(second.stderr.as_slice()));
    eprintln!("MEASURE retry_then_commit_ms={} removed_before_the_retry={removed_before_the_retry}", second_ms.as_millis());
    assert!(first_ms < A_WAIT_NO_OTHER_WORK_MAY_CAUSE, "the waking commit waited {first_ms:?} behind the trash");
    assert!(second_ms < A_WAIT_NO_OTHER_WORK_MAY_CAUSE, "the commit after the retry waited {second_ms:?} behind the trash");
    remove_settled(&setup.fixture);
}
