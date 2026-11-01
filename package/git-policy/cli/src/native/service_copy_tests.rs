//! The service's copies against real Git: every race the settlement lock used to prevent, and every finding of the
//!  review of the claim design, as a scene that fails when the guard that answers it is removed.
//!  Nothing waits for anything: each case ends because the service runs the request, settles the state a kill left,
//!  or refuses a request whose registration is gone.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_copy_intent::{Intent, write_intent};
use crate::worktree_copy_journal::read_pending_journals;
use crate::worktree_copy_registry::{
    COPIED_MARKER_NAME, admin_identity, copied_for, ensure_registration_token, read_admin_ids, write_copied_marker,
};
use crate::worktree_copy_request::{
    GIVEN_UP_NAME, new_request_id, read_requests, requests_directory, requests_exist,
};
use crate::worktree_copy_selection::{CopySelection, STAGE_PREFIX};

/// Entries the copy creates in the scene: the `cache` directory, its `deep` directory and its two files.
pub(crate) const COPIED_ENTRIES: usize = 4;

/// A main checkout with an ignored `cache` of four entries and the repository's canonical common directory.
pub(crate) struct Scene {
    /// The fixture root.
    pub root: PathBuf,
    /// The canonical main checkout, the source of every copy that does not say otherwise.
    pub source: PathBuf,
    /// The canonical common directory.
    pub common: PathBuf,
}

impl Scene {
    /// Builds the scene named `name`.
    pub(crate) fn new(name: &str) -> Scene {
        let root: PathBuf = fixture(name);
        let source: PathBuf = repository(root.as_path(), "source");
        std::fs::write(source.join(".gitignore"), b"cache/\n").expect("gitignore");
        // The ignore file is committed, as in a real repository, so every worktree of it ignores the cache too.
        git(source.as_path(), &["add", "--", ".gitignore"]);
        git(source.as_path(), &["commit", "--quiet", "--message=ignore the cache"]);
        std::fs::create_dir_all(source.join("cache/deep")).expect("cache");
        std::fs::write(source.join("cache/deep/data"), b"payload").expect("data");
        std::fs::write(source.join("cache/top"), b"top").expect("top");
        let common: PathBuf = std::fs::canonicalize(source.join(".git")).expect("common");
        return Scene { root, source: std::fs::canonicalize(source).expect("canonical source"), common };
    }

    /// Adds a worktree named `name` beside the source and returns its canonical root.
    pub(crate) fn add_worktree(&self, name: &str) -> PathBuf {
        let path: PathBuf = self.root.join(name);
        git(
            self.source.as_path(),
            &["worktree", "add", "--quiet", "--detach", path.to_str().expect("utf8")],
        );
        return std::fs::canonicalize(path).expect("canonical worktree");
    }

    /// Removes a worktree the way a person does.
    pub(crate) fn remove_worktree(&self, root: &Path) {
        git(self.source.as_path(), &["worktree", "remove", "--force", root.to_str().expect("utf8")]);
    }

    /// Adds a worktree named `name` beside the source, locked the way `git worktree add --lock` locks it.
    pub(crate) fn add_locked_worktree(&self, name: &str) -> PathBuf {
        let path: PathBuf = self.root.join(name);
        git(
            self.source.as_path(),
            &["worktree", "add", "--quiet", "--detach", "--lock", path.to_str().expect("utf8")],
        );
        return std::fs::canonicalize(path).expect("canonical worktree");
    }

    /// Locks or unlocks the worktree at `root`.
    pub(crate) fn set_lock(&self, root: &Path, locked: bool) {
        git(
            self.source.as_path(),
            &["worktree", if locked { "lock" } else { "unlock" }, root.to_str().expect("utf8")],
        );
    }

    /// Writes the intent a creating command writes before Git runs: the worktrees that exist now are its before-set.
    pub(crate) fn intent(&self, added: Option<&Path>) -> Intent {
        let intent: Intent = Intent {
            id: new_request_id().expect("identity"),
            source_root: self.source.clone(),
            selection: CopySelection::Ignored,
            before_admin_ids: read_admin_ids(self.admin_root().as_path()).expect("names"),
            added_path: added.map(Path::to_path_buf),
            git_returned: false,
        };
        write_intent(self.common.as_path(), &intent).expect("intent");
        return intent;
    }

    /// Rewrites an intent the way the wrapper does when Git returns.
    pub(crate) fn mark_returned(&self, intent: &Intent) -> Intent {
        let returned: Intent = Intent { git_returned: true, ..intent.clone() };
        write_intent(self.common.as_path(), &returned).expect("intent");
        return returned;
    }

    /// Replaces a pending request by a copy of it with the change `change` makes.
    pub(crate) fn rewrite(&self, request: &CopyRequest, change: impl FnOnce(&mut CopyRequest)) -> CopyRequest {
        let mut changed: CopyRequest = request.clone();
        change(&mut changed);
        write_request(self.common.as_path(), &changed).expect("rewritten");
        return changed;
    }

    /// The administrative directory of the worktree at `root`.
    pub(crate) fn admin_root(&self) -> PathBuf {
        return self.common.join("worktrees");
    }

    /// Writes a request for the worktree at `destination`, copied from `source`, with the registration's identity.
    pub(crate) fn request_from(&self, destination: &Path, source: &Path) -> CopyRequest {
        let admin_id: String = destination.file_name().expect("name").to_string_lossy().into_owned();
        let identity: String = admin_identity(self.admin_root().as_path(), admin_id.as_str())
            .expect("identity")
            .expect("registered");
        let token: String = ensure_registration_token(self.admin_root().as_path(), admin_id.as_str(), Some(identity.as_str()))
            .expect("token")
            .expect("registered");
        let request: CopyRequest = CopyRequest {
            id: new_request_id().expect("identity"),
            admin_id,
            registration: token,
            destination_root: destination.to_path_buf(),
            source_root: source.to_path_buf(),
            selection: CopySelection::Ignored,
            tries: 0,
            identity,
            allow_locked: false,
        };
        write_request(self.common.as_path(), &request).expect("written");
        return request;
    }

    /// Writes a request for the worktree at `destination`, copied from the main checkout.
    pub(crate) fn request(&self, destination: &Path) -> CopyRequest {
        return self.request_from(destination, self.source.as_path());
    }

    /// One drain of the repository with a checkpoint function.
    pub(crate) fn drain_at(
        &self,
        checkpoint: &mut dyn FnMut(&CopyRequest, DrainStep) -> bool,
    ) -> (Result<Drained, String>, Vec<CopyEvent>) {
        let mut events: Vec<CopyEvent> = Vec::new();
        let drained = drain_copies(&git_context(), self.common.as_path(), &mut |event| events.push(event), checkpoint);
        return (drained, events);
    }

