//! Controls for landing a batch with real Git: order and content, the index, conflicting and moving branches, hostile
//!  names, and the restart decisions for batches left open.

use super::*;
use crate::service_journal::{DONE_EXTENSION, batch_path, parse_intent};
use crate::service_protocol::{ALLOWED_MODES, Change};
use crate::test_support::{fixture, git, git_context, git_output, git_text, remove, repository};
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
    let context: LandContext = LandContext { git: git_context(), journal, on_objects_written: None, index_wait: std::time::Duration::from_secs(5) };
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
        commit: None,
        cleanup: None,
        index: world.repo.join(".git/index"),
    };
}

/// Moves the branch the way a plain `update-ref` does, which takes no index lock: the landing holds that lock, so a commit by
///  `git commit` cannot happen while it works.
fn move_branch(repo: &Path, message: &str) {
    let tree: String = git_text(repo, &["rev-parse", "HEAD^{tree}"]);
    let head: String = git_text(repo, &["rev-parse", "HEAD"]);
    let made: String = git_text(repo, &["commit-tree", tree.as_str(), "-p", head.as_str(), "-m", message]);
    git(repo, &["update-ref", "refs/heads/main", made.as_str()]);
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
            move_branch(repo.as_path(), "moved by someone");
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
        move_branch(repo.as_path(), "moved again");
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
        move_branch(repo.as_path(), "moved by someone");
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

/// A request carrying a commit Git built: `commit-tree` makes it on `parents` with the tip's tree plus `path`, and the
///  request names it whole.
fn built_request(world: &World, id: &str, path: &str, parents: &[&str]) -> CommitRequest {
    let mut built: CommitRequest = request(world, id, path, format!("{id}\n").as_str());
    git(world.repo.as_path(), &["add", "--", path]);
    let tree: String = git_text(world.repo.as_path(), &["write-tree"]);
    git(world.repo.as_path(), &["reset", "--quiet"]);
    let mut arguments: Vec<String> = vec![String::from("commit-tree"), tree, String::from("-m"), format!("typed {id}")];
    for parent in parents {
        arguments.push(String::from("-p"));
        arguments.push((*parent).to_string());
    }
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    built.commit = Some(git_text(world.repo.as_path(), words.as_slice()));
    return built;
}

/// A commit Git built lands as it is while the branch is at its base: the branch points at that very commit, nothing else
///  is made, and the index shows nothing staged.
#[test]
fn a_commit_git_built_lands_as_it_is_while_the_branch_is_at_its_base() {
    let world: World = world("service-land-built");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let commit: String = built.commit.clone().unwrap_or_default();
    let outcomes = land_batch(&world.context, 1, &[built]).expect("landed");
    assert_eq!(outcomes, vec![Outcome::Landed { commit: commit.clone() }]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), commit);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "");
    assert!(batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION).exists(), "journaled and finished");
    remove(world.root.as_path());
}

/// A commit Git built is never moved onto another tip: with the branch away from its base it is not landed, and the branch stays.
#[test]
fn a_commit_git_built_is_not_landed_when_the_branch_is_not_at_its_base() {
    let world: World = world("service-land-built-moved");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    git(world.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=moved by someone"]);
    let moved: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(land_batch(&world.context, 1, &[built]).expect("answered"), vec![Outcome::BranchMoved]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), moved);
    remove(world.root.as_path());
}

/// The service lands only what the request's rule allows: a commit made on other parents, a name that is not a commit, a
///  name that is not in the store, and an amend whose parents are not the replaced commit's are all refused untouched.
#[test]
fn a_commit_git_built_is_checked_before_the_branch_moves() {
    let world: World = world("service-land-built-checked");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=second"]);
    let second: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let mut on_other_parents: CommitRequest = built_request(&world, "other", "other.txt", &[base.as_str()]);
    on_other_parents.base = second.clone();
    let mut not_a_commit: CommitRequest = built_request(&world, "tree", "tree.txt", &[second.as_str()]);
    not_a_commit.commit = Some(git_text(world.repo.as_path(), &["rev-parse", "HEAD^{tree}"]));
    let mut absent: CommitRequest = built_request(&world, "absent", "absent.txt", &[second.as_str()]);
    absent.commit = Some("1".repeat(40));
    let mut wrong_amend: CommitRequest = built_request(&world, "amend", "amend.txt", &[second.as_str()]);
    wrong_amend.amend = true;
    for (label, refused) in [("other parents", on_other_parents), ("a tree", not_a_commit), ("an absent name", absent), ("an amend on the base itself", wrong_amend)] {
        assert_eq!(land_batch(&world.context, 1, &[refused]), Err(FailureCode::Rejected), "{label}");
        assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), second, "{label}: the branch did not move");
    }
    remove(world.root.as_path());
}

