//! Controls for landing a batch with real Git: order and content, the index, conflicting and moving branches, hostile
//!  names, and the restart decisions for batches left open.

use super::*;
use crate::service_journal::{DONE_EXTENSION, batch_path, parse_intent};
use crate::service_protocol::{ALLOWED_MODES, Change};
use crate::test_support::{fixture, git, git_context, git_output, git_text, remove, repository};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const COMMITTER: &str = "T <t@example.invalid> 1793535889 +0000";

pub(super) struct World {
    pub(super) root: PathBuf,
    pub(super) repo: PathBuf,
    pub(super) context: LandContext,
}

pub(super) fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let journal: PathBuf = root.join("journal");
    std::fs::create_dir(&journal).expect("journal");
    let context: LandContext = LandContext { git: git_context(), journal, on_objects_written: None, lock_ceiling: crate::wait_ceilings::LOCK_CEILING };
    return World { root, repo, context };
}

/// Lands `requests` as batch `seq` with the index build made to fail after the branch moved: the commits are on the branch, the
///  lock is given up, and the batch stays open for a restart, as after a build that ran out of space.
pub(super) fn land_with_failed_index(world: &World, seq: u64, requests: &[CommitRequest]) {
    crate::service_fault::hooks::fail_at("build-index");
    let outcomes = land_batch(&world.context, seq, requests).expect("the commits land whatever the index does");
    crate::service_fault::hooks::clear_failures();
    assert!(outcomes.iter().all(|outcome| { return matches!(outcome, Outcome::Landed { .. }); }), "{outcomes:?}");
    assert!(batch_path(world.context.journal.as_path(), seq, crate::service_journal::OPEN_EXTENSION).exists(), "the batch stays open");
    assert!(!world.repo.join(".git/index.lock").exists(), "the lock was given up");
}

/// A request that sets `path` to `content`, prepared against the branch as it is now.
pub(super) fn request(world: &World, id: &str, path: &str, content: &str) -> CommitRequest {
    let file: PathBuf = world.repo.join(path);
    std::fs::create_dir_all(file.parent().expect("parent")).expect("directories");
    std::fs::write(&file, content).expect("write");
    let blob: String = git_text(world.repo.as_path(), &["hash-object", "-w", "--", path]);
    return CommitRequest {
        id: String::from(id),
        repository: world.repo.clone(),
        common_dir: world.repo.join(".git"),
        git_dir: world.repo.join(".git"),
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
        captured: 0,
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

/// Two requests on one base that change one path cannot both put their bytes in one batch unless the later arrival captured
///  later: it would replace the earlier member's bytes with older ones. The one that captured earlier is sent back.
#[test]
fn a_member_that_overlaps_an_earlier_one_stays_only_if_it_captured_later() {
    let cases: [(u64, u64, bool); 4] = [(10, 20, true), (20, 10, false), (0, 20, false), (10, 10, false)];
    for (first_stamp, second_stamp, both_land) in cases {
        let world: World = world(format!("service-land-shadow-{first_stamp}-{second_stamp}").as_str());
        let first: CommitRequest = CommitRequest { captured: first_stamp, ..request(&world, "first", "shared.txt", "one\n") };
        let second: CommitRequest = CommitRequest { captured: second_stamp, ..request(&world, "second", "shared.txt", "two\n") };
        let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
        let outcomes = land_batch(&world.context, 1, &[first, second]).expect("landed");
        assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
        if both_land {
            assert!(matches!(outcomes[1], Outcome::Landed { .. }), "{first_stamp} then {second_stamp}: {outcomes:?}");
        } else {
            assert_eq!(outcomes[1], Outcome::RetryPrepare { base: head }, "{first_stamp} then {second_stamp}");
        }
        remove(world.root.as_path());
    }
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
pub(super) fn built_request(world: &World, id: &str, path: &str, parents: &[&str]) -> CommitRequest {
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
///  and an amend whose parents are not the replaced commit's are all refused untouched. A name that is not in the store is
///  content a prune took, which `a_built_commit_that_was_pruned_is_unavailable_not_refused` covers.
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
    let mut wrong_amend: CommitRequest = built_request(&world, "amend", "amend.txt", &[second.as_str()]);
    wrong_amend.amend = true;
    for (label, refused) in [("other parents", on_other_parents), ("a tree", not_a_commit), ("an amend on the base itself", wrong_amend)] {
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
    // The holder moves the branch and lets go when the landing first says that it waits: the landing has then found the lock
    //  held and has not read the branch, whatever the host's speed, with no clock deciding it.
    let held: std::cell::Cell<bool> = std::cell::Cell::new(true);
    let holder = |_: Announce| {
        if held.replace(false) {
            move_branch(world.repo.as_path(), "foreign commit");
            std::fs::remove_file(&lock).expect("the holder lets go");
        }
    };
    let outcomes = land_batch_announcing(&world.context, 1, &[request], &holder).expect("landed after the holder let go");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
    assert_eq!(subjects(&world, base.as_str()), vec!["foreign commit", "message of mine"]);
    assert!(!lock.exists(), "the landing gave the lock up");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing");
    remove(world.root.as_path());
}

/// The index lock is not held while the batch's objects are made: it is asked for only once everything that needs no index is
/// done, so a program that wants the index meanwhile (a `git add`) is not made to wait for the import.
#[test]
fn the_index_lock_is_not_held_while_the_objects_are_made() {
    let mut world: World = world("service-land-lock-after-objects");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let lock: PathBuf = world.repo.join(".git/index.lock");
    let seen_held: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let seen_free: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let (held, free, watched) = (seen_held.clone(), seen_free.clone(), lock.clone());
    world.context.on_objects_written = Some(Box::new(move || {
        let counter: &Arc<AtomicUsize> = if watched.exists() { &held } else { &free };
        counter.fetch_add(1, Ordering::SeqCst);
    }));
    let outcomes = land_batch(&world.context, 1, &[request]).expect("landed");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
    assert_eq!(seen_held.load(Ordering::SeqCst), 0, "the lock stood while the objects were being made");
    assert_eq!(seen_free.load(Ordering::SeqCst), 1, "the seam ran once, with no lock in the Git directory");
    assert!(!lock.exists(), "the landing gave the lock up");
    remove(world.root.as_path());
}

/// A foreign holder that is still there when the landing asks for the lock does not keep the landing from making its
/// objects: they are made first, and the lock is waited for after, so a holder that lets go while they are made costs no wait.
#[test]
fn a_foreign_holder_is_waited_for_after_the_objects_are_made_not_before() {
    let mut world: World = world("service-land-wait-after-objects");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let lock: PathBuf = world.repo.join(".git/index.lock");
    std::fs::write(&lock, "foreign").expect("foreign lock");
    let watched: PathBuf = lock.clone();
    world.context.on_objects_written = Some(Box::new(move || {
        std::fs::remove_file(&watched).expect("the holder lets go while the objects are made");
    }));
    let waited: std::cell::Cell<bool> = std::cell::Cell::new(false);
    let outcomes = land_batch_announcing(&world.context, 1, &[request], &|_: Announce| { waited.set(true); }).expect("landed");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{outcomes:?}");
    assert!(!waited.get(), "the holder was gone before the lock was asked for, so nothing was waited for");
    remove(world.root.as_path());
}

/// A dead owner's index lock (Git's lock PID file names a process that is gone): the landing fails before anything moves, the
///  lock stands, and no intent is left. Nothing is waited for, because nothing will release that lock.
#[test]
fn a_dead_owners_index_lock_changes_nothing() {
    let world: World = world("service-land-dead-owner-lock");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let lock: PathBuf = world.repo.join(".git/index.lock");
    std::fs::write(&lock, "foreign").expect("foreign lock");
    let mut gone = std::process::Command::new("true").spawn().expect("a short process");
    let gone_pid: u32 = gone.id();
    gone.wait().expect("it ends");
    std::fs::write(crate::recovery_files::lock_pid_path(world.repo.join(".git/index").as_path()), format!("pid {gone_pid}\n")).expect("PID file");
    assert_eq!(land_batch(&world.context, 1, &[request]), Err(FailureCode::IndexInUse));
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), head, "the branch did not move");
    assert_eq!(std::fs::read_to_string(&lock).expect("the foreign lock stands"), "foreign");
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0);
    assert_eq!(git_text(world.repo.as_path(), &["for-each-ref", "refs/worktree/cli-git-service/"]), "", "the objects were made, so the scratch branch is removed with the refusal");
    remove(world.root.as_path());
}

/// How long the controls give a landing that must end at its ceiling before they call it hung.
const CEILING_PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);