    /// One drain of the repository.
    pub(crate) fn drain(&self) -> Vec<CopyEvent> {
        let (drained, events) = self.drain_at(&mut |_request, _step| return true);
        assert_eq!(drained.expect("drained"), Drained::Complete);
        return events;
    }

    /// What a drain said each request came to.
    pub(crate) fn outcomes(events: &[CopyEvent]) -> Vec<(String, CopyOutcome)> {
        return events
            .iter()
            .filter_map(|event| match event {
                CopyEvent::Done { id, outcome } => return Some((id.clone(), outcome.clone())),
                _ => return None,
            })
            .collect();
    }

    /// The marker of a finished copy of the worktree named `admin_id`.
    pub(crate) fn marker(&self, admin_id: &str) -> PathBuf {
        return self.admin_root().join(admin_id).join(COPIED_MARKER_NAME);
    }

    /// Whether anything of a copy stands: a request, a journal, or a stage beside a destination.
    pub(crate) fn anything_left(&self) -> Vec<String> {
        let mut left: Vec<String> = Vec::new();
        if requests_exist(self.common.as_path()).expect("requests") {
            left.push(String::from("a request"));
        }
        if !read_pending_journals(self.common.as_path()).expect("pending").is_empty() {
            left.push(String::from("a journal"));
        }
        for entry in std::fs::read_dir(self.root.as_path()).expect("listing").flatten() {
            if entry.file_name().to_string_lossy().starts_with(STAGE_PREFIX) {
                left.push(format!("the stage {}", entry.path().display()));
            }
        }
        return left;
    }
}

/// The files of the destination's `cache`, sorted.
pub(crate) fn cache_tree(destination: &Path) -> Vec<String> {
    let mut found: Vec<String> = vec![String::from("cache")];
    let mut pending: Vec<PathBuf> = vec![destination.join("cache")];
    while let Some(directory) = pending.pop() {
        for listed in std::fs::read_dir(directory.as_path()).expect("read").flatten() {
            let path: PathBuf = listed.path();
            found.push(path.strip_prefix(destination).expect("prefix").to_string_lossy().into_owned());
            if path.is_dir() {
                pending.push(path);
            }
        }
    }
    found.sort();
    return found;
}

/// The tree a finished copy leaves.
const COPIED_TREE: [&str; 4] = ["cache", "cache/deep", "cache/deep/data", "cache/top"];

/// The requester that is the service in the same process: it runs the drain the service would run.
pub(crate) struct DirectRequester {
    /// The context the drain runs Git in.
    pub context: GitContext,
    /// How many times the service was nudged.
    pub wakes: usize,
}

impl crate::worktree_copy::CopyRequester for DirectRequester {
    fn request(
        &mut self,
        common_dir: &Path,
        ids: &[String],
        stderr: &mut dyn Write,
    ) -> Result<Vec<(String, CopyOutcome)>, WorktreeCopyError> {
        let mut answers: Vec<(String, CopyOutcome)> = Vec::new();
        drain_copies(&self.context, common_dir, &mut |event| match event {
            CopyEvent::Notice(text) => {
                let _written = writeln!(stderr, "{text}");
            }
            CopyEvent::Done { id, outcome } if ids.contains(&id) => answers.push((id, outcome)),
            _ => {}
        }, &mut |_request, _step| return true)
        .map_err(WorktreeCopyError)?;
        return Ok(answers);
    }

    fn wake(&mut self, _common_dir: &Path) {
        self.wakes += 1;
    }
}

//region Findings 1 and 2: copies run one at a time, and no process table decides anything

/// A runner that records how many drains run at once.
struct CountingRunner {
    running: AtomicUsize,
    most: AtomicUsize,
    runs: AtomicUsize,
}

impl CopyRunner for CountingRunner {
    fn drain(&self, _common_dir: &Path, _events: &mut dyn FnMut(CopyEvent)) -> Result<(), String> {
        let now: usize = self.running.fetch_add(1, Ordering::SeqCst) + 1;
        self.most.fetch_max(now, Ordering::SeqCst);
        std::thread::sleep(std::time::Duration::from_millis(80));
        self.running.fetch_sub(1, Ordering::SeqCst);
        self.runs.fetch_add(1, Ordering::SeqCst);
        return Ok(());
    }
}

/// Clients that ask together for one repository are served one after another by the one copy thread, however many
///  there are, so no two copies of a repository ever overlap and nothing has to prove another process dead.
#[test]
fn simultaneous_requests_are_served_one_at_a_time() {
    let runner: CountingRunner = CountingRunner { running: AtomicUsize::new(0), most: AtomicUsize::new(0), runs: AtomicUsize::new(0) };
    let busy: AtomicUsize = AtomicUsize::new(0);
    let (sender, receiver) = std::sync::mpsc::channel::<CopyJob>();
    let mut clients: Vec<UnixStream> = Vec::new();
    // Wakes always run a drain, so every one of the four jobs drains on the one thread.
    for _number in 0..4 {
        let (client, service) = UnixStream::pair().expect("pair");
        clients.push(client);
        sender.send(CopyJob { common_dir: PathBuf::from("/repository/.git"), ids: Vec::new(), stream: service }).expect("queued");
    }
    drop(sender);
    copy_loop(&receiver, &runner, &busy);
    assert_eq!(runner.most.load(Ordering::SeqCst), 1, "two drains ran at the same moment");
    assert_eq!(runner.runs.load(Ordering::SeqCst), clients.len(), "every job drained");
    assert_eq!(busy.load(Ordering::SeqCst), 0);
}

/// A request the disk knows nothing about is a failure, never a success: the service does not guess that it was finished.
#[test]
fn a_request_nobody_knows_is_answered_as_a_failure() {
    let runner: CountingRunner = CountingRunner { running: AtomicUsize::new(0), most: AtomicUsize::new(0), runs: AtomicUsize::new(0) };
    let busy: AtomicUsize = AtomicUsize::new(0);
    let (sender, receiver) = std::sync::mpsc::channel::<CopyJob>();
    let mut clients: Vec<UnixStream> = Vec::new();
    for number in 0..2 {
        let (client, service) = UnixStream::pair().expect("pair");
        clients.push(client);
        sender
            .send(CopyJob { common_dir: PathBuf::from("/repository/.git"), ids: vec![format!("request-{number}")], stream: service })
            .expect("queued");
    }
    drop(sender);
    copy_loop(&receiver, &runner, &busy);
    assert_eq!(runner.runs.load(Ordering::SeqCst), 0, "nothing is pending, so nothing runs");
    for (number, client) in clients.into_iter().enumerate() {
        let mut text: String = String::new();
        std::io::BufRead::read_line(&mut std::io::BufReader::new(client), &mut text).expect("a reply");
        let reply = decode_copy_reply(text.trim_end()).expect("well formed");
        // The sham runner finishes nothing and the disk knows none of these identities: an unknown request is a failure.
        assert!(
            matches!(&reply, CopyReply::Done { id, outcome: CopyOutcome::Failed(failure) }
                if *id == format!("request-{number}") && failure.contains("no record")),
            "{reply:?}"
        );
    }
}