/// A foreign program holds the index lock while it moves the branch, as Git's own commit does: the landing waits for it, builds
///  on what it left, and installs an index that shows the landing, not the state before the foreign commit.
#[test]
fn a_landing_waits_for_a_foreign_index_lock_and_builds_on_what_the_holder_left() {
    let world: World = world("service-land-foreign-lock");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let lock: PathBuf = world.repo.join(".git/index.lock");
    std::fs::write(&lock, "foreign").expect("foreign lock");
    let repo: PathBuf = world.repo.clone();
    let holder_lock: PathBuf = lock.clone();
    let holder = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1_700));
        move_branch(repo.as_path(), "foreign commit");
        std::fs::remove_file(holder_lock).expect("the holder lets go");
    });
    let outcomes = land_batch(&world.context, 1, &[request]).expect("landed after the holder let go");
    holder.join().expect("thread");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
    assert_eq!(subjects(&world, base.as_str()), vec!["foreign commit", "message of mine"]);
    assert!(!lock.exists(), "the landing gave the lock up");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing");
    remove(world.root.as_path());
}

/// A holder that outlives the limit: the landing fails before anything moves, the holder's lock stands, and no intent is left.
#[test]
fn a_foreign_index_lock_that_outlives_the_limit_changes_nothing() {
    let mut world: World = world("service-land-foreign-lock-limit");
    world.context.index_wait = std::time::Duration::from_millis(400);
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let lock: PathBuf = world.repo.join(".git/index.lock");
    std::fs::write(&lock, "foreign").expect("foreign lock");
    assert_eq!(land_batch(&world.context, 1, &[request]), Err(FailureCode::IndexInUse));
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), head, "the branch did not move");
    assert_eq!(std::fs::read_to_string(&lock).expect("the foreign lock stands"), "foreign");
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0);
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

/// A merge conclusion has the branch tip first and the merged commits after it: the extra parents are allowed, and the
///  index takes the commit's paths from the repository, not from a list in the request.
#[test]
fn a_commit_git_built_with_extra_parents_lands_and_updates_the_index_from_the_repository() {
    let world: World = world("service-land-built-merge");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "-b", "side"]);
    std::fs::write(world.repo.join("side.txt"), "side\n").expect("side");
    git(world.repo.as_path(), &["add", "side.txt"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=side"]);
    let side: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "main"]);
    let mut merged: CommitRequest = built_request(&world, "merge", "side.txt", &[base.as_str(), side.as_str()]);
    merged.changes = Vec::new();
    let commit: String = merged.commit.clone().unwrap_or_default();
    assert_eq!(land_batch(&world.context, 1, &[merged]).expect("landed"), vec![Outcome::Landed { commit: commit.clone() }]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-list", "--parents", "--max-count=1", "HEAD"]), format!("{commit} {base} {side}"));
    assert_eq!(git_text(world.repo.as_path(), &["diff", "--cached", "--name-status"]), "", "the index shows the commit");
    remove(world.root.as_path());
}

/// A commit whose first parent is not the branch tip is refused whatever else it has.
#[test]
fn a_commit_git_built_whose_first_parent_is_not_the_tip_is_refused_even_with_the_tip_as_a_later_parent() {
    let world: World = world("service-land-built-first-parent");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=other"]);
    let other: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let mut wrong_order: CommitRequest = built_request(&world, "order", "order.txt", &[base.as_str(), other.as_str()]);
    wrong_order.base = other.clone();
    assert_eq!(land_batch(&world.context, 1, &[wrong_order]), Err(FailureCode::Rejected));
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), other);
    remove(world.root.as_path());
}

