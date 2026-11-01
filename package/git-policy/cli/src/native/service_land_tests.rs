//! Controls for landing a batch with real Git: order and content, the index, conflicting and moving branches, hostile
//!  names, and the restart decisions for batches left open.

use super::*;
use crate::service_journal::{DONE_EXTENSION, batch_path, parse_intent};
use crate::service_protocol::{ALLOWED_MODES, Change};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const COMMITTER: &str = "T <t@example.invalid> 1793535889 +0000";

struct World {
    root: PathBuf,
    repo: PathBuf,
    context: LandContext,
}

fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("journal");
    let context: LandContext = LandContext { git: git_context(), journal, on_objects_written: None };
    return World { root, repo, context };
}

/// A request that sets `path` to `content`, prepared against the branch as it is now.
fn request(world: &World, id: &str, path: &str, content: &str) -> CommitRequest {
    let file: PathBuf = world.repo.join(path);
    std::fs::create_dir_all(file.parent().expect("parent")).expect("directories");
    std::fs::write(&file, content).expect("write");
    let blob: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", path]);
    return CommitRequest {
        id: String::from(id),
        repository: world.repo.clone(),
        common_dir: world.repo.join(".git"),
        reference: String::from("refs/heads/main"),
        base: git_text(world.repo.as_path(), &["rev-parse", "HEAD"]),
        changes: vec![Change { path: String::from(path), mode: ALLOWED_MODES[0], blob: Some(blob) }],
        message: format!("message of {id}\n"),
        author: None,
        committer: String::from(COMMITTER),
        amend: false,
        tree: None,
    };
}

fn subjects(world: &World, since: &str) -> Vec<String> {
    return git_text(world.repo.as_path(), &["log", "--reverse", "--format=%s", format!("{since}..HEAD").as_str()])
        .lines()
        .map(str::to_string)
        .collect();
}

/// A batch becomes one commit per request, in request order, each holding its content, the branch ends at the last,
///  and the index shows nothing staged against it.
#[test]
fn a_batch_lands_in_order_with_its_content_and_a_clean_index() {
    let world: World = world("service-land-order");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let batch: Vec<CommitRequest> = ["one", "two", "three"].iter().map(|id| { return request(&world, id, format!("{id}.txt").as_str(), format!("{id}\n").as_str()); }).collect();
    let outcomes = land_batch(&world.context, 1, &batch).expect("landed");
    assert_eq!(subjects(&world, base.as_str()), vec!["message of one", "message of two", "message of three"]);
    for (id, outcome) in ["one", "two", "three"].iter().zip(&outcomes) {
        let Outcome::Landed { commit } = outcome else { panic!("{outcome:?}") };
        assert_eq!(git_text(world.repo.as_path(), &["show", format!("{commit}:{id}.txt").as_str()]), *id);
    }
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), match outcomes.last() { Some(Outcome::Landed { commit }) => commit.clone(), other => panic!("{other:?}") });
    assert!(batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION).exists(), "the batch is journaled and finished");
    remove(world.root.as_path());
}

/// A path that changed on the branch since the request's base sends the request back; an unrelated path still lands.
#[test]
fn a_changed_path_is_sent_back_and_an_unrelated_one_lands() {
    let world: World = world("service-land-conflict");
    let stale: CommitRequest = request(&world, "stale", "shared.txt", "mine\n");
    let unrelated: CommitRequest = request(&world, "unrelated", "other.txt", "other\n");
    git(world.repo.as_path(), &["add", "shared.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=someone else"]);
    let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let outcomes = land_batch(&world.context, 1, &[stale, unrelated]).expect("landed");
    assert_eq!(outcomes[0], Outcome::RetryPrepare { base: head });
    assert!(matches!(outcomes[1], Outcome::Landed { .. }), "{:?}", outcomes[1]);
    remove(world.root.as_path());
}

/// A directory and a file of the same name overlap, so a request for one is sent back when the other changed.
#[test]
fn a_file_and_a_directory_of_one_name_overlap() {
    assert!(overlaps("a", "a/b"));
    assert!(overlaps("a/b", "a"));
    assert!(overlaps("a/b", "a/b"));
    assert!(!overlaps("a/b", "a/bc"));
    assert!(!overlaps("ab", "a"));
}

/// Names that need quoting in the import stream land as themselves.
#[test]
fn awkward_names_land_as_themselves() {
    let world: World = world("service-land-names");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let names = ["a \"quoted\" name.txt", "back\\slash.txt", "\"leading-quote.txt", "sp ace/dir with space/f.txt", "ünï.txt"];
    let batch: Vec<CommitRequest> = names.iter().enumerate().map(|(n, name)| { return request(&world, format!("n{n}").as_str(), name, "x\n"); }).collect();
    land_batch(&world.context, 1, &batch).expect("landed");
    let listing: String = git_text(world.repo.as_path(), &["ls-tree", "-r", "--name-only", "-z", "HEAD"]);
    let _ = base;
    for name in names {
        assert!(listing.split('\0').any(|entry| { return entry == name; }), "{name:?} in {listing:?}");
    }
    remove(world.root.as_path());
}