/// The ceiling the controls give the lock wait: short, so the control ends in a moment.
const SHORT_CEILING: std::time::Duration = std::time::Duration::from_millis(300);

/// Runs `land_batch_announcing` for `requests` on a thread of its own with a ceiling of `SHORT_CEILING`, and returns what it
///  answered and said, or nothing when it was still waiting after `CEILING_PATIENCE`. The thread builds its own context.
fn land_at_the_ceiling(world: &World, requests: Vec<CommitRequest>) -> Option<(Result<Vec<Outcome>, FailureCode>, Vec<Announce>)> {
    let (finished, ended) = std::sync::mpsc::channel();
    let journal: PathBuf = world.context.journal.clone();
    std::thread::spawn(move || {
        let context: LandContext = LandContext { git: git_context(), journal, on_objects_written: None, lock_ceiling: SHORT_CEILING };
        let said: std::sync::Mutex<Vec<Announce>> = std::sync::Mutex::new(Vec::new());
        let answered = land_batch_announcing(&context, 1, requests.as_slice(), &|note: Announce| { said.lock().expect("lock").push(note); });
        let heard: Vec<Announce> = said.lock().expect("lock").clone();
        let _sent = finished.send((answered, heard));
    });
    return ended.recv_timeout(CEILING_PATIENCE).ok();
}

/// A foreign owner nothing proves alive that keeps `index.lock` past the ceiling: the batch is withdrawn before anything moved, the
///  built commit is kept on a side ref that the message names, the person's index and the foreign lock stand, no intent is left,
///  and a restart of the service lands nothing.
#[test]
fn a_foreign_index_lock_at_the_ceiling_withdraws_the_commit_and_keeps_it_on_a_side_ref() {
    let world: World = world("service-land-ceiling-withdraws");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let lock: PathBuf = world.repo.join(".git/index.lock");
    std::fs::write(&lock, "foreign").expect("foreign lock");
    let (answered, said) = land_at_the_ceiling(&world, vec![request]).expect("the landing ended at its ceiling");
    assert_eq!(answered, Err(FailureCode::IndexInUse));
    let kept: String = git_text(world.repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/cli-git/kept/"]);
    assert_eq!(kept, "refs/cli-git/kept/1", "the built commit is kept on a side ref");
    let subject: String = git_text(world.repo.as_path(), &["log", "-1", "--format=%s", "refs/cli-git/kept/1"]);
    assert_eq!(subject, "message of mine", "the side ref holds the commit that was built");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), head, "the branch did not move");
    assert_eq!(std::fs::read_to_string(&lock).expect("the foreign lock stands"), "foreign");
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0, "no intent is left to land it later");
    assert_eq!(git_text(world.repo.as_path(), &["for-each-ref", "refs/worktree/cli-git-service/"]), "", "the scratch branch is gone");
    let texts: Vec<String> = said.iter().filter_map(|note| { return if let Announce::Waiting(text) = note { Some(text.clone()) } else { None }; }).collect();
    let ending: String = texts.join("");
    assert!(ending.contains("refs/cli-git/kept/1") && ending.contains("was not made") && ending.contains("still held"), "{ending}");
    assert!(ending.contains("an owner nothing proves alive"), "the blocker is named with its evidence: {ending}");
    // A restart of the service finds nothing to settle, and the branch still has not moved.
    assert!(recover(&world.context).is_empty(), "nothing is left for a restart to land");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), head);
    remove(world.root.as_path());
}

/// The same with the owner proven alive (Git's lock PID file names this process): the message names it.
#[test]
fn a_proven_holder_at_the_ceiling_is_named_in_the_withdrawal() {
    let world: World = world("service-land-ceiling-proven");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let lock: PathBuf = world.repo.join(".git/index.lock");
    std::fs::write(&lock, "foreign").expect("foreign lock");
    std::fs::write(crate::recovery_files::lock_pid_path(world.repo.join(".git/index").as_path()), format!("pid {}\n", std::process::id())).expect("PID file");
    let (answered, said) = land_at_the_ceiling(&world, vec![request]).expect("the landing ended at its ceiling");
    assert_eq!(answered, Err(FailureCode::IndexInUse));
    let texts: Vec<String> = said.iter().filter_map(|note| { return if let Announce::Waiting(text) = note { Some(text.clone()) } else { None }; }).collect();
    let ending: String = texts.join("");
    assert!(ending.contains(format!("PID {}", std::process::id()).as_str()) && ending.contains("still held"), "{ending}");
    remove(world.root.as_path());
}