/// A branch that does not exist yet is prepared against the all-zero name: the commit has no parent and lands only while
///  the branch is still absent.
#[test]
fn a_commit_on_a_branch_that_does_not_exist_lands_once_and_never_over_a_branch_that_appeared() {
    let world: World = world("service-land-unborn");
    let zero: String = "0".repeat(40);
    let mut first: CommitRequest = built_request(&world, "root", "root.txt", &[]);
    first.reference = String::from("refs/heads/fresh");
    first.base = zero;
    let commit: String = first.commit.clone().unwrap_or_default();
    let mut raced: CommitRequest = first.clone();
    raced.id = String::from("raced");
    let mut with_parent: CommitRequest = first.clone();
    with_parent.id = String::from("parent");
    with_parent.commit = built_request(&world, "child", "child.txt", &[commit.as_str()]).commit;
    // The worktree is on the branch that does not exist yet, as it is after `git switch --orphan` or in a new repository.
    git(world.repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/fresh"]);
    assert_eq!(land_batch(&world.context, 1, &[with_parent]), Err(FailureCode::Rejected), "a root commit has no parent");
    assert_eq!(land_batch(&world.context, 2, &[first]).expect("landed"), vec![Outcome::Landed { commit: commit.clone() }]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/heads/fresh"]), commit);
    assert_eq!(land_batch(&world.context, 3, &[raced]).expect("answered"), vec![Outcome::BranchMoved], "the branch exists now");
    remove(world.root.as_path());
}

/// A detached head is moved as `HEAD` itself, only while it is at the base, and never moves a branch it became.
#[test]
fn a_detached_head_is_moved_by_a_compare_that_does_not_follow_a_branch() {
    let world: World = world("service-land-detached");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "--detach"]);
    let mut built: CommitRequest = built_request(&world, "loose", "loose.txt", &[base.as_str()]);
    built.reference = String::from("HEAD");
    let commit: String = built.commit.clone().unwrap_or_default();
    // The head became a branch after the commit was prepared: the compare fails and the branch stays where it was.
    git(world.repo.as_path(), &["checkout", "--quiet", "main"]);
    assert_eq!(land_batch(&world.context, 1, &[built.clone()]).expect("answered"), vec![Outcome::BranchMoved], "a head that became a branch is not landed on");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), base, "the branch did not move");
    assert_eq!(git_text(world.repo.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/main", "the head still names the branch");
    git(world.repo.as_path(), &["checkout", "--quiet", "--detach"]);
    let outcomes = land_batch(&world.context, 2, &[built]).expect("landed");
    assert_eq!(outcomes, vec![Outcome::Landed { commit: commit.clone() }]);
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), commit);
    assert!(!git_output(world.repo.as_path(), &["symbolic-ref", "--quiet", "HEAD"]).status.success(), "the head is still detached");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), base);
    remove(world.root.as_path());
}

/// What a landing leaves of the index lock, and of the branch and the index, after it refused: nothing at all.
fn assert_nothing_changed(world: &World, branch: &str, expected_tip: &str, index_before: &[u8], what: &str) {
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", branch]), expected_tip, "{what}: the branch did not move");
    assert_eq!(std::fs::read(world.repo.join(".git/index")).expect("index"), index_before, "{what}: the index is as it was");
    assert!(!world.repo.join(".git/index.lock").exists(), "{what}: no index lock is left");
}

/// A commit prepared on a branch is not landed once the worktree's head names another branch: the branch, the index and the
///  other branch stay as they are, and every request of the batch is told the branch moved.
#[test]
fn a_commit_is_not_landed_once_the_worktree_switched_to_another_branch() {
    let world: World = world("service-land-switched");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["switch", "--quiet", "--create", "side"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=side work"]);
    let side_tip: String = git_text(world.repo.as_path(), &["rev-parse", "side"]);
    git(world.repo.as_path(), &["switch", "--quiet", "main"]);
    let batch: Vec<CommitRequest> = ["one", "two"].iter().map(|id| { return request(&world, id, format!("{id}.txt").as_str(), "x\n"); }).collect();
    git(world.repo.as_path(), &["switch", "--quiet", "--force", "side"]);
    let index_before: Vec<u8> = std::fs::read(world.repo.join(".git/index")).expect("index");
    let outcomes = land_batch(&world.context, 1, &batch).expect("answered");
    assert_eq!(outcomes, vec![Outcome::BranchMoved, Outcome::BranchMoved], "each request is answered");
    assert_nothing_changed(&world, "main", base.as_str(), index_before.as_slice(), "main");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "side"]), side_tip, "the branch switched to did not move");
    assert_eq!(git_text(world.repo.as_path(), &["symbolic-ref", "HEAD"]), "refs/heads/side");
    remove(world.root.as_path());
}