/// A deletion removes the path from the tree.
#[test]
fn a_deletion_removes_the_path() {
    let world: World = world("service-land-delete");
    let add: CommitRequest = request(&world, "add", "gone.txt", "x\n");
    land_batch(&world.context, 1, &[add]).expect("added");
    let mut delete: CommitRequest = request(&world, "delete", "stays.txt", "y\n");
    delete.changes = vec![Change { path: String::from("gone.txt"), mode: ALLOWED_MODES[0], blob: None }];
    land_batch(&world.context, 2, &[delete]).expect("deleted");
    assert_eq!(git_text(world.repo.as_path(), &["ls-tree", "-r", "--name-only", "HEAD"]).lines().filter(|name| { return *name == "gone.txt"; }).count(), 0);
    remove(world.root.as_path());
}

/// The branch moving under a batch is retried: a plain commit between the objects and the branch update makes the first
///  attempt lose, and the next one lands on top of the plain commit.
#[test]
fn a_branch_that_moves_under_the_batch_is_built_again() {
    let mut world: World = world("service-land-race");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let first: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let repo: PathBuf = world.repo.clone();
    let moves: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let counter: Arc<AtomicUsize> = moves.clone();
    world.context.on_objects_written = Some(Box::new(move || {
        if counter.fetch_add(1, Ordering::SeqCst) == 0 {
            git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=moved by someone"]);
        }
    }));
    let outcomes = land_batch(&world.context, 1, &[first]).expect("landed after a second attempt");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }));
    assert_eq!(subjects(&world, base.as_str()), vec!["moved by someone", "message of mine"]);
    remove(world.root.as_path());
}

/// A branch that moves on every attempt ends in the failure the user is told about, after the attempts there are.
#[test]
fn a_branch_that_never_stops_moving_fails_after_the_attempts() {
    let mut world: World = world("service-land-race-forever");
    let first: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let repo: PathBuf = world.repo.clone();
    let moves: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let counter: Arc<AtomicUsize> = moves.clone();
    world.context.on_objects_written = Some(Box::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=moved again"]);
    }));
    assert_eq!(land_batch(&world.context, 1, &[first]), Err(FailureCode::BranchKeepsChanging));
    assert_eq!(moves.load(Ordering::SeqCst), LANDING_ATTEMPTS);
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0, "no intent is left behind");
    remove(world.root.as_path());
}

/// An amend request for `path` set to `content`: the tree is the tip's tree with that file, written to the object store,
///  and the real index is left as it was.
fn amend_request(world: &World, id: &str, path: &str, content: &str) -> CommitRequest {
    let mut amend: CommitRequest = request(world, id, path, content);
    git(world.repo.as_path(), &["add", "--", path]);
    let tree: String = git_text(world.repo.as_path(), &["write-tree"]);
    git(world.repo.as_path(), &["reset", "--quiet"]);
    amend.amend = true;
    amend.tree = Some(tree);
    return amend;
}

/// An amend replaces the tip: same parents as the replaced commit, the tree it names, its own message and identities, and
///  the index shows nothing staged against it.
#[test]
fn an_amend_replaces_the_tip_with_the_parents_of_the_replaced_commit() {
    let world: World = world("service-land-amend");
    let parent: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let first: CommitRequest = request(&world, "first", "first.txt", "first\n");
    land_batch(&world.context, 1, &[first]).expect("landed");
    let replaced: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let mut amend: CommitRequest = amend_request(&world, "amended", "first.txt", "first, amended\n");
    amend.author = Some(String::from("Old <old@example.invalid> 1700000000 +0200"));
    let outcomes = land_batch(&world.context, 2, &[amend.clone()]).expect("landed");
    let Outcome::Landed { commit } = outcomes[0].clone() else { panic!("{:?}", outcomes[0]) };
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), commit);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", format!("{commit}^").as_str()]), parent, "the replaced commit's parent");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", format!("{commit}^{{tree}}").as_str()]), amend.tree.clone().unwrap_or_default());
    assert_eq!(git_text(world.repo.as_path(), &["log", "-1", "--format=%s|%an|%ad|%cn", "--date=raw"]), "message of amended|Old|1700000000 +0200|T");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "");
    assert_ne!(commit, replaced);
    remove(world.root.as_path());
}