/// A Git that does what the real one does, except that the reference update of the landing (`update-ref --stdin`) never returns:
///  after it did its work (`after`), or instead of it (`before`). The mode is in a file, so a control can change it.
fn slow_reference_git(world: &World, mode: &str) -> PathBuf {
    let mode_file: PathBuf = world.root.join("slow-update-ref-mode");
    std::fs::write(&mode_file, mode).expect("mode");
    let program: PathBuf = world.root.join("slow-update-ref-git");
    let script: String = format!(
        "#!/bin/sh\ncase \" $* \" in\n  *\" update-ref --stdin \"*)\n    case \"$(cat '{mode}')\" in\n      after) '{real}' \"$@\"; sleep 600;;\n      *) sleep 600;;\n    esac;;\n  *) exec '{real}' \"$@\";;\nesac\n",
        mode = mode_file.display(),
        real = crate::test_support::REAL_GIT
    );
    std::fs::write(&program, script).expect("script");
    std::fs::set_permissions(&program, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return program;
}

/// The ceiling the reference-update controls give a step: short, so the control ends in moments.
const SHORT_STEP_CEILING: std::time::Duration = std::time::Duration::from_secs(3);

/// A reference update that is ended at its ceiling after it renamed the reference: the branch is at the request's commit, so the
/// outcome is `Landed`, the scratch branch is gone, and no intent is left open.
#[test]
fn a_reference_update_ended_after_it_renamed_the_branch_reports_landed() {
    let world: World = world("service-land-ref-ceiling-after");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let mut context: LandContext = LandContext { git: git_context(), journal: world.context.journal.clone(), on_objects_written: None, lock_ceiling: crate::wait_ceilings::LOCK_CEILING };
    context.git.real_git = slow_reference_git(&world, "after");
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(SHORT_STEP_CEILING)); });
    let outcomes = land_batch(&context, 1, &[request]).expect("the commit landed although the update never returned");
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    let Outcome::Landed { commit } = &outcomes[0] else { panic!("{outcomes:?}") };
    assert_eq!(&git_text(world.repo.as_path(), &["rev-parse", "refs/heads/main"]), commit, "the branch is at the commit the answer names");
    assert_eq!(git_text(world.repo.as_path(), &["for-each-ref", "refs/worktree/cli-git-service/"]), "", "the scratch branch is gone");
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0);
    remove(world.root.as_path());
}

/// A reference update that is ended at its ceiling before it renamed anything: the batch is withdrawn like one that met
/// `LOCK_CEILING`, the branch has not moved, the built commit is kept on a side ref, the line names the step, the lock is given up,
/// and a restart lands nothing.
#[test]
fn a_reference_update_ended_before_it_renamed_the_branch_withdraws_the_commit() {
    let world: World = world("service-land-ref-ceiling-before");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let mut context: LandContext = LandContext { git: git_context(), journal: world.context.journal.clone(), on_objects_written: None, lock_ceiling: crate::wait_ceilings::LOCK_CEILING };
    context.git.real_git = slow_reference_git(&world, "before");
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(SHORT_STEP_CEILING)); });
    let said: std::sync::Mutex<Vec<Announce>> = std::sync::Mutex::new(Vec::new());
    let answered = land_batch_announcing(&context, 1, &[request], &|note: Announce| { said.lock().expect("lock").push(note); });
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    assert_eq!(answered, Err(FailureCode::StepTooSlow));
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), head, "the branch did not move");
    assert_eq!(git_text(world.repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/cli-git/kept/"]), "refs/cli-git/kept/1");
    assert_eq!(git_text(world.repo.as_path(), &["for-each-ref", "refs/worktree/cli-git-service/"]), "", "the scratch branch is gone");
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0, "no intent is left to land it later");
    assert!(!world.repo.join(".git/index.lock").exists(), "the lock was given up");
    let texts: Vec<String> = said.lock().expect("lock").iter().filter_map(|note| { return if let Announce::Waiting(text) = note { Some(text.clone()) } else { None }; }).collect();
    let line: String = texts.join("");
    assert!(line.contains("update-ref") && line.contains("refs/cli-git/kept/1") && line.contains("was not made"), "{line}");
    assert!(recover(&world.context).is_empty(), "nothing is left for a restart to land");
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), head);
    remove(world.root.as_path());
}

/// A Git whose `subcommand` never returns and whose other commands are the real one's.
fn git_that_hangs_on(world: &World, subcommand: &str) -> PathBuf {
    let program: PathBuf = world.root.join(format!("hang-on-{subcommand}-git"));
    let script: String = format!("#!/bin/sh\ncase \" $* \" in\n  *\" {subcommand} \"*) sleep 600;;\n  *) exec '{}' \"$@\";;\nesac\n", crate::test_support::REAL_GIT);
    std::fs::write(&program, script).expect("script");
    std::fs::set_permissions(&program, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return program;
}

/// A read step of the landing that never returns is ended at its ceiling, though it shows no fault: the landing ends with the step
/// named, nothing moved, no intent is open, and the clients are told that this commit was not made.
#[test]
fn a_read_step_ended_at_its_ceiling_ends_the_landing_with_the_step_named() {
    let world: World = world("service-land-read-ceiling");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let head: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let mut context: LandContext = LandContext { git: git_context(), journal: world.context.journal.clone(), on_objects_written: None, lock_ceiling: crate::wait_ceilings::LOCK_CEILING };
    context.git.real_git = git_that_hangs_on(&world, "rev-parse");
    // The landing reads the branch in process when it can, which starts no Git to hang; the readers are off here so that the step
    //  under test is the one that starts Git.
    context.git.overlay.extend(crate::repo_read_hook::switch_off());
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(Some(SHORT_STEP_CEILING)); });
    let said: std::sync::Mutex<Vec<Announce>> = std::sync::Mutex::new(Vec::new());
    let answered = land_batch_announcing(&context, 1, &[request], &|note: Announce| { said.lock().expect("lock").push(note); });
    crate::transaction_git::STEP_CEILING_FOR_THIS_THREAD.with(|cell| { cell.set(None); });
    assert_eq!(answered, Err(FailureCode::StepTooSlow));
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0);
    let line: String = said.lock().expect("lock").iter().filter_map(|note| { return if let Announce::Waiting(text) = note { Some(text.clone()) } else { None }; }).collect::<Vec<String>>().join("");
    assert!(line.contains("rev-parse") && line.contains("was not made"), "{line}");
    remove(world.root.as_path());
}