/// A worktree whose head went back to the branch lands the commit: the check refuses a switch, not a branch.
#[test]
fn a_commit_lands_when_the_worktree_is_on_its_branch() {
    let world: World = world("service-land-on-branch");
    git(world.repo.as_path(), &["branch", "side"]);
    git(world.repo.as_path(), &["switch", "--quiet", "side"]);
    git(world.repo.as_path(), &["switch", "--quiet", "main"]);
    let sent: CommitRequest = request(&world, "back", "back.txt", "x\n");
    let outcomes = land_batch(&world.context, 1, &[sent]).expect("landed");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{:?}", outcomes[0]);
    remove(world.root.as_path());
}

/// A linked worktree keeps its head in its own Git directory: a switch there is found, and a worktree still on its branch lands.
#[test]
fn a_linked_worktrees_head_is_read_from_its_own_git_directory() {
    let world: World = world("service-land-linked");
    let linked: PathBuf = world.root.join("linked");
    git(world.repo.as_path(), &["worktree", "add", "--quiet", "-b", "wt", linked.to_str().expect("UTF-8")]);
    git(world.repo.as_path(), &["branch", "other"]);
    let git_dir: PathBuf = world.repo.join(".git/worktrees/linked");
    let linked_request = |id: &str| -> CommitRequest {
        std::fs::write(linked.join(format!("{id}.txt")), "x\n").expect("write");
        let blob: String = git_text(linked.as_path(), &["hash-object", "-w", "--", format!("{id}.txt").as_str()]);
        return CommitRequest {
            repository: linked.clone(),
            reference: String::from("refs/heads/wt"),
            base: git_text(linked.as_path(), &["rev-parse", "HEAD"]),
            changes: vec![Change { path: format!("{id}.txt"), mode: ALLOWED_MODES[0], blob: Some(blob) }],
            index: git_dir.join("index"),
            ..request(&world, id, "unused.txt", "x\n")
        };
    };
    let on_branch: CommitRequest = linked_request("kept");
    let outcomes = land_batch(&world.context, 1, &[on_branch]).expect("landed");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{:?}", outcomes[0]);
    let switched: CommitRequest = linked_request("lost");
    let wt_tip: String = git_text(linked.as_path(), &["rev-parse", "refs/heads/wt"]);
    git(linked.as_path(), &["switch", "--quiet", "--force", "other"]);
    let outcomes = land_batch(&world.context, 2, &[switched]).expect("answered");
    assert_eq!(outcomes, vec![Outcome::BranchMoved]);
    assert_eq!(git_text(linked.as_path(), &["rev-parse", "refs/heads/wt"]), wt_tip, "the linked worktree's branch did not move");
    remove(world.root.as_path());
}

/// A repository that keeps its references in reftable tables has only a placeholder in `HEAD`, and the switch is found anyway.
#[test]
fn a_switch_is_found_in_a_reftable_repository() {
    let root: PathBuf = fixture("service-land-reftable");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", "--ref-format=reftable"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("journal");
    let world: World = World {
        root: root.clone(),
        repo: repo.clone(),
        context: LandContext { git: git_context(), journal, on_objects_written: None, index_wait: std::time::Duration::from_secs(5) },
    };
    git(repo.as_path(), &["branch", "side"]);
    let on_branch: CommitRequest = request(&world, "kept", "kept.txt", "x\n");
    let outcomes = land_batch(&world.context, 1, &[on_branch]).expect("landed");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{:?}", outcomes[0]);
    let switched: CommitRequest = request(&world, "lost", "lost.txt", "x\n");
    let tip: String = git_text(repo.as_path(), &["rev-parse", "main"]);
    git(repo.as_path(), &["switch", "--quiet", "--force", "side"]);
    let outcomes = land_batch(&world.context, 2, &[switched]).expect("answered");
    assert_eq!(outcomes, vec![Outcome::BranchMoved]);
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "main"]), tip, "the branch did not move");
    remove(root.as_path());
}

