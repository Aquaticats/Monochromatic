//! The service's copies against real Git: every race the settlement lock used to prevent, and every finding of the
//!  review of the claim design, as a scene that fails when the guard that answers it is removed.
//!  Nothing waits for anything: each case ends because the service runs the request, settles the state a kill left,
//!  or refuses a request whose registration is gone.

use super::*;
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_copy_journal::read_pending_journals;
use crate::worktree_copy_registry::{COPIED_MARKER_NAME, copied_for, ensure_registration_token, write_copied_marker};
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

    /// The administrative directory of the worktree at `root`.
    pub(crate) fn admin_root(&self) -> PathBuf {
        return self.common.join("worktrees");
    }

    /// Writes a request for the worktree at `destination`, copied from `source`, with the registration's identity.
    pub(crate) fn request_from(&self, destination: &Path, source: &Path) -> CopyRequest {
        let admin_id: String = destination.file_name().expect("name").to_string_lossy().into_owned();
        let token: String = ensure_registration_token(self.admin_root().as_path(), admin_id.as_str())
            .expect("identity")
            .expect("registered");
        let request: CopyRequest = CopyRequest {
            id: new_request_id().expect("identity"),
            admin_id,
            registration: token,
            destination_root: destination.to_path_buf(),
            source_root: source.to_path_buf(),
            selection: CopySelection::Ignored,
            tries: 0,
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
    for number in 0..4 {
        let (client, service) = UnixStream::pair().expect("pair");
        clients.push(client);
        sender
            .send(CopyJob { common_dir: PathBuf::from("/repository/.git"), ids: vec![format!("request-{number}")], stream: service })
            .expect("queued");
    }
    drop(sender);
    copy_loop(&receiver, &runner, &busy);
    assert_eq!(runner.most.load(Ordering::SeqCst), 1, "two drains ran at the same moment");
    assert!(runner.runs.load(Ordering::SeqCst) >= 1);
    for (number, client) in clients.into_iter().enumerate() {
        let mut text: String = String::new();
        std::io::BufRead::read_line(&mut std::io::BufReader::new(client), &mut text).expect("a reply");
        let reply = decode_copy_reply(text.trim_end()).expect("well formed");
        assert_eq!(reply, CopyReply::Done { id: format!("request-{number}"), outcome: CopyOutcome::Settled });
    }
    assert_eq!(busy.load(Ordering::SeqCst), 0);
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
        CopyReply::Done { id: String::from("a-1"), outcome: CopyOutcome::Settled },
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
