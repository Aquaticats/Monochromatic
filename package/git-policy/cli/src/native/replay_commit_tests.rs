//! Binary-level replay controls on disposable repositories, through `replay_commit`:
//!  a non-overlapping replay with its event data, an overlapping conflict whose prepared commit cherry-picks onto the base,
//!  an adjacent own edit captured on top of the landed edit, and the reservation granted after the configured lost races.
//!
//!  These run the replay steps against real Git and real repositories; the wrapper's commit path stays refused until integration.

use super::*;
use crate::replay_reservation::{LandingReservation, RESERVATION_REQUEST_FILENAME, reservation_earned, reservation_request_bytes};
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::transaction_owner::{current_transaction_owner, encode_transaction_owner};
use crate::transaction_registry::{OWNER_FILENAME, TRANSACTION_ROOT_NAME, ensure_transaction_root};
use std::path::PathBuf;

/// The empty tree of the SHA-1 object format.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// Revision text of a commit, trimmed.
fn rev(repo: &Path, spec: &str) -> String {
    return git_text(repo, &["rev-parse", spec]).trim().to_string();
}

/// Twelve lines with `second` on line 2 and `third` on line 3 and `eleventh` on line 11.
fn file(second: &str, third: &str, eleventh: &str) -> String {
    return (1..=12)
        .map(|number| { return match number {
            2 => format!("{second}\n"),
            3 => format!("{third}\n"),
            11 => format!("{eleventh}\n"),
            _ => format!("line {number}\n"),
        }; })
        .collect();
}

/// A repository with base, landed on `main`, and prepared on `prepared`; the file `c.txt` changes per the arguments.
struct Scene {
    /// Fixture root.
    root: PathBuf,
    /// Repository.
    repo: PathBuf,
    /// Preparation base.
    base: String,
    /// Target that won.
    current: String,
    /// Prepared commit.
    prepared: String,
    /// Shadow stand-in: the repository's Git directory.
    git_dir: PathBuf,
    /// The Git directory's object store.
    objects: PathBuf,
}

/// Builds the scene; `landed` and `prepared_text` are the `c.txt` bodies the two sides commit.
fn scene(name: &str, landed: &str, prepared_text: &str) -> Scene {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("c.txt"), file("line 2", "line 3", "line 11")).expect("base");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = rev(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    std::fs::write(repo.join("c.txt"), prepared_text).expect("prepared");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=prepared"]);
    let prepared: String = rev(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("c.txt"), landed).expect("landed");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=landed"]);
    let current: String = rev(repo.as_path(), "HEAD");
    let git_dir: PathBuf = repo.join(".git");
    let objects: PathBuf = git_dir.join("objects");
    return Scene {
        root,
        repo,
        base,
        current,
        prepared,
        git_dir,
        objects,
    };
}

/// A transaction directory under the registry of `repo`, with an owner record for this live process.
fn transaction_directory(repo: &Path, id: &str) -> PathBuf {
    let registry: PathBuf = repo.join(".git").join(TRANSACTION_ROOT_NAME);
    ensure_transaction_root(registry.as_path()).expect("registry");
    let directory: PathBuf = registry.join(id);
    std::fs::create_dir_all(&directory).expect("transaction directory");
    let owner = current_transaction_owner(id, "2026-10-10T00:00:00.000Z").expect("owner");
    std::fs::write(directory.join(OWNER_FILENAME), encode_transaction_owner(&owner)).expect("owner record");
    return directory;
}

/// The replay request of a scene: the shadow is the repository's own Git directory in these controls.
fn request<'a>(
    context: &'a GitContext,
    scene: &'a Scene,
    transaction: &'a Path,
    globals: &'a [String],
) -> ReplayRequest<'a> {
    return ReplayRequest {
        context,
        shadow: scene.git_dir.as_path(),
        object_directory: scene.objects.as_path(),
        worktree_root: scene.repo.as_path(),
        git_dir: scene.git_dir.as_path(),
        transaction_directory: transaction,
        replay: 1,
        base: Some(scene.base.as_str()),
        empty_tree: EMPTY_TREE,
        current: scene.current.as_str(),
        prepared: scene.prepared.as_str(),
        signing_key: None,
        global_arguments: globals,
    };
}