/// Removes the unreferenced objects the way `git gc --prune=now` does, between a request's preparation and its landing.
fn prune_now(world: &World) {
    git(world.repo.as_path(), &["prune", "--expire=now"]);
}

/// What a landing that found the prepared content gone leaves of the repository: the branch where it was, no index lock, and
///  no crash report of the program that imports the objects.
fn assert_refused_cleanly(world: &World, base: &str, what: &str) {
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "main"]), base, "{what}: the branch did not move");
    assert!(!world.repo.join(".git/index.lock").exists(), "{what}: no index lock is left");
    let reports: Vec<String> = std::fs::read_dir(world.repo.join(".git"))
        .expect("git directory")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name.starts_with("fast_import_crash"); })
        .collect();
    assert!(reports.is_empty(), "{what}: left {reports:?}");
}

/// A blob the request names that `git gc --prune=now` removed after the preparation is found before anything is imported, and
///  the landing answers that it could not be done, so the client prepares the commit again.
#[test]
fn a_request_whose_blob_was_pruned_is_refused_before_anything_is_imported() {
    let world: World = world("service-land-pruned-blob");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let sent: CommitRequest = request(&world, "pruned", "pruned.txt", "x\n");
    prune_now(&world);
    assert_eq!(land_batch(&world.context, 1, &[sent]), Err(FailureCode::Unavailable));
    assert_refused_cleanly(&world, base.as_str(), "blob");
    remove(world.root.as_path());
}

/// An amend whose tree was removed is refused the same way.
#[test]
fn an_amend_whose_tree_was_pruned_is_refused_before_anything_is_imported() {
    let world: World = world("service-land-pruned-tree");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let mut amend: CommitRequest = request(&world, "amend", "amend.txt", "x\n");
    git(world.repo.as_path(), &["add", "--", "amend.txt"]);
    amend.tree = Some(git_text(world.repo.as_path(), &["write-tree"]));
    git(world.repo.as_path(), &["reset", "--quiet"]);
    amend.amend = true;
    amend.changes = Vec::new();
    prune_now(&world);
    assert_eq!(land_batch(&world.context, 1, &[amend]), Err(FailureCode::Unavailable));
    assert_refused_cleanly(&world, base.as_str(), "tree");
    remove(world.root.as_path());
}

/// A commit Git built that was removed whole is the removal of prepared content, not a request the service refuses as unsafe.
#[test]
fn a_built_commit_that_was_pruned_is_unavailable_not_refused() {
    let world: World = world("service-land-pruned-commit");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    prune_now(&world);
    assert_eq!(land_batch(&world.context, 1, &[built]), Err(FailureCode::Unavailable));
    assert_refused_cleanly(&world, base.as_str(), "commit");
    remove(world.root.as_path());
}

/// A commit Git built whose commit object survived but whose blob did not (a prune that was still deleting) is never put on the
///  branch: Git moves a branch to a commit without looking at what the commit holds.
#[test]
fn a_built_commit_with_a_missing_blob_is_never_put_on_the_branch() {
    let world: World = world("service-land-partly-pruned");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "typed", "typed.txt", &[base.as_str()]);
    let blob: String = git_text(world.repo.as_path(), &["rev-parse", "--verify", format!("{}:typed.txt", built.commit.clone().unwrap_or_default()).as_str()]);
    std::fs::remove_file(world.repo.join(".git/objects").join(&blob[..2]).join(&blob[2..])).expect("the blob is loose");
    assert_eq!(land_batch(&world.context, 1, &[built]), Err(FailureCode::Unavailable));
    assert_refused_cleanly(&world, base.as_str(), "partial");
    remove(world.root.as_path());
}