/// Amending a root commit makes a root commit; amending a merge keeps both parents.
#[test]
fn an_amend_of_a_root_commit_and_of_a_merge_keeps_their_parents() {
    let world: World = world("service-land-amend-shapes");
    git(world.repo.as_path(), &["checkout", "--quiet", "--orphan", "fresh"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=orphan root"]);
    let mut amend: CommitRequest = amend_request(&world, "root", "r.txt", "r\n");
    amend.reference = String::from("refs/heads/fresh");
    let outcomes = land_batch(&world.context, 1, &[amend]).expect("landed");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{:?}", outcomes[0]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-list", "--parents", "--max-count=1", "fresh"]).split_whitespace().count(), 1, "still a root commit");
    git(world.repo.as_path(), &["checkout", "--quiet", "main"]);
    git(world.repo.as_path(), &["merge", "--quiet", "--no-ff", "--allow-unrelated-histories", "--message=merge", "fresh"]);
    let parents: String = git_text(world.repo.as_path(), &["rev-list", "--parents", "--max-count=1", "HEAD"]);
    let amend_merge: CommitRequest = amend_request(&world, "merge", "m.txt", "m\n");
    let merged = land_batch(&world.context, 2, &[amend_merge]).expect("landed");
    assert!(matches!(merged[0], Outcome::Landed { .. }), "{:?}", merged[0]);
    let after: String = git_text(world.repo.as_path(), &["rev-list", "--parents", "--max-count=1", "HEAD"]);
    assert_eq!(parents.split_whitespace().skip(1).collect::<Vec<&str>>(), after.split_whitespace().skip(1).collect::<Vec<&str>>());
    remove(world.root.as_path());
}

/// An amend whose base is no longer the tip is not made, and nothing changes: it cannot be built on another tip.
#[test]
fn an_amend_is_not_made_when_the_branch_is_not_at_its_base() {
    let world: World = world("service-land-amend-moved");
    let amend: CommitRequest = amend_request(&world, "late", "late.txt", "late\n");
    git(world.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=moved by someone"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(land_batch(&world.context, 1, &[amend]).expect("answered"), vec![Outcome::BranchMoved]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), moved);
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0);
    remove(world.root.as_path());
}

/// An amend whose branch moves between the check and the branch update is not made either, and leaves no intent.
#[test]
fn an_amend_is_not_made_when_the_branch_moves_under_it() {
    let mut world: World = world("service-land-amend-race");
    let amend: CommitRequest = amend_request(&world, "racing", "racing.txt", "racing\n");
    let repo: PathBuf = world.repo.clone();
    world.context.on_objects_written = Some(Box::new(move || {
        git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=moved by someone"]);
    }));
    assert_eq!(land_batch(&world.context, 1, &[amend]).expect("answered"), vec![Outcome::BranchMoved]);
    assert_eq!(git_text(world.repo.as_path(), &["log", "-1", "--format=%s"]), "moved by someone");
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0, "no intent is left behind");
    remove(world.root.as_path());
}

/// An amend shares no batch: a neighbour would move the tip it requires.
#[test]
fn an_amend_in_a_batch_with_others_is_refused() {
    let world: World = world("service-land-amend-mixed");
    let plain: CommitRequest = request(&world, "plain", "plain.txt", "p\n");
    let amend: CommitRequest = amend_request(&world, "amend", "amend.txt", "a\n");
    assert_eq!(land_batch(&world.context, 1, &[plain, amend]), Err(FailureCode::Rejected));
    remove(world.root.as_path());
}

/// Requests of different branches or repositories cannot share a batch.
#[test]
fn a_mixed_batch_is_refused() {
    let world: World = world("service-land-mixed");
    let one: CommitRequest = request(&world, "one", "one.txt", "1\n");
    let mut other: CommitRequest = request(&world, "two", "two.txt", "2\n");
    other.reference = String::from("refs/heads/other");
    assert_eq!(land_batch(&world.context, 1, &[one, other]), Err(FailureCode::Rejected));
    assert_eq!(land_batch(&world.context, 2, &[]), Err(FailureCode::Rejected));
    remove(world.root.as_path());
}

/// A restart finishes a batch whose branch moved but whose index update was lost, drops one whose branch never moved,
///  and removes a torn one; the finished batch's identities are remembered.
#[test]
fn a_restart_finishes_drops_and_removes_what_a_kill_left() {
    let world: World = world("service-land-recover");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    let new: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    // The kill came after the branch moved and before the index update: the index shows the file as reverted.
    git(world.repo.as_path(), &["reset", "-q", base.as_str(), "--", "landed.txt"]);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]).lines().count(), 2, "staged deletion and untracked file");
    let done: PathBuf = batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION);
    std::fs::rename(&done, batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("reopen");
    let mut never: Intent = parse_intent(std::fs::read_to_string(batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("read").as_str()).expect("parse");
    never.seq = 2;
    never.old = new.clone();
    never.new = "f".repeat(40);
    crate::service_journal::write_intent(world.context.journal.as_path(), &never).expect("second intent");
    std::fs::write(batch_path(world.context.journal.as_path(), 3, OPEN_EXTENSION), "seq=3\nrepos").expect("torn");
    let reports = recover(&world.context);
    let by_seq: Vec<(u64, Recovery)> = reports.iter().map(|report| { return (report.seq, report.decision); }).collect();
    assert!(by_seq.contains(&(1, Recovery::Finish)), "{by_seq:?}");
    assert!(by_seq.contains(&(2, Recovery::Drop)), "{by_seq:?}");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing again");
    assert!(crate::service_journal::landed_commits(world.context.journal.as_path()).contains_key("landed"));
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0);
    remove(world.root.as_path());
}