/// The copy modules read no process table: nothing about a copy depends on `/proc`, on a process being alive, or on a
///  file being held open, so there is no host, namespace or platform for which the evidence is missing.
#[test]
fn the_copy_modules_do_not_consult_the_process_table() {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/native");
    for name in [
        "service_copy.rs",
        "service_copy_client.rs",
        "worktree_copy.rs",
        "worktree_copy_recovery.rs",
        "worktree_copy_registry.rs",
        "worktree_copy_request.rs",
        "worktree_copy_transaction.rs",
    ] {
        let text: String = std::fs::read_to_string(directory.join(name)).expect("source");
        for forbidden in ["/proc", "process_use", "pid_namespace", "FOREIGN_OWNER_PATIENCE", "hard_link(", "PR_GET_DUMPABLE"] {
            // The test names the strings it looks for, so this file is not scanned; the others must not contain them.
            assert!(!text.contains(forbidden), "{name} contains {forbidden}");
        }
    }
}

//endregion

//region The ordinary copy, and what a retry sees

/// A request is run: the files are installed, the registration is marked, the request and its journal and stage are gone.
#[test]
fn a_request_is_run_and_leaves_only_the_marker() {
    let scene: Scene = Scene::new("copy-ordinary");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let events: Vec<CopyEvent> = scene.drain();
    assert_eq!(Scene::outcomes(&events), vec![(request.id.clone(), CopyOutcome::Copied(COPIED_ENTRIES))]);
    assert!(matches!(events.first(), Some(CopyEvent::Started { .. })));
    assert_eq!(cache_tree(topic.as_path()), COPIED_TREE);
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    assert!(copied_for(scene.admin_root().as_path(), "topic", request.registration.as_str()).expect("marker"));
    remove(scene.root.as_path());
}