/// Non-overlapping hunks replay: the replayed commit holds both edits on the target as its parent,
///  keeps the prepared message, and the event data names the prepared commit, the base, and the target.
#[test]
fn non_overlapping_hunks_replay_onto_the_target() {
    let fixtures = scene(
        "replay-commit-clean",
        &file("LANDED", "line 3", "line 11"),
        &file("line 2", "line 3", "OWN"),
    );
    let transaction: PathBuf = transaction_directory(fixtures.repo.as_path(), "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10");
    let context = git_context();
    let globals: Vec<String> = Vec::new();
    let outcome: ReplayOutcome = replay_commit(&request(&context, &fixtures, transaction.as_path(), &globals)).expect("replay");
    let ReplayOutcome::Replayed(replayed) = outcome else {
        panic!("expected a clean replay");
    };
    assert_eq!(replayed.onto, fixtures.current);
    assert_eq!(replayed.prepared_oid, fixtures.prepared);
    assert_eq!(replayed.from_base.as_deref(), Some(fixtures.base.as_str()));
    assert!(replayed.dropped_headers.is_empty());
    assert_eq!(rev(fixtures.repo.as_path(), &format!("{}^", replayed.oid)), fixtures.current);
    let spec: String = format!("{}:c.txt", replayed.oid);
    let text: String = git_text(fixtures.repo.as_path(), &["cat-file", "-p", spec.as_str()]);
    assert!(text.contains("LANDED\n") && text.contains("OWN\n"), "{text}");
    let message: String = git_text(fixtures.repo.as_path(), &["log", "-1", "--format=%s", replayed.oid.as_str()]);
    assert_eq!(message.trim(), "prepared");
    remove(fixtures.root.as_path());
}

/// Overlapping hunks conflict: the event data names the first landed commit that touched the path,
///  and the prepared commit cherry-picks onto the base, which is what the user needs after the shadow is gone.
#[test]
fn overlapping_hunks_conflict_and_the_prepared_commit_cherry_picks() {
    let fixtures = scene(
        "replay-commit-conflict",
        &file("LANDED", "line 3", "line 11"),
        &file("PREPARED", "line 3", "line 11"),
    );
    let transaction: PathBuf = transaction_directory(fixtures.repo.as_path(), "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10");
    let context = git_context();
    let globals: Vec<String> = Vec::new();
    let outcome: ReplayOutcome = replay_commit(&request(&context, &fixtures, transaction.as_path(), &globals)).expect("replay");
    let ReplayOutcome::Conflict(conflict) = outcome else {
        panic!("expected a conflict");
    };
    assert_eq!(conflict.paths, vec![b"c.txt".to_vec()]);
    assert_eq!(conflict.winning_oid, fixtures.current);
    assert_eq!(conflict.prepared_oid, fixtures.prepared);
    git(fixtures.repo.as_path(), &["checkout", "--quiet", "--detach", fixtures.base.as_str()]);
    git(fixtures.repo.as_path(), &["cherry-pick", "--quiet", conflict.prepared_oid.as_str()]);
    assert!(git_text(fixtures.repo.as_path(), &["log", "-1", "--format=%s"]).contains("prepared"));
    remove(fixtures.root.as_path());
}

/// An own edit next to the landed edit, captured on top of it, is subsumed: the replayed file holds the prepared bytes as they are.
#[test]
fn an_adjacent_own_edit_replays_as_captured() {
    let fixtures = scene(
        "replay-commit-adjacent",
        &file("LANDED", "line 3", "line 11"),
        &file("LANDED", "OWN3", "line 11"),
    );
    let transaction: PathBuf = transaction_directory(fixtures.repo.as_path(), "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10");
    let context = git_context();
    let globals: Vec<String> = Vec::new();
    let outcome: ReplayOutcome = replay_commit(&request(&context, &fixtures, transaction.as_path(), &globals)).expect("replay");
    let ReplayOutcome::Replayed(replayed) = outcome else {
        panic!("expected a clean replay");
    };
    assert_eq!(replayed.subsumed_paths, vec![String::from("c.txt")]);
    let spec: String = format!("{}:c.txt", replayed.oid);
    let replayed_text: String = git_text(fixtures.repo.as_path(), &["cat-file", "-p", spec.as_str()]);
    assert_eq!(replayed_text, file("LANDED", "OWN3", "line 11").trim_end());
    remove(fixtures.root.as_path());
}