/// The hold is measured from the placement of the lock to its release, so it covers the branch move and the index build and
/// nothing before them.
#[test]
fn the_hold_covers_the_branch_move_and_the_index_build() {
    let world: World = world("service-land-hold");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let mut timing: LandTiming = LandTiming::default();
    land_batch_timed(&world.context, 1, &[request], &|_| {}, &mut timing).expect("landed");
    assert!(timing.hold >= timing.ref_update + timing.sync, "{timing:?}");
    assert!(timing.hold > std::time::Duration::ZERO, "{timing:?}");
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
    // The index update was lost after the branch moved: the index shows the file as reverted.
    land_with_failed_index(&world, 1, &[landed]);
    let new: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    assert_ne!(new, base);
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]).lines().count(), 2, "staged deletion and untracked file");
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

/// An instance takes over the open batches of its own worktree even when the batch used an index outside the Git
///  directory (`GIT_INDEX_FILE`): the worktree is named by the Git directory the request carried, not by the index's parent.
#[test]
fn an_instance_recovers_a_batch_that_used_an_index_outside_its_git_directory() {
    let world: World = world("service-land-recover-alternate");
    let mut landed: CommitRequest = request(&world, "alternate", "alternate.txt", "alternate\n");
    landed.index = world.root.join("alternate.index");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    let done: PathBuf = batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION);
    std::fs::rename(&done, batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION)).expect("reopen");
    let instance: String = crate::service_instance::instance_of(world.repo.join(".git").as_path());
    let by_seq: Vec<(u64, Recovery)> = recover_instance(&world.context, instance.as_str()).iter().map(|report| { return (report.seq, report.decision); }).collect();
    assert_eq!(by_seq, vec![(1, Recovery::Finish)], "the worktree's own instance found the batch");
    remove(world.root.as_path());
}

/// A batch written before the intent named its Git directory is still found by its repository.
#[test]
fn an_intent_without_a_git_directory_is_found_by_its_repository() {
    let world: World = world("service-land-recover-older");
    let landed: CommitRequest = request(&world, "older", "older.txt", "older\n");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    let done: PathBuf = batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION);
    let open: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    let text: String = std::fs::read_to_string(&done).expect("read");
    let older: String = text.lines().filter(|line| { return !line.starts_with("gitdir="); }).collect::<Vec<&str>>().join("\n") + "\n";
    std::fs::write(&open, older).expect("older intent");
    std::fs::remove_file(&done).expect("reopen");
    let instance: String = crate::service_instance::instance_of(world.repo.join(".git").as_path());
    let by_seq: Vec<(u64, Recovery)> = recover_instance(&world.context, instance.as_str()).iter().map(|report| { return (report.seq, report.decision); }).collect();
    assert_eq!(by_seq, vec![(1, Recovery::Finish)]);
    remove(world.root.as_path());
}

/// A batch written before the intent named its Git directory, with an index outside it, is still found by its repository:
///  the directory the index sits in is not the worktree's Git directory.
#[test]
fn an_older_intent_with_an_index_outside_its_git_directory_is_found_by_its_repository() {
    let world: World = world("service-land-recover-older-alternate");
    let mut landed: CommitRequest = request(&world, "older-alternate", "older-alternate.txt", "older\n");
    landed.index = world.root.join("alternate.index");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    let done: PathBuf = batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION);
    let open: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    let text: String = std::fs::read_to_string(&done).expect("read");
    let older: String = text.lines().filter(|line| { return !line.starts_with("gitdir="); }).collect::<Vec<&str>>().join("\n") + "\n";
    std::fs::write(&open, older).expect("older intent");
    std::fs::remove_file(&done).expect("reopen");
    let instance: String = crate::service_instance::instance_of(world.repo.join(".git").as_path());
    let by_seq: Vec<(u64, Recovery)> = recover_instance(&world.context, instance.as_str()).iter().map(|report| { return (report.seq, report.decision); }).collect();
    assert_eq!(by_seq, vec![(1, Recovery::Finish)], "the worktree's own instance found the older batch");
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
            git_dir: git_dir.clone(),
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
    let refused = land_batch(&world.context, 2, &[switched]).expect("answered");
    assert_eq!(refused, vec![Outcome::BranchMoved]);
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
        context: LandContext { git: git_context(), journal, on_objects_written: None, lock_ceiling: crate::wait_ceilings::LOCK_CEILING },
    };
    git(repo.as_path(), &["branch", "side"]);
    let on_branch: CommitRequest = request(&world, "kept", "kept.txt", "x\n");
    let outcomes = land_batch(&world.context, 1, &[on_branch]).expect("landed");
    assert!(matches!(outcomes[0], Outcome::Landed { .. }), "{:?}", outcomes[0]);
    let switched: CommitRequest = request(&world, "lost", "lost.txt", "x\n");
    let tip: String = git_text(repo.as_path(), &["rev-parse", "main"]);
    git(repo.as_path(), &["switch", "--quiet", "--force", "side"]);
    let refused = land_batch(&world.context, 2, &[switched]).expect("answered");
    assert_eq!(refused, vec![Outcome::BranchMoved]);
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

/// The marks `fast-import` exports name each commit of a batch by its number from one:
/// a number outside the batch is ignored
/// and never indexes past the batch,
/// and a batch with a commit that has no mark has no answer.
#[test]
fn marks_outside_the_batch_are_ignored_and_a_missing_mark_is_no_answer() {
    let first: String = "a".repeat(40);
    let second: String = "b".repeat(40);
    let stray: String = "c".repeat(40);
    let exact: String = format!(":1 {first}\n:2 {second}\n");
    assert_eq!(commits_from_marks(exact.as_str(), 2), Some(vec![first.clone(), second.clone()]));
    let with_strays: String = format!(":0 {stray}\n:1 {first}\n:3 {stray}\n:2 {second}\n");
    assert_eq!(commits_from_marks(with_strays.as_str(), 2), Some(vec![first.clone(), second]));
    let one_mark: String = format!(":1 {first}\n");
    assert_eq!(commits_from_marks(one_mark.as_str(), 2), None);
}