/// A request for a worktree the marker already covers runs nothing, even after the user edited a copied file.
#[test]
fn a_request_for_a_marked_registration_runs_nothing() {
    let scene: Scene = Scene::new("copy-marked");
    let topic: PathBuf = scene.add_worktree("topic");
    scene.request(topic.as_path());
    scene.drain();
    // An editor saves by writing a new file, so the edited file is no longer the one hard-linked from the stage.
    std::fs::remove_file(topic.join("cache/top")).expect("replaced");
    std::fs::write(topic.join("cache/top"), b"edited by the user").expect("edit");
    let again: CopyRequest = scene.request(topic.as_path());
    let events: Vec<CopyEvent> = scene.drain();
    assert_eq!(Scene::outcomes(&events), vec![(again.id, CopyOutcome::AlreadyCopied)]);
    assert!(!events.iter().any(|event| matches!(event, CopyEvent::Started { .. })), "{events:?}");
    assert_eq!(std::fs::read(topic.join("cache/top")).expect("top"), b"edited by the user");
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

//endregion

//region Finding 3: a registration is told from a later one of the same name

/// A worktree removed and added again under the same name is another registration: the earlier registration's marker is
///  gone with its directory, and a late marker written for the earlier one marks nothing of the new one.
#[test]
fn a_name_reused_by_a_later_registration_is_copied_again() {
    let scene: Scene = Scene::new("copy-name-reused");
    let topic: PathBuf = scene.add_worktree("topic");
    let first: CopyRequest = scene.request(topic.as_path());
    scene.drain();
    scene.remove_worktree(topic.as_path());
    let again: PathBuf = scene.add_worktree("topic");
    assert_eq!(again, topic, "Git reuses the name and the path");
    assert!(!scene.marker("topic").exists(), "the marker went with the registration");
    // The copy of the earlier registration writes its marker late; the marker carries the earlier identity.
    write_copied_marker(scene.admin_root().as_path(), "topic", first.registration.as_str()).expect("late marker");
    let second: CopyRequest = scene.request(topic.as_path());
    assert_ne!(first.registration, second.registration);
    let events: Vec<CopyEvent> = scene.drain();
    assert_eq!(Scene::outcomes(&events), vec![(second.id.clone(), CopyOutcome::Copied(COPIED_ENTRIES))]);
    assert_eq!(cache_tree(topic.as_path()), COPIED_TREE);
    assert!(copied_for(scene.admin_root().as_path(), "topic", second.registration.as_str()).expect("marker"));
    remove(scene.root.as_path());
}

/// A request written for a registration that was replaced creates nothing in its successor.
#[test]
fn a_request_for_a_replaced_registration_creates_nothing() {
    let scene: Scene = Scene::new("copy-replaced");
    let topic: PathBuf = scene.add_worktree("topic");
    let stale: CopyRequest = scene.request(topic.as_path());
    scene.remove_worktree(topic.as_path());
    scene.add_worktree("topic");
    let events: Vec<CopyEvent> = scene.drain();
    assert_eq!(Scene::outcomes(&events), vec![(stale.id, CopyOutcome::Removed)]);
    assert!(!topic.join("cache").exists(), "the successor was filled by its predecessor's request");
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// A worktree replaced by another at the same path and name while its stage is built is not filled: the registration is
///  checked by identity after staging, and the path alone would have said registered.
#[test]
fn a_worktree_replaced_while_staging_is_not_filled() {
    let scene: Scene = Scene::new("copy-replaced-staging");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let (drained, events) = scene.drain_at(&mut |_request, step| {
        if step == DrainStep::Staged {
            scene.remove_worktree(topic.as_path());
            scene.add_worktree("topic");
        }
        return true;
    });
    assert_eq!(drained.expect("drained"), Drained::Complete);
    assert_eq!(Scene::outcomes(&events), vec![(request.id, CopyOutcome::Removed)]);
    assert!(!topic.join("cache").exists(), "the stage of the old registration was installed into the new one");
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// A worktree removed while its stage is built is not filled and not brought back.
#[test]
fn a_worktree_removed_while_staging_is_not_filled() {
    let scene: Scene = Scene::new("copy-removed-staging");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let (drained, events) = scene.drain_at(&mut |_request, step| {
        if step == DrainStep::Staged {
            scene.remove_worktree(topic.as_path());
        }
        return true;
    });
    assert_eq!(drained.expect("drained"), Drained::Complete);
    assert_eq!(Scene::outcomes(&events), vec![(request.id, CopyOutcome::Removed)]);
    assert!(!topic.exists(), "the removed worktree was brought back");
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// A worktree removed before its request runs is never staged for.
#[test]
fn a_worktree_removed_before_its_request_runs_is_never_staged() {
    let scene: Scene = Scene::new("copy-removed-before");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    scene.remove_worktree(topic.as_path());
    let mut reached: Vec<DrainStep> = Vec::new();
    let (drained, events) = scene.drain_at(&mut |_request, step| {
        reached.push(step);
        return true;
    });
    assert_eq!(drained.expect("drained"), Drained::Complete);
    assert_eq!(Scene::outcomes(&events), vec![(request.id, CopyOutcome::Removed)]);
    assert!(reached.is_empty(), "{reached:?}");
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

//endregion

//region Finding 4: a worktree that is being filled is not copied from before it is filled

/// Requests run in the order they were written, except that a worktree another request copies from is filled first.
#[test]
fn a_source_is_filled_before_a_request_copies_from_it() {
    let make = |id: &str, destination: &str, source: &str| -> CopyRequest {
        return CopyRequest {
            id: id.to_string(),
            admin_id: destination.to_string(),
            registration: String::from("token"),
            destination_root: PathBuf::from(format!("/worktrees/{destination}")),
            source_root: PathBuf::from(format!("/worktrees/{source}")),
            selection: CopySelection::Ignored,
            tries: 0,
            identity: String::from("1:1:-"),
            allow_locked: false,
        };
    };
    let ordered: Vec<String> = order_requests(vec![
        make("1", "u", "t"),
        make("2", "v", "main"),
        make("3", "t", "main"),
        make("4", "w", "u"),
    ])
    .into_iter()
    .map(|request| return request.id)
    .collect();
    assert_eq!(ordered, vec!["2", "3", "1", "4"]);
    // A cycle cannot be ordered and runs in the order written.
    let cycle: Vec<String> = order_requests(vec![make("1", "a", "b"), make("2", "b", "a")])
        .into_iter()
        .map(|request| return request.id)
        .collect();
    assert_eq!(cycle, vec!["1", "2"]);
}

/// A worktree created from another that is still waiting for its own copy gets the other's complete copy, not the empty
///  directory it would have copied from had the requests run as they arrived.
#[test]
fn a_destination_used_as_a_source_is_copied_from_after_it_is_filled() {
    let scene: Scene = Scene::new("copy-destination-source");
    let source_worktree: PathBuf = scene.add_worktree("first");
    let derived: PathBuf = scene.add_worktree("second");
    // The derived worktree's request was written first and names the first worktree, which is still empty, as its source.
    let from_first: CopyRequest = scene.request_from(derived.as_path(), source_worktree.as_path());
    let fills_first: CopyRequest = scene.request(source_worktree.as_path());
    let events: Vec<CopyEvent> = scene.drain();
    assert_eq!(
        Scene::outcomes(&events),
        vec![
            (fills_first.id, CopyOutcome::Copied(COPIED_ENTRIES)),
            (from_first.id, CopyOutcome::Copied(COPIED_ENTRIES)),
        ]
    );
    assert_eq!(cache_tree(derived.as_path()), COPIED_TREE);
    assert_eq!(std::fs::read(derived.join("cache/deep/data")).expect("data"), b"payload");
    remove(scene.root.as_path());
}

//endregion

//region Finding 6: a finished copy is never forgotten, and a copy killed at any durable point is finished

/// The state at one durable point of a copy: stop the service there, as a kill would, and drain again.
fn kill_at_and_recover(name: &str, point: DrainStep) -> (Scene, PathBuf, CopyRequest, Vec<CopyEvent>) {
    let scene: Scene = Scene::new(name);
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let (drained, _events) = scene.drain_at(&mut |_request, step| return step != point);
    assert_eq!(drained.expect("stopped"), Drained::Interrupted);
    let events: Vec<CopyEvent> = scene.drain();
    return (scene, topic, request, events);
}

/// Killed before the stage exists: the next run does the whole copy.
#[test]
fn a_service_killed_before_staging_copies_on_the_next_run() {
    let (scene, topic, request, events) = kill_at_and_recover("copy-kill-staging", DrainStep::Staging);
    assert_eq!(Scene::outcomes(&events), vec![(request.id, CopyOutcome::Copied(COPIED_ENTRIES))]);
    assert_eq!(cache_tree(topic.as_path()), COPIED_TREE);
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// Killed while staging, with a half-filled stage beside the destination: the next run removes that stage and starts over.
#[test]
fn a_service_killed_while_staging_starts_over_without_the_half_stage() {
    let scene: Scene = Scene::new("copy-kill-half-stage");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let half: PathBuf = request.stage_container();
    crate::private_storage::create_private_directory(half.as_path()).expect("stage");
    std::fs::write(half.join("half"), b"copied so far").expect("partial copy");
    let events: Vec<CopyEvent> = scene.drain();
    assert_eq!(Scene::outcomes(&events), vec![(request.id, CopyOutcome::Copied(COPIED_ENTRIES))]);
    assert!(!half.exists());
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// Killed with the stage journaled and nothing installed: recovery installs it, marks the registration, and the request
///  that is still on disk finds the copy done.
#[test]
fn a_service_killed_with_the_stage_journaled_finishes_on_the_next_run() {
    let (scene, topic, request, events) = kill_at_and_recover("copy-kill-staged", DrainStep::Staged);
    assert_eq!(Scene::outcomes(&events), vec![(request.id, CopyOutcome::AlreadyCopied)]);
    assert!(events.iter().any(|event| matches!(event, CopyEvent::Notice(text) if text.contains("recovered ignored-state copies"))));
    assert_eq!(cache_tree(topic.as_path()), COPIED_TREE);
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// Killed after the files are installed and before anything records that the copy is over: the registration is not marked
///  yet, so recovery resumes the installation, finds every path in place, and marks it. The copy is not forgotten and
///  not made twice.
#[test]
fn a_service_killed_between_the_install_and_the_marker_does_not_forget_the_copy() {
    let scene: Scene = Scene::new("copy-kill-installed");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let (drained, _events) = scene.drain_at(&mut |_request, step| return step != DrainStep::Installed);
    assert_eq!(drained.expect("stopped"), Drained::Interrupted);
    assert_eq!(cache_tree(topic.as_path()), COPIED_TREE, "the files are installed");
    assert!(!scene.marker("topic").exists(), "nothing records that the copy is over");
    assert_eq!(read_pending_journals(scene.common.as_path()).expect("pending").len(), 1);
    let events: Vec<CopyEvent> = scene.drain();
    assert_eq!(Scene::outcomes(&events), vec![(request.id, CopyOutcome::AlreadyCopied)]);
    assert!(copied_for(scene.admin_root().as_path(), "topic", request.registration.as_str()).expect("marker"));
    assert_eq!(cache_tree(topic.as_path()), COPIED_TREE);
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// Killed after the marker is durable and before the journal is removed: recovery treats the marker as done and only
///  cleans up, so a file the user edited in between is not touched and no failure appears.
#[test]
fn a_service_killed_after_the_marker_only_cleans_up() {
    let scene: Scene = Scene::new("copy-kill-marked");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let (drained, _events) = scene.drain_at(&mut |_request, step| return step != DrainStep::Marked);
    assert_eq!(drained.expect("stopped"), Drained::Interrupted);
    assert!(scene.marker("topic").exists());
    assert_eq!(read_pending_journals(scene.common.as_path()).expect("pending").len(), 1);
    // An editor saves by writing a new file, so the edited file is no longer the one hard-linked from the stage.
    std::fs::remove_file(topic.join("cache/top")).expect("replaced");
    std::fs::write(topic.join("cache/top"), b"edited by the user").expect("edit");
    let events: Vec<CopyEvent> = scene.drain();
    assert_eq!(Scene::outcomes(&events), vec![(request.id, CopyOutcome::AlreadyCopied)]);
    assert!(!events.iter().any(|event| matches!(event, CopyEvent::Notice(text) if text.contains("without finishing"))), "{events:?}");
    assert_eq!(std::fs::read(topic.join("cache/top")).expect("top"), b"edited by the user");
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

//endregion

//region Finding 9: any worktree name works

/// A worktree named like what an adoption key once looked like is copied, marked, and recovered like any other.
#[test]
fn a_worktree_named_journal_is_copied_and_marked() {
    let scene: Scene = Scene::new("copy-journal-name");
    let topic: PathBuf = scene.add_worktree("journal-topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let (drained, _events) = scene.drain_at(&mut |_request, step| return step != DrainStep::Marked);
    assert_eq!(drained.expect("stopped"), Drained::Interrupted);
    let events: Vec<CopyEvent> = scene.drain();
    assert_eq!(Scene::outcomes(&events), vec![(request.id.clone(), CopyOutcome::AlreadyCopied)]);
    assert!(copied_for(scene.admin_root().as_path(), "journal-topic", request.registration.as_str()).expect("marker"));
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

//endregion

//region Finding 10: a copy that failed or never got started is tried again

/// A copy that fails stays as a request and is tried again on the next run; once the cause is gone it succeeds. After the
///  allowed number of tries it is set aside with a notice instead of running on every command.
#[test]
fn a_failed_copy_is_tried_again_and_then_given_up_on() {
    let scene: Scene = Scene::new("copy-retry");
    let topic: PathBuf = scene.add_worktree("topic");
    // A different file where the copy would put one makes the installation refuse to overwrite it.
    std::fs::create_dir_all(topic.join("cache")).expect("cache");
    std::fs::write(topic.join("cache/top"), b"differs").expect("differing entry");
    let request: CopyRequest = scene.request(topic.as_path());
    let first: Vec<(String, CopyOutcome)> = Scene::outcomes(&scene.drain());
    assert!(matches!(first.as_slice(), [(id, CopyOutcome::Failed(text))] if *id == request.id && text.contains("would overwrite")), "{first:?}");
    assert!(!scene.marker("topic").exists(), "a failed copy is not marked");
    let kept: Vec<CopyRequest> = read_requests(scene.common.as_path()).expect("requests").0;
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].tries, 1);
    // The cause goes away; the next run copies.
    std::fs::remove_file(topic.join("cache/top")).expect("cause removed");
    let second: Vec<(String, CopyOutcome)> = Scene::outcomes(&scene.drain());
    assert_eq!(second, vec![(request.id.clone(), CopyOutcome::Copied(COPIED_ENTRIES - 1))]);
    assert_eq!(scene.anything_left(), Vec::<String>::new());
    remove(scene.root.as_path());
}

/// A copy that keeps failing is given up on after `MAX_TRIES` runs, with a request kept for the person.
#[test]
fn a_copy_that_keeps_failing_is_given_up_on() {
    let scene: Scene = Scene::new("copy-give-up");
    let topic: PathBuf = scene.add_worktree("topic");
    std::fs::create_dir_all(topic.join("cache")).expect("cache");
    std::fs::write(topic.join("cache/top"), b"differs").expect("differing entry");
    let request: CopyRequest = scene.request(topic.as_path());
    let mut last: Vec<(String, CopyOutcome)> = Vec::new();
    for _run in 0..MAX_TRIES {
        last = Scene::outcomes(&scene.drain());
    }
    assert!(matches!(last.as_slice(), [(_, CopyOutcome::Failed(text))] if text.contains("given up")), "{last:?}");
    assert!(!requests_exist(scene.common.as_path()).expect("requests"));
    let kept: PathBuf = scene.common.join("cli-git-worktree-copy/v1").join(GIVEN_UP_NAME).join(format!("{}.json", request.id));
    assert!(kept.exists());
    // Nothing runs it again.
    assert!(Scene::outcomes(&scene.drain()).is_empty());
    remove(scene.root.as_path());
}

/// A request file that does not parse is set aside with a notice and does not stop the others.
#[test]
fn a_damaged_request_is_set_aside_and_the_others_run() {
    let scene: Scene = Scene::new("copy-damaged");
    let topic: PathBuf = scene.add_worktree("topic");
    let good: CopyRequest = scene.request(topic.as_path());
    let directory: PathBuf = requests_directory(scene.common.as_path());
    std::fs::write(directory.join("0000-damaged.json"), b"not json").expect("damaged");
    let events: Vec<CopyEvent> = scene.drain();
    assert_eq!(Scene::outcomes(&events), vec![(good.id, CopyOutcome::Copied(COPIED_ENTRIES))]);
    assert!(events.iter().any(|event| matches!(event, CopyEvent::Notice(text) if text.contains("set aside the copy request"))));
    assert!(scene.common.join("cli-git-worktree-copy/v1").join(GIVEN_UP_NAME).join("0000-damaged.json").exists());
    remove(scene.root.as_path());
}

//endregion

//region The wire and the service around it

/// Every reply survives its own line, and a damaged line is refused.
#[test]
fn replies_round_trip_and_damaged_lines_are_refused() {
    for reply in [
        CopyReply::Started { id: String::from("a-1"), destination: String::from("/worktrees/topic") },
        CopyReply::Notice(String::from("cli-git: a notice.")),
        CopyReply::Finished,
        CopyReply::Done { id: String::from("a-1"), outcome: CopyOutcome::Copied(7) },
        CopyReply::Done { id: String::from("a-1"), outcome: CopyOutcome::AlreadyCopied },
        CopyReply::Done { id: String::from("a-1"), outcome: CopyOutcome::Removed },
        CopyReply::Done { id: String::from("a-1"), outcome: CopyOutcome::Waiting(String::from("cli-git: it waits.")) },
        CopyReply::Done { id: String::from("a-1"), outcome: CopyOutcome::Failed(String::from("cli-git: it failed.")) },
    ] {
        assert_eq!(decode_copy_reply(encode_copy_reply(&reply).as_str()).expect("round trip"), reply);
    }
    for damaged in ["", "{", "{\"type\":\"copy-done\"}", "{\"type\":\"copy-done\",\"id\":\"a\",\"outcome\":\"copied\"}", "{\"type\":\"nope\"}"] {
        assert!(decode_copy_reply(damaged).is_err(), "{damaged}");
    }
}

/// The fault point a step names is the one the service tests arm.
#[test]
fn every_step_names_its_fault_point() {
    let points: Vec<&str> = [DrainStep::Staging, DrainStep::Staged, DrainStep::Installed, DrainStep::Marked]
        .into_iter()
        .map(DrainStep::fault_point)
        .collect();
    assert_eq!(points, vec!["copy-staging", "copy-staged", "copy-installed", "copy-marked"]);
}

//endregion

//region The second review

/// What a restarted service does for a client: no drain of its own, so only the disk can answer.
struct NoRunner;

impl CopyRunner for NoRunner {
    fn drain(&self, _common_dir: &Path, _events: &mut dyn FnMut(CopyEvent)) -> Result<(), String> {
        return Ok(());
    }
}

/// Asks a service with no memory about requests, and returns every line it answers.
fn ask_restarted_service(common_dir: &Path, ids: &[String]) -> Vec<CopyReply> {
    let busy: AtomicUsize = AtomicUsize::new(0);
    let (sender, receiver) = std::sync::mpsc::channel::<CopyJob>();
    let (client, service) = UnixStream::pair().expect("pair");
    sender.send(CopyJob { common_dir: common_dir.to_path_buf(), ids: ids.to_vec(), stream: service }).expect("queued");
    drop(sender);
    copy_loop(&receiver, &NoRunner, &busy);
    let mut replies: Vec<CopyReply> = Vec::new();
    for line in std::io::BufRead::lines(std::io::BufReader::new(client)) {
        replies.push(decode_copy_reply(line.expect("a line").as_str()).expect("well formed"));
    }
    return replies;
}

/// The worktree has a differing file where the copy would put one, so the installation refuses and rolls back.
fn make_the_copy_fail(destination: &Path) {
    std::fs::create_dir_all(destination.join("cache")).expect("cache");
    std::fs::write(destination.join("cache/top"), b"differs").expect("differing entry");
}

//region Finding 1: a wrapper killed after Git returns

/// A wrapper killed after Git created the worktree and before it wrote the request leaves only its intent; the service
/// writes the request from it and the worktree is filled.
#[test]
fn a_wrapper_killed_after_git_returns_leaves_the_worktree_to_the_service() {
    let scene: Scene = Scene::new("copy-intent-killed");
    scene.intent(None);
    let topic: PathBuf = scene.add_worktree("topic");
    assert!(!requests_exist(scene.common.as_path()).expect("requests"), "the wrapper was killed before it wrote one");
    let events: Vec<CopyEvent> = scene.drain();
    let outcomes: Vec<(String, CopyOutcome)> = Scene::outcomes(&events);
    assert_eq!(outcomes.len(), 1, "{outcomes:?}");
    assert_eq!(outcomes[0].1, CopyOutcome::Copied(COPIED_ENTRIES));
    assert_eq!(cache_tree(topic.as_path()), COPIED_TREE);
    remove(scene.root.as_path());
}

//endregion

//region Finding 3: readiness, and a replaced directory

/// A worktree that Git still has locked is another command's to finish creating: its request waits, with no try counted and
/// nothing copied, until the lock is gone. A lock its creator asked for does not make the request wait.
#[test]
fn a_locked_worktree_waits_and_the_creators_own_lock_does_not() {
    let scene: Scene = Scene::new("copy-locked");
    let waiting: PathBuf = scene.add_worktree("waiting");
    let request: CopyRequest = scene.request(waiting.as_path());
    scene.set_lock(waiting.as_path(), true);
    let first: Vec<(String, CopyOutcome)> = Scene::outcomes(&scene.drain());
    assert!(matches!(first.as_slice(), [(id, CopyOutcome::Waiting(text))] if *id == request.id && text.contains("locked")), "{first:?}");
    assert!(!waiting.join("cache").exists(), "a locked worktree was filled");
    let kept: Vec<CopyRequest> = read_requests(scene.common.as_path()).expect("requests").0;
    assert_eq!((kept.len(), kept[0].tries), (1, 0), "waiting counts no try");
    scene.set_lock(waiting.as_path(), false);
    let second: Vec<(String, CopyOutcome)> = Scene::outcomes(&scene.drain());
    assert_eq!(second, vec![(request.id.clone(), CopyOutcome::Copied(COPIED_ENTRIES))]);
    let own: PathBuf = scene.add_locked_worktree("own");
    let own_request: CopyRequest = scene.request(own.as_path());
    scene.rewrite(&own_request, |changed| changed.allow_locked = true);
    let third: Vec<(String, CopyOutcome)> = Scene::outcomes(&scene.drain());
    assert_eq!(third, vec![(own_request.id, CopyOutcome::Copied(COPIED_ENTRIES))]);
    remove(scene.root.as_path());
}

/// A request whose recorded directory identity is not the directory now at the name is refused, though the name, the
/// token and the pointer file all still agree: they are exactly what a replacement under the same name would present.
#[test]
fn a_request_for_a_directory_that_is_not_the_observed_one_is_refused() {
    let scene: Scene = Scene::new("copy-identity");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    scene.rewrite(&request, |changed| changed.identity = String::from("9:9:-"));
    let outcomes: Vec<(String, CopyOutcome)> = Scene::outcomes(&scene.drain());
    assert_eq!(outcomes, vec![(request.id, CopyOutcome::Removed)]);
    assert!(!topic.join("cache").exists());
    remove(scene.root.as_path());
}

//endregion

//region Finding 4: a destination is a source only once it is filled

/// A request that copies from a worktree whose own copy failed waits for it instead of copying an empty directory, with no
/// try counted; once the prerequisite is filled the next drain fills both.
#[test]
fn a_dependant_of_a_failed_prerequisite_waits_for_it() {
    let scene: Scene = Scene::new("copy-dependant-failed");
    let first: PathBuf = scene.add_worktree("first");
    let second: PathBuf = scene.add_worktree("second");
    make_the_copy_fail(first.as_path());
    let prerequisite: CopyRequest = scene.request(first.as_path());
    let dependant: CopyRequest = scene.request_from(second.as_path(), first.as_path());
    let outcomes: Vec<(String, CopyOutcome)> = Scene::outcomes(&scene.drain());
    assert_eq!(outcomes.len(), 2, "{outcomes:?}");
    assert_eq!(outcomes[0].0, prerequisite.id);
    assert!(matches!(&outcomes[0].1, CopyOutcome::Failed(_)), "{outcomes:?}");
    assert_eq!(outcomes[1].0, dependant.id);
    assert!(matches!(&outcomes[1].1, CopyOutcome::Waiting(text) if text.contains("not been filled")), "{outcomes:?}");
    assert!(!second.join("cache").exists(), "an empty source was copied");
    let kept: Vec<CopyRequest> = read_requests(scene.common.as_path()).expect("requests").0;
    let waiting: &CopyRequest = kept.iter().find(|request| return request.id == dependant.id).expect("still pending");
    assert_eq!(waiting.tries, 0, "waiting counts no try");
    std::fs::remove_file(first.join("cache/top")).expect("cause removed");
    let later: Vec<(String, CopyOutcome)> = Scene::outcomes(&scene.drain());
    assert!(matches!(later.as_slice(), [(a, CopyOutcome::Copied(_)), (b, CopyOutcome::Copied(_))] if *a == prerequisite.id && *b == dependant.id), "{later:?}");
    assert_eq!(cache_tree(second.as_path()), COPIED_TREE);
    remove(scene.root.as_path());
}

/// A request that copies from a worktree whose copy was given up on, and which is still not filled, is given up on with
/// that reason; it does not wait for ever and does not copy an empty directory.
#[test]
fn a_dependant_of_a_given_up_prerequisite_is_given_up() {
    let scene: Scene = Scene::new("copy-dependant-given-up");
    let first: PathBuf = scene.add_worktree("first");
    let second: PathBuf = scene.add_worktree("second");
    make_the_copy_fail(first.as_path());
    let prerequisite: CopyRequest = scene.request(first.as_path());
    scene.rewrite(&prerequisite, |changed| changed.tries = MAX_TRIES - 1);
    let dependant: CopyRequest = scene.request_from(second.as_path(), first.as_path());
    let first_drain: Vec<(String, CopyOutcome)> = Scene::outcomes(&scene.drain());
    assert!(matches!(&first_drain[0].1, CopyOutcome::Failed(text) if text.contains("given up")), "{first_drain:?}");
    assert!(matches!(&first_drain[1].1, CopyOutcome::Waiting(_)), "{first_drain:?}");
    let second_drain: Vec<(String, CopyOutcome)> = Scene::outcomes(&scene.drain());
    assert!(
        matches!(second_drain.as_slice(), [(id, CopyOutcome::Failed(text))] if *id == dependant.id && text.contains("given up") && text.contains("whose own copy was given up")),
        "{second_drain:?}"
    );
    assert!(matches!(terminal_of(scene.common.as_path(), dependant.id.as_str()), Ok(Terminal::GivenUp(_))));
    assert!(!second.join("cache").exists());
    remove(scene.root.as_path());
}

/// A prerequisite whose request is written while the drain runs is ordered before the request that copies from it, because
/// the drain reads the requests again after each run.
#[test]
fn a_prerequisite_published_during_the_drain_runs_first() {
    let scene: Scene = Scene::new("copy-dependant-published");
    let unrelated: PathBuf = scene.add_worktree("unrelated");
    let prerequisite: PathBuf = scene.add_worktree("prerequisite");
    let dependant: PathBuf = scene.add_worktree("dependant");
    let first: CopyRequest = scene.request(unrelated.as_path());
    let waiting: CopyRequest = scene.request_from(dependant.as_path(), prerequisite.as_path());
    let mut published: Option<CopyRequest> = None;
    let (drained, events) = scene.drain_at(&mut |request, step| {
        // While the first copy runs, another wrapper's request for the dependant's source is written.
        if step == DrainStep::Staging && request.id == first.id {
            published = Some(scene.request(prerequisite.as_path()));
        }
        return true;
    });
    assert_eq!(drained.expect("drained"), Drained::Complete);
    let order: Vec<String> = Scene::outcomes(&events).into_iter().map(|(id, _)| return id).collect();
    let prerequisite_id: String = published.expect("published during the drain").id;
    assert_eq!(order, vec![first.id, prerequisite_id, waiting.id]);
    assert_eq!(cache_tree(dependant.as_path()), COPIED_TREE, "the dependant copied from an unfilled source");
    remove(scene.root.as_path());
}

//endregion

//region Finding 5: a terminal outcome is never lost to a restart

/// A request given up on, then the service killed before it told anybody: the restarted service answers a client from the
/// durable record, and the answer is a failure.
#[test]
fn a_request_given_up_before_a_kill_is_answered_as_a_failure_after_the_restart() {
    let scene: Scene = Scene::new("copy-given-up-restart");
    let topic: PathBuf = scene.add_worktree("topic");
    make_the_copy_fail(topic.as_path());
    let request: CopyRequest = scene.request(topic.as_path());
    scene.rewrite(&request, |changed| changed.tries = MAX_TRIES - 1);
    scene.drain();
    assert!(matches!(terminal_of(scene.common.as_path(), request.id.as_str()), Ok(Terminal::GivenUp(_))));
    let replies: Vec<CopyReply> = ask_restarted_service(scene.common.as_path(), std::slice::from_ref(&request.id));
    assert!(
        matches!(replies.first(), Some(CopyReply::Done { id, outcome: CopyOutcome::Failed(text) }) if *id == request.id && text.contains("given up")),
        "{replies:?}"
    );
    assert_eq!(replies.last(), Some(&CopyReply::Finished));
    remove(scene.root.as_path());
}

/// A request that ended well is answered after a restart from its record, with the count it had.
#[test]
fn a_finished_request_is_answered_from_its_record_after_a_restart() {
    let scene: Scene = Scene::new("copy-done-restart");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    scene.drain();
    let replies: Vec<CopyReply> = ask_restarted_service(scene.common.as_path(), std::slice::from_ref(&request.id));
    assert_eq!(
        replies.first(),
        Some(&CopyReply::Done { id: request.id.clone(), outcome: CopyOutcome::Copied(COPIED_ENTRIES) })
    );
    remove(scene.root.as_path());
}

/// A pending request that a service has not run is not reported as anything it is not: the client is told it waits.
#[test]
fn a_pending_request_the_service_has_not_run_is_reported_as_waiting() {
    let scene: Scene = Scene::new("copy-pending-restart");
    let topic: PathBuf = scene.add_worktree("topic");
    let request: CopyRequest = scene.request(topic.as_path());
    let replies: Vec<CopyReply> = ask_restarted_service(scene.common.as_path(), std::slice::from_ref(&request.id));
    // The sham service has nothing that runs, so the request stays pending and the answer says so.
    assert!(
        matches!(replies.first(), Some(CopyReply::Done { outcome: CopyOutcome::Waiting(text), .. }) if text.contains("has not been run")),
        "{replies:?}"
    );
    remove(scene.root.as_path());
}

/// A runner whose drain reports one request as failed.
struct FailingRunner {
    id: String,
}

impl CopyRunner for FailingRunner {
    fn drain(&self, _common_dir: &Path, events: &mut dyn FnMut(CopyEvent)) -> Result<(), String> {
        events(CopyEvent::Done { id: self.id.clone(), outcome: CopyOutcome::Failed(String::from("cli-git: it failed.")) });
        return Ok(());
    }
}

/// A scene with one pending request, so that a job naming it makes the service run a drain.
fn scene_with_a_pending_request(name: &str) -> (Scene, String) {
    let scene: Scene = Scene::new(name);
    let topic: PathBuf = scene.add_worktree("topic");
    let id: String = scene.request(topic.as_path()).id;
    return (scene, id);
}

/// A failure is a notice on disk before the client is told, a client that is killed before it reads leaves the notice for
/// the next commit, and a client that showed the failure withdraws it.
#[test]
fn a_failure_is_left_as_a_notice_until_the_client_has_shown_it() {
    let (scene, id): (Scene, String) = scene_with_a_pending_request("copy-notice");
    let busy: AtomicUsize = AtomicUsize::new(0);
    let notice: PathBuf = notice_directory(scene.common.as_path()).join(format!("{}.txt", copy_notice_key(id.as_str())));
    let (sender, receiver) = std::sync::mpsc::channel::<CopyJob>();
    let (client, service) = UnixStream::pair().expect("pair");
    // The client reads its line and looks for the notice at once: the notice was there before the line was sent.
    let reader = std::thread::spawn({
        let watched: PathBuf = notice.clone();
        move || {
            let mut line: String = String::new();
            std::io::BufRead::read_line(&mut std::io::BufReader::new(client), &mut line).expect("a reply");
            return (line, watched.exists());
        }
    });
    sender.send(CopyJob { common_dir: scene.common.clone(), ids: vec![id.clone()], stream: service }).expect("queued");
    drop(sender);
    copy_loop(&receiver, &FailingRunner { id: id.clone() }, &busy);
    let (line, existed_when_read): (String, bool) = reader.join().expect("client");
    assert!(line.contains("failed"), "{line}");
    assert!(existed_when_read, "the client was told before the notice was on disk");
    assert_eq!(std::fs::read_to_string(notice.as_path()).expect("notice"), "cli-git: it failed.");
    withdraw_notice(scene.common.as_path(), id.as_str());
    assert!(!notice.exists(), "a shown failure left its notice");
    remove(scene.root.as_path());
}

/// A client that is gone when the failure comes still leaves the notice.
#[test]
fn a_failure_for_a_client_that_is_gone_stays_a_notice() {
    let (scene, id): (Scene, String) = scene_with_a_pending_request("copy-notice-gone");
    let busy: AtomicUsize = AtomicUsize::new(0);
    let (sender, receiver) = std::sync::mpsc::channel::<CopyJob>();
    let (client, service) = UnixStream::pair().expect("pair");
    drop(client);
    sender.send(CopyJob { common_dir: scene.common.clone(), ids: vec![id.clone()], stream: service }).expect("queued");
    drop(sender);
    copy_loop(&receiver, &FailingRunner { id: id.clone() }, &busy);
    let notice: PathBuf = notice_directory(scene.common.as_path()).join(format!("{}.txt", copy_notice_key(id.as_str())));
    assert!(notice.exists());
    remove(scene.root.as_path());
}

//endregion

//region Finding 6: the client is answered when its own requests end

/// A runner that answers one request and then goes on with another until the test lets it end.
struct SlowRunner {
    release: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
}

impl CopyRunner for SlowRunner {
    fn drain(&self, _common_dir: &Path, events: &mut dyn FnMut(CopyEvent)) -> Result<(), String> {
        events(CopyEvent::Done { id: String::from("mine"), outcome: CopyOutcome::Copied(1) });
        // The rest of the drain: another worktree's large copy.
        let _released = self.release.lock().expect("lock").recv();
        return Ok(());
    }
}

/// A client whose requests are all answered is told so at once, while the drain goes on with other worktrees.
#[test]
fn a_client_is_finished_when_its_own_requests_end_not_when_the_drain_does() {
    let scene: Scene = Scene::new("copy-early-finish");
    let busy: AtomicUsize = AtomicUsize::new(0);
    let (release, released) = std::sync::mpsc::channel::<()>();
    let runner: SlowRunner = SlowRunner { release: std::sync::Mutex::new(released) };
    let (sender, receiver) = std::sync::mpsc::channel::<CopyJob>();
    let (client, service) = UnixStream::pair().expect("pair");
    sender.send(CopyJob { common_dir: scene.common.clone(), ids: vec![String::from("mine")], stream: service }).expect("queued");
    drop(sender);
    // A client that is not told in time fails here instead of waiting for ever.
    client.set_read_timeout(Some(std::time::Duration::from_secs(10))).expect("timeout");
    let lines: Vec<String> = std::thread::scope(|threads| {
        threads.spawn(move || copy_loop(&receiver, &runner, &busy));
        let mut reader = std::io::BufReader::new(client);
        let mut seen: Vec<String> = Vec::new();
        for _expected in 0..2 {
            let mut line: String = String::new();
            std::io::BufRead::read_line(&mut reader, &mut line).expect("a reply while the drain still runs");
            seen.push(line);
        }
        // Only now does the rest of the drain end.
        release.send(()).expect("release");
        return seen;
    });
    assert!(lines[0].contains("copy-done"), "{lines:?}");
    assert!(lines[1].contains("copy-finished"), "{lines:?}");
    remove(scene.root.as_path());
}

//endregion

//endregion