/// The starvation reservation is granted on the configured lost race: earlier races only count,
///  each replay of a moved target still replays cleanly, and the request names the count it was granted at.
#[test]
fn the_reservation_is_granted_after_the_configured_lost_races() {
    let fixtures = scene(
        "replay-commit-reservation",
        &file("LANDED", "line 3", "line 11"),
        &file("line 2", "line 3", "OWN"),
    );
    let context = git_context();
    let globals: Vec<String> = Vec::new();
    let transaction: PathBuf = transaction_directory(fixtures.repo.as_path(), "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10");
    let registry: PathBuf = fixtures.repo.join(".git").join(TRANSACTION_ROOT_NAME);
    let configured: u64 = 2;
    let mut reservation: LandingReservation = LandingReservation::open(registry.as_path(), transaction.as_path(), "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10", false);
    let mut granted_at: Option<u64> = None;
    for lost in 1..=configured {
        // Another landing moves the target between this transaction's preparation and its landing.
        std::fs::write(fixtures.repo.join(format!("other-{lost}.txt")), "other\n").expect("other file");
        git(fixtures.repo.as_path(), &["add", "--all"]);
        git(fixtures.repo.as_path(), &["commit", "--quiet", "--message=other landing"]);
        let current: String = rev(fixtures.repo.as_path(), "HEAD");
        let moved = ReplayRequest {
            current: current.as_str(),
            ..request(&context, &fixtures, transaction.as_path(), &globals)
        };
        let outcome: ReplayOutcome = replay_commit(&moved).expect("replay onto the moved target");
        assert!(matches!(outcome, ReplayOutcome::Replayed(_)), "lost race {lost}");
        if reservation_earned(lost, configured) && reservation.reserve(lost).expect("reserve") {
            granted_at = Some(lost);
            break;
        }
    }
    assert_eq!(granted_at, Some(configured));
    let request_bytes: Vec<u8> = std::fs::read(transaction.join(RESERVATION_REQUEST_FILENAME)).expect("request");
    assert_eq!(request_bytes, reservation_request_bytes(configured));
    reservation.release().expect("released");
    remove(fixtures.root.as_path());
}