/// What:
///  The starts of a landing for each way a commit is made: the objects Git imports for a request, and the connectivity check
///  of a commit Git built.
/// Why:
///   The owner's rule is that no Git start of the wrapper or the service gets `--quiet` or `-q`. The import carried it
///  (`fast-import`, whose `--quiet` Git 2.56.0 `builtin/fast-import.c` reads as the end of the `--stats` report) and so did
///  the check (`rev-list --objects`, whose option `builtin/rev-list.c` reads as "list nothing" after the same missing-object
///  check). The control lands both and looks at both starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landBatch(recordingGit, 1, [request]); await landBatch(recordingGit, 2, [built]);
/// expect(startsOf('fast-import')).toHaveLength(1); expect(startsOf('rev-list --objects')).toHaveLength(1); expect(quietStarts()).toEqual([]);
/// ```
#[test]
fn the_import_and_the_connectivity_check_of_a_landing_start_without_quiet() {
    let mut world: World = world("service-land-no-quiet");
    let (mut recording, log) = crate::recording_git::recording_context(world.root.as_path());
    // The import hands its stream on standard input, which a request makes in the common directory.
    recording.common_dir = Some(world.repo.join(".git"));
    world.context.git = recording;
    let imported: CommitRequest = request(&world, "imported", "imported.txt", "imported\n");
    land_batch(&world.context, 1, &[imported]).expect("landed");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let built: CommitRequest = built_request(&world, "built", "built.txt", &[base.as_str()]);
    land_batch(&world.context, 2, &[built]).expect("landed");
    let starts: Vec<String> = crate::recording_git::recorded_starts(log.as_path());
    let named = |word: &str| -> usize { return starts.iter().filter(|start| { return start.split(' ').any(|argument| { return argument == word; }); }).count(); };
    assert_eq!(named("fast-import"), 1, "one import, for the request: {starts:?}");
    assert_eq!(named("--objects"), 1, "one connectivity check (rev-list --objects), for the commit Git built: {starts:?}");
    assert_eq!(crate::recording_git::quiet_starts(log.as_path()), Vec::<String>::new(), "no start carries --quiet or -q");
    remove(world.root.as_path());
}

/// A restart finishes the index of a landed batch without starting a Git `reset`: it takes the lock itself, as the first landing
/// did, and the in-process update runs under it.
#[test]
fn a_restart_finishes_the_index_without_a_git_reset() {
    use crate::recording_git::{recorded_starts, recording_context};
    use crate::repo_read_hook::SWITCH_VARIABLE;
    let mut world: World = world("service-land-recover-no-reset");
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_with_failed_index(&world, 1, &[landed]);
    let (mut recording, log) = recording_context(world.root.as_path());
    recording.overlay.retain(|(name, _)| { return name != SWITCH_VARIABLE; });
    world.context.git = recording;
    let reports = recover(&world.context);
    assert!(reports.iter().any(|report| { return report.seq == 1 && report.decision == Recovery::Finish; }));
    let words: Vec<String> = recorded_starts(log.as_path()).iter().map(|start| { return start.split(' ').next().unwrap_or_default().to_string(); }).collect();
    assert!(!words.iter().any(|word| { return word == "reset"; }), "recovery started a Git reset: {words:?}");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing again");
    assert!(!world.repo.join(".git/index.lock").exists());
    remove(world.root.as_path());
}

/// A landing keeps a change another session staged before the landing took the lock, for a path the commit does not name, and
/// starts no Git `reset`.
#[test]
fn a_landing_keeps_an_earlier_stage_of_another_path() {
    use crate::recording_git::{recorded_starts, recording_context};
    use crate::repo_read_hook::SWITCH_VARIABLE;
    let mut world: World = world("service-land-keeps-stage");
    std::fs::write(world.repo.join("elsewhere.txt"), "staged earlier\n").expect("write");
    // An entry recorded in the second of the lock's copy is racily clean and sends the update to Git; this one is older.
    crate::test_support::age(world.repo.join("elsewhere.txt").as_path());
    git(world.repo.as_path(), &["add", "elsewhere.txt"]);
    let (mut recording, log) = recording_context(world.root.as_path());
    recording.overlay.retain(|(name, _)| { return name != SWITCH_VARIABLE; });
    world.context.git = recording;
    let mine: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    land_batch(&world.context, 1, &[mine]).expect("landed");
    let words: Vec<String> = recorded_starts(log.as_path()).iter().map(|start| { return start.split(' ').next().unwrap_or_default().to_string(); }).collect();
    assert!(!words.iter().any(|word| { return word == "reset"; }), "the landing started a Git reset: {words:?}");
    assert_eq!(git_text(world.repo.as_path(), &["diff", "--cached", "--name-only"]), "elsewhere.txt", "the earlier stage is the only difference left");
    remove(world.root.as_path());
}

/// A restart that finds a foreign program holding the live index lock leaves the batch open, the index and the foreign lock as
/// they are, and finishes the batch on a later restart once the lock is free.
#[test]
fn a_restart_leaves_the_batch_open_while_a_foreign_lock_is_held() {
    let world: World = world("service-land-recover-foreign-lock");
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_with_failed_index(&world, 1, &[landed]);
    let open: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    let lock: PathBuf = world.repo.join(".git/index.lock");
    std::fs::write(&lock, "foreign").expect("foreign lock");
    let before: Vec<u8> = std::fs::read(world.repo.join(".git/index")).expect("index");
    let first = recover(&world.context);
    assert!(first.iter().any(|report| { return report.seq == 1 && report.decision == Recovery::Finish; }));
    assert!(open.exists(), "the batch stays open while the lock is held");
    assert_eq!(std::fs::read(world.repo.join(".git/index")).expect("index"), before, "the index was not touched");
    assert_eq!(std::fs::read_to_string(&lock).expect("the foreign lock stands"), "foreign");
    std::fs::remove_file(&lock).expect("the foreign program lets go");
    recover(&world.context);
    assert!(!open.exists(), "the batch is finished once the lock is free");
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing again");
    remove(world.root.as_path());
}

/// A batch whose index was installed and whose lock was given up, and whose service died before the batch was marked done, is
/// not replayed: a change another session staged to a path of the batch in between survives the restart. The service records the
/// completed index update in the journal (`<batch>.indexed`, holding the commit) before it gives the lock up; the control writes
/// that record as the dead service left it.
#[test]
fn a_restart_does_not_replay_an_index_update_that_was_installed_and_unlocked() {
    let world: World = world("service-land-recover-installed");
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    let done: PathBuf = batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION);
    let open: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    let new: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    // The death came after the lock was given up and before the batch was marked done.
    std::fs::rename(&done, &open).expect("reopen");
    std::fs::write(world.context.journal.join("1.indexed"), format!("{new}\n")).expect("the record of the completed index update");
    // Another session stages a new version of the batch's path.
    std::fs::write(world.repo.join("landed.txt"), "edited and staged after the landing\n").expect("edit");
    git(world.repo.as_path(), &["add", "landed.txt"]);
    let reports = recover(&world.context);
    assert!(reports.iter().any(|report| { return report.seq == 1 && report.decision == Recovery::Finish; }));
    assert_eq!(git_text(world.repo.as_path(), &["diff", "--cached", "--name-only"]), "landed.txt", "the later stage survives the restart");
    assert_eq!(git_text(world.repo.as_path(), &["show", ":landed.txt"]), "edited and staged after the landing");
    assert!(done.exists() && !open.exists(), "the batch is finished");
    assert!(!world.context.journal.join("1.indexed").exists(), "and its record is gone");
    remove(world.root.as_path());
}

/// A landing whose conclusion state could not be removed still records the installed index, so the restart that retries the
/// removal does not replay the index update over what another session staged after the lock was given up.
#[test]
fn a_cleanup_that_fails_does_not_make_the_index_update_replayable() {
    use crate::service_journal::{INDEXED_EXTENSION, Indexed, indexed_state};
    let world: World = world("service-land-cleanup-fails");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    let new: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["reset", "-q", base.as_str(), "--", "landed.txt"]);
    let done: PathBuf = batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION);
    let open: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    std::fs::rename(&done, &open).expect("reopen");
    let mut intent: Intent = parse_intent(std::fs::read_to_string(&open).expect("read").as_str()).expect("parse");
    // A conclusion whose scratch directory exists and whose state cannot be reproduced removes nothing and says so.
    let scratch: PathBuf = world.root.join("scratch");
    std::fs::create_dir_all(scratch.join(crate::conclusion_cleanup::CONCLUSION_COPY_DIRECTORY)).expect("a copy directory with no record in it");
    intent.cleanup = Some(crate::service_protocol::Cleanup { git_dir: world.root.join("no-such-git-dir"), scratch, reftable: false });
    crate::service_journal::write_intent(world.context.journal.as_path(), &intent).expect("intent with a cleanup");
    let landing_git: GitContext = world.context.git.clone();
    let lock = try_acquire(world.repo.join(".git/index").as_path(), 1).expect("the lock is free");
    let finished: bool = follow_up(&landing_git, world.repo.as_path(), &intent, lock, world.context.journal.as_path());
    assert!(!finished, "the cleanup did not finish");
    assert_eq!(indexed_state(world.context.journal.as_path(), 1, new.as_str()), Indexed::Installed, "the installed index is recorded all the same");
    assert!(!world.repo.join(".git/index.lock").exists());
    assert_eq!(git_text(world.repo.as_path(), &["status", "--short"]), "", "the index shows the landing");
    // Another session stages a new version; the restart that retries the cleanup leaves it alone.
    std::fs::write(world.repo.join("landed.txt"), "staged after the landing\n").expect("edit");
    git(world.repo.as_path(), &["add", "landed.txt"]);
    recover(&world.context);
    assert_eq!(git_text(world.repo.as_path(), &["show", ":landed.txt"]), "staged after the landing");
    assert!(open.exists(), "the batch stays open while the cleanup fails");
    assert!(world.context.journal.join(format!("1.{INDEXED_EXTENSION}")).exists());
    remove(world.root.as_path());
}

/// A worktree that switches branches while the landing waits for the index lock does not get the other branch's index replaced:
/// the checked-out branch is read again after the lock is taken and before the ref moves, so the landing is answered as moved,
/// the branch stays where it was, and no lock or intent is left.
#[test]
fn a_branch_switched_while_the_lock_is_awaited_stops_the_landing() {
    let world: World = world("service-land-switched");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    git(world.repo.as_path(), &["branch", "other"]);
    let mine: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let lock: PathBuf = world.repo.join(".git/index.lock");
    std::fs::write(&lock, "foreign").expect("foreign lock");
    let intent: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    let outcomes = std::thread::scope(|scope| {
        let landing = scope.spawn(|| { return land_batch(&world.context, 1, std::slice::from_ref(&mine)); });
        // The intent is written after the checks before the wait, so once it exists the landing is waiting for the lock.
        let deadline: Instant = Instant::now() + std::time::Duration::from_secs(10);
        while !intent.exists() && Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(intent.exists(), "the landing reached the wait");
        git(world.repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]);
        std::fs::remove_file(&lock).expect("the foreign program lets go");
        return landing.join().expect("landing");
    });
    assert_eq!(outcomes, Ok(vec![Outcome::BranchMoved]));
    assert_eq!(git_text(world.repo.as_path(), &["rev-parse", "refs/heads/main"]), base, "the branch did not move");
    assert!(!intent.exists(), "no intent is left");
    assert!(!lock.exists(), "no lock is left");
    assert_eq!(git_text(world.repo.as_path(), &["for-each-ref", "refs/worktree/cli-git-service/"]), "", "no scratch branch is left");
    remove(world.root.as_path());
}

/// A restart that finds the worktree on another branch than the batch's does not touch the index: the branch has its commit, the
/// index now belongs to the branch checked out, and the batch is finished.
#[test]
fn a_restart_leaves_the_index_of_another_checked_out_branch_alone() {
    let world: World = world("service-land-recover-switched");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    git(world.repo.as_path(), &["reset", "-q", base.as_str(), "--", "landed.txt"]);
    git(world.repo.as_path(), &["branch", "other", base.as_str()]);
    git(world.repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]);
    let before: Vec<u8> = std::fs::read(world.repo.join(".git/index")).expect("index");
    let done: PathBuf = batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION);
    let open: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    std::fs::rename(&done, &open).expect("reopen");
    recover(&world.context);
    assert_eq!(std::fs::read(world.repo.join(".git/index")).expect("index"), before, "the index of the branch checked out was not touched");
    assert!(done.exists() && !open.exists(), "the batch is finished");
    remove(world.root.as_path());
}

/// A worktree that switches branches after the landing's last check and before its ref move does not get the other branch's
/// index replaced: Git refuses `symref-verify HEAD` in the same transaction as an update of the branch `HEAD` names ("multiple
/// updates for 'HEAD'"), so the head is read again once the branch has moved, under the lock, and the index is left alone.
#[test]
fn a_switch_after_the_last_check_leaves_the_index_of_the_new_branch_alone() {
    let world: World = world("service-land-switch-late");
    git(world.repo.as_path(), &["branch", "other"]);
    let mine: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    let repo: PathBuf = world.repo.clone();
    let before: Vec<u8> = std::fs::read(world.repo.join(".git/index")).expect("index");
    crate::service_fault::hooks::set("before-ref", std::rc::Rc::new(move || { git(repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]); }));
    let outcome = land_batch(&world.context, 1, std::slice::from_ref(&mine)).expect("lands");
    assert!(matches!(outcome.as_slice(), [Outcome::Landed { .. }]), "the commit is on the branch it was made for: {outcome:?}");
    assert_eq!(std::fs::read(world.repo.join(".git/index")).expect("index"), before, "the index of the branch checked out was not touched");
    assert!(!world.repo.join(".git/index.lock").exists(), "no index lock is left");
    assert_eq!(list_intents(world.context.journal.as_path(), OPEN_EXTENSION).len(), 0, "no open intent is left");
    remove(world.root.as_path());
}