/// A repository whose base has `c.txt` with the given bytes; `prepared` and `landed` each mutate the tree by a closure
///  run inside the repository on its own branch, and the scene's IDs are returned.
fn custom_scene(name: &str, base_bytes: &[u8], prepared_step: &dyn Fn(&Path), landed_step: &dyn Fn(&Path)) -> Scene {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("c.txt"), base_bytes).expect("base");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = rev(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    prepared_step(repo.as_path());
    git(repo.as_path(), &["commit", "--quiet", "--message=prepared"]);
    let prepared: String = rev(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    landed_step(repo.as_path());
    git(repo.as_path(), &["commit", "--quiet", "--message=landed"]);
    let current: String = rev(repo.as_path(), "HEAD");
    let git_dir: PathBuf = repo.join(".git");
    let objects: PathBuf = git_dir.join("objects");
    return Scene {
        root,
        repo,
        base,
        current,
        prepared,
        git_dir,
        objects,
    };
}

/// Replays a custom scene and returns the outcome.
fn replay_scene(fixtures: &Scene) -> ReplayOutcome {
    let transaction: PathBuf = transaction_directory(fixtures.repo.as_path(), "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10");
    let context = git_context();
    let globals: Vec<String> = Vec::new();
    return replay_commit(&request(&context, fixtures, transaction.as_path(), &globals)).expect("replay");
}

/// An edit next to the landed edit, without the landed edit itself, is not subsumed and the merge conflicts.
#[test]
fn an_adjacent_edit_without_the_landed_edit_conflicts() {
    let fixtures = scene(
        "replay-commit-adjacent-conflict",
        &file("LANDED", "line 3", "line 11"),
        &file("line 2", "OWN3", "line 11"),
    );
    let ReplayOutcome::Conflict(conflict) = replay_scene(&fixtures) else {
        panic!("expected a conflict");
    };
    assert_eq!(conflict.paths, vec![b"c.txt".to_vec()]);
    remove(fixtures.root.as_path());
}

/// A landed mode change with prepared content merges three-way: the mode lands, and so does the content.
#[test]
fn a_landed_mode_change_merges_three_way_even_when_the_content_is_subsumed() {
    let fixtures = custom_scene(
        "replay-commit-mode",
        b"line 1\nline 2\n",
        &|repo: &Path| {
            std::fs::write(repo.join("c.txt"), b"line 1\nPREPARED\n").expect("prepared content");
            git(repo, &["add", "c.txt"]);
        },
        &|repo: &Path| {
            git(repo, &["update-index", "--chmod=+x", "c.txt"]);
        },
    );
    let ReplayOutcome::Replayed(replayed) = replay_scene(&fixtures) else {
        panic!("expected a clean replay");
    };
    let listing: String = git_text(fixtures.repo.as_path(), &["ls-tree", replayed.tree.as_str(), "c.txt"]);
    assert!(listing.starts_with("100755 blob "), "{listing}");
    let spec: String = format!("{}:c.txt", replayed.tree);
    assert_eq!(git_text(fixtures.repo.as_path(), &["cat-file", "-p", spec.as_str()]), "line 1\nPREPARED");
    remove(fixtures.root.as_path());
}

/// A landed modification against a prepared deletion conflicts, and a landed deletion against a prepared modification conflicts.
#[test]
fn delete_against_modify_conflicts_either_way_round() {
    let modify = |repo: &Path| {
        std::fs::write(repo.join("c.txt"), b"line 1\nMODIFIED\n").expect("modified");
        git(repo, &["add", "c.txt"]);
    };
    let delete = |repo: &Path| {
        git(repo, &["rm", "--quiet", "c.txt"]);
    };
    let landed_modifies = custom_scene("replay-commit-delete-prepared", b"line 1\nline 2\n", &delete, &modify);
    let ReplayOutcome::Conflict(first) = replay_scene(&landed_modifies) else {
        panic!("a prepared deletion against a landed modification must conflict");
    };
    assert_eq!(first.paths, vec![b"c.txt".to_vec()]);
    remove(landed_modifies.root.as_path());
    let landed_deletes = custom_scene("replay-commit-delete-landed", b"line 1\nline 2\n", &modify, &delete);
    let ReplayOutcome::Conflict(second) = replay_scene(&landed_deletes) else {
        panic!("a landed deletion against a prepared modification must conflict");
    };
    assert_eq!(second.paths, vec![b"c.txt".to_vec()]);
    remove(landed_deletes.root.as_path());
}

/// A path both sides deleted is subsumed: the replayed tree does not hold it.
#[test]
fn a_deletion_on_both_sides_is_subsumed() {
    let delete = |repo: &Path| {
        git(repo, &["rm", "--quiet", "c.txt"]);
    };
    let fixtures = custom_scene("replay-commit-delete-both", b"line 1\nline 2\n", &delete, &delete);
    let ReplayOutcome::Replayed(replayed) = replay_scene(&fixtures) else {
        panic!("expected a clean replay");
    };
    assert_eq!(replayed.subsumed_paths, vec![String::from("c.txt")]);
    let listing: String = git_text(fixtures.repo.as_path(), &["ls-tree", replayed.tree.as_str(), "c.txt"]);
    assert!(listing.is_empty(), "{listing}");
    remove(fixtures.root.as_path());
}