/// The writes of a landing are made in the order a restart after a killed process relies on: the index is installed before its
/// completion is recorded, and the done mark is renamed before the record is removed. Their flushes are queued, not waited for
/// (owner, 2026-11-03: "Force at leaving, else background"), so a power loss may lose the end of the sequence, exactly as native
/// Git loses its last writes; a killed process loses nothing, because the page cache holds every write.
#[test]
fn the_installed_index_and_the_finished_batch_are_recorded_in_the_order_a_restart_relies_on() {
    let world: World = world("service-land-durable");
    crate::service_fault::hooks::take_events();
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    let events: Vec<String> = crate::service_fault::hooks::take_events();
    let at = |name: &str| -> usize { return events.iter().position(|event| { return event == name; }).unwrap_or(usize::MAX); };
    // What a killed process leaves is what the page cache holds, in the order the writes were made; the flushes are queued in that
    //  order and the background thread makes them together, so a power cut may lose any suffix of them, as it may for native Git.
    assert!(at("sync-queued:index-dir") < at("indexed-recorded"), "the index is installed before it is recorded: {events:?}");
    assert!(at("indexed-recorded") < at("sync-queued:journal-dir"), "{events:?}");
    assert!(at("sync-queued:journal-dir") < at("remove-indexed"), "the done mark is renamed before the record is removed: {events:?}");
    remove(world.root.as_path());
}

/// The syncs of the landing that are made while the person waits, and the ones it only queues, from its events in order.
fn syncs_of(events: &[String]) -> (Vec<String>, Vec<String>) {
    let waited: Vec<String> = events.iter().filter_map(|event| { return event.strip_prefix("sync-waited:").map(str::to_string); }).collect();
    let queued: Vec<String> = events.iter().filter_map(|event| { return event.strip_prefix("sync-queued:").map(str::to_string); }).collect();
    return (waited, queued);
}

/// The owner's rule for the commit protocol, "Force at leaving, else background": a landing that publishes a commit waits for no
/// sync of the wrapper's own records. Native Git forces nothing by default (a traced native commit makes no sync call), so each
/// sync on the person's wait is a flush that a loaded disk can stall for seconds (production: `journal_ms` up to 40 s, `sync_ms`
/// up to 119 s). The syncs are queued, and the background thread makes them; the list is the records a power cut may lose.
#[test]
fn a_landing_that_publishes_waits_for_no_wrapper_sync() {
    let world: World = world("service-land-syncs");
    crate::service_fault::hooks::take_events();
    let before: u64 = crate::service_durable::BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst);
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    let outcome = land_batch(&world.context, 1, &[landed]).expect("landed");
    assert!(matches!(outcome.as_slice(), [Outcome::Landed { .. }]), "{outcome:?}");
    let events: Vec<String> = crate::service_fault::hooks::take_events();
    let (waited, queued) = syncs_of(events.as_slice());
    // Positive control: the records are synced, only elsewhere, so a landing that stopped reporting its syncs fails here.
    assert!(!queued.is_empty(), "the landing queues its syncs: {events:?}");
    assert_eq!(waited, Vec::<String>::new(), "the person waited for a sync: {events:?}");
    let expected: Vec<&str> = vec!["intent-file", "intent-dir", "lock-marker", "index-dir", "indexed-file", "indexed-dir", "journal-dir"];
    assert_eq!(queued, expected, "the records one landing queues: {events:?}");
    crate::service_durable::drain();
    let made: u64 = crate::service_durable::BACKGROUND_SYNCED.load(std::sync::atomic::Ordering::SeqCst) - before;
    assert!(made >= queued.len() as u64, "the background thread made the {} queued syncs, made {made}", queued.len());
    remove(world.root.as_path());
}

/// Git's own index write follows `core.fsync`, and the in-process writer follows it too: a repository that names the index
/// there (`added`) gets the flush of the new index before it is installed, on the person's wait as native Git makes it, and the
/// wrapper's own records stay queued.
#[test]
fn a_repository_that_names_the_index_in_core_fsync_gets_it_flushed_as_git_does() {
    let world: World = world("service-land-core-fsync");
    git(world.repo.as_path(), &["config", "core.fsync", "added"]);
    crate::service_fault::hooks::take_events();
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    let events: Vec<String> = crate::service_fault::hooks::take_events();
    let (waited, queued) = syncs_of(events.as_slice());
    assert_eq!(waited, vec![String::from("index-publish")], "only the index flush that core.fsync asks for is waited for: {events:?}");
    assert!(queued.contains(&String::from("intent-file")), "{events:?}");
    crate::service_durable::drain();
    remove(world.root.as_path());
}

/// A landing that publishes nothing forces its records before it returns: a foreign index lock that outlasts the ceiling
/// withdraws the commit and keeps it on a side ref, and the intent and the other records the landing wrote are synced, on the
/// wait, because the command reports what it kept and that promise must survive a power cut.
#[test]
fn a_withdrawal_forces_the_records_it_wrote() {
    let world: World = world("service-land-withdrawal-syncs");
    let request: CommitRequest = request(&world, "mine", "mine.txt", "mine\n");
    std::fs::write(world.repo.join(".git/index.lock"), "foreign").expect("foreign lock");
    let journal: PathBuf = world.context.journal.clone();
    let (finished, ended) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        crate::service_fault::hooks::take_events();
        let context: LandContext = LandContext { git: git_context(), journal, on_objects_written: None, lock_ceiling: SHORT_CEILING };
        let answered = land_batch(&context, 1, &[request]);
        let _sent = finished.send((answered, crate::service_fault::hooks::take_events()));
    });
    let (answered, events) = ended.recv_timeout(CEILING_PATIENCE).expect("the landing ended at its ceiling");
    assert_eq!(answered, Err(FailureCode::IndexInUse));
    let (waited, queued) = syncs_of(events.as_slice());
    assert!(waited.contains(&String::from("intent-file")) && waited.contains(&String::from("intent-dir")), "the intent was forced: {events:?}");
    let mut forced: Vec<String> = waited.clone();
    let mut asked: Vec<String> = queued.clone();
    forced.sort();
    asked.sort();
    assert_eq!(forced, asked, "every sync the landing queued was made before it returned: {events:?}");
    remove(world.root.as_path());
}

/// A completion record that cannot be read is not the same as none: a restart that cannot tell whether the index was installed
/// leaves the batch open and the index alone, instead of replaying over whatever was staged since.
#[test]
fn a_restart_that_cannot_read_the_completion_record_leaves_the_index_alone() {
    let world: World = world("service-land-unreadable-record");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    git(world.repo.as_path(), &["reset", "-q", base.as_str(), "--", "landed.txt"]);
    let done: PathBuf = batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION);
    let open: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    std::fs::rename(&done, &open).expect("reopen");
    // A record that is a directory is present and cannot be read.
    std::fs::create_dir(batch_path(world.context.journal.as_path(), 1, crate::service_journal::INDEXED_EXTENSION)).expect("an unreadable record");
    let before: Vec<u8> = std::fs::read(world.repo.join(".git/index")).expect("index");
    recover(&world.context);
    assert_eq!(std::fs::read(world.repo.join(".git/index")).expect("index"), before, "the index was not replayed over");
    assert!(open.exists() && !done.exists(), "the batch stays open");
    assert!(!world.repo.join(".git/index.lock").exists(), "no lock is left");
    remove(world.root.as_path());
}

/// An unwinding panic in what runs after the index is installed leaves the lock in place, as an error does: the installed
/// index must never be left unlocked while the batch can still be replayed.
#[test]
fn a_panic_after_the_installation_leaves_the_lock() {
    let world: World = world("service-land-panic-record");
    let index: PathBuf = world.repo.join(".git/index");
    let lock = try_acquire(index.as_path(), 1).expect("the lock is free");
    let built: PathBuf = world.root.join("built-index");
    std::fs::copy(&index, &built).expect("a copy to install");
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { let _ = lock.release_with_record(built.as_path(), &|| { panic!("the record failed in an unusual way"); }); }));
    assert!(unwound.is_err(), "the panic went through");
    assert!(world.repo.join(".git/index.lock").exists(), "the lock stays for the restart to reclaim");
    remove(world.root.as_path());
}

/// The index of a worktree that switched away is the branch checked out's own and is left alone, but an index kept in another
/// file (`GIT_INDEX_FILE`) belongs to no branch: a restart still finishes its update.
#[test]
fn a_restart_after_a_switch_still_finishes_an_alternate_index() {
    let world: World = world("service-land-recover-alternate");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let alternate: PathBuf = world.repo.join(".git/alternate-index");
    std::fs::copy(world.repo.join(".git/index"), &alternate).expect("an alternate index");
    let mut landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    landed.index = alternate.clone();
    land_with_failed_index(&world, 1, &[landed]);
    let staged = |index: &Path| -> String { return git_with_index(world.repo.as_path(), index, &["ls-files", "--stage", "--", "landed.txt"]); };
    assert!(staged(alternate.as_path()).is_empty(), "the landing's update of the alternate index did not install");
    git(world.repo.as_path(), &["branch", "other", base.as_str()]);
    git(world.repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]);
    let done: PathBuf = batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION);
    let open: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    recover(&world.context);
    assert!(!staged(alternate.as_path()).is_empty(), "the alternate index shows the landing again");
    assert!(done.exists() && !open.exists(), "the batch is finished");
    remove(world.root.as_path());
}

/// The conclusion state a landing leaves is removed whether or not the worktree switched branches; a batch whose cleanup is not
/// done stays open.
#[test]
fn a_restart_after_a_switch_keeps_the_batch_open_while_its_cleanup_is_pending() {
    let world: World = world("service-land-recover-switched-cleanup");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let landed: CommitRequest = request(&world, "landed", "landed.txt", "landed\n");
    land_batch(&world.context, 1, &[landed]).expect("landed");
    git(world.repo.as_path(), &["branch", "other", base.as_str()]);
    git(world.repo.as_path(), &["symbolic-ref", "HEAD", "refs/heads/other"]);
    let done: PathBuf = batch_path(world.context.journal.as_path(), 1, DONE_EXTENSION);
    let open: PathBuf = batch_path(world.context.journal.as_path(), 1, OPEN_EXTENSION);
    std::fs::rename(&done, &open).expect("reopen");
    let mut intent: Intent = parse_intent(std::fs::read_to_string(&open).expect("read").as_str()).expect("parse");
    let scratch: PathBuf = world.root.join("scratch");
    std::fs::create_dir_all(scratch.join(crate::conclusion_cleanup::CONCLUSION_COPY_DIRECTORY)).expect("a copy directory with no record in it");
    intent.cleanup = Some(crate::service_protocol::Cleanup { git_dir: world.root.join("no-such-git-dir"), scratch, reftable: false });
    crate::service_journal::write_intent(world.context.journal.as_path(), &intent).expect("intent with a cleanup");
    recover(&world.context);
    assert!(open.exists() && !done.exists(), "the batch stays open while its cleanup cannot be done");
    remove(world.root.as_path());
}

/// Git in `repository` with `GIT_INDEX_FILE` set to `index`; its output, trimmed.
pub(super) fn git_with_index(repository: &Path, index: &Path, arguments: &[&str]) -> String {
    let output = std::process::Command::new(crate::test_support::REAL_GIT).current_dir(repository).env("GIT_INDEX_FILE", index).args(arguments).output().expect("git");
    assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
    return String::from_utf8_lossy(&output.stdout).trim().to_string();
}

/// The index Git's own `reset` wrote is flushed by the release; the one the in-process update wrote is not flushed twice, because
/// that update flushes its file before it renames it into place and nothing writes to it after.
#[test]
fn the_release_flushes_a_copy_git_wrote_and_not_one_the_update_already_flushed() {
    let mut world: World = world("service-land-flush-once");
    // The repository names the index in `core.fsync`, so the index is flushed on the wait as native Git flushes it.
    git(world.repo.as_path(), &["config", "core.fsync", "added"]);
    world.context.git.overlay.extend(crate::repo_read_hook::switch_off());
    crate::service_fault::hooks::take_events();
    let by_git: CommitRequest = request(&world, "by-git", "by-git.txt", "x\n");
    land_batch(&world.context, 1, &[by_git]).expect("landed");
    let events: Vec<String> = crate::service_fault::hooks::take_events();
    let (waited, _queued) = syncs_of(events.as_slice());
    assert_eq!(waited, vec![String::from("built-index")], "Git's file is flushed once, by the release: {events:?}");
    world.context.git.overlay.retain(|(name, _)| { return name != crate::repo_read_hook::SWITCH_VARIABLE; });
    let in_process: CommitRequest = request(&world, "in-process", "in-process.txt", "x\n");
    land_batch(&world.context, 2, &[in_process]).expect("landed");
    let in_process_events: Vec<String> = crate::service_fault::hooks::take_events();
    let (in_process_waited, in_process_queued) = syncs_of(in_process_events.as_slice());
    assert!(in_process_queued.contains(&String::from("index-dir")), "the landing installed an index: {in_process_events:?}");
    assert_eq!(in_process_waited, vec![String::from("index-publish")], "the update's own flush is not repeated: {in_process_events:?}");
    remove(world.root.as_path());
}
